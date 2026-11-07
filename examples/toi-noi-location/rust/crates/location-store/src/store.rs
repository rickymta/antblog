//! Kho vị trí mới nhất trong bộ nhớ, đọc nhiều ghi nhiều.
//!
//! Cấu trúc:
//! - `drivers`: `slot → (ô hiện tại, giờ, seq)`, chia `N` mảnh khóa `Mutex` theo `slot`. Một tài xế luôn qua đúng một khóa nên
//!   cập nhật của cùng một tài xế được tuần tự hóa (phát hiện điểm cũ đến sau chính xác).
//! - `cells`: `ô H3 → danh sách (slot, vị trí)`, chia `N` mảnh khóa `RwLock` theo ô. Truy vấn gần nhất **chỉ** lấy khóa đọc
//!   của các mảnh ô, không đụng `drivers`, nên nhiều luồng đọc không chặn nhau và chỉ chặn ghi cùng mảnh.
//! - Thứ tự khóa luôn là `drivers` rồi `cells`, không bao giờ giữ hai khóa `cells` cùng lúc ⇒ không có vòng chờ khóa.
//! - Đổi ô: chèn vào ô mới trước, xóa khỏi ô cũ sau, nên truy vấn không bao giờ **mất** tài xế; trong khoảnh khắc đó có thể
//!   thấy hai bản, truy vấn gộp theo `slot` và giữ bản mới hơn.
//!
//! TTL: tài xế im lặng quá `ttl_ms` (tính theo `received_ms`, đồng hồ server) bị truy vấn bỏ qua ngay, và [`LatestStore::sweep`]
//! dọn hẳn khỏi bộ nhớ.

use crate::model::{Position, Update};
use geo_core::h3::{CellIndex, Resolution, cell_of, edge_length_avg_m, resolution};
use geo_core::{LatLng, equirect_m};
use rustc_hash::{FxHashMap, FxHashSet};
use std::hash::{BuildHasher, BuildHasherDefault};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

#[derive(Debug, Clone)]
pub struct StoreConfig {
    /// Độ phân giải H3. 9 có cạnh ô khoảng 200 m (khớp LOC-08/LOC-09).
    pub h3_res: u8,
    pub ttl_ms: i64,
    /// Số mảnh khóa cho mỗi loại (làm tròn lên lũy thừa của 2 không cần thiết).
    pub shards: usize,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            h3_res: 9,
            ttl_ms: 30_000,
            shards: 64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applied {
    Inserted,
    Updated {
        moved_cell: bool,
    },
    /// Điểm cũ hơn điểm đang giữ (đến muộn, hoặc trùng): bỏ.
    Stale,
}

#[derive(Debug, Clone)]
pub struct Nearby {
    pub id: Arc<str>,
    pub dist_m: f64,
    pub pos: Position,
}

#[derive(Debug, Clone, Default)]
pub struct StoreStats {
    pub drivers: usize,
    pub cells: usize,
    pub applied: u64,
    pub inserted: u64,
    pub stale: u64,
    pub cell_moves: u64,
    pub expired: u64,
}

#[derive(Clone, Copy)]
struct DriverRec {
    cell: CellIndex,
    occurred_ms: i64,
    received_ms: i64,
    seq: u32,
}

#[derive(Clone, Copy)]
struct Entry {
    slot: u32,
    pos: Position,
}

#[derive(Default)]
struct Registry {
    map: RwLock<FxHashMap<Arc<str>, u32>>,
    names: RwLock<Vec<Arc<str>>>,
}

impl Registry {
    fn slot(&self, id: &str) -> u32 {
        if let Some(&s) = self.map.read().expect("registry").get(id) {
            return s;
        }
        let mut map = self.map.write().expect("registry");
        if let Some(&s) = map.get(id) {
            return s;
        }
        let mut names = self.names.write().expect("registry");
        let s = names.len() as u32;
        let name: Arc<str> = Arc::from(id);
        names.push(name.clone());
        map.insert(name, s);
        s
    }
    fn name(&self, slot: u32) -> Arc<str> {
        self.names.read().expect("registry")[slot as usize].clone()
    }
    fn find(&self, id: &str) -> Option<u32> {
        self.map.read().expect("registry").get(id).copied()
    }
}

type CellMap = FxHashMap<CellIndex, Vec<Entry>>;

pub struct LatestStore {
    cfg: StoreConfig,
    res: Resolution,
    registry: Registry,
    drivers: Vec<Mutex<FxHashMap<u32, DriverRec>>>,
    cells: Vec<RwLock<CellMap>>,
    applied: AtomicU64,
    inserted: AtomicU64,
    stale: AtomicU64,
    cell_moves: AtomicU64,
    expired: AtomicU64,
}

fn cell_shard(cell: CellIndex, n: usize) -> usize {
    (BuildHasherDefault::<rustc_hash::FxHasher>::default().hash_one(u64::from(cell)) as usize) % n
}

impl LatestStore {
    pub fn new(cfg: StoreConfig) -> Self {
        let n = cfg.shards.max(1);
        Self {
            res: resolution(cfg.h3_res),
            registry: Registry::default(),
            drivers: (0..n).map(|_| Mutex::new(FxHashMap::default())).collect(),
            cells: (0..n).map(|_| RwLock::new(FxHashMap::default())).collect(),
            applied: AtomicU64::new(0),
            inserted: AtomicU64::new(0),
            stale: AtomicU64::new(0),
            cell_moves: AtomicU64::new(0),
            expired: AtomicU64::new(0),
            cfg,
        }
    }

    pub fn config(&self) -> &StoreConfig {
        &self.cfg
    }

    fn cells_for(&self, cell: CellIndex) -> &RwLock<CellMap> {
        &self.cells[cell_shard(cell, self.cells.len())]
    }

    fn put_in_cell(&self, cell: CellIndex, e: Entry) {
        let mut m = self.cells_for(cell).write().expect("cells");
        let v = m.entry(cell).or_default();
        match v.iter_mut().find(|x| x.slot == e.slot) {
            Some(x) => *x = e,
            None => v.push(e),
        }
    }

    fn take_from_cell(&self, cell: CellIndex, slot: u32) {
        let mut m = self.cells_for(cell).write().expect("cells");
        if let Some(v) = m.get_mut(&cell) {
            if let Some(at) = v.iter().position(|x| x.slot == slot) {
                v.swap_remove(at);
            }
            if v.is_empty() {
                m.remove(&cell);
            }
        }
    }

    /// Áp một cập nhật. Điểm có giờ cũ hơn (hoặc bằng giờ mà `seq` không lớn hơn) bị bỏ.
    pub fn apply(&self, u: &Update) -> Applied {
        let slot = self.registry.slot(&u.driver_id);
        let cell = cell_of(u.pos.latlng(), self.res);
        let entry = Entry { slot, pos: u.pos };
        let rec_new = DriverRec {
            cell,
            occurred_ms: u.pos.occurred_ms,
            received_ms: u.pos.received_ms,
            seq: u.pos.seq,
        };
        let mut shard = self.drivers[slot as usize % self.drivers.len()]
            .lock()
            .expect("drivers");
        self.applied.fetch_add(1, Ordering::Relaxed);
        match shard.get_mut(&slot) {
            Some(rec) => {
                let stale = u.pos.occurred_ms < rec.occurred_ms
                    || (u.pos.occurred_ms == rec.occurred_ms && u.pos.seq <= rec.seq);
                if stale {
                    self.stale.fetch_add(1, Ordering::Relaxed);
                    return Applied::Stale;
                }
                let moved = rec.cell != cell;
                self.put_in_cell(cell, entry);
                if moved {
                    self.take_from_cell(rec.cell, slot);
                    self.cell_moves.fetch_add(1, Ordering::Relaxed);
                }
                *rec = rec_new;
                Applied::Updated { moved_cell: moved }
            }
            None => {
                self.put_in_cell(cell, entry);
                shard.insert(slot, rec_new);
                self.inserted.fetch_add(1, Ordering::Relaxed);
                Applied::Inserted
            }
        }
    }

    /// Vị trí mới nhất của một tài xế, nếu còn trong TTL tại `now_ms`.
    pub fn get(&self, id: &str, now_ms: i64) -> Option<Position> {
        let slot = self.registry.find(id)?;
        let cell = self.drivers[slot as usize % self.drivers.len()]
            .lock()
            .expect("drivers")
            .get(&slot)?
            .cell;
        let m = self.cells_for(cell).read().expect("cells");
        m.get(&cell)?
            .iter()
            .find(|e| e.slot == slot)
            .map(|e| e.pos)
            .filter(|p| p.received_ms + self.cfg.ttl_ms >= now_ms)
    }

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

    /// `k` tài xế gần `q` nhất trong `max_radius_m`, gần trước, bỏ tài xế quá TTL tại `now_ms`.
    /// Mở rộng từng vòng ô H3 quanh điểm hỏi và dừng khi vòng kế tiếp chắc chắn xa hơn tài xế thứ `k`.
    pub fn nearest(&self, q: LatLng, k: usize, max_radius_m: f64, now_ms: i64) -> Vec<Nearby> {
        if k == 0 {
            return Vec::new();
        }
        let c0 = cell_of(q, self.res);
        let edge = edge_length_avg_m(self.res);
        let mut best: Vec<(f64, Entry)> = Vec::with_capacity(k + 1);
        let max_ring = ((max_radius_m / (edge * 1.2)).ceil() as u32 + 1).min(200);
        for r in 0..=max_ring {
            for cell in Self::ring(c0, r) {
                let m = self.cells_for(cell).read().expect("cells");
                let Some(v) = m.get(&cell) else { continue };
                for e in v {
                    if e.pos.received_ms + self.cfg.ttl_ms < now_ms {
                        continue;
                    }
                    let d = equirect_m(q, e.pos.latlng());
                    if d > max_radius_m {
                        continue;
                    }
                    if let Some(old) = best.iter_mut().find(|(_, o)| o.slot == e.slot) {
                        if e.pos.occurred_ms > old.1.pos.occurred_ms {
                            *old = (d, *e);
                        }
                        continue;
                    }
                    if best.len() == k && best.last().is_some_and(|l| d >= l.0) {
                        continue;
                    }
                    let at = best.partition_point(|b| b.0 <= d);
                    best.insert(at, (d, *e));
                    best.truncate(k);
                }
            }
            // mọi điểm ở ô cách >= r+1 bước cách điểm hỏi ít nhất (1,5·r − 0,5)·cạnh; nhân 0,8 cho độ méo của lưới
            let safe = (1.5 * f64::from(r) - 0.5).max(0.0) * edge * 0.8;
            if best.len() == k && best.last().is_some_and(|l| l.0 <= safe) {
                break;
            }
        }
        best.into_iter()
            .map(|(d, e)| Nearby {
                id: self.registry.name(e.slot),
                dist_m: d,
                pos: e.pos,
            })
            .collect()
    }

    /// Dọn tài xế im lặng quá TTL tại `now_ms` khỏi bộ nhớ. Trả danh sách mã đã xóa (để xóa luôn ở bản sao Redis).
    pub fn sweep(&self, now_ms: i64) -> Vec<Arc<str>> {
        let mut gone = Vec::new();
        for shard in &self.drivers {
            let mut g = shard.lock().expect("drivers");
            let dead: Vec<(u32, CellIndex)> = g
                .iter()
                .filter(|(_, r)| r.received_ms + self.cfg.ttl_ms < now_ms)
                .map(|(&s, r)| (s, r.cell))
                .collect();
            for (slot, cell) in dead {
                g.remove(&slot);
                self.take_from_cell(cell, slot);
                gone.push(self.registry.name(slot));
            }
        }
        self.expired.fetch_add(gone.len() as u64, Ordering::Relaxed);
        gone
    }

    /// Mọi tài xế đang giữ (kể cả quá TTL chưa dọn), dùng để lưu bản sao.
    pub fn snapshot(&self) -> Vec<(Arc<str>, Position)> {
        let mut out = Vec::new();
        for shard in &self.cells {
            for v in shard.read().expect("cells").values() {
                for e in v {
                    out.push((self.registry.name(e.slot), e.pos));
                }
            }
        }
        out
    }

    pub fn len(&self) -> usize {
        self.drivers
            .iter()
            .map(|s| s.lock().expect("drivers").len())
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn stats(&self) -> StoreStats {
        StoreStats {
            drivers: self.len(),
            cells: self
                .cells
                .iter()
                .map(|s| s.read().expect("cells").len())
                .sum(),
            applied: self.applied.load(Ordering::Relaxed),
            inserted: self.inserted.load(Ordering::Relaxed),
            stale: self.stale.load(Ordering::Relaxed),
            cell_moves: self.cell_moves.load(Ordering::Relaxed),
            expired: self.expired.load(Ordering::Relaxed),
        }
    }

    /// Kiểm tra bất biến: mỗi tài xế đúng một bản ở đúng ô đã ghi. Dùng trong test.
    pub fn check_invariants(&self) -> Result<(), String> {
        let mut seen: FxHashMap<u32, CellIndex> = FxHashMap::default();
        for shard in &self.cells {
            for (cell, v) in shard.read().expect("cells").iter() {
                if v.is_empty() {
                    return Err("ô rỗng còn trong bảng".into());
                }
                for e in v {
                    if seen.insert(e.slot, *cell).is_some() {
                        return Err(format!("tài xế slot {} có hai bản", e.slot));
                    }
                }
            }
        }
        for shard in &self.drivers {
            for (slot, rec) in shard.lock().expect("drivers").iter() {
                if seen.get(slot) != Some(&rec.cell) {
                    return Err(format!("slot {slot} lệch ô giữa drivers và cells"));
                }
            }
        }
        let n: usize = self
            .drivers
            .iter()
            .map(|s| s.lock().expect("drivers").len())
            .sum();
        if n != seen.len() {
            return Err("số tài xế lệch".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_core::LatLngE7;
    use proptest::prelude::*;

    fn up(id: &str, t_ms: i64, seq: u32, lat: f64, lng: f64) -> Update {
        Update {
            driver_id: id.into(),
            city_code: "HCM".into(),
            trip_id: String::new(),
            pos: Position {
                pos: LatLng::try_new(lat, lng).unwrap().to_e7(),
                occurred_ms: t_ms,
                received_ms: t_ms,
                seq,
                accuracy_dm: None,
                speed_dmps: None,
                heading_ddeg: None,
                flags: 0,
            },
        }
    }

    fn store() -> LatestStore {
        LatestStore::new(StoreConfig {
            ttl_ms: 30_000,
            shards: 8,
            ..StoreConfig::default()
        })
    }

    #[test]
    fn insert_update_stale_and_cell_move() {
        let s = store();
        assert_eq!(
            s.apply(&up("a", 1000, 1, 10.7700, 106.6900)),
            Applied::Inserted
        );
        assert_eq!(
            s.apply(&up("a", 2000, 2, 10.7700, 106.6900)),
            Applied::Updated { moved_cell: false }
        );
        // điểm cũ đến sau, và điểm trùng giờ không có seq mới hơn: bỏ
        assert_eq!(s.apply(&up("a", 1500, 9, 10.80, 106.70)), Applied::Stale);
        assert_eq!(s.apply(&up("a", 2000, 2, 10.80, 106.70)), Applied::Stale);
        assert_eq!(s.get("a", 2000).unwrap().occurred_ms, 2000);
        assert_eq!(
            s.apply(&up("a", 3000, 3, 10.8300, 106.7300)),
            Applied::Updated { moved_cell: true }
        );
        let st = s.stats();
        assert_eq!(
            (st.drivers, st.cells, st.stale, st.cell_moves),
            (1, 1, 2, 1)
        );
        s.check_invariants().unwrap();
    }

    #[test]
    fn ttl_hides_silent_drivers_and_sweep_removes_them() {
        let s = store();
        s.apply(&up("quiet", 0, 1, 10.77, 106.69));
        s.apply(&up("chatty", 25_000, 1, 10.7701, 106.6901));
        let q = LatLng::try_new(10.77, 106.69).unwrap();
        assert_eq!(s.nearest(q, 5, 500.0, 29_000).len(), 2);
        // tại 31 s tài xế im lặng quá TTL 30 s: truy vấn đã bỏ qua dù chưa dọn
        let ids: Vec<_> = s
            .nearest(q, 5, 500.0, 31_000)
            .iter()
            .map(|n| n.id.to_string())
            .collect();
        assert_eq!(ids, ["chatty"]);
        assert!(s.get("quiet", 31_000).is_none());
        assert_eq!(s.len(), 2);
        let gone = s.sweep(31_000);
        assert_eq!(gone.len(), 1);
        assert_eq!(&*gone[0], "quiet");
        assert_eq!(s.len(), 1);
        s.check_invariants().unwrap();
        // quay lại sau khi bị dọn thì vào lại như tài xế mới
        assert_eq!(
            s.apply(&up("quiet", 40_000, 2, 10.77, 106.69)),
            Applied::Inserted
        );
    }

    #[test]
    fn nearest_respects_radius_and_k() {
        let s = store();
        for i in 0..10 {
            s.apply(&up(
                &format!("d{i}"),
                1000,
                1,
                10.77 + 0.0005 * f64::from(i),
                106.69,
            ));
        }
        let q = LatLng::try_new(10.77, 106.69).unwrap();
        let r = s.nearest(q, 3, 10_000.0, 1000);
        assert_eq!(
            r.iter().map(|n| &*n.id).collect::<Vec<_>>(),
            ["d0", "d1", "d2"]
        );
        assert!(r.windows(2).all(|w| w[0].dist_m <= w[1].dist_m));
        // d3 cách ≈ 167 m: bán kính 150 m chỉ thấy d0..d2
        assert_eq!(s.nearest(q, 10, 150.0, 1000).len(), 3);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        #[test]
        fn nearest_matches_brute_force(
            pts in proptest::collection::vec((0.0f64..0.05, 0.0f64..0.05), 1..120),
            q in (0.0f64..0.05, 0.0f64..0.05),
            k in 1usize..8,
        ) {
            let s = store();
            let mut latest: Vec<(String, LatLng)> = Vec::new();
            for (i, (a, b)) in pts.iter().enumerate() {
                let id = format!("d{}", i % 40); // nhiều lần cập nhật cho cùng tài xế
                let u = up(&id, 1000 + i as i64, i as u32, 10.75 + a, 106.65 + b);
                s.apply(&u);
                latest.retain(|(x, _)| x != &id);
                latest.push((id, u.pos.latlng()));
            }
            s.check_invariants().unwrap();
            let q = LatLng::try_new(10.75 + q.0, 106.65 + q.1).unwrap();
            let got: Vec<f64> = s.nearest(q, k, 50_000.0, 2000).iter().map(|n| n.dist_m).collect();
            let mut want: Vec<f64> = latest.iter().map(|(_, p)| equirect_m(q, *p)).collect();
            want.sort_by(f64::total_cmp);
            want.truncate(k);
            prop_assert_eq!(got.len(), want.len());
            for (g, w) in got.iter().zip(&want) {
                prop_assert!((g - w).abs() < 0.01, "{g} vs {w}");
            }
        }
    }

    #[test]
    fn concurrent_writers_and_readers_keep_invariants() {
        let s = Arc::new(store());
        let mut hs = Vec::new();
        for w in 0..4u32 {
            let s = s.clone();
            hs.push(std::thread::spawn(move || {
                for step in 0..400u32 {
                    for d in 0..25u32 {
                        let id = format!("w{w}-d{d}");
                        let lat = 10.75 + 0.0001 * f64::from(step) + 0.001 * f64::from(d);
                        s.apply(&up(
                            &id,
                            1000 + i64::from(step),
                            step,
                            lat,
                            106.65 + 0.0002 * f64::from(w),
                        ));
                    }
                }
            }));
        }
        for _ in 0..3 {
            let s = s.clone();
            hs.push(std::thread::spawn(move || {
                let q = LatLng::try_new(10.77, 106.65).unwrap();
                for _ in 0..300 {
                    let r = s.nearest(q, 5, 5_000.0, 2000);
                    let mut ids: Vec<_> = r.iter().map(|n| n.id.clone()).collect();
                    ids.sort();
                    ids.dedup();
                    assert_eq!(
                        ids.len(),
                        r.len(),
                        "truy vấn không được trả một tài xế hai lần"
                    );
                }
            }));
        }
        for h in hs {
            h.join().unwrap();
        }
        s.check_invariants().unwrap();
        assert_eq!(s.len(), 100);
        let p = s.get("w2-d7", 2000).unwrap();
        assert_eq!(p.seq, 399);
        let _ = LatLngE7::ABSENT;
    }
}
