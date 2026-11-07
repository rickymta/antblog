//! `trip-payment`: không tham số chạy các kịch bản end-to-end và in tường thuật; `--verify` chạy
//! chúng và in `VERIFY OK`; `serve` chạy consumer + relay thật (tiến trình con của ca «chết giữa chừng»).

use std::process::ExitCode;

use anyhow::{Context, Result, anyhow};
use api::scenarios::{self, Report};
use infrastructure::Fault;
use infrastructure::service::{Service, ServiceConfig};
use tokio::signal::unix::{SignalKind, signal};

#[tokio::main]
async fn main() -> ExitCode {
    let arg = std::env::args().nth(1);
    let result = match arg.as_deref() {
        Some("serve") => serve().await,
        Some("--verify") => run_scenarios(true).await,
        None => run_scenarios(false).await,
        Some(other) => Err(anyhow!("tham số lạ: {other} (dùng: serve | --verify)")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("LỖI: {err:#}");
            ExitCode::FAILURE
        }
    }
}

/// Chạy service cho tới khi nhận SIGTERM hoặc Ctrl-C, rồi tắt có thứ tự.
async fn serve() -> Result<()> {
    // Đăng ký tín hiệu TRƯỚC khi báo sẵn sàng (bài 39).
    let mut term = signal(SignalKind::terminate()).context("đăng ký SIGTERM")?;
    let mut int = signal(SignalKind::interrupt()).context("đăng ký SIGINT")?;
    let cfg = ServiceConfig::from_env()?;
    let service = Service::start(&cfg).await?;
    println!("serve: sẵn sàng (fault = {})", cfg.fault.as_str());
    tokio::select! {
        _ = term.recv() => println!("serve: nhận SIGTERM"),
        _ = int.recv() => println!("serve: nhận SIGINT"),
    }
    service.stop().await
}

async fn run_scenarios(verify: bool) -> Result<()> {
    let exe = std::env::current_exe()?;
    let runs: Vec<(&str, Result<Report>)> = vec![
        (
            "Hai thứ tự đến, cùng một kết quả",
            scenarios::both_orders().await,
        ),
        (
            "Hai sự kiện cùng đến, một bên thua version",
            scenarios::version_race().await,
        ),
        (
            "Khuyến mãi đến muộn, ngân sách và số thu lệch",
            scenarios::promo_and_mismatch().await,
        ),
        (
            "Sổ đã có bút toán ba dòng của bài 40",
            scenarios::legacy_three_line().await,
        ),
        ("Hoàn một phần", scenarios::partial_refund().await),
        (
            "Sự kiện trùng và sự kiện đổi nội dung",
            scenarios::duplicates().await,
        ),
        (
            "Chết trước COMMIT (tiến trình thật)",
            scenarios::crash(Fault::AbortBeforeCommit, &exe).await,
        ),
        (
            "Chết sau COMMIT, trước commit offset (tiến trình thật)",
            scenarios::crash(Fault::AbortAfterCommit, &exe).await,
        ),
        (
            "Đối soát theo dòng clearing",
            scenarios::reconcile_clearing().await,
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
