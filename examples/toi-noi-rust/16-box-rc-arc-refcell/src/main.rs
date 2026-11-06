//! Binary: chạy lần lượt các ví dụ smart pointer, hoặc `--verify`, hoặc `--panic-demo`.

use std::process::ExitCode;
use std::rc::Rc;

use ledger_lab::boxed::{self, FareExpr};
use ledger_lab::policy::{self, PercentFee};
use ledger_lab::shared::{self, MeteredLedger};
use ledger_lab::threads;
use ledger_lab::tree::{self, AccountNode};
use ledger_lab::{AccountId, Ledger, LedgerError, Money};

fn acct(raw: &str) -> Result<AccountId, LedgerError> {
    AccountId::new(raw)
}

fn run() -> Result<(), LedgerError> {
    println!("== Box: stack và heap ==");
    for (name, bytes) in boxed::sizes() {
        println!("  size_of::<{name}>() = {bytes}");
    }
    let fare = boxed::tn_0001_fare();
    println!(
        "  cước TN-0001 = {} (cây sâu {})",
        fare.eval()?,
        fare.depth()
    );
    let capped = FareExpr::min(fare, FareExpr::Fixed(Money::from_dong(20_000)));
    println!("  cước có trần 20.000 = {}", capped.eval()?);

    println!("== Box<dyn FeePolicy> ==");
    let policies = policy::default_policies();
    for (name, fee) in policy::quote_all(&policies, Money::from_dong(25_500))? {
        println!("  {name:<18}{fee:>12}");
    }

    println!("== Rc và Weak: cây tài khoản ==");
    let root = AccountNode::new_root(acct("platform")?);
    let revenue = AccountNode::add_child(&root, acct("platform:revenue")?);
    let _fee = AccountNode::add_child(&root, acct("platform:fee")?);
    println!(
        "  root: strong = {}, weak = {}",
        Rc::strong_count(&root),
        Rc::weak_count(&root)
    );
    println!("  revenue: strong = {}", Rc::strong_count(&revenue));
    println!("  đường đi: {}", revenue.path());
    println!(
        "  chu trình Rc mạnh bị rò: {}; cây có Weak được giải phóng: {}",
        tree::strong_cycle_leaks(),
        tree::weak_parent_tree_is_freed()?
    );

    println!("== RefCell ==");
    let shared_ledger = shared::new_shared();
    let id = shared::post_fixed(&shared_ledger, shared::sample_entry()?)?;
    println!("  post_fixed: {id}");
    let busy = {
        let _reader = shared_ledger.borrow();
        shared::post_try(&shared_ledger, shared::sample_entry()?)
    };
    println!("  post_try khi đang có người đọc: {busy:?}");
    match shared::buggy_panic_message()? {
        Some(msg) => println!("  post_buggy panic: {msg}"),
        None => println!("  post_buggy không panic (không mong đợi)"),
    }

    println!("== Cell ==");
    let mut ledger = Ledger::new();
    ledger.post(shared::sample_entry()?)?;
    let metered = MeteredLedger::new(ledger);
    let driver = acct("driver:DR-001")?;
    let balance = metered.balance(&driver)?;
    metered.balance(&driver)?;
    println!(
        "  số dư {driver} = {balance}, số lần tra cứu = {}",
        metered.lookups()
    );

    println!("== Arc<Mutex<Ledger>> với 4 luồng ==");
    let ledger = threads::capture_in_threads(4)?;
    println!("  số bút toán = {}", ledger.len());
    println!(
        "  platform:revenue = {}",
        ledger.balance(&acct("platform:revenue")?)?
    );
    println!("  tổng toàn sổ = {}", ledger.grand_total()?);
    Ok(())
}

/// Kiểm các giá trị kỳ vọng in trong bài; trả danh sách lỗi (rỗng nếu đúng).
fn verify() -> Result<Vec<String>, LedgerError> {
    let mut failures = Vec::new();
    let mut check = |ok: bool, name: &str| {
        if !ok {
            failures.push(name.to_string());
        }
    };

    // Box
    let sizes = boxed::sizes();
    let size_of = |name: &str| sizes.iter().find(|(n, _)| *n == name).map(|(_, b)| *b);
    check(size_of("Money") == Some(8), "Money 8 byte");
    check(
        size_of("RawStatement") == Some(4_096),
        "RawStatement 4096 byte",
    );
    check(
        size_of("Box<RawStatement>") == Some(8),
        "Box<RawStatement> 8 byte",
    );
    check(
        size_of("Box<dyn FeePolicy>") == Some(16),
        "Box<dyn FeePolicy> 16 byte",
    );
    check(size_of("Cell<i64>") == Some(8), "Cell<i64> 8 byte");
    check(
        size_of("RefCell<i64>") == Some(16),
        "RefCell<i64> 16 byte (thêm cờ mượn)",
    );
    check(
        boxed::tn_0001_fare().eval() == Ok(Money::from_dong(25_500)),
        "cước TN-0001 25.500",
    );

    // Box<dyn Trait>
    let quotes = policy::quote_all(&policy::default_policies(), Money::from_dong(25_500))?;
    let fees: Vec<i64> = quotes.iter().map(|(_, m)| m.dong()).collect();
    check(fees == vec![5_100, 3_000, 4_000], "ba chính sách phí");
    let entry = policy::capture_entry(
        "trip:TN-0001:capture",
        "customer:CU-001",
        "driver:DR-001",
        Money::from_dong(25_500),
        &PercentFee { percent: 20 },
    )?;
    let amounts: Vec<i64> = entry.postings().iter().map(|p| p.amount.dong()).collect();
    check(amounts == vec![-25_500, 5_100, 20_400], "bút toán thu cước");

    // Rc / Weak
    let root = AccountNode::new_root(acct("platform")?);
    let revenue = AccountNode::add_child(&root, acct("platform:revenue")?);
    let _fee = AccountNode::add_child(&root, acct("platform:fee")?);
    check(Rc::strong_count(&root) == 1, "strong(root) = 1");
    check(Rc::weak_count(&root) == 2, "weak(root) = 2");
    check(Rc::strong_count(&revenue) == 2, "strong(revenue) = 2");
    check(
        revenue.path() == "platform > platform:revenue",
        "đường đi nút revenue",
    );
    check(tree::strong_cycle_leaks(), "chu trình Rc mạnh bị rò");
    check(
        tree::weak_parent_tree_is_freed()?,
        "cây Weak được giải phóng",
    );

    // RefCell
    let message = shared::buggy_panic_message()?;
    // Chỉ so phần ổn định: chữ đầy đủ ("RefCell already borrowed" ở Rust 1.98) có thể đổi theo phiên bản.
    check(
        message
            .as_deref()
            .is_some_and(|m| m.contains("already borrowed")),
        "post_buggy panic already borrowed",
    );
    let shared_ledger = shared::new_shared();
    check(
        shared::post_fixed(&shared_ledger, shared::sample_entry()?)?
            == "JE-000001 (trước đó có 0 bút toán)",
        "post_fixed ghi JE-000001",
    );
    let busy = {
        let _reader = shared_ledger.borrow();
        shared::post_try(&shared_ledger, shared::sample_entry()?)
    };
    check(busy == Err(LedgerError::Busy), "post_try báo Busy");

    // Cell
    let mut ledger = Ledger::new();
    ledger.post(shared::sample_entry()?)?;
    let metered = MeteredLedger::new(ledger);
    let driver = acct("driver:DR-001")?;
    check(
        metered.balance(&driver)? == Money::from_dong(20_400),
        "số dư tài xế 20.400",
    );
    metered.balance(&driver)?;
    check(metered.lookups() == 2, "Cell đếm 2 lần tra cứu");

    // Arc<Mutex>
    let ledger = threads::capture_in_threads(4)?;
    check(ledger.len() == 4, "4 bút toán từ 4 luồng");
    check(
        ledger.balance(&acct("platform:revenue")?)? == Money::from_dong(20_400),
        "platform:revenue 20.400",
    );
    check(ledger.grand_total()? == Money::ZERO, "tổng toàn sổ bằng 0");
    Ok(failures)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--panic-demo") {
        // Panic THẬT, không bắt: dùng để xem thông điệp và mã thoát 101.
        let shared = shared::new_shared();
        let entry = shared::sample_entry().expect("bút toán mẫu cân");
        let _ = shared::post_buggy(&shared, entry);
        return ExitCode::SUCCESS;
    }
    if args.iter().any(|a| a == "--verify") {
        return match verify() {
            Ok(failures) if failures.is_empty() => {
                println!("VERIFY OK");
                ExitCode::SUCCESS
            }
            Ok(failures) => {
                for f in &failures {
                    eprintln!("VERIFY FAIL: {f}");
                }
                ExitCode::FAILURE
            }
            Err(e) => {
                eprintln!("VERIFY FAIL: lỗi khi chạy kiểm tra: {e}");
                ExitCode::FAILURE
            }
        };
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Lỗi: {e}");
            ExitCode::FAILURE
        }
    }
}
