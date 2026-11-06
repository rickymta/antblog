//! LOC-30: bộ mô phỏng sự kiện rời rạc làm thước đo. Chạy baseline «đón đúng ghim, tài xế gần nhất theo chim bay»,
//! kiểm chạy lại ra đúng từng byte, và đo độ nhiễu của baseline giữa các seed.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc30_mo_phong
//! Tham số: --scenario FILE.toml (mặc định theo đồ thị), --fixture, --seeds N (số seed phụ, mặc định 4), --csv DIR (ghi nhật ký chuyến)

use geo_core::haversine_m;
use labs::p03::{mean, uptime, vn};
use labs::{Args, die, lab_root, load_graph};
use route_engine::engine::{Engine, EngineConfig};
use route_engine::profile::Profile;
use simulator::world::generate;
use simulator::{AtPin, NearestStraightLine, RunOptions, Scenario, SimOutput, run};
use std::time::Instant;

fn sim(engine: &Engine, sc: &Scenario, trace: bool) -> (SimOutput, f64) {
    let t = Instant::now();
    let out = run(
        engine,
        sc,
        Box::new(AtPin),
        Box::new(NearestStraightLine),
        RunOptions { trace },
    );
    (out, t.elapsed().as_secs_f64())
}

/// Đuôi `walk_m`: so đường đi bộ với chim bay từ chỗ đứng thật tới điểm đón, đúng phép tính của `AtPin`, cho mọi khách đặt.
fn walk_tail(engine: &Engine, sc: &Scenario) {
    let g = &engine.g;
    let mut rows = Vec::new();
    let mut same_edge = 0;
    for c in &generate(engine, sc).customers {
        let v = &c.view;
        let Ok(stop) = engine.snap_point(Profile::Car, v.pin, "ghim") else {
            continue;
        };
        let Ok(r) = engine.route(Profile::Foot, v.true_pos, stop.point, None) else {
            continue;
        };
        same_edge += usize::from(r.algo == "same-edge");
        let crow = haversine_m(v.true_pos, stop.point);
        let mut names: Vec<String> = Vec::new();
        for &e in &r.edges {
            let n = g.name[e as usize];
            if n != u32::MAX && names.last() != Some(&g.names[n as usize]) {
                names.push(g.names[n as usize].clone());
            }
        }
        rows.push((r.distance_m, crow, v.id, names));
    }
    if rows.is_empty() {
        return;
    }
    let pct = |mut x: Vec<f64>, p: f64| {
        x.sort_by(f64::total_cmp);
        x[((p / 100.0) * x.len() as f64).ceil().max(1.0) as usize - 1]
    };
    let walk: Vec<f64> = rows.iter().map(|r| r.0).collect();
    let crow: Vec<f64> = rows.iter().map(|r| r.1).collect();
    println!(
        "\n-- 6. đuôi walk_m: đường đi bộ so với chim bay, mọi khách đặt ({}) --",
        rows.len()
    );
    println!(
        "  đường đi bộ p50 {:.1}, p95 {:.1}, lớn nhất {:.1} m; chim bay chỗ đứng → điểm đón p50 {:.1}, p95 {:.1}, lớn nhất {:.1} m; hai đầu snap về cùng một cạnh đi bộ: {}/{}",
        pct(walk.clone(), 50.0),
        pct(walk.clone(), 95.0),
        pct(walk, 100.0),
        pct(crow.clone(), 50.0),
        pct(crow.clone(), 95.0),
        pct(crow, 100.0),
        same_edge,
        rows.len()
    );
    rows.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (w, c, id, names) in rows.iter().take(8) {
        println!(
            "  khách {id}: đi bộ {w:.1} m, chim bay {c:.1} m, qua: {}",
            names.join(" > ")
        );
    }
}

fn main() {
    let args = Args::from_env();
    let (g, source) = load_graph(&args);
    let fixture = args.flag("--fixture");
    println!("== LOC-30: mô phỏng sự kiện rời rạc, baseline ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let sc_path = args.value("--scenario").map_or_else(
        || {
            lab_root().join(if fixture {
                "fixtures/scenario-khu-mau.toml"
            } else {
                "fixtures/scenario-hcm-baseline.toml"
            })
        },
        std::path::PathBuf::from,
    );
    let sc = Scenario::load(&sc_path).unwrap_or_else(|e| die(&format!("{e}")));
    let want = if fixture { "fixture" } else { "hcm" };
    if sc.graph != want {
        die(&format!(
            "kịch bản dành cho đồ thị «{}» nhưng đang chạy trên «{want}»",
            sc.graph
        ));
    }
    println!(
        "đồ thị: {source}: {} nút, {} cạnh; kịch bản: {}",
        vn(g.node_count() as u64),
        vn(g.edge_count() as u64),
        sc_path
            .strip_prefix(lab_root())
            .unwrap_or(&sc_path)
            .display()
    );
    let engine = Engine::new(g, EngineConfig::default());

    println!(
        "\n-- 1. kịch bản --\n  tài xế {}, khách đặt trong {:.0} s với tốc độ cơ sở {:.2}/phút ({} khung giờ đổi tốc độ), {} điểm nóng, {:.0}% khách và {:.0}% tài xế đặt đều trên mạng đường",
        sc.drivers,
        sc.duration_s,
        sc.demand.requests_per_min,
        sc.demand.phases.len(),
        sc.demand.hotspots.len(),
        100.0 * sc.demand.background_share,
        100.0 * sc.driver.background_share
    );
    println!(
        "  quãng chim bay điểm đón → điểm trả {:.0}–{:.0} m; ghim lệch chỗ đứng thật σ = {:.0} m; đi bộ {:.1} m/s; kiên nhẫn {:.0} s",
        sc.demand.trip_min_m,
        sc.demand.trip_max_m,
        sc.customer.pin_sigma_m,
        sc.customer.walk_speed_mps,
        sc.customer.patience_s
    );

    println!(
        "\n-- 2. baseline: đón đúng ghim (khách đi bộ tới điểm ghim đã snap), tài xế rảnh gần nhất theo chim bay --"
    );
    let (out, secs) = sim(&engine, &sc, true);
    print!("{}", out.report());
    println!("thời gian chạy mô phỏng: {secs:.2} s (đo khi máy đang chịu tải, xem uptime)");
    println!("\n{}", out.assumptions());

    println!("\n-- 3. chạy lại cùng seed: có ra đúng từng byte không --");
    let (again, secs2) = sim(&engine, &sc, true);
    let (r1, r2) = (out.report(), again.report());
    let (c1, c2) = (out.trips_csv(), again.trips_csv());
    let (t1, t2) = (out.trace_csv(), again.trace_csv());
    println!(
        "  báo cáo giống từng byte: {}; nhật ký chuyến ({} byte) giống từng byte: {}; nhật ký sự kiện ({} byte, {} dòng) giống từng byte: {}; lần chạy lại {secs2:.2} s",
        r1 == r2,
        c1.len(),
        c1 == c2,
        t1.len(),
        out.trace.len(),
        t1 == t2
    );
    assert!(r1 == r2 && c1 == c2 && t1 == t2, "mô phỏng không tất định");
    let mut other = sc.clone();
    other.seed += 1;
    let (diff, _) = sim(&engine, &other, false);
    println!(
        "  đổi seed {} → {}: nhật ký chuyến khác ({}): {}",
        sc.seed,
        other.seed,
        diff.trips_csv().len(),
        diff.trips_csv() != c1
    );

    println!(
        "\n-- 4. mười dòng đầu của nhật ký sự kiện (t_s, loại, khách, tài xế, vĩ độ, kinh độ) --"
    );
    for l in t1.lines().take(11) {
        println!("  {l}");
    }

    let extra = args.u64_or("--seeds", 4) as usize;
    if extra > 0 {
        println!(
            "\n-- 5. độ nhiễu của baseline giữa các seed (cùng kịch bản, seed {}…{}) --",
            sc.seed,
            sc.seed + extra as u64
        );
        println!(
            "  {:>9} {:>8} {:>10} {:>10} {:>11} {:>11} {:>10} {:>10} {:>9} {:>8}",
            "seed",
            "khách",
            "hoàn thành",
            "cancel",
            "walk_m p50",
            "deadhead p50",
            "eta p50",
            "eta p95",
            "trip_m p50",
            "u_turns/chuyến"
        );
        let mut rows = Vec::new();
        for k in 0..=extra as u64 {
            let mut s2 = sc.clone();
            s2.seed = sc.seed + k;
            let o = if k == 0 {
                out_summary(&out)
            } else {
                out_summary(&sim(&engine, &s2, false).0)
            };
            println!(
                "  {:>9} {:>8} {:>10} {:>10.4} {:>11.1} {:>11.1} {:>10.1} {:>10.1} {:>9.1} {:>8.3}",
                s2.seed, o[0] as usize, o[1] as usize, o[2], o[3], o[4], o[5], o[6], o[7], o[8]
            );
            rows.push(o);
        }
        let col = |i: usize| rows.iter().map(|r| r[i]).collect::<Vec<_>>();
        let sd = |v: &[f64]| {
            let m = mean(v);
            (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len().max(2) - 1) as f64).sqrt()
        };
        println!(
            "  trung bình qua {} seed: cancel {:.4} (độ lệch chuẩn {:.4}); deadhead_m p50 {:.1} (± {:.1}); pickup_eta_s p50 {:.1} (± {:.1}); u_turns/chuyến {:.3} (± {:.3})",
            rows.len(),
            mean(&col(2)),
            sd(&col(2)),
            mean(&col(4)),
            sd(&col(4)),
            mean(&col(5)),
            sd(&col(5)),
            mean(&col(8)),
            sd(&col(8))
        );
        println!(
            "  → một cải thiện trong P04, P05 chỉ đáng tin nếu lớn hơn độ lệch chuẩn này trên cùng kịch bản (hoặc phải so cùng seed từng cặp)"
        );
    }

    if let Some(dir) = args.value("--csv") {
        let d = std::path::Path::new(dir);
        std::fs::create_dir_all(d).unwrap_or_else(|e| die(&format!("{e}")));
        std::fs::write(d.join("trips.csv"), &c1).expect("ghi");
        std::fs::write(d.join("events.csv"), &t1).expect("ghi");
        std::fs::write(d.join("summary.json"), out.summary_json()).expect("ghi");
        println!("\nđã ghi trips.csv, events.csv, summary.json vào {dir}");
    }
    walk_tail(&engine, &sc);

    println!("\nuptime lúc kết thúc: {}", uptime());
}

fn out_summary(o: &SimOutput) -> [f64; 9] {
    let s = &o.summary;
    [
        s.requests as f64,
        s.completed as f64,
        s.cancel_rate,
        s.walk_m.p50,
        s.deadhead_m.p50,
        s.pickup_eta_s.p50,
        s.pickup_eta_s.p95,
        s.trip_m.p50,
        s.u_turns_per_trip,
    ]
}
