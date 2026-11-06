//! Các kịch bản end-to-end dùng chung cho `cargo run`, `--verify` và `cargo test`.
//! Mỗi kịch bản tự dựng một [`LabEnv`] riêng nên chạy song song được và không để lại dữ liệu.

use std::net::SocketAddr;
use std::path::Path;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail, ensure};
use http_body_util::{BodyExt, Empty};
use hyper::body::Bytes;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use ledger_core::fare_for_km;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::{ClientConfig, Message};
use sqlx::PgPool;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time::{Instant, sleep, timeout, timeout_at};

use crate::app::App;
use crate::config::Fault;
use crate::consumer::ConsumerExit;
use crate::events::{LedgerEntryPosted, TripCompleted};
use crate::grpc::pb;
use crate::harness::LabEnv;
use crate::kafka;
use crate::state::Stats;

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

    /// Chờ `LedgerEntryPosted` có đúng `key`, bỏ qua sự kiện khác. Có hạn chót, không ngủ cố định.
    async fn event_for(&self, key: &str, within: Duration) -> Result<LedgerEntryPosted> {
        let deadline = Instant::now() + within;
        loop {
            let received = timeout_at(deadline, self.consumer.recv())
                .await
                .map_err(|_| anyhow!("không thấy LedgerEntryPosted của {key} sau {within:?}"))?;
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

/// Kết quả của một `GET`.
struct Reply {
    status: u16,
    request_id: Option<String>,
    body: String,
}

async fn http_get(addr: SocketAddr, path: &str, request_id: Option<&str>) -> Result<Reply> {
    let client = Client::builder(TokioExecutor::new()).build_http::<Empty<Bytes>>();
    let mut request = hyper::Request::builder().uri(format!("http://{addr}{path}"));
    if let Some(id) = request_id {
        request = request.header("x-request-id", id);
    }
    let response = client.request(request.body(Empty::new())?).await?;
    let status = response.status().as_u16();
    let request_id = response
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let body = String::from_utf8(response.into_body().collect().await?.to_bytes().to_vec())?;
    Ok(Reply {
        status,
        request_id,
        body,
    })
}

async fn http_balance(addr: SocketAddr, account: &str) -> Result<i64> {
    let reply = http_get(addr, &format!("/v1/accounts/{account}/balance"), None).await?;
    ensure!(
        reply.status == 200,
        "HTTP {} cho {account}: {}",
        reply.status,
        reply.body
    );
    let json: serde_json::Value = serde_json::from_str(&reply.body)?;
    ensure!(
        json["account"] == account,
        "thân trả về sai tài khoản: {json}"
    );
    ensure!(json["asOf"].is_string(), "thiếu asOf: {json}");
    json["balanceDong"]
        .as_i64()
        .context("balanceDong không phải số nguyên")
}

async fn grpc_balance(addr: SocketAddr, account: &str) -> Result<i64> {
    let mut client = pb::ledger_client::LedgerClient::connect(format!("http://{addr}")).await?;
    let reply = client
        .get_balance(pb::GetBalanceRequest {
            account: account.to_string(),
        })
        .await?
        .into_inner();
    ensure!(reply.account == account, "gRPC trả sai tài khoản");
    ensure!(reply.as_of.is_some(), "gRPC thiếu as_of");
    Ok(reply.balance_dong)
}

/// Số dư phải bằng nhau qua cả HTTP lẫn gRPC.
async fn expect_balance(app: &App, account: &str, expected: i64) -> Result<()> {
    let over_http = http_balance(app.http_addr, account).await?;
    let over_grpc = grpc_balance(app.grpc_addr, account).await?;
    ensure!(
        over_http == expected && over_grpc == expected,
        "{account}: HTTP {over_http}, gRPC {over_grpc}, mong {expected}"
    );
    Ok(())
}

/// Chờ tới khi `cond` đúng hoặc hết hạn. Kiểm tra mỗi 25 ms, không ngủ một khoảng cố định.
async fn wait_until(what: &str, within: Duration, mut cond: impl FnMut() -> bool) -> Result<()> {
    let deadline = Instant::now() + within;
    while !cond() {
        if Instant::now() >= deadline {
            bail!("quá {within:?} mà chưa thấy: {what}");
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

// ---------- kịch bản ----------

/// TripCompleted → bút toán → LedgerEntryPosted → số dư qua HTTP và gRPC; phát lại không tạo bút toán thứ hai.
///
/// # Errors
/// Mọi kỳ vọng sai, hạ tầng thiếu, hoặc quá hạn chờ.
pub async fn flow_and_replay() -> Result<Report> {
    LabEnv::run(|env| async move {
        let mut report = Report::default();
        let app = App::start(env.config()?).await?;
        let probe = Probe::new(&env)?;
        wait_ready(app.http_addr).await?;
        report.say("readyz 200 khi consumer đã được gán partition");

        // 1. Chuyến TN-0001, 5 km, cước 25.500.
        let trip1 = TripCompleted::sample("TN-0001", fare_for_km(5).dong());
        send_trip(&env, &trip1).await?;
        let posted = probe.event_for("trip:TN-0001:capture", WITHIN).await?;
        ensure!(posted.entry_id == "JE-000001", "entryId {}", posted.entry_id);
        ensure!(
            amounts(&posted)
                == [("customer:CU-001", -25_500), ("platform:revenue", 5_100), ("driver:DR-001", 20_400)],
            "postings {:?}",
            amounts(&posted)
        );
        report.say(format!("LedgerEntryPosted {} {:?}", posted.entry_id, amounts(&posted)));

        // 2. Số dư qua HTTP và gRPC.
        expect_balance(&app, "driver:DR-001", 20_400).await?;
        expect_balance(&app, "platform:revenue", 5_100).await?;
        expect_balance(&app, "customer:CU-001", -25_500).await?;
        report.say("số dư driver:DR-001 = 20400, platform:revenue = 5100, customer:CU-001 = -25500 (HTTP và gRPC khớp)");
        let missing = http_get(app.http_addr, "/v1/accounts/driver:DR-404/balance", None).await?;
        ensure!(missing.status == 404, "tài khoản lạ phải 404, nhận {}", missing.status);
        ensure!(http_get(app.http_addr, "/healthz", None).await?.status == 200, "healthz");

        // 3. Phát lại ĐÚNG sự kiện đó, rồi một chuyến mới làm "cột mốc": khi cột mốc đã ra,
        //    sự kiện phát lại (cùng partition, đứng trước) chắc chắn đã được xử lý.
        send_trip(&env, &trip1).await?;
        let trip2 = TripCompleted::sample("TN-0002", fare_for_km(1).dong());
        send_trip(&env, &trip2).await?;
        let second = probe.event_for("trip:TN-0002:capture", WITHIN).await?;
        ensure!(second.entry_id == "JE-000002", "cột mốc phải là JE-000002, nhận {}", second.entry_id);
        wait_until("duplicates >= 1", WITHIN, || Stats::get(&app.stats.duplicates) >= 1).await?;

        let pool = env.inspect_pool().await?;
        let entries = count(&pool, "SELECT count(*) FROM journal_entries").await?;
        let same_key = count(&pool, "SELECT count(*) FROM journal_entries WHERE idempotency_key = 'trip:TN-0001:capture'").await?;
        let inbox = count(&pool, "SELECT count(*) FROM inbox").await?;
        ensure!(entries == 2 && same_key == 1 && inbox == 2, "entries {entries}, same_key {same_key}, inbox {inbox}");
        ensure!(Stats::get(&app.stats.processed) == 2 && Stats::get(&app.stats.duplicates) == 1, "bộ đếm");
        expect_balance(&app, "driver:DR-001", 20_400 + 9_600).await?;
        report.say("phát lại TripCompleted TN-0001: inbox chặn, vẫn 2 bút toán, 1 bút toán cho key TN-0001");

        // 4. Tắt có trật tự: readyz đổi 503 trước, healthz vẫn 200, outbox rỗng khi xong.
        app.readiness.begin_shutdown();
        ensure!(http_get(app.http_addr, "/readyz", None).await?.status == 503, "readyz khi tắt");
        ensure!(http_get(app.http_addr, "/healthz", None).await?.status == 200, "healthz khi tắt");
        let done = app.shutdown().await;
        ensure!(!done.timed_out, "shutdown quá hạn");
        let pending = count(&pool, "SELECT count(*) FROM outbox WHERE published_at IS NULL").await?;
        ensure!(pending == 0, "outbox còn {pending} dòng chưa phát");
        report.say(format!("shutdown trong {} ms, outbox rỗng", done.elapsed.as_millis()));
        pool.close().await;
        Ok(report)
    })
    .await
}

/// Process chết giữa chừng rồi chạy lại: không mất bút toán, không ghi đôi, vẫn phát đúng một `LedgerEntryPosted` hợp lệ.
///
/// # Errors
/// Như [`flow_and_replay`].
pub async fn crash_and_restart(fault: Fault) -> Result<Report> {
    ensure!(fault != Fault::None, "cần một điểm chết");
    LabEnv::run(|env| async move {
        let mut report = Report::default();
        let probe = Probe::new(&env)?;
        let mut cfg = env.config()?;
        cfg.fault = fault;
        let mut first = App::start(cfg).await?;
        wait_ready(first.http_addr).await?;

        let trip = TripCompleted::sample("TN-0003", fare_for_km(3).dong());
        send_trip(&env, &trip).await?;
        let exit = timeout(WITHIN, first.wait_consumer())
            .await
            .context("consumer không chết")??;
        ensure!(exit == ConsumerExit::Crashed, "consumer phải chết giả lập");
        let pool = env.inspect_pool().await?;
        let (entries, inbox) = (
            count(&pool, "SELECT count(*) FROM journal_entries").await?,
            count(&pool, "SELECT count(*) FROM inbox").await?,
        );
        let expected_before = i64::from(fault == Fault::AfterDbCommitBeforeOffset);
        ensure!(
            entries == expected_before && inbox == expected_before,
            "sau khi chết ({fault:?}): entries {entries}, inbox {inbox}"
        );
        report.say(format!(
            "process chết ({fault:?}): DB có {entries} bút toán, offset chưa commit"
        ));
        first.kill(); // hủy cứng mọi task, không xả gì

        // Chạy lại, cùng consumer group: Kafka giao lại message vì offset chưa commit.
        let second = App::start(env.config()?).await?;
        let posted = probe.event_for("trip:TN-0003:capture", WITHIN).await?;
        ensure!(
            amounts(&posted)
                == [
                    ("customer:CU-001", -16_500),
                    ("platform:revenue", 3_300),
                    ("driver:DR-001", 13_200)
                ],
            "postings {:?}",
            amounts(&posted)
        );
        expect_balance(&second, "driver:DR-001", 13_200).await?;
        let entries = count(&pool, "SELECT count(*) FROM journal_entries").await?;
        ensure!(
            entries == 1,
            "sau khi chạy lại phải đúng 1 bút toán, có {entries}"
        );
        let (processed, duplicates) = (
            Stats::get(&second.stats.processed),
            Stats::get(&second.stats.duplicates),
        );
        match fault {
            Fault::AfterDbCommitBeforeOffset => {
                wait_until("giao lại bị inbox chặn", WITHIN, || {
                    Stats::get(&second.stats.duplicates) == 1
                })
                .await?;
                ensure!(processed == 0, "bản chạy lại không được ghi thêm");
            }
            _ => ensure!(
                processed == 1 && duplicates == 0,
                "bản chạy lại phải ghi 1 lần"
            ),
        }
        report.say(format!(
            "chạy lại: {} -> 1 bút toán {}, số dư tài xế 13200",
            posted.idempotency_key, posted.entry_id
        ));
        ensure!(!second.shutdown().await.timed_out, "shutdown quá hạn");
        pool.close().await;
        Ok(report)
    })
    .await
}

/// gRPC `PostEntry`: idempotent, xung đột và lệch được trả đúng mã `Status`; bút toán cũng ra `LedgerEntryPosted`.
///
/// # Errors
/// Như [`flow_and_replay`].
pub async fn grpc_post_entry() -> Result<Report> {
    LabEnv::run(|env| async move {
        let mut report = Report::default();
        let app = App::start(env.config()?).await?;
        let probe = Probe::new(&env)?;
        let mut client =
            pb::ledger_client::LedgerClient::connect(format!("http://{}", app.grpc_addr)).await?;
        let line = |account: &str, amount_dong: i64| pb::Posting {
            account: account.to_string(),
            amount_dong,
        };
        let request = |postings| pb::PostEntryRequest {
            idempotency_key: "manual:adjust-001".to_string(),
            postings,
        };
        let body = vec![
            line("psp:vixanh:clearing", -1_000),
            line("platform:revenue", 1_000),
        ];

        let first = client.post_entry(request(body.clone())).await?.into_inner();
        ensure!(!first.replayed, "lần đầu không phải replay");
        let again = client.post_entry(request(body)).await?.into_inner();
        ensure!(
            again.replayed && again.entry_id == first.entry_id,
            "lần hai phải replay cùng bút toán"
        );
        let conflict = client
            .post_entry(request(vec![
                line("psp:vixanh:clearing", -2_000),
                line("platform:revenue", 2_000),
            ]))
            .await
            .expect_err("cùng key khác nội dung phải bị từ chối");
        ensure!(
            conflict.code() == tonic::Code::AlreadyExists,
            "mã {:?}",
            conflict.code()
        );
        let skewed = client
            .post_entry(pb::PostEntryRequest {
                idempotency_key: "manual:skewed".to_string(),
                postings: vec![
                    line("psp:vixanh:clearing", -1_000),
                    line("platform:revenue", 900),
                ],
            })
            .await
            .expect_err("bút toán lệch phải bị từ chối");
        ensure!(
            skewed.code() == tonic::Code::InvalidArgument,
            "mã {:?}",
            skewed.code()
        );
        let zero = client
            .post_entry(pb::PostEntryRequest {
                idempotency_key: "manual:zero".to_string(),
                postings: vec![line("psp:vixanh:clearing", 0), line("platform:revenue", 0)],
            })
            .await
            .expect_err("posting 0 đồng phải bị từ chối");
        ensure!(
            zero.code() == tonic::Code::InvalidArgument,
            "mã {:?}",
            zero.code()
        );
        let posted = probe.event_for("manual:adjust-001", WITHIN).await?;
        ensure!(
            posted.entry_id == first.entry_id,
            "sự kiện phải mang cùng mã bút toán"
        );
        expect_balance(&app, "platform:revenue", 1_000).await?;
        report.say(format!(
            "PostEntry {}: replay, AlreadyExists, InvalidArgument đúng mã; sự kiện đã phát",
            first.entry_id
        ));
        ensure!(!app.shutdown().await.timed_out, "shutdown quá hạn");
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
            .context("không chạy được binary ledger-api")?;
        let pid = child.id().context("không có pid")?;
        let mut lines = BufReader::new(child.stdout.take().context("không có stdout")?).lines();

        // Đọc stdout của tiến trình con: lưu mọi dòng, và gửi cổng thật ngay khi thấy dòng "listening".
        let seen = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
        let (addr_tx, addr_rx) = tokio::sync::oneshot::channel::<(SocketAddr, SocketAddr)>();
        let reader = tokio::spawn({
            let seen = seen.clone();
            async move {
                let mut addr_tx = Some(addr_tx);
                while let Ok(Some(line)) = lines.next_line().await {
                    let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) else {
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
                        let _ = tx.send((
                            h.parse().expect("địa chỉ HTTP"),
                            g.parse().expect("địa chỉ gRPC"),
                        ));
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
        send_trip(
            &env,
            &TripCompleted::sample("TN-0001", fare_for_km(5).dong()),
        )
        .await?;
        probe.event_for("trip:TN-0001:capture", WITHIN).await?;
        let reply = http_get(
            http_addr,
            "/v1/accounts/driver:DR-001/balance",
            Some("req-e2e-0001"),
        )
        .await?;
        ensure!(
            reply.status == 200 && reply.body.contains("20400"),
            "HTTP: {} {}",
            reply.status,
            reply.body
        );
        ensure!(
            reply.request_id.as_deref() == Some("req-e2e-0001"),
            "header x-request-id phải được trả lại"
        );
        ensure!(
            grpc_balance(grpc_addr, "driver:DR-001").await? == 20_400,
            "gRPC"
        );
        report.say("tiến trình con trả 20400 qua HTTP (x-request-id được trả lại) và gRPC");

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
        let has = |pred: &dyn Fn(&serde_json::Value) -> bool| logs.iter().any(pred);
        ensure!(
            has(&|l| l["span"]["name"] == "http" && l["span"]["request_id"] == "req-e2e-0001"),
            "log JSON không có span http mang request_id req-e2e-0001"
        );
        ensure!(
            has(&|l| l["fields"]["message"] == "shutdown complete"),
            "thiếu dòng shutdown complete"
        );
        let timed_out = logs
            .iter()
            .find(|l| l["fields"]["message"] == "shutdown complete")
            .map(|l| l["fields"]["timed_out"].clone());
        ensure!(
            timed_out == Some(serde_json::json!(false)),
            "shutdown quá hạn: {timed_out:?}"
        );
        report.say(format!(
            "SIGTERM: thoát mã 0, {} dòng log JSON, có request_id và 'shutdown complete'",
            logs.len()
        ));
        Ok(report)
    })
    .await
}
