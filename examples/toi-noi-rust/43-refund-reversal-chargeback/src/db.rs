//! Kết nối và schema riêng `lab_<uuid>` cho mỗi ca chạy.

use std::future::Future;
use std::str::FromStr;

use anyhow::{Context, Result, bail};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{AssertSqlSafe, Connection, PgConnection, PgPool};
use uuid::Uuid;

const SCHEMA_SQL: &str = include_str!("../sql/01_schema.sql");
const BALANCED_SQL: &str = include_str!("../sql/02_balanced.sql");
const APPEND_ONLY_SQL: &str = include_str!("../sql/03_append_only.sql");

/// Một schema `lab_<uuid>` cùng pool trỏ vào nó.
#[derive(Clone)]
pub struct LabDb {
    /// Pool đã đặt `search_path`.
    pub pool: PgPool,
    schema: String,
    url: String,
}

fn pg_url() -> Result<String> {
    std::env::var("TOI_NOI_PG_URL").map_err(|_| {
        anyhow::anyhow!(
            "thiếu biến TOI_NOI_PG_URL. Bật PostgreSQL lab: \
             docker compose -f examples/toi-noi-rust/infra/docker-compose.yml up -d --wait, \
             rồi export TOI_NOI_PG_URL='postgres://toi_noi:toi_noi_lab@127.0.0.1:5433/toi_noi_rust_lab'"
        )
    })
}

impl LabDb {
    /// Tạo schema mới, dựng bảng và trigger, trả pool trỏ vào schema đó.
    pub async fn create() -> Result<LabDb> {
        let url = pg_url()?;
        // Tên chỉ có `lab_` và 32 ký tự hex nên không chứa gì để tiêm SQL.
        let schema = format!("lab_{}", Uuid::new_v4().simple());
        let mut conn = PgConnection::connect(&url)
            .await
            .context("không kết nối được PostgreSQL lab; kiểm tra docker compose đã chạy chưa")?;
        sqlx::raw_sql(AssertSqlSafe(format!("CREATE SCHEMA {schema}")))
            .execute(&mut conn)
            .await?;
        conn.close().await?;

        let options = PgConnectOptions::from_str(&url)?
            .options([("search_path", schema.as_str()), ("timezone", "UTC")]);
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await?;
        let db = LabDb { pool, schema, url };
        for sql in [SCHEMA_SQL, BALANCED_SQL, APPEND_ONLY_SQL] {
            if let Err(e) = sqlx::raw_sql(sql).execute(&db.pool).await {
                db.cleanup().await?;
                return Err(e.into());
            }
        }
        Ok(db)
    }

    /// Xoá đúng schema của ca này.
    pub async fn cleanup(self) -> Result<()> {
        self.pool.close().await;
        let mut conn = PgConnection::connect(&self.url).await?;
        sqlx::raw_sql(AssertSqlSafe(format!(
            "DROP SCHEMA IF EXISTS {} CASCADE",
            self.schema
        )))
        .execute(&mut conn)
        .await?;
        conn.close().await?;
        Ok(())
    }
}

/// Chạy `body` trong schema riêng và luôn dọn schema, kể cả khi `body` lỗi hoặc panic.
pub async fn with_lab<F, Fut, T>(body: F) -> Result<T>
where
    F: FnOnce(LabDb) -> Fut,
    Fut: Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    let db = LabDb::create().await?;
    // spawn: panic trong `body` thành JoinError, ta vẫn dọn được schema.
    let joined = tokio::spawn(body(db.clone())).await;
    let cleaned = db.cleanup().await;
    match joined {
        Ok(result) => {
            let value = result?;
            cleaned?;
            Ok(value)
        }
        Err(join_err) => bail!("ca chạy panic hoặc bị huỷ: {join_err}"),
    }
}
