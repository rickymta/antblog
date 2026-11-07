//! LOC-15: lọc nhiễu GPS bằng Kalman vận tốc không đổi. Đo trên vết tổng hợp có ground truth: lợi ích, độ trễ phải trả, khi nào hại.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc15_kalman
//! Tham số: --trips N (mặc định 40), --seed S (báo cáo), --fixture

use geo_core::{LatLng, bearing_diff_deg, haversine_m};
use labs::Args;
use labs::p02::{self, EVAL_SEED, TUNE_SEED, Trip, p95, rmse};
use map_match::Obs;
use map_match::kalman::{KalmanConfig, KalmanFilter, Step, rts_fixed_lag};
use map_match::trace::{Canyon, DriveConfig, NoiseConfig, Trace};

struct Run {
    est: Vec<LatLng>,
    steps: Vec<Option<Step>>,
    proj: geo_core::LocalProjection,
}

fn run_filter(obs: &[Obs], cfg: KalmanConfig) -> Run {
    let mut kf = KalmanFilter::new(cfg);
    let (mut est, mut steps) = (Vec::with_capacity(obs.len()), Vec::with_capacity(obs.len()));
    for o in obs {
        match kf.update(o) {
            Some((e, s)) => {
                est.push(e.pos);
                steps.push(Some(s));
            }
            None => {
                est.push(*est.last().unwrap_or(&o.pos));
                steps.push(None);
            }
        }
    }
    Run {
        est,
        steps,
        proj: kf.projection().expect("có điểm"),
    }
}

/// (RMSE thô, RMSE Kalman, p95 thô, p95 Kalman) trên các vết.
fn compare(traces: &[Trace], cfg: KalmanConfig) -> (f64, f64, f64, f64) {
    let (mut sr, mut sk, mut n) = (0.0, 0.0, 0);
    let (mut er, mut ek) = (Vec::new(), Vec::new());
    for tr in traces {
        let run = run_filter(&tr.obs, cfg);
        for i in 0..tr.obs.len() {
            let t = tr.truth_of(i).pos;
            let (a, b) = (haversine_m(tr.obs[i].pos, t), haversine_m(run.est[i], t));
            sr += a * a;
            sk += b * b;
            er.push(a);
            ek.push(b);
            n += 1;
        }
    }
    (rmse(sr, n), rmse(sk, n), p95(er), p95(ek))
}

fn noise(sigma: f64, tau: f64) -> NoiseConfig {
    NoiseConfig {
        sigma_m: sigma,
        tau_s: tau,
        ..NoiseConfig::default()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Stopped,
    Turning,
    Accel,
    Brake,
    Cruise,
}

fn phase_of(tr: &Trace, i: usize) -> Option<Phase> {
    let k = tr.obs_truth[i];
    let t = &tr.truth;
    if k < 2 || k + 2 >= t.len() {
        return None;
    }
    let p = &t[k];
    if p.speed_mps < 0.3 {
        return Some(Phase::Stopped);
    }
    if bearing_diff_deg(t[k - 2].heading_deg, t[k + 2].heading_deg) > 25.0 {
        return Some(Phase::Turning);
    }
    let a = p.speed_mps - t[k - 1].speed_mps;
    Some(if a > 0.8 {
        Phase::Accel
    } else if a < -0.8 {
        Phase::Brake
    } else {
        Phase::Cruise
    })
}

fn main() {
    let args = Args::from_env();
    let n = args.u64_or("--trips", 40) as usize;
    let seed = args.u64_or("--seed", EVAL_SEED);
    let w = p02::world(&args);
    p02::header(
        "LOC-15: Kalman vận tốc không đổi",
        &w,
        &format!(
            "{n} chuyến mỗi bộ (dài 2–4,5 km trên HCM, 0,4–1,5 km trên fixture), lấy mẫu 1 Hz; seed báo cáo {seed}, seed chọn tham số {TUNE_SEED}"
        ),
    );
    let drive1 = DriveConfig::default();
    let tune_trips = p02::trips(&w, n, TUNE_SEED, &drive1);
    let eval_trips = p02::trips(&w, n, seed, &drive1);
    let pts = |t: &[Trip]| t.iter().map(|x| x.truth.len()).sum::<usize>();
    println!(
        "chuyến chọn tham số: {} điểm thật; chuyến báo cáo: {} điểm thật\n",
        pts(&tune_trips),
        pts(&eval_trips)
    );

    // 1. chọn tham số trên bộ tune (nhiễu mặc định: σ = 6 m, tương quan 8 s)
    println!(
        "-- 1. chọn gia tốc nhiễu quá trình trên bộ chọn tham số (σ 6 m, tương quan 8 s, có điểm nhảy ngẫu nhiên) --"
    );
    let tune = p02::corrupt_all(&tune_trips, &noise(6.0, 8.0), TUNE_SEED);
    println!(
        "{:>12} {:>10} {:>14} {:>14}",
        "accel_sigma", "gate_nis", "RMSE (m)", "p95 (m)"
    );
    let mut best = (f64::MAX, KalmanConfig::default());
    for gate in [None, Some(9.21)] {
        for a in [0.25, 0.5, 1.0, 2.0, 4.0, 8.0] {
            let cfg = KalmanConfig {
                accel_sigma: a,
                gate_nis: gate,
                ..KalmanConfig::default()
            };
            let (_, rk, _, pk) = compare(&tune, cfg);
            println!(
                "{a:>12} {:>10} {rk:>14.3} {pk:>14.3}",
                gate.map_or("tắt".into(), |g| format!("{g}"))
            );
            if rk < best.0 {
                best = (rk, cfg);
            }
        }
    }
    let cfg = best.1;
    println!(
        "chọn: accel_sigma = {}, gate_nis = {:?} (RMSE tune {:.3} m); mặc định trong KalmanConfig::default(): accel_sigma = {}, gate_nis = {:?}\n",
        cfg.accel_sigma,
        cfg.gate_nis,
        best.0,
        KalmanConfig::default().accel_sigma,
        KalmanConfig::default().gate_nis
    );

    // 2. lợi ích theo mức nhiễu (bộ báo cáo)
    println!(
        "-- 2. RMSE so với vị trí thật, theo độ lệch chuẩn nhiễu mỗi trục (bộ báo cáo, tham số đã chọn ở mục 1) --"
    );
    println!(
        "{:>10} {:>8} {:>10} {:>12} {:>10} {:>12} {:>10}",
        "σ (m)", "tương quan", "RMSE thô", "RMSE Kalman", "giảm", "p95 thô", "p95 Kalman"
    );
    for (sigma, tau) in [
        (2.0, 8.0),
        (4.0, 8.0),
        (6.0, 8.0),
        (10.0, 8.0),
        (20.0, 8.0),
        (6.0, 0.0),
        (10.0, 0.0),
    ] {
        let tr = p02::corrupt_all(&eval_trips, &noise(sigma, tau), seed);
        let (rr, rk, pr, pk) = compare(&tr, cfg);
        println!(
            "{sigma:>10} {:>8} {rr:>10.2} {rk:>12.2} {:>9.1}% {pr:>10.2} {pk:>10.2}",
            if tau > 0.0 {
                format!("{tau} s")
            } else {
                "trắng".into()
            },
            100.0 * (rr - rk) / rr
        );
    }

    // 3. độ trễ: sai số theo pha chuyển động; sai số dọc hướng đi âm là bộ lọc ở phía sau xe
    println!(
        "\n-- 3. sai số theo pha chuyển động: lọc không làm trễ giờ, nhưng kết quả chạy lệch so với xe khi vận tốc đổi --"
    );
    for (sigma, tau) in [(6.0, 8.0), (2.0, 0.0)] {
        let tr = p02::corrupt_all(&eval_trips, &noise(sigma, tau), seed);
        let mut acc: std::collections::BTreeMap<String, (usize, f64, f64, f64, f64)> =
            Default::default();
        for t in &tr {
            let run = run_filter(&t.obs, cfg);
            for i in 0..t.obs.len() {
                let Some(ph) = phase_of(t, i) else { continue };
                let truth = t.truth_of(i);
                let txy = run.proj.to_xy(truth.pos);
                let (kxy, rxy) = (run.proj.to_xy(run.est[i]), run.proj.to_xy(t.obs[i].pos));
                let (hx, hy) = (
                    truth.heading_deg.to_radians().sin(),
                    truth.heading_deg.to_radians().cos(),
                );
                let along = |p: [f64; 2]| (p[0] - txy[0]) * hx + (p[1] - txy[1]) * hy;
                let e = acc.entry(format!("{ph:?}")).or_default();
                e.0 += 1;
                e.1 += haversine_m(t.obs[i].pos, truth.pos).powi(2);
                e.2 += haversine_m(run.est[i], truth.pos).powi(2);
                e.3 += along(kxy);
                e.4 += along(rxy);
            }
        }
        println!(
            "nhiễu σ = {sigma} m, {}:",
            if tau > 0.0 {
                format!("tương quan {tau} s")
            } else {
                "trắng".into()
            }
        );
        println!(
            "{:<10} {:>8} {:>10} {:>12} {:>18} {:>16}",
            "pha", "số điểm", "RMSE thô", "RMSE Kalman", "dọc hướng (Kalman)", "dọc hướng (thô)"
        );
        for (ph, (n, sr, sk, ak, ar)) in &acc {
            println!(
                "{ph:<10} {n:>8} {:>10.2} {:>12.2} {:>16.2} m {:>14.2} m",
                rmse(*sr, *n),
                rmse(*sk, *n),
                ak / *n as f64,
                ar / *n as f64
            );
        }
    }
    let tr = p02::corrupt_all(&eval_trips, &noise(6.0, 8.0), seed);

    // 4. làm mượt có trễ
    println!(
        "\n-- 4. làm mượt RTS có trễ cố định: biết thêm L điểm sau thì kết quả cho điểm hiện tại tốt hơn, nhưng phải chờ L giây --"
    );
    println!("{:>6} {:>12} {:>10}", "L", "RMSE (m)", "so với lọc");
    let base = {
        let (mut s, mut c) = (0.0, 0);
        for t in &tr {
            let run = run_filter(&t.obs, cfg);
            for i in 0..t.obs.len() {
                s += haversine_m(run.est[i], t.truth_of(i).pos).powi(2);
                c += 1;
            }
        }
        rmse(s, c)
    };
    for lag in [0usize, 1, 2, 3, 5, 8, 12] {
        let (mut s, mut c) = (0.0, 0);
        for t in &tr {
            let run = run_filter(&t.obs, cfg);
            let steps: Vec<Step> = run.steps.iter().map(|x| x.expect("giờ tăng dần")).collect();
            for i in 0..t.obs.len() {
                let xy = rts_fixed_lag(&steps, i, lag);
                let tx = run.proj.to_xy(t.truth_of(i).pos);
                s += (xy[0] - tx[0]).powi(2) + (xy[1] - tx[1]).powi(2);
                c += 1;
            }
        }
        let r = rmse(s, c);
        println!("{lag:>6} {r:>12.3} {:>9.1}%", 100.0 * (r - base) / base);
    }

    // 5. khi nào Kalman không giúp hoặc hại
    println!("\n-- 5. khi nào Kalman không giúp hoặc hại (RMSE m; thô → Kalman) --");
    println!(
        "{:<62} {:>10} {:>10} {:>9}",
        "tình huống", "RMSE thô", "Kalman", "thay đổi"
    );
    let row = |name: &str, tr: &[Trace], c: KalmanConfig| {
        let (rr, rk, _, _) = compare(tr, c);
        println!(
            "{name:<62} {rr:>10.2} {rk:>10.2} {:>8.1}%",
            100.0 * (rk - rr) / rr
        );
    };
    row(
        "GPS tốt: σ = 2 m, nhiễu trắng",
        &p02::corrupt_all(&eval_trips, &noise(2.0, 0.0), seed),
        cfg,
    );
    row("σ = 6 m, tương quan 8 s (mặc định)", &tr, cfg);
    let big = p02::corrupt_all(&eval_trips, &noise(20.0, 8.0), seed);
    row("σ = 20 m, tương quan 8 s, có cổng NIS", &big, cfg);
    row(
        "σ = 20 m, tương quan 8 s, không cổng NIS",
        &big,
        KalmanConfig {
            gate_nis: None,
            ..cfg
        },
    );
    let outl = NoiseConfig {
        p_outlier: 0.03,
        ..noise(6.0, 8.0)
    };
    let tr_out = p02::corrupt_all(&eval_trips, &outl, seed);
    row(
        "3% điểm nhảy xa 25–70 m, không cổng NIS",
        &tr_out,
        KalmanConfig {
            gate_nis: None,
            ..cfg
        },
    );
    row(
        "3% điểm nhảy xa 25–70 m, cổng NIS 9,21",
        &tr_out,
        KalmanConfig {
            gate_nis: Some(9.21),
            ..cfg
        },
    );
    let nolabel = NoiseConfig {
        p_report_accuracy: 0.0,
        ..noise(6.0, 8.0)
    };
    row(
        "app không báo accuracy (dùng 8 m mặc định)",
        &p02::corrupt_all(&eval_trips, &nolabel, seed),
        cfg,
    );
    row(
        "accel_sigma = 0,25 (quá mượt) so với gia tốc thật của xe",
        &tr,
        KalmanConfig {
            accel_sigma: 0.25,
            ..cfg
        },
    );
    row(
        "accel_sigma = 8 (bám sát đo)",
        &tr,
        KalmanConfig {
            accel_sigma: 8.0,
            ..cfg
        },
    );
    let drive5 = DriveConfig {
        sample_s: 5.0,
        ..DriveConfig::default()
    };
    let eval5 = p02::trips(&w, n, seed, &drive5);
    row(
        "lấy mẫu thưa 5 s (cùng tham số)",
        &p02::corrupt_all(&eval5, &noise(6.0, 8.0), seed),
        cfg,
    );
    let canyon_noise = |t: &Trip| {
        let end = t.truth.last().map_or(0.0, |p| p.t_ms as f64 / 1000.0);
        NoiseConfig {
            canyons: vec![Canyon {
                start_s: 0.3 * end,
                end_s: 0.7 * end,
                bias_m: 15.0,
                bearing_deg: 70.0,
                sigma_mult: 1.0,
                reported_mult: 1.0,
            }],
            ..noise(6.0, 8.0)
        }
    };
    let tr_canyon: Vec<Trace> = eval_trips
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let mut rng = map_match::Rng::new(seed ^ ((i as u64 + 1) * 31));
            map_match::trace::corrupt(&t.route, t.truth.clone(), &canyon_noise(t), &mut rng)
        })
        .collect();
    row(
        "lệch ổn định 15 m suốt 40% chuyến (hẻm cao tầng), accuracy không đổi",
        &tr_canyon,
        cfg,
    );

    // 6. trích dữ liệu quanh một khúc cua (cho hình vẽ)
    println!(
        "\n-- 6. trích: khúc cua gắt nhất trong chuyến báo cáo số 0 (cột: giây, thật lat,lng; thô lat,lng; Kalman lat,lng) --"
    );
    let t0 = &tr[0];
    let run = run_filter(&t0.obs, cfg);
    let at = (4..t0.obs.len().saturating_sub(4))
        .max_by(|&a, &b| {
            let d = |i: usize| {
                bearing_diff_deg(
                    t0.truth_of(i - 4).heading_deg,
                    t0.truth_of(i + 4).heading_deg,
                )
            };
            d(a).total_cmp(&d(b))
        })
        .unwrap_or(4);
    println!("t_s,truth_lat,truth_lng,raw_lat,raw_lng,kalman_lat,kalman_lng,truth_speed_mps");
    for i in at.saturating_sub(7)..(at + 8).min(t0.obs.len()) {
        let (t, o, k) = (t0.truth_of(i), t0.obs[i].pos, run.est[i]);
        println!(
            "{},{:.7},{:.7},{:.7},{:.7},{:.7},{:.7},{:.1}",
            t0.obs[i].t_ms / 1000,
            t.pos.lat(),
            t.pos.lng(),
            o.lat(),
            o.lng(),
            k.lat(),
            k.lng(),
            t.speed_mps
        );
    }
    println!("\nuptime lúc kết thúc: {}", p02::uptime());
}
