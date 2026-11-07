//! Điểm hẹn động (LOC-36): tính lại điểm hẹn khi cả hai đang di chuyển, nhưng chỉ đổi khi đáng.
//!
//! Một mô phỏng nhỏ riêng, theo từng giây (bước `tick_s`), cho một cặp (khách, tài xế). Ba chính sách:
//! - [`Policy::Static`]: không bao giờ đổi điểm hẹn (điểm chọn lúc đầu).
//! - [`Policy::Naive`]: tính lại gần như mỗi bước và đổi ngay khi `J` mới nhỏ hơn dù chỉ chút ít, không hỏi khách.
//! - [`Policy::Hysteresis`]: chỉ tính lại khi dự báo chênh lệch giờ tới lớn hơn `trigger_dev_s`, sau thời gian nguội
//!   `cooldown_s`, không đổi khi một bên sắp tới (`lock_s`), chỉ đổi khi `J` giảm ít nhất `min_gain_s` **và** `min_gain_frac`,
//!   và chỉ khi khách đồng ý (mô hình đồng ý là giả định của lab).
//!
//! Nền tảng chỉ thấy vị trí quan sát (khách có nhiễu GPS) và tốc độ ước từ quãng đã đi trong `speed_window_s`, không thấy
//! hệ số tốc độ thật của hai bên.

use crate::objective::{ObjectiveWeights, Terms};
use crate::rank::{DriverBelief, DriverRef, RankConfig, Ranker, Request, Scored};
use geo_core::{LatLng, destination, haversine_m};
use route_engine::engine::Engine;
use route_engine::position::Position;
use route_engine::profile::Profile;
use route_engine::turn::{EdgeRoute, EdgeSearcher, Goal, Start, TurnCosts};
use simulator::sim::ActiveRoute;
use std::cell::RefCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    Static,
    Naive,
    Hysteresis,
}

impl Policy {
    pub fn name(self) -> &'static str {
        match self {
            Policy::Static => "tĩnh",
            Policy::Naive => "đổi ngay",
            Policy::Hysteresis => "có trễ + đồng ý",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DynConfig {
    pub tick_s: f64,
    /// Khoảng nhỏ nhất giữa hai lần tính lại (giây) của chính sách «đổi ngay».
    pub naive_interval_s: f64,
    /// Chỉ tính lại khi giờ hai bên cùng có mặt dự báo trễ hơn kế hoạch quá chừng này (giây).
    pub trigger_dev_s: f64,
    pub cooldown_s: f64,
    pub lock_s: f64,
    pub min_gain_s: f64,
    pub min_gain_frac: f64,
    /// Khách đi thêm tối đa chừng này mét so với phần còn lại tới điểm hẹn cũ.
    pub max_extra_walk_m: f64,
    pub speed_window_s: f64,
    pub planned_walk_mps: f64,
    pub max_time_s: f64,
}

impl Default for DynConfig {
    fn default() -> Self {
        Self {
            tick_s: 5.0,
            naive_interval_s: 10.0,
            trigger_dev_s: 30.0,
            cooldown_s: 45.0,
            lock_s: 60.0,
            min_gain_s: 25.0,
            min_gain_frac: 0.10,
            max_extra_walk_m: 120.0,
            speed_window_s: 30.0,
            planned_walk_mps: 1.3,
            max_time_s: 1_500.0,
        }
    }
}

/// Mô hình khách đồng ý đổi điểm (giả định của lab): xác suất giảm theo quãng đi thêm.
pub fn consent_probability(extra_walk_m: f64) -> f64 {
    (0.95 - 0.004 * extra_walk_m.max(0.0)).clamp(0.05, 0.95)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Nominal,
    /// Khách đi chậm hơn dự kiến: tốc độ thật = hệ số × tốc độ dự kiến.
    SlowWalker(f64),
    /// Tài xế xuất phát muộn `s` giây (kẹt, nhận chuyến trễ).
    LateDriver(f64),
    /// Cả hai.
    Both {
        walk: f64,
        delay_s: f64,
    },
    /// Tốc độ đúng nhưng GPS khách nhiễu σ mét.
    NoisyGps(f64),
}

#[derive(Debug, Clone, Copy)]
pub struct DynScenario {
    pub cust0: LatLng,
    pub pin: LatLng,
    pub dest: LatLng,
    pub driver0: DriverRef,
    pub kind: Kind,
    pub seed: u64,
}

impl DynScenario {
    fn walk_factor(&self) -> f64 {
        match self.kind {
            Kind::SlowWalker(f) => f,
            Kind::Both { walk, .. } => walk,
            _ => 1.0,
        }
    }
    fn delay_s(&self) -> f64 {
        match self.kind {
            Kind::LateDriver(s) => s,
            Kind::Both { delay_s, .. } => delay_s,
            _ => 0.0,
        }
    }
    fn gps_noise(&self) -> f64 {
        match self.kind {
            Kind::NoisyGps(s) => s,
            _ => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DynOutcome {
    /// Giờ cả hai có mặt tại điểm hẹn cuối cùng (s kể từ lúc bắt đầu).
    pub t_pick_s: f64,
    pub meet_gap_s: f64,
    pub walk_total_m: f64,
    pub drive_total_m: f64,
    pub changes: u32,
    /// Đổi sang chỗ cách điểm hẹn đã bỏ trước đó dưới 40 m (đổi qua đổi lại).
    pub flips: u32,
    pub replans: u32,
    pub declined: u32,
    pub completed: bool,
}

struct Walker {
    pts: Vec<LatLng>,
    total_m: f64,
    start_t: f64,
    from_along: f64,
}

impl Walker {
    fn new(engine: &Engine, from: LatLng, to: LatLng, t: f64) -> Self {
        let pts = engine
            .route(Profile::Foot, from, to, None)
            .map(|r| r.points)
            .unwrap_or_else(|_| vec![from, to]);
        let total = geo_core::segment::polyline_length_m(&pts);
        Self {
            pts,
            total_m: total,
            start_t: t,
            from_along: 0.0,
        }
    }
    fn along(&self, t: f64, speed: f64) -> f64 {
        (self.from_along + (t - self.start_t).max(0.0) * speed).min(self.total_m)
    }
    fn pos(&self, t: f64, speed: f64) -> LatLng {
        geo_core::segment::point_at(&self.pts, self.along(t, speed)).unwrap_or(self.pts[0])
    }
}

struct Gauss(fastrand::Rng);
impl Gauss {
    fn n(&mut self) -> f64 {
        let (u1, u2) = (1.0 - self.0.f64(), self.0.f64());
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

pub struct Simulation<'e> {
    engine: &'e Engine,
    pub weights: ObjectiveWeights,
    pub cfg: DynConfig,
    ranker: Ranker,
    searcher: RefCell<EdgeSearcher>,
}

impl<'e> Simulation<'e> {
    pub fn new(engine: &'e Engine, weights: ObjectiveWeights, cfg: DynConfig) -> Self {
        let rc = RankConfig {
            k_drivers: 1,
            ..RankConfig::default()
        };
        Self {
            engine,
            ranker: Ranker::new(engine, weights.clone(), rc),
            weights,
            cfg,
            searcher: RefCell::new(EdgeSearcher::new(engine.g.edge_count())),
        }
    }

    fn route(&self, from: Start, to: Goal) -> Option<EdgeRoute> {
        let tm = self.engine.turn_model(Profile::Car, TurnCosts::urban());
        let vmax = self.engine.costs(Profile::Car).vmax_cm_per_unit;
        self.searcher
            .borrow_mut()
            .route(&tm, &self.engine.xy, vmax, from, to, true)
            .0
    }

    fn drive(&mut self, from: Position, to: Position, t0: f64, factor: f64) -> Option<ActiveRoute> {
        let r = self.route(Start::Pos(from), Goal::Pos(to))?;
        let tm = self.engine.turn_model(Profile::Car, TurnCosts::urban());
        Some(ActiveRoute::build(
            self.engine,
            &tm,
            &r,
            (t0 * 1000.0) as u64,
            factor,
        ))
    }

    /// Chấm lại các ứng viên với tốc độ ước hiện tại của hai bên (J «từ bây giờ»).
    fn rescored(&self, s: &Scored, v_c: f64, rho_d: f64) -> f64 {
        let t = Terms {
            walk_s: s.walk_m / v_c,
            drive_s: s.terms.drive_s / rho_d,
            ..s.terms
        };
        self.weights.j(&t)
    }

    pub fn run(&mut self, sc: &DynScenario, policy: Policy) -> DynOutcome {
        let eng = self.engine;
        let cfg = self.cfg;
        let mut gauss = Gauss(fastrand::Rng::with_seed(sc.seed ^ 0xD1CE));
        let mut coin = fastrand::Rng::with_seed(sc.seed ^ 0xC0FFEE);
        let walk_speed = cfg.planned_walk_mps * sc.walk_factor();
        let delay = sc.delay_s();
        // quyết định ban đầu: tài xế đã biết (phương án tĩnh tốt nhất có thể)
        let req0 = Request {
            from: sc.cust0,
            pin: sc.pin,
            dest: sc.dest,
        };
        let belief = DriverBelief::Live(vec![sc.driver0]);
        let mut cur: Scored = {
            let eng_ref = self.engine;
            let tm = eng_ref.turn_model(Profile::Car, TurnCosts::urban());
            let vmax = eng_ref.costs(Profile::Car).vmax_cm_per_unit;
            let es = &self.searcher;
            let mut rt = |f, t| es.borrow_mut().route(&tm, &eng_ref.xy, vmax, f, t, true).0;
            let r = self.ranker.rank(eng_ref, &req0, &belief, &mut rt);
            match r.top.first().copied() {
                Some(s) => s,
                None => return DynOutcome::default(),
            }
        };
        let mut out = DynOutcome::default();
        let mut walker = Walker::new(eng, sc.cust0, cur.cand.point, 0.0);
        let Some(mut dr) = self.drive(sc.driver0.pos, cur.cand.pos, delay, 1.0) else {
            return out;
        };
        // hệ số tốc độ thật của tài xế: 1.0 (chỉ có trễ xuất phát ở LateDriver)
        let mut driver_pos0 = sc.driver0.pos;
        let mut driver_t0 = delay;
        let mut abandoned: Vec<LatLng> = Vec::new();
        let (mut last_replan, mut last_change) = (-1e9f64, -1e9f64);
        let mut walk_done_m = 0.0;
        let mut drive_done_m = 0.0;
        let mut t = 0.0;
        let mut plan_finish: Option<f64> = None;
        let mut cust_arrived: Option<f64> = None;
        let mut drv_arrived: Option<f64> = None;
        while t <= cfg.max_time_s {
            // trạng thái thật
            let c_along = walker.along(t, walk_speed);
            let c_pos = walker.pos(t, walk_speed);
            let tms = (t * 1000.0) as u64;
            let d_pos = dr.position_at(tms.max(dr.start_ms));
            let d_dist = if tms >= dr.start_ms {
                dr.distance_until(tms)
            } else {
                0.0
            };
            if cust_arrived.is_none() && c_along >= walker.total_m - 0.5 {
                cust_arrived = Some(t);
            }
            if drv_arrived.is_none() && tms >= dr.start_ms + dr.total_ms {
                drv_arrived = Some(t);
            }
            if let (Some(a), Some(b)) = (cust_arrived, drv_arrived) {
                out.t_pick_s = a.max(b);
                out.meet_gap_s = (a - b).abs();
                out.completed = true;
                break;
            }
            // quan sát và dự báo
            let noise = sc.gps_noise();
            let seen_c = if noise > 0.0 {
                let (dx, dy) = (noise * gauss.n(), noise * gauss.n());
                destination(c_pos, dx.atan2(dy).to_degrees(), dx.hypot(dy))
            } else {
                c_pos
            };
            let w = cfg.speed_window_s;
            let v_c = if t >= w {
                let past = (walker.along(t - w, walk_speed)).min(walker.total_m);
                let travelled =
                    (c_along - past).max(0.0) + noise * std::f64::consts::SQRT_2 * gauss.n();
                (travelled / w).clamp(0.3, 2.0)
            } else {
                cfg.planned_walk_mps
            };
            let rho_d = if t >= driver_t0 + w {
                let tw = ((t - w) * 1000.0) as u64;
                let moved = (d_dist - dr.distance_until(tw.max(dr.start_ms))).max(0.0);
                let nominal = (dr.planned_ms as f64 / 1000.0).max(1.0);
                let nominal_speed = dr.length_m() / nominal;
                (moved / w / nominal_speed.max(0.1)).clamp(0.25, 1.5)
            } else if t > driver_t0 {
                1.0
            } else if t >= 15.0 {
                0.25
            } else {
                1.0
            };
            let rem_walk_m = (walker.total_m - c_along).max(0.0);
            let eta_c = rem_walk_m / v_c;
            let rem_drive_s = {
                let total = dr.planned_ms as f64 / 1000.0;
                let done_frac = if dr.length_m() > 0.0 {
                    d_dist / dr.length_m()
                } else {
                    1.0
                };
                total * (1.0 - done_frac)
            };
            // nền tảng không biết tài xế sẽ xuất phát lúc nào: thấy đứng yên thì chia cho `rho_d` nhỏ
            let eta_d = rem_drive_s / rho_d;
            // độ lệch so với kế hoạch: giờ hai bên cùng có mặt dự báo bây giờ trừ giờ đã dự báo lúc lập kế hoạch
            let finish_pred = t + eta_c.max(eta_d);
            let plan_finish_v = *plan_finish.get_or_insert(finish_pred);
            let dev = finish_pred - plan_finish_v;

            let eligible = match policy {
                Policy::Static => false,
                Policy::Naive => t - last_replan >= cfg.naive_interval_s && dev > 5.0,
                Policy::Hysteresis => {
                    dev > cfg.trigger_dev_s
                        && t - last_change >= cfg.cooldown_s
                        && t - last_replan >= cfg.cooldown_s * 0.5
                        && eta_c.max(eta_d) > cfg.lock_s
                }
            };
            if eligible {
                last_replan = t;
                out.replans += 1;
                let req = Request {
                    from: seen_c,
                    pin: sc.pin,
                    dest: sc.dest,
                };
                let belief = DriverBelief::Live(vec![DriverRef {
                    pos: d_pos,
                    latlng: d_pos.point(&eng.g),
                }]);
                let ranked = {
                    let tm = eng.turn_model(Profile::Car, TurnCosts::urban());
                    let vmax = eng.costs(Profile::Car).vmax_cm_per_unit;
                    let es = &self.searcher;
                    let mut rt = |f, to| es.borrow_mut().route(&tm, &eng.xy, vmax, f, to, true).0;
                    self.ranker.rank(eng, &req, &belief, &mut rt)
                };
                // J của điểm hẹn hiện tại tính lại từ bây giờ, và của ứng viên tốt nhất
                let cur_now = {
                    let tm = eng.turn_model(Profile::Car, TurnCosts::urban());
                    let vmax = eng.costs(Profile::Car).vmax_cm_per_unit;
                    let es = &self.searcher;
                    let mut rt = |f, to| es.borrow_mut().route(&tm, &eng.xy, vmax, f, to, true).0;
                    let mut n = 0;
                    let dest_pos: Vec<Position> = eng
                        .snap_point(Profile::Car, sc.dest, "điểm trả")
                        .map(|s| s.positions(eng.view(Profile::Car)))
                        .unwrap_or_default();
                    self.ranker.score_exact(
                        eng,
                        &req,
                        &[DriverRef {
                            pos: d_pos,
                            latlng: d_pos.point(&eng.g),
                        }],
                        false,
                        &dest_pos,
                        &cur.cand,
                        &mut rt,
                        &mut n,
                    )
                };
                let j_cur = self.rescored(&cur_now, v_c, rho_d);
                let best = ranked
                    .all
                    .iter()
                    .filter(|s| s.exact && s.j.is_finite())
                    .map(|s| (self.rescored(s, v_c, rho_d), *s))
                    .min_by(|a, b| a.0.total_cmp(&b.0));
                if let Some((j_new, cand)) = best {
                    let moved = haversine_m(cand.cand.point, cur.cand.point) > 5.0;
                    let extra_walk = cand.walk_m - rem_walk_m;
                    let gain = j_cur - j_new;
                    let ok_gain = match policy {
                        Policy::Naive => gain > 0.0,
                        _ => gain >= cfg.min_gain_s && gain >= cfg.min_gain_frac * j_cur,
                    };
                    let ok_walk = policy == Policy::Naive || extra_walk <= cfg.max_extra_walk_m;
                    if moved && ok_gain && ok_walk {
                        let accept = match policy {
                            Policy::Hysteresis => coin.f64() < consent_probability(extra_walk),
                            _ => true,
                        };
                        if !accept {
                            out.declined += 1;
                        } else if let Some(nd) =
                            self.drive(d_pos, cand.cand.pos, t.max(driver_t0), 1.0)
                        {
                            if abandoned
                                .iter()
                                .any(|a| haversine_m(*a, cand.cand.point) < 40.0)
                            {
                                out.flips += 1;
                            }
                            abandoned.push(cur.cand.point);
                            out.changes += 1;
                            last_change = t;
                            walk_done_m += c_along;
                            drive_done_m += d_dist;
                            let mut nw = Walker::new(eng, c_pos, cand.cand.point, t);
                            nw.from_along = 0.0;
                            walker = nw;
                            dr = nd;
                            driver_pos0 = d_pos;
                            driver_t0 = t.max(driver_t0);
                            cur = cand;
                            cust_arrived = None;
                            drv_arrived = None;
                            plan_finish = None;
                        }
                    }
                }
            }
            t += cfg.tick_s;
        }
        let _ = driver_pos0;
        out.walk_total_m = walk_done_m + walker.along(t.min(cfg.max_time_s), walk_speed);
        out.drive_total_m =
            drive_done_m + dr.distance_until(((t.min(cfg.max_time_s)) * 1000.0) as u64);
        if !out.completed {
            out.t_pick_s = cfg.max_time_s;
        }
        out
    }
}
