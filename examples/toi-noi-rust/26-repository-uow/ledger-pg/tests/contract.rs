//! Cùng một bộ hợp đồng, hai adapter. Test PostgreSQL FAIL (không skip) nếu thiếu `TOI_NOI_PG_URL`.

use std::sync::Arc;

use ledger_pg::PgRepository;
use ledger_pg::harness::LabDb;
use ledger_port::{MemoryRepository, contract};

#[tokio::test(flavor = "multi_thread")]
async fn memory_adapter_passes_contract() {
    let repo = Arc::new(MemoryRepository::with_chart());
    contract::run(Arc::clone(&repo)).await.unwrap();
    contract::run_concurrent(repo).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn pg_adapter_passes_contract() {
    let lab = LabDb::setup().await.unwrap();
    let repo = Arc::new(PgRepository::new(lab.pool.clone()));
    let ran = contract::run(Arc::clone(&repo)).await;
    let concurrent = contract::run_concurrent(repo).await;
    // Dọn schema trước khi assert, để test hỏng cũng không để lại schema.
    lab.cleanup().await.unwrap();
    assert_eq!(ran.unwrap().len(), 7);
    concurrent.unwrap();
}

// Khác biệt có thật giữa hai adapter mà hợp đồng cố ý KHÔNG khẳng định: sequence của PostgreSQL
// không hoàn tác. Replay (ON CONFLICT DO NOTHING) và rollback đều "đốt" một số, mã có lỗ hổng.
#[tokio::test(flavor = "multi_thread")]
async fn pg_identity_has_gaps_memory_does_not() {
    use ledger_core::{AccountId, TripCharge, default_policy, fare_for_km};
    use ledger_port::post_trip_capture;

    fn trip(id: &str) -> TripCharge {
        TripCharge {
            trip_id: id.to_string(),
            customer: AccountId::new("customer:CU-001").unwrap(),
            driver: AccountId::new("driver:DR-001").unwrap(),
            fare: fare_for_km(5),
        }
    }
    let policy = default_policy();

    let mem = MemoryRepository::with_chart();
    post_trip_capture(&mem, &trip("TN-0001"), &policy)
        .await
        .unwrap();
    post_trip_capture(&mem, &trip("TN-0001"), &policy)
        .await
        .unwrap(); // replay
    let second = post_trip_capture(&mem, &trip("TN-0002"), &policy)
        .await
        .unwrap();
    assert_eq!(second.id.to_string(), "JE-000002");

    let lab = LabDb::setup().await.unwrap();
    let pg = PgRepository::new(lab.pool.clone());
    let a = post_trip_capture(&pg, &trip("TN-0001"), &policy).await;
    let b = post_trip_capture(&pg, &trip("TN-0001"), &policy).await; // replay
    let c = post_trip_capture(&pg, &trip("TN-0002"), &policy).await;
    lab.cleanup().await.unwrap();
    assert_eq!(a.unwrap().id.to_string(), "JE-000001");
    assert!(b.unwrap().replayed);
    assert_eq!(c.unwrap().id.to_string(), "JE-000003"); // JE-000002 bị replay đốt
}
