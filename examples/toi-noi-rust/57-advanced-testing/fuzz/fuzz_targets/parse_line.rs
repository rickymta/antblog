#![no_main]
//! Fuzz parser đã sửa: không panic, và dòng đã chấp nhận phải đọc lại ra đúng nó (round-trip).

use ledger_57_advanced_testing::statement::parse_line;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(line) = std::str::from_utf8(data) else {
        return;
    };
    if let Ok(row) = parse_line(line) {
        let again = parse_line(&row.to_line());
        assert_eq!(
            again,
            Ok(row),
            "dòng chấp nhận được phải đọc lại đúng: {line:?}"
        );
    }
});
