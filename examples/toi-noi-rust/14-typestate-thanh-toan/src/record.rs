//! Bản ghi thanh toán dạng "dữ liệu": đúng hình dạng một dòng trong database.
//! Trạng thái là giá trị `PaymentState`, kiểm tra hợp lệ ở lúc chạy qua `apply`.

use crate::ledger::Ledger;
use crate::settle::{PaymentTerms, capture_entry, refund_entry};
use crate::state::{PaymentError, PaymentEvent, PaymentState, transition};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentRecord {
    terms: PaymentTerms,
    // Private: đổi trạng thái chỉ qua `apply`, không gán trực tiếp từ ngoài.
    state: PaymentState,
}

impl PaymentRecord {
    /// Bản ghi mới luôn bắt đầu ở `INITIATED`.
    pub fn new(terms: PaymentTerms) -> PaymentRecord {
        PaymentRecord {
            terms,
            state: PaymentState::Initiated,
        }
    }

    /// Dựng lại từ database: tin rằng `state` đã được lưu đúng.
    pub fn restore(terms: PaymentTerms, state: PaymentState) -> PaymentRecord {
        PaymentRecord { terms, state }
    }

    pub fn state(&self) -> PaymentState {
        self.state
    }

    pub fn terms(&self) -> &PaymentTerms {
        &self.terms
    }

    pub(crate) fn into_terms(self) -> PaymentTerms {
        self.terms
    }

    /// Áp một sự kiện. Với `Capture` và `CompleteRefund` còn ghi bút toán vào sổ cái;
    /// trả mã bút toán (`JE-xxxxxx`) nếu có. Lỗi thì trạng thái giữ nguyên.
    pub fn apply(
        &mut self,
        event: PaymentEvent,
        ledger: &mut Ledger,
    ) -> Result<Option<String>, PaymentError> {
        let next = transition(self.state, event)?;
        // Nhớ ghi sổ cho đúng sự kiện là việc của người viết nhánh này; compiler không nhắc.
        let entry_id = match event {
            PaymentEvent::Capture => Some(ledger.post(capture_entry(&self.terms)?)?.id),
            PaymentEvent::CompleteRefund => Some(ledger.post(refund_entry(&self.terms)?)?.id),
            _ => None,
        };
        self.state = next;
        Ok(entry_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::AccountId;
    use crate::money::Money;

    fn record() -> PaymentRecord {
        PaymentRecord::new(
            PaymentTerms::new(
                "TN-0001",
                "customer:CU-001",
                "driver:DR-001",
                Money::from_dong(25_500),
                Money::from_dong(5_100),
            )
            .unwrap(),
        )
    }

    #[test]
    fn happy_path_posts_capture_and_refund() {
        let mut ledger = Ledger::new();
        let mut rec = record();
        assert_eq!(rec.apply(PaymentEvent::Authorize, &mut ledger), Ok(None));
        assert_eq!(
            rec.apply(PaymentEvent::Capture, &mut ledger),
            Ok(Some("JE-000001".to_string()))
        );
        assert_eq!(
            rec.apply(PaymentEvent::RequestRefund, &mut ledger),
            Ok(None)
        );
        assert_eq!(
            rec.apply(PaymentEvent::CompleteRefund, &mut ledger),
            Ok(Some("JE-000002".to_string()))
        );
        assert_eq!(rec.state(), PaymentState::Refunded);
        assert_eq!(ledger.grand_total(), Ok(Money::ZERO));
        let driver = AccountId::new("driver:DR-001").unwrap();
        assert_eq!(ledger.balance(&driver), Ok(Money::ZERO));
    }

    #[test]
    fn capture_before_authorize_is_rejected_and_state_kept() {
        let mut ledger = Ledger::new();
        let mut rec = record();
        let err = rec.apply(PaymentEvent::Capture, &mut ledger).unwrap_err();
        assert_eq!(
            err,
            PaymentError::InvalidTransition {
                from: PaymentState::Initiated,
                event: PaymentEvent::Capture
            }
        );
        assert_eq!(rec.state(), PaymentState::Initiated);
        assert!(ledger.is_empty());
    }

    #[test]
    fn capture_twice_is_rejected_at_runtime() {
        let mut ledger = Ledger::new();
        let mut rec = record();
        rec.apply(PaymentEvent::Authorize, &mut ledger).unwrap();
        rec.apply(PaymentEvent::Capture, &mut ledger).unwrap();
        assert!(rec.apply(PaymentEvent::Capture, &mut ledger).is_err());
        assert_eq!(ledger.len(), 1);
    }

    #[test]
    fn failed_payment_never_touches_ledger() {
        let mut ledger = Ledger::new();
        let mut rec = record();
        rec.apply(PaymentEvent::Authorize, &mut ledger).unwrap();
        rec.apply(PaymentEvent::Fail, &mut ledger).unwrap();
        assert_eq!(rec.state(), PaymentState::Failed);
        assert!(ledger.is_empty());
    }
}
