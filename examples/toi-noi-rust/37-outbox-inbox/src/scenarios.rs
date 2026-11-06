//! Các thí nghiệm của bài 37. Mỗi hàm trả về một báo cáo; `verify()` kiểm các giá trị bài viết nêu.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Notify;

use anyhow::{Context, Result, ensure};
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;

use crate::consumer::{ConsumerConfig, Handler, Outcome, Stats, process_message, run_consumer};
use crate::dual_write::{db_then_kafka, kafka_then_db};
use crate::events::{EVENT_ID_HEADER, TripCompleted};
use crate::lab::{Env, PgLab, Received};
use crate::relay::{RelayConfig, Shard, drain, relay_once};
use crate::store::{
    balance_of, count_entries, count_inbox, count_outbox, enqueue, entry_from_trip,
    post_with_outbox,
};

/// SQLSTATE nếu lỗi là lỗi của PostgreSQL.
fn sqlstate(err: &sqlx::Error) -> Option<String> {
    match err {
        sqlx::Error::Database(db_err) => db_err.code().map(|c| c.into_owned()),
        _ => None,
    }
}

/// Chuyến mẫu `TN-0001` của tài xế `DR-001` (tài khoản có sẵn trong `schema.sql`).
fn trip(trip_id: &str) -> TripCompleted {
    TripCompleted::sample(trip_id, "DR-001")
}

/// Chuyến của tài xế `DR-999`, tài khoản không tồn tại: ghi sẽ vi phạm khóa ngoại.
fn trip_unknown_driver(trip_id: &str) -> TripCompleted {
    TripCompleted::sample(trip_id, "DR-999")
}

/// Số outbox id trong header `event-id` dạng `ledger:N`.
fn outbox_id(message: &Received) -> Option<i64> {
    message
        .header(EVENT_ID_HEADER)?
        .strip_prefix("ledger:")?
        .parse()
        .ok()
}

/// Gửi một thông điệp có header `event-id`, đợi broker xác nhận. Đóng vai service C# phát sự kiện.
async fn produce(
    producer: &FutureProducer,
    topic: &str,
    key: &str,
    event_id: &str,
    payload: &[u8],
) -> Result<()> {
    let headers = OwnedHeaders::new().insert(Header {
        key: EVENT_ID_HEADER,
        value: Some(event_id),
    });
    let record = FutureRecord::to(topic)
        .key(key)
        .payload(payload)
        .headers(headers);
    producer
        .send(record, Timeout::Never)
        .await
        .map_err(|(err, _)| err)?;
    Ok(())
}

/// Gửi `TripCompleted` dạng JSON.
async fn produce_trip(
    producer: &FutureProducer,
    topic: &str,
    event_id: &str,
    event: &TripCompleted,
) -> Result<()> {
    let payload = serde_json::to_vec(event)?;
    produce(producer, topic, &event.trip_id, event_id, &payload).await
}

/// Chạy lại chính file thực thi của lab ở chế độ con, đợi nó kết thúc.
async fn run_child(env: &Env, mode: &str, topic: &str) -> Result<std::process::ExitStatus> {
    let status = tokio::process::Command::new(&env.child_exe)
        .args(["child", mode, &env.pg.schema, topic])
        .status()
        .await
        .with_context(|| format!("không chạy được tiến trình con {:?}", env.child_exe))?;
    Ok(status)
}

/// Điểm vào của tiến trình con: làm đúng một việc rồi thoát ở chỗ đã định.
///
/// # Errors
/// Chế độ lạ, hoặc lỗi hạ tầng trước khi tới điểm chết.
pub async fn child_main(mode: &str, schema: &str, topic: &str) -> Result<()> {
    let url = std::env::var("TOI_NOI_PG_URL").context("thiếu TOI_NOI_PG_URL")?;
    let brokers = std::env::var("TOI_NOI_KAFKA").context("thiếu TOI_NOI_KAFKA")?;
    let pool = PgLab::attach(&url, schema).await?;
    let producer: FutureProducer = rdkafka::ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("acks", "all")
        .set("enable.idempotence", "true")
        .set("message.timeout.ms", "10000")
        .create()?;
    match mode {
        "db-then-kafka-die" => {
            let entry = entry_from_trip(&trip("TN-0001"))?;
            db_then_kafka(&pool, &producer, topic, &entry, true).await
        }
        "relay-die" => {
            let cfg = RelayConfig {
                die_after_send: true,
                ..RelayConfig::default()
            };
            relay_once(&pool, &producer, &cfg).await.map(|_| ())
        }
        other => anyhow::bail!("chế độ con lạ: {other}"),
    }
}

// ---------------------------------------------------------------------------
// 1. Dual-write: hai thứ tự, hai kiểu hỏng
// ---------------------------------------------------------------------------

/// Kết quả hai cách ghi kép sai.
#[derive(Debug)]
pub struct DualWriteReport {
    /// Mã thoát của tiến trình con chết giữa hai bước.
    pub child_exit_code: Option<i32>,
    /// Số bút toán trong DB sau khi con chết.
    pub entries_after_crash: i64,
    /// Số thông điệp trong topic sau khi con chết.
    pub kafka_after_crash: i64,
    /// SQLSTATE của lần ghi DB lỗi ở cách "Kafka trước".
    pub db_error: Option<String>,
    /// Số thông điệp trong topic "ma".
    pub kafka_ghost: i64,
    /// Số bút toán `trip:TN-0002:capture` trong DB.
    pub ghost_entries: i64,
}

/// DB trước rồi Kafka (chết giữa hai bước), và Kafka trước rồi DB (DB lỗi).
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn dual_write(env: &Env) -> Result<DualWriteReport> {
    let lost_topic = env.kafka.create_topic("db-then-kafka", 1).await?;
    let ghost_topic = env.kafka.create_topic("kafka-then-db", 1).await?;
    let producer = env.kafka.producer()?;

    // a) Tiến trình con ghi DB, commit, rồi chết trước khi gửi.
    let status = run_child(env, "db-then-kafka-die", &lost_topic).await?;
    let entries_after_crash = count_entries(&env.pg.pool).await?;
    let kafka_after_crash = env.kafka.count(&lost_topic).await?;

    // b) Gửi Kafka (đã xác nhận) rồi ghi DB; tài xế DR-999 không tồn tại nên DB từ chối.
    let entry = entry_from_trip(&trip_unknown_driver("TN-0002"))?;
    let err = kafka_then_db(&env.pg.pool, &producer, &ghost_topic, &entry)
        .await
        .expect_err("ghi DB phải lỗi");
    let db_error = err.downcast_ref::<sqlx::Error>().and_then(sqlstate);
    Ok(DualWriteReport {
        child_exit_code: status.code(),
        entries_after_crash,
        kafka_after_crash,
        db_error,
        kafka_ghost: env.kafka.count(&ghost_topic).await?,
        ghost_entries: sqlx::query_scalar(
            "SELECT count(*) FROM journal_entries WHERE idempotency_key = 'trip:TN-0002:capture'",
        )
        .fetch_one(&env.pg.pool)
        .await?,
    })
}

impl DualWriteReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị lệch kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.child_exit_code == Some(137), "con phải thoát 137");
        ensure!(self.entries_after_crash == 1, "bút toán đã commit");
        ensure!(self.kafka_after_crash == 0, "sự kiện phải mất");
        ensure!(
            self.db_error.as_deref() == Some("23503"),
            "DB phải lỗi khóa ngoại"
        );
        ensure!(self.kafka_ghost == 1, "thông điệp ma phải nằm trong topic");
        ensure!(self.ghost_entries == 0, "bút toán không được tồn tại");
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 2. Outbox cùng transaction với bút toán
// ---------------------------------------------------------------------------

/// Kết quả thí nghiệm outbox nguyên tử và relay một lượt.
#[derive(Debug)]
pub struct OutboxReport {
    /// SQLSTATE của lần ghi lỗi giữa chừng.
    pub failed_sqlstate: Option<String>,
    /// Số bút toán và số dòng outbox sau lần lỗi (phải bằng trước lần lỗi).
    pub after_failure: (i64, i64),
    /// Số bút toán sau lần ghi thành công.
    pub entries_after_commit: i64,
    /// Số dòng outbox, số dòng chưa gửi sau commit.
    pub outbox_after_commit: (i64, i64),
    /// Số thông điệp trong Kafka trước khi relay chạy.
    pub kafka_before_relay: i64,
    /// Id các dòng relay đã gửi.
    pub relayed: Vec<i64>,
    /// Số dòng outbox, số dòng chưa gửi sau relay.
    pub outbox_after_relay: (i64, i64),
    /// Thông điệp đọc lại từ Kafka.
    pub message: Received,
}

/// Ghi lỗi giữa chừng không để lại gì; ghi thành công để lại bút toán và dòng outbox; relay gửi.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn outbox_atomic(env: &Env) -> Result<OutboxReport> {
    let topic = env.kafka.create_topic("ledger-entry-posted", 1).await?;
    let producer = env.kafka.producer()?;
    let pool = &env.pg.pool;

    // Ghi thành công: bút toán và dòng outbox cùng commit.
    let good = entry_from_trip(&trip("TN-0001"))?;
    let mut tx = pool.begin().await?;
    post_with_outbox(&mut tx, &good, &topic).await?;
    tx.commit().await?;
    let entries_after_commit = count_entries(pool).await?;
    let outbox_after_commit = count_outbox(pool).await?;
    let kafka_before_relay = env.kafka.count(&topic).await?;

    // Tài xế DR-999 không tồn tại: posting thứ ba vi phạm khóa ngoại. Bút toán đã chèn trước đó
    // trong transaction cũng biến mất, và dòng outbox chưa kịp được ghi.
    let bad = entry_from_trip(&trip_unknown_driver("TN-0002"))?;
    let mut tx = pool.begin().await?;
    let failure = post_with_outbox(&mut tx, &bad, &topic)
        .await
        .expect_err("phải lỗi khóa ngoại");
    drop(tx);
    let after_failure = (count_entries(pool).await?, count_outbox(pool).await?.0);

    let relayed = relay_once(pool, &producer, &RelayConfig::default()).await?;
    let outbox_after_relay = count_outbox(pool).await?;
    let mut messages = env.kafka.read_all(&topic).await?;
    ensure!(messages.len() == 1, "topic phải có đúng 1 thông điệp");
    Ok(OutboxReport {
        failed_sqlstate: sqlstate(&failure),
        after_failure,
        entries_after_commit,
        outbox_after_commit,
        kafka_before_relay,
        relayed,
        outbox_after_relay,
        message: messages.remove(0),
    })
}

impl OutboxReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị lệch kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.failed_sqlstate.as_deref() == Some("23503"));
        ensure!(
            self.after_failure == (1, 1),
            "lỗi giữa chừng không để lại gì, số đếm giữ nguyên"
        );
        ensure!(self.entries_after_commit == 1);
        ensure!(
            self.outbox_after_commit == (1, 1),
            "ý định đã commit, chưa gửi"
        );
        ensure!(
            self.kafka_before_relay == 0,
            "chưa relay thì Kafka chưa có gì"
        );
        ensure!(self.relayed == vec![1]);
        ensure!(self.outbox_after_relay == (1, 0));
        ensure!(self.message.key.as_deref() == Some("trip:TN-0001:capture"));
        ensure!(self.message.header(EVENT_ID_HEADER) == Some("ledger:1"));
        let event: crate::events::LedgerEntryPosted = serde_json::from_str(&self.message.payload)?;
        ensure!(event.entry_id == "JE-000001");
        ensure!(event.postings.len() == 3);
        ensure!(event.postings.iter().map(|p| p.amount_dong).sum::<i64>() == 0);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 3. Relay chết sau khi gửi: gửi lại, consumer phải idempotent
// ---------------------------------------------------------------------------

/// Kết quả relay chết giữa chừng rồi gửi lại, có hoặc không có inbox.
#[derive(Debug)]
pub struct RelayCrashReport {
    /// Có dùng inbox không.
    pub use_inbox: bool,
    /// Mã thoát của relay con.
    pub child_exit_code: Option<i32>,
    /// Số thông điệp trong topic sau khi con chết.
    pub kafka_after_crash: i64,
    /// Số dòng outbox chưa đánh dấu sau khi con chết.
    pub unpublished_after_crash: i64,
    /// Số thông điệp trong topic sau khi relay chạy lại.
    pub kafka_after_retry: i64,
    /// Số `event-id` khác nhau trong topic.
    pub distinct_event_ids: usize,
    /// Outbox id theo thứ tự xuất hiện trong topic (một relay mỗi lần, topic một partition).
    pub order: Vec<i64>,
    /// Số liệu consumer hạ nguồn.
    pub stats: Stats,
    /// Số dư `driver:DR-001`.
    pub driver_balance: i64,
    /// Số dư `customer:CU-001`.
    pub customer_balance: i64,
    /// Số dư `platform:revenue`.
    pub platform_balance: i64,
}

/// Ba bút toán vào outbox; relay con chết sau khi Kafka xác nhận; relay chạy lại; consumer dựng số dư.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn relay_crash(env: &Env, use_inbox: bool) -> Result<RelayCrashReport> {
    let topic = env.kafka.create_topic("ledger-entry-posted", 1).await?;
    let dlq = env.kafka.create_topic("dlq", 1).await?;
    let producer = env.kafka.producer()?;
    let pool = &env.pg.pool;

    for n in 1..=3 {
        let entry = entry_from_trip(&trip(&format!("TN-{n:04}")))?;
        let mut tx = pool.begin().await?;
        post_with_outbox(&mut tx, &entry, &topic).await?;
        tx.commit().await?;
    }

    let status = run_child(env, "relay-die", &topic).await?;
    let kafka_after_crash = env.kafka.count(&topic).await?;
    let unpublished_after_crash = count_outbox(pool).await?.1;

    // Tiến trình con đã thoát, socket của nó đóng, PostgreSQL rollback và nhả khóa hàng. Backend có
    // thể chưa kịp làm xong khi ta chạy tới đây, và SKIP LOCKED sẽ im lặng bỏ qua hàng còn khóa,
    // nên thử lại có hạn chót thay vì coi lần rỗng đầu tiên là hết việc.
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut resent = drain(pool, &producer, &RelayConfig::default()).await?;
    while resent < 3 && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(50)).await;
        resent += drain(pool, &producer, &RelayConfig::default()).await?;
    }
    ensure!(
        resent == 3,
        "relay chạy lại phải gửi 3 dòng, thực tế {resent}"
    );
    let messages = env.kafka.read_all(&topic).await?;
    let distinct: HashSet<&str> = messages
        .iter()
        .filter_map(|m| m.header(EVENT_ID_HEADER))
        .collect();

    let cfg = ConsumerConfig {
        group: env.kafka.group(""),
        topic: topic.clone(),
        dlq_topic: dlq,
        handler: Handler::EntryPosted,
        use_inbox,
        max_attempts: 3,
        retry_delay: Duration::from_millis(50),
        stop_after: messages.len(),
        commit: true,
        idle_timeout: Duration::from_secs(20),
    };
    let stats = run_consumer(pool, &env.kafka.brokers, &producer, &cfg).await?;
    Ok(RelayCrashReport {
        use_inbox,
        child_exit_code: status.code(),
        kafka_after_crash,
        unpublished_after_crash,
        kafka_after_retry: messages.len() as i64,
        distinct_event_ids: distinct.len(),
        order: messages.iter().filter_map(outbox_id).collect(),
        stats,
        driver_balance: balance_of(pool, "driver:DR-001").await?,
        customer_balance: balance_of(pool, "customer:CU-001").await?,
        platform_balance: balance_of(pool, "platform:revenue").await?,
    })
}

impl RelayCrashReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị lệch kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.child_exit_code == Some(137));
        ensure!(self.kafka_after_crash == 3, "con đã gửi 3 trước khi chết");
        ensure!(self.unpublished_after_crash == 3, "con chưa kịp đánh dấu");
        ensure!(self.kafka_after_retry == 6, "gửi lại nên mỗi dòng có 2 bản");
        ensure!(self.distinct_event_ids == 3);
        ensure!(
            self.order == vec![1, 2, 3, 1, 2, 3],
            "một relay giữ thứ tự id"
        );
        if self.use_inbox {
            ensure!(self.stats.applied == 3 && self.stats.duplicates == 3);
            ensure!(self.driver_balance == 3 * 20_400);
            ensure!(self.customer_balance == -3 * 25_500);
            ensure!(self.platform_balance == 3 * 5_100);
        } else {
            ensure!(self.stats.applied == 6 && self.stats.duplicates == 0);
            ensure!(
                self.driver_balance == 6 * 20_400,
                "thiếu inbox: số dư bị cộng đôi"
            );
            ensure!(self.customer_balance == -6 * 25_500);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Hai relay song song
// ---------------------------------------------------------------------------

/// Kết quả hai relay chạy song song.
#[derive(Debug)]
pub struct ParallelReport {
    /// Có `SKIP LOCKED` không.
    pub lock: bool,
    /// Id relay A đã nhận.
    pub a_ids: Vec<i64>,
    /// Id relay B đã nhận.
    pub b_ids: Vec<i64>,
    /// Số thông điệp trong topic.
    pub messages: usize,
    /// Số `event-id` khác nhau.
    pub distinct: usize,
    /// Với `lock = true`: (tổng dòng hai worker gửi, thông điệp trong topic, event-id khác nhau)
    /// khi xả 100 dòng bằng hai worker chạy vòng lặp.
    pub drain: Option<(usize, usize, usize)>,
}

/// Thêm `n` dòng outbox thô (không qua bút toán) vào `topic`.
async fn enqueue_raw(env: &Env, topic: &str, n: usize, keys: &[&str]) -> Result<()> {
    let mut conn = env.pg.pool.acquire().await?;
    for i in 0..n {
        let key = keys[i % keys.len()];
        enqueue(&mut conn, topic, key, &format!("{{\"n\":{i}}}")).await?;
    }
    Ok(())
}

/// Lịch cố định cho hai relay: A nhận dòng rồi đứng lại, B nhận và gửi xong, A mới đi tiếp.
/// Dùng tín hiệu thay vì đợi theo giờ nên kết quả không phụ thuộc tải máy.
async fn a_claims_then_b_overtakes(
    pool: &sqlx::PgPool,
    producer: &FutureProducer,
    a_cfg: RelayConfig,
    b_cfg: &RelayConfig,
) -> (Result<Vec<i64>>, Result<Vec<i64>>) {
    let (claimed, resume) = (Arc::new(Notify::new()), Arc::new(Notify::new()));
    let a_cfg = RelayConfig {
        claimed: Some(claimed.clone()),
        resume: Some(resume.clone()),
        ..a_cfg
    };
    let worker_a = async {
        let result = relay_once(pool, producer, &a_cfg).await;
        claimed.notify_one(); // A lỗi trước khi nhận dòng thì B vẫn chạy, không treo
        result
    };
    let worker_b = async {
        claimed.notified().await;
        let result = relay_once(pool, producer, b_cfg).await;
        resume.notify_one();
        result
    };
    tokio::join!(worker_a, worker_b)
}

/// Hai worker, mỗi worker một lượt 5 dòng trên 10 dòng, theo lịch của [`a_claims_then_b_overtakes`].
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn parallel_relays(env: &Env, lock: bool) -> Result<ParallelReport> {
    let topic = env.kafka.create_topic("parallel", 1).await?;
    let producer = env.kafka.producer()?;
    let pool = &env.pg.pool;
    enqueue_raw(env, &topic, 10, &["k0", "k1"]).await?;

    let cfg = RelayConfig {
        batch: 5,
        lock,
        ..RelayConfig::default()
    };
    let (a, b) = a_claims_then_b_overtakes(pool, &producer, cfg.clone(), &cfg).await;
    let (a_ids, b_ids) = (a?, b?);
    let messages = env.kafka.read_all(&topic).await?;
    let distinct: HashSet<&str> = messages
        .iter()
        .filter_map(|m| m.header(EVENT_ID_HEADER))
        .collect();
    let (messages_len, distinct_len) = (messages.len(), distinct.len());

    let drain_result = if lock {
        // Xả thêm 100 dòng bằng hai worker chạy vòng lặp, không dừng giả tạo.
        enqueue_raw(env, &topic, 100, &["k0", "k1", "k2", "k3"]).await?;
        let cfg_a = RelayConfig {
            batch: 7,
            ..RelayConfig::default()
        };
        let cfg_b = RelayConfig {
            batch: 9,
            ..RelayConfig::default()
        };
        let (x, y) = tokio::join!(
            drain(pool, &producer, &cfg_a),
            drain(pool, &producer, &cfg_b)
        );
        let sent = x? + y?;
        let all = env.kafka.read_all(&topic).await?;
        let ids: HashSet<&str> = all
            .iter()
            .filter_map(|m| m.header(EVENT_ID_HEADER))
            .collect();
        // Trừ 10 thông điệp của thí nghiệm đầu: còn lại là của lượt xả.
        Some((sent, all.len() - messages_len, ids.len() - distinct_len))
    } else {
        None
    };
    Ok(ParallelReport {
        lock,
        a_ids,
        b_ids,
        messages: messages_len,
        distinct: distinct_len,
        drain: drain_result,
    })
}

impl ParallelReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị lệch kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.messages == 10);
        ensure!(self.a_ids == vec![1, 2, 3, 4, 5], "A nhận 1..5");
        if self.lock {
            ensure!(
                self.b_ids == vec![6, 7, 8, 9, 10],
                "B bỏ qua dòng bị khóa, nhận 6..10"
            );
            ensure!(self.distinct == 10, "không có thông điệp trùng");
            ensure!(
                self.drain == Some((100, 100, 100)),
                "xả 100 dòng: không trùng"
            );
        } else {
            ensure!(
                self.a_ids == self.b_ids,
                "không khóa: hai worker nhận cùng 5 dòng"
            );
            ensure!(
                self.distinct == 5,
                "10 thông điệp nhưng chỉ 5 sự kiện khác nhau"
            );
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Thứ tự khi nhiều worker
// ---------------------------------------------------------------------------

/// Thứ tự trong topic khi hai worker cùng relay.
#[derive(Debug)]
pub struct OrderingReport {
    /// Có chia theo key không.
    pub sharded: bool,
    /// Hai key dùng trong thí nghiệm.
    pub keys: (String, String),
    /// Outbox id theo thứ tự xuất hiện trong topic (1 partition).
    pub order: Vec<i64>,
    /// Mỗi key đều đúng thứ tự tăng dần?
    pub per_key_ordered: bool,
    /// Toàn topic đúng thứ tự tăng dần?
    pub globally_ordered: bool,
}

/// Phần chia (0 hoặc 1) của key theo cùng công thức md5 mà relay dùng.
async fn shard_of(env: &Env, key: &str) -> Result<i64> {
    Ok(
        sqlx::query_scalar("SELECT ((('x' || substr(md5($1), 1, 7))::bit(28)::int) % 2)::bigint")
            .bind(key)
            .fetch_one(&env.pg.pool)
            .await?,
    )
}

/// Sáu dòng outbox xen kẽ hai key, hai worker mỗi worker 3 dòng; A nhận trước, B gửi xong A mới gửi.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn ordering(env: &Env, sharded: bool) -> Result<OrderingReport> {
    let topic = env.kafka.create_topic("ordering", 1).await?;
    let producer = env.kafka.producer()?;
    let pool = &env.pg.pool;

    // Chọn hai key rơi vào hai phần khác nhau để thí nghiệm chia theo key có việc cho cả hai worker.
    let (mut key0, mut key1) = (None, None);
    for n in 1..100 {
        let key = format!("trip:TN-{n:04}:capture");
        match shard_of(env, &key).await? {
            0 if key0.is_none() => key0 = Some(key),
            1 if key1.is_none() => key1 = Some(key),
            _ => {}
        }
        if key0.is_some() && key1.is_some() {
            break;
        }
    }
    let (key0, key1) = (
        key0.context("không có key phần 0")?,
        key1.context("không có key phần 1")?,
    );
    enqueue_raw(env, &topic, 6, &[&key0, &key1]).await?;

    let a_cfg = RelayConfig {
        batch: 3,
        shard: sharded.then_some(Shard { index: 0, count: 2 }),
        ..RelayConfig::default()
    };
    let b_cfg = RelayConfig {
        batch: 3,
        shard: sharded.then_some(Shard { index: 1, count: 2 }),
        ..RelayConfig::default()
    };
    let (a, b) = a_claims_then_b_overtakes(pool, &producer, a_cfg, &b_cfg).await;
    a?;
    b?;

    let messages = env.kafka.read_all(&topic).await?;
    let order: Vec<i64> = messages.iter().filter_map(outbox_id).collect();
    let ascending = |ids: &[i64]| ids.windows(2).all(|w| w[0] < w[1]);
    let of_key = |key: &str| -> Vec<i64> {
        messages
            .iter()
            .filter(|m| m.key.as_deref() == Some(key))
            .filter_map(outbox_id)
            .collect()
    };
    Ok(OrderingReport {
        sharded,
        per_key_ordered: ascending(&of_key(&key0)) && ascending(&of_key(&key1)),
        globally_ordered: ascending(&order),
        keys: (key0, key1),
        order,
    })
}

impl OrderingReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị lệch kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.order.len() == 6);
        if self.sharded {
            ensure!(self.per_key_ordered, "chia theo key: mỗi key giữ thứ tự");
            ensure!(
                !self.globally_ordered,
                "nhưng thứ tự giữa các key không được bảo đảm"
            );
            ensure!(
                self.order == vec![2, 4, 6, 1, 3, 5],
                "batch của B (key phần 1) tới trước batch của A"
            );
        } else {
            ensure!(
                self.order == vec![4, 5, 6, 1, 2, 3],
                "batch của B tới trước batch của A"
            );
            ensure!(!self.per_key_ordered, "ngay cả trong một key cũng bị đảo");
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Inbox: trùng event_id, trùng nghiệp vụ, và đua đồng thời
// ---------------------------------------------------------------------------

/// Kết quả consumer `TripCompleted` nhận ba thông điệp.
#[derive(Debug)]
pub struct InboxReport {
    /// Số liệu consumer.
    pub stats: Stats,
    /// Số bút toán.
    pub entries: i64,
    /// Số dòng outbox, số dòng chưa gửi.
    pub outbox: (i64, i64),
    /// Số dòng inbox.
    pub inbox: i64,
}

/// A, A (cùng `event_id`), B (`event_id` khác, cùng chuyến) vào cùng một partition.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn inbox_dedupe(env: &Env) -> Result<InboxReport> {
    let input = env.kafka.create_topic("trip-completed", 1).await?;
    let out = env.kafka.create_topic("ledger-entry-posted", 1).await?;
    let dlq = env.kafka.create_topic("dlq", 1).await?;
    let producer = env.kafka.producer()?;
    let event = trip("TN-0001");
    produce_trip(&producer, &input, "trip-svc:A", &event).await?;
    produce_trip(&producer, &input, "trip-svc:A", &event).await?;
    produce_trip(&producer, &input, "trip-svc:B", &event).await?;

    let cfg = ConsumerConfig {
        group: env.kafka.group(""),
        topic: input,
        dlq_topic: dlq,
        handler: Handler::TripCompleted { out_topic: out },
        use_inbox: true,
        max_attempts: 3,
        retry_delay: Duration::from_millis(50),
        stop_after: 3,
        commit: true,
        idle_timeout: Duration::from_secs(20),
    };
    let stats = run_consumer(&env.pg.pool, &env.kafka.brokers, &producer, &cfg).await?;
    Ok(InboxReport {
        stats,
        entries: count_entries(&env.pg.pool).await?,
        outbox: count_outbox(&env.pg.pool).await?,
        inbox: count_inbox(&env.pg.pool).await?,
    })
}

impl InboxReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị lệch kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.stats.applied == 1, "chỉ thông điệp đầu tạo hiệu ứng");
        ensure!(self.stats.duplicates == 1, "inbox chặn bản trùng event_id");
        ensure!(
            self.stats.entry_replays == 1,
            "khóa nghiệp vụ chặn event_id mới của cùng chuyến"
        );
        ensure!(self.stats.dead_lettered == 0);
        ensure!(self.entries == 1);
        ensure!(self.outbox == (1, 1), "một bút toán, một dòng outbox");
        ensure!(self.inbox == 2, "inbox ghi cả A và B");
        Ok(())
    }
}

/// Kết quả tám task xử lý cùng một `event_id` đồng thời.
#[derive(Debug)]
pub struct InboxRaceReport {
    /// Số task tạo hiệu ứng.
    pub applied: usize,
    /// Số task bị inbox chặn.
    pub duplicates: usize,
    /// Số bút toán.
    pub entries: i64,
    /// Số dòng inbox.
    pub inbox: i64,
}

/// Tám task cùng gọi [`process_message`] với cùng `event_id`.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn inbox_race(env: &Env) -> Result<InboxRaceReport> {
    let payload = Arc::new(serde_json::to_vec(&trip("TN-0001"))?);
    let handler = Arc::new(Handler::TripCompleted {
        out_topic: "ledger.entry-posted".to_string(),
    });
    let mut tasks = Vec::new();
    for _ in 0..8 {
        let (pool, payload, handler) = (env.pg.pool.clone(), payload.clone(), handler.clone());
        tasks.push(tokio::spawn(async move {
            process_message(&pool, &handler, true, "trip-svc:A", &payload).await
        }));
    }
    let (mut applied, mut duplicates) = (0, 0);
    for task in tasks {
        match task.await?? {
            Outcome::Applied => applied += 1,
            Outcome::Duplicate => duplicates += 1,
            Outcome::EntryAlreadyExists => {}
        }
    }
    Ok(InboxRaceReport {
        applied,
        duplicates,
        entries: count_entries(&env.pg.pool).await?,
        inbox: count_inbox(&env.pg.pool).await?,
    })
}

impl InboxRaceReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị lệch kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.applied == 1 && self.duplicates == 7);
        ensure!(self.entries == 1 && self.inbox == 1);
        Ok(())
    }
}

/// Kết quả consumer chết trước khi commit offset rồi chạy lại.
#[derive(Debug)]
pub struct RedeliveryReport {
    /// Lần chạy đầu (không commit).
    pub first: Stats,
    /// Offset đã commit sau lần đầu (-1: chưa).
    pub committed_after_first: i64,
    /// Lần chạy hai (cùng group).
    pub second: Stats,
    /// Offset đã commit sau lần hai.
    pub committed_after_second: i64,
    /// Số bút toán.
    pub entries: i64,
}

/// Lần đầu xử lý xong nhưng không commit offset; lần hai cùng group nhận lại đúng thông điệp đó.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn redelivery(env: &Env) -> Result<RedeliveryReport> {
    let input = env.kafka.create_topic("trip-completed", 1).await?;
    let out = env.kafka.create_topic("ledger-entry-posted", 1).await?;
    let dlq = env.kafka.create_topic("dlq", 1).await?;
    let producer = env.kafka.producer()?;
    produce_trip(&producer, &input, "trip-svc:A", &trip("TN-0001")).await?;

    let group = env.kafka.group("");
    let mut cfg = ConsumerConfig {
        group: group.clone(),
        topic: input.clone(),
        dlq_topic: dlq,
        handler: Handler::TripCompleted { out_topic: out },
        use_inbox: true,
        max_attempts: 3,
        retry_delay: Duration::from_millis(50),
        stop_after: 1,
        commit: false,
        idle_timeout: Duration::from_secs(20),
    };
    let first = run_consumer(&env.pg.pool, &env.kafka.brokers, &producer, &cfg).await?;
    let committed_after_first = env.kafka.committed_offset(&group, &input).await?;
    cfg.commit = true;
    let second = run_consumer(&env.pg.pool, &env.kafka.brokers, &producer, &cfg).await?;
    Ok(RedeliveryReport {
        first,
        committed_after_first,
        second,
        committed_after_second: env.kafka.committed_offset(&group, &input).await?,
        entries: count_entries(&env.pg.pool).await?,
    })
}

impl RedeliveryReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị lệch kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.first.applied == 1);
        ensure!(self.committed_after_first == -1, "chưa commit offset");
        ensure!(
            self.second.applied == 0 && self.second.duplicates == 1,
            "nhận lại, inbox chặn"
        );
        ensure!(
            self.committed_after_second == 1,
            "lần hai commit qua thông điệp"
        );
        ensure!(self.entries == 1);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 7. DLQ
// ---------------------------------------------------------------------------

/// Kết quả consumer gặp hai thông điệp hỏng giữa ba thông điệp tốt.
#[derive(Debug)]
pub struct DlqReport {
    /// Số liệu consumer.
    pub stats: Stats,
    /// Số bút toán.
    pub entries: i64,
    /// Số dòng inbox (chỉ thông điệp thành công).
    pub inbox: i64,
    /// Offset đã commit của topic nguồn.
    pub committed: i64,
    /// Thông điệp trong DLQ.
    pub dlq: Vec<Received>,
}

/// tốt, JSON hỏng, tốt, tài xế không tồn tại, tốt: cùng một partition, `max_attempts = 3`.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn dlq(env: &Env) -> Result<DlqReport> {
    let input = env.kafka.create_topic("trip-completed", 1).await?;
    let out = env.kafka.create_topic("ledger-entry-posted", 1).await?;
    let dlq_topic = env.kafka.create_topic("dlq", 1).await?;
    let producer = env.kafka.producer()?;

    produce_trip(&producer, &input, "trip-svc:1", &trip("TN-0001")).await?;
    produce(
        &producer,
        &input,
        "TN-0002",
        "trip-svc:2",
        b"{khong phai json",
    )
    .await?;
    produce_trip(&producer, &input, "trip-svc:3", &trip("TN-0003")).await?;
    produce_trip(
        &producer,
        &input,
        "trip-svc:4",
        &trip_unknown_driver("TN-0004"),
    )
    .await?;
    produce_trip(&producer, &input, "trip-svc:5", &trip("TN-0005")).await?;

    let group = env.kafka.group("");
    let cfg = ConsumerConfig {
        group: group.clone(),
        topic: input.clone(),
        dlq_topic: dlq_topic.clone(),
        handler: Handler::TripCompleted { out_topic: out },
        use_inbox: true,
        max_attempts: 3,
        retry_delay: Duration::from_millis(50),
        stop_after: 5,
        commit: true,
        idle_timeout: Duration::from_secs(20),
    };
    let stats = run_consumer(&env.pg.pool, &env.kafka.brokers, &producer, &cfg).await?;
    Ok(DlqReport {
        stats,
        entries: count_entries(&env.pg.pool).await?,
        inbox: count_inbox(&env.pg.pool).await?,
        committed: env.kafka.committed_offset(&group, &input).await?,
        dlq: env.kafka.read_all(&dlq_topic).await?,
    })
}

impl DlqReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị lệch kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.stats.applied == 3 && self.stats.dead_lettered == 2);
        ensure!(
            self.stats.attempts == 9,
            "3 thông điệp tốt × 1 lần + 2 hỏng × 3 lần"
        );
        ensure!(
            self.entries == 3 && self.inbox == 3,
            "thông điệp hỏng không để lại dòng inbox"
        );
        ensure!(
            self.committed == 5,
            "offset đi qua cả hai thông điệp hỏng: partition không kẹt"
        );
        ensure!(self.dlq.len() == 2);
        ensure!(self.dlq.iter().all(|m| m.header("x-attempts") == Some("3")));
        ensure!(
            self.dlq[0].payload == "{khong phai json",
            "DLQ giữ nguyên byte gốc"
        );
        ensure!(
            self.dlq[0]
                .header("x-source")
                .is_some_and(|s| s.ends_with("/0/1"))
        );
        ensure!(
            self.dlq[1]
                .header("x-error")
                .is_some_and(|e| e.contains("foreign key"))
        );
        Ok(())
    }
}
