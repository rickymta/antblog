//! Hạ tầng lab: mỗi lần chạy một schema `lab_<uuid>` riêng, dọn sạch khi xong.

use std::str::FromStr;

use anyhow::{Context, Result};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{AssertSqlSafe, Connection, PgConnection, PgPool};
use uuid::Uuid;

use crate::store::CHART;

/// Một schema lab đang sống cùng pool trỏ vào nó.
#[derive(Debug)]
pub struct LabDb {
    url: String,
    schema: String,
    /// Pool có `search_path` = schema lab; mọi bảng nằm trong schema này, không đụng `public`.
    pub pool: PgPool,
}

/// URL PostgreSQL từ `TOI_NOI_PG_URL`; thiếu thì lỗi nói rõ cách bật hạ tầng.
///
/// # Errors
/// Thiếu biến môi trường.
pub fn pg_url() -> Result<String> {
    std::env::var("TOI_NOI_PG_URL").context(
        "thiếu biến TOI_NOI_PG_URL. Bật hạ tầng: docker compose -f \
         examples/toi-noi-rust/infra/docker-compose.yml up -d --wait, rồi export biến theo infra/README.md",
    )
}

impl LabDb {
    /// Tạo schema, bảng và mở sẵn các tài khoản của sơ đồ rút gọn.
    ///
    /// # Errors
    /// Thiếu `TOI_NOI_PG_URL`, không kết nối được, hoặc DDL lỗi (schema đã tạo được dọn lại).
    pub async fn setup() -> Result<LabDb> {
        let url = pg_url()?;
        let schema = format!("lab_{}", Uuid::new_v4().simple());

        let mut admin = PgConnection::connect(&url)
            .await
            .context("không kết nối được PostgreSQL ở TOI_NOI_PG_URL; compose đã chạy chưa?")?;
        // Tên schema do ta sinh (chữ hex), không có dữ liệu ngoài: an toàn để ghép chuỗi.
        sqlx::query(AssertSqlSafe(format!("CREATE SCHEMA {schema}")))
            .execute(&mut admin)
            .await
            .context("CREATE SCHEMA")?;

        // `options` gửi `-c search_path=...` ngay trong lúc mở kết nối, nên mọi kết nối của pool đều trỏ vào schema lab.
        let options = PgConnectOptions::from_str(&url)?.options([("search_path", schema.as_str())]);
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .context("mở pool")?;

        let lab = LabDb { url, schema, pool };
        if let Err(e) = lab.create_tables().await {
            let _ = lab.cleanup().await;
            return Err(e);
        }
        Ok(lab)
    }

    async fn create_tables(&self) -> Result<()> {
        sqlx::raw_sql(include_str!("../schema.sql"))
            .execute(&self.pool)
            .await
            .context("tạo bảng")?;
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

    /// Tên schema lab (để kiểm tra dọn sạch).
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Đóng pool rồi `DROP SCHEMA ... CASCADE`. Trả lại tên schema đã xoá.
    ///
    /// # Errors
    /// Không kết nối được hoặc DROP lỗi.
    pub async fn cleanup(self) -> Result<String> {
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
    pub async fn schema_exists(name: &str) -> Result<bool> {
        let mut admin = PgConnection::connect(&pg_url()?).await?;
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_namespace WHERE nspname = $1")
            .bind(name)
            .fetch_one(&mut admin)
            .await?;
        Ok(n > 0)
    }
}
