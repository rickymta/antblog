//! Kết nối và schema riêng cho mỗi ca chạy.

use std::future::Future;
use std::str::FromStr;

use anyhow::{Context, Result, bail};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{AssertSqlSafe, Connection, PgConnection, PgPool};
use uuid::Uuid;

/// Bảng, trigger append-only và trigger không mở lại kỳ.
pub const SCHEMA_SQL: &str = include_str!("../sql/01_schema.sql");
/// Trigger chặn ghi vào kỳ đã đóng, bản có khoá `FOR SHARE`.
pub const GUARD_SQL: &str = include_str!("../sql/02_period_guard.sql");
/// Cùng trigger nhưng không khoá: bản sai để dựng cuộc đua.
pub const GUARD_NAIVE_SQL: &str = include_str!("../sql/03_period_guard_naive.sql");
/// Trigger xoá snapshot bị ghi lùi vượt qua.
pub const INVALIDATE_SQL: &str = include_str!("../sql/04_invalidate_snapshots.sql");

/// Một schema `lab_<uuid>` cùng pool trỏ vào nó.
#[derive(Clone)]
pub struct LabDb {
    /// Pool có `search_path` trỏ vào schema của ca này.
    pub pool: PgPool,
    /// Tên schema.
    pub schema: String,
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
    /// Tạo schema mới và pool có `search_path` trỏ vào đó. Múi giờ phiên là UTC, như mặc định
    /// của nhiều server: bài viết cần đúng tình huống đó để cho thấy lỗi ngày UTC.
    ///
    /// # Errors
    /// Thiếu `TOI_NOI_PG_URL` hoặc không kết nối được PostgreSQL.
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

    /// Dựng schema đầy đủ: bảng, trigger chặn kỳ đã đóng, trigger vô hiệu snapshot.
    ///
    /// # Errors
    /// Lỗi SQL khi chạy các file.
    pub async fn migrate(&self) -> Result<()> {
        self.migrate_base().await?;
        self.apply_guards().await
    }

    /// Chỉ bảng và trigger append-only (dùng khi nạp dữ liệu hàng loạt trước khi bật trigger).
    ///
    /// # Errors
    /// Lỗi SQL khi chạy file.
    pub async fn migrate_base(&self) -> Result<()> {
        sqlx::raw_sql(SCHEMA_SQL).execute(&self.pool).await?;
        Ok(())
    }

    /// Bật trigger chặn kỳ đã đóng (bản có khoá) và trigger vô hiệu snapshot.
    ///
    /// # Errors
    /// Lỗi SQL khi chạy file.
    pub async fn apply_guards(&self) -> Result<()> {
        sqlx::raw_sql(GUARD_SQL).execute(&self.pool).await?;
        sqlx::raw_sql(INVALIDATE_SQL).execute(&self.pool).await?;
        Ok(())
    }

    /// Thay trigger chặn kỳ bằng bản không khoá (chỉ để dựng lỗi).
    ///
    /// # Errors
    /// Lỗi SQL.
    pub async fn use_naive_guard(&self) -> Result<()> {
        sqlx::raw_sql(GUARD_NAIVE_SQL).execute(&self.pool).await?;
        Ok(())
    }

    /// Thay lại bằng bản có khoá.
    ///
    /// # Errors
    /// Lỗi SQL.
    pub async fn use_locking_guard(&self) -> Result<()> {
        sqlx::raw_sql(GUARD_SQL).execute(&self.pool).await?;
        Ok(())
    }

    /// Xoá đúng schema của ca này.
    ///
    /// # Errors
    /// Lỗi kết nối hoặc SQL.
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
///
/// # Errors
/// Lỗi tạo schema, lỗi của `body`, hoặc `body` panic.
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

/// Tách (SQLSTATE, thông điệp) từ lỗi của PostgreSQL, nếu là lỗi từ server.
#[must_use]
pub fn pg_error(err: &sqlx::Error) -> Option<(String, String)> {
    err.as_database_error().map(|e| {
        (
            e.code().map(|c| c.to_string()).unwrap_or_default(),
            e.message().to_string(),
        )
    })
}
