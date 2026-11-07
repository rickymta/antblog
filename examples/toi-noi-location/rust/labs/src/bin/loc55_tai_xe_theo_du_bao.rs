//! LOC-55: chạy lại LOC-49 (tài xế rảnh đi đâu) với bảng nhu cầu do **dự báo** thay cho bảng cố định, dùng `Repositioner` của simulator.
//! Các bảng do `loc55_demand_forecast.py` xuất (`python/p06/artifacts/demand/scenarios.json`): `fixed` (chỉ phụ thuộc vị trí, kiểu LOC-49,
//! cùng tổng với dự báo), `lastweek` (cùng giờ tuần trước), `model` (LightGBM Poisson); thêm `oracle` = trung bình số khách của 30 seed CHỈNH
//! trên đúng kịch bản (giới hạn trên mà một dự báo hoàn hảo về phân bố đạt được).
//!
//! Thế giới: cùng thế giới ẩn của LOC-51 (cầu theo giờ và điểm nóng theo ngày, giao thông Synthetic cộng hiệu ứng ẩn), 20 seed ĐÁNH GIÁ có cặp.
//!
//! Chạy từ gốc repo (cần `loc55_demand_forecast.py` chạy trước):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc55_tai_xe_theo_du_bao
//! Tham số: --seeds N (20)  --jobs N (4)  --accept P (0.7)  --full (in đủ bảng chỉ số)  --only TÊN (chỉ một kịch bản)

use geo_core::h3::{CellIndex, cell_of, resolution};
use labs::p03::uptime;
use labs::p05::{jobs, n_seeds, print_paired, print_zones, setup};
use labs::p06::{DemandWorld, HiddenStatic, RunTruth, scenario_for};
use labs::{Args, die, lab_root};
use matcher::batch::{Algo, BatchMatcher, MatcherConfig, Window};
use matcher::eval::{RunMetrics, Zones, column, eval_seeds, paired, sweep, tuning_seeds};
use matcher::reposition::{DemandReposition, DemandTable, Mode, RepoConfig};
use route_engine::engine::Engine;
use rustc_hash::FxHashMap;
use simulator::world::generate;
use simulator::{AtPin, Extras, Hooks, RunOptions, Scenario, run_hooked};
use std::sync::Arc;

const RES: u8 = 8;

fn read_table(engine: &Engine, path: &std::path::Path) -> DemandTable {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| die(&format!("{}: {e}", path.display())));
    let mut m: FxHashMap<CellIndex, f64> = FxHashMap::default();
    for l in text.lines().skip(1) {
        let (c, v) = l.split_once(',').unwrap_or_else(|| die("dòng CSV sai"));
        let cell: CellIndex = c.parse().unwrap_or_else(|_| die(&format!("ô H3 sai: {c}")));
        *m.entry(cell).or_default() += v.parse::<f64>().unwrap_or(0.0);
    }
    DemandTable::from_per_hour(engine, RES, &m)
}

fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len() as f64;
    let (ma, mb) = (a.iter().sum::<f64>() / n, b.iter().sum::<f64>() / n);
    let cov: f64 = a.iter().zip(b).map(|(x, y)| (x - ma) * (y - mb)).sum();
    let (va, vb) = (
        a.iter().map(|x| (x - ma).powi(2)).sum::<f64>(),
        b.iter().map(|y| (y - mb).powi(2)).sum::<f64>(),
    );
    cov / (va.sqrt() * vb.sqrt()).max(1e-12)
}

struct Arm {
    name: &'static str,
    table: Option<DemandTable>,
}

#[allow(clippy::too_many_arguments)]
fn run_arm(
    engine: &Engine,
    hidden: &Arc<HiddenStatic>,
    day: u32,
    sc: &Scenario,
    arm: &Arm,
    accept: f64,
) -> simulator::SimOutput {
    let (m, _) = BatchMatcher::new(MatcherConfig::new(
        "hungarian-immediate",
        Algo::Hungarian,
        Window::Immediate,
    ));
    let extras = match &arm.table {
        None => Extras::default(),
        Some(t) => {
            let (rep, _) = DemandReposition::new(
                arm.name,
                t.clone(),
                RepoConfig {
                    mode: Mode::Marginal,
                    ..RepoConfig::default()
                },
            );
            Extras {
                repositioner: Some(Box::new(rep)),
                reposition_accept_prob: accept,
            }
        }
    };
    run_hooked(
        engine,
        sc,
        Box::new(AtPin),
        Box::new(m),
        RunOptions { trace: true },
        extras,
        Hooks {
            truth: Some(Box::new(RunTruth {
                st: Arc::clone(hidden),
                day,
                start_hour: sc.start_hour,
            })),
            observer: None,
        },
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

fn main() {
    let args = Args::from_env();
    println!("== LOC-55: chạy lại LOC-49 với bảng nhu cầu dự báo (trên dữ liệu mô phỏng) ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let (engine, info, base, fixture) = setup(&args);
    let art = args.value("--art").map_or_else(
        || lab_root().join("python/p06/artifacts"),
        std::path::PathBuf::from,
    );
    let dir = art.join("demand");
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("scenarios.json"))
            .unwrap_or_else(|e| die(&format!("cần loc55_demand_forecast.py chạy trước: {e}"))),
    )
    .expect("json");
    let world_seed = manifest["world_seed"].as_u64().expect("world_seed");
    let hidden = HiddenStatic::new(&engine, 1.0, world_seed);
    let dw = DemandWorld::new(world_seed);
    let n = n_seeds(&args);
    let (eval, tune) = (eval_seeds(n), tuning_seeds(if fixture { 3 } else { 30 }));
    let accept: f64 = args
        .value("--accept")
        .map_or(0.7, |v| v.parse().unwrap_or(0.7));
    let zones = Zones::hcm();
    println!(
        "đồ thị: {info}; {n} seed ĐÁNH GIÁ {}…{}; bảng «oracle» dựng từ {} seed CHỈNH {}…{}; tài xế nghe gợi ý với xác suất {accept}; luật gợi ý của LOC-49 (không đổi, không chỉnh lại cho bảng dự báo)",
        eval[0],
        eval[n - 1],
        tune.len(),
        tune[0],
        tune[tune.len() - 1]
    );
    for sc_json in manifest["scenarios"].as_array().expect("mảng") {
        let name = sc_json["name"].as_str().expect("name");
        if args.value("--only").is_some_and(|w| w != name) {
            continue;
        }
        let (day, hour) = (
            sc_json["day"].as_u64().expect("day") as u32,
            sc_json["hour"].as_u64().expect("hour") as u32,
        );
        let sc0 = scenario_for(&base, &dw, day, hour, 0);
        println!(
            "\n=== kịch bản {name}: ngày {day} (thứ {}), {hour:02}h; {} xe, {:.2} yêu cầu/phút ===",
            ["Hai", "Ba", "Tư", "Năm", "Sáu", "Bảy", "Chủ nhật"][(day % 7) as usize],
            sc0.drivers,
            sc0.demand.requests_per_min
        );
        // oracle: số khách trung bình trên seed chỉnh
        let mut acc: FxHashMap<CellIndex, f64> = FxHashMap::default();
        for &s in &tune {
            let mut sc = sc0.clone();
            sc.seed = s;
            for c in &generate(&engine, &sc).customers {
                *acc.entry(cell_of(c.view.pin, resolution(RES))).or_default() += 1.0;
            }
        }
        let per_hour: FxHashMap<CellIndex, f64> = acc
            .iter()
            .map(|(&c, &v)| (c, v / tune.len() as f64 * 3600.0 / sc0.duration_s))
            .collect();
        let oracle = DemandTable::from_per_hour(&engine, RES, &per_hour);
        let files = &sc_json["tables"];
        let arms = [
            Arm {
                name: "không gợi ý",
                table: None,
            },
            Arm {
                name: "bảng cố định (chỉ vị trí)",
                table: Some(read_table(
                    &engine,
                    &art.join(files["fixed"].as_str().expect("fixed")),
                )),
            },
            Arm {
                name: "cùng giờ tuần trước",
                table: Some(read_table(
                    &engine,
                    &dir.join("..")
                        .join(files["lastweek"].as_str().expect("lastweek")),
                )),
            },
            Arm {
                name: "dự báo LightGBM",
                table: Some(read_table(
                    &engine,
                    &art.join(files["model"].as_str().expect("model")),
                )),
            },
            Arm {
                name: "oracle (30 seed chỉnh)",
                table: Some(oracle),
            },
        ];
        // độ khớp của từng bảng với nhu cầu kỳ vọng của kịch bản (oracle)
        let keys: Vec<CellIndex> = {
            let mut k: Vec<CellIndex> = per_hour.keys().copied().collect();
            for a in &arms[1..4] {
                k.extend(a.table.as_ref().expect("bảng").cells.iter().map(|c| c.cell));
            }
            k.sort_by_key(|c| u64::from(*c));
            k.dedup();
            k
        };
        let truth: Vec<f64> = keys
            .iter()
            .map(|k| per_hour.get(k).copied().unwrap_or(0.0))
            .collect();
        println!(
            "  độ khớp của bảng với nhu cầu kỳ vọng của kịch bản (trung bình 30 seed chỉnh; {} ô):",
            keys.len()
        );
        for a in &arms[1..4] {
            let t = a.table.as_ref().expect("bảng");
            let m: FxHashMap<CellIndex, f64> =
                t.cells.iter().map(|c| (c.cell, c.per_hour)).collect();
            let v: Vec<f64> = keys
                .iter()
                .map(|k| m.get(k).copied().unwrap_or(0.0))
                .collect();
            let l1 = v
                .iter()
                .zip(&truth)
                .map(|(x, y)| (x - y).abs())
                .sum::<f64>()
                / truth.iter().sum::<f64>();
            println!(
                "    {:<28} tổng {:>7.1}/giờ (thực {:>6.1}); Pearson {:.3}; Σ|bảng − thực| / Σ thực = {:.3}",
                a.name,
                t.total_per_hour(),
                truth.iter().sum::<f64>(),
                pearson(&v, &truth),
                l1
            );
        }
        let runs: Vec<Vec<RunMetrics>> = arms
            .iter()
            .map(|arm| {
                sweep(&sc0, &eval, &zones, jobs(&args), |sc| {
                    run_arm(&engine, &hidden, day, sc, arm, accept)
                })
            })
            .collect();
        let a = &runs[0];
        println!(
            "\n  không gợi ý (tuyệt đối, TB {n} seed): khách đặt {:.1}, hoàn thành {:.1}, cancel {:.4}, wait_s {:.1}, pickup_eta_s {:.1}, deadhead_m {:.1}, km rỗng/khách {:.1} m",
            a.iter().map(|x| x.requests as f64).sum::<f64>() / n as f64,
            a.iter().map(|x| x.completed as f64).sum::<f64>() / n as f64,
            a.iter().map(|x| x.cancel_rate).sum::<f64>() / n as f64,
            a.iter().map(|x| x.wait_mean).sum::<f64>() / n as f64,
            a.iter().map(|x| x.eta_mean).sum::<f64>() / n as f64,
            a.iter().map(|x| x.deadhead_mean).sum::<f64>() / n as f64,
            a.iter().map(|x| x.empty_m_per_request).sum::<f64>() / n as f64
        );
        println!(
            "\n  hiệu = có gợi ý − không gợi ý (có cặp, KTC 95% t Student; * = KTC không chứa 0):"
        );
        println!(
            "  {:<28} {:>10} {:>22} {:>22} {:>24} {:>24} {:>20}",
            "cấu hình",
            "gợi ý/seed",
            "chạy theo gợi ý/khách (m)",
            "pickup_eta_s TB",
            "wait_s TB",
            "km rỗng TỔNG/khách (m)",
            "hoàn thành"
        );
        for (arm, r) in arms.iter().zip(&runs).skip(1) {
            let sug = r
                .iter()
                .map(|x| x.extras.reposition_suggested as f64)
                .sum::<f64>()
                / n as f64;
            let cruise = r
                .iter()
                .map(|x| x.extras.cruise_m / x.requests.max(1) as f64)
                .sum::<f64>()
                / n as f64;
            println!(
                "  {:<28} {:>10.1} {:>22.1} {:>22} {:>24} {:>24} {:>20}",
                arm.name,
                sug,
                cruise,
                cell(a, r, |x| x.eta_mean, 1),
                cell(a, r, |x| x.wait_mean, 1),
                cell(a, r, |x| x.empty_m_per_request, 1),
                cell(a, r, |x| x.completed as f64, 2)
            );
        }
        println!(
            "\n  so từng cặp với bảng CỐ ĐỊNH (hiệu = cấu hình − bảng cố định, cùng seed): xem dự báo có đáng so với việc không có dự báo theo giờ"
        );
        println!(
            "  {:<28} {:>22} {:>24} {:>24} {:>20}",
            "cấu hình", "pickup_eta_s TB", "wait_s TB", "km rỗng TỔNG/khách (m)", "hoàn thành"
        );
        for (arm, r) in arms.iter().zip(&runs).skip(2) {
            println!(
                "  {:<28} {:>22} {:>24} {:>24} {:>20}",
                arm.name,
                cell(&runs[1], r, |x| x.eta_mean, 1),
                cell(&runs[1], r, |x| x.wait_mean, 1),
                cell(&runs[1], r, |x| x.empty_m_per_request, 1),
                cell(&runs[1], r, |x| x.completed as f64, 2)
            );
        }
        if args.flag("--full") {
            print_paired(
                &format!("\n  đủ chỉ số: bảng cố định so với dự báo LightGBM ({name})"),
                "bảng cố định",
                "dự báo LightGBM",
                &runs[1],
                &runs[3],
            );
            print_zones("bảng cố định", "dự báo LightGBM", &runs[1], &runs[3]);
        }
        print_zones("không gợi ý", "dự báo LightGBM", a, &runs[3]);
    }
    println!(
        "\nGiả định: trên dữ liệu mô phỏng. Tài xế nghe theo gợi ý với xác suất cố định, độc lập; luật gợi ý và ngưỡng của LOC-49 giữ nguyên, không chỉnh theo bảng dự báo; bảng «cố định» được cho cùng tổng với dự báo (có lợi cho nó)."
    );
    println!("uptime lúc kết thúc: {}", uptime());
}
