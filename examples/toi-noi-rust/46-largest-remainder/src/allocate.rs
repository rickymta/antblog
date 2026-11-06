//! Chia tiền theo largest remainder: tổng các phần luôn đúng bằng tổng gốc.
//!
//! Tiêu chí phá hòa cố định: phần dư lớn nhất nhận trước; hòa phần dư thì `key` nhỏ hơn
//! (so sánh byte) nhận trước. Kết quả chỉ phụ thuộc vào (key, trọng số, tổng), không phụ
//! thuộc thứ tự truyền vào hay thứ tự duyệt của `HashMap`.

use std::cmp::Reverse;
use std::collections::BTreeSet;
use std::fmt;

use ledger_core::Money;

/// Một bên được chia: `key` định danh ổn định (thường là mã tài khoản), `weight` là trọng số.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Party {
    /// Định danh ổn định, duy nhất trong một lần chia; dùng để phá hòa.
    pub key: String,
    /// Trọng số; 0 nghĩa là bên này không nhận đồng nào.
    pub weight: u64,
}

impl Party {
    /// Tạo một bên.
    pub fn new(key: &str, weight: u64) -> Party {
        Party {
            key: key.to_string(),
            weight,
        }
    }
}

/// Phần của một bên sau khi chia.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Share {
    /// `key` của bên nhận.
    pub key: String,
    /// Số tiền được chia, cùng dấu với tổng gốc (hoặc 0).
    pub amount: Money,
}

/// Lý do không chia được.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllocError {
    /// Không có bên nào.
    NoParties,
    /// Mọi trọng số bằng 0: không có tỷ lệ để chia.
    ZeroWeightSum,
    /// Hai bên cùng `key`: không phá hòa tất định được.
    DuplicateKey(String),
    /// `allocate_with_plug` không thấy bên nhận phần còn lại.
    UnknownPlug(String),
    /// Phép tính vượt giới hạn số (không xảy ra với số liệu hợp lý).
    Overflow,
}

impl fmt::Display for AllocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllocError::NoParties => write!(f, "không có bên nào để chia"),
            AllocError::ZeroWeightSum => write!(f, "tổng trọng số bằng 0"),
            AllocError::DuplicateKey(k) => write!(f, "key trùng: {k}"),
            AllocError::UnknownPlug(k) => write!(f, "không có bên nhận phần còn lại: {k}"),
            AllocError::Overflow => write!(f, "tràn số"),
        }
    }
}

impl std::error::Error for AllocError {}

/// Kiểm đầu vào, trả tổng trọng số (khác 0).
fn checked_weight_sum(parties: &[Party]) -> Result<i128, AllocError> {
    if parties.is_empty() {
        return Err(AllocError::NoParties);
    }
    let mut seen = BTreeSet::new();
    for p in parties {
        if !seen.insert(p.key.as_str()) {
            return Err(AllocError::DuplicateKey(p.key.clone()));
        }
    }
    let sum = parties.iter().try_fold(0i128, |acc, p| {
        acc.checked_add(i128::from(p.weight))
            .ok_or(AllocError::Overflow)
    })?;
    if sum == 0 {
        return Err(AllocError::ZeroWeightSum);
    }
    Ok(sum)
}

/// Áp dấu của tổng gốc lên độ lớn rồi đổi về `Money`.
fn signed_money(magnitude: i128, negative: bool) -> Result<Money, AllocError> {
    let signed = if negative { -magnitude } else { magnitude };
    i64::try_from(signed)
        .map(Money::from_dong)
        .map_err(|_| AllocError::Overflow)
}

/// Chia `total` theo trọng số các bên bằng largest remainder.
///
/// Mỗi bên nhận phần nguyên (làm tròn về 0) của `|total| * weight / tổng trọng số`; số đồng
/// còn dư cộng thêm 1 đồng cho các bên có phần dư lớn nhất, hòa thì `key` nhỏ hơn đứng trước.
/// Số âm (hoàn tiền) chia theo độ lớn rồi mang dấu âm, nên `allocate(-t)` là đúng đối xứng của
/// `allocate(t)`: hoàn toàn bộ trả lại đúng từng đồng, kể cả đồng lẻ. Kết quả theo thứ tự đầu vào.
///
/// Tích `|total| * weight` tính bằng `i128`: `|i64::MIN|` là 2^63, trọng số `u64` dưới 2^64,
/// tích dưới 2^127 nên không tràn. Bằng `i64` thì tràn ngay với trọng số cỡ 10^15.
///
/// # Errors
/// `NoParties`, `ZeroWeightSum`, `DuplicateKey`, `Overflow`.
pub fn allocate(total: Money, parties: &[Party]) -> Result<Vec<Share>, AllocError> {
    let weight_sum = checked_weight_sum(parties)?;
    let magnitude = i128::from(total.dong()).abs();

    // (phần nguyên, phần dư) của từng bên. Phần dư là tử số của phân số dư / weight_sum.
    let mut quotas: Vec<(i128, i128)> = parties
        .iter()
        .map(|p| {
            let exact = magnitude * i128::from(p.weight);
            (exact / weight_sum, exact % weight_sum)
        })
        .collect();
    let assigned: i128 = quotas.iter().map(|&(base, _)| base).sum();
    let leftover = usize::try_from(magnitude - assigned).map_err(|_| AllocError::Overflow)?;

    // Phần dư lớn nhất trước; hòa thì key nhỏ hơn trước. key duy nhất nên thứ tự là toàn phần.
    let mut order: Vec<usize> = (0..parties.len()).collect();
    order.sort_by_key(|&i| (Reverse(quotas[i].1), parties[i].key.as_str()));
    for &i in order.iter().take(leftover) {
        quotas[i].0 += 1;
    }

    let negative = total.dong() < 0;
    parties
        .iter()
        .zip(&quotas)
        .map(|(p, &(share, _))| {
            Ok(Share {
                key: p.key.clone(),
                amount: signed_money(share, negative)?,
            })
        })
        .collect()
}

/// Cách làm KHÁC để so sánh: mọi bên trừ `plug_key` được làm tròn nửa ra xa 0, bên `plug_key`
/// nhận phần còn lại. Đây là cách `FeePolicy::split` đang làm (phí làm tròn, tài xế là phần còn lại).
/// Tổng luôn khớp, nhưng bên nhận phần còn lại có thể lệch hơn 1 đồng, thậm chí âm.
///
/// # Errors
/// Như [`allocate`], thêm `UnknownPlug`.
pub fn allocate_with_plug(
    total: Money,
    parties: &[Party],
    plug_key: &str,
) -> Result<Vec<Share>, AllocError> {
    let weight_sum = checked_weight_sum(parties)?;
    if !parties.iter().any(|p| p.key == plug_key) {
        return Err(AllocError::UnknownPlug(plug_key.to_string()));
    }
    let magnitude = i128::from(total.dong()).abs();
    let negative = total.dong() < 0;

    let mut rounded: Vec<i128> = Vec::with_capacity(parties.len());
    for p in parties {
        let exact = magnitude * i128::from(p.weight);
        let (base, rem) = (exact / weight_sum, exact % weight_sum);
        // Nửa ra xa 0 trên số không âm. So `rem >= sum - rem` thay vì `2 * exact + sum`:
        // `exact` gần 2^127 nên nhân đôi sẽ tràn `i128`.
        rounded.push(base + i128::from(rem >= weight_sum - rem));
    }
    let others: i128 = parties
        .iter()
        .zip(&rounded)
        .filter(|(p, _)| p.key != plug_key)
        .map(|(_, &r)| r)
        .sum();
    parties
        .iter()
        .zip(&rounded)
        .map(|(p, &r)| {
            let magnitude_share = if p.key == plug_key {
                magnitude - others
            } else {
                r
            };
            Ok(Share {
                key: p.key.clone(),
                amount: signed_money(magnitude_share, negative)?,
            })
        })
        .collect()
}

/// Tổng số đồng của các phần (i128).
pub fn total_of(shares: &[Share]) -> i128 {
    shares.iter().map(|s| i128::from(s.amount.dong())).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parties(spec: &[(&str, u64)]) -> Vec<Party> {
        spec.iter().map(|&(k, w)| Party::new(k, w)).collect()
    }

    fn amounts(shares: &[Share]) -> Vec<i64> {
        shares.iter().map(|s| s.amount.dong()).collect()
    }

    #[test]
    fn canonical_split_of_the_fare() {
        let shares = allocate(
            Money::from_dong(25_500),
            &parties(&[("platform:revenue", 20), ("driver:DR-001", 80)]),
        )
        .unwrap();
        assert_eq!(amounts(&shares), vec![5_100, 20_400]);
    }

    #[test]
    fn hundred_by_three_gives_the_odd_dong_to_the_smallest_key() {
        let shares = allocate(
            Money::from_dong(100),
            &parties(&[("c", 1), ("a", 1), ("b", 1)]),
        )
        .unwrap();
        // key nhỏ nhất ("a") nhận đồng lẻ, dù đứng giữa danh sách.
        assert_eq!(amounts(&shares), vec![33, 34, 33]);
    }

    #[test]
    fn invalid_inputs_are_errors() {
        let m = Money::from_dong(100);
        assert_eq!(allocate(m, &[]), Err(AllocError::NoParties));
        assert_eq!(
            allocate(m, &parties(&[("a", 0), ("b", 0)])),
            Err(AllocError::ZeroWeightSum)
        );
        assert_eq!(
            allocate(m, &parties(&[("a", 1), ("a", 2)])),
            Err(AllocError::DuplicateKey("a".to_string()))
        );
        assert_eq!(
            allocate_with_plug(m, &parties(&[("a", 1)]), "zzz"),
            Err(AllocError::UnknownPlug("zzz".to_string()))
        );
    }

    #[test]
    fn zero_weight_party_gets_nothing() {
        let shares = allocate(
            Money::from_dong(10),
            &parties(&[("a", 1), ("b", 0), ("c", 1), ("d", 1)]),
        )
        .unwrap();
        assert_eq!(amounts(&shares), vec![4, 0, 3, 3]);
    }

    #[test]
    fn refund_mirrors_the_original() {
        let ps = parties(&[("a", 1), ("b", 1), ("c", 1)]);
        let paid = allocate(Money::from_dong(25_501), &ps).unwrap();
        let refund = allocate(Money::from_dong(-25_501), &ps).unwrap();
        assert_eq!(amounts(&paid), vec![8_501, 8_500, 8_500]);
        assert_eq!(amounts(&refund), vec![-8_501, -8_500, -8_500]);
    }

    // Hoàn một phần nhiều lần: chia riêng từng lần cho kết quả khác chia theo số lũy kế.
    #[test]
    fn partial_refunds_are_consistent_only_when_cumulative() {
        let ps = parties(&[("a", 1), ("b", 1), ("c", 1)]);
        let each = allocate(Money::from_dong(10_000), &ps).unwrap();
        assert_eq!(amounts(&each), vec![3_334, 3_333, 3_333]);
        // Hai lần hoàn 10.000, mỗi lần chia riêng: a nhận 6.668 trong khi chia 20.000 một lần là 6.667.
        let once = allocate(Money::from_dong(20_000), &ps).unwrap();
        assert_eq!(amounts(&once), vec![6_667, 6_667, 6_666]);
        // Lần hai tính theo lũy kế: phần của lần hai = chia(20.000) - phần đã hoàn lần một.
        let second: Vec<i64> = once
            .iter()
            .zip(&each)
            .map(|(total, first)| total.amount.dong() - first.amount.dong())
            .collect();
        assert_eq!(second, vec![3_333, 3_334, 3_333]);
        assert_eq!(second.iter().sum::<i64>(), 10_000);
    }

    #[test]
    fn extreme_totals_do_not_overflow() {
        let ps = parties(&[("a", u64::MAX), ("b", u64::MAX)]);
        let min = allocate(Money::from_dong(i64::MIN), &ps).unwrap();
        assert_eq!(total_of(&min), i128::from(i64::MIN));
        let max = allocate(Money::from_dong(i64::MAX), &ps).unwrap();
        assert_eq!(total_of(&max), i128::from(i64::MAX));
    }

    #[test]
    fn plug_does_not_overflow_on_extreme_totals() {
        let ps = parties(&[("a", u64::MAX), ("b", u64::MAX)]);
        let min = allocate_with_plug(Money::from_dong(i64::MIN), &ps, "b").unwrap();
        assert_eq!(total_of(&min), i128::from(i64::MIN));
    }

    #[test]
    fn plug_can_miss_by_more_than_one_dong() {
        // 6 đồng cho 4 bên bằng nhau: mỗi bên chính xác 1,5 đồng.
        let ps = parties(&[("a", 1), ("b", 1), ("c", 1), ("d", 1)]);
        let plugged = allocate_with_plug(Money::from_dong(6), &ps, "d").unwrap();
        assert_eq!(amounts(&plugged), vec![2, 2, 2, 0]);
        let fair = allocate(Money::from_dong(6), &ps).unwrap();
        assert_eq!(amounts(&fair), vec![2, 2, 1, 1]);
    }
}
