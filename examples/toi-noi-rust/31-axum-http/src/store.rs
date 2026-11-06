//! Kho PostgreSQL tối thiểu: ghi bút toán idempotent và đọc số dư.
//!
//! Rút gọn từ adapter bài 26 (không có trait, không Unit of Work): đủ cho hai endpoint của bài.

use std::collections::BTreeMap;

use ledger_core::{AccountId, EntryId, JournalEntry, LedgerError, Money};
use sqlx::PgPool;
use thiserror::Error;

/// Lỗi của kho: nghiệp vụ (đưa về 4xx) tách khỏi hạ tầng (đưa về 500).
#[derive(Debug, Error)]
pub enum StoreError {
    /// Lỗi nghiệp vụ của `ledger-core` (key trùng khác nội dung, tràn số...).
    #[error(transparent)]
    Domain(#[from] LedgerError),
    /// Bút toán nhắc tới tài khoản chưa mở.
    #[error("tài khoản {0} chưa mở")]
    UnknownAccount(AccountId),
    /// Posting có số tiền bằng 0.
    #[error("posting tiền bằng 0 ở tài khoản {0}")]
    ZeroAmount(AccountId),
    /// Lỗi sqlx (mất kết nối, SQL lỗi...). Chi tiết chỉ để log, không bao giờ ra response.
    #[error("lỗi lưu trữ")]
    Storage(#[from] sqlx::Error),
}

/// Kết quả ghi một bút toán.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostOutcome {
    /// Mã `JE-xxxxxx` của bút toán.
    pub id: EntryId,
    /// `true` nếu `idempotency_key` đã có và ta trả lại bút toán cũ.
    pub replayed: bool,
}

/// Số dư đã commit cùng thời điểm đọc theo đồng hồ PostgreSQL (UTC).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BalanceRead {
    /// Số dư.
    pub balance: Money,
    /// Thời điểm đọc, dạng `2026-11-06T09:30:00.123456Z`.
    pub as_of: String,
}

/// Kho PostgreSQL. `PgPool` rẻ để clone, an toàn dùng từ nhiều task.
#[derive(Debug, Clone)]
pub struct PgLedger {
    pool: PgPool,
}

// Mã bút toán dạng số (cột bigint) sang `EntryId` (`JE-000001`).
fn entry_id_from_i64(id: i64) -> Option<EntryId> {
    EntryId::parse(&format!("JE-{id:06}"))
}

fn bad_id(id: i64) -> sqlx::Error {
    sqlx::Error::Decode(format!("id bút toán lạ: {id}").into())
}

impl PgLedger {
    /// Bao một pool đã có bảng và `search_path` đúng.
    pub fn new(pool: PgPool) -> PgLedger {
        PgLedger { pool }
    }

    /// Ghi bút toán trong một transaction, idempotent theo `idempotency_key`.
    ///
    /// # Errors
    /// [`StoreError::ZeroAmount`], [`StoreError::UnknownAccount`], `IdempotencyConflict`
    /// khi cùng key khác nội dung, hoặc lỗi sqlx.
    pub async fn post(&self, entry: &JournalEntry) -> Result<PostOutcome, StoreError> {
        if let Some(p) = entry.postings().iter().find(|p| p.amount == Money::ZERO) {
            return Err(StoreError::ZeroAmount(p.account.clone()));
        }
        // Thả `tx` mà chưa commit (mọi `return Err` bên dưới) là ROLLBACK.
        let mut tx = self.pool.begin().await?;

        let wanted: Vec<String> = entry
            .postings()
            .iter()
            .map(|p| p.account.as_str().to_string())
            .collect();
        let known: Vec<String> = sqlx::query_scalar("SELECT id FROM accounts WHERE id = ANY($1)")
            .bind(&wanted)
            .fetch_all(&mut *tx)
            .await?;
        if let Some(p) = entry
            .postings()
            .iter()
            .find(|p| !known.iter().any(|k| k == p.account.as_str()))
        {
            return Err(StoreError::UnknownAccount(p.account.clone()));
        }

        // Trùng key thì không chèn và không trả dòng nào.
        let inserted: Option<i64> = sqlx::query_scalar(
            "INSERT INTO journal_entries (idempotency_key) VALUES ($1) \
             ON CONFLICT (idempotency_key) DO NOTHING RETURNING id",
        )
        .bind(entry.idempotency_key())
        .fetch_optional(&mut *tx)
        .await?;

        let Some(raw_id) = inserted else {
            let outcome = replay(&mut tx, entry).await?;
            tx.commit().await?;
            return Ok(outcome);
        };

        let mut per_account: BTreeMap<&AccountId, Money> = BTreeMap::new();
        for (index, p) in entry.postings().iter().enumerate() {
            let line_no = i32::try_from(index + 1).map_err(|_| LedgerError::Overflow)?;
            sqlx::query(
                "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(raw_id)
            .bind(line_no)
            .bind(p.account.as_str())
            .bind(p.amount.dong())
            .execute(&mut *tx)
            .await?;
            let slot = per_account.entry(&p.account).or_insert(Money::ZERO);
            *slot = slot.checked_add(p.amount).ok_or(LedgerError::Overflow)?;
        }

        // Cập nhật số dư theo thứ tự tài khoản (BTreeMap) để hai bút toán đồng thời khóa hàng cùng thứ tự.
        for (account, delta) in per_account {
            sqlx::query(
                "INSERT INTO account_balances (account_id, balance_dong, version) \
                 VALUES ($1, $2, 1) \
                 ON CONFLICT (account_id) DO UPDATE SET \
                   balance_dong = account_balances.balance_dong + EXCLUDED.balance_dong, \
                   version = account_balances.version + 1, \
                   updated_at = now()",
            )
            .bind(account.as_str())
            .bind(delta.dong())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        let id = entry_id_from_i64(raw_id).ok_or_else(|| bad_id(raw_id))?;
        Ok(PostOutcome {
            id,
            replayed: false,
        })
    }

    /// Số dư đã commit của một tài khoản; `None` nếu tài khoản chưa mở.
    /// Tài khoản đã mở nhưng chưa có bút toán có số dư 0.
    ///
    /// # Errors
    /// Lỗi sqlx.
    pub async fn balance(&self, account: &AccountId) -> Result<Option<BalanceRead>, StoreError> {
        let row: Option<(i64, String)> = sqlx::query_as(
            "SELECT COALESCE(b.balance_dong, 0), \
                    to_char(now() AT TIME ZONE 'utc', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') \
             FROM accounts a LEFT JOIN account_balances b ON b.account_id = a.id \
             WHERE a.id = $1",
        )
        .bind(account.as_str())
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(dong, as_of)| BalanceRead {
            balance: Money::from_dong(dong),
            as_of,
        }))
    }
}

// Key đã có: so nội dung với bút toán cũ, trả replay hoặc IdempotencyConflict.
async fn replay(
    tx: &mut sqlx::PgConnection,
    entry: &JournalEntry,
) -> Result<PostOutcome, StoreError> {
    let id: i64 = sqlx::query_scalar("SELECT id FROM journal_entries WHERE idempotency_key = $1")
        .bind(entry.idempotency_key())
        .fetch_one(&mut *tx)
        .await?;
    let stored: Vec<(String, i64)> = sqlx::query_as(
        "SELECT account_id, amount_dong FROM postings WHERE entry_id = $1 ORDER BY line_no",
    )
    .bind(id)
    .fetch_all(&mut *tx)
    .await?;
    let same = stored.len() == entry.postings().len()
        && stored
            .iter()
            .zip(entry.postings())
            .all(|((acc, dong), p)| acc == p.account.as_str() && *dong == p.amount.dong());
    if !same {
        return Err(LedgerError::IdempotencyConflict {
            key: entry.idempotency_key().to_string(),
        }
        .into());
    }
    let id = entry_id_from_i64(id).ok_or_else(|| bad_id(id))?;
    Ok(PostOutcome { id, replayed: true })
}
