//! Test hồi quy từ cargo-fuzz: input tối thiểu 7 byte làm bản v1 panic, cùng bảng ngữ pháp số tiền.

use std::panic::catch_unwind;

use ledger_57_advanced_testing::statement::{
    ParseError, StatementRow, format_amount, parse_amount, parse_amount_v1, parse_line,
    parse_line_v1,
};

/// Input tối thiểu do `cargo fuzz tmin` sinh ra: `_,Ώ _,` (7 byte, một ký tự UTF-8 hai byte).
const CRASH_V1: &[u8] = include_bytes!("data/crash_v1_char_boundary.bin");

fn crash_line() -> &'static str {
    std::str::from_utf8(CRASH_V1).expect("input hồi quy là UTF-8 hợp lệ")
}

#[test]
fn crash_input_is_rejected_not_panicking() {
    // Bản đã sửa: lỗi bình thường, không panic.
    assert_eq!(parse_line(crash_line()), Err(ParseError::BadAmount));
    assert_eq!(parse_amount("Ώ _"), Err(ParseError::BadAmount));
}

#[test]
fn crash_input_still_panics_the_naive_v1() {
    // Bằng chứng lỗi có thật: cùng input làm bản v1 panic vì cắt giữa ký tự UTF-8.
    let line = crash_line();
    assert!(catch_unwind(|| parse_line_v1(line)).is_err());
}

#[test]
fn naive_v1_also_accepts_a_double_minus() {
    // Lỗi thứ hai của v1, không panic nên fuzz «có panic không?» không thấy: `--5` thành +5.
    assert_eq!(parse_amount_v1("--5"), Ok(5));
    assert_eq!(parse_amount("--5"), Err(ParseError::BadAmount));
}

#[test]
fn amount_grammar_table() {
    let cases: &[(&str, Result<i64, ParseError>)] = &[
        ("25500", Ok(25_500)),
        ("25.500", Ok(25_500)),
        (" 52.000 VND ", Ok(52_000)),
        ("52.000vnd", Ok(52_000)),
        ("-22000", Ok(-22_000)),
        ("-1.200.000", Ok(-1_200_000)),
        ("9223372036854775807", Ok(i64::MAX)),
        ("-9223372036854775808", Ok(i64::MIN)),
        ("9223372036854775808", Err(ParseError::BadAmount)),
        ("0", Err(ParseError::ZeroAmount)),
        ("-0", Err(ParseError::ZeroAmount)),
        ("", Err(ParseError::BadAmount)),
        ("VND", Err(ParseError::BadAmount)),
        ("1.2", Err(ParseError::BadAmount)),
        ("1234.567", Err(ParseError::BadAmount)),
        ("12a00", Err(ParseError::BadAmount)),
        ("+5", Err(ParseError::BadAmount)),
        ("€5", Err(ParseError::BadAmount)),
    ];
    for (input, expected) in cases {
        assert_eq!(&parse_amount(input), expected, "đầu vào {input:?}");
    }
}

#[test]
fn format_then_parse_round_trips() {
    for amount in [1, -1, 999, 1_000, -25_500, 1_200_000, i64::MAX, i64::MIN] {
        let text = format_amount(amount);
        assert_eq!(parse_amount(&text), Ok(amount), "văn bản {text:?}");
    }
    assert_eq!(format_amount(25_500), "25.500");
    assert_eq!(format_amount(-1_200_000), "-1.200.000");
}

#[test]
fn seed_files_stay_valid() {
    // Các mẫu hạt giống của fuzz không được mục: bốn dòng đúng, một dòng thiếu cột.
    let ok = [
        include_str!("../fuzz/seeds/plain"),
        include_str!("../fuzz/seeds/grouped"),
        include_str!("../fuzz/seeds/suffix"),
        include_str!("../fuzz/seeds/refund"),
    ];
    for line in ok {
        let row: StatementRow = parse_line(line).expect(line);
        assert_eq!(parse_line(&row.to_line()), Ok(row));
    }
    assert_eq!(
        parse_line(include_str!("../fuzz/seeds/short")),
        Err(ParseError::FieldCount(2))
    );
}
