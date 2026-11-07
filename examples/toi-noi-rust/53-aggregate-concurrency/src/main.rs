//! Chạy các thí nghiệm của bài 53 và in kết quả. `--verify` thêm dòng `VERIFY OK` khi mọi kiểm tra qua;
//! `--measure` in số đo trên máy chạy (không nằm trong `--verify` vì giờ và nhịp retry thay đổi mỗi lần).
//! Các kiểm tra (`ensure!`) luôn chạy: lỗi thì thoát mã 1.

use std::process::ExitCode;

use anyhow::{Result, ensure};
use ledger_53_aggregate_concurrency::lab::with_lab;
use ledger_53_aggregate_concurrency::scenarios::{
    deadlock, deadlock_victim, dong, join_lock_trap, journal_is_append_only, lock_variants,
    measure, mode_name, ordered_stress, race, stale_write, style_name,
};
use ledger_53_aggregate_concurrency::store::{LockOrder, LockedLoad, Mode};

async fn run_stale() -> Result<()> {
    println!("== 1. Hai writer xen kẽ: A nạp bản cũ, B nạp thêm 50.000 và commit, rồi A ghi ==");
    for mode in [Mode::Unguarded, Mode::Optimistic] {
        let r = with_lab(async move |lab| stale_write(&lab, mode).await).await?;
        println!(
            "{:<11}: {} lần thử, rút {}; số dư lưu {}, số dư theo posting {}; tổng số dư mọi tài khoản {}, lệch projection {:?}",
            mode_name(mode),
            r.attempts,
            if r.withdrew { "được" } else { "không" },
            r.stored_balance,
            r.ledger_balance,
            dong(r.audit.total_balance),
            r.audit.projection_mismatch,
        );
        match mode {
            Mode::Unguarded => {
                ensure!(r.attempts == 1 && r.withdrew);
                ensure!(r.stored_balance == dong(129_600) && r.ledger_balance == dong(179_600));
                ensure!(r.audit.total_balance == -50_000);
                ensure!(r.audit.projection_mismatch == ["driver:DR-001"]);
            }
            _ => {
                ensure!(r.attempts == 2 && r.withdrew);
                ensure!(r.stored_balance == dong(179_600) && r.ledger_balance == dong(179_600));
                ensure!(r.audit.total_balance == 0 && r.audit.projection_mismatch.is_empty());
            }
        }
    }
    Ok(())
}

async fn run_race() -> Result<()> {
    println!("== 2. 8 task cùng rút 20.400 từ ví 150.000, hạn mức 100.000 ==");
    for mode in [Mode::Unguarded, Mode::Optimistic, Mode::ForUpdate] {
        let r = with_lab(async move |lab| race(&lab, mode, 8).await).await?;
        println!("{:<11}: {}", mode_name(mode), r.line());
        println!("{:<11}  {}", "", r.audit_line());
        ensure!(r.exhausted == 0);
        match mode {
            Mode::Unguarded => {
                ensure!(r.ok == 8 && r.rejected == 0);
                ensure!(r.attempts == [1; 8]);
                ensure!(r.audit.over_limit == [("driver:DR-001".to_string(), 163_200, 100_000)]);
                ensure!(r.audit.projection_mismatch == ["driver:DR-001"]);
                ensure!(r.audit.total_balance != 0);
            }
            Mode::Optimistic | Mode::ForUpdate => {
                ensure!(r.ok == 4 && r.rejected == 4);
                ensure!(r.balance == dong(68_400) && r.withdrawn == dong(81_600));
                ensure!(r.audit.over_limit.is_empty() && r.audit.projection_mismatch.is_empty());
                ensure!(r.audit.total_balance == 0 && r.audit.total_postings == 0);
                if mode == Mode::Optimistic {
                    // Barrier ép cả 8 nạp version cũ: đúng một task qua ngay lần đầu.
                    ensure!(r.attempts.iter().filter(|a| **a == 1).count() == 1);
                    // Mỗi lần xung đột nghĩa là có người commit; chỉ có 4 commit nên tối đa 5 lần thử.
                    ensure!(r.attempts.iter().all(|a| *a <= 5));
                    ensure!(r.version == 5);
                } else {
                    ensure!(r.attempts == [1; 8]);
                    ensure!(r.version == 5);
                }
            }
        }
    }
    Ok(())
}

async fn run_join_trap() -> Result<()> {
    println!("== 2b. Khóa ví rồi đọc hạn mức: một câu JOIN hay hai câu? ==");
    println!("đúng là: số dư 129.600 VND, đã rút 20.400 VND (A vừa commit)");
    for style in [
        LockedLoad::JoinLockOne,
        LockedLoad::JoinLockBoth,
        LockedLoad::Separate,
    ] {
        let t = with_lab(async move |lab| join_lock_trap(&lab, style).await).await?;
        println!(
            "{:<26}: người chờ thấy số dư {}, đã rút {}",
            style_name(style),
            t.balance,
            t.withdrawn
        );
        ensure!(t.balance == dong(129_600), "số dư luôn là bản mới");
        let expected = if style == LockedLoad::JoinLockOne {
            dong(0)
        } else {
            dong(20_400)
        };
        ensure!(t.withdrawn == expected, "{style:?}: đã rút {}", t.withdrawn);
    }
    Ok(())
}

async fn run_deadlock() -> Result<()> {
    println!("== 3. Chuyển ngược chiều giữa hai ví ==");
    let r = with_lab(async move |lab| deadlock(&lab, LockOrder::AsGiven, false).await).await?;
    println!(
        "khóa theo thứ tự nhận được, không retry: {} ok, {} deadlock 40P01; số dư {} / {}",
        r.ok, r.deadlocks, r.balances.0, r.balances.1
    );
    ensure!(r.ok == 1 && r.deadlocks == 1);
    ensure!(r.audit.total_balance == 0 && r.audit.projection_mismatch.is_empty());

    let r = with_lab(async move |lab| deadlock(&lab, LockOrder::AsGiven, true).await).await?;
    println!(
        "khóa theo thứ tự nhận được, retry 40P01: {} ok, {} deadlock; số lần thử {:?}; số dư {} / {}",
        r.ok, r.deadlocks, r.attempts, r.balances.0, r.balances.1
    );
    ensure!(r.ok == 2 && r.deadlocks == 0);
    ensure!(r.attempts.iter().sum::<u32>() >= 3, "nạn nhân phải thử lại");
    ensure!(r.balances == (dong(48_000), dong(52_000)));

    let r = with_lab(async move |lab| deadlock(&lab, LockOrder::ById, false).await).await?;
    println!(
        "khóa theo id, không retry:                {} ok, {} deadlock; số lần thử {:?}; số dư {} / {}",
        r.ok, r.deadlocks, r.attempts, r.balances.0, r.balances.1
    );
    ensure!(r.ok == 2 && r.deadlocks == 0 && r.attempts == [1, 1]);
    ensure!(r.balances == (dong(48_000), dong(52_000)));

    let (r, total) = with_lab(async move |lab| ordered_stress(&lab).await).await?;
    println!(
        "12 lần chuyển ba ví hai chiều, khóa theo id: {} ok, {} deadlock; số lần thử tối đa {}; tổng ba ví {}",
        r.ok,
        r.deadlocks,
        r.attempts.last().copied().unwrap_or(0),
        total
    );
    ensure!(r.ok == 12 && r.deadlocks == 0 && r.attempts == [1; 12]);
    ensure!(total == dong(300_000));
    ensure!(r.audit.total_balance == 0 && r.audit.projection_mismatch.is_empty());

    let v = with_lab(async move |lab| deadlock_victim(&lab).await).await?;
    println!(
        "nạn nhân chưa ROLLBACK: {} bên vẫn khóa đủ hai hàng; nạn nhân nhận {}, lệnh kế của nó SQLSTATE {}",
        v.survivors,
        v.error.as_deref().unwrap_or("-"),
        v.next_statement.as_deref().unwrap_or("-")
    );
    ensure!(v.survivors == 1);
    ensure!(v.error.as_deref() == Some("40P01"));
    ensure!(v.next_statement.as_deref() == Some("25P02"));
    Ok(())
}

async fn run_locks() -> Result<()> {
    println!("== 4. Một ví đang bị FOR UPDATE: ba cách đọc khác nhau ==");
    let c = with_lab(async move |lab| lock_variants(&lab).await).await?;
    println!("SELECT thường        : {:?}", c.plain);
    println!("FOR UPDATE SKIP LOCKED: {:?}", c.skip_locked);
    println!(
        "FOR UPDATE NOWAIT    : SQLSTATE {}",
        c.nowait_sqlstate.as_deref().unwrap_or("-")
    );
    ensure!(c.plain == ["customer:CU-001", "customer:CU-002"]);
    ensure!(c.skip_locked == ["customer:CU-002"]);
    ensure!(c.nowait_sqlstate.as_deref() == Some("55P03"));

    println!("== 5. Bút toán không cần version ==");
    let (on_entry, on_posting) =
        with_lab(async move |lab| journal_is_append_only(&lab).await).await?;
    println!(
        "UPDATE journal_entries: SQLSTATE {}; UPDATE postings: SQLSTATE {}",
        on_entry.as_deref().unwrap_or("-"),
        on_posting.as_deref().unwrap_or("-")
    );
    ensure!(on_entry.as_deref() == Some("TN001") && on_posting.as_deref() == Some("TN001"));
    Ok(())
}

async fn run_measure() -> Result<()> {
    println!("== Số đo trên máy viết bài, không phải SLA (16 task, mọi task đều thành công) ==");
    let mut load = String::new();
    if let Ok(out) = std::process::Command::new("uptime").output() {
        load = String::from_utf8_lossy(&out.stdout).trim().to_string();
    }
    println!("máy: {load}");
    println!(
        "{:<11} {:>5} {:>4}  {:<44} {:>6} {:>4} {:>7}",
        "chế độ", "task", "ví", "số lần thử → số task", "tổng", "max", "ms"
    );
    let mut configs: Vec<(usize, usize)> = vec![(16, 16), (16, 4), (16, 1)];
    configs.extend([(2, 1), (4, 1), (8, 1)]);
    for mode in [Mode::Optimistic, Mode::ForUpdate] {
        for &(tasks, wallets) in &configs {
            let s = measure(mode, tasks, wallets).await?;
            let hist: Vec<String> = s
                .histogram
                .iter()
                .map(|(k, v)| format!("{k}→{v}"))
                .collect();
            println!(
                "{:<11} {:>5} {:>4}  {:<44} {:>6} {:>4} {:>7}",
                mode_name(s.mode),
                s.tasks,
                s.wallets,
                hist.join(" "),
                s.total_attempts,
                s.max_attempts,
                s.elapsed_ms
            );
        }
    }
    Ok(())
}

async fn run(verify: bool, measure_mode: bool) -> Result<()> {
    if measure_mode {
        return run_measure().await;
    }
    run_stale().await?;
    run_race().await?;
    run_join_trap().await?;
    run_deadlock().await?;
    run_locks().await?;
    if verify {
        println!("VERIFY OK");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let verify = args.iter().any(|a| a == "--verify");
    let measure_mode = args.iter().any(|a| a == "--measure");
    match run(verify, measure_mode).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("LỖI: {err:#}");
            ExitCode::FAILURE
        }
    }
}
