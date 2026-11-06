//! Adapter PostgreSQL tối thiểu (rút gọn từ lab bài 27): ghi bút toán, đọc số dư.
//! Không phải `LedgerRepository` của bài 26: không trait, không Unit of Work.

use ledger_core::{JournalEntry, LedgerError};
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

/// Số dư một tài khoản tại thời điểm đọc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BalanceRow {
    /// Số dư (đồng, có dấu).
    pub balance_dong: i64,
    /// Thời điểm đọc theo đồng hồ của PostgreSQL, dạng RFC 3339 UTC.
    pub as_of: String,
}

/// Lỗi ghi bút toán: nghiệp vụ tách khỏi hạ tầng.
#[derive(Debug, Error)]
pub enum PostError {
    /// Lỗi nghiệp vụ từ `ledger-core` (ví dụ `IdempotencyConflict`).
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    /// Posting trỏ tới tài khoản chưa có trong bảng `accounts` (SQLSTATE 23503).
    #[error("tài khoản {0} chưa tồn tại")]
    UnknownAccount(String),
    /// Lỗi từ PostgreSQL hoặc sqlx.
    #[error("lỗi database: {0}")]
    Db(#[from] sqlx::Error),
}

/// Sổ cái trên PostgreSQL. Pool đã đặt `search_path` nên SQL không nhắc tên schema.
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
        let inserted: Option<i64> = sqlx::query_scalar(
            "insert into journal_entries (idempotency_key) values ($1) \
             on conflict (idempotency_key) do nothing returning id",
        )
        .bind(entry.idempotency_key())
        .fetch_optional(&mut *tx)
        .await?;

        let Some(id) = inserted else {
            // Key đã có: đọc lại bút toán cũ và so nội dung.
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
                .map(|r| (r.get("account_id"), r.get("amount_dong")))
                .collect();
            if stored != lines(entry) {
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
            let result = sqlx::query(
                "insert into postings (entry_id, line_no, account_id, amount_dong) values ($1, $2, $3, $4)",
            )
            .bind(id)
            .bind(line_no)
            .bind(account)
            .bind(amount)
            .execute(&mut *tx)
            .await;
            if let Err(e) = result {
                // 23503 = foreign_key_violation: tài khoản không có trong `accounts`.
                let fk = e.as_database_error().and_then(|d| d.code()).as_deref() == Some("23503");
                return Err(if fk {
                    PostError::UnknownAccount(account.clone())
                } else {
                    e.into()
                });
            }
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

    /// Số dư của một tài khoản; `None` nếu tài khoản chưa tồn tại. Tài khoản có mà chưa có
    /// bút toán nào thì số dư bằng 0.
    pub async fn balance(&self, account: &str) -> Result<Option<BalanceRow>, sqlx::Error> {
        let row = sqlx::query(
            "select coalesce(b.balance_dong, 0) as balance_dong, \
                    to_char(now() at time zone 'utc', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as as_of \
             from accounts a left join account_balances b on b.account_id = a.id \
             where a.id = $1",
        )
        .bind(account)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| BalanceRow {
            balance_dong: r.get("balance_dong"),
            as_of: r.get("as_of"),
        }))
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
