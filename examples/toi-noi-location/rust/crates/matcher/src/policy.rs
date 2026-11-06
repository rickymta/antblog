//! LOC-45: chi phí ghép ngoài quãng đường, có phiên bản.
//!
//! Chi phí của cặp (khách `j`, tài xế `i`), đơn vị mili giây «tương đương thời gian đón»:
//!
//! ```text
//! cost = w_eta·ETA                                    thời gian tài xế tới điểm đón
//!      + soft_w·max(0, ETA − soft_pickup)             đón xa bị phạt thêm (đoạn vượt ngưỡng mềm)
//!      + risk_ms·P(huỷ | ETA, đi bộ)                  xác suất huỷ là đầu vào, nhân với «giá» một lần huỷ
//!      − age_bonus·(đã chờ của khách)                 khách chờ lâu được ưu tiên khi thiếu xe
//!      − idle_bonus·(đã rảnh của tài xế)              tài xế rảnh lâu được ưu tiên khi thừa xe
//! cấm cặp nếu ETA > hard_pickup                       giới hạn đón xa
//! ```
//!
//! Hai số hạng thưởng chỉ phụ thuộc một phía của cặp nên chỉ đổi **ai được ghép** khi một phía thừa (ma trận chữ nhật),
//! không đổi cách ghép giữa các phía đã được chọn. Bộ trọng số là chính sách có `version`, ghi cùng mọi kết quả đo.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CostPolicy {
    pub version: String,
    pub w_eta: f64,
    pub soft_pickup_ms: i64,
    pub soft_w: f64,
    /// Phạt (ms) cho một lần huỷ; 0 = không dùng xác suất huỷ.
    pub risk_ms: f64,
    pub age_bonus: f64,
    pub idle_bonus: f64,
    /// `None` = không giới hạn.
    pub hard_pickup_ms: Option<i64>,
}

impl CostPolicy {
    /// Phiên bản gốc: chỉ ETA. Mọi bài trước LOC-45 dùng bản này.
    pub fn eta_only() -> Self {
        Self {
            version: "v0-eta".into(),
            w_eta: 1.0,
            soft_pickup_ms: 0,
            soft_w: 0.0,
            risk_ms: 0.0,
            age_bonus: 0.0,
            idle_bonus: 0.0,
            hard_pickup_ms: None,
        }
    }

    /// Chi phí một cặp (ms, số nguyên); `None` nếu cặp bị cấm (vượt giới hạn đón xa).
    /// `p_cancel`: xác suất huỷ ước lượng (đầu vào); `wait_ms`: khách đã chờ; `idle_ms`: tài xế đã rảnh.
    pub fn cost(&self, eta_ms: i64, wait_ms: i64, idle_ms: i64, p_cancel: f64) -> Option<i64> {
        if self.hard_pickup_ms.is_some_and(|h| eta_ms > h) {
            return None;
        }
        let over = (eta_ms - self.soft_pickup_ms).max(0) as f64;
        let c = self.w_eta * eta_ms as f64 + self.soft_w * over + self.risk_ms * p_cancel
            - self.age_bonus * wait_ms as f64
            - self.idle_bonus * idle_ms as f64;
        Some(c.round() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eta_only_is_just_the_eta() {
        let p = CostPolicy::eta_only();
        assert_eq!(p.cost(1234, 99_999, 99_999, 0.9), Some(1234));
    }

    #[test]
    fn hard_limit_forbids_and_bonuses_reduce_cost() {
        let p = CostPolicy {
            hard_pickup_ms: Some(600_000),
            age_bonus: 0.5,
            idle_bonus: 0.1,
            ..CostPolicy::eta_only()
        };
        assert_eq!(p.cost(600_001, 0, 0, 0.0), None);
        assert_eq!(
            p.cost(100_000, 20_000, 50_000, 0.0),
            Some(100_000 - 10_000 - 5_000)
        );
    }

    #[test]
    fn soft_limit_and_risk_add_cost() {
        let p = CostPolicy {
            soft_pickup_ms: 300_000,
            soft_w: 1.0,
            risk_ms: 600_000.0,
            ..CostPolicy::eta_only()
        };
        assert_eq!(
            p.cost(400_000, 0, 0, 0.25),
            Some(400_000 + 100_000 + 150_000)
        );
    }
}
