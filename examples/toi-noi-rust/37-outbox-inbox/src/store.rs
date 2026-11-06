//! Ghi bút toán và outbox trong cùng một transaction sqlx (copy tối thiểu từ lab 24).

use ledger_core::default_policy;
use ledger_core::{AccountId, JournalEntry, LedgerError, Money, TripCharge, capture_entry};
use sqlx::{PgConnection, PgPool, Row};

use crate::events::{LedgerEntryPosted, PostingDto, SCHEMA_VERSION, TripCompleted};

/// Mã hiển thị `JE-000001` của id trong DB.
pub fn display_id(id: i64) -> String {
    format!("JE-{id:06}")
}

/// Bút toán vừa chèn: id do DB cấp và thời điểm ghi (chuỗi ISO 8601 UTC).
#[derive(Debug, Clone)]
pub struct Inserted {
    /// Id do DB cấp.
    pub id: i64,
    /// `posted_at` do PostgreSQL cấp, định dạng `2026-11-06T02:15:00.123456Z`.
    pub posted_at: String,
}

/// Chèn bút toán và các posting trong transaction của người gọi.
/// `Ok(None)`: `idempotency_key` đã có, không chèn gì.
///
/// # Errors
/// Lỗi SQL, ví dụ khóa ngoại 23503 khi tài khoản không tồn tại.
pub async fn insert_entry(
    conn: &mut PgConnection,
    entry: &JournalEntry,
) -> Result<Option<Inserted>, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO journal_entries (idempotency_key) VALUES ($1) \
         ON CONFLICT (idempotency_key) DO NOTHING \
         RETURNING id, to_char(posted_at AT TIME ZONE 'UTC', \
                               'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS posted_at",
    )
    .bind(entry.idempotency_key())
    .fetch_optional(&mut *conn)
    .await?;
    let Some(row) = row else {
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
    Ok(Some(Inserted {
        id,
        posted_at: row.get("posted_at"),
    }))
}

/// Thêm một dòng vào outbox: chỉ ghi ý định gửi, chưa chạm Kafka.
///
/// # Errors
/// Lỗi SQL.
pub async fn enqueue(
    conn: &mut PgConnection,
    topic: &str,
    key: &str,
    payload_json: &str,
) -> Result<i64, sqlx::Error> {
    // payload là jsonb: bind chuỗi rồi ép kiểu ở SQL, khỏi bật thêm feature `json` của sqlx.
    let row = sqlx::query(
        "INSERT INTO outbox (topic, key, payload) VALUES ($1, $2, $3::jsonb) RETURNING id",
    )
    .bind(topic)
    .bind(key)
    .bind(payload_json)
    .fetch_one(&mut *conn)
    .await?;
    Ok(row.get("id"))
}

/// Dựng sự kiện `LedgerEntryPosted` từ bút toán đã chèn.
pub fn entry_posted(inserted: &Inserted, entry: &JournalEntry) -> LedgerEntryPosted {
    LedgerEntryPosted {
        schema_version: SCHEMA_VERSION,
        entry_id: display_id(inserted.id),
        idempotency_key: entry.idempotency_key().to_string(),
        postings: entry
            .postings()
            .iter()
            .map(|p| PostingDto {
                account: p.account.as_str().to_string(),
                amount_dong: p.amount.dong(),
            })
            .collect(),
        posted_at: inserted.posted_at.clone(),
    }
}

/// Ghi bút toán và dòng outbox `LedgerEntryPosted` trong CÙNG transaction của người gọi.
/// Người gọi commit: cả hai cùng có hiệu lực hoặc cùng biến mất.
/// `Ok(None)` khi key đã có: bút toán cũ đã có outbox của nó, không thêm dòng nào.
///
/// # Errors
/// Lỗi SQL ở bút toán hoặc outbox.
pub async fn post_with_outbox(
    conn: &mut PgConnection,
    entry: &JournalEntry,
    topic: &str,
) -> Result<Option<Inserted>, sqlx::Error> {
    let Some(inserted) = insert_entry(&mut *conn, entry).await? else {
        return Ok(None);
    };
    let event = entry_posted(&inserted, entry);
    let payload = serde_json::to_string(&event).expect("sự kiện luôn serialize được");
    enqueue(&mut *conn, topic, entry.idempotency_key(), &payload).await?;
    Ok(Some(inserted))
}

/// Dựng bút toán thu cước từ `TripCompleted` bằng `ledger-core` (3 posting, phí 20%).
///
/// # Errors
/// [`LedgerError`] khi mã tài khoản rỗng hoặc cước làm bút toán không hợp lệ.
pub fn entry_from_trip(event: &TripCompleted) -> Result<JournalEntry, LedgerError> {
    let charge = TripCharge {
        trip_id: event.trip_id.clone(),
        customer: AccountId::new(&format!("customer:{}", event.customer_id))?,
        driver: AccountId::new(&format!("driver:{}", event.driver_id))?,
        fare: Money::from_dong(event.fare_dong),
    };
    capture_entry(&charge, &default_policy())
}

/// Đếm dòng bằng câu SQL literal (không ghép chuỗi).
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

/// Số dòng outbox, kèm số dòng chưa gửi.
///
/// # Errors
/// Lỗi SQL.
pub async fn count_outbox(pool: &PgPool) -> Result<(i64, i64), sqlx::Error> {
    Ok((
        scalar(pool, "SELECT count(*) FROM outbox").await?,
        scalar(
            pool,
            "SELECT count(*) FROM outbox WHERE published_at IS NULL",
        )
        .await?,
    ))
}

/// Số dòng inbox.
///
/// # Errors
/// Lỗi SQL.
pub async fn count_inbox(pool: &PgPool) -> Result<i64, sqlx::Error> {
    scalar(pool, "SELECT count(*) FROM inbox").await
}

/// Số dư projection của một tài khoản (0 nếu chưa có dòng).
///
/// # Errors
/// Lỗi SQL.
pub async fn balance_of(pool: &PgPool, account: &str) -> Result<i64, sqlx::Error> {
    let found: Option<i64> =
        sqlx::query_scalar("SELECT balance_dong FROM account_balances WHERE account_id = $1")
            .bind(account)
            .fetch_optional(pool)
            .await?;
    Ok(found.unwrap_or(0))
}
