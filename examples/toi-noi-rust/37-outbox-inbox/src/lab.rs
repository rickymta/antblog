//! Môi trường một ca thử: schema PostgreSQL `lab_<uuid>` và topic Kafka `lab.37.<uuid>.<tên>`.
//! Thiếu biến môi trường hoặc không kết nối được thì FAIL rõ, không skip.

use std::future::Future;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use rdkafka::ClientConfig;
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::message::{Headers, Message};
use rdkafka::producer::FutureProducer;
use rdkafka::topic_partition_list::{Offset, TopicPartitionList};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::{AssertSqlSafe, Executor};
use uuid::Uuid;

/// Schema chuẩn của loạt cộng `outbox`, `inbox` (xem `sql/schema.sql`).
const SCHEMA_SQL: &str = include_str!("../sql/schema.sql");

/// Đọc biến môi trường bắt buộc, báo cách bật hạ tầng nếu thiếu.
fn require_env(name: &str) -> Result<String> {
    std::env::var(name).map_err(|_| {
        anyhow::anyhow!(
            "thiếu {name}. Bật hạ tầng: docker compose -f \
             examples/toi-noi-rust/infra/docker-compose.yml up -d --wait, \
             rồi export theo examples/toi-noi-rust/infra/README.md"
        )
    })
}

/// Pool trỏ vào đúng một schema: `search_path` đặt ngay trong tham số khởi động kết nối.
async fn connect_in_schema(url: &str, schema: &str, max: u32) -> Result<PgPool> {
    let options = PgConnectOptions::from_str(url)?.options([("search_path", schema)]);
    PgPoolOptions::new()
        .max_connections(max)
        .connect_with(options)
        .await
        .context("không kết nối được PostgreSQL ở TOI_NOI_PG_URL (compose đã bật chưa?)")
}

/// Schema PostgreSQL riêng của một ca.
pub struct PgLab {
    /// Pool của ca (mọi kết nối đã trỏ vào schema của ca).
    pub pool: PgPool,
    /// URL kết nối, để tiến trình con nối vào cùng schema.
    pub url: String,
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
        let url = require_env("TOI_NOI_PG_URL")?;
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
        let pool = connect_in_schema(&url, &schema, 16).await?;
        sqlx::raw_sql(SCHEMA_SQL).execute(&pool).await?;
        Ok(PgLab {
            pool,
            url,
            schema,
            admin,
        })
    }

    /// Nối vào schema đã có (dùng cho tiến trình con); không sở hữu nên không dọn.
    ///
    /// # Errors
    /// Không kết nối được.
    pub async fn attach(url: &str, schema: &str) -> Result<PgPool> {
        connect_in_schema(url, schema, 2).await
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

/// Một thông điệp đọc lại từ topic để kiểm chứng.
#[derive(Debug, Clone)]
pub struct Received {
    /// Partition.
    pub partition: i32,
    /// Offset trong partition.
    pub offset: i64,
    /// Key (nếu có).
    pub key: Option<String>,
    /// Mọi header dạng chuỗi.
    pub headers: Vec<(String, String)>,
    /// Payload dạng chuỗi.
    pub payload: String,
}

impl Received {
    /// Giá trị của một header.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

/// Topic và consumer group riêng của một ca.
pub struct KafkaLab {
    /// Địa chỉ broker.
    pub brokers: String,
    /// Mã ca (uuid không gạch).
    pub id: String,
    admin: AdminClient<DefaultClientContext>,
    created: Mutex<Vec<String>>,
}

impl KafkaLab {
    /// Đọc `TOI_NOI_KAFKA` và dựng admin client.
    ///
    /// # Errors
    /// Thiếu biến môi trường hoặc dựng client lỗi.
    pub fn new() -> Result<KafkaLab> {
        let brokers = require_env("TOI_NOI_KAFKA")?;
        let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .create()?;
        Ok(KafkaLab {
            brokers,
            id: Uuid::new_v4().simple().to_string(),
            admin,
            created: Mutex::new(Vec::new()),
        })
    }

    /// Tên đầy đủ `lab.37.<uuid>.<name>`.
    pub fn topic_name(&self, name: &str) -> String {
        format!("lab.37.{}.{name}", self.id)
    }

    /// Consumer group `lab-37-<uuid>` (kèm hậu tố nếu một ca cần nhiều group).
    pub fn group(&self, suffix: &str) -> String {
        if suffix.is_empty() {
            format!("lab-37-{}", self.id)
        } else {
            format!("lab-37-{}-{suffix}", self.id)
        }
    }

    /// Tạo topic và nhớ để xoá cuối ca.
    ///
    /// # Errors
    /// Broker từ chối tạo topic.
    pub async fn create_topic(&self, name: &str, partitions: i32) -> Result<String> {
        let full = self.topic_name(name);
        let new_topic = NewTopic::new(&full, partitions, TopicReplication::Fixed(1));
        let results = self
            .admin
            .create_topics([&new_topic], &AdminOptions::new())
            .await?;
        for result in results {
            if let Err((topic, code)) = result {
                bail!("không tạo được topic {topic}: {code}");
            }
        }
        self.created.lock().expect("mutex").push(full.clone());
        Ok(full)
    }

    /// Producer đồng bộ xác nhận: `acks=all`, idempotent.
    ///
    /// # Errors
    /// Dựng client lỗi.
    pub fn producer(&self) -> Result<FutureProducer> {
        Ok(ClientConfig::new()
            .set("bootstrap.servers", &self.brokers)
            .set("acks", "all")
            .set("enable.idempotence", "true")
            .set("message.timeout.ms", "10000")
            .create()?)
    }

    /// Số thông điệp đang có trong topic: tổng (high − low watermark) của mọi partition.
    ///
    /// # Errors
    /// Lỗi metadata hoặc watermark.
    pub async fn count(&self, topic: &str) -> Result<i64> {
        let (brokers, group, topic) =
            (self.brokers.clone(), self.group("count"), topic.to_string());
        tokio::task::spawn_blocking(move || -> Result<i64> {
            let consumer: BaseConsumer = ClientConfig::new()
                .set("bootstrap.servers", &brokers)
                .set("group.id", &group)
                .create()?;
            let metadata = consumer.fetch_metadata(Some(&topic), Duration::from_secs(10))?;
            let mut total = 0;
            for t in metadata.topics() {
                for p in t.partitions() {
                    let (low, high) =
                        consumer.fetch_watermarks(&topic, p.id(), Duration::from_secs(10))?;
                    total += high - low;
                }
            }
            Ok(total)
        })
        .await?
    }

    /// Đọc lại toàn bộ thông điệp đang có trong topic (gán partition từ đầu, không dùng group).
    ///
    /// # Errors
    /// Lỗi metadata, nhận thông điệp, hoặc quá hạn chờ.
    pub async fn read_all(&self, topic: &str) -> Result<Vec<Received>> {
        let (brokers, group, topic) = (
            self.brokers.clone(),
            self.group("reader"),
            topic.to_string(),
        );
        tokio::task::spawn_blocking(move || -> Result<Vec<Received>> {
            let consumer: BaseConsumer = ClientConfig::new()
                .set("bootstrap.servers", &brokers)
                .set("group.id", &group)
                .create()?;
            let metadata = consumer.fetch_metadata(Some(&topic), Duration::from_secs(10))?;
            let mut assignment = TopicPartitionList::new();
            let mut expected = 0;
            for t in metadata.topics() {
                for p in t.partitions() {
                    let (low, high) =
                        consumer.fetch_watermarks(&topic, p.id(), Duration::from_secs(10))?;
                    if high > low {
                        assignment.add_partition_offset(&topic, p.id(), Offset::Beginning)?;
                        expected += high - low;
                    }
                }
            }
            let mut out = Vec::new();
            if expected == 0 {
                return Ok(out);
            }
            consumer.assign(&assignment)?;
            let deadline = std::time::Instant::now() + Duration::from_secs(30);
            while (out.len() as i64) < expected {
                match consumer.poll(Duration::from_millis(500)) {
                    Some(Ok(message)) => {
                        let mut headers = Vec::new();
                        if let Some(h) = message.headers() {
                            for i in 0..h.count() {
                                let header = h.get(i);
                                if let Some(value) = header.value {
                                    headers.push((
                                        header.key.to_string(),
                                        String::from_utf8_lossy(value).into_owned(),
                                    ));
                                }
                            }
                        }
                        out.push(Received {
                            partition: message.partition(),
                            offset: message.offset(),
                            key: message
                                .key()
                                .map(|k| String::from_utf8_lossy(k).into_owned()),
                            headers,
                            payload: String::from_utf8_lossy(message.payload().unwrap_or_default())
                                .into_owned(),
                        });
                    }
                    Some(Err(err)) => return Err(err.into()),
                    None if std::time::Instant::now() > deadline => {
                        bail!(
                            "quá hạn chờ: đọc được {} trên {expected} thông điệp",
                            out.len()
                        )
                    }
                    None => {}
                }
            }
            Ok(out)
        })
        .await?
    }

    /// Offset đã commit của group trên partition 0 của topic (-1 nếu chưa commit).
    ///
    /// # Errors
    /// Lỗi gọi broker.
    pub async fn committed_offset(&self, group: &str, topic: &str) -> Result<i64> {
        let (brokers, group, topic) = (self.brokers.clone(), group.to_string(), topic.to_string());
        tokio::task::spawn_blocking(move || -> Result<i64> {
            let consumer: BaseConsumer = ClientConfig::new()
                .set("bootstrap.servers", &brokers)
                .set("group.id", &group)
                .create()?;
            let mut tpl = TopicPartitionList::new();
            tpl.add_partition(&topic, 0);
            let committed = consumer.committed_offsets(tpl, Duration::from_secs(10))?;
            Ok(
                match committed.find_partition(&topic, 0).map(|e| e.offset()) {
                    Some(Offset::Offset(n)) => n,
                    _ => -1,
                },
            )
        })
        .await?
    }

    /// Xoá mọi topic của ca. Lỗi xoá không làm hỏng kết quả ca nên chỉ báo lên.
    ///
    /// # Errors
    /// Lỗi gọi admin.
    pub async fn cleanup(&self) -> Result<()> {
        let names: Vec<String> = self.created.lock().expect("mutex").clone();
        if names.is_empty() {
            return Ok(());
        }
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        self.admin
            .delete_topics(&refs, &AdminOptions::new())
            .await?;
        Ok(())
    }
}

/// Toàn bộ môi trường một ca.
pub struct Env {
    /// PostgreSQL.
    pub pg: PgLab,
    /// Kafka.
    pub kafka: KafkaLab,
    /// File thực thi của lab, để sinh tiến trình con tự thoát giữa hai bước.
    pub child_exe: PathBuf,
}

/// Chạy `body` trong một ca, luôn dọn schema và topic kể cả khi `body` panic.
///
/// # Errors
/// Lỗi dựng hoặc dọn ca, hoặc lỗi `body` trả về.
pub async fn with_env<F, Fut>(child_exe: PathBuf, body: F) -> Result<()>
where
    F: FnOnce(Arc<Env>) -> Fut,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    let env = Arc::new(Env {
        pg: PgLab::new().await?,
        kafka: KafkaLab::new()?,
        child_exe,
    });
    // Chạy trong task riêng: panic của body thành JoinError, ta vẫn kịp dọn.
    let joined = tokio::spawn(body(env.clone())).await;
    let pg_result = env.pg.cleanup().await;
    let kafka_result = env.kafka.cleanup().await;
    pg_result?;
    kafka_result?;
    match joined {
        Ok(result) => result,
        Err(join_err) => std::panic::resume_unwind(join_err.into_panic()),
    }
}
