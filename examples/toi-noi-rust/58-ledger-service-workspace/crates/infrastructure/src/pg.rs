//! Adapter PostgreSQL cho port `LedgerStore`. Mỗi phương thức của port là MỘT transaction.

use std::collections::BTreeMap;
use std::str::FromStr;

use jiff::Timestamp;
use ledger_application::model::{
    Applied, Funds, HoldOutcome, PostOutcome, PostedEntry, PostedLine,
};
use ledger_application::{Error, LedgerStore};
use ledger_domain::chart::kind_of;
use ledger_domain::hold::{
    ActiveHold, HoldError, HoldStatus, NewHold, Settlement, ensure_available,
};
use ledger_domain::refund::{PaymentSnapshot, RefundError, refund_entry, refund_key};
use ledger_domain::trip::{CapturePlan, TripFare, hold_key};
use ledger_domain::{AccountId, JournalEntry, LedgerError, Money};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::types::Json;
use sqlx::{PgConnection, Row};

use crate::wire::LedgerEntryPosted;

/// Bốn file SQL theo thứ tự. 01 và 02 là bản chuẩn của bài 25, giữ nguyên từng byte.
const SCHEMA: [&str; 4] = [
    include_str!("../sql/01_schema.sql"),
    include_str!("../sql/02_balance_check.sql"),
    include_str!("../sql/03_append_only.sql"),
    include_str!("../sql/04_service_tables.sql"),
];

/// Mã hiển thị `JE-000001` của id trong DB.
pub fn display_id(id: i64) -> String {
    format!("JE-{id:06}")
}

/// Phân loại lỗi sqlx: kết nối và xung đột serialization là tạm thời (thử lại được),
/// còn lại là lỗi lưu trữ. Chi tiết SQL không đi ra ngoài.
pub fn db_error(err: &sqlx::Error) -> Error {
    match err {
        sqlx::Error::Io(_)
        | sqlx::Error::PoolTimedOut
        | sqlx::Error::PoolClosed
        | sqlx::Error::WorkerCrashed => Error::Unavailable(err.to_string()),
        sqlx::Error::Database(db) if matches!(db.code().as_deref(), Some("40001" | "40P01")) => {
            Error::Unavailable(err.to_string())
        }
        _ => Error::Storage(err.to_string()),
    }
}

// Cho phép dùng `?` trên kết quả sqlx trong file này.
trait Db<T> {
    fn db(self) -> Result<T, Error>;
}

impl<T> Db<T> for Result<T, sqlx::Error> {
    fn db(self) -> Result<T, Error> {
        self.map_err(|e| db_error(&e))
    }
}

/// Cổng vào PostgreSQL của service.
#[derive(Clone)]
pub struct PgStore {
    pool: PgPool,
    topic_out: String,
}

impl PgStore {
    /// Mở pool với `search_path` trỏ vào `schema` (nếu có), rồi dựng bảng nếu chưa có.
    ///
    /// # Errors
    /// [`Error::Unavailable`] khi không kết nối được; [`Error::Storage`] khi DDL lỗi.
    pub async fn connect(
        pg_url: &str,
        schema: Option<&str>,
        topic_out: &str,
    ) -> Result<PgStore, Error> {
        let mut options = PgConnectOptions::from_str(pg_url).db()?;
        if let Some(schema) = schema {
            options = options.options([("search_path", schema)]);
        }
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await
            .db()?;
        let store = PgStore {
            pool,
            topic_out: topic_out.to_string(),
        };
        store.migrate().await?;
        Ok(store)
    }

    // Chỉ dựng bảng khi chưa có `accounts` trong schema hiện tại. Production dùng migration thật (bài 23, 56).
    async fn migrate(&self) -> Result<(), Error> {
        let exists: Option<String> = sqlx::query_scalar("SELECT to_regclass('accounts')::text")
            .fetch_one(&self.pool)
            .await
            .db()?;
        if exists.is_none() {
            for sql in SCHEMA {
                sqlx::raw_sql(sql).execute(&self.pool).await.db()?;
            }
        }
        Ok(())
    }

    /// Pool bên dưới (relay và test dùng).
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// Một bút toán vừa được thử ghi trong transaction của người gọi.
enum Written {
    Created(PostedEntry),
    Replayed { entry_id: String },
}

/// Ghi bút toán và outbox; gom thay đổi số dư vào `deltas` để cập nhật MỘT lần ở cuối transaction.
async fn write_entry(
    conn: &mut PgConnection,
    entry: &JournalEntry,
    topic_out: &str,
    deltas: &mut BTreeMap<String, i64>,
) -> Result<Written, Error> {
    let inserted = sqlx::query(
        "INSERT INTO journal_entries (idempotency_key) VALUES ($1) \
         ON CONFLICT (idempotency_key) DO NOTHING \
         RETURNING id, to_char(posted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS posted_at",
    )
    .bind(entry.idempotency_key())
    .fetch_optional(&mut *conn)
    .await
    .db()?;
    let Some(row) = inserted else {
        return replayed(conn, entry).await;
    };
    let id: i64 = row.get("id");
    let posted_at: String = row.get("posted_at");

    for (index, posting) in entry.postings().iter().enumerate() {
        let account = posting.account.as_str();
        let kind = kind_of(&posting.account)
            .ok_or_else(|| Error::Invalid(format!("tài khoản ngoài sơ đồ chuẩn: {account}")))?;
        sqlx::query("INSERT INTO accounts (id, kind) VALUES ($1, $2) ON CONFLICT (id) DO NOTHING")
            .bind(account)
            .bind(kind.as_str())
            .execute(&mut *conn)
            .await
            .db()?;
        let line_no = i32::try_from(index + 1).expect("bút toán không có hàng tỷ dòng");
        sqlx::query(
            "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(line_no)
        .bind(account)
        .bind(posting.amount.dong())
        .execute(&mut *conn)
        .await
        .db()?;
        let slot = deltas.entry(account.to_string()).or_insert(0);
        *slot = slot
            .checked_add(posting.amount.dong())
            .ok_or(LedgerError::Overflow)?;
    }

    let posted = PostedEntry {
        entry_id: display_id(id),
        idempotency_key: entry.idempotency_key().to_string(),
        postings: entry
            .postings()
            .iter()
            .map(|p| PostedLine {
                account: p.account.as_str().to_string(),
                amount_dong: p.amount.dong(),
            })
            .collect(),
        posted_at,
    };
    // Cùng transaction: bút toán và việc "cần phát" cùng có hoặc cùng không.
    sqlx::query("INSERT INTO outbox (topic, key, payload) VALUES ($1, $2, $3)")
        .bind(topic_out)
        .bind(&posted.idempotency_key)
        .bind(Json(LedgerEntryPosted::from(&posted)))
        .execute(&mut *conn)
        .await
        .db()?;
    Ok(Written::Created(posted))
}

// Key đã có: so nội dung với bản cũ. Trùng là replay, khác là xung đột.
async fn replayed(conn: &mut PgConnection, entry: &JournalEntry) -> Result<Written, Error> {
    let head = sqlx::query("SELECT id FROM journal_entries WHERE idempotency_key = $1")
        .bind(entry.idempotency_key())
        .fetch_one(&mut *conn)
        .await
        .db()?;
    let id: i64 = head.get("id");
    let rows = sqlx::query(
        "SELECT account_id, amount_dong FROM postings WHERE entry_id = $1 ORDER BY line_no",
    )
    .bind(id)
    .fetch_all(&mut *conn)
    .await
    .db()?;
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
        Ok(Written::Replayed {
            entry_id: display_id(id),
        })
    } else {
        Err(Error::IdempotencyConflict {
            key: entry.idempotency_key().to_string(),
        })
    }
}

/// Cập nhật `account_balances` theo thứ tự mã tài khoản (`BTreeMap` đã sắp xếp): hai transaction
/// chạm cùng tập tài khoản luôn khóa theo cùng thứ tự nên không deadlock.
async fn apply_balances(
    conn: &mut PgConnection,
    deltas: &BTreeMap<String, i64>,
) -> Result<(), Error> {
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
        .await
        .db()?;
    }
    Ok(())
}

fn parse_ts(raw: &str) -> Result<Timestamp, Error> {
    raw.parse()
        .map_err(|e| Error::Storage(format!("thời điểm lạ {raw:?}: {e}")))
}

// Hold của chuyến, khóa dòng. Chỉ trả hold ACTIVE; hold đã đóng thì không đụng.
async fn lock_trip_hold(
    conn: &mut PgConnection,
    trip_id: &str,
) -> Result<Option<(i64, ActiveHold)>, Error> {
    let row = sqlx::query(
        "SELECT id, amount_dong, status, \
                to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS expires_at \
         FROM holds WHERE idempotency_key = $1 FOR UPDATE",
    )
    .bind(hold_key(trip_id))
    .fetch_optional(&mut *conn)
    .await
    .db()?;
    let Some(row) = row else { return Ok(None) };
    let status: String = row.get("status");
    if HoldStatus::parse(&status) != Some(HoldStatus::Active) {
        return Ok(None);
    }
    let expires_at: String = row.get("expires_at");
    Ok(Some((
        row.get("id"),
        ActiveHold {
            amount: Money::from_dong(row.get("amount_dong")),
            expires_at: parse_ts(&expires_at)?,
        },
    )))
}

impl LedgerStore for PgStore {
    async fn apply_trip(
        &self,
        event_id: &str,
        trip: &TripFare,
        plan: &CapturePlan,
        now: Timestamp,
    ) -> Result<Applied, Error> {
        let mut tx = self.pool.begin().await.db()?;
        let fresh = sqlx::query(
            "INSERT INTO inbox (event_id) VALUES ($1) ON CONFLICT (event_id) DO NOTHING",
        )
        .bind(event_id)
        .execute(&mut *tx)
        .await
        .db()?
        .rows_affected();
        if fresh == 0 {
            tx.rollback().await.db()?;
            return Ok(Applied::Duplicate);
        }

        // Hai bước, cùng transaction: capture rồi split. Số dư cộng dồn một lần ở cuối.
        let mut deltas = BTreeMap::new();
        let mut created = Vec::new();
        for entry in [&plan.capture, &plan.split] {
            if let Written::Created(posted) =
                write_entry(&mut tx, entry, &self.topic_out, &mut deltas).await?
            {
                created.push(posted);
            }
        }
        sqlx::query(
            "INSERT INTO payments (trip_id, customer_id, driver_id, captured_dong, fee_dong, status) \
             VALUES ($1, $2, $3, $4, $5, 'CAPTURED') ON CONFLICT (trip_id) DO NOTHING",
        )
        .bind(&trip.trip_id)
        .bind(trip.customer.as_str())
        .bind(trip.driver.as_str())
        .bind(trip.fare.dong())
        .bind(plan.fee.dong())
        .execute(&mut *tx)
        .await
        .db()?;

        // Chuyến có hold thì đóng hold ngay trong transaction này.
        if let Some((hold_id, hold)) = lock_trip_hold(&mut tx, &trip.trip_id).await? {
            let (status, captured) = match hold.settle(now, trip.fare) {
                Settlement::Captured { captured } => (HoldStatus::Captured, captured.dong()),
                Settlement::Expired => (HoldStatus::Expired, 0),
            };
            sqlx::query("UPDATE holds SET status = $2, captured_dong = $3 WHERE id = $1")
                .bind(hold_id)
                .bind(status.as_str())
                .bind(captured)
                .execute(&mut *tx)
                .await
                .db()?;
        }

        apply_balances(&mut tx, &deltas).await?;
        tx.commit().await.db()?;
        Ok(if created.is_empty() {
            Applied::Duplicate
        } else {
            Applied::Posted(created)
        })
    }

    async fn post_entry(&self, entry: &JournalEntry) -> Result<PostOutcome, Error> {
        let mut tx = self.pool.begin().await.db()?;
        let mut deltas = BTreeMap::new();
        let written = write_entry(&mut tx, entry, &self.topic_out, &mut deltas).await?;
        apply_balances(&mut tx, &deltas).await?;
        tx.commit().await.db()?;
        Ok(match written {
            Written::Created(posted) => PostOutcome::Created(posted),
            Written::Replayed { entry_id } => PostOutcome::Replayed { entry_id },
        })
    }

    async fn balance(&self, account: &AccountId) -> Result<Option<Money>, Error> {
        let dong: Option<i64> =
            sqlx::query_scalar("SELECT balance_dong FROM account_balances WHERE account_id = $1")
                .bind(account.as_str())
                .fetch_optional(&self.pool)
                .await
                .db()?;
        Ok(dong.map(Money::from_dong))
    }

    async fn funds(&self, account: &AccountId, now: Timestamp) -> Result<Option<Funds>, Error> {
        let row = sqlx::query(
            "SELECT b.balance_dong, \
                    COALESCE((SELECT sum(h.amount_dong) FROM holds h \
                              WHERE h.account_id = b.account_id AND h.status = 'ACTIVE' \
                                AND h.expires_at > $2::timestamptz), 0)::bigint AS held \
             FROM account_balances b WHERE b.account_id = $1",
        )
        .bind(account.as_str())
        .bind(now.to_string())
        .fetch_optional(&self.pool)
        .await
        .db()?;
        Ok(row.map(|r| Funds {
            balance: Money::from_dong(r.get("balance_dong")),
            held: Money::from_dong(r.get("held")),
        }))
    }

    async fn place_hold(&self, hold: &NewHold, now: Timestamp) -> Result<HoldOutcome, Error> {
        let mut tx = self.pool.begin().await.db()?;
        // Khóa hàng số dư trước: mọi hold của cùng ví xếp hàng ở đây, nên kiểm khả dụng không bị đua.
        let balance: Option<i64> = sqlx::query_scalar(
            "SELECT balance_dong FROM account_balances WHERE account_id = $1 FOR UPDATE",
        )
        .bind(hold.account.as_str())
        .fetch_optional(&mut *tx)
        .await
        .db()?;
        let balance = balance.ok_or_else(|| Error::UnknownAccount(hold.account.to_string()))?;

        let existing =
            sqlx::query("SELECT id, account_id, amount_dong FROM holds WHERE idempotency_key = $1")
                .bind(&hold.key)
                .fetch_optional(&mut *tx)
                .await
                .db()?;
        if let Some(row) = existing {
            let same = row.get::<String, _>("account_id") == hold.account.as_str()
                && row.get::<i64, _>("amount_dong") == hold.amount.dong();
            return if same {
                Ok(HoldOutcome::Replayed {
                    hold_id: row.get("id"),
                })
            } else {
                Err(HoldError::KeyConflict {
                    key: hold.key.clone(),
                }
                .into())
            };
        }

        let held: i64 = sqlx::query_scalar(
            "SELECT COALESCE(sum(amount_dong), 0)::bigint FROM holds \
             WHERE account_id = $1 AND status = 'ACTIVE' AND expires_at > $2::timestamptz",
        )
        .bind(hold.account.as_str())
        .bind(now.to_string())
        .fetch_one(&mut *tx)
        .await
        .db()?;
        ensure_available(
            Money::from_dong(balance),
            Money::from_dong(held),
            hold.amount,
        )?;

        let hold_id: i64 = sqlx::query_scalar(
            "INSERT INTO holds (account_id, amount_dong, status, idempotency_key, expires_at) \
             VALUES ($1, $2, 'ACTIVE', $3, $4::timestamptz) RETURNING id",
        )
        .bind(hold.account.as_str())
        .bind(hold.amount.dong())
        .bind(&hold.key)
        .bind(hold.expires_at.to_string())
        .fetch_one(&mut *tx)
        .await
        .db()?;
        tx.commit().await.db()?;
        Ok(HoldOutcome::Created { hold_id })
    }

    async fn refund(&self, trip_id: &str, n: u32, amount: Money) -> Result<PostOutcome, Error> {
        let mut tx = self.pool.begin().await.db()?;
        // Khóa dòng thanh toán: hai lần hoàn đồng thời của một chuyến xếp hàng ở đây.
        let row = sqlx::query(
            "SELECT customer_id, driver_id, captured_dong, fee_dong, refunded_dong \
             FROM payments WHERE trip_id = $1 FOR UPDATE",
        )
        .bind(trip_id)
        .fetch_optional(&mut *tx)
        .await
        .db()?
        .ok_or_else(|| Error::UnknownTrip(trip_id.to_string()))?;
        let payment = PaymentSnapshot {
            trip_id: trip_id.to_string(),
            customer: AccountId::new(&row.get::<String, _>("customer_id"))?,
            driver: AccountId::new(&row.get::<String, _>("driver_id"))?,
            captured: Money::from_dong(row.get("captured_dong")),
            fee: Money::from_dong(row.get("fee_dong")),
            refunded: Money::from_dong(row.get("refunded_dong")),
        };

        // Key đã có thì đây là replay: so số tiền, KHÔNG kiểm "còn hoàn được" (số đã hoàn đã gồm lần này).
        let key = refund_key(trip_id, n);
        let old: Option<i64> = sqlx::query_scalar(
            "SELECT COALESCE(sum(p.amount_dong) FILTER (WHERE p.amount_dong > 0), 0)::bigint \
             FROM journal_entries e JOIN postings p ON p.entry_id = e.id \
             WHERE e.idempotency_key = $1 GROUP BY e.id",
        )
        .bind(&key)
        .fetch_optional(&mut *tx)
        .await
        .db()?;
        if let Some(old_amount) = old {
            return if old_amount == amount.dong() {
                let id: i64 =
                    sqlx::query_scalar("SELECT id FROM journal_entries WHERE idempotency_key = $1")
                        .bind(&key)
                        .fetch_one(&mut *tx)
                        .await
                        .db()?;
                Ok(PostOutcome::Replayed {
                    entry_id: display_id(id),
                })
            } else {
                Err(RefundError::KeyConflict { key }.into())
            };
        }

        let entry = refund_entry(&payment, n, amount)?;
        let mut deltas = BTreeMap::new();
        let Written::Created(posted) =
            write_entry(&mut tx, &entry, &self.topic_out, &mut deltas).await?
        else {
            return Err(Error::Storage("key hoàn xuất hiện giữa chừng".to_string()));
        };
        let refunded = payment.refunded.dong() + amount.dong();
        let status = if refunded == payment.captured.dong() {
            "REFUNDED"
        } else {
            "CAPTURED"
        };
        sqlx::query("UPDATE payments SET refunded_dong = $2, status = $3 WHERE trip_id = $1")
            .bind(trip_id)
            .bind(refunded)
            .bind(status)
            .execute(&mut *tx)
            .await
            .db()?;
        apply_balances(&mut tx, &deltas).await?;
        tx.commit().await.db()?;
        Ok(PostOutcome::Created(posted))
    }

    async fn ping(&self) -> Result<(), Error> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .db()
            .map(|_| ())
    }
}
