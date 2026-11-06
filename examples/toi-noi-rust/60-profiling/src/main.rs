//! Lab bài 60: chạy kịch bản "ghi bút toán rồi đọc số dư" trên ba cách cài sổ cái.
//!
//! ```text
//! ledger-profiling --verify                  kiểm bộ đếm và đáp án, không kiểm thời gian
//! ledger-profiling --report [--n N]          bảng bộ đếm + thời gian kèm load average
//! ledger-profiling --workload slow|scan|indexed [--n N]   một kịch bản, dùng làm đích cho profiler
//! ledger-profiling dhat-top FILE [K]         K chỗ cấp phát nhiều nhất trong dhat-heap.json
//! ```

use std::process::ExitCode;
use std::time::{Duration, Instant};

use ledger_core::Money;
use ledger_profiling::books::{Book, IndexedBook, ScanBook, SlowBook};
use ledger_profiling::runtime::{CHUNK, Mode, run_scenario};
use ledger_profiling::workload::{
    Outcome, customer_of, expected_scanned, make_entry, make_workload, post_and_read_balances,
};
use ledger_profiling::{dhat_top, sys};

const DEFAULT_N: u64 = 2_000;
const VERIFY_N: u64 = 500;

fn run(name: &str, n: u64) -> Result<(Outcome, Duration), String> {
    let workload = make_workload(n);
    // Chỉ đo phần kịch bản, không đo dựng dữ liệu.
    let start = Instant::now();
    let outcome = match name {
        "slow" => post_and_read_balances(&mut SlowBook::new(), workload),
        "scan" => post_and_read_balances(&mut ScanBook::new(), workload),
        "indexed" => post_and_read_balances(&mut IndexedBook::new(), workload),
        other => return Err(format!("workload không biết: {other} (slow|scan|indexed)")),
    };
    Ok((outcome, start.elapsed()))
}

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    let at = args.iter().position(|a| a == flag)?;
    args.get(at + 1).cloned()
}

fn parse_n(args: &[String]) -> Result<u64, String> {
    match arg_value(args, "--n") {
        None => Ok(DEFAULT_N),
        Some(raw) => raw.parse().map_err(|_| format!("--n không phải số: {raw}")),
    }
}

fn report(n: u64) -> Result<(), String> {
    println!(
        "Kịch bản cố định: {n} bút toán, mỗi chuyến ghi rồi đọc 2 số dư ({} lần đọc)",
        2 * n
    );
    println!(
        "{:<8} {:>12} {:>18} {:>12}",
        "cách cài", "lần gọi", "posting đã duyệt", "thời gian"
    );
    for name in ["slow", "scan", "indexed"] {
        let (outcome, elapsed) = run(name, n)?;
        println!(
            "{:<8} {:>12} {:>18} {:>9.1?}",
            name, outcome.balance_calls, outcome.scanned_postings, elapsed
        );
    }
    println!("load average lúc đo: {}", sys::load_average());
    println!("Thời gian chỉ để tham khảo, nhiễu theo tải máy; hai cột đầu mới ổn định.");
    Ok(())
}

fn verify() -> Result<(), String> {
    // 1. Bút toán TN-0001 đúng số nghiệp vụ.
    let first = make_entry(1);
    let amounts: Vec<i64> = first.postings().iter().map(|p| p.amount.dong()).collect();
    check(
        amounts == [-25_500, 5_100, 20_400],
        "TN-0001 phải là -25.500 / +5.100 / +20.400",
    )?;

    // 2. Ba cách cài cho cùng đáp án trên cùng kịch bản.
    let (slow, _) = run("slow", VERIFY_N)?;
    let (scan, _) = run("scan", VERIFY_N)?;
    let (indexed, _) = run("indexed", VERIFY_N)?;
    check(
        slow.checksum == scan.checksum,
        "slow và scan phải cùng checksum",
    )?;
    check(
        slow.checksum == indexed.checksum,
        "slow và indexed phải cùng checksum",
    )?;

    // 3. Bộ đếm theo công thức: không phụ thuộc máy, không phụ thuộc tải.
    let calls = 2 * VERIFY_N;
    check(slow.balance_calls == calls, "slow: số lần gọi")?;
    check(scan.balance_calls == calls, "scan: số lần gọi")?;
    check(indexed.balance_calls == calls, "indexed: số lần gọi")?;
    check(
        slow.scanned_postings == expected_scanned(VERIFY_N),
        "slow: posting đã duyệt",
    )?;
    check(
        scan.scanned_postings == expected_scanned(VERIFY_N),
        "scan: posting đã duyệt",
    )?;
    check(indexed.scanned_postings == 0, "indexed: không quét")?;

    // 4. Số dư cụ thể: customer:CU-001 mới có một chuyến sau TN-0001.
    let mut book = IndexedBook::new();
    book.post(first);
    let customer = customer_of(1);
    check(
        book.balance(&customer) == Money::from_dong(-25_500),
        "customer:CU-001 sau TN-0001 phải là -25.500",
    )?;

    // 5. Runtime một luồng: handler không `.await` làm heartbeat đói, handler chia đoạn thì không.
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .map_err(|e| format!("không dựng được runtime: {e}"))?;
    let scan_book = || {
        let mut book = ScanBook::new();
        for entry in make_workload(4 * CHUNK as u64).entries {
            book.post(entry);
        }
        book
    };
    let blocking = rt.block_on(run_scenario(Mode::Blocking, scan_book(), customer_of(1), 3));
    let chunked = rt.block_on(run_scenario(Mode::Chunked, scan_book(), customer_of(1), 3));
    check(
        blocking.ticks_at_finish == 0,
        "handler chặn: heartbeat phải đứng yên",
    )?;
    check(
        chunked.ticks_at_finish > 0,
        "handler chia đoạn: heartbeat phải chạy xen kẽ",
    )?;
    check(
        blocking.balance == chunked.balance,
        "hai kiểu handler cùng số dư",
    )?;

    // 6. Bài đo cùng mã của dhat-top phải đọc được JSON giả lập tối thiểu.
    dhat_top::self_check()?;

    println!(
        "heartbeat khi handler xong: chặn {} nhịp, chia đoạn {} nhịp",
        blocking.ticks_at_finish, chunked.ticks_at_finish
    );
    println!(
        "slow    : {} lần gọi, {} posting đã duyệt",
        slow.balance_calls, slow.scanned_postings
    );
    println!(
        "scan    : {} lần gọi, {} posting đã duyệt",
        scan.balance_calls, scan.scanned_postings
    );
    println!(
        "indexed : {} lần gọi, {} posting đã duyệt",
        indexed.balance_calls, indexed.scanned_postings
    );
    println!("VERIFY OK");
    Ok(())
}

fn check(condition: bool, message: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(format!("SAI: {message}"))
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("--verify") => verify(),
        Some("--report") => parse_n(&args).and_then(report),
        Some("--workload") => match (args.get(1), parse_n(&args)) {
            (Some(name), Ok(n)) => run(name, n).map(|(outcome, elapsed)| {
                println!(
                    "{name}: n={n}, {} lần gọi, {} posting đã duyệt, checksum {}, {elapsed:.1?}",
                    outcome.balance_calls, outcome.scanned_postings, outcome.checksum
                );
            }),
            (_, Err(e)) => Err(e),
            (None, _) => Err("thiếu tên workload".to_string()),
        },
        Some("dhat-top") => match args.get(1) {
            Some(path) => {
                let k = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(5);
                dhat_top::print_top(path, k)
            }
            None => Err("dùng: dhat-top FILE [K]".to_string()),
        },
        _ => Err("dùng: --verify | --report [--n N] | --workload slow|scan|indexed [--n N] | dhat-top FILE [K]".to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("LỖI: {message}");
            ExitCode::FAILURE
        }
    }
}
