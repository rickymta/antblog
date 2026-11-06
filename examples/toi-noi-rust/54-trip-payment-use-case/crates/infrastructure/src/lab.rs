//! Cô lập một ca chạy: schema `lab_<uuid>`, topic `lab.54.<uuid>.*`, consumer group riêng; dọn cuối ca.

use anyhow::{Context, Result};
use sqlx::{AssertSqlSafe, Connection, PgConnection};
use uuid::Uuid;

use crate::pg::PgStore;
use crate::service::ServiceConfig;
use crate::{Fault, kafka};

/// Tài nguyên riêng của một ca chạy. Thiếu biến môi trường hoặc không kết nối được thì lỗi rõ, không skip.
#[derive(Debug, Clone)]
pub struct LabEnv {
    /// URL PostgreSQL từ `TOI_NOI_PG_URL`.
    pub pg_url: String,
    /// Broker từ `TOI_NOI_KAFKA`.
    pub kafka: String,
    /// Schema riêng, dạng `lab_<uuid>`.
    pub schema: String,
    /// Tiền tố topic, dạng `lab.54.<uuid>`.
    pub prefix: String,
    /// Consumer group của service trong ca này.
    pub group: String,
}

impl LabEnv {
    /// Dựng schema (cùng bảng) và ba topic.
    ///
    /// # Errors
    /// Thiếu `TOI_NOI_PG_URL`/`TOI_NOI_KAFKA`, PostgreSQL hoặc Kafka không trả lời.
    pub async fn setup() -> Result<LabEnv> {
        LabEnv::setup_with(true).await
    }

    /// Như [`LabEnv::setup`] nhưng với `with_kafka = false` chỉ dựng schema: các ca chỉ cần PostgreSQL
    /// khỏi tốn thời gian tạo topic. `kafka` để trống và `topic_*` không dùng được.
    ///
    /// # Errors
    /// Như [`LabEnv::setup`].
    pub async fn setup_with(with_kafka: bool) -> Result<LabEnv> {
        let id = Uuid::new_v4();
        let env = LabEnv {
            pg_url: required("TOI_NOI_PG_URL")?,
            kafka: if with_kafka {
                required("TOI_NOI_KAFKA")?
            } else {
                String::new()
            },
            schema: format!("lab_{}", id.simple()),
            prefix: format!("lab.54.{id}"),
            group: format!("lab-54-{id}"),
        };
        let mut admin = PgConnection::connect(&env.pg_url)
            .await
            .context("không kết nối được PostgreSQL ở TOI_NOI_PG_URL; compose đã chạy chưa?")?;
        // Tên schema do ta sinh từ UUID (chữ hex): không có dữ liệu ngoài, an toàn để ghép chuỗi.
        sqlx::query(AssertSqlSafe(format!("CREATE SCHEMA {}", env.schema)))
            .execute(&mut admin)
            .await
            .context("CREATE SCHEMA")?;
        if with_kafka {
            let topics = [env.topic_trips(), env.topic_captures(), env.topic_out()];
            if let Err(err) = kafka::create_topics(&env.kafka, &topics).await {
                let _ = env.drop_schema().await;
                return Err(err.context("không tạo được topic; Kafka đã chạy chưa?"));
            }
        }
        // Dựng bảng ngay bây giờ, một lần, trước khi bất kỳ service hay tiến trình con nào mở kết nối.
        if let Err(err) = env.store(Fault::None).await {
            let _ = env.cleanup_inner().await;
            return Err(err.context("không dựng được schema"));
        }
        Ok(env)
    }

    /// Topic `TripCompleted` của ca này.
    pub fn topic_trips(&self) -> String {
        format!("{}.trip.completed", self.prefix)
    }

    /// Topic `PaymentCaptured` của ca này.
    pub fn topic_captures(&self) -> String {
        format!("{}.payment.captured", self.prefix)
    }

    /// Topic `LedgerEntryPosted` của ca này.
    pub fn topic_out(&self) -> String {
        format!("{}.ledger.entry-posted", self.prefix)
    }

    /// Cấu hình service trong đúng ca này.
    pub fn service_config(&self, fault: Fault, run_relay: bool) -> ServiceConfig {
        ServiceConfig {
            pg_url: self.pg_url.clone(),
            schema: Some(self.schema.clone()),
            kafka: self.kafka.clone(),
            topic_trips: self.topic_trips(),
            topic_captures: self.topic_captures(),
            topic_out: self.topic_out(),
            group: self.group.clone(),
            fault,
            run_relay,
        }
    }

    /// Biến môi trường để chạy `trip-payment serve` như tiến trình con trong đúng ca này.
    pub fn child_vars(&self, fault: Fault, run_relay: bool) -> Vec<(String, String)> {
        vec![
            ("TOI_NOI_PG_URL".into(), self.pg_url.clone()),
            ("TOI_NOI_KAFKA".into(), self.kafka.clone()),
            ("PAYMENT_PG_SCHEMA".into(), self.schema.clone()),
            ("PAYMENT_TOPIC_PREFIX".into(), self.prefix.clone()),
            ("PAYMENT_GROUP".into(), self.group.clone()),
            ("PAYMENT_FAULT".into(), fault.as_str().into()),
            (
                "PAYMENT_RELAY".into(),
                if run_relay { "1" } else { "0" }.into(),
            ),
        ]
    }

    /// Một `PgStore` mới trỏ vào schema của ca (test đọc và ghi trực tiếp, không qua consumer).
    ///
    /// # Errors
    /// Không kết nối được.
    pub async fn store(&self, fault: Fault) -> Result<PgStore> {
        Ok(PgStore::connect(&self.pg_url, Some(&self.schema), &self.topic_out(), fault).await?)
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

    async fn cleanup_inner(&self) -> Result<()> {
        let schema = self.drop_schema().await;
        if self.kafka.is_empty() {
            return schema;
        }
        let topics = kafka::delete_topics(
            &self.kafka,
            &[self.topic_trips(), self.topic_captures(), self.topic_out()],
        )
        .await;
        // Hai group của ca: service và probe (người nghe LedgerEntryPosted).
        let groups = kafka::delete_groups(
            &self.kafka,
            &[self.group.clone(), format!("{}-probe", self.group)],
        )
        .await;
        schema.and(topics).and(groups)
    }

    /// Xoá schema và ba topic của ca này.
    ///
    /// # Errors
    /// Không kết nối được; phần dọn còn lại vẫn được thử.
    pub async fn cleanup(self) -> Result<()> {
        self.cleanup_inner().await
    }

    /// Dựng ca (PostgreSQL và Kafka), chạy `body`, dọn ca dù `body` thành công hay lỗi.
    ///
    /// # Errors
    /// Lỗi của `setup`, của `body`, hoặc của bước dọn khi `body` đã thành công.
    pub async fn run<F, Fut, T>(body: F) -> Result<T>
    where
        F: FnOnce(LabEnv) -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        LabEnv::run_with(true, body).await
    }

    /// Như [`LabEnv::run`] nhưng chỉ cần PostgreSQL.
    ///
    /// # Errors
    /// Như [`LabEnv::run`].
    pub async fn run_pg<F, Fut, T>(body: F) -> Result<T>
    where
        F: FnOnce(LabEnv) -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        LabEnv::run_with(false, body).await
    }

    async fn run_with<F, Fut, T>(with_kafka: bool, body: F) -> Result<T>
    where
        F: FnOnce(LabEnv) -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let env = LabEnv::setup_with(with_kafka).await?;
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
