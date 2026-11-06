//! Cách 1: máy trạng thái bằng `enum`. Hợp lệ hay không do hàm `transition` quyết định lúc chạy.

use std::error::Error;
use std::fmt;
use std::str::FromStr;

use crate::error::LedgerError;
use crate::money::Money;

/// Sáu trạng thái thanh toán (§9.1 kiến trúc Tới Nơi).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaymentState {
    Initiated,
    Authorized,
    Captured,
    Failed,
    RefundPending,
    Refunded,
}

impl PaymentState {
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

/// Đọc từ chuỗi trong database: chuỗi lạ là lỗi, không đoán.
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
    Authorize,
    Capture,
    Fail,
    RequestRefund,
    CompleteRefund,
}

impl PaymentEvent {
    pub const ALL: [PaymentEvent; 5] = [
        PaymentEvent::Authorize,
        PaymentEvent::Capture,
        PaymentEvent::Fail,
        PaymentEvent::RequestRefund,
        PaymentEvent::CompleteRefund,
    ];
}

/// Lỗi của máy trạng thái thanh toán.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentError {
    /// Cặp (trạng thái, sự kiện) không có cạnh nào.
    InvalidTransition {
        from: PaymentState,
        event: PaymentEvent,
    },
    /// Chuỗi trạng thái đọc từ database không thuộc sáu giá trị.
    UnknownState(String),
    /// Nạp bản ghi thành `Payment<S>` nhưng trạng thái lưu khác `S`.
    WrongState {
        expected: PaymentState,
        found: PaymentState,
    },
    EmptyTripId,
    /// Số tiền thu phải dương.
    InvalidAmount {
        amount: Money,
    },
    /// Phí phải nằm trong 0..=số tiền.
    InvalidFee {
        fee: Money,
    },
    /// Lỗi từ sổ cái (bút toán lệch, tràn số, xung đột idempotency...).
    Ledger(LedgerError),
}

impl fmt::Display for PaymentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentError::InvalidTransition { from, event } => {
                write!(f, "không thể {event:?} khi đang ở {from}")
            }
            PaymentError::UnknownState(raw) => write!(f, "trạng thái lạ trong database: {raw:?}"),
            PaymentError::WrongState { expected, found } => {
                write!(f, "cần trạng thái {expected}, bản ghi đang ở {found}")
            }
            PaymentError::EmptyTripId => write!(f, "mã chuyến rỗng"),
            PaymentError::InvalidAmount { amount } => {
                write!(f, "số tiền thu không dương: {amount}")
            }
            PaymentError::InvalidFee { fee } => write!(f, "phí ngoài khoảng cho phép: {fee}"),
            PaymentError::Ledger(e) => write!(f, "lỗi sổ cái: {e}"),
        }
    }
}

impl Error for PaymentError {
    // Lỗi sổ cái là nguyên nhân gốc: để người gọi lần theo chuỗi `source`.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PaymentError::Ledger(e) => Some(e),
            _ => None,
        }
    }
}

impl From<LedgerError> for PaymentError {
    fn from(e: LedgerError) -> PaymentError {
        PaymentError::Ledger(e)
    }
}

/// Bảng chuyển trạng thái. Cạnh nào không có trong `match` đều rơi vào nhánh cuối và bị từ chối.
///
/// ```
/// use ledger_lab::{PaymentEvent, PaymentState, transition};
///
/// assert_eq!(
///     transition(PaymentState::Initiated, PaymentEvent::Authorize),
///     Ok(PaymentState::Authorized)
/// );
/// assert!(transition(PaymentState::Initiated, PaymentEvent::Capture).is_err());
/// ```
pub fn transition(from: PaymentState, event: PaymentEvent) -> Result<PaymentState, PaymentError> {
    use PaymentEvent::*;
    use PaymentState::*;
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
    fn exactly_six_of_thirty_pairs_are_legal() {
        let mut legal = 0;
        let mut illegal = 0;
        for from in PaymentState::ALL {
            for event in PaymentEvent::ALL {
                match transition(from, event) {
                    Ok(_) => legal += 1,
                    Err(PaymentError::InvalidTransition { .. }) => illegal += 1,
                    Err(other) => panic!("lỗi bất ngờ: {other}"),
                }
            }
        }
        assert_eq!((legal, illegal), (6, 24));
    }

    #[test]
    fn terminal_states_accept_nothing() {
        for event in PaymentEvent::ALL {
            assert!(transition(PaymentState::Failed, event).is_err());
            assert!(transition(PaymentState::Refunded, event).is_err());
        }
    }

    #[test]
    fn db_string_round_trips_and_rejects_typos() {
        for s in PaymentState::ALL {
            assert_eq!(s.as_str().parse::<PaymentState>(), Ok(s));
        }
        assert_eq!(
            "AUTHORISED".parse::<PaymentState>(),
            Err(PaymentError::UnknownState("AUTHORISED".to_string()))
        );
        assert!("captured".parse::<PaymentState>().is_err());
    }

    #[test]
    fn error_message_names_state_and_event() {
        let err = transition(PaymentState::Initiated, PaymentEvent::Capture).unwrap_err();
        assert_eq!(err.to_string(), "không thể Capture khi đang ở INITIATED");
    }

    #[test]
    fn ledger_error_is_the_source() {
        let err = PaymentError::from(LedgerError::Overflow);
        assert!(err.source().is_some());
        assert!(PaymentError::EmptyTripId.source().is_none());
    }
}
