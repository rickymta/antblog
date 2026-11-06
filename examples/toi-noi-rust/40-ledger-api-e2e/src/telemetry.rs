//! Log JSON: mỗi event một dòng, kèm các span đang mở (nơi `request_id` nằm). Rút gọn từ bài 29.

use tracing_subscriber::EnvFilter;

/// Cài subscriber toàn cục. `to_stderr` dành cho chế độ demo để stdout chỉ có kết quả.
///
/// # Errors
/// Bộ lọc sai cú pháp hoặc đã có subscriber toàn cục.
pub fn init(filter: &str, to_stderr: bool) -> Result<(), String> {
    let filter = EnvFilter::try_new(filter).map_err(|e| e.to_string())?;
    let builder = tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .with_current_span(true)
        .with_span_list(false);
    let result = if to_stderr {
        builder.with_writer(std::io::stderr).try_init()
    } else {
        builder.try_init()
    };
    result.map_err(|e| e.to_string())
}
