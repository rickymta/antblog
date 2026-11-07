//! LOC-59: kiểm thử engine. CH khớp Dijkstra (trên đồ thị thật), tuyến vàng và báo cáo khác biệt giữa hai phiên bản bản đồ,
//! phát lại GPS tất định, cổng hồi quy cho chỉ số của simulator với ngưỡng suy từ nhiễu giữa các seed.
//!
//! Chạy từ gốc repo:
//!   cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc59_kiem_thu_engine
//! Tham số:
//!   --hcm        thêm các phần trên bản đồ HCM thật (CH so với Dijkstra 1.000 cặp, tuyến vàng và khác biệt bản đồ trên HCM)
//!   --gate       CHỈ chạy cổng hồi quy (như CI): thoát 1 nếu hồi quy; `--inject P` chèn lỗi «tài xế gần thứ hai» với xác suất P
//!   --bless      ghi lại tuyến vàng và file cổng (baseline + ngưỡng) trong fixtures/ bằng mã hiện tại. Chỉ dùng khi đã duyệt thay đổi.

use engine_qa::gate::{self, Verdict};
use engine_qa::golden::{self, Change, GoldenSet};
use engine_qa::replay::{WirePoint, from_text, replay_online, to_text};
use labs::p03::uptime;
use labs::p07::{Arm, FaultyMatch, GATE_METRICS, Match, Pick, Row, sweep_rows};
use labs::{Args, die, lab_root};
use map_match::Rng;
use map_match::online::Lag;
use map_match::route::Router;
use map_match::trace::{DriveConfig, NoiseConfig, corrupt, drive};
use map_shard::edit::close_ways;
use matcher::eval::Zones;
use pickup::rank::DriverModel;
use road_graph::Mode;
use road_graph::fixture::{compile_fixture, generate, load_fixture};
use road_graph::snap::SnapIndex;
use route_engine::engine::{Engine, EngineConfig};
use route_engine::net::{INF, Rev, View, sssp};
use route_engine::profile::{Costs, Metric, Profile};
use simulator::{RunOptions, Scenario, run};
use std::collections::HashMap;

/// 40 seed cổng: fixture chạy cỡ chục ms mỗi seed nên K lớn gần như miễn phí, và ngưỡng tỉ lệ 1/√K.
const GATE_SEEDS: std::ops::RangeInclusive<u64> = 20_272_001..=20_272_040;
const CALIB_SEEDS: std::ops::RangeInclusive<u64> = 20_272_101..=20_272_160;
const VALID_SEEDS: std::ops::RangeInclusive<u64> = 20_272_201..=20_272_520;
const Z: f64 = 3.0;

fn golden_path() -> std::path::PathBuf {
    lab_root().join("fixtures/golden/khu-mau-v1.json")
}
fn gate_path() -> std::path::PathBuf {
    lab_root().join("fixtures/gate/khu-mau-gate.json")
}

/// Cánh tay được bảo vệ: sản phẩm hiện tại = p04-v1 (BestTime) + ghép theo ETA (kết luận của LOC-64).
fn product_arm() -> Arm {
    Arm {
        name: "p04-v1 + nearest-eta",
        pick: Pick::P04v1(DriverModel::BestTime),
        matching: Match::NearestEta,
    }
}

fn run_rows(engine: &Engine, sc: &Scenario, seeds: &[u64], inject: Option<f64>) -> Vec<Row> {
    let arm = product_arm();
    let zones = Zones::hcm();
    sweep_rows(sc, seeds, &zones, 4, |s| match inject {
        None => arm.run(engine, s),
        Some(p) => run(
            engine,
            s,
            arm.pickup_strategy(),
            Box::new(FaultyMatch {
                inner: arm.match_strategy(),
                p,
            }),
            RunOptions { trace: true },
        ),
    })
}

fn col(rows: &[Row], f: fn(&Row) -> f64) -> Vec<f64> {
    rows.iter().map(f).collect()
}

fn fixture_setup() -> (Engine, Scenario) {
    let g = load_fixture(&labs::fixture_path()).unwrap_or_else(|e| die(&format!("{e}")));
    let sc = Scenario::load(&lab_root().join("fixtures/scenario-khu-mau.toml"))
        .unwrap_or_else(|e| die(&format!("{e}")));
    (Engine::new(g, EngineConfig::default()), sc)
}

fn build_gate(engine: &Engine, sc: &Scenario) -> gate::GateFile {
    let gs: Vec<u64> = GATE_SEEDS.collect();
    let cs: Vec<u64> = CALIB_SEEDS.collect();
    let calib = run_rows(engine, sc, &cs, None);
    let gate_rows = run_rows(engine, sc, &gs, None);
    let metrics: Vec<(&str, bool, Vec<f64>, Vec<f64>)> = GATE_METRICS
        .iter()
        .map(|&(n, lower, f)| (n, lower, col(&calib, f), col(&gate_rows, f)))
        .collect();
    gate::build(&sc.name, &gs, &cs, Z, &metrics)
}

fn candidate_means(rows: &[Row]) -> Vec<f64> {
    GATE_METRICS
        .iter()
        .map(|&(_, _, f)| gate::mean(&col(rows, f)))
        .collect()
}

fn print_checks(checks: &[gate::Check]) {
    println!(
        "  {:<26} {:>12} {:>12} {:>10} {:>10}  kết luận",
        "chỉ số", "baseline", "ứng viên", "chênh", "ngưỡng"
    );
    for c in checks {
        println!(
            "  {:<26} {:>12.4} {:>12.4} {:>+10.4} {:>10.4}  {}",
            c.name,
            c.baseline,
            c.candidate,
            c.delta,
            c.tol,
            match c.verdict {
                Verdict::Ok => "ổn",
                Verdict::Improved => "cải thiện",
                Verdict::Regressed => "HỒI QUY",
            }
        );
    }
}

fn gate_only(args: &Args) -> ! {
    let (engine, sc) = fixture_setup();
    let file = std::fs::read_to_string(gate_path()).unwrap_or_else(|e| {
        die(&format!(
            "thiếu {}: {e} (chạy --bless)",
            gate_path().display()
        ))
    });
    let g: gate::GateFile =
        serde_json::from_str(&file).unwrap_or_else(|e| die(&format!("file cổng hỏng: {e}")));
    let inject = args
        .value("--inject")
        .map(|v| v.parse::<f64>().unwrap_or_else(|_| die("--inject cần số")));
    let rows = run_rows(&engine, &sc, &g.gate_seeds, inject);
    println!(
        "== LOC-59 cổng hồi quy (CI): kịch bản {}, {} seed cổng {}…{}, z = {}{} ==",
        g.scenario,
        g.gate_seeds.len(),
        g.gate_seeds[0],
        g.gate_seeds[g.gate_seeds.len() - 1],
        g.z,
        inject.map_or(String::new(), |p| format!("; CHÈN LỖI p = {p}"))
    );
    let checks = gate::compare(&g, &candidate_means(&rows));
    print_checks(&checks);
    if gate::passed(&checks) {
        println!("CỔNG: ĐẠT");
        std::process::exit(0);
    }
    println!(
        "CỔNG: THẤT BẠI ({} chỉ số hồi quy)",
        checks
            .iter()
            .filter(|c| c.verdict == Verdict::Regressed)
            .count()
    );
    std::process::exit(1)
}

fn main() {
    let args = Args::from_env();
    if args.flag("--gate") {
        gate_only(&args);
    }
    println!("== LOC-59: kiểm thử engine (trên dữ liệu mô phỏng) ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let (engine, sc) = fixture_setup();
    println!(
        "đồ thị fixture: {} nút, {} cạnh; kịch bản {} ({} xe, {} s)",
        engine.g.node_count(),
        engine.g.edge_count(),
        sc.name,
        sc.drivers,
        sc.duration_s
    );

    if args.flag("--bless") {
        let pairs = golden::pick_pairs(&engine.g, 40, 59);
        let gs = golden::record(&engine, "v1", &pairs);
        std::fs::write(
            golden_path(),
            serde_json::to_string_pretty(&gs).expect("json"),
        )
        .expect("ghi golden");
        let g = build_gate(&engine, &sc);
        std::fs::write(gate_path(), serde_json::to_string_pretty(&g).expect("json"))
            .expect("ghi gate");
        println!(
            "đã ghi {} và {}",
            golden_path().display(),
            gate_path().display()
        );
        return;
    }

    // ---- 1. CH so với Dijkstra
    println!("\n-- 1. thuật toán nhanh so với thuật toán chậm --");
    println!(
        "  proptest (chạy bằng `cargo test -p engine-qa`, 40 ca mỗi thuộc tính, đồ thị fixture và đồ thị đã đóng ngẫu nhiên 0–5 đường): xem verify-p07.sh"
    );
    if args.flag("--hcm") {
        let gp = labs::graph_bin_path();
        let g = road_graph::io::load(&gp).unwrap_or_else(|e| die(&format!("{e}")));
        let mut rng = labs::Rand::new(5959);
        let (n_src, n_dst) = (40, 25);
        for p in [Profile::Car, Profile::Motorbike] {
            let rev = Rev::build(&g);
            let c = Costs::build(&g, p, Metric::Time);
            let view = View::new(&g, &rev, &c.w);
            let ch = route_engine::ch::Ch::build(view);
            let mut ws = route_engine::ch::ChWorkspace::new(g.node_count());
            let (mut ok, mut bad, mut unreach) = (0, 0, 0);
            for _ in 0..n_src {
                let s = rng.usize(g.node_count()) as u32;
                let truth = sssp(view, s, true);
                for _ in 0..n_dst {
                    let t = rng.usize(g.node_count()) as u32;
                    let want = (truth[t as usize] != INF).then_some(truth[t as usize]);
                    if want.is_none() {
                        unreach += 1;
                    }
                    if ch.distance(&mut ws, s, t) == want {
                        ok += 1;
                    } else {
                        bad += 1;
                    }
                }
            }
            println!(
                "  HCM, profile {}: CH dựng lại trong lab so với Dijkstra, {n_src} nguồn × {n_dst} đích = {} cặp: khớp {ok}, KHÁC {bad} (trong đó {unreach} cặp không có đường ở cả hai)",
                p.name(),
                n_src * n_dst
            );
        }
    }

    // ---- 2. tuyến vàng và khác biệt bản đồ
    println!("\n-- 2. tuyến vàng và báo cáo khác biệt phiên bản bản đồ --");
    let golden_file = std::fs::read_to_string(golden_path()).unwrap_or_else(|e| {
        die(&format!(
            "thiếu {}: {e} (chạy --bless)",
            golden_path().display()
        ))
    });
    let v1: GoldenSet =
        serde_json::from_str(&golden_file).unwrap_or_else(|e| die(&format!("golden hỏng: {e}")));
    let mism = golden::check(&engine, &v1);
    println!(
        "  fixture, mã hiện tại, bản đồ v1: {} tuyến vàng đã commit ({}), không khớp: {}",
        v1.routes.len(),
        golden_path()
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(""),
        mism.len()
    );
    for m in &mism {
        println!("    {m}");
    }
    let pairs: Vec<_> = v1.routes.iter().map(|r| (r.from_e7, r.to_e7)).collect();
    let mut count: HashMap<i64, usize> = HashMap::new();
    for r in v1.routes.iter().filter_map(|r| r.result.as_ref()) {
        let ways: std::collections::BTreeSet<i64> = r
            .edges
            .iter()
            .map(|&e| engine.g.way_id[e as usize])
            .collect();
        for w in ways {
            *count.entry(w).or_default() += 1;
        }
    }
    let (busiest, used) = count
        .into_iter()
        .max_by_key(|&(w, c)| (c, w))
        .expect("có tuyến");
    let e2 = Engine::new(close_ways(&engine.g, &[busiest]), EngineConfig::default());
    let v2 = golden::record(&e2, "v2", &pairs);
    println!(
        "  bản đồ v2 = v1 đóng way {busiest} (nằm trên {used}/{} tuyến vàng)",
        v1.routes.len()
    );
    let mism2 = golden::check(&e2, &v1);
    println!(
        "  tập vàng v1 kiểm trên bản đồ v2: {} tuyến không khớp (đúng: bản đồ đã đổi, nên phải xem báo cáo khác biệt, không phải «sửa golden cho qua»)",
        mism2.len()
    );
    print!("{}", golden::diff(&v1, &v2).render(5));
    if args.flag("--hcm") {
        let g =
            road_graph::io::load(&labs::graph_bin_path()).unwrap_or_else(|e| die(&format!("{e}")));
        let cfg = |tag: &str| EngineConfig {
            ch_cache_dir: Some(lab_root().join("rust/target/ch-cache")),
            cache_tag: tag.into(),
            ..EngineConfig::default()
        };
        let eh = Engine::new(g.clone(), cfg("hcm"));
        let pairs = golden::pick_pairs(&g, 60, 5959);
        let h1 = golden::record(&eh, "v1", &pairs);
        let mut count: HashMap<i64, usize> = HashMap::new();
        for r in h1.routes.iter().filter_map(|r| r.result.as_ref()) {
            let ways: std::collections::BTreeSet<i64> =
                r.edges.iter().map(|&e| g.way_id[e as usize]).collect();
            for w in ways {
                *count.entry(w).or_default() += 1;
            }
        }
        let (bw, bu) = count
            .into_iter()
            .max_by_key(|&(w, c)| (c, w))
            .expect("có tuyến");
        let t = std::time::Instant::now();
        let eh2 = Engine::new(close_ways(&g, &[bw]), cfg("p07-hcm-v2"));
        let h2 = golden::record(&eh2, "v2", &pairs);
        println!(
            "  HCM: 60 tuyến vàng v1; v2 đóng way {bw} (nằm trên {bu}/60 tuyến); dựng engine v2 và ghi vàng mất {:.1} s",
            t.elapsed().as_secs_f64()
        );
        print!("{}", golden::diff(&h1, &h2).render(5));
        let routed = h1.routes.iter().filter(|r| r.result.is_some()).count();
        println!("  ({routed}/60 cặp của v1 có đường; cặp không có đường ghi là BothUnreachable)");
        let _ = Change::Same;
    }

    // ---- 3. phát lại GPS
    println!("\n-- 3. phát lại vết GPS tất định --");
    let g = &engine.g;
    let idx = SnapIndex::build(g);
    let mk = |seed: u64| -> Vec<WirePoint> {
        let mut rng = Rng::new(seed);
        let mut router = Router::new(g);
        let route =
            map_match::scenario::random_trip(g, &mut router, Mode::Car, &mut rng, 600.0, 1400.0)
                .expect("chuyến");
        let truth = drive(g, &route, &DriveConfig::default(), &mut rng);
        corrupt(&route, truth, &NoiseConfig::default(), &mut rng)
            .obs
            .iter()
            .map(WirePoint::from_obs)
            .collect()
    };
    let pts = mk(59);
    println!(
        "  vết tổng hợp seed 59: {} điểm dạng dây (E7, dm, dm/s, ddeg); bám đường trực tuyến, trễ 5 điểm",
        pts.len()
    );
    let a = replay_online(g, &idx, &pts, Lag::Points(5), 1);
    println!(
        "  lần 1: {} quyết định, {} bám được, mã băm {:016x}",
        a.n_decisions, a.n_matched, a.hash
    );
    for (label, r) in [
        (
            "lần 2 (chạy lại)",
            replay_online(g, &idx, &pts, Lag::Points(5), 1),
        ),
        (
            "chia lô 7 điểm",
            replay_online(g, &idx, &pts, Lag::Points(5), 7),
        ),
        (
            "chia lô 1000 điểm",
            replay_online(g, &idx, &pts, Lag::Points(5), 1000),
        ),
        (
            "qua file văn bản",
            replay_online(g, &idx, &from_text(&to_text(&pts)), Lag::Points(5), 1),
        ),
        ("đồ thị dựng lại từ đầu", {
            let g2 = compile_fixture(&generate());
            let i2 = SnapIndex::build(&g2);
            replay_online(&g2, &i2, &pts, Lag::Points(5), 1)
        }),
    ] {
        println!(
            "  {label:<24}: mã băm {:016x} {}",
            r.hash,
            if r == a { "KHỚP" } else { "KHÁC" }
        );
    }
    let other = replay_online(g, &idx, &mk(60), Lag::Points(5), 1);
    println!(
        "  vết khác (seed 60)      : mã băm {:016x} {}",
        other.hash,
        if other.hash == a.hash {
            "TRÙNG (mã băm vô dụng)"
        } else {
            "khác, đúng"
        }
    );
    let mut moved = pts.clone();
    moved[pts.len() / 2].lat_e7 += 4_000;
    let m = replay_online(g, &idx, &moved, Lag::Points(5), 1);
    println!(
        "  đổi một điểm 4.000 E7 (~44 m): mã băm {:016x} {}",
        m.hash,
        if m.hash == a.hash {
            "TRÙNG"
        } else {
            "khác, đúng"
        }
    );

    // ---- 4. cổng hồi quy
    println!("\n-- 4. cổng hồi quy cho chỉ số của simulator --");
    let gs: Vec<u64> = GATE_SEEDS.collect();
    let cs: Vec<u64> = CALIB_SEEDS.collect();
    let valid: Vec<u64> = VALID_SEEDS.collect();
    println!(
        "  cánh tay được bảo vệ: {}; kịch bản fixture; ba dải seed tách nhau: cổng {}…{} ({}), hiệu chuẩn {}…{} ({}), kiểm chứng {}…{} ({}); z = {Z}",
        product_arm().name,
        gs[0],
        gs[gs.len() - 1],
        gs.len(),
        cs[0],
        cs[cs.len() - 1],
        cs.len(),
        valid[0],
        valid[valid.len() - 1],
        valid.len()
    );
    let g_file: gate::GateFile =
        serde_json::from_str(&std::fs::read_to_string(gate_path()).unwrap_or_else(|e| {
            die(&format!(
                "thiếu {}: {e} (chạy --bless)",
                gate_path().display()
            ))
        }))
        .unwrap_or_else(|e| die(&format!("{e}")));
    let calib = run_rows(&engine, &sc, &cs, None);
    let gate_rows = run_rows(&engine, &sc, &gs, None);
    let vrows = run_rows(&engine, &sc, &valid, None);
    let gate_k = |k: usize| -> gate::GateFile {
        let metrics: Vec<(&str, bool, Vec<f64>, Vec<f64>)> = GATE_METRICS
            .iter()
            .map(|&(n, lower, f)| (n, lower, col(&calib, f), col(&gate_rows[..k], f)))
            .collect();
        gate::build(&sc.name, &gs[..k], &cs, Z, &metrics)
    };
    let k_full = gate_k(gs.len());
    let same = k_full.metrics.iter().zip(&g_file.metrics).all(|(a, b)| {
        (a.baseline_mean - b.baseline_mean).abs() < 1e-9 && (a.tol - b.tol).abs() < 1e-9
    });
    println!(
        "  file cổng đã commit (fixtures/gate/khu-mau-gate.json) {} với cổng dựng lại từ mã hiện tại",
        if same { "KHỚP" } else { "KHÁC" }
    );
    println!(
        "  ngưỡng = z · σ · √(2/K); σ là độ lệch chuẩn giữa các seed hiệu chuẩn (không đặt tay). Bảng cho K = {}:",
        gs.len()
    );
    println!(
        "  {:<26} {:>12} {:>12} {:>12} {:>18}",
        "chỉ số", "baseline", "σ giữa seed", "ngưỡng", "ngưỡng / baseline"
    );
    for m in &k_full.metrics {
        println!(
            "  {:<26} {:>12.4} {:>12.4} {:>12.4} {:>17.1}%",
            m.name,
            m.baseline_mean,
            m.calib_sd,
            m.tol,
            100.0 * m.tol / m.baseline_mean.abs().max(1e-12)
        );
    }
    // 4a. cùng mã: đạt
    let checks = gate::compare(
        &g_file,
        &candidate_means(&run_rows(&engine, &sc, &g_file.gate_seeds, None)),
    );
    println!(
        "\n  4a. mã hiện tại so với baseline đã commit (cùng mã, cùng seed), K = {}:",
        gs.len()
    );
    print_checks(&checks);
    println!(
        "  → {}",
        if gate::passed(&checks) {
            "ĐẠT"
        } else {
            "THẤT BẠI"
        }
    );
    // 4b, 4c: theo K
    for k in [8usize, gs.len()] {
        let g = gate_k(k);
        let groups = valid.len() / k;
        let (mut fail, mut per_metric) = (0, vec![0usize; GATE_METRICS.len()]);
        for gi in 0..groups {
            let c = gate::compare(&g, &candidate_means(&vrows[gi * k..(gi + 1) * k]));
            fail += usize::from(!gate::passed(&c));
            for (i, x) in c.iter().enumerate() {
                per_metric[i] += usize::from(x.verdict == Verdict::Regressed);
            }
        }
        let hits: Vec<String> = GATE_METRICS
            .iter()
            .zip(&per_metric)
            .filter(|(_, n)| **n > 0)
            .map(|(m, n)| format!("{}={n}", m.0))
            .collect();
        println!(
            "\n  K = {k}: báo động nhầm: {groups} nhóm {k} seed KIỂM CHỨNG (mã giống hệt, seed khác) so với baseline: cổng thất bại ở {fail}/{groups} nhóm; chỉ số từng bị báo: {}",
            if hits.is_empty() {
                "không".to_string()
            } else {
                hits.join(", ")
            }
        );
        println!(
            "  K = {k}: chèn hồi quy «tài xế gần thứ hai» với xác suất p theo từng khách (cổng chạy trên {k} seed cổng đầu):"
        );
        for p in [0.1, 0.25, 0.5, 1.0] {
            let checks = gate::compare(
                &g,
                &candidate_means(&run_rows(&engine, &sc, &gs[..k], Some(p))),
            );
            let fired: Vec<&str> = checks
                .iter()
                .filter(|c| c.verdict == Verdict::Regressed)
                .map(|c| c.name.as_str())
                .collect();
            println!(
                "      p = {p:<4}: {}; chỉ số báo hồi quy: {}",
                if gate::passed(&checks) {
                    "cổng ĐẠT (KHÔNG bắt được)"
                } else {
                    "cổng THẤT BẠI (bắt được)"
                },
                if fired.is_empty() {
                    "không".to_string()
                } else {
                    fired.join(", ")
                }
            );
            if k == gs.len() && (p - 1.0).abs() < 1e-9 {
                print_checks(&checks);
            }
        }
    }
    println!(
        "\n  Giới hạn: cổng chạy trên fixture (8 xe, 1.200 s) nên nhạy với lỗi lớn, không nhạy với lỗi nhỏ; σ giữa seed của simulator HCM khác (LOC-30); ngưỡng phải hiệu chuẩn lại khi đổi kịch bản hoặc simulator."
    );
    println!("uptime lúc kết thúc: {}", uptime());
}
