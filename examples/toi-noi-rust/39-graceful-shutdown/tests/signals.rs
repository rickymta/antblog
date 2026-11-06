//! Tín hiệu THẬT gửi vào binary: spawn `serve` làm tiến trình con rồi dùng lệnh `kill`.

use std::path::Path;
use std::time::Duration;

use anyhow::{Result, ensure};
use ledger_39_graceful_shutdown::checks::check_kill;
use ledger_39_graceful_shutdown::process::{Proc, describe};
use ledger_39_graceful_shutdown::scenarios::{in_lab, kill_restart, log_number};

const EXE: &str = env!("CARGO_BIN_EXE_ledger_39_graceful_shutdown");

/// Vị trí đầu tiên của `needle` trong log, bắt đầu tìm từ `from`.
fn find_from(logs: &[String], from: usize, needle: &str) -> Result<usize> {
    logs.iter()
        .skip(from)
        .position(|l| l.contains(needle))
        .map(|p| p + from)
        .ok_or_else(|| anyhow::anyhow!("log thiếu {needle:?}: {logs:#?}"))
}

/// Gửi `sig` cho service đang xử lý chậm; kỳ vọng thoát mã 0, đủ sáu bước theo thứ tự,
/// offset đã commit bằng đúng số bút toán đã ghi.
async fn exits_cleanly_on(sig: &str, name: &str) -> Result<()> {
    in_lab(async |lab| {
        lab.produce_trips(1, 12).await?;
        let mut cfg = lab.config();
        cfg.work_delay = Duration::from_millis(40);
        let mut child = Proc::spawn(Path::new(EXE), &cfg).await?;
        lab.wait_entries(3, Duration::from_secs(30)).await?;

        child.signal(sig).await?;
        let status = child.wait(Duration::from_secs(60)).await?;
        ensure!(
            status.code() == Some(0),
            "phải thoát mã 0, thấy: {}",
            describe(&status)
        );

        let logs = child.logs();
        let mut at = find_from(&logs, 0, &format!("nhận {name}"))?;
        for step in [
            "phase=1",
            "phase=2",
            "phase=3 drained",
            "phase=4",
            "phase=5",
            "phase=6",
        ] {
            at = find_from(&logs, at, step)?;
        }
        ensure!(
            log_number(&logs, "exit_code") == Some(0),
            "summary phải báo exit_code=0"
        );

        let counts = lab.counts().await?;
        ensure!(counts.entries >= 3, "đã ghi bút toán trước khi tắt");
        ensure!(
            lab.committed_offset().await? == counts.entries,
            "offset đã commit phải bằng số bút toán đã ghi"
        );
        ensure!(
            counts.postings_sum == 0 && counts.outbox_pending == 0,
            "sổ cái nguyên vẹn, outbox đã dọn"
        );
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn sigterm_exits_zero_after_drain_and_final_commit() -> Result<()> {
    exits_cleanly_on("TERM", "SIGTERM").await
}

#[tokio::test(flavor = "multi_thread")]
async fn ctrl_c_sigint_exits_zero_the_same_way() -> Result<()> {
    exits_cleanly_on("INT", "SIGINT").await
}

#[tokio::test(flavor = "multi_thread")]
async fn drain_timeout_exits_with_code_3_and_keeps_the_ledger_intact() -> Result<()> {
    in_lab(async |lab| {
        lab.produce_trips(1, 4).await?;
        let mut cfg = lab.config();
        cfg.work_delay = Duration::from_millis(1500);
        cfg.drain_timeout = Duration::from_millis(400);
        let mut child = Proc::spawn(Path::new(EXE), &cfg).await?;
        lab.wait_entries(1, Duration::from_secs(30)).await?;

        child.signal("TERM").await?;
        let status = child.wait(Duration::from_secs(60)).await?;
        ensure!(
            status.code() == Some(3),
            "drain quá hạn phải thoát mã 3, thấy: {}",
            describe(&status)
        );
        ensure!(
            child
                .logs()
                .iter()
                .any(|l| l.contains("phase=3 drain=timeout")),
            "log phải ghi drain quá hạn"
        );
        let counts = lab.counts().await?;
        ensure!(counts.postings_sum == 0, "tổng posting vẫn bằng 0");
        ensure!(
            counts.entries == counts.inbox && counts.entries == counts.outbox_total,
            "không có dòng nửa vời sau khi bị cắt giữa transaction"
        );
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn second_signal_forces_exit_code_4() -> Result<()> {
    in_lab(async |lab| {
        lab.produce_trips(1, 4).await?;
        let mut cfg = lab.config();
        cfg.work_delay = Duration::from_millis(2000);
        cfg.drain_timeout = Duration::from_secs(30);
        let mut child = Proc::spawn(Path::new(EXE), &cfg).await?;
        lab.wait_entries(1, Duration::from_secs(30)).await?;

        child.signal("TERM").await?;
        tokio::time::sleep(Duration::from_millis(300)).await;
        child.signal("TERM").await?;
        let status = child.wait(Duration::from_secs(5)).await?;
        ensure!(
            status.code() == Some(4),
            "tín hiệu thứ hai phải thoát mã 4, thấy: {}",
            describe(&status)
        );
        let counts = lab.counts().await?;
        ensure!(
            counts.postings_sum == 0 && counts.entries == counts.inbox,
            "sổ cái vẫn nguyên vẹn"
        );
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn sigkill_loses_no_entry_thanks_to_transaction_and_idempotency() -> Result<()> {
    let report = kill_restart(Path::new(EXE)).await?;
    check_kill(&report, 24)
}
