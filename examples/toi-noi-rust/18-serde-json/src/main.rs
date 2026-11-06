//! Binary crate: ghi bút toán TN-0001, in JSON của sự kiện, đọc lại và báo lỗi có vị trí.

use std::process::ExitCode;

use ledger_lab::{
    AccountId, EntryPosted, JournalEntry, Ledger, LedgerError, LedgerEvent, Money, Posting,
    SchemaV1,
};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};
use serde::{Deserialize, Serialize};

/// JSON mẫu của hợp đồng (cũng là file mà test của BFF C# có thể đọc).
const GOLDEN: &str = include_str!("../tests/golden/ledger_entry_posted.v1.json");

/// Thân yêu cầu `PostEntry`: một bút toán, JSON camelCase, có xuống dòng để thấy số dòng/cột.
const ENTRY_JSON: &str = r#"{
  "idempotencyKey": "trip:TN-0001:capture",
  "postings": [
    { "account": "customer:CU-001", "amount": -25500 },
    { "account": "platform:revenue", "amount": 5100 },
    { "account": "driver:DR-001", "amount": 20400 }
  ]
}"#;

/// Ví dụ dòng chi trả dùng tiền dạng chuỗi (khi client không giữ nổi `i64`).
#[derive(Debug, Serialize, Deserialize)]
struct PayoutLine {
    #[serde(with = "ledger_lab::money::as_string")]
    amount: Money,
}

fn capture_trip_entry() -> Result<JournalEntry, LedgerError> {
    JournalEntry::try_new(
        "trip:TN-0001:capture",
        vec![
            Posting::new(
                AccountId::new("customer:CU-001")?,
                Money::from_dong(-25_500),
            ),
            Posting::new(AccountId::new("platform:revenue")?, Money::from_dong(5_100)),
            Posting::new(AccountId::new("driver:DR-001")?, Money::from_dong(20_400)),
        ],
    )
}

/// Ghi bút toán vào sổ rồi dựng sự kiện `LedgerEntryPosted` từ kết quả.
fn posted_event() -> Result<LedgerEvent, Box<dyn std::error::Error>> {
    let mut ledger = Ledger::new();
    let posted = ledger.post(capture_trip_entry()?)?;
    Ok(LedgerEvent::LedgerEntryPosted(EntryPosted {
        schema_version: SchemaV1,
        entry_id: posted.id.clone(),
        posted_at: "2026-11-06T03:15:00Z".parse()?,
        fee_rate: Decimal::new(20, 2),
        entry: posted.entry.clone(),
        memo: None,
        labels: Vec::new(),
    }))
}

/// Một dòng báo lỗi: vị trí do `serde_json::Error` cung cấp (0 nghĩa là không có vị trí).
fn describe(label: &str, e: &serde_json::Error) -> String {
    format!(
        "{label:<13}(dòng {}, cột {}, {:?}) {e}",
        e.line(),
        e.column(),
        e.classify()
    )
}

/// Các khóa ngoài cùng của JSON (để thấy `memo`/`labels` có mặt hay vắng).
fn top_level_keys(event: &LedgerEvent) -> Result<Vec<String>, serde_json::Error> {
    let value = serde_json::to_value(event)?;
    Ok(value
        .as_object()
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default())
}

/// Biến thể lỗi của thân `PostEntry`: (nhãn, JSON sai).
fn broken_entries() -> Vec<(&'static str, String)> {
    vec![
        ("thiếu phẩy", ENTRY_JSON.replace("-25500 },", "-25500 }")),
        ("amount chuỗi", ENTRY_JSON.replace("-25500", "\"-25500\"")),
        (
            "trường lạ",
            ENTRY_JSON.replace("5100 }", "5100, \"fee\": 1 }"),
        ),
        (
            "thiếu key",
            ENTRY_JSON.replace("  \"idempotencyKey\": \"trip:TN-0001:capture\",\n", ""),
        ),
        (
            "snake_case",
            ENTRY_JSON.replace("idempotencyKey", "idempotency_key"),
        ),
        ("lệch 100đ", ENTRY_JSON.replace("5100", "5000")),
    ]
}

/// Biến thể lỗi của sự kiện (đi qua enum có `tag = "type"`).
fn broken_events() -> Vec<(&'static str, String)> {
    vec![
        (
            "schemaV 2",
            GOLDEN.replace("\"schemaVersion\": 1", "\"schemaVersion\": 2"),
        ),
        (
            "type lạ",
            GOLDEN.replace("LedgerEntryPosted", "LedgerEntryVoided"),
        ),
        ("amount chuỗi", GOLDEN.replace("-25500", "\"-25500\"")),
    ]
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let event = posted_event()?;

    println!("== Ghi JSON (to_string_pretty) ==");
    println!("{}", serde_json::to_string_pretty(&event)?);

    println!("== Money ==");
    println!(
        "Money 25.500 đồng: {}",
        serde_json::to_string(&Money::from_dong(25_500))?
    );
    let line = PayoutLine {
        amount: Money::from_dong(25_500),
    };
    println!("dạng chuỗi:        {}", serde_json::to_string(&line)?);
    println!(
        "i64::MAX:          {}",
        serde_json::to_string(&Money::from_dong(i64::MAX))?
    );

    println!("== Đọc lại (from_str) ==");
    let back: LedgerEvent = serde_json::from_str(GOLDEN)?;
    println!("round-trip bằng nhau: {}", back == event);
    if let LedgerEvent::LedgerEntryPosted(p) = &back {
        let fare = Decimal::from(25_500);
        let fee = (fare * p.fee_rate)
            .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
            .to_i64()
            .ok_or("phí không vừa i64")?;
        println!(
            "feeRate {} nhân cước 25.500 = {}",
            p.fee_rate,
            Money::from_dong(fee)
        );
    }

    println!("== Trường tùy chọn ==");
    let mut with_extras = event.clone();
    if let LedgerEvent::LedgerEntryPosted(p) = &mut with_extras {
        p.memo = Some("Cuốc TN-0001".to_string());
        p.labels = vec!["trip".to_string(), "capture".to_string()];
    }
    println!("không có memo/labels: {:?}", top_level_keys(&event)?);
    println!("có memo/labels:       {:?}", top_level_keys(&with_extras)?);

    println!("== Lỗi khi đọc thân PostEntry ==");
    for (label, json) in broken_entries() {
        match serde_json::from_str::<JournalEntry>(&json) {
            Ok(_) => println!("{label}: lẽ ra phải lỗi"),
            Err(e) => println!("{}", describe(label, &e)),
        }
    }

    println!("== Lỗi khi đọc sự kiện ==");
    for (label, json) in broken_events() {
        match serde_json::from_str::<LedgerEvent>(&json) {
            Ok(_) => println!("{label}: lẽ ra phải lỗi"),
            Err(e) => println!("{}", describe(label, &e)),
        }
    }
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

    let event = posted_event().expect("dựng sự kiện");
    let pretty = serde_json::to_string_pretty(&event).expect("ghi JSON");
    check(pretty == GOLDEN.trim_end(), "JSON ghi ra khớp file golden");
    let back: Result<LedgerEvent, _> = serde_json::from_str(GOLDEN);
    check(back.ok().as_ref() == Some(&event), "round-trip bằng nhau");
    check(
        pretty.contains("\"feeRate\": \"0.20\""),
        "feeRate là chuỗi 0.20",
    );
    check(
        pretty.contains("\"postedAt\": \"2026-11-06T03:15:00Z\""),
        "postedAt ISO 8601 UTC",
    );
    check(
        pretty.contains("\"amount\": -25500"),
        "Money là số nguyên đồng",
    );
    check(!pretty.contains('_'), "mọi khóa JSON là camelCase");

    check(
        serde_json::to_string(&Money::from_dong(25_500))
            .ok()
            .as_deref()
            == Some("25500"),
        "Money transparent",
    );
    let line = PayoutLine {
        amount: Money::from_dong(25_500),
    };
    check(
        serde_json::to_string(&line).ok().as_deref() == Some("{\"amount\":\"25500\"}"),
        "Money dạng chuỗi",
    );

    let mut positions = Vec::new();
    for (_, json) in broken_entries() {
        match serde_json::from_str::<JournalEntry>(&json) {
            Ok(_) => positions.push((usize::MAX, 0)),
            Err(e) => positions.push((e.line(), e.column())),
        }
    }
    check(
        positions == [(5, 5), (4, 54), (5, 58), (7, 1), (2, 19), (0, 0)],
        "vị trí lỗi (dòng, cột) đúng như trong bài",
    );

    let version = serde_json::from_str::<LedgerEvent>(&broken_events()[0].1);
    check(
        version
            .err()
            .is_some_and(|e| e.to_string().contains("schemaVersion 2")),
        "schemaVersion 2 bị từ chối",
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
