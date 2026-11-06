//! Chart of accounts rút gọn: loại của từng tài khoản, chỉ để đọc số dư và lập báo cáo.
//! Loại tài khoản KHÔNG tham gia kiểm cân; bất biến chỉ nhìn số tiền có dấu.
//! Dấu của posting là chiều tiền, không phải nợ/có: kế toán ghi tài sản tăng bên nợ,
//! nợ phải trả và doanh thu tăng bên có.

use crate::account::AccountId;

/// Loại tài khoản trong chart of accounts của lab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountKind {
    /// Tài sản: tiền nền tảng đang được nợ hoặc đang giữ (vd. tiền chờ về từ VíXanh).
    Asset,
    /// Nợ phải trả: nền tảng nợ người khác (vd. ví khách, tiền chờ trả tài xế).
    Liability,
    /// Doanh thu: phần nền tảng đã kiếm (phí 20%).
    Revenue,
}

impl AccountKind {
    pub fn label(self) -> &'static str {
        match self {
            AccountKind::Asset => "asset",
            AccountKind::Liability => "liability",
            AccountKind::Revenue => "revenue",
        }
    }
}

/// Tra loại tài khoản theo tiền tố mã; `None` nếu mã không thuộc chart rút gọn.
pub fn kind_of(account: &AccountId) -> Option<AccountKind> {
    let id = account.as_str();
    if id.starts_with("customer:") || id.starts_with("driver:") {
        Some(AccountKind::Liability)
    } else if id == "platform:revenue" {
        Some(AccountKind::Revenue)
    } else if id.starts_with("psp:") {
        Some(AccountKind::Asset)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind(raw: &str) -> Option<AccountKind> {
        kind_of(&AccountId::new(raw).unwrap())
    }

    #[test]
    fn chart_covers_the_four_accounts() {
        assert_eq!(kind("customer:CU-001"), Some(AccountKind::Liability));
        assert_eq!(kind("driver:DR-001"), Some(AccountKind::Liability));
        assert_eq!(kind("platform:revenue"), Some(AccountKind::Revenue));
        assert_eq!(kind("psp:vixanh:clearing"), Some(AccountKind::Asset));
    }

    #[test]
    fn unknown_prefix_has_no_kind() {
        assert_eq!(kind("bank:xyz"), None);
    }
}
