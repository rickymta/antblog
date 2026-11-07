//! Lab bài 59: Drop tất định, đếm allocation bằng global allocator tự viết, lifetime nâng cao.
//!
//! `cargo run` in các bằng chứng; `-- --verify` kiểm giá trị kỳ vọng; `-- --workload` in số liệu
//! của đúng khối lượng việc mà chương trình C# trong `dotnet-compare/` cũng chạy.

use std::alloc::System;
use std::hint::black_box;
use std::mem::size_of;
use std::process::ExitCode;
use std::time::Instant;

use ledger_core::{AccountId, Money, Posting};
use ledger_memory::counting::{Counting, Stats};
use ledger_memory::lifetimes;
use ledger_memory::raii;
use ledger_memory::rss;
use ledger_memory::workload::{build_ledger, make_entry, summarize};

// Mọi `Box`, `Vec`, `String` của chương trình này đi qua bộ đếm rồi mới tới allocator hệ thống.
#[global_allocator]
static ALLOC: Counting<System> = Counting::new(System);

const ENTRIES: u64 = 10_000;

/// Chạy `f` và trả kết quả cùng số cấp phát xảy ra bên trong. Không in gì trong `f`.
fn measure<R>(f: impl FnOnce() -> R) -> (R, Stats) {
    let before = ALLOC.stats();
    let result = f();
    let after = ALLOC.stats();
    (result, after.since(before))
}

/// Mọi số đo của lần chạy; `--verify` và phần in đều đọc từ đây để không lệch nhau.
struct Report {
    money_on_stack: Stats,
    boxed_money: Stats,
    boxed_array: Stats,
    string_from: Stats,
    account_id: Stats,
    vec_growth: Stats,
    vec_capacities: Vec<usize>,
    vec_with_capacity: Stats,
    one_entry: Stats,
    ten_thousand: Stats,
    ledger_len: usize,
    live_after_build: u64,
    live_after_drop: u64,
    live_baseline: u64,
    drop_frees: u64,
    summary: (i64, i64, i64),
    cow_clean: Stats,
    cow_dirty: Stats,
}

fn collect() -> Report {
    let (_, money_on_stack) = measure(|| black_box(Money::from_dong(25_500)));
    let (boxed, boxed_money) = measure(|| Box::new(Money::from_dong(25_500)));
    drop(boxed);
    let (boxed, boxed_array) = measure(|| Box::new([Money::from_dong(0); 512]));
    drop(boxed);
    let (s, string_from) = measure(|| String::from(black_box("customer:CU-001")));
    drop(s);
    let (id, account_id) = measure(|| AccountId::new(black_box("customer:CU-001")));
    drop(id);

    // Vec tự tăng: ghi lại dung lượng mỗi khi nó đổi.
    let mut capacities = Vec::with_capacity(8);
    let (v, vec_growth) = measure(|| {
        let mut v: Vec<Money> = Vec::new();
        let mut last = v.capacity();
        for _ in 0..10 {
            v.push(Money::from_dong(1));
            if v.capacity() != last {
                last = v.capacity();
                capacities.push(last);
            }
        }
        v
    });
    drop(v);
    let (v2, vec_with_capacity) = measure(|| Vec::<Money>::with_capacity(10));
    drop(v2);

    let (entry, one_entry) = measure(|| make_entry(black_box(1)));
    drop(entry);

    let live_baseline = ALLOC.stats().live_bytes;
    ALLOC.reset_peak();
    let (ledger, ten_thousand) = measure(|| build_ledger(black_box(ENTRIES)));
    let live_after_build = ALLOC.stats().live_bytes;
    let ledger_len = ledger.len();
    let summary = summarize(&ledger);
    let (_, dropped) = measure(|| drop(ledger));
    let live_after_drop = ALLOC.stats().live_bytes;

    let (c, cow_clean) = measure(|| lifetimes::normalize_memo(black_box("chuyến TN-0001")));
    drop(c);
    let (c, cow_dirty) = measure(|| lifetimes::normalize_memo(black_box("  chuyến\nTN-0001 ")));
    drop(c);

    Report {
        money_on_stack,
        boxed_money,
        boxed_array,
        string_from,
        account_id,
        vec_growth,
        vec_capacities: capacities,
        vec_with_capacity,
        one_entry,
        ten_thousand: Stats {
            peak_bytes: ALLOC.stats().peak_bytes,
            ..ten_thousand
        },
        ledger_len,
        live_after_build,
        live_after_drop,
        live_baseline,
        drop_frees: dropped.frees,
        summary,
        cow_clean,
        cow_dirty,
    }
}

fn print_report(r: &Report) {
    println!("== 1. Stack và heap: đếm cấp phát bằng global allocator tự viết ==");
    println!(
        "size_of: Money={} Posting={} String={} Vec<Posting>={} Box<Money>={}",
        size_of::<Money>(),
        size_of::<Posting>(),
        size_of::<String>(),
        size_of::<Vec<Posting>>(),
        size_of::<Box<Money>>()
    );
    let line = |name: &str, s: &Stats| {
        println!(
            "{name:<34} {:>3} block, {:>6} byte, {} realloc",
            s.blocks, s.bytes, s.reallocs
        );
    };
    line("Money trên stack", &r.money_on_stack);
    line("Box::new(Money)", &r.boxed_money);
    line("Box::new([Money; 512])", &r.boxed_array);
    line("String::from(\"customer:CU-001\")", &r.string_from);
    line("AccountId::new(..)", &r.account_id);
    line("Vec::new + 10 push", &r.vec_growth);
    println!("dung lượng Vec đổi qua các lần: {:?}", r.vec_capacities);
    line("Vec::with_capacity(10)", &r.vec_with_capacity);
    line("make_entry(1) (một bút toán)", &r.one_entry);

    println!("\n== 2. Ghi {ENTRIES} bút toán bằng ledger-core ==");
    let t = &r.ten_thousand;
    println!(
        "{} block, {} byte, {} realloc ({} block/bút toán)",
        t.blocks,
        t.bytes,
        t.reallocs,
        t.blocks / ENTRIES
    );
    println!(
        "đỉnh bộ nhớ sống {} byte; sau build {} byte, sau drop {} byte (nền {} byte)",
        t.peak_bytes, r.live_after_build, r.live_after_drop, r.live_baseline
    );
    println!(
        "drop sổ cái trả {} block trong một lời gọi, không chờ GC",
        r.drop_frees
    );
    println!(
        "kết quả nghiệp vụ: DR-001={} revenue={} tổng={}",
        r.summary.0, r.summary.1, r.summary.2
    );

    println!("\n== 3. Drop tất định ==");
    println!("thứ tự drop: {:?}", raii::drop_order());
    println!("giao dịch RAII: {:?}", raii::transaction_outcomes());
    let (held, freed) = raii::mutex_guard_scope();
    println!("khóa: try_lock lúc đang giữ lỗi={held}, sau khi ra scope được={freed}");

    println!("\n== 4. Lifetime nâng cao ==");
    let ledger = build_ledger(100);
    println!(
        "thread::spawn + move: tổng = {}",
        lifetimes::sum_in_spawned_thread(vec![Money::from_dong(20_400), Money::from_dong(5_100)])
    );
    println!(
        "thread::scope mượn &Ledger: (bút toán, posting) = {:?}",
        lifetimes::count_in_scoped_threads(&ledger)
    );
    println!(
        "Cow: ghi chú sạch = {} block, ghi chú cần sửa = {} block",
        r.cow_clean.blocks, r.cow_dirty.blocks
    );
    println!(
        "HRTB: {:?}",
        lifetimes::apply_to_both(
            lifetimes::without_kind,
            "customer:CU-001",
            &String::from("driver:DR-001")
        )
    );
    let st = lifetimes::Statement::parse("TN-0001,25500\nTN-0002,12000".to_string());
    println!("struct tự tham chiếu bằng chỉ số: {:?}", st.first_line());
    println!(
        "Pin: future giữ tham chiếu qua await = {:?}",
        lifetimes::poll_self_referential_future()
    );

    println!("\n== 5. RSS của tiến trình ==");
    match rss::current_kib() {
        Some(kib) => println!("RSS hiện tại: {kib} KiB (số đo trên máy viết bài, không phải SLA)"),
        None => println!("không đọc được RSS (thiếu `ps`)"),
    }
}

/// Số liệu của khối lượng 10.000 bút toán, cùng khuôn với `dotnet-compare`.
fn print_workload(rounds: u32) {
    let rss_start = rss::current_kib();
    ALLOC.reset_peak();
    let before = ALLOC.stats();
    let started = Instant::now();
    let mut last = None;
    for _ in 0..rounds {
        last = Some(build_ledger(ENTRIES));
    }
    let elapsed = started.elapsed();
    let used = ALLOC.stats().since(before);
    let ledger = last.expect("rounds >= 1");
    println!("runtime=rust allocator=system+counting entries={ENTRIES} rounds={rounds}");
    println!(
        "allocated_bytes={} blocks={} peak_live_bytes={}",
        used.bytes, used.blocks, used.peak_bytes
    );
    println!("gc_count=none (không có GC)");
    println!(
        "rss_start_kib={} rss_end_kib={}",
        rss_start.unwrap_or(0),
        rss::current_kib().unwrap_or(0)
    );
    println!("elapsed_ms={:.1}", elapsed.as_secs_f64() * 1000.0);
    println!("checksum={:?}", summarize(&ledger));
}

fn verify() -> Result<(), String> {
    let r = collect();
    let mut failures = Vec::new();
    let mut check = |ok: bool, what: &str| {
        if !ok {
            failures.push(what.to_string());
        }
    };

    // Stack không cấp phát; Box, String, AccountId mỗi cái đúng một block.
    check(
        r.money_on_stack.blocks == 0,
        "Money trên stack không cấp phát",
    );
    check(
        r.boxed_money.blocks == 1 && r.boxed_money.bytes == 8,
        "Box<Money> là một block 8 byte",
    );
    check(
        r.boxed_array.blocks == 1 && r.boxed_array.bytes == 4_096,
        "Box<[Money; 512]> là một block 4096 byte",
    );
    check(
        r.string_from.blocks == 1 && r.string_from.bytes == 15,
        "String::from là một block 15 byte",
    );
    check(r.account_id.blocks == 1, "AccountId::new là một block");

    // Vec tăng 4, 8, 16: ba block (một alloc, hai realloc).
    check(r.vec_capacities == [4, 8, 16], "dung lượng Vec đi 4, 8, 16");
    check(
        r.vec_growth.blocks == 3 && r.vec_growth.reallocs == 2,
        "Vec tăng trưởng: 3 block, trong đó 2 realloc",
    );
    check(r.vec_with_capacity.blocks == 1, "with_capacity: một block");

    // Khối lượng 10.000 bút toán: cùng số block như dhat ở bài 30, và drop trả đúng từng ấy.
    check(r.ledger_len == 10_000, "sổ có 10.000 bút toán");
    check(
        r.ten_thousand.blocks == 90_026,
        "10.000 bút toán cấp phát 90.026 block (trùng dhat bài 30)",
    );
    check(
        r.ten_thousand.blocks / ENTRIES == 9,
        "khoảng 9 block mỗi bút toán",
    );
    check(
        r.live_after_drop == r.live_baseline,
        "drop sổ cái trả hết bộ nhớ về mức nền",
    );
    check(
        r.drop_frees + r.ten_thousand.frees == r.ten_thousand.blocks - r.ten_thousand.reallocs,
        "mọi block đã cấp phát (không tính realloc) đều được trả đúng một lần",
    );
    check(
        r.summary == (1_020_000, 51_000_000, 0),
        "kết quả nghiệp vụ: DR-001, revenue, tổng bằng 0",
    );

    // Drop tất định.
    check(
        raii::drop_order()
            == [
                "open conn",
                "open lock",
                "open early",
                "drop early",
                "open pair.first",
                "open pair.second",
                "open leaked",
                "end of scope",
                "drop pair.first",
                "drop pair.second",
                "drop lock",
                "drop conn",
            ],
        "thứ tự drop: ngược khai báo, field theo khai báo, forget không drop",
    );
    check(
        raii::transaction_outcomes()
            == [
                "tx begin",
                "tx commit",
                "tx begin",
                "tx rollback",
                "tx begin",
                "tx rollback",
                "panic caught: true",
            ],
        "giao dịch RAII: commit, rollback khi lỗi, rollback khi panic",
    );
    check(
        raii::mutex_guard_scope() == (true, true),
        "MutexGuard nhả khóa khi ra scope",
    );

    // Lifetime nâng cao.
    check(
        lifetimes::sum_in_spawned_thread(vec![Money::from_dong(20_400), Money::from_dong(5_100)])
            == 25_500,
        "thread::spawn với move",
    );
    check(
        lifetimes::count_in_scoped_threads(&build_ledger(100)) == (100, 300),
        "thread::scope mượn sổ cái",
    );
    check(r.cow_clean.blocks == 0, "Cow::Borrowed không cấp phát");
    check(r.cow_dirty.blocks >= 1, "Cow::Owned có cấp phát");
    check(
        lifetimes::apply_to_both(lifetimes::without_kind, "customer:CU-001", "driver:DR-001")
            == (6, 6),
        "HRTB với hàm thường",
    );
    check(
        lifetimes::Statement::parse("TN-0001,25500\nTN-0002,12000".to_string()).first_line()
            == "TN-0001,25500",
        "struct lưu chỉ số thay vì tham chiếu",
    );
    check(
        lifetimes::poll_self_referential_future() == (true, -25_500),
        "future tự tham chiếu chạy đúng khi được ghim",
    );

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n  - "))
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--verify") {
        return match verify() {
            Ok(()) => {
                println!("VERIFY OK");
                ExitCode::SUCCESS
            }
            Err(failures) => {
                eprintln!("VERIFY FAILED:\n  - {failures}");
                ExitCode::FAILURE
            }
        };
    }
    if args.iter().any(|a| a == "--workload") {
        let rounds = args
            .iter()
            .position(|a| a == "--rounds")
            .and_then(|i| args.get(i + 1))
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        print_workload(rounds);
        return ExitCode::SUCCESS;
    }
    print_report(&collect());
    ExitCode::SUCCESS
}
