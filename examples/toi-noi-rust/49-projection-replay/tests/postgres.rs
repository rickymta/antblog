//! Test tích hợp trên PostgreSQL thật. Mỗi test có schema `lab_<uuid>` riêng và tự dọn.
//! Thiếu `TOI_NOI_PG_URL` thì test FAIL. Không test nào assert thời gian.

use ledger_49_projection_replay::fixtures::{STANDARD, append_steps, entry_of};
use ledger_49_projection_replay::lab::with_lab;
use ledger_49_projection_replay::log::{ReadMode, append_entry, horizon};
use ledger_49_projection_replay::projection::{
    Balances, Role, cutover, drain, install, run_batch, start_rebuild, wait_horizon_past_head,
};
use ledger_49_projection_replay::scenarios::{
    RaceVariant, basic, lag, log_is_empty, race, rebuild, replay,
};
use sqlx::Row;

#[tokio::test]
async fn two_projections_match_the_log() {
    with_lab(async |lab| basic(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn replay_after_crash_does_not_double_count() {
    replay().await.unwrap().verify().unwrap();
}

#[tokio::test]
async fn naive_reader_loses_late_commit() {
    with_lab(async |lab| {
        race(&lab, RaceVariant::LateCommit, ReadMode::NaiveId)
            .await?
            .verify()
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn horizon_holds_back_late_commit() {
    for mode in [ReadMode::HorizonId, ReadMode::Safe] {
        with_lab(async move |lab| race(&lab, RaceVariant::LateCommit, mode).await?.verify())
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn early_xid_breaks_id_cursor_but_not_tx_id_cursor() {
    for mode in [ReadMode::NaiveId, ReadMode::HorizonId, ReadMode::Safe] {
        with_lab(async move |lab| race(&lab, RaceVariant::EarlyXid, mode).await?.verify())
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn rebuild_and_cutover_swap_tables_atomically() {
    with_lab(async |lab| rebuild(&lab).await?.verify())
        .await
        .unwrap();
}

#[tokio::test]
async fn lag_drops_by_one_batch_and_rebuild_matches() {
    // Log nhỏ hơn bài viết để test nhanh; kiểm cấu trúc chứ không kiểm số tuyệt đối.
    with_lab(async |lab| {
        let r = lag(&lab, 200, 100).await?;
        anyhow::ensure!(r.entries == 400, "số bút toán");
        anyhow::ensure!(
            r.behind == vec![400, 300, 200, 100, 0],
            "lag {:?}",
            r.behind
        );
        anyhow::ensure!(r.rebuild_diff == 0, "dựng lại lệch");
        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn empty_log_is_a_noop() {
    with_lab(async |lab| {
        let p = Balances { version: 1 };
        install(&lab.pool, &p).await?;
        anyhow::ensure!(log_is_empty(&lab.pool).await?, "log phải rỗng");
        wait_horizon_past_head(&lab.pool).await?;
        anyhow::ensure!(
            run_batch(&lab.pool, &p, Role::Live, ReadMode::Safe, 10).await? == 0,
            "log rỗng thì không áp dụng gì"
        );
        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn tx_id_is_assigned_per_transaction_and_below_horizon_when_done() {
    with_lab(async |lab| {
        let mut conn = lab.pool.acquire().await?;
        let mut tx = lab.pool.begin().await?;
        let xid: i64 = sqlx::query("SELECT pg_current_xact_id()::text::bigint AS x")
            .fetch_one(&mut *tx)
            .await?
            .get("x");
        // Khi giao dịch còn mở, khoảng an toàn không thể vượt xid của nó.
        anyhow::ensure!(horizon(&mut conn).await? <= xid, "horizon vượt xid đang mở");
        tx.commit().await?;
        // Sau khi xong, khoảng an toàn tiến qua xid đó (chờ giao dịch lạ nếu có).
        for _ in 0..3000 {
            if horizon(&mut conn).await? > xid {
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        anyhow::bail!("horizon không tiến qua {xid}")
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn read_only_transaction_without_xid_does_not_hold_the_horizon() {
    with_lab(async |lab| {
        // Giao dịch chỉ đọc, giữ snapshot mà chưa có xid: mở suốt ca.
        let mut reader = lab.pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            .execute(&mut *reader)
            .await?;
        sqlx::query("SELECT count(*) FROM accounts")
            .execute(&mut *reader)
            .await?;
        let assigned: Option<String> =
            sqlx::query_scalar("SELECT pg_current_xact_id_if_assigned()::text")
                .fetch_one(&mut *reader)
                .await?;
        anyhow::ensure!(assigned.is_none(), "giao dịch chỉ đọc chưa được có xid");

        let mut writer = lab.pool.begin().await?;
        let xid: i64 = sqlx::query_scalar("SELECT pg_current_xact_id()::text::bigint")
            .fetch_one(&mut *writer)
            .await?;
        writer.commit().await?;

        // Khoảng an toàn vẫn tiến qua xid đó dù reader còn mở (chờ giao dịch lạ nếu có).
        let mut conn = lab.pool.acquire().await?;
        for _ in 0..3000 {
            if horizon(&mut conn).await? > xid {
                reader.rollback().await?;
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        anyhow::bail!("horizon bị giữ lại ở dưới {xid}")
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn tx_id_inside_a_savepoint_is_the_top_level_xid() {
    with_lab(async |lab| {
        let mut tx = lab.pool.begin().await?;
        sqlx::query("SAVEPOINT s1").execute(&mut *tx).await?;
        let entry = entry_of(&STANDARD[0])?;
        let id = append_entry(&mut tx, &entry, STANDARD[0].posted_at, None).await?;
        sqlx::query("RELEASE SAVEPOINT s1")
            .execute(&mut *tx)
            .await?;
        let row = sqlx::query(
            "SELECT tx_id, pg_current_xact_id()::text::bigint AS top FROM journal_entries WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
        anyhow::ensure!(
            row.get::<i64, _>("tx_id") == row.get::<i64, _>("top"),
            "tx_id phải là xid của transaction cha"
        );
        tx.rollback().await?;
        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn repeatable_read_reader_sees_the_new_table_through_its_old_snapshot() {
    with_lab(async |lab| {
        let pool = &lab.pool;
        let (b1, b2) = (Balances { version: 1 }, Balances { version: 2 });
        install(pool, &b1).await?;
        append_steps(pool, &STANDARD).await?;
        drain(pool, &b1, Role::Live, ReadMode::Safe, 100).await?;

        // Snapshot chụp trước khi có bảng mới, và chưa đụng tới bảng `balances`.
        let mut reader = pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            .execute(&mut *reader)
            .await?;
        sqlx::query("SELECT count(*) FROM journal_entries")
            .execute(&mut *reader)
            .await?;

        start_rebuild(pool, &b2).await?;
        drain(pool, &b2, Role::Build, ReadMode::Safe, 100).await?;
        cutover(pool, &b1, &b2, true).await?;

        // Tên `balances` giờ là bảng mới; dòng của nó commit sau snapshot nên reader không thấy.
        let seen: i64 = sqlx::query_scalar("SELECT count(*) FROM balances")
            .fetch_one(&mut *reader)
            .await?;
        reader.rollback().await?;
        let fresh: i64 = sqlx::query_scalar("SELECT count(*) FROM balances")
            .fetch_one(pool)
            .await?;
        anyhow::ensure!(seen == 0, "reader cũ thấy {seen} dòng");
        anyhow::ensure!(fresh == 7, "reader mới thấy {fresh} dòng");
        Ok(())
    })
    .await
    .unwrap();
}
