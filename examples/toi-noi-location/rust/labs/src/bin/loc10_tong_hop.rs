//! LOC-10: tổng hợp P01. Đồ thị tổng hợp «khu mẫu» cho CI, quét thuộc tính ngẫu nhiên ra con số, những gì chưa có.
//!
//! Không cần PBF hay Redis. Chạy từ gốc repo:
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc10_tong_hop
//! Bộ kiểm thử đầy đủ: cargo test --manifest-path examples/toi-noi-location/rust/Cargo.toml --workspace

use geo_core::geojson::{line_feature, point_feature, to_string};
use geo_core::{
    LatLng, bearing_deg, bearing_diff_deg, destination, equirect_m, haversine_m, polyline,
    project_to_segment,
};
use labs::Rand;
use road_graph::components::{component_sizes, strong_components};
use road_graph::fixture::{generate, read_fixture_file, to_json};
use road_graph::snap::SnapIndex;
use road_graph::{Mode, flag, sp};

const SEED: u64 = 20_261_106;

fn ll(lat: f64, lng: f64) -> LatLng {
    LatLng::try_new(lat, lng).expect("hợp lệ")
}

fn main() {
    println!("== LOC-10: tổng hợp P01 ==\n");
    fixture_section();
    property_sweeps();
    println!("\n-- 5. những gì P01 chưa có --");
    for item in [
        "định tuyến (Dijkstra tối giản trong road_graph::sp chỉ để đo hệ số vòng; không A*, không CH, không chi phí rẽ): LOC-21 trở đi",
        "cấm rẽ mới được đọc và giữ thô (270 relation via-way chưa giải); chưa dùng khi tìm đường: LOC-23",
        "nút `entrance=*` mới lưu thô, chưa gắn vào đồ thị: LOC-37",
        "map matching, Kalman, tốc độ theo giờ: P02, P03",
        "VN-2000 chưa hỗ trợ; chỉ WGS84",
        "Redis GEO chạy qua Docker loopback; PostGIS không chạy",
        "chưa có hệ thống tọa độ cho tòa nhà trong nhà (GPS vô dụng): LOC-37",
    ] {
        println!("  - {item}");
    }
}

fn fixture_section() {
    println!("-- 1. đồ thị tổng hợp fixtures/khu-mau.json --");
    let path = labs::fixture_path();
    let on_disk = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| labs::die(&format!("thiếu fixture: {e}")));
    let regenerated = to_json(&generate());
    println!(
        "  file {} byte; sinh lại từ seed {} cho {} byte; trùng từng byte: {}",
        on_disk.len(),
        road_graph::fixture::FIXTURE_SEED,
        regenerated.len(),
        on_disk == regenerated
    );
    let file = read_fixture_file(&path).expect("đọc fixture");
    let g = road_graph::fixture::compile_fixture(&file);
    println!(
        "  nút trong file OSM-mini: {}, way: {}, quan hệ cấm rẽ: {}, cổng: {}",
        file.nodes.len(),
        file.ways.len(),
        file.restrictions.len(),
        file.entrances.len()
    );
    println!(
        "  đồ thị CSR: {} nút (nút giao), {} cạnh có hướng, {} điểm hình dạng; thành phần liên thông trước lọc: {}",
        g.node_count(),
        g.edge_count(),
        g.shape.len(),
        g.meta.counters["components.count"]
    );
    println!(
        "  {:<8} {:>8} {:>10} {:>14}",
        "profile", "cạnh", "SCC lớn", "SCC lớn / nút"
    );
    for mode in Mode::ALL {
        let sizes = component_sizes(&strong_components(&g, mode));
        let max = sizes.iter().copied().max().unwrap_or(0);
        println!(
            "  {:<8} {:>8} {:>10} {:>13.1}%",
            mode.name(),
            g.mode_edge_count(mode),
            max,
            100.0 * f64::from(max) / g.node_count() as f64
        );
    }
    println!(
        "  restriction: {:?}",
        g.restrictions
            .iter()
            .map(|r| (
                r.osm_id,
                r.kind.as_str(),
                r.from_edges.len(),
                r.to_edges.len()
            ))
            .collect::<Vec<_>>()
    );
    println!(
        "  entrance: {:?}",
        g.entrances
            .iter()
            .map(|e| (e.osm_id, e.kind.as_str()))
            .collect::<Vec<_>>()
    );

    println!("\n-- 2. các tình huống mà fixture phải chứa --");
    let has = |f: &dyn Fn(road_graph::EdgeView) -> bool| g.edges().filter(|e| f(*e)).count();
    println!(
        "  cạnh cầu (bridge) cho ô tô: {}, cầu bộ hành (không cho ô tô): {}",
        has(&|e| e.has(flag::BRIDGE) && e.allows(Mode::Car)),
        has(&|e| e.has(flag::BRIDGE) && !e.allows(Mode::Car))
    );
    println!(
        "  hẻm xe máy và đi bộ vào được, ô tô không vào cả hai chiều: {} cạnh (hẻm width=2.2 và hẻm motor_vehicle=no)",
        has(&|e| !e.allows(Mode::Car)
            && e.allows(Mode::Motorbike)
            && e.twin().is_none_or(|t| !g.edge(t).allows(Mode::Car)))
    );
    println!(
        "  hẻm cụt ô tô vào được: {} cạnh",
        has(&|e| e.has(flag::ALLEY) && e.allows(Mode::Car))
    );
    println!(
        "  đường một chiều cho ô tô: {} cạnh; xe máy đi được ngược chiều ô tô: {} cạnh",
        has(&|e| e.has(flag::ONEWAY_CAR) && e.allows(Mode::Car)),
        has(&|e| e.has(flag::ONEWAY_CAR) && !e.allows(Mode::Car) && e.allows(Mode::Motorbike))
    );
    let bridge = g
        .edges()
        .find(|e| e.has(flag::BRIDGE) && e.allows(Mode::Car))
        .expect("có cầu vượt");
    let avenue_touch = g
        .edges()
        .filter(|e| {
            e.name().is_some_and(|n| n.starts_with("Đại lộ"))
                && [e.tail(), e.head()]
                    .iter()
                    .any(|&n| n == bridge.tail() || n == bridge.head())
        })
        .count();
    println!(
        "  cầu vượt «{}» dài {} m, số cạnh đại lộ chạm đầu cầu: {} (cầu cắt ngang đại lộ mà không có nút chung)",
        bridge.name().unwrap_or(""),
        bridge.length_m(),
        avenue_touch
    );

    println!("\n-- 3. ghim sai phía đại lộ có dải phân cách --");
    let pois = &file.pois;
    let get = |n: &str| {
        pois.iter()
            .find(|p| p.name == n)
            .map(|p| ll(p.lat, p.lng))
            .expect("poi")
    };
    let (north, south) = (get("khach_phia_bac_dai_lo"), get("khach_phia_nam_dai_lo"));
    let idx = SnapIndex::build(&g);
    let sn = idx.nearest(&g, north, Mode::Car).expect("snap");
    let ss = idx.nearest(&g, south, Mode::Car).expect("snap");
    let describe = |label: &str, c: &road_graph::snap::SnapCandidate| {
        let e = g.edge(c.edge);
        println!(
            "  {label}: gần nhất «{}» cách {:.1} m, bearing {:.0}°",
            e.name().unwrap_or("?"),
            c.dist_m,
            c.bearing_deg
        );
    };
    describe("ghim phía bắc", &sn);
    describe("ghim phía nam", &ss);
    let straight = haversine_m(north, south);
    let from = g.edge(sn.edge).head();
    let to = g.edge(ss.edge).head();
    let road = sp::shortest_cm(&g, Mode::Car, from, to).map(|c| f64::from(c) / 100.0);
    let road_back = sp::shortest_cm(&g, Mode::Car, to, from).map(|c| f64::from(c) / 100.0);
    println!("  chim bay giữa hai ghim: {straight:.1} m");
    println!(
        "  ô tô, giữa hai nút đầu cạnh snap (xấp xỉ, nút cuối cạnh): {from} → {to}: {road:?} m; chiều ngược lại: {road_back:?} m"
    );
    let walk = sp::shortest_cm(&g, Mode::Foot, from, to).map(|c| f64::from(c) / 100.0);
    println!("  đi bộ cùng cặp nút (nút bắc → nút nam): {walk:?} m");

    let fc = to_string(vec![
        point_feature(north, &[("vai_tro", "ghim phia bac".into())]),
        point_feature(south, &[("vai_tro", "ghim phia nam".into())]),
        line_feature(
            &g.edge_points(sn.edge),
            &[("vai_tro", "canh snap phia bac".into())],
        ),
        line_feature(
            &g.edge_points(ss.edge),
            &[("vai_tro", "canh snap phia nam".into())],
        ),
    ]);
    let out = labs::results_dir().join("geojson/loc10-khu-mau-dai-lo.geojson");
    std::fs::create_dir_all(out.parent().expect("có thư mục cha")).expect("tạo thư mục");
    std::fs::write(&out, &fc).expect("ghi");
    println!(
        "  đã ghi results/geojson/loc10-khu-mau-dai-lo.geojson ({} byte)",
        fc.len()
    );
}

fn property_sweeps() {
    println!(
        "\n-- 4. quét thuộc tính ngẫu nhiên (cùng tính chất với proptest, nhưng in ra con số lớn nhất), seed {SEED} --"
    );
    let mut rng = Rand::new(SEED);
    let n = 200_000;
    let vn = |rng: &mut Rand| ll(rng.range(8.0, 24.0), rng.range(102.0, 110.0));
    let hcm = |rng: &mut Rand| ll(rng.range(10.70, 10.86), rng.range(106.60, 106.78));

    let (mut sym_h, mut sym_e, mut tri_h, mut tri_e_abs, mut tri_e_rel) =
        (0f64, 0f64, f64::MIN, f64::MIN, f64::MIN);
    for _ in 0..n {
        let (a, b, c) = (vn(&mut rng), vn(&mut rng), vn(&mut rng));
        sym_h = sym_h.max((haversine_m(a, b) - haversine_m(b, a)).abs());
        tri_h = tri_h.max(haversine_m(a, c) - (haversine_m(a, b) + haversine_m(b, c)));
        let (a, b, c) = (hcm(&mut rng), hcm(&mut rng), hcm(&mut rng));
        sym_e = sym_e.max((equirect_m(a, b) - equirect_m(b, a)).abs());
        let excess = equirect_m(a, c) - (equirect_m(a, b) + equirect_m(b, c));
        tri_e_abs = tri_e_abs.max(excess);
        tri_e_rel = tri_e_rel.max(excess / equirect_m(a, c).max(1.0));
    }
    println!("  {n} bộ ba điểm:");
    println!(
        "    đối xứng: haversine (cả Việt Nam) lệch tối đa {sym_h:.2e} m; equirect (bbox HCM) lệch tối đa {sym_e:.2e} m"
    );
    println!(
        "    tam giác: d(a,c) - (d(a,b) + d(b,c)) lớn nhất: haversine {tri_h:.4} m (≤ 0 nghĩa là luôn thỏa); equirect {tri_e_abs:.4} m ({:.2e} của d(a,c))",
        tri_e_rel
    );

    let (mut e5, mut e6, mut e7) = (0f64, 0f64, 0f64);
    for _ in 0..2000 {
        let len = 1 + rng.usize(60);
        let pts: Vec<LatLng> = (0..len).map(|_| hcm(&mut rng)).collect();
        for (prec, slot) in [(5u32, &mut e5), (6, &mut e6)] {
            let back = polyline::decode(&polyline::encode(&pts, prec), prec).expect("giải mã");
            for (p, q) in pts.iter().zip(&back) {
                *slot = slot.max(haversine_m(*p, *q));
            }
        }
        for p in &pts {
            e7 = e7.max(haversine_m(*p, p.to_e7().to_latlng()));
        }
    }
    println!(
        "  mã hóa polyline, 2000 tuyến ngẫu nhiên 1–60 điểm: sai số lớn nhất precision 5 = {e5:.4} m, precision 6 = {e6:.4} m; lượng tử hóa E7 = {e7:.4} m"
    );

    let (mut worst_foot, mut t_out, mut farther_than_end) = (0f64, 0usize, 0usize);
    for _ in 0..n {
        let a = hcm(&mut rng);
        let b = destination(a, rng.range(0.0, 360.0), rng.range(1.0, 2000.0));
        let p = destination(a, rng.range(0.0, 360.0), rng.range(0.0, 1500.0));
        let r = project_to_segment(p, a, b);
        let seg = haversine_m(a, b);
        worst_foot =
            worst_foot.max((haversine_m(a, r.point) + haversine_m(r.point, b) - seg).abs());
        if !(0.0..=1.0).contains(&r.t) {
            t_out += 1;
        }
        if r.dist_m > haversine_m(p, a).min(haversine_m(p, b)) + 0.01 + 0.001 * r.dist_m {
            farther_than_end += 1;
        }
    }
    println!(
        "  chiếu lên đoạn, {n} ca (đoạn 1–2000 m, điểm cách gốc đoạn tối đa 1500 m): t ngoài [0,1]: {t_out}; chân lệch khỏi đoạn tối đa {worst_foot:.4} m; chân xa hơn hai đầu mút: {farther_than_end}"
    );

    let mut worst_bearing = 0f64;
    for _ in 0..n {
        let a = hcm(&mut rng);
        let brg = rng.range(0.0, 360.0);
        let b = destination(a, brg, rng.range(100.0, 5000.0));
        worst_bearing = worst_bearing.max(bearing_diff_deg(bearing_deg(a, b), brg));
    }
    println!(
        "  bearing: destination rồi bearing_deg lệch hướng ban đầu tối đa {worst_bearing:.4}° ({n} ca, 100–5000 m)"
    );
}
