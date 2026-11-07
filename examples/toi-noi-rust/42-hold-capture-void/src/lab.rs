//! Ca thử nghiệm PostgreSQL cô lập: mỗi ca có schema `lab_<uuid>` riêng, dọn cuối ca.

use std::future::Future;
use std::str::FromStr;

use anyhow::{Context, Result, bail};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::{AssertSqlSafe, Executor};
use uuid::Uuid;

const SCHEMA_SQL: &str = include_str!("../sql/schema.sql");

/// Một ca thử: pool trỏ vào schema riêng và pool quản trị để dọn.
#[derive(Clone)]
pub struct Lab {
    /// Pool của ca; `search_path` và `application_name` đều là tên schema.
    pub pool: PgPool,
    admin: PgPool,
    /// Tên schema `lab_<uuid>`; cũng là `application_name` của mọi kết nối trong ca.
    pub schema: String,
}

impl Lab {
    /// Tạo schema và nạp `schema.sql`. Thiếu `TOI_NOI_PG_URL` hoặc không kết nối được thì lỗi rõ.
    ///
    /// # Errors
    /// Thiếu biến môi trường, không kết nối được PostgreSQL hoặc nạp schema lỗi.
    pub async fn new() -> Result<Lab> {
        let Ok(url) = std::env::var("TOI_NOI_PG_URL") else {
            bail!(
                "thiếu TOI_NOI_PG_URL. Bật hạ tầng: docker compose -f \
                 examples/toi-noi-rust/infra/docker-compose.yml up -d --wait, \
                 rồi export TOI_NOI_PG_URL theo examples/toi-noi-rust/infra/README.md"
            );
        };
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

        // `options` gửi search_path ngay lúc mở kết nối; application_name để lab tìm
        // đúng phiên của mình trong pg_stat_activity.
        let options = PgConnectOptions::from_str(&url)?
            .options([("search_path", schema.as_str())])
            .application_name(&schema);
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await?;
        sqlx::raw_sql(SCHEMA_SQL).execute(&pool).await?;
        Ok(Lab {
            pool,
            admin,
            schema,
        })
    }

    /// `DROP SCHEMA ... CASCADE` rồi đóng pool.
    ///
    /// # Errors
    /// Lỗi khi xoá schema.
    pub async fn cleanup(self) -> Result<()> {
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

/// Chạy `body` trong một ca, luôn dọn schema, kể cả khi `body` panic (panic được ném lại sau khi dọn).
///
/// # Errors
/// Lỗi tạo hoặc dọn ca, hoặc lỗi `body` trả về.
pub async fn with_lab<F, Fut, T>(body: F) -> Result<T>
where
    F: FnOnce(Lab) -> Fut,
    Fut: Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    let lab = Lab::new().await?;
    // Chạy trong task riêng: panic của body thành JoinError, ta vẫn kịp dọn.
    let joined = tokio::spawn(body(lab.clone())).await;
    lab.cleanup().await?;
    match joined {
        Ok(result) => result,
        Err(join_err) => std::panic::resume_unwind(join_err.into_panic()),
    }
}
