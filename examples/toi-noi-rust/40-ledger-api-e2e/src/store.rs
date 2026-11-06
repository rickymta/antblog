//! Lớp lưu trữ PostgreSQL: ghi bút toán, inbox và outbox trong MỘT transaction, đọc số dư.
//! Rút gọn từ bài 24 (transaction, `ON CONFLICT`), bài 25 (schema) và bài 26 (repository).

use std::collections::BTreeMap;
use std::str::FromStr;

use ledger_core::{
    AccountId, JournalEntry, LedgerError, Money, Posting, TripCharge, capture_entry, default_policy,
};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::types::Json;
use sqlx::{PgConnection, Row};
use thiserror::Error;

use crate::config::{Config, Fault};
use crate::events::{LedgerEntryPosted, PostingDto, SCHEMA_VERSION, TripCompleted};

/// Lỗi của lớp lưu trữ: sai nghiệp vụ (đừng thử lại) tách khỏi lỗi hạ tầng (thử lại được).
#[derive(Debug, Error)]
pub enum StoreError {
    /// Sự kiện hoặc yêu cầu sai nội dung: thử lại bao nhiêu lần cũng vẫn sai.
    #[error("dữ liệu không hợp lệ: {0}")]
    Invalid(String),
    /// Bút toán vi phạm bất biến của `ledger-core`.
    #[error(transparent)]
    Domain(#[from] LedgerError),
    /// Cùng `idempotency_key` nhưng khác nội dung bản đã ghi.
    #[error("idempotency_key {key} đã dùng cho bút toán khác nội dung")]
    IdempotencyConflict {
        /// Key bị trùng.
        key: String,
    },
    /// Điểm chết giả lập của test.
    #[error("process chết giả lập trước khi commit")]
    InjectedCrash,
    /// Lỗi PostgreSQL hoặc kết nối.
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

impl StoreError {
    /// Lỗi vĩnh viễn: consumer bỏ qua sự kiện thay vì thử lại mãi.
    pub fn is_permanent(&self) -> bool {
        matches!(
            self,
            StoreError::Invalid(_) | StoreError::Domain(_) | StoreError::IdempotencyConflict { .. }
        )
    }
}

/// Kết quả xử lý một `TripCompleted`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Applied {
    /// Bút toán mới đã ghi, kèm sự kiện đã xếp vào outbox.
    Posted(LedgerEntryPosted),
    /// Inbox đã có sự kiện này: không ghi gì.
    Duplicate,
}

/// Kết quả ghi một bút toán.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostOutcome {
    /// Bút toán mới, kèm sự kiện đã xếp vào outbox.
    Created(LedgerEntryPosted),
    /// Key đã có, nội dung trùng: mã bút toán cũ.
    Replayed {
        /// Mã dạng `JE-000001`.
        entry_id: String,
    },
}

/// Cổng vào PostgreSQL của service.
#[derive(Clone)]
pub struct Store {
    pool: PgPool,
    topic_out: String,
    fault: Fault,
}

/// Mã hiển thị `JE-000001` của id trong DB.
pub fn display_id(id: i64) -> String {
    format!("JE-{id:06}")
}

/// Loại tài khoản suy từ tiền tố mã; mã lạ bị từ chối thay vì mở bừa.
fn account_kind(account: &str) -> Option<&'static str> {
    match account {
        "platform:revenue" => Some("revenue"),
        a if a.starts_with("psp:") => Some("asset"),
        a if a.starts_with("customer:") || a.starts_with("driver:") => Some("liability"),
        _ => None,
    }
}

impl Store {
    /// Mở pool với `search_path` trỏ vào schema của cấu hình, rồi dựng bảng nếu chưa có.
    ///
    /// # Errors
    /// Không kết nối được hoặc DDL lỗi.
    pub async fn connect(cfg: &Config) -> Result<Store, StoreError> {
        let mut options = PgConnectOptions::from_str(&cfg.pg_url)?;
        if let Some(schema) = &cfg.pg_schema {
            options = options.options([("search_path", schema.as_str())]);
        }
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await?;
        let store = Store {
            pool,
            topic_out: cfg.topic_out.clone(),
            fault: cfg.fault,
        };
        store.migrate().await?;
        Ok(store)
    }

    // Chỉ dựng bảng khi chưa có `accounts` trong schema hiện tại. Production dùng migration thật (bài 23).
    async fn migrate(&self) -> Result<(), StoreError> {
        let exists: Option<String> = sqlx::query_scalar("SELECT to_regclass('accounts')::text")
            .fetch_one(&self.pool)
            .await?;
        if exists.is_none() {
            sqlx::raw_sql(include_str!("../sql/schema.sql"))
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    /// Pool bên dưới (relay và test dùng).
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// `SELECT 1`: PostgreSQL còn trả lời không (cho `/readyz`).
    ///
    /// # Errors
    /// Lỗi kết nối hoặc truy vấn.
    pub async fn ping(&self) -> Result<(), sqlx::Error> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
    }

    /// Số dư hiện tại; `None` nếu tài khoản chưa từng có bút toán.
    ///
    /// # Errors
    /// Lỗi PostgreSQL.
    pub async fn balance(&self, account: &str) -> Result<Option<i64>, sqlx::Error> {
        sqlx::query_scalar("SELECT balance_dong FROM account_balances WHERE account_id = $1")
            .bind(account)
            .fetch_optional(&self.pool)
            .await
    }

    /// Xử lý một `TripCompleted`: inbox, bút toán, số dư và outbox cùng một transaction.
    /// Sự kiện đã có trong inbox thì không làm gì.
    ///
    /// # Errors
    /// [`StoreError::Invalid`]/[`StoreError::Domain`] khi sự kiện sai; lỗi DB khi hạ tầng hỏng
    /// (transaction bị drop nên tự rollback, inbox không giữ dấu vết).
    pub async fn apply_trip_completed(&self, ev: &TripCompleted) -> Result<Applied, StoreError> {
        let entry = entry_for_trip(ev)?;
        let mut tx = self.pool.begin().await?;
        let fresh = sqlx::query(
            "INSERT INTO inbox (event_id) VALUES ($1) ON CONFLICT (event_id) DO NOTHING",
        )
        .bind(ev.event_id())
        .execute(&mut *tx)
        .await?
        .rows_affected();
        if fresh == 0 {
            tx.rollback().await?;
            return Ok(Applied::Duplicate);
        }
        let outcome = post_in_tx(&mut tx, &entry, &self.topic_out).await?;
        if self.fault == Fault::BeforeDbCommit {
            return Err(StoreError::InjectedCrash); // `tx` bị drop: rollback, không dấu vết
        }
        tx.commit().await?;
        Ok(match outcome {
            PostOutcome::Created(event) => Applied::Posted(event),
            PostOutcome::Replayed { .. } => Applied::Duplicate,
        })
    }

    /// Ghi một bút toán thủ công (gRPC `PostEntry`), idempotent theo key.
    ///
    /// # Errors
    /// Bút toán sai bất biến, tài khoản có tiền tố lạ, key xung đột hoặc lỗi DB.
    pub async fn post_entry(
        &self,
        key: &str,
        postings: Vec<(String, i64)>,
    ) -> Result<PostOutcome, StoreError> {
        // `ledger-core` nhận posting 0 đồng; CHECK của bảng thì không. Chặn ở biên để client nhận
        // INVALID_ARGUMENT thay vì INTERNAL.
        if let Some((account, _)) = postings.iter().find(|(_, dong)| *dong == 0) {
            return Err(StoreError::Invalid(format!("posting 0 đồng ở {account}")));
        }
        let postings = postings
            .into_iter()
            .map(|(account, dong)| {
                Ok(Posting::new(
                    AccountId::new(&account)?,
                    Money::from_dong(dong),
                ))
            })
            .collect::<Result<Vec<_>, LedgerError>>()?;
        let entry = JournalEntry::try_new(key, postings)?;
        let mut tx = self.pool.begin().await?;
        let outcome = post_in_tx(&mut tx, &entry, &self.topic_out).await?;
        tx.commit().await?;
        Ok(outcome)
    }
}

/// `TripCompleted` → bút toán thu cước qua `ledger-core` (25.500 → -25.500, 5.100, 20.400).
fn entry_for_trip(ev: &TripCompleted) -> Result<JournalEntry, StoreError> {
    if ev.schema_version != SCHEMA_VERSION {
        return Err(StoreError::Invalid(format!(
            "schemaVersion {} chưa hỗ trợ",
            ev.schema_version
        )));
    }
    if ev.fare_dong <= 0 {
        return Err(StoreError::Invalid(format!(
            "fareDong {} <= 0",
            ev.fare_dong
        )));
    }
    let charge = TripCharge {
        trip_id: ev.trip_id.clone(),
        customer: AccountId::new(&format!("customer:{}", ev.customer_id))?,
        driver: AccountId::new(&format!("driver:{}", ev.driver_id))?,
        fare: Money::from_dong(ev.fare_dong),
    };
    Ok(capture_entry(&charge, &default_policy())?)
}

/// Ghi bút toán, số dư và outbox trong transaction của người gọi.
async fn post_in_tx(
    conn: &mut PgConnection,
    entry: &JournalEntry,
    topic_out: &str,
) -> Result<PostOutcome, StoreError> {
    let inserted = sqlx::query(
        "INSERT INTO journal_entries (idempotency_key) VALUES ($1) \
         ON CONFLICT (idempotency_key) DO NOTHING \
         RETURNING id, to_char(posted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS posted_at",
    )
    .bind(entry.idempotency_key())
    .fetch_optional(&mut *conn)
    .await?;
    let Some(row) = inserted else {
        return replayed(conn, entry).await;
    };
    let id: i64 = row.get("id");
    let posted_at: String = row.get("posted_at");

    // Cộng dồn theo tài khoản và đi theo thứ tự mã: hai transaction chạm cùng tài khoản
    // luôn khoá theo cùng thứ tự nên không deadlock.
    let mut deltas: BTreeMap<&str, i64> = BTreeMap::new();
    for (index, posting) in entry.postings().iter().enumerate() {
        let account = posting.account.as_str();
        let kind = account_kind(account)
            .ok_or_else(|| StoreError::Invalid(format!("tài khoản lạ: {account}")))?;
        sqlx::query("INSERT INTO accounts (id, kind) VALUES ($1, $2) ON CONFLICT (id) DO NOTHING")
            .bind(account)
            .bind(kind)
            .execute(&mut *conn)
            .await?;
        let line_no = i32::try_from(index + 1).expect("bút toán không có hàng tỷ dòng");
        sqlx::query(
            "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(line_no)
        .bind(account)
        .bind(posting.amount.dong())
        .execute(&mut *conn)
        .await?;
        let slot = deltas.entry(account).or_insert(0);
        *slot = slot
            .checked_add(posting.amount.dong())
            .ok_or(LedgerError::Overflow)?;
    }
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

    let event = LedgerEntryPosted {
        schema_version: SCHEMA_VERSION,
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
    // Cùng transaction: bút toán và việc "cần phát" cùng có hoặc cùng không.
    sqlx::query("INSERT INTO outbox (topic, key, payload) VALUES ($1, $2, $3)")
        .bind(topic_out)
        .bind(&event.idempotency_key)
        .bind(Json(&event))
        .execute(&mut *conn)
        .await?;
    Ok(PostOutcome::Created(event))
}

// Key đã có: so nội dung với bản cũ. Trùng là replay, khác là xung đột.
async fn replayed(
    conn: &mut PgConnection,
    entry: &JournalEntry,
) -> Result<PostOutcome, StoreError> {
    let conflict = || StoreError::IdempotencyConflict {
        key: entry.idempotency_key().to_string(),
    };
    let head = sqlx::query("SELECT id FROM journal_entries WHERE idempotency_key = $1")
        .bind(entry.idempotency_key())
        .fetch_one(&mut *conn)
        .await?;
    let id: i64 = head.get("id");
    let rows = sqlx::query(
        "SELECT account_id, amount_dong FROM postings WHERE entry_id = $1 ORDER BY line_no",
    )
    .bind(id)
    .fetch_all(&mut *conn)
    .await?;
    let old: Vec<(String, i64)> = rows
        .iter()
        .map(|r| (r.get("account_id"), r.get("amount_dong")))
        .collect();
    let new: Vec<(String, i64)> = entry
        .postings()
        .iter()
        .map(|p| (p.account.as_str().to_string(), p.amount.dong()))
        .collect();
    if old == new {
        Ok(PostOutcome::Replayed {
            entry_id: display_id(id),
        })
    } else {
        Err(conflict())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trip_5km_splits_into_three_postings() {
        let entry = entry_for_trip(&TripCompleted::sample("TN-0001", 25_500)).unwrap();
        let lines: Vec<(&str, i64)> = entry
            .postings()
            .iter()
            .map(|p| (p.account.as_str(), p.amount.dong()))
            .collect();
        assert_eq!(
            lines,
            vec![
                ("customer:CU-001", -25_500),
                ("platform:revenue", 5_100),
                ("driver:DR-001", 20_400),
            ]
        );
        assert_eq!(entry.idempotency_key(), "trip:TN-0001:capture");
    }

    #[test]
    fn bad_events_are_permanent_errors() {
        let mut ev = TripCompleted::sample("TN-0001", 25_500);
        ev.schema_version = 2;
        assert!(entry_for_trip(&ev).unwrap_err().is_permanent());
        let free = TripCompleted::sample("TN-0001", 0);
        assert!(entry_for_trip(&free).unwrap_err().is_permanent());
    }

    #[test]
    fn account_kinds_by_prefix() {
        assert_eq!(account_kind("driver:DR-001"), Some("liability"));
        assert_eq!(account_kind("platform:revenue"), Some("revenue"));
        assert_eq!(account_kind("psp:vixanh:clearing"), Some("asset"));
        assert_eq!(account_kind("lạ:1"), None);
    }
}
