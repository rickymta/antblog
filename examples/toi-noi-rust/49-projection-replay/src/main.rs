//! Chạy các thí nghiệm của bài 49 và in kết quả. `--verify` kiểm thêm các giá trị bài viết nêu.

use std::process::ExitCode;

use anyhow::Result;
use ledger_49_projection_replay::lab::with_lab;
use ledger_49_projection_replay::log::ReadMode;
use ledger_49_projection_replay::scenarios::{
    IncomeRow, RaceVariant, basic, lag, race, rebuild, replay,
};
use ledger_core::Money;

fn money(dong: i64) -> String {
    Money::from_dong(dong).to_string()
}

fn print_income(label: &str, rows: &[IncomeRow]) {
    println!("{label}");
    for (day, account, dong) in rows {
        println!("  {day} {account:<17} {:>12}", money(*dong));
    }
}

fn mode_name(mode: ReadMode) -> &'static str {
    match mode {
        ReadMode::NaiveId => "id > checkpoint",
        ReadMode::HorizonId => "xmin + con trỏ id",
        ReadMode::Safe => "xmin + (tx_id, id)",
    }
}

async fn run(verify: bool) -> Result<()> {
    // 1. Hai projection từ cùng một log.
    let r = with_lab(async |lab| basic(&lab).await).await?;
    println!("== 1. Hai projection từ cùng một log ==");
    println!(
        "log: {} bút toán, lag trước khi chạy: {}",
        r.entry_ids.len(),
        r.behind_before
    );
    println!(
        "lô số dư (limit 3): {:?}, checkpoint: entry {}",
        r.batches, r.checkpoint_entry
    );
    println!("số dư:");
    for (account, dong) in &r.balances {
        println!("  {account:<20} {:>12}", money(*dong));
    }
    println!(
        "bằng số dư tính thẳng từ postings: {}",
        r.balances == r.truth
    );
    print_income("thu nhập theo ngày, phiên bản 1 (ngày UTC):", &r.income);
    println!(
        "chạy lại: áp dụng {} sự kiện, bảng giữ nguyên: {}",
        r.rerun_applied, r.rerun_unchanged
    );
    if verify {
        r.verify()?;
    }

    // 2. Chạy lại không cộng trùng.
    let r = replay().await?;
    println!("== 2. Chạy lại không cộng trùng ==");
    println!(
        "chết trước COMMIT : lô {} bút toán bị rollback, checkpoint = {}, bảng có {} dòng",
        r.crashed_batch, r.checkpoint_after_crash, r.rows_after_crash
    );
    println!(
        "chạy lại          : bằng số dư chuẩn: {}",
        r.correct_equals_truth
    );
    println!("tách hai transaction (SAI), sau khi chạy lại:");
    for (account, truth, got) in &r.wrong_diffs {
        println!(
            "  {account:<20} chuẩn {:>12}, projection {:>12}",
            money(*truth),
            money(*got)
        );
    }
    println!(
        "4 worker cùng chạy: {} sự kiện áp dụng, bằng chuẩn: {}",
        r.workers_applied, r.workers_equal_truth
    );
    if verify {
        r.verify()?;
    }

    // 3. Giao dịch commit muộn.
    println!("== 3. Giao dịch commit muộn ==");
    for variant in [RaceVariant::LateCommit, RaceVariant::EarlyXid] {
        println!(
            "kịch bản {}:",
            match variant {
                RaceVariant::LateCommit => "A (INSERT trước, commit sau)",
                RaceVariant::EarlyXid => "B (xid cấp sớm, INSERT sau)",
            }
        );
        for mode in [ReadMode::NaiveId, ReadMode::HorizonId, ReadMode::Safe] {
            let r = with_lab(async move |lab| race(&lab, variant, mode).await).await?;
            let loss = match r.diffs.iter().find(|(a, _, _)| a == "driver:DR-001") {
                Some((_, truth, got)) => format!(
                    "MẤT DÒNG, DR-001 phải có {}, projection có {}",
                    money(*truth),
                    money(*got)
                ),
                None => "đủ, bằng số dư chuẩn".to_string(),
            };
            println!(
                "  {:<19} khi giao dịch chậm còn mở: áp dụng {}, lag {}; cuối: {}",
                mode_name(mode),
                r.applied_while_open,
                r.behind_while_open,
                loss
            );
            if verify {
                r.verify()?;
            }
        }
    }

    // 4. Dựng lại, đổi tên nguyên tử, đổi phiên bản.
    let r = with_lab(async |lab| rebuild(&lab).await).await?;
    println!("== 4. Dựng lại và đổi phiên bản ==");
    println!(
        "số dư v2 (cùng logic): dựng dở {} bút toán, rồi bắt kịp {} trong transaction chuyển, lệch {} dòng",
        r.partial, r.balances_caught_up, r.balances_diff
    );
    println!(
        "thu nhập v1 -> v2, đòi giống hệt: bị chặn, lệch {} dòng (rollback, chưa đổi gì)",
        r.income_blocked_diff.unwrap_or_default()
    );
    println!(
        "thu nhập v1 -> v2, chấp nhận khác: bắt kịp {}, lệch {} dòng",
        r.income_caught_up, r.income_diff
    );
    print_income(
        "thu nhập trước (v1, ngày UTC, có chuyến TN-0004):",
        &r.income_before,
    );
    print_income("thu nhập sau (v2, ngày giờ Việt Nam):", &r.income_after);
    println!("bảng cũ income_by_day_v1 còn nguyên: {}", r.old_income_kept);
    for (name, version, active) in &r.checkpoints {
        println!("  checkpoint {name:<14} v{version} active = {active}");
    }
    println!(
        "bản v1 chạy lô sau khi bị thay: bị từ chối: {}",
        r.old_version_refused
    );
    if verify {
        r.verify()?;
    }

    // 5. Lag và tốc độ dựng lại.
    let r = with_lab(async |lab| lag(&lab, 2_500, 500).await).await?;
    println!("== 5. Lag và tốc độ dựng lại ==");
    println!("{} bút toán trong log, lô {} bút toán", r.entries, r.batch);
    println!("lag sau từng lô: {:?}", r.behind);
    println!(
        "dựng lại toàn bộ vào bảng mới: {} ms (≈ {} bút toán/giây), lệch {} dòng, {} tài khoản",
        r.rebuild_elapsed.as_millis(),
        r.entries_per_second(),
        r.rebuild_diff,
        r.balance_rows
    );
    println!("(số đo trên máy viết bài, không phải SLA)");
    if verify {
        r.verify()?;
    }

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
