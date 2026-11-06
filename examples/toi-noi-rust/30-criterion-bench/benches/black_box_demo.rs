//! Benchmark SAI và ĐÚNG cạnh nhau: compiler bỏ code mà kết quả không ai dùng.

use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};

/// Tổng 1.000 số tiền mẫu, sinh ngay lúc biên dịch.
const fn sample() -> [i64; 1_000] {
    let mut a = [0i64; 1_000];
    let mut i = 0;
    while i < 1_000 {
        a[i] = (i as i64 % 7) * 100 - 300;
        i += 1;
    }
    a
}
const AMOUNTS: [i64; 1_000] = sample();

fn sum_amounts(amounts: &[i64]) -> i64 {
    amounts.iter().sum()
}

fn black_box_demo(c: &mut Criterion) {
    let mut group = c.benchmark_group("tong_1000_so_tien");

    // SAI 1: kết quả bị bỏ, compiler thấy hàm không có tác dụng phụ và xóa cả lời gọi.
    group.bench_function("SAI_bo_ket_qua", |b| {
        b.iter(|| {
            sum_amounts(&AMOUNTS);
        });
    });

    // Trả về kết quả nhưng đầu vào là hằng: compiler CÓ THỂ tính sẵn. Với 1.000 phần tử
    // LLVM không làm vậy (số đo bên dưới), nên `black_box` là bảo hiểm, không phải phép màu.
    group.bench_function("tra_ket_qua_dau_vao_hang", |b| {
        b.iter(|| sum_amounts(&AMOUNTS))
    });

    // ĐÚNG: `black_box` che đầu vào khỏi trình tối ưu; criterion tự `black_box` kết quả trả về.
    group.bench_function("dung_black_box_dau_vao", |b| {
        b.iter(|| sum_amounts(black_box(&AMOUNTS)));
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(30)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(2));
    targets = black_box_demo
}
criterion_main!(benches);
