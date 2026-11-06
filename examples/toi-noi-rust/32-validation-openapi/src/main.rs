//! Chạy kịch bản của bài 32. `-- --verify` kiểm các giá trị kỳ vọng, in `VERIFY OK` hoặc thoát mã 1.

use std::process::ExitCode;

use ledger_32_validation_openapi::verify;

#[tokio::main]
async fn main() -> ExitCode {
    let verify_mode = std::env::args().any(|a| a == "--verify");
    let checks = match verify::run().await {
        Ok(checks) => checks,
        Err(err) => {
            eprintln!("LỖI: {err}");
            return ExitCode::FAILURE;
        }
    };
    if !verify_mode {
        return ExitCode::SUCCESS;
    }
    if checks.failures().is_empty() {
        println!("\nVERIFY OK ({} điều kiện)", checks.total());
        ExitCode::SUCCESS
    } else {
        for failure in checks.failures() {
            eprintln!("SAI: {failure}");
        }
        eprintln!("VERIFY FAIL");
        ExitCode::FAILURE
    }
}
