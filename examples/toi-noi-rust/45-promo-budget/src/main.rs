//! CLI của lab 45: chạy mọi kịch bản trong một schema `lab_<uuid>` rồi dọn.
//! `--verify` kiểm các giá trị bài viết nêu và in `VERIFY OK`.

use std::process::ExitCode;

use anyhow::{Result, ensure};
use ledger_promo::db::{TestDb, lab_url};
use ledger_promo::scenarios::run_all;

#[tokio::main]
async fn main() -> ExitCode {
    let verify = match std::env::args().nth(1).as_deref() {
        None => false,
        Some("--verify") => true,
        Some(other) => {
            eprintln!("LỖI: tham số lạ: {other}");
            return ExitCode::FAILURE;
        }
    };
    match run(verify).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("LỖI: {e:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run(verify: bool) -> Result<()> {
    let url = lab_url()?;
    // Pool 8 kết nối: cuộc đua dùng Barrier cần đủ kết nối cho 5 tác vụ cùng chờ.
    let db = TestDb::try_new(8).await?;
    let schema = db.schema.clone();
    let report = run_all(&db).await;
    db.close().await; // luôn dọn, kể cả khi kịch bản lỗi
    let report = report?;

    report.print();
    let gone = !TestDb::schema_exists(&url, &schema).await?;
    println!();
    println!("sau close(): schema lab_<uuid> còn tồn tại = {}", !gone);
    ensure!(gone, "schema {schema} chưa được dọn");
    if verify {
        report.verify()?;
        println!("VERIFY OK");
    }
    Ok(())
}
