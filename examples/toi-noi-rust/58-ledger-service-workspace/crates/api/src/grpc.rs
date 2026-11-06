//! gRPC `ledger.v1.Ledger` (bài 35): `GetBalance` và `PostEntry` gọi cùng use case với HTTP,
//! nên hai giao thức không thể trả hai số khác nhau. JWT kiểm ở interceptor.

use std::sync::Arc;

use ledger_application::{Error, Ledger, LedgerStore};
use ledger_domain::LedgerError;
use ledger_domain::hold::HoldError;
use ledger_domain::refund::RefundError;
use ledger_domain::trip::TripError;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::{Request, Response, Status};
use tracing::{Instrument, error, info, info_span};

use crate::auth::{AuthConfig, AuthUser, Policy, Readers, Writers, bearer_token};

/// Mã sinh từ `proto/ledger/v1/ledger.proto` bởi `build.rs`.
pub mod pb {
    tonic::include_proto!("ledger.v1");
}

use pb::ledger_server::{Ledger as LedgerRpc, LedgerServer};

/// Interceptor JWT: kiểm `authorization: Bearer <token>` rồi gắn [`AuthUser`] vào extensions.
#[derive(Clone)]
pub struct JwtInterceptor {
    auth: Arc<AuthConfig>,
}

impl Interceptor for JwtInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let token = request
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(bearer_token)
            .ok_or_else(|| Status::unauthenticated("thiếu hoặc sai header authorization"))?;
        match self.auth.verify(token) {
            Ok(claims) => {
                request
                    .extensions_mut()
                    .insert(AuthUser::from_claims(&claims));
                Ok(request)
            }
            Err(reason) => {
                // Chỉ ghi loại lỗi. Không ghi token, không ghi claim.
                tracing::warn!(%reason, "từ chối token gRPC");
                Err(Status::unauthenticated("token không hợp lệ"))
            }
        }
    }
}

/// Service gRPC đã bọc interceptor JWT, sẵn sàng `add_service`.
pub fn service<S: LedgerStore>(
    ledger: Arc<Ledger<S>>,
    auth: Arc<AuthConfig>,
) -> InterceptedService<LedgerServer<GrpcService<S>>, JwtInterceptor> {
    InterceptedService::new(
        LedgerServer::new(GrpcService { ledger }),
        JwtInterceptor { auth },
    )
}

/// Triển khai service gRPC.
pub struct GrpcService<S: LedgerStore> {
    ledger: Arc<Ledger<S>>,
}

// `x-request-id` trong metadata; không có thì sinh mới, giống lớp HTTP.
fn request_id<T>(request: &Request<T>) -> String {
    request
        .metadata()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map_or_else(|| uuid::Uuid::new_v4().to_string(), str::to_string)
}

fn require<T, P: Policy>(request: &Request<T>) -> Result<(), Status> {
    let user = request
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| Status::unauthenticated("chưa xác thực"))?;
    if user.has_any(P::ALLOWED) {
        Ok(())
    } else {
        Err(Status::permission_denied("không đủ quyền"))
    }
}

/// Mã gRPC cho từng lỗi. Vét cạn, không có nhánh `_`.
pub fn status_of(err: &Error) -> Status {
    let message = err.to_string();
    match err {
        Error::Invalid(_) | Error::Trip(TripError::NonPositiveFare(_) | TripError::EmptyTripId) => {
            Status::invalid_argument(message)
        }
        Error::Ledger(e) => match e {
            LedgerError::EmptyAccountId
            | LedgerError::EmptyIdempotencyKey
            | LedgerError::TooFewPostings { .. }
            | LedgerError::Unbalanced { .. }
            | LedgerError::Overflow
            | LedgerError::InvalidEntryId { .. } => Status::invalid_argument(message),
            LedgerError::IdempotencyConflict { .. } => Status::already_exists(message),
            LedgerError::UnknownEntry { .. } => Status::not_found(message),
            LedgerError::InvalidFee { .. } => internal(err),
        },
        Error::Trip(TripError::Ledger(_)) | Error::Storage(_) => internal(err),
        Error::Hold(e) => match e {
            HoldError::NonPositive(_) | HoldError::EmptyKey | HoldError::NonPositiveTtl => {
                Status::invalid_argument(message)
            }
            HoldError::InsufficientAvailable { .. } => Status::failed_precondition(message),
            HoldError::KeyConflict { .. } => Status::already_exists(message),
        },
        Error::Refund(e) => match e {
            RefundError::NonPositive(_) | RefundError::BadSequence => {
                Status::invalid_argument(message)
            }
            RefundError::ExceedsRefundable { .. } => Status::failed_precondition(message),
            RefundError::KeyConflict { .. } => Status::already_exists(message),
            RefundError::Ledger(_) => internal(err),
        },
        Error::IdempotencyConflict { .. } => Status::already_exists(message),
        Error::UnknownAccount(_) | Error::UnknownTrip(_) => Status::not_found(message),
        Error::Unavailable(_) => Status::unavailable("tạm thời không sẵn sàng"),
    }
}

fn internal(err: &Error) -> Status {
    error!(%err, "lỗi nội bộ");
    // Không đưa chi tiết hạ tầng ra ngoài dây.
    Status::internal("lỗi nội bộ")
}

#[tonic::async_trait]
impl<S: LedgerStore> LedgerRpc for GrpcService<S> {
    async fn get_balance(
        &self,
        request: Request<pb::GetBalanceRequest>,
    ) -> Result<Response<pb::GetBalanceResponse>, Status> {
        require::<_, Readers>(&request)?;
        let span = info_span!("grpc", request_id = %request_id(&request), method = "GetBalance");
        async {
            let account = request.into_inner().account;
            info!(%account, "đọc số dư");
            let balance = self
                .ledger
                .balance(&account)
                .await
                .map_err(|e| status_of(&e))?;
            let now = self.ledger.now();
            Ok(Response::new(pb::GetBalanceResponse {
                account,
                balance_dong: balance.dong(),
                as_of: Some(prost_types::Timestamp {
                    seconds: now.as_second(),
                    nanos: now.subsec_nanosecond(),
                }),
            }))
        }
        .instrument(span)
        .await
    }

    async fn post_entry(
        &self,
        request: Request<pb::PostEntryRequest>,
    ) -> Result<Response<pb::PostEntryResponse>, Status> {
        require::<_, Writers>(&request)?;
        let span = info_span!("grpc", request_id = %request_id(&request), method = "PostEntry");
        async {
            let body = request.into_inner();
            info!(key = %body.idempotency_key, "ghi bút toán");
            let lines: Vec<(String, i64)> = body
                .postings
                .into_iter()
                .map(|p| (p.account, p.amount_dong))
                .collect();
            let outcome = self
                .ledger
                .post_entry(&body.idempotency_key, &lines)
                .await
                .map_err(|e| status_of(&e))?;
            Ok(Response::new(pb::PostEntryResponse {
                entry_id: outcome.entry_id().to_string(),
                replayed: outcome.replayed(),
            }))
        }
        .instrument(span)
        .await
    }
}
