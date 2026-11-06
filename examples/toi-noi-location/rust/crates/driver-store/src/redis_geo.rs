//! Redis GEO: `GEOADD` ghi vị trí vào một sorted set (geohash 52 bit làm score), `GEOSEARCH` tìm theo bán kính.
//! Đây là đồng hồ ngoài tiến trình: mỗi thao tác là một vòng mạng, kể cả khi chạy trên loopback.
//! Thứ tự tham số của Redis là **kinh độ trước, vĩ độ sau**, ngược với `LatLng`; chuyển qua `to_lnglat`.

use crate::{DriverId, Neighbor};
use geo_core::LatLng;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Redis: {0}")]
    Redis(#[from] redis::RedisError),
}

pub struct RedisGeo {
    con: redis::Connection,
    key: String,
}

impl RedisGeo {
    /// Kết nối (đồng bộ). Lỗi nếu Redis không chạy; người gọi quyết định dừng hay bỏ qua có thông báo.
    pub fn connect(url: &str, key: &str) -> Result<Self, StoreError> {
        let client = redis::Client::open(url)?;
        let con = client.get_connection_with_timeout(std::time::Duration::from_secs(2))?;
        Ok(Self {
            con,
            key: key.to_string(),
        })
    }

    /// Xóa key của lab.
    pub fn clear(&mut self) -> Result<(), StoreError> {
        redis::cmd("DEL").arg(&self.key).exec(&mut self.con)?;
        Ok(())
    }

    /// Thêm/cập nhật một tài xế: một vòng mạng.
    pub fn upsert(&mut self, id: DriverId, pos: LatLng) -> Result<(), StoreError> {
        let [lng, lat] = pos.to_lnglat();
        redis::cmd("GEOADD")
            .arg(&self.key)
            .arg(lng)
            .arg(lat)
            .arg(id)
            .exec(&mut self.con)?;
        Ok(())
    }

    /// Cập nhật nhiều tài xế trong một pipeline (một vòng mạng cho cả lô).
    pub fn upsert_batch(&mut self, items: &[(DriverId, LatLng)]) -> Result<(), StoreError> {
        let mut pipe = redis::pipe();
        for &(id, pos) in items {
            let [lng, lat] = pos.to_lnglat();
            pipe.cmd("GEOADD")
                .arg(&self.key)
                .arg(lng)
                .arg(lat)
                .arg(id)
                .ignore();
        }
        pipe.exec(&mut self.con)?;
        Ok(())
    }

    pub fn remove(&mut self, id: DriverId) -> Result<bool, StoreError> {
        let n: i64 = redis::cmd("ZREM")
            .arg(&self.key)
            .arg(id)
            .query(&mut self.con)?;
        Ok(n > 0)
    }

    /// `k` tài xế gần nhất trong bán kính `radius_m` (`GEOSEARCH ... BYRADIUS ... ASC COUNT k WITHDIST`).
    /// Bán kính là bắt buộc của lệnh: nếu trong bán kính ít hơn `k` người thì chỉ trả bấy nhiêu.
    pub fn nearest(
        &mut self,
        q: LatLng,
        k: usize,
        radius_m: u32,
    ) -> Result<Vec<Neighbor>, StoreError> {
        let [lng, lat] = q.to_lnglat();
        let rows: Vec<(String, f64)> = redis::cmd("GEOSEARCH")
            .arg(&self.key)
            .arg("FROMLONLAT")
            .arg(lng)
            .arg(lat)
            .arg("BYRADIUS")
            .arg(radius_m)
            .arg("m")
            .arg("ASC")
            .arg("COUNT")
            .arg(k)
            .arg("WITHDIST")
            .query(&mut self.con)?;
        Ok(rows
            .into_iter()
            .filter_map(|(m, d)| {
                Some(Neighbor {
                    id: m.parse().ok()?,
                    dist_m: d,
                })
            })
            .collect())
    }

    pub fn len(&mut self) -> Result<usize, StoreError> {
        let n: usize = redis::cmd("ZCARD").arg(&self.key).query(&mut self.con)?;
        Ok(n)
    }

    pub fn is_empty(&mut self) -> Result<bool, StoreError> {
        Ok(self.len()? == 0)
    }

    /// Một lệnh `PING`: dùng đo chi phí vòng mạng tối thiểu.
    pub fn ping(&mut self) -> Result<(), StoreError> {
        redis::cmd("PING").exec(&mut self.con)?;
        Ok(())
    }

    /// `used_memory` của server theo `INFO memory`, byte.
    pub fn used_memory_bytes(&mut self) -> Result<u64, StoreError> {
        let info: String = redis::cmd("INFO").arg("memory").query(&mut self.con)?;
        Ok(info
            .lines()
            .find_map(|l| l.strip_prefix("used_memory:"))
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(0))
    }

    /// Phiên bản Redis server.
    pub fn server_version(&mut self) -> Result<String, StoreError> {
        let info: String = redis::cmd("INFO").arg("server").query(&mut self.con)?;
        Ok(info
            .lines()
            .find_map(|l| l.strip_prefix("redis_version:"))
            .map_or_else(String::new, |v| v.trim().to_string()))
    }
}
