//! Ghi bút toán, số dư projection và outbox trong cùng một transaction (copy tối thiểu từ lab 37, 58).
//! Thời điểm ghi luôn là tham số `at` lấy từ `Clock`, không phải `now()` của PostgreSQL.

use jiff::Timestamp;
use ledger_core::{AccountId, FeePolicy, JournalEntryBuilder, LedgerError, Money, default_policy};
use sqlx::{PgConnection, PgPool, Row};

use crate::events::{
    EventError, LedgerEntryPosted, PostingDto, SCHEMA_VERSION, TripCompleted, parse_trip_completed,
};

/// Tiền VíXanh đang giữ.
pub const CLEARING: &str = "psp:vixanh:clearing";
/// Doanh thu nền tảng.
pub const REVENUE: &str = "platform:revenue";

/// Một bút toán cần ghi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEntry {
    /// `idempotency_key`.
    pub key: String,
    /// Bút toán bị đảo.
    pub reversal_of: Option<i64>,
    /// Các dòng `(tài khoản, số tiền có dấu)`.
    pub postings: Vec<(String, i64)>,
    /// Mã giao dịch VíXanh gắn với bút toán thu tiền.
    pub external_ref: Option<String>,
}

/// Kết quả xử lý một sự kiện.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Bút toán mới đã ghi.
    Applied,
    /// Inbox đã có sự kiện: không làm gì.
    Duplicate,
    /// Sự kiện mới nhưng khoá bút toán đã có: không ghi bút toán thứ hai.
    EntryAlreadyExists,
}

/// Lỗi xử lý một sự kiện.
#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    /// Payload sai hoặc phiên bản lạ.
    #[error(transparent)]
    Event(#[from] EventError),
    /// Nội dung sự kiện không dựng được bút toán hợp lệ.
    #[error("sự kiện sai nội dung: {0}")]
    Invalid(String),
    /// Lỗi của lõi sổ cái.
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    /// Lỗi PostgreSQL.
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

impl ProcessError {
    /// Lỗi do dữ liệu: thử lại cũng không hết, nên đi DLQ thay vì chặn partition.
    pub fn is_permanent(&self) -> bool {
        !matches!(self, ProcessError::Db(_))
    }
}

/// Ngữ cảnh một lần xử lý.
#[derive(Debug, Clone, Copy)]
pub struct ApplyCtx<'a> {
    /// `schemaVersion` mà bản consumer đang chạy hiểu.
    pub accepted: &'a [u32],
    /// Thời điểm ghi, từ `Clock`.
    pub at: Timestamp,
    /// Topic mà dòng outbox sẽ gửi tới.
    pub out_topic: &'a str,
}

/// Mã VíXanh của chuyến theo quy ước của lab: `TN-0001` → `vx_ch_0001`.
pub fn psp_ref_for_trip(trip_id: &str) -> String {
    format!("vx_ch_{}", trip_id.trim_start_matches("TN-"))
}

fn kind_of(account: &str) -> &'static str {
    if account.starts_with("customer:") || account.starts_with("driver:") {
        "liability"
    } else if account == REVENUE {
        "revenue"
    } else {
        "asset"
    }
}

/// Hai bút toán của chuyến (bài 13): capture `khách → clearing`, split `clearing → nền tảng, tài xế`.
///
/// # Errors
/// [`ProcessError::Invalid`] khi cước không dương hoặc mã rỗng; [`ProcessError::Ledger`] từ lõi sổ.
pub fn plan_trip(event: &TripCompleted) -> Result<[NewEntry; 2], ProcessError> {
    if event.trip_id.trim().is_empty() || event.fare_dong <= 0 {
        return Err(ProcessError::Invalid(format!(
            "chuyến {:?} cước {} đồng",
            event.trip_id, event.fare_dong
        )));
    }
    let fare = Money::from_dong(event.fare_dong);
    let split = default_policy().split(fare)?;
    let out = fare.checked_neg().ok_or(LedgerError::Overflow)?;
    let customer = AccountId::new(&format!("customer:{}", event.customer_id))?;
    let driver = AccountId::new(&format!("driver:{}", event.driver_id))?;
    let clearing = AccountId::new(CLEARING)?;
    let capture = JournalEntryBuilder::new(&format!("trip:{}:capture", event.trip_id))
        .posting(customer, out)
        .posting(clearing.clone(), fare)
        .build()?;
    let split_entry = JournalEntryBuilder::new(&format!("trip:{}:split", event.trip_id))
        .posting(clearing, out)
        .posting(AccountId::new(REVENUE)?, split.platform_fee)
        .posting(driver, split.driver_net)
        .build()?;
    let convert = |entry: &ledger_core::JournalEntry, external_ref: Option<String>| NewEntry {
        key: entry.idempotency_key().to_string(),
        reversal_of: None,
        postings: entry
            .postings()
            .iter()
            .map(|p| (p.account.as_str().to_string(), p.amount.dong()))
            .collect(),
        external_ref,
    };
    Ok([
        convert(&capture, Some(psp_ref_for_trip(&event.trip_id))),
        convert(&split_entry, None),
    ])
}

/// Ghi MỘT bút toán, số dư projection và dòng outbox trong transaction của người gọi.
/// `Ok(None)` khi `idempotency_key` đã có: không ghi gì thêm.
///
/// # Errors
/// Lỗi SQL, ví dụ trigger kỳ đã đóng (`23001`) hoặc trigger cân bút toán (`23514`).
pub async fn post_entry(
    conn: &mut PgConnection,
    new: &NewEntry,
    at: Timestamp,
    out_topic: &str,
) -> Result<Option<i64>, sqlx::Error> {
    let at_text = at.to_string();
    let inserted = sqlx::query(
        "INSERT INTO journal_entries (idempotency_key, reversal_of, posted_at) \
         VALUES ($1, $2, $3::timestamptz) ON CONFLICT (idempotency_key) DO NOTHING RETURNING id",
    )
    .bind(&new.key)
    .bind(new.reversal_of)
    .bind(&at_text)
    .fetch_optional(&mut *conn)
    .await?;
    let Some(row) = inserted else {
        return Ok(None);
    };
    let id: i64 = row.get("id");
    for (index, (account, amount)) in new.postings.iter().enumerate() {
        sqlx::query("INSERT INTO accounts (id, kind) VALUES ($1, $2) ON CONFLICT (id) DO NOTHING")
            .bind(account)
            .bind(kind_of(account))
            .execute(&mut *conn)
            .await?;
        sqlx::query(
            "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(i32::try_from(index + 1).expect("ít hơn 2^31 dòng"))
        .bind(account)
        .bind(amount)
        .execute(&mut *conn)
        .await?;
        sqlx::query(
            "INSERT INTO account_balances (account_id, balance_dong, version, updated_at) \
             VALUES ($1, $2, 1, $3::timestamptz) \
             ON CONFLICT (account_id) DO UPDATE SET \
                 balance_dong = account_balances.balance_dong + EXCLUDED.balance_dong, \
                 version = account_balances.version + 1, updated_at = EXCLUDED.updated_at",
        )
        .bind(account)
        .bind(amount)
        .bind(&at_text)
        .execute(&mut *conn)
        .await?;
    }
    if let Some(external_ref) = &new.external_ref {
        sqlx::query("INSERT INTO psp_refs (entry_id, external_ref) VALUES ($1, $2)")
            .bind(id)
            .bind(external_ref)
            .execute(&mut *conn)
            .await?;
    }
    let event = LedgerEntryPosted {
        schema_version: SCHEMA_VERSION,
        entry_id: format!("JE-{id:06}"),
        idempotency_key: new.key.clone(),
        postings: new
            .postings
            .iter()
            .map(|(account, amount)| PostingDto {
                account: account.clone(),
                amount_dong: *amount,
            })
            .collect(),
        posted_at: at_text.clone(),
    };
    let payload = serde_json::to_string(&event).expect("sự kiện luôn serialize được");
    sqlx::query(
        "INSERT INTO outbox (topic, key, payload, created_at) \
         VALUES ($1, $2, $3::jsonb, $4::timestamptz)",
    )
    .bind(out_topic)
    .bind(&new.key)
    .bind(&payload)
    .bind(&at_text)
    .execute(&mut *conn)
    .await?;
    Ok(Some(id))
}

/// Xử lý một `TripCompleted`: đọc payload, rồi inbox + hai bút toán + outbox trong MỘT transaction.
/// Lỗi giữa chừng thì cả dòng inbox cũng rollback, nên lần giao lại sau (hoặc replay) vẫn được xử lý.
///
/// # Errors
/// [`ProcessError`]; lỗi dữ liệu là [`ProcessError::is_permanent`].
pub async fn apply_trip_completed(
    pool: &PgPool,
    ctx: &ApplyCtx<'_>,
    payload: &[u8],
) -> Result<Outcome, ProcessError> {
    let event = parse_trip_completed(payload, ctx.accepted)?;
    let plan = plan_trip(&event)?;
    let event_id = format!("TripCompleted:{}", event.trip_id);
    let mut tx = pool.begin().await?;
    let fresh = sqlx::query(
        "INSERT INTO inbox (event_id, processed_at) VALUES ($1, $2::timestamptz) \
         ON CONFLICT (event_id) DO NOTHING",
    )
    .bind(&event_id)
    .bind(ctx.at.to_string())
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if fresh == 0 {
        tx.rollback().await?;
        return Ok(Outcome::Duplicate);
    }
    let mut outcome = Outcome::Applied;
    for entry in &plan {
        if post_entry(&mut tx, entry, ctx.at, ctx.out_topic)
            .await?
            .is_none()
        {
            outcome = Outcome::EntryAlreadyExists;
        }
    }
    tx.commit().await?;
    Ok(outcome)
}

/// Dry-run của [`apply_trip_completed`]: cùng phép kiểm dữ liệu, cùng hai lớp idempotency, chỉ đọc.
/// Trả đúng kết quả mà lần chạy thật sẽ trả nếu sổ không đổi giữa hai lần.
///
/// # Errors
/// Như [`apply_trip_completed`].
pub async fn preview_trip_completed(
    pool: &PgPool,
    ctx: &ApplyCtx<'_>,
    payload: &[u8],
) -> Result<Outcome, ProcessError> {
    let event = parse_trip_completed(payload, ctx.accepted)?;
    let plan = plan_trip(&event)?;
    let event_id = format!("TripCompleted:{}", event.trip_id);
    let seen: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM inbox WHERE event_id = $1)")
        .bind(&event_id)
        .fetch_one(pool)
        .await?;
    if seen {
        return Ok(Outcome::Duplicate);
    }
    for entry in &plan {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM journal_entries WHERE idempotency_key = $1)",
        )
        .bind(&entry.key)
        .fetch_one(pool)
        .await?;
        if exists {
            return Ok(Outcome::EntryAlreadyExists);
        }
    }
    Ok(Outcome::Applied)
}

/// Đếm dòng bằng câu SQL literal.
///
/// # Errors
/// Lỗi SQL.
pub async fn scalar(pool: &PgPool, sql: &'static str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(sql).fetch_one(pool).await
}

/// Số bút toán.
///
/// # Errors
/// Lỗi SQL.
pub async fn count_entries(pool: &PgPool) -> Result<i64, sqlx::Error> {
    scalar(pool, "SELECT count(*) FROM journal_entries").await
}

/// Số dòng outbox.
///
/// # Errors
/// Lỗi SQL.
pub async fn count_outbox(pool: &PgPool) -> Result<i64, sqlx::Error> {
    scalar(pool, "SELECT count(*) FROM outbox").await
}

/// Số dòng inbox.
///
/// # Errors
/// Lỗi SQL.
pub async fn count_inbox(pool: &PgPool) -> Result<i64, sqlx::Error> {
    scalar(pool, "SELECT count(*) FROM inbox").await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trip_plan_has_the_canonical_two_entries() {
        let [capture, split] = plan_trip(&TripCompleted::sample("TN-0001")).unwrap();
        assert_eq!(capture.key, "trip:TN-0001:capture");
        assert_eq!(
            capture.postings,
            [
                ("customer:CU-001".to_string(), -25_500),
                (CLEARING.to_string(), 25_500)
            ]
        );
        assert_eq!(capture.external_ref.as_deref(), Some("vx_ch_0001"));
        assert_eq!(split.key, "trip:TN-0001:split");
        assert_eq!(
            split.postings,
            [
                (CLEARING.to_string(), -25_500),
                (REVENUE.to_string(), 5_100),
                ("driver:DR-001".to_string(), 20_400)
            ]
        );
        assert_eq!(split.external_ref, None);
    }

    #[test]
    fn free_trips_are_permanent_errors() {
        let mut event = TripCompleted::sample("TN-0001");
        event.fare_dong = 0;
        assert!(plan_trip(&event).unwrap_err().is_permanent());
    }
}
