//! Hoàn tiền một phần (bài 43), đường chính duy nhất: khách nhận lại `amount` vào ví,
//! nền tảng hoàn phần phí tương ứng, tài xế hoàn phần còn lại. Khóa `trip:<id>:refund:<n>`,
//! không đặt `reversal_of` vì đây không phải ảnh gương của một bút toán nào.

use ledger_core::{AccountId, JournalEntry, JournalEntryBuilder, LedgerError, Money};
use thiserror::Error;

use crate::chart::REVENUE;

/// Lỗi nghiệp vụ của hoàn tiền.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RefundError {
    /// Số hoàn không dương.
    #[error("số hoàn phải dương, nhận {0} đồng")]
    NonPositive(i64),
    /// Số thứ tự hoàn bắt đầu từ 1.
    #[error("số thứ tự hoàn phải từ 1")]
    BadSequence,
    /// Hoàn vượt phần còn hoàn được.
    #[error("hoàn {requested} vượt phần còn hoàn được {refundable}")]
    ExceedsRefundable {
        /// Số xin hoàn.
        requested: Money,
        /// Phần còn hoàn được.
        refundable: Money,
    },
    /// Cùng key nhưng khác số tiền so với lần đã ghi.
    #[error("key {key} đã dùng cho lần hoàn khác nội dung")]
    KeyConflict {
        /// Key bị trùng.
        key: String,
    },
    /// Lỗi của lõi sổ cái.
    #[error(transparent)]
    Ledger(#[from] LedgerError),
}

/// Ảnh chụp thanh toán của một chuyến, đọc dưới khóa dòng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentSnapshot {
    /// Mã chuyến.
    pub trip_id: String,
    /// Ví khách.
    pub customer: AccountId,
    /// Ví tài xế.
    pub driver: AccountId,
    /// Số đã thu.
    pub captured: Money,
    /// Phí nền tảng của số đã thu.
    pub fee: Money,
    /// Số đã hoàn từ trước.
    pub refunded: Money,
}

impl PaymentSnapshot {
    /// Phần còn hoàn được.
    pub fn refundable(&self) -> Money {
        Money::from_dong(self.captured.dong() - self.refunded.dong())
    }
}

/// Khóa của lần hoàn thứ `n`.
pub fn refund_key(trip_id: &str, n: u32) -> String {
    format!("trip:{trip_id}:refund:{n}")
}

/// Phí đáng ra đã hoàn khi tổng hoàn luỹ kế là `cumulative`: `cumulative × fee / captured`,
/// làm tròn nửa đồng lên, bằng số nguyên. Tính theo luỹ kế như bài 43 để nhiều lần hoàn nhỏ
/// cộng lại đúng bằng phí gốc, không trôi vì làm tròn từng lần.
fn fee_target(cumulative: i64, fee: i64, captured: i64) -> Option<i64> {
    let numerator = i128::from(cumulative) * i128::from(fee) * 2 + i128::from(captured);
    let target = numerator / (i128::from(captured) * 2);
    i64::try_from(target).ok()
}

/// Dựng bút toán hoàn lần `n` cho `amount`. Không kiểm key đã tồn tại: việc đó thuộc lớp lưu trữ.
///
/// # Errors
/// [`RefundError`] khi số hoàn không dương, `n = 0`, vượt phần còn hoàn được hoặc bút toán sai.
pub fn refund_entry(
    payment: &PaymentSnapshot,
    n: u32,
    amount: Money,
) -> Result<JournalEntry, RefundError> {
    if n == 0 {
        return Err(RefundError::BadSequence);
    }
    if amount <= Money::ZERO {
        return Err(RefundError::NonPositive(amount.dong()));
    }
    let refundable = payment.refundable();
    if amount > refundable {
        return Err(RefundError::ExceedsRefundable {
            requested: amount,
            refundable,
        });
    }
    let (fee_total, captured) = (payment.fee.dong(), payment.captured.dong());
    let before = payment.refunded.dong();
    let fee = fee_target(before + amount.dong(), fee_total, captured)
        .zip(fee_target(before, fee_total, captured))
        .map(|(after, already)| after - already)
        .ok_or(LedgerError::Overflow)?;
    let driver_part = amount.dong() - fee;
    let mut builder = JournalEntryBuilder::new(&refund_key(&payment.trip_id, n))
        .posting(payment.customer.clone(), amount);
    // Posting 0 đồng bị CHECK của bảng từ chối nên bỏ dòng nào bằng 0.
    if fee > 0 {
        builder = builder.posting(AccountId::new(REVENUE)?, Money::from_dong(-fee));
    }
    if driver_part > 0 {
        builder = builder.posting(payment.driver.clone(), Money::from_dong(-driver_part));
    }
    Ok(builder.build()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payment(refunded: i64) -> PaymentSnapshot {
        PaymentSnapshot {
            trip_id: "TN-0001".to_string(),
            customer: AccountId::new("customer:CU-001").unwrap(),
            driver: AccountId::new("driver:DR-001").unwrap(),
            captured: Money::from_dong(25_500),
            fee: Money::from_dong(5_100),
            refunded: Money::from_dong(refunded),
        }
    }

    fn lines(entry: &JournalEntry) -> Vec<(&str, i64)> {
        entry
            .postings()
            .iter()
            .map(|p| (p.account.as_str(), p.amount.dong()))
            .collect()
    }

    #[test]
    fn partial_refund_splits_by_the_original_ratio() {
        let entry = refund_entry(&payment(0), 1, Money::from_dong(10_000)).unwrap();
        assert_eq!(entry.idempotency_key(), "trip:TN-0001:refund:1");
        assert_eq!(
            lines(&entry),
            [
                ("customer:CU-001", 10_000),
                (REVENUE, -2_000),
                ("driver:DR-001", -8_000)
            ]
        );
        assert_eq!(entry.reverses(), None);
    }

    #[test]
    fn refund_cannot_exceed_what_is_left() {
        let err = refund_entry(&payment(10_000), 2, Money::from_dong(15_501)).unwrap_err();
        assert_eq!(
            err,
            RefundError::ExceedsRefundable {
                requested: Money::from_dong(15_501),
                refundable: Money::from_dong(15_500)
            }
        );
        assert!(refund_entry(&payment(10_000), 2, Money::from_dong(15_500)).is_ok());
    }

    #[test]
    fn two_refunds_sum_back_to_the_original_split() {
        let first = refund_entry(&payment(0), 1, Money::from_dong(10_000)).unwrap();
        let second = refund_entry(&payment(10_000), 2, Money::from_dong(15_500)).unwrap();
        let sum_of = |account: &str| -> i64 {
            [&first, &second]
                .iter()
                .flat_map(|e| e.postings())
                .filter(|p| p.account.as_str() == account)
                .map(|p| p.amount.dong())
                .sum()
        };
        assert_eq!(sum_of(REVENUE), -5_100);
        assert_eq!(sum_of("driver:DR-001"), -20_400);
        assert_eq!(sum_of("customer:CU-001"), 25_500);
    }

    // 7 đồng × 20% = 1,4 đồng: làm tròn riêng từng lần thì cộng lại ra phí sai.
    #[test]
    fn many_small_refunds_add_up_to_the_exact_fee() {
        let (mut refunded, mut n, mut fee, mut driver) = (0, 1, 0, 0);
        while refunded < 25_500 {
            let amount = (25_500 - refunded).min(7);
            let entry = refund_entry(&payment(refunded), n, Money::from_dong(amount)).unwrap();
            for (account, dong) in lines(&entry) {
                match account {
                    REVENUE => fee += dong,
                    "driver:DR-001" => driver += dong,
                    _ => assert_eq!(dong, amount),
                }
            }
            refunded += amount;
            n += 1;
        }
        assert_eq!((fee, driver), (-5_100, -20_400));
    }

    #[test]
    fn bad_inputs() {
        assert_eq!(
            refund_entry(&payment(0), 0, Money::from_dong(1)).unwrap_err(),
            RefundError::BadSequence
        );
        assert_eq!(
            refund_entry(&payment(0), 1, Money::ZERO).unwrap_err(),
            RefundError::NonPositive(0)
        );
    }
}
