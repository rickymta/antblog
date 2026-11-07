//! Hai bút toán chuẩn của một chuyến, theo dòng tiền của loạt (bài 13):
//! capture đưa tiền khách vào clearing, chia đưa tiền từ clearing sang nền tảng và tài xế.

use ledger_core::{
    AccountId, FeePolicy, JournalEntry, JournalEntryBuilder, LedgerError, Money, default_policy,
    fare_for_km,
};

/// Cặp bút toán của một chuyến.
#[derive(Debug, Clone)]
pub struct TripEntries {
    /// `trip:<id>:capture`: khách -cước, clearing +cước.
    pub capture: JournalEntry,
    /// `trip:<id>:split`: clearing -cước, doanh thu +phí, tài xế +phần còn lại.
    pub split: JournalEntry,
}

/// Dựng hai bút toán cho chuyến đi `km` km (phí nền tảng 20%).
///
/// # Errors
/// Lỗi nghiệp vụ của `ledger-core`, ví dụ mã tài khoản rỗng.
pub fn trip_entries(
    trip_id: &str,
    customer: &str,
    driver: &str,
    km: u32,
) -> Result<TripEntries, LedgerError> {
    let fare = fare_for_km(km);
    let split = default_policy().split(fare)?;
    let out = fare.checked_neg().ok_or(LedgerError::Overflow)?;
    let clearing = || AccountId::new("psp:vixanh:clearing");

    let capture = JournalEntryBuilder::new(&format!("trip:{trip_id}:capture"))
        .posting(AccountId::new(customer)?, out)
        .posting(clearing()?, fare)
        .build()?;
    let split_entry = JournalEntryBuilder::new(&format!("trip:{trip_id}:split"))
        .posting(clearing()?, out)
        .posting(AccountId::new("platform:revenue")?, split.platform_fee)
        .posting(AccountId::new(driver)?, split.driver_net)
        .build()?;
    Ok(TripEntries {
        capture,
        split: split_entry,
    })
}

/// Bốn tài khoản của chuyến mẫu `TN-0001` và loại của chúng.
pub const TN_0001_ACCOUNTS: [(&str, &str); 4] = [
    ("customer:CU-001", "liability"),
    ("driver:DR-001", "liability"),
    ("platform:revenue", "revenue"),
    ("psp:vixanh:clearing", "asset"),
];

/// Số dư kỳ vọng sau khi ghi cả hai bút toán của `TN-0001`.
pub fn tn_0001_expected() -> [(&'static str, Money); 4] {
    [
        ("customer:CU-001", Money::from_dong(-25_500)),
        ("driver:DR-001", Money::from_dong(20_400)),
        ("platform:revenue", Money::from_dong(5_100)),
        ("psp:vixanh:clearing", Money::ZERO),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tn_0001_entries_have_standard_keys_and_amounts() {
        let e = trip_entries("TN-0001", "customer:CU-001", "driver:DR-001", 5).unwrap();
        assert_eq!(e.capture.idempotency_key(), "trip:TN-0001:capture");
        assert_eq!(e.split.idempotency_key(), "trip:TN-0001:split");
        let amounts: Vec<i64> = e.split.postings().iter().map(|p| p.amount.dong()).collect();
        assert_eq!(amounts, [-25_500, 5_100, 20_400]);
    }

    #[test]
    fn empty_driver_id_is_rejected() {
        assert!(trip_entries("TN-0001", "customer:CU-001", "", 5).is_err());
    }
}
