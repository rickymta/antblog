//! Đối soát thuần theo dòng clearing (bài 47): gom mỗi phía theo `external_ref`, cộng tiền, so hai tổng.
//! Phần đọc sổ và sao kê nằm ở adapter; ở đây chỉ có phép phân loại.

use std::collections::BTreeMap;

/// Một dòng tiền gắn với mã giao dịch VíXanh.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// Mã giao dịch VíXanh.
    pub external_ref: String,
    /// Số tiền, đồng.
    pub amount_dong: i64,
}

impl Line {
    /// Dựng nhanh từ literal.
    pub fn new(external_ref: &str, amount_dong: i64) -> Line {
        Line {
            external_ref: external_ref.to_string(),
            amount_dong,
        }
    }
}

/// Kết luận của một `external_ref`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Hai phía có và bằng nhau.
    Matched,
    /// VíXanh có, sổ ta không có.
    MissingInLedger,
    /// Sổ ta có, VíXanh không có.
    MissingInPsp,
    /// Cả hai có nhưng tổng khác nhau.
    AmountMismatch,
}

impl Kind {
    /// Tên như bài 47.
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Matched => "MATCHED",
            Kind::MissingInLedger => "MISSING_IN_LEDGER",
            Kind::MissingInPsp => "MISSING_IN_PSP",
            Kind::AmountMismatch => "AMOUNT_MISMATCH",
        }
    }
}

/// Một dòng báo cáo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// Mã giao dịch VíXanh.
    pub external_ref: String,
    /// Kết luận.
    pub kind: Kind,
    /// Tổng phía sổ ta.
    pub ours: Option<i64>,
    /// Tổng phía VíXanh.
    pub psp: Option<i64>,
}

fn totals(lines: &[Line]) -> BTreeMap<&str, i64> {
    let mut sums = BTreeMap::new();
    for line in lines {
        *sums.entry(line.external_ref.as_str()).or_insert(0) += line.amount_dong;
    }
    sums
}

/// So hai phía. Kết quả theo thứ tự `external_ref`; 1-1, 1-n và n-1 đều khớp nếu tổng bằng nhau.
pub fn reconcile(ours: &[Line], psp: &[Line]) -> Vec<Item> {
    let (ours, psp) = (totals(ours), totals(psp));
    let mut refs: Vec<&str> = ours.keys().chain(psp.keys()).copied().collect();
    refs.sort_unstable();
    refs.dedup();
    refs.into_iter()
        .map(|external_ref| {
            let (o, p) = (
                ours.get(external_ref).copied(),
                psp.get(external_ref).copied(),
            );
            let kind = match (o, p) {
                (None, _) => Kind::MissingInLedger,
                (_, None) => Kind::MissingInPsp,
                (Some(a), Some(b)) if a == b => Kind::Matched,
                _ => Kind::AmountMismatch,
            };
            Item {
                external_ref: external_ref.to_string(),
                kind,
                ours: o,
                psp: p,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_the_four_outcomes() {
        let ours = [
            Line::new("vx_1", 25_500),
            Line::new("vx_2", 10_000),
            Line::new("vx_2", 10_500),
            Line::new("vx_3", 7_000),
            Line::new("vx_4", 1_000),
        ];
        let psp = [
            Line::new("vx_1", 25_500),
            Line::new("vx_2", 20_500),
            Line::new("vx_3", 6_500),
            Line::new("vx_5", 9_000),
        ];
        let kinds: Vec<_> = reconcile(&ours, &psp)
            .iter()
            .map(|i| (i.external_ref.clone(), i.kind))
            .collect();
        assert_eq!(
            kinds,
            [
                ("vx_1".to_string(), Kind::Matched),
                ("vx_2".to_string(), Kind::Matched),
                ("vx_3".to_string(), Kind::AmountMismatch),
                ("vx_4".to_string(), Kind::MissingInPsp),
                ("vx_5".to_string(), Kind::MissingInLedger),
            ]
        );
    }
}
