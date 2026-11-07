//! LOC-48: phát hiện lệch tuyến từ bám đường, định tuyến lại, ghép lại, và giới hạn để không làm phiền cả hai bên.
//!
//! Đây là **thí nghiệm độc lập**, không cắm vào simulator (simulator chưa có mô hình lệch tuyến). Mọi vết GPS và nhiễu
//! đều do lab sinh ra («trên dữ liệu mô phỏng»), nên độ trễ phát hiện và tỉ lệ báo nhầm chỉ có nghĩa với mô hình nhiễu đó.
//!
//! Ba phần, mỗi phần là một máy trạng thái nhỏ, thuần (không đọc đồng hồ, không I/O) nên test được bằng chuỗi tay:
//! - [`RouteMonitor`]: nhận điểm đã bám đường, quyết định `OffRoute` với hysteresis (cần `k_off` điểm liên tiếp ngoài tuyến,
//!   cách tuyến quá `off_dist_m`; muốn «quay lại» phải có `k_on` điểm liên tiếp trên tuyến), kèm cooldown và trần số lần báo.
//! - [`RematchPolicy`]: có đổi tài xế hay không, chỉ khi ETA gốc xấu đi, tài xế khác hơn hẳn một biên lớn hơn nhiễu ETA,
//!   không gần lúc tới nơi, tối đa số lần đổi mỗi chuyến, có cooldown.
//! - [`Notifier`]: ngân sách thông báo mỗi bên (tài xế nhận cập nhật tuyến, khách nhận cập nhật ETA).

use geo_core::LatLng;
use geo_core::segment::project_to_polyline;
use road_graph::{EdgeId, Graph};
use rustc_hash::FxHashSet;

/// Tuyến đã hứa với hai bên: danh sách cạnh có hướng, tập cạnh để tra nhanh, và polyline để đo khoảng cách tới tuyến.
#[derive(Debug, Clone)]
pub struct PlannedRoute {
    pub edges: Vec<EdgeId>,
    set: FxHashSet<EdgeId>,
    poly: Vec<LatLng>,
}

impl PlannedRoute {
    pub fn new(g: &Graph, edges: Vec<EdgeId>) -> Self {
        let mut poly: Vec<LatLng> = Vec::new();
        for &e in &edges {
            let pts = g.edge_points(e);
            if poly.is_empty() {
                poly.extend(pts);
            } else {
                poly.extend(pts.into_iter().skip(1));
            }
        }
        Self {
            set: edges.iter().copied().collect(),
            edges,
            poly,
        }
    }

    /// Dựng từ dữ liệu có sẵn (test không cần đồ thị).
    pub fn from_parts(edges: Vec<EdgeId>, poly: Vec<LatLng>) -> Self {
        Self {
            set: edges.iter().copied().collect(),
            edges,
            poly,
        }
    }

    pub fn contains(&self, e: EdgeId) -> bool {
        self.set.contains(&e)
    }

    /// Khoảng cách (mét) từ `p` tới polyline của tuyến.
    pub fn dist_to_m(&self, p: LatLng) -> f64 {
        project_to_polyline(p, &self.poly).map_or(f64::INFINITY, |r| r.dist_m)
    }
}

/// Một điểm GPS đã bám đường, như máy chủ thấy: `now_ms` là lúc máy chủ có kết quả (muộn hơn `t_ms` đúng bằng độ trễ chốt).
#[derive(Debug, Clone, Copy)]
pub struct MatchedObs {
    pub now_ms: i64,
    pub t_ms: i64,
    /// `None`: bám đường không có kết quả (mất tín hiệu); điểm đó không tính vào cả hai chiều.
    pub edge: Option<EdgeId>,
    pub pos: LatLng,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonitorConfig {
    /// Số điểm liên tiếp ngoài tuyến để chuyển sang `OffRoute`.
    pub k_off: usize,
    /// Điểm chỉ tính là ngoài tuyến nếu cạnh khớp không thuộc tuyến **và** cách polyline của tuyến quá ngưỡng này (mét).
    /// Ngưỡng nuốt các nhầm lẫn giữa hai đường song song sát nhau.
    pub off_dist_m: f64,
    /// Số điểm liên tiếp trên tuyến để quay lại `OnRoute` (hysteresis phía thoát).
    pub k_on: usize,
    /// Hai lần báo cách nhau tối thiểu chừng này (mili giây).
    pub cooldown_ms: i64,
    /// Trần số lần báo mỗi chuyến.
    pub max_alerts: usize,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            k_off: 3,
            off_dist_m: 30.0,
            k_on: 3,
            cooldown_ms: 30_000,
            max_alerts: 3,
        }
    }
}

impl MonitorConfig {
    /// Không hysteresis: một điểm là đủ để báo, không cooldown, không trần (đối chứng «ngây thơ»).
    pub fn naive() -> Self {
        Self {
            k_off: 1,
            off_dist_m: 0.0,
            k_on: 1,
            cooldown_ms: 0,
            max_alerts: usize::MAX,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Track {
    OnRoute,
    OffRoute,
}

/// Điều giám sát viên báo ra.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MonitorEvent {
    /// Lệch tuyến: máy chủ nên định tuyến lại. `now_ms` là lúc phát hiện, `first_off_t_ms` là giờ điểm GPS ngoài tuyến đầu tiên.
    Alert {
        now_ms: i64,
        first_off_t_ms: i64,
        off_points: usize,
    },
    /// Đã quay lại tuyến (đủ `k_on` điểm liên tiếp).
    Recovered { now_ms: i64 },
}

#[derive(Debug, Clone)]
pub struct RouteMonitor {
    pub cfg: MonitorConfig,
    route: PlannedRoute,
    state: Track,
    off_run: usize,
    on_run: usize,
    first_off_t_ms: i64,
    last_alert_ms: Option<i64>,
    alerts: usize,
    toggles: usize,
    pending: bool,
    /// Điểm cuối cùng được coi là «trên tuyến» (để biết lệch từ đâu).
    pub last_on_pos: Option<LatLng>,
    /// Giờ (điểm GPS) của điểm cuối cùng được coi là «trên tuyến».
    pub last_on_ms: Option<i64>,
}

impl RouteMonitor {
    pub fn new(cfg: MonitorConfig, route: PlannedRoute) -> Self {
        Self {
            cfg,
            route,
            state: Track::OnRoute,
            off_run: 0,
            on_run: 0,
            first_off_t_ms: 0,
            last_alert_ms: None,
            alerts: 0,
            toggles: 0,
            pending: false,
            last_on_pos: None,
            last_on_ms: None,
        }
    }

    pub fn state(&self) -> Track {
        self.state
    }
    pub fn alerts(&self) -> usize {
        self.alerts
    }
    /// Số lần đổi trạng thái `OnRoute` ↔ `OffRoute` (đo flip-flop).
    pub fn toggles(&self) -> usize {
        self.toggles
    }
    pub fn route(&self) -> &PlannedRoute {
        &self.route
    }

    /// `Some(true)` = ngoài tuyến, `Some(false)` = trên tuyến, `None` = không có kết quả bám đường.
    pub fn classify(&self, o: &MatchedObs) -> Option<bool> {
        let e = o.edge?;
        Some(!self.route.contains(e) && self.route.dist_to_m(o.pos) > self.cfg.off_dist_m)
    }

    /// Nhận một điểm. Trả sự kiện nếu có (tối đa một mỗi điểm).
    pub fn observe(&mut self, o: &MatchedObs) -> Option<MonitorEvent> {
        let off = self.classify(o)?;
        if off {
            if self.off_run == 0 {
                self.first_off_t_ms = o.t_ms;
            }
            self.off_run += 1;
            self.on_run = 0;
            if self.state == Track::OnRoute && self.off_run >= self.cfg.k_off {
                self.state = Track::OffRoute;
                self.toggles += 1;
                self.pending = true;
            }
            // báo bị cooldown chặn thì chờ tới khi hết cooldown mà vẫn còn ngoài tuyến; hết ngân sách thì bỏ
            if self.pending && self.state == Track::OffRoute && self.off_run >= self.cfg.k_off {
                if self.alerts >= self.cfg.max_alerts {
                    self.pending = false;
                } else if self
                    .last_alert_ms
                    .is_none_or(|t| o.now_ms - t >= self.cfg.cooldown_ms)
                {
                    self.pending = false;
                    self.alerts += 1;
                    self.last_alert_ms = Some(o.now_ms);
                    return Some(MonitorEvent::Alert {
                        now_ms: o.now_ms,
                        first_off_t_ms: self.first_off_t_ms,
                        off_points: self.off_run,
                    });
                }
            }
        } else {
            self.last_on_pos = Some(o.pos);
            self.last_on_ms = Some(o.t_ms);
            self.pending = false;
            self.on_run += 1;
            self.off_run = 0;
            if self.state == Track::OffRoute && self.on_run >= self.cfg.k_on {
                self.state = Track::OnRoute;
                self.toggles += 1;
                return Some(MonitorEvent::Recovered { now_ms: o.now_ms });
            }
        }
        None
    }

    /// Máy chủ đã định tuyến lại và hai bên đã nhận tuyến mới: theo dõi tuyến mới, trạng thái về `OnRoute`.
    /// Giữ nguyên số lần báo và cooldown, nên định tuyến lại không thể lặp vô hạn.
    pub fn adopt(&mut self, route: PlannedRoute) {
        self.route = route;
        if self.state == Track::OffRoute {
            self.toggles += 1;
        }
        self.state = Track::OnRoute;
        self.pending = false;
        self.off_run = 0;
        self.on_run = 0;
    }
}

/// Lý do không đổi tài xế (để lab đếm).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keep {
    MaxChanges,
    TooCloseToArrival,
    NotWorse,
    Cooldown,
    MarginTooSmall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RematchDecision {
    Keep(Keep),
    Switch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RematchPolicy {
    /// Tài xế khác phải nhanh hơn ít nhất `max(abs_margin_s, rel_margin × ETA hiện tại)`.
    pub abs_margin_s: f64,
    pub rel_margin: f64,
    /// Không đổi khi tài xế hiện tại còn không quá chừng này giây là tới (đổi lúc này chỉ làm khổ cả hai).
    pub no_rematch_within_s: f64,
    /// Chỉ xét đổi nếu ETA hiện tại đã xấu đi so với ETA gốc ít nhất chừng này giây.
    pub min_worsening_s: f64,
    pub max_changes: u32,
    pub cooldown_s: f64,
}

impl RematchPolicy {
    pub fn guarded() -> Self {
        Self {
            abs_margin_s: 120.0,
            rel_margin: 0.25,
            no_rematch_within_s: 90.0,
            min_worsening_s: 60.0,
            max_changes: 1,
            cooldown_s: 60.0,
        }
    }
    /// Đối chứng ngây thơ: đổi mỗi khi tài xế khác nhỉnh hơn dù chỉ một giây, không giới hạn số lần.
    pub fn naive() -> Self {
        Self {
            abs_margin_s: 0.0,
            rel_margin: 0.0,
            no_rematch_within_s: 0.0,
            min_worsening_s: f64::NEG_INFINITY,
            max_changes: u32::MAX,
            cooldown_s: 0.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RematchState {
    changes: u32,
    last_change_ms: Option<i64>,
}

impl RematchState {
    pub fn changes(&self) -> u32 {
        self.changes
    }

    /// Quyết định theo ETA đọc được lúc `now_ms`: `eta_orig_s` là ETA đã hứa với khách lúc gán, `eta_cur_s` của tài xế hiện tại,
    /// `eta_alt_s` của ứng viên tốt nhất khác. Gọi `Switch` thì state tự ghi nhận một lần đổi.
    pub fn decide(
        &mut self,
        p: &RematchPolicy,
        now_ms: i64,
        eta_orig_s: f64,
        eta_cur_s: f64,
        eta_alt_s: f64,
    ) -> RematchDecision {
        use RematchDecision::{Keep as K, Switch};
        if self.changes >= p.max_changes {
            return K(Keep::MaxChanges);
        }
        if eta_cur_s <= p.no_rematch_within_s {
            return K(Keep::TooCloseToArrival);
        }
        if eta_cur_s - eta_orig_s < p.min_worsening_s {
            return K(Keep::NotWorse);
        }
        if self
            .last_change_ms
            .is_some_and(|t| (now_ms - t) as f64 / 1000.0 < p.cooldown_s)
        {
            return K(Keep::Cooldown);
        }
        let gain = eta_cur_s - eta_alt_s;
        if gain > 0.0 && gain >= p.abs_margin_s.max(p.rel_margin * eta_cur_s) {
            self.changes += 1;
            self.last_change_ms = Some(now_ms);
            Switch
        } else {
            K(Keep::MarginTooSmall)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NotifyBudget {
    /// Số cập nhật tuyến tối đa gửi cho tài xế mỗi chuyến.
    pub max_driver: usize,
    /// Số cập nhật ETA tối đa gửi cho khách mỗi chuyến.
    pub max_customer: usize,
    /// Hai thông báo cùng một bên cách nhau tối thiểu (mili giây).
    pub min_gap_ms: i64,
    /// Khách chỉ được báo khi ETA đổi ít nhất chừng này giây.
    pub customer_min_delta_s: f64,
}

impl Default for NotifyBudget {
    fn default() -> Self {
        Self {
            max_driver: 3,
            max_customer: 2,
            min_gap_ms: 30_000,
            customer_min_delta_s: 60.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Notifier {
    pub budget: NotifyBudget,
    driver_sent: usize,
    customer_sent: usize,
    last_driver_ms: Option<i64>,
    last_customer_ms: Option<i64>,
}

impl Notifier {
    pub fn new(budget: NotifyBudget) -> Self {
        Self {
            budget,
            driver_sent: 0,
            customer_sent: 0,
            last_driver_ms: None,
            last_customer_ms: None,
        }
    }
    pub fn driver_sent(&self) -> usize {
        self.driver_sent
    }
    pub fn customer_sent(&self) -> usize {
        self.customer_sent
    }

    /// Tài xế nhận tuyến mới. `true` nếu được gửi.
    pub fn driver_route_update(&mut self, now_ms: i64) -> bool {
        let ok = self.driver_sent < self.budget.max_driver
            && self
                .last_driver_ms
                .is_none_or(|t| now_ms - t >= self.budget.min_gap_ms);
        if ok {
            self.driver_sent += 1;
            self.last_driver_ms = Some(now_ms);
        }
        ok
    }

    /// Khách nhận ETA mới, chỉ khi ETA đổi đủ lớn (`delta_s` có dấu; chỉ xét độ lớn) và còn ngân sách.
    pub fn customer_eta_update(&mut self, now_ms: i64, delta_s: f64) -> bool {
        let ok = delta_s.abs() >= self.budget.customer_min_delta_s
            && self.customer_sent < self.budget.max_customer
            && self
                .last_customer_ms
                .is_none_or(|t| now_ms - t >= self.budget.min_gap_ms);
        if ok {
            self.customer_sent += 1;
            self.last_customer_ms = Some(now_ms);
        }
        ok
    }
}
