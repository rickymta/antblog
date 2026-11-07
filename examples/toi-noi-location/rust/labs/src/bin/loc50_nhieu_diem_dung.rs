//! LOC-50: nhiều điểm dừng có ràng buộc lấy trước giao: chèn + 2-opt/Or-opt trong Rust, đối chiếu OR-Tools (Python, offline) và quy hoạch động chính xác.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin; `--fixture` chạy offline trên đồ thị tổng hợp):
//!   cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc50_nhieu_diem_dung
//! Quy trình đủ ba bước (OR-Tools chạy riêng, không nằm trên đường nóng):
//!   1. `... --write-instances`  sinh instance trên HCM, ghi fixtures/loc50/instances.json
//!   2. `cd examples/toi-noi-location/python/loc50 && uv run python loc50_ortools.py`  ghi fixtures/loc50/ortools-results.json
//!   3. `...` (không tham số)  sinh lại đúng instance đó, giải bằng Rust, so với OR-Tools và với DP (n ≤ 8)
//!
//! Tham số: --fixture, --write-instances, --no-python (bỏ qua kết quả OR-Tools), --instances N (mặc định 20),
//! --sizes 4,8,12,20,30, --ortools FILE.

use geo_core::LatLng;
use labs::p03::{mean, uptime, vn};
use labs::{Args, die, lab_root, load_graph};
use matcher::multistop::{
    DP_MAX_PAIRS, Instance, UNREACHABLE_MS, cheapest_insertion, exact_dp, solve,
};
use route_engine::engine::{Engine, EngineConfig};
use route_engine::net::INF;
use route_engine::profile::Profile;
use simulator::Scenario;
use simulator::world::generate;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

/// Dải seed của instance: 50001, 50002, … (một seed một «ngày»: mọi cỡ n lấy n cặp đầu của cùng một thế giới mô phỏng).
const SEED0: u64 = 50_001;

struct Gen {
    id: String,
    n: usize,
    seed: u64,
    inst: Instance,
}

/// FNV-1a 64 bit trên chuỗi `a,b,c,…` của ma trận hàng-chính: cùng công thức ở phía Python để chắc hai bên giải cùng một ma trận.
fn matrix_fnv(m: &[i64]) -> String {
    let text = m.iter().map(i64::to_string).collect::<Vec<_>>().join(",");
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in text.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{h:016x}")
}

/// Sinh `count` instance cho mỗi cỡ: điểm lấy = ghim của khách, điểm giao = điểm trả của khách (từ `simulator::world::generate`),
/// điểm xuất phát = vị trí tài xế 0. Ma trận bất đối xứng từ CH ô tô (mili giây).
fn generate_instances(engine: &Engine, base: &Scenario, sizes: &[usize], count: usize) -> Vec<Gen> {
    let max_n = *sizes.iter().max().expect("có cỡ");
    let mut out = Vec::new();
    for i in 0..count {
        let seed = SEED0 + i as u64;
        let mut sc = base.clone();
        sc.seed = seed;
        sc.duration_s = sc.duration_s.max(1800.0);
        sc.demand.requests_per_min = sc.demand.requests_per_min.max(2.5);
        let w = generate(engine, &sc);
        if w.customers.len() < max_n {
            die(&format!(
                "seed {seed}: chỉ sinh được {} khách, cần {max_n}",
                w.customers.len()
            ));
        }
        let start: LatLng = w.drivers[0].pos.point(&engine.g);
        for &n in sizes {
            let mut pts = vec![start];
            for c in &w.customers[..n] {
                pts.push(c.view.pin);
                pts.push(c.view.dropoff);
            }
            let m = engine.matrix(Profile::Car, &pts, &pts, None).matrix;
            let k = pts.len();
            let mut cost = vec![0i64; k * k];
            for a in 0..k {
                for b in 0..k {
                    cost[a * k + b] = if a == b {
                        0
                    } else {
                        let v = m.get(a, b);
                        if v == INF {
                            UNREACHABLE_MS
                        } else {
                            i64::from(v)
                        }
                    };
                }
            }
            out.push(Gen {
                id: format!("n{n:02}-{i:02}"),
                n,
                seed,
                inst: Instance::new(n, cost).expect("ma trận hợp lệ"),
            });
        }
    }
    out
}

fn write_instances(path: &std::path::Path, graph: &str, gens: &[Gen]) {
    let mut s = String::from("{\"format\":\"loc50-instances-v1\",");
    s += &format!(
        "\"graph\":\"{graph}\",\"seed_range\":\"{SEED0}..\",\"objective\":\"open-route total drive time ms, pickup before delivery, node 0 = start, node 1+2k = pickup k, node 2+2k = delivery k\",\"instances\":[\n"
    );
    for (i, g) in gens.iter().enumerate() {
        let k = g.inst.nodes();
        let rows: Vec<String> = (0..k)
            .map(|a| {
                let r: Vec<String> = (0..k).map(|b| g.inst.c(a, b).to_string()).collect();
                format!("[{}]", r.join(","))
            })
            .collect();
        s += &format!(
            "{{\"id\":\"{}\",\"n\":{},\"seed\":{},\"matrix_fnv\":\"{}\",\"matrix\":[{}]}}{}\n",
            g.id,
            g.n,
            g.seed,
            matrix_fnv(g.inst.matrix()),
            rows.join(","),
            if i + 1 == gens.len() { "" } else { "," }
        );
    }
    s += "]}\n";
    std::fs::write(path, s)
        .unwrap_or_else(|e| die(&format!("không ghi được {}: {e}", path.display())));
}

/// Giá trị tới hạn hai phía 95% của phân phối t.
fn t975(df: usize) -> f64 {
    const T: [f64; 30] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
        2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
        2.052, 2.048, 2.045, 2.042,
    ];
    match df {
        0 => f64::NAN,
        1..=30 => T[df - 1],
        31..=60 => 2.0,
        _ => 1.96,
    }
}

fn mean_ci(v: &[f64]) -> (f64, f64, f64) {
    let n = v.len();
    let m = mean(v);
    if n < 2 {
        return (m, f64::NAN, f64::NAN);
    }
    let var = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1) as f64;
    let half = t975(n - 1) * (var / n as f64).sqrt();
    (m, m - half, m + half)
}

fn median(v: &[f64]) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    match s.len() {
        0 => f64::NAN,
        n if n % 2 == 1 => s[n / 2],
        n => (s[n / 2 - 1] + s[n / 2]) / 2.0,
    }
}

fn maxf(v: &[f64]) -> f64 {
    v.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

/// Trung vị thời gian (µs) của `reps` lần chạy `f`.
fn time_us<T>(reps: usize, mut f: impl FnMut() -> T) -> f64 {
    let mut ts: Vec<f64> = (0..reps)
        .map(|_| {
            let t = Instant::now();
            std::hint::black_box(f());
            t.elapsed().as_secs_f64() * 1e6
        })
        .collect();
    ts.sort_by(f64::total_cmp);
    ts[ts.len() / 2]
}

struct Ort {
    cost: i64,
    route: Vec<usize>,
    status: String,
    wall_s: f64,
    fnv: String,
}

struct OrtFile {
    version: String,
    meta: String,
    by_id: BTreeMap<String, Ort>,
}

fn read_ortools(path: &std::path::Path) -> Result<OrtFile, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let mut by_id = BTreeMap::new();
    for r in v["results"].as_array().ok_or("thiếu results")? {
        by_id.insert(
            r["id"].as_str().ok_or("thiếu id")?.to_string(),
            Ort {
                cost: r["cost_ms"].as_i64().ok_or("thiếu cost_ms")?,
                route: r["route"]
                    .as_array()
                    .ok_or("thiếu route")?
                    .iter()
                    .map(|x| x.as_u64().unwrap_or(0) as usize)
                    .collect(),
                status: r["status"].as_str().unwrap_or("?").to_string(),
                wall_s: r["wall_s"].as_f64().unwrap_or(f64::NAN),
                fnv: r["matrix_fnv"].as_str().unwrap_or("").to_string(),
            },
        );
    }
    Ok(OrtFile {
        version: v["ortools_version"].as_str().unwrap_or("?").to_string(),
        meta: format!(
            "first_solution = {}, metaheuristic = {}, giới hạn thời gian theo cỡ n = {}, Python {}",
            v["first_solution_strategy"].as_str().unwrap_or("?"),
            v["metaheuristic"].as_str().unwrap_or("?"),
            v["time_limit_s_by_n"],
            v["python"].as_str().unwrap_or("?")
        ),
        by_id,
    })
}

fn pct(x: f64) -> String {
    format!("{:+.2}%", 100.0 * x)
}

fn main() {
    let args = Args::from_env();
    let fixture = args.flag("--fixture");
    println!(
        "== LOC-50: nhiều điểm dừng, lấy trước giao: chèn + 2-opt/Or-opt, đối chiếu OR-Tools =="
    );
    println!("uptime lúc bắt đầu: {}", uptime());
    let (g, source) = load_graph(&args);
    println!(
        "đồ thị: {source}: {} nút, {} cạnh",
        vn(g.node_count() as u64),
        vn(g.edge_count() as u64)
    );
    let root = lab_root();
    let sc_path = root.join(if fixture {
        "fixtures/scenario-khu-mau.toml"
    } else {
        "fixtures/scenario-hcm-baseline.toml"
    });
    let base = Scenario::load(&sc_path).unwrap_or_else(|e| die(&format!("{e}")));
    let engine = Engine::new(
        g,
        EngineConfig {
            ch_cache_dir: (!fixture).then(|| root.join("rust/target/ch-cache")),
            cache_tag: if fixture { "fixture" } else { "hcm" }.into(),
            ..EngineConfig::default()
        },
    );
    let sizes: Vec<usize> = if fixture {
        vec![4, 8]
    } else {
        args.value("--sizes").map_or_else(
            || vec![4, 8, 12, 20, 30],
            |s| {
                s.split(',')
                    .map(|x| x.parse().unwrap_or_else(|_| die("--sizes cần số nguyên")))
                    .collect()
            },
        )
    };
    let count = if fixture {
        3
    } else {
        args.u64_or("--instances", 20) as usize
    };
    println!(
        "bài toán: một người giao hàng, n cặp (lấy → giao), lấy trước giao, tuyến mở, tối thiểu tổng thời gian chạy ô tô (ms, ma trận bất đối xứng từ CH).\n\
         KHÔNG có tải trọng, khung giờ, nhiều xe. Instance trên dữ liệu mô phỏng: điểm lấy/giao = ghim/điểm trả của khách do simulator sinh, xuất phát = tài xế 0; kịch bản {}",
        sc_path.strip_prefix(&root).unwrap_or(&sc_path).display()
    );
    println!(
        "cỡ n (số cặp) = {sizes:?}; {count} instance mỗi cỡ; seed {SEED0}…{} (mọi cỡ lấy n cặp đầu của cùng thế giới mô phỏng của seed đó)",
        SEED0 + count as u64 - 1
    );

    let (gens, gen_s) = labs::timed(|| generate_instances(&engine, &base, &sizes, count));
    println!("sinh {} instance và ma trận: {gen_s:.1} s", gens.len());

    if args.flag("--write-instances") {
        let path = root.join("fixtures/loc50/instances.json");
        if fixture {
            die("--write-instances chỉ cho đồ thị HCM");
        }
        std::fs::create_dir_all(path.parent().expect("có thư mục cha")).ok();
        write_instances(&path, "hcm", &gens);
        println!(
            "đã ghi {} ({} instance, {} byte)",
            path.strip_prefix(&root).unwrap_or(&path).display(),
            gens.len(),
            std::fs::metadata(&path).map_or(0, |m| m.len())
        );
        println!(
            "bước tiếp: cd examples/toi-noi-location/python/loc50 && uv run python loc50_ortools.py"
        );
        println!("\nuptime lúc kết thúc: {}", uptime());
        return;
    }

    // OR-Tools (nếu có)
    let ort_path = args.value("--ortools").map_or_else(
        || root.join("fixtures/loc50/ortools-results.json"),
        std::path::PathBuf::from,
    );
    let ort = if args.flag("--no-python") || fixture {
        println!(
            "\nOR-Tools: bỏ qua ({}), chỉ so Rust với DP.",
            if fixture { "--fixture" } else { "--no-python" }
        );
        None
    } else {
        match read_ortools(&ort_path) {
            Ok(o) => {
                println!(
                    "\nOR-Tools: đọc {} ({} kết quả), ortools {}; {}",
                    ort_path.strip_prefix(&root).unwrap_or(&ort_path).display(),
                    o.by_id.len(),
                    o.version,
                    o.meta
                );
                Some(o)
            }
            Err(e) => {
                println!(
                    "\nOR-Tools: CHƯA CÓ kết quả ({}: {e}). Chạy bước 1 và 2 ở đầu file này. Bảng dưới chỉ có Rust và DP.",
                    ort_path.display()
                );
                None
            }
        }
    };

    println!("\n-- 1. chất lượng và thời gian theo cỡ n --");
    println!(
        "  gap = (chi phí Rust − chi phí mốc) / chi phí mốc, ghép cặp theo instance; CI 95% theo phân phối t trên các instance cùng cỡ. Gap âm = Rust tốt hơn mốc.\n  «chèn» = chỉ chèn rẻ nhất; «+LS» = chèn rồi 2-opt/Or-opt/dời cặp tới cực tiểu cục bộ."
    );
    let reps = if fixture { 3 } else { 5 };
    let mut mismatches = 0usize;
    let mut stale = 0usize;
    for &n in &sizes {
        let set: Vec<&Gen> = gens.iter().filter(|g| g.n == n).collect();
        let (mut gi_o, mut gl_o) = (Vec::new(), Vec::new());
        let (mut gi_d, mut gl_d, mut go_d) = (Vec::new(), Vec::new(), Vec::new());
        let (mut t_ins, mut t_ls, mut t_dp, mut t_ort) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let (mut better, mut equal, mut worse) = (0, 0, 0);
        let mut abs_ls = Vec::new();
        let mut moves = Vec::new();
        for g in &set {
            let inst = &g.inst;
            let ins = cheapest_insertion(inst);
            let ins_cost = inst.route_cost(&ins);
            let sol = solve(inst, None);
            assert!(inst.is_feasible(&sol.route));
            t_ins.push(time_us(reps, || cheapest_insertion(inst)));
            t_ls.push(time_us(reps, || solve(inst, None)));
            abs_ls.push(sol.cost as f64);
            moves.push(sol.moves as f64);
            if n <= DP_MAX_PAIRS {
                let t = Instant::now();
                let (_, opt) = exact_dp(inst).expect("n ≤ 9");
                t_dp.push(t.elapsed().as_secs_f64() * 1e6);
                gi_d.push((ins_cost - opt) as f64 / opt as f64);
                gl_d.push((sol.cost - opt) as f64 / opt as f64);
                if let Some(o) = ort.as_ref().and_then(|o| o.by_id.get(&g.id)) {
                    go_d.push((o.cost - opt) as f64 / opt as f64);
                }
            }
            if let Some(o) = ort.as_ref().and_then(|o| o.by_id.get(&g.id)) {
                if o.fnv != matrix_fnv(inst.matrix()) {
                    stale += 1;
                    continue;
                }
                if !inst.is_feasible(&o.route) || inst.route_cost(&o.route) != o.cost {
                    mismatches += 1;
                }
                gi_o.push((ins_cost - o.cost) as f64 / o.cost as f64);
                let gl = (sol.cost - o.cost) as f64 / o.cost as f64;
                gl_o.push(gl);
                match sol.cost.cmp(&o.cost) {
                    std::cmp::Ordering::Less => better += 1,
                    std::cmp::Ordering::Equal => equal += 1,
                    std::cmp::Ordering::Greater => worse += 1,
                }
                t_ort.push(o.wall_s);
            }
        }
        println!(
            "\n  n = {n} cặp ({} đỉnh), {} instance; chi phí trung bình sau +LS {:.1} s, số bước cải thiện trung bình {:.1}",
            2 * n + 1,
            set.len(),
            mean(&abs_ls) / 1000.0,
            mean(&moves)
        );
        let line = |name: &str, v: &[f64]| {
            if v.is_empty() {
                return;
            }
            let (m, lo, hi) = mean_ci(v);
            println!(
                "    {name:<22} gap trung bình {:>8} [{:>8}, {:>8}]  trung vị {:>8}  lớn nhất {:>8}",
                pct(m),
                pct(lo),
                pct(hi),
                pct(median(v)),
                pct(maxf(v))
            );
        };
        line("chèn vs OR-Tools", &gi_o);
        line("+LS vs OR-Tools", &gl_o);
        if !gl_o.is_empty() {
            println!(
                "    +LS so với OR-Tools trên {} instance: Rust tốt hơn {better}, bằng {equal}, kém hơn {worse}",
                gl_o.len()
            );
        }
        line("chèn vs DP (tối ưu)", &gi_d);
        line("+LS vs DP (tối ưu)", &gl_d);
        line("OR-Tools vs DP", &go_d);
        println!(
            "    thời gian (µs, trung vị mỗi instance, trung bình qua instance): chèn {:.0}; chèn+LS {:.0}{}{}",
            mean(&t_ins),
            mean(&t_ls),
            if t_dp.is_empty() {
                String::new()
            } else {
                format!("; DP chính xác {:.0}", mean(&t_dp))
            },
            if t_ort.is_empty() {
                String::new()
            } else {
                format!(
                    "; OR-Tools (tường thời gian) {:.0} (lớn nhất {:.0})",
                    mean(&t_ort) * 1e6,
                    maxf(&t_ort) * 1e6
                )
            }
        );
        if !t_ort.is_empty() {
            println!(
                "    tỉ lệ thời gian OR-Tools / Rust(+LS) ≈ {:.0}×",
                mean(&t_ort) * 1e6 / mean(&t_ls)
            );
        }
        if let Some(o) = &ort {
            let st: BTreeMap<&str, usize> =
                set.iter()
                    .filter_map(|g| o.by_id.get(&g.id))
                    .fold(BTreeMap::new(), |mut m, r| {
                        *m.entry(r.status.as_str()).or_insert(0) += 1;
                        m
                    });
            println!("    trạng thái OR-Tools: {st:?}");
        }
    }
    if ort.is_some() {
        println!(
            "\n  kiểm tra chéo: tuyến OR-Tools hợp lệ (lấy trước giao) và chi phí Python = chi phí Rust tính lại: {} instance sai; kết quả OR-Tools ứng với ma trận khác (băm FNV lệch): {stale}",
            mismatches
        );
    }

    println!("\n-- 2. đường nóng: ngân sách thời gian (cỡ lớn nhất) --");
    let nmax = *sizes.iter().max().expect("có cỡ");
    let set: Vec<&Gen> = gens.iter().filter(|g| g.n == nmax).collect();
    let full: Vec<i64> = set.iter().map(|g| solve(&g.inst, None).cost).collect();
    println!(
        "  n = {nmax}: tuyến hợp lệ ở mọi ngân sách; «hết hạn» = dừng trước cực tiểu cục bộ; chênh = chi phí so với chạy không giới hạn (cùng instance)"
    );
    for us in [0u64, 50, 200, 1000, 5000] {
        let (mut t_out, mut diffs) = (0, Vec::new());
        for (g, &f) in set.iter().zip(&full) {
            let s = solve(&g.inst, Some(Duration::from_micros(us)));
            assert!(g.inst.is_feasible(&s.route));
            t_out += usize::from(s.timed_out);
            diffs.push((s.cost - f) as f64 / f as f64);
        }
        println!(
            "    ngân sách {us:>5} µs: hết hạn {t_out}/{}, chi phí chênh trung bình {} (lớn nhất {})",
            set.len(),
            pct(mean(&diffs)),
            pct(maxf(&diffs))
        );
    }

    println!("\n-- 3. một instance cụ thể (n = {}) --", sizes[0]);
    if let Some(g) = gens.iter().find(|g| g.n == sizes[0]) {
        let ins = cheapest_insertion(&g.inst);
        let sol = solve(&g.inst, None);
        let fmt = |r: &[usize]| {
            r.iter()
                .map(|&v| match v {
                    0 => "S".to_string(),
                    v if Instance::is_pickup(v) => {
                        format!("L{}", Instance::pair_of(v).unwrap_or(0))
                    }
                    v => format!("G{}", Instance::pair_of(v).unwrap_or(0)),
                })
                .collect::<Vec<_>>()
                .join(" ")
        };
        println!(
            "  {} (seed {}): L = lấy, G = giao, S = xuất phát",
            g.id, g.seed
        );
        println!("  chèn:  {}  → {} ms", fmt(&ins), g.inst.route_cost(&ins));
        println!("  +LS:   {}  → {} ms", fmt(&sol.route), sol.cost);
        if let Some((r, c)) = exact_dp(&g.inst) {
            println!("  DP:    {}  → {} ms", fmt(&r), c);
        }
        if let Some(o) = ort.as_ref().and_then(|o| o.by_id.get(&g.id)) {
            println!("  OR-T:  {}  → {} ms", fmt(&o.route), o.cost);
        }
    }

    println!(
        "\nGiới hạn: instance mô phỏng (điểm lấy/giao rải khắp bản đồ, không phải bưu kiện cùng một quận); một người giao hàng, không tải trọng, không khung giờ; chi phí là thời gian chạy từ CH (không có chi phí rẽ). Thời gian đo khi máy chịu tải nặng (xem uptime); OR-Tools bị chặn bởi giới hạn thời gian nên chất lượng của nó phụ thuộc giới hạn đó và tải máy."
    );
    println!("\nuptime lúc kết thúc: {}", uptime());
}
