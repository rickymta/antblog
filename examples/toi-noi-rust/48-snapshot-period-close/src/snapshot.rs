//! Số dư theo thời điểm: tính lại từ posting, snapshot hằng ngày, snapshot cộng phần chênh.

use anyhow::Result;
use jiff::Timestamp;
use jiff::civil::Date;
use sqlx::{AssertSqlSafe, Connection, PgConnection, PgPool};

use crate::tz::day_bounds;

/// Số dư của `account` ngay trước `cut` (nửa mở: posting có `posted_at < cut`), tính từ đầu.
///
/// # Errors
/// Lỗi PostgreSQL.
pub async fn balance_full(pool: &PgPool, account: &str, cut: Timestamp) -> Result<i64> {
    Ok(sqlx::query_scalar(
        "select coalesce(sum(p.amount_dong), 0)::bigint \
           from postings p join journal_entries e on e.id = p.entry_id \
          where p.account_id = $1 and e.posted_at < $2::timestamptz",
    )
    .bind(account)
    .bind(cut.to_string())
    .fetch_one(pool)
    .await?)
}

/// Câu «snapshot gần nhất + phần chênh»: lấy snapshot mới nhất có `as_of <= cut`,
/// rồi cộng các posting trong `[as_of, cut)`. Không có snapshot thì phần chênh là cả lịch sử.
pub const SNAPSHOT_DELTA_SQL: &str = "\
with snap as (
    select as_of, balance_dong from balance_snapshots
     where account_id = $1 and as_of <= $2::timestamptz
     order by as_of desc limit 1)
select (coalesce((select balance_dong from snap), 0) + coalesce(sum(p.amount_dong), 0))::bigint
  from postings p join journal_entries e on e.id = p.entry_id
 where p.account_id = $1
   and e.posted_at >= coalesce((select as_of from snap), '-infinity'::timestamptz)
   and e.posted_at <  $2::timestamptz";

/// Số dư theo cách snapshot cộng phần chênh.
///
/// # Errors
/// Lỗi PostgreSQL.
pub async fn balance_snapshot_delta(pool: &PgPool, account: &str, cut: Timestamp) -> Result<i64> {
    Ok(sqlx::query_scalar(SNAPSHOT_DELTA_SQL)
        .bind(account)
        .bind(cut.to_string())
        .fetch_one(pool)
        .await?)
}

/// Ghi snapshot tại `cut` cho mọi tài khoản, tính tăng dần từ snapshot trước đó.
/// Chạy lại với cùng `cut` thì ghi đè (upsert). Trả số dòng đã ghi.
///
/// # Errors
/// Lỗi PostgreSQL.
pub async fn write_snapshot(conn: &mut PgConnection, cut: Timestamp) -> Result<u64> {
    let done = sqlx::query(
        "insert into balance_snapshots (account_id, as_of, balance_dong) \
         select a.id, $1::timestamptz, coalesce(s.balance_dong, 0) + coalesce(d.delta, 0) \
           from accounts a \
           left join lateral ( \
                select b.as_of, b.balance_dong from balance_snapshots b \
                 where b.account_id = a.id and b.as_of < $1::timestamptz \
                 order by b.as_of desc limit 1) s on true \
           left join lateral ( \
                select sum(p.amount_dong) as delta \
                  from postings p join journal_entries e on e.id = p.entry_id \
                 where p.account_id = a.id \
                   and e.posted_at >= coalesce(s.as_of, '-infinity'::timestamptz) \
                   and e.posted_at <  $1::timestamptz) d on true \
         on conflict (account_id, as_of) do update set balance_dong = excluded.balance_dong",
    )
    .bind(cut.to_string())
    .execute(conn)
    .await?;
    Ok(done.rows_affected())
}

/// Job hằng ngày: snapshot cuối ngày `day` theo giờ Việt Nam (`as_of` = 00:00 ngày kế tiếp).
/// Khoá mọi kỳ còn mở trước mốc đó (theo thứ tự id) để các lần ghi đang dở xong trước khi tính.
///
/// # Errors
/// Lỗi PostgreSQL hoặc lỗi múi giờ.
pub async fn snapshot_day(pool: &PgPool, day: Date) -> Result<u64> {
    let (_, cut) = day_bounds(day)?;
    let mut tx = pool.begin().await?;
    sqlx::query(
        "select id from accounting_periods \
          where starts_at < $1::timestamptz and status = 'OPEN' order by id for update",
    )
    .bind(cut.to_string())
    .fetch_all(&mut *tx)
    .await?;
    let rows = write_snapshot(&mut tx, cut).await?;
    tx.commit().await?;
    Ok(rows)
}

/// Dựng lại snapshot cuối ngày cho mọi ngày trong `[from_day, to_day]` theo giờ Việt Nam,
/// hoàn toàn từ posting (một lượt, tổng luỹ kế bằng window function). Ghi đè dòng đã có.
///
/// # Errors
/// Lỗi PostgreSQL.
pub async fn rebuild_snapshots(pool: &PgPool, from_day: Date, to_day: Date) -> Result<u64> {
    let done = sqlx::query(
        "with bounds as ( \
             select (($1::date)::timestamp at time zone 'Asia/Ho_Chi_Minh') as from_at, \
                    (($2::date + 1)::timestamp at time zone 'Asia/Ho_Chi_Minh') as to_at), \
         days as ( \
             select ($1::date + g) as day, \
                    ((($1::date + g) + 1)::timestamp at time zone 'Asia/Ho_Chi_Minh') as as_of \
               from generate_series(0, $2::date - $1::date) g), \
         base as ( \
             select p.account_id, sum(p.amount_dong) as total \
               from postings p join journal_entries e on e.id = p.entry_id, bounds \
              where e.posted_at < bounds.from_at group by p.account_id), \
         daily as ( \
             select p.account_id, (e.posted_at at time zone 'Asia/Ho_Chi_Minh')::date as day, \
                    sum(p.amount_dong) as delta \
               from postings p join journal_entries e on e.id = p.entry_id, bounds \
              where e.posted_at >= bounds.from_at and e.posted_at < bounds.to_at \
              group by 1, 2) \
         insert into balance_snapshots (account_id, as_of, balance_dong) \
         select a.id, d.as_of, \
                (coalesce(b.total, 0) + sum(coalesce(x.delta, 0)) over (partition by a.id order by d.day))::bigint \
           from accounts a cross join days d \
           left join base b on b.account_id = a.id \
           left join daily x on x.account_id = a.id and x.day = d.day \
         on conflict (account_id, as_of) do update set balance_dong = excluded.balance_dong",
    )
    .bind(from_day.to_string())
    .bind(to_day.to_string())
    .execute(pool)
    .await?;
    Ok(done.rows_affected())
}

/// Một snapshot lệch so với tính lại từ posting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mismatch {
    /// Tài khoản.
    pub account: String,
    /// `as_of` của snapshot, dạng UTC.
    pub as_of: String,
    /// Số dư đang lưu trong snapshot.
    pub stored: i64,
    /// Số dư tính lại từ posting.
    pub expected: i64,
}

/// So mọi snapshot có `as_of >= since` (hoặc tất cả nếu `None`) với tính lại toàn bộ từ posting.
///
/// # Errors
/// Lỗi PostgreSQL.
pub async fn verify_snapshots(pool: &PgPool, since: Option<Timestamp>) -> Result<Vec<Mismatch>> {
    let rows: Vec<(String, String, i64, i64)> = sqlx::query_as(
        "select s.account_id, to_char(s.as_of at time zone 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'), \
                s.balance_dong, t.total \
           from balance_snapshots s \
           cross join lateral ( \
                select coalesce(sum(p.amount_dong), 0)::bigint as total \
                  from postings p join journal_entries e on e.id = p.entry_id \
                 where p.account_id = s.account_id and e.posted_at < s.as_of) t \
          where s.as_of >= coalesce($1::timestamptz, '-infinity'::timestamptz) \
            and s.balance_dong <> t.total \
          order by s.as_of, s.account_id",
    )
    .bind(since.map(|t| t.to_string()))
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(account, as_of, stored, expected)| Mismatch {
            account,
            as_of,
            stored,
            expected,
        })
        .collect())
}

/// Chạy `EXPLAIN (ANALYZE, BUFFERS)` cho câu `sql` đã có tham số điền sẵn; trả từng dòng kế hoạch.
/// Câu SQL do lab dựng từ hằng số, không có dữ liệu bên ngoài.
///
/// # Errors
/// Lỗi PostgreSQL.
pub async fn explain(pool: &PgPool, sql: String) -> Result<Vec<String>> {
    let mut conn = pool.acquire().await?;
    let mut lines = Vec::new();
    let rows: Vec<String> =
        sqlx::query_scalar(AssertSqlSafe(format!("explain (analyze, buffers) {sql}")))
            .fetch_all(&mut *conn)
            .await?;
    lines.extend(rows);
    // Trả kết nối về pool sạch.
    conn.ping().await?;
    Ok(lines)
}

/// Như [`explain`] nhưng `TIMING OFF`: không đo thời gian từng node, nên `Execution Time`
/// không bị thổi phồng ở câu xử lý nhiều dòng.
///
/// # Errors
/// Lỗi PostgreSQL.
pub async fn explain_no_timing(pool: &PgPool, sql: String) -> Result<Vec<String>> {
    Ok(sqlx::query_scalar(AssertSqlSafe(format!(
        "explain (analyze, timing off) {sql}"
    )))
    .fetch_all(pool)
    .await?)
}
