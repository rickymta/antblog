//! Test tích hợp trên PostgreSQL thật. Mỗi test có schema `lab_<uuid>` riêng và tự dọn,
//! nên chạy song song và chạy lại nhiều lần đều được. Thiếu `TOI_NOI_PG_URL` thì test FAIL.

use ledger_24_transaction_idempotency::lab::with_lab;
use ledger_24_transaction_idempotency::scenarios::{
    append_only, atomicity, idempotency, limit_race, on_conflict_after_snapshot, on_conflict_waits,
};
use ledger_24_transaction_idempotency::store::{
    Isolation, StoreError, retry_serializable, sqlstate,
};
use sqlx::PgPool;

#[tokio::test]
async fn transaction_commit_failure_and_drop() {
    with_lab(async move |lab| atomicity(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn same_key_posts_once() {
    with_lab(async move |lab| idempotency(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn on_conflict_waits_for_first_commit() {
    with_lab(async move |lab| on_conflict_waits(&lab, true).await?.verify(true))
        .await
        .unwrap();
}

#[tokio::test]
async fn on_conflict_inserts_after_first_rollback() {
    with_lab(async move |lab| on_conflict_waits(&lab, false).await?.verify(false))
        .await
        .unwrap();
}

#[tokio::test]
async fn on_conflict_with_old_snapshot_depends_on_isolation() {
    for isolation in [
        Isolation::ReadCommitted,
        Isolation::RepeatableRead,
        Isolation::Serializable,
    ] {
        with_lab(async move |lab| on_conflict_after_snapshot(&lab, isolation).await?.verify())
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn read_committed_breaks_the_limit() {
    with_lab(async move |lab| {
        limit_race(&lab, Isolation::ReadCommitted, "driver:DR-002")
            .await?
            .verify()
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn repeatable_read_also_breaks_the_limit() {
    with_lab(async move |lab| {
        limit_race(&lab, Isolation::RepeatableRead, "driver:DR-002")
            .await?
            .verify()
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn serializable_keeps_the_limit() {
    with_lab(async move |lab| {
        limit_race(&lab, Isolation::Serializable, "driver:DR-003")
            .await?
            .verify()
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn ledger_is_append_only() {
    with_lab(async move |lab| append_only(&lab).await?.verify())
        .await
        .unwrap();
}

// Lỗi 40001 thật do PostgreSQL phát (RAISE với ERRCODE), không cần dựng xung đột tuần tự hoá.
async fn raise(pool: &PgPool, sqlstate: &str) -> Result<(), StoreError> {
    let sql = format!("DO $$ BEGIN RAISE EXCEPTION 'thử' USING ERRCODE = '{sqlstate}'; END $$");
    sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
        .execute(pool)
        .await?;
    Ok(())
}

#[tokio::test]
async fn retry_stops_after_success() {
    with_lab(async move |lab| {
        let ((), attempts) = retry_serializable(5, |attempt| {
            let pool = lab.pool.clone();
            async move {
                if attempt < 3 {
                    raise(&pool, "40001").await
                } else {
                    Ok(())
                }
            }
        })
        .await?;
        assert_eq!(attempts, 3);
        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn retry_is_bounded() {
    with_lab(async move |lab| {
        let result = retry_serializable(3, |_| {
            let pool = lab.pool.clone();
            async move { raise(&pool, "40001").await }
        })
        .await;
        assert!(
            matches!(result, Err(StoreError::RetriesExhausted { attempts: 3 })),
            "{result:?}"
        );
        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn other_errors_are_not_retried() {
    with_lab(async move |lab| {
        let mut calls = 0;
        let result = retry_serializable(5, |_| {
            calls += 1;
            let pool = lab.pool.clone();
            async move { raise(&pool, "23505").await }
        })
        .await;
        assert_eq!(calls, 1);
        let Err(StoreError::Db(err)) = result else {
            panic!("mong lỗi DB: {result:?}")
        };
        assert_eq!(sqlstate(&err).as_deref(), Some("23505"));
        Ok(())
    })
    .await
    .unwrap();
}
