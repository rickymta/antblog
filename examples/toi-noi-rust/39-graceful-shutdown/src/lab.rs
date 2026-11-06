//! Ca thử nghiệm cô lập: schema `lab_<uuid>`, topic `lab.39.<uuid>.*`, group `lab-39-<uuid>`.

use std::time::Duration;

use anyhow::{Context, Result, bail};
use rdkafka::ClientConfig;
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::{Offset, TopicPartitionList};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{AssertSqlSafe, Executor};
use uuid::Uuid;

use crate::events::TripCompleted;
use crate::service::{Config, build_pool};

const SCHEMA_SQL: &str = include_str!("../sql/schema.sql");

fn env_required(name: &str, hint: &str) -> Result<String> {
    std::env::var(name).map_err(|_| anyhow::anyhow!("thiếu {name}. {hint}"))
}

/// Ảnh chụp số liệu trong DB của một ca.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    /// Số bút toán.
    pub entries: i64,
    /// Số `idempotency_key` khác nhau.
    pub distinct_keys: i64,
    /// Số dòng inbox.
    pub inbox: i64,
    /// Số dòng outbox.
    pub outbox_total: i64,
    /// Số dòng outbox chưa phát.
    pub outbox_pending: i64,
    /// Tổng mọi posting: phải bằng 0.
    pub postings_sum: i64,
    /// Số tài khoản có `account_balances` khác tổng posting: phải bằng 0.
    pub balance_mismatch: i64,
}

/// Một ca: schema, hai topic, một group.
pub struct Lab {
    /// URL PostgreSQL.
    pub pg_url: String,
    /// Địa chỉ broker.
    pub kafka: String,
    /// Tên schema.
    pub schema: String,
    /// Topic `trip.completed` của ca.
    pub topic_in: String,
    /// Topic `ledger.entry-posted` của ca.
    pub topic_out: String,
    /// Consumer group của ca.
    pub group: String,
    admin_pool: PgPool,
    /// Pool của ca (đã trỏ `search_path`).
    pub pool: PgPool,
}

fn admin_client(kafka: &str) -> Result<AdminClient<DefaultClientContext>> {
    Ok(ClientConfig::new()
        .set("bootstrap.servers", kafka)
        .create()?)
}

/// Topic vừa tạo chưa chắc đã có leader: subscribe sớm quá thì `recv()` trả `UnknownTopicOrPartition`.
/// Chờ metadata đủ rồi mới trả về, có hạn chót, không ngủ cố định.
async fn wait_topic_ready(admin: &AdminClient<DefaultClientContext>, topic: &str) -> Result<()> {
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    loop {
        // `fetch_metadata` chặn thread; lab và test đều chạy trên runtime nhiều thread.
        let metadata = tokio::task::block_in_place(|| {
            admin
                .inner()
                .fetch_metadata(Some(topic), Duration::from_secs(5))
        });
        let ready = metadata.is_ok_and(|m| {
            m.topics().iter().any(|t| {
                t.name() == topic
                    && t.error().is_none()
                    && !t.partitions().is_empty()
                    && t.partitions()
                        .iter()
                        .all(|p| p.error().is_none() && p.leader() >= 0)
            })
        });
        if ready {
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            bail!("topic {topic} chưa có leader sau 30 giây");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

impl Lab {
    /// Tạo schema, nạp bảng, tạo hai topic một partition, chờ topic có leader.
    ///
    /// # Errors
    /// Thiếu biến môi trường, không kết nối được PostgreSQL/Kafka: lab FAIL rõ, không bỏ qua.
    pub async fn new() -> Result<Lab> {
        let hint =
            "Bật hạ tầng theo examples/toi-noi-rust/infra/README.md rồi export biến môi trường.";
        let pg_url = env_required("TOI_NOI_PG_URL", hint)?;
        let kafka = env_required("TOI_NOI_KAFKA", hint)?;
        let admin_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&pg_url)
            .await
            .context("không kết nối được PostgreSQL ở TOI_NOI_PG_URL")?;

        let id = Uuid::new_v4().simple().to_string();
        let schema = format!("lab_{id}");
        // Tên do ta sinh (hex): an toàn để ghép vào SQL.
        admin_pool
            .execute(AssertSqlSafe(format!("CREATE SCHEMA {schema}")))
            .await?;
        let pool = build_pool(&pg_url, &schema).await?;
        sqlx::raw_sql(SCHEMA_SQL).execute(&pool).await?;

        let topic_in = format!("lab.39.{id}.trip.completed");
        let topic_out = format!("lab.39.{id}.ledger.entry-posted");
        let admin = admin_client(&kafka)?;
        let topics = [
            NewTopic::new(&topic_in, 1, TopicReplication::Fixed(1)),
            NewTopic::new(&topic_out, 1, TopicReplication::Fixed(1)),
        ];
        let options = AdminOptions::new().operation_timeout(Some(Duration::from_secs(10)));
        let created = async {
            for result in admin
                .create_topics(&topics, &options)
                .await
                .context("không tạo được topic (Kafka đã bật chưa?)")?
            {
                if let Err((name, code)) = result {
                    bail!("tạo topic {name} lỗi: {code}");
                }
            }
            wait_topic_ready(&admin, &topic_in).await?;
            wait_topic_ready(&admin, &topic_out).await
        }
        .await;
        if let Err(err) = created {
            // Không để sót schema khi Kafka hỏng giữa chừng.
            pool.close().await;
            let _ = admin_pool
                .execute(AssertSqlSafe(format!("DROP SCHEMA {schema} CASCADE")))
                .await;
            return Err(err);
        }
        Ok(Lab {
            pg_url,
            kafka,
            schema,
            topic_in,
            topic_out,
            group: format!("lab-39-{id}"),
            admin_pool,
            pool,
        })
    }

    /// Cấu hình mặc định của ca (test và scenario chỉnh tiếp từng trường).
    pub fn config(&self) -> Config {
        Config {
            pg_url: self.pg_url.clone(),
            schema: self.schema.clone(),
            kafka: self.kafka.clone(),
            topic_in: self.topic_in.clone(),
            topic_out: self.topic_out.clone(),
            group: self.group.clone(),
            queue_capacity: 32,
            resume_depth: 24,
            work_delay: Duration::ZERO,
            readiness_grace: Duration::from_millis(200),
            drain_timeout: Duration::from_secs(5),
            commit_interval_ms: 200,
            session_timeout_ms: 6000,
            max_poll_interval_ms: 300_000,
        }
    }

    /// Gửi `TripCompleted` cho chuyến `from..=to` (khoá partition = `tripId`).
    ///
    /// # Errors
    /// Lỗi gửi Kafka.
    pub async fn produce_trips(&self, from: u32, to: u32) -> Result<()> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &self.kafka)
            .set("message.timeout.ms", "10000")
            .create()?;
        for n in from..=to {
            let event = TripCompleted::sample(n);
            let payload = serde_json::to_string(&event)?;
            producer
                .send(
                    FutureRecord::to(&self.topic_in)
                        .key(&event.trip_id)
                        .payload(&payload),
                    Duration::from_secs(5),
                )
                .await
                .map_err(|(err, _)| err)?;
        }
        Ok(())
    }

    /// Đếm trong DB.
    ///
    /// # Errors
    /// Lỗi SQL.
    pub async fn counts(&self) -> Result<Counts> {
        let row: (i64, i64, i64, i64, i64, i64, i64) = sqlx::query_as(
            "SELECT \
               (SELECT count(*) FROM journal_entries), \
               (SELECT count(DISTINCT idempotency_key) FROM journal_entries), \
               (SELECT count(*) FROM inbox), \
               (SELECT count(*) FROM outbox), \
               (SELECT count(*) FROM outbox WHERE published_at IS NULL), \
               (SELECT COALESCE(sum(amount_dong), 0)::bigint FROM postings), \
               (SELECT count(*) FROM account_balances b \
                  JOIN (SELECT account_id, sum(amount_dong) AS s FROM postings GROUP BY 1) p \
                    ON p.account_id = b.account_id WHERE b.balance_dong <> p.s)",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(Counts {
            entries: row.0,
            distinct_keys: row.1,
            inbox: row.2,
            outbox_total: row.3,
            outbox_pending: row.4,
            postings_sum: row.5,
            balance_mismatch: row.6,
        })
    }

    /// Chờ đến khi số bút toán đạt `n`.
    ///
    /// # Errors
    /// Hết `timeout` mà chưa đủ.
    pub async fn wait_entries(&self, n: i64, timeout: Duration) -> Result<()> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let have = self.counts().await?.entries;
            if have >= n {
                return Ok(());
            }
            if tokio::time::Instant::now() > deadline {
                bail!("chờ {n} bút toán nhưng mới có {have} sau {timeout:?}");
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    /// Offset đã commit của group trên `topic_in` (partition 0); 0 nếu chưa commit.
    ///
    /// # Errors
    /// Lỗi Kafka.
    pub async fn committed_offset(&self) -> Result<i64> {
        let (kafka, group, topic) = (
            self.kafka.clone(),
            self.group.clone(),
            self.topic_in.clone(),
        );
        tokio::task::spawn_blocking(move || -> Result<i64> {
            let consumer: BaseConsumer = ClientConfig::new()
                .set("bootstrap.servers", &kafka)
                .set("group.id", &group)
                .create()?;
            let mut tpl = TopicPartitionList::new();
            tpl.add_partition(&topic, 0);
            let committed = consumer.committed_offsets(tpl, Duration::from_secs(5))?;
            let offset = committed.find_partition(&topic, 0).map(|e| e.offset());
            Ok(match offset {
                Some(Offset::Offset(n)) => n,
                _ => 0,
            })
        })
        .await?
    }

    /// Mọi key đã nằm trên `topic_out`, theo thứ tự (đọc bằng group riêng, không commit).
    ///
    /// # Errors
    /// Lỗi Kafka.
    pub async fn out_keys(&self) -> Result<Vec<String>> {
        let (kafka, topic) = (self.kafka.clone(), self.topic_out.clone());
        tokio::task::spawn_blocking(move || -> Result<Vec<String>> {
            let consumer: BaseConsumer = ClientConfig::new()
                .set("bootstrap.servers", &kafka)
                .set("group.id", format!("reader-{}", Uuid::new_v4().simple()))
                .set("enable.auto.commit", "false")
                .create()?;
            let (low, high) = consumer.fetch_watermarks(&topic, 0, Duration::from_secs(5))?;
            let mut tpl = TopicPartitionList::new();
            tpl.add_partition_offset(&topic, 0, Offset::Offset(low))?;
            consumer.assign(&tpl)?;
            let mut keys = Vec::new();
            let deadline = std::time::Instant::now() + Duration::from_secs(10);
            while (keys.len() as i64) < high - low {
                if std::time::Instant::now() > deadline {
                    bail!(
                        "đọc topic_out quá thời gian: {} / {}",
                        keys.len(),
                        high - low
                    );
                }
                if let Some(msg) = consumer.poll(Duration::from_millis(200)) {
                    let msg = msg?;
                    keys.push(String::from_utf8_lossy(msg.key().unwrap_or_default()).to_string());
                }
            }
            Ok(keys)
        })
        .await?
    }

    /// Xoá schema và hai topic của ca. Gọi cả khi kịch bản lỗi.
    ///
    /// # Errors
    /// Lỗi xoá schema hoặc topic.
    pub async fn cleanup(self) -> Result<()> {
        self.pool.close().await;
        self.admin_pool
            .execute(AssertSqlSafe(format!(
                "DROP SCHEMA {} CASCADE",
                self.schema
            )))
            .await?;
        self.admin_pool.close().await;
        let admin = admin_client(&self.kafka)?;
        let names = [self.topic_in.as_str(), self.topic_out.as_str()];
        let options = AdminOptions::new().operation_timeout(Some(Duration::from_secs(10)));
        admin.delete_topics(&names, &options).await?;
        Ok(())
    }
}
