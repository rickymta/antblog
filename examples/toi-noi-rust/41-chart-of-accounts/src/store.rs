//! Ghi bút toán vào PostgreSQL, mở tài khoản theo chart, giữ `account_balances` làm projection.

use anyhow::{Result, bail};
use ledger_core::{AccountId, JournalEntry, LedgerError};
use sqlx::{PgPool, Postgres, Transaction};

use crate::chart::{AccountKind, kind_of};

/// Kết quả ghi một bút toán.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostOutcome {
    /// Bút toán mới.
    Created { entry_id: i64 },
    /// Cùng key, cùng nội dung: trả bút toán cũ, không ghi gì thêm.
    Replayed { entry_id: i64 },
}

/// Mã hiển thị: `JE-000001`.
pub fn display_id(id: i64) -> String {
    format!("JE-{id:06}")
}

/// Cộng `delta` vào số dư một tài khoản: tạo dòng nếu chưa có, cộng dồn nếu đã có.
const UPSERT_BALANCE: &str = "\
INSERT INTO account_balances (account_id, balance_dong, version)
VALUES ($1, $2, 1)
ON CONFLICT (account_id) DO UPDATE
SET balance_dong = account_balances.balance_dong + EXCLUDED.balance_dong,
    version      = account_balances.version + 1,
    updated_at   = now()";

/// Mở tài khoản nếu chưa có. Loại lấy từ chart theo mã; mã ngoài sơ đồ thì từ chối.
async fn open_account(tx: &mut Transaction<'_, Postgres>, id: &str) -> Result<()> {
    let kind = kind_of(&AccountId::new(id)?)
        .ok_or_else(|| anyhow::anyhow!("tài khoản {id} không thuộc sơ đồ tài khoản"))?;
    sqlx::query("INSERT INTO accounts (id, kind) VALUES ($1, $2) ON CONFLICT (id) DO NOTHING")
        .bind(id)
        .bind(kind.as_str())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// Ghi các dòng của một bút toán đã có hàng `journal_entries`, và cộng projection.
async fn insert_lines(
    tx: &mut Transaction<'_, Postgres>,
    entry_id: i64,
    lines: &[(&str, i64)],
) -> Result<()> {
    for (index, (account, amount)) in lines.iter().enumerate() {
        open_account(tx, account).await?;
        let line_no = i32::try_from(index + 1)?;
        sqlx::query(
            "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(entry_id)
        .bind(line_no)
        .bind(account)
        .bind(amount)
        .execute(&mut **tx)
        .await?;
        sqlx::query(UPSERT_BALANCE)
            .bind(account)
            .bind(amount)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

/// Các dòng của bút toán `entry_id`, sắp theo (tài khoản, số tiền) để so sánh.
async fn stored_lines(pool: &PgPool, entry_id: i64) -> Result<Vec<(String, i64)>> {
    Ok(sqlx::query_as(
        "SELECT account_id, amount_dong FROM postings WHERE entry_id = $1 \
         ORDER BY account_id, amount_dong",
    )
    .bind(entry_id)
    .fetch_all(pool)
    .await?)
}

/// Ghi một bút toán hợp lệ của `ledger-core`. `posted_at` là chuỗi ISO 8601 do người gọi truyền
/// (đồng hồ nghiệp vụ, không phải `now()`). Cùng key: cùng nội dung thì `Replayed`, khác nội dung
/// thì `LedgerError::IdempotencyConflict`.
pub async fn post(pool: &PgPool, entry: &JournalEntry, posted_at: &str) -> Result<PostOutcome> {
    let lines: Vec<(&str, i64)> = entry
        .postings()
        .iter()
        .map(|p| (p.account.as_str(), p.amount.dong()))
        .collect();
    let mut tx = pool.begin().await?;
    let inserted: Option<i64> = sqlx::query_scalar(
        "INSERT INTO journal_entries (idempotency_key, posted_at) \
         VALUES ($1, $2::text::timestamptz) \
         ON CONFLICT (idempotency_key) DO NOTHING RETURNING id",
    )
    .bind(entry.idempotency_key())
    .bind(posted_at)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(entry_id) = inserted else {
        tx.rollback().await?;
        return replay(pool, entry.idempotency_key(), &lines).await;
    };
    insert_lines(&mut tx, entry_id, &lines).await?;
    tx.commit().await?;
    Ok(PostOutcome::Created { entry_id })
}

async fn replay(pool: &PgPool, key: &str, lines: &[(&str, i64)]) -> Result<PostOutcome> {
    let existing: i64 =
        sqlx::query_scalar("SELECT id FROM journal_entries WHERE idempotency_key = $1")
            .bind(key)
            .fetch_one(pool)
            .await?;
    let mut wanted: Vec<(String, i64)> =
        lines.iter().map(|(a, n)| ((*a).to_string(), *n)).collect();
    wanted.sort();
    if stored_lines(pool, existing).await? == wanted {
        Ok(PostOutcome::Replayed { entry_id: existing })
    } else {
        bail!(LedgerError::IdempotencyConflict {
            key: key.to_string()
        })
    }
}

/// Ghi thẳng các dòng, KHÔNG qua `JournalEntry`: mô phỏng SQL tay, import hàng loạt hoặc service lỗi.
/// Nếu trigger kiểm cân đang bật, `COMMIT` sẽ từ chối bút toán lệch.
pub async fn post_raw(
    pool: &PgPool,
    key: &str,
    posted_at: &str,
    lines: &[(&str, i64)],
) -> Result<i64> {
    let mut tx = pool.begin().await?;
    let entry_id: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (idempotency_key, posted_at) \
         VALUES ($1, $2::text::timestamptz) RETURNING id",
    )
    .bind(key)
    .bind(posted_at)
    .fetch_one(&mut *tx)
    .await?;
    insert_lines(&mut tx, entry_id, lines).await?;
    tx.commit().await?;
    Ok(entry_id)
}

/// Bật hoặc tắt hai trigger kiểm cân. Tắt chỉ để dựng ca "sổ đã hỏng"; bật lại KHÔNG kiểm dữ liệu cũ.
pub async fn set_balance_triggers(pool: &PgPool, enabled: bool) -> Result<()> {
    let verb = if enabled { "ENABLE" } else { "DISABLE" };
    // `verb` chỉ nhận hai giá trị cố định ở trên, không có dữ liệu ngoài.
    for stmt in [
        format!("ALTER TABLE postings {verb} TRIGGER postings_balanced"),
        format!("ALTER TABLE journal_entries {verb} TRIGGER entries_have_postings"),
    ] {
        sqlx::raw_sql(sqlx::AssertSqlSafe(stmt))
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// Số dư một tài khoản tính từ postings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Balance {
    pub account: String,
    pub kind: AccountKind,
    pub dong: i64,
}

/// Số dư mọi tài khoản đã mở, tính từ `postings` (nguồn sự thật), sắp theo mã.
pub async fn balances(pool: &PgPool) -> Result<Vec<Balance>> {
    let rows: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT a.id, a.kind, COALESCE(SUM(p.amount_dong), 0)::bigint \
         FROM accounts a LEFT JOIN postings p ON p.account_id = a.id \
         GROUP BY a.id, a.kind ORDER BY a.id",
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|(account, kind, dong)| {
            let kind = AccountKind::parse(&kind)
                .ok_or_else(|| anyhow::anyhow!("loại tài khoản lạ trong DB: {kind}"))?;
            Ok(Balance {
                account,
                kind,
                dong,
            })
        })
        .collect()
}
