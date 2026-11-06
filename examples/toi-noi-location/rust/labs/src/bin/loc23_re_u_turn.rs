//! LOC-23: đồ thị theo cạnh, chi phí rẽ, quay đầu, cấm rẽ OSM, qua dải phân cách.
//! Phần 1 chạy trên fixture (ca dựng sẵn có thể xem được); phần 2 thống kê trên bản đồ chọn (mặc định HCM).
//!
//! Chạy từ gốc repo (cần data/hcm-graph.bin cho phần 2, hoặc thêm --fixture):
//!   cargo run --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc23_re_u_turn
//! Tham số: --queries N (mặc định 300), --seed S, --fixture

use geo_core::LatLng;
use labs::p03::{mean, uptime, vn};
use labs::{Args, Rand, fixture_path, load_graph};
use road_graph::{EdgeId, Graph};
use route_engine::alt::largest_scc_nodes;
use route_engine::engine::{Engine, EngineConfig};
use route_engine::position::Position;
use route_engine::profile::Profile;
use route_engine::turn::{
    EdgeRoute, EdgeSearcher, Goal, Restrictions, Start, TurnCosts, TurnKind, TurnModel, UTurnPolicy,
};
use std::time::Instant;

const SEED: u64 = 20_261_106;

fn kind_vn(k: TurnKind) -> &'static str {
    match k {
        TurnKind::Straight => "thẳng",
        TurnKind::Right => "phải",
        TurnKind::Left => "trái",
        TurnKind::UTurn => "quay đầu",
    }
}

/// Mô tả tuyến theo từng đoạn cùng tên đường, kèm kiểu rẽ vào đoạn.
fn describe(g: &Graph, tm: &TurnModel, r: &EdgeRoute) -> String {
    let mut parts: Vec<(String, f64, Option<TurnKind>)> = Vec::new();
    let mut prev: Option<EdgeId> = None;
    let n_edges = r.edges.len();
    for (i, &e) in r.edges.iter().enumerate() {
        let name = g.edge(e).name().map_or_else(
            || format!("(không tên, way {})", g.way_id[e as usize]),
            str::to_string,
        );
        // cạnh đầu và cạnh cuối chỉ đi một phần
        let from = if i == 0 { f64::from(r.start_frac) } else { 0.0 };
        let to = if i + 1 == n_edges {
            f64::from(r.end_frac)
        } else {
            1.0
        };
        let len = f64::from(g.length_cm[e as usize]) / 100.0 * (to - from).max(0.0);
        let kind = prev.map(|p| tm.classify(p, e));
        let median = tm.geom.median_cross[e as usize];
        let label = if median {
            format!("{name} [qua dải phân cách]")
        } else {
            name
        };
        match parts.last_mut() {
            Some(last) if last.0 == label => last.1 += len,
            _ => parts.push((label, len, kind)),
        }
        prev = Some(e);
    }
    let total = parts.len();
    let mut lines: Vec<String> = parts
        .iter()
        .take(12)
        .map(|(n, l, k)| match k {
            Some(k) if *k != TurnKind::Straight => format!("{n} {l:.0} m (rẽ {})", kind_vn(*k)),
            _ => format!("{n} {l:.0} m"),
        })
        .collect();
    if total > 12 {
        lines.push(format!("… còn {} đoạn nữa", total - 12));
    }
    lines.join("\n      → ")
}

struct Models<'a> {
    node: TurnModel<'a>,
    restr_only: TurnModel<'a>,
    urban: TurnModel<'a>,
}

fn route(
    eng: &Engine,
    es: &mut EdgeSearcher,
    tm: &TurnModel,
    from: Start,
    to: Goal,
) -> Option<EdgeRoute> {
    es.route(
        tm,
        &eng.xy,
        eng.costs(Profile::Car).vmax_cm_per_unit,
        from,
        to,
        true,
    )
    .0
}

fn fixture_scenarios() {
    println!("-- 1. ca dựng sẵn trên fixture «khu mẫu» (ô tô) --");
    let g = road_graph::fixture::load_fixture(&fixture_path()).expect("fixture");
    let eng = Engine::new(g, EngineConfig::default());
    let g = &eng.g;
    let none = Restrictions::default();
    let m = Models {
        node: TurnModel {
            restr: &none,
            costs: TurnCosts::none(),
            ..eng.turn_model(Profile::Car, TurnCosts::none())
        },
        restr_only: eng.turn_model(Profile::Car, TurnCosts::none()),
        urban: eng.turn_model(Profile::Car, TurnCosts::urban()),
    };
    let mut es = EdgeSearcher::new(g.edge_count());
    let view = eng.view(Profile::Car);
    println!(
        "  quy ước chi phí rẽ (giả định): thẳng 0 s, rẽ phải 3 s, rẽ trái 6 s, quay đầu trực tiếp chỉ ở ngõ cụt (20 s), vào lối qua dải phân cách +4 s"
    );
    println!(
        "  cấm rẽ đọc từ OSM cho ô tô: {} quan hệ, áp dụng {}, cặp (cạnh vào, cạnh ra) bị cấm {}",
        eng.restrictions[0].stats.relations,
        eng.restrictions[0].stats.applied,
        eng.restrictions[0].stats.banned_pairs
    );
    println!(
        "  cạnh nhận diện là lối qua dải phân cách: {}",
        eng.geom.median_cross.iter().filter(|&&b| b).count()
    );

    // ca A: quay đầu giữa đường trên đường hai chiều
    println!(
        "\n  ca A. Xe đang chạy trên đường hai chiều, khách đứng ở làn ngược chiều ngay chỗ xe đang đứng"
    );
    let e0 = (0..g.edge_count() as u32)
        .find(|&e| {
            let t = g.twin[e as usize];
            view.w[e as usize] != u32::MAX
                && t != u32::MAX
                && view.w[t as usize] != u32::MAX
                && g.length_cm[e as usize] > 6000
                && view.out(g.head[e as usize]).count() >= 4
                && g.edge(e)
                    .name()
                    .is_some_and(|n| n.starts_with("Đường Ngang"))
        })
        .expect("có cạnh hai chiều");
    let t0 = g.twin[e0 as usize];
    let (from, to) = (Position::new(e0, 0.5), Position::new(t0, 0.5));
    println!(
        "    xe: {} (hướng {:.0}°), khách: làn ngược chiều của cùng đoạn",
        g.edge(e0).name().unwrap_or("?"),
        m.node.geom.bear_out[e0 as usize]
    );
    for (name, tm) in [
        ("mô hình theo nút (quay đầu miễn phí)", &m.node),
        ("theo cạnh, cấm quay đầu giữa đường", &m.urban),
    ] {
        let r = route(&eng, &mut es, tm, Start::Pos(from), Goal::Pos(to)).expect("có đường");
        println!(
            "    {name}: {:.1} s, {:.0} m, {} cạnh, quay đầu {}, rẽ trái/phải/quay {:?}",
            f64::from(r.cost) / 1000.0,
            r.length_cm(g) as f64 / 100.0,
            r.edges.len(),
            tm.u_turns(&r.edges),
            {
                let c = tm.turn_counts(&r.edges);
                [
                    c[TurnKind::Left as usize],
                    c[TurnKind::Right as usize],
                    c[TurnKind::UTurn as usize],
                ]
            }
        );
        println!("      → {}", describe(g, tm, &r));
    }

    // ca B: khách ở làn bên kia đại lộ có dải phân cách
    println!(
        "\n  ca B. Xe chạy chiều đông trên đại lộ có dải phân cách, khách đứng ở làn chiều tây (cách khoảng 300 m về phía trước)"
    );
    let proj = geo_core::LocalProjection::new(LatLng::try_new(10.7700, 106.6900).expect("hợp lệ"));
    let at = |x: f64, y: f64| proj.to_latlng([x, y]);
    let driver = eng
        .snap_point(Profile::Car, at(650.0, 590.0), "xe")
        .expect("snap");
    let cust = eng
        .snap_point(Profile::Car, at(950.0, 628.0), "khách")
        .expect("snap");
    let (fp, tp) = (
        driver.positions(view)[0],
        *cust.positions(view).first().expect("có hướng"),
    );
    println!(
        "    xe: {} ({} hướng được phép); khách: {} ({} hướng được phép)",
        g.edge(fp.edge).name().unwrap_or("?"),
        driver.positions(view).len(),
        g.edge(tp.edge).name().unwrap_or("?"),
        cust.positions(view).len()
    );
    for (name, tm) in [
        ("mô hình theo nút", &m.node),
        ("theo cạnh, có chi phí rẽ", &m.urban),
    ] {
        let r = route(&eng, &mut es, tm, Start::Pos(fp), Goal::Pos(tp)).expect("có đường");
        let c = tm.turn_counts(&r.edges);
        println!(
            "    {name}: {:.1} s (trong đó chi phí rẽ {:.1} s), {:.0} m, rẽ trái {} phải {} quay {}, quay đầu {}",
            f64::from(r.cost) / 1000.0,
            f64::from(r.turn_cost) / 1000.0,
            r.length_cm(g) as f64 / 100.0,
            c[TurnKind::Left as usize],
            c[TurnKind::Right as usize],
            c[TurnKind::UTurn as usize],
            tm.u_turns(&r.edges)
        );
        println!("      → {}", describe(g, tm, &r));
    }
    // cùng tuyến của mô hình theo nút, chấm lại bằng luật đầy đủ
    let r_node = route(&eng, &mut es, &m.node, Start::Pos(fp), Goal::Pos(tp)).expect("có đường");
    println!(
        "    chấm lại tuyến của mô hình theo nút bằng chi phí rẽ: {} s thay vì {:.1} s",
        m.urban
            .route_cost(&r_node)
            .map_or("bị cấm".to_string(), |c| format!(
                "{:.1}",
                c as f64 / 1000.0
            )),
        f64::from(r_node.cost) / 1000.0
    );

    // ca C: cấm rẽ
    println!(
        "\n  ca C. Cấm rẽ OSM: xe đi trên cạnh vào, khách ở trên cạnh ra bị cấm (mô hình theo nút rẽ thẳng vào đó)"
    );
    let mut pairs: Vec<(EdgeId, EdgeId)> = eng.restrictions[0].banned.iter().copied().collect();
    pairs.sort_unstable();
    for (f, t) in pairs {
        let (from, to) = (Position::new(f, 0.4), Position::new(t, 0.5));
        let (Some(a), Some(b)) = (
            route(&eng, &mut es, &m.node, Start::Pos(from), Goal::Pos(to)),
            route(
                &eng,
                &mut es,
                &m.restr_only,
                Start::Pos(from),
                Goal::Pos(to),
            ),
        ) else {
            continue;
        };
        let used = a.edges.windows(2).any(|w| w[0] == f && w[1] == t);
        println!(
            "    cấm {} → {}: theo nút {:.1} s{} ; có cấm rẽ {:.1} s ({:+.1} s, {:+.0} m)",
            g.edge(f).name().unwrap_or("?"),
            g.edge(t).name().unwrap_or("?"),
            f64::from(a.cost) / 1000.0,
            if used {
                " (RẼ VÀO CHỖ BỊ CẤM)"
            } else {
                ""
            },
            f64::from(b.cost) / 1000.0,
            (f64::from(b.cost) - f64::from(a.cost)) / 1000.0,
            (b.length_cm(g) as f64 - a.length_cm(g) as f64) / 100.0
        );
    }
}

fn main() {
    let args = Args::from_env();
    let seed = args.u64_or("--seed", SEED);
    let nq = args.u64_or("--queries", 300) as usize;
    println!("== LOC-23: đồ thị theo cạnh, chi phí rẽ, quay đầu ==");
    println!("uptime lúc bắt đầu: {}\n", uptime());
    fixture_scenarios();

    let (g, source) = load_graph(&args);
    println!("\n-- 2. thống kê trên bản đồ: {source} --");
    let eng = Engine::new(g, EngineConfig::default());
    let g = &eng.g;
    println!(
        "  {} nút, {} cạnh; seed {seed}; {nq} truy vấn (xe ở giữa một cạnh có hướng, khách ở giữa một cạnh khác, ô tô)",
        vn(g.node_count() as u64),
        vn(g.edge_count() as u64)
    );
    for (p, name) in [(Profile::Car, "ô tô"), (Profile::Motorbike, "xe máy")] {
        let s = &eng.restrictions[p.index()].stats;
        println!(
            "  cấm rẽ cho {name}: {} quan hệ → áp dụng {} (no_* {}, only_* {}), cặp bị cấm {}; bỏ qua: điều kiện thời gian {}, không khớp cạnh {}, via-way {}, loại phương tiện khác {}, ngoại lệ {}",
            s.relations,
            s.applied,
            s.no_kinds,
            s.only_kinds,
            s.banned_pairs,
            s.conditional_skipped,
            s.unresolved_skipped,
            s.via_way_skipped,
            s.other_mode_skipped,
            s.excepted_skipped
        );
    }
    println!(
        "  cạnh nhận diện là lối qua dải phân cách (nút đầu và cuối đều nằm trên hai chiều ngược nhau của đường lớn một chiều): {}",
        vn(eng.geom.median_cross.iter().filter(|&&b| b).count() as u64)
    );

    let none = Restrictions::default();
    let node = TurnModel {
        restr: &none,
        costs: TurnCosts::none(),
        ..eng.turn_model(Profile::Car, TurnCosts::none())
    };
    let restr_only = eng.turn_model(Profile::Car, TurnCosts::none());
    let urban = eng.turn_model(Profile::Car, TurnCosts::urban());
    let costed = eng.turn_model(
        Profile::Car,
        TurnCosts {
            u_turn_policy: UTurnPolicy::Costed,
            ..TurnCosts::urban()
        },
    );
    let view = eng.view(Profile::Car);
    let scc = largest_scc_nodes(g, &eng.rev, &eng.costs(Profile::Car).w);
    // vị trí: cạnh ô tô có hướng, cả hai đầu thuộc SCC lớn nhất, dài ≥ 20 m
    let edges: Vec<u32> = (0..g.edge_count() as u32)
        .filter(|&e| {
            view.w[e as usize] != u32::MAX
                && scc[g.tail[e as usize] as usize]
                && scc[g.head[e as usize] as usize]
                && g.length_cm[e as usize] >= 2000
        })
        .collect();
    let mut rng = Rand::new(seed);
    let mut es = EdgeSearcher::new(g.edge_count());
    #[derive(Default)]
    struct Acc {
        delta_restr: Vec<f64>,
        delta_urban: Vec<f64>,
        a_violates: usize,
        a_free_reversal: usize,
        u_a: usize,
        u_urban: usize,
        u_costed: usize,
        ub_a: [usize; 3],
        ub_urban: [usize; 3],
        settled_a: Vec<f64>,
        settled_urban: Vec<f64>,
        us_a: Vec<f64>,
        us_urban: Vec<f64>,
        used: usize,
        rescored_gap: Vec<f64>,
        median_urban: usize,
        left_a: usize,
        left_urban: usize,
        example: Option<(f64, f64, Position, Position, EdgeRoute, EdgeRoute)>,
    }
    let mut acc = Acc::default();
    let vmax = eng.costs(Profile::Car).vmax_cm_per_unit;
    let mut tried = 0;
    while acc.used < nq && tried < nq * 10 {
        tried += 1;
        let (e1, e2) = (edges[rng.usize(edges.len())], edges[rng.usize(edges.len())]);
        if e1 == e2 {
            continue;
        }
        let (from, to) = (
            Position::new(e1, rng.range(0.1, 0.9) as f32),
            Position::new(e2, rng.range(0.1, 0.9) as f32),
        );
        let t0 = Instant::now();
        let (a, sa) = es.route(&node, &eng.xy, vmax, Start::Pos(from), Goal::Pos(to), true);
        let ta = t0.elapsed().as_secs_f64() * 1e6;
        let (b, _) = es.route(
            &restr_only,
            &eng.xy,
            vmax,
            Start::Pos(from),
            Goal::Pos(to),
            true,
        );
        let t0 = Instant::now();
        let (c, sc) = es.route(&urban, &eng.xy, vmax, Start::Pos(from), Goal::Pos(to), true);
        let tc = t0.elapsed().as_secs_f64() * 1e6;
        let (d, _) = es.route(
            &costed,
            &eng.xy,
            vmax,
            Start::Pos(from),
            Goal::Pos(to),
            true,
        );
        let (Some(a), Some(b), Some(c), Some(d)) = (a, b, c, d) else {
            continue;
        };
        acc.used += 1;
        acc.delta_restr
            .push((f64::from(b.cost) - f64::from(a.cost)) / 1000.0);
        acc.delta_urban
            .push((f64::from(c.cost) - f64::from(a.cost)) / 1000.0);
        let rescored = urban.route_cost(&a);
        match rescored {
            None => acc.a_violates += 1,
            Some(x) => acc
                .rescored_gap
                .push((x as f64 - f64::from(a.cost)) / 1000.0),
        }
        acc.a_free_reversal +=
            usize::from(a.edges.windows(2).any(|w| g.twin[w[0] as usize] == w[1]));
        acc.u_a += urban.u_turns(&a.edges) as usize;
        acc.u_urban += urban.u_turns(&c.edges) as usize;
        acc.u_costed += urban.u_turns(&d.edges) as usize;
        for (k, v) in urban.u_turn_breakdown(&a.edges).iter().enumerate() {
            acc.ub_a[k] += *v as usize;
        }
        for (k, v) in urban.u_turn_breakdown(&c.edges).iter().enumerate() {
            acc.ub_urban[k] += *v as usize;
        }
        acc.settled_a.push(f64::from(sa.settled));
        acc.settled_urban.push(f64::from(sc.settled));
        acc.us_a.push(ta);
        acc.us_urban.push(tc);
        acc.median_urban += c
            .edges
            .iter()
            .filter(|&&e| eng.geom.median_cross[e as usize])
            .count();
        acc.left_a += urban.turn_counts(&a.edges)[TurnKind::Left as usize] as usize;
        acc.left_urban += urban.turn_counts(&c.edges)[TurnKind::Left as usize] as usize;
        // ca tiêu biểu: theo nút quay ngược miễn phí, theo cạnh phải đi vòng; chọn tuyến vòng ngắn nhất cho dễ đọc
        let delta_s = (f64::from(c.cost) - f64::from(a.cost)) / 1000.0;
        let c_len = c.length_cm(g) as f64 / 100.0;
        if a.edges.windows(2).any(|w| g.twin[w[0] as usize] == w[1])
            && a.edges != c.edges
            && delta_s >= 40.0
            && acc.example.as_ref().is_none_or(|x| c_len < x.1)
        {
            acc.example = Some((delta_s, c_len, from, to, a.clone(), c.clone()));
        }
    }
    let n = acc.used as f64;
    let q = |v: &mut Vec<f64>, p: f64| {
        v.sort_by(f64::total_cmp);
        v[((p / 100.0) * v.len() as f64).ceil().max(1.0) as usize - 1]
    };
    println!("\n  {} truy vấn có đủ bốn mô hình cho ra đường", acc.used);
    println!(
        "  A = theo nút (không cấm rẽ, không chi phí rẽ, quay đầu miễn phí); B = A + cấm rẽ OSM; C = B + chi phí rẽ + cấm quay đầu giữa đường; D = như C nhưng quay đầu được ở mọi nơi với 20 s"
    );
    println!(
        "  tuyến của A vi phạm cấm rẽ hoặc quay đầu giữa đường theo luật của C: {}/{} ({:.1}%)",
        acc.a_violates,
        acc.used,
        100.0 * acc.a_violates as f64 / n
    );
    println!(
        "  tuyến của A có quay ngược ngay trên cùng một đoạn (đi vào cạnh twin): {}/{} ({:.1}%)",
        acc.a_free_reversal,
        acc.used,
        100.0 * acc.a_free_reversal as f64 / n
    );
    println!("  chênh thời gian ước tính so với A (giây, dương = A đã ước lượng thấp):");
    for (name, v) in [
        ("B (có cấm rẽ)", &mut acc.delta_restr),
        ("C (đầy đủ)", &mut acc.delta_urban),
    ] {
        println!(
            "    {name:<14} trung bình {:+.1}, trung vị {:+.1}, p95 {:+.1}, lớn nhất {:+.1}",
            mean(v),
            q(v, 50.0),
            q(v, 95.0),
            q(v, 100.0)
        );
    }
    if !acc.rescored_gap.is_empty() {
        println!(
            "  với tuyến của A hợp lệ ({}): chấm lại bằng chi phí rẽ cộng thêm trung bình {:.1} s",
            acc.rescored_gap.len(),
            mean(&acc.rescored_gap)
        );
    }
    println!(
        "  số lần quay đầu theo định nghĩa hình học (rẽ ≥ 150° trong chuỗi nút cách nhau ≤ 60 m), tổng trên {} truy vấn: A {}, C {}, D {}",
        acc.used, acc.u_a, acc.u_urban, acc.u_costed
    );
    println!(
        "  quay đầu của A theo cách quay: đi ngược trực tiếp {}, qua lối dải phân cách {}, khác (chuỗi nút gần nhau cộng góc ≥ 150°) {}; của C: {}, {}, {}",
        acc.ub_a[0], acc.ub_a[1], acc.ub_a[2], acc.ub_urban[0], acc.ub_urban[1], acc.ub_urban[2]
    );
    println!(
        "  lần đi vào lối qua dải phân cách của C: {}; số lần rẽ trái: A {}, C {}",
        acc.median_urban, acc.left_a, acc.left_urban
    );
    println!(
        "  chi phí tìm kiếm: số trạng thái cạnh đã duyệt A {:.0}, C {:.0}; thời gian trung bình A {:.0} µs, C {:.0} µs",
        mean(&acc.settled_a),
        mean(&acc.settled_urban),
        mean(&acc.us_a),
        mean(&acc.us_urban)
    );
    if let Some((delta_s, c_len, from, to, a, c)) = &acc.example {
        println!(
            "\n  một ca thật trên bản đồ này: xe ở {:.0}% cạnh {} (hướng {:.0}°), khách ở {:.0}% cạnh {}; theo nút đi ngược ngay còn theo cạnh phải đi vòng {c_len:.0} m, chậm hơn {delta_s:.1} s",
            f64::from(from.frac) * 100.0,
            from.edge,
            node.geom.bear_out[from.edge as usize],
            f64::from(to.frac) * 100.0,
            to.edge
        );
        for (name, tm, r) in [
            ("theo nút (quay ngược miễn phí)", &node, a),
            ("theo cạnh (cấm quay đầu giữa đường)", &urban, c),
        ] {
            println!(
                "    {name}: {:.1} s, {:.0} m, quay đầu {}",
                f64::from(r.cost) / 1000.0,
                r.length_cm(g) as f64 / 100.0,
                tm.u_turns(&r.edges)
            );
            println!("      → {}", describe(g, tm, r));
        }
    }
    println!("\nuptime lúc kết thúc: {}", uptime());
}
