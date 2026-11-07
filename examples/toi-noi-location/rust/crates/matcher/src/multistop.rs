//! LOC-50: một người giao hàng, `n` cặp (lấy → giao), lấy **trước** giao. Tuyến mở (không về điểm xuất phát),
//! tối thiểu tổng thời gian chạy. Không có tải trọng, không có khung giờ: chỉ ràng buộc thứ tự.
//!
//! Quy ước đỉnh trong ma trận `(2n + 1) × (2n + 1)`: đỉnh 0 là điểm xuất phát, đỉnh `1 + 2k` là điểm lấy của cặp `k`,
//! đỉnh `2 + 2k` là điểm giao của cặp `k`. Ma trận **bất đối xứng** (đường một chiều), đơn vị mili giây nguyên.
//!
//! Ba thành phần, từ rẻ tới đắt:
//! 1. [`cheapest_insertion`]: chèn cặp rẻ nhất vào tuyến hiện có, mỗi cặp thử mọi vị trí lấy rồi mọi vị trí giao sau nó.
//!    Độ phức tạp `O(n² · L²)` với `L = 2n + 1`, mỗi ô thử tính delta trong `O(1)`.
//! 2. [`local_search`]: 2-opt (đảo một đoạn), Or-opt (dời đoạn 1–3 đỉnh) và dời nguyên một cặp, lặp tới khi hết cải thiện.
//!    Mỗi lượt quét 2-opt và Or-opt `O(L²)`; mọi bước chỉ nhận khi **giảm** tổng chi phí nên luôn dừng.
//! 3. [`exact_dp`]: quy hoạch động theo tập đỉnh đã đi, chính xác nhưng `O(4ⁿ · n²)` nên chỉ dùng làm mốc cho `n ≤ 9`.
//!
//! Mọi thứ tất định (hoà thì lấy chỉ số nhỏ hơn). Đường nóng dùng [`solve`] với ngân sách thời gian.

use std::time::{Duration, Instant};

/// Chi phí thay cho cặp không tới được (lớn nhưng cộng được hàng trăm lần mà không tràn `i64`).
pub const UNREACHABLE_MS: i64 = 100_000_000;
/// Số cặp tối đa của [`exact_dp`].
pub const DP_MAX_PAIRS: usize = 9;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    pairs: usize,
    nodes: usize,
    cost: Vec<i64>,
}

impl Instance {
    /// `cost` là ma trận hàng-chính `(2·pairs + 1)²`, mili giây, không âm.
    pub fn new(pairs: usize, cost: Vec<i64>) -> Result<Self, String> {
        let nodes = 2 * pairs + 1;
        if cost.len() != nodes * nodes {
            return Err(format!(
                "ma trận cần {nodes}² = {} ô, nhận {}",
                nodes * nodes,
                cost.len()
            ));
        }
        if cost.iter().any(|&c| c < 0) {
            return Err("chi phí không được âm".into());
        }
        Ok(Self { pairs, nodes, cost })
    }

    pub fn pairs(&self) -> usize {
        self.pairs
    }
    pub fn nodes(&self) -> usize {
        self.nodes
    }
    pub fn matrix(&self) -> &[i64] {
        &self.cost
    }
    #[inline]
    pub fn c(&self, a: usize, b: usize) -> i64 {
        self.cost[a * self.nodes + b]
    }
    pub fn pickup(k: usize) -> usize {
        1 + 2 * k
    }
    pub fn delivery(k: usize) -> usize {
        2 + 2 * k
    }
    /// Cặp của đỉnh (`None` với điểm xuất phát).
    pub fn pair_of(node: usize) -> Option<usize> {
        (node > 0).then(|| (node - 1) / 2)
    }
    pub fn is_pickup(node: usize) -> bool {
        node % 2 == 1
    }
    /// Đỉnh cùng cặp còn lại.
    fn partner(node: usize) -> usize {
        if Self::is_pickup(node) {
            node + 1
        } else {
            node - 1
        }
    }

    /// Tổng chi phí của một tuyến (dãy đỉnh, bắt đầu ở 0).
    pub fn route_cost(&self, route: &[usize]) -> i64 {
        route.windows(2).map(|w| self.c(w[0], w[1])).sum()
    }

    /// Tuyến hợp lệ: bắt đầu ở 0, mỗi đỉnh đúng một lần, lấy trước giao.
    pub fn is_feasible(&self, route: &[usize]) -> bool {
        if route.len() != self.nodes || route[0] != 0 {
            return false;
        }
        let mut pos = vec![usize::MAX; self.nodes];
        for (i, &v) in route.iter().enumerate() {
            if v >= self.nodes || pos[v] != usize::MAX {
                return false;
            }
            pos[v] = i;
        }
        (0..self.pairs).all(|k| pos[Self::pickup(k)] < pos[Self::delivery(k)])
    }
}

/// Chi phí chèn một đỉnh `x` vào giữa `a` và `b` (`b = None`: chèn vào cuối tuyến mở).
#[inline]
fn ins(inst: &Instance, a: usize, x: usize, b: Option<usize>) -> i64 {
    match b {
        Some(b) => inst.c(a, x) + inst.c(x, b) - inst.c(a, b),
        None => inst.c(a, x),
    }
}

/// Vị trí chèn tốt nhất của cặp `k` vào `route` (không chứa cặp đó): `(delta, i, m)`.
/// Điểm lấy chèn ngay sau `route[i]`; điểm giao chèn ngay sau điểm lấy nếu `m == i`, ngược lại ngay sau `route[m]` (`m > i`).
fn best_pair_insertion(inst: &Instance, route: &[usize], k: usize) -> (i64, usize, usize) {
    let (p, d) = (Instance::pickup(k), Instance::delivery(k));
    let l = route.len();
    let mut best = (i64::MAX, 0, 0);
    for i in 0..l {
        let a = route[i];
        let b = route.get(i + 1).copied();
        let pick = ins(inst, a, p, b);
        // giao liền sau điểm lấy: a → p → d → b
        let direct = match b {
            Some(b) => inst.c(a, p) + inst.c(p, d) + inst.c(d, b) - inst.c(a, b),
            None => inst.c(a, p) + inst.c(p, d),
        };
        if direct < best.0 {
            best = (direct, i, i);
        }
        for m in (i + 1)..l {
            let delta = pick + ins(inst, route[m], d, route.get(m + 1).copied());
            if delta < best.0 {
                best = (delta, i, m);
            }
        }
    }
    best
}

fn apply_pair_insertion(route: &[usize], k: usize, i: usize, m: usize) -> Vec<usize> {
    let (p, d) = (Instance::pickup(k), Instance::delivery(k));
    let mut out = Vec::with_capacity(route.len() + 2);
    for (idx, &v) in route.iter().enumerate() {
        out.push(v);
        if idx == i {
            out.push(p);
            if m == i {
                out.push(d);
            }
        }
        if m > i && idx == m {
            out.push(d);
        }
    }
    out
}

/// Chèn rẻ nhất: mỗi bước chọn cặp chưa vào tuyến có chi phí chèn nhỏ nhất rồi chèn vào đúng chỗ đó.
pub fn cheapest_insertion(inst: &Instance) -> Vec<usize> {
    insertion_until(inst, None).0
}

/// Như [`cheapest_insertion`] nhưng quá hạn thì nối nốt các cặp còn lại (lấy rồi giao) vào cuối. Trả `(tuyến, hết hạn?)`.
fn insertion_until(inst: &Instance, deadline: Option<Instant>) -> (Vec<usize>, bool) {
    let mut route = vec![0usize];
    let mut left: Vec<usize> = (0..inst.pairs).collect();
    while !left.is_empty() {
        if deadline.is_some_and(|d| Instant::now() >= d) {
            for k in left {
                route.push(Instance::pickup(k));
                route.push(Instance::delivery(k));
            }
            return (route, true);
        }
        let mut best: Option<(i64, usize, usize, usize)> = None; // (delta, vị trí trong left, i, m)
        for (idx, &k) in left.iter().enumerate() {
            let (delta, i, m) = best_pair_insertion(inst, &route, k);
            if best.is_none_or(|b| delta < b.0) {
                best = Some((delta, idx, i, m));
            }
        }
        let (_, idx, i, m) = best.expect("left không rỗng");
        let k = left.remove(idx);
        route = apply_pair_insertion(&route, k, i, m);
    }
    (route, false)
}

/// Kết quả [`local_search`]/[`solve`].
#[derive(Debug, Clone)]
pub struct Solution {
    pub route: Vec<usize>,
    pub cost: i64,
    /// Chi phí ngay sau bước chèn (trước tìm kiếm cục bộ).
    pub insertion_cost: i64,
    /// Số bước cải thiện đã áp dụng.
    pub moves: usize,
    /// Hết ngân sách thời gian trước khi tới cực tiểu cục bộ.
    pub timed_out: bool,
}

#[derive(Clone, Copy)]
enum Move {
    TwoOpt { i: usize, j: usize },
    OrOpt { i: usize, len: usize, a: usize },
    Pair { k: usize, i: usize, m: usize },
}

/// 2-opt tốt nhất: đảo đoạn `route[i..=j]`. Đảo làm lật thứ tự mọi cặp nằm trọn trong đoạn nên đoạn chứa cả điểm lấy lẫn
/// điểm giao của một cặp là không hợp lệ; với `i` cố định, `j` tăng dần thì gặp điểm giao có điểm lấy đã trong đoạn là dừng.
fn best_two_opt(inst: &Instance, r: &[usize]) -> Option<(i64, Move)> {
    let l = r.len();
    let mut best: Option<(i64, Move)> = None;
    let mut in_seg = vec![false; inst.nodes];
    for i in 1..l.saturating_sub(1) {
        in_seg.iter_mut().for_each(|x| *x = false);
        let (mut fwd, mut bwd) = (0i64, 0i64);
        in_seg[r[i]] = true;
        for j in (i + 1)..l {
            let v = r[j];
            if !Instance::is_pickup(v) && in_seg[Instance::partner(v)] {
                break;
            }
            in_seg[v] = true;
            fwd += inst.c(r[j - 1], v);
            bwd += inst.c(v, r[j - 1]);
            let prev = r[i - 1];
            let next = r.get(j + 1).copied();
            let old = inst.c(prev, r[i]) + fwd + next.map_or(0, |n| inst.c(v, n));
            let new = inst.c(prev, v) + bwd + next.map_or(0, |n| inst.c(r[i], n));
            let delta = new - old;
            if delta < 0 && best.is_none_or(|b| delta < b.0) {
                best = Some((delta, Move::TwoOpt { i, j }));
            }
        }
    }
    best
}

/// Or-opt tốt nhất: dời đoạn `len ∈ {1,2,3}` đỉnh liên tiếp (giữ chiều) tới ngay sau phần tử `a` của tuyến đã bỏ đoạn.
fn best_or_opt(inst: &Instance, r: &[usize]) -> Option<(i64, Move)> {
    let l = r.len();
    let mut best: Option<(i64, Move)> = None;
    let mut pos = vec![0usize; inst.nodes];
    for (i, &v) in r.iter().enumerate() {
        pos[v] = i;
    }
    for len in 1..=3usize {
        for i in 1..l {
            let last = i + len - 1;
            if last >= l {
                break;
            }
            let (s0, sl) = (r[i], r[last]);
            let prev = r[i - 1];
            let next = r.get(last + 1).copied();
            let removal = -inst.c(prev, s0) - next.map_or(0, |n| inst.c(sl, n))
                + next.map_or(0, |n| inst.c(prev, n));
            let lr = l - len;
            for a in 0..lr {
                if a == i - 1 {
                    continue; // đúng chỗ cũ
                }
                let red = |t: usize| if t < i { r[t] } else { r[t + len] };
                let (ea, eb) = (red(a), (a + 1 < lr).then(|| red(a + 1)));
                let insd = inst.c(ea, s0) + eb.map_or(0, |b| inst.c(sl, b) - inst.c(ea, b));
                let delta = removal + insd;
                if delta >= 0 || best.is_some_and(|b| delta >= b.0) {
                    continue;
                }
                // thứ tự lấy trước giao còn đúng sau khi dời?
                let ok = (i..=last).all(|t| {
                    let v = r[t];
                    let q = pos[Instance::partner(v)];
                    if (i..=last).contains(&q) {
                        return true;
                    }
                    if Instance::is_pickup(v) {
                        q - len > a // điểm giao (đã sau đoạn) phải ở sau chỗ chèn
                    } else {
                        q <= a // điểm lấy (đã trước đoạn) phải ở trước hoặc ngay tại chỗ chèn
                    }
                });
                if ok {
                    best = Some((delta, Move::OrOpt { i, len, a }));
                }
            }
        }
    }
    best
}

fn apply_or_opt(r: &[usize], i: usize, len: usize, a: usize) -> Vec<usize> {
    let seg = &r[i..i + len];
    let reduced: Vec<usize> = r[..i].iter().chain(&r[i + len..]).copied().collect();
    let mut out = Vec::with_capacity(r.len());
    out.extend_from_slice(&reduced[..=a]);
    out.extend_from_slice(seg);
    out.extend_from_slice(&reduced[a + 1..]);
    out
}

/// Dời nguyên một cặp tới vị trí chèn tốt nhất.
fn best_pair_move(inst: &Instance, r: &[usize]) -> Option<(i64, Move)> {
    let mut best: Option<(i64, Move)> = None;
    let old = inst.route_cost(r);
    for k in 0..inst.pairs {
        let (p, d) = (Instance::pickup(k), Instance::delivery(k));
        let reduced: Vec<usize> = r.iter().copied().filter(|&v| v != p && v != d).collect();
        let (_, i, m) = best_pair_insertion(inst, &reduced, k);
        let new = inst.route_cost(&apply_pair_insertion(&reduced, k, i, m));
        let delta = new - old;
        if delta < 0 && best.is_none_or(|b| delta < b.0) {
            best = Some((delta, Move::Pair { k, i, m }));
        }
    }
    best
}

fn apply_move(r: &[usize], mv: Move) -> Vec<usize> {
    match mv {
        Move::TwoOpt { i, j } => {
            let mut out = r.to_vec();
            out[i..=j].reverse();
            out
        }
        Move::OrOpt { i, len, a } => apply_or_opt(r, i, len, a),
        Move::Pair { k, i, m } => {
            let (p, d) = (Instance::pickup(k), Instance::delivery(k));
            let reduced: Vec<usize> = r.iter().copied().filter(|&v| v != p && v != d).collect();
            apply_pair_insertion(&reduced, k, i, m)
        }
    }
}

/// Tìm kiếm cục bộ từ `start` (tuyến hợp lệ): lần lượt 2-opt, Or-opt, dời cặp; áp dụng bước **tốt nhất** của loại đầu tiên
/// có cải thiện rồi quay lại từ đầu, tới khi không loại nào cải thiện (hoặc hết hạn).
pub fn local_search(
    inst: &Instance,
    start: Vec<usize>,
    deadline: Option<Instant>,
) -> (Vec<usize>, usize, bool) {
    let mut r = start;
    let mut moves = 0;
    loop {
        if deadline.is_some_and(|d| Instant::now() >= d) {
            return (r, moves, true);
        }
        let step = best_two_opt(inst, &r)
            .or_else(|| best_or_opt(inst, &r))
            .or_else(|| best_pair_move(inst, &r));
        match step {
            Some((_, mv)) => {
                r = apply_move(&r, mv);
                moves += 1;
            }
            None => return (r, moves, false),
        }
    }
}

/// Đường nóng: chèn rồi tìm kiếm cục bộ, dừng khi hết `budget` (nếu có). Luôn trả tuyến hợp lệ.
pub fn solve(inst: &Instance, budget: Option<Duration>) -> Solution {
    let deadline = budget.map(|b| Instant::now() + b);
    let (ins_route, t1) = insertion_until(inst, deadline);
    let insertion_cost = inst.route_cost(&ins_route);
    let (route, moves, t2) = local_search(inst, ins_route, deadline);
    Solution {
        cost: inst.route_cost(&route),
        route,
        insertion_cost,
        moves,
        timed_out: t1 || t2,
    }
}

/// Tối ưu chính xác bằng quy hoạch động theo tập đỉnh đã đi: `dp[tập][đỉnh cuối]`. Chỉ cho `pairs ≤ DP_MAX_PAIRS`.
/// Bộ nhớ `2^(2n) · 2n` ô; thời gian `O(2^(2n) · (2n)²)`.
pub fn exact_dp(inst: &Instance) -> Option<(Vec<usize>, i64)> {
    let n = inst.pairs;
    if n == 0 {
        return Some((vec![0], 0));
    }
    if n > DP_MAX_PAIRS {
        return None;
    }
    let s = 2 * n; // số điểm dừng; điểm dừng t ứng với đỉnh t + 1; chẵn = lấy, lẻ = giao của cặp t/2
    let full = 1usize << s;
    let mut dp = vec![i64::MAX; full * s];
    let mut parent = vec![u8::MAX; full * s];
    for t in (0..s).step_by(2) {
        dp[(1 << t) * s + t] = inst.c(0, t + 1);
    }
    for mask in 1..full {
        for last in 0..s {
            let cur = dp[mask * s + last];
            if mask & (1 << last) == 0 || cur == i64::MAX {
                continue;
            }
            for t in 0..s {
                if mask & (1 << t) != 0 || (t % 2 == 1 && mask & (1 << (t - 1)) == 0) {
                    continue;
                }
                let nm = mask | (1 << t);
                let v = cur + inst.c(last + 1, t + 1);
                if v < dp[nm * s + t] {
                    dp[nm * s + t] = v;
                    parent[nm * s + t] = last as u8;
                }
            }
        }
    }
    let (mut last, mut best) = (0, i64::MAX);
    for t in 0..s {
        let v = dp[(full - 1) * s + t];
        if v < best {
            best = v;
            last = t;
        }
    }
    let mut route = Vec::with_capacity(s + 1);
    let mut mask = full - 1;
    loop {
        route.push(last + 1);
        let p = parent[mask * s + last];
        mask &= !(1 << last);
        if p == u8::MAX {
            break;
        }
        last = p as usize;
    }
    route.push(0);
    route.reverse();
    Some((route, best))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Điểm trên một đường thẳng: chi phí = |x_a − x_b|. Đỉnh 0 ở x = 0.
    fn line(xs: &[i64]) -> Instance {
        let n = xs.len();
        let mut cost = vec![0; n * n];
        for a in 0..n {
            for b in 0..n {
                cost[a * n + b] = (xs[a] - xs[b]).abs();
            }
        }
        Instance::new((n - 1) / 2, cost).unwrap()
    }

    #[test]
    fn single_pair_goes_pickup_then_delivery() {
        // lấy ở x = 10, giao ở x = 3: 0 → 10 → 3 = 10 + 7
        let inst = line(&[0, 10, 3]);
        let sol = solve(&inst, None);
        assert_eq!(sol.route, vec![0, 1, 2]);
        assert_eq!(sol.cost, 17);
        assert_eq!(exact_dp(&inst).unwrap().1, 17);
    }

    #[test]
    fn precedence_forces_a_detour_the_unconstrained_tsp_would_skip() {
        // cặp 0: lấy x = 5, giao x = 1; cặp 1: lấy x = 2, giao x = 4. Không ràng buộc thì 0→1→2→4→5 (chi phí 5), có ràng buộc lấy trước giao
        // thì phải qua điểm lấy trước điểm giao của từng cặp.
        let inst = line(&[0, 5, 1, 2, 4]);
        let (route, cost) = exact_dp(&inst).unwrap();
        assert!(inst.is_feasible(&route));
        // tối ưu thủ công: 0 →(lấy 1: x=2) → (lấy 0: x=5)? 2→5 = 3, →(giao 1: x=4) = 1, →(giao 0: x=1) = 3: tổng 2+3+1+3 = 9
        // hoặc 0 → lấy 1 (2) → giao 1 (4) → lấy 0 (5) → giao 0 (1): 2 + 2 + 1 + 4 = 9
        assert_eq!(cost, 9);
        let sol = solve(&inst, None);
        assert!(inst.is_feasible(&sol.route));
        assert!(sol.cost >= cost);
    }

    #[test]
    fn two_opt_rejects_reversing_a_whole_pair() {
        // tuyến 0, p0, d0, p1, d1: đoạn [p0, d0] chứa cả cặp 0, đảo sẽ đặt giao trước lấy
        let inst = line(&[0, 1, 2, 3, 4]);
        let r = vec![0, 1, 2, 3, 4];
        if let Some((_, Move::TwoOpt { i, j })) = best_two_opt(&inst, &r) {
            let out = apply_move(&r, Move::TwoOpt { i, j });
            assert!(inst.is_feasible(&out));
        }
    }

    #[test]
    fn asymmetric_costs_are_respected() {
        // đường một chiều: 0 → p → d rẻ, ngược lại rất đắt
        let n = 3;
        let mut cost = vec![1000; n * n];
        cost[1] = 1;
        cost[n + 2] = 1;
        for i in 0..n {
            cost[i * n + i] = 0;
        }
        let inst = Instance::new(1, cost).unwrap();
        assert_eq!(solve(&inst, None).cost, 2);
    }

    #[test]
    fn deadline_zero_still_returns_a_feasible_route() {
        let xs: Vec<i64> = (0..13).map(|i| (i * 37 % 101) as i64).collect();
        let inst = line(&xs);
        let sol = solve(&inst, Some(Duration::ZERO));
        assert!(sol.timed_out);
        assert!(inst.is_feasible(&sol.route));
    }

    #[test]
    fn bad_matrix_is_rejected() {
        assert!(Instance::new(2, vec![0; 24]).is_err());
        assert!(Instance::new(1, vec![0, 0, 0, 0, 0, 0, 0, -1, 0]).is_err());
    }
}
