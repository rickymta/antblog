//! Môi trường một ca chạy: schema PostgreSQL `lab_<uuid>` và tiền tố key Redis `lab:28:<uuid>:`.

use std::str::FromStr;

use anyhow::{Context, Result, bail};
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{AssertSqlSafe, Connection, PgConnection, PgPool};
use uuid::Uuid;

use crate::store;

/// Mọi thứ một ca cần; mỗi ca có schema và tiền tố key riêng nên chạy song song được.
pub struct Lab {
    pub pool: PgPool,
    pub redis: ConnectionManager,
    pub schema: String,
    pub prefix: String,
    pg_url: String,
}

impl Lab {
    /// Kết nối PostgreSQL và Redis, tạo schema riêng và bảng sổ cái. Thiếu biến môi trường
    /// hoặc không kết nối được thì trả lỗi rõ ràng, không bỏ qua.
    pub async fn start() -> Result<Lab> {
        let pg_url = env_var("TOI_NOI_PG_URL")?;
        let redis_url = env_var("TOI_NOI_REDIS")?;
        let id = Uuid::new_v4().simple().to_string();
        let schema = format!("lab_{id}");
        let prefix = format!("lab:28:{id}:");

        // Kết nối Redis trước khi tạo schema: Redis hỏng thì không để lại schema mồ côi.
        let client = Client::open(redis_url.as_str()).context("TOI_NOI_REDIS không hợp lệ")?;
        let redis = ConnectionManager::new(client).await.context(
            "không kết nối được Redis; bật bằng docker compose trong examples/toi-noi-rust/infra",
        )?;

        let options = PgConnectOptions::from_str(&pg_url).context("TOI_NOI_PG_URL không hợp lệ")?;
        let mut admin = PgConnection::connect_with(&options)
            .await
            .context("không kết nối được PostgreSQL; bật bằng docker compose trong examples/toi-noi-rust/infra")?;
        // Tên schema do chính ta sinh (`lab_` + hex của UUID), không có đầu vào bên ngoài.
        sqlx::query(AssertSqlSafe(format!("CREATE SCHEMA {schema}")))
            .execute(&mut admin)
            .await?;
        admin.close().await?;

        // `search_path` đặt khi mở kết nối: mọi câu lệnh của pool chỉ thấy schema của ca này.
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect_with(options.options([("search_path", schema.as_str())]))
            .await?;
        store::create_tables(&pool).await?;

        Ok(Lab {
            pool,
            redis,
            schema,
            prefix,
            pg_url,
        })
    }

    /// Key có tiền tố của ca.
    pub fn key(&self, suffix: &str) -> String {
        format!("{}{suffix}", self.prefix)
    }

    /// Dọn đúng phần của ca này: xoá key theo tiền tố bằng SCAN (không bao giờ FLUSHALL/FLUSHDB)
    /// rồi `DROP SCHEMA ... CASCADE`.
    pub async fn finish(self) -> Result<()> {
        let mut conn = self.redis.clone();
        let pattern = format!("{}*", self.prefix);
        let mut keys: Vec<String> = Vec::new();
        {
            let mut iter = conn.scan_match::<_, String>(&pattern).await?;
            while let Some(key) = iter.next_item().await {
                keys.push(key?);
            }
        }
        for key in keys {
            let _: () = conn.del(key).await?;
        }
        self.pool.close().await;
        let options = PgConnectOptions::from_str(&self.pg_url)?;
        let mut admin = PgConnection::connect_with(&options).await?;
        sqlx::query(AssertSqlSafe(format!(
            "DROP SCHEMA {} CASCADE",
            self.schema
        )))
        .execute(&mut admin)
        .await?;
        admin.close().await?;
        Ok(())
    }
}

fn env_var(name: &str) -> Result<String> {
    match std::env::var(name) {
        Ok(value) if !value.is_empty() => Ok(value),
        _ => bail!(
            "thiếu biến môi trường {name}; xem examples/toi-noi-rust/infra/README.md \
             (docker compose up -d --wait, rồi export biến)"
        ),
    }
}
