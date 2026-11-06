//! Các thí nghiệm của bài 24. Mỗi hàm chạy thật trên PostgreSQL và trả một báo cáo;
//! `verify()` của báo cáo kiểm các giá trị mà bài viết nêu.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use anyhow::{Result, ensure};
use ledger_core::{
    AccountId, JournalEntry, LedgerError, Money, Posting, TripCharge, capture_entry,
    default_policy, fare_for_km,
};
use sqlx::Row;
use sqlx::postgres::PgPool;
use tokio::sync::{Barrier, oneshot};
use tokio::task::JoinSet;

use crate::lab::Lab;
use crate::store::{
    Isolation, PostOutcome, StoreError, insert_entry, load_by_key, post_entry, retry_serializable,
    sqlstate,
};

/// Hạn mức rút của một tài xế trong thí nghiệm: 30.000 đồng.
pub const WITHDRAW_LIMIT: Money = Money::from_dong(30_000);

fn acct(raw: &str) -> AccountId {
    AccountId::new(raw).expect("mã tài khoản hợp lệ")
}

/// Bút toán thu cước của một chuyến `km` km, dựng bằng `ledger-core` (bài 20).
///
/// # Errors
/// Lỗi dựng bút toán của `ledger-core`.
pub fn capture(trip_id: &str, driver: &str, km: u32) -> Result<JournalEntry, LedgerError> {
    let charge = TripCharge {
        trip_id: trip_id.to_string(),
        customer: acct("customer:CU-001"),
        driver: acct(driver),
        fare: fare_for_km(km),
    };
    capture_entry(&charge, &default_policy())
}

/// Bút toán rút tiền: tiền ra khỏi ví tài xế, vào tài khoản clearing.
fn payout(key: &str, driver: &str, amount: Money) -> Result<JournalEntry, LedgerError> {
    let out = amount.checked_neg().ok_or(LedgerError::Overflow)?;
    JournalEntry::try_new(
        key,
        vec![
            Posting::new(acct(driver), out),
            Posting::new(acct("psp:vixanh:clearing"), amount),
        ],
    )
}

async fn count(pool: &PgPool, table: Table) -> Result<i64> {
    let sql = match table {
        Table::Entries => "SELECT count(*) FROM journal_entries",
        Table::Postings => "SELECT count(*) FROM postings",
    };
    Ok(sqlx::query(sql).fetch_one(pool).await?.get(0))
}

#[derive(Clone, Copy)]
enum Table {
    Entries,
    Postings,
}

// ---------------------------------------------------------------- 1. Transaction

/// Kết quả thí nghiệm transaction: commit, lỗi giữa chừng, drop không commit.
#[derive(Debug)]
pub struct AtomicityReport {
    /// Kết quả ghi bút toán `TN-0001`.
    pub committed: PostOutcome,
    /// Số dòng (bút toán, posting) sau commit.
    pub after_commit: (i64, i64),
    /// SQLSTATE của lỗi ở posting thứ ba (tài khoản không tồn tại).
    pub failure_sqlstate: Option<String>,
    /// Tên ràng buộc bị vi phạm.
    pub failure_constraint: Option<String>,
    /// Số dòng sau lỗi giữa chừng: phải bằng `after_commit`.
    pub after_failure: (i64, i64),
    /// Số dòng nhìn từ BÊN TRONG transaction chưa commit.
    pub inside_open_tx: (i64, i64),
    /// Số dòng sau khi drop transaction không commit.
    pub after_drop: (i64, i64),
    /// Ghi lại đúng key của transaction đã drop: không bị chặn, được cấp id mới.
    pub rewrite_after_drop: PostOutcome,
}

/// Chạy thí nghiệm transaction.
///
/// # Errors
/// Lỗi hạ tầng (kết nối, SQL) hoặc kết quả bất ngờ.
pub async fn atomicity(lab: &Lab) -> Result<AtomicityReport> {
    let pool = &lab.pool;
    let committed = post_entry(pool, &capture("TN-0001", "driver:DR-001", 5)?).await?;
    let after_commit = (
        count(pool, Table::Entries).await?,
        count(pool, Table::Postings).await?,
    );

    // Posting thứ ba trỏ tới tài khoản không có: hai dòng đầu đã INSERT xong rồi mới lỗi.
    let broken = capture("TN-0002", "driver:DR-999", 5)?;
    let failure = post_entry(pool, &broken).await;
    let (failure_sqlstate, failure_constraint) = match &failure {
        Err(StoreError::Db(sqlx::Error::Database(db))) => (
            db.code().map(|c| c.into_owned()),
            db.constraint().map(str::to_owned),
        ),
        other => anyhow::bail!("mong lỗi DB, nhận {other:?}"),
    };
    let after_failure = (
        count(pool, Table::Entries).await?,
        count(pool, Table::Postings).await?,
    );

    // Ghi xong nhưng không commit: nhìn từ trong thấy dữ liệu, drop thì mất.
    let pending = capture("TN-0003", "driver:DR-001", 5)?;
    let inside_open_tx = {
        let mut tx = pool.begin().await?;
        insert_entry(&mut tx, &pending).await?;
        let entries: i64 = sqlx::query("SELECT count(*) FROM journal_entries")
            .fetch_one(&mut *tx)
            .await?
            .get(0);
        let postings: i64 = sqlx::query("SELECT count(*) FROM postings")
            .fetch_one(&mut *tx)
            .await?
            .get(0);
        (entries, postings)
        // `tx` bị drop ở đây mà chưa commit: sqlx xếp ROLLBACK, gửi khi kết nối về pool.
    };
    let after_drop = (
        count(pool, Table::Entries).await?,
        count(pool, Table::Postings).await?,
    );

    // Nếu transaction cũ còn mở, ON CONFLICT sẽ đợi mãi; timeout để lỗi rõ thay vì treo.
    let rewrite_after_drop =
        tokio::time::timeout(Duration::from_secs(5), post_entry(pool, &pending))
            .await
            .map_err(|_| anyhow::anyhow!("ghi lại bị chặn: transaction đã drop chưa rollback"))??;

    Ok(AtomicityReport {
        committed,
        after_commit,
        failure_sqlstate,
        failure_constraint,
        after_failure,
        inside_open_tx,
        after_drop,
        rewrite_after_drop,
    })
}

impl AtomicityReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không như kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(
            self.committed == PostOutcome::Created { id: 1 },
            "commit: {:?}",
            self.committed
        );
        ensure!(
            self.after_commit == (1, 3),
            "sau commit: {:?}",
            self.after_commit
        );
        ensure!(
            self.failure_sqlstate.as_deref() == Some("23503"),
            "SQLSTATE: {:?}",
            self.failure_sqlstate
        );
        ensure!(self.failure_constraint.as_deref() == Some("postings_account_id_fkey"));
        ensure!(
            self.after_failure == (1, 3),
            "lỗi giữa chừng để lại dữ liệu: {:?}",
            self.after_failure
        );
        ensure!(
            self.inside_open_tx == (2, 6),
            "trong tx: {:?}",
            self.inside_open_tx
        );
        ensure!(self.after_drop == (1, 3), "sau drop: {:?}", self.after_drop);
        // id 2 (TN-0002 lỗi) và id 3 (TN-0003 bị rollback) đã bị cấp phát: id có khoảng trống.
        ensure!(
            self.rewrite_after_drop == PostOutcome::Created { id: 4 },
            "ghi lại: {:?}",
            self.rewrite_after_drop
        );
        Ok(())
    }
}

// ------------------------------------------------------- 2. Idempotency đồng thời

/// Kết quả thí nghiệm idempotency.
#[derive(Debug)]
pub struct IdempotencyReport {
    /// Cặp hai task chạy bằng `tokio::join!`.
    pub pair: (PostOutcome, PostOutcome),
    /// Số task `Created` trong 8 task cùng key.
    pub created_of_8: usize,
    /// Số task `Replayed` trong 8 task cùng key.
    pub replayed_of_8: usize,
    /// Mọi task cùng thấy một id?
    pub same_id: bool,
    /// Số dòng (bút toán, posting) sau tất cả.
    pub rows: (i64, i64),
    /// Cùng key, cước khác: lỗi trả về.
    pub conflict: String,
}

/// Chạy thí nghiệm idempotency đồng thời.
///
/// # Errors
/// Lỗi hạ tầng hoặc kết quả bất ngờ.
pub async fn idempotency(lab: &Lab) -> Result<IdempotencyReport> {
    let pool = &lab.pool;

    // Hai task cùng key, cùng nội dung.
    let first = capture("TN-0001", "driver:DR-001", 5)?;
    let (a, b) = tokio::join!(post_entry(pool, &first), post_entry(pool, &first));
    let pair = (a?, b?);

    // Tám task cùng key khác (TN-0002), chờ nhau ở barrier để xuất phát cùng lúc.
    let entry = Arc::new(capture("TN-0002", "driver:DR-001", 5)?);
    let gate = Arc::new(Barrier::new(8));
    let mut tasks = JoinSet::new();
    for _ in 0..8 {
        let (pool, entry, gate) = (pool.clone(), entry.clone(), gate.clone());
        tasks.spawn(async move {
            gate.wait().await;
            post_entry(&pool, &entry).await
        });
    }
    let mut outcomes = Vec::new();
    while let Some(joined) = tasks.join_next().await {
        outcomes.push(joined??);
    }
    let created_of_8 = outcomes
        .iter()
        .filter(|o| matches!(o, PostOutcome::Created { .. }))
        .count();
    let replayed_of_8 = outcomes.len() - created_of_8;
    let same_id = outcomes.windows(2).all(|w| w[0].id() == w[1].id());
    let rows = (
        count(pool, Table::Entries).await?,
        count(pool, Table::Postings).await?,
    );

    // Cùng key TN-0001 nhưng cước 6 km (30.000) thay vì 5 km (25.500).
    let different = capture("TN-0001", "driver:DR-001", 6)?;
    let conflict = match post_entry(pool, &different).await {
        Err(err @ StoreError::IdempotencyConflict { .. }) => err.to_string(),
        other => anyhow::bail!("mong IdempotencyConflict, nhận {other:?}"),
    };
    Ok(IdempotencyReport {
        pair,
        created_of_8,
        replayed_of_8,
        same_id,
        rows,
        conflict,
    })
}

impl IdempotencyReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không như kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        let (a, b) = self.pair;
        let created = [a, b]
            .iter()
            .filter(|o| matches!(o, PostOutcome::Created { .. }))
            .count();
        ensure!(created == 1 && a.id() == b.id(), "cặp: {:?}", self.pair);
        ensure!(
            self.created_of_8 == 1 && self.replayed_of_8 == 7,
            "8 task: {self:?}"
        );
        ensure!(self.same_id, "các task thấy id khác nhau");
        ensure!(self.rows == (2, 6), "số dòng: {:?}", self.rows);
        ensure!(
            self.conflict.contains("trip:TN-0001:capture"),
            "conflict: {}",
            self.conflict
        );
        Ok(())
    }
}

// ------------------------------------------------- 3. ON CONFLICT đợi giao dịch kia

/// Quan sát `INSERT ... ON CONFLICT DO NOTHING` khi key đang bị giữ bởi giao dịch chưa kết thúc.
#[derive(Debug)]
pub struct LockWaitReport {
    /// `wait_event_type` của phiên thứ hai trong `pg_stat_activity` khi nó bị chặn.
    pub wait_event_type: Option<String>,
    /// `wait_event` của phiên thứ hai.
    pub wait_event: Option<String>,
    /// Phiên thứ hai chưa xong khi giao dịch đầu còn mở?
    pub blocked_while_open: bool,
    /// Kết quả của phiên thứ hai sau khi giao dịch đầu kết thúc (`None` = không chèn được).
    pub second_result: Option<i64>,
}

/// Giao dịch đầu chèn key rồi giữ mở; phiên hai chèn cùng key. `first_commits` chọn commit hay rollback.
///
/// # Errors
/// Lỗi hạ tầng, hoặc phiên hai không bị chặn trong 5 giây.
pub async fn on_conflict_waits(lab: &Lab, first_commits: bool) -> Result<LockWaitReport> {
    let entry = capture("TN-0001", "driver:DR-001", 5)?;
    let mut first = lab.pool.begin().await?;
    insert_entry(&mut first, &entry)
        .await?
        .expect("key mới phải chèn được");

    let (pid_tx, pid_rx) = oneshot::channel();
    let pool = lab.pool.clone();
    let second = tokio::spawn(async move {
        let mut tx = pool.begin().await?;
        let pid: i32 = sqlx::query("SELECT pg_backend_pid()")
            .fetch_one(&mut *tx)
            .await?
            .get(0);
        pid_tx.send(pid).ok();
        // Câu này chặn cho tới khi giao dịch đầu kết thúc.
        let inserted = insert_entry(&mut tx, &entry).await?;
        if inserted.is_some() {
            tx.commit().await?;
        }
        Ok::<_, sqlx::Error>(inserted)
    });

    let pid = pid_rx.await?;
    let mut observed = (None, None);
    for _ in 0..250 {
        let row =
            sqlx::query("SELECT wait_event_type, wait_event FROM pg_stat_activity WHERE pid = $1")
                .bind(pid)
                .fetch_one(&lab.pool)
                .await?;
        observed = (row.get("wait_event_type"), row.get("wait_event"));
        if observed.1.as_deref() == Some("transactionid") {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let blocked_while_open = !second.is_finished();

    if first_commits {
        first.commit().await?;
    } else {
        first.rollback().await?;
    }
    let second_result = second.await??;
    Ok(LockWaitReport {
        wait_event_type: observed.0,
        wait_event: observed.1,
        blocked_while_open,
        second_result,
    })
}

impl LockWaitReport {
    /// Kiểm: bị chặn ở khoá `transactionid`; kết quả đúng với cách giao dịch đầu kết thúc.
    ///
    /// # Errors
    /// Giá trị không như kỳ vọng.
    pub fn verify(&self, first_committed: bool) -> Result<()> {
        ensure!(self.wait_event_type.as_deref() == Some("Lock"), "{self:?}");
        ensure!(
            self.wait_event.as_deref() == Some("transactionid"),
            "{self:?}"
        );
        ensure!(self.blocked_while_open, "phiên hai không bị chặn");
        ensure!(self.second_result.is_none() == first_committed, "{self:?}");
        Ok(())
    }
}

/// `ON CONFLICT DO NOTHING` với một bản đã commit SAU khi snapshot của giao dịch được chốt.
#[derive(Debug)]
pub struct SnapshotReport {
    /// Mức cô lập của giao dịch đến sau.
    pub isolation: Isolation,
    /// SQLSTATE nếu chính câu `INSERT ... ON CONFLICT` báo lỗi; `None` nếu nó trả 0 dòng.
    pub insert_sqlstate: Option<String>,
    /// Id bút toán đọc lại được (cùng giao dịch, hoặc giao dịch mới nếu câu `INSERT` lỗi).
    pub seen_id: Option<i64>,
    /// Id của bút toán đã commit trước đó.
    pub committed_id: i64,
}

/// Chốt snapshot trước, để bản khác commit, rồi mới chèn cùng key.
///
/// # Errors
/// Lỗi hạ tầng, hoặc câu `INSERT` lại chèn được key đã có.
pub async fn on_conflict_after_snapshot(lab: &Lab, isolation: Isolation) -> Result<SnapshotReport> {
    let entry = capture("TN-0001", "driver:DR-001", 5)?;
    let key = entry.idempotency_key();
    let mut late = isolation.begin(&lab.pool).await?;
    // Câu đầu tiên chốt snapshot của REPEATABLE READ và SERIALIZABLE.
    sqlx::query("SELECT count(*) FROM journal_entries")
        .fetch_one(&mut *late)
        .await?;
    let committed_id = post_entry(&lab.pool, &entry).await?.id();

    let (insert_sqlstate, loaded) = match insert_entry(&mut late, &entry).await {
        Ok(Some(id)) => anyhow::bail!("key đã có mà vẫn chèn được: id {id}"),
        Ok(None) => (None, load_by_key(&mut late, key).await?),
        Err(err) => {
            let code = sqlstate(&err);
            late.rollback().await?;
            // Giao dịch mới, snapshot mới: giống một lần `retry_serializable`.
            let mut again = isolation.begin(&lab.pool).await?;
            ensure!(insert_entry(&mut again, &entry).await?.is_none());
            (code, load_by_key(&mut again, key).await?)
        }
    };
    Ok(SnapshotReport {
        isolation,
        insert_sqlstate,
        seen_id: loaded.map(|(id, _)| id),
        committed_id,
    })
}

impl SnapshotReport {
    /// Kiểm: READ COMMITTED trả 0 dòng; hai mức cao hơn báo 40001, giao dịch mới mới thấy bản đó.
    ///
    /// # Errors
    /// Giá trị không như kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        let expected = match self.isolation {
            Isolation::ReadCommitted => None,
            Isolation::RepeatableRead | Isolation::Serializable => Some("40001"),
        };
        ensure!(self.insert_sqlstate.as_deref() == expected, "{self:?}");
        ensure!(self.seen_id == Some(self.committed_id), "{self:?}");
        Ok(())
    }
}

// ------------------------------------------------- 4. Write skew và SERIALIZABLE

/// Kết quả hai yêu cầu rút 20.400 đồng cùng lúc với hạn mức 30.000.
#[derive(Debug)]
pub struct LimitReport {
    /// Mức cô lập đã dùng.
    pub isolation: Isolation,
    /// Số yêu cầu rút thành công.
    pub ok: u32,
    /// Số yêu cầu bị từ chối vì vượt hạn mức.
    pub rejected: u32,
    /// Số lần PostgreSQL trả 40001.
    pub serialization_failures: u32,
    /// Số lần thử của từng yêu cầu.
    pub attempts: Vec<u32>,
    /// Tổng tiền đã rút khỏi ví tài xế sau cùng.
    pub total_out: i64,
}

/// Kiểm hạn mức rồi ghi, trong MỘT transaction. `gate` giữ hai bên lại sau khi cùng kiểm xong.
async fn withdraw(
    pool: &PgPool,
    isolation: Isolation,
    driver: &str,
    key: &str,
    amount: Money,
    gate: Option<&Barrier>,
) -> Result<(), StoreError> {
    let mut tx = isolation.begin(pool).await?;
    let used: i64 = sqlx::query(
        "SELECT COALESCE(SUM(-amount_dong), 0)::bigint FROM postings \
         WHERE account_id = $1 AND amount_dong < 0",
    )
    .bind(driver)
    .fetch_one(&mut *tx)
    .await?
    .get(0);
    let used = Money::from_dong(used);
    if used
        .checked_add(amount)
        .is_none_or(|total| total > WITHDRAW_LIMIT)
    {
        return Err(StoreError::LimitExceeded {
            used,
            requested: amount,
            limit: WITHDRAW_LIMIT,
        });
    }
    if let Some(gate) = gate {
        gate.wait().await; // cả hai đã đọc `used`; giờ mới cùng ghi
    }
    // Mỗi yêu cầu có key riêng nên `Ok(None)` không xảy ra trong thí nghiệm này.
    insert_entry(&mut tx, &payout(key, driver, amount)?).await?;
    tx.commit().await?; // ở SERIALIZABLE, 40001 có thể đến ngay tại COMMIT
    Ok(())
}

/// Hai yêu cầu rút 20.400 đồng của cùng một tài xế chạy chồng lấp, mỗi bên thử tối đa 3 lần.
///
/// # Errors
/// Lỗi hạ tầng không thuộc dự kiến.
pub async fn limit_race(lab: &Lab, isolation: Isolation, driver: &str) -> Result<LimitReport> {
    let gate = Arc::new(Barrier::new(2));
    let failures = Arc::new(AtomicU32::new(0));
    let mut tasks = JoinSet::new();
    for n in 1..=2 {
        let (pool, gate, failures) = (lab.pool.clone(), gate.clone(), failures.clone());
        let driver = driver.to_string();
        tasks.spawn(async move {
            let key = format!("payout:{driver}:{n}");
            retry_serializable(3, |attempt| {
                let (pool, gate, failures, driver, key) = (
                    pool.clone(),
                    gate.clone(),
                    failures.clone(),
                    driver.clone(),
                    key.clone(),
                );
                async move {
                    // Chỉ lần thử đầu cần đồng bộ; lần thử lại chạy tự do.
                    let wait = (attempt == 1).then_some(&*gate);
                    let result = withdraw(
                        &pool,
                        isolation,
                        &driver,
                        &key,
                        Money::from_dong(20_400),
                        wait,
                    )
                    .await;
                    if let Err(err) = &result
                        && err.is_serialization_failure()
                    {
                        failures.fetch_add(1, Ordering::SeqCst);
                    }
                    result
                }
            })
            .await
        });
    }
    let (mut ok, mut rejected, mut attempts) = (0, 0, Vec::new());
    while let Some(joined) = tasks.join_next().await {
        match joined? {
            Ok(((), tries)) => {
                ok += 1;
                attempts.push(tries);
            }
            Err(StoreError::LimitExceeded { .. }) => rejected += 1,
            Err(other) => return Err(other.into()),
        }
    }
    let total_out: i64 = sqlx::query(
        "SELECT COALESCE(SUM(-amount_dong), 0)::bigint FROM postings \
         WHERE account_id = $1 AND amount_dong < 0",
    )
    .bind(driver)
    .fetch_one(&lab.pool)
    .await?
    .get(0);
    Ok(LimitReport {
        isolation,
        ok,
        rejected,
        serialization_failures: failures.load(Ordering::SeqCst),
        attempts,
        total_out,
    })
}

impl LimitReport {
    /// Kiểm: READ COMMITTED và REPEATABLE READ phá hạn mức; SERIALIZABLE giữ nhờ 40001 và retry.
    ///
    /// # Errors
    /// Giá trị không như kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        match self.isolation {
            Isolation::ReadCommitted | Isolation::RepeatableRead => {
                ensure!(self.ok == 2 && self.rejected == 0, "{self:?}");
                ensure!(self.serialization_failures == 0, "{self:?}");
                ensure!(self.total_out == 40_800, "{self:?}");
                ensure!(
                    self.total_out > WITHDRAW_LIMIT.dong(),
                    "mức này lẽ ra phá hạn mức"
                );
            }
            Isolation::Serializable => {
                ensure!(
                    self.serialization_failures >= 1,
                    "không thấy 40001: {self:?}"
                );
                ensure!(self.ok == 1 && self.rejected == 1, "{self:?}");
                ensure!(self.total_out == 20_400, "{self:?}");
                ensure!(self.total_out <= WITHDRAW_LIMIT.dong());
            }
        }
        Ok(())
    }
}

// ------------------------------------------------------------ 5. Append-only

/// Một câu lệnh sửa sổ bị trigger từ chối.
#[derive(Debug)]
pub struct Rejected {
    /// Câu lệnh đã thử.
    pub statement: &'static str,
    /// SQLSTATE nhận được.
    pub sqlstate: Option<String>,
    /// Thông điệp của PostgreSQL.
    pub message: String,
}

/// Kết quả thí nghiệm append-only.
#[derive(Debug)]
pub struct AppendOnlyReport {
    /// Các câu lệnh bị từ chối.
    pub rejected: Vec<Rejected>,
    /// Số dòng (bút toán, posting) và tổng số tiền sau khi thử phá.
    pub after: (i64, i64, i64),
}

/// Thử UPDATE, DELETE, TRUNCATE lên sổ đã có bút toán `TN-0001`.
///
/// # Errors
/// Lỗi hạ tầng, hoặc một câu lệnh sửa sổ lại thành công.
pub async fn append_only(lab: &Lab) -> Result<AppendOnlyReport> {
    let pool = &lab.pool;
    post_entry(pool, &capture("TN-0001", "driver:DR-001", 5)?).await?;
    let attempts: [&'static str; 4] = [
        "UPDATE postings SET amount_dong = 1 WHERE entry_id = 1",
        "DELETE FROM postings WHERE entry_id = 1",
        "UPDATE journal_entries SET idempotency_key = 'x' WHERE id = 1",
        "TRUNCATE postings",
    ];
    let mut rejected = Vec::new();
    for statement in attempts {
        match sqlx::query(statement).execute(pool).await {
            Ok(done) => {
                anyhow::bail!("câu lệnh lẽ ra bị cấm lại chạy được: {statement} ({done:?})")
            }
            Err(err) => {
                let message = match &err {
                    sqlx::Error::Database(db) => db.message().to_string(),
                    other => other.to_string(),
                };
                rejected.push(Rejected {
                    statement,
                    sqlstate: sqlstate(&err),
                    message,
                });
            }
        }
    }
    let after = (
        count(pool, Table::Entries).await?,
        count(pool, Table::Postings).await?,
        sqlx::query("SELECT COALESCE(SUM(amount_dong), 0)::bigint FROM postings")
            .fetch_one(pool)
            .await?
            .get(0),
    );
    Ok(AppendOnlyReport { rejected, after })
}

impl AppendOnlyReport {
    /// Kiểm: cả bốn câu bị từ chối với SQLSTATE `TN001`, dữ liệu nguyên vẹn.
    ///
    /// # Errors
    /// Giá trị không như kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.rejected.len() == 4);
        for r in &self.rejected {
            ensure!(r.sqlstate.as_deref() == Some("TN001"), "{r:?}");
            ensure!(r.message.contains("append-only"), "{r:?}");
        }
        ensure!(self.after == (1, 3, 0), "dữ liệu đổi: {:?}", self.after);
        Ok(())
    }
}
