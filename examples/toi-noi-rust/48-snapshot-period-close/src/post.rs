//! Ghi một bút toán với `posted_at` và `recorded_at` do người gọi truyền vào.

use jiff::Timestamp;
use ledger_core::{AccountId, JournalEntry, JournalEntryBuilder, LedgerError, Money};
use sqlx::{PgConnection, PgPool};

/// Loại tài khoản suy ra từ tiền tố mã, theo sơ đồ tài khoản chuẩn của loạt.
#[must_use]
pub fn kind_of(account: &str) -> &'static str {
    if account.starts_with("customer:") || account.starts_with("driver:") {
        "liability"
    } else if account == "platform:revenue" {
        "revenue"
    } else {
        "asset"
    }
}

/// Dựng bút toán từ danh sách `(tài khoản, số tiền có dấu)`; `ledger-core` kiểm tổng bằng 0.
///
/// # Errors
/// `LedgerError` nếu mã tài khoản rỗng hoặc bút toán không cân.
pub fn entry(key: &str, lines: &[(&str, i64)]) -> Result<JournalEntry, LedgerError> {
    let mut builder = JournalEntryBuilder::new(key);
    for (account, dong) in lines {
        builder = builder.posting(AccountId::new(account)?, Money::from_dong(*dong));
    }
    builder.build()
}

/// Ghi bút toán trên một kết nối (thường là trong transaction của người gọi).
/// `posted_at` quyết định kỳ kế toán; `recorded_at` là lúc hệ thống ghi.
///
/// # Errors
/// Lỗi PostgreSQL, gồm cả lỗi do trigger chặn ghi vào kỳ đã đóng (SQLSTATE `23001`).
pub async fn post_on(
    conn: &mut PgConnection,
    entry: &JournalEntry,
    posted_at: Timestamp,
    recorded_at: Timestamp,
    reversal_of: Option<i64>,
) -> Result<i64, sqlx::Error> {
    for p in entry.postings() {
        sqlx::query("insert into accounts (id, kind) values ($1, $2) on conflict (id) do nothing")
            .bind(p.account.as_str())
            .bind(kind_of(p.account.as_str()))
            .execute(&mut *conn)
            .await?;
    }
    let id: i64 = sqlx::query_scalar(
        "insert into journal_entries (idempotency_key, reversal_of, posted_at, recorded_at) \
         values ($1, $2, $3::timestamptz, $4::timestamptz) returning id",
    )
    .bind(entry.idempotency_key())
    .bind(reversal_of)
    .bind(posted_at.to_string())
    .bind(recorded_at.to_string())
    .fetch_one(&mut *conn)
    .await?;
    for (i, p) in entry.postings().iter().enumerate() {
        let line_no = i32::try_from(i + 1).expect("bút toán có ít hơn 2^31 posting");
        sqlx::query(
            "insert into postings (entry_id, line_no, account_id, amount_dong) values ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(line_no)
        .bind(p.account.as_str())
        .bind(p.amount.dong())
        .execute(&mut *conn)
        .await?;
    }
    Ok(id)
}

/// Ghi một bút toán trong transaction riêng.
///
/// # Errors
/// Như [`post_on`].
pub async fn post(
    pool: &PgPool,
    entry: &JournalEntry,
    posted_at: Timestamp,
    recorded_at: Timestamp,
    reversal_of: Option<i64>,
) -> Result<i64, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let id = post_on(&mut tx, entry, posted_at, recorded_at, reversal_of).await?;
    tx.commit().await?;
    Ok(id)
}
