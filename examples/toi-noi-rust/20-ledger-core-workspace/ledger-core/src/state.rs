//! Trạng thái thanh toán, sự kiện làm đổi trạng thái và bảng chuyển hợp lệ.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Sáu trạng thái thanh toán của một chuyến (§9.1 kiến trúc Tới Nơi).
///
/// Trạng thái đến từ bên ngoài (sự kiện Kafka, cột `state` trong database) nên được
/// biểu diễn bằng enum lúc chạy. Phiên bản typestate `Payment<S>` ở bài 14 hợp khi cả
/// chuỗi bước nằm trong code của ta; ở đây enum là đủ.
///
/// ```
/// use ledger_core::{PaymentEvent, PaymentState, transition};
///
/// let state = transition(PaymentState::Initiated, PaymentEvent::Authorize)?;
/// let state = transition(state, PaymentEvent::Capture)?;
/// assert_eq!(state.to_string(), "CAPTURED");
/// // Không được nhảy cóc từ INITIATED sang CAPTURED.
/// assert!(transition(PaymentState::Initiated, PaymentEvent::Capture).is_err());
/// # Ok::<(), ledger_core::PaymentError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentState {
    /// Đã tạo yêu cầu thanh toán.
    Initiated,
    /// Cổng thanh toán đã giữ tiền (hold).
    Authorized,
    /// Đã thu tiền thật.
    Captured,
    /// Thất bại, trạng thái cuối.
    Failed,
    /// Đang chờ hoàn tiền.
    RefundPending,
    /// Đã hoàn tiền, trạng thái cuối.
    Refunded,
}

impl PaymentState {
    /// Cả sáu trạng thái, dùng để duyệt bảng chuyển trong test.
    pub const ALL: [PaymentState; 6] = [
        PaymentState::Initiated,
        PaymentState::Authorized,
        PaymentState::Captured,
        PaymentState::Failed,
        PaymentState::RefundPending,
        PaymentState::Refunded,
    ];

    /// Chuỗi lưu trong cột `state` của database.
    pub const fn as_str(self) -> &'static str {
        match self {
            PaymentState::Initiated => "INITIATED",
            PaymentState::Authorized => "AUTHORIZED",
            PaymentState::Captured => "CAPTURED",
            PaymentState::Failed => "FAILED",
            PaymentState::RefundPending => "REFUND_PENDING",
            PaymentState::Refunded => "REFUNDED",
        }
    }
}

impl fmt::Display for PaymentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// Đọc từ chuỗi trong database: chuỗi lạ là lỗi, không đoán.
impl FromStr for PaymentState {
    type Err = PaymentError;

    fn from_str(raw: &str) -> Result<PaymentState, PaymentError> {
        PaymentState::ALL
            .into_iter()
            .find(|s| s.as_str() == raw)
            .ok_or_else(|| PaymentError::UnknownState(raw.to_string()))
    }
}

/// Sự kiện làm đổi trạng thái.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentEvent {
    /// Cổng thanh toán giữ tiền.
    Authorize,
    /// Thu tiền thật.
    Capture,
    /// Thất bại.
    Fail,
    /// Yêu cầu hoàn tiền.
    RequestRefund,
    /// Hoàn tiền xong.
    CompleteRefund,
}

impl PaymentEvent {
    /// Cả năm sự kiện, dùng để duyệt bảng chuyển trong test.
    pub const ALL: [PaymentEvent; 5] = [
        PaymentEvent::Authorize,
        PaymentEvent::Capture,
        PaymentEvent::Fail,
        PaymentEvent::RequestRefund,
        PaymentEvent::CompleteRefund,
    ];
}

/// Lỗi của máy trạng thái thanh toán.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PaymentError {
    /// Cặp (trạng thái, sự kiện) không có cạnh nào.
    #[error("không thể {event:?} khi đang ở {from}")]
    InvalidTransition {
        /// Trạng thái hiện tại.
        from: PaymentState,
        /// Sự kiện bị từ chối.
        event: PaymentEvent,
    },
    /// Chuỗi trạng thái đọc từ database không thuộc sáu giá trị.
    #[error("trạng thái lạ trong database: {0:?}")]
    UnknownState(String),
}

/// Bảng chuyển trạng thái. Cạnh nào không có trong `match` đều rơi vào nhánh cuối và bị từ chối.
///
/// # Errors
/// [`PaymentError::InvalidTransition`] nếu `event` không áp dụng được cho `from`.
pub fn transition(from: PaymentState, event: PaymentEvent) -> Result<PaymentState, PaymentError> {
    use PaymentEvent::{Authorize, Capture, CompleteRefund, Fail, RequestRefund};
    use PaymentState::{Authorized, Captured, Failed, Initiated, RefundPending, Refunded};
    match (from, event) {
        (Initiated, Authorize) => Ok(Authorized),
        (Authorized, Capture) => Ok(Captured),
        (Initiated | Authorized, Fail) => Ok(Failed),
        (Captured, RequestRefund) => Ok(RefundPending),
        (RefundPending, CompleteRefund) => Ok(Refunded),
        (from, event) => Err(PaymentError::InvalidTransition { from, event }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path_reaches_refunded() -> Result<(), PaymentError> {
        let mut state = PaymentState::Initiated;
        for event in [
            PaymentEvent::Authorize,
            PaymentEvent::Capture,
            PaymentEvent::RequestRefund,
            PaymentEvent::CompleteRefund,
        ] {
            state = transition(state, event)?;
        }
        assert_eq!(state, PaymentState::Refunded);
        Ok(())
    }

    // Duyệt đủ 6 x 5 = 30 cặp: đúng 6 cạnh hợp lệ, 24 cặp còn lại bị từ chối.
    #[test]
    fn exactly_six_edges_are_valid() {
        let valid = PaymentState::ALL
            .into_iter()
            .flat_map(|s| PaymentEvent::ALL.into_iter().map(move |e| (s, e)))
            .filter(|&(s, e)| transition(s, e).is_ok())
            .count();
        assert_eq!(valid, 6);
    }

    #[test]
    fn terminal_states_go_nowhere() {
        for event in PaymentEvent::ALL {
            assert!(transition(PaymentState::Failed, event).is_err());
            assert!(transition(PaymentState::Refunded, event).is_err());
        }
    }

    #[test]
    fn from_str_roundtrips_and_rejects_unknown() {
        for state in PaymentState::ALL {
            assert_eq!(state.as_str().parse::<PaymentState>(), Ok(state));
        }
        assert_eq!(
            "captured".parse::<PaymentState>(),
            Err(PaymentError::UnknownState("captured".to_string()))
        );
    }

    #[test]
    fn serde_uses_screaming_snake_case() {
        let json = serde_json::to_string(&PaymentState::RefundPending).unwrap();
        assert_eq!(json, "\"REFUND_PENDING\"");
        let back: PaymentState = serde_json::from_str("\"CAPTURED\"").unwrap();
        assert_eq!(back, PaymentState::Captured);
    }

    #[test]
    fn error_message_is_readable() {
        let err = transition(PaymentState::Initiated, PaymentEvent::Capture).unwrap_err();
        assert_eq!(err.to_string(), "không thể Capture khi đang ở INITIATED");
    }
}
