//! Xếp hạng ứng viên theo `J` trong hai tầng (LOC-34, LOC-35).
//!
//! **Vì sao hai tầng.** CH và ma trận của `route-engine` là theo nút và *không có chi phí rẽ*: chúng không thấy rằng
//! xe đang chạy ngược phía phải quay đầu mất 20 s (hoặc bị cấm). Chỉ A* theo cạnh (`turn::EdgeSearcher`) có chi phí rẽ,
//! nhưng đắt hơn. Nên:
//! - tầng 1: ma trận many-to-many rẻ cho **mọi** ứng viên → `J1`, cắt xuống `refine_top` ứng viên;
//! - tầng 2: chỉ ứng viên đứng đầu (và điểm ghim, luôn) được chấm lại bằng A* theo cạnh → `J2`, con số mà mô phỏng thực thi.
//!
//! Kết quả luôn gồm điểm ghim trong số được chấm tầng 2, nên ứng viên tốt nhất không bao giờ tệ hơn điểm ghim theo `J`.

use crate::candidates::{Candidate, CandidateGen, GenConfig, GenStats, Side};
use crate::objective::{ObjectiveWeights, Terms, Violation};
use geo_core::{LatLng, haversine_m};
use road_graph::{HighwayClass, flag};
use route_engine::engine::Engine;
use route_engine::matrix::bucket_matrix;
use route_engine::net::INF;
use route_engine::position::{Leg, Position};
use route_engine::profile::Profile;
use route_engine::turn::{EdgeRoute, Goal, Start};
use std::time::{Duration, Instant};

/// Tài xế như bộ xếp hạng thấy.
#[derive(Debug, Clone, Copy)]
pub struct DriverRef {
    pub pos: Position,
    pub latlng: LatLng,
}

/// Điều bộ xếp hạng biết về tài xế lúc chọn điểm đón.
#[derive(Debug, Clone)]
pub enum DriverBelief {
    /// Các tài xế đang rảnh (vị trí có hướng). Tài xế sẽ nhận chuyến được ước bằng `k_drivers` người gần nhất.
    Live(Vec<DriverRef>),
    /// Chưa biết tài xế nào: dùng `n` tài xế ảo rải đều quanh điểm ghim ở bán kính `radius_m` (phân bố quanh đó).
    Prior { n: usize, radius_m: f64 },
    /// Không có thông tin: số hạng tài xế bằng 0 cho mọi ứng viên.
    Unknown,
}

/// Tài xế nào sẽ nhận chuyến, theo bộ ghép phía sau.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverModel {
    /// Bộ ghép chọn tài xế tới điểm hẹn nhanh nhất: `T_drive` = nhỏ nhất theo thời gian trong `k_drivers` người gần khách nhất.
    BestTime,
    /// Bộ ghép baseline `NearestStraightLine`: tài xế rảnh gần ĐIỂM hẹn nhất theo chim bay. Mỗi ứng viên có tài xế riêng.
    NearestCrow,
}

#[derive(Debug, Clone)]
pub struct RankConfig {
    pub driver_model: DriverModel,
    pub k_drivers: usize,
    /// Số tài xế (trong số `k_drivers`, theo thứ tự gần nhất tới ứng viên theo ma trận) được chấm bằng A* theo cạnh.
    pub k_exact_drivers: usize,
    /// Số ứng viên (ngoài điểm ghim) được chấm lại ở tầng 2.
    pub refine_top: usize,
    /// `false` = chỉ ma trận (tầng 1).
    pub stage2: bool,
    /// Bật phạt phía đường bất lợi.
    pub use_side: bool,
    /// Chạy tầng 2 tối đa chừng này, hết thì dừng và đánh dấu `degraded` (None = không giới hạn, tất định).
    pub deadline: Option<Duration>,
    pub walk_speed_mps: f64,
    /// Số vị trí khác nhau trả về.
    pub top_n: usize,
    pub profile: Profile,
}

impl Default for RankConfig {
    fn default() -> Self {
        Self {
            driver_model: DriverModel::NearestCrow,
            k_drivers: 4,
            k_exact_drivers: 2,
            refine_top: 4,
            stage2: true,
            use_side: true,
            deadline: None,
            walk_speed_mps: 1.3,
            top_n: 3,
            profile: Profile::Car,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Request {
    /// Chỗ khách đứng theo điện thoại.
    pub from: LatLng,
    pub pin: LatLng,
    /// Điểm đến (đã snap).
    pub dest: LatLng,
}

#[derive(Debug, Clone, Copy)]
pub struct Scored {
    pub cand: Candidate,
    pub terms: Terms,
    pub j: f64,
    /// `true` nếu đã chấm bằng A* theo cạnh.
    pub exact: bool,
    /// Quãng đi bộ đã tính (mét): đồ thị đi bộ nếu `exact`, ước lượng nếu không.
    pub walk_m: f64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RankStats {
    pub gen_stats: GenStats,
    pub n_eligible: usize,
    pub matrix_us: u64,
    pub refine_us: u64,
    pub total_us: u64,
    pub n_refined: usize,
    pub n_edge_routes: usize,
    pub degraded: bool,
    /// Số tài xế dùng để ước `T_drive`.
    pub drivers_used: usize,
    /// Điểm ghim không dùng được (vi phạm ràng buộc cứng) và không còn ứng viên khác: buộc dùng ghim.
    pub pin_forced: bool,
    /// Số ứng viên thêm từ bảng điểm đón học được.
    pub hubs_added: usize,
}

pub struct Ranked {
    /// Tối đa `top_n` vị trí khác nhau, `J` tăng dần.
    pub top: Vec<Scored>,
    /// Mọi ứng viên đã được chấm (gồm cả hai chiều của cùng một vị trí), `J` tăng dần; tầng 2 trước tầng 1.
    pub all: Vec<Scored>,
    /// Điểm ghim tốt nhất theo `J` (nếu snap được).
    pub pin: Option<Scored>,
    pub stats: RankStats,
}

/// Ứng viên đã chấm tầng 1 kèm chỉ số trong danh sách hợp lệ.
type Indexed = (usize, Scored);

pub type RouteFn<'a> = dyn FnMut(Start, Goal) -> Option<EdgeRoute> + 'a;

pub struct Ranker {
    pub weights: ObjectiveWeights,
    pub cfg: RankConfig,
    gen_: CandidateGen,
    /// Điểm đón «học từ lịch sử» (LOC-39): thêm làm ứng viên nếu trong tầm đi bộ và hợp lệ.
    hubs: Vec<LatLng>,
}

/// Hệ số phạt phía đường theo cấp đường: đường lớn băng khó, hẻm thì không đáng kể.
pub fn side_scale(class: HighwayClass, lanes: u8) -> f64 {
    use HighwayClass::*;
    match class {
        Trunk | Primary | Secondary | TrunkLink | PrimaryLink | SecondaryLink => 1.0,
        Tertiary | TertiaryLink => 0.5,
        _ if lanes >= 3 => 1.0,
        _ if lanes == 2 => 0.5,
        _ => 0.0,
    }
}

impl Ranker {
    pub fn new(engine: &Engine, weights: ObjectiveWeights, cfg: RankConfig) -> Self {
        let gen_cfg = GenConfig {
            max_walk_m: weights.max_walk_m,
            profile: cfg.profile,
            ..GenConfig::default()
        };
        Self {
            gen_: CandidateGen::new(engine, gen_cfg),
            weights,
            cfg,
            hubs: Vec::new(),
        }
    }

    /// Đặt bảng điểm đón học được (LOC-39). Mỗi điểm được snap, kiểm ràng buộc cứng và ngưỡng đi bộ như mọi ứng viên.
    pub fn set_hubs(&mut self, hubs: Vec<LatLng>) {
        self.hubs = hubs;
    }

    pub fn generator(&mut self) -> &mut CandidateGen {
        &mut self.gen_
    }

    fn penalty(&self, engine: &Engine, c: &Candidate, walk_m: f64) -> f64 {
        let p = &self.weights.penalties;
        let mut s = self.weights.tail_walk_penalty(walk_m);
        if self.cfg.use_side && c.side == Side::Left {
            let lanes = engine.g.lanes[c.pos.edge as usize];
            s += p.wrong_side_s * side_scale(c.class, lanes);
        }
        if self.cfg.profile == Profile::Car && c.flags & flag::ALLEY != 0 {
            s += p.alley_stop_s;
        }
        s
    }

    /// Quãng đi bộ theo đồ thị đi bộ như baseline (`Ctx::walk`): khách → `to`; chim bay × 1,3 nếu không snap được.
    pub fn walk_m(engine: &Engine, from: LatLng, to: LatLng) -> f64 {
        match engine.route(Profile::Foot, from, to, None) {
            Ok(r) => r.distance_m,
            Err(_) => haversine_m(from, to) * 1.3,
        }
    }

    /// Chọn các tài xế dùng để ước `T_drive` (vị trí có hướng).
    fn drivers_for(
        &self,
        engine: &Engine,
        req: &Request,
        b: &DriverBelief,
    ) -> (Vec<DriverRef>, bool) {
        match b {
            DriverBelief::Live(all) => {
                let mut v = all.clone();
                v.sort_by(|a, b| {
                    haversine_m(a.latlng, req.from).total_cmp(&haversine_m(b.latlng, req.from))
                });
                v.truncate(self.cfg.k_drivers.max(1));
                (v, false)
            }
            DriverBelief::Prior { n, radius_m } => {
                let view = engine.view(self.cfg.profile);
                let mut v = Vec::new();
                for i in 0..*n {
                    let bearing = 360.0 * i as f64 / (*n).max(1) as f64;
                    let p = geo_core::destination(req.pin, bearing, *radius_m);
                    if let Ok(s) = engine.snap_point(self.cfg.profile, p, "tài xế ảo")
                        && let Some(pos) = s.positions(view).first().copied()
                    {
                        v.push(DriverRef {
                            pos,
                            latlng: s.point,
                        });
                    }
                }
                (v, true)
            }
            DriverBelief::Unknown => (Vec::new(), false),
        }
    }

    /// Gộp thời gian của các tài xế (giây): người gần nhất nếu đã biết tài xế, trung bình nếu là phân bố.
    fn fold_drive(times_ms: &[u32], prior: bool) -> f64 {
        let fin: Vec<f64> = times_ms
            .iter()
            .filter(|&&t| t != INF)
            .map(|&t| f64::from(t) / 1000.0)
            .collect();
        if fin.is_empty() {
            return if times_ms.is_empty() {
                0.0
            } else {
                f64::INFINITY
            };
        }
        if prior {
            fin.iter().sum::<f64>() / fin.len() as f64
        } else {
            fin.iter().copied().fold(f64::INFINITY, f64::min)
        }
    }

    /// Chấm một ứng viên bằng A* theo cạnh (tầng 2). `drivers` đã chọn sẵn; `dest_pos` là các vị trí vào điểm trả.
    #[allow(clippy::too_many_arguments)]
    pub fn score_exact(
        &self,
        engine: &Engine,
        req: &Request,
        drivers: &[DriverRef],
        prior: bool,
        dest_pos: &[Position],
        c: &Candidate,
        route: &mut RouteFn,
        n_routes: &mut usize,
    ) -> Scored {
        let walk_m = Self::walk_m(engine, req.from, c.point);
        let walk_s = walk_m / self.cfg.walk_speed_mps;
        let mut times = Vec::with_capacity(drivers.len());
        // phân bố quanh đó: lấy trung bình mọi tài xế ảo được truyền vào; `k_exact_drivers` chỉ giới hạn tài xế thật
        let k_exact = if prior {
            drivers.len()
        } else {
            self.cfg.k_exact_drivers.max(1)
        };
        for d in drivers.iter().take(k_exact) {
            *n_routes += 1;
            times.push(route(Start::Pos(d.pos), Goal::Pos(c.pos)).map_or(INF, |r| r.cost));
        }
        let drive_s = Self::fold_drive(&times, prior);
        let mut trip = f64::INFINITY;
        for &dp in dest_pos {
            *n_routes += 1;
            if let Some(r) = route(Start::Pos(c.pos), Goal::Pos(dp)) {
                trip = trip.min(f64::from(r.cost) / 1000.0);
            }
        }
        if dest_pos.is_empty() {
            trip = 0.0;
        }
        let terms = Terms {
            walk_s,
            drive_s,
            trip_s: trip,
            penalty_s: self.penalty(engine, c, walk_m),
        };
        Scored {
            cand: *c,
            terms,
            j: self.weights.j(&terms),
            exact: true,
            walk_m,
        }
    }

    pub fn rank(
        &mut self,
        engine: &Engine,
        req: &Request,
        belief: &DriverBelief,
        route: &mut RouteFn,
    ) -> Ranked {
        let t0 = Instant::now();
        let gen_out = self.gen_.generate(engine, req.from, req.pin);
        let mut stats = RankStats {
            gen_stats: gen_out.stats,
            ..RankStats::default()
        };
        let pin_ok: Vec<Candidate> = gen_out
            .pin
            .iter()
            .copied()
            .filter(|p| p.violation.is_none())
            .collect();
        let mut elig: Vec<Candidate> = gen_out.candidates.clone();
        elig.extend(pin_ok.iter().copied());
        if elig.is_empty() {
            // không còn gì hợp lệ: buộc dùng điểm ghim (kể cả khi nó vi phạm), không bao giờ bỏ khách
            elig.extend(gen_out.pin.iter().copied());
            stats.pin_forced = !elig.is_empty();
        }
        if !self.hubs.is_empty() {
            let view = engine.view(self.cfg.profile);
            for h in &self.hubs {
                if haversine_m(*h, req.from) > self.weights.max_walk_m {
                    continue;
                }
                let Ok(sn) = engine.snap_point(self.cfg.profile, *h, "điểm học được")
                else {
                    continue;
                };
                if sn.snap_dist_m > 30.0 {
                    continue;
                }
                let walk = Self::walk_m(engine, req.from, sn.point);
                if walk > self.weights.max_walk_m {
                    continue;
                }
                for p in sn.positions(view) {
                    if crate::candidates::stop_violation(engine, p.edge, 20.0).is_some() {
                        continue;
                    }
                    if elig
                        .iter()
                        .any(|c| c.pos.edge == p.edge && haversine_m(c.point, sn.point) < 10.0)
                    {
                        continue;
                    }
                    elig.push(Candidate {
                        pos: p,
                        point: sn.point,
                        walk_est_m: walk,
                        is_pin: false,
                        class: HighwayClass::from_u8(engine.g.class[p.edge as usize])
                            .unwrap_or(HighwayClass::Residential),
                        flags: engine.g.flags[p.edge as usize],
                        side: crate::candidates::side_of(engine, p.edge, p.frac, req.from).0,
                        merged: 1,
                        violation: None,
                    });
                    stats.hubs_added += 1;
                }
            }
        }
        stats.n_eligible = elig.len();
        if elig.is_empty() {
            stats.total_us = t0.elapsed().as_micros() as u64;
            return Ranked {
                top: Vec::new(),
                all: Vec::new(),
                pin: None,
                stats,
            };
        }
        let view = engine.view(self.cfg.profile);
        let (mut drivers, prior) = self.drivers_for(engine, req, belief);
        // NearestCrow: mỗi ứng viên có đúng một tài xế (rảnh gần nó nhất theo chim bay), như bộ ghép baseline
        let mut assigned: Option<Vec<usize>> = None;
        if let (DriverBelief::Live(all), DriverModel::NearestCrow) = (belief, self.cfg.driver_model)
            && !all.is_empty()
        {
            let mut uniq: Vec<DriverRef> = Vec::new();
            let mut rows: Vec<usize> = Vec::with_capacity(elig.len());
            for c in &elig {
                let best = all
                    .iter()
                    .enumerate()
                    .min_by(|a, b| {
                        haversine_m(a.1.latlng, c.point)
                            .total_cmp(&haversine_m(b.1.latlng, c.point))
                    })
                    .map(|x| x.0)
                    .expect("không rỗng");
                let d = all[best];
                let r = uniq
                    .iter()
                    .position(|u| u.pos.edge == d.pos.edge && u.pos.frac == d.pos.frac)
                    .unwrap_or_else(|| {
                        uniq.push(d);
                        uniq.len() - 1
                    });
                rows.push(r);
            }
            drivers = uniq;
            assigned = Some(rows);
        }
        stats.drivers_used = drivers.len();
        let dest_pos: Vec<Position> = engine
            .snap_point(self.cfg.profile, req.dest, "điểm trả")
            .map(|s| s.positions(view))
            .unwrap_or_default();

        // tầng 1: hai ma trận
        let tm = Instant::now();
        let ch = engine.ch(self.cfg.profile);
        let tgt_c: Vec<Vec<Leg>> = elig.iter().map(|c| c.pos.target_legs(view)).collect();
        let drive_ms: Vec<Vec<u32>> = if drivers.is_empty() {
            Vec::new()
        } else {
            let src: Vec<Vec<Leg>> = drivers.iter().map(|d| d.pos.source_legs(view)).collect();
            let m = bucket_matrix(ch, &src, &tgt_c, false);
            (0..elig.len())
                .map(|j| match &assigned {
                    Some(a) => vec![m.get(a[j], j)],
                    None => (0..drivers.len()).map(|i| m.get(i, j)).collect(),
                })
                .collect()
        };
        let mut best_dest: Vec<usize> = vec![0; elig.len()];
        let trip_ms: Vec<u32> = if dest_pos.is_empty() {
            Vec::new()
        } else {
            let src: Vec<Vec<Leg>> = elig.iter().map(|c| c.pos.source_legs(view)).collect();
            let tgt: Vec<Vec<Leg>> = dest_pos.iter().map(|d| d.target_legs(view)).collect();
            let m = bucket_matrix(ch, &src, &tgt, false);
            (0..elig.len())
                .map(|i| {
                    let (j, v) = (0..dest_pos.len())
                        .map(|j| (j, m.get(i, j)))
                        .min_by_key(|x| x.1)
                        .unwrap_or((0, INF));
                    best_dest[i] = j;
                    v
                })
                .collect()
        };
        stats.matrix_us = tm.elapsed().as_micros() as u64;

        let mut stage1: Vec<(usize, Scored)> = elig
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let walk_m = if c.is_pin {
                    Self::walk_m(engine, req.from, c.point)
                } else {
                    c.walk_est_m
                };
                let drive_s = if drivers.is_empty() {
                    0.0
                } else {
                    Self::fold_drive(&drive_ms[i], prior)
                };
                let trip_s = if dest_pos.is_empty() || trip_ms[i] == INF {
                    if dest_pos.is_empty() {
                        0.0
                    } else {
                        f64::INFINITY
                    }
                } else {
                    f64::from(trip_ms[i]) / 1000.0
                };
                let terms = Terms {
                    walk_s: walk_m / self.cfg.walk_speed_mps,
                    drive_s,
                    trip_s,
                    penalty_s: self.penalty(engine, c, walk_m),
                };
                (
                    i,
                    Scored {
                        cand: *c,
                        terms,
                        j: self.weights.j(&terms),
                        exact: false,
                        walk_m,
                    },
                )
            })
            .collect();
        stage1.sort_by(|a, b| a.1.j.total_cmp(&b.1.j));

        // tầng 2
        let mut all: Vec<Scored> = Vec::new();
        let mut rest: Vec<Scored> = Vec::new();
        if self.cfg.stage2 {
            let tr = Instant::now();
            let (pins, others): (Vec<Indexed>, Vec<Indexed>) =
                stage1.into_iter().partition(|s| s.1.cand.is_pin);
            let mut order: Vec<Indexed> = pins;
            let mut n_other = 0;
            let mut tail: Vec<Scored> = Vec::new();
            for s in others {
                if n_other < self.cfg.refine_top {
                    order.push(s);
                    n_other += 1;
                } else {
                    tail.push(s.1);
                }
            }
            for (idx, s) in order {
                if let Some(dl) = self.cfg.deadline
                    && !s.cand.is_pin
                    && tr.elapsed() > dl
                {
                    stats.degraded = true;
                    tail.push(s);
                    continue;
                }
                // tài xế gần ứng viên này nhất theo ma trận đứng trước; điểm trả: cột tốt nhất theo ma trận
                let sel: Vec<DriverRef> = if drive_ms.is_empty() {
                    Vec::new()
                } else if prior {
                    // bốn tài xế ảo cách đều nhau, không phải bốn người đầu danh sách (sẽ lệch về một phía của ghim)
                    let step = (drivers.len() / 4).max(1);
                    drivers.iter().step_by(step).take(4).copied().collect()
                } else {
                    if let Some(a) = &assigned {
                        vec![drivers[a[idx]]]
                    } else {
                        let mut o: Vec<usize> = (0..drivers.len()).collect();
                        o.sort_by_key(|&k| drive_ms[idx][k]);
                        o.into_iter().map(|k| drivers[k]).collect()
                    }
                };
                let dsel: Vec<Position> = if dest_pos.len() > 1 {
                    vec![dest_pos[best_dest[idx]]]
                } else {
                    dest_pos.clone()
                };
                let ex = self.score_exact(
                    engine,
                    req,
                    &sel,
                    prior,
                    &dsel,
                    &s.cand,
                    route,
                    &mut stats.n_edge_routes,
                );
                stats.n_refined += 1;
                all.push(ex);
            }
            rest = tail;
            stats.refine_us = tr.elapsed().as_micros() as u64;
        } else {
            all = stage1.into_iter().map(|x| x.1).collect();
        }
        all.sort_by(|a, b| a.j.total_cmp(&b.j));
        rest.sort_by(|a, b| a.j.total_cmp(&b.j));
        let pin = all
            .iter()
            .chain(rest.iter())
            .filter(|s| s.cand.is_pin)
            .min_by(|a, b| a.j.total_cmp(&b.j))
            .copied();
        // top: vị trí khác nhau, `J` tăng dần
        let mut top: Vec<Scored> = Vec::new();
        for s in all.iter() {
            if !s.j.is_finite() {
                continue;
            }
            let dup = top
                .iter()
                .any(|t| haversine_m(t.cand.point, s.cand.point) < 1.0);
            if !dup {
                top.push(*s);
            }
            if top.len() >= self.cfg.top_n {
                break;
            }
        }
        all.extend(rest);
        stats.total_us = t0.elapsed().as_micros() as u64;
        Ranked {
            top,
            all,
            pin,
            stats,
        }
    }
}

/// Mô tả ngắn một ràng buộc cứng cho lab.
pub fn violation_name(v: Violation) -> &'static str {
    match v {
        Violation::WalkTooFar => "đi bộ quá xa",
        Violation::BridgeOrTunnel => "cầu/hầm/cầu vượt",
        Violation::NoStopClass => "đường cấm dừng",
        Violation::Roundabout => "vòng xuyến",
        Violation::TooShort => "cạnh quá ngắn",
        Violation::NoVehicleAccess => "xe không vào được",
        Violation::OutsideMandatoryZone => "ngoài vùng đón bắt buộc",
    }
}
