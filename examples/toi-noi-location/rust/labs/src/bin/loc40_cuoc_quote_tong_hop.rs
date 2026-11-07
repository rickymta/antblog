//! LOC-40: đổi điểm đón thì cước ra sao, và tổng hợp P04. Cước 12.000 + 4.500/km (số nguyên), snapshot quote, không tăng giá lén;
//! rồi đo toàn bộ chính sách `p04-v1` trên seed đánh giá, dải seed xác nhận chưa từng nhìn, và bốn bộ giả định khác.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc40_cuoc_quote_tong_hop
//! Tham số: --fixture, --jobs N, --seeds N (mặc định 20), --skip-sensitivity

#![allow(clippy::print_literal)] // tiêu đề cột bảng viết thẳng trong chuỗi định dạng cho dễ đọc

use labs::p03::{mean, uptime};
use labs::p04::*;
use labs::{Args, percentile};
use pickup::dropoff::DropoffPolicy;
use pickup::dynamic::consent_probability;
use pickup::fare::{
    FareEffect, QuoteSnapshot, fare_vnd, message_for_customer, message_for_driver, propose_move,
    settle,
};
use pickup::objective::ObjectiveWeights;
use pickup::stats::paired;
use pickup::strategy::JConfig;
use simulator::Scenario;

/// Dải seed xác nhận: chưa từng dùng để chỉnh hay xem kết quả khi chọn trọng số.
const CONFIRM_SEEDS: std::ops::RangeInclusive<u64> = 20_270_101..=20_270_120;

fn fare_section(env: &Env, base: &[RunResult], new: &[RunResult]) {
    println!("\n-- 1. mô hình cước và snapshot quote --");
    println!(
        "  cước = 12.000 + 4.500/km vượt 2 km, số nguyên: fare_vnd(m) = 12000 + (max(0, m − 2000)·4500 + 500) / 1000"
    );
    for m in [1_500u64, 2_000, 2_001, 3_000, 5_000, 6_392, 12_133] {
        println!(
            "    {:>6} m → {:>6} đồng",
            m,
            pickup::fmt_thousands(fare_vnd(m))
        );
    }
    let w = ObjectiveWeights::default();
    let snap = w.snapshot();
    println!(
        "  chính sách điểm đón đi kèm quote: {}",
        serde_json::to_string(&snap).expect("json")
    );
    println!("  mã băm chính sách (FNV-1a): {:016x}", snap.fingerprint());
    let sc = &env.scenario;
    let world = simulator::world::generate(&env.engine, &with_seed(sc, *EVAL_SEEDS.start()));
    if let Some(c) = world.customers.first() {
        let q = QuoteSnapshot::new(
            "Q-20270001-0000",
            c.view.pin,
            c.view.dropoff,
            6_392,
            snap.clone(),
        );
        println!(
            "  ví dụ snapshot quote (bất biến, lưu lúc đặt xe): {}",
            serde_json::to_string(&q).expect("json")
        );
        let p = propose_move(&q, 6_850, 90);
        println!(
            "  đề xuất đổi điểm đón: {}",
            serde_json::to_string(&p).expect("json")
        );
        println!("  câu cho khách (cước tăng): {}", message_for_customer(&p));
        println!(
            "  câu cho khách (cước giảm): {}",
            message_for_customer(&propose_move(&q, 5_900, 90))
        );
        println!("  câu cho tài xế: {}", message_for_driver(-180));
        println!(
            "  hợp đồng: khách không đồng ý/chưa trả lời → trả {} đồng (cước đã báo), nền tảng chịu {} đồng; đồng ý → trả {} đồng",
            pickup::fmt_thousands(settle(&q, 6_850, None).charged_vnd),
            pickup::fmt_thousands(settle(&q, 6_850, None).absorbed_vnd),
            pickup::fmt_thousands(settle(&q, 6_850, Some(true)).charged_vnd)
        );
        assert_eq!(p.effect, FareEffect::NeedsConsent);
    }

    println!(
        "\n-- 2. đổi điểm đón làm cước đổi bao nhiêu (J p04-v1 so với baseline, cặp theo khách, {} seed) --",
        base.len()
    );
    println!(
        "  quote = cước theo quãng có khách của baseline (đón đúng ghim); cước thật = theo quãng có khách của chuyến với điểm đón mới. Chỉ tính khách hoàn thành ở cả hai."
    );
    let (mut rise, mut same, mut fall) = (0usize, 0usize, 0usize);
    let (mut deltas, mut quote_sum, mut actual_sum, mut absorbed, mut charged_consent) =
        (Vec::new(), 0u64, 0u64, 0u64, 0u64);
    let mut declined = 0usize;
    for (b, n) in base.iter().zip(new) {
        for (tb, tn) in customer_pairs(&b.out, &n.out) {
            let (q, a) = (
                fare_vnd(tb.trip_m.round() as u64),
                fare_vnd(tn.trip_m.round() as u64),
            );
            let d = a as i64 - q as i64;
            deltas.push(d as f64);
            match d.cmp(&0) {
                std::cmp::Ordering::Greater => rise += 1,
                std::cmp::Ordering::Equal => same += 1,
                std::cmp::Ordering::Less => fall += 1,
            }
            quote_sum += q;
            actual_sum += a;
            let quote = QuoteSnapshot::new(
                "Q",
                env.engine.g.center(),
                env.engine.g.center(),
                tb.trip_m.round() as u64,
                ObjectiveWeights::default().snapshot(),
            );
            absorbed += settle(&quote, tn.trip_m.round() as u64, None).absorbed_vnd;
            // mô hình đồng ý (giả định): xác suất theo quãng đi bộ thêm; rút bằng băm tất định (seed, khách)
            if d > 0 {
                let extra = (tn.walk_m - tb.walk_m).max(0.0);
                let h = (b.out.scenario.seed
                    ^ (u64::from(tb.customer) + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15))
                .wrapping_mul(0xBF58_476D_1CE4_E5B9);
                let u = (h >> 11) as f64 / (1u64 << 53) as f64;
                let consent = u < consent_probability(extra);
                declined += usize::from(!consent);
                charged_consent +=
                    settle(&quote, tn.trip_m.round() as u64, Some(consent)).charged_vnd;
            } else {
                charged_consent += a;
            }
        }
    }
    let n_all = deltas.len().max(1);
    deltas.sort_by(f64::total_cmp);
    println!(
        "  {} chuyến ghép cặp: cước tăng {:.1}%, bằng nhau {:.1}%, giảm {:.1}%; chênh theo chuyến: TB {:+.0}, p5 {:+.0}, p50 {:+.0}, p95 {:+.0}, min {:+.0}, max {:+.0} đồng",
        n_all,
        100.0 * rise as f64 / n_all as f64,
        100.0 * same as f64 / n_all as f64,
        100.0 * fall as f64 / n_all as f64,
        mean(&deltas),
        percentile(&deltas, 5.0),
        percentile(&deltas, 50.0),
        percentile(&deltas, 95.0),
        deltas.first().copied().unwrap_or(f64::NAN),
        deltas.last().copied().unwrap_or(f64::NAN)
    );
    println!(
        "  tổng cước theo quote (baseline) {} đồng; tổng cước theo quãng thật nếu tính hết {} đồng ({:+.2}%)",
        pickup::fmt_thousands(quote_sum),
        pickup::fmt_thousands(actual_sum),
        100.0 * (actual_sum as f64 / quote_sum.max(1) as f64 - 1.0)
    );
    println!(
        "  KHÔNG TĂNG GIÁ LÉN: khách chưa đồng ý chỉ trả tối đa cước đã báo; phần chênh nền tảng chịu = {} đồng ({:.2}% tổng quote); tổng khách trả = {} đồng ({:+.2}% so với quote)",
        pickup::fmt_thousands(absorbed),
        100.0 * absorbed as f64 / quote_sum.max(1) as f64,
        pickup::fmt_thousands(actual_sum - absorbed),
        100.0 * ((actual_sum - absorbed) as f64 / quote_sum.max(1) as f64 - 1.0)
    );
    println!(
        "  nếu hỏi khách (mô hình đồng ý giả định: 0,95 − 0,004 × mét đi thêm): {} chuyến tăng cước bị từ chối; tổng khách trả {} đồng ({:+.2}% so với quote)",
        declined,
        pickup::fmt_thousands(charged_consent),
        100.0 * (charged_consent as f64 / quote_sum.max(1) as f64 - 1.0)
    );
    println!(
        "  lưu ý: cước chỉ phụ thuộc quãng có khách (trip_m), nên điểm đón gần đích hơn làm cước giảm tự động, điểm đón xa đích hơn mới tăng; doanh thu mỗi chuyến của nền tảng/tài xế giảm khi quãng có khách ngắn đi"
    );
}

const KEY: [&str; 13] = [
    "walk_m trung bình",
    "walk_m p50",
    "walk_m p95",
    "walk_m max",
    "deadhead_m trung bình",
    "deadhead_m p95",
    "pickup_eta_s trung bình",
    "pickup_eta_s p95",
    "meet_gap_s trung bình",
    "u_turns mỗi chuyến",
    "cancel_rate",
    "chuyến hoàn thành",
    "walk_m + deadhead_m mỗi chuyến",
];

/// Một dòng gọn cho mỗi chỉ số chính: chênh cặp và khoảng tin cậy t; trả dấu (lợi/thiệt/không rõ).
fn compact(title: &str, base: &[RunResult], new: &[RunResult]) -> Vec<(String, char)> {
    println!("{title}");
    println!(
        "  {:<32} {:>10} {:>10} {:>10} {:>22}  {}",
        "chỉ số", "baseline", "chiến lược", "chênh TB", "KTC 95% (t)", "kết luận"
    );
    let mut out = Vec::new();
    for (name, get) in metric_defs() {
        if !KEY.contains(&name) {
            continue;
        }
        let b: Vec<f64> = base.iter().map(|r| get(&r.out)).collect();
        let n: Vec<f64> = new.iter().map(|r| get(&r.out)).collect();
        let p = paired(&b, &n);
        let higher_is_better = name == "chuyến hoàn thành";
        let sign = if !p.significant() {
            '0'
        } else if (p.mean_diff > 0.0) == higher_is_better {
            '+'
        } else {
            '-'
        };
        let d = if name == "cancel_rate" { 4 } else { 1 };
        println!(
            "  {:<32} {:>10.d$} {:>10.d$} {:>+10.d$} {:>22}  {}",
            name,
            p.mean_base,
            p.mean_new,
            p.mean_diff,
            format!("[{:+.d$}, {:+.d$}]", p.t_lo, p.t_hi),
            match sign {
                '+' => "tốt hơn",
                '-' => "XẤU hơn",
                _ => "không khác biệt rõ",
            }
        );
        out.push((name.to_string(), sign));
    }
    out
}

fn main() {
    let args = Args::from_env();
    let env = setup(&args);
    println!("== LOC-40: cước và quote khi đổi điểm đón; tổng hợp P04 ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    let eval = seeds(&args, false);
    let j = jobs(&args);
    let confirm: Vec<u64> = CONFIRM_SEEDS.take(eval.len()).collect();
    println!(
        "{}",
        config_line(
            &env,
            &eval,
            &format!(
                "jobs {j}; baseline = AtPin + NearestStraightLine; p04-v1 (a=b=c=d=1); seed xác nhận {}…{}; trên dữ liệu mô phỏng",
                confirm[0],
                confirm[confirm.len() - 1]
            )
        )
    );
    let w = ObjectiveWeights::default();
    let pick = JConfig::new("p04-v1-pickup", w.clone());
    let mut both = JConfig::new("p04-v1-pickup+drop", w.clone());
    both.dropoff = Some(DropoffPolicy::default());

    let base = par_seeds(j, &eval, |s| {
        run_baseline(&env.engine, &with_seed(&env.scenario, s), true)
    });
    let r_pick = par_seeds(j, &eval, |s| {
        run_j(&env.engine, &with_seed(&env.scenario, s), &pick)
    });
    fare_section(&env, &base, &r_pick);

    println!("\n-- 3. tổng hợp P04 trên seed ĐÁNH GIÁ: điểm đón p04-v1 --");
    print_paired("  mọi chỉ số, so sánh cặp:", &base, &r_pick);
    print_who_pays(&base, &r_pick);

    println!("\n-- 4. tổng hợp: điểm đón + điểm trả --");
    let r_both = par_seeds(j, &eval, |s| {
        run_j(&env.engine, &with_seed(&env.scenario, s), &both)
    });
    print_paired("  mọi chỉ số, so sánh cặp:", &base, &r_both);
    print_who_pays(&base, &r_both);

    println!("\n-- 5. dải seed XÁC NHẬN (chưa từng nhìn khi chọn trọng số hay sửa quy tắc) --");
    println!(
        "  {}",
        config_line(
            &env,
            &confirm,
            "cùng cấu hình p04-v1 điểm đón, không chỉnh gì thêm"
        )
    );
    let base_c = par_seeds(j, &confirm, |s| {
        run_baseline(&env.engine, &with_seed(&env.scenario, s), true)
    });
    let r_c = par_seeds(j, &confirm, |s| {
        run_j(&env.engine, &with_seed(&env.scenario, s), &pick)
    });
    print_paired("  điểm đón p04-v1, seed xác nhận:", &base_c, &r_c);
    print_who_pays(&base_c, &r_c);
    let r_cb = par_seeds(j, &confirm, |s| {
        run_j(&env.engine, &with_seed(&env.scenario, s), &both)
    });
    println!();
    compact("  điểm đón + điểm trả, seed xác nhận:", &base_c, &r_cb);

    if !args.flag("--skip-sensitivity") {
        println!(
            "\n-- 6. độ nhạy với giả định (mỗi bộ đổi MỘT giả định; baseline chạy lại với cùng giả định; điểm đón p04-v1; {} seed đánh giá) --",
            eval.len()
        );
        let mut summary: Vec<(String, Vec<(String, char)>)> = Vec::new();
        let mut vs: Vec<(String, String, Scenario)> = variants()
            .into_iter()
            .skip(1)
            .map(|(n, d, f)| {
                let mut sc = env.scenario.clone();
                f(&mut sc);
                (n.to_string(), d.to_string(), sc)
            })
            .collect();
        // nhiễu GPS của điện thoại: chỉ chiến lược thấy vị trí lệch, khách vẫn đi từ chỗ thật
        vs.push((
            "GPS nhiễu 15 m".into(),
            "chiến lược thấy chỗ đứng lệch σ = 15 m mỗi trục (baseline không dùng GPS)".into(),
            env.scenario.clone(),
        ));
        for (name, desc, sc) in &vs {
            let cfg = if name == "GPS nhiễu 15 m" {
                let mut c = pick.clone();
                c.gps_noise_m = 15.0;
                c
            } else {
                pick.clone()
            };
            let b = par_seeds(j, &eval, |s| {
                run_baseline(&env.engine, &with_seed(sc, s), false)
            });
            let r = par_seeds(j, &eval, |s| run_j(&env.engine, &with_seed(sc, s), &cfg));
            println!();
            let sg = compact(&format!("  bộ giả định «{name}»: {desc}"), &b, &r);
            summary.push((name.clone(), sg));
        }
        // chính sách gốc làm cột đầu
        let mut sg0 = Vec::new();
        for (name, get) in metric_defs() {
            if !KEY.contains(&name) {
                continue;
            }
            let bb: Vec<f64> = base.iter().map(|r| get(&r.out)).collect();
            let nn: Vec<f64> = r_pick.iter().map(|r| get(&r.out)).collect();
            let p = paired(&bb, &nn);
            let hib = name == "chuyến hoàn thành";
            sg0.push((
                name.to_string(),
                if !p.significant() {
                    '0'
                } else if (p.mean_diff > 0.0) == hib {
                    '+'
                } else {
                    '-'
                },
            ));
        }
        summary.insert(0, ("gốc".into(), sg0));
        println!(
            "\n  tóm tắt độ nhạy: «+» tốt hơn baseline (KTC 95% t không chứa 0), «-» XẤU hơn, «0» không khác biệt rõ"
        );
        print!("  {:<32}", "chỉ số");
        for (n, _) in &summary {
            print!(" {:>16}", n);
        }
        println!();
        for (i, name) in KEY.iter().enumerate() {
            print!("  {:<32}", name);
            for (_, sg) in &summary {
                print!(" {:>16}", sg.get(i).map_or('?', |x| x.1));
            }
            println!();
        }
        let robust: Vec<&str> = KEY
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                summary
                    .iter()
                    .all(|(_, sg)| sg.get(*i).is_some_and(|x| x.1 == '+'))
            })
            .map(|(_, n)| *n)
            .collect();
        println!("  chỉ số tốt hơn baseline ở MỌI bộ giả định: {robust:?}");
        let bad: Vec<String> = KEY
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                summary
                    .iter()
                    .any(|(_, sg)| sg.get(*i).is_some_and(|x| x.1 == '-'))
            })
            .map(|(_, n)| n.to_string())
            .collect();
        println!("  chỉ số xấu hơn baseline ở ÍT NHẤT một bộ giả định: {bad:?}");
    }
    println!("\nuptime lúc kết thúc: {}", uptime());
}
