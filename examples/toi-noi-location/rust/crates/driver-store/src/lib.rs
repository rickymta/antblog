//! `driver-store`: bốn cách giữ vị trí tài xế và tìm k tài xế gần nhất (LOC-09).
//!
//! | Cách | Kiểu | Sửa vị trí | Truy vấn |
//! |---|---|---|---|
//! | quét toàn bộ | [`FullScan`] | gán một ô nhớ | duyệt mọi tài xế |
//! | R-tree `rstar` | [`RTreeIndex`], [`RTreeBulk`] | xóa rồi chèn / dựng lại theo lô | duyệt láng giềng gần nhất |
//! | ô H3 | [`H3Buckets`] | chuyển giỏ khi đổi ô | mở rộng vòng ô quanh điểm hỏi |
//! | Redis GEO | [`redis_geo::RedisGeo`] (feature `redis`) | `GEOADD` | `GEOSEARCH` |
//!
//! Ba cách trong bộ nhớ cùng implement [`DriverIndex`]. Redis có API riêng vì có lỗi mạng và cần `&mut` kết nối.
//! Mọi cách giữ vị trí dạng `LatLngE7` (8 byte) nên cùng một dữ liệu, cùng một khoảng cách `equirect_m`.

mod h3_buckets;
mod rtree;
mod scan;

#[cfg(feature = "redis")]
pub mod redis_geo;

pub use h3_buckets::H3Buckets;
pub use rtree::{RTreeBulk, RTreeIndex};
pub use scan::FullScan;

use geo_core::LatLng;

/// Mã tài xế dạng số nguyên dày đặc (`DR-001` thành 1...). Chỉ mục mảng dùng trực tiếp.
pub type DriverId = u32;

/// Một tài xế trong kết quả, cách điểm hỏi `dist_m` mét (equirectangular).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Neighbor {
    pub id: DriverId,
    pub dist_m: f64,
}

/// Giao diện chung của các chỉ mục trong bộ nhớ.
pub trait DriverIndex {
    /// Tên hiển thị trong bảng đo.
    fn name(&self) -> String;
    /// Thêm hoặc cập nhật vị trí.
    fn upsert(&mut self, id: DriverId, pos: LatLng);
    /// Xóa tài xế (offline). Trả `true` nếu có.
    fn remove(&mut self, id: DriverId) -> bool;
    /// `k` tài xế gần `q` nhất, gần trước.
    fn nearest(&self, q: LatLng, k: usize) -> Vec<Neighbor>;
    /// Áp dụng các thay đổi còn treo (chỉ [`RTreeBulk`] dùng); các cách khác không làm gì.
    fn flush(&mut self) {}
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Chèn `n` vào danh sách đã sắp theo khoảng cách và giữ tối đa `k` phần tử.
pub(crate) fn push_top_k(best: &mut Vec<Neighbor>, k: usize, n: Neighbor) {
    if best.len() == k && best.last().is_some_and(|l| n.dist_m >= l.dist_m) {
        return;
    }
    let at = best.partition_point(|b| b.dist_m <= n.dist_m);
    best.insert(at, n);
    best.truncate(k);
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_core::{LatLng, LatLngE7, equirect_m};

    pub(crate) fn random_points(n: usize, seed: u64) -> Vec<LatLng> {
        let mut rng = fastrand_like(seed);
        (0..n)
            .map(|_| {
                let lat = 10.70 + 0.16 * rng();
                let lng = 106.60 + 0.18 * rng();
                LatLng::try_new(lat, lng).unwrap().to_e7().to_latlng()
            })
            .collect()
    }

    /// Bộ sinh số đơn giản (xorshift) để test không cần thêm thư viện.
    pub(crate) fn fastrand_like(seed: u64) -> impl FnMut() -> f64 {
        let mut s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        move || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            (s >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    fn brute(points: &[LatLng], q: LatLng, k: usize) -> Vec<f64> {
        let mut d: Vec<f64> = points.iter().map(|&p| equirect_m(q, p)).collect();
        d.sort_by(f64::total_cmp);
        d.truncate(k);
        d
    }

    fn check_matches_brute(mut idx: impl DriverIndex) {
        let pts = random_points(3000, 7);
        for (i, p) in pts.iter().enumerate() {
            idx.upsert(i as u32, *p);
        }
        idx.flush();
        assert_eq!(idx.len(), 3000);
        for q in random_points(200, 99) {
            let got: Vec<f64> = idx.nearest(q, 10).iter().map(|n| n.dist_m).collect();
            let want = brute(&pts, q, 10);
            assert_eq!(got.len(), want.len());
            for (g, w) in got.iter().zip(&want) {
                assert!((g - w).abs() < 0.5, "{}: {g} vs {w}", idx.name());
            }
        }
    }

    fn check_update_and_remove(mut idx: impl DriverIndex) {
        let a = LatLng::try_new(10.77, 106.70).unwrap();
        let b = LatLng::try_new(10.78, 106.71).unwrap();
        idx.upsert(1, a);
        idx.upsert(2, b);
        idx.flush();
        assert_eq!(idx.nearest(a, 1)[0].id, 1);
        idx.upsert(1, LatLng::try_new(10.85, 106.77).unwrap()); // tài xế 1 đi xa
        idx.flush();
        assert_eq!(idx.nearest(a, 1)[0].id, 2);
        assert_eq!(idx.len(), 2);
        assert!(idx.remove(2));
        assert!(!idx.remove(2));
        idx.flush();
        assert_eq!(idx.len(), 1);
        assert_eq!(idx.nearest(a, 5).len(), 1);
        let _ = LatLngE7::ABSENT;
    }

    #[test]
    fn scan_matches_brute() {
        check_matches_brute(FullScan::new(4000));
    }
    #[test]
    fn rtree_matches_brute() {
        check_matches_brute(RTreeIndex::new(center(), 4000));
    }
    #[test]
    fn rtree_bulk_matches_brute() {
        check_matches_brute(RTreeBulk::new(center(), 4000));
    }
    #[test]
    fn h3_matches_brute_res8_and_res9() {
        check_matches_brute(H3Buckets::new(8, 4000));
        check_matches_brute(H3Buckets::new(9, 4000));
    }

    #[test]
    fn update_remove_all_variants() {
        check_update_and_remove(FullScan::new(10));
        check_update_and_remove(RTreeIndex::new(center(), 10));
        check_update_and_remove(RTreeBulk::new(center(), 10));
        check_update_and_remove(H3Buckets::new(8, 10));
    }

    #[test]
    fn k_larger_than_population() {
        let mut idx = H3Buckets::new(9, 4);
        idx.upsert(0, LatLng::try_new(10.77, 106.70).unwrap());
        idx.upsert(3, LatLng::try_new(10.80, 106.75).unwrap());
        assert_eq!(
            idx.nearest(LatLng::try_new(10.77, 106.70).unwrap(), 10)
                .len(),
            2
        );
    }

    fn center() -> LatLng {
        LatLng::try_new(10.78, 106.69).unwrap()
    }
}
