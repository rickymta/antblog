//! Lõi mô phỏng sự kiện rời rạc. Một luồng, hàng đợi sự kiện theo (giờ, số thứ tự), không có `HashMap` duyệt thứ tự ngẫu nhiên,
//! nên cùng kịch bản, cùng seed và cùng chiến lược luôn cho cùng từng byte.

use crate::metrics::{Outcome, Summary, TripRecord, fnv1a, summarize, trips_csv};
use crate::rng::Rng;
use crate::scenario::{CancelBasis, Scenario};
use crate::strategy::{
    Assignment, Ctx, CustomerView, DriverView, DropView, FinishingView, FleetStat, MatchStrategy,
    PendingView, PickupChoice, PickupStrategy, Repositioner,
};
use crate::world::{Router, generate};
use geo_core::LatLng;
use road_graph::EdgeId;
use route_engine::engine::Engine;
use route_engine::position::Position;
use route_engine::turn::{EdgeRoute, Goal, Start};
use serde::Serialize;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Tuyến đang chạy: thời điểm mỗi cạnh kết thúc (gồm thời gian chờ rẽ ở đầu cạnh), có nhiễu tốc độ của chặng.
#[derive(Debug, Clone)]
pub struct ActiveRoute {
    pub start_ms: u64,
    pub edges: Vec<EdgeId>,
    from_frac: Vec<f32>,
    to_frac: Vec<f32>,
    /// Từ lúc bắt đầu: thời điểm hết chờ rẽ ở đầu cạnh `i`.
    wait_end_ms: Vec<u64>,
    /// Từ lúc bắt đầu: thời điểm tới cuối cạnh `i`.
    end_ms: Vec<u64>,
    /// Quãng đường (mét) tính tới cuối cạnh `i`.
    cum_m: Vec<f64>,
    pub planned_ms: u64,
    pub total_ms: u64,
    /// P06: thời gian chạy khi chỉ có hiệu ứng «thế giới thật» của [`Truth`], chưa có nhiễu của chặng (`factor`). Không có `Truth`
    /// thì bằng `planned_ms`. Chỉ để lab P06 tính sàn nhiễu; không dùng trong mô phỏng.
    pub noiseless_ms: u64,
}

impl ActiveRoute {
    pub fn build(
        engine: &Engine,
        tm: &route_engine::turn::TurnModel,
        route: &EdgeRoute,
        start_ms: u64,
        factor: f64,
    ) -> Self {
        Self::build_with(engine, tm, route, start_ms, factor, None)
    }

    /// Như [`ActiveRoute::build`], thêm hệ số «thế giới thật» của từng cạnh (P06). `truth = None` cho đúng từng bit kết quả của `build`
    /// (nhân với 1,0 không đổi giá trị).
    pub fn build_with(
        engine: &Engine,
        tm: &route_engine::turn::TurnModel,
        route: &EdgeRoute,
        start_ms: u64,
        factor: f64,
        truth: Option<&dyn Truth>,
    ) -> Self {
        let g = &engine.g;
        let n = route.edges.len();
        let (mut from_frac, mut to_frac) = (Vec::with_capacity(n), Vec::with_capacity(n));
        let (mut wait_end, mut end, mut cum_m) = (Vec::new(), Vec::new(), Vec::new());
        let (mut t, mut dist) = (0f64, 0f64);
        let mut planned = 0f64;
        let mut noiseless = 0f64;
        for (i, &e) in route.edges.iter().enumerate() {
            let a = if i == 0 { route.start_frac } else { 0.0 };
            let b = if i + 1 == n { route.end_frac } else { 1.0 };
            from_frac.push(a);
            to_frac.push(b);
            let wait = if i > 0 {
                f64::from(tm.turn_ms(route.edges[i - 1], e).unwrap_or(0))
            } else {
                0.0
            };
            let mv = f64::from(tm.view.w[e as usize]) * f64::from((b - a).max(0.0));
            planned += wait + mv;
            // hệ số thế giới thật của cạnh, tính theo giờ lúc xe vào cạnh (1,0 khi không có `Truth`)
            let ef = truth.map_or(1.0, |tr| tr.edge_factor(engine, e, start_ms + t.round() as u64));
            noiseless += (wait + mv) * ef;
            t += (wait + mv) * factor * ef;
            wait_end.push((t - mv * factor * ef).round() as u64);
            end.push(t.round() as u64);
            dist += f64::from(g.length_cm[e as usize]) / 100.0 * f64::from((b - a).max(0.0));
            cum_m.push(dist);
        }
        Self {
            start_ms,
            edges: route.edges.clone(),
            from_frac,
            to_frac,
            wait_end_ms: wait_end,
            end_ms: end,
            cum_m,
            planned_ms: planned.round() as u64,
            total_ms: t.round() as u64,
            noiseless_ms: noiseless.round() as u64,
        }
    }

    pub fn length_m(&self) -> f64 {
        self.cum_m.last().copied().unwrap_or(0.0)
    }

    pub fn end_position(&self) -> Position {
        let i = self.edges.len() - 1;
        Position::new(self.edges[i], self.to_frac[i])
    }

    fn segment_at(&self, off: u64) -> usize {
        self.end_ms
            .partition_point(|&e| e < off)
            .min(self.edges.len() - 1)
    }

    /// Vị trí tại thời điểm `t_ms` (kẹp ở hai đầu tuyến). Trong lúc chờ rẽ, xe đứng ở đầu cạnh sắp vào.
    pub fn position_at(&self, t_ms: u64) -> Position {
        let off = t_ms.saturating_sub(self.start_ms);
        let i = self.segment_at(off);
        let (a, b) = (self.from_frac[i], self.to_frac[i]);
        if off >= self.end_ms[i] {
            return Position::new(self.edges[i], b);
        }
        if off <= self.wait_end_ms[i] {
            return Position::new(self.edges[i], a);
        }
        let move_ms = (self.end_ms[i] - self.wait_end_ms[i]).max(1) as f32;
        let f = (off - self.wait_end_ms[i]) as f32 / move_ms;
        Position::new(self.edges[i], a + (b - a) * f)
    }

    /// Quãng đường (mét) đã đi được tới `t_ms`.
    pub fn distance_until(&self, t_ms: u64) -> f64 {
        let off = t_ms.saturating_sub(self.start_ms);
        let i = self.segment_at(off);
        let p = self.position_at(t_ms);
        let before = if i == 0 { 0.0 } else { self.cum_m[i - 1] };
        let (a, b) = (self.from_frac[i], self.to_frac[i]);
        if b > a {
            before + (self.cum_m[i] - before) * f64::from((p.frac - a).max(0.0) / (b - a))
        } else {
            before
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ev {
    Request(u32),
    DriverArrives(u32),
    PickupDone(u32),
    TripEnds(u32),
    Cancel(u32),
    Patience(u32),
    /// Hết cửa sổ lô (P05, LOC-43): ghép một lần.
    Tick,
    /// Khách đã xếp hàng chờ tài xế đang bận huỷ (P05, LOC-47).
    CancelQueued(u32),
    /// Chu kỳ gợi ý di chuyển cho tài xế rảnh (P05, LOC-49).
    RepoTick,
    RepoDone(u32, u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CStatus {
    NotYet,
    Pending,
    Assigned,
    /// Đã xếp hàng chờ một tài xế đang chở khách (P05, LOC-47).
    Queued,
    Closed,
}

struct Cust {
    view: CustomerView,
    request_ms: u64,
    dropoff: Vec<Position>,
    /// Điểm đến thật chưa snap (P04).
    dest_true: LatLng,
    status: CStatus,
    pickup: Option<PickupChoice>,
    queued_ms: u64,
    queued_driver: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DState {
    Idle,
    ToPickup(u32),
    WithPassenger(u32),
}

/// Tài xế đang di chuyển theo gợi ý (P05, LOC-49). Vẫn tính là rảnh: có khách thì đổi sang đón ngay.
struct Repos {
    id: u32,
    route: ActiveRoute,
}

struct Drv {
    pos: Position,
    state: DState,
    busy_since_ms: u64,
    idle_since_ms: u64,
    trips_done: u32,
    busy_total_ms: u64,
    /// Giờ (mô phỏng) tài xế rảnh khi đang chở khách, gồm thời gian dừng trả khách (thật, có nhiễu).
    free_at_ms: u64,
    /// Khách xếp hàng chờ chuyến kế (P05, LOC-47).
    queued: Option<u32>,
    repos: Option<Repos>,
}

struct Trip {
    customer: u32,
    driver: u32,
    assign_ms: u64,
    to_pickup: ActiveRoute,
    cust_arrive_ms: u64,
    driver_arrive_ms: u64,
    arrived: bool,
    meet_gap_s: f64,
    walk_m: f64,
    deadhead_m: f64,
    u_turns: u32,
    trip: Option<ActiveRoute>,
    cancelled: bool,
}

/// Một dòng nhật ký sự kiện (chỉ ghi khi `RunOptions::trace`).
#[derive(Debug, Clone, Serialize)]
pub struct TraceEvent {
    pub t_s: f64,
    pub kind: &'static str,
    pub customer: Option<u32>,
    pub driver: Option<u32>,
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RunOptions {
    pub trace: bool,
}

/// «Thế giới thật» của P06: hệ số nhân thời gian chạy của từng cạnh mà route-engine **không biết** (hiệu ứng vùng, giờ, ngày).
/// Mô phỏng dùng nó để quyết định xe thật sự mất bao lâu; mọi ETA mà chiến lược nhìn thấy vẫn là ước lượng của route-engine.
pub trait Truth: Send {
    /// Hệ số nhân thời gian chạy của cạnh `edge` khi xe vào cạnh lúc `sim_ms` (mili giây mô phỏng); 1,0 = đúng như route-engine tin.
    fn edge_factor(&self, engine: &Engine, edge: EdgeId, sim_ms: u64) -> f64;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegKind {
    /// Tài xế chạy không tới điểm đón.
    Pickup,
    /// Đoạn có khách, từ điểm đón tới điểm trả.
    Onboard,
}

/// Một chặng xe vừa bắt đầu (P06): để lab xuất chuyến và GPS. `route` gồm cả `planned_ms` (ETA của route-engine) và `total_ms` (thật).
pub struct LegInfo<'a> {
    pub kind: LegKind,
    pub customer: u32,
    pub driver: u32,
    pub start_ms: u64,
    pub route: &'a ActiveRoute,
}

/// Người quan sát chỉ đọc (P06): không ảnh hưởng mô phỏng.
pub trait Observer: Send {
    fn leg_started(&mut self, leg: &LegInfo);
    /// Khách huỷ khi xe còn đang chạy tới điểm đón: chặng đón dừng ở `at_ms`.
    fn leg_cancelled(&mut self, customer: u32, at_ms: u64);
}

/// Phần cắm thêm của P06 cho [`run_hooked`]; mặc định không có gì nên baseline giữ nguyên từng byte.
#[derive(Default)]
pub struct Hooks<'a> {
    pub truth: Option<Box<dyn Truth + 'a>>,
    pub observer: Option<Box<dyn Observer + 'a>>,
}

/// Số đếm phụ của P05 (cửa sổ lô, ghép nối chuyến, gợi ý di chuyển). Toàn 0 ở baseline.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ExtraStats {
    /// Số lần cửa sổ lô đóng lại và ghép.
    pub batches: usize,
    /// Khách được xếp hàng chờ tài xế đang chở khách.
    pub chained: usize,
    /// Trong số đó, khách huỷ khi còn đang xếp hàng.
    pub chain_cancelled_in_queue: usize,
    pub reposition_suggested: usize,
    pub reposition_accepted: usize,
    /// Gợi ý bị bỏ dở vì tài xế được gán khách trước khi tới nơi.
    pub reposition_interrupted: usize,
    /// Tổng quãng tài xế chạy không theo gợi ý (mét), gồm cả quãng chạy dở.
    pub cruise_m: f64,
}

/// Phần cắm thêm (P05) cho [`run_with`]; mặc định không có gì nên [`run`] giữ nguyên hành vi baseline.
#[derive(Default)]
pub struct Extras<'a> {
    pub repositioner: Option<Box<dyn Repositioner + 'a>>,
    /// Xác suất tài xế nghe theo một gợi ý di chuyển (rút từ dòng ngẫu nhiên riêng).
    pub reposition_accept_prob: f64,
}

pub struct SimOutput {
    pub scenario: Scenario,
    pub pickup_strategy: String,
    pub match_strategy: String,
    pub summary: Summary,
    pub trips: Vec<TripRecord>,
    pub trace: Vec<TraceEvent>,
    pub extras: ExtraStats,
}

impl SimOutput {
    pub fn trips_csv(&self) -> String {
        trips_csv(&self.trips)
    }
    pub fn summary_json(&self) -> String {
        serde_json::to_string_pretty(&self.summary).expect("serialize")
    }
    pub fn trace_csv(&self) -> String {
        let mut s = String::from("t_s,kind,customer,driver,lat,lng\n");
        for e in &self.trace {
            s += &format!(
                "{:.1},{},{},{},{:.6},{:.6}\n",
                e.t_s,
                e.kind,
                e.customer.map_or("-".into(), |x| x.to_string()),
                e.driver.map_or("-".into(), |x| x.to_string()),
                e.lat,
                e.lng
            );
        }
        s
    }

    /// Báo cáo văn bản tất định (định dạng số cố định). Kết thúc bằng mã băm FNV-1a của nhật ký chuyến.
    pub fn report(&self) -> String {
        let (sc, s) = (&self.scenario, &self.summary);
        let d = |x: &crate::metrics::Dist| {
            format!(
                "trung bình {:>8.1}  p50 {:>8.1}  p95 {:>8.1}  max {:>8.1}  (n = {})",
                x.mean, x.p50, x.p95, x.max, x.n
            )
        };
        let mut r = String::new();
        r += &format!(
            "kịch bản: {}  (đồ thị: {}, seed {}, bắt đầu lúc {:02}h)\n",
            sc.name, sc.graph, sc.seed, sc.start_hour
        );
        r += &format!(
            "chiến lược: điểm đón = {}, ghép chuyến = {}; tài xế {}, khách đặt trong {:.0} s, tốc độ đặt xe cơ sở {:.2}/phút, giao thông: {:?}\n",
            self.pickup_strategy,
            self.match_strategy,
            sc.drivers,
            sc.duration_s,
            sc.demand.requests_per_min,
            sc.traffic
        );
        r += &format!(
            "khách đặt {}: hoàn thành {}, huỷ sau khi gán {}, huỷ vì hết kiên nhẫn chưa có xe {}, huỷ vì không có đường {}\n",
            s.requests,
            s.completed,
            s.cancelled_after_assign,
            s.cancelled_no_driver,
            s.cancelled_no_route
        );
        r += &format!(
            "mô phỏng kết thúc ở {:.0} s; tài xế bận trung bình {:.1}% thời gian\n\n",
            s.sim_end_s,
            100.0 * s.driver_busy_share
        );
        r += "chỉ số (§3.4), tính trên các chuyến hoàn thành:\n";
        r += &format!("  walk_m        {}\n", d(&s.walk_m));
        r += &format!("  deadhead_m    {}\n", d(&s.deadhead_m));
        r += &format!("  pickup_eta_s  {}\n", d(&s.pickup_eta_s));
        r += &format!("  meet_gap_s    {}\n", d(&s.meet_gap_s));
        r += &format!("  trip_m        {}\n", d(&s.trip_m));
        r += &format!(
            "  u_turns       tổng {}, trung bình {:.3} mỗi chuyến\n",
            s.u_turns_total, s.u_turns_per_trip
        );
        r += &format!(
            "  cancel_rate   {:.4}  ({} huỷ trên {} khách đặt)\n",
            s.cancel_rate,
            s.cancelled_after_assign + s.cancelled_no_driver + s.cancelled_no_route,
            s.requests
        );
        r += "chỉ số phụ:\n";
        r += &format!(
            "  wait_s        {}   (từ lúc đặt tới lúc tài xế tới)\n",
            d(&s.wait_s)
        );
        r += &format!(
            "  pin_snap_m    {}   (ghim → đường, chẩn đoán)\n",
            d(&s.pin_snap_m)
        );
        r += &format!(
            "  tổng quãng: chạy không của chuyến hoàn thành {:.0} m, có khách {:.0} m, tỉ lệ chạy không {:.4}; chạy không bỏ phí do huỷ {:.0} m\n",
            s.deadhead_total_m, s.trip_total_m, s.deadhead_share, s.wasted_deadhead_m
        );
        r += &format!(
            "nhật ký chuyến: {} dòng, FNV-1a = {:016x}\n",
            self.trips.len(),
            fnv1a(self.trips_csv().as_bytes())
        );
        r
    }

    /// Các giả định cần nêu cùng mọi con số.
    pub fn assumptions(&self) -> String {
        let c = &self.scenario.cancel;
        format!(
            "Giả định (mô phỏng, không đo từ đời thật): khách đứng cách ghim một khoảng ngẫu nhiên σ = {:.0} m mỗi trục; xe chạy theo tốc độ tự do của bảng theo loại đường{} với chi phí rẽ của LOC-23 (rẽ phải 3 s, rẽ trái 6 s, quay đầu giữa đường bị cấm); \
             nhiễu thời gian chạy mỗi chặng log-chuẩn σ = {:.2}; huỷ chuyến: p = {:.2} × clamp((eta − {:.0}) / ({:.0} − {:.0}), 0, 1) + {:.3} × walk_m/100, khách huỷ sau {:.0}–{:.0} s nếu tài xế chưa tới, và huỷ nếu quá {:.0} s chưa có xe. Mô hình huỷ chưa hiệu chuẩn bằng dữ liệu thật.",
            self.scenario.customer.pin_sigma_m,
            if self.scenario.traffic == crate::scenario::Traffic::Synthetic {
                " nhân hồ sơ giờ tổng hợp"
            } else {
                ""
            },
            self.scenario.driver.speed_jitter,
            c.max_prob,
            c.eta_free_s,
            c.eta_max_s,
            c.eta_free_s,
            c.walk_extra_per_100m,
            c.reaction_min_s,
            c.reaction_max_s,
            self.scenario.customer.patience_s
        )
    }
}

struct Sim<'a> {
    engine: &'a Engine,
    sc: &'a Scenario,
    router: Router<'a>,
    pickup: Box<dyn PickupStrategy + 'a>,
    matcher: Box<dyn MatchStrategy + 'a>,
    custs: Vec<Cust>,
    drivers: Vec<Drv>,
    trips: Vec<Trip>,
    heap: BinaryHeap<Reverse<(u64, u64)>>,
    events: Vec<Ev>,
    now_ms: u64,
    rng_cancel: Rng,
    rng_jitter: Rng,
    records: Vec<TripRecord>,
    busy_ms: u64,
    trace: Vec<TraceEvent>,
    opts: RunOptions,
    tick_open: bool,
    in_tick: bool,
    extras: ExtraStats,
    repositioner: Option<Box<dyn Repositioner + 'a>>,
    reposition_accept_prob: f64,
    rng_repo: Rng,
    repo_seq: u32,
    truth: Option<Box<dyn Truth + 'a>>,
    observer: Option<Box<dyn Observer + 'a>>,
}

impl Sim<'_> {
    fn push(&mut self, t_ms: u64, ev: Ev) {
        let seq = self.events.len() as u64;
        self.events.push(ev);
        self.heap.push(Reverse((t_ms, seq)));
    }

    fn latlng(&self, p: Position) -> LatLng {
        p.point(&self.engine.g)
    }

    fn trace(
        &mut self,
        kind: &'static str,
        customer: Option<u32>,
        driver: Option<u32>,
        at: LatLng,
    ) {
        if self.opts.trace {
            self.trace.push(TraceEvent {
                t_s: self.now_ms as f64 / 1000.0,
                kind,
                customer,
                driver,
                lat: at.lat(),
                lng: at.lng(),
            });
        }
    }

    fn jitter(&mut self) -> f64 {
        let s = self.sc.driver.speed_jitter;
        // luôn rút một số để dòng ngẫu nhiên không phụ thuộc vào việc bật nhiễu
        let z = self.rng_jitter.gauss();
        if s > 0.0 { (s * z).exp() } else { 1.0 }
    }

    fn close_customer(
        &mut self,
        c: u32,
        outcome: Outcome,
        driver: Option<u32>,
        rec: Option<TripRecord>,
    ) {
        let cu = &mut self.custs[c as usize];
        cu.status = CStatus::Closed;
        let pin_snap_m = cu.pickup.as_ref().map_or(0.0, |p| p.pin_snap_m);
        let request_s = cu.request_ms as f64 / 1000.0;
        let walk_m = cu.pickup.as_ref().map_or(0.0, |p| p.walk_m);
        self.records.push(rec.unwrap_or(TripRecord {
            customer: c,
            driver,
            request_s,
            outcome,
            walk_m,
            deadhead_m: 0.0,
            pickup_eta_s: 0.0,
            wait_s: 0.0,
            meet_gap_s: 0.0,
            trip_m: 0.0,
            trip_s: 0.0,
            u_turns: 0,
            pin_snap_m,
        }));
    }

    fn current_pos(&self, d: &Drv) -> Position {
        d.repos
            .as_ref()
            .map_or(d.pos, |r| r.route.position_at(self.now_ms))
    }

    fn idle_views(&self) -> Vec<DriverView> {
        self.drivers
            .iter()
            .enumerate()
            .filter(|(_, d)| d.state == DState::Idle)
            .map(|(i, d)| {
                let pos = self.current_pos(d);
                DriverView {
                    id: i as u32,
                    pos,
                    latlng: self.latlng(pos),
                }
            })
            .collect()
    }

    fn fleet_stats(&self) -> Vec<FleetStat> {
        self.drivers
            .iter()
            .map(|d| FleetStat {
                idle_since_ms: d.idle_since_ms,
                trips_done: d.trips_done,
                busy_ms: d.busy_total_ms,
            })
            .collect()
    }

    /// Tài xế đang chở khách và sẽ rảnh trong `chain_horizon_ms` tới (ước lượng theo kế hoạch, không biết nhiễu tốc độ).
    fn finishing_views(&self) -> Vec<FinishingView> {
        let Some(horizon) = self.matcher.chain_horizon_ms() else {
            return Vec::new();
        };
        let dwell = (self.sc.driver.dwell_dropoff_s * 1000.0) as u64;
        let mut out = Vec::new();
        for (i, d) in self.drivers.iter().enumerate() {
            let DState::WithPassenger(tid) = d.state else {
                continue;
            };
            if d.queued.is_some() {
                continue;
            }
            let Some(a) = self.trips[tid as usize].trip.as_ref() else {
                continue;
            };
            let left_true = d.free_at_ms.saturating_sub(self.now_ms);
            let dwell_left = left_true.min(dwell);
            let drive_true = (left_true - dwell_left) as f64;
            let est = (drive_true * a.planned_ms as f64 / a.total_ms.max(1) as f64).round() as u64
                + dwell_left;
            if est <= horizon {
                let end_pos = a.end_position();
                out.push(FinishingView {
                    id: i as u32,
                    end_pos,
                    end_latlng: self.latlng(end_pos),
                    remaining_est_ms: est,
                });
            }
        }
        out
    }

    /// Cửa sổ lô: nếu chiến lược muốn gom thì mở lô (một `Tick`) và báo đã hoãn việc ghép.
    fn maybe_open_window(&mut self) -> bool {
        if self.tick_open {
            return true;
        }
        let idle = self.idle_views();
        let finishing = self.finishing_views();
        if idle.is_empty() && finishing.is_empty() {
            return false;
        }
        let mut pending_ids: Vec<u32> = (0..self.custs.len() as u32)
            .filter(|&c| self.custs[c as usize].status == CStatus::Pending)
            .collect();
        if pending_ids.is_empty() {
            return false;
        }
        pending_ids.sort_by_key(|&c| (self.custs[c as usize].request_ms, c));
        let w = {
            let pending: Vec<PendingView> = pending_ids
                .iter()
                .map(|&c| PendingView {
                    customer: c,
                    request_ms: self.custs[c as usize].request_ms,
                    pickup: self.custs[c as usize]
                        .pickup
                        .as_ref()
                        .expect("đang chờ thì có điểm đón"),
                })
                .collect();
            let fleet = self.fleet_stats();
            let ctx = Ctx {
                engine: self.engine,
                scenario: self.sc,
                now_ms: self.now_ms,
                router: &self.router,
                fleet: &fleet,
            };
            self.matcher.window_ms(&ctx, &pending, &idle)
        };
        match w {
            Some(ms) => {
                self.tick_open = true;
                self.push(self.now_ms + ms.max(1), Ev::Tick);
                true
            }
            None => false,
        }
    }

    fn try_match(&mut self) {
        if !self.in_tick && self.maybe_open_window() {
            return;
        }
        loop {
            let idle = self.idle_views();
            let finishing = self.finishing_views();
            if idle.is_empty() && finishing.is_empty() {
                return;
            }
            let mut pending_ids: Vec<u32> = (0..self.custs.len() as u32)
                .filter(|&c| self.custs[c as usize].status == CStatus::Pending)
                .collect();
            pending_ids.sort_by_key(|&c| (self.custs[c as usize].request_ms, c));
            if pending_ids.is_empty() {
                return;
            }
            let assignments: Vec<Assignment> = {
                let pending: Vec<PendingView> = pending_ids
                    .iter()
                    .map(|&c| PendingView {
                        customer: c,
                        request_ms: self.custs[c as usize].request_ms,
                        pickup: self.custs[c as usize]
                            .pickup
                            .as_ref()
                            .expect("đang chờ thì có điểm đón"),
                    })
                    .collect();
                let fleet = self.fleet_stats();
                let ctx = Ctx {
                    engine: self.engine,
                    scenario: self.sc,
                    now_ms: self.now_ms,
                    router: &self.router,
                    fleet: &fleet,
                };
                if self.matcher.chain_horizon_ms().is_some() {
                    self.matcher.assign_chain(&ctx, &pending, &idle, &finishing)
                } else {
                    self.matcher.assign(&ctx, &pending, &idle)
                }
            };
            let mut progressed = false;
            for a in assignments {
                if self
                    .custs
                    .get(a.customer as usize)
                    .is_none_or(|c| c.status != CStatus::Pending)
                {
                    continue;
                }
                match self.drivers.get(a.driver as usize).map(|d| d.state) {
                    Some(DState::Idle) => {
                        self.start_trip(a.customer, a.driver, None);
                        progressed = true;
                    }
                    Some(DState::WithPassenger(_))
                        if self.matcher.chain_horizon_ms().is_some()
                            && self.drivers[a.driver as usize].queued.is_none() =>
                    {
                        progressed |= self.queue_chain(a.customer, a.driver);
                    }
                    _ => {}
                }
            }
            // nếu một khách không có đường tới (bị đóng), tài xế vẫn rảnh: thử lại với phần còn lại
            if !progressed {
                return;
            }
            // vòng lặp tiếp để ghép nốt phần còn lại nếu chiến lược chỉ gán một phần
            if self.custs.iter().all(|c| c.status != CStatus::Pending) {
                return;
            }
        }
    }

    /// Xếp khách `c` vào hàng chờ của tài xế đang chở khách `d` (LOC-47). `false` nếu không có đường tới điểm đón.
    fn queue_chain(&mut self, c: u32, d: u32) -> bool {
        let now = self.now_ms;
        let Some(fv) = self.finishing_views().into_iter().find(|f| f.id == d) else {
            return false;
        };
        let pickup = self.custs[c as usize].pickup.clone().expect("có điểm đón");
        let mut best: Option<EdgeRoute> = None;
        for &ap in &pickup.approach {
            if let Some(r) = self
                .router
                .route_car(now, Start::Pos(fv.end_pos), Goal::Pos(ap))
                && best.as_ref().is_none_or(|b| r.cost < b.cost)
            {
                best = Some(r);
            }
        }
        let Some(route) = best else {
            return false;
        };
        let tm = self.router.turn_model(now);
        let planned = ActiveRoute::build(self.engine, &tm, &route, now, 1.0).planned_ms;
        let waited_s = match self.sc.cancel.basis {
            CancelBasis::Eta => 0.0,
            CancelBasis::TotalWait => (now - self.custs[c as usize].request_ms) as f64 / 1000.0,
        };
        let eta_planned_s = (fv.remaining_est_ms + planned) as f64 / 1000.0 + waited_s;
        // mô hình huỷ như start_trip: rút (u, phản ứng) ngay lúc gán, vì khách huỷ được cả khi còn xếp hàng
        let (u, react) = (self.rng_cancel.f64(), self.rng_cancel.f64());
        let cn = &self.sc.cancel;
        let p = cn.probability(eta_planned_s, pickup.walk_m);
        let cancel_at = now
            + ((cn.reaction_min_s + react * (cn.reaction_max_s - cn.reaction_min_s)) * 1000.0)
                .round() as u64;
        let expected_arrival = now + fv.remaining_est_ms + planned;
        {
            let cu = &mut self.custs[c as usize];
            cu.status = CStatus::Queued;
            cu.queued_ms = now;
            cu.queued_driver = Some(d);
        }
        self.drivers[d as usize].queued = Some(c);
        self.extras.chained += 1;
        let at = self.latlng(fv.end_pos);
        self.trace("chain_queue", Some(c), Some(d), at);
        if u < p && cancel_at < expected_arrival {
            self.push(cancel_at, Ev::CancelQueued(c));
        }
        true
    }

    /// `chain_assign_ms`: khách đã xếp hàng từ lúc đó (đã rút xác suất huỷ lúc xếp hàng); `None` = gán ngay như baseline.
    fn start_trip(&mut self, c: u32, d: u32, chain_assign_ms: Option<u64>) {
        let now = self.now_ms;
        // đang di chuyển theo gợi ý (LOC-49): dừng lại tại chỗ, ghi quãng đã chạy
        if let Some(r) = self.drivers[d as usize].repos.take() {
            self.extras.cruise_m += r.route.distance_until(now);
            self.extras.reposition_interrupted += 1;
            self.drivers[d as usize].pos = r.route.position_at(now);
        }
        let driver_pos = self.drivers[d as usize].pos;
        let pickup = self.custs[c as usize].pickup.clone().expect("có điểm đón");
        // đường rẻ nhất tới một trong các hướng vào điểm đón
        let mut best: Option<EdgeRoute> = None;
        for &ap in &pickup.approach {
            if let Some(r) = self
                .router
                .route_car(now, Start::Pos(driver_pos), Goal::Pos(ap))
                && best.as_ref().is_none_or(|b| r.cost < b.cost)
            {
                best = Some(r);
            }
        }
        let Some(route) = best else {
            self.close_customer(c, Outcome::CancelledNoRoute, Some(d), None);
            return;
        };
        let factor = self.jitter();
        let tm = self.router.turn_model(now);
        let active = ActiveRoute::build_with(
            self.engine,
            &tm,
            &route,
            now,
            factor,
            self.truth.as_deref(),
        );
        let u_turns = tm.u_turns(&route.edges);
        let eta_planned_s = active.planned_ms as f64 / 1000.0;
        let walk_m = pickup.walk_m;
        let cust_arrive_ms =
            self.custs[c as usize].request_ms + (pickup.walk_s * 1000.0).round() as u64;

        // huỷ chuyến: rút hai số để dòng ngẫu nhiên ổn định (khách xếp hàng đã rút lúc xếp hàng)
        let (u, react) = if chain_assign_ms.is_some() {
            (1.0, 0.0)
        } else {
            (self.rng_cancel.f64(), self.rng_cancel.f64())
        };
        let cn = &self.sc.cancel;
        let waited_s = match cn.basis {
            CancelBasis::Eta => 0.0,
            CancelBasis::TotalWait => (now - self.custs[c as usize].request_ms) as f64 / 1000.0,
        };
        let p = cn.probability(eta_planned_s + waited_s, walk_m);
        let arrive_ms = now + active.total_ms;
        let cancel_at = now
            + ((cn.reaction_min_s + react * (cn.reaction_max_s - cn.reaction_min_s)) * 1000.0)
                .round() as u64;

        let tid = self.trips.len() as u32;
        let deadhead_m = active.length_m();
        self.trips.push(Trip {
            customer: c,
            driver: d,
            assign_ms: chain_assign_ms.unwrap_or(now),
            to_pickup: active,
            cust_arrive_ms,
            driver_arrive_ms: 0,
            arrived: false,
            meet_gap_s: 0.0,
            walk_m,
            deadhead_m,
            u_turns,
            trip: None,
            cancelled: false,
        });
        self.custs[c as usize].status = CStatus::Assigned;
        if let Some(o) = self.observer.as_mut() {
            o.leg_started(&LegInfo {
                kind: LegKind::Pickup,
                customer: c,
                driver: d,
                start_ms: now,
                route: &self.trips[tid as usize].to_pickup,
            });
        }
        let drv = &mut self.drivers[d as usize];
        drv.state = DState::ToPickup(tid);
        drv.busy_since_ms = now;
        let at = self.latlng(driver_pos);
        self.trace(
            if chain_assign_ms.is_some() {
                "chain_start"
            } else {
                "assign"
            },
            Some(c),
            Some(d),
            at,
        );
        self.push(arrive_ms, Ev::DriverArrives(tid));
        if u < p && cancel_at < arrive_ms {
            self.push(cancel_at, Ev::Cancel(tid));
        }
    }

    fn release_driver(&mut self, d: u32, pos: Position) {
        let drv = &mut self.drivers[d as usize];
        self.busy_ms += self.now_ms - drv.busy_since_ms;
        drv.busy_total_ms += self.now_ms - drv.busy_since_ms;
        drv.idle_since_ms = self.now_ms;
        drv.pos = pos;
        drv.state = DState::Idle;
    }

    fn handle(&mut self, ev: Ev) {
        match ev {
            Ev::Request(c) => {
                let view = self.custs[c as usize].view;
                // P04: chiến lược điểm đón được biết các tài xế rảnh (mặc định bỏ qua, baseline không đổi)
                let idle_now = self.idle_views();
                self.pickup.observe_idle(&idle_now);
                let choice = {
                    let fleet = self.fleet_stats();
                    let ctx = Ctx {
                        engine: self.engine,
                        scenario: self.sc,
                        now_ms: self.now_ms,
                        router: &self.router,
                        fleet: &fleet,
                    };
                    self.pickup.choose(&ctx, &view)
                };
                self.trace("request", Some(c), None, view.pin);
                match choice {
                    None => {
                        self.close_customer(c, Outcome::CancelledNoRoute, None, None);
                    }
                    Some(ch) => {
                        let cu = &mut self.custs[c as usize];
                        cu.pickup = Some(ch);
                        cu.status = CStatus::Pending;
                        let t = self.now_ms + (self.sc.customer.patience_s * 1000.0) as u64;
                        self.push(t, Ev::Patience(c));
                        self.try_match();
                    }
                }
            }
            Ev::DriverArrives(tid) => {
                let (c, d) = (
                    self.trips[tid as usize].customer,
                    self.trips[tid as usize].driver,
                );
                if self.trips[tid as usize].cancelled {
                    return;
                }
                let end = self.trips[tid as usize].to_pickup.end_position();
                self.drivers[d as usize].pos = end;
                let t = &mut self.trips[tid as usize];
                t.driver_arrive_ms = self.now_ms;
                t.arrived = true;
                t.meet_gap_s = (self.now_ms as f64 - t.cust_arrive_ms as f64).abs() / 1000.0;
                let ready = self.now_ms.max(t.cust_arrive_ms)
                    + (self.sc.driver.dwell_pickup_s * 1000.0) as u64;
                let at = self.latlng(end);
                self.trace("driver_arrives", Some(c), Some(d), at);
                self.push(ready, Ev::PickupDone(tid));
            }
            Ev::PickupDone(tid) => {
                let (c, d) = (
                    self.trips[tid as usize].customer,
                    self.trips[tid as usize].driver,
                );
                let from = self.drivers[d as usize].pos;
                let mut best: Option<EdgeRoute> = None;
                // P04, LOC-38: chiến lược có thể chọn điểm trả khác điểm đích đã snap (mặc định `None`: baseline)
                let drop_positions: Vec<Position> = {
                    let cu = &self.custs[c as usize];
                    let dv = DropView {
                        customer: c,
                        dest_true: cu.dest_true,
                        dest_snap: cu.view.dropoff,
                        from,
                        default_positions: cu.dropoff.clone(),
                    };
                    let fleet = self.fleet_stats();
                    let ctx = Ctx {
                        engine: self.engine,
                        scenario: self.sc,
                        now_ms: self.now_ms,
                        router: &self.router,
                        fleet: &fleet,
                    };
                    self.pickup
                        .choose_dropoff(&ctx, &dv)
                        .filter(|v| !v.is_empty())
                        .unwrap_or_else(|| dv.default_positions.clone())
                };
                for &p in &drop_positions {
                    if let Some(r) =
                        self.router
                            .route_car(self.now_ms, Start::Pos(from), Goal::Pos(p))
                        && best.as_ref().is_none_or(|b| r.cost < b.cost)
                    {
                        best = Some(r);
                    }
                }
                let at = self.latlng(from);
                self.trace("pickup", Some(c), Some(d), at);
                let Some(route) = best else {
                    // không có đường tới điểm trả: huỷ tại điểm đón
                    self.release_driver(d, from);
                    self.close_customer(c, Outcome::CancelledNoRoute, Some(d), None);
                    self.try_match();
                    return;
                };
                let factor = self.jitter();
                let tm = self.router.turn_model(self.now_ms);
                let active = ActiveRoute::build_with(
                    self.engine,
                    &tm,
                    &route,
                    self.now_ms,
                    factor,
                    self.truth.as_deref(),
                );
                let u = tm.u_turns(&route.edges);
                let end_ms = self.now_ms
                    + active.total_ms
                    + (self.sc.driver.dwell_dropoff_s * 1000.0) as u64;
                let t = &mut self.trips[tid as usize];
                t.u_turns += u;
                t.trip = Some(active);
                if let Some(o) = self.observer.as_mut()
                    && let Some(a) = self.trips[tid as usize].trip.as_ref()
                {
                    o.leg_started(&LegInfo {
                        kind: LegKind::Onboard,
                        customer: c,
                        driver: d,
                        start_ms: self.now_ms,
                        route: a,
                    });
                }
                self.drivers[d as usize].state = DState::WithPassenger(tid);
                self.drivers[d as usize].free_at_ms = end_ms;
                self.push(end_ms, Ev::TripEnds(tid));
            }
            Ev::TripEnds(tid) => {
                let (c, d) = (
                    self.trips[tid as usize].customer,
                    self.trips[tid as usize].driver,
                );
                let (end, trip_m, trip_s) = {
                    let a = self.trips[tid as usize].trip.as_ref().expect("đã đón");
                    (a.end_position(), a.length_m(), a.total_ms as f64 / 1000.0)
                };
                let t = &self.trips[tid as usize];
                let cu = &self.custs[c as usize];
                let rec = TripRecord {
                    customer: c,
                    driver: Some(d),
                    request_s: cu.request_ms as f64 / 1000.0,
                    outcome: Outcome::Completed,
                    walk_m: t.walk_m,
                    deadhead_m: t.deadhead_m,
                    pickup_eta_s: (t.driver_arrive_ms - t.assign_ms) as f64 / 1000.0,
                    wait_s: (t.driver_arrive_ms - cu.request_ms) as f64 / 1000.0,
                    meet_gap_s: t.meet_gap_s,
                    trip_m,
                    trip_s,
                    u_turns: t.u_turns,
                    pin_snap_m: cu.pickup.as_ref().map_or(0.0, |p| p.pin_snap_m),
                };
                let at = self.latlng(end);
                self.trace("trip_ends", Some(c), Some(d), at);
                self.release_driver(d, end);
                self.drivers[d as usize].trips_done += 1;
                self.close_customer(c, Outcome::Completed, Some(d), Some(rec));
                // khách đã xếp hàng chờ tài xế này (LOC-47): đi đón ngay
                if let Some(next) = self.drivers[d as usize].queued.take()
                    && self.custs[next as usize].status == CStatus::Queued
                {
                    let queued_ms = self.custs[next as usize].queued_ms;
                    self.start_trip(next, d, Some(queued_ms));
                }
                self.try_match();
            }
            Ev::Cancel(tid) => {
                let (c, d) = (
                    self.trips[tid as usize].customer,
                    self.trips[tid as usize].driver,
                );
                if self.trips[tid as usize].arrived || self.trips[tid as usize].cancelled {
                    return;
                }
                self.trips[tid as usize].cancelled = true;
                if let Some(o) = self.observer.as_mut() {
                    o.leg_cancelled(c, self.now_ms);
                }
                let (pos, travelled) = {
                    let r = &self.trips[tid as usize].to_pickup;
                    (r.position_at(self.now_ms), r.distance_until(self.now_ms))
                };
                let cu = &self.custs[c as usize];
                let t = &self.trips[tid as usize];
                let rec = TripRecord {
                    customer: c,
                    driver: Some(d),
                    request_s: cu.request_ms as f64 / 1000.0,
                    outcome: Outcome::CancelledAfterAssign,
                    walk_m: t.walk_m,
                    deadhead_m: travelled,
                    pickup_eta_s: (self.now_ms - t.assign_ms) as f64 / 1000.0,
                    wait_s: (self.now_ms - cu.request_ms) as f64 / 1000.0,
                    meet_gap_s: 0.0,
                    trip_m: 0.0,
                    trip_s: 0.0,
                    u_turns: 0,
                    pin_snap_m: cu.pickup.as_ref().map_or(0.0, |p| p.pin_snap_m),
                };
                let at = self.latlng(pos);
                self.trace("cancel", Some(c), Some(d), at);
                self.release_driver(d, pos);
                self.close_customer(c, Outcome::CancelledAfterAssign, Some(d), Some(rec));
                self.try_match();
            }
            Ev::Patience(c) => {
                if self.custs[c as usize].status == CStatus::Pending {
                    let at = self.custs[c as usize].view.pin;
                    self.trace("patience_expired", Some(c), None, at);
                    self.close_customer(c, Outcome::CancelledNoDriver, None, None);
                }
            }
            Ev::Tick => {
                self.tick_open = false;
                self.in_tick = true;
                self.extras.batches += 1;
                self.try_match();
                self.in_tick = false;
                // khách chưa được ghép ở lô này (ví dụ vượt giới hạn đón xa) chờ lô sau
                self.maybe_open_window();
            }
            Ev::CancelQueued(c) => match self.custs[c as usize].status {
                CStatus::Queued => {
                    let d = self.custs[c as usize]
                        .queued_driver
                        .expect("đang xếp hàng thì có tài xế");
                    self.drivers[d as usize].queued = None;
                    self.extras.chain_cancelled_in_queue += 1;
                    let cu = &self.custs[c as usize];
                    let rec = TripRecord {
                        customer: c,
                        driver: Some(d),
                        request_s: cu.request_ms as f64 / 1000.0,
                        outcome: Outcome::CancelledAfterAssign,
                        walk_m: cu.pickup.as_ref().map_or(0.0, |p| p.walk_m),
                        deadhead_m: 0.0,
                        pickup_eta_s: (self.now_ms - cu.queued_ms) as f64 / 1000.0,
                        wait_s: (self.now_ms - cu.request_ms) as f64 / 1000.0,
                        meet_gap_s: 0.0,
                        trip_m: 0.0,
                        trip_s: 0.0,
                        u_turns: 0,
                        pin_snap_m: cu.pickup.as_ref().map_or(0.0, |p| p.pin_snap_m),
                    };
                    let at = cu.view.pin;
                    self.trace("cancel_queued", Some(c), Some(d), at);
                    self.close_customer(c, Outcome::CancelledAfterAssign, Some(d), Some(rec));
                    self.try_match();
                }
                CStatus::Assigned => {
                    if let Some(tid) = self.trips.iter().rposition(|t| t.customer == c) {
                        self.handle(Ev::Cancel(tid as u32));
                    }
                }
                _ => {}
            },
            Ev::RepoTick => self.reposition_tick(),
            Ev::RepoDone(d, id) => {
                let drv = &mut self.drivers[d as usize];
                if drv.repos.as_ref().is_some_and(|r| r.id == id) {
                    let r = drv.repos.take().expect("vừa kiểm tra");
                    drv.pos = r.route.end_position();
                    self.extras.cruise_m += r.route.length_m();
                }
            }
        }
    }

    /// Gọi `Repositioner` cho các tài xế rảnh chưa đang di chuyển; tài xế nghe theo với xác suất `reposition_accept_prob`.
    fn reposition_tick(&mut self) {
        let Some(interval) = self.repositioner.as_ref().map(|r| r.interval_ms()) else {
            return;
        };
        let idle: Vec<DriverView> = self
            .idle_views()
            .into_iter()
            .filter(|v| self.drivers[v.id as usize].repos.is_none())
            .collect();
        let mut pending_ids: Vec<u32> = (0..self.custs.len() as u32)
            .filter(|&c| self.custs[c as usize].status == CStatus::Pending)
            .collect();
        pending_ids.sort_by_key(|&c| (self.custs[c as usize].request_ms, c));
        let suggestions = {
            let pending: Vec<PendingView> = pending_ids
                .iter()
                .map(|&c| PendingView {
                    customer: c,
                    request_ms: self.custs[c as usize].request_ms,
                    pickup: self.custs[c as usize]
                        .pickup
                        .as_ref()
                        .expect("đang chờ thì có điểm đón"),
                })
                .collect();
            let fleet = self.fleet_stats();
            let ctx = Ctx {
                engine: self.engine,
                scenario: self.sc,
                now_ms: self.now_ms,
                router: &self.router,
                fleet: &fleet,
            };
            self.repositioner
                .as_mut()
                .map(|r| r.suggest(&ctx, &idle, &pending))
                .unwrap_or_default()
        };
        for s in suggestions {
            let Some(drv) = self.drivers.get(s.driver as usize) else {
                continue;
            };
            if drv.state != DState::Idle || drv.repos.is_some() {
                continue;
            }
            self.extras.reposition_suggested += 1;
            let u = self.rng_repo.f64();
            if u >= self.reposition_accept_prob {
                continue;
            }
            let from = drv.pos;
            let Some(route) =
                self.router
                    .route_car(self.now_ms, Start::Pos(from), Goal::Pos(s.target))
            else {
                continue;
            };
            let tm = self.router.turn_model(self.now_ms);
            let active = ActiveRoute::build_with(
                self.engine,
                &tm,
                &route,
                self.now_ms,
                1.0,
                self.truth.as_deref(),
            );
            let id = self.repo_seq;
            self.repo_seq += 1;
            let done = self.now_ms + active.total_ms;
            self.drivers[s.driver as usize].repos = Some(Repos { id, route: active });
            self.extras.reposition_accepted += 1;
            let at = self.latlng(from);
            self.trace("reposition", None, Some(s.driver), at);
            self.push(done, Ev::RepoDone(s.driver, id));
        }
        if (self.now_ms + interval) as f64 / 1000.0 < self.sc.duration_s {
            self.push(self.now_ms + interval, Ev::RepoTick);
        }
    }
}

/// Chạy một mô phỏng tới khi hết sự kiện.
pub fn run<'a>(
    engine: &'a Engine,
    sc: &'a Scenario,
    pickup: Box<dyn PickupStrategy + 'a>,
    matcher: Box<dyn MatchStrategy + 'a>,
    opts: RunOptions,
) -> SimOutput {
    run_with(engine, sc, pickup, matcher, opts, Extras::default())
}

/// Như [`run`] nhưng có phần cắm thêm của P05 (gợi ý di chuyển cho tài xế rảnh). `Extras::default()` cho đúng kết quả của `run`.
pub fn run_with<'a>(
    engine: &'a Engine,
    sc: &'a Scenario,
    pickup: Box<dyn PickupStrategy + 'a>,
    matcher: Box<dyn MatchStrategy + 'a>,
    opts: RunOptions,
    extras: Extras<'a>,
) -> SimOutput {
    run_hooked(
        engine,
        sc,
        pickup,
        matcher,
        opts,
        extras,
        Hooks::default(),
    )
}

/// Như [`run_with`] nhưng có thêm «thế giới thật» và người quan sát của P06. `Hooks::default()` cho đúng kết quả của `run_with`.
pub fn run_hooked<'a>(
    engine: &'a Engine,
    sc: &'a Scenario,
    pickup: Box<dyn PickupStrategy + 'a>,
    matcher: Box<dyn MatchStrategy + 'a>,
    opts: RunOptions,
    extras: Extras<'a>,
    hooks: Hooks<'a>,
) -> SimOutput {
    let world = generate(engine, sc);
    let router = Router::new(engine, sc);
    let (pickup_name, match_name) = (pickup.name().to_string(), matcher.name().to_string());
    let mut sim = Sim {
        engine,
        sc,
        router,
        pickup,
        matcher,
        custs: world
            .customers
            .iter()
            .map(|c| Cust {
                view: c.view,
                request_ms: c.request_ms,
                dropoff: c.dropoff.clone(),
                dest_true: c.dest_true,
                status: CStatus::NotYet,
                pickup: None,
                queued_ms: 0,
                queued_driver: None,
            })
            .collect(),
        drivers: world
            .drivers
            .iter()
            .map(|d| Drv {
                pos: d.pos,
                state: DState::Idle,
                busy_since_ms: 0,
                idle_since_ms: 0,
                trips_done: 0,
                busy_total_ms: 0,
                free_at_ms: 0,
                queued: None,
                repos: None,
            })
            .collect(),
        trips: Vec::new(),
        heap: BinaryHeap::new(),
        events: Vec::new(),
        now_ms: 0,
        rng_cancel: Rng::new(sc.seed, 3),
        rng_jitter: Rng::new(sc.seed, 4),
        records: Vec::new(),
        busy_ms: 0,
        trace: Vec::new(),
        opts,
        tick_open: false,
        in_tick: false,
        extras: ExtraStats::default(),
        reposition_accept_prob: extras.reposition_accept_prob,
        repositioner: extras.repositioner,
        rng_repo: Rng::new(sc.seed, 5),
        repo_seq: 0,
        truth: hooks.truth,
        observer: hooks.observer,
    };
    for i in 0..sim.custs.len() {
        let t = sim.custs[i].request_ms;
        sim.push(t, Ev::Request(i as u32));
    }
    if let Some(iv) = sim.repositioner.as_ref().map(|r| r.interval_ms()) {
        sim.push(iv, Ev::RepoTick);
    }
    while let Some(Reverse((t, seq))) = sim.heap.pop() {
        sim.now_ms = t;
        let ev = sim.events[seq as usize];
        sim.handle(ev);
    }
    let end_s = sim.now_ms as f64 / 1000.0;
    let mut records = std::mem::take(&mut sim.records);
    records.sort_by_key(|r| r.customer);
    let summary = summarize(
        &records,
        sim.custs.len(),
        end_s,
        sim.busy_ms as f64 / 1000.0,
        sim.drivers.len(),
    );
    SimOutput {
        scenario: sc.clone(),
        pickup_strategy: pickup_name,
        match_strategy: match_name,
        summary,
        trips: records,
        trace: std::mem::take(&mut sim.trace),
        extras: sim.extras,
    }
}
