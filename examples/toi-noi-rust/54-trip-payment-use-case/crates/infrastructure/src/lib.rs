//! Adapter của use case thanh toán chuyến: PostgreSQL (sqlx), Kafka (rdkafka), relay outbox và
//! môi trường lab riêng mỗi ca. Crate này phụ thuộc `application` và `domain`, không ngược lại.

pub mod consumer;
pub mod kafka;
pub mod lab;
pub mod pg;
pub mod probe;
pub mod progress;
pub mod relay;
pub mod service;

/// Điểm có thể «chết» trong lúc xử lý. Chỉ tiến trình con `serve` của test đọc nó từ môi trường;
/// tiến trình chính của lab và test luôn là [`Fault::None`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Fault {
    /// Chạy bình thường.
    #[default]
    None,
    /// Chết (`abort`) ngay trước `COMMIT` của transaction ghi bút toán: DB không có gì, offset chưa commit.
    AbortBeforeCommit,
    /// Chết ngay sau `COMMIT`, trước khi commit offset: DB đã ghi, Kafka chưa biết.
    AbortAfterCommit,
}

impl Fault {
    /// Tên dùng trong biến môi trường `PAYMENT_FAULT`.
    pub fn as_str(self) -> &'static str {
        match self {
            Fault::None => "none",
            Fault::AbortBeforeCommit => "before-commit",
            Fault::AbortAfterCommit => "after-commit",
        }
    }

    /// Ngược lại của [`Fault::as_str`].
    pub fn parse(raw: &str) -> Option<Fault> {
        match raw {
            "none" => Some(Fault::None),
            "before-commit" => Some(Fault::AbortBeforeCommit),
            "after-commit" => Some(Fault::AbortAfterCommit),
            _ => None,
        }
    }
}

/// Tự giết tiến trình bằng `abort` (SIGABRT): không chạy `Drop`, không dọn gì, như bị kill giữa chừng.
pub fn die(point: &str) -> ! {
    eprintln!("FAULT: tự abort ở điểm chọn trước: {point}");
    std::process::abort()
}
