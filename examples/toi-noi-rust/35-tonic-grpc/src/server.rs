//! Khởi động server tonic trên `127.0.0.1:0` và đọc cổng thật.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

use crate::interceptor::auth_interceptor;
use crate::pb::ledger_server::LedgerServer;
use crate::service::{LedgerService, Probe};
use crate::store::LedgerStore;

/// Server đang chạy nền. Gọi [`RunningServer::stop`] để tắt êm.
pub struct RunningServer {
    /// Địa chỉ thật (cổng do hệ điều hành cấp).
    pub addr: SocketAddr,
    /// Thông tin request gần nhất mà handler thấy.
    pub probe: Arc<Probe>,
    shutdown: oneshot::Sender<()>,
    task: JoinHandle<Result<(), tonic::transport::Error>>,
}

/// Bind `127.0.0.1:0`, chạy service với interceptor kiểm token.
///
/// # Errors
/// Không bind được cổng.
pub async fn start<S: LedgerStore>(store: S, token: &str) -> Result<RunningServer> {
    start_with(store, token, true).await
}

/// Như [`start`], chọn có cho handler tự đếm deadline hay chỉ dựa vào timeout của tonic.
///
/// # Errors
/// Không bind được cổng.
pub async fn start_with<S: LedgerStore>(
    store: S,
    token: &str,
    own_deadline: bool,
) -> Result<RunningServer> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("bind 127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let probe = Arc::new(Probe::default());
    let mut ledger = LedgerService::new(store, Arc::clone(&probe));
    if !own_deadline {
        ledger = ledger.without_own_deadline();
    }
    let service = LedgerServer::with_interceptor(ledger, auth_interceptor(token));
    let (shutdown, rx) = oneshot::channel::<()>();
    let task = tokio::spawn(
        Server::builder()
            .add_service(service)
            .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                // Gửi tín hiệu hoặc thả sender đều làm server dừng nhận kết nối mới.
                let _ = rx.await;
            }),
    );
    Ok(RunningServer {
        addr,
        probe,
        shutdown,
        task,
    })
}

impl RunningServer {
    /// Dừng nhận kết nối mới, chờ server thoát.
    ///
    /// # Errors
    /// Lỗi transport khi server thoát.
    pub async fn stop(self) -> Result<()> {
        let _ = self.shutdown.send(());
        self.task.await.context("task server")??;
        Ok(())
    }
}
