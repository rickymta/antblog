//! Test với PostgreSQL thật. Mỗi ca tự dựng schema `lab_<uuid>` riêng nên chạy song song được;
//! `with_db` dọn schema kể cả khi assert hỏng. Thiếu `TOI_NOI_PG_URL` thì test FAIL, không skip.

use std::sync::{Arc, Mutex};

use ledger_ci::PostError;
use ledger_ci::testdb::{ENV_PG_URL, TestDb, pg_url_from, with_db};
use ledger_ci::trip::{TN_0001_ACCOUNTS, trip_entries};
use ledger_core::{AccountId, JournalEntryBuilder, LedgerError, Money};
use sqlx::{Connection, Executor};

#[tokio::test]
async fn missing_env_is_a_failure_not_a_skip() {
    let err = pg_url_from(None).unwrap_err().to_string();
    assert!(err.contains(ENV_PG_URL), "{err}");
    assert!(pg_url_from(Some("  ".to_owned())).is_err());
}

#[tokio::test]
async fn capture_then_split_leaves_tn_0001_balances() {
    with_db(|ledger| async move {
        for (id, kind) in TN_0001_ACCOUNTS {
            ledger.ensure_account(id, kind).await.unwrap();
        }
        let e = trip_entries("TN-0001", "customer:CU-001", "driver:DR-001", 5).unwrap();
        assert_eq!(
            ledger.post(&e.capture).await.unwrap().display_id(),
            "JE-000001"
        );
        assert_eq!(
            ledger.post(&e.split).await.unwrap().display_id(),
            "JE-000002"
        );

        let bal = |a: &'static str| {
            let l = ledger.clone();
            async move { l.balance(a).await.unwrap().unwrap() }
        };
        assert_eq!(bal("customer:CU-001").await, Money::from_dong(-25_500));
        assert_eq!(bal("platform:revenue").await, Money::from_dong(5_100));
        assert_eq!(bal("driver:DR-001").await, Money::from_dong(20_400));
        assert_eq!(bal("psp:vixanh:clearing").await, Money::ZERO);
    })
    .await;
}

#[tokio::test]
async fn same_key_is_a_replay_and_different_content_conflicts() {
    with_db(|ledger| async move {
        for (id, kind) in TN_0001_ACCOUNTS {
            ledger.ensure_account(id, kind).await.unwrap();
        }
        let e = trip_entries("TN-0001", "customer:CU-001", "driver:DR-001", 5).unwrap();
        let first = ledger.post(&e.capture).await.unwrap();
        let again = ledger.post(&e.capture).await.unwrap();
        assert_eq!((first.id, again.id, again.replayed), (1, 1, true));

        // Cùng key nhưng số tiền khác: xung đột, không ghi đè.
        let other = JournalEntryBuilder::new("trip:TN-0001:capture")
            .posting(
                AccountId::new("customer:CU-001").unwrap(),
                Money::from_dong(-1),
            )
            .posting(
                AccountId::new("psp:vixanh:clearing").unwrap(),
                Money::from_dong(1),
            )
            .build()
            .unwrap();
        let err = ledger.post(&other).await.unwrap_err();
        assert!(matches!(
            err,
            PostError::Ledger(LedgerError::IdempotencyConflict { .. })
        ));
        // Replay không cộng thêm vào số dư.
        let customer = ledger.balance("customer:CU-001").await.unwrap().unwrap();
        assert_eq!(customer, Money::from_dong(-25_500));
    })
    .await;
}

#[tokio::test]
async fn unbalanced_commit_is_rejected_by_the_database_trigger() {
    // Dùng pool trực tiếp: chứng minh ràng buộc nằm ở PostgreSQL, không chỉ ở ledger-core.
    let db = TestDb::try_new().await.unwrap();
    let result = async {
        let mut conn = db.pool.acquire().await?;
        let mut tx = conn.begin().await?;
        tx.execute("insert into accounts (id, kind) values ('customer:CU-001', 'liability')")
            .await?;
        tx.execute("insert into journal_entries (idempotency_key) values ('demo:lech')")
            .await?;
        tx.execute(
            "insert into postings (entry_id, line_no, account_id, amount_dong) \
             values (1, 1, 'customer:CU-001', 100)",
        )
        .await?;
        tx.commit().await
    }
    .await;
    db.close().await;
    let code = result
        .unwrap_err()
        .as_database_error()
        .and_then(|d| d.code())
        .map(|c| c.into_owned());
    assert_eq!(code.as_deref(), Some("23514"));
}

#[tokio::test]
async fn schema_is_dropped_even_when_the_body_panics() {
    let url = pg_url_from(std::env::var(ENV_PG_URL).ok()).unwrap();
    let seen = Arc::new(Mutex::new(String::new()));
    let seen_in_body = Arc::clone(&seen);
    // Body đọc tên schema rồi assert hỏng: with_db phải dọn rồi ném lại panic.
    let outcome = tokio::spawn(with_db(move |ledger| async move {
        let schema = ledger.current_schema().await.unwrap();
        seen_in_body.lock().unwrap().clone_from(&schema);
        assert!(schema.is_empty(), "giả lập assert hỏng trong test");
    }))
    .await;
    assert!(outcome.unwrap_err().is_panic(), "panic phải đi qua with_db");

    let schema = seen.lock().unwrap().clone();
    assert!(schema.starts_with("lab_"), "{schema}");
    assert!(
        !TestDb::schema_exists(&url, &schema).await.unwrap(),
        "schema {schema} còn sót"
    );
}
