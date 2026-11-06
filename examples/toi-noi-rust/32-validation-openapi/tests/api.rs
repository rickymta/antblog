//! Test tầng HTTP bằng `tower::ServiceExt::oneshot`: không mở cổng, mỗi test một sổ cái riêng.

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use http_body_util::BodyExt;
use ledger_32_validation_openapi::handlers::AppState;
use ledger_32_validation_openapi::openapi::ApiDoc;
use ledger_32_validation_openapi::{OPENAPI_PATH, app, verify};
use serde_json::{Value, json};
use tower::ServiceExt;
use utoipa::OpenApi;

async fn call(
    router: &Router,
    method: Method,
    uri: &str,
    body: Option<&str>,
) -> (StatusCode, String, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    let request = builder
        .body(Body::from(body.unwrap_or("").to_string()))
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, content_type, value)
}

fn capture(key: &str) -> String {
    json!({
        "idempotencyKey": key,
        "postings": [
            {"account": "customer:CU-001", "amountDong": -25_500},
            {"account": "platform:revenue", "amountDong": 5_100},
            {"account": "driver:DR-001", "amountDong": 20_400},
        ]
    })
    .to_string()
}

#[tokio::test]
async fn created_then_replayed() {
    let router = app(AppState::default());
    let body = capture("trip:TN-0001:capture");
    let (first, _, v) = call(&router, Method::POST, "/v1/entries", Some(&body)).await;
    assert_eq!(first, StatusCode::CREATED);
    assert_eq!(v["entryId"], "JE-000001");
    let (second, _, v) = call(&router, Method::POST, "/v1/entries", Some(&body)).await;
    assert_eq!(second, StatusCode::OK);
    assert_eq!(v["replayed"], true);
}

#[tokio::test]
async fn every_error_is_problem_json_with_the_documented_status() {
    let router = app(AppState::default());
    let ok = capture("trip:TN-0001:capture");
    call(&router, Method::POST, "/v1/entries", Some(&ok)).await;
    let changed = json!({"idempotencyKey": "trip:TN-0001:capture", "postings": [
        {"account": "a", "amountDong": -1}, {"account": "b", "amountDong": 1}]})
    .to_string();
    let unbalanced = json!({"idempotencyKey": "k2", "postings": [
        {"account": "a", "amountDong": -2}, {"account": "b", "amountDong": 1}]})
    .to_string();
    let overflow = json!({"idempotencyKey": "k3", "postings": [
        {"account": "a", "amountDong": i64::MAX}, {"account": "b", "amountDong": 1}]})
    .to_string();
    let single = json!({"idempotencyKey": "k4", "postings": [
        {"account": "a", "amountDong": 1}]})
    .to_string();
    let blank_key = json!({"idempotencyKey": " ", "postings": []}).to_string();
    let cases: [(&str, &str, StatusCode, &str); 7] = [
        (
            "conflict",
            &changed,
            StatusCode::CONFLICT,
            "idempotency-conflict",
        ),
        (
            "unbalanced",
            &unbalanced,
            StatusCode::UNPROCESSABLE_ENTITY,
            "unbalanced-entry",
        ),
        (
            "overflow",
            &overflow,
            StatusCode::UNPROCESSABLE_ENTITY,
            "amount-overflow",
        ),
        (
            "one posting",
            &single,
            StatusCode::UNPROCESSABLE_ENTITY,
            "too-few-postings",
        ),
        (
            "blank key",
            &blank_key,
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation-failed",
        ),
        ("syntax", "{", StatusCode::BAD_REQUEST, "malformed-json"),
        (
            "unknown field",
            r#"{"idempotencyKey":"k","postings":[],"memo":"x"}"#,
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid-json-body",
        ),
    ];
    for (name, body, status, slug) in cases {
        let (got, content_type, v) = call(&router, Method::POST, "/v1/entries", Some(body)).await;
        assert_eq!(got, status, "{name}");
        assert_eq!(content_type, "application/problem+json", "{name}");
        assert_eq!(v["status"], status.as_u16(), "{name}");
        assert_eq!(
            v["type"],
            format!("https://toi-noi.example/problems/{slug}"),
            "{name}"
        );
        assert_eq!(v["instance"], "/v1/entries", "{name}");
        assert!(
            v["traceId"].as_str().is_some_and(|t| t.len() == 32),
            "{name}"
        );
    }
}

#[tokio::test]
async fn missing_content_type_is_415_problem() {
    let router = app(AppState::default());
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/entries")
        .body(Body::from(capture("k")))
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
}

// Biên của `DefaultBodyLimit`: đúng 2 MiB lọt giới hạn rồi hỏng cú pháp (400), thêm 1 byte là 413.
// Dùng `oneshot` vì qua TCP thật, server đóng kết nối khi chưa đọc hết body nên client có thể nhận
// reset thay vì 413.
#[tokio::test]
async fn body_limit_is_2_mib_and_over_is_problem() {
    const LIMIT: usize = 2_097_152;
    let router = app(AppState::default());
    let at_limit = " ".repeat(LIMIT);
    let (status, _, v) = call(&router, Method::POST, "/v1/entries", Some(&at_limit)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(v["type"], "https://toi-noi.example/problems/malformed-json");
    let over = " ".repeat(LIMIT + 1);
    let (status, content_type, v) = call(&router, Method::POST, "/v1/entries", Some(&over)).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(content_type, "application/problem+json");
    assert_eq!(
        v["type"],
        "https://toi-noi.example/problems/unreadable-body"
    );
}

// RFC 9110: 405 phải có `Allow`; fallback riêng của router không được làm mất header này.
#[tokio::test]
async fn method_not_allowed_is_problem_with_allow_header() {
    let router = app(AppState::default());
    let request = Request::builder()
        .method(Method::GET)
        .uri("/v1/entries")
        .body(Body::empty())
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(response.headers()[header::ALLOW], "POST");
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
}

#[tokio::test]
async fn validation_errors_list_every_field_and_omit_errors_otherwise() {
    let router = app(AppState::default());
    let body = json!({"idempotencyKey": "", "postings": [
        {"account": " ", "amountDong": 0}, {"account": "ok", "amountDong": 1}]})
    .to_string();
    let (_, _, v) = call(&router, Method::POST, "/v1/entries", Some(&body)).await;
    let fields: Vec<&str> = v["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["field"].as_str().unwrap())
        .collect();
    assert_eq!(
        fields,
        [
            "idempotencyKey",
            "postings[0].account",
            "postings[0].amountDong"
        ]
    );
    // Lỗi miền không có `errors`: thành phần mở rộng chỉ xuất hiện khi có nội dung.
    let unbalanced = json!({"idempotencyKey": "k", "postings": [
        {"account": "a", "amountDong": -2}, {"account": "b", "amountDong": 1}]})
    .to_string();
    let (_, _, v) = call(&router, Method::POST, "/v1/entries", Some(&unbalanced)).await;
    assert!(v.get("errors").is_none());
}

#[tokio::test]
async fn openapi_paths_are_exactly_the_routes_that_exist() {
    let spec = serde_json::to_value(ApiDoc::openapi()).unwrap();
    assert_eq!(spec["openapi"], "3.1.0");
    let router = app(AppState::default());
    let mut seen = Vec::new();
    for (path, item) in spec["paths"].as_object().unwrap() {
        seen.push(path.as_str());
        let uri = path.replace("{id}", "driver:DR-001");
        for method in item.as_object().unwrap().keys() {
            let method = Method::from_bytes(method.to_uppercase().as_bytes()).unwrap();
            let body = (method == Method::POST).then_some("{}");
            let (status, _, v) = call(&router, method, &uri, body).await;
            let slug = v["type"].as_str().unwrap_or("");
            assert_ne!(status, StatusCode::METHOD_NOT_ALLOWED, "{uri}");
            assert!(!slug.ends_with("route-not-found"), "{uri}");
        }
    }
    seen.sort_unstable();
    assert_eq!(seen, ["/v1/accounts/{id}/balance", "/v1/entries"]);
}

#[tokio::test]
async fn problem_schema_documents_the_extension_members() {
    let spec = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let problem = &spec["components"]["schemas"]["Problem"];
    let props: Vec<&str> = problem["properties"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    for name in [
        "type", "title", "status", "detail", "instance", "errors", "traceId",
    ] {
        assert!(props.contains(&name), "thiếu {name}: {props:?}");
    }
    let required: Vec<&str> = problem["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(required.contains(&"traceId"));
    assert!(!required.contains(&"errors"));
}

#[tokio::test]
async fn served_document_equals_generated_document() {
    let router = app(AppState::default());
    let (status, _, served) = call(&router, Method::GET, OPENAPI_PATH, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(served, serde_json::to_value(ApiDoc::openapi()).unwrap());
}

#[tokio::test]
async fn full_scenario_over_real_tcp_passes_all_checks() {
    let checks = verify::run().await.unwrap();
    assert!(checks.failures().is_empty(), "{:?}", checks.failures());
    assert!(checks.total() > 30);
}
