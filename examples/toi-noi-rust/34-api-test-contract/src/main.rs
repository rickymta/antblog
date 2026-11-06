//! Chạy kịch bản API: cùng một router, gọi bằng `oneshot` rồi bằng server thật.
//! `--verify` kiểm các giá trị in trong bài và thoát mã 1 nếu có chỗ sai.

use std::process::ExitCode;

use axum::http::Method;
use ledger_api::api::{ApiDoc, OPENAPI_PATH};
use ledger_api::rawhttp;
use ledger_api::testkit::{TestApp, send};
use serde_json::Value;
use utoipa::OpenApi;

const REQUEST: &str = include_str!("../contracts/post-entry.request.json");
const RESPONSE: &str = include_str!("../contracts/post-entry.response.json");

/// Gom kết quả kiểm: in từng dòng, nhớ dòng sai.
struct Report {
    failed: Vec<String>,
}

impl Report {
    fn check(&mut self, label: &str, ok: bool) {
        println!("{} {label}", if ok { "OK  " } else { "FAIL" });
        if !ok {
            self.failed.push(label.to_string());
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let verify = std::env::args().any(|a| a == "--verify");
    if std::env::args().any(|a| a == "--measure") {
        return match measure().await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("LỖI: {e:#}");
                ExitCode::FAILURE
            }
        };
    }
    match run().await {
        Ok(report) if report.failed.is_empty() => {
            if verify {
                println!("VERIFY OK");
            }
            ExitCode::SUCCESS
        }
        Ok(report) => {
            eprintln!(
                "LỖI: {} kiểm tra sai: {:?}",
                report.failed.len(),
                report.failed
            );
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("LỖI: {e:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> anyhow::Result<Report> {
    let mut r = Report { failed: Vec::new() };
    let app = TestApp::try_start().await?;
    let router = app.router();
    let token = app.full_token();
    let expected: Value = serde_json::from_str(RESPONSE)?;

    // 1. oneshot: không socket, chỉ Request vào và Response ra.
    println!("-- oneshot (không socket) --");
    let created = send(
        &router,
        Method::POST,
        "/v1/entries",
        Some(&token),
        Some(REQUEST),
    )
    .await;
    println!(
        "POST /v1/entries -> {} {}",
        created.status.as_u16(),
        created.json()
    );
    r.check(
        "201 và thân bằng file hợp đồng",
        created.status == 201 && created.json() == expected,
    );
    let replay = send(
        &router,
        Method::POST,
        "/v1/entries",
        Some(&token),
        Some(REQUEST),
    )
    .await;
    println!(
        "POST lại cùng key -> {} replayed={}",
        replay.status.as_u16(),
        replay.json()["replayed"]
    );
    r.check(
        "replay trả 200",
        replay.status == 200 && replay.json()["replayed"] == true,
    );

    let conflict_body = REQUEST.replace("20400", "20300").replace("5100", "5200");
    let conflict = send(
        &router,
        Method::POST,
        "/v1/entries",
        Some(&token),
        Some(&conflict_body),
    )
    .await;
    println!(
        "POST cùng key khác số tiền -> {} {}",
        conflict.status.as_u16(),
        conflict.json()["type"]
    );
    r.check("khác nội dung trả 409", conflict.status == 409);

    let unbalanced = REQUEST
        .replace("20400", "20300")
        .replace("trip:TN-0001:capture", "k-lech");
    let bad = send(
        &router,
        Method::POST,
        "/v1/entries",
        Some(&token),
        Some(&unbalanced),
    )
    .await;
    println!(
        "POST bút toán lệch -> {} {}",
        bad.status.as_u16(),
        bad.json()["type"]
    );
    r.check("lệch trả 422", bad.status == 422);

    let broken = send(
        &router,
        Method::POST,
        "/v1/entries",
        Some(&token),
        Some("{\"idempotencyKey\": "),
    )
    .await;
    println!(
        "POST JSON hỏng -> {} {}",
        broken.status.as_u16(),
        broken.json()["type"]
    );
    r.check("JSON hỏng trả 400", broken.status == 400);

    let anon = send(&router, Method::POST, "/v1/entries", None, Some(REQUEST)).await;
    let read_only = app.token(&["ledger:read"]);
    let forbidden = send(
        &router,
        Method::POST,
        "/v1/entries",
        Some(&read_only),
        Some(REQUEST),
    )
    .await;
    println!(
        "không token -> {}; token chỉ đọc -> {}",
        anon.status.as_u16(),
        forbidden.status.as_u16()
    );
    r.check(
        "không token 401, thiếu quyền 403",
        anon.status == 401 && forbidden.status == 403,
    );

    // 2. Server thật trên 127.0.0.1:0, cổng do hệ điều hành chọn.
    println!("-- server thật --");
    let (addr, server) = rawhttp::serve(router.clone()).await?;
    println!(
        "lắng nghe trên 127.0.0.1:<cổng do OS cấp> (cổng khác 0: {})",
        addr.port() != 0
    );
    let auth = format!("Bearer {token}");
    let path = "/v1/accounts/driver:DR-001/balance";
    let socket = rawhttp::request(addr, "GET", path, &[("Authorization", &auth)], None).await?;
    let mut body = socket.json()?;
    println!(
        "GET {path} -> {} account={} balanceDong={}",
        socket.status, body["account"], body["balanceDong"]
    );
    r.check(
        "socket: số dư tài xế 20400",
        socket.status == 200 && body["balanceDong"] == 20_400,
    );
    body["asOf"] = Value::Null;
    let mut via_oneshot = send(&router, Method::GET, path, Some(&token), None)
        .await
        .json();
    via_oneshot["asOf"] = Value::Null;
    r.check("socket và oneshot cho cùng thân", body == via_oneshot);

    let oneshot_get = send(&router, Method::GET, path, Some(&token), None).await;
    r.check(
        "header date/content-length chỉ có trên socket",
        oneshot_get.headers.get("date").is_none()
            && socket.header("date").is_some()
            && socket.header("content-length").is_some(),
    );
    println!(
        "header chỉ có trên socket: date={}, content-length={}",
        socket.header("date").is_some(),
        socket.header("content-length").unwrap_or("?")
    );
    server.abort();

    // 3. Hợp đồng và OpenAPI.
    println!("-- hợp đồng --");
    let paths: Vec<String> = ApiDoc::openapi().paths.paths.keys().cloned().collect();
    println!("OpenAPI paths: {paths:?}");
    r.check("OpenAPI có 2 path", paths.len() == 2);
    println!("hợp đồng JSON: contracts/post-entry.{{request,response,unbalanced.problem}}.json");

    app.close().await;
    Ok(r)
}

/// Đo chi phí hai đường vào trên `GET /api-docs/openapi.json` (không chạm database).
/// Số đo trên máy chạy lab, không phải SLA.
async fn measure() -> anyhow::Result<()> {
    use std::time::Instant;
    const N: u32 = 2_000;
    let router = ledger_api::testkit::offline_router();

    let start = Instant::now();
    for _ in 0..N {
        let reply = send(&router, Method::GET, OPENAPI_PATH, None, None).await;
        assert_eq!(reply.status, 200);
    }
    let oneshot = start.elapsed() / N;

    let (addr, server) = rawhttp::serve(router).await?;
    let start = Instant::now();
    for _ in 0..N {
        let reply = rawhttp::request(addr, "GET", OPENAPI_PATH, &[], None).await?;
        assert_eq!(reply.status, 200);
    }
    let socket = start.elapsed() / N;
    server.abort();

    println!("oneshot: {oneshot:?} mỗi request ({N} lần)");
    println!("socket 127.0.0.1 (kết nối mới mỗi lần): {socket:?} mỗi request ({N} lần)");
    Ok(())
}
