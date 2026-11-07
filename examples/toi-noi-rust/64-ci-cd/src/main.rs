//! CLI của lab 64. `cargo run` in những gì pipeline kiểm; `cargo run -- --verify` đòi các giá trị kỳ vọng
//! trong bài (đúng thì in `VERIFY OK`, sai thì thoát mã 1). Cần `TOI_NOI_PG_URL`; thiếu là lỗi, không bỏ qua.

use std::path::Path;
use std::process::ExitCode;

use anyhow::{Result, ensure};
use ledger_ci::pipeline::{self, REQUIRED_CI_JOBS};
use ledger_ci::testdb::TestDb;
use ledger_ci::trip::{TN_0001_ACCOUNTS, tn_0001_expected, trip_entries};
use sqlx::{Acquire, Executor};

/// Những gì kịch bản PostgreSQL quan sát được.
struct Facts {
    capture: String,
    split: String,
    replayed: bool,
    balances: Vec<(String, i64)>,
    unbalanced_code: Option<String>,
    schema_gone: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    let verify = std::env::args().any(|a| a == "--verify");
    match run(verify).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("VERIFY FAIL: {e:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run(verify: bool) -> Result<()> {
    // 1. Pipeline: đọc file trong lab, không gọi mạng.
    let report = pipeline::check_lab(Path::new(env!("CARGO_MANIFEST_DIR")))?;
    println!(
        "ci.yml: {} job ({})",
        report.ci_jobs.len(),
        report.ci_jobs.join(", ")
    );
    println!(
        "MSRV trong Cargo.toml: {} (có trong matrix của job test)",
        report.msrv
    );
    println!(
        "action pin theo SHA: {} lượt `uses:` ở 3 workflow",
        report.action_refs
    );
    println!("vi phạm quy tắc pipeline: {}", report.problems.len());
    for p in &report.problems {
        eprintln!("  - {p}");
    }

    // 2. Sổ cái trên PostgreSQL thật, trong schema riêng; luôn dọn trước khi kiểm.
    let db = TestDb::try_new().await?;
    let url = db.url().to_owned();
    let schema = db.schema.clone();
    let steps = scenario(&db).await;
    db.close().await;
    let mut facts = steps?;
    facts.schema_gone = !TestDb::schema_exists(&url, &schema).await?;

    println!("ghi trip:TN-0001:capture -> {}", facts.capture);
    println!("ghi trip:TN-0001:split -> {}", facts.split);
    println!("ghi lại capture -> replayed = {}", facts.replayed);
    for (account, dong) in &facts.balances {
        println!("  {account:<20} {dong:>8}");
    }
    println!(
        "COMMIT bút toán lệch -> SQLSTATE {}",
        facts.unbalanced_code.as_deref().unwrap_or("không lỗi")
    );
    println!("sau close(): schema còn tồn tại = {}", !facts.schema_gone);

    if !verify {
        return Ok(());
    }
    ensure!(
        report.problems.is_empty(),
        "pipeline vi phạm {} quy tắc",
        report.problems.len()
    );
    for job in REQUIRED_CI_JOBS {
        ensure!(
            report.ci_jobs.iter().any(|j| j == job),
            "ci.yml thiếu job {job}"
        );
    }
    ensure!(
        facts.capture == "JE-000001" && facts.split == "JE-000002",
        "mã bút toán"
    );
    ensure!(facts.replayed, "ghi lại cùng key phải là replay");
    let expected: Vec<(String, i64)> = tn_0001_expected()
        .iter()
        .map(|(a, m)| ((*a).to_owned(), m.dong()))
        .collect();
    ensure!(
        facts.balances == expected,
        "số dư sau capture + split: {:?}",
        facts.balances
    );
    ensure!(
        facts.unbalanced_code.as_deref() == Some("23514"),
        "bút toán lệch phải bị trigger từ chối"
    );
    ensure!(facts.schema_gone, "schema riêng phải bị xóa");
    println!("VERIFY OK");
    Ok(())
}

async fn scenario(db: &TestDb) -> Result<Facts> {
    let ledger = db.ledger();
    for (id, kind) in TN_0001_ACCOUNTS {
        ledger.ensure_account(id, kind).await?;
    }
    let e = trip_entries("TN-0001", "customer:CU-001", "driver:DR-001", 5)?;
    let capture = ledger.post(&e.capture).await?;
    let split = ledger.post(&e.split).await?;
    let again = ledger.post(&e.capture).await?;

    let mut balances = Vec::new();
    for (account, _) in tn_0001_expected() {
        let m = ledger.balance(account).await?;
        balances.push((account.to_owned(), m.map_or(0, |m| m.dong())));
    }

    // Trigger cân bút toán chạy lúc COMMIT: một posting đơn lẻ không bao giờ cân.
    let mut conn = db.pool.acquire().await?;
    let mut tx = conn.begin().await?;
    tx.execute("insert into journal_entries (idempotency_key) values ('demo:lech')")
        .await?;
    tx.execute(
        "insert into postings (entry_id, line_no, account_id, amount_dong) \
         select id, 1, 'customer:CU-001', 100 from journal_entries where idempotency_key = 'demo:lech'",
    )
    .await?;
    let unbalanced_code = match tx.commit().await {
        Ok(()) => None,
        Err(e) => e
            .as_database_error()
            .and_then(|d| d.code())
            .map(std::borrow::Cow::into_owned),
    };

    Ok(Facts {
        capture: capture.display_id(),
        split: split.display_id(),
        replayed: again.replayed,
        balances,
        unbalanced_code,
        schema_gone: false,
    })
}
