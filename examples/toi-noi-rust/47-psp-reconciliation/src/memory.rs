//! Khớp trong bộ nhớ Rust: cùng luật phân loại với `sql/joined.sql`, dùng để đối chiếu và đo.

use std::collections::HashMap;

use jiff::Timestamp;

use crate::kind::Kind;
use crate::period::Period;

/// Một dòng ở một phía: số tiền và thời điểm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Row {
    /// Số tiền có dấu, đồng.
    pub amount: i64,
    /// Thời điểm của dòng (posted_at phía ta, settled_at phía VíXanh).
    pub at: Timestamp,
}

/// Kết luận cho một `external_ref`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Không phía nào có dòng trong kỳ: thuộc kỳ khác.
    OutOfPeriod,
    /// Khớp. `split` là true khi một phía có nhiều dòng (1-n, n-1, n-n).
    Matched {
        /// Có phía nào tách thành nhiều dòng không.
        split: bool,
    },
    /// Ngoại lệ.
    Exception(Kind),
}

struct SideStats {
    n: i64,
    total: i64,
    n_in: i64,
    dup_n: i64,
    dup_total: i64,
}

fn side_stats(rows: &[Row], period: &Period) -> SideStats {
    let mut stats = SideStats {
        n: 0,
        total: 0,
        n_in: 0,
        dup_n: 0,
        dup_total: 0,
    };
    // Dòng trùng = cùng số tiền với một dòng đã thấy của cùng ref (giống row_number() > 1 trong SQL).
    let mut seen: Vec<i64> = Vec::new();
    for row in rows {
        stats.n += 1;
        stats.total += row.amount;
        if period.contains(row.at) {
            stats.n_in += 1;
        }
        if seen.contains(&row.amount) {
            stats.dup_n += 1;
            stats.dup_total += row.amount;
        } else {
            seen.push(row.amount);
        }
    }
    stats
}

/// Phân loại một `external_ref` từ các dòng hai phía. Luật khớp với `sql/joined.sql`.
pub fn classify(ours: &[Row], psp: &[Row], period: &Period) -> Verdict {
    let o = side_stats(ours, period);
    let p = side_stats(psp, period);
    if o.n_in + p.n_in == 0 {
        return Verdict::OutOfPeriod;
    }
    if o.n == 0 {
        return Verdict::Exception(Kind::MissingInLedger);
    }
    if p.n == 0 {
        return Verdict::Exception(Kind::MissingInPsp);
    }
    if o.total == p.total {
        return if o.n_in == o.n && p.n_in == p.n {
            Verdict::Matched {
                split: o.n > 1 || p.n > 1,
            }
        } else {
            Verdict::Exception(Kind::DateShift)
        };
    }
    if (o.dup_n > 0 || p.dup_n > 0) && o.total - o.dup_total == p.total - p.dup_total {
        return Verdict::Exception(Kind::Duplicate);
    }
    Verdict::Exception(Kind::AmountMismatch)
}

/// Một ngoại lệ, cùng hình dạng với dòng `reconciliation_items`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exception {
    /// Loại lệch.
    pub kind: Kind,
    /// Mã giao dịch VíXanh.
    pub external_ref: String,
    /// Tổng phía ta; `None` nếu phía ta không có dòng.
    pub ours_dong: Option<i64>,
    /// Tổng phía VíXanh; `None` nếu sao kê không có dòng.
    pub psp_dong: Option<i64>,
    /// Số dòng phía ta.
    pub ours_rows: i32,
    /// Số dòng phía VíXanh.
    pub psp_rows: i32,
}

/// Kết quả khớp trong bộ nhớ.
#[derive(Debug, Default)]
pub struct MatchResult {
    /// Số ref khớp.
    pub matched: u64,
    /// Trong đó số ref có nhiều dòng ở một phía.
    pub matched_split: u64,
    /// Số ref nằm ngoài kỳ ở cả hai phía.
    pub out_of_period: u64,
    /// Ngoại lệ, sắp theo `external_ref` (so byte).
    pub exceptions: Vec<Exception>,
}

#[derive(Default)]
struct Sides {
    ours: Vec<Row>,
    psp: Vec<Row>,
}

/// Gom mọi dòng của hai phía theo `external_ref` rồi phân loại, giống `group_by` ở bài 15.
/// Bộ nhớ tỷ lệ với số ref và số dòng: đây là cái giá cần so với khớp bằng SQL.
#[derive(Default)]
pub struct Matcher {
    by_ref: HashMap<String, Sides>,
}

impl Matcher {
    fn sides(&mut self, external_ref: &str) -> &mut Sides {
        if !self.by_ref.contains_key(external_ref) {
            self.by_ref
                .insert(external_ref.to_string(), Sides::default());
        }
        self.by_ref
            .get_mut(external_ref)
            .expect("vừa chèn nếu chưa có")
    }

    /// Thêm một dòng phía sổ cái.
    pub fn add_ours(&mut self, external_ref: &str, row: Row) {
        self.sides(external_ref).ours.push(row);
    }

    /// Thêm một dòng phía sao kê.
    pub fn add_psp(&mut self, external_ref: &str, row: Row) {
        self.sides(external_ref).psp.push(row);
    }

    /// Số ref phân biệt đang giữ.
    pub fn refs(&self) -> usize {
        self.by_ref.len()
    }

    /// Phân loại mọi ref.
    pub fn finish(self, period: &Period) -> MatchResult {
        let mut result = MatchResult::default();
        for (external_ref, sides) in self.by_ref {
            match classify(&sides.ours, &sides.psp, period) {
                Verdict::OutOfPeriod => result.out_of_period += 1,
                Verdict::Matched { split } => {
                    result.matched += 1;
                    result.matched_split += u64::from(split);
                }
                Verdict::Exception(kind) => {
                    let total = |rows: &[Row]| {
                        (!rows.is_empty()).then(|| rows.iter().map(|r| r.amount).sum::<i64>())
                    };
                    result.exceptions.push(Exception {
                        kind,
                        ours_dong: total(&sides.ours),
                        psp_dong: total(&sides.psp),
                        ours_rows: i32::try_from(sides.ours.len()).unwrap_or(i32::MAX),
                        psp_rows: i32::try_from(sides.psp.len()).unwrap_or(i32::MAX),
                        external_ref,
                    });
                }
            }
        }
        result
            .exceptions
            .sort_by(|a, b| a.external_ref.cmp(&b.external_ref));
        result
    }
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;

    use super::*;

    fn period() -> Period {
        Period::business_day(date(2026, 11, 5)).unwrap()
    }

    fn at(raw: &str) -> Timestamp {
        raw.parse().unwrap()
    }

    fn row(amount: i64, raw: &str) -> Row {
        Row {
            amount,
            at: at(raw),
        }
    }

    const NOON: &str = "2026-11-05T05:00:00Z";
    const NEXT_DAY: &str = "2026-11-05T17:02:00Z";

    #[test]
    fn one_to_one_and_split_both_match() {
        let p = period();
        assert_eq!(
            classify(&[row(100, NOON)], &[row(100, NOON)], &p),
            Verdict::Matched { split: false }
        );
        assert_eq!(
            classify(&[row(100, NOON)], &[row(60, NOON), row(40, NOON)], &p),
            Verdict::Matched { split: true }
        );
        assert_eq!(
            classify(&[row(50, NOON), row(50, NOON)], &[row(100, NOON)], &p),
            Verdict::Matched { split: true }
        );
    }

    #[test]
    fn missing_on_either_side() {
        let p = period();
        assert_eq!(
            classify(&[], &[row(100, NOON)], &p),
            Verdict::Exception(Kind::MissingInLedger)
        );
        assert_eq!(
            classify(&[row(100, NOON)], &[], &p),
            Verdict::Exception(Kind::MissingInPsp)
        );
    }

    #[test]
    fn amount_mismatch_and_duplicate_are_told_apart() {
        let p = period();
        assert_eq!(
            classify(&[row(100, NOON)], &[row(90, NOON)], &p),
            Verdict::Exception(Kind::AmountMismatch)
        );
        // Sao kê có hai dòng y hệt; bỏ một bản thì khớp sổ ta.
        assert_eq!(
            classify(&[row(100, NOON)], &[row(100, NOON), row(100, NOON)], &p),
            Verdict::Exception(Kind::Duplicate)
        );
        // Dòng thừa không phải bản sao của dòng nào thì vẫn là lệch tiền.
        assert_eq!(
            classify(&[row(100, NOON)], &[row(100, NOON), row(30, NOON)], &p),
            Verdict::Exception(Kind::AmountMismatch)
        );
    }

    #[test]
    fn date_shift_when_equal_but_one_side_outside_period() {
        let p = period();
        assert_eq!(
            classify(&[row(100, NOON)], &[row(100, NEXT_DAY)], &p),
            Verdict::Exception(Kind::DateShift)
        );
    }

    #[test]
    fn both_sides_outside_period_are_ignored() {
        let p = period();
        assert_eq!(
            classify(&[row(100, NEXT_DAY)], &[row(100, NEXT_DAY)], &p),
            Verdict::OutOfPeriod
        );
    }

    #[test]
    fn matcher_counts_and_sorts() {
        let p = period();
        let mut m = Matcher::default();
        m.add_ours("b", row(10, NOON));
        m.add_psp("b", row(10, NOON));
        m.add_psp("a", row(5, NOON));
        m.add_ours("c", row(7, NOON));
        assert_eq!(m.refs(), 3);
        let r = m.finish(&p);
        assert_eq!(r.matched, 1);
        let refs: Vec<&str> = r
            .exceptions
            .iter()
            .map(|e| e.external_ref.as_str())
            .collect();
        assert_eq!(refs, ["a", "c"]);
        assert_eq!(r.exceptions[0].ours_dong, None);
        assert_eq!(r.exceptions[0].psp_dong, Some(5));
    }
}
