//! Test concurrency của lab 22. Dùng runtime nhiều luồng để các task thật sự chạy song song.

use std::time::Duration;

use ledger_lab::actor::{self, ActorError, SubmitError};
use ledger_lab::deadlock;
use ledger_lab::fixtures::{tn_0001_entry, trip_entry};
use ledger_lab::readers::{self, Config, Scenario};
use ledger_lab::shared::{self, SharedError};
use ledger_lab::{Ledger, LedgerError, Money, Posting};
use tokio::sync::broadcast::error::{RecvError, TryRecvError};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn mutex_many_tasks_same_key_one_entry() {
    let race = shared::race_same_key(100).await.unwrap();
    assert_eq!(race.entries, 1);
    assert_eq!(race.created, 1);
    assert_eq!(race.replayed, 99);
    assert_eq!(race.ids, ["JE-000001"]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn mutex_different_keys_all_land_and_ledger_sums_to_zero() {
    let shared = shared::new_shared();
    let mut tasks = Vec::new();
    for n in 1..=40 {
        let shared = std::sync::Arc::clone(&shared);
        let entry = trip_entry(n, 5).unwrap();
        tasks.push(tokio::spawn(async move {
            shared::post_then_notify(&shared, entry).await
        }));
    }
    for task in tasks {
        assert!(!task.await.unwrap().unwrap().replayed);
    }
    let ledger = shared.lock().unwrap();
    assert_eq!(ledger.len(), 40);
    assert_eq!(ledger.grand_total().unwrap(), Money::ZERO);
}

#[tokio::test]
async fn same_key_different_content_is_a_ledger_error_not_a_second_entry() {
    let shared = shared::new_shared();
    shared::post_shared(&shared, tn_0001_entry().unwrap()).unwrap();
    let other = ledger_lab::JournalEntry::try_new(
        "trip:TN-0001:capture",
        vec![
            Posting::new(
                ledger_lab::AccountId::new("customer:CU-001").unwrap(),
                Money::from_dong(-100),
            ),
            Posting::new(
                ledger_lab::AccountId::new("driver:DR-001").unwrap(),
                Money::from_dong(100),
            ),
        ],
    )
    .unwrap();
    let err = shared::post_shared(&shared, other).unwrap_err();
    assert!(matches!(
        err,
        SharedError::Ledger(LedgerError::IdempotencyConflict { .. })
    ));
    assert_eq!(shared.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn poisoned_mutex_is_reported_not_unwrapped() {
    let shared = shared::new_shared();
    let clone = std::sync::Arc::clone(&shared);
    let _ = std::thread::spawn(move || {
        let _guard = clone.lock().unwrap();
        panic!("cố ý panic khi giữ khóa");
    })
    .join();
    let err = shared::post_shared(&shared, tn_0001_entry().unwrap()).unwrap_err();
    assert_eq!(err, SharedError::Poisoned);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn actor_many_tasks_same_key_one_entry_one_event() {
    let (handle, rx) = actor::channel(8, 16);
    let owner = actor::spawn(Ledger::new(), rx, &handle);
    let mut sub = handle.subscribe();
    let mut tasks = Vec::new();
    for _ in 0..100 {
        let handle = handle.clone();
        let entry = tn_0001_entry().unwrap();
        tasks.push(tokio::spawn(async move { handle.post(entry).await }));
    }
    let mut created = 0;
    for task in tasks {
        if !task.await.unwrap().unwrap().replayed {
            created += 1;
        }
    }
    assert_eq!(created, 1);
    let event = sub.recv().await.unwrap();
    assert_eq!(event.entry_id.to_string(), "JE-000001");
    assert_eq!(event.moved, Money::from_dong(25_500));
    assert_eq!(sub.try_recv(), Err(TryRecvError::Empty));
    drop(handle);
    assert_eq!(owner.await.unwrap().len(), 1);
}

#[tokio::test]
async fn full_queue_rejects_try_post_and_blocks_post() {
    let (handle, rx) = actor::channel(1, 4);
    let first = handle.try_post(trip_entry(1, 5).unwrap()).unwrap();
    assert_eq!(
        handle.try_post(trip_entry(2, 5).unwrap()).unwrap_err(),
        SubmitError::Full
    );
    let waiting = tokio::time::timeout(
        Duration::from_millis(30),
        handle.post(trip_entry(3, 5).unwrap()),
    )
    .await;
    assert!(waiting.is_err(), "post phải chờ chỗ trống");
    // Có actor thì hàng đợi chạy tiếp và chỗ trống xuất hiện.
    let owner = actor::spawn(Ledger::new(), rx, &handle);
    assert_eq!(first.await.unwrap().unwrap().id.to_string(), "JE-000001");
    let next = handle.post(trip_entry(4, 5).unwrap()).await.unwrap();
    assert_eq!(next.id.to_string(), "JE-000002");
    drop(handle);
    assert_eq!(owner.await.unwrap().len(), 2);
}

#[tokio::test]
async fn slow_subscriber_gets_lagged_then_latest_events() {
    let (handle, rx) = actor::channel(8, 2);
    let owner = actor::spawn(Ledger::new(), rx, &handle);
    let mut slow = handle.subscribe();
    for n in 1..=5 {
        handle.post(trip_entry(n, 5).unwrap()).await.unwrap();
    }
    assert_eq!(slow.recv().await, Err(RecvError::Lagged(3)));
    assert_eq!(slow.recv().await.unwrap().entry_id.to_string(), "JE-000004");
    assert_eq!(slow.recv().await.unwrap().entry_id.to_string(), "JE-000005");
    drop(handle);
    owner.await.unwrap();
}

#[tokio::test]
async fn stopped_actor_reports_closed() {
    let (handle, rx) = actor::channel(2, 2);
    let owner = actor::spawn(Ledger::new(), rx, &handle);
    owner.abort();
    let _ = owner.await;
    assert_eq!(
        handle.post(tn_0001_entry().unwrap()).await,
        Err(ActorError::Closed)
    );
    assert_eq!(
        handle.try_post(tn_0001_entry().unwrap()).unwrap_err(),
        SubmitError::Closed
    );
}

#[tokio::test]
async fn actor_returns_ledger_after_last_handle_drops() {
    let (handle, rx) = actor::channel(2, 2);
    let owner = actor::spawn(Ledger::new(), rx, &handle);
    handle.post(tn_0001_entry().unwrap()).await.unwrap();
    drop(handle);
    let ledger = owner.await.unwrap();
    assert_eq!(ledger.len(), 1);
    assert_eq!(ledger.grand_total().unwrap(), Money::ZERO);
}

#[test]
fn std_guard_across_await_sticks_on_one_thread_but_tokio_mutex_finishes() {
    assert!(!deadlock::std_mutex_finishes(Duration::from_millis(300)));
    assert!(deadlock::tokio_mutex_finishes(Duration::from_secs(5)));
}

#[test]
fn bench_harness_runs_all_scenarios() {
    let cfg = Config {
        entries: 20,
        tasks: 2,
        ops_per_task: 20,
        repeats: 1,
    };
    for scenario in [
        Scenario::HeavyRead,
        Scenario::HeavyReadWithWrites,
        Scenario::TinyRead,
    ] {
        let t = readers::measure(scenario, cfg).unwrap();
        assert!(t.mutex_ms > 0.0 && t.rwlock_ms > 0.0);
    }
}
