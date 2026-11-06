//! Integration test: mỗi ca tạo schema `lab_<uuid>` riêng, nên chạy song song được.

use anyhow::{Result, ensure};
use ledger_core::{AccountId, JournalEntryBuilder, Money};
use ledger_lab::db::{self, with_schema};
use ledger_lab::mapping::to_jiff;
use ledger_lab::store;
use time::OffsetDateTime;

fn capture(key: &str) -> Result<ledger_core::JournalEntry> {
    Ok(JournalEntryBuilder::new(key)
        .posting(
            AccountId::new("customer:CU-001")?,
            Money::from_dong(-25_500),
        )
        .posting(AccountId::new("platform:revenue")?, Money::from_dong(5_100))
        .posting(AccountId::new("driver:DR-001")?, Money::from_dong(20_400))
        .build()?)
}

#[tokio::test]
async fn migration_creates_exactly_three_ledger_tables() -> Result<()> {
    with_schema(|pool| async move {
        let tables: Vec<String> = sqlx::query_scalar(
            "select table_name::text from information_schema.tables \
             where table_schema = current_schema() and table_name <> '_sqlx_migrations' \
             order by table_name",
        )
        .fetch_all(&pool)
        .await?;
        ensure!(
            tables == ["accounts", "journal_entries", "postings"],
            "{tables:?}"
        );
        Ok(())
    })
    .await
}

#[tokio::test]
async fn tn_0001_round_trips_with_exact_amounts() -> Result<()> {
    with_schema(|pool| async move {
        let entry = capture("trip:TN-0001:capture")?;
        let id = store::insert_entry(&pool, &entry).await?;
        ensure!(id.to_string() == "JE-000001");
        let stored = store::load_entry(&pool, id)
            .await?
            .expect("phải có bút toán");
        ensure!(stored.entry == entry, "đọc lại khác bút toán đã ghi");
        let balances = store::balances(&pool).await?;
        let by_name: Vec<(String, i64)> = balances
            .iter()
            .map(|(a, m)| (a.to_string(), m.dong()))
            .collect();
        ensure!(
            by_name
                == [
                    ("customer:CU-001".to_string(), -25_500),
                    ("driver:DR-001".to_string(), 20_400),
                    ("platform:revenue".to_string(), 5_100),
                ]
        );
        Ok(())
    })
    .await
}

#[tokio::test]
async fn unknown_entry_is_none() -> Result<()> {
    with_schema(|pool| async move {
        let id = ledger_lab::mapping::entry_id_from_pk(999)?;
        ensure!(store::load_entry(&pool, id).await?.is_none());
        Ok(())
    })
    .await
}

#[tokio::test]
async fn duplicate_idempotency_key_is_unique_violation() -> Result<()> {
    with_schema(|pool| async move {
        let entry = capture("trip:TN-0001:capture")?;
        store::insert_entry(&pool, &entry).await?;
        let err = store::insert_entry(&pool, &entry).await.unwrap_err();
        let code = err
            .downcast_ref::<sqlx::Error>()
            .and_then(sqlx::Error::as_database_error)
            .and_then(|e| e.code().map(|c| c.to_string()));
        ensure!(code.as_deref() == Some("23505"), "{code:?}");
        // Transaction đã rollback: vẫn chỉ có đúng một bút toán, ba posting.
        let postings: i64 = sqlx::query_scalar("select count(*) from postings")
            .fetch_one(&pool)
            .await?;
        ensure!(postings == 3, "postings = {postings}");
        Ok(())
    })
    .await
}

#[tokio::test]
async fn zero_amount_posting_is_rejected_by_check_constraint() -> Result<()> {
    with_schema(|pool| async move {
        store::insert_entry(&pool, &capture("k1")?).await?;
        let result = sqlx::query("update postings set amount_dong = 0 where line_no = 1")
            .execute(&pool)
            .await;
        ensure!(result.is_err(), "CHECK amount_dong <> 0 không chặn");
        Ok(())
    })
    .await
}

#[tokio::test]
async fn acquire_times_out_when_pool_is_exhausted() -> Result<()> {
    let url = db::database_url()?;
    let pool = db::pool_options(1, std::time::Duration::from_millis(200))
        .connect(&url)
        .await?;
    let held = pool.acquire().await?;
    let second = pool.acquire().await;
    drop(held);
    ensure!(
        matches!(second, Err(sqlx::Error::PoolTimedOut)),
        "{second:?}"
    );
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn bigint_holds_what_int4_cannot() -> Result<()> {
    with_schema(|pool| async move {
        let wide: i64 = sqlx::query_scalar("select (2147483647::bigint + 1)")
            .fetch_one(&pool)
            .await?;
        ensure!(wide == 2_147_483_648);
        let narrow = sqlx::query_scalar::<_, i32>("select (2147483647 + 1)")
            .fetch_one(&pool)
            .await;
        ensure!(narrow.is_err(), "int4 + 1 phải tràn");
        // Đọc cột bigint vào i32 bị từ chối ngay cả khi giá trị nhỏ.
        let mismatch = sqlx::query_scalar::<_, i32>("select 1::bigint")
            .fetch_one(&pool)
            .await;
        ensure!(mismatch.is_err(), "i32 không được nhận bigint");
        Ok(())
    })
    .await
}

#[tokio::test]
async fn timestamptz_keeps_only_microseconds() -> Result<()> {
    with_schema(|pool| async move {
        // Chiều đọc: micro giây đi nguyên qua `time` rồi `jiff`.
        let read: OffsetDateTime =
            sqlx::query_scalar("select timestamptz '2026-11-06 01:02:03.123456+00'")
                .fetch_one(&pool)
                .await?;
        let ts = to_jiff(read)?;
        ensure!(ts.as_nanosecond() == 1_793_926_923_123_456_000, "{ts}");
        // Chiều ghi: sqlx cắt phần nano giây khi encode.
        let with_nanos = OffsetDateTime::from_unix_timestamp_nanos(1_793_926_923_123_456_789)?;
        let back: OffsetDateTime = sqlx::query_scalar("select $1::timestamptz")
            .bind(with_nanos)
            .fetch_one(&pool)
            .await?;
        ensure!(back.unix_timestamp_nanos() == 1_793_926_923_123_456_000);
        Ok(())
    })
    .await
}
