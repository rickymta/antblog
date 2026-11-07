//! Quy tắc phí theo phiên bản. Phiên bản là enum: thêm `V3` mà quên cập nhật `match` là lỗi biên dịch.

use ledger_core::{
    FeePolicy, FeeSplit, HalfAwayFromZero, LedgerError, Money, RoundingPolicy, default_policy,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Phiên bản quy tắc phí, lưu trong sự kiện quyết định phí (`fee_events.policy_version`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyVersion {
    /// 20%, nửa đồng làm tròn ra xa 0 (quy tắc từ trước tới nay).
    V1,
    /// 15%, nhưng không dưới 3.000 đồng mỗi chuyến.
    V2,
}

impl PolicyVersion {
    /// Mọi phiên bản, theo thứ tự cũ tới mới.
    pub const ALL: [PolicyVersion; 2] = [PolicyVersion::V1, PolicyVersion::V2];

    /// Phiên bản mới nhất (quy tắc "hiện hành").
    pub const fn latest() -> PolicyVersion {
        PolicyVersion::V2
    }

    /// Số lưu trong cột `policy_version`.
    pub const fn number(self) -> i32 {
        match self {
            PolicyVersion::V1 => 1,
            PolicyVersion::V2 => 2,
        }
    }

    /// Đọc lại từ cột; `None` nếu DB có số lạ (dữ liệu hỏng thì không đoán).
    pub const fn from_number(n: i32) -> Option<PolicyVersion> {
        match n {
            1 => Some(PolicyVersion::V1),
            2 => Some(PolicyVersion::V2),
            _ => None,
        }
    }

    /// Tỷ lệ danh nghĩa ghi vào cột cũ `fee_percent` khi ghi đúp. Với `V2` nó chỉ gần đúng:
    /// sàn 3.000 đồng làm tỷ lệ thực của chuyến ngắn cao hơn 15%.
    pub const fn nominal_percent(self) -> i32 {
        match self {
            PolicyVersion::V1 => 20,
            PolicyVersion::V2 => 15,
        }
    }
}

/// Phí theo tỷ lệ nhưng có sàn: `max(làm_tròn(cước × tỷ lệ), sàn)`, không vượt cước.
#[derive(Debug, Clone)]
pub struct FloorFee<R: RoundingPolicy> {
    rate: Decimal,
    floor: Money,
    rounding: R,
}

impl<R: RoundingPolicy> FloorFee<R> {
    /// Phí `rate` với sàn `floor`, làm tròn bằng `rounding`.
    pub fn new(rate: Decimal, floor: Money, rounding: R) -> Self {
        FloorFee {
            rate,
            floor,
            rounding,
        }
    }
}

impl<R: RoundingPolicy> FeePolicy for FloorFee<R> {
    fn name(&self) -> String {
        format!(
            "{}% sàn {} · {}",
            (self.rate * Decimal::from(100)).normalize(),
            self.floor,
            self.rounding.name()
        )
    }

    fn fee(&self, fare: Money) -> Result<Money, LedgerError> {
        let by_rate = self.rounding.round_ratio(fare, self.rate)?;
        Ok(by_rate.max(self.floor).min(fare))
    }
}

/// Quy tắc phí của một phiên bản. `match` vét cạn: không có nhánh mặc định.
pub fn policy_for(version: PolicyVersion) -> Box<dyn FeePolicy + Send + Sync> {
    match version {
        PolicyVersion::V1 => Box::new(default_policy()),
        PolicyVersion::V2 => Box::new(FloorFee::new(
            Decimal::new(15, 2),
            Money::from_dong(3_000),
            HalfAwayFromZero,
        )),
    }
}

/// Chia cước giữa nền tảng và tài xế theo quy tắc của `version`.
///
/// # Errors
/// Lỗi của [`FeePolicy::split`].
pub fn split_for(version: PolicyVersion, fare: Money) -> Result<FeeSplit, LedgerError> {
    policy_for(version).split(fare)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dong(n: i64) -> Money {
        Money::from_dong(n)
    }

    #[test]
    fn v1_is_twenty_percent() {
        let split = split_for(PolicyVersion::V1, dong(25_500)).unwrap();
        assert_eq!(split.platform_fee, dong(5_100));
        assert_eq!(split.driver_net, dong(20_400));
    }

    #[test]
    fn v2_is_fifteen_percent_with_a_floor() {
        // 25.500 × 15% = 3.825 (trên sàn); 12.000 × 15% = 1.800 (dưới sàn, lên 3.000).
        let long = split_for(PolicyVersion::V2, dong(25_500)).unwrap();
        assert_eq!(long.platform_fee, dong(3_825));
        assert_eq!(long.driver_net, dong(21_675));
        let short = split_for(PolicyVersion::V2, dong(12_000)).unwrap();
        assert_eq!(short.platform_fee, dong(3_000));
        assert_eq!(short.driver_net, dong(9_000));
    }

    #[test]
    fn floor_never_exceeds_the_fare() {
        let split = split_for(PolicyVersion::V2, dong(2_000)).unwrap();
        assert_eq!(split.platform_fee, dong(2_000));
        assert_eq!(split.driver_net, Money::ZERO);
    }

    #[test]
    fn numbers_round_trip_and_unknown_is_none() {
        for v in PolicyVersion::ALL {
            assert_eq!(PolicyVersion::from_number(v.number()), Some(v));
        }
        assert_eq!(PolicyVersion::from_number(0), None);
        assert_eq!(PolicyVersion::from_number(3), None);
    }
}
