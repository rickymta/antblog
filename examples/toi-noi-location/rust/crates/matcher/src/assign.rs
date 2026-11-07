//! Bài toán phân công trên ma trận chi phí số nguyên: tham lam theo thứ tự, tham lam toàn cục, và Hungarian (Kuhn–Munkres).
//!
//! Quy ước: hàng = khách, cột = tài xế, ô = chi phí (ms, có thể âm sau khi trừ thưởng); `FORBIDDEN` = cấm cặp đó.
//! Mọi hàm trả danh sách `(khách, tài xế)`: mỗi khách và mỗi tài xế **tối đa một lần**, không bao giờ chứa cặp bị cấm.

use crate::cost::{Dense, FORBIDDEN};
use pathfinding::prelude::{Matrix, kuhn_munkres};

/// Tổng chi phí của một phép gán.
pub fn total_cost(costs: &Dense, pairs: &[(usize, usize)]) -> i64 {
    pairs.iter().map(|&(r, c)| costs.get(r, c)).sum()
}

fn allowed(v: i64) -> bool {
    v < FORBIDDEN / 2
}

/// Tham lam theo thứ tự hàng (khách đặt trước chọn trước): mỗi khách lấy tài xế rẻ nhất còn trống. Đây là «gần nhất theo ETA»
/// khi hàng xếp theo giờ đặt; hoà thì chọn cột nhỏ hơn.
pub fn greedy_fifo(costs: &Dense) -> Vec<(usize, usize)> {
    let mut taken = vec![false; costs.cols];
    let mut out = Vec::new();
    for r in 0..costs.rows {
        let mut best: Option<(i64, usize)> = None;
        for (c, &is_taken) in taken.iter().enumerate() {
            let v = costs.get(r, c);
            if !is_taken && allowed(v) && best.is_none_or(|(b, _)| v < b) {
                best = Some((v, c));
            }
        }
        if let Some((_, c)) = best {
            taken[c] = true;
            out.push((r, c));
        }
    }
    out
}

/// Tham lam toàn cục: lặp lại việc lấy cặp rẻ nhất trong mọi cặp còn lại. Mạnh hơn `greedy_fifo` nhưng vẫn không tối ưu.
pub fn greedy_global(costs: &Dense) -> Vec<(usize, usize)> {
    let mut cells: Vec<(i64, usize, usize)> = Vec::new();
    for r in 0..costs.rows {
        for c in 0..costs.cols {
            let v = costs.get(r, c);
            if allowed(v) {
                cells.push((v, r, c));
            }
        }
    }
    cells.sort_unstable();
    let (mut rt, mut ct) = (vec![false; costs.rows], vec![false; costs.cols]);
    let mut out = Vec::new();
    for (_, r, c) in cells {
        if !rt[r] && !ct[c] {
            rt[r] = true;
            ct[c] = true;
            out.push((r, c));
        }
    }
    out.sort_unstable();
    out
}

/// Hungarian: tối đa số cặp được phép trước, rồi tối thiểu tổng chi phí. Ma trận chữ nhật bất kỳ (thừa khách hoặc thừa xe).
///
/// `pathfinding::kuhn_munkres` *tối đa hoá* tổng trọng số và đòi số hàng ≤ số cột, nên: chuyển ma trận nếu thừa khách; đổi
/// chi phí thành trọng số `M − chi phí` với `M` lớn hơn mọi tổng chi phí (một cặp thêm luôn thắng mọi tiết kiệm chi phí), ô bị cấm
/// nhận trọng số 0 và bị loại khỏi kết quả. Độ phức tạp O(r²·c) cho r ≤ c (r = min(hàng, cột)).
pub fn hungarian(costs: &Dense) -> Vec<(usize, usize)> {
    if costs.rows == 0 || costs.cols == 0 {
        return Vec::new();
    }
    let transposed = costs.rows > costs.cols;
    let m = if transposed {
        costs.transpose()
    } else {
        costs.clone()
    };
    let (lo, hi) = m
        .data
        .iter()
        .filter(|&&v| allowed(v))
        .fold((i64::MAX, i64::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    if lo > hi {
        return Vec::new(); // không cặp nào được phép
    }
    // M > r · (khoảng dao động chi phí) để thêm một cặp luôn có lợi hơn mọi tiết kiệm chi phí
    let span = hi - lo + 1;
    let big = (m.rows as i64 + 1) * span + 1;
    let weights: Vec<i64> = m
        .data
        .iter()
        .map(|&v| if allowed(v) { big - (v - lo) } else { 0 })
        .collect();
    let w = Matrix::from_vec(m.rows, m.cols, weights).expect("kích thước khớp");
    let (_, cols_of_row) = kuhn_munkres(&w);
    let mut out: Vec<(usize, usize)> = cols_of_row
        .iter()
        .enumerate()
        .filter(|&(r, &c)| allowed(m.get(r, c)))
        .map(|(r, &c)| if transposed { (c, r) } else { (r, c) })
        .collect();
    out.sort_unstable();
    out
}

/// Vét cạn (chỉ cho ma trận nhỏ): tối đa số cặp được phép, rồi tối thiểu tổng chi phí. Dùng làm chuẩn đối chiếu cho `hungarian`.
pub fn brute_force(costs: &Dense) -> Vec<(usize, usize)> {
    fn rec(
        costs: &Dense,
        r: usize,
        used: &mut Vec<bool>,
        cur: &mut Vec<(usize, usize)>,
        cost: i64,
        best: &mut (usize, i64, Vec<(usize, usize)>),
    ) {
        if r == costs.rows {
            if cur.len() > best.0 || (cur.len() == best.0 && cost < best.1) {
                *best = (cur.len(), cost, cur.clone());
            }
            return;
        }
        rec(costs, r + 1, used, cur, cost, best); // bỏ qua khách r
        for c in 0..costs.cols {
            let v = costs.get(r, c);
            if !used[c] && allowed(v) {
                used[c] = true;
                cur.push((r, c));
                rec(costs, r + 1, used, cur, cost + v, best);
                cur.pop();
                used[c] = false;
            }
        }
    }
    let mut best = (0usize, i64::MAX, Vec::new());
    rec(
        costs,
        0,
        &mut vec![false; costs.cols],
        &mut Vec::new(),
        0,
        &mut best,
    );
    best.2
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(rows: &[&[i64]]) -> Dense {
        Dense::from_fn(rows.len(), rows[0].len(), |r, c| rows[r][c])
    }

    #[test]
    fn greedy_steals_the_driver_hungarian_does_not() {
        // khách 0 đến trước và «gần» tài xế 0 (10 so với 12); khách 1 chỉ có tài xế 0 là gần (11 so với 100)
        let c = m(&[&[10, 12], &[11, 100]]);
        let g = greedy_fifo(&c);
        let h = hungarian(&c);
        assert_eq!(g, vec![(0, 0), (1, 1)]);
        assert_eq!(total_cost(&c, &g), 110);
        assert_eq!(h, vec![(0, 1), (1, 0)]);
        assert_eq!(total_cost(&c, &h), 23);
    }

    #[test]
    fn rectangular_more_customers_than_drivers_and_the_reverse() {
        let c = m(&[&[5, 9], &[6, 1], &[7, 8]]); // 3 khách, 2 xe
        let h = hungarian(&c);
        assert_eq!(h.len(), 2);
        assert_eq!(total_cost(&c, &h), total_cost(&c, &brute_force(&c)));
        let t = c.transpose(); // 2 khách, 3 xe
        let h = hungarian(&t);
        assert_eq!(h.len(), 2);
        assert_eq!(total_cost(&t, &h), total_cost(&t, &brute_force(&t)));
    }

    #[test]
    fn forbidden_pairs_are_never_assigned_and_cardinality_comes_first() {
        // khách 1 chỉ đi được với xe 0; khách 0 rẻ nhất với xe 0 nhưng cũng đi được xe 1
        let c = m(&[&[1, 50], &[2, FORBIDDEN]]);
        let h = hungarian(&c);
        assert_eq!(h, vec![(0, 1), (1, 0)]);
        let none = m(&[&[FORBIDDEN, FORBIDDEN]]);
        assert!(hungarian(&none).is_empty() && greedy_fifo(&none).is_empty());
    }

    #[test]
    fn negative_costs_are_handled() {
        let c = m(&[&[-5, 3], &[2, -7]]);
        assert_eq!(hungarian(&c), vec![(0, 0), (1, 1)]);
    }
}
