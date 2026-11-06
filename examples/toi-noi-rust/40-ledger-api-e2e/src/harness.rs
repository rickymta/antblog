//! Cô lập một ca chạy: schema `lab_<uuid>`, topic `lab.40.<uuid>.*`, consumer group riêng; dọn cuối ca.

use std::str::FromStr;

use anyhow::{Context, Result};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::{AssertSqlSafe, Connection, PgConnection};
use uuid::Uuid;

use crate::config::Config;
use crate::kafka;

/// Tài nguyên riêng của một ca chạy. Thiếu biến môi trường hoặc không kết nối được thì lỗi rõ, không skip.
#[derive(Debug, Clone)]
pub struct LabEnv {
    /// URL PostgreSQL từ `TOI_NOI_PG_URL`.
    pub pg_url: String,
    /// Broker từ `TOI_NOI_KAFKA`.
    pub kafka: String,
    /// Schema riêng, dạng `lab_<uuid>`.
    pub schema: String,
    /// Tiền tố topic, dạng `lab.40.<uuid>`.
    pub prefix: String,
    /// Consumer group của service trong ca này.
    pub group: String,
}

impl LabEnv {
    /// Dựng schema và hai topic.
    ///
    /// # Errors
    /// Thiếu `TOI_NOI_PG_URL`/`TOI_NOI_KAFKA`, PostgreSQL hoặc Kafka không trả lời.
    pub async fn setup() -> Result<LabEnv> {
        let id = Uuid::new_v4();
        let env = LabEnv {
            pg_url: required("TOI_NOI_PG_URL")?,
            kafka: required("TOI_NOI_KAFKA")?,
            schema: format!("lab_{}", id.simple()),
            prefix: format!("lab.40.{id}"),
            group: format!("lab-40-{id}"),
        };
        let mut admin = PgConnection::connect(&env.pg_url)
            .await
            .context("không kết nối được PostgreSQL ở TOI_NOI_PG_URL; compose đã chạy chưa?")?;
        // Tên schema do ta sinh từ UUID (chữ hex): không có dữ liệu ngoài, an toàn để ghép chuỗi.
        sqlx::query(AssertSqlSafe(format!("CREATE SCHEMA {}", env.schema)))
            .execute(&mut admin)
            .await
            .context("CREATE SCHEMA")?;
        if let Err(err) = kafka::create_topics(&env.kafka, &[env.topic_in(), env.topic_out()]).await
        {
            let _ = env.drop_schema().await;
            return Err(err.context("không tạo được topic; Kafka đã chạy chưa?"));
        }
        Ok(env)
    }

    /// Topic `TripCompleted` của ca này.
    pub fn topic_in(&self) -> String {
        format!("{}.trip.completed", self.prefix)
    }

    /// Topic `LedgerEntryPosted` của ca này.
    pub fn topic_out(&self) -> String {
        format!("{}.ledger.entry-posted", self.prefix)
    }

    /// Biến môi trường để chạy binary `ledger-api serve` trong đúng ca này.
    pub fn vars(&self) -> Vec<(String, String)> {
        vec![
            ("TOI_NOI_PG_URL".into(), self.pg_url.clone()),
            ("TOI_NOI_KAFKA".into(), self.kafka.clone()),
            ("LEDGER_PG_SCHEMA".into(), self.schema.clone()),
            ("LEDGER_TOPIC_PREFIX".into(), self.prefix.clone()),
            ("LEDGER_CONSUMER_GROUP".into(), self.group.clone()),
        ]
    }

    /// Cấu hình của service, cùng đường đọc với binary thật.
    ///
    /// # Errors
    /// Không xảy ra với giá trị đã qua `setup`; giữ `Result` để dùng chung kiểu lỗi.
    pub fn config(&self) -> Result<Config> {
        let vars = self.vars();
        Ok(Config::from_lookup(|name| {
            vars.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone())
        })?)
    }

    /// Pool riêng cho test đọc thẳng bảng (kiểm bút toán, inbox, outbox), không qua service.
    ///
    /// # Errors
    /// Không kết nối được.
    pub async fn inspect_pool(&self) -> Result<PgPool> {
        let options = PgConnectOptions::from_str(&self.pg_url)?
            .options([("search_path", self.schema.as_str())]);
        Ok(PgPoolOptions::new()
            .max_connections(2)
            .connect_with(options)
            .await?)
    }

    async fn drop_schema(&self) -> Result<()> {
        let mut admin = PgConnection::connect(&self.pg_url).await?;
        sqlx::query(AssertSqlSafe(format!(
            "DROP SCHEMA IF EXISTS {} CASCADE",
            self.schema
        )))
        .execute(&mut admin)
        .await?;
        Ok(())
    }

    /// Xoá schema và hai topic của ca này.
    ///
    /// # Errors
    /// Không kết nối được; phần dọn còn lại vẫn được thử.
    pub async fn cleanup(self) -> Result<()> {
        let schema = self.drop_schema().await;
        let topics = kafka::delete_topics(&self.kafka, &[self.topic_in(), self.topic_out()]).await;
        schema.and(topics)
    }

    /// Dựng ca, chạy `body`, dọn ca dù `body` thành công hay lỗi.
    ///
    /// # Errors
    /// Lỗi của `setup`, của `body`, hoặc của bước dọn khi `body` đã thành công.
    pub async fn run<F, Fut, T>(body: F) -> Result<T>
    where
        F: FnOnce(LabEnv) -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let env = LabEnv::setup().await?;
        let outcome = body(env.clone()).await;
        let cleaned = env.cleanup().await;
        let value = outcome?;
        cleaned.context("dọn ca chạy")?;
        Ok(value)
    }
}

fn required(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| {
        format!(
            "thiếu biến {name}. Bật hạ tầng: docker compose -f examples/toi-noi-rust/infra/docker-compose.yml up -d --wait, \
             rồi export biến theo infra/README.md"
        )
    })
}
