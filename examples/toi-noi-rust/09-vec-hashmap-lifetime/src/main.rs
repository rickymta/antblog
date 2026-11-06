//! Bài 09: Vec, HashMap, BTreeMap, HashSet và lifetime cơ bản (chỉ dùng std).
//! Sổ cái trong bộ nhớ: tiền là i64 đơn vị đồng (newtype Money ở bài 04 và 05).

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::panic;

/// Một dòng của bút toán: số tiền có dấu (dương = ghi có cho tài khoản nhận, âm = trừ).
#[derive(Debug, Clone, PartialEq)]
struct Posting {
    account: String,
    amount: i64,
}

#[derive(Debug, Clone, PartialEq)]
struct JournalEntry {
    id: String,
    idempotency_key: String,
    postings: Vec<Posting>,
}

#[derive(Debug, PartialEq)]
enum PostError {
    TooFewPostings(usize),
    Unbalanced(i64),
    DuplicateKey(String),
}

/// Sổ cái trong bộ nhớ: ba cấu trúc phải luôn khớp nhau.
#[derive(Default)]
struct Ledger {
    entries: BTreeMap<String, JournalEntry>, // sắp theo id (JE-000001, ...)
    balances: HashMap<String, i64>,          // số dư theo tài khoản, tra O(1) trung bình
    seen_keys: HashSet<String>,              // idempotency key đã ghi
}

impl Ledger {
    /// Ghi một bút toán: kiểm tra trước, sau đó cập nhật cả ba cấu trúc.
    fn post(&mut self, entry: JournalEntry) -> Result<(), PostError> {
        if entry.postings.len() < 2 {
            return Err(PostError::TooFewPostings(entry.postings.len()));
        }
        let sum: i64 = entry.postings.iter().map(|p| p.amount).sum();
        if sum != 0 {
            return Err(PostError::Unbalanced(sum));
        }
        // insert trả false nếu key đã có. Kiểm tra cân bằng trước để key không bị "đốt" oan.
        if !self.seen_keys.insert(entry.idempotency_key.clone()) {
            return Err(PostError::DuplicateKey(entry.idempotency_key));
        }
        for p in &entry.postings {
            *self.balances.entry(p.account.clone()).or_insert(0) += p.amount;
        }
        self.entries.insert(entry.id.clone(), entry);
        Ok(())
    }

    /// Trả tham chiếu vào map: không sao chép, nhưng mượn `&self` chừng nào còn dùng.
    fn balance(&self, account: &str) -> Option<&i64> {
        self.balances.get(account)
    }

    /// Các bút toán có id từ `from` trở đi, theo thứ tự id.
    fn entries_from<'a>(&'a self, from: &str) -> impl Iterator<Item = &'a JournalEntry> + 'a {
        // `range(from..)` với `&str` không biên dịch được (E0277); dùng String làm cận dưới.
        self.entries.range(from.to_string()..).map(|(_, e)| e)
    }

    /// Bài tập: bút toán có id từ `from` đến `to` (gồm cả hai đầu).
    /// Có `&self` nên kết quả tự mượn từ `self`, không cần ghi `'a`.
    fn entries_between(&self, from: &str, to: &str) -> Vec<&JournalEntry> {
        self.entries
            .range(from.to_string()..=to.to_string())
            .map(|(_, e)| e)
            .collect()
    }

    fn total(&self) -> i64 {
        self.balances.values().sum()
    }
}

/// Elision: một tham chiếu vào, một tham chiếu ra, nên không cần ghi `'a`.
fn largest_posting(entry: &JournalEntry) -> Option<&Posting> {
    entry.postings.iter().max_by_key(|p| p.amount.abs())
}

/// Hai tham chiếu vào: compiler không đoán được kết quả mượn từ đâu, phải ghi `'a`.
fn pick<'a>(a: &'a Posting, b: &'a Posting) -> &'a Posting {
    if a.amount.abs() >= b.amount.abs() {
        a
    } else {
        b
    }
}
// Lỗi E0106 nếu bỏ `<'a>` và `'a`:
// fn pick(a: &Posting, b: &Posting) -> &Posting { ... }

/// Slice chỉ đọc: nhận được cả `&Vec<Posting>` lẫn mảng.
fn sum_amounts(postings: &[Posting]) -> i64 {
    postings.iter().map(|p| p.amount).sum()
}

/// Slice có thể sửa: đổi dấu mọi posting (dùng cho bút toán đảo).
fn flip_signs(postings: &mut [Posting]) {
    for p in postings.iter_mut() {
        p.amount = -p.amount;
    }
}

fn posting(account: &str, amount: i64) -> Posting {
    Posting {
        account: account.to_string(),
        amount,
    }
}

/// Bút toán thu cước một chuyến: khách trả `fare`, nền tảng giữ 20%, tài xế nhận phần còn lại.
fn trip_entry(seq: u32, trip: &str, customer: &str, fare: i64) -> JournalEntry {
    let fee = fare / 5; // 20%; làm tròn tiền thật dùng rust_decimal ở bài 03
    JournalEntry {
        id: format!("JE-{seq:06}"),
        idempotency_key: format!("trip:{trip}:capture"),
        postings: vec![
            posting(&format!("customer:{customer}"), -fare),
            posting("driver:DR-001", fare - fee),
            posting("platform:revenue", fee),
        ],
    }
}

/// Chạy `f`, bắt panic và trả thông điệp panic (không in ra stderr).
fn panic_message<F: FnOnce() -> i64 + panic::UnwindSafe>(f: F) -> Option<String> {
    let old = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let result = panic::catch_unwind(f);
    panic::set_hook(old);
    result
        .err()
        .and_then(|p| p.downcast_ref::<String>().cloned())
}

struct VecReport {
    len_after_push: usize,
    get_missing: Option<i64>,
    index_panic: Option<String>,
    sorted: Vec<i64>,
    after_retain: Vec<i64>,
    capacity_ok: bool,
}

fn demo_vec() -> VecReport {
    let mut amounts: Vec<i64> = Vec::with_capacity(8); // cấp sẵn chỗ cho 8 phần tử
    let capacity_ok = amounts.capacity() >= 8;
    for a in [20_400, -25_500, 5_100, 9_600] {
        amounts.push(a);
    }
    let len_after_push = amounts.len();

    // `get` trả Option, không panic; `v[i]` ngoài biên thì panic.
    let get_missing = amounts.get(7).copied();
    let idx = amounts.len() + 3;
    let probe = amounts.clone();
    let index_panic = panic_message(move || probe[idx]);

    amounts.sort_by_key(|a| a.abs()); // sắp theo giá trị tuyệt đối
    let sorted = amounts.clone();
    amounts.retain(|a| *a > 0); // giữ lại phần tử thỏa điều kiện
    VecReport {
        len_after_push,
        get_missing,
        index_panic,
        sorted,
        after_retain: amounts,
        capacity_ok,
    }
}

fn main() {
    let verify = std::env::args().any(|a| a == "--verify");

    // --- Vec và slice ---
    let v = demo_vec();
    println!(
        "Vec: len sau push = {}, get(7) = {:?}",
        v.len_after_push, v.get_missing
    );
    println!(
        "Vec: index ngoài biên -> panic: {}",
        v.index_panic.as_deref().unwrap_or("-")
    );
    println!(
        "Vec: sort_by_key(abs) = {:?}; retain(>0) = {:?}",
        v.sorted, v.after_retain
    );

    let mut entry = trip_entry(1, "TN-0001", "CU-001", 25_500);
    println!("Slice: tổng posting = {}", sum_amounts(&entry.postings));
    flip_signs(&mut entry.postings[..]);
    println!(
        "Slice: sau flip_signs, posting đầu = {}",
        entry.postings[0].amount
    );
    flip_signs(&mut entry.postings);

    // --- VecDeque: hàng đợi bút toán chờ ghi, vào trước ra trước ---
    let mut queue: VecDeque<JournalEntry> = VecDeque::new();
    queue.push_back(trip_entry(1, "TN-0001", "CU-001", 25_500));
    queue.push_back(trip_entry(2, "TN-0002", "CU-002", 12_000));
    queue.push_back(trip_entry(3, "TN-0003", "CU-003", 34_500));
    queue.push_back(trip_entry(4, "TN-0001", "CU-001", 25_500)); // gửi lại trùng key

    // --- Ledger ---
    let mut ledger = Ledger::default();
    let mut results = Vec::new();
    while let Some(e) = queue.pop_front() {
        let id = e.id.clone();
        let r = ledger.post(e);
        println!("post {id} -> {r:?}");
        results.push(r);
    }

    let mut rows: Vec<(&String, &i64)> = ledger.balances.iter().collect();
    rows.sort_by_key(|(account, _)| account.as_str()); // HashMap không có thứ tự: sắp trước khi in
    for (account, balance) in &rows {
        println!("  {account:<20} {balance:>8}");
    }
    println!("Tổng mọi số dư = {}", ledger.total());

    let from_2: Vec<String> = ledger
        .entries_from("JE-000002")
        .map(|e| e.id.clone())
        .collect();
    println!("range(\"JE-000002\"..) -> {from_2:?}");

    let first = &ledger.entries["JE-000001"];
    println!("largest_posting(JE-000001) = {:?}", largest_posting(first));
    println!("pick = {:?}", pick(&first.postings[1], &first.postings[2]));

    // Bút toán lệch bị từ chối và không đốt idempotency key (thử trên sổ cái nháp).
    let mut scratch = Ledger::default();
    let mut bad = trip_entry(5, "TN-0004", "CU-004", 10_000);
    bad.postings[2].amount += 1;
    let unbalanced = scratch.post(bad.clone());
    bad.postings[2].amount -= 1;
    let retry = scratch.post(bad);
    println!("lệch 1 đồng -> {unbalanced:?}; sửa rồi ghi lại -> {retry:?}");

    if verify {
        run_verify(&v, &ledger, &results, &from_2, unbalanced, retry);
    }
}

fn run_verify(
    v: &VecReport,
    ledger: &Ledger,
    results: &[Result<(), PostError>],
    from_2: &[String],
    unbalanced: Result<(), PostError>,
    retry: Result<(), PostError>,
) {
    let mut failed = Vec::new();
    let mut check = |ok: bool, name: &str| {
        if !ok {
            failed.push(name.to_string());
        }
    };

    check(v.capacity_ok, "with_capacity(8) cho capacity >= 8");
    check(v.len_after_push == 4, "len sau 4 lần push = 4");
    check(v.get_missing.is_none(), "get(7) trả None");
    check(
        v.index_panic.as_deref() == Some("index out of bounds: the len is 4 but the index is 7"),
        "v[7] panic đúng thông điệp",
    );
    check(
        v.sorted == [5_100, 9_600, 20_400, -25_500],
        "sort_by_key theo abs",
    );
    check(
        v.after_retain == [5_100, 9_600, 20_400],
        "retain giữ số dương",
    );

    check(
        results[..3].iter().all(Result::is_ok),
        "3 bút toán đầu ghi được",
    );
    check(
        results[3] == Err(PostError::DuplicateKey("trip:TN-0001:capture".into())),
        "ghi trùng idempotency key bị từ chối",
    );
    check(
        ledger.balance("driver:DR-001") == Some(&57_600),
        "số dư driver = 57.600",
    );
    check(
        ledger.balance("platform:revenue") == Some(&14_400),
        "số dư platform = 14.400",
    );
    check(
        ledger.balance("customer:CU-001") == Some(&-25_500),
        "số dư CU-001 = -25.500",
    );
    check(
        ledger.balance("customer:CU-999").is_none(),
        "tài khoản lạ trả None",
    );
    check(ledger.total() == 0, "tổng mọi số dư = 0");
    check(
        from_2 == ["JE-000002", "JE-000003"],
        "range từ JE-000002 trả 2 bút toán",
    );
    check(
        ledger.entries_between("JE-000001", "JE-000002").len() == 2,
        "entries_between(JE-000001, JE-000002) trả 2 bút toán",
    );
    check(
        unbalanced == Err(PostError::Unbalanced(1)),
        "bút toán lệch bị từ chối",
    );
    check(retry.is_ok(), "key không bị đốt khi bút toán lệch");
    check(
        ledger.entries.len() == 3 && ledger.seen_keys.len() == 3 && ledger.balances.len() == 5,
        "entries, seen_keys và balances khớp nhau",
    );

    if failed.is_empty() {
        println!("VERIFY OK");
    } else {
        for f in &failed {
            eprintln!("VERIFY FAIL: {f}");
        }
        std::process::exit(1);
    }
}
