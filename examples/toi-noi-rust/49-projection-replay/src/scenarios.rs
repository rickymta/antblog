//! Các thí nghiệm của bài 49. Mỗi thí nghiệm nhận một `Lab` (schema riêng), trả kết quả để in và kiểm.

use std::time::{Duration, Instant};

use anyhow::{Result, ensure};
use sqlx::{AssertSqlSafe, PgPool, Row};

use crate::fixtures::{STANDARD, TN_0004, append_steps, entry_of};
use crate::lab::Lab;
use crate::log::{Cursor, ReadMode, append_entry, entries_behind, head};
use crate::projection::{
    Balances, Crash, IncomeByDay, ProjectionError, Role, apply_commit_without_checkpoint,
    checkpoint, cutover, drain, install, run_batch, run_batch_with, start_rebuild,
    wait_horizon_past_head,
};

/// `(account_id, balance_dong)`.
pub type BalanceRow = (String, i64);
/// `(day, account_id, income_dong)`.
pub type IncomeRow = (String, String, i64);

/// Đọc bảng số dư của một projection.
///
/// # Errors
/// Lỗi SQL.
pub async fn read_balances(pool: &PgPool, table: &str) -> Result<Vec<BalanceRow>> {
    let sql = format!("SELECT account_id, balance_dong FROM {table} ORDER BY 1");
    let rows = sqlx::query(AssertSqlSafe(sql)).fetch_all(pool).await?;
    Ok(rows
        .iter()
        .map(|r| (r.get("account_id"), r.get("balance_dong")))
        .collect())
}

/// Số dư tính thẳng từ `postings`, không qua projection: chuẩn để đối chiếu.
///
/// # Errors
/// Lỗi SQL.
pub async fn truth_balances(pool: &PgPool) -> Result<Vec<BalanceRow>> {
    let rows = sqlx::query(
        "SELECT account_id, SUM(amount_dong)::bigint AS total FROM postings GROUP BY 1 ORDER BY 1",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .iter()
        .map(|r| (r.get("account_id"), r.get("total")))
        .collect())
}

/// Đọc bảng thu nhập theo ngày.
///
/// # Errors
/// Lỗi SQL.
pub async fn read_income(pool: &PgPool, table: &str) -> Result<Vec<IncomeRow>> {
    let sql =
        format!("SELECT day::text AS day, account_id, income_dong FROM {table} ORDER BY 1, 2");
    let rows = sqlx::query(AssertSqlSafe(sql)).fetch_all(pool).await?;
    Ok(rows
        .iter()
        .map(|r| (r.get("day"), r.get("account_id"), r.get("income_dong")))
        .collect())
}

/// Thu nhập theo ngày tính thẳng từ log bằng SQL, cùng quy tắc với `income_delta`, múi giờ tùy chọn.
///
/// # Errors
/// Lỗi SQL.
pub async fn truth_income(pool: &PgPool, time_zone: &str) -> Result<Vec<IncomeRow>> {
    let rows = sqlx::query(
        "SELECT (e.posted_at AT TIME ZONE $1)::date::text AS day, p.account_id, \
                SUM(p.amount_dong)::bigint AS total \
         FROM postings p JOIN journal_entries e ON e.id = p.entry_id \
         WHERE (p.account_id = 'platform:revenue' OR p.account_id LIKE 'driver:%') \
           AND ((e.reversal_of IS NULL AND p.amount_dong > 0) \
                OR (e.reversal_of IS NOT NULL AND p.amount_dong < 0)) \
         GROUP BY 1, 2 ORDER BY 1, 2",
    )
    .bind(time_zone)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .iter()
        .map(|r| (r.get("day"), r.get("account_id"), r.get("total")))
        .collect())
}

fn balance_of(rows: &[BalanceRow], account: &str) -> i64 {
    rows.iter()
        .find(|(a, _)| a == account)
        .map_or(0, |(_, v)| *v)
}

// ---------------------------------------------------------------------------------------------
// 1. Hai projection từ cùng một log

/// Kết quả thí nghiệm 1.
pub struct Basic {
    /// Mã bút toán đã ghi, theo thứ tự.
    pub entry_ids: Vec<i64>,
    /// Số bút toán chậm hơn log trước khi chạy projection.
    pub behind_before: i64,
    /// Kích thước từng lô số dư.
    pub batches: Vec<usize>,
    /// Số dư trong projection.
    pub balances: Vec<BalanceRow>,
    /// Số dư tính thẳng từ postings.
    pub truth: Vec<BalanceRow>,
    /// Thu nhập theo ngày, phiên bản 1 (chia ngày UTC).
    pub income: Vec<IncomeRow>,
    /// Chuẩn tính theo UTC.
    pub income_truth: Vec<IncomeRow>,
    /// `entry_id` trong checkpoint số dư.
    pub checkpoint_entry: i64,
    /// Số sự kiện áp dụng ở lần chạy lại ngay sau đó.
    pub rerun_applied: usize,
    /// Bảng số dư có giữ nguyên sau lần chạy lại không.
    pub rerun_unchanged: bool,
}

/// Thí nghiệm 1: ghi 8 bút toán, chạy hai projection theo lô 3, rồi chạy lại.
///
/// # Errors
/// Lỗi SQL hoặc projection.
pub async fn basic(lab: &Lab) -> Result<Basic> {
    let pool = &lab.pool;
    let balances = Balances { version: 1 };
    let income = IncomeByDay { version: 1 };
    install(pool, &balances).await?;
    install(pool, &income).await?;
    let ids = append_steps(pool, &STANDARD).await?;
    let entry_ids: Vec<i64> = STANDARD.iter().map(|s| ids[s.key]).collect();

    let mut conn = pool.acquire().await?;
    let behind_before = entries_behind(&mut conn, Cursor::default()).await?;
    drop(conn);

    wait_horizon_past_head(pool).await?;
    let mut batches = Vec::new();
    loop {
        let n = run_batch(pool, &balances, Role::Live, ReadMode::Safe, 3).await?;
        if n == 0 {
            break;
        }
        batches.push(n);
    }
    drain(pool, &income, Role::Live, ReadMode::Safe, 3).await?;
    let before_rerun = read_balances(pool, "balances").await?;
    let rerun_applied = drain(pool, &balances, Role::Live, ReadMode::Safe, 3).await?;
    let after_rerun = read_balances(pool, "balances").await?;

    Ok(Basic {
        entry_ids,
        behind_before,
        batches,
        balances: after_rerun.clone(),
        truth: truth_balances(pool).await?,
        income: read_income(pool, "income_by_day").await?,
        income_truth: truth_income(pool, "UTC").await?,
        checkpoint_entry: checkpoint(pool, "balances", 1).await?.entry_id,
        rerun_applied,
        rerun_unchanged: before_rerun == after_rerun,
    })
}

impl Basic {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không như kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(
            self.entry_ids == (1..=8).collect::<Vec<i64>>(),
            "id bút toán"
        );
        ensure!(self.behind_before == 8, "lag trước khi chạy");
        ensure!(self.batches == vec![3, 3, 2], "lô {:?}", self.batches);
        ensure!(self.balances == self.truth, "số dư khác chuẩn");
        let expected: Vec<BalanceRow> = [
            ("customer:CU-001", -25_500),
            ("customer:CU-002", 0),
            ("customer:CU-003", -12_000),
            ("driver:DR-001", 20_400),
            ("driver:DR-002", 9_600),
            ("platform:revenue", 7_500),
            ("psp:vixanh:clearing", 0),
        ]
        .iter()
        .map(|(a, v)| ((*a).to_string(), *v))
        .collect();
        ensure!(self.balances == expected, "số dư kỳ vọng");
        ensure!(self.income == self.income_truth, "thu nhập khác chuẩn");
        ensure!(self.checkpoint_entry == 8, "checkpoint");
        ensure!(self.rerun_applied == 0 && self.rerun_unchanged, "chạy lại");
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------
// 2. Chạy lại không cộng trùng

/// Kết quả thí nghiệm 2 (ba nửa chạy trong ba schema riêng).
pub struct Replay {
    /// Số sự kiện lô đầu đã tính rồi bị rollback.
    pub crashed_batch: usize,
    /// Checkpoint sau sự cố trước COMMIT.
    pub checkpoint_after_crash: i64,
    /// Số dòng bảng số dư sau sự cố trước COMMIT.
    pub rows_after_crash: usize,
    /// Sau khi chạy lại, số dư có bằng chuẩn không (cách đúng).
    pub correct_equals_truth: bool,
    /// Cách sai: các tài khoản lệch `(tài khoản, chuẩn, projection)`.
    pub wrong_diffs: Vec<(String, i64, i64)>,
    /// Bốn worker cùng chạy: tổng sự kiện đã áp dụng.
    pub workers_applied: usize,
    /// Bốn worker cùng chạy: số dư có bằng chuẩn không.
    pub workers_equal_truth: bool,
}

/// Cách đúng: sự cố trước COMMIT rồi chạy lại.
///
/// # Errors
/// Lỗi SQL hoặc projection.
pub async fn replay_correct(lab: &Lab) -> Result<(usize, i64, usize, bool)> {
    let pool = &lab.pool;
    let p = Balances { version: 1 };
    install(pool, &p).await?;
    append_steps(pool, &STANDARD).await?;
    wait_horizon_past_head(pool).await?;
    let crashed =
        run_batch_with(pool, &p, Role::Live, ReadMode::Safe, 3, Crash::BeforeCommit).await?;
    let cp = checkpoint(pool, "balances", 1).await?.entry_id;
    let rows = read_balances(pool, "balances").await?.len();
    drain(pool, &p, Role::Live, ReadMode::Safe, 3).await?;
    let equal = read_balances(pool, "balances").await? == truth_balances(pool).await?;
    Ok((crashed, cp, rows, equal))
}

/// Cách sai: cộng bảng ở một transaction, ghi checkpoint ở transaction khác, chết giữa chừng.
///
/// # Errors
/// Lỗi SQL hoặc projection.
pub async fn replay_wrong(lab: &Lab) -> Result<Vec<(String, i64, i64)>> {
    let pool = &lab.pool;
    let p = Balances { version: 1 };
    install(pool, &p).await?;
    append_steps(pool, &STANDARD).await?;
    wait_horizon_past_head(pool).await?;
    apply_commit_without_checkpoint(pool, &p, 3).await?;
    drain(pool, &p, Role::Live, ReadMode::Safe, 3).await?;
    let projected = read_balances(pool, "balances").await?;
    let truth = truth_balances(pool).await?;
    Ok(truth
        .iter()
        .filter_map(|(account, expected)| {
            let got = balance_of(&projected, account);
            (got != *expected).then(|| (account.clone(), *expected, got))
        })
        .collect())
}

/// Bốn worker cùng chạy một projection: khoá dòng checkpoint xếp hàng họ.
///
/// # Errors
/// Lỗi SQL hoặc projection.
pub async fn replay_workers(lab: &Lab) -> Result<(usize, bool)> {
    let pool = &lab.pool;
    install(pool, &Balances { version: 1 }).await?;
    append_steps(pool, &STANDARD).await?;
    let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(4));
    let mut tasks = Vec::new();
    for _ in 0..4 {
        let pool = pool.clone();
        let barrier = barrier.clone();
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            drain(
                &pool,
                &Balances { version: 1 },
                Role::Live,
                ReadMode::Safe,
                2,
            )
            .await
        }));
    }
    let mut applied = 0;
    for task in tasks {
        applied += task.await??;
    }
    let equal = read_balances(pool, "balances").await? == truth_balances(pool).await?;
    Ok((applied, equal))
}

/// Thí nghiệm 2 đầy đủ: chạy ba nửa trong ba schema riêng.
///
/// # Errors
/// Lỗi SQL hoặc projection.
pub async fn replay() -> Result<Replay> {
    let crashed = crate::lab::with_lab(async |lab| replay_correct(&lab).await).await?;
    let wrong = crate::lab::with_lab(async |lab| replay_wrong(&lab).await).await?;
    let workers = crate::lab::with_lab(async |lab| replay_workers(&lab).await).await?;
    Ok(Replay {
        crashed_batch: crashed.0,
        checkpoint_after_crash: crashed.1,
        rows_after_crash: crashed.2,
        correct_equals_truth: crashed.3,
        wrong_diffs: wrong,
        workers_applied: workers.0,
        workers_equal_truth: workers.1,
    })
}

impl Replay {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không như kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.crashed_batch == 3, "lô bị rollback");
        ensure!(
            self.checkpoint_after_crash == 0 && self.rows_after_crash == 0,
            "rollback phải trả cả bảng lẫn checkpoint"
        );
        ensure!(self.correct_equals_truth, "cách đúng phải bằng chuẩn");
        ensure!(!self.wrong_diffs.is_empty(), "cách sai phải lệch");
        ensure!(
            self.wrong_diffs
                .iter()
                .any(|(a, truth, got)| a == "customer:CU-001"
                    && *truth == -25_500
                    && *got == -51_000),
            "CU-001 phải bị cộng đôi"
        );
        ensure!(self.workers_applied == 8, "mỗi bút toán đúng một lần");
        ensure!(self.workers_equal_truth, "bốn worker phải bằng chuẩn");
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------
// 3. Giao dịch commit muộn

/// Hai kịch bản dựng cuộc đua.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaceVariant {
    /// A INSERT trước (id 1) nhưng commit sau B (id 2).
    LateCommit,
    /// Giao dịch cũ lấy xid sớm, INSERT sau (id 2) và commit; giao dịch mới INSERT trước (id 1), commit sau.
    EarlyXid,
}

/// Kết quả một cuộc đua.
pub struct Race {
    /// Kịch bản.
    pub variant: RaceVariant,
    /// Cách đọc log.
    pub mode: ReadMode,
    /// Số sự kiện đã áp dụng khi giao dịch chậm còn mở.
    pub applied_while_open: usize,
    /// Số bút toán đã commit nằm sau checkpoint khi giao dịch chậm còn mở.
    pub behind_while_open: i64,
    /// Số dư cuối có bằng chuẩn không.
    pub equals_truth: bool,
    /// Tài khoản lệch `(tài khoản, chuẩn, projection)`.
    pub diffs: Vec<(String, i64, i64)>,
}

/// Một cuộc đua có kiểm soát giữa hai transaction và một projection số dư.
///
/// # Errors
/// Lỗi SQL hoặc projection.
pub async fn race(lab: &Lab, variant: RaceVariant, mode: ReadMode) -> Result<Race> {
    let pool = &lab.pool;
    let p = Balances { version: 1 };
    install(pool, &p).await?;
    // `slow`: chia tiền TN-0001 (tài xế DR-001 nhận 20.400). `fast`: chia tiền TN-0003.
    let slow = entry_of(&STANDARD[1])?;
    let fast = entry_of(&STANDARD[7])?;

    let (applied_while_open, behind_while_open);
    match variant {
        RaceVariant::LateCommit => {
            let mut a = pool.begin().await?;
            append_entry(&mut a, &slow, STANDARD[1].posted_at, None).await?; // id 1, chưa commit
            let mut b = pool.begin().await?;
            append_entry(&mut b, &fast, STANDARD[7].posted_at, None).await?; // id 2
            b.commit().await?;
            applied_while_open = run_batch(pool, &p, Role::Live, mode, 100).await?;
            behind_while_open = behind(pool).await?;
            a.commit().await?;
        }
        RaceVariant::EarlyXid => {
            let mut old = pool.begin().await?;
            // Ví dụ: cập nhật trạng thái thanh toán trước khi ghi bút toán, nên xid được cấp sớm.
            sqlx::query("SELECT pg_current_xact_id()::text")
                .execute(&mut *old)
                .await?;
            let mut new = pool.begin().await?;
            append_entry(&mut new, &slow, STANDARD[1].posted_at, None).await?; // id 1, chưa commit
            append_entry(&mut old, &fast, STANDARD[7].posted_at, None).await?; // id 2, xid nhỏ hơn
            old.commit().await?;
            // Chờ đến khi người đọc nhìn thấy bút toán của `old` (khoảng an toàn có thể bị giao
            // dịch lạ trên cùng server giữ lại một lúc).
            let mut applied = 0;
            for _ in 0..3000 {
                applied += run_batch(pool, &p, Role::Live, mode, 100).await?;
                if applied > 0 {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            applied_while_open = applied;
            behind_while_open = behind(pool).await?;
            new.commit().await?;
        }
    }

    wait_horizon_past_head(pool).await?;
    while run_batch(pool, &p, Role::Live, mode, 100).await? > 0 {}

    let projected = read_balances(pool, "balances").await?;
    let truth = truth_balances(pool).await?;
    let diffs: Vec<(String, i64, i64)> = truth
        .iter()
        .filter_map(|(account, expected)| {
            let got = balance_of(&projected, account);
            (got != *expected).then(|| (account.clone(), *expected, got))
        })
        .collect();
    Ok(Race {
        variant,
        mode,
        applied_while_open,
        behind_while_open,
        equals_truth: projected == truth,
        diffs,
    })
}

async fn behind(pool: &PgPool) -> Result<i64> {
    let cursor = checkpoint(pool, "balances", 1).await?;
    let mut conn = pool.acquire().await?;
    Ok(entries_behind(&mut conn, cursor).await?)
}

impl Race {
    /// Kỳ vọng theo từng cặp (kịch bản, cách đọc).
    ///
    /// # Errors
    /// Kết quả không như kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        let loses = matches!(
            (self.variant, self.mode),
            (RaceVariant::LateCommit, ReadMode::NaiveId)
                | (
                    RaceVariant::EarlyXid,
                    ReadMode::NaiveId | ReadMode::HorizonId
                )
        );
        ensure!(
            self.equals_truth != loses,
            "{:?}/{:?}: mất dòng = {loses}, nhưng equals_truth = {}",
            self.variant,
            self.mode,
            self.equals_truth
        );
        if loses {
            // Mất đúng bút toán chia tiền TN-0001: tài xế DR-001 thiếu 20.400.
            ensure!(
                self.diffs
                    .iter()
                    .any(|(a, truth, got)| a == "driver:DR-001" && *truth == 20_400 && *got == 0),
                "DR-001 phải thiếu 20.400"
            );
        }
        if self.variant == RaceVariant::LateCommit {
            // Khi A còn mở: cách naive đã áp dụng bút toán của B; hai cách còn lại không áp dụng gì.
            let expected_applied = usize::from(self.mode == ReadMode::NaiveId);
            ensure!(
                self.applied_while_open == expected_applied,
                "áp dụng khi mở: {}",
                self.applied_while_open
            );
            // Bút toán của B đã commit nhưng chưa qua khoảng an toàn: lag = 1 với hai cách có khoảng an toàn.
            let expected_behind = i64::from(self.mode != ReadMode::NaiveId);
            ensure!(
                self.behind_while_open == expected_behind,
                "lag khi mở: {}",
                self.behind_while_open
            );
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------
// 4. Dựng lại, so sánh, đổi tên nguyên tử, đổi phiên bản

/// Kết quả thí nghiệm 4.
pub struct Rebuild {
    /// Số sự kiện bản dựng lại đã xử lý trước khi chuyển.
    pub partial: usize,
    /// Bản số dư v2 so với v1 ngay trước khi chuyển.
    pub balances_diff: i64,
    /// Số sự kiện số dư v2 phải bắt kịp trong transaction chuyển.
    pub balances_caught_up: usize,
    /// Chuyển thu nhập v1 → v2 khi đòi «giống hệt» có bị chặn không: số dòng lệch.
    pub income_blocked_diff: Option<i64>,
    /// Số dòng thu nhập khác nhau giữa v1 và v2 khi được phép chuyển.
    pub income_diff: i64,
    /// Số sự kiện thu nhập v2 phải bắt kịp.
    pub income_caught_up: usize,
    /// Thu nhập v1 trước khi chuyển.
    pub income_before: Vec<IncomeRow>,
    /// Thu nhập sau khi chuyển (bảng `income_by_day` giờ là v2).
    pub income_after: Vec<IncomeRow>,
    /// Chuẩn theo giờ Việt Nam.
    pub income_truth_vn: Vec<IncomeRow>,
    /// Bảng v1 cũ giữ lại có còn đúng với chuẩn UTC không.
    pub old_income_kept: bool,
    /// Số dư sau chuyển có bằng chuẩn không.
    pub balances_equal_truth: bool,
    /// `(name, version, active)` trong checkpoint sau khi chuyển.
    pub checkpoints: Vec<(String, i32, bool)>,
    /// Bản v1 cũ chạy lô sau khi bị thay thế có bị từ chối không.
    pub old_version_refused: bool,
}

/// Thí nghiệm 4: dựng lại số dư (cùng logic) và thu nhập (sửa múi giờ) khi sổ vẫn nhận bút toán.
///
/// # Errors
/// Lỗi SQL hoặc projection.
pub async fn rebuild(lab: &Lab) -> Result<Rebuild> {
    let pool = &lab.pool;
    let (b1, b2) = (Balances { version: 1 }, Balances { version: 2 });
    let (i1, i2) = (IncomeByDay { version: 1 }, IncomeByDay { version: 2 });
    install(pool, &b1).await?;
    install(pool, &i1).await?;
    append_steps(pool, &STANDARD).await?;
    drain(pool, &b1, Role::Live, ReadMode::Safe, 3).await?;
    drain(pool, &i1, Role::Live, ReadMode::Safe, 3).await?;

    // Dựng lại từ đầu vào bảng mới; mới chạy hai lô thì có chuyến mới.
    start_rebuild(pool, &b2).await?;
    start_rebuild(pool, &i2).await?;
    let mut partial = 0;
    partial += run_batch(pool, &b2, Role::Build, ReadMode::Safe, 2).await?;
    partial += run_batch(pool, &b2, Role::Build, ReadMode::Safe, 2).await?;
    run_batch(pool, &i2, Role::Build, ReadMode::Safe, 2).await?;
    run_batch(pool, &i2, Role::Build, ReadMode::Safe, 2).await?;
    append_steps(pool, &TN_0004).await?;
    drain(pool, &b1, Role::Live, ReadMode::Safe, 3).await?;
    drain(pool, &i1, Role::Live, ReadMode::Safe, 3).await?;
    let income_before = read_income(pool, "income_by_day").await?;

    // Số dư: cùng logic nên đòi giống hệt. Bản dựng lại bắt kịp phần còn lại bên trong transaction.
    let bal = cutover(pool, &b1, &b2, true).await?;

    // Thu nhập: logic đổi nên không thể giống. Đòi giống thì bị chặn, và transaction rollback.
    let income_blocked_diff = match cutover(pool, &i1, &i2, true).await {
        Err(ProjectionError::Mismatch { diff_rows }) => Some(diff_rows),
        Ok(_) => None,
        Err(other) => return Err(other.into()),
    };
    let inc = cutover(pool, &i1, &i2, false).await?;

    let checkpoints = sqlx::query(
        "SELECT name, version, active FROM projection_checkpoints ORDER BY name, version",
    )
    .fetch_all(pool)
    .await?
    .iter()
    .map(|r| (r.get("name"), r.get("version"), r.get("active")))
    .collect();
    let old_version_refused = matches!(
        run_batch(pool, &b1, Role::Live, ReadMode::Safe, 3).await,
        Err(ProjectionError::WrongRole { .. })
    );
    Ok(Rebuild {
        partial,
        balances_diff: bal.diff_rows,
        balances_caught_up: bal.caught_up,
        income_blocked_diff,
        income_diff: inc.diff_rows,
        income_caught_up: inc.caught_up,
        income_before,
        income_after: read_income(pool, "income_by_day").await?,
        income_truth_vn: truth_income(pool, "Asia/Ho_Chi_Minh").await?,
        old_income_kept: read_income(pool, "income_by_day_v1").await?
            == truth_income(pool, "UTC").await?,
        balances_equal_truth: read_balances(pool, "balances").await?
            == truth_balances(pool).await?,
        checkpoints,
        old_version_refused,
    })
}

impl Rebuild {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không như kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.partial == 4, "dựng dở");
        ensure!(self.balances_diff == 0, "số dư v2 phải giống v1");
        ensure!(
            self.balances_caught_up == 6,
            "bắt kịp {}",
            self.balances_caught_up
        );
        ensure!(self.balances_equal_truth, "số dư sau chuyển");
        ensure!(
            self.income_blocked_diff.is_some_and(|d| d > 0),
            "phải bị chặn"
        );
        ensure!(self.income_diff > 0, "v1 và v2 phải khác nhau");
        ensure!(
            self.income_caught_up == 6,
            "bắt kịp {}",
            self.income_caught_up
        );
        ensure!(
            self.income_after == self.income_truth_vn,
            "thu nhập v2 bằng chuẩn VN"
        );
        ensure!(self.income_before != self.income_after, "số liệu phải đổi");
        ensure!(self.old_income_kept, "bảng v1 cũ giữ nguyên");
        ensure!(self.old_version_refused, "bản cũ phải bị từ chối");
        let expected = vec![
            ("balances".to_string(), 1, false),
            ("balances".to_string(), 2, true),
            ("income_by_day".to_string(), 1, false),
            ("income_by_day".to_string(), 2, true),
        ];
        ensure!(
            self.checkpoints == expected,
            "checkpoint {:?}",
            self.checkpoints
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------
// 5. Độ trễ và tốc độ dựng lại

/// Nạp `trips` chuyến (mỗi chuyến hai bút toán: capture và split) bằng SQL, trong một transaction.
///
/// # Errors
/// Lỗi SQL.
pub async fn bulk_load(pool: &PgPool, trips: i64) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO accounts (id, kind) SELECT 'customer:BULK-' || g, 'liability' FROM generate_series(1, 50) g",
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO accounts (id, kind) SELECT 'driver:BULK-' || g, 'liability' FROM generate_series(1, 20) g",
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO journal_entries (idempotency_key, posted_at) \
         SELECT 'bulk:' || g || ':' || k.kind, \
                timestamptz '2026-11-01 00:00:00+00' + (g * 2 + (k.kind = 'split')::int) * interval '1 minute' \
         FROM generate_series(1, $1::int) g, (VALUES ('capture'), ('split')) k(kind) \
         ORDER BY g, (k.kind = 'split')",
    )
    .bind(trips)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
         SELECT e.id, v.line_no, v.account_id, v.amount \
         FROM journal_entries e \
         CROSS JOIN LATERAL (SELECT split_part(e.idempotency_key, ':', 2)::int AS g, \
                                    split_part(e.idempotency_key, ':', 3) AS kind) k \
         CROSS JOIN LATERAL (VALUES \
            (1, 'customer:BULK-' || (k.g % 50 + 1), -25500::bigint, k.kind = 'capture'), \
            (2, 'psp:vixanh:clearing', 25500::bigint, k.kind = 'capture'), \
            (1, 'psp:vixanh:clearing', -25500::bigint, k.kind = 'split'), \
            (2, 'platform:revenue', 5100::bigint, k.kind = 'split'), \
            (3, 'driver:BULK-' || (k.g % 20 + 1), 20400::bigint, k.kind = 'split') \
         ) v(line_no, account_id, amount, applies) \
         WHERE e.idempotency_key LIKE 'bulk:%' AND v.applies",
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Kết quả thí nghiệm 5.
pub struct Lag {
    /// Số bút toán trong log.
    pub entries: i64,
    /// Lag (số bút toán chưa chiếu) sau mỗi lô, bắt đầu từ trước lô đầu.
    pub behind: Vec<i64>,
    /// Số bút toán một lô.
    pub batch: i64,
    /// Thời gian dựng lại toàn bộ vào bảng mới (chỉ để in, không assert).
    pub rebuild_elapsed: Duration,
    /// Bảng dựng lại có giống bảng đang chạy không.
    pub rebuild_diff: i64,
    /// Số dòng của bảng số dư.
    pub balance_rows: usize,
}

/// Thí nghiệm 5: lag giảm theo từng lô, rồi đo thời gian dựng lại từ đầu.
///
/// # Errors
/// Lỗi SQL hoặc projection.
pub async fn lag(lab: &Lab, trips: i64, batch: i64) -> Result<Lag> {
    let pool = &lab.pool;
    let b1 = Balances { version: 1 };
    install(pool, &b1).await?;
    bulk_load(pool, trips).await?;
    wait_horizon_past_head(pool).await?;
    let mut conn = pool.acquire().await?;
    let entries = entries_behind(&mut conn, Cursor::default()).await?;
    drop(conn);
    let mut series = vec![entries];
    while run_batch(pool, &b1, Role::Live, ReadMode::Safe, batch).await? > 0 {
        let cursor = checkpoint(pool, "balances", 1).await?;
        let mut conn = pool.acquire().await?;
        series.push(entries_behind(&mut conn, cursor).await?);
    }

    let b2 = Balances { version: 2 };
    let started = Instant::now();
    start_rebuild(pool, &b2).await?;
    drain(pool, &b2, Role::Build, ReadMode::Safe, batch).await?;
    let rebuild_elapsed = started.elapsed();
    let report = cutover(pool, &b1, &b2, true).await?;
    Ok(Lag {
        entries,
        behind: series,
        batch,
        rebuild_elapsed,
        rebuild_diff: report.diff_rows,
        balance_rows: read_balances(pool, "balances").await?.len(),
    })
}

impl Lag {
    /// Tốc độ dựng lại (bút toán mỗi giây), làm tròn xuống chục; chỉ để in số đo.
    pub fn entries_per_second(&self) -> i64 {
        let millis = i64::try_from(self.rebuild_elapsed.as_millis())
            .unwrap_or(i64::MAX)
            .max(1);
        self.entries * 1000 / millis / 10 * 10
    }

    /// Kiểm các giá trị bài viết nêu (không kiểm thời gian).
    ///
    /// # Errors
    /// Giá trị không như kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.entries == 5_000, "số bút toán");
        ensure!(self.behind.first() == Some(&5_000), "lag đầu");
        ensure!(self.behind.last() == Some(&0), "lag cuối");
        ensure!(
            self.behind.windows(2).all(|w| w[0] - w[1] == self.batch),
            "mỗi lô giảm đúng {} bút toán",
            self.batch
        );
        ensure!(self.rebuild_diff == 0, "dựng lại phải giống bảng đang chạy");
        // 50 khách + 20 tài xế + doanh thu + clearing.
        ensure!(self.balance_rows == 72, "số dòng {}", self.balance_rows);
        Ok(())
    }
}

/// Có bút toán nào trong log không (kiểm nhanh dùng trong test).
///
/// # Errors
/// Lỗi SQL.
pub async fn log_is_empty(pool: &PgPool) -> Result<bool> {
    let mut conn = pool.acquire().await?;
    Ok(head(&mut conn).await?.is_none())
}
