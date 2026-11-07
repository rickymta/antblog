//! Khuyến mãi và trợ giá thành bút toán. Promotion service (C#) quyết định mức giảm và ai chịu;
//! module này chỉ nhận con số đã quyết, kiểm chúng khớp tiền thật rồi ghi.

use ledger_core::{
    AccountId, EntryId, FeePolicy, JournalEntry, JournalEntryBuilder, Money, default_policy,
};

use crate::store::{BudgetGuard, BudgetSpend, PgLedger, PostError, Posted, budget_account};

/// Tài khoản chi phí khuyến mãi (expense).
pub const PROMO_EXPENSE: &str = "platform:promo-expense";
/// Tài khoản doanh thu nền tảng.
pub const REVENUE: &str = "platform:revenue";
/// Quỹ của nền tảng (asset), nguồn cấp ngân sách campaign.
pub const TREASURY: &str = "platform:treasury";
/// Ví khuyến mãi của nền tảng (liability, cùng vai với ví khách): trả thay khách phần nền tảng chịu.
pub const PROMO_WALLET: &str = "platform:promo-wallet";
/// Tài khoản tiền VíXanh đang giữ.
pub const CLEARING: &str = "psp:vixanh:clearing";

/// Ai chịu mức giảm: Promotion C# gửi sang dưới dạng hai con số, không phải một chính sách.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shares {
    /// Phần nền tảng chịu (tiêu ngân sách campaign, ghi chi phí, trả thay khách).
    pub platform: Money,
    /// Phần tài xế chịu (trừ vào thu nhập tài xế, không đụng ngân sách).
    pub driver: Money,
}

/// Cách chia mức giảm, dùng để dựng bảng so sánh trong lab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Funding {
    /// Nền tảng chịu toàn bộ.
    Platform,
    /// Tài xế chịu toàn bộ.
    Driver,
    /// Nền tảng chịu `platform`, tài xế chịu phần còn lại.
    Shared {
        /// Phần nền tảng chịu.
        platform: Money,
    },
}

impl Shares {
    /// Không giảm giá.
    pub const NONE: Shares = Shares {
        platform: Money::ZERO,
        driver: Money::ZERO,
    };

    /// Tách `discount` theo `funding`. Lỗi nếu phần nền tảng âm hoặc vượt mức giảm.
    pub fn from_funding(discount: Money, funding: Funding) -> Result<Shares, PostError> {
        let platform = match funding {
            Funding::Platform => discount,
            Funding::Driver => Money::ZERO,
            Funding::Shared { platform } => platform,
        };
        if platform < Money::ZERO || platform > discount {
            return Err(PostError::InvalidShares(format!(
                "phần nền tảng {platform} nằm ngoài [0, {discount}]"
            )));
        }
        let driver = discount
            .checked_sub(platform)
            .ok_or(PostError::Ledger(ledger_core::LedgerError::Overflow))?;
        Ok(Shares { platform, driver })
    }

    /// Tổng mức giảm khách được hưởng. Lỗi nếu một phần âm hoặc cộng tràn số.
    pub fn discount(&self) -> Result<Money, PostError> {
        if self.platform < Money::ZERO || self.driver < Money::ZERO {
            return Err(PostError::InvalidShares(format!(
                "phần chịu âm: nền tảng {}, tài xế {}",
                self.platform, self.driver
            )));
        }
        self.platform
            .checked_add(self.driver)
            .ok_or(PostError::Ledger(ledger_core::LedgerError::Overflow))
    }
}

/// Quyết định của Promotion cho một chuyến. Ledger không tự tính, không tự sửa.
#[derive(Debug, Clone)]
pub struct PromoGrant {
    /// Mã chuyến, ví dụ `TN-0002`.
    pub trip: String,
    /// Mã campaign, ví dụ `WELCOME5K`.
    pub campaign: String,
    /// Ai chịu bao nhiêu.
    pub shares: Shares,
}

fn acct(raw: &str) -> Result<AccountId, PostError> {
    Ok(AccountId::new(raw)?)
}

/// Cấp ngân sách campaign từ quỹ nền tảng: asset sang asset, doanh thu không đổi.
pub fn fund_entry(campaign: &str, cap: Money) -> Result<JournalEntry, PostError> {
    let out = cap
        .checked_neg()
        .ok_or(ledger_core::LedgerError::Overflow)?;
    Ok(
        JournalEntryBuilder::new(&format!("campaign:{campaign}:fund"))
            .posting(acct(TREASURY)?, out)
            .posting(acct(&budget_account(campaign))?, cap)
            .build()?,
    )
}

/// Thu tiền khách đúng số VíXanh thực thu (đã trừ phần giảm).
pub fn capture_entry(
    trip: &str,
    customer: &str,
    charged: Money,
) -> Result<JournalEntry, PostError> {
    let out = charged
        .checked_neg()
        .ok_or(ledger_core::LedgerError::Overflow)?;
    Ok(JournalEntryBuilder::new(&format!("trip:{trip}:capture"))
        .posting(acct(&format!("customer:{customer}"))?, out)
        .posting(acct(CLEARING)?, charged)
        .build()?)
}

/// Dùng ngân sách cho phần nền tảng chịu: ngân sách ra, chi phí khuyến mãi vào.
pub fn redeem_entry(grant: &PromoGrant) -> Result<JournalEntry, PostError> {
    grant.shares.discount()?; // từ chối phần chịu âm trước khi đụng ngân sách
    let amount = grant.shares.platform;
    let out = amount
        .checked_neg()
        .ok_or(ledger_core::LedgerError::Overflow)?;
    Ok(
        JournalEntryBuilder::new(&format!("trip:{}:promo:{}", grant.trip, grant.campaign))
            .posting(acct(&budget_account(&grant.campaign))?, out)
            .posting(acct(PROMO_EXPENSE)?, amount)
            .build()?,
    )
}

/// Chia tiền chuyến: phí tính trên cước gốc. Quỹ chuyến gồm tiền VíXanh đã thu và phần nền tảng
/// trả thay từ ví khuyến mãi; tài xế chỉ bị trừ phần mình chịu, doanh thu luôn là phí đủ.
pub fn split_entry(
    trip: &str,
    driver: &str,
    fare: Money,
    shares: Shares,
) -> Result<JournalEntry, PostError> {
    let fee = default_policy().split(fare)?; // 20% của cước GỐC, không phải của số khách trả
    let discount = shares.discount()?;
    let collected = fare
        .checked_sub(discount)
        .ok_or(ledger_core::LedgerError::Overflow)?;
    if collected < Money::ZERO {
        return Err(PostError::InvalidShares(format!(
            "mức giảm {discount} vượt cước {fare}"
        )));
    }
    let driver_amount = fee
        .driver_net
        .checked_sub(shares.driver)
        .ok_or(ledger_core::LedgerError::Overflow)?;
    if driver_amount < Money::ZERO {
        return Err(PostError::InvalidShares(format!(
            "tài xế chịu {} nhưng chỉ có {}",
            shares.driver, fee.driver_net
        )));
    }
    let out = collected
        .checked_neg()
        .ok_or(ledger_core::LedgerError::Overflow)?;
    let paid_for_customer = shares
        .platform
        .checked_neg()
        .ok_or(ledger_core::LedgerError::Overflow)?;
    // Posting bằng 0 không có nghĩa (và cột `amount_dong` cấm 0): bỏ.
    let mut b = JournalEntryBuilder::new(&format!("trip:{trip}:split"));
    for (account, amount) in [
        (CLEARING.to_string(), out),
        (PROMO_WALLET.to_string(), paid_for_customer),
        (format!("driver:{driver}"), driver_amount),
        (REVENUE.to_string(), fee.platform_fee),
    ] {
        if amount != Money::ZERO {
            b = b.posting(acct(&account)?, amount);
        }
    }
    Ok(b.build()?)
}

/// Trợ giá tài xế: ngân sách ra, chi phí vào, ví khuyến mãi trả, tài xế nhận thêm. 4 posting,
/// doanh thu không đổi.
pub fn subsidy_entry(
    trip: &str,
    campaign: &str,
    driver: &str,
    amount: Money,
) -> Result<JournalEntry, PostError> {
    let out = amount
        .checked_neg()
        .ok_or(ledger_core::LedgerError::Overflow)?;
    Ok(
        JournalEntryBuilder::new(&format!("trip:{trip}:subsidy:{campaign}"))
            .posting(acct(&budget_account(campaign))?, out)
            .posting(acct(PROMO_EXPENSE)?, amount)
            .posting(acct(PROMO_WALLET)?, out)
            .posting(acct(&format!("driver:{driver}"))?, amount)
            .build()?,
    )
}

impl PgLedger {
    /// Mở các tài khoản của một chuyến (khách, tài xế) và tài khoản dùng chung. Gọi lại không sao.
    pub async fn open_trip_accounts(
        &self,
        customer: &str,
        driver: &str,
        campaign: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        self.ensure_account(&format!("customer:{customer}"), "liability")
            .await?;
        self.ensure_account(&format!("driver:{driver}"), "liability")
            .await?;
        self.ensure_account(REVENUE, "revenue").await?;
        self.ensure_account(PROMO_EXPENSE, "expense").await?;
        self.ensure_account(PROMO_WALLET, "liability").await?;
        self.ensure_account(CLEARING, "asset").await?;
        if let Some(c) = campaign {
            self.ensure_account(&budget_account(c), "asset").await?;
        }
        Ok(())
    }

    /// Cấp ngân sách campaign.
    pub async fn fund_campaign(&self, campaign: &str, cap: Money) -> Result<Posted, PostError> {
        self.ensure_account(TREASURY, "asset").await?;
        self.ensure_account(&budget_account(campaign), "asset")
            .await?;
        self.post(&fund_entry(campaign, cap)?).await
    }

    /// Ghi bút toán thu tiền khách.
    pub async fn capture(
        &self,
        trip: &str,
        customer: &str,
        charged: Money,
    ) -> Result<Posted, PostError> {
        self.post(&capture_entry(trip, customer, charged)?).await
    }

    /// Dùng ngân sách cho phần nền tảng chịu. `None` nếu nền tảng không chịu đồng nào.
    pub async fn redeem(
        &self,
        grant: &PromoGrant,
        guard: &BudgetGuard,
    ) -> Result<Option<Posted>, PostError> {
        if grant.shares.platform == Money::ZERO {
            return Ok(None);
        }
        let spend = BudgetSpend {
            campaign: grant.campaign.clone(),
            amount: grant.shares.platform,
            guard: guard.clone(),
        };
        Ok(Some(
            self.post_with(&redeem_entry(grant)?, Some(&spend)).await?,
        ))
    }

    /// Chia tiền chuyến sau khi đã capture. Kiểm số VíXanh thu = cước − mức giảm, và phần nền tảng
    /// chịu đã có bút toán redeem, trước khi ghi.
    pub async fn settle(
        &self,
        trip: &str,
        driver: &str,
        fare: Money,
        shares: Shares,
    ) -> Result<Posted, PostError> {
        let captured = self
            .captured_amount(trip)
            .await?
            .ok_or_else(|| PostError::NotCaptured {
                trip: trip.to_string(),
            })?;
        let discount = shares.discount()?;
        let expected = fare
            .checked_sub(discount)
            .ok_or(ledger_core::LedgerError::Overflow)?;
        if captured != expected {
            return Err(PostError::AmountMismatch {
                fare,
                discount,
                expected,
                captured,
            });
        }
        // Ví khuyến mãi chỉ trả thay đúng phần đã tiêu ngân sách, nếu không chi phí và ví lệch nhau.
        let redeemed = self.redeemed_amount(trip).await?;
        if redeemed != shares.platform {
            return Err(PostError::NotRedeemed {
                trip: trip.to_string(),
                expected: shares.platform,
                redeemed,
            });
        }
        self.post(&split_entry(trip, driver, fare, shares)?).await
    }

    /// Trợ giá tài xế từ ngân sách campaign.
    pub async fn subsidize(
        &self,
        trip: &str,
        campaign: &str,
        driver: &str,
        amount: Money,
        guard: &BudgetGuard,
    ) -> Result<Posted, PostError> {
        let spend = BudgetSpend {
            campaign: campaign.to_string(),
            amount,
            guard: guard.clone(),
        };
        self.post_with(
            &subsidy_entry(trip, campaign, driver, amount)?,
            Some(&spend),
        )
        .await
    }

    /// Hoàn chuyến: đảo mọi bút toán gốc của chuyến, mới nhất trước. Ngân sách được trả lại
    /// và chi phí được đảo vì bút toán `promo`/`subsidy` cũng bị đảo. Gọi lại là replay.
    pub async fn reverse_trip(&self, trip: &str) -> Result<Vec<Posted>, PostError> {
        let mut out = Vec::new();
        for (id, key) in self.trip_entries(trip).await? {
            let postings = self
                .entry_by_key(&key)
                .await?
                .unwrap_or_default()
                .into_iter()
                .map(|(a, d)| Ok(ledger_core::Posting::new(acct(&a)?, Money::from_dong(d))))
                .collect::<Result<Vec<_>, PostError>>()?;
            let original = JournalEntry::try_new(&key, postings)?;
            let entry_id = EntryId::parse(&format!("JE-{id:06}")).ok_or_else(|| {
                PostError::InvalidShares(format!("mã bút toán {id} không hợp lệ"))
            })?;
            let reversal = JournalEntry::reversal_of(entry_id, &original)?;
            out.push(self.post(&reversal).await?); // bút toán đảo không đi qua kiểm ngân sách
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dong(n: i64) -> Money {
        Money::from_dong(n)
    }

    fn sums(entry: &JournalEntry) -> Vec<(String, i64)> {
        entry
            .postings()
            .iter()
            .map(|p| (p.account.as_str().to_string(), p.amount.dong()))
            .collect()
    }

    #[test]
    fn platform_funded_split_charges_fee_on_gross_fare() {
        let shares = Shares::from_funding(dong(5_000), Funding::Platform).unwrap();
        let entry = split_entry("TN-0002", "DR-002", dong(25_500), shares).unwrap();
        assert_eq!(
            sums(&entry),
            vec![
                (CLEARING.to_string(), -20_500),
                (PROMO_WALLET.to_string(), -5_000),
                ("driver:DR-002".to_string(), 20_400),
                (REVENUE.to_string(), 5_100),
            ]
        );
    }

    #[test]
    fn driver_funded_discount_lowers_driver_not_platform() {
        let shares = Shares::from_funding(dong(5_000), Funding::Driver).unwrap();
        let entry = split_entry("TN-0102", "DR-102", dong(25_500), shares).unwrap();
        assert_eq!(
            sums(&entry),
            vec![
                (CLEARING.to_string(), -20_500),
                ("driver:DR-102".to_string(), 15_400),
                (REVENUE.to_string(), 5_100),
            ]
        );
    }

    #[test]
    fn negative_or_oversized_shares_are_rejected_when_splitting() {
        let negative = Shares {
            platform: Money::ZERO,
            driver: dong(-1),
        };
        let err = split_entry("TN-0002", "DR-002", dong(25_500), negative);
        assert!(matches!(err, Err(PostError::InvalidShares(_))));
        let oversized = Shares {
            platform: dong(25_501),
            driver: Money::ZERO,
        };
        let err = split_entry("TN-0002", "DR-002", dong(25_500), oversized);
        assert!(matches!(err, Err(PostError::InvalidShares(_))));
    }

    #[test]
    fn shares_reject_a_platform_part_larger_than_the_discount() {
        let err = Shares::from_funding(
            dong(5_000),
            Funding::Shared {
                platform: dong(6_000),
            },
        );
        assert!(matches!(err, Err(PostError::InvalidShares(_))));
        let err = Shares::from_funding(dong(5_000), Funding::Shared { platform: dong(-1) });
        assert!(matches!(err, Err(PostError::InvalidShares(_))));
    }

    #[test]
    fn driver_cannot_bear_more_than_the_driver_net() {
        let shares = Shares {
            platform: Money::ZERO,
            driver: dong(21_000),
        };
        let err = split_entry("TN-0002", "DR-002", dong(25_500), shares);
        assert!(matches!(err, Err(PostError::InvalidShares(_))));
    }

    #[test]
    fn subsidy_entry_has_four_postings_and_balances() {
        let entry = subsidy_entry("TN-0003", "PEAKDRV", "DR-003", dong(3_000)).unwrap();
        assert_eq!(entry.postings().len(), 4);
        let total: i64 = entry.postings().iter().map(|p| p.amount.dong()).sum();
        assert_eq!(total, 0);
    }

    #[test]
    fn full_discount_drops_the_zero_clearing_posting() {
        let shares = Shares::from_funding(dong(25_500), Funding::Platform).unwrap();
        let entry = split_entry("TN-0009", "DR-009", dong(25_500), shares).unwrap();
        // VíXanh không thu đồng nào nên không có posting clearing; ví khuyến mãi trả cả cước.
        assert_eq!(
            sums(&entry),
            vec![
                (PROMO_WALLET.to_string(), -25_500),
                ("driver:DR-009".to_string(), 20_400),
                (REVENUE.to_string(), 5_100),
            ]
        );
    }
}
