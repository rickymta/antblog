//! Bản partition theo tháng của `postings` và các hệ quả của nó.

use anyhow::{Result, bail};
use sqlx::PgPool;

use crate::db::{PARTITIONED_SQL, pg_error};

/// Truy vấn chỉ chạm một tháng: partition pruning phải loại 5 partition còn lại.
pub const ONE_MONTH_SQL: &str = "SELECT count(*), sum(amount_dong) FROM postings_p \
     WHERE posted_at >= '2026-03-01' AND posted_at < '2026-04-01'";

/// Cùng truy vấn nhưng không lọc theo posted_at: phải quét cả 6 partition.
pub const ALL_MONTHS_SQL: &str = "SELECT count(*), sum(amount_dong) FROM postings_p";

/// Một tài khoản trong một tháng: pruning cộng với index `(account_id, posted_at)`.
pub const ACCOUNT_MONTH_SQL: &str = "SELECT count(*), sum(amount_dong) FROM postings_p \
     WHERE account_id = 'acct:0042' AND posted_at >= '2026-03-01' AND posted_at < '2026-04-01'";

/// Chạy một câu DDL/DML trong transaction rồi rollback; trả (SQLSTATE, thông điệp) nếu bị từ chối.
async fn rejected(pool: &PgPool, sql: &'static str) -> Result<Option<(String, String)>> {
    let mut tx = pool.begin().await?;
    let outcome = sqlx::raw_sql(sql).execute(&mut *tx).await;
    tx.rollback().await?;
    match outcome {
        Ok(_) => Ok(None),
        Err(e) => match pg_error(&e) {
            Some(pair) => Ok(Some(pair)),
            None => bail!("lỗi không phải từ PostgreSQL: {e}"),
        },
    }
}

/// Khóa chính `(entry_id, line_no)` không chứa partition key `posted_at` thì bị từ chối.
pub async fn pk_without_partition_key(pool: &PgPool) -> Result<Option<(String, String)>> {
    rejected(
        pool,
        "CREATE TABLE postings_bad (
             entry_id bigint NOT NULL, line_no int NOT NULL, posted_at timestamptz NOT NULL,
             PRIMARY KEY (entry_id, line_no)
         ) PARTITION BY RANGE (posted_at)",
    )
    .await
}

/// Bảng khác trỏ FK vào `postings_p(entry_id, line_no)`: không có unique khớp nên bị từ chối.
pub async fn fk_to_partitioned_two_columns(pool: &PgPool) -> Result<Option<(String, String)>> {
    rejected(
        pool,
        "CREATE TABLE posting_notes (
             entry_id bigint NOT NULL, line_no int NOT NULL, note text,
             FOREIGN KEY (entry_id, line_no) REFERENCES postings_p (entry_id, line_no)
         )",
    )
    .await
}

/// FK phải gồm đủ cả `posted_at` mới trỏ được; trả `None` nếu được chấp nhận.
pub async fn fk_to_partitioned_with_key(pool: &PgPool) -> Result<Option<(String, String)>> {
    rejected(
        pool,
        "CREATE TABLE posting_notes (
             entry_id bigint NOT NULL, line_no int NOT NULL, posted_at timestamptz NOT NULL, note text,
             FOREIGN KEY (entry_id, line_no, posted_at)
                 REFERENCES postings_p (entry_id, line_no, posted_at)
         )",
    )
    .await
}

/// `UNIQUE (idempotency_key)` trên bảng partition theo `posted_at`: bị từ chối.
pub async fn unique_key_without_partition_key(pool: &PgPool) -> Result<Option<(String, String)>> {
    rejected(
        pool,
        "CREATE TABLE journal_entries_bad (
             idempotency_key text NOT NULL UNIQUE, posted_at timestamptz NOT NULL
         ) PARTITION BY RANGE (posted_at)",
    )
    .await
}

/// Thêm `posted_at` vào unique thì tạo được, nhưng cùng một key khác `posted_at` đều lọt:
/// trả `None` nghĩa là không có gì chặn lần ghi thứ hai.
pub async fn same_key_two_months(pool: &PgPool) -> Result<Option<(String, String)>> {
    rejected(
        pool,
        "CREATE TABLE journal_entries_pk (
             idempotency_key text NOT NULL, posted_at timestamptz NOT NULL,
             UNIQUE (idempotency_key, posted_at)
         ) PARTITION BY RANGE (posted_at);
         CREATE TABLE journal_entries_pk_03 PARTITION OF journal_entries_pk
             FOR VALUES FROM ('2026-03-01') TO ('2026-04-01');
         CREATE TABLE journal_entries_pk_04 PARTITION OF journal_entries_pk
             FOR VALUES FROM ('2026-04-01') TO ('2026-05-01');
         INSERT INTO journal_entries_pk VALUES
             ('trip:TN-0001:capture', '2026-03-31 23:59+00'),
             ('trip:TN-0001:capture', '2026-04-01 00:01+00');",
    )
    .await
}

/// Dựng `postings_p` cùng 6 partition tháng và sao chép posting sang (kèm `posted_at` của bút toán).
pub async fn create_and_fill(pool: &PgPool) -> Result<u64> {
    sqlx::raw_sql(PARTITIONED_SQL).execute(pool).await?;
    let copied = sqlx::query(
        "INSERT INTO postings_p (entry_id, line_no, posted_at, account_id, amount_dong)
         SELECT p.entry_id, p.line_no, e.posted_at, p.account_id, p.amount_dong
         FROM postings p JOIN journal_entries e ON e.id = p.entry_id",
    )
    .execute(pool)
    .await?;
    sqlx::query("ANALYZE postings_p").execute(pool).await?;
    Ok(copied.rows_affected())
}

/// Số dòng trong từng partition, theo tên.
pub async fn rows_per_partition(pool: &PgPool) -> Result<Vec<(String, i64)>> {
    Ok(sqlx::query_as(
        "SELECT tableoid::regclass::text AS part, count(*)
         FROM postings_p GROUP BY 1 ORDER BY 1",
    )
    .fetch_all(pool)
    .await?)
}

/// Dòng có `posted_at` ngoài mọi partition (tháng 11/2026) bị từ chối.
pub async fn insert_out_of_range(pool: &PgPool) -> Result<Option<(String, String)>> {
    rejected(
        pool,
        "INSERT INTO postings_p (entry_id, line_no, posted_at, account_id, amount_dong)
         VALUES (1, 9, '2026-11-06 10:00+00', 'acct:0001', 1000)",
    )
    .await
}

/// Kết quả thử "duy nhất" của `(entry_id, line_no)` khi khóa chính có thêm `posted_at`.
#[derive(Debug)]
pub struct UniquenessProbe {
    /// Chèn trùng `(entry_id, line_no, posted_at)` có bị từ chối không (phải có).
    pub same_month_rejected: bool,
    /// Chèn trùng `(entry_id, line_no)` nhưng khác tháng có được nhận không (lỗ hổng).
    pub other_month_accepted: bool,
}

pub async fn probe_uniqueness(pool: &PgPool) -> Result<UniquenessProbe> {
    // Lấy một dòng có sẵn: entry nhỏ nhất, dòng 1.
    let (entry_id, posted_at): (i64, String) = sqlx::query_as(
        "SELECT entry_id, posted_at::text FROM postings_p ORDER BY entry_id, line_no LIMIT 1",
    )
    .fetch_one(pool)
    .await?;
    let same = sqlx::query(
        "INSERT INTO postings_p (entry_id, line_no, posted_at, account_id, amount_dong)
         VALUES ($1, 1, $2::timestamptz, 'acct:0001', 1000)",
    )
    .bind(entry_id)
    .bind(&posted_at)
    .execute(pool)
    .await;
    let same_month_rejected =
        matches!(&same, Err(e) if pg_error(e).is_some_and(|(c, _)| c == "23505"));

    // Cùng (entry_id, line_no) nhưng posted_at tháng khác, chạy trong transaction rồi rollback.
    let mut tx = pool.begin().await?;
    let other = sqlx::query(
        "INSERT INTO postings_p (entry_id, line_no, posted_at, account_id, amount_dong)
         VALUES ($1, 1, '2026-06-30 12:00+00', 'acct:0001', 1000)",
    )
    .bind(entry_id)
    .execute(&mut *tx)
    .await;
    tx.rollback().await?;
    Ok(UniquenessProbe {
        same_month_rejected,
        other_month_accepted: other.is_ok(),
    })
}
