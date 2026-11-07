//! Tiện ích Kafka dùng chung: producer, tạo/xoá topic, gửi JSON. Chép từ lab 40, chờ topic có leader như lab 36.

use std::time::Duration;

use anyhow::{Context, Result, bail};
use rdkafka::ClientConfig;
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::types::RDKafkaErrorCode;
use serde::Serialize;

/// Producer an toàn cho sổ cái: `acks=all` và idempotent producer (không nhân đôi khi retry nội bộ).
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

/// Tạo topic một partition (thứ tự toàn cục, đủ cho lab), không tạo ngầm.
///
/// # Errors
/// Không kết nối được broker hoặc broker từ chối.
pub async fn create_topics(brokers: &str, names: &[String]) -> Result<()> {
    let topics: Vec<NewTopic<'_>> = names
        .iter()
        .map(|n| NewTopic::new(n, 1, TopicReplication::Fixed(1)))
        .collect();
    let results = admin(brokers)?
        .create_topics(
            &topics,
            &AdminOptions::new().request_timeout(Some(Duration::from_secs(10))),
        )
        .await
        .context("create_topics")?;
    for result in results {
        if let Err((topic, code)) = result {
            bail!("không tạo được topic {topic}: {code}");
        }
    }
    wait_for_leaders(brokers, names).await
}

// `create_topics` trả về trước khi metadata kịp lan: subscribe ngay lúc đó có thể nhận
// UnknownTopicOrPartition. Chờ tới khi mọi topic có leader, có hạn chót, không ngủ cố định.
async fn wait_for_leaders(brokers: &str, names: &[String]) -> Result<()> {
    let client = admin(brokers)?;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    for name in names {
        loop {
            // `fetch_metadata` chặn thread; lab và test đều chạy trên runtime nhiều thread.
            let metadata = tokio::task::block_in_place(|| {
                client
                    .inner()
                    .fetch_metadata(Some(name), Duration::from_secs(5))
            });
            let ready = metadata.is_ok_and(|m| {
                m.topics().iter().any(|t| {
                    t.name() == name.as_str()
                        && t.error().is_none()
                        && !t.partitions().is_empty()
                        && t.partitions()
                            .iter()
                            .all(|p| p.error().is_none() && p.leader() >= 0)
                })
            });
            if ready {
                break;
            }
            if tokio::time::Instant::now() >= deadline {
                bail!("topic {name} chưa có leader sau 30 giây");
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    Ok(())
}

/// Xoá topic của ca chạy. Lỗi "topic không tồn tại" bị bỏ qua.
///
/// # Errors
/// Không kết nối được broker.
pub async fn delete_topics(brokers: &str, names: &[String]) -> Result<()> {
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let _ = admin(brokers)?
        .delete_topics(
            &refs,
            &AdminOptions::new().request_timeout(Some(Duration::from_secs(10))),
        )
        .await
        .context("delete_topics")?;
    Ok(())
}

/// Gửi một giá trị JSON và chờ broker xác nhận.
///
/// # Errors
/// Serialize lỗi hoặc broker không xác nhận trong 10 giây.
pub async fn send_json<T: Serialize>(
    producer: &FutureProducer,
    topic: &str,
    key: &str,
    value: &T,
) -> Result<()> {
    let payload = serde_json::to_vec(value)?;
    producer
        .send(
            FutureRecord::to(topic).key(key).payload(&payload),
            Duration::from_secs(10),
        )
        .await
        .map_err(|(err, _)| err)
        .with_context(|| format!("gửi tới {topic}"))?;
    Ok(())
}

/// Xoá consumer group của ca chạy. Group chỉ xoá được khi đã rỗng; consumer vừa đóng có thể chưa kịp
/// rời group nên thử lại vài lần. Group chưa từng tồn tại (kịch bản hỏng sớm) không phải lỗi.
///
/// # Errors
/// Không kết nối được broker.
pub async fn delete_groups(brokers: &str, names: &[String]) -> Result<()> {
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
