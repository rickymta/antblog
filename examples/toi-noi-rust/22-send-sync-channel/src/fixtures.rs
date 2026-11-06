//! Dữ liệu hư cấu của Tới Nơi dùng chung cho demo, test và đo.

use ledger_core::{
    AccountId, JournalEntry, LedgerError, TripCharge, capture_entry, default_policy, fare_for_km,
};

/// Bút toán thu cước chuyến `TN-0001` (5 km): `trip:TN-0001:capture`, 25.500 VND.
///
/// # Errors
/// Lỗi dựng bút toán (không xảy ra với dữ liệu cố định).
pub fn tn_0001_entry() -> Result<JournalEntry, LedgerError> {
    trip_entry(1, 5)
}

/// Bút toán thu cước chuyến thứ `n` (`TN-0002`, ...) đi `km` km, cùng khách và tài xế mẫu.
///
/// # Errors
/// Lỗi dựng bút toán hoặc mã tài khoản.
pub fn trip_entry(n: u32, km: u32) -> Result<JournalEntry, LedgerError> {
    let trip = TripCharge {
        trip_id: format!("TN-{n:04}"),
        customer: AccountId::new("customer:CU-001")?,
        driver: AccountId::new("driver:DR-001")?,
        fare: fare_for_km(km),
    };
    capture_entry(&trip, &default_policy())
}
