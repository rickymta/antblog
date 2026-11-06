//! Benchmark sổ cái bằng criterion: tạo bút toán, ghi, tính số dư, thu Vec.
//! Chạy: `cargo bench --manifest-path examples/toi-noi-rust/30-criterion-bench/Cargo.toml`.

use std::hint::black_box;
use std::time::Duration;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use ledger_bench::workload::{flatten_grow, flatten_with_capacity, flatten_with_known_capacity};
use ledger_bench::{IndexedLedger, make_entry, make_ledger};
use ledger_core::{AccountId, Ledger};

const SIZES: [u64; 3] = [1_000, 10_000, 100_000];

/// Chi phí tạo một `JournalEntry` (3 posting, 5 chuỗi cấp phát) chưa gồm việc ghi vào sổ.
fn create_entry(c: &mut Criterion) {
    c.bench_function("tao_journal_entry", |b| {
        // `black_box` che số chuyến để compiler không tính sẵn chuỗi.
        b.iter(|| make_entry(black_box(1)));
    });
}

/// Ghi `n` bút toán vào sổ rỗng. Bút toán dựng sẵn trong `setup`, nên chỉ đo `Ledger::post`.
fn post(c: &mut Criterion) {
    let mut group = c.benchmark_group("ledger_post");
    for n in SIZES {
        group.throughput(Throughput::Elements(n));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || (1..=n).map(make_entry).collect::<Vec<_>>(),
                |entries| {
                    let mut ledger = Ledger::new();
                    for entry in entries {
                        ledger.post(entry).expect("key không trùng");
                    }
                    ledger
                },
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

/// Số dư một tài khoản: quét toàn sổ (O(n)) so với tra bảng cộng dồn (O(1)).
fn balance(c: &mut Criterion) {
    let mut group = c.benchmark_group("so_du_driver_DR_001");
    let driver = AccountId::new("driver:DR-001").expect("hợp lệ");
    for n in SIZES {
        let mut book = IndexedLedger::new();
        for i in 1..=n {
            book.post(make_entry(i)).expect("key không trùng");
        }
        group.bench_with_input(BenchmarkId::new("quet_toan_so", n), &book, |b, book| {
            b.iter(|| book.ledger().balance(black_box(&driver)));
        });
        group.bench_with_input(BenchmarkId::new("cong_don_hashmap", n), &book, |b, book| {
            b.iter(|| book.balance(black_box(&driver)));
        });
    }
    group.finish();
}

/// Thu mọi posting vào `Vec`: để vector tự tăng so với `with_capacity`.
fn flatten(c: &mut Criterion) {
    let mut group = c.benchmark_group("thu_posting_vao_vec");
    for n in SIZES {
        let ledger = make_ledger(n);
        group.throughput(Throughput::Elements(3 * n));
        group.bench_with_input(BenchmarkId::new("vec_new", n), &ledger, |b, l| {
            b.iter(|| flatten_grow(black_box(l)));
        });
        group.bench_with_input(BenchmarkId::new("with_capacity", n), &ledger, |b, l| {
            b.iter(|| flatten_with_capacity(black_box(l)));
        });
        let known = usize::try_from(3 * n).expect("vừa usize");
        group.bench_with_input(
            BenchmarkId::new("capacity_biet_truoc", n),
            &ledger,
            |b, l| {
                b.iter(|| flatten_with_known_capacity(black_box(l), black_box(known)));
            },
        );
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(20)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3));
    targets = create_entry, post, balance, flatten
}
criterion_main!(benches);
