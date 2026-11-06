//! LOC-47: ghép nối chuyến: tài xế đang chở khách cũng là ứng viên cho chuyến kế khi sắp trả khách.
//! ETA còn lại chỉ là ước lượng (simulator có nhiễu tốc độ log-chuẩn mà chiến lược không biết); chuyến trước kéo dài thì khách chờ lâu hơn dự kiến.
//!
//! Chạy từ gốc repo:
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc47_chain_dispatch
//! Tham số: --seeds N (20), --jobs N (3), --fixture

use labs::Args;
use labs::p03::uptime;
use labs::p05::{jobs, n_seeds, print_paired, print_zones, setup, variants};
use matcher::batch::{Algo, BatchMatcher, ChainConfig, MatcherConfig, Window};
use matcher::eval::{RunMetrics, Zones, column, eval_seeds, paired, sweep};
use simulator::{AtPin, RunOptions, Scenario, run};

fn cfg(name: &str, chain: Option<ChainConfig>) -> MatcherConfig {
    let mut c = MatcherConfig::new(name, Algo::Hungarian, Window::Immediate);
    c.chain = chain;
    c
}

fn run_cfg(
    engine: &route_engine::engine::Engine,
    sc: &Scenario,
    c: &MatcherConfig,
) -> simulator::SimOutput {
    let (m, _) = BatchMatcher::new(c.clone());
    run(
        engine,
        sc,
        Box::new(AtPin),
        Box::new(m),
        RunOptions { trace: true },
    )
}

fn cell(a: &[RunMetrics], b: &[RunMetrics], f: fn(&RunMetrics) -> f64, d: usize) -> String {
    let p = paired(&column(a, f), &column(b, f));
    format!(
        "{:+.d$} [{:.d$}, {:.d$}]{}",
        p.mean_diff,
        p.ci_lo,
        p.ci_hi,
        if p.significant() { "*" } else { " " }
    )
}

fn table(a: &[RunMetrics], rows: &[(String, Vec<RunMetrics>)]) {
    println!(
        "  hiệu = chain − không chain (Hungarian ghép ngay), có cặp theo seed, KTC 95%; * = KTC không chứa 0"
    );
    println!(
        "  {:<22} {:>9} {:>9} {:>20} {:>22} {:>22} {:>22} {:>18} {:>20}",
        "cấu hình",
        "xếp hàng",
        "huỷ khi chờ",
        "wait_s TB",
        "wait_s p95",
        "wait_s max",
        "deadhead_m TB",
        "cancel_rate",
        "hoàn thành"
    );
    for (name, r) in rows {
        let ch = r.iter().map(|x| x.extras.chained as f64).sum::<f64>() / r.len() as f64;
        let cq = r
            .iter()
            .map(|x| x.extras.chain_cancelled_in_queue as f64)
            .sum::<f64>()
            / r.len() as f64;
        println!(
            "  {:<22} {:>9.1} {:>9.1} {:>20} {:>22} {:>22} {:>22} {:>18} {:>20}",
            name,
            ch,
            cq,
            cell(a, r, |x| x.wait_mean, 1),
            cell(a, r, |x| x.wait_p95, 1),
            cell(a, r, |x| x.wait_max, 1),
            cell(a, r, |x| x.deadhead_mean, 1),
            cell(a, r, |x| x.cancel_rate, 4),
            cell(a, r, |x| x.completed as f64, 1)
        );
    }
    println!("  (xếp hàng, huỷ khi chờ: trung bình mỗi seed)");
    println!(
        "  {:<22} {:>20} {:>22} {:>22} {:>24}",
        "cấu hình",
        "pickup_eta_s TB",
        "km rỗng/khách (m)",
        "Gini chuyến/xe",
        "rảnh dài nhất p95 (s)"
    );
    for (name, r) in rows {
        println!(
            "  {:<22} {:>20} {:>22} {:>22} {:>24}",
            name,
            cell(a, r, |x| x.eta_mean, 1),
            cell(a, r, |x| x.empty_m_per_request, 1),
            cell(a, r, |x| x.trips_gini, 3),
            cell(a, r, |x| x.idle_gap_p95_s, 0)
        );
    }
}

fn main() {
    let args = Args::from_env();
    println!(
        "== LOC-47: ghép nối chuyến, nhận chuyến kế khi sắp trả khách (trên dữ liệu mô phỏng) =="
    );
    println!("uptime lúc bắt đầu: {}", uptime());
    let (engine, info, sc, fixture) = setup(&args);
    let zones = Zones::hcm();
    let n = n_seeds(&args);
    let seeds = eval_seeds(n);
    println!(
        "đồ thị: {info}; kịch bản gốc: {}; {n} seed đánh giá {}…{}",
        sc.name,
        seeds[0],
        seeds[n - 1]
    );
    println!(
        "cơ chế: tài xế đang chở khách có `remaining_est` = phần còn lại theo kế hoạch (không biết nhiễu) + thời gian dừng trả khách; khách xếp hàng chờ xe đó, xe đi đón ngay khi trả khách xong."
    );
    println!(
        "chi phí cặp với xe sắp rảnh = remaining_est + ETA(điểm trả → điểm đón) + risk_k × độ lệch chuẩn của phần còn lại, độ lệch chuẩn ≈ remaining_est × (e^σ − 1) với σ = driver.speed_jitter của kịch bản."
    );
    println!(
        "chain-H: tầm nhìn H giây, risk_k = 0 (tin ước lượng); chain-120+r1: tầm nhìn 120 s, risk_k = 1."
    );

    let a_cfg = cfg("hungarian-immediate", None);
    let chains: Vec<(String, MatcherConfig)> = vec![
        (
            "chain-60".into(),
            cfg(
                "chain-60",
                Some(ChainConfig {
                    horizon_ms: 60_000,
                    risk_k: 0.0,
                }),
            ),
        ),
        (
            "chain-120".into(),
            cfg(
                "chain-120",
                Some(ChainConfig {
                    horizon_ms: 120_000,
                    risk_k: 0.0,
                }),
            ),
        ),
        (
            "chain-300".into(),
            cfg(
                "chain-300",
                Some(ChainConfig {
                    horizon_ms: 300_000,
                    risk_k: 0.0,
                }),
            ),
        ),
        (
            "chain-120+r1".into(),
            cfg(
                "chain-120-r1",
                Some(ChainConfig {
                    horizon_ms: 120_000,
                    risk_k: 1.0,
                }),
            ),
        ),
    ];
    let scenarios: Vec<(String, String, Scenario, Vec<&str>)> = {
        let vs = variants();
        let get = |name: &str| {
            let (nm, d, t) = vs.iter().find(|v| v.0 == name).expect("có bộ giả định");
            let mut s = sc.clone();
            t(&mut s);
            (nm.to_string(), d.to_string(), s)
        };
        let mut out = Vec::new();
        let (nm, d, s) = get("thưa xe");
        out.push((
            nm,
            d,
            s,
            vec!["chain-60", "chain-120", "chain-300", "chain-120+r1"],
        ));
        let (nm, d, s) = get("cao điểm");
        out.push((
            nm,
            d,
            s,
            vec!["chain-60", "chain-120", "chain-300", "chain-120+r1"],
        ));
        let (nm, d, s) = get("thường");
        out.push((nm, d, s, vec!["chain-120", "chain-300"]));
        let (nm, _, mut s) = get("thưa xe");
        s.driver.speed_jitter = 0.3;
        out.push((
            format!("{nm} + nhiễu tốc độ σ = 0,3"),
            "60 xe, 3,5/phút, chuyến kéo dài thất thường hơn baseline (σ 0,3 thay vì 0,1)".into(),
            s,
            vec!["chain-120", "chain-120+r1"],
        ));
        out
    };
    for (vname, vdesc, s, names) in scenarios.into_iter().take(if fixture { 1 } else { 4 }) {
        println!("\n=== bộ giả định «{vname}»: {vdesc} ===");
        let a = sweep(&s, &seeds, &zones, jobs(&args), |sc| {
            run_cfg(&engine, sc, &a_cfg)
        });
        println!(
            "  không chain (tuyệt đối, TB {n} seed): khách đặt {:.1}, hoàn thành {:.1}, cancel {:.4}, unserved {:.4}, wait_s {:.1} (p95 {:.1}), pickup_eta_s {:.1}, deadhead_m {:.1}",
            a.iter().map(|x| x.requests as f64).sum::<f64>() / n as f64,
            a.iter().map(|x| x.completed as f64).sum::<f64>() / n as f64,
            a.iter().map(|x| x.cancel_rate).sum::<f64>() / n as f64,
            a.iter().map(|x| x.unserved_rate).sum::<f64>() / n as f64,
            a.iter().map(|x| x.wait_mean).sum::<f64>() / n as f64,
            a.iter().map(|x| x.wait_p95).sum::<f64>() / n as f64,
            a.iter().map(|x| x.eta_mean).sum::<f64>() / n as f64,
            a.iter().map(|x| x.deadhead_mean).sum::<f64>() / n as f64
        );
        let mut rows = Vec::new();
        for (name, c) in chains.iter().filter(|(nm, _)| names.contains(&nm.as_str())) {
            let r = sweep(&s, &seeds, &zones, jobs(&args), |sc| {
                run_cfg(&engine, sc, c)
            });
            rows.push((name.clone(), r));
        }
        table(&a, &rows);
        if let Some((nm, r)) = rows.iter().find(|(nm, _)| nm == "chain-120") {
            print_paired(
                &format!("\n  chi tiết đủ chỉ số: không chain so với {nm}"),
                "không chain",
                nm,
                &a,
                r,
            );
            if vname == "thưa xe" {
                print_zones("không chain", nm, &a, r);
            }
        }
    }
    println!(
        "\nGiả định: trên dữ liệu mô phỏng. Mô hình huỷ của khách xếp hàng dùng ETA tổng (phần còn lại của chuyến trước + đón) như khách thường; không có chuyện tài xế từ chối chuyến kế hay đổi ý khi chuyến trước kéo dài."
    );
    println!("uptime lúc kết thúc: {}", uptime());
}
