//! Đồ dùng của lab: client tonic, kho chậm để thử deadline và các kịch bản chạy chung cho `--verify` và test.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use ledger_core::{AccountId, JournalEntry, Money};
use tokio::net::TcpListener;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tonic::{Code, Request, Status};

use crate::interceptor::{LAB_TOKEN, ascii};
use crate::pb::ledger_client::LedgerClient;
use crate::pb::{GetBalanceRequest, PostEntryRequest, Posting};
use crate::server;
use crate::store::{LedgerStore, MemoryStore, PostOutcome, StoreError};

/// Interceptor phía client: gắn `authorization` và `x-request-id` vào mọi lời gọi.
#[derive(Clone, Debug)]
pub struct ClientAuth {
    token: Option<String>,
    request_id: String,
}

impl Interceptor for ClientAuth {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        if let Some(token) = &self.token {
            req.metadata_mut()
                .insert("authorization", ascii(&format!("Bearer {token}"))?);
        }
        req.metadata_mut()
            .insert("x-request-id", ascii(&self.request_id)?);
        Ok(req)
    }
}

/// Client tonic đã gắn interceptor.
pub type Client = LedgerClient<InterceptedService<Channel, ClientAuth>>;

/// Kết nối tới server qua HTTP/2 không TLS (`http://`).
///
/// # Errors
/// Không kết nối được.
pub async fn connect(addr: SocketAddr, token: Option<&str>, request_id: &str) -> Result<Client> {
    let channel = Channel::from_shared(format!("http://{addr}"))?
        .connect()
        .await
        .context("kết nối gRPC")?;
    Ok(LedgerClient::with_interceptor(
        channel,
        ClientAuth {
            token: token.map(str::to_owned),
            request_id: request_id.to_owned(),
        },
    ))
}

/// Request `PostEntry` từ danh sách (tài khoản, số đồng).
pub fn post_request(key: &str, postings: &[(&str, i64)]) -> PostEntryRequest {
    PostEntryRequest {
        idempotency_key: key.to_owned(),
        postings: postings
            .iter()
            .map(|(account, amount_dong)| Posting {
                account: (*account).to_owned(),
                amount_dong: *amount_dong,
            })
            .collect(),
    }
}

/// Bút toán thu cước chuyến `TN-0001`: khách -25.500, nền tảng +5.100, tài xế +20.400.
pub fn capture_request() -> PostEntryRequest {
    post_request(
        "trip:TN-0001:capture",
        &[
            ("customer:CU-001", -25_500),
            ("platform:revenue", 5_100),
            ("driver:DR-001", 20_400),
        ],
    )
}

/// Mọi quan sát của kịch bản chính, để `--verify` và test cùng assert.
#[derive(Debug)]
pub struct FlowReport {
    /// Mã khi không gửi `authorization`.
    pub no_token: Code,
    /// Mã khi gửi token sai.
    pub wrong_token: Code,
    /// (mã bút toán, replayed) lần ghi đầu.
    pub first: (String, bool),
    /// (mã bút toán, replayed) lần ghi lại cùng request.
    pub second: (String, bool),
    /// Số dư (khách, nền tảng, tài xế) sau khi ghi.
    pub balances: (i64, i64, i64),
    /// Mã khi cùng key nhưng khác số tiền.
    pub conflict: Code,
    /// Mã khi bút toán lệch tổng.
    pub unbalanced: Code,
    /// Mã khi ghi vào tài khoản chưa mở.
    pub unknown_account_post: Code,
    /// Mã khi đọc số dư tài khoản chưa mở.
    pub unknown_account_read: Code,
    /// `x-request-id` server trả lại trong metadata phản hồi.
    pub echoed_request_id: String,
    /// `x-request-id` mà handler thấy.
    pub seen_request_id: String,
}

async fn code_of<T>(result: Result<tonic::Response<T>, Status>) -> Code {
    match result {
        Ok(_) => Code::Ok,
        Err(status) => status.code(),
    }
}

/// Chạy kịch bản chính trên một kho bất kỳ: server thật trên `127.0.0.1:0`, client tonic thật.
///
/// # Errors
/// Lỗi hạ tầng (bind, kết nối); lỗi gRPC mong đợi nằm trong `FlowReport`.
pub async fn run_flow<S: LedgerStore>(store: S) -> Result<FlowReport> {
    let server = server::start(store, LAB_TOKEN).await?;
    let request_id = "req-lab-0001";
    let report = {
        let mut anonymous = connect(server.addr, None, request_id).await?;
        let mut wrong = connect(server.addr, Some("sai-token"), request_id).await?;
        let mut client = connect(server.addr, Some(LAB_TOKEN), request_id).await?;

        let no_token = code_of(anonymous.post_entry(capture_request()).await).await;
        let wrong_token = code_of(wrong.post_entry(capture_request()).await).await;

        let first = client.post_entry(capture_request()).await?;
        let echoed_request_id = first
            .metadata()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        let first = first.into_inner();
        let second = client.post_entry(capture_request()).await?.into_inner();

        let mut balance = async |account: &str| -> Result<i64> {
            let response = client
                .get_balance(GetBalanceRequest {
                    account: account.to_owned(),
                })
                .await?
                .into_inner();
            Ok(response.balance_dong)
        };
        let balances = (
            balance("customer:CU-001").await?,
            balance("platform:revenue").await?,
            balance("driver:DR-001").await?,
        );

        let conflict = code_of(
            client
                .post_entry(post_request(
                    "trip:TN-0001:capture",
                    &[("customer:CU-001", -30_000), ("driver:DR-001", 30_000)],
                ))
                .await,
        )
        .await;
        let unbalanced = code_of(
            client
                .post_entry(post_request(
                    "trip:TN-0002:capture",
                    &[("customer:CU-001", -25_500), ("driver:DR-001", 20_400)],
                ))
                .await,
        )
        .await;
        let unknown_account_post = code_of(
            client
                .post_entry(post_request(
                    "trip:TN-0003:capture",
                    &[("customer:CU-001", -1_000), ("driver:DR-999", 1_000)],
                ))
                .await,
        )
        .await;
        let unknown_account_read = code_of(
            client
                .get_balance(GetBalanceRequest {
                    account: "driver:DR-999".to_owned(),
                })
                .await,
        )
        .await;
        let seen_request_id = server
            .probe
            .last()
            .map(|ctx| ctx.request_id)
            .unwrap_or_default();

        FlowReport {
            no_token,
            wrong_token,
            first: (first.entry_id, first.replayed),
            second: (second.entry_id, second.replayed),
            balances,
            conflict,
            unbalanced,
            unknown_account_post,
            unknown_account_read,
            echoed_request_id,
            seen_request_id,
        }
    };
    // Mọi client đã bị thả ở cuối khối trên, nên server tắt êm không phải chờ kết nối còn mở.
    server.stop().await?;
    Ok(report)
}

/// Kho bọc ngoài làm chậm `post` và đếm số lần future bị hủy giữa chừng.
#[derive(Debug)]
pub struct SlowStore<S> {
    inner: S,
    delay: Duration,
    /// Số lần `post` chạy hết.
    pub completed: Arc<AtomicUsize>,
    /// Số lần future `post` bị thả khi chưa xong (server đã bỏ việc).
    pub cancelled: Arc<AtomicUsize>,
}

impl<S> SlowStore<S> {
    /// Bao `inner`, mỗi `post` chờ thêm `delay`.
    pub fn new(inner: S, delay: Duration) -> SlowStore<S> {
        SlowStore {
            inner,
            delay,
            completed: Arc::default(),
            cancelled: Arc::default(),
        }
    }
}

// Thả guard khi chưa `finish` nghĩa là future bị hủy giữa chừng.
struct CancelGuard<'a> {
    done: bool,
    cancelled: &'a AtomicUsize,
}

impl Drop for CancelGuard<'_> {
    fn drop(&mut self) {
        if !self.done {
            self.cancelled.fetch_add(1, Ordering::SeqCst);
        }
    }
}

impl<S: LedgerStore> LedgerStore for SlowStore<S> {
    async fn post(&self, entry: &JournalEntry) -> Result<PostOutcome, StoreError> {
        let mut guard = CancelGuard {
            done: false,
            cancelled: &self.cancelled,
        };
        tokio::time::sleep(self.delay).await;
        let outcome = self.inner.post(entry).await;
        guard.done = true;
        self.completed.fetch_add(1, Ordering::SeqCst);
        outcome
    }

    async fn balance(&self, account: &AccountId) -> Result<Money, StoreError> {
        self.inner.balance(account).await
    }
}

/// Mã client tonic tự sinh khi peer nhận kết nối mà không bao giờ trả lời: không có server tonic nào
/// tham gia, nên `CANCELLED` ở đây đến từ tầng timeout phía client.
///
/// # Errors
/// Lỗi hạ tầng (bind, kết nối).
pub async fn silent_peer_code() -> Result<Code> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("bind 127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let holder = tokio::spawn(async move {
        // Giữ socket mở, không đọc, không ghi.
        let accepted = listener.accept().await;
        tokio::time::sleep(Duration::from_secs(5)).await;
        drop(accepted);
    });
    let mut client = connect(addr, Some(LAB_TOKEN), "req-silent").await?;
    let mut request = Request::new(capture_request());
    request.set_timeout(Duration::from_millis(300));
    let code = client
        .post_entry(request)
        .await
        .err()
        .map_or(Code::Ok, |s| s.code());
    holder.abort();
    Ok(code)
}

/// Quan sát của kịch bản deadline.
#[derive(Debug)]
pub struct DeadlineReport {
    /// Mã client nhận khi đặt deadline ngắn hơn thời gian xử lý.
    pub code: Code,
    /// Thời gian client chờ thực tế.
    pub elapsed: Duration,
    /// Giá trị `grpc-timeout` mà handler thấy.
    pub seen_timeout: Option<String>,
    /// Số lần future của handler bị hủy trước khi xong.
    pub cancelled: usize,
    /// Số lần handler chạy hết sau lần gọi có deadline.
    pub completed_after_deadline: usize,
    /// Mã của lần gọi đối chứng với deadline dài (kho vẫn chậm như cũ).
    pub control_code: Code,
}

/// Deadline 300 ms với kho cần 1200 ms; sau đó gọi đối chứng với deadline 5 s.
/// `own_deadline = false` để mặc tầng timeout của tonic, không để handler tự đếm.
///
/// # Errors
/// Lỗi hạ tầng (bind, kết nối).
pub async fn run_deadline(own_deadline: bool) -> Result<DeadlineReport> {
    let store = SlowStore::new(MemoryStore::new(), Duration::from_millis(1200));
    let (completed, cancelled) = (Arc::clone(&store.completed), Arc::clone(&store.cancelled));
    let server = server::start_with(store, LAB_TOKEN, own_deadline).await?;
    let report = {
        let mut client = connect(server.addr, Some(LAB_TOKEN), "req-deadline").await?;

        let mut request = Request::new(capture_request());
        request.set_timeout(Duration::from_millis(300));
        let started = Instant::now();
        let result = client.post_entry(request).await;
        let elapsed = started.elapsed();
        let code = result.err().map_or(Code::Ok, |s| s.code());
        let seen_timeout = server.probe.last().and_then(|ctx| ctx.grpc_timeout);

        // Server bỏ future ngay khi hết hạn; chờ tối đa 1 giây để chắc việc thả đã chạy xong.
        for _ in 0..100 {
            if cancelled.load(Ordering::SeqCst) > 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let cancelled_count = cancelled.load(Ordering::SeqCst);
        let completed_after_deadline = completed.load(Ordering::SeqCst);

        let mut control = Request::new(capture_request());
        control.set_timeout(Duration::from_secs(5));
        let control_code = client
            .post_entry(control)
            .await
            .err()
            .map_or(Code::Ok, |s| s.code());

        DeadlineReport {
            code,
            elapsed,
            seen_timeout,
            cancelled: cancelled_count,
            completed_after_deadline,
            control_code,
        }
    };
    server.stop().await?;
    Ok(report)
}
