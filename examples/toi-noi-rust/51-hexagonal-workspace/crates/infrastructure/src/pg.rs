//! Adapter sqlx/PostgreSQL cho `LedgerStore`, và `LabDb`: schema `lab_<uuid>` riêng cho mỗi lần chạy.

use std::collections::BTreeMap;

use anyhow::Context;
use application::{LedgerStore, PostOutcome, StoreError};
use domain::{AccountId, EntryId, JournalEntry, LedgerError, Money};
use sqlx::postgres::PgPoolOptions;
use sqlx::{AssertSqlSafe, Connection, PgConnection, PgPool};
use uuid::Uuid;

use crate::CHART;

// Mọi lỗi sqlx là lỗi hạ tầng: gói vào `StoreError::Storage`, application không biết sqlx.
fn storage(e: sqlx::Error) -> StoreError {
    StoreError::Storage(Box::new(e))
}

fn entry_id(id: i64) -> Result<EntryId, StoreError> {
    EntryId::parse(&format!("JE-{id:06}"))
        .ok_or_else(|| StoreError::Storage(format!("id bút toán lạ: {id}").into()))
}

/// Kho PostgreSQL. Giữ một `PgPool` (rẻ để clone, dùng được từ nhiều task).
#[derive(Debug, Clone)]
pub struct PgStore {
    pool: PgPool,
}

impl PgStore {
    /// Bao một pool đã trỏ vào schema có đủ bảng.
    pub fn new(pool: PgPool) -> PgStore {
        PgStore { pool }
    }
}

impl LedgerStore for PgStore {
    async fn post(&self, entry: &JournalEntry) -> Result<PostOutcome, StoreError> {
        let mut tx = self.pool.begin().await.map_err(storage)?;

        // Tài khoản phải có trong sổ: kiểm trước để trả lỗi nghiệp vụ rõ, không để FK làm hỏng giao dịch.
        let wanted: Vec<String> = entry
            .postings()
            .iter()
            .map(|p| p.account.as_str().to_string())
            .collect();
        let known: Vec<String> = sqlx::query_scalar("SELECT id FROM accounts WHERE id = ANY($1)")
            .bind(&wanted)
            .fetch_all(&mut *tx)
            .await
            .map_err(storage)?;
        if let Some(missing) = wanted.iter().find(|w| !known.contains(w)) {
            return Err(StoreError::UnknownAccount(missing.clone()));
        }

        // Trùng key thì INSERT không làm gì và không trả dòng nào.
        let inserted: Option<i64> = sqlx::query_scalar(
            "INSERT INTO journal_entries (idempotency_key) VALUES ($1) \
             ON CONFLICT (idempotency_key) DO NOTHING RETURNING id",
        )
        .bind(entry.idempotency_key())
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage)?;

        let Some(id) = inserted else {
            return replay(&mut tx, entry).await;
        };

        let mut per_account: BTreeMap<&AccountId, Money> = BTreeMap::new();
        for (index, p) in entry.postings().iter().enumerate() {
            let line_no = i32::try_from(index + 1).map_err(|_| LedgerError::Overflow)?;
            sqlx::query(
                "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(id)
            .bind(line_no)
            .bind(p.account.as_str())
            .bind(p.amount.dong())
            .execute(&mut *tx)
            .await
            .map_err(storage)?;
            let slot = per_account.entry(&p.account).or_insert(Money::ZERO);
            *slot = slot.checked_add(p.amount).ok_or(LedgerError::Overflow)?;
        }
        // Cập nhật số dư theo thứ tự tài khoản (BTreeMap) để các giao dịch khoá hàng cùng thứ tự.
        for (account, delta) in per_account {
            sqlx::query(
                "INSERT INTO account_balances (account_id, balance_dong, version) \
                 VALUES ($1, $2, 1) \
                 ON CONFLICT (account_id) DO UPDATE SET \
                   balance_dong = account_balances.balance_dong + EXCLUDED.balance_dong, \
                   version = account_balances.version + 1, updated_at = now()",
            )
            .bind(account.as_str())
            .bind(delta.dong())
            .execute(&mut *tx)
            .await
            .map_err(storage)?;
        }
        tx.commit().await.map_err(storage)?;
        Ok(PostOutcome {
            id: entry_id(id)?,
            replayed: false,
        })
    }

    async fn balance(&self, account: &AccountId) -> Result<Money, StoreError> {
        let dong: Option<i64> =
            sqlx::query_scalar("SELECT balance_dong FROM account_balances WHERE account_id = $1")
                .bind(account.as_str())
                .fetch_optional(&self.pool)
                .await
                .map_err(storage)?;
        Ok(Money::from_dong(dong.unwrap_or(0)))
    }
}

// Key đã có: so nội dung với bút toán cũ, giống nhau thì replay, khác thì xung đột.
async fn replay(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    entry: &JournalEntry,
) -> Result<PostOutcome, StoreError> {
    let id: i64 = sqlx::query_scalar("SELECT id FROM journal_entries WHERE idempotency_key = $1")
        .bind(entry.idempotency_key())
        .fetch_one(&mut **tx)
        .await
        .map_err(storage)?;
    let stored: Vec<(String, i64)> = sqlx::query_as(
        "SELECT account_id, amount_dong FROM postings WHERE entry_id = $1 ORDER BY line_no",
    )
    .bind(id)
    .fetch_all(&mut **tx)
    .await
    .map_err(storage)?;
    let same = stored.len() == entry.postings().len()
        && stored
            .iter()
            .zip(entry.postings())
            .all(|((acc, dong), p)| acc == p.account.as_str() && *dong == p.amount.dong());
    if !same {
        return Err(StoreError::IdempotencyConflict(
            entry.idempotency_key().to_string(),
        ));
    }
    Ok(PostOutcome {
        id: entry_id(id)?,
        replayed: true,
    })
}

/// Một schema lab đang sống cùng pool trỏ vào nó. Dọn bằng [`LabDb::cleanup`].
#[derive(Debug)]
pub struct LabDb {
    url: String,
    schema: String,
    /// Pool có `search_path` = schema lab; mọi bảng nằm trong schema này, không đụng `public`.
    pub pool: PgPool,
}

impl LabDb {
    /// Đọc `TOI_NOI_PG_URL`, tạo schema `lab_<uuid>`, bảng, trigger cân bút toán và tài khoản của [`CHART`].
    ///
    /// # Errors
    /// Thiếu biến môi trường, không kết nối được, hoặc DDL lỗi (schema đã tạo được dọn lại).
    pub async fn setup() -> anyhow::Result<LabDb> {
        let url = std::env::var("TOI_NOI_PG_URL")
            .context("thiếu biến TOI_NOI_PG_URL (xem examples/toi-noi-rust/infra/README.md)")?;
        let schema = format!("lab_{}", Uuid::new_v4().simple());
        let mut admin = PgConnection::connect(&url)
            .await
            .context("không kết nối được PostgreSQL ở TOI_NOI_PG_URL; compose đã chạy chưa?")?;
        // Tên schema do ta sinh (chữ hex), không có dữ liệu ngoài: an toàn để ghép chuỗi.
        sqlx::query(AssertSqlSafe(format!("CREATE SCHEMA {schema}")))
            .execute(&mut admin)
            .await
            .context("CREATE SCHEMA")?;

        let search_path = format!("SET search_path TO {schema}");
        let opened = PgPoolOptions::new()
            .max_connections(5)
            .after_connect(move |conn, _meta| {
                let sql = search_path.clone();
                Box::pin(async move {
                    sqlx::query(AssertSqlSafe(sql)).execute(conn).await?;
                    Ok(())
                })
            })
            .connect(&url)
            .await;
        let pool = match opened {
            Ok(pool) => pool,
            Err(e) => {
                // Không mở được pool: xóa schema vừa tạo rồi mới báo lỗi, không để sót.
                let _ = sqlx::query(AssertSqlSafe(format!("DROP SCHEMA {schema} CASCADE")))
                    .execute(&mut admin)
                    .await;
                return Err(e).context("mở pool");
            }
        };

        let lab = LabDb { url, schema, pool };
        if let Err(e) = lab.create_tables().await {
            let _ = lab.cleanup().await;
            return Err(e);
        }
        Ok(lab)
    }

    async fn create_tables(&self) -> anyhow::Result<()> {
        sqlx::raw_sql(include_str!("../sql/01_schema.sql"))
            .execute(&self.pool)
            .await
            .context("tạo bảng")?;
        // Trigger cân bút toán chuẩn của bài 25, dùng nguyên bản.
        sqlx::raw_sql(include_str!("../sql/02_balance_check.sql"))
            .execute(&self.pool)
            .await
            .context("tạo trigger cân bút toán")?;
        for (id, kind) in CHART {
            sqlx::query("INSERT INTO accounts (id, kind) VALUES ($1, $2)")
                .bind(id)
                .bind(kind)
                .execute(&self.pool)
                .await
                .context("mở tài khoản")?;
        }
        Ok(())
    }

    /// Tên schema lab.
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Đóng pool rồi `DROP SCHEMA ... CASCADE`. Trả lại tên schema đã xoá.
    ///
    /// # Errors
    /// Không kết nối được hoặc DROP lỗi.
    pub async fn cleanup(self) -> anyhow::Result<String> {
        self.pool.close().await;
        let mut admin = PgConnection::connect(&self.url).await?;
        sqlx::query(AssertSqlSafe(format!(
            "DROP SCHEMA IF EXISTS {} CASCADE",
            self.schema
        )))
        .execute(&mut admin)
        .await?;
        Ok(self.schema)
    }

    /// Schema `name` còn tồn tại không (kiểm sau cleanup).
    ///
    /// # Errors
    /// Không kết nối được.
    pub async fn schema_exists(url: &str, name: &str) -> anyhow::Result<bool> {
        let mut admin = PgConnection::connect(url).await?;
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_namespace WHERE nspname = $1")
            .bind(name)
            .fetch_one(&mut admin)
            .await?;
        Ok(n > 0)
    }
}
