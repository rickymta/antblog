//! Ghi bút toán, inbox và outbox trong MỘT transaction. Rút gọn từ lab 24 và 26.

use std::time::Duration;

use anyhow::{Context, Result};
use ledger_core::{
    AccountId, JournalEntry, Money, Posting, TripCharge, capture_entry, default_policy,
};
use sqlx::postgres::PgPool;
use sqlx::{PgConnection, Row};

use crate::events::{LedgerEntryPosted, PostingDto, TripCompleted};

/// Kết quả xử lý một `TripCompleted`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applied {
    /// Bút toán mới với id này đã commit.
    Posted(i64),
    /// Đã xử lý từ trước: không ghi gì.
    Duplicate,
}

/// Kết quả ghi bút toán qua HTTP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostOutcome {
    /// Key mới: 201.
    Created(i64),
    /// Key đã có, nội dung trùng: 200.
    Replayed(i64),
    /// Key đã có, nội dung khác: 409.
    Conflict,
}

/// Mã hiển thị `JE-000001`.
pub fn display_id(id: i64) -> String {
    format!("JE-{id:06}")
}

/// Ghi bút toán, số dư và dòng outbox trong transaction của người gọi.
/// `None` nghĩa là `idempotency_key` đã tồn tại (không chèn gì).
///
/// # Errors
/// Lỗi SQL; người gọi để transaction tự rollback.
pub async fn write_entry(
    conn: &mut PgConnection,
    entry: &JournalEntry,
    topic_out: &str,
) -> Result<Option<i64>, sqlx::Error> {
    let inserted = sqlx::query(
        "INSERT INTO journal_entries (idempotency_key) VALUES ($1) \
         ON CONFLICT (idempotency_key) DO NOTHING \
         RETURNING id, to_char(posted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS posted_at",
    )
    .bind(entry.idempotency_key())
    .fetch_optional(&mut *conn)
    .await?;
    let Some(row) = inserted else {
        return Ok(None);
    };
    let id: i64 = row.get("id");
    let posted_at: String = row.get("posted_at");

    for (index, posting) in entry.postings().iter().enumerate() {
        let line_no = i32::try_from(index + 1).expect("bút toán không có hàng tỷ dòng");
        sqlx::query(
            "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(line_no)
        .bind(posting.account.as_str())
        .bind(posting.amount.dong())
        .execute(&mut *conn)
        .await?;
    }

    // Cập nhật số dư theo thứ tự tài khoản cố định để hai transaction không khoá chéo nhau.
    let mut deltas: Vec<(&str, i64)> = entry
        .postings()
        .iter()
        .map(|p| (p.account.as_str(), p.amount.dong()))
        .collect();
    deltas.sort_unstable();
    for (account, delta) in deltas {
        sqlx::query(
            "INSERT INTO account_balances (account_id, balance_dong, version) VALUES ($1, $2, 1) \
             ON CONFLICT (account_id) DO UPDATE SET \
                 balance_dong = account_balances.balance_dong + EXCLUDED.balance_dong, \
                 version = account_balances.version + 1, updated_at = now()",
        )
        .bind(account)
        .bind(delta)
        .execute(&mut *conn)
        .await?;
    }

    // Sự kiện phát ra nằm cùng transaction: commit thì có, rollback thì không.
    let event = LedgerEntryPosted {
        schema_version: 1,
        entry_id: display_id(id),
        idempotency_key: entry.idempotency_key().to_string(),
        postings: entry
            .postings()
            .iter()
            .map(|p| PostingDto {
                account: p.account.as_str().to_string(),
                amount_dong: p.amount.dong(),
            })
            .collect(),
        posted_at,
    };
    sqlx::query("INSERT INTO outbox (topic, key, payload) VALUES ($1, $2, $3::text::jsonb)")
        .bind(topic_out)
        .bind(entry.idempotency_key())
        .bind(serde_json::to_string(&event).expect("sự kiện luôn serialize được"))
        .execute(&mut *conn)
        .await?;
    Ok(Some(id))
}

/// Xử lý một `TripCompleted`: inbox, bút toán, outbox, rồi mới commit.
/// `hold` mô phỏng một bước chậm nằm TRONG transaction (để thấy cửa sổ "đang xử lý dở").
///
/// # Errors
/// Lỗi dữ liệu (tài khoản rỗng, tổng lệch) hoặc lỗi SQL; transaction bị drop nên rollback.
pub async fn apply_trip_completed(
    pool: &PgPool,
    event: &TripCompleted,
    topic_out: &str,
    hold: Duration,
) -> Result<Applied> {
    let mut tx = pool.begin().await?;
    let event_id = format!("trip.completed:{}", event.trip_id);
    let fresh =
        sqlx::query("INSERT INTO inbox (event_id) VALUES ($1) ON CONFLICT (event_id) DO NOTHING")
            .bind(&event_id)
            .execute(&mut *tx)
            .await?
            .rows_affected();
    if fresh == 0 {
        tx.rollback().await?;
        return Ok(Applied::Duplicate);
    }

    let charge = TripCharge {
        trip_id: event.trip_id.clone(),
        customer: AccountId::new(&format!("customer:{}", event.customer_id))?,
        driver: AccountId::new(&format!("driver:{}", event.driver_id))?,
        fare: Money::from_dong(event.fare_dong),
    };
    let entry = capture_entry(&charge, &default_policy())?;
    let Some(id) = write_entry(&mut tx, &entry, topic_out).await? else {
        tx.rollback().await?;
        return Ok(Applied::Duplicate);
    };
    if !hold.is_zero() {
        tokio::time::sleep(hold).await;
    }
    tx.commit().await?;
    Ok(Applied::Posted(id))
}

/// Đọc lại bút toán theo key để so với yêu cầu replay.
async fn load_by_key(conn: &mut PgConnection, key: &str) -> Result<Option<(i64, JournalEntry)>> {
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

/// `POST /v1/entries`: ghi idempotent. Cũng dùng `hold` để thấy một request đang bay.
///
/// # Errors
/// Lỗi SQL (ví dụ tài khoản không tồn tại: SQLSTATE 23503).
pub async fn post_entry(
    pool: &PgPool,
    entry: &JournalEntry,
    topic_out: &str,
    hold: Duration,
) -> Result<PostOutcome> {
    let mut tx = pool.begin().await?;
    match write_entry(&mut tx, entry, topic_out).await? {
        Some(id) => {
            if !hold.is_zero() {
                tokio::time::sleep(hold).await;
            }
            tx.commit().await.context("commit bút toán")?;
            Ok(PostOutcome::Created(id))
        }
        None => {
            let existing = load_by_key(&mut tx, entry.idempotency_key()).await?;
            tx.rollback().await?;
            Ok(match existing {
                Some((id, old)) if &old == entry => PostOutcome::Replayed(id),
                _ => PostOutcome::Conflict,
            })
        }
    }
}
