//! Chạy cùng một bộ test hợp đồng trên adapter bộ nhớ và adapter PostgreSQL.
//!
//! `cargo run` in kịch bản mẫu; `cargo run -- --verify` kiểm các giá trị kỳ vọng trong bài.

use std::process::ExitCode;
use std::sync::Arc;

use anyhow::{Result, bail};
use ledger_core::{AccountId, TripCharge, default_policy, fare_for_km};
use ledger_pg::PgRepository;
use ledger_pg::harness::LabDb;
use ledger_port::{LedgerRepository, MemoryRepository, contract, post_trip_capture};

fn trip() -> TripCharge {
    TripCharge {
        trip_id: "TN-0001".to_string(),
        customer: AccountId::new("customer:CU-001").expect("hợp lệ"),
        driver: AccountId::new("driver:DR-001").expect("hợp lệ"),
        fare: fare_for_km(5),
    }
}

// Cùng một hàm generic chạy trên mọi adapter: service không biết đang nói chuyện với ai.
async fn demo<R: LedgerRepository>(name: &str, repo: &R) -> Result<()> {
    println!("== {name} ==");
    let policy = default_policy();
    for _ in 0..2 {
        let out = post_trip_capture(repo, &trip(), &policy).await?;
        println!(
            "post_trip_capture(TN-0001) -> {} replayed={}",
            out.id, out.replayed
        );
    }
    for account in ["customer:CU-001", "platform:revenue", "driver:DR-001"] {
        let balance = repo.balance(&AccountId::new(account)?).await?;
        println!("  {account:<18} {balance}");
    }
    Ok(())
}

async fn contract_on<R>(name: &str, repo: Arc<R>) -> Result<Vec<&'static str>>
where
    R: LedgerRepository + 'static,
{
    let mut passed = match contract::run(Arc::clone(&repo)).await {
        Ok(p) => p,
        Err(e) => bail!("{name}: hợp đồng hỏng: {e}"),
    };
    match contract::run_concurrent(repo).await {
        Ok(case) => passed.push(case),
        Err(e) => bail!("{name}: ca đồng thời hỏng: {e}"),
    }
    println!("== hợp đồng trên {name} ==");
    for case in &passed {
        println!("  ok  {case}");
    }
    Ok(passed)
}

async fn run(verify: bool) -> Result<()> {
    // Phần 1: kịch bản mẫu trên từng adapter (mỗi adapter một sổ mới).
    demo("adapter bộ nhớ", &MemoryRepository::with_chart()).await?;
    let lab = LabDb::setup().await?;
    let shown = demo(
        "adapter PostgreSQL (sqlx)",
        &PgRepository::new(lab.pool.clone()),
    )
    .await;
    lab.cleanup().await?;
    shown?;

    // Phần 2: cùng bộ hợp đồng, hai adapter.
    let memory = contract_on("bộ nhớ", Arc::new(MemoryRepository::with_chart())).await?;
    let lab = LabDb::setup().await?;
    let schema = lab.schema().to_string();
    let url = std::env::var("TOI_NOI_PG_URL")?;
    let pg = contract_on("PostgreSQL", Arc::new(PgRepository::new(lab.pool.clone()))).await;
    lab.cleanup().await?;
    let pg = pg?;

    if verify {
        if memory != pg {
            bail!("hai adapter chạy khác danh sách ca: {memory:?} vs {pg:?}");
        }
        if memory.len() != 8 {
            bail!("cần 8 ca hợp đồng, có {}", memory.len());
        }
        if LabDb::schema_exists(&url, &schema).await? {
            bail!("schema {schema} vẫn còn sau cleanup");
        }
        println!("VERIFY OK");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    let verify = std::env::args().any(|a| a == "--verify");
    match run(verify).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("LỖI: {e:#}");
            ExitCode::FAILURE
        }
    }
}
