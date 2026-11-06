//! Integration test với PostgreSQL thật. Mỗi ca tự dựng schema `lab_<uuid>` riêng,
//! nên `cargo test` chạy các ca song song mà không đụng nhau.

mod common;

use common::{MARKER, TestDb, cleanup_stale_schemas, now_epoch, pg_url_from, schema_exists};
use ledger_core::{
    AccountId, JournalEntry, LedgerError, Money, Posting, TripCharge, capture_entry,
    default_policy, fare_for_km,
};
use ledger_lab::PostError;
use sqlx::{Connection, Executor, PgConnection};
use tokio::task::JoinSet;

// Bút toán thu cước của chuyến TN-0001: -25.500, +5.100, +20.400.
fn capture_tn_0001() -> JournalEntry {
    let trip = TripCharge {
        trip_id: "TN-0001".to_string(),
        customer: AccountId::new("customer:CU-001").unwrap(),
        driver: AccountId::new("driver:DR-001").unwrap(),
        fare: fare_for_km(5),
    };
    capture_entry(&trip, &default_policy()).unwrap()
}

// SQLSTATE của lỗi database; `None` nếu không phải lỗi từ PostgreSQL.
fn sqlstate(e: &sqlx::Error) -> Option<String> {
    e.as_database_error()
        .and_then(|d| d.code())
        .map(|c| c.into_owned())
}

#[tokio::test]
async fn capture_updates_balances_and_versions() {
    let db = TestDb::new().await;
    db.seed_accounts().await.unwrap();
    let ledger = db.ledger();

    let posted = ledger.post(&capture_tn_0001()).await.unwrap();
    assert_eq!(posted.display_id(), "JE-000001");
    assert!(!posted.replayed);

    let customer = ledger.balance("customer:CU-001").await.unwrap().unwrap();
    let driver = ledger.balance("driver:DR-001").await.unwrap().unwrap();
    let platform = ledger.balance("platform:revenue").await.unwrap().unwrap();
    assert_eq!(customer, (Money::from_dong(-25_500), 1));
    assert_eq!(driver, (Money::from_dong(20_400), 1));
    assert_eq!(platform, (Money::from_dong(5_100), 1));
    db.close().await;
}

#[tokio::test]
async fn same_key_same_content_is_a_replay_and_different_content_conflicts() {
    let db = TestDb::new().await;
    db.seed_accounts().await.unwrap();
    let ledger = db.ledger();
    let entry = capture_tn_0001();

    let first = ledger.post(&entry).await.unwrap();
    let again = ledger.post(&entry).await.unwrap();
    assert_eq!(again.id, first.id);
    assert!(again.replayed);
    // Ghi lại không nhân đôi số dư.
    assert_eq!(ledger.balance("driver:DR-001").await.unwrap().unwrap().1, 1);

    // Cùng key, số tiền khác: lỗi nghiệp vụ, không phải lỗi database.
    let other = JournalEntry::try_new(
        "trip:TN-0001:capture",
        vec![
            Posting::new(
                AccountId::new("customer:CU-001").unwrap(),
                Money::from_dong(-30_000),
            ),
            Posting::new(
                AccountId::new("driver:DR-001").unwrap(),
                Money::from_dong(30_000),
            ),
        ],
    )
    .unwrap();
    let err = ledger.post(&other).await.unwrap_err();
    assert!(
        matches!(
            err,
            PostError::Ledger(LedgerError::IdempotencyConflict { .. })
        ),
        "{err:?}"
    );
    db.close().await;
}

// Chứng minh cô lập: tám ca cùng ghi đúng một idempotency_key, cùng lúc, mỗi ca ở schema của mình.
// Nếu dùng chung một schema, bảy ca sau sẽ là replay hoặc đụng unique.
#[tokio::test]
async fn same_idempotency_key_in_parallel_schemas_never_collides() {
    let mut tasks = JoinSet::new();
    for _ in 0..8 {
        tasks.spawn(async {
            let db = TestDb::new().await;
            db.seed_accounts().await.unwrap();
            let posted = db.ledger().post(&capture_tn_0001()).await.unwrap();
            let schema = db.schema.clone();
            db.close().await;
            (schema, posted)
        });
    }
    let mut schemas = Vec::new();
    while let Some(done) = tasks.join_next().await {
        let (schema, posted) = done.unwrap();
        assert_eq!(
            posted.display_id(),
            "JE-000001",
            "identity bắt đầu lại từ 1 ở mỗi schema"
        );
        assert!(
            !posted.replayed,
            "ca ở schema {schema} không được thấy bút toán của ca khác"
        );
        schemas.push(schema);
    }
    schemas.sort();
    schemas.dedup();
    assert_eq!(schemas.len(), 8, "tám schema khác nhau");
}

#[tokio::test]
async fn nothing_is_created_in_the_public_schema() {
    let db = TestDb::new().await;
    let in_public: i64 = sqlx::query_scalar(
        "select count(*) from information_schema.tables \
         where table_schema = 'public' and table_name in ('accounts', 'journal_entries', 'postings', 'account_balances')",
    )
    .fetch_one(&db.pool)
    .await
    .unwrap();
    let in_lab: i64 = sqlx::query_scalar(
        "select count(*) from information_schema.tables where table_schema = current_schema() and table_name = 'postings'",
    )
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(in_public, 0);
    assert_eq!(in_lab, 1);
    db.close().await;
}

#[tokio::test]
async fn append_only_triggers_reject_update_delete_and_truncate() {
    let db = TestDb::new().await;
    db.seed_accounts().await.unwrap();
    db.ledger().post(&capture_tn_0001()).await.unwrap();

    for sql in [
        "update journal_entries set idempotency_key = 'x'",
        "delete from journal_entries",
        "update postings set amount_dong = 1",
        "delete from postings",
        "truncate postings",
    ] {
        let err = db.pool.execute(sql).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some("23001"), "{sql}: {err}");
        assert!(err.to_string().contains("append-only"), "{sql}: {err}");
    }
    // Sổ vẫn nguyên: 1 bút toán, 3 posting.
    let postings: i64 = sqlx::query_scalar("select count(*) from postings")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(postings, 3);
    // account_balances là projection nên không bị chặn.
    db.pool
        .execute("update account_balances set version = version")
        .await
        .unwrap();
    db.close().await;
}

// Constraint trigger hoãn: INSERT từng posting đều thành công, lỗi chỉ đến khi COMMIT.
#[tokio::test]
async fn unbalanced_entry_fails_at_commit_not_at_insert() {
    let db = TestDb::new().await;
    db.seed_accounts().await.unwrap();

    let mut tx = db.pool.begin().await.unwrap();
    let id: i64 = sqlx::query_scalar(
        "insert into journal_entries (idempotency_key) values ('demo:lech') returning id",
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    for (line, account, amount) in [(1, "customer:CU-001", -100_i64), (2, "driver:DR-001", 90)] {
        sqlx::query("insert into postings (entry_id, line_no, account_id, amount_dong) values ($1, $2, $3, $4)")
            .bind(id)
            .bind(line)
            .bind(account)
            .bind(amount)
            .execute(&mut *tx)
            .await
            .expect("INSERT chưa bị chặn vì trigger hoãn tới COMMIT");
    }
    let err = tx.commit().await.unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some("23514"), "{err}");
    assert!(err.to_string().contains("lệch"), "{err}");
    let left: i64 = sqlx::query_scalar("select count(*) from journal_entries")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(left, 0, "transaction đã rollback: không còn bút toán nào");
    db.close().await;
}

// Bút toán không có posting nào cũng phải bị chặn; trigger trên `journal_entries` bắt ca này.
#[tokio::test]
async fn entry_without_postings_fails_at_commit() {
    let db = TestDb::new().await;
    let mut tx = db.pool.begin().await.unwrap();
    sqlx::query("insert into journal_entries (idempotency_key) values ('demo:rong')")
        .execute(&mut *tx)
        .await
        .unwrap();
    let err = tx.commit().await.unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some("23514"), "{err}");
    db.close().await;
}

#[tokio::test]
async fn single_posting_and_zero_amount_are_rejected() {
    let db = TestDb::new().await;
    db.seed_accounts().await.unwrap();

    // Chỉ một posting: tổng khác 0 nên cũng không commit được (và dưới 2 dòng).
    let mut tx = db.pool.begin().await.unwrap();
    let id: i64 = sqlx::query_scalar(
        "insert into journal_entries (idempotency_key) values ('demo:mot-dong') returning id",
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    sqlx::query("insert into postings (entry_id, line_no, account_id, amount_dong) values ($1, 1, 'driver:DR-001', 0)")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map(|_| ())
        .expect_err("amount_dong = 0 vi phạm CHECK ngay lúc INSERT");
    drop(tx);

    let mut tx = db.pool.begin().await.unwrap();
    let id: i64 = sqlx::query_scalar(
        "insert into journal_entries (idempotency_key) values ('demo:mot-dong-2') returning id",
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    sqlx::query("insert into postings (entry_id, line_no, account_id, amount_dong) values ($1, 1, 'driver:DR-001', 5)")
        .bind(id)
        .execute(&mut *tx)
        .await
        .unwrap();
    let err = tx.commit().await.unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some("23514"), "{err}");
    db.close().await;
}

// SQLSTATE 40001: hai transaction SERIALIZABLE cùng đọc rồi ghi chéo nhau (write skew).
// SQLite không có mức cô lập này, nên không dựng được ca này trên SQLite.
#[tokio::test]
async fn serializable_write_skew_raises_sqlstate_40001() {
    let db = TestDb::new().await;
    db.seed_accounts().await.unwrap();
    db.ledger().post(&capture_tn_0001()).await.unwrap();

    let mut t1 = db.pool.acquire().await.unwrap();
    let mut t2 = db.pool.acquire().await.unwrap();
    t1.execute("begin isolation level serializable")
        .await
        .unwrap();
    t2.execute("begin isolation level serializable")
        .await
        .unwrap();
    // Cả hai đọc tổng số dư trước, rồi mỗi bên ghi vào một tài khoản khác nhau.
    for t in [&mut t1, &mut t2] {
        let _: i64 = sqlx::query_scalar("select sum(balance_dong)::bigint from account_balances")
            .fetch_one(&mut **t)
            .await
            .unwrap();
    }
    t1.execute("update account_balances set balance_dong = balance_dong + 1 where account_id = 'driver:DR-001'")
        .await
        .unwrap();
    t2.execute("update account_balances set balance_dong = balance_dong + 1 where account_id = 'platform:revenue'")
        .await
        .unwrap();
    t1.execute("commit").await.expect("bên commit trước thắng");
    let err = t2.execute("commit").await.unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some("40001"), "{err}");
    drop(t1);
    drop(t2);
    db.close().await;
}

// Drop không async nên không dọn được: nếu test panic trước close(), schema sót.
// Hàm dọn theo tuổi gỡ nó, nhưng không đụng schema của ca đang chạy.
#[tokio::test]
async fn schema_left_by_a_skipped_close_is_removed_by_age_based_cleanup() {
    let orphan = TestDb::new().await;
    let live = TestDb::new().await;
    let url = orphan.url.clone();
    let orphan_name = orphan.schema.clone();
    let live_name = live.schema.clone();

    drop(orphan); // giống đường panic: close() không được gọi
    assert!(
        schema_exists(&url, &orphan_name).await.unwrap(),
        "schema sót vì Drop không dọn được"
    );

    // Chưa đủ tuổi: dọn với ngưỡng 1 giờ không gỡ gì của hai schema này.
    let dropped = cleanup_stale_schemas(&url, 3600).await.unwrap();
    assert!(!dropped.contains(&orphan_name));

    // Giả lập schema đã mồ côi hai giờ bằng cách lùi dấu thời gian trong comment.
    let mut admin = PgConnection::connect(&url).await.unwrap();
    admin
        .execute(sqlx::AssertSqlSafe(format!(
            "comment on schema {orphan_name} is '{MARKER}{}'",
            now_epoch() - 7200
        )))
        .await
        .unwrap();
    admin.close().await.unwrap();

    let dropped = cleanup_stale_schemas(&url, 3600).await.unwrap();
    assert!(dropped.contains(&orphan_name), "{dropped:?}");
    assert!(!schema_exists(&url, &orphan_name).await.unwrap());
    assert!(
        schema_exists(&url, &live_name).await.unwrap(),
        "schema của ca đang chạy còn nguyên"
    );
    live.close().await;
}

#[test]
fn missing_or_blank_pg_url_is_an_error_that_says_how_to_fix_it() {
    for value in [None, Some(String::new()), Some("   ".to_string())] {
        let message = pg_url_from(value).unwrap_err().to_string();
        assert!(message.contains("TOI_NOI_PG_URL"), "{message}");
        assert!(message.contains("docker compose"), "{message}");
    }
    assert_eq!(
        pg_url_from(Some("postgres://x".to_string())).unwrap(),
        "postgres://x"
    );
}
