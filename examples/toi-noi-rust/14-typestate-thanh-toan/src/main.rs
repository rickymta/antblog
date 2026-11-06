//! Binary crate: chạy cùng một chuyến `TN-0001` qua hai cách mô hình hóa trạng thái.

use std::mem::size_of;
use std::process::ExitCode;

use ledger_lab::{
    AccountId, AnyPayment, Authorized, Initiated, Ledger, Money, Payment, PaymentError,
    PaymentEvent, PaymentRecord, PaymentState, PaymentTerms, RefundPending, transition,
};

/// Chuyến TN-0001: khách trả 25.500, phí nền tảng 5.100, tài xế nhận 20.400.
fn terms() -> Result<PaymentTerms, PaymentError> {
    PaymentTerms::new(
        "TN-0001",
        "customer:CU-001",
        "driver:DR-001",
        Money::from_dong(25_500),
        Money::from_dong(5_100),
    )
}

fn balance(ledger: &Ledger, raw: &str) -> Money {
    // Mã cố định trong lab nên `expect` an toàn; sổ nhỏ nên không thể tràn.
    let id = AccountId::new(raw).expect("mã tài khoản hợp lệ");
    ledger.balance(&id).expect("không tràn số")
}

fn run() -> Result<(), PaymentError> {
    println!("== Cách 1: enum PaymentState, kiểm tra lúc chạy ==");
    let mut ledger = Ledger::new();
    let mut record = PaymentRecord::new(terms()?);
    println!("trạng thái đầu: {}", record.state());
    // Gọi sai thứ tự: biên dịch vẫn qua, lỗi nằm ở giá trị trả về.
    match record.apply(PaymentEvent::Capture, &mut ledger) {
        Ok(_) => println!("Capture: ok (không mong đợi)"),
        Err(e) => println!("Capture khi INITIATED: Err({e})"),
    }
    for event in [PaymentEvent::Authorize, PaymentEvent::Capture] {
        let entry = record.apply(event, &mut ledger)?;
        match entry {
            Some(id) => println!("{event:?} -> {} (bút toán {id})", record.state()),
            None => println!("{event:?} -> {}", record.state()),
        }
    }
    println!(
        "Capture lần hai: Err({})",
        record
            .apply(PaymentEvent::Capture, &mut ledger)
            .expect_err("CAPTURED không nhận Capture")
    );
    println!(
        "Số bút toán: {}; tài xế: {}",
        ledger.len(),
        balance(&ledger, "driver:DR-001")
    );

    println!("\n== Cách 2: typestate Payment<S>, kiểm tra lúc biên dịch ==");
    let mut ledger = Ledger::new();
    let payment = Payment::new(terms()?);
    println!("kiểu: Payment<Initiated>, state() = {}", payment.state());
    let payment = payment.authorize();
    println!("kiểu: Payment<Authorized>, state() = {}", payment.state());
    let (payment, entry_id) = payment.capture(&mut ledger)?;
    println!(
        "kiểu: Payment<Captured>, state() = {} (bút toán {entry_id})",
        payment.state()
    );
    println!(
        "khách {}, nền tảng {}, tài xế {}",
        balance(&ledger, "customer:CU-001"),
        balance(&ledger, "platform:revenue"),
        balance(&ledger, "driver:DR-001")
    );
    // `PhantomData` không chiếm byte nào: hai kiểu cùng kích thước.
    println!(
        "size_of Payment<Initiated> = {}, Payment<Captured> = {}",
        size_of::<Payment<Initiated>>(),
        size_of::<Payment<ledger_lab::Captured>>()
    );

    println!("\n== Ở biên: lưu xuống và nạp lên ==");
    let record = payment.to_record();
    println!("lưu: state = {:?}", record.state().as_str());
    // Dòng database chỉ có chuỗi. Chuỗi lạ bị từ chối ngay ở biên.
    println!(
        "đọc \"AUTHORISED\": {:?}",
        "AUTHORISED".parse::<PaymentState>()
    );
    // Người gọi biết mình cần trạng thái nào: nạp thẳng thành Payment<Authorized>.
    let wrong = Payment::<Authorized>::try_from(record.clone());
    println!("nạp CAPTURED thành Payment<Authorized>: {:?}", wrong.err());
    // Không biết trước: nạp vào enum bọc đủ sáu kiểu rồi match.
    match AnyPayment::from(record) {
        AnyPayment::Captured(p) => {
            let p: Payment<RefundPending> = p.refund();
            let (done, id) = p.settle_refund(&mut ledger)?;
            println!("hoàn tiền -> {} (bút toán {id})", done.state());
        }
        other => println!("không hoàn được từ {}", other.state()),
    }
    println!(
        "tài xế: {}; tổng toàn sổ: {}",
        balance(&ledger, "driver:DR-001"),
        ledger.grand_total()?
    );

    // Bảng enum vẫn là nguồn sự thật cho 30 cặp (trạng thái, sự kiện).
    let legal = PaymentState::ALL
        .into_iter()
        .flat_map(|s| PaymentEvent::ALL.into_iter().map(move |e| (s, e)))
        .filter(|&(s, e)| transition(s, e).is_ok())
        .count();
    println!("\ncặp hợp lệ: {legal}/30");
    Ok(())
}

/// Kiểm các giá trị kỳ vọng in trong bài; trả danh sách lỗi (rỗng nếu đúng).
fn verify() -> Vec<String> {
    let mut failures = Vec::new();
    let mut check = |ok: bool, name: &str| {
        if !ok {
            failures.push(name.to_string());
        }
    };
    let Ok(t) = terms() else {
        return vec!["dựng PaymentTerms".to_string()];
    };

    // Cách 1: enum.
    let mut ledger = Ledger::new();
    let mut record = PaymentRecord::new(t.clone());
    check(
        matches!(
            record.apply(PaymentEvent::Capture, &mut ledger),
            Err(PaymentError::InvalidTransition {
                from: PaymentState::Initiated,
                event: PaymentEvent::Capture
            })
        ),
        "Capture khi INITIATED bị từ chối",
    );
    check(ledger.is_empty(), "từ chối thì không ghi sổ");
    check(
        record.apply(PaymentEvent::Authorize, &mut ledger) == Ok(None),
        "Authorize không ghi sổ",
    );
    check(
        record.apply(PaymentEvent::Capture, &mut ledger) == Ok(Some("JE-000001".to_string())),
        "Capture ghi JE-000001",
    );
    check(record.state() == PaymentState::Captured, "state = CAPTURED");
    check(
        record.apply(PaymentEvent::Capture, &mut ledger).is_err(),
        "Capture lần hai bị từ chối",
    );
    check(ledger.len() == 1, "chỉ một bút toán");

    // Cách 2: typestate cho cùng kết quả sổ cái.
    let mut ledger2 = Ledger::new();
    let payment = Payment::new(t.clone()).authorize();
    let Ok((payment, id)) = payment.capture(&mut ledger2) else {
        return vec!["capture typestate".to_string()];
    };
    check(id == "JE-000001", "typestate capture ghi JE-000001");
    check(
        balance(&ledger2, "customer:CU-001") == Money::from_dong(-25_500),
        "khách -25.500",
    );
    check(
        balance(&ledger2, "platform:revenue") == Money::from_dong(5_100),
        "nền tảng 5.100",
    );
    check(
        balance(&ledger2, "driver:DR-001") == Money::from_dong(20_400),
        "tài xế 20.400",
    );
    check(
        size_of::<Payment<Initiated>>() == size_of::<Payment<ledger_lab::Captured>>(),
        "PhantomData không tốn byte",
    );

    // Biên enum <-> typestate.
    let stored = payment.to_record();
    check(stored.state() == PaymentState::Captured, "lưu CAPTURED");
    check(
        stored.state().as_str() == "CAPTURED",
        "chuỗi database là CAPTURED",
    );
    check(
        "AUTHORISED".parse::<PaymentState>()
            == Err(PaymentError::UnknownState("AUTHORISED".into())),
        "chuỗi lạ bị từ chối",
    );
    check(
        matches!(
            Payment::<Authorized>::try_from(stored.clone()),
            Err(PaymentError::WrongState {
                expected: PaymentState::Authorized,
                found: PaymentState::Captured
            })
        ),
        "nạp sai trạng thái bị từ chối",
    );
    if let AnyPayment::Captured(p) = AnyPayment::from(stored) {
        match p.refund().settle_refund(&mut ledger2) {
            Ok((_, id)) => check(id == "JE-000002", "hoàn tiền ghi JE-000002"),
            Err(_) => check(false, "settle_refund"),
        }
    } else {
        check(false, "AnyPayment::Captured");
    }
    check(
        balance(&ledger2, "driver:DR-001") == Money::ZERO,
        "tài xế về 0 sau hoàn tiền",
    );
    check(
        ledger2.grand_total() == Ok(Money::ZERO),
        "tổng toàn sổ bằng 0",
    );

    let legal = PaymentState::ALL
        .into_iter()
        .flat_map(|s| PaymentEvent::ALL.into_iter().map(move |e| (s, e)))
        .filter(|&(s, e)| transition(s, e).is_ok())
        .count();
    check(legal == 6, "6/30 cặp hợp lệ");
    failures
}

fn main() -> ExitCode {
    if std::env::args().any(|arg| arg == "--verify") {
        let failures = verify();
        if failures.is_empty() {
            println!("VERIFY OK");
            return ExitCode::SUCCESS;
        }
        for f in &failures {
            eprintln!("VERIFY FAIL: {f}");
        }
        return ExitCode::FAILURE;
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Lỗi: {e}");
            ExitCode::FAILURE
        }
    }
}
