//! LOC-25: Contraction Hierarchies tự cài đặt, đối chiếu `fast_paths`, Dijkstra và A*.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc25_contraction_hierarchies
//! Tham số: --queries N (mặc định 2000 cho CH, 300 cho Dijkstra), --seed S, --fixture, --save-ch PATH (ghi CH để dùng lại)

use fast_paths::InputGraph;
use labs::p03::{mb, mean, random_pairs, uptime, vn};
use labs::{Args, load_graph, timed};
use route_engine::alt::largest_scc_nodes;
use route_engine::ch::{Ch, ChWorkspace};
use route_engine::net::{INF, Rev, View};
use route_engine::profile::{Costs, Metric, Profile};
use route_engine::search::{NodeXy, Searcher};
use std::time::Instant;

const SEED: u64 = 20_261_106;

fn main() {
    let args = Args::from_env();
    let seed = args.u64_or("--seed", SEED);
    let nq = args.u64_or("--queries", 2000) as usize;
    let (g, source) = load_graph(&args);
    println!("== LOC-25: Contraction Hierarchies ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    println!(
        "đồ thị: {source}: {} nút, {} cạnh; ô tô, trọng số thời gian (ms); seed {seed}\n",
        vn(g.node_count() as u64),
        vn(g.edge_count() as u64)
    );
    let rev = Rev::build(&g);
    let xy = NodeXy::build(&g);
    let costs = Costs::build(&g, Profile::Car, Metric::Time);
    let view = View::new(&g, &rev, &costs.w);

    println!("-- 1. tiền xử lý: CH của lab --");
    let (ch, secs) = timed(|| Ch::build(view));
    let st = &ch.stats;
    println!("  thời gian dựng: {secs:.2} s (một luồng)");
    println!(
        "  cạnh gốc đi được (đã gộp song song): {}; shortcut thêm: {} ({:.2}x số cạnh gốc); độ sâu phân cấp lớn nhất (level): {}",
        vn(st.original_arcs as u64),
        vn(st.shortcuts as u64),
        st.shortcuts as f64 / st.original_arcs as f64,
        st.max_level
    );
    println!(
        "  cung trong hai đồ thị đi lên: thuận {}, ngược {}, tổng {} ({:.2} cung/nút); bộ nhớ {:.2} MB",
        vn(ch.fwd.len() as u64),
        vn(ch.bwd.len() as u64),
        vn(ch.arc_count() as u64),
        ch.arc_count() as f64 / g.node_count() as f64,
        mb(ch.bytes())
    );
    if let Some(p) = args.value("--save-ch") {
        ch.save(std::path::Path::new(p)).expect("ghi CH");
        println!("  đã ghi CH ra {p}");
    }

    println!("\n-- 2. tiền xử lý: fast_paths trên cùng đầu vào --");
    let mut input = InputGraph::new();
    for e in 0..g.edge_count() {
        if costs.w[e] != INF && g.tail[e] != g.head[e] {
            input.add_edge(g.tail[e] as usize, g.head[e] as usize, costs.w[e] as usize);
        }
    }
    input.freeze();
    let (fg, fsecs) = timed(|| fast_paths::prepare(&input));
    // theo mã nguồn fast_paths 1.0.0: ranks + hai mảng first_edge_ids (3 usize mỗi nút), mỗi cung 5 usize = 40 byte
    let fbytes = fg.get_num_nodes() * (size_of::<usize>() * 3)
        + (fg.get_num_in_edges() + fg.get_num_out_edges()) * 40;
    println!(
        "  thời gian dựng: {fsecs:.2} s; nút {}, cung đi lên: vào {}, ra {}, tổng {} ({:.2} cung/nút); bộ nhớ ước tính từ cấu trúc (3 usize/nút + 40 byte/cung) {:.2} MB",
        vn(fg.get_num_nodes() as u64),
        vn(fg.get_num_in_edges() as u64),
        vn(fg.get_num_out_edges() as u64),
        vn((fg.get_num_in_edges() + fg.get_num_out_edges()) as u64),
        (fg.get_num_in_edges() + fg.get_num_out_edges()) as f64 / g.node_count() as f64,
        mb(fbytes)
    );
    let order: Vec<usize> = ch.old_id.iter().map(|&v| v as usize).collect();
    let (fg2, fsecs2) =
        timed(|| fast_paths::prepare_with_order(&input, &order).expect("thứ tự hợp lệ"));
    println!(
        "  fast_paths với đúng THỨ TỰ NÚT của lab (tách ảnh hưởng của thứ tự khỏi tìm kiếm nhân chứng): {fsecs2:.2} s, tổng cung đi lên {} so với {} của lab",
        vn((fg2.get_num_in_edges() + fg2.get_num_out_edges()) as u64),
        vn(ch.arc_count() as u64)
    );
    println!(
        "  so sánh thời gian dựng: lab / fast_paths = {:.2}x (lab dùng độ ưu tiên đơn giản 2·ED + số láng giềng đã co + level và giới hạn nhân chứng 500 nút; số này để biết cỡ, không phải so sánh công bằng về chất lượng cài đặt)",
        secs / fsecs
    );

    println!("\n-- 3. tính đúng --");
    let scc = largest_scc_nodes(&g, &rev, &costs.w);
    let nodes: Vec<u32> = (0..g.node_count() as u32)
        .filter(|&v| scc[v as usize])
        .collect();
    let pairs = random_pairs(&nodes, nq, seed);
    let mut ws = ChWorkspace::new(g.node_count());
    let mut uni = Searcher::new(g.node_count());
    let mut calc = fast_paths::create_calculator(&fg);
    let (mut bad_dij, mut bad_fp, mut bad_path, mut checked_dij) = (0, 0, 0, 0);
    let n_dij = (nq / 4).clamp(1, 600);
    for (i, &(a, b)) in pairs.iter().enumerate() {
        let own = ch.path(&mut ws, a, b);
        let fp = calc.calc_path(&fg, a as usize, b as usize);
        let own_cost = own.as_ref().map(|p| p.cost);
        let fp_cost = fp
            .as_ref()
            .filter(|p| p.is_found())
            .map(|p| p.get_weight() as u32);
        bad_fp += usize::from(own_cost != fp_cost);
        if let Some(p) = &own {
            // đường giải nén phải liền mạch và đúng tổng
            let ok = p.cost_under(&costs.w) == u64::from(p.cost)
                && p.edges
                    .windows(2)
                    .all(|w| g.head[w[0] as usize] == g.tail[w[1] as usize])
                && g.tail[p.edges[0] as usize] == a
                && g.head[*p.edges.last().expect("không rỗng") as usize] == b;
            bad_path += usize::from(!ok);
        }
        if i < n_dij {
            checked_dij += 1;
            let truth = uni.dijkstra(view, a, b).0.map(|p| p.cost);
            bad_dij += usize::from(own_cost != truth);
        }
    }
    println!("  CH của lab so với Dijkstra: {bad_dij} khác biệt trên {checked_dij} truy vấn");
    println!(
        "  CH của lab so với fast_paths: {bad_fp} khác biệt trên {} truy vấn",
        pairs.len()
    );
    println!(
        "  đường giải nén sai (không liền mạch / sai tổng / sai đầu cuối): {bad_path} trên {}",
        pairs.len()
    );
    println!(
        "  (kiểm tra ngẫu nhiên có seed ở đây; bản proptest nằm trong crates/route-engine/tests/proptest_vs_dijkstra.rs)"
    );

    println!(
        "\n-- 4. tốc độ truy vấn ({} cặp ngẫu nhiên trong SCC lớn nhất; Dijkstra/A* chỉ {n_dij} cặp đầu) --",
        pairs.len()
    );
    let mut t_own_d = Vec::new();
    let mut t_own_p = Vec::new();
    let mut settled = Vec::new();
    let mut t_fp = Vec::new();
    let mut t_dij = Vec::new();
    let mut t_ast = Vec::new();
    let mut dij_settled = Vec::new();
    for (i, &(a, b)) in pairs.iter().enumerate() {
        // chạy nóng một lần không tính giờ để mỗi cách làm đều đo với cache đã ấm
        std::hint::black_box(ch.distance(&mut ws, a, b));
        std::hint::black_box(ch.path(&mut ws, a, b));
        std::hint::black_box(calc.calc_path(&fg, a as usize, b as usize));
        let t0 = Instant::now();
        let r = ch.distance(&mut ws, a, b);
        t_own_d.push(t0.elapsed().as_secs_f64() * 1e6);
        settled.push(f64::from(ws.stats.settled));
        std::hint::black_box(r);
        let t0 = Instant::now();
        let r = ch.path(&mut ws, a, b);
        t_own_p.push(t0.elapsed().as_secs_f64() * 1e6);
        std::hint::black_box(r);
        let t0 = Instant::now();
        let r = calc.calc_path(&fg, a as usize, b as usize);
        t_fp.push(t0.elapsed().as_secs_f64() * 1e6);
        std::hint::black_box(r);
        if i < n_dij {
            std::hint::black_box(uni.dijkstra(view, a, b));
            let t0 = Instant::now();
            let (r, st) = uni.dijkstra(view, a, b);
            t_dij.push(t0.elapsed().as_secs_f64() * 1e6);
            dij_settled.push(f64::from(st.settled));
            std::hint::black_box(r);
            std::hint::black_box(uni.astar(view, &xy, costs.vmax_cm_per_unit, a, b));
            let t0 = Instant::now();
            std::hint::black_box(uni.astar(view, &xy, costs.vmax_cm_per_unit, a, b));
            t_ast.push(t0.elapsed().as_secs_f64() * 1e6);
        }
    }
    let med = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let p99 = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[((0.99 * v.len() as f64).ceil() as usize).saturating_sub(1)]
    };
    println!("  (mỗi truy vấn chạy nóng một lần trước khi bấm giờ)");
    println!(
        "  {:<34} {:>12} {:>12} {:>12}",
        "cách làm", "trung bình µs", "trung vị µs", "p99 µs"
    );
    for (name, v) in [
        ("CH lab: chỉ khoảng cách", &mut t_own_d),
        ("CH lab: khoảng cách + giải nén đường", &mut t_own_p),
        ("fast_paths: calc_path (có đường)", &mut t_fp),
        ("Dijkstra CSR", &mut t_dij),
        ("A* chim bay", &mut t_ast),
    ] {
        println!(
            "  {name:<34} {:>12.1} {:>12.1} {:>12.1}",
            mean(v),
            med(v),
            p99(v)
        );
    }
    println!(
        "  nút đã chốt mỗi truy vấn: CH lab trung bình {:.0} (cả hai chiều), Dijkstra trung bình {:.0} (trên {} cặp đầu)",
        mean(&settled),
        mean(&dij_settled),
        dij_settled.len()
    );
    println!(
        "  Dijkstra / CH lab (khoảng cách + đường) trung bình = {:.0}x; fast_paths / CH lab = {:.2}x",
        mean(&t_dij) / mean(&t_own_p),
        mean(&t_fp) / mean(&t_own_p)
    );

    println!("\n-- 5. tiền xử lý đổi lấy truy vấn --");
    let saved = mean(&t_dij) - mean(&t_own_p);
    println!(
        "  mỗi truy vấn CH nhanh hơn Dijkstra {saved:.0} µs; tiền xử lý {secs:.2} s hoà vốn sau {:.0} truy vấn (so với Dijkstra, cùng máy, cùng tải)",
        secs * 1e6 / saved.max(1e-9)
    );
    println!("\nuptime lúc kết thúc: {}", uptime());
}
