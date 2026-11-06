//! LOC-28: port `RoutingProvider` và các adapter.
//!
//! Phần còn lại của hệ thống (simulator, dịch vụ gRPC, chọn điểm đón) chỉ biết trait này, không biết engine nào ở sau.
//! Adapter có trong crate:
//! - [`OwnEngineProvider`]: engine CH của lab (chạy thật)
//! - [`StraightLineProvider`]: chim bay × hệ số vòng (dùng làm fallback; chạy thật, rất rẻ)
//! - [`OsrmHttpProvider`]: **hình dạng** của adapter HTTP cho OSRM: dựng URL `route/v1`, `table/v1`, đọc JSON.
//!   **Chưa được chạy với server OSRM thật.** Test dùng [`MockOsrmTransport`], một transport giả sinh JSON đúng schema OSRM
//!   từ chính engine của lab; nó kiểm tra mã dựng URL và đọc JSON, không chứng minh gì về OSRM.
//! - [`FallbackProvider`]: thử adapter chính, lỗi thì dùng adapter phụ.

use crate::engine::{Engine, RouteError};
use crate::net::INF;
use crate::profile::Profile;
use geo_core::{LatLng, haversine_m};
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RouteQuery {
    pub from: LatLng,
    pub to: LatLng,
    pub profile: Profile,
    /// Giây trong ngày lúc khởi hành (để chọn hồ sơ giờ nếu provider hỗ trợ).
    pub depart_sec_of_day: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    /// Tính trên đồ thị đường.
    Routed,
    /// Ước lượng chim bay × hệ số vòng.
    Estimated,
}

#[derive(Debug, Clone)]
pub struct RouteAnswer {
    pub duration_s: f64,
    pub distance_m: f64,
    pub geometry: Vec<LatLng>,
    pub provider: String,
    pub quality: Quality,
}

#[derive(Debug, Clone)]
pub struct MatrixQuery {
    pub sources: Vec<LatLng>,
    pub targets: Vec<LatLng>,
    pub profile: Profile,
    pub depart_sec_of_day: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct MatrixAnswer {
    pub rows: usize,
    pub cols: usize,
    /// Hàng-chính, giây. `None` = không tới được.
    pub duration_s: Vec<Option<f64>>,
    pub provider: String,
    pub quality: Quality,
}

impl MatrixAnswer {
    pub fn get(&self, r: usize, c: usize) -> Option<f64> {
        self.duration_s[r * self.cols + c]
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ProviderError {
    #[error("không có đường: {0}")]
    NoRoute(String),
    #[error("lỗi vận chuyển: {0}")]
    Transport(String),
    #[error("provider trả dữ liệu sai dạng: {0}")]
    BadResponse(String),
    #[error("yêu cầu không hỗ trợ: {0}")]
    Unsupported(String),
}

pub trait RoutingProvider: Send + Sync {
    fn name(&self) -> &str;
    fn route(&self, q: &RouteQuery) -> Result<RouteAnswer, ProviderError>;
    fn matrix(&self, q: &MatrixQuery) -> Result<MatrixAnswer, ProviderError>;
}

// ---------------------------------------------------------------- engine của lab

pub struct OwnEngineProvider {
    pub engine: Arc<Engine>,
}

fn slot_of(q: Option<u32>) -> Option<usize> {
    q.map(crate::traffic::slot_of_second)
}

impl RoutingProvider for OwnEngineProvider {
    fn name(&self) -> &str {
        "own-engine"
    }
    fn route(&self, q: &RouteQuery) -> Result<RouteAnswer, ProviderError> {
        let r = self
            .engine
            .route(q.profile, q.from, q.to, slot_of(q.depart_sec_of_day))
            .map_err(|e| match e {
                RouteError::NoRoute => ProviderError::NoRoute(e.to_string()),
                other => ProviderError::NoRoute(other.to_string()),
            })?;
        Ok(RouteAnswer {
            duration_s: f64::from(r.duration_ms) / 1000.0,
            distance_m: r.distance_m,
            geometry: r.points,
            provider: self.name().into(),
            quality: Quality::Routed,
        })
    }
    fn matrix(&self, q: &MatrixQuery) -> Result<MatrixAnswer, ProviderError> {
        let m = self.engine.matrix(
            q.profile,
            &q.sources,
            &q.targets,
            slot_of(q.depart_sec_of_day),
        );
        Ok(MatrixAnswer {
            rows: m.matrix.rows,
            cols: m.matrix.cols,
            duration_s: m
                .matrix
                .data
                .iter()
                .map(|&v| (v != INF).then(|| f64::from(v) / 1000.0))
                .collect(),
            provider: self.name().into(),
            quality: Quality::Routed,
        })
    }
}

// ---------------------------------------------------------------- chim bay × hệ số vòng

/// Ước lượng không cần bản đồ: `haversine × detour / tốc_độ_trung_bình`.
#[derive(Debug, Clone)]
pub struct StraightLineProvider {
    /// Hệ số vòng đường/chim bay.
    pub detour: f64,
    /// Tốc độ chạy trên đường (km/h, đã tính cả hệ số vòng) theo `Profile::index()`.
    pub speed_kmh: [f64; 3],
}

impl Default for StraightLineProvider {
    fn default() -> Self {
        // Hiệu chỉnh theo engine của lab trên bản đồ HCM (LOC-28, mục 4): trung vị hệ số vòng ô tô 1,296 và tốc độ
        // chạy trên đường suy ra từ trung vị chim bay/thời gian. Xe máy và đi bộ đặt tay (xe máy hơn ô tô một chút).
        // Vì engine cũng dựa trên tốc độ giả định, đây là «khớp với engine», không phải «khớp với đường thật».
        Self {
            detour: 1.3,
            speed_kmh: [35.0, 37.0, 4.9],
        }
    }
}

impl StraightLineProvider {
    pub fn estimate(&self, p: Profile, a: LatLng, b: LatLng) -> (f64, f64) {
        let dist = haversine_m(a, b) * self.detour;
        (dist, dist / (self.speed_kmh[p.index()] / 3.6))
    }
    /// Như [`StraightLineProvider::estimate`] nhưng dùng equirectangular (rẻ hơn haversine, tỉ lệ dao động theo tải, xem LOC-04; sai rất nhỏ trong một thành phố),
    /// dành cho ma trận dự phòng hàng trăm nghìn ô.
    pub fn estimate_fast(&self, p: Profile, a: LatLng, b: LatLng) -> (f64, f64) {
        let dist = geo_core::equirect_m(a, b) * self.detour;
        (dist, dist / (self.speed_kmh[p.index()] / 3.6))
    }
}

impl RoutingProvider for StraightLineProvider {
    fn name(&self) -> &str {
        "straight-line"
    }
    fn route(&self, q: &RouteQuery) -> Result<RouteAnswer, ProviderError> {
        let (dist, dur) = self.estimate(q.profile, q.from, q.to);
        Ok(RouteAnswer {
            duration_s: dur,
            distance_m: dist,
            geometry: vec![q.from, q.to],
            provider: self.name().into(),
            quality: Quality::Estimated,
        })
    }
    fn matrix(&self, q: &MatrixQuery) -> Result<MatrixAnswer, ProviderError> {
        let mut data = Vec::with_capacity(q.sources.len() * q.targets.len());
        for &a in &q.sources {
            for &b in &q.targets {
                data.push(Some(self.estimate_fast(q.profile, a, b).1));
            }
        }
        Ok(MatrixAnswer {
            rows: q.sources.len(),
            cols: q.targets.len(),
            duration_s: data,
            provider: self.name().into(),
            quality: Quality::Estimated,
        })
    }
}

// ---------------------------------------------------------------- adapter HTTP kiểu OSRM

#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct TransportError(pub String);

/// Cổng HTTP tối thiểu. Bản thật dùng `reqwest`/`hyper`; lab không kéo thêm thư viện HTTP client.
pub trait HttpTransport: Send + Sync {
    fn get(&self, url: &str, timeout_ms: u64) -> Result<String, TransportError>;
}

pub struct OsrmHttpProvider<T: HttpTransport> {
    pub base_url: String,
    pub transport: T,
    pub timeout_ms: u64,
}

impl<T: HttpTransport> OsrmHttpProvider<T> {
    pub fn new(base_url: &str, transport: T) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            transport,
            timeout_ms: 800,
        }
    }

    /// OSRM không có profile xe máy mặc định: dùng `car` và nói rõ đó là xấp xỉ.
    fn profile_name(p: Profile) -> &'static str {
        match p {
            Profile::Car | Profile::Motorbike => "car",
            Profile::Foot => "foot",
        }
    }

    /// OSRM nhận `lng,lat` (đảo thứ tự so với `LatLng`), phân tách các điểm bằng `;`.
    pub fn coords(points: &[LatLng]) -> String {
        points
            .iter()
            .map(|p| format!("{:.6},{:.6}", p.lng(), p.lat()))
            .collect::<Vec<_>>()
            .join(";")
    }

    pub fn route_url(&self, q: &RouteQuery) -> String {
        format!(
            "{}/route/v1/{}/{}?overview=full&geometries=polyline&steps=false",
            self.base_url,
            Self::profile_name(q.profile),
            Self::coords(&[q.from, q.to])
        )
    }

    pub fn table_url(&self, q: &MatrixQuery) -> String {
        let mut all = q.sources.clone();
        all.extend(q.targets.iter().copied());
        let src: Vec<String> = (0..q.sources.len()).map(|i| i.to_string()).collect();
        let dst: Vec<String> = (q.sources.len()..all.len())
            .map(|i| i.to_string())
            .collect();
        format!(
            "{}/table/v1/{}/{}?sources={}&destinations={}&annotations=duration",
            self.base_url,
            Self::profile_name(q.profile),
            Self::coords(&all),
            src.join(";"),
            dst.join(";")
        )
    }

    fn check_code(v: &Value) -> Result<(), ProviderError> {
        match v.get("code").and_then(Value::as_str) {
            Some("Ok") => Ok(()),
            Some("NoRoute") => Err(ProviderError::NoRoute("OSRM trả NoRoute".into())),
            Some(other) => Err(ProviderError::BadResponse(format!("code={other}"))),
            None => Err(ProviderError::BadResponse("thiếu code".into())),
        }
    }

    pub fn parse_route(&self, body: &str) -> Result<RouteAnswer, ProviderError> {
        let v: Value =
            serde_json::from_str(body).map_err(|e| ProviderError::BadResponse(e.to_string()))?;
        Self::check_code(&v)?;
        let r = v
            .pointer("/routes/0")
            .ok_or_else(|| ProviderError::BadResponse("thiếu routes[0]".into()))?;
        let num = |k: &str| {
            r.get(k)
                .and_then(Value::as_f64)
                .ok_or_else(|| ProviderError::BadResponse(format!("thiếu {k}")))
        };
        let geometry =
            geo_core::polyline::decode(r.get("geometry").and_then(Value::as_str).unwrap_or(""), 5)
                .map_err(|e| ProviderError::BadResponse(e.to_string()))?;
        Ok(RouteAnswer {
            duration_s: num("duration")?,
            distance_m: num("distance")?,
            geometry,
            provider: self.name().into(),
            quality: Quality::Routed,
        })
    }

    pub fn parse_table(
        &self,
        body: &str,
        rows: usize,
        cols: usize,
    ) -> Result<MatrixAnswer, ProviderError> {
        let v: Value =
            serde_json::from_str(body).map_err(|e| ProviderError::BadResponse(e.to_string()))?;
        Self::check_code(&v)?;
        let table = v
            .get("durations")
            .and_then(Value::as_array)
            .ok_or_else(|| ProviderError::BadResponse("thiếu durations".into()))?;
        if table.len() != rows {
            return Err(ProviderError::BadResponse("sai số hàng".into()));
        }
        let mut data = Vec::with_capacity(rows * cols);
        for row in table {
            let row = row
                .as_array()
                .filter(|r| r.len() == cols)
                .ok_or_else(|| ProviderError::BadResponse("sai số cột".into()))?;
            data.extend(row.iter().map(Value::as_f64)); // null → None
        }
        Ok(MatrixAnswer {
            rows,
            cols,
            duration_s: data,
            provider: self.name().into(),
            quality: Quality::Routed,
        })
    }
}

impl<T: HttpTransport> RoutingProvider for OsrmHttpProvider<T> {
    fn name(&self) -> &str {
        "osrm-http"
    }
    fn route(&self, q: &RouteQuery) -> Result<RouteAnswer, ProviderError> {
        let body = self
            .transport
            .get(&self.route_url(q), self.timeout_ms)
            .map_err(|e| ProviderError::Transport(e.0))?;
        self.parse_route(&body)
    }
    fn matrix(&self, q: &MatrixQuery) -> Result<MatrixAnswer, ProviderError> {
        let body = self
            .transport
            .get(&self.table_url(q), self.timeout_ms)
            .map_err(|e| ProviderError::Transport(e.0))?;
        self.parse_table(&body, q.sources.len(), q.targets.len())
    }
}

/// Valhalla dùng POST JSON thay vì URL: chỉ dựng thân yêu cầu để thấy hình dạng khác nhau (không gửi đi đâu cả).
pub fn valhalla_route_request(q: &RouteQuery) -> Value {
    let costing = match q.profile {
        Profile::Car => "auto",
        Profile::Motorbike => "motor_scooter",
        Profile::Foot => "pedestrian",
    };
    json!({
        "locations": [
            {"lat": q.from.lat(), "lon": q.from.lng()},
            {"lat": q.to.lat(), "lon": q.to.lng()},
        ],
        "costing": costing,
        "units": "kilometers",
    })
}

/// Transport giả: nhận URL do [`OsrmHttpProvider`] dựng, gọi engine của lab, trả JSON đúng schema OSRM.
/// Đây **không phải** OSRM. Dùng để kiểm tra adapter mà không cần server.
pub struct MockOsrmTransport {
    pub engine: Arc<Engine>,
    /// Số lần được gọi, để test kiểm tra.
    pub calls: std::sync::atomic::AtomicUsize,
}

impl MockOsrmTransport {
    pub fn new(engine: Arc<Engine>) -> Self {
        Self {
            engine,
            calls: std::sync::atomic::AtomicUsize::new(0),
        }
    }
    fn parse_coords(s: &str) -> Option<Vec<LatLng>> {
        s.split(';')
            .map(|p| {
                let (lng, lat) = p.split_once(',')?;
                LatLng::try_new(lat.parse().ok()?, lng.parse().ok()?).ok()
            })
            .collect()
    }
}

impl HttpTransport for MockOsrmTransport {
    fn get(&self, url: &str, _timeout_ms: u64) -> Result<String, TransportError> {
        self.calls
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let rest = url
            .split("/v1/")
            .nth(1)
            .ok_or_else(|| TransportError("URL không có /v1/".into()))?;
        let (profile_s, rest) = rest
            .split_once('/')
            .ok_or_else(|| TransportError("thiếu profile".into()))?;
        let (coords_s, query) = rest.split_once('?').unwrap_or((rest, ""));
        let profile = if profile_s == "foot" {
            Profile::Foot
        } else {
            Profile::Car
        };
        let pts =
            Self::parse_coords(coords_s).ok_or_else(|| TransportError("toạ độ sai".into()))?;
        let own = OwnEngineProvider {
            engine: Arc::clone(&self.engine),
        };
        if url.contains("/route/v1/") {
            let q = RouteQuery {
                from: pts[0],
                to: pts[1],
                profile,
                depart_sec_of_day: None,
            };
            return match own.route(&q) {
                Ok(a) => Ok(json!({
                    "code": "Ok",
                    "routes": [{
                        "distance": a.distance_m,
                        "duration": a.duration_s,
                        "geometry": geo_core::polyline::encode(&a.geometry, 5),
                    }]
                })
                .to_string()),
                Err(_) => Ok(json!({"code": "NoRoute", "message": "mock"}).to_string()),
            };
        }
        // table: sources=0;1&destinations=2;3
        let param = |k: &str| {
            query.split('&').find_map(|kv| {
                let (key, v) = kv.split_once('=')?;
                (key == k).then(|| {
                    v.split(';')
                        .filter_map(|x| x.parse::<usize>().ok())
                        .collect::<Vec<_>>()
                })
            })
        };
        let (src, dst) = (
            param("sources").ok_or_else(|| TransportError("thiếu sources".into()))?,
            param("destinations").ok_or_else(|| TransportError("thiếu destinations".into()))?,
        );
        let q = MatrixQuery {
            sources: src.iter().map(|&i| pts[i]).collect(),
            targets: dst.iter().map(|&i| pts[i]).collect(),
            profile,
            depart_sec_of_day: None,
        };
        let m = own.matrix(&q).map_err(|e| TransportError(e.to_string()))?;
        let rows: Vec<Value> = (0..m.rows)
            .map(|r| {
                Value::Array(
                    (0..m.cols)
                        .map(|c| m.get(r, c).map_or(Value::Null, |d| json!(d)))
                        .collect(),
                )
            })
            .collect();
        Ok(json!({"code": "Ok", "durations": rows}).to_string())
    }
}

// ---------------------------------------------------------------- tổ hợp

/// Thử `primary`; nếu lỗi (hoặc không có đường) thì dùng `secondary`. Đây là cách ghép adapter thành một chuỗi suy giảm.
pub struct FallbackProvider {
    pub primary: Box<dyn RoutingProvider>,
    pub secondary: Box<dyn RoutingProvider>,
}

impl RoutingProvider for FallbackProvider {
    fn name(&self) -> &str {
        "fallback"
    }
    fn route(&self, q: &RouteQuery) -> Result<RouteAnswer, ProviderError> {
        self.primary.route(q).or_else(|_| self.secondary.route(q))
    }
    fn matrix(&self, q: &MatrixQuery) -> Result<MatrixAnswer, ProviderError> {
        self.primary.matrix(q).or_else(|_| self.secondary.matrix(q))
    }
}

/// Kiểm tra hợp đồng chung mọi adapter phải thoả. Trả danh sách vi phạm (rỗng = đạt).
/// `snap_tolerance_m`: điểm đầu vào được snap lên đường nên tuyến có thể ngắn hơn chim bay giữa hai điểm thô cỡ tổng hai khoảng snap.
pub fn contract_violations(
    p: &dyn RoutingProvider,
    a: LatLng,
    b: LatLng,
    profile: Profile,
    snap_tolerance_m: f64,
) -> Vec<String> {
    let mut bad = Vec::new();
    let q = RouteQuery {
        from: a,
        to: b,
        profile,
        depart_sec_of_day: None,
    };
    match p.route(&q) {
        Err(e) => bad.push(format!("route lỗi: {e}")),
        Ok(r) => {
            let straight = haversine_m(a, b);
            if r.duration_s <= 0.0 {
                bad.push("duration phải dương".into());
            }
            if r.distance_m + snap_tolerance_m < straight * 0.98 {
                bad.push(format!(
                    "distance {:.0} m ngắn hơn chim bay {:.0} m",
                    r.distance_m, straight
                ));
            }
            if r.geometry.len() < 2 {
                bad.push("geometry phải có ít nhất 2 điểm".into());
            } else if haversine_m(r.geometry[0], a) > 400.0 {
                bad.push("geometry không bắt đầu gần điểm đi".into());
            }
            let m = p.matrix(&MatrixQuery {
                sources: vec![a],
                targets: vec![b],
                profile,
                depart_sec_of_day: None,
            });
            match m {
                Err(e) => bad.push(format!("matrix lỗi: {e}")),
                Ok(m) => match m.get(0, 0) {
                    None => bad.push("matrix 1x1 không có giá trị".into()),
                    Some(d) => {
                        if (d - r.duration_s).abs() > 0.05 * r.duration_s + 2.0 {
                            bad.push(format!(
                                "matrix {d:.1}s lệch route {:.1}s quá 5%",
                                r.duration_s
                            ));
                        }
                    }
                },
            }
        }
    }
    bad
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::EngineConfig;
    use road_graph::fixture::{compile_fixture, generate};

    fn engine() -> Arc<Engine> {
        Arc::new(Engine::new(
            compile_fixture(&generate()),
            EngineConfig::default(),
        ))
    }
    fn pts() -> (LatLng, LatLng) {
        (
            LatLng::try_new(10.7705, 106.6905).unwrap(),
            LatLng::try_new(10.7760, 106.7020).unwrap(),
        )
    }

    #[test]
    fn all_adapters_satisfy_the_same_contract() {
        let e = engine();
        let (a, b) = pts();
        let own = OwnEngineProvider {
            engine: Arc::clone(&e),
        };
        let osrm = OsrmHttpProvider::new(
            "http://osrm.invalid:5000/",
            MockOsrmTransport::new(Arc::clone(&e)),
        );
        let line = StraightLineProvider::default();
        for p in [&own as &dyn RoutingProvider, &osrm, &line] {
            assert!(
                contract_violations(p, a, b, Profile::Car, 60.0).is_empty(),
                "{}",
                p.name()
            );
        }
    }

    #[test]
    fn osrm_url_uses_lng_lat_order_and_semicolons() {
        let e = engine();
        let (a, b) = pts();
        let osrm = OsrmHttpProvider::new("http://h:1", MockOsrmTransport::new(e));
        let url = osrm.route_url(&RouteQuery {
            from: a,
            to: b,
            profile: Profile::Car,
            depart_sec_of_day: None,
        });
        assert!(
            url.starts_with("http://h:1/route/v1/car/106.690500,10.770500;106.702000,10.776000?")
        );
    }

    #[test]
    fn osrm_adapter_matches_own_engine_through_the_mock() {
        let e = engine();
        let (a, b) = pts();
        let own = OwnEngineProvider {
            engine: Arc::clone(&e),
        };
        let osrm = OsrmHttpProvider::new("http://h:1", MockOsrmTransport::new(Arc::clone(&e)));
        let q = RouteQuery {
            from: a,
            to: b,
            profile: Profile::Car,
            depart_sec_of_day: None,
        };
        let (x, y) = (own.route(&q).unwrap(), osrm.route(&q).unwrap());
        assert!((x.duration_s - y.duration_s).abs() < 1e-9);
        assert!((x.distance_m - y.distance_m).abs() < 1e-6);
        assert!(y.geometry.len() >= 2);
    }

    #[test]
    fn fallback_uses_secondary_when_primary_fails() {
        struct Broken;
        impl RoutingProvider for Broken {
            fn name(&self) -> &str {
                "broken"
            }
            fn route(&self, _: &RouteQuery) -> Result<RouteAnswer, ProviderError> {
                Err(ProviderError::Transport("timeout".into()))
            }
            fn matrix(&self, _: &MatrixQuery) -> Result<MatrixAnswer, ProviderError> {
                Err(ProviderError::Transport("timeout".into()))
            }
        }
        let (a, b) = pts();
        let f = FallbackProvider {
            primary: Box::new(Broken),
            secondary: Box::new(StraightLineProvider::default()),
        };
        let r = f
            .route(&RouteQuery {
                from: a,
                to: b,
                profile: Profile::Car,
                depart_sec_of_day: None,
            })
            .unwrap();
        assert_eq!(r.quality, Quality::Estimated);
    }

    #[test]
    fn valhalla_request_has_costing_and_two_locations() {
        let (a, b) = pts();
        let v = valhalla_route_request(&RouteQuery {
            from: a,
            to: b,
            profile: Profile::Motorbike,
            depart_sec_of_day: None,
        });
        assert_eq!(v["costing"], "motor_scooter");
        assert_eq!(v["locations"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn osrm_error_codes_map_to_errors() {
        let e = engine();
        let osrm = OsrmHttpProvider::new("http://h:1", MockOsrmTransport::new(e));
        assert!(matches!(
            osrm.parse_route(r#"{"code":"NoRoute"}"#),
            Err(ProviderError::NoRoute(_))
        ));
        assert!(matches!(
            osrm.parse_route("not json"),
            Err(ProviderError::BadResponse(_))
        ));
    }
}
