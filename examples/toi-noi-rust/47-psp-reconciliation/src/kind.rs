//! Các loại lệch mà đối soát phân loại ra.

/// Loại ngoại lệ. Thứ tự khai báo cũng là thứ tự trong báo cáo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    /// VíXanh có, sổ ta không có (thiếu bên ta).
    MissingInLedger,
    /// Sổ ta có, VíXanh không có (thiếu bên PSP).
    MissingInPsp,
    /// Cả hai có nhưng tổng tiền khác nhau.
    AmountMismatch,
    /// Một phía có dòng trùng hệt; bỏ bản trùng thì hai phía khớp.
    Duplicate,
    /// Cùng số tiền, nhưng một phía nằm ngoài kỳ (cắt kỳ theo múi giờ khác nhau).
    DateShift,
    /// Dòng sao kê không đọc được.
    BadLine,
}

impl Kind {
    /// Tên lưu trong cột `kind` của `reconciliation_items`.
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::MissingInLedger => "MISSING_IN_LEDGER",
            Kind::MissingInPsp => "MISSING_IN_PSP",
            Kind::AmountMismatch => "AMOUNT_MISMATCH",
            Kind::Duplicate => "DUPLICATE",
            Kind::DateShift => "DATE_SHIFT",
            Kind::BadLine => "BAD_LINE",
        }
    }

    /// Ngược lại của [`Kind::as_str`].
    pub fn parse(raw: &str) -> Option<Kind> {
        Some(match raw {
            "MISSING_IN_LEDGER" => Kind::MissingInLedger,
            "MISSING_IN_PSP" => Kind::MissingInPsp,
            "AMOUNT_MISMATCH" => Kind::AmountMismatch,
            "DUPLICATE" => Kind::Duplicate,
            "DATE_SHIFT" => Kind::DateShift,
            "BAD_LINE" => Kind::BadLine,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for kind in [
            Kind::MissingInLedger,
            Kind::MissingInPsp,
            Kind::AmountMismatch,
            Kind::Duplicate,
            Kind::DateShift,
            Kind::BadLine,
        ] {
            assert_eq!(Kind::parse(kind.as_str()), Some(kind));
        }
        assert_eq!(Kind::parse("MATCHED"), None);
    }
}
