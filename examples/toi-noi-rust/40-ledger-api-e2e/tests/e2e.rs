//! Test end-to-end với PostgreSQL và Kafka thật. Mỗi test tự dựng schema và topic riêng,
//! nên chạy song song được. Thiếu hạ tầng thì test FAIL rõ, không skip.

use std::time::{Duration, Instant};

use anyhow::Result;
use ledger_api::config::Fault;
use ledger_api::events::TripCompleted;
use ledger_api::harness::LabEnv;
use ledger_api::scenario;
use ledger_api::store::{Applied, Store, StoreError};
use rdkafka::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};

#[tokio::test(flavor = "multi_thread")]
async fn trip_completed_to_balance_over_http_and_grpc_then_replay() -> Result<()> {
    scenario::flow_and_replay().await.map(drop)
}

#[tokio::test(flavor = "multi_thread")]
async fn crash_after_db_commit_before_offset_then_restart() -> Result<()> {
    scenario::crash_and_restart(Fault::AfterDbCommitBeforeOffset)
        .await
        .map(drop)
}

#[tokio::test(flavor = "multi_thread")]
async fn crash_before_db_commit_then_restart() -> Result<()> {
    scenario::crash_and_restart(Fault::BeforeDbCommit)
        .await
        .map(drop)
}

#[tokio::test(flavor = "multi_thread")]
async fn grpc_post_entry_is_idempotent() -> Result<()> {
    scenario::grpc_post_entry().await.map(drop)
}

#[tokio::test(flavor = "multi_thread")]
async fn sigterm_exits_zero_and_logs_request_id() -> Result<()> {
    scenario::sigterm_child(std::path::Path::new(env!("CARGO_BIN_EXE_ledger-api")))
        .await
        .map(drop)
}

// Không cần Kafka chạy consumer: gọi thẳng `Store`. Rollback phải xoá sạch dấu vết ở CẢ NĂM bảng.
#[tokio::test(flavor = "multi_thread")]
async fn rollback_leaves_no_trace_in_any_table() -> Result<()> {
    LabEnv::run(|env| async move {
        let mut cfg = env.config()?;
        cfg.fault = Fault::BeforeDbCommit;
        let store = Store::connect(&cfg).await?;
        let trip = TripCompleted::sample("TN-0001", 25_500);
        let outcome = store.apply_trip_completed(&trip).await;
        assert!(
            matches!(outcome, Err(StoreError::InjectedCrash)),
            "{outcome:?}"
        );

        let pool = env.inspect_pool().await?;
        for table in [
            "journal_entries",
            "postings",
            "account_balances",
            "inbox",
            "outbox",
        ] {
            // Tên bảng là hằng trong mảng trên, không có dữ liệu ngoài.
            let n: i64 =
                sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT count(*) FROM {table}")))
                    .fetch_one(&pool)
                    .await?;
            assert_eq!(n, 0, "bảng {table} còn dòng sau rollback");
        }
        pool.close().await;
        Ok(())
    })
    .await
}

// Cùng sự kiện hai lần: lần hai là Duplicate và số dư không đổi.
#[tokio::test(flavor = "multi_thread")]
async fn second_delivery_is_a_duplicate() -> Result<()> {
    LabEnv::run(|env| async move {
        let store = Store::connect(&env.config()?).await?;
        let trip = TripCompleted::sample("TN-0001", 25_500);
        assert!(matches!(
            store.apply_trip_completed(&trip).await?,
            Applied::Posted(_)
        ));
        assert_eq!(store.apply_trip_completed(&trip).await?, Applied::Duplicate);
        assert_eq!(store.balance("driver:DR-001").await?, Some(20_400));
        assert_eq!(store.balance("driver:DR-404").await?, None);
        Ok(())
    })
    .await
}

// Giới hạn đã biết: inbox khoá theo mã chuyến, không theo nội dung. Cùng chuyến mà cước khác
// bị coi là bản trùng và bỏ qua IM LẶNG. Sửa cước đúng cách (bút toán đảo) thuộc cấp nâng cao.
#[tokio::test(flavor = "multi_thread")]
async fn same_trip_with_other_fare_is_ignored_silently() -> Result<()> {
    LabEnv::run(|env| async move {
        let store = Store::connect(&env.config()?).await?;
        assert!(matches!(
            store
                .apply_trip_completed(&TripCompleted::sample("TN-0001", 25_500))
                .await?,
            Applied::Posted(_)
        ));
        let corrected = TripCompleted::sample("TN-0001", 26_000);
        assert_eq!(
            store.apply_trip_completed(&corrected).await?,
            Applied::Duplicate
        );
        assert_eq!(
            store.balance("driver:DR-001").await?,
            Some(20_400),
            "cước mới không được áp"
        );
        Ok(())
    })
    .await
}

// Dọn đúng: sau `cleanup` không còn schema `lab_<uuid>` và hai topic biến mất.
#[tokio::test(flavor = "multi_thread")]
async fn cleanup_removes_schema_and_topics() -> Result<()> {
    let env = LabEnv::setup().await?;
    let probe = env.clone();
    env.cleanup().await?;

    let mut admin = <sqlx::PgConnection as sqlx::Connection>::connect(&probe.pg_url).await?;
    let schemas: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_namespace WHERE nspname = $1")
        .bind(&probe.schema)
        .fetch_one(&mut admin)
        .await?;
    assert_eq!(schemas, 0, "schema {} còn tồn tại", probe.schema);

    // Xoá topic của Kafka là bất đồng bộ: chờ tới hạn chót thay vì ngủ cố định.
    let consumer: BaseConsumer = ClientConfig::new()
        .set("bootstrap.servers", &probe.kafka)
        .set("group.id", format!("{}-meta", probe.group))
        .create()?;
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let metadata = consumer.fetch_metadata(None, Duration::from_secs(5))?;
        let left = metadata
            .topics()
            .iter()
            .filter(|t| t.name().starts_with(&probe.prefix))
            .count();
        if left == 0 {
            return Ok(());
        }
        assert!(
            Instant::now() < deadline,
            "còn {left} topic của ca sau khi dọn"
        );
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
