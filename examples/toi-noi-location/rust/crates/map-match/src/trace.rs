//! Sinh vết GPS tổng hợp có ground truth: cho xe chạy trên một lộ trình thật của đồ thị (có tăng tốc, phanh, rẽ chậm, dừng
//! đèn), rồi làm hỏng các điểm: nhiễu tương quan theo thời gian, accuracy báo sai lệch, điểm vắng, mất tín hiệu, lệch
//! hướng ổn định trong hẻm cao tầng (urban canyon), vài điểm nhảy xa (đa đường).
//!
//! Ground truth là của **chúng ta tự sinh**: mọi độ chính xác đo trên vết này là độ chính xác trên mô hình nhiễu này,
//! không phải trên GPS thật. Cùng `seed` luôn cho cùng vết.

use crate::{Obs, Rng};
use geo_core::{LatLng, LocalProjection, bearing_deg, bearing_diff_deg, haversine_m};
use road_graph::{EdgeId, Graph, HighwayClass};

#[derive(Debug, Clone)]
pub struct DriveConfig {
    /// Chu kỳ lấy mẫu của app, giây.
    pub sample_s: f64,
    /// Hệ số tốc độ của tài xế so với tốc độ đường, rút đều trong khoảng này.
    pub speed_factor: (f64, f64),
    pub accel: f64,
    pub decel: f64,
    /// Xác suất dừng (đèn đỏ, nhường đường) ở mỗi nút giao có từ ba cạnh ra trở lên.
    pub stop_prob: f64,
    pub dwell_s: (f64, f64),
}

impl Default for DriveConfig {
    fn default() -> Self {
        Self {
            sample_s: 1.0,
            speed_factor: (0.75, 1.0),
            accel: 1.8,
            decel: 2.5,
            stop_prob: 0.08,
            dwell_s: (8.0, 35.0),
        }
    }
}

/// Trạng thái thật của xe tại một thời điểm lấy mẫu.
#[derive(Debug, Clone, Copy)]
pub struct TruthPoint {
    pub t_ms: i64,
    pub pos: LatLng,
    /// Cạnh có hướng xe đang đi.
    pub edge: EdgeId,
    /// Chỉ số cạnh trong lộ trình.
    pub edge_idx: usize,
    pub along_m: f64,
    pub speed_mps: f64,
    pub heading_deg: f64,
}

fn class_speed_mps(c: HighwayClass) -> f64 {
    use HighwayClass::*;
    match c {
        Motorway => 22.0,
        MotorwayLink => 14.0,
        Trunk => 14.0,
        TrunkLink => 11.0,
        Primary => 11.0,
        PrimaryLink => 9.0,
        Secondary => 10.0,
        SecondaryLink => 8.0,
        Tertiary => 8.5,
        TertiaryLink => 7.0,
        Unclassified => 7.0,
        Residential => 6.5,
        LivingStreet => 4.0,
        Service => 4.5,
        Road => 6.0,
        _ => 5.0,
    }
}

fn turn_speed_mps(diff_deg: f64) -> f64 {
    if diff_deg < 15.0 {
        f64::INFINITY
    } else if diff_deg < 45.0 {
        9.0
    } else if diff_deg < 100.0 {
        5.0
    } else {
        3.0
    }
}

struct Geom {
    xy: Vec<[f64; 2]>,
    cum: Vec<f64>,
    seg_bearing: Vec<f64>,
    edge_end: Vec<f64>,
    /// Chỉ số đoạn cuối của từng cạnh trong `seg_bearing`.
    edge_last_seg: Vec<usize>,
}

fn build_geom(g: &Graph, route: &[EdgeId], proj: &LocalProjection) -> Geom {
    let mut xy: Vec<[f64; 2]> = Vec::new();
    let mut cum = vec![0.0];
    let mut seg_bearing = Vec::new();
    let mut edge_end = Vec::new();
    let mut edge_last_seg = Vec::new();
    let mut ll_prev: Option<LatLng> = None;
    for &e in route {
        for p in g.edge_points(e) {
            if let Some(q) = ll_prev {
                let d = haversine_m(q, p);
                if d < 1e-6 {
                    continue;
                }
                cum.push(cum.last().copied().unwrap_or(0.0) + d);
                seg_bearing.push(bearing_deg(q, p));
                xy.push(proj.to_xy(p));
            } else {
                xy.push(proj.to_xy(p));
            }
            ll_prev = Some(p);
        }
        edge_end.push(cum.last().copied().unwrap_or(0.0));
        edge_last_seg.push(seg_bearing.len().saturating_sub(1));
    }
    Geom {
        xy,
        cum,
        seg_bearing,
        edge_end,
        edge_last_seg,
    }
}

impl Geom {
    fn at(&self, s: f64) -> ([f64; 2], f64) {
        let i = self
            .cum
            .partition_point(|&c| c <= s)
            .clamp(1, self.cum.len() - 1)
            - 1;
        let l = (self.cum[i + 1] - self.cum[i]).max(1e-9);
        let t = ((s - self.cum[i]) / l).clamp(0.0, 1.0);
        let (a, b) = (self.xy[i], self.xy[i + 1]);
        (
            [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])],
            self.seg_bearing[i],
        )
    }
}

/// Cho xe chạy hết `route` (các cạnh có hướng, nối liền nhau) và lấy mẫu mỗi `cfg.sample_s` giây.
pub fn drive(g: &Graph, route: &[EdgeId], cfg: &DriveConfig, rng: &mut Rng) -> Vec<TruthPoint> {
    assert!(!route.is_empty(), "lộ trình rỗng");
    let origin = g.node_latlng(g.tail[route[0] as usize]);
    let proj = LocalProjection::new(origin);
    let geom = build_geom(g, route, &proj);
    let total = *geom.cum.last().expect("có đoạn");
    let factor = rng.range(cfg.speed_factor.0, cfg.speed_factor.1);
    let vc: Vec<f64> = route
        .iter()
        .map(|&e| {
            let v = g.edge(e);
            let mut base = class_speed_mps(v.class());
            if let Some(ms) = v.maxspeed_kmh() {
                base = base.min(f64::from(ms) / 3.6 * 0.6);
            }
            (base * factor).max(2.5)
        })
        .collect();
    // ràng buộc tốc độ tại ranh giới cạnh: (vị trí, tốc độ cho phép, thời gian dừng)
    let mut cons: Vec<(f64, f64, f64)> = Vec::new();
    for k in 0..route.len().saturating_sub(1) {
        let last = geom.edge_last_seg[k];
        let a = geom.seg_bearing[last];
        let b = geom.seg_bearing.get(last + 1).copied().unwrap_or(a);
        let mut v = turn_speed_mps(bearing_diff_deg(a, b))
            .min(vc[k])
            .min(vc[k + 1]);
        let deg = g.out_edges(g.head[route[k] as usize]).len();
        let mut dwell = 0.0;
        if deg >= 3 && rng.chance(cfg.stop_prob) {
            v = 0.0;
            dwell = rng.range(cfg.dwell_s.0, cfg.dwell_s.1);
        }
        cons.push((geom.edge_end[k], v, dwell));
    }
    cons.push((total, 0.0, 0.0));

    let dt = 0.1;
    let (mut t, mut s, mut v) = (0.0f64, 0.0f64, 0.0f64);
    let (mut ci, mut dwell) = (0usize, 0.0f64);
    let mut next_sample = 0.0;
    let mut out = Vec::new();
    let sample = |s: f64, v: f64, t: f64| {
        let (p, hd) = geom.at(s);
        let edge_idx = geom
            .edge_end
            .partition_point(|&e| e < s - 1e-6)
            .min(route.len() - 1);
        let start = if edge_idx == 0 {
            0.0
        } else {
            geom.edge_end[edge_idx - 1]
        };
        TruthPoint {
            t_ms: (t * 1000.0).round() as i64,
            pos: proj.to_latlng(p),
            edge: route[edge_idx],
            edge_idx,
            along_m: s - start,
            speed_mps: v,
            heading_deg: hd,
        }
    };
    while t < 4.0 * 3600.0 {
        if t >= next_sample - 1e-9 {
            out.push(sample(s, v, t));
            next_sample += cfg.sample_s;
        }
        if ci >= cons.len() {
            break;
        }
        let (cs, cv, cdwell) = cons[ci];
        if dwell > 0.0 {
            dwell -= dt;
            v = 0.0;
            t += dt;
            if dwell <= 0.0 {
                ci += 1;
            }
            continue;
        }
        let ek = geom
            .edge_end
            .partition_point(|&e| e < s - 1e-6)
            .min(route.len() - 1);
        let mut va = vc[ek];
        for &(sj, vj, _) in cons[ci..].iter().take(4) {
            va = va.min((vj * vj + 2.0 * cfg.decel * (sj - s).max(0.0)).sqrt());
        }
        v = (v + (va - v).clamp(-cfg.decel * dt, cfg.accel * dt)).max(0.0);
        s += v * dt;
        t += dt;
        if cv == 0.0 {
            if s >= cs - 0.3 && v < 1.0 {
                s = s.max(cs);
                v = 0.0;
                if cdwell > 0.0 {
                    dwell = cdwell;
                } else {
                    ci += 1;
                }
            }
        } else if s >= cs {
            ci += 1;
        }
    }
    out
}

/// Vùng hẻm cao tầng: lệch ổn định một hướng, nhiễu to hơn, và điện thoại chỉ báo accuracy tăng một phần.
#[derive(Debug, Clone, Copy)]
pub struct Canyon {
    pub start_s: f64,
    pub end_s: f64,
    pub bias_m: f64,
    pub bearing_deg: f64,
    pub sigma_mult: f64,
    /// Accuracy báo cáo nhân với hệ số này (nhỏ hơn `sigma_mult` nghĩa là điện thoại báo thiếu).
    pub reported_mult: f64,
}

#[derive(Debug, Clone)]
pub struct NoiseConfig {
    /// Độ lệch chuẩn mỗi trục, mét.
    pub sigma_m: f64,
    /// Hằng số tương quan thời gian (giây) của sai số GPS; 0 là nhiễu trắng.
    pub tau_s: f64,
    pub p_report_accuracy: f64,
    /// Độ lệch chuẩn của log hệ số sai lệch giữa accuracy báo cáo và sai số thật.
    pub accuracy_miscal: f64,
    pub p_outlier: f64,
    pub outlier_m: (f64, f64),
    /// Xác suất vắng ngẫu nhiên một điểm.
    pub p_drop: f64,
    /// Các khoảng thời gian (giây) hoàn toàn không có điểm (hầm, mất tín hiệu).
    pub dropouts: Vec<(f64, f64)>,
    pub canyons: Vec<Canyon>,
    pub heading_sigma_deg: f64,
    pub speed_sigma_mps: f64,
    pub p_report_speed: f64,
}

impl Default for NoiseConfig {
    fn default() -> Self {
        Self {
            sigma_m: 6.0,
            tau_s: 8.0,
            p_report_accuracy: 0.9,
            accuracy_miscal: 0.25,
            p_outlier: 0.005,
            outlier_m: (25.0, 70.0),
            p_drop: 0.01,
            dropouts: Vec::new(),
            canyons: Vec::new(),
            heading_sigma_deg: 10.0,
            speed_sigma_mps: 0.4,
            p_report_speed: 0.9,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Trace {
    pub route: Vec<EdgeId>,
    /// Trạng thái thật ở mọi thời điểm lấy mẫu, kể cả lúc không có điểm GPS.
    pub truth: Vec<TruthPoint>,
    /// Điểm GPS đã làm hỏng (không gồm điểm vắng).
    pub obs: Vec<Obs>,
    /// `obs[i]` ứng với `truth[obs_truth[i]]`.
    pub obs_truth: Vec<usize>,
}

impl Trace {
    pub fn truth_of(&self, i: usize) -> &TruthPoint {
        &self.truth[self.obs_truth[i]]
    }
}

/// Khoảng thời gian (giây) mà xe nằm trong `radius_m` quanh `focus`, để đặt `Canyon` hay `dropouts`.
pub fn zone_around(truth: &[TruthPoint], focus: LatLng, radius_m: f64) -> Option<(f64, f64)> {
    let mut it = truth
        .iter()
        .filter(|p| haversine_m(p.pos, focus) <= radius_m)
        .map(|p| p.t_ms as f64 / 1000.0);
    let first = it.next()?;
    let last = it.next_back().unwrap_or(first);
    Some((first, last))
}

/// Làm hỏng ground truth thành chuỗi điểm GPS theo `cfg`.
pub fn corrupt(
    route: &[EdgeId],
    truth: Vec<TruthPoint>,
    cfg: &NoiseConfig,
    rng: &mut Rng,
) -> Trace {
    let proj = LocalProjection::new(truth[0].pos);
    let (mut ex, mut ey) = (rng.gauss() * cfg.sigma_m, rng.gauss() * cfg.sigma_m);
    let mut obs = Vec::new();
    let mut obs_truth = Vec::new();
    let mut prev_t = truth[0].t_ms;
    for (i, tp) in truth.iter().enumerate() {
        let ts = tp.t_ms as f64 / 1000.0;
        let canyon = cfg
            .canyons
            .iter()
            .find(|c| ts >= c.start_s && ts <= c.end_s);
        let (smult, rmult) = canyon.map_or((1.0, 1.0), |c| (c.sigma_mult, c.reported_mult));
        let sigma = cfg.sigma_m * smult;
        let dt = (tp.t_ms - prev_t) as f64 / 1000.0;
        prev_t = tp.t_ms;
        let rho = if cfg.tau_s > 0.0 {
            (-dt / cfg.tau_s).exp()
        } else {
            0.0
        };
        let k = (1.0 - rho * rho).sqrt();
        ex = rho * ex + k * sigma * rng.gauss();
        ey = rho * ey + k * sigma * rng.gauss();
        let mut p = proj.to_xy(tp.pos);
        p[0] += ex;
        p[1] += ey;
        if let Some(c) = canyon {
            p[0] += c.bias_m * c.bearing_deg.to_radians().sin();
            p[1] += c.bias_m * c.bearing_deg.to_radians().cos();
        }
        if rng.chance(cfg.p_outlier) {
            let (b, d) = (
                rng.range(0.0, 360.0),
                rng.range(cfg.outlier_m.0, cfg.outlier_m.1),
            );
            p[0] += d * b.to_radians().sin();
            p[1] += d * b.to_radians().cos();
        }
        let dropped =
            cfg.dropouts.iter().any(|&(a, b)| ts >= a && ts < b) || rng.chance(cfg.p_drop);
        // luôn rút số ngẫu nhiên như nhau dù có bỏ điểm hay không, để vết không đổi khi thêm khoảng mất tín hiệu
        let acc_roll = (rng.f64(), rng.gauss());
        let speed_roll = (rng.f64(), rng.gauss());
        let head_gauss = rng.gauss();
        if dropped {
            continue;
        }
        let accuracy_m = (acc_roll.0 < cfg.p_report_accuracy).then(|| {
            (1.5 * sigma * rmult * (cfg.accuracy_miscal * acc_roll.1).exp()).max(3.0) as f32
        });
        let speed_mps = (speed_roll.0 < cfg.p_report_speed)
            .then(|| (tp.speed_mps + cfg.speed_sigma_mps * speed_roll.1).max(0.0) as f32);
        let heading_deg = (tp.speed_mps > 1.0).then(|| {
            (tp.heading_deg + cfg.heading_sigma_deg * head_gauss).rem_euclid(360.0) as f32
        });
        obs.push(Obs {
            t_ms: tp.t_ms,
            pos: proj.to_latlng(p),
            accuracy_m,
            speed_mps,
            heading_deg,
        });
        obs_truth.push(i);
    }
    Trace {
        route: route.to_vec(),
        truth,
        obs,
        obs_truth,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route::Router;
    use road_graph::Mode;
    use road_graph::fixture::{compile_fixture, generate};

    fn some_route(g: &Graph) -> Vec<EdgeId> {
        let mut r = Router::new(g);
        r.shortest_path(g, Mode::Car, 0, 150)
            .or_else(|| r.shortest_path(g, Mode::Car, 5, 120))
            .expect("fixture có đường")
            .0
    }

    #[test]
    fn same_seed_same_trace_and_truth_follows_route() {
        let g = compile_fixture(&generate());
        let route = some_route(&g);
        let make = |seed| {
            let mut rng = Rng::new(seed);
            let truth = drive(&g, &route, &DriveConfig::default(), &mut rng);
            corrupt(&route, truth, &NoiseConfig::default(), &mut rng)
        };
        let (a, b, c) = (make(1), make(1), make(2));
        assert_eq!(a.obs.len(), b.obs.len());
        assert!(a.obs.iter().zip(&b.obs).all(|(x, y)| x == y));
        assert!(a.obs.len() != c.obs.len() || a.obs.iter().zip(&c.obs).any(|(x, y)| x != y));
        // truth đi hết lộ trình, theo thứ tự cạnh, tốc độ không âm và không vượt mức vật lý
        assert_eq!(a.truth.first().unwrap().edge, route[0]);
        assert_eq!(a.truth.last().unwrap().edge, *route.last().unwrap());
        let idx: Vec<usize> = a.truth.iter().map(|p| p.edge_idx).collect();
        assert!(idx.windows(2).all(|w| w[0] <= w[1]));
        assert!(a.truth.iter().all(|p| (0.0..30.0).contains(&p.speed_mps)));
    }

    #[test]
    fn dropout_removes_points_and_keeps_truth() {
        let g = compile_fixture(&generate());
        let route = some_route(&g);
        let mut rng = Rng::new(3);
        let truth = drive(&g, &route, &DriveConfig::default(), &mut rng);
        let n_truth = truth.len();
        let cfg = NoiseConfig {
            p_drop: 0.0,
            dropouts: vec![(10.0, 25.0)],
            ..NoiseConfig::default()
        };
        let tr = corrupt(&route, truth, &cfg, &mut rng);
        assert_eq!(tr.truth.len(), n_truth);
        assert!(tr.obs.iter().all(|o| !(10_000..25_000).contains(&o.t_ms)));
        assert_eq!(tr.obs.len(), n_truth - 15);
    }

    #[test]
    fn noise_has_roughly_the_requested_scale() {
        let g = compile_fixture(&generate());
        let route = some_route(&g);
        let mut rng = Rng::new(9);
        let truth = drive(&g, &route, &DriveConfig::default(), &mut rng);
        let cfg = NoiseConfig {
            sigma_m: 8.0,
            tau_s: 0.0,
            p_outlier: 0.0,
            p_drop: 0.0,
            ..NoiseConfig::default()
        };
        let tr = corrupt(&route, truth, &cfg, &mut rng);
        let mean_sq: f64 = tr
            .obs
            .iter()
            .enumerate()
            .map(|(i, o)| haversine_m(o.pos, tr.truth_of(i).pos).powi(2))
            .sum::<f64>()
            / tr.obs.len() as f64;
        // hai trục độc lập: E[d²] = 2σ²
        let rms = mean_sq.sqrt();
        assert!((rms - 8.0 * 2f64.sqrt()).abs() < 3.0, "rms {rms}");
    }
}
