//! Bài 02: ghi MỘT bút toán cho chuyến TN-0001 rồi in số dư. Chỉ dùng thư viện chuẩn.

use std::env;

// Một dòng ghi (posting): tài khoản và số tiền có dấu, đơn vị đồng.
// `#[derive(Debug)]` cho phép in bằng `{:?}`.
#[derive(Debug)]
struct Posting {
    account: &'static str,
    amount: i64,
}

// Một bút toán (journal entry) gồm id, khóa idempotency và các posting.
#[derive(Debug)]
struct JournalEntry {
    id: &'static str,
    idempotency_key: &'static str,
    postings: Vec<Posting>,
}

// Khách CU-001 trả 25.500 VND qua VíXanh: ví khách giảm, tiền chờ về tăng.
fn trip_capture_entry() -> JournalEntry {
    JournalEntry {
        id: "JE-000001",
        idempotency_key: "trip:TN-0001:capture",
        postings: vec![
            Posting {
                account: "customer:CU-001",
                amount: -25_500,
            },
            Posting {
                account: "psp:vixanh:clearing",
                amount: 25_500,
            },
        ],
    }
}

// Tổng số tiền có dấu của mọi posting: bút toán hợp lệ phải bằng 0.
fn total(entry: &JournalEntry) -> i64 {
    let mut sum: i64 = 0;
    for posting in &entry.postings {
        sum += posting.amount;
    }
    sum
}

// Số dư của một tài khoản trong bút toán này (bài 06 sẽ viết lại bằng iterator).
fn balance(entry: &JournalEntry, account: &str) -> i64 {
    let mut sum: i64 = 0;
    for posting in &entry.postings {
        if posting.account == account {
            sum += posting.amount;
        }
    }
    sum
}

// Sai thì in lý do và thoát với mã 1.
fn expect(name: &str, condition: bool) {
    if !condition {
        eprintln!("VERIFY FAIL: {name}");
        std::process::exit(1);
    }
}

fn verify(entry: &JournalEntry) {
    expect("có đúng 2 posting", entry.postings.len() == 2);
    expect("tổng các posting bằng 0", total(entry) == 0);
    expect(
        "customer:CU-001 = -25500",
        balance(entry, "customer:CU-001") == -25_500,
    );
    expect(
        "psp:vixanh:clearing = 25500",
        balance(entry, "psp:vixanh:clearing") == 25_500,
    );
    expect(
        "tài khoản chưa có posting = 0",
        balance(entry, "driver:DR-001") == 0,
    );
    println!("VERIFY OK");
}

fn main() {
    let entry = trip_capture_entry();

    println!("Bút toán {} (key {})", entry.id, entry.idempotency_key);
    for posting in &entry.postings {
        println!("  {} {}", posting.account, posting.amount);
    }
    println!("Tổng các posting: {}", total(&entry));
    println!(
        "Số dư customer:CU-001: {}",
        balance(&entry, "customer:CU-001")
    );
    println!(
        "Số dư psp:vixanh:clearing: {}",
        balance(&entry, "psp:vixanh:clearing")
    );
    println!("Posting đầu tiên: {:?}", entry.postings[0]);

    // args[0] là đường dẫn chương trình; tham số của ta bắt đầu từ args[1].
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--verify" {
        verify(&entry);
    }
}
