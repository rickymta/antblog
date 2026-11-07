//! LOC-44: cắt tỉa ứng viên bằng H3 k-ring và k gần nhất, ma trận thưa, tách vùng độc lập.
//!
//! Ý tưởng: không tính ETA cho mọi cặp (khách × tài xế). Với mỗi khách chỉ giữ tài xế nằm trong k-ring quanh ô H3 của điểm đón,
//! rồi cắt còn `k_nearest` tài xế gần nhất (chim bay). Đồ thị hai phía thưa này tách thành các thành phần liên thông độc lập;
//! mỗi thành phần giải Hungarian riêng trên ma trận nhỏ.

use crate::assign::{greedy_global, hungarian};
use crate::cost::{Dense, FORBIDDEN};
use geo_core::h3::{cell_of, k_ring, resolution};
use geo_core::{LatLng, haversine_m};
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PruneConfig {
    /// Độ phân giải H3 của chỉ mục tài xế (8: cạnh ô ~ 530 m; 7: ~ 1,4 km).
    pub res: u8,
    /// Bán kính k-ring ban đầu quanh ô của điểm đón.
    pub k_ring: u32,
    /// Giữ tối đa chừng này tài xế gần nhất cho mỗi khách.
    pub k_nearest: usize,
    /// Nếu k-ring không đủ `k_nearest` ứng viên thì nới thêm 1 vòng, tới `k_ring_max`; vẫn thiếu thì lấy những gì có.
    pub k_ring_max: u32,
    /// Lượt bù: sau khi giải theo vùng, khách và tài xế còn thừa được ghép lại trên ma trận đầy đủ của riêng họ.
    pub leftover: bool,
}

impl Default for PruneConfig {
    fn default() -> Self {
        Self {
            res: 8,
            k_ring: 2,
            k_nearest: 8,
            k_ring_max: 8,
            leftover: false,
        }
    }
}

/// Ứng viên theo khách: chỉ số tài xế, sắp theo khoảng cách chim bay tăng dần.
pub fn candidates(drivers: &[LatLng], pickups: &[LatLng], cfg: PruneConfig) -> Vec<Vec<usize>> {
    let res = resolution(cfg.res);
    let mut index: FxHashMap<u64, Vec<usize>> = FxHashMap::default();
    for (i, &d) in drivers.iter().enumerate() {
        index.entry(u64::from(cell_of(d, res))).or_default().push(i);
    }
    pickups
        .iter()
        .map(|&p| {
            let cell = cell_of(p, res);
            let mut k = cfg.k_ring;
            let mut found: Vec<usize>;
            loop {
                found = k_ring(cell, k)
                    .into_iter()
                    .flat_map(|c| index.get(&u64::from(c)).cloned().unwrap_or_default())
                    .collect();
                if found.len() >= cfg.k_nearest || k >= cfg.k_ring_max {
                    break;
                }
                k += 1;
            }
            found.sort_by(|&a, &b| {
                haversine_m(drivers[a], p)
                    .total_cmp(&haversine_m(drivers[b], p))
                    .then(a.cmp(&b))
            });
            found.truncate(cfg.k_nearest);
            found
        })
        .collect()
}

/// Ma trận thưa dạng danh sách cạnh `(khách, tài xế, chi phí)`.
#[derive(Debug, Clone, Default)]
pub struct Sparse {
    pub rows: usize,
    pub cols: usize,
    pub edges: Vec<(usize, usize, i64)>,
}

impl Sparse {
    pub fn density(&self) -> f64 {
        self.edges.len() as f64 / (self.rows.max(1) * self.cols.max(1)) as f64
    }
}

/// Tách thành các thành phần liên thông (union-find trên khách ∪ tài xế). Trả mỗi thành phần: (danh sách khách, danh sách tài xế).
pub fn components(s: &Sparse) -> Vec<(Vec<usize>, Vec<usize>)> {
    let n = s.rows + s.cols;
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(p: &mut [usize], x: usize) -> usize {
        let mut r = x;
        while p[r] != r {
            r = p[r];
        }
        let mut y = x;
        while p[y] != r {
            let nx = p[y];
            p[y] = r;
            y = nx;
        }
        r
    }
    for &(r, c, _) in &s.edges {
        let (a, b) = (find(&mut parent, r), find(&mut parent, s.rows + c));
        if a != b {
            parent[a] = b;
        }
    }
    let mut groups: FxHashMap<usize, (Vec<usize>, Vec<usize>)> = FxHashMap::default();
    let touched_r: Vec<bool> = {
        let mut t = vec![false; s.rows];
        for &(r, _, _) in &s.edges {
            t[r] = true;
        }
        t
    };
    let touched_c: Vec<bool> = {
        let mut t = vec![false; s.cols];
        for &(_, c, _) in &s.edges {
            t[c] = true;
        }
        t
    };
    for r in (0..s.rows).filter(|&r| touched_r[r]) {
        groups.entry(find(&mut parent, r)).or_default().0.push(r);
    }
    for c in (0..s.cols).filter(|&c| touched_c[c]) {
        groups
            .entry(find(&mut parent, s.rows + c))
            .or_default()
            .1
            .push(c);
    }
    let mut out: Vec<_> = groups.into_values().collect();
    out.sort_by_key(|(r, c)| (r.first().copied(), c.first().copied()));
    out
}

/// Kết quả giải ma trận thưa theo vùng.
#[derive(Debug, Clone)]
pub struct SparseSolution {
    pub pairs: Vec<(usize, usize)>,
    pub components: usize,
    /// Cạnh của thành phần lớn nhất (số khách × số tài xế), để thấy vùng không «dính» thành một khối.
    pub largest: (usize, usize),
}

/// Giải từng thành phần độc lập bằng `solver` (Hungarian hoặc tham lam) trên ma trận đặc nhỏ của riêng nó.
pub fn solve_sparse(s: &Sparse, solver: fn(&Dense) -> Vec<(usize, usize)>) -> SparseSolution {
    let comps = components(s);
    let mut pairs = Vec::new();
    let mut largest = (0, 0);
    for (rs, cs) in &comps {
        if rs.len() * cs.len() > largest.0 * largest.1 {
            largest = (rs.len(), cs.len());
        }
        let ridx: FxHashMap<usize, usize> = rs.iter().enumerate().map(|(i, &r)| (r, i)).collect();
        let cidx: FxHashMap<usize, usize> = cs.iter().enumerate().map(|(i, &c)| (c, i)).collect();
        let mut d = Dense::new(rs.len(), cs.len(), FORBIDDEN);
        for &(r, c, v) in &s.edges {
            if let (Some(&i), Some(&j)) = (ridx.get(&r), cidx.get(&c)) {
                d.set(i, j, v);
            }
        }
        for (i, j) in solver(&d) {
            pairs.push((rs[i], cs[j]));
        }
    }
    pairs.sort_unstable();
    SparseSolution {
        pairs,
        components: comps.len(),
        largest,
    }
}

pub fn solve_sparse_hungarian(s: &Sparse) -> SparseSolution {
    solve_sparse(s, hungarian)
}
pub fn solve_sparse_greedy(s: &Sparse) -> SparseSolution {
    solve_sparse(s, greedy_global)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn components_split_independent_regions() {
        // khách 0,1 với xe 0,1; khách 2 với xe 2; khách 3 không có ứng viên
        let s = Sparse {
            rows: 4,
            cols: 3,
            edges: vec![(0, 0, 5), (0, 1, 7), (1, 1, 4), (2, 2, 9)],
        };
        let c = components(&s);
        assert_eq!(c.len(), 2);
        assert_eq!(c[0], (vec![0, 1], vec![0, 1]));
        assert_eq!(c[1], (vec![2], vec![2]));
        let sol = solve_sparse_hungarian(&s);
        assert_eq!(sol.pairs, vec![(0, 0), (1, 1), (2, 2)]);
    }
}

/// Kết quả của đường ống cắt tỉa đầy đủ trên một lô, kèm thời gian từng bước (µs; đo khi máy chịu tải).
#[derive(Debug, Clone)]
pub struct PrunedRun {
    pub pairs: Vec<(usize, usize)>,
    pub cost_ms: i64,
    pub edges: usize,
    pub components: usize,
    pub largest: (usize, usize),
    pub candidates_us: f64,
    pub eta_us: f64,
    pub solve_us: f64,
}

/// Cắt tỉa → ETA chỉ cho các cặp ứng viên → ma trận thưa → giải Hungarian từng vùng độc lập.
/// `drivers`/`pickups` là vị trí có hướng để tính ETA; `*_ll` là toạ độ để lập chỉ mục H3.
pub fn pruned_hungarian(
    engine: &route_engine::engine::Engine,
    drivers: &[route_engine::position::Position],
    drivers_ll: &[LatLng],
    pickups: &[&[route_engine::position::Position]],
    pickups_ll: &[LatLng],
    cfg: PruneConfig,
) -> PrunedRun {
    let t0 = std::time::Instant::now();
    let cand = candidates(drivers_ll, pickups_ll, cfg);
    let candidates_us = t0.elapsed().as_secs_f64() * 1e6;
    let t1 = std::time::Instant::now();
    let mut q = crate::cost::PairQuery::new(engine);
    let mut sp = Sparse {
        rows: pickups.len(),
        cols: drivers.len(),
        edges: Vec::new(),
    };
    for (j, ci) in cand.iter().enumerate() {
        for &i in ci {
            let e = q.eta_ms(drivers[i], pickups[j]);
            if e < FORBIDDEN / 2 {
                sp.edges.push((j, i, e));
            }
        }
    }
    let eta_us = t1.elapsed().as_secs_f64() * 1e6;
    let t2 = std::time::Instant::now();
    let sol = solve_sparse_hungarian(&sp);
    let solve_us = t2.elapsed().as_secs_f64() * 1e6;
    let mut pairs = sol.pairs;
    let mut by_pair: FxHashMap<(usize, usize), i64> =
        sp.edges.iter().map(|&(r, c, v)| ((r, c), v)).collect();
    let mut leftover_us = 0.0;
    if cfg.leftover {
        let t3 = std::time::Instant::now();
        let mut ur: Vec<usize> = (0..pickups.len()).collect();
        let mut uc: Vec<usize> = (0..drivers.len()).collect();
        let (mr, mc): (FxHashSet<usize>, FxHashSet<usize>) = pairs.iter().copied().unzip();
        ur.retain(|r| !mr.contains(r));
        uc.retain(|c| !mc.contains(c));
        if !ur.is_empty() && !uc.is_empty() {
            let sub = Dense::from_fn(ur.len(), uc.len(), |a, b| {
                let (j, i) = (ur[a], uc[b]);
                by_pair
                    .get(&(j, i))
                    .copied()
                    .unwrap_or_else(|| q.eta_ms(drivers[i], pickups[j]))
            });
            for (a, b) in hungarian(&sub) {
                let (j, i) = (ur[a], uc[b]);
                by_pair.insert((j, i), sub.get(a, b));
                pairs.push((j, i));
            }
        }
        leftover_us = t3.elapsed().as_secs_f64() * 1e6;
    }
    pairs.sort_unstable();
    let cost_ms = pairs.iter().map(|p| by_pair[p]).sum();
    PrunedRun {
        pairs,
        cost_ms,
        edges: sp.edges.len(),
        components: sol.components,
        largest: sol.largest,
        candidates_us,
        eta_us,
        solve_us: solve_us + leftover_us,
    }
}
