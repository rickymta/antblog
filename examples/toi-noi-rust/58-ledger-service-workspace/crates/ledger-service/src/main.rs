//! `ledger-service`: `serve` chạy service thật; không tham số chạy kiểm kiến trúc rồi các kịch bản
//! end-to-end; `--verify` làm như vậy nhưng im lặng và kết thúc bằng `VERIFY OK`.

use std::process::ExitCode;
use std::sync::Arc;

use anyhow::{Context, Result};
use ledger_domain::clock::SystemClock;
use ledger_service::app::App;
use ledger_service::config::Config;
use ledger_service::{arch, scenario, telemetry};
use tokio::signal::unix::{SignalKind, signal};
use tracing::info;

#[tokio::main]
async fn main() -> ExitCode {
    let arg = std::env::args().nth(1);
    let result = match arg.as_deref() {
        Some("serve") => serve().await,
        Some("--verify") => run_checks(true).await,
        None => run_checks(false).await,
        Some(other) => Err(anyhow::anyhow!(
            "tham số lạ: {other} (dùng: serve | --verify)"
        )),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("LỖI: {err:#}");
            ExitCode::FAILURE
        }
    }
}

/// Chạy service cho tới khi nhận SIGTERM hoặc Ctrl-C, rồi tắt có trật tự.
async fn serve() -> Result<()> {
    telemetry::init(
        &std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        false,
    )
    .map_err(anyhow::Error::msg)?;
    // Đăng ký tín hiệu TRƯỚC khi báo "listening" (bài 39): `ctrl_c()` chỉ cài handler khi được poll.
    let mut term = signal(SignalKind::terminate()).context("đăng ký SIGTERM")?;
    let mut int = signal(SignalKind::interrupt()).context("đăng ký SIGINT")?;
    let app = App::start(Config::from_env()?, Arc::new(SystemClock)).await?;
    info!(http_addr = %app.http_addr, grpc_addr = %app.grpc_addr, "listening");
    tokio::select! {
        _ = term.recv() => info!("nhận SIGTERM"),
        _ = int.recv() => info!("nhận SIGINT"),
    }
    let report = app.shutdown().await;
    anyhow::ensure!(!report.timed_out, "tắt quá hạn, một số task bị hủy cứng");
    Ok(())
}

async fn run_checks(verify: bool) -> Result<()> {
    // Chế độ demo: log của service ra stderr, mặc định chỉ lỗi, để stdout chỉ có kết quả.
    telemetry::init(
        &std::env::var("RUST_LOG").unwrap_or_else(|_| "error".into()),
        true,
    )
    .map_err(anyhow::Error::msg)?;

    println!("== Chiều phụ thuộc (cargo tree)");
    for line in arch::check_all().context("kiểm chiều phụ thuộc")? {
        if !verify {
            println!("   {line}");
        }
    }

    let exe = std::env::current_exe()?;
    let runs = [
        (
            "TripCompleted -> hai bút toán -> số dư, phát lại, tắt",
            scenario::flow_and_replay().await,
        ),
        (
            "POST /v1/entries và gRPC PostEntry với JWT",
            scenario::http_and_grpc_contract().await,
        ),
        (
            "Hold, capture, hết hạn theo đồng hồ giả, hoàn tiền",
            scenario::hold_and_refund().await,
        ),
        (
            "Trigger cân bút toán, append-only, CHECK",
            scenario::database_guards().await,
        ),
        (
            "Thứ tự khóa account_balances",
            scenario::balance_lock_order().await,
        ),
        (
            "SIGTERM tiến trình thật",
            scenario::sigterm_child(&exe).await,
        ),
    ];
    for (name, outcome) in runs {
        let report = outcome.with_context(|| format!("kịch bản «{name}» thất bại"))?;
        println!("== {name}");
        if !verify {
            for line in report.0 {
                println!("   {line}");
            }
        }
    }
    println!("VERIFY OK");
    Ok(())
}
