//! PostgreSQL là nguồn sự thật: ghi bút toán trong một transaction, số dư là projection.

use anyhow::{Context, Result};
use ledger_core::{JournalEntry, Money};
use sqlx::{PgPool, Row};

// Schema chuẩn của loạt (bài 25 giải thích thiết kế). Chạy trong schema `lab_<uuid>` của ca.
const DDL: &str = "
CREATE TABLE accounts (
    id text PRIMARY KEY,
    kind text NOT NULL CHECK (kind IN ('asset','liability','revenue','expense')),
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE journal_entries (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    idempotency_key text NOT NULL UNIQUE,
    reversal_of bigint REFERENCES journal_entries(id),
    posted_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE postings (
    entry_id bigint NOT NULL REFERENCES journal_entries(id),
    line_no int NOT NULL,
    account_id text NOT NULL REFERENCES accounts(id),
    amount_dong bigint NOT NULL CHECK (amount_dong <> 0),
    PRIMARY KEY (entry_id, line_no)
);
CREATE TABLE account_balances (
    account_id text PRIMARY KEY REFERENCES accounts(id),
    balance_dong bigint NOT NULL,
    version bigint NOT NULL DEFAULT 0,
    updated_at timestamptz NOT NULL DEFAULT now()
);";

/// Tạo bốn bảng trong schema hiện tại của kết nối.
pub async fn create_tables(pool: &PgPool) -> Result<()> {
    sqlx::raw_sql(DDL).execute(pool).await?;
    Ok(())
}

/// Kết quả ghi: bút toán mới hoặc phát lại bút toán cũ cùng `idempotency_key`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostOutcome {
    Posted { entry_id: i64 },
    Replayed { entry_id: i64 },
}

// Loại tài khoản rút gọn theo tiền tố của mã.
fn kind_of(account: &str) -> &'static str {
    match account.split(':').next() {
        Some("platform") => "revenue",
        Some("psp") => "asset",
        _ => "liability",
    }
}

/// Ghi một bút toán: mọi thứ trong một transaction, trùng key thì không ghi lần hai.
/// Ràng buộc UNIQUE của PostgreSQL, không phải Redis, là thứ chặn ghi trùng.
pub async fn post_entry(pool: &PgPool, entry: &JournalEntry) -> Result<PostOutcome> {
    let mut tx = pool.begin().await?;
    let inserted = sqlx::query(
        "INSERT INTO journal_entries (idempotency_key) VALUES ($1)
         ON CONFLICT (idempotency_key) DO NOTHING RETURNING id",
    )
    .bind(entry.idempotency_key())
    .fetch_optional(&mut *tx)
    .await?;
    let Some(row) = inserted else {
        let existing: i64 =
            sqlx::query("SELECT id FROM journal_entries WHERE idempotency_key = $1")
                .bind(entry.idempotency_key())
                .fetch_one(&mut *tx)
                .await?
                .get("id");
        tx.rollback().await?;
        return Ok(PostOutcome::Replayed { entry_id: existing });
    };
    let entry_id: i64 = row.get("id");
    for (i, posting) in entry.postings().iter().enumerate() {
        let account = posting.account.as_str();
        let line_no = i32::try_from(i + 1).context("quá nhiều dòng")?;
        sqlx::query("INSERT INTO accounts (id, kind) VALUES ($1, $2) ON CONFLICT (id) DO NOTHING")
            .bind(account)
            .bind(kind_of(account))
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO postings (entry_id, line_no, account_id, amount_dong) VALUES ($1, $2, $3, $4)")
            .bind(entry_id)
            .bind(line_no)
            .bind(account)
            .bind(posting.amount.dong())
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT INTO account_balances (account_id, balance_dong, version) VALUES ($1, $2, 1)
             ON CONFLICT (account_id) DO UPDATE SET
                 balance_dong = account_balances.balance_dong + EXCLUDED.balance_dong,
                 version = account_balances.version + 1,
                 updated_at = now()",
        )
        .bind(account)
        .bind(posting.amount.dong())
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(PostOutcome::Posted { entry_id })
}

/// Số dư theo sổ cái (0 nếu tài khoản chưa có dòng nào).
pub async fn balance(pool: &PgPool, account: &str) -> Result<Money> {
    let row = sqlx::query("SELECT balance_dong FROM account_balances WHERE account_id = $1")
        .bind(account)
        .fetch_optional(pool)
        .await?;
    Ok(Money::from_dong(
        row.map_or(0, |r| r.get::<i64, _>("balance_dong")),
    ))
}

/// Số bút toán đã ghi.
pub async fn entry_count(pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query("SELECT count(*) AS n FROM journal_entries")
        .fetch_one(pool)
        .await?
        .get("n"))
}

/// Tổng toàn bộ dòng: phải bằng 0 trong sổ cái cân.
pub async fn grand_total(pool: &PgPool) -> Result<i64> {
    let row = sqlx::query("SELECT coalesce(sum(amount_dong), 0)::bigint AS total FROM postings")
        .fetch_one(pool)
        .await?;
    Ok(row.get("total"))
}
