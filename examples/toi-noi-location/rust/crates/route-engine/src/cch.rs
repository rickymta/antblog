//! LOC-27: Customizable Contraction Hierarchies (CCH): tách thứ tự nút khỏi trọng số.
//!
//! Ba pha:
//! 1. **Thứ tự** (một lần, chỉ phụ thuộc hình dạng đồ thị): chia đệ quy theo toạ độ (nested dissection kiểu đơn giản):
//!    cắt đôi theo trục dài, lấy tập phủ đỉnh nhỏ nhất của các cạnh cắt làm «vách ngăn» và xếp vách ngăn **sau** hai nửa.
//! 2. **Cấu trúc** (một lần): co nút theo thứ tự đó **không cần tìm nhân chứng**, nên tập shortcut không phụ thuộc trọng số.
//! 3. **Tuỳ biến** (mỗi lần đổi trọng số, nhanh): duyệt các tam giác theo hạng, `w(y,z) = min(w(y,z), w(y,x) + w(x,z))`.
//!    Kết quả là một [`Ch`] dùng được với mọi truy vấn và ma trận của LOC-25, LOC-26.
//!
//! Đánh đổi: không nhân chứng nên nhiều cung hơn CH thường, truy vấn chậm hơn nhiều (đo ở LOC-27); đổi lại cập nhật trọng số
//! (theo giờ, theo kẹt xe) không phải dựng lại.

use crate::ch::{Ch, ChStats, UpGraph};
use crate::net::{INF, NONE, View};
use crate::search::NodeXy;
use road_graph::Graph;

#[derive(Debug, Clone, Default)]
pub struct CchStats {
    pub nodes: usize,
    pub arcs: usize,
    pub max_up_degree: usize,
    pub triangles: u64,
    pub order_s: f64,
    pub symbolic_s: f64,
}

pub struct Cch {
    pub new_id: Vec<u32>,
    pub old_id: Vec<u32>,
    up_first: Vec<u32>,
    up_head: Vec<u32>,
    /// Với mỗi cạnh của đồ thị đường: cung chứa nó (hoặc `NONE` nếu cạnh không thuộc topo), và có đi lên không.
    edge_arc: Vec<u32>,
    edge_up: Vec<bool>,
    pub stats: CchStats,
}

struct Nd<'a> {
    xy: &'a [[f64; 2]],
    adj_first: &'a [u32],
    adj: &'a [u32],
    mark: Vec<u32>,
    side: Vec<u8>,
    stamp: u32,
    leaf: usize,
}

impl Nd<'_> {
    /// Tập phủ đỉnh nhỏ nhất của các cạnh nối nửa 0 với nửa 1 trong ô hiện tại: cặp ghép cực đại (Kuhn) rồi dựng theo Kőnig.
    fn min_vertex_cover(&self, nodes: &[u32], stamp: u32) -> Vec<u32> {
        let mut left: Vec<u32> = Vec::new();
        let mut adj: Vec<Vec<u32>> = Vec::new();
        let mut right_ids: rustc_hash::FxHashMap<u32, u32> = rustc_hash::FxHashMap::default();
        let mut right: Vec<u32> = Vec::new();
        for &v in nodes {
            if self.side[v as usize] != 0 {
                continue;
            }
            let mut nb = Vec::new();
            for &u in &self.adj
                [self.adj_first[v as usize] as usize..self.adj_first[v as usize + 1] as usize]
            {
                if self.mark[u as usize] == stamp && self.side[u as usize] == 1 {
                    let id = *right_ids.entry(u).or_insert_with(|| {
                        right.push(u);
                        (right.len() - 1) as u32
                    });
                    if !nb.contains(&id) {
                        nb.push(id);
                    }
                }
            }
            if !nb.is_empty() {
                left.push(v);
                adj.push(nb);
            }
        }
        let (nl, nr) = (left.len(), right.len());
        let mut match_r = vec![u32::MAX; nr];
        let mut match_l = vec![u32::MAX; nl];
        fn augment(
            u: usize,
            adj: &[Vec<u32>],
            seen: &mut [bool],
            match_r: &mut [u32],
            match_l: &mut [u32],
        ) -> bool {
            for &v in &adj[u] {
                if seen[v as usize] {
                    continue;
                }
                seen[v as usize] = true;
                if match_r[v as usize] == u32::MAX
                    || augment(match_r[v as usize] as usize, adj, seen, match_r, match_l)
                {
                    match_r[v as usize] = u as u32;
                    match_l[u] = v;
                    return true;
                }
            }
            false
        }
        let mut seen = vec![false; nr];
        for u in 0..nl {
            seen.fill(false);
            augment(u, &adj, &mut seen, &mut match_r, &mut match_l);
        }
        // Kőnig: Z = các đỉnh tới được từ đỉnh trái chưa ghép bằng đường xen kẽ; phủ = (trái \ Z) ∪ (phải ∩ Z)
        let mut zl = vec![false; nl];
        let mut zr = vec![false; nr];
        let mut stack: Vec<usize> = (0..nl).filter(|&u| match_l[u] == u32::MAX).collect();
        for &u in &stack {
            zl[u] = true;
        }
        while let Some(u) = stack.pop() {
            for &v in &adj[u] {
                if !zr[v as usize] && match_l[u] != v {
                    zr[v as usize] = true;
                    let m = match_r[v as usize];
                    if m != u32::MAX && !zl[m as usize] {
                        zl[m as usize] = true;
                        stack.push(m as usize);
                    }
                }
            }
        }
        let mut cover: Vec<u32> = (0..nl).filter(|&u| !zl[u]).map(|u| left[u]).collect();
        cover.extend((0..nr).filter(|&v| zr[v]).map(|v| right[v]));
        cover
    }

    fn rec(&mut self, mut nodes: Vec<u32>, out: &mut Vec<u32>) {
        if nodes.len() <= self.leaf {
            out.extend(nodes);
            return;
        }
        self.stamp += 1;
        let stamp = self.stamp;
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for &v in &nodes {
            self.mark[v as usize] = stamp;
            for k in 0..2 {
                lo[k] = lo[k].min(self.xy[v as usize][k]);
                hi[k] = hi[k].max(self.xy[v as usize][k]);
            }
        }
        let axis = usize::from(hi[1] - lo[1] > hi[0] - lo[0]);
        let mid = nodes.len() / 2;
        let xy = self.xy;
        nodes.select_nth_unstable_by(mid, |&a, &b| {
            xy[a as usize][axis]
                .total_cmp(&xy[b as usize][axis])
                .then(a.cmp(&b))
        });
        for (i, &v) in nodes.iter().enumerate() {
            self.side[v as usize] = u8::from(i >= mid);
        }
        // các cạnh cắt giữa hai nửa tạo thành đồ thị hai phía; vách ngăn = tập phủ đỉnh nhỏ nhất của nó (định lý Kőnig)
        let sep = self.min_vertex_cover(&nodes, stamp);
        let sep_set: rustc_hash::FxHashSet<u32> = sep.iter().copied().collect();
        let (mut a, mut b) = (Vec::new(), Vec::new());
        for &v in &nodes {
            if sep_set.contains(&v) {
                continue;
            }
            if self.side[v as usize] == 0 {
                a.push(v)
            } else {
                b.push(v)
            }
        }
        if a.is_empty() || b.is_empty() {
            // cắt không tách được (vd. mọi nút trùng toạ độ): xếp tuỳ ý
            out.extend(nodes);
            return;
        }
        self.rec(a, out);
        self.rec(b, out);
        out.extend(sep);
    }
}

fn sorted_union(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut r = Vec::with_capacity(a.len() + b.len());
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => {
                r.push(a[i]);
                i += 1;
            }
            std::cmp::Ordering::Greater => {
                r.push(b[j]);
                j += 1;
            }
            std::cmp::Ordering::Equal => {
                r.push(a[i]);
                i += 1;
                j += 1;
            }
        }
    }
    r.extend_from_slice(&a[i..]);
    r.extend_from_slice(&b[j..]);
    r
}

impl Cch {
    /// Dựng thứ tự và cấu trúc từ tập cạnh đi được của `view` (profile), không dùng giá trị trọng số.
    pub fn build(view: View, xy: &NodeXy) -> Self {
        let g: &Graph = view.g;
        let n = g.node_count();
        let t0 = std::time::Instant::now();
        // kề vô hướng từ các cạnh đi được
        let mut deg = vec![0u32; n + 1];
        for e in 0..g.edge_count() {
            if view.w[e] != INF && g.tail[e] != g.head[e] {
                deg[g.tail[e] as usize + 1] += 1;
                deg[g.head[e] as usize + 1] += 1;
            }
        }
        for i in 0..n {
            deg[i + 1] += deg[i];
        }
        let mut fill = deg.clone();
        let mut adj = vec![0u32; deg[n] as usize];
        for e in 0..g.edge_count() {
            if view.w[e] != INF && g.tail[e] != g.head[e] {
                let (a, b) = (g.tail[e] as usize, g.head[e] as usize);
                adj[fill[a] as usize] = b as u32;
                fill[a] += 1;
                adj[fill[b] as usize] = a as u32;
                fill[b] += 1;
            }
        }
        let mut nd = Nd {
            xy: &xy.xy,
            adj_first: &deg,
            adj: &adj,
            mark: vec![0; n],
            side: vec![0; n],
            stamp: 0,
            leaf: 3,
        };
        let mut order = Vec::with_capacity(n);
        nd.rec((0..n as u32).collect(), &mut order);
        debug_assert_eq!(order.len(), n);
        let order_s = t0.elapsed().as_secs_f64();

        let t1 = std::time::Instant::now();
        let mut new_id = vec![0u32; n];
        for (r, &v) in order.iter().enumerate() {
            new_id[v as usize] = r as u32;
        }
        let mut up: Vec<Vec<u32>> = vec![Vec::new(); n];
        for e in 0..g.edge_count() {
            if view.w[e] != INF && g.tail[e] != g.head[e] {
                let (a, b) = (new_id[g.tail[e] as usize], new_id[g.head[e] as usize]);
                up[a.min(b) as usize].push(a.max(b));
            }
        }
        for l in &mut up {
            l.sort_unstable();
            l.dedup();
        }
        for r in 0..n {
            let mine = std::mem::take(&mut up[r]);
            if let Some(&p) = mine.first() {
                let merged = sorted_union(&up[p as usize], &mine[1..]);
                up[p as usize] = merged;
            }
            up[r] = mine;
        }
        let mut up_first = Vec::with_capacity(n + 1);
        let mut up_head = Vec::new();
        up_first.push(0u32);
        let mut max_deg = 0;
        let mut triangles = 0u64;
        for l in &up {
            max_deg = max_deg.max(l.len());
            triangles += (l.len() as u64 * (l.len() as u64).saturating_sub(1)) / 2;
            up_head.extend_from_slice(l);
            up_first.push(up_head.len() as u32);
        }
        let find = |lo: u32, hi: u32| -> u32 {
            let s = up_first[lo as usize] as usize..up_first[lo as usize + 1] as usize;
            let i = up_head[s.clone()]
                .binary_search(&hi)
                .expect("cung phải tồn tại");
            (s.start + i) as u32
        };
        let mut edge_arc = vec![NONE; g.edge_count()];
        let mut edge_up = vec![false; g.edge_count()];
        for e in 0..g.edge_count() {
            if view.w[e] != INF && g.tail[e] != g.head[e] {
                let (a, b) = (new_id[g.tail[e] as usize], new_id[g.head[e] as usize]);
                edge_arc[e] = find(a.min(b), a.max(b));
                edge_up[e] = a < b;
            }
        }
        let symbolic_s = t1.elapsed().as_secs_f64();
        let arcs = up_head.len();
        Cch {
            new_id,
            old_id: order,
            up_first,
            up_head,
            edge_arc,
            edge_up,
            stats: CchStats {
                nodes: n,
                arcs,
                max_up_degree: max_deg,
                triangles,
                order_s,
                symbolic_s,
            },
        }
    }

    fn find(&self, lo: u32, hi: u32) -> usize {
        let s = self.up_first[lo as usize] as usize..self.up_first[lo as usize + 1] as usize;
        s.start
            + self.up_head[s]
                .binary_search(&hi)
                .expect("cung phải tồn tại")
    }

    /// Tính trọng số các cung theo mảng trọng số cạnh `w` (cùng tập cạnh đi được với lúc dựng). Trả về một [`Ch`].
    pub fn customize(&self, w: &[u32]) -> Ch {
        let t0 = std::time::Instant::now();
        let m = self.up_head.len();
        let (mut w_up, mut w_dn) = (vec![INF; m], vec![INF; m]);
        let (mut mid_up, mut mid_dn) = (vec![NONE; m], vec![NONE; m]);
        let (mut e_up, mut e_dn) = (vec![NONE; m], vec![NONE; m]);
        for (e, &a) in self.edge_arc.iter().enumerate() {
            if a == NONE || w[e] == INF {
                continue;
            }
            let a = a as usize;
            if self.edge_up[e] {
                if w[e] < w_up[a] {
                    w_up[a] = w[e];
                    e_up[a] = e as u32;
                }
            } else if w[e] < w_dn[a] {
                w_dn[a] = w[e];
                e_dn[a] = e as u32;
            }
        }
        let n = self.up_first.len() - 1;
        for x in 0..n as u32 {
            let (s, t) = (
                self.up_first[x as usize] as usize,
                self.up_first[x as usize + 1] as usize,
            );
            for i in s..t {
                let y = self.up_head[i];
                let (dn_xy, up_xy) = (w_dn[i], w_up[i]);
                for j in i + 1..t {
                    let z = self.up_head[j];
                    let a = self.find(y, z);
                    // y → x → z
                    let c1 = dn_xy.saturating_add(w_up[j]);
                    if c1 < w_up[a] {
                        w_up[a] = c1;
                        mid_up[a] = x;
                        e_up[a] = NONE;
                    }
                    // z → x → y
                    let c2 = w_dn[j].saturating_add(up_xy);
                    if c2 < w_dn[a] {
                        w_dn[a] = c2;
                        mid_dn[a] = x;
                        e_dn[a] = NONE;
                    }
                }
            }
        }
        let shortcuts = (0..m)
            .filter(|&a| {
                (w_up[a] != INF && mid_up[a] != NONE) || (w_dn[a] != INF && mid_dn[a] != NONE)
            })
            .count();
        // chỉ giữ cung có trọng số hữu hạn theo từng chiều (cung vô hạn không bao giờ được dùng)
        let mut fl: Vec<Vec<(u32, u32, u32, u32)>> = vec![Vec::new(); n];
        let mut bl: Vec<Vec<(u32, u32, u32, u32)>> = vec![Vec::new(); n];
        for x in 0..n {
            for a in self.up_first[x] as usize..self.up_first[x + 1] as usize {
                if w_up[a] != INF {
                    fl[x].push((self.up_head[a], w_up[a], mid_up[a], e_up[a]));
                }
                if w_dn[a] != INF {
                    bl[x].push((self.up_head[a], w_dn[a], mid_dn[a], e_dn[a]));
                }
            }
        }
        let fwd = UpGraph::from_lists(&fl);
        let bwd = UpGraph::from_lists(&bl);
        Ch {
            new_id: self.new_id.clone(),
            old_id: self.old_id.clone(),
            fwd,
            bwd,
            stats: ChStats {
                nodes: n,
                original_arcs: self.edge_arc.iter().filter(|&&a| a != NONE).count(),
                shortcuts,
                preprocess_s: t0.elapsed().as_secs_f64(),
                max_level: 0,
            },
        }
    }

    pub fn bytes(&self) -> usize {
        (self.new_id.len()
            + self.old_id.len()
            + self.up_first.len()
            + self.up_head.len()
            + self.edge_arc.len())
            * size_of::<u32>()
            + self.edge_up.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ch::ChWorkspace;
    use crate::net::{Rev, sssp};
    use crate::profile::{Costs, Metric, Profile};
    use road_graph::fixture::{compile_fixture, generate};

    #[test]
    fn customized_ch_matches_dijkstra_for_two_different_metrics() {
        let g = compile_fixture(&generate());
        let rev = Rev::build(&g);
        let xy = NodeXy::build(&g);
        let c1 = Costs::build(&g, Profile::Car, Metric::Time);
        let cch = Cch::build(View::new(&g, &rev, &c1.w), &xy);
        // metric thứ hai: cùng tập cạnh, trọng số khác hẳn
        let w2: Vec<u32> =
            c1.w.iter()
                .enumerate()
                .map(|(e, &w)| {
                    if w == INF {
                        INF
                    } else {
                        w / 2 + (e as u32 * 7919) % 5000 + 1
                    }
                })
                .collect();
        for w in [&c1.w, &w2] {
            let ch = cch.customize(w);
            let view = View::new(&g, &rev, w);
            let mut ws = ChWorkspace::new(g.node_count());
            let n = g.node_count() as u32;
            for s in (0..n).step_by(9) {
                let truth = sssp(view, s, true);
                for t in (0..n).step_by(7) {
                    let want = (truth[t as usize] != INF).then_some(truth[t as usize]);
                    assert_eq!(ch.distance(&mut ws, s, t), want, "{s}->{t}");
                }
            }
            // giải nén cũng đúng
            let p = ch.path(&mut ws, 0, n - 1);
            if let Some(p) = p {
                assert_eq!(u64::from(p.cost), p.cost_under(w));
            }
        }
    }

    #[test]
    fn order_is_a_permutation() {
        let g = compile_fixture(&generate());
        let rev = Rev::build(&g);
        let xy = NodeXy::build(&g);
        let c = Costs::build(&g, Profile::Motorbike, Metric::Time);
        let cch = Cch::build(View::new(&g, &rev, &c.w), &xy);
        let mut seen = vec![false; g.node_count()];
        for &v in &cch.old_id {
            assert!(!seen[v as usize]);
            seen[v as usize] = true;
        }
        assert!(seen.iter().all(|&b| b));
    }
}
