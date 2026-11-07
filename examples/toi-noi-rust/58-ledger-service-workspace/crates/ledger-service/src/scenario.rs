//! Các kịch bản end-to-end dùng chung cho `cargo run`, `--verify` và `cargo test`.
//! Mỗi kịch bản tự dựng một [`LabEnv`] riêng nên chạy song song được và không để lại dữ liệu.
//! Không kịch bản nào assert thời gian: thời gian nghiệp vụ đi qua `ManualClock`, việc chờ
//! sự kiện thật luôn có hạn chót thay vì ngủ cố định.

use std::net::SocketAddr;
use std::path::Path;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail, ensure};
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use jsonwebtoken::Algorithm;
use ledger_api::auth::{AUDIENCE, Claims, now_secs};
use ledger_api::grpc::pb;
use ledger_application::LedgerStore;
use ledger_application::model::PostOutcome;
use ledger_domain::chart::{CLEARING, REVENUE};
use ledger_domain::clock::ManualClock;
use ledger_domain::{AccountId, JournalEntry, Money, Posting, fare_for_km};
use ledger_infrastructure::kafka;
use ledger_infrastructure::pg::PgStore;
use ledger_infrastructure::stats::Stats;
use ledger_infrastructure::wire::{LedgerEntryPosted, TripCompleted};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::{ClientConfig, Message};
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time::{Instant, sleep, timeout, timeout_at};

use crate::app::App;
use crate::harness::LabEnv;

/// Thời gian chờ tối đa cho một sự kiện đi hết vòng Kafka → PostgreSQL → Kafka.
const WITHIN: Duration = Duration::from_secs(30);

/// Những dòng tường thuật của một kịch bản (in ra ở `cargo run`).
#[derive(Debug, Default)]
pub struct Report(pub Vec<String>);

impl Report {
    fn say(&mut self, line: impl Into<String>) {
        self.0.push(line.into());
    }
}

// ---------- công cụ phía test ----------

/// Consumer đọc topic `ledger.entry-posted` như một service khác (Payment) sẽ làm.
struct Probe {
    consumer: StreamConsumer,
}

impl Probe {
    fn new(env: &LabEnv) -> Result<Probe> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &env.kafka)
            .set("group.id", format!("{}-probe", env.group))
            .set("auto.offset.reset", "earliest")
            .create()?;
        consumer.subscribe(&[&env.topic_out()])?;
        Ok(Probe { consumer })
    }

    /// Chờ `LedgerEntryPosted` có đúng `key`, bỏ qua sự kiện đứng trước. Có hạn chót, không ngủ cố định.
    async fn event_for(&self, key: &str) -> Result<LedgerEntryPosted> {
        let deadline = Instant::now() + WITHIN;
        loop {
            let received = timeout_at(deadline, self.consumer.recv())
                .await
                .map_err(|_| anyhow!("không thấy LedgerEntryPosted của {key} sau {WITHIN:?}"))?;
            // Lỗi tạm của broker không làm test đỏ ngay: hạn chót mới là phán quyết.
            let Ok(msg) = received else { continue };
            let event: LedgerEntryPosted =
                serde_json::from_slice(msg.payload().context("payload rỗng")?)?;
            ensure!(
                msg.key() == Some(event.idempotency_key.as_bytes()),
                "key Kafka phải là idempotencyKey"
            );
            if event.idempotency_key == key {
                return Ok(event);
            }
        }
    }
}

/// Kết quả của một request HTTP.
struct Reply {
    status: u16,
    content_type: Option<String>,
    www_authenticate: Option<String>,
    request_id: Option<String>,
    body: String,
}

impl Reply {
    fn json(&self) -> Result<Value> {
        serde_json::from_str(&self.body)
            .with_context(|| format!("thân không phải JSON: {}", self.body))
    }

    /// Kỳ vọng một Problem Details: mã, media type và đuôi của `type`.
    fn expect_problem(&self, status: u16, slug: &str) -> Result<Value> {
        ensure!(
            self.status == status,
            "HTTP {} (mong {status}): {}",
            self.status,
            self.body
        );
        ensure!(
            self.content_type.as_deref() == Some("application/problem+json"),
            "content-type {:?}",
            self.content_type
        );
        let json = self.json()?;
        let type_uri = json["type"].as_str().unwrap_or_default();
        ensure!(
            type_uri.ends_with(slug),
            "type {type_uri} (mong đuôi {slug})"
        );
        ensure!(json["status"] == status, "status trong body: {json}");
        ensure!(
            json["traceId"].as_str().is_some_and(|t| t.len() == 32),
            "traceId: {json}"
        );
        Ok(json)
    }
}

async fn http_call(
    addr: SocketAddr,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Option<&Value>,
    request_id: Option<&str>,
) -> Result<Reply> {
    let client = Client::builder(TokioExecutor::new()).build_http::<Full<Bytes>>();
    let mut request = hyper::Request::builder()
        .method(method)
        .uri(format!("http://{addr}{path}"));
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    if let Some(id) = request_id {
        request = request.header("x-request-id", id);
    }
    let payload = match body {
        Some(value) => {
            request = request.header("content-type", "application/json");
            Bytes::from(serde_json::to_vec(value)?)
        }
        None => Bytes::new(),
    };
    let response = client.request(request.body(Full::new(payload))?).await?;
    let header = |name: &str| {
        response
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
    };
    let (status, content_type, www_authenticate, request_id) = (
        response.status().as_u16(),
        header("content-type"),
        header("www-authenticate"),
        header("x-request-id"),
    );
    let body = String::from_utf8(response.into_body().collect().await?.to_bytes().to_vec())?;
    Ok(Reply {
        status,
        content_type,
        www_authenticate,
        request_id,
        body,
    })
}

async fn http_get(addr: SocketAddr, path: &str, token: Option<&str>) -> Result<Reply> {
    http_call(addr, "GET", path, token, None, None).await
}

async fn http_post(addr: SocketAddr, path: &str, token: &str, body: &Value) -> Result<Reply> {
    http_call(addr, "POST", path, Some(token), Some(body), None).await
}

async fn http_balance(addr: SocketAddr, token: &str, account: &str) -> Result<i64> {
    let reply = http_get(
        addr,
        &format!("/v1/accounts/{account}/balance"),
        Some(token),
    )
    .await?;
    ensure!(
        reply.status == 200,
        "HTTP {} cho {account}: {}",
        reply.status,
        reply.body
    );
    let json = reply.json()?;
    ensure!(
        json["account"] == account,
        "thân trả về sai tài khoản: {json}"
    );
    ensure!(json["asOf"].is_string(), "thiếu asOf: {json}");
    json["balanceDong"]
        .as_i64()
        .context("balanceDong không phải số nguyên")
}

type GrpcClient = pb::ledger_client::LedgerClient<tonic::transport::Channel>;

async fn grpc_client(addr: SocketAddr) -> Result<GrpcClient> {
    Ok(pb::ledger_client::LedgerClient::connect(format!("http://{addr}")).await?)
}

fn authed<T>(message: T, token: &str) -> Result<tonic::Request<T>> {
    let mut request = tonic::Request::new(message);
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse()?);
    Ok(request)
}

async fn grpc_balance(addr: SocketAddr, token: &str, account: &str) -> Result<i64> {
    let reply = grpc_client(addr)
        .await?
        .get_balance(authed(
            pb::GetBalanceRequest {
                account: account.to_string(),
            },
            token,
        )?)
        .await?
        .into_inner();
    ensure!(
        reply.account == account && reply.as_of.is_some(),
        "gRPC trả thiếu trường"
    );
    Ok(reply.balance_dong)
}

/// Số dư phải bằng nhau qua cả HTTP lẫn gRPC.
async fn expect_balance(app: &App, token: &str, account: &str, expected: i64) -> Result<()> {
    let over_http = http_balance(app.http_addr, token, account).await?;
    let over_grpc = grpc_balance(app.grpc_addr, token, account).await?;
    ensure!(
        over_http == expected && over_grpc == expected,
        "{account}: HTTP {over_http}, gRPC {over_grpc}, mong {expected}"
    );
    Ok(())
}

/// Chờ tới khi `cond` đúng hoặc hết hạn. Kiểm tra mỗi 25 ms, không ngủ một khoảng cố định.
async fn wait_until(what: &str, mut cond: impl FnMut() -> bool) -> Result<()> {
    let deadline = Instant::now() + WITHIN;
    while !cond() {
        if Instant::now() >= deadline {
            bail!("quá {WITHIN:?} mà chưa thấy: {what}");
        }
        sleep(Duration::from_millis(25)).await;
    }
    Ok(())
}

async fn wait_ready(addr: SocketAddr) -> Result<()> {
    let deadline = Instant::now() + WITHIN;
    loop {
        if let Ok(reply) = http_get(addr, "/readyz", None).await
            && reply.status == 200
        {
            return Ok(());
        }
        ensure!(
            Instant::now() < deadline,
            "/readyz không bao giờ 200 (consumer chưa có partition?)"
        );
        sleep(Duration::from_millis(50)).await;
    }
}

async fn count(pool: &PgPool, sql: &'static str) -> Result<i64> {
    Ok(sqlx::query_scalar(sql).fetch_one(pool).await?)
}

async fn send_trip(env: &LabEnv, trip: &TripCompleted) -> Result<()> {
    let producer = kafka::producer(&env.kafka)?;
    kafka::send_json(&producer, &env.topic_in(), &trip.trip_id, trip).await
}

fn amounts(event: &LedgerEntryPosted) -> Vec<(&str, i64)> {
    event
        .postings
        .iter()
        .map(|p| (p.account.as_str(), p.amount_dong))
        .collect()
}

async fn start(env: &LabEnv) -> Result<(App, Arc<ManualClock>)> {
    let clock = Arc::new(ManualClock::lab_start());
    let app = App::start(env.config()?, clock.clone()).await?;
    wait_ready(app.http_addr).await?;
    Ok((app, clock))
}

// ---------- kịch bản ----------

/// TripCompleted → hai bút toán → outbox → LedgerEntryPosted → số dư qua HTTP và gRPC;
/// phát lại không tạo bút toán thứ hai; tắt có trật tự.
///
/// # Errors
/// Mọi kỳ vọng sai, hạ tầng thiếu, hoặc quá hạn chờ.
pub async fn flow_and_replay() -> Result<Report> {
    LabEnv::run(|env| async move {
        let mut report = Report::default();
        let probe = Probe::new(&env)?;
        let (app, _clock) = start(&env).await?;
        let cs = env.token(&["cs"])?;
        report.say("readyz 200 khi consumer đã được gán partition");

        // 1. Chuyến TN-0001, 5 km, cước 25.500: hai bút toán, hai sự kiện.
        let trip1 = TripCompleted::sample("TN-0001", fare_for_km(5).dong());
        send_trip(&env, &trip1).await?;
        let capture = probe.event_for("trip:TN-0001:capture").await?;
        let split = probe.event_for("trip:TN-0001:split").await?;
        ensure!(
            capture.entry_id == "JE-000001" && split.entry_id == "JE-000002",
            "mã bút toán"
        );
        ensure!(
            amounts(&capture)
                == [
                    ("customer:CU-001", -25_500),
                    ("psp:vixanh:clearing", 25_500)
                ],
            "capture {:?}",
            amounts(&capture)
        );
        ensure!(
            amounts(&split)
                == [
                    ("psp:vixanh:clearing", -25_500),
                    ("platform:revenue", 5_100),
                    ("driver:DR-001", 20_400)
                ],
            "split {:?}",
            amounts(&split)
        );
        report.say(format!(
            "capture {} {:?}",
            capture.entry_id,
            amounts(&capture)
        ));
        report.say(format!("split   {} {:?}", split.entry_id, amounts(&split)));

        // 2. Số dư qua HTTP và gRPC: cùng một use case nên cùng một số. Clearing về 0 sau bước chia.
        expect_balance(&app, &cs, "driver:DR-001", 20_400).await?;
        expect_balance(&app, &cs, "platform:revenue", 5_100).await?;
        expect_balance(&app, &cs, "customer:CU-001", -25_500).await?;
        expect_balance(&app, &cs, "psp:vixanh:clearing", 0).await?;
        report.say("driver 20400, revenue 5100, customer -25500, clearing 0 (HTTP và gRPC khớp)");
        let missing = http_get(
            app.http_addr,
            "/v1/accounts/driver:DR-404/balance",
            Some(&cs),
        )
        .await?;
        missing.expect_problem(404, "unknown-account")?;
        ensure!(
            http_get(app.http_addr, "/healthz", None).await?.status == 200,
            "healthz"
        );

        // 3. Phát lại ĐÚNG sự kiện đó, rồi một chuyến mới làm "cột mốc": khi cột mốc đã ra,
        //    sự kiện phát lại (cùng partition, đứng trước) chắc chắn đã được xử lý.
        send_trip(&env, &trip1).await?;
        send_trip(
            &env,
            &TripCompleted::sample("TN-0002", fare_for_km(1).dong()),
        )
        .await?;
        probe.event_for("trip:TN-0002:capture").await?;
        let last = probe.event_for("trip:TN-0002:split").await?;
        ensure!(
            last.entry_id == "JE-000004",
            "cột mốc phải là JE-000004, nhận {}",
            last.entry_id
        );
        wait_until("duplicates >= 1", || Stats::get(&app.stats.duplicates) >= 1).await?;

        let pool = env.inspect_pool().await?;
        let entries = count(&pool, "SELECT count(*) FROM journal_entries").await?;
        let same_key = count(
            &pool,
            "SELECT count(*) FROM journal_entries WHERE idempotency_key LIKE 'trip:TN-0001:%'",
        )
        .await?;
        let (inbox, payments) = (
            count(&pool, "SELECT count(*) FROM inbox").await?,
            count(&pool, "SELECT count(*) FROM payments").await?,
        );
        ensure!(
            entries == 4 && same_key == 2 && inbox == 2 && payments == 2,
            "entries {entries}, same_key {same_key}, inbox {inbox}, payments {payments}"
        );
        ensure!(
            Stats::get(&app.stats.processed) == 2 && Stats::get(&app.stats.duplicates) == 1,
            "bộ đếm"
        );
        expect_balance(&app, &cs, "driver:DR-001", 20_400 + 9_600).await?;
        report.say(
            "phát lại TripCompleted TN-0001: inbox chặn, vẫn 4 bút toán, 2 bút toán cho TN-0001",
        );

        // 4. Tắt có trật tự: readyz đổi 503 trước, healthz vẫn 200, outbox rỗng khi xong.
        app.readiness.begin_shutdown();
        ensure!(
            http_get(app.http_addr, "/readyz", None).await?.status == 503,
            "readyz khi tắt"
        );
        ensure!(
            http_get(app.http_addr, "/healthz", None).await?.status == 200,
            "healthz khi tắt"
        );
        let done = app.shutdown().await;
        ensure!(!done.timed_out, "shutdown quá hạn");
        let pending = count(
            &pool,
            "SELECT count(*) FROM outbox WHERE published_at IS NULL",
        )
        .await?;
        ensure!(pending == 0, "outbox còn {pending} dòng chưa phát");
        report.say("shutdown có trật tự, outbox rỗng");
        pool.close().await;
        Ok(report)
    })
    .await
}

/// `POST /v1/entries` và gRPC `PostEntry` với JWT: 401, 403, 201, 200, 409, 422, 400, 415 đúng hợp đồng
/// Problem Details; idempotency chung giữa HTTP và gRPC; bút toán thủ công cũng ra `LedgerEntryPosted`.
///
/// # Errors
/// Như [`flow_and_replay`].
pub async fn http_and_grpc_contract() -> Result<Report> {
    LabEnv::run(|env| async move {
        let mut report = Report::default();
        let probe = Probe::new(&env)?;
        let (app, _clock) = start(&env).await?;
        let (fin, cs) = (env.token(&["finance"])?, env.token(&["cs"])?);
        let topup = json!({
            "idempotencyKey": "manual:topup-CU-001",
            "postings": [
                { "account": "customer:CU-001", "amountDong": 100_000 },
                { "account": "psp:vixanh:clearing", "amountDong": -100_000 }
            ]
        });
        let http = app.http_addr;

        // Xác thực và phân quyền: lỗi đến trước khi đọc body.
        let anonymous = http_call(http, "POST", "/v1/entries", None, Some(&topup), None).await?;
        anonymous.expect_problem(401, "unauthenticated")?;
        ensure!(anonymous.www_authenticate.is_some(), "401 phải có WWW-Authenticate");
        http_post(http, "/v1/entries", &cs, &topup).await?.expect_problem(403, "forbidden")?;
        let auth = env.auth()?;
        let claims = |aud: &str, exp: u64| Claims {
            sub: "svc-lab".to_string(),
            roles: vec!["finance".to_string()],
            aud: aud.to_string(),
            exp,
        };
        let bad_tokens = [
            ("sai audience", auth.issue(&claims("ledger-khac", now_secs() + 300))),
            ("hết hạn", auth.issue(&claims(AUDIENCE, now_secs() - 3_600))),
            ("thuật toán HS512", auth.sign(Algorithm::HS512, &claims(AUDIENCE, now_secs() + 300))),
            ("không phải JWT", "khong.phai.jwt".to_string()),
        ];
        for (label, token) in &bad_tokens {
            http_post(http, "/v1/entries", token, &topup)
                .await?
                .expect_problem(401, "unauthenticated")
                .with_context(|| format!("token {label}"))?;
        }
        report.say("401 (không token, sai audience, hết hạn, HS512, rác) và 403 (cs ghi) đều là problem+json");

        // Ghi, replay, xung đột, lệch.
        let created = http_post(http, "/v1/entries", &fin, &topup).await?;
        ensure!(created.status == 201, "HTTP {}: {}", created.status, created.body);
        let created_json = created.json()?;
        ensure!(created_json["entryId"] == "JE-000001" && created_json["replayed"] == false, "{created_json}");
        let posted = probe.event_for("manual:topup-CU-001").await?;
        ensure!(posted.entry_id == "JE-000001", "sự kiện phải mang cùng mã bút toán");
        let again = http_post(http, "/v1/entries", &fin, &topup).await?;
        ensure!(again.status == 200 && again.json()?["replayed"] == true, "replay: {}", again.body);
        let other = json!({
            "idempotencyKey": "manual:topup-CU-001",
            "postings": [
                { "account": "customer:CU-001", "amountDong": 5_000 },
                { "account": "psp:vixanh:clearing", "amountDong": -5_000 }
            ]
        });
        http_post(http, "/v1/entries", &fin, &other).await?.expect_problem(409, "idempotency-conflict")?;
        let skewed = json!({
            "idempotencyKey": "manual:skewed",
            "postings": [
                { "account": "customer:CU-001", "amountDong": 1_000 },
                { "account": "psp:vixanh:clearing", "amountDong": -900 }
            ]
        });
        http_post(http, "/v1/entries", &fin, &skewed).await?.expect_problem(422, "unbalanced-entry")?;
        report.say("201 JE-000001, replay 200, cùng key khác nội dung 409, lệch 422; sự kiện đã phát");

        // Hình dạng sai: gom mọi lỗi theo trường; JSON hỏng; thiếu content-type; tài khoản ngoài sơ đồ.
        let shape = json!({
            "idempotencyKey": " ",
            "postings": [
                { "account": "", "amountDong": 5 },
                { "account": "driver:DR-001", "amountDong": 0 }
            ]
        });
        let problem = http_post(http, "/v1/entries", &fin, &shape).await?.expect_problem(422, "validation-failed")?;
        ensure!(problem["errors"].as_array().is_some_and(|e| e.len() == 3), "errors: {problem}");
        let broken = {
            let client = Client::builder(TokioExecutor::new()).build_http::<Full<Bytes>>();
            let request = hyper::Request::builder()
                .method("POST")
                .uri(format!("http://{http}/v1/entries"))
                .header("authorization", format!("Bearer {fin}"))
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from_static(b"{khong phai json")))?;
            let response = client.request(request).await?;
            let status = response.status().as_u16();
            let ctype = response.headers().get("content-type").and_then(|v| v.to_str().ok()).map(str::to_string);
            (status, ctype)
        };
        ensure!(broken.0 == 400 && broken.1.as_deref() == Some("application/problem+json"), "{broken:?}");
        let unknown_kind = json!({
            "idempotencyKey": "manual:odd",
            "postings": [
                { "account": "la:1", "amountDong": -5 },
                { "account": "platform:revenue", "amountDong": 5 }
            ]
        });
        http_post(http, "/v1/entries", &fin, &unknown_kind).await?.expect_problem(422, "invalid-request")?;
        let echoed = http_call(http, "POST", "/v1/entries", Some(&fin), Some(&topup), Some("req-contract-1")).await?;
        ensure!(echoed.request_id.as_deref() == Some("req-contract-1"), "x-request-id phải được trả lại");
        report.say("422 gom 3 lỗi theo trường, 400 JSON hỏng, 422 tài khoản ngoài sơ đồ, x-request-id được trả lại");
        expect_balance(&app, &cs, "customer:CU-001", 100_000).await?;

        // gRPC: cùng JWT, cùng use case, cùng idempotency với HTTP.
        let mut client = grpc_client(app.grpc_addr).await?;
        let no_token = client
            .get_balance(pb::GetBalanceRequest { account: "customer:CU-001".into() })
            .await
            .expect_err("không token phải bị từ chối");
        ensure!(no_token.code() == tonic::Code::Unauthenticated, "mã {:?}", no_token.code());
        let line = |account: &str, amount_dong: i64| pb::Posting { account: account.to_string(), amount_dong };
        let request = |key: &str, postings: Vec<pb::Posting>| pb::PostEntryRequest { idempotency_key: key.to_string(), postings };
        let topup_lines = vec![line("customer:CU-001", 100_000), line("psp:vixanh:clearing", -100_000)];
        let forbidden = client
            .post_entry(authed(request("manual:topup-CU-001", topup_lines.clone()), &cs)?)
            .await
            .expect_err("cs không được ghi");
        ensure!(forbidden.code() == tonic::Code::PermissionDenied, "mã {:?}", forbidden.code());
        let via_grpc = client
            .post_entry(authed(request("manual:topup-CU-001", topup_lines), &fin)?)
            .await?
            .into_inner();
        ensure!(via_grpc.replayed && via_grpc.entry_id == "JE-000001", "gRPC phải replay bút toán của HTTP");
        let conflict = client
            .post_entry(authed(request("manual:topup-CU-001", vec![line("customer:CU-001", 1), line("psp:vixanh:clearing", -1)]), &fin)?)
            .await
            .expect_err("cùng key khác nội dung");
        ensure!(conflict.code() == tonic::Code::AlreadyExists, "mã {:?}", conflict.code());
        let skewed = client
            .post_entry(authed(request("manual:skewed-grpc", vec![line("customer:CU-001", 1_000), line("psp:vixanh:clearing", -900)]), &fin)?)
            .await
            .expect_err("lệch");
        ensure!(skewed.code() == tonic::Code::InvalidArgument, "mã {:?}", skewed.code());
        let missing = client
            .get_balance(authed(pb::GetBalanceRequest { account: "driver:DR-404".into() }, &cs)?)
            .await
            .expect_err("không có tài khoản");
        ensure!(missing.code() == tonic::Code::NotFound, "mã {:?}", missing.code());
        report.say("gRPC: Unauthenticated, PermissionDenied, replay chung với HTTP, AlreadyExists, InvalidArgument, NotFound");

        ensure!(!app.shutdown().await.timed_out, "shutdown quá hạn");
        Ok(report)
    })
    .await
}

/// Hold rồi capture khi chuyến hoàn thành; hold hết hạn theo đồng hồ giả; hoàn tiền một phần hai lần
/// tới hết; mọi lỗi nghiệp vụ trả đúng Problem Details.
///
/// # Errors
/// Như [`flow_and_replay`].
pub async fn hold_and_refund() -> Result<Report> {
    LabEnv::run(|env| async move {
        let mut report = Report::default();
        let probe = Probe::new(&env)?;
        let (app, clock) = start(&env).await?;
        let (fin, cs) = (env.token(&["finance"])?, env.token(&["cs"])?);
        let http = app.http_addr;
        let pool = env.inspect_pool().await?;

        // Nạp ví khách 100.000 qua POST /v1/entries (tiền từ ngoài vào hiện thành số âm ở clearing).
        let topup = json!({
            "idempotencyKey": "manual:topup-CU-001",
            "postings": [
                { "account": "customer:CU-001", "amountDong": 100_000 },
                { "account": "psp:vixanh:clearing", "amountDong": -100_000 }
            ]
        });
        ensure!(http_post(http, "/v1/entries", &fin, &topup).await?.status == 201, "nạp ví");
        probe.event_for("manual:topup-CU-001").await?;

        let available = |label: &'static str| {
            let (cs, http) = (cs.clone(), http);
            async move {
                let reply = http_get(http, "/v1/accounts/customer:CU-001/available", Some(&cs)).await?;
                ensure!(reply.status == 200, "{label}: HTTP {} {}", reply.status, reply.body);
                let json = reply.json()?;
                let field = |name: &str| json[name].as_i64().with_context(|| format!("{label}: thiếu {name}"));
                Ok::<_, anyhow::Error>((field("balanceDong")?, field("heldDong")?, field("availableDong")?))
            }
        };
        ensure!(available("sau nạp").await? == (100_000, 0, 100_000), "sau nạp ví");

        // Hold 30.000: khả dụng còn 70.000; hold thứ hai 80.000 bị từ chối.
        let hold = |key: &str, amount: i64| json!({ "account": "customer:CU-001", "amountDong": amount, "idempotencyKey": key });
        let first = http_post(http, "/v1/holds", &fin, &hold("trip:TN-0001:hold", 30_000)).await?;
        ensure!(first.status == 201 && first.json()?["replayed"] == false, "hold: {}", first.body);
        let replay = http_post(http, "/v1/holds", &fin, &hold("trip:TN-0001:hold", 30_000)).await?;
        ensure!(replay.status == 200 && replay.json()?["holdId"] == first.json()?["holdId"], "replay hold");
        http_post(http, "/v1/holds", &fin, &hold("trip:TN-0001:hold", 31_000)).await?.expect_problem(409, "idempotency-conflict")?;
        http_post(http, "/v1/holds", &fin, &hold("trip:TN-0009:hold", 80_000)).await?.expect_problem(422, "insufficient-available")?;
        ensure!(available("sau hold").await? == (100_000, 30_000, 70_000), "sau hold");
        report.say("nạp 100000, hold 30000 → khả dụng 70000; hold 80000 bị 422 insufficient-available");

        // Chuyến hoàn thành: capture 25.500 đóng hold TN-0001 trong cùng transaction.
        send_trip(&env, &TripCompleted::sample("TN-0001", fare_for_km(5).dong())).await?;
        probe.event_for("trip:TN-0001:capture").await?;
        probe.event_for("trip:TN-0001:split").await?;
        let row: (String, i64) = sqlx::query_as("SELECT status, captured_dong FROM holds WHERE idempotency_key = 'trip:TN-0001:hold'")
            .fetch_one(&pool)
            .await?;
        ensure!(row == ("CAPTURED".to_string(), 25_500), "hold sau capture: {row:?}");
        ensure!(available("sau capture").await? == (74_500, 0, 74_500), "sau capture");
        report.say("TripCompleted TN-0001: hold CAPTURED 25500, phần dư 4500 nhả; ví còn 74500, khả dụng 74500");

        // Hết hạn bằng đồng hồ giả: không sleep. Hold 30.000 hạn 900 giây.
        ensure!(http_post(http, "/v1/holds", &fin, &hold("trip:TN-0002:hold", 30_000)).await?.status == 201, "hold 2");
        ensure!(available("hold 2").await? == (74_500, 30_000, 44_500), "hold 2");
        clock.advance(jiff::SignedDuration::from_secs(901));
        ensure!(available("sau 901 giây").await? == (74_500, 0, 74_500), "hold hết hạn không còn trừ khả dụng");
        send_trip(&env, &TripCompleted::sample("TN-0002", fare_for_km(1).dong())).await?;
        probe.event_for("trip:TN-0002:capture").await?;
        probe.event_for("trip:TN-0002:split").await?;
        let row: (String, i64) = sqlx::query_as("SELECT status, captured_dong FROM holds WHERE idempotency_key = 'trip:TN-0002:hold'")
            .fetch_one(&pool)
            .await?;
        ensure!(row == ("EXPIRED".to_string(), 0), "hold hết hạn: {row:?}");
        ensure!(available("sau TN-0002").await? == (62_500, 0, 62_500), "sau TN-0002");
        report.say("đồng hồ giả +901 giây: hold hết hạn, khả dụng về 74500; TN-0002 đóng hold EXPIRED, ví còn 62500");

        // Hoàn tiền một phần TN-0001: 10.000, rồi nốt 15.500.
        let refund = |n: u32, amount: i64| json!({ "refundNo": n, "amountDong": amount });
        let path = "/v1/trips/TN-0001/refunds";
        let first = http_post(http, path, &fin, &refund(1, 10_000)).await?;
        ensure!(first.status == 201, "hoàn 1: {} {}", first.status, first.body);
        let entry_id = first.json()?["entryId"].clone();
        let replay = http_post(http, path, &fin, &refund(1, 10_000)).await?;
        ensure!(replay.status == 200 && replay.json()?["entryId"] == entry_id, "replay hoàn");
        http_post(http, path, &fin, &refund(1, 9_000)).await?.expect_problem(409, "idempotency-conflict")?;
        http_post(http, path, &fin, &refund(2, 15_501)).await?.expect_problem(422, "refund-exceeds-refundable")?;
        http_post(http, "/v1/trips/TN-9999/refunds", &fin, &refund(1, 1)).await?.expect_problem(404, "unknown-trip")?;
        http_post(http, path, &cs, &refund(2, 1)).await?.expect_problem(403, "forbidden")?;
        let last = http_post(http, path, &fin, &refund(2, 15_500)).await?;
        ensure!(last.status == 201, "hoàn 2: {} {}", last.status, last.body);
        let posted = probe.event_for("trip:TN-0001:refund:1").await?;
        ensure!(
            amounts(&posted) == [("customer:CU-001", 10_000), ("platform:revenue", -2_000), ("driver:DR-001", -8_000)],
            "hoàn 1: {:?}",
            amounts(&posted)
        );
        let payment: (String, i64) = sqlx::query_as("SELECT status, refunded_dong FROM payments WHERE trip_id = 'TN-0001'")
            .fetch_one(&pool)
            .await?;
        ensure!(payment == ("REFUNDED".to_string(), 25_500), "payments: {payment:?}");
        // Hoàn đủ 25.500: tài xế và doanh thu của TN-0001 về đúng phần của TN-0002 (9.600 và 2.400).
        expect_balance(&app, &cs, "driver:DR-001", 9_600).await?;
        expect_balance(&app, &cs, "platform:revenue", 2_400).await?;
        expect_balance(&app, &cs, "customer:CU-001", 100_000 - 12_000).await?;
        ensure!(
            count(&pool, "SELECT count(*) FROM journal_entries WHERE idempotency_key LIKE 'trip:TN-0001:refund:%'").await? == 2,
            "đúng hai bút toán hoàn"
        );
        report.say("hoàn 10000 (201), replay 200, khác số 409, vượt 422, chuyến lạ 404, cs 403; hoàn nốt 15500 → REFUNDED");
        report.say("sau hoàn đủ TN-0001: tài xế 9600, doanh thu 2400, ví khách 88000 (chỉ còn TN-0002)");

        ensure!(!app.shutdown().await.timed_out, "shutdown quá hạn");
        pool.close().await;
        Ok(report)
    })
    .await
}

fn sqlstate(result: Result<impl Sized, sqlx::Error>) -> String {
    match result {
        Ok(_) => "OK".to_string(),
        Err(sqlx::Error::Database(db)) => db.code().map_or("?".to_string(), |c| c.to_string()),
        Err(other) => format!("lỗi khác: {other}"),
    }
}

/// Hai trigger của DB chặn đúng thứ cần chặn kể cả khi bỏ qua application: bút toán lệch, thiếu posting,
/// UPDATE/DELETE/TRUNCATE sổ; và hai CHECK của `holds`, `payments`.
///
/// # Errors
/// Một chốt chặn không chặn, hoặc chặn bằng mã lạ.
pub async fn database_guards() -> Result<Report> {
    LabEnv::run(|env| async move {
        let mut report = Report::default();
        let cfg = env.config()?;
        // Chỉ để dựng bảng bằng đúng bốn file SQL của service.
        let store = PgStore::connect(&cfg.pg_url, cfg.pg_schema.as_deref(), &cfg.topic_out).await?;
        let pool = env.inspect_pool().await?;
        sqlx::query("INSERT INTO accounts (id, kind) VALUES ('customer:CU-001', 'liability'), ('platform:revenue', 'revenue')")
            .execute(&pool)
            .await?;

        // Ghi thẳng bằng SQL một bút toán với các dòng cho trước; lỗi hiện ra lúc COMMIT.
        async fn raw_entry(pool: &PgPool, key: &str, lines: &[i64]) -> Result<(), sqlx::Error> {
            let mut tx = pool.begin().await?;
            let id: i64 = sqlx::query_scalar("INSERT INTO journal_entries (idempotency_key) VALUES ($1) RETURNING id")
                .bind(key)
                .fetch_one(&mut *tx)
                .await?;
            for (i, amount) in lines.iter().enumerate() {
                let account = if i == 0 { "customer:CU-001" } else { "platform:revenue" };
                sqlx::query("INSERT INTO postings (entry_id, line_no, account_id, amount_dong) VALUES ($1, $2, $3, $4)")
                    .bind(id)
                    .bind(i32::try_from(i + 1).expect("ít dòng"))
                    .bind(account)
                    .bind(amount)
                    .execute(&mut *tx)
                    .await?;
            }
            tx.commit().await
        }

        ensure!(sqlstate(raw_entry(&pool, "guard:ok", &[-5_100, 5_100]).await) == "OK", "bút toán cân phải qua");
        let unbalanced = sqlstate(raw_entry(&pool, "guard:skewed", &[-5_100, 5_000]).await);
        let single = sqlstate(raw_entry(&pool, "guard:single", &[-5_100]).await);
        ensure!(unbalanced == "23514" && single == "23514", "lệch {unbalanced}, thiếu posting {single}");
        report.say("bút toán cân qua; lệch tổng và thiếu posting bị 23514 (check_violation) lúc COMMIT");

        let update = sqlstate(sqlx::query("UPDATE postings SET amount_dong = 1").execute(&pool).await);
        let delete = sqlstate(sqlx::query("DELETE FROM journal_entries").execute(&pool).await);
        let truncate = sqlstate(sqlx::query("TRUNCATE postings").execute(&pool).await);
        ensure!(
            update == "TN001" && delete == "TN001" && truncate == "TN001",
            "append-only: UPDATE {update}, DELETE {delete}, TRUNCATE {truncate}"
        );
        report.say("UPDATE, DELETE, TRUNCATE trên sổ đều bị TN001 (append-only)");

        let hold = sqlstate(
            sqlx::query("INSERT INTO holds (account_id, amount_dong, status, idempotency_key, expires_at) VALUES ('customer:CU-001', 0, 'ACTIVE', 'k', now())")
                .execute(&pool)
                .await,
        );
        ensure!(hold == "23514", "hold 0 đồng: {hold}");
        sqlx::query("INSERT INTO payments (trip_id, customer_id, driver_id, captured_dong, fee_dong, status) VALUES ('TN-1', 'CU-001', 'DR-001', 100, 20, 'CAPTURED')")
            .execute(&pool)
            .await?;
        let over = sqlstate(sqlx::query("UPDATE payments SET refunded_dong = 101 WHERE trip_id = 'TN-1'").execute(&pool).await);
        ensure!(over == "23514", "hoàn vượt số thu: {over}");
        report.say("CHECK: hold 0 đồng và hoàn vượt số đã thu đều bị 23514, kể cả khi bỏ qua khóa dòng");
        pool.close().await;
        store.pool().close().await;
        Ok(report)
    })
    .await
}

/// `account_balances` khóa theo thứ tự mã tài khoản, không theo thứ tự dòng trong bút toán.
/// Không dựng deadlock và không đo giờ: giữ khóa của tài khoản đứng đầu, chờ writer bị chặn
/// đúng bởi phiên đó (`pg_blocking_pids`), rồi kiểm tài khoản đứng sau vẫn chưa bị khóa.
///
/// # Errors
/// Writer khóa sai thứ tự, hoặc không bị chặn trước hạn chót.
pub async fn balance_lock_order() -> Result<Report> {
    LabEnv::run(|env| async move {
        let mut report = Report::default();
        let cfg = env.config()?;
        let store = PgStore::connect(&cfg.pg_url, cfg.pg_schema.as_deref(), &cfg.topic_out).await?;
        let pool = env.inspect_pool().await?;
        // Dòng clearing đứng trước, trong khi mã `platform:revenue` nhỏ hơn `psp:vixanh:clearing`.
        let entry = |key: &str| -> Result<JournalEntry> {
            Ok(JournalEntry::try_new(
                key,
                vec![
                    Posting::new(AccountId::new(CLEARING)?, Money::from_dong(-100)),
                    Posting::new(AccountId::new(REVENUE)?, Money::from_dong(100)),
                ],
            )?)
        };
        store.post_entry(&entry("lock:seed")?).await?;

        let mut holder = pool.begin().await?;
        let holder_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *holder)
            .await?;
        sqlx::query("SELECT 1 FROM account_balances WHERE account_id = $1 FOR UPDATE")
            .bind(REVENUE)
            .execute(&mut *holder)
            .await?;

        let writer = tokio::spawn({
            let store = store.clone();
            let second = entry("lock:second")?;
            async move { store.post_entry(&second).await }
        });
        let deadline = Instant::now() + WITHIN;
        loop {
            let blocked: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM pg_stat_activity WHERE $1 = ANY(pg_blocking_pids(pid))",
            )
            .bind(holder_pid)
            .fetch_one(&pool)
            .await?;
            if blocked > 0 {
                break;
            }
            ensure!(
                Instant::now() < deadline,
                "writer không bị chặn sau {WITHIN:?}"
            );
            sleep(Duration::from_millis(50)).await;
        }

        // Writer đang chờ revenue. Nếu nó khóa theo thứ tự dòng thì clearing đã bị giữ: 55P03.
        let clearing = sqlstate(
            sqlx::query("SELECT 1 FROM account_balances WHERE account_id = $1 FOR UPDATE NOWAIT")
                .bind(CLEARING)
                .execute(&pool)
                .await,
        );
        ensure!(clearing == "OK", "clearing đã bị khóa trước: {clearing}");
        report.say("bút toán ghi clearing trước revenue, nhưng khóa revenue trước; revenue bị giữ thì clearing vẫn trống");

        holder.rollback().await?;
        let written = writer.await??;
        ensure!(matches!(written, PostOutcome::Created(_)), "{written:?}");
        let revenue: i64 =
            sqlx::query_scalar("SELECT balance_dong FROM account_balances WHERE account_id = $1")
                .bind(REVENUE)
                .fetch_one(&pool)
                .await?;
        ensure!(revenue == 200, "revenue {revenue}");
        report.say("nhả khóa: bút toán ghi xong, revenue 200");
        pool.close().await;
        store.pool().close().await;
        Ok(report)
    })
    .await
}

/// Chạy binary thật (`exe serve`), gửi SIGTERM và kiểm: thoát mã 0, log JSON có `request_id`.
///
/// # Errors
/// Như [`flow_and_replay`], cộng lỗi tạo/giết tiến trình con.
pub async fn sigterm_child(exe: &Path) -> Result<Report> {
    LabEnv::run(|env| async move {
        let mut report = Report::default();
        let mut child = tokio::process::Command::new(exe)
            .arg("serve")
            .envs(env.vars())
            .env("RUST_LOG", "info")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .context("không chạy được binary ledger-service")?;
        let pid = child.id().context("không có pid")?;
        let mut lines = BufReader::new(child.stdout.take().context("không có stdout")?).lines();

        // Đọc stdout của tiến trình con TỚI EOF: lưu mọi dòng, gửi cổng thật ngay khi thấy "listening".
        let seen = Arc::new(Mutex::new(Vec::<Value>::new()));
        let (addr_tx, addr_rx) = tokio::sync::oneshot::channel::<(SocketAddr, SocketAddr)>();
        let reader = tokio::spawn({
            let seen = seen.clone();
            async move {
                let mut addr_tx = Some(addr_tx);
                while let Ok(Some(line)) = lines.next_line().await {
                    let Ok(json) = serde_json::from_str::<Value>(&line) else {
                        continue;
                    };
                    let fields = &json["fields"];
                    if fields["message"] == "listening"
                        && let (Some(h), Some(g), Some(tx)) = (
                            fields["http_addr"].as_str(),
                            fields["grpc_addr"].as_str(),
                            addr_tx.take(),
                        )
                    {
                        let _ = tx.send((h.parse().expect("địa chỉ HTTP"), g.parse().expect("địa chỉ gRPC")));
                    }
                    seen.lock().expect("khoá log").push(json);
                }
            }
        });
        let (http_addr, grpc_addr) = timeout(WITHIN * 2, addr_rx)
            .await
            .context("binary không in dòng listening")??;
        wait_ready(http_addr).await?;

        let probe = Probe::new(&env)?;
        send_trip(&env, &TripCompleted::sample("TN-0001", fare_for_km(5).dong())).await?;
        probe.event_for("trip:TN-0001:capture").await?;
        probe.event_for("trip:TN-0001:split").await?;
        let cs = env.token(&["cs"])?;
        let reply = http_call(
            http_addr,
            "GET",
            "/v1/accounts/driver:DR-001/balance",
            Some(&cs),
            None,
            Some("req-e2e-0001"),
        )
        .await?;
        ensure!(reply.status == 200 && reply.body.contains("20400"), "HTTP: {} {}", reply.status, reply.body);
        ensure!(reply.request_id.as_deref() == Some("req-e2e-0001"), "header x-request-id phải được trả lại");
        ensure!(grpc_balance(grpc_addr, &cs, "driver:DR-001").await? == 20_400, "gRPC");
        report.say("tiến trình con trả 20400 qua HTTP (x-request-id được trả lại) và gRPC, có JWT");

        // SIGTERM như orchestrator làm khi tắt pod.
        let status = tokio::process::Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()
            .await?;
        ensure!(status.success(), "kill -TERM thất bại");
        let exit = timeout(WITHIN, child.wait())
            .await
            .context("tiến trình không thoát sau SIGTERM")??;
        ensure!(exit.success(), "mã thoát {exit}");
        reader.await?;

        let logs = seen.lock().expect("khoá log");
        let has = |pred: &dyn Fn(&Value) -> bool| logs.iter().any(pred);
        ensure!(
            has(&|l| l["span"]["name"] == "http" && l["span"]["request_id"] == "req-e2e-0001"),
            "log JSON không có span http mang request_id req-e2e-0001"
        );
        let done = logs.iter().find(|l| l["fields"]["message"] == "shutdown complete");
        ensure!(done.is_some(), "thiếu dòng shutdown complete");
        ensure!(
            done.map(|l| l["fields"]["timed_out"].clone()) == Some(json!(false)),
            "shutdown quá hạn"
        );
        // Khóa JWT chỉ có trong biến môi trường: không dòng log nào được chứa nó.
        ensure!(
            !logs.iter().any(|l| l.to_string().contains(&env.jwt_secret)),
            "khóa JWT lọt vào log"
        );
        report.say(format!(
            "SIGTERM: thoát mã 0, {} dòng log JSON, có request_id và 'shutdown complete', không lộ khóa JWT",
            logs.len()
        ));
        Ok(report)
    })
    .await
}
