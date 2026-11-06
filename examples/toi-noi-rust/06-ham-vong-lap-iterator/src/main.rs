//! Bài 06: hàm, điều kiện, vòng lặp và iterator.
//! Tính cước và số dư tài khoản từ dòng bút toán (std-only).

use std::process::ExitCode;

/// Giá cơ bản (đồng), đã gồm 2 km đầu.
const BASE_FARE: i64 = 12_000;
/// Số km nằm trong giá cơ bản.
const INCLUDED_KM: u32 = 2;
/// Giá mỗi km vượt (đồng).
const PER_KM: i64 = 4_500;

/// Cách 1: vòng lặp. Mỗi km vượt cộng thêm PER_KM.
fn fare_loop(distance_km: u32) -> i64 {
    let mut total = BASE_FARE;
    for km in 1..=distance_km {
        if km > INCLUDED_KM {
            total += PER_KM;
        }
    }
    total
}

/// Cách 2: biểu thức. Dòng cuối không có `;` là giá trị trả về.
/// Quy tắc: 0 km vẫn thu giá cơ bản.
/// Lỗi E0308 nếu thêm `;` sau dòng cuối: thân hàm sẽ trả `()`, không phải `i64`.
fn fare(distance_km: u32) -> i64 {
    let extra_km = distance_km.saturating_sub(INCLUDED_KM);
    BASE_FARE + i64::from(extra_km) * PER_KM
}

/// `if` là biểu thức: hai nhánh phải cùng kiểu.
fn fare_label(distance_km: u32) -> &'static str {
    if distance_km <= INCLUDED_KM {
        "giá cơ bản"
    } else {
        "có km vượt"
    }
}

/// Rust không có tham số mặc định hay overload: "tùy chọn" phải nói rõ trong kiểu.
/// Phụ phí ở đây chỉ để minh họa chữ ký hàm, chưa là quy tắc nghiệp vụ.
fn fare_with_surcharge(distance_km: u32, surcharge: Option<i64>) -> i64 {
    fare(distance_km) + surcharge.unwrap_or(0)
}

/// `loop` có thể trả giá trị qua `break giá_trị`: km nhỏ nhất để cước vượt `limit`.
fn first_km_over_loop(limit: i64) -> u32 {
    let mut km = 0;
    loop {
        if fare(km) > limit {
            break km;
        }
        km += 1;
    }
}

/// Cùng kết quả bằng `while`.
fn first_km_over_while(limit: i64) -> u32 {
    let mut km = 0;
    while fare(km) <= limit {
        km += 1;
    }
    km
}

/// Một dòng bút toán. Số tiền có dấu: âm là tiền ra, dương là tiền vào.
#[derive(Debug, Clone)]
struct Posting {
    entry: String,
    account: String,
    amount: i64,
}

impl Posting {
    fn new(entry: &str, account: &str, amount: i64) -> Self {
        Posting {
            entry: entry.to_string(),
            account: account.to_string(),
            amount,
        }
    }
}

/// Bút toán JE-000001 của chuyến TN-0001 (rút gọn: chưa có psp:vixanh:clearing).
fn trip_postings() -> Vec<Posting> {
    vec![
        Posting::new("JE-000001", "customer:CU-001", -25_500),
        Posting::new("JE-000001", "platform:revenue", 5_100),
        Posting::new("JE-000001", "driver:DR-001", 20_400),
    ]
}

/// Số dư một tài khoản bằng vòng `for`.
fn balance_of_loop(postings: &[Posting], account: &str) -> i64 {
    let mut balance = 0;
    for p in postings {
        if p.account == account {
            balance += p.amount;
        }
    }
    balance
}

/// Cùng việc, viết bằng iterator.
fn balance_of(postings: &[Posting], account: &str) -> i64 {
    postings
        .iter()
        .filter(|p| p.account == account)
        .map(|p| p.amount)
        .sum::<i64>()
}

const ACCOUNTS: [&str; 3] = ["customer:CU-001", "platform:revenue", "driver:DR-001"];

/// Số dư của từng tài khoản trong ACCOUNTS.
fn balances(postings: &[Posting]) -> Vec<(&'static str, i64)> {
    ACCOUNTS
        .iter()
        .map(|&account| (account, balance_of(postings, account)))
        .collect::<Vec<_>>()
}

fn main() -> ExitCode {
    // --- 1. Hàm và biểu thức ---
    println!("fare_loop(5) = {}", fare_loop(5));
    println!("fare(5)      = {}", fare(5));
    println!("fare(2)      = {}", fare(2));
    println!("fare(0)      = {}", fare(0));
    println!("fare(7)      = {}", fare(7));
    println!("nhãn 2 km: {}, 5 km: {}", fare_label(2), fare_label(5));
    println!(
        "phụ phí: {} / {}",
        fare_with_surcharge(5, None),
        fare_with_surcharge(5, Some(2_000))
    );

    // Khối `{ ... }` cũng là biểu thức; giá trị là dòng cuối không có `;`.
    let km_over = {
        let distance = 5_u32;
        distance - INCLUDED_KM
    };
    println!("km vượt của 5 km: {km_over}");
    // Thêm `;` sau `distance - INCLUDED_KM` thì khối trả `()`, km_over không còn là `u32`.

    println!(
        "km đầu tiên có cước > 30.000: {}",
        first_km_over_loop(30_000)
    );

    // --- 2. Iterator trên dòng bút toán ---
    let postings = trip_postings();

    for (i, p) in postings.iter().enumerate() {
        println!("#{} {} {:>7} {}", i + 1, p.entry, p.amount, p.account);
    }

    let driver = balance_of(&postings, "driver:DR-001");
    println!("số dư driver:DR-001 = {driver}");
    for (account, balance) in balances(&postings) {
        println!("  {account:<18} {balance:>7}");
    }

    let credits = postings
        .iter()
        .filter(|p| p.amount > 0)
        .map(|p| p.amount)
        .sum::<i64>();
    let total = postings.iter().fold(0_i64, |acc, p| acc + p.amount);
    println!("tổng tiền vào = {credits}, tổng mọi posting = {total}");

    if let Some(top) = postings.iter().max_by_key(|p| p.amount) {
        println!("posting lớn nhất: {} ({})", top.account, top.amount);
    }

    // zip: ghép từng tài khoản với số dư kỳ vọng.
    let expected = [-25_500, 5_100, 20_400];
    let all_match = ACCOUNTS
        .iter()
        .zip(expected)
        .all(|(account, want)| balance_of(&postings, account) == want);
    println!("zip: số dư khớp kỳ vọng = {all_match}");

    // iter_mut: sửa tại chỗ trên BẢN SAO để tạo bút toán đảo (reversal).
    let mut reversal = postings.clone();
    for p in reversal.iter_mut() {
        p.entry = "JE-000002".to_string();
        p.amount = -p.amount;
    }
    println!(
        "đảo: số dư driver sau JE-000002 = {}",
        balance_of(&postings, "driver:DR-001") + balance_of(&reversal, "driver:DR-001")
    );

    // Lazy: adapter chưa chạy cho tới khi có `sum`/`collect`.
    let mut checked = 0;
    {
        let _lazy = postings.iter().filter(|p| {
            checked += 1;
            p.amount > 0
        });
        // Không gọi sum/collect: closure chưa chạy lần nào.
    }
    println!("sau khi chỉ dựng filter: closure đã chạy {checked} lần");
    let positive = postings
        .iter()
        .filter(|p| {
            checked += 1;
            p.amount > 0
        })
        .count();
    println!("sau count(): closure đã chạy {checked} lần, {positive} posting dương");

    // into_iter: lấy quyền sở hữu, postings không dùng được nữa.
    let amounts = postings.into_iter().map(|p| p.amount).collect::<Vec<_>>();
    println!("amounts = {amounts:?}");
    // Lỗi E0382 nếu dùng lại `postings` ở đây: giá trị đã bị move vào into_iter.

    if std::env::args().any(|a| a == "--verify") {
        return verify();
    }
    ExitCode::SUCCESS
}

fn verify() -> ExitCode {
    let postings = trip_postings();
    let checks = [
        ("fare(5) = 25.500", fare(5) == 25_500),
        ("fare(2) = 12.000", fare(2) == 12_000),
        ("fare(0) = 12.000", fare(0) == 12_000),
        ("fare(7) = 34.500", fare(7) == 34_500),
        (
            "fare_loop == fare với 0..=50 km",
            (0..=50).all(|km| fare_loop(km) == fare(km)),
        ),
        (
            "loop == while",
            first_km_over_loop(30_000) == 7 && first_km_over_while(30_000) == 7,
        ),
        (
            "driver:DR-001 = 20.400",
            balance_of(&postings, "driver:DR-001") == 20_400,
        ),
        (
            "vòng for == iterator",
            ACCOUNTS
                .iter()
                .all(|a| balance_of_loop(&postings, a) == balance_of(&postings, a)),
        ),
        (
            "tổng mọi posting = 0",
            postings.iter().map(|p| p.amount).sum::<i64>() == 0,
        ),
        (
            "zip khớp kỳ vọng",
            ACCOUNTS
                .iter()
                .zip([-25_500, 5_100, 20_400])
                .all(|(a, want)| balance_of(&postings, a) == want),
        ),
        (
            "số dư từng tài khoản",
            balances(&postings)
                == vec![
                    ("customer:CU-001", -25_500),
                    ("platform:revenue", 5_100),
                    ("driver:DR-001", 20_400),
                ],
        ),
    ];

    let mut failed = 0;
    for (name, ok) in checks {
        if !ok {
            eprintln!("FAIL: {name}");
            failed += 1;
        }
    }
    if failed == 0 {
        println!("VERIFY OK ({} kiểm tra)", checks.len());
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
