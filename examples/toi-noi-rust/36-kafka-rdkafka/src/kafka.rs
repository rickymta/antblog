//! Hạ tầng Kafka của lab: kết nối từ `TOI_NOI_KAFKA`, topic/group riêng mỗi ca, dọn cuối ca,
//! và các hàm chờ có hạn (không bao giờ treo).

use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, ConsumerContext, StreamConsumer};
use rdkafka::error::KafkaError;
use rdkafka::message::BorrowedMessage;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::{Offset, TopicPartitionList};
use thiserror::Error;
use uuid::Uuid;

/// Thời gian tối đa chờ một thông điệp «chắc chắn sẽ đến» (đủ cho consumer vào group lần đầu).
pub const WAIT: Duration = Duration::from_secs(30);
/// Thời gian chờ khi kỳ vọng KHÔNG có thông điệp nào (kiểm chứng sự vắng mặt).
pub const QUIET: Duration = Duration::from_millis(2_000);

/// Lỗi riêng của lab. `Timeout` tách khỏi lỗi Kafka để test phân biệt được «không có gì đến».
#[derive(Debug, Error)]
pub enum LabError {
    /// Hết hạn chờ; `what` nói đang chờ gì.
    #[error("hết {wait:?} chờ {what}")]
    Timeout { what: String, wait: Duration },
    /// Lỗi từ librdkafka.
    #[error("lỗi Kafka: {0}")]
    Kafka(#[from] KafkaError),
}

/// Một ca lab: topic tên `lab.36.<uuid>.<tên>`, group `lab-36-<uuid>`; `finish` xoá đúng những thứ đã tạo.
pub struct Lab {
    pub id: String,
    bootstrap: String,
    admin: AdminClient<DefaultClientContext>,
    topics: Mutex<Vec<String>>,
    groups: Mutex<Vec<String>>,
}

impl Lab {
    /// Đọc `TOI_NOI_KAFKA` và kiểm tra nối được tới broker. Thiếu biến hoặc không nối được là lỗi rõ.
    pub async fn start() -> Result<Lab> {
        Lab::connect(std::env::var("TOI_NOI_KAFKA").ok().as_deref()).await
    }

    /// Như `start` nhưng nhận địa chỉ trực tiếp (`None` = thiếu biến môi trường); để test đường lỗi.
    pub async fn connect(bootstrap: Option<&str>) -> Result<Lab> {
        let bootstrap = bootstrap
            .context(
                "thiếu biến môi trường TOI_NOI_KAFKA (ví dụ 127.0.0.1:9092); xem infra/README.md",
            )?
            .to_string();
        // fetch_metadata chặn thread nên chạy trong spawn_blocking; quá 5 giây là lỗi, không treo.
        let servers = bootstrap.clone();
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

    /// Tên topic của ca: `lab.36.<uuid>.<tên>`.
    pub fn topic_name(&self, name: &str) -> String {
        format!("lab.36.{}.{name}", self.id)
    }

    /// Group chính của ca: `lab-36-<uuid>`; `suffix` tách các group của từng kịch bản.
    pub fn group_name(&self, suffix: Option<&str>) -> String {
        match suffix {
            Some(s) => format!("lab-36-{}-{s}", self.id),
            None => format!("lab-36-{}", self.id),
        }
    }

    /// Tạo topic bằng `AdminClient` và nhớ tên để xoá cuối ca.
    pub async fn create_topic(&self, name: &str, partitions: i32) -> Result<String> {
        let full = self.topic_name(name);
        let topic = NewTopic::new(&full, partitions, TopicReplication::Fixed(1));
        let opts = AdminOptions::new().operation_timeout(Some(Duration::from_secs(15)));
        let results = self.admin.create_topics([&topic], &opts).await?;
        for result in results {
            if let Err((topic, code)) = result {
                bail!("tạo topic {topic} thất bại: {code}");
            }
        }
        self.topics.lock().unwrap().push(full.clone());
        self.wait_topic_ready(&full, partitions).await?;
        Ok(full)
    }

    // Topic vừa tạo chưa chắc đã có leader. Consumer subscribe sớm quá sẽ nhận
    // `UnknownTopicOrPartition` từ `recv()`, nên chờ metadata đủ partition rồi mới trả về.
    async fn wait_topic_ready(&self, topic: &str, partitions: i32) -> Result<()> {
        let start = Instant::now();
        loop {
            // fetch_metadata chặn thread; lab và test đều chạy trên runtime nhiều thread.
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

    fn consumer_config(&self, group: &str) -> ClientConfig {
        self.groups.lock().unwrap().push(group.to_string());
        let mut config = ClientConfig::new();
        config
            .set("bootstrap.servers", &self.bootstrap)
            .set("group.id", group)
            // Commit thủ công: offset chỉ tiến khi code của ta nói «đã ghi bút toán xong».
            .set("enable.auto.commit", "false")
            // Group mới chưa có offset: đọc từ đầu topic thay vì bỏ lỡ thông điệp đã gửi.
            .set("auto.offset.reset", "earliest")
            .set("session.timeout.ms", "10000");
        config
    }

    /// Consumer mặc định (không cần callback).
    pub fn consumer(&self, group: &str) -> Result<StreamConsumer, KafkaError> {
        self.consumer_config(group).create()
    }

    /// Consumer có context riêng (ví dụ để ghi lại rebalance).
    pub fn consumer_with<C: ConsumerContext + 'static>(
        &self,
        group: &str,
        context: C,
    ) -> Result<StreamConsumer<C>, KafkaError> {
        self.consumer_config(group).create_with_context(context)
    }

    /// Producer idempotent: `enable.idempotence` bật `acks=all` và chống trùng khi chính librdkafka gửi lại.
    pub fn producer(&self) -> Result<FutureProducer, KafkaError> {
        ClientConfig::new()
            .set("bootstrap.servers", &self.bootstrap)
            .set("enable.idempotence", "true")
            .set("message.timeout.ms", "15000")
            .create()
    }

    /// Producer dùng thuật toán chọn partition khác mặc định (`consistent_random`, tức CRC32).
    pub fn producer_with_partitioner(
        &self,
        partitioner: &str,
    ) -> Result<FutureProducer, KafkaError> {
        ClientConfig::new()
            .set("bootstrap.servers", &self.bootstrap)
            .set("partitioner", partitioner)
            .set("message.timeout.ms", "15000")
            .create()
    }

    /// Xoá topic và group của ca. Chạy kể cả khi kịch bản lỗi; lỗi dọn cũng được báo.
    pub async fn finish(self) -> Result<()> {
        let topics = self.topics.lock().unwrap().clone();
        let groups = self.groups.lock().unwrap().clone();
        let opts = AdminOptions::new().operation_timeout(Some(Duration::from_secs(15)));
        let names: Vec<&str> = topics.iter().map(String::as_str).collect();
        if !names.is_empty() {
            for result in self.admin.delete_topics(&names, &opts).await? {
                if let Err((topic, code)) = result {
                    bail!("xoá topic {topic} thất bại: {code}");
                }
            }
        }
        // Group chỉ xoá được khi đã rỗng; consumer vừa đóng có thể chưa kịp rời group, nên thử lại vài lần.
        let mut pending: Vec<String> = groups;
        pending.sort();
        pending.dedup();
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
        // Group chưa từng nhận thành viên (ví dụ kịch bản hỏng sớm) có thể không tồn tại: báo nhưng không che lỗi khác.
        eprintln!("CẢNH BÁO: chưa xoá được group {pending:?}");
        Ok(())
    }
}

/// Gửi một thông điệp và chờ broker xác nhận; trả `(partition, offset)` thật.
pub async fn send(
    producer: &FutureProducer,
    topic: &str,
    key: &str,
    payload: &[u8],
) -> Result<(i32, i64), LabError> {
    let record = FutureRecord::to(topic).key(key).payload(payload);
    producer
        .send(record, Duration::from_secs(10))
        .await
        .map(|delivery| (delivery.partition, delivery.offset))
        .map_err(|(err, _)| LabError::Kafka(err))
}

/// Chờ thông điệp tiếp theo, tối đa `wait`. Hết giờ là `LabError::Timeout`, không treo.
pub async fn recv_within<'a, C: ConsumerContext + 'static>(
    consumer: &'a StreamConsumer<C>,
    wait: Duration,
    what: &str,
) -> Result<BorrowedMessage<'a>, LabError> {
    match tokio::time::timeout(wait, consumer.recv()).await {
        Err(_) => Err(LabError::Timeout {
            what: what.to_string(),
            wait,
        }),
        Ok(received) => Ok(received?),
    }
}

/// Offset đã commit của group cho một partition (`None` nếu chưa commit lần nào).
/// Đó là offset của thông điệp SẼ đọc tiếp, tức offset thông điệp đã xử lý cộng 1.
pub fn committed_offset<C: ConsumerContext + 'static>(
    consumer: &StreamConsumer<C>,
    topic: &str,
    partition: i32,
) -> Result<Option<i64>, KafkaError> {
    let mut tpl = TopicPartitionList::new();
    tpl.add_partition(topic, partition);
    let committed = consumer.committed_offsets(tpl, Duration::from_secs(10))?;
    Ok(committed
        .find_partition(topic, partition)
        .and_then(|e| match e.offset() {
            Offset::Offset(n) => Some(n),
            _ => None,
        }))
}
