//! Lỗi nghiệp vụ tách khỏi lỗi hạ tầng.

use ledger_core::{LedgerError, Money, PaymentError};
use thiserror::Error;

/// Mọi lý do một thao tác hoàn tiền, đảo hay chargeback bị từ chối.
#[derive(Debug, Error)]
pub enum RefundError {
    /// Lỗi nghiệp vụ từ `ledger-core` (bút toán lệch, `IdempotencyConflict`...).
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    /// Lỗi của máy trạng thái thanh toán (bài 14, 20).
    #[error(transparent)]
    State(#[from] PaymentError),
    /// Lỗi từ PostgreSQL hoặc sqlx.
    #[error("lỗi database: {0}")]
    Db(#[from] sqlx::Error),
    /// Không có chuyến này trong bảng `payments`.
    #[error("không có thanh toán của chuyến {0}")]
    UnknownTrip(String),
    /// Thanh toán đang ở trạng thái không cho thao tác này.
    #[error("thanh toán đang {status}, không thao tác được")]
    PaymentNotRefundable {
        /// Trạng thái hiện tại.
        status: String,
    },
    /// Yêu cầu vượt phần còn hoàn được (`đã thu - đã hoàn`).
    #[error("yêu cầu hoàn {requested} vượt phần còn hoàn được {refundable}")]
    ExceedsRefundable {
        /// Số tiền yêu cầu.
        requested: Money,
        /// Phần còn hoàn được tại thời điểm kiểm.
        refundable: Money,
    },
    /// Người duyệt trùng người tạo.
    #[error("{actor} vừa tạo vừa duyệt: maker-checker không cho phép")]
    SelfApproval {
        /// Người thực hiện.
        actor: String,
    },
    /// Không có yêu cầu hoàn với mã này.
    #[error("không có yêu cầu hoàn tiền {0}")]
    RequestNotFound(i64),
    /// Yêu cầu hoặc chargeback đã được quyết định theo hướng khác.
    #[error("{what} đã ở trạng thái {status}")]
    AlreadyDecided {
        /// Đối tượng, ví dụ `yêu cầu 2`.
        what: String,
        /// Trạng thái hiện tại.
        status: String,
    },
    /// Bút toán này đã có bút toán đảo.
    #[error("bút toán {0} đã được đảo")]
    AlreadyReversed(i64),
    /// Không đảo bút toán đảo.
    #[error("bút toán {0} là bút toán đảo, không đảo tiếp")]
    CannotReverseReversal(i64),
    /// Đảo toàn phần chỉ cho phép khi chưa hoàn đồng nào.
    #[error("chuyến đã hoàn một phần {refunded}, không đảo toàn phần được")]
    PartialRefundExists {
        /// Số đã hoàn.
        refunded: Money,
    },
    /// Reason code không đúng loại (refund dùng mã chargeback hoặc ngược lại).
    #[error("reason code {0} không dùng cho thao tác này")]
    WrongReasonKind(String),
    /// Chargeback lớn hơn số đã thu.
    #[error("chargeback {amount} lớn hơn số đã thu {captured}")]
    ChargebackExceedsCaptured {
        /// Số PSP đòi.
        amount: Money,
        /// Số đã thu của chuyến.
        captured: Money,
    },
    /// Số tiền không dương.
    #[error("số tiền {0} phải lớn hơn 0")]
    NonPositiveAmount(Money),
    /// Không có chargeback với mã vụ việc này.
    #[error("không có chargeback {0}")]
    UnknownChargeback(String),
}

impl RefundError {
    /// Tên constraint bị vi phạm nếu đây là lỗi ràng buộc từ PostgreSQL.
    pub fn constraint(&self) -> Option<String> {
        match self {
            RefundError::Db(e) => e
                .as_database_error()
                .and_then(|d| d.constraint())
                .map(str::to_string),
            _ => None,
        }
    }

    /// SQLSTATE nếu đây là lỗi từ server PostgreSQL (ví dụ `23514` là CHECK).
    pub fn sqlstate(&self) -> Option<String> {
        match self {
            RefundError::Db(e) => e
                .as_database_error()
                .and_then(|d| d.code())
                .map(|c| c.to_string()),
            _ => None,
        }
    }
}
