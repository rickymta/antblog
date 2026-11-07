//! LOC-22: Dijkstra hai chiều (điều kiện dừng đúng và lỗi kinh điển) và ALT landmark.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc22_hai_chieu_alt
//! Tham số: --queries N (mặc định 300), --seed S, --fixture

use labs::p03::{DIST_BUCKETS, bucket_of, crow_m, mb, mean, random_pairs, uptime, vn};
use labs::{Args, load_graph, timed};
use route_engine::alt::{Landmarks, largest_scc_nodes};
use route_engine::bidir::{BiSearcher, Stop, classic_counterexample};
use route_engine::net::{Rev, View};
use route_engine::profile::{Costs, Metric, Profile};
use route_engine::search::{NodeXy, Searcher};
use std::time::Instant;

const SEED: u64 = 20_261_106;

fn main() {
    let args = Args::from_env();
    let seed = args.u64_or("--seed", SEED);
    let nq = args.u64_or("--queries", 300) as usize;
    let (g, source) = load_graph(&args);
    println!("== LOC-22: hai chiều và ALT ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    println!(
        "đồ thị: {source}: {} nút, {} cạnh; ô tô, trọng số thời gian; seed {seed}; {nq} truy vấn\n",
        vn(g.node_count() as u64),
        vn(g.edge_count() as u64)
    );

    println!("-- 1. lỗi dừng sớm: ví dụ nhỏ dựng tay (s=0, a=1, m=2, b=3, t=4) --");
    {
        let (hg, hw) = classic_counterexample();
        let hrev = Rev::build(&hg);
        let hv = View::new(&hg, &hrev, &hw);
        let mut bi = BiSearcher::new(hg.node_count());
        let (good, _) = bi.run(hv, 0, 4, Stop::Correct);
        let (bad, _) = bi.run(hv, 0, 4, Stop::FirstMeeting);
        let (truth, _) = Searcher::new(hg.node_count()).dijkstra(hv, 0, 4);
        println!("  cạnh: s→a 1, a→b 10, b→t 1, s→m 6, m→t 7");
        println!(
            "  Dijkstra một chiều        : chi phí {}",
            truth.unwrap().cost
        );
        println!(
            "  hai chiều, dừng đúng      : chi phí {}",
            good.unwrap().cost
        );
        let bad = bad.unwrap();
        println!(
            "  hai chiều, dừng ở nút gặp đầu tiên: chi phí {} (qua nút m, đi {:?})",
            bad.cost,
            bad.nodes(&hg)
        );
    }

    let rev = Rev::build(&g);
    let xy = NodeXy::build(&g);
    let costs = Costs::build(&g, Profile::Car, Metric::Time);
    let view = View::new(&g, &rev, &costs.w);
    let scc = largest_scc_nodes(&g, &rev, &costs.w);
    let nodes: Vec<u32> = (0..g.node_count() as u32)
        .filter(|&v| scc[v as usize])
        .collect();
    let pairs = random_pairs(&nodes, nq, seed);

    println!(
        "\n-- 2. cùng lỗi đó trên bản đồ thật: {} truy vấn ngẫu nhiên --",
        pairs.len()
    );
    let mut uni = Searcher::new(g.node_count());
    let mut bi = BiSearcher::new(g.node_count());
    let (mut wrong, mut worst) = (0usize, (0.0f64, 0u32, 0u32, 0u32, 0u32));
    let mut excess = Vec::new();
    let mut correct_bad = 0;
    for &(a, b) in &pairs {
        let truth = uni.dijkstra(view, a, b).0.map(|p| p.cost);
        let good = bi.run(view, a, b, Stop::Correct).0.map(|p| p.cost);
        let bad = bi.run(view, a, b, Stop::FirstMeeting).0.map(|p| p.cost);
        correct_bad += usize::from(good != truth);
        if bad != truth {
            wrong += 1;
            let (t, w) = (truth.unwrap(), bad.unwrap());
            let r = f64::from(w) / f64::from(t) - 1.0;
            excess.push(r * 100.0);
            if r > worst.0 {
                worst = (r, a, b, t, w);
            }
        }
    }
    println!("  dừng đúng khác Dijkstra: {correct_bad}/{}", pairs.len());
    println!(
        "  dừng ở nút gặp đầu tiên cho chi phí sai: {wrong}/{} truy vấn ({:.1}%)",
        pairs.len(),
        100.0 * wrong as f64 / pairs.len() as f64
    );
    if wrong > 0 {
        excess.sort_by(f64::total_cmp);
        println!(
            "  đường sai dài hơn đường đúng: trung bình {:.2}%, trung vị {:.2}%, lớn nhất {:.2}%",
            mean(&excess),
            excess[excess.len() / 2],
            excess.last().copied().unwrap_or(0.0)
        );
        let (_, a, b, t, w) = worst;
        println!(
            "  ca tệ nhất: nút {a} → nút {b}, chim bay {:.0} m: đúng {:.1} s, sai {:.1} s",
            crow_m(&g, a, b),
            f64::from(t) / 1000.0,
            f64::from(w) / 1000.0
        );
    }

    println!("\n-- 3. tiền xử lý ALT --");
    let seed_node = nodes[0];
    let mut alts = Vec::new();
    for k in [4usize, 8, 16] {
        let (lm, s) = timed(|| Landmarks::build(view, k, &scc, seed_node));
        println!(
            "  k = {k:<2}: tiền xử lý {s:.2} s (2×{k} lần Dijkstra song song), bộ nhớ {:.1} MB ({:.0} byte/nút), đồ thị gốc {:.1} MB",
            mb(lm.bytes()),
            lm.bytes() as f64 / g.node_count() as f64,
            mb(g.first_out.len() * 4 + g.head.len() * 4 + costs.w.len() * 4)
        );
        alts.push((k, lm));
    }

    println!("\n-- 4. số nút đã duyệt và thời gian trung bình theo nhóm khoảng cách --");
    println!("  tất cả cho chi phí đúng bằng Dijkstra; cột cuối là số truy vấn khác Dijkstra");
    let names = [
        "Dijkstra",
        "A* chim bay",
        "hai chiều",
        "ALT k=4",
        "ALT k=8",
        "ALT k=16",
    ];
    // [thuật toán][nhóm] = (nút, µs)
    let mut acc: Vec<Vec<Vec<(f64, f64)>>> = vec![vec![Vec::new(); 4]; names.len()];
    let mut mismatch = vec![0usize; names.len()];
    for &(a, b) in &pairs {
        let bk = bucket_of(crow_m(&g, a, b));
        let t0 = Instant::now();
        let (d, ds) = uni.dijkstra(view, a, b);
        let want = d.map(|p| p.cost);
        acc[0][bk].push((f64::from(ds.settled), t0.elapsed().as_secs_f64() * 1e6));
        let t0 = Instant::now();
        let (x, xs) = uni.astar(view, &xy, costs.vmax_cm_per_unit, a, b);
        acc[1][bk].push((f64::from(xs.settled), t0.elapsed().as_secs_f64() * 1e6));
        mismatch[1] += usize::from(x.map(|p| p.cost) != want);
        let t0 = Instant::now();
        let (x, xs) = bi.run(view, a, b, Stop::Correct);
        acc[2][bk].push((f64::from(xs.settled), t0.elapsed().as_secs_f64() * 1e6));
        mismatch[2] += usize::from(x.map(|p| p.cost) != want);
        for (i, (_, lm)) in alts.iter().enumerate() {
            let t0 = Instant::now();
            let (x, xs) = uni.run(view, a, b, |u| lm.lower_bound(u, b));
            acc[3 + i][bk].push((f64::from(xs.settled), t0.elapsed().as_secs_f64() * 1e6));
            mismatch[3 + i] += usize::from(x.map(|p| p.cost) != want);
        }
    }
    print!("  {:<12}", "nhóm");
    for n in &names {
        print!(" {n:>12}");
    }
    println!();
    for (bk, (bname, _)) in DIST_BUCKETS.iter().enumerate() {
        if acc[0][bk].is_empty() {
            continue;
        }
        print!("  {:<12}", format!("{bname} nút"));
        for a in &acc {
            print!(
                " {:>12.0}",
                mean(&a[bk].iter().map(|x| x.0).collect::<Vec<_>>())
            );
        }
        println!("   (n = {})", acc[0][bk].len());
        print!("  {:<12}", "   µs");
        for a in &acc {
            print!(
                " {:>12.0}",
                mean(&a[bk].iter().map(|x| x.1).collect::<Vec<_>>())
            );
        }
        println!();
    }
    print!("  {:<12}", "khác Dijkstra");
    for m in &mismatch {
        print!(" {m:>12}");
    }
    println!();
    let all = |i: usize| mean(&acc[i].iter().flatten().map(|x| x.0).collect::<Vec<_>>());
    println!(
        "  tỉ lệ nút duyệt so với Dijkstra (mọi nhóm): A* {:.2}x, hai chiều {:.2}x, ALT k=4 {:.2}x, ALT k=8 {:.2}x, ALT k=16 {:.2}x ít hơn",
        all(0) / all(1),
        all(0) / all(2),
        all(0) / all(3),
        all(0) / all(4),
        all(0) / all(5)
    );
    println!("\nuptime lúc kết thúc: {}", uptime());
}
