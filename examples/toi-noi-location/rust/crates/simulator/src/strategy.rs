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
    /// Thống kê từng tài xế, chỉ mục theo `DriverView::id` (P05: công bằng cho tài xế, LOC-45).
    pub(crate) fleet: &'a [FleetStat],
}

/// Thống kê một tài xế tại thời điểm gọi chiến lược (cộng thêm, không đổi baseline).
#[derive(Debug, Clone, Copy, Default)]
pub struct FleetStat {
    /// Lúc tài xế rảnh gần nhất (0 nếu rảnh từ đầu mô phỏng).
    pub idle_since_ms: u64,
    pub trips_done: u32,
    /// Tổng thời gian đã bận (đón và chở khách), mili giây, chưa tính chuyến đang chạy.
    pub busy_ms: u64,
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
    /// Thống kê của tài xế `id` (mặc định toàn 0 nếu `id` ngoài phạm vi).
    pub fn fleet_stat(&self, id: u32) -> FleetStat {
        self.fleet.get(id as usize).copied().unwrap_or_default()
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
    /// P04: gọi ngay trước `choose` với các tài xế đang rảnh lúc khách đặt (vị trí có hướng). Mặc định bỏ qua,
    /// nên baseline và mọi chiến lược cũ không đổi.
    fn observe_idle(&mut self, _idle: &[DriverView]) {}
    /// P04, LOC-38: chọn điểm trả khi tài xế đã đón khách. `None` (mặc định) = baseline: xe tới đúng điểm đích đã snap.
    /// Trả về các vị trí có hướng mà xe được phép vào; mô phỏng chọn đường rẻ nhất tới một trong chúng.
    fn choose_dropoff(&mut self, _ctx: &Ctx, _d: &DropView) -> Option<Vec<Position>> {
        None
    }
}

/// Thông tin cho `choose_dropoff` (P04): đích thật, đích đã snap lên đường, và chỗ xe đang đứng.
#[derive(Debug, Clone)]
pub struct DropView {
    pub customer: u32,
    /// Điểm đến thật của khách (có thể cách đường tới vài chục mét).
    pub dest_true: LatLng,
    /// Điểm đến đã snap lên đường: nơi baseline trả khách.
    pub dest_snap: LatLng,
    /// Vị trí có hướng của xe lúc đón xong.
    pub from: Position,
    /// Các vị trí baseline sẽ dùng.
    pub default_positions: Vec<Position>,
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

    /// Cửa sổ lô (LOC-43). `None` (mặc định, baseline) = ghép ngay mỗi khi có khách mới hoặc tài xế rảnh.
    /// `Some(ms)` = khi có khách chờ và có tài xế rảnh mà chưa có lô đang mở, mở lô và ghép **một lần** sau `ms` mili giây;
    /// khách chưa được ghép ở lô đó chờ lô sau. Gọi lại mỗi lần cần quyết định, nên cửa sổ có thể thích ứng.
    fn window_ms(
        &mut self,
        _ctx: &Ctx,
        _pending: &[PendingView],
        _idle: &[DriverView],
    ) -> Option<u64> {
        None
    }

    /// Tầm nhìn của ghép nối chuyến (LOC-47): `Some(ms)` thì simulator đưa thêm các tài xế đang chở khách và sẽ trả khách
    /// trong vòng `ms` tới (ước lượng) vào `assign_chain`. `None` (mặc định) = chỉ tài xế rảnh.
    fn chain_horizon_ms(&self) -> Option<u64> {
        None
    }

    /// Ghép có cả tài xế sắp trả khách. Cặp (khách, tài xế đang bận) nghĩa là xếp khách vào hàng chờ của tài xế đó:
    /// tài xế tự đi đón ngay khi trả khách hiện tại. Mặc định bỏ qua `finishing` và gọi [`MatchStrategy::assign`].
    fn assign_chain(
        &mut self,
        ctx: &Ctx,
        pending: &[PendingView],
        idle: &[DriverView],
        _finishing: &[FinishingView],
    ) -> Vec<Assignment> {
        self.assign(ctx, pending, idle)
    }
}

/// Tài xế đang chở khách (LOC-47): ứng viên cho chuyến kế. Chỉ có **ước lượng** theo kế hoạch; thời gian thật còn nhiễu tốc độ
/// của mô phỏng (`driver.speed_jitter`) mà chiến lược không được biết.
#[derive(Debug, Clone, Copy)]
pub struct FinishingView {
    pub id: u32,
    /// Nơi tài xế sẽ trả khách (cạnh có hướng).
    pub end_pos: Position,
    pub end_latlng: LatLng,
    /// Còn bao lâu nữa tài xế rảnh (ước lượng theo kế hoạch, gồm thời gian dừng trả khách), mili giây.
    pub remaining_est_ms: u64,
}

/// Một gợi ý di chuyển cho tài xế rảnh (LOC-49). Chỉ là gợi ý: tài xế nghe theo với xác suất do kịch bản đặt.
#[derive(Debug, Clone, Copy)]
pub struct RepositionSuggestion {
    pub driver: u32,
    pub target: Position,
}

/// Điều hướng tài xế rảnh theo nhu cầu (LOC-49). Gắn qua [`crate::sim::run_with`]; không gắn thì không có gì đổi.
pub trait Repositioner {
    fn name(&self) -> &str;
    /// Chu kỳ gọi `suggest`, mili giây.
    fn interval_ms(&self) -> u64;
    fn suggest(
        &mut self,
        ctx: &Ctx,
        idle: &[DriverView],
        pending: &[PendingView],
    ) -> Vec<RepositionSuggestion>;
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
