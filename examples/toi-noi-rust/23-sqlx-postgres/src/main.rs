//! Lab bài 23: pool, migration, ghi và đọc bút toán `TN-0001` bằng sqlx.
//! `cargo run` chạy kịch bản và in kết quả; `cargo run -- --verify` kiểm thêm các giá trị kỳ vọng.

use std::process::ExitCode;
use std::time::{Duration, Instant};

use anyhow::{Result, ensure};
use ledger_core::{AccountId, JournalEntry, JournalEntryBuilder, Money};
use ledger_lab::db::{self, LabSchema};
use ledger_lab::store;
use rust_decimal::Decimal;
use sqlx::PgPool;

/// Những gì kịch bản quan sát được, để `--verify` kiểm lại.
struct Findings {
    id: String,
    amounts: Vec<i64>,
    balances: Vec<(String, i64)>,
    volume: Decimal,
    tables: Vec<String>,
    posted_age_secs: i64,
    duplicate_sqlstate: Option<String>,
    pool_timeout_is_pool_timed_out: bool,
    pool_waited_enough: bool,
    runtime_error_mentions_column: bool,
    i32_overflow_rejected: bool,
}

// Bút toán TN-0001: khách trả 25.500, nền tảng giữ 5.100, tài xế nhận 20.400.
fn tn_0001() -> Result<JournalEntry> {
    Ok(JournalEntryBuilder::new("trip:TN-0001:capture")
        .posting(
            AccountId::new("customer:CU-001")?,
            Money::from_dong(-25_500),
        )
        .posting(AccountId::new("platform:revenue")?, Money::from_dong(5_100))
        .posting(AccountId::new("driver:DR-001")?, Money::from_dong(20_400))
        .build()?)
}

// Tên các bảng nằm trong schema hiện tại (search_path của pool).
async fn tables_in_current_schema(pool: &PgPool) -> Result<Vec<String>> {
    Ok(sqlx::query_scalar(
        "select table_name::text from information_schema.tables \
         where table_schema = current_schema() order by table_name",
    )
    .fetch_all(pool)
    .await?)
}

// Ghi lần hai cùng idempotency_key: PostgreSQL từ chối bằng SQLSTATE 23505 (unique_violation).
async fn duplicate_key_sqlstate(pool: &PgPool, entry: &JournalEntry) -> Option<String> {
    let err = store::insert_entry(pool, entry).await.err()?;
    let db_err = err.downcast_ref::<sqlx::Error>()?.as_database_error()?;
    db_err.code().map(|c| c.to_string())
}

// Pool 1 kết nối, giữ nó, rồi xin kết nối thứ hai: phải hết hạn sau acquire_timeout.
async fn pool_timeout_demo(schema: &LabSchema) -> Result<(bool, bool)> {
    let url = db::database_url()?;
    let small = db::pool_options(1, Duration::from_millis(300))
        .connect(&url)
        .await?;
    let held = small.acquire().await?;
    let started = Instant::now();
    let second = small.acquire().await;
    let waited = started.elapsed();
    drop(held);
    small.close().await;
    // Pool chính vẫn dùng được: lỗi chỉ nằm ở pool nhỏ.
    ensure!(!schema.pool.is_closed());
    Ok((
        matches!(second, Err(sqlx::Error::PoolTimedOut)),
        waited >= Duration::from_millis(300),
    ))
}

// Câu SQL ghép tay (không qua query!) sai tên cột: biên dịch vẫn xanh, chạy mới lỗi.
async fn runtime_wrong_column(pool: &PgPool) -> bool {
    let result = sqlx::query_scalar::<_, i64>("select amount from postings")
        .fetch_all(pool)
        .await;
    result.is_err_and(|e| e.to_string().contains("amount"))
}

// 2.147.483.647 là giá trị lớn nhất của int4; chỉ hơn 84.000 chuyến TN-0001 cộng lại là vượt.
async fn i32_is_too_small(pool: &PgPool) -> Result<bool> {
    let in_i32 = sqlx::query_scalar::<_, i32>("select (2147483647 + 1)")
        .fetch_one(pool)
        .await;
    let in_i64 = sqlx::query_scalar::<_, i64>("select (2147483647::bigint + 1)")
        .fetch_one(pool)
        .await?;
    Ok(in_i32.is_err() && in_i64 == 2_147_483_648)
}

async fn run(schema: &LabSchema) -> Result<Findings> {
    let pool = &schema.pool;
    // Phần đầu tên schema đủ để nhận ra; phần còn lại là uuid ngẫu nhiên.
    println!("schema tạm: {}… (xoá cuối ca)", &schema.name()[..12]);
    println!(
        "pool: max_connections = {}, acquire_timeout = {:?}",
        pool.options().get_max_connections(),
        pool.options().get_acquire_timeout()
    );

    let tables = tables_in_current_schema(pool).await?;
    println!("bảng sau migration: {}", tables.join(", "));

    let entry = tn_0001()?;
    let id = store::insert_entry(pool, &entry).await?;
    println!("đã ghi {id} ({})", entry.idempotency_key());

    let Some(stored) = store::load_entry(pool, id).await? else {
        anyhow::bail!("không đọc lại được {id}");
    };
    for p in stored.entry.postings() {
        println!("  {:<22} {:>12}", p.account, p.amount);
    }
    let now = jiff::Timestamp::now();
    let posted_age_secs = now.as_second() - stored.posted_at.as_second();
    println!(
        "posted_at là jiff::Timestamp UTC (lệch giờ máy < 1 phút: {})",
        posted_age_secs.abs() < 60
    );

    let balances = store::balances(pool).await?;
    for (account, balance) in &balances {
        println!("số dư {account:<22} {balance:>12}");
    }
    let volume = store::gross_volume(pool).await?;
    println!("tổng |posting| (NUMERIC -> Decimal): {volume}");

    let duplicate_sqlstate = duplicate_key_sqlstate(pool, &entry).await;
    println!("ghi trùng idempotency_key -> SQLSTATE {duplicate_sqlstate:?}");

    let (timed_out, waited) = pool_timeout_demo(schema).await?;
    println!(
        "pool 1 kết nối, xin kết nối thứ hai -> PoolTimedOut: {timed_out}, đã chờ ≥ 300 ms: {waited}"
    );

    let runtime_error = runtime_wrong_column(pool).await;
    println!("query() sai tên cột chỉ lỗi lúc chạy: {runtime_error}");

    let i32_rejected = i32_is_too_small(pool).await?;
    println!("int4 tràn ở 2.147.483.647, bigint thì không: {i32_rejected}");

    Ok(Findings {
        id: id.to_string(),
        amounts: stored
            .entry
            .postings()
            .iter()
            .map(|p| p.amount.dong())
            .collect(),
        balances: balances
            .iter()
            .map(|(a, m)| (a.to_string(), m.dong()))
            .collect(),
        volume,
        tables,
        posted_age_secs,
        duplicate_sqlstate,
        pool_timeout_is_pool_timed_out: timed_out,
        pool_waited_enough: waited,
        runtime_error_mentions_column: runtime_error,
        i32_overflow_rejected: i32_rejected,
    })
}

fn verify(f: &Findings) -> Result<()> {
    ensure!(f.id == "JE-000001", "mã bút toán: {}", f.id);
    ensure!(
        f.amounts == [-25_500, 5_100, 20_400],
        "số tiền: {:?}",
        f.amounts
    );
    ensure!(f.amounts.iter().sum::<i64>() == 0, "tổng khác 0");
    ensure!(
        f.balances
            == [
                ("customer:CU-001".to_string(), -25_500),
                ("driver:DR-001".to_string(), 20_400),
                ("platform:revenue".to_string(), 5_100),
            ],
        "số dư: {:?}",
        f.balances
    );
    ensure!(f.volume == Decimal::from(51_000), "volume: {}", f.volume);
    ensure!(
        f.tables
            == [
                "_sqlx_migrations",
                "accounts",
                "journal_entries",
                "postings"
            ],
        "bảng: {:?}",
        f.tables
    );
    ensure!(
        f.posted_age_secs.abs() < 300,
        "posted_at lệch {}s",
        f.posted_age_secs
    );
    ensure!(
        f.duplicate_sqlstate.as_deref() == Some("23505"),
        "sqlstate: {:?}",
        f.duplicate_sqlstate
    );
    ensure!(f.pool_timeout_is_pool_timed_out, "không gặp PoolTimedOut");
    ensure!(f.pool_waited_enough, "pool trả lỗi sớm hơn acquire_timeout");
    ensure!(
        f.runtime_error_mentions_column,
        "query() sai cột không lỗi lúc chạy"
    );
    ensure!(f.i32_overflow_rejected, "int4 không tràn như kỳ vọng");
    println!("VERIFY OK");
    Ok(())
}

async fn real_main(verify_mode: bool) -> Result<()> {
    let schema = LabSchema::create().await?;
    let outcome = run(&schema).await;
    // Luôn dọn schema, kể cả khi kịch bản lỗi.
    let dropped = schema.drop_schema().await;
    let findings = outcome?;
    dropped?;
    if verify_mode {
        verify(&findings)?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    let verify_mode = std::env::args().any(|a| a == "--verify");
    match real_main(verify_mode).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("LỖI: {err:#}");
            ExitCode::FAILURE
        }
    }
}
