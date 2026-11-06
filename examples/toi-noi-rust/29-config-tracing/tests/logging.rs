//! Test log bằng subscriber ghi vào bộ đệm: đọc lại từng dòng JSON và kiểm trường.

use ledger_29_config_tracing::scenarios::{
    CARD, PHONE, audit, capture_flow, enter_guard_bug, instrument_fix,
};
use ledger_29_config_tracing::service::SpanPropagation;
use ledger_29_config_tracing::telemetry::{LogBuffer, json_subscriber};
use serde_json::Value;
use tracing::info;
use tracing::instrument::WithSubscriber;

fn request_id(event: &Value) -> Option<&str> {
    event["spans"]
        .as_array()?
        .iter()
        .find_map(|span| span["request_id"].as_str())
}

fn message(event: &Value) -> &str {
    event["fields"]["message"].as_str().unwrap_or("")
}

#[test]
fn every_event_of_a_request_carries_its_request_id() {
    let events = capture_flow("info", SpanPropagation::Attached).events();
    assert_eq!(events.len(), 8);
    assert!(events.iter().all(|e| request_id(e).is_some()));
    let recorded: Vec<&str> = events
        .iter()
        .filter(|e| message(e) == "đã ghi bút toán")
        .filter_map(request_id)
        .collect();
    assert_eq!(recorded, ["req-0001", "req-0002"]);
}

#[test]
fn child_span_adds_idempotency_key_without_losing_request_id() {
    let events = capture_flow("info", SpanPropagation::Attached).events();
    let posted = events
        .iter()
        .find(|e| message(e) == "đã ghi bút toán")
        .unwrap();
    assert_eq!(posted["fields"]["entry_id"], "JE-000001");
    assert_eq!(posted["span"]["name"], "post_entry");
    assert_eq!(posted["span"]["idempotency_key"], "trip:TN-0001:capture");
    assert_eq!(request_id(posted), Some("req-0001"));
}

#[test]
fn replay_and_conflict_are_logged() {
    let events = capture_flow("info", SpanPropagation::Attached).events();
    let posted: Vec<&Value> = events
        .iter()
        .filter(|e| message(e) == "đã ghi bút toán")
        .collect();
    assert_eq!(posted[0]["fields"]["replayed"], false);
    assert_eq!(posted[1]["fields"]["replayed"], true);
    let error = events.iter().find(|e| e["level"] == "ERROR").unwrap();
    assert_eq!(request_id(error), Some("req-0003"));
}

#[test]
fn forgetting_instrument_on_spawned_task_loses_request_id() {
    let events = capture_flow("info", SpanPropagation::Forgotten).events();
    let notified: Vec<&Value> = events
        .iter()
        .filter(|e| message(e).contains("thông báo"))
        .collect();
    assert_eq!(notified.len(), 2);
    assert!(notified.iter().all(|e| request_id(e).is_none()));
}

#[test]
fn span_enter_guard_across_await_misattributes_other_work() {
    let buggy = enter_guard_bug().events();
    let other = buggy
        .iter()
        .find(|e| message(e) == "việc không liên quan")
        .unwrap();
    assert_eq!(
        request_id(other),
        Some("req-A"),
        "bị gán nhầm cho request khác"
    );

    let fixed = instrument_fix().events();
    let other = fixed
        .iter()
        .find(|e| message(e) == "việc không liên quan")
        .unwrap();
    assert_eq!(request_id(other), None);
    let resumed = fixed
        .iter()
        .find(|e| message(e) == "tiếp tục sau await")
        .unwrap();
    assert_eq!(request_id(resumed), Some("req-A"));
}

#[test]
fn env_filter_decides_which_levels_are_emitted() {
    assert!(
        !capture_flow("info", SpanPropagation::Attached)
            .text()
            .contains("\"DEBUG\"")
    );
    assert!(
        capture_flow("debug", SpanPropagation::Attached)
            .text()
            .contains("\"DEBUG\"")
    );
    // Lọc theo module: chỉ bật debug cho crate này, phần còn lại ở warn.
    let scoped = capture_flow(
        "warn,ledger_29_config_tracing=debug",
        SpanPropagation::Attached,
    );
    assert!(scoped.text().contains("\"DEBUG\""));
}

#[test]
fn bare_instrument_leaks_pii_and_skip_does_not() {
    let leaky = audit(true).text();
    assert!(leaky.contains(PHONE) && leaky.contains(CARD));
    let safe = audit(false).text();
    assert!(!safe.contains(PHONE) && !safe.contains(CARD));
}

#[test]
fn ledger_flow_logs_no_pii_and_no_amounts() {
    let text = capture_flow("debug", SpanPropagation::Attached).text_without_timestamps();
    for forbidden in [
        PHONE, CARD, "25500", "20400", "5100", "25.500", "20.400", "balance",
    ] {
        assert!(!text.contains(forbidden), "log chứa {forbidden}");
    }
}

fn multi_thread_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .build()
        .expect("dựng runtime")
}

/// `set_default` chỉ có hiệu lực trên luồng gọi; task `spawn` chạy trên worker nên event mất.
#[test]
fn set_default_does_not_reach_worker_threads() {
    let buffer = LogBuffer::default();
    let subscriber = json_subscriber("info", buffer.clone()).unwrap();
    let _guard = tracing::subscriber::set_default(subscriber);
    multi_thread_runtime().block_on(async {
        info!("trên luồng gọi");
        tokio::spawn(async { info!("trên worker") }).await.unwrap();
    });
    let text = buffer.text();
    assert!(text.contains("trên luồng gọi"));
    assert!(!text.contains("trên worker"));
}

/// Gắn subscriber vào future thì nó đi theo task, chạy ở luồng nào cũng thấy.
#[test]
fn with_subscriber_follows_the_task_to_worker_threads() {
    let buffer = LogBuffer::default();
    let subscriber = json_subscriber("info", buffer.clone()).unwrap();
    multi_thread_runtime().block_on(
        async {
            let task = async { info!("trên worker") }.with_current_subscriber();
            tokio::spawn(task).await.unwrap();
        }
        .with_subscriber(subscriber),
    );
    assert!(buffer.text().contains("trên worker"));
}
