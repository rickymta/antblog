//! Dữ liệu lịch sử sinh bằng SQL cho phép đo: nhiều chuyến rải đều từ 01/07 đến hết 31/10/2026.
//! Cứ 10 chuyến thì 1 chuyến mới capture, chưa chia: `psp:vixanh:clearing` luôn còn số dư dương.

use anyhow::Result;
use sqlx::{AssertSqlSafe, PgPool};

/// Số ngày từ 01/07 đến 31/10/2026 (giờ Việt Nam).
pub const HISTORY_DAYS: i64 = 123;

/// Nạp `trips` chuyến: capture 2 posting, split 3 posting (trừ chuyến thứ 10, 20, … chưa chia).
/// 300 khách và 100 tài xế, nhưng `psp:vixanh:clearing` và `platform:revenue` dính vào mọi chuyến.
/// Gọi sau `migrate_base` (chưa có trigger kỳ và snapshot) để nạp nhanh; dữ liệu cân theo cách dựng.
///
/// # Errors
/// Lỗi PostgreSQL.
pub async fn seed_history(pool: &PgPool, trips: u32) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::raw_sql(
        "insert into accounts (id, kind) values ('platform:revenue', 'revenue'), ('psp:vixanh:clearing', 'asset'); \
         insert into accounts (id, kind) select 'customer:CU-' || lpad(i::text, 4, '0'), 'liability' from generate_series(1, 300) i; \
         insert into accounts (id, kind) select 'driver:DR-' || lpad(i::text, 3, '0'), 'liability' from generate_series(1, 100) i;",
    )
    .execute(&mut *tx)
    .await?;
    // `trips` là số nguyên do lab truyền, không có dữ liệu ngoài; bảng tạm để hai INSERT dùng chung.
    sqlx::raw_sql(AssertSqlSafe(format!(
        "create temp table seed_trips on commit drop as \
         select i::bigint as i, 12000 + 4500 * ((i * 7) % 9) as fare, \
                'customer:CU-' || lpad((i % 300 + 1)::text, 4, '0') as customer, \
                'driver:DR-' || lpad(((i * 13) % 100 + 1)::text, 3, '0') as driver, \
                timestamptz '2026-06-30 17:00:00+00' + (i - 1) * make_interval(secs => {HISTORY_DAYS} * 86400.0 / {trips}) as t \
           from generate_series(1, {trips}) i"
    )))
    .execute(&mut *tx)
    .await?;
    sqlx::raw_sql(
        "insert into journal_entries (idempotency_key, posted_at, recorded_at) \
         select 'bench:' || lpad(s.i::text, 6, '0') || ':' || k.kind, s.t + k.lag, s.t + k.lag \
           from seed_trips s cross join (values ('capture', interval '0'), ('split', interval '30 seconds')) k(kind, lag) \
          where k.kind = 'capture' or s.i % 10 <> 0 \
          order by s.t + k.lag; \
         insert into postings (entry_id, line_no, account_id, amount_dong) \
         select e.id, v.n, v.acct, v.amt \
           from seed_trips s \
           join journal_entries e on e.idempotency_key = 'bench:' || lpad(s.i::text, 6, '0') || ':capture' \
           cross join lateral (values (1, s.customer, -s.fare::bigint), (2, 'psp:vixanh:clearing', s.fare::bigint)) v(n, acct, amt) \
         union all \
         select e.id, v.n, v.acct, v.amt \
           from seed_trips s \
           join journal_entries e on e.idempotency_key = 'bench:' || lpad(s.i::text, 6, '0') || ':split' \
           cross join lateral (values (1, 'psp:vixanh:clearing', -s.fare::bigint), \
                                      (2, 'platform:revenue', (s.fare / 5)::bigint), \
                                      (3, s.driver, (s.fare - s.fare / 5)::bigint)) v(n, acct, amt);",
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    sqlx::raw_sql("analyze postings; analyze journal_entries;")
        .execute(pool)
        .await?;
    Ok(())
}
