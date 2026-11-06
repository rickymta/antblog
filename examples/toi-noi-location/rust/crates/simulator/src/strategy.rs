//! Điểm cắm cho các bài sau: chọn điểm đón (`PickupStrategy`) và ghép chuyến (`MatchStrategy`).
//! Baseline của bài này: [`AtPin`] «đón đúng chỗ ghim» và [`NearestStraightLine`] «tài xế gần nhất theo đường chim bay».
//! LOC-32…38 thay `PickupStrategy`, LOC-41…47 thay `MatchStrategy`; simulator và chỉ số không đổi.

use crate::scenario::Scenario;
use crate::world::Router;
use geo_core::{LatLng, haversine_m};
use route_engine::engine::Engine;
use route_engine::position::{Position, Snapped};
use route_engine::profile::Profile;
use route_engine::turn::{EdgeRoute, Goal, Start};

/// Ngữ cảnh chỉ đọc mà chiến lược được phép dùng: engine, kịch bản, giờ mô phỏng và bộ định tuyến ô tô có chi phí rẽ.
pub struct Ctx<'a> {
    pub engine: &'a Engine,
    pub scenario: &'a Scenario,
    pub now_ms: u64,
    pub(crate) router: &'a Router<'a>,
}

impl Ctx<'_> {
    /// Tuyến ô tô theo cạnh có chi phí rẽ, theo trọng số của giờ hiện tại trong mô phỏng.
    pub fn route_car(&self, from: Start, to: Goal) -> Option<EdgeRoute> {
        self.router.route_car(self.now_ms, from, to)
    }
    /// Snap một điểm lên đường ô tô đi được, trong ngưỡng của engine.
    pub fn snap_car(&self, p: LatLng) -> Option<Snapped> {
        self.engine.snap_point(Profile::Car, p, "điểm").ok()
    }
    /// Quãng và thời gian đi bộ từ `from` tới `to` theo đồ thị đi bộ (mét, giây); chim bay × 1,3 nếu không snap được.
    pub fn walk(&self, from: LatLng, to: LatLng) -> (f64, f64) {
        let speed = self.scenario.customer.walk_speed_mps;
        match self.engine.route(Profile::Foot, from, to, None) {
            Ok(r) => (r.distance_m, r.distance_m / speed),
            Err(_) => {
                let d = haversine_m(from, to) * 1.3;
                (d, d / speed)
            }
        }
    }
    pub fn point_of(&self, p: Position) -> LatLng {
        p.point(&self.engine.g)
    }
}

/// Khách khi chọn điểm đón: chỗ đứng thật, điểm ghim trên app, điểm đến.
#[derive(Debug, Clone, Copy)]
pub struct CustomerView {
    pub id: u32,
    pub true_pos: LatLng,
    pub pin: LatLng,
    pub dropoff: LatLng,
}

/// Điểm đón đã chọn: tài xế dừng ở `stop_point` và vào được qua một trong các `approach` (vị trí trên cạnh có hướng,
/// nên đã gồm thông tin phía đường); khách đi bộ `walk_m` tới đó.
#[derive(Debug, Clone)]
pub struct PickupChoice {
    pub stop_point: LatLng,
    pub approach: Vec<Position>,
    pub walk_m: f64,
    pub walk_s: f64,
    /// Khoảng cách từ điểm ghim tới đường (chẩn đoán).
    pub pin_snap_m: f64,
}

pub trait PickupStrategy {
    fn name(&self) -> &str;
    /// `None` nếu không có điểm đón khả thi (khách bị tính huỷ vì không có đường).
    fn choose(&mut self, ctx: &Ctx, c: &CustomerView) -> Option<PickupChoice>;
}

/// Baseline: tài xế đến đúng điểm ghim (đã snap lên đường), khách đi bộ từ chỗ đứng thật tới đó.
#[derive(Debug, Default)]
pub struct AtPin;

impl PickupStrategy for AtPin {
    fn name(&self) -> &str {
        "at-pin"
    }
    fn choose(&mut self, ctx: &Ctx, c: &CustomerView) -> Option<PickupChoice> {
        let snapped = ctx.snap_car(c.pin)?;
        let view = ctx.engine.view(Profile::Car);
        let approach = snapped.positions(view);
        if approach.is_empty() {
            return None;
        }
        let (walk_m, walk_s) = ctx.walk(c.true_pos, snapped.point);
        Some(PickupChoice {
            stop_point: snapped.point,
            approach,
            walk_m,
            walk_s,
            pin_snap_m: snapped.snap_dist_m,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DriverView {
    pub id: u32,
    /// Vị trí có hướng hiện tại (cạnh và tỉ lệ), tức là tài xế đang quay mặt về phía nào.
    pub pos: Position,
    pub latlng: LatLng,
}

#[derive(Debug, Clone, Copy)]
pub struct PendingView<'a> {
    pub customer: u32,
    pub request_ms: u64,
    pub pickup: &'a PickupChoice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Assignment {
    pub customer: u32,
    pub driver: u32,
}

pub trait MatchStrategy {
    fn name(&self) -> &str;
    /// Gọi mỗi khi có khách mới hoặc có tài xế rảnh. `pending` đã sắp theo giờ đặt. Trả các cặp muốn gán;
    /// khách hoặc tài xế bị gán hai lần thì chỉ lần đầu có hiệu lực.
    fn assign(
        &mut self,
        ctx: &Ctx,
        pending: &[PendingView],
        idle: &[DriverView],
    ) -> Vec<Assignment>;
}

/// Baseline: duyệt khách theo thứ tự đặt, mỗi khách lấy tài xế rảnh gần nhất theo đường chim bay (hoà thì mã nhỏ hơn).
#[derive(Debug, Default)]
pub struct NearestStraightLine;

/// Lõi của baseline, tách ra để thử mà không cần dựng ngữ cảnh mô phỏng.
pub fn nearest_straight_line(pending: &[PendingView], idle: &[DriverView]) -> Vec<Assignment> {
    let mut taken = vec![false; idle.len()];
    let mut out = Vec::new();
    for p in pending {
        let mut best: Option<(f64, usize)> = None;
        for (i, d) in idle.iter().enumerate() {
            if taken[i] {
                continue;
            }
            let dist = haversine_m(d.latlng, p.pickup.stop_point);
            if best.is_none_or(|(b, j)| dist < b || (dist == b && d.id < idle[j].id)) {
                best = Some((dist, i));
            }
        }
        if let Some((_, i)) = best {
            taken[i] = true;
            out.push(Assignment {
                customer: p.customer,
                driver: idle[i].id,
            });
        }
    }
    out
}

impl MatchStrategy for NearestStraightLine {
    fn name(&self) -> &str {
        "nearest-straight-line"
    }
    fn assign(
        &mut self,
        _ctx: &Ctx,
        pending: &[PendingView],
        idle: &[DriverView],
    ) -> Vec<Assignment> {
        nearest_straight_line(pending, idle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ll(lat: f64, lng: f64) -> LatLng {
        LatLng::try_new(lat, lng).unwrap()
    }

    #[test]
    fn nearest_straight_line_picks_closest_idle_driver_and_never_reuses() {
        let pick = |lat: f64, lng: f64| PickupChoice {
            stop_point: ll(lat, lng),
            approach: Vec::new(),
            walk_m: 0.0,
            walk_s: 0.0,
            pin_snap_m: 0.0,
        };
        let (p0, p1) = (pick(10.770, 106.690), pick(10.771, 106.690));
        let pending = [
            PendingView {
                customer: 0,
                request_ms: 0,
                pickup: &p0,
            },
            PendingView {
                customer: 1,
                request_ms: 1,
                pickup: &p1,
            },
        ];
        let d = |id: u32, lat: f64| DriverView {
            id,
            pos: Position::new(0, 0.0),
            latlng: ll(lat, 106.690),
        };
        let idle = [d(7, 10.7701), d(3, 10.7705), d(9, 10.800)];
        let a = nearest_straight_line(&pending, &idle);
        assert_eq!(
            a,
            vec![
                Assignment {
                    customer: 0,
                    driver: 7
                },
                Assignment {
                    customer: 1,
                    driver: 3
                }
            ]
        );
    }
}
