//! Contract test: chạy trong `cargo test`, dùng fixture commit sẵn, không cần .NET/Go/Python và không cần hạ tầng.
use ledger_55_event_contract::{
    compat::{self, Mode, Wire},
    consumers, fixtures, json_contract, lab_dir, matrix, pool, semantics, typed,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn fixture(rel: &str) -> Vec<u8> {
    std::fs::read(lab_dir().join("fixtures").join(rel)).expect("fixture commit sẵn")
}

fn rust_outcomes() -> BTreeMap<String, matrix::Outcome> {
    let cases = matrix::all_cases();
    let readings = matrix::rust_readings(&cases, &lab_dir().join("fixtures")).unwrap();
    cases
        .iter()
        .map(|c| (c.id().to_owned(), matrix::classify(c, &readings[c.id()])))
        .collect()
}

#[test]
fn committed_fixtures_match_what_code_generates() {
    // Đổi hợp đồng mà quên cập nhật fixture (hoặc ngược lại) thì test này đỏ.
    let drift = fixtures::drift(&lab_dir().join("fixtures")).unwrap();
    assert!(drift.is_empty(), "fixture lệch: {drift:?}");
}

#[test]
fn v1_wire_bytes_are_pinned() {
    // Golden bytes: field 1 (schema_version) = 1, field 2 (entry_id) dài 9 byte "JE-000001".
    let bin = fixture("data/s1.bin");
    assert_eq!(&bin[..13], b"\x08\x01\x12\x09JE-000001");
    assert_eq!(bin.len(), 100);
    let event = typed::decode(&bin).unwrap();
    assert_eq!(event, typed::sample_capture());
    assert_eq!(event.postings.iter().map(|p| p.amount_dong).sum::<i64>(), 0);
}

#[test]
fn typed_prost_and_dynamic_reflection_agree() {
    let bin = fixture("data/s1.bin");
    let dynamic = pool::from_bin(&pool::descriptor("s1").unwrap(), &bin).unwrap();
    assert_eq!(pool::to_bin(&dynamic), bin);
    assert_eq!(
        pool::to_json_bytes(&dynamic, true, false).unwrap(),
        fixture("data/s1.json")
    );
    assert_eq!(
        pool::to_json_bytes(&dynamic, false, false).unwrap(),
        fixture("data/s1.plain.json")
    );
}

#[test]
fn json_fixture_satisfies_json_schema_and_mutations_do_not() {
    let good: Value = serde_json::from_slice(&fixture("data/s1.plain.json")).unwrap();
    json_contract::validate_v1(&good).expect("fixture JSON thuần phải hợp lệ");

    let mutate = |f: &dyn Fn(&mut Value)| {
        let mut v = good.clone();
        f(&mut v);
        v
    };
    let bad = [
        (
            "thiếu idempotencyKey",
            mutate(&|v| {
                v.as_object_mut().unwrap().remove("idempotencyKey");
            }),
        ),
        (
            "schemaVersion 2",
            mutate(&|v| v["schemaVersion"] = json!(2)),
        ),
        ("trường lạ", mutate(&|v| v["traceId"] = json!("abc"))),
        (
            "chỉ một posting",
            mutate(&|v| {
                v["postings"].as_array_mut().unwrap().pop();
            }),
        ),
        (
            "amountDong là chuỗi",
            mutate(&|v| v["postings"][0]["amountDong"] = json!("-25500")),
        ),
        (
            "amountDong ngoài khoảng an toàn",
            mutate(&|v| v["postings"][0]["amountDong"] = json!(typed::BIG_AMOUNT)),
        ),
    ];
    for (why, value) in bad {
        assert!(
            json_contract::validate_v1(&value).is_err(),
            "đáng lẽ bị từ chối: {why}"
        );
    }
}

#[test]
fn big_amount_is_exact_in_protobuf_but_not_in_f64_or_json_schema() {
    let big = typed::decode(&fixture("data/s1.big.bin")).unwrap();
    assert_eq!(big.postings[1].amount_dong, 9_007_199_254_740_993);
    // 2^53 + 1 không có trong f64: client đọc số JSON bằng double sẽ thấy 2^53.
    assert_eq!(
        9_007_199_254_740_993_i64 as f64 as i64,
        9_007_199_254_740_992
    );
    assert!(
        json_contract::validate_v1_bytes(&fixture("data/s1.big.plain.json"))
            .unwrap()
            .is_err()
    );
    // Proto3 JSON chuẩn viết int64 thành chuỗi nên không mất chữ số.
    let canonical = String::from_utf8(fixture("data/s1.big.json")).unwrap();
    assert!(canonical.contains("\"amountDong\":\"9007199254740993\""));
}

#[test]
fn rust_reader_matches_committed_matrix() {
    let cases = matrix::all_cases();
    let readings = matrix::rust_readings(&cases, &lab_dir().join("fixtures")).unwrap();
    let measured =
        consumers::assemble(&cases, &BTreeMap::from([("rust".to_owned(), readings)])).unwrap();
    let expected: Value = serde_json::from_slice(&fixture("expected_matrix.json")).unwrap();
    let diff = consumers::diff_expected(&measured, &expected);
    assert!(diff.is_empty(), "{diff:#?}");
}

#[test]
fn silent_losses_are_the_dangerous_cells() {
    let o = rust_outcomes();
    // Đổi số field: đọc được, không lỗi, số tiền thành 0.
    assert_eq!(
        o["renumber/backward/bin"],
        matrix::Outcome::Loss(vec![semantics::Sem::P0Amount, semantics::Sem::P1Amount])
    );
    // Dùng lại số 6: trace_id của bản cũ hiện thành region của bản mới.
    assert_eq!(
        o["reuse-number/backward/bin"],
        matrix::Outcome::Loss(vec![semantics::Sem::Region])
    );
    // Thêm trường: nhị phân an toàn cả hai chiều, JSON chặt vỡ chiều FORWARD.
    assert_eq!(o["add-field/forward/bin"], matrix::Outcome::Ok);
    assert!(matches!(
        o["add-field/forward/json"],
        matrix::Outcome::Err(_)
    ));
    // Enum thêm giá trị: nhị phân giữ số lạ, JSON chặt báo lỗi.
    assert_eq!(o["enum-add/forward/bin"], matrix::Outcome::OkUnknownEnum);
    assert!(matches!(
        o["enum-add/forward/json"],
        matrix::Outcome::Err(_)
    ));
}

#[test]
fn descriptor_checker_never_calls_a_broken_change_safe() {
    let (false_neg, false_pos) = compat::check_against_measured(&rust_outcomes()).unwrap();
    assert!(false_neg.is_empty(), "checker bỏ sót: {false_neg:?}");
    // Dương tính giả duy nhất đã biết: đổi tên giữ số vẫn đúng ở nhị phân.
    assert_eq!(false_pos, ["rename/backward/bin", "rename/forward/bin"]);
}

#[test]
fn descriptor_checker_names_the_rule_behind_each_change() {
    let rules = |old, new| -> Vec<&'static str> {
        compat::compare(old, new)
            .unwrap()
            .findings
            .iter()
            .map(|f| f.rule)
            .collect()
    };
    assert_eq!(rules("s1", "s2"), ["FIELD_ADDED", "FIELD_ADDED"]);
    assert_eq!(rules("s2", "s2r"), ["NAME_CHANGED_AT_NUMBER"]);
    assert_eq!(rules("s2", "s2n"), ["NUMBER_CHANGED"]);
    assert_eq!(rules("s2", "s3"), ["FIELD_REMOVED"]);
    assert_eq!(rules("s2", "s3x"), ["NAME_CHANGED_AT_NUMBER"]);
    assert_eq!(rules("s2", "s2e"), ["ENUM_VALUE_ADDED"]);

    // Gỡ có reserved thì không bị nhắc; mức FULL của thêm trường ổn ở nhị phân, vỡ ở JSON chặt.
    assert!(compat::compare("s2", "s3").unwrap().lints.is_empty());
    let add = compat::compare("s1", "s2").unwrap();
    assert!(add.is_compatible(Mode::Full, Wire::Binary));
    assert!(add.is_compatible(Mode::Backward, Wire::Json));
    assert!(!add.is_compatible(Mode::Forward, Wire::Json));
}

#[test]
fn protoc_refuses_to_reuse_a_reserved_number() {
    let message = compat::protoc_rejects_reserved_reuse().unwrap();
    assert!(message.contains("uses reserved number 6"), "{message}");
}

#[test]
fn rollout_order_follows_the_measurements() {
    use matrix::{Format, Rollout};
    let o = rust_outcomes();
    let pair = |name: &str| matrix::PAIRS.iter().find(|p| p.name == name).unwrap();
    // Thêm trường: nhị phân thứ tự nào cũng được, JSON chặt buộc nâng consumer trước.
    assert_eq!(
        matrix::rollout(pair("add-field"), Format::Bin, &o),
        Rollout::Any
    );
    assert_eq!(
        matrix::rollout(pair("add-field"), Format::Json, &o),
        Rollout::ConsumersFirst
    );
    // Đổi số field: không có thứ tự an toàn.
    assert_eq!(
        matrix::rollout(pair("renumber"), Format::Bin, &o),
        Rollout::Neither
    );
}

#[test]
fn typed_v1_consumer_reads_bytes_from_a_newer_producer() {
    // Consumer biên dịch với proto v1 đọc fixture do producer s2 (thêm trace_id, kind) viết:
    // trường lạ được bỏ qua, mọi trường v1 vẫn đúng. Đây là FORWARD ở dạng nhị phân.
    let event = typed::decode(&fixture("data/s2.bin")).unwrap();
    assert_eq!(event, typed::sample_capture());
}
