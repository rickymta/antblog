//! Bộ lọc Kalman vận tốc không đổi cho vị trí GPS, trên mặt phẳng mét cục bộ.
//!
//! Trạng thái mỗi trục là (vị trí, vận tốc). Hai trục độc lập, cùng mô hình, cùng nhiễu đo (accuracy báo cáo là
//! bán kính nên đều hướng) ⇒ ma trận hiệp phương sai 2×2 giống nhau ở cả hai trục: lưu một lần, tính một lần.
//! Nhiễu quá trình là gia tốc trắng `accel_sigma` (m/s²): lớn thì bám sát đo (ít trễ, nhiều nhiễu), nhỏ thì mượt (trễ khi rẽ, phanh).

use crate::{NoiseModel, Obs};
use geo_core::{LatLng, LocalProjection};

#[derive(Debug, Clone, Copy)]
pub struct KalmanConfig {
    /// Độ lệch chuẩn gia tốc ngẫu nhiên, m/s².
    pub accel_sigma: f64,
    pub noise: NoiseModel,
    /// Ngưỡng NIS (bình phương khoảng cách Mahalanobis của đổi mới, 2 bậc tự do): vượt thì phóng to R để điểm lạ ít ảnh hưởng.
    pub gate_nis: Option<f64>,
    /// Cách nhau quá số giây này thì khởi tạo lại (vận tốc cũ không còn đáng tin).
    pub reset_gap_s: f64,
    /// Độ bất định vận tốc ban đầu, m/s.
    pub init_speed_sigma: f64,
}

impl Default for KalmanConfig {
    fn default() -> Self {
        Self {
            // chọn trên bộ seed chọn tham số của lab LOC-15 (accel_sigma 4, cổng NIS 9,21), không phải bộ báo cáo
            accel_sigma: 4.0,
            noise: NoiseModel::default(),
            gate_nis: Some(9.21),
            reset_gap_s: 30.0,
            init_speed_sigma: 8.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Estimate {
    pub pos: LatLng,
    /// (đông, bắc) mét so với điểm đầu của bộ lọc.
    pub xy: [f64; 2],
    pub vel: [f64; 2],
    /// Độ lệch chuẩn vị trí mỗi trục, mét.
    pub pos_sigma_m: f64,
}

/// Một bước đã lọc, đủ dữ liệu để làm mượt RTS về sau.
#[derive(Debug, Clone, Copy)]
pub struct Step {
    pub dt: f64,
    pub reset: bool,
    x_pred: [[f64; 2]; 2],
    p_pred: [f64; 3],
    x_filt: [[f64; 2]; 2],
    p_filt: [f64; 3],
}

impl Step {
    /// Vị trí đã lọc (không làm mượt) của bước này, mét.
    pub fn filtered_xy(&self) -> [f64; 2] {
        [self.x_filt[0][0], self.x_filt[1][0]]
    }
}

pub struct KalmanFilter {
    cfg: KalmanConfig,
    proj: Option<LocalProjection>,
    t_ms: i64,
    /// `x[trục][0]` vị trí, `x[trục][1]` vận tốc.
    x: [[f64; 2]; 2],
    /// pp, pv, vv (giống nhau ở hai trục).
    p: [f64; 3],
}

impl KalmanFilter {
    pub fn new(cfg: KalmanConfig) -> Self {
        Self {
            cfg,
            proj: None,
            t_ms: 0,
            x: [[0.0; 2]; 2],
            p: [0.0; 3],
        }
    }

    pub fn projection(&self) -> Option<LocalProjection> {
        self.proj
    }

    fn estimate(&self) -> Estimate {
        let proj = self.proj.expect("đã khởi tạo");
        let xy = [self.x[0][0], self.x[1][0]];
        Estimate {
            pos: proj.to_latlng(xy),
            xy,
            vel: [self.x[0][1], self.x[1][1]],
            pos_sigma_m: self.p[0].max(0.0).sqrt(),
        }
    }

    /// Xử lý một điểm. Điểm trùng giờ hoặc đến muộn hơn điểm đã xử lý bị bỏ qua (trả `None`).
    pub fn update(&mut self, obs: &Obs) -> Option<(Estimate, Step)> {
        let r = self.cfg.noise.sigma_m(obs.accuracy_m).powi(2);
        let Some(proj) = self.proj else {
            let proj = LocalProjection::new(obs.pos);
            self.proj = Some(proj);
            self.t_ms = obs.t_ms;
            self.x = [[0.0; 2]; 2];
            self.p = [r, 0.0, self.cfg.init_speed_sigma.powi(2)];
            return Some((self.estimate(), self.step_init()));
        };
        let dt = (obs.t_ms - self.t_ms) as f64 / 1000.0;
        if dt <= 0.0 {
            return None;
        }
        let z = proj.to_xy(obs.pos);
        self.t_ms = obs.t_ms;
        if dt > self.cfg.reset_gap_s {
            self.x = [[z[0], 0.0], [z[1], 0.0]];
            self.p = [r, 0.0, self.cfg.init_speed_sigma.powi(2)];
            let mut s = self.step_init();
            s.dt = dt;
            return Some((self.estimate(), s));
        }
        // dự đoán: x' = F x, P' = F P Fᵀ + Q (gia tốc trắng)
        let q = self.cfg.accel_sigma.powi(2);
        let [pp, pv, vv] = self.p;
        let p_pred = [
            pp + 2.0 * dt * pv + dt * dt * vv + q * dt.powi(4) / 4.0,
            pv + dt * vv + q * dt.powi(3) / 2.0,
            vv + q * dt * dt,
        ];
        let mut x_pred = self.x;
        for a in &mut x_pred {
            a[0] += dt * a[1];
        }
        // cập nhật; cổng NIS phóng to R nếu đổi mới quá lớn so với dự đoán
        let nu = [z[0] - x_pred[0][0], z[1] - x_pred[1][0]];
        let mut r_eff = r;
        if let Some(gate) = self.cfg.gate_nis {
            let nis = (nu[0] * nu[0] + nu[1] * nu[1]) / (p_pred[0] + r);
            if nis > gate {
                r_eff = r * nis / gate;
            }
        }
        let s = p_pred[0] + r_eff;
        let k = [p_pred[0] / s, p_pred[1] / s];
        let mut x = x_pred;
        for a in 0..2 {
            x[a][0] += k[0] * nu[a];
            x[a][1] += k[1] * nu[a];
        }
        let p_filt = [
            (1.0 - k[0]) * p_pred[0],
            (1.0 - k[0]) * p_pred[1],
            p_pred[2] - k[1] * p_pred[1],
        ];
        self.x = x;
        self.p = p_filt;
        Some((
            self.estimate(),
            Step {
                dt,
                reset: false,
                x_pred,
                p_pred,
                x_filt: x,
                p_filt,
            },
        ))
    }

    fn step_init(&self) -> Step {
        Step {
            dt: 0.0,
            reset: true,
            x_pred: self.x,
            p_pred: self.p,
            x_filt: self.x,
            p_filt: self.p,
        }
    }
}

type M2 = [[f64; 2]; 2];

fn mat(p: [f64; 3]) -> M2 {
    [[p[0], p[1]], [p[1], p[2]]]
}
fn mul(a: M2, b: M2) -> M2 {
    let mut c = [[0.0; 2]; 2];
    for (i, row) in c.iter_mut().enumerate() {
        for (j, v) in row.iter_mut().enumerate() {
            *v = a[i][0] * b[0][j] + a[i][1] * b[1][j];
        }
    }
    c
}
fn inv(a: M2) -> M2 {
    let det = a[0][0] * a[1][1] - a[0][1] * a[1][0];
    [
        [a[1][1] / det, -a[0][1] / det],
        [-a[1][0] / det, a[0][0] / det],
    ]
}

/// Làm mượt Rauch–Tung–Striebel có trễ cố định: vị trí (đông, bắc) mét của bước `k` khi đã biết thêm `lag` bước sau nó.
/// `lag = 0` trả đúng kết quả lọc. Chuỗi dừng ở bước khởi tạo lại (`reset`), vì khi đó không có liên hệ giữa hai bên.
pub fn rts_fixed_lag(steps: &[Step], k: usize, lag: usize) -> [f64; 2] {
    let mut end = k;
    while end < k + lag && end + 1 < steps.len() && !steps[end + 1].reset {
        end += 1;
    }
    let mut xs = steps[end].x_filt;
    for j in (k..end).rev() {
        let next = &steps[j + 1];
        let ft = [[1.0, 0.0], [next.dt, 1.0]];
        let c = mul(mul(mat(steps[j].p_filt), ft), inv(mat(next.p_pred)));
        for (a, x) in xs.iter_mut().enumerate() {
            let d = [x[0] - next.x_pred[a][0], x[1] - next.x_pred[a][1]];
            *x = [
                steps[j].x_filt[a][0] + c[0][0] * d[0] + c[0][1] * d[1],
                steps[j].x_filt[a][1] + c[1][0] * d[0] + c[1][1] * d[1],
            ];
        }
    }
    [xs[0][0], xs[1][0]]
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_core::destination;

    fn obs_at(t_s: i64, p: LatLng, acc: Option<f32>) -> Obs {
        Obs {
            accuracy_m: acc,
            ..Obs::new(t_s * 1000, p)
        }
    }

    #[test]
    fn follows_constant_velocity_without_bias() {
        let start = LatLng::try_new(10.77, 106.69).unwrap();
        let mut kf = KalmanFilter::new(KalmanConfig::default());
        let mut last = None;
        for t in 0..60 {
            let p = destination(start, 90.0, 10.0 * t as f64);
            last = kf.update(&obs_at(t, p, Some(5.0))).map(|(e, _)| e);
        }
        let e = last.unwrap();
        assert!((e.vel[0] - 10.0).abs() < 0.2, "{:?}", e.vel);
        assert!(e.vel[1].abs() < 0.2);
    }

    #[test]
    fn ignores_out_of_order_and_resets_after_gap() {
        let p = LatLng::try_new(10.77, 106.69).unwrap();
        let mut kf = KalmanFilter::new(KalmanConfig::default());
        assert!(kf.update(&obs_at(10, p, None)).is_some());
        assert!(kf.update(&obs_at(10, p, None)).is_none());
        assert!(kf.update(&obs_at(9, p, None)).is_none());
        let far = destination(p, 0.0, 800.0);
        let (e, step) = kf.update(&obs_at(100, far, None)).unwrap();
        assert!(step.reset);
        assert!(geo_core::haversine_m(e.pos, far) < 1.0);
    }

    #[test]
    fn missing_accuracy_differs_from_zero() {
        // vắng accuracy dùng mặc định 8 m; accuracy = 0 bị kẹp về sàn 3 m, hai đường khác nhau
        let nm = NoiseModel::default();
        assert_eq!(nm.sigma_m(None), 8.0);
        assert_eq!(nm.sigma_m(Some(0.0)), 3.0);
        assert!((nm.sigma_m(Some(10.0)) - 6.7).abs() < 1e-9);
    }

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn smoother_with_lag_is_no_worse_than_filter_on_noisy_line() {
        let start = LatLng::try_new(10.77, 106.69).unwrap();
        let mut rng = crate::Rng::new(5);
        let mut kf = KalmanFilter::new(KalmanConfig::default());
        let mut steps = Vec::new();
        let mut truth = Vec::new();
        for t in 0..120 {
            let tp = destination(start, 45.0, 8.0 * t as f64);
            let noisy = destination(tp, rng.range(0.0, 360.0), 6.0 * rng.gauss().abs());
            let (_, s) = kf.update(&obs_at(t, noisy, Some(8.0))).unwrap();
            steps.push(s);
            truth.push(tp);
        }
        let proj = kf.projection().unwrap();
        let rmse = |lag: usize| {
            let mut acc = 0.0;
            for k in 10..100 {
                let xy = rts_fixed_lag(&steps, k, lag);
                let t = proj.to_xy(truth[k]);
                acc += (xy[0] - t[0]).powi(2) + (xy[1] - t[1]).powi(2);
            }
            (acc / 90.0).sqrt()
        };
        assert!(rmse(5) <= rmse(0) + 1e-9, "{} vs {}", rmse(5), rmse(0));
    }
}
