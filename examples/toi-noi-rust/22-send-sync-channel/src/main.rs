//! Binary: chạy các demo concurrency, hoặc `--verify`, hoặc `--bench` (đo Mutex với RwLock).

use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use ledger_lab::actor::{self, ActorError, SubmitError};
use ledger_lab::deadlock;
use ledger_lab::fixtures::{tn_0001_entry, trip_entry};
use ledger_lab::readers::{self, Config, Scenario};
use ledger_lab::shared::{self, post_holding_across_await};
use ledger_lab::{AccountId, Ledger, Money};
use tokio::sync::broadcast::error::{RecvError, TryRecvError};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

/// Gom các kiểm tra của `--verify`: in từng dòng lỗi, không dừng ở lỗi đầu tiên.
#[derive(Default)]
struct Checks {
    failures: Vec<String>,
}

impl Checks {
    fn expect(&mut self, ok: bool, what: &str) {
        if !ok {
            self.failures.push(what.to_string());
        }
    }
}

async fn demo(checks: &mut Checks) -> Res<()> {
    println!("== Arc<Mutex<Ledger>>: 50 task ghi cùng key ==");
    let race = shared::race_same_key(50).await?;
    println!(
        "  bút toán trong sổ = {}, ghi mới = {}, trả lại bút toán cũ = {}, mã = {:?}",
        race.entries, race.created, race.replayed, race.ids
    );
    checks.expect(race.entries == 1, "Mutex: đúng 1 bút toán");
    checks.expect(
        race.created == 1 && race.replayed == 49,
        "Mutex: 1 mới, 49 cũ",
    );
    checks.expect(
        race.ids == ["JE-000001"],
        "Mutex: mọi task cùng thấy JE-000001",
    );

    println!("== tokio::sync::Mutex: giữ khóa qua .await ==");
    let tokio_ledger = Arc::new(tokio::sync::Mutex::new(Ledger::new()));
    let mut handles = Vec::new();
    for _ in 0..20 {
        let ledger = Arc::clone(&tokio_ledger);
        let entry = tn_0001_entry()?;
        handles.push(tokio::spawn(async move {
            post_holding_across_await(&ledger, entry).await
        }));
    }
    let mut created = 0;
    for handle in handles {
        if !handle.await??.replayed {
            created += 1;
        }
    }
    let len = tokio_ledger.lock().await.len();
    println!("  20 task, bút toán trong sổ = {len}, ghi mới = {created}");
    checks.expect(len == 1 && created == 1, "tokio Mutex: đúng 1 bút toán");

    println!("== Guard qua .await khi không đòi Send: join! trên runtime một luồng ==");
    // Việc này chặn thread để chờ, nên đưa sang blocking pool thay vì chặn worker.
    let (std_done, tokio_done) = tokio::task::spawn_blocking(|| {
        (
            deadlock::std_mutex_finishes(Duration::from_millis(500)),
            deadlock::tokio_mutex_finishes(Duration::from_secs(5)),
        )
    })
    .await?;
    println!("  std::sync::Mutex: xong sau 500 ms = {std_done}");
    println!("  tokio::sync::Mutex: xong = {tokio_done}");
    checks.expect(!std_done, "std Mutex: kẹt khi giữ guard qua .await");
    checks.expect(tokio_done, "tokio Mutex: hai nhánh đều xong");

    println!("== Actor: một task sở hữu Ledger ==");
    let (handle, rx) = actor::channel(8, 16);
    let owner = actor::spawn(Ledger::new(), rx, &handle);
    let mut subscribers = [handle.subscribe(), handle.subscribe(), handle.subscribe()];
    let mut tasks = Vec::new();
    for _ in 0..50 {
        let handle = handle.clone();
        let entry = tn_0001_entry()?;
        tasks.push(tokio::spawn(async move { handle.post(entry).await }));
    }
    let mut outcomes = Vec::new();
    for task in tasks {
        outcomes.push(task.await?.map_err(|e| format!("{e:?}"))?);
    }
    let created = outcomes.iter().filter(|o| !o.replayed).count();
    println!(
        "  50 task: ghi mới = {created}, trả lại bút toán cũ = {}",
        outcomes.len() - created
    );
    checks.expect(created == 1, "Actor: đúng 1 lần ghi mới");
    let driver = AccountId::new("driver:DR-001")?;
    let balance = handle.balance(driver).await.map_err(|e| format!("{e:?}"))?;
    println!("  số dư driver:DR-001 hỏi qua actor = {balance}");
    checks.expect(
        balance == Money::from_dong(20_400),
        "Actor: số dư tài xế 20.400",
    );
    for (i, sub) in subscribers.iter_mut().enumerate() {
        let event = sub.recv().await?;
        println!(
            "  subscriber {}: LedgerEntryPosted {{ {}, {}, moved = {} }}",
            i + 1,
            event.entry_id,
            event.idempotency_key,
            event.moved
        );
        checks.expect(
            event.entry_id.to_string() == "JE-000001"
                && event.idempotency_key == "trip:TN-0001:capture"
                && event.moved == Money::from_dong(25_500),
            "broadcast: nội dung sự kiện",
        );
        checks.expect(
            sub.try_recv() == Err(TryRecvError::Empty),
            "broadcast: 49 lần ghi lại không phát thêm sự kiện",
        );
    }
    drop(handle);
    let ledger = owner.await?;
    println!(
        "  actor dừng khi hết tay cầm, trả Ledger về: {} bút toán",
        ledger.len()
    );
    checks.expect(ledger.len() == 1, "Actor: trả Ledger 1 bút toán");

    println!("== Backpressure: hàng đợi dung lượng 2, chưa có actor chạy ==");
    let (handle, rx) = actor::channel(2, 16);
    let first = handle.try_post(trip_entry(1, 5)?);
    let second = handle.try_post(trip_entry(2, 3)?);
    let third = handle.try_post(trip_entry(3, 4)?);
    println!(
        "  try_post lần 1 = {}, lần 2 = {}, lần 3 = {:?}",
        if first.is_ok() {
            "vào hàng đợi"
        } else {
            "lỗi"
        },
        if second.is_ok() {
            "vào hàng đợi"
        } else {
            "lỗi"
        },
        third.as_ref().err()
    );
    checks.expect(
        first.is_ok() && second.is_ok() && third.as_ref().err() == Some(&SubmitError::Full),
        "hàng đợi đầy trả Full ở lần thứ 3",
    );
    let waited =
        tokio::time::timeout(Duration::from_millis(50), handle.post(trip_entry(4, 2)?)).await;
    println!(
        "  post().await khi đầy, chờ 50 ms: hết hạn = {}",
        waited.is_err()
    );
    checks.expect(
        waited.is_err(),
        "post().await chờ chỗ trống, không thoát ra",
    );
    let owner = actor::spawn(Ledger::new(), rx, &handle);
    let id1 = first.map_err(|e| format!("{e:?}"))?.await??.id;
    let id2 = second.map_err(|e| format!("{e:?}"))?.await??.id;
    println!("  actor chạy: hai lệnh đã xếp hàng được xử lý, {id1} và {id2}");
    checks.expect(
        id1.to_string() == "JE-000001" && id2.to_string() == "JE-000002",
        "FIFO",
    );
    drop(handle);
    let ledger = owner.await?;
    // Lệnh bị timeout bỏ chờ chưa bao giờ vào hàng đợi, lệnh Full bị từ chối: sổ chỉ có 2.
    println!("  số bút toán trong sổ = {}", ledger.len());
    checks.expect(ledger.len() == 2, "chỉ 2 bút toán vào sổ");

    println!("== broadcast: subscriber chậm bị Lagged ==");
    let (handle, rx) = actor::channel(8, 2);
    let owner = actor::spawn(Ledger::new(), rx, &handle);
    let mut slow = handle.subscribe();
    for n in 1..=5 {
        handle
            .post(trip_entry(n, 5)?)
            .await
            .map_err(|e: ActorError| format!("{e:?}"))?;
    }
    let lagged = slow.recv().await;
    println!("  sau 5 bút toán, kênh chứa 2: recv() = {lagged:?}");
    let next = slow.recv().await?;
    let last = slow.recv().await?;
    println!("  rồi nhận {} và {}", next.entry_id, last.entry_id);
    checks.expect(lagged == Err(RecvError::Lagged(3)), "Lagged(3)");
    checks.expect(
        next.entry_id.to_string() == "JE-000004" && last.entry_id.to_string() == "JE-000005",
        "sau Lagged vẫn nhận 2 sự kiện mới nhất",
    );
    drop(handle);
    owner.await?;
    Ok(())
}

fn bench() -> Res<()> {
    let cfg = Config {
        entries: 2_000,
        tasks: 8,
        ops_per_task: 2_000,
        repeats: 5,
    };
    println!(
        "Đo trên máy viết bài, không phải SLA: {} bút toán, {} worker thread, {} thao tác/task, trung vị {} lần",
        cfg.entries, cfg.tasks, cfg.ops_per_task, cfg.repeats
    );
    println!(
        "{:<34}{:>12}{:>12}{:>8}",
        "kịch bản", "Mutex ms", "RwLock ms", "tỉ lệ"
    );
    for scenario in [
        Scenario::HeavyRead,
        Scenario::HeavyReadWithWrites,
        Scenario::TinyRead,
    ] {
        let t = readers::measure(scenario, cfg)?;
        println!(
            "{:<34}{:>12.1}{:>12.1}{:>7.1}x",
            scenario.name(),
            t.mutex_ms,
            t.rwlock_ms,
            t.mutex_ms / t.rwlock_ms
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    let arg = std::env::args().nth(1);
    if arg.as_deref() == Some("--bench") {
        return match bench() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("lỗi: {e}");
                ExitCode::FAILURE
            }
        };
    }
    let verify = arg.as_deref() == Some("--verify");
    let outcome = tokio::runtime::Runtime::new()
        .map_err(Into::into)
        .and_then(|rt| {
            let mut checks = Checks::default();
            rt.block_on(demo(&mut checks)).map(|()| checks)
        });
    match outcome {
        Ok(checks) if checks.failures.is_empty() => {
            if verify {
                println!("VERIFY OK");
            }
            ExitCode::SUCCESS
        }
        Ok(checks) => {
            for failure in &checks.failures {
                eprintln!("SAI: {failure}");
            }
            if verify {
                eprintln!("VERIFY FAILED");
            }
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("lỗi: {e}");
            ExitCode::FAILURE
        }
    }
}
