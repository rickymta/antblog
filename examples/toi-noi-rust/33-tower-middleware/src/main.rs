//! `cargo run` in kết quả các kiểm chứng; `cargo run -- --verify` thoát mã 1 nếu có kiểm chứng sai.

use ledger_33_tower_middleware::verify;

// `current_thread`: mọi task chung một thread nên bộ đệm theo thread của `capture()` nhận log của cả server lẫn handler.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let verify_mode = std::env::args().any(|a| a == "--verify");
    let failed = verify::run_all().await;
    if failed > 0 {
        eprintln!("{failed} kiểm chứng sai");
        std::process::exit(1);
    }
    if verify_mode {
        println!("VERIFY OK");
    }
}
