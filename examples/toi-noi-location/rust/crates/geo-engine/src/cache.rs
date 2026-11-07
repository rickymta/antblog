//! Cache kết quả tuyến theo ô H3 của điểm đi, ô H3 của điểm đến, profile và khung giờ.
//!
//! Hai yêu cầu cùng cặp ô và cùng khung giờ dùng chung một đáp án, nên đáp án lệch cỡ kích thước ô (H3 độ phân giải 9: cạnh
//! ô cỡ 200 m, h3o báo 201 m). Đáp án trong cache là của yêu cầu **đến trước** điền vào cặp ô đó (phụ thuộc thứ tự đến), không phải tính
//! tại tâm ô. Cặp có điểm đi và điểm đến cùng một ô không dùng cache vì sai số có thể lớn hơn quãng đường.

use geo_core::LatLng;
use geo_core::h3::{cell_of, resolution};
use rustc_hash::FxHashMap;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub from_cell: u64,
    pub to_cell: u64,
    pub profile: u8,
    /// `sec_of_day / time_bucket_s`, hoặc `u16::MAX` nếu yêu cầu không có giờ khởi hành.
    pub bucket: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CachedRoute {
    pub duration_ms: u32,
    pub distance_m: u32,
}

struct Shard {
    map: FxHashMap<CacheKey, CachedRoute>,
    order: VecDeque<CacheKey>,
}

pub struct RouteCache {
    shards: Vec<Mutex<Shard>>,
    cap_per_shard: usize,
    res: u8,
    bucket_s: u32,
    hits: AtomicU64,
    misses: AtomicU64,
    bypass: AtomicU64,
    evictions: AtomicU64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub bypass: u64,
    pub evictions: u64,
    pub entries: usize,
}

impl RouteCache {
    pub fn new(capacity: usize, h3_res: u8, bucket_s: u32) -> Self {
        let n = 16;
        Self {
            shards: (0..n)
                .map(|_| {
                    Mutex::new(Shard {
                        map: FxHashMap::default(),
                        order: VecDeque::new(),
                    })
                })
                .collect(),
            cap_per_shard: (capacity / n).max(1),
            res: h3_res,
            bucket_s: bucket_s.max(1),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            bypass: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
        }
    }

    /// Khoá cache, hoặc `None` nếu cặp điểm không nên dùng cache (cùng một ô).
    pub fn key(
        &self,
        from: LatLng,
        to: LatLng,
        profile: u8,
        depart_sec: Option<u32>,
    ) -> Option<CacheKey> {
        let r = resolution(self.res);
        let (a, b) = (u64::from(cell_of(from, r)), u64::from(cell_of(to, r)));
        if a == b {
            self.bypass.fetch_add(1, Ordering::Relaxed);
            return None;
        }
        Some(CacheKey {
            from_cell: a,
            to_cell: b,
            profile,
            bucket: depart_sec.map_or(u16::MAX, |s| ((s % 86_400) / self.bucket_s) as u16),
        })
    }

    fn shard(&self, k: &CacheKey) -> &Mutex<Shard> {
        let h = k
            .from_cell
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .rotate_left(17)
            ^ k.to_cell.wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
            ^ (u64::from(k.profile) << 48 | u64::from(k.bucket));
        &self.shards[(h >> 20) as usize % self.shards.len()]
    }

    pub fn get(&self, k: &CacheKey) -> Option<CachedRoute> {
        let v = self
            .shard(k)
            .lock()
            .expect("khoá không bị hỏng")
            .map
            .get(k)
            .copied();
        if v.is_some() {
            self.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
        }
        v
    }

    pub fn put(&self, k: CacheKey, v: CachedRoute) {
        let mut s = self.shard(&k).lock().expect("khoá không bị hỏng");
        if s.map.insert(k, v).is_none() {
            s.order.push_back(k);
            while s.map.len() > self.cap_per_shard {
                if let Some(old) = s.order.pop_front() {
                    s.map.remove(&old);
                    self.evictions.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }

    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            bypass: self.bypass.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            entries: self
                .shards
                .iter()
                .map(|s| s.lock().expect("khoá không bị hỏng").map.len())
                .sum(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ll(lat: f64, lng: f64) -> LatLng {
        LatLng::try_new(lat, lng).unwrap()
    }

    #[test]
    fn nearby_points_share_a_key_and_far_ones_do_not() {
        let c = RouteCache::new(100, 9, 900);
        let k1 = c
            .key(
                ll(10.7725, 106.6980),
                ll(10.8000, 106.7200),
                1,
                Some(8 * 3600),
            )
            .unwrap();
        let k2 = c
            .key(
                ll(10.77252, 106.69802),
                ll(10.80003, 106.72003),
                1,
                Some(8 * 3600 + 600),
            )
            .unwrap();
        let k3 = c
            .key(
                ll(10.7725, 106.6980),
                ll(10.8000, 106.7200),
                1,
                Some(8 * 3600 + 901),
            )
            .unwrap();
        let k4 = c
            .key(
                ll(10.7725, 106.6980),
                ll(10.8000, 106.7200),
                2,
                Some(8 * 3600),
            )
            .unwrap();
        assert_eq!(k1, k2, "cùng ô, cùng khung 15 phút");
        assert_ne!(k1, k3, "khung giờ khác");
        assert_ne!(k1, k4, "profile khác");
    }

    #[test]
    fn same_cell_pair_bypasses_the_cache() {
        let c = RouteCache::new(100, 9, 900);
        assert!(
            c.key(ll(10.7725, 106.6980), ll(10.77251, 106.69801), 1, None)
                .is_none()
        );
        assert_eq!(c.stats().bypass, 1);
    }

    #[test]
    fn put_get_and_bounded_eviction() {
        let c = RouteCache::new(32, 9, 900);
        let mk = |i: u64| CacheKey {
            from_cell: i,
            to_cell: i + 1,
            profile: 1,
            bucket: 0,
        };
        for i in 0..1000 {
            c.put(
                mk(i),
                CachedRoute {
                    duration_ms: i as u32,
                    distance_m: 1,
                },
            );
        }
        let st = c.stats();
        assert!(st.entries <= 32 + 16, "{}", st.entries);
        assert!(st.evictions > 900);
        // mục mới nhất còn
        assert_eq!(c.get(&mk(999)).map(|v| v.duration_ms), Some(999));
        assert!(c.get(&mk(0)).is_none());
        assert!(c.stats().hits >= 1 && c.stats().misses >= 1);
    }
}
