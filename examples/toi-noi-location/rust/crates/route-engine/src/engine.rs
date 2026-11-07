//! `Engine`: gom đồ thị, chỉ mục snap, trọng số ba profile, CH (dựng lười) và hồ sơ theo giờ thành một đối tượng dùng chung
//! giữa các luồng (`Send + Sync`). Dịch vụ gRPC (LOC-29) và simulator (LOC-30) đều đi qua đây.

use crate::cch::Cch;
use crate::ch::{Ch, ChWorkspace};
use crate::matrix::{Matrix, bucket_matrix};
use crate::net::{INF, Path, Rev, View};
use crate::position::{Leg, Position, Snapped};
use crate::profile::{Costs, Metric, Profile};
use crate::search::{NodeXy, Searcher};
use crate::traffic::{SLOTS, TrafficModel};
use crate::turn::{EdgeGeom, Restrictions, TurnCosts, TurnModel};
use geo_core::{LatLng, haversine_m};
use rayon::prelude::*;
use road_graph::snap::SnapIndex;
use road_graph::{EdgeId, Graph, NodeId};
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Điểm GPS cách đường (của profile) xa hơn ngưỡng này thì từ chối.
    pub max_snap_m: f64,
    /// Thư mục lưu CH đã dựng để các lần chạy sau nạp lại (bỏ trống = luôn dựng).
    pub ch_cache_dir: Option<PathBuf>,
    /// Nhãn đồ thị dùng trong tên file cache.
    pub cache_tag: String,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_snap_m: 250.0,
            ch_cache_dir: None,
            cache_tag: "graph".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RouteError {
    #[error(
        "không snap được {which} lên đường của profile (cách {dist_m:.0} m, ngưỡng {limit_m:.0} m)"
    )]
    TooFarFromRoad {
        which: &'static str,
        dist_m: f64,
        limit_m: f64,
    },
    #[error("không có đường cho profile này quanh {which}")]
    NoRoadNearby { which: &'static str },
    #[error("không có đường đi giữa hai điểm")]
    NoRoute,
}

/// Trọng số của một profile ở một thời điểm kèm CH tương ứng (dựng lười).
pub struct Routing {
    pub costs: Costs,
    pub ch: OnceLock<Ch>,
}

impl Routing {
    fn new(costs: Costs) -> Self {
        Self {
            costs,
            ch: OnceLock::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouteResult {
    pub duration_ms: u32,
    pub distance_m: f64,
    pub edges: Vec<EdgeId>,
    pub start_frac: f32,
    pub end_frac: f32,
    pub points: Vec<LatLng>,
    pub algo: &'static str,
    pub from: Snapped,
    pub to: Snapped,
}

#[derive(Debug, Clone)]
pub struct MatrixResult {
    pub matrix: Matrix,
    /// Số nguồn/đích bị từ chối vì không snap được (hàng/cột toàn INF).
    pub unsnapped_sources: Vec<usize>,
    pub unsnapped_targets: Vec<usize>,
}

struct TimeOfDay {
    model: TrafficModel,
    cch: Cch,
    profile: Profile,
    recent: Mutex<VecDeque<(usize, Arc<Routing>)>>,
}

pub struct Engine {
    pub g: Graph,
    pub rev: Rev,
    pub xy: NodeXy,
    pub snap: SnapIndex,
    pub geom: EdgeGeom,
    pub restrictions: [Restrictions; 3],
    base: [Arc<Routing>; 3],
    pub cfg: EngineConfig,
    tod: Option<TimeOfDay>,
}

thread_local! {
    static CH_WS: RefCell<Option<ChWorkspace>> = const { RefCell::new(None) };
    static SEARCH_WS: RefCell<Option<Searcher>> = const { RefCell::new(None) };
}

fn with_ch_ws<R>(n: usize, f: impl FnOnce(&mut ChWorkspace) -> R) -> R {
    CH_WS.with(|c| {
        let mut b = c.borrow_mut();
        if b.as_ref().is_none_or(|w| w.f.len() != n) {
            *b = Some(ChWorkspace::new(n));
        }
        f(b.as_mut().expect("vừa tạo"))
    })
}

fn with_searcher<R>(n: usize, f: impl FnOnce(&mut Searcher) -> R) -> R {
    SEARCH_WS.with(|c| {
        let mut b = c.borrow_mut();
        if b.as_ref().is_none_or(|w| w.labels.len() != n) {
            *b = Some(Searcher::new(n));
        }
        f(b.as_mut().expect("vừa tạo"))
    })
}

/// Cắt polyline của cạnh `e` từ tỉ lệ `a` tới `b`.
pub fn slice_edge(g: &Graph, e: EdgeId, a: f32, b: f32) -> Vec<LatLng> {
    let pts = g.edge_points(e);
    let total: f64 = pts.windows(2).map(|w| haversine_m(w[0], w[1])).sum();
    let (da, db) = (total * f64::from(a), total * f64::from(b));
    let mut out = vec![geo_core::segment::point_at(&pts, da).unwrap_or(pts[0])];
    let mut acc = 0.0;
    for w in pts.windows(2) {
        acc += haversine_m(w[0], w[1]);
        if acc > da && acc < db {
            out.push(w[1]);
        }
    }
    out.push(geo_core::segment::point_at(&pts, db).unwrap_or(*pts.last().expect("có điểm")));
    out
}

fn cost_ms(w: u32, frac: f32) -> u32 {
    (f64::from(w) * f64::from(frac)).round() as u32
}

impl Engine {
    pub fn new(g: Graph, cfg: EngineConfig) -> Self {
        let rev = Rev::build(&g);
        let xy = NodeXy::build(&g);
        let snap = SnapIndex::build(&g);
        let geom = EdgeGeom::build(&g);
        let restrictions = Profile::ALL.map(|p| Restrictions::build(&g, p));
        let base = Profile::ALL.map(|p| Arc::new(Routing::new(Costs::build(&g, p, Metric::Time))));
        Self {
            g,
            rev,
            xy,
            snap,
            geom,
            restrictions,
            base,
            cfg,
            tod: None,
        }
    }

    /// Bật hồ sơ tốc độ theo giờ cho một profile: dựng CCH một lần, sau đó mỗi giờ chỉ tuỳ biến lại trọng số.
    pub fn enable_time_of_day(&mut self, profile: Profile, model: TrafficModel) {
        let view = self.view(profile);
        let cch = Cch::build(view, &self.xy);
        self.tod = Some(TimeOfDay {
            model,
            cch,
            profile,
            recent: Mutex::new(VecDeque::new()),
        });
    }

    pub fn view(&self, p: Profile) -> View<'_> {
        View::new(&self.g, &self.rev, &self.base[p.index()].costs.w)
    }
    pub fn costs(&self, p: Profile) -> &Costs {
        &self.base[p.index()].costs
    }
    pub fn turn_model(&self, p: Profile, costs: TurnCosts) -> TurnModel<'_> {
        TurnModel {
            view: self.view(p),
            geom: &self.geom,
            restr: &self.restrictions[p.index()],
            costs,
        }
    }

    fn fingerprint(&self, p: Profile) -> u64 {
        let c = self.costs(p);
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for x in [
            self.g.node_count() as u64,
            self.g.edge_count() as u64,
            c.w.iter()
                .map(|&w| u64::from(w))
                .fold(0u64, u64::wrapping_add),
            c.allowed_edges as u64,
        ] {
            h = (h ^ x).wrapping_mul(0x0100_0000_01b3);
        }
        h
    }

    /// CH tự do (không kẹt xe) của profile; dựng ở lần gọi đầu (hoặc nạp từ cache).
    pub fn ch(&self, p: Profile) -> &Ch {
        let r = &self.base[p.index()];
        r.ch.get_or_init(|| {
            let path = self.cfg.ch_cache_dir.as_ref().map(|d| {
                d.join(format!(
                    "{}-{}-{:016x}.ch",
                    self.cfg.cache_tag,
                    p.name(),
                    self.fingerprint(p)
                ))
            });
            if let Some(path) = &path
                && let Ok(ch) = Ch::load(path)
                && ch.node_count() == self.g.node_count()
            {
                return ch;
            }
            let ch = Ch::build(self.view(p));
            if let Some(path) = &path {
                if let Some(dir) = path.parent() {
                    let _ = std::fs::create_dir_all(dir);
                }
                let _ = ch.save(path);
            }
            ch
        })
    }

    /// Dữ liệu định tuyến cho profile ở giờ `slot` (nếu có hồ sơ theo giờ cho profile đó), nếu không là tự do.
    pub fn routing(&self, p: Profile, slot: Option<usize>) -> Arc<Routing> {
        if let (Some(tod), Some(slot)) = (&self.tod, slot)
            && tod.profile == p
        {
            let slot = slot % SLOTS;
            let mut recent = tod.recent.lock().expect("khoá không bị hỏng");
            if let Some((_, r)) = recent.iter().find(|(s, _)| *s == slot) {
                return Arc::clone(r);
            }
            let costs = tod.model.costs(&self.g, p, slot);
            let ch = tod.cch.customize(&costs.w);
            let r = Arc::new(Routing {
                costs,
                ch: OnceLock::new(),
            });
            let _ = r.ch.set(ch);
            recent.push_back((slot, Arc::clone(&r)));
            while recent.len() > 4 {
                recent.pop_front();
            }
            return r;
        }
        Arc::clone(&self.base[p.index()])
    }

    pub fn time_of_day_enabled(&self) -> Option<Profile> {
        self.tod.as_ref().map(|t| t.profile)
    }

    pub fn snap_point(
        &self,
        p: Profile,
        ll: LatLng,
        which: &'static str,
    ) -> Result<Snapped, RouteError> {
        let Some(c) = self.snap.nearest(&self.g, ll, p.mode()) else {
            return Err(RouteError::NoRoadNearby { which });
        };
        if c.dist_m > self.cfg.max_snap_m {
            return Err(RouteError::TooFarFromRoad {
                which,
                dist_m: c.dist_m,
                limit_m: self.cfg.max_snap_m,
            });
        }
        Ok(Snapped::from_candidate(&self.g, &c))
    }

    pub fn route(
        &self,
        p: Profile,
        from: LatLng,
        to: LatLng,
        slot: Option<usize>,
    ) -> Result<RouteResult, RouteError> {
        let a = self.snap_point(p, from, "điểm đi")?;
        let b = self.snap_point(p, to, "điểm đến")?;
        self.route_snapped(p, &a, &b, slot)
    }

    pub fn route_snapped(
        &self,
        p: Profile,
        a: &Snapped,
        b: &Snapped,
        slot: Option<usize>,
    ) -> Result<RouteResult, RouteError> {
        let routing = self.routing(p, slot);
        let view = View::new(&self.g, &self.rev, &routing.costs.w);
        let (from_pos, to_pos) = (a.positions(view), b.positions(view));
        if from_pos.is_empty() {
            return Err(RouteError::NoRoadNearby {
                which: "điểm đi"
            });
        }
        if to_pos.is_empty() {
            return Err(RouteError::NoRoadNearby {
                which: "điểm đến"
            });
        }
        // cùng một cạnh có hướng, đích ở phía trước: đi thẳng
        let mut best: Option<(u32, Vec<EdgeId>, f32, f32, &'static str)> = None;
        for fp in &from_pos {
            for tp in &to_pos {
                if fp.edge == tp.edge && tp.frac >= fp.frac {
                    let c = cost_ms(view.w[fp.edge as usize], tp.frac - fp.frac);
                    if best.as_ref().is_none_or(|b| c < b.0) {
                        best = Some((c, vec![fp.edge], fp.frac, tp.frac, "same-edge"));
                    }
                }
            }
        }
        let src: Vec<Leg> = from_pos.iter().flat_map(|q| q.source_legs(view)).collect();
        let dst: Vec<Leg> = to_pos.iter().flat_map(|q| q.target_legs(view)).collect();
        let n = self.g.node_count();
        let routed = if p == Profile::Foot {
            self.route_legs_astar(view, routing.costs.vmax_cm_per_unit, &src, &dst)
        } else {
            let ch = routing.ch.get().unwrap_or_else(|| self.ch(p));
            with_ch_ws(n, |ws| {
                ch.query_legs(ws, &src, &dst).map(|r| {
                    let (edges, s, t) = ch.unpack(ws, &r);
                    (r.cost, edges, s, t)
                })
            })
            .map(|(c, e, s, t)| (c, e, s, t, "ch"))
        };
        if let Some((cost, mid, s_node, t_node, algo)) = routed
            && best.as_ref().is_none_or(|b| cost < b.0)
        {
            let fp = from_pos
                .iter()
                .find(|q| self.g.head[q.edge as usize] == s_node)
                .copied()
                .unwrap_or(from_pos[0]);
            let tp = to_pos
                .iter()
                .find(|q| self.g.tail[q.edge as usize] == t_node)
                .copied()
                .unwrap_or(to_pos[0]);
            let mut edges = Vec::with_capacity(mid.len() + 2);
            edges.push(fp.edge);
            edges.extend(mid);
            edges.push(tp.edge);
            best = Some((cost, edges, fp.frac, tp.frac, algo));
        }
        let Some((cost, edges, start_frac, end_frac, algo)) = best else {
            return Err(RouteError::NoRoute);
        };
        let points = self.polyline(&edges, start_frac, end_frac);
        let distance_m = self.distance_m(&edges, start_frac, end_frac);
        Ok(RouteResult {
            duration_ms: cost,
            distance_m,
            edges,
            start_frac,
            end_frac,
            points,
            algo,
            from: *a,
            to: *b,
        })
    }

    fn route_legs_astar(
        &self,
        view: View,
        vmax: f64,
        src: &[Leg],
        dst: &[Leg],
    ) -> Option<(u32, Vec<EdgeId>, NodeId, NodeId, &'static str)> {
        let n = self.g.node_count();
        let mut best: Option<(u32, Path, NodeId, NodeId)> = None;
        for s in src {
            for t in dst {
                let r = with_searcher(n, |se| se.astar(view, &self.xy, vmax, s.node, t.node).0);
                if let Some(path) = r {
                    let c = s.cost.saturating_add(path.cost).saturating_add(t.cost);
                    if best.as_ref().is_none_or(|b| c < b.0) {
                        best = Some((c, path, s.node, t.node));
                    }
                }
            }
        }
        best.map(|(c, p, s, t)| (c, p.edges, s, t, "astar"))
    }

    pub fn distance_m(&self, edges: &[EdgeId], start_frac: f32, end_frac: f32) -> f64 {
        let n = edges.len();
        let mut total = 0.0;
        for (i, &e) in edges.iter().enumerate() {
            let l = f64::from(self.g.length_cm[e as usize]) / 100.0;
            let a = if i == 0 { f64::from(start_frac) } else { 0.0 };
            let b = if i + 1 == n { f64::from(end_frac) } else { 1.0 };
            total += l * (b - a).max(0.0);
        }
        total
    }

    pub fn polyline(&self, edges: &[EdgeId], start_frac: f32, end_frac: f32) -> Vec<LatLng> {
        let n = edges.len();
        let mut out: Vec<LatLng> = Vec::new();
        for (i, &e) in edges.iter().enumerate() {
            let a = if i == 0 { start_frac } else { 0.0 };
            let b = if i + 1 == n { end_frac } else { 1.0 };
            let seg = slice_edge(&self.g, e, a, b);
            if out.is_empty() {
                out.extend(seg);
            } else {
                out.extend(seg.into_iter().skip(1));
            }
        }
        out
    }

    /// Ma trận thời gian (ms) từ mọi nguồn tới mọi đích. Điểm không snap được cho hàng/cột `INF` và được liệt kê.
    pub fn matrix(
        &self,
        p: Profile,
        from: &[LatLng],
        to: &[LatLng],
        slot: Option<usize>,
    ) -> MatrixResult {
        let routing = self.routing(p, slot);
        let view = View::new(&self.g, &self.rev, &routing.costs.w);
        let snap_all = |pts: &[LatLng], which: &'static str| -> Vec<Option<Snapped>> {
            pts.iter()
                .map(|&q| self.snap_point(p, q, which).ok())
                .collect()
        };
        let (ss, ts) = (snap_all(from, "nguồn"), snap_all(to, "đích"));
        let src_legs: Vec<Vec<Leg>> = ss
            .iter()
            .map(|s| s.map_or_else(Vec::new, |s| s.source_legs(view)))
            .collect();
        let dst_legs: Vec<Vec<Leg>> = ts
            .iter()
            .map(|s| s.map_or_else(Vec::new, |s| s.target_legs(view)))
            .collect();
        let mut matrix = if p == Profile::Foot {
            self.matrix_astar(view, routing.costs.vmax_cm_per_unit, &src_legs, &dst_legs)
        } else {
            let ch = routing.ch.get().unwrap_or_else(|| self.ch(p));
            bucket_matrix(ch, &src_legs, &dst_legs, true)
        };
        // cặp điểm cùng một cạnh có hướng và đích ở phía trước: đi thẳng, không phải vòng qua nút như các chân (legs) bắt buộc
        let mut targets_on_edge: FxHashMap<EdgeId, Vec<(usize, f32)>> = FxHashMap::default();
        for (j, t) in ts.iter().enumerate() {
            for q in t.map_or_else(Vec::new, |t| t.positions(view)) {
                targets_on_edge.entry(q.edge).or_default().push((j, q.frac));
            }
        }
        for (i, s) in ss.iter().enumerate() {
            for q in s.map_or_else(Vec::new, |s| s.positions(view)) {
                for &(j, ft) in targets_on_edge.get(&q.edge).map_or(&[][..], Vec::as_slice) {
                    if ft >= q.frac {
                        let c = cost_ms(view.w[q.edge as usize], ft - q.frac);
                        let cell = &mut matrix.data[i * dst_legs.len() + j];
                        *cell = (*cell).min(c);
                    }
                }
            }
        }
        MatrixResult {
            matrix,
            unsnapped_sources: ss
                .iter()
                .enumerate()
                .filter(|(_, s)| s.is_none())
                .map(|(i, _)| i)
                .collect(),
            unsnapped_targets: ts
                .iter()
                .enumerate()
                .filter(|(_, s)| s.is_none())
                .map(|(i, _)| i)
                .collect(),
        }
    }

    fn matrix_astar(&self, view: View, vmax: f64, src: &[Vec<Leg>], dst: &[Vec<Leg>]) -> Matrix {
        let n = self.g.node_count();
        let rows: Vec<Vec<u32>> = src
            .par_iter()
            .map(|s| {
                dst.iter()
                    .map(|t| {
                        let mut best = INF;
                        for a in s {
                            for b in t {
                                let r = with_searcher(n, |se| {
                                    se.astar(view, &self.xy, vmax, a.node, b.node).0
                                });
                                if let Some(p) = r {
                                    best = best
                                        .min(a.cost.saturating_add(p.cost).saturating_add(b.cost));
                                }
                            }
                        }
                        best
                    })
                    .collect()
            })
            .collect();
        Matrix {
            rows: src.len(),
            cols: dst.len(),
            data: rows.into_iter().flatten().collect(),
        }
    }

    /// Vị trí có hướng đầu tiên mà profile đi được tại điểm đã snap.
    pub fn first_position(&self, p: Profile, s: &Snapped) -> Option<Position> {
        s.positions(self.view(p)).first().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use road_graph::fixture::{compile_fixture, generate};

    fn engine() -> Engine {
        Engine::new(compile_fixture(&generate()), EngineConfig::default())
    }

    #[test]
    fn ch_and_astar_routes_agree_between_two_points() {
        let e = engine();
        let a = LatLng::try_new(10.7705, 106.6905).unwrap();
        let b = LatLng::try_new(10.7760, 106.7020).unwrap();
        for p in [Profile::Car, Profile::Motorbike] {
            let via_ch = e.route(p, a, b, None).unwrap();
            assert_eq!(via_ch.algo, "ch");
            // đối chiếu: chạy cùng cặp chân bằng A*
            let (sa, sb) = (
                e.snap_point(p, a, "a").unwrap(),
                e.snap_point(p, b, "b").unwrap(),
            );
            let view = e.view(p);
            let src: Vec<Leg> = sa.source_legs(view);
            let dst: Vec<Leg> = sb.target_legs(view);
            let (cost, ..) = e
                .route_legs_astar(view, e.costs(p).vmax_cm_per_unit, &src, &dst)
                .unwrap();
            assert_eq!(cost, via_ch.duration_ms);
            assert!(via_ch.distance_m > 100.0);
            assert!(via_ch.points.len() >= 2);
        }
    }

    #[test]
    fn foot_route_exists_and_is_slower_than_car() {
        let e = engine();
        let a = LatLng::try_new(10.7705, 106.6905).unwrap();
        let b = LatLng::try_new(10.7760, 106.7020).unwrap();
        let car = e.route(Profile::Car, a, b, None).unwrap();
        let foot = e.route(Profile::Foot, a, b, None).unwrap();
        assert!(foot.duration_ms > car.duration_ms);
    }

    #[test]
    fn far_point_is_rejected() {
        let e = engine();
        let far = LatLng::try_new(10.9, 106.9).unwrap();
        let near = LatLng::try_new(10.7705, 106.6905).unwrap();
        assert!(matches!(
            e.route(Profile::Car, far, near, None),
            Err(RouteError::TooFarFromRoad { .. })
        ));
    }

    #[test]
    fn matrix_matches_pairwise_routes() {
        let e = engine();
        let pts: Vec<LatLng> = [
            (10.7705, 106.6905),
            (10.7760, 106.7020),
            (10.7720, 106.6950),
            (10.7690, 106.7000),
        ]
        .iter()
        .map(|&(a, b)| LatLng::try_new(a, b).unwrap())
        .collect();
        let m = e.matrix(Profile::Car, &pts, &pts, None);
        for (i, &a) in pts.iter().enumerate() {
            for (j, &b) in pts.iter().enumerate() {
                let r = e.route(Profile::Car, a, b, None).unwrap();
                let got = m.matrix.get(i, j);
                assert_eq!(got, r.duration_ms, "{i}->{j}");
                if i == j {
                    assert_eq!(got, 0, "cùng một điểm thì chi phí 0");
                }
            }
        }
    }

    #[test]
    fn time_of_day_changes_costs_without_rebuilding_ordering() {
        let mut e = engine();
        let model = TrafficModel::new(&e.g, e.g.center(), 600.0);
        e.enable_time_of_day(Profile::Car, model);
        let a = LatLng::try_new(10.7705, 106.6905).unwrap();
        let b = LatLng::try_new(10.7760, 106.7020).unwrap();
        let night = e.route(Profile::Car, a, b, Some(3)).unwrap();
        let peak = e.route(Profile::Car, a, b, Some(8)).unwrap();
        assert!(peak.duration_ms >= night.duration_ms);
        let free = e.route(Profile::Car, a, b, None).unwrap();
        assert_eq!(night.duration_ms, free.duration_ms);
    }
}
