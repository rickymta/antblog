//! LOC-38: tối ưu điểm trả. Hàm mục tiêu J_trả (khách đi bộ tới đích thật hoặc cổng, xe tới điểm trả, chi phí thoát của
//! tài xế, phía đường), đo trên simulator bằng so sánh cặp: ai đi bộ thêm, ai chạy ít đi, chuyến kế được lợi bao nhiêu.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc38_diem_tra
//! Tham số: --fixture, --jobs N, --seeds N (mặc định 20)

use labs::p03::{mean, uptime};
use labs::p04::*;
use labs::{Args, percentile};
use pickup::dropoff::DropoffPolicy;
use pickup::objective::ObjectiveWeights;
use pickup::stats::paired;
use pickup::strategy::JConfig;

fn drop_summary(label: &str, base: &[RunResult], new: &[RunResult]) {
    // theo từng khách hoàn thành ở cả hai lần chạy: quãng đi bộ từ điểm trả tới đích (hoặc cổng)
    let (mut wb, mut wn, mut diff) = (Vec::new(), Vec::new(), Vec::new());
    let (mut default_n, mut gate_n, mut tot, mut exit) = (0usize, 0usize, 0usize, Vec::new());
    for (b, n) in base.iter().zip(new) {
        let done: std::collections::BTreeSet<u32> = customer_pairs(&b.out, &n.out)
            .iter()
            .map(|(t, _)| t.customer)
            .collect();
        let bm: std::collections::BTreeMap<u32, f64> =
            b.drops.iter().map(|d| (d.customer, d.walk_m)).collect();
        for d in &n.drops {
            if !done.contains(&d.customer) {
                continue;
            }
            if let Some(&bw) = bm.get(&d.customer) {
                wb.push(bw);
                wn.push(d.walk_m);
                diff.push(d.walk_m - bw);
                tot += 1;
                default_n += usize::from(d.chose_default);
                gate_n += usize::from(d.gate);
                if d.exit_s > 0.0 {
                    exit.push(d.exit_s);
                }
            }
        }
    }
    let mut s = diff.clone();
    s.sort_by(f64::total_cmp);
    let (mut a, mut c) = (wb.clone(), wn.clone());
    a.sort_by(f64::total_cmp);
    c.sort_by(f64::total_cmp);
    println!(
        "  {label}: quãng khách đi bộ từ điểm trả tới đích thật/cổng ({tot} chuyến ghép cặp theo khách)"
    );
    println!(
        "    baseline (trả đúng điểm đích đã snap): TB {:.1}  p50 {:.1}  p95 {:.1}  max {:.1} m",
        mean(&a),
        percentile(&a, 50.0),
        percentile(&a, 95.0),
        a.last().copied().unwrap_or(f64::NAN)
    );
    println!(
        "    chiến lược:                            TB {:.1}  p50 {:.1}  p95 {:.1}  max {:.1} m; chênh theo khách p5 {:+.1}  p50 {:+.1}  p95 {:+.1}  max {:+.1} m",
        mean(&c),
        percentile(&c, 50.0),
        percentile(&c, 95.0),
        c.last().copied().unwrap_or(f64::NAN),
        percentile(&s, 5.0),
        percentile(&s, 50.0),
        percentile(&s, 95.0),
        s.last().copied().unwrap_or(f64::NAN)
    );
    println!(
        "    chọn đúng điểm baseline: {:.1}% chuyến; có cổng OSM trong {} m quanh đích: {:.1}% chuyến; chi phí thoát (A* theo cạnh tới tâm nhu cầu gần nhất) của điểm chọn, trung bình {:.0} s",
        100.0 * default_n as f64 / tot.max(1) as f64,
        80,
        100.0 * gate_n as f64 / tot.max(1) as f64,
        mean(&exit)
    );
    // so cặp theo seed của quãng đi bộ trung bình tại điểm trả
    let per_seed = |rs: &[RunResult]| -> Vec<f64> {
        rs.iter()
            .map(|r| mean(&r.drops.iter().map(|d| d.walk_m).collect::<Vec<_>>()))
            .collect()
    };
    let p = paired(&per_seed(base), &per_seed(new));
    println!(
        "    đi bộ ở điểm trả TB theo seed: baseline {:.1} → {:.1} m, chênh cặp {:+.1} [{:+.1}, {:+.1}] (KTC t 95%, {} seed){}",
        p.mean_base,
        p.mean_new,
        p.mean_diff,
        p.t_lo,
        p.t_hi,
        p.n,
        if p.significant() { " *" } else { "" }
    );
}

fn main() {
    let args = Args::from_env();
    let env = setup(&args);
    println!("== LOC-38: tối ưu điểm trả ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let eval = seeds(&args, false);
    let j = jobs(&args);
    println!(
        "{}",
        config_line(
            &env,
            &eval,
            &format!(
                "jobs {j}; baseline = AtPin + NearestStraightLine, trả đúng điểm đích đã snap; trên dữ liệu mô phỏng"
            )
        )
    );
    println!(
        "  Giới hạn của mô hình: «đích thật» là điểm sinh ngẫu nhiên cách điểm đón 1.500–8.000 m chim bay rồi snap lên đường (cách đường tới 120 m); cổng lấy từ OSM entrance=* nếu có trong 80 m (thưa); chi phí thoát dùng tâm điểm nóng của kịch bản làm đại diện cho chuyến kế (mô phỏng không biết khách kế tiếp lúc trả khách); quãng khách đi bộ sau khi xuống xe KHÔNG nằm trong chỉ số §3.4 của mô phỏng nên được đo riêng bằng nhật ký của chiến lược."
    );
    let base = par_seeds(j, &eval, |s| {
        run_baseline(&env.engine, &with_seed(&env.scenario, s), true)
    });
    let w = ObjectiveWeights::default();
    let variants: Vec<(&str, JConfig)> = {
        let mut a = JConfig::new("drop-only", w.clone());
        a.pickup_j = false;
        a.dropoff = Some(DropoffPolicy::default());
        let mut b = JConfig::new("drop-no-exit", w.clone());
        b.pickup_j = false;
        b.dropoff = Some(DropoffPolicy {
            use_exit: false,
            ..DropoffPolicy::default()
        });
        let mut c = JConfig::new("drop-walk-only", w.clone());
        c.pickup_j = false;
        c.dropoff = Some(DropoffPolicy {
            use_exit: false,
            d_trip: 0.0,
            use_side: false,
            ..DropoffPolicy::default()
        });
        let mut d = JConfig::new("pickup+drop", w);
        d.dropoff = Some(DropoffPolicy::default());
        vec![
            ("(a) chỉ điểm trả J_trả đầy đủ (điểm đón vẫn là ghim)", a),
            ("(b) điểm trả không tính chi phí thoát của tài xế", b),
            (
                "(c) chỉ tối thiểu quãng khách đi bộ từ điểm trả (bỏ trip, thoát, phía đường)",
                c,
            ),
            ("(d) điểm đón J (p04-v1) + điểm trả J_trả", d),
        ]
    };
    for (name, cfg) in &variants {
        let runs = par_seeds(j, &eval, |s| {
            run_j(&env.engine, &with_seed(&env.scenario, s), cfg)
        });
        println!();
        print_paired(&format!("  {name}"), &base, &runs);
        print_who_pays(&base, &runs);
        drop_summary("điểm trả", &base, &runs);
    }
    println!("\nuptime lúc kết thúc: {}", uptime());
}
