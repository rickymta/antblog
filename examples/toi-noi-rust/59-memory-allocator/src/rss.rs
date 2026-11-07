//! RSS (resident set size) của chính tiến trình, đọc bằng `ps` để không cần `unsafe` hay crate ngoài.
//! Hoạt động trên macOS và Linux; không đọc được thì trả `None` (không đoán).

use std::process::Command;

/// RSS hiện tại, đơn vị KiB.
pub fn current_kib() -> Option<u64> {
    let out = Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;
    String::from_utf8(out.stdout).ok()?.trim().parse().ok()
}
