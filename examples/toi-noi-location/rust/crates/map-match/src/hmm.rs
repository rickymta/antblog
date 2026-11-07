//! Bám đường bằng HMM và Viterbi, theo mô hình của Newson & Krumm: trạng thái ẩn là «xe đang ở điểm nào của cạnh nào»,
//! xác suất phát đo theo khoảng cách điểm GPS tới cạnh, xác suất chuyển đo theo chênh lệch giữa quãng đường đi trên mạng
//! đường và khoảng cách chim bay giữa hai điểm GPS liên tiếp.
//!
//! Chọn của lab (không phải của bài báo): trạng thái là **cạnh có hướng** (cạnh và `twin` là hai trạng thái, nên hướng đi
//! cũng được suy ra từ chuỗi điểm), tham số đo theo accuracy báo cáo của từng điểm, thêm ràng buộc vận tốc tối đa, và
//! xử lý «gãy chuỗi» khi không còn đường nối. Bỏ qua cấm rẽ (xem `route`).

use crate::route::{INF, Router};
use crate::{NoiseModel, Obs};
use geo_core::{LatLng, bearing_diff_deg, haversine_m};
use road_graph::snap::SnapIndex;
use road_graph::{EdgeId, Graph, Mode, NodeId};

pub const NO_BACK: u32 = u32::MAX;

#[derive(Debug, Clone)]
pub struct MatchConfig {
    pub mode: Mode,
    /// Bán kính tìm ứng viên quanh điểm GPS, mét.
    pub radius_m: f64,
    /// Số cạnh vật lý gần nhất giữ làm ứng viên (mỗi cạnh hai chiều thành tối đa hai trạng thái).
    pub max_candidates: usize,
    pub noise: NoiseModel,
    /// Thang xác suất chuyển: `β = beta_m + beta_frac · khoảng_cách_chim_bay`.
    pub beta_m: f64,
    pub beta_frac: f64,
    /// Vận tốc tối đa khả dĩ; đường đi dài hơn `max_speed·dt + slack` bị loại.
    pub max_speed_mps: f64,
    pub slack_m: f64,
    /// Đi lùi trên cùng một cạnh nhỏ hơn mức này được coi là nhiễu (quãng đường đi tính 0), mét.
    pub backward_tol_m: f64,
    /// Nhưng mỗi mét đi lùi bị phạt `1/backward_beta_m` nat, để «đi ngược chiều cạnh» không rẻ hơn đổi sang cạnh `twin`.
    pub backward_beta_m: f64,
    /// Cách nhau quá số giây này thì không nối (gãy chuỗi, khởi động lại).
    pub gap_break_s: f64,
    /// Ngân sách CPU: số nút tối đa mỗi lần Dijkstra.
    pub max_settled: usize,
    /// Có thì thêm số hạng hướng đi (khi điểm có heading và đang chạy), độ lệch chuẩn tính bằng độ.
    pub heading_sigma_deg: Option<f64>,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            mode: Mode::Car,
            radius_m: 50.0,
            max_candidates: 8,
            // beta_m, beta_frac, accuracy_to_sigma: chọn trên bộ seed chọn tham số của lab LOC-16, không phải bộ báo cáo
            noise: NoiseModel::default(),
            beta_m: 1.0,
            beta_frac: 0.0,
            max_speed_mps: 38.0,
            slack_m: 40.0,
            backward_tol_m: 25.0,
            backward_beta_m: 4.0,
            gap_break_s: 90.0,
            max_settled: 4000,
            heading_sigma_deg: None,
        }
    }
}

/// Kết quả bám đường cho một điểm GPS.
#[derive(Debug, Clone, Copy)]
pub struct MatchedPoint {
    /// Cạnh **có hướng**: hướng xe đi theo suy luận.
    pub edge: EdgeId,
    /// Mét từ nút đầu của `edge`.
    pub offset_m: f32,
    /// Hình chiếu của điểm GPS lên cạnh.
    pub pos: LatLng,
    /// Khoảng cách từ điểm GPS tới hình chiếu.
    pub dist_m: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct State {
    pub m: MatchedPoint,
    pub len_m: f32,
    pub score: f64,
    pub back: u32,
}

impl State {
    pub(crate) fn alive(&self) -> bool {
        self.score.is_finite()
    }
}

/// Một lớp của lưới Viterbi: các trạng thái khả dĩ của một điểm GPS.
#[derive(Debug, Clone)]
pub struct Layer {
    pub seq: u64,
    pub obs: Obs,
    pub(crate) states: Vec<State>,
    /// Lớp này mở đầu một đoạn chuỗi mới (không nối được với lớp trước).
    pub broke: bool,
}

impl Layer {
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
    pub(crate) fn best(&self) -> Option<usize> {
        let mut best: Option<usize> = None;
        for (i, s) in self.states.iter().enumerate() {
            if s.alive() && best.is_none_or(|b| s.score > self.states[b].score) {
                best = Some(i);
            }
        }
        best
    }
    pub fn mem_bytes(&self) -> usize {
        size_of::<Self>() + self.states.capacity() * size_of::<State>()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Stats {
    pub points: u64,
    pub candidates: u64,
    pub dijkstras: u64,
    pub breaks: u64,
}

#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Một phần tử mỗi điểm GPS đầu vào; `None` nếu không có cạnh nào trong bán kính.
    pub points: Vec<Option<MatchedPoint>>,
    /// Số lần gãy chuỗi (không có đường nối hợp lệ giữa hai điểm liên tiếp).
    pub breaks: usize,
    /// Lộ trình suy ra: các cạnh có hướng, nối bằng đường ngắn nhất giữa hai điểm liên tiếp.
    pub route: Vec<EdgeId>,
}

pub struct MatchCtx<'a> {
    pub g: &'a Graph,
    pub idx: &'a SnapIndex,
    pub cfg: MatchConfig,
    pub router: Router,
    pub stats: Stats,
}

impl<'a> MatchCtx<'a> {
    pub fn new(g: &'a Graph, idx: &'a SnapIndex, cfg: MatchConfig) -> Self {
        Self {
            g,
            idx,
            cfg,
            router: Router::new(g),
            stats: Stats::default(),
        }
    }

    /// Ứng viên cho một điểm: cạnh có hướng, điểm chiếu, và điểm số phát (log, bỏ hằng số).
    fn states_for(&self, obs: &Obs) -> Vec<State> {
        let g = self.g;
        let bit = self.cfg.mode.bit();
        let sigma = self.cfg.noise.sigma_m(obs.accuracy_m);
        let cands = self.idx.candidates(
            g,
            obs.pos,
            self.cfg.radius_m,
            self.cfg.mode,
            self.cfg.max_candidates,
        );
        let use_heading = self.cfg.heading_sigma_deg.is_some()
            && obs.heading_deg.is_some()
            && obs.speed_mps.is_none_or(|s| s > 1.5);
        let mut out = Vec::with_capacity(cands.len() * 2);
        for c in cands {
            let len_e = f64::from(g.length_cm[c.edge as usize]) / 100.0;
            let a_rep = c.along_m.min(len_e).max(0.0);
            let mut push = |edge: EdgeId, offset: f64, bearing: f64| {
                let len = f64::from(g.length_cm[edge as usize]) / 100.0;
                let mut score = -0.5 * (c.dist_m / sigma).powi(2);
                if use_heading
                    && let (Some(h), Some(hs)) = (obs.heading_deg, self.cfg.heading_sigma_deg)
                {
                    score -= 0.5 * (bearing_diff_deg(f64::from(h), bearing) / hs).powi(2);
                }
                out.push(State {
                    m: MatchedPoint {
                        edge,
                        offset_m: offset.clamp(0.0, len) as f32,
                        pos: c.point,
                        dist_m: c.dist_m as f32,
                    },
                    len_m: len as f32,
                    score,
                    back: NO_BACK,
                });
            };
            if g.modes[c.edge as usize] & bit != 0 {
                push(c.edge, a_rep, c.bearing_deg);
            }
            if let Some(t) = c.twin
                && g.modes[t as usize] & bit != 0
            {
                let len_t = f64::from(g.length_cm[t as usize]) / 100.0;
                let frac = if len_e > 0.0 { a_rep / len_e } else { 0.0 };
                push(t, len_t * (1.0 - frac), (c.bearing_deg + 180.0) % 360.0);
            }
        }
        out
    }

    /// Tạo lớp cho `obs`, nối với lớp trước `prev` (nếu có). Không đổi `prev`.
    pub fn extend(&mut self, prev: Option<&Layer>, obs: Obs, seq: u64) -> Layer {
        let mut states = self.states_for(&obs);
        self.stats.points += 1;
        self.stats.candidates += states.len() as u64;
        let mut broke = false;
        if !states.is_empty()
            && let Some(p) = prev.filter(|p| p.states.iter().any(State::alive))
        {
            let dt = ((obs.t_ms - p.obs.t_ms) as f64 / 1000.0).max(0.2);
            if dt > self.cfg.gap_break_s || !self.link(p, &obs, &mut states, dt) {
                broke = true;
                self.stats.breaks += 1;
                for s in &mut states {
                    s.back = NO_BACK;
                }
            }
        }
        if let Some(m) = states
            .iter()
            .filter(|s| s.alive())
            .map(|s| s.score)
            .reduce(f64::max)
        {
            for s in &mut states {
                s.score -= m; // chuẩn hóa để điểm số không trôi về -∞ trên vết dài
            }
        }
        Layer {
            seq,
            obs,
            states,
            broke,
        }
    }

    /// Viterbi một bước: điểm số mới = max(điểm lớp trước + log xác suất chuyển) + log xác suất phát (đã nằm sẵn trong `score`).
    fn link(&mut self, p: &Layer, obs: &Obs, states: &mut [State], dt: f64) -> bool {
        let g = self.g;
        let cfg = &self.cfg;
        let straight = haversine_m(p.obs.pos, obs.pos);
        let beta = cfg.beta_m + cfg.beta_frac * straight;
        let reach = cfg.max_speed_mps * dt + cfg.slack_m;
        let limit_m = (straight + 6.0 * beta + cfg.slack_m).min(reach);
        let limit_cm = (limit_m * 100.0).min(f64::from(u32::MAX / 2)) as u32;
        let targets: Vec<NodeId> = states.iter().map(|s| g.tail[s.m.edge as usize]).collect();
        let mut rows: Vec<(NodeId, Vec<u32>)> = Vec::new();
        let mut best: Vec<(f64, u32)> = vec![(f64::NEG_INFINITY, NO_BACK); states.len()];
        for (i, ps) in p.states.iter().enumerate() {
            if !ps.alive() {
                continue;
            }
            let e1 = ps.m.edge;
            let h1 = g.head[e1 as usize];
            let ri = match rows.iter().position(|(n, _)| *n == h1) {
                Some(ri) => ri,
                None => {
                    let mut out = vec![INF; targets.len()];
                    self.stats.dijkstras += 1;
                    self.router.distances_to(
                        g,
                        cfg.mode,
                        h1,
                        &targets,
                        limit_cm,
                        cfg.max_settled,
                        &mut out,
                    );
                    rows.push((h1, out));
                    rows.len() - 1
                }
            };
            let row = &rows[ri].1;
            let a1 = f64::from(ps.m.offset_m);
            for (j, s) in states.iter().enumerate() {
                let a2 = f64::from(s.m.offset_m);
                let route = if s.m.edge == e1 && a2 + cfg.backward_tol_m >= a1 {
                    (a2 - a1).max(0.0)
                } else {
                    if row[j] == INF {
                        continue;
                    }
                    (f64::from(ps.len_m) - a1) + f64::from(row[j]) / 100.0 + a2
                };
                if route > reach {
                    continue;
                }
                let back = if s.m.edge == e1 {
                    (a1 - a2).max(0.0)
                } else {
                    0.0
                };
                let sc = ps.score - (route - straight).abs() / beta - back / cfg.backward_beta_m;
                if sc > best[j].0 {
                    best[j] = (sc, i as u32);
                }
            }
        }
        let mut any = false;
        for (s, (sc, back)) in states.iter_mut().zip(best) {
            if sc.is_finite() {
                s.score += sc;
                s.back = back;
                any = true;
            } else {
                s.score = f64::NEG_INFINITY;
            }
        }
        any
    }

    /// Bám đường offline: Viterbi trên toàn vết rồi truy ngược.
    pub fn match_offline(&mut self, obs: &[Obs]) -> MatchResult {
        let mut layers: Vec<Layer> = Vec::with_capacity(obs.len());
        let mut nonempty: Vec<usize> = Vec::new();
        for (i, o) in obs.iter().enumerate() {
            let layer = {
                let prev = nonempty.last().map(|&k| &layers[k]);
                self.extend(prev, *o, i as u64)
            };
            if !layer.is_empty() {
                nonempty.push(layers.len());
            }
            layers.push(layer);
        }
        let mut points: Vec<Option<MatchedPoint>> = vec![None; obs.len()];
        let mut seg_of: Vec<u32> = vec![0; obs.len()];
        let mut breaks = 0usize;
        // đoạn chuỗi kết thúc ở lớp không rỗng ngay trước lớp `broke`, và ở lớp không rỗng cuối cùng
        let mut seg_end: Vec<usize> = Vec::new(); // vị trí trong `nonempty`
        for (pos, &li) in nonempty.iter().enumerate() {
            if layers[li].broke && pos > 0 {
                seg_end.push(pos - 1);
                breaks += 1;
            }
        }
        if !nonempty.is_empty() {
            seg_end.push(nonempty.len() - 1);
        }
        for (si, &end) in seg_end.iter().enumerate() {
            let mut pos = end;
            let Some(mut s) = layers[nonempty[pos]].best() else {
                continue;
            };
            loop {
                let l = &layers[nonempty[pos]];
                points[l.seq as usize] = Some(l.states[s].m);
                seg_of[l.seq as usize] = si as u32;
                let b = l.states[s].back;
                if b == NO_BACK || pos == 0 {
                    break;
                }
                pos -= 1;
                s = b as usize;
            }
        }
        let route = self.route_of(&points, &seg_of);
        MatchResult {
            points,
            breaks,
            route,
        }
    }

    /// Các cạnh có hướng giữa hai điểm đã bám (gồm cả hai cạnh đầu cuối), nối bằng đường ngắn nhất.
    pub fn path_between(&mut self, a: &MatchedPoint, b: &MatchedPoint) -> Vec<EdgeId> {
        if a.edge == b.edge && b.offset_m + self.cfg.backward_tol_m as f32 >= a.offset_m {
            return vec![a.edge];
        }
        let (h, t) = (self.g.head[a.edge as usize], self.g.tail[b.edge as usize]);
        let mut v = vec![a.edge];
        if h != t {
            // không nối được thì chỉ giữ hai cạnh đầu cuối
            if let Some((edges, _)) = self.router.shortest_path(self.g, self.cfg.mode, h, t) {
                v.extend(edges);
            }
        }
        v.push(b.edge);
        v
    }

    /// Lộ trình từ dãy điểm đã bám (coi như một đoạn liền): dùng cho kết quả online, nơi không có ranh giới đoạn.
    pub fn reconstruct_route(&mut self, points: &[Option<MatchedPoint>]) -> Vec<EdgeId> {
        let seg = vec![0u32; points.len()];
        self.route_of(points, &seg)
    }

    fn route_of(&mut self, points: &[Option<MatchedPoint>], seg_of: &[u32]) -> Vec<EdgeId> {
        let mut route: Vec<EdgeId> = Vec::new();
        let mut prev: Option<(MatchedPoint, u32)> = None;
        for (i, p) in points.iter().enumerate() {
            let Some(p) = p else { continue };
            let leg = match prev {
                Some((q, sq)) if sq == seg_of[i] => self.path_between(&q, p),
                _ => vec![p.edge],
            };
            for e in leg {
                if route.last() != Some(&e) {
                    route.push(e);
                }
            }
            prev = Some((*p, seg_of[i]));
        }
        route
    }

    /// Nội suy vị trí trong khoảng mất tín hiệu: đi dọc đường ngắn nhất giữa hai điểm đã bám, đều theo thời gian.
    /// `fracs` là tỉ lệ thời gian trong khoảng `[0,1]` giữa điểm trước và điểm sau.
    pub fn interpolate_gap(
        &mut self,
        a: &MatchedPoint,
        b: &MatchedPoint,
        fracs: &[f64],
    ) -> Vec<LatLng> {
        let edges = self.path_between(a, b);
        let mut pts: Vec<LatLng> = Vec::new();
        let mut before_last = 0.0;
        for (k, &e) in edges.iter().enumerate() {
            let ep = self.g.edge_points(e);
            if k + 1 == edges.len() {
                before_last = geo_core::segment::polyline_length_m(&pts);
            }
            for p in ep {
                if pts.last() != Some(&p) {
                    pts.push(p);
                }
            }
        }
        let start = f64::from(a.offset_m);
        let end = if edges.len() == 1 {
            f64::from(b.offset_m)
        } else {
            before_last + f64::from(b.offset_m)
        };
        fracs
            .iter()
            .filter_map(|&f| {
                geo_core::segment::point_at(&pts, start + (end - start) * f.clamp(0.0, 1.0))
            })
            .collect()
    }
}
