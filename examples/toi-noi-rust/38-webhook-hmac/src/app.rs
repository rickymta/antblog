//! Router axum: `POST /webhooks/vixanh`. Đọc body thô trước, xác thực, rồi mới parse JSON.

use std::sync::{Arc, RwLock};

use axum::Router;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use crate::signature::{Clock, KeyRing, SIGNATURE_HEADER, TOLERANCE_SECS, VerifyError, verify};
use crate::store::{Outcome, Webhook, receive_webhook};

/// Trạng thái dùng chung của handler.
#[derive(Clone)]
pub struct AppState {
    /// Pool PostgreSQL.
    pub pool: PgPool,
    /// Khóa đang hợp lệ; đổi được lúc chạy khi xoay vòng.
    pub keys: Arc<RwLock<KeyRing>>,
    /// Đồng hồ (test cố định được).
    pub clock: Arc<dyn Clock>,
}

/// Scheme trong `WWW-Authenticate` của response 401.
pub const AUTH_SCHEME: &str = "ViXanh-HMAC-SHA256";

/// Dạng JSON VíXanh gửi (định dạng giả định của lab).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VixanhEvent {
    #[allow(dead_code)] // lab không dùng eventId; production lưu để truy vết
    event_id: String,
    #[serde(rename = "type")]
    kind: String,
    external_ref: String,
    trip_id: String,
    amount_dong: i64,
}

/// Dựng router. Giới hạn body 64 KiB: webhook thật nhỏ, và `Bytes` đọc hết body vào RAM.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/webhooks/vixanh", post(vixanh_webhook))
        .layer(DefaultBodyLimit::max(64 * 1024))
        .with_state(state)
}

fn reply(status: StatusCode, body: serde_json::Value) -> Response {
    (status, axum::Json(body)).into_response()
}

// `Bytes` là extractor cuối cùng và duy nhất lấy body: ta có đúng những byte VíXanh đã ký.
// Nếu dùng `Json<VixanhEvent>` thì body bị parse trước khi ta thấy byte thô.
async fn vixanh_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let verdict = match headers.get(SIGNATURE_HEADER).and_then(|v| v.to_str().ok()) {
        None => Err(VerifyError::MissingHeader),
        Some(header) => {
            // Khóa đọc xong là nhả ngay, không giữ qua `.await`.
            let ring = state.keys.read().expect("khóa RwLock không bị poison");
            verify(
                &ring,
                header,
                &body,
                state.clock.now_epoch(),
                TOLERANCE_SECS,
            )
        }
    };
    if let Err(reason) = verdict {
        // Ra ngoài chỉ một câu trả lời cho mọi lý do, để không dạy kẻ tấn công điều gì.
        eprintln!("webhook bị từ chối: {reason:?}");
        // RFC 9110: 401 phải kèm `WWW-Authenticate`. Tên scheme là giả định của lab.
        return (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, AUTH_SCHEME)],
            axum::Json(json!({ "error": "invalid_signature" })),
        )
            .into_response();
    }

    // Từ đây body là thật: parse bằng serde_json trên đúng byte đã xác thực.
    let Ok(event) = serde_json::from_slice::<VixanhEvent>(&body) else {
        return reply(
            StatusCode::BAD_REQUEST,
            json!({ "error": "invalid_payload" }),
        );
    };
    if event.kind != "payment.captured" {
        return reply(StatusCode::OK, json!({ "status": "ignored" }));
    }
    let hook = Webhook {
        external_ref: event.external_ref,
        trip_id: event.trip_id,
        amount_dong: event.amount_dong,
    };
    match receive_webhook(&state.pool, &hook).await {
        Ok(Outcome::Pending) => reply(StatusCode::ACCEPTED, json!({ "status": "pending" })),
        Ok(Outcome::Applied { entry_id }) => reply(
            StatusCode::OK,
            json!({ "status": "applied", "entryId": entry_id }),
        ),
        Ok(other) => reply(StatusCode::OK, json!({ "status": other.label() })),
        Err(err) => {
            // Chưa lưu gì (transaction đã rollback): 503 để VíXanh gửi lại.
            eprintln!("webhook lỗi hạ tầng: {err:#}");
            reply(
                StatusCode::SERVICE_UNAVAILABLE,
                json!({ "error": "temporarily_unavailable" }),
            )
        }
    }
}
