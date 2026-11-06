//! Adapter sqlx: `PgRepository` mở `Transaction`, `PgUow` bao nó.

use std::collections::BTreeMap;

use ledger_core::{AccountId, JournalEntry, LedgerError, Money};
use ledger_port::{
    LedgerRepository, LedgerUnitOfWork, PostOutcome, RepoError, entry_id_from_i64, validate_entry,
};
use sqlx::{PgPool, Postgres, Transaction};

// Mọi lỗi sqlx là lỗi hạ tầng: gói vào `RepoError::Storage`, cổng không biết sqlx.
fn storage(e: sqlx::Error) -> RepoError {
    RepoError::Storage(Box::new(e))
}

/// Kho PostgreSQL. Giữ một `PgPool` (rẻ để clone, an toàn dùng từ nhiều task).
#[derive(Debug, Clone)]
pub struct PgRepository {
    pool: PgPool,
}

impl PgRepository {
    /// Bao một pool đã cấu hình `search_path` và đã có bảng.
    pub fn new(pool: PgPool) -> PgRepository {
        PgRepository { pool }
    }
}

/// Unit of Work: bao `sqlx::Transaction`. Thả mà không `commit` thì sqlx xếp ROLLBACK vào hàng
/// đợi của kết nối; lệnh đi khi kết nối được dùng tiếp hoặc được trả về pool.
#[derive(Debug)]
pub struct PgUow {
    tx: Transaction<'static, Postgres>,
}

impl LedgerRepository for PgRepository {
    type Uow = PgUow;

    async fn begin(&self) -> Result<PgUow, RepoError> {
        // `Pool::begin` trả `Transaction<'static, _>`: nó giữ một kết nối lấy từ pool, trả lại khi xong.
        let tx = self.pool.begin().await.map_err(storage)?;
        Ok(PgUow { tx })
    }

    async fn balance(&self, account: &AccountId) -> Result<Money, RepoError> {
        let dong: Option<i64> =
            sqlx::query_scalar("SELECT balance_dong FROM account_balances WHERE account_id = $1")
                .bind(account.as_str())
                .fetch_optional(&self.pool)
                .await
                .map_err(storage)?;
        Ok(Money::from_dong(dong.unwrap_or(0)))
    }
}

impl LedgerUnitOfWork for PgUow {
    async fn post(&mut self, entry: &JournalEntry) -> Result<PostOutcome, RepoError> {
        validate_entry(entry)?;

        // Tài khoản phải tồn tại. Kiểm trước để trả lỗi nghiệp vụ rõ, không để FK làm hỏng giao dịch.
        let wanted: Vec<String> = entry
            .postings()
            .iter()
            .map(|p| p.account.as_str().to_string())
            .collect();
        let known: Vec<String> = sqlx::query_scalar("SELECT id FROM accounts WHERE id = ANY($1)")
            .bind(&wanted)
            .fetch_all(&mut *self.tx)
            .await
            .map_err(storage)?;
        if let Some(p) = entry
            .postings()
            .iter()
            .find(|p| !known.iter().any(|k| k == p.account.as_str()))
        {
            return Err(RepoError::UnknownAccount(p.account.clone()));
        }

        // Ghi header; trùng key thì không làm gì và không trả dòng nào.
        let inserted: Option<i64> = sqlx::query_scalar(
            "INSERT INTO journal_entries (idempotency_key) VALUES ($1) \
             ON CONFLICT (idempotency_key) DO NOTHING RETURNING id",
        )
        .bind(entry.idempotency_key())
        .fetch_optional(&mut *self.tx)
        .await
        .map_err(storage)?;

        let Some(entry_id) = inserted else {
            return self.replay(entry).await;
        };

        let mut per_account: BTreeMap<&AccountId, Money> = BTreeMap::new();
        for (index, p) in entry.postings().iter().enumerate() {
            let line_no = i32::try_from(index + 1).map_err(|_| LedgerError::Overflow)?;
            sqlx::query(
                "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(entry_id)
            .bind(line_no)
            .bind(p.account.as_str())
            .bind(p.amount.dong())
            .execute(&mut *self.tx)
            .await
            .map_err(storage)?;
            let slot = per_account.entry(&p.account).or_insert(Money::ZERO);
            *slot = slot.checked_add(p.amount).ok_or(LedgerError::Overflow)?;
        }

        // Cập nhật số dư theo thứ tự tài khoản (BTreeMap): hai lần `post` cùng chạm một tập tài khoản
        // luôn khóa hàng theo cùng thứ tự. Một Uow nhiều `post` thì không còn bảo đảm này.
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
            .execute(&mut *self.tx)
            .await
            .map_err(storage)?;
        }

        let id = entry_id_from_i64(entry_id)
            .ok_or_else(|| RepoError::Storage(format!("id bút toán lạ: {entry_id}").into()))?;
        Ok(PostOutcome {
            id,
            replayed: false,
        })
    }

    async fn balance(&mut self, account: &AccountId) -> Result<Money, RepoError> {
        let dong: Option<i64> =
            sqlx::query_scalar("SELECT balance_dong FROM account_balances WHERE account_id = $1")
                .bind(account.as_str())
                .fetch_optional(&mut *self.tx)
                .await
                .map_err(storage)?;
        Ok(Money::from_dong(dong.unwrap_or(0)))
    }

    async fn commit(self) -> Result<(), RepoError> {
        self.tx.commit().await.map_err(storage)
    }
}

impl PgUow {
    // Key đã có: đọc bút toán cũ, so nội dung, trả replay hoặc IdempotencyConflict.
    async fn replay(&mut self, entry: &JournalEntry) -> Result<PostOutcome, RepoError> {
        let id: i64 =
            sqlx::query_scalar("SELECT id FROM journal_entries WHERE idempotency_key = $1")
                .bind(entry.idempotency_key())
                .fetch_one(&mut *self.tx)
                .await
                .map_err(storage)?;
        let stored: Vec<(String, i64)> = sqlx::query_as(
            "SELECT account_id, amount_dong FROM postings WHERE entry_id = $1 ORDER BY line_no",
        )
        .bind(id)
        .fetch_all(&mut *self.tx)
        .await
        .map_err(storage)?;
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
        let id = entry_id_from_i64(id)
            .ok_or_else(|| RepoError::Storage(format!("id bút toán lạ: {id}").into()))?;
        Ok(PostOutcome { id, replayed: true })
    }
}
