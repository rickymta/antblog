//! Cách làm SAI: làm tròn từng phần riêng lẻ rồi hy vọng tổng khớp.

use rust_decimal::{Decimal, RoundingStrategy};

use crate::strategies::round_dong;

/// Phần chính xác `total * weight / sum` dạng `Decimal` (nhân trước, chia sau).
/// `None` nếu tổng trọng số bằng 0 hoặc phép tính tràn.
pub fn exact_share(total: i64, weight: u32, weight_sum: u32) -> Option<Decimal> {
    if weight_sum == 0 {
        return None;
    }
    Decimal::from(total)
        .checked_mul(Decimal::from(weight))?
        .checked_div(Decimal::from(weight_sum))
}

/// Làm tròn từng phần bằng `strategy`, không bù gì. Tổng các phần có thể khác `total`.
pub fn round_each(total: i64, weights: &[u32], strategy: RoundingStrategy) -> Option<Vec<i64>> {
    let sum: u32 = weights
        .iter()
        .try_fold(0u32, |acc, &w| acc.checked_add(w))?;
    weights
        .iter()
        .map(|&w| round_dong(exact_share(total, w, sum)?, strategy))
        .collect()
}

/// Tổng số đồng của các phần (i128 để không tràn khi ghép nhiều phần).
pub fn sum_of(parts: &[i64]) -> i128 {
    parts.iter().map(|&p| i128::from(p)).sum()
}

/// Lệch của tổng các phần so với tổng gốc: dương là thừa, âm là thiếu.
pub fn drift(total: i64, parts: &[i64]) -> i128 {
    sum_of(parts) - i128::from(total)
}
