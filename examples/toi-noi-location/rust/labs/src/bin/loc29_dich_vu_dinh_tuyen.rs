//! LOC-29: dịch vụ định tuyến gRPC (tonic): Route, Matrix, deadline, cache theo ô H3 và khung giờ, fallback, p50/p99 dưới tải.
//!
//! Mặc định máy chủ chạy **cùng tiến trình** với bộ tạo tải (hai runtime tokio riêng, qua TCP loopback), nên máy khách và máy chủ
//! tranh nhau CPU của một máy đang dùng chung. Muốn đo máy chủ riêng thì chạy `geo-engine` rồi thêm `--addr`.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc29_dich_vu_dinh_tuyen
//! Tham số: --clients N (mặc định 64), --requests N mỗi pha (mặc định 3000), --server-threads N (mặc định 4),
//!          --client-threads N (mặc định 2), --seed S, --fixture, --addr HOST:PORT

use geo_core::{LatLng, destination};
use geo_engine::pb::routing_service_client::RoutingServiceClient;
use geo_engine::pb::{LatLngE7, MatrixRequest, RouteRequest, RoutingProfile, RoutingQuality};
use geo_engine::{GeoEngineConfig, MetricsSnapshot, RoutingSvc};
use labs::p03::{mean, uptime, vn};
use labs::{Args, Rand, lab_root, load_graph};
use route_engine::alt::largest_scc_nodes;
use route_engine::engine::{Engine, EngineConfig};
use route_engine::profile::Profile;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tonic::transport::Channel;

const SEED: u64 = 20_261_106;

fn e7(p: LatLng) -> Option<LatLngE7> {
    Some(LatLngE7 {
        lat_e7: (p.lat() * 1e7).round() as i32,
        lng_e7: (p.lng() * 1e7).round() as i32,
    })
}

fn pct(v: &[f64], p: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    s[((p / 100.0) * s.len() as f64).ceil().max(1.0) as usize - 1]
}

#[derive(Clone, Copy)]
struct Sample {
    us: f64,
    quality: i32,
    /// None = lỗi gRPC
    ok: bool,
    code: Option<tonic::Code>,
}

struct PhaseResult {
    wall_s: f64,
    samples: Vec<Sample>,
}

impl PhaseResult {
    fn rps(&self) -> f64 {
        self.samples.len() as f64 / self.wall_s
    }
    fn lat(&self, f: impl Fn(&Sample) -> bool) -> Vec<f64> {
        self.samples
            .iter()
            .filter(|s| f(s))
            .map(|s| s.us / 1000.0)
            .collect()
    }
    fn share(&self, f: impl Fn(&Sample) -> bool) -> f64 {
        100.0 * self.samples.iter().filter(|s| f(s)).count() as f64
            / self.samples.len().max(1) as f64
    }
}

type Fut = std::pin::Pin<Box<dyn std::future::Future<Output = Sample> + Send>>;
type Call<R> = Arc<dyn Fn(RoutingServiceClient<Channel>, R) -> Fut + Send + Sync>;

/// Chạy `n` yêu cầu với `clients` tác vụ song song, mỗi tác vụ lấy việc tiếp theo từ một bộ đếm chung.
async fn drive<R: Send + 'static>(
    chans: &[RoutingServiceClient<Channel>],
    clients: usize,
    n: usize,
    make: Arc<dyn Fn(usize) -> R + Send + Sync>,
    call: Call<R>,
) -> PhaseResult {
    let next = Arc::new(AtomicUsize::new(0));
    let t0 = Instant::now();
    let mut tasks = Vec::new();
    for c in 0..clients {
        let (next, make, call) = (Arc::clone(&next), Arc::clone(&make), Arc::clone(&call));
        let client = chans[c % chans.len()].clone();
        tasks.push(tokio::spawn(async move {
            let mut out = Vec::new();
            loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= n {
                    break;
                }
                out.push(call(client.clone(), make(i)).await);
            }
            out
        }));
    }
    let mut samples = Vec::with_capacity(n);
    for t in tasks {
        samples.extend(t.await.expect("tác vụ"));
    }
    PhaseResult {
        wall_s: t0.elapsed().as_secs_f64(),
        samples,
    }
}

fn route_call(deadline: Option<Duration>) -> Call<RouteRequest> {
    Arc::new(move |mut c, req| {
        Box::pin(async move {
            let mut r = tonic::Request::new(req);
            if let Some(d) = deadline {
                r.set_timeout(d);
            }
            let t = Instant::now();
            let res = c.route(r).await;
            let us = t.elapsed().as_secs_f64() * 1e6;
            match res {
                Ok(x) => {
                    let x = x.into_inner();
                    Sample {
                        us,
                        quality: x.quality,
                        ok: true,
                        code: None,
                    }
                }
                Err(e) => Sample {
                    us,
                    quality: 0,
                    ok: false,
                    code: Some(e.code()),
                },
            }
        })
    })
}

fn matrix_call(deadline: Option<Duration>) -> Call<MatrixRequest> {
    Arc::new(move |mut c, req| {
        Box::pin(async move {
            let mut r = tonic::Request::new(req);
            if let Some(d) = deadline {
                r.set_timeout(d);
            }
            let t = Instant::now();
            let res = c.matrix(r).await;
            let us = t.elapsed().as_secs_f64() * 1e6;
            match res {
                Ok(x) => {
                    let x = x.into_inner();
                    Sample {
                        us,
                        quality: x.quality,
                        ok: true,
                        code: None,
                    }
                }
                Err(e) => Sample {
                    us,
                    quality: 0,
                    ok: false,
                    code: Some(e.code()),
                },
            }
        })
    })
}

fn row(name: &str, r: &PhaseResult) {
    let all = r.lat(|s| s.ok);
    let f = |v: f64| {
        if v.is_nan() {
            format!("{:>8}", "-")
        } else {
            format!("{v:>8.2}")
        }
    };
    println!(
        "  {name:<34} {:>7} {:>8.0} {} {} {} {}",
        r.samples.len(),
        r.rps(),
        f(pct(&all, 50.0)),
        f(pct(&all, 90.0)),
        f(pct(&all, 99.0)),
        f(if all.is_empty() {
            f64::NAN
        } else {
            all.iter().copied().fold(0.0, f64::max)
        })
    );
}

fn quality_line(r: &PhaseResult) -> String {
    let code = |c: tonic::Code| r.share(|s| s.code == Some(c));
    format!(
        "ROUTED {:.1}%, CACHED {:.1}%, ESTIMATED {:.1}%, lỗi gRPC {:.1}% (DEADLINE_EXCEEDED {:.1}%, CANCELLED {:.1}%, mã khác {:.1}%)",
        r.share(|s| s.ok && s.quality == RoutingQuality::Routed as i32),
        r.share(|s| s.ok && s.quality == RoutingQuality::Cached as i32),
        r.share(|s| s.ok && s.quality == RoutingQuality::Estimated as i32),
        r.share(|s| !s.ok),
        code(tonic::Code::DeadlineExceeded),
        code(tonic::Code::Cancelled),
        r.share(|s| !s.ok
            && s.code != Some(tonic::Code::DeadlineExceeded)
            && s.code != Some(tonic::Code::Cancelled))
    )
}

fn delta(a: MetricsSnapshot, b: MetricsSnapshot) -> String {
    format!(
        "máy chủ: routed +{}, cache +{}, fallback quá hạn +{}, quá tải +{}, không có đường +{}",
        b.routed - a.routed,
        b.cache_hits - a.cache_hits,
        b.fallback_deadline - a.fallback_deadline,
        b.fallback_overload - a.fallback_overload,
        b.fallback_no_route - a.fallback_no_route
    )
}

fn main() {
    let args = Args::from_env();
    let seed = args.u64_or("--seed", SEED);
    let clients = args.u64_or("--clients", 64) as usize;
    let n = args.u64_or("--requests", 3000) as usize;
    let sthreads = args.u64_or("--server-threads", 4) as usize;
    let cthreads = args.u64_or("--client-threads", 2) as usize;
    let (g, source) = load_graph(&args);
    let fixture = args.flag("--fixture");
    println!("== LOC-29: dịch vụ định tuyến gRPC ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    println!(
        "đồ thị: {source}: {} nút, {} cạnh; seed {seed}; {clients} client song song, {n} yêu cầu mỗi pha; runtime máy chủ {sthreads} luồng, runtime client {cthreads} luồng",
        vn(g.node_count() as u64),
        vn(g.edge_count() as u64)
    );

    let cache = (!fixture).then(|| lab_root().join("rust/target/ch-cache"));
    let engine = Arc::new(Engine::new(
        g,
        EngineConfig {
            ch_cache_dir: cache,
            cache_tag: if fixture {
                "fixture".into()
            } else {
                "hcm".into()
            },
            ..EngineConfig::default()
        },
    ));
    let t = Instant::now();
    engine.ch(Profile::Car);
    engine.ch(Profile::Motorbike);
    println!(
        "CH ô tô và xe máy sẵn sàng sau {:.2} s (nạp từ rust/target/ch-cache nếu có)",
        t.elapsed().as_secs_f64()
    );

    // mẫu điểm: gần các nút trong SCC lớn nhất của ô tô
    let scc = largest_scc_nodes(&engine.g, &engine.rev, &engine.costs(Profile::Car).w);
    let nodes: Vec<u32> = (0..engine.g.node_count() as u32)
        .filter(|&v| scc[v as usize])
        .collect();
    let mut rng = Rand::new(seed);
    let mut point = |rng: &mut Rand| -> LatLng {
        destination(
            engine.g.node_latlng(nodes[rng.usize(nodes.len())]),
            rng.range(0.0, 360.0),
            rng.range(3.0, 25.0),
        )
    };

    let cfg = GeoEngineConfig {
        max_inflight: sthreads,
        ..GeoEngineConfig::default()
    };
    println!(
        "cấu hình dịch vụ: H3 độ phân giải {} (cạnh ô ≈ {:.0} m), khung giờ cache {} s, dung lượng cache {}, tối đa {} tính toán đồng thời, deadline mặc định {} ms, dự trữ {} ms, ma trận tối đa {} ô",
        cfg.h3_res,
        geo_core::h3::edge_length_avg_m(geo_core::h3::resolution(cfg.h3_res)),
        cfg.time_bucket_s,
        vn(cfg.cache_capacity as u64),
        cfg.max_inflight,
        cfg.default_deadline_ms,
        cfg.reserve_ms,
        vn(cfg.max_matrix_cells as u64)
    );

    // máy chủ cùng tiến trình trên runtime riêng, hoặc máy chủ ngoài
    let srv_rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(sthreads)
        .enable_all()
        .build()
        .expect("runtime máy chủ");
    let svc = RoutingSvc::new(Arc::clone(&engine), cfg);
    let metrics = Arc::clone(&svc.metrics);
    let cache_ref = Arc::clone(&svc.cache);
    let addr = if let Some(a) = args.value("--addr") {
        println!("dùng máy chủ ngoài {a} (số đếm phía máy chủ trong báo cáo sẽ bằng 0)");
        a.to_string()
    } else {
        let listener = srv_rt
            .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
            .expect("bind");
        let a = listener.local_addr().expect("addr");
        let server = svc.clone().into_server();
        srv_rt.spawn(async move {
            tonic::transport::Server::builder()
                .add_service(server)
                .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
                .await
                .expect("serve");
        });
        println!("máy chủ cùng tiến trình nghe trên {a}");
        a.to_string()
    };

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(cthreads)
        .enable_all()
        .build()
        .expect("runtime client");
    let chans: Vec<RoutingServiceClient<Channel>> = rt.block_on(async {
        let mut v = Vec::new();
        for _ in 0..4 {
            v.push(
                RoutingServiceClient::connect(format!("http://{addr}"))
                    .await
                    .expect("kết nối"),
            );
        }
        v
    });

    println!(
        "\n-- 1. Route, cache lạnh, điểm đi và đến ngẫu nhiên (hầu như mọi yêu cầu trượt cache) --"
    );
    let pairs: Vec<(LatLng, LatLng)> = (0..n).map(|_| (point(&mut rng), point(&mut rng))).collect();
    // đối chứng: gọi thẳng engine trong tiến trình, không qua gRPC, cùng cặp điểm
    let direct = {
        let m = 400.min(n);
        let mut v = Vec::with_capacity(m);
        for &(a, b) in pairs.iter().take(m) {
            let t = Instant::now();
            let _ = std::hint::black_box(engine.route(Profile::Car, a, b, None));
            v.push(t.elapsed().as_secs_f64() * 1e3);
        }
        v
    };
    println!(
        "  đối chứng gọi thẳng engine.route trong tiến trình (một luồng, không gRPC, {} cặp đầu): p50 {:.3} ms, p99 {:.3} ms, trung bình {:.3} ms",
        direct.len(),
        pct(&direct, 50.0),
        pct(&direct, 99.0),
        mean(&direct)
    );
    let m0 = metrics.snapshot();
    let pairs = Arc::new(pairs);
    let p2 = Arc::clone(&pairs);
    let cold = rt.block_on(drive(
        &chans,
        clients,
        n,
        Arc::new(move |i| mk_route_static(p2[i].0, p2[i].1)),
        route_call(Some(Duration::from_secs(5))),
    ));
    println!(
        "  {:<34} {:>7} {:>8} {:>8} {:>8} {:>8} {:>9}",
        "pha", "n", "req/s", "p50 ms", "p90 ms", "p99 ms", "max ms"
    );
    row("Route lạnh (deadline 5 s)", &cold);
    println!("  chất lượng: {}", quality_line(&cold));
    println!("  {}", delta(m0, metrics.snapshot()));

    println!(
        "\n-- 2. Route với nhu cầu lệch (một số cặp ô rất phổ biến): hiệu quả của cache theo ô H3 + khung giờ --"
    );
    // 150 điểm đi và 150 điểm đến «phổ biến»; chọn theo luật Zipf (xác suất ∝ 1/hạng); mỗi yêu cầu lệch tối đa 40 m
    let hot_a: Vec<LatLng> = (0..150).map(|_| point(&mut rng)).collect();
    let hot_b: Vec<LatLng> = (0..150).map(|_| point(&mut rng)).collect();
    let zipf: Vec<f64> = (1..=150).map(|r| 1.0 / f64::from(r)).collect();
    let total: f64 = zipf.iter().sum();
    let pick = |rng: &mut Rand| -> usize {
        let mut x = rng.f64() * total;
        for (i, w) in zipf.iter().enumerate() {
            if x < *w {
                return i;
            }
            x -= w;
        }
        149
    };
    let jitter =
        |p: LatLng, rng: &mut Rand| destination(p, rng.range(0.0, 360.0), rng.range(0.0, 40.0));
    let skew: Vec<(LatLng, LatLng)> = (0..n)
        .map(|_| {
            let (a, b) = (hot_a[pick(&mut rng)], hot_b[pick(&mut rng)]);
            (jitter(a, &mut rng), jitter(b, &mut rng))
        })
        .collect();
    let skew = Arc::new(skew);
    let s2 = Arc::clone(&skew);
    let before = cache_ref.stats();
    let m1 = metrics.snapshot();
    let warm = rt.block_on(drive(
        &chans,
        clients,
        n,
        Arc::new(move |i| mk_route_static(s2[i].0, s2[i].1)),
        route_call(Some(Duration::from_secs(5))),
    ));
    let after = cache_ref.stats();
    row("Route lệch (cache đang ấm dần)", &warm);
    println!("  chất lượng: {}", quality_line(&warm));
    println!("  {}", delta(m1, metrics.snapshot()));
    println!(
        "  cache: {} mục, {} trúng, {} trượt, {} bỏ qua (cùng ô) trong pha này; tỉ lệ trúng {:.1}% trên các yêu cầu dùng được cache",
        vn(after.entries as u64),
        after.hits - before.hits,
        after.misses - before.misses,
        after.bypass - before.bypass,
        100.0 * (after.hits - before.hits) as f64
            / ((after.hits - before.hits) + (after.misses - before.misses)).max(1) as f64
    );
    let hit = warm.lat(|s| s.ok && s.quality == RoutingQuality::Cached as i32);
    let miss = warm.lat(|s| s.ok && s.quality == RoutingQuality::Routed as i32);
    println!(
        "  độ trễ theo nguồn: CACHED p50 {:.2} ms p99 {:.2} ms (n = {}); ROUTED p50 {:.2} ms p99 {:.2} ms (n = {})",
        pct(&hit, 50.0),
        pct(&hit, 99.0),
        hit.len(),
        pct(&miss, 50.0),
        pct(&miss, 99.0),
        miss.len()
    );
    // pha hai: chạy lại đúng chuỗi yêu cầu trên cache đã ấm
    let s3 = Arc::clone(&skew);
    let m2 = metrics.snapshot();
    let hot = rt.block_on(drive(
        &chans,
        clients,
        n,
        Arc::new(move |i| mk_route_static(s3[i].0, s3[i].1)),
        route_call(Some(Duration::from_secs(5))),
    ));
    row("Route lệch, chạy lại (cache ấm)", &hot);
    println!("  chất lượng: {}", quality_line(&hot));
    println!("  {}", delta(m2, metrics.snapshot()));
    // sai số của đáp án cache so với tính mới
    let mut errs = Vec::new();
    {
        let mut ch = chans[0].clone();
        let sample = 400.min(skew.len());
        rt.block_on(async {
            for &(a, b) in skew.iter().take(sample) {
                let Ok(r) = ch.route(mk_route_static(a, b)).await else {
                    continue;
                };
                let r = r.into_inner();
                if r.quality != RoutingQuality::Cached as i32 {
                    continue;
                }
                if let Ok(exact) = engine.route(Profile::Car, a, b, None) {
                    // máy chủ trong lab không bật hồ sơ giờ nên khung giờ của yêu cầu chỉ nằm trong khoá cache, trọng số là tốc độ tự do
                    errs.push(
                        100.0 * (f64::from(r.duration_ms) - f64::from(exact.duration_ms))
                            / f64::from(exact.duration_ms),
                    );
                }
            }
        });
    }
    if errs.is_empty() {
        println!("  không có đáp án cache để so sánh");
    } else {
        let abs: Vec<f64> = errs.iter().map(|e| e.abs()).collect();
        println!(
            "  sai số của đáp án CACHED so với tính mới đúng điểm ({} mẫu): trung bình {:+.1}%, |sai số| trung vị {:.1}%, p95 {:.1}%, lớn nhất {:.1}%",
            errs.len(),
            mean(&errs),
            pct(&abs, 50.0),
            pct(&abs, 95.0),
            pct(&abs, 100.0)
        );
    }

    println!(
        "\n-- 3. deadline chặt trên Route lạnh: đổi chính xác lấy khả năng trả lời đúng hạn --"
    );
    println!(
        "  {:<34} {:>7} {:>8} {:>8} {:>8} {:>8} {:>9}",
        "deadline của client", "n", "req/s", "p50 ms", "p90 ms", "p99 ms", "max ms"
    );
    for ms in [50u64, 10, 4] {
        let fresh: Vec<(LatLng, LatLng)> = (0..n.min(1500))
            .map(|_| (point(&mut rng), point(&mut rng)))
            .collect();
        let f2 = Arc::new(fresh);
        let f3 = Arc::clone(&f2);
        let m = metrics.snapshot();
        let r = rt.block_on(drive(
            &chans,
            clients,
            f2.len(),
            Arc::new(move |i| mk_route_static(f3[i].0, f3[i].1)),
            route_call(Some(Duration::from_millis(ms))),
        ));
        row(&format!("{ms} ms (dự trữ 3 ms)"), &r);
        println!("    chất lượng: {}", quality_line(&r));
        println!("    {}", delta(m, metrics.snapshot()));
    }

    println!("\n-- 4. Matrix --");
    println!(
        "  {:<34} {:>7} {:>8} {:>8} {:>8} {:>8} {:>9}",
        "hình dạng (client song song)", "n", "req/s", "p50 ms", "p90 ms", "p99 ms", "max ms"
    );
    let mk_matrix =
        |a: usize, b: usize, rng: &mut Rand, point: &mut dyn FnMut(&mut Rand) -> LatLng| {
            MatrixRequest {
                sources: (0..a).filter_map(|_| e7(point(rng))).collect(),
                targets: (0..b).filter_map(|_| e7(point(rng))).collect(),
                profile: RoutingProfile::Car as i32,
                depart_sec_of_day: None,
            }
        };
    for (a, b, par, reps) in [
        (20usize, 20usize, 1usize, 60usize),
        (100, 100, 1, 40),
        (100, 100, 8, 80),
        (300, 300, 1, 20),
        (300, 300, 8, 40),
    ] {
        let reqs: Vec<MatrixRequest> = (0..reps)
            .map(|_| mk_matrix(a, b, &mut rng, &mut point))
            .collect();
        let reqs = Arc::new(reqs);
        let r2 = Arc::clone(&reqs);
        let r = rt.block_on(drive(
            &chans,
            par,
            reps,
            Arc::new(move |i| r2[i].clone()),
            matrix_call(Some(Duration::from_secs(10))),
        ));
        row(&format!("{a}×{b} ({par} client)"), &r);
    }
    {
        // deadline chặt cho ma trận lớn: không kịp tính thì trả ước lượng
        let reqs: Vec<MatrixRequest> = (0..12)
            .map(|_| mk_matrix(500, 500, &mut rng, &mut point))
            .collect();
        let reqs = Arc::new(reqs);
        for ms in [2000u64, 40, 12] {
            let r2 = Arc::clone(&reqs);
            let m = metrics.snapshot();
            let r = rt.block_on(drive(
                &chans,
                1,
                12,
                Arc::new(move |i| r2[i].clone()),
                matrix_call(Some(Duration::from_millis(ms))),
            ));
            row(&format!("500×500, deadline {ms} ms"), &r);
            println!(
                "    chất lượng: {}; {}",
                quality_line(&r),
                delta(m, metrics.snapshot())
            );
        }
    }

    let end = metrics.snapshot();
    let cs = cache_ref.stats();
    println!(
        "\ntổng cộng phía máy chủ: {} Route, {} Matrix, {} đầu vào sai; cache {} mục, tỉ lệ trúng toàn phiên {:.1}%",
        end.route_requests,
        end.matrix_requests,
        end.invalid,
        vn(cs.entries as u64),
        100.0 * cs.hits as f64 / (cs.hits + cs.misses).max(1) as f64
    );
    println!(
        "(đo khi máy đang chịu tải, máy khách và máy chủ cùng một máy; chỉ so sánh tương đối trong cùng một lần chạy)"
    );
    println!("uptime lúc kết thúc: {}", uptime());
    drop(rt);
    srv_rt.shutdown_background();
}

fn mk_route_static(from: LatLng, to: LatLng) -> RouteRequest {
    RouteRequest {
        from: e7(from),
        to: e7(to),
        profile: RoutingProfile::Car as i32,
        depart_sec_of_day: Some(8 * 3600),
        want_geometry: false,
    }
}
