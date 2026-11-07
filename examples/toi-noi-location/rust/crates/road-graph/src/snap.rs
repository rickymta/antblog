//! Chỉ mục R-tree các đoạn cạnh và snap điểm GPS về cạnh gần nhất.
//!
//! Mỗi cạnh được tách thành các đoạn thẳng giữa hai điểm liên tiếp (nút, điểm hình dạng, nút). Cặp cạnh hai chiều
//! (`twin`) chỉ được lập chỉ mục một lần; ứng viên trả về cạnh đại diện và `twin` để người gọi chọn chiều theo hướng đi.
//! Khoảng cách trong cây tính trên mặt phẳng mét của [`LocalProjection`] (sai số dưới 0,1% trong bbox thành phố),
//! còn kết quả cuối được tính lại chính xác bằng `geo_core::project_to_segment`.

use crate::model::{EdgeId, Graph, Mode, NONE};
use geo_core::{LatLng, LatLngE7, LocalProjection, bearing_deg, haversine_m, project_to_segment};
use rstar::{AABB, PointDistance, RTree, RTreeObject};

/// Một đoạn thẳng của một cạnh, trong mặt phẳng mét.
#[derive(Debug, Clone, Copy)]
pub struct Seg {
    a: [f64; 2],
    b: [f64; 2],
    a_ll: LatLngE7,
    b_ll: LatLngE7,
    pub edge: EdgeId,
    pub index: u32,
    pub modes: u8,
}

fn dist2_point_segment(p: &[f64; 2], a: &[f64; 2], b: &[f64; 2]) -> f64 {
    let d = [b[0] - a[0], b[1] - a[1]];
    let len2 = d[0] * d[0] + d[1] * d[1];
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / len2).clamp(0.0, 1.0)
    };
    let q = [a[0] + t * d[0], a[1] + t * d[1]];
    (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)
}

impl RTreeObject for Seg {
    type Envelope = AABB<[f64; 2]>;
    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(self.a, self.b)
    }
}

impl PointDistance for Seg {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        dist2_point_segment(point, &self.a, &self.b)
    }
}

/// Kết quả snap một điểm lên một cạnh.
#[derive(Debug, Clone, Copy)]
pub struct SnapCandidate {
    /// Cạnh đại diện (chiều của cạnh có chỉ số nhỏ hơn trong cặp twin).
    pub edge: EdgeId,
    /// Cạnh chiều ngược lại nếu có.
    pub twin: Option<EdgeId>,
    /// Đoạn thứ mấy trong cạnh.
    pub segment: u32,
    pub t: f64,
    pub point: LatLng,
    pub dist_m: f64,
    /// Từ nút đầu của `edge` tới chân đường vuông góc, mét.
    pub along_m: f64,
    /// Hướng của đoạn theo chiều `edge`, độ. Chiều `twin` lệch 180.
    pub bearing_deg: f64,
    /// Hợp các profile đi được ở một trong hai chiều.
    pub modes: u8,
}

pub struct SnapIndex {
    proj: LocalProjection,
    tree: RTree<Seg>,
    segs: Vec<Seg>,
}

impl SnapIndex {
    /// Dựng chỉ mục (bulk load) cho mọi đoạn của đồ thị.
    pub fn build(g: &Graph) -> Self {
        let proj = LocalProjection::new(g.center());
        let mut segs = Vec::new();
        for e in 0..g.edge_count() as u32 {
            let twin = g.twin[e as usize];
            if twin != NONE && twin < e {
                continue; // đã lập chỉ mục qua cạnh twin có chỉ số nhỏ hơn
            }
            let modes = g.modes[e as usize]
                | if twin != NONE {
                    g.modes[twin as usize]
                } else {
                    0
                };
            let (s, l) = (
                g.shape_start[e as usize] as usize,
                g.shape_len[e as usize] as usize,
            );
            let mut pts: Vec<LatLngE7> = Vec::with_capacity(l + 2);
            pts.push(g.node_ll[g.tail[e as usize] as usize]);
            if g.flags[e as usize] & crate::model::flag::REVERSED != 0 {
                pts.extend(g.shape[s..s + l].iter().rev());
            } else {
                pts.extend(&g.shape[s..s + l]);
            }
            pts.push(g.node_ll[g.head[e as usize] as usize]);
            for (i, w) in pts.windows(2).enumerate() {
                segs.push(Seg {
                    a: proj.to_xy(w[0].to_latlng()),
                    b: proj.to_xy(w[1].to_latlng()),
                    a_ll: w[0],
                    b_ll: w[1],
                    edge: e,
                    index: i as u32,
                    modes,
                });
            }
        }
        let tree = RTree::bulk_load(segs.clone());
        Self { proj, tree, segs }
    }

    pub fn segment_count(&self) -> usize {
        self.segs.len()
    }

    /// Khoảng cách chính xác từ `p` tới polyline của cạnh `e` (không dùng chỉ mục).
    pub fn build_one_distance(g: &Graph, e: EdgeId, p: LatLng) -> f64 {
        geo_core::project_to_polyline(p, &g.edge_points(e)).map_or(f64::INFINITY, |r| r.dist_m)
    }

    fn make_candidate(&self, g: &Graph, s: &Seg, p: LatLng) -> SnapCandidate {
        let (a, b) = (s.a_ll.to_latlng(), s.b_ll.to_latlng());
        let sp = project_to_segment(p, a, b);
        let pts = g.edge_points(s.edge);
        let before: f64 = pts[..=s.index as usize]
            .windows(2)
            .map(|w| haversine_m(w[0], w[1]))
            .sum();
        let twin = g.twin[s.edge as usize];
        SnapCandidate {
            edge: s.edge,
            twin: (twin != NONE).then_some(twin),
            segment: s.index,
            t: sp.t,
            point: sp.point,
            dist_m: sp.dist_m,
            along_m: before + sp.t * haversine_m(a, b),
            bearing_deg: bearing_deg(a, b),
            modes: s.modes,
        }
    }

    /// Cạnh gần nhất mà profile `mode` đi được (ít nhất một chiều), dùng duyệt láng giềng của R-tree.
    pub fn nearest(&self, g: &Graph, p: LatLng, mode: Mode) -> Option<SnapCandidate> {
        let xy = self.proj.to_xy(p);
        let s = self
            .tree
            .nearest_neighbor_iter(xy)
            .find(|s| s.modes & mode.bit() != 0)?;
        Some(self.make_candidate(g, s, p))
    }

    /// Cùng kết quả bằng cách quét toàn bộ đoạn, để đo R-tree nhanh hơn bao nhiêu (cùng công thức khoảng cách).
    pub fn nearest_scan(&self, g: &Graph, p: LatLng, mode: Mode) -> Option<SnapCandidate> {
        let xy = self.proj.to_xy(p);
        let bit = mode.bit();
        let mut best: Option<(f64, &Seg)> = None;
        for s in &self.segs {
            if s.modes & bit == 0 {
                continue;
            }
            let d2 = dist2_point_segment(&xy, &s.a, &s.b);
            if best.is_none_or(|(b, _)| d2 < b) {
                best = Some((d2, s));
            }
        }
        best.map(|(_, s)| self.make_candidate(g, s, p))
    }

    /// Mọi cạnh trong bán kính `radius_m` (mỗi cạnh một ứng viên tốt nhất), gần nhất trước, tối đa `limit`.
    pub fn candidates(
        &self,
        g: &Graph,
        p: LatLng,
        radius_m: f64,
        mode: Mode,
        limit: usize,
    ) -> Vec<SnapCandidate> {
        let xy = self.proj.to_xy(p);
        let r = radius_m * 1.002;
        let mut best: std::collections::BTreeMap<EdgeId, SnapCandidate> =
            std::collections::BTreeMap::new();
        for s in self.tree.locate_within_distance(xy, r * r) {
            if s.modes & mode.bit() == 0 {
                continue;
            }
            let c = self.make_candidate(g, s, p);
            if c.dist_m > radius_m {
                continue;
            }
            match best.get(&c.edge) {
                Some(old) if old.dist_m <= c.dist_m => {}
                _ => {
                    best.insert(c.edge, c);
                }
            }
        }
        let mut v: Vec<SnapCandidate> = best.into_values().collect();
        v.sort_by(|a, b| a.dist_m.total_cmp(&b.dist_m).then(a.edge.cmp(&b.edge)));
        v.truncate(limit);
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{compile_fixture, generate};
    use geo_core::destination;

    fn setup() -> (Graph, SnapIndex) {
        let g = compile_fixture(&generate());
        let idx = SnapIndex::build(&g);
        (g, idx)
    }

    fn lcg(seed: &mut u64) -> f64 {
        *seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (*seed >> 11) as f64 / (1u64 << 53) as f64
    }

    fn random_near(g: &Graph, seed: &mut u64) -> LatLng {
        let n = g.node_latlng((lcg(seed) * g.node_count() as f64) as u32 % g.node_count() as u32);
        destination(n, lcg(seed) * 360.0, lcg(seed) * 60.0)
    }

    #[test]
    fn rtree_equals_scan_for_every_mode() {
        let (g, idx) = setup();
        let mut seed = 42;
        for mode in Mode::ALL {
            for _ in 0..500 {
                let q = random_near(&g, &mut seed);
                let a = idx.nearest(&g, q, mode).unwrap();
                let b = idx.nearest_scan(&g, q, mode).unwrap();
                assert!(
                    (a.dist_m - b.dist_m).abs() < 0.01,
                    "{mode:?}: {} vs {}",
                    a.dist_m,
                    b.dist_m
                );
            }
        }
    }

    #[test]
    fn point_on_edge_snaps_to_it() {
        let (g, idx) = setup();
        let e = g
            .edges()
            .find(|e| e.allows(Mode::Car) && e.length_m() > 50)
            .unwrap();
        let pts = g.edge_points(e.id);
        let mid = geo_core::segment::point_at(&pts, f64::from(e.length_cm()) / 200.0).unwrap();
        let c = idx.nearest(&g, mid, Mode::Car).unwrap();
        assert!(c.dist_m < 0.5, "{}", c.dist_m);
        assert_eq!(g.way_id[c.edge as usize], e.way_id());
    }

    #[test]
    fn candidates_within_radius_sorted_and_include_nearest() {
        let (g, idx) = setup();
        let mut seed = 7;
        for _ in 0..200 {
            let q = random_near(&g, &mut seed);
            let near = idx.nearest(&g, q, Mode::Motorbike).unwrap();
            let cs = idx.candidates(&g, q, near.dist_m + 25.0, Mode::Motorbike, usize::MAX);
            assert!(cs.iter().all(|c| c.dist_m <= near.dist_m + 25.0 + 1e-6));
            assert!(cs.windows(2).all(|w| w[0].dist_m <= w[1].dist_m));
            assert!(cs.iter().any(|c| c.edge == near.edge));
            let mut edges: Vec<_> = cs.iter().map(|c| c.edge).collect();
            edges.sort_unstable();
            edges.dedup();
            assert_eq!(edges.len(), cs.len(), "mỗi cạnh một ứng viên");
        }
    }

    #[test]
    fn car_cannot_snap_to_footbridge_but_foot_can() {
        let (g, idx) = setup();
        let foot_only = g
            .edges()
            .find(|e| {
                e.has(crate::model::flag::BRIDGE)
                    && !e.allows(Mode::Car)
                    && !e.allows(Mode::Motorbike)
            })
            .unwrap();
        let pts = g.edge_points(foot_only.id);
        let mid =
            geo_core::segment::point_at(&pts, f64::from(foot_only.length_cm()) / 200.0).unwrap();
        let car = idx.nearest(&g, mid, Mode::Car).unwrap();
        let foot = idx.nearest(&g, mid, Mode::Foot).unwrap();
        assert!(foot.dist_m < 0.5);
        assert!(car.dist_m > 1.0 || g.way_id[car.edge as usize] != foot_only.way_id());
    }
}
