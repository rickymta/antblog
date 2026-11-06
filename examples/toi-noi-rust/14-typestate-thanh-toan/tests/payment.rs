//! Integration test: chỉ dùng API public. Hai cách mô hình hóa phải cho cùng một sổ cái.

use ledger_lab::{
    AccountId, Ledger, Money, Payment, PaymentError, PaymentEvent, PaymentRecord, PaymentState,
    PaymentTerms,
};

fn terms() -> PaymentTerms {
    PaymentTerms::new(
        "TN-0001",
        "customer:CU-001",
        "driver:DR-001",
        Money::from_dong(25_500),
        Money::from_dong(5_100),
    )
    .expect("dữ kiện hợp lệ")
}

fn balances(ledger: &Ledger) -> Vec<Money> {
    ["customer:CU-001", "platform:revenue", "driver:DR-001"]
        .iter()
        .map(|raw| ledger.balance(&AccountId::new(raw).expect("mã hợp lệ")))
        .collect::<Result<_, _>>()
        .expect("không tràn")
}

#[test]
fn enum_and_typestate_produce_identical_ledgers() -> Result<(), PaymentError> {
    let mut by_enum = Ledger::new();
    let mut record = PaymentRecord::new(terms());
    for event in [
        PaymentEvent::Authorize,
        PaymentEvent::Capture,
        PaymentEvent::RequestRefund,
        PaymentEvent::CompleteRefund,
    ] {
        record.apply(event, &mut by_enum)?;
    }

    let mut by_type = Ledger::new();
    let (captured, _) = Payment::new(terms()).authorize().capture(&mut by_type)?;
    captured.refund().settle_refund(&mut by_type)?;

    assert_eq!(by_enum.len(), by_type.len());
    assert_eq!(balances(&by_enum), balances(&by_type));
    assert_eq!(record.state(), PaymentState::Refunded);
    Ok(())
}

#[test]
fn failed_payment_leaves_no_trace_in_ledger() {
    let mut ledger = Ledger::new();
    let payment = Payment::new(terms()).authorize().fail();
    assert_eq!(payment.state(), PaymentState::Failed);
    assert!(ledger.is_empty());
    // Bản ghi nạp lại từ DB ở trạng thái FAILED không nhận thêm sự kiện nào.
    let mut record = payment.to_record();
    assert!(record.apply(PaymentEvent::Authorize, &mut ledger).is_err());
}
