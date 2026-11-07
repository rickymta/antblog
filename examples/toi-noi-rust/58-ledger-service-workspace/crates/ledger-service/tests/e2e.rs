//! Test end-to-end với PostgreSQL và Kafka thật. Mỗi test tự dựng schema và topic riêng,
//! nên chạy song song được. Thiếu hạ tầng thì test FAIL rõ, không skip.

use std::time::{Duration, Instant};

use anyhow::Result;
use ledger_service::arch;
use ledger_service::harness::LabEnv;
use ledger_service::scenario;
use rdkafka::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};

#[tokio::test(flavor = "multi_thread")]
async fn trip_completed_to_two_entries_and_balances_then_replay() -> Result<()> {
    scenario::flow_and_replay().await.map(drop)
}

#[tokio::test(flavor = "multi_thread")]
async fn post_entry_over_http_and_grpc_with_jwt() -> Result<()> {
    scenario::http_and_grpc_contract().await.map(drop)
}

#[tokio::test(flavor = "multi_thread")]
async fn hold_capture_expiry_and_refund() -> Result<()> {
    scenario::hold_and_refund().await.map(drop)
}

#[tokio::test(flavor = "multi_thread")]
async fn database_triggers_and_checks_hold_without_the_application() -> Result<()> {
    scenario::database_guards().await.map(drop)
}

#[tokio::test(flavor = "multi_thread")]
async fn balances_lock_in_account_order_not_posting_order() -> Result<()> {
    scenario::balance_lock_order().await.map(drop)
}

#[tokio::test(flavor = "multi_thread")]
async fn sigterm_exits_zero_and_logs_request_id() -> Result<()> {
    scenario::sigterm_child(std::path::Path::new(env!("CARGO_BIN_EXE_ledger-service")))
        .await
        .map(drop)
}

// Không cần PostgreSQL hay Kafka: chỉ cần cargo.
#[test]
fn dependency_direction_holds() -> Result<()> {
    arch::check_all().map(drop)
}

// Dọn đúng: sau `cleanup` không còn schema `lab_<uuid>` và hai topic biến mất.
#[tokio::test(flavor = "multi_thread")]
async fn cleanup_removes_schema_and_topics() -> Result<()> {
    let env = LabEnv::setup().await?;
    let probe = env.clone();
    env.cleanup().await?;

    let mut admin = <sqlx::PgConnection as sqlx::Connection>::connect(&probe.pg_url).await?;
    let schemas: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_namespace WHERE nspname = $1")
        .bind(&probe.schema)
        .fetch_one(&mut admin)
        .await?;
    assert_eq!(schemas, 0, "schema {} còn tồn tại", probe.schema);

    // Xoá topic của Kafka là bất đồng bộ: chờ tới hạn chót thay vì ngủ cố định.
    let consumer: BaseConsumer = ClientConfig::new()
        .set("bootstrap.servers", &probe.kafka)
        .set("group.id", format!("{}-meta", probe.group))
        .create()?;
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let metadata = consumer.fetch_metadata(None, Duration::from_secs(5))?;
        let left = metadata
            .topics()
            .iter()
            .filter(|t| t.name().starts_with(&probe.prefix))
            .count();
        if left == 0 {
            return Ok(());
        }
        assert!(
            Instant::now() < deadline,
            "còn {left} topic của ca sau khi dọn"
        );
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
