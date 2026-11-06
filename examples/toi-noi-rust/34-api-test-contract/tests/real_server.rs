//! Cùng router, hai đường vào: `oneshot` (không socket) và server thật trên `127.0.0.1:0`.
//! Test dưới đây chỉ ra khi nào phải chọn đường thứ hai.

use std::net::SocketAddr;

use axum::Router;
use axum::extract::ConnectInfo;
use axum::extract::connect_info::MockConnectInfo;
use axum::http::{Method, StatusCode};
use axum::routing::get;
use ledger_api::rawhttp;
use ledger_api::testkit::{TestApp, send};

const CAPTURE: &str = include_str!("../contracts/post-entry.request.json");

#[tokio::test]
async fn socket_and_oneshot_agree_on_status_and_body() {
    let app = TestApp::start().await;
    let router = app.router();
    let token = app.full_token();
    let (addr, server) = rawhttp::serve(router.clone()).await.unwrap();
    let auth = format!("Bearer {token}");

    // Ghi một lần bằng oneshot, đọc lại bằng cả hai đường.
    let posted = send(
        &router,
        Method::POST,
        "/v1/entries",
        Some(&token),
        Some(CAPTURE),
    )
    .await;
    assert_eq!(posted.status, StatusCode::CREATED);

    let path = "/v1/accounts/driver:DR-001/balance";
    let via_oneshot = send(&router, Method::GET, path, Some(&token), None).await;
    let via_socket = rawhttp::request(addr, "GET", path, &[("Authorization", &auth)], None)
        .await
        .unwrap();

    assert_eq!(via_socket.status, via_oneshot.status.as_u16());
    let (mut a, mut b) = (via_oneshot.json(), via_socket.json().unwrap());
    // asOf là giờ đọc của PostgreSQL: có thể lệch một giây giữa hai lần gọi.
    a["asOf"] = "-".into();
    b["asOf"] = "-".into();
    assert_eq!(a, b);

    // Replay qua socket: cùng bút toán, 200.
    let replay = rawhttp::request(
        addr,
        "POST",
        "/v1/entries",
        &[
            ("Authorization", &auth),
            ("Content-Type", "application/json"),
        ],
        Some(CAPTURE),
    )
    .await
    .unwrap();
    assert_eq!(replay.status, 200);
    assert_eq!(replay.json().unwrap()["replayed"], true);

    server.abort();
    app.close().await;
}

#[tokio::test]
async fn only_the_socket_path_shows_what_hyper_adds_to_the_response() {
    let app = TestApp::start().await;
    let router = app.router();
    let token = app.full_token();
    let (addr, server) = rawhttp::serve(router.clone()).await.unwrap();
    let auth = format!("Bearer {token}");
    let path = "/v1/accounts/driver:DR-001/balance";

    let via_oneshot = send(&router, Method::GET, path, Some(&token), None).await;
    let via_socket = rawhttp::request(addr, "GET", path, &[("Authorization", &auth)], None)
        .await
        .unwrap();

    // `date` và `content-length` do hyper thêm khi ghi ra dây; oneshot dừng trước bước đó.
    assert!(via_oneshot.headers.get("date").is_none());
    assert!(via_socket.header("date").is_some());
    assert!(via_socket.header("content-length").is_some());

    server.abort();
    app.close().await;
}

// Handler dùng địa chỉ client, ví dụ để giới hạn theo IP (`tower::limit` ở bài 33 không phân biệt client).
fn ip_router() -> Router {
    Router::new().route(
        "/ip",
        get(|ConnectInfo(addr): ConnectInfo<SocketAddr>| async move { addr.ip().to_string() }),
    )
}

#[tokio::test]
async fn connect_info_is_missing_under_oneshot_and_present_on_a_socket() {
    // oneshot: không có socket nên không ai chèn ConnectInfo; extractor trả 500.
    let router = ip_router();
    let bare = send(&router, Method::GET, "/ip", None, None).await;
    assert_eq!(bare.status, StatusCode::INTERNAL_SERVER_ERROR);

    // Cách chữa trong test: chèn địa chỉ giả bằng MockConnectInfo.
    let mocked = ip_router().layer(MockConnectInfo(SocketAddr::from(([203, 0, 113, 7], 5000))));
    let reply = send(&mocked, Method::GET, "/ip", None, None).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(&reply.body[..], b"203.0.113.7");

    // Server thật: địa chỉ là của kết nối TCP thật.
    let (addr, server) = rawhttp::serve(ip_router()).await.unwrap();
    let real = rawhttp::request(addr, "GET", "/ip", &[], None)
        .await
        .unwrap();
    assert_eq!(real.status, 200);
    assert_eq!(real.body, "127.0.0.1");
    server.abort();
}

#[tokio::test]
async fn hyper_rejects_a_malformed_request_before_the_router_sees_it() {
    // Request viết tay có tên header chứa dấu cách: sai giao thức HTTP/1.1.
    // oneshot nhận `Request` đã dựng sẵn nên không có byte nào để sai; chỉ server thật
    // trả 400 từ hyper, và thân 400 đó không phải problem+json của ta (router không được gọi).
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let app = TestApp::start().await;
    let (addr, server) = rawhttp::serve(app.router()).await.unwrap();

    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    stream
        .write_all(
            b"GET /api-docs/openapi.json HTTP/1.1\r\nHost: x\r\nBad Header: v\r\nConnection: close\r\n\r\n",
        )
        .await
        .unwrap();
    let mut raw = String::new();
    stream.read_to_string(&mut raw).await.unwrap();
    assert!(raw.starts_with("HTTP/1.1 400"), "{raw}");
    assert!(!raw.contains("problem+json"), "{raw}");

    server.abort();
    app.close().await;
}
