//! API test với `oneshot` trên PostgreSQL thật: mỗi ca một schema `lab_<uuid>`.

use axum::http::{Method, StatusCode, header};
use ledger_api::testkit::{TestApp, send};
use serde_json::{Value, json};

/// Body mẫu chung với client C#: bút toán `trip:TN-0001:capture`.
const CAPTURE: &str = include_str!("../contracts/post-entry.request.json");
const POST: &str = "/v1/entries";

fn problem_type(body: &Value) -> &str {
    body["type"].as_str().expect("problem có trường type")
}

// Cùng tiền tố `type` với bài 32.
fn type_of(slug: &str) -> String {
    format!("https://toi-noi.example/problems/{slug}")
}

#[tokio::test]
async fn post_creates_201_then_replays_200() {
    let app = TestApp::start().await;
    let router = app.router();
    let token = app.full_token();

    let first = send(&router, Method::POST, POST, Some(&token), Some(CAPTURE)).await;
    assert_eq!(first.status, StatusCode::CREATED);
    assert_eq!(first.json()["entryId"], "JE-000001");
    assert_eq!(first.json()["replayed"], false);

    let again = send(&router, Method::POST, POST, Some(&token), Some(CAPTURE)).await;
    assert_eq!(again.status, StatusCode::OK);
    assert_eq!(again.json()["entryId"], "JE-000001");
    assert_eq!(again.json()["replayed"], true);

    // Số dư không nhân đôi sau replay.
    let bal = send(
        &router,
        Method::GET,
        "/v1/accounts/driver:DR-001/balance",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(bal.status, StatusCode::OK);
    assert_eq!(bal.json()["balanceDong"], 20_400);
    app.close().await;
}

#[tokio::test]
async fn same_key_with_other_amounts_is_409_and_changes_nothing() {
    let app = TestApp::start().await;
    let router = app.router();
    let token = app.full_token();
    send(&router, Method::POST, POST, Some(&token), Some(CAPTURE)).await;

    let other = CAPTURE.replace("20400", "20300").replace("5100", "5200");
    let reply = send(&router, Method::POST, POST, Some(&token), Some(&other)).await;

    assert_eq!(reply.status, StatusCode::CONFLICT);
    assert_eq!(
        reply.headers[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_eq!(problem_type(&reply.json()), type_of("idempotency-conflict"));
    assert_eq!(reply.json()["instance"], POST);
    assert_eq!(reply.json()["traceId"].as_str().map(str::len), Some(32));
    let entries: i64 = sqlx::query_scalar("select count(*) from journal_entries")
        .fetch_one(&app.db.pool)
        .await
        .unwrap();
    assert_eq!(entries, 1, "bút toán xung đột không được ghi");
    app.close().await;
}

#[tokio::test]
async fn invalid_entries_are_422_with_a_stable_type() {
    let app = TestApp::start().await;
    let router = app.router();
    let token = app.full_token();
    let cases = [
        (
            "lệch 100 đồng",
            json!({"idempotencyKey": "k1", "postings": [
                {"account": "customer:CU-001", "amountDong": -25500},
                {"account": "driver:DR-001", "amountDong": 25400}]}),
            "unbalanced-entry",
        ),
        (
            "một posting",
            json!({"idempotencyKey": "k2", "postings": [
                {"account": "customer:CU-001", "amountDong": -1}]}),
            "too-few-postings",
        ),
        (
            "posting 0 đồng",
            json!({"idempotencyKey": "k3", "postings": [
                {"account": "customer:CU-001", "amountDong": 0},
                {"account": "driver:DR-001", "amountDong": 0}]}),
            "validation-failed",
        ),
        (
            "tài khoản chưa có",
            json!({"idempotencyKey": "k4", "postings": [
                {"account": "customer:CU-999", "amountDong": -5},
                {"account": "driver:DR-001", "amountDong": 5}]}),
            "unknown-account",
        ),
        (
            "tràn i64",
            json!({"idempotencyKey": "k5", "postings": [
                {"account": "customer:CU-001", "amountDong": i64::MAX},
                {"account": "driver:DR-001", "amountDong": 1}]}),
            "amount-overflow",
        ),
    ];
    for (name, body, expected) in cases {
        let reply = send(
            &router,
            Method::POST,
            POST,
            Some(&token),
            Some(&body.to_string()),
        )
        .await;
        assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY, "{name}");
        assert_eq!(problem_type(&reply.json()), type_of(expected), "{name}");
    }
    // Lỗi theo trường nằm trong `errors`, gom đủ, như bài 32.
    let zero = json!({"idempotencyKey": "k6", "postings": [
        {"account": "customer:CU-001", "amountDong": 0},
        {"account": " ", "amountDong": 5}]});
    let reply = send(
        &router,
        Method::POST,
        POST,
        Some(&token),
        Some(&zero.to_string()),
    )
    .await;
    let errors = reply.json()["errors"].clone();
    assert_eq!(errors[0]["field"], "postings[0].amountDong");
    assert_eq!(errors[0]["code"], "zero_amount");
    assert_eq!(errors[1]["field"], "postings[1].account");
    assert_eq!(errors[1]["code"], "blank");

    // Không ca nào để lại dấu vết trong sổ (ca unknown-account đã rollback cả header).
    let entries: i64 = sqlx::query_scalar("select count(*) from journal_entries")
        .fetch_one(&app.db.pool)
        .await
        .unwrap();
    assert_eq!(entries, 0);
    app.close().await;
}

#[tokio::test]
async fn broken_json_is_400_but_wrong_shape_is_422() {
    let app = TestApp::start().await;
    let router = app.router();
    let token = app.full_token();

    // Cú pháp hỏng (thiếu dấu đóng): 400.
    let broken = send(
        &router,
        Method::POST,
        POST,
        Some(&token),
        Some(r#"{"idempotencyKey": "k", "postings": ["#),
    )
    .await;
    assert_eq!(broken.status, StatusCode::BAD_REQUEST);
    assert_eq!(problem_type(&broken.json()), type_of("malformed-json"));

    // JSON đúng cú pháp nhưng sai hình dạng: 422.
    let shapes = [
        ("thiếu postings", r#"{"idempotencyKey": "k"}"#),
        (
            "số viết dạng chuỗi",
            r#"{"idempotencyKey":"k","postings":[{"account":"a","amountDong":"5"},{"account":"b","amountDong":-5}]}"#,
        ),
        (
            "trường lạ",
            r#"{"idempotencyKey":"k","postings":[],"memo":"x"}"#,
        ),
        (
            "số thực",
            r#"{"idempotencyKey":"k","postings":[{"account":"a","amountDong":1.5}]}"#,
        ),
    ];
    for (name, body) in shapes {
        let reply = send(&router, Method::POST, POST, Some(&token), Some(body)).await;
        assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY, "{name}");
        assert_eq!(
            problem_type(&reply.json()),
            type_of("invalid-json-body"),
            "{name}"
        );
    }
    app.close().await;
}

#[tokio::test]
async fn body_without_json_content_type_is_415() {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let app = TestApp::start().await;
    let request = Request::builder()
        .method(Method::POST)
        .uri(POST)
        .header("authorization", format!("Bearer {}", app.full_token()))
        .body(Body::from(CAPTURE))
        .unwrap();
    let response = app.router().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    app.close().await;
}

#[tokio::test]
async fn missing_or_bad_tokens_are_401() {
    let app = TestApp::start().await;
    let router = app.router();
    let forged = app.forged_token();
    let expired = app.expired_token();
    let cases: [(&str, Option<&str>); 3] = [
        ("không có token", None),
        ("khoá ký khác", Some(forged.as_str())),
        ("token hết hạn", Some(expired.as_str())),
    ];
    for (name, token) in cases {
        let reply = send(&router, Method::POST, POST, token, Some(CAPTURE)).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{name}");
        assert_eq!(reply.headers[header::WWW_AUTHENTICATE], "Bearer", "{name}");
        assert_eq!(
            problem_type(&reply.json()),
            type_of("unauthenticated"),
            "{name}"
        );
    }
    // Header sai dạng (không phải Bearer) cũng là 401.
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;
    let request = Request::builder()
        .method(Method::GET)
        .uri("/v1/accounts/driver:DR-001/balance")
        .header("authorization", "Basic dXNlcjpwYXNz")
        .body(Body::empty())
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Xác thực chạy TRƯỚC khi đọc body: body hỏng mà không token vẫn là 401, không phải 400.
    let reply = send(&router, Method::POST, POST, None, Some("{hỏng")).await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);

    // Thân 401 giống nhau cho mọi lý do (trừ traceId sinh mới): không nói token sai ở đâu.
    let (mut a, mut b) = (
        send(&router, Method::POST, POST, Some(&forged), Some(CAPTURE))
            .await
            .json(),
        send(&router, Method::POST, POST, Some(&expired), Some(CAPTURE))
            .await
            .json(),
    );
    a["traceId"] = Value::Null;
    b["traceId"] = Value::Null;
    assert_eq!(a, b);
    app.close().await;
}

#[tokio::test]
async fn valid_token_without_the_scope_is_403() {
    let app = TestApp::start().await;
    let router = app.router();
    let read_only = app.token(&["ledger:read"]);
    let write_only = app.token(&["ledger:write"]);

    let post = send(&router, Method::POST, POST, Some(&read_only), Some(CAPTURE)).await;
    assert_eq!(post.status, StatusCode::FORBIDDEN);
    assert_eq!(problem_type(&post.json()), type_of("forbidden"));

    // Kiểm quyền chạy TRƯỚC khi đọc body: body hỏng mà thiếu quyền vẫn là 403, không phải 400.
    let broken = send(&router, Method::POST, POST, Some(&read_only), Some("{hỏng")).await;
    assert_eq!(broken.status, StatusCode::FORBIDDEN);

    let get = send(
        &router,
        Method::GET,
        "/v1/accounts/driver:DR-001/balance",
        Some(&write_only),
        None,
    )
    .await;
    assert_eq!(get.status, StatusCode::FORBIDDEN);

    let entries: i64 = sqlx::query_scalar("select count(*) from journal_entries")
        .fetch_one(&app.db.pool)
        .await
        .unwrap();
    assert_eq!(entries, 0, "403 không được ghi gì");
    app.close().await;
}

#[tokio::test]
async fn balance_is_404_for_unknown_account_and_zero_for_untouched_one() {
    let app = TestApp::start().await;
    let router = app.router();
    let token = app.full_token();

    let unknown = send(
        &router,
        Method::GET,
        "/v1/accounts/driver:DR-404/balance",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    assert_eq!(problem_type(&unknown.json()), type_of("unknown-account"));

    // Mã rỗng trên đường dẫn là tham số sai dạng: 400, trong body nó là 422.
    let blank = send(
        &router,
        Method::GET,
        "/v1/accounts/%20/balance",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(blank.status, StatusCode::BAD_REQUEST);
    assert_eq!(problem_type(&blank.json()), type_of("invalid-account-id"));

    let untouched = send(
        &router,
        Method::GET,
        "/v1/accounts/psp:vixanh:clearing/balance",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(untouched.status, StatusCode::OK);
    assert_eq!(untouched.json()["balanceDong"], 0);
    app.close().await;
}

#[tokio::test]
async fn unknown_route_and_wrong_method_are_problems_too() {
    let app = TestApp::start().await;
    let router = app.router();

    let missing = send(&router, Method::GET, "/v1/nope", None, None).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(problem_type(&missing.json()), type_of("route-not-found"));
    assert_eq!(missing.json()["instance"], "/v1/nope");

    let wrong = send(&router, Method::GET, POST, None, None).await;
    assert_eq!(wrong.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(
        wrong.headers[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_eq!(problem_type(&wrong.json()), type_of("method-not-allowed"));
    app.close().await;
}
