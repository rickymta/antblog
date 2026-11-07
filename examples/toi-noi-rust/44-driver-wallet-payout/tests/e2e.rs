//! Test tích hợp trên PostgreSQL thật. Mỗi test có schema `lab_<uuid>` riêng, tự dọn, nên chạy
//! song song và chạy lại đều được. Thiếu `TOI_NOI_PG_URL` thì test FAIL, không skip.

use ledger_44_driver_wallet_payout::lab::with_pg;
use ledger_44_driver_wallet_payout::scenarios::{
    ParallelMode, earnings, idempotency, lost_before_receipt, parallel_batches, payout_flow,
    reconcile_anomalies, timeout_retry, withdrawal_limit,
};

#[tokio::test]
async fn earnings_split_by_local_day_not_utc_day() {
    with_pg(async move |lab| earnings(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn batch_goes_through_every_status_and_failures_are_reversed() {
    with_pg(async move |lab| payout_flow(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn daily_limit_resets_at_local_midnight_with_injected_clock() {
    with_pg(async move |lab| withdrawal_limit(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn parallel_batches_without_lock_and_unique_double_pay() {
    with_pg(async move |lab| {
        parallel_batches(&lab, ParallelMode::NaiveNoUnique)
            .await?
            .verify()
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn parallel_batches_unique_per_period_rejects_the_second() {
    with_pg(async move |lab| {
        parallel_batches(&lab, ParallelMode::NaiveUnique)
            .await?
            .verify()
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn parallel_batches_skip_locked_split_the_drivers() {
    with_pg(async move |lab| {
        parallel_batches(&lab, ParallelMode::SkipLocked)
            .await?
            .verify()
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn bank_timeout_after_receipt_looks_up_and_never_pays_again() {
    with_pg(async move |lab| timeout_retry(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn order_lost_before_bank_is_resent_with_the_same_reference() {
    with_pg(async move |lab| lost_before_receipt(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn reconcile_classifies_every_kind_of_mismatch() {
    with_pg(async move |lab| reconcile_anomalies(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn batch_code_and_driver_key_make_everything_idempotent() {
    with_pg(async move |lab| idempotency(&lab).await?.verify())
        .await
        .unwrap();
}
