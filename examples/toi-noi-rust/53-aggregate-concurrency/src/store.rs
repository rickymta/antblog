//! Lưu và nạp aggregate `Account` trên PostgreSQL, với ba cách bảo vệ bất biến:
//! không khóa (sai, chỉ để chứng minh), optimistic bằng `version`, và `SELECT ... FOR UPDATE`.

use ledger_core::{AccountId, JournalEntry, Money, Posting};
use sqlx::postgres::PgPool;
use sqlx::{PgConnection, Row};
use thiserror::Error;
use tokio::sync::{Barrier, Notify};

use crate::account::{Account, AccountError};
use crate::retry::{Jitter, RetryPolicy, Run, with_retry};

/// Lỗi của tầng lưu trữ. Phân biệt lỗi tạm thời (thử lại được) và lỗi nghiệp vụ (không).
#[derive(Debug, Error)]
pub enum StoreError {
    /// Aggregate từ chối: lỗi nghiệp vụ, không thử lại.
    #[error(transparent)]
    Domain(#[from] AccountError),
    /// `UPDATE ... WHERE version = $n` trúng 0 dòng: có người ghi trước. Thử lại được.
    #[error("xung đột version: tài khoản đã bị người khác sửa")]
    Conflict,
    /// Tài khoản không có hàng số dư hoặc hạn mức.
    #[error("không có tài khoản {0}")]
    NoAccount(String),
    /// `idempotency_key` đã được dùng (bài 24 xử lý chi tiết; bài này chỉ báo).
    #[error("idempotency_key {0} đã dùng")]
    KeyReused(String),
    /// Chuyển tiền từ một ví sang chính nó.
    #[error("nguồn và đích là cùng một ví")]
    SameAccount,
    /// Hết số lần thử mà vẫn gặp lỗi tạm thời.
    #[error("hết {attempts} lần thử")]
    RetriesExhausted {
        /// Số lần đã thử.
        attempts: u32,
    },
    /// Lỗi SQL.
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
}

/// SQLSTATE của lỗi database, nếu lỗi đến từ database.
pub fn sqlstate(err: &sqlx::Error) -> Option<String> {
    match err {
        sqlx::Error::Database(db) => db.code().map(|code| code.into_owned()),
        _ => None,
    }
}

impl StoreError {
    /// SQLSTATE nếu đây là lỗi SQL từ database.
    pub fn sqlstate(&self) -> Option<String> {
        match self {
            StoreError::Sql(err) => sqlstate(err),
            _ => None,
        }
    }

    /// Lỗi tạm thời: xung đột version, `40001` (serialization failure) hoặc `40P01` (deadlock).
    /// Chạy lại cả transaction từ đầu có thể thành công. Lỗi nghiệp vụ thì không.
    pub fn is_retryable(&self) -> bool {
        match self {
            StoreError::Conflict => true,
            other => matches!(other.sqlstate().as_deref(), Some("40001" | "40P01")),
        }
    }
}

/// Cách bảo vệ khi ghi một aggregate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Đọc thường rồi ghi đè giá trị đã tính (read-modify-write). SAI: chỉ để chứng minh.
    Unguarded,
    /// Đọc thường, ghi với `WHERE version = $n`; 0 dòng là [`StoreError::Conflict`].
    Optimistic,
    /// `SELECT ... FOR UPDATE` lúc nạp: người sau xếp hàng chờ.
    ForUpdate,
}

/// Điểm dừng sau khi nạp aggregate, trước khi ghi. Chỉ dùng để dựng cuộc đua trong lab.
pub enum Gate<'a> {
    /// Không dừng.
    Open,
    /// Đợi đủ số bên tới điểm này rồi cùng đi tiếp.
    Barrier(&'a Barrier),
    /// Báo «đã tới», rồi đợi được cho đi tiếp. Dựng lịch xen kẽ chính xác với một bên khác.
    Pause(&'a Pause),
}

/// Cặp tín hiệu cho [`Gate::Pause`].
#[derive(Default)]
pub struct Pause {
    /// Phát khi thao tác đã tới điểm dừng.
    pub reached: Notify,
    /// Người điều khiển phát để cho thao tác đi tiếp.
    pub resume: Notify,
}

impl Gate<'_> {
    async fn reach(&self) {
        match self {
            Gate::Open => {}
            Gate::Barrier(barrier) => {
                barrier.wait().await;
            }
            Gate::Pause(pause) => {
                pause.reached.notify_one();
                pause.resume.notified().await;
            }
        }
    }
}

/// Thứ tự khóa hai ví khi chuyển tiền.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockOrder {
    /// Khóa ví nguồn trước, ví đích sau (chuyển ngược chiều sẽ đan vòng).
    AsGiven,
    /// Khóa ví có id nhỏ hơn trước, bất kể chiều chuyển.
    ById,
}

/// Nạp aggregate. `lock = true` khóa hàng `account_balances` bằng `FOR UPDATE` TRƯỚC, trong một câu
/// riêng, rồi mới đọc hạn mức bằng câu thứ hai. Gộp thành một câu `JOIN ... FOR UPDATE OF b` là sai:
/// sau khi chờ khóa, PostgreSQL chỉ đọc lại hàng đã khóa (`b`), còn hàng `account_limits` vẫn lấy từ
/// snapshot cũ của câu lệnh, nên `withdrawn` cũ đi cùng `balance` mới (thí nghiệm này từng ra 7 lần rút
/// thành công vượt hạn mức).
///
/// # Errors
/// [`StoreError::NoAccount`], dữ liệu hỏng hoặc lỗi SQL.
pub async fn load(conn: &mut PgConnection, id: &str, lock: bool) -> Result<Account, StoreError> {
    let sql = if lock {
        "SELECT balance_dong, version FROM account_balances WHERE account_id = $1 FOR UPDATE"
    } else {
        "SELECT balance_dong, version FROM account_balances WHERE account_id = $1"
    };
    let head = sqlx::query(sql)
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?
        .ok_or_else(|| StoreError::NoAccount(id.to_string()))?;
    let limits = sqlx::query(
        "SELECT withdraw_limit_dong, withdrawn_dong FROM account_limits WHERE account_id = $1",
    )
    .bind(id)
    .fetch_optional(&mut *conn)
    .await?
    .ok_or_else(|| StoreError::NoAccount(id.to_string()))?;
    Ok(Account::restore(
        AccountId::new(id).map_err(AccountError::from)?,
        Money::from_dong(head.get("balance_dong")),
        Money::from_dong(limits.get("withdraw_limit_dong")),
        Money::from_dong(limits.get("withdrawn_dong")),
        head.get("version"),
    )?)
}

/// Ghi trạng thái mới của aggregate (`account` đã qua phương thức domain; `version` vẫn là bản đã nạp).
async fn save(conn: &mut PgConnection, mode: Mode, account: &Account) -> Result<(), StoreError> {
    let id = account.id().as_str();
    match mode {
        Mode::Optimistic => {
            let done = sqlx::query(
                "UPDATE account_balances \
                 SET balance_dong = $2, version = version + 1, updated_at = now() \
                 WHERE account_id = $1 AND version = $3",
            )
            .bind(id)
            .bind(account.balance().dong())
            .bind(account.version())
            .execute(&mut *conn)
            .await?;
            if done.rows_affected() == 0 {
                return Err(StoreError::Conflict);
            }
        }
        Mode::Unguarded | Mode::ForUpdate => {
            sqlx::query(
                "UPDATE account_balances \
                 SET balance_dong = $2, version = version + 1, updated_at = now() \
                 WHERE account_id = $1",
            )
            .bind(id)
            .bind(account.balance().dong())
            .execute(&mut *conn)
            .await?;
        }
    }
    sqlx::query("UPDATE account_limits SET withdrawn_dong = $2 WHERE account_id = $1")
        .bind(id)
        .bind(account.withdrawn().dong())
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Chèn bút toán và posting, rồi cộng dồn `account_balances` cho các tài khoản KHÔNG nằm trong `skip`
/// (aggregate đã tự ghi số dư của mình trong [`save`]). Trả id bút toán.
///
/// # Errors
/// [`StoreError::KeyReused`] hoặc lỗi SQL.
pub async fn insert_entry(
    conn: &mut PgConnection,
    entry: &JournalEntry,
    skip: &[&str],
) -> Result<i64, StoreError> {
    let inserted = sqlx::query(
        "INSERT INTO journal_entries (idempotency_key) VALUES ($1) \
         ON CONFLICT (idempotency_key) DO NOTHING RETURNING id",
    )
    .bind(entry.idempotency_key())
    .fetch_optional(&mut *conn)
    .await?;
    let Some(row) = inserted else {
        return Err(StoreError::KeyReused(entry.idempotency_key().to_string()));
    };
    let id: i64 = row.get("id");
    for (index, posting) in entry.postings().iter().enumerate() {
        let line_no = i32::try_from(index + 1).expect("bút toán không có hàng tỷ dòng");
        sqlx::query(
            "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(line_no)
        .bind(posting.account.as_str())
        .bind(posting.amount.dong())
        .execute(&mut *conn)
        .await?;
    }
    // Cộng dồn bằng phép cộng tương đối ở CUỐI transaction để giữ khóa hàng đối ứng càng ngắn càng tốt.
    for posting in entry.postings() {
        if skip.contains(&posting.account.as_str()) {
            continue;
        }
        sqlx::query(
            "INSERT INTO account_balances (account_id, balance_dong, version) VALUES ($1, $2, 1) \
             ON CONFLICT (account_id) DO UPDATE \
             SET balance_dong = account_balances.balance_dong + EXCLUDED.balance_dong, \
                 version = account_balances.version + 1, updated_at = now()",
        )
        .bind(posting.account.as_str())
        .bind(posting.amount.dong())
        .execute(&mut *conn)
        .await?;
    }
    Ok(id)
}

/// Chạy `body` trong một transaction: commit khi `Ok`, rollback tường minh khi `Err`.
/// Khóa của nạn nhân deadlock đã được server nhả lúc huỷ transaction; `ROLLBACK` ở đây để đưa kết nối
/// ra khỏi trạng thái transaction hỏng (`25P02`) trước khi trả về pool.
async fn in_tx<T>(
    pool: &PgPool,
    body: impl AsyncFnOnce(&mut PgConnection) -> Result<T, StoreError>,
) -> Result<T, StoreError> {
    let mut tx = pool.begin().await?;
    match body(&mut tx).await {
        Ok(value) => {
            tx.commit().await?;
            Ok(value)
        }
        Err(err) => {
            // Lỗi rollback không đè lên lỗi gốc.
            let _ = tx.rollback().await;
            Err(err)
        }
    }
}

/// Một lần thử rút tiền (một transaction). Trả id bút toán.
///
/// # Errors
/// Lỗi domain, [`StoreError::Conflict`] (chỉ ở `Optimistic`), lỗi SQL.
pub async fn withdraw_once(
    pool: &PgPool,
    mode: Mode,
    id: &str,
    amount: Money,
    key: &str,
    gate: Gate<'_>,
) -> Result<i64, StoreError> {
    in_tx(pool, async |conn| {
        withdraw_on(conn, mode, id, amount, key, gate).await
    })
    .await
}

/// Thân của một lần rút, chạy trong transaction của người gọi (người gọi commit hoặc rollback).
///
/// # Errors
/// Như [`withdraw_once`].
pub async fn withdraw_on(
    conn: &mut PgConnection,
    mode: Mode,
    id: &str,
    amount: Money,
    key: &str,
    gate: Gate<'_>,
) -> Result<i64, StoreError> {
    let mut account = load(conn, id, mode == Mode::ForUpdate).await?;
    gate.reach().await;
    let entry = account.withdraw(amount, key)?;
    save(conn, mode, &account).await?;
    insert_entry(conn, &entry, &[id]).await
}

/// Cách khóa-rồi-đọc, để so sánh. Chỉ `Separate` đúng.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockedLoad {
    /// Hai câu: `FOR UPDATE` trên `account_balances`, rồi đọc `account_limits` (đúng, là [`load`]).
    Separate,
    /// Một câu `JOIN ... FOR UPDATE OF b`: hàng `account_limits` có thể là bản cũ.
    JoinLockOne,
    /// Một câu `JOIN ... FOR UPDATE OF b, l`: khóa cả hai bảng.
    JoinLockBoth,
}

/// Nạp aggregate đã khóa theo `style`. Chỉ lab dùng để dựng bẫy `JOIN ... FOR UPDATE`.
///
/// # Errors
/// [`StoreError::NoAccount`], dữ liệu hỏng hoặc lỗi SQL.
pub async fn load_locked(
    conn: &mut PgConnection,
    id: &str,
    style: LockedLoad,
) -> Result<Account, StoreError> {
    let sql = match style {
        LockedLoad::Separate => return load(conn, id, true).await,
        LockedLoad::JoinLockOne => {
            "SELECT b.balance_dong, b.version, l.withdraw_limit_dong, l.withdrawn_dong \
             FROM account_balances b JOIN account_limits l ON l.account_id = b.account_id \
             WHERE b.account_id = $1 FOR UPDATE OF b"
        }
        LockedLoad::JoinLockBoth => {
            "SELECT b.balance_dong, b.version, l.withdraw_limit_dong, l.withdrawn_dong \
             FROM account_balances b JOIN account_limits l ON l.account_id = b.account_id \
             WHERE b.account_id = $1 FOR UPDATE OF b, l"
        }
    };
    let row = sqlx::query(sql)
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?
        .ok_or_else(|| StoreError::NoAccount(id.to_string()))?;
    Ok(Account::restore(
        AccountId::new(id).map_err(AccountError::from)?,
        Money::from_dong(row.get("balance_dong")),
        Money::from_dong(row.get("withdraw_limit_dong")),
        Money::from_dong(row.get("withdrawn_dong")),
        row.get("version"),
    )?)
}

/// Tham số vòng thử lại của một task.
#[derive(Clone, Copy)]
pub struct Attempts<'a> {
    /// Giới hạn và nhịp chờ.
    pub policy: RetryPolicy,
    /// Hạt giống jitter (mỗi task một số khác nhau).
    pub seed: u64,
    /// Chỉ áp dụng cho lần thử đầu: ép N task cùng nạp bản cũ rồi cùng ghi (lab).
    pub barrier: Option<&'a Barrier>,
}

impl<'a> Attempts<'a> {
    fn gate(&self, attempt: u32) -> Gate<'a> {
        match self.barrier {
            Some(b) if attempt == 1 => Gate::Barrier(b),
            _ => Gate::Open,
        }
    }
}

/// Rút tiền, thử lại khi gặp lỗi tạm thời; các lần thử lại chạy tự do (không qua barrier).
pub async fn withdraw_retrying(
    pool: &PgPool,
    mode: Mode,
    id: &str,
    amount: Money,
    key: &str,
    retry: Attempts<'_>,
) -> Run<i64> {
    with_retry(retry.policy, Jitter::new(retry.seed), async |attempt| {
        withdraw_once(pool, mode, id, amount, key, retry.gate(attempt)).await
    })
    .await
}

/// Một lần thử chuyển tiền giữa hai ví, khóa cả hai hàng bằng `FOR UPDATE` theo `order`.
/// `gate` dừng ngay sau khi khóa hàng ĐẦU TIÊN (để lab dựng deadlock có chủ đích).
///
/// # Errors
/// Lỗi domain, `40P01` khi deadlock (qua [`StoreError::Sql`]), lỗi SQL.
pub async fn transfer_once(
    pool: &PgPool,
    from: &str,
    to: &str,
    amount: Money,
    key: &str,
    order: LockOrder,
    gate: Gate<'_>,
) -> Result<i64, StoreError> {
    if from == to {
        return Err(StoreError::SameAccount);
    }
    in_tx(pool, async |conn| {
        let (first, second) = match order {
            LockOrder::AsGiven => (from, to),
            LockOrder::ById if from <= to => (from, to),
            LockOrder::ById => (to, from),
        };
        let a = load(conn, first, true).await?;
        gate.reach().await;
        let b = load(conn, second, true).await?;
        let (mut source, mut target) = if first == from { (a, b) } else { (b, a) };
        let entry = source.transfer_to(&mut target, amount, key)?;
        save(conn, Mode::ForUpdate, &source).await?;
        save(conn, Mode::ForUpdate, &target).await?;
        insert_entry(conn, &entry, &[from, to]).await
    })
    .await
}

/// Chuyển tiền, thử lại khi gặp lỗi tạm thời (kể cả `40P01`).
pub async fn transfer_retrying(
    pool: &PgPool,
    from: &str,
    to: &str,
    amount: Money,
    key: &str,
    order: LockOrder,
    retry: Attempts<'_>,
) -> Run<i64> {
    with_retry(retry.policy, Jitter::new(retry.seed), async |attempt| {
        transfer_once(pool, from, to, amount, key, order, retry.gate(attempt)).await
    })
    .await
}

/// Bút toán nạp ví: ví +`amount`, `psp:vixanh:clearing` −`amount`.
fn topup_entry(id: &str, amount: Money, key: &str) -> Result<JournalEntry, AccountError> {
    let out = amount.checked_neg().ok_or(AccountError::Overflow)?;
    Ok(JournalEntry::try_new(
        key,
        vec![
            Posting::new(AccountId::new(id)?, amount),
            Posting::new(AccountId::new("psp:vixanh:clearing")?, out),
        ],
    )?)
}

/// Nạp tiền vào ví bằng đường GHI THƯỜNG, không qua aggregate: chỉ chèn bút toán và cộng dồn
/// `account_balances` (tăng `version`). Mô phỏng một writer khác của cùng tài khoản.
///
/// # Errors
/// [`StoreError::KeyReused`] hoặc lỗi SQL.
pub async fn topup(pool: &PgPool, id: &str, amount: Money, key: &str) -> Result<i64, StoreError> {
    let entry = topup_entry(id, amount, key)?;
    in_tx(pool, async |conn| insert_entry(conn, &entry, &[]).await).await
}

/// Mở ví: tạo tài khoản, hàng số dư, hạn mức; nạp `funds` bằng bút toán fixture
/// (ví +`funds`, `psp:vixanh:clearing` −`funds`, như bài 42).
///
/// # Errors
/// Lỗi SQL hoặc domain.
pub async fn open_wallet(
    pool: &PgPool,
    id: &str,
    funds: Money,
    withdraw_limit: Money,
) -> Result<(), StoreError> {
    in_tx(pool, async |conn| {
        sqlx::query("INSERT INTO accounts (id, kind) VALUES ($1, 'liability')")
            .bind(id)
            .execute(&mut *conn)
            .await?;
        sqlx::query("INSERT INTO account_balances (account_id, balance_dong) VALUES ($1, 0)")
            .bind(id)
            .execute(&mut *conn)
            .await?;
        sqlx::query("INSERT INTO account_limits (account_id, withdraw_limit_dong) VALUES ($1, $2)")
            .bind(id)
            .bind(withdraw_limit.dong())
            .execute(&mut *conn)
            .await?;
        if funds > Money::ZERO {
            let entry = topup_entry(id, funds, &format!("fixture:topup:{id}"))?;
            insert_entry(conn, &entry, &[]).await?;
        }
        Ok(())
    })
    .await
}

/// Kết quả đối chiếu sổ với projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Audit {
    /// Tổng `balance_dong` của mọi tài khoản. Sổ đóng kín nên phải bằng 0.
    pub total_balance: i64,
    /// Tổng posting của mọi tài khoản. Luôn 0 nếu mọi bút toán cân.
    pub total_postings: i64,
    /// Tài khoản có `balance_dong` khác tổng posting của nó (projection lệch sổ).
    pub projection_mismatch: Vec<String>,
    /// `(tài khoản, tổng thực chi ra theo posting, hạn mức)` của ví đã chi quá hạn mức.
    pub over_limit: Vec<(String, i64, i64)>,
    /// Số bút toán.
    pub entries: i64,
}

/// Đối chiếu sổ với projection. Chỉ đọc.
///
/// # Errors
/// Lỗi SQL.
pub async fn audit(pool: &PgPool) -> Result<Audit, sqlx::Error> {
    let total_balance: i64 =
        sqlx::query("SELECT COALESCE(SUM(balance_dong), 0)::bigint FROM account_balances")
            .fetch_one(pool)
            .await?
            .get(0);
    let total_postings: i64 =
        sqlx::query("SELECT COALESCE(SUM(amount_dong), 0)::bigint FROM postings")
            .fetch_one(pool)
            .await?
            .get(0);
    let entries: i64 = sqlx::query("SELECT count(*) FROM journal_entries")
        .fetch_one(pool)
        .await?
        .get(0);
    let projection_mismatch = sqlx::query(
        "SELECT b.account_id FROM account_balances b \
         LEFT JOIN (SELECT account_id, SUM(amount_dong) AS s FROM postings GROUP BY account_id) p \
           ON p.account_id = b.account_id \
         WHERE b.balance_dong <> COALESCE(p.s, 0) ORDER BY b.account_id",
    )
    .fetch_all(pool)
    .await?
    .iter()
    .map(|row| row.get::<String, _>(0))
    .collect();
    let over_limit = sqlx::query(
        "SELECT l.account_id, \
                COALESCE(-SUM(p.amount_dong) FILTER (WHERE p.amount_dong < 0), 0)::bigint AS spent, \
                l.withdraw_limit_dong \
         FROM account_limits l LEFT JOIN postings p ON p.account_id = l.account_id \
         GROUP BY l.account_id, l.withdraw_limit_dong \
         HAVING COALESCE(-SUM(p.amount_dong) FILTER (WHERE p.amount_dong < 0), 0) > l.withdraw_limit_dong \
         ORDER BY l.account_id",
    )
    .fetch_all(pool)
    .await?
    .iter()
    .map(|row| (row.get("account_id"), row.get("spent"), row.get("withdraw_limit_dong")))
    .collect();
    Ok(Audit {
        total_balance,
        total_postings,
        projection_mismatch,
        over_limit,
        entries,
    })
}

/// Trạng thái đã lưu của một ví (đọc thường, ngoài transaction): `(số dư, đã rút, version)`.
///
/// # Errors
/// [`StoreError::NoAccount`] hoặc lỗi SQL.
pub async fn snapshot(pool: &PgPool, id: &str) -> Result<(Money, Money, i64), StoreError> {
    let mut conn = pool.acquire().await?;
    let account = load(&mut conn, id, false).await?;
    Ok((account.balance(), account.withdrawn(), account.version()))
}

/// Số dư theo sổ: tổng posting của tài khoản (nguồn sự thật mà `balance_dong` phải khớp).
///
/// # Errors
/// Lỗi SQL.
pub async fn ledger_balance(pool: &PgPool, id: &str) -> Result<Money, sqlx::Error> {
    let sum: i64 = sqlx::query(
        "SELECT COALESCE(SUM(amount_dong), 0)::bigint FROM postings WHERE account_id = $1",
    )
    .bind(id)
    .fetch_one(pool)
    .await?
    .get(0);
    Ok(Money::from_dong(sum))
}
