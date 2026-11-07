//! Integration test trên PostgreSQL thật; mỗi ca có schema `lab_<uuid>` riêng.
//! Thiếu `TOI_NOI_PG_URL` thì ca fail rõ, không skip.

use std::sync::Arc;

use anyhow::Result;
use ledger_core::Money;
use ledger_lab::chargeback::{ChargebackService, LossPolicy};
use ledger_lab::db::{LabDb, with_lab};
use ledger_lab::error::RefundError;
use ledger_lab::ledger::{self, Trip, reverse_entry};
use ledger_lab::model::ReasonCode;
use ledger_lab::refund::{Guard, RefundService, SyncPoint};
use sqlx::{PgPool, Row};
use tokio::sync::Barrier;

const TRIP: Trip<'static> = Trip {
    id: "TN-0001",
    customer: "CU-001",
    driver: "DR-001",
    fare: Money::from_dong(25_500),
    external_ref: "VX-9F3A21",
};

fn d(n: i64) -> Money {
    Money::from_dong(n)
}

async fn status(pool: &PgPool) -> Result<String> {
    Ok(
        sqlx::query_scalar("select status from payments where trip_id = 'TN-0001'")
            .fetch_one(pool)
            .await?,
    )
}

async fn setup(db: &LabDb) -> Result<RefundService> {
    ledger::capture_trip(&db.pool, &TRIP).await?;
    Ok(RefundService::new(db.pool.clone()))
}

#[tokio::test(flavor = "multi_thread")]
async fn reject_returns_payment_to_captured_and_is_idempotent() -> Result<()> {
    with_lab(|db| async move {
        let svc = setup(&db).await?;
        let r = svc
            .create_request("TN-0001", d(5_000), ReasonCode::FareDispute, "cs.lan")
            .await?;
        assert_eq!(status(&db.pool).await?, "REFUND_PENDING");
        assert!(matches!(
            svc.reject(r, "cs.lan").await,
            Err(RefundError::SelfApproval { .. })
        ));
        let denied: i64 =
            sqlx::query_scalar("select count(*) from audit_log where action = 'REJECTION_DENIED'")
                .fetch_one(&db.pool)
                .await?;
        assert_eq!(denied, 1, "lần tự từ chối bị chặn vẫn có dấu vết");
        assert!(!svc.reject(r, "fin.binh").await?);
        assert!(
            svc.reject(r, "fin.cuong").await?,
            "từ chối lặp lại là replay"
        );
        assert_eq!(status(&db.pool).await?, "CAPTURED");
        assert!(matches!(
            svc.approve(r, "fin.binh").await,
            Err(RefundError::AlreadyDecided { .. })
        ));
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn two_pending_requests_keep_refund_pending_until_both_decided() -> Result<()> {
    with_lab(|db| async move {
        let svc = setup(&db).await?;
        let r1 = svc
            .create_request("TN-0001", d(5_000), ReasonCode::FareDispute, "cs.lan")
            .await?;
        let r2 = svc
            .create_request("TN-0001", d(7_000), ReasonCode::DuplicateCharge, "cs.lan")
            .await?;
        svc.approve(r1, "fin.binh").await?;
        assert_eq!(
            status(&db.pool).await?,
            "REFUND_PENDING",
            "còn yêu cầu 2 đang chờ"
        );
        svc.approve(r2, "fin.binh").await?;
        assert_eq!(
            status(&db.pool).await?,
            "CAPTURED",
            "hoàn một phần xong, chưa hết số đã thu"
        );
        let refunded: i64 = sqlx::query_scalar("select refunded_dong from payments")
            .fetch_one(&db.pool)
            .await?;
        assert_eq!(refunded, 12_000);
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn full_reversal_is_refused_after_partial_refund_or_while_pending() -> Result<()> {
    with_lab(|db| async move {
        let svc = setup(&db).await?;
        let r = svc
            .create_request("TN-0001", d(5_000), ReasonCode::FareDispute, "cs.lan")
            .await?;
        assert!(matches!(
            svc.reverse_trip("TN-0001", "fin.binh").await,
            Err(RefundError::PaymentNotRefundable { .. })
        ));
        svc.approve(r, "fin.binh").await?;
        assert!(matches!(
            svc.reverse_trip("TN-0001", "fin.binh").await,
            Err(RefundError::PartialRefundExists { .. })
        ));
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn nothing_to_refund_after_full_reversal() -> Result<()> {
    with_lab(|db| async move {
        let svc = setup(&db).await?;
        svc.reverse_trip("TN-0001", "fin.binh").await?;
        assert!(matches!(
            svc.create_request("TN-0001", d(1), ReasonCode::FareDispute, "cs.lan")
                .await,
            Err(RefundError::PaymentNotRefundable { .. })
        ));
        assert_eq!(ledger::grand_total(&db.pool).await?, 0);
        // Projection khớp nguồn sự thật.
        assert_eq!(
            ledger::balances(&db.pool).await?,
            ledger::balances_from_postings(&db.pool).await?
        );
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn cannot_reverse_a_reversal_inside_a_transaction() -> Result<()> {
    with_lab(|db| async move {
        let cap = ledger::capture_trip(&db.pool, &TRIP).await?;
        let mut tx = db.pool.begin().await?;
        let first = reverse_entry(&mut tx, cap.capture.id).await?;
        assert!(!first.replayed);
        assert!(matches!(
            reverse_entry(&mut tx, first.id).await,
            Err(RefundError::CannotReverseReversal(_))
        ));
        tx.rollback().await?;
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn wrong_reason_kind_is_rejected_both_ways() -> Result<()> {
    with_lab(|db| async move {
        let svc = setup(&db).await?;
        assert!(matches!(
            svc.create_request("TN-0001", d(1_000), ReasonCode::Fraud, "cs.lan")
                .await,
            Err(RefundError::WrongReasonKind(_))
        ));
        let cb = ChargebackService::new(db.pool.clone());
        assert!(matches!(
            cb.open("VX-CB-1", "TN-0001", d(1_000), ReasonCode::FareDispute)
                .await,
            Err(RefundError::WrongReasonKind(_))
        ));
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn chargeback_rules() -> Result<()> {
    with_lab(|db| async move {
        setup(&db).await?;
        let cb = ChargebackService::new(db.pool.clone());
        assert!(matches!(
            cb.open("VX-CB-1", "TN-0001", d(25_501), ReasonCode::NotReceived)
                .await,
            Err(RefundError::ChargebackExceedsCaptured { .. })
        ));
        cb.open("VX-CB-1", "TN-0001", d(25_500), ReasonCode::NotReceived)
            .await?;
        cb.resolve_lost("VX-CB-1", LossPolicy::RecoverFromDriver, "fin.binh")
            .await?;
        // Đã kết luận: không kết luận lần hai, kể cả theo hướng khác.
        assert!(matches!(
            cb.resolve_won("VX-CB-1", "fin.binh").await,
            Err(RefundError::AlreadyDecided { .. })
        ));
        assert!(matches!(
            cb.resolve_lost("VX-CB-1", LossPolicy::PlatformAbsorbs, "fin.binh")
                .await,
            Err(RefundError::AlreadyDecided { .. })
        ));
        assert!(matches!(
            cb.resolve_won("VX-CB-404", "fin.binh").await,
            Err(RefundError::UnknownChargeback(_))
        ));
        assert_eq!(ledger::grand_total(&db.pool).await?, 0);
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn ledger_and_audit_are_append_only() -> Result<()> {
    with_lab(|db| async move {
        setup(&db).await?;
        for sql in [
            "update journal_entries set idempotency_key = 'x' where id = 1",
            "delete from postings where entry_id = 1",
            "update audit_log set actor = 'ai-do'",
            "delete from audit_log",
        ] {
            let err = sqlx::query(sql).execute(&db.pool).await.expect_err(sql);
            // 23001 = restrict_violation, do trigger append-only.
            assert_eq!(
                err.as_database_error().and_then(|e| e.code()).as_deref(),
                Some("23001"),
                "{sql}"
            );
        }
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn database_rejects_unbalanced_entry_and_maker_equals_checker() -> Result<()> {
    with_lab(|db| async move {
        setup(&db).await?;
        // Bút toán lệch: trigger hoãn bắt lúc COMMIT.
        let mut tx = db.pool.begin().await?;
        sqlx::query("insert into journal_entries (idempotency_key) values ('thu-cong:lech')").execute(&mut *tx).await?;
        let id: i64 = sqlx::query_scalar("select id from journal_entries where idempotency_key = 'thu-cong:lech'").fetch_one(&mut *tx).await?;
        sqlx::query("insert into postings values ($1, 1, 'platform:revenue', 100), ($1, 2, 'driver:DR-001', -99)").bind(id).execute(&mut *tx).await?;
        assert!(tx.commit().await.is_err());
        // Yêu cầu có người duyệt trùng người tạo.
        let svc = RefundService::new(db.pool.clone());
        let r = svc.create_request("TN-0001", d(1_000), ReasonCode::FareDispute, "cs.lan").await?;
        let err = sqlx::query("update refund_requests set status = 'REJECTED', decided_by = 'cs.lan', decided_at = now() where id = $1")
            .bind(r)
            .execute(&db.pool)
            .await
            .expect_err("CHECK");
        assert_eq!(err.as_database_error().and_then(|e| e.constraint()), Some("checker_differs_from_maker"));
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrent_approvals_never_refund_more_than_captured() -> Result<()> {
    with_lab(|db| async move {
        let svc = setup(&db).await?;
        // Ba yêu cầu 10.000: hai cái là tối đa (20.000 ≤ 25.500), cái thứ ba làm tổng vượt.
        let mut ids = Vec::new();
        for _ in 0..3 {
            ids.push(svc.create_request("TN-0001", d(10_000), ReasonCode::FareDispute, "cs.lan").await?);
        }
        let gate = Arc::new(Barrier::new(3));
        let mut tasks = Vec::new();
        for (i, id) in ids.into_iter().enumerate() {
            let (s, g) = (svc.clone(), Arc::clone(&gate));
            let checker = format!("fin.{i}");
            tasks.push(tokio::spawn(async move {
                s.approve_with(id, &checker, Guard::Locked, SyncPoint::Start(g)).await
            }));
        }
        let mut ok = 0;
        for t in tasks {
            match t.await? {
                Ok(_) => ok += 1,
                Err(RefundError::ExceedsRefundable { .. }) => {}
                Err(other) => panic!("lỗi không mong đợi: {other}"),
            }
        }
        // Bất biến, không phụ thuộc ai thắng: đúng hai lần hoàn, tổng 20.000, khách còn nợ 5.500.
        assert_eq!(ok, 2);
        let refunded: i64 = sqlx::query_scalar("select refunded_dong from payments").fetch_one(&db.pool).await?;
        assert_eq!(refunded, 20_000);
        let customer = ledger::balances(&db.pool).await?["customer:CU-001"];
        assert_eq!(customer, -5_500);
        let n: i64 = sqlx::query("select count(*) as n from journal_entries where idempotency_key like 'trip:TN-0001:refund:%'")
            .fetch_one(&db.pool)
            .await?
            .get("n");
        assert_eq!(n, 2);
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn reason_codes_table_matches_enum() -> Result<()> {
    with_lab(|db| async move {
        let rows = sqlx::query("select code from reason_codes")
            .fetch_all(&db.pool)
            .await?;
        let mut table: Vec<String> = rows.iter().map(|r| r.get("code")).collect();
        let mut en: Vec<String> = ReasonCode::ALL
            .iter()
            .map(|r| r.as_str().to_string())
            .collect();
        table.sort();
        en.sort();
        assert_eq!(table, en);
        Ok(())
    })
    .await
}
