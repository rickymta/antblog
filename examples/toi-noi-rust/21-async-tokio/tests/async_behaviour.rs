//! Test tất định bằng đồng hồ ảo: `start_paused = true` làm thời gian chỉ nhảy khi mọi task đang chờ timer.

use std::time::Duration;

use ledger_21_async_tokio::blocking::{self, Work};
use ledger_21_async_tokio::flows::*;
use ledger_21_async_tokio::vixanh::{AuthId, PspError, Vixanh};
use ledger_core::{Ledger, Money};
use tokio::time::{Instant, sleep};
use tokio_util::sync::CancellationToken;

const fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

// Authorize 100 ms; lần capture đầu chậm, các lần sau nhanh.
fn slow_first_capture() -> Vixanh {
    Vixanh::builder()
        .authorize_latency("TN-0001", ms(100))
        .slow_captures(1)
        .build()
}

#[tokio::test(start_paused = true)]
async fn future_is_lazy_until_polled() {
    let psp = Vixanh::standard();
    let future = psp.authorize("TN-0001", Money::from_dong(25_500));
    assert_eq!(psp.authorize_calls(), 0, "tạo future chưa chạy gì");
    future.await.expect("authorize thành công");
    assert_eq!(psp.authorize_calls(), 1);
}

#[tokio::test(start_paused = true)]
async fn spawn_starts_without_awaiting() {
    let psp = Vixanh::standard();
    let task = {
        let psp = psp.clone();
        tokio::spawn(async move { psp.authorize("TN-0001", Money::from_dong(25_500)).await })
    };
    assert_eq!(psp.authorize_calls(), 0, "task chưa được lên lịch");
    tokio::task::yield_now().await; // nhường để runtime chạy task vừa spawn
    assert_eq!(
        psp.authorize_calls(),
        1,
        "spawn chạy mà không cần .await handle"
    );
    task.abort();
}

#[tokio::test(start_paused = true)]
async fn join_takes_max_not_sum() {
    let (psp, trips) = demo_trips();
    let (results, sequential) = authorize_sequential(&psp, &trips).await;
    assert!(results.iter().all(Result::is_ok));
    assert_eq!(sequential, ms(120 + 300 + 200));
    let (results, joined) = authorize_join(&psp, &trips).await;
    assert!(results.iter().all(Result::is_ok));
    assert_eq!(joined, ms(300));
}

#[tokio::test(start_paused = true)]
async fn spawned_tasks_also_overlap() {
    let (psp, trips) = demo_trips();
    let (results, elapsed) = authorize_spawned(&psp, &trips).await;
    assert_eq!(results.len(), 3);
    assert_eq!(elapsed, ms(300));
}

#[tokio::test(start_paused = true)]
async fn try_join_fails_fast_and_drops_the_rest() {
    let psp = Vixanh::builder()
        .authorize_latency("TN-0001", ms(120))
        .authorize_failure("TN-0002", ms(150), PspError::Unavailable)
        .authorize_latency("TN-0003", ms(200))
        .build();
    let trips = [
        Trip::new("TN-0001", 5),
        Trip::new("TN-0002", 3),
        Trip::new("TN-0003", 8),
    ];
    let (result, elapsed) = authorize_try_join(&psp, &trips).await;
    assert_eq!(result.unwrap_err(), PspError::Unavailable);
    assert_eq!(elapsed, ms(150), "không chờ TN-0003 (200 ms)");
}

#[tokio::test(start_paused = true)]
async fn timeout_and_select_give_the_same_answer() {
    let (psp, trips) = demo_trips();
    let slow = &trips[1]; // 300 ms
    let by_timeout = authorize_with_timeout(&psp, slow, ms(250)).await;
    let by_select = authorize_with_select(&psp, slow, ms(250)).await;
    assert_eq!(by_timeout, Err(CallError::Timeout(ms(250))));
    assert_eq!(by_select, Err(CallError::Timeout(ms(250))));
    let fast = &trips[0]; // 120 ms
    assert!(authorize_with_timeout(&psp, fast, ms(250)).await.is_ok());
    assert!(authorize_with_select(&psp, fast, ms(250)).await.is_ok());
}

#[tokio::test(start_paused = true)]
async fn timeout_does_not_mean_not_charged() {
    let psp = Vixanh::builder().slow_captures(1).build();
    let trip = Trip::new("TN-0001", 5);
    let auth = AuthId::for_trip(&trip.id);
    // Lần capture đầu chậm: VíXanh trừ tiền ở 100 ms, phản hồi ở 400 ms; ta bỏ chờ ở 250 ms.
    let outcome =
        tokio::time::timeout(ms(250), psp.capture(&auth, &trip.capture_key(), trip.fare)).await;
    assert!(outcome.is_err(), "khách thấy hết hạn");
    assert_eq!(psp.charged_total(), trip.fare, "nhưng VíXanh đã trừ tiền");
}

#[tokio::test(start_paused = true)]
async fn stable_key_retry_charges_once() {
    let psp = slow_first_capture();
    let trip = Trip::new("TN-0001", 5);
    let payment = pay_trip(&psp, &trip, ms(250), 3, KeyMode::Stable)
        .await
        .unwrap();
    assert_eq!(payment.attempts, 2);
    assert_eq!(payment.elapsed, ms(100 + 250 + 50));
    assert!(payment.receipt.replayed, "lần 2 nhận lại biên lai cũ");
    assert_eq!(psp.charge_count(), 1);
    assert_eq!(psp.charged_total(), Money::from_dong(25_500));
}

#[tokio::test(start_paused = true)]
async fn two_slow_captures_third_attempt_replays() {
    let psp = Vixanh::builder()
        .authorize_latency("TN-0001", ms(100))
        .slow_captures(2)
        .build();
    let trip = Trip::new("TN-0001", 5);
    let payment = pay_trip(&psp, &trip, ms(250), 3, KeyMode::Stable)
        .await
        .unwrap();
    assert_eq!(payment.attempts, 3);
    assert!(payment.receipt.replayed);
    assert_eq!(psp.charge_count(), 1);
    assert_eq!(psp.charged_total(), Money::from_dong(25_500));
    // Chỉ cho hai lần thử: hết hạn, nhưng tiền vẫn đã bị trừ.
    let psp = Vixanh::builder()
        .authorize_latency("TN-0001", ms(100))
        .slow_captures(2)
        .build();
    let failed = pay_trip(&psp, &trip, ms(250), 2, KeyMode::Stable).await;
    assert!(matches!(failed, Err(CallError::Timeout(_))));
    sleep(ms(1_000)).await;
    assert_eq!(psp.charged_total(), Money::from_dong(25_500));
}

#[tokio::test(start_paused = true)]
async fn per_attempt_key_charges_twice() {
    let psp = slow_first_capture();
    let trip = Trip::new("TN-0001", 5);
    let payment = pay_trip(&psp, &trip, ms(250), 3, KeyMode::PerAttempt)
        .await
        .unwrap();
    assert_eq!(payment.attempts, 2);
    assert!(!payment.receipt.replayed);
    assert_eq!(psp.charge_count(), 2);
    assert_eq!(
        psp.charged_total(),
        Money::from_dong(51_000),
        "khách bị trừ gấp đôi"
    );
}

#[tokio::test(start_paused = true)]
async fn ledger_replay_after_retry_keeps_one_entry() {
    let psp = slow_first_capture();
    let trip = Trip::new("TN-0001", 5);
    let payment = pay_trip(&psp, &trip, ms(250), 3, KeyMode::Stable)
        .await
        .unwrap();
    assert_eq!(payment.receipt.key, trip.capture_key());
    let mut ledger = Ledger::new();
    let (id, replayed) =
        post_capture(&mut ledger, &trip, "customer:CU-001", "driver:DR-001").unwrap();
    assert_eq!((id.to_string().as_str(), replayed), ("JE-000001", false));
    let (_, replayed) =
        post_capture(&mut ledger, &trip, "customer:CU-001", "driver:DR-001").unwrap();
    assert!(replayed);
    assert_eq!(ledger.len(), 1);
}

#[tokio::test(start_paused = true)]
async fn dropped_future_skips_cleanup_unless_guard() {
    let psp = Vixanh::standard();
    let trip = Trip::new("TN-0001", 5);
    let auth = AuthId::for_trip(&trip.id);

    let naive = InFlight::default();
    let outcome = tokio::time::timeout(ms(10), capture_naive(&psp, &naive, &auth, &trip)).await;
    assert!(outcome.is_err());
    assert!(!naive.is_empty(), "bản ngây thơ: key kẹt vĩnh viễn");

    let guarded = InFlight::default();
    let outcome = tokio::time::timeout(ms(10), capture_guarded(&psp, &guarded, &auth, &trip)).await;
    assert!(outcome.is_err());
    assert!(guarded.is_empty(), "bản có guard: Drop đã dọn");
}

#[tokio::test(start_paused = true)]
async fn token_stops_batch_and_leaves_one_in_doubt() {
    let psp = Vixanh::standard();
    let trips: Vec<Trip> = (1..=4)
        .map(|n| Trip::new(&format!("TN-000{n}"), 5))
        .collect();
    let token = CancellationToken::new();
    {
        let token = token.clone();
        tokio::spawn(async move {
            sleep(ms(130)).await;
            token.cancel();
        });
    }
    let start = Instant::now();
    let report = capture_batch(&psp, &trips, &token).await;
    assert_eq!(start.elapsed(), ms(130));
    assert_eq!(report.completed, vec!["TN-0001", "TN-0002"]);
    assert_eq!(report.interrupted.as_deref(), Some("TN-0003"));
    assert_eq!(report.not_started, vec!["TN-0004"]);
    // VíXanh đã trừ tiền TN-0003 ở 120 ms dù khách không nhận được phản hồi (150 ms).
    sleep(ms(500)).await;
    assert_eq!(psp.charge_count(), 3);
    // Đối soát: gửi lại cùng key, nhận biên lai cũ, không trừ thêm.
    let again = psp
        .capture(
            &AuthId::for_trip("TN-0003"),
            &trips[2].capture_key(),
            trips[2].fare,
        )
        .await
        .unwrap();
    assert!(again.replayed);
    assert_eq!(psp.charge_count(), 3);
}

#[tokio::test]
async fn child_token_follows_parent_only() {
    let parent = CancellationToken::new();
    let child = parent.child_token();
    child.cancel();
    assert!(child.is_cancelled());
    assert!(!parent.is_cancelled(), "hủy con không hủy cha");
    let other_child = parent.child_token();
    parent.cancel();
    assert!(other_child.is_cancelled(), "hủy cha hủy mọi con");
}

#[tokio::test(start_paused = true)]
async fn abort_and_panic_surface_in_join_error() {
    let aborted = tokio::spawn(async {
        sleep(ms(1_000)).await;
    });
    aborted.abort();
    assert!(aborted.await.unwrap_err().is_cancelled());

    let panicked = tokio::spawn(async {
        panic!("VíXanh trả dữ liệu hỏng");
    });
    assert!(panicked.await.unwrap_err().is_panic());
}

// Đồng hồ thật, runtime một worker: số đo thay đổi theo máy. Cận dưới không phụ thuộc tải máy;
// cận trên lấy lần đo tốt nhất vì máy tải có thể làm xấu một lần đo bất kỳ.
#[test]
fn blocking_in_task_stalls_the_runtime_but_spawn_blocking_does_not() {
    let blocked = blocking::measure(Work::BlockInTask);
    let offloaded = blocking::best_gap(Work::SpawnBlocking, ms(250), 5);
    assert!(blocked.max_gap >= ms(250), "bị chặn: {:?}", blocked.max_gap);
    assert!(offloaded < ms(250), "không bị chặn: {offloaded:?}");
}
