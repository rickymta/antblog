//! Hiện thực `ledger.v1.Ledger`: đổi message protobuf sang kiểu miền, gọi kho, đổi lỗi sang `Status`.

use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use ledger_core::{AccountId, JournalEntry, LedgerError, Money, Posting};
use tonic::{Request, Response, Status};

use crate::interceptor::{RequestContext, ascii};
use crate::pb::ledger_server::Ledger;
use crate::pb::{GetBalanceRequest, GetBalanceResponse, PostEntryRequest, PostEntryResponse};
use crate::status::ledger_error_status;
use crate::store::{LedgerStore, StoreError};

/// Điều handler đã thấy ở request gần nhất, để lab kiểm chứng interceptor và deadline.
#[derive(Debug, Default)]
pub struct Probe {
    last: Mutex<Option<RequestContext>>,
}

impl Probe {
    /// Bản sao thông tin request gần nhất.
    pub fn last(&self) -> Option<RequestContext> {
        self.last.lock().expect("mutex không bị poison").clone()
    }
}

/// Service gRPC, generic theo kho.
#[derive(Debug)]
pub struct LedgerService<S> {
    store: Arc<S>,
    probe: Arc<Probe>,
    // `false`: để mặc tầng timeout của tonic (dùng để so sánh trong lab).
    own_deadline: bool,
}

impl<S: LedgerStore> LedgerService<S> {
    /// Bao một kho và một `Probe` dùng chung với bên ngoài.
    pub fn new(store: S, probe: Arc<Probe>) -> LedgerService<S> {
        LedgerService {
            store: Arc::new(store),
            probe,
            own_deadline: true,
        }
    }

    /// Tắt việc handler tự đếm deadline: chỉ còn tầng timeout có sẵn của tonic.
    #[must_use]
    pub fn without_own_deadline(mut self) -> LedgerService<S> {
        self.own_deadline = false;
        self
    }

    // Lấy RequestContext do interceptor để lại và lưu cho Probe.
    fn context<T>(&self, request: &Request<T>) -> RequestContext {
        let ctx = request
            .extensions()
            .get::<RequestContext>()
            .cloned()
            .expect("interceptor luôn đặt RequestContext");
        *self.probe.last.lock().expect("mutex không bị poison") = Some(ctx.clone());
        ctx
    }
}

#[tonic::async_trait]
impl<S: LedgerStore> Ledger for LedgerService<S> {
    async fn get_balance(
        &self,
        request: Request<GetBalanceRequest>,
    ) -> Result<Response<GetBalanceResponse>, Status> {
        let ctx = self.context(&request);
        let account =
            AccountId::new(&request.get_ref().account).map_err(|e| ledger_error_status(&e))?;
        let balance = within_deadline(self.own_deadline, &ctx, async {
            self.store.balance(&account).await.map_err(|e| match e {
                // Với đọc, tài khoản không tồn tại là NOT_FOUND; với ghi nó là FAILED_PRECONDITION.
                StoreError::UnknownAccount(a) => {
                    Status::not_found(format!("tài khoản {a} chưa mở"))
                }
                other => Status::from(other),
            })
        })
        .await?;
        let mut response = Response::new(GetBalanceResponse {
            account: account.to_string(),
            balance_dong: balance.dong(),
            // Giờ của server khi trả lời. Production nên lấy mốc từ chính truy vấn số dư.
            as_of: Some(SystemTime::now().into()),
        });
        response
            .metadata_mut()
            .insert("x-request-id", ascii(&ctx.request_id)?);
        Ok(response)
    }

    async fn post_entry(
        &self,
        request: Request<PostEntryRequest>,
    ) -> Result<Response<PostEntryResponse>, Status> {
        let ctx = self.context(&request);
        let entry = entry_from_request(request.get_ref()).map_err(|e| ledger_error_status(&e))?;
        let outcome = within_deadline(self.own_deadline, &ctx, async {
            Ok(self.store.post(&entry).await?)
        })
        .await?;
        let mut response = Response::new(PostEntryResponse {
            entry_id: outcome.entry_id,
            replayed: outcome.replayed,
        });
        response
            .metadata_mut()
            .insert("x-request-id", ascii(&ctx.request_id)?);
        Ok(response)
    }
}

// Chạy `work` trong ngân sách của deadline. Hết giờ thì future bị thả (việc dang dở dừng lại,
// giao dịch PostgreSQL chưa commit tự rollback) và client nhận DEADLINE_EXCEEDED.
async fn within_deadline<T>(
    own_deadline: bool,
    ctx: &RequestContext,
    work: impl Future<Output = Result<T, Status>>,
) -> Result<T, Status> {
    match ctx.budget().filter(|_| own_deadline) {
        None => work.await,
        Some(budget) => tokio::time::timeout(budget, work)
            .await
            .unwrap_or_else(|_| Err(Status::deadline_exceeded("hết deadline ở server"))),
    }
}

// Message sang `JournalEntry`: mọi bất biến (tối thiểu 2 posting, tổng bằng 0) do `ledger-core` kiểm.
fn entry_from_request(req: &PostEntryRequest) -> Result<JournalEntry, LedgerError> {
    let postings = req
        .postings
        .iter()
        .map(|p| {
            Ok(Posting::new(
                AccountId::new(&p.account)?,
                Money::from_dong(p.amount_dong),
            ))
        })
        .collect::<Result<Vec<_>, LedgerError>>()?;
    JournalEntry::try_new(&req.idempotency_key, postings)
}
