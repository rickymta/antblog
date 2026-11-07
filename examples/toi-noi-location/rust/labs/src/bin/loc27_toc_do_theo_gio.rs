//! LOC-27: tốc độ theo giờ (TỔNG HỢP, không phải dữ liệu giao thông thật) và Customizable CH: đổi trọng số không dựng lại.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc27_toc_do_theo_gio
//! Tham số: --queries N (mặc định 500), --seed S, --fixture

use geo_core::LatLng;
use labs::p03::{DIST_BUCKETS, bucket_of, crow_m, mean, random_pairs, uptime, vn};
use labs::{Args, load_graph, timed};
use route_engine::alt::largest_scc_nodes;
use route_engine::cch::Cch;
use route_engine::ch::{Ch, ChWorkspace};
use route_engine::net::{INF, Rev, View};
use route_engine::profile::{Costs, Metric, Profile};
use route_engine::search::{NodeXy, Searcher};
use route_engine::traffic::{CONGESTION_BY_HOUR, SLOTS, TrafficModel};
use std::time::Instant;

const SEED: u64 = 20_261_106;

fn pct(v: &mut [f64], p: f64) -> f64 {
    v.sort_by(f64::total_cmp);
    v[((p / 100.0) * v.len() as f64).ceil().max(1.0) as usize - 1]
}

fn main() {
    let args = Args::from_env();
    let seed = args.u64_or("--seed", SEED);
    let nq = args.u64_or("--queries", 500) as usize;
    let fixture = args.flag("--fixture");
    let (g, source) = load_graph(&args);
    println!("== LOC-27: tốc độ theo giờ và Customizable CH ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    println!(
        "đồ thị: {source}: {} nút, {} cạnh; ô tô; seed {seed}; {nq} cặp nút",
        vn(g.node_count() as u64),
        vn(g.edge_count() as u64)
    );
    println!(
        "DỮ LIỆU KẸT XE Ở ĐÂY LÀ TỔNG HỢP: hệ số do lab tự đặt (xem crates/route-engine/src/traffic.rs), không đo từ xe thật.\n"
    );
    let rev = Rev::build(&g);
    let xy = NodeXy::build(&g);
    // vùng trung tâm: bán kính 5 km quanh Bến Thành trên HCM; 500 m quanh tâm khu mẫu
    let (center, radius) = if fixture {
        (g.center(), 500.0)
    } else {
        (LatLng::try_new(10.7725, 106.6980).expect("hợp lệ"), 5_000.0)
    };
    let model = TrafficModel::new(&g, center, radius);
    let free = Costs::build(&g, Profile::Car, Metric::Time);
    let view = View::new(&g, &rev, &free.w);

    println!("-- 1. hồ sơ tốc độ tổng hợp --");
    println!(
        "  vùng trung tâm: bán kính {radius:.0} m quanh ({:.4}, {:.4}), {} cạnh trên {}",
        center.lat(),
        center.lng(),
        vn(model.central_edges() as u64),
        vn(g.edge_count() as u64)
    );
    println!(
        "  {:>4} {:>13} {:>26} {:>26}",
        "giờ", "chỉ số kẹt", "tốc độ TB ô tô (km/h)", "so với giờ thông thoáng"
    );
    let slot_costs: Vec<Costs> = (0..SLOTS)
        .map(|h| model.costs(&g, Profile::Car, h))
        .collect();
    let len_total: f64 = (0..g.edge_count())
        .filter(|&e| free.w[e] != INF)
        .map(|e| f64::from(g.length_cm[e]))
        .sum();
    let mean_speed = |c: &Costs| {
        // tốc độ trung bình theo độ dài = tổng độ dài / tổng thời gian
        let t: f64 = (0..g.edge_count())
            .filter(|&e| c.w[e] != INF)
            .map(|e| f64::from(c.w[e]))
            .sum();
        len_total / 100.0 / (t / 1000.0) * 3.6
    };
    let base_speed = mean_speed(&free);
    for (h, c) in slot_costs.iter().enumerate() {
        let sp = mean_speed(c);
        println!(
            "  {h:>4} {:>13.2} {sp:>26.2} {:>25.0}%",
            CONGESTION_BY_HOUR[h],
            100.0 * sp / base_speed
        );
    }

    println!("\n-- 2. Customizable CH: thứ tự một lần, tuỳ biến mỗi lần đổi trọng số --");
    let (cch, secs_cch) = timed(|| Cch::build(view, &xy));
    let cs = &cch.stats;
    println!(
        "  dựng CCH (thứ tự bằng chia đệ quy theo toạ độ + cấu trúc không nhân chứng): {secs_cch:.2} s (thứ tự {:.2} s, cấu trúc {:.2} s)",
        cs.order_s, cs.symbolic_s
    );
    println!(
        "  cung đi lên {} ({:.2}/nút), bậc đi lên lớn nhất {}, số tam giác phải duyệt khi tuỳ biến {}, bộ nhớ cấu trúc {:.1} MB",
        vn(cs.arcs as u64),
        cs.arcs as f64 / g.node_count() as f64,
        cs.max_up_degree,
        vn(cs.triangles),
        cch.bytes() as f64 / 1048576.0
    );
    // tuỳ biến cho cả 24 giờ
    let mut t_cust = Vec::new();
    let mut chs: Vec<Ch> = Vec::new();
    for c in &slot_costs {
        let (ch, s) = timed(|| cch.customize(&c.w));
        t_cust.push(s * 1e3);
        chs.push(ch);
    }
    // dựng lại CH thường từ đầu cho từng giờ, để so cùng điều kiện
    let mut t_rebuild = Vec::new();
    let mut ch_slots: Vec<Ch> = Vec::new();
    for c in &slot_costs {
        let v = View::new(&g, &rev, &c.w);
        let (ch, s) = timed(|| Ch::build(v));
        t_rebuild.push(s * 1e3);
        ch_slots.push(ch);
    }
    let ch0 = &ch_slots[3];
    println!(
        "  tuỳ biến lại trọng số cho 24 giờ: mỗi lần trung bình {:.0} ms (nhỏ nhất {:.0}, lớn nhất {:.0}); tổng {:.2} s",
        mean(&t_cust),
        t_cust.iter().copied().fold(f64::MAX, f64::min),
        t_cust.iter().copied().fold(0.0, f64::max),
        t_cust.iter().sum::<f64>() / 1000.0
    );
    println!(
        "  dựng lại CH thường từ đầu cho cùng 24 giờ: mỗi lần trung bình {:.0} ms (nhỏ nhất {:.0}, lớn nhất {:.0}); tổng {:.2} s",
        mean(&t_rebuild),
        t_rebuild.iter().copied().fold(f64::MAX, f64::min),
        t_rebuild.iter().copied().fold(0.0, f64::max),
        t_rebuild.iter().sum::<f64>() / 1000.0
    );
    println!(
        "  dựng lại / tuỳ biến: trung bình {:.2}x, nhỏ nhất {:.2}x (cùng máy, cùng lần chạy, luân phiên theo giờ)",
        mean(&t_rebuild) / mean(&t_cust),
        t_rebuild
            .iter()
            .zip(&t_cust)
            .map(|(r, c)| r / c)
            .fold(f64::MAX, f64::min)
    );
    println!(
        "  cung đi lên sau tuỳ biến (giờ 8, chỉ giữ cung hữu hạn): thuận {}, ngược {}, tổng {} ({:.2}/nút) trên {} cung của cấu trúc ×2; CH thường giờ 8: {} ({:.2}/nút)",
        vn(chs[8].fwd.len() as u64),
        vn(chs[8].bwd.len() as u64),
        vn(chs[8].arc_count() as u64),
        chs[8].arc_count() as f64 / g.node_count() as f64,
        vn(2 * cs.arcs as u64),
        vn(ch_slots[8].arc_count() as u64),
        ch_slots[8].arc_count() as f64 / g.node_count() as f64
    );
    println!(
        "  shortcut: CCH {} (giờ 8), CH thường {} (giờ 8); CH thường dựng ban đầu có độ sâu level {}",
        vn(chs[8].stats.shortcuts as u64),
        vn(ch_slots[8].stats.shortcuts as u64),
        ch_slots[8].stats.max_level
    );

    println!("\n-- 3. tính đúng sau tuỳ biến, và tốc độ truy vấn --");
    let scc = largest_scc_nodes(&g, &rev, &free.w);
    let nodes: Vec<u32> = (0..g.node_count() as u32)
        .filter(|&v| scc[v as usize])
        .collect();
    let pairs = random_pairs(&nodes, nq, seed);
    let mut ws = ChWorkspace::new(g.node_count());
    let mut uni = Searcher::new(g.node_count());
    for h in [3usize, 8, 13, 18] {
        let v = View::new(&g, &rev, &slot_costs[h].w);
        let mut bad = 0;
        let n_check = pairs.len().min(300);
        for &(a, b) in pairs.iter().take(n_check) {
            let truth = uni.dijkstra(v, a, b).0.map(|p| p.cost);
            bad += usize::from(chs[h].distance(&mut ws, a, b) != truth);
        }
        println!(
            "  giờ {h:>2}: CCH tuỳ biến so với Dijkstra: {bad} khác biệt trên {n_check} truy vấn"
        );
    }
    let (mut t_ch, mut t_cch, mut s_ch, mut s_cch) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for &(a, b) in &pairs {
        std::hint::black_box(ch0.distance(&mut ws, a, b));
        let t = Instant::now();
        std::hint::black_box(ch0.distance(&mut ws, a, b));
        t_ch.push(t.elapsed().as_secs_f64() * 1e6);
        s_ch.push(f64::from(ws.stats.settled));
        std::hint::black_box(chs[3].distance(&mut ws, a, b));
        let t = Instant::now();
        std::hint::black_box(chs[3].distance(&mut ws, a, b));
        t_cch.push(t.elapsed().as_secs_f64() * 1e6);
        s_cch.push(f64::from(ws.stats.settled));
    }
    println!(
        "  truy vấn khoảng cách, trung bình: CH thường {:.1} µs ({:.0} nút chốt), CCH {:.1} µs ({:.0} nút chốt); CCH chậm hơn {:.2}x",
        mean(&t_ch),
        mean(&s_ch),
        mean(&t_cch),
        mean(&s_cch),
        mean(&t_cch) / mean(&t_ch)
    );

    println!(
        "\n-- 4. sai lệch khi dùng tốc độ tĩnh (tốc độ tự do của giờ thông thoáng) trong giờ khác --"
    );
    println!(
        "  «tĩnh» = tuyến tối ưu theo tốc độ giờ 3 sáng, rồi chấm lại bằng trọng số của giờ đang xét"
    );
    println!(
        "  {:>4} {:>11} {:>12} {:>12} {:>12} {:>14} {:>14} {:>10}",
        "giờ",
        "ETA tĩnh thấp hơn (TB)",
        "p95",
        "lớn nhất",
        "tuyến tĩnh chậm hơn tối ưu (TB)",
        "p95",
        "tuyến khác",
        "n"
    );
    let free_ch = &chs[3];
    for h in [8usize, 13, 18, 22] {
        let (mut under, mut subopt) = (Vec::new(), Vec::new());
        let mut differ = 0;
        for &(a, b) in &pairs {
            let (Some(ps), Some(po)) = (free_ch.path(&mut ws, a, b), chs[h].path(&mut ws, a, b))
            else {
                continue;
            };
            let est = f64::from(ps.cost);
            let actual = ps.cost_under(&slot_costs[h].w) as f64;
            under.push(100.0 * (1.0 - est / actual));
            subopt.push(100.0 * (actual / f64::from(po.cost) - 1.0));
            differ += usize::from(ps.edges != po.edges);
        }
        println!(
            "  {h:>4} {:>20.1}% {:>11.1}% {:>11.1}% {:>26.2}% {:>13.2}% {:>13.1}% {:>10}",
            mean(&under),
            pct(&mut under, 95.0),
            pct(&mut under, 100.0),
            mean(&subopt),
            pct(&mut subopt, 95.0),
            100.0 * differ as f64 / under.len() as f64,
            under.len()
        );
    }
    println!("  theo nhóm khoảng cách chim bay, giờ 8 (cao điểm sáng):");
    println!(
        "    {:<10} {:>5} {:>22} {:>26}",
        "nhóm", "n", "ETA tĩnh thấp hơn (TB)", "tuyến tĩnh chậm hơn tối ưu"
    );
    for (i, (name, _)) in DIST_BUCKETS.iter().enumerate() {
        let (mut under, mut subopt) = (Vec::new(), Vec::new());
        for &(a, b) in pairs
            .iter()
            .filter(|&&(a, b)| bucket_of(crow_m(&g, a, b)) == i)
        {
            let (Some(ps), Some(po)) = (free_ch.path(&mut ws, a, b), chs[8].path(&mut ws, a, b))
            else {
                continue;
            };
            let actual = ps.cost_under(&slot_costs[8].w) as f64;
            under.push(100.0 * (1.0 - f64::from(ps.cost) / actual));
            subopt.push(100.0 * (actual / f64::from(po.cost) - 1.0));
        }
        if !under.is_empty() {
            println!(
                "    {:<10} {:>5} {:>21.1}% {:>25.2}%",
                name,
                under.len(),
                mean(&under),
                mean(&subopt)
            );
        }
    }

    println!("\n-- 5. một chuyến cố định qua các giờ (nút → nút xa nhất trong mẫu) --");
    let &(a, b) = pairs
        .iter()
        .max_by(|x, y| crow_m(&g, x.0, x.1).total_cmp(&crow_m(&g, y.0, y.1)))
        .expect("có mẫu");
    println!(
        "  nút {a} → nút {b}, chim bay {:.0} m; thời gian (phút) theo giờ khởi hành:",
        crow_m(&g, a, b)
    );
    let mut line = String::new();
    for (h, ch) in chs.iter().enumerate() {
        let c = ch
            .distance(&mut ws, a, b)
            .map_or(f64::NAN, |c| f64::from(c) / 60_000.0);
        line.push_str(&format!("{h:02}h:{c:.1} "));
        if h % 8 == 7 {
            println!("    {line}");
            line.clear();
        }
    }
    println!("\nuptime lúc kết thúc: {}", uptime());
}
