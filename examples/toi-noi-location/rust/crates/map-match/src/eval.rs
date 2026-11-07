//! Chấm điểm bám đường so với ground truth của vết tổng hợp, và baseline «snap về cạnh gần nhất» để so sánh.
//!
//! Chỉ số:
//! - **cạnh (chặt)**: cạnh được gán trùng cạnh thật hoặc cạnh `twin` của nó (cùng một đoạn đường vật lý, bỏ qua chiều);
//! - **cạnh (nới nút)**: như trên, hoặc cạnh kề (chung nút) mà điểm chiếu cách vị trí thật không quá [`JUNCTION_TOL_M`]. Cạnh đồ thị
//!   HCM rất ngắn (nhiều cạnh 5–40 m) nên điểm thật nằm cách nút vài mét bị gán sang cạnh kề là chuyện hợp lý, không phải lỗi bám đường;
//! - **chiều (nới nút)**: như «nới nút» nhưng phải đúng chiều đi (cạnh có hướng trùng hoặc nối tiếp theo chiều xe chạy).
//!   Điểm không có ứng viên tính là sai ở cả ba;
//! - **lộ trình** (kiểu Newson–Krumm): độ dài các cạnh thật mà lộ trình suy ra có (`recall`), độ dài cạnh thừa (`extra`),
//!   `RMF = (thiếu + thừa) / độ dài thật`. Cạnh `twin` được coi là cùng một cạnh.
//!
//! Ranh giới nút giao: điểm thật nằm sát nút có thể bị gán sang cạnh kề; ai cũng chịu như nhau và không được nới ngưỡng.

use crate::hmm::MatchedPoint;
use crate::trace::Trace;
use geo_core::{LatLng, haversine_m};
use road_graph::snap::SnapIndex;
use road_graph::{EdgeId, Graph, Mode, NONE};
use rustc_hash::FxHashSet;

/// Ngưỡng khoảng cách (mét) cho phép nới ở nút giao khi chấm «cạnh (nới nút)».
pub const JUNCTION_TOL_M: f64 = 12.0;

pub fn canon(g: &Graph, e: EdgeId) -> EdgeId {
    let t = g.twin[e as usize];
    if t == NONE { e } else { e.min(t) }
}

#[derive(Debug, Clone, Default)]
pub struct Score {
    pub n: usize,
    pub edge_ok: usize,
    pub dir_ok: usize,
    pub edge_near: usize,
    pub dir_near: usize,
    pub unmatched: usize,
    /// Tổng khoảng cách (mét) từ điểm hình chiếu đã bám tới vị trí thật, chỉ tính điểm có kết quả.
    pub pos_err_sum: f64,
    pub pos_err_n: usize,
    pub route_true_m: f64,
    pub route_common_m: f64,
    pub route_extra_m: f64,
}

impl Score {
    pub fn add(&mut self, o: &Score) {
        self.n += o.n;
        self.edge_ok += o.edge_ok;
        self.dir_ok += o.dir_ok;
        self.edge_near += o.edge_near;
        self.dir_near += o.dir_near;
        self.unmatched += o.unmatched;
        self.pos_err_sum += o.pos_err_sum;
        self.pos_err_n += o.pos_err_n;
        self.route_true_m += o.route_true_m;
        self.route_common_m += o.route_common_m;
        self.route_extra_m += o.route_extra_m;
    }
    pub fn edge_pct(&self) -> f64 {
        100.0 * self.edge_ok as f64 / self.n.max(1) as f64
    }
    pub fn dir_pct(&self) -> f64 {
        100.0 * self.dir_ok as f64 / self.n.max(1) as f64
    }
    pub fn edge_near_pct(&self) -> f64 {
        100.0 * self.edge_near as f64 / self.n.max(1) as f64
    }
    pub fn dir_near_pct(&self) -> f64 {
        100.0 * self.dir_near as f64 / self.n.max(1) as f64
    }
    pub fn recall_pct(&self) -> f64 {
        100.0 * self.route_common_m / self.route_true_m.max(1.0)
    }
    pub fn rmf(&self) -> f64 {
        ((self.route_true_m - self.route_common_m) + self.route_extra_m)
            / self.route_true_m.max(1.0)
    }
    pub fn mean_pos_err(&self) -> f64 {
        self.pos_err_sum / self.pos_err_n.max(1) as f64
    }
}

/// Độ dài (mét) cạnh vật lý đã chuẩn hóa twin.
fn edge_set(g: &Graph, route: &[EdgeId]) -> FxHashSet<EdgeId> {
    route.iter().map(|&e| canon(g, e)).collect()
}

/// Chấm điểm từng điểm (`focus` giới hạn ở các điểm có vị trí thật trong bán kính) và lộ trình toàn vết.
pub fn score_trace(
    g: &Graph,
    tr: &Trace,
    matched: &[Option<MatchedPoint>],
    matched_route: &[EdgeId],
    focus: Option<(LatLng, f64)>,
) -> Score {
    let mut s = Score::default();
    for (i, m) in matched.iter().enumerate() {
        let t = tr.truth_of(i);
        if let Some((c, r)) = focus
            && haversine_m(t.pos, c) > r
        {
            continue;
        }
        s.n += 1;
        match m {
            None => s.unmatched += 1,
            Some(m) => {
                if canon(g, m.edge) == canon(g, t.edge) {
                    s.edge_ok += 1;
                }
                if m.edge == t.edge {
                    s.dir_ok += 1;
                }
                let near = haversine_m(m.pos, t.pos) <= JUNCTION_TOL_M;
                let (mt, mh, tt, th) = (
                    g.tail[m.edge as usize],
                    g.head[m.edge as usize],
                    g.tail[t.edge as usize],
                    g.head[t.edge as usize],
                );
                let shares = mt == tt || mt == th || mh == tt || mh == th;
                let consecutive = mt == th || mh == tt;
                if canon(g, m.edge) == canon(g, t.edge) || (near && shares) {
                    s.edge_near += 1;
                }
                if m.edge == t.edge || (near && consecutive) {
                    s.dir_near += 1;
                }
                s.pos_err_sum += haversine_m(m.pos, t.pos);
                s.pos_err_n += 1;
            }
        }
    }
    let (truth, got) = (edge_set(g, &tr.route), edge_set(g, matched_route));
    let len = |e: EdgeId| f64::from(g.length_cm[e as usize]) / 100.0;
    s.route_true_m = truth.iter().map(|&e| len(e)).sum();
    s.route_common_m = truth
        .iter()
        .filter(|e| got.contains(e))
        .map(|&e| len(e))
        .sum();
    s.route_extra_m = got
        .iter()
        .filter(|e| !truth.contains(e))
        .map(|&e| len(e))
        .sum();
    s
}

/// Baseline ngây thơ: mỗi điểm snap về cạnh gần nhất đi được bằng `mode`; lộ trình là dãy cạnh không lặp liên tiếp.
pub fn nearest_snap(
    g: &Graph,
    idx: &SnapIndex,
    mode: Mode,
    obs: &[crate::Obs],
) -> (Vec<Option<MatchedPoint>>, Vec<EdgeId>) {
    let bit = mode.bit();
    let mut pts = Vec::with_capacity(obs.len());
    let mut route: Vec<EdgeId> = Vec::new();
    for o in obs {
        let m = idx.nearest(g, o.pos, mode).map(|c| {
            let edge = if g.modes[c.edge as usize] & bit != 0 {
                c.edge
            } else {
                c.twin.unwrap_or(c.edge)
            };
            MatchedPoint {
                edge,
                offset_m: c.along_m as f32,
                pos: c.point,
                dist_m: c.dist_m as f32,
            }
        });
        if let Some(m) = &m
            && route.last() != Some(&m.edge)
        {
            route.push(m.edge);
        }
        pts.push(m);
    }
    (pts, route)
}

/// Sai số vị trí của chính điểm GPS thô so với vị trí thật (để so với sau lọc/bám đường).
pub fn raw_error_m(tr: &Trace, i: usize) -> f64 {
    haversine_m(tr.obs[i].pos, tr.truth_of(i).pos)
}
