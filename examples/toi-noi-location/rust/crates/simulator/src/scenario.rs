//! Kịch bản mô phỏng đọc từ TOML (`fixtures/scenario-*.toml`). Mọi tham số là **giả định của lab**, không đo từ dữ liệu thật.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Hotspot {
    pub name: String,
    pub lat: f64,
    pub lng: f64,
    /// Độ lệch chuẩn (mét) của điểm khách quanh tâm.
    pub sigma_m: f64,
    pub weight: f64,
}

/// Một khung thời gian của đường cong nhu cầu: tốc độ đặt xe = `requests_per_min × multiplier` trong `[from_s, to_s)`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Phase {
    pub from_s: f64,
    pub to_s: f64,
    pub multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Demand {
    /// Tốc độ đặt xe cơ sở (chuyến/phút).
    pub requests_per_min: f64,
    /// Ngoài các khung này, hệ số = 1. Rỗng = nhu cầu đều.
    #[serde(default)]
    pub phases: Vec<Phase>,
    /// Tỉ lệ khách ở nơi bất kỳ trên mạng đường (đều), phần còn lại chọn theo điểm nóng.
    pub background_share: f64,
    #[serde(default, rename = "hotspot")]
    pub hotspots: Vec<Hotspot>,
    /// Khoảng cách chim bay từ điểm đón tới điểm trả, mét.
    pub trip_min_m: f64,
    pub trip_max_m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Customer {
    /// Độ lệch chuẩn (mét, mỗi trục) giữa vị trí khách thật sự đứng và điểm ghim trên app.
    pub pin_sigma_m: f64,
    pub walk_speed_mps: f64,
    /// Khách huỷ nếu chưa được gán tài xế sau chừng này giây kể từ lúc đặt.
    pub patience_s: f64,
}

/// Mô hình huỷ chuyến đơn giản, tường minh, **chưa hiệu chuẩn bằng dữ liệu thật**.
///
/// Lúc gán tài xế, xác suất khách huỷ là
/// `p = max_prob × clamp((eta_s - eta_free_s) / (eta_max_s - eta_free_s), 0, 1) + walk_extra_per_100m × walk_m / 100`
/// (kẹp vào [0, 1]). Rút một số đều: nếu nhỏ hơn `p` thì khách huỷ sau một khoảng phản ứng ngẫu nhiên đều trong
/// `[reaction_min_s, reaction_max_s]`, miễn là tài xế chưa tới. Ngoài ra khách huỷ khi hết `patience_s` chưa có tài xế.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cancel {
    pub eta_free_s: f64,
    pub eta_max_s: f64,
    pub max_prob: f64,
    #[serde(default)]
    pub walk_extra_per_100m: f64,
    pub reaction_min_s: f64,
    pub reaction_max_s: f64,
}

impl Cancel {
    /// Xác suất khách huỷ lúc được gán tài xế, theo công thức ghi ở trên.
    pub fn probability(&self, eta_s: f64, walk_m: f64) -> f64 {
        let ramp = ((eta_s - self.eta_free_s) / (self.eta_max_s - self.eta_free_s)).clamp(0.0, 1.0);
        (self.max_prob * ramp + self.walk_extra_per_100m * walk_m / 100.0).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Driver {
    /// Tỉ lệ tài xế đặt đều trên mạng đường, phần còn lại quanh các điểm nóng.
    pub background_share: f64,
    /// Nhiễu thời gian chạy mỗi chặng: nhân với `exp(sigma × N(0,1))`. 0 = chạy đúng như dự tính.
    pub speed_jitter: f64,
    pub dwell_pickup_s: f64,
    pub dwell_dropoff_s: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Traffic {
    /// Tốc độ tự do của bảng theo loại đường.
    FreeFlow,
    /// Hồ sơ giờ tổng hợp (LOC-27) theo giờ trong ngày của thời điểm khởi hành mỗi chặng.
    Synthetic,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    /// Ghép ngay mỗi khi có khách mới hoặc có tài xế rảnh.
    Immediate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scenario {
    pub name: String,
    /// `fixture` hoặc `hcm`; chỉ để người chạy chọn đúng đồ thị và để kiểm tra nhầm.
    pub graph: String,
    pub seed: u64,
    /// Giờ trong ngày (0–23) lúc thời gian mô phỏng bằng 0.
    pub start_hour: u32,
    /// Khách chỉ đặt xe trong `[0, duration_s)`; mô phỏng chạy tiếp tới khi mọi chuyến kết thúc.
    pub duration_s: f64,
    pub drivers: usize,
    pub traffic: Traffic,
    /// Tâm và bán kính vùng trung tâm của hồ sơ kẹt xe tổng hợp (mặc định: tâm bản đồ, 5 km).
    #[serde(default)]
    pub traffic_center: Option<[f64; 2]>,
    #[serde(default)]
    pub traffic_radius_m: Option<f64>,
    pub decision: Decision,
    pub demand: Demand,
    pub customer: Customer,
    pub cancel: Cancel,
    pub driver: Driver,
}

#[derive(Debug, thiserror::Error)]
pub enum ScenarioError {
    #[error("không đọc được kịch bản: {0}")]
    Io(#[from] std::io::Error),
    #[error("kịch bản sai định dạng: {0}")]
    Parse(String),
    #[error("kịch bản không hợp lệ: {0}")]
    Invalid(String),
}

impl Scenario {
    pub fn from_toml(s: &str) -> Result<Self, ScenarioError> {
        let sc: Scenario = toml::from_str(s).map_err(|e| ScenarioError::Parse(e.to_string()))?;
        sc.validate()?;
        Ok(sc)
    }
    pub fn load(path: &std::path::Path) -> Result<Self, ScenarioError> {
        Self::from_toml(&std::fs::read_to_string(path)?)
    }
    pub fn validate(&self) -> Result<(), ScenarioError> {
        let bad = |m: &str| Err(ScenarioError::Invalid(m.to_string()));
        if self.drivers == 0 {
            return bad("drivers phải > 0");
        }
        let positive = |x: f64| x.is_finite() && x > 0.0;
        if !positive(self.duration_s) || !positive(self.demand.requests_per_min) {
            return bad("duration_s và requests_per_min phải > 0");
        }
        if !(0.0..=1.0).contains(&self.demand.background_share)
            || !(0.0..=1.0).contains(&self.driver.background_share)
        {
            return bad("background_share phải trong [0, 1]");
        }
        if self.demand.background_share < 1.0
            && self.demand.hotspots.iter().all(|h| h.weight <= 0.0)
        {
            return bad("cần ít nhất một hotspot có weight > 0 khi background_share < 1");
        }
        if self.demand.trip_min_m <= 0.0 || self.demand.trip_max_m < self.demand.trip_min_m {
            return bad("trip_min_m, trip_max_m không hợp lệ");
        }
        if self.cancel.eta_max_s <= self.cancel.eta_free_s
            || !(0.0..=1.0).contains(&self.cancel.max_prob)
        {
            return bad("cancel: eta_max_s > eta_free_s và max_prob trong [0, 1]");
        }
        if self.cancel.reaction_max_s < self.cancel.reaction_min_s {
            return bad("cancel: reaction_max_s < reaction_min_s");
        }
        if self.customer.walk_speed_mps <= 0.0 || self.customer.patience_s <= 0.0 {
            return bad("customer: walk_speed_mps và patience_s phải > 0");
        }
        if self.start_hour > 23 {
            return bad("start_hour trong [0, 23]");
        }
        Ok(())
    }

    /// Hệ số nhân của tốc độ đặt xe ở thời điểm `t` giây.
    pub fn demand_multiplier(&self, t: f64) -> f64 {
        self.demand
            .phases
            .iter()
            .find(|p| t >= p.from_s && t < p.to_s)
            .map_or(1.0, |p| p.multiplier)
    }
    pub fn max_multiplier(&self) -> f64 {
        self.demand
            .phases
            .iter()
            .map(|p| p.multiplier)
            .fold(1.0, f64::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub const SAMPLE: &str = r#"
name = "t"
graph = "fixture"
seed = 1
start_hour = 8
duration_s = 600
drivers = 5
traffic = "free_flow"
decision = "immediate"
[demand]
requests_per_min = 2.0
background_share = 0.5
trip_min_m = 300
trip_max_m = 900
[[demand.hotspot]]
name = "a"
lat = 10.77
lng = 106.69
sigma_m = 100
weight = 1
[[demand.phases]]
from_s = 0
to_s = 300
multiplier = 2.0
[customer]
pin_sigma_m = 10
walk_speed_mps = 1.3
patience_s = 300
[cancel]
eta_free_s = 120
eta_max_s = 600
max_prob = 0.5
reaction_min_s = 10
reaction_max_s = 40
[driver]
background_share = 0.5
speed_jitter = 0.1
dwell_pickup_s = 20
dwell_dropoff_s = 10
"#;

    #[test]
    fn parses_and_validates() {
        let s = Scenario::from_toml(SAMPLE).unwrap();
        assert_eq!(s.drivers, 5);
        assert_eq!(s.demand.hotspots.len(), 1);
        assert_eq!(s.demand_multiplier(100.0), 2.0);
        assert_eq!(s.demand_multiplier(400.0), 1.0);
        assert_eq!(s.max_multiplier(), 2.0);
    }

    #[test]
    fn cancel_probability_follows_the_stated_formula() {
        let s = Scenario::from_toml(SAMPLE).unwrap();
        let c = &s.cancel; // free 120, max 600, max_prob 0.5, walk_extra 0
        assert_eq!(c.probability(60.0, 0.0), 0.0);
        assert_eq!(c.probability(120.0, 0.0), 0.0);
        assert!((c.probability(360.0, 0.0) - 0.25).abs() < 1e-12);
        assert_eq!(c.probability(600.0, 0.0), 0.5);
        assert_eq!(c.probability(5000.0, 0.0), 0.5);
        let with_walk = Cancel {
            walk_extra_per_100m: 0.1,
            ..c.clone()
        };
        assert!((with_walk.probability(0.0, 300.0) - 0.3).abs() < 1e-12);
        assert_eq!(with_walk.probability(5000.0, 5000.0), 1.0);
    }

    #[test]
    fn rejects_bad_values() {
        let bad = SAMPLE.replace("drivers = 5", "drivers = 0");
        assert!(matches!(
            Scenario::from_toml(&bad),
            Err(ScenarioError::Invalid(_))
        ));
        let bad = SAMPLE.replace("eta_max_s = 600", "eta_max_s = 100");
        assert!(matches!(
            Scenario::from_toml(&bad),
            Err(ScenarioError::Invalid(_))
        ));
        assert!(matches!(
            Scenario::from_toml("name = 1"),
            Err(ScenarioError::Parse(_))
        ));
    }
}
