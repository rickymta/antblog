//! Hạ tầng Kafka của lab: topic/group riêng mỗi ca, dọn cuối ca, mọi lần chờ đều có hạn.
//! Rút gọn từ `36-kafka-rdkafka/src/kafka.rs` (không path dependency sang lab khác).

use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::StreamConsumer;
use rdkafka::error::KafkaError;
use rdkafka::message::BorrowedMessage;
use rdkafka::producer::FutureProducer;
use uuid::Uuid;

/// Thời gian tối đa chờ một thông điệp «chắc chắn sẽ đến».
pub const WAIT: Duration = Duration::from_secs(30);

/// Một ca lab: topic `lab.66.<uuid>.<tên>`, group `lab-66-<uuid>`; `finish` xoá đúng những thứ đã tạo.
pub struct Lab {
    pub id: String,
    bootstrap: String,
    admin: AdminClient<DefaultClientContext>,
    topics: Mutex<Vec<String>>,
    groups: Mutex<Vec<String>>,
}

impl Lab {
    /// Đọc `TOI_NOI_KAFKA` và kiểm tra nối được tới broker; thiếu biến hoặc không nối được là lỗi rõ.
    pub async fn start() -> Result<Lab> {
        let bootstrap = std::env::var("TOI_NOI_KAFKA").context(
            "thiếu biến môi trường TOI_NOI_KAFKA (ví dụ 127.0.0.1:9092); xem infra/README.md",
        )?;
        let servers = bootstrap.clone();
        // fetch_metadata chặn thread nên chạy trong spawn_blocking; quá 5 giây là lỗi, không treo.
        let admin = tokio::task::spawn_blocking(move || -> Result<_, KafkaError> {
            let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
                .set("bootstrap.servers", &servers)
                .create()?;
            admin.inner().fetch_metadata(None, Duration::from_secs(5))?;
            Ok(admin)
        })
        .await?
        .with_context(|| format!("không kết nối được Kafka tại {bootstrap}"))?;
        Ok(Lab {
            id: Uuid::new_v4().simple().to_string(),
            bootstrap,
            admin,
            topics: Mutex::new(Vec::new()),
            groups: Mutex::new(Vec::new()),
        })
    }

    /// Group của ca: `lab-66-<uuid>`; `suffix` tách các group của từng kịch bản.
    pub fn group_name(&self, suffix: &str) -> String {
        format!("lab-66-{}-{suffix}", self.id)
    }

    /// Tạo topic bằng `AdminClient`, nhớ tên để xoá cuối ca, chờ đủ leader.
    pub async fn create_topic(&self, name: &str, partitions: i32) -> Result<String> {
        let full = format!("lab.66.{}.{name}", self.id);
        let topic = NewTopic::new(&full, partitions, TopicReplication::Fixed(1));
        let opts = AdminOptions::new().operation_timeout(Some(Duration::from_secs(15)));
        for result in self.admin.create_topics([&topic], &opts).await? {
            if let Err((topic, code)) = result {
                bail!("tạo topic {topic} thất bại: {code}");
            }
        }
        self.topics.lock().unwrap().push(full.clone());
        self.wait_topic_ready(&full, partitions).await?;
        Ok(full)
    }

    // Topic vừa tạo chưa chắc đã có leader; subscribe sớm quá sẽ nhận `UnknownTopicOrPartition`.
    async fn wait_topic_ready(&self, topic: &str, partitions: i32) -> Result<()> {
        let start = Instant::now();
        loop {
            let metadata = tokio::task::block_in_place(|| {
                self.admin
                    .inner()
                    .fetch_metadata(Some(topic), Duration::from_secs(5))
            })?;
            let ready = metadata.topics().iter().any(|t| {
                t.name() == topic
                    && t.error().is_none()
                    && t.partitions().len() == partitions as usize
                    && t.partitions()
                        .iter()
                        .all(|p| p.error().is_none() && p.leader() >= 0)
            });
            if ready {
                return Ok(());
            }
            if start.elapsed() >= WAIT {
                bail!("topic {topic} chưa sẵn sàng sau {WAIT:?}");
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Consumer commit thủ công, group mới đọc từ đầu topic.
    pub fn consumer(&self, group: &str) -> Result<StreamConsumer, KafkaError> {
        self.groups.lock().unwrap().push(group.to_string());
        ClientConfig::new()
            .set("bootstrap.servers", &self.bootstrap)
            .set("group.id", group)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .set("session.timeout.ms", "10000")
            .create()
    }

    /// Producer idempotent.
    pub fn producer(&self) -> Result<FutureProducer, KafkaError> {
        ClientConfig::new()
            .set("bootstrap.servers", &self.bootstrap)
            .set("enable.idempotence", "true")
            .set("message.timeout.ms", "15000")
            .create()
    }

    /// Xoá topic và group của ca. Nhóm có thể chưa rỗng ngay sau khi consumer đóng, nên thử lại vài lần.
    pub async fn finish(self) -> Result<()> {
        let topics = self.topics.lock().unwrap().clone();
        let mut pending = self.groups.lock().unwrap().clone();
        pending.sort();
        pending.dedup();
        let opts = AdminOptions::new().operation_timeout(Some(Duration::from_secs(15)));
        let names: Vec<&str> = topics.iter().map(String::as_str).collect();
        if !names.is_empty() {
            for result in self.admin.delete_topics(&names, &opts).await? {
                if let Err((topic, code)) = result {
                    bail!("xoá topic {topic} thất bại: {code}");
                }
            }
        }
        for attempt in 0..10 {
            let names: Vec<&str> = pending.iter().map(String::as_str).collect();
            let results = self.admin.delete_groups(&names, &opts).await?;
            pending = results
                .into_iter()
                .filter_map(|r| r.err().map(|(group, _)| group))
                .collect();
            if pending.is_empty() {
                return Ok(());
            }
            if attempt < 9 {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
        eprintln!("CẢNH BÁO: chưa xoá được group {pending:?}");
        Ok(())
    }
}

/// Chờ thông điệp tiếp theo, tối đa `wait`; hết giờ là lỗi rõ, không treo.
pub async fn recv_within<'a>(
    consumer: &'a StreamConsumer,
    wait: Duration,
    what: &str,
) -> Result<BorrowedMessage<'a>> {
    match tokio::time::timeout(wait, consumer.recv()).await {
        Err(_) => bail!("hết {wait:?} chờ {what}"),
        Ok(received) => Ok(received?),
    }
}
