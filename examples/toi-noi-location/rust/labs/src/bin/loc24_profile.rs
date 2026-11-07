//! LOC-24: ba profile (ô tô, xe máy, đi bộ) trên cùng một đồ thị. Tốc độ theo loại đường, hẻm, và đo tần suất tuyến ô tô khác tuyến xe máy.
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc24_profile
//! Tham số: --queries N (mặc định 300), --seed S, --fixture

use labs::p03::{DIST_BUCKETS, bucket_of, crow_m, mb, mean, random_pairs, uptime, vn};
use labs::{Args, load_graph};
use road_graph::{HighwayClass, Mode, flag};
use route_engine::alt::largest_scc_nodes;
use route_engine::net::{INF, Rev, View};
use route_engine::profile::{Costs, Metric, Profile, SpeedTable};
use route_engine::search::{NodeXy, Searcher};

const SEED: u64 = 20_261_106;

fn pct(v: &mut [f64], p: f64) -> f64 {
    v.sort_by(f64::total_cmp);
    v[((p / 100.0) * v.len() as f64).ceil().max(1.0) as usize - 1]
}

fn main() {
    let args = Args::from_env();
    let seed = args.u64_or("--seed", SEED);
    let nq = args.u64_or("--queries", 300) as usize;
    let (g, source) = load_graph(&args);
    println!("== LOC-24: profile ô tô, xe máy, đi bộ ==");
    println!("uptime lúc bắt đầu: {}", uptime());
    println!(
        "đồ thị: {source}: {} nút, {} cạnh; seed {seed}; {nq} cặp nút\n",
        vn(g.node_count() as u64),
        vn(g.edge_count() as u64)
    );
    let rev = Rev::build(&g);
    let xy = NodeXy::build(&g);
    let costs: Vec<Costs> = Profile::ALL
        .iter()
        .map(|&p| Costs::build(&g, p, Metric::Time))
        .collect();
    let (car, bike, foot) = (&costs[0], &costs[1], &costs[2]);

    println!("-- 1. cấu trúc dữ liệu dùng chung --");
    let topo =
        (g.first_out.len() + g.head.len() + g.tail.len() + g.length_cm.len()) * 4 + rev.bytes();
    let separate: usize = costs
        .iter()
        .map(|c| (g.node_count() + 1) * 4 + c.allowed_edges * (4 + 4 + 4))
        .sum();
    println!(
        "  hình dạng đồ thị dùng chung (CSR thuận + ngược + độ dài): {:.2} MB, giữ một bản; mỗi profile chỉ thêm mảng trọng số {:.2} MB",
        mb(topo),
        mb(car.bytes())
    );
    println!(
        "  tổng ba profile dùng chung: {:.2} MB; nếu dựng ba đồ thị riêng (chỉ giữ cạnh đi được, CSR + độ dài + trọng số): {:.2} MB",
        mb(topo + 3 * car.bytes()),
        mb(separate)
    );
    for c in &costs {
        println!(
            "  {:<10} cạnh đi được {:>9}  ({:.1}% cạnh)",
            c.profile.name(),
            vn(c.allowed_edges as u64),
            100.0 * c.allowed_edges as f64 / g.edge_count() as f64
        );
    }

    println!("\n-- 2. tốc độ theo loại đường (km/h, GIẢ ĐỊNH của lab) và số cạnh --");
    println!(
        "  {:<16} {:>9} {:>7} {:>7} {:>7}   cạnh: ô tô / xe máy / đi bộ",
        "highway", "cạnh", "ô tô", "xe máy", "đi bộ"
    );
    let tables: Vec<SpeedTable> = Profile::ALL
        .iter()
        .map(|&p| SpeedTable::for_profile(p))
        .collect();
    for c in HighwayClass::ALL {
        let on: Vec<usize> = Profile::ALL
            .iter()
            .map(|p| {
                (0..g.edge_count())
                    .filter(|&e| g.class[e] == c as u8 && g.modes[e] & p.mode().bit() != 0)
                    .count()
            })
            .collect();
        let total = (0..g.edge_count())
            .filter(|&e| g.class[e] == c as u8)
            .count();
        if total == 0 {
            continue;
        }
        let sp = |i: usize, on: usize| {
            if on == 0 {
                "-".to_string()
            } else {
                format!("{:.0}", tables[i].kmh[c as usize])
            }
        };
        println!(
            "  {:<16} {:>9} {:>7} {:>7} {:>7}   {} / {} / {}",
            c.osm_value(),
            vn(total as u64),
            sp(0, on[0]),
            sp(1, on[1]),
            sp(2, on[2]),
            vn(on[0] as u64),
            vn(on[1] as u64),
            vn(on[2] as u64)
        );
    }
    println!(
        "  (tốc độ còn bị chặn bởi thẻ maxspeed nếu có; hẻm gắn service=alley: ô tô tối đa 10, xe máy 18; width < 4 m: ô tô tối đa 10)"
    );

    println!("\n-- 3. hẻm và những chỗ ba profile khác nhau trên dữ liệu thật --");
    let alley = (0..g.edge_count())
        .filter(|&e| g.flags[e] & flag::ALLEY != 0)
        .count();
    let service = (0..g.edge_count())
        .filter(|&e| g.flags[e] & flag::SERVICE != 0)
        .count();
    let with_width = (0..g.edge_count()).filter(|&e| g.width_dm[e] != 0).count();
    let narrow = (0..g.edge_count())
        .filter(|&e| g.width_dm[e] != 0 && g.width_dm[e] < 30)
        .count();
    println!(
        "  cạnh highway=service: {}; trong đó service=alley: {}; có thẻ width: {} ({:.2}% cạnh); width < 3,0 m (ô tô bị chặn): {}",
        vn(service as u64),
        vn(alley as u64),
        vn(with_width as u64),
        100.0 * with_width as f64 / g.edge_count() as f64,
        vn(narrow as u64)
    );
    let (mut moto_only, mut by_narrow, mut by_oneway, mut by_other) =
        (0usize, 0usize, 0usize, 0usize);
    let mut by_class = std::collections::BTreeMap::<u8, usize>::new();
    let (mut car_only, mut foot_only) = (0usize, 0usize);
    let mut car_only_class = std::collections::BTreeMap::<u8, usize>::new();
    for e in 0..g.edge_count() {
        let m = g.modes[e];
        let (c, b, f) = (
            m & Mode::CAR_BIT != 0,
            m & Mode::MOTORBIKE_BIT != 0,
            m & Mode::FOOT_BIT != 0,
        );
        if b && !c {
            moto_only += 1;
            *by_class.entry(g.class[e]).or_default() += 1;
            let tw = g.twin[e];
            if g.width_dm[e] != 0 && g.width_dm[e] < 30 {
                by_narrow += 1;
            } else if tw != u32::MAX && g.modes[tw as usize] & Mode::CAR_BIT != 0 {
                by_oneway += 1;
            } else {
                by_other += 1;
            }
        }
        if c && !b {
            car_only += 1;
            *car_only_class.entry(g.class[e]).or_default() += 1;
        }
        if f && !c && !b {
            foot_only += 1;
        }
    }
    println!(
        "  cạnh xe máy đi được mà ô tô không: {} ({:.1}%): do width < 3 m {}, do một chiều chỉ với ô tô (chiều ngược còn cho ô tô: cạnh twin đi được) {}, lý do khác (thẻ access, motor_vehicle, đường dành cho xe máy...) {}",
        vn(moto_only as u64),
        100.0 * moto_only as f64 / g.edge_count() as f64,
        vn(by_narrow as u64),
        vn(by_oneway as u64),
        vn(by_other as u64)
    );
    let top = |m: &std::collections::BTreeMap<u8, usize>| {
        let mut v: Vec<_> = m.iter().collect();
        v.sort_by_key(|&(_, n)| std::cmp::Reverse(*n));
        v.iter()
            .take(4)
            .map(|&(c, n)| {
                format!(
                    "{} {}",
                    HighwayClass::from_u8(*c).map_or("?", |c| c.osm_value()),
                    vn(*n as u64)
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    println!("  loại đường của cạnh chỉ xe máy: {}", top(&by_class));
    println!(
        "  cạnh ô tô đi được mà xe máy không: {} (loại đường: {}); cạnh chỉ đi bộ: {}",
        vn(car_only as u64),
        top(&car_only_class),
        vn(foot_only as u64)
    );

    // mẫu truy vấn: nút thuộc SCC lớn nhất của cả ô tô lẫn xe máy
    let (vc, vb, vf) = (
        View::new(&g, &rev, &car.w),
        View::new(&g, &rev, &bike.w),
        View::new(&g, &rev, &foot.w),
    );
    let (sc, sb) = (
        largest_scc_nodes(&g, &rev, &car.w),
        largest_scc_nodes(&g, &rev, &bike.w),
    );
    let nodes: Vec<u32> = (0..g.node_count() as u32)
        .filter(|&v| sc[v as usize] && sb[v as usize])
        .collect();
    let pairs = random_pairs(&nodes, nq, seed);
    println!(
        "\n-- 4. tuyến ô tô so với tuyến xe máy: {} cặp ngẫu nhiên trong giao của hai SCC lớn nhất ({} nút) --",
        pairs.len(),
        vn(nodes.len() as u64)
    );
    // xe máy chỉ trên mạng ô tô đi được: cùng tốc độ xe máy, chỉ khác quyền đi
    let bike_on_car_net: Vec<u32> = (0..g.edge_count())
        .map(|e| {
            if g.modes[e] & Mode::CAR_BIT != 0 {
                bike.w[e]
            } else {
                INF
            }
        })
        .collect();
    let vbc = View::new(&g, &rev, &bike_on_car_net);
    let mut s = Searcher::new(g.node_count());
    struct Row {
        crow: f64,
        t_car: f64,
        t_bike: f64,
        d_car: f64,
        d_bike: f64,
        t_foot: f64,
        d_foot: f64,
        differ: bool,
        differ_access: bool,
        saving_access: f64,
        bike_uses_nocar: usize,
    }
    let mut rows = Vec::new();
    let mut best_example: Option<(f64, u32, u32, Vec<u32>)> = None;
    let mut best_alley: Option<(f64, u32, u32, Vec<u32>)> = None;
    let small = |e: u32| {
        matches!(
            HighwayClass::from_u8(g.class[e as usize]),
            Some(HighwayClass::Service | HighwayClass::Residential | HighwayClass::LivingStreet)
        )
    };
    for &(a, b) in &pairs {
        let (Some(pc), Some(pb), Some(pbc), Some(pf)) = (
            s.astar(vc, &xy, car.vmax_cm_per_unit, a, b).0,
            s.astar(vb, &xy, bike.vmax_cm_per_unit, a, b).0,
            s.astar(vbc, &xy, bike.vmax_cm_per_unit, a, b).0,
            s.astar(vf, &xy, foot.vmax_cm_per_unit, a, b).0,
        ) else {
            continue;
        };
        let nocar: Vec<u32> = pb
            .edges
            .iter()
            .copied()
            .filter(|&e| g.modes[e as usize] & Mode::CAR_BIT == 0)
            .collect();
        let saving = 1.0 - f64::from(pb.cost) / f64::from(pbc.cost);
        if !nocar.is_empty()
            && crow_m(&g, a, b) > 500.0
            && best_example.as_ref().is_none_or(|x| saving > x.0)
        {
            best_example = Some((saving, a, b, nocar.clone()));
        }
        // ca «hẻm»: mọi cạnh xe máy dùng mà ô tô không vào được đều là đường nhỏ (service, residential, living_street)
        if !nocar.is_empty()
            && nocar.iter().all(|&e| small(e))
            && crow_m(&g, a, b) > 500.0
            && best_alley.as_ref().is_none_or(|x| saving > x.0)
        {
            best_alley = Some((saving, a, b, nocar.clone()));
        }
        rows.push(Row {
            crow: crow_m(&g, a, b),
            t_car: f64::from(pc.cost) / 1000.0,
            t_bike: f64::from(pb.cost) / 1000.0,
            d_car: pc.length_cm(&g) as f64 / 100.0,
            d_bike: pb.length_cm(&g) as f64 / 100.0,
            t_foot: f64::from(pf.cost) / 1000.0,
            d_foot: pf.length_cm(&g) as f64 / 100.0,
            differ: pc.edges != pb.edges,
            differ_access: pb.edges != pbc.edges,
            saving_access: saving,
            bike_uses_nocar: nocar.len(),
        });
    }
    let n = rows.len() as f64;
    let frac = |f: &dyn Fn(&Row) -> bool| 100.0 * rows.iter().filter(|r| f(r)).count() as f64 / n;
    println!(
        "  tuyến ô tô khác tuyến xe máy (khác dãy cạnh): {:.1}% (gồm cả khác do tốc độ và do quyền đi)",
        frac(&|r| r.differ)
    );
    println!(
        "  khác chỉ vì quyền đi (xe máy trên toàn mạng so với xe máy chỉ trên cạnh ô tô đi được, cùng tốc độ): {:.1}%; tuyến xe máy dùng ít nhất một cạnh ô tô không vào được: {:.1}%",
        frac(&|r| r.differ_access),
        frac(&|r| r.bike_uses_nocar > 0)
    );
    let mut sv: Vec<f64> = rows
        .iter()
        .filter(|r| r.differ_access)
        .map(|r| r.saving_access * 100.0)
        .collect();
    if !sv.is_empty() {
        println!(
            "  trong các ca khác vì quyền đi ({}): xe máy nhanh hơn {:.1}% trung bình, trung vị {:.1}%, p95 {:.1}%",
            sv.len(),
            mean(&sv),
            pct(&mut sv, 50.0),
            pct(&mut sv, 95.0)
        );
    }
    let ratio = |f: &dyn Fn(&Row) -> f64| rows.iter().map(f).collect::<Vec<_>>();
    let (mut rt, mut rd) = (
        ratio(&|r| r.t_bike / r.t_car),
        ratio(&|r| r.d_bike / r.d_car),
    );
    println!(
        "  xe máy / ô tô: thời gian trung bình {:.3} (trung vị {:.3}, p5 {:.3}, p95 {:.3}); độ dài trung bình {:.3} (trung vị {:.3})",
        mean(&rt),
        pct(&mut rt, 50.0),
        pct(&mut rt, 5.0),
        pct(&mut rt, 95.0),
        mean(&rd),
        pct(&mut rd, 50.0)
    );
    println!("  theo nhóm khoảng cách chim bay:");
    println!(
        "    {:<10} {:>5} {:>10} {:>12} {:>12} {:>13}",
        "nhóm", "n", "khác tuyến", "t xe máy/ô tô", "t đi bộ/ô tô", "d đi bộ/ô tô"
    );
    for (i, (name, _)) in DIST_BUCKETS.iter().enumerate() {
        let sel: Vec<&Row> = rows.iter().filter(|r| bucket_of(r.crow) == i).collect();
        if sel.is_empty() {
            continue;
        }
        let m = |f: &dyn Fn(&&Row) -> f64| mean(&sel.iter().map(f).collect::<Vec<_>>());
        println!(
            "    {:<10} {:>5} {:>9.1}% {:>12.3} {:>12.2} {:>13.3}",
            name,
            sel.len(),
            100.0 * sel.iter().filter(|r| r.differ).count() as f64 / sel.len() as f64,
            m(&|r| r.t_bike / r.t_car),
            m(&|r| r.t_foot / r.t_car),
            m(&|r| r.d_foot / r.d_car)
        );
    }
    let foot_shorter = frac(&|r| r.d_foot + 1.0 < r.d_car);
    let (mut rf,) = (ratio(&|r| r.d_foot / r.d_car),);
    println!(
        "  đi bộ so với ô tô: đường đi bộ ngắn hơn đường ô tô trong {:.1}% cặp (đi ngược đường một chiều, qua cầu bộ hành, hẻm); độ dài đi bộ/ô tô trung bình {:.3}, trung vị {:.3}, nhỏ nhất {:.3}",
        foot_shorter,
        mean(&rf),
        pct(&mut rf, 50.0),
        pct(&mut rf, 0.0)
    );

    println!("\n-- 5. hai ca thật trong dữ liệu: xe máy đi được chỗ ô tô không đi được --");
    let mut show = |title: &str, ex: Option<(f64, u32, u32, Vec<u32>)>| {
        println!("  {title}");
        let Some((saving, a, b, nocar)) = ex else {
            println!("    không có cặp nào trong mẫu thoả điều kiện");
            return;
        };
        let pb = s
            .astar(vb, &xy, bike.vmax_cm_per_unit, a, b)
            .0
            .expect("đã có");
        let pbc = s
            .astar(vbc, &xy, bike.vmax_cm_per_unit, a, b)
            .0
            .expect("đã có");
        let pc = s
            .astar(vc, &xy, car.vmax_cm_per_unit, a, b)
            .0
            .expect("đã có");
        println!(
            "    nút {a} → nút {b}, chim bay {:.0} m: ô tô {:.1} s / {:.0} m; xe máy {:.1} s / {:.0} m; xe máy chỉ trên cạnh ô tô vào được {:.1} s / {:.0} m (xe máy nhanh hơn {:.1}%)",
            crow_m(&g, a, b),
            f64::from(pc.cost) / 1000.0,
            pc.length_cm(&g) as f64 / 100.0,
            f64::from(pb.cost) / 1000.0,
            pb.length_cm(&g) as f64 / 100.0,
            f64::from(pbc.cost) / 1000.0,
            pbc.length_cm(&g) as f64 / 100.0,
            saving * 100.0
        );
        println!(
            "    tuyến xe máy đi qua {} cạnh ô tô không vào được:",
            nocar.len()
        );
        for &e in nocar.iter().take(5) {
            let v = g.edge(e);
            println!(
                "      cạnh {e}: way {} «{}», loại {}, dài {} m, width {}, service=alley {}, chiều ngược cho ô tô {}",
                v.way_id(),
                v.name().unwrap_or("(không tên)"),
                v.class().osm_value(),
                v.length_m(),
                v.width_m()
                    .map_or("không có thẻ".to_string(), |w| format!("{w:.1} m")),
                v.has(flag::ALLEY),
                v.twin()
                    .is_some_and(|t| g.modes[t as usize] & Mode::CAR_BIT != 0)
            );
        }
    };
    show(
        "(a) ca hẻm: mọi cạnh chỉ xe máy trên tuyến là service/residential/living_street, mức tiết kiệm lớn nhất trong mẫu",
        best_alley,
    );
    show(
        "(b) ca tiết kiệm lớn nhất trong mẫu, loại đường bất kỳ",
        best_example,
    );
    println!(
        "\n-- 6. tìm kiếm có giới hạn trên đồ thị đi bộ: khách đi bộ tới đâu trong N phút (nền cho LOC-33) --"
    );
    {
        let mut rng = labs::Rand::new(seed ^ 0x9);
        let starts: Vec<u32> = (0..60).map(|_| nodes[rng.usize(nodes.len())]).collect();
        println!(
            "  {:>8} {:>14} {:>14} {:>14} {:>16} {:>12}",
            "ngân sách",
            "nút TB",
            "nút p95",
            "chim bay xa nhất TB (m)",
            "xe máy cùng thời gian: nút TB",
            "µs mỗi lần"
        );
        for minutes in [2u32, 5, 10] {
            let budget = minutes * 60_000;
            let (mut cnt, mut far, mut bike_cnt, mut us) =
                (Vec::new(), Vec::new(), Vec::new(), Vec::new());
            for &st in &starts {
                let t = std::time::Instant::now();
                let reach = s.within(vf, st, budget, true);
                us.push(t.elapsed().as_secs_f64() * 1e6);
                cnt.push(reach.len() as f64);
                far.push(
                    reach
                        .iter()
                        .map(|&(v, _)| crow_m(&g, st, v))
                        .fold(0.0, f64::max),
                );
                bike_cnt.push(s.within(vb, st, budget, true).len() as f64);
            }
            println!(
                "  {:>5} ph {:>14.0} {:>14.0} {:>14.0} {:>16.0} {:>12.0}",
                minutes,
                mean(&cnt),
                pct(&mut cnt, 95.0),
                mean(&far),
                mean(&bike_cnt),
                mean(&us)
            );
        }
        println!(
            "  (đi bộ 5 km/h; mỗi lần tìm là Dijkstra dừng khi vượt ngân sách, không quét cả đồ thị; {} điểm xuất phát ngẫu nhiên có seed)",
            starts.len()
        );
    }
    println!("\nuptime lúc kết thúc: {}", uptime());
}
