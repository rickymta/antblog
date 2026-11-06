//! Tính cước và chia tiền một chuyến: bản sao tối thiểu để chạy `cargo mutants`.
//! Số tiền là `i64` đơn vị đồng, không dùng số thực.

/// Cước cơ bản cho 2 km đầu (đồng).
pub const BASE_FARE: i64 = 12_000;
/// Giá mỗi km vượt 2 km (đồng).
pub const PER_KM: i64 = 4_500;

/// Kết quả chia một chuyến: `fee + driver == fare`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Split {
    /// Cước khách trả.
    pub fare: i64,
    /// Phí nền tảng.
    pub fee: i64,
    /// Phần tài xế nhận.
    pub driver: i64,
}

/// Lỗi chia tiền.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitError {
    /// Cước âm.
    NegativeFare,
    /// Phần trăm phí ngoài khoảng 0..=100.
    PercentOutOfRange,
    /// Phép nhân tràn `i64`.
    Overflow,
}

/// Cước theo số km: 12.000 cho 2 km đầu, 4.500 cho mỗi km vượt.
#[must_use]
pub fn fare_for_km(km: u32) -> i64 {
    if km <= 2 {
        BASE_FARE
    } else {
        BASE_FARE + PER_KM * i64::from(km - 2)
    }
}

/// Chia `fare` thành phí nền tảng (`fee_percent` %, làm tròn nửa lên về đồng) và phần tài xế.
///
/// # Errors
/// [`SplitError`] khi cước âm, phần trăm ngoài 0..=100 hoặc tràn số.
pub fn split_fare(fare: i64, fee_percent: i64) -> Result<Split, SplitError> {
    if fare < 0 {
        return Err(SplitError::NegativeFare);
    }
    if !(0..=100).contains(&fee_percent) {
        return Err(SplitError::PercentOutOfRange);
    }
    let scaled = fare.checked_mul(fee_percent).ok_or(SplitError::Overflow)?;
    let mut fee = scaled / 100;
    if scaled % 100 >= 50 {
        fee += 1; // nửa đồng trở lên thì làm tròn lên
    }
    Ok(Split {
        fare,
        fee,
        driver: fare - fee,
    })
}
