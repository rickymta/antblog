//! Sơ đồ tài khoản chuẩn của loạt (bài 41). Mã không khớp mẫu nào thì không mở được.

use ledger_core::{AccountId, LedgerError};

/// Tiền VíXanh đang giữ, chờ chia.
pub const CLEARING: &str = "psp:vixanh:clearing";
/// Doanh thu nền tảng.
pub const REVENUE: &str = "platform:revenue";

/// Bốn loại tài khoản.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountKind {
    /// Tiền hoặc quyền đòi tiền nền tảng đang giữ.
    Asset,
    /// Nền tảng nợ người giữ ví.
    Liability,
    /// Phần nền tảng đã kiếm.
    Revenue,
    /// Chi phí đã phát sinh.
    Expense,
}

impl AccountKind {
    /// Giá trị lưu ở cột `accounts.kind`.
    pub fn as_str(self) -> &'static str {
        match self {
            AccountKind::Asset => "asset",
            AccountKind::Liability => "liability",
            AccountKind::Revenue => "revenue",
            AccountKind::Expense => "expense",
        }
    }
}

/// Tám mẫu tài khoản của sơ đồ chuẩn.
pub fn kind_of(account: &AccountId) -> Option<AccountKind> {
    let parts: Vec<&str> = account.as_str().split(':').collect();
    match parts.as_slice() {
        ["customer" | "driver", who] if !who.is_empty() => Some(AccountKind::Liability),
        ["platform", "revenue"] => Some(AccountKind::Revenue),
        ["platform", "promo-expense"] => Some(AccountKind::Expense),
        ["promo", "budget", campaign] if !campaign.is_empty() => Some(AccountKind::Asset),
        ["psp", "vixanh", "clearing"] | ["bank", "payout", "clearing"] => Some(AccountKind::Asset),
        ["tax", "vat", "payable"] => Some(AccountKind::Liability),
        _ => None,
    }
}

/// `customer:<id>`.
///
/// # Errors
/// [`LedgerError::EmptyAccountId`] nếu `id` rỗng.
pub fn customer(id: &str) -> Result<AccountId, LedgerError> {
    AccountId::new(&format!("customer:{id}"))
}

/// `driver:<id>`.
///
/// # Errors
/// [`LedgerError::EmptyAccountId`] nếu `id` rỗng.
pub fn driver(id: &str) -> Result<AccountId, LedgerError> {
    AccountId::new(&format!("driver:{id}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind(raw: &str) -> Option<AccountKind> {
        kind_of(&AccountId::new(raw).unwrap())
    }

    #[test]
    fn standard_accounts_have_kinds() {
        assert_eq!(kind("customer:CU-001"), Some(AccountKind::Liability));
        assert_eq!(kind(REVENUE), Some(AccountKind::Revenue));
        assert_eq!(kind(CLEARING), Some(AccountKind::Asset));
        assert_eq!(kind("promo:budget:T11"), Some(AccountKind::Asset));
    }

    #[test]
    fn unknown_patterns_are_rejected() {
        assert_eq!(kind("lạ:1"), None);
        assert_eq!(kind("customer:"), None);
        assert_eq!(kind("platform:other"), None);
    }
}
