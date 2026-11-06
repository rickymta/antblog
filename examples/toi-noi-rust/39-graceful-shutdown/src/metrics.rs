//! Bộ đếm dùng chung giữa các task; mọi con số trong bài đều đọc từ đây.

use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

/// Bộ đếm của một lần chạy service.
#[derive(Debug, Default)]
pub struct Metrics {
    /// Tin đã lấy từ Kafka.
    pub received: AtomicU64,
    /// Bút toán mới được ghi.
    pub posted: AtomicU64,
    /// Tin trùng (inbox hoặc idempotency key đã có), bỏ qua.
    pub duplicates: AtomicU64,
    /// Tin không đọc được, bỏ qua (bài 37 nói về DLQ).
    pub poison: AtomicU64,
    /// 1 khi worker đang giữ một transaction chưa commit.
    pub in_flight: AtomicU64,
    /// Số lần consumer pause vì hàng đợi đầy.
    pub paused: AtomicU64,
    /// Số lần resume.
    pub resumed: AtomicU64,
    /// Số lần gọi poll trong lúc đang pause.
    pub polls_while_paused: AtomicU64,
    /// Số tin tối đa từng nằm trong hàng đợi.
    pub max_depth: AtomicU64,
    /// Lỗi consumer trả về qua `recv` (ví dụ MaxPollExceeded).
    pub consumer_errors: AtomicU64,
    /// Dòng outbox đã đẩy lên Kafka.
    pub relayed: AtomicU64,
}

impl Metrics {
    /// Số tin đã xử lý xong (mới, trùng hoặc bỏ qua): đã sẵn sàng để commit offset.
    pub fn done(&self) -> u64 {
        self.posted.load(Relaxed) + self.duplicates.load(Relaxed) + self.poison.load(Relaxed)
    }

    /// Đọc một bộ đếm.
    pub fn get(counter: &AtomicU64) -> u64 {
        counter.load(Relaxed)
    }
}
