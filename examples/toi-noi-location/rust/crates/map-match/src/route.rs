//! Dijkstra có giới hạn, dùng lại bộ nhớ giữa các lần gọi (đánh dấu thế hệ thay vì cấp phát lại mảng 126.699 phần tử).
//!
//! `road_graph::sp::shortest_cm` cấp phát một mảng cỡ cả đồ thị mỗi lần gọi, đủ cho đo hệ số vòng ở LOC-04 nhưng quá đắt khi
//! bám đường phải chạy hàng chục lần cho mỗi điểm GPS. Ở đây mỗi lần chạy chỉ chạm vào các nút trong bán kính tìm kiếm.
//! Giống `sp`, bỏ qua cấm rẽ và chi phí rẽ (LOC-23 mới thêm); đó là giới hạn cần nói rõ.

use road_graph::{EdgeId, Graph, Mode, NONE, NodeId};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub const INF: u32 = u32::MAX;

pub struct Router {
    dist: Vec<u32>,
    stamp: Vec<u32>,
    parent: Vec<u32>,
    cur: u32,
    heap: BinaryHeap<Reverse<(u32, u32)>>,
    /// Tổng số nút đã chốt (settled) từ khi tạo, để lab báo cáo công việc thực sự.
    pub settled_total: u64,
    /// Số lần chạy bị cắt vì chạm trần `max_settled`.
    pub budget_hits: u64,
}

impl Router {
    pub fn new(g: &Graph) -> Self {
        let n = g.node_count();
        Self {
            dist: vec![INF; n],
            stamp: vec![0; n],
            parent: vec![NONE; n],
            cur: 0,
            heap: BinaryHeap::new(),
            settled_total: 0,
            budget_hits: 0,
        }
    }

    fn begin(&mut self) {
        self.cur = self.cur.wrapping_add(1);
        if self.cur == 0 {
            self.stamp.fill(0);
            self.cur = 1;
        }
        self.heap.clear();
    }
    #[inline]
    fn get(&self, n: NodeId) -> u32 {
        if self.stamp[n as usize] == self.cur {
            self.dist[n as usize]
        } else {
            INF
        }
    }
    #[inline]
    fn set(&mut self, n: NodeId, d: u32, parent_edge: u32) {
        self.stamp[n as usize] = self.cur;
        self.dist[n as usize] = d;
        self.parent[n as usize] = parent_edge;
    }

    /// Khoảng cách (cm) từ `src` tới từng nút trong `targets`, `INF` nếu không tới được trong `limit_cm` hoặc trong ngân sách
    /// `max_settled` nút. Dừng ngay khi đã chốt đủ mọi đích. `out` phải dài bằng `targets`.
    #[allow(clippy::too_many_arguments)]
    pub fn distances_to(
        &mut self,
        g: &Graph,
        mode: Mode,
        src: NodeId,
        targets: &[NodeId],
        limit_cm: u32,
        max_settled: usize,
        out: &mut [u32],
    ) {
        out.fill(INF);
        let mut remaining = targets.len();
        self.begin();
        self.set(src, 0, NONE);
        self.heap.push(Reverse((0, src)));
        let bit = mode.bit();
        let mut settled = 0usize;
        while let Some(Reverse((d, u))) = self.heap.pop() {
            if d > self.get(u) {
                continue;
            }
            if d > limit_cm {
                break;
            }
            settled += 1;
            for (i, &t) in targets.iter().enumerate() {
                if t == u && out[i] == INF {
                    out[i] = d;
                    remaining -= 1;
                }
            }
            if remaining == 0 {
                break;
            }
            if settled >= max_settled {
                self.budget_hits += 1;
                break;
            }
            for e in g.out_edges(u) {
                let e = e as usize;
                if g.modes[e] & bit == 0 {
                    continue;
                }
                let nd = d.saturating_add(g.length_cm[e]);
                let v = g.head[e];
                if nd < self.get(v) {
                    self.set(v, nd, e as u32);
                    self.heap.push(Reverse((nd, v)));
                }
            }
        }
        self.settled_total += settled as u64;
    }

    /// Đường ngắn nhất `src` → `dst`: danh sách cạnh có hướng và độ dài cm.
    pub fn shortest_path(
        &mut self,
        g: &Graph,
        mode: Mode,
        src: NodeId,
        dst: NodeId,
    ) -> Option<(Vec<EdgeId>, u32)> {
        self.begin();
        self.set(src, 0, NONE);
        self.heap.push(Reverse((0, src)));
        let bit = mode.bit();
        while let Some(Reverse((d, u))) = self.heap.pop() {
            if d > self.get(u) {
                continue;
            }
            self.settled_total += 1;
            if u == dst {
                let mut edges = Vec::new();
                let mut n = dst;
                while n != src {
                    let e = self.parent[n as usize];
                    edges.push(e);
                    n = g.tail[e as usize];
                }
                edges.reverse();
                return Some((edges, d));
            }
            for e in g.out_edges(u) {
                let e = e as usize;
                if g.modes[e] & bit == 0 {
                    continue;
                }
                let nd = d.saturating_add(g.length_cm[e]);
                let v = g.head[e];
                if nd < self.get(v) {
                    self.set(v, nd, e as u32);
                    self.heap.push(Reverse((nd, v)));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use road_graph::fixture::{compile_fixture, generate};
    use road_graph::sp;

    #[test]
    fn matches_reference_dijkstra_and_path_is_contiguous() {
        let g = compile_fixture(&generate());
        let mut r = Router::new(&g);
        for (s, t) in [(0u32, 100u32), (5, 200), (150, 3), (7, 7)] {
            let want = sp::shortest_cm(&g, Mode::Car, s, t);
            let got = r.shortest_path(&g, Mode::Car, s, t);
            assert_eq!(got.as_ref().map(|x| x.1), want);
            if let Some((edges, d)) = got {
                let sum: u32 = edges.iter().map(|&e| g.length_cm[e as usize]).sum();
                assert_eq!(sum, d);
                for w in edges.windows(2) {
                    assert_eq!(g.head[w[0] as usize], g.tail[w[1] as usize]);
                }
                if let Some(&f) = edges.first() {
                    assert_eq!(g.tail[f as usize], s);
                }
            }
            let mut out = [0u32; 1];
            r.distances_to(&g, Mode::Car, s, &[t], u32::MAX, usize::MAX, &mut out);
            assert_eq!((out[0] != INF).then_some(out[0]), want);
        }
    }

    #[test]
    fn limit_and_budget_cut_the_search() {
        let g = compile_fixture(&generate());
        let mut r = Router::new(&g);
        let mut out = [0u32; 1];
        let far = sp::distances_cm(&g, Mode::Car, 0)
            .iter()
            .enumerate()
            .filter(|&(_, &d)| d != INF && d > 50_000)
            .map(|(i, _)| i as u32)
            .next()
            .unwrap();
        r.distances_to(&g, Mode::Car, 0, &[far], 10_000, usize::MAX, &mut out);
        assert_eq!(out[0], INF, "đích xa hơn giới hạn");
        r.distances_to(&g, Mode::Car, 0, &[far], u32::MAX, 3, &mut out);
        assert_eq!(out[0], INF, "ngân sách 3 nút không đủ");
        assert!(r.budget_hits >= 1);
    }
}
