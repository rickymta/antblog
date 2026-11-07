//! LOC-56: thí nghiệm trên thị trường hai phía trong simulator. Chạy một «thay đổi» (treatment) ở sáu cách gán:
//! `off` (không ai được), `on` (mọi người được) = sự thật toàn cục, `ab` (ngẫu nhiên theo khách), `sb_zt` (switchback theo vùng 2×2 và
//! khung 10 phút), `sb_t10`, `sb_t20` (switchback theo khung 10 và 20 phút trên toàn thành phố). Cùng seed ⇒ cùng dòng khách, nên
//! `on − off` là chênh lệch toàn cục đo được có cặp. Phân tích (ước lượng, thiên lệch, KTC, tiêu chí dừng) làm ở Python.
//!
//! Hai thay đổi:
//! - `priority`: khách được thay đổi xếp hàng trước khách khác khi tài xế rảnh (như hạng ưu tiên). Bật cho MỌI người thì thứ tự
//!   giống hệt baseline, nên chênh lệch toàn cục bằng đúng 0 theo cấu trúc: mọi «hiệu quả» mà A/B báo là thiên lệch do nhiễu giữa nhóm.
//!   Đây là thay đổi do lab CHỌN vì nó làm nhiễu giữa hai nhóm rõ nhất; nó không đại diện cho mọi thí nghiệm.
//! - `eta`: khách được thay đổi chọn tài xế gần nhất theo ETA đường bộ, khách khác theo đường chim bay (LOC-41). Cải thiện thật, nhỏ.
//!
//! Chạy từ gốc repo:
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc56_ab_switchback -- --treatment priority --variant "thưa xe" --tag thua_xe
//! Tham số: --treatment priority|eta  --variant tên  --tag tên_thư_mục  --seeds N (40)  --jobs N (4)  --fixture

use geo_core::LatLng;
use labs::p03::uptime;
use labs::p05::{jobs, select_variants, setup};
use labs::{Args, die, lab_root};
use matcher::batch::{BatchMatcher, MatcherConfig};
use matcher::eval::{Zones, eval_seeds, metrics_of};
use route_engine::engine::Engine;
use simulator::metrics::Outcome;
use simulator::rng::Rng;
use simulator::strategy::{
    Assignment, Ctx, DriverView, MatchStrategy, PendingView, nearest_straight_line,
};
use simulator::{AtPin, RunOptions, Scenario, run};
use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Treat {
    Priority,
    Eta,
}

/// Quy tắc gán khách vào nhóm được thay đổi.
#[derive(Clone)]
enum Rule {
    Off,
    On,
    /// A/B ngẫu nhiên theo khách (50%).
    Ab {
        salt: u64,
    },
    /// Switchback: đơn vị = (vùng lưới `grid`×`grid`, khung `window_s` giây theo giờ đặt). Mỗi vùng được gán cân bằng (một nửa số khung bật).
    Switch {
        grid: u32,
        window_s: u64,
        bbox: [f64; 4],
        table: Arc<Vec<bool>>,
        n_windows: usize,
    },
}

fn zone_of(bbox: &[f64; 4], grid: u32, p: LatLng) -> usize {
    let fx = ((p.lat() - bbox[0]) / (bbox[1] - bbox[0])).clamp(0.0, 0.999_999);
    let fy = ((p.lng() - bbox[2]) / (bbox[3] - bbox[2])).clamp(0.0, 0.999_999);
    ((fx * f64::from(grid)) as usize) * grid as usize + (fy * f64::from(grid)) as usize
}

impl Rule {
    fn switch(seed: u64, salt: u64, grid: u32, window_s: u64, bbox: [f64; 4]) -> Self {
        let n_windows = 3600usize.div_ceil(window_s as usize);
        let zones = (grid * grid) as usize;
        let mut rng = Rng::new(seed ^ salt, 56);
        let mut table = vec![false; zones * n_windows];
        for z in 0..zones {
            // Fisher–Yates trên các khung của vùng, rồi bật một nửa đầu (khung dư, nếu có, bật ngẫu nhiên)
            let mut idx: Vec<usize> = (0..n_windows).collect();
            for i in (1..n_windows).rev() {
                idx.swap(i, rng.usize(i + 1));
            }
            let on = n_windows / 2 + usize::from(n_windows % 2 == 1 && rng.f64() < 0.5);
            for &w in idx.iter().take(on) {
                table[z * n_windows + w] = true;
            }
        }
        Rule::Switch {
            grid,
            window_s,
            bbox,
            table: Arc::new(table),
            n_windows,
        }
    }

    fn treated(&self, customer: u32, request_ms: u64, pin: LatLng) -> bool {
        match self {
            Rule::Off => false,
            Rule::On => true,
            Rule::Ab { salt } => simulator_hash(*salt ^ u64::from(customer)) & 1 == 1,
            Rule::Switch {
                grid,
                window_s,
                bbox,
                table,
                n_windows,
            } => {
                let w = ((request_ms / 1000 / window_s) as usize).min(n_windows - 1);
                table[zone_of(bbox, *grid, pin) * n_windows + w]
            }
        }
    }

    fn unit(&self, request_ms: u64, pin: LatLng) -> (usize, usize) {
        match self {
            Rule::Switch {
                grid,
                window_s,
                bbox,
                n_windows,
                ..
            } => (
                zone_of(bbox, *grid, pin),
                ((request_ms / 1000 / window_s) as usize).min(n_windows - 1),
            ),
            _ => (0, 0),
        }
    }
}

fn simulator_hash(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Chiến lược ghép chuyến có hai nhóm khách. Mọi khách dùng chung một bể tài xế rảnh: chính điều đó gây nhiễu giữa hai nhóm.
struct Mixed {
    treat: Treat,
    rule: Rule,
    eta: BatchMatcher,
}

impl MatchStrategy for Mixed {
    fn name(&self) -> &str {
        match self.treat {
            Treat::Priority => "mixed-priority",
            Treat::Eta => "mixed-eta",
        }
    }
    fn assign(
        &mut self,
        ctx: &Ctx,
        pending: &[PendingView],
        idle: &[DriverView],
    ) -> Vec<Assignment> {
        let treated = |p: &PendingView| {
            self.rule
                .treated(p.customer, p.request_ms, p.pickup.stop_point)
        };
        match self.treat {
            Treat::Priority => {
                // khách được ưu tiên xếp trước, còn lại giữ thứ tự đặt; mỗi khách lấy tài xế gần nhất theo chim bay
                let mut order: Vec<PendingView> =
                    pending.iter().copied().filter(|p| treated(p)).collect();
                order.extend(pending.iter().copied().filter(|p| !treated(p)));
                nearest_straight_line(&order, idle)
            }
            Treat::Eta => {
                // giữ thứ tự đặt; khách được thay đổi chọn theo ETA đường bộ, khách khác theo chim bay
                let mut remaining: Vec<DriverView> = idle.to_vec();
                let mut out = Vec::new();
                for p in pending {
                    if remaining.is_empty() {
                        break;
                    }
                    let pick = if treated(p) {
                        self.eta.assign(ctx, std::slice::from_ref(p), &remaining)
                    } else {
                        nearest_straight_line(std::slice::from_ref(p), &remaining)
                    };
                    if let Some(a) = pick.first() {
                        remaining.retain(|d| d.id != a.driver);
                        out.push(*a);
                    }
                }
                out
            }
        }
    }
}

fn arm_rules(seed: u64, bbox: [f64; 4]) -> Vec<(&'static str, Rule)> {
    vec![
        ("off", Rule::Off),
        ("on", Rule::On),
        ("ab", Rule::Ab { salt: seed ^ 0xAB }),
        ("sb_zt", Rule::switch(seed, 0x5B01, 2, 600, bbox)),
        ("sb_t10", Rule::switch(seed, 0x5B02, 1, 600, bbox)),
        ("sb_t20", Rule::switch(seed, 0x5B03, 1, 1200, bbox)),
    ]
}

struct Out {
    customers: String,
    runs: String,
}

fn one_seed(
    engine: &Engine,
    sc: &Scenario,
    treat: Treat,
    tag: &str,
    seed: u64,
    zones: &Zones,
) -> Out {
    let mut s = sc.clone();
    s.seed = seed;
    let bb = engine.g.node_bbox();
    let bbox = [bb[0], bb[1], bb[2], bb[3]];
    let mut customers = String::new();
    let mut runs = String::new();
    for (arm, rule) in arm_rules(seed, bbox) {
        let (eta, _) = BatchMatcher::new(MatcherConfig::nearest_eta());
        let t0 = Instant::now();
        let out = run(
            engine,
            &s,
            Box::new(AtPin),
            Box::new(Mixed {
                treat,
                rule: rule.clone(),
                eta,
            }),
            RunOptions { trace: true },
        );
        let wall = t0.elapsed().as_secs_f64();
        let m = metrics_of(&out, zones, seed, wall);
        let _ = writeln!(
            runs,
            "{tag},{seed},{arm},{},{},{:.6},{:.6},{:.3},{:.3},{:.3},{:.3},{:.5},{:.5},{:.1},{:.5},{:.2}",
            m.requests,
            m.completed,
            m.cancel_rate,
            m.unserved_rate,
            m.wait_mean,
            m.eta_mean,
            m.deadhead_mean,
            m.empty_m_per_request,
            m.trips_gini,
            m.zero_trip_share,
            m.idle_gap_p95_s,
            out.summary.driver_busy_share,
            wall
        );
        // vị trí ghim của khách từ nhật ký sự kiện
        let mut pin: Vec<Option<LatLng>> = vec![None; out.summary.requests];
        for e in out.trace.iter().filter(|e| e.kind == "request") {
            if let (Some(c), Ok(p)) = (e.customer, LatLng::try_new(e.lat, e.lng)) {
                pin[c as usize] = Some(p);
            }
        }
        for r in &out.trips {
            let Some(p) = pin[r.customer as usize] else {
                continue;
            };
            let req_ms = (r.request_s * 1000.0).round() as u64;
            let treated = rule.treated(r.customer, req_ms, p);
            let (z, w) = rule.unit(req_ms, p);
            // thời gian chờ có chặn: huỷ vì hết kiên nhẫn tính bằng kiên nhẫn của khách; huỷ sau khi gán tính tới lúc huỷ
            let wait_cens = match r.outcome {
                Outcome::Completed | Outcome::CancelledAfterAssign => r.wait_s,
                Outcome::CancelledNoDriver => s.customer.patience_s,
                Outcome::CancelledNoRoute => f64::NAN,
            };
            let _ = writeln!(
                customers,
                "{tag},{seed},{arm},{},{:.1},{:.6},{:.6},{},{},{},{},{},{:.3},{:.3},{:.1}",
                r.customer,
                r.request_s,
                p.lat(),
                p.lng(),
                u8::from(treated),
                z,
                w,
                r.outcome.as_str(),
                u8::from(r.outcome == Outcome::Completed),
                wait_cens,
                r.pickup_eta_s,
                r.deadhead_m
            );
        }
    }
    Out { customers, runs }
}

fn main() {
    let args = Args::from_env();
    let treat = match args.value("--treatment").unwrap_or("priority") {
        "priority" => Treat::Priority,
        "eta" => Treat::Eta,
        x => die(&format!("--treatment không hợp lệ: {x}")),
    };
    println!(
        "== LOC-56: A/B theo khách so với switchback, và sự thật toàn cục (trên dữ liệu mô phỏng) =="
    );
    println!("uptime lúc bắt đầu: {}", uptime());
    let (engine, info, sc, fixture) = setup(&args);
    let vname = args.value("--variant").unwrap_or("thưa xe").to_string();
    let (_, vdesc, tweak) = select_variants(&args, &[vname.as_str()])
        .into_iter()
        .next()
        .unwrap_or_else(|| die("không có bộ giả định tên này"));
    let mut base = sc.clone();
    tweak(&mut base);
    let tag = args
        .value("--tag")
        .map_or_else(|| vname.replace(' ', "_"), str::to_string);
    let n = args.u64_or("--seeds", if fixture { 4 } else { 40 }) as usize;
    let seeds = eval_seeds(n);
    let tname = match treat {
        Treat::Priority => "priority",
        Treat::Eta => "eta",
    };
    let out_dir: PathBuf = args.value("--out").map_or_else(
        || {
            lab_root()
                .join("python/p06/artifacts/loc56")
                .join(format!("{tname}_{tag}"))
        },
        PathBuf::from,
    );
    println!(
        "đồ thị: {info}; kịch bản gốc {}; bộ giả định «{vname}»: {vdesc}",
        sc.name
    );
    println!(
        "thay đổi: {tname}; {n} seed đánh giá {}…{}; sáu cách gán: off, on, ab, sb_zt (2×2 vùng, 10 phút), sb_t10, sb_t20",
        seeds[0],
        seeds[n - 1]
    );
    let zones = Zones::hcm();
    let jobs = jobs(&args);
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<Option<Out>>> = Mutex::new((0..n).map(|_| None).collect());
    let t0 = Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..jobs.max(1).min(n) {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::SeqCst);
                    if i >= n {
                        break;
                    }
                    let o = one_seed(&engine, &base, treat, &tag, seeds[i], &zones);
                    results.lock().expect("khoá")[i] = Some(o);
                    eprintln!(
                        "  xong seed {} ({}/{n}), {:.0} s",
                        seeds[i],
                        i + 1,
                        t0.elapsed().as_secs_f64()
                    );
                }
            });
        }
    });
    std::fs::create_dir_all(&out_dir).unwrap_or_else(|e| die(&format!("{e}")));
    let mut c = String::from(
        "tag,seed,arm,customer,request_s,pin_lat,pin_lng,treated,zone,window,outcome,completed,wait_cens_s,pickup_eta_s,deadhead_m\n",
    );
    let mut r = String::from(
        "tag,seed,arm,requests,completed,cancel_rate,unserved_rate,wait_mean,eta_mean,deadhead_mean,empty_m_per_request,trips_gini,zero_trip_share,idle_gap_p95_s,busy_share,wall_s\n",
    );
    for o in results.into_inner().expect("khoá").into_iter().flatten() {
        c += &o.customers;
        r += &o.runs;
    }
    std::fs::write(out_dir.join("customers.csv"), &c).unwrap_or_else(|e| die(&format!("{e}")));
    std::fs::write(out_dir.join("runs.csv"), &r).unwrap_or_else(|e| die(&format!("{e}")));
    println!(
        "đã ghi {}/{{customers,runs}}.csv: {} dòng khách, {} dòng chạy; {:.0} s",
        out_dir.display(),
        c.lines().count() - 1,
        r.lines().count() - 1,
        t0.elapsed().as_secs_f64()
    );
    println!("uptime lúc kết thúc: {}", uptime());
}
