//! Adapter PostgreSQL tối thiểu cho sổ cái: đủ để có thứ mà integration test kiểm.
//! Không phải `LedgerRepository` đầy đủ của bài 26: không trait, không Unit of Work.

use ledger_core::{JournalEntry, LedgerError, Money};
use sqlx::migrate::Migrator;
use sqlx::{PgPool, Row};
use thiserror::Error;

/// Migration nhúng vào binary lúc biên dịch; fixture chạy chúng trong schema riêng của mỗi ca.
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Kết quả ghi một bút toán.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posted {
    /// Mã bút toán trong schema (identity, bắt đầu từ 1).
    pub id: i64,
    /// `true` nếu key đã có sẵn và nội dung trùng: không ghi gì thêm.
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
    /// Lỗi nghiệp vụ từ `ledger-core` (ví dụ `IdempotencyConflict`).
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    /// Lỗi từ PostgreSQL hoặc sqlx.
    #[error("lỗi database: {0}")]
    Db(#[from] sqlx::Error),
}

/// Sổ cái trên PostgreSQL. Dùng pool đã đặt `search_path` nên không nhắc tên schema.
#[derive(Clone)]
pub struct PgLedger {
    pool: PgPool,
}

impl PgLedger {
    /// Bọc một pool.
    pub fn new(pool: PgPool) -> PgLedger {
        PgLedger { pool }
    }

    /// Tạo tài khoản nếu chưa có.
    pub async fn ensure_account(&self, id: &str, kind: &str) -> Result<(), sqlx::Error> {
        sqlx::query("insert into accounts (id, kind) values ($1, $2) on conflict (id) do nothing")
            .bind(id)
            .bind(kind)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Ghi bút toán trong một transaction: header, posting, rồi cập nhật `account_balances`.
    /// Trùng `idempotency_key` thì không ghi: cùng nội dung là replay, khác nội dung là xung đột.
    pub async fn post(&self, entry: &JournalEntry) -> Result<Posted, PostError> {
        let mut tx = self.pool.begin().await?;
        let reversal_of = entry
            .reverses()
            .and_then(|id| id.to_string()[3..].parse::<i64>().ok());
        // ON CONFLICT DO NOTHING: nếu key đã có, RETURNING không trả dòng nào.
        let inserted: Option<i64> = sqlx::query_scalar(
            "insert into journal_entries (idempotency_key, reversal_of) values ($1, $2) \
             on conflict (idempotency_key) do nothing returning id",
        )
        .bind(entry.idempotency_key())
        .bind(reversal_of)
        .fetch_optional(&mut *tx)
        .await?;

        let Some(id) = inserted else {
            // Key đã có: đọc lại bút toán cũ và so nội dung (đọc sau khi bên kia commit).
            let old_id: i64 =
                sqlx::query_scalar("select id from journal_entries where idempotency_key = $1")
                    .bind(entry.idempotency_key())
                    .fetch_one(&mut *tx)
                    .await?;
            let rows = sqlx::query(
                "select account_id, amount_dong from postings where entry_id = $1 order by line_no",
            )
            .bind(old_id)
            .fetch_all(&mut *tx)
            .await?;
            let stored: Vec<(String, i64)> = rows
                .iter()
                .map(|r| {
                    (
                        r.get::<String, _>("account_id"),
                        r.get::<i64, _>("amount_dong"),
                    )
                })
                .collect();
            let wanted = lines(entry);
            if stored != wanted {
                return Err(LedgerError::IdempotencyConflict {
                    key: entry.idempotency_key().to_string(),
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
            sqlx::query(
                "insert into postings (entry_id, line_no, account_id, amount_dong) values ($1, $2, $3, $4)",
            )
            .bind(id)
            .bind(line_no)
            .bind(account)
            .bind(amount)
            .execute(&mut *tx)
            .await?;
        }
        // Cập nhật theo thứ tự account_id để hai bút toán đồng thời không khoá chéo nhau.
        let mut deltas = lines(entry);
        deltas.sort();
        for (account, amount) in &deltas {
            sqlx::query(
                "insert into account_balances (account_id, balance_dong, version) values ($1, $2, 1) \
                 on conflict (account_id) do update set \
                 balance_dong = account_balances.balance_dong + excluded.balance_dong, \
                 version = account_balances.version + 1, updated_at = now()",
            )
            .bind(account)
            .bind(amount)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(Posted {
            id,
            replayed: false,
        })
    }

    /// Số dư và version của một tài khoản; `None` nếu chưa có bút toán nào chạm tới nó.
    pub async fn balance(&self, account: &str) -> Result<Option<(Money, i64)>, sqlx::Error> {
        let row =
            sqlx::query("select balance_dong, version from account_balances where account_id = $1")
                .bind(account)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.map(|r| (Money::from_dong(r.get("balance_dong")), r.get("version"))))
    }
}

// (account_id, số tiền có dấu) theo thứ tự posting của bút toán.
fn lines(entry: &JournalEntry) -> Vec<(String, i64)> {
    entry
        .postings()
        .iter()
        .map(|p| (p.account.as_str().to_string(), p.amount.dong()))
        .collect()
}
