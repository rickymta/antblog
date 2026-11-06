//! Ca thử nghiệm PostgreSQL cô lập: mỗi ca có schema `lab_<uuid>` riêng, dọn cuối ca.

use std::future::Future;
use std::str::FromStr;

use anyhow::{Context, Result, bail};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::{AssertSqlSafe, Connection, Executor, PgConnection};
use uuid::Uuid;

/// Schema phiên bản 1 (bốn bảng chuẩn + `fee_events`, `feature_flags`, checkpoint).
const SCHEMA_V1: &str = include_str!("../sql/schema_v1.sql");
/// Trigger cân bút toán: đúng bản của bài 25.
const BALANCE_CHECK: &str = include_str!("../sql/02_balance_check.sql");

/// Một ca thử: pool trỏ vào schema riêng (qua `search_path`) và pool quản trị để dọn.
#[derive(Clone)]
pub struct Lab {
    /// Pool của ca; mọi kết nối đã đặt `search_path` về schema của ca.
    pub pool: PgPool,
    admin: PgPool,
    /// Tên schema `lab_<uuid>`.
    pub schema: String,
    options: PgConnectOptions,
}

impl Lab {
    /// Tạo schema và nạp schema phiên bản 1. Thiếu `TOI_NOI_PG_URL` hoặc không kết nối được thì lỗi rõ.
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

        // Schema đã tồn tại: lỗi từ đây trở đi phải dọn nó rồi mới trả lỗi.
        match Lab::open(&url, &schema).await {
            Ok((pool, options)) => Ok(Lab {
                pool,
                admin,
                schema,
                options,
            }),
            Err(err) => {
                let drop_schema = format!("DROP SCHEMA IF EXISTS {schema} CASCADE");
                let _ = admin.execute(AssertSqlSafe(drop_schema)).await;
                admin.close().await;
                Err(err.context("dựng ca thử thất bại, đã dọn schema"))
            }
        }
    }

    async fn open(url: &str, schema: &str) -> Result<(PgPool, PgConnectOptions)> {
        let options = PgConnectOptions::from_str(url)?.options([("search_path", schema)]);
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect_with(options.clone())
            .await?;
        sqlx::raw_sql(SCHEMA_V1).execute(&pool).await?;
        sqlx::raw_sql(BALANCE_CHECK).execute(&pool).await?;
        // Autovacuum giữ ShareUpdateExclusiveLock có thể làm ALTER với lock_timeout ngắn hết hạn
        // ngoài ý muốn khi máy tải cao: tắt trên hai bảng bị ALTER để kết quả không phụ thuộc thời điểm.
        sqlx::raw_sql(
            "ALTER TABLE postings SET (autovacuum_enabled = false); \
             ALTER TABLE fee_events SET (autovacuum_enabled = false);",
        )
        .execute(&pool)
        .await?;
        Ok((pool, options))
    }

    /// Một kết nối riêng, có `application_name` để tìm lại nó trong `pg_stat_activity`.
    ///
    /// # Errors
    /// Không kết nối được.
    pub async fn connect(&self, application_name: &str) -> Result<PgConnection> {
        let options = self.options.clone().application_name(application_name);
        Ok(PgConnection::connect_with(&options).await?)
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

/// Chạy `body` trong một ca và trả giá trị của nó; luôn dọn schema, kể cả khi `body` panic
/// (panic được ném lại sau khi dọn).
///
/// # Errors
/// Lỗi tạo hoặc dọn ca, hoặc lỗi `body` trả về.
pub async fn with_lab<T, F, Fut>(body: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce(Lab) -> Fut,
    Fut: Future<Output = Result<T>> + Send + 'static,
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
