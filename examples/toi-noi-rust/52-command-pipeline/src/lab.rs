//! Môi trường một ca thử: schema PostgreSQL `lab_<uuid>`, dọn cuối ca.
//! Thiếu biến môi trường hoặc không kết nối được thì FAIL rõ, không skip.

use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::{AssertSqlSafe, Executor};
use uuid::Uuid;

// Schema chuẩn và trigger cân bút toán: đúng bản của lab 25.
const SCHEMA_SQL: &str = include_str!("../../25-schema-so-cai/sql/01_schema.sql");
const BALANCE_CHECK_SQL: &str = include_str!("../../25-schema-so-cai/sql/02_balance_check.sql");
const OUTBOX_SQL: &str = include_str!("../sql/outbox.sql");

/// Schema PostgreSQL riêng của một ca.
pub struct PgLab {
    /// Pool của ca (mọi kết nối đã trỏ vào schema của ca).
    pub pool: PgPool,
    /// Tên schema `lab_<uuid>`.
    pub schema: String,
    admin: PgPool,
}

impl PgLab {
    /// Tạo schema mới và nạp schema chuẩn, trigger cân bút toán, outbox, tài khoản mẫu.
    ///
    /// # Errors
    /// Thiếu `TOI_NOI_PG_URL`, không kết nối được hoặc nạp schema lỗi.
    pub async fn new() -> Result<PgLab> {
        let url = std::env::var("TOI_NOI_PG_URL").map_err(|_| {
            anyhow::anyhow!(
                "thiếu TOI_NOI_PG_URL. Bật hạ tầng: docker compose -f \
                 examples/toi-noi-rust/infra/docker-compose.yml up -d --wait, \
                 rồi export theo examples/toi-noi-rust/infra/README.md"
            )
        })?;
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
        // Dựng lỗi giữa chừng thì xóa schema vừa tạo rồi mới báo lỗi, không để sót.
        let prepared = async {
            let options =
                PgConnectOptions::from_str(&url)?.options([("search_path", schema.as_str())]);
            let pool = PgPoolOptions::new()
                .max_connections(4)
                .connect_with(options)
                .await?;
            for sql in [SCHEMA_SQL, BALANCE_CHECK_SQL, OUTBOX_SQL] {
                sqlx::raw_sql(sql).execute(&pool).await?;
            }
            Ok::<_, anyhow::Error>(pool)
        }
        .await;
        match prepared {
            Ok(pool) => Ok(PgLab {
                pool,
                schema,
                admin,
            }),
            Err(err) => {
                let _ = admin
                    .execute(AssertSqlSafe(format!("DROP SCHEMA {schema} CASCADE")))
                    .await;
                Err(err)
            }
        }
    }

    /// `DROP SCHEMA ... CASCADE` rồi đóng pool.
    ///
    /// # Errors
    /// Lỗi khi xóa schema.
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

    /// Số dòng của `journal_entries` và `outbox`, đọc ngoài mọi transaction.
    ///
    /// # Errors
    /// Lỗi truy vấn.
    pub async fn counts(&self) -> Result<(i64, i64)> {
        let entries = sqlx::query_scalar("SELECT count(*) FROM journal_entries")
            .fetch_one(&self.pool)
            .await?;
        let outbox = sqlx::query_scalar("SELECT count(*) FROM outbox")
            .fetch_one(&self.pool)
            .await?;
        Ok((entries, outbox))
    }
}

/// Chạy `body` với một `PgLab`, luôn dọn schema kể cả khi `body` lỗi hoặc panic.
///
/// # Errors
/// Lỗi dựng, dọn hoặc lỗi `body` trả về.
pub async fn with_lab<T, F, Fut>(body: F) -> Result<T>
where
    F: FnOnce(Arc<PgLab>) -> Fut,
    Fut: Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    let lab = Arc::new(PgLab::new().await?);
    // Task riêng: panic của body thành JoinError, ta vẫn kịp dọn.
    let joined = tokio::spawn(body(lab.clone())).await;
    let cleaned = lab.cleanup().await;
    cleaned?;
    match joined {
        Ok(result) => result,
        Err(join_err) => std::panic::resume_unwind(join_err.into_panic()),
    }
}
