//! LOC-22: Dijkstra hai chiều với điều kiện dừng đúng, và biến thể dừng sai (lỗi kinh điển) để chứng minh.
//!
//! Điều kiện dừng đúng: giữ `mu` là chi phí tốt nhất của mọi đường đã thấy (cập nhật mỗi khi nới một cạnh chạm nút đã có
//! nhãn của chiều kia). Dừng khi `top_thuận + top_ngược >= mu`: mọi đường chưa thấy đều dài hơn tổng hai khoá nhỏ nhất.
//! Điều kiện sai: dừng ngay khi có một nút được chốt ở cả hai chiều rồi trả `d_thuận + d_ngược` của nút đó.

use crate::net::{INF, Labels, MinHeap, NONE, Path, Stats, View};
use road_graph::{EdgeId, Graph, Mode, NodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stop {
    /// `top_f + top_b >= mu`.
    Correct,
    /// Lỗi kinh điển: dừng khi nút đầu tiên được cả hai chiều chốt.
    FirstMeeting,
}

pub struct BiSearcher {
    fwd: Labels,
    bwd: Labels,
    hf: MinHeap,
    hb: MinHeap,
}

impl BiSearcher {
    pub fn new(n: usize) -> Self {
        Self {
            fwd: Labels::new(n),
            bwd: Labels::new(n),
            hf: MinHeap::new(),
            hb: MinHeap::new(),
        }
    }

    /// Bỏ các mục cũ ở đỉnh hàng đợi (nút đã chốt, hoặc khoá lớn hơn nhãn hiện tại), trả khoá nhỏ nhất còn hợp lệ.
    fn clean_top(h: &mut MinHeap, l: &Labels) -> Option<u32> {
        while let Some((k, u)) = h.peek() {
            if l.is_closed(u) || k != l.dist(u) {
                h.pop();
            } else {
                return Some(k);
            }
        }
        None
    }

    pub fn run(&mut self, view: View, s: NodeId, t: NodeId, stop: Stop) -> (Option<Path>, Stats) {
        let mut st = Stats::default();
        if s == t {
            return (
                Some(Path {
                    cost: 0,
                    edges: Vec::new(),
                }),
                st,
            );
        }
        self.fwd.reset();
        self.bwd.reset();
        self.hf.clear();
        self.hb.clear();
        self.fwd.set(s, 0, NONE);
        self.bwd.set(t, 0, NONE);
        self.hf.push(0, s);
        self.hb.push(0, t);
        let mut mu = INF;
        let mut meet: NodeId = NONE;
        // nút đầu tiên chốt ở cả hai chiều (dùng cho biến thể sai)
        let mut first_meet: NodeId = NONE;

        loop {
            let tf = Self::clean_top(&mut self.hf, &self.fwd);
            let tb = Self::clean_top(&mut self.hb, &self.bwd);
            let (Some(kf), Some(kb)) = (tf, tb) else {
                // một hàng đợi cạn: không còn gì để mở rộng ở chiều đó
                break;
            };
            match stop {
                Stop::Correct => {
                    if mu != INF && u64::from(kf) + u64::from(kb) >= u64::from(mu) {
                        break;
                    }
                }
                Stop::FirstMeeting => {
                    if first_meet != NONE {
                        break;
                    }
                }
            }
            // mở rộng chiều có khoá nhỏ hơn
            if kf <= kb {
                let (_, u) = self.hf.pop().expect("đã kiểm tra");
                self.fwd.close(u);
                st.settled += 1;
                let d = self.fwd.dist(u);
                if self.bwd.is_closed(u) && first_meet == NONE {
                    first_meet = u;
                }
                for (e, v, c) in view.out(u) {
                    st.relaxed += 1;
                    let nd = d.saturating_add(c);
                    if nd < self.fwd.dist(v) {
                        self.fwd.set(v, nd, e);
                        self.hf.push(nd, v);
                    }
                    let db = self.bwd.dist(v);
                    if db != INF && nd.saturating_add(db) < mu {
                        mu = nd + db;
                        meet = v;
                    }
                }
            } else {
                let (_, u) = self.hb.pop().expect("đã kiểm tra");
                self.bwd.close(u);
                st.settled += 1;
                let d = self.bwd.dist(u);
                if self.fwd.is_closed(u) && first_meet == NONE {
                    first_meet = u;
                }
                for (e, v, c) in view.inn(u) {
                    st.relaxed += 1;
                    let nd = d.saturating_add(c);
                    if nd < self.bwd.dist(v) {
                        self.bwd.set(v, nd, e);
                        self.hb.push(nd, v);
                    }
                    let df = self.fwd.dist(v);
                    if df != INF && nd.saturating_add(df) < mu {
                        mu = nd + df;
                        meet = v;
                    }
                }
            }
        }

        let (cost, m) = match stop {
            Stop::Correct => (mu, meet),
            Stop::FirstMeeting => {
                if first_meet == NONE {
                    (mu, meet)
                } else {
                    // đáp án của biến thể sai: tổng hai nhãn ở đúng nút gặp đầu tiên
                    (
                        self.fwd
                            .dist(first_meet)
                            .saturating_add(self.bwd.dist(first_meet)),
                        first_meet,
                    )
                }
            }
        };
        if m == NONE || cost == INF {
            return (None, st);
        }
        (
            Some(Path {
                cost,
                edges: self.assemble(view, m),
            }),
            st,
        )
    }

    fn assemble(&self, view: View, m: NodeId) -> Vec<EdgeId> {
        let mut edges = crate::net::trace_back(view.g, &self.fwd, m);
        let mut v = m;
        loop {
            let e = self.bwd.pred(v);
            if e == NONE {
                break;
            }
            edges.push(e);
            v = view.g.head[e as usize];
        }
        edges
    }
}

/// Đồ thị tay 5 nút: s=0, a=1, m=2, b=3, t=4. Đường tối ưu s→a→b→t = 1+10+1 = 12 (cạnh giữa dài).
/// Đường qua m: s→m→t = 6+7 = 13. Nút m được cả hai chiều chốt trước khi hai chiều gặp nhau trên đường tối ưu.
pub fn classic_counterexample() -> (Graph, Vec<u32>) {
    let mut g = Graph::default();
    let n = 5;
    let es: [(u32, u32, u32); 5] = [(0, 1, 1), (1, 3, 10), (3, 4, 1), (0, 2, 6), (2, 4, 7)];
    let mut by_tail: Vec<Vec<(u32, u32, u32)>> = vec![Vec::new(); n];
    for &(a, b, w) in &es {
        by_tail[a as usize].push((a, b, w));
    }
    g.first_out.push(0);
    let mut w = Vec::new();
    for list in by_tail {
        for (a, b, c) in list {
            g.tail.push(a);
            g.head.push(b);
            g.length_cm.push(c);
            g.modes.push(Mode::CAR_BIT);
            w.push(c);
        }
        g.first_out.push(g.head.len() as u32);
    }
    // các cột còn lại chỉ để Graph hợp lệ về độ dài
    let m = g.head.len();
    g.node_ll = vec![geo_core::LatLngE7::new(0, 0); n];
    g.node_osm = vec![0; n];
    g.class = vec![0; m];
    g.flags = vec![0; m];
    g.maxspeed_kmh = vec![0; m];
    g.lanes = vec![0; m];
    g.width_dm = vec![0; m];
    g.layer = vec![0; m];
    g.twin = vec![u32::MAX; m];
    g.way_id = vec![0; m];
    g.name = vec![u32::MAX; m];
    g.shape_start = vec![0; m];
    g.shape_len = vec![0; m];
    (g, w)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::Rev;
    use crate::search::Searcher;
    use road_graph::fixture::{compile_fixture, generate};

    #[test]
    fn wrong_stop_returns_longer_path_on_the_hand_made_graph() {
        let (g, w) = classic_counterexample();
        g.validate().unwrap();
        let rev = Rev::build(&g);
        let view = View::new(&g, &rev, &w);
        let mut bi = BiSearcher::new(g.node_count());
        let (good, _) = bi.run(view, 0, 4, Stop::Correct);
        let (bad, _) = bi.run(view, 0, 4, Stop::FirstMeeting);
        let (truth, _) = Searcher::new(g.node_count()).dijkstra(view, 0, 4);
        assert_eq!(truth.unwrap().cost, 12);
        assert_eq!(good.unwrap().cost, 12);
        assert_eq!(bad.unwrap().cost, 13, "dừng sai trả 13 thay vì 12");
    }

    #[test]
    fn correct_stop_matches_dijkstra_on_fixture() {
        let g = compile_fixture(&generate());
        let rev = Rev::build(&g);
        let c = crate::profile::Costs::build(
            &g,
            crate::profile::Profile::Car,
            crate::profile::Metric::Time,
        );
        let view = View::new(&g, &rev, &c.w);
        let mut bi = BiSearcher::new(g.node_count());
        let mut uni = Searcher::new(g.node_count());
        let n = g.node_count() as u32;
        let mut seed = 5u64;
        for _ in 0..300 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let a = (seed >> 33) as u32 % n;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let b = (seed >> 33) as u32 % n;
            let (x, _) = bi.run(view, a, b, Stop::Correct);
            let (y, _) = uni.dijkstra(view, a, b);
            assert_eq!(
                x.as_ref().map(|p| p.cost),
                y.as_ref().map(|p| p.cost),
                "{a}->{b}"
            );
            if let Some(p) = x {
                assert_eq!(u64::from(p.cost), p.cost_under(&c.w));
                let nodes = p.nodes(&g);
                if a != b {
                    assert_eq!((nodes[0], *nodes.last().unwrap()), (a, b));
                }
            }
        }
    }
}
