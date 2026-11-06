//! LOC-57: một dịch vụ cho Route, Matrix, SuggestMatch, SuggestPickup (và UpdateRendezvous) qua `geo_app::GeoApp`, hai adapter `DriverStore`.
//!
//! Chạy từ gốc repo:
//!   cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc57_dich_vu_hop_nhat -- --fixture
//! Tham số: --fixture (đồ thị tổng hợp nhỏ; bỏ cờ để dùng HCM), --redis URL (mặc định redis://127.0.0.1:6391).

use geo_app::degrade::GovernorConfig;
use geo_app::ports::{DriverStore, MapSource};
use geo_app::{AppConfig, GeoApp, InMemoryDriverStore, StaticMap};
use geo_core::LatLng;
use geo_engine::adapters::RedisDriverStore;
use geo_engine::pb::matching_service_client::MatchingServiceClient;
use geo_engine::pb::pickup_service_client::PickupServiceClient;
use geo_engine::pb::routing_service_client::RoutingServiceClient;
use geo_engine::pb::{
    DriverCandidate, LatLngE7, MatchPolicy, MatrixRequest, PendingRequest, RouteRequest,
    RoutingProfile, RoutingQuality, SuggestMatchRequest, SuggestPickupRequest,
    UpdateRendezvousRequest,
};
use geo_engine::{GeoEngineConfig, UnifiedSvc};
use labs::p03::uptime;
use labs::p05::setup;
use labs::{Args, Rand};
use route_engine::profile::Profile;
use std::sync::Arc;

fn e7(p: LatLng) -> Option<LatLngE7> {
    Some(LatLngE7 {
        lat_e7: (p.lat() * 1e7).round() as i32,
        lng_e7: (p.lng() * 1e7).round() as i32,
    })
}
fn q(r: i32) -> &'static str {
    match RoutingQuality::try_from(r) {
        Ok(RoutingQuality::Routed) => "ROUTED",
        Ok(RoutingQuality::Cached) => "CACHED",
        Ok(RoutingQuality::Estimated) => "ESTIMATED",
        _ => "?",
    }
}

#[tokio::main]
async fn main() {
    let args = Args::from_env();
    println!(
        "== LOC-57: một dịch vụ, bốn việc, hai adapter DriverStore (trên dữ liệu mô phỏng) =="
    );
    println!("uptime lúc bắt đầu: {}", uptime());
    let (engine, info, _sc, fixture) = setup(&args);
    let engine = Arc::new(engine);
    engine.ch(Profile::Car);
    let g = &engine.g;
    println!("đồ thị: {info}");

    // tài xế: cùng một tập vị trí cho cả hai adapter
    let n_drv = if fixture { 8 } else { 60 };
    let mut rng = Rand::new(57);
    let positions: Vec<LatLng> = (0..n_drv)
        .map(|_| g.node_latlng(rng.usize(g.node_count()) as u32))
        .collect();
    let mem: Arc<dyn DriverStore> = Arc::new(InMemoryDriverStore::new(1000));
    let url = args.value("--redis").unwrap_or("redis://127.0.0.1:6391");
    let redis = RedisDriverStore::connect(url, "toi-noi-location:p07:drivers", 20_000);
    println!("\n-- 1. cổng DriverStore, hai adapter, cùng dữ liệu --");
    for (i, p) in positions.iter().enumerate() {
        mem.upsert(i as u32, *p).expect("memory");
    }
    let probe = positions[0];
    let want: Vec<u32> = mem
        .nearest(probe, 5)
        .expect("memory")
        .iter()
        .map(|n| n.id)
        .collect();
    println!(
        "  memory-h3  : nearest(5) quanh tài xế 0 = {want:?}; len = {}",
        mem.len().unwrap()
    );
    let store: Arc<dyn DriverStore> = match redis {
        Ok(r) => {
            r.clear().expect("xoá key của lab");
            for (i, p) in positions.iter().enumerate() {
                r.upsert(i as u32, *p).expect("redis");
            }
            let got: Vec<u32> = r
                .nearest(probe, 5)
                .expect("redis")
                .iter()
                .map(|n| n.id)
                .collect();
            println!(
                "  redis-geo  : nearest(5) quanh tài xế 0 = {got:?}; len = {}",
                r.len().unwrap()
            );
            let pos = r.position(0).expect("geopos").expect("có vị trí");
            println!(
                "  hai adapter {} về 5 tài xế gần nhất; vị trí Redis lệch bộ nhớ {:.2} m (geohash 52 bit)",
                if got == want { "KHỚP" } else { "KHÁC NHAU" },
                geo_core::haversine_m(pos, probe)
            );
            r.clear().expect("dọn key của lab");
            for (i, p) in positions.iter().enumerate() {
                r.upsert(i as u32, *p).expect("redis");
            }
            Arc::new(r)
        }
        Err(e) => {
            println!("  redis-geo  : KHÔNG CHẠY ({e}); dịch vụ dùng adapter trong bộ nhớ");
            Arc::clone(&mem)
        }
    };
    println!("  dịch vụ dưới đây dùng adapter: {}", store.name());

    // dịch vụ
    let maps: Arc<dyn MapSource> = Arc::new(StaticMap::new("KM", "v1", Arc::clone(&engine)));
    let app = Arc::new(GeoApp::new(
        maps,
        store,
        AppConfig {
            default_city: "KM".into(),
            ..AppConfig::default()
        },
    ));
    let svc = UnifiedSvc::new(app, GeoEngineConfig::default(), GovernorConfig::default());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    let s = svc.clone();
    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(s.routing_server())
            .add_service(s.matching_server())
            .add_service(s.pickup_server())
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
            .expect("serve");
    });
    let ch = tonic::transport::Endpoint::from_shared(format!("http://{addr}"))
        .expect("uri")
        .connect()
        .await
        .expect("kết nối");
    println!(
        "\n-- 2. một máy chủ gRPC {addr} phục vụ RoutingService + MatchingService + PickupService --"
    );
    let mut rt = RoutingServiceClient::new(ch.clone());
    let mut mt = MatchingServiceClient::new(ch.clone());
    let mut pk = PickupServiceClient::new(ch.clone());

    let c = g.center();
    let off =
        |dlat: f64, dlng: f64| LatLng::try_new(c.lat() + dlat, c.lng() + dlng).expect("hợp lệ");
    let (a, b) = (off(-0.002, -0.004), off(0.003, 0.005));
    let r = rt
        .route(RouteRequest {
            from: e7(a),
            to: e7(b),
            profile: RoutingProfile::Car as i32,
            depart_sec_of_day: None,
            want_geometry: false,
        })
        .await
        .expect("route")
        .into_inner();
    println!(
        "  Route            : {} ms, {} m, {}, nguồn {}",
        r.duration_ms,
        r.distance_m,
        q(r.quality),
        r.source
    );
    let m = rt
        .matrix(MatrixRequest {
            sources: vec![e7(a).unwrap()],
            targets: vec![e7(b).unwrap(), e7(off(0.001, 0.001)).unwrap()],
            profile: RoutingProfile::Car as i32,
            depart_sec_of_day: None,
        })
        .await
        .expect("matrix")
        .into_inner();
    println!(
        "  Matrix           : {}×{} = {:?} ms, {}, nguồn {}",
        m.rows,
        m.cols,
        m.duration_ms,
        q(m.quality),
        m.source
    );

    let drv = |i: usize| DriverCandidate {
        driver_id: format!("DR-{:03}", i + 1),
        position: e7(positions[i]),
        state_version: i as u64 + 1,
        idle_since_ms: 0,
        busy_remaining_ms: None,
    };
    let req = SuggestMatchRequest {
        request_id: "REQ-57".into(),
        city_code: "KM".into(),
        now_ms: 100_000,
        policy: Some(MatchPolicy::default()),
        snapshot_version: 1,
        pending: vec![
            PendingRequest {
                booking_id: "TN-0001".into(),
                pickup: e7(a),
                requested_at_ms: 90_000,
                waited_ms: None,
            },
            PendingRequest {
                booking_id: "TN-0002".into(),
                pickup: e7(b),
                requested_at_ms: 92_000,
                waited_ms: None,
            },
        ],
        drivers: (0..n_drv.min(6)).map(drv).collect(),
        suggestion_ttl_ms: None,
        compute_budget_ms: None,
    };
    let s = mt
        .suggest_match(req)
        .await
        .expect("suggest_match")
        .into_inner();
    println!(
        "  SuggestMatch     : {} gợi ý, {}, thuật toán {}",
        s.suggestions.len(),
        q(s.quality),
        s.stats.as_ref().map_or("?", |x| x.algo.as_str())
    );
    for x in &s.suggestions {
        println!(
            "                     {} ← {} ETA {} ms (chi phí {}, hết hạn {})",
            x.booking_id, x.driver_id, x.eta_ms, x.cost, x.valid_until_ms
        );
    }
    let p = pk
        .suggest_pickup(SuggestPickupRequest {
            request_id: "PK-57".into(),
            city_code: "KM".into(),
            customer: e7(a),
            pin: e7(a),
            dest: e7(b),
        })
        .await
        .expect("suggest_pickup")
        .into_inner();
    println!(
        "  SuggestPickup    : {} lựa chọn, J tốt nhất {:.1} s so với điểm ghim {}, {} tài xế lấy từ DriverStore, bản đồ {}, {}, nguồn {}",
        p.options.len(),
        p.options[0].j,
        p.pin_j.map_or("n/a".to_string(), |v| format!("{v:.1} s")),
        p.drivers_used,
        p.map_version,
        q(p.quality),
        p.source
    );
    for (i, o) in p.options.iter().enumerate() {
        println!(
            "                     #{} đi bộ {} m, lái {:.0} s, chuyến {:.0} s, J {:.1}{}",
            i + 1,
            o.walk_m,
            o.drive_s,
            o.trip_s,
            o.j,
            if o.is_pin { " (điểm ghim)" } else { "" }
        );
    }
    let u = pk
        .update_rendezvous(UpdateRendezvousRequest {
            request_id: "RV-57".into(),
            city_code: "KM".into(),
            customer: e7(a),
            driver: e7(positions[0]),
            current_point: p.options[0].point,
            pin: e7(a),
            dest: e7(b),
        })
        .await
        .expect("rendezvous")
        .into_inner();
    println!(
        "  UpdateRendezvous : hành động {:?}, J hiện tại {:?}, J tốt nhất {:?}, bậc suy giảm {}",
        geo_engine::pb::RendezvousAction::try_from(u.action).ok(),
        u.j_current.map(|v| (v * 10.0).round() / 10.0),
        u.j_best.map(|v| (v * 10.0).round() / 10.0),
        u.degrade_level
    );

    println!("\n-- 3. chỗ port chưa che hết (nói thẳng) --");
    println!(
        "  Route và Matrix chỉ dùng cổng RoutingProvider; SuggestMatch và SuggestPickup lấy `Engine` từ MapSnapshot vì làm việc trên vị trí có hướng,"
    );
    println!(
        "  thứ RoutingProvider (điểm tới điểm) không diễn đạt được. Muốn thay engine bằng OSRM thì hai use case đó phải đổi."
    );
    println!("\n-- 4. số đếm của dịch vụ sau các lời gọi trên (rút từ /metrics) --");
    for l in svc.metrics_text().lines().filter(|l| {
        l.starts_with("geo_requests_total")
            || l.starts_with("geo_degrade")
            || l.starts_with("walk_m_count")
            || l.starts_with("deadhead_m_count")
    }) {
        println!("  {l}");
    }
    println!("uptime lúc kết thúc: {}", uptime());
}
