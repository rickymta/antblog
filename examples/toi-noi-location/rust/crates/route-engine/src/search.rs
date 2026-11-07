//! LOC-21: Dijkstra và A* trên CSR, kèm số nút đã duyệt.
//!
//! Một hàm lõi [`Searcher::run`] chạy cả hai: Dijkstra là A* với heuristic bằng 0.

use crate::net::{INF, Labels, MinHeap, Path, Stats, View, trace_back};
use geo_core::LocalProjection;
use road_graph::{Graph, NodeId};

/// Toạ độ mét phẳng của mọi nút, để tính khoảng cách chim bay rẻ (heuristic).
#[derive(Debug, Clone)]
pub struct NodeXy {
    pub xy: Vec<[f64; 2]>,
    pub proj: LocalProjection,
}

/// Hệ số an toàn: đoạn thẳng trên mặt phẳng địa phương có thể dài hơn đường thật cỡ 0,1%, nên nhân 0,99 để chắc chắn chặn dưới.
pub const HEURISTIC_SAFETY: f64 = 0.99;

impl NodeXy {
    pub fn build(g: &Graph) -> Self {
        let proj = LocalProjection::new(g.center());
        Self {
            xy: (0..g.node_count() as u32)
                .map(|n| proj.to_xy(g.node_latlng(n)))
                .collect(),
            proj,
        }
    }
    #[inline]
    pub fn dist_m(&self, a: NodeId, b: NodeId) -> f64 {
        let (p, q) = (self.xy[a as usize], self.xy[b as usize]);
        (p[0] - q[0]).hypot(p[1] - q[1])
    }
    #[inline]
    pub fn dist_to_point_m(&self, a: NodeId, q: [f64; 2]) -> f64 {
        let p = self.xy[a as usize];
        (p[0] - q[0]).hypot(p[1] - q[1])
    }
    /// Hệ số đổi mét chim bay thành đơn vị trọng số: `100 * an_toàn / vmax` (vmax tính theo cm trên đơn vị).
    pub fn scale(vmax_cm_per_unit: f64) -> f64 {
        100.0 * HEURISTIC_SAFETY / vmax_cm_per_unit
    }
    pub fn bytes(&self) -> usize {
        self.xy.len() * size_of::<[f64; 2]>()
    }
}

/// Bộ nhớ làm việc tái sử dụng giữa các truy vấn.
pub struct Searcher {
    pub labels: Labels,
    heap: MinHeap,
}

impl Searcher {
    pub fn new(n: usize) -> Self {
        Self {
            labels: Labels::new(n),
            heap: MinHeap::new(),
        }
    }

    /// A* từ `s` tới `t`. `h(u)` phải chặn dưới chi phí thật từ `u` tới `t` và nhất quán
    /// (`h(u) <= w(u,v) + h(v)`); khi đó mỗi nút chỉ cần chốt một lần và kết quả tối ưu.
    pub fn run(
        &mut self,
        view: View,
        s: NodeId,
        t: NodeId,
        h: impl Fn(NodeId) -> u32,
    ) -> (Option<Path>, Stats) {
        self.labels.reset();
        self.heap.clear();
        let mut st = Stats::default();
        self.labels.set(s, 0, crate::net::NONE);
        self.heap.push(h(s), s);
        while let Some((_, u)) = self.heap.pop() {
            if self.labels.is_closed(u) {
                continue;
            }
            self.labels.close(u);
            st.settled += 1;
            let d = self.labels.dist(u);
            if u == t {
                let edges = trace_back(view.g, &self.labels, t);
                return (Some(Path { cost: d, edges }), st);
            }
            for (e, v, c) in view.out(u) {
                st.relaxed += 1;
                let nd = d.saturating_add(c);
                if nd < self.labels.dist(v) {
                    self.labels.set(v, nd, e);
                    self.heap.push(nd.saturating_add(h(v)), v);
                }
            }
        }
        (None, st)
    }

    pub fn dijkstra(&mut self, view: View, s: NodeId, t: NodeId) -> (Option<Path>, Stats) {
        self.run(view, s, t, |_| 0)
    }

    /// A* với heuristic chim bay `xy` và hằng số `vmax` của profile (xem [`NodeXy::scale`]).
    pub fn astar(
        &mut self,
        view: View,
        xy: &NodeXy,
        vmax_cm_per_unit: f64,
        s: NodeId,
        t: NodeId,
    ) -> (Option<Path>, Stats) {
        let scale = NodeXy::scale(vmax_cm_per_unit);
        let tp = xy.xy[t as usize];
        self.run(view, s, t, move |u| {
            (xy.dist_to_point_m(u, tp) * scale) as u32
        })
    }

    /// Mọi nút trong ngưỡng chi phí `budget` kể từ `s` (đi xuôi hoặc ngược), theo thứ tự chốt. Dùng cho tìm kiếm có giới hạn
    /// (khách đi bộ tới đâu trong 5 phút, LOC-33).
    pub fn within(
        &mut self,
        view: View,
        s: NodeId,
        budget: u32,
        forward: bool,
    ) -> Vec<(NodeId, u32)> {
        self.labels.reset();
        self.heap.clear();
        let mut out = Vec::new();
        self.labels.set(s, 0, crate::net::NONE);
        self.heap.push(0, s);
        while let Some((_, u)) = self.heap.pop() {
            if self.labels.is_closed(u) {
                continue;
            }
            self.labels.close(u);
            let d = self.labels.dist(u);
            out.push((u, d));
            let mut relax = |e: u32, v: NodeId, c: u32| {
                let nd = d.saturating_add(c);
                if nd <= budget && nd < self.labels.dist(v) {
                    self.labels.set(v, nd, e);
                    self.heap.push(nd, v);
                }
            };
            if forward {
                for (e, v, c) in view.out(u) {
                    relax(e, v, c);
                }
            } else {
                for (e, v, c) in view.inn(u) {
                    relax(e, v, c);
                }
            }
        }
        out
    }

    /// Đường tới `v` trong lần chạy `within` gần nhất (đi xuôi).
    pub fn path_to(&self, g: &Graph, v: NodeId) -> Option<Vec<u32>> {
        (self.labels.dist(v) != INF).then(|| trace_back(g, &self.labels, v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::{Rev, sssp};
    use crate::profile::{Costs, Metric, Profile};
    use road_graph::fixture::{compile_fixture, generate};

    fn setup(metric: Metric) -> (Graph, Rev, Costs, NodeXy) {
        let g = compile_fixture(&generate());
        let rev = Rev::build(&g);
        let c = Costs::build(&g, Profile::Car, metric);
        let xy = NodeXy::build(&g);
        (g, rev, c, xy)
    }

    #[test]
    fn heuristic_never_exceeds_edge_cost_for_both_metrics() {
        for metric in [Metric::Distance, Metric::Time] {
            let (g, _, c, xy) = setup(metric);
            let scale = NodeXy::scale(c.vmax_cm_per_unit);
            for e in 0..g.edge_count() {
                if c.w[e] == INF {
                    continue;
                }
                let h = xy.dist_m(g.tail[e], g.head[e]) * scale;
                assert!(
                    h <= f64::from(c.w[e]) + 1e-9,
                    "cạnh {e}: h={h} w={}",
                    c.w[e]
                );
            }
        }
    }

    #[test]
    fn astar_equals_dijkstra_equals_sssp() {
        for metric in [Metric::Distance, Metric::Time] {
            let (g, rev, c, xy) = setup(metric);
            let view = View::new(&g, &rev, &c.w);
            let mut s = Searcher::new(g.node_count());
            let n = g.node_count() as u32;
            let mut seed = 11u64;
            for _ in 0..200 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let a = (seed >> 33) as u32 % n;
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let b = (seed >> 33) as u32 % n;
                let truth = sssp(view, a, true)[b as usize];
                let (d, ds) = s.dijkstra(view, a, b);
                let (x, xs) = s.astar(view, &xy, c.vmax_cm_per_unit, a, b);
                assert_eq!(d.as_ref().map(|p| p.cost), (truth != INF).then_some(truth));
                assert_eq!(x.as_ref().map(|p| p.cost), d.as_ref().map(|p| p.cost));
                if let Some(p) = &x {
                    assert_eq!(p.cost as u64, p.cost_under(&c.w));
                }
                assert!(ds.settled > 0 && xs.settled > 0);
            }
        }
    }

    #[test]
    fn within_matches_sssp_under_budget() {
        let (g, rev, c, _) = setup(Metric::Time);
        let view = View::new(&g, &rev, &c.w);
        let mut s = Searcher::new(g.node_count());
        let full = sssp(view, 0, true);
        let budget = 120_000;
        let got = s.within(view, 0, budget, true);
        let expect = full.iter().filter(|&&d| d <= budget).count();
        assert_eq!(got.len(), expect);
        assert!(got.iter().all(|&(v, d)| d == full[v as usize]));
    }
}
