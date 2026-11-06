//! Đối soát trong bộ nhớ: sổ cái Tới Nơi so với sao kê VíXanh, khớp theo `external_ref`.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::hash::Hash;

use crate::entry::JournalEntry;
use crate::error::LedgerError;
use crate::ledger::Ledger;
use crate::money::Money;

/// Tài khoản clearing: số dương là tiền VíXanh đang giữ, chờ về nền tảng (bài 13).
pub const CLEARING: &str = "psp:vixanh:clearing";
/// Quy ước riêng của lab này: bút toán thu qua VíXanh có `idempotency_key` =
/// `psp:vixanh:<external_ref>`. Không thay `trip:TN-0001:capture` của các bài khác;
/// sổ thật giữ key theo chuyến và lưu `external_ref` ở trường riêng.
pub const KEY_PREFIX: &str = "psp:vixanh:";

/// Một dòng phía sổ cái: số tiền VíXanh đã thu hộ theo bút toán.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerRecord {
    pub external_ref: String,
    pub entry_id: String,
    pub amount: Money,
}

/// Một dòng phía sao kê VíXanh.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PspRecord {
    pub external_ref: String,
    pub amount: Money,
}

impl LedgerRecord {
    /// `Ok(None)` nếu bút toán không đi qua VíXanh (khóa không có tiền tố).
    /// Quy ước dấu (bài 13): `clearing` dương là tiền VíXanh đã thu, so thẳng được với sao kê.
    pub fn from_entry(
        entry_id: String,
        entry: &JournalEntry,
    ) -> Result<Option<LedgerRecord>, LedgerError> {
        let Some(external_ref) = entry.idempotency_key().strip_prefix(KEY_PREFIX) else {
            return Ok(None);
        };
        let amount = entry
            .postings()
            .iter()
            .filter(|p| p.account.as_str() == CLEARING)
            .try_fold(Money::ZERO, |acc, p| {
                acc.checked_add(p.amount).ok_or(LedgerError::Overflow)
            })?;
        Ok(Some(LedgerRecord {
            external_ref: external_ref.to_string(),
            entry_id,
            amount,
        }))
    }
}

/// Lấy các dòng cần đối soát từ sổ cái. `filter_map` bỏ bút toán không qua VíXanh;
/// `collect` vào `Result<Vec<_>, _>` dừng ở lỗi đầu tiên.
pub fn ledger_records(ledger: &Ledger) -> Result<Vec<LedgerRecord>, LedgerError> {
    ledger
        .entries_with_id()
        .filter_map(|(id, entry)| LedgerRecord::from_entry(id, entry).transpose())
        .collect()
}

/// Group-by thủ công: gom `items` theo khóa. `key_of` chỉ đọc nên chỉ cần `Fn`.
pub fn group_by<T, K, F>(items: impl IntoIterator<Item = T>, key_of: F) -> HashMap<K, Vec<T>>
where
    K: Eq + Hash,
    F: Fn(&T) -> K,
{
    let mut groups: HashMap<K, Vec<T>> = HashMap::new();
    for item in items {
        // `entry().or_default()` tạo Vec rỗng nếu khóa chưa có, rồi trả `&mut Vec`.
        groups.entry(key_of(&item)).or_default().push(item);
    }
    groups
}

/// Cộng từng nhóm có kiểm tra tràn số; một nhóm tràn thì cả `collect` trả `Err`.
fn sum_groups<T>(
    groups: HashMap<String, Vec<T>>,
    amount_of: impl Fn(&T) -> Money,
) -> Result<HashMap<String, Money>, LedgerError> {
    groups
        .into_iter()
        .map(|(key, rows)| {
            let sum = rows.iter().try_fold(Money::ZERO, |acc, row| {
                acc.checked_add(amount_of(row)).ok_or(LedgerError::Overflow)
            })?;
            Ok((key, sum))
        })
        .collect()
}

/// Tổng tiền theo `external_ref` phía sổ cái.
pub fn ledger_totals(records: &[LedgerRecord]) -> Result<HashMap<String, Money>, LedgerError> {
    sum_groups(group_by(records, |r| r.external_ref.clone()), |r| r.amount)
}

/// Tổng tiền theo `external_ref` phía sao kê (một ref có thể có nhiều dòng).
pub fn psp_totals(records: &[PspRecord]) -> Result<HashMap<String, Money>, LedgerError> {
    sum_groups(group_by(records, |r| r.external_ref.clone()), |r| r.amount)
}

/// Gọi `visit(ref, số_sổ, số_sao_kê)` cho mỗi ref có ở cả hai phía.
/// `FnMut` vì closure của người gọi thường tích lũy trạng thái (đếm, gom lỗi).
pub fn visit_common<F>(ours: &HashMap<String, Money>, theirs: &HashMap<String, Money>, mut visit: F)
where
    F: FnMut(&str, Money, Money),
{
    ours.iter()
        .filter_map(|(key, &ledger)| theirs.get(key).map(|&psp| (key, ledger, psp)))
        .for_each(|(key, ledger, psp)| visit(key, ledger, psp));
}

/// Một chênh lệch giữa hai phía.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Finding {
    /// Sổ ghi nhưng sao kê không có: thiếu bên PSP.
    MissingInPsp { ledger: Money },
    /// Sao kê có nhưng sổ chưa ghi: thiếu bên ta.
    MissingInLedger { psp: Money },
    /// Có ở cả hai nhưng số tiền khác nhau.
    AmountMismatch { ledger: Money, psp: Money },
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::MissingInPsp { ledger } => {
                write!(f, "thiếu bên PSP: sổ ghi {ledger}, sao kê không có")
            }
            Finding::MissingInLedger { psp } => {
                write!(f, "thiếu bên ta: sao kê có {psp}, sổ chưa ghi")
            }
            Finding::AmountMismatch { ledger, psp } => {
                write!(f, "lệch số tiền: sổ {ledger}, sao kê {psp}")?;
                match ledger.checked_sub(*psp) {
                    Some(diff) => write!(f, ", chênh {diff}"),
                    None => write!(f, ", chênh vượt giới hạn"),
                }
            }
        }
    }
}

/// Kết quả đối soát: số ref khớp và các chênh lệch, sắp theo `external_ref`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub matched: usize,
    pub findings: BTreeMap<String, Finding>,
}

/// Số lượng theo từng loại chênh lệch.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Summary {
    pub missing_in_psp: usize,
    pub missing_in_ledger: usize,
    pub amount_mismatch: usize,
}

impl Report {
    /// `fold` gom cả báo cáo thành một `Summary` qua một lượt duyệt.
    pub fn summary(&self) -> Summary {
        self.findings
            .values()
            .fold(Summary::default(), |mut s, finding| {
                match finding {
                    Finding::MissingInPsp { .. } => s.missing_in_psp += 1,
                    Finding::MissingInLedger { .. } => s.missing_in_ledger += 1,
                    Finding::AmountMismatch { .. } => s.amount_mismatch += 1,
                }
                s
            })
    }
}

/// Đối soát: gom theo ref, hợp hai tập ref, rồi phân loại từng ref.
pub fn reconcile(ledger: &[LedgerRecord], psp: &[PspRecord]) -> Result<Report, LedgerError> {
    let ours = ledger_totals(ledger)?;
    let theirs = psp_totals(psp)?;
    // BTreeSet: hợp hai tập ref, không trùng, đã sắp thứ tự.
    let refs: BTreeSet<&str> = ours
        .keys()
        .chain(theirs.keys())
        .map(String::as_str)
        .collect();
    let findings: BTreeMap<String, Finding> = refs
        .iter()
        .filter_map(|&key| {
            let finding = match (ours.get(key).copied(), theirs.get(key).copied()) {
                (Some(l), Some(p)) if l == p => return None,
                (Some(l), Some(p)) => Finding::AmountMismatch { ledger: l, psp: p },
                (Some(l), None) => Finding::MissingInPsp { ledger: l },
                (None, Some(p)) => Finding::MissingInLedger { psp: p },
                (None, None) => return None,
            };
            Some((key.to_string(), finding))
        })
        .collect();
    Ok(Report {
        matched: refs.len() - findings.len(),
        findings,
    })
}

/// Cách làm SAI để so sánh: ghép hai danh sách theo vị trí bằng `zip`.
/// `zip` dừng ở danh sách ngắn hơn và không biết gì về khóa.
pub fn positional_ref_clashes<'a>(
    ledger: &'a [LedgerRecord],
    psp: &'a [PspRecord],
) -> Vec<(&'a str, &'a str)> {
    ledger
        .iter()
        .zip(psp)
        .filter(|(l, p)| l.external_ref != p.external_ref)
        .map(|(l, p)| (l.external_ref.as_str(), p.external_ref.as_str()))
        .collect()
}

/// Trả về closure: phải có `move`, vì `min` là biến cục bộ sẽ biến mất khi hàm kết thúc.
pub fn at_least(min: Money) -> impl Fn(&LedgerRecord) -> bool {
    move |record| record.amount >= min
}

/// Số lần `inspect` chạy ở ba cách tiêu thụ cùng một chuỗi adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LazyProbe {
    /// Dựng chuỗi adapter rồi bỏ đi, không gọi `next`.
    pub after_build: usize,
    /// Lấy phần tử đầu tiên khớp (`next`).
    pub after_first: usize,
    /// `collect` toàn bộ.
    pub after_collect: usize,
}

pub fn lazy_probe(records: &[LedgerRecord], min: Money) -> LazyProbe {
    let big = at_least(min);

    let mut built = 0;
    let unused = records
        .iter()
        .inspect(|_| built += 1) // FnMut: sửa biến `built` của hàm này
        .filter(|r| big(r));
    drop(unused); // trả lại quyền mượn `built`; không phần tử nào bị duyệt
    let after_build = built;

    let mut first = 0;
    let _first_match = records.iter().inspect(|_| first += 1).find(|r| big(r));

    let mut all = 0;
    let _all: Vec<&LedgerRecord> = records
        .iter()
        .inspect(|_| all += 1)
        .filter(|r| big(r))
        .collect();

    LazyProbe {
        after_build,
        after_first: first,
        after_collect: all,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::AccountId;
    use crate::entry::Posting;
    use crate::statement::parse_statement;

    fn acct(raw: &str) -> AccountId {
        AccountId::new(raw).unwrap()
    }

    fn collect_via_psp(ledger: &mut Ledger, external_ref: &str, customer: &str, dong: i64) {
        let entry = JournalEntry::try_new(
            &format!("{KEY_PREFIX}{external_ref}"),
            vec![
                Posting::new(acct(customer), Money::from_dong(-dong)),
                Posting::new(acct(CLEARING), Money::from_dong(dong)),
            ],
        )
        .unwrap();
        ledger.post(entry).unwrap();
    }

    fn lrec(external_ref: &str, dong: i64) -> LedgerRecord {
        LedgerRecord {
            external_ref: external_ref.to_string(),
            entry_id: "JE-000000".to_string(),
            amount: Money::from_dong(dong),
        }
    }

    fn prec(external_ref: &str, dong: i64) -> PspRecord {
        PspRecord {
            external_ref: external_ref.to_string(),
            amount: Money::from_dong(dong),
        }
    }

    #[test]
    fn group_by_collects_rows_per_key() {
        let groups = group_by(["a1", "b1", "a2"], |s| s.chars().next().unwrap());
        assert_eq!(groups[&'a'], vec!["a1", "a2"]);
        assert_eq!(groups[&'b'], vec!["b1"]);
    }

    #[test]
    fn ledger_records_skips_entries_without_prefix() {
        let mut ledger = Ledger::new();
        collect_via_psp(&mut ledger, "VX-1", "customer:CU-001", 25_500);
        let split = JournalEntry::try_new(
            "trip:TN-0001:split",
            vec![
                Posting::new(acct(CLEARING), Money::from_dong(-25_500)),
                Posting::new(acct("platform:revenue"), Money::from_dong(5_100)),
                Posting::new(acct("driver:DR-001"), Money::from_dong(20_400)),
            ],
        )
        .unwrap();
        ledger.post(split).unwrap();
        let records = ledger_records(&ledger).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].external_ref, "VX-1");
        assert_eq!(records[0].entry_id, "JE-000001");
        assert_eq!(records[0].amount, Money::from_dong(25_500));
    }

    #[test]
    fn reconcile_classifies_three_kinds() {
        let ours = [lrec("A", 100), lrec("B", 200), lrec("C", 300)];
        let theirs = [prec("A", 100), prec("B", 150), prec("D", 400)];
        let report = reconcile(&ours, &theirs).unwrap();
        assert_eq!(report.matched, 1);
        assert_eq!(
            report.findings["B"],
            Finding::AmountMismatch {
                ledger: Money::from_dong(200),
                psp: Money::from_dong(150)
            }
        );
        assert_eq!(
            report.findings["C"],
            Finding::MissingInPsp {
                ledger: Money::from_dong(300)
            }
        );
        assert_eq!(
            report.findings["D"],
            Finding::MissingInLedger {
                psp: Money::from_dong(400)
            }
        );
        assert_eq!(
            report.summary(),
            Summary {
                missing_in_psp: 1,
                missing_in_ledger: 1,
                amount_mismatch: 1
            }
        );
    }

    #[test]
    fn split_psp_lines_are_summed_per_ref() {
        let ours = [lrec("A", 40_000)];
        let theirs = [prec("A", 25_000), prec("A", 15_000)];
        let report = reconcile(&ours, &theirs).unwrap();
        assert_eq!(report.matched, 1);
        assert!(report.findings.is_empty());
    }

    #[test]
    fn overflow_in_a_group_is_an_error() {
        let theirs = [prec("A", i64::MAX), prec("A", 1)];
        assert_eq!(psp_totals(&theirs), Err(LedgerError::Overflow));
    }

    #[test]
    fn visit_common_counts_with_fn_mut() {
        let ours = ledger_totals(&[lrec("A", 1), lrec("B", 2)]).unwrap();
        let theirs = psp_totals(&[prec("A", 1), prec("B", 9), prec("C", 3)]).unwrap();
        let mut visited = 0;
        let mut mismatched = 0;
        visit_common(&ours, &theirs, |_, l, p| {
            visited += 1;
            if l != p {
                mismatched += 1;
            }
        });
        assert_eq!((visited, mismatched), (2, 1));
    }

    #[test]
    fn zip_pairs_by_position_not_by_key() {
        let ours = [lrec("A", 1), lrec("B", 2), lrec("C", 3)];
        let theirs = [prec("A", 1), prec("C", 3)];
        // B ghép với C: hai ref khác nhau; dòng C của ta không có cặp nên bị bỏ qua.
        assert_eq!(positional_ref_clashes(&ours, &theirs), vec![("B", "C")]);
    }

    #[test]
    fn lazy_probe_counts_inspections() {
        let records = [
            lrec("A", 25_500),
            lrec("B", 40_000),
            lrec("C", 31_500),
            lrec("D", 18_000),
        ];
        let probe = lazy_probe(&records, Money::from_dong(30_000));
        assert_eq!(
            probe,
            LazyProbe {
                after_build: 0,
                after_first: 2,
                after_collect: 4
            }
        );
    }

    #[test]
    fn at_least_returns_a_reusable_closure() {
        let big = at_least(Money::from_dong(100));
        assert!(big(&lrec("A", 100)));
        assert!(!big(&lrec("B", 99)));
    }

    #[test]
    fn end_to_end_with_statement_text() {
        let mut ledger = Ledger::new();
        collect_via_psp(&mut ledger, "VX-1", "customer:CU-001", 100);
        let text = "external_ref,amount_vnd\nVX-1,100\nVX-2,50\n";
        let psp = parse_statement(text).unwrap();
        let report = reconcile(&ledger_records(&ledger).unwrap(), &psp).unwrap();
        assert_eq!(report.matched, 1);
        assert_eq!(report.summary().missing_in_ledger, 1);
    }
}
