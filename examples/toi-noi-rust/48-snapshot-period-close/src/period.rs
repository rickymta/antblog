//! Kỳ kế toán: tạo kỳ theo tháng và đóng kỳ.

use anyhow::Result;
use jiff::Timestamp;
use sqlx::{Acquire, PgConnection, PgPool};
use thiserror::Error;

use crate::snapshot::write_snapshot;
use crate::tz::month_bounds;

/// Lỗi đóng kỳ: nghiệp vụ tách khỏi hạ tầng.
#[derive(Debug, Error)]
pub enum PeriodError {
    /// Không có kỳ mang nhãn này.
    #[error("không có kỳ {0}")]
    NotFound(String),
    /// Kỳ đã đóng từ trước.
    #[error("kỳ {0} đã đóng")]
    AlreadyClosed(String),
    /// Còn kỳ trước đang mở: phải đóng theo thứ tự.
    #[error("không đóng kỳ {label} được: kỳ trước {earlier} còn mở")]
    EarlierOpen {
        /// Kỳ đang định đóng.
        label: String,
        /// Kỳ trước còn mở.
        earlier: String,
    },
    /// Lỗi PostgreSQL.
    #[error("lỗi database: {0}")]
    Db(#[from] sqlx::Error),
    /// Lỗi khác (ví dụ thời gian).
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Nhãn kỳ tháng dạng `2026-11`.
#[must_use]
pub fn month_label(year: i16, month: i8) -> String {
    format!("{year:04}-{month:02}")
}

/// Tạo `count` kỳ tháng liên tiếp (giờ Việt Nam), bắt đầu từ `(year, month)`.
///
/// # Errors
/// Lỗi thời gian hoặc PostgreSQL (ví dụ kỳ chồng nhau).
pub async fn seed_months(pool: &PgPool, year: i16, month: i8, count: u32) -> Result<()> {
    let (mut y, mut m) = (year, month);
    for _ in 0..count {
        let (from, to) = month_bounds(y, m)?;
        sqlx::query(
            "insert into accounting_periods (label, starts_at, ends_at) \
             values ($1, $2::timestamptz, $3::timestamptz)",
        )
        .bind(month_label(y, m))
        .bind(from.to_string())
        .bind(to.to_string())
        .execute(pool)
        .await?;
        if m == 12 {
            (y, m) = (y + 1, 1);
        } else {
            m += 1;
        }
    }
    Ok(())
}

/// Đóng kỳ `label` trong một transaction. Bước 1 khoá dòng kỳ bằng `FOR UPDATE`: đợi các lần ghi
/// đang dở (giữ `FOR SHARE`) xong và chặn lần ghi mới. Sau đó mới tính snapshot cuối kỳ, rồi đổi
/// trạng thái. Trả số dòng snapshot đã ghi.
///
/// # Errors
/// [`PeriodError`] nếu không có kỳ, kỳ đã đóng, còn kỳ trước đang mở, hoặc lỗi database.
pub async fn close_period(
    conn: &mut PgConnection,
    label: &str,
    now: Timestamp,
) -> Result<u64, PeriodError> {
    let mut tx = conn.begin().await?;
    let row: Option<(i64, String, i64, i64)> = sqlx::query_as(
        "select id, status, extract(epoch from starts_at)::bigint, extract(epoch from ends_at)::bigint \
           from accounting_periods where label = $1 for update",
    )
    .bind(label)
    .fetch_optional(&mut *tx)
    .await?;
    let (id, status, starts, ends) = row.ok_or_else(|| PeriodError::NotFound(label.to_string()))?;
    if status == "CLOSED" {
        return Err(PeriodError::AlreadyClosed(label.to_string()));
    }
    let earlier: Option<String> = sqlx::query_scalar(
        "select label from accounting_periods \
          where ends_at <= to_timestamp($1) and status = 'OPEN' order by starts_at limit 1",
    )
    .bind(starts)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(earlier) = earlier {
        return Err(PeriodError::EarlierOpen {
            label: label.to_string(),
            earlier,
        });
    }
    let cut = Timestamp::from_second(ends).map_err(anyhow::Error::from)?;
    let rows = write_snapshot(&mut tx, cut).await?;
    sqlx::query("update accounting_periods set status = 'CLOSED', closed_at = $2::timestamptz where id = $1")
        .bind(id)
        .bind(now.to_string())
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(rows)
}

/// BẢN SAI, chỉ để dựng lỗi: đọc kỳ không khoá, tính snapshot, rồi mới đổi trạng thái.
/// Một lần ghi đang dở lúc này sẽ lọt vào kỳ đã đóng mà snapshot không có nó.
///
/// # Errors
/// Như [`close_period`].
pub async fn close_period_naive(
    conn: &mut PgConnection,
    label: &str,
    now: Timestamp,
) -> Result<u64, PeriodError> {
    let mut tx = conn.begin().await?;
    let row: Option<(i64, String, i64)> = sqlx::query_as(
        "select id, status, extract(epoch from ends_at)::bigint \
           from accounting_periods where label = $1",
    )
    .bind(label)
    .fetch_optional(&mut *tx)
    .await?;
    let (id, status, ends) = row.ok_or_else(|| PeriodError::NotFound(label.to_string()))?;
    if status == "CLOSED" {
        return Err(PeriodError::AlreadyClosed(label.to_string()));
    }
    let cut = Timestamp::from_second(ends).map_err(anyhow::Error::from)?;
    let rows = write_snapshot(&mut tx, cut).await?;
    sqlx::query("update accounting_periods set status = 'CLOSED', closed_at = $2::timestamptz where id = $1")
        .bind(id)
        .bind(now.to_string())
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(rows)
}
