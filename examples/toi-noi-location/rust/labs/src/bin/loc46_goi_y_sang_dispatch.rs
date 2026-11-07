//! LOC-46: từ gợi ý tới assignment, hợp đồng với Dispatch C#. Rust đề xuất (`SuggestMatch`), C# quyết định.
//!
//! Lab này (1) chạy `MatchingSvc` thật trong tiến trình trên đồ thị fixture và cho **client .NET thật** (`dotnet/DispatchStub`) chạy
//! contract test với Dispatch stub trong bộ nhớ (lease, TTL, accept idempotent, gợi ý cũ, phản hồi muộn), in nguyên văn kết quả;
//! (2) đo độ trễ `SuggestMatch` trên đồ thị TP.HCM với bể 120 tài xế và 10–40 booking (máy chịu tải nặng: chỉ so sánh tương đối).
//!
//! Chạy từ gốc repo (cần .NET SDK; đo HCM cần data/hcm-graph.bin):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc46_goi_y_sang_dispatch
//! Tham số: --no-hcm (bỏ phần đo HCM), --no-dotnet (bỏ contract test .NET), --requests N (số lời gọi mỗi cỡ, mặc định 100)

use geo_core::LatLng;
use geo_engine::pb::matching_service_client::MatchingServiceClient;
use geo_engine::pb::{DriverCandidate, LatLngE7, MatchPolicy, PendingRequest, SuggestMatchRequest};
use geo_engine::{GeoEngineConfig, MatchingSvc};
use labs::p03::uptime;
use labs::p05::setup;
use labs::{Args, fixture_path, lab_root, percentile};
use route_engine::engine::{Engine, EngineConfig};
use simulator::world::generate;
use std::sync::Arc;
use tokio_stream::wrappers::TcpListenerStream;

fn e7(p: LatLng) -> Option<LatLngE7> {
    Some(LatLngE7 {
        lat_e7: (p.lat() * 1e7).round() as i32,
        lng_e7: (p.lng() * 1e7).round() as i32,
    })
}

async fn serve(engine: Arc<Engine>) -> String {
    let svc = MatchingSvc::new(engine, GeoEngineConfig::default());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    let server = svc.into_server();
    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(server)
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .expect("máy chủ gRPC");
    });
    format!("http://{addr}")
}

fn request(
    id: &str,
    pins: &[LatLng],
    drivers: &[LatLng],
    n_pending: usize,
    shift: usize,
) -> SuggestMatchRequest {
    SuggestMatchRequest {
        request_id: id.into(),
        city_code: "HCM".into(),
        now_ms: 1_700_000_000_000,
        policy: Some(MatchPolicy::default()),
        snapshot_version: 1,
        pending: (0..n_pending)
            .map(|k| PendingRequest {
                booking_id: format!("BK-{k:03}"),
                pickup: e7(pins[(k + shift) % pins.len()]),
                requested_at_ms: 1_700_000_000_000 - 20_000,
                waited_ms: None,
            })
            .collect(),
        drivers: drivers
            .iter()
            .enumerate()
            .map(|(k, &p)| DriverCandidate {
                driver_id: format!("DR-{k:03}"),
                position: e7(p),
                state_version: 1,
                idle_since_ms: 1_700_000_000_000 - 60_000,
                busy_remaining_ms: None,
            })
            .collect(),
        suggestion_ttl_ms: None,
        compute_budget_ms: None,
    }
}

fn main() {
    let args = Args::from_env();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("runtime");
    println!(
        "== LOC-46: từ gợi ý tới assignment, hợp đồng Rust ↔ Dispatch C# (trên dữ liệu mô phỏng) =="
    );
    println!("uptime lúc bắt đầu: {}", uptime());
    let mut all_ok = true;

    // ---------- 1. contract test với client .NET thật ----------
    if args.flag("--no-dotnet") {
        println!("\n-- 1. contract test .NET: BỎ QUA (--no-dotnet) --");
    } else {
        println!(
            "\n-- 1. contract test: máy chủ Rust thật (đồ thị fixture) ↔ client .NET thật + Dispatch stub trong bộ nhớ --"
        );
        let g = road_graph::fixture::load_fixture(&fixture_path()).expect("nạp fixture");
        println!(
            "  đồ thị fixtures/khu-mau.json: {} nút, {} cạnh",
            g.node_count(),
            g.edge_count()
        );
        let engine = Arc::new(Engine::new(g, EngineConfig::default()));
        let addr = rt.block_on(serve(engine));
        println!("  MatchingSvc nghe ở {addr}");
        let proj = lab_root().join("dotnet/DispatchStub");
        println!(
            "  lệnh: dotnet run --project examples/toi-noi-location/dotnet/DispatchStub -c Release -- contract --addr {addr}"
        );
        let out = std::process::Command::new("dotnet")
            .args(["run", "--project"])
            .arg(&proj)
            .args(["-c", "Release", "--", "contract", "--addr", &addr])
            .output();
        match out {
            Ok(o) => {
                print!("{}", String::from_utf8_lossy(&o.stdout));
                let err = String::from_utf8_lossy(&o.stderr);
                if !err.trim().is_empty() {
                    println!("[stderr của dotnet]\n{err}");
                }
                println!("  mã thoát của client .NET: {:?}", o.status.code());
                all_ok &= o.status.success();
            }
            Err(e) => {
                println!("  KHÔNG chạy được dotnet: {e}");
                all_ok = false;
            }
        }
    }

    // ---------- 2. độ trễ trên HCM ----------
    if args.flag("--no-hcm") || args.flag("--fixture") {
        println!("\n-- 2. đo độ trễ trên HCM: BỎ QUA --");
    } else {
        println!(
            "\n-- 2. độ trễ SuggestMatch trên đồ thị HCM: bể 120 tài xế rảnh, 10 / 20 / 40 booking, client tonic cùng tiến trình --"
        );
        let (engine, info, sc, _) = setup(&args);
        println!("  đồ thị: {info}; kịch bản {} (seed {})", sc.name, sc.seed);
        let w = generate(&engine, &sc);
        let drivers: Vec<LatLng> = w.drivers.iter().map(|d| d.pos.point(&engine.g)).collect();
        let pins: Vec<LatLng> = w.customers.iter().map(|c| c.view.pin).collect();
        println!(
            "  bể: {} tài xế, {} ghim khách có sẵn từ simulator",
            drivers.len(),
            pins.len()
        );
        let engine = Arc::new(engine);
        engine.ch(route_engine::profile::Profile::Car); // dựng/nạp CH trước khi đo, không tính vào độ trễ
        let n = args.u64_or("--requests", 100) as usize;
        rt.block_on(async {
            let addr = serve(Arc::clone(&engine)).await;
            let mut client = MatchingServiceClient::connect(addr).await.expect("kết nối");
            println!(
                "  {:>9} {:>6} {:>10} {:>10} {:>10} {:>12} {:>12} {:>10}",
                "booking",
                "n",
                "p50 (ms)",
                "p95 (ms)",
                "p99 (ms)",
                "ma trận µs*",
                "giải µs*",
                "gợi ý/lời gọi"
            );
            for &m in &[10usize, 20, 40] {
                let (mut lat, mut mat, mut sol, mut sugg) = (Vec::new(), Vec::new(), Vec::new(), 0usize);
                let mut estimated = 0usize;
                for k in 0..n {
                    let r = request(&format!("REQ-{m}-{k}"), &pins, &drivers, m, k * 7);
                    let t = std::time::Instant::now();
                    let res = client.suggest_match(r).await.expect("SuggestMatch").into_inner();
                    lat.push(t.elapsed().as_secs_f64() * 1e3);
                    let st = res.stats.expect("stats");
                    mat.push(f64::from(st.matrix_us));
                    sol.push(f64::from(st.solve_us));
                    sugg += res.suggestions.len();
                    estimated += usize::from(res.quality != geo_engine::pb::RoutingQuality::Routed as i32);
                }
                for v in [&mut lat, &mut mat, &mut sol] {
                    v.sort_by(f64::total_cmp);
                }
                println!(
                    "  {:>9} {:>6} {:>10.2} {:>10.2} {:>10.2} {:>12.0} {:>12.0} {:>10.1}{}",
                    m,
                    n,
                    percentile(&lat, 50.0),
                    percentile(&lat, 95.0),
                    percentile(&lat, 99.0),
                    percentile(&mat, 50.0),
                    percentile(&sol, 50.0),
                    sugg as f64 / n as f64,
                    if estimated > 0 { format!("  ({estimated} lời gọi rơi về ESTIMATED)") } else { String::new() }
                );
            }
            // 8 client đồng thời, 20 booking
            let t = std::time::Instant::now();
            let mut hs = Vec::new();
            for c in 0..8usize {
                let mut cl = client.clone();
                let (pins, drivers) = (pins.clone(), drivers.clone());
                hs.push(tokio::spawn(async move {
                    let mut lat = Vec::new();
                    for k in 0..n.max(8) / 2 {
                        let r = request(&format!("REQ-c{c}-{k}"), &pins, &drivers, 20, c * 31 + k * 5);
                        let t = std::time::Instant::now();
                        let res = cl.suggest_match(r).await.expect("SuggestMatch").into_inner();
                        lat.push((t.elapsed().as_secs_f64() * 1e3, res.quality));
                    }
                    lat
                }));
            }
            let mut lat = Vec::new();
            let mut est = 0;
            for h in hs {
                for (l, q) in h.await.expect("tác vụ") {
                    lat.push(l);
                    est += usize::from(q != geo_engine::pb::RoutingQuality::Routed as i32);
                }
            }
            lat.sort_by(f64::total_cmp);
            println!(
                "  8 client đồng thời × {} lời gọi, 20 booking: p50 {:.2} ms, p95 {:.2} ms, p99 {:.2} ms, tổng {:.2} s, {} lời gọi rơi về ESTIMATED (deadline mặc định {} ms)",
                lat.len() / 8,
                percentile(&lat, 50.0),
                percentile(&lat, 95.0),
                percentile(&lat, 99.0),
                t.elapsed().as_secs_f64(),
                est,
                GeoEngineConfig::default().default_deadline_ms
            );
            println!("  (* ma trận µs, giải µs: do máy chủ báo trong SolverStats, trung vị. Đo khi máy đang chịu tải nặng, chỉ so sánh tương đối.)");
        });
    }

    println!(
        "\nGiới hạn: Dispatch ở đây là STUB trong bộ nhớ, một tiến trình, không lưu bền, không Redis; contract test kiểm hợp đồng thông điệp và bất biến của stub, không kiểm tranh chấp đa luồng hay bền vững của Dispatch thật."
    );
    println!("uptime lúc kết thúc: {}", uptime());
    println!("{}", if all_ok { "PASS" } else { "FAIL" });
    if !all_ok {
        std::process::exit(1);
    }
}
