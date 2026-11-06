//! Dịch vụ gRPC `MatchingService.SuggestMatch` (LOC-46): Rust **đề xuất**, Dispatch (C#) **quyết định**.
//!
//! Dịch vụ này không giữ trạng thái assignment, không lease, không gửi offer: nó nhận ảnh chụp bể tài xế và danh sách booking đang chờ
//! từ Dispatch, trả về gợi ý có hạn dùng ([`matcher::suggest`]). Luồng xử lý giống `RoutingSvc`:
//! 1. kiểm tra đầu vào → `InvalidArgument`;
//! 2. ngân sách = deadline gRPC trừ dự trữ, rồi lấy nhỏ hơn `compute_budget_ms` nếu client đặt;
//! 3. tính trong `spawn_blocking` sau khi xin giấy phép; hết ngân sách (hoặc quá tải) thì **không báo lỗi**, trả gợi ý dựng từ ETA
//!    chim bay với `quality = ESTIMATED` để Dispatch tự quyết có dùng hay không.

use crate::pb::matching_service_server::{MatchingService, MatchingServiceServer};
use crate::pb::{
    self, LatLngE7, RoutingQuality, SolverStats, SuggestMatchRequest, SuggestMatchResponse,
};
use crate::service::{GeoEngineConfig, parse_grpc_timeout};
use geo_core::LatLng;
use matcher::suggest::{
    self, DriverCand, PendingReq, PolicyKnobs, Quality, SuggestRequest, SuggestResponse,
    UnmatchedReason,
};
use route_engine::engine::Engine;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::Semaphore;
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct MatchingMetrics {
    pub requests: AtomicU64,
    pub routed: AtomicU64,
    pub estimated_budget: AtomicU64,
    pub estimated_deadline: AtomicU64,
    pub estimated_overload: AtomicU64,
    pub invalid: AtomicU64,
}

#[derive(Clone)]
pub struct MatchingSvc {
    pub engine: Arc<Engine>,
    pub cfg: Arc<GeoEngineConfig>,
    pub metrics: Arc<MatchingMetrics>,
    permits: Arc<Semaphore>,
}

fn point(p: Option<&LatLngE7>, what: &str) -> Result<LatLng, Status> {
    let p = p.ok_or_else(|| Status::invalid_argument(format!("thiếu {what}")))?;
    LatLng::try_new(f64::from(p.lat_e7) / 1e7, f64::from(p.lng_e7) / 1e7)
        .map_err(|e| Status::invalid_argument(format!("{what} không hợp lệ: {e}")))
}

fn convert(req: &SuggestMatchRequest) -> Result<SuggestRequest, Status> {
    let policy = req
        .policy
        .as_ref()
        .map_or_else(PolicyKnobs::default, |p| PolicyKnobs {
            version: p.version.clone(),
            hard_pickup_s: p.hard_pickup_s,
            soft_pickup_s: p.soft_pickup_s,
            soft_weight: p.soft_weight,
            age_bonus: p.age_bonus,
            idle_bonus: p.idle_bonus,
        });
    let mut pending = Vec::with_capacity(req.pending.len());
    for p in &req.pending {
        pending.push(PendingReq {
            booking_id: p.booking_id.clone(),
            pickup: point(p.pickup.as_ref(), "điểm đón")?,
            requested_at_ms: p.requested_at_ms,
            waited_ms: p.waited_ms,
        });
    }
    let mut drivers = Vec::with_capacity(req.drivers.len());
    for d in &req.drivers {
        drivers.push(DriverCand {
            driver_id: d.driver_id.clone(),
            position: point(d.position.as_ref(), "vị trí tài xế")?,
            state_version: d.state_version,
            idle_since_ms: d.idle_since_ms,
            busy_remaining_ms: d.busy_remaining_ms,
        });
    }
    let r = SuggestRequest {
        request_id: req.request_id.clone(),
        now_ms: req.now_ms,
        policy,
        snapshot_version: req.snapshot_version,
        pending,
        drivers,
        ttl_ms: req.suggestion_ttl_ms,
    };
    suggest::validate(&r).map_err(Status::invalid_argument)?;
    Ok(r)
}

fn to_pb(r: SuggestResponse) -> SuggestMatchResponse {
    SuggestMatchResponse {
        request_id: r.request_id,
        snapshot_version: r.snapshot_version,
        policy_version: r.policy_version,
        suggestions: r
            .suggestions
            .into_iter()
            .map(|s| pb::Suggestion {
                suggestion_id: s.suggestion_id,
                booking_id: s.booking_id,
                driver_id: s.driver_id,
                eta_ms: s.eta_ms,
                distance_hint_m: s.distance_hint_m,
                cost: s.cost,
                driver_state_version: s.driver_state_version,
                valid_until_ms: s.valid_until_ms,
                policy_version: s.policy_version,
            })
            .collect(),
        unmatched: r
            .unmatched
            .into_iter()
            .map(|(booking_id, why)| pb::Unmatched {
                booking_id,
                reason: match why {
                    UnmatchedReason::NoDriverAvailable => pb::UnmatchedReason::NoDriverAvailable,
                    UnmatchedReason::BeyondPickupLimit => pb::UnmatchedReason::BeyondPickupLimit,
                    UnmatchedReason::NoRoute => pb::UnmatchedReason::NoRoute,
                    UnmatchedReason::NotSnappable => pb::UnmatchedReason::NotSnappable,
                } as i32,
            })
            .collect(),
        stats: Some(SolverStats {
            algo: r.stats.algo,
            rows: r.stats.rows,
            cols: r.stats.cols,
            matrix_us: r.stats.matrix_us,
            solve_us: r.stats.solve_us,
        }),
        quality: match r.quality {
            Quality::Routed => RoutingQuality::Routed,
            Quality::Estimated => RoutingQuality::Estimated,
        } as i32,
        source: r.source,
    }
}

impl MatchingSvc {
    pub fn new(engine: Arc<Engine>, cfg: GeoEngineConfig) -> Self {
        Self {
            permits: Arc::new(Semaphore::new(cfg.max_inflight.max(1))),
            cfg: Arc::new(cfg),
            metrics: Arc::new(MatchingMetrics::default()),
            engine,
        }
    }

    pub fn into_server(self) -> MatchingServiceServer<Self> {
        MatchingServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl MatchingService for MatchingSvc {
    async fn suggest_match(
        &self,
        request: Request<SuggestMatchRequest>,
    ) -> Result<Response<SuggestMatchResponse>, Status> {
        self.metrics.requests.fetch_add(1, Ordering::Relaxed);
        let client = request
            .metadata()
            .get("grpc-timeout")
            .and_then(|v| v.to_str().ok())
            .and_then(parse_grpc_timeout);
        let total = client.unwrap_or(Duration::from_millis(self.cfg.default_deadline_ms));
        let mut budget = total.saturating_sub(Duration::from_millis(self.cfg.reserve_ms));
        let req = request.into_inner();
        let explicit = req
            .compute_budget_ms
            .map(|ms| Duration::from_millis(u64::from(ms)));
        if let Some(b) = explicit {
            budget = budget.min(b);
        }
        let parsed = convert(&req).inspect_err(|_| {
            self.metrics.invalid.fetch_add(1, Ordering::Relaxed);
        })?;

        let estimated = |why: &str, c: &AtomicU64| {
            c.fetch_add(1, Ordering::Relaxed);
            Ok(Response::new(to_pb(suggest::suggest_estimated(
                &parsed, why,
            ))))
        };
        if budget.is_zero() {
            return if explicit.is_some_and(|b| b.is_zero()) {
                estimated("budget", &self.metrics.estimated_budget)
            } else {
                estimated("deadline", &self.metrics.estimated_deadline)
            };
        }
        let started = std::time::Instant::now();
        let permit =
            match tokio::time::timeout(budget, Arc::clone(&self.permits).acquire_owned()).await {
                Ok(Ok(p)) => p,
                _ => return estimated("overload", &self.metrics.estimated_overload),
            };
        let remaining = budget.saturating_sub(started.elapsed());
        let engine = Arc::clone(&self.engine);
        let work = parsed.clone();
        let handle = tokio::task::spawn_blocking(move || {
            let r = suggest::suggest(&engine, &work);
            drop(permit);
            r
        });
        match tokio::time::timeout(remaining, handle).await {
            Ok(Ok(r)) => {
                self.metrics.routed.fetch_add(1, Ordering::Relaxed);
                Ok(Response::new(to_pb(r)))
            }
            _ => estimated("deadline", &self.metrics.estimated_deadline),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pb::matching_service_client::MatchingServiceClient;
    use crate::pb::{DriverCandidate, MatchPolicy, PendingRequest};
    use road_graph::fixture::{compile_fixture, generate};
    use route_engine::engine::EngineConfig;
    use tokio_stream::wrappers::TcpListenerStream;

    fn e7(lat: f64, lng: f64) -> Option<LatLngE7> {
        Some(LatLngE7 {
            lat_e7: (lat * 1e7).round() as i32,
            lng_e7: (lng * 1e7).round() as i32,
        })
    }

    async fn start() -> MatchingServiceClient<tonic::transport::Channel> {
        let engine = Arc::new(Engine::new(
            compile_fixture(&generate()),
            EngineConfig::default(),
        ));
        let svc = MatchingSvc::new(engine, GeoEngineConfig::default());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = svc.into_server();
        tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(server)
                .serve_with_incoming(TcpListenerStream::new(listener))
                .await
                .unwrap();
        });
        MatchingServiceClient::connect(format!("http://{addr}"))
            .await
            .unwrap()
    }

    fn request() -> SuggestMatchRequest {
        SuggestMatchRequest {
            request_id: "REQ-T".into(),
            city_code: "HCM".into(),
            now_ms: 50_000,
            policy: Some(MatchPolicy::default()),
            snapshot_version: 9,
            pending: vec![
                PendingRequest {
                    booking_id: "BK-1".into(),
                    pickup: e7(10.7705, 106.6915),
                    requested_at_ms: 40_000,
                    waited_ms: None,
                },
                PendingRequest {
                    booking_id: "BK-2".into(),
                    pickup: e7(10.7745, 106.7005),
                    requested_at_ms: 41_000,
                    waited_ms: None,
                },
            ],
            drivers: vec![
                DriverCandidate {
                    driver_id: "DR-1".into(),
                    position: e7(10.7710, 106.6920),
                    state_version: 3,
                    idle_since_ms: 0,
                    busy_remaining_ms: None,
                },
                DriverCandidate {
                    driver_id: "DR-2".into(),
                    position: e7(10.7740, 106.7000),
                    state_version: 5,
                    idle_since_ms: 0,
                    busy_remaining_ms: None,
                },
            ],
            suggestion_ttl_ms: None,
            compute_budget_ms: None,
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn suggests_routed_pairs_with_ttl_and_is_deterministic() {
        let mut c = start().await;
        let a = c.suggest_match(request()).await.unwrap().into_inner();
        assert_eq!(a.quality, RoutingQuality::Routed as i32);
        assert_eq!(a.suggestions.len(), 2);
        assert_eq!(a.snapshot_version, 9);
        assert!(
            a.suggestions
                .iter()
                .all(|s| s.valid_until_ms == 60_000 && s.eta_ms > 0)
        );
        let b = c.suggest_match(request()).await.unwrap().into_inner();
        assert_eq!(a.suggestions, b.suggestions);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn zero_compute_budget_returns_estimated_not_error() {
        let mut c = start().await;
        let mut r = request();
        r.compute_budget_ms = Some(0);
        let a = c.suggest_match(r).await.unwrap().into_inner();
        assert_eq!(a.quality, RoutingQuality::Estimated as i32);
        assert_eq!(a.source, "straight-line:budget");
        assert_eq!(a.suggestions.len(), 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn invalid_requests_are_rejected() {
        let mut c = start().await;
        let mut r = request();
        r.request_id.clear();
        assert_eq!(
            c.suggest_match(r).await.unwrap_err().code(),
            tonic::Code::InvalidArgument
        );
        let mut r = request();
        r.pending[0].pickup = None;
        assert_eq!(
            c.suggest_match(r).await.unwrap_err().code(),
            tonic::Code::InvalidArgument
        );
        let mut r = request();
        r.drivers.clear();
        assert_eq!(
            c.suggest_match(r).await.unwrap_err().code(),
            tonic::Code::InvalidArgument
        );
    }
}
