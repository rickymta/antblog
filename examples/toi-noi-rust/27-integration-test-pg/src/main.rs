//! CLI của lab 27: chạy kịch bản trong một schema `lab_<uuid>` rồi dọn.
//! `--verify` kiểm giá trị kỳ vọng; `--cleanup-stale [giây]` xoá schema mồ côi; `--measure` đo thời gian dựng.

// Dùng chung một fixture với `cargo test`: không có hai bản để lệch nhau.
#[path = "../tests/common/mod.rs"]
mod common;

use std::process::ExitCode;
use std::time::Instant;

use anyhow::{Result, ensure};
use common::{
    ENV_PG_URL, TestDb, cleanup_stale_schemas, count_lab_schemas, pg_url_from, schema_exists,
};
use ledger_core::{AccountId, TripCharge, capture_entry, default_policy, fare_for_km};
use sqlx::Executor;

/// Những gì kịch bản quan sát được, để vừa in vừa kiểm.
struct Facts {
    schema_is_lab: bool,
    migrations: i64,
    first: String,
    first_replayed: bool,
    second_replayed: bool,
    balances: Vec<(String, i64, i64)>,
    update_code: String,
    unbalanced_code: String,
    public_has_ledger_tables: bool,
    schema_gone: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("--cleanup-stale") => cleanup(args.get(1)).await,
        Some("--measure") => measure().await,
        Some("--verify") => scenario(true).await,
        None => scenario(false).await,
        Some(other) => Err(anyhow::anyhow!("tham số lạ: {other}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("LỖI: {e:#}");
            ExitCode::FAILURE
        }
    }
}

async fn scenario(verify: bool) -> Result<()> {
    let db = TestDb::try_new().await?;
    let url = db.url.clone();
    let schema = db.schema.clone();
    let before_close = run_steps(&db).await;
    db.close().await; // luôn dọn, kể cả khi bước giữa chừng lỗi
    let mut facts = before_close?;
    facts.schema_gone = !schema_exists(&url, &schema).await?;

    println!(
        "schema riêng: lab_<uuid> ({} ký tự), search_path trỏ vào đó: {}",
        schema.len(),
        facts.schema_is_lab
    );
    println!("migration đã chạy trong schema: {}", facts.migrations);
    println!(
        "ghi trip:TN-0001:capture -> {} (replayed = {})",
        facts.first, facts.first_replayed
    );
    println!("ghi lại cùng key -> replayed = {}", facts.second_replayed);
    for (account, balance, version) in &facts.balances {
        println!("  {account:<20} {balance:>8} (version {version})");
    }
    println!("UPDATE journal_entries -> SQLSTATE {}", facts.update_code);
    println!("COMMIT bút toán lệch -> SQLSTATE {}", facts.unbalanced_code);
    println!(
        "bảng sổ cái ở schema public: {}",
        facts.public_has_ledger_tables
    );
    println!("sau close(): schema còn tồn tại = {}", !facts.schema_gone);

    if verify {
        ensure!(facts.schema_is_lab, "search_path không trỏ vào schema lab_");
        ensure!(
            facts.migrations == 3,
            "cần 3 migration, có {}",
            facts.migrations
        );
        ensure!(
            facts.first == "JE-000001" && !facts.first_replayed,
            "lần ghi đầu sai"
        );
        ensure!(facts.second_replayed, "ghi lại cùng key phải là replay");
        let expect = [
            ("customer:CU-001", -25_500),
            ("driver:DR-001", 20_400),
            ("platform:revenue", 5_100),
        ];
        for (account, dong) in expect {
            ensure!(
                facts
                    .balances
                    .iter()
                    .any(|(a, b, v)| a == account && *b == dong && *v == 1),
                "số dư {account} khác {dong}"
            );
        }
        ensure!(
            facts.update_code == "23001",
            "UPDATE phải bị trigger chặn bằng 23001"
        );
        ensure!(
            facts.unbalanced_code == "23514",
            "bút toán lệch phải bị chặn lúc COMMIT bằng 23514"
        );
        ensure!(
            !facts.public_has_ledger_tables,
            "không được tạo bảng ở public"
        );
        ensure!(facts.schema_gone, "close() phải xoá schema");
        println!("VERIFY OK");
    }
    Ok(())
}

async fn run_steps(db: &TestDb) -> Result<Facts> {
    let search_path: String = sqlx::query_scalar("select current_schema()")
        .fetch_one(&db.pool)
        .await?;
    let migrations: i64 = sqlx::query_scalar("select count(*) from _sqlx_migrations where success")
        .fetch_one(&db.pool)
        .await?;
    db.seed_accounts().await?;

    // Chuyến TN-0001: cước 25.500, phí nền tảng 20% = 5.100, tài xế 20.400.
    let trip = TripCharge {
        trip_id: "TN-0001".to_string(),
        customer: AccountId::new("customer:CU-001")?,
        driver: AccountId::new("driver:DR-001")?,
        fare: fare_for_km(5),
    };
    let entry = capture_entry(&trip, &default_policy())?;
    let ledger = db.ledger();
    let first = ledger.post(&entry).await?;
    let second = ledger.post(&entry).await?;

    let mut balances = Vec::new();
    for account in ["customer:CU-001", "driver:DR-001", "platform:revenue"] {
        if let Some((money, version)) = ledger.balance(account).await? {
            balances.push((account.to_string(), money.dong(), version));
        }
    }

    let update_code = sqlstate(
        db.pool
            .execute("update journal_entries set idempotency_key = 'x'")
            .await,
    );

    // Bút toán lệch: INSERT thành công, lỗi chỉ xuất hiện khi COMMIT (constraint trigger hoãn).
    let mut tx = db.pool.begin().await?;
    sqlx::query("insert into journal_entries (idempotency_key) values ('demo:unbalanced')")
        .execute(&mut *tx)
        .await?;
    sqlx::query("insert into postings (entry_id, line_no, account_id, amount_dong) select max(id), 1, 'customer:CU-001', -100 from journal_entries")
        .execute(&mut *tx)
        .await?;
    sqlx::query("insert into postings (entry_id, line_no, account_id, amount_dong) select max(id), 2, 'driver:DR-001', 90 from journal_entries")
        .execute(&mut *tx)
        .await?;
    let unbalanced_code = sqlstate(tx.commit().await);

    let public_tables: i64 = sqlx::query_scalar(
        "select count(*) from information_schema.tables \
         where table_schema = 'public' and table_name in ('accounts', 'journal_entries', 'postings', 'account_balances')",
    )
    .fetch_one(&db.pool)
    .await?;

    Ok(Facts {
        schema_is_lab: search_path.starts_with("lab_") && search_path == db.schema,
        migrations,
        first: first.display_id(),
        first_replayed: first.replayed,
        second_replayed: second.replayed,
        balances,
        update_code,
        unbalanced_code,
        public_has_ledger_tables: public_tables > 0,
        schema_gone: false,
    })
}

// Lấy SQLSTATE từ kết quả một câu lệnh; "OK" nếu không lỗi.
fn sqlstate<T>(result: Result<T, sqlx::Error>) -> String {
    match result {
        Ok(_) => "OK".to_string(),
        Err(sqlx::Error::Database(e)) => e.code().map_or("?".to_string(), |c| c.into_owned()),
        Err(other) => format!("không phải lỗi database: {other}"),
    }
}

async fn cleanup(arg: Option<&String>) -> Result<()> {
    let max_age: i64 = arg.map_or(Ok(3600), |s| s.parse())?;
    let url = pg_url_from(std::env::var(ENV_PG_URL).ok())?;
    let dropped = cleanup_stale_schemas(&url, max_age).await?;
    println!("đã xoá {} schema cũ hơn {max_age} giây", dropped.len());
    for name in dropped {
        println!("  {name}");
    }
    println!("schema lab_% còn lại: {}", count_lab_schemas(&url).await?);
    Ok(())
}

// Đo thời gian dựng và dọn một TestDb (schema + pool + 3 migration). Số đo trên máy chạy lệnh, không phải SLA.
async fn measure() -> Result<()> {
    const ROUNDS: u32 = 20;
    let mut setup = std::time::Duration::ZERO;
    let mut teardown = std::time::Duration::ZERO;
    for _ in 0..ROUNDS {
        let t = Instant::now();
        let db = TestDb::try_new().await?;
        setup += t.elapsed();
        let t = Instant::now();
        db.close().await;
        teardown += t.elapsed();
    }
    println!(
        "dựng TestDb: trung bình {} ms; close(): trung bình {} ms ({ROUNDS} lần, tuần tự)",
        setup.as_millis() / u128::from(ROUNDS),
        teardown.as_millis() / u128::from(ROUNDS)
    );
    Ok(())
}
