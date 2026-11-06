//! Property test (proptest): bất biến phải đúng với MỌI đầu vào, không chỉ vài ví dụ chọn tay.

use ledger_core::{
    AccountId, FeePolicy, HalfAwayFromZero, JournalEntry, Ledger, Money, NearestEven,
    PercentageFee, Posting,
};
use proptest::prelude::*;
use rust_decimal::Decimal;

const ACCOUNTS: [&str; 4] = [
    "customer:CU-001",
    "driver:DR-001",
    "platform:revenue",
    "psp:vixanh:clearing",
];

// Bút toán cân: chọn ngẫu nhiên các posting đầu, posting cuối bù đúng phần lệch.
fn balanced_entry(key: &str, lines: &[(usize, i64)]) -> JournalEntry {
    let mut postings: Vec<Posting> = lines
        .iter()
        .map(|&(account, dong)| {
            let id = AccountId::new(ACCOUNTS[account % ACCOUNTS.len()]).unwrap();
            Posting::new(id, Money::from_dong(dong))
        })
        .collect();
    let sum: i64 = lines.iter().map(|&(_, dong)| dong).sum();
    let balancer = AccountId::new("platform:revenue").unwrap();
    postings.push(Posting::new(balancer, Money::from_dong(-sum)));
    JournalEntry::try_new(key, postings).unwrap()
}

fn lines() -> impl Strategy<Value = Vec<(usize, i64)>> {
    prop::collection::vec((0..4usize, -1_000_000_000_000i64..1_000_000_000_000), 1..6)
}

proptest! {
    // Bất biến 1: ghi bao nhiêu bút toán cân thì tổng toàn sổ vẫn bằng 0
    // và mọi số dư cộng lại cũng bằng 0.
    #[test]
    fn grand_total_stays_zero(batches in prop::collection::vec(lines(), 1..8)) {
        let mut ledger = Ledger::new();
        for (i, lines) in batches.iter().enumerate() {
            ledger.post(balanced_entry(&format!("k{i}"), lines)).unwrap();
        }
        prop_assert_eq!(ledger.grand_total().unwrap(), Money::ZERO);
        let sum: i64 = ledger.balances().unwrap().values().map(|m| m.dong()).sum();
        prop_assert_eq!(sum, 0);
    }

    // Bất biến 2: ghi lại cùng bút toán (cùng key) bao nhiêu lần cũng không đổi số dư.
    #[test]
    fn replay_never_changes_balances(lines in lines(), repeats in 1usize..5) {
        let mut ledger = Ledger::new();
        ledger.post(balanced_entry("k", &lines)).unwrap();
        let before = ledger.balances().unwrap();
        for _ in 0..repeats {
            let again = ledger.post(balanced_entry("k", &lines)).unwrap();
            prop_assert!(again.replayed);
        }
        prop_assert_eq!(ledger.len(), 1);
        prop_assert_eq!(ledger.balances().unwrap(), before);
    }

    // Bất biến 3: đảo một bút toán đưa mọi số dư về 0; đảo lần hai chỉ là replay.
    #[test]
    fn reversal_cancels_the_entry(lines in lines()) {
        let mut ledger = Ledger::new();
        let id = ledger.post(balanced_entry("k", &lines)).unwrap().id;
        ledger.reverse(id).unwrap();
        prop_assert!(ledger.balances().unwrap().values().all(|m| *m == Money::ZERO));
        let again = ledger.reverse(id).unwrap();
        prop_assert!(again.replayed);
        prop_assert_eq!(ledger.len(), 2);
    }

    // Bất biến 4: chia cước không mất và không thừa đồng nào, với mọi mức cước và tỷ lệ,
    // dù làm tròn kiểu nào.
    #[test]
    fn fare_split_never_loses_a_dong(fare in 0i64..=1_000_000_000_000, percent in 0i64..=100) {
        let fare = Money::from_dong(fare);
        let rate = Decimal::new(percent, 2);
        for split in [
            PercentageFee::new(rate, HalfAwayFromZero).split(fare).unwrap(),
            PercentageFee::new(rate, NearestEven).split(fare).unwrap(),
        ] {
            prop_assert_eq!(split.platform_fee.checked_add(split.driver_net), Some(fare));
            prop_assert!(split.platform_fee >= Money::ZERO && split.driver_net >= Money::ZERO);
        }
    }

    // Bất biến 5: hai cách làm tròn nửa đồng hơn kém nhau tối đa 1 đồng.
    #[test]
    fn rounding_policies_differ_by_at_most_one_dong(
        fare in 0i64..=1_000_000_000_000,
        percent in 0i64..=100,
    ) {
        let fare = Money::from_dong(fare);
        let rate = Decimal::new(percent, 2);
        let away = PercentageFee::new(rate, HalfAwayFromZero).fee(fare).unwrap().dong();
        let even = PercentageFee::new(rate, NearestEven).fee(fare).unwrap().dong();
        prop_assert!((away - even).abs() <= 1);
    }

    // Bất biến 6: bút toán lệch bất kỳ đều bị từ chối, không lọt vào sổ.
    #[test]
    fn unbalanced_entry_is_always_rejected(a in -1_000_000i64..1_000_000, b in -1_000_000i64..1_000_000) {
        prop_assume!(a + b != 0);
        let result = JournalEntry::try_new(
            "k",
            vec![
                Posting::new(AccountId::new("a").unwrap(), Money::from_dong(a)),
                Posting::new(AccountId::new("b").unwrap(), Money::from_dong(b)),
            ],
        );
        prop_assert!(result.is_err());
    }
}
