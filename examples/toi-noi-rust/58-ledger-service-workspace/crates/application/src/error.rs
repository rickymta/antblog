//! Lỗi của ứng dụng: một enum cho mọi port và use case, tách lỗi nghiệp vụ (đừng thử lại)
//! khỏi lỗi hạ tầng (thử lại được).

use ledger_domain::LedgerError;
use ledger_domain::hold::HoldError;
use ledger_domain::refund::RefundError;
use ledger_domain::trip::TripError;
use thiserror::Error;

/// Mọi lý do một use case thất bại.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Error {
    /// Yêu cầu hoặc sự kiện sai nội dung dù sổ ở trạng thái nào.
    #[error("dữ liệu không hợp lệ: {0}")]
    Invalid(String),
    /// Bút toán vi phạm bất biến của lõi sổ cái.
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    /// Dữ liệu chuyến sai.
    #[error(transparent)]
    Trip(#[from] TripError),
    /// Lỗi nghiệp vụ của hold.
    #[error(transparent)]
    Hold(#[from] HoldError),
    /// Lỗi nghiệp vụ của hoàn tiền.
    #[error(transparent)]
    Refund(#[from] RefundError),
    /// Cùng `idempotency_key` nhưng khác nội dung bản đã ghi.
    #[error("idempotency_key {key} đã dùng cho bút toán khác nội dung")]
    IdempotencyConflict {
        /// Key bị trùng.
        key: String,
    },
    /// Tài khoản chưa mở (chưa có bút toán nào chạm tới).
    #[error("không có tài khoản {0}")]
    UnknownAccount(String),
    /// Chuyến chưa có thanh toán nào.
    #[error("không có thanh toán của chuyến {0}")]
    UnknownTrip(String),
    /// Hạ tầng tạm thời không dùng được: gửi lại cả yêu cầu là hợp lệ.
    #[error("hạ tầng tạm thời không sẵn sàng: {0}")]
    Unavailable(String),
    /// Lỗi lưu trữ không phân loại được; chi tiết chỉ nằm trong log.
    #[error("lỗi lưu trữ")]
    Storage(String),
}

impl Error {
    /// Lỗi vĩnh viễn: thử lại bao nhiêu lần cũng vẫn sai, consumer bỏ qua sự kiện thay vì kẹt.
    pub fn is_permanent(&self) -> bool {
        !matches!(self, Error::Unavailable(_) | Error::Storage(_))
    }
}
