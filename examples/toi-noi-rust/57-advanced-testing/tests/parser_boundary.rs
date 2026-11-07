//! Test thêm sau lượt `cargo mutants` thứ hai (trên `src/statement.rs`): biên độ dài ref và số 0.

use ledger_57_advanced_testing::statement::{ParseError, format_amount, parse_line};

fn line_with_ref(external_ref: &str) -> String {
    format!("{external_ref},25500,2026-11-05T03:00:12Z")
}

#[test]
fn ref_length_limit_is_64_bytes() {
    // Diệt mutant `> 64` thành `>= 64` và `== 64`.
    assert!(parse_line(&line_with_ref(&"a".repeat(64))).is_ok());
    assert_eq!(
        parse_line(&line_with_ref(&"a".repeat(65))),
        Err(ParseError::BadRef)
    );
}

#[test]
fn short_ref_with_bad_character_is_rejected() {
    // Diệt mutant `||` thành `&&`: ref ngắn nhưng có ký tự cấm vẫn phải bị từ chối.
    assert_eq!(parse_line(&line_with_ref("vx-ch")), Err(ParseError::BadRef));
}

#[test]
fn zero_is_formatted_without_a_sign() {
    // Diệt mutant `amount < 0` thành `amount <= 0` (sẽ in «-0»).
    assert_eq!(format_amount(0), "0");
}
