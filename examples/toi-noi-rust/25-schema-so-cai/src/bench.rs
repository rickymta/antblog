//! Sinh dữ liệu lớn bằng `generate_series` và đọc `EXPLAIN (ANALYZE, BUFFERS)`.

use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use sqlx::{AssertSqlSafe, PgPool};

use crate::db::INDEX_SQL;

/// Truy vấn đo: tổng số dư của một tài khoản từ posting.
pub const ACCOUNT_SUM_SQL: &str =
    "SELECT count(*), sum(amount_dong) FROM postings WHERE account_id = 'acct:0042'";

/// Tài khoản bench và bút toán 2 dòng (+x, -x), mỗi bút toán rải posted_at trong 181 ngày đầu 2026.
/// Một transaction: nếu `check_balance` thì constraint trigger kiểm cân của mọi bút toán chạy lúc commit;
/// nếu không, tắt hai trigger đó trong transaction này để đo giá của chúng.
pub async fn seed_bulk(pool: &PgPool, entries: i64, check_balance: bool) -> Result<Duration> {
    let started = Instant::now();
    let mut tx = pool.begin().await?;
    if !check_balance {
        sqlx::query("ALTER TABLE postings DISABLE TRIGGER postings_balanced")
            .execute(&mut *tx)
            .await?;
        sqlx::query("ALTER TABLE journal_entries DISABLE TRIGGER entries_have_postings")
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query(
        "INSERT INTO accounts (id, kind)
         SELECT 'acct:' || lpad(i::text, 4, '0'),
                (ARRAY['asset', 'liability', 'revenue', 'expense'])[1 + i % 4]
         FROM generate_series(1, 1000) AS i",
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO journal_entries (idempotency_key, posted_at)
         SELECT 'bench:' || i, timestamptz '2026-01-01' + (i % 181) * interval '1 day'
         FROM generate_series(1, $1::bigint) AS i",
    )
    .bind(entries)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO postings (entry_id, line_no, account_id, amount_dong)
         SELECT e.id, l.line_no,
                'acct:' || lpad(((e.id * 7 + l.line_no * 131) % 1000 + 1)::text, 4, '0'),
                CASE l.line_no WHEN 1 THEN 1 ELSE -1 END * (1000 * (e.id % 9 + 1))
         FROM journal_entries e CROSS JOIN generate_series(1, 2) AS l(line_no)",
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    let elapsed = started.elapsed();
    sqlx::query("ANALYZE postings").execute(pool).await?;
    sqlx::query("ANALYZE journal_entries").execute(pool).await?;
    Ok(elapsed)
}

/// Tạo index `(account_id, entry_id)`; trả thời gian dựng.
pub async fn create_index(pool: &PgPool) -> Result<Duration> {
    let started = Instant::now();
    sqlx::raw_sql(INDEX_SQL).execute(pool).await?;
    let elapsed = started.elapsed();
    sqlx::query("ANALYZE postings").execute(pool).await?;
    Ok(elapsed)
}

/// Kết quả đọc từ một lần `EXPLAIN (ANALYZE, BUFFERS)`.
#[derive(Debug, Clone)]
pub struct Plan {
    pub lines: Vec<String>,
    pub execution_ms: f64,
    /// Số page đọc từ shared buffers (`shared hit`) ở node trên cùng.
    pub shared_hit: i64,
    /// Số page phải đọc từ đĩa/OS cache (`shared read`).
    pub shared_read: i64,
}

impl Plan {
    pub fn pages(&self) -> i64 {
        self.shared_hit + self.shared_read
    }

    pub fn mentions(&self, needle: &str) -> bool {
        self.lines.iter().any(|l| l.contains(needle))
    }
}

/// Lấy số sau `key=` trong một dòng `Buffers: shared hit=12 read=3`.
fn number_after(line: &str, key: &str) -> i64 {
    line.split(key)
        .nth(1)
        .and_then(|rest| {
            rest.chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse()
                .ok()
        })
        .unwrap_or(0)
}

/// Chạy `EXPLAIN (ANALYZE, BUFFERS)` hai lần trên cùng một kết nối và lấy lần sau (cache đã ấm).
/// Tắt parallel query để so hai kế hoạch trên cùng điều kiện.
pub async fn explain(pool: &PgPool, sql: &'static str) -> Result<Plan> {
    let mut conn = pool.acquire().await?;
    sqlx::query("SET max_parallel_workers_per_gather = 0")
        .execute(&mut *conn)
        .await?;
    let mut lines = Vec::new();
    for _ in 0..2 {
        // `sql` là hằng của lab, không có dữ liệu từ ngoài.
        lines = sqlx::query_scalar::<_, String>(AssertSqlSafe(format!(
            "EXPLAIN (ANALYZE, BUFFERS) {sql}"
        )))
        .fetch_all(&mut *conn)
        .await?;
    }
    let execution_ms = lines
        .iter()
        .find_map(|l| l.trim().strip_prefix("Execution Time: "))
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|n| n.parse().ok())
        .context("không đọc được Execution Time")?;
    let buffers = lines
        .iter()
        .find(|l| l.contains("Buffers:"))
        .context("không có dòng Buffers")?;
    Ok(Plan {
        shared_hit: number_after(buffers, "hit="),
        shared_read: number_after(buffers, "read="),
        execution_ms,
        lines,
    })
}

/// Kích thước bảng postings (kèm index) tính bằng KB.
pub async fn postings_size_kb(pool: &PgPool) -> Result<i64> {
    let bytes: i64 = sqlx::query_scalar("SELECT pg_total_relation_size('postings')::bigint")
        .fetch_one(pool)
        .await?;
    Ok(bytes / 1024)
}
