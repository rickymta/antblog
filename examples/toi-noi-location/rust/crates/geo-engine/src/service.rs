//! Dịch vụ gRPC `RoutingService`: Route và Matrix.
//!
//! Luồng xử lý một yêu cầu:
//! 1. kiểm tra đầu vào (toạ độ, profile, kích thước ma trận) → `InvalidArgument`;
//! 2. ngân sách thời gian = deadline của client (header `grpc-timeout`) trừ một khoảng dự trữ, hoặc mặc định của dịch vụ;
//! 3. tra cache (chỉ Route, không hình học);
//! 4. tính trên đồ thị trong `spawn_blocking`, giới hạn số tính toán đồng thời bằng semaphore; chờ giấy phép và chờ kết quả
//!    cộng lại không quá ngân sách, hết ngân sách thì trả ước lượng (với giấy phép: «shed load»);
//! 5. quá hạn, quá tải hoặc không có đường thì **không báo lỗi**: trả ước lượng chim bay × hệ số vòng, `quality = ESTIMATED`.
//!
//! Lưu ý trung thực: tác vụ tính toán đã bắt đầu không huỷ được khi quá hạn (nó chạy tiếp tới cùng và giữ giấy phép của nó),
//! nên quá hạn nhiều thì các giấy phép hết và các yêu cầu sau bị shed. Đó là cơ chế bảo vệ, không phải miễn phí.

use crate::cache::{CachedRoute, RouteCache};
use crate::pb::routing_service_server::{RoutingService, RoutingServiceServer};
use crate::pb::{
    LatLngE7, MatrixRequest, MatrixResponse, RouteRequest, RouteResponse, RoutingProfile,
    RoutingQuality,
};
use geo_core::LatLng;
use route_engine::engine::{Engine, RouteError};
use route_engine::net::INF;
use route_engine::profile::Profile;
use route_engine::provider::StraightLineProvider;
use route_engine::traffic::slot_of_second;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::Semaphore;
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
pub struct GeoEngineConfig {
    /// Độ phân giải H3 của khoá cache (9: cạnh ô cỡ 200 m, h3o báo 201 m).
    pub h3_res: u8,
    pub cache_capacity: usize,
    /// Độ rộng khung giờ của khoá cache, giây.
    pub time_bucket_s: u32,
    /// Ngân sách khi client không gửi deadline.
    pub default_deadline_ms: u64,
    /// Phần deadline giữ lại để gửi trả lời (trừ khỏi ngân sách tính toán).
    pub reserve_ms: u64,
    pub max_matrix_cells: usize,
    /// Số tính toán đồ thị chạy đồng thời tối đa.
    pub max_inflight: usize,
    pub fallback: StraightLineProvider,
}

impl Default for GeoEngineConfig {
    fn default() -> Self {
        Self {
            h3_res: 9,
            cache_capacity: 200_000,
            time_bucket_s: 900,
            default_deadline_ms: 250,
            reserve_ms: 3,
            max_matrix_cells: 250_000,
            max_inflight: std::thread::available_parallelism().map_or(4, usize::from),
            fallback: StraightLineProvider::default(),
        }
    }
}

#[derive(Default)]
pub struct Metrics {
    pub route_requests: AtomicU64,
    pub matrix_requests: AtomicU64,
    pub cache_hits: AtomicU64,
    pub routed: AtomicU64,
    pub fallback_deadline: AtomicU64,
    pub fallback_overload: AtomicU64,
    pub fallback_no_route: AtomicU64,
    pub invalid: AtomicU64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MetricsSnapshot {
    pub route_requests: u64,
    pub matrix_requests: u64,
    pub cache_hits: u64,
    pub routed: u64,
    pub fallback_deadline: u64,
    pub fallback_overload: u64,
    pub fallback_no_route: u64,
    pub invalid: u64,
}

impl Metrics {
    pub fn snapshot(&self) -> MetricsSnapshot {
        let g = |a: &AtomicU64| a.load(Ordering::Relaxed);
        MetricsSnapshot {
            route_requests: g(&self.route_requests),
            matrix_requests: g(&self.matrix_requests),
            cache_hits: g(&self.cache_hits),
            routed: g(&self.routed),
            fallback_deadline: g(&self.fallback_deadline),
            fallback_overload: g(&self.fallback_overload),
            fallback_no_route: g(&self.fallback_no_route),
            invalid: g(&self.invalid),
        }
    }
}

#[derive(Clone)]
pub struct RoutingSvc {
    pub engine: Arc<Engine>,
    pub cache: Arc<RouteCache>,
    pub cfg: Arc<GeoEngineConfig>,
    pub metrics: Arc<Metrics>,
    permits: Arc<Semaphore>,
}

/// Đọc header `grpc-timeout`: số nguyên tối đa 8 chữ số kèm đơn vị H, M, S, m, u hoặc n.
pub fn parse_grpc_timeout(s: &str) -> Option<Duration> {
    let (num, unit) = s.split_at(s.len().checked_sub(1)?);
    let n: u64 = num.parse().ok()?;
    Some(match unit {
        "H" => Duration::from_secs(n * 3600),
        "M" => Duration::from_secs(n * 60),
        "S" => Duration::from_secs(n),
        "m" => Duration::from_millis(n),
        "u" => Duration::from_micros(n),
        "n" => Duration::from_nanos(n),
        _ => return None,
    })
}

fn to_latlng(p: Option<&LatLngE7>, what: &str) -> Result<LatLng, Status> {
    let p = p.ok_or_else(|| Status::invalid_argument(format!("thiếu {what}")))?;
    LatLng::try_new(f64::from(p.lat_e7) / 1e7, f64::from(p.lng_e7) / 1e7)
        .map_err(|e| Status::invalid_argument(format!("{what} không hợp lệ: {e}")))
}

fn to_profile(p: i32) -> Result<Profile, Status> {
    match RoutingProfile::try_from(p) {
        Ok(RoutingProfile::Car) => Ok(Profile::Car),
        Ok(RoutingProfile::Motorbike) => Ok(Profile::Motorbike),
        Ok(RoutingProfile::Foot) => Ok(Profile::Foot),
        _ => Err(Status::invalid_argument(
            "cần profile (ô tô, xe máy hoặc đi bộ)",
        )),
    }
}

#[derive(Debug, Clone, Copy)]
enum Fallback {
    Deadline,
    Overload,
    NoRoute,
}

impl Fallback {
    fn source(self) -> &'static str {
        match self {
            Fallback::Deadline => "straight-line:deadline",
            Fallback::Overload => "straight-line:overload",
            Fallback::NoRoute => "straight-line:no-route",
        }
    }
}

impl RoutingSvc {
    pub fn new(engine: Arc<Engine>, cfg: GeoEngineConfig) -> Self {
        Self {
            cache: Arc::new(RouteCache::new(
                cfg.cache_capacity,
                cfg.h3_res,
                cfg.time_bucket_s,
            )),
            permits: Arc::new(Semaphore::new(cfg.max_inflight.max(1))),
            cfg: Arc::new(cfg),
            metrics: Arc::new(Metrics::default()),
            engine,
        }
    }

    pub fn into_server(self) -> RoutingServiceServer<Self> {
        RoutingServiceServer::new(self)
    }

    /// Ngân sách tính toán của một yêu cầu: deadline của client trừ dự trữ, hoặc mặc định của dịch vụ.
    fn budget<T>(&self, req: &Request<T>) -> Duration {
        let client = req
            .metadata()
            .get("grpc-timeout")
            .and_then(|v| v.to_str().ok())
            .and_then(parse_grpc_timeout);
        let total = client.unwrap_or(Duration::from_millis(self.cfg.default_deadline_ms));
        total.saturating_sub(Duration::from_millis(self.cfg.reserve_ms))
    }

    /// Chạy `work` trên luồng chặn. Chờ giấy phép tối đa bằng ngân sách (xếp hàng có hạn); hết ngân sách mà chưa có giấy phép
    /// thì coi là quá tải (shed). Có giấy phép rồi thì chờ kết quả tối đa phần ngân sách còn lại.
    async fn compute<R: Send + 'static>(
        &self,
        budget: Duration,
        work: impl FnOnce() -> Result<R, RouteError> + Send + 'static,
    ) -> Result<R, Fallback> {
        if budget.is_zero() {
            return Err(Fallback::Deadline);
        }
        let started = std::time::Instant::now();
        let permit =
            match tokio::time::timeout(budget, Arc::clone(&self.permits).acquire_owned()).await {
                Ok(Ok(p)) => p,
                _ => return Err(Fallback::Overload),
            };
        let remaining = budget.saturating_sub(started.elapsed());
        if remaining.is_zero() {
            return Err(Fallback::Deadline);
        }
        let handle = tokio::task::spawn_blocking(move || {
            let r = work();
            drop(permit);
            r
        });
        match tokio::time::timeout(remaining, handle).await {
            Ok(Ok(Ok(r))) => Ok(r),
            Ok(Ok(Err(_))) => Err(Fallback::NoRoute),
            Ok(Err(_)) => Err(Fallback::NoRoute),
            Err(_) => Err(Fallback::Deadline),
        }
    }

    fn count_fallback(&self, f: Fallback) {
        let c = match f {
            Fallback::Deadline => &self.metrics.fallback_deadline,
            Fallback::Overload => &self.metrics.fallback_overload,
            Fallback::NoRoute => &self.metrics.fallback_no_route,
        };
        c.fetch_add(1, Ordering::Relaxed);
    }
}

#[tonic::async_trait]
impl RoutingService for RoutingSvc {
    async fn route(
        &self,
        request: Request<RouteRequest>,
    ) -> Result<Response<RouteResponse>, Status> {
        self.metrics.route_requests.fetch_add(1, Ordering::Relaxed);
        let budget = self.budget(&request);
        let req = request.into_inner();
        let parsed = (|| -> Result<_, Status> {
            Ok((
                to_latlng(req.from.as_ref(), "điểm đi")?,
                to_latlng(req.to.as_ref(), "điểm đến")?,
                to_profile(req.profile)?,
            ))
        })();
        let (from, to, profile) = match parsed {
            Ok(x) => x,
            Err(e) => {
                self.metrics.invalid.fetch_add(1, Ordering::Relaxed);
                return Err(e);
            }
        };
        let depart = req.depart_sec_of_day;

        let key = self.cache.key(from, to, profile.index() as u8, depart);
        if !req.want_geometry
            && let Some(k) = &key
            && let Some(hit) = self.cache.get(k)
        {
            self.metrics.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(Response::new(RouteResponse {
                duration_ms: hit.duration_ms,
                distance_m: hit.distance_m,
                quality: RoutingQuality::Cached as i32,
                source: "cache".into(),
                polyline6: String::new(),
            }));
        }

        let engine = Arc::clone(&self.engine);
        let slot = depart.map(slot_of_second);
        let want_geometry = req.want_geometry;
        let res = self
            .compute(budget, move || {
                let r = engine.route(profile, from, to, slot)?;
                let poly = want_geometry.then(|| geo_core::polyline::encode(&r.points, 6));
                Ok((r.duration_ms, r.distance_m, r.algo, poly))
            })
            .await;
        match res {
            Ok((duration_ms, distance_m, algo, poly)) => {
                self.metrics.routed.fetch_add(1, Ordering::Relaxed);
                let distance_m = distance_m.round() as u32;
                if let Some(k) = key {
                    self.cache.put(
                        k,
                        CachedRoute {
                            duration_ms,
                            distance_m,
                        },
                    );
                }
                Ok(Response::new(RouteResponse {
                    duration_ms,
                    distance_m,
                    quality: RoutingQuality::Routed as i32,
                    source: algo.into(),
                    polyline6: poly.unwrap_or_default(),
                }))
            }
            Err(f) => {
                self.count_fallback(f);
                let (dist, dur) = self.cfg.fallback.estimate(profile, from, to);
                Ok(Response::new(RouteResponse {
                    duration_ms: (dur * 1000.0).round() as u32,
                    distance_m: dist.round() as u32,
                    quality: RoutingQuality::Estimated as i32,
                    source: f.source().into(),
                    polyline6: String::new(),
                }))
            }
        }
    }

    async fn matrix(
        &self,
        request: Request<MatrixRequest>,
    ) -> Result<Response<MatrixResponse>, Status> {
        self.metrics.matrix_requests.fetch_add(1, Ordering::Relaxed);
        let mut budget = self.budget(&request);
        let req = request.into_inner();
        // ma trận lớn: giữ thêm 1 ms mỗi 100.000 ô cho việc dựng ước lượng dự phòng và mã hoá trả lời
        budget = budget.saturating_sub(Duration::from_millis(
            (req.sources.len().saturating_mul(req.targets.len()) / 100_000) as u64,
        ));
        let invalid = |m: String| {
            self.metrics.invalid.fetch_add(1, Ordering::Relaxed);
            Status::invalid_argument(m)
        };
        if req.sources.is_empty() || req.targets.is_empty() {
            return Err(invalid("cần ít nhất một nguồn và một đích".into()));
        }
        let cells = req.sources.len().saturating_mul(req.targets.len());
        if cells > self.cfg.max_matrix_cells {
            return Err(invalid(format!(
                "ma trận {} × {} = {cells} ô vượt giới hạn {}",
                req.sources.len(),
                req.targets.len(),
                self.cfg.max_matrix_cells
            )));
        }
        let profile = to_profile(req.profile).map_err(|e| invalid(e.message().to_string()))?;
        let conv = |v: &[LatLngE7], what: &str| -> Result<Vec<LatLng>, Status> {
            v.iter().map(|p| to_latlng(Some(p), what)).collect()
        };
        let src = conv(&req.sources, "nguồn").map_err(|e| invalid(e.message().to_string()))?;
        let dst = conv(&req.targets, "đích").map_err(|e| invalid(e.message().to_string()))?;
        let slot = req.depart_sec_of_day.map(slot_of_second);
        let (rows, cols) = (src.len(), dst.len());

        let engine = Arc::clone(&self.engine);
        let (s2, d2) = (src.clone(), dst.clone());
        let res = self
            .compute(budget, move || {
                Ok(engine.matrix(profile, &s2, &d2, slot).matrix)
            })
            .await;
        match res {
            Ok(m) => {
                self.metrics.routed.fetch_add(1, Ordering::Relaxed);
                Ok(Response::new(MatrixResponse {
                    rows: rows as u32,
                    cols: cols as u32,
                    duration_ms: m
                        .data
                        .iter()
                        .map(|&v| if v == INF { u32::MAX } else { v })
                        .collect(),
                    quality: RoutingQuality::Routed as i32,
                    source: "ch-bucket".into(),
                }))
            }
            Err(f) => {
                self.count_fallback(f);
                let mut out = Vec::with_capacity(cells);
                for &a in &src {
                    for &b in &dst {
                        out.push(
                            (self.cfg.fallback.estimate_fast(profile, a, b).1 * 1000.0).round()
                                as u32,
                        );
                    }
                }
                Ok(Response::new(MatrixResponse {
                    rows: rows as u32,
                    cols: cols as u32,
                    duration_ms: out,
                    quality: RoutingQuality::Estimated as i32,
                    source: f.source().into(),
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pb::routing_service_client::RoutingServiceClient;
    use road_graph::fixture::{compile_fixture, generate};
    use route_engine::engine::EngineConfig;
    use tokio_stream::wrappers::TcpListenerStream;

    fn e7(lat: f64, lng: f64) -> Option<LatLngE7> {
        Some(LatLngE7 {
            lat_e7: (lat * 1e7).round() as i32,
            lng_e7: (lng * 1e7).round() as i32,
        })
    }

    async fn start(
        cfg: GeoEngineConfig,
    ) -> (RoutingSvc, RoutingServiceClient<tonic::transport::Channel>) {
        let engine = Arc::new(Engine::new(
            compile_fixture(&generate()),
            EngineConfig::default(),
        ));
        let svc = RoutingSvc::new(engine, cfg);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = svc.clone().into_server();
        tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(server)
                .serve_with_incoming(TcpListenerStream::new(listener))
                .await
                .unwrap();
        });
        let client = RoutingServiceClient::connect(format!("http://{addr}"))
            .await
            .unwrap();
        (svc, client)
    }

    fn route_req() -> RouteRequest {
        RouteRequest {
            from: e7(10.7705, 106.6905),
            to: e7(10.7760, 106.7020),
            profile: RoutingProfile::Car as i32,
            depart_sec_of_day: None,
            want_geometry: false,
        }
    }

    #[test]
    fn grpc_timeout_header_parses_all_units() {
        assert_eq!(parse_grpc_timeout("100m"), Some(Duration::from_millis(100)));
        assert_eq!(parse_grpc_timeout("2S"), Some(Duration::from_secs(2)));
        assert_eq!(parse_grpc_timeout("500u"), Some(Duration::from_micros(500)));
        assert_eq!(parse_grpc_timeout("1H"), Some(Duration::from_secs(3600)));
        assert_eq!(parse_grpc_timeout("x"), None);
        assert_eq!(parse_grpc_timeout("12q"), None);
        assert_eq!(parse_grpc_timeout(""), None);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn route_then_cache_hit_then_geometry_bypass() {
        let (svc, mut c) = start(GeoEngineConfig::default()).await;
        let a = c.route(route_req()).await.unwrap().into_inner();
        assert_eq!(a.quality, RoutingQuality::Routed as i32);
        assert!(a.duration_ms > 0 && a.distance_m > 100);
        // điểm lệch vài mét, cùng ô H3 và cùng khung giờ → trúng cache với đúng đáp án của yêu cầu đầu
        let mut near = route_req();
        near.from = e7(10.77051, 106.69051);
        let b = c.route(near).await.unwrap().into_inner();
        assert_eq!(b.quality, RoutingQuality::Cached as i32);
        assert_eq!((b.duration_ms, b.distance_m), (a.duration_ms, a.distance_m));
        // yêu cầu hình học không dùng cache
        let mut g = route_req();
        g.want_geometry = true;
        let r = c.route(g).await.unwrap().into_inner();
        assert_eq!(r.quality, RoutingQuality::Routed as i32);
        assert!(!r.polyline6.is_empty());
        let pts = geo_core::polyline::decode(&r.polyline6, 6).unwrap();
        assert!(pts.len() >= 2);
        let m = svc.metrics.snapshot();
        assert_eq!((m.route_requests, m.cache_hits), (3, 1));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn invalid_requests_are_rejected() {
        let (_, mut c) = start(GeoEngineConfig::default()).await;
        let mut r = route_req();
        r.profile = 0;
        assert_eq!(
            c.route(r).await.unwrap_err().code(),
            tonic::Code::InvalidArgument
        );
        let mut r = route_req();
        r.from = None;
        assert_eq!(
            c.route(r).await.unwrap_err().code(),
            tonic::Code::InvalidArgument
        );
        let mut r = route_req();
        r.to = e7(95.0, 0.0);
        assert_eq!(
            c.route(r).await.unwrap_err().code(),
            tonic::Code::InvalidArgument
        );
        let m = MatrixRequest {
            sources: vec![],
            targets: vec![],
            profile: 1,
            depart_sec_of_day: None,
        };
        assert_eq!(
            c.matrix(m).await.unwrap_err().code(),
            tonic::Code::InvalidArgument
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn matrix_matches_route_and_enforces_the_size_limit() {
        let (_, mut c) = start(GeoEngineConfig {
            max_matrix_cells: 16,
            ..GeoEngineConfig::default()
        })
        .await;
        let pts = [
            e7(10.7705, 106.6905),
            e7(10.7760, 106.7020),
            e7(10.7720, 106.6950),
        ];
        let m = c
            .matrix(MatrixRequest {
                sources: pts.iter().flatten().cloned().collect(),
                targets: pts.iter().flatten().cloned().collect(),
                profile: 1,
                depart_sec_of_day: None,
            })
            .await
            .unwrap()
            .into_inner();
        assert_eq!((m.rows, m.cols, m.duration_ms.len()), (3, 3, 9));
        let r = c.route(route_req()).await.unwrap().into_inner();
        assert_eq!(m.duration_ms[1], r.duration_ms);
        let big = MatrixRequest {
            sources: vec![pts[0].unwrap(); 5],
            targets: vec![pts[1].unwrap(); 5],
            profile: 1,
            depart_sec_of_day: None,
        };
        assert_eq!(
            c.matrix(big).await.unwrap_err().code(),
            tonic::Code::InvalidArgument
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn zero_budget_falls_back_to_straight_line_without_error() {
        let (svc, mut c) = start(GeoEngineConfig {
            reserve_ms: 10_000,
            ..GeoEngineConfig::default()
        })
        .await;
        let r = c.route(route_req()).await.unwrap().into_inner();
        assert_eq!(r.quality, RoutingQuality::Estimated as i32);
        assert_eq!(r.source, "straight-line:deadline");
        assert!(r.duration_ms > 0);
        assert_eq!(svc.metrics.snapshot().fallback_deadline, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn client_deadline_header_is_read_and_far_point_falls_back_with_no_route() {
        let (svc, mut c) = start(GeoEngineConfig::default()).await;
        let mut req = tonic::Request::new(RouteRequest {
            to: e7(10.95, 106.95), // ngoài bản đồ
            ..route_req()
        });
        req.set_timeout(Duration::from_millis(500));
        let r = c.route(req).await.unwrap().into_inner();
        assert_eq!(r.quality, RoutingQuality::Estimated as i32);
        assert_eq!(r.source, "straight-line:no-route");
        assert_eq!(svc.metrics.snapshot().fallback_no_route, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn budget_comes_from_the_grpc_timeout_header_minus_the_reserve() {
        let (svc, _c) = start(GeoEngineConfig::default()).await; // reserve 3 ms, mặc định 250 ms
        let mut with = Request::new(());
        with.metadata_mut()
            .insert("grpc-timeout", "100m".parse().unwrap());
        assert_eq!(svc.budget(&with), Duration::from_millis(97));
        let without = Request::new(());
        assert_eq!(svc.budget(&without), Duration::from_millis(247));
        let mut tiny = Request::new(());
        tiny.metadata_mut()
            .insert("grpc-timeout", "2m".parse().unwrap());
        assert!(svc.budget(&tiny).is_zero());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn overload_waits_within_the_budget_then_sheds() {
        let (svc, _c) = start(GeoEngineConfig {
            max_inflight: 1,
            ..GeoEngineConfig::default()
        })
        .await;
        // giữ giấy phép duy nhất: yêu cầu chờ hết ngân sách (7 ms) rồi bị shed
        let hold = Arc::clone(&svc.permits).acquire_owned().await.unwrap();
        let mut req = tonic::Request::new(route_req());
        req.metadata_mut()
            .insert("grpc-timeout", "10m".parse().unwrap());
        let t = std::time::Instant::now();
        let r = RoutingService::route(&svc, req).await.unwrap().into_inner();
        assert_eq!(r.source, "straight-line:overload");
        assert!(
            t.elapsed() >= Duration::from_millis(6),
            "phải chờ giấy phép tới hết ngân sách: {:?}",
            t.elapsed()
        );
        // trả giấy phép giữa chừng: yêu cầu được phục vụ thay vì bị shed
        let svc2 = svc.clone();
        let waiter = tokio::spawn(async move {
            let mut req = tonic::Request::new(route_req());
            req.metadata_mut()
                .insert("grpc-timeout", "2S".parse().unwrap());
            RoutingService::route(&svc2, req)
                .await
                .unwrap()
                .into_inner()
        });
        tokio::time::sleep(Duration::from_millis(20)).await;
        drop(hold);
        let r = waiter.await.unwrap();
        assert_eq!(r.quality, RoutingQuality::Routed as i32);
    }
}
