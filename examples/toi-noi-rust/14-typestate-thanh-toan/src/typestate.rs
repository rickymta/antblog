//! Cách 2: typestate. Trạng thái là tham số kiểu `S` của `Payment<S>`:
//! method nào có ở trạng thái nào do compiler quyết định lúc biên dịch.

use std::marker::PhantomData;

use crate::ledger::Ledger;
use crate::record::PaymentRecord;
use crate::settle::{PaymentTerms, capture_entry, refund_entry};
use crate::state::{PaymentError, PaymentState};

mod sealed {
    // Trait "niêm phong": crate khác không thêm được trạng thái mới.
    pub trait Sealed {}
}

/// Trạng thái ở cấp kiểu. `KIND` là cầu nối sang giá trị `enum` khi cần lưu hoặc in.
pub trait State: sealed::Sealed {
    const KIND: PaymentState;
}

/// Trạng thái còn hủy (`fail`) được: chỉ `Initiated` và `Authorized`.
pub trait Cancellable: State {}

// Marker: struct rỗng, không có dữ liệu, chỉ để làm tham số kiểu.
macro_rules! marker {
    ($name:ident, $kind:expr) => {
        #[derive(Debug)]
        pub struct $name;
        impl sealed::Sealed for $name {}
        impl State for $name {
            const KIND: PaymentState = $kind;
        }
    };
}

marker!(Initiated, PaymentState::Initiated);
marker!(Authorized, PaymentState::Authorized);
marker!(Captured, PaymentState::Captured);
marker!(Failed, PaymentState::Failed);
marker!(RefundPending, PaymentState::RefundPending);
marker!(Refunded, PaymentState::Refunded);

impl Cancellable for Initiated {}
impl Cancellable for Authorized {}

/// Khoản thanh toán ở trạng thái `S`. `PhantomData<S>` không chiếm byte nào,
/// nhưng làm `Payment<Initiated>` và `Payment<Captured>` thành hai kiểu khác nhau.
/// Không derive `Clone`: mỗi lần chuyển trạng thái tiêu thụ `self`, giá trị cũ không dùng lại được.
#[derive(Debug)]
pub struct Payment<S> {
    terms: PaymentTerms,
    _state: PhantomData<S>,
}

impl<S: State> Payment<S> {
    fn from_terms(terms: PaymentTerms) -> Payment<S> {
        Payment {
            terms,
            _state: PhantomData,
        }
    }

    /// Đổi sang trạng thái khác: chỉ gọi nội bộ từ các method hợp lệ bên dưới.
    fn into_state<T: State>(self) -> Payment<T> {
        Payment::from_terms(self.terms)
    }

    pub fn terms(&self) -> &PaymentTerms {
        &self.terms
    }

    /// Giá trị `enum` tương ứng kiểu `S`, đọc từ hằng `S::KIND`.
    pub fn state(&self) -> PaymentState {
        S::KIND
    }

    /// Xuống biên: kiểu `S` biến mất, thay bằng giá trị `PaymentState` để lưu.
    pub fn to_record(&self) -> PaymentRecord {
        PaymentRecord::restore(self.terms.clone(), S::KIND)
    }
}

impl Payment<Initiated> {
    pub fn new(terms: PaymentTerms) -> Payment<Initiated> {
        Payment::from_terms(terms)
    }

    /// Giữ tiền (hold). Nhận `self` theo giá trị: `Payment<Initiated>` không còn dùng được sau lệnh này.
    pub fn authorize(self) -> Payment<Authorized> {
        self.into_state()
    }
}

impl Payment<Authorized> {
    /// Thu tiền và ghi bút toán `trip:<mã chuyến>:capture`. Trả thêm mã bút toán.
    /// `self` bị tiêu thụ cả khi lỗi: muốn thử lại phải nạp lại từ bản ghi.
    /// Lỗi xác định (ví dụ `IdempotencyConflict`) thì thử lại vẫn ra đúng lỗi đó.
    ///
    /// Gọi `capture` trên `Payment<Initiated>` không biên dịch được (E0599):
    ///
    /// ```compile_fail,E0599
    /// use ledger_lab::{Ledger, Money, Payment, PaymentTerms};
    ///
    /// let terms = PaymentTerms::new(
    ///     "TN-0001", "customer:CU-001", "driver:DR-001",
    ///     Money::from_dong(25_500), Money::from_dong(5_100),
    /// ).unwrap();
    /// let mut ledger = Ledger::new();
    /// let payment = Payment::new(terms);
    /// let _ = payment.capture(&mut ledger);
    /// ```
    ///
    /// Thu hai lần cũng không biên dịch được (E0382), vì lần đầu đã tiêu thụ `payment`:
    ///
    /// ```compile_fail,E0382
    /// use ledger_lab::{Ledger, Money, Payment, PaymentTerms};
    ///
    /// let terms = PaymentTerms::new(
    ///     "TN-0001", "customer:CU-001", "driver:DR-001",
    ///     Money::from_dong(25_500), Money::from_dong(5_100),
    /// ).unwrap();
    /// let mut ledger = Ledger::new();
    /// let payment = Payment::new(terms).authorize();
    /// let _first = payment.capture(&mut ledger);
    /// let _second = payment.capture(&mut ledger);
    /// ```
    pub fn capture(self, ledger: &mut Ledger) -> Result<(Payment<Captured>, String), PaymentError> {
        let entry_id = ledger.post(capture_entry(&self.terms)?)?.id;
        Ok((self.into_state(), entry_id))
    }
}

impl Payment<Captured> {
    /// Khách yêu cầu hoàn tiền: chưa có bút toán, chỉ chờ VíXanh xác nhận.
    pub fn refund(self) -> Payment<RefundPending> {
        self.into_state()
    }
}

impl Payment<RefundPending> {
    /// VíXanh đã hoàn: ghi bút toán đảo `trip:<mã chuyến>:refund`.
    pub fn settle_refund(
        self,
        ledger: &mut Ledger,
    ) -> Result<(Payment<Refunded>, String), PaymentError> {
        let entry_id = ledger.post(refund_entry(&self.terms)?)?.id;
        Ok((self.into_state(), entry_id))
    }
}

// Một `impl` chung cho mọi trạng thái còn hủy được; `Captured` hay `Refunded` không có `fail`.
impl<S: Cancellable> Payment<S> {
    pub fn fail(self) -> Payment<Failed> {
        self.into_state()
    }
}

/// Nạp bản ghi thành `Payment<S>` khi người gọi đã biết mình cần trạng thái nào.
impl<S: State> TryFrom<PaymentRecord> for Payment<S> {
    type Error = PaymentError;

    fn try_from(record: PaymentRecord) -> Result<Payment<S>, PaymentError> {
        if record.state() != S::KIND {
            return Err(PaymentError::WrongState {
                expected: S::KIND,
                found: record.state(),
            });
        }
        Ok(Payment::from_terms(record.into_terms()))
    }
}

/// Ở biên (đọc từ database) chưa biết trạng thái là gì: `enum` bọc đủ sáu kiểu.
#[derive(Debug)]
pub enum AnyPayment {
    Initiated(Payment<Initiated>),
    Authorized(Payment<Authorized>),
    Captured(Payment<Captured>),
    Failed(Payment<Failed>),
    RefundPending(Payment<RefundPending>),
    Refunded(Payment<Refunded>),
}

impl AnyPayment {
    pub fn state(&self) -> PaymentState {
        match self {
            AnyPayment::Initiated(_) => PaymentState::Initiated,
            AnyPayment::Authorized(_) => PaymentState::Authorized,
            AnyPayment::Captured(_) => PaymentState::Captured,
            AnyPayment::Failed(_) => PaymentState::Failed,
            AnyPayment::RefundPending(_) => PaymentState::RefundPending,
            AnyPayment::Refunded(_) => PaymentState::Refunded,
        }
    }
}

/// Chiều "lên" từ giá trị `enum` sang kiểu: một `match` trên `record.state()`.
impl From<PaymentRecord> for AnyPayment {
    fn from(record: PaymentRecord) -> AnyPayment {
        let terms = record.terms().clone();
        match record.state() {
            PaymentState::Initiated => AnyPayment::Initiated(Payment::from_terms(terms)),
            PaymentState::Authorized => AnyPayment::Authorized(Payment::from_terms(terms)),
            PaymentState::Captured => AnyPayment::Captured(Payment::from_terms(terms)),
            PaymentState::Failed => AnyPayment::Failed(Payment::from_terms(terms)),
            PaymentState::RefundPending => AnyPayment::RefundPending(Payment::from_terms(terms)),
            PaymentState::Refunded => AnyPayment::Refunded(Payment::from_terms(terms)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::AccountId;
    use crate::entry::{JournalEntry, Posting};
    use crate::error::LedgerError;
    use crate::money::Money;
    use std::mem::size_of;

    fn terms() -> PaymentTerms {
        PaymentTerms::new(
            "TN-0001",
            "customer:CU-001",
            "driver:DR-001",
            Money::from_dong(25_500),
            Money::from_dong(5_100),
        )
        .unwrap()
    }

    #[test]
    fn full_flow_posts_two_entries_and_zeroes_balances() -> Result<(), PaymentError> {
        let mut ledger = Ledger::new();
        let authorized = Payment::new(terms()).authorize();
        let (captured, id1) = authorized.capture(&mut ledger)?;
        assert_eq!(id1, "JE-000001");
        let driver = AccountId::new("driver:DR-001").unwrap();
        assert_eq!(ledger.balance(&driver), Ok(Money::from_dong(20_400)));

        let (refunded, id2) = captured.refund().settle_refund(&mut ledger)?;
        assert_eq!(id2, "JE-000002");
        assert_eq!(refunded.state(), PaymentState::Refunded);
        assert_eq!(ledger.balance(&driver), Ok(Money::ZERO));
        assert_eq!(ledger.grand_total(), Ok(Money::ZERO));
        Ok(())
    }

    #[test]
    fn fail_works_from_initiated_and_authorized() {
        let a = Payment::new(terms()).fail();
        let b = Payment::new(terms()).authorize().fail();
        assert_eq!(a.state(), PaymentState::Failed);
        assert_eq!(b.state(), PaymentState::Failed);
    }

    #[test]
    fn marker_costs_zero_bytes() {
        assert_eq!(size_of::<Payment<Initiated>>(), size_of::<PaymentTerms>());
        assert_eq!(
            size_of::<Payment<Captured>>(),
            size_of::<Payment<Initiated>>()
        );
        assert_eq!(size_of::<Initiated>(), 0);
    }

    #[test]
    fn record_round_trip_keeps_state() {
        let record = Payment::new(terms()).authorize().to_record();
        assert_eq!(record.state(), PaymentState::Authorized);
        let back = Payment::<Authorized>::try_from(record.clone()).unwrap();
        assert_eq!(back.to_record(), record);
    }

    #[test]
    fn loading_with_wrong_expectation_fails() {
        let record = Payment::new(terms()).authorize().to_record();
        let err = Payment::<Captured>::try_from(record).unwrap_err();
        assert_eq!(
            err,
            PaymentError::WrongState {
                expected: PaymentState::Captured,
                found: PaymentState::Authorized
            }
        );
    }

    #[test]
    fn any_payment_matches_every_stored_state() {
        for state in PaymentState::ALL {
            let any = AnyPayment::from(PaymentRecord::restore(terms(), state));
            assert_eq!(any.state(), state);
        }
    }

    #[test]
    fn capture_error_consumes_value_and_retry_repeats_same_error() -> Result<(), PaymentError> {
        let mut ledger = Ledger::new();
        // Key capture đã bị chiếm bởi bút toán khác nội dung: lỗi xác định, không phải lỗi tạm thời.
        ledger.post(JournalEntry::try_new(
            "trip:TN-0001:capture",
            vec![
                Posting::new(AccountId::new("a")?, Money::from_dong(-1)),
                Posting::new(AccountId::new("b")?, Money::from_dong(1)),
            ],
        )?)?;
        let record = Payment::new(terms()).authorize().to_record();
        let err = Payment::<Authorized>::try_from(record.clone())?
            .capture(&mut ledger)
            .unwrap_err();
        assert!(matches!(
            err,
            PaymentError::Ledger(LedgerError::IdempotencyConflict { .. })
        ));
        // Giá trị `Payment<Authorized>` đã mất; bản ghi vẫn còn và vẫn là AUTHORIZED.
        assert_eq!(record.state(), PaymentState::Authorized);
        let again = Payment::<Authorized>::try_from(record)?.capture(&mut ledger);
        assert_eq!(again.err(), Some(err));
        assert_eq!(ledger.len(), 1);
        Ok(())
    }

    #[test]
    fn reload_and_retry_capture_is_idempotent() -> Result<(), PaymentError> {
        let mut ledger = Ledger::new();
        let record = Payment::new(terms()).authorize().to_record();
        // Lần đầu thành công nhưng giả sử DB chưa kịp cập nhật trạng thái: nạp lại, thu lần nữa.
        let first = Payment::<Authorized>::try_from(record.clone())?;
        first.capture(&mut ledger)?;
        let retry = Payment::<Authorized>::try_from(record)?;
        let (_, id) = retry.capture(&mut ledger)?;
        assert_eq!(id, "JE-000001");
        assert_eq!(ledger.len(), 1);
        Ok(())
    }
}
