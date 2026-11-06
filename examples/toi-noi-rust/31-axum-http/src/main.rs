//! `cargo run`: server axum thật trên `127.0.0.1:0`, client gọi chuỗi request mẫu và in kết quả.
//! `cargo run -- --verify`: kiểm các giá trị kỳ vọng trong bài.
//! `cargo run -- --serve`: chạy server đến khi Ctrl-C để tự gọi bằng curl.

use std::process::ExitCode;

use anyhow::{Result, ensure};
use ledger_31_axum_http::lab_db::LabDb;
use ledger_31_axum_http::scenario::{self, Exchange, capture_body, serve_lab};
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::Value;

fn find<'a>(all: &'a [Exchange], label: &str) -> &'a Exchange {
    all.iter()
        .find(|e| e.label == label)
        .unwrap_or_else(|| panic!("thiếu bước {label}"))
}

fn expect(all: &[Exchange], label: &str, status: u16, code: Option<&str>) -> Result<()> {
    let e = find(all, label);
    ensure!(
        e.status == status,
        "{label}: status {} (cần {status}), body {}",
        e.status,
        e.body
    );
    if let Some(code) = code {
        ensure!(
            e.body["code"] == code,
            "{label}: code {} (cần {code})",
            e.body["code"]
        );
    }
    Ok(())
}

fn balance_of(all: &[Exchange], label: &str, account: &str, dong: i64) -> Result<()> {
    let e = find(all, label);
    ensure!(e.status == 200, "{label}: status {}", e.status);
    ensure!(
        e.body["account"] == account,
        "{label}: account {}",
        e.body["account"]
    );
    ensure!(
        e.body["balanceDong"] == dong,
        "{label}: balanceDong {} (cần {dong})",
        e.body["balanceDong"]
    );
    let as_of = e.body["asOf"].as_str().unwrap_or("");
    ensure!(
        as_of.len() == 27 && as_of.ends_with('Z'),
        "{label}: asOf lạ: {as_of}"
    );
    Ok(())
}

// Kiểm toàn bộ bảng mã trạng thái và số dư sau chuỗi request mẫu.
fn check_scenario(all: &[Exchange]) -> Result<()> {
    expect(all, "tao_moi", 201, None)?;
    let created = find(all, "tao_moi");
    ensure!(
        created.body["entryId"] == "JE-000001" && created.body["replayed"] == false,
        "tao_moi: {}",
        created.body
    );
    expect(all, "gui_lai", 200, None)?;
    let again = find(all, "gui_lai");
    ensure!(
        again.body["entryId"] == "JE-000001" && again.body["replayed"] == true,
        "gui_lai: {}",
        again.body
    );
    expect(all, "xung_dot_key", 409, Some("idempotency_conflict"))?;
    expect(all, "lech", 422, Some("unbalanced"))?;
    expect(all, "tai_khoan_la", 422, Some("unknown_account"))?;
    expect(all, "json_hong", 400, Some("bad_body"))?;
    balance_of(all, "so_du_khach", "customer:CU-001", -25_500)?;
    balance_of(all, "so_du_tai_xe", "driver:DR-001", 20_400)?;
    balance_of(all, "so_du_nen_tang", "platform:revenue", 5_100)?;
    balance_of(all, "so_du_clearing", "psp:vixanh:clearing", 0)?;
    expect(all, "tai_khoan_chua_mo", 404, Some("account_not_found"))?;
    expect(all, "id_rong", 400, Some("bad_account_id"))?;
    expect(all, "route_la", 404, Some("route_not_found"))?;
    expect(all, "healthz", 200, None)?;
    Ok(())
}

// Gửi body rồi kiểm status và `code` của JSON lỗi.
async fn expect_body_error(
    client: &Client,
    entries: &str,
    label: &str,
    body: &'static str,
    status: u16,
) -> Result<()> {
    let r = client
        .post(entries)
        .header(CONTENT_TYPE, "application/json")
        .body(body)
        .send()
        .await?;
    ensure!(r.status() == status, "{label}: {}", r.status());
    let v: Value = r.json().await?;
    ensure!(v["code"] == "bad_body", "{label}: code {}", v["code"]);
    Ok(())
}

// Ca ngoài chuỗi mẫu: sai hình dạng 422, 415 khi thiếu Content-Type JSON, 405 khi sai
// phương thức, `Path` không phải UTF-8, và 8 request đồng thời cùng một key mới (đúng một 201, bảy 200, số dư chỉ cộng một lần).
async fn check_extras(base: &str) -> Result<()> {
    let client = Client::new();
    let entries = format!("{base}/v1/entries");

    let r = client
        .post(&entries)
        .header(CONTENT_TYPE, "text/plain")
        .body("{}")
        .send()
        .await?;
    ensure!(r.status() == 415, "thiếu Content-Type JSON: {}", r.status());

    for (label, body) in [
        ("thiếu trường", r#"{"idempotencyKey":"k"}"#),
        (
            "sai kiểu",
            r#"{"idempotencyKey":"k","postings":[{"account":"a","amountDong":"100"}]}"#,
        ),
        (
            "trường lạ",
            r#"{"idempotencyKey":"k","postings":[],"memo":"x"}"#,
        ),
    ] {
        expect_body_error(&client, &entries, label, body, 422).await?;
    }

    // RFC 9110: 405 phải có header `Allow`. axum tự thêm; thân rỗng.
    let r = client.get(&entries).send().await?;
    ensure!(r.status() == 405, "GET /v1/entries: {}", r.status());
    ensure!(
        r.headers().get("allow").is_some_and(|v| v == "POST"),
        "405 thiếu Allow: {:?}",
        r.headers()
    );
    ensure!(r.bytes().await?.is_empty(), "405 có thân");

    let r = client
        .get(format!("{base}/v1/accounts/%FF/balance"))
        .send()
        .await?;
    ensure!(r.status() == 400, "Path không phải UTF-8: {}", r.status());
    let v: Value = r.json().await?;
    ensure!(v["code"] == "bad_account_id", "Path: {v}");

    let mut body = capture_body();
    body["idempotencyKey"] = Value::from("trip:TN-0009:capture");
    let mut tasks = Vec::new();
    for _ in 0..8 {
        let (client, entries, body) = (client.clone(), entries.clone(), body.clone());
        tasks.push(tokio::spawn(async move {
            client
                .post(&entries)
                .json(&body)
                .send()
                .await
                .map(|r| r.status().as_u16())
        }));
    }
    let mut statuses = Vec::new();
    for t in tasks {
        statuses.push(t.await??);
    }
    let created = statuses.iter().filter(|s| **s == 201).count();
    let replayed = statuses.iter().filter(|s| **s == 200).count();
    ensure!(created == 1 && replayed == 7, "đồng thời: {statuses:?}");

    let v: Value = client
        .get(format!("{base}/v1/accounts/driver:DR-001/balance"))
        .send()
        .await?
        .json()
        .await?;
    ensure!(
        v["balanceDong"] == 40_800,
        "sau 2 chuyến tài xế phải có 40.800: {v}"
    );
    Ok(())
}

async fn run_verify() -> Result<()> {
    let lab = LabDb::setup().await?;
    let server = serve_lab(&lab).await?;
    let outcome = async {
        let all = scenario::run(&server.base_url()).await?;
        check_scenario(&all)?;
        check_extras(&server.base_url()).await
    }
    .await;
    // Dọn dù kiểm có lỗi hay không, rồi xác nhận schema đã biến mất.
    server.handle.abort();
    let url = std::env::var("TOI_NOI_PG_URL")?;
    let schema = lab.cleanup().await?;
    ensure!(
        !LabDb::schema_exists(&url, &schema).await?,
        "schema {schema} vẫn còn"
    );
    outcome
}

async fn run_demo() -> Result<()> {
    let lab = LabDb::setup().await?;
    let server = serve_lab(&lab).await?;
    println!("server nghe tại http://127.0.0.1:<cổng do hệ điều hành cấp>");
    let result = scenario::run(&server.base_url()).await;
    server.handle.abort();
    lab.cleanup().await?;
    for e in result? {
        println!("{:<42} -> {} {}", e.request, e.status, e.body);
    }
    Ok(())
}

async fn run_serve() -> Result<()> {
    let lab = LabDb::setup().await?;
    let server = serve_lab(&lab).await?;
    println!(
        "nghe tại {} (schema {}). Ctrl-C để dừng.",
        server.base_url(),
        lab.schema()
    );
    tokio::signal::ctrl_c().await?;
    server.handle.abort();
    lab.cleanup().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    let arg = std::env::args().nth(1);
    let result = match arg.as_deref() {
        None => run_demo().await,
        Some("--verify") => run_verify().await.map(|()| println!("VERIFY OK")),
        Some("--serve") => run_serve().await,
        Some(other) => Err(anyhow::anyhow!("tham số lạ: {other}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("LỖI: {e:#}");
            ExitCode::FAILURE
        }
    }
}
