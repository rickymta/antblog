//! Reason code là enum có bảng đối ứng, và công thức tách một khoản hoàn theo tỷ lệ.

use std::fmt;
use std::str::FromStr;

use ledger_core::Money;

/// Loại của reason code: dùng cho hoàn tiền hay cho chargeback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasonKind {
    /// Nền tảng chủ động hoàn.
    Refund,
    /// PSP đòi lại tiền.
    Chargeback,
}

impl ReasonKind {
    /// Chuỗi lưu trong cột `reason_codes.kind`.
    pub const fn as_str(self) -> &'static str {
        match self {
            ReasonKind::Refund => "REFUND",
            ReasonKind::Chargeback => "CHARGEBACK",
        }
    }
}

/// Reason code đóng: thêm mã mới là thêm biến thể (compiler chỉ chỗ phải sửa)
/// và thêm một dòng vào bảng `reason_codes`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReasonCode {
    /// Tài xế hủy sau khi đã thu cước.
    TripCancelledByDriver,
    /// Khách khiếu nại cước.
    FareDispute,
    /// Thu trùng.
    DuplicateCharge,
    /// Chuyến không thực hiện được.
    ServiceNotRendered,
    /// Chủ thẻ không nhận giao dịch (chargeback).
    Fraud,
    /// Chủ thẻ nói chưa nhận dịch vụ (chargeback).
    NotReceived,
}

impl ReasonCode {
    /// Cả sáu mã, dùng để so với bảng trong database.
    pub const ALL: [ReasonCode; 6] = [
        ReasonCode::TripCancelledByDriver,
        ReasonCode::FareDispute,
        ReasonCode::DuplicateCharge,
        ReasonCode::ServiceNotRendered,
        ReasonCode::Fraud,
        ReasonCode::NotReceived,
    ];

    /// Chuỗi lưu trong database.
    pub const fn as_str(self) -> &'static str {
        match self {
            ReasonCode::TripCancelledByDriver => "TRIP_CANCELLED_BY_DRIVER",
            ReasonCode::FareDispute => "FARE_DISPUTE",
            ReasonCode::DuplicateCharge => "DUPLICATE_CHARGE",
            ReasonCode::ServiceNotRendered => "SERVICE_NOT_RENDERED",
            ReasonCode::Fraud => "FRAUD",
            ReasonCode::NotReceived => "NOT_RECEIVED",
        }
    }

    /// Mã này dùng cho refund hay chargeback.
    pub const fn kind(self) -> ReasonKind {
        match self {
            ReasonCode::Fraud | ReasonCode::NotReceived => ReasonKind::Chargeback,
            _ => ReasonKind::Refund,
        }
    }
}

impl fmt::Display for ReasonCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ReasonCode {
    type Err = String;

    fn from_str(raw: &str) -> Result<ReasonCode, String> {
        ReasonCode::ALL
            .into_iter()
            .find(|c| c.as_str() == raw)
            .ok_or_else(|| format!("reason code lạ: {raw:?}"))
    }
}

/// Một khoản hoàn tách giữa nền tảng (phần phí) và tài xế (phần còn lại).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefundSplit {
    /// Phần phí nền tảng hoàn lại.
    pub fee_part: i64,
    /// Phần tài xế hoàn lại.
    pub driver_part: i64,
}

// Phí đã hoàn khi tổng hoàn luỹ kế là `cumulative`: round_half_up(cumulative * fee / captured),
// chỉ dùng số nguyên (i128 để tích không tràn).
fn fee_target(cumulative: i64, fee: i64, captured: i64) -> i64 {
    let num = i128::from(cumulative) * i128::from(fee) * 2 + i128::from(captured);
    let den = i128::from(captured) * 2;
    i64::try_from(num / den).expect("kết quả không lớn hơn phí, vừa i64")
}

/// Chính sách của lab: hoàn theo tỷ lệ. Nền tảng hoàn phần phí tương ứng, tài xế hoàn phần còn lại,
/// khách không chịu phần nào. Tính theo tổng luỹ kế để nhiều lần hoàn nhỏ cộng lại đúng bằng phí gốc,
/// không trôi vì làm tròn từng lần.
pub fn refund_split(
    captured: Money,
    fee: Money,
    already_refunded: Money,
    amount: Money,
) -> RefundSplit {
    let (c, f) = (captured.dong(), fee.dong());
    let before = already_refunded.dong();
    let after = before + amount.dong();
    let fee_part = fee_target(after, f, c) - fee_target(before, f, c);
    RefundSplit {
        fee_part,
        driver_part: amount.dong() - fee_part,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(n: i64) -> Money {
        Money::from_dong(n)
    }

    #[test]
    fn reason_codes_round_trip() {
        for code in ReasonCode::ALL {
            assert_eq!(code.as_str().parse::<ReasonCode>(), Ok(code));
        }
        assert!("KHONG_CO".parse::<ReasonCode>().is_err());
    }

    #[test]
    fn exact_split_when_fee_is_twenty_percent() {
        let s = refund_split(d(25_500), d(5_100), d(0), d(10_000));
        assert_eq!(
            s,
            RefundSplit {
                fee_part: 2_000,
                driver_part: 8_000
            }
        );
    }

    #[test]
    fn rounding_half_up_never_loses_a_dong() {
        // 12.347 * 20% = 2.469,4 -> 2.469
        let s = refund_split(d(25_500), d(5_100), d(0), d(12_347));
        assert_eq!(
            s,
            RefundSplit {
                fee_part: 2_469,
                driver_part: 9_878
            }
        );
        assert_eq!(s.fee_part + s.driver_part, 12_347);
    }

    #[test]
    fn pieces_add_up_to_original_fee() {
        // Hoàn 7 lần lẻ cho đến hết: tổng phần phí đúng 5.100.
        let amounts = [3_333, 4_001, 777, 5_555, 2_222, 6_000, 3_612];
        assert_eq!(amounts.iter().sum::<i64>(), 25_500);
        let (mut done, mut fee_sum, mut driver_sum) = (0, 0, 0);
        for a in amounts {
            let s = refund_split(d(25_500), d(5_100), d(done), d(a));
            fee_sum += s.fee_part;
            driver_sum += s.driver_part;
            done += a;
        }
        assert_eq!((fee_sum, driver_sum), (5_100, 20_400));
    }
}
