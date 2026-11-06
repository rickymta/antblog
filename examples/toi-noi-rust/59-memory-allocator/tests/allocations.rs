//! Số cấp phát là sự thật của đoạn code, không phụ thuộc tải máy: kiểm bằng `assert_eq!`.
//! Chỉ MỘT hàm `#[test]`: allocator là toàn cục, nhiều test song song sẽ đếm lẫn của nhau.

use std::alloc::System;
use std::hint::black_box;

use ledger_memory::counting::Counting;
use ledger_memory::workload::{build_ledger, make_entry};

#[global_allocator]
static ALLOC: Counting<System> = Counting::new(System);

#[test]
fn allocation_counts_are_exact_and_deterministic() {
    // Box<i64>: đúng một block 8 byte, và được trả khi Box bị drop.
    let before = ALLOC.stats();
    let boxed = Box::new(black_box(25_500_i64));
    let during = ALLOC.stats().since(before);
    assert_eq!((during.blocks, during.bytes), (1, 8));
    drop(boxed);
    assert_eq!(ALLOC.stats().live_bytes, before.live_bytes);

    // Vec tự tăng 4, 8, 16 cho 10 phần tử: 3 block (2 realloc); with_capacity: 1 block.
    let before = ALLOC.stats();
    let mut grown: Vec<i64> = Vec::new();
    for i in 0..10 {
        grown.push(black_box(i));
    }
    let growth = ALLOC.stats().since(before);
    assert_eq!((growth.blocks, growth.reallocs, growth.bytes), (3, 2, 224));
    drop(grown);
    let before = ALLOC.stats();
    let sized: Vec<i64> = Vec::with_capacity(black_box(10));
    assert_eq!(ALLOC.stats().since(before).blocks, 1);
    drop(sized);

    // Hai lần đo cùng một việc ra cùng một số: đó là điều làm số cấp phát đáng tin hơn số giờ.
    let count = || {
        let before = ALLOC.stats();
        let entry = make_entry(black_box(1));
        let used = ALLOC.stats().since(before);
        drop(entry);
        (used.blocks, used.bytes)
    };
    assert_eq!(count(), count());
    assert_eq!(count().0, 8);

    // 10.000 bút toán: 90.026 block, và drop trả bộ nhớ về mức nền.
    let baseline = ALLOC.stats().live_bytes;
    let before = ALLOC.stats();
    let ledger = build_ledger(black_box(10_000));
    let built = ALLOC.stats().since(before);
    assert_eq!(built.blocks, 90_026);
    assert!(ALLOC.stats().live_bytes > baseline);
    drop(ledger);
    assert_eq!(ALLOC.stats().live_bytes, baseline);
}
