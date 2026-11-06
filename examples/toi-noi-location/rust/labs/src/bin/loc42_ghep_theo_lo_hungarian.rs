//! LOC-42: ghép theo lô bằng Hungarian (Kuhn–Munkres, crate `pathfinding`) so với tham lam.
//! (1) mô hình hoá ma trận chi phí ETA và so ba cách giải trên lô ngẫu nhiên; (2) thừa khách / thừa xe; (3) độ phức tạp đo được;
//! (4) trong simulator với cửa sổ lô 10 s: Hungarian so với tham lam trên cùng lô, so có cặp.
//!
//! Chạy từ gốc repo:
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc42_ghep_theo_lo_hungarian
//! Tham số: --seeds N (20), --jobs N (3), --batches N (200), --fixture

use labs::p03::uptime;
use labs::p05::{f1, jobs, n_seeds, pickup_of, print_paired, setup};
use labs::{Args, Rand, percentile, timed};
use matcher::assign::{brute_force, greedy_fifo, greedy_global, hungarian, total_cost};
use matcher::batch::{Algo, BatchMatcher, MatcherConfig, MatcherStats, Window};
use matcher::cost::{Dense, FORBIDDEN, eta_matrix_ms};
use matcher::eval::{Zones, eval_seeds, paired, sweep};
use route_engine::position::Position;
use simulator::world::generate;
use simulator::{AtPin, RunOptions, run};
use std::sync::Mutex;

fn main() {
    let args = Args::from_env();
    println!(
        "== LOC-42: ghép theo lô, Hungarian (Kuhn–Munkres) so với tham lam (trên dữ liệu mô phỏng) =="
    );
    println!("uptime lúc bắt đầu: {}", uptime());
    let (engine, info, sc, fixture) = setup(&args);
    println!(
        "đồ thị: {info}; kịch bản gốc: {} (seed gốc {})",
        sc.name, sc.seed
    );
    let zones = Zones::hcm();
    let batches = args.u64_or("--batches", if fixture { 60 } else { 200 }) as usize;

    // ---------- 1. ma trận ETA và ba cách giải ----------
    println!(
        "\n-- 1. lô ngẫu nhiên: {batches} lô mỗi cỡ; khách và tài xế lấy từ thế giới mô phỏng, ô = ETA đường bộ (CH, ms) --"
    );
    println!(
        "  chi phí lô = tổng ETA các cặp được ghép (s). Tham lam theo thứ tự = khách xếp theo thứ tự đặt (ngẫu nhiên ở đây); toàn cục = lấy cặp rẻ nhất lặp lại."
    );
    let mut wide = sc.clone();
    wide.drivers = 300;
    let w = generate(&engine, &wide);
    let drivers: Vec<Position> = w.drivers.iter().map(|d| d.pos).collect();
    let picks: Vec<Vec<Position>> = w
        .customers
        .iter()
        .filter_map(|c| pickup_of(&engine, c.view.pin).map(|(_, ap)| ap))
        .collect();
    let mut rng = Rand::new(42_001);
    let shapes: &[(usize, usize)] = &[
        (5, 5),
        (10, 10),
        (20, 20),
        (30, 50),
        (50, 30),
        (10, 60),
        (60, 10),
    ];
    println!(
        "  {:>5} {:>5} {:>7}  {:>14} {:>14} {:>14}  {:>22} {:>22}  {:>10} {:>10}",
        "khách",
        "xe",
        "ghép",
        "tham lam TT",
        "tham lam TC",
        "Hungarian",
        "Hungarian − TT (s) [KTC]",
        "Hungarian − TC (s) [KTC]",
        "ETA max TT",
        "ETA max H"
    );
    for &(m, n) in shapes {
        if fixture && m.max(n) > 30 {
            continue;
        }
        let (mut tt, mut tc, mut hh) = (Vec::new(), Vec::new(), Vec::new());
        let (mut mx_t, mut mx_h) = (Vec::new(), Vec::new());
        let mut matched = 0usize;
        for _ in 0..batches {
            let pick_idx: Vec<usize> = (0..m).map(|_| rng.usize(picks.len())).collect();
            let drv_idx: Vec<usize> = (0..n).map(|_| rng.usize(drivers.len())).collect();
            let from: Vec<Position> = drv_idx.iter().map(|&i| drivers[i]).collect();
            let to: Vec<&[Position]> = pick_idx.iter().map(|&i| picks[i].as_slice()).collect();
            let c = eta_matrix_ms(&engine, &from, &to).transpose(); // hàng = khách
            if c.data.iter().any(|&v| v >= FORBIDDEN / 2) {
                continue; // có cặp không tới được (hiếm): bỏ lô để so cùng số cặp
            }
            let (g, gg, h) = (greedy_fifo(&c), greedy_global(&c), hungarian(&c));
            matched = h.len();
            assert_eq!(g.len(), h.len());
            tt.push(total_cost(&c, &g) as f64 / 1000.0);
            tc.push(total_cost(&c, &gg) as f64 / 1000.0);
            hh.push(total_cost(&c, &h) as f64 / 1000.0);
            let mx = |p: &[(usize, usize)]| {
                p.iter()
                    .map(|&(r, k)| c.get(r, k) as f64 / 1000.0)
                    .fold(0.0, f64::max)
            };
            mx_t.push(mx(&g));
            mx_h.push(mx(&h));
        }
        let (p_tt, p_tc) = (paired(&tt, &hh), paired(&tc, &hh));
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        println!(
            "  {:>5} {:>5} {:>7}  {:>14.1} {:>14.1} {:>14.1}  {:>22} {:>22}  {:>10.1} {:>10.1}",
            m,
            n,
            matched,
            p_tt.mean_a,
            p_tc.mean_a,
            p_tt.mean_b,
            format!(
                "{:+.1} [{:.1}, {:.1}] ({:+.1}%)",
                p_tt.mean_diff,
                p_tt.ci_lo,
                p_tt.ci_hi,
                100.0 * p_tt.mean_diff / p_tt.mean_a
            ),
            format!(
                "{:+.1} [{:.1}, {:.1}] ({:+.1}%)",
                p_tc.mean_diff,
                p_tc.ci_lo,
                p_tc.ci_hi,
                100.0 * p_tc.mean_diff / p_tc.mean_a
            ),
            mean(&mx_t),
            mean(&mx_h)
        );
        assert!(
            hh.iter().zip(&tt).all(|(h, g)| h <= &(g + 1e-9)),
            "Hungarian không bao giờ tệ hơn tham lam"
        );
    }
    println!(
        "  Ghi chú: cột «ghép» = số cặp được gán = min(khách, xe) (ma trận chữ nhật: bên thừa chờ lô sau). «ETA max» = ETA lớn nhất trong một lô, trung bình các lô: cái giá của việc tối ưu tổng là một cặp đơn lẻ có thể xa hơn."
    );

    // đối chiếu với vét cạn
    let mut mism = 0;
    let trials = if fixture { 100 } else { 300 };
    for _ in 0..trials {
        let (m, n) = (1 + rng.usize(6), 1 + rng.usize(6));
        let c = Dense::from_fn(m, n, |_, _| {
            if rng.usize(10) == 0 {
                FORBIDDEN
            } else {
                1 + rng.usize(5000) as i64
            }
        });
        if total_cost(&c, &hungarian(&c)) != total_cost(&c, &brute_force(&c))
            || hungarian(&c).len() != brute_force(&c).len()
        {
            mism += 1;
        }
    }
    println!(
        "  đối chiếu vét cạn (ma trận ngẫu nhiên 1–6 × 1–6, 10% ô bị cấm, {trials} ma trận): số lần Hungarian khác vét cạn = {mism}"
    );
    assert_eq!(mism, 0);

    // ---------- 2. độ phức tạp ----------
    println!(
        "\n-- 2. độ phức tạp đo được: Hungarian trên ma trận vuông n × n ngẫu nhiên (số nguyên), trung vị 5 lần đo --"
    );
    println!("  uptime: {}", uptime());
    println!(
        "  {:>6} {:>14} {:>14}  {:>10}",
        "n", "Hungarian (ms)", "tham lam (ms)", "H(n)/H(n/2)"
    );
    let mut prev: Option<f64> = None;
    let sizes: &[usize] = if fixture {
        &[25, 50, 100]
    } else {
        &[25, 50, 100, 200, 400, 800]
    };
    let mut pts = Vec::new();
    for &n in sizes {
        let mut r = Rand::new(42_002 + n as u64);
        let c = Dense::from_fn(n, n, |_, _| 30_000 + r.usize(900_000) as i64);
        let mut th: Vec<f64> = (0..5).map(|_| timed(|| hungarian(&c)).1 * 1e3).collect();
        let mut tg: Vec<f64> = (0..5).map(|_| timed(|| greedy_fifo(&c)).1 * 1e3).collect();
        th.sort_by(f64::total_cmp);
        tg.sort_by(f64::total_cmp);
        let (h, g) = (th[2], tg[2]);
        println!(
            "  {:>6} {:>14.3} {:>14.3}  {:>10}",
            n,
            h,
            g,
            prev.map_or("-".into(), |p| format!("{:.1}", h / p))
        );
        pts.push(((n as f64).ln(), h.max(1e-6).ln()));
        prev = Some(h);
    }
    let (mx, my) = (
        pts.iter().map(|p| p.0).sum::<f64>() / pts.len() as f64,
        pts.iter().map(|p| p.1).sum::<f64>() / pts.len() as f64,
    );
    let slope = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum::<f64>()
        / pts.iter().map(|p| (p.0 - mx).powi(2)).sum::<f64>();
    println!(
        "  độ dốc log–log của thời gian theo n (ma trận vuông): {slope:.2} (lý thuyết O(n³) → 3; ma trận ngẫu nhiên thường thấp hơn trường hợp xấu nhất)"
    );
    println!("  uptime: {}", uptime());

    // ---------- 3. trong simulator ----------
    let n = n_seeds(&args);
    let seeds = eval_seeds(n);
    println!(
        "\n-- 3. trong simulator: cửa sổ lô 10 s, ghép Hungarian so với tham lam theo thứ tự trên CÙNG lô; {n} seed đánh giá {}…{} --",
        seeds[0],
        seeds[n - 1]
    );
    for (vname, vdesc, tweak) in labs::p05::variants()
        .into_iter()
        .take(if fixture { 1 } else { 3 })
    {
        let mut s = sc.clone();
        tweak(&mut s);
        println!("\n=== bộ giả định «{vname}»: {vdesc} ===");
        let stats_a: Mutex<Vec<MatcherStats>> = Mutex::new(Vec::new());
        let stats_b: Mutex<Vec<MatcherStats>> = Mutex::new(Vec::new());
        let run_algo = |algo: Algo, sink: &Mutex<Vec<MatcherStats>>, sc: &simulator::Scenario| {
            let (m, h) = BatchMatcher::new(MatcherConfig::new(
                if algo == Algo::Hungarian {
                    "hungarian-w10"
                } else {
                    "greedy-w10"
                },
                algo,
                Window::Fixed(10_000),
            ));
            let out = run(
                &engine,
                sc,
                Box::new(AtPin),
                Box::new(m),
                RunOptions { trace: true },
            );
            sink.lock()
                .expect("khoá")
                .push(h.lock().expect("khoá").clone());
            out
        };
        let a = sweep(&s, &seeds, &zones, jobs(&args), |sc| {
            run_algo(Algo::FifoGreedy, &stats_a, sc)
        });
        let b = sweep(&s, &seeds, &zones, jobs(&args), |sc| {
            run_algo(Algo::Hungarian, &stats_b, sc)
        });
        let summarize = |v: &Mutex<Vec<MatcherStats>>| {
            let v = v.lock().expect("khoá");
            let calls: usize = v.iter().map(|s| s.calls).sum();
            let rows: usize = v.iter().map(|s| s.rows_sum).sum();
            let cols: usize = v.iter().map(|s| s.cols_sum).sum();
            let max_rows = v.iter().map(|s| s.max_rows).max().unwrap_or(0);
            let mut solve: Vec<f64> = v.iter().flat_map(|s| s.solve_us.iter().copied()).collect();
            solve.sort_by(f64::total_cmp);
            (
                calls,
                rows as f64 / calls.max(1) as f64,
                cols as f64 / calls.max(1) as f64,
                max_rows,
                percentile(&solve, 50.0),
                percentile(&solve, 99.0),
            )
        };
        let (calls, r, c, mr, p50, p99) = summarize(&stats_b);
        println!(
            "  kích thước lô thực tế (Hungarian): {calls} lần ghép trên {n} seed; trung bình {r:.2} khách × {c:.2} xe mỗi lô, lô lớn nhất {mr} khách; thời gian giải p50 {p50:.1} µs, p99 {p99:.1} µs (đo khi máy chịu tải)"
        );
        println!(
            "  khách đặt TB/seed {}",
            f1(a.iter().map(|r| r.requests as f64).sum::<f64>() / n as f64)
        );
        print_paired(
            "  so sánh có cặp:",
            "tham lam theo thứ tự, cửa sổ 10 s",
            "Hungarian, cửa sổ 10 s",
            &a,
            &b,
        );
        let same = a
            .iter()
            .zip(&b)
            .filter(|(x, y)| (x.deadhead_mean - y.deadhead_mean).abs() < 1e-9)
            .count();
        println!("  số seed mà hai cách cho đúng cùng deadhead_m trung bình: {same}/{n}");
    }
    println!(
        "\nGiả định: trên dữ liệu mô phỏng; đọc kích thước lô thực tế ở từng bộ giả định để biết còn gì để tối ưu trong một lô."
    );
    println!("uptime lúc kết thúc: {}", uptime());
}
