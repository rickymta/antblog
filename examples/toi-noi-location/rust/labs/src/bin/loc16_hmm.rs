//! LOC-16: bám đường bằng HMM + Viterbi (Newson–Krumm) so với snap về cạnh gần nhất, trên vết tổng hợp có ground truth.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc16_hmm
//! Tham số: --trips N (chuyến mỗi mức nhiễu, mặc định 40), --seed S (báo cáo), --fixture

use geo_core::haversine_m;
use labs::Args;
use labs::p02::{self, EVAL_SEED, TUNE_SEED, Trip, World};
use map_match::Obs;
use map_match::eval::{Score, nearest_snap, score_trace};
use map_match::hmm::{MatchConfig, MatchCtx, MatchedPoint};
use map_match::kalman::{KalmanConfig, KalmanFilter};
use map_match::trace::{DriveConfig, NoiseConfig, Trace};
use road_graph::EdgeId;
use std::time::Instant;

fn noise(sigma: f64) -> NoiseConfig {
    NoiseConfig {
        sigma_m: sigma,
        ..NoiseConfig::default()
    }
}

struct Out {
    score: Score,
    secs: f64,
    points: usize,
    settled: u64,
    dijkstras: u64,
    cands: u64,
    breaks: usize,
}

fn kalman_filtered(obs: &[Obs]) -> Vec<Obs> {
    let mut kf = KalmanFilter::new(KalmanConfig::default());
    let mut last = None;
    obs.iter()
        .map(|o| {
            if let Some((e, _)) = kf.update(o) {
                last = Some(e.pos);
            }
            Obs {
                pos: last.unwrap_or(o.pos),
                ..*o
            }
        })
        .collect()
}

fn run_hmm(
    w: &World,
    traces: &[Trace],
    cfg: &MatchConfig,
    prep: impl Fn(&Trace) -> Vec<Obs>,
) -> Out {
    let mut ctx = MatchCtx::new(&w.g, &w.idx, cfg.clone());
    let mut o = Out {
        score: Score::default(),
        secs: 0.0,
        points: 0,
        settled: 0,
        dijkstras: 0,
        cands: 0,
        breaks: 0,
    };
    for tr in traces {
        let obs = prep(tr);
        let t = Instant::now();
        let r = ctx.match_offline(&obs);
        o.secs += t.elapsed().as_secs_f64();
        o.points += obs.len();
        o.breaks += r.breaks;
        o.score
            .add(&score_trace(&w.g, tr, &r.points, &r.route, None));
    }
    o.settled = ctx.router.settled_total;
    o.dijkstras = ctx.stats.dijkstras;
    o.cands = ctx.stats.candidates;
    o
}

fn run_snap(w: &World, traces: &[Trace]) -> Score {
    let mut s = Score::default();
    for tr in traces {
        let (pts, route) = nearest_snap(
            &w.g,
            &w.idx,
            map_match::hmm::MatchConfig::default().mode,
            &tr.obs,
        );
        s.add(&score_trace(&w.g, tr, &pts, &route, None));
    }
    s
}

fn raw_rmse(traces: &[Trace]) -> f64 {
    let (mut s, mut n) = (0.0, 0);
    for tr in traces {
        for i in 0..tr.obs.len() {
            s += haversine_m(tr.obs[i].pos, tr.truth_of(i).pos).powi(2);
            n += 1;
        }
    }
    (s / n.max(1) as f64).sqrt()
}

fn line(name: &str, s: &Score) -> String {
    format!(
        "{name:<38} {:>8.2}% {:>8.2}% {:>8.2}% {:>8.2}% {:>7.3} {:>9.2}",
        s.edge_pct(),
        s.edge_near_pct(),
        s.dir_near_pct(),
        s.recall_pct(),
        s.rmf(),
        s.mean_pos_err()
    )
}

const HEAD: &str = "                                        cạnh chặt  cạnh nới  chiều nới  recall    RMF  sai số vị trí (m)";

fn main() {
    let args = Args::from_env();
    let n = args.u64_or("--trips", 40) as usize;
    let seed = args.u64_or("--seed", EVAL_SEED);
    let w = p02::world(&args);
    p02::header(
        "LOC-16: HMM + Viterbi",
        &w,
        &format!(
            "{n} chuyến mỗi mức nhiễu (dài 2–4,5 km trên HCM, 0,4–1,5 km trên fixture); seed báo cáo {seed}, seed chọn tham số {TUNE_SEED}; xe ô tô"
        ),
    );
    let drive1 = DriveConfig::default();

    // 1. chọn β và hệ số accuracy → σ trên bộ chọn tham số
    println!(
        "-- 1. chọn tham số trên bộ chọn tham số (40 chuyến: 10 mỗi mức σ = 4, 8, 15 m ở 1 Hz, và 10 chuyến σ = 8 m ở 5 s/điểm; không dùng bộ báo cáo) --"
    );
    let tune_trips = p02::trips(&w, 30, TUNE_SEED, &drive1);
    let mut tune: Vec<Trace> = Vec::new();
    for (k, sigma) in [4.0, 8.0, 15.0].iter().enumerate() {
        tune.extend(p02::corrupt_all(
            &tune_trips[k * 10..(k + 1) * 10],
            &noise(*sigma),
            TUNE_SEED + k as u64,
        ));
    }
    let tune5 = p02::trips(
        &w,
        10,
        TUNE_SEED + 5,
        &DriveConfig {
            sample_s: 5.0,
            ..DriveConfig::default()
        },
    );
    tune.extend(p02::corrupt_all(&tune5, &noise(8.0), TUNE_SEED + 5));
    println!(
        "{:>8} {:>10} {:>12} {:>11} {:>11} {:>8} {:>8}",
        "beta_m", "beta_frac", "acc→σ", "cạnh chặt", "cạnh nới", "RMF", "gãy"
    );
    let mut best: Option<(f64, MatchConfig)> = None;
    for beta in [1.0, 2.0, 3.0, 6.0, 10.0] {
        for frac in [0.0, 0.1, 0.3] {
            for a2s in [0.5, 0.67, 1.0, 1.5] {
                let cfg = MatchConfig {
                    beta_m: beta,
                    beta_frac: frac,
                    noise: map_match::NoiseModel {
                        accuracy_to_sigma: a2s,
                        ..map_match::NoiseModel::default()
                    },
                    ..MatchConfig::default()
                };
                let o = run_hmm(&w, &tune, &cfg, |t| t.obs.clone());
                println!(
                    "{beta:>8} {frac:>10} {a2s:>12} {:>10.2}% {:>10.2}% {:>8.3} {:>8}",
                    o.score.edge_pct(),
                    o.score.edge_near_pct(),
                    o.score.rmf(),
                    o.breaks
                );
                let key = o.score.edge_near_pct() - 10.0 * o.score.rmf();
                if best.as_ref().is_none_or(|b| key > b.0) {
                    best = Some((key, cfg));
                }
            }
        }
    }
    let (_, tuned) = best.expect("có lưới");
    let d = MatchConfig::default();
    println!(
        "chọn (tối đa cạnh nới% − 10·RMF): beta_m = {}, beta_frac = {}, accuracy_to_sigma = {}; mặc định trong MatchConfig::default(): {}, {}, {}\n",
        tuned.beta_m,
        tuned.beta_frac,
        tuned.noise.accuracy_to_sigma,
        d.beta_m,
        d.beta_frac,
        d.noise.accuracy_to_sigma
    );
    let cfg = d;

    // 2. bộ báo cáo
    let eval_trips = p02::trips(&w, n, seed, &drive1);
    println!(
        "-- 2. độ chính xác theo độ lệch chuẩn nhiễu (bộ báo cáo, lấy mẫu 1 Hz, tương quan 8 s, MatchConfig::default()) --"
    );
    println!("{HEAD}");
    for sigma in [3.0, 6.0, 10.0, 15.0, 25.0] {
        let tr = p02::corrupt_all(&eval_trips, &noise(sigma), seed);
        let pts: usize = tr.iter().map(|t| t.obs.len()).sum();
        println!("σ = {sigma} m: {pts} điểm, RMSE thô {:.2} m", raw_rmse(&tr));
        println!("{}", line("  snap về cạnh gần nhất", &run_snap(&w, &tr)));
        let o = run_hmm(&w, &tr, &cfg, |t| t.obs.clone());
        println!(
            "{}  (gãy chuỗi {})",
            line("  HMM + Viterbi offline", &o.score),
            o.breaks
        );
    }

    // 3. lấy mẫu thưa
    println!("\n-- 3. lấy mẫu thưa (σ = 6 m): app tiết kiệm pin gửi 5 s hoặc 10 s một điểm --");
    println!("{HEAD}");
    for period in [1.0, 5.0, 10.0] {
        let trips5: Vec<Trip> = p02::trips(
            &w,
            n,
            seed,
            &DriveConfig {
                sample_s: period,
                ..DriveConfig::default()
            },
        );
        let tr = p02::corrupt_all(&trips5, &noise(6.0), seed);
        println!(
            "chu kỳ {period} s: {} điểm",
            tr.iter().map(|t| t.obs.len()).sum::<usize>()
        );
        println!("{}", line("  snap về cạnh gần nhất", &run_snap(&w, &tr)));
        let o = run_hmm(&w, &tr, &cfg, |t| t.obs.clone());
        println!(
            "{}  (gãy chuỗi {})",
            line("  HMM + Viterbi offline", &o.score),
            o.breaks
        );
    }

    // 4. cắt bớt thành phần (σ = 10 m)
    println!("\n-- 4. thay đổi từng thành phần của mô hình (σ = 10 m, 1 Hz) --");
    println!("{HEAD}");
    let tr10 = p02::corrupt_all(&eval_trips, &noise(10.0), seed);
    println!("{}", line("snap về cạnh gần nhất", &run_snap(&w, &tr10)));
    println!(
        "{}",
        line(
            "HMM mặc định",
            &run_hmm(&w, &tr10, &cfg, |t| t.obs.clone()).score
        )
    );
    println!(
        "{}",
        line(
            "HMM trên điểm đã lọc Kalman (LOC-15)",
            &run_hmm(&w, &tr10, &cfg, |t| kalman_filtered(&t.obs)).score
        )
    );
    let no_acc = |t: &Trace| {
        t.obs
            .iter()
            .map(|o| Obs {
                accuracy_m: None,
                ..*o
            })
            .collect::<Vec<_>>()
    };
    println!(
        "{}",
        line(
            "HMM bỏ accuracy (σ cố định 8 m)",
            &run_hmm(&w, &tr10, &cfg, no_acc).score
        )
    );
    let mut c = cfg.clone();
    c.heading_sigma_deg = Some(30.0);
    println!(
        "{}",
        line(
            "HMM thêm số hạng hướng đi (σ_h 30°)",
            &run_hmm(&w, &tr10, &c, |t| t.obs.clone()).score
        )
    );
    let mut c = cfg.clone();
    c.max_candidates = 3;
    println!(
        "{}",
        line(
            "HMM chỉ 3 ứng viên mỗi điểm",
            &run_hmm(&w, &tr10, &c, |t| t.obs.clone()).score
        )
    );
    let mut c = cfg.clone();
    c.radius_m = 25.0;
    println!(
        "{}",
        line(
            "HMM bán kính tìm 25 m (nhỏ hơn nhiễu)",
            &run_hmm(&w, &tr10, &c, |t| t.obs.clone()).score
        )
    );
    let mut c = cfg.clone();
    c.beta_m = 30.0;
    c.beta_frac = 0.0;
    println!(
        "{}",
        line(
            "HMM beta_m = 30 (quá lỏng)",
            &run_hmm(&w, &tr10, &c, |t| t.obs.clone()).score
        )
    );
    let mut c = cfg.clone();
    c.backward_beta_m = 1e9;
    println!(
        "{}",
        line(
            "HMM không phạt đi lùi trên cùng cạnh",
            &run_hmm(&w, &tr10, &c, |t| t.obs.clone()).score
        )
    );

    // 4b. điểm nhảy xa làm gãy chuỗi
    println!("\n-- 4b. điểm nhảy xa 25–70 m (đa đường) và gãy chuỗi (σ = 10 m, 1 Hz) --");
    println!("{HEAD}");
    for p_out in [0.0, 0.005, 0.03] {
        let nz = NoiseConfig {
            p_outlier: p_out,
            ..noise(10.0)
        };
        let tr = p02::corrupt_all(&eval_trips, &nz, seed);
        let o = run_hmm(&w, &tr, &cfg, |t| t.obs.clone());
        println!(
            "{}  (gãy chuỗi {})",
            line(&format!("HMM, {}% điểm nhảy xa", p_out * 100.0), &o.score),
            o.breaks
        );
        println!(
            "{}",
            line(
                &format!("snap, {}% điểm nhảy xa", p_out * 100.0),
                &run_snap(&w, &tr)
            )
        );
    }

    // 5. chi phí CPU
    println!(
        "\n-- 5. chi phí CPU của HMM offline (1 luồng, σ = 10 m; máy chịu tải: xem uptime, chỉ so sánh tương đối) --"
    );
    println!("uptime: {}", p02::uptime());
    for (name, k) in [("8 ứng viên (mặc định)", 8usize), ("3 ứng viên", 3)] {
        let mut c = cfg.clone();
        c.max_candidates = k;
        let o = run_hmm(&w, &tr10, &c, |t| t.obs.clone());
        println!(
            "{name:<24} {:>8.1} µs/điểm; {:>6.1} ứng viên/điểm; {:>5.1} lần Dijkstra/điểm; {:>7.1} nút duyệt/điểm",
            1e6 * o.secs / o.points as f64,
            o.cands as f64 / o.points as f64,
            o.dijkstras as f64 / o.points as f64,
            o.settled as f64 / o.points as f64
        );
    }
    let t = Instant::now();
    let mut s = 0usize;
    for tr in &tr10 {
        s += run_snap(&w, std::slice::from_ref(tr)).n;
    }
    println!(
        "snap về cạnh gần nhất: {:.2} µs/điểm ({s} điểm)",
        1e6 * t.elapsed().as_secs_f64() / s as f64
    );

    // 6. một ví dụ snap nhảy mà HMM giữ đúng
    println!(
        "\n-- 6. trích: đoạn trên đường từ tertiary trở lên mà snap gán sai cạnh còn HMM đúng (σ = 10 m; cột: giây, cạnh thật, cạnh snap, cạnh HMM, tọa độ thô, tọa độ HMM) --"
    );
    let mut ctx = MatchCtx::new(&w.g, &w.idx, cfg.clone());
    'find: for tr in &tr10 {
        let (sp, _) = nearest_snap(&w.g, &w.idx, cfg.mode, &tr.obs);
        let r = ctx.match_offline(&tr.obs);
        let canon = |e: EdgeId| map_match::eval::canon(&w.g, e);
        let bad = |m: &Option<MatchedPoint>, i: usize| {
            m.is_none_or(|m| canon(m.edge) != canon(tr.truth_of(i).edge))
        };
        for i in 6..tr.obs.len().saturating_sub(6) {
            let window = i - 3..i + 4;
            // chọn đoạn trên đường từ tertiary trở lên (không phải hẻm) và xe đang chạy, để hình minh họa có ý nghĩa
            let main_road = window.clone().all(|k| {
                w.g.class[tr.truth_of(k).edge as usize] <= road_graph::HighwayClass::Tertiary as u8
                    && tr.truth_of(k).speed_mps > 3.0
            });
            if main_road
                && window.clone().filter(|&k| bad(&sp[k], k)).count() >= 3
                && window.clone().all(|k| !bad(&r.points[k], k))
            {
                println!(
                    "t_s,truth_edge,snap_edge,hmm_edge,raw_lat,raw_lng,hmm_lat,hmm_lng,truth_street"
                );
                #[allow(clippy::needless_range_loop)]
                for k in i - 3..i + 4 {
                    let name = |e: EdgeId| w.g.edge(e).name().unwrap_or("(không tên)").to_string();
                    let (t, s, h) = (tr.truth_of(k), sp[k].unwrap(), r.points[k].unwrap());
                    println!(
                        "{},{},{},{},{:.7},{:.7},{:.7},{:.7},{}",
                        tr.obs[k].t_ms / 1000,
                        t.edge,
                        s.edge,
                        h.edge,
                        tr.obs[k].pos.lat(),
                        tr.obs[k].pos.lng(),
                        h.pos.lat(),
                        h.pos.lng(),
                        name(t.edge)
                    );
                }
                break 'find;
            }
        }
    }
    println!("\nuptime lúc kết thúc: {}", p02::uptime());
}
