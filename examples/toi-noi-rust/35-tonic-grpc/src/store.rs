//! Kho lưu trữ mà service gRPC gọi: một trait, hai adapter (bộ nhớ và PostgreSQL).

use std::collections::{BTreeMap, HashSet};
use std::future::Future;
use std::sync::Mutex;

use ledger_core::{AccountId, EntryId, JournalEntry, Ledger, LedgerError, Money};
use sqlx::{PgPool, Postgres, Transaction};
use thiserror::Error;

/// Sơ đồ tài khoản rút gọn của chuyến mẫu: (mã, loại). Cả hai adapter mở đúng danh sách này.
pub const CHART: [(&str, &str); 4] = [
    ("customer:CU-001", "liability"),
    ("driver:DR-001", "liability"),
    ("platform:revenue", "revenue"),
    ("psp:vixanh:clearing", "asset"),
];

type Cause = Box<dyn std::error::Error + Send + Sync>;

/// Mọi lý do kho từ chối hoặc không làm được. Tầng gRPC đổi từng biến thể sang một `Status`.
#[derive(Debug, Error)]
pub enum StoreError {
    /// Lỗi nghiệp vụ của `ledger-core` (cùng key khác nội dung, tràn số...).
    #[error(transparent)]
    Domain(#[from] LedgerError),
    /// Bút toán hoặc truy vấn nhắc tới tài khoản chưa được mở.
    #[error("tài khoản {0} chưa mở")]
    UnknownAccount(String),
    /// Posting có số tiền bằng 0 (sổ cái không lưu dòng rỗng).
    #[error("posting tiền bằng 0 ở tài khoản {0}")]
    ZeroAmount(String),
    /// PostgreSQL hủy giao dịch vì tranh chấp (serialization failure hoặc deadlock).
    #[error("giao dịch bị hủy do tranh chấp (SQLSTATE {0})")]
    Aborted(String),
    /// Không với tới được kho (hết pool, mất kết nối). Gửi lại thường được.
    #[error("kho lưu trữ chưa sẵn sàng")]
    Unavailable(#[source] Cause),
    /// Lỗi hạ tầng còn lại (SQL lỗi, dữ liệu lạ).
    #[error("lỗi lưu trữ")]
    Storage(#[source] Cause),
}

/// Kết quả ghi một bút toán.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostOutcome {
    /// Mã `JE-xxxxxx`.
    pub entry_id: String,
    /// `true` nếu `idempotency_key` đã có và ta trả lại bút toán cũ.
    pub replayed: bool,
}

/// Cổng ra kho. Trả `impl Future + Send` để `tonic` (spawn qua thread khác) dùng được với kiểu generic.
pub trait LedgerStore: Send + Sync + 'static {
    /// Ghi bút toán, idempotent theo `idempotency_key`.
    fn post(
        &self,
        entry: &JournalEntry,
    ) -> impl Future<Output = Result<PostOutcome, StoreError>> + Send;

    /// Số dư đã commit của một tài khoản đã mở.
    fn balance(
        &self,
        account: &AccountId,
    ) -> impl Future<Output = Result<Money, StoreError>> + Send;
}

/// SQLSTATE mà client nên thử lại cả giao dịch: 40001 (serialization_failure), 40P01 (deadlock_detected).
pub fn is_contention(sqlstate: &str) -> bool {
    matches!(sqlstate, "40001" | "40P01")
}

/// Phân loại một lỗi sqlx thành `StoreError`.
pub fn classify_sqlx(e: sqlx::Error) -> StoreError {
    let code = e
        .as_database_error()
        .and_then(|d| d.code())
        .map(std::borrow::Cow::into_owned);
    if let Some(code) = code.filter(|c| is_contention(c)) {
        return StoreError::Aborted(code);
    }
    match e {
        sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed | sqlx::Error::Io(_) => {
            StoreError::Unavailable(Box::new(e))
        }
        other => StoreError::Storage(Box::new(other)),
    }
}

fn check_no_zero(entry: &JournalEntry) -> Result<(), StoreError> {
    match entry.postings().iter().find(|p| p.amount == Money::ZERO) {
        Some(p) => Err(StoreError::ZeroAmount(p.account.to_string())),
        None => Ok(()),
    }
}

/// Adapter bộ nhớ: bao `ledger_core::Ledger` sau một `Mutex` chuẩn (không giữ qua `.await`).
#[derive(Debug)]
pub struct MemoryStore {
    ledger: Mutex<Ledger>,
    known: HashSet<String>,
}

impl MemoryStore {
    /// Sổ rỗng, đã mở bốn tài khoản của `CHART`.
    pub fn new() -> MemoryStore {
        MemoryStore {
            ledger: Mutex::new(Ledger::new()),
            known: CHART.iter().map(|(id, _)| (*id).to_string()).collect(),
        }
    }
}

impl Default for MemoryStore {
    fn default() -> MemoryStore {
        MemoryStore::new()
    }
}

impl LedgerStore for MemoryStore {
    async fn post(&self, entry: &JournalEntry) -> Result<PostOutcome, StoreError> {
        check_no_zero(entry)?;
        if let Some(p) = entry
            .postings()
            .iter()
            .find(|p| !self.known.contains(p.account.as_str()))
        {
            return Err(StoreError::UnknownAccount(p.account.to_string()));
        }
        let mut ledger = self.ledger.lock().expect("mutex không bị poison");
        let posted = ledger.post(entry.clone())?;
        Ok(PostOutcome {
            entry_id: posted.id.to_string(),
            replayed: posted.replayed,
        })
    }

    async fn balance(&self, account: &AccountId) -> Result<Money, StoreError> {
        if !self.known.contains(account.as_str()) {
            return Err(StoreError::UnknownAccount(account.to_string()));
        }
        let ledger = self.ledger.lock().expect("mutex không bị poison");
        Ok(ledger.balance(account)?)
    }
}

/// Adapter PostgreSQL. Giữ một `PgPool` (rẻ để clone, an toàn dùng từ nhiều task).
#[derive(Debug, Clone)]
pub struct PgStore {
    pool: PgPool,
}

impl PgStore {
    /// Bao một pool đã trỏ vào schema có bảng và tài khoản.
    pub fn new(pool: PgPool) -> PgStore {
        PgStore { pool }
    }
}

impl LedgerStore for PgStore {
    async fn post(&self, entry: &JournalEntry) -> Result<PostOutcome, StoreError> {
        check_no_zero(entry)?;
        let mut tx = self.pool.begin().await.map_err(classify_sqlx)?;
        // Lỗi ở giữa: `?` thả `tx` chưa commit, tức rollback.
        let outcome = post_in_tx(&mut tx, entry).await?;
        tx.commit().await.map_err(classify_sqlx)?;
        Ok(outcome)
    }

    async fn balance(&self, account: &AccountId) -> Result<Money, StoreError> {
        // LEFT JOIN: tài khoản đã mở mà chưa có bút toán có số dư 0; tài khoản chưa mở không ra dòng nào.
        let row: Option<(Option<i64>,)> = sqlx::query_as(
            "SELECT b.balance_dong FROM accounts a \
             LEFT JOIN account_balances b ON b.account_id = a.id WHERE a.id = $1",
        )
        .bind(account.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(classify_sqlx)?;
        match row {
            Some((dong,)) => Ok(Money::from_dong(dong.unwrap_or(0))),
            None => Err(StoreError::UnknownAccount(account.to_string())),
        }
    }
}

async fn post_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    entry: &JournalEntry,
) -> Result<PostOutcome, StoreError> {
    // Tài khoản phải tồn tại: kiểm trước để trả lỗi nghiệp vụ rõ, không để khóa ngoại làm hỏng giao dịch.
    let wanted: Vec<String> = entry
        .postings()
        .iter()
        .map(|p| p.account.to_string())
        .collect();
    let known: Vec<String> = sqlx::query_scalar("SELECT id FROM accounts WHERE id = ANY($1)")
        .bind(&wanted)
        .fetch_all(&mut **tx)
        .await
        .map_err(classify_sqlx)?;
    if let Some(missing) = wanted.iter().find(|w| !known.contains(w)) {
        return Err(StoreError::UnknownAccount(missing.clone()));
    }

    // Ghi header; trùng key thì không làm gì và không trả dòng nào.
    let inserted: Option<i64> = sqlx::query_scalar(
        "INSERT INTO journal_entries (idempotency_key) VALUES ($1) \
         ON CONFLICT (idempotency_key) DO NOTHING RETURNING id",
    )
    .bind(entry.idempotency_key())
    .fetch_optional(&mut **tx)
    .await
    .map_err(classify_sqlx)?;
    let Some(entry_id) = inserted else {
        return replay(tx, entry).await;
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
        .execute(&mut **tx)
        .await
        .map_err(classify_sqlx)?;
        let slot = per_account.entry(&p.account).or_insert(Money::ZERO);
        *slot = slot.checked_add(p.amount).ok_or(LedgerError::Overflow)?;
    }

    // Cập nhật số dư theo thứ tự tài khoản (BTreeMap): cùng thứ tự khóa hàng giữa các giao dịch.
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
        .execute(&mut **tx)
        .await
        .map_err(classify_sqlx)?;
    }

    Ok(PostOutcome {
        entry_id: format_entry_id(entry_id)?,
        replayed: false,
    })
}

// Key đã có: đọc bút toán cũ, so nội dung, trả replay hoặc IdempotencyConflict.
async fn replay(
    tx: &mut Transaction<'_, Postgres>,
    entry: &JournalEntry,
) -> Result<PostOutcome, StoreError> {
    let id: i64 = sqlx::query_scalar("SELECT id FROM journal_entries WHERE idempotency_key = $1")
        .bind(entry.idempotency_key())
        .fetch_one(&mut **tx)
        .await
        .map_err(classify_sqlx)?;
    let stored: Vec<(String, i64)> = sqlx::query_as(
        "SELECT account_id, amount_dong FROM postings WHERE entry_id = $1 ORDER BY line_no",
    )
    .bind(id)
    .fetch_all(&mut **tx)
    .await
    .map_err(classify_sqlx)?;
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
    Ok(PostOutcome {
        entry_id: format_entry_id(id)?,
        replayed: true,
    })
}

// Mã bút toán dạng số (cột bigint) sang `JE-000001`, qua `EntryId` để đúng một định dạng chuẩn.
fn format_entry_id(id: i64) -> Result<String, StoreError> {
    EntryId::parse(&format!("JE-{id:06}"))
        .map(|e| e.to_string())
        .ok_or_else(|| StoreError::Storage(format!("id bút toán lạ: {id}").into()))
}
