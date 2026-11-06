//! Ca thử nghiệm PostgreSQL cô lập: mỗi ca có schema `lab_<uuid>` riêng, dọn cuối ca.

use std::future::Future;

use anyhow::{Context, Result, bail};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{AssertSqlSafe, Executor};
use uuid::Uuid;

/// Schema của bài (xem `sql/schema.sql`).
const SCHEMA_SQL: &str = include_str!("../sql/schema.sql");

/// Một ca thử: pool trỏ vào schema riêng (qua `search_path`) và pool quản trị để dọn.
#[derive(Clone)]
pub struct Lab {
    /// Pool của ca; mọi kết nối đã `SET search_path` về schema của ca.
    pub pool: PgPool,
    admin: PgPool,
    /// Tên schema `lab_<uuid>`.
    pub schema: String,
}

impl Lab {
    /// Tạo schema, nạp `schema.sql`. Thiếu `TOI_NOI_PG_URL` hoặc không kết nối được thì lỗi rõ.
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

        let search_path = format!("SET search_path TO {schema}");
        let pool = PgPoolOptions::new()
            .max_connections(16)
            .after_connect(move |conn, _meta| {
                let sql = search_path.clone();
                Box::pin(async move {
                    conn.execute(AssertSqlSafe(sql)).await?;
                    Ok(())
                })
            })
            .connect(&url)
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
pub async fn with_lab<F, Fut>(body: F) -> Result<()>
where
    F: FnOnce(Lab) -> Fut,
    Fut: Future<Output = Result<()>> + Send + 'static,
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
