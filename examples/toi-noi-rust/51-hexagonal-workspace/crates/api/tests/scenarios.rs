//! Chạy các kịch bản của `--verify` dưới dạng test, và kiểm hợp đồng chung của hai kho.
//!
//! Cần PostgreSQL (`TOI_NOI_PG_URL`) và, khi bật feature `kafka`, Kafka (`TOI_NOI_KAFKA`).
//! Thiếu biến hoặc không kết nối được thì test FAIL, không skip.

use api::scenario;
use application::{LedgerStore, StoreError};
use domain::{AccountId, JournalEntry, Money, Posting};
use infrastructure::memory::MemoryStore;
use infrastructure::pg::{LabDb, PgStore};

fn entry(key: &str, from: &str, to: &str, dong: i64) -> JournalEntry {
    JournalEntry::try_new(
        key,
        vec![
            Posting::new(AccountId::new(from).unwrap(), Money::from_dong(-dong)),
            Posting::new(AccountId::new(to).unwrap(), Money::from_dong(dong)),
        ],
    )
    .unwrap()
}

// Hợp đồng mà MỌI adapter của `LedgerStore` phải giữ. Viết một lần, chạy cho từng adapter.
async fn store_contract<S: LedgerStore>(store: &S) {
    let capture = entry(
        "trip:TN-0001:capture",
        "customer:CU-001",
        "psp:vixanh:clearing",
        25_500,
    );
    let first = store.post(&capture).await.unwrap();
    assert!(!first.replayed);

    let again = store.post(&capture).await.unwrap();
    assert!(again.replayed);
    assert_eq!(again.id, first.id);

    let other_amount = entry(
        "trip:TN-0001:capture",
        "customer:CU-001",
        "psp:vixanh:clearing",
        1,
    );
    assert!(matches!(
        store.post(&other_amount).await,
        Err(StoreError::IdempotencyConflict(_))
    ));

    let unknown = entry(
        "trip:TN-0002:capture",
        "customer:CU-404",
        "psp:vixanh:clearing",
        1,
    );
    assert!(
        matches!(store.post(&unknown).await, Err(StoreError::UnknownAccount(a)) if a == "customer:CU-404")
    );

    let customer = AccountId::new("customer:CU-001").unwrap();
    assert_eq!(
        store.balance(&customer).await.unwrap(),
        Money::from_dong(-25_500)
    );
    let nobody = AccountId::new("driver:DR-001").unwrap();
    assert_eq!(store.balance(&nobody).await.unwrap(), Money::ZERO);
}

#[tokio::test]
async fn memory_store_keeps_the_contract() {
    store_contract(&MemoryStore::new()).await;
}

#[tokio::test]
async fn pg_store_keeps_the_contract() {
    let lab = LabDb::setup().await.unwrap();
    // Chạy rồi dọn schema kể cả khi hợp đồng vỡ giữa chừng.
    let store = PgStore::new(lab.pool.clone());
    let run = tokio::spawn(async move { store_contract(&store).await });
    let joined = run.await;
    lab.cleanup().await.unwrap();
    joined.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn scenarios_agree_across_adapters() {
    let events = scenario::memory().await.unwrap();
    scenario::postgres(&events).await.unwrap();
    scenario::kafka(&events).await.unwrap();
}
