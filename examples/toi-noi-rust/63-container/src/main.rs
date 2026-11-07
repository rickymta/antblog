//! `ledger-service`: `serve` chạy service, `healthcheck` tự hỏi `/healthz` rồi thoát 0/1.
//! Build mặc định (feature `lab`) có thêm `--verify` và `--with-docker`; image không mang mã này.

use std::process::ExitCode;

use ledger_container::config::Config;
use ledger_container::{probe, server};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("serve") => serve(),
        Some("healthcheck") => healthcheck(),
        #[cfg(feature = "lab")]
        None => ledger_container::lab::run(ledger_container::lab::Mode::Demo, false),
        #[cfg(feature = "lab")]
        Some("--verify") => ledger_container::lab::run(
            ledger_container::lab::Mode::Verify,
            args.iter().any(|a| a == "--with-docker"),
        ),
        #[cfg(feature = "lab")]
        Some("--with-docker") => {
            ledger_container::lab::run(ledger_container::lab::Mode::Verify, true)
        }
        other => Err(format!(
            "tham số lạ: {other:?} (dùng: serve | healthcheck{})",
            if cfg!(feature = "lab") {
                " | --verify [--with-docker]"
            } else {
                ""
            }
        )),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("LỖI: {err}");
            ExitCode::FAILURE
        }
    }
}

fn serve() -> Result<(), String> {
    server::init_logging()?;
    let cfg = Config::from_env()?;
    tokio::runtime::Runtime::new()
        .map_err(|e| e.to_string())?
        .block_on(server::serve(cfg))
        .map_err(|e| e.to_string())
}

// Không dựng tokio runtime: probe chạy mỗi vài giây trong container, phải rẻ.
fn healthcheck() -> Result<(), String> {
    let cfg = Config::from_env()?;
    probe::healthcheck(probe::loopback_of(cfg.addr))
}
