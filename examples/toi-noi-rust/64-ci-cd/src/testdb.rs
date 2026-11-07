//! Fixture PostgreSQL: mỗi ca có một schema `lab_<uuid>` riêng và một pool chỉ nhìn thấy schema đó.
//! Chạy giống hệt nhau ở máy dev (database dùng chung) và trong CI (service container trống).

use std::future::Future;

use anyhow::{Context, Result, bail};
use sqlx::postgres::PgPoolOptions;
use sqlx::{AssertSqlSafe, Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use crate::store::{MIGRATOR, PgLedger};

/// Biến môi trường chứa URL PostgreSQL. Thiếu là lỗi, không phải lý do để bỏ qua test.
pub const ENV_PG_URL: &str = "TOI_NOI_PG_URL";

/// Đọc URL từ giá trị biến môi trường; thiếu hoặc rỗng thì báo lỗi rõ.
pub fn pg_url_from(value: Option<String>) -> Result<String> {
    match value {
        Some(v) if !v.trim().is_empty() => Ok(v),
        _ => bail!("thiếu {ENV_PG_URL}: đặt URL PostgreSQL trước khi chạy (test không tự bỏ qua)"),
    }
}

/// Một ca = một schema + một pool.
pub struct TestDb {
    /// Pool đã đặt `search_path` về schema của ca.
    pub pool: PgPool,
    /// Tên schema: `lab_` + 32 ký tự hex.
    pub schema: String,
    url: String,
}

impl TestDb {
    /// Dựng schema, pool và chạy migration.
    pub async fn try_new() -> Result<TestDb> {
        let url = pg_url_from(std::env::var(ENV_PG_URL).ok())?;
        let schema = format!("lab_{}", Uuid::new_v4().simple());

        let mut admin = PgConnection::connect(&url)
            .await
            .with_context(|| format!("không kết nối được PostgreSQL ở {ENV_PG_URL}"))?;
        admin
            .execute(AssertSqlSafe(format!("create schema {schema}")))
            .await?;
        admin.close().await?;

        // Mọi kết nối mới của pool tự đặt search_path trước khi dùng.
        let sp = schema.clone();
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .after_connect(move |conn, _meta| {
                let sql = format!("set search_path to {sp}");
                Box::pin(async move {
                    conn.execute(AssertSqlSafe(sql)).await?;
                    Ok(())
                })
            })
            .connect(&url)
            .await?;

        let db = TestDb { pool, schema, url };
        if let Err(e) = MIGRATOR.run(&db.pool).await {
            db.close().await; // dựng hỏng thì dọn ngay
            return Err(e).context("chạy migration trong schema riêng thất bại");
        }
        Ok(db)
    }

    /// Adapter sổ cái trên pool của ca.
    pub fn ledger(&self) -> PgLedger {
        PgLedger::new(self.pool.clone())
    }

    /// Schema còn tồn tại không (kết nối quản trị riêng).
    pub async fn schema_exists(url: &str, schema: &str) -> Result<bool> {
        let mut admin = PgConnection::connect(url).await?;
        let found: bool =
            sqlx::query_scalar("select exists (select 1 from pg_namespace where nspname = $1)")
                .bind(schema)
                .fetch_one(&mut admin)
                .await?;
        admin.close().await?;
        Ok(found)
    }

    /// URL của database (để kiểm sau khi dọn).
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Đóng pool rồi `DROP SCHEMA ... CASCADE`. `Drop` không async được nên phải gọi tường minh.
    pub async fn close(self) {
        self.pool.close().await;
        match PgConnection::connect(&self.url).await {
            Ok(mut admin) => {
                let sql = format!("drop schema if exists {} cascade", self.schema);
                if let Err(e) = admin.execute(AssertSqlSafe(sql)).await {
                    eprintln!("không DROP được schema {}: {e}", self.schema);
                }
                let _ = admin.close().await;
            }
            Err(e) => eprintln!("không dọn được schema {}: {e}", self.schema),
        }
    }
}

/// Chạy `body` trên một schema riêng và LUÔN dọn, kể cả khi `body` panic (assert hỏng).
/// Panic được bắt qua `JoinHandle`, schema bị xóa, rồi panic được ném lại cho test đỏ.
pub async fn with_db<F, Fut>(body: F)
where
    F: FnOnce(PgLedger) -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    let db = match TestDb::try_new().await {
        Ok(db) => db,
        Err(e) => panic!("không dựng được TestDb: {e:#}"),
    };
    let outcome = tokio::spawn(body(db.ledger())).await;
    db.close().await;
    if let Err(e) = outcome {
        std::panic::resume_unwind(e.into_panic());
    }
}
