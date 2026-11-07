//! Cái nhìn tính toán lên đồ thị: CSR chiều thuận (từ `road-graph`), CSR chiều ngược, mảng trọng số của một profile.
//!
//! Mọi thuật toán nhận [`View`] (chỉ là ba tham chiếu, `Copy`) nên ba profile dùng chung một bản đồ trong bộ nhớ.

use road_graph::{EdgeId, Graph, NodeId};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Trọng số «không đi được» và khoảng cách «chưa tới».
pub const INF: u32 = u32::MAX;
pub const NONE: u32 = u32::MAX;

/// CSR chiều ngược: các cạnh đi vào mỗi nút. Không phụ thuộc profile, dựng một lần.
#[derive(Debug, Clone)]
pub struct Rev {
    pub first_in: Vec<u32>,
    pub in_edge: Vec<EdgeId>,
}

impl Rev {
    pub fn build(g: &Graph) -> Self {
        let n = g.node_count();
        let mut first_in = vec![0u32; n + 1];
        for &h in &g.head {
            first_in[h as usize + 1] += 1;
        }
        for i in 0..n {
            first_in[i + 1] += first_in[i];
        }
        let mut fill = first_in.clone();
        let mut in_edge = vec![0u32; g.edge_count()];
        for e in 0..g.edge_count() {
            let h = g.head[e] as usize;
            in_edge[fill[h] as usize] = e as u32;
            fill[h] += 1;
        }
        Self { first_in, in_edge }
    }
    pub fn bytes(&self) -> usize {
        (self.first_in.len() + self.in_edge.len()) * size_of::<u32>()
    }
}

/// Đồ thị + chiều ngược + trọng số của một profile. Rẻ để sao chép.
#[derive(Clone, Copy)]
pub struct View<'a> {
    pub g: &'a Graph,
    pub rev: &'a Rev,
    pub w: &'a [u32],
}

impl<'a> View<'a> {
    pub fn new(g: &'a Graph, rev: &'a Rev, w: &'a [u32]) -> Self {
        Self { g, rev, w }
    }
    pub fn node_count(&self) -> usize {
        self.g.node_count()
    }
    /// Cạnh đi ra khỏi `u` mà profile đi được: `(cạnh, nút đích, trọng số)`.
    #[inline]
    pub fn out(&self, u: NodeId) -> impl Iterator<Item = (EdgeId, NodeId, u32)> + 'a {
        let (g, w) = (self.g, self.w);
        g.out_edges(u).filter_map(move |e| {
            let c = w[e as usize];
            (c != INF).then(|| (e, g.head[e as usize], c))
        })
    }
    /// Cạnh đi vào `v` mà profile đi được: `(cạnh, nút nguồn, trọng số)`.
    #[inline]
    pub fn inn(&self, v: NodeId) -> impl Iterator<Item = (EdgeId, NodeId, u32)> + 'a {
        let (g, w, rev) = (self.g, self.w, self.rev);
        let r = rev.first_in[v as usize] as usize..rev.first_in[v as usize + 1] as usize;
        rev.in_edge[r].iter().filter_map(move |&e| {
            let c = w[e as usize];
            (c != INF).then(|| (e, g.tail[e as usize], c))
        })
    }
}

/// Nhãn khoảng cách có đánh dấu phiên bản: đặt lại cho truy vấn kế tiếp là O(1), không `memset` cả đồ thị.
#[derive(Debug, Clone)]
pub struct Labels {
    dist: Vec<u32>,
    pred: Vec<u32>,
    seen: Vec<u32>,
    closed: Vec<u32>,
    cur: u32,
    /// Các nút đã chạm vào trong truy vấn hiện tại (để duyệt kết quả mà không quét n phần tử).
    pub touched: Vec<NodeId>,
}

impl Labels {
    pub fn new(n: usize) -> Self {
        Self {
            dist: vec![INF; n],
            pred: vec![NONE; n],
            seen: vec![0; n],
            closed: vec![0; n],
            cur: 1,
            touched: Vec::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.dist.len()
    }
    pub fn is_empty(&self) -> bool {
        self.dist.is_empty()
    }
    pub fn reset(&mut self) {
        self.cur = self.cur.wrapping_add(1);
        if self.cur == 0 {
            self.seen.fill(0);
            self.closed.fill(0);
            self.cur = 1;
        }
        self.touched.clear();
    }
    #[inline]
    pub fn dist(&self, v: NodeId) -> u32 {
        if self.seen[v as usize] == self.cur {
            self.dist[v as usize]
        } else {
            INF
        }
    }
    #[inline]
    pub fn pred(&self, v: NodeId) -> u32 {
        if self.seen[v as usize] == self.cur {
            self.pred[v as usize]
        } else {
            NONE
        }
    }
    #[inline]
    pub fn set(&mut self, v: NodeId, d: u32, pred: u32) {
        let i = v as usize;
        if self.seen[i] != self.cur {
            self.seen[i] = self.cur;
            self.touched.push(v);
        }
        self.dist[i] = d;
        self.pred[i] = pred;
    }
    #[inline]
    pub fn is_closed(&self, v: NodeId) -> bool {
        self.closed[v as usize] == self.cur
    }
    #[inline]
    pub fn close(&mut self, v: NodeId) {
        self.closed[v as usize] = self.cur;
    }
}

/// Hàng đợi ưu tiên nhỏ nhất ở đầu: khoá và nút đóng gói trong một `u64`.
#[derive(Debug, Default, Clone)]
pub struct MinHeap(BinaryHeap<Reverse<u64>>);

impl MinHeap {
    pub fn new() -> Self {
        Self::default()
    }
    #[inline]
    pub fn push(&mut self, key: u32, node: u32) {
        self.0
            .push(Reverse((u64::from(key) << 32) | u64::from(node)));
    }
    #[inline]
    pub fn pop(&mut self) -> Option<(u32, u32)> {
        self.0.pop().map(|Reverse(x)| ((x >> 32) as u32, x as u32))
    }
    #[inline]
    pub fn peek(&self) -> Option<(u32, u32)> {
        self.0
            .peek()
            .map(|Reverse(x)| ((x >> 32) as u32, *x as u32))
    }
    #[inline]
    pub fn peek_key(&self) -> Option<u32> {
        self.0.peek().map(|Reverse(x)| (x >> 32) as u32)
    }
    pub fn clear(&mut self) {
        self.0.clear();
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Đường đi: tổng chi phí và danh sách cạnh theo thứ tự.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    pub cost: u32,
    pub edges: Vec<EdgeId>,
}

impl Path {
    /// Dãy nút từ nút đầu của cạnh đầu tới nút cuối của cạnh cuối; rỗng nếu đường rỗng.
    pub fn nodes(&self, g: &Graph) -> Vec<NodeId> {
        let mut v = Vec::with_capacity(self.edges.len() + 1);
        if let Some(&e0) = self.edges.first() {
            v.push(g.tail[e0 as usize]);
        }
        v.extend(self.edges.iter().map(|&e| g.head[e as usize]));
        v
    }
    pub fn length_cm(&self, g: &Graph) -> u64 {
        self.edges
            .iter()
            .map(|&e| u64::from(g.length_cm[e as usize]))
            .sum()
    }
    /// Chi phí tính lại bằng một mảng trọng số khác (vd. đánh giá tuyến tĩnh trong giờ cao điểm).
    pub fn cost_under(&self, w: &[u32]) -> u64 {
        self.edges.iter().map(|&e| u64::from(w[e as usize])).sum()
    }
}

/// Số liệu một lần tìm kiếm.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    /// Số nút được lấy ra khỏi hàng đợi và chốt (đã «duyệt»).
    pub settled: u32,
    /// Số cạnh đã xét (relax).
    pub relaxed: u32,
}

/// Dựng lại đường từ nhãn `pred` (chỉ số cạnh) ngược về nguồn.
pub fn trace_back(g: &Graph, labels: &Labels, mut v: NodeId) -> Vec<EdgeId> {
    let mut edges = Vec::new();
    loop {
        let e = labels.pred(v);
        if e == NONE {
            break;
        }
        edges.push(e);
        v = g.tail[e as usize];
    }
    edges.reverse();
    edges
}

/// Dijkstra một nguồn tới mọi nút. `forward = false` đi ngược (khoảng cách từ mọi nút tới `src`).
pub fn sssp(view: View, src: NodeId, forward: bool) -> Vec<u32> {
    let mut dist = vec![INF; view.node_count()];
    let mut heap = MinHeap::new();
    dist[src as usize] = 0;
    heap.push(0, src);
    while let Some((d, u)) = heap.pop() {
        if d > dist[u as usize] {
            continue;
        }
        let mut relax = |v: NodeId, c: u32| {
            let nd = d.saturating_add(c);
            if nd < dist[v as usize] {
                dist[v as usize] = nd;
                heap.push(nd, v);
            }
        };
        if forward {
            for (_, v, c) in view.out(u) {
                relax(v, c);
            }
        } else {
            for (_, v, c) in view.inn(u) {
                relax(v, c);
            }
        }
    }
    dist
}

#[cfg(test)]
mod tests {
    use super::*;
    use road_graph::fixture::{compile_fixture, generate};

    #[test]
    fn rev_lists_exactly_incoming_edges() {
        let g = compile_fixture(&generate());
        let rev = Rev::build(&g);
        let mut total = 0;
        for v in 0..g.node_count() as u32 {
            let r = rev.first_in[v as usize] as usize..rev.first_in[v as usize + 1] as usize;
            for &e in &rev.in_edge[r] {
                assert_eq!(g.head[e as usize], v);
                total += 1;
            }
        }
        assert_eq!(total, g.edge_count());
    }

    #[test]
    fn labels_reset_is_constant_time_and_forgets() {
        let mut l = Labels::new(10);
        l.set(3, 7, 1);
        assert_eq!(l.dist(3), 7);
        l.reset();
        assert_eq!(l.dist(3), INF);
        assert!(l.touched.is_empty());
    }

    #[test]
    fn heap_orders_by_key_then_node() {
        let mut h = MinHeap::new();
        h.push(5, 1);
        h.push(2, 9);
        h.push(2, 3);
        assert_eq!(h.pop(), Some((2, 3)));
        assert_eq!(h.pop(), Some((2, 9)));
        assert_eq!(h.pop(), Some((5, 1)));
    }
}
