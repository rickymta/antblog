//! LOC-36: điểm hẹn động, tính lại khi cả hai đang di chuyển. Ba chính sách (tĩnh, đổi ngay, có trễ + khách đồng ý)
//! trên các tình huống khách đi chậm, tài xế trễ, GPS nhiễu; so sánh cặp theo seed.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc36_diem_hen_dong
//! Tham số: --fixture, --jobs N, --seeds N (mặc định 20), --per-seed K (cặp khách–tài xế mỗi seed, mặc định 8)

use geo_core::haversine_m;
use labs::Args;
use labs::p03::{mean, uptime};
use labs::p04::*;
use pickup::dynamic::{DynConfig, DynOutcome, DynScenario, Kind, Policy, Simulation};
use pickup::objective::ObjectiveWeights;
use pickup::rank::DriverRef;
use pickup::stats::paired;
use route_engine::engine::Engine;
use simulator::world::generate;

fn scenarios(e: &Engine, sc: &simulator::Scenario, kind: Kind, k: usize) -> Vec<DynScenario> {
    let w = generate(e, sc);
    let mut used = vec![false; w.drivers.len()];
    let mut out = Vec::new();
    for c in &w.customers {
        if out.len() >= k {
            break;
        }
        // tài xế rảnh gần nhất chưa dùng, cách 700–4.000 m chim bay để có thời gian di chuyển đủ dài
        let best = w
            .drivers
            .iter()
            .enumerate()
            .filter(|(i, _)| !used[*i])
            .map(|(i, d)| (i, haversine_m(d.pos.point(&e.g), c.view.true_pos)))
            .filter(|(_, d)| (700.0..=4000.0).contains(d))
            .min_by(|a, b| a.1.total_cmp(&b.1));
        let Some((i, _)) = best else { continue };
        used[i] = true;
        let p = w.drivers[i].pos;
        out.push(DynScenario {
            cust0: c.view.true_pos,
            pin: c.view.pin,
            dest: c.view.dropoff,
            driver0: DriverRef {
                pos: p,
                latlng: p.point(&e.g),
            },
            kind,
            seed: sc.seed * 1000 + c.view.id as u64,
        });
    }
    out
}

fn mean_outcome(o: &[DynOutcome]) -> [f64; 9] {
    let f = |g: &dyn Fn(&DynOutcome) -> f64| mean(&o.iter().map(g).collect::<Vec<_>>());
    [
        f(&|x| x.t_pick_s),
        f(&|x| x.meet_gap_s),
        f(&|x| x.walk_total_m),
        f(&|x| x.drive_total_m),
        f(&|x| f64::from(x.changes)),
        f(&|x| f64::from(x.flips)),
        f(&|x| f64::from(x.declined)),
        f(&|x| f64::from(x.replans)),
        f(&|x| f64::from(u8::from(x.completed))),
    ]
}

const COLS: [&str; 9] = [
    "t_pick_s",
    "meet_gap_s",
    "walk_m",
    "drive_m",
    "đổi/ca",
    "flip/ca",
    "từ chối/ca",
    "tính lại/ca",
    "xong",
];

fn main() {
    let args = Args::from_env();
    let env = setup(&args);
    let e = &env.engine;
    println!("== LOC-36: điểm hẹn động, cập nhật khi cả hai đang di chuyển ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let eval = seeds(&args, false);
    let j = jobs(&args);
    let per = args.u64_or("--per-seed", 8) as usize;
    println!(
        "{}",
        config_line(
            &env,
            &eval,
            &format!(
                "{per} cặp khách–tài xế mỗi seed (tài xế rảnh gần nhất cách 700–4.000 m chim bay); jobs {j}; mô phỏng theo bước 5 s; trên dữ liệu mô phỏng"
            )
        )
    );
    let cfg = DynConfig::default();
    println!(
        "  tham số chính sách có trễ: kích hoạt khi giờ hai bên cùng có mặt dự báo trễ hơn kế hoạch > {} s, nguội {} s, khoá khi cả hai còn < {} s, chỉ đổi khi J giảm ≥ {} s và ≥ {:.0}%, khách đi thêm ≤ {} m; mô hình đồng ý: xác suất 0,95 − 0,004 × mét đi thêm (kẹp 0,05–0,95), GIẢ ĐỊNH của lab",
        cfg.trigger_dev_s,
        cfg.cooldown_s,
        cfg.lock_s,
        cfg.min_gain_s,
        100.0 * cfg.min_gain_frac,
        cfg.max_extra_walk_m
    );
    println!(
        "  nền tảng chỉ thấy vị trí quan sát và tốc độ ước từ quãng đã đi trong {} s, không thấy hệ số tốc độ thật; chính sách tĩnh dùng điểm hẹn lúc đầu đã BIẾT tài xế (cận tốt nhất của «không đổi»)",
        cfg.speed_window_s
    );
    let kinds: Vec<(&str, Kind)> = vec![
        ("bình thường (đối chứng)", Kind::Nominal),
        (
            "khách đi chậm (0,5 × tốc độ dự kiến)",
            Kind::SlowWalker(0.5),
        ),
        ("tài xế trễ 120 s", Kind::LateDriver(120.0)),
        (
            "cả hai: khách chậm 0,5 × và tài xế trễ 120 s",
            Kind::Both {
                walk: 0.5,
                delay_s: 120.0,
            },
        ),
        (
            "GPS khách nhiễu σ = 10 m (tốc độ đúng)",
            Kind::NoisyGps(10.0),
        ),
    ];
    let policies = [Policy::Static, Policy::Naive, Policy::Hysteresis];
    for (kname, kind) in &kinds {
        // mỗi seed: giá trị trung bình qua `per` cặp, theo từng chính sách
        let rows: Vec<Vec<[f64; 9]>> = par_seeds(j, &eval, |s| {
            let sc = with_seed(&env.scenario, s);
            let scs = scenarios(e, &sc, *kind, per);
            policies
                .iter()
                .map(|&p| {
                    let mut sim = Simulation::new(
                        e,
                        ObjectiveWeights::default(),
                        DynConfig {
                            naive_interval_s: 15.0,
                            ..cfg
                        },
                    );
                    let outs: Vec<DynOutcome> = scs.iter().map(|x| sim.run(x, p)).collect();
                    mean_outcome(&outs)
                })
                .collect()
        });
        println!("\n  tình huống: {kname}");
        println!(
            "    {:<18} {}",
            "chính sách",
            COLS.iter()
                .map(|c| format!("{c:>11}"))
                .collect::<Vec<_>>()
                .join(" ")
        );
        for (pi, p) in policies.iter().enumerate() {
            let m: Vec<f64> = (0..9)
                .map(|c| mean(&rows.iter().map(|r| r[pi][c]).collect::<Vec<_>>()))
                .collect();
            println!(
                "    {:<18} {}",
                p.name(),
                m.iter()
                    .map(|x| format!("{x:>11.2}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
        for (pi, p) in policies.iter().enumerate().skip(1) {
            let mut line = format!("    {:<18}", format!("{} − tĩnh", p.name()));
            for (ci, name) in [
                (0usize, "t_pick_s"),
                (1, "meet_gap_s"),
                (2, "walk_m"),
                (3, "drive_m"),
            ] {
                let b: Vec<f64> = rows.iter().map(|r| r[0][ci]).collect();
                let n: Vec<f64> = rows.iter().map(|r| r[pi][ci]).collect();
                let pr = paired(&b, &n);
                line += &format!(
                    "  {name} {:+.1} [{:+.1}, {:+.1}]{}",
                    pr.mean_diff,
                    pr.t_lo,
                    pr.t_hi,
                    if pr.significant() { "*" } else { "" }
                );
            }
            println!("{line}");
        }
    }
    println!(
        "\n  (chênh = trung bình của chênh lệch cặp theo seed, trong ngoặc là KTC 95% t, n = {} seed; * = KTC không chứa 0; t_pick_s: giờ cả hai có mặt; meet_gap_s: lệch giờ tới; walk_m, drive_m: tổng quãng khách đi bộ và tài xế chạy kể cả quãng bỏ dở sau khi đổi)",
        eval.len()
    );

    println!(
        "\n  quét ngưỡng «J giảm ít nhất» của chính sách có trễ, tình huống tài xế trễ 120 s (đổi nhiều thì flip tăng, đổi ít thì không hưởng được):"
    );
    println!(
        "    {:<18} {}",
        "min_gain_s",
        COLS.iter()
            .map(|c| format!("{c:>11}"))
            .collect::<Vec<_>>()
            .join(" ")
    );
    let kind = Kind::LateDriver(120.0);
    for gain in [5.0, 25.0, 60.0] {
        let rows: Vec<[f64; 9]> = par_seeds(j, &eval, |s| {
            let sc = with_seed(&env.scenario, s);
            let scs = scenarios(e, &sc, kind, per);
            let mut sim = Simulation::new(
                e,
                ObjectiveWeights::default(),
                DynConfig {
                    min_gain_s: gain,
                    min_gain_frac: 0.0,
                    ..cfg
                },
            );
            let outs: Vec<DynOutcome> =
                scs.iter().map(|x| sim.run(x, Policy::Hysteresis)).collect();
            mean_outcome(&outs)
        });
        let m: Vec<f64> = (0..9)
            .map(|c| mean(&rows.iter().map(|r| r[c]).collect::<Vec<_>>()))
            .collect();
        println!(
            "    {:<18} {}",
            format!("{gain:.0}"),
            m.iter()
                .map(|x| format!("{x:>11.2}"))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
    println!("\nuptime lúc kết thúc: {}", uptime());
}
