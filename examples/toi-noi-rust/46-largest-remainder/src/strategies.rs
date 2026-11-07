//! Các chiến lược làm tròn của `rust_decimal` đặt cạnh nhau.

use rust_decimal::{Decimal, RoundingStrategy};

/// Bảy chiến lược của `rust_decimal`, kèm tên đọc được.
pub const ALL: [(&str, RoundingStrategy); 7] = [
    ("MidpointNearestEven", RoundingStrategy::MidpointNearestEven),
    (
        "MidpointAwayFromZero",
        RoundingStrategy::MidpointAwayFromZero,
    ),
    ("MidpointTowardZero", RoundingStrategy::MidpointTowardZero),
    ("ToZero", RoundingStrategy::ToZero),
    ("AwayFromZero", RoundingStrategy::AwayFromZero),
    ("ToNegativeInfinity", RoundingStrategy::ToNegativeInfinity),
    ("ToPositiveInfinity", RoundingStrategy::ToPositiveInfinity),
];

/// Làm tròn về đồng (0 chữ số thập phân) rồi lấy `i64`; `None` nếu không vừa.
pub fn round_dong(value: Decimal, strategy: RoundingStrategy) -> Option<i64> {
    use rust_decimal::prelude::ToPrimitive;
    value.round_dp_with_strategy(0, strategy).to_i64()
}

/// Kết quả của một chiến lược trên danh sách giá trị, theo thứ tự.
pub fn round_all(values: &[Decimal], strategy: RoundingStrategy) -> Vec<i64> {
    values
        .iter()
        .map(|&v| round_dong(v, strategy).expect("giá trị bảng vừa i64"))
        .collect()
}
