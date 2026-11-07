//! Fixture PostgreSQL: mỗi ca một schema `lab_<uuid>`, dọn bằng `close()` kể cả khi ca lỗi.

use anyhow::{Context, Result, bail};
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use sqlx::{AssertSqlSafe, Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use crate::store::PgLedger;

/// Tên biến môi trường chứa URL PostgreSQL của lab.
pub const ENV_PG_URL: &str = "TOI_NOI_PG_URL";
/// Dấu ghi trong comment của schema để biết ai tạo ra nó.
pub const MARKER: &str = "toi-noi-lab-45";

/// Migration nhúng lúc biên dịch; chạy trong schema riêng của từng ca.
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Thiếu biến hoặc rỗng là lỗi, không phải lý do để bỏ qua ca test.
pub fn pg_url_from(value: Option<String>) -> Result<String> {
    match value {
        Some(v) if !v.trim().is_empty() => Ok(v),
        _ => bail!(
            "thiếu {ENV_PG_URL}. Bật PostgreSQL lab: \
             docker compose -f examples/toi-noi-rust/infra/docker-compose.yml up -d --wait, \
             rồi export {ENV_PG_URL}='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'"
        ),
    }
}

/// Một ca = một schema riêng + một pool chỉ nhìn thấy schema đó.
pub struct TestDb {
    /// Pool đã đặt `search_path` về schema của ca.
    pub pool: PgPool,
    /// Tên schema, dạng `lab_` + 32 ký tự hex.
    pub schema: String,
    url: String,
}

impl TestDb {
    /// Dựng schema, pool (`pool_size` kết nối) và chạy migration.
    /// Cuộc đua dùng `Barrier` cần pool không nhỏ hơn số tác vụ cùng chờ.
    pub async fn try_new(pool_size: u32) -> Result<TestDb> {
        let url = pg_url_from(std::env::var(ENV_PG_URL).ok())?;
        let schema = format!("lab_{}", Uuid::new_v4().simple());

        let mut admin = PgConnection::connect(&url).await.with_context(|| {
            format!("không kết nối được PostgreSQL ở {ENV_PG_URL}; compose đã bật chưa?")
        })?;
        admin
            .execute(AssertSqlSafe(format!("create schema {schema}")))
            .await?;
        admin
            .execute(AssertSqlSafe(format!(
                "comment on schema {schema} is '{MARKER}'"
            )))
            .await?;
        admin.close().await?;

        // Mọi kết nối mới của pool tự đặt search_path trước khi dùng.
        let sp = schema.clone();
        let pool = PgPoolOptions::new()
            .max_connections(pool_size)
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
            db.close().await; // setup hỏng thì dọn ngay
            return Err(e).context("chạy migration trong schema riêng thất bại");
        }
        Ok(db)
    }

    /// Sổ cái trên pool của ca.
    pub fn ledger(&self) -> PgLedger {
        PgLedger::new(self.pool.clone())
    }

    /// Đóng pool rồi `DROP SCHEMA ... CASCADE`. `Drop` không async được nên phải gọi tay.
    pub async fn close(self) {
        self.pool.close().await; // hết kết nối còn giữ schema
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

    /// Có schema này trong database không (dùng để kiểm đã dọn).
    pub async fn schema_exists(url_source: &str, schema: &str) -> Result<bool> {
        let mut conn = PgConnection::connect(url_source).await?;
        let found: bool =
            sqlx::query_scalar("select exists (select 1 from pg_namespace where nspname = $1)")
                .bind(schema)
                .fetch_one(&mut conn)
                .await?;
        conn.close().await?;
        Ok(found)
    }
}

/// Chạy `body` trên một `TestDb` mới và luôn dọn, dù `body` trả `Ok` hay `Err`.
/// Test trả `Result` thay vì panic nên đường lỗi cũng đi qua `close()`.
pub async fn with_db<T>(pool_size: u32, body: impl AsyncFnOnce(&TestDb) -> Result<T>) -> Result<T> {
    let db = TestDb::try_new(pool_size).await?;
    let out = body(&db).await;
    db.close().await;
    out
}

/// URL của database lab (để kiểm schema đã biến mất).
pub fn lab_url() -> Result<String> {
    pg_url_from(std::env::var(ENV_PG_URL).ok())
}
