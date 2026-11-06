//! Test hợp đồng JSON: chỉ dùng API `pub` của thư viện, như một consumer thật.

use ledger_lab::{EntryReversed, JournalEntry, LedgerEvent, Money, SchemaV1};
use serde_json::{Value, json};

const GOLDEN: &str = include_str!("golden/ledger_entry_posted.v1.json");

fn golden_value() -> Value {
    serde_json::from_str(GOLDEN).expect("file golden là JSON hợp lệ")
}

#[test]
fn golden_round_trip_is_stable() -> Result<(), serde_json::Error> {
    let event: LedgerEvent = serde_json::from_str(GOLDEN)?;
    // Đọc rồi ghi lại phải ra đúng chuỗi ban đầu (file golden không có dòng trống cuối).
    assert_eq!(serde_json::to_string_pretty(&event)?, GOLDEN.trim_end());
    Ok(())
}

#[test]
fn keys_are_camel_case() {
    let v = golden_value();
    for key in ["schemaVersion", "entryId", "postedAt", "feeRate", "entry"] {
        assert!(v.get(key).is_some(), "thiếu khóa {key}");
    }
    assert!(v["entry"].get("idempotencyKey").is_some());
}

#[test]
fn money_is_integer_by_default_and_string_on_request() -> Result<(), serde_json::Error> {
    assert_eq!(serde_json::to_string(&Money::from_dong(-25_500))?, "-25500");
    let back: Money = serde_json::from_str("20400")?;
    assert_eq!(back, Money::from_dong(20_400));
    // Số thực không phải tiền: 25500.5 bị từ chối chứ không bị cắt.
    assert!(serde_json::from_str::<Money>("25500.5").is_err());
    Ok(())
}

#[test]
fn fee_rate_must_be_a_string() {
    let mut v = golden_value();
    v["feeRate"] = json!(0.2);
    let err = serde_json::from_value::<LedgerEvent>(v).unwrap_err();
    assert!(err.to_string().contains("floating point"), "{err}");
}

#[test]
fn fee_rate_keeps_its_scale() -> Result<(), serde_json::Error> {
    let mut v = golden_value();
    v["feeRate"] = json!("0.2");
    let event: LedgerEvent = serde_json::from_value(v)?;
    // "0.2" và "0.20" cùng giá trị nhưng khác dạng chữ: chuỗi ghi lại giữ đúng scale đã đọc.
    assert!(serde_json::to_string(&event)?.contains("\"feeRate\":\"0.2\""));
    Ok(())
}

#[test]
fn timestamp_with_offset_is_normalized_to_utc() -> Result<(), serde_json::Error> {
    let mut v = golden_value();
    v["postedAt"] = json!("2026-11-06T10:15:00+07:00");
    let event: LedgerEvent = serde_json::from_value(v)?;
    let out = serde_json::to_value(&event)?;
    assert_eq!(out["postedAt"], "2026-11-06T03:15:00Z");
    Ok(())
}

#[test]
fn timestamp_without_offset_is_rejected() {
    let mut v = golden_value();
    v["postedAt"] = json!("2026-11-06T03:15:00");
    assert!(serde_json::from_value::<LedgerEvent>(v).is_err());
}

#[test]
fn optional_fields_default_and_are_skipped() -> Result<(), serde_json::Error> {
    let event: LedgerEvent = serde_json::from_str(GOLDEN)?;
    let LedgerEvent::LedgerEntryPosted(posted) = &event else {
        panic!("golden phải là LedgerEntryPosted");
    };
    assert_eq!(posted.memo, None);
    assert!(posted.labels.is_empty());
    let out = serde_json::to_value(&event)?;
    assert!(out.get("memo").is_none() && out.get("labels").is_none());
    Ok(())
}

#[test]
fn unknown_field_is_rejected() {
    let mut v = golden_value();
    v["extra"] = json!(1);
    let err = serde_json::from_value::<LedgerEvent>(v).unwrap_err();
    assert!(err.to_string().contains("unknown field `extra`"), "{err}");
}

#[test]
fn unsupported_schema_version_is_rejected() {
    let mut v = golden_value();
    v["schemaVersion"] = json!(2);
    let err = serde_json::from_value::<LedgerEvent>(v).unwrap_err();
    assert!(err.to_string().contains("schemaVersion 2"), "{err}");
}

#[test]
fn unknown_event_type_is_rejected() {
    let mut v = golden_value();
    v["type"] = json!("LedgerEntryVoided");
    let err = serde_json::from_value::<LedgerEvent>(v).unwrap_err();
    assert!(err.to_string().contains("unknown variant"), "{err}");
}

#[test]
fn reading_json_cannot_bypass_the_balance_invariant() {
    let text = r#"{"idempotencyKey":"k","postings":[
        {"account":"a","amount":-100},{"account":"b","amount":90}]}"#;
    let err = serde_json::from_str::<JournalEntry>(text).unwrap_err();
    assert!(err.to_string().contains("lệch"), "{err}");
    let one = r#"{"idempotencyKey":"k","postings":[{"account":"a","amount":0}]}"#;
    assert!(serde_json::from_str::<JournalEntry>(one).is_err());
}

#[test]
fn blank_account_id_is_rejected_with_position() {
    let text = "{\"idempotencyKey\":\"k\",\n\"postings\":[{\"account\":\" \",\"amount\":1}]}";
    let err = serde_json::from_str::<JournalEntry>(text).unwrap_err();
    assert_eq!(err.line(), 2);
    assert!(err.to_string().contains("mã tài khoản rỗng"), "{err}");
}

#[test]
fn syntax_error_reports_line_and_column() {
    let err = serde_json::from_str::<JournalEntry>("{\n  \"idempotencyKey\" \"k\"\n}").unwrap_err();
    assert_eq!((err.line(), err.column()), (2, 20));
    assert!(err.is_syntax());
}

#[test]
fn reversed_event_round_trips() -> Result<(), serde_json::Error> {
    let event = LedgerEvent::LedgerEntryReversed(EntryReversed {
        schema_version: SchemaV1,
        entry_id: "JE-000002".to_string(),
        reverses_entry_id: "JE-000001".to_string(),
        reason: "Hoàn tiền chuyến TN-0001".to_string(),
    });
    let text = serde_json::to_string(&event)?;
    assert!(text.starts_with("{\"type\":\"LedgerEntryReversed\",\"schemaVersion\":1"));
    assert_eq!(serde_json::from_str::<LedgerEvent>(&text)?, event);
    Ok(())
}
