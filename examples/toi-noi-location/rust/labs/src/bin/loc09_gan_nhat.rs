//! LOC-09: tìm tài xế gần nhất bằng quét toàn bộ, R-tree, ô H3 và Redis GEO, dưới tải theo hồ sơ Tới Nơi.
//!
//! Hồ sơ (bài du-an-mau-toi-noi): 25.000 tài xế trực tuyến lúc cao điểm, mỗi người gửi vị trí 4 giây một lần
//! = 6.250 cập nhật/giây; 70 chuyến mới mỗi giây, mỗi chuyến một truy vấn «10 tài xế gần nhất».
//!
//! Chạy từ gốc repo (cần Redis ở 127.0.0.1:6391: docker compose -f examples/toi-noi-location/infra/docker-compose.yml up -d --wait redis):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc09_gan_nhat
//! Tham số: --scales 2500,25000,250000 (số tài xế), --seconds S (giây mô phỏng cho mỗi pha, mặc định 8), --seed S,
//!          --redis-url URL, --no-redis (bỏ Redis một cách tường minh; không có cờ này mà Redis thiếu thì lab báo lỗi và dừng).

use driver_store::redis_geo::RedisGeo;
use driver_store::{DriverId, DriverIndex, FullScan, H3Buckets, RTreeBulk, RTreeIndex};
use geo_core::{LatLng, destination};
use labs::{Args, Rand, die, percentile};
use std::time::Instant;

const SEED: u64 = 20_261_106;
const PROFILE_DRIVERS: usize = 25_000;
const QUERIES_PER_SEC_AT_PROFILE: usize = 70;
const K: usize = 10;
const REDIS_RADIUS_M: u32 = 3_000;
const BBOX: [f64; 4] = [10.70, 10.86, 106.60, 106.78];
const HOTSPOTS: [(f64, f64); 8] = [
    (10.7769, 106.7009),
    (10.7626, 106.6602),
    (10.8231, 106.6297),
    (10.7380, 106.7220),
    (10.8010, 106.7140),
    (10.7500, 106.6300),
    (10.8450, 106.7700),
    (10.7200, 106.6800),
];

fn in_bbox(p: LatLng) -> bool {
    (BBOX[0]..=BBOX[1]).contains(&p.lat()) && (BBOX[2]..=BBOX[3]).contains(&p.lng())
}

/// 70% quanh 8 điểm nóng (σ ≈ 1,5 km mỗi trục), 30% rải đều trong bbox.
fn sample_point(rng: &mut Rand) -> LatLng {
    loop {
        let p = if rng.f64() < 0.7 {
            let (la, ln) = HOTSPOTS[rng.usize(HOTSPOTS.len())];
            let c = LatLng::try_new(la, ln).expect("hợp lệ");
            let east = destination(c, 90.0, rng.gauss() * 1500.0);
            destination(east, 0.0, rng.gauss() * 1500.0)
        } else {
            LatLng::try_new(rng.range(BBOX[0], BBOX[1]), rng.range(BBOX[2], BBOX[3]))
                .expect("hợp lệ")
        };
        if in_bbox(p) {
            return p.to_e7().to_latlng();
        }
    }
}

struct Workload {
    init: Vec<LatLng>,
    /// updates[s]: các cập nhật của giây s (N/4 tài xế, quay vòng).
    updates: Vec<Vec<(DriverId, LatLng)>>,
    /// queries[s]: các điểm hỏi của giây s.
    queries: Vec<Vec<LatLng>>,
    probe: Vec<LatLng>,
}

fn build_workload(n: usize, total_secs: usize, seed: u64) -> Workload {
    let mut rng = Rand::new(seed);
    let init: Vec<LatLng> = (0..n).map(|_| sample_point(&mut rng)).collect();
    let mut cur = init.clone();
    let mut heading: Vec<f64> = (0..n).map(|_| rng.range(0.0, 360.0)).collect();
    let speed: Vec<f64> = (0..n).map(|_| rng.range(0.0, 11.0)).collect(); // m/s, 0–40 km/h
    let per_sec = n / 4;
    let qps = (QUERIES_PER_SEC_AT_PROFILE * n / PROFILE_DRIVERS).max(1);
    let mut updates = Vec::with_capacity(total_secs);
    let mut queries = Vec::with_capacity(total_secs);
    for s in 0..total_secs {
        let chunk = s % 4;
        let mut ups = Vec::with_capacity(per_sec);
        for id in (chunk * per_sec)..((chunk + 1) * per_sec) {
            heading[id] = (heading[id] + rng.gauss() * 20.0).rem_euclid(360.0);
            let next = destination(cur[id], heading[id], speed[id] * 4.0);
            if in_bbox(next) {
                cur[id] = next.to_e7().to_latlng();
            } else {
                heading[id] = (heading[id] + 180.0) % 360.0;
            }
            ups.push((id as DriverId, cur[id]));
        }
        updates.push(ups);
        queries.push((0..qps).map(|_| sample_point(&mut rng)).collect());
    }
    let probe = (0..2000).map(|_| sample_point(&mut rng)).collect();
    Workload {
        init,
        updates,
        queries,
        probe,
    }
}

struct Row {
    name: String,
    load_ms: f64,
    upd_ns: f64,
    q_p50: f64,
    q_p99: f64,
    q_mean: f64,
    sec_ms: f64,
    agree: f64,
    note: String,
}

fn print_header() {
    println!(
        "{:<30} {:>9} {:>12} {:>10} {:>10} {:>10} {:>16} {:>9}",
        "cách",
        "nạp (ms)",
        "cập nhật ns",
        "hỏi p50 µs",
        "hỏi p99 µs",
        "hỏi TB µs",
        "1 giây tải (ms)",
        "khớp quét"
    );
}

fn print_row(r: &Row) {
    println!(
        "{:<30} {:>9.1} {:>12.1} {:>10.1} {:>10.1} {:>10.1} {:>16.1} {:>8.1}%  {}",
        r.name, r.load_ms, r.upd_ns, r.q_p50, r.q_p99, r.q_mean, r.sec_ms, r.agree, r.note
    );
}

fn lat_stats(mut v: Vec<f64>) -> (f64, f64, f64) {
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    v.sort_by(f64::total_cmp);
    (percentile(&v, 50.0), percentile(&v, 99.0), mean)
}

/// Truy vấn đúng nếu khoảng cách thứ i của hai danh sách lệch dưới 1 m (hai cách tính khoảng cách phẳng/equirect khác nhau chút ít).
fn agrees(a: &[f64], b: &[f64]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1.0)
}

fn run_memory(
    mut idx: Box<dyn DriverIndex>,
    w: &Workload,
    half: usize,
    scan_truth: &[Vec<f64>],
) -> Row {
    let name = idx.name();
    let t = Instant::now();
    for (id, p) in w.init.iter().enumerate() {
        idx.upsert(id as DriverId, *p);
    }
    idx.flush();
    let load_ms = t.elapsed().as_secs_f64() * 1e3;

    // Pha 1: chỉ cập nhật, flush mỗi giây mô phỏng.
    let t = Instant::now();
    let mut n_up = 0usize;
    for s in 0..half {
        for &(id, p) in &w.updates[s] {
            idx.upsert(id, p);
        }
        idx.flush();
        n_up += w.updates[s].len();
    }
    let upd_ns = t.elapsed().as_secs_f64() * 1e9 / n_up as f64;

    // Pha 2: truy vấn trên trạng thái cuối, đo từng truy vấn.
    let mut lat = Vec::with_capacity(w.probe.len());
    let mut ok = 0usize;
    let mut cmp = 0usize;
    for (i, &q) in w.probe.iter().enumerate() {
        let t = Instant::now();
        let r = idx.nearest(q, K);
        lat.push(t.elapsed().as_secs_f64() * 1e6);
        if i < scan_truth.len() {
            cmp += 1;
            let d: Vec<f64> = r.iter().map(|n| n.dist_m).collect();
            if agrees(&d, &scan_truth[i]) {
                ok += 1;
            }
        }
    }
    let (q_p50, q_p99, q_mean) = lat_stats(lat);

    // Pha 3: xen kẽ như hồ sơ thật: mỗi giây các cập nhật rồi các truy vấn, đo thời gian cả giây.
    let t = Instant::now();
    let mut sink = 0usize;
    for s in half..w.updates.len() {
        for &(id, p) in &w.updates[s] {
            idx.upsert(id, p);
        }
        idx.flush();
        for &q in &w.queries[s] {
            sink += idx.nearest(q, K).len();
        }
    }
    std::hint::black_box(sink);
    let sec_ms = t.elapsed().as_secs_f64() * 1e3 / (w.updates.len() - half) as f64;
    Row {
        name,
        load_ms,
        upd_ns,
        q_p50,
        q_p99,
        q_mean,
        sec_ms,
        agree: 100.0 * ok as f64 / cmp as f64,
        note: String::new(),
    }
}

fn run_redis(
    url: &str,
    w: &Workload,
    half: usize,
    scan_ids: &[Vec<DriverId>],
    pipelined: bool,
) -> Row {
    let key = if pipelined {
        "tn:lab:loc09:pipe"
    } else {
        "tn:lab:loc09:seq"
    };
    let mut r = RedisGeo::connect(url, key)
        .unwrap_or_else(|e| die(&format!("mất kết nối Redis giữa chừng: {e}")));
    r.clear().expect("xóa key");
    let mem_before = r.used_memory_bytes().unwrap_or(0);
    let pipe_batch = 500usize;
    let upsert_all = |r: &mut RedisGeo, ups: &[(DriverId, LatLng)]| {
        if pipelined {
            for c in ups.chunks(pipe_batch) {
                r.upsert_batch(c).expect("pipeline");
            }
        } else {
            for &(id, p) in ups {
                r.upsert(id, p).expect("GEOADD");
            }
        }
    };
    let init: Vec<(DriverId, LatLng)> = w
        .init
        .iter()
        .enumerate()
        .map(|(i, p)| (i as DriverId, *p))
        .collect();
    let t = Instant::now();
    // Nạp ban đầu luôn dùng pipeline (không thuộc đường nóng), để bảng so sánh tập trung vào cập nhật + truy vấn.
    for c in init.chunks(pipe_batch) {
        r.upsert_batch(c).expect("pipeline");
    }
    let load_ms = t.elapsed().as_secs_f64() * 1e3;
    let mem_mb = r
        .used_memory_bytes()
        .unwrap_or(0)
        .saturating_sub(mem_before) as f64
        / 1e6;

    let t = Instant::now();
    let mut n_up = 0usize;
    for s in 0..half {
        upsert_all(&mut r, &w.updates[s]);
        n_up += w.updates[s].len();
    }
    let upd_ns = t.elapsed().as_secs_f64() * 1e9 / n_up as f64;

    let mut lat = Vec::with_capacity(w.probe.len());
    let (mut ok, mut cmp) = (0usize, 0usize);
    for (i, &q) in w.probe.iter().enumerate() {
        let t = Instant::now();
        let res = r.nearest(q, K, REDIS_RADIUS_M).expect("GEOSEARCH");
        lat.push(t.elapsed().as_secs_f64() * 1e6);
        if i < scan_ids.len() {
            cmp += 1;
            let mut a: Vec<DriverId> = res.iter().map(|n| n.id).collect();
            let mut b = scan_ids[i].clone();
            a.sort_unstable();
            b.sort_unstable();
            if a == b {
                ok += 1;
            }
        }
    }
    let (q_p50, q_p99, q_mean) = lat_stats(lat);

    let t = Instant::now();
    let mut sink = 0usize;
    for s in half..w.updates.len() {
        upsert_all(&mut r, &w.updates[s]);
        for &q in &w.queries[s] {
            sink += r.nearest(q, K, REDIS_RADIUS_M).expect("GEOSEARCH").len();
        }
    }
    std::hint::black_box(sink);
    let sec_ms = t.elapsed().as_secs_f64() * 1e3 / (w.updates.len() - half) as f64;
    if pipelined {
        // Chi phí GEOSEARCH theo bán kính: COUNT k với ASC vẫn phải xếp hạng mọi điểm trong bán kính.
        println!("  GEOSEARCH theo bán kính (1000 truy vấn, k = {K}):");
        for radius in [300u32, 1_000, 3_000] {
            let mut lat = Vec::new();
            let mut got = 0usize;
            for &q in w.probe.iter().take(1000) {
                let t = Instant::now();
                got += r.nearest(q, K, radius).expect("GEOSEARCH").len();
                lat.push(t.elapsed().as_secs_f64() * 1e6);
            }
            let (p50, p99, _) = lat_stats(lat);
            println!(
                "    bán kính {radius:>5} m: p50 {p50:>8.1} µs, p99 {p99:>9.1} µs, trung bình {:.2} kết quả/truy vấn",
                got as f64 / 1000.0
            );
        }
    }
    r.clear().expect("dọn key");
    Row {
        name: if pipelined {
            format!("Redis GEO, pipeline {pipe_batch}/lô")
        } else {
            "Redis GEO, 1 lệnh/vòng mạng".into()
        },
        load_ms,
        upd_ns,
        q_p50,
        q_p99,
        q_mean,
        sec_ms,
        agree: 100.0 * ok as f64 / cmp as f64,
        note: format!(
            "(Redis used_memory tăng {mem_mb:.1} MB sau nạp, khớp = cùng tập id, bán kính {REDIS_RADIUS_M} m)"
        ),
    }
}

fn main() {
    let args = Args::from_env();
    let seed = args.u64_or("--seed", SEED);
    let half = args.u64_or("--seconds", 8) as usize;
    let no_redis = args.flag("--no-redis");
    let url = args
        .value("--redis-url")
        .unwrap_or("redis://127.0.0.1:6391/")
        .to_string();
    let scales: Vec<usize> = args
        .value("--scales")
        .unwrap_or("2500,25000,250000")
        .split(',')
        .map(|s| {
            s.trim()
                .parse()
                .unwrap_or_else(|_| die("--scales cần danh sách số nguyên"))
        })
        .collect();

    println!("== LOC-09: tìm tài xế gần nhất ==");
    println!(
        "seed {seed}; mỗi pha {half} giây mô phỏng; k = {K}; một luồng, không khóa, không tranh chấp"
    );
    println!(
        "tải theo hồ sơ: N tài xế gửi vị trí 4 giây một lần (N/4 cập nhật/giây), {QUERIES_PER_SEC_AT_PROFILE} truy vấn/giây ở N = {PROFILE_DRIVERS} (tỉ lệ theo N ở các quy mô khác)"
    );
    println!(
        "vị trí: 70% quanh 8 điểm nóng (σ 1,5 km), 30% đều trong bbox 10.70–10.86 / 106.60–106.78; tài xế đi 0–40 km/h, đổi hướng ngẫu nhiên σ 20° mỗi lần báo\n"
    );

    let mut redis_info = None;
    if no_redis {
        println!("[bỏ qua theo yêu cầu --no-redis] Redis GEO không được đo trong lần chạy này\n");
    } else {
        match RedisGeo::connect(&url, "tn:lab:loc09:probe") {
            Ok(mut r) => {
                let ver = r.server_version().unwrap_or_default();
                let mut rtts: Vec<f64> = (0..2000)
                    .map(|_| {
                        let t = Instant::now();
                        r.ping().expect("PING");
                        t.elapsed().as_secs_f64() * 1e6
                    })
                    .collect();
                rtts.sort_by(f64::total_cmp);
                println!(
                    "Redis {ver} tại {url}: PING trung vị {:.1} µs, p99 {:.1} µs (2000 lần): chi phí tối thiểu của MỌI lệnh, vì đi qua mạng loopback/Docker",
                    percentile(&rtts, 50.0),
                    percentile(&rtts, 99.0)
                );
                println!(
                    "(Redis chạy trong container Docker Desktop; số Redis phụ thuộc Docker và loopback của máy viết, không phải server Redis riêng)\n"
                );
                redis_info = Some(url.clone());
            }
            Err(e) => die(&format!(
                "không kết nối được Redis tại {url}: {e}. Bật bằng `docker compose -f examples/toi-noi-location/infra/docker-compose.yml up -d --wait redis`, hoặc chạy lại với --no-redis để bỏ Redis một cách tường minh"
            )),
        }
    }

    for &n in &scales {
        let factor = n as f64 / PROFILE_DRIVERS as f64;
        println!(
            "---- N = {n} tài xế (×{factor:.1} so với hồ sơ): {} cập nhật/giây, {} truy vấn/giây ----",
            n / 4,
            (QUERIES_PER_SEC_AT_PROFILE * n / PROFILE_DRIVERS).max(1)
        );
        let w = build_workload(n, half * 2, seed ^ n as u64);
        let center = LatLng::try_new(10.78, 106.69).expect("hợp lệ");

        // Đáp án chuẩn: quét toàn bộ trên trạng thái sau pha 1.
        let mut truth = FullScan::new(n);
        for (id, p) in w.init.iter().enumerate() {
            truth.upsert(id as DriverId, *p);
        }
        for s in 0..half {
            for &(id, p) in &w.updates[s] {
                truth.upsert(id, p);
            }
        }
        let ncmp = 1000.min(w.probe.len());
        let scan_truth: Vec<Vec<f64>> = w.probe[..ncmp]
            .iter()
            .map(|&q| truth.nearest(q, K).iter().map(|n| n.dist_m).collect())
            .collect();
        let scan_ids: Vec<Vec<DriverId>> = w.probe[..ncmp]
            .iter()
            .map(|&q| {
                truth
                    .nearest(q, K)
                    .iter()
                    .filter(|x| x.dist_m <= f64::from(REDIS_RADIUS_M) - 5.0)
                    .map(|x| x.id)
                    .collect()
            })
            .collect();

        print_header();
        let variants: Vec<Box<dyn DriverIndex>> = vec![
            Box::new(FullScan::new(n)),
            Box::new(RTreeIndex::new(center, n)),
            Box::new(RTreeBulk::new(center, n)),
            Box::new(H3Buckets::new(8, n)),
            Box::new(H3Buckets::new(9, n)),
        ];
        for v in variants {
            let is_h3 = v.name().starts_with("ô H3");
            let mut row = run_memory(v, &w, half, &scan_truth);
            if is_h3 {
                row.note = String::new();
            }
            print_row(&row);
        }
        if n == PROFILE_DRIVERS {
            if let Some(url) = &redis_info {
                for pipelined in [false, true] {
                    // Redis chỉ trả người trong bán kính; so khớp với đáp án quét đã lọc cùng bán kính.
                    let row = run_redis(url, &w, half, &scan_ids, pipelined);
                    print_row(&row);
                }
            }
        } else if redis_info.is_some() {
            println!("(Redis chỉ đo ở N = {PROFILE_DRIVERS}, quy mô theo hồ sơ)");
        }
        // Số lần đổi ô H3 trong pha cập nhật, để so chi phí sửa R-tree với H3.
        let mut h3 = H3Buckets::new(9, n);
        for (id, p) in w.init.iter().enumerate() {
            h3.upsert(id as DriverId, *p);
        }
        let before = h3.cell_changes;
        let mut total = 0usize;
        for s in 0..half {
            for &(id, p) in &w.updates[s] {
                h3.upsert(id, p);
                total += 1;
            }
        }
        println!(
            "H3 res 9: {} / {total} cập nhật đổi ô ({:.1}%) trong pha cập nhật",
            h3.cell_changes - before,
            100.0 * (h3.cell_changes - before) as f64 / total as f64
        );
        println!();
    }
    println!(
        "PostGIS: không chạy (lab không có PostGIS; image chưa có sẵn), nên không có số đo. Không ước lượng thay."
    );
}
