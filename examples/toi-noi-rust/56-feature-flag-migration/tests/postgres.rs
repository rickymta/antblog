//! Test tích hợp trên PostgreSQL thật. Mỗi test có schema `lab_<uuid>` riêng và tự dọn.
//! Thiếu `TOI_NOI_PG_URL` thì test FAIL. Không test nào assert thời gian.

use ledger_56_feature_flag_migration::app::{
    AppV1, AppV2, Decision, PostOutcome, WriteMode, balances_match_postings, sample_trip, seed_v1,
    sqlstate,
};
use ledger_56_feature_flag_migration::config::{Config, DEFAULT_JSON};
use ledger_56_feature_flag_migration::lab::with_lab;
use ledger_56_feature_flag_migration::migrate::{
    EVENTS_VERSION, POSTINGS_CURRENCY, backfill_all, backfill_batch, checkpoint, expand,
    migrate_all,
};
use ledger_56_feature_flag_migration::policy::PolicyVersion;
use ledger_56_feature_flag_migration::scenarios::{
    alternatives, compat_matrix, fee_flag, lock_queue, naive_path, rollback, safe_path,
    timeout_retry, typed_config, walkthrough,
};
use sqlx::{Connection, Row};

fn cfg() -> Config {
    Config::from_json(DEFAULT_JSON).unwrap()
}

#[tokio::test]
async fn walkthrough_keeps_old_entries_unchanged() {
    with_lab(async |lab| walkthrough(&lab, &cfg()).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn compatibility_matrix_of_three_code_versions() {
    with_lab(async |lab| compat_matrix(&lab, &cfg()).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn safe_path_does_not_block_writers() {
    with_lab(async |lab| safe_path(&lab, &cfg()).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn constant_default_and_not_null_not_valid_skip_the_scan() {
    with_lab(async |lab| alternatives(&lab, &cfg()).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn naive_not_null_scans_and_blocks() {
    with_lab(async |lab| naive_path(&lab, &cfg()).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn pending_alter_queues_later_readers() {
    with_lab(async |lab| lock_queue(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn lock_timeout_fails_fast_and_retry_succeeds() {
    with_lab(async |lab| timeout_retry(&lab, &cfg()).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn fee_flag_policy_version_and_replay() {
    with_lab(async |lab| fee_flag(&lab, &cfg()).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn rollback_before_and_after_contract() {
    with_lab(async |lab| rollback(&lab, &cfg()).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn typed_config_and_flag_rows_are_validated() {
    with_lab(async |lab| typed_config(&lab).await?.verify())
        .await
        .unwrap();
}

// Cùng một chuyến, cùng quy tắc V1: hai bản code ghi ra đúng cùng các posting.
#[tokio::test]
async fn both_code_versions_post_identical_postings() {
    async fn postings(pool: &sqlx::PgPool) -> Vec<(String, String, i64)> {
        sqlx::query(
            "SELECT e.idempotency_key AS key, p.account_id, p.amount_dong FROM postings p \
             JOIN journal_entries e ON e.id = p.entry_id ORDER BY e.idempotency_key, p.line_no",
        )
        .fetch_all(pool)
        .await
        .unwrap()
        .iter()
        .map(|r| (r.get("key"), r.get("account_id"), r.get("amount_dong")))
        .collect()
    }
    let old = with_lab(async |lab| {
        AppV1::record_trip(&lab.pool, &sample_trip(1)?).await?;
        Ok(postings(&lab.pool).await)
    })
    .await
    .unwrap();
    let new = with_lab(async |lab| {
        migrate_all(&lab.pool, 100, 200).await?;
        AppV2::record_trip(
            &lab.pool,
            &sample_trip(1)?,
            PolicyVersion::V1,
            WriteMode::NewOnly,
            Decision::Frozen,
        )
        .await?;
        Ok(postings(&lab.pool).await)
    })
    .await
    .unwrap();
    assert_eq!(old, new);
    assert_eq!(old.len(), 5);
}

#[tokio::test]
async fn recording_a_trip_twice_returns_the_first_entry() {
    with_lab(async |lab| {
        let trip = sample_trip(1)?;
        let first = AppV1::record_trip(&lab.pool, &trip).await?;
        let second = AppV1::record_trip(&lab.pool, &trip).await?;
        assert!(matches!(first.split, PostOutcome::Posted(_)));
        assert_eq!(second.split, PostOutcome::Replayed(first.split.entry_id()));
        assert!(balances_match_postings(&lab.pool).await?);
        Ok(())
    })
    .await
    .unwrap();
}

// Chạy lại backfill sau khi xong: không cập nhật dòng nào, checkpoint không lùi.
#[tokio::test]
async fn backfill_is_idempotent_and_resumable() {
    with_lab(async |lab| {
        let pool = &lab.pool;
        seed_v1(pool, 30).await?;
        expand(pool, 200).await?;
        let first = backfill_batch(pool, &POSTINGS_CURRENCY, 10).await?.unwrap();
        assert_eq!(first.checkpoint, 10);
        assert_eq!(checkpoint(pool, &POSTINGS_CURRENCY).await?, 10);
        let rest = backfill_all(pool, &POSTINGS_CURRENCY, 10).await?;
        let filled: u64 = first.rows + rest.iter().map(|b| b.rows).sum::<u64>();
        assert_eq!(filled, 150);
        assert!(
            backfill_batch(pool, &POSTINGS_CURRENCY, 10)
                .await?
                .is_none()
        );
        let again = backfill_all(pool, &EVENTS_VERSION, 7).await?;
        assert_eq!(again.iter().map(|b| b.rows).sum::<u64>(), 30);
        assert!(again.windows(2).all(|w| w[0].checkpoint < w[1].checkpoint));
        Ok(())
    })
    .await
    .unwrap();
}

// Cửa sổ nới guard chỉ cho điền NULL -> 'VND': giá trị khác, sửa số tiền hay xoá đều bị từ chối.
#[tokio::test]
async fn relaxed_guard_only_allows_filling_vnd() {
    with_lab(async |lab| {
        let pool = &lab.pool;
        seed_v1(pool, 2).await?;
        expand(pool, 200).await?;
        for sql in [
            "UPDATE postings SET currency = 'USD' WHERE entry_id = 1 AND line_no = 1",
            "UPDATE postings SET currency = 'VND', amount_dong = amount_dong + 1 \
             WHERE entry_id = 1 AND line_no = 1",
            "DELETE FROM postings WHERE entry_id = 1 AND line_no = 1",
        ] {
            let err = sqlx::query(sql).execute(pool).await.unwrap_err();
            assert_eq!(sqlstate(&err).as_deref(), Some("TN001"), "{sql}");
        }
        let filled =
            sqlx::query("UPDATE postings SET currency = 'VND' WHERE entry_id = 1 AND line_no = 1")
                .execute(pool)
                .await?;
        assert_eq!(filled.rows_affected(), 1);
        let err =
            sqlx::query("UPDATE postings SET currency = 'USD' WHERE entry_id = 1 AND line_no = 1")
                .execute(pool)
                .await
                .unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some("TN001"));
        Ok(())
    })
    .await
    .unwrap();
}

// Hai tiến trình backfill cùng tên: một phiên giữ dòng checkpoint cho tới khi cả hai đã xếp hàng
// (đọc từ pg_stat_activity, không đoán bằng sleep), rồi nhả. Không lô nào được dùng lại checkpoint.
#[tokio::test]
async fn parallel_backfills_queue_on_the_checkpoint_row() {
    with_lab(async |lab| {
        let pool = lab.pool.clone();
        seed_v1(&pool, 30).await?;
        expand(&pool, 200).await?;
        let first = backfill_batch(&pool, &POSTINGS_CURRENCY, 5).await?.unwrap();

        let mut gate = lab.connect("gate-checkpoint").await?;
        let gate_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut gate)
            .await?;
        let mut hold = gate.begin().await?;
        sqlx::query("SELECT 1 FROM migration_checkpoints WHERE name = $1 FOR UPDATE")
            .bind(POSTINGS_CURRENCY.name)
            .fetch_one(&mut *hold)
            .await?;

        let spawn_worker = || {
            let pool = pool.clone();
            tokio::spawn(async move { backfill_all(&pool, &POSTINGS_CURRENCY, 5).await })
        };
        let (a, b) = (spawn_worker(), spawn_worker());

        // Phiên sau chờ tuple lock của phiên trước, nên phải đi theo chuỗi chặn, không chỉ gate.
        let mut observer = lab.connect("observer-checkpoint").await?;
        let mut queued = 0_i64;
        for _ in 0..12_000 {
            queued = sqlx::query_scalar(
                "WITH RECURSIVE blocked AS ( \
                   SELECT pid FROM pg_stat_activity \
                    WHERE wait_event_type = 'Lock' AND $1 = ANY(pg_blocking_pids(pid)) \
                   UNION \
                   SELECT a.pid FROM pg_stat_activity a \
                     JOIN blocked ON blocked.pid = ANY(pg_blocking_pids(a.pid)) \
                    WHERE a.wait_event_type = 'Lock') \
                 SELECT count(*) FROM blocked",
            )
            .bind(gate_pid)
            .fetch_one(&mut observer)
            .await?;
            if queued == 2 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert_eq!(
            queued, 2,
            "cả hai tiến trình phải xếp hàng trên dòng checkpoint"
        );
        hold.commit().await?;

        let batches_a = a.await??;
        let batches_b = b.await??;
        let mut seen: Vec<i64> = batches_a
            .iter()
            .chain(&batches_b)
            .map(|batch| batch.checkpoint)
            .collect();
        let rows: u64 = first.rows
            + batches_a
                .iter()
                .chain(&batches_b)
                .map(|batch| batch.rows)
                .sum::<u64>();
        seen.sort_unstable();
        let total = seen.len();
        seen.dedup();
        assert_eq!(seen.len(), total, "hai lô không được dùng chung checkpoint");
        assert_eq!(rows, 150, "mỗi dòng được điền đúng một lần");
        let nulls: i64 = sqlx::query_scalar("SELECT count(*) FROM postings WHERE currency IS NULL")
            .fetch_one(&pool)
            .await?;
        assert_eq!(nulls, 0);
        assert_eq!(
            checkpoint(&pool, &POSTINGS_CURRENCY).await?,
            *seen.last().unwrap()
        );
        Ok(())
    })
    .await
    .unwrap();
}
