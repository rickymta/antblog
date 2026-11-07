//! LOC-46: bất biến của `suggest` trên đồ thị fixture (không mạng): mỗi tài xế và mỗi booking tối đa một lần, mã gợi ý duy nhất,
//! không gợi ý vượt giới hạn đón xa, mọi cặp trong gợi ý đều có trong yêu cầu, hạn dùng đúng TTL.

use geo_core::LatLng;
use matcher::suggest::*;
use proptest::prelude::*;
use road_graph::fixture::{compile_fixture, generate};
use route_engine::engine::{Engine, EngineConfig};
use std::sync::OnceLock;

fn engine() -> &'static Engine {
    static E: OnceLock<Engine> = OnceLock::new();
    E.get_or_init(|| Engine::new(compile_fixture(&generate()), EngineConfig::default()))
}

fn point() -> impl Strategy<Value = (f64, f64)> {
    (10.7690f64..10.7760, 106.6905f64..106.7020)
}

fn request() -> impl Strategy<Value = SuggestRequest> {
    (
        prop::collection::vec(point(), 1..7),
        prop::collection::vec((point(), 0u64..5, prop::option::of(0u32..120_000)), 1..8),
        0u32..600,
        prop::option::of(0u32..300_000),
        any::<u8>(),
    )
        .prop_map(|(pend, drv, hard, ttl, salt)| SuggestRequest {
            request_id: format!("REQ-{salt}"),
            now_ms: 1_000_000,
            policy: PolicyKnobs {
                hard_pickup_s: hard,
                ..PolicyKnobs::default()
            },
            snapshot_version: 1,
            pending: pend
                .into_iter()
                .enumerate()
                .map(|(k, (la, ln))| PendingReq {
                    booking_id: format!("BK-{k}"),
                    pickup: LatLng::try_new(la, ln).unwrap(),
                    requested_at_ms: 999_000,
                    waited_ms: None,
                })
                .collect(),
            drivers: drv
                .into_iter()
                .enumerate()
                .map(|(k, ((la, ln), v, busy))| DriverCand {
                    driver_id: format!("DR-{k}"),
                    position: LatLng::try_new(la, ln).unwrap(),
                    state_version: v,
                    idle_since_ms: 0,
                    busy_remaining_ms: busy,
                })
                .collect(),
            ttl_ms: ttl,
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn suggestions_are_a_valid_partial_matching(r in request()) {
        validate(&r).unwrap();
        let a = suggest(engine(), &r);
        let mut bookings: Vec<&str> = a.suggestions.iter().map(|s| s.booking_id.as_str()).collect();
        let mut drivers: Vec<&str> = a.suggestions.iter().map(|s| s.driver_id.as_str()).collect();
        let mut ids: Vec<&str> = a.suggestions.iter().map(|s| s.suggestion_id.as_str()).collect();
        let n = a.suggestions.len();
        bookings.sort_unstable(); bookings.dedup();
        drivers.sort_unstable(); drivers.dedup();
        ids.sort_unstable(); ids.dedup();
        prop_assert_eq!(bookings.len(), n);
        prop_assert_eq!(drivers.len(), n);
        prop_assert_eq!(ids.len(), n);
        // mỗi booking hoặc được gợi ý hoặc nằm trong unmatched, không cả hai, không thiếu
        prop_assert_eq!(n + a.unmatched.len(), r.pending.len());
        let ttl = i64::from(clamp_ttl(r.ttl_ms));
        for s in &a.suggestions {
            prop_assert_eq!(s.valid_until_ms, r.now_ms + ttl);
            let d = r.drivers.iter().find(|d| d.driver_id == s.driver_id).expect("tài xế có trong yêu cầu");
            prop_assert_eq!(s.driver_state_version, d.state_version);
            prop_assert!(r.pending.iter().any(|p| p.booking_id == s.booking_id));
            if r.policy.hard_pickup_s > 0 {
                prop_assert!(i64::from(s.eta_ms) <= i64::from(r.policy.hard_pickup_s) * 1000,
                    "eta {} vượt giới hạn {} s", s.eta_ms, r.policy.hard_pickup_s);
            }
        }
        // lặp lại cho đúng kết quả cũ (tất định)
        prop_assert_eq!(suggest(engine(), &r).suggestions, a.suggestions);
    }

    #[test]
    fn estimated_fallback_is_also_a_valid_partial_matching(r in request()) {
        let a = suggest_estimated(&r, "deadline");
        let mut drivers: Vec<&str> = a.suggestions.iter().map(|s| s.driver_id.as_str()).collect();
        let n = a.suggestions.len();
        drivers.sort_unstable(); drivers.dedup();
        prop_assert_eq!(drivers.len(), n);
        prop_assert_eq!(n + a.unmatched.len(), r.pending.len());
        prop_assert!(n <= r.pending.len().min(r.drivers.len()));
    }
}
