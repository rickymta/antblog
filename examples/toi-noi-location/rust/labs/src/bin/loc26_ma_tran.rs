//! LOC-26: ma trận chi phí many-to-many dạng bucket trên CH, so với chạy lặp N×M, song song bằng rayon, và ngân sách độ trễ.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc26_ma_tran
//! Tham số: --seed S, --repeats R (mặc định 15 lần mỗi hình dạng), --fixture, --no-cache (luôn dựng lại CH)

use geo_core::LatLng;
use labs::p03::{mean, random_pairs, uptime, vn};
use labs::{Args, Rand, lab_root, load_graph, timed};
use route_engine::alt::largest_scc_nodes;
use route_engine::engine::{Engine, EngineConfig};
use route_engine::matrix::{Matrix, bucket_matrix, repeated_matrix};
use route_engine::net::INF;
use route_engine::position::Leg;
use route_engine::profile::Profile;
use route_engine::search::Searcher;
use std::time::Instant;

const SEED: u64 = 20_261_106;

fn legs(nodes: &[u32]) -> Vec<Vec<Leg>> {
    nodes
        .iter()
        .map(|&node| vec![Leg { node, cost: 0 }])
        .collect()
}

fn ms(f: impl FnOnce() -> Matrix) -> (Matrix, f64) {
    let t = Instant::now();
    let m = f();
    (m, t.elapsed().as_secs_f64() * 1e3)
}

fn pct(v: &mut [f64], p: f64) -> f64 {
    v.sort_by(f64::total_cmp);
    v[((p / 100.0) * v.len() as f64).ceil().max(1.0) as usize - 1]
}

fn main() {
    let args = Args::from_env();
    let seed = args.u64_or("--seed", SEED);
    let repeats = args.u64_or("--repeats", 15) as usize;
    let (g, source) = load_graph(&args);
    println!("== LOC-26: ma trận many-to-many bằng bucket trên CH ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    println!(
        "đồ thị: {source}: {} nút, {} cạnh; ô tô, thời gian (ms); seed {seed}; rayon dùng {} luồng",
        vn(g.node_count() as u64),
        vn(g.edge_count() as u64),
        rayon::current_num_threads()
    );
    let cache = (!args.flag("--no-cache") && !args.flag("--fixture"))
        .then(|| lab_root().join("rust/target/ch-cache"));
    let cfg = EngineConfig {
        ch_cache_dir: cache.clone(),
        cache_tag: "hcm".into(),
        ..EngineConfig::default()
    };
    let (eng, s) = timed(|| Engine::new(g, cfg));
    println!("dựng Engine (snap, hình học cạnh, trọng số): {s:.2} s");
    let had_cache = cache
        .as_ref()
        .is_some_and(|d| d.is_dir() && std::fs::read_dir(d).is_ok_and(|mut r| r.next().is_some()));
    let (_, s) = timed(|| eng.ch(Profile::Car));
    println!(
        "CH ô tô: {s:.2} s ({}; thời gian dựng gốc ghi trong CH: {:.2} s)\n",
        match (cache.is_some(), had_cache) {
            (false, _) => "dựng mới",
            (true, true) => "đã có file trong rust/target/ch-cache nên nạp từ đó",
            (true, false) => "chưa có file trong rust/target/ch-cache nên dựng mới và ghi lại",
        },
        eng.ch(Profile::Car).stats.preprocess_s
    );
    let ch = eng.ch(Profile::Car);
    let g = &eng.g;
    let view = eng.view(Profile::Car);
    let scc = largest_scc_nodes(g, &eng.rev, &eng.costs(Profile::Car).w);
    let nodes: Vec<u32> = (0..g.node_count() as u32)
        .filter(|&v| scc[v as usize])
        .collect();

    println!(
        "-- 1. so sánh các cách tính ma trận N×M (nguồn và đích là nút ngẫu nhiên; thời gian ms, một lần chạy mỗi cỡ) --"
    );
    println!(
        "  {:<10} {:>13} {:>13} {:>13} {:>14} {:>11} {:>9} {:>9}",
        "N×M",
        "bucket 1 luồng",
        "bucket rayon",
        "N×M truy vấn",
        "N×M truy vấn rayon",
        "µs/ô bucket",
        "lặp/bucket",
        "khác nhau"
    );
    let sizes = [
        (10usize, 10usize),
        (25, 25),
        (50, 50),
        (100, 100),
        (200, 200),
    ];
    let mut mismatches_total = 0usize;
    for (n, m) in sizes {
        let pairs = random_pairs(&nodes, n + m, seed + n as u64);
        let src: Vec<u32> = pairs.iter().take(n).map(|p| p.0).collect();
        let dst: Vec<u32> = pairs.iter().skip(n).take(m).map(|p| p.1).collect();
        let (sl, dl) = (legs(&src), legs(&dst));
        // chạy nóng một lần
        let _ = bucket_matrix(ch, &sl, &dl, false);
        let (b1, t_b1) = ms(|| bucket_matrix(ch, &sl, &dl, false));
        let (bp, t_bp) = ms(|| bucket_matrix(ch, &sl, &dl, true));
        let (r1, t_r1) = ms(|| repeated_matrix(ch, &sl, &dl, false));
        let (rp, t_rp) = ms(|| repeated_matrix(ch, &sl, &dl, true));
        let diff = [&bp, &r1, &rp]
            .iter()
            .map(|x| x.data.iter().zip(&b1.data).filter(|(a, b)| a != b).count())
            .sum::<usize>();
        mismatches_total += diff;
        println!(
            "  {:<10} {:>13.2} {:>13.2} {:>13.2} {:>14.2} {:>11.2} {:>8.1}x {:>9}",
            format!("{n}×{m}"),
            t_b1,
            t_bp,
            t_r1,
            t_rp,
            t_b1 * 1e3 / (n * m) as f64,
            t_r1 / t_b1,
            diff
        );
    }
    println!(
        "  cột «khác nhau»: số ô khác với bucket 1 luồng, cộng ba cách còn lại; tổng = {mismatches_total}"
    );

    println!(
        "\n-- 2. đối chiếu với Dijkstra (20×20 = 400 truy vấn A*) và với N lần Dijkstra một-nguồn-tới-tất-cả --"
    );
    {
        let pairs = random_pairs(&nodes, 40, seed + 7);
        let src: Vec<u32> = pairs.iter().take(20).map(|p| p.0).collect();
        let dst: Vec<u32> = pairs.iter().skip(20).take(20).map(|p| p.1).collect();
        let m = bucket_matrix(ch, &legs(&src), &legs(&dst), true);
        let mut se = Searcher::new(g.node_count());
        let t = Instant::now();
        let mut bad = 0;
        for (i, &a) in src.iter().enumerate() {
            for (j, &b) in dst.iter().enumerate() {
                let want = se
                    .astar(
                        view,
                        &eng.xy,
                        eng.costs(Profile::Car).vmax_cm_per_unit,
                        a,
                        b,
                    )
                    .0
                    .map_or(INF, |p| p.cost);
                bad += usize::from(m.get(i, j) != want);
            }
        }
        let t_astar = t.elapsed().as_secs_f64() * 1e3;
        let t = Instant::now();
        let mut bad_all = 0;
        for (i, &a) in src.iter().enumerate() {
            let d = route_engine::net::sssp(view, a, true);
            for (j, &b) in dst.iter().enumerate() {
                bad_all += usize::from(m.get(i, j) != d[b as usize]);
            }
        }
        let t_sssp = t.elapsed().as_secs_f64() * 1e3;
        println!("  400 ô: khác A* {bad}, khác sssp {bad_all}");
        println!(
            "  thời gian: 400 truy vấn A* {t_astar:.1} ms; 20 lần Dijkstra một-nguồn-tới-tất-cả {t_sssp:.1} ms"
        );
    }

    println!(
        "\n-- 3. ngân sách độ trễ theo hình dạng yêu cầu ({repeats} lần mỗi hình dạng, mẫu mới mỗi lần; ms) --"
    );
    println!(
        "  {:<34} {:>9} {:>9} {:>9} {:>9}",
        "hình dạng", "p50 1 luồng", "p95 1 luồng", "p50 rayon", "p95 rayon"
    );
    let shapes: [(&str, usize, usize); 6] = [
        ("1 khách × 50 tài xế (one-to-many)", 1, 50),
        ("1 khách × 200 ứng viên điểm đón", 1, 200),
        ("50 tài xế × 50 khách (một lô)", 50, 50),
        ("100 × 100", 100, 100),
        ("300 × 300 (lô lớn)", 300, 300),
        ("500 × 500", 500, 500),
    ];
    let mut rng = Rand::new(seed ^ 0x51);
    for (name, n, m) in shapes {
        let (mut t1, mut tp) = (Vec::new(), Vec::new());
        for _ in 0..repeats {
            let pick = |k: usize, rng: &mut Rand| -> Vec<u32> {
                (0..k).map(|_| nodes[rng.usize(nodes.len())]).collect()
            };
            let (sl, dl) = (legs(&pick(n, &mut rng)), legs(&pick(m, &mut rng)));
            t1.push(ms(|| bucket_matrix(ch, &sl, &dl, false)).1);
            tp.push(ms(|| bucket_matrix(ch, &sl, &dl, true)).1);
        }
        println!(
            "  {name:<34} {:>9.2} {:>9.2} {:>9.2} {:>9.2}",
            pct(&mut t1, 50.0),
            pct(&mut t1, 95.0),
            pct(&mut tp, 50.0),
            pct(&mut tp, 95.0)
        );
    }
    println!(
        "  (đo khi máy đang chịu tải, xem uptime ở đầu và cuối; chỉ so sánh tương đối trong cùng một lần chạy)"
    );

    println!(
        "\n-- 4. đích ở một phía đường cụ thể: ma trận trên vị trí có hướng (nền cho LOC-34, LOC-35) --"
    );
    {
        // điểm GPS ngẫu nhiên gần đường, snap, rồi so đích «phía nào cũng được» với «chỉ phía này»
        let mut rng = Rand::new(seed ^ 0x77);
        let mk = |rng: &mut Rand| -> LatLng {
            let n = nodes[rng.usize(nodes.len())];
            geo_core::destination(
                g.node_latlng(n),
                rng.range(0.0, 360.0),
                rng.range(5.0, 40.0),
            )
        };
        let srcs: Vec<_> = (0..100)
            .filter_map(|_| eng.snap_point(Profile::Car, mk(&mut rng), "nguồn").ok())
            .collect();
        let dsts: Vec<_> = (0..100)
            .filter_map(|_| eng.snap_point(Profile::Car, mk(&mut rng), "đích").ok())
            .filter(|s| s.positions(view).len() == 2)
            .collect();
        let sl: Vec<Vec<Leg>> = srcs.iter().map(|s| s.source_legs(view)).collect();
        let both: Vec<Vec<Leg>> = dsts.iter().map(|s| s.target_legs(view)).collect();
        let side = |k: usize| -> Vec<Vec<Leg>> {
            dsts.iter()
                .map(|s| s.positions(view)[k].target_legs(view))
                .collect()
        };
        let (s0, s1) = (side(0), side(1));
        let (mb_, m0, m1) = (
            bucket_matrix(ch, &sl, &both, true),
            bucket_matrix(ch, &sl, &s0, true),
            bucket_matrix(ch, &sl, &s1, true),
        );
        let violated = (0..mb_.data.len())
            .filter(|&i| mb_.data[i] != m0.data[i].min(m1.data[i]))
            .count();
        println!(
            "  {} nguồn × {} đích hai chiều: «hai phía» = min(phía 0, phía 1) đúng ở mọi ô: {} ô vi phạm trên {}",
            srcs.len(),
            dsts.len(),
            violated,
            mb_.data.len()
        );
        let mut extra: Vec<f64> = Vec::new();
        let mut forced_bigger = 0usize;
        for i in 0..mb_.data.len() {
            let worst = m0.data[i].max(m1.data[i]);
            if mb_.data[i] != INF && worst != INF {
                extra.push(f64::from(worst - mb_.data[i]) / 1000.0);
                forced_bigger += usize::from(worst > mb_.data[i]);
            }
        }
        println!(
            "  đi tới điểm đúng phía bất lợi (phía kia của hai phía) tốn thêm so với «phía nào cũng được»: trong {:.1}% ô; trung bình {:.1} s, p95 {:.1} s, lớn nhất {:.1} s",
            100.0 * forced_bigger as f64 / extra.len() as f64,
            mean(&extra),
            pct(&mut extra, 95.0),
            pct(&mut extra, 100.0)
        );
        println!(
            "  (mô hình này chưa tính chi phí quay đầu: chỉ là chi phí đi vòng để vào đúng chiều cạnh; LOC-34 thêm chi phí rẽ)"
        );
    }
    println!("\nuptime lúc kết thúc: {}", uptime());
}
