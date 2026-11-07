//! `map-match`: lọc nhiễu và bám đường cho điểm GPS của tài xế (loạt LOC, LOC-15 đến LOC-17).
//!
//! API các phần sau (điểm hẹn động LOC-36, lệch tuyến LOC-48, tracking LOC-19) dựa vào:
//! - [`Obs`]: một điểm GPS (thời gian, tọa độ, accuracy/speed/heading có thể vắng; vắng khác 0)
//! - [`kalman::KalmanFilter`]: bộ lọc vận tốc không đổi, nhiễu đo theo accuracy; [`kalman::rts_fixed_lag`] làm mượt có trễ
//! - [`hmm::MatchCtx`] + [`hmm::MatchConfig`]: bám đường HMM + Viterbi (Newson–Krumm), `match_offline`
//! - [`online::OnlineMatcher`]: cùng mô hình nhưng theo luồng, quyết định sau một độ trễ cố định
//! - [`hmm::MatchedPoint`]: kết quả cho một điểm (cạnh có hướng, vị trí dọc cạnh, điểm chiếu)
//! - [`trace`], [`scenario`], [`eval`]: sinh vết GPS tổng hợp có ground truth, tìm ca khó trên đồ thị, chấm điểm
//!
//! Mọi độ chính xác đo bằng vết do [`trace`] sinh ra là độ chính xác trên **nhiễu tổng hợp**, không phải GPS thật.

pub mod eval;
pub mod hmm;
pub mod kalman;
pub mod online;
pub mod route;
pub mod scenario;
pub mod trace;

use geo_core::{LatLng, LatLngE7};

/// Một điểm GPS. `accuracy_m` là bán kính tin cậy 68% do hệ điều hành báo; vắng nghĩa là app không biết (không phải 0).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Obs {
    pub t_ms: i64,
    pub pos: LatLng,
    pub accuracy_m: Option<f32>,
    pub speed_mps: Option<f32>,
    pub heading_deg: Option<f32>,
}

impl Obs {
    pub fn new(t_ms: i64, pos: LatLng) -> Self {
        Self {
            t_ms,
            pos,
            accuracy_m: None,
            speed_mps: None,
            heading_deg: None,
        }
    }

    /// Dựng từ đúng đơn vị của `GpsPoint` trong proto: E7, đề-xi-mét, đề-xi-mét/giây, đề-xi-độ.
    pub fn from_wire(
        t_ms: i64,
        lat_e7: i32,
        lng_e7: i32,
        accuracy_dm: Option<u32>,
        speed_dmps: Option<u32>,
        heading_ddeg: Option<u32>,
    ) -> Self {
        Self {
            t_ms,
            pos: LatLngE7::new(lat_e7, lng_e7).to_latlng(),
            accuracy_m: accuracy_dm.map(|v| v as f32 / 10.0),
            speed_mps: speed_dmps.map(|v| v as f32 / 10.0),
            heading_deg: heading_ddeg.map(|v| v as f32 / 10.0),
        }
    }
}

/// Đổi accuracy báo cáo thành độ lệch chuẩn mỗi trục (mét). Dùng chung cho Kalman (nhiễu đo) và HMM (xác suất phát).
///
/// Bán kính 68% của phân phối hai chiều đều hướng bằng khoảng 1,5 sigma mỗi trục, nên `accuracy_to_sigma` mặc định 0,67.
/// Khi vắng accuracy dùng `default_sigma_m`; kẹp vào `[floor_m, cap_m]` để một giá trị báo sai không làm bộ lọc tin tuyệt đối hay bỏ qua điểm.
#[derive(Debug, Clone, Copy)]
pub struct NoiseModel {
    pub accuracy_to_sigma: f64,
    pub default_sigma_m: f64,
    pub floor_m: f64,
    pub cap_m: f64,
}

impl Default for NoiseModel {
    fn default() -> Self {
        Self {
            accuracy_to_sigma: 0.67,
            default_sigma_m: 8.0,
            floor_m: 3.0,
            cap_m: 40.0,
        }
    }
}

impl NoiseModel {
    pub fn sigma_m(&self, accuracy_m: Option<f32>) -> f64 {
        accuracy_m
            .map_or(self.default_sigma_m, |a| {
                f64::from(a) * self.accuracy_to_sigma
            })
            .clamp(self.floor_m, self.cap_m)
    }
}

/// Bộ sinh số có seed dùng chung cho sinh vết và chọn ca (fastrand + Gaussian Box–Muller).
pub struct Rng(pub fastrand::Rng);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(fastrand::Rng::with_seed(seed))
    }
    pub fn f64(&mut self) -> f64 {
        self.0.f64()
    }
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.0.f64()
    }
    pub fn gauss(&mut self) -> f64 {
        let (u1, u2) = (1.0 - self.0.f64(), self.0.f64());
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
    pub fn usize(&mut self, n: usize) -> usize {
        self.0.usize(..n)
    }
    pub fn chance(&mut self, p: f64) -> bool {
        self.0.f64() < p
    }
}
