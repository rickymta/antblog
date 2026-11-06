//! Test HTTP với PostgreSQL thật. Không mở cổng mạng: `Router` là một `tower::Service`,
//! nên `oneshot` đưa thẳng một `Request` vào và nhận `Response` ra.

use std::convert::Infallible;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, Response, StatusCode, header};
use ledger_31_axum_http::app::{AppState, router};
use ledger_31_axum_http::lab_db::LabDb;
use ledger_31_axum_http::scenario::capture_body;
use ledger_31_axum_http::store::PgLedger;
use serde_json::Value;
use tower::{Service, ServiceExt};

// Hàm này chỉ cần "một Service nhận Request<Body>, trả Response<Body>, không bao giờ lỗi".
// Router thỏa hợp đồng đó, không cần biết nó là router.
async fn call<S>(service: S, request: Request<Body>) -> (StatusCode, Value)
where
    S: Service<Request<Body>, Response = Response<Body>, Error = Infallible>,
{
    let response = service
        .oneshot(request)
        .await
        .unwrap_or_else(|e| match e {});
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("đọc body");
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn post_json(body: &str) -> Request<Body> {
    Request::post("/v1/entries")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request hợp lệ")
}

fn get(path: &str) -> Request<Body> {
    Request::get(path)
        .body(Body::empty())
        .expect("request hợp lệ")
}

async fn with_app<F, Fut>(test: F)
where
    F: FnOnce(Router) -> Fut,
    Fut: Future<Output = ()>,
{
    let lab = LabDb::setup().await.expect("lab PostgreSQL");
    let app = router(AppState {
        ledger: PgLedger::new(lab.pool.clone()),
    });
    test(app).await;
    lab.cleanup().await.expect("dọn schema");
}

#[tokio::test]
async fn created_then_replayed_then_conflict() {
    with_app(|app| async move {
        let body = capture_body().to_string();
        let (status, json) = call(app.clone(), post_json(&body)).await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(json["entryId"], "JE-000001");

        let (status, json) = call(app.clone(), post_json(&body)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["replayed"], true);

        let altered = body.replace("5100", "5000").replace("20400", "20500");
        let (status, json) = call(app, post_json(&altered)).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(json["code"], "idempotency_conflict");
    })
    .await;
}

#[tokio::test]
async fn rejected_entry_leaves_no_trace() {
    with_app(|app| async move {
        let unbalanced = r#"{"idempotencyKey":"k1","postings":[
            {"account":"customer:CU-001","amountDong":-100},
            {"account":"driver:DR-001","amountDong":90}]}"#;
        let (status, json) = call(app.clone(), post_json(unbalanced)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json["code"], "unbalanced");

        let zero = r#"{"idempotencyKey":"k2","postings":[
            {"account":"customer:CU-001","amountDong":0},
            {"account":"driver:DR-001","amountDong":0}]}"#;
        let (status, json) = call(app.clone(), post_json(zero)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json["code"], "zero_amount");

        let (status, json) = call(app, get("/v1/accounts/customer:CU-001/balance")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["balanceDong"], 0);
    })
    .await;
}

#[tokio::test]
async fn body_errors_keep_axum_status() {
    with_app(|app| async move {
        // Cú pháp hỏng: 400.
        let (status, json) = call(app.clone(), post_json("{")).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["code"], "bad_body");

        // Sai hình dạng: trường lạ bị `deny_unknown_fields` từ chối ngay ở extractor.
        let unknown_field = r#"{"idempotencyKey":"k","postings":[],"memo":"x"}"#;
        let (status, json) = call(app.clone(), post_json(unknown_field)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json["code"], "bad_body");
        assert!(
            json["message"]
                .as_str()
                .unwrap_or("")
                .contains("unknown field")
        );

        // Thiếu Content-Type: extractor từ chối trước khi đọc body.
        let no_type = Request::post("/v1/entries")
            .body(Body::from("{}"))
            .expect("request hợp lệ");
        let (status, _) = call(app, no_type).await;
        assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    })
    .await;
}

// Biên của `DefaultBodyLimit`: đúng 2 MiB lọt giới hạn rồi hỏng cú pháp (400), thêm 1 byte là 413.
// Dùng `oneshot` vì qua TCP thật, server đóng kết nối khi chưa đọc hết body nên client có thể nhận
// reset thay vì 413.
#[tokio::test]
async fn body_limit_is_2_mib() {
    const LIMIT: usize = 2_097_152;
    with_app(|app| async move {
        for (len, want) in [
            (LIMIT, StatusCode::BAD_REQUEST),
            (LIMIT + 1, StatusCode::PAYLOAD_TOO_LARGE),
        ] {
            let (status, json) = call(app.clone(), post_json(&" ".repeat(len))).await;
            assert_eq!(status, want, "{len} byte");
            assert_eq!(json["code"], "bad_body");
        }
    })
    .await;
}

#[tokio::test]
async fn unknown_account_is_404_and_clearing_is_zero() {
    with_app(|app| async move {
        let (status, json) = call(app.clone(), get("/v1/accounts/customer:CU-999/balance")).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json["code"], "account_not_found");

        let (status, json) = call(app, get("/v1/accounts/psp:vixanh:clearing/balance")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["balanceDong"], 0);
    })
    .await;
}

// Handler và Service viết tay nằm cạnh nhau trong cùng một Router, gọi qua cùng một giao diện.
#[tokio::test]
async fn hand_written_service_and_unknown_route() {
    with_app(|app| async move {
        let (status, json) = call(app.clone(), get("/healthz")).await;
        assert_eq!(
            (status, &json["status"]),
            (StatusCode::OK, &Value::from("ok"))
        );

        let (status, json) = call(app, get("/v2/anything")).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json["code"], "route_not_found");
    })
    .await;
}
