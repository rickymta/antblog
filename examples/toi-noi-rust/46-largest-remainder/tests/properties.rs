//! Property test cho `allocate`: các bất biến của phép chia tiền không mất đồng.
//!
//! Chỉ dùng API `pub`, như người dùng thật của thư viện.

use std::collections::BTreeMap;

use ledger_core::{Ledger, Money};
use ledger_lab::book::distribute_entry;
use ledger_lab::search::{
    alabama_counterexample, hashmap_tiebreak_winners, naive_minimal_counterexample,
};
use ledger_lab::{Party, allocate, allocate_with_plug};
use proptest::prelude::*;

/// Tổng tiền có thiên hướng về biên: cỡ tiền thật, mọi độ lớn, và hai đầu mút `i64`.
fn total_dong() -> impl Strategy<Value = i64> {
    prop_oneof![
        3 => -100_000i64..=100_000,
        2 => any::<i64>(),
        1 => Just(i64::MIN),
        1 => Just(i64::MAX),
    ]
}

/// Trọng số vừa phải (đến `u32::MAX`), nhiều số 0.
fn moderate_weight() -> impl Strategy<Value = u64> {
    prop_oneof![
        2 => Just(0u64),
        4 => 1u64..=1_000,
        1 => 0u64..=u64::from(u32::MAX),
    ]
}

/// 1..=8 bên có key khác nhau (không xếp theo thứ tự nào đặc biệt sau `prop_shuffle`),
/// ít nhất một trọng số khác 0.
fn parties_with(weight: impl Strategy<Value = u64> + 'static) -> impl Strategy<Value = Vec<Party>> {
    prop::collection::btree_map("[a-z]{1,4}", weight, 1..=8)
        .prop_filter("tổng trọng số > 0", |m| m.values().any(|&w| w > 0))
        .prop_map(|m| m.into_iter().map(|(k, w)| Party::new(&k, w)).collect())
}

fn moderate_parties() -> impl Strategy<Value = Vec<Party>> {
    parties_with(moderate_weight())
}

/// Trọng số chỉ 0..=3: nhiều bên bằng nhau nên phần dư HÒA nhau thường xuyên, đúng chỗ tiêu chí
/// phá hòa phải làm việc. Trọng số ngẫu nhiên rộng gần như không bao giờ hòa.
fn tie_prone_parties() -> impl Strategy<Value = Vec<Party>> {
    parties_with(0u64..=3)
}

/// Gồm cả trọng số khổng lồ (đến `u64::MAX`).
fn any_parties() -> impl Strategy<Value = Vec<Party>> {
    parties_with(prop_oneof![
        3 => moderate_weight(),
        1 => any::<u64>(),
        1 => Just(u64::MAX),
    ])
}

fn amounts(shares: &[ledger_lab::Share]) -> Vec<i128> {
    shares.iter().map(|s| i128::from(s.amount.dong())).collect()
}

fn by_key(shares: &[ledger_lab::Share]) -> BTreeMap<String, i64> {
    shares
        .iter()
        .map(|s| (s.key.clone(), s.amount.dong()))
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 1000, ..ProptestConfig::default() })]

    // Bất biến chính: tổng các phần đúng bằng tổng gốc, với mọi tổng i64 và trọng số u64.
    #[test]
    fn parts_sum_to_total(total in total_dong(), parties in any_parties()) {
        let shares = allocate(Money::from_dong(total), &parties).unwrap();
        prop_assert_eq!(shares.len(), parties.len());
        let sum: i128 = amounts(&shares).iter().sum();
        prop_assert_eq!(sum, i128::from(total));
    }

    // Mỗi phần lệch phần chính xác (tổng × trọng số / tổng trọng số) dưới 1 đồng.
    // Nhân chéo bằng i128 để không có phép chia nào che lỗi.
    #[test]
    fn each_part_is_within_one_dong(total in total_dong(), parties in moderate_parties()) {
        let shares = allocate(Money::from_dong(total), &parties).unwrap();
        let weight_sum: i128 = parties.iter().map(|p| i128::from(p.weight)).sum();
        for (share, party) in shares.iter().zip(&parties) {
            let diff = i128::from(share.amount.dong()) * weight_sum
                - i128::from(total) * i128::from(party.weight);
            prop_assert!(diff.abs() < weight_sum);
        }
    }

    // Không bên nào nhận phần trái dấu hay lớn hơn tổng.
    #[test]
    fn parts_never_flip_sign_or_exceed_total(total in total_dong(), parties in any_parties()) {
        let shares = allocate(Money::from_dong(total), &parties).unwrap();
        for amount in amounts(&shares) {
            prop_assert!(amount.abs() <= i128::from(total).abs());
            prop_assert!(amount == 0 || (amount < 0) == (total < 0));
        }
    }

    // Tất định: đảo thứ tự đầu vào không đổi phần của từng key.
    #[test]
    fn result_does_not_depend_on_input_order(
        total in prop_oneof![-1_000i64..=1_000, total_dong()],
        (parties, shuffled) in tie_prone_parties()
            .prop_flat_map(|ps| (Just(ps.clone()), Just(ps).prop_shuffle())),
    ) {
        let a = allocate(Money::from_dong(total), &parties).unwrap();
        let b = allocate(Money::from_dong(total), &shuffled).unwrap();
        prop_assert_eq!(by_key(&a), by_key(&b));
        // Gọi lại với cùng đầu vào: cùng kết quả.
        prop_assert_eq!(a, allocate(Money::from_dong(total), &parties).unwrap());
    }

    // Bên trọng số 0 không nhận đồng nào, kể cả đồng lẻ.
    #[test]
    fn zero_weight_gets_nothing(total in total_dong(), parties in moderate_parties()) {
        let shares = allocate(Money::from_dong(total), &parties).unwrap();
        for (share, party) in shares.iter().zip(&parties) {
            if party.weight == 0 {
                prop_assert_eq!(share.amount, Money::ZERO);
            }
        }
    }

    // Đơn điệu ở mức chứng minh được: trọng số lớn hơn thì không nhận ít hơn.
    #[test]
    fn heavier_party_never_gets_less(total in total_dong(), parties in any_parties()) {
        let shares = allocate(Money::from_dong(total), &parties).unwrap();
        for (si, pi) in shares.iter().zip(&parties) {
            for (sj, pj) in shares.iter().zip(&parties) {
                if pi.weight > pj.weight {
                    prop_assert!(
                        i128::from(si.amount.dong()).abs() >= i128::from(sj.amount.dong()).abs()
                    );
                }
            }
        }
    }

    // Hoàn tiền là đối xứng của thanh toán, từng đồng, kể cả đồng lẻ.
    #[test]
    fn refund_mirrors_payment(total in total_dong(), parties in any_parties()) {
        prop_assume!(total != i64::MIN);
        let paid = allocate(Money::from_dong(total), &parties).unwrap();
        let refund = allocate(Money::from_dong(-total), &parties).unwrap();
        let negated: Vec<i128> = amounts(&paid).iter().map(|a| -a).collect();
        prop_assert_eq!(amounts(&refund), negated);
    }

    // Chia đều cho n bên: hai phần bất kỳ chênh nhau không quá 1 đồng.
    #[test]
    fn equal_weights_differ_by_at_most_one(total in total_dong(), n in 1usize..=8) {
        let parties: Vec<Party> = (0..n).map(|i| Party::new(&format!("p{i}"), 1)).collect();
        let dongs = amounts(&allocate(Money::from_dong(total), &parties).unwrap());
        let (min, max) = (dongs.iter().min().unwrap(), dongs.iter().max().unwrap());
        prop_assert!(max - min <= 1);
    }

    // Cách «bên cuối nhận phần còn lại» cũng giữ tổng; các bên khác lệch không quá nửa đồng.
    #[test]
    fn plug_keeps_total_and_others_within_half_a_dong(
        total in total_dong(),
        parties in moderate_parties(),
    ) {
        let plug = parties[0].key.clone();
        let shares = allocate_with_plug(Money::from_dong(total), &parties, &plug).unwrap();
        let sum: i128 = amounts(&shares).iter().sum();
        prop_assert_eq!(sum, i128::from(total));
        let weight_sum: i128 = parties.iter().map(|p| i128::from(p.weight)).sum();
        for (share, party) in shares.iter().zip(&parties) {
            if party.key != plug {
                let diff = i128::from(share.amount.dong()) * weight_sum
                    - i128::from(total) * i128::from(party.weight);
                prop_assert!(2 * diff.abs() <= weight_sum);
            }
        }
    }

    // Phần chia dựng được bút toán cân, ghi vào sổ, tổng toàn sổ bằng 0.
    #[test]
    fn distribution_is_a_balanced_entry(total in -1_000_000_000i64..=1_000_000_000, parties in moderate_parties()) {
        let shares = allocate(Money::from_dong(total), &parties).unwrap();
        let entry = distribute_entry("trip:TN-9999:split", "pool:test", &shares).unwrap();
        let mut ledger = Ledger::new();
        ledger.post(entry).unwrap();
        prop_assert_eq!(ledger.grand_total(), Ok(Money::ZERO));
    }
}

// Proptest tìm được phản ví dụ của bản làm tròn từng phần (RNG cố định, không ghi file).
#[test]
fn proptest_finds_a_counterexample_to_naive_rounding() {
    let (total, weights) = naive_minimal_counterexample().expect("bản ngây thơ phải bị bắt");
    assert!(total >= 0 && !weights.is_empty());
}

// Giới hạn của «đơn điệu»: tăng tổng thêm 1 đồng có thể làm một bên nhận ít đi (nghịch lý
// Alabama của largest remainder), nên test không khẳng định điều đó.
#[test]
fn total_monotonicity_does_not_hold() {
    let case = alabama_counterexample(10, 3).expect("có ca nhỏ");
    assert_eq!(case.total, 3);
    assert_eq!(case.weights, [1, 3, 3]);
    assert!(case.after[case.party] < case.before[case.party]);
}

// Phá hòa theo HashMap không tất định: nhiều lần thử cho nhiều bên nhận đồng lẻ khác nhau.
#[test]
fn hashmap_order_is_not_a_stable_tie_break() {
    assert!(hashmap_tiebreak_winners(200).len() > 1);
}
