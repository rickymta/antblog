//! Adapter HTTP: đổi JSON thành lời gọi use case và ngược lại. Không có luật nghiệp vụ ở đây.

use std::sync::Arc;

use application::{Clock, EventPublisher, LedgerStore, SettleError, StoreError};
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use domain::{AccountId, Trip};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::compose::Services;

/// Thân yêu cầu `POST /trips/settle`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettleRequest {
    trip_id: String,
    customer_id: String,
    driver_id: String,
    distance_km: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StepDto {
    key: String,
    entry_id: String,
    replayed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BalanceDto {
    account: String,
    balance_dong: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettleResponse {
    steps: Vec<StepDto>,
    balances: Vec<BalanceDto>,
}

// Lỗi tối giản dạng problem+json (hợp đồng đầy đủ là bài 32).
fn problem(status: StatusCode, title: &str, detail: &str) -> Response {
    let body = json!({ "title": title, "status": status.as_u16(), "detail": detail });
    (
        status,
        [(header::CONTENT_TYPE, "application/problem+json")],
        body.to_string(),
    )
        .into_response()
}

fn store_problem(e: &StoreError) -> Response {
    match e {
        StoreError::IdempotencyConflict(_) => {
            problem(StatusCode::CONFLICT, "Xung đột idempotency", &e.to_string())
        }
        StoreError::UnknownAccount(_) | StoreError::Ledger(_) => problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Bút toán không hợp lệ",
            &e.to_string(),
        ),
        // Chi tiết lỗi hạ tầng không trả cho client.
        StoreError::Storage(_) => problem(StatusCode::INTERNAL_SERVER_ERROR, "Lỗi kho lưu trữ", ""),
    }
}

async fn settle<S, P, C>(
    State(svc): State<Arc<Services<S, P, C>>>,
    Json(req): Json<SettleRequest>,
) -> Response
where
    S: LedgerStore + 'static,
    P: EventPublisher + 'static,
    C: Clock + 'static,
{
    let trip = match Trip::new(
        &req.trip_id,
        &req.customer_id,
        &req.driver_id,
        req.distance_km,
    ) {
        Ok(t) => t,
        Err(e) => {
            return problem(
                StatusCode::UNPROCESSABLE_ENTITY,
                "Chuyến không hợp lệ",
                &e.to_string(),
            );
        }
    };
    match svc.settle.execute(&trip).await {
        Ok(receipt) => Json(SettleResponse {
            steps: receipt
                .steps
                .into_iter()
                .map(|s| StepDto {
                    key: s.key,
                    entry_id: s.entry_id.to_string(),
                    replayed: s.replayed,
                })
                .collect(),
            balances: receipt
                .balances
                .into_iter()
                .map(|(a, m)| BalanceDto {
                    account: a.as_str().to_string(),
                    balance_dong: m.dong(),
                })
                .collect(),
        })
        .into_response(),
        Err(SettleError::Trip(e)) => problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Chuyến không hợp lệ",
            &e.to_string(),
        ),
        Err(SettleError::Store(e)) => store_problem(&e),
        Err(SettleError::Publish(_)) => {
            problem(StatusCode::BAD_GATEWAY, "Chưa phát được sự kiện", "")
        }
    }
}

async fn balance<S, P, C>(
    State(svc): State<Arc<Services<S, P, C>>>,
    Path(id): Path<String>,
) -> Response
where
    S: LedgerStore + 'static,
    P: EventPublisher + 'static,
    C: Clock + 'static,
{
    let account = match AccountId::new(&id) {
        Ok(a) => a,
        Err(e) => {
            return problem(
                StatusCode::UNPROCESSABLE_ENTITY,
                "Mã tài khoản không hợp lệ",
                &e.to_string(),
            );
        }
    };
    match svc.balance.execute(&account).await {
        Ok(m) => Json(BalanceDto {
            account: id,
            balance_dong: m.dong(),
        })
        .into_response(),
        Err(e) => store_problem(&e),
    }
}

/// Router của service: `POST /trips/settle`, `GET /accounts/{id}/balance`.
pub fn router<S, P, C>(services: Arc<Services<S, P, C>>) -> Router
where
    S: LedgerStore + 'static,
    P: EventPublisher + 'static,
    C: Clock + 'static,
{
    Router::new()
        .route("/trips/settle", post(settle::<S, P, C>))
        .route("/accounts/{id}/balance", get(balance::<S, P, C>))
        .with_state(services)
}
