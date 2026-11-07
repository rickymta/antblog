//! LOC-23: đồ thị theo cạnh (edge-based) với chi phí rẽ, quay đầu, cấm rẽ OSM và qua dải phân cách.
//!
//! Trạng thái tìm kiếm là **cạnh có hướng** (xe đang đi trên cạnh nào), không phải nút. Chuyển trạng thái `e1 → e2`
//! chỉ khi `head(e1) = tail(e2)`, trừ khi bị cấm; chi phí = trọng số `e2` + chi phí rẽ theo góc đổi hướng.
//! Đồ thị theo cạnh không được dựng ra bộ nhớ: các chuyển trạng thái sinh ngay lúc duyệt từ CSR theo nút.
//!
//! Mô hình theo nút bỏ sót ba thứ vì nút không có hướng tới: (1) xe vừa tới nút có thể quay ngược lại miễn phí,
//! (2) cấm rẽ là quan hệ giữa **hai cạnh**, (3) rẽ trái hai lần qua dải phân cách (một cách quay đầu) không tốn gì.

use crate::net::{INF, Labels, MinHeap, NONE, Stats, View};
use crate::position::Position;
use crate::profile::Profile;
use crate::search::NodeXy;
use geo_core::bearing_deg;
use road_graph::{EdgeId, Graph, HighwayClass, NodeId, flag};
use rustc_hash::{FxHashMap, FxHashSet};

/// Quy tắc quay đầu trực tiếp (đi ngược lại đúng cạnh `twin`, hoặc đổi hướng ≥ `reversal_deg`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UTurnPolicy {
    /// Miễn phí ở mọi nơi: đúng là điều mô hình theo nút ngầm giả định.
    Free,
    /// Cho phép ở mọi nơi nhưng tốn `u_turn_ms`.
    Costed,
    /// Chỉ ở ngõ cụt (không còn đường nào khác đi ra). Muốn quay đầu giữa đường phải đi vòng hoặc qua lối mở dải phân cách.
    DeadEndOnly,
}

/// Chi phí rẽ (mili giây), **giả định của lab**: xe chạy bên phải, rẽ trái cắt dòng ngược chiều nên tốn hơn rẽ phải.
#[derive(Debug, Clone, Copy)]
pub struct TurnCosts {
    pub straight_ms: u32,
    pub right_ms: u32,
    pub left_ms: u32,
    pub u_turn_ms: u32,
    /// Cộng thêm khi đi vào cạnh nối hai chiều của đường có dải phân cách.
    pub median_ms: u32,
    pub u_turn_policy: UTurnPolicy,
    /// Dưới góc này (độ) là đi thẳng.
    pub straight_deg: f32,
    /// Từ góc này trở lên là quay đầu.
    pub reversal_deg: f32,
}

impl TurnCosts {
    /// Không phạt gì, quay đầu miễn phí: kết quả tương đương mô hình theo nút (chỉ còn cấm rẽ nếu có).
    pub fn none() -> Self {
        Self {
            straight_ms: 0,
            right_ms: 0,
            left_ms: 0,
            u_turn_ms: 0,
            median_ms: 0,
            u_turn_policy: UTurnPolicy::Free,
            straight_deg: 35.0,
            reversal_deg: 150.0,
        }
    }
    /// Cấu hình đô thị mặc định của lab.
    pub fn urban() -> Self {
        Self {
            straight_ms: 0,
            right_ms: 3_000,
            left_ms: 6_000,
            u_turn_ms: 20_000,
            median_ms: 4_000,
            u_turn_policy: UTurnPolicy::DeadEndOnly,
            straight_deg: 35.0,
            reversal_deg: 150.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnKind {
    Straight,
    Right,
    Left,
    UTurn,
}

/// Hình học theo cạnh dùng chung cho mọi profile: hướng vào, hướng ra, cạnh nối qua dải phân cách.
#[derive(Debug, Clone)]
pub struct EdgeGeom {
    /// Hướng (độ, 0 = bắc) của đoạn cuối cạnh (lúc rời cạnh).
    pub bear_in: Vec<f32>,
    /// Hướng của đoạn đầu cạnh (lúc vào cạnh).
    pub bear_out: Vec<f32>,
    pub median_cross: Vec<bool>,
}

fn first_last_bearing(pts: &[geo_core::LatLng]) -> (f32, f32) {
    let seg = |a: geo_core::LatLng, b: geo_core::LatLng| (a != b).then(|| bearing_deg(a, b) as f32);
    let first = pts.windows(2).find_map(|w| seg(w[0], w[1])).unwrap_or(0.0);
    let last = pts
        .windows(2)
        .rev()
        .find_map(|w| seg(w[0], w[1]))
        .unwrap_or(first);
    (first, last)
}

impl EdgeGeom {
    pub fn build(g: &Graph) -> Self {
        let m = g.edge_count();
        let mut bear_in = vec![0.0f32; m];
        let mut bear_out = vec![0.0f32; m];
        for e in 0..m {
            let (f, l) = first_last_bearing(&g.edge_points(e as u32));
            bear_out[e] = f;
            bear_in[e] = l;
        }
        let median_cross = detect_median_crossings(g, &bear_in, &bear_out);
        Self {
            bear_in,
            bear_out,
            median_cross,
        }
    }
}

fn ang_diff(a: f32, b: f32) -> f32 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

/// Cạnh ngắn (≤ 40 m) nối hai nút mà mỗi nút nằm trên một chiều đường lớn một chiều (ô tô), hai chiều đó ngược hướng nhau.
/// Trong fixture là lối mở dải phân cách trên «Đại lộ Trung Tâm»; ngoài đời là đường dẫn qua dải phân cách.
fn detect_median_crossings(g: &Graph, bear_in: &[f32], bear_out: &[f32]) -> Vec<bool> {
    let big = |c: u8| {
        matches!(
            HighwayClass::from_u8(c),
            Some(
                HighwayClass::Trunk
                    | HighwayClass::Primary
                    | HighwayClass::Secondary
                    | HighwayClass::TrunkLink
                    | HighwayClass::PrimaryLink
            )
        )
    };
    // hướng của các chiều đường lớn một chiều chạm vào mỗi nút (cạnh ra dùng bear_out, cạnh vào dùng bear_in)
    let n = g.node_count();
    let mut carriage: Vec<Vec<f32>> = vec![Vec::new(); n];
    for e in 0..g.edge_count() {
        if g.flags[e] & flag::ONEWAY_CAR != 0
            && big(g.class[e])
            && g.modes[e] & road_graph::Mode::CAR_BIT != 0
        {
            carriage[g.tail[e] as usize].push(bear_out[e]);
            carriage[g.head[e] as usize].push(bear_in[e]);
        }
    }
    (0..g.edge_count())
        .map(|e| {
            if g.length_cm[e] > 4_000 || g.modes[e] & road_graph::Mode::CAR_BIT == 0 {
                return false;
            }
            let (a, b) = (g.tail[e] as usize, g.head[e] as usize);
            // chính cạnh này không được là một phần của đường lớn một chiều
            if g.flags[e] & flag::ONEWAY_CAR != 0 && big(g.class[e]) {
                return false;
            }
            carriage[a]
                .iter()
                .any(|&x| carriage[b].iter().any(|&y| ang_diff(x, y) > 150.0))
        })
        .collect()
}

/// Số liệu về cách đọc các quan hệ cấm rẽ.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RestrictionStats {
    pub relations: usize,
    pub applied: usize,
    pub conditional_skipped: usize,
    pub unresolved_skipped: usize,
    pub via_way_skipped: usize,
    pub other_mode_skipped: usize,
    pub excepted_skipped: usize,
    pub no_kinds: usize,
    pub only_kinds: usize,
    pub banned_pairs: usize,
}

/// Cấm rẽ áp cho một profile: cặp (cạnh vào, cạnh ra) bị cấm, và `only_*` (chỉ được rẽ vào các cạnh liệt kê).
#[derive(Debug, Clone, Default)]
pub struct Restrictions {
    pub banned: FxHashSet<(EdgeId, EdgeId)>,
    pub only: FxHashMap<EdgeId, Vec<EdgeId>>,
    pub stats: RestrictionStats,
}

impl Restrictions {
    pub fn build(g: &Graph, profile: Profile) -> Self {
        let mut r = Restrictions::default();
        let applies_key = |key: &str| match profile {
            Profile::Car => matches!(
                key,
                "restriction"
                    | "restriction:motorcar"
                    | "restriction:motor_vehicle"
                    | "restriction:vehicle"
            ),
            Profile::Motorbike => matches!(
                key,
                "restriction"
                    | "restriction:motorcycle"
                    | "restriction:motor_vehicle"
                    | "restriction:vehicle"
            ),
            Profile::Foot => false,
        };
        let excepts = |tags: &[(String, String)]| {
            tags.iter().any(|(k, v)| {
                k == "except"
                    && v.split(';').any(|x| {
                        let x = x.trim();
                        match profile {
                            Profile::Car => x == "motorcar" || x == "motor_vehicle",
                            Profile::Motorbike => x == "motorcycle" || x == "motor_vehicle",
                            Profile::Foot => false,
                        }
                    })
            })
        };
        for rs in &g.restrictions {
            r.stats.relations += 1;
            if rs.condition.is_some() || rs.key.ends_with(":conditional") {
                r.stats.conditional_skipped += 1;
                continue;
            }
            if !applies_key(&rs.key) {
                r.stats.other_mode_skipped += 1;
                continue;
            }
            if !rs.via_ways.is_empty() {
                r.stats.via_way_skipped += 1;
                continue;
            }
            if rs.from_edges.is_empty() || rs.to_edges.is_empty() {
                r.stats.unresolved_skipped += 1;
                continue;
            }
            if excepts(&rs.tags) {
                r.stats.excepted_skipped += 1;
                continue;
            }
            r.stats.applied += 1;
            if rs.kind.starts_with("only_") {
                r.stats.only_kinds += 1;
                for &f in &rs.from_edges {
                    let list = r.only.entry(f).or_default();
                    for &t in &rs.to_edges {
                        if !list.contains(&t) {
                            list.push(t);
                        }
                    }
                }
            } else if rs.kind.starts_with("no_") {
                r.stats.no_kinds += 1;
                for &f in &rs.from_edges {
                    for &t in &rs.to_edges {
                        r.banned.insert((f, t));
                    }
                }
            }
        }
        r.stats.banned_pairs = r.banned.len();
        r
    }

    #[inline]
    pub fn forbids(&self, from: EdgeId, to: EdgeId) -> bool {
        if self.banned.contains(&(from, to)) {
            return true;
        }
        self.only.get(&from).is_some_and(|ok| !ok.contains(&to))
    }
}

/// Mô hình rẽ của một profile: tham chiếu hình học, cấm rẽ và bảng chi phí.
#[derive(Clone, Copy)]
pub struct TurnModel<'a> {
    pub view: View<'a>,
    pub geom: &'a EdgeGeom,
    pub restr: &'a Restrictions,
    pub costs: TurnCosts,
}

impl TurnModel<'_> {
    pub fn classify(&self, e1: EdgeId, e2: EdgeId) -> TurnKind {
        let g = self.view.g;
        if g.twin[e1 as usize] == e2 {
            return TurnKind::UTurn;
        }
        let d = (self.geom.bear_out[e2 as usize] - self.geom.bear_in[e1 as usize] + 540.0)
            .rem_euclid(360.0)
            - 180.0;
        let a = d.abs();
        if a >= self.costs.reversal_deg {
            TurnKind::UTurn
        } else if a < self.costs.straight_deg {
            TurnKind::Straight
        } else if d > 0.0 {
            TurnKind::Right
        } else {
            TurnKind::Left
        }
    }

    fn dead_end(&self, node: NodeId) -> bool {
        // ngõ cụt: mọi cạnh ra của nút (đi được) đều quay về nút vừa tới, nên không còn đường nào khác
        self.view.out(node).count() <= 1
    }

    /// Chi phí rẽ `e1 → e2` (ms), `None` nếu bị cấm.
    #[inline]
    pub fn turn_ms(&self, e1: EdgeId, e2: EdgeId) -> Option<u32> {
        if self.restr.forbids(e1, e2) {
            return None;
        }
        let kind = self.classify(e1, e2);
        let c = &self.costs;
        let mut cost = match kind {
            TurnKind::Straight => c.straight_ms,
            TurnKind::Right => c.right_ms,
            TurnKind::Left => c.left_ms,
            TurnKind::UTurn => match c.u_turn_policy {
                UTurnPolicy::Free => 0,
                UTurnPolicy::Costed => c.u_turn_ms,
                UTurnPolicy::DeadEndOnly => {
                    if self.dead_end(self.view.g.head[e1 as usize]) {
                        c.u_turn_ms
                    } else {
                        return None;
                    }
                }
            },
        };
        if self.geom.median_cross[e2 as usize] && !self.geom.median_cross[e1 as usize] {
            cost += c.median_ms;
        }
        Some(cost)
    }

    /// Chi phí đầy đủ của một dãy cạnh (trọng số + rẽ). `None` nếu có chuyển trạng thái bị cấm hoặc cạnh không đi được.
    /// Dùng để chấm điểm lại đường do mô hình theo nút tìm ra.
    pub fn path_cost(&self, edges: &[EdgeId]) -> Option<u64> {
        let mut total = 0u64;
        for (i, &e) in edges.iter().enumerate() {
            let w = self.view.w[e as usize];
            if w == INF {
                return None;
            }
            total += u64::from(w);
            if i > 0 {
                total += u64::from(self.turn_ms(edges[i - 1], e)?);
            }
        }
        Some(total)
    }

    /// Như [`TurnModel::path_cost`] nhưng cho tuyến có cạnh đầu và cạnh cuối đi dở (xuất phát/đích giữa cạnh).
    pub fn route_cost(&self, r: &EdgeRoute) -> Option<u64> {
        let n = r.edges.len();
        let mut total = 0u64;
        for (i, &e) in r.edges.iter().enumerate() {
            let w = self.view.w[e as usize];
            if w == INF {
                return None;
            }
            let from = if i == 0 { f64::from(r.start_frac) } else { 0.0 };
            let to = if i + 1 == n {
                f64::from(r.end_frac)
            } else {
                1.0
            };
            total += (f64::from(w) * (to - from).max(0.0)).round() as u64;
            if i > 0 {
                total += u64::from(self.turn_ms(r.edges[i - 1], e)?);
            }
        }
        Some(total)
    }

    /// Đếm số lần rẽ theo loại trên một dãy cạnh.
    pub fn turn_counts(&self, edges: &[EdgeId]) -> [u32; 4] {
        let mut c = [0u32; 4];
        for w in edges.windows(2) {
            let k = self.classify(w[0], w[1]);
            c[k as usize] += 1;
        }
        c
    }

    /// Số lần quay đầu theo định nghĩa hình học của lab: tổng góc rẽ trong một chuỗi các nút liên tiếp cách nhau không quá
    /// 60 m đạt ≥ 150 độ (quay ngược lại). Bắt được cả quay đầu trực tiếp lẫn «rẽ trái hai lần qua dải phân cách».
    /// Chuỗi có cạnh vòng xuyến bị bỏ qua. Đây là xấp xỉ hình học, không phải nhận diện biển báo quay đầu.
    pub fn u_turns(&self, edges: &[EdgeId]) -> u32 {
        self.u_turn_breakdown(edges).iter().sum()
    }

    /// Như [`TurnModel::u_turns`] nhưng chia theo cách quay đầu: `[đi ngược trực tiếp (cạnh twin), qua lối dải phân cách, khác]`.
    /// «Khác» là chuỗi nút gần nhau mà hình học cộng lại ≥ 150° (vd. đường cong gắt, nút giao Y, hẻm gấp khúc).
    pub fn u_turn_breakdown(&self, edges: &[EdgeId]) -> [u32; 3] {
        let g = self.view.g;
        let round = |e: EdgeId| g.flags[e as usize] & flag::ROUNDABOUT != 0;
        let mut count = [0u32; 3];
        let mut i = 1;
        while i < edges.len() {
            // chuỗi bắt đầu ở chỗ rẽ giữa edges[i-1] và edges[i]
            let mut sum = self.signed_turn(edges[i - 1], edges[i]);
            let mut in_roundabout = round(edges[i - 1]) || round(edges[i]);
            let mut j = i;
            while j + 1 < edges.len() && g.length_cm[edges[j] as usize] <= 6_000 {
                sum += self.signed_turn(edges[j], edges[j + 1]);
                in_roundabout |= round(edges[j + 1]);
                j += 1;
            }
            // đi quanh vòng xuyến không phải quay đầu
            if !in_roundabout && sum.abs() >= self.costs.reversal_deg {
                let direct = (i..=j).any(|k| g.twin[edges[k - 1] as usize] == edges[k]);
                let median = (i - 1..=j).any(|k| self.geom.median_cross[edges[k] as usize]);
                count[if direct {
                    0
                } else if median {
                    1
                } else {
                    2
                }] += 1;
                i = j + 1;
            } else {
                i += 1;
            }
        }
        count
    }

    fn signed_turn(&self, e1: EdgeId, e2: EdgeId) -> f32 {
        if self.view.g.twin[e1 as usize] == e2 {
            return 180.0;
        }
        (self.geom.bear_out[e2 as usize] - self.geom.bear_in[e1 as usize] + 540.0).rem_euclid(360.0)
            - 180.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Start {
    /// Xuất phát tại nút, chưa có hướng: được đi bất kỳ cạnh ra nào, không tốn chi phí rẽ.
    Node(NodeId),
    /// Đang ở trên một cạnh có hướng: phải đi hết cạnh theo hướng đó, rồi mới rẽ.
    Pos(Position),
}

#[derive(Debug, Clone, Copy)]
pub enum Goal {
    Node(NodeId),
    /// Một điểm trên cạnh có hướng cụ thể: phải đi vào điểm đó theo chiều cạnh (đúng phía đường).
    Pos(Position),
}

/// Kết quả định tuyến theo cạnh.
#[derive(Debug, Clone)]
pub struct EdgeRoute {
    /// Chi phí tổng gồm cả chi phí rẽ.
    pub cost: u32,
    /// Phần chi phí là chi phí rẽ.
    pub turn_cost: u32,
    pub edges: Vec<EdgeId>,
    /// Cạnh đầu chỉ đi từ tỉ lệ này tới hết (0 nếu xuất phát tại nút).
    pub start_frac: f32,
    /// Cạnh cuối chỉ đi tới tỉ lệ này (1 nếu đích là nút).
    pub end_frac: f32,
}

impl EdgeRoute {
    /// Độ dài thật (cm) có tính phần cạnh đầu và cuối.
    pub fn length_cm(&self, g: &Graph) -> u64 {
        let n = self.edges.len();
        let mut total = 0f64;
        for (i, &e) in self.edges.iter().enumerate() {
            let l = f64::from(g.length_cm[e as usize]);
            let from = if i == 0 {
                f64::from(self.start_frac)
            } else {
                0.0
            };
            let to = if i + 1 == n {
                f64::from(self.end_frac)
            } else {
                1.0
            };
            total += l * (to - from).max(0.0);
        }
        total.round() as u64
    }
}

/// A* theo cạnh. Bộ nhớ làm việc có kích thước số cạnh.
pub struct EdgeSearcher {
    labels: Labels,
    heap: MinHeap,
}

impl EdgeSearcher {
    pub fn new(edge_count: usize) -> Self {
        Self {
            labels: Labels::new(edge_count),
            heap: MinHeap::new(),
        }
    }

    /// Tìm đường theo cạnh. `xy` và `vmax` dùng cho heuristic chim bay (luôn hợp lệ vì chi phí rẽ không âm).
    /// `use_heuristic = false` chạy Dijkstra thuần để đối chiếu.
    pub fn route(
        &mut self,
        tm: &TurnModel,
        xy: &NodeXy,
        vmax_cm_per_unit: f64,
        from: Start,
        to: Goal,
        use_heuristic: bool,
    ) -> (Option<EdgeRoute>, Stats) {
        let g = tm.view.g;
        let w = tm.view.w;
        let mut st = Stats::default();
        self.labels.reset();
        self.heap.clear();

        // heuristic: khoảng cách chim bay tới điểm đích (nút hoặc điểm trên cạnh)
        let target_xy = match to {
            Goal::Node(t) => xy.xy[t as usize],
            Goal::Pos(p) => xy.proj.to_xy(p.point(g)),
        };
        let scale = NodeXy::scale(vmax_cm_per_unit);
        let h = |e: EdgeId| -> u32 {
            if !use_heuristic {
                return 0;
            }
            (xy.dist_to_point_m(g.head[e as usize], target_xy) * scale) as u32
        };

        let mut start_frac = 0.0f32;
        match from {
            Start::Node(s) => {
                if let Goal::Node(t) = to
                    && s == t
                {
                    return (
                        Some(EdgeRoute {
                            cost: 0,
                            turn_cost: 0,
                            edges: Vec::new(),
                            start_frac: 0.0,
                            end_frac: 1.0,
                        }),
                        st,
                    );
                }
                for (e, _, c) in tm.view.out(s) {
                    self.labels.set(e, c, NONE);
                    self.heap.push(c.saturating_add(h(e)), e);
                }
            }
            Start::Pos(p) => {
                let Some(c) = p.cost_to_head(tm.view) else {
                    return (None, st);
                };
                start_frac = p.frac;
                // đích trên đúng cạnh này và ở phía trước: đi thẳng, không cần tìm kiếm
                if let Goal::Pos(q) = to
                    && q.edge == p.edge
                    && q.frac >= p.frac
                {
                    let cost =
                        (f64::from(w[p.edge as usize]) * f64::from(q.frac - p.frac)).round() as u32;
                    return (
                        Some(EdgeRoute {
                            cost,
                            turn_cost: 0,
                            edges: vec![p.edge],
                            start_frac: p.frac,
                            end_frac: q.frac,
                        }),
                        st,
                    );
                }
                self.labels.set(p.edge, c, NONE);
                self.heap.push(c.saturating_add(h(p.edge)), p.edge);
            }
        }

        // tốt nhất đã thấy tới đích: (chi phí, cạnh cuối cùng đã chốt dẫn tới đích)
        let mut best: Option<(u32, EdgeId)> = None;
        while let Some((key, e)) = self.heap.pop() {
            if let Some((b, _)) = best
                && key >= b
            {
                break;
            }
            if self.labels.is_closed(e) {
                continue;
            }
            self.labels.close(e);
            st.settled += 1;
            let d = self.labels.dist(e);
            let v = g.head[e as usize];
            match to {
                Goal::Node(t) => {
                    if v == t {
                        best = Some((d, e));
                        break;
                    }
                }
                Goal::Pos(q) => {
                    if v == g.tail[q.edge as usize]
                        && let Some(tc) = tm.turn_ms(e, q.edge)
                        && let Some(leg) = q.cost_from_tail(tm.view)
                    {
                        let cand = d.saturating_add(tc).saturating_add(leg);
                        if best.is_none_or(|(b, _)| cand < b) {
                            best = Some((cand, e));
                        }
                    }
                }
            }
            for e2 in g.out_edges(v) {
                let c = w[e2 as usize];
                if c == INF {
                    continue;
                }
                let Some(tc) = tm.turn_ms(e, e2) else {
                    continue;
                };
                st.relaxed += 1;
                let nd = d.saturating_add(tc).saturating_add(c);
                if nd < self.labels.dist(e2) {
                    self.labels.set(e2, nd, e);
                    self.heap.push(nd.saturating_add(h(e2)), e2);
                }
            }
        }

        let Some((cost, last)) = best else {
            return (None, st);
        };
        let mut edges = Vec::new();
        let mut cur = last;
        loop {
            edges.push(cur);
            let p = self.labels.pred(cur);
            if p == NONE {
                break;
            }
            cur = p;
        }
        edges.reverse();
        let mut end_frac = 1.0;
        if let Goal::Pos(q) = to {
            edges.push(q.edge);
            end_frac = q.frac;
        }
        let route = EdgeRoute {
            cost,
            turn_cost: 0,
            start_frac,
            end_frac,
            edges,
        };
        let turn_cost = {
            let mut t = 0u32;
            for win in route.edges.windows(2) {
                t = t.saturating_add(tm.turn_ms(win[0], win[1]).unwrap_or(0));
            }
            t
        };
        (Some(EdgeRoute { turn_cost, ..route }), st)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::Rev;
    use crate::profile::{Costs, Metric};
    use crate::search::Searcher;
    use road_graph::fixture::{compile_fixture, generate};

    struct Env {
        g: Graph,
        rev: Rev,
        costs: Costs,
        geom: EdgeGeom,
        xy: NodeXy,
        restr: Restrictions,
    }
    fn env() -> Env {
        let g = compile_fixture(&generate());
        let rev = Rev::build(&g);
        let costs = Costs::build(&g, Profile::Car, Metric::Time);
        let geom = EdgeGeom::build(&g);
        let xy = NodeXy::build(&g);
        let restr = Restrictions::build(&g, Profile::Car);
        Env {
            g,
            rev,
            costs,
            geom,
            xy,
            restr,
        }
    }
    fn model<'a>(env: &'a Env, costs: TurnCosts, restr: &'a Restrictions) -> TurnModel<'a> {
        TurnModel {
            view: View::new(&env.g, &env.rev, &env.costs.w),
            geom: &env.geom,
            restr,
            costs,
        }
    }

    #[test]
    fn fixture_has_three_restrictions_and_a_median_opening() {
        let e = env();
        assert_eq!(e.restr.stats.relations, 3);
        assert!(e.restr.stats.applied >= 2);
        assert!(e.geom.median_cross.iter().filter(|&&b| b).count() >= 4);
    }

    #[test]
    fn no_cost_model_equals_node_dijkstra_when_nothing_is_restricted() {
        let e = env();
        let none = Restrictions::default();
        let tm = model(&e, TurnCosts::none(), &none);
        let mut es = EdgeSearcher::new(e.g.edge_count());
        let mut ns = Searcher::new(e.g.node_count());
        let n = e.g.node_count() as u32;
        let mut seed = 9u64;
        for _ in 0..100 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let a = (seed >> 33) as u32 % n;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let b = (seed >> 33) as u32 % n;
            let (r, _) = es.route(
                &tm,
                &e.xy,
                e.costs.vmax_cm_per_unit,
                Start::Node(a),
                Goal::Node(b),
                true,
            );
            let (d, _) = ns.dijkstra(tm.view, a, b);
            assert_eq!(r.map(|r| r.cost), d.map(|p| p.cost), "{a}->{b}");
        }
    }

    #[test]
    fn astar_and_dijkstra_agree_with_turn_costs() {
        let e = env();
        let tm = model(&e, TurnCosts::urban(), &e.restr);
        let mut es = EdgeSearcher::new(e.g.edge_count());
        let n = e.g.node_count() as u32;
        let mut seed = 21u64;
        for _ in 0..100 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let a = (seed >> 33) as u32 % n;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let b = (seed >> 33) as u32 % n;
            let (x, _) = es.route(
                &tm,
                &e.xy,
                e.costs.vmax_cm_per_unit,
                Start::Node(a),
                Goal::Node(b),
                true,
            );
            let (y, _) = es.route(
                &tm,
                &e.xy,
                e.costs.vmax_cm_per_unit,
                Start::Node(a),
                Goal::Node(b),
                false,
            );
            assert_eq!(x.as_ref().map(|r| r.cost), y.as_ref().map(|r| r.cost));
            if let Some(r) = x {
                assert_eq!(u64::from(r.cost), tm.path_cost(&r.edges).unwrap());
            }
        }
    }

    #[test]
    fn turn_costs_never_make_a_route_cheaper() {
        let e = env();
        let none = Restrictions::default();
        let free = model(&e, TurnCosts::none(), &none);
        let urban = model(&e, TurnCosts::urban(), &e.restr);
        let mut es = EdgeSearcher::new(e.g.edge_count());
        let n = e.g.node_count() as u32;
        for k in 0..60u32 {
            let (a, b) = ((k * 37) % n, (k * 101 + 7) % n);
            let (x, _) = es.route(
                &free,
                &e.xy,
                e.costs.vmax_cm_per_unit,
                Start::Node(a),
                Goal::Node(b),
                true,
            );
            let (y, _) = es.route(
                &urban,
                &e.xy,
                e.costs.vmax_cm_per_unit,
                Start::Node(a),
                Goal::Node(b),
                true,
            );
            if let (Some(x), Some(y)) = (x, y) {
                assert!(y.cost >= x.cost);
            }
        }
    }

    #[test]
    fn route_cost_matches_search_cost_for_position_routes() {
        let e = env();
        let tm = model(&e, TurnCosts::urban(), &e.restr);
        let mut es = EdgeSearcher::new(e.g.edge_count());
        let edges: Vec<u32> = (0..e.g.edge_count() as u32)
            .filter(|&x| e.costs.w[x as usize] != INF)
            .collect();
        let mut seed = 77u64;
        let mut checked = 0;
        for _ in 0..200 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let a = edges[(seed >> 33) as usize % edges.len()];
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let b = edges[(seed >> 33) as usize % edges.len()];
            let (fa, fb) = (
                ((seed >> 20) % 90) as f32 / 100.0 + 0.05,
                ((seed >> 40) % 90) as f32 / 100.0 + 0.05,
            );
            let (r, _) = es.route(
                &tm,
                &e.xy,
                e.costs.vmax_cm_per_unit,
                Start::Pos(Position::new(a, fa)),
                Goal::Pos(Position::new(b, fb)),
                true,
            );
            if let Some(r) = r {
                let c = tm.route_cost(&r).expect("hợp lệ");
                assert!(
                    (c as i64 - i64::from(r.cost)).abs() <= 2,
                    "{c} vs {}",
                    r.cost
                );
                checked += 1;
            }
        }
        assert!(checked > 100);
    }

    #[test]
    fn banned_pair_is_never_used() {
        let e = env();
        let tm = model(&e, TurnCosts::urban(), &e.restr);
        let &(f, t) = e.restr.banned.iter().next().expect("có cặp bị cấm");
        let mut es = EdgeSearcher::new(e.g.edge_count());
        // đi từ điểm đầu cạnh `f` tới cuối cạnh `t`: không được dùng chuyển f → t trực tiếp
        let (r, _) = es.route(
            &tm,
            &e.xy,
            e.costs.vmax_cm_per_unit,
            Start::Pos(Position::new(f, 0.5)),
            Goal::Pos(Position::new(t, 0.5)),
            true,
        );
        if let Some(r) = r {
            assert!(r.edges.windows(2).all(|w| !(w[0] == f && w[1] == t)));
        }
    }

    #[test]
    fn dead_end_only_policy_forbids_midstreet_reversal() {
        let e = env();
        let empty = Restrictions::default();
        let tm = model(&e, TurnCosts::urban(), &empty);
        // cạnh hai chiều có nút cuối còn nhiều đường ra: quay đầu bị cấm
        let f = (0..e.g.edge_count() as u32)
            .find(|&f| {
                e.costs.w[f as usize] != INF
                    && e.g.twin[f as usize] != NONE
                    && e.costs.w[e.g.twin[f as usize] as usize] != INF
                    && tm.view.out(e.g.head[f as usize]).count() > 2
            })
            .unwrap();
        assert_eq!(tm.turn_ms(f, e.g.twin[f as usize]), None);
        let costed = TurnModel {
            costs: TurnCosts {
                u_turn_policy: UTurnPolicy::Costed,
                ..TurnCosts::urban()
            },
            ..tm
        };
        assert_eq!(costed.turn_ms(f, e.g.twin[f as usize]), Some(20_000));
    }
}
