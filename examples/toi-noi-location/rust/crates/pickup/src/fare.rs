//! Cước và quote khi đổi điểm đón (LOC-40). Cùng mô hình cước với các loạt khác: 12.000 VND cho 2 km đầu,
//! 4.500 VND mỗi km vượt, tính bằng số nguyên đồng.
//!
//! Hợp đồng với Pricing (C#): quote là một **snapshot bất biến**; đổi điểm đón chỉ tạo *đề xuất* (`Proposal`).
//! Cước chỉ được giảm tự động. Cước tăng thì khách phải đồng ý rõ ràng, nếu không khách trả đúng cước đã báo
//! (**không tăng giá lén**), phần chênh nền tảng chịu.

use crate::objective::PolicySnapshot;
use geo_core::LatLng;
use serde::{Deserialize, Serialize};

pub const BASE_FARE_VND: u64 = 12_000;
pub const BASE_METERS: u64 = 2_000;
pub const PER_KM_VND: u64 = 4_500;

/// Cước theo quãng đường (mét): `12000 + (extraMeters·4500 + 500) / 1000` với `extraMeters = max(0, m − 2000)`.
pub fn fare_vnd(distance_m: u64) -> u64 {
    let extra = distance_m.saturating_sub(BASE_METERS);
    BASE_FARE_VND + (extra * PER_KM_VND + 500) / 1000
}

/// Snapshot quote: những gì khách đã thấy và đồng ý lúc đặt xe. Không sửa sau khi tạo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuoteSnapshot {
    pub quote_id: String,
    pub pickup: [f64; 2],
    pub dropoff: [f64; 2],
    pub est_distance_m: u64,
    pub fare_vnd: u64,
    pub policy: PolicySnapshot,
}

impl QuoteSnapshot {
    pub fn new(
        quote_id: &str,
        pickup: LatLng,
        dropoff: LatLng,
        est_distance_m: u64,
        policy: PolicySnapshot,
    ) -> Self {
        Self {
            quote_id: quote_id.into(),
            pickup: [pickup.lat(), pickup.lng()],
            dropoff: [dropoff.lat(), dropoff.lng()],
            est_distance_m,
            fare_vnd: fare_vnd(est_distance_m),
            policy,
        }
    }
}

/// Kết luận về cước khi đề xuất đổi điểm đón.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FareEffect {
    /// Cước không đổi (cùng bậc làm tròn).
    Same,
    /// Cước giảm: áp dụng tự động, thông báo cho khách.
    Lower,
    /// Cước tăng: cần khách đồng ý rõ ràng.
    NeedsConsent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Proposal {
    pub quote_id: String,
    pub old_fare_vnd: u64,
    pub new_fare_vnd: u64,
    /// Chênh lệch có dấu (đồng).
    pub delta_vnd: i64,
    pub extra_walk_m: i64,
    pub effect: FareEffect,
    pub policy_version: String,
}

/// Đề xuất đổi điểm đón: `new_distance_m` là quãng có khách tính lại từ điểm đón mới tới điểm trả.
pub fn propose_move(q: &QuoteSnapshot, new_distance_m: u64, extra_walk_m: i64) -> Proposal {
    let new_fare = fare_vnd(new_distance_m);
    let delta = new_fare as i64 - q.fare_vnd as i64;
    Proposal {
        quote_id: q.quote_id.clone(),
        old_fare_vnd: q.fare_vnd,
        new_fare_vnd: new_fare,
        delta_vnd: delta,
        extra_walk_m,
        effect: match delta {
            0 => FareEffect::Same,
            d if d < 0 => FareEffect::Lower,
            _ => FareEffect::NeedsConsent,
        },
        policy_version: q.policy.policy_version.clone(),
    }
}

/// Kết quả thanh toán sau khi biết khách có đồng ý hay không.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settlement {
    pub charged_vnd: u64,
    /// Phần cước nền tảng chịu thay khách (không bao giờ tính vào khách).
    pub absorbed_vnd: u64,
    pub moved: bool,
}

/// `consent`: `Some(true)` khách đồng ý đổi điểm (và cước), `Some(false)` từ chối, `None` chưa trả lời.
/// `actual_distance_m` là quãng thật của chuyến. Nếu chưa có đồng ý thì khách không bao giờ trả hơn cước đã báo.
pub fn settle(q: &QuoteSnapshot, actual_distance_m: u64, consent: Option<bool>) -> Settlement {
    let actual = fare_vnd(actual_distance_m);
    match consent {
        Some(true) => Settlement {
            charged_vnd: actual,
            absorbed_vnd: 0,
            moved: true,
        },
        _ => {
            let charged = actual.min(q.fare_vnd);
            Settlement {
                charged_vnd: charged,
                absorbed_vnd: actual - charged,
                moved: consent.is_some_and(|c| c),
            }
        }
    }
}

/// Câu chữ cho khách (tiếng Việt), kèm số liệu của đề xuất. Không dùng từ ép buộc, luôn nói rõ cước.
pub fn message_for_customer(p: &Proposal) -> String {
    let vnd = |n: u64| format!("{} đồng", crate::fmt_thousands(n));
    match p.effect {
        FareEffect::Same => format!(
            "Chúng tôi đề xuất đổi điểm đón để tài xế tới nhanh hơn, bạn đi bộ thêm {} m. Cước giữ nguyên {}.",
            p.extra_walk_m.max(0),
            vnd(p.new_fare_vnd)
        ),
        FareEffect::Lower => format!(
            "Chúng tôi đề xuất đổi điểm đón, bạn đi bộ thêm {} m. Cước giảm từ {} còn {}.",
            p.extra_walk_m.max(0),
            vnd(p.old_fare_vnd),
            vnd(p.new_fare_vnd)
        ),
        FareEffect::NeedsConsent => format!(
            "Điểm đón mới làm cước tăng từ {} lên {} (bạn đi bộ thêm {} m). Bạn chỉ bị tính cước mới nếu bấm đồng ý; nếu không, cước vẫn là {}.",
            vnd(p.old_fare_vnd),
            vnd(p.new_fare_vnd),
            p.extra_walk_m.max(0),
            vnd(p.old_fare_vnd)
        ),
    }
}

/// Câu chữ cho tài xế: nói rõ điểm đón đổi và quãng chạy thay đổi, không nói về cước của khách.
pub fn message_for_driver(delta_deadhead_m: i64) -> String {
    if delta_deadhead_m < 0 {
        format!(
            "Điểm đón đã đổi: bạn chạy ít hơn {} m tới chỗ khách.",
            -delta_deadhead_m
        )
    } else {
        format!("Điểm đón đã đổi: bạn chạy thêm {delta_deadhead_m} m tới chỗ khách.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objective::ObjectiveWeights;

    fn quote(m: u64) -> QuoteSnapshot {
        let p = LatLng::try_new(10.77, 106.69).unwrap();
        QuoteSnapshot::new("Q-1", p, p, m, ObjectiveWeights::default().snapshot())
    }

    #[test]
    fn fare_model_matches_hand_values() {
        assert_eq!(fare_vnd(0), 12_000);
        assert_eq!(fare_vnd(2_000), 12_000);
        assert_eq!(fare_vnd(3_000), 12_000 + 4_500);
        assert_eq!(fare_vnd(2_001), 12_000 + 5); // (1·4500 + 500)/1000 = 5
        assert_eq!(fare_vnd(6_392), 12_000 + (4_392 * 4_500 + 500) / 1_000);
    }

    #[test]
    fn rising_fare_needs_consent_and_is_never_charged_without_it() {
        let q = quote(5_000);
        let p = propose_move(&q, 5_800, 80);
        assert_eq!(p.effect, FareEffect::NeedsConsent);
        assert!(p.delta_vnd > 0);
        let s = settle(&q, 5_800, None);
        assert_eq!(s.charged_vnd, q.fare_vnd);
        assert_eq!(s.absorbed_vnd, p.delta_vnd as u64);
        let s = settle(&q, 5_800, Some(false));
        assert_eq!(s.charged_vnd, q.fare_vnd);
        let s = settle(&q, 5_800, Some(true));
        assert_eq!(s.charged_vnd, p.new_fare_vnd);
    }

    #[test]
    fn lower_fare_applies_automatically() {
        let q = quote(5_000);
        let p = propose_move(&q, 4_200, 30);
        assert_eq!(p.effect, FareEffect::Lower);
        assert!(settle(&q, 4_200, None).charged_vnd < q.fare_vnd);
    }

    #[test]
    fn messages_state_the_fare_plainly() {
        let q = quote(5_000);
        let m = message_for_customer(&propose_move(&q, 5_800, 80));
        assert!(m.contains("chỉ bị tính cước mới nếu bấm đồng ý"));
        assert!(m.contains("đồng"));
    }
}
