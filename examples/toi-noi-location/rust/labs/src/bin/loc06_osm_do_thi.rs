//! LOC-06: từ OpenStreetMap tới đồ thị đường bộ CSR (ô tô, xe máy, đi bộ).
//!
//! Chạy từ gốc repo (cần data/vietnam-latest.osm.pbf, xem data/fetch.sh):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc06_osm_do_thi
//! `--fixture` biên dịch fixtures/khu-mau.json thay cho PBF (không ghi data/hcm-graph.bin).

use labs::{Args, die, timed};
use road_graph::components::{component_sizes, strong_components};
use road_graph::{Graph, HCM_BBOX, HighwayClass, Mode, compile, io};

fn main() {
    let args = Args::from_env();
    let use_fixture = args.flag("--fixture");
    let pbf = args
        .value("--pbf")
        .map_or_else(labs::pbf_path, std::path::PathBuf::from);
    let out = args
        .value("--out")
        .map_or_else(labs::graph_bin_path, std::path::PathBuf::from);

    println!("== LOC-06: OSM -> đồ thị CSR ==");
    let (g, read_s) = if use_fixture {
        let f = road_graph::fixture::read_fixture_file(&labs::fixture_path())
            .unwrap_or_else(|e| die(&format!("không đọc được fixture: {e}")));
        timed(|| road_graph::fixture::compile_fixture(&f))
    } else {
        println!("nguồn: {}", pbf.display());
        println!(
            "bbox: lat {:.2}–{:.2}, lng {:.2}–{:.2}",
            HCM_BBOX[0], HCM_BBOX[1], HCM_BBOX[2], HCM_BBOX[3]
        );
        let (raw, s) = timed(|| {
            road_graph::pbf::read_pbf(&pbf, HCM_BBOX).unwrap_or_else(|e| die(&e.to_string()))
        });
        println!("đọc PBF (3 lượt, song song): {s:.2} s");
        let (g, c) = timed(|| compile(raw));
        println!("biên dịch (tách cạnh, liên thông, CSR, cấm rẽ): {c:.2} s");
        (g, s + c)
    };
    println!("tổng đọc + biên dịch: {read_s:.2} s");

    if !use_fixture {
        let (bytes, save_s) = timed(|| io::save(&g, &out).unwrap_or_else(|e| die(&e.to_string())));
        println!(
            "ghi {} : {:.1} MB trong {save_s:.2} s",
            out.display(),
            bytes as f64 / 1e6
        );
        let (loaded, load_s) = timed(|| io::load(&out).unwrap_or_else(|e| die(&e.to_string())));
        println!(
            "nạp lại file: {:.0} ms, {} nút, {} cạnh (khớp: {})",
            load_s * 1000.0,
            loaded.node_count(),
            loaded.edge_count(),
            loaded.head == g.head && loaded.length_cm == g.length_cm
        );
    }
    println!();
    report(&g);
}

fn pct(a: u64, b: u64) -> String {
    if b == 0 {
        "n/a".into()
    } else {
        format!("{:.1}%", 100.0 * a as f64 / b as f64)
    }
}

fn report(g: &Graph) {
    let c = &g.meta.counters;
    let get = |k: &str| c.get(k).copied().unwrap_or(0);

    println!("-- thời gian từng bước --");
    for (name, s) in &g.meta.timings_s {
        println!("  {name}: {s:.2} s");
    }

    println!("-- đầu vào đọc từ PBF --");
    for k in c.keys().filter(|k| k.starts_with("pbf.")) {
        println!("  {k} = {}", c[k]);
    }

    println!("-- đồ thị --");
    println!("  nút = {}", g.node_count());
    println!("  cạnh có hướng = {}", g.edge_count());
    println!(
        "  cạnh có twin (đường hai chiều) = {} ({})",
        get("graph.edges_with_twin"),
        pct(get("graph.edges_with_twin"), g.edge_count() as u64)
    );
    println!("  điểm hình dạng = {}", g.shape.len());
    println!(
        "  way highway ứng viên = {}, mô hình hóa = {}, bị bbox cắt = {}",
        get("ways.highway_candidates"),
        get("ways.modeled"),
        get("ways.clipped_by_bbox")
    );
    let mem = g.node_ll.len() * 8
        + g.node_osm.len() * 8
        + g.first_out.len() * 4
        + g.edge_count() * (4 + 4 + 4 + 1 + 1 + 2 + 1 + 1 + 2 + 1 + 4 + 8 + 4 + 4 + 2)
        + g.shape.len() * 8;
    println!(
        "  bộ nhớ các cột (ước tính từ kích thước kiểu) = {:.1} MB",
        mem as f64 / 1e6
    );

    println!("-- thành phần liên thông (vô hướng, trước khi lọc) --");
    println!("  số thành phần = {}", get("components.count"));
    for i in 1..=5 {
        if let Some(s) = c.get(&format!("components.size_rank{i}")) {
            println!("  thành phần hạng {i}: {s} nút");
        }
    }
    println!(
        "  nút bị loại (ngoài thành phần lớn nhất) = {} trên {} ({})",
        get("components.nodes_dropped"),
        get("graph.nodes_before_component_filter"),
        pct(
            get("components.nodes_dropped"),
            get("graph.nodes_before_component_filter")
        )
    );

    println!("-- theo profile (đồ thị đã lọc) --");
    println!(
        "  {:<8} {:>10} {:>9} {:>12} {:>10} {:>9}",
        "profile", "cạnh", "% cạnh", "SCC lớn nhất", "% nút", "số SCC"
    );
    for mode in Mode::ALL {
        let ecount = g.mode_edge_count(mode) as u64;
        let scc = strong_components(g, mode);
        let sizes = component_sizes(&scc);
        let max = sizes.iter().copied().max().unwrap_or(0) as u64;
        let nontrivial = sizes.iter().filter(|&&s| s > 1).count();
        println!(
            "  {:<8} {:>10} {:>9} {:>12} {:>10} {:>9}",
            mode.name(),
            ecount,
            pct(ecount, g.edge_count() as u64),
            max,
            pct(max, g.node_count() as u64),
            nontrivial
        );
    }
    let mixed = g.modes.iter().filter(|&&m| m != 7).count();
    println!(
        "  cạnh không đủ cả ba profile = {} ({})",
        mixed,
        pct(mixed as u64, g.edge_count() as u64)
    );
    let bike_not_car = g
        .modes
        .iter()
        .filter(|&&m| m & Mode::MOTORBIKE_BIT != 0 && m & Mode::CAR_BIT == 0)
        .count();
    println!(
        "  cạnh xe máy đi được mà ô tô không = {} ({})",
        bike_not_car,
        pct(bike_not_car as u64, g.edge_count() as u64)
    );

    println!("-- độ dài cạnh (mét) --");
    let mut lens: Vec<f64> = g.length_cm.iter().map(|&l| f64::from(l) / 100.0).collect();
    lens.sort_by(f64::total_cmp);
    println!(
        "  nhỏ nhất {:.2}, p50 {:.1}, p90 {:.1}, p99 {:.1}, lớn nhất {:.1}, tổng {:.0} km (hai chiều tính riêng)",
        lens[0],
        labs::percentile(&lens, 50.0),
        labs::percentile(&lens, 90.0),
        labs::percentile(&lens, 99.0),
        lens[lens.len() - 1],
        lens.iter().sum::<f64>() / 1000.0
    );
    println!(
        "  cạnh ngắn hơn 1 m = {}",
        lens.iter().filter(|&&l| l < 1.0).count()
    );

    println!("-- chất lượng thẻ (đếm theo way đã mô hình hóa) --");
    for class in HighwayClass::ALL {
        let n = get(&format!("quality.class.{}", class.osm_value()));
        if n > 0 {
            println!("  highway={}: {n}", class.osm_value());
        }
    }
    let road = get("quality.motor_roads.ways");
    println!("  đường cơ giới (motorway…track) = {road}");
    for (label, key) in [
        ("có name", "quality.motor_roads.with_name"),
        ("có maxspeed", "quality.motor_roads.with_maxspeed"),
        ("có lanes", "quality.motor_roads.with_lanes"),
        ("có width", "quality.motor_roads.with_width"),
        ("có thẻ oneway", "quality.motor_roads.with_oneway_tag"),
    ] {
        println!("    {label}: {} ({})", get(key), pct(get(key), road));
    }
    println!(
        "  hẻm (service=alley) = {}, trong đó có width = {} ({})",
        get("quality.alley.ways"),
        get("quality.alley.with_width"),
        pct(get("quality.alley.with_width"), get("quality.alley.ways"))
    );
    println!(
        "  highway=service = {}, trong đó có thẻ service=* = {} ({})",
        get("quality.service.ways"),
        get("quality.service.with_service_tag"),
        pct(
            get("quality.service.with_service_tag"),
            get("quality.service.ways")
        )
    );
    println!(
        "  cầu (bridge) = {}, hầm (tunnel) = {}",
        get("quality.bridge.ways"),
        get("quality.tunnel.ways")
    );
    println!(
        "  một chiều: ô tô {}, xe máy {}, khác nhau giữa hai loại {}",
        get("quality.oneway_car.ways"),
        get("quality.oneway_motorbike.ways"),
        get("quality.oneway_differs_car_vs_motorbike.ways")
    );
    for (k, v) in c.iter().filter(|(k, _)| k.starts_with("tag.")) {
        println!("  {k}: {v}");
    }
    for (k, v) in c.iter().filter(|(k, _)| k.starts_with("skip.")) {
        println!("  bỏ qua {k}: {v}");
    }

    println!("-- cấm rẽ và cổng (giữ thô cho LOC-23, LOC-37) --");
    for (k, v) in c.iter().filter(|(k, _)| k.starts_with("restrictions.")) {
        println!("  {k} = {v}");
    }
    println!("  restrictions đã lưu = {}", g.restrictions.len());
    for (k, v) in c.iter().filter(|(k, _)| k.starts_with("entrances.")) {
        println!("  {k} = {v}");
    }
}
