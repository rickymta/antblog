//! Aggregate `Account`: ranh giới nhất quán của một ví. Invariant nằm ở đây, không nằm ở SQL.
//!
//! Hai invariant của ví:
//! 1. `balance >= 0` sau mọi lần rút hay chuyển đi;
//! 2. `withdrawn <= withdraw_limit`: tổng đã rút trong kỳ không vượt hạn mức.
//!
//! Trường là private và `balance`/`withdrawn` chỉ đổi qua [`Account::withdraw`], [`Account::transfer_to`]
//! và [`Account::receive`], nên không dựng được `Account` vi phạm invariant bằng code thường.

use ledger_core::{AccountId, JournalEntry, LedgerError, Money, Posting};
use thiserror::Error;

/// Tài khoản đối ứng khi ví rút tiền ra ngân hàng.
pub const BANK_PAYOUT: &str = "bank:payout:clearing";

/// Lý do aggregate từ chối. Đều là lỗi nghiệp vụ: thử lại không làm chúng hết.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AccountError {
    /// Số tiền không dương.
    #[error("số tiền phải dương, nhận {0}")]
    NotPositive(Money),
    /// Rút quá số dư.
    #[error("không đủ số dư: có {balance}, cần {requested}")]
    InsufficientBalance {
        /// Số dư hiện có.
        balance: Money,
        /// Số xin rút.
        requested: Money,
    },
    /// Rút quá hạn mức trong kỳ.
    #[error("vượt hạn mức rút: đã rút {withdrawn}, xin thêm {requested}, hạn mức {limit}")]
    LimitExceeded {
        /// Đã rút trong kỳ.
        withdrawn: Money,
        /// Số xin rút.
        requested: Money,
        /// Hạn mức của kỳ.
        limit: Money,
    },
    /// Phép cộng làm tràn `i64`.
    #[error("tiền vượt giới hạn i64")]
    Overflow,
    /// Dữ liệu nạp từ DB đã vi phạm invariant (không được xảy ra nếu mọi đường ghi đi qua aggregate).
    #[error("trạng thái nạp từ DB không hợp lệ: {0}")]
    CorruptState(&'static str),
    /// Bút toán dựng ra không hợp lệ.
    #[error(transparent)]
    Entry(#[from] LedgerError),
}

/// Aggregate ví. `version` là phiên bản lúc nạp, dùng làm điều kiện `UPDATE ... WHERE version = $n`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    id: AccountId,
    balance: Money,
    withdraw_limit: Money,
    withdrawn: Money,
    version: i64,
}

impl Account {
    /// Dựng lại từ dữ liệu đã lưu, kiểm lại invariant (dữ liệu hỏng thì không nạp).
    ///
    /// # Errors
    /// [`AccountError::CorruptState`] nếu số dư hoặc số đã rút âm, hoặc đã rút quá hạn mức.
    pub fn restore(
        id: AccountId,
        balance: Money,
        withdraw_limit: Money,
        withdrawn: Money,
        version: i64,
    ) -> Result<Account, AccountError> {
        if balance < Money::ZERO {
            return Err(AccountError::CorruptState("số dư âm"));
        }
        if withdrawn < Money::ZERO || withdraw_limit < Money::ZERO {
            return Err(AccountError::CorruptState("số đã rút hoặc hạn mức âm"));
        }
        if withdrawn > withdraw_limit {
            return Err(AccountError::CorruptState("đã rút vượt hạn mức"));
        }
        Ok(Account {
            id,
            balance,
            withdraw_limit,
            withdrawn,
            version,
        })
    }

    /// Mã tài khoản.
    pub fn id(&self) -> &AccountId {
        &self.id
    }

    /// Số dư hiện có (theo bản đã nạp, cộng các thay đổi chưa lưu).
    pub fn balance(&self) -> Money {
        self.balance
    }

    /// Hạn mức rút của kỳ.
    pub fn withdraw_limit(&self) -> Money {
        self.withdraw_limit
    }

    /// Đã rút trong kỳ.
    pub fn withdrawn(&self) -> Money {
        self.withdrawn
    }

    /// Phiên bản lúc nạp.
    pub fn version(&self) -> i64 {
        self.version
    }

    /// Kiểm và trừ `amount` khỏi số dư, cộng vào số đã rút. Chỉ đổi trạng thái khi mọi kiểm tra qua.
    fn spend(&mut self, amount: Money) -> Result<(), AccountError> {
        if amount <= Money::ZERO {
            return Err(AccountError::NotPositive(amount));
        }
        let balance = self
            .balance
            .checked_sub(amount)
            .filter(|left| *left >= Money::ZERO)
            .ok_or(AccountError::InsufficientBalance {
                balance: self.balance,
                requested: amount,
            })?;
        let withdrawn = self
            .withdrawn
            .checked_add(amount)
            .filter(|total| *total <= self.withdraw_limit)
            .ok_or(AccountError::LimitExceeded {
                withdrawn: self.withdrawn,
                requested: amount,
                limit: self.withdraw_limit,
            })?;
        self.balance = balance;
        self.withdrawn = withdrawn;
        Ok(())
    }

    /// Rút `amount` ra ngân hàng. Trả bút toán cần ghi: ví −`amount`, `bank:payout:clearing` +`amount`.
    ///
    /// # Errors
    /// [`AccountError::NotPositive`], [`AccountError::InsufficientBalance`] hoặc
    /// [`AccountError::LimitExceeded`]; aggregate không đổi khi lỗi.
    pub fn withdraw(&mut self, amount: Money, key: &str) -> Result<JournalEntry, AccountError> {
        self.spend(amount)?;
        let out = amount.checked_neg().ok_or(AccountError::Overflow)?;
        Ok(JournalEntry::try_new(
            key,
            vec![
                Posting::new(self.id.clone(), out),
                Posting::new(AccountId::new(BANK_PAYOUT)?, amount),
            ],
        )?)
    }

    /// Nhận `amount` từ ví khác (phía nhận của một lần chuyển).
    ///
    /// # Errors
    /// [`AccountError::NotPositive`] hoặc [`AccountError::Overflow`].
    pub fn receive(&mut self, amount: Money) -> Result<(), AccountError> {
        if amount <= Money::ZERO {
            return Err(AccountError::NotPositive(amount));
        }
        self.balance = self
            .balance
            .checked_add(amount)
            .ok_or(AccountError::Overflow)?;
        Ok(())
    }

    /// Chuyển `amount` sang ví `to`: hai aggregate đổi trong cùng một bút toán.
    /// Quy tắc của lab: tiền chuyển đi cũng tính vào hạn mức rút của ví nguồn.
    ///
    /// # Errors
    /// Như [`Account::withdraw`] và [`Account::receive`]; khi lỗi, cả hai ví đều không đổi.
    pub fn transfer_to(
        &mut self,
        to: &mut Account,
        amount: Money,
        key: &str,
    ) -> Result<JournalEntry, AccountError> {
        let mut source = self.clone();
        let mut target = to.clone();
        source.spend(amount)?;
        target.receive(amount)?;
        let out = amount.checked_neg().ok_or(AccountError::Overflow)?;
        let entry = JournalEntry::try_new(
            key,
            vec![
                Posting::new(self.id.clone(), out),
                Posting::new(to.id.clone(), amount),
            ],
        )?;
        *self = source;
        *to = target;
        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn money(dong: i64) -> Money {
        Money::from_dong(dong)
    }

    fn account(balance: i64, limit: i64, withdrawn: i64) -> Account {
        Account::restore(
            AccountId::new("driver:DR-001").unwrap(),
            money(balance),
            money(limit),
            money(withdrawn),
            0,
        )
        .unwrap()
    }

    #[test]
    fn withdraw_updates_balance_and_withdrawn_and_posts_a_balanced_entry() {
        let mut acc = account(150_000, 100_000, 0);
        let entry = acc.withdraw(money(20_400), "withdraw:DR-001:1").unwrap();
        assert_eq!(acc.balance(), money(129_600));
        assert_eq!(acc.withdrawn(), money(20_400));
        let sum: i64 = entry.postings().iter().map(|p| p.amount.dong()).sum();
        assert_eq!(sum, 0);
    }

    #[test]
    fn limit_is_inclusive_and_one_dong_more_is_rejected() {
        let mut acc = account(150_000, 100_000, 79_600);
        assert!(acc.withdraw(money(20_400), "k1").is_ok());
        assert_eq!(acc.withdrawn(), money(100_000));
        let mut acc = account(150_000, 100_000, 79_600);
        let err = acc.withdraw(money(20_401), "k2").unwrap_err();
        assert!(matches!(err, AccountError::LimitExceeded { .. }));
        assert_eq!(acc.withdrawn(), money(79_600), "lỗi thì không đổi gì");
    }

    #[test]
    fn cannot_withdraw_more_than_balance() {
        let mut acc = account(10_000, 100_000, 0);
        let err = acc.withdraw(money(10_001), "k").unwrap_err();
        assert!(matches!(err, AccountError::InsufficientBalance { .. }));
        assert_eq!(acc.balance(), money(10_000));
    }

    #[test]
    fn rejects_non_positive_amounts() {
        let mut acc = account(10_000, 100_000, 0);
        assert!(matches!(
            acc.withdraw(money(0), "k"),
            Err(AccountError::NotPositive(_))
        ));
        assert!(matches!(
            acc.withdraw(money(-5), "k"),
            Err(AccountError::NotPositive(_))
        ));
        assert!(matches!(
            acc.receive(money(-5)),
            Err(AccountError::NotPositive(_))
        ));
    }

    #[test]
    fn transfer_is_all_or_nothing() {
        let mut a = account(30_000, 100_000, 0);
        let mut b = Account::restore(
            AccountId::new("customer:CU-002").unwrap(),
            money(i64::MAX - 10),
            money(1_000),
            money(0),
            0,
        )
        .unwrap();
        // Phía nhận tràn số: ví nguồn không được trừ.
        let err = a.transfer_to(&mut b, money(20), "k").unwrap_err();
        assert_eq!(err, AccountError::Overflow);
        assert_eq!(a.balance(), money(30_000));
        assert_eq!(a.withdrawn(), money(0));
    }

    #[test]
    fn corrupt_state_is_not_loaded() {
        let id = AccountId::new("driver:DR-001").unwrap();
        assert!(Account::restore(id.clone(), money(-1), money(10), money(0), 0).is_err());
        assert!(Account::restore(id, money(5), money(10), money(11), 0).is_err());
    }
}
