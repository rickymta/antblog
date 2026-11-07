//! Chạy server và tắt có trật tự khi nhận SIGTERM hoặc SIGINT.

use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::signal::unix::{SignalKind, signal};
use tracing::info;

use crate::config::Config;
use crate::http::{AppState, router};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Cài log JSON ra stdout (container: log ra stdout/stderr, không ghi file).
///
/// # Errors
/// Đã có subscriber toàn cục.
pub fn init_logging() -> Result<(), String> {
    tracing_subscriber::fmt()
        .json()
        .with_current_span(false)
        .with_span_list(false)
        .try_init()
        .map_err(|e| e.to_string())
}

/// UID của tiến trình, đọc từ `/proc/self/status` (chỉ có trên Linux; nơi khác trả `None`).
#[must_use]
pub fn effective_uid() -> Option<u32> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|l| l.starts_with("Uid:"))?;
    line.split_whitespace().nth(1)?.parse().ok()
}

/// Chạy service theo cấu hình tới khi nhận tín hiệu tắt.
///
/// # Errors
/// Không bind được địa chỉ, chuỗi kết nối DB sai hoặc không đăng ký được tín hiệu.
pub async fn serve(cfg: Config) -> Result<(), BoxError> {
    // Đăng ký tín hiệu TRƯỚC khi báo "listening": tín hiệu tới sớm hơn sẽ không bị mất.
    let mut term = signal(SignalKind::terminate())?;
    let mut int = signal(SignalKind::interrupt())?;
    let state = match &cfg.pg_url {
        Some(url) => AppState::with_db(url)?,
        None => AppState::without_db(),
    };
    let listener = TcpListener::bind(cfg.addr).await?;
    let local: SocketAddr = listener.local_addr()?;
    // PID 1 + uid giúp đọc từ `docker logs` mà không cần shell trong image.
    info!(
        addr = %local,
        pid = std::process::id(),
        uid = effective_uid(),
        database = cfg.pg_url.is_some(),
        tag = crate::BUILD_TAG,
        "listening"
    );
    let drain = state.clone();
    axum::serve(listener, router(state))
        .with_graceful_shutdown(async move {
            tokio::select! {
                _ = term.recv() => info!("nhận SIGTERM"),
                _ = int.recv() => info!("nhận SIGINT"),
            }
            drain.begin_drain();
        })
        .await?;
    info!("shutdown complete");
    Ok(())
}
