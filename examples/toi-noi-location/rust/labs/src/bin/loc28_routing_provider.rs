//! LOC-28: port `RoutingProvider` và các adapter (engine của lab, hình dạng adapter OSRM, chim bay × hệ số vòng).
//! Bài này chủ yếu là ADR; lab chỉ chứng minh port thay được adapter và đo độ lệch của phương án dự phòng chim bay.
//!
//! KHÔNG chạy với OSRM hay Valhalla thật: adapter OSRM đi qua transport giả (MockOsrmTransport) sinh JSON đúng schema OSRM
//! từ chính engine của lab. Việc đó kiểm tra mã dựng URL và đọc JSON, không nói gì về chất lượng của OSRM.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc28_routing_provider
//! Tham số: --queries N (mặc định 300), --seed S, --fixture

use geo_core::{LatLng, destination, haversine_m};
use labs::p03::{mean, uptime, vn};
use labs::{Args, Rand, lab_root, load_graph};
use route_engine::alt::largest_scc_nodes;
use route_engine::engine::{Engine, EngineConfig};
use route_engine::profile::Profile;
use route_engine::provider::{
    FallbackProvider, MatrixQuery, MockOsrmTransport, OsrmHttpProvider, OwnEngineProvider,
    ProviderError, Quality, RouteAnswer, RouteQuery, RoutingProvider, StraightLineProvider,
    contract_violations, valhalla_route_request,
};
use std::sync::Arc;

const SEED: u64 = 20_261_106;

fn pct(v: &mut [f64], p: f64) -> f64 {
    v.sort_by(f64::total_cmp);
    v[((p / 100.0) * v.len() as f64).ceil().max(1.0) as usize - 1]
}

/// Adapter luôn lỗi, để thấy chuỗi dự phòng hoạt động.
struct Down;
impl RoutingProvider for Down {
    fn name(&self) -> &str {
        "down"
    }
    fn route(&self, _: &RouteQuery) -> Result<RouteAnswer, ProviderError> {
        Err(ProviderError::Transport(
            "connection refused (mô phỏng)".into(),
        ))
    }
    fn matrix(
        &self,
        _: &MatrixQuery,
    ) -> Result<route_engine::provider::MatrixAnswer, ProviderError> {
        Err(ProviderError::Transport(
            "connection refused (mô phỏng)".into(),
        ))
    }
}

fn main() {
    let args = Args::from_env();
    let seed = args.u64_or("--seed", SEED);
    let nq = args.u64_or("--queries", 300) as usize;
    let (g, source) = load_graph(&args);
    println!("== LOC-28: port RoutingProvider và adapter ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    println!(
        "đồ thị: {source}: {} nút, {} cạnh; seed {seed}; {nq} cặp điểm",
        vn(g.node_count() as u64),
        vn(g.edge_count() as u64)
    );
    println!(
        "CHƯA CHẠY với OSRM hay Valhalla thật (máy viết không có server OSRM/Valhalla và không có bản đồ đã biên dịch cho chúng).\n"
    );
    let cache = (!args.flag("--fixture")).then(|| lab_root().join("rust/target/ch-cache"));
    let eng = Arc::new(Engine::new(
        g,
        EngineConfig {
            ch_cache_dir: cache,
            cache_tag: "hcm".into(),
            ..EngineConfig::default()
        },
    ));
    let g = &eng.g;
    let scc = largest_scc_nodes(g, &eng.rev, &eng.costs(Profile::Car).w);
    let nodes: Vec<u32> = (0..g.node_count() as u32)
        .filter(|&v| scc[v as usize])
        .collect();
    let mut rng = Rand::new(seed);
    // điểm GPS cách nút 5–30 m theo hướng ngẫu nhiên
    let mut pt = || -> LatLng {
        destination(
            g.node_latlng(nodes[rng.usize(nodes.len())]),
            rng.range(0.0, 360.0),
            rng.range(5.0, 30.0),
        )
    };
    let pairs: Vec<(LatLng, LatLng)> = (0..nq).map(|_| (pt(), pt())).collect();

    let own = OwnEngineProvider {
        engine: Arc::clone(&eng),
    };
    let osrm = OsrmHttpProvider::new(
        "http://osrm.invalid:5000",
        MockOsrmTransport::new(Arc::clone(&eng)),
    );
    let line = StraightLineProvider::default();

    println!("-- 1. hình dạng yêu cầu của từng adapter --");
    let (a, b) = pairs[0];
    let q = RouteQuery {
        from: a,
        to: b,
        profile: Profile::Car,
        depart_sec_of_day: None,
    };
    println!("  OSRM route : {}", osrm.route_url(&q));
    println!(
        "  OSRM table : {}",
        osrm.table_url(&MatrixQuery {
            sources: vec![a, b],
            targets: vec![b, a],
            profile: Profile::Car,
            depart_sec_of_day: None
        })
    );
    println!(
        "  Valhalla POST /route, thân JSON: {}",
        valhalla_route_request(&q)
    );
    println!(
        "  (OSRM dùng URL, toạ độ lng,lat; Valhalla dùng POST JSON, lat/lon; cả hai không có profile xe máy sẵn trong OSRM: adapter ánh xạ xe máy sang car và ghi chú là xấp xỉ)"
    );

    println!(
        "\n-- 2. cùng một bộ kiểm tra hợp đồng cho mọi adapter ({} cặp, ô tô, dung sai snap 60 m) --",
        pairs.len().min(50)
    );
    for p in [&own as &dyn RoutingProvider, &osrm, &line] {
        let mut bad = 0;
        let mut first = String::new();
        for &(a, b) in pairs.iter().take(50) {
            let v = contract_violations(p, a, b, Profile::Car, 60.0);
            if !v.is_empty() {
                bad += 1;
                if first.is_empty() {
                    first = v.join("; ");
                }
            }
        }
        println!("  {:<14} cặp vi phạm hợp đồng: {bad}/50 {first}", p.name());
    }

    println!(
        "\n-- 3. adapter OSRM (qua transport giả) so với engine của lab: phải gần như trùng khớp --"
    );
    let (mut dd, mut dm, mut worst_geo) = (Vec::new(), Vec::new(), 0.0f64);
    let mut compared = 0;
    for &(a, b) in &pairs {
        let q = RouteQuery {
            from: a,
            to: b,
            profile: Profile::Car,
            depart_sec_of_day: None,
        };
        let (Ok(x), Ok(y)) = (own.route(&q), osrm.route(&q)) else {
            continue;
        };
        compared += 1;
        dd.push((x.duration_s - y.duration_s).abs());
        dm.push((x.distance_m - y.distance_m).abs());
        if let (Some(p), Some(r)) = (x.geometry.first(), y.geometry.first()) {
            worst_geo = worst_geo.max(haversine_m(*p, *r));
        }
    }
    println!(
        "  {compared} cặp: chênh lệch thời gian trung vị {:.3} s, lớn nhất {:.3} s; chênh quãng đường trung vị {:.3} m, lớn nhất {:.3} m; điểm đầu hình học lệch tối đa {worst_geo:.2} m",
        pct(&mut dd.clone(), 50.0),
        pct(&mut dd, 100.0),
        pct(&mut dm.clone(), 50.0),
        pct(&mut dm, 100.0)
    );
    println!(
        "  (không trùng tuyệt đối vì toạ độ đi qua URL bị làm tròn 6 chữ số thập phân (~0,1 m) và hình học qua polyline precision 5; điểm snap trượt vài chục cm dọc cạnh nên thời gian lệch vài giây ở ca tệ nhất)"
    );
    println!(
        "  số lần transport giả được gọi: {}",
        osrm.transport
            .calls
            .load(std::sync::atomic::Ordering::Relaxed)
    );

    println!("\n-- 4. dự phòng chim bay × hệ số vòng so với engine (số liệu cho LOC-29) --");
    // hiệu chỉnh trên nửa đầu của mẫu, đánh giá trên nửa sau (ngoài mẫu)
    struct Obs {
        crow: f64,
        route_m: f64,
        dur_s: f64,
    }
    let mut obs = Vec::new();
    for &(a, b) in &pairs {
        let q = RouteQuery {
            from: a,
            to: b,
            profile: Profile::Car,
            depart_sec_of_day: None,
        };
        if let Ok(x) = own.route(&q) {
            let crow = haversine_m(a, b);
            if crow >= 300.0 {
                obs.push(Obs {
                    crow,
                    route_m: x.distance_m,
                    dur_s: x.duration_s,
                });
            }
        }
    }
    let half = obs.len() / 2;
    let (train, test) = obs.split_at(half);
    let mut ratio_d: Vec<f64> = obs.iter().map(|o| o.route_m / o.crow).collect();
    println!(
        "  {} cặp có chim bay ≥ 300 m. Hệ số vòng quãng đường (đường ô tô / chim bay): trung bình {:.3}, p5 {:.3}, trung vị {:.3}, p95 {:.3}",
        obs.len(),
        mean(&ratio_d),
        pct(&mut ratio_d, 5.0),
        pct(&mut ratio_d, 50.0),
        pct(&mut ratio_d, 95.0)
    );
    let detour_cal = pct(
        &mut train.iter().map(|o| o.route_m / o.crow).collect::<Vec<_>>(),
        50.0,
    );
    let speed_cal = pct(
        &mut train
            .iter()
            .map(|o| o.route_m / o.dur_s * 3.6)
            .collect::<Vec<_>>(),
        50.0,
    );
    println!(
        "  hiệu chỉnh trên {} cặp đầu: hệ số vòng trung vị {detour_cal:.3}, tốc độ trên đường trung vị {speed_cal:.1} km/h",
        train.len()
    );
    let eval = |name: &str, p: &StraightLineProvider| {
        let mut err: Vec<f64> = test
            .iter()
            .map(|o| {
                let est = o.crow * p.detour / (p.speed_kmh[0] / 3.6);
                100.0 * (est - o.dur_s) / o.dur_s
            })
            .collect();
        let mut abs: Vec<f64> = err.iter().map(|e| e.abs()).collect();
        println!(
            "  {name:<46} trên {} cặp sau: sai số trung bình {:+.1}%, |sai số| trung vị {:.1}%, p95 {:.1}%",
            test.len(),
            mean(&err),
            pct(&mut abs, 50.0),
            pct(&mut abs, 95.0)
        );
        err.clear();
    };
    eval(
        "đoán tay (hệ số vòng 1,4; 22 km/h)",
        &StraightLineProvider {
            detour: 1.4,
            speed_kmh: [22.0, 26.0, 4.8],
        },
    );
    eval(
        "hiệu chỉnh trên nửa đầu của chính mẫu này",
        &StraightLineProvider {
            detour: detour_cal,
            speed_kmh: [speed_cal, speed_cal, 4.9],
        },
    );
    eval(
        "mặc định của StraightLineProvider (1,3; 35 km/h)",
        &StraightLineProvider::default(),
    );
    println!(
        "  (engine ở đây cũng là mô hình với tốc độ giả định, nên đây là độ lệch giữa hai mô hình, không phải so với đường thật)"
    );

    println!("\n-- 5. chuỗi dự phòng: adapter chính hỏng thì dùng chim bay --");
    let chain = FallbackProvider {
        primary: Box::new(Down),
        secondary: Box::new(StraightLineProvider::default()),
    };
    let r = chain.route(&q).expect("dự phòng trả lời");
    println!(
        "  primary «down» lỗi → trả lời bởi «{}», chất lượng {:?}, {:.0} s, {:.0} m",
        r.provider, r.quality, r.duration_s, r.distance_m
    );
    assert_eq!(r.quality, Quality::Estimated);
    let chain2 = FallbackProvider {
        primary: Box::new(OwnEngineProvider {
            engine: Arc::clone(&eng),
        }),
        secondary: Box::new(StraightLineProvider::default()),
    };
    let r = chain2.route(&q).expect("engine trả lời");
    println!(
        "  primary «own-engine» sống → trả lời bởi «{}», chất lượng {:?}, {:.0} s",
        r.provider, r.quality, r.duration_s
    );
    println!("\nuptime lúc kết thúc: {}", uptime());
}
