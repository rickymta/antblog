use crate::{DriverId, DriverIndex, Neighbor, push_top_k};
use geo_core::h3::{CellIndex, Resolution, cell_of, edge_length_avg_m, resolution};
use geo_core::{LatLng, LatLngE7, equirect_m};
use rustc_hash::{FxHashMap, FxHashSet};

/// Mỗi ô H3 là một giỏ danh sách tài xế. Đổi ô thì chuyển giỏ; không đổi ô thì chỉ ghi vị trí mới.
pub struct H3Buckets {
    res: Resolution,
    pos: Vec<LatLngE7>,
    cell: Vec<Option<CellIndex>>,
    buckets: FxHashMap<CellIndex, Vec<DriverId>>,
    count: usize,
    /// Số lần cập nhật làm tài xế đổi ô (để lab báo cáo).
    pub cell_changes: u64,
}

const MAX_RING: u32 = 64;

impl H3Buckets {
    pub fn new(res: u8, capacity: usize) -> Self {
        Self {
            res: resolution(res),
            pos: vec![LatLngE7::ABSENT; capacity],
            cell: vec![None; capacity],
            buckets: FxHashMap::default(),
            count: 0,
            cell_changes: 0,
        }
    }

    pub fn resolution(&self) -> u8 {
        u8::from(self.res)
    }

    fn detach(&mut self, id: DriverId, cell: CellIndex) {
        if let Some(v) = self.buckets.get_mut(&cell) {
            if let Some(at) = v.iter().position(|&d| d == id) {
                v.swap_remove(at);
            }
            if v.is_empty() {
                self.buckets.remove(&cell);
            }
        }
    }

    /// Các ô cách đúng `r` bước. `grid_ring_fast` nhanh nhưng có thể trả `None` gần ô ngũ giác; khi đó dùng đĩa trừ đĩa.
    fn ring(c0: CellIndex, r: u32) -> Vec<CellIndex> {
        if r == 0 {
            return vec![c0];
        }
        let fast: Option<Vec<CellIndex>> = c0.grid_ring_fast(r).collect();
        if let Some(v) = fast {
            return v;
        }
        let inner: FxHashSet<CellIndex> = c0.grid_disk::<Vec<_>>(r - 1).into_iter().collect();
        c0.grid_disk::<Vec<_>>(r)
            .into_iter()
            .filter(|c| !inner.contains(c))
            .collect()
    }
}

impl DriverIndex for H3Buckets {
    fn name(&self) -> String {
        format!("ô H3 độ phân giải {}", self.resolution())
    }

    fn upsert(&mut self, id: DriverId, pos: LatLng) {
        let i = id as usize;
        let e7 = pos.to_e7();
        let new_cell = cell_of(e7.to_latlng(), self.res);
        match self.cell[i] {
            Some(old) if old == new_cell => {}
            Some(old) => {
                self.detach(id, old);
                self.buckets.entry(new_cell).or_default().push(id);
                self.cell[i] = Some(new_cell);
                self.cell_changes += 1;
            }
            None => {
                self.buckets.entry(new_cell).or_default().push(id);
                self.cell[i] = Some(new_cell);
                self.count += 1;
            }
        }
        self.pos[i] = e7;
    }

    fn remove(&mut self, id: DriverId) -> bool {
        let i = id as usize;
        let Some(c) = self.cell[i].take() else {
            return false;
        };
        self.detach(id, c);
        self.pos[i] = LatLngE7::ABSENT;
        self.count -= 1;
        true
    }

    fn nearest(&self, q: LatLng, k: usize) -> Vec<Neighbor> {
        let c0 = cell_of(q, self.res);
        let edge = edge_length_avg_m(self.res);
        let mut best: Vec<Neighbor> = Vec::with_capacity(k + 1);
        for r in 0..=MAX_RING {
            for cell in Self::ring(c0, r) {
                if let Some(ids) = self.buckets.get(&cell) {
                    for &id in ids {
                        let d = equirect_m(q, self.pos[id as usize].to_latlng());
                        push_top_k(&mut best, k, Neighbor { id, dist_m: d });
                    }
                }
            }
            // Mọi điểm ở ô cách >= r+1 bước cách điểm hỏi ít nhất (1,5·r − 0,5)·cạnh; nhân 0,8 cho độ méo của lưới.
            let safe = (1.5 * f64::from(r) - 0.5).max(0.0) * edge * 0.8;
            if best.len() == k && best.last().is_some_and(|l| l.dist_m <= safe) {
                break;
            }
            if best.len() == self.count.min(k) && best.len() == self.count {
                break; // đã lấy hết tài xế
            }
        }
        best
    }

    fn len(&self) -> usize {
        self.count
    }
}
