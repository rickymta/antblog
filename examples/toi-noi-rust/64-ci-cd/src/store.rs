//! Sổ cái PostgreSQL tối thiểu. Mọi câu SQL dùng `query!`/`query_scalar!`: tên cột và kiểu được
//! kiểm lúc biên dịch, dựa vào `.sqlx/` khi `SQLX_OFFLINE=true` (đúng cách CI chạy, không cần DB để build).

use ledger_core::{JournalEntry, LedgerError, Money};
use sqlx::PgPool;
use sqlx::migrate::Migrator;
use thiserror::Error;

/// Migration nhúng vào binary lúc biên dịch.
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Kết quả ghi một bút toán.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posted {
    /// Khóa chính trong schema (identity, từ 1).
    pub id: i64,
    /// `true` nếu key đã có với cùng nội dung: không ghi gì thêm.
    pub replayed: bool,
}

impl Posted {
    /// Mã hiển thị `JE-000001`.
    pub fn display_id(&self) -> String {
        format!("JE-{:06}", self.id)
    }
}

/// Lỗi ghi bút toán: nghiệp vụ tách khỏi hạ tầng.
#[derive(Debug, Error)]
pub enum PostError {
    /// Lỗi nghiệp vụ từ `ledger-core`.
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    /// Lỗi từ PostgreSQL hoặc sqlx.
    #[error("lỗi database: {0}")]
    Db(#[from] sqlx::Error),
}

/// Sổ cái trên một pool đã đặt `search_path`, nên câu SQL không nhắc tên schema.
#[derive(Clone)]
pub struct PgLedger {
    pool: PgPool,
}

// (account_id, số tiền có dấu) theo thứ tự posting.
fn lines(entry: &JournalEntry) -> Vec<(String, i64)> {
    entry
        .postings()
        .iter()
        .map(|p| (p.account.as_str().to_owned(), p.amount.dong()))
        .collect()
}

impl PgLedger {
    /// Bọc một pool.
    pub fn new(pool: PgPool) -> PgLedger {
        PgLedger { pool }
    }

    /// Tạo tài khoản nếu chưa có.
    pub async fn ensure_account(&self, id: &str, kind: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "insert into accounts (id, kind) values ($1, $2) on conflict (id) do nothing",
            id,
            kind,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Ghi bút toán trong một transaction. Trùng `idempotency_key`: cùng nội dung là replay,
    /// khác nội dung là `IdempotencyConflict`.
    pub async fn post(&self, entry: &JournalEntry) -> Result<Posted, PostError> {
        let mut tx = self.pool.begin().await?;
        let inserted = sqlx::query_scalar!(
            "insert into journal_entries (idempotency_key) values ($1) \
             on conflict (idempotency_key) do nothing returning id",
            entry.idempotency_key(),
        )
        .fetch_optional(&mut *tx)
        .await?;

        let Some(id) = inserted else {
            let old_id = sqlx::query_scalar!(
                "select id from journal_entries where idempotency_key = $1",
                entry.idempotency_key(),
            )
            .fetch_one(&mut *tx)
            .await?;
            let stored: Vec<(String, i64)> = sqlx::query!(
                "select account_id, amount_dong from postings where entry_id = $1 order by line_no",
                old_id,
            )
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .map(|r| (r.account_id, r.amount_dong))
            .collect();
            if stored != lines(entry) {
                return Err(LedgerError::IdempotencyConflict {
                    key: entry.idempotency_key().to_owned(),
                }
                .into());
            }
            return Ok(Posted {
                id: old_id,
                replayed: true,
            });
        };

        for (i, (account, amount)) in lines(entry).iter().enumerate() {
            let line_no = i32::try_from(i + 1).expect("bút toán có ít hơn 2^31 posting");
            sqlx::query!(
                "insert into postings (entry_id, line_no, account_id, amount_dong) \
                 values ($1, $2, $3, $4)",
                id,
                line_no,
                account,
                amount,
            )
            .execute(&mut *tx)
            .await?;
        }
        // Cập nhật số dư theo thứ tự account_id để hai bút toán đồng thời không khóa chéo nhau.
        let mut deltas = lines(entry);
        deltas.sort();
        for (account, amount) in &deltas {
            sqlx::query!(
                "insert into account_balances (account_id, balance_dong, version) values ($1, $2, 1) \
                 on conflict (account_id) do update set \
                 balance_dong = account_balances.balance_dong + excluded.balance_dong, \
                 version = account_balances.version + 1, updated_at = now()",
                account,
                amount,
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(Posted {
            id,
            replayed: false,
        })
    }

    /// Số dư của một tài khoản; `None` nếu chưa có bút toán nào chạm tới.
    pub async fn balance(&self, account: &str) -> Result<Option<Money>, sqlx::Error> {
        let row = sqlx::query_scalar!(
            "select balance_dong from account_balances where account_id = $1",
            account,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Money::from_dong))
    }

    /// Schema hiện hành của kết nối (để test biết mình đang ở schema nào).
    pub async fn current_schema(&self) -> Result<String, sqlx::Error> {
        let name = sqlx::query_scalar!("select current_schema()")
            .fetch_one(&self.pool)
            .await?;
        Ok(name.unwrap_or_default())
    }
}
