//! Máy chủ gRPC `geo-engine`.
//!
//!   cargo run --release -p geo-engine -- --addr 127.0.0.1:50071            (đồ thị TP.HCM, data/hcm-graph.bin)
//!   cargo run --release -p geo-engine -- --fixture --addr 127.0.0.1:50071   (đồ thị tổng hợp nhỏ)
//! Tuỳ chọn: --graph FILE, --time-of-day (bật hồ sơ tốc độ theo giờ tổng hợp cho ô tô), --no-ch-cache
//! In ra dòng `READY <địa chỉ>` khi đã sẵn sàng nhận yêu cầu.

use geo_core::LatLng;
use geo_engine::{GeoEngineConfig, MatchingSvc, RoutingSvc};
use route_engine::engine::{Engine, EngineConfig};
use route_engine::profile::Profile;
use route_engine::traffic::TrafficModel;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |n: &str| args.iter().any(|a| a == n);
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let fixture = flag("--fixture");
    let (graph, tag) = if fixture {
        (
            road_graph::fixture::load_fixture(&root.join("fixtures/khu-mau.json"))
                .expect("nạp fixture"),
            "fixture",
        )
    } else {
        let p =
            arg(&args, "--graph").map_or_else(|| root.join("data/hcm-graph.bin"), PathBuf::from);
        (
            road_graph::io::load(&p).unwrap_or_else(|e| {
                eprintln!("LỖI: không nạp được {}: {e}", p.display());
                std::process::exit(2)
            }),
            "hcm",
        )
    };
    let addr = arg(&args, "--addr").unwrap_or_else(|| "127.0.0.1:50071".into());
    let t = Instant::now();
    let mut engine = Engine::new(
        graph,
        EngineConfig {
            ch_cache_dir: (!fixture && !flag("--no-ch-cache"))
                .then(|| root.join("rust/target/ch-cache")),
            cache_tag: tag.into(),
            ..EngineConfig::default()
        },
    );
    if flag("--time-of-day") {
        let center = if fixture {
            engine.g.center()
        } else {
            LatLng::try_new(10.7725, 106.6980).expect("hợp lệ")
        };
        let model = TrafficModel::new(&engine.g, center, if fixture { 500.0 } else { 5_000.0 });
        engine.enable_time_of_day(Profile::Car, model);
        eprintln!("hồ sơ tốc độ theo giờ TỔNG HỢP đã bật cho ô tô");
    }
    for p in [Profile::Car, Profile::Motorbike] {
        let t = Instant::now();
        engine.ch(p);
        eprintln!("CH {}: {:.2} s", p.name(), t.elapsed().as_secs_f64());
    }
    eprintln!("engine sẵn sàng sau {:.2} s", t.elapsed().as_secs_f64());
    let engine = Arc::new(engine);
    let matching = MatchingSvc::new(Arc::clone(&engine), GeoEngineConfig::default());
    let svc = RoutingSvc::new(engine, GeoEngineConfig::default());
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("LỖI: không bind được {addr}: {e}");
            std::process::exit(2)
        });
    let local = listener.local_addr().expect("địa chỉ");
    println!("READY {local}");
    tonic::transport::Server::builder()
        .add_service(svc.into_server())
        .add_service(matching.into_server())
        .serve_with_incoming_shutdown(
            tokio_stream::wrappers::TcpListenerStream::new(listener),
            async {
                let _ = tokio::signal::ctrl_c().await;
                eprintln!("nhận Ctrl-C, đang tắt");
            },
        )
        .await
        .expect("máy chủ gRPC");
}
