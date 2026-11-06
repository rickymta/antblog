use crate::pb;
use geo_core::{LatLng, LatLngE7};
use std::time::{SystemTime, UNIX_EPOCH};

/// Epoch mili giây UTC của đồng hồ máy.
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}

/// Vị trí mới nhất đã biết của một tài xế. `accuracy/speed/heading` vắng nghĩa là app không biết (khác 0).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub pos: LatLngE7,
    /// Giờ lấy mẫu (đã quy về đồng hồ server nếu ingest có hiệu chỉnh): dùng so thứ tự điểm.
    pub occurred_ms: i64,
    /// Giờ ingest nhận lô, đồng hồ server: dùng cho TTL, không bị đồng hồ điện thoại lệch làm sai.
    pub received_ms: i64,
    pub seq: u32,
    pub accuracy_dm: Option<u32>,
    pub speed_dmps: Option<u32>,
    pub heading_ddeg: Option<u32>,
    pub flags: u32,
}

impl Position {
    pub fn latlng(&self) -> LatLng {
        self.pos.to_latlng()
    }
}

/// Một cập nhật vị trí của một tài xế (đã bóc khỏi `DriverLocationUpdated`).
#[derive(Debug, Clone, PartialEq)]
pub struct Update {
    pub driver_id: String,
    pub city_code: String,
    pub trip_id: String,
    pub pos: Position,
}

impl Update {
    /// `None` nếu thiếu `point` hoặc `driver_id` rỗng (bản ghi hỏng, bỏ qua).
    pub fn from_proto(m: &pb::DriverLocationUpdated) -> Option<Self> {
        let p = m.point.as_ref()?;
        if m.driver_id.is_empty() {
            return None;
        }
        let received = if m.received_at_ms > 0 {
            m.received_at_ms
        } else {
            m.occurred_at_ms
        };
        Some(Self {
            driver_id: m.driver_id.clone(),
            city_code: m.city_code.clone(),
            trip_id: m.trip_id.clone(),
            pos: Position {
                pos: LatLngE7::new(p.lat_e7, p.lng_e7),
                occurred_ms: m.occurred_at_ms,
                received_ms: received,
                seq: p.seq,
                accuracy_dm: p.accuracy_dm,
                speed_dmps: p.speed_dmps,
                heading_ddeg: p.heading_ddeg,
                flags: m.flags,
            },
        })
    }

    pub fn to_proto(&self) -> pb::DriverLocationUpdated {
        pb::DriverLocationUpdated {
            event_id: format!("{}-{}", self.driver_id, self.pos.seq),
            occurred_at_ms: self.pos.occurred_ms,
            received_at_ms: self.pos.received_ms,
            producer: "location-store-lab".into(),
            schema_version: 1,
            trace_id: String::new(),
            driver_id: self.driver_id.clone(),
            city_code: self.city_code.clone(),
            trip_id: self.trip_id.clone(),
            point: Some(pb::GpsPoint {
                lat_e7: self.pos.pos.lat_e7(),
                lng_e7: self.pos.pos.lng_e7(),
                device_time_ms: self.pos.occurred_ms,
                seq: self.pos.seq,
                accuracy_dm: self.pos.accuracy_dm,
                speed_dmps: self.pos.speed_dmps,
                heading_ddeg: self.pos.heading_ddeg,
                is_mock: false,
            }),
            flags: self.pos.flags,
            clock_skew_ms: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;

    fn sample(acc: Option<u32>, speed: Option<u32>) -> Update {
        Update {
            driver_id: "DR-001".into(),
            city_code: "HCM".into(),
            trip_id: String::new(),
            pos: Position {
                pos: LatLngE7::new(107_725_000, 1_066_980_000),
                occurred_ms: 1_000,
                received_ms: 1_200,
                seq: 7,
                accuracy_dm: acc,
                speed_dmps: speed,
                heading_ddeg: None,
                flags: 2,
            },
        }
    }

    #[test]
    fn proto_round_trip_keeps_absent_distinct_from_zero() {
        for (acc, speed) in [(None, None), (Some(0), Some(0)), (Some(45), None)] {
            let u = sample(acc, speed);
            let bytes = u.to_proto().encode_to_vec();
            let back = Update::from_proto(&pb::DriverLocationUpdated::decode(&bytes[..]).unwrap())
                .unwrap();
            assert_eq!(back, u);
            assert_eq!(back.pos.accuracy_dm, acc);
            assert_eq!(back.pos.speed_dmps, speed);
        }
    }

    #[test]
    fn rejects_missing_point_or_driver() {
        let mut m = sample(None, None).to_proto();
        m.point = None;
        assert!(Update::from_proto(&m).is_none());
        let mut m = sample(None, None).to_proto();
        m.driver_id.clear();
        assert!(Update::from_proto(&m).is_none());
    }
}
