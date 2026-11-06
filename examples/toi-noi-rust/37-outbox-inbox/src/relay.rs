//! Relay: đọc outbox bằng `FOR UPDATE SKIP LOCKED`, gửi Kafka, đánh dấu `published_at`.

use std::sync::Arc;

use anyhow::{Result, anyhow};
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord};
use sqlx::{PgPool, Row};
use tokio::sync::Notify;

use crate::events::EVENT_ID_HEADER;

/// Chỉ nhận các key thuộc một phần của `count` phần chia theo md5(key).
#[derive(Debug, Clone, Copy)]
pub struct Shard {
    /// Phần của worker này, từ 0.
    pub index: i64,
    /// Tổng số phần.
    pub count: i64,
}

/// Tham số một lượt relay.
#[derive(Debug, Clone)]
pub struct RelayConfig {
    /// Số dòng tối đa mỗi lượt.
    pub batch: i64,
    /// `Some`: chỉ lấy key của phần này (giữ thứ tự theo key khi chạy nhiều worker).
    pub shard: Option<Shard>,
    /// `true`: `FOR UPDATE SKIP LOCKED`. `false`: SELECT thường (chỉ để chứng minh lỗi).
    pub lock: bool,
    /// Báo hiệu ngay sau khi nhận dòng: để thí nghiệm khởi động worker kia đúng lúc, không đoán bằng giờ.
    pub claimed: Option<Arc<Notify>>,
    /// Đợi tín hiệu này sau khi nhận dòng, trước khi gửi: giữ worker lại tới khi worker khác làm xong.
    pub resume: Option<Arc<Notify>>,
    /// `process::exit(137)` ngay sau khi Kafka xác nhận, trước khi đánh dấu: giả lập bị kill, không phải SIGKILL.
    pub die_after_send: bool,
}

impl Default for RelayConfig {
    fn default() -> RelayConfig {
        RelayConfig {
            batch: 10,
            shard: None,
            lock: true,
            claimed: None,
            resume: None,
            die_after_send: false,
        }
    }
}

/// Câu nhận dòng: khóa hàng, bỏ qua hàng worker khác đang giữ. Tham số: $1 batch,
/// $2 số phần (0 = không chia), $3 phần của worker này.
const CLAIM_LOCKED: &str = "SELECT id, topic, key, payload::text AS payload FROM outbox \
     WHERE published_at IS NULL \
       AND ($2::bigint = 0 OR (('x' || substr(md5(key), 1, 7))::bit(28)::int) % $2 = $3) \
     ORDER BY id LIMIT $1 FOR UPDATE SKIP LOCKED";

/// Cùng câu trên nhưng không khóa: hai worker có thể nhận cùng dòng.
const CLAIM_PLAIN: &str = "SELECT id, topic, key, payload::text AS payload FROM outbox \
     WHERE published_at IS NULL \
       AND ($2::bigint = 0 OR (('x' || substr(md5(key), 1, 7))::bit(28)::int) % $2 = $3) \
     ORDER BY id LIMIT $1";

/// `event_id` của một dòng outbox: ổn định giữa các lần gửi lại.
pub fn event_id_of(outbox_id: i64) -> String {
    format!("ledger:{outbox_id}")
}

/// Một lượt relay. Trả về id các dòng đã gửi và đánh dấu; rỗng nghĩa là không còn gì để nhận.
///
/// Transaction (và khóa hàng) mở suốt lúc gửi Kafka: giữ một kết nối của pool tới khi có ack.
///
/// Lỗi gửi giữa chừng làm cả lượt rollback: các dòng chưa đánh dấu sẽ được gửi lại,
/// kể cả những dòng broker đã nhận (at-least-once).
///
/// # Errors
/// Lỗi SQL hoặc lỗi gửi Kafka.
pub async fn relay_once(
    pool: &PgPool,
    producer: &FutureProducer,
    cfg: &RelayConfig,
) -> Result<Vec<i64>> {
    let mut tx = pool.begin().await?;
    let (shard_count, shard_index) = cfg.shard.map_or((0, 0), |s| (s.count, s.index));
    let sql = if cfg.lock { CLAIM_LOCKED } else { CLAIM_PLAIN };
    let rows = sqlx::query(sql)
        .bind(cfg.batch)
        .bind(shard_count)
        .bind(shard_index)
        .fetch_all(&mut *tx)
        .await?;
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    if let Some(claimed) = &cfg.claimed {
        claimed.notify_one();
    }
    if let Some(resume) = &cfg.resume {
        resume.notified().await;
    }

    // Xếp hàng hết rồi mới đợi xác nhận theo đúng thứ tự: librdkafka giữ thứ tự trong partition.
    let mut in_flight = Vec::with_capacity(rows.len());
    for row in &rows {
        let id: i64 = row.get("id");
        let topic: String = row.get("topic");
        let key: String = row.get("key");
        let payload: String = row.get("payload");
        let event_id = event_id_of(id);
        let headers = OwnedHeaders::new().insert(Header {
            key: EVENT_ID_HEADER,
            value: Some(&event_id),
        });
        let record = FutureRecord::to(&topic)
            .key(&key)
            .payload(&payload)
            .headers(headers);
        let delivery = producer
            .send_result(record)
            .map_err(|(err, _)| anyhow!("không xếp hàng được dòng outbox {id}: {err}"))?;
        in_flight.push((id, delivery));
    }
    let mut ids = Vec::with_capacity(in_flight.len());
    for (id, delivery) in in_flight {
        delivery
            .await
            .map_err(|_| anyhow!("producer bị huỷ khi gửi dòng outbox {id}"))?
            .map_err(|(err, _)| anyhow!("Kafka từ chối dòng outbox {id}: {err}"))?;
        ids.push(id);
    }

    if cfg.die_after_send {
        // Kafka đã xác nhận, outbox chưa đánh dấu: giả lập bị kill (destructor không chạy).
        std::process::exit(137);
    }

    sqlx::query("UPDATE outbox SET published_at = now() WHERE id = ANY($1)")
        .bind(&ids)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(ids)
}

/// Gọi [`relay_once`] đến khi không còn dòng nào để nhận. Trả về tổng số dòng đã gửi.
///
/// # Errors
/// Lỗi của [`relay_once`].
pub async fn drain(pool: &PgPool, producer: &FutureProducer, cfg: &RelayConfig) -> Result<usize> {
    let mut total = 0;
    loop {
        let sent = relay_once(pool, producer, cfg).await?;
        if sent.is_empty() {
            return Ok(total);
        }
        total += sent.len();
    }
}
