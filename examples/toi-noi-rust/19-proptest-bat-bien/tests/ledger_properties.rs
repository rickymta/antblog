//! Property test cho sổ cái: chỉ dùng API pub, như người dùng thư viện.

use std::collections::{HashMap, HashSet};

use ledger_lab::strategies::{self, ACCOUNTS};
use ledger_lab::{AccountId, JournalEntry, Ledger, LedgerError, Money, Posting};
use proptest::prelude::*;

fn acct(raw: &str) -> AccountId {
    AccountId::new(raw).expect("mã tài khoản hợp lệ")
}

/// Gán key `k-0`, `k-1`, ... rồi ghi cả dãy; trả sổ cái.
fn post_all(sequence: &[Vec<Posting>]) -> Ledger {
    let mut ledger = Ledger::new();
    for (i, postings) in sequence.iter().enumerate() {
        let entry = JournalEntry::try_new(&format!("k-{i}"), postings.clone())
            .expect("strategy chỉ sinh bút toán cân");
        ledger.post(entry).expect("key mới không xung đột");
    }
    ledger
}

proptest! {
    // 256 ca mỗi test là mặc định; ở đây nâng lên 1000 cho các bất biến của sổ cái.
    #![proptest_config(ProptestConfig { cases: 1000, ..ProptestConfig::default() })]

    // Bất biến 1: strategy sinh bút toán cân, và try_new nhận mọi bút toán đó.
    #[test]
    fn generated_entries_sum_to_zero(postings in strategies::balanced_postings()) {
        let sum: i128 = postings.iter().map(|p| i128::from(p.amount.dong())).sum();
        prop_assert_eq!(sum, 0);
        prop_assert!(JournalEntry::try_new("k", postings).is_ok());
    }

    // Bất biến 2: sau mọi dãy post, tổng toàn sổ và tổng các số dư đều bằng 0.
    #[test]
    fn ledger_stays_balanced_after_any_sequence(sequence in strategies::entry_sequence()) {
        let ledger = post_all(&sequence);
        prop_assert_eq!(ledger.len(), sequence.len());
        prop_assert_eq!(ledger.grand_total(), Ok(Money::ZERO));
        let mut total = 0i128;
        for raw in ACCOUNTS {
            total += i128::from(ledger.balance(&acct(raw)).unwrap().dong());
        }
        prop_assert_eq!(total, 0);
    }

    // Bất biến 3: số dư khớp mô hình tham chiếu viết độc lập (HashMap + i128).
    #[test]
    fn balances_match_a_reference_model(sequence in strategies::entry_sequence()) {
        let ledger = post_all(&sequence);
        let mut model: HashMap<String, i128> = HashMap::new();
        for postings in &sequence {
            for p in postings {
                *model.entry(p.account.to_string()).or_default() += i128::from(p.amount.dong());
            }
        }
        for raw in ACCOUNTS {
            let expected = model.get(raw).copied().unwrap_or(0);
            prop_assert_eq!(i128::from(ledger.balance(&acct(raw)).unwrap().dong()), expected);
        }
    }

    // Bất biến 4 (idempotency): ghi lại cả dãy với cùng key thì không thêm bút toán,
    // mọi lần ghi lại đều là replay, số dư không đổi.
    #[test]
    fn posting_twice_is_the_same_as_posting_once(sequence in strategies::entry_sequence()) {
        let mut ledger = post_all(&sequence);
        let before: Vec<Money> = ACCOUNTS.iter().map(|r| ledger.balance(&acct(r)).unwrap()).collect();
        for (i, postings) in sequence.iter().enumerate() {
            let entry = JournalEntry::try_new(&format!("k-{i}"), postings.clone()).unwrap();
            prop_assert!(ledger.post(entry).unwrap().replayed);
        }
        prop_assert_eq!(ledger.len(), sequence.len());
        let after: Vec<Money> = ACCOUNTS.iter().map(|r| ledger.balance(&acct(r)).unwrap()).collect();
        prop_assert_eq!(before, after);
    }

    // Bất biến 5: key trùng tự do (chỉ có 3 key) thì sổ cái chỉ giữ một bút toán mỗi key,
    // còn lần ghi khác nội dung bị từ chối và không làm hỏng gì.
    #[test]
    fn colliding_keys_never_corrupt_the_ledger(
        attempts in prop::collection::vec(
            (prop::sample::select(vec!["a", "b", "c"]), strategies::balanced_postings()),
            0..=12,
        )
    ) {
        let mut ledger = Ledger::new();
        let mut seen: HashSet<&str> = HashSet::new();
        for (key, postings) in &attempts {
            let entry = JournalEntry::try_new(key, postings.clone()).unwrap();
            match ledger.post(entry) {
                Ok(posted) => {
                    prop_assert_eq!(posted.replayed, seen.contains(key));
                    seen.insert(key);
                }
                Err(e) => {
                    // Tách ra biến: `prop_assert!` không nhận biểu thức có dấu `{ .. }`.
                    let is_conflict = matches!(e, LedgerError::IdempotencyConflict { .. });
                    prop_assert!(is_conflict);
                    prop_assert!(seen.contains(key));
                }
            }
        }
        prop_assert_eq!(ledger.len(), seen.len());
        prop_assert_eq!(ledger.grand_total(), Ok(Money::ZERO));
    }

    // Bất biến 6: lệch một posting `delta` đồng thì bị từ chối, và báo đúng độ lệch.
    #[test]
    fn perturbed_entry_is_rejected(
        postings in strategies::balanced_postings(),
        delta in 1i64..=1_000_000,
    ) {
        let mut broken = postings;
        let first = broken[0].amount.dong();
        broken[0] = Posting::new(broken[0].account.clone(), Money::from_dong(first + delta));
        prop_assert_eq!(
            JournalEntry::try_new("k", broken),
            Err(LedgerError::Unbalanced { sum: Money::from_dong(delta) })
        );
    }
}

// Chạy với `cargo test -- --nocapture` để xem vài bút toán strategy sinh ra.
#[test]
fn show_sample_entries() {
    use proptest::strategy::{Strategy, ValueTree};
    let mut runner = ledger_lab::shrink_demo::deterministic_runner(1);
    for n in 1..=3 {
        let postings = strategies::balanced_postings()
            .new_tree(&mut runner)
            .expect("sinh được giá trị")
            .current();
        println!("mẫu {n}:");
        for p in &postings {
            println!("  {:<20}{:>22}", p.account.to_string(), p.amount);
        }
    }
}
