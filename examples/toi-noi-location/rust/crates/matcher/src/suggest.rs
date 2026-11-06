//! LOC-46: gợi ý ghép chuyến cho Dispatch. Hàm thuần (không tonic, không đồng hồ, không trạng thái): yêu cầu → gợi ý có hạn dùng.
//!
//! Ranh giới: Rust **đề xuất**, Dispatch (C#) **quyết định**. Kết quả ở đây là lời khuyên: không giữ chỗ tài xế, không ghi assignment.
//! Dispatch phải kiểm lại bằng trạng thái của nó (`driver_state_version`, hạn `valid_until_ms`, tài xế còn rảnh, booking chưa có
//! assignment) rồi mới lấy lease và tạo offer.
//!
//! Luồng: snap điểm đón và tài xế lên đường ô tô → ma trận ETA CH (một lần `bucket_matrix`) → chi phí theo [`CostPolicy`] →
//! Hungarian → gợi ý (mỗi tài xế, mỗi booking tối đa một lần). Quá ngân sách thì [`suggest_estimated`] dựng lại bằng chim bay.

use crate::assign::hungarian;
use crate::cost::{Dense, FORBIDDEN, eta_matrix_ms};
use crate::policy::CostPolicy;
use geo_core::{LatLng, haversine_m};
use route_engine::engine::Engine;
use route_engine::position::Position;
use route_engine::profile::Profile;
use route_engine::provider::StraightLineProvider;
use std::time::Instant;

pub const DEFAULT_TTL_MS: u32 = 10_000;
pub const MIN_TTL_MS: u32 = 1_000;
pub const MAX_TTL_MS: u32 = 120_000;
pub const MAX_PENDING: usize = 2_000;
pub const MAX_DRIVERS: usize = 5_000;
pub const MAX_CELLS: usize = 250_000;

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyKnobs {
    /// Rỗng = «v0-eta».
    pub version: String,
    pub hard_pickup_s: u32,
    pub soft_pickup_s: u32,
    pub soft_weight: f64,
    pub age_bonus: f64,
    pub idle_bonus: f64,
}

impl Default for PolicyKnobs {
    fn default() -> Self {
        Self {
            version: String::new(),
            hard_pickup_s: 0,
            soft_pickup_s: 0,
            soft_weight: 0.0,
            age_bonus: 0.0,
            idle_bonus: 0.0,
        }
    }
}

impl PolicyKnobs {
    pub fn version_label(&self) -> String {
        if self.version.is_empty() {
            "v0-eta".into()
        } else {
            self.version.clone()
        }
    }
    pub fn to_policy(&self) -> CostPolicy {
        CostPolicy {
            version: self.version_label(),
            w_eta: 1.0,
            soft_pickup_ms: i64::from(self.soft_pickup_s) * 1000,
            soft_w: self.soft_weight,
            risk_ms: 0.0,
            age_bonus: self.age_bonus,
            idle_bonus: self.idle_bonus,
            hard_pickup_ms: (self.hard_pickup_s > 0).then(|| i64::from(self.hard_pickup_s) * 1000),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReq {
    pub booking_id: String,
    pub pickup: LatLng,
    pub requested_at_ms: i64,
    pub waited_ms: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DriverCand {
    pub driver_id: String,
    pub position: LatLng,
    pub state_version: u64,
    pub idle_since_ms: i64,
    /// `Some` = đang chở khách (chain): `position` là nơi dự kiến trả khách.
    pub busy_remaining_ms: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuggestRequest {
    pub request_id: String,
    pub now_ms: i64,
    pub policy: PolicyKnobs,
    pub snapshot_version: u64,
    pub pending: Vec<PendingReq>,
    pub drivers: Vec<DriverCand>,
    pub ttl_ms: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Routed,
    Estimated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnmatchedReason {
    NoDriverAvailable,
    BeyondPickupLimit,
    NoRoute,
    NotSnappable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Suggestion {
    pub suggestion_id: String,
    pub booking_id: String,
    pub driver_id: String,
    pub eta_ms: u32,
    pub distance_hint_m: u32,
    pub cost: i64,
    pub driver_state_version: u64,
    pub valid_until_ms: i64,
    pub policy_version: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SolverStats {
    pub algo: String,
    pub rows: u32,
    pub cols: u32,
    pub matrix_us: u32,
    pub solve_us: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuggestResponse {
    pub request_id: String,
    pub snapshot_version: u64,
    pub policy_version: String,
    pub suggestions: Vec<Suggestion>,
    pub unmatched: Vec<(String, UnmatchedReason)>,
    pub stats: SolverStats,
    pub quality: Quality,
    pub source: String,
}

/// Kiểm tra đầu vào (lỗi → `InvalidArgument` ở tầng dịch vụ).
pub fn validate(req: &SuggestRequest) -> Result<(), String> {
    if req.request_id.is_empty() {
        return Err("thiếu request_id".into());
    }
    if req.pending.is_empty() || req.drivers.is_empty() {
        return Err("cần ít nhất một yêu cầu đang chờ và một tài xế".into());
    }
    if req.pending.len() > MAX_PENDING || req.drivers.len() > MAX_DRIVERS {
        return Err(format!(
            "quá lớn: {} yêu cầu (tối đa {MAX_PENDING}), {} tài xế (tối đa {MAX_DRIVERS})",
            req.pending.len(),
            req.drivers.len()
        ));
    }
    if req.pending.len() * req.drivers.len() > MAX_CELLS {
        return Err(format!("ma trận vượt giới hạn {MAX_CELLS} ô"));
    }
    let dup = |ids: Vec<&str>| {
        let mut v = ids;
        v.sort_unstable();
        v.windows(2).any(|w| w[0] == w[1])
    };
    if req.pending.iter().any(|p| p.booking_id.is_empty())
        || dup(req.pending.iter().map(|p| p.booking_id.as_str()).collect())
    {
        return Err("booking_id rỗng hoặc trùng".into());
    }
    if req.drivers.iter().any(|d| d.driver_id.is_empty())
        || dup(req.drivers.iter().map(|d| d.driver_id.as_str()).collect())
    {
        return Err("driver_id rỗng hoặc trùng".into());
    }
    Ok(())
}

pub fn clamp_ttl(ttl_ms: Option<u32>) -> u32 {
    ttl_ms
        .unwrap_or(DEFAULT_TTL_MS)
        .clamp(MIN_TTL_MS, MAX_TTL_MS)
}

fn fnv1a(s: &str) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in s.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// Mã gợi ý tất định: cùng (request_id, booking, tài xế, phiên bản trạng thái tài xế) cho cùng mã.
pub fn suggestion_id(request_id: &str, booking: &str, driver: &str, state_version: u64) -> String {
    format!(
        "sg-{:016x}",
        fnv1a(&format!("{request_id}|{booking}|{driver}|{state_version}"))
    )
}

/// Ma trận ETA (ms): hàng = khách, cột = tài xế; `None` hàng nghĩa là điểm đón không snap được.
trait EtaSource {
    fn matrix(
        &self,
        req: &SuggestRequest,
        pickups_ok: &[Option<Vec<Position>>],
    ) -> (Dense, Option<Vec<bool>>);
}

struct Routed<'e>(&'e Engine);

impl EtaSource for Routed<'_> {
    fn matrix(
        &self,
        req: &SuggestRequest,
        pickups_ok: &[Option<Vec<Position>>],
    ) -> (Dense, Option<Vec<bool>>) {
        let e = self.0;
        let view = e.view(Profile::Car);
        // mỗi tài xế có một hoặc hai vị trí có hướng (đường hai chiều): lấy ETA nhỏ nhất
        let mut from: Vec<Position> = Vec::new();
        let mut owner: Vec<usize> = Vec::new();
        let mut usable = vec![false; req.drivers.len()];
        for (i, d) in req.drivers.iter().enumerate() {
            if let Ok(s) = e.snap_point(Profile::Car, d.position, "tài xế") {
                for p in s.positions(view) {
                    from.push(p);
                    owner.push(i);
                    usable[i] = true;
                }
            }
        }
        let rows_idx: Vec<usize> = (0..req.pending.len())
            .filter(|&j| pickups_ok[j].is_some())
            .collect();
        let to: Vec<&[Position]> = rows_idx
            .iter()
            .map(|&j| pickups_ok[j].as_deref().expect("đã lọc"))
            .collect();
        let m = if from.is_empty() || to.is_empty() {
            Dense::new(from.len(), to.len(), FORBIDDEN)
        } else {
            eta_matrix_ms(e, &from, &to)
        };
        let mut out = Dense::new(req.pending.len(), req.drivers.len(), FORBIDDEN);
        for (r, &j) in rows_idx.iter().enumerate() {
            for (k, &i) in owner.iter().enumerate() {
                let v = m.get(k, r);
                if v < out.get(j, i) {
                    out.set(j, i, v);
                }
            }
        }
        (out, Some(usable))
    }
}

struct Straight;

impl EtaSource for Straight {
    fn matrix(
        &self,
        req: &SuggestRequest,
        _pickups: &[Option<Vec<Position>>],
    ) -> (Dense, Option<Vec<bool>>) {
        let sl = StraightLineProvider::default();
        // ở chế độ ước lượng mọi điểm đều dùng được (không cần snap)
        let m = Dense::from_fn(req.pending.len(), req.drivers.len(), |j, i| {
            (sl.estimate_fast(Profile::Car, req.drivers[i].position, req.pending[j].pickup)
                .1
                * 1000.0)
                .round() as i64
        });
        (m, None)
    }
}

fn solve(
    req: &SuggestRequest,
    src: &dyn EtaSource,
    snap: Option<&Engine>,
    estimated: bool,
) -> SuggestResponse {
    let policy = req.policy.to_policy();
    let ttl = i64::from(clamp_ttl(req.ttl_ms));
    let pickups_ok: Vec<Option<Vec<Position>>> = req
        .pending
        .iter()
        .map(|p| {
            snap.and_then(|e| {
                let s = e.snap_point(Profile::Car, p.pickup, "điểm đón").ok()?;
                let ps = s.positions(e.view(Profile::Car));
                (!ps.is_empty()).then_some(ps)
            })
        })
        .collect();
    let t0 = Instant::now();
    let (eta, usable) = src.matrix(req, &pickups_ok);
    let matrix_us = t0.elapsed().as_micros() as u32;

    let mut costs = Dense::new(req.pending.len(), req.drivers.len(), FORBIDDEN);
    // lý do chưa ghép: có ô tới được nào không, có ô nào trong giới hạn cứng không
    let mut reachable = vec![false; req.pending.len()];
    let mut within = vec![false; req.pending.len()];
    for (j, p) in req.pending.iter().enumerate() {
        let wait_ms = p
            .waited_ms
            .map_or_else(|| (req.now_ms - p.requested_at_ms).max(0), i64::from);
        for (i, d) in req.drivers.iter().enumerate() {
            if usable.as_ref().is_some_and(|u| !u[i]) {
                continue;
            }
            let e = eta.get(j, i);
            if e >= FORBIDDEN / 2 {
                continue;
            }
            reachable[j] = true;
            let total = e + i64::from(d.busy_remaining_ms.unwrap_or(0));
            let idle_ms = (req.now_ms - d.idle_since_ms).max(0);
            if let Some(c) = policy.cost(total, wait_ms, idle_ms, 0.0) {
                within[j] = true;
                costs.set(j, i, c);
            }
        }
    }
    let ts = Instant::now();
    let pairs = hungarian(&costs);
    let solve_us = ts.elapsed().as_micros() as u32;

    let mut suggestions = Vec::with_capacity(pairs.len());
    let mut matched = vec![false; req.pending.len()];
    for &(j, i) in &pairs {
        matched[j] = true;
        let (p, d) = (&req.pending[j], &req.drivers[i]);
        suggestions.push(Suggestion {
            suggestion_id: suggestion_id(
                &req.request_id,
                &p.booking_id,
                &d.driver_id,
                d.state_version,
            ),
            booking_id: p.booking_id.clone(),
            driver_id: d.driver_id.clone(),
            eta_ms: (eta.get(j, i) + i64::from(d.busy_remaining_ms.unwrap_or(0))).max(0) as u32,
            distance_hint_m: haversine_m(d.position, p.pickup).round() as u32,
            cost: costs.get(j, i),
            driver_state_version: d.state_version,
            valid_until_ms: req.now_ms + ttl,
            policy_version: policy.version.clone(),
        });
    }
    suggestions.sort_by_key(|s| {
        req.pending
            .iter()
            .position(|p| p.booking_id == s.booking_id)
            .unwrap_or(usize::MAX)
    });
    let unmatched = req
        .pending
        .iter()
        .enumerate()
        .filter(|&(j, _)| !matched[j])
        .map(|(j, p)| {
            let why = if !estimated && pickups_ok[j].is_none() {
                UnmatchedReason::NotSnappable
            } else if !reachable[j] {
                UnmatchedReason::NoRoute
            } else if !within[j] {
                UnmatchedReason::BeyondPickupLimit
            } else {
                UnmatchedReason::NoDriverAvailable
            };
            (p.booking_id.clone(), why)
        })
        .collect();
    SuggestResponse {
        request_id: req.request_id.clone(),
        snapshot_version: req.snapshot_version,
        policy_version: policy.version,
        suggestions,
        unmatched,
        stats: SolverStats {
            algo: if estimated {
                "hungarian-estimated"
            } else {
                "hungarian"
            }
            .into(),
            rows: req.pending.len() as u32,
            cols: req.drivers.len() as u32,
            matrix_us,
            solve_us,
        },
        quality: if estimated {
            Quality::Estimated
        } else {
            Quality::Routed
        },
        source: if estimated {
            "straight-line"
        } else {
            "ch-bucket"
        }
        .into(),
    }
}

/// Gợi ý theo ETA đường bộ (CH). Giả định `validate(req)` đã qua.
pub fn suggest(engine: &Engine, req: &SuggestRequest) -> SuggestResponse {
    solve(req, &Routed(engine), Some(engine), false)
}

/// Dự phòng khi hết ngân sách: cùng thuật toán trên ETA chim bay × hệ số vòng. Rẻ, không đụng đồ thị.
pub fn suggest_estimated(req: &SuggestRequest, source: &str) -> SuggestResponse {
    let mut r = solve(req, &Straight, None, true);
    r.source = format!("straight-line:{source}");
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use road_graph::fixture::{compile_fixture, generate};
    use route_engine::engine::EngineConfig;

    pub fn engine() -> Engine {
        Engine::new(compile_fixture(&generate()), EngineConfig::default())
    }

    pub fn ll(lat: f64, lng: f64) -> LatLng {
        LatLng::try_new(lat, lng).unwrap()
    }

    pub fn req(n_pending: usize, n_drivers: usize) -> SuggestRequest {
        // lưới nhỏ trong khu mẫu (10.7690–10.7760, 106.6905–106.7020)
        let pending = (0..n_pending)
            .map(|k| PendingReq {
                booking_id: format!("BK-{k:03}"),
                pickup: ll(10.7700 + 0.0006 * k as f64, 106.6910 + 0.0009 * k as f64),
                requested_at_ms: 1_000 * k as i64,
                waited_ms: None,
            })
            .collect();
        let drivers = (0..n_drivers)
            .map(|k| DriverCand {
                driver_id: format!("DR-{k:03}"),
                position: ll(10.7695 + 0.0005 * k as f64, 106.7015 - 0.0008 * k as f64),
                state_version: 7 + k as u64,
                idle_since_ms: 0,
                busy_remaining_ms: None,
            })
            .collect();
        SuggestRequest {
            request_id: "REQ-1".into(),
            now_ms: 100_000,
            policy: PolicyKnobs::default(),
            snapshot_version: 42,
            pending,
            drivers,
            ttl_ms: None,
        }
    }

    #[test]
    fn suggestions_are_valid_deterministic_and_carry_ttl() {
        let e = engine();
        let r = req(3, 4);
        validate(&r).unwrap();
        let a = suggest(&e, &r);
        let b = suggest(&e, &r);
        assert_eq!(a.suggestions, b.suggestions);
        assert_eq!(a.suggestions.len(), 3);
        assert_eq!(a.quality, Quality::Routed);
        assert_eq!(a.policy_version, "v0-eta");
        for s in &a.suggestions {
            assert_eq!(s.valid_until_ms, 100_000 + 10_000);
            assert!(s.eta_ms > 0 && s.suggestion_id.starts_with("sg-"));
            let d = r
                .drivers
                .iter()
                .find(|d| d.driver_id == s.driver_id)
                .unwrap();
            assert_eq!(s.driver_state_version, d.state_version);
        }
        // đổi request_id → đổi suggestion_id
        let mut r2 = r.clone();
        r2.request_id = "REQ-2".into();
        assert_ne!(
            suggest(&e, &r2).suggestions[0].suggestion_id,
            a.suggestions[0].suggestion_id
        );
    }

    #[test]
    fn more_customers_than_drivers_leaves_the_rest_unmatched() {
        let e = engine();
        let a = suggest(&e, &req(5, 2));
        assert_eq!(a.suggestions.len(), 2);
        assert_eq!(a.unmatched.len(), 3);
        assert!(
            a.unmatched
                .iter()
                .all(|u| u.1 == UnmatchedReason::NoDriverAvailable)
        );
    }

    #[test]
    fn hard_limit_removes_far_pairs_and_explains() {
        let e = engine();
        let mut r = req(2, 2);
        r.policy.hard_pickup_s = 1; // 1 giây: không cặp nào đủ gần
        let a = suggest(&e, &r);
        assert!(a.suggestions.is_empty());
        assert!(
            a.unmatched
                .iter()
                .all(|u| u.1 == UnmatchedReason::BeyondPickupLimit)
        );
    }

    #[test]
    fn off_road_pickup_is_not_snappable() {
        let e = engine();
        let mut r = req(2, 2);
        r.pending[1].pickup = ll(10.9, 106.9);
        let a = suggest(&e, &r);
        assert!(
            a.unmatched
                .contains(&("BK-001".into(), UnmatchedReason::NotSnappable))
        );
    }

    #[test]
    fn estimated_fallback_still_returns_a_valid_assignment() {
        let r = req(3, 3);
        let a = suggest_estimated(&r, "deadline");
        assert_eq!(a.quality, Quality::Estimated);
        assert_eq!(a.source, "straight-line:deadline");
        assert_eq!(a.suggestions.len(), 3);
    }

    #[test]
    fn busy_driver_adds_its_remaining_time_to_eta() {
        let e = engine();
        let mut r = req(1, 1);
        let base = suggest(&e, &r).suggestions[0].eta_ms;
        r.drivers[0].busy_remaining_ms = Some(60_000);
        let chained = suggest(&e, &r).suggestions[0].eta_ms;
        assert_eq!(chained, base + 60_000);
    }

    #[test]
    fn validation_rejects_bad_requests() {
        let mut r = req(2, 2);
        r.request_id.clear();
        assert!(validate(&r).is_err());
        let mut r = req(2, 2);
        r.drivers[1].driver_id = r.drivers[0].driver_id.clone();
        assert!(validate(&r).is_err());
        let mut r = req(2, 2);
        r.pending.clear();
        assert!(validate(&r).is_err());
    }
}
