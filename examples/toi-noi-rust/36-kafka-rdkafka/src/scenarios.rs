//! Sáu kịch bản chạy trên Kafka thật. Mỗi kịch bản trả một báo cáo; `main` in, `checks` kiểm.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use ledger_core::fare_for_km;
use rdkafka::consumer::{
    BaseConsumer, CommitMode, Consumer, ConsumerContext, Rebalance, StreamConsumer,
};
use rdkafka::{ClientContext, Message, Offset, TopicPartitionList};
use tokio::task::JoinHandle;

use crate::events::{LedgerEntryPosted, SCHEMA_VERSION, TripCompleted};
use crate::kafka::{Lab, LabError, QUIET, WAIT, committed_offset, recv_within, send};
use crate::service::{Outcome, Processor, Summary, Worker};

/// Mã chuyến `TN-000n`.
pub fn trip_key(n: u32) -> String {
    format!("TN-{n:04}")
}

/// Payload `TripCompleted` hợp lệ cho chuyến `n` đi `km` km (khách `CU-001`, tài xế `DR-001`).
pub fn trip_json(n: u32, km: u32) -> Vec<u8> {
    let event = TripCompleted {
        schema_version: SCHEMA_VERSION,
        trip_id: trip_key(n),
        customer_id: "CU-001".to_string(),
        driver_id: "DR-001".to_string(),
        fare_dong: fare_for_km(km).dong(),
        occurred_at: "2026-11-06T02:30:00Z".parse().expect("hằng số hợp lệ"),
    };
    serde_json::to_vec(&event).expect("TripCompleted luôn serialize được")
}

/// Một thông điệp đã xử lý: nằm ở đâu và kết quả ra sao.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub partition: i32,
    pub offset: i64,
    pub summary: Summary,
}

fn step_of(partition: i32, offset: i64, outcome: &Outcome) -> Step {
    Step {
        partition,
        offset,
        summary: outcome.summary(),
    }
}

fn parts(tpl: &TopicPartitionList) -> Vec<i32> {
    let mut v: Vec<i32> = tpl.elements().iter().map(|e| e.partition()).collect();
    v.sort_unstable();
    v
}

/// `true` nếu trong `quiet` (tính từ lúc consumer đã được gán partition) không có thông điệp nào đến.
/// Chờ gán trước, vì hết giờ khi chưa được gán partition không chứng minh được gì.
pub async fn stays_quiet<C: ConsumerContext + 'static>(
    consumer: &StreamConsumer<C>,
    quiet: Duration,
) -> Result<bool, LabError> {
    let start = Instant::now();
    let mut assigned_at: Option<Instant> = None;
    loop {
        if assigned_at.is_none() && !parts(&consumer.assignment()?).is_empty() {
            assigned_at = Some(Instant::now());
        }
        match assigned_at {
            Some(t) if t.elapsed() >= quiet => return Ok(true),
            None if start.elapsed() >= WAIT => {
                return Err(LabError::Timeout {
                    what: "gán partition cho consumer".to_string(),
                    wait: WAIT,
                });
            }
            _ => {}
        }
        match tokio::time::timeout(Duration::from_millis(200), consumer.recv()).await {
            Err(_) => {}
            Ok(Ok(_)) => return Ok(false),
            Ok(Err(e)) => return Err(e.into()),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Kịch bản 1: at-least-once. Xử lý xong, chưa commit, khởi động lại: nhận lại, bút toán vẫn một.
// ---------------------------------------------------------------------------------------------

pub struct AtLeastOnceReport {
    pub delivered: (i32, i64),
    pub run1: Step,
    pub committed_before_run2: Option<i64>,
    pub run2: Step,
    pub committed_after_run2: Option<i64>,
    pub redelivered_after_commit: bool,
    pub ledger_entries: usize,
    pub driver_balance: i64,
    pub output: Vec<LedgerEntryPosted>,
}

pub async fn at_least_once(lab: &Lab) -> Result<AtLeastOnceReport> {
    let input = lab.create_topic("trip-completed", 3).await?;
    let output = lab.create_topic("ledger-entry-posted", 3).await?;
    let producer = lab.producer()?;
    let delivered = send(&producer, &input, "TN-0001", &trip_json(1, 5)).await?;
    // Sổ cái sống qua các lần khởi động lại, như PostgreSQL.
    let worker = Worker::new(Arc::new(Processor::new()), producer, output.clone());
    let group = lab.group_name(None);

    // Lần chạy 1: ghi bút toán, phát sự kiện, rồi «chết» trước khi commit (consumer bị drop).
    let run1 = {
        let consumer = lab.consumer(&group)?;
        consumer.subscribe(&[&input])?;
        let msg = recv_within(&consumer, WAIT, "TripCompleted ở lần chạy 1").await?;
        let outcome = worker.handle(msg.payload()).await?;
        step_of(msg.partition(), msg.offset(), &outcome)
    };

    // Lần chạy 2: group chưa có offset nào, nên cùng thông điệp đến lần nữa. Lần này commit sau khi ghi.
    let (committed_before_run2, run2, committed_after_run2) = {
        let consumer = lab.consumer(&group)?;
        let before = committed_offset(&consumer, &input, delivered.0)?;
        consumer.subscribe(&[&input])?;
        let msg = recv_within(&consumer, WAIT, "TripCompleted ở lần chạy 2").await?;
        let outcome = worker.handle(msg.payload()).await?;
        consumer.commit_message(&msg, CommitMode::Sync)?;
        let step = step_of(msg.partition(), msg.offset(), &outcome);
        (
            before,
            step,
            committed_offset(&consumer, &input, delivered.0)?,
        )
    };

    // Lần chạy 3: đã commit, nên không còn gì để nhận.
    let redelivered_after_commit = {
        let consumer = lab.consumer(&group)?;
        consumer.subscribe(&[&input])?;
        !stays_quiet(&consumer, QUIET).await?
    };

    // Đọc topic đầu ra: sự kiện bị phát hai lần dù sổ cái chỉ có một bút toán.
    let consumer = lab.consumer(&lab.group_name(Some("out")))?;
    consumer.subscribe(&[&output])?;
    let mut events = Vec::new();
    for _ in 0..2 {
        let msg = recv_within(&consumer, WAIT, "LedgerEntryPosted").await?;
        events.push(serde_json::from_slice(msg.payload().unwrap_or_default())?);
    }
    if !stays_quiet(&consumer, QUIET).await? {
        return Err(anyhow!("topic đầu ra có nhiều hơn 2 sự kiện"));
    }

    Ok(AtLeastOnceReport {
        delivered,
        run1,
        committed_before_run2,
        run2,
        committed_after_run2,
        redelivered_after_commit,
        ledger_entries: worker.processor.entry_count(),
        driver_balance: worker.processor.balance("driver:DR-001")?,
        output: events,
    })
}

// ---------------------------------------------------------------------------------------------
// Kịch bản 2: thứ tự theo partition và chọn key.
// ---------------------------------------------------------------------------------------------

pub struct KeysReport {
    pub partitions: i32,
    pub by_trip_key: BTreeMap<i32, usize>,
    pub by_platform_key: BTreeMap<i32, usize>,
    pub account_partitions: Vec<(&'static str, i32)>,
    pub same_key: Vec<(i32, i64)>,
    pub consumed_n: Vec<u64>,
    /// `(key, partition theo consistent_random, partition theo murmur2_random)`.
    pub hash_compare: Vec<(String, i32, i32)>,
}

fn stream_payload(stream: &str, n: u64) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({ "stream": stream, "n": n })).expect("json hợp lệ")
}

pub async fn keys_and_order(lab: &Lab) -> Result<KeysReport> {
    const PARTITIONS: i32 = 6;
    let topic = lab.create_topic("keys", PARTITIONS).await?;
    let producer = lab.producer()?;
    let mut total = 0_usize;

    // A: 12 bút toán, key = tripId. B: 12 bút toán, key = tài khoản nền tảng (mọi bút toán đều chạm nó).
    let mut by_trip_key = BTreeMap::new();
    let mut by_platform_key = BTreeMap::new();
    for n in 1..=12_u32 {
        let (p, _) = send(
            &producer,
            &topic,
            &trip_key(n),
            &stream_payload("A", n.into()),
        )
        .await?;
        *by_trip_key.entry(p).or_insert(0) += 1;
        let (p, _) = send(
            &producer,
            &topic,
            "platform:revenue",
            &stream_payload("B", n.into()),
        )
        .await?;
        *by_platform_key.entry(p).or_insert(0) += 1;
        total += 2;
    }

    // Ba tài khoản của MỘT bút toán `trip:TN-0001:capture`: nếu key là tài khoản thì chúng đi ba nơi.
    let mut account_partitions = Vec::new();
    for account in ["customer:CU-001", "driver:DR-001", "platform:revenue"] {
        let (p, _) = send(&producer, &topic, account, &stream_payload("D", 0)).await?;
        account_partitions.push((account, p));
        total += 1;
    }

    // C: năm thông điệp cùng key TN-0001, gửi lần lượt.
    let mut same_key = Vec::new();
    for n in 1..=5_u64 {
        same_key.push(send(&producer, &topic, "TN-0001", &stream_payload("C", n)).await?);
        total += 1;
    }

    // Cùng key, hai thuật toán băm: CRC32 (mặc định của librdkafka) và murmur2 (tương thích producer Java).
    let murmur = lab.producer_with_partitioner("murmur2_random")?;
    let mut hash_compare = Vec::new();
    for n in 1..=3_u32 {
        let key = trip_key(n);
        let (crc, _) = send(&producer, &topic, &key, &stream_payload("H", 0)).await?;
        let (mur, _) = send(&murmur, &topic, &key, &stream_payload("H", 0)).await?;
        hash_compare.push((key, crc, mur));
        total += 2;
    }

    // Đọc lại cả topic; nhóm C phải đến đúng thứ tự gửi vì cùng partition.
    let consumer = lab.consumer(&lab.group_name(None))?;
    consumer.subscribe(&[&topic])?;
    let mut consumed_n = Vec::new();
    for _ in 0..total {
        let msg = recv_within(&consumer, WAIT, "thông điệp của topic keys").await?;
        let value: serde_json::Value = serde_json::from_slice(msg.payload().unwrap_or_default())?;
        if value["stream"] == "C" {
            consumed_n.push(value["n"].as_u64().unwrap_or(0));
        }
    }
    Ok(KeysReport {
        partitions: PARTITIONS,
        by_trip_key,
        by_platform_key,
        account_partitions,
        same_key,
        consumed_n,
        hash_compare,
    })
}

// ---------------------------------------------------------------------------------------------
// Kịch bản 3: consumer group và rebalance.
// ---------------------------------------------------------------------------------------------

/// Ghi lại mọi lần rebalance của một consumer (callback chạy trong luồng poll của librdkafka).
#[derive(Clone, Default)]
pub struct RebalanceLog {
    events: Arc<Mutex<Vec<String>>>,
}

impl ClientContext for RebalanceLog {}

impl ConsumerContext for RebalanceLog {
    fn post_rebalance(&self, _base: &BaseConsumer<Self>, rebalance: &Rebalance<'_>) {
        let text = match rebalance {
            Rebalance::Assign(tpl) => format!("gán {:?}", parts(tpl)),
            Rebalance::Revoke(tpl) => format!("thu hồi {:?}", parts(tpl)),
            Rebalance::Error(err) => format!("lỗi {err}"),
        };
        self.events.lock().unwrap().push(text);
    }
}

#[derive(Debug, Default)]
pub struct MemberStats {
    pub processed: usize,
    pub partitions: BTreeSet<i32>,
}

/// Một consumer trong group, chạy vòng nhận-ghi-commit trong task riêng.
struct Member {
    consumer: Arc<StreamConsumer<RebalanceLog>>,
    log: RebalanceLog,
    stop: Arc<AtomicBool>,
    join: JoinHandle<Result<MemberStats>>,
}

impl Member {
    fn start(lab: &Lab, group: &str, topic: &str, worker: Worker) -> Result<Member> {
        let log = RebalanceLog::default();
        let consumer = Arc::new(lab.consumer_with(group, log.clone())?);
        consumer.subscribe(&[topic])?;
        let stop = Arc::new(AtomicBool::new(false));
        let join = tokio::spawn({
            let (consumer, stop) = (consumer.clone(), stop.clone());
            async move {
                let mut stats = MemberStats::default();
                while !stop.load(Ordering::Relaxed) {
                    // Nhận có hạn để thấy cờ dừng; cũng chính lời gọi này phục vụ callback rebalance.
                    match tokio::time::timeout(Duration::from_millis(100), consumer.recv()).await {
                        Err(_) => {}
                        // Phần lớn lỗi của `recv()` là thông báo, librdkafka tự phục hồi: log rồi đọc tiếp.
                        // Nếu lỗi không qua, `wait_until` hết giờ và kịch bản fail rõ.
                        Ok(Err(e)) => eprintln!("consumer: {e}"),
                        Ok(Ok(msg)) => {
                            worker.handle(msg.payload()).await?;
                            consumer.commit_message(&msg, CommitMode::Sync)?;
                            stats.processed += 1;
                            stats.partitions.insert(msg.partition());
                        }
                    }
                }
                Ok(stats)
            }
        });
        Ok(Member {
            consumer,
            log,
            stop,
            join,
        })
    }

    fn assignment(&self) -> Result<Vec<i32>> {
        // Vòng đọc đã dừng vì lỗi thì báo ngay, không chờ hết giờ rồi đổ cho rebalance.
        if self.join.is_finished() {
            return Err(anyhow!("vòng đọc của consumer đã dừng"));
        }
        Ok(parts(&self.consumer.assignment()?))
    }

    fn events(&self) -> Vec<String> {
        self.log.events.lock().unwrap().clone()
    }

    /// Dừng vòng lặp, lấy thống kê; consumer bị drop và rời group.
    async fn stop(self) -> Result<MemberStats> {
        self.stop.store(true, Ordering::Relaxed);
        self.join.await?
    }
}

async fn wait_until(what: &str, mut condition: impl FnMut() -> Result<bool>) -> Result<()> {
    let start = Instant::now();
    while !condition()? {
        if start.elapsed() >= WAIT {
            return Err(LabError::Timeout {
                what: what.to_string(),
                wait: WAIT,
            }
            .into());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Ok(())
}

pub struct GroupReport {
    pub partitions: usize,
    pub alone: Vec<i32>,
    pub first_after_join: Vec<i32>,
    pub second_after_join: Vec<i32>,
    pub processed: (usize, usize),
    pub partitions_seen: (BTreeSet<i32>, BTreeSet<i32>),
    pub ledger_entries: usize,
    pub second_after_leave: Vec<i32>,
    pub events: (Vec<String>, Vec<String>),
}

pub async fn group_rebalance(lab: &Lab) -> Result<GroupReport> {
    const PARTITIONS: usize = 4;
    let input = lab
        .create_topic("trip-completed", PARTITIONS as i32)
        .await?;
    let output = lab.create_topic("ledger-entry-posted", 1).await?;
    let producer = lab.producer()?;
    let worker = Worker::new(Arc::new(Processor::new()), producer.clone(), output);
    let group = lab.group_name(None);

    let first = Member::start(lab, &group, &input, worker.clone())?;
    wait_until("consumer 1 được gán đủ partition", || {
        Ok(first.assignment()?.len() == PARTITIONS)
    })
    .await?;
    let alone = first.assignment()?;

    // Consumer 2 vào cùng group: partition được chia lại.
    let second = Member::start(lab, &group, &input, worker.clone())?;
    wait_until("hai consumer chia xong partition", || {
        let (a, b) = (first.assignment()?, second.assignment()?);
        Ok(!a.is_empty() && !b.is_empty() && a.len() + b.len() == PARTITIONS)
    })
    .await?;
    let (first_after_join, second_after_join) = (first.assignment()?, second.assignment()?);

    // 12 chuyến, key = tripId: mỗi chuyến chỉ một consumer nhận.
    for n in 1..=12_u32 {
        send(&producer, &input, &trip_key(n), &trip_json(n, 5)).await?;
    }
    wait_until("12 bút toán được ghi", || {
        Ok(worker.processor.entry_count() == 12)
    })
    .await?;

    // Consumer 1 rời group: consumer 2 nhận lại toàn bộ partition.
    let events_first = first.events();
    let stats_first = first.stop().await?;
    wait_until("consumer 2 nhận lại mọi partition", || {
        Ok(second.assignment()?.len() == PARTITIONS)
    })
    .await?;
    let second_after_leave = second.assignment()?;
    let events_second = second.events();
    let stats_second = second.stop().await?;

    Ok(GroupReport {
        partitions: PARTITIONS,
        alone,
        first_after_join,
        second_after_join,
        processed: (stats_first.processed, stats_second.processed),
        partitions_seen: (stats_first.partitions, stats_second.partitions),
        ledger_entries: worker.processor.entry_count(),
        second_after_leave,
        events: (events_first, events_second),
    })
}

// ---------------------------------------------------------------------------------------------
// Kịch bản 4: thông điệp hỏng và version lạ không được làm kẹt partition.
// ---------------------------------------------------------------------------------------------

/// Năm thông điệp trong MỘT partition: hợp lệ, JSON cụt, version 2, thiếu `fareDong`, hợp lệ.
fn poison_messages() -> Vec<(String, Vec<u8>)> {
    let truncated = br#"{"schemaVersion":1,"tripId":"TN-0002","custom"#.to_vec();
    let v2 = br#"{"schemaVersion":2,"tripId":"TN-0003","fare":{"amount":25500,"currency":"VND"}}"#
        .to_vec();
    let no_fare = br#"{"schemaVersion":1,"tripId":"TN-0004","customerId":"CU-001","driverId":"DR-001","occurredAt":"2026-11-06T02:30:00Z"}"#.to_vec();
    vec![
        (trip_key(1), trip_json(1, 5)),
        (trip_key(2), truncated),
        (trip_key(3), v2),
        (trip_key(4), no_fare),
        (trip_key(2), trip_json(2, 3)),
    ]
}

pub struct PoisonReport {
    pub resilient: Vec<Step>,
    pub resilient_committed: Option<i64>,
    pub resilient_entries: usize,
    pub naive_failures: Vec<(i64, String)>,
    pub naive_committed: Option<i64>,
    pub naive_entries: usize,
}

pub async fn poison_messages_scenario(lab: &Lab) -> Result<PoisonReport> {
    let input = lab.create_topic("trip-completed", 1).await?;
    let output = lab.create_topic("ledger-entry-posted", 1).await?;
    let producer = lab.producer()?;
    let messages = poison_messages();
    for (key, payload) in &messages {
        send(&producer, &input, key, payload).await?;
    }

    // Consumer đúng: lỗi giải mã thì log, bỏ qua, VẪN commit để partition đi tiếp.
    let worker = Worker::new(Arc::new(Processor::new()), producer.clone(), output.clone());
    let group = lab.group_name(None);
    let consumer = lab.consumer(&group)?;
    consumer.subscribe(&[&input])?;
    let mut resilient = Vec::new();
    for _ in 0..messages.len() {
        let msg = recv_within(&consumer, WAIT, "thông điệp trong topic poison").await?;
        let outcome = worker.handle(msg.payload()).await?;
        consumer.commit_message(&msg, CommitMode::Sync)?;
        resilient.push(step_of(msg.partition(), msg.offset(), &outcome));
    }
    let resilient_committed = committed_offset(&consumer, &input, 0)?;

    // Consumer ngây thơ: gặp thông điệp xấu thì dừng, không commit. Khởi động lại bao nhiêu lần
    // cũng dừng đúng chỗ đó, vì offset đã commit chỉ tới ngay trước thông điệp xấu.
    let naive_worker = Worker::new(Arc::new(Processor::new()), producer, output);
    let naive_group = lab.group_name(Some("naive"));
    let mut naive_failures = Vec::new();
    for _ in 0..2 {
        let consumer = lab.consumer(&naive_group)?;
        consumer.subscribe(&[&input])?;
        loop {
            let msg = recv_within(&consumer, WAIT, "thông điệp của consumer ngây thơ").await?;
            let outcome = naive_worker.handle(msg.payload()).await?;
            if let Summary::Skipped(why) = outcome.summary() {
                naive_failures.push((msg.offset(), why));
                break; // «crash»: không commit
            }
            consumer.commit_message(&msg, CommitMode::Sync)?;
        }
    }
    let probe = lab.consumer(&naive_group)?;
    let naive_committed = committed_offset(&probe, &input, 0)?;

    Ok(PoisonReport {
        resilient,
        resilient_committed,
        resilient_entries: worker.processor.entry_count(),
        naive_failures,
        naive_committed,
        naive_entries: naive_worker.processor.entry_count(),
    })
}

// ---------------------------------------------------------------------------------------------
// Kịch bản 5: không commit thì consumer vẫn đi tiếp; muốn đọc lại phải seek.
// ---------------------------------------------------------------------------------------------

pub struct NoRewindReport {
    pub failed: i64,
    pub next: i64,
    pub committed: Option<i64>,
    pub after_seek: i64,
    pub after_restart: i64,
}

pub async fn no_commit_does_not_rewind(lab: &Lab) -> Result<NoRewindReport> {
    let input = lab.create_topic("trip-completed", 1).await?;
    let producer = lab.producer()?;
    for n in 1..=3_u32 {
        send(&producer, &input, &trip_key(n), &trip_json(n, 5)).await?;
    }
    let group = lab.group_name(None);
    let consumer = lab.consumer(&group)?;
    consumer.subscribe(&[&input])?;

    // Offset 0 gặp lỗi hạ tầng giả lập: không commit, nhưng vị trí đọc trong bộ nhớ đã qua nó.
    let failed = recv_within(&consumer, WAIT, "offset 0").await?.offset();
    // Commit thông điệp sau cũng che luôn thông điệp lỗi: offset đã commit là một con trỏ, không phải tập hợp.
    let next = {
        let msg = recv_within(&consumer, WAIT, "offset 1").await?;
        consumer.commit_message(&msg, CommitMode::Sync)?;
        msg.offset()
    };
    let committed = committed_offset(&consumer, &input, 0)?;
    // Muốn xử lý lại trong cùng phiên thì phải seek.
    consumer.seek(&input, 0, Offset::Offset(failed), WAIT)?;
    let after_seek = recv_within(&consumer, WAIT, "offset 0 sau seek")
        .await?
        .offset();
    drop(consumer);

    // Khởi động lại: đọc từ offset đã commit, offset 0 không bao giờ quay lại.
    let consumer = lab.consumer(&group)?;
    consumer.subscribe(&[&input])?;
    let after_restart = recv_within(&consumer, WAIT, "offset sau khi khởi động lại")
        .await?
        .offset();
    Ok(NoRewindReport {
        failed,
        next,
        committed,
        after_seek,
        after_restart,
    })
}

// ---------------------------------------------------------------------------------------------
// Kịch bản 6: hết giờ chờ thì lỗi rõ, không treo.
// ---------------------------------------------------------------------------------------------

pub async fn idle_timeout(lab: &Lab) -> Result<String> {
    let topic = lab.create_topic("idle", 1).await?;
    let consumer = lab.consumer(&lab.group_name(None))?;
    consumer.subscribe(&[&topic])?;
    match recv_within(
        &consumer,
        Duration::from_millis(1_500),
        "thông điệp trên topic rỗng",
    )
    .await
    {
        Err(err @ LabError::Timeout { .. }) => Ok(err.to_string()),
        Err(other) => Err(other.into()),
        Ok(_) => Err(anyhow!("topic rỗng mà nhận được thông điệp")),
    }
}
