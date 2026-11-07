//! Dòng tiền hai bước qua clearing: capture rồi chia. Dùng cho chuyến `TN-xxxx` và đơn giao `DL-xxxx`.
//!
//! Mọi bút toán dựng bằng `JournalEntryBuilder` của `ledger-core`, nên chỉ tồn tại khi cân.

use ledger_core::{
    AccountId, FeePolicy, JournalEntry, JournalEntryBuilder, LedgerError, Money, TripCharge,
    capture_entry, default_policy,
};

/// Tiền VíXanh đang giữ, chờ chia.
pub const CLEARING: &str = "psp:vixanh:clearing";
/// Doanh thu nền tảng.
pub const REVENUE: &str = "platform:revenue";
/// Thuế giả định (mô hình học tập, không phải tư vấn thuế).
pub const VAT: &str = "tax:vat:payable";

/// Thuế giả định 10% đã nằm TRONG phí nền tảng: thuế = phí / 11 (chia hết ở số liệu của bài).
const VAT_INCLUSIVE_DIVISOR: i64 = 11;

fn acct(raw: &str) -> Result<AccountId, LedgerError> {
    AccountId::new(raw)
}

fn neg(amount: Money) -> Result<Money, LedgerError> {
    amount.checked_neg().ok_or(LedgerError::Overflow)
}

/// Bước 1: khách trả `amount`, tiền nằm ở clearing. Key `<reference>:capture`,
/// ví dụ `trip:TN-0001:capture`.
fn capture(reference: &str, customer: &str, amount: Money) -> Result<JournalEntry, LedgerError> {
    JournalEntryBuilder::new(&format!("{reference}:capture"))
        .posting(acct(&format!("customer:{customer}"))?, neg(amount)?)
        .posting(acct(CLEARING)?, amount)
        .build()
}

/// Chuyến: bước 1, `customer:CU-001` -25.500, clearing +25.500.
pub fn trip_capture(trip: &str, customer: &str, fare: Money) -> Result<JournalEntry, LedgerError> {
    capture(&format!("trip:{trip}"), customer, fare)
}

/// Chuyến: bước 2, clearing -25.500, `platform:revenue` +5.100, `driver:DR-001` +20.400.
pub fn trip_split(trip: &str, driver: &str, fare: Money) -> Result<JournalEntry, LedgerError> {
    let split = default_policy().split(fare)?;
    JournalEntryBuilder::new(&format!("trip:{trip}:split"))
        .posting(acct(CLEARING)?, neg(fare)?)
        .posting(acct(REVENUE)?, split.platform_fee)
        .posting(acct(&format!("driver:{driver}"))?, split.driver_net)
        .build()
}

/// Đơn giao hàng: bước 1, cùng dạng với chuyến.
pub fn delivery_capture(
    order: &str,
    customer: &str,
    fee: Money,
) -> Result<JournalEntry, LedgerError> {
    capture(&format!("delivery:{order}"), customer, fee)
}

/// Đơn giao hàng: bước 2 chia bốn đường: doanh thu nền tảng, thuế giả định, shipper.
/// Phí nền tảng 20% của phí giao; trong đó thuế giả định 10% đã nằm sẵn.
pub fn delivery_split(order: &str, shipper: &str, fee: Money) -> Result<JournalEntry, LedgerError> {
    let split = default_policy().split(fee)?;
    let vat = Money::from_dong(split.platform_fee.dong() / VAT_INCLUSIVE_DIVISOR);
    let revenue = split
        .platform_fee
        .checked_sub(vat)
        .ok_or(LedgerError::Overflow)?;
    JournalEntryBuilder::new(&format!("delivery:{order}:split"))
        .posting(acct(CLEARING)?, neg(fee)?)
        .posting(acct(REVENUE)?, revenue)
        .posting(acct(VAT)?, vat)
        .posting(acct(&format!("driver:{shipper}"))?, split.driver_net)
        .build()
}

/// Nạp ví (fixture, đúng dạng lab 42): ví +`amount`, clearing -`amount`. Tiền từ ngoài vào sổ
/// đóng kín hiện thành số âm ở clearing, tài khoản biên của mô hình này.
pub fn wallet_topup(customer: &str, amount: Money) -> Result<JournalEntry, LedgerError> {
    let wallet = format!("customer:{customer}");
    JournalEntryBuilder::new(&format!("fixture:topup:{wallet}"))
        .posting(acct(&wallet)?, amount)
        .posting(acct(CLEARING)?, neg(amount)?)
        .build()
}

/// Dạng rút gọn ba dòng của bài 10, 14, 20 (`ledger-core::capture_entry`): không qua clearing.
/// Dùng ở đây chỉ để chứng minh nó KHÔNG sống chung sổ với dạng hai bước: cùng key `...:capture`.
pub fn legacy_three_line(
    trip: &str,
    customer: &str,
    driver: &str,
    fare: Money,
) -> Result<JournalEntry, LedgerError> {
    capture_entry(
        &TripCharge {
            trip_id: trip.to_string(),
            customer: acct(&format!("customer:{customer}"))?,
            driver: acct(&format!("driver:{driver}"))?,
            fare,
        },
        &default_policy(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(entry: &JournalEntry) -> Vec<(String, i64)> {
        entry
            .postings()
            .iter()
            .map(|p| (p.account.to_string(), p.amount.dong()))
            .collect()
    }

    #[test]
    fn trip_two_steps_match_the_brief() {
        let fare = Money::from_dong(25_500);
        let capture = trip_capture("TN-0001", "CU-001", fare).unwrap();
        assert_eq!(capture.idempotency_key(), "trip:TN-0001:capture");
        assert_eq!(
            lines(&capture),
            [
                ("customer:CU-001".to_string(), -25_500),
                (CLEARING.to_string(), 25_500)
            ]
        );
        let split = trip_split("TN-0001", "DR-001", fare).unwrap();
        assert_eq!(split.idempotency_key(), "trip:TN-0001:split");
        assert_eq!(
            lines(&split),
            [
                (CLEARING.to_string(), -25_500),
                (REVENUE.to_string(), 5_100),
                ("driver:DR-001".to_string(), 20_400)
            ]
        );
    }

    #[test]
    fn delivery_split_carves_vat_out_of_the_platform_fee() {
        let split = delivery_split("DL-0001", "DR-002", Money::from_dong(33_000)).unwrap();
        assert_eq!(
            lines(&split),
            [
                (CLEARING.to_string(), -33_000),
                (REVENUE.to_string(), 6_000),
                (VAT.to_string(), 600),
                ("driver:DR-002".to_string(), 26_400)
            ]
        );
    }

    #[test]
    fn legacy_form_skips_clearing() {
        let entry =
            legacy_three_line("TN-0001", "CU-001", "DR-001", Money::from_dong(25_500)).unwrap();
        assert_eq!(entry.idempotency_key(), "trip:TN-0001:capture");
        assert!(lines(&entry).iter().all(|(a, _)| a != CLEARING));
    }
}
