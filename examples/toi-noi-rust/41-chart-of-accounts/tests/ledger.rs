//! Test với PostgreSQL thật. Mỗi test có schema `lab_<uuid>` riêng, dọn cuối ca kể cả khi lỗi.
//! Không test nào đọc đồng hồ máy hay `sleep`: thời gian nghiệp vụ là chuỗi ISO truyền vào.

use anyhow::Result;
use ledger_41_chart_of_accounts::chart::trial_totals;
use ledger_41_chart_of_accounts::db::with_lab;
use ledger_41_chart_of_accounts::flows::{self, CLEARING};
use ledger_41_chart_of_accounts::metrics::{self, Policy};
use ledger_41_chart_of_accounts::store::{self, PostOutcome};
use ledger_core::{JournalEntryBuilder, LedgerError, Money, fare_for_km};
use sqlx::PgPool;

const AS_OF: &str = "2026-11-06T00:00:00Z";

fn policy(customer_may_be_negative: bool) -> Policy<'static> {
    Policy {
        as_of: AS_OF,
        clearing_max_days: 3,
        customer_may_be_negative,
    }
}

async fn count(pool: &PgPool, table: &'static str) -> Result<i64> {
    let sql: &'static str = match table {
        "accounts" => "SELECT COUNT(*) FROM accounts",
        "journal_entries" => "SELECT COUNT(*) FROM journal_entries",
        _ => "SELECT COUNT(*) FROM postings",
    };
    Ok(sqlx::query_scalar(sql).fetch_one(pool).await?)
}

async fn dong_of(pool: &PgPool, account: &str) -> Result<i64> {
    let all = store::balances(pool).await?;
    Ok(all
        .iter()
        .find(|b| b.account == account)
        .map_or(0, |b| b.dong))
}

#[tokio::test]
async fn trip_two_steps_leave_clearing_empty_and_total_zero() -> Result<()> {
    with_lab(|pool| async move {
        let fare = Money::from_dong(25_500);
        store::post(
            &pool,
            &flows::trip_capture("TN-0001", "CU-001", fare)?,
            "2026-11-05T01:30:00Z",
        )
        .await?;
        assert_eq!(dong_of(&pool, CLEARING).await?, 25_500);
        store::post(
            &pool,
            &flows::trip_split("TN-0001", "DR-001", fare)?,
            "2026-11-05T01:31:00Z",
        )
        .await?;
        assert_eq!(dong_of(&pool, CLEARING).await?, 0);
        assert_eq!(dong_of(&pool, "customer:CU-001").await?, -25_500);
        assert_eq!(dong_of(&pool, "platform:revenue").await?, 5_100);
        assert_eq!(dong_of(&pool, "driver:DR-001").await?, 20_400);
        let report = metrics::measure(&pool, &policy(true)).await?;
        assert!(report.is_healthy());
        Ok(())
    })
    .await
}

#[tokio::test]
async fn reading_by_kind_balances_only_once_clearing_is_empty() -> Result<()> {
    with_lab(|pool| async move {
        let fare = Money::from_dong(25_500);
        store::post(
            &pool,
            &flows::trip_capture("TN-0001", "CU-001", fare)?,
            "2026-11-05T01:30:00Z",
        )
        .await?;
        let totals = |b: Vec<store::Balance>| {
            trial_totals(&b.iter().map(|x| (x.kind, x.dong)).collect::<Vec<_>>())
        };
        assert_eq!(totals(store::balances(&pool).await?), (51_000, 0));
        store::post(
            &pool,
            &flows::trip_split("TN-0001", "DR-001", fare)?,
            "2026-11-05T01:31:00Z",
        )
        .await?;
        assert_eq!(totals(store::balances(&pool).await?), (25_500, 25_500));
        Ok(())
    })
    .await
}

#[tokio::test]
async fn delivery_posts_four_lines_with_assumed_vat() -> Result<()> {
    with_lab(|pool| async move {
        let fee = Money::from_dong(33_000);
        store::post(
            &pool,
            &flows::delivery_capture("DL-0001", "CU-001", fee)?,
            "2026-11-05T05:10:00Z",
        )
        .await?;
        store::post(
            &pool,
            &flows::delivery_split("DL-0001", "DR-002", fee)?,
            "2026-11-05T05:11:00Z",
        )
        .await?;
        assert_eq!(dong_of(&pool, "platform:revenue").await?, 6_000);
        assert_eq!(dong_of(&pool, flows::VAT).await?, 600);
        assert_eq!(dong_of(&pool, "driver:DR-002").await?, 26_400);
        assert_eq!(dong_of(&pool, CLEARING).await?, 0);
        assert!(metrics::measure(&pool, &policy(true)).await?.is_healthy());
        Ok(())
    })
    .await
}

#[tokio::test]
async fn same_key_same_content_replays_other_content_conflicts() -> Result<()> {
    with_lab(|pool| async move {
        let fare = Money::from_dong(25_500);
        let capture = flows::trip_capture("TN-0001", "CU-001", fare)?;
        let first = store::post(&pool, &capture, "2026-11-05T01:30:00Z").await?;
        let again = store::post(&pool, &capture, "2026-11-05T01:30:00Z").await?;
        let PostOutcome::Created { entry_id } = first else {
            panic!("lần đầu phải là Created")
        };
        assert_eq!(again, PostOutcome::Replayed { entry_id });

        let legacy = flows::legacy_three_line("TN-0001", "CU-001", "DR-001", fare)?;
        let err = store::post(&pool, &legacy, "2026-11-05T01:30:00Z")
            .await
            .unwrap_err();
        assert_eq!(
            err.downcast_ref::<LedgerError>(),
            Some(&LedgerError::IdempotencyConflict {
                key: "trip:TN-0001:capture".to_string()
            })
        );
        // Chỉ còn bút toán đầu tiên: hai dòng.
        assert_eq!(count(&pool, "journal_entries").await?, 1);
        assert_eq!(count(&pool, "postings").await?, 2);
        Ok(())
    })
    .await
}

#[tokio::test]
async fn account_outside_the_chart_is_rejected_without_trace() -> Result<()> {
    with_lab(|pool| async move {
        let entry = JournalEntryBuilder::new("demo:wallet")
            .posting(
                ledger_core::AccountId::new("wallet:CU-001")?,
                Money::from_dong(-100),
            )
            .posting(
                ledger_core::AccountId::new(CLEARING)?,
                Money::from_dong(100),
            )
            .build()?;
        let err = store::post(&pool, &entry, "2026-11-05T01:30:00Z")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("không thuộc sơ đồ"), "{err}");
        assert_eq!(count(&pool, "journal_entries").await?, 0);
        assert_eq!(count(&pool, "accounts").await?, 0);
        Ok(())
    })
    .await
}

#[tokio::test]
async fn db_trigger_rejects_unbalanced_commit_and_nothing_remains() -> Result<()> {
    with_lab(|pool| async move {
        let err = store::post_raw(
            &pool,
            "demo:unbalanced",
            "2026-11-05T10:00:00Z",
            &[("customer:CU-001", -25_500), (CLEARING, 25_000)],
        )
        .await
        .unwrap_err();
        let db = err
            .downcast_ref::<sqlx::Error>()
            .and_then(|e| e.as_database_error())
            .expect("lỗi từ PostgreSQL");
        assert_eq!(db.code().as_deref(), Some("23514"));
        assert_eq!(count(&pool, "journal_entries").await?, 0);
        assert_eq!(count(&pool, "postings").await?, 0);
        Ok(())
    })
    .await
}

#[tokio::test]
async fn pending_clearing_is_normal_until_it_is_older_than_the_limit() -> Result<()> {
    with_lab(|pool| async move {
        // Đúng 3 ngày trước mốc đo: quá hạn (so sánh <=). Sớm hơn 1 giây thì chưa.
        let on_the_line = flows::trip_capture("TN-0010", "CU-010", Money::from_dong(10_000))?;
        let just_inside = flows::trip_capture("TN-0011", "CU-011", Money::from_dong(11_000))?;
        store::post(&pool, &on_the_line, "2026-11-03T00:00:00Z").await?;
        store::post(&pool, &just_inside, "2026-11-03T00:00:01Z").await?;
        let report = metrics::measure(&pool, &policy(true)).await?;
        assert_eq!(report.stuck_clearing.len(), 1);
        assert_eq!(report.stuck_clearing[0].reference, "trip:TN-0010");
        assert_eq!(report.stuck_clearing[0].stuck_dong, 10_000);
        assert_eq!(report.stuck_clearing[0].age_days, 3);
        // Chia xong thì hết tồn đọng, dù capture cũ đến đâu.
        store::post(
            &pool,
            &flows::trip_split("TN-0010", "DR-001", Money::from_dong(10_000))?,
            "2026-11-05T00:00:00Z",
        )
        .await?;
        let after = metrics::measure(&pool, &policy(true)).await?;
        assert!(after.stuck_clearing.is_empty());
        Ok(())
    })
    .await
}

#[tokio::test]
async fn customer_wallet_may_go_negative_only_when_policy_allows() -> Result<()> {
    with_lab(|pool| async move {
        let fare = Money::from_dong(25_500);
        store::post(
            &pool,
            &flows::trip_capture("TN-0001", "CU-001", fare)?,
            "2026-11-05T01:30:00Z",
        )
        .await?;
        store::post(
            &pool,
            &flows::trip_split("TN-0001", "DR-001", fare)?,
            "2026-11-05T01:31:00Z",
        )
        .await?;
        assert!(
            metrics::measure(&pool, &policy(true))
                .await?
                .abnormal_negative
                .is_empty()
        );
        let strict = metrics::measure(&pool, &policy(false)).await?;
        assert_eq!(strict.abnormal_negative.len(), 1);
        assert_eq!(strict.abnormal_negative[0].account, "customer:CU-001");
        assert_eq!(strict.abnormal_negative[0].balance_dong, -25_500);
        Ok(())
    })
    .await
}

#[tokio::test]
async fn every_injected_fault_is_found_by_its_own_metric() -> Result<()> {
    with_lab(|pool| async move {
        let lopsided = [("customer:CU-001", -25_500), (CLEARING, 25_000)];
        store::set_balance_triggers(&pool, false).await?;
        store::post_raw(
            &pool,
            "trip:TN-0004:capture",
            "2026-11-05T10:00:00Z",
            &lopsided,
        )
        .await?;
        store::set_balance_triggers(&pool, true).await?;
        store::post(
            &pool,
            &flows::trip_capture("TN-0003", "CU-003", fare_for_km(4))?,
            "2026-10-20T03:00:00Z",
        )
        .await?;
        let report = metrics::measure(&pool, &policy(true)).await?;
        assert_eq!(report.unbalanced.len(), 1);
        assert_eq!(report.unbalanced[0].total_dong, -500);
        assert_eq!(report.postings_total, -500);
        assert_eq!(report.projection_total, -500);
        assert_eq!(report.stuck_clearing.len(), 1);
        assert_eq!(report.stuck_clearing[0].reference, "trip:TN-0003");
        assert!(report.abnormal_negative.is_empty());
        assert!(!report.is_healthy());
        Ok(())
    })
    .await
}

#[tokio::test]
async fn schema_is_dropped_even_when_the_body_fails() -> Result<()> {
    use sqlx::Connection;
    let url = std::env::var("TOI_NOI_PG_URL")?;
    let seen = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let seen_in = seen.clone();
    let outcome: Result<()> = with_lab(|pool| async move {
        let schema: String = sqlx::query_scalar("SELECT current_schema()")
            .fetch_one(&pool)
            .await?;
        *seen_in.lock().unwrap() = schema;
        anyhow::bail!("lỗi cố ý trong thân ca chạy")
    })
    .await;
    assert!(outcome.is_err());
    let schema = seen.lock().unwrap().clone();
    assert!(schema.starts_with("lab_"));
    let mut conn = sqlx::PgConnection::connect(&url).await?;
    let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pg_namespace WHERE nspname = $1")
        .bind(&schema)
        .fetch_one(&mut conn)
        .await?;
    assert_eq!(left, 0);
    Ok(())
}

#[tokio::test]
async fn topup_makes_clearing_the_boundary_account() -> Result<()> {
    with_lab(|pool| async move {
        let fare = Money::from_dong(25_500);
        store::post(
            &pool,
            &flows::wallet_topup("CU-001", Money::from_dong(40_000))?,
            "2026-11-01T01:00:00Z",
        )
        .await?;
        assert_eq!(dong_of(&pool, CLEARING).await?, -40_000);
        assert_eq!(dong_of(&pool, "customer:CU-001").await?, 40_000);
        store::post(
            &pool,
            &flows::trip_capture("TN-0001", "CU-001", fare)?,
            "2026-11-01T01:30:00Z",
        )
        .await?;
        assert_eq!(dong_of(&pool, CLEARING).await?, -14_500);
        store::post(
            &pool,
            &flows::trip_split("TN-0001", "DR-001", fare)?,
            "2026-11-01T01:31:00Z",
        )
        .await?;
        assert_eq!(dong_of(&pool, CLEARING).await?, -40_000);
        assert_eq!(dong_of(&pool, "customer:CU-001").await?, 14_500);

        // Sổ cân, ví khách không âm; hai metric viết cho mô hình trả theo chuyến báo nhầm clearing.
        let report = metrics::measure(&pool, &policy(false)).await?;
        assert!(report.unbalanced.is_empty());
        assert_eq!(report.postings_total, 0);
        assert_eq!(report.projection_total, 0);
        assert_eq!(report.abnormal_negative.len(), 1);
        assert_eq!(report.abnormal_negative[0].account, CLEARING);
        assert_eq!(report.stuck_clearing.len(), 1);
        assert_eq!(report.stuck_clearing[0].reference, "fixture:topup");
        assert_eq!(report.stuck_clearing[0].stuck_dong, -40_000);
        Ok(())
    })
    .await
}

#[tokio::test]
async fn split_without_capture_is_caught_by_two_metrics() -> Result<()> {
    with_lab(|pool| async move {
        // Sự kiện chia đến trước capture: clearing âm 25.500, sổ vẫn cân từng bút toán.
        store::post(
            &pool,
            &flows::trip_split("TN-0001", "DR-001", Money::from_dong(25_500))?,
            "2026-11-01T01:31:00Z",
        )
        .await?;
        let report = metrics::measure(&pool, &policy(true)).await?;
        assert!(report.unbalanced.is_empty());
        assert_eq!(report.postings_total, 0);
        let negative: Vec<_> = report
            .abnormal_negative
            .iter()
            .map(|a| a.account.as_str())
            .collect();
        assert_eq!(negative, [CLEARING]);
        assert_eq!(report.abnormal_negative[0].kind, "asset");
        assert_eq!(report.stuck_clearing.len(), 1);
        assert_eq!(report.stuck_clearing[0].stuck_dong, -25_500);
        assert_eq!(report.stuck_clearing[0].age_days, 4);
        Ok(())
    })
    .await
}
