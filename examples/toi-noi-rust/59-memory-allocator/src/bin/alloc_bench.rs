//! So allocator trên cùng một khối lượng việc: mỗi vòng ghi 10.000 bút toán rồi bỏ sổ cái.
//!
//! Chọn allocator bằng feature (chỉ một): mặc định là allocator hệ thống,
//! `--features mimalloc`, `--features jemalloc`. Binary này KHÔNG có bộ đếm allocation,
//! vì bộ đếm cộng thêm chi phí vào đúng thứ ta đang đo.
//!
//! `cargo run --release --bin alloc-bench -- --threads 4 --rounds 20`
//! Thêm `--verify` để chỉ kiểm kết quả nghiệp vụ không đổi theo allocator.

use std::process::ExitCode;
use std::time::Instant;

use ledger_memory::rss;
use ledger_memory::workload::{build_ledger, summarize};

#[cfg(all(feature = "mimalloc", feature = "jemalloc"))]
compile_error!("chỉ bật một trong hai feature: mimalloc hoặc jemalloc");

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

const ENTRIES: u64 = 10_000;

fn allocator_name() -> &'static str {
    if cfg!(feature = "mimalloc") {
        "mimalloc"
    } else if cfg!(feature = "jemalloc") {
        "jemalloc"
    } else {
        "system"
    }
}

fn arg(args: &[String], name: &str, default: u32) -> u32 {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Mỗi thread ghi `rounds` sổ cái, giữ lại tóm tắt của sổ cuối; các sổ cũ bị drop mỗi vòng.
fn run(threads: u32, rounds: u32) -> (i64, i64, i64) {
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            std::thread::spawn(move || {
                let mut summary = (0, 0, 0);
                for _ in 0..rounds {
                    summary = summarize(&build_ledger(ENTRIES));
                }
                summary
            })
        })
        .collect();
    let mut summary = (0, 0, 0);
    for h in handles {
        summary = h.join().expect("thread không panic");
    }
    summary
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let threads = arg(&args, "--threads", 1);
    let rounds = arg(&args, "--rounds", 1);

    if args.iter().any(|a| a == "--verify") {
        let summary = run(2, 2);
        if summary == (1_020_000, 51_000_000, 0) {
            println!("VERIFY OK allocator={}", allocator_name());
            return ExitCode::SUCCESS;
        }
        eprintln!(
            "VERIFY FAILED allocator={} summary={summary:?}",
            allocator_name()
        );
        return ExitCode::FAILURE;
    }

    let rss_start = rss::current_kib().unwrap_or(0);
    let started = Instant::now();
    let summary = run(threads, rounds);
    let elapsed = started.elapsed();
    println!(
        "runtime=rust allocator={} threads={threads} rounds={rounds} entries={ENTRIES}",
        allocator_name()
    );
    println!(
        "rss_start_kib={rss_start} rss_end_kib={}",
        rss::current_kib().unwrap_or(0)
    );
    println!("elapsed_ms={:.1}", elapsed.as_secs_f64() * 1000.0);
    println!("checksum={summary:?}");
    ExitCode::SUCCESS
}
