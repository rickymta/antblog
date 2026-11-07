//! Chuyến hoàn thành → bút toán hai bước qua clearing (bài 13, 41).
//!
//! Bước 1 `trip:<id>:capture`: khách ra, clearing vào. Bước 2 `trip:<id>:split`: clearing ra,
//! nền tảng và tài xế vào. Dạng ba dòng của `ledger-core::capture_entry` chỉ là rút gọn.

use ledger_core::{AccountId, FeePolicy, JournalEntry, JournalEntryBuilder, LedgerError, Money};
use thiserror::Error;

use crate::chart::{self, CLEARING, REVENUE};

/// Lỗi của dữ liệu chuyến.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TripError {
    /// Cước không dương: chuyến miễn phí không sinh bút toán.
    #[error("cước {0} đồng không dương")]
    NonPositiveFare(i64),
    /// Mã chuyến rỗng.
    #[error("mã chuyến rỗng")]
    EmptyTripId,
    /// Lỗi của lõi sổ cái (mã tài khoản rỗng, tràn số, phí sai).
    #[error(transparent)]
    Ledger(#[from] LedgerError),
}

/// Chuyến đã kiểm: cước dương, tài khoản hợp lệ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TripFare {
    /// Mã chuyến, ví dụ `TN-0001`.
    pub trip_id: String,
    /// Ví khách.
    pub customer: AccountId,
    /// Ví tài xế.
    pub driver: AccountId,
    /// Cước khách trả.
    pub fare: Money,
}

impl TripFare {
    /// Dựng từ các mã thô của sự kiện.
    ///
    /// # Errors
    /// [`TripError`] khi mã chuyến rỗng, cước không dương hoặc mã tài khoản rỗng.
    pub fn new(
        trip_id: &str,
        customer_id: &str,
        driver_id: &str,
        fare_dong: i64,
    ) -> Result<TripFare, TripError> {
        if trip_id.trim().is_empty() {
            return Err(TripError::EmptyTripId);
        }
        if fare_dong <= 0 {
            return Err(TripError::NonPositiveFare(fare_dong));
        }
        Ok(TripFare {
            trip_id: trip_id.to_string(),
            customer: chart::customer(customer_id)?,
            driver: chart::driver(driver_id)?,
            fare: Money::from_dong(fare_dong),
        })
    }

    /// Khóa của hold gắn với chuyến này: `trip:<id>:hold`.
    pub fn hold_key(&self) -> String {
        hold_key(&self.trip_id)
    }
}

/// Khóa hold của một chuyến.
pub fn hold_key(trip_id: &str) -> String {
    format!("trip:{trip_id}:hold")
}

/// Hai bút toán của một chuyến, cộng phí nền tảng để lưu vào `payments`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturePlan {
    /// Bước 1: khách -cước, clearing +cước.
    pub capture: JournalEntry,
    /// Bước 2: clearing -cước, nền tảng +phí, tài xế +phần còn lại.
    pub split: JournalEntry,
    /// Phí nền tảng.
    pub fee: Money,
}

/// Dựng hai bút toán của chuyến theo chính sách phí. Hàm chỉ biết trait `FeePolicy`.
///
/// # Errors
/// [`LedgerError`] của chính sách phí hoặc của việc dựng bút toán.
pub fn plan_capture<F>(trip: &TripFare, policy: &F) -> Result<CapturePlan, LedgerError>
where
    F: FeePolicy + ?Sized,
{
    let split = policy.split(trip.fare)?;
    let out = trip.fare.checked_neg().ok_or(LedgerError::Overflow)?;
    let clearing = AccountId::new(CLEARING)?;
    let capture = JournalEntryBuilder::new(&format!("trip:{}:capture", trip.trip_id))
        .posting(trip.customer.clone(), out)
        .posting(clearing.clone(), trip.fare)
        .build()?;
    let split_entry = JournalEntryBuilder::new(&format!("trip:{}:split", trip.trip_id))
        .posting(clearing, out)
        .posting(AccountId::new(REVENUE)?, split.platform_fee)
        .posting(trip.driver.clone(), split.driver_net)
        .build()?;
    Ok(CapturePlan {
        capture,
        split: split_entry,
        fee: split.platform_fee,
    })
}

#[cfg(test)]
mod tests {
    use ledger_core::{default_policy, fare_for_km};

    use super::*;

    fn lines(entry: &JournalEntry) -> Vec<(&str, i64)> {
        entry
            .postings()
            .iter()
            .map(|p| (p.account.as_str(), p.amount.dong()))
            .collect()
    }

    #[test]
    fn trip_5km_becomes_two_balanced_entries() {
        let trip = TripFare::new("TN-0001", "CU-001", "DR-001", fare_for_km(5).dong()).unwrap();
        let plan = plan_capture(&trip, &default_policy()).unwrap();
        assert_eq!(plan.capture.idempotency_key(), "trip:TN-0001:capture");
        assert_eq!(
            lines(&plan.capture),
            [("customer:CU-001", -25_500), (CLEARING, 25_500)]
        );
        assert_eq!(plan.split.idempotency_key(), "trip:TN-0001:split");
        assert_eq!(
            lines(&plan.split),
            [
                (CLEARING, -25_500),
                (REVENUE, 5_100),
                ("driver:DR-001", 20_400)
            ]
        );
        assert_eq!(plan.fee, Money::from_dong(5_100));
    }

    #[test]
    fn bad_trips_are_rejected() {
        assert_eq!(
            TripFare::new("TN-1", "CU-001", "DR-001", 0).unwrap_err(),
            TripError::NonPositiveFare(0)
        );
        assert_eq!(
            TripFare::new(" ", "CU-001", "DR-001", 100).unwrap_err(),
            TripError::EmptyTripId
        );
    }
}
