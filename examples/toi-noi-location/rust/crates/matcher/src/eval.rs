//! Đánh giá có cặp (paired): cùng seed, chiến lược so với baseline; trung bình hiệu, khoảng tin cậy 95%, trung vị, đuôi, theo vùng.
//!
//! Quy tắc của P05: một seed không chứng minh gì (`deadhead_m` p50 của baseline dao động 1.874,9 ± 286,6 m giữa các seed), nên mọi
//! khẳng định tối ưu dùng ít nhất 20 seed **đánh giá**, tách khỏi dải seed **chỉnh** (tuning); không báo cáo trên seed chỉnh.

use geo_core::{LatLng, haversine_m};
use simulator::metrics::Outcome;
use simulator::{ExtraStats, Scenario, SimOutput};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Dải seed chỉnh tham số (không dùng để báo cáo). Tách hẳn khỏi dải đánh giá.
pub fn tuning_seeds(n: usize) -> Vec<u64> {
    (0..n as u64).map(|i| 9001 + i).collect()
}

/// Dải seed đánh giá: 20261106, … (gồm 5 seed của baseline LOC-30, 20261106–20261110).
pub fn eval_seeds(n: usize) -> Vec<u64> {
    (0..n as u64).map(|i| 20_261_106 + i).collect()
}

/// Ba vùng theo khoảng cách chim bay từ điểm đặt xe tới tâm (Chợ Bến Thành): trung tâm, vành trong, ngoại vi.
#[derive(Debug, Clone, Copy)]
pub struct Zones {
    pub centre: LatLng,
    pub inner_m: f64,
    pub mid_m: f64,
}

pub const ZONE_NAMES: [&str; 3] = [
    "trung tâm (<3 km)",
    "vành trong (3–6 km)",
    "ngoại vi (≥6 km)",
];

impl Zones {
    pub fn hcm() -> Self {
        Self {
            centre: LatLng::try_new(10.7725, 106.6980).expect("hợp lệ"),
            inner_m: 3_000.0,
            mid_m: 6_000.0,
        }
    }
    pub fn zone_of(&self, p: LatLng) -> usize {
        let d = haversine_m(self.centre, p);
        if d < self.inner_m {
            0
        } else if d < self.mid_m {
            1
        } else {
            2
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ZoneM {
    pub requests: usize,
    pub completed: usize,
    pub cancelled_after: usize,
    pub unserved: usize,
    pub eta_mean: f64,
    pub wait_mean: f64,
    pub wait_p95: f64,
    pub deadhead_mean: f64,
}

#[derive(Debug, Clone)]
pub struct RunMetrics {
    pub seed: u64,
    pub requests: usize,
    pub completed: usize,
    pub cancel_rate: f64,
    /// Khách hết kiên nhẫn mà chưa có xe, trên số khách đặt.
    pub unserved_rate: f64,
    pub deadhead_mean: f64,
    pub deadhead_p50: f64,
    pub deadhead_p95: f64,
    pub deadhead_max: f64,
    pub eta_mean: f64,
    pub eta_p50: f64,
    pub eta_p95: f64,
    pub eta_max: f64,
    pub wait_mean: f64,
    pub wait_p50: f64,
    pub wait_p95: f64,
    pub wait_max: f64,
    pub walk_mean: f64,
    pub trip_mean: f64,
    pub u_turns_per_trip: f64,
    /// (chạy không của chuyến hoàn thành + chạy không bỏ phí do huỷ + chạy theo gợi ý) / số khách đặt, mét.
    pub empty_m_per_request: f64,
    /// Km rỗng tổng (đủ ba thành phần ở trên), mét.
    pub empty_m_total: f64,
    pub zone: [ZoneM; 3],
    /// Gini của số chuyến hoàn thành mỗi tài xế (0 = đều).
    pub trips_gini: f64,
    /// Tài xế không có chuyến nào (tỉ lệ).
    pub zero_trip_share: f64,
    /// Khoảng rảnh liên tiếp dài nhất của mỗi tài xế: p95 và max, giây.
    pub idle_gap_p95_s: f64,
    pub idle_gap_max_s: f64,
    pub extras: ExtraStats,
    pub wall_s: f64,
}

fn pct(v: &mut [f64], p: f64) -> f64 {
    v.sort_by(f64::total_cmp);
    simulator::metrics::percentile(v, p)
}

pub fn gini(v: &[f64]) -> f64 {
    let n = v.len();
    let sum: f64 = v.iter().sum();
    if n == 0 || sum <= 0.0 {
        return 0.0;
    }
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    let mut acc = 0.0;
    for (i, x) in s.iter().enumerate() {
        acc += (2.0 * (i as f64 + 1.0) - n as f64 - 1.0) * x;
    }
    acc / (n as f64 * sum)
}

/// Tính chỉ số của một lần chạy. Cần `trace` bật để biết vùng của khách và lịch bận của tài xế.
pub fn metrics_of(out: &SimOutput, zones: &Zones, seed: u64, wall_s: f64) -> RunMetrics {
    let s = &out.summary;
    let mut cust_zone: Vec<usize> = vec![0; s.requests];
    // vùng theo điểm ghim của khách (sự kiện `request` trong nhật ký)
    for e in out.trace.iter().filter(|e| e.kind == "request") {
        if let (Some(c), Ok(p)) = (e.customer, LatLng::try_new(e.lat, e.lng))
            && (c as usize) < cust_zone.len()
        {
            cust_zone[c as usize] = zones.zone_of(p);
        }
    }
    let mut zr = [ZoneM::default(); 3];
    let mut zeta: [Vec<f64>; 3] = Default::default();
    let mut zwait: [Vec<f64>; 3] = Default::default();
    let mut zdead: [Vec<f64>; 3] = Default::default();
    for t in &out.trips {
        let z = cust_zone[t.customer as usize];
        zr[z].requests += 1;
        match t.outcome {
            Outcome::Completed => {
                zr[z].completed += 1;
                zeta[z].push(t.pickup_eta_s);
                zwait[z].push(t.wait_s);
                zdead[z].push(t.deadhead_m);
            }
            Outcome::CancelledAfterAssign => zr[z].cancelled_after += 1,
            Outcome::CancelledNoDriver => zr[z].unserved += 1,
            Outcome::CancelledNoRoute => {}
        }
    }
    for z in 0..3 {
        let mean = |v: &[f64]| {
            if v.is_empty() {
                f64::NAN
            } else {
                v.iter().sum::<f64>() / v.len() as f64
            }
        };
        zr[z].eta_mean = mean(&zeta[z]);
        zr[z].wait_mean = mean(&zwait[z]);
        zr[z].wait_p95 = pct(&mut zwait[z], 95.0);
        zr[z].deadhead_mean = mean(&zdead[z]);
    }
    // lịch bận của tài xế: bận từ assign/chain_start tới trip_ends/cancel
    let n_drv = out.scenario.drivers;
    let mut trips = vec![0.0f64; n_drv];
    let mut busy_since: Vec<Option<f64>> = vec![None; n_drv];
    let mut last_free: Vec<f64> = vec![0.0; n_drv];
    let mut gaps: Vec<Vec<f64>> = vec![Vec::new(); n_drv];
    let end_s = s.sim_end_s;
    for e in &out.trace {
        let Some(d) = e.driver.map(|d| d as usize) else {
            continue;
        };
        match e.kind {
            "assign" | "chain_start" => {
                gaps[d].push(e.t_s - last_free[d]);
                busy_since[d] = Some(e.t_s);
            }
            "trip_ends" => {
                trips[d] += 1.0;
                busy_since[d] = None;
                last_free[d] = e.t_s;
            }
            "cancel" => {
                busy_since[d] = None;
                last_free[d] = e.t_s;
            }
            _ => {}
        }
    }
    for d in 0..n_drv {
        if busy_since[d].is_none() {
            gaps[d].push(end_s - last_free[d]);
        }
    }
    let longest: Vec<f64> = gaps
        .iter()
        .map(|g| g.iter().copied().fold(0.0, f64::max))
        .collect();
    let empty_total = s.deadhead_total_m + s.wasted_deadhead_m + out.extras.cruise_m;
    RunMetrics {
        seed,
        requests: s.requests,
        completed: s.completed,
        cancel_rate: s.cancel_rate,
        unserved_rate: s.cancelled_no_driver as f64 / s.requests.max(1) as f64,
        deadhead_mean: s.deadhead_m.mean,
        deadhead_p50: s.deadhead_m.p50,
        deadhead_p95: s.deadhead_m.p95,
        deadhead_max: s.deadhead_m.max,
        eta_mean: s.pickup_eta_s.mean,
        eta_p50: s.pickup_eta_s.p50,
        eta_p95: s.pickup_eta_s.p95,
        eta_max: s.pickup_eta_s.max,
        wait_mean: s.wait_s.mean,
        wait_p50: s.wait_s.p50,
        wait_p95: s.wait_s.p95,
        wait_max: s.wait_s.max,
        walk_mean: s.walk_m.mean,
        trip_mean: s.trip_m.mean,
        u_turns_per_trip: s.u_turns_per_trip,
        empty_m_per_request: empty_total / s.requests.max(1) as f64,
        empty_m_total: empty_total,
        zone: zr,
        trips_gini: gini(&trips),
        zero_trip_share: trips.iter().filter(|&&t| t == 0.0).count() as f64 / n_drv.max(1) as f64,
        idle_gap_p95_s: pct(&mut longest.clone(), 95.0),
        idle_gap_max_s: longest.iter().copied().fold(0.0, f64::max),
        extras: out.extras,
        wall_s,
    }
}

/// Chạy `run_one` trên từng seed (song song tối đa `jobs` luồng) và tính chỉ số. Kết quả theo thứ tự `seeds`.
pub fn sweep<F>(
    sc: &Scenario,
    seeds: &[u64],
    zones: &Zones,
    jobs: usize,
    run_one: F,
) -> Vec<RunMetrics>
where
    F: Fn(&Scenario) -> SimOutput + Sync,
{
    let next = AtomicUsize::new(0);
    let out: Mutex<Vec<Option<RunMetrics>>> = Mutex::new(vec![None; seeds.len()]);
    std::thread::scope(|scope| {
        for _ in 0..jobs.max(1).min(seeds.len().max(1)) {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::SeqCst);
                    if i >= seeds.len() {
                        break;
                    }
                    let mut s = sc.clone();
                    s.seed = seeds[i];
                    let t = std::time::Instant::now();
                    let o = run_one(&s);
                    let m = metrics_of(&o, zones, seeds[i], t.elapsed().as_secs_f64());
                    out.lock().expect("khoá")[i] = Some(m);
                }
            });
        }
    });
    out.into_inner()
        .expect("khoá")
        .into_iter()
        .map(|m| m.expect("mọi seed có kết quả"))
        .collect()
}

/// Giá trị tới hạn hai phía 95% của phân phối t Student theo bậc tự do.
pub fn t_crit_975(df: usize) -> f64 {
    const T: [f64; 30] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
        2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
        2.052, 2.048, 2.045, 2.042,
    ];
    match df {
        0 => f64::NAN,
        1..=30 => T[df - 1],
        31..=40 => 2.021,
        41..=60 => 2.000,
        61..=120 => 1.980,
        _ => 1.960,
    }
}

/// So sánh có cặp: `diff = b − a` (chiến lược trừ baseline).
#[derive(Debug, Clone, Copy)]
pub struct Paired {
    pub n: usize,
    pub mean_a: f64,
    pub mean_b: f64,
    pub mean_diff: f64,
    pub ci_lo: f64,
    pub ci_hi: f64,
    pub med_a: f64,
    pub med_b: f64,
    pub med_diff: f64,
    /// Số seed mà `b < a` (chỉ số thấp hơn).
    pub b_lower: usize,
    pub b_higher: usize,
}

impl Paired {
    /// Khoảng tin cậy không chứa 0.
    pub fn significant(&self) -> bool {
        self.ci_lo > 0.0 || self.ci_hi < 0.0
    }
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n == 0 {
        return f64::NAN;
    }
    if n % 2 == 1 {
        v[n / 2]
    } else {
        0.5 * (v[n / 2 - 1] + v[n / 2])
    }
}

pub fn paired(a: &[f64], b: &[f64]) -> Paired {
    let pairs: Vec<(f64, f64)> = a
        .iter()
        .zip(b)
        .filter(|(x, y)| x.is_finite() && y.is_finite())
        .map(|(&x, &y)| (x, y))
        .collect();
    let n = pairs.len();
    let (xa, xb): (Vec<f64>, Vec<f64>) = pairs.iter().copied().unzip();
    let mut d: Vec<f64> = pairs.iter().map(|(x, y)| y - x).collect();
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
    let md = mean(&d);
    let sd = if n > 1 {
        (d.iter().map(|x| (x - md).powi(2)).sum::<f64>() / (n - 1) as f64).sqrt()
    } else {
        f64::NAN
    };
    let half = t_crit_975(n.saturating_sub(1)) * sd / (n as f64).sqrt();
    Paired {
        n,
        mean_a: mean(&xa),
        mean_b: mean(&xb),
        mean_diff: md,
        ci_lo: md - half,
        ci_hi: md + half,
        med_a: median(&mut xa.clone()),
        med_b: median(&mut xb.clone()),
        med_diff: median(&mut d),
        b_lower: pairs.iter().filter(|(x, y)| y < x).count(),
        b_higher: pairs.iter().filter(|(x, y)| y > x).count(),
    }
}

/// Chọn một chỉ số từ các lần chạy.
pub fn column(runs: &[RunMetrics], f: impl Fn(&RunMetrics) -> f64) -> Vec<f64> {
    runs.iter().map(f).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paired_difference_and_ci_on_a_known_sample() {
        let a = [10.0, 12.0, 11.0, 13.0, 9.0];
        let b = [8.0, 11.0, 9.0, 12.0, 8.0]; // b − a = −2, −1, −2, −1, −1
        let p = paired(&a, &b);
        assert_eq!(p.n, 5);
        assert!((p.mean_diff + 1.4).abs() < 1e-12);
        assert_eq!(p.b_lower, 5);
        // sd = 0.5477, t(4) = 2.776 → nửa khoảng = 2.776 × 0.5477 / √5 = 0.680
        assert!((p.ci_hi - (-1.4 + 0.680)).abs() < 5e-3, "{p:?}");
        assert!(p.significant());
        assert_eq!(p.med_diff, -1.0);
    }

    #[test]
    fn paired_skips_nan_pairs() {
        let p = paired(&[1.0, f64::NAN, 3.0], &[2.0, 5.0, 3.0]);
        assert_eq!(p.n, 2);
        assert!((p.mean_diff - 0.5).abs() < 1e-12);
    }

    #[test]
    fn gini_extremes() {
        assert_eq!(gini(&[1.0, 1.0, 1.0, 1.0]), 0.0);
        assert!((gini(&[0.0, 0.0, 0.0, 4.0]) - 0.75).abs() < 1e-12);
    }

    #[test]
    fn seed_ranges_do_not_overlap() {
        let (t, e) = (tuning_seeds(50), eval_seeds(50));
        assert!(t.iter().all(|x| !e.contains(x)));
    }
}
