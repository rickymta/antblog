//! LOC-08: H3 và geohash. Lỗi biên ô của geohash, k-ring của H3, độ phân giải so với độ dài cạnh.
//!
//! Chạy từ gốc repo:
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc08_h3
//! Ghi results/geojson/loc08-*.geojson (ô geohash ở biên, k-ring H3) để vẽ hình từ output thật.

use geo_core::geojson::{point_feature, polygon_feature, to_string};
use geo_core::h3::{self, cell_boundary, cell_center, cell_of, k_ring, resolution};
use geo_core::{LatLng, destination, geohash, haversine_m};
use labs::Rand;
use std::hint::black_box;
use std::time::Instant;

const SEED: u64 = 20_261_106;
const TRIALS: usize = 200_000;

fn ll(lat: f64, lng: f64) -> LatLng {
    LatLng::try_new(lat, lng).expect("hợp lệ")
}

fn main() {
    println!("== LOC-08: lưới H3 và geohash ==");
    println!(
        "seed {SEED}, {TRIALS} mẫu mỗi ô trong bảng, điểm hỏi ngẫu nhiên trong lat 10.73–10.83, lng 106.63–106.75\n"
    );
    resolution_table();
    geohash_boundary_example();
    coverage_tables();
    cost();
}

fn resolution_table() {
    println!("-- 1. độ phân giải H3 và độ dài cạnh (quanh Bến Thành, lat 10.7725, lng 106.698) --");
    let c = ll(10.7725, 106.6980);
    let bbox_w = haversine_m(ll(10.78, 106.60), ll(10.78, 106.78));
    let bbox_h = haversine_m(ll(10.70, 106.69), ll(10.86, 106.69));
    println!(
        "bbox lab: {:.1} km × {:.1} km = {:.1} km²",
        bbox_w / 1000.0,
        bbox_h / 1000.0,
        bbox_w * bbox_h / 1e6
    );
    println!(
        "{:>3} {:>16} {:>22} {:>16} {:>22}",
        "res",
        "cạnh TB (m) [H3]",
        "cạnh đo tại HCM (m)",
        "diện tích TB km²",
        "số ô phủ bbox (ước tính)"
    );
    for r in 5..=12u8 {
        let res = resolution(r);
        let cell = cell_of(c, res);
        let b = cell_boundary(cell);
        let n = b.len();
        let mean_edge = (0..n)
            .map(|i| haversine_m(b[i], b[(i + 1) % n]))
            .sum::<f64>()
            / n as f64;
        let area = h3::area_avg_m2(res);
        println!(
            "{r:>3} {:>16.1} {:>22.1} {:>16.4} {:>22.0}",
            h3::edge_length_avg_m(res),
            mean_edge,
            area / 1e6,
            bbox_w * bbox_h / area
        );
    }
    println!(
        "(số ô phủ bbox = diện tích bbox / diện tích ô TB, chỉ là ước tính; không đếm ô thật)"
    );
    println!("\n-- geohash: kích thước ô tại cùng điểm --");
    println!("{:>9} {:>14} {:>14}", "độ dài", "rộng (m)", "cao (m)");
    for p in 4..=9usize {
        let bx = geohash::decode_box(&geohash::encode(c, p)).expect("hợp lệ");
        let w = haversine_m(ll(c.lat(), bx.lng_min), ll(c.lat(), bx.lng_max));
        let h = haversine_m(ll(bx.lat_min, c.lng()), ll(bx.lat_max, c.lng()));
        println!("{p:>9} {w:>14.1} {h:>14.1}");
    }
    println!();
}

/// Quét hai đường (một theo kinh độ, một theo vĩ độ) trong bbox, bước ~5,5 m, tìm cặp điểm liền kề khác ô geohash độ dài 8
/// với tiền tố chung ngắn nhất.
fn geohash_boundary_example() {
    println!("-- 2. lỗi biên ô geohash: hai điểm cách nhau vài mét nhưng khác ô --");
    let step = 0.00005; // độ, ~5,5 m
    let mut worst: Option<(usize, LatLng, LatLng)> = None;
    let mut shortest_prefix_counts = [0usize; 9];
    let mut scan = |a: LatLng, b: LatLng| {
        let (ha, hb) = (geohash::encode(a, 8), geohash::encode(b, 8));
        if ha != hb {
            let common = ha
                .bytes()
                .zip(hb.bytes())
                .take_while(|(x, y)| x == y)
                .count();
            shortest_prefix_counts[common] += 1;
            if worst.as_ref().is_none_or(|w| common < w.0) {
                worst = Some((common, a, b));
            }
        }
    };
    let mut lng = 106.60;
    while lng < 106.78 {
        scan(ll(10.78, lng), ll(10.78, lng + step));
        lng += step;
    }
    let mut lat = 10.70;
    while lat < 10.86 {
        scan(ll(lat, 106.69), ll(lat + step, 106.69));
        lat += step;
    }
    println!(
        "quét đường lat = 10.78 (lng 106.60→106.78) và đường lng = 106.69 (lat 10.70→10.86), bước {step}° (~5,5 m), geohash độ dài 8"
    );
    println!("số lần hai điểm liền kề khác ô, theo độ dài tiền tố chung:");
    for (len, n) in shortest_prefix_counts.iter().enumerate() {
        if *n > 0 {
            println!("  tiền tố chung {len} ký tự: {n} lần");
        }
    }
    let (common, a, b) = worst.expect("có ít nhất một biên ô");
    let (ha, hb) = (geohash::encode(a, 8), geohash::encode(b, 8));
    println!("cặp tệ nhất (tiền tố chung ngắn nhất = {common} ký tự):");
    println!("  A = {a}  geohash {ha}");
    println!("  B = {b}  geohash {hb}");
    println!("  khoảng cách A–B = {:.2} m", haversine_m(a, b));
    for p in [5usize, 6, 7] {
        let (pa, pb) = (geohash::encode(a, p), geohash::encode(b, p));
        let nb = geohash::neighbors(&pa);
        println!(
            "  độ dài {p}: A = {pa}, B = {pb}; cùng ô: {}; B nằm trong 8 ô láng giềng của A: {}",
            pa == pb,
            nb.contains(&pb)
        );
    }
    println!(
        "  → tìm «cùng tiền tố geohash» bỏ sót B dù chỉ cách {:.1} m; phải quét cả 8 ô láng giềng.",
        haversine_m(a, b)
    );

    // H3 cho cùng cặp điểm: ô khác nhau nhưng luôn là láng giềng.
    for r in [8u8, 9, 10] {
        let (ca, cb) = (cell_of(a, resolution(r)), cell_of(b, resolution(r)));
        println!(
            "  H3 res {r}: cùng ô: {}; B trong k-ring(1) của A: {}",
            ca == cb,
            k_ring(ca, 1).contains(&cb)
        );
    }

    let mut feats = Vec::new();
    for (name, p) in [("A", a), ("B", b)] {
        let hash = geohash::encode(p, 6);
        let bx = geohash::decode_box(&hash).expect("hợp lệ");
        let ring = [
            ll(bx.lat_min, bx.lng_min),
            ll(bx.lat_min, bx.lng_max),
            ll(bx.lat_max, bx.lng_max),
            ll(bx.lat_max, bx.lng_min),
        ];
        feats.push(polygon_feature(
            &ring,
            &[
                ("vai_tro", "o geohash 6".into()),
                ("diem", name.into()),
                ("geohash", hash.into()),
            ],
        ));
        feats.push(point_feature(
            p,
            &[("vai_tro", format!("diem {name}").into())],
        ));
    }
    let path = labs::results_dir().join("geojson/loc08-geohash-bien.geojson");
    std::fs::create_dir_all(path.parent().expect("có thư mục cha")).expect("tạo thư mục");
    std::fs::write(&path, to_string(feats)).expect("ghi geojson");
    println!("  đã ghi results/geojson/loc08-geohash-bien.geojson (hai ô geohash 6 và hai điểm)\n");
}

fn coverage_tables() {
    let mut rng = Rand::new(SEED);
    let radii = [100.0, 300.0, 1000.0];

    println!(
        "-- 3. tỉ lệ bỏ sót: đích cách điểm hỏi khoảng cách r (đều trong [0, r], hướng ngẫu nhiên), cần nằm trong tập ô được quét --"
    );
    println!("geohash: «1 ô» = chỉ ô chứa điểm hỏi; «9 ô» = ô đó và 8 láng giềng");
    println!(
        "{:>10} {:>8} {:>10} {:>10}",
        "geohash", "r (m)", "bỏ sót 1 ô", "bỏ sót 9 ô"
    );
    let sample = |rng: &mut Rand, r: f64| {
        let q = ll(rng.range(10.73, 10.83), rng.range(106.63, 106.75));
        let t = destination(q, rng.range(0.0, 360.0), rng.range(0.0, r));
        (q, t)
    };
    for p in [5usize, 6, 7] {
        for &r in &radii {
            let (mut miss1, mut miss9) = (0usize, 0usize);
            for _ in 0..TRIALS {
                let (q, t) = sample(&mut rng, r);
                let (hq, ht) = (geohash::encode(q, p), geohash::encode(t, p));
                if hq != ht {
                    miss1 += 1;
                    if !geohash::neighbors(&hq).contains(&ht) {
                        miss9 += 1;
                    }
                }
            }
            println!(
                "{:>10} {r:>8.0} {:>9.2}% {:>9.2}%",
                format!("độ dài {p}"),
                100.0 * miss1 as f64 / TRIALS as f64,
                100.0 * miss9 as f64 / TRIALS as f64
            );
        }
    }

    println!(
        "\nH3: k = 0 là ô chứa điểm hỏi, k = 1 là 7 ô, k = 2 là 19 ô. «k nhỏ nhất» = k nhỏ nhất cho 0 bỏ sót trong {TRIALS} mẫu (tối đa thử 8)"
    );
    println!(
        "«phủ/tròn» = tổng diện tích các ô quét / diện tích hình tròn bán kính r (càng gần 1 càng ít quét thừa)"
    );
    println!(
        "{:>8} {:>8} {:>10} {:>10} {:>10} {:>11} {:>8} {:>10}",
        "H3 res",
        "r (m)",
        "bỏ sót k=0",
        "bỏ sót k=1",
        "bỏ sót k=2",
        "k nhỏ nhất",
        "số ô",
        "phủ/tròn"
    );
    for r_res in [7u8, 8, 9, 10] {
        let res = resolution(r_res);
        for &r in &radii {
            let mut miss = [0usize; 9];
            for _ in 0..TRIALS {
                let (q, t) = sample(&mut rng, r);
                let (cq, ct) = (cell_of(q, res), cell_of(t, res));
                // khoảng cách lưới giữa hai ô: ô nhỏ nhất k mà ct nằm trong k-ring(cq)
                let need = cq.grid_distance(ct).map_or(usize::MAX, |d| d as usize);
                for (k, m) in miss.iter_mut().enumerate() {
                    if need > k {
                        *m += 1;
                    }
                }
            }
            let kmin = miss
                .iter()
                .position(|&m| m == 0)
                .map_or_else(|| ">8".to_string(), |k| k.to_string());
            let cells = |k: usize| 3 * k * k + 3 * k + 1;
            let ratio = kmin
                .parse::<usize>()
                .ok()
                .map(|k| cells(k) as f64 * h3::area_avg_m2(res) / (std::f64::consts::PI * r * r));
            println!(
                "{r_res:>8} {r:>8.0} {:>9.2}% {:>9.2}% {:>9.2}% {kmin:>11} {:>8} {:>10}",
                100.0 * miss[0] as f64 / TRIALS as f64,
                100.0 * miss[1] as f64 / TRIALS as f64,
                100.0 * miss[2] as f64 / TRIALS as f64,
                kmin.parse::<usize>()
                    .map_or("-".to_string(), |k| cells(k).to_string()),
                ratio.map_or("-".to_string(), |x| format!("{x:.1}"))
            );
        }
    }

    // GeoJSON cho k-ring minh họa quanh Bến Thành.
    let center = ll(10.7725, 106.6980);
    let cell = cell_of(center, resolution(9));
    let mut feats = vec![point_feature(center, &[("vai_tro", "diem hoi".into())])];
    for c in k_ring(cell, 2) {
        let d = cell.grid_distance(c).unwrap_or(-1);
        feats.push(polygon_feature(
            &cell_boundary(c),
            &[
                ("vai_tro", "o H3 res 9".into()),
                ("khoang_cach_luoi", d.into()),
                ("o", c.to_string().into()),
            ],
        ));
        let _ = cell_center(c);
    }
    let path = labs::results_dir().join("geojson/loc08-h3-kring.geojson");
    std::fs::write(&path, to_string(feats)).expect("ghi geojson");
    println!(
        "\nđã ghi results/geojson/loc08-h3-kring.geojson (k-ring 2 ở res 9 quanh {center}, 19 ô)\n"
    );
}

fn cost() {
    println!("-- 4. chi phí tính ô (1.000.000 điểm, trung vị 5 lần, --release) --");
    let mut rng = Rand::new(SEED);
    let pts: Vec<LatLng> = (0..1_000_000)
        .map(|_| ll(rng.range(10.70, 10.86), rng.range(106.60, 106.78)))
        .collect();
    let res9 = resolution(9);
    let bench = |name: &str, f: &dyn Fn() -> u64| {
        let mut v: Vec<f64> = (0..5)
            .map(|_| {
                let t = Instant::now();
                black_box(f());
                t.elapsed().as_secs_f64() * 1e9 / pts.len() as f64
            })
            .collect();
        v.sort_by(f64::total_cmp);
        println!("  {name:<34} {:>8.1} ns/điểm", v[2]);
    };
    bench("H3 cell_of res 9", &|| {
        pts.iter()
            .map(|&p| u64::from(cell_of(black_box(p), res9)))
            .fold(0, u64::wrapping_add)
    });
    bench("geohash encode độ dài 7", &|| {
        pts.iter()
            .map(|&p| geohash::encode(black_box(p), 7).len() as u64)
            .sum()
    });
    let cells: Vec<_> = pts
        .iter()
        .take(100_000)
        .map(|&p| cell_of(p, res9))
        .collect();
    let t = Instant::now();
    let n: usize = cells
        .iter()
        .map(|&c| black_box(k_ring(black_box(c), 1)).len())
        .sum();
    println!(
        "  {:<34} {:>8.1} ns/ô (tổng {n} ô)",
        "H3 k_ring(1) res 9",
        t.elapsed().as_secs_f64() * 1e9 / cells.len() as f64
    );
    let t = Instant::now();
    let n: usize = cells
        .iter()
        .map(|&c| black_box(k_ring(black_box(c), 2)).len())
        .sum();
    println!(
        "  {:<34} {:>8.1} ns/ô (tổng {n} ô)",
        "H3 k_ring(2) res 9",
        t.elapsed().as_secs_f64() * 1e9 / cells.len() as f64
    );
}
