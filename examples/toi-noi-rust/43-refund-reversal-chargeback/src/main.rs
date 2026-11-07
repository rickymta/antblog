//! Lab bài 43: chạy năm thí nghiệm trên PostgreSQL thật, mỗi thí nghiệm trong schema riêng.
//! `--verify` kiểm các giá trị in ra; sai thì thoát mã 1.

use std::sync::Arc;

use anyhow::Result;
use ledger_core::{Money, PaymentState};
use ledger_lab::chargeback::{ChargebackService, LossPolicy};
use ledger_lab::db::{LabDb, with_lab};
use ledger_lab::error::RefundError;
use ledger_lab::ledger::{self, Trip, entry_lines, reverse_entry};
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

// Gom các kỳ vọng sai thay vì dừng ở cái đầu tiên, để một lần chạy báo hết.
#[derive(Default)]
struct Checks(Vec<String>);

impl Checks {
    fn expect(&mut self, ok: bool, what: &str) {
        if !ok {
            self.0.push(what.to_string());
        }
    }
}

async fn balance_line(pool: &PgPool) -> Result<String> {
    let b = ledger::balances(pool).await?;
    let get = |k: &str| Money::from_dong(b.get(k).copied().unwrap_or(0));
    Ok(format!(
        "customer {} | clearing {} | revenue {} | driver {}",
        get("customer:CU-001"),
        get("psp:vixanh:clearing"),
        get("platform:revenue"),
        get("driver:DR-001"),
    ))
}

async fn account(pool: &PgPool, id: &str) -> Result<i64> {
    Ok(ledger::balances(pool).await?.get(id).copied().unwrap_or(0))
}

async fn payment(pool: &PgPool) -> Result<(String, i64)> {
    let row = sqlx::query("select status, refunded_dong from payments where trip_id = 'TN-0001'")
        .fetch_one(pool)
        .await?;
    Ok((row.get("status"), row.get("refunded_dong")))
}

fn lines_text(lines: &[(String, i64)]) -> String {
    lines
        .iter()
        .map(|(a, n)| format!("{a} {}", Money::from_dong(*n)))
        .collect::<Vec<_>>()
        .join(", ")
}

// Thí nghiệm 1: đảo toàn phần và không đảo được hai lần.
async fn full_reversal(db: LabDb) -> Result<Checks> {
    let mut c = Checks::default();
    let pool = &db.pool;
    let cap = ledger::capture_trip(pool, &TRIP).await?;
    println!("-- 1. Đảo toàn phần");
    println!(
        "capture JE-{:06}, split JE-{:06}: {}",
        cap.capture.id,
        cap.split.id,
        balance_line(pool).await?
    );
    let svc = RefundService::new(pool.clone());
    let rev = svc.reverse_trip("TN-0001", "fin.binh").await?;
    println!(
        "đảo split -> JE-{:06}, đảo capture -> JE-{:06}: {}",
        rev.split.id,
        rev.capture.id,
        balance_line(pool).await?
    );
    let (status, refunded) = payment(pool).await?;
    println!("payment {status}, đã hoàn {refunded} đồng");
    c.expect(
        status == "REFUNDED" && refunded == 25_500,
        "trạng thái REFUNDED, hoàn 25.500",
    );
    c.expect(
        ledger::balances(pool).await?.values().all(|v| *v == 0),
        "mọi tài khoản về 0 sau đảo toàn phần",
    );

    let links: Vec<(i64, Option<i64>)> =
        sqlx::query("select id, reversal_of from journal_entries order by id")
            .fetch_all(pool)
            .await?
            .iter()
            .map(|r| (r.get("id"), r.get("reversal_of")))
            .collect();
    println!("reversal_of: {links:?}");
    c.expect(
        links == [(1, None), (2, None), (3, Some(2)), (4, Some(1))],
        "bút toán 3 đảo 2, bút toán 4 đảo 1",
    );

    // Lần 1: gọi lại cả chuyến.
    let again = svc.reverse_trip("TN-0001", "fin.binh").await;
    println!(
        "đảo chuyến lần hai: {}",
        again
            .as_ref()
            .err()
            .map_or("không lỗi".to_string(), ToString::to_string)
    );
    c.expect(
        matches!(again, Err(RefundError::AlreadyReversed(_))),
        "đảo chuyến lần hai bị từ chối",
    );

    // Lần 2: gọi thẳng reverse_entry cùng một bút toán gốc: cùng key, replay.
    let mut tx = pool.begin().await?;
    let replay = reverse_entry(&mut tx, cap.split.id).await?;
    tx.rollback().await?;
    println!(
        "reverse_entry(JE-{:06}) lại: JE-{:06}, replayed={}",
        cap.split.id, replay.id, replay.replayed
    );
    c.expect(
        replay.replayed && replay.id == rev.split.id,
        "đảo lại cùng bút toán là replay",
    );

    // Lần 3: đảo một bút toán đảo.
    let mut tx = pool.begin().await?;
    let nested = reverse_entry(&mut tx, rev.split.id).await;
    tx.rollback().await?;
    println!(
        "đảo bút toán đảo: {}",
        nested
            .as_ref()
            .err()
            .map_or("không lỗi".to_string(), ToString::to_string)
    );
    c.expect(
        matches!(nested, Err(RefundError::CannotReverseReversal(_))),
        "không đảo bút toán đảo",
    );

    // Lần 4: key khác nhưng cùng reversal_of: chỉ mục duy nhất chặn ở tầng dữ liệu.
    let raw = sqlx::query("insert into journal_entries (idempotency_key, reversal_of) values ('thu-cong:dao-lai', $1)")
        .bind(cap.split.id)
        .execute(pool)
        .await;
    let code = raw
        .as_ref()
        .err()
        .and_then(|e| e.as_database_error())
        .map(|d| {
            (
                d.code().map(|x| x.to_string()),
                d.constraint().map(str::to_string),
            )
        });
    println!("chèn tay một bút toán đảo thứ hai: {code:?}");
    c.expect(
        code == Some((
            Some("23505".into()),
            Some("journal_entries_one_reversal".into()),
        )),
        "chỉ mục journal_entries_one_reversal chặn đảo hai lần",
    );
    Ok(c)
}

// Thí nghiệm 2: refund một phần qua maker-checker, tách theo tỷ lệ.
async fn partial_refunds(db: LabDb) -> Result<Checks> {
    let mut c = Checks::default();
    let pool = &db.pool;
    ledger::capture_trip(pool, &TRIP).await?;
    let svc = RefundService::new(pool.clone());
    println!("-- 2. Refund một phần, maker-checker");

    let r1 = svc
        .create_request(
            "TN-0001",
            Money::from_dong(10_000),
            ReasonCode::FareDispute,
            "cs.lan",
        )
        .await?;
    let (status, _) = payment(pool).await?;
    println!("cs.lan tạo yêu cầu {r1}: payment {status}");
    c.expect(
        status == "REFUND_PENDING",
        "tạo yêu cầu đưa payment sang REFUND_PENDING",
    );

    let own = svc.approve(r1, "cs.lan").await;
    println!(
        "cs.lan tự duyệt: {}",
        own.as_ref()
            .err()
            .map_or("không lỗi".to_string(), ToString::to_string)
    );
    c.expect(
        matches!(own, Err(RefundError::SelfApproval { .. })),
        "người tạo không tự duyệt",
    );

    // Bỏ qua code Rust, ghi thẳng SQL: CHECK ở database vẫn chặn.
    let raw = sqlx::query(
        "update refund_requests set status = 'APPROVED', decided_by = created_by, decided_at = now(), entry_id = 1 where id = $1",
    )
    .bind(r1)
    .execute(pool)
    .await;
    let named = raw
        .as_ref()
        .err()
        .and_then(|e| e.as_database_error())
        .and_then(|d| d.constraint().map(str::to_string));
    println!("UPDATE thẳng để người duyệt = người tạo: constraint {named:?}");
    c.expect(
        named.as_deref() == Some("checker_differs_from_maker"),
        "CHECK checker_differs_from_maker chặn ở DB",
    );

    let a1 = svc.approve(r1, "fin.binh").await?;
    let e1 = entry_lines(&mut *pool.acquire().await?, a1.entry_id).await?;
    println!(
        "fin.binh duyệt: JE-{:06} [{}], payment {}",
        a1.entry_id,
        lines_text(&e1),
        a1.status
    );
    c.expect(
        e1 == [
            ("customer:CU-001".to_string(), 10_000),
            ("platform:revenue".to_string(), -2_000),
            ("driver:DR-001".to_string(), -8_000),
        ],
        "hoàn 10.000: khách +10.000, doanh thu -2.000, tài xế -8.000",
    );
    c.expect(
        a1.status == PaymentState::Captured && !a1.replayed,
        "hoàn một phần xong, payment về CAPTURED",
    );

    let a1b = svc.approve(r1, "fin.binh").await?;
    let a1c = svc.approve(r1, "fin.cuong").await?;
    println!(
        "duyệt lại: replayed={} (cùng người), replayed={} (người khác), cùng JE-{:06}",
        a1b.replayed, a1c.replayed, a1b.entry_id
    );
    c.expect(
        a1b.replayed && a1c.replayed && a1b.entry_id == a1.entry_id,
        "duyệt lặp lại idempotent",
    );
    let n: i64 = sqlx::query_scalar(
        "select count(*) from journal_entries where idempotency_key like 'trip:TN-0001:refund:%'",
    )
    .fetch_one(pool)
    .await?;
    c.expect(n == 1, "chỉ một bút toán hoàn dù duyệt ba lần");

    let r2 = svc
        .create_request(
            "TN-0001",
            Money::from_dong(15_500),
            ReasonCode::TripCancelledByDriver,
            "cs.lan",
        )
        .await?;
    let a2 = svc.approve(r2, "fin.binh").await?;
    let e2 = entry_lines(&mut *pool.acquire().await?, a2.entry_id).await?;
    println!(
        "yêu cầu {r2} (15.500) duyệt: JE-{:06} [{}], payment {}",
        a2.entry_id,
        lines_text(&e2),
        a2.status
    );
    c.expect(
        e2 == [
            ("customer:CU-001".to_string(), 15_500),
            ("platform:revenue".to_string(), -3_100),
            ("driver:DR-001".to_string(), -12_400),
        ],
        "hoàn 15.500: doanh thu -3.100, tài xế -12.400 (tổng hai lần: phí 5.100)",
    );
    c.expect(
        a2.status == PaymentState::Refunded,
        "hoàn đủ 25.500: REFUNDED",
    );
    println!("sau hai lần hoàn: {}", balance_line(pool).await?);
    c.expect(
        ledger::balances(pool).await?.values().all(|v| *v == 0),
        "mọi tài khoản về 0",
    );

    let over = svc
        .create_request(
            "TN-0001",
            Money::from_dong(1),
            ReasonCode::DuplicateCharge,
            "cs.lan",
        )
        .await;
    println!(
        "yêu cầu thêm 1 đồng khi đã REFUNDED: {}",
        over.as_ref()
            .err()
            .map_or("không lỗi".to_string(), ToString::to_string)
    );
    c.expect(
        matches!(over, Err(RefundError::PaymentNotRefundable { .. })),
        "không hoàn thêm khi đã REFUNDED",
    );

    println!("audit:");
    let rows = sqlx::query("select actor, action, subject from audit_log order by id")
        .fetch_all(pool)
        .await?;
    let mut actions = Vec::new();
    for r in &rows {
        let (a, b, s): (String, String, String) =
            (r.get("actor"), r.get("action"), r.get("subject"));
        println!("  {a:<12} {b:<17} {s}");
        actions.push(b);
    }
    c.expect(
        actions.iter().any(|a| a == "APPROVAL_DENIED"),
        "audit giữ lần tự duyệt bị từ chối",
    );
    c.expect(
        actions.iter().filter(|a| *a == "REFUND_APPROVED").count() == 2,
        "audit có hai lần duyệt",
    );
    Ok(c)
}

// Thí nghiệm 3: hai yêu cầu đồng thời, tổng vượt số đã thu.
async fn concurrent_refunds(guard: Guard) -> Result<Checks> {
    with_lab(move |db| async move {
        let mut c = Checks::default();
        let pool = db.pool.clone();
        ledger::capture_trip(&pool, &TRIP).await?;
        let svc = RefundService::new(pool.clone());
        let r1 = svc.create_request("TN-0001", Money::from_dong(15_000), ReasonCode::FareDispute, "cs.lan").await?;
        let r2 = svc.create_request("TN-0001", Money::from_dong(15_000), ReasonCode::DuplicateCharge, "cs.lan").await?;

        let gate = Arc::new(Barrier::new(2));
        let sync = |g: &Arc<Barrier>| match guard {
            Guard::Locked => SyncPoint::Start(Arc::clone(g)),
            Guard::Unlocked => SyncPoint::AfterRead(Arc::clone(g)),
        };
        let (s1, s2) = (svc.clone(), svc.clone());
        let (sy1, sy2) = (sync(&gate), sync(&gate));
        let t1 = tokio::spawn(async move { s1.approve_with(r1, "fin.binh", guard, sy1).await });
        let t2 = tokio::spawn(async move { s2.approve_with(r2, "fin.cuong", guard, sy2).await });
        let results = [t1.await?, t2.await?];

        let ok = results.iter().filter(|r| r.is_ok()).count();
        let loser = results.iter().find_map(|r| r.as_ref().err());
        let (status, refunded) = payment(&pool).await?;
        let entries: i64 = sqlx::query_scalar("select count(*) from journal_entries where idempotency_key like 'trip:TN-0001:refund:%'")
            .fetch_one(&pool)
            .await?;
        println!("-- 3. Hai yêu cầu 15.000 duyệt đồng thời ({guard:?})");
        let loser_text = match loser {
            Some(e) if e.sqlstate().is_some() => {
                format!("lỗi database, SQLSTATE {:?}, constraint {:?}", e.sqlstate(), e.constraint())
            }
            Some(e) => e.to_string(),
            None => "không".to_string(),
        };
        println!("thành công {ok}/2; bên thua: {loser_text}");
        println!("đã hoàn {refunded}, bút toán hoàn {entries}, payment {status}, customer {}", Money::from_dong(account(&pool, "customer:CU-001").await?));
        c.expect(ok == 1 && refunded == 15_000 && entries == 1, "đúng một bên thắng, tổng hoàn 15.000");
        c.expect(account(&pool, "customer:CU-001").await? == -10_500, "khách còn -10.500, không được hoàn quá số đã thu");
        match guard {
            Guard::Locked => c.expect(
                matches!(loser, Some(RefundError::ExceedsRefundable { refundable, .. }) if refundable.dong() == 10_500),
                "bên thua nhận ExceedsRefundable (còn 10.500)",
            ),
            Guard::Unlocked => {
                c.expect(
                    loser.and_then(RefundError::sqlstate).as_deref() == Some("23514")
                        && loser.and_then(RefundError::constraint).as_deref() == Some("refund_within_captured"),
                    "thiếu khoá: CHECK refund_within_captured (23514) là chốt chặn",
                );
            }
        }
        // Yêu cầu bên thua vẫn PENDING: transaction đã rollback trọn vẹn.
        let pending: i64 = sqlx::query_scalar("select count(*) from refund_requests where status = 'PENDING'").fetch_one(&pool).await?;
        c.expect(pending == 1, "yêu cầu bên thua còn PENDING");
        Ok(c)
    })
    .await
}

// Thí nghiệm 3b: hai checker cùng duyệt một yêu cầu cùng lúc.
async fn concurrent_same_request(db: LabDb) -> Result<Checks> {
    let mut c = Checks::default();
    let pool = db.pool.clone();
    ledger::capture_trip(&pool, &TRIP).await?;
    let svc = RefundService::new(pool.clone());
    let r = svc
        .create_request(
            "TN-0001",
            Money::from_dong(10_000),
            ReasonCode::FareDispute,
            "cs.lan",
        )
        .await?;
    let gate = Arc::new(Barrier::new(2));
    let (a, b) = (svc.clone(), svc.clone());
    let (ga, gb) = (Arc::clone(&gate), Arc::clone(&gate));
    let t1 = tokio::spawn(async move {
        a.approve_with(r, "fin.binh", Guard::Locked, SyncPoint::Start(ga))
            .await
    });
    let t2 = tokio::spawn(async move {
        b.approve_with(r, "fin.cuong", Guard::Locked, SyncPoint::Start(gb))
            .await
    });
    let (x, y) = (t1.await??, t2.await??);
    let fresh = [&x, &y].iter().filter(|a| !a.replayed).count();
    let (_, refunded) = payment(&pool).await?;
    println!("-- 3b. Hai checker duyệt cùng một yêu cầu đồng thời");
    println!(
        "bản ghi mới {fresh}, replay {}, cùng JE-{:06}, đã hoàn {refunded}",
        2 - fresh,
        x.entry_id
    );
    c.expect(
        fresh == 1 && x.entry_id == y.entry_id && refunded == 10_000,
        "một lần hoàn duy nhất",
    );
    Ok(c)
}

// Thí nghiệm 4: chargeback, ba cách kết thúc.
async fn chargebacks() -> Result<Checks> {
    let mut c = Checks::default();
    println!("-- 4. Chargeback từ VíXanh");
    for (label, case) in [
        ("thua, thu hồi từ tài xế", 0),
        ("thua, nền tảng chịu", 1),
        ("thắng", 2),
    ] {
        let sum = with_lab(move |db| async move {
            let mut c = Checks::default();
            let pool = db.pool.clone();
            ledger::capture_trip(&pool, &TRIP).await?;
            let svc = ChargebackService::new(pool.clone());
            let opened = svc
                .open(
                    "VX-CB-9001",
                    "TN-0001",
                    Money::from_dong(25_500),
                    ReasonCode::Fraud,
                )
                .await?;
            let again = svc
                .open(
                    "VX-CB-9001",
                    "TN-0001",
                    Money::from_dong(25_500),
                    ReasonCode::Fraud,
                )
                .await?;
            let mid = balance_line(&pool).await?;
            if case == 0 {
                println!("PSP rút 25.500 (JE-{:06}): {mid}", opened.entry_id);
                println!(
                    "webhook gửi lại: replayed={}, cùng JE-{:06}",
                    again.replayed, again.entry_id
                );
                c.expect(
                    again.replayed && again.entry_id == opened.entry_id,
                    "chargeback gửi lại không nhân đôi",
                );
                c.expect(
                    account(&pool, "psp:vixanh:clearing").await? == -25_500,
                    "clearing âm 25.500: nền tảng chịu tạm",
                );
                c.expect(
                    account(&pool, "driver:DR-001").await? == 20_400,
                    "tài xế vẫn giữ 20.400 lúc chưa kết luận",
                );
            }
            match case {
                0 => {
                    svc.resolve_lost("VX-CB-9001", LossPolicy::RecoverFromDriver, "fin.binh")
                        .await?;
                    c.expect(
                        account(&pool, "driver:DR-001").await? == 0
                            && account(&pool, "platform:revenue").await? == 0,
                        "thu hồi: tài xế và doanh thu về 0",
                    );
                }
                1 => {
                    svc.resolve_lost("VX-CB-9001", LossPolicy::PlatformAbsorbs, "fin.binh")
                        .await?;
                    c.expect(
                        account(&pool, "platform:revenue").await? == -20_400
                            && account(&pool, "driver:DR-001").await? == 20_400,
                        "nền tảng chịu: doanh thu -20.400, tài xế giữ 20.400",
                    );
                }
                _ => {
                    let rev = svc.resolve_won("VX-CB-9001", "fin.binh").await?;
                    let link: Option<i64> =
                        sqlx::query_scalar("select reversal_of from journal_entries where id = $1")
                            .bind(rev.id)
                            .fetch_one(&pool)
                            .await?;
                    c.expect(
                        link == Some(opened.entry_id),
                        "thắng: đảo đúng bút toán chargeback",
                    );
                    c.expect(
                        account(&pool, "psp:vixanh:clearing").await? == 0,
                        "thắng: clearing về 0",
                    );
                }
            }
            c.expect(
                ledger::grand_total(&pool).await? == 0,
                "tổng posting của sổ bằng 0",
            );
            println!("{label:<24} -> {}", balance_line(&pool).await?);
            Ok(c)
        })
        .await?;
        c.0.extend(sum.0);
    }
    Ok(c)
}

// Thí nghiệm 5: reason code trong enum khớp bảng, và sổ cái cân.
async fn reason_codes(db: LabDb) -> Result<Checks> {
    let mut c = Checks::default();
    let rows = sqlx::query("select code, kind from reason_codes order by code")
        .fetch_all(&db.pool)
        .await?;
    let mut table: Vec<(String, String)> = rows
        .iter()
        .map(|r| (r.get("code"), r.get("kind")))
        .collect();
    let mut en: Vec<(String, String)> = ReasonCode::ALL
        .iter()
        .map(|r| (r.as_str().to_string(), r.kind().as_str().to_string()))
        .collect();
    table.sort();
    en.sort();
    println!(
        "-- 5. Reason code: {} dòng trong bảng, {} biến thể enum, khớp: {}",
        table.len(),
        en.len(),
        table == en
    );
    c.expect(table == en, "enum ReasonCode khớp bảng reason_codes");
    let bad = sqlx::query("insert into refund_requests (trip_id, amount_dong, reason_code, created_by) values ('TN-0001', 1, 'LY_DO_LA', 'cs.lan')")
        .execute(&db.pool)
        .await;
    c.expect(bad.is_err(), "reason code lạ bị từ chối");
    Ok(c)
}

#[tokio::main]
async fn main() -> Result<()> {
    let verify = std::env::args().any(|a| a == "--verify");
    let mut all = Checks::default();
    all.0.extend(with_lab(full_reversal).await?.0);
    all.0.extend(with_lab(partial_refunds).await?.0);
    all.0.extend(concurrent_refunds(Guard::Locked).await?.0);
    all.0.extend(concurrent_refunds(Guard::Unlocked).await?.0);
    all.0.extend(with_lab(concurrent_same_request).await?.0);
    all.0.extend(chargebacks().await?.0);
    all.0.extend(with_lab(reason_codes).await?.0);

    if all.0.is_empty() {
        if verify {
            println!("VERIFY OK");
        }
        Ok(())
    } else {
        for f in &all.0 {
            eprintln!("SAI: {f}");
        }
        std::process::exit(1);
    }
}
