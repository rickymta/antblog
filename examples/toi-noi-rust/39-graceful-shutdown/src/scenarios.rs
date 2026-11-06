//! Các kịch bản của bài: mỗi hàm chạy trong một ca cô lập và trả về số liệu thật.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use rdkafka::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use tokio::task::JoinHandle;

use crate::api::{AppState, router};
use crate::client::{http_get, http_post};
use crate::lab::{Counts, Lab};
use crate::metrics::Metrics;
use crate::probes::Probes;
use crate::process::{Proc, describe};
use crate::service::{Config, ShutdownReport, build_consumer, build_pool, build_producer, start};

/// Chạy `body` trong một ca; luôn dọn schema và topic, kể cả khi `body` lỗi.
///
/// # Errors
/// Lỗi tạo/dọn ca hoặc lỗi của `body`.
pub async fn in_lab<T>(body: impl AsyncFnOnce(&Lab) -> Result<T>) -> Result<T> {
    let lab = Lab::new().await?;
    let out = body(&lab).await;
    let cleanup = lab.cleanup().await;
    let out = out?;
    cleanup?;
    Ok(out)
}

async fn wait_until(timeout: Duration, what: &str, mut cond: impl FnMut() -> bool) -> Result<()> {
    let deadline = Instant::now() + timeout;
    while !cond() {
        if Instant::now() > deadline {
            bail!("hết {timeout:?} mà chưa thấy: {what}");
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    Ok(())
}

async fn readyz_tag(addr: SocketAddr) -> String {
    match http_get(addr, "/readyz").await {
        Ok((status, _)) => status.to_string(),
        Err(_) => "refused".to_string(),
    }
}

/// Gọi `/readyz` liên tục, ghi lại dãy trạng thái đã thấy (`200`, `503`, `refused`) theo thứ tự bắt đầu.
/// Mỗi nhịp một request riêng: khi máy tải cao một lần `/readyz` có thể chậm tới `PROBE_TIMEOUT`,
/// gọi nối đuôi nhau sẽ lỡ cả cửa sổ 503.
async fn poll_readyz(addr: SocketAddr, stop: Arc<AtomicBool>) -> Vec<String> {
    let mut probes = Vec::new();
    while !stop.load(Ordering::Acquire) {
        probes.push(tokio::spawn(readyz_tag(addr)));
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    probes.push(tokio::spawn(readyz_tag(addr))); // service đã tắt xong: phải bị từ chối
    let mut timeline: Vec<String> = Vec::new();
    for probe in probes {
        let tag = probe.await.unwrap_or_else(|_| "refused".to_string());
        if timeline.last() != Some(&tag) {
            timeline.push(tag);
        }
    }
    timeline
}

// ---------------------------------------------------------------- tắt đẹp ----

/// Số liệu của kịch bản tắt đẹp rồi chạy lại.
#[derive(Debug)]
pub struct GracefulReport {
    /// Số chuyến đã gửi.
    pub total: u32,
    /// Dãy trạng thái `/readyz` quan sát được trong lúc tắt.
    pub readyz: Vec<String>,
    /// Báo cáo tắt của lần chạy đầu.
    pub first: ShutdownReport,
    /// Số tin lần đầu đã xử lý xong khi tắt xong.
    pub done_first: u64,
    /// Offset đã commit sau lần tắt đầu.
    pub committed_after_first: i64,
    /// Tin lần chạy thứ hai nhận lại mà phải bỏ vì trùng.
    pub second_duplicates: u64,
    /// Bút toán lần chạy thứ hai ghi thêm.
    pub second_posted: u64,
    /// Thời gian lần chạy thứ hai từ lúc khởi động đến khi đủ bút toán, mili giây.
    pub second_ms: u128,
    /// Số liệu DB cuối cùng.
    pub counts: Counts,
    /// Mọi key trên topic phát ra.
    pub out_keys: Vec<String>,
}

/// Chạy chậm, tắt giữa chừng bằng huỷ token (cùng đường đi với SIGTERM), rồi chạy lại.
///
/// # Errors
/// Mọi lỗi hạ tầng.
pub async fn graceful() -> Result<GracefulReport> {
    in_lab(async |lab| {
        let total = 24;
        lab.produce_trips(1, total).await?;
        let mut cfg = lab.config();
        cfg.work_delay = Duration::from_millis(40);
        cfg.readiness_grace = Duration::from_millis(100);
        cfg.queue_capacity = 8;
        cfg.resume_depth = 6;

        let svc = start(cfg.clone()).await?;
        let metrics = svc.metrics.clone();
        // Tắt khi đã xong 8 tin VÀ còn ít nhất 4 tin đang nằm trong hàng đợi hoặc đang xử lý.
        wait_until(
            Duration::from_secs(30),
            "8 tin xong và còn việc dở",
            || metrics.done() >= 8 && Metrics::get(&metrics.received) >= metrics.done() + 4,
        )
        .await?;

        let stop = Arc::new(AtomicBool::new(false));
        let poller = tokio::spawn(poll_readyz(svc.addr, stop.clone()));
        tokio::time::sleep(Duration::from_millis(100)).await; // để poller kịp thấy 200
        let first = svc.shutdown().await;
        stop.store(true, Ordering::Release);
        let readyz = poller.await?;

        let done_first = Metrics::get(&metrics.posted);
        let committed_after_first = lab.committed_offset().await?;

        let mut cfg2 = cfg;
        cfg2.work_delay = Duration::ZERO;
        let started = Instant::now();
        let svc2 = start(cfg2).await?;
        let metrics2 = svc2.metrics.clone();
        lab.wait_entries(i64::from(total), Duration::from_secs(30))
            .await?;
        let second_ms = started.elapsed().as_millis();
        svc2.shutdown().await;

        Ok(GracefulReport {
            total,
            readyz,
            first,
            done_first,
            committed_after_first,
            second_duplicates: Metrics::get(&metrics2.duplicates),
            second_posted: Metrics::get(&metrics2.posted),
            second_ms,
            counts: lab.counts().await?,
            out_keys: lab.out_keys().await?,
        })
    })
    .await
}

// ------------------------------------------------------------- HTTP đang bay ----

/// Số liệu của kịch bản một request HTTP đang chạy khi bắt đầu tắt.
#[derive(Debug)]
pub struct HttpDrainReport {
    /// Status của request đang bay.
    pub post_status: u16,
    /// Request trả lời SAU khi lệnh tắt đã bắt đầu.
    pub reply_after_shutdown_began: bool,
    /// Status `/readyz` trong thời gian grace.
    pub readyz_in_grace: u16,
    /// Kết nối mới sau khi ngừng nhận việc bị từ chối.
    pub refused_after: bool,
    /// Số bút toán trong DB sau đó.
    pub entries: i64,
    /// Dòng outbox còn chưa phát.
    pub outbox_pending: i64,
}

/// POST chậm (800 ms), bắt đầu tắt sau 200 ms: request vẫn phải xong.
///
/// # Errors
/// Mọi lỗi hạ tầng.
pub async fn http_drain() -> Result<HttpDrainReport> {
    in_lab(async |lab| {
        let mut cfg = lab.config();
        cfg.work_delay = Duration::from_millis(800);
        cfg.readiness_grace = Duration::from_millis(300);
        let svc = start(cfg).await?;
        let addr = svc.addr;

        let body = r#"{"idempotencyKey":"manual:drain-1","postings":[
            {"account":"customer:CU-001","amountDong":-10000},
            {"account":"driver:DR-001","amountDong":10000}]}"#;
        let post = tokio::spawn(async move {
            let result = http_post(addr, "/v1/entries", body).await;
            (result, Instant::now())
        });
        tokio::time::sleep(Duration::from_millis(200)).await;

        let shutdown_began = Instant::now();
        let shutdown: JoinHandle<ShutdownReport> = tokio::spawn(svc.shutdown());
        tokio::time::sleep(Duration::from_millis(100)).await;
        let (readyz_in_grace, _) = http_get(addr, "/readyz").await?;

        let (result, replied_at) = post.await?;
        let (post_status, _) = result?;
        shutdown.await?;
        let refused_after = http_get(addr, "/healthz").await.is_err();
        let counts = lab.counts().await?;
        Ok(HttpDrainReport {
            post_status,
            reply_after_shutdown_began: replied_at > shutdown_began,
            readyz_in_grace,
            refused_after,
            entries: counts.entries,
            outbox_pending: counts.outbox_pending,
        })
    })
    .await
}

// ------------------------------------------------------------ timeout khi tắt ----

/// Số liệu của kịch bản drain quá hạn.
#[derive(Debug)]
pub struct TimeoutReport {
    /// Báo cáo tắt của lần chạy đầu.
    pub first: ShutdownReport,
    /// Tin đã xong trước khi tắt.
    pub done_first: u64,
    /// DB ngay sau khi tắt (transaction dở đã rollback).
    pub after_first: Counts,
    /// Offset đã commit sau lần tắt đầu.
    pub committed_after_first: i64,
    /// Tin trùng ở lần chạy lại.
    pub second_duplicates: u64,
    /// DB cuối cùng.
    pub counts: Counts,
}

/// Mỗi tin chậm 1,5 s nhưng drain chỉ cho 0,4 s: task bị huỷ, transaction dở rollback.
///
/// # Errors
/// Mọi lỗi hạ tầng.
pub async fn timeout() -> Result<TimeoutReport> {
    in_lab(async |lab| {
        lab.produce_trips(1, 6).await?;
        let mut cfg = lab.config();
        cfg.work_delay = Duration::from_millis(1500);
        cfg.drain_timeout = Duration::from_millis(400);
        cfg.readiness_grace = Duration::ZERO;
        let svc = start(cfg.clone()).await?;
        let metrics = svc.metrics.clone();
        wait_until(
            Duration::from_secs(30),
            "một tin xong và một tin đang dở",
            || metrics.done() >= 1 && Metrics::get(&metrics.in_flight) == 1,
        )
        .await?;
        let first = svc.shutdown().await;
        let done_first = metrics.done();
        let after_first = lab.counts().await?;
        let committed_after_first = lab.committed_offset().await?;

        cfg.work_delay = Duration::ZERO;
        cfg.drain_timeout = Duration::from_secs(5);
        let svc2 = start(cfg).await?;
        let metrics2 = svc2.metrics.clone();
        lab.wait_entries(6, Duration::from_secs(30)).await?;
        svc2.shutdown().await;
        Ok(TimeoutReport {
            first,
            done_first,
            after_first,
            committed_after_first,
            second_duplicates: Metrics::get(&metrics2.duplicates),
            counts: lab.counts().await?,
        })
    })
    .await
}

// ----------------------------------------------------------------- backpressure ----

/// Một lần chạy backpressure với một cặp (sức chứa, ngưỡng resume).
#[derive(Debug)]
pub struct BackpressureRun {
    /// Sức chứa hàng đợi.
    pub capacity: usize,
    /// Ngưỡng resume: số tin còn lại trong hàng đợi.
    pub resume_depth: usize,
    /// Tin consumer đã nhận.
    pub received: u64,
    /// Tin xử lý xong.
    pub done: u64,
    /// Số tin tối đa từng nằm trong hàng đợi.
    pub max_depth: u64,
    /// Số lần pause.
    pub paused: u64,
    /// Số lần resume.
    pub resumed: u64,
    /// Số lần poll trong lúc đang pause.
    pub polls_while_paused: u64,
    /// Lỗi consumer.
    pub consumer_errors: u64,
    /// Thời gian xử lý hết, mili giây (số đo trên máy viết bài, không phải SLA).
    pub elapsed_ms: u128,
    /// DB cuối cùng.
    pub counts: Counts,
}

/// Số liệu của kịch bản worker chậm hơn nguồn tin, chạy với hai ngưỡng resume.
#[derive(Debug)]
pub struct BackpressureReport {
    /// Số tin đã gửi cho mỗi lần chạy.
    pub total: u32,
    /// Mỗi tin worker mất bao lâu, mili giây.
    pub work_ms: u64,
    /// Hàng đợi nhỏ, resume sớm: worker bị bỏ đói mỗi lần resume.
    pub narrow: BackpressureRun,
    /// Hàng đợi lớn, resume khi còn nhiều việc: worker không bị bỏ đói.
    pub wide: BackpressureRun,
}

async fn backpressure_run(
    total: u32,
    work_ms: u64,
    capacity: usize,
    resume_depth: usize,
) -> Result<BackpressureRun> {
    in_lab(async |lab| {
        lab.produce_trips(1, total).await?;
        let mut cfg = lab.config();
        cfg.work_delay = Duration::from_millis(work_ms);
        cfg.readiness_grace = Duration::ZERO;
        cfg.queue_capacity = capacity;
        cfg.resume_depth = resume_depth;
        let started = Instant::now();
        let svc = start(cfg).await?;
        let metrics = svc.metrics.clone();
        wait_until(Duration::from_secs(60), "xử lý hết các tin", || {
            metrics.done() >= u64::from(total)
        })
        .await?;
        let elapsed_ms = started.elapsed().as_millis();
        svc.shutdown().await;
        Ok(BackpressureRun {
            capacity,
            resume_depth,
            received: Metrics::get(&metrics.received),
            done: metrics.done(),
            max_depth: Metrics::get(&metrics.max_depth),
            paused: Metrics::get(&metrics.paused),
            resumed: Metrics::get(&metrics.resumed),
            polls_while_paused: Metrics::get(&metrics.polls_while_paused),
            consumer_errors: Metrics::get(&metrics.consumer_errors),
            elapsed_ms,
            counts: lab.counts().await?,
        })
    })
    .await
}

/// 60 tin gửi một lượt, worker 30 ms mỗi tin: một lần hàng đợi 4 resume sớm, một lần hàng đợi 32.
///
/// # Errors
/// Mọi lỗi hạ tầng.
pub async fn backpressure() -> Result<BackpressureReport> {
    let (total, work_ms) = (60, 30);
    let (narrow, wide) = tokio::try_join!(
        backpressure_run(total, work_ms, 4, 2),
        backpressure_run(total, work_ms, 32, 24)
    )?;
    Ok(BackpressureReport {
        total,
        work_ms,
        narrow,
        wide,
    })
}

// ------------------------------------------------------- max.poll.interval.ms ----

/// Số liệu của hai consumer cùng `max.poll.interval.ms`.
#[derive(Debug)]
pub struct PollReport {
    /// `max.poll.interval.ms` của cả hai consumer.
    pub max_poll_ms: u32,
    /// Thời gian mỗi consumer "đứng yên", mili giây.
    pub idle_ms: u128,
    /// Consumer ngừng poll: lỗi nhận được sau khi tỉnh dậy (nếu có).
    pub no_poll_error: Option<String>,
    /// Consumer pause nhưng vẫn poll: số lần poll không có gì.
    pub paused_polls: u64,
    /// Consumer pause nhưng vẫn poll: số tin nhận được trong lúc pause.
    pub paused_messages: u64,
    /// Consumer pause nhưng vẫn poll: số lỗi nhận được.
    pub paused_errors: u64,
    /// Sau resume vẫn nhận được tin.
    pub resumed_ok: bool,
}

fn poll_config(lab: &Lab) -> Config {
    let mut cfg = lab.config();
    cfg.session_timeout_ms = 6000;
    cfg.max_poll_interval_ms = 6000;
    cfg
}

async fn no_poll_run(lab: &Lab, idle: Duration) -> Result<Option<String>> {
    lab.produce_trips(1, 3).await?;
    let consumer = build_consumer(&poll_config(lab))?;
    consumer.subscribe(&[lab.topic_in.as_str()])?;
    tokio::time::timeout(Duration::from_secs(30), consumer.recv())
        .await
        .context("không nhận được tin đầu")??;
    tokio::time::sleep(idle).await; // KHÔNG poll trong khoảng này
    for _ in 0..8 {
        match tokio::time::timeout(Duration::from_secs(3), consumer.recv()).await {
            Ok(Err(err)) => return Ok(Some(err.to_string())),
            Ok(Ok(_)) => {}
            Err(_) => break,
        }
    }
    Ok(None)
}

async fn paused_run(lab: &Lab, idle: Duration) -> Result<(u64, u64, u64, bool)> {
    lab.produce_trips(1, 3).await?;
    let consumer = build_consumer(&poll_config(lab))?;
    consumer.subscribe(&[lab.topic_in.as_str()])?;
    tokio::time::timeout(Duration::from_secs(30), consumer.recv())
        .await
        .context("không nhận được tin đầu")??;
    let assignment = consumer.assignment()?;
    consumer.pause(&assignment)?;
    lab.produce_trips(4, 6).await?; // có tin mới, nhưng partition đang pause
    let (mut polls, mut messages, mut errors) = (0, 0, 0);
    let end = Instant::now() + idle;
    while Instant::now() < end {
        match tokio::time::timeout(Duration::from_millis(200), consumer.recv()).await {
            Err(_) => polls += 1,
            Ok(Ok(_)) => messages += 1,
            Ok(Err(_)) => errors += 1,
        }
    }
    consumer.resume(&assignment)?;
    let resumed_ok = matches!(
        tokio::time::timeout(Duration::from_secs(10), consumer.recv()).await,
        Ok(Ok(_))
    );
    Ok((polls, messages, errors, resumed_ok))
}

/// Hai consumer: một ngừng poll, một pause nhưng vẫn poll, cùng đứng yên 8 giây.
///
/// # Errors
/// Mọi lỗi hạ tầng.
pub async fn poll_interval() -> Result<PollReport> {
    let idle = Duration::from_secs(8);
    let (lab_a, lab_b) = tokio::try_join!(Lab::new(), Lab::new())?;
    let (no_poll, paused) = tokio::join!(no_poll_run(&lab_a, idle), paused_run(&lab_b, idle));
    let (clean_a, clean_b) = tokio::join!(lab_a.cleanup(), lab_b.cleanup());
    let no_poll_error = no_poll?;
    let (paused_polls, paused_messages, paused_errors, resumed_ok) = paused?;
    clean_a?;
    clean_b?;
    Ok(PollReport {
        max_poll_ms: 6000,
        idle_ms: idle.as_millis(),
        no_poll_error,
        paused_polls,
        paused_messages,
        paused_errors,
        resumed_ok,
    })
}

// ------------------------------------------------- độ trễ sau resume (chỉ đo) ----

/// Độ trễ từ `resume` tới tin đầu tiên, mili giây: nhỏ nhất, trung vị, lớn nhất.
#[derive(Debug)]
pub struct ResumeLatency {
    /// Giá trị `fetch.wait.max.ms` của lần đo.
    pub fetch_wait_ms: u32,
    /// Số vòng pause rồi resume.
    pub rounds: usize,
    /// Nhỏ nhất.
    pub min_ms: u128,
    /// Trung vị.
    pub median_ms: u128,
    /// Lớn nhất.
    pub max_ms: u128,
}

async fn resume_latency_run(fetch_wait_ms: u32) -> Result<ResumeLatency> {
    let rounds = 20_usize;
    in_lab(async |lab| {
        lab.produce_trips(1, rounds as u32 + 2).await?;
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &lab.config().kafka)
            .set("group.id", &lab.group)
            .set("auto.offset.reset", "earliest")
            .set("enable.auto.commit", "false")
            .set("fetch.wait.max.ms", fetch_wait_ms.to_string())
            .create()?;
        consumer.subscribe(&[lab.topic_in.as_str()])?;
        tokio::time::timeout(Duration::from_secs(30), consumer.recv())
            .await
            .context("không nhận được tin đầu")??;
        let assignment = consumer.assignment()?;
        let mut samples = Vec::with_capacity(rounds);
        for round in 0..rounds {
            consumer.pause(&assignment)?;
            // Pause dài ngắn khác nhau (100 đến 400 ms), vẫn poll trong lúc đó.
            let end = Instant::now() + Duration::from_millis(100 + (round as u64 * 37) % 300);
            while Instant::now() < end {
                let _ = tokio::time::timeout(Duration::from_millis(20), consumer.recv()).await;
            }
            let resumed = Instant::now();
            consumer.resume(&assignment)?;
            tokio::time::timeout(Duration::from_secs(10), consumer.recv())
                .await
                .context("resume xong không có tin")??;
            samples.push(resumed.elapsed().as_millis());
        }
        samples.sort_unstable();
        Ok(ResumeLatency {
            fetch_wait_ms,
            rounds,
            min_ms: samples[0],
            median_ms: samples[rounds / 2],
            max_ms: samples[rounds - 1],
        })
    })
    .await
}

/// Đo độ trễ sau `resume` ở `fetch.wait.max.ms` mặc định và ở giá trị rất nhỏ. Không nằm trong `--verify`.
///
/// # Errors
/// Mọi lỗi hạ tầng.
pub async fn resume_latency() -> Result<[ResumeLatency; 2]> {
    let (default_wait, short_wait) =
        tokio::try_join!(resume_latency_run(500), resume_latency_run(20))?;
    Ok([default_wait, short_wait])
}

// ----------------------------------------------------------- healthz và readyz ----

/// Một dòng kết quả: tên tình huống, status `/healthz`, status `/readyz`, body `/readyz`.
#[derive(Debug)]
pub struct ProbeRow {
    /// Tình huống.
    pub name: &'static str,
    /// Status `/healthz`.
    pub health: u16,
    /// Status `/readyz`.
    pub ready: u16,
    /// Body `/readyz`.
    pub body: String,
    /// Thời gian `/readyz` trả lời, mili giây.
    pub ready_ms: u128,
}

async fn serve_state(state: AppState) -> Result<SocketAddr> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
        let _ = axum::serve(listener, router(state)).await;
    });
    Ok(addr)
}

async fn probe_row(name: &'static str, state: AppState) -> Result<ProbeRow> {
    let addr = serve_state(state).await?;
    let (health, _) = http_get(addr, "/healthz").await?;
    let started = Instant::now();
    let (ready, body) = http_get(addr, "/readyz").await?;
    Ok(ProbeRow {
        name,
        health,
        ready,
        body,
        ready_ms: started.elapsed().as_millis(),
    })
}

/// Bốn tình huống: bình thường, đang tắt, PostgreSQL mất, Kafka mất.
///
/// # Errors
/// Mọi lỗi hạ tầng.
pub async fn probes() -> Result<Vec<ProbeRow>> {
    in_lab(async |lab| {
        let cfg = lab.config();
        let make = |pool, producer, draining| AppState {
            probes: Arc::new(Probes::new(pool, producer, cfg.topic_out.clone())),
            pool: lab.pool.clone(),
            draining: Arc::new(AtomicBool::new(draining)),
            topic_out: Arc::from(cfg.topic_out.as_str()),
            hold: Duration::ZERO,
        };
        let good_producer = build_producer(&cfg.kafka)?;
        // Kafka "mất": producer trỏ vào cổng không ai lắng nghe.
        let dead_producer = build_producer("127.0.0.1:1")?;
        // PostgreSQL "mất": pool đã đóng, mọi truy vấn trả lỗi ngay.
        let closed_pool = build_pool(&cfg.pg_url, &cfg.schema).await?;
        closed_pool.close().await;

        Ok(vec![
            probe_row(
                "bình thường",
                make(lab.pool.clone(), good_producer.clone(), false),
            )
            .await?,
            probe_row(
                "đang tắt",
                make(lab.pool.clone(), good_producer.clone(), true),
            )
            .await?,
            probe_row("PostgreSQL mất", make(closed_pool, good_producer, false)).await?,
            probe_row("Kafka mất", make(lab.pool.clone(), dead_producer, false)).await?,
        ])
    })
    .await
}

// -------------------------------------------------------- bị kill ngang xương ----

/// Số liệu của kịch bản SIGKILL rồi chạy lại.
#[derive(Debug)]
pub struct KillReport {
    /// Cách tiến trình đầu kết thúc.
    pub first_exit: String,
    /// Số tín hiệu giết nó (9 nếu SIGKILL).
    pub first_signal: Option<i32>,
    /// DB ngay sau khi bị giết.
    pub at_kill: Counts,
    /// Thời gian từ lúc spawn lần hai đến khi đủ bút toán, mili giây (gồm chờ `session.timeout.ms`).
    pub restart_ms: u128,
    /// Tin trùng lần hai tự báo trong log.
    pub second_duplicates: Option<u64>,
    /// Mã thoát của lần hai sau SIGTERM.
    pub second_exit_code: Option<i32>,
    /// DB cuối cùng.
    pub counts: Counts,
    /// Mọi key trên topic phát ra.
    pub out_keys: Vec<String>,
}

/// Lấy số sau `name=` trong dòng log `summary`.
pub fn log_number(logs: &[String], name: &str) -> Option<u64> {
    let line = logs.iter().rev().find(|l| l.contains("summary"))?;
    let rest = line.split(&format!("{name}=")).nth(1)?;
    rest.split_whitespace().next()?.parse().ok()
}

/// Tiến trình con `serve` bị `kill -KILL` giữa chừng, rồi một tiến trình mới đọc tiếp.
///
/// # Errors
/// Mọi lỗi hạ tầng.
pub async fn kill_restart(exe: &Path) -> Result<KillReport> {
    in_lab(async |lab| {
        let total = 24;
        lab.produce_trips(1, total).await?;
        let mut cfg = lab.config();
        cfg.work_delay = Duration::from_millis(40);
        // Chu kỳ commit dài hơn cả bài: offset chưa kịp lên broker lúc bị giết, tức là tình huống xấu nhất.
        cfg.commit_interval_ms = 60_000;

        let mut first = Proc::spawn(exe, &cfg).await?;
        lab.wait_entries(8, Duration::from_secs(30)).await?;
        first.signal("KILL").await?;
        let status = first.wait(Duration::from_secs(10)).await?;
        let at_kill = lab.counts().await?;

        cfg.work_delay = Duration::ZERO;
        let started = Instant::now();
        let mut second = Proc::spawn(exe, &cfg).await?;
        lab.wait_entries(i64::from(total), Duration::from_secs(60))
            .await?;
        let restart_ms = started.elapsed().as_millis();
        second.signal("TERM").await?;
        let status2 = second.wait(Duration::from_secs(20)).await?;
        let logs = second.logs();
        use std::os::unix::process::ExitStatusExt;
        Ok(KillReport {
            first_exit: describe(&status),
            first_signal: status.signal(),
            at_kill,
            restart_ms,
            second_duplicates: log_number(&logs, "duplicates"),
            second_exit_code: status2.code(),
            counts: lab.counts().await?,
            out_keys: lab.out_keys().await?,
        })
    })
    .await
}
