//! Lab 55: hợp đồng sự kiện LedgerEntryPosted xuyên ngôn ngữ.
//!
//! - không tham số: in bản mẫu ở ba dạng (nhị phân, JSON thuần, proto3 JSON chuẩn).
//! - `--verify`: kiểm mọi giá trị in trong bài, KHÔNG cần .NET/Go/Python.
//! - `--with-consumers`: chạy consumer C#, Go, Python thật, so với `fixtures/expected_matrix.json`;
//!   thoát mã khác 0 nếu một consumer lỗi hoặc kết quả lệch. Thêm `--accept` để ghi lại kỳ vọng.
//! - `--write-fixtures`: sinh lại fixture (chỉ khi cố ý đổi hợp đồng).
use anyhow::{Result, bail};
use ledger_55_event_contract::{
    compat::{self, Mode, Wire},
    consumers, fixtures, json_contract, lab_dir, matrix, pool, typed,
};
use serde_json::Value;
use std::collections::BTreeMap;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |flag: &str| args.iter().any(|a| a == flag);
    if has("--write-fixtures") {
        let n = fixtures::write_all(&lab_dir().join("fixtures"))?;
        println!("đã ghi {n} tệp fixture");
    } else if has("--verify") {
        verify()?;
    } else if has("--with-consumers") {
        with_consumers(has("--accept"))?;
    } else {
        demo()?;
    }
    Ok(())
}

/// In bản mẫu của chuyến TN-0001 ở các dạng.
fn demo() -> Result<()> {
    let event = typed::sample_capture();
    let bin = typed::encode(&event);
    let desc = pool::descriptor("s1")?;
    let msg = pool::from_bin(&desc, &bin)?;
    let plain = pool::to_json_bytes(&msg, false, false)?;
    let canonical = pool::to_json_bytes(&msg, true, false)?;
    println!(
        "LedgerEntryPosted {} ({})",
        event.entry_id, event.idempotency_key
    );
    println!("  protobuf nhị phân     : {:>3} byte", bin.len());
    println!("  JSON thuần (int64=số) : {:>3} byte", plain.len());
    println!("  proto3 JSON chuẩn     : {:>3} byte", canonical.len());
    println!("JSON thuần   : {}", String::from_utf8_lossy(&plain));
    println!("proto3 JSON  : {}", String::from_utf8_lossy(&canonical));
    Ok(())
}

fn expected_path() -> std::path::PathBuf {
    lab_dir().join("fixtures/expected_matrix.json")
}

fn load_expected() -> Result<Value> {
    Ok(serde_json::from_slice(&std::fs::read(expected_path())?)?)
}

/// Kiểm không cần công cụ ngoài. Sai thì trả `Err` (exit 1).
fn verify() -> Result<()> {
    let dir = lab_dir().join("fixtures");

    // 1. Fixture commit sẵn khớp bản sinh lại từ code (nhị phân, JSON, descriptor set, danh sách ca).
    let drift = fixtures::drift(&dir)?;
    if !drift.is_empty() {
        bail!("fixture lệch: {drift:?}");
    }
    println!(
        "1. fixture commit sẵn khớp bản sinh lại ({} tệp)",
        fixtures::generate()?.len()
    );

    // 2. Kiểu prost có kiểu và message động cho cùng một byte; giá trị đúng sau khi giải mã.
    let bin = std::fs::read(dir.join("data/s1.bin"))?;
    let event = typed::decode(&bin)?;
    assert_eq!(event, typed::sample_capture());
    assert_eq!(event.postings[0].amount_dong, -25_500);
    assert_eq!(event.postings.iter().map(|p| p.amount_dong).sum::<i64>(), 0);
    let dynamic = pool::from_bin(&pool::descriptor("s1")?, &bin)?;
    assert_eq!(pool::to_bin(&dynamic), bin);
    println!(
        "2. nhị phân s1: {} byte, giải mã bằng prost có kiểu và prost-reflect cho cùng kết quả, tổng dòng = 0",
        bin.len()
    );

    // 3. Ba dạng của cùng một sự kiện: kích thước và hình dạng int64.
    let plain = std::fs::read(dir.join("data/s1.plain.json"))?;
    let canonical = std::fs::read(dir.join("data/s1.json"))?;
    assert!(String::from_utf8_lossy(&plain).contains("\"amountDong\":-25500"));
    assert!(String::from_utf8_lossy(&canonical).contains("\"amountDong\":\"-25500\""));
    println!(
        "3. kích thước: nhị phân {} B, JSON thuần {} B, proto3 JSON chuẩn {} B (int64 là số / là chuỗi)",
        bin.len(),
        plain.len(),
        canonical.len()
    );

    // 4. JSON Schema: bản thuần hợp lệ; số 2^53+1, trường lạ và dạng chuỗi bị từ chối.
    json_contract::validate_v1_bytes(&plain)?.expect("s1.plain.json phải hợp lệ");
    let big_plain = std::fs::read(dir.join("data/s1.big.plain.json"))?;
    let big_err =
        json_contract::validate_v1_bytes(&big_plain)?.expect_err("2^53+1 phải bị từ chối");
    let mut with_extra: Value = serde_json::from_slice(&plain)?;
    with_extra["traceId"] = Value::from("4bf9");
    json_contract::validate_v1(&with_extra).expect_err("schema v1 đóng, trường lạ phải bị từ chối");
    json_contract::validate_v1_bytes(&canonical)?
        .expect_err("dạng chuỗi không thuộc schema JSON thuần");
    println!(
        "4. JSON Schema v1: bản thuần hợp lệ; 2^53+1 bị từ chối ({}); trường lạ và dạng chuỗi bị từ chối",
        big_err[0]
    );
    // Cùng số đó, protobuf nhị phân giữ đúng.
    let big = typed::decode(&std::fs::read(dir.join("data/s1.big.bin"))?)?;
    assert_eq!(big.postings[1].amount_dong, typed::BIG_AMOUNT);
    assert_eq!(typed::BIG_AMOUNT as f64 as i64, typed::BIG_AMOUNT - 1);
    println!(
        "   protobuf nhị phân giữ đúng {} nhưng f64 chỉ giữ {}",
        typed::BIG_AMOUNT,
        typed::BIG_AMOUNT as f64 as i64
    );

    // 5. Ma trận do reader Rust đo, so với bảng kỳ vọng commit sẵn.
    let cases = matrix::all_cases();
    let rust = matrix::rust_readings(&cases, &dir)?;
    let measured = consumers::assemble(&cases, &BTreeMap::from([("rust".to_owned(), rust)]))?;
    let diff = consumers::diff_expected(&measured, &load_expected()?);
    if !diff.is_empty() {
        bail!("ma trận Rust lệch kỳ vọng: {diff:#?}");
    }
    println!(
        "5. ma trận Rust ({} ca) khớp expected_matrix.json",
        measured.outcomes.len() + measured.naive.len()
    );

    // 6. Checker so descriptor: không âm tính giả; dương tính giả đúng như đã biết.
    let rust_outcomes: BTreeMap<String, matrix::Outcome> = measured
        .outcomes
        .iter()
        .map(|(id, by)| (id.clone(), by["rust"].clone()))
        .collect();
    let (false_neg, false_pos) = compat::check_against_measured(&rust_outcomes)?;
    assert!(false_neg.is_empty(), "checker bỏ sót: {false_neg:?}");
    assert_eq!(false_pos, ["rename/backward/bin", "rename/forward/bin"]);
    println!(
        "6. checker descriptor: 0 âm tính giả; dương tính giả: {false_pos:?} (đổi tên: binary ổn nhưng không phân biệt được với dùng lại số)"
    );
    let ok = |old, new, mode, wire| compat::compare(old, new).map(|r| r.is_compatible(mode, wire));
    assert!(ok("s1", "s2", Mode::Full, Wire::Binary)?);
    assert!(!ok("s1", "s2", Mode::Forward, Wire::Json)?);
    assert!(!ok("s2", "s2n", Mode::Backward, Wire::Binary)?);
    println!(
        "   s1->s2 FULL binary: ổn; s1->s2 FORWARD json chặt: vỡ; s2->s2n BACKWARD binary: vỡ"
    );

    // 7. protoc thật từ chối dùng lại số đã reserved.
    let msg = compat::protoc_rejects_reserved_reuse()?;
    assert!(msg.contains("reserved number 6"));
    println!("7. protoc từ chối s3r.proto: {msg}");

    // 8. Thứ tự triển khai rút từ số đo.
    println!("8. thứ tự triển khai an toàn (đo bằng reader Rust):");
    println!("   {:<16} {:<16} JSON chặt", "thay đổi", "nhị phân");
    for pair in &matrix::PAIRS {
        let bin = matrix::rollout(pair, matrix::Format::Bin, &rust_outcomes);
        let json = matrix::rollout(pair, matrix::Format::Json, &rust_outcomes);
        println!("   {:<16} {:<16} {json:?}", pair.name, format!("{bin:?}"));
    }
    println!("VERIFY OK");
    Ok(())
}

/// Chạy consumer thật. Mọi lỗi (thiếu công cụ, consumer lỗi, lệch kỳ vọng) là `Err` (exit 1).
fn with_consumers(accept: bool) -> Result<()> {
    let dir = lab_dir().join("fixtures");
    let drift = fixtures::drift(&dir)?;
    if !drift.is_empty() {
        bail!("fixture lệch bản sinh lại: {drift:?}");
    }
    let cases = matrix::all_cases();
    let mut per_lang = BTreeMap::new();
    per_lang.insert("rust".to_owned(), matrix::rust_readings(&cases, &dir)?);
    for lang in ["dotnet", "go", "python"] {
        println!("chạy consumer {lang} ...");
        per_lang.insert(lang.to_owned(), consumers::run_consumer(lang, &dir)?);
    }
    let measured = consumers::assemble(&cases, &per_lang)?;
    println!(
        "\nR = Rust (prost-reflect), C# = Google.Protobuf, Go = protobuf-go, Py = protobuf Python\n"
    );
    print!("{}", consumers::render_table(&measured));
    println!("\nsố 2^53+1 đọc như client hay làm:");
    for (id, by) in &measured.naive {
        for (lang, v) in by {
            println!("  {id:<16} {lang:<7} {v}");
        }
    }
    println!("\nlỗi nguyên văn của từng thư viện (JSON chặt):");
    for id in [
        "add-field/forward/json",
        "rename/backward/json",
        "enum-add/forward/json",
    ] {
        for lang in consumers::LANGS {
            if let Some(matrix::Reading::Error(e)) = per_lang[lang].get(id) {
                let first = e.lines().next().unwrap_or("");
                println!("  {id:<24} {lang:<7} {first}");
            }
        }
    }
    println!("\nca hợp đồng (s1, big):");
    for id in [
        "s1/bin",
        "s1/json",
        "s1/plain",
        "big/bin",
        "big/json",
        "big/plain",
    ] {
        let codes: Vec<String> = measured.outcomes[id]
            .iter()
            .map(|(l, o)| format!("{l}={}", o.code()))
            .collect();
        println!("  {id:<10} {}", codes.join(" "));
    }

    if accept {
        let mut text = serde_json::to_string_pretty(&consumers::to_expected(&measured))?;
        text.push('\n');
        std::fs::write(expected_path(), text)?;
        println!("\nđã ghi expected_matrix.json");
        return Ok(());
    }
    let diff = consumers::diff_expected(&measured, &load_expected()?);
    if !diff.is_empty() {
        bail!(
            "kết quả consumer lệch expected_matrix.json:\n{}",
            diff.join("\n")
        );
    }
    println!(
        "\nCONSUMERS OK ({} ca x 4 reader khớp expected_matrix.json)",
        cases.len()
    );
    Ok(())
}
