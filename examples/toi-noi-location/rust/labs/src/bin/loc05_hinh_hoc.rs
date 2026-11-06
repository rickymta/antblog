//! LOC-05: chiếu điểm lên polyline, vị trí dọc tuyến, bearing, mã hóa polyline, xuất GeoJSON.
//!
//! Chạy từ gốc repo:
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc05_hinh_hoc
//! Ghi examples/toi-noi-location/results/geojson/loc05-hinh-hoc.geojson để mở trên geojson.io hoặc QGIS.

use geo::{Closest, ClosestPoint, Coord, LineString, Point};
use geo_core::geojson::{line_feature, point_feature, to_string};
use geo_core::polyline;
use geo_core::segment::polyline_length_m;
use geo_core::{
    LatLng, bearing_deg, bearing_diff_deg, destination, haversine_m, project_to_polyline,
};
use labs::Rand;

fn ll(lat: f64, lng: f64) -> LatLng {
    LatLng::try_new(lat, lng).expect("hợp lệ")
}

fn compass(deg: f64) -> &'static str {
    ["B", "ĐB", "Đ", "ĐN", "N", "TN", "T", "TB"][((deg + 22.5) / 45.0) as usize % 8]
}

fn main() {
    println!("== LOC-05: hình học trên polyline ==");
    println!(
        "Đường gấp khúc minh họa nối ba địa danh ở trung tâm TP.HCM bằng tọa độ gần đúng; KHÔNG phải tuyến đường thật.\n"
    );
    let route = [
        ll(10.7725, 106.6980), // gần chợ Bến Thành
        ll(10.7798, 106.6990), // gần Nhà thờ Đức Bà
        ll(10.7769, 106.6955), // gần Dinh Độc Lập
        ll(10.7712, 106.7012),
    ];

    println!("-- 1. độ dài và bearing từng đoạn --");
    for (i, w) in route.windows(2).enumerate() {
        let b = bearing_deg(w[0], w[1]);
        println!(
            "  đoạn {i}: dài {:>8.2} m, bearing {:>6.2}° ({})",
            haversine_m(w[0], w[1]),
            b,
            compass(b)
        );
    }
    println!("  tổng độ dài polyline: {:.2} m", polyline_length_m(&route));
    println!(
        "  bearing_diff_deg(350, 10) = {}, bearing_diff_deg(90, 270) = {}",
        bearing_diff_deg(350.0, 10.0),
        bearing_diff_deg(90.0, 270.0)
    );

    println!("\n-- 2. chiếu một điểm GPS lên polyline --");
    // Dựng điểm GPS cách đoạn 1 đúng 25 m về phía vuông góc, tại 40% đoạn.
    let seg1_len = haversine_m(route[1], route[2]);
    let b1 = bearing_deg(route[1], route[2]);
    let on_line = destination(route[1], b1, 0.40 * seg1_len);
    let gps = destination(on_line, (b1 + 90.0) % 360.0, 25.0);
    println!("  điểm trên đoạn 1 tại 40%: {on_line}");
    println!("  điểm GPS dựng cách đó 25 m vuông góc: {gps}");
    let proj = project_to_polyline(gps, &route).expect("polyline không rỗng");
    let expected_along = haversine_m(route[0], route[1]) + 0.40 * seg1_len;
    println!(
        "  kết quả: đoạn {}, t = {:.4}, dist = {:.3} m, along = {:.3} m",
        proj.segment, proj.t, proj.dist_m, proj.along_m
    );
    println!("  kỳ vọng:  đoạn 1, t = 0.4000, dist = 25.000 m, along = {expected_along:.3} m");
    println!("  chân đường vuông góc: {}", proj.point);

    println!("\n-- 3. chiếu bằng hệ độ (coi lat/lng như x,y phẳng) so với chiếu theo mét --");
    println!(
        "  crate geo (ClosestPoint) làm việc trong mặt phẳng của dữ liệu đầu vào; với độ lat/lng, 1° kinh độ ngắn hơn 1° vĩ độ nên điểm gần nhất bị lệch."
    );
    for (label, base) in [
        ("TP.HCM", ll(10.7725, 106.6980)),
        ("Hà Nội", ll(21.0285, 105.8542)),
    ] {
        let (mut max_off, mut sum_off, n) = (0f64, 0f64, 2000);
        let mut rng = Rand::new(7);
        for _ in 0..n {
            // Đoạn dài 200 m theo hướng ngẫu nhiên; điểm GPS cách trung điểm 30 m về hướng ngẫu nhiên.
            let a = base;
            let b = destination(a, rng.range(0.0, 360.0), 200.0);
            let mid = destination(a, bearing_deg(a, b), 100.0);
            let q = destination(mid, rng.range(0.0, 360.0), 30.0);
            let ours = project_to_polyline(q, &[a, b]).expect("hợp lệ").point;
            let line = LineString::new(vec![
                Coord {
                    x: a.lng(),
                    y: a.lat(),
                },
                Coord {
                    x: b.lng(),
                    y: b.lat(),
                },
            ]);
            let theirs = match line.closest_point(&Point::new(q.lng(), q.lat())) {
                Closest::SinglePoint(p) | Closest::Intersection(p) => ll(p.y(), p.x()),
                Closest::Indeterminate => continue,
            };
            let off = haversine_m(ours, theirs);
            max_off = max_off.max(off);
            sum_off += off;
        }
        println!(
            "  {label}: {n} đoạn 200 m hướng ngẫu nhiên, điểm GPS cách trung điểm 30 m: chân của geo::ClosestPoint (hệ độ) lệch chân theo mét trung bình {:.3} m, lớn nhất {:.3} m",
            sum_off / f64::from(n),
            max_off
        );
    }

    println!("\n-- 4. mã hóa polyline (Google Encoded Polyline) --");
    // Làm dày tuyến: một điểm mỗi ~5 m dọc tuyến, lệch ngang nhẹ có seed để giống GPS thật.
    let mut rng = Rand::new(20_261_106);
    let total = polyline_length_m(&route);
    let dense: Vec<LatLng> = (0..=(total / 5.0) as usize)
        .map(|i| {
            let p = geo_core::segment::point_at(&route, i as f64 * 5.0).expect("không rỗng");
            destination(p, rng.range(0.0, 360.0), rng.range(0.0, 1.0))
        })
        .collect();
    println!(
        "  tuyến dày: {} điểm, mỗi ~5 m, nhiễu 0–1 m (seed 20261106)",
        dense.len()
    );
    let small = &route[..];
    println!(
        "  ví dụ nhỏ (4 đỉnh): precision 5 = \"{}\"",
        polyline::encode(small, 5)
    );
    println!(
        "                       precision 6 = \"{}\"",
        polyline::encode(small, 6)
    );
    let json_bytes: usize = dense
        .iter()
        .map(|p| format!("[{},{}]", p.lng(), p.lat()).len() + 1)
        .sum();
    for prec in [5u32, 6] {
        let s = polyline::encode(&dense, prec);
        let back = polyline::decode(&s, prec).expect("giải mã được");
        let max_err = dense
            .iter()
            .zip(&back)
            .map(|(a, b)| haversine_m(*a, *b))
            .fold(0f64, f64::max);
        println!(
            "  precision {prec}: {:>6} byte chuỗi, sai số lớn nhất sau vòng mã hóa/giải mã {:.4} m",
            s.len(),
            max_err
        );
    }
    println!(
        "  so sánh kích thước: JSON [lng,lat] f64 ≈ {json_bytes} byte, nhị phân E7 = {} byte, f64 thô = {} byte",
        dense.len() * 8,
        dense.len() * 16
    );

    println!("\n-- 5. xuất GeoJSON --");
    let foot = proj.point;
    let features = vec![
        line_feature(&route, &[("vai_tro", "polyline minh hoa".into())]),
        line_feature(
            &[gps, foot],
            &[
                ("vai_tro", "doan chieu".into()),
                ("do_dai_m", ((proj.dist_m * 100.0).round() / 100.0).into()),
            ],
        ),
        point_feature(gps, &[("vai_tro", "diem GPS".into())]),
        point_feature(
            foot,
            &[
                ("vai_tro", "chan duong vuong goc".into()),
                ("along_m", ((proj.along_m * 100.0).round() / 100.0).into()),
            ],
        ),
    ];
    let json = to_string(features);
    let path = labs::results_dir().join("geojson/loc05-hinh-hoc.geojson");
    std::fs::create_dir_all(path.parent().expect("có thư mục cha")).expect("tạo thư mục");
    std::fs::write(&path, &json).expect("ghi geojson");
    println!(
        "  đã ghi {} ({} byte)",
        path.strip_prefix(labs::lab_root())
            .unwrap_or(&path)
            .display(),
        json.len()
    );
    println!("  nội dung: {json}");
}
