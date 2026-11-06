use crate::{DriverId, DriverIndex, Neighbor, push_top_k};
use geo_core::{LatLng, LatLngE7, LocalProjection, equirect_m};
use rstar::{RTree, primitives::GeomWithData};

type Entry = GeomWithData<[f64; 2], DriverId>;

/// R-tree trên mặt phẳng mét của thành phố. Sửa vị trí là xóa rồi chèn (mỗi lần O(log n) nhưng hằng số lớn).
pub struct RTreeIndex {
    proj: LocalProjection,
    tree: RTree<Entry>,
    pos: Vec<LatLngE7>,
    xy: Vec<[f64; 2]>,
}

impl RTreeIndex {
    pub fn new(city_center: LatLng, capacity: usize) -> Self {
        Self {
            proj: LocalProjection::new(city_center),
            tree: RTree::new(),
            pos: vec![LatLngE7::ABSENT; capacity],
            xy: vec![[f64::NAN; 2]; capacity],
        }
    }
}

fn query(
    tree: &RTree<Entry>,
    proj: &LocalProjection,
    pos: &[LatLngE7],
    q: LatLng,
    k: usize,
) -> Vec<Neighbor> {
    let mut out = Vec::with_capacity(k);
    for e in tree.nearest_neighbor_iter(proj.to_xy(q)).take(k) {
        let n = Neighbor {
            id: e.data,
            dist_m: equirect_m(q, pos[e.data as usize].to_latlng()),
        };
        push_top_k(&mut out, k, n);
    }
    out
}

impl DriverIndex for RTreeIndex {
    fn name(&self) -> String {
        "R-tree (xóa + chèn)".into()
    }

    fn upsert(&mut self, id: DriverId, pos: LatLng) {
        let i = id as usize;
        if !self.pos[i].is_absent() {
            self.tree.remove(&Entry::new(self.xy[i], id));
        }
        let e7 = pos.to_e7();
        self.pos[i] = e7;
        self.xy[i] = self.proj.to_xy(e7.to_latlng());
        self.tree.insert(Entry::new(self.xy[i], id));
    }

    fn remove(&mut self, id: DriverId) -> bool {
        let i = id as usize;
        if self.pos[i].is_absent() {
            return false;
        }
        self.tree.remove(&Entry::new(self.xy[i], id));
        self.pos[i] = LatLngE7::ABSENT;
        true
    }

    fn nearest(&self, q: LatLng, k: usize) -> Vec<Neighbor> {
        query(&self.tree, &self.proj, &self.pos, q, k)
    }

    fn len(&self) -> usize {
        self.tree.size()
    }
}

/// R-tree dựng lại theo lô: `upsert` chỉ ghi vị trí, `flush` dựng lại cây bằng `bulk_load`.
/// Truy vấn giữa hai lần `flush` thấy trạng thái cũ (chấp nhận được khi vị trí vốn cũ vài giây).
pub struct RTreeBulk {
    proj: LocalProjection,
    tree: RTree<Entry>,
    pos: Vec<LatLngE7>,
}

impl RTreeBulk {
    pub fn new(city_center: LatLng, capacity: usize) -> Self {
        Self {
            proj: LocalProjection::new(city_center),
            tree: RTree::new(),
            pos: vec![LatLngE7::ABSENT; capacity],
        }
    }
}

impl DriverIndex for RTreeBulk {
    fn name(&self) -> String {
        "R-tree (dựng lại theo lô)".into()
    }

    fn upsert(&mut self, id: DriverId, pos: LatLng) {
        self.pos[id as usize] = pos.to_e7();
    }

    fn remove(&mut self, id: DriverId) -> bool {
        let had = !self.pos[id as usize].is_absent();
        self.pos[id as usize] = LatLngE7::ABSENT;
        had
    }

    fn flush(&mut self) {
        let entries: Vec<Entry> = self
            .pos
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.is_absent())
            .map(|(id, p)| Entry::new(self.proj.to_xy(p.to_latlng()), id as DriverId))
            .collect();
        self.tree = RTree::bulk_load(entries);
    }

    fn nearest(&self, q: LatLng, k: usize) -> Vec<Neighbor> {
        query(&self.tree, &self.proj, &self.pos, q, k)
    }

    fn len(&self) -> usize {
        self.pos.iter().filter(|p| !p.is_absent()).count()
    }
}
