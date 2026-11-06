//! So sánh với `#[sqlx::test]`: macro tự tạo một DATABASE riêng cho mỗi ca và chạy migration.
//! Chỉ biên dịch khi bật feature: `DATABASE_URL="$TOI_NOI_PG_URL" cargo test --features sqlx-test --test sqlx_test_compare`.
#![cfg(feature = "sqlx-test")]

use sqlx::PgPool;

// `pool` đã trỏ vào database mới, đã chạy migration trong ./migrations.
#[sqlx::test(migrations = "./migrations")]
async fn each_case_gets_its_own_database(pool: PgPool) -> sqlx::Result<()> {
    let database: String = sqlx::query_scalar("select current_database()")
        .fetch_one(&pool)
        .await?;
    assert!(database.starts_with("_sqlx_test_"), "{database}");
    // Bảng nằm ở schema public của database riêng đó: không cần search_path.
    let schema: String = sqlx::query_scalar(
        "select table_schema from information_schema.tables where table_name = 'postings'",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(schema, "public");
    println!("database của ca: {database}, bảng postings ở schema: {schema}");
    Ok(())
}

// Hai ca cùng ghi một key ở hai database khác nhau: không đụng nhau.
#[sqlx::test(migrations = "./migrations")]
async fn same_key_case_a(pool: PgPool) -> sqlx::Result<()> {
    insert_demo_entry(&pool).await
}

#[sqlx::test(migrations = "./migrations")]
async fn same_key_case_b(pool: PgPool) -> sqlx::Result<()> {
    insert_demo_entry(&pool).await
}

async fn insert_demo_entry(pool: &PgPool) -> sqlx::Result<()> {
    sqlx::query("insert into journal_entries (idempotency_key) values ('trip:TN-0001:capture')")
        .execute(pool)
        .await?;
    Ok(())
}
