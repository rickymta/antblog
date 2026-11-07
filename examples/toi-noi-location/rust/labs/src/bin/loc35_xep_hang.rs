//! LOC-35: xếp hạng điểm đón trong ngân sách độ trễ. Tính J cho mọi ứng viên từ ma trận, trả top 3, đặt và đo ngân sách p99,
//! xử lý «chưa biết tài xế» bằng phân bố quanh đó, và kiểm bất biến «kết quả không tệ hơn điểm ghim».
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc35_xep_hang
//! Tham số: --fixture, --n N (số yêu cầu, mặc định 150), --threads N (luồng song song cho đo tải, mặc định 6)

use cpu_time::ThreadTime;
use geo_core::haversine_m;
use labs::p03::{mean, uptime};
use labs::p04::*;
use labs::{Args, percentile};
use pickup::objective::ObjectiveWeights;
use pickup::rank::{DriverBelief, DriverModel, DriverRef, RankConfig, Ranker, Request};
use route_engine::engine::Engine;
use route_engine::profile::Profile;
use route_engine::turn::{EdgeSearcher, TurnCosts};
use simulator::world::generate;
use std::cell::RefCell;
use std::time::{Duration, Instant};

/// Ngân sách độ trễ cho một lần xếp hạng (một phần của Quote p95 < 500 ms, kiến trúc §14.1): 120 ms cho tầng 2.
const BUDGET_MS: u64 = 120;

fn run_one(
    e: &Engine,
    ranker: &mut Ranker,
    es: &RefCell<EdgeSearcher>,
    req: &Request,
    belief: &DriverBelief,
) -> pickup::rank::Ranked {
    let tm = e.turn_model(Profile::Car, TurnCosts::urban());
    let vmax = e.costs(Profile::Car).vmax_cm_per_unit;
    let mut route = |f, t| es.borrow_mut().route(&tm, &e.xy, vmax, f, t, true).0;
    ranker.rank(e, req, belief, &mut route)
}

/// Mục 3: hối tiếc của từng mức hiểu biết về tài xế. Cờ `--belief-only` chỉ chạy mục này.
fn belief_section(
    e: &Engine,
    drivers: &[DriverRef],
    reqs: &[Request],
    es: &RefCell<EdgeSearcher>,
    live: &DriverBelief,
) {
    println!("\n-- 3. chưa biết tài xế: phân bố quanh đó so với biết tài xế --");
    println!(
        "  «J thật» của một ứng viên = J chính xác (A* theo cạnh) với tài xế mà bộ ghép baseline sẽ gán cho ĐIỂM đó (rảnh gần nhất theo chim bay). Hối tiếc = J thật của điểm được chọn trừ J thật nhỏ nhất trong mọi ứng viên."
    );
    // J thật cho mọi ứng viên của mỗi yêu cầu
    let truth_cfg = RankConfig {
        stage2: false,
        ..RankConfig::default()
    };
    let belief_modes: Vec<(String, DriverBelief, DriverModel)> = vec![
        (
            "biết tài xế rảnh, bộ ghép = gần điểm nhất (mặc định)".into(),
            live.clone(),
            DriverModel::NearestCrow,
        ),
        (
            "biết tài xế rảnh, giả định ghép nhanh nhất (4 người)".into(),
            live.clone(),
            DriverModel::BestTime,
        ),
        (
            "biết đúng 1 tài xế gần khách nhất".into(),
            DriverBelief::Live(vec![]),
            DriverModel::BestTime,
        ),
        (
            "phân bố quanh đó, bán kính 800 m".into(),
            DriverBelief::Prior {
                n: 8,
                radius_m: 800.0,
            },
            DriverModel::BestTime,
        ),
        (
            "phân bố quanh đó, bán kính 1.500 m".into(),
            DriverBelief::Prior {
                n: 8,
                radius_m: 1500.0,
            },
            DriverModel::BestTime,
        ),
        (
            "phân bố quanh đó, bán kính 3.000 m".into(),
            DriverBelief::Prior {
                n: 8,
                radius_m: 3000.0,
            },
            DriverModel::BestTime,
        ),
        (
            "không biết gì về tài xế (T_drive = 0)".into(),
            DriverBelief::Unknown,
            DriverModel::BestTime,
        ),
    ];
    let mut reg: Vec<Vec<f64>> = vec![Vec::new(); belief_modes.len() + 1];
    let mut hit: Vec<usize> = vec![0; belief_modes.len() + 1];
    let mut counted = 0usize;
    let mut truth_ranker = Ranker::new(e, ObjectiveWeights::default(), truth_cfg);
    let mut rankers: Vec<Ranker> = (0..belief_modes.len())
        .map(|i| {
            Ranker::new(
                e,
                ObjectiveWeights::default(),
                RankConfig {
                    refine_top: 4,
                    driver_model: belief_modes[i].2,
                    ..RankConfig::default()
                },
            )
        })
        .collect();
    let m = reqs.len().min(80);
    for rq in reqs.iter().take(m) {
        // ứng viên: dùng kết quả tầng 1 để liệt kê, rồi chấm J thật cho từng ứng viên
        let list = run_one(e, &mut truth_ranker, es, rq, live);
        let cands: Vec<_> = list.all.iter().map(|s| s.cand).collect();
        let dest_pos: Vec<_> = e
            .snap_point(Profile::Car, rq.dest, "đích")
            .map(|s| s.positions(e.view(Profile::Car)))
            .unwrap_or_default();
        let tm = e.turn_model(Profile::Car, TurnCosts::urban());
        let vmax = e.costs(Profile::Car).vmax_cm_per_unit;
        let mut route = |f, t| es.borrow_mut().route(&tm, &e.xy, vmax, f, t, true).0;
        let mut nr = 0;
        let mut truth: Vec<(pickup::rank::Scored, f64)> = Vec::new();
        for c in &cands {
            let near = drivers
                .iter()
                .min_by(|a, b| {
                    haversine_m(a.latlng, c.point).total_cmp(&haversine_m(b.latlng, c.point))
                })
                .copied();
            let Some(d) = near else { continue };
            let s = truth_ranker.score_exact(e, rq, &[d], false, &dest_pos, c, &mut route, &mut nr);
            if s.j.is_finite() {
                truth.push((s, s.j));
            }
        }
        let best = truth.iter().map(|t| t.1).fold(f64::INFINITY, f64::min);
        if !best.is_finite() {
            continue;
        }
        counted += 1;
        let truth_of = |pos: route_engine::position::Position| -> Option<f64> {
            truth
                .iter()
                .find(|(s, _)| s.cand.pos.edge == pos.edge && s.cand.pos.frac == pos.frac)
                .map(|t| t.1)
        };
        for (i, (_, b, _)) in belief_modes.iter().enumerate() {
            let belief = if i == 2 {
                // một tài xế gần nhất theo chim bay tới chỗ khách
                let d = drivers
                    .iter()
                    .min_by(|a, b| {
                        haversine_m(a.latlng, rq.from).total_cmp(&haversine_m(b.latlng, rq.from))
                    })
                    .copied()
                    .expect("tài xế");
                DriverBelief::Live(vec![d])
            } else {
                b.clone()
            };
            let r = run_one(e, &mut rankers[i], es, rq, &belief);
            if let Some(top) = r.top.first()
                && let Some(t) = truth_of(top.cand.pos)
            {
                reg[i].push(t - best);
                hit[i] += usize::from((t - best).abs() < 1e-6);
            }
        }
        // đối chứng: điểm ghim (baseline)
        let pin_best = truth
            .iter()
            .filter(|(s, _)| s.cand.is_pin)
            .map(|t| t.1)
            .fold(f64::INFINITY, f64::min);
        if pin_best.is_finite() {
            let i = belief_modes.len();
            reg[i].push(pin_best - best);
            hit[i] += usize::from((pin_best - best).abs() < 1e-6);
        }
    }
    println!(
        "  {counted} yêu cầu; {:<42} {:>10} {:>14} {:>10} {:>10}",
        "chế độ", "chọn đúng", "hối tiếc TB (s)", "p95", "max"
    );
    for i in 0..=belief_modes.len() {
        reg[i].sort_by(f64::total_cmp);
        let name = if i < belief_modes.len() {
            belief_modes[i].0.clone()
        } else {
            "đối chứng: luôn chọn điểm ghim".into()
        };
        println!(
            "  {:>14} {:<42} {:>9.1}% {:>14.2} {:>10.2} {:>10.2}",
            "",
            name,
            100.0 * hit[i] as f64 / counted.max(1) as f64,
            mean(&reg[i]),
            percentile(&reg[i], 95.0),
            reg[i].last().copied().unwrap_or(f64::NAN)
        );
    }
}

fn main() {
    let args = Args::from_env();
    let env = setup(&args);
    let e = &env.engine;
    println!("== LOC-35: xếp hạng điểm đón trong ngân sách độ trễ ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let seed = *EVAL_SEEDS.start();
    let sc = with_seed(&env.scenario, seed);
    let n = args.u64_or("--n", 150) as usize;
    println!(
        "{}",
        config_line(
            &env,
            &[seed],
            &format!(
                "{n} yêu cầu mẫu và 120 tài xế rảnh của seed, trên dữ liệu mô phỏng; p04-v1; Apple Silicon, --release, đo thời gian khi máy chịu tải nặng"
            )
        )
    );
    let world = generate(e, &sc);
    let drivers: Vec<DriverRef> = world
        .drivers
        .iter()
        .map(|d| DriverRef {
            pos: d.pos,
            latlng: d.pos.point(&e.g),
        })
        .collect();
    let reqs: Vec<Request> = world
        .customers
        .iter()
        .take(n)
        .map(|c| Request {
            from: c.view.true_pos,
            pin: c.view.pin,
            dest: c.view.dropoff,
        })
        .collect();
    let live = DriverBelief::Live(drivers.clone());
    let es = RefCell::new(EdgeSearcher::new(e.g.edge_count()));
    // làm nóng bộ nhớ đệm (CH nạp lười)
    let _ = run_one(
        e,
        &mut Ranker::new(e, ObjectiveWeights::default(), RankConfig::default()),
        &es,
        &reqs[0],
        &live,
    );

    if args.flag("--belief-only") {
        belief_section(e, &drivers, &reqs, &es, &live);
        println!("\nuptime lúc kết thúc: {}", uptime());
        return;
    }

    println!("\n-- 1. độ trễ một lần xếp hạng, một luồng (µs) --");
    println!(
        "  {:<44} {:>9} {:>9} {:>9} {:>9} | {:>9} {:>9} {:>9} {:>9} | {:>9}",
        "chế độ", "tường p50", "p95", "p99", "max", "CPU p50", "p95", "p99", "max", "tuyến A*"
    );
    let modes: Vec<(&str, RankConfig)> = vec![
        (
            "chỉ ma trận (tầng 1)",
            RankConfig {
                stage2: false,
                ..RankConfig::default()
            },
        ),
        (
            "hai tầng, A* top 4 + ghim (mặc định)",
            RankConfig::default(),
        ),
        (
            "hai tầng, A* top 8 + ghim",
            RankConfig {
                refine_top: 8,
                ..RankConfig::default()
            },
        ),
        (
            "hai tầng, top 8 có hạn chót 120 ms cho tầng 2",
            RankConfig {
                refine_top: 8,
                deadline: Some(Duration::from_millis(BUDGET_MS)),
                ..RankConfig::default()
            },
        ),
    ];
    for (name, cfg) in &modes {
        let mut ranker = Ranker::new(e, ObjectiveWeights::default(), cfg.clone());
        let (mut t, mut tc) = (Vec::new(), Vec::new());
        let (mut routes, mut degraded) = (0usize, 0usize);
        for rq in &reqs {
            let (s, c) = (Instant::now(), ThreadTime::now());
            let r = run_one(e, &mut ranker, &es, rq, &live);
            tc.push(c.elapsed().as_micros() as f64);
            t.push(s.elapsed().as_micros() as f64);
            routes += r.stats.n_edge_routes;
            degraded += usize::from(r.stats.degraded);
        }
        t.sort_by(f64::total_cmp);
        tc.sort_by(f64::total_cmp);
        println!(
            "  {:<44} {:>9.0} {:>9.0} {:>9.0} {:>9.0} | {:>9.0} {:>9.0} {:>9.0} {:>9.0} | {:>9.1}{}",
            name,
            percentile(&t, 50.0),
            percentile(&t, 95.0),
            percentile(&t, 99.0),
            t.last().copied().unwrap_or(f64::NAN),
            percentile(&tc, 50.0),
            percentile(&tc, 95.0),
            percentile(&tc, 99.0),
            tc.last().copied().unwrap_or(f64::NAN),
            routes as f64 / reqs.len() as f64,
            if cfg.deadline.is_some() {
                format!("   (bị cắt hạn chót: {degraded}/{})", reqs.len())
            } else {
                String::new()
            }
        );
    }
    println!(
        "  (µs; mỗi dòng {} yêu cầu liên tiếp, một luồng. «tường» = đồng hồ treo tường, bị phóng đại vì máy dùng chung đang chịu tải nặng; «CPU» = thời gian CPU của luồng (ThreadTime), ít bị nhiễu bởi việc luồng bị đẩy ra khỏi CPU nhưng vẫn phụ thuộc cache và xung nhịp; chỉ so sánh tương đối giữa các dòng)",
        reqs.len()
    );

    println!("\n-- 2. đo dưới tải: nhiều luồng cùng xếp hạng (hai tầng mặc định) --");
    let threads = args.u64_or("--threads", 6) as usize;
    let lat: Vec<Vec<(f64, f64)>> = {
        use rayon::prelude::*;
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("pool");
        pool.install(|| {
            (0..threads)
                .into_par_iter()
                .map(|k| {
                    let es = RefCell::new(EdgeSearcher::new(e.g.edge_count()));
                    let mut ranker =
                        Ranker::new(e, ObjectiveWeights::default(), RankConfig::default());
                    reqs.iter()
                        .cycle()
                        .skip(k * 7)
                        .take(reqs.len())
                        .map(|rq| {
                            let (s, c) = (Instant::now(), ThreadTime::now());
                            let _ = run_one(e, &mut ranker, &es, rq, &live);
                            (
                                s.elapsed().as_micros() as f64,
                                c.elapsed().as_micros() as f64,
                            )
                        })
                        .collect::<Vec<(f64, f64)>>()
                })
                .collect()
        })
    };
    let flat: Vec<(f64, f64)> = lat.into_iter().flatten().collect();
    let mut all: Vec<f64> = flat.iter().map(|x| x.0).collect();
    let mut cpu: Vec<f64> = flat.iter().map(|x| x.1).collect();
    all.sort_by(f64::total_cmp);
    cpu.sort_by(f64::total_cmp);
    println!(
        "  {threads} luồng × {n} yêu cầu, tường: p50 {:.0} µs, p95 {:.0} µs, p99 {:.0} µs, max {:.0} µs; CPU luồng: p50 {:.0} µs, p95 {:.0} µs, p99 {:.0} µs, max {:.0} µs",
        percentile(&all, 50.0),
        percentile(&all, 95.0),
        percentile(&all, 99.0),
        all.last().copied().unwrap_or(f64::NAN),
        percentile(&cpu, 50.0),
        percentile(&cpu, 95.0),
        percentile(&cpu, 99.0),
        cpu.last().copied().unwrap_or(f64::NAN)
    );
    println!(
        "  ngân sách đặt cho tầng xếp hạng: {BUDGET_MS} ms cho p99 của một lần xếp hạng (một phần nhỏ của Quote p95 < 500 ms, kiến trúc §14.1). p99 CPU đo được {} ngân sách; p99 tường {} ngân sách",
        if percentile(&cpu, 99.0) / 1000.0 <= BUDGET_MS as f64 {
            "TRONG"
        } else {
            "VƯỢT"
        },
        if percentile(&all, 99.0) / 1000.0 <= BUDGET_MS as f64 {
            "TRONG"
        } else {
            "VƯỢT"
        }
    );
    println!(
        "  caveat: máy đang chịu tải rất nặng (xem uptime): p99 tường là cận trên bị phóng đại; chưa đo trên máy rảnh; p99 CPU chưa bao gồm chờ I/O hay mạng của một dịch vụ thật"
    );

    belief_section(e, &drivers, &reqs, &es, &live);

    println!("\n-- 4. bất biến: kết quả không bao giờ tệ hơn điểm ghim theo J --");
    let mut violations = 0usize;
    let mut ranker = Ranker::new(e, ObjectiveWeights::default(), RankConfig::default());
    let mut checked = 0usize;
    for rq in &reqs {
        let r = run_one(e, &mut ranker, &es, rq, &live);
        if let (Some(b), Some(p)) = (r.top.first(), r.pin)
            && p.cand.violation.is_none()
            && p.j.is_finite()
        {
            checked += 1;
            violations += usize::from(b.j > p.j + 1e-9);
        }
    }
    println!(
        "  {checked} yêu cầu có điểm ghim hợp lệ: số lần ứng viên tốt nhất có J lớn hơn J điểm ghim = {violations}"
    );
    println!(
        "  (proptest cùng bất biến chạy trong `cargo test -p pickup`, tests/rank_props.rs, trên đồ thị fixture)"
    );

    println!("\n-- 5. một yêu cầu mẫu: top 3 --");
    let rq = &reqs[0];
    let r = run_one(e, &mut ranker, &es, rq, &live);
    println!(
        "  khách ({:.6}, {:.6}), ghim ({:.6}, {:.6}), {} tài xế rảnh, {} ứng viên hợp lệ, {} chấm lại bằng A* theo cạnh",
        rq.from.lat(),
        rq.from.lng(),
        rq.pin.lat(),
        rq.pin.lng(),
        drivers.len(),
        r.stats.n_eligible,
        r.stats.n_refined
    );
    for (i, s) in r.top.iter().enumerate() {
        println!(
            "  #{}: ({:.6}, {:.6}) J {:.1} = đi bộ {:.1} s + tài xế {:.1} s + đoạn có khách {:.1} s + phạt {:.1} s; đi bộ {:.0} m; {}",
            i + 1,
            s.cand.point.lat(),
            s.cand.point.lng(),
            s.j,
            s.terms.walk_s,
            s.terms.drive_s,
            s.terms.trip_s,
            s.terms.penalty_s,
            s.walk_m,
            if s.cand.is_pin {
                "(chính là điểm ghim)"
            } else {
                ""
            }
        );
    }
    if let Some(p) = r.pin {
        println!("  điểm ghim: J {:.1}", p.j);
    }
    println!("\nuptime lúc kết thúc: {}", uptime());
}
