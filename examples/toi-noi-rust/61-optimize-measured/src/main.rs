//! Lab bài 61: chạy thí nghiệm. Không tham số: phần không cần database. `--verify`: kiểm toàn bộ.
//! `--wire`, `--prepared`, `--pg-bench`, `--latency`, `--breakdown`: các số đo cần PostgreSQL.

mod report;
mod verify;

use std::process::ExitCode;

use optimize_measured::db::BoxError;

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .and_then(|v| v.parse::<u64>().ok())
    };
    let (n_arg, rate_arg) = (flag("--n"), flag("--rate"));
    let result: Result<(), BoxError> = match args.first().map(String::as_str) {
        None => {
            report::demo();
            Ok(())
        }
        Some("--verify") => verify::run().await,
        Some("--wire") => report::wire().await,
        Some("--prepared") => report::prepared().await,
        Some("--pg-bench") => report::pg_bench(n_arg.unwrap_or(10_000)).await,
        Some("--latency") => report::latency(rate_arg.unwrap_or(20)).await,
        Some("--breakdown") => report::breakdown(n_arg.unwrap_or(200)).await,
        Some(other) => Err(format!("tham số lạ: {other}").into()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("LỖI: {e}");
            ExitCode::FAILURE
        }
    }
}
