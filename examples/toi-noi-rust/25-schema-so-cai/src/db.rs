//! Kết nối và schema riêng cho mỗi ca chạy.

use std::future::Future;
use std::str::FromStr;

use anyhow::{Context, Result, bail};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{AssertSqlSafe, Connection, PgConnection, PgPool};
use uuid::Uuid;

/// Các file SQL của lab, chạy theo thứ tự này (file 03 và 04 do từng thí nghiệm gọi).
pub const SCHEMA_SQL: &str = include_str!("../sql/01_schema.sql");
pub const BALANCE_CHECK_SQL: &str = include_str!("../sql/02_balance_check.sql");
pub const INDEX_SQL: &str = include_str!("../sql/03_index.sql");
pub const PARTITIONED_SQL: &str = include_str!("../sql/04_partitioned.sql");

/// Một schema `lab_<uuid>` cùng pool trỏ vào nó.
#[derive(Clone)]
pub struct LabDb {
    pub pool: PgPool,
    pub schema: String,
    url: String,
}

/// Đọc `TOI_NOI_PG_URL`; thiếu thì báo cách bật compose, không bỏ qua.
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
    /// Tạo schema mới và pool có `search_path` trỏ vào đó (không đụng schema `public`).
    pub async fn create() -> Result<LabDb> {
        let url = pg_url()?;
        // Tên chỉ gồm `lab_` và 32 ký tự hex nên không chứa gì để tiêm SQL.
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
        Ok(LabDb { pool, schema, url })
    }

    /// Dựng bốn bảng chuẩn và constraint trigger kiểm tổng bằng 0.
    pub async fn migrate_base(&self) -> Result<()> {
        sqlx::raw_sql(SCHEMA_SQL).execute(&self.pool).await?;
        sqlx::raw_sql(BALANCE_CHECK_SQL).execute(&self.pool).await?;
        Ok(())
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
    // spawn để panic trong `body` thành JoinError, còn ta vẫn dọn được schema.
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

/// Tách SQLSTATE và thông điệp từ lỗi của PostgreSQL (nếu là lỗi từ server).
pub fn pg_error(err: &sqlx::Error) -> Option<(String, String)> {
    err.as_database_error().map(|e| {
        (
            e.code().map(|c| c.to_string()).unwrap_or_default(),
            e.message().to_string(),
        )
    })
}
