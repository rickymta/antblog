//! Test với PostgreSQL thật. Mỗi test dựng schema `lab_<uuid>` riêng và dọn đúng schema đó,
//! nên chạy song song được. Thiếu `TOI_NOI_PG_URL` thì test FAIL, không bỏ qua.

use anyhow::Result;
use ledger_25_schema_so_cai::db::{pg_error, with_lab};
use ledger_25_schema_so_cai::ledger::{self, display_id};
use ledger_25_schema_so_cai::partition;
use ledger_core::{AccountId, JournalEntryBuilder, Money};

fn entry(key: &str, amount: i64) -> Result<ledger_core::JournalEntry> {
    Ok(JournalEntryBuilder::new(key)
        .posting(
            AccountId::new("customer:CU-001")?,
            Money::from_dong(-amount),
        )
        .posting(AccountId::new("driver:DR-001")?, Money::from_dong(amount))
        .build()?)
}

#[test]
fn display_id_pads_to_six_digits() {
    assert_eq!(display_id(1), "JE-000001");
    assert_eq!(display_id(123_456), "JE-123456");
}

#[tokio::test]
async fn unbalanced_entry_is_rejected_at_commit_and_leaves_nothing() -> Result<()> {
    with_lab(|db| async move {
        db.migrate_base().await?;
        ledger::seed_accounts(&db.pool).await?;
        let err = ledger::post_unbalanced(&db.pool).await.unwrap_err();
        let (code, message) = pg_error(&err).expect("lỗi từ server");
        assert_eq!(code, "23514");
        assert!(message.contains("lệch: tổng posting = -10"), "{message}");
        let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM postings")
            .fetch_one(&db.pool)
            .await?;
        assert_eq!(rows, 0);
        assert!(ledger::reconcile(&db.pool).await?.is_empty());
        Ok(())
    })
    .await
}

#[tokio::test]
async fn balanced_entry_updates_projection_in_same_transaction() -> Result<()> {
    with_lab(|db| async move {
        db.migrate_base().await?;
        ledger::seed_accounts(&db.pool).await?;
        let id = ledger::post_entry(&db.pool, &entry("t:1", 5_000)?).await?;
        assert_eq!(display_id(id), "JE-000001");
        assert_eq!(
            ledger::projected_balance(&db.pool, "driver:DR-001").await?,
            5_000
        );
        assert_eq!(
            ledger::projected_balance(&db.pool, "customer:CU-001").await?,
            -5_000
        );
        // Cùng idempotency_key lần hai: unique chặn, và số dư không đổi (transaction bị huỷ).
        assert!(
            ledger::post_entry(&db.pool, &entry("t:1", 5_000)?)
                .await
                .is_err()
        );
        assert_eq!(
            ledger::projected_balance(&db.pool, "driver:DR-001").await?,
            5_000
        );
        Ok(())
    })
    .await
}

#[tokio::test]
async fn reconcile_detects_and_rebuild_repairs_drift() -> Result<()> {
    with_lab(|db| async move {
        db.migrate_base().await?;
        ledger::seed_accounts(&db.pool).await?;
        ledger::post_entry(&db.pool, &entry("t:1", 7_000)?).await?;
        sqlx::query(
            "UPDATE account_balances SET balance_dong = 1 WHERE account_id = 'driver:DR-001'",
        )
        .execute(&db.pool)
        .await?;
        let drift = ledger::reconcile(&db.pool).await?;
        assert_eq!(drift.len(), 1);
        assert_eq!((drift[0].projected, drift[0].actual), (1, 7_000));
        ledger::rebuild_balances(&db.pool).await?;
        assert!(ledger::reconcile(&db.pool).await?.is_empty());
        Ok(())
    })
    .await
}

#[tokio::test]
async fn concurrent_posts_keep_projection_equal_to_postings() -> Result<()> {
    with_lab(|db| async move {
        db.migrate_base().await?;
        ledger::seed_accounts(&db.pool).await?;
        let mut tasks = tokio::task::JoinSet::new();
        for i in 0..30_i64 {
            let pool = db.pool.clone();
            tasks.spawn(async move {
                ledger::post_entry(&pool, &entry(&format!("c:{i}"), 100 * (i + 1))?)
                    .await
                    .map_err(anyhow::Error::from)
            });
        }
        while let Some(done) = tasks.join_next().await {
            done??;
        }
        assert!(ledger::reconcile(&db.pool).await?.is_empty());
        let total: i64 =
            sqlx::query_scalar("SELECT sum(balance_dong)::bigint FROM account_balances")
                .fetch_one(&db.pool)
                .await?;
        assert_eq!(total, 0);
        Ok(())
    })
    .await
}

#[tokio::test]
async fn partition_requires_partition_key_in_primary_key() -> Result<()> {
    with_lab(|db| async move {
        db.migrate_base().await?;
        let rejected = partition::pk_without_partition_key(&db.pool).await?;
        assert_eq!(rejected.map(|(code, _)| code).as_deref(), Some("0A000"));
        Ok(())
    })
    .await
}

#[tokio::test]
async fn trigger_checks_every_entry_a_change_touches() -> Result<()> {
    with_lab(|db| async move {
        db.migrate_base().await?;
        ledger::seed_accounts(&db.pool).await?;
        let mut tx = db.pool.begin().await?;
        sqlx::raw_sql(
            "INSERT INTO journal_entries (idempotency_key) VALUES ('a'), ('b');
             INSERT INTO postings (entry_id, line_no, account_id, amount_dong) VALUES
                 (1, 1, 'customer:CU-001', -100), (1, 2, 'driver:DR-001', 60),
                 (1, 3, 'platform:revenue', 40),
                 (2, 1, 'customer:CU-001', -50), (2, 2, 'driver:DR-001', 50);",
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        // Chuyển posting sang bút toán khác: bút toán nhận vẫn cân, bút toán mất dòng phải bị bắt.
        let mut tx = db.pool.begin().await?;
        sqlx::raw_sql(
            "UPDATE postings SET entry_id = 2, line_no = 3 WHERE entry_id = 1 AND line_no = 3;
             INSERT INTO postings (entry_id, line_no, account_id, amount_dong)
             VALUES (2, 4, 'customer:CU-001', -40);",
        )
        .execute(&mut *tx)
        .await?;
        let (code, message) = pg_error(&tx.commit().await.unwrap_err()).expect("lỗi từ server");
        assert_eq!(code, "23514");
        assert!(message.contains("bút toán 1 lệch"), "{message}");

        // Xoá hết posting của một bút toán đã commit.
        let mut tx = db.pool.begin().await?;
        sqlx::query("DELETE FROM postings WHERE entry_id = 2")
            .execute(&mut *tx)
            .await?;
        let (_, message) = pg_error(&tx.commit().await.unwrap_err()).expect("lỗi từ server");
        assert!(message.contains("có 0 posting"), "{message}");

        // Một dòng.
        let (_, message) =
            pg_error(&ledger::post_one_line(&db.pool).await.unwrap_err()).expect("lỗi từ server");
        assert!(message.contains("có 1 posting"), "{message}");
        Ok(())
    })
    .await
}

#[tokio::test]
async fn idempotency_key_cannot_be_unique_across_partitions() -> Result<()> {
    with_lab(|db| async move {
        let rejected = partition::unique_key_without_partition_key(&db.pool).await?;
        assert_eq!(rejected.map(|(code, _)| code).as_deref(), Some("0A000"));
        assert!(partition::same_key_two_months(&db.pool).await?.is_none());
        Ok(())
    })
    .await
}

// Hai transaction khóa hai dòng số dư theo thứ tự ngược nhau: PostgreSQL phát hiện chờ vòng tròn
// sau `deadlock_timeout` và huỷ đúng một bên. `post_entry_in` tránh được nhờ duyệt theo tên tài khoản.
#[tokio::test]
async fn opposite_lock_order_deadlocks() -> Result<()> {
    with_lab(|db| async move {
        db.migrate_base().await?;
        ledger::seed_accounts(&db.pool).await?;
        let (x, y) = ("customer:CU-001", "driver:DR-001");
        let mut a = db.pool.begin().await?;
        let mut b = db.pool.begin().await?;
        for (tx, account) in [(&mut a, x), (&mut b, y)] {
            sqlx::query(ledger::UPSERT_BALANCE)
                .bind(account)
                .bind(1_i64)
                .execute(&mut **tx)
                .await?;
        }
        let (first, second) = tokio::join!(
            sqlx::query(ledger::UPSERT_BALANCE)
                .bind(y)
                .bind(1_i64)
                .execute(&mut *a),
            sqlx::query(ledger::UPSERT_BALANCE)
                .bind(x)
                .bind(1_i64)
                .execute(&mut *b),
        );
        let codes: Vec<String> = [first.err(), second.err()]
            .iter()
            .flatten()
            .filter_map(pg_error)
            .map(|(code, _)| code)
            .collect();
        assert_eq!(codes, ["40P01"]);
        Ok(())
    })
    .await
}
