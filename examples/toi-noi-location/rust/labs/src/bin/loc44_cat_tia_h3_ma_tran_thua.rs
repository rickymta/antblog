//! LOC-44: cắt tỉa ứng viên bằng H3 k-ring và k gần nhất, ma trận thưa, tách vùng độc lập.
//! (A) sai số do cắt tỉa so với tối ưu đầy đủ trên lô cỡ thành phố; (B) trong simulator, cắt tỉa so với không cắt, có cặp;
//! (C) ngân sách độ trễ (đo khi máy chịu tải nặng: chỉ so sánh tương đối).
//!
//! Chạy từ gốc repo:
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc44_cat_tia_h3_ma_tran_thua
//! Tham số: --seeds N (20), --jobs N (3), --instances N (số lô mỗi cỡ cho phần A, mặc định 10), --fixture,
//! --chi-thanh-pho (chỉ chạy cỡ thành phố trên một lô riêng, có thêm Hungarian trên ma trận đầy đủ làm mốc tối ưu)

use labs::p03::uptime;
use labs::p05::{jobs, n_seeds, pickup_of, print_paired, select_variants, setup};
use labs::{Args, Rand, percentile, timed};
use matcher::assign::{greedy_global, hungarian, total_cost};
use matcher::batch::{Algo, BatchMatcher, MatcherConfig, MatcherStats, Window};
use matcher::cost::{FORBIDDEN, eta_matrix_ms};
use matcher::eval::{Zones, eval_seeds, paired, sweep};
use matcher::prune::{PruneConfig, PrunedRun, pruned_hungarian};
use route_engine::position::Position;
use simulator::world::generate;
use simulator::{AtPin, RunOptions, run};
use std::sync::Mutex;

const CONFIGS: [(&str, PruneConfig); 5] = [
    (
        "H3 r8, k-ring 1, 4 gần nhất",
        PruneConfig {
            res: 8,
            k_ring: 1,
            k_nearest: 4,
            k_ring_max: 8,
            leftover: false,
        },
    ),
    (
        "H3 r8, k-ring 2, 8 gần nhất",
        PruneConfig {
            res: 8,
            k_ring: 2,
            k_nearest: 8,
            k_ring_max: 8,
            leftover: false,
        },
    ),
    (
        "H3 r8, k-ring 2, 8 + lượt bù",
        PruneConfig {
            res: 8,
            k_ring: 2,
            k_nearest: 8,
            k_ring_max: 8,
            leftover: true,
        },
    ),
    (
        "H3 r8, k-ring 3, 16 gần nhất",
        PruneConfig {
            res: 8,
            k_ring: 3,
            k_nearest: 16,
            k_ring_max: 8,
            leftover: false,
        },
    ),
    (
        "H3 r7, k-ring 1, 8 gần nhất",
        PruneConfig {
            res: 7,
            k_ring: 1,
            k_nearest: 8,
            k_ring_max: 8,
            leftover: false,
        },
    ),
];

struct Inst {
    drivers: Vec<Position>,
    drivers_ll: Vec<geo_core::LatLng>,
    picks: Vec<Vec<Position>>,
    picks_ll: Vec<geo_core::LatLng>,
}

fn main() {
    let args = Args::from_env();
    println!(
        "== LOC-44: cắt tỉa ứng viên, ma trận thưa ở quy mô thành phố (trên dữ liệu mô phỏng) =="
    );
    println!("uptime lúc bắt đầu: {}", uptime());
    let (engine, info, sc, fixture) = setup(&args);
    let zones = Zones::hcm();
    println!("đồ thị: {info}; kịch bản gốc: {}", sc.name);
    let n_inst = args.u64_or("--instances", if fixture { 3 } else { 10 }) as usize;

    // thế giới lớn: 3000 tài xế, ~6000 khách (chỉ dùng để lấy vị trí)
    let mut wide = sc.clone();
    wide.drivers = if fixture { 300 } else { 3000 };
    wide.demand.requests_per_min = if fixture { 60.0 } else { 200.0 };
    wide.duration_s = 1800.0;
    let w = generate(&engine, &wide);
    println!(
        "kho vị trí để rút mẫu: {} tài xế, {} khách (cùng bộ sinh của simulator, seed {})",
        w.drivers.len(),
        w.customers.len(),
        wide.seed
    );
    let draw = |c: usize, d: usize, rng: &mut Rand| -> Inst {
        let (mut drivers, mut dl) = (Vec::new(), Vec::new());
        for _ in 0..d {
            let p = w.drivers[rng.usize(w.drivers.len())].pos;
            drivers.push(p);
            dl.push(p.point(&engine.g));
        }
        let (mut picks, mut pl) = (Vec::new(), Vec::new());
        while picks.len() < c {
            let cu = &w.customers[rng.usize(w.customers.len())];
            if let Some((stop, ap)) = pickup_of(&engine, cu.view.pin) {
                picks.push(ap);
                pl.push(stop);
            }
        }
        Inst {
            drivers,
            drivers_ll: dl,
            picks,
            picks_ll: pl,
        }
    };

    // --only-b: chỉ chạy phần B (để thử thêm bộ giả định bằng --variant mà không lặp lại A và C)
    let only_b = args.flag("--only-b");
    let only_city = args.flag("--chi-thanh-pho");
    let mut rng = Rand::new(44_001);
    if !only_b {
        // ---------- A ----------
        println!(
            "\n-- A. sai số do cắt tỉa so với tối ưu đầy đủ ({n_inst} lô mỗi cỡ, cùng lô cho mọi cấu hình; xe nhiều bằng hoặc gấp 4 lần khách) --"
        );
        println!(
            "  «đầy đủ» = ma trận ETA n × n bằng bucket CH + Hungarian toàn ma trận (tối ưu). Sai số = tổng ETA cắt tỉa / tổng ETA tối ưu đầy đủ − 1, chỉ tính trên các lô mà cắt tỉa ghép đủ số cặp như bản đầy đủ (cột «lô đủ cặp»); «ghép TB» cho biết cắt tỉa mất bao nhiêu khách ở các lô còn lại."
        );
        // (số khách, số xe): bằng nhau (khó nhất: mọi xe phải dùng) và xe gấp 4 lần khách (gần cảnh thực tế: nhiều xe rảnh hơn khách chờ)
        let sizes: &[(usize, usize)] = if fixture {
            &[(40, 40), (40, 160)]
        } else {
            &[
                (200, 200),
                (500, 500),
                (800, 800),
                (100, 400),
                (250, 1000),
                (400, 1600),
            ]
        };
        for &(cn, dn) in sizes.iter().filter(|_| !only_city) {
            let insts: Vec<Inst> = (0..n_inst).map(|_| draw(cn, dn, &mut rng)).collect();
            let mut full_cost = Vec::new();
            let mut full_match = Vec::new();
            let mut full_greedy_ratio = Vec::new();
            let (mut t_eta_full, mut t_sol_full) = (Vec::new(), Vec::new());
            for inst in &insts {
                let to: Vec<&[Position]> = inst.picks.iter().map(Vec::as_slice).collect();
                let (m, te) = timed(|| eta_matrix_ms(&engine, &inst.drivers, &to).transpose());
                let (h, ts) = timed(|| hungarian(&m));
                let g = greedy_global(&m);
                full_cost.push(total_cost(&m, &h) as f64 / 1000.0);
                full_match.push(h.len());
                full_greedy_ratio
                    .push(total_cost(&m, &g) as f64 / total_cost(&m, &h).max(1) as f64);
                t_eta_full.push(te * 1e3);
                t_sol_full.push(ts * 1e3);
            }
            let med = |v: &mut Vec<f64>| {
                v.sort_by(f64::total_cmp);
                v[v.len() / 2]
            };
            println!(
                "\n  cỡ {cn} khách × {dn} xe: đầy đủ: ghép TB {:.1}/{cn}, tổng ETA TB {:.0} s, ETA ma trận trung vị {:.1} ms + Hungarian trung vị {:.1} ms; tham lam toàn cục / Hungarian = {:.3}",
                full_match.iter().sum::<usize>() as f64 / n_inst as f64,
                full_cost.iter().sum::<f64>() / n_inst as f64,
                med(&mut t_eta_full),
                med(&mut t_sol_full),
                full_greedy_ratio.iter().sum::<f64>() / n_inst as f64
            );
            println!(
                "  {:<30} {:>9} {:>8} {:>9} {:>11} {:>9} {:>9} {:>26} {:>9} {:>20} {:>10}",
                "cấu hình cắt tỉa",
                "cạnh/lô",
                "mật độ%",
                "vùng",
                "vùng lớn",
                "ghép TB",
                "lô đủ cặp",
                "sai số tổng ETA [KTC]",
                "sai số max",
                "thời gian cắt tỉa (ms)",
                "so đầy đủ"
            );
            for (name, pc) in CONFIGS {
                let (mut errs, mut matched, mut edges, mut comps, mut largest) =
                    (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
                let mut full_count = 0usize;
                let mut times = Vec::new();
                let mut lower_bound_violations = 0;
                for (k, inst) in insts.iter().enumerate() {
                    let to: Vec<&[Position]> = inst.picks.iter().map(Vec::as_slice).collect();
                    let r: PrunedRun = pruned_hungarian(
                        &engine,
                        &inst.drivers,
                        &inst.drivers_ll,
                        &to,
                        &inst.picks_ll,
                        pc,
                    );
                    // tổng ETA chỉ so được khi hai bên ghép cùng số cặp
                    if r.pairs.len() == full_match[k] {
                        full_count += 1;
                        errs.push(r.cost_ms as f64 / 1000.0 / full_cost[k] - 1.0);
                    }
                    if r.pairs.len() == full_match[k]
                        && (r.cost_ms as f64 / 1000.0) < full_cost[k] - 1e-6
                    {
                        lower_bound_violations += 1; // cắt tỉa không được rẻ hơn tối ưu đầy đủ khi cùng số cặp
                    }
                    matched.push(r.pairs.len() as f64);
                    edges.push(r.edges as f64);
                    comps.push(r.components as f64);
                    largest.push((r.largest.0 + r.largest.1) as f64);
                    times.push((r.candidates_us + r.eta_us + r.solve_us) / 1e3);
                }
                assert_eq!(
                    lower_bound_violations, 0,
                    "cắt tỉa rẻ hơn tối ưu đầy đủ: lỗi"
                );
                let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
                let pe = paired(&vec![0.0; errs.len()], &errs);
                let tmed = {
                    times.sort_by(f64::total_cmp);
                    times[times.len() / 2]
                };
                println!(
                    "  {:<30} {:>9.0} {:>8.2} {:>9.1} {:>11.1} {:>9.1} {:>9} {:>26} {:>9} {:>20.1} {:>9.1}×",
                    name,
                    mean(&edges),
                    100.0 * mean(&edges) / (cn * dn) as f64,
                    mean(&comps),
                    mean(&largest),
                    mean(&matched),
                    format!("{}/{}", full_count, n_inst),
                    if errs.len() >= 2 {
                        format!(
                            "{:+.2}% [{:+.2}, {:+.2}]",
                            100.0 * pe.mean_diff,
                            100.0 * pe.ci_lo,
                            100.0 * pe.ci_hi
                        )
                    } else {
                        "n/a".to_string()
                    },
                    if errs.is_empty() {
                        "n/a".to_string()
                    } else {
                        format!(
                            "{:+.2}%",
                            100.0 * errs.iter().copied().fold(f64::MIN, f64::max)
                        )
                    },
                    tmed,
                    (med(&mut t_eta_full.clone()) + med(&mut t_sol_full.clone())) / tmed
                );
            }
        }

        // ---------- city scale ----------
        let big = if fixture { 150 } else { 3000 };
        println!(
            "\n  cỡ thành phố {big} × {big} (mỗi cấu hình 1 lô; {}, chuẩn so sánh = tham lam toàn cục trên ma trận đầy đủ:",
            if only_city {
                "có chạy Hungarian trên ma trận đầy đủ, in ngay dưới)".to_string()
            } else {
                format!(
                    "Hungarian đầy đủ KHÔNG chạy ở cỡ này vì O(n³) ≈ {:.1e} phép)",
                    (big as f64).powi(3)
                )
            }
        );
        let inst = draw(big, big, &mut rng);
        let to: Vec<&[Position]> = inst.picks.iter().map(Vec::as_slice).collect();
        let (m, te) = timed(|| eta_matrix_ms(&engine, &inst.drivers, &to).transpose());
        let gref = greedy_global(&m);
        let gcost = total_cost(&m, &gref) as f64 / 1000.0;
        println!(
            "  ma trận ETA đầy đủ {big}×{big} = {} ô: {:.2} s; tham lam toàn cục trên đó: {} cặp, tổng {:.0} s ({:.1} s/cặp)",
            m.rows * m.cols,
            te,
            gref.len(),
            gcost,
            gcost / gref.len().max(1) as f64
        );
        if only_city {
            let (h, th) = timed(|| hungarian(&m));
            let hcost = total_cost(&m, &h) as f64 / 1000.0;
            println!(
                "  Hungarian trên ma trận đầy đủ của lô này: {} cặp, tổng {:.0} s ({:.1} s/cặp), giải {:.2} s (máy chịu tải); tham lam toàn cục / Hungarian = {:.3}",
                h.len(),
                hcost,
                hcost / h.len().max(1) as f64,
                th,
                gcost / hcost
            );
        }
        println!(
            "  {:<30} {:>9} {:>9} {:>9} {:>11} {:>8} {:>13} {:>13} {:>13} {:>10}",
            "cấu hình cắt tỉa",
            "cạnh",
            "vùng",
            "vùng lớn",
            "ghép",
            "s/cặp",
            "cắt tỉa (ms)",
            "ETA cặp (ms)",
            "giải (ms)",
            "tổng (s)"
        );
        for (name, pc) in CONFIGS {
            let r = pruned_hungarian(
                &engine,
                &inst.drivers,
                &inst.drivers_ll,
                &to,
                &inst.picks_ll,
                pc,
            );
            println!(
                "  {:<30} {:>9} {:>9} {:>9} {:>11} {:>8.1} {:>13.1} {:>13.1} {:>13.1} {:>10.2}",
                name,
                r.edges,
                r.components,
                format!("{}×{}", r.largest.0, r.largest.1),
                r.pairs.len(),
                r.cost_ms as f64 / 1000.0 / r.pairs.len().max(1) as f64,
                r.candidates_us / 1e3,
                r.eta_us / 1e3,
                r.solve_us / 1e3,
                (r.candidates_us + r.eta_us + r.solve_us) / 1e6
            );
        }
    }

    if only_city {
        println!("uptime lúc kết thúc: {}", uptime());
        return;
    }

    // ---------- B ----------
    let n = n_seeds(&args);
    let seeds = eval_seeds(n);
    println!(
        "\n-- B. trong simulator: Hungarian cửa sổ 10 s, không cắt tỉa (A) so với cắt tỉa «{}» (B); {n} seed đánh giá {}…{} --",
        CONFIGS[1].0,
        seeds[0],
        seeds[n - 1]
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
        println!("\n=== bộ giả định «{vname}»: {vdesc} ===");
        let (sa, sb): (Mutex<Vec<MatcherStats>>, Mutex<Vec<MatcherStats>>) =
            (Mutex::new(Vec::new()), Mutex::new(Vec::new()));
        let go = |prune: Option<PruneConfig>,
                  sink: &Mutex<Vec<MatcherStats>>,
                  sc: &simulator::Scenario| {
            let mut cfg = MatcherConfig::new(
                if prune.is_some() {
                    "hungarian-pruned"
                } else {
                    "hungarian-full"
                },
                Algo::Hungarian,
                Window::Fixed(10_000),
            );
            cfg.prune = prune;
            let (m, h) = BatchMatcher::new(cfg);
            let o = run(
                &engine,
                sc,
                Box::new(AtPin),
                Box::new(m),
                RunOptions { trace: true },
            );
            sink.lock()
                .expect("khoá")
                .push(h.lock().expect("khoá").clone());
            o
        };
        let a = sweep(&s, &seeds, &zones, jobs(&args), |sc| go(None, &sa, sc));
        let b = sweep(&s, &seeds, &zones, jobs(&args), |sc| {
            go(Some(CONFIGS[1].1), &sb, sc)
        });
        let stat = |v: &Mutex<Vec<MatcherStats>>| {
            let v = v.lock().expect("khoá");
            let mut calls: Vec<f64> = v.iter().flat_map(|s| s.call_us.iter().copied()).collect();
            calls.sort_by(f64::total_cmp);
            let cells: usize = v.iter().map(|s| s.eta_cells).sum();
            let n_calls: usize = v.iter().map(|s| s.calls).sum();
            (
                cells as f64 / n_calls.max(1) as f64,
                percentile(&calls, 50.0) / 1e3,
                percentile(&calls, 99.0) / 1e3,
                n_calls,
            )
        };
        let (ca, a50, a99, na) = stat(&sa);
        let (cb, b50, b99, nb) = stat(&sb);
        println!(
            "  ô ETA phải tính mỗi lần ghép: không cắt {ca:.1} (n = {na} lần), cắt tỉa {cb:.1} (n = {nb} lần); thời gian mỗi lần ghép p50/p99: {a50:.2}/{a99:.2} ms so với {b50:.2}/{b99:.2} ms (máy chịu tải)"
        );
        print_paired(
            "  so sánh có cặp:",
            "Hungarian cửa sổ 10 s, không cắt tỉa",
            "Hungarian cửa sổ 10 s, cắt tỉa",
            &a,
            &b,
        );
        let sc_ = Mutex::new(Vec::new());
        let c = sweep(&s, &seeds, &zones, jobs(&args), |sc| {
            go(Some(CONFIGS[2].1), &sc_, sc)
        });
        print_paired(
            "  thêm lượt bù (khách và xe còn thừa được ghép lại trên ma trận đầy đủ của riêng họ):",
            "Hungarian cửa sổ 10 s, không cắt tỉa",
            "Hungarian cửa sổ 10 s, cắt tỉa + lượt bù",
            &a,
            &c,
        );
    }

    if !only_b {
        // ---------- C ----------
        println!(
            "\n-- C. ngân sách độ trễ của một lần ghép (HCM, đo khi máy chịu tải nặng: chỉ đọc tỉ lệ, không đọc số tuyệt đối) --"
        );
        println!("  uptime: {}", uptime());
        println!(
            "  ngân sách GIẢ ĐỊNH của lab: 100 ms (p99) cho một lô; số thật cần đặt theo SLO của Dispatch."
        );
        println!(
            "  {:>7} {:<30} {:>12} {:>12} {:>12} {:>12} {:>14}",
            "cỡ", "đường ống", "p50 (ms)", "p95 (ms)", "p99 (ms)", "max (ms)", "trong 100 ms?"
        );
        let reps = if fixture { 10 } else { 30 };
        for &c in if fixture {
            &[30usize, 60][..]
        } else {
            &[50usize, 200, 1000][..]
        } {
            let insts: Vec<Inst> = (0..reps).map(|_| draw(c, c, &mut rng)).collect();
            let stat = |name: &str, f: &dyn Fn(&Inst) -> f64| {
                let mut v: Vec<f64> = insts.iter().map(f).collect();
                v.sort_by(f64::total_cmp);
                println!(
                    "  {:>7} {:<30} {:>12.2} {:>12.2} {:>12.2} {:>12.2} {:>14}",
                    format!("{c}×{c}"),
                    name,
                    percentile(&v, 50.0),
                    percentile(&v, 95.0),
                    percentile(&v, 99.0),
                    v.last().copied().unwrap_or(0.0),
                    if percentile(&v, 99.0) <= 100.0 {
                        "có"
                    } else {
                        "không"
                    }
                );
            };
            if c <= 200 {
                stat("đầy đủ: ma trận + Hungarian", &|i| {
                    let to: Vec<&[Position]> = i.picks.iter().map(Vec::as_slice).collect();
                    timed(|| {
                        let m = eta_matrix_ms(&engine, &i.drivers, &to).transpose();
                        hungarian(&m)
                    })
                    .1 * 1e3
                });
            }
            stat("cắt tỉa r8, k-ring 2, 8 gần nhất", &|i| {
                let to: Vec<&[Position]> = i.picks.iter().map(Vec::as_slice).collect();
                let r = pruned_hungarian(
                    &engine,
                    &i.drivers,
                    &i.drivers_ll,
                    &to,
                    &i.picks_ll,
                    CONFIGS[1].1,
                );
                (r.candidates_us + r.eta_us + r.solve_us) / 1e3
            });
        }
        let _ = FORBIDDEN;
    }
    println!(
        "\nGiả định: trên dữ liệu mô phỏng; vị trí tài xế và khách rút ngẫu nhiên từ bộ sinh của simulator (phân bố điểm nóng + nền đều), nên mật độ cục bộ phụ thuộc vào đó. Đồ thị thật có sông, kênh làm k-ring chim bay chọn sai ứng viên; sai số đo ở trên đã gồm hiệu ứng đó trên bản đồ HCM."
    );
    println!("uptime lúc kết thúc: {}", uptime());
}
