//! Test end-to-end trên PostgreSQL và Kafka thật. Mỗi test dựng schema `lab_<uuid>`, topic
//! `lab.54.<uuid>.*` và consumer group riêng nên chạy song song được. Thiếu `TOI_NOI_PG_URL` hoặc
//! `TOI_NOI_KAFKA` thì test FAIL rõ, không bỏ qua. Không test nào assert thời gian: chờ theo
//! điều kiện có hạn chót.

use std::path::{Path, PathBuf};

use anyhow::Result;
use api::scenarios::{self, Order};
use infrastructure::Fault;

fn exe() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_trip-payment"))
}

#[tokio::test(flavor = "multi_thread")]
async fn trip_first_settles_through_kafka() -> Result<()> {
    scenarios::flow(Order::TripFirst).await.map(drop)
}

#[tokio::test(flavor = "multi_thread")]
async fn capture_first_settles_through_kafka() -> Result<()> {
    scenarios::flow(Order::CaptureFirst).await.map(drop)
}

// Chạy cả hai thứ tự rồi so từng bút toán, từng số dư, từng LedgerEntryPosted.
#[tokio::test(flavor = "multi_thread")]
async fn both_arrival_orders_give_identical_books() -> Result<()> {
    scenarios::both_orders().await.map(drop)
}

#[tokio::test(flavor = "multi_thread")]
async fn losing_the_version_race_still_settles_exactly_once() -> Result<()> {
    scenarios::version_race().await.map(drop)
}

#[tokio::test(flavor = "multi_thread")]
async fn late_promo_budget_limit_and_amount_mismatch() -> Result<()> {
    scenarios::promo_and_mismatch().await.map(drop)
}

#[tokio::test(flavor = "multi_thread")]
async fn three_line_ledger_rejects_two_step_capture_with_same_key() -> Result<()> {
    scenarios::legacy_three_line().await.map(drop)
}

#[tokio::test(flavor = "multi_thread")]
async fn partial_refund_follows_the_cumulative_split() -> Result<()> {
    scenarios::partial_refund().await.map(drop)
}

#[tokio::test(flavor = "multi_thread")]
async fn duplicate_and_repriced_events_do_not_double_post() -> Result<()> {
    scenarios::duplicates().await.map(drop)
}

#[tokio::test(flavor = "multi_thread")]
async fn process_killed_before_commit_leaves_no_trace_and_recovers() -> Result<()> {
    scenarios::crash(Fault::AbortBeforeCommit, &exe())
        .await
        .map(drop)
}

#[tokio::test(flavor = "multi_thread")]
async fn process_killed_after_commit_is_deduplicated_on_redelivery() -> Result<()> {
    scenarios::crash(Fault::AbortAfterCommit, &exe())
        .await
        .map(drop)
}

#[tokio::test(flavor = "multi_thread")]
async fn reconciliation_matches_clearing_lines_and_flags_tampering() -> Result<()> {
    scenarios::reconcile_clearing().await.map(drop)
}

// ---------- kiến trúc: chiều phụ thuộc và schema chuẩn (không cần hạ tầng) ----------

// Tên các dependency trong mục `[dependencies]`; hiểu cả dạng `domain.workspace = true`.
fn dependencies_of(manifest: &str) -> Vec<String> {
    let mut in_deps = false;
    let mut names = Vec::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_deps = line == "[dependencies]";
        } else if in_deps && let Some((key, _)) = line.split_once('=') {
            let key = key.trim();
            names.push(key.split('.').next().unwrap_or(key).to_string());
        }
    }
    names
}

fn read(path: &str) -> String {
    let full = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    std::fs::read_to_string(&full)
        .unwrap_or_else(|e| panic!("không đọc được {}: {e}", full.display()))
}

#[test]
fn dependency_direction_is_domain_then_application_then_infrastructure() {
    let domain = dependencies_of(&read("../domain/Cargo.toml"));
    let application = dependencies_of(&read("../application/Cargo.toml"));
    for name in &domain {
        assert!(
            ![
                "tokio",
                "sqlx",
                "rdkafka",
                "tokio-util",
                "application",
                "infrastructure"
            ]
            .contains(&name.as_str()),
            "domain không được phụ thuộc {name}"
        );
    }
    for name in &application {
        assert!(
            !["tokio", "sqlx", "rdkafka", "tokio-util", "infrastructure"].contains(&name.as_str()),
            "application không được phụ thuộc {name}"
        );
    }
    assert!(application.contains(&"domain".to_string()));
}

// Trigger cân bút toán phải là đúng bản của lab 25, không phải bản chép tay đã lệch.
#[test]
fn balance_trigger_is_byte_identical_to_lab_25() {
    let ours = read("../infrastructure/sql/02_balance_check.sql");
    let lab25 = read("../../../25-schema-so-cai/sql/02_balance_check.sql");
    assert_eq!(ours, lab25);
}
