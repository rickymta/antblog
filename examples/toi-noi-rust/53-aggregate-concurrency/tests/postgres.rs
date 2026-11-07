//! Test tích hợp trên PostgreSQL thật. Mỗi test có schema `lab_<uuid>` riêng và tự dọn.
//! Thiếu `TOI_NOI_PG_URL` thì test FAIL. Không test nào assert thời gian chạy; cuộc đua dựng bằng
//! `Barrier`/`Notify` và kiểm bất biến, không bằng `sleep`.

use ledger_53_aggregate_concurrency::lab::with_lab;
use ledger_53_aggregate_concurrency::scenarios::{
    deadlock, deadlock_victim, dong, join_lock_trap, journal_is_append_only, lock_variants,
    ordered_stress, race, stale_write,
};
use ledger_53_aggregate_concurrency::store::{LockOrder, LockedLoad, Mode};

#[tokio::test]
async fn unguarded_read_modify_write_loses_the_other_writers_deposit() {
    let r = with_lab(async move |lab| stale_write(&lab, Mode::Unguarded).await)
        .await
        .unwrap();
    assert_eq!(r.attempts, 1);
    assert_eq!(r.stored_balance, dong(129_600));
    assert_eq!(r.ledger_balance, dong(179_600));
    assert_eq!(r.audit.total_balance, -50_000);
    assert_eq!(r.audit.projection_mismatch, ["driver:DR-001"]);
}

#[tokio::test]
async fn optimistic_stale_write_is_a_conflict_then_a_correct_retry() {
    let r = with_lab(async move |lab| stale_write(&lab, Mode::Optimistic).await)
        .await
        .unwrap();
    assert_eq!(r.attempts, 2);
    assert!(r.withdrew);
    assert_eq!(r.stored_balance, dong(179_600));
    assert_eq!(r.ledger_balance, dong(179_600));
    assert_eq!(r.audit.total_balance, 0);
    assert!(r.audit.projection_mismatch.is_empty());
}

#[tokio::test]
async fn unguarded_race_exceeds_the_limit() {
    let r = with_lab(async move |lab| race(&lab, Mode::Unguarded, 8).await)
        .await
        .unwrap();
    assert_eq!((r.ok, r.rejected), (8, 0));
    assert_eq!(
        r.audit.over_limit,
        [("driver:DR-001".to_string(), 163_200, 100_000)]
    );
    assert_ne!(r.audit.total_balance, 0);
    assert_eq!(r.audit.projection_mismatch, ["driver:DR-001"]);
}

#[tokio::test]
async fn optimistic_race_keeps_every_invariant() {
    let r = with_lab(async move |lab| race(&lab, Mode::Optimistic, 8).await)
        .await
        .unwrap();
    assert_eq!((r.ok, r.rejected, r.exhausted), (4, 4, 0));
    assert_eq!(r.withdrawn, dong(81_600));
    assert_eq!(r.balance, dong(68_400));
    assert_eq!(r.version, 5);
    assert_eq!(r.attempts.iter().filter(|a| **a == 1).count(), 1);
    // Mỗi xung đột cần một commit của người khác và chỉ có 4 commit.
    assert!(r.attempts.iter().all(|a| *a <= 5));
    assert!(r.audit.over_limit.is_empty() && r.audit.projection_mismatch.is_empty());
    assert_eq!((r.audit.total_balance, r.audit.total_postings), (0, 0));
}

#[tokio::test]
async fn for_update_race_keeps_every_invariant_without_retries() {
    let r = with_lab(async move |lab| race(&lab, Mode::ForUpdate, 8).await)
        .await
        .unwrap();
    assert_eq!((r.ok, r.rejected), (4, 4));
    assert_eq!(r.attempts, [1; 8]);
    assert_eq!(r.withdrawn, dong(81_600));
    assert!(r.audit.over_limit.is_empty() && r.audit.projection_mismatch.is_empty());
    assert_eq!(r.audit.total_balance, 0);
}

#[tokio::test]
async fn join_with_for_update_of_one_table_reads_a_stale_limits_row() {
    let t = with_lab(async move |lab| join_lock_trap(&lab, LockedLoad::JoinLockOne).await)
        .await
        .unwrap();
    assert_eq!(t.balance, dong(129_600));
    assert_eq!(t.withdrawn, dong(0), "hàng account_limits là bản cũ");
}

#[tokio::test]
async fn locking_in_its_own_statement_or_both_tables_reads_fresh_rows() {
    for style in [LockedLoad::Separate, LockedLoad::JoinLockBoth] {
        let t = with_lab(async move |lab| join_lock_trap(&lab, style).await)
            .await
            .unwrap();
        assert_eq!((t.balance, t.withdrawn), (dong(129_600), dong(20_400)));
    }
}

#[tokio::test]
async fn opposite_transfers_in_given_order_deadlock_with_40p01() {
    let r = with_lab(async move |lab| deadlock(&lab, LockOrder::AsGiven, false).await)
        .await
        .unwrap();
    assert_eq!((r.ok, r.deadlocks), (1, 1));
    assert_eq!(r.audit.total_balance, 0);
    assert!(r.audit.projection_mismatch.is_empty());
}

#[tokio::test]
async fn victim_locks_are_released_before_it_sends_rollback() {
    let v = with_lab(async move |lab| deadlock_victim(&lab).await)
        .await
        .unwrap();
    assert_eq!(v.survivors, 1);
    assert_eq!(v.error.as_deref(), Some("40P01"));
    assert_eq!(v.next_statement.as_deref(), Some("25P02"));
}

#[tokio::test]
async fn retrying_40p01_lets_the_victim_finish() {
    let r = with_lab(async move |lab| deadlock(&lab, LockOrder::AsGiven, true).await)
        .await
        .unwrap();
    assert_eq!((r.ok, r.deadlocks), (2, 0));
    assert!(r.attempts.iter().sum::<u32>() >= 3);
    assert_eq!(r.balances, (dong(48_000), dong(52_000)));
}

#[tokio::test]
async fn locking_by_id_never_deadlocks() {
    let r = with_lab(async move |lab| deadlock(&lab, LockOrder::ById, false).await)
        .await
        .unwrap();
    assert_eq!((r.ok, r.deadlocks, r.attempts), (2, 0, vec![1, 1]));
    let (r, total) = with_lab(async move |lab| ordered_stress(&lab).await)
        .await
        .unwrap();
    assert_eq!((r.ok, r.deadlocks), (12, 0));
    assert_eq!(r.attempts, [1; 12]);
    assert_eq!(total, dong(300_000));
    assert_eq!(r.audit.total_balance, 0);
}

#[tokio::test]
async fn skip_locked_and_nowait_see_the_lock_but_plain_reads_do_not() {
    let c = with_lab(async move |lab| lock_variants(&lab).await)
        .await
        .unwrap();
    assert_eq!(c.plain, ["customer:CU-001", "customer:CU-002"]);
    assert_eq!(c.skip_locked, ["customer:CU-002"]);
    assert_eq!(c.nowait_sqlstate.as_deref(), Some("55P03"));
}

#[tokio::test]
async fn journal_entries_and_postings_cannot_be_updated() {
    let (entry, posting) = with_lab(async move |lab| journal_is_append_only(&lab).await)
        .await
        .unwrap();
    assert_eq!(entry.as_deref(), Some("TN001"));
    assert_eq!(posting.as_deref(), Some("TN001"));
}
