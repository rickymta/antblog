//! Môi trường một ca: schema `lab_<uuid>`, topic `lab.67.<uuid>.<tên>`, group `lab-67-<uuid>-<hậu tố>`.
//! Thiếu biến môi trường hoặc không kết nối được thì FAIL rõ, không skip. Dọn cuối ca kể cả khi lỗi.

use std::future::Future;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::{AssertSqlSafe, Executor};
use uuid::Uuid;

use jiff::Timestamp;

use crate::clock::{Clock, ManualClock};
use crate::kafka;

const SQL_FILES: [&str; 5] = [
    include_str!("../sql/01_schema.sql"),
    include_str!("../sql/02_balance_check.sql"),
    include_str!("../sql/03_append_only.sql"),
    include_str!("../sql/04_ops_tables.sql"),
    include_str!("../sql/05_period_guard.sql"),
];

fn require_env(name: &str) -> Result<String> {
    std::env::var(name).map_err(|_| {
        anyhow::anyhow!(
            "thiếu {name}. Bật hạ tầng: docker compose -f \
             examples/toi-noi-rust/infra/docker-compose.yml up -d --wait, \
             rồi export theo examples/toi-noi-rust/infra/README.md"
        )
    })
}

/// Tài nguyên riêng của một ca.
pub struct Env {
    /// Pool đã trỏ vào schema của ca.
    pub pool: PgPool,
    /// URL PostgreSQL từ `TOI_NOI_PG_URL`.
    pub pg_url: String,
    /// Tên schema `lab_<uuid>`.
    pub schema: String,
    /// Broker từ `TOI_NOI_KAFKA`.
    pub brokers: String,
    /// Mã ca (uuid không gạch).
    pub id: String,
    /// Đồng hồ chỉnh tay của ca.
    pub clock: ManualClock,
    admin: PgPool,
    topics: Mutex<Vec<String>>,
    groups: Mutex<Vec<String>>,
}

impl Env {
    /// Tạo schema, nạp SQL.
    ///
    /// # Errors
    /// Thiếu biến môi trường, PostgreSQL không trả lời, hoặc nạp SQL lỗi.
    pub async fn new() -> Result<Env> {
        let pg_url = require_env("TOI_NOI_PG_URL")?;
        let brokers = require_env("TOI_NOI_KAFKA")?;
        let admin = PgPoolOptions::new()
            .max_connections(1)
            .connect(&pg_url)
            .await
            .context("không kết nối được PostgreSQL ở TOI_NOI_PG_URL (compose đã bật chưa?)")?;
        let id = Uuid::new_v4().simple().to_string();
        // Tên schema do ta sinh từ uuid (chữ hex), không phải dữ liệu ngoài: an toàn để ghép vào SQL.
        let schema = format!("lab_{id}");
        admin
            .execute(AssertSqlSafe(format!("CREATE SCHEMA {schema}")))
            .await?;
        let options = PgConnectOptions::from_str(&pg_url)?.options([("search_path", &*schema)]);
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await?;
        for sql in SQL_FILES {
            sqlx::raw_sql(sql).execute(&pool).await?;
        }
        Ok(Env {
            pool,
            pg_url,
            schema,
            brokers,
            id,
            clock: ManualClock::lab_start(),
            admin,
            topics: Mutex::new(Vec::new()),
            groups: Mutex::new(Vec::new()),
        })
    }

    /// Giờ hiện tại của đồng hồ chỉnh tay.
    pub fn clock_now(&self) -> Timestamp {
        self.clock.now()
    }

    /// Tạo topic `lab.67.<uuid>.<name>` và nhớ để xoá cuối ca.
    ///
    /// # Errors
    /// Broker từ chối hoặc topic không có leader.
    pub async fn create_topic(&self, name: &str) -> Result<String> {
        let full = format!("lab.67.{}.{name}", self.id);
        kafka::create_topic(&self.brokers, &full).await?;
        self.topics.lock().expect("mutex").push(full.clone());
        Ok(full)
    }

    /// Tên group `lab-67-<uuid>-<suffix>`; nhớ để xoá cuối ca.
    pub fn group(&self, suffix: &str) -> String {
        let full = format!("lab-67-{}-{suffix}", self.id);
        self.groups.lock().expect("mutex").push(full.clone());
        full
    }

    /// Xoá schema, topic và group của ca. Mọi bước đều được thử dù bước trước lỗi.
    ///
    /// # Errors
    /// Lỗi đầu tiên gặp phải.
    pub async fn cleanup(&self) -> Result<()> {
        self.pool.close().await;
        let schema = self
            .admin
            .execute(AssertSqlSafe(format!(
                "DROP SCHEMA IF EXISTS {} CASCADE",
                self.schema
            )))
            .await
            .map(|_| ())
            .map_err(anyhow::Error::from);
        self.admin.close().await;
        let topics: Vec<String> = self.topics.lock().expect("mutex").clone();
        let groups: Vec<String> = self.groups.lock().expect("mutex").clone();
        let t = kafka::delete_topics(&self.brokers, &topics).await;
        let g = kafka::delete_groups(&self.brokers, &groups).await;
        schema.and(t).and(g)
    }

    /// Dựng ca, chạy `body`, dọn dù `body` thành công, lỗi hay panic.
    ///
    /// # Errors
    /// Lỗi dựng ca, lỗi của `body`, hoặc lỗi dọn khi `body` đã thành công.
    pub async fn run<F, Fut, T>(body: F) -> Result<T>
    where
        F: FnOnce(Arc<Env>) -> Fut,
        Fut: Future<Output = Result<T>> + Send + 'static,
        T: Send + 'static,
    {
        let env = Arc::new(Env::new().await?);
        // Task riêng: panic của body thành JoinError, ta vẫn kịp dọn.
        let joined = tokio::spawn(body(env.clone())).await;
        let cleaned = env.cleanup().await;
        match joined {
            Ok(result) => {
                let value = result?;
                cleaned.context("dọn ca chạy")?;
                Ok(value)
            }
            Err(join_err) => std::panic::resume_unwind(join_err.into_panic()),
        }
    }
}
