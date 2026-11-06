//! Mỗi test chạy một thí nghiệm trong schema `lab_<uuid>` riêng nên chạy song song được.

use ledger_38_webhook_hmac::harness::{Harness, key_a};
use ledger_38_webhook_hmac::lab::with_lab;
use ledger_38_webhook_hmac::scenarios::{
    arrival_order, compare_model, key_rotation, psp_retries_on_5xx, replay_and_duplicates,
    signature_on_raw_bytes,
};
use ledger_38_webhook_hmac::signature::KeyRing;

async fn harness(pool: sqlx::PgPool) -> anyhow::Result<Harness> {
    Harness::start(pool, KeyRing::new(vec![key_a()])).await
}

#[tokio::test(flavor = "multi_thread")]
async fn signature_is_checked_on_raw_bytes() -> anyhow::Result<()> {
    with_lab(async move |lab| {
        signature_on_raw_bytes(&harness(lab.pool.clone()).await?)
            .await?
            .verify()
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn replay_is_bounded_and_duplicates_apply_once() -> anyhow::Result<()> {
    with_lab(async move |lab| {
        replay_and_duplicates(&harness(lab.pool.clone()).await?)
            .await?
            .verify()
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn webhook_and_event_meet_in_any_order() -> anyhow::Result<()> {
    with_lab(async move |lab| {
        arrival_order(&harness(lab.pool.clone()).await?)
            .await?
            .verify()
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn two_keys_valid_during_rotation() -> anyhow::Result<()> {
    with_lab(async move |lab| {
        key_rotation(&harness(lab.pool.clone()).await?)
            .await?
            .verify()
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn database_failure_gives_5xx_and_retry_succeeds() -> anyhow::Result<()> {
    with_lab(async move |lab| {
        psp_retries_on_5xx(&harness(lab.pool.clone()).await?)
            .await?
            .verify()
    })
    .await
}

#[test]
fn verify_slice_rejects_forged_tags_that_share_a_prefix() {
    compare_model().verify().unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn oversized_body_is_refused_before_the_handler() -> anyhow::Result<()> {
    with_lab(async move |lab| {
        let h = harness(lab.pool.clone()).await?;
        let huge = vec![b' '; 100 * 1024];
        let r = h.send_signed(&key_a(), &huge).await?;
        anyhow::ensure!(r.status == 413, "mã: {}", r.status);
        Ok(())
    })
    .await
}
