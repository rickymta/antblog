//! Chạy các thí nghiệm của bài 42 và in kết quả. `--verify` thêm dòng `VERIFY OK` khi mọi kiểm tra qua.
//! Các kiểm tra (`ensure!`) luôn chạy, kể cả khi không có `--verify`: lỗi thì thoát mã 1.

use std::process::ExitCode;

use anyhow::Result;
use ledger_42_hold_capture_void::lab::with_lab;
use ledger_42_hold_capture_void::scenarios::{
    capture_full, capture_partial, capture_vs_void, combined_statement, expiry_skip_locked,
    hold_does_not_post, late_capture, lock_wait, race_report, void_on_cancel,
    wallet_without_balance_row,
};
use ledger_42_hold_capture_void::store::LockMode;

async fn run(verify: bool) -> Result<()> {
    with_lab(async move |lab| hold_does_not_post(&lab).await).await?;
    with_lab(async move |lab| capture_full(&lab).await).await?;
    with_lab(async move |lab| capture_partial(&lab).await).await?;
    with_lab(async move |lab| void_on_cancel(&lab).await).await?;

    println!("== 5. Hai hold 25.500 cùng lúc trên ví 40.000 ==");
    race_report(LockMode::None).await?;
    race_report(LockMode::ForUpdate).await?;
    with_lab(async move |lab| lock_wait(&lab).await).await?;
    with_lab(async move |lab| combined_statement(&lab).await).await?;
    with_lab(async move |lab| wallet_without_balance_row(&lab).await).await?;

    with_lab(async move |lab| capture_vs_void(&lab).await).await?;
    with_lab(async move |lab| expiry_skip_locked(&lab).await).await?;
    with_lab(async move |lab| late_capture(&lab).await).await?;

    if verify {
        println!("VERIFY OK");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    let verify = std::env::args().any(|arg| arg == "--verify");
    match run(verify).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("LỖI: {err:#}");
            ExitCode::FAILURE
        }
    }
}
