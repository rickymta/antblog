//! Tiêu thụ và (cho lab, test) phát `location.updates.v1` bằng `rdkafka`.
//!
//! Quyết định thiết kế quan trọng: mỗi bản `geo-engine` cần **toàn bộ** vị trí của thành phố nên **không dùng nhóm consumer chia
//! partition**; mỗi bản `assign` tất cả partition (nhóm id chỉ để thỏa librdkafka, không commit offset). Khởi động lại thì
//! đọc lại từ `now − TTL` bằng `offsets_for_times`: kho chỉ cần các điểm còn trong TTL nên phần phát lại rất ngắn.
//!
//! `rdkafka` build ở đây không cần `cmake`: dùng build tự cấu hình (`make` + `cc`) của librdkafka đi kèm crate `rdkafka-sys`.

use crate::StoreError;
use crate::model::Update;
use crate::pb;
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::error::{KafkaError, RDKafkaErrorCode};
use rdkafka::message::Message as _;
use rdkafka::producer::{BaseProducer, BaseRecord, Producer};
use rdkafka::{Offset, TopicPartitionList};
use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::time::{Duration, Instant};

pub const TOPIC: &str = "location.updates.v1";
pub const DEFAULT_BROKERS: &str = "127.0.0.1:9096";

pub fn brokers_from_env() -> String {
    std::env::var("TOI_NOI_KAFKA_BROKERS").unwrap_or_else(|_| DEFAULT_BROKERS.to_string())
}

fn unreachable_msg(brokers: &str, e: &KafkaError) -> StoreError {
    StoreError::Other(format!(
        "không kết nối được Kafka ở {brokers}: {e}. Bật hạ tầng lab: docker compose -f examples/toi-noi-location/infra/docker-compose.yml up -d --wait kafka"
    ))
}

#[derive(Debug, Clone, Copy)]
pub enum Start {
    Latest,
    Earliest,
    /// Từ bản ghi đầu tiên có timestamp ≥ giá trị này (epoch ms).
    SinceMs(i64),
}

pub enum Event {
    Update(Update),
    /// Bản ghi không giải mã được hoặc thiếu trường bắt buộc: bỏ, nhưng đếm.
    Poison,
}

#[derive(Debug, Clone, Default)]
pub struct ReplayStats {
    pub messages: u64,
    pub poison: u64,
    pub partitions: usize,
    pub elapsed: Duration,
    pub timed_out: bool,
}

pub struct KafkaConsumer {
    c: BaseConsumer,
    topic: String,
    /// (partition, offset bắt đầu, high watermark lúc kết nối)
    layout: Vec<(i32, i64, i64)>,
}

impl KafkaConsumer {
    pub fn connect(brokers: &str, topic: &str, start: Start) -> Result<Self, StoreError> {
        let c: BaseConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", format!("location-store-{}", std::process::id()))
            .set("enable.auto.commit", "false")
            .set("fetch.wait.max.ms", "20")
            .set("socket.timeout.ms", "6000")
            .create()?;
        let md = c
            .fetch_metadata(Some(topic), Duration::from_secs(5))
            .map_err(|e| unreachable_msg(brokers, &e))?;
        let t = md
            .topics()
            .iter()
            .find(|t| t.name() == topic)
            .filter(|t| !t.partitions().is_empty())
            .ok_or_else(|| {
                StoreError::Other(format!(
                    "topic {topic} chưa có partition nào trên {brokers}"
                ))
            })?;
        let parts: Vec<i32> = t.partitions().iter().map(|p| p.id()).collect();
        let timeout = Duration::from_secs(5);
        let mut layout = Vec::new();
        let mut tpl = TopicPartitionList::new();
        let mut by_time = TopicPartitionList::new();
        for &p in &parts {
            let (low, high) = c.fetch_watermarks(topic, p, timeout)?;
            let off = match start {
                Start::Latest => Offset::Offset(high),
                Start::Earliest => Offset::Offset(low),
                Start::SinceMs(ms) => {
                    by_time.add_partition_offset(topic, p, Offset::Offset(ms))?;
                    Offset::Offset(low) // thay bằng kết quả offsets_for_times bên dưới
                }
            };
            tpl.add_partition_offset(topic, p, off)?;
            layout.push((p, low, high));
        }
        if let Start::SinceMs(_) = start {
            let resolved = c.offsets_for_times(by_time, timeout)?;
            tpl = TopicPartitionList::new();
            for (p, low, high) in &mut layout {
                let off = resolved
                    .find_partition(topic, *p)
                    .map_or(Offset::End, |e| e.offset());
                let at = match off {
                    Offset::Offset(o) if o >= 0 => o,
                    _ => *high, // không có bản ghi nào từ mốc đó: đọc từ cuối
                };
                *low = at;
                tpl.add_partition_offset(topic, *p, Offset::Offset(at))?;
            }
        } else if let Start::Latest = start {
            for (_, low, high) in &mut layout {
                *low = *high;
            }
        }
        c.assign(&tpl)?;
        Ok(Self {
            c,
            topic: topic.to_string(),
            layout,
        })
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn partitions(&self) -> usize {
        self.layout.len()
    }

    /// Lấy một bản ghi, `Ok(None)` nếu hết hạn chờ.
    pub fn poll(&self, timeout: Duration) -> Result<Option<(Event, i32, i64)>, StoreError> {
        match self.c.poll(timeout) {
            None => Ok(None),
            Some(Err(e)) => Err(e.into()),
            Some(Ok(m)) => {
                let ev = m
                    .payload()
                    .and_then(|b| <pb::DriverLocationUpdated as prost::Message>::decode(b).ok())
                    .and_then(|d| Update::from_proto(&d))
                    .map_or(Event::Poison, Event::Update);
                Ok(Some((ev, m.partition(), m.offset())))
            }
        }
    }

    /// Đọc cho tới khi mọi partition tới high watermark lúc kết nối (hoặc hết `timeout`), gọi `f` cho từng cập nhật.
    /// Dùng để khởi động lại: sau khi trả về, kho đã bắt kịp luồng tại thời điểm kết nối.
    pub fn catch_up(
        &self,
        mut f: impl FnMut(Update),
        timeout: Duration,
    ) -> Result<ReplayStats, StoreError> {
        let t0 = Instant::now();
        let mut next: Vec<i64> = self.layout.iter().map(|&(_, start, _)| start).collect();
        let high: Vec<i64> = self.layout.iter().map(|&(_, _, h)| h).collect();
        let mut st = ReplayStats {
            partitions: self.layout.len(),
            ..ReplayStats::default()
        };
        loop {
            if next.iter().zip(&high).all(|(n, h)| n >= h) {
                break;
            }
            if t0.elapsed() > timeout {
                st.timed_out = true;
                break;
            }
            if let Some((ev, part, off)) = self.poll(Duration::from_millis(50))? {
                st.messages += 1;
                if let Some(i) = self.layout.iter().position(|&(p, _, _)| p == part) {
                    next[i] = off + 1;
                }
                match ev {
                    Event::Update(u) => f(u),
                    Event::Poison => st.poison += 1,
                }
            }
        }
        st.elapsed = t0.elapsed();
        Ok(st)
    }
}

fn block_on<F: Future>(f: F) -> F::Output {
    struct Unpark(std::thread::Thread);
    impl Wake for Unpark {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker = Waker::from(Arc::new(Unpark(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut f = pin!(f);
    loop {
        if let Poll::Ready(v) = f.as_mut().poll(&mut cx) {
            return v;
        }
        std::thread::park_timeout(Duration::from_millis(20));
    }
}

fn admin(brokers: &str) -> Result<AdminClient<DefaultClientContext>, StoreError> {
    Ok(ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .create()?)
}

/// Tạo topic với `partitions` partition, replication 1 (broker lab chỉ có một nút). Đã tồn tại thì bỏ qua.
/// Broker lab chạy chung máy với nhiều việc khác nên đôi khi trả lời chậm: thử tối đa 3 lần, mỗi lần chờ tới 20 giây.
pub fn ensure_topic(brokers: &str, topic: &str, partitions: i32) -> Result<(), StoreError> {
    let mut last: Option<KafkaError> = None;
    for _ in 0..3 {
        let a = admin(brokers)?;
        let res = block_on(
            a.create_topics(
                &[NewTopic::new(topic, partitions, TopicReplication::Fixed(1))],
                &AdminOptions::new()
                    .request_timeout(Some(Duration::from_secs(20)))
                    .operation_timeout(Some(Duration::from_secs(10))),
            ),
        );
        match res {
            Ok(results) => {
                for r in results {
                    match r {
                        Ok(_) | Err((_, RDKafkaErrorCode::TopicAlreadyExists)) => {}
                        Err((t, code)) => {
                            return Err(StoreError::Other(format!("tạo topic {t}: {code}")));
                        }
                    }
                }
                return Ok(());
            }
            Err(e) => last = Some(e),
        }
    }
    Err(unreachable_msg(
        brokers,
        &last.expect("đã thử ít nhất một lần"),
    ))
}

/// Xóa topic (chỉ dùng cho topic lab do chính mình tạo).
pub fn delete_topic(brokers: &str, topic: &str) -> Result<(), StoreError> {
    let a = admin(brokers)?;
    let _ = block_on(a.delete_topics(
        &[topic],
        &AdminOptions::new().request_timeout(Some(Duration::from_secs(15))),
    ))?;
    Ok(())
}

/// Producer cho lab và test: key = `driver_id`, giá trị = `DriverLocationUpdated`, timestamp bản ghi = `received_ms`.
pub struct LabProducer {
    p: BaseProducer,
    topic: String,
}

impl LabProducer {
    pub fn new(brokers: &str, topic: &str) -> Result<Self, StoreError> {
        let p: BaseProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("linger.ms", "5")
            .set("queue.buffering.max.messages", "500000")
            .create()?;
        Ok(Self {
            p,
            topic: topic.to_string(),
        })
    }

    pub fn send(&self, u: &Update) -> Result<(), StoreError> {
        let payload = prost::Message::encode_to_vec(&u.to_proto());
        loop {
            let rec = BaseRecord::to(&self.topic)
                .key(u.driver_id.as_bytes())
                .payload(&payload)
                .timestamp(u.pos.received_ms);
            match self.p.send(rec) {
                Ok(()) => {
                    self.p.poll(Duration::ZERO);
                    return Ok(());
                }
                Err((KafkaError::MessageProduction(RDKafkaErrorCode::QueueFull), _)) => {
                    self.p.poll(Duration::from_millis(20));
                }
                Err((e, _)) => return Err(e.into()),
            }
        }
    }

    /// Gửi nguyên byte (test bản ghi hỏng).
    pub fn send_raw(&self, key: &str, payload: &[u8]) -> Result<(), StoreError> {
        loop {
            match self.p.send(
                BaseRecord::to(&self.topic)
                    .key(key.as_bytes())
                    .payload(payload),
            ) {
                Ok(()) => return Ok(()),
                Err((KafkaError::MessageProduction(RDKafkaErrorCode::QueueFull), _)) => {
                    self.p.poll(Duration::from_millis(20));
                }
                Err((e, _)) => return Err(e.into()),
            }
        }
    }

    pub fn flush(&self) -> Result<(), StoreError> {
        self.p.flush(Duration::from_secs(30))?;
        Ok(())
    }
}
