//! Binary crate: dùng thư viện như người ngoài, và tự định nghĩa thêm một chính sách phí.

use std::mem::size_of;
use std::process::ExitCode;

use ledger_lab::{
    AccountId, FeePolicy, FlatFee, HalfAwayFromZero, Ledger, LedgerError, Money, NearestEven,
    PercentageFee, TripCharge, capture_entry, default_policy, fare_for_km,
};
use rust_decimal::Decimal;

/// Chính sách thứ hai, nằm ngoài thư viện: phí cố định theo bậc cước.
/// Chỉ cần `impl FeePolicy`; `capture_entry` không bị sửa dòng nào.
struct TieredFee;

impl FeePolicy for TieredFee {
    fn name(&self) -> String {
        "cố định theo bậc".to_string()
    }

    fn fee(&self, fare: Money) -> Result<Money, LedgerError> {
        let dong = match fare.dong() {
            ..20_000 => 3_000,
            20_000..50_000 => 4_500,
            _ => 8_000,
        };
        Ok(Money::from_dong(dong))
    }
}

fn sample_trip() -> Result<TripCharge, LedgerError> {
    Ok(TripCharge {
        trip_id: "TN-0001".to_string(),
        customer: AccountId::new("customer:CU-001")?,
        driver: AccountId::new("driver:DR-001")?,
        fare: fare_for_km(5),
    })
}

/// `impl FeePolicy` ở vị trí tham số: cú pháp gọn của hàm generic `<P: FeePolicy>`.
fn summarize(policy: &impl FeePolicy, fare: Money) -> Result<String, LedgerError> {
    let split = policy.split(fare)?;
    Ok(format!(
        "{:<22} phí {:>10}  tài xế {:>11}",
        policy.name(),
        split.platform_fee,
        split.driver_net
    ))
}

/// Tên kiểu mà compiler đã sinh bản hàm riêng cho nó (monomorphization).
fn instantiated_for<F: FeePolicy>(_policy: &F) -> &'static str {
    std::any::type_name::<F>()
}

/// Danh sách chính sách khác kiểu nhau: bắt buộc `Box<dyn FeePolicy>`.
fn policy_catalog() -> Vec<Box<dyn FeePolicy>> {
    vec![
        Box::new(default_policy()),
        Box::new(PercentageFee::new(Decimal::new(55, 3), HalfAwayFromZero)),
        Box::new(PercentageFee::new(Decimal::new(55, 3), NearestEven)),
        Box::new(FlatFee::new(Money::from_dong(3_000))),
        Box::new(TieredFee),
    ]
}

fn run() -> Result<(), LedgerError> {
    let trip = sample_trip()?;
    let mut ledger = Ledger::new();

    println!("== Chính sách mặc định: {}", default_policy().name());
    let posted = ledger.post(capture_entry(&trip, &default_policy())?)?;
    println!(
        "{} {} ({} posting, replayed = {})",
        posted.id,
        posted.entry.idempotency_key(),
        posted.entry.postings().len(),
        posted.replayed
    );
    for raw in ["customer:CU-001", "platform:revenue", "driver:DR-001"] {
        let balance = ledger.balance(&AccountId::new(raw)?)?;
        println!("  {raw:<18}{balance:>12}");
    }
    println!("Tổng toàn sổ: {}", ledger.grand_total()?);

    println!();
    println!("== Cùng cước {}, thay chính sách phí", trip.fare);
    for policy in policy_catalog() {
        // `policy` là `Box<dyn FeePolicy>`: gọi phương thức qua vtable.
        let split = policy.split(trip.fare)?;
        println!(
            "  {:<22} phí {:>10}  tài xế {:>11}",
            policy.name(),
            split.platform_fee,
            split.driver_net
        );
    }

    println!();
    println!("== Static dispatch và dynamic dispatch");
    println!("{}", summarize(&TieredFee, trip.fare)?);
    println!("hàm generic được sinh bản cho:");
    println!("  {}", instantiated_for(&default_policy()));
    println!(
        "  {}",
        instantiated_for(&FlatFee::new(Money::from_dong(3_000)))
    );
    println!(
        "kích thước con trỏ: &FlatFee = {} byte, &dyn FeePolicy = {} byte, Box<dyn FeePolicy> = {} byte",
        size_of::<&FlatFee>(),
        size_of::<&dyn FeePolicy>(),
        size_of::<Box<dyn FeePolicy>>()
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

    let trip = sample_trip().expect("chuyến mẫu hợp lệ");
    check(trip.fare == dong(25_500), "cước 5 km là 25.500");

    // Chính sách mặc định: 5.100 / 20.400, ghi vào sổ thành JE-000001.
    let mut ledger = Ledger::new();
    let entry = capture_entry(&trip, &default_policy()).expect("bút toán cân");
    let posted = ledger.post(entry).expect("ghi lần đầu");
    check(posted.id == "JE-000001", "mã bút toán JE-000001");
    check(
        posted.entry.idempotency_key() == "trip:TN-0001:capture",
        "idempotency_key của chuyến",
    );
    let balance_of = |raw: &str| {
        ledger
            .balance(&AccountId::new(raw).expect("mã hợp lệ"))
            .expect("không tràn")
    };
    check(
        balance_of("customer:CU-001") == dong(-25_500),
        "khách -25.500",
    );
    check(
        balance_of("platform:revenue") == dong(5_100),
        "nền tảng 5.100",
    );
    check(balance_of("driver:DR-001") == dong(20_400), "tài xế 20.400");
    check(
        ledger.grand_total() == Ok(Money::ZERO),
        "tổng toàn sổ bằng 0",
    );

    // Thay chính sách: cùng capture_entry, kết quả khác, bút toán vẫn cân.
    let fees: Vec<Money> = policy_catalog()
        .iter()
        .map(|p| p.split(trip.fare).expect("chính sách hợp lệ").platform_fee)
        .collect();
    check(
        fees == [
            dong(5_100),
            dong(1_403),
            dong(1_402),
            dong(3_000),
            dong(4_500),
        ],
        "phí của năm chính sách trong danh mục",
    );
    for policy in policy_catalog() {
        let entry = capture_entry(&trip, &*policy);
        check(entry.is_ok(), "mọi chính sách dựng được bút toán cân");
    }
    let tiered = capture_entry(&trip, &TieredFee).expect("bút toán cân");
    check(
        tiered.postings()[2].amount == dong(21_000),
        "phí theo bậc: tài xế 21.000",
    );

    // Hai kiểu con trỏ: thin pointer và fat pointer (con trỏ dữ liệu + con trỏ vtable).
    check(
        size_of::<&FlatFee>() == size_of::<usize>(),
        "&FlatFee là 1 word",
    );
    check(
        size_of::<&dyn FeePolicy>() == 2 * size_of::<usize>(),
        "&dyn FeePolicy là 2 word",
    );
    check(
        size_of::<Box<dyn FeePolicy>>() == 2 * size_of::<usize>(),
        "Box<dyn FeePolicy> là 2 word",
    );
    check(
        instantiated_for(&default_policy())
            .contains("PercentageFee<ledger_lab::policy::HalfAwayFromZero>"),
        "bản generic cho PercentageFee<HalfAwayFromZero>",
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
