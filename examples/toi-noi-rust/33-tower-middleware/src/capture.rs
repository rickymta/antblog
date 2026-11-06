//! Bắt log của `tracing` vào bộ nhớ để kiểm tra nội dung log.
//!
//! Một subscriber toàn cục, ghi vào bộ đệm theo thread. Bản đầu dùng `set_default` riêng cho
//! từng test: `tracing` cache `Interest` của callsite cho cả process, nên khi nhiều test chạy
//! song song dưới tải cao, thỉnh thoảng span rớt khỏi log và test đỏ oan.

use std::cell::RefCell;
use std::io;
use std::sync::{Arc, Mutex, Once};

thread_local! {
    static CURRENT: RefCell<Option<LogBuf>> = const { RefCell::new(None) };
}

/// Bộ đệm log dùng chung, nhân bản rẻ.
#[derive(Clone, Default)]
pub struct LogBuf(Arc<Mutex<Vec<u8>>>);

impl LogBuf {
    /// Toàn bộ log đã ghi, dạng chuỗi.
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().expect("buf")).into_owned()
    }
}

// Writer của subscriber toàn cục: ghi vào bộ đệm của thread hiện tại, thread không bắt log thì bỏ.
struct ThreadWriter;

impl io::Write for ThreadWriter {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        CURRENT.with_borrow(|current| {
            if let Some(buf) = current {
                buf.0.lock().expect("buf").extend_from_slice(data);
            }
        });
        Ok(data.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Giữ việc bắt log tới khi bị drop.
pub struct CaptureGuard;

impl Drop for CaptureGuard {
    fn drop(&mut self) {
        CURRENT.set(None);
    }
}

/// Cài subscriber toàn cục. Mỗi test gọi hàm này ĐẦU TIÊN, trước khi chạm callsite nào: callsite gặp
/// lúc chưa có subscriber có thể bị cache `Interest::never` và span rớt khỏi log.
pub fn init() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let subscriber = tracing_subscriber::fmt()
            .with_writer(|| ThreadWriter)
            .with_ansi(false)
            .without_time()
            .with_target(false)
            .finish();
        tracing::subscriber::set_global_default(subscriber).expect("subscriber toàn cục");
    });
}

/// Bắt log của thread hiện tại. Dùng runtime `current_thread` thì mọi task cùng thread nên đều bị bắt.
pub fn capture() -> (LogBuf, CaptureGuard) {
    init();
    let buf = LogBuf::default();
    CURRENT.set(Some(buf.clone()));
    (buf, CaptureGuard)
}
