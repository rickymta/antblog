//! Điểm vào: `api` (chạy thử bộ nhớ) hoặc `api -- --verify` (bộ nhớ, PostgreSQL, Kafka thật).

use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let verify = std::env::args().any(|a| a == "--verify");
    let result = if verify {
        run_verify().await
    } else {
        api::scenario::memory().await.map(|_| ())
    };
    match result {
        Ok(()) if verify => {
            println!("VERIFY OK");
            ExitCode::SUCCESS
        }
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("VERIFY FAIL: {e:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run_verify() -> anyhow::Result<()> {
    let events = api::scenario::memory().await?;
    api::scenario::postgres(&events).await?;
    api::scenario::kafka(&events).await
}
