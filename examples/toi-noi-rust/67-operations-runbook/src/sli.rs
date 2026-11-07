//! SLI đo bằng SQL. Thời điểm `now` là tham số, nên cùng dữ liệu luôn ra cùng con số.

use jiff::Timestamp;
use sqlx::{PgPool, Row};

use crate::slo::{Bucket, Counts};

/// Tuổi (giây) của dòng outbox chưa gửi cũ nhất tại `now`; `None` khi outbox sạch.
///
/// # Errors
/// Lỗi SQL.
pub async fn outbox_oldest_age_secs(
    pool: &PgPool,
    now: Timestamp,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT (extract(epoch FROM ($1::timestamptz - min(created_at))))::bigint \
         FROM outbox WHERE published_at IS NULL",
    )
    .bind(now.to_string())
    .fetch_one(pool)
    .await
}

/// Số dòng outbox chưa gửi.
///
/// # Errors
/// Lỗi SQL.
pub async fn outbox_backlog(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM outbox WHERE published_at IS NULL")
        .fetch_one(pool)
        .await
}

/// SLI độ tươi theo bucket phút (theo `created_at`): một sự kiện «tốt» khi đã tới Kafka trong
/// `threshold_secs`. Dòng chưa gửi được tính tới `now`: để quá hạn là hỏng, kể cả khi chưa gửi.
///
/// # Errors
/// Lỗi SQL.
pub async fn freshness_buckets(
    pool: &PgPool,
    threshold_secs: i64,
    now: Timestamp,
) -> Result<Vec<Bucket>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT (floor(extract(epoch FROM created_at) / 60) * 60)::bigint AS ts, \
                count(*) FILTER (WHERE extract(epoch FROM \
                    (COALESCE(published_at, $2::timestamptz) - created_at)) <= $1::bigint) AS good, \
                count(*) AS total \
         FROM outbox GROUP BY 1 ORDER BY 1",
    )
    .bind(threshold_secs)
    .bind(now.to_string())
    .fetch_all(pool)
    .await?;
    Ok(rows
        .iter()
        .map(|row| Bucket {
            ts: row.get("ts"),
            counts: Counts {
                good: u64::try_from(row.get::<i64, _>("good")).unwrap_or(0),
                total: u64::try_from(row.get::<i64, _>("total")).unwrap_or(0),
            },
        })
        .collect())
}

/// Tổng mọi posting trong sổ. Bất biến: luôn bằng 0.
///
/// # Errors
/// Lỗi SQL.
pub async fn ledger_total(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COALESCE(sum(amount_dong), 0)::bigint FROM postings")
        .fetch_one(pool)
        .await
}

/// Số tài khoản có `account_balances` khác tổng posting thật (projection lệch).
///
/// # Errors
/// Lỗi SQL.
pub async fn balance_drift_accounts(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT count(*) FROM ( \
            SELECT a.id FROM accounts a \
            LEFT JOIN account_balances b ON b.account_id = a.id \
            LEFT JOIN postings p ON p.account_id = a.id \
            GROUP BY a.id, b.balance_dong \
            HAVING COALESCE(b.balance_dong, 0) <> COALESCE(sum(p.amount_dong), 0) \
         ) AS drift",
    )
    .fetch_one(pool)
    .await
}
