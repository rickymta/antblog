use crate::{DriverId, DriverIndex, Neighbor, push_top_k};
use geo_core::{EARTH_RADIUS_M, LatLng, LatLngE7, equirect_m};

/// Mảng phẳng chỉ số theo `DriverId`; truy vấn duyệt mọi phần tử. Cập nhật O(1).
pub struct FullScan {
    pos: Vec<LatLngE7>,
    count: usize,
}

impl FullScan {
    /// `capacity` là `DriverId` lớn nhất cộng một.
    pub fn new(capacity: usize) -> Self {
        Self {
            pos: vec![LatLngE7::ABSENT; capacity],
            count: 0,
        }
    }
}

impl DriverIndex for FullScan {
    fn name(&self) -> String {
        "quét toàn bộ".into()
    }

    fn upsert(&mut self, id: DriverId, pos: LatLng) {
        let slot = &mut self.pos[id as usize];
        if slot.is_absent() {
            self.count += 1;
        }
        *slot = pos.to_e7();
    }

    fn remove(&mut self, id: DriverId) -> bool {
        let slot = &mut self.pos[id as usize];
        let had = !slot.is_absent();
        *slot = LatLngE7::ABSENT;
        if had {
            self.count -= 1;
        }
        had
    }

    fn nearest(&self, q: LatLng, k: usize) -> Vec<Neighbor> {
        // Xếp hạng bằng bình phương khoảng cách phẳng với cos(vĩ độ) cố định tại điểm hỏi; chỉ tính
        // `equirect_m` đầy đủ cho k kết quả cuối.
        let cos = q.lat().to_radians().cos();
        let (qlat, qlng) = (q.to_e7().lat_e7(), q.to_e7().lng_e7());
        let e7_to_m = EARTH_RADIUS_M * std::f64::consts::PI / 180.0 / 1e7;
        let mut best: Vec<(f64, DriverId)> = Vec::with_capacity(k + 1);
        for (id, p) in self.pos.iter().enumerate() {
            if p.is_absent() {
                continue;
            }
            let dy = f64::from(p.lat_e7() - qlat) * e7_to_m;
            let dx = f64::from(p.lng_e7() - qlng) * e7_to_m * cos;
            let d2 = dx * dx + dy * dy;
            if best.len() == k && best.last().is_some_and(|&(w, _)| d2 >= w) {
                continue;
            }
            let at = best.partition_point(|&(w, _)| w <= d2);
            best.insert(at, (d2, id as DriverId));
            best.truncate(k);
        }
        let mut out = Vec::with_capacity(best.len());
        for (_, id) in best {
            push_top_k(
                &mut out,
                k,
                Neighbor {
                    id,
                    dist_m: equirect_m(q, self.pos[id as usize].to_latlng()),
                },
            );
        }
        out
    }

    fn len(&self) -> usize {
        self.count
    }
}
