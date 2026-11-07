//! Mỗi ca diễn tập là một test, chạy trong schema/topic/group riêng, cần PostgreSQL và Kafka của lab.
//! Thiếu `TOI_NOI_PG_URL` hoặc `TOI_NOI_KAFKA` thì test FAIL rõ, không skip.

use ledger_67_operations_runbook::drills;
use ledger_67_operations_runbook::lab::Env;

#[tokio::test(flavor = "multi_thread")]
async fn relay_down_shows_stale_outbox_and_burn_rate() {
    Env::run(|env| async move { drills::relay_down(&env).await })
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn poison_message_goes_to_dlq_and_safe_replay_is_idempotent() {
    Env::run(|env| async move { drills::poison_and_replay(&env).await })
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn missing_entry_is_fixed_by_a_new_referenced_entry() {
    Env::run(|env| async move { drills::reconciliation_mismatch(&env).await })
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn closed_period_is_adjusted_in_the_next_period() {
    Env::run(|env| async move { drills::closed_period(&env).await })
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn nonzero_ledger_total_pages_and_nothing_self_heals() {
    Env::run(|env| async move { drills::ledger_total_nonzero(&env).await })
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn pg_dump_restore_matches_balances_and_hash_chain() {
    Env::run(|env| async move { drills::backup_restore(&env).await })
        .await
        .unwrap();
}
