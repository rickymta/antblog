//! LOC-32: hàm mục tiêu điểm hẹn J(p) đặt khách và tài xế lên cùng một cân. Ví dụ tay trên đồ thị nhỏ, điều tra
//! `walk_m` đuôi dài của baseline, quét trọng số (đường đánh đổi đi bộ ↔ km rỗng) trên seed chỉnh, quy tắc chọn,
//! rồi đo trên seed đánh giá bằng so sánh cặp.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc32_ham_muc_tieu
//! Tham số: --fixture, --jobs N (mặc định 3), --seeds N (số seed đánh giá, mặc định 20), --tuning-seeds N (mặc định 10),
//!          --skip-golden, --skip-oddity,
//!          --walk-tail (chỉ chạy `walk_m` khi bỏ đuôi đi vòng của baseline, trên ba dải seed, rồi thoát)

#![allow(clippy::print_literal)] // tiêu đề cột bảng viết thẳng trong chuỗi định dạng cho dễ đọc

use geo_core::haversine_m;
use labs::p03::{mean, uptime};
use labs::p04::*;
use labs::{Args, fixture_path};
use pickup::objective::ObjectiveWeights;
use pickup::rank::{DriverBelief, DriverRef, RankConfig, Ranker, Request};
use pickup::stats::paired;
use pickup::strategy::JConfig;
use road_graph::HighwayClass;
use route_engine::engine::{Engine, EngineConfig};
use route_engine::position::Position;
use route_engine::profile::Profile;
use route_engine::turn::{EdgeSearcher, TurnCosts};
use simulator::world::generate;
use std::cell::RefCell;

/// Một dòng bảng quét: tên, a, Δwalk p95, Δcancel, Δhoàn thành (tỉ lệ), Δ(walk+dead), Δdead, Δwalk (chênh cặp so với baseline).
type Row = (String, f64, f64, f64, f64, f64, f64, f64);

fn hand_example() {
    println!("\n-- 1. ví dụ tay trên đồ thị nhỏ (fixtures/khu-mau.json, đồ thị tổng hợp) --");
    let g = road_graph::fixture::load_fixture(&fixture_path()).expect("fixture");
    let e = Engine::new(g, EngineConfig::default());
    let c = e.g.center();
    // khách đứng lệch tim đường, ghim lệch thêm; một tài xế ở cách đó vài trăm mét; đích cách ~1 km
    let from = geo_core::destination(c, 20.0, 60.0);
    let pin = geo_core::destination(from, 110.0, 35.0);
    let view = e.view(Profile::Car);
    // đích: điểm giữa của cạnh xe chạy gần nhất với vị trí cách tâm 800 m về hướng nam-tây-nam (nằm trong bản đồ nhỏ)
    let dest_target = geo_core::destination(c, 200.0, 800.0);
    let dest = (0..e.g.edge_count() as u32)
        .filter(|&x| view.w[x as usize] != route_engine::INF)
        .map(|x| Position::new(x, 0.5).point(&e.g))
        .min_by(|a, b| haversine_m(*a, dest_target).total_cmp(&haversine_m(*b, dest_target)))
        .expect("có cạnh");
    let drv_edge = (0..e.g.edge_count() as u32)
        .filter(|&x| view.w[x as usize] != route_engine::INF)
        .min_by(|&a, &b| {
            let pa = Position::new(a, 0.5).point(&e.g);
            let pb = Position::new(b, 0.5).point(&e.g);
            let t = geo_core::destination(c, 280.0, 600.0);
            haversine_m(pa, t).total_cmp(&haversine_m(pb, t))
        })
        .expect("có cạnh");
    let dpos = Position::new(drv_edge, 0.5);
    let drv = DriverRef {
        pos: dpos,
        latlng: dpos.point(&e.g),
    };
    let w = ObjectiveWeights::default();
    println!(
        "  chính sách {} : J = a·T_walk + b·max(T_walk, T_drive) + c·T_drive + d·T_trip + phạt, a={} b={} c={} d={}, max_walk_m={}",
        w.policy_version, w.a, w.b, w.c, w.d, w.max_walk_m
    );
    println!(
        "  khách ở ({:.5}, {:.5}), ghim cách {:.0} m, đích cách {:.0} m chim bay, tài xế cách khách {:.0} m chim bay",
        from.lat(),
        from.lng(),
        haversine_m(from, pin),
        haversine_m(from, dest),
        haversine_m(from, drv.latlng)
    );
    let mut ranker = Ranker::new(&e, w.clone(), RankConfig::default());
    let tm = e.turn_model(Profile::Car, TurnCosts::urban());
    let vmax = e.costs(Profile::Car).vmax_cm_per_unit;
    let es = RefCell::new(EdgeSearcher::new(e.g.edge_count()));
    let mut route = |f, t| es.borrow_mut().route(&tm, &e.xy, vmax, f, t, true).0;
    let r = ranker.rank(
        &e,
        &Request { from, pin, dest },
        &DriverBelief::Live(vec![drv]),
        &mut route,
    );
    let gs = r.stats.gen_stats;
    println!(
        "  ứng viên: {} cạnh xe chạy trong tầm đi bộ → loại cấm dừng {}, vòng xuyến {}, quá ngắn {}, quá xa {}, cầu/hầm {}; gom {}; còn {}; chấm A* theo cạnh {}",
        gs.raw,
        gs.excl_no_stop_class,
        gs.excl_roundabout,
        gs.excl_too_short,
        gs.excl_walk,
        gs.excl_bridge_tunnel,
        gs.merged_away,
        gs.kept,
        r.stats.n_refined
    );
    println!(
        "  {:<16} {:>8} {:>8} {:>8} {:>8} {:>9} {:>9}   J = a·{} + b·max + c·{} + d·{} + phạt",
        "ứng viên", "walk_s", "drive_s", "trip_s", "phạt_s", "J", "walk_m", "walk", "drive", "trip"
    );
    for (i, s) in r.all.iter().filter(|s| s.exact).take(6).enumerate() {
        let name = if s.cand.is_pin {
            format!("ghim, cạnh {}", s.cand.pos.edge)
        } else {
            format!("cạnh {}", s.cand.pos.edge)
        };
        let t = &s.terms;
        println!(
            "  {:<16} {:>8.1} {:>8.1} {:>8.1} {:>8.1} {:>9.1} {:>9.1}   {}",
            name,
            t.walk_s,
            t.drive_s,
            t.trip_s,
            t.penalty_s,
            s.j,
            s.walk_m,
            if i == 0 { "← nhỏ nhất" } else { "" }
        );
    }
    if let (Some(best), Some(pin_s)) = (r.top.first(), r.pin) {
        println!(
            "  J tốt nhất {:.1} s so với điểm ghim {:.1} s; bất biến «không tệ hơn ghim»: {}",
            best.j,
            pin_s.j,
            best.j <= pin_s.j + 1e-9
        );
    }
    println!(
        "  ràng buộc cứng so với phạt: ứng viên cấm dừng (cao tốc, cầu/hầm, vòng xuyến) bị LOẠI hẳn; phía đường bất lợi, hẻm, đi bộ quá 150 m chỉ cộng giây phạt vào J"
    );
}

/// Điều tra walk_m đuôi dài của baseline trên seed 20261106.
fn oddity(env: &Env) {
    println!(
        "\n-- 2. điều tra: vì sao baseline có walk_m lớn nhất 3.492,5 m trong khi p50 chỉ 11 m --"
    );
    let e = &env.engine;
    let w = generate(e, &env.scenario);
    struct Row {
        id: u32,
        walk: f64,
        crow: f64,
        pin_sigma: f64,
        true_to_foot: f64,
        stop_to_foot: f64,
        true_to_road: f64,
        foot_err: bool,
        foot_class: String,
    }
    let mut rows: Vec<Row> = Vec::new();
    for c in &w.customers {
        let Ok(snap) = e.snap_point(Profile::Car, c.view.pin, "ghim") else {
            continue;
        };
        let stop = snap.point;
        let walk = Ranker::walk_m(e, c.view.true_pos, stop);
        let fr = e.route(Profile::Foot, c.view.true_pos, stop, None);
        let ts = e.snap_point(Profile::Foot, c.view.true_pos, "khách");
        let ss = e.snap_point(Profile::Foot, stop, "điểm đón");
        let cls = ts.as_ref().map_or("?".into(), |s| {
            HighwayClass::from_u8(e.g.class[s.edge as usize])
                .map_or("?".to_string(), |h| h.osm_value().to_string())
        });
        let to_road = e
            .snap_point(Profile::Car, c.view.true_pos, "khách")
            .map_or(f64::NAN, |s| s.snap_dist_m);
        rows.push(Row {
            id: c.view.id,
            walk,
            crow: haversine_m(c.view.true_pos, stop),
            pin_sigma: haversine_m(c.view.true_pos, c.view.pin),
            true_to_foot: ts.map_or(f64::NAN, |s| s.snap_dist_m),
            stop_to_foot: ss.map_or(f64::NAN, |s| s.snap_dist_m),
            true_to_road: to_road,
            foot_err: fr.is_err(),
            foot_class: cls,
        });
    }
    let n = rows.len();
    let tail: Vec<&Row> = rows.iter().filter(|r| r.walk > 300.0).collect();
    println!(
        "  {} khách của seed {} (đường đi bộ từ chỗ đứng thật tới điểm ghim đã snap lên đường ô tô, đúng như AtPin): {} khách có walk_m > 300 m, {} lỗi định tuyến đi bộ (rơi về chim bay × 1,3)",
        n,
        env.scenario.seed,
        tail.len(),
        rows.iter().filter(|r| r.foot_err).count()
    );
    println!(
        "  {:>5} {:>9} {:>9} {:>7} {:>10} {:>12} {:>12} {:>12} {:>9}",
        "khách",
        "walk_m",
        "chim bay",
        "tỉ lệ",
        "ghim lệch",
        "đứng→đường xe",
        "đứng→đường bộ",
        "điểm đón→đb",
        "loại cạnh bộ"
    );
    let mut worst: Vec<&Row> = rows.iter().collect();
    worst.sort_by(|a, b| b.walk.total_cmp(&a.walk));
    for r in worst.iter().take(8) {
        println!(
            "  {:>5} {:>9.1} {:>9.1} {:>7.1} {:>10.1} {:>12.1} {:>12.1} {:>12.1} {:>9}",
            r.id,
            r.walk,
            r.crow,
            r.walk / r.crow.max(1.0),
            r.pin_sigma,
            r.true_to_road,
            r.true_to_foot,
            r.stop_to_foot,
            r.foot_class
        );
    }
    let near_tail = tail.iter().filter(|r| r.crow < 150.0).count();
    let far_road = tail.iter().filter(|r| r.true_to_road > 40.0).count();
    let big_pin = tail.iter().filter(|r| r.pin_sigma > 25.0).count();
    println!(
        "  trong {} khách đuôi: {} có chim bay < 150 m (vòng đường, không phải xa thật), {} có chỗ đứng cách đường xe > 40 m, {} có ghim lệch chỗ đứng > 25 m",
        tail.len(),
        near_tail,
        far_road,
        big_pin
    );
    let all_w: Vec<f64> = rows.iter().map(|r| r.walk).collect();
    let rest_w: Vec<f64> = rows
        .iter()
        .filter(|r| r.walk <= 300.0)
        .map(|r| r.walk)
        .collect();
    let share = tail.iter().map(|r| r.walk).sum::<f64>() / all_w.iter().sum::<f64>().max(1.0);
    println!(
        "  walk_m trung bình mọi khách {:.1} m; bỏ khách đuôi {:.1} m; khách đuôi chiếm {:.1}% tổng quãng đi bộ của baseline",
        mean(&all_w),
        mean(&rest_w),
        100.0 * share
    );
    println!(
        "  → một phần lớn của «cải thiện walk_m» có thể chỉ là bỏ các ca này; mọi so sánh bên dưới phân tầng theo walk_m baseline > 300 m"
    );
}

/// `walk_m` của `p04-v1` khi bỏ đuôi đi vòng của baseline, trên cả ba dải seed (ghi ở results/p04.md, mục LOC-32).
fn walk_tail(args: &Args, env: &Env, j: usize) {
    println!(
        "\n-- walk_m khi bỏ đuôi đi vòng của baseline: p04-v1 (a=b=c=d=1), J hai tầng, so cặp theo seed --"
    );
    println!(
        "  chỉ tính khách hoàn thành ở cả hai lần chạy; p50, p95 tính trong từng seed trên đúng các cặp còn lại nên khác bảng 19 chỉ số (bảng đó tính trên mọi chuyến hoàn thành của từng lần chạy)"
    );
    let n = args.u64_or("--seeds", 20) as usize;
    let ranges: [(&str, Vec<u64>); 3] = [
        (
            "dải CHỈNH",
            TUNING_SEEDS
                .take(args.u64_or("--tuning-seeds", 10) as usize)
                .collect(),
        ),
        ("dải ĐÁNH GIÁ", EVAL_SEEDS.take(n).collect()),
        ("dải XÁC NHẬN", (20_270_101..=20_270_120).take(n).collect()),
    ];
    let cfg = JConfig::new("j-p04-v1", ObjectiveWeights::default());
    for (label, seeds) in &ranges {
        println!(
            "\n  {label}: {}",
            config_line(env, seeds, &format!("jobs {j}"))
        );
        let base = par_seeds(j, seeds, |s| {
            run_baseline(&env.engine, &with_seed(&env.scenario, s), false)
        });
        let new = par_seeds(j, seeds, |s| {
            run_j(&env.engine, &with_seed(&env.scenario, s), &cfg)
        });
        print_walk_excl_tail(&base, &new);
        let col =
            |f: &dyn Fn(&RunResult) -> f64, v: &[RunResult]| v.iter().map(f).collect::<Vec<_>>();
        let d = paired(
            &col(&|r| r.out.summary.deadhead_m.mean, &base),
            &col(&|r| r.out.summary.deadhead_m.mean, &new),
        );
        let w = paired(
            &col(&|r| r.out.summary.walk_m.mean, &base),
            &col(&|r| r.out.summary.walk_m.mean, &new),
        );
        println!(
            "  đối chiếu với bảng 19 chỉ số: deadhead_m trung bình {:+.1} [{:+.1}, {:+.1}], walk_m trung bình {:+.1} [{:+.1}, {:+.1}]",
            d.mean_diff, d.t_lo, d.t_hi, w.mean_diff, w.t_lo, w.t_hi
        );
    }
}

fn main() {
    let args = Args::from_env();
    let env = setup(&args);
    println!("== LOC-32: hàm mục tiêu điểm hẹn J(p), đánh đổi đi bộ ↔ km rỗng ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    println!(
        "tất cả số liệu trên dữ liệu mô phỏng (simulator, không có chuyến thật). {}",
        env.graph_info
    );
    let j = jobs(&args);

    if args.flag("--walk-tail") {
        walk_tail(&args, &env, j);
        println!("\nuptime lúc kết thúc: {}", uptime());
        return;
    }

    hand_example();

    if !args.flag("--skip-golden") && !env.fixture {
        println!("\n-- baseline không đổi sau khi thêm các điểm cắm P04 --");
        let r = run_baseline(&env.engine, &env.scenario, false);
        let h = simulator::metrics::fnv1a(r.out.trips_csv().as_bytes());
        println!(
            "  seed {}: FNV-1a nhật ký chuyến = {h:016x}; giá trị ghi ở results/p03.md = 100a862df76819aa; giống từng byte: {}",
            env.scenario.seed,
            format!("{h:016x}") == "100a862df76819aa"
        );
    }
    if !args.flag("--skip-oddity") && !env.fixture {
        oddity(&env);
    }

    // ---- quét trọng số trên seed chỉnh ----
    let tune = {
        let n = args.u64_or("--tuning-seeds", 10) as usize;
        TUNING_SEEDS.take(n).collect::<Vec<_>>()
    };
    let eval = seeds(&args, false);
    println!("\n-- 3. quét trọng số a/c (khách đi bộ so với tài xế chạy không) trên seed CHỈNH --");
    println!(
        "  {}",
        config_line(
            &env,
            &tune,
            &format!(
                "b = c = d = 1; jobs {j}; seed chỉnh KHÔNG dùng để báo kết quả, seed đánh giá {}…{}",
                eval.first().copied().unwrap_or(0),
                eval.last().copied().unwrap_or(0)
            )
        )
    );
    let grid: Vec<(String, f64, f64)> = vec![
        ("a=0,b=0 (bỏ qua khách)".into(), 0.0, 0.0),
        ("a/c = 0,25".into(), 0.25, 1.0),
        ("a/c = 0,5".into(), 0.5, 1.0),
        ("a/c = 1 (mặc định)".into(), 1.0, 1.0),
        ("a/c = 2".into(), 2.0, 1.0),
        ("a/c = 4".into(), 4.0, 1.0),
        ("a/c = 8".into(), 8.0, 1.0),
    ];
    let base_t = par_seeds(j, &tune, |s| {
        run_baseline(&env.engine, &with_seed(&env.scenario, s), false)
    });
    println!(
        "  {:<26} {:>9} {:>9} {:>10} {:>9} {:>11} {:>9} {:>9} {:>9}  {}",
        "cấu hình",
        "walk TB",
        "walk p95",
        "deadhead TB",
        "eta TB",
        "walk+dead TB",
        "cancel",
        "hoàn thành",
        "u_turns",
        "ghi chú (chênh cặp so với baseline, mét)"
    );
    println!(
        "  {:<26} {:>9.1} {:>9.1} {:>10.1} {:>9.1} {:>11.1} {:>9.4} {:>9.1} {:>9.3}",
        "baseline (ghim)",
        mean(
            &base_t
                .iter()
                .map(|r| r.out.summary.walk_m.mean)
                .collect::<Vec<_>>()
        ),
        mean(
            &base_t
                .iter()
                .map(|r| r.out.summary.walk_m.p95)
                .collect::<Vec<_>>()
        ),
        mean(
            &base_t
                .iter()
                .map(|r| r.out.summary.deadhead_m.mean)
                .collect::<Vec<_>>()
        ),
        mean(
            &base_t
                .iter()
                .map(|r| r.out.summary.pickup_eta_s.mean)
                .collect::<Vec<_>>()
        ),
        mean(
            &base_t
                .iter()
                .map(|r| combined_mean(&r.out))
                .collect::<Vec<_>>()
        ),
        mean(
            &base_t
                .iter()
                .map(|r| r.out.summary.cancel_rate)
                .collect::<Vec<_>>()
        ),
        mean(
            &base_t
                .iter()
                .map(|r| r.out.summary.completed as f64)
                .collect::<Vec<_>>()
        ),
        mean(
            &base_t
                .iter()
                .map(|r| r.out.summary.u_turns_per_trip)
                .collect::<Vec<_>>()
        ),
    );
    let mut table: Vec<Row> = Vec::new();
    let base_walk_p95: Vec<f64> = base_t.iter().map(|r| r.out.summary.walk_m.p95).collect();
    let base_cancel: Vec<f64> = base_t.iter().map(|r| r.out.summary.cancel_rate).collect();
    let base_comp: Vec<f64> = base_t
        .iter()
        .map(|r| r.out.summary.completed as f64)
        .collect();
    let base_comb: Vec<f64> = base_t.iter().map(|r| combined_mean(&r.out)).collect();
    let base_dead: Vec<f64> = base_t
        .iter()
        .map(|r| r.out.summary.deadhead_m.mean)
        .collect();
    let base_walk: Vec<f64> = base_t.iter().map(|r| r.out.summary.walk_m.mean).collect();
    for (name, a, b) in &grid {
        let w = ObjectiveWeights {
            a: *a,
            b: *b,
            policy_version: format!("p04-v1-a{a}-b{b}"),
            ..ObjectiveWeights::default()
        };
        let cfg = JConfig::new("j", w);
        let runs = par_seeds(j, &tune, |s| {
            run_j(&env.engine, &with_seed(&env.scenario, s), &cfg)
        });
        let col = |f: &dyn Fn(&RunResult) -> f64| runs.iter().map(f).collect::<Vec<_>>();
        let wm = col(&|r| r.out.summary.walk_m.mean);
        let wp = col(&|r| r.out.summary.walk_m.p95);
        let dm = col(&|r| r.out.summary.deadhead_m.mean);
        let em = col(&|r| r.out.summary.pickup_eta_s.mean);
        let cm = col(&|r| combined_mean(&r.out));
        let cr = col(&|r| r.out.summary.cancel_rate);
        let cp = col(&|r| r.out.summary.completed as f64);
        let ut = col(&|r| r.out.summary.u_turns_per_trip);
        let pd = |x: &[f64], b: &[f64]| paired(b, x);
        println!(
            "  {:<26} {:>9.1} {:>9.1} {:>10.1} {:>9.1} {:>11.1} {:>9.4} {:>9.1} {:>9.3}  Δwalk {:+.1}, Δdead {:+.1}, Δ(walk+dead) {:+.1} [{:+.1}, {:+.1}]",
            name,
            mean(&wm),
            mean(&wp),
            mean(&dm),
            mean(&em),
            mean(&cm),
            mean(&cr),
            mean(&cp),
            mean(&ut),
            pd(&wm, &base_walk).mean_diff,
            pd(&dm, &base_dead).mean_diff,
            pd(&cm, &base_comb).mean_diff,
            pd(&cm, &base_comb).t_lo,
            pd(&cm, &base_comb).t_hi,
        );
        table.push((
            name.clone(),
            *a,
            pd(&wp, &base_walk_p95).mean_diff,
            pd(&cr, &base_cancel).mean_diff,
            pd(&cp, &base_comp).mean_diff / mean(&base_comp),
            pd(&cm, &base_comb).mean_diff,
            pd(&dm, &base_dead).mean_diff,
            pd(&wm, &base_walk).mean_diff,
        ));
    }
    println!(
        "  (đường đánh đổi: đi dọc bảng từ trên xuống, khách đi bộ nhiều dần lên nếu tài xế được lợi, hay ngược lại)"
    );

    // quy tắc chọn
    println!(
        "\n-- 4. quy tắc chọn trọng số (áp dụng trên seed chỉnh, KHÔNG dùng seed đánh giá) --"
    );
    println!(
        "  Quy tắc v1 (đặt trước khi chạy): cổng chặn = chênh cặp TB của walk_m p95 ≤ +25 m, cancel_rate ≤ +0,01, số chuyến hoàn thành không giảm quá 2%;"
    );
    println!(
        "    trong các cấu hình qua cổng chọn cấu hình có walk_m + deadhead_m mỗi chuyến nhỏ nhất (tổng quãng của cả hai người). Không cấu hình nào qua cổng thì giữ a/c = 1."
    );
    println!("  Quy tắc v2 = v1 + cổng thứ hai: chênh cặp TB của walk_m TRUNG BÌNH ≤ +25 m.");
    println!(
        "    LÝ DO SỬA (ghi thẳng): lần chạy đầu cho thấy cổng p95 của v1 mù với khách điển hình, vì p95 của baseline do ~5% khách ở các ca ghim sai (mục 2) quyết định:"
    );
    println!(
        "    cấu hình nào cũng làm đuôi tốt lên nên cổng p95 luôn qua, kể cả khi khách điển hình đi bộ thêm hơn 100 m. Việc sửa được quyết định SAU KHI thấy kết quả trên seed đánh giá của cấu hình mà v1 chọn,"
    );
    println!(
        "    nên số của chính sách đã chọn trên seed đánh giá không còn «sạch» tuyệt đối cho quyết định này; LOC-40 đo lại bằng dải seed xác nhận chưa từng nhìn."
    );
    let pass_v1 = |t: &Row| t.2 <= 25.0 && t.3 <= 0.01 && t.4 >= -0.02;
    let pass_v2 = |t: &Row| pass_v1(t) && t.7 <= 25.0;
    for t in &table {
        println!(
            "  {:<26} Δwalk TB {:+7.1}  Δwalk p95 {:+7.1}  Δcancel {:+.4}  Δhoàn thành {:+.1}%  Δ(walk+dead) {:+8.1}  → v1: {}, v2: {}",
            t.0,
            t.7,
            t.2,
            t.3,
            100.0 * t.4,
            t.5,
            if pass_v1(t) { "qua" } else { "BỊ CHẶN" },
            if pass_v2(t) { "qua" } else { "BỊ CHẶN" }
        );
    }
    let pick = |f: &dyn Fn(&Row) -> bool| {
        table
            .iter()
            .filter(|t| f(t))
            .min_by(|a, b| a.5.total_cmp(&b.5))
            .map(|t| (t.0.clone(), t.1))
            .unwrap_or(("a/c = 1 (mặc định)".into(), 1.0))
    };
    let (c1, c2) = (pick(&pass_v1), pick(&pass_v2));
    let b_of = |name: &str| grid.iter().find(|g| g.0 == name).map_or(1.0, |g| g.2);
    println!(
        "  → v1 chọn: {} (a = {}, b = {}); v2 chọn: {} (a = {}, b = {})",
        c1.0,
        c1.1,
        b_of(&c1.0),
        c2.0,
        c2.1,
        b_of(&c2.0)
    );
    println!(
        "  → CHÍNH SÁCH `p04-v1` dùng cho các bài sau = lựa chọn của quy tắc v2 ({}); lựa chọn của v1 vẫn được đo bên dưới để thấy khác biệt",
        c2.0
    );

    // ---- đánh giá trên seed đánh giá ----
    let base = par_seeds(j, &eval, |s| {
        run_baseline(&env.engine, &with_seed(&env.scenario, s), false)
    });
    let mut todo: Vec<(&str, (String, f64))> = vec![("quy tắc v2 = p04-v1", c2.clone())];
    if c1.1 != c2.1 {
        todo.push(("quy tắc v1 (để so)", c1.clone()));
    }
    for (label, (name, a)) in todo {
        println!("\n-- 5. đo trên seed ĐÁNH GIÁ: {label}: {name} (so sánh cặp, J hai tầng) --");
        println!(
            "  {}",
            config_line(
                &env,
                &eval,
                &format!("jobs {j}; seed đánh giá chưa dùng để chỉnh trọng số")
            )
        );
        let w = ObjectiveWeights {
            a,
            b: b_of(&name),
            policy_version: "p04-v1".into(),
            ..ObjectiveWeights::default()
        };
        let cfg = JConfig::new("j-p04-v1", w);
        let new = par_seeds(j, &eval, |s| {
            run_j(&env.engine, &with_seed(&env.scenario, s), &cfg)
        });
        print_paired("  bảng so sánh cặp:", &base, &new);
        print_who_pays(&base, &new);
        let ch: Vec<_> = new.iter().flat_map(|r| r.choices.iter()).collect();
        println!(
            "  quyết định: {} lượt chọn; đúng điểm ghim {:.1}%; J điểm ghim trừ J điểm chọn trung bình {:.1} s (chỉ tính khi cả hai hữu hạn)",
            ch.len(),
            100.0 * ch.iter().filter(|c| c.chose_pin).count() as f64 / ch.len().max(1) as f64,
            mean(
                &ch.iter()
                    .filter(|c| c.j_chosen.is_finite() && c.j_pin.is_finite())
                    .map(|c| c.j_pin - c.j_chosen)
                    .collect::<Vec<_>>()
            )
        );
    }
    println!("\nuptime lúc kết thúc: {}", uptime());
}
