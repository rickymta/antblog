//! Test qua dây thật: server tonic trên `127.0.0.1:0`, client tonic. Bản PostgreSQL cần `TOI_NOI_PG_URL`.

use ledger_35_tonic_grpc::lab::{self, FlowReport};
use ledger_35_tonic_grpc::lab_db::LabDb;
use ledger_35_tonic_grpc::store::{MemoryStore, PgStore};
use tonic::Code;

fn assert_flow(f: &FlowReport) {
    assert_eq!(f.no_token, Code::Unauthenticated);
    assert_eq!(f.wrong_token, Code::Unauthenticated);
    assert_eq!(f.first, ("JE-000001".to_owned(), false));
    assert_eq!(f.second, ("JE-000001".to_owned(), true));
    assert_eq!(f.balances, (-25_500, 5_100, 20_400));
    assert_eq!(f.conflict, Code::AlreadyExists);
    assert_eq!(f.unbalanced, Code::InvalidArgument);
    assert_eq!(f.unknown_account_post, Code::FailedPrecondition);
    assert_eq!(f.unknown_account_read, Code::NotFound);
    assert_eq!(f.echoed_request_id, "req-lab-0001");
    assert_eq!(f.seen_request_id, "req-lab-0001");
}

#[tokio::test]
async fn memory_store_behaves_like_the_contract() {
    assert_flow(&lab::run_flow(MemoryStore::new()).await.unwrap());
}

// Cùng kịch bản, cùng assert, nhưng với PostgreSQL thật: hai adapter phải cư xử như nhau.
#[tokio::test]
async fn postgres_store_behaves_like_the_contract() {
    let db = LabDb::setup().await.unwrap();
    let schema = db.schema().to_owned();
    let flow = lab::run_flow(PgStore::new(db.pool.clone())).await;
    db.cleanup().await.unwrap();
    assert_flow(&flow.unwrap());
    assert!(
        !LabDb::schema_exists(&schema).await.unwrap(),
        "schema lab phải được dọn"
    );
}

#[tokio::test]
async fn deadline_stops_the_handler_and_returns_deadline_exceeded() {
    let d = lab::run_deadline(true).await.unwrap();
    assert_eq!(d.code, Code::DeadlineExceeded);
    assert_eq!(d.seen_timeout.as_deref(), Some("300000u"));
    assert_eq!(d.cancelled, 1);
    assert_eq!(d.completed_after_deadline, 0);
    assert_eq!(d.control_code, Code::Ok);
}

#[tokio::test]
async fn tonic_timeout_layer_alone_reports_cancelled() {
    let d = lab::run_deadline(false).await.unwrap();
    assert_eq!(d.code, Code::Cancelled);
    assert_eq!(d.control_code, Code::Ok);
}

#[tokio::test]
async fn tonic_client_layer_also_reports_cancelled() {
    assert_eq!(lab::silent_peer_code().await.unwrap(), Code::Cancelled);
}

// Hai ca PostgreSQL chạy song song không đụng nhau vì mỗi ca một schema.
#[tokio::test]
async fn concurrent_posts_of_the_same_key_make_one_entry() {
    use ledger_35_tonic_grpc::interceptor::LAB_TOKEN;
    use ledger_35_tonic_grpc::server;

    let db = LabDb::setup().await.unwrap();
    let server = server::start(PgStore::new(db.pool.clone()), LAB_TOKEN)
        .await
        .unwrap();
    let mut tasks = Vec::new();
    for _ in 0..8 {
        let addr = server.addr;
        tasks.push(tokio::spawn(async move {
            let mut client = lab::connect(addr, Some(LAB_TOKEN), "req-concurrent")
                .await
                .unwrap();
            client
                .post_entry(lab::capture_request())
                .await
                .unwrap()
                .into_inner()
        }));
    }
    let mut replays = 0;
    let mut ids = std::collections::BTreeSet::new();
    for task in tasks {
        let reply = task.await.unwrap();
        ids.insert(reply.entry_id);
        replays += usize::from(reply.replayed);
    }
    // Không khẳng định `JE-000001`: cột identity của PostgreSQL cấp số cho cả lần INSERT thua ON CONFLICT,
    // nên mã bút toán có thể nhảy cóc. Điều bảo đảm là cả tám lần cùng trỏ về MỘT bút toán.
    assert_eq!(ids.len(), 1, "mọi lời gọi thấy cùng một bút toán: {ids:?}");
    server.stop().await.unwrap();
    let entries: i64 = sqlx::query_scalar("SELECT count(*) FROM journal_entries")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    db.cleanup().await.unwrap();
    assert_eq!(entries, 1);
    assert_eq!(replays, 7, "một lần ghi thật, bảy lần replay");
}

#[test]
fn money_does_not_survive_double_above_2_pow_53() {
    let amount: i64 = (1 << 53) + 1;
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    let through_double = (amount as f64) as i64;
    assert_ne!(amount, through_double);
    // Số tiền thật của sổ cái nằm xa dưới 2^53, nhưng hợp đồng không nên phụ thuộc vào điều đó.
    let fare: i64 = 25_500;
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    let fare_through_double = (fare as f64) as i64;
    assert_eq!(fare, fare_through_double);
}
