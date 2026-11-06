//! Đọc số liệu task từ console-subscriber mà không cần giao diện TUI của `tokio-console`.
//!
//! Chạy (cần feature `console` và cfg `tokio_unstable`):
//! `RUSTFLAGS="--cfg tokio_unstable" CARGO_TARGET_DIR=target/console \
//!    cargo run --features console --bin console-probe -- --verify`
//!
//! Chương trình tự dựng hai kịch bản trên runtime một luồng (handler chặn và handler chia đoạn),
//! kết nối tới chính server gRPC của console-subscriber bằng client của `console-api`, và in
//! số poll, thời gian bận, thời gian chờ được lên lịch của từng task.

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::process::ExitCode;
use std::sync::mpsc;
use std::time::Duration;

use console_api::field::{Name, Value};
use console_api::instrument::InstrumentRequest;
use console_api::instrument::instrument_client::InstrumentClient;
use ledger_profiling::books::{Book, ScanBook};
use ledger_profiling::runtime::{CHUNK, Mode, run_scenario};
use ledger_profiling::workload::{customer_of, make_workload};

/// Số bút toán trong sổ và số lần quét của mỗi handler.
const ENTRIES: u64 = 4_096;
const ROUNDS: u32 = 300;

/// Số liệu của một task khi nó kết thúc.
#[derive(Debug, Default, Clone)]
struct TaskStats {
    polls: u64,
    busy: Duration,
    scheduled: Duration,
    /// Từ lúc spawn tới lúc được poll lần đầu: task phải chờ runtime rảnh bao lâu.
    first_poll_delay: Duration,
    finished: bool,
}

// Duration của protobuf là (giây, nano) có dấu; số liệu của console không âm.
fn dur(seconds: i64, nanos: i32) -> Duration {
    Duration::new(
        u64::try_from(seconds).unwrap_or(0),
        u32::try_from(nanos).unwrap_or(0),
    )
}

/// Chạy trên một luồng riêng: nối vào server, đọc cập nhật tới khi cả bốn task đã kết thúc.
fn probe(
    port: u16,
    ready: mpsc::Sender<()>,
    wanted: Vec<&'static str>,
) -> HashMap<String, TaskStats> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime của probe");
    rt.block_on(async move {
        let uri = format!("http://127.0.0.1:{port}");
        // Server của console-subscriber khởi động trên luồng khác nên có thể chưa nghe: thử lại tới khi nối được.
        let channel = loop {
            match tonic::transport::Channel::from_shared(uri.clone())
                .expect("uri hợp lệ")
                .connect()
                .await
            {
                Ok(channel) => break channel,
                Err(_) => tokio::time::sleep(Duration::from_millis(20)).await,
            }
        };
        let mut client = InstrumentClient::new(channel);
        let mut updates = client
            .watch_updates(InstrumentRequest {})
            .await
            .expect("watch_updates")
            .into_inner();
        // Báo cho luồng chính: đã đăng ký xong, có thể chạy kịch bản.
        ready.send(()).expect("luồng chính còn sống");

        let mut names: HashMap<u64, String> = HashMap::new();
        let mut stats: HashMap<u64, TaskStats> = HashMap::new();
        while let Ok(Some(update)) = updates.message().await {
            let Some(tasks) = update.task_update else {
                continue;
            };
            for task in tasks.new_tasks {
                let id = task.id.map_or(0, |id| id.id);
                for field in task.fields {
                    // Tên task do `tokio::task::Builder::name` đặt, gửi dưới dạng Debug hoặc string.
                    if let (Some(Name::StrName(key)), Some(Value::DebugVal(v) | Value::StrVal(v))) =
                        (field.name, field.value)
                        && key == "task.name"
                    {
                        names.insert(id, v);
                    }
                }
            }
            for (id, s) in tasks.stats_update {
                let poll = s.poll_stats.unwrap_or_default();
                stats.insert(
                    id,
                    TaskStats {
                        polls: poll.polls,
                        busy: poll
                            .busy_time
                            .map_or(Duration::ZERO, |d| dur(d.seconds, d.nanos)),
                        scheduled: s
                            .scheduled_time
                            .map_or(Duration::ZERO, |d| dur(d.seconds, d.nanos)),
                        first_poll_delay: match (s.created_at, poll.first_poll) {
                            (Some(c), Some(f)) => {
                                dur(f.seconds, f.nanos).saturating_sub(dur(c.seconds, c.nanos))
                            }
                            _ => Duration::ZERO,
                        },
                        finished: s.dropped_at.is_some(),
                    },
                );
            }
            let done = wanted.iter().all(|want| {
                names
                    .iter()
                    .any(|(id, name)| name == want && stats.get(id).is_some_and(|s| s.finished))
            });
            if done {
                break;
            }
        }
        names
            .into_iter()
            .filter_map(|(id, name)| stats.get(&id).cloned().map(|s| (name, s)))
            .collect()
    })
}

fn main() -> ExitCode {
    // Cổng do hệ điều hành cấp (`127.0.0.1:0`), rồi nhả ra cho console-subscriber nghe.
    let port = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .and_then(|l| l.local_addr())
        .map(|a| a.port())
        .expect("xin cổng tạm");
    console_subscriber::ConsoleLayer::builder()
        .server_addr(SocketAddr::from((Ipv4Addr::LOCALHOST, port)))
        .publish_interval(Duration::from_millis(50))
        .init();

    let wanted: Vec<&'static str> = [Mode::Blocking, Mode::Chunked]
        .iter()
        .flat_map(|m| [m.handler_task(), m.heartbeat_task()])
        .collect();
    let (ready_tx, ready_rx) = mpsc::channel();
    let probe_thread = std::thread::spawn({
        let wanted = wanted.clone();
        move || probe(port, ready_tx, wanted)
    });
    ready_rx.recv().expect("probe đã đăng ký");

    // Runtime một luồng: task nào chặn thì mọi task khác đứng chờ.
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("runtime của kịch bản");
    let mut runs = Vec::new();
    for mode in [Mode::Blocking, Mode::Chunked] {
        let mut book = ScanBook::new();
        for entry in make_workload(ENTRIES).entries {
            book.post(entry);
        }
        let run = rt.block_on(run_scenario(mode, book, customer_of(1), ROUNDS));
        runs.push((mode, run));
    }

    let stats = probe_thread.join().expect("probe không panic");
    println!("{ENTRIES} bút toán, {ROUNDS} lần quét mỗi handler, runtime một luồng");
    println!(
        "{:<20} {:>7} {:>12} {:>12} {:>14}",
        "task", "polls", "busy", "scheduled", "chờ poll đầu"
    );
    for name in &wanted {
        match stats.get(*name) {
            Some(s) => println!(
                "{name:<20} {:>7} {:>12.1?} {:>12.1?} {:>14.1?}",
                s.polls, s.busy, s.scheduled, s.first_poll_delay
            ),
            None => println!("{name:<20} (không thấy trong console)"),
        }
    }
    for (mode, run) in &runs {
        println!(
            "{:?}: heartbeat đã đập {} nhịp khi handler xong",
            mode, run.ticks_at_finish
        );
    }

    if std::env::args().any(|a| a == "--verify") {
        let chunks = ENTRIES.div_ceil(CHUNK as u64);
        let polls = |name: &str| stats.get(name).map(|s| s.polls);
        // Console báo số poll cao hơn số lần poll thật đúng một với task đã xong: `Instrumented`
        // (tracing 0.1.44) vào span thêm một lần khi drop future, và `on_enter` của
        // console-subscriber 0.5.0 đếm mỗi lần vào span là một poll.
        let extra = |name: &str| polls(name).map(|p| p.saturating_sub(1));
        // Số poll thật: handler chặn chạy hết trong 1 poll; handler chia đoạn trả quyền
        // `ROUNDS * chunks` lần nên có `ROUNDS * chunks + 1` poll.
        let ok = extra("balance-blocking") == Some(1)
            && extra("balance-chunked") == Some(u64::from(ROUNDS) * chunks + 1)
            && runs[0].1.ticks_at_finish == 0
            && runs[1].1.ticks_at_finish > 0
            && runs[0].1.balance == runs[1].1.balance;
        if !ok {
            eprintln!("LỖI: số poll hoặc số nhịp heartbeat không như mong đợi");
            return ExitCode::FAILURE;
        }
        println!("VERIFY OK");
    }
    ExitCode::SUCCESS
}
