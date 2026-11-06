//! LOC-04: haversine, equirectangular, sai số đo được, tốc độ và hệ số vòng đường/chim bay.
//!
//! Chạy từ gốc repo (phần hệ số vòng cần data/hcm-graph.bin từ lab LOC-06, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc04_khoang_cach
//! Tham số: --sources N (số nút nguồn cho hệ số vòng, mặc định 200), --seed S, --no-graph (bỏ phần hệ số vòng).

use geo::{Distance, Geodesic, Haversine, Point};
use geo_core::{LatLng, LocalProjection, destination, equirect_m, haversine_m};
use labs::{Args, Rand, percentile};
use road_graph::components::largest_scc_mask;
use road_graph::{Mode, sp};
use std::hint::black_box;
use std::time::Instant;

const SEED: u64 = 20_261_106;

fn main() {
    let args = Args::from_env();
    let seed = args.u64_or("--seed", SEED);
    println!("== LOC-04: khoảng cách ==");
    println!("seed {seed}\n");
    accuracy(seed);
    speed(seed);
    if args.flag("--no-graph") {
        println!("-- 4. hệ số vòng: bỏ qua theo yêu cầu --no-graph");
    } else {
        detour(&args, seed);
    }
}

fn pt(p: LatLng) -> Point<f64> {
    Point::new(p.lng(), p.lat())
}

fn accuracy(seed: u64) {
    println!(
        "-- 1. sai số equirectangular so với haversine (20.000 cặp mỗi ô, hướng ngẫu nhiên) --"
    );
    println!(
        "điểm B = destination(A, hướng ngẫu nhiên, d) trên mặt cầu nên haversine(A,B) = d (độ lệch do làm tròn số học dưới 1e-8 m)"
    );
    println!(
        "{:>9} {:>9} {:>14} {:>14} {:>14}",
        "vĩ độ A", "d (m)", "trung bình (m)", "lớn nhất (m)", "lớn nhất (%)"
    );
    let mut rng = Rand::new(seed);
    let lats = [
        ("8.60", 8.60),
        ("10.77", 10.77),
        ("16.05", 16.05),
        ("21.03", 21.03),
        ("23.00", 23.00),
    ];
    for (label, lat) in lats {
        let a = LatLng::try_new(lat, 106.0).expect("hợp lệ");
        for d in [500.0, 1_000.0, 2_000.0, 5_000.0, 10_000.0, 50_000.0] {
            let (mut sum, mut max_abs, mut max_rel) = (0.0f64, 0.0f64, 0.0f64);
            let n = 20_000;
            for _ in 0..n {
                let b = destination(a, rng.range(0.0, 360.0), d);
                let h = haversine_m(a, b);
                let e = equirect_m(a, b);
                sum += e - h;
                max_abs = max_abs.max((e - h).abs());
                max_rel = max_rel.max((e - h).abs() / h);
            }
            println!(
                "{label:>9} {d:>9.0} {:>14.5} {max_abs:>14.5} {:>14.7}",
                sum / f64::from(n),
                max_rel * 100.0
            );
        }
    }

    println!(
        "\n-- 1b. mặt phẳng mét cố định tại tâm bbox TP.HCM (LocalProjection, cos vĩ độ không đổi) so với haversine --"
    );
    println!(
        "điểm A ngẫu nhiên trong bbox lat 10.70–10.86, lng 106.60–106.78; B cách A d mét theo hướng ngẫu nhiên; 20.000 cặp mỗi dòng"
    );
    let proj = LocalProjection::new(LatLng::try_new(10.78, 106.69).expect("hợp lệ"));
    println!(
        "{:>9} {:>14} {:>14}",
        "d (m)", "lớn nhất (m)", "lớn nhất (%)"
    );
    for d in [500.0, 2_000.0, 5_000.0, 10_000.0] {
        let (mut max_abs, mut max_rel) = (0f64, 0f64);
        for _ in 0..20_000 {
            let a = LatLng::try_new(rng.range(10.70, 10.86), rng.range(106.60, 106.78))
                .expect("hợp lệ");
            let b = destination(a, rng.range(0.0, 360.0), d);
            let (xa, xb) = (proj.to_xy(a), proj.to_xy(b));
            let planar = (xa[0] - xb[0]).hypot(xa[1] - xb[1]);
            let h = haversine_m(a, b);
            max_abs = max_abs.max((planar - h).abs());
            max_rel = max_rel.max((planar - h).abs() / h);
        }
        println!("{d:>9.0} {max_abs:>14.4} {:>14.5}", max_rel * 100.0);
    }
    println!(
        "\n-- 2. haversine (mặt cầu) so với trắc địa trên ellipsoid WGS84 (geo::Geodesic), 20.000 cặp, vĩ độ A 10.77 --"
    );
    println!(
        "{:>9} {:>20} {:>20} {:>26} {:>20}",
        "d (m)",
        "haversine: lớn nhất (m)",
        "haversine: lớn nhất (%)",
        "equirect: lớn nhất (m)",
        "equirect: lớn nhất (%)"
    );
    let a = LatLng::try_new(10.77, 106.70).expect("hợp lệ");
    for d in [500.0, 2_000.0, 5_000.0, 10_000.0] {
        let (mut hm, mut hr, mut em, mut er) = (0f64, 0f64, 0f64, 0f64);
        let mut geo_diff = 0f64;
        for _ in 0..20_000 {
            let b = destination(a, rng.range(0.0, 360.0), d);
            let truth = Geodesic.distance(pt(a), pt(b));
            let (h, e) = (haversine_m(a, b), equirect_m(a, b));
            hm = hm.max((h - truth).abs());
            hr = hr.max((h - truth).abs() / truth);
            em = em.max((e - truth).abs());
            er = er.max((e - truth).abs() / truth);
            geo_diff = geo_diff.max((h - Haversine.distance(pt(a), pt(b))).abs());
        }
        println!(
            "{d:>9.0} {hm:>23.4} {:>23.5} {em:>26.4} {:>20.5}   (haversine của geo-core lệch haversine của crate geo tối đa {geo_diff:.2e} m)",
            hr * 100.0,
            er * 100.0
        );
    }
    println!();
}

fn speed(seed: u64) {
    println!(
        "-- 3. tốc độ (đồng hồ thô: 1.000.000 cặp quanh TP.HCM, cách nhau tối đa vài km, 5 lần chạy, --release) --"
    );
    let mut rng = Rand::new(seed ^ 0xABCD);
    let n = 1_000_000;
    let pairs: Vec<(LatLng, LatLng)> = (0..n)
        .map(|_| {
            let a = LatLng::try_new(rng.range(10.70, 10.86), rng.range(106.60, 106.78))
                .expect("hợp lệ");
            let b = destination(a, rng.range(0.0, 360.0), rng.range(50.0, 5_000.0));
            (a, b)
        })
        .collect();
    let geo_pairs: Vec<(Point<f64>, Point<f64>)> =
        pairs.iter().map(|&(a, b)| (pt(a), pt(b))).collect();

    let bench = |name: &str, f: &dyn Fn() -> f64| {
        let mut ns: Vec<f64> = (0..5)
            .map(|_| {
                let t = Instant::now();
                black_box(f());
                t.elapsed().as_secs_f64() * 1e9 / n as f64
            })
            .collect();
        ns.sort_by(f64::total_cmp);
        println!(
            "  {name:<28} nhanh nhất {:>8.2} ns/cặp   trung vị {:>8.2}   chậm nhất {:>8.2}",
            ns[0], ns[2], ns[4]
        );
        ns[2]
    };
    let hav = bench("haversine (geo-core)", &|| {
        pairs
            .iter()
            .map(|&(a, b)| haversine_m(black_box(a), black_box(b)))
            .sum()
    });
    let eq = bench("equirectangular (geo-core)", &|| {
        pairs
            .iter()
            .map(|&(a, b)| equirect_m(black_box(a), black_box(b)))
            .sum()
    });
    let _ = bench("geo::Haversine", &|| {
        geo_pairs
            .iter()
            .map(|&(a, b)| Haversine.distance(black_box(a), black_box(b)))
            .sum()
    });
    let geod = bench("geo::Geodesic (ellipsoid)", &|| {
        geo_pairs
            .iter()
            .map(|&(a, b)| Geodesic.distance(black_box(a), black_box(b)))
            .sum()
    });
    println!(
        "  tỉ lệ thời gian trung vị: haversine / equirect = {:.2}, geodesic / haversine = {:.1}, geodesic / equirect = {:.1}",
        hav / eq,
        geod / hav,
        geod / eq
    );
    println!("  (số đo chặt hơn: cargo bench -p labs --bench loc04_khoang_cach, dùng criterion)\n");
}

fn detour(args: &Args, seed: u64) {
    let (g, source) = labs::load_graph(args);
    println!(
        "-- 4. hệ số vòng = đường đi ngắn nhất theo đường / khoảng cách chim bay (haversine) --"
    );
    println!(
        "đồ thị: {source}: {} nút, {} cạnh",
        g.node_count(),
        g.edge_count()
    );
    let small = g.node_count() < 2_000;
    let bands: [(f64, f64); 3] = if small {
        [(200.0, 500.0), (500.0, 1000.0), (1000.0, 2000.0)]
    } else {
        [(1000.0, 2000.0), (2000.0, 5000.0), (5000.0, 10_000.0)]
    };
    let sources = args.u64_or("--sources", if small { 60 } else { 200 }) as usize;
    println!(
        "{sources} nút nguồn ngẫu nhiên trong thành phần liên thông mạnh lớn nhất của từng profile; mỗi nguồn lấy tối đa 15 đích mỗi dải; Dijkstra đầy đủ; bỏ qua cấm rẽ"
    );
    println!(
        "{:<8} {:>14} {:>7} {:>8} {:>8} {:>8} {:>8} {:>10}",
        "profile", "dải chim bay", "số cặp", "trung bình", "trung vị", "p90", "lớn nhất", ">= 2,0"
    );
    let mut rng = Rand::new(seed ^ 0x1234);
    for mode in Mode::ALL {
        let mask = largest_scc_mask(&g, mode);
        let members: Vec<u32> = (0..g.node_count() as u32)
            .filter(|&n| mask[n as usize])
            .collect();
        let mut samples: Vec<Vec<f64>> = vec![Vec::new(); bands.len()];
        for _ in 0..sources {
            let s = members[rng.usize(members.len())];
            let dist = sp::distances_cm(&g, mode, s);
            let sll = g.node_latlng(s);
            let mut pool: Vec<Vec<(u32, f64)>> = vec![Vec::new(); bands.len()];
            for &t in &members {
                let d = dist[t as usize];
                if d == sp::INF || t == s {
                    continue;
                }
                let straight = equirect_m(sll, g.node_latlng(t));
                if let Some(b) = bands
                    .iter()
                    .position(|&(lo, hi)| straight >= lo && straight < hi)
                {
                    pool[b].push((t, f64::from(d) / 100.0));
                }
            }
            for (b, p) in pool.iter().enumerate() {
                for _ in 0..15.min(p.len()) {
                    let (t, road) = p[rng.usize(p.len())];
                    let straight = haversine_m(sll, g.node_latlng(t));
                    samples[b].push(road / straight);
                }
            }
        }
        for (b, v) in samples.iter_mut().enumerate() {
            v.sort_by(f64::total_cmp);
            if v.is_empty() {
                continue;
            }
            let mean = v.iter().sum::<f64>() / v.len() as f64;
            let ge2 = v.iter().filter(|&&x| x >= 2.0).count() as f64 / v.len() as f64 * 100.0;
            println!(
                "{:<8} {:>5.0}–{:<5.0} m {:>7} {:>8.3} {:>8.3} {:>8.3} {:>8.3} {:>9.1}%",
                mode.name(),
                bands[b].0,
                bands[b].1,
                v.len(),
                mean,
                percentile(v, 50.0),
                percentile(v, 90.0),
                v[v.len() - 1],
                ge2
            );
        }
    }
}
