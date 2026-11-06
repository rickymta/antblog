//! Ghi bút toán vào PostgreSQL: một transaction, idempotency key, retry SQLSTATE 40001.

use std::future::Future;
use std::time::Duration;

use ledger_core::{AccountId, JournalEntry, LedgerError, Money, Posting};
use sqlx::postgres::PgPool;
use sqlx::{PgConnection, Row};
use thiserror::Error;

/// Lỗi của lớp lưu trữ: lỗi nghiệp vụ tách khỏi lỗi hạ tầng.
#[derive(Debug, Error)]
pub enum StoreError {
    /// Cùng `idempotency_key` nhưng nội dung khác bản đã ghi.
    #[error("idempotency_key {key} đã dùng cho bút toán khác nội dung")]
    IdempotencyConflict {
        /// Key bị trùng.
        key: String,
    },
    /// Rút thêm sẽ vượt hạn mức của tài khoản.
    #[error("vượt hạn mức rút: đã rút {used}, xin thêm {requested}, hạn mức {limit}")]
    LimitExceeded {
        /// Tổng đã rút theo các posting âm.
        used: Money,
        /// Số xin rút thêm.
        requested: Money,
        /// Hạn mức.
        limit: Money,
    },
    /// Hết số lần thử lại mà vẫn bị từ chối vì xung đột tuần tự hoá.
    #[error("không tuần tự hoá được sau {attempts} lần thử")]
    RetriesExhausted {
        /// Số lần đã thử.
        attempts: u32,
    },
    /// Dữ liệu đọc từ DB không dựng lại được thành bút toán hợp lệ.
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    /// Lỗi từ PostgreSQL hoặc kết nối.
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

impl StoreError {
    /// SQLSTATE nếu đây là lỗi do PostgreSQL trả về.
    pub fn sqlstate(&self) -> Option<String> {
        match self {
            StoreError::Db(err) => sqlstate(err),
            _ => None,
        }
    }

    /// 40001 (`serialization_failure`): giao dịch bị huỷ, thử lại cả giao dịch từ đầu là hợp lệ.
    pub fn is_serialization_failure(&self) -> bool {
        self.sqlstate().as_deref() == Some("40001")
    }
}

/// Lấy SQLSTATE qua `sqlx::Error::Database`; lỗi không phải của DB thì `None`.
pub fn sqlstate(err: &sqlx::Error) -> Option<String> {
    match err {
        sqlx::Error::Database(db_err) => db_err.code().map(|code| code.into_owned()),
        _ => None,
    }
}

/// Mức cô lập của một transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Isolation {
    /// Mặc định của PostgreSQL: mỗi câu lệnh thấy snapshot mới.
    ReadCommitted,
    /// Snapshot chốt ở câu đầu tiên; chưa chặn được write skew.
    RepeatableRead,
    /// Mức chặt nhất: kết quả như thể các giao dịch chạy lần lượt, đổi lại có thể bị huỷ (40001).
    Serializable,
}

impl Isolation {
    fn begin_sql(self) -> &'static str {
        match self {
            Isolation::ReadCommitted => "BEGIN ISOLATION LEVEL READ COMMITTED",
            Isolation::RepeatableRead => "BEGIN ISOLATION LEVEL REPEATABLE READ",
            Isolation::Serializable => "BEGIN ISOLATION LEVEL SERIALIZABLE",
        }
    }

    /// Mở transaction ở mức này.
    ///
    /// # Errors
    /// Lỗi lấy kết nối hoặc `BEGIN`.
    pub async fn begin(
        self,
        pool: &PgPool,
    ) -> Result<sqlx::Transaction<'static, sqlx::Postgres>, sqlx::Error> {
        pool.begin_with(self.begin_sql()).await
    }
}

/// Kết quả ghi một bút toán.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostOutcome {
    /// Bút toán mới được ghi với id này.
    Created {
        /// Id do DB cấp.
        id: i64,
    },
    /// Key đã có, nội dung trùng: trả bút toán cũ, không ghi gì thêm.
    Replayed {
        /// Id của bút toán đã ghi trước đó.
        id: i64,
    },
}

impl PostOutcome {
    /// Id bút toán, bất kể mới hay cũ.
    pub fn id(self) -> i64 {
        match self {
            PostOutcome::Created { id } | PostOutcome::Replayed { id } => id,
        }
    }
}

/// Mã hiển thị `JE-000001` của id trong DB.
pub fn display_id(id: i64) -> String {
    format!("JE-{id:06}")
}

/// Chèn bút toán và các posting trong transaction của người gọi.
/// `Ok(None)` nghĩa là key đã tồn tại (không chèn gì); `Ok(Some(id))` là bút toán mới.
///
/// # Errors
/// Lỗi SQL; posting sai (tài khoản lạ, số tiền 0) làm lỗi giữa chừng, người gọi để transaction rollback.
pub async fn insert_entry(
    conn: &mut PgConnection,
    entry: &JournalEntry,
) -> Result<Option<i64>, sqlx::Error> {
    let inserted = sqlx::query(
        "INSERT INTO journal_entries (idempotency_key) VALUES ($1) \
         ON CONFLICT (idempotency_key) DO NOTHING RETURNING id",
    )
    .bind(entry.idempotency_key())
    .fetch_optional(&mut *conn)
    .await?;
    let Some(row) = inserted else {
        return Ok(None); // key đã có: không chèn gì
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
    Ok(Some(id))
}

/// Đọc lại bút toán đã ghi theo key (cùng thứ tự dòng). `None` nếu chưa có.
///
/// # Errors
/// Lỗi SQL, hoặc dữ liệu trong DB không thành bút toán hợp lệ.
pub async fn load_by_key(
    conn: &mut PgConnection,
    key: &str,
) -> Result<Option<(i64, JournalEntry)>, StoreError> {
    let Some(head) = sqlx::query("SELECT id FROM journal_entries WHERE idempotency_key = $1")
        .bind(key)
        .fetch_optional(&mut *conn)
        .await?
    else {
        return Ok(None);
    };
    let id: i64 = head.get("id");
    let rows = sqlx::query(
        "SELECT account_id, amount_dong FROM postings WHERE entry_id = $1 ORDER BY line_no",
    )
    .bind(id)
    .fetch_all(&mut *conn)
    .await?;
    let mut postings = Vec::with_capacity(rows.len());
    for row in rows {
        let account: String = row.get("account_id");
        let amount: i64 = row.get("amount_dong");
        postings.push(Posting::new(
            AccountId::new(&account)?,
            Money::from_dong(amount),
        ));
    }
    Ok(Some((id, JournalEntry::try_new(key, postings)?)))
}

/// Ghi bút toán một cách idempotent trong một transaction ở mức mặc định của server
/// (READ COMMITTED nếu không ai đổi `default_transaction_isolation`).
///
/// Key mới: ghi bút toán và mọi posting rồi `commit`. Key đã có: không ghi gì, đọc bản cũ
/// và so nội dung. Trùng nội dung là [`PostOutcome::Replayed`], khác nội dung là
/// [`StoreError::IdempotencyConflict`].
///
/// # Errors
/// Mọi [`StoreError`]. Transaction bị drop khi trả lỗi nên tự rollback.
pub async fn post_entry(pool: &PgPool, entry: &JournalEntry) -> Result<PostOutcome, StoreError> {
    let mut tx = pool.begin().await?;
    match insert_entry(&mut tx, entry).await? {
        Some(id) => {
            tx.commit().await?;
            Ok(PostOutcome::Created { id })
        }
        None => {
            // `ON CONFLICT` đã đợi giao dịch kia commit; câu SELECT mới (READ COMMITTED) thấy bản đó.
            let existing = load_by_key(&mut tx, entry.idempotency_key()).await?;
            tx.rollback().await?;
            match existing {
                Some((id, old)) if &old == entry => Ok(PostOutcome::Replayed { id }),
                _ => Err(StoreError::IdempotencyConflict {
                    key: entry.idempotency_key().to_string(),
                }),
            }
        }
    }
}

/// Chạy `op` tối đa `max_attempts` lần; chỉ thử lại khi gặp 40001. Lỗi khác trả ngay.
/// `op` nhận số thứ tự lần thử (từ 1) và phải mở transaction MỚI mỗi lần.
///
/// # Errors
/// Lỗi của `op`, hoặc [`StoreError::RetriesExhausted`] khi hết lượt.
pub async fn retry_serializable<T, F, Fut>(
    max_attempts: u32,
    mut op: F,
) -> Result<(T, u32), StoreError>
where
    F: FnMut(u32) -> Fut,
    Fut: Future<Output = Result<T, StoreError>>,
{
    let mut attempt = 1;
    loop {
        match op(attempt).await {
            Ok(value) => return Ok((value, attempt)),
            Err(err) if err.is_serialization_failure() => {
                if attempt >= max_attempts {
                    return Err(StoreError::RetriesExhausted { attempts: attempt });
                }
                // Lùi nhẹ tăng dần để hai bên không đụng nhau đúng nhịp.
                tokio::time::sleep(Duration::from_millis(10 * u64::from(attempt))).await;
                attempt += 1;
            }
            Err(other) => return Err(other),
        }
    }
}
