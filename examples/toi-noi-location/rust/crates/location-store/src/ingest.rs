//! Ống nạp: một cập nhật đi qua kho trong bộ nhớ, bản sao Redis và bộ giảm mẫu sang lịch sử, rồi vòng lặp tiêu thụ Kafka.
//!
//! Thứ tự cố ý: kho trong bộ nhớ trước (đường nóng, phục vụ truy vấn), Redis và lịch sử sau và **không bao giờ chặn** kho.
//! Điểm cũ đến sau (bị kho bỏ) không được ghi vào Redis hay lịch sử.

use crate::StoreError;
use crate::history::{HistorySink, Sampler};
use crate::kafka::{Event, KafkaConsumer};
use crate::mirror::MirrorWriter;
use crate::model::{Update, now_ms};
use crate::store::{Applied, LatestStore};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
pub struct IngestCounters {
    pub messages: u64,
    pub applied: u64,
    pub stale: u64,
    pub poison: u64,
    pub history_kept: u64,
    pub swept: u64,
}

pub struct Ingest<S: HistorySink> {
    pub store: Arc<LatestStore>,
    pub sampler: Sampler,
    pub sink: S,
    pub mirror: Option<MirrorWriter>,
    pub counters: IngestCounters,
}

impl<S: HistorySink> Ingest<S> {
    pub fn new(
        store: Arc<LatestStore>,
        sampler: Sampler,
        sink: S,
        mirror: Option<MirrorWriter>,
    ) -> Self {
        Self {
            store,
            sampler,
            sink,
            mirror,
            counters: IngestCounters::default(),
        }
    }

    /// Xử lý một cập nhật.
    pub fn handle(&mut self, u: &Update) -> Applied {
        self.counters.messages += 1;
        let r = self.store.apply(u);
        if r == Applied::Stale {
            self.counters.stale += 1;
            return r;
        }
        self.counters.applied += 1;
        if let Some(m) = &self.mirror {
            m.upsert(Arc::from(u.driver_id.as_str()), u.pos);
        }
        if let Some(why) = self.sampler.keep(u) {
            self.sink.write(u, why);
            self.counters.history_kept += 1;
        }
        r
    }

    /// Dọn tài xế im lặng, báo cho bản sao Redis xóa theo.
    pub fn sweep(&mut self, now_ms: i64) -> usize {
        let gone = self.store.sweep(now_ms);
        if let Some(m) = &self.mirror {
            for id in &gone {
                m.remove(id.clone());
            }
        }
        self.counters.swept += gone.len() as u64;
        gone.len()
    }

    /// Vòng lặp tiêu thụ cho tới khi `stop`. `on_apply(&update, thời gian xử lý)` cho lab đo độ trễ; `sweep_every` là chu kỳ dọn TTL.
    pub fn run(
        &mut self,
        consumer: &KafkaConsumer,
        stop: &AtomicBool,
        sweep_every: Duration,
        mut on_apply: impl FnMut(&Update),
    ) -> Result<(), StoreError> {
        let mut last_sweep = Instant::now();
        while !stop.load(Ordering::Relaxed) {
            match consumer.poll(Duration::from_millis(20))? {
                Some((Event::Update(u), _, _)) => {
                    self.handle(&u);
                    on_apply(&u);
                }
                Some((Event::Poison, _, _)) => {
                    self.counters.messages += 1;
                    self.counters.poison += 1;
                }
                None => {}
            }
            if last_sweep.elapsed() >= sweep_every {
                self.sweep(now_ms());
                last_sweep = Instant::now();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::{CountSink, Downsample};
    use crate::model::Position;
    use crate::store::StoreConfig;
    use geo_core::LatLngE7;

    fn up(id: &str, t_s: i64, lat_e7: i32) -> Update {
        Update {
            driver_id: id.into(),
            city_code: "HCM".into(),
            trip_id: String::new(),
            pos: Position {
                pos: LatLngE7::new(lat_e7, 1_066_980_000),
                occurred_ms: t_s * 1000,
                received_ms: t_s * 1000,
                seq: t_s as u32,
                accuracy_dm: None,
                speed_dmps: None,
                heading_ddeg: None,
                flags: 0,
            },
        }
    }

    #[test]
    fn stale_points_reach_neither_history_nor_store() {
        let store = Arc::new(LatestStore::new(StoreConfig::default()));
        let mut ing = Ingest::new(
            store.clone(),
            Sampler::new(Downsample::default()),
            CountSink::default(),
            None,
        );
        ing.handle(&up("a", 10, 107_725_000));
        ing.handle(&up("a", 5, 107_800_000)); // đến muộn, ở xa
        assert_eq!(ing.counters.stale, 1);
        assert_eq!(ing.sink.kept, 1, "chỉ điểm đầu vào lịch sử");
        assert_eq!(store.get("a", 10_000).unwrap().occurred_ms, 10_000);
    }

    #[test]
    fn sweep_removes_silent_driver_and_counts() {
        let store = Arc::new(LatestStore::new(StoreConfig {
            ttl_ms: 1_000,
            ..StoreConfig::default()
        }));
        let mut ing = Ingest::new(
            store.clone(),
            Sampler::default(),
            CountSink::default(),
            None,
        );
        ing.handle(&up("a", 1, 107_725_000));
        assert_eq!(ing.sweep(10_000), 1);
        assert_eq!(store.len(), 0);
    }
}
