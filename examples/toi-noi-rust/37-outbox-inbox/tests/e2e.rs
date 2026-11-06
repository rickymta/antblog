//! Test tích hợp trên PostgreSQL và Kafka thật. Mỗi test có schema `lab_<uuid>` và topic
//! `lab.37.<uuid>.*` riêng, tự dọn, nên chạy song song và chạy lại đều được.
//! Thiếu `TOI_NOI_PG_URL` hoặc `TOI_NOI_KAFKA` thì test FAIL, không skip.

use std::path::PathBuf;

use ledger_37_outbox_inbox::lab::with_env;
use ledger_37_outbox_inbox::scenarios::{
    dlq, dual_write, inbox_dedupe, inbox_race, ordering, outbox_atomic, parallel_relays,
    redelivery, relay_crash,
};

/// File thực thi của lab: tiến trình con giả lập bị kill chạy lại chính nó.
fn exe() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ledger_37_outbox_inbox"))
}

#[tokio::test]
async fn dual_write_loses_or_invents_events() {
    with_env(exe(), async move |env| dual_write(&env).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn outbox_commits_with_the_entry() {
    with_env(exe(), async move |env| outbox_atomic(&env).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn relay_crash_without_inbox_doubles_balances() {
    with_env(exe(), async move |env| {
        relay_crash(&env, false).await?.verify()
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn relay_crash_with_inbox_applies_once() {
    with_env(exe(), async move |env| {
        relay_crash(&env, true).await?.verify()
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn parallel_relays_without_lock_duplicate() {
    with_env(exe(), async move |env| {
        parallel_relays(&env, false).await?.verify()
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn parallel_relays_skip_locked_do_not_duplicate() {
    with_env(exe(), async move |env| {
        parallel_relays(&env, true).await?.verify()
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn two_workers_reorder_even_within_a_key() {
    with_env(exe(), async move |env| {
        ordering(&env, false).await?.verify()
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn sharding_by_key_keeps_per_key_order() {
    with_env(exe(), async move |env| ordering(&env, true).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn inbox_blocks_duplicate_event_and_business_key() {
    with_env(exe(), async move |env| inbox_dedupe(&env).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn inbox_survives_eight_concurrent_deliveries() {
    with_env(exe(), async move |env| inbox_race(&env).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn uncommitted_offset_redelivers_and_inbox_absorbs() {
    with_env(exe(), async move |env| redelivery(&env).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn poison_messages_go_to_dlq_and_partition_moves_on() {
    with_env(exe(), async move |env| dlq(&env).await?.verify())
        .await
        .unwrap();
}
