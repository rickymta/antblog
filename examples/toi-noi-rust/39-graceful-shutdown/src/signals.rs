//! Bắt SIGTERM và ctrl-c (SIGINT) bằng `tokio::signal`.

use std::io;

use tokio::signal::unix::{Signal, SignalKind, signal};

/// Hai luồng tín hiệu đã được ĐĂNG KÝ ngay khi gọi `install`.
pub struct Signals {
    term: Signal,
    int: Signal,
}

impl Signals {
    /// Đăng ký handler. Phải gọi TRƯỚC khi báo "sẵn sàng": `tokio::signal::ctrl_c()` chỉ cài
    /// handler khi future được poll lần đầu, tín hiệu đến sớm hơn sẽ giết tiến trình theo mặc định.
    ///
    /// # Errors
    /// Lỗi hệ điều hành khi đăng ký.
    pub fn install() -> io::Result<Signals> {
        Ok(Signals {
            term: signal(SignalKind::terminate())?,
            // SIGINT là tín hiệu của ctrl-c trên Unix.
            int: signal(SignalKind::interrupt())?,
        })
    }

    /// Chờ tín hiệu kế tiếp, trả tên của nó.
    pub async fn recv(&mut self) -> &'static str {
        tokio::select! {
            _ = self.term.recv() => "SIGTERM",
            _ = self.int.recv() => "SIGINT",
        }
    }
}
