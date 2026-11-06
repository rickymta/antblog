//! Vòng nhận tin từ Kafka và cơ chế backpressure: hàng đợi `mpsc` có giới hạn.

use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;

use rdkafka::Message;
use rdkafka::consumer::{Consumer, StreamConsumer};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::metrics::Metrics;

/// Bước poll khi đang pause: đủ ngắn để phản ứng nhanh, đủ dài để không quay vòng rỗng.
const POLL_STEP: Duration = Duration::from_millis(50);

/// Một tin đã tách khỏi bộ đệm của librdkafka, đủ để xử lý và lưu offset sau này.
#[derive(Debug)]
pub struct Job {
    /// Topic nguồn.
    pub topic: String,
    /// Partition nguồn.
    pub partition: i32,
    /// Offset của chính tin này.
    pub offset: i64,
    /// Nội dung JSON.
    pub payload: Vec<u8>,
}

impl Job {
    fn from_message(msg: &rdkafka::message::BorrowedMessage<'_>) -> Job {
        Job {
            topic: msg.topic().to_string(),
            partition: msg.partition(),
            offset: msg.offset(),
            payload: msg.payload().unwrap_or_default().to_vec(),
        }
    }
}

/// Lấy tin từ Kafka đẩy vào hàng đợi cho đến khi `token` bị huỷ.
/// Hàng đợi đầy thì `pause` các partition nhưng VẪN poll; rút `tx` khi thoát để worker biết hết việc.
pub async fn intake_loop(
    consumer: Arc<StreamConsumer>,
    tx: mpsc::Sender<Job>,
    token: CancellationToken,
    metrics: Arc<Metrics>,
    resume_depth: usize,
) {
    loop {
        if tx.capacity() == 0
            && !wait_for_room(&consumer, &tx, &token, &metrics, resume_depth).await
        {
            break;
        }
        tokio::select! {
            biased;
            () = token.cancelled() => break,
            received = consumer.recv() => match received {
                Ok(msg) => {
                    metrics.received.fetch_add(1, Relaxed);
                    // Chỉ vòng này gửi vào hàng đợi, và `capacity() > 0` đã kiểm ở trên: không chặn.
                    if tx.send(Job::from_message(&msg)).await.is_err() {
                        break;
                    }
                    let depth = (tx.max_capacity() - tx.capacity()) as u64;
                    metrics.max_depth.fetch_max(depth, Relaxed);
                }
                Err(err) => {
                    // Ví dụ PollExceeded (quá max.poll.interval.ms): ghi nhận rồi poll tiếp.
                    metrics.consumer_errors.fetch_add(1, Relaxed);
                    warn!("consumer báo lỗi: {err}");
                }
            },
        }
    }
    info!("intake dừng: không lấy thêm tin, offset cuối do worker lưu");
}

/// Pause, chờ worker rút bớt hàng đợi, rồi resume. Trả `false` nếu bị huỷ trong lúc chờ.
async fn wait_for_room(
    consumer: &StreamConsumer,
    tx: &mpsc::Sender<Job>,
    token: &CancellationToken,
    metrics: &Metrics,
    resume_depth: usize,
) -> bool {
    let assignment = consumer.assignment().unwrap_or_default();
    if let Err(err) = consumer.pause(&assignment) {
        warn!("pause lỗi: {err}");
    }
    metrics.paused.fetch_add(1, Relaxed);
    // Resume khi hàng đợi còn tối đa `resume_depth` tin. Resume không tức thì (luồng broker của
    // librdkafka chỉ xét lại partition khi nó thức dậy), nên ngưỡng phải đủ cao để worker còn việc.
    while tx.max_capacity() - tx.capacity() > resume_depth {
        if token.is_cancelled() {
            return false;
        }
        // Pause không thay cho poll: không gọi poll quá max.poll.interval.ms là bị đá khỏi group.
        // Partition đã pause nên không có tin mới, trừ tin nằm sẵn trước lúc pause có hiệu lực.
        match tokio::time::timeout(POLL_STEP, consumer.recv()).await {
            Err(_elapsed) => {
                metrics.polls_while_paused.fetch_add(1, Relaxed);
            }
            Ok(Ok(msg)) => {
                metrics.received.fetch_add(1, Relaxed);
                // Không được bỏ tin này: offset của nó đã đi qua. Chờ worker nhả một chỗ.
                if tx.send(Job::from_message(&msg)).await.is_err() {
                    return false;
                }
            }
            Ok(Err(err)) => {
                metrics.consumer_errors.fetch_add(1, Relaxed);
                warn!("consumer báo lỗi khi đang pause: {err}");
            }
        }
    }
    if let Err(err) = consumer.resume(&assignment) {
        warn!("resume lỗi: {err}");
    }
    metrics.resumed.fetch_add(1, Relaxed);
    true
}
