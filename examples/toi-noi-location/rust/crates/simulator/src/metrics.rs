//! Chỉ số §3.4 của curriculum: `walk_m`, `deadhead_m`, `pickup_eta_s` p50/p95, `meet_gap_s`, `trip_m`, `u_turns`, tỉ lệ huỷ.
//!
//! Định nghĩa (cố định để mọi bài so sánh cùng một thước đo):
//! - `walk_m`: quãng khách đi bộ từ chỗ đứng thật tới điểm đón (theo đồ thị đi bộ).
//! - `deadhead_m`: quãng tài xế chạy không từ lúc nhận chuyến tới điểm đón.
//! - `pickup_eta_s`: từ lúc gán tài xế tới lúc tài xế tới điểm đón. `wait_s` là từ lúc khách đặt tới lúc tài xế tới.
//! - `meet_gap_s`: |giờ tài xế tới − giờ khách tới điểm đón|.
//! - `trip_m`: quãng đường có khách, từ điểm đón tới điểm trả.
//! - `u_turns`: số lần quay đầu (định nghĩa hình học của `route-engine::turn`) trên cả chặng đón lẫn chặng có khách.
//! - tỉ lệ huỷ: mọi khách huỷ (sau khi gán, hết kiên nhẫn chưa có xe, hoặc không có đường) trên số khách đặt.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Outcome {
    Completed,
    CancelledAfterAssign,
    CancelledNoDriver,
    CancelledNoRoute,
}

impl Outcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Completed => "completed",
            Outcome::CancelledAfterAssign => "cancelled_after_assign",
            Outcome::CancelledNoDriver => "cancelled_no_driver",
            Outcome::CancelledNoRoute => "cancelled_no_route",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TripRecord {
    pub customer: u32,
    pub driver: Option<u32>,
    pub request_s: f64,
    pub outcome: Outcome,
    pub walk_m: f64,
    /// Chuyến đã hoàn thành: độ dài tuyến đón. Chuyến huỷ sau khi gán: quãng đã chạy tới lúc huỷ.
    pub deadhead_m: f64,
    pub pickup_eta_s: f64,
    pub wait_s: f64,
    pub meet_gap_s: f64,
    pub trip_m: f64,
    pub trip_s: f64,
    pub u_turns: u32,
    /// Khoảng cách từ điểm ghim tới đường (chẩn đoán, không thuộc §3.4).
    pub pin_snap_m: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct Dist {
    pub n: usize,
    pub mean: f64,
    pub p50: f64,
    pub p95: f64,
    pub max: f64,
}

/// Phân vị nearest-rank trên bản sao đã sắp xếp.
pub fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let idx = ((p / 100.0) * sorted.len() as f64).ceil() as usize;
    sorted[idx.saturating_sub(1).min(sorted.len() - 1)]
}

pub fn dist(values: impl Iterator<Item = f64>) -> Dist {
    let mut v: Vec<f64> = values.collect();
    if v.is_empty() {
        return Dist::default();
    }
    v.sort_by(f64::total_cmp);
    Dist {
        n: v.len(),
        mean: v.iter().sum::<f64>() / v.len() as f64,
        p50: percentile(&v, 50.0),
        p95: percentile(&v, 95.0),
        max: *v.last().expect("không rỗng"),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub requests: usize,
    pub completed: usize,
    pub cancelled_after_assign: usize,
    pub cancelled_no_driver: usize,
    pub cancelled_no_route: usize,
    pub cancel_rate: f64,
    pub walk_m: Dist,
    pub deadhead_m: Dist,
    pub pickup_eta_s: Dist,
    pub wait_s: Dist,
    pub meet_gap_s: Dist,
    pub trip_m: Dist,
    pub u_turns_total: u32,
    pub u_turns_per_trip: f64,
    pub deadhead_total_m: f64,
    pub trip_total_m: f64,
    /// Quãng chạy không đã bỏ phí vì khách huỷ sau khi gán.
    pub wasted_deadhead_m: f64,
    /// `deadhead / (deadhead + trip)` của các chuyến hoàn thành.
    pub deadhead_share: f64,
    pub pin_snap_m: Dist,
    pub sim_end_s: f64,
    pub driver_busy_share: f64,
}

pub fn summarize(
    trips: &[TripRecord],
    requests: usize,
    sim_end_s: f64,
    driver_busy_s: f64,
    drivers: usize,
) -> Summary {
    let done: Vec<&TripRecord> = trips
        .iter()
        .filter(|t| t.outcome == Outcome::Completed)
        .collect();
    let count = |o: Outcome| trips.iter().filter(|t| t.outcome == o).count();
    let cancelled = trips.len() - done.len();
    let dead: f64 = done.iter().map(|t| t.deadhead_m).sum();
    let trip: f64 = done.iter().map(|t| t.trip_m).sum();
    let u: u32 = done.iter().map(|t| t.u_turns).sum();
    Summary {
        requests,
        completed: done.len(),
        cancelled_after_assign: count(Outcome::CancelledAfterAssign),
        cancelled_no_driver: count(Outcome::CancelledNoDriver),
        cancelled_no_route: count(Outcome::CancelledNoRoute),
        cancel_rate: if requests == 0 {
            0.0
        } else {
            cancelled as f64 / requests as f64
        },
        walk_m: dist(done.iter().map(|t| t.walk_m)),
        deadhead_m: dist(done.iter().map(|t| t.deadhead_m)),
        pickup_eta_s: dist(done.iter().map(|t| t.pickup_eta_s)),
        wait_s: dist(done.iter().map(|t| t.wait_s)),
        meet_gap_s: dist(done.iter().map(|t| t.meet_gap_s)),
        trip_m: dist(done.iter().map(|t| t.trip_m)),
        u_turns_total: u,
        u_turns_per_trip: if done.is_empty() {
            0.0
        } else {
            f64::from(u) / done.len() as f64
        },
        deadhead_total_m: dead,
        trip_total_m: trip,
        wasted_deadhead_m: trips
            .iter()
            .filter(|t| t.outcome == Outcome::CancelledAfterAssign)
            .map(|t| t.deadhead_m)
            .sum(),
        deadhead_share: if dead + trip > 0.0 {
            dead / (dead + trip)
        } else {
            0.0
        },
        pin_snap_m: dist(trips.iter().map(|t| t.pin_snap_m)),
        sim_end_s,
        driver_busy_share: if sim_end_s > 0.0 && drivers > 0 {
            driver_busy_s / (sim_end_s * drivers as f64)
        } else {
            0.0
        },
    }
}

/// FNV-1a 64 bit, để một con số đại diện cho cả nhật ký chuyến khi so «byte-identical».
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

pub fn trips_csv(trips: &[TripRecord]) -> String {
    let mut s = String::from(
        "customer,driver,request_s,outcome,walk_m,deadhead_m,pickup_eta_s,wait_s,meet_gap_s,trip_m,trip_s,u_turns,pin_snap_m\n",
    );
    for t in trips {
        s += &format!(
            "{},{},{:.1},{},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{},{:.1}\n",
            t.customer,
            t.driver.map_or("-".to_string(), |d| d.to_string()),
            t.request_s,
            t.outcome.as_str(),
            t.walk_m,
            t.deadhead_m,
            t.pickup_eta_s,
            t.wait_s,
            t.meet_gap_s,
            t.trip_m,
            t.trip_s,
            t.u_turns,
            t.pin_snap_m
        );
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(c: u32, o: Outcome, dead: f64) -> TripRecord {
        TripRecord {
            customer: c,
            driver: Some(1),
            request_s: 0.0,
            outcome: o,
            walk_m: 0.0,
            deadhead_m: dead,
            pickup_eta_s: dead / 5.0,
            wait_s: dead / 5.0,
            meet_gap_s: 1.0,
            trip_m: 1000.0,
            trip_s: 200.0,
            u_turns: 1,
            pin_snap_m: 3.0,
        }
    }

    #[test]
    fn percentile_is_nearest_rank() {
        let v = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        assert_eq!(percentile(&v, 50.0), 5.0);
        assert_eq!(percentile(&v, 95.0), 10.0);
        assert!(percentile(&[], 50.0).is_nan());
    }

    #[test]
    fn summary_counts_cancellations_and_waste() {
        let trips = vec![
            rec(0, Outcome::Completed, 500.0),
            rec(1, Outcome::Completed, 1500.0),
            rec(2, Outcome::CancelledAfterAssign, 300.0),
            rec(3, Outcome::CancelledNoDriver, 0.0),
        ];
        let s = summarize(&trips, 4, 1000.0, 2000.0, 2);
        assert_eq!(s.completed, 2);
        assert!((s.cancel_rate - 0.5).abs() < 1e-12);
        assert_eq!(s.wasted_deadhead_m, 300.0);
        assert_eq!(s.deadhead_m.mean, 1000.0);
        assert_eq!(s.u_turns_total, 2);
        assert!((s.deadhead_share - 2000.0 / 4000.0).abs() < 1e-12);
        assert!((s.driver_busy_share - 1.0).abs() < 1e-12);
    }

    #[test]
    fn csv_has_header_and_one_line_per_record() {
        let csv = trips_csv(&[
            rec(0, Outcome::Completed, 1.0),
            rec(1, Outcome::Completed, 2.0),
        ]);
        assert_eq!(csv.lines().count(), 3);
        assert!(csv.starts_with("customer,driver"));
        assert_eq!(fnv1a(csv.as_bytes()), fnv1a(csv.clone().as_bytes()));
    }
}
