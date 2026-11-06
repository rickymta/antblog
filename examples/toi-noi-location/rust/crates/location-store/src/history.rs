//! Giảm mẫu sang lịch sử: không phải điểm nào của kho vị trí cũng đáng lưu. Cùng chính sách với `history-writer` (Go, LOC-20):
//! giữ điểm đầu, điểm có cờ, khi đã di chuyển đủ xa, khi rẽ đủ gắt (và đủ lâu), và một điểm «còn sống» mỗi `heartbeat_s` khi đứng yên.

use crate::model::Update;
use geo_core::{bearing_diff_deg, haversine_m};
use rustc_hash::FxHashMap;

#[derive(Debug, Clone, Copy)]
pub struct Downsample {
    pub min_move_m: f64,
    pub turn_deg: f64,
    pub turn_min_s: f64,
    pub heartbeat_s: f64,
}

impl Default for Downsample {
    fn default() -> Self {
        Self {
            min_move_m: 60.0,
            turn_deg: 30.0,
            turn_min_s: 4.0,
            heartbeat_s: 60.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    First,
    Flagged,
    Moved,
    Turned,
    Heartbeat,
}

#[derive(Clone, Copy)]
struct Kept {
    pos: geo_core::LatLng,
    t_ms: i64,
    heading: Option<f64>,
}

/// Nhớ điểm đã giữ gần nhất của từng tài xế. Không an toàn cho nhiều luồng: mỗi luồng tiêu thụ giữ một `Sampler`
/// (key Kafka là `driver_id` nên một tài xế luôn về một partition, tức một luồng).
#[derive(Default)]
pub struct Sampler {
    cfg: Downsample,
    last: FxHashMap<String, Kept>,
}

impl Sampler {
    pub fn new(cfg: Downsample) -> Self {
        Self {
            cfg,
            last: FxHashMap::default(),
        }
    }

    pub fn keep(&mut self, u: &Update) -> Option<Reason> {
        let cur = Kept {
            pos: u.pos.latlng(),
            t_ms: u.pos.occurred_ms,
            heading: u.pos.heading_ddeg.map(|h| f64::from(h) / 10.0),
        };
        let reason = match self.last.get(&u.driver_id) {
            None => Some(Reason::First),
            Some(prev) => {
                let dt_s = (cur.t_ms - prev.t_ms) as f64 / 1000.0;
                if u.pos.flags != 0 {
                    Some(Reason::Flagged)
                } else if haversine_m(prev.pos, cur.pos) >= self.cfg.min_move_m {
                    Some(Reason::Moved)
                } else if let (Some(a), Some(b)) = (prev.heading, cur.heading)
                    && dt_s >= self.cfg.turn_min_s
                    && bearing_diff_deg(a, b) >= self.cfg.turn_deg
                {
                    Some(Reason::Turned)
                } else if dt_s >= self.cfg.heartbeat_s {
                    Some(Reason::Heartbeat)
                } else {
                    None
                }
            }
        };
        if reason.is_some() {
            self.last.insert(u.driver_id.clone(), cur);
        }
        reason
    }

    pub fn tracked(&self) -> usize {
        self.last.len()
    }
}

/// Nơi nhận điểm lịch sử.
pub trait HistorySink {
    fn write(&mut self, u: &Update, why: Reason);
}

/// Sink đếm theo lý do, không lưu gì (đo tỉ lệ giữ).
#[derive(Default, Debug, Clone)]
pub struct CountSink {
    pub kept: u64,
    pub first: u64,
    pub flagged: u64,
    pub moved: u64,
    pub turned: u64,
    pub heartbeat: u64,
}

impl HistorySink for CountSink {
    fn write(&mut self, _: &Update, why: Reason) {
        self.kept += 1;
        match why {
            Reason::First => self.first += 1,
            Reason::Flagged => self.flagged += 1,
            Reason::Moved => self.moved += 1,
            Reason::Turned => self.turned += 1,
            Reason::Heartbeat => self.heartbeat += 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Position;
    use geo_core::LatLngE7;

    fn up(id: &str, t_s: i64, lat_e7: i32, heading: Option<u32>, flags: u32) -> Update {
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
                heading_ddeg: heading,
                flags,
            },
        }
    }

    #[test]
    fn keeps_first_move_flag_turn_heartbeat_and_drops_the_rest() {
        let mut s = Sampler::new(Downsample::default());
        assert_eq!(
            s.keep(&up("a", 0, 107_725_000, Some(0), 0)),
            Some(Reason::First)
        );
        // 1 e-7 độ ≈ 1,1 cm; 100 đơn vị ≈ 1,1 m: đứng yên
        assert_eq!(s.keep(&up("a", 4, 107_725_100, Some(5), 0)), None);
        assert_eq!(s.keep(&up("a", 8, 107_725_000, Some(10), 0)), None);
        // rẽ 90 độ sau 8 giây
        assert_eq!(
            s.keep(&up("a", 8, 107_725_000, Some(900), 0)),
            Some(Reason::Turned)
        );
        // cờ luôn được giữ
        assert_eq!(
            s.keep(&up("a", 9, 107_725_000, Some(900), 2)),
            Some(Reason::Flagged)
        );
        // di chuyển > 60 m: 60 m ≈ 54.000 đơn vị
        assert_eq!(
            s.keep(&up("a", 12, 107_780_000, Some(900), 0)),
            Some(Reason::Moved)
        );
        // đứng yên 61 giây
        assert_eq!(
            s.keep(&up("a", 73, 107_780_000, Some(900), 0)),
            Some(Reason::Heartbeat)
        );
        // tài xế khác độc lập
        assert_eq!(
            s.keep(&up("b", 73, 107_780_000, None, 0)),
            Some(Reason::First)
        );
        assert_eq!(s.tracked(), 2);
    }
}
