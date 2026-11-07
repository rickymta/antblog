//! Lab 68: bản đồ 68 bài và kiểm toàn loạt.
//!
//! `cargo run -- ` in bản đồ; `--routes`, `--evidence` in đường đọc và sổ bằng chứng;
//! `--verify` kiểm file bài, lab, lộ trình, đường đọc và sổ bằng chứng, thoát mã 1 nếu thiếu.

use ledger_68_series_wrapup::{evidence, load, render, repo, verify};
use std::process::ExitCode;

fn run(arg: Option<&str>) -> Result<ExitCode, String> {
    let root = repo::find_root()?;
    match arg {
        None | Some("--map") => {
            let c = load(&root)?;
            print!("{}", render::map(&c));
        }
        Some("--routes") => print!("{}", render::routes(&load(&root)?)),
        Some("--evidence") => {
            print!("{}", render::evidence());
            println!(
                "\n{}",
                render::summary(&evidence::summarize(evidence::CLAIMS))
            );
        }
        Some("--verify") => {
            let report = verify(&root)?;
            for line in &report.ok {
                println!("OK   {line}");
            }
            for line in &report.problems {
                println!("THIẾU {line}");
            }
            if report.problems.is_empty() {
                println!("VERIFY OK");
            } else {
                println!("VERIFY FAIL: {} vấn đề", report.problems.len());
                return Ok(ExitCode::FAILURE);
            }
        }
        Some(other) => {
            return Err(format!(
                "cờ lạ: {other}. Dùng --map, --routes, --evidence hoặc --verify"
            ));
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(args.first().map(String::as_str)) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("LỖI: {message}");
            ExitCode::from(2)
        }
    }
}
