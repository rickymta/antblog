//! Chạy service sổ cái giả: nạp cấu hình, bật log JSON, xử lý ba request.
//!
//! - `cargo run`: cần `LEDGER__DATABASE__PASSWORD`; thiếu hoặc sai cấu hình thì thoát mã 2 ngay.
//! - `cargo run -- --verify`: tự chứa, không cần biến môi trường.

mod verify;

use std::process::ExitCode;

use ledger_29_config_tracing::scenarios::three_requests;
use ledger_29_config_tracing::service::{LedgerService, SpanPropagation};
use ledger_29_config_tracing::settings::Settings;
use ledger_29_config_tracing::telemetry;

fn main() -> ExitCode {
    if std::env::args().any(|a| a == "--verify") {
        return verify::run();
    }

    // Fail fast: cấu hình sai thì in mọi vấn đề rồi thoát, chưa mở cổng, chưa kết nối gì.
    let settings = match Settings::load() {
        Ok(settings) => settings,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    if let Err(error) = telemetry::init_stdout(&settings.log.filter) {
        eprintln!("không bật được log: {error}");
        return ExitCode::from(2);
    }
    // `?settings` dùng Debug: mật khẩu hiện là [REDACTED], không lọt vào log.
    tracing::info!(?settings, "ledger khởi động");

    let runtime = tokio::runtime::Runtime::new().expect("dựng runtime");
    runtime.block_on(async {
        let service = LedgerService::new(SpanPropagation::Attached);
        three_requests(&service).await;
    });
    ExitCode::SUCCESS
}
