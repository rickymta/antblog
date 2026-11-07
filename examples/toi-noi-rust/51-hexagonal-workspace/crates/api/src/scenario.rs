//! Kịch bản chạy thật cho `--verify`: cùng một use case, ba cách cắm adapter, một kết quả.

use std::sync::Arc;

use anyhow::{Result, ensure};
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use infrastructure::clock::FixedClock;
use infrastructure::memory::{MemoryPublisher, MemoryStore};
use infrastructure::pg::{LabDb, PgStore};
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::compose::Services;
use crate::http::router;

/// 2026-11-09T01:30:00Z. Đồng hồ cố định nên JSON phát ra luôn giống nhau.
pub const T0: i64 = 1_794_187_800;

/// Gọi router không qua mạng (`oneshot`); trả mã trạng thái, content-type và thân.
pub async fn call(
    app: &Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> Result<(StatusCode, String, String)> {
    let mut req = Request::builder().method(method).uri(uri);
    let body = match body {
        Some(v) => {
            req = req.header(header::CONTENT_TYPE, "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    let resp = app.clone().oneshot(req.body(body)?).await?;
    let status = resp.status();
    let ctype = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let bytes = resp.into_body().collect().await?.to_bytes();
    Ok((status, ctype, String::from_utf8_lossy(&bytes).into_owned()))
}

fn tn_0001() -> Value {
    json!({ "tripId": "TN-0001", "customerId": "CU-001", "driverId": "DR-001", "distanceKm": 5 })
}

/// Chạy chuyến `TN-0001` hai lần qua HTTP và kiểm kết quả. Trả về số dư theo thứ tự trả về.
async fn settle_twice(app: &Router, label: &str) -> Result<Vec<(String, i64)>> {
    let (status, _, body) = call(app, "POST", "/trips/settle", Some(tn_0001())).await?;
    ensure!(
        status == StatusCode::OK,
        "{label}: POST lần 1 trả {status}: {body}"
    );
    let first: Value = serde_json::from_str(&body)?;
    println!("POST /trips/settle -> {status}");
    let mut balances = Vec::new();
    for s in first["steps"].as_array().into_iter().flatten() {
        println!(
            "  {:<22} {}  replayed={}",
            s["key"].as_str().unwrap_or("?"),
            s["entryId"].as_str().unwrap_or("?"),
            s["replayed"]
        );
        ensure!(s["replayed"] == false, "{label}: lần 1 không được replay");
    }
    for b in first["balances"].as_array().into_iter().flatten() {
        let (a, d) = (
            b["account"].as_str().unwrap_or("?").to_string(),
            b["balanceDong"].as_i64().unwrap_or(i64::MIN),
        );
        println!("  {a:<22} {d:>7}");
        balances.push((a, d));
    }
    let (status2, _, body2) = call(app, "POST", "/trips/settle", Some(tn_0001())).await?;
    ensure!(
        status2 == StatusCode::OK,
        "{label}: POST lần 2 trả {status2}"
    );
    let second: Value = serde_json::from_str(&body2)?;
    let replayed: Vec<bool> = second["steps"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|s| s["replayed"] == true)
        .collect();
    println!("POST lần hai: replayed={replayed:?}");
    ensure!(
        replayed == [true, true],
        "{label}: lần 2 phải replay cả hai bước"
    );
    ensure!(
        second["balances"] == first["balances"],
        "{label}: số dư đổi sau replay"
    );

    let (st, _, bal) = call(app, "GET", "/accounts/driver:DR-001/balance", None).await?;
    println!("GET /accounts/driver:DR-001/balance -> {st} {bal}");
    ensure!(
        st == StatusCode::OK && bal.contains("\"balanceDong\":20400"),
        "{label}: số dư tài xế sai: {bal}"
    );

    let mut bad = tn_0001();
    bad["distanceKm"] = json!(0);
    let (st, ctype, _) = call(app, "POST", "/trips/settle", Some(bad)).await?;
    println!("POST distanceKm=0 -> {st} {ctype}");
    ensure!(
        st == StatusCode::UNPROCESSABLE_ENTITY && ctype == "application/problem+json",
        "{label}: lỗi miền phải là 422 problem+json"
    );
    Ok(balances)
}

fn expected_balances() -> Vec<(String, i64)> {
    vec![
        ("customer:CU-001".to_string(), -25_500),
        ("psp:vixanh:clearing".to_string(), 0),
        ("platform:revenue".to_string(), 5_100),
        ("driver:DR-001".to_string(), 20_400),
    ]
}

/// Cắm `MemoryStore` + `MemoryPublisher`. Trả về JSON các sự kiện đã phát.
pub async fn memory() -> Result<Vec<String>> {
    println!("== 1. Bộ nhớ: MemoryStore + MemoryPublisher");
    let publisher = Arc::new(MemoryPublisher::new());
    let svc = Arc::new(Services::new(
        Arc::new(MemoryStore::new()),
        Arc::clone(&publisher),
        Arc::new(FixedClock(T0)),
    ));
    let app = router(svc);
    let balances = settle_twice(&app, "memory").await?;
    ensure!(
        balances == expected_balances(),
        "memory: số dư {balances:?}"
    );
    let sent = publisher.sent();
    println!("sự kiện đã phát: {} (lần hai không phát thêm)", sent.len());
    ensure!(
        sent.len() == 2,
        "memory: phải đúng 2 sự kiện, có {}",
        sent.len()
    );
    println!("  {}", sent[0]);
    Ok(sent)
}

/// Cắm `PgStore` (schema lab riêng) + `MemoryPublisher`. Dọn schema kể cả khi lỗi.
pub async fn postgres(expected_events: &[String]) -> Result<()> {
    println!("== 2. PostgreSQL: PgStore + MemoryPublisher");
    let lab = LabDb::setup().await?;
    let schema = lab.schema().to_string();
    let url = std::env::var("TOI_NOI_PG_URL")?;
    let outcome = async {
        let publisher = Arc::new(MemoryPublisher::new());
        let svc = Arc::new(Services::new(
            Arc::new(PgStore::new(lab.pool.clone())),
            Arc::clone(&publisher),
            Arc::new(FixedClock(T0)),
        ));
        let balances = settle_twice(&router(svc), "pg").await?;
        ensure!(balances == expected_balances(), "pg: số dư {balances:?}");
        ensure!(
            publisher.sent() == expected_events,
            "pg: JSON sự kiện khác bản bộ nhớ"
        );
        println!("JSON sự kiện giống hệt bản bộ nhớ: true");
        Ok::<_, anyhow::Error>(())
    }
    .await;
    lab.cleanup().await?;
    let gone = !LabDb::schema_exists(&url, &schema).await?;
    println!("đã dọn schema lab_<uuid>: {gone}");
    outcome?;
    ensure!(gone, "schema {schema} còn sót");
    Ok(())
}

/// Cắm `PgStore` + `KafkaPublisher` (topic riêng), đọc lại message từ Kafka.
#[cfg(feature = "kafka")]
pub async fn kafka(expected_events: &[String]) -> Result<()> {
    use infrastructure::kafka::{KafkaLab, KafkaPublisher};

    println!("== 3. PostgreSQL + Kafka: PgStore + KafkaPublisher");
    let kafka = KafkaLab::setup().await?;
    let outcome = async {
        let lab = LabDb::setup().await?;
        let result = async {
            let publisher = Arc::new(KafkaPublisher::new(kafka.brokers(), &kafka.topic)?);
            let svc = Arc::new(Services::new(
                Arc::new(PgStore::new(lab.pool.clone())),
                publisher,
                Arc::new(FixedClock(T0)),
            ));
            let (status, _, body) =
                call(&router(svc), "POST", "/trips/settle", Some(tn_0001())).await?;
            ensure!(status == StatusCode::OK, "kafka: POST trả {status}: {body}");
            let read = kafka.read(2).await?;
            let keys: Vec<&str> = read.iter().map(|(k, _)| k.as_str()).collect();
            println!("đọc lại từ Kafka: khoá={keys:?}");
            let payloads: Vec<String> = read.into_iter().map(|(_, v)| v).collect();
            ensure!(payloads == expected_events, "kafka: JSON khác bản bộ nhớ");
            println!("JSON trên Kafka giống hệt bản bộ nhớ: true");
            Ok::<_, anyhow::Error>(())
        }
        .await;
        lab.cleanup().await?;
        result
    }
    .await;
    kafka.cleanup().await?;
    outcome
}

/// Không có feature `kafka`: crate hạ tầng không biên dịch rdkafka, nên không có gì để chạy.
#[cfg(not(feature = "kafka"))]
#[allow(clippy::unused_async)]
pub async fn kafka(_expected_events: &[String]) -> Result<()> {
    println!(
        "== 3. Kafka: bỏ qua vì build không có feature `kafka` (không có rdkafka trong binary)"
    );
    Ok(())
}
