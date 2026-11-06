//! LOC-25: Contraction Hierarchies tự cài đặt.
//!
//! Tiền xử lý: xếp thứ tự nút theo độ ưu tiên (chênh lệch cạnh + số láng giềng đã co + độ sâu), co lần lượt; mỗi lần co nút `v`,
//! với mỗi cặp `u → v → x` thêm shortcut `u → x` trừ khi **tìm kiếm nhân chứng** (Dijkstra bỏ qua `v`) tìm được đường không
//! dài hơn. Kết quả: hai đồ thị «chỉ đi lên theo thứ hạng» (thuận và ngược). Truy vấn: Dijkstra hai chiều trên hai đồ thị đó,
//! gặp nhau ở nút hạng cao nhất của đường; sau đó giải nén shortcut đệ quy về các cạnh gốc.
//!
//! Mọi chỉ số nút bên trong `Ch` là **hạng** (rank); `new_id` đổi nút gốc sang hạng.

use crate::net::{INF, Labels, MinHeap, NONE, Path, Stats, View};
use crate::position::Leg;
use road_graph::{EdgeId, NodeId};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Đồ thị «đi lên» dạng CSR, chỉ số theo nút **thấp hơn** của cung.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpGraph {
    pub first: Vec<u32>,
    /// Đầu kia của cung (hạng cao hơn).
    pub head: Vec<u32>,
    pub w: Vec<u32>,
    /// Nút bị co tạo ra shortcut (hạng), hoặc [`NONE`] nếu là cạnh gốc.
    pub mid: Vec<u32>,
    /// Cạnh gốc của đồ thị đường nếu `mid == NONE`.
    pub edge: Vec<u32>,
}

impl UpGraph {
    pub fn arcs(&self, x: u32) -> std::ops::Range<usize> {
        self.first[x as usize] as usize..self.first[x as usize + 1] as usize
    }
    pub fn len(&self) -> usize {
        self.head.len()
    }
    pub fn is_empty(&self) -> bool {
        self.head.is_empty()
    }
    pub fn bytes(&self) -> usize {
        (self.first.len() + self.head.len() * 4) * size_of::<u32>()
    }
    fn find(&self, x: u32, head: u32) -> Option<usize> {
        self.arcs(x).find(|&a| self.head[a] == head)
    }
    pub(crate) fn from_lists(lists: &[Vec<(u32, u32, u32, u32)>]) -> Self {
        let mut g = UpGraph {
            first: Vec::with_capacity(lists.len() + 1),
            ..Default::default()
        };
        g.first.push(0);
        for l in lists {
            for &(h, w, mid, edge) in l {
                g.head.push(h);
                g.w.push(w);
                g.mid.push(mid);
                g.edge.push(edge);
            }
            g.first.push(g.head.len() as u32);
        }
        g
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChStats {
    pub nodes: usize,
    pub original_arcs: usize,
    pub shortcuts: usize,
    pub preprocess_s: f64,
    pub max_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ch {
    /// nút gốc → hạng
    pub new_id: Vec<u32>,
    /// hạng → nút gốc
    pub old_id: Vec<u32>,
    pub fwd: UpGraph,
    pub bwd: UpGraph,
    pub stats: ChStats,
}

#[derive(Clone, Copy)]
struct Arc {
    to: u32,
    w: u32,
    mid: u32,
    edge: u32,
}

fn add_arc(list: &mut Vec<Arc>, a: Arc) -> bool {
    for x in list.iter_mut() {
        if x.to == a.to {
            if a.w < x.w {
                *x = a;
            }
            return false;
        }
    }
    list.push(a);
    true
}

struct Dyn {
    out: Vec<Vec<Arc>>,
    inn: Vec<Vec<Arc>>,
    ws: Labels,
    heap: MinHeap,
}

impl Dyn {
    /// Dijkstra bỏ qua `skip`, dừng khi vượt `max_cost` hoặc đã chốt `limit` nút.
    fn witness(&mut self, u: u32, skip: u32, max_cost: u32, limit: u32) {
        self.ws.reset();
        self.heap.clear();
        self.ws.set(u, 0, NONE);
        self.heap.push(0, u);
        let mut settled = 0;
        while let Some((d, x)) = self.heap.pop() {
            if self.ws.is_closed(x) {
                continue;
            }
            self.ws.close(x);
            settled += 1;
            if d > max_cost || settled > limit {
                break;
            }
            for a in &self.out[x as usize] {
                if a.to == skip {
                    continue;
                }
                let nd = d.saturating_add(a.w);
                if nd <= max_cost && nd < self.ws.dist(a.to) {
                    self.ws.set(a.to, nd, x);
                    self.heap.push(nd, a.to);
                }
            }
        }
    }

    /// Các shortcut cần thêm khi co `v`: `(u, x, trọng số)`.
    fn shortcuts_for(&mut self, v: u32, limit: u32) -> Vec<(u32, u32, u32)> {
        let mut res = Vec::new();
        let max_out = self.out[v as usize].iter().map(|a| a.w).max().unwrap_or(0);
        let ins: Vec<Arc> = self.inn[v as usize].clone();
        for ain in &ins {
            let u = ain.to;
            if self.out[v as usize].iter().all(|a| a.to == u) {
                continue;
            }
            let max_cost = ain.w.saturating_add(max_out);
            self.witness(u, v, max_cost, limit);
            for aout in &self.out[v as usize] {
                let x = aout.to;
                if x == u {
                    continue;
                }
                let s = ain.w.saturating_add(aout.w);
                if self.ws.dist(x) > s {
                    res.push((u, x, s));
                }
            }
        }
        res
    }
}

impl Ch {
    /// Dựng CH trên trọng số của `view` (theo nút, bỏ qua chi phí rẽ).
    pub fn build(view: View) -> Self {
        let t0 = std::time::Instant::now();
        let n = view.node_count();
        let mut d = Dyn {
            out: vec![Vec::new(); n],
            inn: vec![Vec::new(); n],
            ws: Labels::new(n),
            heap: MinHeap::new(),
        };
        let mut original_arcs = 0;
        for e in 0..view.g.edge_count() {
            let w = view.w[e];
            let (u, v) = (view.g.tail[e], view.g.head[e]);
            if w == INF || u == v {
                continue;
            }
            let a = Arc {
                to: v,
                w,
                mid: NONE,
                edge: e as u32,
            };
            let b = Arc { to: u, ..a };
            if add_arc(&mut d.out[u as usize], a) {
                original_arcs += 1;
            }
            add_arc(&mut d.inn[v as usize], b);
        }

        let mut contracted = vec![false; n];
        let mut deleted_nb = vec![0i32; n];
        let mut level = vec![0u32; n];
        let mut prio = vec![0i32; n];
        let mut heap: BinaryHeap<Reverse<(i32, u32)>> = BinaryHeap::new();
        const SIM_LIMIT: u32 = 60;
        const REAL_LIMIT: u32 = 500;

        let priority = |d: &mut Dyn, v: u32, dn: i32, lvl: u32| -> i32 {
            let sc = d.shortcuts_for(v, SIM_LIMIT).len() as i32;
            let removed = (d.out[v as usize].len() + d.inn[v as usize].len()) as i32;
            2 * (sc - removed) + dn + lvl as i32
        };
        for v in 0..n as u32 {
            prio[v as usize] = priority(&mut d, v, 0, 0);
            heap.push(Reverse((prio[v as usize], v)));
        }

        let mut rank = vec![NONE; n];
        let mut order: Vec<u32> = Vec::with_capacity(n);
        let mut fwd_l: Vec<Vec<(u32, u32, u32, u32)>> = vec![Vec::new(); n];
        let mut bwd_l: Vec<Vec<(u32, u32, u32, u32)>> = vec![Vec::new(); n];
        let mut shortcuts = 0usize;

        while let Some(Reverse((p, v))) = heap.pop() {
            let vi = v as usize;
            if contracted[vi] || p != prio[vi] {
                continue;
            }
            // cập nhật lười: tính lại, nếu tệ hơn đỉnh hàng đợi thì đẩy lại
            let np = priority(&mut d, v, deleted_nb[vi], level[vi]);
            if np > p
                && let Some(&Reverse((top, _))) = heap.peek()
                && np > top
            {
                prio[vi] = np;
                heap.push(Reverse((np, v)));
                continue;
            }
            // co v
            let sc = d.shortcuts_for(v, REAL_LIMIT);
            rank[vi] = order.len() as u32;
            order.push(v);
            contracted[vi] = true;
            let outs = std::mem::take(&mut d.out[vi]);
            let ins = std::mem::take(&mut d.inn[vi]);
            // ghi các cung của v (chỉ tới nút chưa co, hạng cao hơn) vào đồ thị đi lên, theo nút gốc; đổi sang hạng ở cuối
            for a in &outs {
                fwd_l[vi].push((a.to, a.w, a.mid, a.edge));
            }
            for a in &ins {
                bwd_l[vi].push((a.to, a.w, a.mid, a.edge));
            }
            for a in &outs {
                d.inn[a.to as usize].retain(|x| x.to != v);
            }
            for a in &ins {
                d.out[a.to as usize].retain(|x| x.to != v);
            }
            for (u, x, w) in sc {
                let a = Arc {
                    to: x,
                    w,
                    mid: v,
                    edge: NONE,
                };
                if add_arc(&mut d.out[u as usize], a) {
                    shortcuts += 1;
                }
                add_arc(&mut d.inn[x as usize], Arc { to: u, ..a });
            }
            // cập nhật láng giềng
            let mut nbrs: Vec<u32> = outs.iter().chain(ins.iter()).map(|a| a.to).collect();
            nbrs.sort_unstable();
            nbrs.dedup();
            for nb in nbrs {
                let ni = nb as usize;
                deleted_nb[ni] += 1;
                level[ni] = level[ni].max(level[vi] + 1);
                prio[ni] = priority(&mut d, nb, deleted_nb[ni], level[ni]);
                heap.push(Reverse((prio[ni], nb)));
            }
        }

        // đổi sang không gian hạng
        let conv = |lists: Vec<Vec<(u32, u32, u32, u32)>>| -> Vec<Vec<(u32, u32, u32, u32)>> {
            let mut by_rank: Vec<Vec<(u32, u32, u32, u32)>> = vec![Vec::new(); n];
            for (v, l) in lists.into_iter().enumerate() {
                let r = rank[v] as usize;
                by_rank[r] = l
                    .into_iter()
                    .map(|(h, w, mid, e)| {
                        (
                            rank[h as usize],
                            w,
                            if mid == NONE {
                                NONE
                            } else {
                                rank[mid as usize]
                            },
                            e,
                        )
                    })
                    .collect();
            }
            by_rank
        };
        let fwd = UpGraph::from_lists(&conv(fwd_l));
        let bwd = UpGraph::from_lists(&conv(bwd_l));
        let max_level = level.iter().copied().max().unwrap_or(0);
        Ch {
            new_id: rank,
            old_id: order,
            fwd,
            bwd,
            stats: ChStats {
                nodes: n,
                original_arcs,
                shortcuts,
                preprocess_s: t0.elapsed().as_secs_f64(),
                max_level,
            },
        }
    }

    pub fn bytes(&self) -> usize {
        self.fwd.bytes()
            + self.bwd.bytes()
            + (self.new_id.len() + self.old_id.len()) * size_of::<u32>()
    }
    pub fn node_count(&self) -> usize {
        self.new_id.len()
    }
    /// Tổng số cung đi lên (thuận + ngược).
    pub fn arc_count(&self) -> usize {
        self.fwd.len() + self.bwd.len()
    }

    /// Chi phí ngắn nhất `s → t` (nút gốc), `None` nếu không tới được.
    pub fn distance(&self, ws: &mut ChWorkspace, s: NodeId, t: NodeId) -> Option<u32> {
        self.query_legs(ws, &[Leg { node: s, cost: 0 }], &[Leg { node: t, cost: 0 }])
            .map(|r| r.cost)
    }

    /// Truy vấn nhiều chân nguồn và nhiều chân đích (điểm trên cạnh). Chi phí đã gồm chi phí chân.
    pub fn query_legs(&self, ws: &mut ChWorkspace, src: &[Leg], dst: &[Leg]) -> Option<ChResult> {
        ws.f.reset();
        ws.b.reset();
        ws.hf.clear();
        ws.hb.clear();
        for l in src {
            let r = self.new_id[l.node as usize];
            if l.cost < ws.f.dist(r) {
                ws.f.set(r, l.cost, NONE);
                ws.hf.push(l.cost, r);
            }
        }
        for l in dst {
            let r = self.new_id[l.node as usize];
            if l.cost < ws.b.dist(r) {
                ws.b.set(r, l.cost, NONE);
                ws.hb.push(l.cost, r);
            }
        }
        let mut mu = INF;
        let mut meet = NONE;
        let mut st = Stats::default();
        loop {
            let kf = top(&mut ws.hf, &ws.f).filter(|&k| k < mu);
            let kb = top(&mut ws.hb, &ws.b).filter(|&k| k < mu);
            let forward = match (kf, kb) {
                (None, None) => break,
                (Some(_), None) => true,
                (None, Some(_)) => false,
                (Some(a), Some(b)) => a <= b,
            };
            if forward {
                let (_, u) = ws.hf.pop().expect("đã kiểm tra");
                ws.f.close(u);
                st.settled += 1;
                let d = ws.f.dist(u);
                let ob = ws.b.dist(u);
                if ob != INF && d.saturating_add(ob) < mu {
                    mu = d + ob;
                    meet = u;
                }
                for a in self.fwd.arcs(u) {
                    st.relaxed += 1;
                    let v = self.fwd.head[a];
                    let nd = d.saturating_add(self.fwd.w[a]);
                    if nd < ws.f.dist(v) {
                        ws.f.set(v, nd, u);
                        ws.hf.push(nd, v);
                    }
                }
            } else {
                let (_, u) = ws.hb.pop().expect("đã kiểm tra");
                ws.b.close(u);
                st.settled += 1;
                let d = ws.b.dist(u);
                let of = ws.f.dist(u);
                if of != INF && d.saturating_add(of) < mu {
                    mu = d + of;
                    meet = u;
                }
                for a in self.bwd.arcs(u) {
                    st.relaxed += 1;
                    let v = self.bwd.head[a];
                    let nd = d.saturating_add(self.bwd.w[a]);
                    if nd < ws.b.dist(v) {
                        ws.b.set(v, nd, u);
                        ws.hb.push(nd, v);
                    }
                }
            }
        }
        ws.stats = st;
        (meet != NONE).then_some(ChResult { cost: mu, meet })
    }

    /// Dãy cạnh gốc của đường tìm được ở truy vấn gần nhất (đã giải nén shortcut). Trả cả nút gốc đầu và cuối.
    pub fn unpack(&self, ws: &ChWorkspace, r: &ChResult) -> (Vec<EdgeId>, NodeId, NodeId) {
        let mut up_nodes = vec![r.meet];
        let mut x = r.meet;
        while ws.f.pred(x) != NONE {
            x = ws.f.pred(x);
            up_nodes.push(x);
        }
        up_nodes.reverse(); // nguồn ... meet
        let mut edges = Vec::new();
        for w in up_nodes.windows(2) {
            self.unpack_arc(w[0], w[1], &mut edges);
        }
        let src = self.old_id[up_nodes[0] as usize];
        let mut y = r.meet;
        while ws.b.pred(y) != NONE {
            let nxt = ws.b.pred(y);
            self.unpack_arc(y, nxt, &mut edges);
            y = nxt;
        }
        (edges, src, self.old_id[y as usize])
    }

    /// Giải nén cung `a → b` (hạng) thành các cạnh gốc, đệ quy qua nút giữa.
    fn unpack_arc(&self, a: u32, b: u32, out: &mut Vec<EdgeId>) {
        let (g, idx) = if a < b {
            let i = self.fwd.find(a, b).expect("cung thuận phải tồn tại");
            (&self.fwd, i)
        } else {
            let i = self.bwd.find(b, a).expect("cung ngược phải tồn tại");
            (&self.bwd, i)
        };
        let mid = g.mid[idx];
        if mid == NONE {
            out.push(g.edge[idx]);
        } else {
            self.unpack_arc(a, mid, out);
            self.unpack_arc(mid, b, out);
        }
    }

    /// Đường ngắn nhất theo nút (nút gốc), đã giải nén.
    pub fn path(&self, ws: &mut ChWorkspace, s: NodeId, t: NodeId) -> Option<Path> {
        if s == t {
            return Some(Path {
                cost: 0,
                edges: Vec::new(),
            });
        }
        let r = self.query_legs(ws, &[Leg { node: s, cost: 0 }], &[Leg { node: t, cost: 0 }])?;
        let (edges, _, _) = self.unpack(ws, &r);
        Some(Path {
            cost: r.cost,
            edges,
        })
    }

    /// Lưu ra file nhị phân (postcard) để khỏi dựng lại mỗi lần chạy.
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        let bytes = postcard::to_stdvec(self).map_err(std::io::Error::other)?;
        std::fs::write(path, bytes)
    }
    pub fn load(path: &std::path::Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        postcard::from_bytes(&bytes).map_err(std::io::Error::other)
    }
}

fn top(h: &mut MinHeap, l: &Labels) -> Option<u32> {
    while let Some((k, u)) = h.peek() {
        if l.is_closed(u) || k != l.dist(u) {
            h.pop();
        } else {
            return Some(k);
        }
    }
    None
}

#[derive(Debug, Clone, Copy)]
pub struct ChResult {
    pub cost: u32,
    /// Nút gặp nhau (hạng).
    pub meet: u32,
}

/// Bộ nhớ làm việc cho truy vấn CH, dùng lại giữa các truy vấn của một luồng.
pub struct ChWorkspace {
    pub f: Labels,
    pub b: Labels,
    hf: MinHeap,
    hb: MinHeap,
    pub stats: Stats,
}

impl ChWorkspace {
    pub fn new(n: usize) -> Self {
        Self {
            f: Labels::new(n),
            b: Labels::new(n),
            hf: MinHeap::new(),
            hb: MinHeap::new(),
            stats: Stats::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::{Rev, sssp};
    use crate::profile::{Costs, Metric, Profile};
    use road_graph::fixture::{compile_fixture, generate};

    #[test]
    fn ch_distances_equal_dijkstra_on_fixture() {
        let g = compile_fixture(&generate());
        let rev = Rev::build(&g);
        for profile in Profile::ALL {
            let c = Costs::build(&g, profile, Metric::Time);
            let view = View::new(&g, &rev, &c.w);
            let ch = Ch::build(view);
            let mut ws = ChWorkspace::new(g.node_count());
            let n = g.node_count() as u32;
            for s in (0..n).step_by(7) {
                let truth = sssp(view, s, true);
                for t in (0..n).step_by(5) {
                    let got = ch.distance(&mut ws, s, t);
                    let want = (truth[t as usize] != INF).then_some(truth[t as usize]);
                    assert_eq!(got, want, "{profile:?} {s}->{t}");
                }
            }
        }
    }

    #[test]
    fn unpacked_path_is_valid_and_costs_the_same() {
        let g = compile_fixture(&generate());
        let rev = Rev::build(&g);
        let c = Costs::build(&g, Profile::Car, Metric::Time);
        let view = View::new(&g, &rev, &c.w);
        let ch = Ch::build(view);
        let mut ws = ChWorkspace::new(g.node_count());
        let n = g.node_count() as u32;
        let mut checked = 0;
        for s in (0..n).step_by(11) {
            for t in (0..n).step_by(9) {
                if let Some(p) = ch.path(&mut ws, s, t) {
                    assert_eq!(p.cost as u64, p.cost_under(&c.w));
                    if s != t {
                        let nodes = p.nodes(&g);
                        assert_eq!((nodes[0], *nodes.last().unwrap()), (s, t));
                        for w in p.edges.windows(2) {
                            assert_eq!(g.head[w[0] as usize], g.tail[w[1] as usize]);
                        }
                    }
                    checked += 1;
                }
            }
        }
        assert!(checked > 100);
    }

    #[test]
    fn save_and_load_round_trip() {
        let g = compile_fixture(&generate());
        let rev = Rev::build(&g);
        let c = Costs::build(&g, Profile::Car, Metric::Time);
        let ch = Ch::build(View::new(&g, &rev, &c.w));
        let path = std::env::temp_dir().join(format!("route-engine-ch-{}.bin", std::process::id()));
        ch.save(&path).unwrap();
        let back = Ch::load(&path).unwrap();
        std::fs::remove_file(&path).ok();
        let mut ws = ChWorkspace::new(g.node_count());
        assert_eq!(ch.distance(&mut ws, 0, 100), back.distance(&mut ws, 0, 100));
        assert_eq!(ch.fwd.head, back.fwd.head);
    }
}
