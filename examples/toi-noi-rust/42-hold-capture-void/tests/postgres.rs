//! Test tích hợp trên PostgreSQL thật. Mỗi test có schema `lab_<uuid>` riêng và tự dọn.
//! Thiếu `TOI_NOI_PG_URL` thì test FAIL. Không test nào assert thời gian chạy; thời gian
//! nghiệp vụ đi qua `ManualClock`.

use ledger_42_hold_capture_void::lab::with_lab;
use ledger_42_hold_capture_void::scenarios::{
    capture_full, capture_partial, capture_vs_void, combined_statement, expiry_skip_locked,
    hold_does_not_post, late_capture, lock_wait, race, void_on_cancel, wallet_without_balance_row,
};
use ledger_42_hold_capture_void::store::LockMode;

#[tokio::test]
async fn hold_is_not_a_posting() {
    with_lab(async move |lab| hold_does_not_post(&lab).await)
        .await
        .unwrap();
}

#[tokio::test]
async fn capture_full_and_idempotent() {
    with_lab(async move |lab| capture_full(&lab).await)
        .await
        .unwrap();
}

#[tokio::test]
async fn capture_partial_releases_the_rest() {
    with_lab(async move |lab| capture_partial(&lab).await)
        .await
        .unwrap();
}

#[tokio::test]
async fn void_releases_everything() {
    with_lab(async move |lab| void_on_cancel(&lab).await)
        .await
        .unwrap();
}

#[tokio::test]
async fn without_lock_both_holds_pass_and_available_goes_negative() {
    let r = with_lab(async move |lab| race(&lab, LockMode::None).await)
        .await
        .unwrap();
    assert_eq!((r.created, r.rejected), (2, 0));
    assert_eq!(r.funds.available.dong(), -11_000);
}

#[tokio::test]
async fn with_for_update_exactly_one_hold_passes() {
    let r = with_lab(async move |lab| race(&lab, LockMode::ForUpdate).await)
        .await
        .unwrap();
    assert_eq!((r.created, r.rejected), (1, 1));
    assert_eq!(r.funds.available.dong(), 14_500);
}

#[tokio::test]
async fn second_hold_waits_on_the_lock() {
    with_lab(async move |lab| lock_wait(&lab).await)
        .await
        .unwrap();
}

#[tokio::test]
async fn lock_and_sum_in_one_statement_reads_the_old_snapshot() {
    with_lab(async move |lab| combined_statement(&lab).await)
        .await
        .unwrap();
}

#[tokio::test]
async fn wallet_without_balance_row_is_rejected_not_assumed_zero() {
    with_lab(async move |lab| wallet_without_balance_row(&lab).await)
        .await
        .unwrap();
}

#[tokio::test]
async fn capture_and_void_race_has_one_winner() {
    with_lab(async move |lab| capture_vs_void(&lab).await)
        .await
        .unwrap();
}

#[tokio::test]
async fn expiry_job_skips_locked_holds() {
    with_lab(async move |lab| expiry_skip_locked(&lab).await)
        .await
        .unwrap();
}

#[tokio::test]
async fn late_capture_is_rejected_before_and_after_the_job() {
    with_lab(async move |lab| late_capture(&lab).await)
        .await
        .unwrap();
}
