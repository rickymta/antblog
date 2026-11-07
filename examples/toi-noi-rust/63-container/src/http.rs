//! Router axum: `/healthz` (sống), `/readyz` (sẵn sàng nhận việc) và `/v1/fare`.
//!
//! Hai probe tách nhau có chủ đích: sống = tiến trình còn trả lời; sẵn sàng = phụ thuộc cần thiết dùng
//! được. DB chết thì `/readyz` 503 nhưng `/healthz` vẫn 200, để orchestrator ngừng gửi việc chứ không
//! giết một tiến trình không có lỗi.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::fare::{self, fare_for_km};

/// Hạn cho một lần kiểm DB trong `/readyz`.
const DB_CHECK_TIMEOUT: Duration = Duration::from_secs(2);

/// Trạng thái dùng chung của các handler.
#[derive(Clone)]
pub struct AppState {
    db: Option<PgPool>,
    draining: Arc<AtomicBool>,
}

impl AppState {
    /// Chế độ không DB.
    #[must_use]
    pub fn without_db() -> AppState {
        AppState {
            db: None,
            draining: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Có DB. Pool tạo lazy: service vẫn lên được khi DB chưa sẵn sàng, `/readyz` sẽ báo 503.
    ///
    /// # Errors
    /// Chuỗi kết nối không parse được.
    pub fn with_db(url: &str) -> Result<AppState, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .acquire_timeout(DB_CHECK_TIMEOUT)
            .connect_lazy(url)?;
        Ok(AppState {
            db: Some(pool),
            draining: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Bắt đầu tắt: từ đây `/readyz` trả 503, `/healthz` vẫn 200.
    pub fn begin_drain(&self) {
        self.draining.store(true, Ordering::SeqCst);
    }
}

/// Dựng router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/v1/fare", get(get_fare))
        .with_state(state)
}

async fn healthz() -> &'static str {
    "ok"
}

async fn readyz(State(state): State<AppState>) -> Response {
    if state.draining.load(Ordering::SeqCst) {
        return not_ready("draining");
    }
    let Some(pool) = &state.db else {
        return Json(json!({ "status": "ready", "database": "not-configured" })).into_response();
    };
    let check = sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(pool);
    match tokio::time::timeout(DB_CHECK_TIMEOUT, check).await {
        Ok(Ok(_)) => Json(json!({ "status": "ready", "database": "ok" })).into_response(),
        _ => not_ready("database"),
    }
}

fn not_ready(reason: &str) -> Response {
    let body = Json(json!({ "status": "unavailable", "reason": reason }));
    (StatusCode::SERVICE_UNAVAILABLE, body).into_response()
}

#[derive(Deserialize)]
struct FareQuery {
    km: i64,
}

async fn get_fare(Query(q): Query<FareQuery>) -> Response {
    match fare_for_km(q.km) {
        Some(fare) => Json(fare).into_response(),
        None => {
            let body = Json(json!({
                "error": "invalid-km",
                "detail": format!("km phải trong 1..={}", fare::MAX_KM),
            }));
            (StatusCode::BAD_REQUEST, body).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    async fn get(state: &AppState, uri: &str) -> (StatusCode, String) {
        let req = Request::get(uri).body(Body::empty()).unwrap();
        let res = router(state.clone()).oneshot(req).await.unwrap();
        let status = res.status();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn healthz_is_ok() {
        let (status, body) = get(&AppState::without_db(), "/healthz").await;
        assert_eq!((status, body.as_str()), (StatusCode::OK, "ok"));
    }

    #[tokio::test]
    async fn readyz_without_database_is_ready() {
        let (status, body) = get(&AppState::without_db(), "/readyz").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("not-configured"), "{body}");
    }

    #[tokio::test]
    async fn draining_flips_readyz_but_not_healthz() {
        let state = AppState::without_db();
        state.begin_drain();
        assert_eq!(
            get(&state, "/readyz").await.0,
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(get(&state, "/healthz").await.0, StatusCode::OK);
    }

    #[tokio::test]
    async fn fare_for_five_km() {
        let (status, body) = get(&AppState::without_db(), "/v1/fare?km=5").await;
        assert_eq!(status, StatusCode::OK);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["fareDong"], 25_500);
        assert_eq!(v["platformFeeDong"], 5_100);
        assert_eq!(v["driverDong"], 20_400);
    }

    #[tokio::test]
    async fn fare_rejects_zero_km() {
        let (status, body) = get(&AppState::without_db(), "/v1/fare?km=0").await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.contains("invalid-km"), "{body}");
    }
}
