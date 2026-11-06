//! Chuỗi request thật qua TCP: server axum nghe `127.0.0.1:0`, client `reqwest` gọi vào.
//! Dùng chung cho `cargo run` (in) và `--verify` (kiểm).

use std::net::SocketAddr;

use anyhow::{Context, Result};
use axum::Router;
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::app::{AppState, router};
use crate::lab_db::LabDb;
use crate::store::PgLedger;

/// Một cặp request/response đã chạy.
#[derive(Debug, Clone)]
pub struct Exchange {
    /// Nhãn ngắn của bước.
    pub label: &'static str,
    /// Dòng request, ví dụ `POST /v1/entries`.
    pub request: String,
    /// Status HTTP nhận được.
    pub status: u16,
    /// Thân JSON nhận được.
    pub body: Value,
}

/// Server đang chạy trong một task. `abort` khi xong.
#[derive(Debug)]
pub struct RunningServer {
    /// Địa chỉ thật (cổng do hệ điều hành cấp).
    pub addr: SocketAddr,
    /// Task chạy `axum::serve`.
    pub handle: JoinHandle<()>,
}

impl RunningServer {
    /// URL gốc, ví dụ `http://127.0.0.1:54321`.
    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }
}

/// Bind `127.0.0.1:0`, đọc cổng thật rồi chạy `app` trong một task nền.
///
/// # Errors
/// Không bind được.
pub async fn spawn_server(app: Router) -> Result<RunningServer> {
    // Cổng 0: hệ điều hành chọn cổng trống, nên chạy song song không đụng nhau.
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("bind 127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("server dừng vì lỗi: {e}");
        }
    });
    Ok(RunningServer { addr, handle })
}

/// Dựng router trên kho của `lab` và chạy nó.
///
/// # Errors
/// Không bind được.
pub async fn serve_lab(lab: &LabDb) -> Result<RunningServer> {
    let state = AppState {
        ledger: PgLedger::new(lab.pool.clone()),
    };
    spawn_server(router(state)).await
}

/// Body chuẩn của chuyến `TN-0001`: khách trả 25.500, nền tảng 5.100, tài xế 20.400.
pub fn capture_body() -> Value {
    json!({
        "idempotencyKey": "trip:TN-0001:capture",
        "postings": [
            { "account": "customer:CU-001", "amountDong": -25_500 },
            { "account": "platform:revenue", "amountDong": 5_100 },
            { "account": "driver:DR-001", "amountDong": 20_400 }
        ]
    })
}

async fn send(
    client: &Client,
    label: &'static str,
    request: reqwest::RequestBuilder,
) -> Result<Exchange> {
    let built = request.build()?;
    let line = format!("{} {}", built.method(), built.url().path());
    let response = client.execute(built).await.with_context(|| label)?;
    let status = response.status().as_u16();
    let body: Value = response
        .json()
        .await
        .with_context(|| format!("{label}: body không phải JSON"))?;
    Ok(Exchange {
        label,
        request: line,
        status,
        body,
    })
}

/// Chạy chuỗi request mẫu, theo thứ tự, và trả về từng cặp request/response.
///
/// # Errors
/// Lỗi mạng hoặc thân phản hồi không phải JSON.
pub async fn run(base: &str) -> Result<Vec<Exchange>> {
    let client = Client::new();
    let url = |path: &str| format!("{base}{path}");
    let entries = url("/v1/entries");
    let mut out = Vec::new();

    out.push(
        send(
            &client,
            "tao_moi",
            client.post(&entries).json(&capture_body()),
        )
        .await?,
    );
    out.push(
        send(
            &client,
            "gui_lai",
            client.post(&entries).json(&capture_body()),
        )
        .await?,
    );

    // Cùng key, khác nội dung (vẫn cân): xung đột.
    let altered = json!({
        "idempotencyKey": "trip:TN-0001:capture",
        "postings": [
            { "account": "customer:CU-001", "amountDong": -25_500 },
            { "account": "platform:revenue", "amountDong": 5_000 },
            { "account": "driver:DR-001", "amountDong": 20_500 }
        ]
    });
    out.push(
        send(
            &client,
            "xung_dot_key",
            client.post(&entries).json(&altered),
        )
        .await?,
    );

    // Lệch 100 đồng: tổng khác 0.
    let unbalanced = json!({
        "idempotencyKey": "trip:TN-0002:capture",
        "postings": [
            { "account": "customer:CU-001", "amountDong": -25_500 },
            { "account": "platform:revenue", "amountDong": 5_100 },
            { "account": "driver:DR-001", "amountDong": 20_300 }
        ]
    });
    out.push(send(&client, "lech", client.post(&entries).json(&unbalanced)).await?);

    // Tài khoản chưa mở.
    let unknown = json!({
        "idempotencyKey": "trip:TN-0003:capture",
        "postings": [
            { "account": "customer:CU-999", "amountDong": -100 },
            { "account": "platform:revenue", "amountDong": 100 }
        ]
    });
    out.push(
        send(
            &client,
            "tai_khoan_la",
            client.post(&entries).json(&unknown),
        )
        .await?,
    );

    // JSON bị cắt giữa chừng: 400.
    let broken = client
        .post(&entries)
        .header(CONTENT_TYPE, "application/json")
        .body(r#"{"idempotencyKey": "trip:TN-0004:capture", "postings": ["#);
    out.push(send(&client, "json_hong", broken).await?);

    for (label, account) in [
        ("so_du_khach", "customer:CU-001"),
        ("so_du_tai_xe", "driver:DR-001"),
        ("so_du_nen_tang", "platform:revenue"),
        ("so_du_clearing", "psp:vixanh:clearing"),
    ] {
        out.push(
            send(
                &client,
                label,
                client.get(url(&format!("/v1/accounts/{account}/balance"))),
            )
            .await?,
        );
    }

    out.push(
        send(
            &client,
            "tai_khoan_chua_mo",
            client.get(url("/v1/accounts/customer:CU-999/balance")),
        )
        .await?,
    );
    out.push(
        send(
            &client,
            "id_rong",
            client.get(url("/v1/accounts/%20/balance")),
        )
        .await?,
    );
    out.push(send(&client, "route_la", client.get(url("/v1/nope"))).await?);
    out.push(send(&client, "healthz", client.get(url("/healthz"))).await?);
    Ok(out)
}
