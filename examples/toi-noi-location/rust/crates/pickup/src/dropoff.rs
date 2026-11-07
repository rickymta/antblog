//! Điểm trả (LOC-38): gần đích thật, đúng phía, dễ bắt chuyến kế.
//!
//! Hàm mục tiêu cùng dạng với điểm đón nhưng đổi vai:
//!
//! ```text
//! J_trả(p) = a·T_bộ(p → đích thật hoặc cổng) + d·T_trip(xe → p) + e·T_thoát(p) + phạt(phía đường, đi bộ quá xa)
//! ```
//!
//! `T_thoát` là chi phí tài xế rời điểm trả: thời gian A* theo cạnh (có quay đầu, cấm rẽ) từ `p` tới nơi nhu cầu gần nhất.
//! Đây là **đại diện** cho «chuyến kế» (mô phỏng không cho biết khách kế tiếp lúc trả khách): các tâm điểm nóng của kịch bản
//! đóng vai bản đồ nhu cầu mà nền tảng có (LOC-49). Điểm trả mặc định của baseline (đích đã snap) luôn là một ứng viên.

use crate::candidates::{Candidate, CandidateGen, GenConfig, Side, side_of};
use crate::objective::ObjectiveWeights;
use crate::rank::{Ranker, side_scale};
use geo_core::{LatLng, haversine_m};
use route_engine::engine::Engine;
use route_engine::matrix::bucket_matrix;
use route_engine::net::INF;
use route_engine::position::Position;
use route_engine::profile::Profile;
use route_engine::turn::{Goal, Start};
use simulator::strategy::{AtPin, Ctx, CustomerView, DropView, PickupChoice, PickupStrategy};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropoffPolicy {
    /// Khách đi bộ từ điểm trả tới đích (giây × a).
    pub a_walk: f64,
    /// Thời gian xe tới điểm trả (đoạn có khách).
    pub d_trip: f64,
    /// Chi phí thoát của tài xế.
    pub e_exit: f64,
    pub max_walk_m: f64,
    /// Số ứng viên được chấm đầy đủ sau khi lọc bằng ma trận.
    pub refine_top: usize,
    /// Số tâm nhu cầu gần nhất dùng cho `T_thoát`.
    pub anchors_k: usize,
    /// Bán kính tìm cổng (`entrance=*`) quanh đích thật; 0 = không dùng cổng.
    pub gate_radius_m: f64,
    pub use_exit: bool,
    pub use_side: bool,
}

impl Default for DropoffPolicy {
    fn default() -> Self {
        Self {
            a_walk: 1.0,
            d_trip: 1.0,
            e_exit: 1.0,
            max_walk_m: 150.0,
            refine_top: 6,
            anchors_k: 2,
            gate_radius_m: 80.0,
            use_exit: true,
            use_side: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DropEntry {
    pub customer: u32,
    /// Khách đi bộ từ điểm trả thật sự tới đích (hoặc cổng), mét.
    pub walk_m: f64,
    pub chose_default: bool,
    pub gate: bool,
    pub exit_s: f64,
    pub j_default: f64,
    pub j_chosen: f64,
    pub n_candidates: usize,
}

#[derive(Debug, Default, Clone)]
pub struct DropLog {
    pub entries: Vec<DropEntry>,
}

pub struct Dropper {
    gen_: CandidateGen,
    policy: DropoffPolicy,
    weights: ObjectiveWeights,
    anchors: Option<Vec<(LatLng, Position)>>,
}

/// Cổng gần nhất của đích thật trong `radius_m` (nút `entrance=*` thô của OSM trong đồ thị), nếu có.
pub fn nearest_gate(engine: &Engine, dest: LatLng, radius_m: f64) -> Option<LatLng> {
    if radius_m <= 0.0 {
        return None;
    }
    engine
        .g
        .entrances
        .iter()
        .map(|e| LatLng::try_new(f64::from(e.lat_e7) / 1e7, f64::from(e.lng_e7) / 1e7))
        .filter_map(Result::ok)
        .map(|p| (haversine_m(p, dest), p))
        .filter(|(d, _)| *d <= radius_m)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, p)| p)
}

impl Dropper {
    pub fn new(engine: &Engine, weights: ObjectiveWeights, policy: DropoffPolicy) -> Self {
        let cfg = GenConfig {
            max_walk_m: policy.max_walk_m,
            profile: Profile::Car,
            ..GenConfig::default()
        };
        Self {
            gen_: CandidateGen::new(engine, cfg),
            policy,
            weights,
            anchors: None,
        }
    }

    fn anchors(&mut self, ctx: &Ctx) -> &[(LatLng, Position)] {
        if self.anchors.is_none() {
            let view = ctx.engine.view(Profile::Car);
            let v = ctx
                .scenario
                .demand
                .hotspots
                .iter()
                .filter(|h| h.weight > 0.0)
                .filter_map(|h| {
                    let ll = LatLng::try_new(h.lat, h.lng).ok()?;
                    let s = ctx.snap_car(ll)?;
                    Some((ll, *s.positions(view).first()?))
                })
                .collect();
            self.anchors = Some(v);
        }
        self.anchors.as_deref().unwrap_or(&[])
    }

    /// Chi phí thoát (giây) từ `p`: A* theo cạnh tới `anchors_k` tâm nhu cầu gần đích nhất, lấy nhỏ nhất.
    fn exit_s(&mut self, ctx: &Ctx, p: Position, dest: LatLng) -> f64 {
        let k = self.policy.anchors_k;
        let mut near: Vec<(f64, Position)> = self
            .anchors(ctx)
            .iter()
            .map(|(ll, pos)| (haversine_m(*ll, dest), *pos))
            .collect();
        near.sort_by(|a, b| a.0.total_cmp(&b.0));
        near.truncate(k);
        let mut best = f64::INFINITY;
        for (_, pos) in near {
            if let Some(r) = ctx.route_car(Start::Pos(p), Goal::Pos(pos)) {
                best = best.min(f64::from(r.cost) / 1000.0);
            }
        }
        if best.is_finite() { best } else { 0.0 }
    }

    pub fn choose(
        &mut self,
        ctx: &Ctx,
        d: &DropView,
        log: &Rc<RefCell<DropLog>>,
    ) -> Option<Vec<Position>> {
        let engine = ctx.engine;
        let speed = ctx.scenario.customer.walk_speed_mps;
        let gate = nearest_gate(engine, d.dest_true, self.policy.gate_radius_m);
        let target = gate.unwrap_or(d.dest_true);
        let gen_out = self.gen_.generate(engine, target, d.dest_snap);
        let mut cands: Vec<Candidate> = gen_out.candidates;
        let default_ok: Vec<Candidate> = gen_out
            .pin
            .iter()
            .copied()
            .filter(|c| c.violation.is_none())
            .collect();
        cands.extend(default_ok.iter().copied());
        if cands.is_empty() {
            log_default(engine, d, target, gate.is_some(), speed, log);
            return None;
        }
        // lọc nhanh bằng ma trận: tài xế (một nguồn) → ứng viên
        let view = engine.view(Profile::Car);
        let ch = engine.ch(Profile::Car);
        let src = vec![d.from.source_legs(view)];
        let tgt: Vec<_> = cands.iter().map(|c| c.pos.target_legs(view)).collect();
        let m = bucket_matrix(ch, &src, &tgt, false);
        let mut pre: Vec<(f64, usize)> = cands
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let trip = m.get(0, i);
                let walk_m = if c.is_pin {
                    Ranker::walk_m(engine, target, c.point)
                } else {
                    c.walk_est_m
                };
                let j = if trip == INF {
                    f64::INFINITY
                } else {
                    self.policy.a_walk * walk_m / speed
                        + self.policy.d_trip * f64::from(trip) / 1000.0
                };
                (j, i)
            })
            .collect();
        pre.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut chosen: Vec<usize> = pre
            .iter()
            .filter(|(_, i)| !cands[*i].is_pin)
            .take(self.policy.refine_top)
            .map(|x| x.1)
            .collect();
        chosen.extend(pre.iter().filter(|(_, i)| cands[*i].is_pin).map(|x| x.1));

        let mut best: Option<(f64, Candidate, f64, f64)> = None; // (J, ứng viên, walk_m, exit_s)
        let mut j_default = f64::INFINITY;
        for &i in &chosen {
            let c = cands[i];
            let walk_m = Ranker::walk_m(engine, target, c.point);
            let trip = ctx
                .route_car(Start::Pos(d.from), Goal::Pos(c.pos))
                .map_or(f64::INFINITY, |r| f64::from(r.cost) / 1000.0);
            if !trip.is_finite() {
                continue;
            }
            let exit = if self.policy.use_exit {
                self.exit_s(ctx, c.pos, d.dest_true)
            } else {
                0.0
            };
            let mut pen = self.weights.tail_walk_penalty(walk_m);
            if self.policy.use_side {
                let (side, _) = side_of(engine, c.pos.edge, c.pos.frac, target);
                if side == Side::Left {
                    pen += self.weights.penalties.wrong_side_s
                        * side_scale(c.class, engine.g.lanes[c.pos.edge as usize]);
                }
            }
            let j = self.policy.a_walk * walk_m / speed
                + self.policy.d_trip * trip
                + self.policy.e_exit * exit
                + pen;
            if c.is_pin {
                j_default = j_default.min(j);
            }
            if best.as_ref().is_none_or(|b| j < b.0) {
                best = Some((j, c, walk_m, exit));
            }
        }
        let Some((j, c, walk_m, exit)) = best else {
            log_default(engine, d, target, gate.is_some(), speed, log);
            return None;
        };
        log.borrow_mut().entries.push(DropEntry {
            customer: d.customer,
            walk_m,
            chose_default: c.is_pin,
            gate: gate.is_some(),
            exit_s: exit,
            j_default,
            j_chosen: j,
            n_candidates: cands.len(),
        });
        if c.is_pin {
            return None;
        }
        Some(vec![c.pos])
    }
}

fn log_default(
    engine: &Engine,
    d: &DropView,
    target: LatLng,
    gate: bool,
    _speed: f64,
    log: &Rc<RefCell<DropLog>>,
) {
    log.borrow_mut().entries.push(DropEntry {
        customer: d.customer,
        walk_m: Ranker::walk_m(engine, target, d.dest_snap),
        chose_default: true,
        gate,
        exit_s: 0.0,
        j_default: f64::NAN,
        j_chosen: f64::NAN,
        n_candidates: 0,
    });
}

/// Baseline có ghi nhật ký điểm trả: đón đúng ghim như `AtPin`, trả đúng điểm đích đã snap, nhưng ghi lại quãng khách
/// phải đi bộ từ điểm trả tới đích thật (hoặc cổng) để so với chiến lược chọn điểm trả. Không đổi bất kỳ số nào của mô phỏng.
pub struct BaselineLogged {
    pub log: Rc<RefCell<DropLog>>,
    pub gate_radius_m: f64,
}

impl PickupStrategy for BaselineLogged {
    fn name(&self) -> &str {
        "at-pin"
    }
    fn choose(&mut self, ctx: &Ctx, c: &CustomerView) -> Option<PickupChoice> {
        AtPin.choose(ctx, c)
    }
    fn choose_dropoff(&mut self, ctx: &Ctx, d: &DropView) -> Option<Vec<Position>> {
        let gate = nearest_gate(ctx.engine, d.dest_true, self.gate_radius_m);
        let target = gate.unwrap_or(d.dest_true);
        log_default(ctx.engine, d, target, gate.is_some(), 0.0, &self.log);
        None
    }
}
