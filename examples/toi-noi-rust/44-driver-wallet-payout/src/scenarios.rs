//! Các thí nghiệm của bài 44. Mỗi thí nghiệm trả một struct dữ liệu và có `verify()` kiểm
//! các giá trị bài viết nêu. Không `sleep`, không assert thời gian: thời gian nghiệp vụ đi
//! qua `FixedClock`, cuộc đua dựng bằng `Barrier` và kiểm bất biến.

use std::sync::Arc;

use anyhow::{Result, anyhow, ensure};
use ledger_core::Money;
use sqlx::{PgPool, Row};
use tokio::sync::Barrier;

use crate::bank::{Bank, Behavior, FakeBank, LineStatus, StatementLine};
use crate::clock::{FixedClock, LOCAL_TZ, local_date, local_day_bounds};
use crate::earnings::{DayEarning, earnings_by_day, sql_local_day};
use crate::fixture::{seed_standard, seed_uniform};
use crate::lab::PgLab;
use crate::payout::{
    BatchRequest, CLEARING, Created, PayoutRow, Policy, Selection, SendReport, Status,
    batch_status, create_batch, load_payouts, payout_key, remaining_limit, send_batch,
};
use crate::reconcile::{Report, reconcile};
use crate::store::{balance, count_entries, grand_total, insert_entry, postings_sum};

fn request(code: &str, period: &str, selection: Selection, policy: Policy) -> BatchRequest {
    BatchRequest {
        code: code.to_string(),
        period: period.to_string(),
        policy,
        selection,
    }
}

/// Số dư ví của các tài xế, theo thứ tự đã cho.
async fn wallets(pool: &PgPool, drivers: &[&str]) -> Result<Vec<(String, i64)>> {
    let mut out = Vec::new();
    for driver in drivers {
        out.push((
            (*driver).to_string(),
            balance(pool, &format!("driver:{driver}")).await?,
        ));
    }
    Ok(out)
}

/// Mọi `account_balances` phải bằng tổng posting của chính tài khoản đó.
async fn projection_matches(pool: &PgPool) -> Result<bool> {
    let rows = sqlx::query("SELECT account_id FROM account_balances")
        .fetch_all(pool)
        .await?;
    for row in &rows {
        let id: String = row.get("account_id");
        if balance(pool, &id).await? != postings_sum(pool, &id).await? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn statuses(rows: &[PayoutRow]) -> Vec<(String, Status)> {
    rows.iter()
        .map(|p| (p.driver_id.clone(), p.status))
        .collect()
}

/// Mã SQLSTATE nếu lỗi bắt nguồn từ PostgreSQL.
pub fn sqlstate(err: &anyhow::Error) -> Option<String> {
    err.downcast_ref::<sqlx::Error>()
        .and_then(|e| e.as_database_error())
        .and_then(|d| d.code().map(|c| c.to_string()))
}

/// Số dư theo tài xế.
pub type Wallets = Vec<(String, i64)>;

const FIVE: [&str; 5] = ["DR-001", "DR-002", "DR-003", "DR-004", "DR-005"];

// ---------------------------------------------------------------- 1. thu nhập theo ngày

/// Một mốc sát nửa đêm: PostgreSQL và `jiff` phải gán cùng một ngày.
#[derive(Debug, Clone)]
pub struct Boundary {
    /// Thời điểm UTC.
    pub at: String,
    /// Ngày theo UTC.
    pub utc_day: String,
    /// Ngày giờ Việt Nam do PostgreSQL tính.
    pub sql_day: String,
    /// Ngày giờ Việt Nam do `jiff` tính.
    pub jiff_day: String,
}

/// Kết quả thí nghiệm 1.
#[derive(Debug, Clone)]
pub struct EarningsReport {
    /// Thu nhập `DR-001` theo ngày giờ Việt Nam.
    pub ict: Vec<DayEarning>,
    /// Cùng dữ liệu, cắt ngày theo UTC.
    pub utc: Vec<DayEarning>,
    /// Các mốc sát nửa đêm.
    pub boundaries: Vec<Boundary>,
    /// Cửa sổ ngày giờ Việt Nam của `2026-11-06T10:00:00Z`, đổi về UTC.
    pub window: (String, String),
}

/// Thí nghiệm 1: thu nhập theo ngày, chuyến qua nửa đêm UTC.
///
/// # Errors
/// Lỗi SQL hoặc múi giờ.
pub async fn earnings(lab: &PgLab) -> Result<EarningsReport> {
    seed_standard(&lab.pool).await?;
    let ict = earnings_by_day(&lab.pool, "DR-001", LOCAL_TZ).await?;
    let utc = earnings_by_day(&lab.pool, "DR-001", "UTC").await?;
    let mut boundaries = Vec::new();
    for at in [
        "2026-11-05T16:59:59Z",
        "2026-11-05T17:00:00Z",
        "2026-11-06T16:59:59Z",
        "2026-11-06T17:00:00Z",
    ] {
        boundaries.push(Boundary {
            at: at.to_string(),
            utc_day: sql_local_day(&lab.pool, at, "UTC").await?,
            sql_day: sql_local_day(&lab.pool, at, LOCAL_TZ).await?,
            jiff_day: local_date(at.parse()?)?,
        });
    }
    let (from, to) = local_day_bounds("2026-11-06T10:00:00Z".parse()?)?;
    Ok(EarningsReport {
        ict,
        utc,
        boundaries,
        window: (from.to_string(), to.to_string()),
    })
}

impl EarningsReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không khớp.
    pub fn verify(&self) -> Result<()> {
        let day = |d: &str, trips, earned| DayEarning {
            day: d.to_string(),
            trips,
            earned_dong: earned,
        };
        ensure!(
            self.ict == [day("2026-11-05", 2, 40_800), day("2026-11-06", 3, 61_200)],
            "ICT: {:?}",
            self.ict
        );
        ensure!(
            self.utc == [day("2026-11-05", 4, 81_600), day("2026-11-06", 1, 20_400)],
            "UTC: {:?}",
            self.utc
        );
        for b in &self.boundaries {
            ensure!(
                b.sql_day == b.jiff_day,
                "PostgreSQL và jiff lệch ngày ở {}",
                b.at
            );
        }
        let days: Vec<&str> = self.boundaries.iter().map(|b| b.sql_day.as_str()).collect();
        ensure!(
            days == ["2026-11-05", "2026-11-06", "2026-11-06", "2026-11-07"],
            "{days:?}"
        );
        ensure!(
            self.window
                == (
                    "2026-11-05T17:00:00Z".to_string(),
                    "2026-11-06T17:00:00Z".to_string()
                )
        );
        Ok(())
    }
}

// ---------------------------------------------------------------- 2. một lô chạy trọn

/// Kết quả thí nghiệm 2: một lô qua đủ CREATED, SENT, SETTLED, FAILED.
#[derive(Debug)]
pub struct FlowReport {
    /// Ví trước khi tạo lô.
    pub wallets_before: Vec<(String, i64)>,
    /// Lô vừa tạo.
    pub created: Created,
    /// Ví sau khi tạo lô.
    pub wallets_after_create: Vec<(String, i64)>,
    /// Số dư `bank:payout:clearing` sau khi tạo lô.
    pub clearing_after_create: i64,
    /// Trạng thái lô sau khi tạo.
    pub batch_after_create: Status,
    /// Lượt gửi thứ nhất.
    pub send1: SendReport,
    /// Trạng thái từng khoản sau lượt gửi 1.
    pub statuses_after_send1: Vec<(String, Status)>,
    /// Trạng thái lô sau lượt gửi 1.
    pub batch_after_send1: Status,
    /// Ví `DR-004` sau khi ngân hàng từ chối.
    pub dr004_after_reject: i64,
    /// Lượt gửi thứ hai (gửi lại khoản timeout).
    pub send2: SendReport,
    /// Trạng thái lô sau lượt gửi 2.
    pub batch_after_send2: Status,
    /// Số lệnh ngân hàng đã nhận cho `DR-001` (timeout rồi gửi lại cùng mã).
    pub bank_orders_dr001: usize,
    /// Số lần gửi `DR-001` ghi trong DB.
    pub attempts_dr001: i32,
    /// Kết quả đối chiếu.
    pub reconcile: Report,
    /// Khoản chi sau đối chiếu.
    pub final_rows: Vec<PayoutRow>,
    /// Trạng thái lô cuối.
    pub batch_final: Status,
    /// Ví cuối.
    pub wallets_final: Vec<(String, i64)>,
    /// Clearing cuối.
    pub clearing_final: i64,
    /// Mỗi khoản FAILED: bút toán đảo trỏ đúng bút toán gốc.
    pub reversal_links: Vec<(String, bool)>,
    /// Tổng mọi posting.
    pub grand_total: i64,
    /// `account_balances` khớp tổng posting.
    pub projection_ok: bool,
}

/// Thí nghiệm 2.
///
/// # Errors
/// Lỗi SQL hoặc ngân hàng.
pub async fn payout_flow(lab: &PgLab) -> Result<FlowReport> {
    let pool = &lab.pool;
    seed_standard(pool).await?;
    let clock = FixedClock::at("2026-11-06T10:00:00Z")?;
    let bank = FakeBank::new();
    bank.set_behavior("DR-001", Behavior::TimeoutOnce);
    bank.set_behavior(
        "DR-004",
        Behavior::Reject("tài khoản nhận không hợp lệ".into()),
    );
    bank.set_behavior("DR-005", Behavior::ReturnedLater);

    let wallets_before = wallets(pool, &FIVE).await?;
    let req = request(
        "PB-2026-11-06.1",
        "2026-11-06.1",
        Selection::SkipLocked,
        Policy::standard(),
    );
    let created = create_batch(pool, &clock, &req, None).await?;
    let id = created.batch_id;
    let wallets_after_create = wallets(pool, &FIVE).await?;
    let clearing_after_create = balance(pool, CLEARING).await?;
    let batch_after_create = batch_status(pool, id).await?;

    let send1 = send_batch(pool, &bank, &clock, id).await?;
    let statuses_after_send1 = statuses(&load_payouts(pool, id).await?);
    let batch_after_send1 = batch_status(pool, id).await?;
    let dr004_after_reject = balance(pool, "driver:DR-004").await?;

    let send2 = send_batch(pool, &bank, &clock, id).await?;
    let batch_after_send2 = batch_status(pool, id).await?;
    let rows = load_payouts(pool, id).await?;
    let dr001 = rows
        .iter()
        .find(|p| p.driver_id == "DR-001")
        .ok_or_else(|| anyhow!("thiếu DR-001"))?;
    let attempts_dr001 = dr001.attempts;
    let bank_orders_dr001 = bank.received_count("DR-001");

    let reconcile = reconcile(pool, &bank, &clock, id).await?;
    let final_rows = load_payouts(pool, id).await?;
    let batch_final = batch_status(pool, id).await?;
    let wallets_final = wallets(pool, &FIVE).await?;
    let clearing_final = balance(pool, CLEARING).await?;
    let links = sqlx::query(
        "SELECT p.payout_key, e.reversal_of = p.entry_id AS linked \
           FROM payouts p JOIN journal_entries e ON e.id = p.reversal_entry_id \
          ORDER BY p.id",
    )
    .fetch_all(pool)
    .await?;
    let reversal_links = links
        .iter()
        .map(|r| (r.get("payout_key"), r.get("linked")))
        .collect();
    Ok(FlowReport {
        wallets_before,
        created,
        wallets_after_create,
        clearing_after_create,
        batch_after_create,
        send1,
        statuses_after_send1,
        batch_after_send1,
        dr004_after_reject,
        send2,
        batch_after_send2,
        bank_orders_dr001,
        attempts_dr001,
        reconcile,
        final_rows,
        batch_final,
        wallets_final,
        clearing_final,
        reversal_links,
        grand_total: grand_total(pool).await?,
        projection_ok: projection_matches(pool).await?,
    })
}

fn pairs(items: &[(&str, i64)]) -> Vec<(String, i64)> {
    items.iter().map(|(d, n)| ((*d).to_string(), *n)).collect()
}

impl FlowReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không khớp.
    pub fn verify(&self) -> Result<()> {
        ensure!(
            self.wallets_before
                == pairs(&[
                    ("DR-001", 102_000),
                    ("DR-002", 40_800),
                    ("DR-003", 244_800),
                    ("DR-004", 61_200),
                    ("DR-005", 81_600)
                ])
        );
        ensure!(
            self.created.items
                == pairs(&[
                    ("DR-001", 102_000),
                    ("DR-003", 150_000),
                    ("DR-004", 61_200),
                    ("DR-005", 81_600)
                ]),
            "{:?}",
            self.created.items
        );
        ensure!(self.created.skipped.is_empty());
        ensure!(
            self.wallets_after_create
                == pairs(&[
                    ("DR-001", 0),
                    ("DR-002", 40_800),
                    ("DR-003", 94_800),
                    ("DR-004", 0),
                    ("DR-005", 0)
                ])
        );
        ensure!(self.clearing_after_create == 394_800);
        ensure!(self.batch_after_create == Status::Created);
        ensure!(
            self.send1
                == SendReport {
                    sent: 2,
                    rejected: 1,
                    timed_out: 1
                },
            "{:?}",
            self.send1
        );
        let st = |d: &str, s| (d.to_string(), s);
        ensure!(
            self.statuses_after_send1
                == [
                    st("DR-001", Status::Created),
                    st("DR-003", Status::Sent),
                    st("DR-004", Status::Failed),
                    st("DR-005", Status::Sent)
                ]
        );
        ensure!(self.batch_after_send1 == Status::Created);
        ensure!(
            self.dr004_after_reject == 61_200,
            "ví DR-004 phải được trả lại"
        );
        ensure!(
            self.send2
                == SendReport {
                    sent: 1,
                    rejected: 0,
                    timed_out: 0
                },
            "{:?}",
            self.send2
        );
        ensure!(self.batch_after_send2 == Status::Sent);
        ensure!(
            self.bank_orders_dr001 == 1,
            "gửi lại cùng mã không được chi thêm"
        );
        ensure!(
            self.attempts_dr001 == 1,
            "tra cứu thấy lệnh thì không gửi lại"
        );
        let r = &self.reconcile;
        ensure!(
            r.settled
                == [
                    payout_key("PB-2026-11-06.1", "DR-001"),
                    payout_key("PB-2026-11-06.1", "DR-003")
                ],
            "{r:?}"
        );
        ensure!(r.returned == [payout_key("PB-2026-11-06.1", "DR-005")]);
        ensure!(r.recovered_after_timeout.is_empty() && r.amount_mismatch.is_empty());
        ensure!(r.missing_in_statement.is_empty() && r.unknown_in_statement.is_empty());
        let fin: Vec<(String, Status)> = statuses(&self.final_rows);
        ensure!(
            fin == [
                st("DR-001", Status::Settled),
                st("DR-003", Status::Settled),
                st("DR-004", Status::Failed),
                st("DR-005", Status::Failed)
            ],
            "{fin:?}"
        );
        ensure!(self.batch_final == Status::Settled);
        ensure!(
            self.wallets_final
                == pairs(&[
                    ("DR-001", 0),
                    ("DR-002", 40_800),
                    ("DR-003", 94_800),
                    ("DR-004", 61_200),
                    ("DR-005", 81_600)
                ])
        );
        ensure!(self.clearing_final == 252_000);
        ensure!(self.reversal_links.len() == 2 && self.reversal_links.iter().all(|(_, ok)| *ok));
        ensure!(self.grand_total == 0, "tổng mọi posting phải bằng 0");
        ensure!(self.projection_ok, "account_balances lệch posting");
        Ok(())
    }
}

// ---------------------------------------------------------------- 3. giới hạn rút

/// Kết quả thí nghiệm 3: hạn mức rút theo ngày giờ Việt Nam, đồng hồ truyền vào.
#[derive(Debug)]
pub struct LimitReport {
    /// Lô 1 lúc 10:00Z: khoản chi của `DR-003`.
    pub first_dr003: Option<i64>,
    /// Hạn mức còn lại của `DR-003` lúc 16:59:59Z.
    pub remaining_before_midnight: i64,
    /// Lô 2 lúc 16:59:59Z: ví bị bỏ qua.
    pub second_skipped: Vec<crate::payout::Skipped>,
    /// Số khoản của lô 2.
    pub second_items: usize,
    /// Hạn mức còn lại của `DR-003` lúc 17:00:00Z (đã sang ngày 07/11 giờ Việt Nam).
    pub remaining_after_midnight: i64,
    /// Lô 3 lúc 17:00:00Z.
    pub third_items: Vec<(String, i64)>,
    /// Số đã rút của `DR-003` nếu cắt ngày theo UTC lúc 17:00:00Z.
    pub used_if_utc_day: i64,
}

/// Thí nghiệm 3.
///
/// # Errors
/// Lỗi SQL hoặc múi giờ.
pub async fn withdrawal_limit(lab: &PgLab) -> Result<LimitReport> {
    let pool = &lab.pool;
    seed_standard(pool).await?;
    let policy = Policy::standard();
    let clock = FixedClock::at("2026-11-06T10:00:00Z")?;
    let first = create_batch(
        pool,
        &clock,
        &request(
            "PB-2026-11-06.1",
            "2026-11-06.1",
            Selection::SkipLocked,
            policy,
        ),
        None,
    )
    .await?;
    let first_dr003 = first
        .items
        .iter()
        .find(|(d, _)| d == "DR-003")
        .map(|(_, n)| *n);

    clock.set("2026-11-06T16:59:59Z")?;
    let remaining_before_midnight =
        remaining_limit(pool, &policy, "DR-003", clock.now_ts()).await?;
    let second = create_batch(
        pool,
        &clock,
        &request(
            "PB-2026-11-06.2",
            "2026-11-06.2",
            Selection::SkipLocked,
            policy,
        ),
        None,
    )
    .await?;

    clock.set("2026-11-06T17:00:00Z")?;
    let remaining_after_midnight = remaining_limit(pool, &policy, "DR-003", clock.now_ts()).await?;
    // Cửa sổ UTC sẽ coi DR-003 đã dùng bao nhiêu ở thời điểm này (trước khi tạo lô 3)?
    let row = sqlx::query(
        "SELECT COALESCE(SUM(amount_dong), 0)::bigint AS used FROM payouts \
          WHERE driver_id = 'DR-003' AND status <> 'FAILED' \
            AND (created_at AT TIME ZONE 'UTC')::date = '2026-11-06'",
    )
    .fetch_one(pool)
    .await?;
    let third = create_batch(
        pool,
        &clock,
        &request(
            "PB-2026-11-07.1",
            "2026-11-07.1",
            Selection::SkipLocked,
            policy,
        ),
        None,
    )
    .await?;

    Ok(LimitReport {
        first_dr003,
        remaining_before_midnight,
        second_skipped: second.skipped,
        second_items: second.items.len(),
        remaining_after_midnight,
        third_items: third.items,
        used_if_utc_day: row.get("used"),
    })
}

impl LimitReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không khớp.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.first_dr003 == Some(150_000));
        ensure!(self.remaining_before_midnight == 0);
        ensure!(self.second_items == 0);
        ensure!(self.second_skipped.len() == 1, "{:?}", self.second_skipped);
        let s = &self.second_skipped[0];
        ensure!(s.driver_id == "DR-003" && s.balance_dong == 94_800 && s.remaining_limit_dong == 0);
        ensure!(self.remaining_after_midnight == 150_000);
        ensure!(
            self.third_items == pairs(&[("DR-003", 94_800)]),
            "{:?}",
            self.third_items
        );
        ensure!(self.used_if_utc_day == 150_000);
        Ok(())
    }
}

// ---------------------------------------------------------------- 4. hai lô song song

/// Ba cách chạy hai lô song song.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParallelMode {
    /// `SELECT` thường và không có chỉ mục unique theo kỳ.
    NaiveNoUnique,
    /// `SELECT` thường nhưng còn chỉ mục unique theo kỳ.
    NaiveUnique,
    /// `FOR UPDATE SKIP LOCKED` (và chỉ mục unique theo kỳ).
    SkipLocked,
}

/// Kết quả thí nghiệm 4.
#[derive(Debug)]
pub struct ParallelReport {
    /// Cách chạy.
    pub mode: ParallelMode,
    /// Kết quả lô A rồi lô B: danh sách tài xế hoặc SQLSTATE của lỗi.
    pub outcomes: Vec<Result<Vec<String>, String>>,
    /// Số khoản chi còn hiệu lực của từng tài xế.
    pub payouts_per_driver: Vec<(String, i64)>,
    /// Ví cuối.
    pub wallets: Vec<(String, i64)>,
    /// Số lô còn lại trong DB.
    pub batches: i64,
}

/// Thí nghiệm 4: năm ví đủ ngưỡng, mỗi lô lấy tối đa 3 ví, hai lô cùng chờ ở hàng rào
/// sau bước chọn nên cả hai đã chọn xong trước khi bên nào ghi.
///
/// # Errors
/// Lỗi hạ tầng (lỗi nghiệp vụ của từng lô nằm trong `outcomes`).
pub async fn parallel_batches(lab: &PgLab, mode: ParallelMode) -> Result<ParallelReport> {
    let pool = lab.pool.clone();
    seed_uniform(&pool, 5, 3).await?;
    if mode == ParallelMode::NaiveNoUnique {
        sqlx::query("DROP INDEX payouts_period_driver_uq")
            .execute(&pool)
            .await?;
    }
    let selection = if mode == ParallelMode::SkipLocked {
        Selection::SkipLocked
    } else {
        Selection::Naive
    };
    let policy = Policy {
        max_items: 3,
        ..Policy::standard()
    };
    let clock = Arc::new(FixedClock::at("2026-11-06T10:00:00Z")?);
    let barrier = Arc::new(Barrier::new(2));
    let mut tasks = Vec::new();
    for code in ["PB-2026-11-06.1-A", "PB-2026-11-06.1-B"] {
        let (pool, clock, barrier) = (pool.clone(), clock.clone(), barrier.clone());
        let req = request(code, "2026-11-06.1", selection, policy);
        tasks.push(tokio::spawn(async move {
            create_batch(&pool, &*clock, &req, Some(&*barrier)).await
        }));
    }
    let mut outcomes = Vec::new();
    for task in tasks {
        outcomes.push(match task.await? {
            Ok(created) => Ok(created.items.into_iter().map(|(d, _)| d).collect()),
            Err(err) => Err(sqlstate(&err).unwrap_or_else(|| format!("{err:#}"))),
        });
    }
    let rows = sqlx::query(
        "SELECT driver_id, count(*) AS n FROM payouts WHERE status <> 'FAILED' GROUP BY driver_id",
    )
    .fetch_all(&pool)
    .await?;
    let counted: std::collections::HashMap<String, i64> = rows
        .iter()
        .map(|r| (r.get("driver_id"), r.get("n")))
        .collect();
    let payouts_per_driver = FIVE
        .iter()
        .map(|d| ((*d).to_string(), counted.get(*d).copied().unwrap_or(0)))
        .collect();
    let batches: i64 = sqlx::query("SELECT count(*) AS n FROM payout_batches")
        .fetch_one(&pool)
        .await?
        .get("n");
    Ok(ParallelReport {
        mode,
        outcomes,
        payouts_per_driver,
        wallets: wallets(&pool, &FIVE).await?,
        batches,
    })
}

impl ParallelReport {
    /// Kiểm bất biến của từng cách chạy.
    ///
    /// # Errors
    /// Bất biến bị vi phạm.
    pub fn verify(&self) -> Result<()> {
        let first3 = vec![
            "DR-001".to_string(),
            "DR-002".to_string(),
            "DR-003".to_string(),
        ];
        match self.mode {
            ParallelMode::NaiveNoUnique => {
                ensure!(
                    self.outcomes
                        .iter()
                        .all(|o| o.as_ref().ok() == Some(&first3)),
                    "{:?}",
                    self.outcomes
                );
                ensure!(
                    self.payouts_per_driver
                        == pairs(&[
                            ("DR-001", 2),
                            ("DR-002", 2),
                            ("DR-003", 2),
                            ("DR-004", 0),
                            ("DR-005", 0)
                        ])
                );
                ensure!(
                    self.wallets
                        == pairs(&[
                            ("DR-001", -61_200),
                            ("DR-002", -61_200),
                            ("DR-003", -61_200),
                            ("DR-004", 61_200),
                            ("DR-005", 61_200)
                        ])
                );
            }
            ParallelMode::NaiveUnique => {
                let ok = self.outcomes.iter().filter(|o| o.is_ok()).count();
                let dup = self
                    .outcomes
                    .iter()
                    .filter(|o| matches!(o, Err(code) if code == "23505"))
                    .count();
                ensure!(ok == 1 && dup == 1, "{:?}", self.outcomes);
                ensure!(
                    self.payouts_per_driver
                        == pairs(&[
                            ("DR-001", 1),
                            ("DR-002", 1),
                            ("DR-003", 1),
                            ("DR-004", 0),
                            ("DR-005", 0)
                        ])
                );
                ensure!(
                    self.wallets
                        == pairs(&[
                            ("DR-001", 0),
                            ("DR-002", 0),
                            ("DR-003", 0),
                            ("DR-004", 61_200),
                            ("DR-005", 61_200)
                        ])
                );
                ensure!(self.batches == 1, "lô lỗi phải rollback cả hàng lô");
            }
            ParallelMode::SkipLocked => {
                let sets: Vec<&Vec<String>> = self
                    .outcomes
                    .iter()
                    .filter_map(|o| o.as_ref().ok())
                    .collect();
                ensure!(sets.len() == 2, "{:?}", self.outcomes);
                let mut sizes = [sets[0].len(), sets[1].len()];
                sizes.sort_unstable();
                ensure!(sizes == [2, 3], "{sizes:?}");
                ensure!(
                    sets[0].iter().all(|d| !sets[1].contains(d)),
                    "hai lô chọn trùng tài xế"
                );
                ensure!(self.payouts_per_driver.iter().all(|(_, n)| *n == 1));
                ensure!(self.wallets.iter().all(|(_, w)| *w == 0));
                ensure!(self.batches == 2);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------- 5. timeout sau khi đã nhận lệnh

/// Kết quả thí nghiệm 5.
#[derive(Debug)]
pub struct TimeoutReport {
    /// Số khoản timeout ở lượt gửi 1.
    pub first_timed_out: usize,
    /// Trạng thái, số lần gửi và lỗi gần nhất sau lượt 1.
    pub after_first: (Status, i32, Option<String>),
    /// Ví `DR-001` sau lượt 1 (tiền đã rời ví từ lúc tạo lô).
    pub wallet_after_first: i64,
    /// Số lệnh ngân hàng đã nhận sau lượt 1.
    pub bank_orders_after_first: usize,
    /// Lượt gửi 2, khi tra cứu cũng lỗi.
    pub lookup_down: SendReport,
    /// Trạng thái khoản và số lần gọi `transfer` sau lượt 2.
    pub after_lookup_down: (Status, usize),
    /// Lượt gửi 3, tra cứu thấy lệnh đã có.
    pub second: SendReport,
    /// Trạng thái, số lần gửi và `bank_ref` sau lượt 3.
    pub after_second: (Status, i32, Option<String>),
    /// Số lệnh ngân hàng đã nhận sau lượt 3.
    pub bank_orders_after_second: usize,
    /// Số lần gọi `transfer` sau lượt 3.
    pub transfer_calls: usize,
    /// Số lệnh khi retry bằng mã MỚI trên một ngân hàng khác có cùng hành vi.
    pub bank_orders_with_new_reference: usize,
}

/// Thí nghiệm 5.
///
/// # Errors
/// Lỗi SQL hoặc ngân hàng.
pub async fn timeout_retry(lab: &PgLab) -> Result<TimeoutReport> {
    let pool = &lab.pool;
    seed_uniform(pool, 1, 3).await?;
    let clock = FixedClock::at("2026-11-06T10:00:00Z")?;
    let bank = FakeBank::new();
    bank.set_behavior("DR-001", Behavior::TimeoutOnce);
    let created = create_batch(
        pool,
        &clock,
        &request(
            "PB-2026-11-06.1",
            "2026-11-06.1",
            Selection::SkipLocked,
            Policy::standard(),
        ),
        None,
    )
    .await?;
    let id = created.batch_id;
    let pick = |rows: &[PayoutRow]| (rows[0].status, rows[0].attempts, rows[0].last_error.clone());

    let first = send_batch(pool, &bank, &clock, id).await?;
    let rows = load_payouts(pool, id).await?;
    let after_first = pick(&rows);
    let wallet_after_first = balance(pool, "driver:DR-001").await?;
    let bank_orders_after_first = bank.received_count("DR-001");

    bank.set_lookup_down(true);
    let lookup_down = send_batch(pool, &bank, &clock, id).await?;
    let rows = load_payouts(pool, id).await?;
    let after_lookup_down = (rows[0].status, bank.transfer_calls("DR-001"));
    bank.set_lookup_down(false);

    let second = send_batch(pool, &bank, &clock, id).await?;
    let rows = load_payouts(pool, id).await?;
    let after_second = (rows[0].status, rows[0].attempts, rows[0].bank_ref.clone());
    let bank_orders_after_second = bank.received_count("DR-001");
    let transfer_calls = bank.transfer_calls("DR-001");

    // Đối chứng: retry bằng mã mới trên một ngân hàng có cùng hành vi.
    let other = FakeBank::new();
    other.set_behavior("DR-001", Behavior::TimeoutOnce);
    let mut request = crate::bank::TransferRequest {
        reference: payout_key("PB-2026-11-06.1", "DR-001"),
        beneficiary: "DR-001".into(),
        amount: Money::from_dong(created.items[0].1),
    };
    let _ = other.transfer(&request).await; // timeout
    request.reference = format!("{}:retry2", request.reference); // mã mới mỗi lần retry
    other.transfer(&request).await?;
    Ok(TimeoutReport {
        first_timed_out: first.timed_out,
        after_first,
        wallet_after_first,
        bank_orders_after_first,
        lookup_down,
        after_lookup_down,
        second,
        after_second,
        bank_orders_after_second,
        transfer_calls,
        bank_orders_with_new_reference: other.received_count("DR-001"),
    })
}

impl TimeoutReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không khớp.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.first_timed_out == 1);
        ensure!(
            self.after_first == (Status::Created, 1, Some("timeout".to_string())),
            "{:?}",
            self.after_first
        );
        ensure!(self.wallet_after_first == 0);
        ensure!(
            self.bank_orders_after_first == 1,
            "ngân hàng đã nhận lệnh trước khi ta timeout"
        );
        ensure!(
            self.lookup_down
                == SendReport {
                    sent: 0,
                    rejected: 0,
                    timed_out: 1
                },
            "tra cứu lỗi thì không gửi"
        );
        ensure!(self.after_lookup_down == (Status::Created, 1));
        ensure!(
            self.second
                == SendReport {
                    sent: 1,
                    rejected: 0,
                    timed_out: 0
                }
        );
        ensure!(
            self.after_second == (Status::Sent, 1, Some("BK-00001".to_string())),
            "{:?}",
            self.after_second
        );
        ensure!(self.bank_orders_after_second == 1);
        ensure!(
            self.transfer_calls == 1,
            "tra cứu thấy lệnh thì không gọi lại"
        );
        ensure!(self.bank_orders_with_new_reference == 2);
        Ok(())
    }
}

/// Kết quả thí nghiệm 5b: lệnh mất trước khi tới ngân hàng.
#[derive(Debug)]
pub struct LostReport {
    /// Lượt gửi 1.
    pub first: SendReport,
    /// Số lệnh ngân hàng đã nhận sau lượt 1.
    pub orders_after_first: usize,
    /// Lượt gửi 2: tra cứu không thấy nên gửi lại.
    pub second: SendReport,
    /// Trạng thái, số lần gửi và mã tham chiếu đã chi sau lượt 2.
    pub after_second: (Status, i32, String),
    /// Số lệnh ngân hàng đã nhận sau lượt 2.
    pub orders_after_second: usize,
}

/// Thí nghiệm 5b.
///
/// # Errors
/// Lỗi SQL hoặc ngân hàng.
pub async fn lost_before_receipt(lab: &PgLab) -> Result<LostReport> {
    let pool = &lab.pool;
    seed_uniform(pool, 1, 3).await?;
    let clock = FixedClock::at("2026-11-06T10:00:00Z")?;
    let bank = FakeBank::new();
    bank.set_behavior("DR-001", Behavior::LostOnce);
    let created = create_batch(
        pool,
        &clock,
        &request(
            "PB-2026-11-06.1",
            "2026-11-06.1",
            Selection::SkipLocked,
            Policy::standard(),
        ),
        None,
    )
    .await?;
    let first = send_batch(pool, &bank, &clock, created.batch_id).await?;
    let orders_after_first = bank.received_count("DR-001");
    let second = send_batch(pool, &bank, &clock, created.batch_id).await?;
    let rows = load_payouts(pool, created.batch_id).await?;
    Ok(LostReport {
        first,
        orders_after_first,
        second,
        after_second: (rows[0].status, rows[0].attempts, rows[0].payout_key.clone()),
        orders_after_second: bank.received_count("DR-001"),
    })
}

impl LostReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không khớp.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.first.timed_out == 1);
        ensure!(self.orders_after_first == 0);
        ensure!(self.second.sent == 1);
        ensure!(
            self.after_second == (Status::Sent, 2, payout_key("PB-2026-11-06.1", "DR-001")),
            "{:?}",
            self.after_second
        );
        ensure!(self.orders_after_second == 1);
        Ok(())
    }
}

// ---------------------------------------------------------------- 6. đối chiếu lệch

/// Kết quả thí nghiệm 6.
#[derive(Debug)]
pub struct ReconcileReport {
    /// Lượt đối chiếu đầu.
    pub first: Report,
    /// Trạng thái từng khoản sau lượt đầu.
    pub statuses: Vec<(String, Status)>,
    /// Trạng thái lô sau lượt đầu.
    pub batch: Status,
    /// Lượt đối chiếu thứ hai (không đổi gì).
    pub second: Report,
    /// Ví sau lượt đầu và sau lượt hai.
    pub wallets: (Wallets, Wallets),
    /// Số bút toán đảo sau lượt hai.
    pub reversal_entries: i64,
}

/// Thí nghiệm 6: năm khoản, mỗi khoản một kiểu kết cục trên sao kê.
///
/// # Errors
/// Lỗi SQL hoặc ngân hàng.
pub async fn reconcile_anomalies(lab: &PgLab) -> Result<ReconcileReport> {
    let pool = &lab.pool;
    seed_uniform(pool, 5, 3).await?;
    let code = "PB-2026-11-06.1";
    let clock = FixedClock::at("2026-11-06T10:00:00Z")?;
    let bank = FakeBank::new();
    bank.set_behavior("DR-002", Behavior::TimeoutOnce); // ngân hàng nhận, ta không biết, không gửi lại
    bank.set_behavior("DR-005", Behavior::ReturnedLater);
    let created = create_batch(
        pool,
        &clock,
        &request(
            code,
            "2026-11-06.1",
            Selection::SkipLocked,
            Policy::standard(),
        ),
        None,
    )
    .await?;
    let id = created.batch_id;
    send_batch(pool, &bank, &clock, id).await?;
    bank.tamper_amount(&payout_key(code, "DR-003"), Money::from_dong(60_000));
    bank.hide_line(&payout_key(code, "DR-004"));
    bank.inject_line(StatementLine {
        reference: "payout:PB-2026-11-06.9:DR-999".into(),
        bank_ref: "BK-99999".into(),
        amount: Money::from_dong(20_000),
        status: LineStatus::Settled,
    });

    let first = reconcile(pool, &bank, &clock, id).await?;
    let statuses_after = statuses(&load_payouts(pool, id).await?);
    let batch = batch_status(pool, id).await?;
    let wallets_first = wallets(pool, &FIVE).await?;
    let second = reconcile(pool, &bank, &clock, id).await?;
    Ok(ReconcileReport {
        first,
        statuses: statuses_after,
        batch,
        second,
        wallets: (wallets_first, wallets(pool, &FIVE).await?),
        reversal_entries: count_entries(pool, "payout:%:reversal").await?,
    })
}

impl ReconcileReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không khớp.
    pub fn verify(&self) -> Result<()> {
        let key = |d| payout_key("PB-2026-11-06.1", d);
        let r = &self.first;
        ensure!(r.settled == [key("DR-001"), key("DR-002")], "{r:?}");
        ensure!(r.recovered_after_timeout == [key("DR-002")]);
        ensure!(r.returned == [key("DR-005")]);
        ensure!(r.amount_mismatch == [(key("DR-003"), 61_200, 60_000)]);
        ensure!(r.missing_in_statement == [key("DR-004")]);
        ensure!(r.unknown_in_statement == ["payout:PB-2026-11-06.9:DR-999".to_string()]);
        let st = |d: &str, s| (d.to_string(), s);
        ensure!(
            self.statuses
                == [
                    st("DR-001", Status::Settled),
                    st("DR-002", Status::Settled),
                    st("DR-003", Status::Sent),
                    st("DR-004", Status::Sent),
                    st("DR-005", Status::Failed)
                ],
            "{:?}",
            self.statuses
        );
        ensure!(self.batch == Status::Sent);
        ensure!(
            self.second.settled.is_empty()
                && self.second.returned.is_empty()
                && self.second.recovered_after_timeout.is_empty()
        );
        ensure!(
            self.second.amount_mismatch == r.amount_mismatch
                && self.second.missing_in_statement == r.missing_in_statement
        );
        ensure!(
            self.wallets.0 == self.wallets.1,
            "lượt hai không được đổi ví"
        );
        ensure!(self.reversal_entries == 1);
        Ok(())
    }
}

// ---------------------------------------------------------------- 7. idempotency theo mã lô

/// Kết quả thí nghiệm 7.
#[derive(Debug)]
pub struct IdempotencyReport {
    /// Số lệnh tạo lô trả `fresh = true` trong hai lệnh chạy đồng thời cùng mã.
    pub fresh_count: usize,
    /// Hai lệnh trả cùng id lô.
    pub same_batch: bool,
    /// Hai lệnh trả cùng danh sách khoản.
    pub same_items: bool,
    /// Số khoản và số bút toán `payout:*` trong DB.
    pub payouts_and_entries: (i64, i64),
    /// Chèn lại đúng bút toán của một khoản có tạo bản mới không.
    pub duplicate_entry_inserted: bool,
    /// Tổng `sent` của hai lượt gửi chạy đồng thời.
    pub concurrent_sent: usize,
    /// Lượt gửi thứ ba (tuần tự).
    pub third_send: SendReport,
    /// Số lệnh ngân hàng đã nhận của từng tài xế trong lô.
    pub bank_orders: Vec<(String, usize)>,
    /// Ví sau tất cả.
    pub wallets: Vec<(String, i64)>,
}

/// Thí nghiệm 7.
///
/// # Errors
/// Lỗi SQL hoặc ngân hàng.
pub async fn idempotency(lab: &PgLab) -> Result<IdempotencyReport> {
    let pool = &lab.pool;
    seed_standard(pool).await?;
    let clock = FixedClock::at("2026-11-06T10:00:00Z")?;
    let bank = FakeBank::new();
    let req = request(
        "PB-2026-11-06.1",
        "2026-11-06.1",
        Selection::SkipLocked,
        Policy::standard(),
    );

    let (a, b) = tokio::join!(
        create_batch(pool, &clock, &req, None),
        create_batch(pool, &clock, &req, None)
    );
    let (a, b) = (a?, b?);
    let payouts: i64 = sqlx::query("SELECT count(*) AS n FROM payouts")
        .fetch_one(pool)
        .await?
        .get("n");
    let entries = count_entries(pool, "payout:%").await?;

    // Chèn lại đúng key của một khoản: DB trả None, số dư không đổi.
    let entry = crate::payout::payout_entry(&payout_key(&req.code, "DR-001"), "DR-001", 102_000)?;
    let mut conn = pool.acquire().await?;
    let duplicate_entry_inserted = insert_entry(&mut conn, &entry, clock.now_ts(), None)
        .await?
        .is_some();
    drop(conn);

    let (s1, s2) = tokio::join!(
        send_batch(pool, &bank, &clock, a.batch_id),
        send_batch(pool, &bank, &clock, a.batch_id)
    );
    let concurrent_sent = s1?.sent + s2?.sent;
    let third_send = send_batch(pool, &bank, &clock, a.batch_id).await?;
    let bank_orders = ["DR-001", "DR-003", "DR-004", "DR-005"]
        .iter()
        .map(|d| ((*d).to_string(), bank.received_count(d)))
        .collect();
    Ok(IdempotencyReport {
        fresh_count: [a.fresh, b.fresh].iter().filter(|f| **f).count(),
        same_batch: a.batch_id == b.batch_id,
        same_items: a.items == b.items,
        payouts_and_entries: (payouts, entries),
        duplicate_entry_inserted,
        concurrent_sent,
        third_send,
        bank_orders,
        wallets: wallets(pool, &FIVE).await?,
    })
}

impl IdempotencyReport {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không khớp.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.fresh_count == 1, "đúng một lệnh tạo lô");
        ensure!(self.same_batch && self.same_items);
        ensure!(self.payouts_and_entries == (4, 4));
        ensure!(!self.duplicate_entry_inserted);
        ensure!(
            self.concurrent_sent == 4,
            "mỗi khoản chỉ được tính SENT một lần"
        );
        ensure!(self.third_send == SendReport::default());
        ensure!(
            self.bank_orders.iter().all(|(_, n)| *n == 1),
            "{:?}",
            self.bank_orders
        );
        ensure!(
            self.wallets
                == pairs(&[
                    ("DR-001", 0),
                    ("DR-002", 40_800),
                    ("DR-003", 94_800),
                    ("DR-004", 0),
                    ("DR-005", 0)
                ])
        );
        Ok(())
    }
}
