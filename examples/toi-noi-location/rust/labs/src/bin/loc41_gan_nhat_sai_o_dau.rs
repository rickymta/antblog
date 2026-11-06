//! LOC-41: tài xế gần nhất sai ở đâu. Đo (1) chim bay so với ETA đường bộ, (2) phản ví dụ «tham lam cướp tài xế»,
//! (3) baseline của cả phần P05: so có cặp trên simulator, nhiều seed đánh giá, nhiều bộ giả định.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc41_gan_nhat_sai_o_dau
//! Tham số: --seeds N (mặc định 20), --jobs N (mặc định 3), --trials N (mẫu cho phần 1–2, mặc định 2000), --fixture,
//! --ca-te (chỉ chạy mục 1, in thêm tuyến thật của ca tệ nhất rồi dừng),
//! --variant 'thường' --skip-seeds 20 (lặp lại mục 3 trên dải seed chưa dùng, để xem một kết quả có lặp lại không)

use geo_core::haversine_m;
use labs::p03::uptime;
use labs::p05::{
    f1, jobs, n_seeds, pickup_of, print_paired, print_zones, select_variants, setup, variants,
};
use labs::{Args, Rand, percentile};
use matcher::assign::{greedy_fifo, hungarian, total_cost};
use matcher::batch::{BatchMatcher, MatcherConfig};
use matcher::cost::{Dense, eta_matrix_ms};
use matcher::eval::{Zones, eval_seeds, sweep};
use route_engine::engine::Engine;
use route_engine::position::Position;
use route_engine::profile::Profile;
use route_engine::turn::{EdgeRoute, Goal, Start};
use simulator::world::{Router, generate};
use simulator::{AtPin, NearestStraightLine, RunOptions, Scenario, run};

fn pool(
    engine: &Engine,
    sc: &Scenario,
) -> (Vec<(Position, geo_core::LatLng)>, Vec<geo_core::LatLng>) {
    let w = generate(engine, sc);
    let drivers = w
        .drivers
        .iter()
        .map(|d| (d.pos, d.pos.point(&engine.g)))
        .collect();
    let pins = w.customers.iter().map(|c| c.view.pin).collect();
    (drivers, pins)
}

/// Tuyến thật của ca tệ nhất: định tuyến theo cạnh có hướng như simulator, và theo nút như ma trận CH sau khi snap lại không xét hướng.
fn explain_worst(
    engine: &Engine,
    sc: &Scenario,
    drivers: &[(Position, geo_core::LatLng)],
    (pin, crow_driver, eta_driver): (geo_core::LatLng, usize, usize),
) {
    println!("  tuyến của ca tệ nhất (bbox đồ thị HCM: lat 10,70–10,86, lng 106,60–106,78):");
    let router = Router::new(engine, sc);
    let Some((_, approach)) = pickup_of(engine, pin) else {
        return;
    };
    for (label, d) in [("chim bay chọn", crow_driver), ("ETA chọn", eta_driver)] {
        let (pos, ll) = drivers[d];
        let mut best: Option<EdgeRoute> = None;
        for &a in &approach {
            if let Some(r) = router.route_car(0, Start::Pos(pos), Goal::Pos(a))
                && best.as_ref().is_none_or(|b| r.cost < b.cost)
            {
                best = Some(r);
            }
        }
        let Some(r) = best else {
            println!("    {label} DR-{d:03}: không có tuyến");
            continue;
        };
        let pts: Vec<geo_core::LatLng> = r
            .edges
            .iter()
            .flat_map(|&e| engine.g.edge_points(e))
            .collect();
        let (mut lat_min, mut lng_min) = (f64::MAX, f64::MAX);
        let far = pts
            .iter()
            .map(|&p| {
                lat_min = lat_min.min(p.lat());
                lng_min = lng_min.min(p.lng());
                haversine_m(p, pin)
            })
            .fold(0.0, f64::max);
        println!(
            "    {label} DR-{d:03} ({:.5}, {:.5}): tuyến có hướng {:.0} m, {:.0} s (kể cả chi phí rẽ {:.0} s), {} cạnh; xa ghim nhất {:.0} m; lat nhỏ nhất {:.5}, lng nhỏ nhất {:.5}",
            ll.lat(),
            ll.lng(),
            r.length_cm(&engine.g) as f64 / 100.0,
            f64::from(r.cost) / 1000.0,
            f64::from(r.turn_cost) / 1000.0,
            r.edges.len(),
            far,
            lat_min,
            lng_min
        );
        if let Ok(u) = engine.route(Profile::Car, ll, pin, None) {
            println!(
                "      snap lại tài xế không xét hướng xe đang chạy (được đi cả hai chiều cạnh nếu đường cho phép): {:.0} m, {:.0} s",
                u.distance_m,
                f64::from(u.duration_ms) / 1000.0
            );
        }
    }
    // DR-079 đứng sát mép cắt bản đồ: thử lại trên đồ thị cắt rộng hơn về phía nam và tây để xem đường vòng có do mép cắt không
    let pbf = labs::pbf_path();
    if !pbf.exists() {
        println!("  không có {}: bỏ qua phép thử mép bản đồ", pbf.display());
        return;
    }
    let wide = [10.62, 10.86, 106.54, 106.78];
    let Ok(g) = road_graph::compile_pbf(&pbf, wide) else {
        println!("  không đọc được PBF: bỏ qua phép thử mép bản đồ");
        return;
    };
    println!(
        "  cùng hai cặp điểm trên đồ thị cắt rộng hơn (lat {:.2}–{:.2}, lng {:.2}–{:.2}): {} nút, {} cạnh",
        wide[0],
        wide[1],
        wide[2],
        wide[3],
        g.node_count(),
        g.edge_count()
    );
    let e = Engine::new(g, route_engine::engine::EngineConfig::default());
    for (label, d) in [("chim bay chọn", crow_driver), ("ETA chọn", eta_driver)] {
        match e.route(Profile::Car, drivers[d].1, pin, None) {
            Ok(u) => println!(
                "    {label} DR-{d:03}: {:.0} m, {:.0} s",
                u.distance_m,
                f64::from(u.duration_ms) / 1000.0
            ),
            Err(err) => println!("    {label} DR-{d:03}: không có tuyến ({err:?})"),
        }
    }
}

fn main() {
    let args = Args::from_env();
    println!("== LOC-41: tài xế gần nhất sai ở đâu (trên dữ liệu mô phỏng) ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let (engine, info, sc, fixture) = setup(&args);
    let zones = Zones::hcm();
    println!(
        "đồ thị: {info}; kịch bản gốc: {} (seed gốc {})",
        sc.name, sc.seed
    );
    let trials = args.u64_or("--trials", if fixture { 200 } else { 2000 }) as usize;

    // ---------- 1. chim bay so với ETA ----------
    println!(
        "\n-- 1. chọn theo chim bay hay theo ETA đường bộ (mẫu {trials} khách, mỗi khách 80 tài xế rảnh ngẫu nhiên trong 120) --"
    );
    let (drivers, pins) = pool(&engine, &sc);
    let mut rng = Rand::new(41_001);
    let idle_n = (drivers.len() * 2 / 3).max(2);
    let (mut regrets, mut differ, mut tried) = (Vec::new(), 0usize, 0usize);
    let mut worst: Option<(f64, String)> = None;
    let mut worst_case: Option<(geo_core::LatLng, usize, usize)> = None;
    let mut ratio_far = Vec::new();
    while tried < trials {
        let pin = pins[rng.usize(pins.len())];
        let Some((stop, ap)) = pickup_of(&engine, pin) else {
            continue;
        };
        let mut idx: Vec<usize> = (0..drivers.len()).collect();
        for i in 0..idle_n {
            let j = i + rng.usize(idx.len() - i);
            idx.swap(i, j);
        }
        idx.truncate(idle_n);
        let from: Vec<Position> = idx.iter().map(|&i| drivers[i].0).collect();
        let eta = eta_matrix_ms(&engine, &from, &[ap.as_slice()]);
        let crow: Vec<f64> = idx
            .iter()
            .map(|&i| haversine_m(drivers[i].1, stop))
            .collect();
        let by_crow = (0..idx.len())
            .min_by(|&a, &b| crow[a].total_cmp(&crow[b]))
            .expect("không rỗng");
        let by_eta = (0..idx.len())
            .min_by_key(|&i| eta.get(i, 0))
            .expect("không rỗng");
        tried += 1;
        let (e_c, e_e) = (eta.get(by_crow, 0), eta.get(by_eta, 0));
        if e_c >= matcher::cost::FORBIDDEN / 2 || e_e >= matcher::cost::FORBIDDEN / 2 {
            tried -= 1;
            continue;
        }
        let regret = (e_c - e_e) as f64 / 1000.0;
        regrets.push(regret);
        if by_crow != by_eta {
            differ += 1;
            ratio_far.push(e_c as f64 / e_e.max(1) as f64);
            if worst.as_ref().is_none_or(|w| regret > w.0) {
                worst_case = Some((pin, idx[by_crow], idx[by_eta]));
                worst = Some((
                    regret,
                    format!(
                        "khách ghim ({:.5}, {:.5}); chim bay chọn tài xế DR-{:03} cách {:.0} m chim bay, ETA {:.0} s; ETA chọn DR-{:03} cách {:.0} m chim bay, ETA {:.0} s; thiệt {:.0} s ({:.1} phút), tỉ lệ ETA {:.2}×",
                        pin.lat(),
                        pin.lng(),
                        idx[by_crow],
                        crow[by_crow],
                        e_c as f64 / 1000.0,
                        idx[by_eta],
                        crow[by_eta],
                        e_e as f64 / 1000.0,
                        regret,
                        regret / 60.0,
                        e_c as f64 / e_e.max(1) as f64
                    ),
                ));
            }
        }
    }
    let mut sorted = regrets.clone();
    sorted.sort_by(f64::total_cmp);
    let diff_regret: Vec<f64> = {
        let mut v: Vec<f64> = regrets.iter().copied().filter(|&r| r > 0.0).collect();
        v.sort_by(f64::total_cmp);
        v
    };
    println!(
        "  hai cách chọn KHÁC nhau ở {differ}/{tried} khách ({:.1}%)",
        100.0 * differ as f64 / tried as f64
    );
    println!(
        "  thiệt hại ETA của «gần nhất chim bay» so với «gần nhất ETA» (giây, trên mọi khách): trung bình {:.1}  p50 {:.1}  p95 {:.1}  p99 {:.1}  max {:.1}",
        regrets.iter().sum::<f64>() / regrets.len() as f64,
        percentile(&sorted, 50.0),
        percentile(&sorted, 95.0),
        percentile(&sorted, 99.0),
        sorted.last().copied().unwrap_or(0.0)
    );
    if !diff_regret.is_empty() {
        println!(
            "  chỉ trên các khách mà hai cách khác nhau (n = {}): thiệt hại p50 {:.1} s, p95 {:.1} s; ETA của lựa chọn chim bay gấp {:.2}× (trung vị) ETA tốt nhất",
            diff_regret.len(),
            percentile(&diff_regret, 50.0),
            percentile(&diff_regret, 95.0),
            {
                ratio_far.sort_by(f64::total_cmp);
                percentile(&ratio_far, 50.0)
            }
        );
    }
    if let Some((_, w)) = worst {
        println!("  ca tệ nhất trong mẫu: {w}");
    }
    if args.flag("--ca-te") {
        if let Some(case) = worst_case {
            explain_worst(&engine, &sc, &drivers, case);
        }
        println!("uptime lúc kết thúc: {}", uptime());
        return;
    }

    // ---------- 2. phản ví dụ: tham lam cướp tài xế ----------
    println!("\n-- 2. phản ví dụ: tham lam theo thứ tự đặt «cướp» tài xế của khách sau --");
    println!(
        "  lô gồm 2 khách đặt gần nhau (ghim cách nhau ≤ 1500 m) và 3 tài xế rảnh gần nhất theo chim bay quanh trung điểm; chi phí = ETA đường bộ (s)"
    );
    let mut rng = Rand::new(41_002);
    let (mut batches, mut sub, mut excess) = (0usize, 0usize, Vec::new());
    let mut best_ex: Option<(f64, String)> = None;
    while batches < trials {
        let (a, b) = (pins[rng.usize(pins.len())], pins[rng.usize(pins.len())]);
        if a == b || haversine_m(a, b) > 1500.0 {
            continue;
        }
        let (Some((sa, apa)), Some((sb, apb))) = (pickup_of(&engine, a), pickup_of(&engine, b))
        else {
            continue;
        };
        let mid = geo_core::LatLng::try_new((a.lat() + b.lat()) / 2.0, (a.lng() + b.lng()) / 2.0)
            .expect("hợp lệ");
        let mut idx: Vec<usize> = (0..drivers.len()).collect();
        for i in 0..idle_n {
            let j = i + rng.usize(idx.len() - i);
            idx.swap(i, j);
        }
        idx.truncate(idle_n);
        idx.sort_by(|&x, &y| {
            haversine_m(drivers[x].1, mid).total_cmp(&haversine_m(drivers[y].1, mid))
        });
        idx.truncate(3);
        let from: Vec<Position> = idx.iter().map(|&i| drivers[i].0).collect();
        let eta = eta_matrix_ms(&engine, &from, &[apa.as_slice(), apb.as_slice()]).transpose(); // hàng = khách
        if eta.data.iter().any(|&v| v >= matcher::cost::FORBIDDEN / 2) {
            continue;
        }
        batches += 1;
        let c: Dense = eta;
        let (g, h) = (greedy_fifo(&c), hungarian(&c));
        let (gc, hc) = (total_cost(&c, &g), total_cost(&c, &h));
        if gc > hc {
            sub += 1;
            let ex = (gc - hc) as f64 / 1000.0;
            excess.push(ex);
            if best_ex.as_ref().is_none_or(|x| ex > x.0) {
                let row = |r: usize| {
                    (0..c.cols)
                        .map(|k| format!("DR-{:03}={:>5.0}", idx[k], c.get(r, k) as f64 / 1000.0))
                        .collect::<Vec<_>>()
                        .join("  ")
                };
                let who = |p: &[(usize, usize)], r: usize| {
                    p.iter().find(|x| x.0 == r).map_or("-".into(), |x| {
                        format!(
                            "DR-{:03} ({:.0} s)",
                            idx[x.1],
                            c.get(r, x.1) as f64 / 1000.0
                        )
                    })
                };
                best_ex = Some((
                    ex,
                    format!(
                        "  khách A (đặt trước) ghim ({:.5}, {:.5}), khách B ghim ({:.5}, {:.5}), cách nhau {:.0} m chim bay\n    ETA khách A: {}\n    ETA khách B: {}\n    tham lam (A chọn trước): A ← {}, B ← {}; tổng {:.0} s\n    Hungarian (tối ưu cả lô): A ← {}, B ← {}; tổng {:.0} s\n    chênh {:.0} s ({:.1}%): A mất {:+.0} s, B được {:+.0} s",
                        a.lat(),
                        a.lng(),
                        b.lat(),
                        b.lng(),
                        haversine_m(a, b),
                        row(0),
                        row(1),
                        who(&g, 0),
                        who(&g, 1),
                        gc as f64 / 1000.0,
                        who(&h, 0),
                        who(&h, 1),
                        hc as f64 / 1000.0,
                        ex,
                        100.0 * ex / (hc as f64 / 1000.0),
                        h.iter()
                            .find(|x| x.0 == 0)
                            .map_or(0.0, |x| c.get(0, x.1) as f64 / 1000.0)
                            - g.iter()
                                .find(|x| x.0 == 0)
                                .map_or(0.0, |x| c.get(0, x.1) as f64 / 1000.0),
                        h.iter()
                            .find(|x| x.0 == 1)
                            .map_or(0.0, |x| c.get(1, x.1) as f64 / 1000.0)
                            - g.iter()
                                .find(|x| x.0 == 1)
                                .map_or(0.0, |x| c.get(1, x.1) as f64 / 1000.0),
                    ),
                ));
            }
        }
        let _ = (sa, sb);
    }
    excess.sort_by(f64::total_cmp);
    println!(
        "  tham lam theo thứ tự đặt tốn thêm ETA ở {sub}/{batches} lô ({:.1}%); phần tốn thêm trong các lô đó (s): p50 {:.1}  p95 {:.1}  max {:.1}",
        100.0 * sub as f64 / batches as f64,
        percentile(&excess, 50.0),
        percentile(&excess, 95.0),
        excess.last().copied().unwrap_or(0.0)
    );
    if let Some((_, text)) = best_ex {
        println!("  ví dụ cụ thể (lô tệ nhất trong mẫu):\n{text}");
    }

    // ---------- 3. baseline trên simulator, so có cặp ----------
    let n = n_seeds(&args);
    let skip = args.u64_or("--skip-seeds", 0) as usize;
    let seeds = eval_seeds(n + skip)[skip..].to_vec();
    println!(
        "\n-- 3. trong simulator: «gần nhất chim bay» (baseline P03) so với «gần nhất ETA» (ghép ngay), {n} seed đánh giá {}…{} --",
        seeds[0],
        seeds[n - 1]
    );
    let base_out = {
        let mut s = sc.clone();
        s.seed = seeds[0];
        run(
            &engine,
            &s,
            Box::new(AtPin),
            Box::new(NearestStraightLine),
            RunOptions { trace: false },
        )
    };
    let fnv = simulator::metrics::fnv1a(base_out.trips_csv().as_bytes());
    println!(
        "  kiểm baseline seed {}: nhật ký chuyến {} dòng, FNV-1a = {fnv:016x}{} (P03 công bố 100a862df76819aa cho kịch bản HCM gốc)",
        seeds[0],
        base_out.trips.len(),
        if !fixture && fnv == 0x100a_862d_f768_19aa {
            " → KHỚP từng byte"
        } else {
            ""
        }
    );
    let all_names: Vec<&str> = variants().iter().map(|v| v.0).collect();
    for (i, (vname, vdesc, tweak)) in select_variants(&args, &all_names).into_iter().enumerate() {
        if fixture && i > 1 {
            break;
        }
        let mut s = sc.clone();
        tweak(&mut s);
        println!("\n=== bộ giả định «{vname}»: {vdesc} ===");
        let a = sweep(&s, &seeds, &zones, jobs(&args), |sc| {
            run(
                &engine,
                sc,
                Box::new(AtPin),
                Box::new(NearestStraightLine),
                RunOptions { trace: true },
            )
        });
        let b = sweep(&s, &seeds, &zones, jobs(&args), |sc| {
            let (m, _) = BatchMatcher::new(MatcherConfig::nearest_eta());
            run(
                &engine,
                sc,
                Box::new(AtPin),
                Box::new(m),
                RunOptions { trace: true },
            )
        });
        println!(
            "  khách đặt trung bình/seed {}; baseline: hoàn thành {}, huỷ {}",
            f1(a.iter().map(|r| r.requests as f64).sum::<f64>() / n as f64),
            f1(a.iter().map(|r| r.completed as f64).sum::<f64>() / n as f64),
            f1(a.iter().map(|r| r.cancel_rate).sum::<f64>() / n as f64)
        );
        print_paired(
            "  so sánh có cặp:",
            "nearest-straight-line",
            "nearest-eta",
            &a,
            &b,
        );
        if i == 0 {
            print_zones("nearest-straight-line", "nearest-eta", &a, &b);
        }
    }
    println!(
        "\nGiả định: mọi số trên là trên dữ liệu mô phỏng (xem LOC-30). ETA của matcher là ETA CH theo nút (không có chi phí rẽ), simulator đo thời gian thật có chi phí rẽ và nhiễu tốc độ."
    );
    println!("uptime lúc kết thúc: {}", uptime());
}
