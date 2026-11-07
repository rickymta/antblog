//! Môi trường một ca thử: schema PostgreSQL `lab_<uuid>`, dọn cuối ca kể cả khi lỗi.
//! Thiếu biến môi trường hoặc không kết nối được thì FAIL rõ, không skip.

use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::{AssertSqlSafe, Executor};
use uuid::Uuid;

/// Schema chuẩn của loạt cộng bảng payout (xem `sql/schema.sql`).
const SCHEMA_SQL: &str = include_str!("../sql/schema.sql");

/// Đọc `TOI_NOI_PG_URL`, báo cách bật hạ tầng nếu thiếu.
fn pg_url() -> Result<String> {
    std::env::var("TOI_NOI_PG_URL").map_err(|_| {
        anyhow::anyhow!(
            "thiếu TOI_NOI_PG_URL. Bật hạ tầng: docker compose -f \
             examples/toi-noi-rust/infra/docker-compose.yml up -d --wait, \
             rồi export theo examples/toi-noi-rust/infra/README.md"
        )
    })
}

/// Schema PostgreSQL riêng của một ca.
pub struct PgLab {
    /// Pool của ca: mọi kết nối đã trỏ vào schema của ca qua `search_path`.
    pub pool: PgPool,
    /// Tên schema `lab_<uuid>`.
    pub schema: String,
    admin: PgPool,
}

impl PgLab {
    /// Tạo schema mới và nạp `schema.sql`.
    ///
    /// # Errors
    /// Thiếu `TOI_NOI_PG_URL`, không kết nối được hoặc nạp schema lỗi.
    pub async fn new() -> Result<PgLab> {
        let url = pg_url()?;
        let admin = PgPoolOptions::new()
            .max_connections(1)
            .connect(&url)
            .await
            .context("không kết nối được PostgreSQL ở TOI_NOI_PG_URL (compose đã bật chưa?)")?;
        // Tên schema do ta sinh (hex), không phải dữ liệu ngoài: an toàn để ghép vào SQL.
        let schema = format!("lab_{}", Uuid::new_v4().simple());
        admin
            .execute(AssertSqlSafe(format!("CREATE SCHEMA {schema}")))
            .await?;
        // search_path đặt ngay trong tham số khởi động kết nối, không cần SET ở từng kết nối.
        let options = PgConnectOptions::from_str(&url)?.options([("search_path", schema.as_str())]);
        let pool = PgPoolOptions::new()
            .max_connections(16)
            .connect_lazy_with(options);
        let lab = PgLab {
            pool,
            schema,
            admin,
        };
        // Nạp schema lỗi thì vẫn xoá schema vừa tạo rồi mới trả lỗi.
        if let Err(err) = sqlx::raw_sql(SCHEMA_SQL).execute(&lab.pool).await {
            lab.cleanup().await?;
            return Err(err.into());
        }
        Ok(lab)
    }

    /// `DROP SCHEMA ... CASCADE` rồi đóng pool.
    ///
    /// # Errors
    /// Lỗi khi xoá schema.
    pub async fn cleanup(&self) -> Result<()> {
        self.pool.close().await;
        self.admin
            .execute(AssertSqlSafe(format!(
                "DROP SCHEMA {} CASCADE",
                self.schema
            )))
            .await?;
        self.admin.close().await;
        Ok(())
    }
}

/// Chạy `body` trong một ca, luôn dọn schema kể cả khi `body` lỗi hoặc panic.
///
/// # Errors
/// Lỗi dựng hoặc dọn ca, hoặc lỗi `body` trả về.
pub async fn with_pg<F, Fut>(body: F) -> Result<()>
where
    F: FnOnce(Arc<PgLab>) -> Fut,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    let lab = Arc::new(PgLab::new().await?);
    // Chạy trong task riêng: panic của body thành JoinError, ta vẫn kịp dọn.
    let joined = tokio::spawn(body(lab.clone())).await;
    let cleaned = lab.cleanup().await;
    cleaned?;
    match joined {
        Ok(result) => result,
        Err(join_err) => std::panic::resume_unwind(join_err.into_panic()),
    }
}
