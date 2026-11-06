//! Đếm allocation của ba cách cài bằng dhat. Chỉ build khi bật feature `dhat-heap`:
//! `cargo run --release --features dhat-heap --bin alloc-profile`.
//!
//! Tham số: `--n N` (mặc định 300), `--only slow|scan|indexed` (chỉ chạy một cách, để
//! `dhat-heap.json` chỉ chứa cách đó), `--out FILE` (đường dẫn JSON, mặc định `dhat-heap.json` trong thư mục tạm).

use std::process::ExitCode;

use dhat::{HeapStats, Profiler};
use ledger_profiling::books::{Book, IndexedBook, ScanBook, SlowBook};
use ledger_profiling::workload::{Outcome, make_workload, post_and_read_balances};

// Allocator của dhat thay allocator mặc định cho cả chương trình; binary này chỉ tồn tại khi bật feature.
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// Số allocation (block) và byte cấp phát trong lúc chạy một kịch bản, cùng đỉnh bộ nhớ sống.
struct Measured {
    outcome: Outcome,
    blocks: u64,
    bytes: u64,
    peak_bytes: usize,
}

fn measure<B: Book>(mut book: B, n: u64) -> Measured {
    // Dữ liệu đầu vào dựng TRƯỚC khi chụp số liệu, nên không tính vào kịch bản.
    let workload = make_workload(n);
    let before = HeapStats::get();
    let outcome = post_and_read_balances(&mut book, workload);
    let after = HeapStats::get();
    Measured {
        outcome,
        blocks: after.total_blocks - before.total_blocks,
        bytes: after.total_bytes - before.total_bytes,
        peak_bytes: after.max_bytes,
    }
}

fn show(name: &str, m: &Measured) {
    println!(
        "{name:<8} {:>12} block {:>14} byte  (đỉnh sống toàn tiến trình {} byte)",
        m.blocks, m.bytes, m.peak_bytes
    );
}

fn flag(args: &[String], name: &str) -> Option<String> {
    let at = args.iter().position(|a| a == name)?;
    args.get(at + 1).cloned()
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let n: u64 = flag(&args, "--n")
        .and_then(|s| s.parse().ok())
        .unwrap_or(300);
    let only = flag(&args, "--only");
    let out = flag(&args, "--out").unwrap_or_else(|| {
        std::env::temp_dir()
            .join("dhat-heap.json")
            .to_string_lossy()
            .into_owned()
    });

    // Không `.testing()`: khi `_profiler` bị drop ở cuối `main`, dhat ghi file JSON cho `dhat-top`/dh_view.
    let _profiler = Profiler::builder().file_name(&out).build();

    println!("{n} bút toán, {} lần đọc số dư", 2 * n);
    if let Some(name) = only {
        let m = match name.as_str() {
            "slow" => measure(SlowBook::new(), n),
            "scan" => measure(ScanBook::new(), n),
            "indexed" => measure(IndexedBook::new(), n),
            other => {
                eprintln!("LỖI: --only không biết: {other}");
                return ExitCode::FAILURE;
            }
        };
        show(&name, &m);
        println!("JSON: {out}");
        return ExitCode::SUCCESS;
    }

    let slow = measure(SlowBook::new(), n);
    let scan = measure(ScanBook::new(), n);
    let indexed = measure(IndexedBook::new(), n);
    show("slow", &slow);
    show("scan", &scan);
    show("indexed", &indexed);

    // Kiểm hợp đồng, không kiểm số tuyệt đối của máy. Phần `slow` cấp phát THÊM so với `scan` là
    // 10 block cho mỗi bút toán được duyệt (1 Vec + 3 String của to_vec, 6 chuỗi của format!),
    // tức đúng 10 * n * (n + 1) block, vì mỗi lần đọc thứ k duyệt k bút toán và có 2n lần đọc.
    assert_eq!(slow.outcome.checksum, scan.outcome.checksum);
    assert_eq!(slow.outcome.checksum, indexed.outcome.checksum);
    assert_eq!(
        slow.blocks - scan.blocks,
        10 * n * (n + 1),
        "slow: 10 block mỗi bút toán được duyệt"
    );
    // scan không cấp phát khi quét: chỉ Vec của sổ tự lớn (cỡ log2 n lần).
    assert!(scan.blocks <= 2 * n.ilog2() as u64 + 8);
    // indexed cấp phát nhiều hơn scan một chút (mỗi tài khoản mới một lần clone + bảng băm tự lớn),
    // nhưng không phụ thuộc số lần đọc.
    assert!(indexed.blocks > scan.blocks && indexed.blocks < 3 * n);
    assert!(indexed.blocks < slow.blocks / 100);
    println!("ALLOC OK");
    ExitCode::SUCCESS
}
