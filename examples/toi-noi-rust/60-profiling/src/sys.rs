//! Đọc load average của máy để ghi kèm số đo thời gian.

use std::process::Command;

/// Load average 1/5/15 phút theo `uptime`, hoặc `"không đọc được"`.
pub fn load_average() -> String {
    let out = Command::new("uptime").output();
    let Ok(out) = out else {
        return "không đọc được".to_string();
    };
    let text = String::from_utf8_lossy(&out.stdout);
    // macOS in "load averages:", Linux in "load average:".
    match text.split("load average").nth(1) {
        Some(rest) => rest
            .trim_start_matches('s')
            .trim_start_matches(':')
            .trim()
            .to_string(),
        None => "không đọc được".to_string(),
    }
}
