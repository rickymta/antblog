//! LOC-07: R-tree (rstar) snap điểm GPS về cạnh đường, so với quét toàn bộ, và ca snap sai sang đường song song.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin từ lab LOC-06, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc07_snap
//! Tham số: --queries N (mặc định 20000 cho R-tree), --scan-queries N (mặc định 1000), --seed S, --fixture,
//! --ratio-rounds N (so R-tree với quét xen kẽ N lượt; mặc định không chạy)

use geo_core::geojson::{line_feature, point_feature, to_string};
use geo_core::segment::point_at;
use geo_core::{LatLng, bearing_deg, bearing_diff_deg, destination, haversine_m};
use labs::{Args, Rand, percentile, timed};
use road_graph::snap::{SnapCandidate, SnapIndex};
use road_graph::{Graph, Mode, flag};
use std::hint::black_box;

const SEED: u64 = 20_261_106;

/// Điểm thật trên một cạnh ô tô, hướng đi và cạnh chứa nó.
struct Truth {
    edge: u32,
    point: LatLng,
    heading: f64,
}

fn sample_truth(g: &Graph, edges: &[u32], rng: &mut Rand) -> Truth {
    let e = edges[rng.usize(edges.len())];
    let pts = g.edge_points(e);
    let len: f64 = pts.windows(2).map(|w| haversine_m(w[0], w[1])).sum();
    let at = rng.range(0.0, len);
    let point = point_at(&pts, at).expect("cạnh có điểm");
    let mut acc = 0.0;
    let mut heading = 0.0;
    for w in pts.windows(2) {
        let l = haversine_m(w[0], w[1]);
        heading = bearing_deg(w[0], w[1]);
        if at <= acc + l {
            break;
        }
        acc += l;
    }
    Truth {
        edge: e,
        point,
        heading,
    }
}

/// Nhiễu hai chiều dạng Gaussian với độ lệch chuẩn `sigma` mỗi trục (khoảng cách Rayleigh, hướng đều).
fn noisy(p: LatLng, sigma: f64, rng: &mut Rand) -> LatLng {
    let r = sigma * (-2.0 * (1.0 - rng.f64()).ln()).sqrt();
    destination(p, rng.range(0.0, 360.0), r)
}

/// Góc lệch nhỏ nhất giữa hướng đi và các chiều ô tô được phép của ứng viên.
fn heading_penalty(g: &Graph, c: &SnapCandidate, heading: f64) -> f64 {
    let car = Mode::Car.bit();
    let mut best = 180.0f64;
    if g.modes[c.edge as usize] & car != 0 {
        best = best.min(bearing_diff_deg(heading, c.bearing_deg));
    }
    if let Some(t) = c.twin
        && g.modes[t as usize] & car != 0
    {
        best = best.min(bearing_diff_deg(heading, (c.bearing_deg + 180.0) % 360.0));
    }
    best
}

/// So R-tree với quét xen kẽ từng lượt, để hai bên chịu cùng một mức tải máy. Hai bộ điểm: 256 điểm cố định sát nút
/// lặp lại (như `benches/loc07_snap.rs`, cache đã nóng) và điểm mới mỗi lượt (như mục 2).
fn ratio_rounds(g: &Graph, index: &SnapIndex, queries: &[LatLng], seed: u64, rounds: usize) {
    let mut rng = Rand::new(seed ^ 0x7);
    let fixed: Vec<LatLng> = (0..256)
        .map(|_| {
            let e = loop {
                let e = rng.0.u32(0..g.edge_count() as u32);
                if g.edge(e).allows(Mode::Car) {
                    break e;
                }
            };
            destination(
                g.node_latlng(g.tail[e as usize]),
                rng.range(0.0, 360.0),
                rng.range(0.0, 10.0),
            )
        })
        .collect();
    let per_rtree = |pts: &[LatLng]| {
        let (_, s) = timed(|| {
            pts.iter()
                .filter_map(|&q| black_box(index.nearest(g, black_box(q), Mode::Car)))
                .count()
        });
        s * 1e6 / pts.len() as f64
    };
    let per_scan = |pts: &[LatLng]| {
        let (_, s) = timed(|| {
            pts.iter()
                .filter_map(|&q| black_box(index.nearest_scan(g, black_box(q), Mode::Car)))
                .count()
        });
        s * 1e6 / pts.len() as f64
    };
    println!("  xen kẽ {rounds} lượt, tỉ lệ quét / R-tree từng lượt rồi trung vị:");
    let report = |ten: &str, mut v: Vec<(f64, f64)>| {
        let line: Vec<String> = v.iter().map(|(r, q)| format!("{:.0}", q / r)).collect();
        v.sort_by(|a, b| (a.1 / a.0).total_cmp(&(b.1 / b.0)));
        let (r, q) = v[v.len() / 2];
        println!(
            "    {ten}: {} ; trung vị {:.0} (R-tree {r:.2} µs, quét {q:.1} µs)",
            line.join(" "),
            q / r
        );
    };
    let warm: Vec<(f64, f64)> = (0..rounds)
        .map(|_| {
            let r = (0..20).map(|_| per_rtree(&fixed)).sum::<f64>() / 20.0;
            (r, per_scan(&fixed[..64]))
        })
        .collect();
    report("256 điểm cố định sát nút, lặp lại", warm);
    let fresh: Vec<(f64, f64)> = (0..rounds)
        .map(|i| {
            let n = queries.len() / rounds;
            let chunk = &queries[i * n..(i + 1) * n];
            (per_rtree(chunk), per_scan(&chunk[..64.min(chunk.len())]))
        })
        .collect();
    report("điểm mới mỗi lượt, nhiễu σ = 10 m", fresh);
}

fn main() {
    let args = Args::from_env();
    let seed = args.u64_or("--seed", SEED);
    let (g, source) = labs::load_graph(&args);
    println!("== LOC-07: R-tree và snap ==");
    println!(
        "đồ thị: {source}: {} nút, {} cạnh; seed {seed}\n",
        g.node_count(),
        g.edge_count()
    );

    let (index, build_s) = timed(|| SnapIndex::build(&g));
    println!("-- 1. dựng chỉ mục --");
    println!(
        "  {} đoạn thẳng trong R-tree (cặp hai chiều chỉ lập một lần), bulk_load {:.0} ms",
        index.segment_count(),
        build_s * 1000.0
    );

    let car_edges: Vec<u32> = (0..g.edge_count() as u32)
        .filter(|&e| g.edge(e).allows(Mode::Car) && g.length_cm[e as usize] >= 3_000)
        .collect();
    let mut rng = Rand::new(seed);

    println!(
        "\n-- 2. R-tree so với quét toàn bộ (cùng công thức khoảng cách phẳng, profile ô tô, nhiễu σ = 10 m) --"
    );
    let nq = args.u64_or("--queries", 20_000) as usize;
    let ns = (args.u64_or("--scan-queries", 1_000) as usize).min(nq);
    let queries: Vec<LatLng> = (0..nq)
        .map(|_| {
            let t = sample_truth(&g, &car_edges, &mut rng);
            noisy(t.point, 10.0, &mut rng)
        })
        .collect();
    let (hits, rt_s) = timed(|| {
        queries
            .iter()
            .filter_map(|&q| black_box(index.nearest(&g, black_box(q), Mode::Car)))
            .count()
    });
    let rt_us = rt_s * 1e6 / nq as f64;
    let (scan_res, sc_s) = timed(|| {
        queries[..ns]
            .iter()
            .map(|&q| index.nearest_scan(&g, black_box(q), Mode::Car))
            .collect::<Vec<_>>()
    });
    let sc_us = sc_s * 1e6 / ns as f64;
    let mut differ = Vec::new();
    for (&q, s) in queries[..ns].iter().zip(&scan_res) {
        let r = index.nearest(&g, q, Mode::Car);
        if r.map(|c| c.edge) != s.map(|c| c.edge) {
            differ.push((q, r, *s));
        }
    }
    let agree = ns - differ.len();
    println!(
        "  R-tree (nearest_neighbor_iter): {nq} truy vấn, {hits} có kết quả, {:.2} µs/truy vấn",
        rt_us
    );
    println!(
        "  quét toàn bộ {} đoạn:           {ns} truy vấn, {:.1} µs/truy vấn",
        index.segment_count(),
        sc_us
    );
    println!(
        "  tỉ lệ thời gian quét / R-tree = {:.0}; cùng cạnh gần nhất ở {agree}/{ns} truy vấn",
        sc_us / rt_us
    );
    for (q, r, s) in differ
        .iter()
        .filter_map(|(q, r, s)| Some((q, (*r)?, (*s)?)))
    {
        println!("  truy vấn khác cạnh, điểm {q}:");
        for (ten, c) in [("R-tree", r), ("quét  ", s)] {
            println!(
                "    {ten}: cạnh {} (nút {} → {}), đoạn {}, t = {:.6}, cách {:.9} m, chân {}",
                c.edge,
                g.tail[c.edge as usize],
                g.head[c.edge as usize],
                c.segment,
                c.t,
                c.dist_m,
                c.point
            );
        }
    }
    println!("  (đo chặt hơn bằng criterion: cargo bench -p labs --bench loc07_snap)");
    let rounds = args.u64_or("--ratio-rounds", 0) as usize;
    if rounds > 0 {
        ratio_rounds(&g, &index, &queries, seed, rounds);
    }

    println!(
        "\n-- 3. ứng viên theo bán kính (σ = 10 m): số cạnh trong bán kính và tỉ lệ có cạnh thật --"
    );
    println!(
        "{:>12} {:>16} {:>14} {:>10}",
        "bán kính (m)", "ứng viên TB", "ứng viên lớn nhất", "có cạnh thật"
    );
    let trials = 5_000;
    let truths: Vec<(Truth, LatLng)> = (0..trials)
        .map(|_| {
            let t = sample_truth(&g, &car_edges, &mut rng);
            let q = noisy(t.point, 10.0, &mut rng);
            (t, q)
        })
        .collect();
    for radius in [10.0, 20.0, 30.0, 50.0, 100.0] {
        let (mut total, mut max, mut found) = (0usize, 0usize, 0usize);
        for (t, q) in &truths {
            let cs = index.candidates(&g, *q, radius, Mode::Car, usize::MAX);
            total += cs.len();
            max = max.max(cs.len());
            let truth_way = g.way_id[t.edge as usize];
            if cs.iter().any(|c| g.way_id[c.edge as usize] == truth_way) {
                found += 1;
            }
        }
        println!(
            "{radius:>12.0} {:>16.2} {max:>14} {:>9.1}%",
            total as f64 / trials as f64,
            100.0 * found as f64 / trials as f64
        );
    }

    println!("\n-- 4. snap sai: tỉ lệ cạnh gần nhất không thuộc way thật (20.000 điểm, ô tô) --");
    println!(
        "  nhiễu GPS = Gaussian σ mỗi trục; «có hướng»: chọn trong bán kính max(30, 3σ) m theo điểm = khoảng cách + 0,5 m × độ lệch hướng (hướng nhiễu σ 10°)"
    );
    println!(
        "{:>7} {:>16} {:>16}",
        "σ (m)", "gần nhất: sai", "có hướng: sai"
    );
    let n = 20_000;
    let mut first_cases: Vec<(f64, Truth, LatLng, SnapCandidate, Option<SnapCandidate>)> =
        Vec::new();
    let mut bridge_case: Option<(Truth, LatLng, SnapCandidate)> = None;
    let (mut wrong_total_10, mut wrong_bridge_10, mut wrong_between_15_40) =
        (0usize, 0usize, 0usize);
    for sigma in [3.0, 5.0, 10.0, 15.0, 20.0, 30.0] {
        let (mut wrong_plain, mut wrong_heading) = (0usize, 0usize);
        for _ in 0..n {
            let t = sample_truth(&g, &car_edges, &mut rng);
            let q = noisy(t.point, sigma, &mut rng);
            let truth_way = g.way_id[t.edge as usize];
            let Some(near) = index.nearest(&g, q, Mode::Car) else {
                continue;
            };
            let is_wrong = g.way_id[near.edge as usize] != truth_way;
            if is_wrong {
                wrong_plain += 1;
            }
            let hd = (t.heading + rng.gauss() * 10.0).rem_euclid(360.0);
            let radius = (3.0 * sigma).max(30.0);
            let best = index
                .candidates(&g, q, radius, Mode::Car, usize::MAX)
                .into_iter()
                .min_by(|a, b| {
                    (a.dist_m + 0.5 * heading_penalty(&g, a, hd))
                        .total_cmp(&(b.dist_m + 0.5 * heading_penalty(&g, b, hd)))
                });
            let fixed = best.is_some_and(|b| g.way_id[b.edge as usize] == truth_way);
            if !fixed {
                wrong_heading += 1;
            }
            if sigma == 10.0 && is_wrong {
                wrong_total_10 += 1;
                let sep = haversine_m(t.point, near.point);
                let bridgey = g.edge(near.edge).has(flag::BRIDGE | flag::TUNNEL)
                    || g.edge(t.edge).has(flag::BRIDGE | flag::TUNNEL);
                if bridgey {
                    wrong_bridge_10 += 1;
                    // cầu vượt thật sự: một bên là cầu/hầm, bên kia nằm dưới/trên, cách nhau từ 8 m
                    let differ = g.edge(near.edge).has(flag::BRIDGE | flag::TUNNEL)
                        != g.edge(t.edge).has(flag::BRIDGE | flag::TUNNEL);
                    if bridge_case.is_none()
                        && differ
                        && g.edge(near.edge).name().is_some()
                        && (8.0..60.0).contains(&sep)
                    {
                        bridge_case = Some((
                            Truth {
                                edge: t.edge,
                                point: t.point,
                                heading: t.heading,
                            },
                            q,
                            near,
                        ));
                    }
                }
                if (15.0..40.0).contains(&sep) {
                    wrong_between_15_40 += 1;
                    let main_road = |e: u32| {
                        use road_graph::HighwayClass::*;
                        matches!(g.edge(e).class(), Trunk | Primary | Secondary | Tertiary)
                    };
                    let named_diff = g.edge(near.edge).name() != g.edge(t.edge).name()
                        && g.edge(near.edge).name().is_some()
                        && g.edge(t.edge).name().is_some()
                        && main_road(near.edge)
                        && main_road(t.edge);
                    if named_diff && first_cases.len() < 3 {
                        first_cases.push((
                            sigma,
                            Truth {
                                edge: t.edge,
                                point: t.point,
                                heading: t.heading,
                            },
                            q,
                            near,
                            best,
                        ));
                    }
                }
            }
        }
        println!(
            "{sigma:>7.0} {:>15.2}% {:>15.2}%",
            100.0 * wrong_plain as f64 / n as f64,
            100.0 * wrong_heading as f64 / n as f64
        );
    }
    println!(
        "  σ = 10 m: {wrong_total_10} ca sai; trong đó chân snap cách điểm thật 15–40 m (đường song song hoặc gần nhau): {wrong_between_15_40}; liên quan cầu/hầm: {wrong_bridge_10}"
    );

    println!(
        "\n-- 5. ba ca snap sai cụ thể (σ = 10 m, hai đường chính (trunk…tertiary) khác tên, chân snap cách điểm thật 15–40 m) --"
    );
    for (i, (sigma, t, q, near, fixed)) in first_cases.iter().enumerate() {
        describe_case(&g, i + 1, *sigma, t, *q, near, fixed.as_ref());
    }
    if let Some((t, q, near)) = &bridge_case {
        println!("\n-- 6. một ca liên quan cầu/hầm --");
        describe_case(&g, 0, 10.0, t, *q, near, None);
    }

    for (i, (_, t, q, near, _)) in first_cases.iter().enumerate() {
        let tname = g.edge(t.edge).name().unwrap_or("").to_string();
        let sname = g.edge(near.edge).name().unwrap_or("").to_string();
        let fc = to_string(vec![
            line_feature(
                &g.edge_points(t.edge),
                &[("vai_tro", "canh that".into()), ("ten", tname.into())],
            ),
            line_feature(
                &g.edge_points(near.edge),
                &[
                    ("vai_tro", "canh bi snap sai".into()),
                    ("ten", sname.into()),
                ],
            ),
            point_feature(t.point, &[("vai_tro", "vi tri that".into())]),
            point_feature(*q, &[("vai_tro", "diem GPS nhieu".into())]),
            point_feature(near.point, &[("vai_tro", "chan snap gan nhat".into())]),
        ]);
        let name = format!(
            "geojson/loc07-{}-snap-sai-{}.geojson",
            if args.flag("--fixture") {
                "khu-mau"
            } else {
                "hcm"
            },
            i + 1
        );
        let path = labs::results_dir().join(&name);
        std::fs::create_dir_all(path.parent().expect("có thư mục cha")).expect("tạo thư mục");
        std::fs::write(&path, &fc).expect("ghi geojson");
        println!(
            "\n  đã ghi results/{name} ({} byte): ca {} ở mục 5",
            fc.len(),
            i + 1
        );
    }

    let mut lens: Vec<f64> = Vec::new();
    for _ in 0..1000 {
        let t = sample_truth(&g, &car_edges, &mut rng);
        let q = noisy(t.point, 10.0, &mut rng);
        if let Some(c) = index.nearest(&g, q, Mode::Car) {
            lens.push(c.dist_m);
        }
    }
    lens.sort_by(f64::total_cmp);
    println!(
        "\n  khoảng cách GPS tới cạnh gần nhất (σ = 10 m, 1000 mẫu): p50 {:.1} m, p90 {:.1} m, p99 {:.1} m",
        percentile(&lens, 50.0),
        percentile(&lens, 90.0),
        percentile(&lens, 99.0)
    );
}

fn describe_case(
    g: &Graph,
    no: usize,
    sigma: f64,
    t: &Truth,
    q: LatLng,
    near: &SnapCandidate,
    fixed: Option<&SnapCandidate>,
) {
    let te = g.edge(t.edge);
    let ne = g.edge(near.edge);
    let tc = te.name().unwrap_or("(không tên)");
    let nc = ne.name().unwrap_or("(không tên)");
    let true_dist = road_graph::snap::SnapIndex::build_one_distance(g, t.edge, q);
    println!("  ca {no}: σ = {sigma} m");
    println!("    vị trí thật   = {}   hướng {:.0}°", t.point, t.heading);
    println!(
        "    điểm GPS      = {q}   (lệch {:.1} m)",
        haversine_m(t.point, q)
    );
    println!(
        "    cạnh thật     = cạnh {} way {} «{}» highway={} {}{}  cách GPS {:.1} m",
        t.edge,
        te.way_id(),
        tc,
        te.class().osm_value(),
        if te.has(flag::BRIDGE) { "cầu " } else { "" },
        if te.has(flag::TUNNEL) { "hầm " } else { "" },
        true_dist
    );
    println!(
        "    snap gần nhất = cạnh {} way {} «{}» highway={} {}{}  cách GPS {:.1} m, chân snap {}",
        near.edge,
        ne.way_id(),
        nc,
        ne.class().osm_value(),
        if ne.has(flag::BRIDGE) { "cầu " } else { "" },
        if ne.has(flag::TUNNEL) { "hầm " } else { "" },
        near.dist_m,
        near.point
    );
    println!(
        "    hai đường cách nhau (từ vị trí thật tới chân snap) = {:.1} m",
        haversine_m(t.point, near.point)
    );
    if let Some(f) = fixed {
        println!(
            "    chọn có hướng = way {} «{}» ({})",
            g.edge(f.edge).way_id(),
            g.edge(f.edge).name().unwrap_or("(không tên)"),
            if g.edge(f.edge).way_id() == te.way_id() {
                "đúng"
            } else {
                "vẫn sai"
            }
        );
    }
}
