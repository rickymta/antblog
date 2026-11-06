//! LOC-58 (1/2): mỗi thành phố một mảnh; nạp đồ thị bằng mmap so với đọc vào bộ nhớ (RSS và thời gian nạp); chuyến vắt qua biên.
//!
//! Chạy từ gốc repo:
//!   cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc58_phan_manh_mmap
//! Tham số: --repeats N (5, số lần mỗi chế độ, xen kẽ), --pairs N (1500, cặp để đo biên), --skip-boundary.
//! Các cờ `--child` là nội bộ: mỗi chế độ nạp chạy trong MỘT TIẾN TRÌNH RIÊNG để RSS không lẫn nhau.

use geo_core::LatLng;
use labs::p03::uptime;
use labs::p07::{local_pairs_near, median, rss_kb};
use labs::{Args, Rand, die, graph_bin_path, lab_root};
use map_shard::flat::{FlatGraph, write_flat};
use map_shard::reorder::{hilbert_order, renumber_nodes};
use map_shard::{induced_subgraph, split_by_lng};
use road_graph::Mode;
use route_engine::engine::{Engine, EngineConfig};
use route_engine::profile::Profile;
use route_engine::provider::StraightLineProvider;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

fn work_dir() -> PathBuf {
    let d = lab_root().join("rust/target/p07");
    std::fs::create_dir_all(&d).expect("tạo thư mục làm việc");
    d
}

/// Chế độ con: nạp đồ thị theo `mode`, chạy các truy vấn trong file `queries`, in một dòng `KEY=value`.
fn child(args: &Args) {
    let mode = args
        .value("--child")
        .unwrap_or_else(|| die("--child cần chế độ"));
    let file = PathBuf::from(args.value("--file").unwrap_or_else(|| die("--file")));
    let qfile = args.value("--queries").map(PathBuf::from);
    let pid = std::process::id();
    let rss0 = rss_kb(pid).unwrap_or(0);
    let t = Instant::now();
    let mut touched = 0usize;
    let (rss_load, load_ms, rss_q, q_ms, checksum);
    let queries: Vec<(u32, u32)> = qfile
        .map(|p| {
            let b = std::fs::read(&p).expect("đọc truy vấn");
            b.chunks_exact(8)
                .map(|c| {
                    (
                        u32::from_le_bytes(c[..4].try_into().unwrap()),
                        u32::from_le_bytes(c[4..].try_into().unwrap()),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    match mode {
        "mmap" | "read" => {
            let fg = if mode == "mmap" {
                FlatGraph::open_mmap(&file)
            } else {
                FlatGraph::open_read(&file)
            }
            .unwrap_or_else(|e| die(&format!("{e}")));
            load_ms = t.elapsed().as_secs_f64() * 1000.0;
            rss_load = rss_kb(pid).unwrap_or(0);
            let tq = Instant::now();
            let mut sum = 0u64;
            for &(s, d) in &queries {
                let (c, st) = fg.shortest_cm(Mode::CAR_BIT, s, d);
                sum = sum.wrapping_add(u64::from(c.unwrap_or(0)));
                touched += st;
            }
            q_ms = tq.elapsed().as_secs_f64() * 1000.0;
            checksum = sum;
            rss_q = rss_kb(pid).unwrap_or(0);
            let _ = fg.file_bytes();
        }
        "owned" => {
            let g = road_graph::io::load(&file).unwrap_or_else(|e| die(&format!("{e}")));
            load_ms = t.elapsed().as_secs_f64() * 1000.0;
            rss_load = rss_kb(pid).unwrap_or(0);
            let tq = Instant::now();
            let mut sum = 0u64;
            for &(s, d) in &queries {
                sum = sum.wrapping_add(u64::from(
                    road_graph::sp::shortest_cm(&g, Mode::Car, s, d).unwrap_or(0),
                ));
            }
            q_ms = tq.elapsed().as_secs_f64() * 1000.0;
            checksum = sum;
            rss_q = rss_kb(pid).unwrap_or(0);
        }
        "engine" => {
            let g = road_graph::io::load(&file).unwrap_or_else(|e| die(&format!("{e}")));
            let eng = Engine::new(
                g,
                EngineConfig {
                    ch_cache_dir: Some(lab_root().join("rust/target/ch-cache")),
                    cache_tag: "hcm".into(),
                    ..EngineConfig::default()
                },
            );
            eng.ch(Profile::Car);
            load_ms = t.elapsed().as_secs_f64() * 1000.0;
            rss_load = rss_kb(pid).unwrap_or(0);
            rss_q = rss_load;
            q_ms = 0.0;
            checksum = eng.g.node_count() as u64;
        }
        _ => die("chế độ con không biết"),
    }
    println!(
        "RESULT mode={mode} rss0_kb={rss0} load_ms={load_ms:.3} rss_after_load_kb={rss_load} query_ms={q_ms:.1} rss_after_queries_kb={rss_q} settled={touched} dist_cm_sum={checksum}"
    );
}

fn field(line: &str, k: &str) -> f64 {
    line.split_whitespace()
        .find_map(|t| t.strip_prefix(&format!("{k}=")))
        .and_then(|v| v.parse().ok())
        .unwrap_or(f64::NAN)
}

fn run_child(exe: &Path, mode: &str, file: &Path, queries: Option<&Path>) -> String {
    let mut c = Command::new(exe);
    c.args(["--child", mode, "--file"]).arg(file);
    if let Some(q) = queries {
        c.arg("--queries").arg(q);
    }
    let o = c
        .output()
        .unwrap_or_else(|e| die(&format!("không chạy được tiến trình con: {e}")));
    String::from_utf8_lossy(&o.stdout)
        .lines()
        .find(|l| l.starts_with("RESULT"))
        .unwrap_or_else(|| {
            die(&format!(
                "tiến trình con không in RESULT: {}",
                String::from_utf8_lossy(&o.stderr)
            ))
        })
        .to_string()
}

fn main() {
    let args = Args::from_env();
    if args.flag("--child") {
        child(&args);
        return;
    }
    println!(
        "== LOC-58: mỗi thành phố một mảnh, mmap và chuyến vắt qua biên (trên dữ liệu mô phỏng) =="
    );
    println!("uptime lúc bắt đầu: {}", uptime());
    let gp = graph_bin_path();
    if !gp.exists() {
        die("thiếu data/hcm-graph.bin (chạy lab LOC-06)");
    }
    let g = road_graph::io::load(&gp).unwrap_or_else(|e| die(&format!("{e}")));
    let dir = work_dir();
    println!(
        "đồ thị: {}: {} nút, {} cạnh",
        gp.display(),
        g.node_count(),
        g.edge_count()
    );

    // ---- 1. mảnh theo thành phố
    let cut = 106.69;
    let (west, east) = split_by_lng(&g, cut);
    println!(
        "\n-- 1. chia HCM thành hai «thành phố» giả theo kinh độ {cut} (bản đồ thật chỉ có một thành phố; đây là cách có đáp án thật để đo biên) --"
    );
    for (n, sg) in [("TÂY", &west), ("ĐÔNG", &east)] {
        println!(
            "  mảnh {n:<5}: {:>7} nút, {:>7} cạnh",
            sg.node_count(),
            sg.edge_count()
        );
    }
    println!(
        "  cạnh mất ở biên cắt: {} ({:.2}% số cạnh gốc)",
        g.edge_count() - west.edge_count() - east.edge_count(),
        100.0 * (g.edge_count() - west.edge_count() - east.edge_count()) as f64
            / g.edge_count() as f64
    );

    // ---- 2. mmap so với đọc so với nạp đầy đủ
    let flat = dir.join("hcm.flat");
    let (perm, hil_flat) = {
        let perm = hilbert_order(&g);
        let (h, _) = renumber_nodes(&g, &perm);
        let p = dir.join("hcm-hilbert.flat");
        write_flat(&h, &p).expect("ghi flat hilbert");
        (perm, p)
    };
    let bytes = write_flat(&g, &flat).expect("ghi flat");
    println!(
        "\n-- 2. nạp: mmap so với đọc vào bộ nhớ so với nạp đầy đủ `Graph` (mỗi lần một tiến trình riêng, xen kẽ các chế độ) --"
    );
    println!(
        "  file phẳng {} ({:.1} MiB); file đồ thị gốc {} ({:.1} MiB)",
        flat.display(),
        bytes as f64 / 1_048_576.0,
        gp.display(),
        std::fs::metadata(&gp).map_or(0.0, |m| m.len() as f64) / 1_048_576.0
    );
    let n_q = 400;
    let exe = std::env::current_exe().expect("exe");
    let reps = args.u64_or("--repeats", 5) as usize;
    let centre = LatLng::try_new(10.7725, 106.6980).expect("hợp lệ");
    let sets: [(&str, Option<(LatLng, f64)>); 2] = [
        ("A. nguồn rải khắp thành phố", None),
        (
            "B. nguồn trong một khu vực hẹp (bán kính 1,5 km quanh Chợ Bến Thành)",
            Some((centre, 1500.0)),
        ),
    ];
    for (si, (set_name, area)) in sets.iter().enumerate() {
        let pairs = local_pairs_near(&g, n_q, 120, 58, *area);
        let write_q = |name: &str, f: &dyn Fn(u32) -> u32| -> PathBuf {
            let p = dir.join(format!("q{si}-{name}.bin"));
            let mut b = Vec::new();
            for &(s, d) in &pairs {
                b.extend_from_slice(&f(s).to_le_bytes());
                b.extend_from_slice(&f(d).to_le_bytes());
            }
            std::fs::write(&p, b).expect("ghi truy vấn");
            p
        };
        let q_orig = write_q("orig", &|x| x);
        let q_hil = write_q("hilbert", &|x| perm[x as usize]);
        println!(
            "\n  bộ truy vấn {set_name}: {n_q} truy vấn Dijkstra «cục bộ» (đi ngẫu nhiên 120 bước từ nút nguồn; tất định, seed 58), cùng một tập trên mọi chế độ"
        );
        let modes: [(&str, &str, &Path, Option<&Path>); 5] = [
            ("owned", "Graph nạp đầy đủ (io::load)", &gp, Some(&q_orig)),
            ("read", "phẳng, đọc vào bộ nhớ", &flat, Some(&q_orig)),
            ("mmap", "phẳng, mmap", &flat, Some(&q_orig)),
            ("mmap", "phẳng Hilbert, mmap", &hil_flat, Some(&q_hil)),
            ("read", "phẳng Hilbert, đọc", &hil_flat, Some(&q_hil)),
        ];
        let mut acc: Vec<Vec<String>> = vec![Vec::new(); modes.len()];
        for _ in 0..reps {
            for (i, (m, _, f, q)) in modes.iter().enumerate() {
                acc[i].push(run_child(&exe, m, f, *q));
            }
        }
        println!(
            "  {reps} lần mỗi chế độ, xen kẽ; bảng ghi TRUNG VỊ; RSS tính bằng `ps -o rss` (MiB ở bảng, KB trong output con)"
        );
        println!(
            "  {:<30} {:>9} {:>13} {:>16} {:>13} {:>14} {:>16}",
            "chế độ",
            "nạp (ms)",
            "RSS sau nạp",
            "RSS sau truy vấn",
            "truy vấn (ms)",
            "nút đã chốt",
            "Σ khoảng cách cm"
        );
        let mut sums = Vec::new();
        for (i, (_, label, _, _)) in modes.iter().enumerate() {
            let col = |k: &str| median(&mut acc[i].iter().map(|l| field(l, k)).collect::<Vec<_>>());
            let ck = field(&acc[i][0], "dist_cm_sum");
            sums.push(ck);
            println!(
                "  {:<30} {:>9.2} {:>9.1} MiB {:>12.1} MiB {:>13.1} {:>14} {:>16}",
                label,
                col("load_ms"),
                col("rss_after_load_kb") / 1024.0,
                col("rss_after_queries_kb") / 1024.0,
                col("query_ms"),
                match field(&acc[i][0], "settled") {
                    v if v > 0.0 => format!("{v}"),
                    _ => "(không đếm)".to_string(),
                },
                ck
            );
        }
        println!(
            "  Σ khoảng cách {} ở cả năm chế độ (đánh số nút khác nhau không đổi đáp án; «nút đã chốt» có thể lệch vài nút do hoà khi so sánh)",
            if sums.iter().all(|&x| (x - sums[0]).abs() < 0.5) {
                "KHỚP"
            } else {
                "KHÁC NHAU, LỖI"
            }
        );
    }
    let e = run_child(&exe, "engine", &gp, None);
    println!(
        "  tham chiếu: nạp Graph + Engine::new + CH ô tô từ cache: {:.0} ms, RSS {:.1} MiB (đây là bộ nhớ mà dịch vụ thật dùng; cột «Graph» chỉ là một phần của nó)",
        field(&e, "load_ms"),
        field(&e, "rss_after_load_kb") / 1024.0
    );

    if args.flag("--skip-boundary") {
        println!("uptime lúc kết thúc: {}", uptime());
        return;
    }
    // ---- 3. chuyến vắt qua biên
    println!("\n-- 3. chuyến vắt qua biên giữa hai mảnh --");
    let cfg = |tag: &str| EngineConfig {
        ch_cache_dir: Some(lab_root().join("rust/target/ch-cache")),
        cache_tag: tag.into(),
        ..EngineConfig::default()
    };
    let t = Instant::now();
    let whole = Engine::new(g.clone(), cfg("hcm"));
    whole.ch(Profile::Car);
    let ew = Engine::new(west, cfg("p07-west"));
    ew.ch(Profile::Car);
    let ee = Engine::new(east, cfg("p07-east"));
    ee.ch(Profile::Car);
    println!(
        "  dựng ba engine (cả bản đồ, mảnh tây, mảnh đông; CH ô tô, lần đầu dựng, các lần sau nạp từ cache): {:.1} s",
        t.elapsed().as_secs_f64()
    );
    let n = args.u64_or("--pairs", 1500) as usize;
    let mut rng = Rand::new(58_58);
    let sl = StraightLineProvider::default();
    let (mut within, mut cross, mut outside) = (0, 0, 0);
    let mut within_worse = Vec::new();
    let mut within_unreach = 0;
    let mut est_err = Vec::new();
    let mut est_over = 0usize;
    let mut cross_dist = Vec::new();
    for _ in 0..n {
        let (a, b) = (
            g.node_latlng(rng.usize(g.node_count()) as u32),
            g.node_latlng(rng.usize(g.node_count()) as u32),
        );
        let side = |p: LatLng| p.lng() < cut;
        let Ok(truth) = whole.route(Profile::Car, a, b, None) else {
            outside += 1;
            continue;
        };
        if side(a) == side(b) {
            within += 1;
            let eng = if side(a) { &ew } else { &ee };
            match eng.route(Profile::Car, a, b, None) {
                Ok(r) => within_worse.push(r.distance_m / truth.distance_m.max(1.0)),
                Err(_) => within_unreach += 1,
            }
        } else {
            cross += 1;
            cross_dist.push(truth.distance_m);
            let (est_m, _) = sl.estimate(Profile::Car, a, b);
            est_err.push(est_m / truth.distance_m.max(1.0));
            if est_m > truth.distance_m {
                est_over += 1;
            }
        }
    }
    println!(
        "  {n} cặp điểm ngẫu nhiên (nút), tất định (seed 5858); cặp không có đường trên cả bản đồ: {outside}"
    );
    println!(
        "  cùng mảnh: {within} ({:.1}%); vắt qua biên: {cross} ({:.1}%)",
        100.0 * within as f64 / (within + cross).max(1) as f64,
        100.0 * cross as f64 / (within + cross).max(1) as f64
    );
    let mut w = within_worse.clone();
    w.sort_by(f64::total_cmp);
    let pct = |v: &[f64], p: f64| {
        if v.is_empty() {
            f64::NAN
        } else {
            v[((p / 100.0 * v.len() as f64).ceil() as usize).clamp(1, v.len()) - 1]
        }
    };
    let longer = w.iter().filter(|&&r| r > 1.0 + 1e-9).count();
    println!(
        "  cùng mảnh, định tuyến chỉ trong mảnh so với cả bản đồ: tuyến dài hơn ở {longer}/{} cặp ({:.1}%); tỉ số quãng p50 {:.3}, p95 {:.3}, max {:.3}; không có đường trong mảnh nhưng có trên cả bản đồ: {within_unreach} cặp",
        w.len(),
        100.0 * longer as f64 / w.len().max(1) as f64,
        pct(&w, 50.0),
        pct(&w, 95.0),
        w.last().copied().unwrap_or(f64::NAN)
    );
    let mut e = est_err.clone();
    e.sort_by(f64::total_cmp);
    println!(
        "  vắt qua biên, lab trả ước lượng chim bay × hệ số vòng: quãng ước lượng / quãng thật p5 {:.2}, p50 {:.2}, p95 {:.2}; ước lượng DÀI hơn thật ở {est_over}/{} cặp",
        pct(&e, 5.0),
        pct(&e, 50.0),
        pct(&e, 95.0),
        e.len()
    );
    let mut cd = cross_dist.clone();
    cd.sort_by(f64::total_cmp);
    println!(
        "  quãng thật của các chuyến vắt qua biên: p50 {:.0} m, p95 {:.0} m",
        pct(&cd, 50.0),
        pct(&cd, 95.0)
    );
    println!(
        "  hành vi của lab: `ShardedMaps::place` trả CrossBoundary, bên gọi dùng ước lượng và gắn ESTIMATED; KHÔNG nối hai đồ thị"
    );
    let _ = induced_subgraph;
    println!("uptime lúc kết thúc: {}", uptime());
}
