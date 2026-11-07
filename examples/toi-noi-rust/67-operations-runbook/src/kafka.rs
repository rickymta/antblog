//! Tiện ích Kafka của lab: tạo/xoá topic, producer, đếm, đọc lại, offset đã commit và consumer lag.
//! Mọi hàm đọc metadata chạy trong `spawn_blocking` vì librdkafka chặn thread.

use std::time::Duration;

use anyhow::{Context, Result, bail};
use rdkafka::ClientConfig;
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::error::RDKafkaErrorCode;
use rdkafka::message::{Headers, Message};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::topic_partition_list::{Offset, TopicPartitionList};
use rdkafka::util::Timeout;

/// Producer an toàn cho sổ cái: `acks=all`, idempotent.
///
/// # Errors
/// Cấu hình client lỗi.
pub fn producer(brokers: &str) -> Result<FutureProducer> {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("acks", "all")
        .set("enable.idempotence", "true")
        .set("message.timeout.ms", "10000")
        .create()
        .context("tạo producer")
}

fn admin(brokers: &str) -> Result<AdminClient<DefaultClientContext>> {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .create()
        .context("tạo admin client")
}

/// Tạo topic một partition rồi chờ tới khi có leader (có hạn chót, không ngủ cố định).
///
/// # Errors
/// Broker từ chối, hoặc topic chưa có leader sau 30 giây.
pub async fn create_topic(brokers: &str, name: &str) -> Result<()> {
    let topic = NewTopic::new(name, 1, TopicReplication::Fixed(1));
    let results = admin(brokers)?
        .create_topics(
            [&topic],
            &AdminOptions::new().request_timeout(Some(Duration::from_secs(10))),
        )
        .await
        .context("create_topics")?;
    for result in results {
        if let Err((topic, code)) = result {
            bail!("không tạo được topic {topic}: {code}");
        }
    }
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let (brokers, name_owned) = (brokers.to_string(), name.to_string());
        let ready = tokio::task::spawn_blocking(move || -> bool {
            let Ok(client) = admin(&brokers) else {
                return false;
            };
            client
                .inner()
                .fetch_metadata(Some(&name_owned), Duration::from_secs(5))
                .is_ok_and(|m| {
                    m.topics().iter().any(|t| {
                        t.name() == name_owned
                            && t.error().is_none()
                            && !t.partitions().is_empty()
                            && t.partitions()
                                .iter()
                                .all(|p| p.error().is_none() && p.leader() >= 0)
                    })
                })
        })
        .await?;
        if ready {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            bail!("topic {name} chưa có leader sau 30 giây");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Xoá các topic của ca.
///
/// # Errors
/// Không kết nối được broker.
pub async fn delete_topics(brokers: &str, names: &[String]) -> Result<()> {
    if names.is_empty() {
        return Ok(());
    }
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    admin(brokers)?
        .delete_topics(
            &refs,
            &AdminOptions::new().request_timeout(Some(Duration::from_secs(10))),
        )
        .await
        .context("delete_topics")?;
    Ok(())
}

/// Xoá các consumer group của ca (group chỉ xoá được khi đã rỗng nên thử lại vài lần).
///
/// # Errors
/// Không kết nối được broker.
pub async fn delete_groups(brokers: &str, names: &[String]) -> Result<()> {
    if names.is_empty() {
        return Ok(());
    }
    let admin = admin(brokers)?;
    let opts = AdminOptions::new().request_timeout(Some(Duration::from_secs(10)));
    let mut pending: Vec<String> = names.to_vec();
    for attempt in 0..10 {
        let refs: Vec<&str> = pending.iter().map(String::as_str).collect();
        let results = admin
            .delete_groups(&refs, &opts)
            .await
            .context("delete_groups")?;
        pending = results
            .into_iter()
            .filter_map(|result| match result {
                Err((group, code)) if code != RDKafkaErrorCode::GroupIdNotFound => Some(group),
                _ => None,
            })
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

/// Gửi một message và chờ broker xác nhận.
///
/// # Errors
/// Broker không xác nhận.
pub async fn send(producer: &FutureProducer, topic: &str, key: &str, payload: &[u8]) -> Result<()> {
    producer
        .send(
            FutureRecord::to(topic).key(key).payload(payload),
            Timeout::After(Duration::from_secs(10)),
        )
        .await
        .map_err(|(err, _)| err)
        .with_context(|| format!("gửi tới {topic}"))?;
    Ok(())
}

/// Hai mốc của partition 0: `(low, high)`; `high` là offset của message kế tiếp sẽ được ghi.
///
/// # Errors
/// Lỗi gọi broker.
pub async fn watermarks(brokers: &str, topic: &str) -> Result<(i64, i64)> {
    let (brokers, topic) = (brokers.to_string(), topic.to_string());
    tokio::task::spawn_blocking(move || -> Result<(i64, i64)> {
        let consumer: BaseConsumer = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .set("group.id", "lab-67-watermarks")
            .set("enable.auto.commit", "false")
            .create()?;
        Ok(consumer.fetch_watermarks(&topic, 0, Duration::from_secs(10))?)
    })
    .await?
}

/// Số message đang có trong topic (high − low của partition 0).
///
/// # Errors
/// Lỗi gọi broker.
pub async fn count(brokers: &str, topic: &str) -> Result<i64> {
    let (low, high) = watermarks(brokers, topic).await?;
    Ok(high - low)
}

/// Offset đã commit của group trên partition 0; `None` khi group chưa commit gì.
///
/// # Errors
/// Lỗi gọi broker.
pub async fn committed_offset(brokers: &str, group: &str, topic: &str) -> Result<Option<i64>> {
    let (brokers, group, topic) = (brokers.to_string(), group.to_string(), topic.to_string());
    tokio::task::spawn_blocking(move || -> Result<Option<i64>> {
        let consumer: BaseConsumer = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .set("group.id", &group)
            .create()?;
        let mut tpl = TopicPartitionList::new();
        tpl.add_partition(&topic, 0);
        let committed = consumer.committed_offsets(tpl, Duration::from_secs(10))?;
        Ok(
            match committed.find_partition(&topic, 0).map(|e| e.offset()) {
                Some(Offset::Offset(n)) => Some(n),
                _ => None,
            },
        )
    })
    .await?
}

/// Consumer lag của group: `high watermark − offset đã commit`. Group chưa commit gì thì đếm từ
/// `low` (các lab đặt `auto.offset.reset=earliest`).
///
/// # Errors
/// Lỗi gọi broker.
pub async fn consumer_lag(brokers: &str, group: &str, topic: &str) -> Result<i64> {
    let (low, high) = watermarks(brokers, topic).await?;
    let committed = committed_offset(brokers, group, topic).await?;
    Ok(high - committed.unwrap_or(low).max(low))
}

/// Một message đọc lại để kiểm chứng.
#[derive(Debug, Clone)]
pub struct Received {
    /// Offset trong partition 0.
    pub offset: i64,
    /// Key.
    pub key: Option<String>,
    /// Header dạng chuỗi.
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

/// Đọc lại toàn bộ message của partition 0, không dùng group (gán thẳng từ đầu).
///
/// # Errors
/// Lỗi broker, hoặc quá hạn chờ.
pub async fn read_all(brokers: &str, topic: &str) -> Result<Vec<Received>> {
    let (low, high) = watermarks(brokers, topic).await?;
    let expected = high - low;
    let (brokers, topic) = (brokers.to_string(), topic.to_string());
    tokio::task::spawn_blocking(move || -> Result<Vec<Received>> {
        let mut out = Vec::new();
        if expected == 0 {
            return Ok(out);
        }
        let consumer: BaseConsumer = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .set("group.id", "lab-67-reader")
            // Chỉ đọc bằng assign, không commit: nếu không tắt thì group còn sót trên broker.
            .set("enable.auto.commit", "false")
            .create()?;
        let mut tpl = TopicPartitionList::new();
        tpl.add_partition_offset(&topic, 0, Offset::Beginning)?;
        consumer.assign(&tpl)?;
        let deadline = std::time::Instant::now() + Duration::from_secs(60);
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
                    bail!("quá hạn: đọc {} trên {expected} message", out.len())
                }
                None => {}
            }
        }
        Ok(out)
    })
    .await?
}
