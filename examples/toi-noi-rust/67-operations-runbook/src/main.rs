//! `cargo run` kể lại từng ca diễn tập; `cargo run -- --verify` chạy cùng các ca, im lặng, in `VERIFY OK`.

use std::process::ExitCode;

use anyhow::Result;
use ledger_67_operations_runbook::drills::run_all;

#[tokio::main]
async fn main() -> ExitCode {
    let arg = std::env::args().nth(1);
    let verify = match arg.as_deref() {
        None => false,
        Some("--verify") => true,
        Some(other) => {
            eprintln!("LỖI: tham số lạ {other} (dùng: --verify)");
            return ExitCode::FAILURE;
        }
    };
    match run(verify).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("LỖI: {err:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run(verify: bool) -> Result<()> {
    for (title, lines) in run_all().await? {
        println!("== {title}");
        if !verify {
            for line in lines {
                println!("   {line}");
            }
        }
    }
    println!("VERIFY OK");
    Ok(())
}
