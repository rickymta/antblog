//! LOC-49: tài xế rảnh nên đi đâu. Gợi ý di chuyển theo bảng nhu cầu H3 cố định (LOC-55 sẽ thay bằng dự báo), chống bầy đàn,
//! chỉ gợi ý không ép (tài xế nghe theo với một xác suất). Chi phí chạy theo gợi ý được tính vào km rỗng.
//!
//! Chạy từ gốc repo:
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc49_tai_xe_ranh_di_dau
//! Tham số: --seeds N (20), --jobs N (3), --fixture

use geo_core::h3::{CellIndex, cell_of, resolution};
use labs::Args;
use labs::p03::uptime;
use labs::p05::{jobs, n_seeds, print_paired, print_zones, setup, variants};
use matcher::batch::{Algo, BatchMatcher, MatcherConfig, Window};
use matcher::eval::{RunMetrics, Zones, column, eval_seeds, paired, sweep, tuning_seeds};
use matcher::reposition::{DemandReposition, DemandTable, Mode, RepoConfig, RepoStats};
use route_engine::engine::Engine;
use rustc_hash::FxHashMap;
use simulator::world::generate;
use simulator::{AtPin, Extras, RunOptions, Scenario, run_with};
use std::sync::Mutex;

const RES: u8 = 8;

fn counts(engine: &Engine, sc: &Scenario, seed: u64) -> FxHashMap<CellIndex, f64> {
    let mut s = sc.clone();
    s.seed = seed;
    let w = generate(engine, &s);
    let res = resolution(RES);
    let mut m: FxHashMap<CellIndex, f64> = FxHashMap::default();
    for c in &w.customers {
        *m.entry(cell_of(c.view.pin, res)).or_default() += 1.0;
    }
    m
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
    name: String,
    table: Option<DemandTable>,
    mode: Mode,
    accept: f64,
}

fn run_arm(
    engine: &Engine,
    sc: &Scenario,
    arm: &Arm,
    sink: &Mutex<Vec<RepoStats>>,
) -> simulator::SimOutput {
    let (m, _) = BatchMatcher::new(MatcherConfig::new(
        "hungarian-immediate",
        Algo::Hungarian,
        Window::Immediate,
    ));
    let extras = match &arm.table {
        None => Extras::default(),
        Some(t) => {
            let (rep, h) = DemandReposition::new(
                &arm.name,
                t.clone(),
                RepoConfig {
                    mode: arm.mode,
                    ..RepoConfig::default()
                },
            );
            let out = run_with(
                engine,
                sc,
                Box::new(AtPin),
                Box::new(m),
                RunOptions { trace: true },
                Extras {
                    repositioner: Some(Box::new(rep)),
                    reposition_accept_prob: arm.accept,
                },
            );
            sink.lock()
                .expect("khoá")
                .push(h.lock().expect("khoá").clone());
            return out;
        }
    };
    run_with(
        engine,
        sc,
        Box::new(AtPin),
        Box::new(m),
        RunOptions { trace: true },
        extras,
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
    println!(
        "== LOC-49: tài xế rảnh nên đi đâu, điều hướng theo nhu cầu (trên dữ liệu mô phỏng) =="
    );
    println!("uptime lúc bắt đầu: {}", uptime());
    let (engine, info, sc, fixture) = setup(&args);
    let zones = Zones::hcm();
    let n = n_seeds(&args);
    let (eval, tune) = (eval_seeds(n), tuning_seeds(if fixture { 3 } else { 10 }));
    println!("đồ thị: {info}; kịch bản gốc: {}", sc.name);

    // ---- bảng nhu cầu cố định ----
    println!(
        "\n-- 1. bảng nhu cầu cố định (H3 độ phân giải {RES}), dựng từ {} seed CHỈNH {}…{}, không dùng seed đánh giá --",
        tune.len(),
        tune[0],
        tune[tune.len() - 1]
    );
    let mut acc: FxHashMap<CellIndex, f64> = FxHashMap::default();
    for &s in &tune {
        for (c, v) in counts(&engine, &sc, s) {
            *acc.entry(c).or_default() += v;
        }
    }
    let per_hour: FxHashMap<CellIndex, f64> = acc
        .iter()
        .map(|(&c, &v)| (c, v / tune.len() as f64 * 3600.0 / sc.duration_s))
        .collect();
    let table = DemandTable::from_per_hour(&engine, RES, &per_hour);
    let total = table.total_per_hour();
    let mut top: Vec<_> = table.cells.iter().collect();
    top.sort_by(|a, b| b.per_hour.total_cmp(&a.per_hour));
    let top10: f64 = top.iter().take(10).map(|c| c.per_hour).sum();
    println!(
        "  {} ô có nhu cầu (có đường ô tô gần tâm ô); tổng {:.1} yêu cầu/giờ; 10 ô đầu chiếm {:.1}%; cạnh ô H3 độ phân giải {RES} ≈ {:.0} m",
        table.cells.len(),
        total,
        100.0 * top10 / total,
        geo_core::h3::edge_length_avg_m(resolution(RES))
    );
    println!("  5 ô nhu cầu cao nhất (vĩ độ, kinh độ tâm ô, yêu cầu/giờ):");
    for c in top.iter().take(5) {
        println!(
            "    ({:.5}, {:.5})  {:.2}",
            c.center.lat(),
            c.center.lng(),
            c.per_hour
        );
    }
    // bảng có thông tin không: tương quan với số đếm thực trên seed đánh giá
    let mut cors = Vec::new();
    for &s in eval.iter().take(10) {
        let real = counts(&engine, &sc, s);
        let keys: Vec<CellIndex> = {
            let mut k: Vec<CellIndex> = per_hour.keys().chain(real.keys()).copied().collect();
            k.sort_by_key(|c| u64::from(*c));
            k.dedup();
            k
        };
        let a: Vec<f64> = keys
            .iter()
            .map(|k| per_hour.get(k).copied().unwrap_or(0.0))
            .collect();
        let b: Vec<f64> = keys
            .iter()
            .map(|k| real.get(k).copied().unwrap_or(0.0))
            .collect();
        cors.push(pearson(&a, &b));
    }
    println!(
        "  độ khớp của bảng với số khách thực theo ô trên 10 seed đánh giá đầu: Pearson trung bình {:.3} (từng seed {:.3}…{:.3}). Bảng là MỘT DỰ BÁO LÝ TƯỞNG vì cùng phân bố sinh khách với simulator; dự báo thật (LOC-55) sẽ kém hơn.",
        cors.iter().sum::<f64>() / cors.len() as f64,
        cors.iter().copied().fold(f64::MAX, f64::min),
        cors.iter().copied().fold(f64::MIN, f64::max)
    );
    let cfg = RepoConfig::default();
    println!(
        "  luật: gọi mỗi {} s; xét {} ô nhu cầu cao nhất; đi nếu «chờ ở chỗ đang đứng − chờ ở ô đích (sau khi thêm mình) − thời gian đi» ≥ {} s và ETA ≤ {} s; chờ ô = 3600 / (λ_ô / số xe rảnh trong ô)",
        cfg.interval_ms / 1000,
        cfg.top_cells,
        cfg.margin_s,
        cfg.max_move_s
    );

    // ---- thử nghiệm ----
    println!(
        "\n-- 2. trong simulator, {n} seed đánh giá {}…{}; hiệu = có gợi ý − không gợi ý (Hungarian ghép ngay), có cặp, KTC 95% --",
        eval[0],
        eval[n - 1]
    );
    let mk = |name: &str, t: Option<&DemandTable>, mode: Mode, accept: f64| Arm {
        name: name.into(),
        table: t.cloned(),
        mode,
        accept,
    };
    let noisy = table.noisy(1.0, 49_001);
    let uniform = table.uniform();
    let scenarios: Vec<(String, String, Scenario, Vec<Arm>)> = {
        let vs = variants();
        let get = |nm: &str| {
            let (a, b, t) = vs.iter().find(|v| v.0 == nm).expect("có");
            let mut s = sc.clone();
            t(&mut s);
            (a.to_string(), b.to_string(), s)
        };
        let full = vec![
            mk("không gợi ý", None, Mode::Marginal, 0.0),
            mk(
                "gợi ý theo nhu cầu, nghe 70%",
                Some(&table),
                Mode::Marginal,
                0.7,
            ),
            mk(
                "bầy đàn: mọi xe về ô nóng nhất, nghe 70%",
                Some(&table),
                Mode::HerdToTop,
                0.7,
            ),
            mk(
                "gợi ý theo nhu cầu, nghe 30%",
                Some(&table),
                Mode::Marginal,
                0.3,
            ),
            mk(
                "gợi ý theo nhu cầu, nghe 100%",
                Some(&table),
                Mode::Marginal,
                1.0,
            ),
            mk(
                "bảng nhiễu σ=1 (dự báo sai), nghe 70%",
                Some(&noisy),
                Mode::Marginal,
                0.7,
            ),
            mk(
                "bảng đều (không có dự báo), nghe 70%",
                Some(&uniform),
                Mode::Marginal,
                0.7,
            ),
        ];
        let short = |t: &DemandTable| {
            vec![
                mk("không gợi ý", None, Mode::Marginal, 0.0),
                mk("gợi ý theo nhu cầu, nghe 70%", Some(t), Mode::Marginal, 0.7),
                mk(
                    "bầy đàn: mọi xe về ô nóng nhất, nghe 70%",
                    Some(t),
                    Mode::HerdToTop,
                    0.7,
                ),
            ]
        };
        let mut out = Vec::new();
        let (a, b, s) = get("thường");
        out.push((a, b, s, full));
        let (a, b, s) = get("thưa xe");
        out.push((a, b, s, short(&table)));
        let (a, b, s) = get("dày xe");
        out.push((a, b, s, short(&table)));
        out
    };
    for (vname, vdesc, s, arms) in scenarios.into_iter().take(if fixture { 1 } else { 3 }) {
        println!("\n=== bộ giả định «{vname}»: {vdesc} ===");
        let mut runs: Vec<(String, Vec<RunMetrics>, Vec<RepoStats>)> = Vec::new();
        for arm in &arms {
            let sink: Mutex<Vec<RepoStats>> = Mutex::new(Vec::new());
            let r = sweep(&s, &eval, &zones, jobs(&args), |sc| {
                run_arm(&engine, sc, arm, &sink)
            });
            let st = sink.into_inner().expect("khoá");
            runs.push((arm.name.clone(), r, st));
        }
        let a = runs[0].1.clone();
        println!(
            "  không gợi ý (tuyệt đối, TB {n} seed): khách đặt {:.1}, hoàn thành {:.1}, cancel {:.4}, wait_s {:.1}, pickup_eta_s {:.1}, deadhead_m {:.1}, km rỗng/khách {:.1} m",
            a.iter().map(|x| x.requests as f64).sum::<f64>() / n as f64,
            a.iter().map(|x| x.completed as f64).sum::<f64>() / n as f64,
            a.iter().map(|x| x.cancel_rate).sum::<f64>() / n as f64,
            a.iter().map(|x| x.wait_mean).sum::<f64>() / n as f64,
            a.iter().map(|x| x.eta_mean).sum::<f64>() / n as f64,
            a.iter().map(|x| x.deadhead_mean).sum::<f64>() / n as f64,
            a.iter().map(|x| x.empty_m_per_request).sum::<f64>() / n as f64
        );
        println!(
            "  {:<44} {:>10} {:>10} {:>18} {:>20} {:>20} {:>22} {:>22} {:>16}",
            "cấu hình",
            "gợi ý/seed",
            "nghe/seed",
            "chạy theo gợi ý/khách (m)",
            "pickup_eta_s TB",
            "wait_s TB",
            "deadhead_m TB",
            "km rỗng TỔNG/khách (m)",
            "hoàn thành"
        );
        for (name, r, _) in &runs[1..] {
            let sug = r
                .iter()
                .map(|x| x.extras.reposition_suggested as f64)
                .sum::<f64>()
                / n as f64;
            let acc = r
                .iter()
                .map(|x| x.extras.reposition_accepted as f64)
                .sum::<f64>()
                / n as f64;
            let cruise = r
                .iter()
                .map(|x| x.extras.cruise_m / x.requests.max(1) as f64)
                .sum::<f64>()
                / n as f64;
            println!(
                "  {:<44} {:>10.1} {:>10.1} {:>18.1} {:>20} {:>20} {:>22} {:>22} {:>16}",
                name,
                sug,
                acc,
                cruise,
                cell(&a, r, |x| x.eta_mean, 1),
                cell(&a, r, |x| x.wait_mean, 1),
                cell(&a, r, |x| x.deadhead_mean, 1),
                cell(&a, r, |x| x.empty_m_per_request, 1),
                cell(&a, r, |x| x.completed as f64, 1)
            );
        }
        println!("\n  Bầy đàn: đo trên các lần gợi ý của từng cấu hình (trung bình các seed):");
        println!(
            "  {:<44} {:>14} {:>22} {:>18}",
            "cấu hình",
            "HHI đích đến",
            "phần gợi ý về ô nhiều nhất",
            "số xe về cùng ô/lần gọi (max)"
        );
        for (name, _, st) in &runs[1..] {
            if st.is_empty() {
                continue;
            }
            let k = st.len() as f64;
            println!(
                "  {:<44} {:>14.3} {:>21.1}% {:>18}",
                name,
                st.iter().map(RepoStats::hhi).sum::<f64>() / k,
                100.0 * st.iter().map(RepoStats::top_cell_share).sum::<f64>() / k,
                st.iter().map(RepoStats::max_burst).max().unwrap_or(0)
            );
        }
        let pick = |key: &str| runs.iter().find(|(nm, _, _)| nm.starts_with(key));
        if let Some((nm, r, _)) = pick("gợi ý theo nhu cầu, nghe 70%") {
            print_paired(
                &format!("\n  chi tiết đủ chỉ số: không gợi ý so với «{nm}»"),
                "không gợi ý",
                nm,
                &a,
                r,
            );
            print_zones("không gợi ý", nm, &a, r);
        }
        if vname == "thường"
            && let Some((nm, r, _)) = pick("bầy đàn")
        {
            print_zones("không gợi ý", nm, &a, r);
        }
    }
    println!(
        "\nGiả định: trên dữ liệu mô phỏng. Tài xế nghe theo gợi ý với xác suất cố định, độc lập; nhu cầu theo ô chỉ phụ thuộc vị trí (không theo giờ trong ngày); xe chạy theo gợi ý được tính vào km rỗng (cột «km rỗng TỔNG»). Đích gợi ý là điểm trên đường gần tâm ô, không xét chỗ đỗ hợp lệ."
    );
    println!("uptime lúc kết thúc: {}", uptime());
}
