//! Thu nhập theo ngày: gom theo múi giờ ngay trong SQL.

use sqlx::{PgPool, Row};

/// Thu nhập một ngày của một tài xế.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayEarning {
    /// Ngày lịch `YYYY-MM-DD` theo múi giờ đã chọn.
    pub day: String,
    /// Số chuyến đã chia tiền cho tài xế trong ngày.
    pub trips: i64,
    /// Tổng tiền tài xế nhận (đồng).
    pub earned_dong: i64,
}

/// Thu nhập từng ngày của `driver` ("DR-001"), ngày cắt theo `tz` (tên IANA, ví dụ
/// `Asia/Ho_Chi_Minh` hoặc `UTC`). Chỉ tính posting dương của bút toán `trip:*:split`,
/// nên khoản rút và khoản đảo của payout không lẫn vào thu nhập.
///
/// # Errors
/// Lỗi SQL, hoặc múi giờ không có trong cơ sở dữ liệu múi giờ của PostgreSQL.
pub async fn earnings_by_day(
    pool: &PgPool,
    driver: &str,
    tz: &str,
) -> Result<Vec<DayEarning>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT to_char(e.posted_at AT TIME ZONE $2, 'YYYY-MM-DD') AS day, \
                count(*) AS trips, \
                SUM(p.amount_dong)::bigint AS earned \
           FROM postings p \
           JOIN journal_entries e ON e.id = p.entry_id \
          WHERE p.account_id = $1 \
            AND p.amount_dong > 0 \
            AND e.idempotency_key LIKE 'trip:%:split' \
          GROUP BY day \
          ORDER BY day",
    )
    .bind(format!("driver:{driver}"))
    .bind(tz)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .iter()
        .map(|r| DayEarning {
            day: r.get("day"),
            trips: r.get("trips"),
            earned_dong: r.get("earned"),
        })
        .collect())
}

/// Ngày lịch mà PostgreSQL gán cho một thời điểm theo múi giờ `tz`.
///
/// # Errors
/// Lỗi SQL.
pub async fn sql_local_day(pool: &PgPool, at: &str, tz: &str) -> Result<String, sqlx::Error> {
    let row = sqlx::query("SELECT to_char($1::timestamptz AT TIME ZONE $2, 'YYYY-MM-DD') AS day")
        .bind(at)
        .bind(tz)
        .fetch_one(pool)
        .await?;
    Ok(row.get("day"))
}
