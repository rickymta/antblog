//! Cước và phí nền tảng bằng số nguyên đồng. Rút gọn từ lab 20: chỉ cần đủ để service có việc để làm.

use serde::Serialize;

/// Cước 2 km đầu.
pub const BASE_FARE_DONG: i64 = 12_000;
/// Số km đã nằm trong cước nền.
pub const BASE_KM: i64 = 2;
/// Mỗi km sau đó.
pub const PER_KM_DONG: i64 = 4_500;
/// Phí nền tảng, phần trăm trên cước.
pub const PLATFORM_FEE_PERCENT: i64 = 20;
/// Chặn đầu vào để phép nhân không tràn.
pub const MAX_KM: i64 = 1_000;

/// Kết quả tính cước, JSON camelCase như hợp đồng sự kiện của loạt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fare {
    /// Quãng đường (km, số nguyên).
    pub km: i64,
    /// Cước khách trả.
    pub fare_dong: i64,
    /// Phần của nền tảng.
    pub platform_fee_dong: i64,
    /// Phần của tài xế.
    pub driver_dong: i64,
}

/// Tính cước. `None` nếu `km` ngoài `1..=MAX_KM`.
#[must_use]
pub fn fare_for_km(km: i64) -> Option<Fare> {
    if !(1..=MAX_KM).contains(&km) {
        return None;
    }
    let extra_km = (km - BASE_KM).max(0);
    let fare_dong = BASE_FARE_DONG + extra_km * PER_KM_DONG;
    let platform_fee_dong = fare_dong * PLATFORM_FEE_PERCENT / 100;
    Some(Fare {
        km,
        fare_dong,
        platform_fee_dong,
        driver_dong: fare_dong - platform_fee_dong,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trip_tn_0001_five_km() {
        let fare = fare_for_km(5).unwrap();
        assert_eq!(fare.fare_dong, 25_500);
        assert_eq!(fare.platform_fee_dong, 5_100);
        assert_eq!(fare.driver_dong, 20_400);
    }

    #[test]
    fn two_km_is_base_fare_only() {
        assert_eq!(fare_for_km(2).unwrap().fare_dong, 12_000);
        assert_eq!(fare_for_km(1).unwrap().fare_dong, 12_000);
    }

    #[test]
    fn out_of_range_is_none() {
        assert_eq!(fare_for_km(0), None);
        assert_eq!(fare_for_km(-3), None);
        assert_eq!(fare_for_km(MAX_KM + 1), None);
    }

    #[test]
    fn parts_always_sum_to_fare() {
        for km in 1..=MAX_KM {
            let f = fare_for_km(km).unwrap();
            assert_eq!(f.platform_fee_dong + f.driver_dong, f.fare_dong);
        }
    }
}
