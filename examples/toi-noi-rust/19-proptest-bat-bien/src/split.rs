//! Chia tiền cho nhiều bên không mất đồng: phương pháp largest remainder.

use std::error::Error;
use std::fmt;

use crate::money::Money;

/// Lý do không chia được.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitError {
    /// Danh sách trọng số rỗng.
    NoParties,
    /// Mọi trọng số bằng 0, không có tỷ lệ để chia.
    ZeroWeights,
}

impl fmt::Display for SplitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SplitError::NoParties => write!(f, "không có bên nào để chia"),
            SplitError::ZeroWeights => write!(f, "mọi trọng số bằng 0"),
        }
    }
}

impl Error for SplitError {}

/// Phần nguyên và phần dư của `magnitude * weight / weight_sum` cho từng bên.
fn floor_shares(magnitude: i128, weights: &[u32], weight_sum: i128) -> Vec<(i128, i128)> {
    weights
        .iter()
        .map(|&w| {
            let exact = magnitude * i128::from(w);
            (exact / weight_sum, exact % weight_sum)
        })
        .collect()
}

fn to_money(shares: &[(i128, i128)], negative: bool) -> Vec<Money> {
    shares
        .iter()
        .map(|&(share, _)| {
            let signed = if negative { -share } else { share };
            // Mỗi phần có độ lớn không vượt tổng gốc nên luôn vừa i64.
            Money::from_dong(i64::try_from(signed).expect("phần chia vừa i64"))
        })
        .collect()
}

fn weight_sum(weights: &[u32]) -> Result<i128, SplitError> {
    if weights.is_empty() {
        return Err(SplitError::NoParties);
    }
    let sum: i128 = weights.iter().map(|&w| i128::from(w)).sum();
    if sum == 0 {
        return Err(SplitError::ZeroWeights);
    }
    Ok(sum)
}

/// Chia `total` theo `weights`; tổng các phần luôn đúng bằng `total`.
///
/// Mỗi bên nhận phần nguyên (làm tròn xuống) trước. Số đồng còn dư được cộng thêm
/// 1 đồng cho các bên có phần dư lớn nhất; hòa thì bên đứng trước được ưu tiên.
///
/// ```
/// use ledger_lab::{Money, split};
///
/// let parts = split(Money::from_dong(25_500), &[80, 20]).unwrap();
/// assert_eq!(parts, vec![Money::from_dong(20_400), Money::from_dong(5_100)]);
/// ```
pub fn split(total: Money, weights: &[u32]) -> Result<Vec<Money>, SplitError> {
    let sum = weight_sum(weights)?;
    let magnitude = i128::from(total.dong()).abs();
    let mut shares = floor_shares(magnitude, weights, sum);
    let assigned: i128 = shares.iter().map(|&(base, _)| base).sum();
    let leftover = usize::try_from(magnitude - assigned).expect("số đồng dư nhỏ hơn số bên");

    // Phần dư lớn nhất đứng trước; hòa thì chỉ số nhỏ hơn đứng trước.
    let mut order: Vec<usize> = (0..shares.len()).collect();
    order.sort_by(|&a, &b| shares[b].1.cmp(&shares[a].1).then(a.cmp(&b)));
    for &i in order.iter().take(leftover) {
        shares[i].0 += 1;
    }
    Ok(to_money(&shares, total.dong() < 0))
}

/// Bản SAI, cố ý để xem shrinking: chỉ làm tròn xuống, quên phân phát số đồng dư.
/// Không dùng ngoài bài học.
pub fn split_floor_only(total: Money, weights: &[u32]) -> Result<Vec<Money>, SplitError> {
    let sum = weight_sum(weights)?;
    let magnitude = i128::from(total.dong()).abs();
    let shares = floor_shares(magnitude, weights, sum);
    Ok(to_money(&shares, total.dong() < 0))
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::strategies;

    fn dong(parts: &[Money]) -> Vec<i64> {
        parts.iter().map(|m| m.dong()).collect()
    }

    // Example test: vài giá trị ta tự chọn và tự tính tay.
    #[test]
    fn example_splits() {
        let ten = Money::from_dong(10);
        assert_eq!(dong(&split(ten, &[1, 1, 1]).unwrap()), vec![4, 3, 3]);
        let fare = Money::from_dong(25_500);
        assert_eq!(dong(&split(fare, &[80, 20]).unwrap()), vec![20_400, 5_100]);
        assert_eq!(dong(&split(fare, &[1, 1, 1]).unwrap()), vec![8_500; 3]);
        assert_eq!(split(fare, &[]), Err(SplitError::NoParties));
        assert_eq!(split(fare, &[0, 0]), Err(SplitError::ZeroWeights));
    }

    proptest! {
        // Property test: đúng với mọi tổng i64 và mọi bộ trọng số hợp lệ.
        #[test]
        fn parts_sum_to_total((total, weights) in strategies::split_input()) {
            let parts = split(Money::from_dong(total), &weights).unwrap();
            prop_assert_eq!(parts.len(), weights.len());
            let sum: i128 = parts.iter().map(|m| i128::from(m.dong())).sum();
            prop_assert_eq!(sum, i128::from(total));
        }

        // Mỗi bên lệch phần chính xác (tổng × trọng số / tổng trọng số) dưới 1 đồng.
        #[test]
        fn each_part_is_within_one_dong((total, weights) in strategies::split_input()) {
            let parts = split(Money::from_dong(total), &weights).unwrap();
            let weight_sum: i128 = weights.iter().map(|&w| i128::from(w)).sum();
            for (part, &w) in parts.iter().zip(&weights) {
                let diff = i128::from(part.dong()) * weight_sum - i128::from(total) * i128::from(w);
                prop_assert!(diff.abs() < weight_sum);
            }
        }

        // Chia đều cho n bên: hai phần bất kỳ chênh nhau không quá 1 đồng.
        #[test]
        fn equal_weights_differ_by_at_most_one(total in any::<i64>(), n in 1usize..=8) {
            let parts = split(Money::from_dong(total), &vec![1; n]).unwrap();
            let dongs: Vec<i128> = parts.iter().map(|m| i128::from(m.dong())).collect();
            let (min, max) = (dongs.iter().min().unwrap(), dongs.iter().max().unwrap());
            prop_assert!(max - min <= 1);
        }

        // Bên có trọng số 0 không nhận đồng nào.
        #[test]
        fn zero_weight_gets_nothing((total, weights) in strategies::split_input()) {
            let parts = split(Money::from_dong(total), &weights).unwrap();
            for (part, &w) in parts.iter().zip(&weights) {
                if w == 0 {
                    prop_assert_eq!(*part, Money::ZERO);
                }
            }
        }
    }
}
