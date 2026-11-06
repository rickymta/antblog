//! Bài 03: kiểu số và tiền VND. Không float cho tiền, `i64` đơn vị đồng,
//! `rust_decimal` chỉ cho tỷ lệ phí.

use std::hint::black_box;
use std::process::ExitCode;

use rust_decimal::prelude::*;

/// Phí nền tảng của một khoản cước (đơn vị đồng), tỷ lệ cho dưới dạng Decimal.
/// Trả `None` nếu phép nhân tràn hoặc kết quả không vừa `i64`.
fn platform_fee(fare: i64, rate: Decimal) -> Option<i64> {
    let raw = Decimal::from(fare).checked_mul(rate)?;
    // Làm tròn về đồng: nửa đồng làm tròn ra xa số 0 (ROUND_HALF_UP).
    let rounded = raw.round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero);
    rounded.to_i64()
}

/// Phí 20% của Tới Nơi.
fn fee(fare: i64) -> Option<i64> {
    platform_fee(fare, Decimal::new(20, 2)) // 20 * 10^-2 = 0.20
}

/// Phần tài xế nhận = cước - phí, dùng `checked_sub`.
fn driver_share(fare: i64) -> Option<i64> {
    fare.checked_sub(fee(fare)?)
}

fn show_integer_types() {
    println!("== Kiểu số nguyên ==");
    println!("i8   : {} .. {}", i8::MIN, i8::MAX);
    println!("u8   : {} .. {}", u8::MIN, u8::MAX);
    println!("i32  : {} .. {}", i32::MIN, i32::MAX);
    println!("i64  : {} .. {}", i64::MIN, i64::MAX);
    println!("i128 : {} .. {}", i128::MIN, i128::MAX);
    println!("usize: {} byte trên máy này", size_of::<usize>());

    // Suy luận kiểu và hậu tố; `_` chỉ để dễ đọc.
    let fare = 25_500; // không có gì ép kiểu, mặc định i32
    let fare_vnd = 25_500i64; // hậu tố i64
    let fare_typed: i64 = 25_500; // chú thích kiểu
    // Lỗi E0308 nếu bỏ comment (i64 + i32 không tự ép kiểu):
    // let km: i32 = 5;
    // let total = fare_vnd + km;
    println!(
        "25_500 mặc định: {} byte; hậu tố i64: {} byte; chú thích i64: {} byte",
        size_of_val(&fare),
        size_of_val(&fare_vnd),
        size_of_val(&fare_typed)
    );
}

fn show_float_trap() {
    println!("== f64 không an toàn cho tiền ==");
    let a = 0.1_f64;
    let b = 0.2_f64;
    let sum = a + b;
    println!("0.1 + 0.2 = {sum}");
    println!("0.1 + 0.2 == 0.3 ? {}", sum == 0.3);
    println!("0.1 + 0.2 với 20 chữ số: {sum:.20}");
    println!("0.1 * 3.0 = {}", 0.1_f64 * 3.0);
}

fn show_checked_arithmetic() {
    println!("== Phép toán an toàn trên i64 ==");
    println!("i64::MAX.checked_add(1)    = {:?}", i64::MAX.checked_add(1));
    println!(
        "25_500i64.checked_add(100) = {:?}",
        25_500i64.checked_add(100)
    );
    println!("i64::MAX.checked_mul(2)    = {:?}", i64::MAX.checked_mul(2));
    println!(
        "i64::MAX.saturating_add(1) = {}",
        i64::MAX.saturating_add(1)
    );
    println!("i64::MAX.wrapping_add(1)   = {}", i64::MAX.wrapping_add(1));
    println!(
        "i64::MAX.overflowing_add(1) = {:?}",
        i64::MAX.overflowing_add(1)
    );

    // Tổng lớn: cộng trong i128 rồi mới quyết định có đủ chỗ trong i64 không.
    let big = i128::from(i64::MAX) + i128::from(i64::MAX);
    println!("i64::MAX + i64::MAX trong i128 = {big}");
    println!("i64::try_from(big) = {:?}", i64::try_from(big));
}

fn show_casts() {
    println!("== as và TryFrom ==");
    let fare: i64 = 25_500;
    let large: i64 = 5_000_000_000;
    println!("25_500i64 as i32      = {}", fare as i32);
    println!("5_000_000_000i64 as i32 = {}", large as i32);
    println!("300i64 as u8          = {}", 300i64 as u8);
    println!("-1i64 as u64          = {}", -1i64 as u64);
    println!("1e20 as i64           = {}", 1e20_f64 as i64);
    println!(
        "i32::try_from(25_500i64)         = {:?}",
        i32::try_from(fare)
    );
    println!(
        "i32::try_from(5_000_000_000i64)  = {:?}",
        i32::try_from(large)
    );
    let converted: Result<u8, _> = 300i64.try_into();
    println!("300i64.try_into::<u8>()          = {converted:?}");
}

fn show_decimal_fee() {
    println!("== Phí nền tảng bằng rust_decimal ==");
    for fare in [25_500i64, 25_503] {
        let raw = Decimal::from(fare) * Decimal::new(20, 2);
        println!("20% của {fare} = {raw} -> làm tròn {:?}", fee(fare));
    }
    println!("Tài xế nhận từ 25_500: {:?}", driver_share(25_500));

    // Ca nửa đồng: so hai chiến lược làm tròn.
    let rate = Decimal::new(5, 2); // 5%
    let raw = Decimal::from(25_530i64) * rate;
    let away = raw.round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero);
    let even = raw.round_dp_with_strategy(0, RoundingStrategy::MidpointNearestEven);
    println!("5% của 25_530 = {raw}; ra xa số 0 -> {away}; về số chẵn -> {even}");
}

/// Chỉ chạy khi gọi `--overflow`: debug build panic, release build wrap.
fn show_overflow() {
    println!("== Tràn số i32 (build debug hay release?) ==");
    println!("debug_assertions = {}", cfg!(debug_assertions));
    // Viết thẳng `i32::MAX + 1` thì rustc từ chối ngay lúc biên dịch:
    // error: this arithmetic operation will overflow (arithmetic_overflow).
    // black_box che giá trị khỏi compiler để thấy hành vi lúc chạy.
    let max = black_box(i32::MAX);
    let one = black_box(1);
    let total = max + one;
    println!("i32::MAX + 1 = {total}");
}

fn verify() -> ExitCode {
    // 1. Float không biểu diễn chính xác.
    assert!(black_box(0.1_f64) + black_box(0.2_f64) != 0.3);
    // 2. Phép toán có kiểm tra trả None thay vì tràn.
    assert_eq!(i64::MAX.checked_add(1), None);
    assert_eq!(i64::MAX.saturating_add(1), i64::MAX);
    assert_eq!(i32::MAX.wrapping_add(1), i32::MIN);
    // 3. Ép kiểu: as cắt bit, try_from báo lỗi.
    assert_eq!(300i64 as u8, 44);
    assert!(u8::try_from(300i64).is_err());
    assert_eq!(i64::try_from(i128::from(i64::MAX) * 2).ok(), None);
    // 4. Phí 20% bằng Decimal rồi về i64.
    assert_eq!(fee(25_500), Some(5_100));
    assert_eq!(fee(25_503), Some(5_101));
    // 5. Tài xế nhận 20_400; ca bài tập 25_502 -> phí 5_100, tài xế 20_402.
    assert_eq!(driver_share(25_500), Some(20_400));
    assert_eq!(fee(25_502), Some(5_100));
    assert_eq!(driver_share(25_502), Some(20_402));
    // 6. Nửa đồng: hai chiến lược khác nhau.
    let half = Decimal::new(12765, 1); // 1276.5
    assert_eq!(
        half.round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero),
        Decimal::from(1277)
    );
    assert_eq!(
        half.round_dp_with_strategy(0, RoundingStrategy::MidpointNearestEven),
        Decimal::from(1276)
    );
    println!("VERIFY OK");
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let arg = std::env::args().nth(1);
    match arg.as_deref() {
        Some("--verify") => return verify(),
        Some("--overflow") => {
            show_overflow();
            return ExitCode::SUCCESS;
        }
        _ => {}
    }
    show_integer_types();
    show_float_trap();
    show_checked_arithmetic();
    show_casts();
    show_decimal_fee();
    ExitCode::SUCCESS
}
