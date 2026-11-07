//! LOC-34: phía đường và hướng tiếp cận. CH và ma trận là theo nút, KHÔNG có chi phí rẽ; chỉ A* theo cạnh có. Nên xếp hạng
//! hai tầng: tỉa bằng ma trận rồi chấm lại các ứng viên đầu bằng A* theo cạnh. Lab đo (1) ma trận một mình sai bao nhiêu so với
//! chấm lại, (2) tác dụng của phía đường và hướng tiếp cận trên simulator (so sánh cặp, seed đánh giá).
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc34_phia_duong
//! Tham số: --fixture, --jobs N, --n N (số yêu cầu mẫu cho phần 1–2, mặc định 120), --seeds N (mặc định 20), --skip-sim

use geo_core::haversine_m;
use labs::p03::{mean, uptime};
use labs::p04::*;
use labs::{Args, percentile};
use pickup::objective::ObjectiveWeights;
use pickup::rank::{DriverBelief, DriverRef, RankConfig, Ranker, Request, Scored};
use pickup::strategy::JConfig;
use route_engine::engine::Engine;
use route_engine::position::Position;
use route_engine::profile::Profile;
use route_engine::turn::{EdgeSearcher, Goal, Start, TurnCosts};
use simulator::world::generate;
use std::cell::RefCell;
use std::collections::HashMap;

type Key = (u32, u32);
fn key(s: &Scored) -> Key {
    (s.cand.pos.edge, s.cand.pos.frac.to_bits())
}

fn rank_with(
    e: &Engine,
    cfg: RankConfig,
    req: &Request,
    belief: &DriverBelief,
    es: &RefCell<EdgeSearcher>,
) -> pickup::rank::Ranked {
    let tm = e.turn_model(Profile::Car, TurnCosts::urban());
    let vmax = e.costs(Profile::Car).vmax_cm_per_unit;
    let mut route = |f, t| es.borrow_mut().route(&tm, &e.xy, vmax, f, t, true).0;
    let mut r = Ranker::new(e, ObjectiveWeights::default(), cfg);
    r.rank(e, req, belief, &mut route)
}

fn main() {
    let args = Args::from_env();
    let env = setup(&args);
    let e = &env.engine;
    println!("== LOC-34: phía đường, hướng tiếp cận, và hai tầng ma trận → A* theo cạnh ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let seed = *EVAL_SEEDS.start();
    let sc = with_seed(&env.scenario, seed);
    let n = args.u64_or("--n", 120) as usize;
    println!(
        "{}",
        config_line(
            &env,
            &[seed],
            &format!(
                "{n} yêu cầu mẫu (khách + ghim + đích) và 120 tài xế rảnh của seed, trên dữ liệu mô phỏng; chính sách p04-v1 (a=b=c=d=1)"
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
    let belief = DriverBelief::Live(drivers.clone());
    let es = RefCell::new(EdgeSearcher::new(e.g.edge_count()));
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

    println!("\n-- 1. ma trận một mình sai bao nhiêu so với chấm lại bằng A* theo cạnh --");
    let all_cfg = RankConfig {
        refine_top: 10_000,
        ..RankConfig::default()
    };
    let mat_cfg = RankConfig {
        stage2: false,
        ..RankConfig::default()
    };
    let ks = [0usize, 1, 2, 4, 8, 16];
    let mut regrets: Vec<Vec<f64>> = vec![Vec::new(); ks.len()];
    let mut agree: Vec<usize> = vec![0; ks.len()];
    let mut drive_gap: Vec<f64> = Vec::new();
    let mut trip_gap: Vec<f64> = Vec::new();
    let mut ncand: Vec<f64> = Vec::new();
    let mut used = 0usize;
    for rq in &reqs {
        let full = rank_with(e, all_cfg.clone(), rq, &belief, &es);
        let mat = rank_with(e, mat_cfg.clone(), rq, &belief, &es);
        let exact: HashMap<Key, &Scored> = full
            .all
            .iter()
            .filter(|s| s.exact)
            .map(|s| (key(s), s))
            .collect();
        let best = full
            .all
            .iter()
            .filter(|s| s.exact && s.j.is_finite())
            .map(|s| s.j)
            .fold(f64::INFINITY, f64::min);
        if !best.is_finite() {
            continue;
        }
        used += 1;
        ncand.push(mat.all.len() as f64);
        let order: Vec<&Scored> = mat.all.iter().filter(|s| s.j.is_finite()).collect();
        for (i, &k) in ks.iter().enumerate() {
            let chosen_j = if k == 0 {
                order
                    .first()
                    .and_then(|s| exact.get(&key(s)))
                    .map_or(f64::INFINITY, |x| x.j)
            } else {
                let mut set: Vec<&Scored> =
                    order.iter().filter(|s| s.cand.is_pin).copied().collect();
                set.extend(order.iter().filter(|s| !s.cand.is_pin).take(k).copied());
                set.iter()
                    .filter_map(|s| exact.get(&key(s)))
                    .map(|x| x.j)
                    .fold(f64::INFINITY, f64::min)
            };
            if chosen_j.is_finite() {
                regrets[i].push(chosen_j - best);
                agree[i] += usize::from((chosen_j - best).abs() < 1e-6);
            }
        }
        for s in &mat.all {
            if let Some(x) = exact.get(&key(s))
                && s.terms.drive_s.is_finite()
                && x.terms.drive_s.is_finite()
            {
                drive_gap.push(x.terms.drive_s - s.terms.drive_s);
                if s.terms.trip_s.is_finite() && x.terms.trip_s.is_finite() {
                    trip_gap.push(x.terms.trip_s - s.terms.trip_s);
                }
            }
        }
    }
    println!(
        "  {used} yêu cầu có đáp án; mỗi yêu cầu {:.1} ứng viên hợp lệ (gồm hai chiều của điểm ghim) và 120 tài xế rảnh; «đáp án đúng» = chấm A* theo cạnh MỌI ứng viên",
        mean(&ncand)
    );
    println!(
        "  {:>14} {:>14} {:>16} {:>14} {:>11} {:>14}",
        "A* chấm lại",
        "chọn đúng đáp án",
        "hối tiếc TB (s)",
        "p95 (s)",
        "max (s)",
        "tuyến A*/yêu cầu"
    );
    for (i, &k) in ks.iter().enumerate() {
        regrets[i].sort_by(f64::total_cmp);
        let routes = if k == 0 { 0 } else { (k + 2) * 3 };
        println!(
            "  {:>14} {:>13.1}% {:>16.2} {:>14.2} {:>11.2} {:>14}",
            if k == 0 {
                "0 (chỉ ma trận)".to_string()
            } else {
                format!("top {k} + ghim")
            },
            100.0 * agree[i] as f64 / used.max(1) as f64,
            mean(&regrets[i]),
            percentile(&regrets[i], 95.0),
            regrets[i].last().copied().unwrap_or(f64::NAN),
            routes
        );
    }
    println!(
        "  (hối tiếc = J chính xác của ứng viên được chọn trừ J chính xác nhỏ nhất; đơn vị giây quy đổi của J)"
    );
    drive_gap.sort_by(f64::total_cmp);
    trip_gap.sort_by(f64::total_cmp);
    println!(
        "  sai số số hạng tài xế (A* theo cạnh trừ ma trận, giây): trung bình {:+.1}  p5 {:+.1}  p50 {:+.1}  p95 {:+.1}  max {:+.1}  ({} cặp ứng viên đã chấm lại)",
        mean(&drive_gap),
        percentile(&drive_gap, 5.0),
        percentile(&drive_gap, 50.0),
        percentile(&drive_gap, 95.0),
        drive_gap.last().copied().unwrap_or(f64::NAN),
        drive_gap.len()
    );
    let cheaper = drive_gap.iter().filter(|d| **d < -1.0).count()
        + trip_gap.iter().filter(|d| **d < -1.0).count();
    println!(
        "  sai số số hạng đoạn có khách (A* trừ ma trận, giây): trung bình {:+.1}  p50 {:+.1}  p95 {:+.1}  max {:+.1}; số số hạng mà A* RẺ hơn ma trận quá 1 s: {} (đích cùng cạnh phía trước xe: ma trận phải vòng qua nút, A* đi thẳng)",
        mean(&trip_gap),
        percentile(&trip_gap, 50.0),
        percentile(&trip_gap, 95.0),
        trip_gap.last().copied().unwrap_or(f64::NAN),
        cheaper
    );

    println!(
        "\n-- 2. quay đầu trên tuyến tài xế: điểm ghim so với điểm chọn (tài xế gần nhất theo chim bay) --"
    );
    let tm = e.turn_model(Profile::Car, TurnCosts::urban());
    let vmax = e.costs(Profile::Car).vmax_cm_per_unit;
    let mut route = |f: Start, t: Goal| es.borrow_mut().route(&tm, &e.xy, vmax, f, t, true).0;
    let (mut ub_pin, mut ub_new) = ([0u32; 3], [0u32; 3]);
    let (mut ut_pin, mut ut_new, mut cnt) = (0u32, 0u32, 0u32);
    let (mut tpin, mut tnew) = (Vec::new(), Vec::new());
    for rq in &reqs {
        let full = rank_with(e, RankConfig::default(), rq, &belief, &es);
        let (Some(best), Some(pin)) = (full.top.first(), full.pin) else {
            continue;
        };
        let near = drivers
            .iter()
            .min_by(|a, b| {
                haversine_m(a.latlng, rq.from).total_cmp(&haversine_m(b.latlng, rq.from))
            })
            .expect("có tài xế");
        let go = |p: Position, route: &mut dyn FnMut(Start, Goal) -> Option<route_engine::turn::EdgeRoute>| {
            route(Start::Pos(near.pos), Goal::Pos(p))
        };
        // ghim: lấy chiều rẻ nhất trong hai chiều, như baseline
        let pins: Vec<Position> = full
            .all
            .iter()
            .filter(|s| s.cand.is_pin)
            .map(|s| s.cand.pos)
            .collect();
        let rp = pins
            .iter()
            .filter_map(|&p| go(p, &mut route))
            .min_by_key(|r| r.cost);
        let rn = go(best.cand.pos, &mut route);
        let _ = pin;
        if let (Some(a), Some(b)) = (rp, rn) {
            let (ba, bb) = (tm.u_turn_breakdown(&a.edges), tm.u_turn_breakdown(&b.edges));
            for i in 0..3 {
                ub_pin[i] += ba[i];
                ub_new[i] += bb[i];
            }
            ut_pin += ba.iter().sum::<u32>();
            ut_new += bb.iter().sum::<u32>();
            tpin.push(f64::from(a.cost) / 1000.0);
            tnew.push(f64::from(b.cost) / 1000.0);
            cnt += 1;
        }
    }
    println!(
        "  {cnt} yêu cầu: quay đầu trên tuyến tài xế tới điểm ghim = {ut_pin} (đi ngược trực tiếp {}, qua lối dải phân cách {}, khác {}); tới điểm chọn = {ut_new} (đi ngược trực tiếp {}, qua lối dải phân cách {}, khác {})",
        ub_pin[0], ub_pin[1], ub_pin[2], ub_new[0], ub_new[1], ub_new[2]
    );
    println!(
        "  thời gian tài xế tới điểm hẹn (A* theo cạnh, trung bình): ghim {:.1} s, điểm chọn {:.1} s",
        mean(&tpin),
        mean(&tnew)
    );

    if args.flag("--skip-sim") {
        println!("\nuptime lúc kết thúc: {}", uptime());
        return;
    }
    println!(
        "\n-- 3. trên simulator: phía đường và hướng tiếp cận (so sánh cặp với baseline, seed ĐÁNH GIÁ) --"
    );
    let eval = seeds(&args, false);
    let j = jobs(&args);
    println!(
        "  {}",
        config_line(
            &env,
            &eval,
            &format!("jobs {j}; baseline = AtPin + NearestStraightLine; chính sách p04-v1")
        )
    );
    println!(
        "  Phạt phía đường chỉ là số hạng của J: mô phỏng KHÔNG tính thời gian băng đường, nên lợi ích thật của nó (khách khỏi băng đường) không hiện ra ở chỉ số nào bên dưới, chỉ có chi phí (nếu có)."
    );
    let base = par_seeds(j, &eval, |s| {
        run_baseline(&env.engine, &with_seed(&env.scenario, s), false)
    });
    let variants: Vec<(&str, JConfig)> = {
        let w = ObjectiveWeights::default();
        let full = JConfig::new("j-full", w.clone());
        let mut no_side = JConfig::new("j-no-side", w.clone());
        no_side.rank.use_side = false;
        let mut mat = JConfig::new("j-matrix-only", w.clone());
        mat.rank.stage2 = false;
        let mut best_time = JConfig::new("j-best-time", w.clone());
        best_time.rank.driver_model = pickup::rank::DriverModel::BestTime;
        let mut one_dir = JConfig::new("j-one-direction", w);
        one_dir.commit_margin_s = f64::NEG_INFINITY;
        vec![
            (
                "(a) J đầy đủ: ma trận + A* theo cạnh, phạt phía đường, hai hướng nếu J hơn kém ≤ 20 s",
                full,
            ),
            ("(b) bỏ phạt phía đường", no_side),
            ("(c) chỉ ma trận, không A* theo cạnh", mat),
            (
                "(d) chỉ một hướng tiếp cận (không bao giờ cho phép hướng thứ hai)",
                one_dir,
            ),
            (
                "(e) giả định bộ ghép chọn tài xế nhanh nhất (BestTime) trong khi bộ ghép thật là gần nhất theo chim bay",
                best_time,
            ),
        ]
    };
    for (name, cfg) in &variants {
        let runs = par_seeds(j, &eval, |s| {
            run_j(&env.engine, &with_seed(&env.scenario, s), cfg)
        });
        println!();
        print_paired(&format!("  {name}"), &base, &runs);
    }
    println!("\nuptime lúc kết thúc: {}", uptime());
}
