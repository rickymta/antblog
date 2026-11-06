//! LOC-26: ma trận chi phí many-to-many dạng bucket trên CH.
//!
//! Với `M` đích: mỗi đích chạy **một** tìm kiếm ngược đi lên, rồi đặt `(đích, khoảng cách)` vào «bucket» của từng nút đã chốt.
//! Với `N` nguồn: mỗi nguồn chạy **một** tìm kiếm thuận đi lên; ở mỗi nút chốt, quét bucket của nút đó để cập nhật cả hàng.
//! Tổng `N + M` tìm kiếm nhỏ thay cho `N × M` truy vấn. Mỗi nguồn/đích là danh sách chân ([`Leg`]), nên nhận được điểm trên cạnh.

use crate::ch::{Ch, ChWorkspace};
use crate::net::{INF, Labels, MinHeap, NONE};
use crate::position::Leg;
use rayon::prelude::*;

/// Ma trận chi phí hàng-chính. `INF` là không tới được.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<u32>,
}

impl Matrix {
    pub fn get(&self, r: usize, c: usize) -> u32 {
        self.data[r * self.cols + c]
    }
    pub fn row(&self, r: usize) -> &[u32] {
        &self.data[r * self.cols..(r + 1) * self.cols]
    }
}

/// Bộ nhớ làm việc cho một tìm kiếm đi lên một chiều.
pub struct UpWorkspace {
    labels: Labels,
    heap: MinHeap,
}

impl UpWorkspace {
    pub fn new(n: usize) -> Self {
        Self {
            labels: Labels::new(n),
            heap: MinHeap::new(),
        }
    }
}

/// Tìm kiếm đi lên đầy đủ (không dừng sớm) từ các chân; gọi `visit(hạng, khoảng cách)` ở mỗi nút chốt.
fn upward_search(
    ch: &Ch,
    forward: bool,
    legs: &[Leg],
    ws: &mut UpWorkspace,
    mut visit: impl FnMut(u32, u32),
) {
    let g = if forward { &ch.fwd } else { &ch.bwd };
    ws.labels.reset();
    ws.heap.clear();
    for l in legs {
        let r = ch.new_id[l.node as usize];
        if l.cost < ws.labels.dist(r) {
            ws.labels.set(r, l.cost, NONE);
            ws.heap.push(l.cost, r);
        }
    }
    while let Some((k, u)) = ws.heap.pop() {
        if ws.labels.is_closed(u) || k != ws.labels.dist(u) {
            continue;
        }
        ws.labels.close(u);
        visit(u, k);
        for a in g.arcs(u) {
            let nd = k.saturating_add(g.w[a]);
            let v = g.head[a];
            if nd < ws.labels.dist(v) {
                ws.labels.set(v, nd, u);
                ws.heap.push(nd, v);
            }
        }
    }
}

/// Bucket theo nút: CSR `(đích, khoảng cách)`.
struct Buckets {
    first: Vec<u32>,
    entries: Vec<(u32, u32)>,
}

fn build_buckets(ch: &Ch, targets: &[Vec<Leg>], parallel: bool) -> Buckets {
    let n = ch.node_count();
    let run = |j: usize, ws: &mut UpWorkspace| {
        let mut v: Vec<(u32, u32)> = Vec::new();
        upward_search(ch, false, &targets[j], ws, |node, d| v.push((node, d)));
        v
    };
    let per_target: Vec<Vec<(u32, u32)>> = if parallel {
        (0..targets.len())
            .into_par_iter()
            .map_init(|| UpWorkspace::new(n), |ws, j| run(j, ws))
            .collect()
    } else {
        let mut ws = UpWorkspace::new(n);
        (0..targets.len()).map(|j| run(j, &mut ws)).collect()
    };
    let mut first = vec![0u32; n + 1];
    for v in &per_target {
        for &(node, _) in v {
            first[node as usize + 1] += 1;
        }
    }
    for i in 0..n {
        first[i + 1] += first[i];
    }
    let mut fill = first.clone();
    let mut entries = vec![(0u32, 0u32); first[n] as usize];
    for (j, v) in per_target.iter().enumerate() {
        for &(node, d) in v {
            entries[fill[node as usize] as usize] = (j as u32, d);
            fill[node as usize] += 1;
        }
    }
    Buckets { first, entries }
}

/// Ma trận bucket. `parallel = true` chạy tìm kiếm ngược và các hàng song song bằng `rayon`.
pub fn bucket_matrix(
    ch: &Ch,
    sources: &[Vec<Leg>],
    targets: &[Vec<Leg>],
    parallel: bool,
) -> Matrix {
    let n = ch.node_count();
    let cols = targets.len();
    let buckets = build_buckets(ch, targets, parallel);
    let row = |i: usize, ws: &mut UpWorkspace| -> Vec<u32> {
        let mut out = vec![INF; cols];
        upward_search(ch, true, &sources[i], ws, |node, d| {
            let s =
                buckets.first[node as usize] as usize..buckets.first[node as usize + 1] as usize;
            for &(j, dt) in &buckets.entries[s] {
                let c = d.saturating_add(dt);
                if c < out[j as usize] {
                    out[j as usize] = c;
                }
            }
        });
        out
    };
    let rows: Vec<Vec<u32>> = if parallel {
        (0..sources.len())
            .into_par_iter()
            .map_init(|| UpWorkspace::new(n), |ws, i| row(i, ws))
            .collect()
    } else {
        let mut ws = UpWorkspace::new(n);
        (0..sources.len()).map(|i| row(i, &mut ws)).collect()
    };
    Matrix {
        rows: sources.len(),
        cols,
        data: rows.into_iter().flatten().collect(),
    }
}

/// Đối chứng: chạy lặp `N × M` truy vấn CH hai chiều.
pub fn repeated_matrix(
    ch: &Ch,
    sources: &[Vec<Leg>],
    targets: &[Vec<Leg>],
    parallel: bool,
) -> Matrix {
    let n = ch.node_count();
    let cols = targets.len();
    let one = |i: usize, ws: &mut ChWorkspace| -> Vec<u32> {
        (0..cols)
            .map(|j| {
                ch.query_legs(ws, &sources[i], &targets[j])
                    .map_or(INF, |r| r.cost)
            })
            .collect()
    };
    let rows: Vec<Vec<u32>> = if parallel {
        (0..sources.len())
            .into_par_iter()
            .map_init(|| ChWorkspace::new(n), |ws, i| one(i, ws))
            .collect()
    } else {
        let mut ws = ChWorkspace::new(n);
        (0..sources.len()).map(|i| one(i, &mut ws)).collect()
    };
    Matrix {
        rows: sources.len(),
        cols,
        data: rows.into_iter().flatten().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::{Rev, View, sssp};
    use crate::profile::{Costs, Metric, Profile};
    use road_graph::fixture::{compile_fixture, generate};

    #[test]
    fn bucket_matrix_equals_repeated_queries_and_dijkstra() {
        let g = compile_fixture(&generate());
        let rev = Rev::build(&g);
        let c = Costs::build(&g, Profile::Car, Metric::Time);
        let view = View::new(&g, &rev, &c.w);
        let ch = Ch::build(view);
        let n = g.node_count() as u32;
        let leg = |v: u32| vec![Leg { node: v, cost: 0 }];
        let srcs: Vec<Vec<Leg>> = (0..n).step_by(23).map(leg).collect();
        let tgts: Vec<Vec<Leg>> = (3..n).step_by(19).map(leg).collect();
        let a = bucket_matrix(&ch, &srcs, &tgts, false);
        let b = bucket_matrix(&ch, &srcs, &tgts, true);
        let r = repeated_matrix(&ch, &srcs, &tgts, false);
        assert_eq!(a, b);
        assert_eq!(a, r);
        for (i, s) in (0..n).step_by(23).enumerate() {
            let truth = sssp(view, s, true);
            for (j, t) in (3..n).step_by(19).enumerate() {
                assert_eq!(a.get(i, j), truth[t as usize], "{s}->{t}");
            }
        }
    }

    #[test]
    fn legs_with_offsets_add_to_the_cost() {
        let g = compile_fixture(&generate());
        let rev = Rev::build(&g);
        let c = Costs::build(&g, Profile::Car, Metric::Time);
        let view = View::new(&g, &rev, &c.w);
        let ch = Ch::build(view);
        let base = bucket_matrix(
            &ch,
            &[vec![Leg { node: 0, cost: 0 }]],
            &[vec![Leg { node: 50, cost: 0 }]],
            false,
        );
        let off = bucket_matrix(
            &ch,
            &[vec![Leg {
                node: 0,
                cost: 1000,
            }]],
            &[vec![Leg {
                node: 50,
                cost: 250,
            }]],
            false,
        );
        assert_eq!(off.get(0, 0), base.get(0, 0) + 1250);
    }
}
