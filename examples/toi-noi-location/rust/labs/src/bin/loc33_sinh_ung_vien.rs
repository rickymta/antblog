//! LOC-33: sinh ứng viên điểm đón. Dijkstra có giới hạn trên đồ thị đi bộ, snap về cạnh xe vào được, loại cầu vượt, hầm,
//! điểm cấm dừng, gom ứng viên gần nhau, và đếm số ứng viên ở từng bước.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc33_sinh_ung_vien
//! Tham số: --fixture, --n N (số khách mẫu, mặc định 300)

use geo_core::haversine_m;
use labs::p03::{mean, uptime};
use labs::p04::*;
use labs::{Args, percentile};
use pickup::candidates::{CandidateGen, GenConfig, GenStats, stop_violation};
use pickup::objective::Violation;
use road_graph::Mode;
use route_engine::net::INF;
use route_engine::profile::Profile;
use simulator::world::generate;
use std::time::Instant;

fn stat(label: &str, v: &mut [f64]) {
    v.sort_by(f64::total_cmp);
    println!(
        "  {:<34} trung bình {:>7.1}  p50 {:>6.0}  p95 {:>6.0}  max {:>6.0}",
        label,
        mean(v),
        percentile(v, 50.0),
        percentile(v, 95.0),
        v.last().copied().unwrap_or(f64::NAN)
    );
}

fn main() {
    let args = Args::from_env();
    let env = setup(&args);
    let e = &env.engine;
    println!("== LOC-33: sinh ứng viên điểm đón từ đồ thị đi bộ ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let seed = *EVAL_SEEDS.start();
    let sc = with_seed(&env.scenario, seed);
    let n = args.u64_or("--n", 300) as usize;
    println!(
        "cấu hình: kịch bản {}, seed {seed}, đồ thị {}, {n} khách đầu tiên của seed (chỗ đứng thật + ghim, trên dữ liệu mô phỏng); đi bộ ≤ 250 m, gom trong 25 m, cạnh ≥ 20 m",
        sc.name, env.graph_info
    );
    let world = generate(e, &sc);
    let custs: Vec<_> = world.customers.iter().take(n).collect();
    let mut gen_ = CandidateGen::new(e, GenConfig::default());

    println!("\n-- 1. đếm ứng viên ở từng bước (ngân sách đi bộ 250 m) --");
    let mut cols: [Vec<f64>; 9] = Default::default();
    let mut totals = GenStats::default();
    let mut us = Vec::new();
    let mut violations_in_output = 0usize;
    let mut walk_over = 0usize;
    for c in &custs {
        let t = Instant::now();
        let g = gen_.generate(e, c.view.true_pos, c.view.pin);
        us.push(t.elapsed().as_micros() as f64);
        let s = g.stats;
        for (i, v) in [
            s.reached_nodes,
            s.raw,
            s.excl_bridge_tunnel,
            s.excl_no_stop_class + s.excl_roundabout,
            s.excl_too_short,
            s.excl_walk,
            s.merged_away,
            s.kept,
            g.pin.len(),
        ]
        .into_iter()
        .enumerate()
        {
            cols[i].push(v as f64);
        }
        totals.raw += s.raw;
        totals.excl_bridge_tunnel += s.excl_bridge_tunnel;
        totals.excl_no_stop_class += s.excl_no_stop_class;
        totals.excl_roundabout += s.excl_roundabout;
        totals.excl_too_short += s.excl_too_short;
        totals.excl_walk += s.excl_walk;
        totals.merged_away += s.merged_away;
        totals.kept += s.kept;
        for k in &g.candidates {
            violations_in_output += usize::from(stop_violation(e, k.pos.edge, 20.0).is_some());
            walk_over += usize::from(k.walk_est_m > 250.0 + 1e-6);
        }
    }
    let names = [
        "nút đi bộ tới được",
        "cạnh xe có đầu trong tầm (theo chiều)",
        "loại: cầu, hầm, cầu vượt",
        "loại: cao tốc, nhánh nối, vòng xuyến",
        "loại: cạnh quá ngắn (< 20 m)",
        "loại: quá 250 m đi bộ",
        "gom vào ứng viên gần (< 25 m)",
        "ứng viên còn lại",
        "ứng viên là điểm ghim (mỗi chiều)",
    ];
    for (i, nm) in names.iter().enumerate() {
        stat(nm, &mut cols[i]);
    }
    let raw = totals.raw.max(1) as f64;
    println!(
        "  tổng trên {} khách: {} cạnh theo chiều → loại cầu/hầm {} ({:.1}%), cao tốc/vòng xuyến {} ({:.1}%), quá ngắn {} ({:.1}%), quá xa {} ({:.1}%), gom {} ({:.1}%), còn {} ({:.1}%)",
        custs.len(),
        totals.raw,
        totals.excl_bridge_tunnel,
        100.0 * totals.excl_bridge_tunnel as f64 / raw,
        totals.excl_no_stop_class + totals.excl_roundabout,
        100.0 * (totals.excl_no_stop_class + totals.excl_roundabout) as f64 / raw,
        totals.excl_too_short,
        100.0 * totals.excl_too_short as f64 / raw,
        totals.excl_walk,
        100.0 * totals.excl_walk as f64 / raw,
        totals.merged_away,
        100.0 * totals.merged_away as f64 / raw,
        totals.kept,
        100.0 * totals.kept as f64 / raw
    );
    println!(
        "  kiểm tra: ứng viên đầu ra vi phạm ràng buộc cứng (cầu/hầm/cấm dừng/vòng xuyến/quá ngắn) = {violations_in_output}; ứng viên đi bộ ước lượng quá 250 m = {walk_over}"
    );
    us.sort_by(f64::total_cmp);
    println!(
        "  thời gian một lần sinh (µs, đo khi máy đang chịu tải nặng, xem uptime): p50 {:.0}  p95 {:.0}  max {:.0}",
        percentile(&us, 50.0),
        percentile(&us, 95.0),
        us.last().copied().unwrap_or(f64::NAN)
    );

    println!("\n-- 2. Dijkstra có giới hạn so với «mọi cạnh trong bán kính chim bay» --");
    let (mut crow_cnt, mut walk_cnt, mut crow_not_walk) = (Vec::new(), Vec::new(), Vec::new());
    let view = e.view(Profile::Car);
    for c in &custs {
        let within = e
            .snap
            .candidates(&e.g, c.view.true_pos, 250.0, Mode::Car, 5000);
        let reach = gen_.foot_reach(e, c.view.true_pos).0;
        let walkable = |ed: u32| {
            [e.g.tail[ed as usize], e.g.head[ed as usize]]
                .iter()
                .any(|n| reach.contains_key(n))
        };
        let crow: Vec<u32> = within
            .iter()
            .map(|s| s.edge)
            .filter(|&ed| view.w[ed as usize] != INF)
            .collect();
        crow_cnt.push(crow.len() as f64);
        let wk = crow.iter().filter(|&&ed| walkable(ed)).count();
        walk_cnt.push(wk as f64);
        crow_not_walk.push((crow.len() - wk) as f64);
    }
    stat("cạnh xe trong 250 m chim bay", &mut crow_cnt);
    stat("trong số đó đi bộ tới được ≤ 250 m", &mut walk_cnt);
    stat("chim bay gần nhưng đi bộ quá 250 m", &mut crow_not_walk);
    let nodes_total = e.g.node_count() as f64;
    let mut reach_n: Vec<f64> = custs
        .iter()
        .map(|c| gen_.foot_reach(e, c.view.true_pos).0.len() as f64)
        .collect();
    println!(
        "  Dijkstra có giới hạn duyệt trung bình {:.1} nút trên tổng {:.0} nút của đồ thị ({:.4}%)",
        mean(&reach_n),
        nodes_total,
        100.0 * mean(&reach_n) / nodes_total
    );
    reach_n.clear();

    println!("\n-- 3. ngân sách đi bộ: số ứng viên còn lại và thời gian --");
    println!(
        "  {:>10} {:>14} {:>14} {:>14}",
        "ngân sách", "ứng viên TB", "ứng viên p95", "µs p50"
    );
    for budget in [100.0, 150.0, 250.0, 400.0] {
        let cfg = GenConfig {
            max_walk_m: budget,
            ..GenConfig::default()
        };
        let mut gg = CandidateGen::new(e, cfg);
        let (mut k, mut t) = (Vec::new(), Vec::new());
        for c in &custs {
            let s = Instant::now();
            let g = gg.generate(e, c.view.true_pos, c.view.pin);
            t.push(s.elapsed().as_micros() as f64);
            k.push(g.candidates.len() as f64);
        }
        k.sort_by(f64::total_cmp);
        t.sort_by(f64::total_cmp);
        println!(
            "  {:>8.0} m {:>14.1} {:>14.0} {:>14.0}",
            budget,
            mean(&k),
            percentile(&k, 95.0),
            percentile(&t, 50.0)
        );
    }

    println!("\n-- 4. một khách mẫu (trích để vẽ hình): chỗ đứng, ghim, ứng viên, điểm bị loại --");
    // khách có nhiều loại cạnh bị loại nhất
    let pick = custs
        .iter()
        .max_by_key(|c| {
            let g = gen_.generate(e, c.view.true_pos, c.view.pin);
            g.stats.excl_bridge_tunnel * 4
                + g.stats.excl_no_stop_class
                + g.stats.excl_roundabout
                + g.stats.merged_away
        })
        .expect("có khách");
    let g = gen_.generate(e, pick.view.true_pos, pick.view.pin);
    println!(
        "  khách {}: chỗ đứng ({:.6}, {:.6}), ghim ({:.6}, {:.6}), cách nhau {:.1} m",
        pick.view.id,
        pick.view.true_pos.lat(),
        pick.view.true_pos.lng(),
        pick.view.pin.lat(),
        pick.view.pin.lng(),
        haversine_m(pick.view.true_pos, pick.view.pin)
    );
    println!("  đếm: {:?}", g.stats);
    println!("  ứng viên còn lại (vĩ độ, kinh độ, đi bộ ước lượng m, phía, loại đường, đã gom):");
    let mut cs = g.candidates.clone();
    cs.sort_by(|a, b| a.walk_est_m.total_cmp(&b.walk_est_m));
    for k in cs.iter().take(8) {
        println!(
            "    ({:.6}, {:.6}) {:>6.1} m  {:?}  {}  gom {}",
            k.point.lat(),
            k.point.lng(),
            k.walk_est_m,
            k.side,
            k.class.osm_value(),
            k.merged
        );
    }
    let reach = gen_.foot_reach(e, pick.view.true_pos).0;
    let mut seen = std::collections::BTreeSet::new();
    println!("  cạnh bị loại vì cấm dừng (tối đa 8, tọa độ giữa cạnh, lý do):");
    let mut shown = 0;
    let mut nodes: Vec<_> = reach.keys().copied().collect();
    nodes.sort_unstable();
    'o: for u in nodes {
        for (ed, _, _) in view.out(u).chain(view.inn(u)) {
            if !seen.insert(ed) {
                continue;
            }
            if let Some(v) = stop_violation(e, ed, 20.0)
                && v != Violation::TooShort
            {
                let p = route_engine::position::point_on_edge(&e.g, ed, 0.5);
                println!(
                    "    ({:.6}, {:.6}) {}",
                    p.lat(),
                    p.lng(),
                    pickup::rank::violation_name(v)
                );
                shown += 1;
                if shown >= 8 {
                    break 'o;
                }
            }
        }
    }
    println!("\nuptime lúc kết thúc: {}", uptime());
}
