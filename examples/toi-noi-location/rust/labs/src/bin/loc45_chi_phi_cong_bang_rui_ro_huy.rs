//! LOC-45: chi phí ghép ngoài quãng đường: công bằng cho tài xế (thời gian rảnh), giới hạn đón xa, rủi ro huỷ, và tránh
//! làm xấu đi một khu vực mà con số toàn cục che mất. Trọng số là chính sách có `version`, đặt TRƯỚC khi đo, không chỉnh theo kết quả.
//!
//! Chạy từ gốc repo:
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc45_chi_phi_cong_bang_rui_ro_huy
//! Tham số: --seeds N (20), --jobs N (3), --fixture

use labs::Args;
use labs::p03::uptime;
use labs::p05::{jobs, n_seeds, print_paired, print_zones, setup, variants};
use matcher::batch::{Algo, BatchMatcher, MatcherConfig, Window};
use matcher::eval::{RunMetrics, ZONE_NAMES, Zones, column, eval_seeds, paired, sweep};
use matcher::policy::CostPolicy;
use simulator::{AtPin, RunOptions, Scenario, run};

/// Chính sách đặt trước. Mọi bộ trọng số đều được đo và báo cáo, kể cả bộ xấu.
fn policies() -> Vec<(&'static str, CostPolicy)> {
    let v0 = CostPolicy::eta_only();
    let mk = |version: &str, f: &dyn Fn(&mut CostPolicy)| {
        let mut p = v0.clone();
        p.version = version.into();
        f(&mut p);
        p
    };
    vec![
        ("v0-eta (chỉ ETA)", v0.clone()),
        (
            "v1-risk (+ xác suất huỷ × 600 s)",
            mk("v1-risk", &|p| p.risk_ms = 600_000.0),
        ),
        (
            "v1-age (+ thưởng chờ lâu 0,5)",
            mk("v1-age", &|p| p.age_bonus = 0.5),
        ),
        (
            "v1-idle (+ thưởng xe rảnh lâu 0,1)",
            mk("v1-idle", &|p| p.idle_bonus = 0.1),
        ),
        (
            "v1-limit (đón xa: mềm 300 s ×1, cứng 600 s)",
            mk("v1-limit", &|p| {
                p.soft_pickup_ms = 300_000;
                p.soft_w = 1.0;
                p.hard_pickup_ms = Some(600_000);
            }),
        ),
        (
            "v1-all (tất cả các số hạng trên)",
            mk("v1-all", &|p| {
                p.risk_ms = 600_000.0;
                p.age_bonus = 0.5;
                p.idle_bonus = 0.1;
                p.soft_pickup_ms = 300_000;
                p.soft_w = 1.0;
                p.hard_pickup_ms = Some(600_000);
            }),
        ),
    ]
}

fn run_policy(
    engine: &route_engine::engine::Engine,
    sc: &Scenario,
    p: &CostPolicy,
) -> simulator::SimOutput {
    let mut cfg = MatcherConfig::new(&p.version, Algo::Hungarian, Window::Immediate);
    cfg.policy = p.clone();
    let (m, _) = BatchMatcher::new(cfg);
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

fn compact(a: &[RunMetrics], rows: &[(String, Vec<RunMetrics>)]) {
    println!(
        "  hiệu = chính sách − v0-eta, có cặp theo seed, KTC 95%; * = KTC không chứa 0. ETA, wait, deadhead tính trên chuyến hoàn thành."
    );
    println!(
        "  {:<44} {:>20} {:>20} {:>22} {:>22} {:>20} {:>18}",
        "chính sách",
        "pickup_eta_s TB",
        "wait_s TB",
        "wait_s p95",
        "deadhead_m TB",
        "chuyến hoàn thành",
        "cancel_rate"
    );
    for (name, r) in rows {
        println!(
            "  {:<44} {:>20} {:>20} {:>22} {:>22} {:>20} {:>18}",
            name,
            cell(a, r, |x| x.eta_mean, 1),
            cell(a, r, |x| x.wait_mean, 1),
            cell(a, r, |x| x.wait_p95, 1),
            cell(a, r, |x| x.deadhead_mean, 1),
            cell(a, r, |x| x.completed as f64, 1),
            cell(a, r, |x| x.cancel_rate, 4)
        );
    }
    println!(
        "\n  Công bằng cho tài xế (số chuyến hoàn thành mỗi tài xế, khoảng rảnh liên tiếp dài nhất của mỗi tài xế):"
    );
    println!(
        "  {:<44} {:>12} {:>22} {:>20} {:>22} {:>22}",
        "chính sách",
        "Gini (v0)",
        "Gini hiệu",
        "xe 0 chuyến hiệu",
        "rảnh dài nhất p95 (s)",
        "rảnh dài nhất max (s)"
    );
    for (name, r) in rows {
        println!(
            "  {:<44} {:>12.3} {:>22} {:>20} {:>22} {:>22}",
            name,
            a.iter().map(|x| x.trips_gini).sum::<f64>() / a.len() as f64,
            cell(a, r, |x| x.trips_gini, 3),
            cell(a, r, |x| x.zero_trip_share, 3),
            cell(a, r, |x| x.idle_gap_p95_s, 0),
            cell(a, r, |x| x.idle_gap_max_s, 0)
        );
    }
    println!(
        "\n  Công bằng theo vùng: hiệu của tỉ lệ khách ĐƯỢC PHỤC VỤ (hoàn thành) và wait_s TB của khách ở từng vùng; vùng = khoảng cách ghim tới Chợ Bến Thành."
    );
    println!(
        "  {:<44} {}",
        "chính sách",
        ZONE_NAMES
            .iter()
            .map(|z| format!("{z:<44}"))
            .collect::<Vec<_>>()
            .join(" ")
    );
    for (name, r) in rows {
        let zc = |z: usize| {
            let comp = |rs: &[RunMetrics]| {
                column(rs, |x| {
                    x.zone[z].completed as f64 / x.zone[z].requests.max(1) as f64
                })
            };
            let pc = paired(&comp(a), &comp(r));
            let pw = paired(
                &column(a, |x| x.zone[z].wait_mean),
                &column(r, |x| x.zone[z].wait_mean),
            );
            format!(
                "{:<44}",
                format!(
                    "hoàn thành {:+.3}{} · wait {:+.0} s{}",
                    pc.mean_diff,
                    if pc.significant() { "*" } else { "" },
                    pw.mean_diff,
                    if pw.significant() { "*" } else { "" }
                )
            )
        };
        println!("  {:<44} {} {} {}", name, zc(0), zc(1), zc(2));
    }
}

fn main() {
    let args = Args::from_env();
    println!("== LOC-45: chi phí ghép ngoài quãng đường (trên dữ liệu mô phỏng) ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let (engine, info, sc, fixture) = setup(&args);
    let zones = Zones::hcm();
    let n = n_seeds(&args);
    let seeds = eval_seeds(n);
    println!("đồ thị: {info}; kịch bản gốc: {}", sc.name);
    println!(
        "{n} seed đánh giá {}…{}; thuật toán: Hungarian, ghép ngay (không cửa sổ); chi phí cặp theo chính sách có version:",
        seeds[0],
        seeds[n - 1]
    );
    println!(
        "  cost = w_eta·ETA + soft_w·max(0, ETA − mềm) + risk·P(huỷ) − age·(khách đã chờ) − idle·(xe đã rảnh); cấm cặp nếu ETA > cứng"
    );
    for (name, p) in policies() {
        println!(
            "  [{}] {name}: w_eta {}, mềm {} ms ×{}, cứng {:?} ms, risk {} ms, age {}, idle {}",
            p.version,
            p.w_eta,
            p.soft_pickup_ms,
            p.soft_w,
            p.hard_pickup_ms,
            p.risk_ms,
            p.age_bonus,
            p.idle_bonus
        );
    }
    println!(
        "  Ghi chú: xác suất huỷ là đầu vào lấy từ mô hình huỷ của kịch bản (chiến lược «biết» mô hình). Bộ «huỷ nặng» và «huỷ theo chờ» đổi mô hình THẬT của simulator mà chính sách vẫn dùng đúng mô hình đó (không sai lệch); sai lệch được thử riêng ở cuối."
    );

    let all = policies();
    let wanted: Vec<(&str, &str, labs::p05::Tweak)> = variants()
        .into_iter()
        .filter(|(nm, _, _)| ["thường", "thưa xe", "huỷ nặng"].contains(nm))
        .take(if fixture { 1 } else { 3 })
        .collect();
    for (vname, vdesc, tweak) in wanted {
        let mut s = sc.clone();
        tweak(&mut s);
        println!("\n=== bộ giả định «{vname}»: {vdesc} ===");
        let chosen: Vec<&(&str, CostPolicy)> = if vname == "huỷ nặng" {
            all.iter()
                .filter(|(nm, _)| {
                    nm.starts_with("v0") || nm.starts_with("v1-risk") || nm.starts_with("v1-all")
                })
                .collect()
        } else {
            all.iter().collect()
        };
        let mut results: Vec<(String, Vec<RunMetrics>)> = Vec::new();
        for (name, p) in &chosen {
            let r = sweep(&s, &seeds, &zones, jobs(&args), |sc| {
                run_policy(&engine, sc, p)
            });
            results.push(((*name).to_string(), r));
        }
        let a = results[0].1.clone();
        println!(
            "  v0-eta (tuyệt đối, TB {n} seed): khách đặt {:.1}, hoàn thành {:.1}, cancel {:.4}, wait_s {:.1}, deadhead_m {:.1}; Gini số chuyến/xe {:.3}",
            a.iter().map(|x| x.requests as f64).sum::<f64>() / n as f64,
            a.iter().map(|x| x.completed as f64).sum::<f64>() / n as f64,
            a.iter().map(|x| x.cancel_rate).sum::<f64>() / n as f64,
            a.iter().map(|x| x.wait_mean).sum::<f64>() / n as f64,
            a.iter().map(|x| x.deadhead_mean).sum::<f64>() / n as f64,
            a.iter().map(|x| x.trips_gini).sum::<f64>() / n as f64
        );
        compact(&a, &results[1..]);
        if let Some((nm, r)) = results.iter().find(|(nm, _)| nm.starts_with("v1-all")) {
            print_paired(
                &format!("\n  chi tiết đủ chỉ số: v0-eta so với {nm}"),
                "v0-eta",
                nm,
                &a,
                r,
            );
            print_zones("v0-eta", nm, &a, r);
        }
    }

    // sai lệch mô hình huỷ: chiến lược tin mô hình mặc định, simulator dùng mô hình «huỷ nặng»
    if !fixture {
        println!(
            "\n=== sai lệch mô hình huỷ: simulator dùng «huỷ nặng» (0,8 từ ETA 120 s), chính sách v1-risk vẫn tin mô hình mặc định (0,5 từ 180 s) ==="
        );
        let mut s = sc.clone();
        (variants()
            .into_iter()
            .find(|v| v.0 == "huỷ nặng")
            .expect("có")
            .2)(&mut s);
        let base = sc.cancel.clone();
        let p = policies().remove(1).1;
        let a = sweep(&s, &seeds, &zones, jobs(&args), |c| {
            run_policy(&engine, c, &CostPolicy::eta_only())
        });
        let b = sweep(&s, &seeds, &zones, jobs(&args), |c| {
            let mut cfg =
                MatcherConfig::new("v1-risk-misspecified", Algo::Hungarian, Window::Immediate);
            cfg.policy = p.clone();
            cfg.assumed_cancel = Some(base.clone());
            let (m, _) = BatchMatcher::new(cfg);
            run(
                &engine,
                c,
                Box::new(AtPin),
                Box::new(m),
                RunOptions { trace: true },
            )
        });
        print_paired(
            "  so sánh có cặp:",
            "v0-eta",
            "v1-risk (tin sai mô hình huỷ)",
            &a,
            &b,
        );
    }
    println!(
        "\nGiả định: trên dữ liệu mô phỏng; mô hình huỷ và nhu cầu là giả định của kịch bản. Công bằng cho tài xế đo bằng số chuyến và khoảng rảnh, không phải thu nhập thật."
    );
    println!("uptime lúc kết thúc: {}", uptime());
}
