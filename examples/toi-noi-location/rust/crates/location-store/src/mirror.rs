//! Redis làm **bản phụ** của kho vị trí: không nằm trên đường đọc nóng, chỉ để (a) khởi động lại nhanh, (b) dịch vụ khác
//! cần `GEOSEARCH` mà không muốn giữ bản đồ trong RAM.
//!
//! - `<prefix>:geo`: sorted set cho `GEOSEARCH` (member = driver_id).
//! - `<prefix>:pos`: hash `driver_id → bản ghi nhị phân 46 byte` (đủ giờ, seq, accuracy/speed/heading, cờ) để nạp lại.
//! - [`MirrorWriter`]: luồng riêng nhận cập nhật qua hàng đợi **có giới hạn**, gộp theo tài xế và ghi theo lô (pipeline).
//!   Redis chậm hay chết thì hàng đợi đầy và cập nhật bị bỏ (đếm lại), không bao giờ chặn luồng tiêu thụ Kafka.

use crate::StoreError;
use crate::model::Position;
use geo_core::LatLngE7;
use rustc_hash::FxHashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{RecvTimeoutError, SyncSender, sync_channel};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

const REC_LEN: usize = 46;

/// Mã hóa vị trí thành 46 byte little-endian: phiên bản (1), lat, lng (i32), occurred, received (i64), seq, flags (u32),
/// mặt nạ có mặt (1: bit0 accuracy, bit1 speed, bit2 heading), accuracy, speed, heading (u32).
pub fn pack(p: &Position) -> [u8; REC_LEN] {
    let mut b = [0u8; REC_LEN];
    b[0] = 1;
    b[1..5].copy_from_slice(&p.pos.lat_e7().to_le_bytes());
    b[5..9].copy_from_slice(&p.pos.lng_e7().to_le_bytes());
    b[9..17].copy_from_slice(&p.occurred_ms.to_le_bytes());
    b[17..25].copy_from_slice(&p.received_ms.to_le_bytes());
    b[25..29].copy_from_slice(&p.seq.to_le_bytes());
    b[29..33].copy_from_slice(&p.flags.to_le_bytes());
    let mask = u8::from(p.accuracy_dm.is_some())
        | u8::from(p.speed_dmps.is_some()) << 1
        | u8::from(p.heading_ddeg.is_some()) << 2;
    b[33] = mask;
    b[34..38].copy_from_slice(&p.accuracy_dm.unwrap_or(0).to_le_bytes());
    b[38..42].copy_from_slice(&p.speed_dmps.unwrap_or(0).to_le_bytes());
    b[42..46].copy_from_slice(&p.heading_ddeg.unwrap_or(0).to_le_bytes());
    b
}

/// Giải mã; `None` nếu sai độ dài hoặc phiên bản lạ (bản ghi hỏng thì bỏ, không làm hỏng việc khởi động).
pub fn unpack(b: &[u8]) -> Option<Position> {
    if b.len() != REC_LEN || b[0] != 1 {
        return None;
    }
    let i32_at = |o: usize| i32::from_le_bytes(b[o..o + 4].try_into().expect("4 byte"));
    let u32_at = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().expect("4 byte"));
    let i64_at = |o: usize| i64::from_le_bytes(b[o..o + 8].try_into().expect("8 byte"));
    let mask = b[33];
    Some(Position {
        pos: LatLngE7::new(i32_at(1), i32_at(5)),
        occurred_ms: i64_at(9),
        received_ms: i64_at(17),
        seq: u32_at(25),
        flags: u32_at(29),
        accuracy_dm: (mask & 1 != 0).then(|| u32_at(34)),
        speed_dmps: (mask & 2 != 0).then(|| u32_at(38)),
        heading_ddeg: (mask & 4 != 0).then(|| u32_at(42)),
    })
}

pub struct RedisMirror {
    con: redis::Connection,
    geo_key: String,
    pos_key: String,
}

impl RedisMirror {
    pub fn connect(url: &str, prefix: &str) -> Result<Self, StoreError> {
        let client = redis::Client::open(url)?;
        let con = client.get_connection_with_timeout(Duration::from_secs(2))?;
        Ok(Self {
            con,
            geo_key: format!("{prefix}:geo"),
            pos_key: format!("{prefix}:pos"),
        })
    }

    pub fn clear(&mut self) -> Result<(), StoreError> {
        redis::cmd("DEL")
            .arg(&self.geo_key)
            .arg(&self.pos_key)
            .exec(&mut self.con)?;
        Ok(())
    }

    /// Ghi một lô trong một vòng mạng.
    pub fn write(&mut self, items: &[(Arc<str>, Position)]) -> Result<(), StoreError> {
        let mut pipe = redis::pipe();
        for (id, p) in items {
            let ll = p.latlng();
            pipe.cmd("GEOADD")
                .arg(&self.geo_key)
                .arg(ll.lng())
                .arg(ll.lat())
                .arg(&**id)
                .ignore();
            pipe.cmd("HSET")
                .arg(&self.pos_key)
                .arg(&**id)
                .arg(&pack(p)[..])
                .ignore();
        }
        pipe.exec(&mut self.con)?;
        Ok(())
    }

    pub fn remove(&mut self, ids: &[Arc<str>]) -> Result<(), StoreError> {
        let mut pipe = redis::pipe();
        for id in ids {
            pipe.cmd("ZREM").arg(&self.geo_key).arg(&**id).ignore();
            pipe.cmd("HDEL").arg(&self.pos_key).arg(&**id).ignore();
        }
        pipe.exec(&mut self.con)?;
        Ok(())
    }

    /// Nạp lại mọi bản ghi còn tươi (`received_ms + ttl_ms >= now_ms`). Bản ghi hỏng hoặc quá hạn bị bỏ, được đếm.
    pub fn load_fresh(
        &mut self,
        now_ms: i64,
        ttl_ms: i64,
    ) -> Result<(Vec<(String, Position)>, LoadStats), StoreError> {
        let rows: FxHashMap<String, Vec<u8>> = redis::cmd("HGETALL")
            .arg(&self.pos_key)
            .query(&mut self.con)?;
        let mut st = LoadStats {
            read: rows.len(),
            ..LoadStats::default()
        };
        let mut out = Vec::with_capacity(rows.len());
        for (id, raw) in rows {
            match unpack(&raw) {
                None => st.corrupt += 1,
                Some(p) if p.received_ms + ttl_ms < now_ms => st.expired += 1,
                Some(p) => out.push((id, p)),
            }
        }
        st.loaded = out.len();
        Ok((out, st))
    }

    /// `GEOSEARCH ... BYRADIUS ... ASC COUNT k`: mã tài xế gần nhất do chính Redis trả (thứ tự kinh độ trước, vĩ độ sau).
    pub fn geo_nearest(
        &mut self,
        q: geo_core::LatLng,
        k: usize,
        radius_m: u32,
    ) -> Result<Vec<String>, StoreError> {
        Ok(redis::cmd("GEOSEARCH")
            .arg(&self.geo_key)
            .arg("FROMLONLAT")
            .arg(q.lng())
            .arg(q.lat())
            .arg("BYRADIUS")
            .arg(radius_m)
            .arg("m")
            .arg("ASC")
            .arg("COUNT")
            .arg(k)
            .query(&mut self.con)?)
    }

    pub fn geo_len(&mut self) -> Result<usize, StoreError> {
        Ok(redis::cmd("ZCARD")
            .arg(&self.geo_key)
            .query(&mut self.con)?)
    }

    pub fn ping(&mut self) -> Result<(), StoreError> {
        redis::cmd("PING").exec(&mut self.con)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct LoadStats {
    pub read: usize,
    pub loaded: usize,
    pub expired: usize,
    pub corrupt: usize,
}

#[derive(Default)]
pub struct MirrorStats {
    pub queued: AtomicU64,
    /// Cập nhật bị bỏ vì hàng đợi đầy (Redis chậm hoặc không chạy).
    pub dropped: AtomicU64,
    /// Số lệnh ghi thực sự gửi sau khi gộp theo tài xế.
    pub written: AtomicU64,
    pub batches: AtomicU64,
    pub errors: AtomicU64,
}

enum Msg {
    Upsert(Arc<str>, Position),
    Remove(Arc<str>),
    Flush(SyncSender<()>),
    Stop,
}

/// Luồng ghi Redis: gộp theo tài xế trong `flush_every` hoặc khi đủ `max_batch`, rồi ghi một pipeline.
pub struct MirrorWriter {
    tx: SyncSender<Msg>,
    pub stats: Arc<MirrorStats>,
    join: Option<JoinHandle<()>>,
}

impl MirrorWriter {
    pub fn spawn(
        mirror: RedisMirror,
        queue: usize,
        max_batch: usize,
        flush_every: Duration,
    ) -> Self {
        let (tx, rx) = sync_channel::<Msg>(queue);
        let stats = Arc::new(MirrorStats::default());
        let st = stats.clone();
        let join = std::thread::spawn(move || {
            let mut mirror = mirror;
            let mut pending: FxHashMap<Arc<str>, Position> = FxHashMap::default();
            let mut removals: Vec<Arc<str>> = Vec::new();
            let mut last = Instant::now();
            let flush = |m: &mut RedisMirror,
                         pending: &mut FxHashMap<Arc<str>, Position>,
                         removals: &mut Vec<Arc<str>>| {
                if !pending.is_empty() {
                    let items: Vec<(Arc<str>, Position)> = pending.drain().collect();
                    match m.write(&items) {
                        Ok(()) => {
                            st.written.fetch_add(items.len() as u64, Ordering::Relaxed);
                            st.batches.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(_) => {
                            st.errors.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
                if !removals.is_empty() {
                    if m.remove(removals).is_err() {
                        st.errors.fetch_add(1, Ordering::Relaxed);
                    }
                    removals.clear();
                }
            };
            loop {
                match rx.recv_timeout(flush_every) {
                    Ok(Msg::Upsert(id, p)) => {
                        pending.insert(id, p);
                    }
                    Ok(Msg::Remove(id)) => {
                        pending.remove(&id);
                        removals.push(id);
                    }
                    Ok(Msg::Flush(ack)) => {
                        flush(&mut mirror, &mut pending, &mut removals);
                        let _ = ack.send(());
                    }
                    Ok(Msg::Stop) | Err(RecvTimeoutError::Disconnected) => {
                        flush(&mut mirror, &mut pending, &mut removals);
                        return;
                    }
                    Err(RecvTimeoutError::Timeout) => {}
                }
                if pending.len() >= max_batch
                    || (last.elapsed() >= flush_every && !pending.is_empty())
                {
                    flush(&mut mirror, &mut pending, &mut removals);
                    last = Instant::now();
                }
            }
        });
        Self {
            tx,
            stats,
            join: Some(join),
        }
    }

    /// Không bao giờ chặn: hàng đợi đầy thì bỏ và đếm.
    pub fn upsert(&self, id: Arc<str>, p: Position) {
        self.stats.queued.fetch_add(1, Ordering::Relaxed);
        if self.tx.try_send(Msg::Upsert(id, p)).is_err() {
            self.stats.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn remove(&self, id: Arc<str>) {
        let _ = self.tx.try_send(Msg::Remove(id));
    }

    /// Chờ ghi xong mọi thứ đã nhận (dùng trong test và khi tắt).
    pub fn flush(&self) {
        let (ack_tx, ack_rx) = sync_channel(1);
        if self.tx.send(Msg::Flush(ack_tx)).is_ok() {
            let _ = ack_rx.recv_timeout(Duration::from_secs(30));
        }
    }

    pub fn stop(mut self) {
        let _ = self.tx.send(Msg::Stop);
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(acc: Option<u32>, speed: Option<u32>, head: Option<u32>) -> Position {
        Position {
            pos: LatLngE7::new(-107_725_001, 1_066_980_000),
            occurred_ms: 1_700_000_000_123,
            received_ms: 1_700_000_000_456,
            seq: 4_000_000_000,
            accuracy_dm: acc,
            speed_dmps: speed,
            heading_ddeg: head,
            flags: 0b101010,
        }
    }

    #[test]
    fn pack_round_trip_all_presence_combinations() {
        for mask in 0..8u32 {
            let p = pos(
                (mask & 1 != 0).then_some(0),
                (mask & 2 != 0).then_some(0),
                (mask & 4 != 0).then_some(3599),
            );
            assert_eq!(unpack(&pack(&p)), Some(p), "mask {mask}");
        }
    }

    #[test]
    fn unpack_rejects_wrong_length_and_version() {
        let mut b = pack(&pos(None, None, None)).to_vec();
        assert!(unpack(&b[..45]).is_none());
        b[0] = 9;
        assert!(unpack(&b).is_none());
    }
}
