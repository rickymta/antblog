//! Kiểm thử ngẫu nhiên: mọi thuật toán nhanh phải cho đúng chi phí của Dijkstra trên đồ thị fixture
//! (ALT, hai chiều, CH, CCH, ma trận bucket, A* theo cạnh khi không có chi phí rẽ). Không dùng mạng, không cần bản đồ thật.

use proptest::prelude::*;
use road_graph::Graph;
use road_graph::fixture::{compile_fixture, generate};
use route_engine::alt::{Landmarks, largest_scc_nodes};
use route_engine::bidir::{BiSearcher, Stop};
use route_engine::cch::Cch;
use route_engine::ch::{Ch, ChWorkspace};
use route_engine::matrix::bucket_matrix;
use route_engine::net::{INF, Rev, View};
use route_engine::position::Leg;
use route_engine::profile::{Costs, Metric, Profile};
use route_engine::search::{NodeXy, Searcher};
use route_engine::turn::{EdgeGeom, EdgeSearcher, Goal, Restrictions, Start, TurnCosts, TurnModel};
use std::sync::OnceLock;

struct World {
    g: Graph,
    rev: Rev,
    xy: NodeXy,
    costs: Vec<Costs>,
    ch: Vec<Ch>,
    cch_ch: Vec<Ch>,
    alt: Vec<Landmarks>,
    geom: EdgeGeom,
}

fn world() -> &'static World {
    static W: OnceLock<World> = OnceLock::new();
    W.get_or_init(|| {
        let g = compile_fixture(&generate());
        let rev = Rev::build(&g);
        let xy = NodeXy::build(&g);
        let costs: Vec<Costs> = Profile::ALL
            .iter()
            .map(|&p| Costs::build(&g, p, Metric::Time))
            .collect();
        let mut ch = Vec::new();
        let mut cch_ch = Vec::new();
        let mut alt = Vec::new();
        for c in &costs {
            let view = View::new(&g, &rev, &c.w);
            ch.push(Ch::build(view));
            cch_ch.push(Cch::build(view, &xy).customize(&c.w));
            let scc = largest_scc_nodes(&g, &rev, &c.w);
            let seed = scc.iter().position(|&b| b).unwrap() as u32;
            alt.push(Landmarks::build(view, 8, &scc, seed));
        }
        let geom = EdgeGeom::build(&g);
        World {
            g,
            rev,
            xy,
            costs,
            ch,
            cch_ch,
            alt,
            geom,
        }
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]

    #[test]
    fn every_fast_algorithm_matches_dijkstra(pi in 0usize..3, a in 0u32..10_000, b in 0u32..10_000) {
        let w = world();
        let n = w.g.node_count() as u32;
        let (s, t) = (a % n, b % n);
        let view = View::new(&w.g, &w.rev, &w.costs[pi].w);
        let mut uni = Searcher::new(w.g.node_count());
        let (truth, _) = uni.dijkstra(view, s, t);
        let want = truth.as_ref().map(|p| p.cost);

        // A* chim bay
        let (x, _) = uni.astar(view, &w.xy, w.costs[pi].vmax_cm_per_unit, s, t);
        prop_assert_eq!(x.map(|p| p.cost), want);
        // ALT
        let (x, _) = uni.run(view, s, t, |u| w.alt[pi].lower_bound(u, t));
        prop_assert_eq!(x.map(|p| p.cost), want);
        // hai chiều (đúng)
        let mut bi = BiSearcher::new(w.g.node_count());
        let (x, _) = bi.run(view, s, t, Stop::Correct);
        prop_assert_eq!(x.map(|p| p.cost), want);
        // CH và CCH
        let mut ws = ChWorkspace::new(w.g.node_count());
        prop_assert_eq!(w.ch[pi].distance(&mut ws, s, t), want);
        prop_assert_eq!(w.cch_ch[pi].distance(&mut ws, s, t), want);
        // đường giải nén hợp lệ và đúng chi phí
        if let Some(p) = w.ch[pi].path(&mut ws, s, t) {
            prop_assert_eq!(u64::from(p.cost), p.cost_under(&w.costs[pi].w));
            for win in p.edges.windows(2) {
                prop_assert_eq!(w.g.head[win[0] as usize], w.g.tail[win[1] as usize]);
            }
        }
        // đồ thị theo cạnh không phạt, không cấm = Dijkstra theo nút
        let none = Restrictions::default();
        let tm = TurnModel { view, geom: &w.geom, restr: &none, costs: TurnCosts::none() };
        let mut es = EdgeSearcher::new(w.g.edge_count());
        let (r, _) = es.route(&tm, &w.xy, w.costs[pi].vmax_cm_per_unit, Start::Node(s), Goal::Node(t), true);
        prop_assert_eq!(r.map(|r| r.cost), want);
    }

    #[test]
    fn bucket_matrix_matches_dijkstra(pi in 0usize..2, srcs in proptest::collection::vec(0u32..10_000, 1..6), dsts in proptest::collection::vec(0u32..10_000, 1..6)) {
        let w = world();
        let n = w.g.node_count() as u32;
        let view = View::new(&w.g, &w.rev, &w.costs[pi].w);
        let mk = |v: &Vec<u32>| v.iter().map(|&x| vec![Leg { node: x % n, cost: 0 }]).collect::<Vec<_>>();
        let m = bucket_matrix(&w.ch[pi], &mk(&srcs), &mk(&dsts), true);
        let mut uni = Searcher::new(w.g.node_count());
        for (i, &s) in srcs.iter().enumerate() {
            for (j, &t) in dsts.iter().enumerate() {
                let (truth, _) = uni.dijkstra(view, s % n, t % n);
                let want = truth.map_or(INF, |p| p.cost);
                prop_assert_eq!(m.get(i, j), want);
            }
        }
    }
}
