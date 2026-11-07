//! Loại tài khoản suy từ mã (bài 41). Mã ngoài sơ đồ bị từ chối thay vì mở bừa.

/// Loại tài khoản như cột `accounts.kind`; `None` nếu mã không thuộc sơ đồ.
pub fn kind_of(account: &str) -> Option<&'static str> {
    let parts: Vec<&str> = account.split(':').collect();
    match parts.as_slice() {
        ["customer" | "driver", who] if !who.is_empty() => Some("liability"),
        // Bài 45 thêm ví khuyến mãi (liability) và quỹ nền tảng (asset) ngoài tám mẫu của bài 41.
        ["platform", "promo-wallet"] | ["tax", "vat", "payable"] => Some("liability"),
        ["platform", "revenue"] => Some("revenue"),
        ["platform", "promo-expense"] => Some("expense"),
        ["platform", "treasury"]
        | ["psp", "vixanh", "clearing"]
        | ["bank", "payout", "clearing"] => Some("asset"),
        ["promo", "budget", campaign] if !campaign.is_empty() => Some("asset"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_by_shape() {
        assert_eq!(kind_of("customer:CU-001"), Some("liability"));
        assert_eq!(kind_of("platform:revenue"), Some("revenue"));
        assert_eq!(kind_of("psp:vixanh:clearing"), Some("asset"));
        assert_eq!(kind_of("promo:budget:WELCOME5K"), Some("asset"));
        assert_eq!(kind_of("platform:promo-wallet"), Some("liability"));
        assert_eq!(kind_of("custmer:CU-001"), None);
    }
}
