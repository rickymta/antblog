//! Lab bài 47. Cách dùng:
//!   ledger_47_psp_reconciliation [--verify]
//!   ledger_47_psp_reconciliation gen <đường dẫn csv> <số ref>
//!   ledger_47_psp_reconciliation bench <sql|rust> <đường dẫn csv> <số ref>

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use ledger_47_psp_reconciliation::bench::{
    Mode, expected_counts, run_bench, write_generated_statement,
};
use ledger_47_psp_reconciliation::demo;

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match dispatch(&args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("LỖI: {e:#}");
            ExitCode::FAILURE
        }
    }
}

async fn dispatch(args: &[String]) -> Result<()> {
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => demo::run(false).await,
        ["--verify"] => demo::run(true).await,
        ["gen", path, rows] => {
            let rows: i64 = rows.parse().context("số ref phải là số nguyên")?;
            let period = demo::sample_period()?;
            let g = write_generated_statement(&PathBuf::from(path), rows, &period)?;
            println!("đã ghi {path}: {} dòng dữ liệu, {} byte", g.lines, g.bytes);
            println!("kỳ vọng: {:?}", expected_counts(rows));
            Ok(())
        }
        ["bench", mode, path, rows] => {
            let mode = match *mode {
                "sql" => Mode::Sql,
                "rust" => Mode::Rust,
                other => bail!("chế độ lạ {other:?}, dùng sql hoặc rust"),
            };
            run_bench(
                mode,
                PathBuf::from(path),
                rows.parse().context("số ref phải là số nguyên")?,
            )
            .await
        }
        _ => bail!("cách dùng: [--verify] | gen <csv> <số ref> | bench <sql|rust> <csv> <số ref>"),
    }
}
