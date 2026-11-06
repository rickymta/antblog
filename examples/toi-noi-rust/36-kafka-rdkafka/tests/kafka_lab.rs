//! Test trên Kafka thật (cần `TOI_NOI_KAFKA`). Mỗi test có topic/group riêng nên chạy song song được.

use ledger_36_kafka_rdkafka::checks::{
    check_at_least_once, check_group, check_keys, check_no_rewind, check_poison, check_timeout,
};
use ledger_36_kafka_rdkafka::kafka::Lab;
use ledger_36_kafka_rdkafka::scenarios::{
    at_least_once, group_rebalance, idle_timeout, keys_and_order, no_commit_does_not_rewind,
    poison_messages_scenario,
};

#[tokio::test(flavor = "multi_thread")]
async fn at_least_once_replays_once() -> anyhow::Result<()> {
    let lab = Lab::start().await?;
    let result = at_least_once(&lab).await;
    let cleanup = lab.finish().await;
    check_at_least_once(&result?)?;
    cleanup
}

#[tokio::test(flavor = "multi_thread")]
async fn key_decides_partition_and_order() -> anyhow::Result<()> {
    let lab = Lab::start().await?;
    let result = keys_and_order(&lab).await;
    let cleanup = lab.finish().await;
    check_keys(&result?)?;
    cleanup
}

#[tokio::test(flavor = "multi_thread")]
async fn group_splits_partitions_and_rebalances() -> anyhow::Result<()> {
    let lab = Lab::start().await?;
    let result = group_rebalance(&lab).await;
    let cleanup = lab.finish().await;
    check_group(&result?)?;
    cleanup
}

#[tokio::test(flavor = "multi_thread")]
async fn poison_messages_do_not_stick_the_partition() -> anyhow::Result<()> {
    let lab = Lab::start().await?;
    let result = poison_messages_scenario(&lab).await;
    let cleanup = lab.finish().await;
    check_poison(&result?)?;
    cleanup
}

#[tokio::test(flavor = "multi_thread")]
async fn skipping_a_commit_does_not_rewind_the_consumer() -> anyhow::Result<()> {
    let lab = Lab::start().await?;
    let result = no_commit_does_not_rewind(&lab).await;
    let cleanup = lab.finish().await;
    check_no_rewind(&result?)?;
    cleanup
}

#[tokio::test(flavor = "multi_thread")]
async fn idle_topic_times_out_instead_of_hanging() -> anyhow::Result<()> {
    let lab = Lab::start().await?;
    let result = idle_timeout(&lab).await;
    let cleanup = lab.finish().await;
    check_timeout(&result?)?;
    cleanup
}

#[tokio::test(flavor = "multi_thread")]
async fn missing_or_unreachable_broker_fails_clearly() {
    let missing = Lab::connect(None).await.err().expect("thiếu biến phải lỗi");
    assert!(format!("{missing:#}").contains("TOI_NOI_KAFKA"));
    // Cổng 1 không có broker: phải báo lỗi sau tối đa vài giây, không treo.
    let dead = Lab::connect(Some("127.0.0.1:1"))
        .await
        .err()
        .expect("không có broker phải lỗi");
    assert!(format!("{dead:#}").contains("không kết nối được Kafka"));
}
