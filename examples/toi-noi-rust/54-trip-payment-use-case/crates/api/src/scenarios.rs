//! Các kịch bản end-to-end dùng chung cho `cargo run`, `--verify` và `cargo test`.
//! Mỗi kịch bản tự dựng một `LabEnv` riêng (schema `lab_<uuid>`, topic `lab.54.<uuid>.*`) nên chạy
//! song song được và không để lại dữ liệu. Không kịch bản nào assert thời gian: chờ theo điều kiện
//! có hạn chót, thứ tự đến dựng bằng cách đợi bộ đếm tiến độ.

use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail, ensure};
use application::{AppError, Command, Outcome, PaymentStore, StoreError, handle};
use domain::reconcile::{Kind, Line, reconcile};
use domain::{
    AccountId, JournalEntryBuilder, LedgerEntryPosted, Money, PaymentCaptured, PlannedEntry,
    PromoGrant, TripCompleted, Waiting, flows,
};
use infrastructure::lab::LabEnv;
use infrastructure::pg::{EntryDump, LedgerDump, PgStore};
use infrastructure::probe::Probe;
use infrastructure::progress::wait_for;
use infrastructure::service::Service;
use infrastructure::{Fault, kafka};
use tokio::process::Command as Process;

use crate::gated::Gated;

/// Hạn chót cho một sự kiện đi hết vòng Kafka → PostgreSQL → Kafka. Chỉ là hạn chót, không phải thời gian chờ.
const WITHIN: Duration = Duration::from_secs(60);

/// Những dòng tường thuật của một kịch bản (in ra ở `cargo run`).
#[derive(Debug, Default)]
pub struct Report(pub Vec<String>);

impl Report {
    fn say(&mut self, line: impl Into<String>) {
        self.0.push(line.into());
    }
}

/// Thứ tự đến của hai sự kiện.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    /// `TripCompleted` rồi `PaymentCaptured`.
    TripFirst,
    /// `PaymentCaptured` rồi `TripCompleted`.
    CaptureFirst,
}

/// Kết quả một lần chạy luồng chính.
#[derive(Debug)]
pub struct FlowResult {
    /// Tường thuật.
    pub report: Report,
    /// Ảnh chụp sổ cuối ca.
    pub dump: LedgerDump,
    /// `LedgerEntryPosted` nhận được ở topic ra, theo thứ tự đến: `(key, [(tài khoản, số tiền)])`.
    pub events: Vec<(String, Vec<(String, i64)>)>,
}

// ---------- công cụ dùng chung ----------

fn trip_event(trip: &str, customer: &str, driver: &str, fare: i64) -> TripCompleted {
    TripCompleted::sample(trip, customer, driver, fare)
}

fn capture_event(trip: &str, payment: &str, external_ref: &str, amount: i64) -> PaymentCaptured {
    PaymentCaptured::sample(trip, payment, external_ref, amount)
}

fn lines_of(event: &LedgerEntryPosted) -> Vec<(String, i64)> {
    event
        .postings
        .iter()
        .map(|p| (p.account.clone(), p.amount_dong))
        .collect()
}

fn show(postings: &[(String, i64)]) -> String {
    postings
        .iter()
        .map(|(account, dong)| format!("{account} {}", Money::from_dong(*dong)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn amount_or_dash(amount: Option<i64>) -> String {
    amount.map_or_else(
        || "không có".to_string(),
        |dong| Money::from_dong(dong).to_string(),
    )
}

/// Số dư kỳ vọng của `TN-0001` sau khi chốt sổ: không phụ thuộc thứ tự đến.
const TN_0001: [(&str, i64); 4] = [
    ("customer:CU-001", -25_500),
    ("psp:vixanh:clearing", 0),
    ("platform:revenue", 5_100),
    ("driver:DR-001", 20_400),
];

async fn expect_balances(store: &PgStore, expected: &[(&str, i64)]) -> Result<()> {
    for (account, dong) in expected {
        let got = store.balance(account).await?;
        ensure!(got == Some(*dong), "{account}: có {got:?}, mong {dong}");
    }
    Ok(())
}

/// Ba bất biến đọc bằng SQL: không bút toán lệch, tổng toàn sổ 0, số dư khớp tổng posting.
async fn expect_books_sound(store: &PgStore) -> Result<()> {
    let inv = store.invariants().await?;
    ensure!(
        inv.unbalanced == 0,
        "có {} bút toán không cân",
        inv.unbalanced
    );
    ensure!(
        inv.postings_total == 0,
        "tổng toàn sổ {}, mong 0",
        inv.postings_total
    );
    ensure!(
        inv.projection_drift == 0,
        "{} tài khoản có số dư lệch tổng posting",
        inv.projection_drift
    );
    Ok(())
}

fn expected_tn_0001() -> Vec<EntryDump> {
    let line = |a: &str, d: i64| (a.to_string(), d);
    vec![
        EntryDump {
            key: "trip:TN-0001:capture".into(),
            postings: vec![
                line("customer:CU-001", -25_500),
                line("psp:vixanh:clearing", 25_500),
            ],
        },
        EntryDump {
            key: "trip:TN-0001:split".into(),
            postings: vec![
                line("psp:vixanh:clearing", -25_500),
                line("platform:revenue", 5_100),
                line("driver:DR-001", 20_400),
            ],
        },
    ]
}

/// Chốt sổ một chuyến qua `handle`, như consumer làm khi cả hai sự kiện đã đến.
async fn settle<S: PaymentStore>(
    store: &S,
    trip: &TripCompleted,
    capture: &PaymentCaptured,
) -> Result<()> {
    handle(store, &Command::TripCompleted(trip.clone())).await?;
    handle(store, &Command::PaymentCaptured(capture.clone())).await?;
    Ok(())
}

// ---------- 1 và 2: hai thứ tự đến, cùng một kết quả ----------

/// Luồng chính qua PostgreSQL và Kafka thật, theo thứ tự `order`.
///
/// # Errors
/// Bất kỳ bước nào sai kỳ vọng hoặc hết hạn chót.
pub async fn flow(order: Order) -> Result<FlowResult> {
    LabEnv::run(|env| async move {
        let mut report = Report::default();
        let svc = Service::start(&env.service_config(Fault::None, true)).await?;
        let store = svc.store.clone();
        let probe = Probe::new(&env)?;
        let producer = kafka::producer(&env.kafka)?;
        let mut rx = svc.progress.subscribe();
        let trip = trip_event("TN-0001", "CU-001", "DR-001", 25_500);
        let paid = capture_event("TN-0001", "PAY-0001", "vx_ch_0001", 25_500);

        let (first, still_waiting) = match order {
            Order::TripFirst => ("TripCompleted", Waiting::PaymentCaptured),
            Order::CaptureFirst => ("PaymentCaptured", Waiting::TripCompleted),
        };
        match order {
            Order::TripFirst => {
                kafka::send_json(&producer, &env.topic_trips(), "TN-0001", &trip).await?;
            }
            Order::CaptureFirst => {
                kafka::send_json(&producer, &env.topic_captures(), "TN-0001", &paid).await?;
            }
        }
        wait_for(&mut rx, &format!("{first} đã xử lý"), WITHIN, |c| c.handled >= 1).await?;
        // Giữa chừng: aggregate đã nhớ dữ kiện đầu và biết mình còn chờ gì; sổ chưa có bút toán nào.
        let mid = store.load("TN-0001").await?.context("aggregate phải tồn tại")?;
        ensure!(mid.payment.waiting_for() == vec![still_waiting.clone()], "chờ sai: {:?}", mid.payment.waiting_for());
        ensure!(store.dump().await?.entries.is_empty(), "chưa đủ dữ kiện mà sổ đã có bút toán");
        report.say(format!("{first} đến trước (version {}): {}; sổ còn 0 bút toán", mid.version, still_waiting));

        match order {
            Order::TripFirst => {
                kafka::send_json(&producer, &env.topic_captures(), "TN-0001", &paid).await?;
            }
            Order::CaptureFirst => {
                kafka::send_json(&producer, &env.topic_trips(), "TN-0001", &trip).await?;
            }
        }
        let events = probe.next_events(2, WITHIN).await?;
        let keys: Vec<&str> = events.iter().map(|e| e.idempotency_key.as_str()).collect();
        ensure!(keys == ["trip:TN-0001:capture", "trip:TN-0001:split"], "key/thứ tự sai: {keys:?}");
        let ids: Vec<&str> = events.iter().map(|e| e.entry_id.as_str()).collect();
        ensure!(ids == ["JE-000001", "JE-000002"], "mã bút toán: {ids:?}");
        for event in &events {
            report.say(format!("{} {}: {}", event.entry_id, event.idempotency_key, show(&lines_of(event))));
        }
        expect_balances(&store, &TN_0001).await?;
        expect_books_sound(&store).await?;
        let last = store.load("TN-0001").await?.context("aggregate")?;
        ensure!(last.payment.settled && last.version == 2, "aggregate: settled={}, version={}", last.payment.settled, last.version);
        let counters = svc.progress.snapshot();
        ensure!(counters.conflicts == 0, "một consumer tuần tự không có tranh chấp");
        let dump = store.dump().await?;
        svc.stop().await?;
        let inv = store.invariants().await?;
        ensure!(inv.outbox_total == 2 && inv.outbox_unpublished == 0, "outbox: {inv:?}");
        report.say("số dư: customer -25.500, clearing 0, revenue 5.100, driver 20.400; aggregate version 2");
        let events = events.iter().map(|e| (e.idempotency_key.clone(), lines_of(e))).collect();
        Ok(FlowResult { report, dump, events })
    })
    .await
}

/// Chạy cả hai thứ tự rồi so: bút toán, số dư và sự kiện phát ra phải y hệt nhau.
///
/// # Errors
/// Một trong hai luồng sai, hoặc hai kết quả khác nhau.
pub async fn both_orders() -> Result<Report> {
    let a = flow(Order::TripFirst)
        .await
        .context("thứ tự chuyến-trước")?;
    let b = flow(Order::CaptureFirst)
        .await
        .context("thứ tự tiền-trước")?;
    ensure!(
        a.dump == b.dump,
        "sổ khác nhau:\n{:#?}\n{:#?}",
        a.dump,
        b.dump
    );
    ensure!(a.events == b.events, "sự kiện phát ra khác nhau");
    ensure!(
        a.dump.entries == expected_tn_0001(),
        "bút toán lệch kỳ vọng: {:#?}",
        a.dump.entries
    );
    let mut report = Report::default();
    report.say("-- chuyến xong trước");
    report.0.extend(a.report.0.iter().map(|l| format!("  {l}")));
    report.say("-- tiền về trước");
    report.0.extend(b.report.0.iter().map(|l| format!("  {l}")));
    report.say("hai thứ tự cho cùng bút toán, cùng số dư, cùng LedgerEntryPosted");
    Ok(report)
}

// ---------- 3: hai sự kiện cùng đến, một bên thua version ----------

/// Chạy hai lệnh cùng chuyến đồng thời trên một kho bọc `Gated`: cả hai đọc xong cùng version cũ
/// rồi mới cùng đi tới commit. Trả hai `Outcome` theo thứ tự lệnh.
async fn run_pair(store: &PgStore, commands: [Command; 2]) -> Result<Vec<Outcome>> {
    let gated = Arc::new(Gated::new(store.clone(), 2));
    let tasks: Vec<_> = commands
        .into_iter()
        .map(|command| {
            let gated = Arc::clone(&gated);
            tokio::spawn(async move { handle(&*gated, &command).await })
        })
        .collect();
    let mut outcomes = Vec::new();
    for task in tasks {
        outcomes.push(task.await??);
    }
    Ok(outcomes)
}

// Mỗi vòng đúng một bên thua; bên thua đọc lại và làm nốt. `posted` và `version` là số kỳ vọng của vòng.
fn expect_one_loser(phase: u8, round: usize, outcomes: &[Outcome], posted: usize) -> Result<()> {
    let conflicts: u32 = outcomes.iter().map(|o| o.conflicts).sum();
    ensure!(
        conflicts == 1,
        "pha {phase}, vòng {round}: {conflicts} lần thua version, mong đúng 1"
    );
    let mut attempts: Vec<u32> = outcomes.iter().map(|o| o.attempts).collect();
    attempts.sort_unstable();
    ensure!(
        attempts == [1, 2],
        "pha {phase}, vòng {round}: số lần thử {attempts:?}"
    );
    let written: usize = outcomes.iter().map(|o| o.posted.len()).sum();
    ensure!(
        written == posted,
        "pha {phase}, vòng {round}: ghi {written} bút toán, mong {posted}"
    );
    Ok(())
}

/// Hai pha, mỗi pha mười vòng. Pha 1 (tạo): `TripCompleted` và `PaymentCaptured` cùng thấy «chưa có
/// aggregate». Pha 2 (cập nhật): aggregate đã có version 1, `PaymentCaptured` và quyết định khuyến
/// mãi cùng đọc nó. Mỗi vòng đúng một bên thua, đọc lại và chốt sổ; không bao giờ có bút toán thừa.
///
/// # Errors
/// Kết quả một vòng lệch kỳ vọng.
pub async fn version_race() -> Result<Report> {
    LabEnv::run_pg(|env| async move {
        let mut report = Report::default();
        let store = env.store(Fault::None).await?;

        // Pha 1: hai bên cùng INSERT; unique_violation của bên sau được dịch thành VersionConflict.
        let mut wins = [0u32; 2]; // [bên chuyến-xong thắng, bên tiền-về thắng]
        for round in 0..10 {
            let trip_id = format!("TN-{:04}", 100 + round);
            let commands = [
                Command::TripCompleted(trip_event(&trip_id, "CU-001", "DR-001", 25_500)),
                Command::PaymentCaptured(capture_event(&trip_id, &format!("PAY-{round:04}"), &format!("vx_ch_{round:04}"), 25_500)),
            ];
            let outcomes = run_pair(&store, commands).await?;
            expect_one_loser(1, round, &outcomes, 2)?;
            let loaded = store.load(&trip_id).await?.context("aggregate")?;
            ensure!(loaded.payment.settled && loaded.version == 2, "vòng {round}: version {}", loaded.version);
            // Bên thua là bên có `attempts == 2`: index 0 là TripCompleted.
            wins[usize::from(outcomes[0].attempts == 2)] += 1;
        }
        report.say("pha 1, tạo: 10 vòng, hai lệnh cùng thấy «chưa có aggregate» rồi cùng commit");
        report.say("  mỗi vòng: đúng 1 lần thua version, số lần thử [1, 2], 2 bút toán, aggregate version 2");
        report.say(format!("  bên thắng đổi giữa các vòng ({} lần TripCompleted, {} lần PaymentCaptured), kết quả không đổi", wins[0], wins[1]));

        // Pha 2: UPDATE ... WHERE version = 1; bên sau thấy 0 dòng khớp.
        let fund = PlannedEntry {
            entry: flows::fund_entry("WELCOME5K", Money::from_dong(50_000))?,
            external_ref: None,
            requires_funds: None,
        };
        store.post_standalone(&fund).await?;
        for round in 0..10 {
            let trip_id = format!("TN-{:04}", 200 + round);
            handle(&store, &Command::TripCompleted(trip_event(&trip_id, "CU-002", "DR-002", 25_500))).await?;
            let grant = PromoGrant { campaign: "WELCOME5K".into(), platform_dong: 5_000, driver_dong: 0 };
            let commands = [
                Command::PaymentCaptured(capture_event(&trip_id, &format!("PAY-{}", 200 + round), &format!("vx_ch_{}", 200 + round), 20_500)),
                Command::GrantPromo { trip_id: trip_id.clone(), grant },
            ];
            let outcomes = run_pair(&store, commands).await?;
            expect_one_loser(2, round, &outcomes, 3)?;
            let loaded = store.load(&trip_id).await?.context("aggregate")?;
            ensure!(
                loaded.payment.settled && loaded.payment.promo.is_some() && loaded.version == 3,
                "vòng {round}: settled={}, version {}",
                loaded.payment.settled,
                loaded.version
            );
        }
        report.say("pha 2, cập nhật: 10 vòng, aggregate đã có version 1; PaymentCaptured 20.500 và khuyến mãi 5.000 cùng đọc nó");
        report.say("  mỗi vòng: đúng 1 lần thua version, 3 bút toán (promo, capture, split), aggregate version 3");

        let inv = store.invariants().await?;
        ensure!(inv.entries == 1 + 20 + 30, "{} bút toán, mong 51 (1 cấp ngân sách + 20 + 30)", inv.entries);
        expect_books_sound(&store).await?;
        ensure!(store.balance("promo:budget:WELCOME5K").await? == Some(0), "ngân sách phải về 0 sau 10 lần dùng 5.000");
        report.say("toàn sổ: 51 bút toán, 0 bút toán lệch, tổng 0, projection khớp, ngân sách 50.000 dùng hết đúng 10 lần");
        Ok(report)
    })
    .await
}

// ---------- 4: khuyến mãi, ngân sách, số thu lệch ----------

/// `TN-0002` có mã giảm 5.000 do nền tảng chịu và quyết định đến SAU hai sự kiện kia; ngân sách
/// campaign không đủ thì từ chối; số thu lệch cước thì không ghi gì.
///
/// # Errors
/// Kết quả lệch kỳ vọng.
pub async fn promo_and_mismatch() -> Result<Report> {
    LabEnv::run_pg(|env| async move {
        let mut report = Report::default();
        let store = env.store(Fault::None).await?;
        let fund = PlannedEntry {
            entry: flows::fund_entry("WELCOME5K", Money::from_dong(10_000))?,
            external_ref: None,
            requires_funds: None,
        };
        store.post_standalone(&fund).await?;
        report.say("cấp ngân sách WELCOME5K 10.000 từ platform:treasury (bài 45)");

        let trip = trip_event("TN-0002", "CU-002", "DR-002", 25_500);
        let paid = capture_event("TN-0002", "PAY-0002", "vx_ch_0002", 20_500);
        handle(&store, &Command::TripCompleted(trip)).await?;
        let waiting = handle(&store, &Command::PaymentCaptured(paid)).await?;
        ensure!(waiting.posted.is_empty(), "số thu lệch cước mà đã ghi sổ");
        let state = store.load("TN-0002").await?.context("aggregate")?.payment;
        report.say(format!("TN-0002 có cả hai sự kiện: {}", state.waiting_for()[0]));

        let grant = PromoGrant { campaign: "WELCOME5K".into(), platform_dong: 5_000, driver_dong: 0 };
        let settled = handle(&store, &Command::GrantPromo { trip_id: "TN-0002".into(), grant }).await?;
        ensure!(settled.events == ["PromoGranted", "Settled"], "sự kiện: {:?}", settled.events);
        let dump = store.dump().await?;
        for entry in dump.entries.iter().filter(|e| e.key.contains("TN-0002")) {
            report.say(format!("{}: {}", entry.key, show(&entry.postings)));
        }
        expect_balances(
            &store,
            &[
                ("customer:CU-002", -20_500),
                ("psp:vixanh:clearing", 0),
                ("platform:promo-wallet", -5_000),
                ("platform:revenue", 5_100),
                ("driver:DR-002", 20_400),
                ("platform:promo-expense", 5_000),
                ("promo:budget:WELCOME5K", 5_000),
                ("platform:treasury", -10_000),
            ],
        )
        .await?;
        report.say("doanh thu 5.100 - chi phí khuyến mãi 5.000 = 100; ngân sách còn 5.000; clearing 0");

        // Ngân sách còn 5.000, TN-0003 xin 6.000: bị từ chối và không để lại dấu vết nào.
        let greedy = PromoGrant { campaign: "WELCOME5K".into(), platform_dong: 6_000, driver_dong: 0 };
        let refused = handle(&store, &Command::GrantPromo { trip_id: "TN-0003".into(), grant: greedy }).await;
        match refused {
            Err(AppError::Store(StoreError::BudgetExhausted { available, requested, .. })) => {
                ensure!(available == 5_000 && requested == 6_000, "số liệu từ chối: {available}/{requested}");
            }
            other => bail!("mong BudgetExhausted, nhận {other:?}"),
        }
        ensure!(store.load("TN-0003").await?.is_none(), "từ chối mà vẫn lưu aggregate");
        ensure!(store.balance("promo:budget:WELCOME5K").await? == Some(5_000), "ngân sách đã bị trừ");
        report.say("TN-0003 xin 6.000 khi ngân sách còn 5.000: BudgetExhausted, không có aggregate, không có bút toán");

        // Số thu 24.000 không bằng cước 25.500 và không có khuyến mãi nào giải thích: chờ, không đoán.
        let before = store.invariants().await?.entries;
        handle(&store, &Command::TripCompleted(trip_event("TN-0004", "CU-004", "DR-004", 25_500))).await?;
        let odd = handle(&store, &Command::PaymentCaptured(capture_event("TN-0004", "PAY-0004", "vx_ch_0004", 24_000))).await?;
        ensure!(odd.posted.is_empty() && store.invariants().await?.entries == before, "số thu lệch mà vẫn ghi sổ");
        let waiting = store.load("TN-0004").await?.context("aggregate")?.payment.waiting_for();
        report.say(format!("TN-0004: {}", waiting[0]));
        expect_books_sound(&store).await?;
        Ok(report)
    })
    .await
}

// ---------- 4b: sổ đã có bút toán ba dòng của bài 40 ----------

/// Sổ đã có `trip:TN-0001:capture` dạng ba dòng của bài 40. Bước capture hai dòng của bài này
/// dùng CÙNG key với nội dung khác nên bị từ chối, và việc từ chối kéo theo cả aggregate rollback.
///
/// # Errors
/// Kết quả lệch kỳ vọng.
pub async fn legacy_three_line() -> Result<Report> {
    LabEnv::run_pg(|env| async move {
        let mut report = Report::default();
        let store = env.store(Fault::None).await?;
        let acct = |raw: &str| AccountId::new(raw);
        let legacy = JournalEntryBuilder::new("trip:TN-0001:capture")
            .posting(acct("customer:CU-001")?, Money::from_dong(-25_500))
            .posting(acct("platform:revenue")?, Money::from_dong(5_100))
            .posting(acct("driver:DR-001")?, Money::from_dong(20_400))
            .build()?;
        let planned = PlannedEntry { entry: legacy, external_ref: None, requires_funds: None };
        store.post_standalone(&planned).await?;
        report.say("sổ có sẵn dạng ba dòng của bài 40: customer -25.500, revenue 5.100, driver 20.400");

        handle(&store, &Command::TripCompleted(trip_event("TN-0001", "CU-001", "DR-001", 25_500))).await?;
        let refused = handle(&store, &Command::PaymentCaptured(capture_event("TN-0001", "PAY-0001", "vx_ch_0001", 25_500))).await;
        match refused {
            Err(err @ AppError::Store(StoreError::IdempotencyConflict { .. })) => {
                ensure!(err.is_permanent(), "xung đột key phải là lỗi vĩnh viễn");
                report.say(format!("capture hai dòng cùng key: {err}"));
            }
            other => bail!("mong IdempotencyConflict, nhận {other:?}"),
        }
        let left = store.load("TN-0001").await?.context("aggregate")?;
        ensure!(left.version == 1 && left.payment.capture.is_none() && !left.payment.settled, "aggregate phải rollback cùng bút toán");
        ensure!(store.invariants().await?.entries == 1, "chỉ được còn bút toán ba dòng");
        ensure!(store.balance("psp:vixanh:clearing").await?.is_none(), "clearing không được có dòng nào");
        report.say("hậu quả: aggregate vẫn version 1 và không nhớ PaymentCaptured; sổ còn 1 bút toán; clearing chưa tồn tại");
        expect_books_sound(&store).await?;
        Ok(report)
    })
    .await
}

// ---------- 5: hoàn một phần ----------

/// Hoàn 10.000 của `TN-0001` (lần 1), gửi lại cùng lệnh, rồi xin hoàn 16.000 (vượt phần còn lại).
///
/// # Errors
/// Kết quả lệch kỳ vọng.
pub async fn partial_refund() -> Result<Report> {
    LabEnv::run_pg(|env| async move {
        let mut report = Report::default();
        let store = env.store(Fault::None).await?;
        settle(
            &store,
            &trip_event("TN-0001", "CU-001", "DR-001", 25_500),
            &capture_event("TN-0001", "PAY-0001", "vx_ch_0001", 25_500),
        )
        .await?;
        let refund = |n: u32, amount: i64| Command::Refund {
            trip_id: "TN-0001".into(),
            n,
            amount_dong: amount,
        };

        let first = handle(&store, &refund(1, 10_000)).await?;
        ensure!(
            first.posted.len() == 1 && first.posted[0].key == "trip:TN-0001:refund:1",
            "lần 1: {first:?}"
        );
        let dump = store.dump().await?;
        let entry = dump.entries.last().context("bút toán hoàn")?;
        report.say(format!("{}: {}", entry.key, show(&entry.postings)));
        // Hoàn một phần không phải ảnh gương của bút toán nào nên không đặt `reversal_of` (bài 43).
        let reversals = store
            .count("SELECT count(*) FROM journal_entries WHERE reversal_of IS NOT NULL")
            .await?;
        ensure!(reversals == 0, "hoàn một phần không được đặt reversal_of");
        report.say("bút toán hoàn không đặt reversal_of");
        expect_balances(
            &store,
            &[
                ("customer:CU-001", -15_500),
                ("platform:revenue", 3_100),
                ("driver:DR-001", 12_400),
                ("psp:vixanh:clearing", 0),
            ],
        )
        .await?;

        let again = handle(&store, &refund(1, 10_000)).await?;
        ensure!(
            again.is_noop() && again.posted.is_empty(),
            "gửi lại lần 1 không được ghi thêm"
        );
        report.say("gửi lại lần 1 (cùng số tiền): không làm gì, vẫn 3 bút toán");

        match handle(&store, &refund(2, 16_000)).await {
            Err(AppError::Domain(err)) => report.say(format!("lần 2 xin 16.000: {err}")),
            other => bail!("mong RefundExceeds, nhận {other:?}"),
        }
        match handle(
            &store,
            &Command::Refund {
                trip_id: "TN-0099".into(),
                n: 1,
                amount_dong: 1_000,
            },
        )
        .await
        {
            Err(AppError::Domain(err)) => report.say(format!("chuyến chưa có dữ kiện nào: {err}")),
            other => bail!("mong NotSettled, nhận {other:?}"),
        }
        ensure!(
            store.invariants().await?.entries == 3,
            "chỉ được có 3 bút toán"
        );
        expect_books_sound(&store).await?;
        report.say("sau hoàn: customer -15.500, revenue 3.100, driver 12.400, clearing 0");
        Ok(report)
    })
    .await
}

// ---------- 6: sự kiện trùng và sự kiện sửa nội dung ----------

/// Phát `TripCompleted` và `PaymentCaptured` mỗi loại hai lần, thêm một `TripCompleted` đổi cước.
/// Đợi đủ 5 message được xử lý (bộ đếm, không ngủ) rồi kiểm sổ.
///
/// # Errors
/// Kết quả lệch kỳ vọng.
pub async fn duplicates() -> Result<Report> {
    LabEnv::run(|env| async move {
        let mut report = Report::default();
        let svc = Service::start(&env.service_config(Fault::None, true)).await?;
        let store = svc.store.clone();
        let probe = Probe::new(&env)?;
        let producer = kafka::producer(&env.kafka)?;
        let mut rx = svc.progress.subscribe();
        let trip = trip_event("TN-0001", "CU-001", "DR-001", 25_500);
        let paid = capture_event("TN-0001", "PAY-0001", "vx_ch_0001", 25_500);
        let mut repriced = trip.clone();
        repriced.fare_dong = 26_000;
        for event in [&trip, &trip, &repriced] {
            kafka::send_json(&producer, &env.topic_trips(), "TN-0001", event).await?;
        }
        for event in [&paid, &paid] {
            kafka::send_json(&producer, &env.topic_captures(), "TN-0001", event).await?;
        }
        let counters = wait_for(&mut rx, "5 message được xử lý", WITHIN, |c| {
            c.handled >= 5
        })
        .await?;
        ensure!(
            counters.noops == 2,
            "mong 2 message trùng, có {}",
            counters.noops
        );
        ensure!(
            counters.poison == 1,
            "mong 1 message đổi nội dung bị bỏ qua, có {}",
            counters.poison
        );
        ensure!(
            counters.posted == 2,
            "mong 2 bút toán, có {}",
            counters.posted
        );
        let events = probe.next_events(2, WITHIN).await?;
        ensure!(events.len() == 2, "phải có đúng 2 LedgerEntryPosted");
        let dump = store.dump().await?;
        ensure!(
            dump.entries == expected_tn_0001(),
            "sổ lệch kỳ vọng: {:#?}",
            dump.entries
        );
        expect_balances(&store, &TN_0001).await?;
        expect_books_sound(&store).await?;
        svc.stop().await?;
        let inv = store.invariants().await?;
        ensure!(
            inv.outbox_total == 2 && inv.outbox_unpublished == 0,
            "outbox: {inv:?}"
        );
        report
            .say("5 message: TripCompleted x2, PaymentCaptured x2, TripCompleted đổi cước 26.000");
        report.say(format!(
            "đã xử lý {}, trùng bỏ qua {}, đổi nội dung bị từ chối {}, bút toán mới {}",
            counters.handled, counters.noops, counters.poison, counters.posted
        ));
        report.say("sổ vẫn 2 bút toán, outbox 2 dòng đã phát, driver:DR-001 vẫn 20.400");
        Ok(report)
    })
    .await
}

// ---------- 7 và 8: chết giữa chừng bằng tiến trình thật ----------

/// Tiến trình con `trip-payment serve` tự `abort` ở `fault`; tiến trình chính kiểm hiện trường rồi
/// chạy service mới cùng consumer group. `exe` là đường tới binary `trip-payment`.
///
/// # Errors
/// Tiến trình con không chết đúng cách, hoặc kết quả sau khi chạy lại lệch kỳ vọng.
pub async fn crash(fault: Fault, exe: &Path) -> Result<Report> {
    ensure!(fault != Fault::None, "ca này cần một điểm chết");
    LabEnv::run(|env| async move {
        let mut report = Report::default();
        let store = env.store(Fault::None).await?;
        let producer = kafka::producer(&env.kafka)?;
        // Gửi cả hai sự kiện TRƯỚC khi tiến trình con chạy: nó sẽ chết ở message làm đủ dữ kiện.
        let trip = trip_event("TN-0001", "CU-001", "DR-001", 25_500);
        let paid = capture_event("TN-0001", "PAY-0001", "vx_ch_0001", 25_500);
        kafka::send_json(&producer, &env.topic_trips(), "TN-0001", &trip).await?;
        kafka::send_json(&producer, &env.topic_captures(), "TN-0001", &paid).await?;

        let child = Process::new(exe)
            .arg("serve")
            .envs(env.child_vars(fault, false))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .with_context(|| format!("không chạy được {}", exe.display()))?;
        // `wait_with_output` đọc stdout và stderr tới EOF rồi mới trả về: không đoán log bằng ngủ.
        let output = tokio::time::timeout(WITHIN, child.wait_with_output())
            .await
            .context("tiến trình con không tự chết trong hạn chót")??;
        let stderr = String::from_utf8_lossy(&output.stderr);
        {
            use std::os::unix::process::ExitStatusExt;
            ensure!(output.status.signal() == Some(6), "mong chết vì SIGABRT, nhận {:?}\n{stderr}", output.status);
        }
        let fault_line = stderr.lines().find(|l| l.starts_with("FAULT:")).context("thiếu dòng FAULT trong log")?;
        report.say(format!("tiến trình con chết (SIGABRT): {fault_line}"));

        let inv = store.invariants().await?;
        match fault {
            Fault::AbortBeforeCommit => {
                ensure!(inv.entries == 0 && inv.outbox_total == 0, "chết trước COMMIT mà DB có dấu vết: {inv:?}");
                let left = store.load("TN-0001").await?.context("dữ kiện đầu đã commit")?;
                ensure!(left.version == 1 && !left.payment.settled, "aggregate sau khi chết: version {}", left.version);
                report.say("hiện trường: 0 bút toán, 0 dòng outbox, aggregate version 1 (dữ kiện thứ hai bị rollback)");
            }
            Fault::AbortAfterCommit => {
                ensure!(inv.entries == 2 && inv.outbox_total == 2 && inv.outbox_unpublished == 2, "hiện trường: {inv:?}");
                let left = store.load("TN-0001").await?.context("aggregate")?;
                ensure!(left.version == 2 && left.payment.settled, "aggregate sau khi chết: version {}", left.version);
                report.say("hiện trường: 2 bút toán đã commit, 2 dòng outbox chưa phát, aggregate version 2, offset chưa commit");
            }
            Fault::None => unreachable!("đã kiểm ở đầu hàm"),
        }

        // Chạy lại cùng consumer group: Kafka giao lại message chưa commit offset.
        let probe = Probe::new(&env)?;
        let svc = Service::start(&env.service_config(Fault::None, true)).await?;
        let mut rx = svc.progress.subscribe();
        let counters = wait_for(&mut rx, "message giao lại được xử lý", WITHIN, |c| c.handled >= 1).await?;
        let events = probe.next_events(2, WITHIN).await?;
        let keys: Vec<&str> = events.iter().map(|e| e.idempotency_key.as_str()).collect();
        ensure!(keys == ["trip:TN-0001:capture", "trip:TN-0001:split"], "sự kiện sau khi chạy lại: {keys:?}");
        let dump = store.dump().await?;
        ensure!(dump.entries == expected_tn_0001(), "sổ sau khi chạy lại: {:#?}", dump.entries);
        expect_balances(&store, &TN_0001).await?;
        expect_books_sound(&store).await?;
        svc.stop().await?;
        let inv = store.invariants().await?;
        ensure!(inv.outbox_total == 2 && inv.outbox_unpublished == 0, "outbox sau khi chạy lại: {inv:?}");
        match fault {
            Fault::AbortAfterCommit => {
                ensure!(counters.noops == 1 && counters.posted == 0, "giao lại phải là message trùng: {counters:?}");
                report.say("chạy lại: message giao lại bị aggregate nhận ra là trùng (noops 1, ghi mới 0); relay phát nốt 2 dòng outbox");
            }
            Fault::AbortBeforeCommit => {
                ensure!(counters.noops == 0 && counters.posted == 2, "giao lại phải ghi thật: {counters:?}");
                report.say("chạy lại: message giao lại được xử lý thật (ghi mới 2); relay phát 2 dòng outbox");
            }
            Fault::None => unreachable!("đã kiểm ở đầu hàm"),
        }
        report.say("kết quả cuối: đúng 2 bút toán, đúng 2 LedgerEntryPosted, driver:DR-001 20.400, y hệt luồng không chết");
        Ok(report)
    })
    .await
}

// ---------- 9: đối soát cuối luồng ----------

/// Chốt sổ `TN-0001` (25.500) và `TN-0002` (có khuyến mãi, VíXanh thu 20.500) rồi đối soát theo
/// dòng clearing với một sao kê dựng từ chính hai `external_ref`; sau đó bóp méo sao kê để thấy báo lệch.
///
/// # Errors
/// Kết quả lệch kỳ vọng.
pub async fn reconcile_clearing() -> Result<Report> {
    LabEnv::run_pg(|env| async move {
        let mut report = Report::default();
        let store = env.store(Fault::None).await?;
        let fund = PlannedEntry {
            entry: flows::fund_entry("WELCOME5K", Money::from_dong(10_000))?,
            external_ref: None,
            requires_funds: None,
        };
        store.post_standalone(&fund).await?;
        settle(
            &store,
            &trip_event("TN-0001", "CU-001", "DR-001", 25_500),
            &capture_event("TN-0001", "PAY-0001", "vx_ch_0001", 25_500),
        )
        .await?;
        let grant = PromoGrant {
            campaign: "WELCOME5K".into(),
            platform_dong: 5_000,
            driver_dong: 0,
        };
        handle(
            &store,
            &Command::GrantPromo {
                trip_id: "TN-0002".into(),
                grant,
            },
        )
        .await?;
        settle(
            &store,
            &trip_event("TN-0002", "CU-002", "DR-002", 25_500),
            &capture_event("TN-0002", "PAY-0002", "vx_ch_0002", 20_500),
        )
        .await?;
        // Hoàn một phần không đụng clearing (khách nhận lại vào ví), nên không làm đổi đối soát.
        handle(
            &store,
            &Command::Refund {
                trip_id: "TN-0001".into(),
                n: 1,
                amount_dong: 10_000,
            },
        )
        .await?;

        let ours = store.clearing_lines().await?;
        report.say(format!(
            "dòng clearing của bút toán capture: {}",
            ours.iter()
                .map(|l| format!("{} {}", l.external_ref, Money::from_dong(l.amount_dong)))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        // Sao kê giả của VíXanh: thu đúng số tiền thật đã thu của từng giao dịch.
        let statement = vec![
            Line::new("vx_ch_0001", 25_500),
            Line::new("vx_ch_0002", 20_500),
        ];
        let items = reconcile(&ours, &statement);
        ensure!(
            items.iter().all(|i| i.kind == Kind::Matched),
            "sao kê đúng mà vẫn lệch: {items:?}"
        );
        ensure!(
            store.balance("psp:vixanh:clearing").await? == Some(0),
            "clearing phải về 0 sau khi chia hết"
        );
        report.say(
            "sao kê đúng: 2 ref MATCHED; số dư clearing 0 (đối soát so theo dòng, không so số dư)",
        );

        // Bóp méo: đổi một số tiền, bỏ một dòng, thêm một dòng lạ.
        let bad = vec![
            Line::new("vx_ch_0002", 20_000),
            Line::new("vx_ch_0077", 9_000),
        ];
        let items = reconcile(&ours, &bad);
        let got: Vec<(&str, Kind)> = items
            .iter()
            .map(|i| (i.external_ref.as_str(), i.kind))
            .collect();
        ensure!(
            got == [
                ("vx_ch_0001", Kind::MissingInPsp),
                ("vx_ch_0002", Kind::AmountMismatch),
                ("vx_ch_0077", Kind::MissingInLedger)
            ],
            "phân loại sai: {got:?}"
        );
        for item in &items {
            report.say(format!(
                "  {} {}: sổ ta {}, VíXanh {}",
                item.external_ref,
                item.kind.as_str(),
                amount_or_dash(item.ours),
                amount_or_dash(item.psp)
            ));
        }
        expect_books_sound(&store).await?;
        Ok(report)
    })
    .await
}
