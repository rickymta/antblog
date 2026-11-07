//! LOC-43: cửa sổ lô, đợi bao lâu là đáng. Quét độ dài cửa sổ theo mật độ (thường, cao điểm), cửa sổ thích ứng,
//! thời gian chờ đổi lấy km rỗng. Tham số cửa sổ thích ứng chỉ chọn trên dải seed CHỈNH; báo cáo trên dải seed ĐÁNH GIÁ.
//!
//! Tiêu chí chọn đặt trước khi đo: cấu hình có `wait_s` trung bình (khách chờ từ lúc đặt tới lúc xe tới, đã gồm thời gian đợi
//! trong cửa sổ) thấp nhất trên seed chỉnh. Km rỗng báo cáo cạnh đó, không dùng để chọn.
//!
//! Chạy từ gốc repo:
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc43_cua_so_lo
//! Tham số: --seeds N (20), --tuning-seeds N (10), --jobs N (3), --fixture

use labs::Args;
use labs::p03::uptime;
use labs::p05::{jobs, n_seeds, print_paired, print_zones, select_variants, setup};
use matcher::batch::{AdaptiveWindow, Algo, BatchMatcher, MatcherConfig, Window};
use matcher::eval::{RunMetrics, Zones, column, eval_seeds, paired, sweep, tuning_seeds};
use simulator::{AtPin, RunOptions, Scenario, run};

fn run_cfg(
    engine: &route_engine::engine::Engine,
    sc: &Scenario,
    cfg: &MatcherConfig,
) -> simulator::SimOutput {
    let (m, _) = BatchMatcher::new(cfg.clone());
    run(
        engine,
        sc,
        Box::new(AtPin),
        Box::new(m),
        RunOptions { trace: true },
    )
}

fn cfgs_fixed() -> Vec<(String, MatcherConfig)> {
    let mut v = vec![(
        "tức thì".to_string(),
        MatcherConfig::new("immediate-hungarian", Algo::Hungarian, Window::Immediate),
    )];
    for w in [5u64, 10, 20, 30, 60] {
        v.push((
            format!("cố định {w} s"),
            MatcherConfig::new(
                &format!("hungarian-w{w}"),
                Algo::Hungarian,
                Window::Fixed(w * 1000),
            ),
        ));
    }
    v
}

fn adaptive_grid() -> Vec<(String, MatcherConfig)> {
    let mut v = Vec::new();
    for k in [1.0f64, 3.0] {
        for max_s in [10u64, 20, 40] {
            let a = AdaptiveWindow {
                k,
                min_ms: 2_000,
                max_ms: max_s * 1000,
                horizon_s: 300.0,
            };
            v.push((
                format!("thích ứng k={k} max={max_s}s"),
                MatcherConfig::new("hungarian-adaptive", Algo::Hungarian, Window::Adaptive(a)),
            ));
        }
    }
    v
}

fn mean(rs: &[RunMetrics], f: impl Fn(&RunMetrics) -> f64) -> f64 {
    let c = column(rs, f);
    c.iter().sum::<f64>() / c.len() as f64
}

fn row(name: &str, base: &[RunMetrics], r: &[RunMetrics]) {
    let cell = |f: fn(&RunMetrics) -> f64, d: usize| {
        let p = paired(&column(base, f), &column(r, f));
        format!(
            "{:+.d$} [{:.d$}, {:.d$}]{}",
            p.mean_diff,
            p.ci_lo,
            p.ci_hi,
            if p.significant() { "*" } else { " " }
        )
    };
    println!(
        "  {:<26} {:>22} {:>24} {:>20} {:>20} {:>20} {:>18} {:>22}",
        name,
        cell(|x| x.wait_mean, 1),
        cell(|x| x.deadhead_mean, 1),
        cell(|x| x.eta_mean, 1),
        cell(|x| x.wait_p95, 1),
        cell(|x| x.empty_m_per_request, 1),
        cell(|x| x.cancel_rate, 4),
        cell(|x| x.completed as f64, 1)
    );
}

fn header() {
    println!(
        "  {:<26} {:>22} {:>24} {:>20} {:>20} {:>20} {:>18} {:>22}",
        "cấu hình (hiệu so với tức thì)",
        "wait_s TB [KTC 95%]",
        "deadhead_m TB [KTC]",
        "pickup_eta_s TB",
        "wait_s p95",
        "km rỗng/khách (m)",
        "cancel_rate",
        "hoàn thành"
    );
}

fn main() {
    let args = Args::from_env();
    println!("== LOC-43: cửa sổ lô, đợi bao lâu là đáng (trên dữ liệu mô phỏng) ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let (engine, info, sc, fixture) = setup(&args);
    let zones = Zones::hcm();
    let n = n_seeds(&args);
    let nt = args.u64_or("--tuning-seeds", if fixture { 3 } else { 10 }) as usize;
    let (eval, tune) = (eval_seeds(n), tuning_seeds(nt));
    println!("đồ thị: {info}; kịch bản gốc: {}", sc.name);
    println!(
        "seed chỉnh (không dùng để báo cáo): {}…{} ({nt} seed); seed đánh giá: {}…{} ({n} seed)",
        tune[0],
        tune[nt - 1],
        eval[0],
        eval[n - 1]
    );
    println!(
        "thuật toán trong lô: Hungarian; chi phí cặp = ETA (CH). «tức thì» = Hungarian trên các khách đang chờ mỗi khi có sự kiện, không gom."
    );

    for (vname, vdesc, tweak) in select_variants(
        &args,
        if fixture {
            &["thường"]
        } else {
            &["thường", "cao điểm"]
        },
    ) {
        let mut s = sc.clone();
        tweak(&mut s);
        println!("\n=== mật độ «{vname}»: {vdesc} ===");

        // ---- chỉnh trên seed chỉnh ----
        println!(
            "\n  [chỉnh, seed {}…{}] KHÔNG dùng để kết luận; chỉ để chọn tham số cửa sổ thích ứng",
            tune[0],
            tune[nt - 1]
        );
        let fixed = cfgs_fixed();
        let grid = adaptive_grid();
        let mut tuned: Vec<(String, MatcherConfig, Vec<RunMetrics>)> = Vec::new();
        for (name, cfg) in fixed.iter().chain(grid.iter()) {
            let r = sweep(&s, &tune, &zones, jobs(&args), |sc| {
                run_cfg(&engine, sc, cfg)
            });
            tuned.push((name.clone(), cfg.clone(), r));
        }
        println!(
            "  {:<26} {:>12} {:>14}",
            "cấu hình", "wait_s TB", "deadhead_m TB"
        );
        for (name, _, r) in &tuned {
            println!(
                "  {:<26} {:>12.1} {:>14.1}",
                name,
                mean(r, |x| x.wait_mean),
                mean(r, |x| x.deadhead_mean)
            );
        }
        let best_adaptive = tuned
            .iter()
            .filter(|(nm, _, _)| nm.starts_with("thích ứng"))
            .min_by(|a, b| mean(&a.2, |x| x.wait_mean).total_cmp(&mean(&b.2, |x| x.wait_mean)))
            .expect("có lưới");
        let best_fixed = tuned
            .iter()
            .filter(|(nm, _, _)| nm.starts_with("cố định"))
            .min_by(|a, b| mean(&a.2, |x| x.wait_mean).total_cmp(&mean(&b.2, |x| x.wait_mean)))
            .expect("có cửa sổ cố định");
        println!(
            "  → chọn theo tiêu chí đặt trước (wait_s TB thấp nhất trên seed chỉnh): thích ứng = «{}», cố định = «{}»",
            best_adaptive.0, best_fixed.0
        );

        // ---- đánh giá ----
        println!(
            "\n  [đánh giá, seed {}…{}] hiệu = cấu hình − «tức thì», có cặp theo seed, KTC 95% (t Student); * = KTC không chứa 0",
            eval[0],
            eval[n - 1]
        );
        let base = sweep(&s, &eval, &zones, jobs(&args), |sc| {
            run_cfg(&engine, sc, &fixed[0].1)
        });
        println!(
            "  «tức thì» (tuyệt đối, trung bình {n} seed): wait_s {:.1}, deadhead_m {:.1}, pickup_eta_s {:.1}, wait_s p95 {:.1}, km rỗng/khách {:.1}, cancel {:.4}, hoàn thành {:.1}/{:.1}",
            mean(&base, |x| x.wait_mean),
            mean(&base, |x| x.deadhead_mean),
            mean(&base, |x| x.eta_mean),
            mean(&base, |x| x.wait_p95),
            mean(&base, |x| x.empty_m_per_request),
            mean(&base, |x| x.cancel_rate),
            mean(&base, |x| x.completed as f64),
            mean(&base, |x| x.requests as f64)
        );
        header();
        let mut evals: Vec<(String, Vec<RunMetrics>)> = Vec::new();
        for (name, cfg) in fixed.iter().skip(1) {
            let r = sweep(&s, &eval, &zones, jobs(&args), |sc| {
                run_cfg(&engine, sc, cfg)
            });
            row(name, &base, &r);
            evals.push((name.clone(), r));
        }
        let ra = sweep(&s, &eval, &zones, jobs(&args), |sc| {
            run_cfg(&engine, sc, &best_adaptive.1)
        });
        row(&format!("{} (đã chọn)", best_adaptive.0), &base, &ra);
        let nb = ra.iter().map(|r| r.extras.batches).sum::<usize>() as f64 / n as f64;
        println!("  (cửa sổ thích ứng đã chọn đóng {nb:.1} lô mỗi seed)");
        // chi tiết cho cửa sổ cố định tốt nhất trên seed chỉnh và cửa sổ thích ứng
        let bf = evals
            .iter()
            .find(|(nm, _)| *nm == best_fixed.0)
            .map(|(_, r)| r.as_slice())
            .unwrap_or(&base);
        print_paired(
            &format!(
                "  chi tiết: «tức thì» so với {} (đã chọn trên seed chỉnh)",
                best_fixed.0
            ),
            "tức thì",
            &best_fixed.0,
            &base,
            bf,
        );
        if vname == "thường" {
            print_zones("tức thì", &best_fixed.0, &base, bf);
        }
        print_paired(
            &format!("  chi tiết: «tức thì» so với {} (đã chọn)", best_adaptive.0),
            "tức thì",
            &best_adaptive.0,
            &base,
            &ra,
        );
    }
    println!(
        "\nGiả định: trên dữ liệu mô phỏng. Khách huỷ theo mô hình của kịch bản, bắt đầu tính từ lúc gán (không phải lúc đặt), nên đợi trong cửa sổ không làm tăng xác suất huỷ trực tiếp; chỉ có hết kiên nhẫn 300 s mới tính từ lúc đặt."
    );
    println!("uptime lúc kết thúc: {}", uptime());
}
