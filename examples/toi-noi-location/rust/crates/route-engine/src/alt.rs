//! LOC-22: ALT (A*, Landmarks, Triangle inequality).
//!
//! Chọn `k` landmark, tính trước khoảng cách từ và tới mỗi landmark. Với đích `t`, chặn dưới chi phí `u → t` là
//! `max_l max( d(u,l) - d(t,l),  d(l,t) - d(l,u) )` theo bất đẳng thức tam giác. Đây là heuristic nhất quán nên A* vẫn tối ưu.

use crate::net::{INF, Rev, View, sssp};
use rayon::prelude::*;
use road_graph::{Graph, NodeId};

#[derive(Debug, Clone)]
pub struct Landmarks {
    pub ids: Vec<NodeId>,
    k: usize,
    /// `data[(v * k + i) * 2]` = d(v → landmark i), `+1` = d(landmark i → v). Xen kẽ theo nút để một lần đọc cache.
    data: Vec<u32>,
}

impl Landmarks {
    /// Chọn landmark theo «xa nhất» (farthest) trong các nút của `candidates` (vd. thành phần liên thông mạnh lớn nhất),
    /// rồi tính trước bằng `2k` lần Dijkstra song song.
    pub fn build(view: View, k: usize, candidates: &[bool], seed_node: NodeId) -> Self {
        let n = view.node_count();
        let mut ids: Vec<NodeId> = Vec::with_capacity(k);
        let mut from_l: Vec<Vec<u32>> = Vec::with_capacity(k); // d(l -> v)
        let mut to_l: Vec<Vec<u32>> = Vec::with_capacity(k); // d(v -> l)
        // landmark đầu: nút xa nhất so với nút hạt giống
        let d0 = sssp(view, seed_node, true);
        let first = farthest(&d0, candidates);
        let mut min_d = vec![INF; n];
        let mut next = first;
        for _ in 0..k {
            let (f, b) = rayon::join(|| sssp(view, next, true), || sssp(view, next, false));
            for v in 0..n {
                let m = f[v].min(b[v]);
                if m < min_d[v] {
                    min_d[v] = m;
                }
            }
            ids.push(next);
            from_l.push(f);
            to_l.push(b);
            // nút tiếp theo: xa nhất so với tập landmark đã chọn
            let mut best = (0u32, NodeId::MAX);
            for v in 0..n {
                if candidates[v]
                    && min_d[v] != INF
                    && min_d[v] >= best.0
                    && !ids.contains(&(v as u32))
                {
                    best = (min_d[v], v as u32);
                }
            }
            if best.1 == NodeId::MAX {
                break;
            }
            next = best.1;
        }
        let k = ids.len();
        let mut data = vec![INF; n * k * 2];
        data.par_chunks_mut(k * 2)
            .enumerate()
            .for_each(|(v, chunk)| {
                for i in 0..k {
                    chunk[i * 2] = to_l[i][v];
                    chunk[i * 2 + 1] = from_l[i][v];
                }
            });
        Self { ids, k, data }
    }

    pub fn count(&self) -> usize {
        self.k
    }
    pub fn bytes(&self) -> usize {
        self.data.len() * size_of::<u32>()
    }

    /// Chặn dưới của chi phí `u → t`.
    #[inline]
    pub fn lower_bound(&self, u: NodeId, t: NodeId) -> u32 {
        let (bu, bt) = (u as usize * self.k * 2, t as usize * self.k * 2);
        let mut best = 0u32;
        for i in 0..self.k {
            let (u_to, u_from) = (self.data[bu + i * 2], self.data[bu + i * 2 + 1]);
            let (t_to, t_from) = (self.data[bt + i * 2], self.data[bt + i * 2 + 1]);
            // d(u,t) >= d(u,l) - d(t,l)
            if u_to != INF && t_to != INF && u_to > t_to {
                best = best.max(u_to - t_to);
            }
            // d(u,t) >= d(l,t) - d(l,u)
            if u_from != INF && t_from != INF && t_from > u_from {
                best = best.max(t_from - u_from);
            }
        }
        best
    }
}

fn farthest(d: &[u32], candidates: &[bool]) -> NodeId {
    let mut best = (0u32, 0u32);
    for (v, &x) in d.iter().enumerate() {
        if candidates[v] && x != INF && x >= best.0 {
            best = (x, v as u32);
        }
    }
    best.1
}

/// Tiện ích: mặt nạ nút thuộc thành phần liên thông mạnh lớn nhất của profile (theo cạnh đi được).
pub fn largest_scc_nodes(g: &Graph, rev: &Rev, w: &[u32]) -> Vec<bool> {
    // Kosaraju lặp, chỉ để chọn landmark và mẫu truy vấn; dùng cạnh có trọng số hữu hạn.
    let n = g.node_count();
    let view = View::new(g, rev, w);
    let mut order = Vec::with_capacity(n);
    let mut seen = vec![false; n];
    for s in 0..n as u32 {
        if seen[s as usize] {
            continue;
        }
        seen[s as usize] = true;
        let mut stack: Vec<(u32, Vec<u32>)> = vec![(s, view.out(s).map(|(_, v, _)| v).collect())];
        while let Some((u, nbrs)) = stack.last_mut() {
            if let Some(v) = nbrs.pop() {
                if !seen[v as usize] {
                    seen[v as usize] = true;
                    let list: Vec<u32> = view.out(v).map(|(_, x, _)| x).collect();
                    stack.push((v, list));
                }
            } else {
                order.push(*u);
                stack.pop();
            }
        }
    }
    let mut comp = vec![u32::MAX; n];
    let mut sizes: Vec<u32> = Vec::new();
    for &s in order.iter().rev() {
        if comp[s as usize] != u32::MAX {
            continue;
        }
        let id = sizes.len() as u32;
        let mut size = 0;
        let mut stack = vec![s];
        comp[s as usize] = id;
        while let Some(u) = stack.pop() {
            size += 1;
            for (_, v, _) in view.inn(u) {
                if comp[v as usize] == u32::MAX {
                    comp[v as usize] = id;
                    stack.push(v);
                }
            }
        }
        sizes.push(size);
    }
    let big = sizes
        .iter()
        .enumerate()
        .max_by_key(|&(i, &s)| (s, std::cmp::Reverse(i)))
        .map(|(i, _)| i as u32);
    comp.iter().map(|&c| Some(c) == big).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{Costs, Metric, Profile};
    use crate::search::Searcher;
    use road_graph::fixture::{compile_fixture, generate};

    #[test]
    fn alt_is_exact_and_settles_no_more_than_dijkstra_on_average() {
        let g = compile_fixture(&generate());
        let rev = Rev::build(&g);
        let c = Costs::build(&g, Profile::Car, Metric::Time);
        let view = View::new(&g, &rev, &c.w);
        let scc = largest_scc_nodes(&g, &rev, &c.w);
        let nodes: Vec<u32> = (0..g.node_count() as u32)
            .filter(|&v| scc[v as usize])
            .collect();
        let lm = Landmarks::build(view, 8, &scc, nodes[0]);
        assert_eq!(lm.count(), 8);
        let mut s = Searcher::new(g.node_count());
        let (mut sd, mut sa) = (0u64, 0u64);
        let mut seed = 3u64;
        for _ in 0..200 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let a = nodes[(seed >> 33) as usize % nodes.len()];
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let b = nodes[(seed >> 33) as usize % nodes.len()];
            let (d, ds) = s.dijkstra(view, a, b);
            let (x, xs) = s.run(view, a, b, |u| lm.lower_bound(u, b));
            assert_eq!(d.map(|p| p.cost), x.map(|p| p.cost));
            sd += u64::from(ds.settled);
            sa += u64::from(xs.settled);
        }
        assert!(sa < sd, "ALT {sa} phải ít nút hơn Dijkstra {sd}");
    }

    #[test]
    fn lower_bound_never_exceeds_true_distance() {
        let g = compile_fixture(&generate());
        let rev = Rev::build(&g);
        let c = Costs::build(&g, Profile::Car, Metric::Distance);
        let view = View::new(&g, &rev, &c.w);
        let scc = largest_scc_nodes(&g, &rev, &c.w);
        let first = scc.iter().position(|&b| b).unwrap() as u32;
        let lm = Landmarks::build(view, 6, &scc, first);
        for src in (0..g.node_count() as u32).step_by(17) {
            let d = sssp(view, src, true);
            for t in (0..g.node_count() as u32).step_by(13) {
                if d[t as usize] != INF {
                    assert!(lm.lower_bound(src, t) <= d[t as usize]);
                }
            }
        }
    }
}
