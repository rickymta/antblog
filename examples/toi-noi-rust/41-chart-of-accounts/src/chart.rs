//! Chart of accounts: loại của từng tài khoản và cách đọc số dư theo loại.
//!
//! Loại tài khoản KHÔNG tham gia kiểm cân: bất biến của sổ chỉ nhìn số tiền có dấu.
//! Posting vẫn là dòng tiền (`Flow` ở bài 13). `BalanceSide` chỉ dùng để đọc SỐ DƯ.

use ledger_core::AccountId;

/// Bốn loại tài khoản của chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountKind {
    /// Tiền hoặc quyền đòi tiền mà nền tảng đang giữ.
    Asset,
    /// Nền tảng nợ người giữ ví hoặc nhà nước (thuế giả định).
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

    /// Đọc lại từ cột `accounts.kind`.
    pub fn parse(raw: &str) -> Option<AccountKind> {
        match raw {
            "asset" => Some(AccountKind::Asset),
            "liability" => Some(AccountKind::Liability),
            "revenue" => Some(AccountKind::Revenue),
            "expense" => Some(AccountKind::Expense),
            _ => None,
        }
    }
}

/// Tám mẫu tài khoản của sơ đồ chuẩn. Mã không khớp mẫu nào thì không mở được.
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

/// Số dư nằm ở phía nào khi đọc theo ngôn ngữ kế toán.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BalanceSide {
    /// Số dư bên nợ.
    Debit,
    /// Số dư bên có.
    Credit,
}

/// Đọc một số dư (dương là tiền vào ròng) theo loại tài khoản. `None` khi số dư bằng 0.
///
/// Asset và expense: dương là bên nợ. Liability và revenue: dương là bên có.
pub fn side_of_balance(kind: AccountKind, balance_dong: i64) -> Option<BalanceSide> {
    let positive_is_debit = matches!(kind, AccountKind::Asset | AccountKind::Expense);
    match (balance_dong.signum(), positive_is_debit) {
        (0, _) => None,
        (1, true) | (-1, false) => Some(BalanceSide::Debit),
        _ => Some(BalanceSide::Credit),
    }
}

/// Bảng đọc số dư: mỗi loại một dòng.
pub struct Reading {
    /// Loại tài khoản.
    pub kind: AccountKind,
    /// Số dư dương nghĩa là gì.
    pub positive: &'static str,
    /// Số dư âm nghĩa là gì.
    pub negative: &'static str,
    /// Số dư bình thường nằm bên nào trong ngôn ngữ kế toán.
    pub normal_side: &'static str,
}

/// Bốn dòng của bảng ánh xạ loại tài khoản sang ngôn ngữ kế toán.
pub const READINGS: [Reading; 4] = [
    Reading {
        kind: AccountKind::Asset,
        positive: "tiền hoặc quyền đòi tiền nền tảng đang giữ",
        negative: "bất thường: đã trả ra nhiều hơn đã nhận (clearing có ngoại lệ, xem bài)",
        normal_side: "bên nợ",
    },
    Reading {
        kind: AccountKind::Liability,
        positive: "nền tảng nợ người giữ ví",
        negative: "người giữ ví nợ nền tảng",
        normal_side: "bên có",
    },
    Reading {
        kind: AccountKind::Revenue,
        positive: "phần nền tảng đã kiếm",
        negative: "bất thường: hoàn nhiều hơn đã kiếm",
        normal_side: "bên có",
    },
    Reading {
        kind: AccountKind::Expense,
        positive: "chi phí đã phát sinh",
        negative: "bất thường: hoàn nhiều hơn đã chi",
        normal_side: "bên nợ",
    },
];

/// Cộng số dư theo phía: `(tổng bên nợ, tổng bên có)`, đọc theo loại từng tài khoản.
/// Không đúng với tài khoản biên âm của mô hình nạp ví: `psp:vixanh:clearing` -40.000 đọc thành
/// bên có, trong khi kế toán ghi tài sản tăng bên nợ.
pub fn trial_totals(balances: &[(AccountKind, i64)]) -> (i64, i64) {
    let mut debit = 0;
    let mut credit = 0;
    for (kind, balance) in balances {
        match side_of_balance(*kind, *balance) {
            Some(BalanceSide::Debit) => debit += balance.abs(),
            Some(BalanceSide::Credit) => credit += balance.abs(),
            None => {}
        }
    }
    (debit, credit)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind(raw: &str) -> Option<AccountKind> {
        kind_of(&AccountId::new(raw).unwrap())
    }

    #[test]
    fn chart_has_eight_patterns() {
        assert_eq!(kind("customer:CU-001"), Some(AccountKind::Liability));
        assert_eq!(kind("driver:DR-001"), Some(AccountKind::Liability));
        assert_eq!(kind("platform:revenue"), Some(AccountKind::Revenue));
        assert_eq!(kind("platform:promo-expense"), Some(AccountKind::Expense));
        assert_eq!(kind("promo:budget:SUMMER"), Some(AccountKind::Asset));
        assert_eq!(kind("psp:vixanh:clearing"), Some(AccountKind::Asset));
        assert_eq!(kind("bank:payout:clearing"), Some(AccountKind::Asset));
        assert_eq!(kind("tax:vat:payable"), Some(AccountKind::Liability));
    }

    #[test]
    fn unknown_or_malformed_codes_have_no_kind() {
        assert_eq!(kind("wallet:CU-001"), None);
        assert_eq!(kind("customer:"), None);
        assert_eq!(kind("customer:CU-001:extra"), None);
        assert_eq!(kind("promo:budget:"), None);
        assert_eq!(kind("psp:other:clearing"), None);
    }

    #[test]
    fn kind_round_trips_through_text() {
        for r in &READINGS {
            assert_eq!(AccountKind::parse(r.kind.as_str()), Some(r.kind));
        }
        assert_eq!(AccountKind::parse("equity"), None);
    }

    #[test]
    fn side_depends_on_kind_and_sign() {
        use AccountKind::*;
        assert_eq!(side_of_balance(Asset, 25_500), Some(BalanceSide::Debit));
        assert_eq!(
            side_of_balance(Liability, 20_400),
            Some(BalanceSide::Credit)
        );
        assert_eq!(
            side_of_balance(Liability, -25_500),
            Some(BalanceSide::Debit)
        );
        assert_eq!(side_of_balance(Revenue, 5_100), Some(BalanceSide::Credit));
        assert_eq!(side_of_balance(Expense, 5_000), Some(BalanceSide::Debit));
        assert_eq!(side_of_balance(Expense, -1), Some(BalanceSide::Credit));
        assert_eq!(side_of_balance(Asset, 0), None);
    }

    #[test]
    fn trial_totals_balance_only_when_clearing_is_empty() {
        use AccountKind::*;
        // Sau bước 1: tiền nằm ở clearing, khách âm 25.500.
        assert_eq!(
            trial_totals(&[(Liability, -25_500), (Asset, 25_500)]),
            (51_000, 0)
        );
        // Sau bước 2: clearing về 0.
        assert_eq!(
            trial_totals(&[
                (Liability, -25_500),
                (Asset, 0),
                (Revenue, 5_100),
                (Liability, 20_400)
            ]),
            (25_500, 25_500)
        );
    }
}
