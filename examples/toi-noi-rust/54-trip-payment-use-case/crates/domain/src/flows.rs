//! Bút toán của từng bước. Mỗi hàm chỉ dựng bút toán đã cân qua `JournalEntryBuilder` của
//! `ledger-core`; không biết sổ nằm ở đâu. Phí đến từ `default_policy()` của bài 11, không tính tay.

use ledger_core::{
    AccountId, FeePolicy, JournalEntry, JournalEntryBuilder, LedgerError, Money, default_policy,
};

use crate::payment::{DomainError, PromoGrant};

/// Tiền VíXanh đang giữ.
pub const CLEARING: &str = "psp:vixanh:clearing";
/// Doanh thu nền tảng.
pub const REVENUE: &str = "platform:revenue";
/// Chi phí khuyến mãi.
pub const PROMO_EXPENSE: &str = "platform:promo-expense";
/// Quỹ nền tảng, nguồn cấp ngân sách campaign (bài 45).
pub const TREASURY: &str = "platform:treasury";
/// Ví khuyến mãi của nền tảng, trả thay khách phần nền tảng chịu (bài 45).
pub const PROMO_WALLET: &str = "platform:promo-wallet";

/// Tài khoản ngân sách của một campaign.
pub fn budget_account(campaign: &str) -> String {
    format!("promo:budget:{campaign}")
}

fn acct(raw: &str) -> Result<AccountId, LedgerError> {
    AccountId::new(raw)
}

fn neg(amount: Money) -> Result<Money, LedgerError> {
    amount.checked_neg().ok_or(LedgerError::Overflow)
}

/// Bước 1, `trip:<id>:capture`: VíXanh thu đúng `amount`, tiền từ ví khách sang clearing.
///
/// # Errors
/// Mã tài khoản rỗng hoặc tràn số.
pub fn capture_entry(
    trip: &str,
    customer: &str,
    amount: Money,
) -> Result<JournalEntry, LedgerError> {
    JournalEntryBuilder::new(&format!("trip:{trip}:capture"))
        .posting(acct(&format!("customer:{customer}"))?, neg(amount)?)
        .posting(acct(CLEARING)?, amount)
        .build()
}

/// Bước 2, `trip:<id>:split`: clearing trả ra số đã thu, doanh thu nhận phí 20% tính trên cước GỐC,
/// tài xế nhận phần còn lại trừ phần mình chịu khuyến mãi. Có khuyến mãi thì ví khuyến mãi trả
/// thay phần nền tảng chịu. Posting 0 đồng bị bỏ vì cột `amount_dong` cấm 0.
///
/// # Errors
/// Tài xế chịu nhiều hơn phần họ nhận, hoặc lỗi của `ledger-core`.
pub fn split_entry(
    trip: &str,
    driver: &str,
    fare: Money,
    collected: Money,
    promo: Option<&PromoGrant>,
) -> Result<JournalEntry, DomainError> {
    let fee = default_policy().split(fare)?;
    let (platform_share, driver_share) = promo.map_or((Money::ZERO, Money::ZERO), |grant| {
        (
            Money::from_dong(grant.platform_dong),
            Money::from_dong(grant.driver_dong),
        )
    });
    let driver_amount = fee
        .driver_net
        .checked_sub(driver_share)
        .ok_or(LedgerError::Overflow)?;
    if driver_amount < Money::ZERO {
        return Err(DomainError::Invalid(format!(
            "tài xế chịu {driver_share} nhưng chỉ nhận {}",
            fee.driver_net
        )));
    }
    let mut builder = JournalEntryBuilder::new(&format!("trip:{trip}:split"));
    for (account, amount) in [
        (CLEARING.to_string(), neg(collected)?),
        (PROMO_WALLET.to_string(), neg(platform_share)?),
        (REVENUE.to_string(), fee.platform_fee),
        (format!("driver:{driver}"), driver_amount),
    ] {
        if amount != Money::ZERO {
            builder = builder.posting(acct(&account)?, amount);
        }
    }
    Ok(builder.build()?)
}

/// Dùng ngân sách campaign cho phần nền tảng chịu: ngân sách ra, chi phí khuyến mãi vào (bài 45).
///
/// # Errors
/// Lỗi của `ledger-core`.
pub fn redeem_entry(
    trip: &str,
    campaign: &str,
    platform_share: Money,
) -> Result<JournalEntry, LedgerError> {
    JournalEntryBuilder::new(&format!("trip:{trip}:promo:{campaign}"))
        .posting(acct(&budget_account(campaign))?, neg(platform_share)?)
        .posting(acct(PROMO_EXPENSE)?, platform_share)
        .build()
}

/// Cấp ngân sách campaign từ quỹ nền tảng: asset sang asset, doanh thu không đổi.
///
/// # Errors
/// Lỗi của `ledger-core`.
pub fn fund_entry(campaign: &str, cap: Money) -> Result<JournalEntry, LedgerError> {
    JournalEntryBuilder::new(&format!("campaign:{campaign}:fund"))
        .posting(acct(TREASURY)?, neg(cap)?)
        .posting(acct(&budget_account(campaign))?, cap)
        .build()
}

/// Phần phí «đáng ra đã hoàn» khi tổng hoàn luỹ kế là `cumulative`, làm tròn nửa lên, chỉ số nguyên (bài 43).
fn fee_target(cumulative: i64, fee: i64, captured: i64) -> Result<i64, LedgerError> {
    let num = i128::from(cumulative) * i128::from(fee) * 2 + i128::from(captured);
    let den = i128::from(captured) * 2;
    i64::try_from(num / den).map_err(|_| LedgerError::Overflow)
}

/// Chia một lần hoàn: `(phần phí nền tảng trả lại, phần tài xế trả)`. Theo tổng luỹ kế nên
/// nhiều lần hoàn lẻ cộng lại vẫn đúng từng đồng.
///
/// # Errors
/// Tràn số.
pub fn refund_split(
    captured: Money,
    fee: Money,
    already_refunded: i64,
    amount: Money,
) -> Result<(i64, i64), LedgerError> {
    let after = already_refunded
        .checked_add(amount.dong())
        .ok_or(LedgerError::Overflow)?;
    let fee_part = fee_target(after, fee.dong(), captured.dong())?
        - fee_target(already_refunded, fee.dong(), captured.dong())?;
    Ok((fee_part, amount.dong() - fee_part))
}

/// Hoàn một phần, `trip:<id>:refund:<n>`: khách nhận đủ số hoàn, nền tảng và tài xế trả phần của mình.
/// Không đặt `reversal_of` vì đây không phải ảnh gương của bút toán nào (bài 43).
///
/// # Errors
/// Lỗi của `ledger-core`.
pub fn refund_entry(
    trip: &str,
    n: u32,
    customer: &str,
    driver: &str,
    amount: Money,
    fee_part: i64,
    driver_part: i64,
) -> Result<JournalEntry, LedgerError> {
    let mut builder = JournalEntryBuilder::new(&format!("trip:{trip}:refund:{n}"));
    for (account, dong) in [
        (format!("customer:{customer}"), amount.dong()),
        (REVENUE.to_string(), -fee_part),
        (format!("driver:{driver}"), -driver_part),
    ] {
        if dong != 0 {
            builder = builder.posting(acct(&account)?, Money::from_dong(dong));
        }
    }
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(entry: &JournalEntry) -> Vec<(&str, i64)> {
        entry
            .postings()
            .iter()
            .map(|p| (p.account.as_str(), p.amount.dong()))
            .collect()
    }

    #[test]
    fn two_step_flow_of_tn_0001() {
        let fare = Money::from_dong(25_500);
        let capture = capture_entry("TN-0001", "CU-001", fare).unwrap();
        assert_eq!(capture.idempotency_key(), "trip:TN-0001:capture");
        assert_eq!(
            lines(&capture),
            vec![("customer:CU-001", -25_500), (CLEARING, 25_500)]
        );
        let split = split_entry("TN-0001", "DR-001", fare, fare, None).unwrap();
        assert_eq!(split.idempotency_key(), "trip:TN-0001:split");
        assert_eq!(
            lines(&split),
            vec![
                (CLEARING, -25_500),
                (REVENUE, 5_100),
                ("driver:DR-001", 20_400)
            ]
        );
    }

    #[test]
    fn promo_split_keeps_full_fee_on_the_original_fare() {
        let grant = PromoGrant {
            campaign: "WELCOME5K".to_string(),
            platform_dong: 5_000,
            driver_dong: 0,
        };
        let split = split_entry(
            "TN-0002",
            "DR-002",
            Money::from_dong(25_500),
            Money::from_dong(20_500),
            Some(&grant),
        )
        .unwrap();
        assert_eq!(
            lines(&split),
            vec![
                (CLEARING, -20_500),
                (PROMO_WALLET, -5_000),
                (REVENUE, 5_100),
                ("driver:DR-002", 20_400)
            ]
        );
    }

    #[test]
    fn refund_pieces_add_up_to_the_original_fee() {
        let (captured, fee) = (Money::from_dong(25_500), Money::from_dong(5_100));
        let first = refund_split(captured, fee, 0, Money::from_dong(10_000)).unwrap();
        let second = refund_split(captured, fee, 10_000, Money::from_dong(15_500)).unwrap();
        assert_eq!(first, (2_000, 8_000));
        assert_eq!(second, (3_100, 12_400));
        assert_eq!(first.0 + second.0, 5_100);
        assert_eq!(first.1 + second.1, 20_400);
    }
}
