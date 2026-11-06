//! Test của lab 52. Test cần PostgreSQL đọc `TOI_NOI_PG_URL`; thiếu biến thì FAIL rõ (không skip).

use std::sync::Arc;

use ledger_52_command_pipeline::domain::TripEntry;
use ledger_52_command_pipeline::events::DomainEvent;
use ledger_52_command_pipeline::lab::with_lab;
use ledger_52_command_pipeline::scenarios::{NOW, run_all, tn_0001};
use ledger_52_command_pipeline::{
    AppError, CommandHandler, Ctx, DynBus, Echo, LedgerHandlers, Ping, Traced, Validated, VoidHold,
    ledger_pipeline,
};

// Thứ tự lớp quan sát được bằng log, không cần database.
#[tokio::test]
async fn layer_order_is_visible_in_the_log() {
    let outer_validate = Validated(Traced(Echo));
    let mut ctx = Ctx::new(NOW);
    let out = outer_validate.handle(Ping { n: 41 }, &mut ctx).await;
    assert_eq!(out.unwrap(), 42);
    assert_eq!(
        ctx.trace(),
        [
            "validate: ok",
            "trace: bắt đầu Ping",
            "handler: Ping n=41",
            "trace: kết thúc Ping ok"
        ]
    );

    // Đổi thứ tự: lệnh bị từ chối vẫn được trace thấy.
    let outer_trace = Traced(Validated(Echo));
    let mut ctx = Ctx::new(NOW);
    let out = outer_trace.handle(Ping { n: 0 }, &mut ctx).await;
    assert!(matches!(out, Err(AppError::Validation(_))));
    assert_eq!(
        ctx.trace(),
        [
            "trace: bắt đầu Ping",
            "validate: từ chối Ping (n phải khác 0)",
            "trace: kết thúc Ping lỗi"
        ]
    );
}

// Kiểu hệ thống kiểm handler tồn tại, nhưng không kiểm lớp Transactional có mặt:
// handler trần gọi thẳng thì báo lỗi lúc chạy.
#[tokio::test]
async fn handler_without_transaction_layer_fails_at_runtime() {
    let mut ctx = Ctx::new(NOW);
    let out = LedgerHandlers.handle(tn_0001(), &mut ctx).await;
    assert!(matches!(out, Err(AppError::NoTransaction)));
}

#[tokio::test]
async fn dyn_bus_reports_missing_handler_at_runtime() {
    let mut bus = DynBus::new();
    bus.register::<Ping, _, _>(|cmd: Ping| async move { Ok::<u32, AppError>(cmd.n + 1) });
    assert_eq!(bus.send(Ping { n: 1 }).await.unwrap(), 2);
    let err = bus.send(VoidHold { hold_id: 7 }).await.unwrap_err();
    assert!(matches!(err, AppError::NoHandler("VoidHold")));
}

// Aggregate chỉ gom sự kiện; lấy ra một lần.
#[test]
fn aggregate_collects_events_until_taken() {
    use ledger_core::{AccountId, Money, TripCharge, default_policy};
    let trip = TripCharge {
        trip_id: "TN-0001".to_string(),
        customer: AccountId::new("customer:CU-001").unwrap(),
        driver: AccountId::new("driver:DR-001").unwrap(),
        fare: Money::from_dong(25_500),
    };
    let mut aggregate = TripEntry::capture(&trip, &default_policy()).unwrap();
    assert!(
        aggregate.take_events().is_empty(),
        "chưa lưu thì chưa có sự kiện"
    );
    aggregate.mark_posted("JE-000001", NOW);
    let events = aggregate.take_events();
    assert_eq!(events.len(), 1);
    assert!(aggregate.take_events().is_empty());
    let DomainEvent::EntryPosted(posted) = &events[0];
    assert_eq!(posted.schema_version, 1);
    assert_eq!(posted.postings.len(), 3);
    assert!(
        events[0]
            .payload()
            .contains("\"idempotencyKey\":\"trip:TN-0001:capture\"")
    );
}

// Mọi ca của bài trên PostgreSQL thật: thứ tự lớp, rollback, lỗi lúc commit, outbox.
#[tokio::test]
async fn scenarios_match_the_article() {
    with_lab(async |lab| {
        let report = run_all(&lab).await?;
        report.verify()
    })
    .await
    .unwrap();
}

// Future của pipeline generic là Send: spawn được, và bốn lệnh chạy song song trên nhiều thread.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn pipeline_future_is_send_and_idempotent_under_parallel_calls() {
    with_lab(async |lab| {
        let pipeline = Arc::new(ledger_pipeline(lab.pool.clone()));
        let mut tasks = Vec::new();
        for _ in 0..4 {
            let pipeline = pipeline.clone();
            tasks.push(tokio::spawn(async move {
                let mut ctx = Ctx::new(NOW);
                pipeline.handle(tn_0001(), &mut ctx).await
            }));
        }
        let mut created = 0;
        for task in tasks {
            // Cùng key chạy song song: lệnh đến sau chờ lệnh đầu commit rồi nhận replay;
            // không lệnh nào lỗi và không bao giờ có hai bút toán.
            if !task.await??.replayed {
                created += 1;
            }
        }
        let (entries, outbox) = lab.counts().await?;
        anyhow::ensure!(
            created == 1 && entries == 1 && outbox == 1,
            "{created} {entries} {outbox}"
        );
        Ok(())
    })
    .await
    .unwrap();
}

// Hàm thẳng và pipeline cho cùng kết quả và cùng dòng outbox trên hai schema riêng.
#[tokio::test]
async fn direct_function_matches_pipeline() {
    use ledger_52_command_pipeline::direct::capture_direct;

    let via_pipeline = with_lab(async |lab| {
        let pipeline = ledger_pipeline(lab.pool.clone());
        let mut ctx = Ctx::new(NOW);
        let receipt = pipeline.handle(tn_0001(), &mut ctx).await?;
        let payload: String = sqlx::query_scalar("SELECT payload::text FROM outbox")
            .fetch_one(&lab.pool)
            .await?;
        Ok((receipt, payload))
    })
    .await
    .unwrap();

    let via_direct = with_lab(async |lab| {
        let receipt = capture_direct(&lab.pool, tn_0001(), NOW).await?;
        let payload: String = sqlx::query_scalar("SELECT payload::text FROM outbox")
            .fetch_one(&lab.pool)
            .await?;
        Ok((receipt, payload))
    })
    .await
    .unwrap();

    assert_eq!(via_pipeline, via_direct);
}
