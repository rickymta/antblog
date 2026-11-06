//! HTTP: `/healthz`, `/readyz` và `POST /v1/entries`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use ledger_core::{AccountId, JournalEntry, Money, Posting};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;

use crate::events::PostingDto;
use crate::probes::Probes;
use crate::store::{PostOutcome, display_id, post_entry};

/// Trạng thái dùng chung của các handler.
#[derive(Clone)]
pub struct AppState {
    /// Pool PostgreSQL.
    pub pool: PgPool,
    /// Kiểm phụ thuộc cho `/readyz`.
    pub probes: Arc<Probes>,
    /// Đặt `true` khi bắt đầu tắt: `/readyz` trả 503.
    pub draining: Arc<AtomicBool>,
    /// Topic phát `LedgerEntryPosted` (ghi vào outbox).
    pub topic_out: Arc<str>,
    /// Bước chậm mô phỏng trong transaction (0 khi chạy thật).
    pub hold: Duration,
}

/// Dựng router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/v1/entries", post(post_entries))
        .with_state(state)
}

/// Liveness: chỉ nói tiến trình còn chạy và còn nhận request. KHÔNG chạm DB hay Kafka.
async fn healthz() -> &'static str {
    "ok"
}

/// Readiness: 503 khi đang tắt hoặc khi một phụ thuộc không trả lời.
async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    if state.draining.load(Ordering::Acquire) {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "draining" })),
        );
    }
    let readiness = state.probes.check().await;
    if readiness.is_ready() {
        (StatusCode::OK, Json(json!({ "status": "ready" })))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "status": "not-ready",
                "postgres": readiness.postgres.err(),
                "kafka": readiness.kafka.err(),
            })),
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EntryRequest {
    idempotency_key: String,
    postings: Vec<PostingDto>,
}

/// 201 khi tạo mới, 200 khi replay, 422 khi bút toán sai, 409 khi key xung đột.
async fn post_entries(State(state): State<AppState>, Json(body): Json<EntryRequest>) -> Response {
    let built = body
        .postings
        .iter()
        .map(|p| {
            Ok(Posting::new(
                AccountId::new(&p.account)?,
                Money::from_dong(p.amount_dong),
            ))
        })
        .collect::<Result<Vec<_>, ledger_core::LedgerError>>()
        .and_then(|postings| JournalEntry::try_new(&body.idempotency_key, postings));
    let entry = match built {
        Ok(entry) => entry,
        Err(err) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({ "error": err.to_string() })),
            )
                .into_response();
        }
    };
    match post_entry(&state.pool, &entry, &state.topic_out, state.hold).await {
        Ok(PostOutcome::Created(id)) => created(StatusCode::CREATED, id, &body.idempotency_key),
        Ok(PostOutcome::Replayed(id)) => created(StatusCode::OK, id, &body.idempotency_key),
        Ok(PostOutcome::Conflict) => (
            StatusCode::CONFLICT,
            Json(json!({ "error": "idempotencyKey đã dùng cho bút toán khác nội dung" })),
        )
            .into_response(),
        Err(err) => {
            tracing::error!("ghi bút toán lỗi: {err:#}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "lỗi nội bộ" })),
            )
                .into_response()
        }
    }
}

fn created(status: StatusCode, id: i64, key: &str) -> Response {
    (
        status,
        Json(json!({ "entryId": display_id(id), "idempotencyKey": key })),
    )
        .into_response()
}
