//! Lõi mô phỏng sự kiện rời rạc. Một luồng, hàng đợi sự kiện theo (giờ, số thứ tự), không có `HashMap` duyệt thứ tự ngẫu nhiên,
//! nên cùng kịch bản, cùng seed và cùng chiến lược luôn cho cùng từng byte.

use crate::metrics::{Outcome, Summary, TripRecord, fnv1a, summarize, trips_csv};
use crate::rng::Rng;
use crate::scenario::Scenario;
use crate::strategy::{
    Assignment, Ctx, CustomerView, DriverView, MatchStrategy, PendingView, PickupChoice,
    PickupStrategy,
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
}

impl ActiveRoute {
    pub fn build(
        engine: &Engine,
        tm: &route_engine::turn::TurnModel,
        route: &EdgeRoute,
        start_ms: u64,
        factor: f64,
    ) -> Self {
        let g = &engine.g;
        let n = route.edges.len();
        let (mut from_frac, mut to_frac) = (Vec::with_capacity(n), Vec::with_capacity(n));
        let (mut wait_end, mut end, mut cum_m) = (Vec::new(), Vec::new(), Vec::new());
        let (mut t, mut dist) = (0f64, 0f64);
        let mut planned = 0f64;
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
            t += (wait + mv) * factor;
            wait_end.push((t - mv * factor).round() as u64);
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CStatus {
    NotYet,
    Pending,
    Assigned,
    Closed,
}

struct Cust {
    view: CustomerView,
    request_ms: u64,
    dropoff: Vec<Position>,
    status: CStatus,
    pickup: Option<PickupChoice>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DState {
    Idle,
    ToPickup(u32),
    WithPassenger(u32),
}

struct Drv {
    pos: Position,
    state: DState,
    busy_since_ms: u64,
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

pub struct SimOutput {
    pub scenario: Scenario,
    pub pickup_strategy: String,
    pub match_strategy: String,
    pub summary: Summary,
    pub trips: Vec<TripRecord>,
    pub trace: Vec<TraceEvent>,
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

    fn idle_views(&self) -> Vec<DriverView> {
        self.drivers
            .iter()
            .enumerate()
            .filter(|(_, d)| d.state == DState::Idle)
            .map(|(i, d)| DriverView {
                id: i as u32,
                pos: d.pos,
                latlng: self.latlng(d.pos),
            })
            .collect()
    }

    fn try_match(&mut self) {
        loop {
            let idle = self.idle_views();
            if idle.is_empty() {
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
                let ctx = Ctx {
                    engine: self.engine,
                    scenario: self.sc,
                    now_ms: self.now_ms,
                    router: &self.router,
                };
                self.matcher.assign(&ctx, &pending, &idle)
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
                if self
                    .drivers
                    .get(a.driver as usize)
                    .is_none_or(|d| d.state != DState::Idle)
                {
                    continue;
                }
                self.start_trip(a.customer, a.driver);
                progressed = true;
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

    fn start_trip(&mut self, c: u32, d: u32) {
        let now = self.now_ms;
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
        let active = ActiveRoute::build(self.engine, &tm, &route, now, factor);
        let u_turns = tm.u_turns(&route.edges);
        let eta_planned_s = active.planned_ms as f64 / 1000.0;
        let walk_m = pickup.walk_m;
        let cust_arrive_ms =
            self.custs[c as usize].request_ms + (pickup.walk_s * 1000.0).round() as u64;

        // huỷ chuyến: rút hai số để dòng ngẫu nhiên ổn định
        let (u, react) = (self.rng_cancel.f64(), self.rng_cancel.f64());
        let cn = &self.sc.cancel;
        let p = cn.probability(eta_planned_s, walk_m);
        let arrive_ms = now + active.total_ms;
        let cancel_at = now
            + ((cn.reaction_min_s + react * (cn.reaction_max_s - cn.reaction_min_s)) * 1000.0)
                .round() as u64;

        let tid = self.trips.len() as u32;
        let deadhead_m = active.length_m();
        self.trips.push(Trip {
            customer: c,
            driver: d,
            assign_ms: now,
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
        let drv = &mut self.drivers[d as usize];
        drv.state = DState::ToPickup(tid);
        drv.busy_since_ms = now;
        let at = self.latlng(driver_pos);
        self.trace("assign", Some(c), Some(d), at);
        self.push(arrive_ms, Ev::DriverArrives(tid));
        if u < p && cancel_at < arrive_ms {
            self.push(cancel_at, Ev::Cancel(tid));
        }
    }

    fn release_driver(&mut self, d: u32, pos: Position) {
        let drv = &mut self.drivers[d as usize];
        self.busy_ms += self.now_ms - drv.busy_since_ms;
        drv.pos = pos;
        drv.state = DState::Idle;
    }

    fn handle(&mut self, ev: Ev) {
        match ev {
            Ev::Request(c) => {
                let view = self.custs[c as usize].view;
                let choice = {
                    let ctx = Ctx {
                        engine: self.engine,
                        scenario: self.sc,
                        now_ms: self.now_ms,
                        router: &self.router,
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
                for &p in &self.custs[c as usize].dropoff {
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
                let active = ActiveRoute::build(self.engine, &tm, &route, self.now_ms, factor);
                let u = tm.u_turns(&route.edges);
                let end_ms = self.now_ms
                    + active.total_ms
                    + (self.sc.driver.dwell_dropoff_s * 1000.0) as u64;
                let t = &mut self.trips[tid as usize];
                t.u_turns += u;
                t.trip = Some(active);
                self.drivers[d as usize].state = DState::WithPassenger(tid);
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
                self.close_customer(c, Outcome::Completed, Some(d), Some(rec));
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
                status: CStatus::NotYet,
                pickup: None,
            })
            .collect(),
        drivers: world
            .drivers
            .iter()
            .map(|d| Drv {
                pos: d.pos,
                state: DState::Idle,
                busy_since_ms: 0,
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
    };
    for i in 0..sim.custs.len() {
        let t = sim.custs[i].request_ms;
        sim.push(t, Ev::Request(i as u32));
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
    }
}
