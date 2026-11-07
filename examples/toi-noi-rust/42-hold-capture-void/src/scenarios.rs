//! Các thí nghiệm của bài 42. Mỗi hàm chạy trên một `Lab` mới (schema riêng), in kết quả
//! và `ensure!` đúng những con số mà bài viết nêu. Không hàm nào assert thời gian chạy.

use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, bail, ensure};
use jiff::SignedDuration;
use ledger_core::Money;
use sqlx::postgres::PgPool;
use sqlx::{Row, query};
use tokio::sync::Barrier;

use crate::clock::{Clock, ManualClock};
use crate::hold::{self, HoldError, HoldStatus, Locked};
use crate::lab::Lab;
use crate::ledger::{PSP_CLEARING, fixture_topup};
use crate::store::{
    CaptureOutcome, Funds, Gate, HoldOutcome, LockMode, NewHold, Pause, VoidOutcome, capture_hold,
    create_hold, expire_due, funds, get_hold, void_hold,
};

/// Cước chuyến `TN-0001`: 12.000 cho 2 km đầu + 3 km × 4.500.
fn fare() -> Money {
    Money::from_dong(25_500)
}

fn wallet() -> Money {
    Money::from_dong(40_000)
}

const CU1: &str = "customer:CU-001";
const TTL: SignedDuration = SignedDuration::from_mins(15);

fn req<'a>(account: &'a str, amount: Money, key: &'a str) -> NewHold<'a> {
    NewHold {
        account,
        amount,
        key,
        ttl: TTL,
    }
}

fn show(f: Funds) -> String {
    format!(
        "số dư {}, giữ {}, khả dụng {}",
        f.balance, f.held, f.available
    )
}

// Lỗi in bằng Display (thông điệp tiếng Việt), thành công in bằng Debug.
fn text<T: Debug>(result: &Result<T, HoldError>) -> String {
    match result {
        Ok(value) => format!("{value:?}"),
        Err(err) => format!("Err({err})"),
    }
}

async fn counts(pool: &PgPool) -> Result<(i64, i64)> {
    let row = query(
        "SELECT (SELECT count(*) FROM journal_entries)::bigint, \
                (SELECT count(*) FROM postings)::bigint",
    )
    .fetch_one(pool)
    .await?;
    Ok((row.get(0), row.get(1)))
}

/// Bất biến sổ: tổng mọi posting bằng 0 và `account_balances` khớp tổng posting của từng tài khoản.
async fn ensure_ledger_consistent(pool: &PgPool) -> Result<()> {
    let row = query(
        "SELECT (SELECT COALESCE(SUM(amount_dong), 0) FROM postings)::bigint AS total, \
                (SELECT count(*) FROM account_balances b WHERE b.balance_dong <> \
                    COALESCE((SELECT SUM(p.amount_dong) FROM postings p \
                              WHERE p.account_id = b.account_id), 0))::bigint AS drift",
    )
    .fetch_one(pool)
    .await?;
    let (total, drift): (i64, i64) = (row.get("total"), row.get("drift"));
    ensure!(total == 0, "tổng posting = {total}");
    ensure!(drift == 0, "{drift} tài khoản có số dư lệch posting");
    Ok(())
}

/// Số dư sổ cái của một tài khoản bất kỳ, kể cả clearing.
async fn balance_of(pool: &PgPool, account: &str) -> Result<Money> {
    Ok(funds(pool, account).await?.balance)
}

async fn setup(lab: &Lab, accounts: &[&str]) -> Result<()> {
    for account in accounts {
        fixture_topup(&lab.pool, account, wallet()).await?;
    }
    Ok(())
}

// ------------------------------------------------------------ 1. Hold không ghi bút toán

/// Hold chỉ giảm khả dụng: không bút toán, không posting, số dư sổ cái giữ nguyên.
pub async fn hold_does_not_post(lab: &Lab) -> Result<()> {
    println!("== 1. Hold không ghi bút toán ==");
    let clock = ManualClock::lab_start();
    setup(lab, &[CU1]).await?;
    let before = funds(&lab.pool, CU1).await?;
    let counts_before = counts(&lab.pool).await?;
    println!(
        "nạp ví (fixture)    : {}; {} bút toán, {} posting",
        show(before),
        counts_before.0,
        counts_before.1
    );
    let clearing = balance_of(&lab.pool, PSP_CLEARING).await?;
    println!("clearing sau nạp ví : {clearing}");
    ensure!(
        clearing == Money::from_dong(-40_000),
        "fixture làm clearing âm"
    );

    let first = create_hold(
        &lab.pool,
        &clock,
        &req(CU1, fare(), "trip:TN-0001:hold"),
        LockMode::ForUpdate,
        Gate::Open,
    )
    .await;
    println!("hold TN-0001 25.500 : {}", text(&first));
    let after = funds(&lab.pool, CU1).await?;
    let counts_after = counts(&lab.pool).await?;
    println!(
        "sau hold            : {}; {} bút toán, {} posting",
        show(after),
        counts_after.0,
        counts_after.1
    );
    ensure!(first.is_ok_and(|o| o == HoldOutcome::Created { id: 1 }));
    ensure!(counts_after == counts_before, "hold không được ghi sổ");
    ensure!(after.balance == wallet() && after.held == fare());
    ensure!(after.available == Money::from_dong(14_500));
    ensure!(
        balance_of(&lab.pool, PSP_CLEARING).await? == clearing,
        "hold không đụng clearing"
    );

    let replay = create_hold(
        &lab.pool,
        &clock,
        &req(CU1, fare(), "trip:TN-0001:hold"),
        LockMode::ForUpdate,
        Gate::Open,
    )
    .await;
    let held_after_replay = funds(&lab.pool, CU1).await?.held;
    println!(
        "gửi lại cùng key    : {}, giữ vẫn {}",
        text(&replay),
        held_after_replay
    );
    ensure!(replay.is_ok_and(|o| o == HoldOutcome::Replayed { id: 1 }));
    ensure!(held_after_replay == fare());

    let too_much = create_hold(
        &lab.pool,
        &clock,
        &req(CU1, Money::from_dong(14_501), "trip:TN-0002:hold"),
        LockMode::ForUpdate,
        Gate::Open,
    )
    .await;
    println!("hold thêm 14.501    : {}", text(&too_much));
    ensure!(matches!(
        too_much,
        Err(HoldError::InsufficientAvailable { available, requested })
            if available == Money::from_dong(14_500) && requested == Money::from_dong(14_501)
    ));
    ensure_ledger_consistent(&lab.pool).await
}

// ------------------------------------------------------------ 2. Capture toàn phần + idempotent

/// Capture toàn phần ghi đúng một bút toán; capture lặp lại không ghi thêm.
pub async fn capture_full(lab: &Lab) -> Result<()> {
    println!("== 2. Capture toàn phần và gọi lặp ==");
    let clock = ManualClock::lab_start();
    setup(lab, &[CU1]).await?;
    let id = create_hold(
        &lab.pool,
        &clock,
        &req(CU1, fare(), "trip:TN-0001:hold"),
        LockMode::ForUpdate,
        Gate::Open,
    )
    .await?
    .id();
    let before = funds(&lab.pool, CU1).await?;
    println!("trước capture       : {}", show(before));

    let key = "trip:TN-0001:capture";
    let first = capture_hold(&lab.pool, &clock, id, fare(), key).await;
    let after = funds(&lab.pool, CU1).await?;
    let (entries, postings) = counts(&lab.pool).await?;
    let row = get_hold(&lab.pool, id).await?;
    println!("capture 25.500      : {}", text(&first));
    println!(
        "sau capture         : {}; {entries} bút toán, {postings} posting",
        show(after)
    );
    println!(
        "hold                : {}, captured {}, nhả {}",
        row.status.as_str(),
        row.captured,
        row.released()
    );
    let clearing = balance_of(&lab.pool, PSP_CLEARING).await?;
    println!("clearing sau capture: {clearing}");
    ensure!(clearing == Money::from_dong(-14_500), "-40.000 + 25.500");
    ensure!(first.is_ok_and(|o| o == CaptureOutcome::Captured { entry_id: 2 }));
    ensure!((entries, postings) == (2, 4));
    ensure!(after.balance == Money::from_dong(14_500) && after.held == Money::ZERO);
    // Capture đổi số dư và đóng hold trong một transaction: khả dụng không nhảy.
    ensure!(after.available == before.available);
    ensure!(row.status == HoldStatus::Captured && row.captured == fare());

    let replay = capture_hold(&lab.pool, &clock, id, fare(), key).await;
    let counts_replay = counts(&lab.pool).await?;
    println!(
        "capture lặp, cùng key: {}; vẫn {} bút toán",
        text(&replay),
        counts_replay.0
    );
    ensure!(replay.is_ok_and(|o| o == CaptureOutcome::Replayed { entry_id: 2 }));
    ensure!(counts_replay == (2, 4));

    let other_amount = capture_hold(&lab.pool, &clock, id, Money::from_dong(12_000), key).await;
    println!("cùng key, số khác   : {}", text(&other_amount));
    ensure!(matches!(
        other_amount,
        Err(HoldError::IdempotencyConflict { .. })
    ));

    let other_key = capture_hold(&lab.pool, &clock, id, fare(), "trip:TN-0001:capture:2").await;
    println!("key khác, hold đã đóng: {}", text(&other_key));
    ensure!(matches!(
        other_key,
        Err(HoldError::NotActive {
            status: HoldStatus::Captured,
            ..
        })
    ));

    let void_after = void_hold(&lab.pool, id).await;
    println!("void sau capture    : {}", text(&void_after));
    ensure!(matches!(void_after, Err(HoldError::NotActive { .. })));
    ensure_ledger_consistent(&lab.pool).await
}

// ------------------------------------------------------------ 3. Capture một phần

/// Capture ít hơn số giữ: phần dư được nhả; capture nhiều hơn bị từ chối.
pub async fn capture_partial(lab: &Lab) -> Result<()> {
    println!("== 3. Capture một phần ==");
    let clock = ManualClock::lab_start();
    setup(lab, &[CU1]).await?;
    let id = create_hold(
        &lab.pool,
        &clock,
        &req(CU1, fare(), "trip:TN-0001:hold"),
        LockMode::ForUpdate,
        Gate::Open,
    )
    .await?
    .id();
    println!(
        "giữ 25.500          : {}",
        show(funds(&lab.pool, CU1).await?)
    );

    // Chuyến dừng sau 2 km: cước chỉ còn 12.000.
    let partial = Money::from_dong(12_000);
    let result = capture_hold(&lab.pool, &clock, id, partial, "trip:TN-0001:capture").await;
    let after = funds(&lab.pool, CU1).await?;
    let row = get_hold(&lab.pool, id).await?;
    println!("capture 12.000      : {}", text(&result));
    println!("sau capture         : {}", show(after));
    println!(
        "hold                : {}, giữ {}, captured {}, nhả {}",
        row.status.as_str(),
        row.amount,
        row.captured,
        row.released()
    );
    ensure!(result.is_ok());
    ensure!(after.balance == Money::from_dong(28_000) && after.held == Money::ZERO);
    ensure!(after.available == Money::from_dong(28_000));
    ensure!(row.released() == Money::from_dong(13_500));

    // Hold thứ hai: capture quá số giữ.
    let id2 = create_hold(
        &lab.pool,
        &clock,
        &req(CU1, fare(), "trip:TN-0002:hold"),
        LockMode::ForUpdate,
        Gate::Open,
    )
    .await?
    .id();
    let too_much = capture_hold(
        &lab.pool,
        &clock,
        id2,
        Money::from_dong(25_501),
        "trip:TN-0002:capture",
    )
    .await;
    println!("capture 25.501/25.500: {}", text(&too_much));
    ensure!(matches!(
        too_much,
        Err(HoldError::CaptureExceedsHold { .. })
    ));
    ensure!(get_hold(&lab.pool, id2).await?.status == HoldStatus::Active);
    ensure_ledger_consistent(&lab.pool).await
}

// ------------------------------------------------------------ 4. Void khi hủy chuyến

/// Hủy chuyến: void nhả toàn bộ, không bút toán; gọi lặp thành công; capture sau void bị từ chối.
pub async fn void_on_cancel(lab: &Lab) -> Result<()> {
    println!("== 4. Void khi hủy chuyến ==");
    let clock = ManualClock::lab_start();
    setup(lab, &[CU1]).await?;
    let id = create_hold(
        &lab.pool,
        &clock,
        &req(CU1, fare(), "trip:TN-0001:hold"),
        LockMode::ForUpdate,
        Gate::Open,
    )
    .await?
    .id();
    let held = funds(&lab.pool, CU1).await?;
    let entries_before = counts(&lab.pool).await?;

    let first = void_hold(&lab.pool, id).await;
    let after = funds(&lab.pool, CU1).await?;
    let entries_after = counts(&lab.pool).await?;
    println!("đang giữ            : {}", show(held));
    println!("void                : {}", text(&first));
    println!(
        "sau void            : {}; {} bút toán",
        show(after),
        entries_after.0
    );
    ensure!(first.is_ok_and(|o| o == VoidOutcome::Voided));
    ensure!(entries_after == entries_before, "void không ghi sổ");
    ensure!(after.available == wallet() && after.held == Money::ZERO);

    let again = void_hold(&lab.pool, id).await;
    println!("void lần hai        : {}", text(&again));
    ensure!(again.is_ok_and(|o| o == VoidOutcome::AlreadyVoided));

    let late = capture_hold(&lab.pool, &clock, id, fare(), "trip:TN-0001:capture").await;
    println!("capture sau void    : {}", text(&late));
    ensure!(matches!(
        late,
        Err(HoldError::NotActive {
            status: HoldStatus::Voided,
            ..
        })
    ));
    ensure!(counts(&lab.pool).await? == entries_before);
    ensure_ledger_consistent(&lab.pool).await
}

// ------------------------------------------------------------ 5. Hai hold đồng thời

/// Kết quả hai hold 25.500 cùng lúc trên ví 40.000.
#[derive(Debug)]
pub struct RaceReport {
    /// Số hold được tạo.
    pub created: usize,
    /// Số hold bị từ chối vì thiếu khả dụng.
    pub rejected: usize,
    /// Số dư và phần giữ sau cuộc đua.
    pub funds: Funds,
}

/// Hai hold 25.500 cho `customer:CU-001` (ví 40.000) chạy cùng lúc, đồng bộ bằng `Barrier`.
/// `LockMode::None` thì cả hai cùng thấy khả dụng 40.000 rồi cùng ghi; `ForUpdate` thì xếp hàng.
pub async fn race(lab: &Lab, mode: LockMode) -> Result<RaceReport> {
    setup(lab, &[CU1]).await?;
    let clock = Arc::new(ManualClock::lab_start());
    let start = Arc::new(Barrier::new(2));
    let after_check = Arc::new(Barrier::new(2));
    let mut tasks = Vec::new();
    for n in 1..=2 {
        let (pool, clock, start, after_check) = (
            lab.pool.clone(),
            clock.clone(),
            start.clone(),
            after_check.clone(),
        );
        tasks.push(tokio::spawn(async move {
            let key = format!("trip:TN-000{n}:hold");
            start.wait().await;
            // Không khóa: dừng giữa «kiểm» và «ghi» để chắc chắn cả hai đã kiểm xong.
            // Có khóa: dừng ở đây sẽ tự khóa chết (bên thứ hai chờ khóa, bên thứ nhất chờ Barrier).
            let gate = match mode {
                LockMode::None => Gate::Barrier(&after_check),
                LockMode::ForUpdate => Gate::Open,
            };
            create_hold(&pool, &*clock, &req(CU1, fare(), &key), mode, gate).await
        }));
    }
    let (mut created, mut rejected) = (0, 0);
    for task in tasks {
        match task.await? {
            Ok(HoldOutcome::Created { .. }) => created += 1,
            Err(HoldError::InsufficientAvailable { .. }) => rejected += 1,
            other => bail!("kết quả ngoài dự kiến: {other:?}"),
        }
    }
    Ok(RaceReport {
        created,
        rejected,
        funds: funds(&lab.pool, CU1).await?,
    })
}

/// Chạy `race` cả hai kiểu khóa, in và kiểm.
pub async fn race_report(mode: LockMode) -> Result<()> {
    let label = match mode {
        LockMode::None => "không khóa",
        LockMode::ForUpdate => "FOR UPDATE",
    };
    let r = crate::lab::with_lab(async move |lab| race(&lab, mode).await).await?;
    println!(
        "{label:11}: {} tạo được, {} bị từ chối; {}",
        r.created,
        r.rejected,
        show(r.funds)
    );
    match mode {
        LockMode::None => {
            ensure!(r.created == 2 && r.rejected == 0, "{r:?}");
            ensure!(r.funds.held == Money::from_dong(51_000), "{r:?}");
            ensure!(r.funds.available == Money::from_dong(-11_000), "{r:?}");
        }
        LockMode::ForUpdate => {
            ensure!(r.created == 1 && r.rejected == 1, "{r:?}");
            ensure!(r.funds.held == fare(), "{r:?}");
            ensure!(r.funds.available == Money::from_dong(14_500), "{r:?}");
        }
    }
    Ok(())
}

/// Chờ tới khi có một phiên của ca này đang bị chặn bởi khóa dòng trong câu `FOR UPDATE`.
/// Chờ theo điều kiện (poll), không theo đồng hồ: trần 60 giây chỉ để khỏi treo vô hạn.
async fn wait_until_blocked(pool: &PgPool, application_name: &str) -> Result<()> {
    for _ in 0..6_000 {
        let blocked: i64 = query(
            "SELECT count(*) FROM pg_stat_activity WHERE application_name = $1 \
             AND wait_event_type = 'Lock' AND query LIKE '%FOR UPDATE'",
        )
        .bind(application_name)
        .fetch_one(pool)
        .await?
        .get(0);
        if blocked >= 1 {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    bail!("không thấy phiên nào bị chặn bởi FOR UPDATE")
}

/// Bên A giữ khóa và dừng giữa «kiểm» và «ghi»; bên B phải ĐỨNG CHỜ ở khóa (đọc từ
/// `pg_stat_activity`); A commit thì B thấy hold của A và bị từ chối.
pub async fn lock_wait(lab: &Lab) -> Result<()> {
    println!("== 5b. Bên thứ hai chờ khóa ==");
    setup(lab, &[CU1]).await?;
    let clock = Arc::new(ManualClock::lab_start());
    let pause = Arc::new(Pause::default());
    let a = {
        let (pool, clock, pause) = (lab.pool.clone(), clock.clone(), pause.clone());
        tokio::spawn(async move {
            create_hold(
                &pool,
                &*clock,
                &req(CU1, fare(), "trip:TN-0001:hold"),
                LockMode::ForUpdate,
                Gate::Pause(&pause),
            )
            .await
        })
    };
    pause.reached.notified().await; // A đã giữ khóa và tính xong khả dụng
    let b = {
        let (pool, clock) = (lab.pool.clone(), clock.clone());
        tokio::spawn(async move {
            create_hold(
                &pool,
                &*clock,
                &req(CU1, fare(), "trip:TN-0002:hold"),
                LockMode::ForUpdate,
                Gate::Open,
            )
            .await
        })
    };
    wait_until_blocked(&lab.pool, &lab.schema).await?;
    println!("B đang chờ khóa của A (wait_event_type = Lock): true");
    ensure!(!b.is_finished(), "B không được xong khi A còn giữ khóa");
    pause.resume.notify_one();
    let (a, b) = (a.await?, b.await?);
    println!("A: {}", text(&a));
    println!("B: {}", text(&b));
    ensure!(a.is_ok_and(|o| matches!(o, HoldOutcome::Created { .. })));
    ensure!(matches!(
        b,
        Err(HoldError::InsufficientAvailable { available, requested })
            if available == Money::from_dong(14_500) && requested == fare()
    ));
    ensure_ledger_consistent(&lab.pool).await
}

/// Gộp khóa và `SUM` vào MỘT câu: B chờ khóa của A, nhưng snapshot của câu lệnh đã lấy từ lúc bắt
/// đầu nên `SUM` không thấy hold A vừa commit. Câu `SUM` riêng sau khóa thì thấy.
pub async fn combined_statement(lab: &Lab) -> Result<()> {
    println!("== 5c. Gộp khóa và SUM vào một câu ==");
    setup(lab, &[CU1]).await?;
    let clock = Arc::new(ManualClock::lab_start());
    let pause = Arc::new(Pause::default());
    let a = {
        let (pool, clock, pause) = (lab.pool.clone(), clock.clone(), pause.clone());
        tokio::spawn(async move {
            create_hold(
                &pool,
                &*clock,
                &req(CU1, fare(), "trip:TN-0001:hold"),
                LockMode::ForUpdate,
                Gate::Pause(&pause),
            )
            .await
        })
    };
    pause.reached.notified().await; // A đã giữ khóa, chưa chèn hold

    let pool = lab.pool.clone();
    let b = tokio::spawn(async move {
        let mut tx = pool.begin().await?;
        let combined = query(
            "SELECT b.balance_dong, COALESCE((SELECT SUM(h.amount_dong - h.captured_dong) \
             FROM holds h WHERE h.account_id = b.account_id AND h.status = 'ACTIVE'), 0)::bigint \
             FROM account_balances b WHERE b.account_id = $1 FOR UPDATE",
        )
        .bind(CU1)
        .fetch_one(&mut *tx)
        .await?;
        let stale: i64 = combined.get(1);
        // Vẫn giữ khóa; câu lệnh mới, snapshot mới.
        let fresh: i64 = query(
            "SELECT COALESCE(SUM(amount_dong - captured_dong), 0)::bigint FROM holds \
             WHERE account_id = $1 AND status = 'ACTIVE'",
        )
        .bind(CU1)
        .fetch_one(&mut *tx)
        .await?
        .get(0);
        tx.rollback().await?;
        Ok::<_, sqlx::Error>((stale, fresh))
    });
    wait_until_blocked(&lab.pool, &lab.schema).await?;
    pause.resume.notify_one();
    ensure!(a.await?.is_ok(), "A phải tạo được hold");
    let (stale, fresh) = b.await??;
    println!(
        "một câu, sau khi chờ khóa: SUM thấy {}",
        Money::from_dong(stale)
    );
    println!(
        "câu SUM riêng sau đó     : SUM thấy {}",
        Money::from_dong(fresh)
    );
    ensure!(stale == 0, "snapshot của câu lệnh có trước hold của A");
    ensure!(fresh == fare().dong());
    ensure_ledger_consistent(&lab.pool).await
}

/// `FOR UPDATE` trên hàng chưa tồn tại không khóa gì; ví chưa có hàng số dư thì hold bị từ chối rõ.
pub async fn wallet_without_balance_row(lab: &Lab) -> Result<()> {
    println!("== 5d. Ví chưa có hàng số dư ==");
    let clock = ManualClock::lab_start();
    let fresh = "customer:CU-004";
    query("INSERT INTO accounts (id, kind) VALUES ($1, 'liability')")
        .bind(fresh)
        .execute(&lab.pool)
        .await?;

    // Hai phiên cùng «khóa» một hàng chưa có. NOWAIT: nếu có khóa thật, phiên hai sẽ lỗi 55P03.
    let lock_sql =
        "SELECT balance_dong FROM account_balances WHERE account_id = $1 FOR UPDATE NOWAIT";
    let mut first = lab.pool.begin().await?;
    let mut second = lab.pool.begin().await?;
    let one = query(lock_sql)
        .bind(fresh)
        .fetch_all(&mut *first)
        .await?
        .len();
    let two = query(lock_sql)
        .bind(fresh)
        .fetch_all(&mut *second)
        .await?
        .len();
    first.rollback().await?;
    second.rollback().await?;
    println!("hai phiên FOR UPDATE NOWAIT: {one} hàng và {two} hàng, không ai bị chặn");
    ensure!((one, two) == (0, 0));

    let no_row = create_hold(
        &lab.pool,
        &clock,
        &req(fresh, fare(), "trip:TN-0040:hold"),
        LockMode::ForUpdate,
        Gate::Open,
    )
    .await;
    println!("hold khi chưa có hàng: {}", text(&no_row));
    ensure!(matches!(no_row, Err(HoldError::NoBalanceRow(_))));
    ensure!(matches!(
        funds(&lab.pool, fresh).await,
        Err(HoldError::NoBalanceRow(_))
    ));

    fixture_topup(&lab.pool, fresh, wallet()).await?; // upsert tạo hàng cùng bút toán
    let after_topup = create_hold(
        &lab.pool,
        &clock,
        &req(fresh, fare(), "trip:TN-0040:hold"),
        LockMode::ForUpdate,
        Gate::Open,
    )
    .await;
    println!("hold sau nạp ví      : {}", text(&after_topup));
    ensure!(after_topup.is_ok_and(|o| matches!(o, HoldOutcome::Created { .. })));
    ensure_ledger_consistent(&lab.pool).await
}

// ------------------------------------------------------------ 6. Capture và void cùng lúc

/// Capture và void cùng một hold, cùng lúc: đúng một bên thắng, trạng thái cuối nhất quán.
pub async fn capture_vs_void(lab: &Lab) -> Result<()> {
    println!("== 6. Capture và void cùng lúc ==");
    setup(lab, &[CU1]).await?;
    let clock = Arc::new(ManualClock::lab_start());
    let id = create_hold(
        &lab.pool,
        &*clock,
        &req(CU1, fare(), "trip:TN-0001:hold"),
        LockMode::ForUpdate,
        Gate::Open,
    )
    .await?
    .id();
    let start = Arc::new(Barrier::new(2));
    let cap = {
        let (pool, clock, start) = (lab.pool.clone(), clock.clone(), start.clone());
        tokio::spawn(async move {
            start.wait().await;
            capture_hold(&pool, &*clock, id, fare(), "trip:TN-0001:capture").await
        })
    };
    let void = {
        let (pool, start) = (lab.pool.clone(), start.clone());
        tokio::spawn(async move {
            start.wait().await;
            void_hold(&pool, id).await
        })
    };
    let (cap, void) = (cap.await?, void.await?);
    let row = get_hold(&lab.pool, id).await?;
    let f = funds(&lab.pool, CU1).await?;
    let (entries, _) = counts(&lab.pool).await?;
    // Kiểm bất biến, không kiểm bên nào thắng: thứ tự do lịch của máy quyết định.
    let one_winner = cap.is_ok() != void.is_ok();
    let consistent = match row.status {
        HoldStatus::Captured => {
            cap.is_ok() && entries == 2 && f.balance == Money::from_dong(14_500)
        }
        HoldStatus::Voided => void.is_ok() && entries == 1 && f.balance == wallet(),
        _ => false,
    };
    println!("đúng một bên thắng  : {one_winner}");
    println!("trạng thái nhất quán: {consistent}");
    ensure!(one_winner, "capture {cap:?}, void {void:?}");
    ensure!(consistent, "{row:?} {f:?} entries={entries}");
    ensure!(f.held == Money::ZERO);
    ensure_ledger_consistent(&lab.pool).await
}

// ------------------------------------------------------------ 7. Job hết hạn

/// Job hết hạn dùng đồng hồ truyền vào và `FOR UPDATE SKIP LOCKED`: bỏ qua hold đang bị capture giữ khóa.
pub async fn expiry_skip_locked(lab: &Lab) -> Result<()> {
    println!("== 7. Job hết hạn và SKIP LOCKED ==");
    let accounts = [CU1, "customer:CU-002", "customer:CU-003"];
    setup(lab, &accounts).await?;
    let early = ManualClock::lab_start();
    let mut ids = Vec::new();
    for (n, account) in accounts.iter().enumerate() {
        let key = format!("trip:TN-000{}:hold", n + 1);
        ids.push(
            create_hold(
                &lab.pool,
                &early,
                &req(account, fare(), &key),
                LockMode::ForUpdate,
                Gate::Open,
            )
            .await?
            .id(),
        );
    }
    println!("3 hold đang ACTIVE  : id {ids:?}");

    // Đồng hồ của job đã qua hạn cả ba hold; đồng hồ của capture thì chưa.
    let late = ManualClock::lab_start();
    late.advance(TTL + SignedDuration::from_secs(1));
    for id in &ids {
        ensure!(get_hold(&lab.pool, *id).await?.expires_at <= late.now());
    }

    // Capture của hold 1 đang chạy: đã khóa dòng, chưa commit.
    let mut tx = lab.pool.begin().await?;
    let Locked::Active(active) = hold::lock(&mut tx, ids[0]).await? else {
        bail!("hold 1 phải ACTIVE");
    };

    // Không có SKIP LOCKED thì job phải chờ; NOWAIT cho thấy dòng đang bị khóa (SQLSTATE 55P03).
    let probe = query("SELECT id FROM holds WHERE id = $1 FOR UPDATE NOWAIT")
        .bind(ids[0])
        .fetch_one(&lab.pool)
        .await;
    let code = probe.err().and_then(|e| {
        e.as_database_error()
            .and_then(|d| d.code().map(|c| c.into_owned()))
    });
    println!(
        "FOR UPDATE NOWAIT   : SQLSTATE {} (dòng đang bị giữ)",
        code.as_deref().unwrap_or("?")
    );
    ensure!(code.as_deref() == Some("55P03"));

    let first_run = expire_due(&lab.pool, &late, 100).await?;
    let still = get_hold(&lab.pool, ids[0]).await?.status;
    println!(
        "job lần 1           : hết hạn id {first_run:?}; hold {} vẫn {}",
        ids[0],
        still.as_str()
    );
    ensure!(first_run == ids[1..]);
    ensure!(still == HoldStatus::Active);

    // Capture hoàn tất theo đồng hồ của nó (chưa quá hạn) rồi commit.
    let entry = active
        .capture(&mut tx, early.now(), fare(), "trip:TN-0001:capture")
        .await?;
    tx.commit().await?;
    let second_run = expire_due(&lab.pool, &late, 100).await?;
    println!("job lần 2           : hết hạn id {second_run:?}");
    ensure!(second_run.is_empty());

    let mut statuses = Vec::new();
    for id in &ids {
        statuses.push(get_hold(&lab.pool, *id).await?.status.as_str());
    }
    println!("trạng thái cuối     : {statuses:?} (capture ghi bút toán {entry})");
    ensure!(statuses == ["CAPTURED", "EXPIRED", "EXPIRED"]);
    let cu2 = funds(&lab.pool, "customer:CU-002").await?;
    println!("CU-002 sau hết hạn  : {}", show(cu2));
    ensure!(cu2.available == wallet() && cu2.held == Money::ZERO);
    ensure_ledger_consistent(&lab.pool).await
}

// ------------------------------------------------------------ 8. Capture đến sau khi hết hạn

/// Capture quá hạn bị từ chối theo đồng hồ truyền vào, cả khi job chưa chạy lẫn sau khi job chạy.
pub async fn late_capture(lab: &Lab) -> Result<()> {
    println!("== 8. Capture đến sau khi hết hạn ==");
    let accounts = [CU1, "customer:CU-002"];
    setup(lab, &accounts).await?;
    let clock = ManualClock::lab_start();
    let a = create_hold(
        &lab.pool,
        &clock,
        &req(CU1, fare(), "trip:TN-0001:hold"),
        LockMode::ForUpdate,
        Gate::Open,
    )
    .await?
    .id();
    let b = create_hold(
        &lab.pool,
        &clock,
        &req(accounts[1], fare(), "trip:TN-0002:hold"),
        LockMode::ForUpdate,
        Gate::Open,
    )
    .await?
    .id();

    clock.advance(TTL - SignedDuration::from_secs(1));
    let in_time = capture_hold(&lab.pool, &clock, b, fare(), "trip:TN-0002:capture").await;
    println!("t0+14:59, hold {b}    : {}", text(&in_time));
    ensure!(in_time.is_ok());

    clock.advance(SignedDuration::from_secs(1));
    let at_expiry = capture_hold(&lab.pool, &clock, a, fare(), "trip:TN-0001:capture").await;
    println!("t0+15:00, hold {a}    : {}", text(&at_expiry));
    ensure!(matches!(at_expiry, Err(HoldError::Expired { .. })));
    let still = funds(&lab.pool, CU1).await?;
    println!("job chưa chạy       : {}", show(still));
    ensure!(get_hold(&lab.pool, a).await?.status == HoldStatus::Active);
    // Hold quá hạn nhưng chưa được job đóng vẫn trừ khả dụng: nghiêng về phía an toàn.
    ensure!(still.held == fare() && still.available == Money::from_dong(14_500));

    let expired = expire_due(&lab.pool, &clock, 100).await?;
    let released = funds(&lab.pool, CU1).await?;
    println!("job chạy            : hết hạn id {expired:?}");
    println!("sau job             : {}", show(released));
    ensure!(expired == [a]);
    ensure!(released.available == wallet());

    let after_job = capture_hold(&lab.pool, &clock, a, fare(), "trip:TN-0001:capture").await;
    println!("capture sau job     : {}", text(&after_job));
    ensure!(matches!(
        after_job,
        Err(HoldError::NotActive {
            status: HoldStatus::Expired,
            ..
        })
    ));
    let (entries, _) = counts(&lab.pool).await?;
    println!("bút toán            : {entries} (2 nạp ví + 1 capture của hold {b})");
    ensure!(entries == 3);
    ensure_ledger_consistent(&lab.pool).await
}
