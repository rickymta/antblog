//! LOC-21: Dijkstra và A* trên đồ thị CSR. Số nút đã duyệt, heuristic hợp lệ, so với `Vec<Vec<..>>` và `petgraph`.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc21_dijkstra_astar
//! Tham số: --queries N (mặc định 300), --seed S, --fixture

use labs::p03::{DIST_BUCKETS, bucket_of, crow_m, mb, mean, random_pairs, uptime, vn};
use labs::{Args, load_graph};
use petgraph::algo::{astar, dijkstra};
use petgraph::graph::{DiGraph, NodeIndex};
use road_graph::Mode;
use route_engine::alt::largest_scc_nodes;
use route_engine::net::{INF, Rev, View};
use route_engine::profile::{Costs, Metric, Profile};
use route_engine::search::{NodeXy, Searcher};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::time::Instant;

const SEED: u64 = 20_261_106;

/// Cách làm «đối tượng con trỏ»: mỗi nút giữ một `Vec` cạnh riêng trên heap.
struct VecGraph {
    out: Vec<Vec<(u32, u32)>>,
}

impl VecGraph {
    fn build(view: View) -> Self {
        let mut out = vec![Vec::new(); view.node_count()];
        for u in 0..view.node_count() as u32 {
            for (_, v, w) in view.out(u) {
                out[u as usize].push((v, w));
            }
        }
        Self { out }
    }
    /// Số byte thật: tiêu đề `Vec` mỗi nút + vùng cấp phát theo `capacity`.
    fn bytes(&self) -> usize {
        self.out.len() * size_of::<Vec<(u32, u32)>>()
            + self
                .out
                .iter()
                .map(|v| v.capacity() * size_of::<(u32, u32)>())
                .sum::<usize>()
    }
    fn dijkstra(&self, s: u32, t: u32) -> Option<u32> {
        let mut dist = vec![INF; self.out.len()];
        let mut heap = BinaryHeap::new();
        dist[s as usize] = 0;
        heap.push(Reverse((0u32, s)));
        while let Some(Reverse((d, u))) = heap.pop() {
            if u == t {
                return Some(d);
            }
            if d > dist[u as usize] {
                continue;
            }
            for &(v, w) in &self.out[u as usize] {
                let nd = d + w;
                if nd < dist[v as usize] {
                    dist[v as usize] = nd;
                    heap.push(Reverse((nd, v)));
                }
            }
        }
        None
    }
}

fn main() {
    let args = Args::from_env();
    let seed = args.u64_or("--seed", SEED);
    let nq = args.u64_or("--queries", 300) as usize;
    let (g, source) = load_graph(&args);
    println!("== LOC-21: Dijkstra và A* trên CSR ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    println!(
        "đồ thị: {source}: {} nút, {} cạnh có hướng; seed {seed}; {nq} truy vấn mỗi cấu hình\n",
        vn(g.node_count() as u64),
        vn(g.edge_count() as u64)
    );

    let rev = Rev::build(&g);
    let xy = NodeXy::build(&g);
    let car_t = Costs::build(&g, Profile::Car, Metric::Time);
    let car_d = Costs::build(&g, Profile::Car, Metric::Distance);
    println!("-- 1. CSR so với Vec<Vec<..>> (ô tô, trọng số thời gian) --");
    let view = View::new(&g, &rev, &car_t.w);
    let csr_bytes = g.first_out.len() * 4 + g.head.len() * 4 + car_t.w.len() * 4;
    let vg = VecGraph::build(view);
    println!(
        "  CSR (first_out + head + trọng số, cả cạnh ô tô không đi được): {:.2} MB; Vec<Vec<(đích, trọng số)>> chỉ giữ cạnh đi được: {:.2} MB",
        mb(csr_bytes),
        mb(vg.bytes())
    );
    println!(
        "  cạnh ô tô đi được: {} trên {} cạnh của đồ thị chung",
        vn(car_t.allowed_edges as u64),
        vn(g.edge_count() as u64)
    );

    // mẫu truy vấn: các nút thuộc thành phần liên thông mạnh lớn nhất của ô tô, nên mọi cặp đều có đường
    let scc = largest_scc_nodes(&g, &rev, &car_t.w);
    let nodes: Vec<u32> = (0..g.node_count() as u32)
        .filter(|&v| scc[v as usize])
        .collect();
    println!(
        "  thành phần liên thông mạnh lớn nhất của ô tô: {} nút ({:.1}% đồ thị); truy vấn lấy mẫu trong đó",
        vn(nodes.len() as u64),
        100.0 * nodes.len() as f64 / g.node_count() as f64
    );
    let pairs = random_pairs(&nodes, nq, seed);

    println!("\n-- 2. heuristic chim bay có hợp lệ không (kiểm trên mọi cạnh ô tô) --");
    for (name, c) in [("khoảng cách (cm)", &car_d), ("thời gian (ms)", &car_t)] {
        let scale = NodeXy::scale(c.vmax_cm_per_unit);
        let (mut worst, mut worst_e, mut viol) = (0.0f64, 0usize, 0usize);
        for e in 0..g.edge_count() {
            if c.w[e] == INF {
                continue;
            }
            let h = xy.dist_m(g.tail[e], g.head[e]) * scale;
            let r = h / f64::from(c.w[e]);
            if r > worst {
                (worst, worst_e) = (r, e);
            }
            if h > f64::from(c.w[e]) {
                viol += 1;
            }
        }
        println!(
            "  trọng số {name}: vmax = {:.4} cm/đơn vị; max h/w trên một cạnh = {worst:.4} (cạnh {worst_e}); số cạnh vi phạm h > w = {viol}",
            c.vmax_cm_per_unit
        );
    }
    println!(
        "  (h = 0,99 × chim bay × 100 / vmax; vmax lấy từ chính dữ liệu nên h luôn chặn dưới; 0,99 là biên an toàn cho mặt phẳng cục bộ)"
    );

    println!("\n-- 3. Dijkstra so với A* theo khoảng cách chim bay (ô tô) --");
    for (mname, costs) in [("thời gian", &car_t), ("khoảng cách", &car_d)] {
        let view = View::new(&g, &rev, &costs.w);
        let mut s = Searcher::new(g.node_count());
        let mut rows: Vec<Vec<(f64, f64, f64, f64)>> = vec![Vec::new(); 4];
        let mut mismatches = 0;
        for &(a, b) in &pairs {
            let t0 = Instant::now();
            let (d, ds) = s.dijkstra(view, a, b);
            let td = t0.elapsed().as_secs_f64() * 1e6;
            let t0 = Instant::now();
            let (x, xs) = s.astar(view, &xy, costs.vmax_cm_per_unit, a, b);
            let ta = t0.elapsed().as_secs_f64() * 1e6;
            if d.as_ref().map(|p| p.cost) != x.as_ref().map(|p| p.cost) {
                mismatches += 1;
            }
            rows[bucket_of(crow_m(&g, a, b))].push((
                f64::from(ds.settled),
                f64::from(xs.settled),
                td,
                ta,
            ));
        }
        println!(
            "  trọng số {mname}; chi phí khác nhau giữa Dijkstra và A*: {mismatches}/{}",
            pairs.len()
        );
        println!(
            "  {:<10} {:>5} {:>14} {:>14} {:>8} {:>13} {:>13}",
            "nhóm", "n", "Dijkstra nút", "A* nút", "tỉ lệ", "Dijkstra µs", "A* µs"
        );
        for (i, r) in rows.iter().enumerate() {
            if r.is_empty() {
                continue;
            }
            let col = |k: usize| {
                mean(
                    &r.iter()
                        .map(|x| [x.0, x.1, x.2, x.3][k])
                        .collect::<Vec<_>>(),
                )
            };
            println!(
                "  {:<10} {:>5} {:>14.0} {:>14.0} {:>7.2}x {:>13.0} {:>13.0}",
                DIST_BUCKETS[i].0,
                r.len(),
                col(0),
                col(1),
                col(0) / col(1),
                col(2),
                col(3)
            );
        }
    }

    println!("\n-- 4. cùng truy vấn: CSR so với Vec<Vec<..>> so với petgraph (ô tô, thời gian) --");
    let mut pg: DiGraph<(), u32> = DiGraph::with_capacity(g.node_count(), car_t.allowed_edges);
    let idx: Vec<NodeIndex> = (0..g.node_count()).map(|_| pg.add_node(())).collect();
    for u in 0..g.node_count() as u32 {
        for (_, v, w) in view.out(u) {
            pg.add_edge(idx[u as usize], idx[v as usize], w);
        }
    }
    let scale = NodeXy::scale(car_t.vmax_cm_per_unit);
    let mut s = Searcher::new(g.node_count());
    let (mut t_csr, mut t_vec, mut t_pgd, mut t_pga, mut t_csr_a) =
        (vec![], vec![], vec![], vec![], vec![]);
    let (mut bad_vec, mut bad_pgd, mut bad_pga) = (0, 0, 0);
    let mut pg_labels = Vec::new();
    for &(a, b) in &pairs {
        let t0 = Instant::now();
        let (d, _) = s.dijkstra(view, a, b);
        t_csr.push(t0.elapsed().as_secs_f64() * 1e6);
        let want = d.map(|p| p.cost);
        let t0 = Instant::now();
        let (x, _) = s.astar(view, &xy, car_t.vmax_cm_per_unit, a, b);
        t_csr_a.push(t0.elapsed().as_secs_f64() * 1e6);
        assert_eq!(x.map(|p| p.cost), want);

        let t0 = Instant::now();
        let r = vg.dijkstra(a, b);
        t_vec.push(t0.elapsed().as_secs_f64() * 1e6);
        bad_vec += usize::from(r != want);

        let t0 = Instant::now();
        let m = dijkstra(&pg, idx[a as usize], Some(idx[b as usize]), |e| *e.weight());
        t_pgd.push(t0.elapsed().as_secs_f64() * 1e6);
        pg_labels.push(m.len() as f64);
        bad_pgd += usize::from(m.get(&idx[b as usize]).copied() != want);

        let tp = xy.xy[b as usize];
        let t0 = Instant::now();
        let r = astar(
            &pg,
            idx[a as usize],
            |n| n == idx[b as usize],
            |e| *e.weight(),
            |n| (xy.dist_to_point_m(n.index() as u32, tp) * scale) as u32,
        );
        t_pga.push(t0.elapsed().as_secs_f64() * 1e6);
        bad_pga += usize::from(r.map(|(c, _)| c) != want);
    }
    let med = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    println!(
        "  chi phí khác Dijkstra CSR: Vec<Vec>: {bad_vec}, petgraph dijkstra: {bad_pgd}, petgraph astar: {bad_pga} (trên {} truy vấn)",
        pairs.len()
    );
    println!(
        "  {:<28} {:>12} {:>12}",
        "cách làm", "trung bình µs", "trung vị µs"
    );
    for (name, v) in [
        ("CSR Dijkstra (lab)", &mut t_csr),
        ("CSR A* (lab)", &mut t_csr_a),
        ("Vec<Vec> Dijkstra", &mut t_vec),
        ("petgraph::dijkstra", &mut t_pgd),
        ("petgraph::astar", &mut t_pga),
    ] {
        println!("  {name:<28} {:>12.0} {:>12.0}", mean(v), med(v));
    }
    println!(
        "  petgraph::dijkstra trả bản đồ nhãn trung bình {:.0} nút/truy vấn (tính cả nút chạm nhưng chưa chốt, không so trực tiếp với «nút đã duyệt» ở trên)",
        mean(&pg_labels)
    );

    println!("\n-- 5. một tuyến cụ thể --");
    let (a, b) = pairs[0];
    let (r, st) = s.astar(view, &xy, car_t.vmax_cm_per_unit, a, b);
    let p = r.expect("cặp trong SCC");
    println!(
        "  nút {a} → nút {b}: chim bay {:.0} m; thời gian {:.1} s; độ dài {:.0} m; {} cạnh; A* duyệt {} nút, xét {} cạnh",
        crow_m(&g, a, b),
        f64::from(p.cost) / 1000.0,
        p.length_cm(&g) as f64 / 100.0,
        p.edges.len(),
        vn(u64::from(st.settled)),
        vn(u64::from(st.relaxed))
    );
    println!(
        "  số cạnh ô tô / xe máy / đi bộ của đồ thị: {} / {} / {}",
        vn(g.mode_edge_count(Mode::Car) as u64),
        vn(g.mode_edge_count(Mode::Motorbike) as u64),
        vn(g.mode_edge_count(Mode::Foot) as u64)
    );
    println!("\nuptime lúc kết thúc: {}", uptime());
}
