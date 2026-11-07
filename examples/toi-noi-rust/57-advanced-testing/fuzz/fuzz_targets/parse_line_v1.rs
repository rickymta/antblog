#![no_main]
//! Fuzz bản đọc số tiền ngây thơ (v1): chỉ hỏi «có panic không?».

use ledger_57_advanced_testing::statement::parse_line_v1;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Sao kê là UTF-8; byte không hợp lệ không phải việc của parser dòng.
    let Ok(line) = std::str::from_utf8(data) else {
        return;
    };
    let _ = parse_line_v1(line);
});
