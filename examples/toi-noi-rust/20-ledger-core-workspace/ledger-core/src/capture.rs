//! Dựng bút toán thu cước. Hàm ở đây chỉ biết trait `FeePolicy`, không biết chính sách cụ thể nào.

use crate::account::AccountId;
use crate::entry::{JournalEntry, Posting};
use crate::error::LedgerError;
use crate::money::Money;
use crate::policy::FeePolicy;

/// Dữ liệu một chuyến đã hoàn thành cần thu cước.
#[derive(Debug, Clone)]
pub struct TripCharge {
    /// Mã chuyến, ví dụ `TN-0001`.
    pub trip_id: String,
    /// Tài khoản khách bị trừ cước.
    pub customer: AccountId,
    /// Tài khoản tài xế nhận phần còn lại.
    pub driver: AccountId,
    /// Cước khách trả.
    pub fare: Money,
}

/// Cước học tập: 12.000 đồng cho 2 km đầu, 4.500 đồng mỗi km vượt.
///
/// ```
/// use ledger_core::{Money, fare_for_km};
///
/// assert_eq!(fare_for_km(5), Money::from_dong(25_500));
/// ```
pub fn fare_for_km(km: u32) -> Money {
    let extra_km = i64::from(km.saturating_sub(2));
    Money::from_dong(12_000 + 4_500 * extra_km)
}

/// Bút toán thu cước: khách bị trừ cước, nền tảng nhận phí, tài xế nhận phần còn lại.
///
/// `F: FeePolicy + ?Sized`: nhận kiểu cụ thể (static dispatch) lẫn `&dyn FeePolicy`
/// (dynamic dispatch). Thêm chính sách phí mới không phải sửa hàm này.
///
/// # Errors
/// Mọi lỗi của [`FeePolicy::split`], [`LedgerError::EmptyAccountId`] và
/// [`LedgerError::Overflow`].
pub fn capture_entry<F>(trip: &TripCharge, policy: &F) -> Result<JournalEntry, LedgerError>
where
    F: FeePolicy + ?Sized,
{
    let split = policy.split(trip.fare)?;
    let platform = AccountId::new("platform:revenue")?;
    let charge = trip.fare.checked_neg().ok_or(LedgerError::Overflow)?;
    JournalEntry::try_new(
        &format!("trip:{}:capture", trip.trip_id),
        vec![
            Posting::new(trip.customer.clone(), charge),
            Posting::new(platform, split.platform_fee),
            Posting::new(trip.driver.clone(), split.driver_net),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{FlatFee, default_policy};

    fn trip() -> TripCharge {
        TripCharge {
            trip_id: "TN-0001".to_string(),
            customer: AccountId::new("customer:CU-001").unwrap(),
            driver: AccountId::new("driver:DR-001").unwrap(),
            fare: fare_for_km(5),
        }
    }

    #[test]
    fn fare_for_five_km_is_25_500() {
        assert_eq!(fare_for_km(5), Money::from_dong(25_500));
        assert_eq!(fare_for_km(1), Money::from_dong(12_000));
    }

    #[test]
    fn capture_with_default_policy() -> Result<(), LedgerError> {
        let entry = capture_entry(&trip(), &default_policy())?;
        assert_eq!(entry.idempotency_key(), "trip:TN-0001:capture");
        let amounts: Vec<i64> = entry.postings().iter().map(|p| p.amount.dong()).collect();
        assert_eq!(amounts, vec![-25_500, 5_100, 20_400]);
        Ok(())
    }

    // Cùng một hàm, hai chính sách: một qua generic, một qua `&dyn`.
    #[test]
    fn capture_accepts_static_and_dynamic_policies() -> Result<(), LedgerError> {
        let flat = FlatFee::new(Money::from_dong(3_000));
        let dynamic: &dyn FeePolicy = &flat;
        let entry = capture_entry(&trip(), dynamic)?;
        let amounts: Vec<i64> = entry.postings().iter().map(|p| p.amount.dong()).collect();
        assert_eq!(amounts, vec![-25_500, 3_000, 22_500]);
        Ok(())
    }
}
