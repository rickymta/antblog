//! gRPC `ledger.v1.Ledger`: `GetBalance` và `PostEntry` dùng chung `Store` với HTTP.

use tonic::{Request, Response, Status};
use tracing::{Instrument, error, info, info_span};

use crate::store::{PostOutcome, Store, StoreError};

/// Mã sinh từ `proto/ledger/v1/ledger.proto` bởi `build.rs`.
pub mod pb {
    tonic::include_proto!("ledger.v1");
}

use pb::ledger_server::Ledger;

/// Triển khai service gRPC.
pub struct GrpcService {
    store: Store,
}

impl GrpcService {
    /// Dựng từ kho dữ liệu.
    pub fn new(store: Store) -> GrpcService {
        GrpcService { store }
    }
}

// `x-request-id` trong metadata; không có thì sinh mới, giống lớp HTTP.
fn request_id<T>(request: &Request<T>) -> String {
    request
        .metadata()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map_or_else(|| uuid::Uuid::new_v4().to_string(), str::to_string)
}

fn status_of(err: &StoreError) -> Status {
    match err {
        StoreError::Invalid(_) | StoreError::Domain(_) => Status::invalid_argument(err.to_string()),
        StoreError::IdempotencyConflict { .. } => Status::already_exists(err.to_string()),
        StoreError::InjectedCrash | StoreError::Db(_) => {
            error!(%err, "lỗi nội bộ");
            Status::internal("lỗi nội bộ")
        }
    }
}

#[tonic::async_trait]
impl Ledger for GrpcService {
    async fn get_balance(
        &self,
        request: Request<pb::GetBalanceRequest>,
    ) -> Result<Response<pb::GetBalanceResponse>, Status> {
        let span = info_span!("grpc", request_id = %request_id(&request), method = "GetBalance");
        async {
            let account = request.into_inner().account;
            info!(%account, "đọc số dư");
            match self.store.balance(&account).await {
                Ok(Some(balance_dong)) => Ok(Response::new(pb::GetBalanceResponse {
                    account,
                    balance_dong,
                    as_of: Some(std::time::SystemTime::now().into()),
                })),
                Ok(None) => Err(Status::not_found(format!("không có tài khoản {account}"))),
                Err(err) => Err(status_of(&StoreError::Db(err))),
            }
        }
        .instrument(span)
        .await
    }

    async fn post_entry(
        &self,
        request: Request<pb::PostEntryRequest>,
    ) -> Result<Response<pb::PostEntryResponse>, Status> {
        let span = info_span!("grpc", request_id = %request_id(&request), method = "PostEntry");
        async {
            let body = request.into_inner();
            info!(key = %body.idempotency_key, "ghi bút toán");
            let postings = body
                .postings
                .into_iter()
                .map(|p| (p.account, p.amount_dong))
                .collect();
            match self.store.post_entry(&body.idempotency_key, postings).await {
                Ok(PostOutcome::Created(event)) => Ok(Response::new(pb::PostEntryResponse {
                    entry_id: event.entry_id,
                    replayed: false,
                })),
                Ok(PostOutcome::Replayed { entry_id }) => {
                    Ok(Response::new(pb::PostEntryResponse {
                        entry_id,
                        replayed: true,
                    }))
                }
                Err(err) => Err(status_of(&err)),
            }
        }
        .instrument(span)
        .await
    }
}
