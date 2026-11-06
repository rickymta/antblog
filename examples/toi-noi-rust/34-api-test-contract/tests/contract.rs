//! Hợp đồng thủ công với client C#: hai file JSON dùng chung, mỗi phía tự kiểm.
//! Phía C# nằm ở `dotnet-client/` và đọc đúng các file trong `contracts/`.

use axum::http::{Method, StatusCode};
use ledger_api::api::{EntryResponse, PostEntryRequest};
use ledger_api::problem::Problem;
use ledger_api::testkit::{TestApp, send};
use serde_json::Value;

const REQUEST: &str = include_str!("../contracts/post-entry.request.json");
const RESPONSE: &str = include_str!("../contracts/post-entry.response.json");
const UNBALANCED: &str = include_str!("../contracts/post-entry.unbalanced.problem.json");

#[test]
fn contract_files_parse_into_the_server_types() {
    let request: PostEntryRequest = serde_json::from_str(REQUEST).expect("request đúng hợp đồng");
    let response: EntryResponse = serde_json::from_str(RESPONSE).expect("response đúng hợp đồng");

    let sum: i64 = request.postings.iter().map(|p| p.amount_dong).sum();
    assert_eq!(sum, 0, "bút toán mẫu phải cân");
    assert_eq!(response.idempotency_key, request.idempotency_key);
    assert_eq!(response.postings, request.postings);
}

#[tokio::test]
async fn server_answers_the_request_file_with_exactly_the_response_file() {
    let app = TestApp::start().await;
    let reply = send(
        &app.router(),
        Method::POST,
        "/v1/entries",
        Some(&app.full_token()),
        Some(REQUEST),
    )
    .await;

    assert_eq!(reply.status, StatusCode::CREATED);
    let expected: Value = serde_json::from_str(RESPONSE).unwrap();
    assert_eq!(
        reply.json(),
        expected,
        "thân trả về phải bằng file hợp đồng"
    );
    app.close().await;
}

#[tokio::test]
async fn server_answers_an_unbalanced_entry_with_the_problem_file() {
    let app = TestApp::start().await;
    let lech = REQUEST.replace("20400", "15300"); // tổng -5.100
    let reply = send(
        &app.router(),
        Method::POST,
        "/v1/entries",
        Some(&app.full_token()),
        Some(&lech),
    )
    .await;

    assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(reply.headers["content-type"], "application/problem+json");
    let mut actual = reply.json();
    let mut expected: Value = serde_json::from_str(UNBALANCED).unwrap();
    // traceId sinh mới mỗi lần: hợp đồng là dạng 32 hex, không phải giá trị.
    let trace = actual["traceId"].as_str().expect("có traceId").to_string();
    assert!(trace.len() == 32 && trace.chars().all(|c| c.is_ascii_hexdigit()));
    actual["traceId"] = Value::Null;
    expected["traceId"] = Value::Null;
    assert_eq!(actual, expected, "thân lỗi phải bằng file hợp đồng");
    serde_json::from_str::<Problem>(UNBALANCED).expect("file đúng kiểu Problem");
    app.close().await;
}

#[test]
fn request_is_strict_about_unknown_fields() {
    // Phía nhận chặt: trường lạ là lỗi. Client C# thêm trường cũng bị 422 chứ không bị bỏ qua.
    let with_extra = REQUEST.replacen('{', r#"{"memo": "x","#, 1);
    assert!(serde_json::from_str::<PostEntryRequest>(&with_extra).is_err());
}

#[test]
fn response_is_the_shape_csharp_expects_camel_case_keys_only() {
    let value: Value = serde_json::from_str(RESPONSE).unwrap();
    let mut keys: Vec<&str> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, ["entryId", "idempotencyKey", "postings", "replayed"]);
}
