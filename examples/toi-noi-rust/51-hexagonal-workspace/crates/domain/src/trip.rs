//! Chuyến đã hoàn thành và hai bút toán của nó: capture rồi chia tiền.

use ledger_core::{
    AccountId, FeePolicy, JournalEntry, JournalEntryBuilder, LedgerError, Money, default_policy,
    fare_for_km,
};
use thiserror::Error;

/// Lỗi khi dựng chuyến hoặc bút toán của chuyến.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TripError {
    /// Mã chuyến rỗng.
    #[error("mã chuyến rỗng")]
    EmptyTripId,
    /// Quãng đường bằng 0.
    #[error("quãng đường phải lớn hơn 0 km")]
    ZeroDistance,
    /// Lỗi của sổ cái (mã tài khoản rỗng, tràn số, bút toán lệch).
    #[error(transparent)]
    Ledger(#[from] LedgerError),
}

/// Chuyến đã hoàn thành. Trường private: chỉ [`Trip::new`] tạo được, nên mọi `Trip` đã hợp lệ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trip {
    id: String,
    customer: AccountId,
    driver: AccountId,
    distance_km: u32,
}

/// Hai bút toán của một chuyến (dạng hai bước của bài 13).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settlement {
    /// Cước khách trả.
    pub fare: Money,
    /// Phí nền tảng.
    pub platform_fee: Money,
    /// Phần tài xế nhận.
    pub driver_net: Money,
    /// Bước 1: khách trả, tiền vào clearing của VíXanh.
    pub capture: JournalEntry,
    /// Bước 2: clearing chia cho nền tảng và tài xế.
    pub split: JournalEntry,
}

// Hàm nội bộ: `pub(crate)` nên crate khác không gọi được, dù cùng workspace.
pub(crate) fn step_key(trip_id: &str, step: &str) -> String {
    format!("trip:{trip_id}:{step}")
}

impl Trip {
    /// Dựng chuyến từ mã chuyến, mã khách (`CU-001`), mã tài xế (`DR-001`) và số km.
    ///
    /// # Errors
    /// [`TripError::EmptyTripId`], [`TripError::ZeroDistance`] hoặc lỗi mã tài khoản rỗng.
    pub fn new(
        id: &str,
        customer_id: &str,
        driver_id: &str,
        distance_km: u32,
    ) -> Result<Trip, TripError> {
        if id.trim().is_empty() {
            return Err(TripError::EmptyTripId);
        }
        if distance_km == 0 {
            return Err(TripError::ZeroDistance);
        }
        Ok(Trip {
            id: id.to_string(),
            customer: AccountId::new(&format!("customer:{customer_id}"))?,
            driver: AccountId::new(&format!("driver:{driver_id}"))?,
            distance_km,
        })
    }

    /// Mã chuyến.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Cước theo mô hình học tập: 12.000 đồng cho 2 km đầu, 4.500 đồng mỗi km vượt.
    pub fn fare(&self) -> Money {
        fare_for_km(self.distance_km)
    }

    /// Dựng hai bút toán `trip:<id>:capture` và `trip:<id>:split`. Phí nền tảng 20% làm tròn ra xa 0.
    ///
    /// # Errors
    /// Lỗi của chính sách phí hoặc của bút toán (tràn số, lệch).
    pub fn settlement(&self) -> Result<Settlement, TripError> {
        let fare = self.fare();
        let split = default_policy().split(fare)?;
        let clearing = AccountId::new("psp:vixanh:clearing")?;
        let revenue = AccountId::new("platform:revenue")?;
        let out = |m: Money| m.checked_neg().ok_or(LedgerError::Overflow);

        // Dương = tiền vào tài khoản, âm = tiền ra (quy ước sổ dòng tiền của loạt).
        let capture = JournalEntryBuilder::new(&step_key(&self.id, "capture"))
            .posting(self.customer.clone(), out(fare)?)
            .posting(clearing.clone(), fare)
            .build()?;
        let split_entry = JournalEntryBuilder::new(&step_key(&self.id, "split"))
            .posting(clearing, out(fare)?)
            .posting(revenue, split.platform_fee)
            .posting(self.driver.clone(), split.driver_net)
            .build()?;
        Ok(Settlement {
            fare,
            platform_fee: split.platform_fee,
            driver_net: split.driver_net,
            capture,
            split: split_entry,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amounts(entry: &JournalEntry) -> Vec<i64> {
        entry.postings().iter().map(|p| p.amount.dong()).collect()
    }

    #[test]
    fn trip_tn_0001_has_two_balanced_entries() {
        let trip = Trip::new("TN-0001", "CU-001", "DR-001", 5).unwrap();
        let s = trip.settlement().unwrap();
        assert_eq!(s.fare.dong(), 25_500);
        assert_eq!(s.platform_fee.dong(), 5_100);
        assert_eq!(s.driver_net.dong(), 20_400);
        assert_eq!(s.capture.idempotency_key(), "trip:TN-0001:capture");
        assert_eq!(s.split.idempotency_key(), "trip:TN-0001:split");
        assert_eq!(amounts(&s.capture), vec![-25_500, 25_500]);
        assert_eq!(amounts(&s.split), vec![-25_500, 5_100, 20_400]);
    }

    #[test]
    fn invalid_trips_are_rejected() {
        assert_eq!(
            Trip::new(" ", "CU-001", "DR-001", 5).unwrap_err(),
            TripError::EmptyTripId
        );
        assert_eq!(
            Trip::new("TN-0001", "CU-001", "DR-001", 0).unwrap_err(),
            TripError::ZeroDistance
        );
    }
}
