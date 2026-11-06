//! Đếm allocation bằng dhat. Chỉ build khi bật feature `dhat-heap`:
//! `cargo run --release --features dhat-heap --bin alloc-profile`.

use std::hint::black_box;

use dhat::{HeapStats, Profiler};
use ledger_bench::workload::{flatten_grow, flatten_with_capacity, make_ledger};

// Allocator của dhat thay allocator mặc định cho cả chương trình. Binary này chỉ tồn tại khi
// bật feature, nên build bình thường không bao giờ trả chi phí theo dõi từng allocation.
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// Hiệu hai lần chụp: (số block cấp phát, số byte cấp phát) trong đoạn giữa.
fn delta(before: &HeapStats, after: &HeapStats) -> (u64, u64) {
    (
        after.total_blocks - before.total_blocks,
        after.total_bytes - before.total_bytes,
    )
}

fn main() {
    // `.testing()`: không ghi dhat-heap.json khi kết thúc; số liệu đọc bằng `HeapStats::get()`.
    let _profiler = Profiler::builder().testing().build();
    const N: u64 = 10_000;

    let s0 = HeapStats::get();
    let ledger = make_ledger(N);
    let s1 = HeapStats::get();
    let (blocks, bytes) = delta(&s0, &s1);
    println!(
        "make_ledger({N}): {blocks} block, {bytes} byte ({} block/bút toán)",
        blocks / N
    );

    let s2 = HeapStats::get();
    let grown = flatten_grow(black_box(&ledger));
    let s3 = HeapStats::get();
    let (grow_blocks, grow_bytes) = delta(&s2, &s3);
    println!(
        "Vec::new + push        : {grow_blocks} block, {grow_bytes} byte, len {}",
        grown.len()
    );
    drop(grown);

    let s4 = HeapStats::get();
    let sized = flatten_with_capacity(black_box(&ledger));
    let s5 = HeapStats::get();
    let (cap_blocks, cap_bytes) = delta(&s4, &s5);
    println!(
        "Vec::with_capacity     : {cap_blocks} block, {cap_bytes} byte, len {}",
        sized.len()
    );

    // Kiểm tra hợp đồng, không kiểm số tuyệt đối của máy: with_capacity cấp phát đúng một lần.
    assert_eq!(cap_blocks, 1, "with_capacity chỉ cấp phát một lần");
    assert!(
        grow_blocks > cap_blocks,
        "tự tăng phải cấp phát nhiều lần hơn"
    );
    assert!(
        grow_bytes > cap_bytes,
        "tự tăng phải cấp phát nhiều byte hơn"
    );
    println!("ALLOC OK");
}
