//! Hàm mục tiêu điểm hẹn (LOC-32), đúng §3.4 của curriculum. Với điểm hẹn ứng viên `p`:
//!
//! ```text
//! J(p) = a·T_walk + b·max(T_walk, T_drive) + c·T_drive + d·T_trip + phạt
//! ```
//!
//! Đơn vị mọi số hạng là **giây**. Ràng buộc cứng loại ứng viên ([`Violation`]), phạt chỉ cộng vào `J`.
//! Bộ trọng số là một chính sách có `policy_version`; [`PolicySnapshot`] là bản chụp để lưu cùng quote.

use serde::{Deserialize, Serialize};

/// Các số hạng của `J` cho một ứng viên, tính bằng giây.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Terms {
    /// Khách đi bộ tới điểm hẹn.
    pub walk_s: f64,
    /// Tài xế chạy tới điểm hẹn (kỳ vọng nếu chưa biết tài xế).
    pub drive_s: f64,
    /// Đoạn có khách, từ điểm hẹn tới điểm trả (gồm quay đầu).
    pub trip_s: f64,
    /// Tổng phạt mềm (phía đường bất lợi, hẻm, đi bộ quá ngưỡng thoải mái...).
    pub penalty_s: f64,
}

/// Lý do loại một ứng viên (ràng buộc cứng).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Violation {
    /// Đi bộ quá `max_walk_m`.
    WalkTooFar,
    /// Cầu vượt, cầu, hầm: không được dừng.
    BridgeOrTunnel,
    /// Đường cao tốc hay nhánh nối: không được dừng.
    NoStopClass,
    /// Vòng xuyến.
    Roundabout,
    /// Cạnh quá ngắn: nằm trọn trong nút giao.
    TooShort,
    /// Ô tô (hay xe máy) không vào được.
    NoVehicleAccess,
    /// Vùng bắt buộc đón ở điểm riêng (sân bay, TTTM) mà ứng viên nằm ngoài điểm đó.
    OutsideMandatoryZone,
}

/// Phạt mềm. Mọi số là giả định của lab, chưa hiệu chuẩn bằng dữ liệu thật.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Penalties {
    /// Khách ở phía đối diện của đường rộng (phải băng qua mặt đường để lên xe), cho đường cấp cao.
    pub wrong_side_s: f64,
    /// Đi bộ vượt ngưỡng thoải mái: mỗi 100 m vượt thêm bao nhiêu giây phạt.
    pub tail_walk_s_per_100m: f64,
    pub comfort_walk_m: f64,
    /// Dừng trong hẻm (cản đường): phạt cho xe ô tô.
    pub alley_stop_s: f64,
}

impl Default for Penalties {
    fn default() -> Self {
        Self {
            wrong_side_s: 30.0,
            tail_walk_s_per_100m: 20.0,
            comfort_walk_m: 150.0,
            alley_stop_s: 20.0,
        }
    }
}

/// Bộ trọng số và ràng buộc của một chính sách điểm đón.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectiveWeights {
    /// Phiên bản chính sách, lưu vào snapshot quote (kiến trúc §4.3).
    pub policy_version: String,
    /// Khách đi bộ.
    pub a: f64,
    /// Thời gian tới lúc hai người gặp nhau, `max(T_walk, T_drive)`.
    pub b: f64,
    /// Tài xế chạy không.
    pub c: f64,
    /// Đoạn có khách.
    pub d: f64,
    pub penalties: Penalties,
    /// Ràng buộc cứng: khách không phải đi bộ quá chừng này (mét), trừ điểm ghim luôn là phương án lùi.
    pub max_walk_m: f64,
}

impl Default for ObjectiveWeights {
    /// Chính sách `p04-v1`: **một giây của khách bằng một giây của tài xế** (`a = b = c = d = 1`).
    /// Quy tắc chọn ghi ở `results/p04.md`, mục LOC-32: các trọng số khác chỉ được dùng nếu qua cổng chặn trên tập seed chỉnh.
    fn default() -> Self {
        Self {
            policy_version: "p04-v1".into(),
            a: 1.0,
            b: 1.0,
            c: 1.0,
            d: 1.0,
            penalties: Penalties::default(),
            max_walk_m: 250.0,
        }
    }
}

impl ObjectiveWeights {
    pub fn with_ratio(mut self, a_over_c: f64) -> Self {
        self.a = a_over_c;
        self.policy_version = format!("p04-v1-a{a_over_c:.3}");
        self
    }

    /// `J` từ các số hạng. Giá trị không hữu hạn (không có đường) cho `INFINITY`.
    pub fn j(&self, t: &Terms) -> f64 {
        let v = self.a * t.walk_s
            + self.b * t.walk_s.max(t.drive_s)
            + self.c * t.drive_s
            + self.d * t.trip_s
            + t.penalty_s;
        if v.is_finite() { v } else { f64::INFINITY }
    }

    /// Phạt đi bộ vượt ngưỡng thoải mái (mét → giây phạt).
    pub fn tail_walk_penalty(&self, walk_m: f64) -> f64 {
        let over = (walk_m - self.penalties.comfort_walk_m).max(0.0);
        over / 100.0 * self.penalties.tail_walk_s_per_100m
    }

    pub fn snapshot(&self) -> PolicySnapshot {
        PolicySnapshot {
            policy_version: self.policy_version.clone(),
            weights: [self.a, self.b, self.c, self.d],
            max_walk_m: self.max_walk_m,
            penalties: self.penalties,
        }
    }
}

/// Bản chụp chính sách đi cùng quote để dựng lại được vì sao chọn điểm đón này (LOC-32, LOC-40).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicySnapshot {
    pub policy_version: String,
    /// `[a, b, c, d]`.
    pub weights: [f64; 4],
    pub max_walk_m: f64,
    pub penalties: Penalties,
}

impl PolicySnapshot {
    /// Mã băm FNV-1a 64 bit của bản chụp (JSON chuẩn), để so hai quote cùng chính sách.
    pub fn fingerprint(&self) -> u64 {
        let json = serde_json::to_string(self).expect("tuần tự hoá");
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for b in json.bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j_matches_the_formula_by_hand() {
        // ví dụ tay: đi bộ 120 s, tài xế 300 s, đoạn có khách 900 s, phạt 30 s
        let w = ObjectiveWeights::default();
        let t = Terms {
            walk_s: 120.0,
            drive_s: 300.0,
            trip_s: 900.0,
            penalty_s: 30.0,
        };
        // 1·120 + 1·max(120, 300) + 1·300 + 1·900 + 30 = 1650
        assert_eq!(w.j(&t), 1650.0);
        let w2 = ObjectiveWeights {
            a: 2.0,
            b: 0.5,
            c: 1.0,
            d: 0.0,
            ..ObjectiveWeights::default()
        };
        // 2·120 + 0.5·300 + 300 + 0 + 30 = 720
        assert_eq!(w2.j(&t), 720.0);
    }

    #[test]
    fn max_term_charges_the_slower_party() {
        let w = ObjectiveWeights::default();
        let slow_driver = Terms {
            walk_s: 60.0,
            drive_s: 300.0,
            ..Terms::default()
        };
        let slow_walker = Terms {
            walk_s: 300.0,
            drive_s: 60.0,
            ..Terms::default()
        };
        // đối xứng khi a = c
        assert_eq!(w.j(&slow_driver), w.j(&slow_walker));
    }

    #[test]
    fn missing_route_is_infinite_not_nan() {
        let w = ObjectiveWeights::default();
        let t = Terms {
            drive_s: f64::INFINITY,
            ..Terms::default()
        };
        assert!(w.j(&t).is_infinite());
    }

    #[test]
    fn snapshot_fingerprint_changes_with_any_weight() {
        let a = ObjectiveWeights::default().snapshot();
        let mut w = ObjectiveWeights::default();
        assert_eq!(a.fingerprint(), w.snapshot().fingerprint());
        w.c = 1.5;
        assert_ne!(a.fingerprint(), w.snapshot().fingerprint());
        assert_eq!(a.policy_version, "p04-v1");
    }

    #[test]
    fn tail_walk_penalty_starts_after_comfort_distance() {
        let w = ObjectiveWeights::default();
        assert_eq!(w.tail_walk_penalty(100.0), 0.0);
        assert!((w.tail_walk_penalty(250.0) - 20.0).abs() < 1e-9);
    }
}
