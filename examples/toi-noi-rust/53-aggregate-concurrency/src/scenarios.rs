//! Các thí nghiệm của bài 53. Mỗi hàm chạy trên một `Lab` mới (schema riêng), trả số liệu để
//! `main` in và để test `assert`. Không hàm nào assert thời gian chạy; chỉ `measure` có đo giờ.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Result, bail, ensure};
use ledger_core::Money;
use sqlx::postgres::PgPool;
use sqlx::{Executor, Row};
use tokio::sync::Barrier;

use crate::lab::Lab;
use crate::retry::{Jitter, RetryPolicy, Run, with_retry};
use crate::store::{
    Attempts, Audit, Gate, LockOrder, LockedLoad, Mode, Pause, StoreError, audit, ledger_balance,
    load_locked, open_wallet, snapshot, topup, transfer_once, transfer_retrying, withdraw_on,
    withdraw_once, withdraw_retrying,
};

/// Ví tài xế dùng cho thí nghiệm rút tiền.
pub const DR1: &str = "driver:DR-001";
const CU1: &str = "customer:CU-001";
const CU2: &str = "customer:CU-002";

/// Số tiền hợp lệ từ số đồng.
pub fn dong(n: i64) -> Money {
    Money::from_dong(n)
}

/// Tên chế độ để in.
pub fn mode_name(mode: Mode) -> &'static str {
    match mode {
        Mode::Unguarded => "không khóa",
        Mode::Optimistic => "optimistic",
        Mode::ForUpdate => "FOR UPDATE",
    }
}

/// Kết quả một lần chạy N task rút tiền.
#[derive(Debug)]
pub struct RaceReport {
    /// Số task rút thành công.
    pub ok: usize,
    /// Số task bị aggregate từ chối vì vượt hạn mức hoặc số dư.
    pub rejected: usize,
    /// Số task hết lượt thử mà vẫn xung đột.
    pub exhausted: usize,
    /// Số lần thử của từng task, đã sắp tăng dần.
    pub attempts: Vec<u32>,
    /// Số dư đã lưu của ví.
    pub balance: Money,
    /// Số đã rút đã lưu của ví.
    pub withdrawn: Money,
    /// `version` cuối.
    pub version: i64,
    /// Đối chiếu sổ với projection.
    pub audit: Audit,
}

impl RaceReport {
    /// Một dòng tóm tắt, dùng cho `--verify` và bài viết.
    pub fn line(&self) -> String {
        format!(
            "{} ok, {} bị từ chối, {} hết lượt; số lần thử {:?}; số dư {}, đã rút {}, version {}",
            self.ok,
            self.rejected,
            self.exhausted,
            self.attempts,
            self.balance,
            self.withdrawn,
            self.version
        )
    }

    /// Dòng đối chiếu sổ.
    pub fn audit_line(&self) -> String {
        format!(
            "tổng số dư mọi tài khoản {}, lệch projection {:?}, chi quá hạn mức {:?}",
            dong(self.audit.total_balance),
            self.audit.projection_mismatch,
            self.audit.over_limit
        )
    }
}

/// Chạy `tasks` task rút `amount` từ các ví `wallets` (chia vòng tròn). `Unguarded` và `Optimistic`
/// dùng `Barrier` để mọi task cùng nạp bản cũ rồi mới cùng ghi; `ForUpdate` không thể (task đầu giữ
/// khóa nên các task sau không bao giờ tới được barrier).
pub async fn run_withdrawals(
    pool: &PgPool,
    mode: Mode,
    wallets: &[String],
    tasks: usize,
    amount: Money,
    policy: RetryPolicy,
) -> Result<Vec<Run<i64>>> {
    let barrier = (mode != Mode::ForUpdate).then(|| Arc::new(Barrier::new(tasks)));
    let mut handles = Vec::with_capacity(tasks);
    for i in 0..tasks {
        let pool = pool.clone();
        let barrier = barrier.clone();
        let wallet = wallets[i % wallets.len()].clone();
        handles.push(tokio::spawn(async move {
            let key = format!("withdraw:{wallet}:{i}");
            withdraw_retrying(
                &pool,
                mode,
                &wallet,
                amount,
                &key,
                Attempts {
                    policy,
                    seed: i as u64 + 1,
                    barrier: barrier.as_deref(),
                },
            )
            .await
        }));
    }
    let mut runs = Vec::with_capacity(tasks);
    for handle in handles {
        runs.push(handle.await?);
    }
    Ok(runs)
}

fn tally(runs: &[Run<i64>]) -> Result<(usize, usize, usize, Vec<u32>)> {
    let (mut ok, mut rejected, mut exhausted) = (0, 0, 0);
    for run in runs {
        match &run.result {
            Ok(_) => ok += 1,
            Err(StoreError::Domain(_)) => rejected += 1,
            Err(StoreError::RetriesExhausted { .. }) => exhausted += 1,
            Err(other) => bail!("lỗi không mong đợi: {other}"),
        }
    }
    let mut attempts: Vec<u32> = runs.iter().map(|r| r.attempts).collect();
    attempts.sort_unstable();
    Ok((ok, rejected, exhausted, attempts))
}

/// Thí nghiệm chính: ví tài xế 150.000 đồng, hạn mức rút 100.000, `n` task cùng rút 20.400
/// (tiền chuyến `TN-0001` của tài xế). Tối đa 4 lần rút được (81.600); lần thứ 5 sẽ là 102.000.
pub async fn race(lab: &Lab, mode: Mode, n: usize) -> Result<RaceReport> {
    let pool = &lab.pool;
    open_wallet(pool, DR1, dong(150_000), dong(100_000)).await?;
    let runs = run_withdrawals(
        pool,
        mode,
        &[DR1.to_string()],
        n,
        dong(20_400),
        RetryPolicy::lab(),
    )
    .await?;
    let (ok, rejected, exhausted, attempts) = tally(&runs)?;
    let (balance, withdrawn, version) = snapshot(pool, DR1).await?;
    Ok(RaceReport {
        ok,
        rejected,
        exhausted,
        attempts,
        balance,
        withdrawn,
        version,
        audit: audit(pool).await?,
    })
}

/// Kết quả lịch xen kẽ có kiểm soát giữa một lần rút và một lần nạp tiền của writer khác.
#[derive(Debug)]
pub struct StaleReport {
    /// Số lần thử của lần rút.
    pub attempts: u32,
    /// Lần rút có thành công không.
    pub withdrew: bool,
    /// `balance_dong` đã lưu.
    pub stored_balance: Money,
    /// Số dư theo tổng posting (sự thật).
    pub ledger_balance: Money,
    /// Đối chiếu sổ.
    pub audit: Audit,
}

/// Lịch cố định, không phải cuộc đua may rủi: A nạp ví (version v), B nạp thêm 50.000 và commit,
/// rồi A mới ghi. Với `Optimistic`, A thấy 0 dòng, thử lại và đúng. Với `Unguarded`, A ghi đè số dư
/// tính từ bản cũ, 50.000 của B biến mất khỏi projection nhưng vẫn nằm trong posting.
pub async fn stale_write(lab: &Lab, mode: Mode) -> Result<StaleReport> {
    let pool = &lab.pool;
    open_wallet(pool, DR1, dong(150_000), dong(100_000)).await?;
    let pause = Arc::new(Pause::default());
    let worker = {
        let pool = pool.clone();
        let pause = Arc::clone(&pause);
        tokio::spawn(async move {
            with_retry(RetryPolicy::lab(), Jitter::new(1), async |attempt| {
                let gate = if attempt == 1 {
                    Gate::Pause(&pause)
                } else {
                    Gate::Open
                };
                withdraw_once(&pool, mode, DR1, dong(20_400), "withdraw:DR-001:1", gate).await
            })
            .await
        })
    };
    pause.reached.notified().await; // A đã nạp bản cũ
    topup(pool, DR1, dong(50_000), "topup:DR-001:2").await?; // B commit xong
    pause.resume.notify_one();
    let run = worker.await?;
    Ok(StaleReport {
        attempts: run.attempts,
        withdrew: run.result.is_ok(),
        stored_balance: snapshot(pool, DR1).await?.0,
        ledger_balance: ledger_balance(pool, DR1).await?,
        audit: audit(pool).await?,
    })
}

/// Kết quả hai (hoặc nhiều) lần chuyển tiền đồng thời.
#[derive(Debug)]
pub struct TransferReport {
    /// Số lần chuyển thành công.
    pub ok: usize,
    /// Số lần chết vì `40P01`.
    pub deadlocks: usize,
    /// Số lần thử của từng task, đã sắp tăng dần.
    pub attempts: Vec<u32>,
    /// Số dư đã lưu của hai ví đầu.
    pub balances: (Money, Money),
    /// Đối chiếu sổ.
    pub audit: Audit,
}

fn pause_free(r: Result<i64, StoreError>) -> Run<i64> {
    Run {
        attempts: 1,
        result: r,
    }
}

/// Hai ví 50.000 mỗi ví; T1 chuyển CU-001 → CU-002 (5.000), T2 chuyển CU-002 → CU-001 (3.000).
/// `AsGiven` có `Barrier` sau khi mỗi bên khóa ví nguồn: đan vòng chắc chắn. `ById` không dùng
/// `Barrier` (bên chậm chân sẽ không bao giờ tới được barrier, vì hàng đầu đã bị khóa).
/// `retry`: có thử lại `40P01` hay không.
pub async fn deadlock(lab: &Lab, order: LockOrder, retry: bool) -> Result<TransferReport> {
    let pool = &lab.pool;
    open_wallet(pool, CU1, dong(50_000), dong(100_000)).await?;
    open_wallet(pool, CU2, dong(50_000), dong(100_000)).await?;
    let barrier = (order == LockOrder::AsGiven).then(|| Arc::new(Barrier::new(2)));
    let legs = [
        (CU1, CU2, 5_000, "transfer:1"),
        (CU2, CU1, 3_000, "transfer:2"),
    ];
    let mut handles = Vec::new();
    for (i, (from, to, amount, key)) in legs.into_iter().enumerate() {
        let pool = pool.clone();
        let barrier = barrier.clone();
        handles.push(tokio::spawn(async move {
            if retry {
                transfer_retrying(
                    &pool,
                    from,
                    to,
                    dong(amount),
                    key,
                    order,
                    Attempts {
                        policy: RetryPolicy::lab(),
                        seed: i as u64 + 1,
                        barrier: barrier.as_deref(),
                    },
                )
                .await
            } else {
                let gate = barrier.as_deref().map_or(Gate::Open, Gate::Barrier);
                pause_free(transfer_once(&pool, from, to, dong(amount), key, order, gate).await)
            }
        }));
    }
    let mut runs = Vec::new();
    for handle in handles {
        runs.push(handle.await?);
    }
    transfer_report(pool, &runs).await
}

async fn transfer_report(pool: &PgPool, runs: &[Run<i64>]) -> Result<TransferReport> {
    let mut ok = 0;
    let mut deadlocks = 0;
    for run in runs {
        match &run.result {
            Ok(_) => ok += 1,
            Err(err) if err.sqlstate().as_deref() == Some("40P01") => deadlocks += 1,
            Err(other) => bail!("lỗi không mong đợi: {other}"),
        }
    }
    let mut attempts: Vec<u32> = runs.iter().map(|r| r.attempts).collect();
    attempts.sort_unstable();
    Ok(TransferReport {
        ok,
        deadlocks,
        attempts,
        balances: (snapshot(pool, CU1).await?.0, snapshot(pool, CU2).await?.0),
        audit: audit(pool).await?,
    })
}

/// Nạn nhân deadlock lúc CHƯA gửi `ROLLBACK`.
#[derive(Debug)]
pub struct VictimState {
    /// Số bên khóa được đủ hai hàng trong khi transaction của nạn nhân vẫn chưa kết thúc phía client.
    pub survivors: usize,
    /// SQLSTATE nạn nhân nhận được.
    pub error: Option<String>,
    /// SQLSTATE của lệnh kế tiếp trên transaction của nạn nhân.
    pub next_statement: Option<String>,
}

/// Hai transaction khóa ngược chiều rồi giữ nguyên, không ai `ROLLBACK` cho tới khi cả hai task trả về.
/// Bên thắng chỉ trả về được nếu khóa của nạn nhân đã nhả: server nhả lúc huỷ, không đợi client.
pub async fn deadlock_victim(lab: &Lab) -> Result<VictimState> {
    let pool = &lab.pool;
    open_wallet(pool, CU1, dong(50_000), dong(100_000)).await?;
    open_wallet(pool, CU2, dong(50_000), dong(100_000)).await?;
    const LOCK: &str = "SELECT 1 FROM account_balances WHERE account_id = $1 FOR UPDATE";
    let barrier = Arc::new(Barrier::new(2));
    let mut handles = Vec::new();
    for (first, second) in [(CU1, CU2), (CU2, CU1)] {
        let pool = pool.clone();
        let barrier = Arc::clone(&barrier);
        handles.push(tokio::spawn(async move {
            let mut tx = pool.begin().await?;
            sqlx::query(LOCK).bind(first).fetch_one(&mut *tx).await?;
            barrier.wait().await;
            let locked = sqlx::query(LOCK).bind(second).fetch_one(&mut *tx).await;
            Ok::<_, sqlx::Error>((tx, locked.map(drop)))
        }));
    }
    let mut report = VictimState {
        survivors: 0,
        error: None,
        next_statement: None,
    };
    let mut open = Vec::new();
    for handle in handles {
        open.push(handle.await??);
    }
    for (tx, locked) in &mut open {
        match locked {
            Ok(()) => report.survivors += 1,
            Err(err) => {
                report.error = crate::store::sqlstate(err);
                let next = sqlx::query("SELECT 1").execute(&mut **tx).await;
                report.next_statement = next.err().and_then(|e| crate::store::sqlstate(&e));
            }
        }
    }
    for (tx, _) in open {
        tx.rollback().await?;
    }
    Ok(report)
}

/// 12 lần chuyển giữa ba ví theo cả hai chiều, khóa theo id, không `Barrier`, không thử lại:
/// mọi lần phải xong ngay ở lần thử đầu.
pub async fn ordered_stress(lab: &Lab) -> Result<(TransferReport, Money)> {
    let pool = &lab.pool;
    let wallets = [CU1, CU2, "customer:CU-003"];
    for w in wallets {
        open_wallet(pool, w, dong(100_000), dong(1_000_000)).await?;
    }
    let pairs = [(0, 1), (1, 0), (1, 2), (2, 1), (2, 0), (0, 2)];
    let mut handles = Vec::new();
    for i in 0..12usize {
        let (from, to) = pairs[i % pairs.len()];
        let pool = pool.clone();
        handles.push(tokio::spawn(async move {
            pause_free(
                transfer_once(
                    &pool,
                    wallets[from],
                    wallets[to],
                    dong(1_000),
                    &format!("transfer:{i}"),
                    LockOrder::ById,
                    Gate::Open,
                )
                .await,
            )
        }));
    }
    let mut runs = Vec::new();
    for handle in handles {
        runs.push(handle.await?);
    }
    let report = transfer_report(pool, &runs).await?;
    let mut total = Money::ZERO;
    for w in wallets {
        total = total
            .checked_add(snapshot(pool, w).await?.0)
            .expect("không tràn");
    }
    Ok((report, total))
}

/// Ba cách đọc khi một giao dịch đang giữ `FOR UPDATE` trên `customer:CU-001`.
#[derive(Debug)]
pub struct LockCompare {
    /// Đọc thường (MVCC): vẫn thấy cả hai ví.
    pub plain: Vec<String>,
    /// `FOR UPDATE SKIP LOCKED`: bỏ qua ví đang bị khóa.
    pub skip_locked: Vec<String>,
    /// SQLSTATE của `FOR UPDATE NOWAIT`.
    pub nowait_sqlstate: Option<String>,
}

/// So `SELECT` thường, `SKIP LOCKED` và `NOWAIT` khi một ví đang bị khóa. Không chờ theo thời gian.
pub async fn lock_variants(lab: &Lab) -> Result<LockCompare> {
    let pool = &lab.pool;
    open_wallet(pool, CU1, dong(50_000), dong(100_000)).await?;
    open_wallet(pool, CU2, dong(50_000), dong(100_000)).await?;
    let mut holder = pool.begin().await?;
    sqlx::query("SELECT 1 FROM account_balances WHERE account_id = $1 FOR UPDATE")
        .bind(CU1)
        .fetch_one(&mut *holder)
        .await?;

    let plain = sqlx::query(
        "SELECT account_id FROM account_balances WHERE account_id IN ($1, $2) ORDER BY account_id",
    )
    .bind(CU1)
    .bind(CU2)
    .fetch_all(pool)
    .await?
    .iter()
    .map(|r| r.get::<String, _>(0))
    .collect();
    let skip_locked = sqlx::query(
        "SELECT account_id FROM account_balances WHERE account_id IN ($1, $2) \
         ORDER BY account_id FOR UPDATE SKIP LOCKED",
    )
    .bind(CU1)
    .bind(CU2)
    .fetch_all(pool)
    .await?
    .iter()
    .map(|r| r.get::<String, _>(0))
    .collect();
    let nowait = sqlx::query(
        "SELECT account_id FROM account_balances WHERE account_id IN ($1, $2) \
         ORDER BY account_id FOR UPDATE NOWAIT",
    )
    .bind(CU1)
    .bind(CU2)
    .fetch_all(pool)
    .await;
    holder.rollback().await?;
    Ok(LockCompare {
        plain,
        skip_locked,
        nowait_sqlstate: nowait.err().and_then(|e| crate::store::sqlstate(&e)),
    })
}

/// Sau một lần rút, thử sửa bút toán và posting: cả hai bị trigger append-only từ chối (`TN001`).
/// Trả SQLSTATE của hai lần thử.
pub async fn journal_is_append_only(lab: &Lab) -> Result<(Option<String>, Option<String>)> {
    let pool = &lab.pool;
    open_wallet(pool, DR1, dong(150_000), dong(100_000)).await?;
    withdraw_once(
        pool,
        Mode::Optimistic,
        DR1,
        dong(20_400),
        "withdraw:DR-001:1",
        Gate::Open,
    )
    .await?;
    let state = |r: Result<_, sqlx::Error>| r.err().and_then(|e| crate::store::sqlstate(&e));
    let on_entry = state(
        pool.execute("UPDATE journal_entries SET idempotency_key = 'sửa trộm'")
            .await,
    );
    let on_posting = state(pool.execute("UPDATE postings SET amount_dong = 1").await);
    Ok((on_entry, on_posting))
}

/// Tên cách khóa-rồi-đọc để in.
pub fn style_name(style: LockedLoad) -> &'static str {
    match style {
        LockedLoad::Separate => "hai câu (lock rồi đọc)",
        LockedLoad::JoinLockOne => "JOIN, FOR UPDATE OF b",
        LockedLoad::JoinLockBoth => "JOIN, FOR UPDATE OF b, l",
    }
}

/// Hàng `account_limits` mà người chờ khóa nhìn thấy.
#[derive(Debug)]
pub struct JoinTrap {
    /// Số dư người chờ nạp được (sau khi người giữ khóa commit).
    pub balance: Money,
    /// Số đã rút người chờ nạp được.
    pub withdrawn: Money,
}

/// Ví 150.000, hạn mức 30.000. A khóa ví, rút 20.400, CHƯA commit. B nạp bằng `style` và bị chặn
/// (lab chờ tới khi `pg_stat_activity` báo B đang chờ khóa, không đoán bằng thời gian). Rồi A commit.
/// Số dư đúng là 129.600 và đã rút đúng là 20.400; trả cái B thực sự thấy.
pub async fn join_lock_trap(lab: &Lab, style: LockedLoad) -> Result<JoinTrap> {
    let pool = &lab.pool;
    open_wallet(pool, DR1, dong(150_000), dong(30_000)).await?;
    let mut a = pool.begin().await?;
    withdraw_on(
        &mut a,
        Mode::ForUpdate,
        DR1,
        dong(20_400),
        "withdraw:DR-001:1",
        Gate::Open,
    )
    .await?;

    let waiter = {
        let pool = pool.clone();
        tokio::spawn(async move {
            let mut conn = pool.acquire().await?;
            load_locked(&mut conn, DR1, style).await
        })
    };
    // Chờ theo điều kiện: có một phiên của ca này đang chờ khóa.
    loop {
        let waiting: i64 = sqlx::query(
            "SELECT count(*) FROM pg_stat_activity \
             WHERE application_name = $1 AND wait_event_type = 'Lock'",
        )
        .bind(&lab.schema)
        .fetch_one(pool)
        .await?
        .get(0);
        // `is_finished`: nếu người chờ lỗi trước khi bị chặn thì không kẹt ở vòng này.
        if waiting > 0 || waiter.is_finished() {
            break;
        }
        tokio::task::yield_now().await;
    }
    a.commit().await?;
    let seen = waiter.await??;
    Ok(JoinTrap {
        balance: seen.balance(),
        withdrawn: seen.withdrawn(),
    })
}

/// Một dòng số đo: N task trên A ví, tất cả đều thành công (số tiền nhỏ, hạn mức lớn).
#[derive(Debug)]
pub struct Sample {
    /// Cách bảo vệ.
    pub mode: Mode,
    /// Số task.
    pub tasks: usize,
    /// Số ví mà các task chia nhau.
    pub wallets: usize,
    /// Phân bố: số lần thử → số task.
    pub histogram: BTreeMap<u32, usize>,
    /// Tổng số lần thử của mọi task.
    pub total_attempts: u32,
    /// Số lần thử của task nhiều nhất.
    pub max_attempts: u32,
    /// Thời gian chạy, ms (số đo trên máy viết bài, không phải SLA).
    pub elapsed_ms: u128,
}

/// Đo số lần thử theo độ tranh chấp trên một schema riêng. Có đo giờ; KHÔNG dùng trong `--verify`/test.
pub async fn measure(mode: Mode, tasks: usize, wallets: usize) -> Result<Sample> {
    crate::lab::with_lab(async move |lab| {
        let ids: Vec<String> = (0..wallets)
            .map(|k| format!("driver:DR-{:03}", k + 1))
            .collect();
        for id in &ids {
            open_wallet(&lab.pool, id, dong(1_000_000), dong(1_000_000)).await?;
        }
        // Số lần thử tối đa của một task chặn bởi số lần commit của người khác trên cùng ví, nên
        // 64 là dư cho mọi cấu hình trong bài.
        let policy = RetryPolicy {
            max_attempts: 64,
            ..RetryPolicy::lab()
        };
        let started = Instant::now();
        let runs = run_withdrawals(&lab.pool, mode, &ids, tasks, dong(1_000), policy).await?;
        let elapsed_ms = started.elapsed().as_millis();
        let (ok, _, _, attempts) = tally(&runs)?;
        ensure!(ok == tasks, "mọi task phải thành công, được {ok}/{tasks}");
        let mut histogram = BTreeMap::new();
        for a in &attempts {
            *histogram.entry(*a).or_insert(0) += 1;
        }
        let a = audit(&lab.pool).await?;
        ensure!(
            a.total_balance == 0 && a.projection_mismatch.is_empty(),
            "sổ lệch: {a:?}"
        );
        Ok(Sample {
            mode,
            tasks,
            wallets,
            histogram,
            total_attempts: attempts.iter().sum(),
            max_attempts: attempts.last().copied().unwrap_or(0),
            elapsed_ms,
        })
    })
    .await
}
