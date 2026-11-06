//! Adapter PostgreSQL của port `PaymentStore`. Một `commit` = MỘT transaction ghi aggregate (có
//! khóa lạc quan theo `version`), bút toán, số dư và outbox. Rút gọn từ bài 24, 25, 37 và 40.

use std::collections::BTreeMap;
use std::str::FromStr;

use application::{Committed, Loaded, PaymentStore, PostedEntry, StoreError, Work};
use domain::chart::kind_of;
use domain::reconcile::Line;
use domain::{LedgerEntryPosted, PlannedEntry, PostingDto, SCHEMA_VERSION, TripPayment};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::types::Json;
use sqlx::{PgConnection, Row};

use crate::{Fault, die};

const SCHEMA_SQL: &str = include_str!("../sql/01_schema.sql");
const BALANCE_CHECK_SQL: &str = include_str!("../sql/02_balance_check.sql");

/// Mã hiển thị `JE-000001` của id trong DB.
pub fn display_id(id: i64) -> String {
    format!("JE-{id:06}")
}

fn db(err: sqlx::Error) -> StoreError {
    StoreError::infra(err)
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .is_some_and(|d| d.is_unique_violation())
}

/// Một bút toán trong ảnh chụp sổ, bỏ mã số và thời điểm để so sánh hai lần chạy được.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryDump {
    /// `idempotency_key`.
    pub key: String,
    /// `(tài khoản, số tiền)` theo `line_no`.
    pub postings: Vec<(String, i64)>,
}

/// Ảnh chụp toàn sổ: bút toán theo thứ tự ghi và số dư từng tài khoản.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerDump {
    /// Bút toán theo thứ tự ghi.
    pub entries: Vec<EntryDump>,
    /// Số dư từng tài khoản.
    pub balances: BTreeMap<String, i64>,
}

/// Các con số bất biến của sổ, đọc bằng SQL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invariants {
    /// Số bút toán.
    pub entries: i64,
    /// Số bút toán không cân (tổng khác 0 hoặc dưới 2 dòng).
    pub unbalanced: i64,
    /// Tổng mọi posting của toàn sổ.
    pub postings_total: i64,
    /// Số tài khoản mà `account_balances` khác tổng posting tính lại.
    pub projection_drift: i64,
    /// Số dòng outbox.
    pub outbox_total: i64,
    /// Số dòng outbox chưa phát.
    pub outbox_unpublished: i64,
}

/// Cổng vào PostgreSQL của service.
#[derive(Clone)]
pub struct PgStore {
    pool: PgPool,
    topic_out: String,
    fault: Fault,
}

impl PgStore {
    /// Mở pool với `search_path` trỏ vào `schema`, rồi dựng bảng nếu schema chưa có.
    ///
    /// # Errors
    /// Không kết nối được hoặc DDL lỗi.
    pub async fn connect(
        pg_url: &str,
        schema: Option<&str>,
        topic_out: &str,
        fault: Fault,
    ) -> Result<PgStore, sqlx::Error> {
        let mut options = PgConnectOptions::from_str(pg_url)?;
        if let Some(schema) = schema {
            options = options.options([("search_path", schema)]);
        }
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await?;
        let store = PgStore {
            pool,
            topic_out: topic_out.to_string(),
            fault,
        };
        store.migrate().await?;
        Ok(store)
    }

    // Chỉ dựng bảng khi chưa có `accounts` trong schema hiện tại. Production dùng migration thật (bài 23).
    async fn migrate(&self) -> Result<(), sqlx::Error> {
        let exists: Option<String> = sqlx::query_scalar("SELECT to_regclass('accounts')::text")
            .fetch_one(&self.pool)
            .await?;
        if exists.is_none() {
            sqlx::raw_sql(SCHEMA_SQL).execute(&self.pool).await?;
            sqlx::raw_sql(BALANCE_CHECK_SQL).execute(&self.pool).await?;
        }
        Ok(())
    }

    /// Pool bên dưới (relay dùng).
    pub fn pool(&self) -> &PgPool {
        &self.pool
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

    /// Ghi một bút toán không thuộc aggregate nào (cấp ngân sách campaign, bài 45), idempotent theo key.
    ///
    /// # Errors
    /// Lỗi của kho, kể cả [`StoreError::IdempotencyConflict`] khi key trùng mà nội dung khác.
    pub async fn post_standalone(
        &self,
        planned: &PlannedEntry,
    ) -> Result<Option<PostedEntry>, StoreError> {
        let mut tx = self.pool.begin().await.map_err(db)?;
        let posted = post_in_tx(&mut tx, planned, &self.topic_out).await?;
        tx.commit().await.map_err(db)?;
        Ok(posted)
    }

    /// Ảnh chụp toàn sổ, để so hai lần chạy.
    ///
    /// # Errors
    /// Lỗi PostgreSQL.
    pub async fn dump(&self) -> Result<LedgerDump, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT e.idempotency_key, p.account_id, p.amount_dong \
             FROM journal_entries e JOIN postings p ON p.entry_id = e.id \
             ORDER BY e.id, p.line_no",
        )
        .fetch_all(&self.pool)
        .await?;
        let mut entries: Vec<EntryDump> = Vec::new();
        for row in rows {
            let key: String = row.get("idempotency_key");
            let posting = (
                row.get::<String, _>("account_id"),
                row.get::<i64, _>("amount_dong"),
            );
            match entries.last_mut() {
                Some(last) if last.key == key => last.postings.push(posting),
                _ => entries.push(EntryDump {
                    key,
                    postings: vec![posting],
                }),
            }
        }
        let balances = sqlx::query("SELECT account_id, balance_dong FROM account_balances")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| (r.get("account_id"), r.get("balance_dong")))
            .collect();
        Ok(LedgerDump { entries, balances })
    }

    /// Mã `JE-…` theo key, theo thứ tự ghi.
    ///
    /// # Errors
    /// Lỗi PostgreSQL.
    pub async fn entry_ids(&self) -> Result<Vec<(String, String)>, sqlx::Error> {
        let rows = sqlx::query("SELECT id, idempotency_key FROM journal_entries ORDER BY id")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows
            .into_iter()
            .map(|r| (r.get("idempotency_key"), display_id(r.get("id"))))
            .collect())
    }

    /// Các con số bất biến của sổ.
    ///
    /// # Errors
    /// Lỗi PostgreSQL.
    pub async fn invariants(&self) -> Result<Invariants, sqlx::Error> {
        let one = |sql: &'static str| async move {
            sqlx::query_scalar::<_, i64>(sql)
                .fetch_one(&self.pool)
                .await
        };
        Ok(Invariants {
            entries: one("SELECT count(*) FROM journal_entries").await?,
            unbalanced: one(
                "SELECT count(*) FROM (SELECT entry_id FROM postings GROUP BY entry_id \
                 HAVING sum(amount_dong) <> 0 OR count(*) < 2) t",
            )
            .await?,
            postings_total: one("SELECT COALESCE(sum(amount_dong), 0)::bigint FROM postings").await?,
            projection_drift: one(
                "SELECT count(*) FROM account_balances b \
                 WHERE b.balance_dong <> (SELECT COALESCE(sum(amount_dong), 0) FROM postings p WHERE p.account_id = b.account_id)",
            )
            .await?,
            outbox_total: one("SELECT count(*) FROM outbox").await?,
            outbox_unpublished: one("SELECT count(*) FROM outbox WHERE published_at IS NULL").await?,
        })
    }

    /// Dòng clearing của các bút toán capture, gom theo mã giao dịch VíXanh (bài 47: so theo dòng, không so số dư).
    /// Bút toán chia cũng có dòng clearing nhưng không có trong `psp_refs`, vì sao kê không có nó.
    ///
    /// # Errors
    /// Lỗi PostgreSQL.
    pub async fn clearing_lines(&self) -> Result<Vec<Line>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT r.external_ref, SUM(p.amount_dong)::bigint AS amount_dong \
             FROM psp_refs r JOIN postings p ON p.entry_id = r.entry_id AND p.account_id = 'psp:vixanh:clearing' \
             GROUP BY r.external_ref ORDER BY r.external_ref",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| Line {
                external_ref: r.get("external_ref"),
                amount_dong: r.get("amount_dong"),
            })
            .collect())
    }

    /// Đếm theo một câu SQL tĩnh (test dùng để kiểm bảng).
    ///
    /// # Errors
    /// Lỗi PostgreSQL.
    pub async fn count(&self, sql: &'static str) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(sql).fetch_one(&self.pool).await
    }
}

impl PaymentStore for PgStore {
    async fn load(&self, trip_id: &str) -> Result<Option<Loaded>, StoreError> {
        let row = sqlx::query("SELECT version, state FROM trip_payments WHERE trip_id = $1")
            .bind(trip_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(db)?;
        Ok(row.map(|row| {
            let Json(payment): Json<TripPayment> = row.get("state");
            Loaded {
                payment,
                version: row.get("version"),
            }
        }))
    }

    async fn commit(&self, work: Work) -> Result<Committed, StoreError> {
        let mut tx = self.pool.begin().await.map_err(db)?;
        // Aggregate trước: thua tranh chấp thì dừng ngay, chưa ghi bút toán nào.
        let version = save_payment(&mut tx, &work).await?;
        let mut posted = Vec::new();
        for planned in &work.entries {
            if let Some(entry) = post_in_tx(&mut tx, planned, &self.topic_out).await? {
                posted.push(entry);
            }
        }
        if self.fault == Fault::AbortBeforeCommit && !posted.is_empty() {
            die("trước COMMIT: transaction chưa xong, DB sẽ không có gì");
        }
        tx.commit().await.map_err(db)?;
        Ok(Committed { version, posted })
    }
}

// Khóa lạc quan. `expected_version == 0`: chưa có dòng nào, INSERT; hai bên cùng INSERT thì bên sau
// nhận unique_violation sau khi bên trước commit. Ngược lại UPDATE có điều kiện `version = $3`; sau
// khi chờ bên kia commit, PostgreSQL kiểm lại điều kiện trên dòng mới nên không dòng nào khớp.
async fn save_payment(conn: &mut PgConnection, work: &Work) -> Result<i64, StoreError> {
    if work.expected_version == 0 {
        let inserted =
            sqlx::query("INSERT INTO trip_payments (trip_id, version, state) VALUES ($1, 1, $2)")
                .bind(&work.payment.trip_id)
                .bind(Json(&work.payment))
                .execute(&mut *conn)
                .await;
        return match inserted {
            Ok(_) => Ok(1),
            Err(err) if is_unique_violation(&err) => Err(StoreError::VersionConflict),
            Err(err) => Err(db(err)),
        };
    }
    let updated = sqlx::query(
        "UPDATE trip_payments SET version = version + 1, state = $2, updated_at = now() \
         WHERE trip_id = $1 AND version = $3",
    )
    .bind(&work.payment.trip_id)
    .bind(Json(&work.payment))
    .bind(work.expected_version)
    .execute(&mut *conn)
    .await
    .map_err(db)?
    .rows_affected();
    if updated == 0 {
        return Err(StoreError::VersionConflict);
    }
    Ok(work.expected_version + 1)
}

/// Ghi một bút toán, số dư, `psp_refs` và outbox trong transaction của người gọi.
/// `Ok(None)`: key đã có với nội dung y hệt, không ghi gì thêm.
async fn post_in_tx(
    conn: &mut PgConnection,
    planned: &PlannedEntry,
    topic_out: &str,
) -> Result<Option<PostedEntry>, StoreError> {
    let entry = &planned.entry;
    // Kiểm ngân sách TRƯỚC khi ghi, và khóa dòng số dư để hai yêu cầu cùng campaign xếp hàng (bài 45).
    if let Some(funds) = &planned.requires_funds {
        let available: i64 = sqlx::query_scalar(
            "SELECT balance_dong FROM account_balances WHERE account_id = $1 FOR UPDATE",
        )
        .bind(&funds.account)
        .fetch_optional(&mut *conn)
        .await
        .map_err(db)?
        .unwrap_or(0);
        if available < funds.at_least {
            return Err(StoreError::BudgetExhausted {
                account: funds.account.clone(),
                requested: funds.at_least,
                available,
            });
        }
    }
    let inserted = sqlx::query(
        "INSERT INTO journal_entries (idempotency_key) VALUES ($1) \
         ON CONFLICT (idempotency_key) DO NOTHING \
         RETURNING id, to_char(posted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS posted_at",
    )
    .bind(entry.idempotency_key())
    .fetch_optional(&mut *conn)
    .await
    .map_err(db)?;
    let Some(row) = inserted else {
        return check_replay(conn, planned).await;
    };
    let id: i64 = row.get("id");
    let posted_at: String = row.get("posted_at");

    // Cộng dồn theo tài khoản và đi theo thứ tự mã trong TỪNG bút toán. Một commit nhiều bút toán
    // thì không có thứ tự chung cho cả transaction, nên `40P01` vẫn có thể xảy ra về lý thuyết;
    // nó rơi vào lỗi hạ tầng và consumer xử lý lại message.
    let mut deltas: BTreeMap<&str, i64> = BTreeMap::new();
    for posting in entry.postings() {
        let slot = deltas.entry(posting.account.as_str()).or_insert(0);
        *slot += posting.amount.dong();
    }
    for account in deltas.keys() {
        let kind = kind_of(account).ok_or_else(|| {
            StoreError::infra(std::io::Error::other(format!(
                "tài khoản ngoài sơ đồ: {account}"
            )))
        })?;
        sqlx::query("INSERT INTO accounts (id, kind) VALUES ($1, $2) ON CONFLICT (id) DO NOTHING")
            .bind(account)
            .bind(kind)
            .execute(&mut *conn)
            .await
            .map_err(db)?;
    }
    for (index, posting) in entry.postings().iter().enumerate() {
        let line_no = i32::try_from(index + 1).unwrap_or(i32::MAX);
        sqlx::query("INSERT INTO postings (entry_id, line_no, account_id, amount_dong) VALUES ($1, $2, $3, $4)")
            .bind(id)
            .bind(line_no)
            .bind(posting.account.as_str())
            .bind(posting.amount.dong())
            .execute(&mut *conn)
            .await
            .map_err(db)?;
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
        .await
        .map_err(db)?;
    }
    if let Some(external_ref) = &planned.external_ref {
        sqlx::query("INSERT INTO psp_refs (entry_id, external_ref) VALUES ($1, $2)")
            .bind(id)
            .bind(external_ref)
            .execute(&mut *conn)
            .await
            .map_err(db)?;
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
    // Cùng transaction: bút toán và việc «cần phát» cùng có hoặc cùng không.
    sqlx::query("INSERT INTO outbox (topic, key, payload) VALUES ($1, $2, $3)")
        .bind(topic_out)
        .bind(&event.idempotency_key)
        .bind(Json(&event))
        .execute(&mut *conn)
        .await
        .map_err(db)?;
    Ok(Some(PostedEntry {
        entry_id: event.entry_id,
        key: event.idempotency_key,
    }))
}

// Key đã có: so nội dung với bản cũ. Trùng là replay (không làm gì), khác là xung đột.
async fn check_replay(
    conn: &mut PgConnection,
    planned: &PlannedEntry,
) -> Result<Option<PostedEntry>, StoreError> {
    let entry = &planned.entry;
    let rows = sqlx::query(
        "SELECT p.account_id, p.amount_dong FROM journal_entries e \
         JOIN postings p ON p.entry_id = e.id WHERE e.idempotency_key = $1 ORDER BY p.line_no",
    )
    .bind(entry.idempotency_key())
    .fetch_all(&mut *conn)
    .await
    .map_err(db)?;
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
        Ok(None)
    } else {
        Err(StoreError::IdempotencyConflict {
            key: entry.idempotency_key().to_string(),
        })
    }
}
