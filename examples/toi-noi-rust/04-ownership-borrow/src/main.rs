//! Bài 04: ownership, move và borrow trên sổ cái Tới Nơi (chỉ dùng std).

use std::process::ExitCode;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Số lần `Tracked::drop` đã chạy, để --verify kiểm quy tắc "ra scope thì drop".
static DROPS: AtomicUsize = AtomicUsize::new(0);

/// Tiền VND, đơn vị đồng. `Copy` vì chỉ là một `i64` nằm trọn trên stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Money(i64);

/// Một dòng ghi: số tiền có dấu trên một tài khoản. Sở hữu `String` nên không `Copy`.
#[derive(Debug)]
struct Posting {
    account: String,
    amount: Money,
}

/// Bút toán: tổng số tiền có dấu của các posting bằng 0.
#[derive(Debug)]
struct JournalEntry {
    id: String,
    idempotency_key: String,
    postings: Vec<Posting>,
}

/// Sổ cái sở hữu `Vec<JournalEntry>`; bút toán đã vào sổ thuộc về `Ledger`.
#[derive(Debug, Default)]
struct Ledger {
    entries: Vec<JournalEntry>,
}

/// Giá trị có `Drop` để thấy thời điểm nó bị giải phóng.
struct Tracked(String);

impl Drop for Tracked {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
        println!("  drop {}", self.0);
    }
}

/// Bút toán JE-000001: VíXanh thu 25.500 của khách cho chuyến TN-0001.
fn capture_entry() -> JournalEntry {
    JournalEntry {
        id: String::from("JE-000001"),
        idempotency_key: String::from("trip:TN-0001:capture"),
        postings: vec![
            Posting {
                account: String::from("psp:vixanh:clearing"),
                amount: Money(25_500),
            },
            Posting {
                account: String::from("customer:CU-001"),
                amount: Money(-25_500),
            },
        ],
    }
}

/// Bút toán JE-000002: chia 25.500 thành 20.400 cho tài xế và 5.100 phí nền tảng (20%).
fn split_entry() -> JournalEntry {
    JournalEntry {
        id: String::from("JE-000002"),
        idempotency_key: String::from("trip:TN-0001:split"),
        postings: vec![
            Posting {
                account: String::from("psp:vixanh:clearing"),
                amount: Money(-25_500),
            },
            Posting {
                account: String::from("driver:DR-001"),
                amount: Money(20_400),
            },
            Posting {
                account: String::from("platform:revenue"),
                amount: Money(5_100),
            },
        ],
    }
}

/// Nhận ownership của `entry`: sau lời gọi, caller không còn bút toán này.
fn post(ledger: &mut Ledger, entry: JournalEntry) {
    ledger.entries.push(entry);
}

/// Chỉ mượn sổ cái (`&Ledger`) và tên tài khoản (`&str`); không lấy gì làm của mình.
fn balance(ledger: &Ledger, account: &str) -> i64 {
    let mut total: i64 = 0;
    for entry in &ledger.entries {
        for posting in &entry.postings {
            if posting.account == account {
                // Bài 07 sẽ thay expect bằng Result.
                total = total.checked_add(posting.amount.0).expect("tràn i64");
            }
        }
    }
    total
}

/// Trả tham chiếu tới bút toán cuối; lifetime được suy ra từ `&Ledger`.
fn last_entry(ledger: &Ledger) -> Option<&JournalEntry> {
    ledger.entries.last()
}

/// Nhận ownership của chuỗi; chuỗi bị drop khi hàm kết thúc.
fn take_id(id: String) -> usize {
    id.len()
}

/// Chỉ mượn chuỗi để đọc.
fn peek_id(id: &str) -> usize {
    id.len()
}

fn demo_copy_move() {
    println!("== 1. Copy và move ==");
    let a: i64 = 25_500;
    let b = a; // i64 là Copy: a vẫn dùng được
    println!("a = {a}, b = {b}");

    let fare = Money(25_500);
    let same = fare; // Money derive Copy: fare vẫn dùng được
    println!("fare = {fare:?}, same = {same:?}");

    let id = String::from("JE-000001");
    let moved = id; // String không Copy: quyền sở hữu chuyển sang `moved`
    // Lỗi E0382 nếu bỏ comment: borrow of moved value: `id`
    // println!("{id}");
    let copy = moved.clone(); // clone tường minh: sao chép cả vùng heap
    println!("moved = {moved}, copy = {copy}");

    let len = take_id(moved); // moved chuyển vào hàm
    // Lỗi E0382 nếu bỏ comment: borrow of moved value: `moved`
    // println!("{moved}");
    println!("take_id đếm {len} byte; copy vẫn dùng được: {copy}");
    println!("peek_id mượn {} byte; copy vẫn còn: {copy}", peek_id(&copy));
}

fn demo_drop() {
    println!("== 2. Ra khỏi scope thì drop ==");
    let _outer = Tracked(String::from("outer"));
    {
        let _inner = Tracked(String::from("inner"));
        println!("  trong scope con");
    }
    println!("  đã ra scope con");
}

fn demo_ledger() -> Ledger {
    println!("== 3. Post bút toán ==");
    let mut ledger = Ledger::default();
    let entry = capture_entry();
    println!(
        "trước post: {} (khóa {}) có {} posting",
        entry.id,
        entry.idempotency_key,
        entry.postings.len()
    );
    post(&mut ledger, entry); // entry được move vào sổ
    // Lỗi E0382 nếu bỏ comment: borrow of moved value: `entry`
    // println!("{}", entry.id);
    println!(
        "số dư psp:vixanh:clearing = {}",
        balance(&ledger, "psp:vixanh:clearing")
    );
    println!(
        "số dư customer:CU-001 = {}",
        balance(&ledger, "customer:CU-001")
    );
    ledger
}

fn demo_borrow(mut ledger: Ledger) {
    println!("== 4. Nhiều người đọc, một người ghi ==");
    let r1 = &ledger;
    let r2 = &ledger; // nhiều &T cùng lúc: hợp lệ
    println!(
        "hai người đọc thấy {} và {}",
        balance(r1, "psp:vixanh:clearing"),
        balance(r2, "psp:vixanh:clearing")
    );
    // r1, r2 không còn được dùng, nên &mut bên dưới hợp lệ (NLL).

    let last_id = last_entry(&ledger).map(|e| e.id.clone());
    println!("bút toán cuối trước khi ghi thêm: {last_id:?}");

    // Lỗi E0502 nếu bỏ comment: mượn &mut khi tham chiếu & còn được dùng sau đó
    // let last = last_entry(&ledger);
    // post(&mut ledger, split_entry());
    // println!("{:?}", last);

    // Lỗi E0499 nếu bỏ comment: cannot borrow `ledger` as mutable more than once at a time
    // let w1 = &mut ledger;
    // let w2 = &mut ledger;
    // post(w1, split_entry());
    // post(w2, split_entry());

    post(&mut ledger, split_entry()); // một người ghi duy nhất
    println!("sổ có {} bút toán", ledger.entries.len());
    println!("driver:DR-001 = {}", balance(&ledger, "driver:DR-001"));
    println!(
        "platform:revenue = {}",
        balance(&ledger, "platform:revenue")
    );
    println!(
        "psp:vixanh:clearing = {}",
        balance(&ledger, "psp:vixanh:clearing")
    );
}

// Lỗi E0106 nếu bỏ comment: missing lifetime specifier
// fn dangle() -> &String {
//     let s = String::from("JE-000001");
//     &s
// }

fn run_demo() {
    demo_copy_move();
    demo_drop();
    let ledger = demo_ledger();
    demo_borrow(ledger); // ledger chuyển hẳn vào demo_borrow
}

fn run_verify() {
    // Copy vs move
    let a: i64 = 25_500;
    let b = a;
    assert_eq!(a, b);
    let fare = Money(25_500);
    let same = fare;
    assert_eq!(fare, same);

    // Drop: hai giá trị Tracked bị giải phóng khi ra scope.
    let before = DROPS.load(Ordering::SeqCst);
    {
        let _x = Tracked(String::from("verify-x"));
        let _y = Tracked(String::from("verify-y"));
    }
    assert_eq!(DROPS.load(Ordering::SeqCst) - before, 2);

    // Post và số dư
    let mut ledger = Ledger::default();
    let entry = capture_entry();
    assert_eq!(entry.postings.len(), 2);
    let sum: i64 = entry.postings.iter().map(|p| p.amount.0).sum();
    assert_eq!(sum, 0, "bút toán phải cân");
    post(&mut ledger, entry);
    assert_eq!(balance(&ledger, "psp:vixanh:clearing"), 25_500);
    assert_eq!(balance(&ledger, "customer:CU-001"), -25_500);
    assert_eq!(balance(&ledger, "driver:DR-001"), 0);
    assert_eq!(
        last_entry(&ledger).map(|e| e.id.as_str()),
        Some("JE-000001")
    );
    post(&mut ledger, split_entry());
    assert_eq!(balance(&ledger, "driver:DR-001"), 20_400);
    assert_eq!(balance(&ledger, "platform:revenue"), 5_100);
    assert_eq!(balance(&ledger, "psp:vixanh:clearing"), 0);
    assert_eq!(take_id(String::from("JE-000001")), 9);
    assert_eq!(peek_id("JE-000001"), 9);

    println!("VERIFY OK");
}

fn main() -> ExitCode {
    let verify = std::env::args().any(|a| a == "--verify");
    if verify {
        // assert thất bại gây panic: exit code 101; ta đổi thành 1 cho rõ nghĩa.
        match std::panic::catch_unwind(run_verify) {
            Ok(()) => ExitCode::SUCCESS,
            Err(_) => {
                eprintln!("VERIFY FAILED");
                ExitCode::from(1)
            }
        }
    } else {
        run_demo();
        ExitCode::SUCCESS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_moves_entry_and_balance_is_25500() {
        let mut ledger = Ledger::default();
        post(&mut ledger, capture_entry());
        assert_eq!(balance(&ledger, "psp:vixanh:clearing"), 25_500);
        assert_eq!(ledger.entries.len(), 1);
    }

    #[test]
    fn money_is_copy_string_is_clone() {
        let m = Money(1);
        let n = m;
        assert_eq!(m, n);
        let s = String::from("x");
        let t = s.clone();
        assert_eq!(s, t);
    }
}
