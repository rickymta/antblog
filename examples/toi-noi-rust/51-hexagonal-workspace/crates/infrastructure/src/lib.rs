//! `infrastructure`: adapter cho các port của `application`.
//!
//! Luôn có: bộ nhớ (`memory`), PostgreSQL qua sqlx (`pg`), đồng hồ (`clock`), dạng JSON trên dây (`wire`).
//! Chỉ có khi bật feature `kafka`: `KafkaPublisher` (cần librdkafka, build từ nguồn C).

pub mod clock;
#[cfg(feature = "kafka")]
pub mod kafka;
pub mod memory;
pub mod pg;
pub mod wire;

/// Sơ đồ tài khoản tối thiểu của chuyến `TN-0001`: mã và loại (`kind`).
pub const CHART: [(&str, &str); 4] = [
    ("customer:CU-001", "liability"),
    ("driver:DR-001", "liability"),
    ("platform:revenue", "revenue"),
    ("psp:vixanh:clearing", "asset"),
];
