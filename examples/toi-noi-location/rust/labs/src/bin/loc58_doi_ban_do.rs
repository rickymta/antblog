//! LOC-58 (2/2): đổi phiên bản bản đồ trong lúc phục vụ. Yêu cầu đang chạy xong trên phiên bản cũ, yêu cầu mới thấy phiên bản mới ngay,
//! phiên bản cũ được giải phóng khi yêu cầu cuối cùng xong. Nhiều luồng truy vấn liên tục trong khi một luồng đổi phiên bản.
//!
//! Chạy từ gốc repo:
//!   cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc58_doi_ban_do -- --fixture
//! Tham số: --fixture (đồ thị tổng hợp; bỏ cờ để dùng HCM), --readers N (6), --swaps N (200).

use geo_app::ports::{MapSnapshot, MapSource, RouteQuery, RoutingProvider};
use geo_core::LatLng;
use labs::p03::uptime;
use labs::{Args, Rand, die, lab_root, load_graph, percentile};
use map_shard::ShardedMaps;
use map_shard::edit::close_ways;
use road_graph::Graph;
use route_engine::engine::{Engine, EngineConfig};
use route_engine::profile::Profile;
use route_engine::provider::OwnEngineProvider;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

fn snapshot(g: Graph, version: &str, tag: &str, fixture: bool) -> (MapSnapshot, Weak<Engine>, f64) {
    let t = Instant::now();
    let e = Arc::new(Engine::new(
        g,
        EngineConfig {
            ch_cache_dir: (!fixture).then(|| lab_root().join("rust/target/ch-cache")),
            cache_tag: tag.into(),
            ..EngineConfig::default()
        },
    ));
    e.ch(Profile::Car);
    let secs = t.elapsed().as_secs_f64();
    let weak = Arc::downgrade(&e);
    let provider: Arc<dyn RoutingProvider> = Arc::new(OwnEngineProvider {
        engine: Arc::clone(&e),
    });
    (
        MapSnapshot {
            city: "KM".into(),
            version: version.into(),
            engine: e,
            provider,
        },
        weak,
        secs,
    )
}

fn ms(p: &dyn RoutingProvider, a: LatLng, b: LatLng) -> Option<u32> {
    p.route(&RouteQuery {
        from: a,
        to: b,
        profile: Profile::Car,
        depart_sec_of_day: None,
    })
    .ok()
    .map(|r| (r.duration_s * 1000.0).round() as u32)
}

fn main() {
    let args = Args::from_env();
    let fixture = args.flag("--fixture");
    println!("== LOC-58: đổi phiên bản bản đồ không dừng (trên dữ liệu mô phỏng) ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let (g1, info) = load_graph(&args);
    println!("đồ thị: {info}");
    let readers = args.u64_or("--readers", 6) as usize;
    let swaps = args.u64_or("--swaps", 200) as usize;
    let mut rng = Rand::new(5858);
    let n_pairs = if fixture { 60 } else { 200 };
    let pairs: Vec<(LatLng, LatLng)> = (0..n_pairs)
        .map(|_| {
            (
                g1.node_latlng(rng.usize(g1.node_count()) as u32),
                g1.node_latlng(rng.usize(g1.node_count()) as u32),
            )
        })
        .collect();

    // v2 = v1 với con đường nhiều tuyến dùng nhất bị đóng
    let (s1, _, t1) = snapshot(
        g1.clone(),
        "v1",
        if fixture { "fixture" } else { "hcm" },
        fixture,
    );
    let mut count: HashMap<i64, usize> = HashMap::new();
    for &(a, b) in &pairs {
        if let Ok(r) = s1.engine.route(Profile::Car, a, b, None) {
            let ways: std::collections::BTreeSet<i64> =
                r.edges.iter().map(|&e| g1.way_id[e as usize]).collect();
            for w in ways {
                *count.entry(w).or_default() += 1;
            }
        }
    }
    let (busiest, used) = count
        .into_iter()
        .max_by_key(|&(w, c)| (c, w))
        .expect("có tuyến");
    let g2 = close_ways(&g1, &[busiest]);
    let (s2, _, t2) = snapshot(
        g2.clone(),
        "v2",
        if fixture { "fixture-v2" } else { "p07-hcm-v2" },
        fixture,
    );
    println!(
        "v1 dựng trong {t1:.2} s; v2 = v1 đóng way {busiest} (nằm trên {used}/{n_pairs} tuyến mẫu), dựng trong {t2:.2} s (engine + CH ô tô)"
    );
    let want1: Vec<Option<u32>> = pairs
        .iter()
        .map(|&(a, b)| ms(&*s1.provider, a, b))
        .collect();
    let want2: Vec<Option<u32>> = pairs
        .iter()
        .map(|&(a, b)| ms(&*s2.provider, a, b))
        .collect();
    let differ = want1.iter().zip(&want2).filter(|(x, y)| x != y).count();
    println!(
        "đáp án khác nhau giữa v1 và v2 ở {differ}/{n_pairs} cặp mẫu (nếu 0 thì thử nghiệm không chứng minh gì)"
    );
    if differ == 0 {
        die("hai phiên bản cho cùng đáp án");
    }

    let mut maps = ShardedMaps::new();
    maps.add(s1, g1.node_bbox());
    let maps = Arc::new(maps);

    // ---- 1. nhiều luồng truy vấn liên tục trong khi bản đồ bị đổi nhiều lần
    println!(
        "\n-- 1. {readers} luồng truy vấn liên tục, một luồng đổi qua lại v1/v2 {swaps} lần --"
    );
    let stop = AtomicBool::new(false);
    // seq của lần đổi: `started` tăng TRƯỚC khi gọi swap, `done` tăng SAU khi swap trả về
    let (started_seq, done_seq) = (AtomicU64::new(0), AtomicU64::new(0));
    let (n_v1, n_v2, bad, late_old) = (
        AtomicUsize::new(0),
        AtomicUsize::new(0),
        AtomicUsize::new(0),
        AtomicUsize::new(0),
    );
    let lat: std::sync::Mutex<Vec<f64>> = Default::default();
    let mut swap_us: Vec<f64> = Vec::new();
    std::thread::scope(|sc| {
        for t in 0..readers {
            let (maps, stop, pairs, want1, want2) = (&maps, &stop, &pairs, &want1, &want2);
            let (n_v1, n_v2, bad, late_old, lat, started_seq, done_seq) = (
                &n_v1,
                &n_v2,
                &bad,
                &late_old,
                &lat,
                &started_seq,
                &done_seq,
            );
            sc.spawn(move || {
                let mut i = t;
                let mut local = Vec::new();
                while !stop.load(Ordering::Relaxed) {
                    let done_before = done_seq.load(Ordering::SeqCst);
                    let t0 = Instant::now();
                    let snap = maps.snapshot("km").expect("luôn có bản đồ");
                    let k = i % pairs.len();
                    let got = ms(&*snap.provider, pairs[k].0, pairs[k].1);
                    local.push(t0.elapsed().as_secs_f64() * 1e6);
                    let v1 = snap.version == "v1";
                    if got != if v1 { want1[k] } else { want2[k] } {
                        bad.fetch_add(1, Ordering::Relaxed);
                    }
                    if v1 {
                        n_v1.fetch_add(1, Ordering::Relaxed)
                    } else {
                        n_v2.fetch_add(1, Ordering::Relaxed)
                    };
                    // Nếu lúc lấy snapshot chưa có lần đổi nào BẮT ĐẦU sau lần đổi `done_before`, thì snapshot phải là phiên bản của lần đổi đó.
                    // Lần đổi thứ n (n = 1, 2, …) đặt v2 khi n lẻ, v1 khi n chẵn; trước lần đổi nào (n = 0) là v1.
                    if started_seq.load(Ordering::SeqCst) == done_before {
                        let want_v1 = done_before % 2 == 0;
                        if v1 != want_v1 {
                            late_old.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    i += 1;
                }
                lat.lock().expect("khoá").extend(local);
            });
        }
        let (c1, c2) = (&g1, &g2);
        for k in 0..swaps {
            let (g, v) = if k % 2 == 0 { (c2, "v2") } else { (c1, "v1") };
            // dùng engine đã dựng sẵn, bọc vào một MapSnapshot mới: đo riêng thao tác «đổi con trỏ», không tính thời gian dựng
            let (s, _, _) = prebuilt(g, v, &g1, &g2, fixture);
            let t = Instant::now();
            started_seq.store(k as u64 + 1, Ordering::SeqCst);
            maps.swap("KM", s).expect("có thành phố");
            swap_us.push(t.elapsed().as_secs_f64() * 1e6);
            done_seq.store(k as u64 + 1, Ordering::SeqCst);
            std::thread::sleep(Duration::from_micros(400));
        }
        stop.store(true, Ordering::Relaxed);
    });
    let mut l = lat.into_inner().expect("khoá");
    l.sort_by(f64::total_cmp);
    swap_us.sort_by(f64::total_cmp);
    let total = n_v1.load(Ordering::Relaxed) + n_v2.load(Ordering::Relaxed);
    println!(
        "  tổng {total} yêu cầu: {} trên v1, {} trên v2",
        n_v1.load(Ordering::Relaxed),
        n_v2.load(Ordering::Relaxed)
    );
    println!(
        "  yêu cầu có đáp án KHÔNG khớp phiên bản nó báo (v1 hay v2): {}   (kỳ vọng 0)",
        bad.load(Ordering::Relaxed)
    );
    println!(
        "  độ trễ yêu cầu: p50 {:.0} µs, p99 {:.0} µs, max {:.0} µs",
        percentile(&l, 50.0),
        percentile(&l, 99.0),
        l.last().copied().unwrap_or(f64::NAN)
    );
    println!(
        "  thời gian của lệnh `swap` (đặt con trỏ): p50 {:.1} µs, p99 {:.1} µs, max {:.1} µs; số lần đổi {}",
        percentile(&swap_us, 50.0),
        percentile(&swap_us, 99.0),
        swap_us.last().copied().unwrap_or(f64::NAN),
        swap_us.len()
    );

    // ---- 2. yêu cầu đang chạy giữ phiên bản cũ; phiên bản cũ được giải phóng
    println!(
        "\n-- 2. một yêu cầu chạy lâu trong lúc bản đồ đổi, rồi phiên bản cũ được giải phóng --"
    );
    let (s3, weak3, t3) = snapshot(
        g2.clone(),
        "v3",
        if fixture { "fixture-v2" } else { "p07-hcm-v2" },
        fixture,
    );
    println!(
        "  dựng v3 NGOÀI đường phục vụ mất {t3:.2} s (đây là chi phí thật của «nạp bản đồ mới»; đường phục vụ không bị chặn trong lúc đó)"
    );
    let _ = maps.swap("KM", s3);
    let in_flight = maps.snapshot("KM").expect("có");
    println!(
        "  yêu cầu chạy lâu bắt đầu, giữ phiên bản {}",
        in_flight.version
    );
    let (s_next, _, _) = snapshot(
        g1.clone(),
        "v4",
        if fixture { "fixture" } else { "hcm" },
        fixture,
    );
    let t = Instant::now();
    let retired = maps.swap("KM", s_next).expect("có");
    println!(
        "  bản đồ đổi sang v4 sau {:.2} s kể từ lúc dựng xong; `swap` mất {:.1} µs; phiên bản bị thay: {}",
        0.0,
        t.elapsed().as_secs_f64() * 1e6,
        retired.version
    );
    drop(retired);
    println!(
        "  yêu cầu MỚI thấy phiên bản: {}",
        maps.version("KM").unwrap_or_default()
    );
    println!(
        "  v3 còn sống khi yêu cầu cũ chưa xong: {}",
        if weak3.upgrade().is_some() {
            "CÓ (đúng)"
        } else {
            "không (SAI)"
        }
    );
    let k = 0.min(pairs.len() - 1);
    let r = ms(&*in_flight.provider, pairs[k].0, pairs[k].1);
    println!(
        "  yêu cầu cũ vẫn tính xong trên {}: {:?} ms (đáp án v2/v3 kỳ vọng {:?})",
        in_flight.version, r, want2[k]
    );
    drop(in_flight);
    let t = Instant::now();
    while weak3.upgrade().is_some() && t.elapsed() < Duration::from_secs(2) {
        std::thread::sleep(Duration::from_millis(1));
    }
    println!(
        "  v3 được giải phóng sau khi yêu cầu cuối xong: {}",
        if weak3.upgrade().is_none() {
            "CÓ"
        } else {
            "CHƯA"
        }
    );
    println!(
        "  yêu cầu bắt đầu SAU khi `swap` xong mà vẫn thấy phiên bản cũ ở mục 1: {}   (kỳ vọng 0)",
        late_old.load(Ordering::Relaxed)
    );
    println!("uptime lúc kết thúc: {}", uptime());
}

/// Phiên bản dựng sẵn (cache theo đồ thị): `swap` nhận `MapSnapshot` theo giá trị nên mỗi lần đổi bọc lại cùng một engine.
fn prebuilt(
    g: &Graph,
    v: &str,
    g1: &Graph,
    g2: &Graph,
    fixture: bool,
) -> (MapSnapshot, Weak<Engine>, f64) {
    use std::sync::OnceLock;
    static ENGINES: OnceLock<[Arc<Engine>; 2]> = OnceLock::new();
    let e = ENGINES.get_or_init(|| {
        let mk = |g: &Graph, tag: &str| {
            let e = Arc::new(Engine::new(
                g.clone(),
                EngineConfig {
                    ch_cache_dir: (!fixture).then(|| lab_root().join("rust/target/ch-cache")),
                    cache_tag: tag.into(),
                    ..EngineConfig::default()
                },
            ));
            e.ch(Profile::Car);
            e
        };
        [
            mk(g1, if fixture { "fixture" } else { "hcm" }),
            mk(g2, if fixture { "fixture-v2" } else { "p07-hcm-v2" }),
        ]
    });
    let _ = g;
    let eng = Arc::clone(&e[usize::from(v == "v2")]);
    let weak = Arc::downgrade(&eng);
    let provider: Arc<dyn RoutingProvider> = Arc::new(OwnEngineProvider {
        engine: Arc::clone(&eng),
    });
    (
        MapSnapshot {
            city: "KM".into(),
            version: v.into(),
            engine: eng,
            provider,
        },
        weak,
        0.0,
    )
}
