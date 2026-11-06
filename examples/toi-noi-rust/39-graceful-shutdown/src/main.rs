//! `cargo run` in kết quả các kịch bản; `-- --verify` kiểm giá trị kỳ vọng;
//! `-- serve` chạy service thật, tắt bằng SIGTERM hoặc ctrl-c.

use std::process::ExitCode;

use anyhow::Result;
use ledger_39_graceful_shutdown::checks::{
    check_backpressure, check_graceful, check_http_drain, check_kill, check_poll, check_probes,
    check_timeout,
};
use ledger_39_graceful_shutdown::lab::Lab;
use ledger_39_graceful_shutdown::scenarios::{
    self, BackpressureReport, GracefulReport, HttpDrainReport, KillReport, PollReport, ProbeRow,
    TimeoutReport,
};
use ledger_39_graceful_shutdown::service::{self, Config};
use ledger_39_graceful_shutdown::signals::Signals;
use tracing::{error, info};

fn init_logging() {
    // sqlx và librdkafka rất nhiều lời: chỉ giữ cảnh báo của chúng.
    let filter = tracing_subscriber::EnvFilter::new("info,sqlx=warn,librdkafka=off");
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .without_time()
        .with_target(false)
        .init();
}

#[tokio::main]
async fn main() -> ExitCode {
    init_logging();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("serve") {
        return serve().await;
    }
    let verify = args.iter().any(|a| a == "--verify");
    // Một tên kịch bản (ví dụ `graceful`) chạy riêng kịch bản đó.
    let only = args.iter().find(|a| !a.starts_with("--")).cloned();
    match demo(verify, only.as_deref()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("LỖI: {err:#}");
            ExitCode::FAILURE
        }
    }
}

/// Service thật: có `LAB_SCHEMA` thì dùng ca của tiến trình cha, không thì tự tạo ca và tự dọn.
async fn serve() -> ExitCode {
    // Đăng ký handler TRƯỚC khi báo READY.
    let mut signals = match Signals::install() {
        Ok(signals) => signals,
        Err(err) => {
            error!("không đăng ký được tín hiệu: {err}");
            return ExitCode::FAILURE;
        }
    };
    let (cfg, owned) = match std::env::var("LAB_SCHEMA") {
        Ok(_) => match Config::from_env() {
            Ok(cfg) => (cfg, None),
            Err(err) => {
                error!("cấu hình sai: {err:#}");
                return ExitCode::FAILURE;
            }
        },
        Err(_) => match Lab::new().await {
            Ok(lab) => (lab.config(), Some(lab)),
            Err(err) => {
                error!("không tạo được ca: {err:#}");
                return ExitCode::FAILURE;
            }
        },
    };
    let running = match service::start(cfg).await {
        Ok(running) => running,
        Err(err) => {
            error!("không khởi động được: {err:#}");
            return ExitCode::FAILURE;
        }
    };
    println!("READY addr={} pid={}", running.addr, std::process::id());

    let first = signals.recv().await;
    info!("nhận {first}: bắt đầu tắt");
    // Tín hiệu thứ hai nghĩa là "tắt ngay": thoát mã 4, bỏ qua mọi bước còn lại.
    tokio::spawn(async move {
        let second = signals.recv().await;
        error!("nhận {second} lần hai: thoát ngay, không chờ drain");
        std::process::exit(4);
    });

    let metrics = running.metrics.clone();
    let report = running.shutdown().await;
    info!(
        "summary exit_code={} posted={} duplicates={} done_in_grace={} done_in_drain={} total_ms={}",
        report.exit_code(),
        scenarios_get(&metrics.posted),
        scenarios_get(&metrics.duplicates),
        report.done_in_grace,
        report.done_in_drain,
        report.total_ms
    );
    if let Some(lab) = owned
        && let Err(err) = lab.cleanup().await
    {
        error!("dọn ca lỗi: {err:#}");
    }
    ExitCode::from(report.exit_code())
}

fn scenarios_get(counter: &std::sync::atomic::AtomicU64) -> u64 {
    ledger_39_graceful_shutdown::metrics::Metrics::get(counter)
}

async fn demo(verify: bool, only: Option<&str>) -> Result<()> {
    let exe = std::env::current_exe()?;
    let want = |name: &str| only.is_none_or(|o| o == name);
    // Chỉ đo, không kiểm: chạy khi gọi đúng tên, không nằm trong `--verify`.
    if only == Some("resume") {
        for r in scenarios::resume_latency().await? {
            println!(
                "fetch.wait.max.ms = {}: {} lần, nhỏ nhất {} ms, trung vị {} ms, lớn nhất {} ms",
                r.fetch_wait_ms, r.rounds, r.min_ms, r.median_ms, r.max_ms
            );
        }
        println!("(số đo trên máy viết bài, không phải SLA)");
        return Ok(());
    }

    if want("probes") {
        let rows = scenarios::probes().await?;
        print_probes(&rows);
        if verify {
            check_probes(&rows)?;
        }
    }
    if want("graceful") {
        let report = scenarios::graceful().await?;
        print_graceful(&report);
        if verify {
            check_graceful(&report)?;
        }
    }
    if want("http") {
        let report = scenarios::http_drain().await?;
        print_http_drain(&report);
        if verify {
            check_http_drain(&report)?;
        }
    }
    if want("timeout") {
        let report = scenarios::timeout().await?;
        print_timeout(&report);
        if verify {
            check_timeout(&report)?;
        }
    }
    if want("backpressure") {
        let report = scenarios::backpressure().await?;
        print_backpressure(&report);
        if verify {
            check_backpressure(&report)?;
        }
    }
    if want("poll") {
        let report = scenarios::poll_interval().await?;
        print_poll(&report);
        if verify {
            check_poll(&report)?;
        }
    }
    if want("kill") {
        let report = scenarios::kill_restart(&exe).await?;
        print_kill(&report);
        if verify {
            check_kill(&report, 24)?;
        }
    }
    if verify {
        println!("\nVERIFY OK");
    }
    Ok(())
}

fn print_probes(rows: &[ProbeRow]) {
    println!("== /healthz và /readyz ==");
    for r in rows {
        println!(
            "{:<15} healthz {}  readyz {}  ({} ms)  {}",
            r.name, r.health, r.ready, r.ready_ms, r.body
        );
    }
}

fn print_graceful(r: &GracefulReport) {
    println!(
        "\n== Tắt đẹp giữa chừng, rồi chạy lại ({} chuyến) ==",
        r.total
    );
    println!("readyz quan sát được: {}", r.readyz.join(" -> "));
    println!(
        "lần 1: xong {} tin; trong grace {}, trong drain {}; tắt mất {} ms; commit {:?}; outbox còn {:?}",
        r.done_first,
        r.first.done_in_grace,
        r.first.done_in_drain,
        r.first.total_ms,
        r.first.commit,
        r.first.outbox_pending
    );
    println!(
        "offset đã commit {} (bằng số tin đã xong {})",
        r.committed_after_first, r.done_first
    );
    println!(
        "lần 2: ghi thêm {}, tin trùng {}, mất {} ms để xong",
        r.second_posted, r.second_duplicates, r.second_ms
    );
    println!(
        "cuối: {} bút toán, {} key khác nhau, inbox {}, outbox {} (chưa phát {}), tổng posting {}, {} sự kiện trên topic",
        r.counts.entries,
        r.counts.distinct_keys,
        r.counts.inbox,
        r.counts.outbox_total,
        r.counts.outbox_pending,
        r.counts.postings_sum,
        r.out_keys.len()
    );
}

fn print_http_drain(r: &HttpDrainReport) {
    println!("\n== Request HTTP đang bay khi bắt đầu tắt ==");
    println!(
        "POST trả {} sau khi lệnh tắt đã bắt đầu: {}; /readyz trong grace: {}; kết nối mới sau đó bị từ chối: {}",
        r.post_status, r.reply_after_shutdown_began, r.readyz_in_grace, r.refused_after
    );
    println!(
        "bút toán {} , outbox chưa phát {}",
        r.entries, r.outbox_pending
    );
}

fn print_timeout(r: &TimeoutReport) {
    println!("\n== Drain quá hạn (mỗi tin 1500 ms, hạn 400 ms) ==");
    println!(
        "drain quá hạn: {}; mã thoát nếu là binary: {}; xong trước khi tắt {}",
        r.first.drain_timed_out,
        r.first.exit_code(),
        r.done_first
    );
    println!(
        "ngay sau tắt: {} bút toán, inbox {}, outbox {}, offset commit {}",
        r.after_first.entries,
        r.after_first.inbox,
        r.after_first.outbox_total,
        r.committed_after_first
    );
    println!(
        "chạy lại: tin trùng {}, cuối cùng {} bút toán, {} key khác nhau, tổng posting {}",
        r.second_duplicates, r.counts.entries, r.counts.distinct_keys, r.counts.postings_sum
    );
}

fn print_backpressure(r: &BackpressureReport) {
    println!(
        "\n== Backpressure: {} tin, worker {} ms mỗi tin (lý tưởng {} ms) ==",
        r.total,
        r.work_ms,
        u64::from(r.total) * r.work_ms
    );
    for (name, run) in [("hàng đợi nhỏ", &r.narrow), ("hàng đợi lớn", &r.wide)] {
        println!(
            "{name}: sức chứa {}, resume khi còn <= {}: nhận {}, xong {}, hàng đợi tối đa {}; pause {} lần, resume {} lần, poll trong lúc pause {} lần, lỗi consumer {}; xong sau {} ms",
            run.capacity,
            run.resume_depth,
            run.received,
            run.done,
            run.max_depth,
            run.paused,
            run.resumed,
            run.polls_while_paused,
            run.consumer_errors,
            run.elapsed_ms
        );
    }
    println!("(số đo trên máy viết bài, không phải SLA)");
}

fn print_poll(r: &PollReport) {
    println!(
        "\n== max.poll.interval.ms = {} ms, mỗi consumer đứng yên {} ms ==",
        r.max_poll_ms, r.idle_ms
    );
    println!("ngừng poll: lỗi nhận được = {:?}", r.no_poll_error);
    println!(
        "pause và vẫn poll: poll {} lần, tin giao trong lúc pause {}, lỗi {}, sau resume nhận lại tin: {}",
        r.paused_polls, r.paused_messages, r.paused_errors, r.resumed_ok
    );
}

fn print_kill(r: &KillReport) {
    println!("\n== SIGKILL giữa chừng rồi chạy lại ==");
    println!("tiến trình đầu: {}", r.first_exit);
    println!(
        "ngay sau khi bị giết: {} bút toán, inbox {}, outbox {}, tổng posting {}",
        r.at_kill.entries, r.at_kill.inbox, r.at_kill.outbox_total, r.at_kill.postings_sum
    );
    println!(
        "chạy lại: tin trùng {:?}, mất {} ms để đủ bút toán (gồm chờ session.timeout.ms), SIGTERM -> exit {:?}",
        r.second_duplicates, r.restart_ms, r.second_exit_code
    );
    println!(
        "cuối: {} bút toán, {} key khác nhau, tổng posting {}, {} sự kiện / {} key khác nhau trên topic",
        r.counts.entries,
        r.counts.distinct_keys,
        r.counts.postings_sum,
        r.out_keys.len(),
        r.out_keys
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
    );
}
