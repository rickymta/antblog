//! Kịch bản chạy thật: dựng server trên cổng do hệ điều hành chọn rồi gọi qua TCP.

use std::error::Error;
use std::net::SocketAddr;

use axum::Json;
use axum::Router;
use axum::routing::post;
use serde_json::Value;
use tokio::net::TcpListener;

use crate::dto::PostEntryRequest;
use crate::handlers::AppState;
use crate::probe::{Reply, send};
use crate::{OPENAPI_PATH, SWAGGER_PATH, app};

const CAPTURE: &str = r#"{"idempotencyKey":"trip:TN-0001:capture","postings":[
  {"account":"customer:CU-001","amountDong":-25500},
  {"account":"platform:revenue","amountDong":5100},
  {"account":"driver:DR-001","amountDong":20400}]}"#;

/// Ghi lại các điều kiện đã kiểm; mỗi điều kiện sai là một dòng lỗi.
#[derive(Default)]
pub struct Checks {
    failures: Vec<String>,
    total: usize,
}

impl Checks {
    fn ensure(&mut self, ok: bool, what: &str) {
        self.total += 1;
        if !ok {
            self.failures.push(what.to_string());
        }
    }

    /// Số điều kiện đã kiểm.
    pub fn total(&self) -> usize {
        self.total
    }

    /// Các điều kiện sai.
    pub fn failures(&self) -> &[String] {
        &self.failures
    }
}

async fn serve(router: Router) -> Result<SocketAddr, Box<dyn Error>> {
    // Cổng 0: hệ điều hành chọn cổng trống, chạy song song không đụng nhau.
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    Ok(addr)
}

fn show(label: &str, reply: &Reply) {
    let ct = reply.content_type.as_deref().unwrap_or("-");
    println!("{label} -> {} {ct}", reply.status);
}

fn type_slug(problem: &Value) -> &str {
    problem["type"]
        .as_str()
        .and_then(|t| t.rsplit('/').next())
        .unwrap_or("")
}

/// Chạy mọi kịch bản, in kết quả và trả về các điều kiện đã kiểm.
///
/// # Errors
/// Lỗi kết nối hoặc JSON sai dạng (không phải lỗi của điều kiện kiểm).
pub async fn run() -> Result<Checks, Box<dyn Error>> {
    let mut c = Checks::default();
    let addr = serve(app(AppState::default())).await?;
    println!("server nghe ở 127.0.0.1:<cổng do hệ điều hành chọn>");
    let json = Some("application/json");
    let entries = "/v1/entries";

    println!("\n== 1. Ghi bút toán hợp lệ, rồi replay ==");
    let created = send(addr, "POST", entries, json, CAPTURE).await?;
    show("POST /v1/entries", &created);
    println!("{}", created.body);
    c.ensure(created.status == 201, "ghi mới trả 201");
    c.ensure(
        created.json()?["entryId"] == "JE-000001",
        "entryId là JE-000001",
    );
    let replay = send(addr, "POST", entries, json, CAPTURE).await?;
    show("POST /v1/entries (lần 2)", &replay);
    c.ensure(replay.status == 200, "replay trả 200");
    c.ensure(replay.json()?["replayed"] == true, "replayed bằng true");

    println!("\n== 2. Cùng key, nội dung khác: 409 ==");
    let other = r#"{"idempotencyKey":"trip:TN-0001:capture","postings":[
      {"account":"customer:CU-001","amountDong":-1000},
      {"account":"driver:DR-001","amountDong":1000}]}"#;
    let conflict = send(addr, "POST", entries, json, other).await?;
    show("POST /v1/entries", &conflict);
    println!("{}", conflict.body);
    c.ensure(conflict.status == 409, "xung đột key trả 409");
    c.ensure(
        conflict.content_type.as_deref() == Some("application/problem+json"),
        "409 có media type problem+json",
    );
    c.ensure(
        type_slug(&conflict.json()?) == "idempotency-conflict",
        "type idempotency-conflict",
    );

    println!("\n== 3. Bút toán lệch: 422 từ ledger-core ==");
    let lech = r#"{"idempotencyKey":"trip:TN-0002:capture","postings":[
      {"account":"customer:CU-001","amountDong":-25500},
      {"account":"driver:DR-001","amountDong":20400}]}"#;
    let unbalanced = send(addr, "POST", entries, json, lech).await?;
    show("POST /v1/entries", &unbalanced);
    println!("{}", unbalanced.body);
    let p = unbalanced.json()?;
    c.ensure(unbalanced.status == 422, "lệch trả 422");
    c.ensure(type_slug(&p) == "unbalanced-entry", "type unbalanced-entry");
    c.ensure(
        p["detail"] == "bút toán lệch, tổng bằng -5.100 VND",
        "detail nêu tổng lệch",
    );
    c.ensure(p["instance"] == "/v1/entries", "instance là đường dẫn gốc");
    c.ensure(
        p["traceId"].as_str().is_some_and(|t| t.len() == 32),
        "traceId 32 ký tự",
    );

    println!("\n== 4. Validate thủ công gom nhiều lỗi vào errors ==");
    let bad = r#"{"idempotencyKey":"  ","postings":[
      {"account":"","amountDong":-100},
      {"account":"driver:DR-001","amountDong":0}]}"#;
    let invalid = send(addr, "POST", entries, json, bad).await?;
    show("POST /v1/entries", &invalid);
    let p = invalid.json()?;
    println!("detail: {}", p["detail"].as_str().unwrap_or(""));
    let errors = p["errors"].as_array().cloned().unwrap_or_default();
    for e in &errors {
        println!(
            "  {} [{}] {}",
            e["field"].as_str().unwrap_or(""),
            e["code"].as_str().unwrap_or(""),
            e["message"].as_str().unwrap_or("")
        );
    }
    c.ensure(invalid.status == 422, "sai hình dạng trả 422");
    c.ensure(
        type_slug(&p) == "validation-failed",
        "type validation-failed",
    );
    c.ensure(errors.len() == 3, "errors có đủ 3 lỗi");
    let trace_a = p["traceId"].clone();
    c.ensure(
        trace_a != unbalanced.json()?["traceId"],
        "traceId khác nhau giữa hai lỗi",
    );

    println!("\n== 5. JSON hỏng và sai Content-Type: rejection thành Problem ==");
    let broken = send(
        addr,
        "POST",
        entries,
        json,
        r#"{"idempotencyKey": "k", "postings": ["#,
    )
    .await?;
    show("body cụt", &broken);
    println!("{}", broken.body);
    c.ensure(broken.status == 400, "JSON hỏng trả 400");
    c.ensure(
        broken.content_type.as_deref() == Some("application/problem+json"),
        "400 có media type problem+json",
    );
    c.ensure(
        type_slug(&broken.json()?) == "malformed-json",
        "type malformed-json",
    );
    let missing = send(addr, "POST", entries, json, r#"{"idempotencyKey":"k"}"#).await?;
    show("thiếu postings", &missing);
    println!("{}", missing.json()?["detail"].as_str().unwrap_or(""));
    c.ensure(missing.status == 422, "thiếu trường trả 422");
    c.ensure(
        type_slug(&missing.json()?) == "invalid-json-body",
        "type invalid-json-body",
    );
    let no_ct = send(addr, "POST", entries, None, CAPTURE).await?;
    show("không có Content-Type", &no_ct);
    c.ensure(no_ct.status == 415, "thiếu Content-Type trả 415");
    c.ensure(
        no_ct.content_type.as_deref() == Some("application/problem+json"),
        "415 có media type problem+json",
    );

    println!("\n== 6. So với axum::Json nguyên bản ==");
    let plain = serve(Router::new().route(
        "/plain",
        post(|Json(_): Json<PostEntryRequest>| async { "ok" }),
    ))
    .await?;
    let stock = send(plain, "POST", "/plain", json, r#"{"idempotencyKey": "#).await?;
    show("axum::Json", &stock);
    println!("{}", stock.body);
    c.ensure(stock.status == 400, "axum::Json nguyên bản trả 400");
    c.ensure(
        stock
            .content_type
            .as_deref()
            .is_some_and(|t| t.starts_with("text/plain")),
        "axum::Json nguyên bản trả text/plain",
    );

    println!("\n== 7. Tra số dư và đường dẫn lạ ==");
    let balance = send(addr, "GET", "/v1/accounts/driver:DR-001/balance", None, "").await?;
    show("GET balance driver:DR-001", &balance);
    println!("{}", balance.body);
    c.ensure(balance.status == 200, "số dư trả 200");
    c.ensure(
        balance.json()?["balanceDong"] == 20_400,
        "số dư tài xế là 20.400",
    );
    let unknown = send(addr, "GET", "/v1/accounts/driver:DR-999/balance", None, "").await?;
    show("GET balance driver:DR-999", &unknown);
    c.ensure(unknown.status == 404, "tài khoản lạ trả 404");
    c.ensure(
        type_slug(&unknown.json()?) == "unknown-account",
        "type unknown-account",
    );
    for path in ["/v1/accounts/%20/balance", "/v1/accounts/%FF/balance"] {
        let bad = send(addr, "GET", path, None, "").await?;
        show(&format!("GET {path}"), &bad);
        c.ensure(bad.status == 400, "mã tài khoản sai dạng trả 400");
        c.ensure(
            bad.content_type.as_deref() == Some("application/problem+json"),
            "lỗi đường dẫn có media type problem+json",
        );
    }
    let no_route = send(addr, "GET", "/v2/entries", None, "").await?;
    show("GET /v2/entries", &no_route);
    c.ensure(no_route.status == 404, "đường dẫn lạ trả 404");
    c.ensure(
        type_slug(&no_route.json()?) == "route-not-found",
        "type route-not-found",
    );
    let wrong_method = send(addr, "GET", entries, None, "").await?;
    show("GET /v1/entries", &wrong_method);
    c.ensure(wrong_method.status == 405, "sai phương thức trả 405");
    c.ensure(
        wrong_method.content_type.as_deref() == Some("application/problem+json"),
        "405 có media type problem+json",
    );

    println!("\n== 8. OpenAPI và Swagger UI ==");
    let doc = send(addr, "GET", OPENAPI_PATH, None, "").await?;
    show(&format!("GET {OPENAPI_PATH}"), &doc);
    let spec = doc.json()?;
    let paths: Vec<&str> = spec["paths"]
        .as_object()
        .map_or_else(Vec::new, |m| m.keys().map(String::as_str).collect());
    let schemas: Vec<&str> = spec["components"]["schemas"]
        .as_object()
        .map_or_else(Vec::new, |m| m.keys().map(String::as_str).collect());
    println!(
        "openapi: {}; info.version: {}",
        spec["openapi"], spec["info"]["version"]
    );
    println!("paths: {paths:?}");
    println!("schemas: {schemas:?}");
    let post_422 = &spec["paths"]["/v1/entries"]["post"]["responses"]["422"]["content"];
    println!(
        "POST /v1/entries 422 content: {:?}",
        post_422.as_object().map(|m| m.keys().collect::<Vec<_>>())
    );
    println!(
        "Problem.required: {}",
        spec["components"]["schemas"]["Problem"]["required"]
    );
    println!(
        "POST /v1/entries 409: {}",
        spec["paths"]["/v1/entries"]["post"]["responses"]["409"]
    );
    println!(
        "PostingDto.additionalProperties: {}",
        spec["components"]["schemas"]["PostingDto"]["additionalProperties"]
    );
    c.ensure(
        spec["components"]["schemas"]["PostingDto"]["additionalProperties"] == false,
        "deny_unknown_fields thành additionalProperties: false",
    );
    c.ensure(spec["openapi"] == "3.1.0", "openapi là 3.1.0");
    c.ensure(
        paths == ["/v1/accounts/{id}/balance", "/v1/entries"],
        "spec có đủ hai đường dẫn /v1",
    );
    c.ensure(
        [
            "BalanceResponse",
            "EntryResponse",
            "FieldError",
            "PostEntryRequest",
            "PostingDto",
            "Problem",
        ]
        .iter()
        .all(|s| schemas.contains(s)),
        "components có đủ 6 schema",
    );
    c.ensure(
        post_422["application/problem+json"].is_object(),
        "422 công bố problem+json",
    );
    let ui = send(addr, "GET", &format!("{SWAGGER_PATH}/"), None, "").await?;
    show(&format!("GET {SWAGGER_PATH}/"), &ui);
    c.ensure(
        ui.status == 200 && ui.body.contains("swagger-ui"),
        "Swagger UI phục vụ trang HTML",
    );
    Ok(c)
}
