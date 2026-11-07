//! LOC-37: hẻm, chung cư, trung tâm thương mại, sân bay, nơi GPS vô dụng. Lớp cổng/sảnh từ OSM `entrance=*`, vùng đón bắt buộc
//! (DỮ LIỆU BỊA, `fixtures/hard-places-mau.json`), hẻm xe máy và ô tô, GPS kém trong nhà, và chuỗi lùi khi dữ liệu thiếu.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc37_hem_cong_sanh
//! Tham số: --fixture, --n N (số yêu cầu mẫu, mặc định 150)

use geo_core::{LatLng, destination, haversine_m};
use labs::p03::{mean, uptime};
use labs::p04::*;
use labs::{Args, lab_root, percentile};
use pickup::candidates::{CandidateGen, GenConfig};
use pickup::hard::{
    CandidateSource, EntranceLayer, OriginSource, ZoneKind, ZoneTable, alley_stats,
    resolve_candidates, resolve_origin,
};
use pickup::objective::ObjectiveWeights;
use pickup::rank::{DriverBelief, DriverRef, RankConfig, Ranker, Request};
use road_graph::Mode;
use route_engine::engine::Engine;
use route_engine::profile::Profile;
use route_engine::turn::{EdgeSearcher, TurnCosts};
use simulator::world::generate;
use std::cell::RefCell;

fn rank_once(
    e: &Engine,
    r: &mut Ranker,
    es: &RefCell<EdgeSearcher>,
    req: &Request,
    b: &DriverBelief,
) -> pickup::rank::Ranked {
    let tm = e.turn_model(Profile::Car, TurnCosts::urban());
    let vmax = e.costs(Profile::Car).vmax_cm_per_unit;
    let mut route = |f, t| es.borrow_mut().route(&tm, &e.xy, vmax, f, t, true).0;
    r.rank(e, req, b, &mut route)
}

fn noisy(rng: &mut fastrand::Rng, at: LatLng, sigma: f64) -> LatLng {
    let mut g = || {
        let (u1, u2) = (1.0 - rng.f64(), rng.f64());
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    };
    let (dx, dy) = (sigma * g(), sigma * g());
    destination(at, dx.atan2(dy).to_degrees(), dx.hypot(dy))
}

fn main() {
    let args = Args::from_env();
    let env = setup(&args);
    let e = &env.engine;
    println!("== LOC-37: hẻm, chung cư, TTTM, sân bay: nơi GPS vô dụng ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let seed = *EVAL_SEEDS.start();
    let sc = with_seed(&env.scenario, seed);
    let n = args.u64_or("--n", 150) as usize;
    println!(
        "{}",
        config_line(
            &env,
            &[seed],
            &format!("{n} yêu cầu mẫu; p04-v1; trên dữ liệu mô phỏng")
        )
    );

    // ---- 1. lớp cổng ----
    println!("\n-- 1. lớp cổng/sảnh từ OSM entrance=* (có sẵn trong đồ thị, chưa gắn vào cạnh) --");
    let layer = EntranceLayer::from_graph(&e.g);
    let bb = e.g.node_bbox();
    let area_km2 = haversine_m(
        LatLng::try_new(bb[0], bb[2]).expect("bbox"),
        LatLng::try_new(bb[1], bb[2]).expect("bbox"),
    ) * haversine_m(
        LatLng::try_new(bb[0], bb[2]).expect("bbox"),
        LatLng::try_new(bb[0], bb[3]).expect("bbox"),
    ) / 1e6;
    println!(
        "  {} nút entrance=* trong bbox của đồ thị (~{:.0} km²: {:.2} nút/km²). Theo loại: {:?}",
        layer.total,
        area_km2,
        layer.total as f64 / area_km2.max(1e-9),
        layer.by_kind
    );
    let mut near_car = 0usize;
    let mut near_foot = 0usize;
    let mut private = 0usize;
    for en in &e.g.entrances {
        let p = LatLng::try_new(f64::from(en.lat_e7) / 1e7, f64::from(en.lng_e7) / 1e7)
            .expect("toạ độ");
        near_car += usize::from(
            e.snap
                .nearest(&e.g, p, Mode::Car)
                .is_some_and(|c| c.dist_m <= 30.0),
        );
        near_foot += usize::from(
            e.snap
                .nearest(&e.g, p, Mode::Foot)
                .is_some_and(|c| c.dist_m <= 30.0),
        );
        private += usize::from(
            en.tags
                .iter()
                .any(|(k, v)| k == "access" && matches!(v.as_str(), "private" | "no" | "staff")),
        );
    }
    println!(
        "  trong đó cách đường xe ≤ 30 m: {} ({:.1}%); cách đường bộ ≤ 30 m: {} ({:.1}%); access=private/no/staff (không dùng làm điểm hẹn): {}",
        near_car,
        100.0 * near_car as f64 / layer.total.max(1) as f64,
        near_foot,
        100.0 * near_foot as f64 / layer.total.max(1) as f64,
        private
    );
    // khách có cổng gần
    let world = generate(e, &sc);
    let custs: Vec<_> = world.customers.iter().take(n).collect();
    let mut with_gate = 0usize;
    let mut gate_d = Vec::new();
    for c in &custs {
        if let Some((_, d, _)) = layer.nearest(c.view.true_pos, 80.0) {
            with_gate += 1;
            gate_d.push(d);
        }
    }
    println!(
        "  {} khách mẫu: {} ({:.1}%) có cổng công khai trong 80 m (khoảng cách p50 {:.0} m); còn lại không có dữ liệu cổng, chuỗi lùi ở mục 5 xử lý",
        custs.len(),
        with_gate,
        100.0 * with_gate as f64 / custs.len().max(1) as f64,
        percentile(
            &{
                let mut v = gate_d.clone();
                v.sort_by(f64::total_cmp);
                v
            },
            50.0
        )
    );

    // ---- 2. hẻm ----
    println!("\n-- 2. hẻm: xe máy vào được, ô tô thì không --");
    let st = alley_stats(&e.g);
    println!(
        "  cạnh hẻm/đường nhỏ (service=alley, service, living_street): {}; ô tô vào được {}, xe máy vào được {}, chỉ xe máy {}, ô tô vào được nhưng hẹp (< 4 m) {}",
        st.edges_total, st.car_ok, st.bike_ok, st.bike_only, st.car_slow
    );
    let mut rng = fastrand::Rng::with_seed(seed);
    let bike_only: Vec<u32> = (0..e.g.edge_count() as u32)
        .filter(|&x| {
            let ed = e.g.edge(x);
            ed.allows(Mode::Motorbike)
                && !ed.allows(Mode::Car)
                && ed.allows(Mode::Foot)
                && ed.length_m() >= 15
        })
        .collect();
    println!(
        "  cạnh xe máy đi được mà ô tô không, đi bộ được, dài ≥ 15 m: {}",
        bike_only.len()
    );
    if bike_only.is_empty() {
        println!("  (không có cạnh nào để lấy mẫu)");
    } else {
        let pts: Vec<LatLng> = (0..n.min(200))
            .map(|_| {
                let ed = bike_only[rng.usize(..bike_only.len())];
                route_engine::position::point_on_edge(&e.g, ed, 0.5)
            })
            .collect();
        println!(
            "  lấy mẫu {} điểm giữa các cạnh đó (khách đứng trong hẻm mô phỏng) và tìm điểm dừng gần nhất theo từng loại xe, đi bộ ≤ 250 m:",
            pts.len()
        );
        println!(
            "    {:<10} {:>12} {:>18} {:>12} {:>12}",
            "loại xe", "có ứng viên", "đi bộ tới điểm gần nhất TB / p95", "ứng viên TB", "không có"
        );
        for (name, prof) in [("ô tô", Profile::Car), ("xe máy", Profile::Motorbike)] {
            let mut gen_ = CandidateGen::new(
                e,
                GenConfig {
                    profile: prof,
                    ..GenConfig::default()
                },
            );
            let (mut best, mut cnt, mut none) = (Vec::new(), Vec::new(), 0usize);
            for p in &pts {
                let g = gen_.generate(e, *p, *p);
                match g
                    .candidates
                    .iter()
                    .map(|c| c.walk_est_m)
                    .min_by(f64::total_cmp)
                {
                    Some(m) => best.push(m),
                    None => none += 1,
                }
                cnt.push(g.candidates.len() as f64);
            }
            best.sort_by(f64::total_cmp);
            println!(
                "    {:<10} {:>12} {:>9.0} / {:<8.0} {:>12.1} {:>12}",
                name,
                best.len(),
                mean(&best),
                percentile(&best, 95.0),
                mean(&cnt),
                none
            );
        }
        println!(
            "  (xe máy đón sâu trong hẻm, ô tô phải đón ở miệng hẻm hoặc trên đường lớn: khách đi bộ thêm; đây là lý do loại xe là đầu vào của bộ chọn điểm đón)"
        );
    }

    // ---- 3. vùng đón bắt buộc (BỊA) ----
    println!("\n-- 3. vùng có điểm đón bắt buộc (DỮ LIỆU BỊA: fixtures/hard-places-mau.json) --");
    let zpath = lab_root().join("fixtures/hard-places-mau.json");
    let zones = ZoneTable::from_json(&std::fs::read_to_string(&zpath).expect("đọc vùng"))
        .expect("vùng hợp lệ");
    println!("  ghi chú trong file: {}", zones.note);
    for z in &zones.zones {
        println!(
            "  - {} ({:?}, bán kính {:.0} m, {} điểm đón, bịa = {})",
            z.name,
            z.kind,
            z.radius_m,
            z.bays.len(),
            z.invented
        );
    }
    let live = |seedv: u64| {
        let w = generate(e, &with_seed(&env.scenario, seedv));
        let d: Vec<DriverRef> = w
            .drivers
            .iter()
            .map(|d| DriverRef {
                pos: d.pos,
                latlng: d.pos.point(&e.g),
            })
            .collect();
        (w, d)
    };
    let es = RefCell::new(EdgeSearcher::new(e.g.edge_count()));
    for z in zones
        .zones
        .iter()
        .filter(|z| z.kind == ZoneKind::MandatoryPickup)
    {
        let center = LatLng::try_new(z.center[0], z.center[1]).expect("tâm");
        let mut walk_free = Vec::new();
        let mut walk_bay = Vec::new();
        let mut cnt = 0usize;
        let mut ranker = Ranker::new(e, ObjectiveWeights::default(), RankConfig::default());
        for sd in EVAL_SEEDS.take(6) {
            let (w, d) = live(sd);
            let belief = DriverBelief::Live(d);
            for c in w
                .customers
                .iter()
                .filter(|c| haversine_m(c.view.true_pos, center) <= z.radius_m)
            {
                let req = Request {
                    from: c.view.true_pos,
                    pin: c.view.pin,
                    dest: c.view.dropoff,
                };
                let free = rank_once(e, &mut ranker, &es, &req, &belief);
                let res =
                    resolve_candidates(e, Some(&zones), c.view.true_pos, c.view.pin, Profile::Car);
                if res.source != CandidateSource::ZoneBays || res.candidates.is_empty() {
                    continue;
                }
                let Some(f) = free.top.first() else { continue };
                let bay_walk = res
                    .candidates
                    .iter()
                    .map(|k| Ranker::walk_m(e, c.view.true_pos, k.point))
                    .fold(f64::INFINITY, f64::min);
                walk_free.push(f.walk_m);
                walk_bay.push(bay_walk);
                cnt += 1;
            }
        }
        walk_free.sort_by(f64::total_cmp);
        walk_bay.sort_by(f64::total_cmp);
        println!(
            "  «{}»: {cnt} khách mô phỏng trong vùng (6 seed). Đi bộ tới điểm J chọn tự do: TB {:.0} m, p95 {:.0} m; tới điểm đón bắt buộc gần nhất: TB {:.0} m, p95 {:.0} m, max {:.0} m",
            z.name,
            mean(&walk_free),
            percentile(&walk_free, 95.0),
            mean(&walk_bay),
            percentile(&walk_bay, 95.0),
            walk_bay.last().copied().unwrap_or(f64::NAN)
        );
        println!(
            "    chi phí của quy tắc: khách đi thêm TB {:+.0} m (ràng buộc cứng của vùng thắng J; đi bộ vượt ngưỡng 250 m KHÔNG loại điểm đón bắt buộc)",
            mean(&walk_bay) - mean(&walk_free)
        );
    }

    // ---- 4. GPS kém ----
    println!("\n-- 4. GPS kém trong nhà: nhiễu σ so với việc dùng cổng/ghim thay chỗ đứng --");
    let (w, d) = live(seed);
    let belief = DriverBelief::Live(d);
    let mut ranker = Ranker::new(e, ObjectiveWeights::default(), RankConfig::default());
    let mut rr = Ranker::new(e, ObjectiveWeights::default(), RankConfig::default());
    let reqs: Vec<(Request, LatLng)> = w
        .customers
        .iter()
        .take(n)
        .map(|c| {
            (
                Request {
                    from: c.view.true_pos,
                    pin: c.view.pin,
                    dest: c.view.dropoff,
                },
                c.view.true_pos,
            )
        })
        .collect();
    println!(
        "  hối tiếc = J thật (vị trí thật của khách) của điểm được chọn trừ J thật nhỏ nhất; «đi bộ thật» = quãng khách đi từ chỗ đứng THẬT tới điểm được chọn. Ngưỡng coi GPS là kém: 30 m."
    );
    println!(
        "  {:>6} | {:^38} | {:^38} | {:>20}",
        "σ (m)", "tin GPS như báo", "dùng cổng/ghim khi GPS kém", "nguồn vị trí (GPS/cổng/ghim)"
    );
    println!(
        "  {:>6} | {:>12} {:>12} {:>10} | {:>12} {:>12} {:>10} |",
        "", "hối tiếc TB", "đi bộ TB", "p95", "hối tiếc TB", "đi bộ TB", "p95"
    );
    let mut rng = fastrand::Rng::with_seed(seed ^ 0x6e55);
    for sigma in [5.0, 15.0, 40.0, 80.0] {
        let (mut reg_t, mut walk_t, mut reg_f, mut walk_f) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let mut src = [0usize; 3];
        for (req, truth) in &reqs {
            let oracle = rank_once(e, &mut ranker, &es, req, &belief);
            let Some(best) = oracle.top.first() else {
                continue;
            };
            let seen = noisy(&mut rng, *truth, sigma);
            let try_origin = |at: LatLng, ranker: &mut Ranker| -> Option<(f64, f64)> {
                let r = rank_once(e, ranker, &es, &Request { from: at, ..*req }, &belief);
                let top = r.top.first()?;
                let walk = Ranker::walk_m(e, *truth, top.cand.point);
                // J thật của điểm đã chọn (vị trí thật của khách)
                let tm = e.turn_model(Profile::Car, TurnCosts::urban());
                let vmax = e.costs(Profile::Car).vmax_cm_per_unit;
                let mut route = |f, t| es.borrow_mut().route(&tm, &e.xy, vmax, f, t, true).0;
                let dest_pos = e
                    .snap_point(Profile::Car, req.dest, "đích")
                    .map(|s| s.positions(e.view(Profile::Car)))
                    .unwrap_or_default();
                let mut nr = 0;
                let drivers_near: Vec<DriverRef> = match &belief {
                    DriverBelief::Live(v) => {
                        let mut v = v.clone();
                        v.sort_by(|a, b| {
                            haversine_m(a.latlng, top.cand.point)
                                .total_cmp(&haversine_m(b.latlng, top.cand.point))
                        });
                        v.truncate(1);
                        v
                    }
                    _ => Vec::new(),
                };
                let s = ranker.score_exact(
                    e,
                    &Request {
                        from: *truth,
                        ..*req
                    },
                    &drivers_near,
                    false,
                    &dest_pos,
                    &top.cand,
                    &mut route,
                    &mut nr,
                );
                Some((s.j, walk))
            };
            // J thật của điểm tốt nhất khi biết đúng chỗ khách (cùng cách tính: tài xế gần nhất theo chim bay)
            let best_true = try_origin(*truth, &mut rr).map_or(best.j, |x| x.0);
            let _ = best;
            if let Some((j, wlk)) = try_origin(seen, &mut rr) {
                reg_t.push(j - best_true);
                walk_t.push(wlk);
            }
            let o = resolve_origin(Some(&layer), seen, sigma, req.pin, 30.0);
            src[match o.source {
                OriginSource::Gps => 0,
                OriginSource::Entrance => 1,
                OriginSource::Pin => 2,
            }] += 1;
            if let Some((j, wlk)) = try_origin(o.at, &mut rr) {
                reg_f.push(j - best_true);
                walk_f.push(wlk);
            }
        }
        walk_t.sort_by(f64::total_cmp);
        walk_f.sort_by(f64::total_cmp);
        println!(
            "  {:>6.0} | {:>12.2} {:>12.1} {:>10.1} | {:>12.2} {:>12.1} {:>10.1} | {:>6}/{}/{}",
            sigma,
            mean(&reg_t),
            mean(&walk_t),
            percentile(&walk_t, 95.0),
            mean(&reg_f),
            mean(&walk_f),
            percentile(&walk_f, 95.0),
            src[0],
            src[1],
            src[2]
        );
    }
    println!(
        "  (hối tiếc có thể âm vài giây vì J thật của «biết đúng chỗ khách» cũng chỉ là xếp hạng hai tầng, không phải tối ưu toàn cục)"
    );

    // ---- 5. chuỗi lùi ----
    println!("\n-- 5. chuỗi lùi khi dữ liệu thiếu: không bao giờ lùi âm thầm --");
    let c0 = custs.first().expect("có khách");
    let (from, pin) = (c0.view.true_pos, c0.view.pin);
    let show = |name: &str, r: &pickup::hard::Resolution| {
        println!(
            "  {name}: nguồn = {:?}, {} ứng viên, cảnh báo: {:?}",
            r.source,
            r.candidates.len(),
            r.warnings
        );
    };
    show(
        "bình thường (không có vùng)",
        &resolve_candidates(e, None, from, pin, Profile::Car),
    );
    let bad_zone = ZoneTable {
        note: "bịa".into(),
        zones: vec![pickup::hard::Zone {
            name: "vùng giả có điểm đón nằm ngoài bản đồ (bịa)".into(),
            kind: ZoneKind::MandatoryPickup,
            center: [from.lat(), from.lng()],
            radius_m: 500.0,
            bays: vec![[from.lat() + 0.5, from.lng() + 0.5]],
            gps_sigma_m: 0.0,
            invented: true,
        }],
    };
    show(
        "vùng bắt buộc nhưng điểm đón không snap được",
        &resolve_candidates(e, Some(&bad_zone), from, pin, Profile::Car),
    );
    let far = destination(from, 0.0, 30_000.0);
    show(
        "khách ở ngoài bản đồ (không đường nào)",
        &resolve_candidates(e, None, far, far, Profile::Car),
    );
    let o = resolve_origin(None, from, 60.0, pin, 30.0);
    println!(
        "  GPS ±60 m, không có lớp cổng: nguồn vị trí = {:?}, cảnh báo: {:?}",
        o.source, o.warnings
    );
    let o = resolve_origin(Some(&layer), from, 60.0, pin, 30.0);
    println!(
        "  GPS ±60 m, có lớp cổng OSM (khách đầu tiên): nguồn vị trí = {:?}, cảnh báo: {:?}",
        o.source, o.warnings
    );
    println!("\nuptime lúc kết thúc: {}", uptime());
}
