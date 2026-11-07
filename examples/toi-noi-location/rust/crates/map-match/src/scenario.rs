//! Tìm các «ca khó» thật trên đồ thị và dựng lộ trình đi qua chúng: cầu vượt (đường ở trên và đường ở dưới cùng một chỗ trên
//! bản đồ phẳng), đường song song, đường gom bên cạnh đường chính có dải phân cách, hầm.
//!
//! Việc tìm chỉ dùng hình học và thẻ của đồ thị (không dùng ground truth của vết), nên chạy được trên đồ thị HCM thật lẫn
//! fixture. Kết quả sắp theo mã cạnh để tất định; tập «tune» và «eval» chia theo chẵn/lẻ chỉ số để không dùng chung ca.

use crate::Rng;
use crate::route::Router;
use crate::trace::{Canyon, DriveConfig, NoiseConfig, Trace, corrupt, drive, zone_around};
use geo_core::{LatLng, bearing_deg, bearing_diff_deg, haversine_m};
use road_graph::snap::SnapIndex;
use road_graph::{EdgeId, Graph, HighwayClass, Mode, NONE, flag};
use rustc_hash::FxHashSet;

/// Cạnh vào theo từng nút (đảo chiều CSR), để đi ngược lộ trình từ một cạnh.
pub struct InEdges {
    first: Vec<u32>,
    list: Vec<EdgeId>,
}

impl InEdges {
    pub fn new(g: &Graph) -> Self {
        let n = g.node_count();
        let mut first = vec![0u32; n + 1];
        for &h in &g.head {
            first[h as usize + 1] += 1;
        }
        for i in 0..n {
            first[i + 1] += first[i];
        }
        let mut fill = first.clone();
        let mut list = vec![0; g.edge_count()];
        for e in 0..g.edge_count() {
            let h = g.head[e] as usize;
            list[fill[h] as usize] = e as u32;
            fill[h] += 1;
        }
        Self { first, list }
    }
    fn incoming(&self, n: u32) -> &[EdgeId] {
        &self.list[self.first[n as usize] as usize..self.first[n as usize + 1] as usize]
    }
}

fn start_bearing(g: &Graph, e: EdgeId) -> f64 {
    let p = g.edge_points(e);
    bearing_deg(p[0], p[1])
}
fn end_bearing(g: &Graph, e: EdgeId) -> f64 {
    let p = g.edge_points(e);
    bearing_deg(p[p.len() - 2], p[p.len() - 1])
}
fn len_m(g: &Graph, e: EdgeId) -> f64 {
    f64::from(g.length_cm[e as usize]) / 100.0
}
fn is_rep(g: &Graph, e: EdgeId) -> bool {
    let t = g.twin[e as usize];
    t == NONE || e < t
}

/// Kéo dài lộ trình từ cạnh `via` về hai phía bằng đi ngẫu nhiên ưu tiên đi thẳng (không quay đầu), ít nhất `before_m` trước và `after_m` sau.
pub fn extend_route(
    g: &Graph,
    rev: &InEdges,
    mode: Mode,
    via: EdgeId,
    before_m: f64,
    after_m: f64,
    rng: &mut Rng,
) -> Vec<EdgeId> {
    let bit = mode.bit();
    let mut seen: FxHashSet<EdgeId> = FxHashSet::default();
    seen.insert(via);
    let pick = |opts: &mut Vec<(EdgeId, f64)>, rng: &mut Rng| -> Option<EdgeId> {
        if opts.is_empty() {
            return None;
        }
        opts.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
        Some(if rng.chance(0.8) {
            opts[0].0
        } else {
            opts[rng.usize(opts.len())].0
        })
    };
    let mut fwd = Vec::new();
    let (mut cur, mut acc) = (via, 0.0);
    while acc < after_m {
        let eb = end_bearing(g, cur);
        let mut opts: Vec<(EdgeId, f64)> = g
            .out_edges(g.head[cur as usize])
            .filter(|&f| {
                g.modes[f as usize] & bit != 0 && !seen.contains(&f) && g.twin[cur as usize] != f
            })
            .map(|f| (f, bearing_diff_deg(eb, start_bearing(g, f))))
            .collect();
        let Some(n) = pick(&mut opts, rng) else { break };
        seen.insert(n);
        fwd.push(n);
        acc += len_m(g, n);
        cur = n;
    }
    let mut back = Vec::new();
    let (mut cur, mut acc) = (via, 0.0);
    while acc < before_m {
        let sb = start_bearing(g, cur);
        let mut opts: Vec<(EdgeId, f64)> = rev
            .incoming(g.tail[cur as usize])
            .iter()
            .copied()
            .filter(|&f| {
                g.modes[f as usize] & bit != 0 && !seen.contains(&f) && g.twin[cur as usize] != f
            })
            .map(|f| (f, bearing_diff_deg(end_bearing(g, f), sb)))
            .collect();
        let Some(n) = pick(&mut opts, rng) else { break };
        seen.insert(n);
        back.push(n);
        acc += len_m(g, n);
        cur = n;
    }
    back.reverse();
    back.push(via);
    back.extend(fwd);
    back
}

/// Chuyến ngẫu nhiên dài khoảng `[min_m, max_m]` theo đường ngắn nhất giữa hai nút ngẫu nhiên.
pub fn random_trip(
    g: &Graph,
    router: &mut Router,
    mode: Mode,
    rng: &mut Rng,
    min_m: f64,
    max_m: f64,
) -> Option<Vec<EdgeId>> {
    let bit = mode.bit();
    let ok_node = |n: u32| g.out_edges(n).any(|e| g.modes[e as usize] & bit != 0);
    for _ in 0..60 {
        let a = rng.usize(g.node_count()) as u32;
        if !ok_node(a) {
            continue;
        }
        let pa = g.node_latlng(a);
        // chọn đích trong vành chim bay [0,6·min, max] bằng cách thử vài nút
        for _ in 0..40 {
            let b = rng.usize(g.node_count()) as u32;
            let d = haversine_m(pa, g.node_latlng(b));
            if b != a && ok_node(b) && d >= min_m * 0.7 && d <= max_m * 0.8 {
                if let Some((edges, cm)) = router.shortest_path(g, mode, a, b) {
                    let m = f64::from(cm) / 100.0;
                    if m >= min_m && m <= max_m * 1.3 {
                        return Some(edges);
                    }
                }
                break;
            }
        }
    }
    None
}

#[derive(Debug, Clone, Copy)]
pub struct Crossing {
    pub bridge: EdgeId,
    pub lower: EdgeId,
    pub point: LatLng,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairKind {
    /// Hai đường cùng hạng (kể cả hai chiều xe của đại lộ có dải phân cách, là hai way một chiều).
    Parallel,
    /// Đường chính (hạng ≤ secondary) với đường gom/đường nhỏ chạy sát bên.
    Frontage,
}

#[derive(Debug, Clone, Copy)]
pub struct ParallelPair {
    /// Với `Frontage`, `a` là đường chính và `b` là đường gom.
    pub a: EdgeId,
    pub b: EdgeId,
    pub overlap_m: f64,
    pub gap_m: f64,
    pub same_dir: bool,
    pub kind: PairKind,
}

fn is_main(g: &Graph, e: EdgeId) -> bool {
    g.class[e as usize] <= HighwayClass::SecondaryLink as u8
}

fn shares_node(g: &Graph, a: EdgeId, b: EdgeId) -> bool {
    let (ta, ha, tb, hb) = (
        g.tail[a as usize],
        g.head[a as usize],
        g.tail[b as usize],
        g.head[b as usize],
    );
    ta == tb || ta == hb || ha == tb || ha == hb
}

/// Mẫu đều trên polyline: (điểm, hướng đoạn chứa điểm).
fn sample_polyline(pts: &[LatLng], step: f64) -> Vec<(LatLng, f64)> {
    let mut out = Vec::new();
    let mut next = step / 2.0;
    let mut acc = 0.0;
    for w in pts.windows(2) {
        let l = haversine_m(w[0], w[1]);
        let b = bearing_deg(w[0], w[1]);
        while next <= acc + l && l > 0.0 {
            let t = (next - acc) / l;
            let p = LatLng::try_new(
                w[0].lat() + t * (w[1].lat() - w[0].lat()),
                w[0].lng() + t * (w[1].lng() - w[0].lng()),
            )
            .expect("hợp lệ");
            out.push((p, b));
            next += step;
        }
        acc += l;
    }
    out
}

pub struct Catalog {
    pub overpasses: Vec<Crossing>,
    pub pairs: Vec<ParallelPair>,
    pub tunnels: Vec<EdgeId>,
}

impl Catalog {
    pub fn new(g: &Graph, idx: &SnapIndex, mode: Mode) -> Self {
        let bit = mode.bit();
        let mut overpasses = Vec::new();
        let mut tunnels = Vec::new();
        let mut pairs: Vec<ParallelPair> = Vec::new();
        let mut seen_pair: FxHashSet<(EdgeId, EdgeId)> = FxHashSet::default();
        for e in 0..g.edge_count() as u32 {
            if g.modes[e as usize] & bit == 0 || !is_rep(g, e) {
                continue;
            }
            let len = len_m(g, e);
            let fl = g.flags[e as usize];
            if fl & flag::TUNNEL != 0 && len >= 80.0 {
                tunnels.push(e);
            }
            let pts = g.edge_points(e);
            if fl & flag::BRIDGE != 0 && len >= 80.0 {
                let mut found: FxHashSet<EdgeId> = FxHashSet::default();
                for (p, _) in sample_polyline(&pts, 10.0) {
                    for c in idx.candidates(g, p, 6.0, mode, 6) {
                        let lower_ok = c.edge != e
                            && c.twin != Some(e)
                            && !shares_node(g, c.edge, e)
                            && g.flags[c.edge as usize] & flag::BRIDGE == 0;
                        if lower_ok && found.insert(c.edge) {
                            overpasses.push(Crossing {
                                bridge: e,
                                lower: c.edge,
                                point: p,
                            });
                        }
                    }
                }
            }
            if len >= 100.0 {
                let mut count: std::collections::BTreeMap<EdgeId, (usize, f64, bool)> =
                    std::collections::BTreeMap::new();
                for (p, b) in sample_polyline(&pts, 12.0) {
                    for c in idx.candidates(g, p, 35.0, mode, 12) {
                        if c.dist_m < 6.0
                            || c.edge == e
                            || c.twin == Some(e)
                            || g.way_id[c.edge as usize] == g.way_id[e as usize]
                        {
                            continue;
                        }
                        let d = bearing_diff_deg(b, c.bearing_deg);
                        if d < 25.0 || d > 155.0 {
                            let ent = count.entry(c.edge).or_insert((0, 0.0, d < 25.0));
                            ent.0 += 1;
                            ent.1 += c.dist_m;
                        }
                    }
                }
                if let Some((&b_edge, &(n, dsum, same))) = count.iter().max_by_key(|(_, v)| v.0)
                    && n as f64 * 12.0 >= 100.0
                {
                    let key = (e.min(b_edge), e.max(b_edge));
                    if seen_pair.insert(key) {
                        let (ma, mb) = (is_main(g, e), is_main(g, b_edge));
                        let (a, b, kind) = if ma != mb {
                            (
                                if ma { e } else { b_edge },
                                if ma { b_edge } else { e },
                                PairKind::Frontage,
                            )
                        } else {
                            (e, b_edge, PairKind::Parallel)
                        };
                        pairs.push(ParallelPair {
                            a,
                            b,
                            overlap_m: n as f64 * 12.0,
                            gap_m: dsum / n as f64,
                            same_dir: same,
                            kind,
                        });
                    }
                }
            }
        }
        overpasses.sort_by_key(|c| (c.bridge, c.lower));
        pairs.sort_by_key(|p| (p.a, p.b));
        Self {
            overpasses,
            pairs,
            tunnels,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Overpass,
    Underpass,
    Parallel,
    FrontageMain,
    FrontageSide,
    Tunnel,
    SignalLoss,
    General,
}

impl Kind {
    pub const HARD: [Kind; 7] = [
        Kind::Overpass,
        Kind::Underpass,
        Kind::Parallel,
        Kind::FrontageMain,
        Kind::FrontageSide,
        Kind::Tunnel,
        Kind::SignalLoss,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Kind::Overpass => "chạy trên cầu vượt",
            Kind::Underpass => "chạy dưới cầu vượt",
            Kind::Parallel => "hai đường song song",
            Kind::FrontageMain => "đường chính, có đường gom bên cạnh",
            Kind::FrontageSide => "đường gom bên cạnh đường chính",
            Kind::Tunnel => "hầm (mất tín hiệu)",
            Kind::SignalLoss => "mất tín hiệu 25 giây",
            Kind::General => "chuyến thường",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Split {
    /// Chỉ số chẵn, dùng để chọn tham số.
    Tune,
    /// Chỉ số lẻ, dùng để báo cáo.
    Eval,
    All,
}

#[derive(Debug, Clone)]
pub struct Scenario {
    pub kind: Kind,
    pub name: String,
    pub route: Vec<EdgeId>,
    pub focus: Option<LatLng>,
    pub focus_radius_m: f64,
    /// Cặp cạnh gây ca khó: (cầu, đường dưới), (đường chính, đường gom), hoặc hai đường song song.
    pub pair: Option<(EdgeId, EdgeId)>,
}

fn stride<T: Clone>(v: &[T], split: Split, limit: usize) -> Vec<T> {
    let picked: Vec<T> = v
        .iter()
        .enumerate()
        .filter(|(i, _)| match split {
            Split::Tune => i % 2 == 0,
            Split::Eval => i % 2 == 1,
            Split::All => true,
        })
        .map(|(_, x)| x.clone())
        .collect();
    if picked.len() <= limit {
        return picked;
    }
    (0..limit)
        .map(|i| picked[i * picked.len() / limit].clone())
        .collect()
}

/// Dựng các kịch bản khó. Mỗi loại tối đa `per_kind` ca. `SignalLoss` là chuyến thường có khoảng mất tín hiệu.
#[allow(clippy::too_many_arguments)]
pub fn build(
    g: &Graph,
    cat: &Catalog,
    rev: &InEdges,
    router: &mut Router,
    mode: Mode,
    split: Split,
    per_kind: usize,
    seed: u64,
) -> Vec<Scenario> {
    let mut rng = Rng::new(seed);
    let mut out = Vec::new();
    let mut add = |kind: Kind,
                   name: String,
                   via: EdgeId,
                   focus: LatLng,
                   pair: Option<(EdgeId, EdgeId)>,
                   rng: &mut Rng| {
        let route = extend_route(g, rev, mode, via, 350.0, 350.0, rng);
        out.push(Scenario {
            kind,
            name,
            route,
            focus: Some(focus),
            focus_radius_m: 45.0,
            pair,
        });
    };
    for c in stride(&cat.overpasses, split, per_kind) {
        add(
            Kind::Overpass,
            format!("cầu vượt cạnh {} cắt cạnh {}", c.bridge, c.lower),
            c.bridge,
            c.point,
            Some((c.bridge, c.lower)),
            &mut rng,
        );
        add(
            Kind::Underpass,
            format!("đường dưới cạnh {} dưới cầu cạnh {}", c.lower, c.bridge),
            c.lower,
            c.point,
            Some((c.bridge, c.lower)),
            &mut rng,
        );
    }
    let parallel: Vec<ParallelPair> = cat
        .pairs
        .iter()
        .copied()
        .filter(|p| p.kind == PairKind::Parallel)
        .collect();
    for p in stride(&parallel, split, per_kind) {
        let mid = g.edge_points(p.a);
        add(
            Kind::Parallel,
            format!(
                "cạnh {} song song cạnh {} ({:.0} m, cách {:.0} m)",
                p.a, p.b, p.overlap_m, p.gap_m
            ),
            p.a,
            mid[mid.len() / 2],
            Some((p.a, p.b)),
            &mut rng,
        );
    }
    let frontage: Vec<ParallelPair> = cat
        .pairs
        .iter()
        .copied()
        .filter(|p| p.kind == PairKind::Frontage)
        .collect();
    for p in stride(&frontage, split, per_kind) {
        let (ma, mb) = (g.edge_points(p.a), g.edge_points(p.b));
        add(
            Kind::FrontageMain,
            format!(
                "đường chính {} có đường gom {} ({:.0} m, cách {:.0} m)",
                p.a, p.b, p.overlap_m, p.gap_m
            ),
            p.a,
            ma[ma.len() / 2],
            Some((p.a, p.b)),
            &mut rng,
        );
        add(
            Kind::FrontageSide,
            format!(
                "đường gom {} bên đường chính {} ({:.0} m, cách {:.0} m)",
                p.b, p.a, p.overlap_m, p.gap_m
            ),
            p.b,
            mb[mb.len() / 2],
            Some((p.a, p.b)),
            &mut rng,
        );
    }
    for e in stride(&cat.tunnels, split, per_kind) {
        let pts = g.edge_points(e);
        add(
            Kind::Tunnel,
            format!("hầm cạnh {e}"),
            e,
            pts[pts.len() / 2],
            None,
            &mut rng,
        );
    }
    for i in 0..per_kind {
        if let Some(route) = random_trip(g, router, mode, &mut rng, 2000.0, 4500.0) {
            out.push(Scenario {
                kind: Kind::SignalLoss,
                name: format!("chuyến ngẫu nhiên {i} có khoảng mất tín hiệu"),
                route,
                focus: None,
                focus_radius_m: 0.0,
                pair: None,
            });
        }
    }
    out
}

/// Biến kịch bản thành vết có nhiễu. `canyon_bias_m > 0` thêm vùng lệch ổn định quanh điểm khó (hướng ngẫu nhiên).
pub fn realize(
    g: &Graph,
    sc: &Scenario,
    drive_cfg: &DriveConfig,
    base: &NoiseConfig,
    canyon_bias_m: f64,
    seed: u64,
) -> Trace {
    let mut rng = Rng::new(seed);
    let truth = drive(g, &sc.route, drive_cfg, &mut rng);
    let mut noise = base.clone();
    let end_s = truth.last().map_or(0.0, |p| p.t_ms as f64 / 1000.0);
    if let Some(f) = sc.focus
        && canyon_bias_m > 0.0
        && let Some((a, b)) = zone_around(&truth, f, sc.focus_radius_m)
    {
        noise.canyons.push(Canyon {
            start_s: a,
            end_s: b,
            bias_m: canyon_bias_m,
            bearing_deg: rng.range(0.0, 360.0),
            sigma_mult: 1.5,
            reported_mult: 1.2,
        });
    }
    match sc.kind {
        Kind::Tunnel => {
            // không có điểm nào trong lúc xe nằm trên cạnh có cờ hầm, thêm 1 giây mỗi đầu
            let mut inside = truth
                .iter()
                .filter(|p| g.flags[p.edge as usize] & flag::TUNNEL != 0)
                .map(|p| p.t_ms as f64 / 1000.0);
            if let Some(first) = inside.next() {
                let last = inside.next_back().unwrap_or(first);
                noise.dropouts.push((first - 1.0, last + 1.0));
            }
        }
        Kind::SignalLoss => {
            let start = 0.4 * end_s;
            noise.dropouts.push((start, start + 25.0));
        }
        _ => {}
    }
    corrupt(&sc.route, truth, &noise, &mut rng)
}

#[cfg(test)]
mod tests {
    use super::*;
    use road_graph::fixture::{compile_fixture, generate};

    fn setup() -> (Graph, SnapIndex) {
        let g = compile_fixture(&generate());
        let idx = SnapIndex::build(&g);
        (g, idx)
    }

    #[test]
    fn fixture_has_overpass_and_divided_avenue_pair() {
        let (g, idx) = setup();
        let cat = Catalog::new(&g, &idx, Mode::Car);
        assert!(!cat.overpasses.is_empty(), "fixture có cầu vượt cắt đại lộ");
        assert!(
            cat.pairs
                .iter()
                .any(|p| p.kind == PairKind::Parallel && !p.same_dir),
            "hai chiều đại lộ là cặp song song ngược chiều"
        );
    }

    #[test]
    fn routes_are_contiguous_and_forced_through_via() {
        let (g, idx) = setup();
        let cat = Catalog::new(&g, &idx, Mode::Car);
        let rev = InEdges::new(&g);
        let mut rng = Rng::new(1);
        for c in cat.overpasses.iter().take(3) {
            let r = extend_route(&g, &rev, Mode::Car, c.bridge, 300.0, 300.0, &mut rng);
            assert!(r.contains(&c.bridge));
            for w in r.windows(2) {
                assert_eq!(g.head[w[0] as usize], g.tail[w[1] as usize]);
            }
            assert!(r.iter().all(|&e| g.modes[e as usize] & Mode::CAR_BIT != 0));
        }
    }

    #[test]
    fn split_is_disjoint() {
        let v: Vec<u32> = (0..20).collect();
        let (a, b) = (stride(&v, Split::Tune, 100), stride(&v, Split::Eval, 100));
        assert!(a.iter().all(|x| !b.contains(x)));
        assert_eq!(a.len() + b.len(), 20);
    }
}
