//! Chiến lược cắm vào simulator qua `PickupStrategy` (LOC-32…38): chọn điểm đón theo `J` hai tầng.
//!
//! Khi điểm đón tốt nhất chính là điểm ghim, kết quả trả về **y hệt** `AtPin` (cùng `PickupChoice`), nên với khách đó
//! mô phỏng không đổi một byte nào so với baseline.

use crate::dropoff::{DropLog, DropoffPolicy};
use crate::objective::ObjectiveWeights;
use crate::rank::{DriverBelief, DriverRef, RankConfig, RankStats, Ranker, Request};
use geo_core::{LatLng, destination};
use route_engine::position::Position;
use simulator::strategy::{
    AtPin, Ctx, CustomerView, DriverView, DropView, PickupChoice, PickupStrategy,
};
use std::cell::RefCell;
use std::rc::Rc;

/// Điều bộ xếp hạng được phép biết về tài xế lúc khách đặt xe.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DriverInfo {
    /// Biết các tài xế rảnh (vị trí có hướng); nếu không còn ai rảnh thì dùng phân bố quanh đó.
    Live { prior_radius_m: f64 },
    /// Không bao giờ biết tài xế, chỉ dùng phân bố quanh điểm ghim.
    PriorOnly { radius_m: f64 },
    /// Không dùng thông tin tài xế (`T_drive` = 0).
    None,
}

#[derive(Debug, Clone)]
pub struct JConfig {
    pub name: String,
    pub weights: ObjectiveWeights,
    pub rank: RankConfig,
    pub drivers: DriverInfo,
    /// Sai số GPS (σ mét mỗi trục) của chỗ đứng mà chiến lược nhìn thấy; khách vẫn đi bộ từ chỗ đứng thật. 0 = hoàn hảo.
    pub gps_noise_m: f64,
    /// Hướng thứ hai của cùng vị trí được thêm vào `approach` nếu `J` hơn kém không quá chừng này (giây).
    pub commit_margin_s: f64,
    /// Chọn điểm trả (LOC-38); `None` = baseline trả đúng điểm đích đã snap.
    pub dropoff: Option<DropoffPolicy>,
    /// `false` = điểm đón vẫn là baseline (đúng ghim); dùng để đo riêng tác dụng của điểm trả.
    pub pickup_j: bool,
    /// Bảng điểm đón học từ lịch sử (LOC-39), rỗng = không dùng.
    pub hubs: Vec<LatLng>,
}

impl JConfig {
    pub fn new(name: &str, weights: ObjectiveWeights) -> Self {
        Self {
            name: name.into(),
            weights,
            rank: RankConfig::default(),
            drivers: DriverInfo::Live {
                prior_radius_m: 1500.0,
            },
            gps_noise_m: 0.0,
            commit_margin_s: 20.0,
            dropoff: None,
            pickup_j: true,
            hubs: Vec::new(),
        }
    }
}

/// Ghi lại từng quyết định để lab thống kê (không ảnh hưởng mô phỏng).
#[derive(Debug, Clone, Copy)]
pub struct ChoiceLog {
    pub customer: u32,
    pub chose_pin: bool,
    pub j_chosen: f64,
    pub j_pin: f64,
    pub walk_m: f64,
    pub stats: RankStats,
}

pub type ChoiceHandle = Rc<RefCell<Vec<ChoiceLog>>>;

pub struct JPickup {
    cfg: JConfig,
    ranker: Option<Ranker>,
    idle: Vec<DriverView>,
    pub log: ChoiceHandle,
    pub drop_log: Rc<RefCell<DropLog>>,
    dropper: Option<crate::dropoff::Dropper>,
}

impl JPickup {
    pub fn new(cfg: JConfig) -> Self {
        Self {
            cfg,
            ranker: None,
            idle: Vec::new(),
            log: Rc::new(RefCell::new(Vec::new())),
            drop_log: Rc::new(RefCell::new(DropLog::default())),
            dropper: None,
        }
    }

    /// Dùng chung bộ ghi với bên ngoài (để đọc sau khi mô phỏng chạy xong).
    pub fn with_logs(mut self, log: ChoiceHandle, drop_log: Rc<RefCell<DropLog>>) -> Self {
        self.log = log;
        self.drop_log = drop_log;
        self
    }
}

impl PickupStrategy for JPickup {
    fn name(&self) -> &str {
        &self.cfg.name
    }

    fn observe_idle(&mut self, idle: &[DriverView]) {
        self.idle.clear();
        self.idle.extend_from_slice(idle);
    }

    fn choose(&mut self, ctx: &Ctx, c: &CustomerView) -> Option<PickupChoice> {
        if !self.cfg.pickup_j {
            return AtPin.choose(ctx, c);
        }
        let engine = ctx.engine;
        if self.ranker.is_none() {
            let mut rc = self.cfg.rank.clone();
            rc.walk_speed_mps = ctx.scenario.customer.walk_speed_mps;
            let mut r = Ranker::new(engine, self.cfg.weights.clone(), rc);
            r.set_hubs(self.cfg.hubs.clone());
            self.ranker = Some(r);
        }
        let ranker = self.ranker.as_mut().expect("vừa tạo");
        let belief = match self.cfg.drivers {
            DriverInfo::Live { prior_radius_m } => {
                if self.idle.is_empty() {
                    DriverBelief::Prior {
                        n: 8,
                        radius_m: prior_radius_m,
                    }
                } else {
                    DriverBelief::Live(
                        self.idle
                            .iter()
                            .map(|d| DriverRef {
                                pos: d.pos,
                                latlng: d.latlng,
                            })
                            .collect(),
                    )
                }
            }
            DriverInfo::PriorOnly { radius_m } => DriverBelief::Prior { n: 8, radius_m },
            DriverInfo::None => DriverBelief::Unknown,
        };
        let from = seen_position(c, self.cfg.gps_noise_m, ctx.scenario.seed);
        let req = Request {
            from,
            pin: c.pin,
            dest: c.dropoff,
        };
        let ranked = {
            let mut route = |s, g| ctx.route_car(s, g);
            ranker.rank(engine, &req, &belief, &mut route)
        };
        let best = ranked.top.first().copied();
        let j_pin = ranked.pin.map_or(f64::INFINITY, |p| p.j);
        let Some(best) = best else {
            // không có ứng viên nào hợp lệ: lùi về baseline
            return AtPin.choose(ctx, c);
        };
        self.log.borrow_mut().push(ChoiceLog {
            customer: c.id,
            chose_pin: best.cand.is_pin,
            j_chosen: best.j,
            j_pin,
            walk_m: best.walk_m,
            stats: ranked.stats,
        });
        if best.cand.is_pin {
            // đúng như baseline: cùng điểm, cùng hướng vào
            return AtPin.choose(ctx, c);
        }
        // chiều thứ hai của cùng vị trí, nếu `J` không tệ hơn quá `commit_margin_s`
        let g = &engine.g;
        let mut approach = vec![best.cand.pos];
        let tw = g.twin[best.cand.pos.edge as usize];
        if tw != road_graph::model::NONE
            && let Some(o) = ranked.all.iter().find(|s| s.cand.pos.edge == tw)
            && o.j - best.j <= self.cfg.commit_margin_s
        {
            approach.push(o.cand.pos);
        }
        let (walk_m, walk_s) = ctx.walk(c.true_pos, best.cand.point);
        let pin_snap_m = ctx.snap_car(c.pin).map_or(0.0, |s| s.snap_dist_m);
        Some(PickupChoice {
            stop_point: best.cand.point,
            approach,
            walk_m,
            walk_s,
            pin_snap_m,
        })
    }

    fn choose_dropoff(&mut self, ctx: &Ctx, d: &DropView) -> Option<Vec<Position>> {
        let policy = self.cfg.dropoff?;
        if self.dropper.is_none() {
            self.dropper = Some(crate::dropoff::Dropper::new(
                ctx.engine,
                self.cfg.weights.clone(),
                policy,
            ));
        }
        let dropper = self.dropper.as_mut().expect("vừa tạo");
        dropper.choose(ctx, d, &self.drop_log)
    }
}

/// Chỗ đứng mà chiến lược nhìn thấy: chỗ thật cộng nhiễu GPS Gauss σ mét mỗi trục, tất định theo (seed, mã khách).
pub fn seen_position(c: &CustomerView, sigma_m: f64, seed: u64) -> LatLng {
    if sigma_m <= 0.0 {
        return c.true_pos;
    }
    let mut r =
        fastrand::Rng::with_seed(seed ^ (u64::from(c.id) + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let mut g = || {
        let (u1, u2) = (1.0 - r.f64(), r.f64());
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    };
    let (dx, dy) = (sigma_m * g(), sigma_m * g());
    destination(c.true_pos, dx.atan2(dy).to_degrees(), dx.hypot(dy))
}
