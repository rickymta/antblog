//! Event log = `journal_entries` + `postings`. Ghi một bút toán và đọc log theo ba cách,
//! để thấy cách nào bỏ sót giao dịch commit muộn.

use std::collections::HashMap;

use ledger_core::JournalEntry;
use sqlx::{PgConnection, Row};

/// Vị trí trong log: `(tx_id, entry_id)`. `Ord` derive theo đúng thứ tự trường,
/// khớp với so sánh dòng `(tx_id, id)` trong SQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Cursor {
    /// xid của transaction đã ghi bút toán.
    pub tx_id: i64,
    /// `journal_entries.id` (identity, cấp lúc INSERT).
    pub entry_id: i64,
}

/// Một bút toán đã commit, kèm posting, đủ để chiếu ra projection.
#[derive(Debug, Clone)]
pub struct LogEvent {
    /// `journal_entries.id`.
    pub entry_id: i64,
    /// xid của transaction ghi.
    pub tx_id: i64,
    /// `idempotency_key`.
    pub key: String,
    /// Bút toán bị đảo (nếu đây là bút toán đảo).
    pub reversal_of: Option<i64>,
    /// Ngày của `posted_at` theo UTC, dạng `YYYY-MM-DD`.
    pub day_utc: String,
    /// Ngày của `posted_at` theo `Asia/Ho_Chi_Minh`.
    pub day_vn: String,
    /// `(account_id, amount_dong)` theo `line_no`.
    pub postings: Vec<(String, i64)>,
}

impl LogEvent {
    /// Vị trí của sự kiện trong log.
    pub fn cursor(&self) -> Cursor {
        Cursor {
            tx_id: self.tx_id,
            entry_id: self.entry_id,
        }
    }
}

/// Cách đọc log. Chỉ `Safe` đúng; hai cách đầu có mặt để lab dựng lại lỗi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadMode {
    /// `id > last_entry_id`. Bỏ sót nếu giao dịch có id nhỏ commit muộn.
    NaiveId,
    /// Chỉ đọc giao dịch đã xong (`tx_id < xmin`) nhưng con trỏ vẫn là `id`.
    HorizonId,
    /// Chỉ đọc giao dịch đã xong và con trỏ là `(tx_id, id)`. Cách dùng thật.
    Safe,
}

/// Ghi một bút toán (cùng transaction với người gọi). `tx_id` tự lấy từ `pg_current_xact_id()`.
///
/// # Errors
/// Lỗi SQL, ví dụ trùng `idempotency_key` hoặc tài khoản chưa có.
pub async fn append_entry(
    conn: &mut PgConnection,
    entry: &JournalEntry,
    posted_at: &str,
    reversal_of: Option<i64>,
) -> Result<i64, sqlx::Error> {
    let id: i64 = sqlx::query(
        "INSERT INTO journal_entries (idempotency_key, reversal_of, posted_at) \
         VALUES ($1, $2, $3::timestamptz) RETURNING id",
    )
    .bind(entry.idempotency_key())
    .bind(reversal_of)
    .bind(posted_at)
    .fetch_one(&mut *conn)
    .await?
    .get("id");
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
    Ok(id)
}

/// Mọi giao dịch có xid nhỏ hơn giá trị này đã commit hoặc rollback (theo tài liệu
/// PostgreSQL về `pg_snapshot_xmin`): nó là xid nhỏ nhất còn đang chạy tại thời điểm chụp snapshot.
///
/// # Errors
/// Lỗi SQL.
pub async fn horizon(conn: &mut PgConnection) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT pg_snapshot_xmin(pg_current_snapshot())::text::bigint")
        .fetch_one(&mut *conn)
        .await
}

/// Vị trí lớn nhất trong những bút toán đã commit (kể cả chưa qua khoảng an toàn).
///
/// # Errors
/// Lỗi SQL.
pub async fn head(conn: &mut PgConnection) -> Result<Option<Cursor>, sqlx::Error> {
    let row =
        sqlx::query("SELECT tx_id, id FROM journal_entries ORDER BY tx_id DESC, id DESC LIMIT 1")
            .fetch_optional(&mut *conn)
            .await?;
    Ok(row.map(|r| Cursor {
        tx_id: r.get("tx_id"),
        entry_id: r.get("id"),
    }))
}

/// Số bút toán đã commit nằm sau `cursor`: độ trễ (lag) của một projection tính bằng bút toán.
///
/// # Errors
/// Lỗi SQL.
pub async fn entries_behind(conn: &mut PgConnection, cursor: Cursor) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM journal_entries WHERE (tx_id, id) > ($1, $2)")
        .bind(cursor.tx_id)
        .bind(cursor.entry_id)
        .fetch_one(&mut *conn)
        .await
}

// Các cột của một sự kiện; ngày được tính ở database để Rust không cần bảng múi giờ.
macro_rules! event_columns {
    () => {
        "id, tx_id, idempotency_key, reversal_of, \
         (posted_at AT TIME ZONE 'UTC')::date::text AS day_utc, \
         (posted_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date::text AS day_vn"
    };
}

/// Đọc tối đa `limit` sự kiện sau `from`. `upper` (nếu có) chặn trên theo `(tx_id, id)`, dùng khi
/// chuyển phiên bản projection. Với `Naive`/`HorizonId` chỉ `from.entry_id` được dùng.
///
/// # Errors
/// Lỗi SQL.
pub async fn read_batch(
    conn: &mut PgConnection,
    mode: ReadMode,
    from: Cursor,
    upper: Option<Cursor>,
    limit: i64,
) -> Result<Vec<LogEvent>, sqlx::Error> {
    let rows = match mode {
        ReadMode::NaiveId => {
            sqlx::query(
                concat!(
                    "SELECT ",
                    event_columns!(),
                    " FROM journal_entries WHERE id > $1 ORDER BY id LIMIT $2"
                ),
            )
            .bind(from.entry_id)
            .bind(limit)
            .fetch_all(&mut *conn)
            .await?
        }
        ReadMode::HorizonId => {
            sqlx::query(
                concat!(
                    "SELECT ",
                    event_columns!(),
                    " FROM journal_entries \
                     WHERE id > $1 AND tx_id < pg_snapshot_xmin(pg_current_snapshot())::text::bigint \
                     ORDER BY id LIMIT $2"
                ),
            )
            .bind(from.entry_id)
            .bind(limit)
            .fetch_all(&mut *conn)
            .await?
        }
        ReadMode::Safe => {
            let upper = upper.unwrap_or(Cursor {
                tx_id: i64::MAX,
                entry_id: i64::MAX,
            });
            sqlx::query(
                concat!(
                    "SELECT ",
                    event_columns!(),
                    " FROM journal_entries \
                     WHERE (tx_id, id) > ($1, $2) AND (tx_id, id) <= ($3, $4) \
                       AND tx_id < pg_snapshot_xmin(pg_current_snapshot())::text::bigint \
                     ORDER BY tx_id, id LIMIT $5"
                ),
            )
            .bind(from.tx_id)
            .bind(from.entry_id)
            .bind(upper.tx_id)
            .bind(upper.entry_id)
            .bind(limit)
            .fetch_all(&mut *conn)
            .await?
        }
    };
    let mut events: Vec<LogEvent> = rows
        .iter()
        .map(|row| LogEvent {
            entry_id: row.get("id"),
            tx_id: row.get("tx_id"),
            key: row.get("idempotency_key"),
            reversal_of: row.get("reversal_of"),
            day_utc: row.get("day_utc"),
            day_vn: row.get("day_vn"),
            postings: Vec::new(),
        })
        .collect();
    if events.is_empty() {
        return Ok(events);
    }
    let ids: Vec<i64> = events.iter().map(|e| e.entry_id).collect();
    let position: HashMap<i64, usize> = ids.iter().enumerate().map(|(i, id)| (*id, i)).collect();
    let posting_rows = sqlx::query(
        "SELECT entry_id, account_id, amount_dong FROM postings \
         WHERE entry_id = ANY($1) ORDER BY entry_id, line_no",
    )
    .bind(&ids)
    .fetch_all(&mut *conn)
    .await?;
    for row in posting_rows {
        let entry_id: i64 = row.get("entry_id");
        if let Some(index) = position.get(&entry_id) {
            events[*index]
                .postings
                .push((row.get("account_id"), row.get("amount_dong")));
        }
    }
    Ok(events)
}
