//! Lab bài 21: gọi VíXanh giả bằng async/await trên tokio.
//!
//! `cargo run` chạy kịch bản bằng đồng hồ THẬT (số mili giây thay đổi vài ms mỗi lần).
//! `cargo run -- --verify` kiểm các giá trị kỳ vọng trong bài bằng đồng hồ ẢO (tất định),
//! riêng phần chặn runtime vẫn đo bằng đồng hồ thật với ngưỡng rộng.

use std::process::ExitCode;
use std::time::Duration;

use ledger_21_async_tokio::blocking::{self, Work};
use ledger_21_async_tokio::flows::{
    BatchReport, CallError, InFlight, KeyMode, Trip, authorize_join, authorize_sequential,
    authorize_spawned, authorize_try_join, authorize_with_select, authorize_with_timeout,
    capture_batch, capture_guarded, capture_naive, demo_trips, pay_trip, post_capture,
};
use ledger_21_async_tokio::vixanh::{AuthId, PspError, Vixanh};
use ledger_core::{Ledger, Money};
use tokio::runtime::{Builder, Runtime};
use tokio::time::{Instant, sleep};
use tokio_util::sync::CancellationToken;

const fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

const CUSTOMER: &str = "customer:CU-001";
const DRIVER: &str = "driver:DR-001";

fn main() -> ExitCode {
    if std::env::args().any(|arg| arg == "--verify") {
        verify()
    } else {
        demo();
        ExitCode::SUCCESS
    }
}

// Authorize 100 ms; lần capture đầu chậm (trừ tiền ở 100 ms, phản hồi ở 400 ms), sau đó nhanh.
fn slow_first_capture() -> Vixanh {
    Vixanh::builder()
        .authorize_latency("TN-0001", ms(100))
        .slow_captures(1)
        .build()
}

// Chuyến TN-0001 và bốn chuyến cho lô capture.
fn batch_trips() -> Vec<Trip> {
    (1..=4)
        .map(|n| Trip::new(&format!("TN-000{n}"), 5))
        .collect()
}

fn demo() {
    let runtime = Runtime::new().expect("dựng được runtime");
    runtime.block_on(demo_async());

    println!("\n== CancellationToken dừng lô 4 chuyến ở 130 ms (đồng hồ ảo) ==");
    let (report, _, charged) = run_paused(batch_scenario());
    println!("đã xong     : {:?}", report.completed);
    println!("bị hủy giữa : {:?}", report.interrupted);
    println!("chưa gửi    : {:?}", report.not_started);
    println!("VíXanh đã trừ {charged} giao dịch");

    println!("\n== Chặn runtime (1 worker thread, việc nặng 300 ms) ==");
    for (label, work) in [
        ("std::thread::sleep trong task", Work::BlockInTask),
        ("spawn_blocking", Work::SpawnBlocking),
    ] {
        let stall = blocking::measure(work);
        println!(
            "{label:<32} tick chậm nhất {:>3} ms ({} tick)",
            stall.max_gap.as_millis(),
            stall.ticks
        );
    }
}

async fn demo_async() {
    println!("== Future lười ==");
    let psp = Vixanh::standard();
    let future = psp.authorize("TN-0001", Money::from_dong(25_500));
    println!("tạo future, authorize_calls = {}", psp.authorize_calls());
    let auth = future.await.expect("authorize");
    println!(
        "sau .await, authorize_calls = {} ({})",
        psp.authorize_calls(),
        auth.as_str()
    );

    println!("\n== Tuần tự và join! (3 chuyến: 120, 300, 200 ms) ==");
    let (psp, trips) = demo_trips();
    let (_, sequential) = authorize_sequential(&psp, &trips).await;
    let (_, joined) = authorize_join(&psp, &trips).await;
    let (_, spawned) = authorize_spawned(&psp, &trips).await;
    println!("tuần tự : {:>3} ms", sequential.as_millis());
    println!("join!   : {:>3} ms", joined.as_millis());
    println!("spawn   : {:>3} ms", spawned.as_millis());

    println!("\n== Timeout 250 ms với authorize 300 ms ==");
    let slow = &trips[1];
    println!(
        "timeout(): {:?}",
        authorize_with_timeout(&psp, slow, ms(250)).await
    );
    println!(
        "select!  : {:?}",
        authorize_with_select(&psp, slow, ms(250)).await
    );

    println!("\n== Hết hạn không có nghĩa là chưa trừ tiền ==");
    for (label, mode) in [
        ("key cố định  ", KeyMode::Stable),
        ("key mỗi lần  ", KeyMode::PerAttempt),
    ] {
        let psp = slow_first_capture();
        let trip = Trip::new("TN-0001", 5);
        let payment = pay_trip(&psp, &trip, ms(250), 3, mode)
            .await
            .expect("thanh toán");
        println!(
            "{label}: {} lần thử, replayed = {}, VíXanh đã trừ {} ({} giao dịch)",
            payment.attempts,
            payment.receipt.replayed,
            psp.charged_total(),
            psp.charge_count()
        );
    }
    let psp = slow_first_capture();
    let trip = Trip::new("TN-0001", 5);
    pay_trip(&psp, &trip, ms(250), 3, KeyMode::Stable)
        .await
        .expect("thanh toán");
    let mut ledger = Ledger::new();
    let (id, replayed) = post_capture(&mut ledger, &trip, CUSTOMER, DRIVER).expect("ghi sổ");
    println!("sổ cái: {id} {} replayed = {replayed}", trip.capture_key());

    println!("\n== Hủy giữa chừng: dọn dẹp nằm ở Drop ==");
    let psp = Vixanh::standard();
    let auth = AuthId::for_trip("TN-0001");
    let naive = InFlight::default();
    let _ = tokio::time::timeout(ms(10), capture_naive(&psp, &naive, &auth, &trip)).await;
    let guarded = InFlight::default();
    let _ = tokio::time::timeout(ms(10), capture_guarded(&psp, &guarded, &auth, &trip)).await;
    println!("bản ngây thơ: key còn kẹt = {}", !naive.is_empty());
    println!("bản có guard: key còn kẹt = {}", !guarded.is_empty());
}

// Lô 4 chuyến, `CancellationToken` bị hủy ở 130 ms. Trả báo cáo, thời gian chạy và số giao dịch
// VíXanh đã trừ (sau khi chờ đủ lâu để mọi task phía VíXanh xong).
async fn batch_scenario() -> (BatchReport, Duration, usize) {
    let psp = Vixanh::standard();
    let token = CancellationToken::new();
    let canceller = token.clone();
    tokio::spawn(async move {
        sleep(ms(130)).await;
        canceller.cancel();
    });
    let start = Instant::now();
    let report = capture_batch(&psp, &batch_trips(), &token).await;
    let elapsed = start.elapsed();
    sleep(ms(500)).await;
    (report, elapsed, psp.charge_count())
}

// Chạy một future trên runtime đồng hồ ảo: thời gian nhảy thẳng tới timer kế tiếp.
fn run_paused<F: Future>(future: F) -> F::Output {
    Builder::new_current_thread()
        .enable_time()
        .start_paused(true)
        .build()
        .expect("dựng được runtime")
        .block_on(future)
}

// ---- --verify ----

struct Checks {
    failures: Vec<String>,
}

impl Checks {
    fn check(&mut self, ok: bool, name: &str) {
        println!("{} {name}", if ok { "ok  " } else { "FAIL" });
        if !ok {
            self.failures.push(name.to_string());
        }
    }
}

fn verify() -> ExitCode {
    // Đồng hồ ảo: thời gian nhảy thẳng tới timer kế tiếp khi mọi task đang chờ.
    let runtime = Builder::new_current_thread()
        .enable_time()
        .start_paused(true)
        .build()
        .expect("dựng được runtime");
    let mut checks = Checks {
        failures: Vec::new(),
    };
    runtime.block_on(verify_async(&mut checks));
    let (report, elapsed, charged) = run_paused(batch_scenario());
    checks.check(
        report.completed == ["TN-0001", "TN-0002"],
        "hai chuyến đầu đã xong",
    );
    checks.check(
        report.interrupted.as_deref() == Some("TN-0003"),
        "TN-0003 bị hủy giữa chừng",
    );
    checks.check(report.not_started == ["TN-0004"], "TN-0004 chưa gửi");
    checks.check(elapsed == ms(130), "lô dừng đúng lúc hủy (130 ms)");
    checks.check(
        charged == 3,
        "VíXanh đã trừ cả TN-0003 dù khách không nhận phản hồi",
    );

    // Phần này cần đồng hồ thật: chặn runtime là chuyện của thread thật.
    // Một lần đo là đủ cho cận dưới: không tick nào chen vào được trong lúc worker bị giữ 300 ms.
    let blocked = blocking::measure(Work::BlockInTask);
    let offloaded = blocking::best_gap(Work::SpawnBlocking, ms(250), 5);
    checks.check(
        blocked.max_gap >= ms(250),
        "std::thread::sleep chặn runtime (tick chậm >= 250 ms)",
    );
    checks.check(
        offloaded < ms(250),
        "spawn_blocking không chặn runtime (tick chậm < 250 ms, lần đo tốt nhất)",
    );

    if checks.failures.is_empty() {
        println!("VERIFY OK");
        ExitCode::SUCCESS
    } else {
        eprintln!("VERIFY FAILED: {}", checks.failures.join("; "));
        ExitCode::FAILURE
    }
}

async fn verify_async(c: &mut Checks) {
    let dong = Money::from_dong;

    // Future lười.
    let psp = Vixanh::standard();
    let future = psp.authorize("TN-0001", dong(25_500));
    c.check(psp.authorize_calls() == 0, "tạo future chưa chạy authorize");
    future.await.expect("authorize");
    c.check(
        psp.authorize_calls() == 1,
        "sau .await authorize đã chạy một lần",
    );

    // join! nhanh hơn tuần tự.
    let (psp, trips) = demo_trips();
    let (_, sequential) = authorize_sequential(&psp, &trips).await;
    let (_, joined) = authorize_join(&psp, &trips).await;
    let (results, spawned) = authorize_spawned(&psp, &trips).await;
    c.check(trips[0].fare == dong(25_500), "cước TN-0001 = 25.500");
    c.check(sequential == ms(620), "tuần tự = 120 + 300 + 200 = 620 ms");
    c.check(joined == ms(300), "join! = max = 300 ms");
    c.check(
        spawned == ms(300) && results.len() == 3,
        "spawn + JoinHandle = 300 ms",
    );

    // try_join! fail-fast.
    let failing = Vixanh::builder()
        .authorize_latency("TN-0001", ms(120))
        .authorize_failure("TN-0002", ms(150), PspError::Unavailable)
        .authorize_latency("TN-0003", ms(200))
        .build();
    let (result, elapsed) = authorize_try_join(&failing, &trips).await;
    c.check(
        result == Err(PspError::Unavailable) && elapsed == ms(150),
        "try_join! dừng ở 150 ms, bỏ TN-0003",
    );

    // timeout và select!.
    let slow = &trips[1];
    c.check(
        authorize_with_timeout(&psp, slow, ms(250)).await == Err(CallError::Timeout(ms(250))),
        "timeout(250 ms) với authorize 300 ms -> Timeout",
    );
    c.check(
        authorize_with_select(&psp, slow, ms(250)).await == Err(CallError::Timeout(ms(250))),
        "select! cho cùng kết quả",
    );

    // Hết hạn không có nghĩa là chưa trừ tiền; idempotency key.
    let trip = Trip::new("TN-0001", 5);
    let psp = slow_first_capture();
    let auth = AuthId::for_trip("TN-0001");
    let outcome =
        tokio::time::timeout(ms(250), psp.capture(&auth, &trip.capture_key(), trip.fare)).await;
    c.check(outcome.is_err(), "capture hết hạn ở 250 ms");
    c.check(
        psp.charged_total() == dong(25_500),
        "VíXanh đã trừ 25.500 dù khách thấy hết hạn",
    );

    let psp = slow_first_capture();
    let payment = pay_trip(&psp, &trip, ms(250), 3, KeyMode::Stable)
        .await
        .expect("thanh toán");
    c.check(
        payment.attempts == 2 && payment.receipt.replayed,
        "key cố định: lần 2 là replay",
    );
    c.check(
        psp.charged_total() == dong(25_500) && psp.charge_count() == 1,
        "key cố định: trừ một lần 25.500",
    );
    let psp = slow_first_capture();
    pay_trip(&psp, &trip, ms(250), 3, KeyMode::PerAttempt)
        .await
        .expect("thanh toán");
    c.check(
        psp.charged_total() == dong(51_000) && psp.charge_count() == 2,
        "key mỗi lần: trừ hai lần 51.000",
    );

    let mut ledger = Ledger::new();
    let (id, replayed) = post_capture(&mut ledger, &trip, CUSTOMER, DRIVER).expect("ghi sổ");
    c.check(
        id.to_string() == "JE-000001" && !replayed,
        "bút toán JE-000001",
    );
    let (_, replayed) = post_capture(&mut ledger, &trip, CUSTOMER, DRIVER).expect("ghi lại");
    c.check(
        replayed && ledger.len() == 1,
        "ghi lại cùng key là replay, sổ vẫn 1 bút toán",
    );

    // Hủy giữa chừng.
    let psp = Vixanh::standard();
    let naive = InFlight::default();
    let _ = tokio::time::timeout(ms(10), capture_naive(&psp, &naive, &auth, &trip)).await;
    let guarded = InFlight::default();
    let _ = tokio::time::timeout(ms(10), capture_guarded(&psp, &guarded, &auth, &trip)).await;
    c.check(!naive.is_empty(), "bản ngây thơ để kẹt key sau khi bị hủy");
    c.check(guarded.is_empty(), "bản có guard dọn key khi bị hủy");
}
