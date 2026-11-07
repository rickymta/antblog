//! Vị trí trên đường và cách nối vào các thuật toán theo nút.
//!
//! Một điểm GPS sau khi snap nằm **trên một cạnh có hướng** (`Position`), không phải trên một nút. Với thuật toán theo nút
//! (CH, ma trận), vị trí được biểu diễn bằng danh sách «chân» (`Leg`): nút đồng thời chi phí đi nốt phần cạnh.
//!
//! - nguồn (đang đi trên cạnh `e`, đã đi được `frac`): ra được qua `head(e)` với chi phí `(1 - frac) * w(e)`
//! - đích (một điểm trên cạnh `e`): tới được qua `tail(e)` với chi phí `frac * w(e)`. Chỉ có **một** chân nghĩa là phải vào
//!   điểm này đúng chiều `e` (đúng phía đường); điểm trên đường hai chiều có hai chân (từ cả hai đầu).

use crate::net::{INF, View};
use geo_core::LatLng;
use road_graph::snap::SnapCandidate;
use road_graph::{EdgeId, Graph, NodeId};

/// Một «chân» nối điểm trên đường với nút của đồ thị.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Leg {
    pub node: NodeId,
    pub cost: u32,
}

/// Điểm trên cạnh có hướng: `frac` là tỉ lệ độ dài đã đi từ nút đầu, trong [0, 1].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub edge: EdgeId,
    pub frac: f32,
}

impl Position {
    pub fn new(edge: EdgeId, frac: f32) -> Self {
        Self {
            edge,
            frac: frac.clamp(0.0, 1.0),
        }
    }
    /// Chi phí từ điểm này tới nút cuối của cạnh. `None` nếu profile không đi được cạnh.
    pub fn cost_to_head(&self, view: View) -> Option<u32> {
        let w = view.w[self.edge as usize];
        (w != INF).then(|| scaled(w, 1.0 - self.frac))
    }
    pub fn cost_from_tail(&self, view: View) -> Option<u32> {
        let w = view.w[self.edge as usize];
        (w != INF).then(|| scaled(w, self.frac))
    }
    /// Chân nguồn: đi tiếp theo chiều cạnh.
    pub fn source_legs(&self, view: View) -> Vec<Leg> {
        self.cost_to_head(view)
            .map(|cost| Leg {
                node: view.g.head[self.edge as usize],
                cost,
            })
            .into_iter()
            .collect()
    }
    /// Chân đích: phải tới điểm này bằng cách đi trên chính cạnh này.
    pub fn target_legs(&self, view: View) -> Vec<Leg> {
        self.cost_from_tail(view)
            .map(|cost| Leg {
                node: view.g.tail[self.edge as usize],
                cost,
            })
            .into_iter()
            .collect()
    }
    pub fn point(&self, g: &Graph) -> LatLng {
        point_on_edge(g, self.edge, self.frac)
    }
}

fn scaled(w: u32, f: f32) -> u32 {
    (f64::from(w) * f64::from(f)).round() as u32
}

/// Toạ độ điểm ở tỉ lệ `frac` dọc cạnh (theo độ dài hình học).
pub fn point_on_edge(g: &Graph, e: EdgeId, frac: f32) -> LatLng {
    let pts = g.edge_points(e);
    let len: f64 = pts
        .windows(2)
        .map(|w| geo_core::haversine_m(w[0], w[1]))
        .sum();
    geo_core::segment::point_at(&pts, len * f64::from(frac.clamp(0.0, 1.0))).unwrap_or(pts[0])
}

/// Điểm GPS đã snap lên đường của một profile: đường hai chiều cho cả hai hướng, đường một chiều chỉ một hướng.
#[derive(Debug, Clone, Copy)]
pub struct Snapped {
    pub edge: EdgeId,
    pub twin: Option<EdgeId>,
    /// Tỉ lệ dọc theo `edge` (không phải `twin`).
    pub frac: f32,
    pub point: LatLng,
    pub snap_dist_m: f64,
}

impl Snapped {
    pub fn from_candidate(g: &Graph, c: &SnapCandidate) -> Self {
        let len_m = f64::from(g.length_cm[c.edge as usize]) / 100.0;
        Self {
            edge: c.edge,
            twin: c.twin,
            frac: if len_m > 0.0 {
                (c.along_m / len_m).clamp(0.0, 1.0) as f32
            } else {
                0.0
            },
            point: c.point,
            snap_dist_m: c.dist_m,
        }
    }
    /// Các vị trí có hướng mà profile đi được tại điểm này (một hoặc hai).
    pub fn positions(&self, view: View) -> Vec<Position> {
        let mut v = Vec::with_capacity(2);
        if view.w[self.edge as usize] != INF {
            v.push(Position::new(self.edge, self.frac));
        }
        if let Some(t) = self.twin
            && view.w[t as usize] != INF
        {
            v.push(Position::new(t, 1.0 - self.frac));
        }
        v
    }
    pub fn source_legs(&self, view: View) -> Vec<Leg> {
        self.positions(view)
            .iter()
            .flat_map(|p| p.source_legs(view))
            .collect()
    }
    pub fn target_legs(&self, view: View) -> Vec<Leg> {
        self.positions(view)
            .iter()
            .flat_map(|p| p.target_legs(view))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::Rev;
    use crate::profile::{Costs, Metric, Profile};
    use road_graph::fixture::{compile_fixture, generate};

    #[test]
    fn legs_split_the_edge_cost() {
        let g = compile_fixture(&generate());
        let rev = Rev::build(&g);
        let c = Costs::build(&g, Profile::Car, Metric::Time);
        let view = View::new(&g, &rev, &c.w);
        let e = (0..g.edge_count() as u32)
            .find(|&e| c.w[e as usize] != INF)
            .unwrap();
        let p = Position::new(e, 0.25);
        let (a, b) = (
            p.cost_from_tail(view).unwrap(),
            p.cost_to_head(view).unwrap(),
        );
        assert!((i64::from(a + b) - i64::from(c.w[e as usize])).abs() <= 1);
        assert_eq!(p.source_legs(view)[0].node, g.head[e as usize]);
        assert_eq!(p.target_legs(view)[0].node, g.tail[e as usize]);
    }

    #[test]
    fn one_way_point_has_a_single_direction_for_car() {
        let g = compile_fixture(&generate());
        let rev = Rev::build(&g);
        let c = Costs::build(&g, Profile::Car, Metric::Time);
        let view = View::new(&g, &rev, &c.w);
        let e = (0..g.edge_count() as u32)
            .find(|&e| {
                c.w[e as usize] != INF
                    && g.twin[e as usize] != u32::MAX
                    && c.w[g.twin[e as usize] as usize] == INF
            })
            .expect("fixture có đường một chiều cho ô tô");
        let s = Snapped {
            edge: e,
            twin: Some(g.twin[e as usize]),
            frac: 0.5,
            point: g.node_latlng(g.tail[e as usize]),
            snap_dist_m: 0.0,
        };
        assert_eq!(s.positions(view).len(), 1);
        assert_eq!(s.target_legs(view).len(), 1);
    }
}
