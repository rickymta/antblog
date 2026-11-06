//! Kết nối, pool, migration và schema riêng cho mỗi lần chạy.

use std::str::FromStr;
use std::time::Duration;

use anyhow::{Context, Result};
use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use uuid::Uuid;

/// Tên biến môi trường chứa URL PostgreSQL của lab.
pub const ENV_URL: &str = "TOI_NOI_PG_URL";

/// Số kết nối tối đa của pool lab. PostgreSQL giới hạn tổng kết nối (mặc định 100),
/// nên mỗi service phải chọn con số có chủ ý, không để nguyên mặc định 10.
pub const MAX_CONNECTIONS: u32 = 5;

/// Chờ tối đa bao lâu để lấy một kết nối rảnh từ pool trước khi báo lỗi.
pub const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// Đọc URL từ môi trường. Thiếu biến thì lỗi rõ, không bỏ qua.
///
/// # Errors
/// Lỗi nếu `TOI_NOI_PG_URL` chưa đặt.
pub fn database_url() -> Result<String> {
    std::env::var(ENV_URL).with_context(|| {
        format!(
            "thiếu biến {ENV_URL}; bật hạ tầng bằng `docker compose -f \
             examples/toi-noi-rust/infra/docker-compose.yml up -d --wait` \
             rồi đặt biến theo infra/README.md"
        )
    })
}

/// Cấu hình pool dùng chung cho lab: giới hạn kết nối và thời gian chờ.
pub fn pool_options(max_connections: u32, acquire_timeout: Duration) -> PgPoolOptions {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(acquire_timeout)
}

/// Một schema tạm `lab_<uuid>` cùng pool trỏ vào đó. Mọi bảng của lab nằm trong schema này,
/// không bao giờ nằm ở `public`, nên nhiều lần chạy song song không đụng nhau.
pub struct LabSchema {
    name: String,
    /// Pool của lab: mỗi kết nối đã đặt `search_path` về schema tạm.
    pub pool: PgPool,
    admin: PgPool,
}

impl LabSchema {
    /// Tạo schema, mở pool vào schema đó và chạy migration trong `migrations/`.
    ///
    /// # Errors
    /// Lỗi nếu thiếu `TOI_NOI_PG_URL`, không kết nối được, hoặc migration lỗi.
    pub async fn create() -> Result<LabSchema> {
        let url = database_url()?;
        // Pool quản trị chỉ để CREATE/DROP SCHEMA; một kết nối là đủ.
        let admin = pool_options(1, ACQUIRE_TIMEOUT)
            .connect(&url)
            .await
            .with_context(|| {
                format!(
                    "không kết nối được PostgreSQL qua {ENV_URL}; \
                     kiểm tra `docker compose ... up -d --wait` trong infra/README.md"
                )
            })?;

        let name = format!("lab_{}", Uuid::new_v4().simple());
        // Tên schema tự sinh (chữ thường + số), nên ghép chuỗi ở đây là an toàn.
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!("create schema {name}")))
            .execute(&admin)
            .await
            .context("tạo schema lab")?;

        // `options` gửi `-c search_path=<schema>` trong startup message của mỗi kết nối.
        let connect = PgConnectOptions::from_str(&url)
            .context("URL PostgreSQL không hợp lệ")?
            .options([("search_path", name.as_str())]);
        let pool = pool_options(MAX_CONNECTIONS, ACQUIRE_TIMEOUT)
            .connect_with(connect)
            .await
            .context("mở pool vào schema lab")?;

        // Đường dẫn tính từ thư mục chứa Cargo.toml; file được nhúng vào binary lúc biên dịch.
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .context("chạy migration")?;

        Ok(LabSchema { name, pool, admin })
    }

    /// Tên schema tạm.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Đóng pool và `DROP SCHEMA ... CASCADE`. Gọi cuối ca, kể cả khi ca lỗi.
    ///
    /// # Errors
    /// Lỗi nếu không xoá được schema.
    pub async fn drop_schema(self) -> Result<()> {
        self.pool.close().await;
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "drop schema {} cascade",
            self.name
        )))
        .execute(&self.admin)
        .await
        .context("xoá schema lab")?;
        self.admin.close().await;
        Ok(())
    }
}

/// Chạy `body` trong một schema tạm rồi luôn dọn schema, dù `body` trả lỗi.
///
/// # Errors
/// Lỗi của `body` hoặc lỗi dọn schema (ưu tiên lỗi của `body`).
pub async fn with_schema<F, Fut, T>(body: F) -> Result<T>
where
    F: FnOnce(PgPool) -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let schema = LabSchema::create().await?;
    let result = body(schema.pool.clone()).await;
    let dropped = schema.drop_schema().await;
    let value = result?;
    dropped?;
    Ok(value)
}
