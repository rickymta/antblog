//! `ledger-cli`: chạy kịch bản chuyến TN-0001 trên thư viện `ledger-core`.
//!
//! `cargo run` in kịch bản; `cargo run -- --verify` kiểm các giá trị kỳ vọng.

use std::error::Error;
use std::process::ExitCode;

use ledger_core::{
    AccountId, FeePolicy, JournalEntry, Ledger, LedgerError, Money, PaymentEvent, PaymentState,
    Posting, TripCharge, capture_entry, default_policy, fare_for_km, transition,
};

const CUSTOMER: &str = "customer:CU-001";
const DRIVER: &str = "driver:DR-001";
const PLATFORM: &str = "platform:revenue";

/// Chuyến TN-0001 đi 5 km, khách CU-001, tài xế DR-001.
fn trip_0001() -> Result<TripCharge, LedgerError> {
    Ok(TripCharge {
        trip_id: "TN-0001".to_string(),
        customer: AccountId::new(CUSTOMER)?,
        driver: AccountId::new(DRIVER)?,
        fare: fare_for_km(5),
    })
}

fn print_balances(ledger: &Ledger) -> Result<(), LedgerError> {
    for (account, balance) in ledger.balances()? {
        println!("  {account:<18}{balance:>12}");
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    // Chính sách là dyn FeePolicy: đổi sang chính sách khác không phải sửa dòng nào bên dưới.
    let policy: Box<dyn FeePolicy> = Box::new(default_policy());
    let trip = trip_0001()?;
    let split = policy.split(trip.fare)?;
    println!("Cước TN-0001 (5 km): {}", trip.fare);
    println!("Phí nền tảng 20%: {}", split.platform_fee);
    println!("Tài xế DR-001 nhận: {}", split.driver_net);

    let state = transition(PaymentState::Initiated, PaymentEvent::Authorize)?;
    let state = transition(state, PaymentEvent::Capture)?;
    println!("Thanh toán: INITIATED -> AUTHORIZED -> {state}");

    let mut ledger = Ledger::new();
    let entry = capture_entry(&trip, policy.as_ref())?;
    println!("JSON: {}", serde_json::to_string(&entry)?);
    let posted = ledger.post(entry)?;
    let id = posted.id;
    println!(
        "{} {} ({} posting, replayed = {})",
        posted.id,
        posted.entry.idempotency_key(),
        posted.entry.postings().len(),
        posted.replayed
    );

    // Ghi lần hai cùng key: không tạo bản sao.
    let again = ledger.post(capture_entry(&trip, policy.as_ref())?)?;
    println!("{} ghi lại: replayed = {}", again.id, again.replayed);
    println!("Số dư sau thu cước:");
    print_balances(&ledger)?;

    // Sửa sai bằng bút toán đảo, không sửa bút toán gốc.
    let reversal = ledger.reverse(id)?;
    println!(
        "{} {} (đảo {id})",
        reversal.id,
        reversal.entry.idempotency_key()
    );
    println!("Số dư sau bút toán đảo:");
    print_balances(&ledger)?;
    println!(
        "Số bút toán: {}, tổng toàn sổ: {}",
        ledger.len(),
        ledger.grand_total()?
    );
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
    let dong = Money::from_dong;

    let trip = trip_0001().expect("mã tài khoản mẫu hợp lệ");
    check(trip.fare == dong(25_500), "cước 5 km = 25.500");
    let policy = default_policy();
    let split = policy.split(trip.fare).expect("tách cước");
    check(split.platform_fee == dong(5_100), "phí nền tảng 5.100");
    check(split.driver_net == dong(20_400), "tài xế 20.400");

    let mut ledger = Ledger::new();
    let entry = capture_entry(&trip, &policy).expect("bút toán mẫu cân");
    let posted = ledger.post(entry).expect("ghi lần đầu");
    let id = posted.id;
    check(posted.id.to_string() == "JE-000001", "mã JE-000001");
    check(!posted.replayed, "lần đầu không phải replay");
    let again = ledger
        .post(capture_entry(&trip, &policy).expect("bút toán mẫu cân"))
        .expect("ghi lại cùng key");
    check(again.replayed, "lần hai là replay");
    check(ledger.len() == 1, "chỉ một bút toán sau replay");

    let balance_of = |ledger: &Ledger, raw: &str| {
        ledger
            .balance(&AccountId::new(raw).expect("mã hợp lệ"))
            .expect("không tràn")
    };
    check(
        balance_of(&ledger, CUSTOMER) == dong(-25_500),
        "khách -25.500",
    );
    check(
        balance_of(&ledger, PLATFORM) == dong(5_100),
        "nền tảng 5.100",
    );
    check(balance_of(&ledger, DRIVER) == dong(20_400), "tài xế 20.400");

    let reversal = ledger.reverse(id).expect("đảo bút toán");
    check(
        reversal.id.to_string() == "JE-000002",
        "mã bút toán đảo JE-000002",
    );
    check(
        reversal.entry.reverses() == Some(id),
        "bút toán đảo trỏ về JE-000001",
    );
    check(ledger.len() == 2, "sổ có 2 bút toán sau khi đảo");
    for raw in [CUSTOMER, PLATFORM, DRIVER] {
        check(
            balance_of(&ledger, raw) == Money::ZERO,
            "số dư về 0 sau khi đảo",
        );
    }
    check(
        ledger.grand_total() == Ok(Money::ZERO),
        "tổng toàn sổ bằng 0",
    );
    check(
        ledger.reverse(id).is_ok_and(|p| p.replayed) && ledger.len() == 2,
        "đảo lần hai chỉ là replay",
    );

    let unbalanced = JournalEntry::try_new(
        "k1",
        vec![
            Posting::new(AccountId::new("a").expect("mã hợp lệ"), dong(-100)),
            Posting::new(AccountId::new("b").expect("mã hợp lệ"), dong(90)),
        ],
    );
    check(
        unbalanced == Err(LedgerError::Unbalanced { sum: dong(-10) }),
        "bút toán lệch 10 đồng bị từ chối",
    );
    check(
        transition(PaymentState::Initiated, PaymentEvent::Capture).is_err(),
        "INITIATED -> CAPTURED bị từ chối",
    );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_has_no_failures() {
        assert_eq!(verify(), Vec::<String>::new());
    }

    #[test]
    fn scenario_runs_to_the_end() {
        assert!(run().is_ok());
    }
}
