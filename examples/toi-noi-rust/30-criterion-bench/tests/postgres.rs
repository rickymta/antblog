//! Ghi PostgreSQL thật, mỗi ca một schema `lab_<uuid>` và dọn đúng schema đó.
//! Cần `TOI_NOI_PG_URL` (xem infra/README.md); thiếu thì test FAIL, không bỏ qua.

use ledger_bench::pg::{self, BoxError, LabDb, Strategy};
use ledger_bench::{IndexedLedger, make_entry};
use ledger_core::{AccountId, JournalEntry, Money};

const N: u64 = 120;

async fn write_and_check(db: &LabDb, strategy: Strategy) -> Result<(), BoxError> {
    let entries: Vec<JournalEntry> = (1..=N).map(make_entry).collect();
    let mut book = IndexedLedger::new();
    for e in &entries {
        book.post(e.clone())?;
    }
    let expected = book.balance(&AccountId::new("driver:DR-001")?);
    assert_eq!(expected, Money::from_dong(20_400));
    db.seed_accounts(&entries).await?;
    let run = pg::run(db, strategy, &entries).await?;
    assert_eq!(run.entries, entries.len());
    pg::check(db, entries.len(), "driver:DR-001", expected).await
}

async fn with_schema(strategy: Strategy) {
    let db = LabDb::create(4).await.expect("PostgreSQL lab phải chạy");
    let outcome = write_and_check(&db, strategy).await;
    db.drop_schema().await;
    outcome.unwrap();
}

#[tokio::test]
async fn per_entry_transaction() {
    with_schema(Strategy::PerEntryTx).await;
}

#[tokio::test]
async fn batched_statements() {
    with_schema(Strategy::Batched(25)).await;
}

#[tokio::test]
async fn batched_unnest() {
    with_schema(Strategy::BatchedUnnest(25)).await;
}

#[tokio::test]
async fn concurrent_transactions() {
    with_schema(Strategy::ConcurrentPerEntryTx(4)).await;
}

/// Ghi lại cùng 120 bút toán vào bảng đã có phải bị unique `idempotency_key` từ chối,
/// và transaction lỗi không để lại dòng nào.
#[tokio::test]
async fn duplicate_key_is_rejected_and_rolls_back() {
    let db = LabDb::create(2).await.expect("PostgreSQL lab phải chạy");
    let outcome = async {
        let entries: Vec<JournalEntry> = (1..=10).map(make_entry).collect();
        db.seed_accounts(&entries).await?;
        pg::run(&db, Strategy::BatchedUnnest(10), &entries).await?;
        // Cố ghi lại một lô trùng key, không reset: phải lỗi.
        let again = pg::run_on_existing(&db, Strategy::BatchedUnnest(10), &entries).await;
        assert!(again.is_err(), "key trùng phải bị từ chối");
        pg::check(&db, 10, "driver:DR-001", Money::from_dong(20_400)).await
    }
    .await;
    db.drop_schema().await;
    outcome.unwrap();
}
