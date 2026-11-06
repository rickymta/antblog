//! Các kịch bản của bài: chuyến TN-0002 có mã giảm 5.000 do nền tảng chịu, bảng "ai chịu",
//! cuộc đua ngân sách, lỗi hết ngân sách, hoàn chuyến và trợ giá tài xế TN-0003.

use std::collections::BTreeMap;

use anyhow::{Result, ensure};
use ledger_core::Money;

use crate::db::TestDb;
use crate::promo::{CLEARING, Funding, PROMO_EXPENSE, PROMO_WALLET, PromoGrant, REVENUE, Shares};
use crate::race::{RaceMode, RaceResult, race};
use crate::store::{BudgetGuard, PgLedger, PostError, Posted, budget_account};

/// Một bút toán đã ghi, để in.
#[derive(Debug, Clone)]
pub struct EntryView {
    /// Mã `JE-000001`.
    pub id: String,
    /// `idempotency_key`.
    pub key: String,
    /// Posting `(tài khoản, số tiền có dấu)`.
    pub lines: Vec<(String, i64)>,
}

/// Số dư thay đổi của bảy tài khoản, theo thứ tự cố định:
/// khách, clearing, ngân sách, chi phí khuyến mãi, ví khuyến mãi, tài xế, doanh thu.
pub type Deltas = [i64; 7];

/// Một chuyến chạy trọn: redeem, capture, chia tiền.
#[derive(Debug, Clone)]
pub struct TripRun {
    /// Các bút toán đã ghi theo thứ tự.
    pub entries: Vec<EntryView>,
    /// Thay đổi số dư của bảy tài khoản.
    pub deltas: Deltas,
    /// Ngân sách còn lại của campaign.
    pub budget_after: i64,
}

/// Một hàng của bảng "ai chịu giảm giá".
#[derive(Debug, Clone)]
pub struct FunderRow {
    /// Nhãn hàng.
    pub label: &'static str,
    /// Thay đổi số dư.
    pub deltas: Deltas,
    /// Số posting của bút toán redeem (0 nếu không có) và của bút toán chia tiền.
    pub postings: (usize, usize),
}

/// Lỗi hết ngân sách nhìn từ phía Promotion service.
#[derive(Debug, Clone)]
pub struct ExhaustedView {
    /// Thông điệp lỗi.
    pub message: String,
    /// Mã gRPC.
    pub grpc: &'static str,
    /// Mã nghiệp vụ.
    pub domain: &'static str,
    /// Có nên gửi lại không.
    pub retryable: bool,
    /// Gửi lại yêu cầu đã thắng từ trước (cùng key) có còn trả `Ok` không.
    pub replay_ok: bool,
}

/// Hoàn chuyến TN-0002.
#[derive(Debug, Clone)]
pub struct RefundView {
    /// Các bút toán đảo: `(id, key)`.
    pub reversals: Vec<(String, String)>,
    /// Gọi hoàn chuyến lần hai: có toàn replay không.
    pub second_call_all_replays: bool,
    /// Thay đổi của chuyến cộng thay đổi của hoàn chuyến (mong đợi toàn 0).
    pub net_deltas: Deltas,
    /// Ngân sách sau khi hoàn.
    pub budget_after: i64,
}

/// Trợ giá tài xế TN-0003.
#[derive(Debug)]
pub struct SubsidyView {
    /// Bút toán trợ giá (4 posting).
    pub entry: EntryView,
    /// Thay đổi số dư của cả chuyến.
    pub deltas: Deltas,
    /// Lỗi của lần trợ giá thứ hai khi ngân sách không đủ.
    pub second: PostError,
}

/// Mọi thứ bài viết in ra.
pub struct Report {
    /// TN-0002, nền tảng chịu.
    pub tn0002: TripRun,
    /// Đọc số dư theo loại sau TN-0002: `(bên nợ, bên có)`.
    pub trial: (i64, i64),
    /// Bảng ai chịu.
    pub funders: Vec<FunderRow>,
    /// Cuộc đua có khóa.
    pub locked: RaceResult,
    /// Cuộc đua không khóa.
    pub naive: RaceResult,
    /// Cuộc đua không khóa nhưng có trigger chặn số dư âm.
    pub floor: RaceResult,
    /// Hết ngân sách nhìn từ Promotion.
    pub exhausted: ExhaustedView,
    /// Hoàn chuyến.
    pub refund: RefundView,
    /// Trợ giá tài xế.
    pub subsidy: SubsidyView,
    /// Lỗi khi tiền thu không khớp cước trừ giảm.
    pub mismatch: PostError,
}

fn dong(n: i64) -> Money {
    Money::from_dong(n)
}

async fn view(ledger: &PgLedger, posted: &Posted, key: &str) -> Result<EntryView> {
    let lines = ledger.entry_by_key(key).await?.unwrap_or_default();
    Ok(EntryView {
        id: posted.display_id(),
        key: key.to_string(),
        lines,
    })
}

fn deltas_of(
    before: &BTreeMap<String, i64>,
    after: &BTreeMap<String, i64>,
    names: [String; 7],
) -> Deltas {
    let get = |m: &BTreeMap<String, i64>, k: &str| m.get(k).copied().unwrap_or(0);
    let mut out = [0; 7];
    for (slot, name) in out.iter_mut().zip(names.iter()) {
        *slot = get(after, name) - get(before, name);
    }
    out
}

fn seven_names(customer: &str, driver: &str, campaign: &str) -> [String; 7] {
    [
        format!("customer:{customer}"),
        CLEARING.to_string(),
        budget_account(campaign),
        PROMO_EXPENSE.to_string(),
        PROMO_WALLET.to_string(),
        format!("driver:{driver}"),
        REVENUE.to_string(),
    ]
}

/// Số posting của bút toán redeem (0 nếu không có) và của bút toán chia tiền.
fn entry_shape(run: &TripRun) -> (usize, usize) {
    let split = run.entries.last().map_or(0, |e| e.lines.len());
    let redeem = if run.entries.len() == 3 {
        run.entries[0].lines.len()
    } else {
        0
    };
    (redeem, split)
}

/// Chạy một chuyến theo đúng thứ tự hợp đồng: Promotion redeem, Payment capture, Trip chia tiền.
#[allow(clippy::too_many_arguments)]
pub async fn run_trip(
    ledger: &PgLedger,
    trip: &str,
    customer: &str,
    driver: &str,
    fare: Money,
    campaign: &str,
    discount: Money,
    funding: Funding,
) -> Result<TripRun> {
    ledger
        .open_trip_accounts(customer, driver, Some(campaign))
        .await?;
    let before = ledger.balances().await?;
    let shares = Shares::from_funding(discount, funding)?;
    let grant = PromoGrant {
        trip: trip.to_string(),
        campaign: campaign.to_string(),
        shares,
    };
    let mut entries = Vec::new();
    // 1. Promotion gọi Ledger trước khi khách bị trừ tiền: hết ngân sách thì biết sớm.
    if let Some(p) = ledger.redeem(&grant, &BudgetGuard::Locked).await? {
        entries.push(view(ledger, &p, &format!("trip:{trip}:promo:{campaign}")).await?);
    }
    // 2. Payment: VíXanh thu đúng cước trừ giảm.
    let charged = fare
        .checked_sub(discount)
        .ok_or_else(|| anyhow::anyhow!("tràn số"))?;
    let cap = ledger.capture(trip, customer, charged).await?;
    entries.push(view(ledger, &cap, &format!("trip:{trip}:capture")).await?);
    // 3. Trip hoàn tất: chia tiền, phí tính trên cước gốc.
    let split = ledger.settle(trip, driver, fare, shares).await?;
    entries.push(view(ledger, &split, &format!("trip:{trip}:split")).await?);

    let after = ledger.balances().await?;
    let names = seven_names(customer, driver, campaign);
    Ok(TripRun {
        deltas: deltas_of(&before, &after, names),
        budget_after: after.get(&budget_account(campaign)).copied().unwrap_or(0),
        entries,
    })
}

/// Chạy mọi kịch bản trên một `TestDb` đã dựng (pool đủ lớn cho cuộc đua).
pub async fn run_all(db: &TestDb) -> Result<Report> {
    let ledger = db.ledger();
    let fare = dong(25_500);

    // --- TN-0002: mã giảm 5.000, nền tảng chịu ---
    ledger.fund_campaign("WELCOME5K", dong(10_000)).await?;
    // Chụp số dư sau khi cấp ngân sách để chỉ đo tác động của chuyến.
    let tn0002 = run_trip(
        &ledger,
        "TN-0002",
        "CU-002",
        "DR-002",
        fare,
        "WELCOME5K",
        dong(5_000),
        Funding::Platform,
    )
    .await?;
    let trial = ledger.trial_by_kind().await?;

    // --- Bảng ai chịu: cùng cước, cùng mức giảm 5.000, đổi người chịu ---
    ledger.fund_campaign("SHARED", dong(10_000)).await?;
    let driver_funded = run_trip(
        &ledger,
        "TN-0102",
        "CU-102",
        "DR-102",
        fare,
        "SHARED",
        dong(5_000),
        Funding::Driver,
    )
    .await?;
    let shared = run_trip(
        &ledger,
        "TN-0103",
        "CU-103",
        "DR-103",
        fare,
        "SHARED",
        dong(5_000),
        Funding::Shared {
            platform: dong(2_500),
        },
    )
    .await?;
    let funders = vec![
        FunderRow {
            label: "Nền tảng chịu 5.000",
            deltas: tn0002.deltas,
            postings: entry_shape(&tn0002),
        },
        FunderRow {
            label: "Tài xế chịu 5.000",
            deltas: driver_funded.deltas,
            postings: entry_shape(&driver_funded),
        },
        FunderRow {
            label: "Nền tảng 2.500, tài xế 2.500",
            deltas: shared.deltas,
            postings: entry_shape(&shared),
        },
    ];

    // --- Cuộc đua: 5 yêu cầu x 5.000 đồng vào ngân sách 10.000 ---
    ledger.fund_campaign("RACE-LOCK", dong(10_000)).await?;
    let locked = race(&ledger, "RACE-LOCK", 201, 5, dong(5_000), RaceMode::Locked).await?;
    ledger.fund_campaign("RACE-NAIVE", dong(10_000)).await?;
    let naive = race(&ledger, "RACE-NAIVE", 211, 5, dong(5_000), RaceMode::Naive).await?;

    // --- Hết ngân sách: lỗi trả cho Promotion, và replay yêu cầu đã thắng vẫn Ok ---
    let late = PromoGrant {
        trip: "TN-0206".to_string(),
        campaign: "RACE-LOCK".to_string(),
        shares: Shares::from_funding(dong(5_000), Funding::Platform)?,
    };
    let err = ledger
        .redeem(&late, &BudgetGuard::Locked)
        .await
        .err()
        .ok_or_else(|| anyhow::anyhow!("lẽ ra phải hết ngân sách"))?;
    let winner = locked
        .winners
        .first()
        .ok_or_else(|| anyhow::anyhow!("cuộc đua có khóa không có người thắng"))?;
    let replay = ledger
        .redeem(
            &PromoGrant {
                trip: winner.clone(),
                campaign: "RACE-LOCK".to_string(),
                shares: Shares::from_funding(dong(5_000), Funding::Platform)?,
            },
            &BudgetGuard::Locked,
        )
        .await;
    let exhausted = ExhaustedView {
        message: err.to_string(),
        grpc: err.grpc_code(),
        domain: err.domain_code(),
        retryable: err.retryable(),
        replay_ok: matches!(replay, Ok(Some(ref p)) if p.replayed),
    };

    // --- Tiền thu không khớp cước trừ giảm: Ledger từ chối chia ---
    ledger
        .open_trip_accounts("CU-301", "DR-301", Some("WELCOME5K"))
        .await?;
    ledger.capture("TN-0301", "CU-301", fare).await?; // thu đủ 25.500 dù Promotion nói giảm 5.000
    let mismatch = ledger
        .settle(
            "TN-0301",
            "DR-301",
            fare,
            Shares::from_funding(dong(5_000), Funding::Platform)?,
        )
        .await
        .err()
        .ok_or_else(|| anyhow::anyhow!("lẽ ra phải lệch số"))?;

    // --- Hoàn chuyến TN-0002 ---
    let before_refund = ledger.balances().await?;
    let reversed = ledger.reverse_trip("TN-0002").await?;
    let mut reversals = Vec::new();
    for p in &reversed {
        reversals.push((p.display_id(), p.id));
    }
    let after_refund = ledger.balances().await?;
    let names = seven_names("CU-002", "DR-002", "WELCOME5K");
    // Thay đổi do hoàn chuyến cộng thay đổi do chuyến phải bằng 0: mọi thứ về nguyên trạng.
    let undo = deltas_of(&before_refund, &after_refund, names);
    let mut net_deltas = [0; 7];
    for (i, slot) in net_deltas.iter_mut().enumerate() {
        *slot = tn0002.deltas[i] + undo[i];
    }
    let again = ledger.reverse_trip("TN-0002").await?;
    let mut refund_entries = Vec::new();
    for (display, id) in &reversals {
        let key = key_of(&ledger, *id).await?;
        refund_entries.push((display.clone(), key));
    }
    let refund = RefundView {
        reversals: refund_entries,
        second_call_all_replays: again.iter().all(|p| p.replayed),
        net_deltas,
        budget_after: after_refund
            .get(&budget_account("WELCOME5K"))
            .copied()
            .unwrap_or(0),
    };

    // --- TN-0003: trợ giá tài xế, khách trả đủ 25.500 ---
    ledger.fund_campaign("PEAKDRV", dong(5_000)).await?;
    ledger
        .open_trip_accounts("CU-003", "DR-003", Some("PEAKDRV"))
        .await?;
    let before = ledger.balances().await?;
    ledger.capture("TN-0003", "CU-003", fare).await?;
    ledger
        .settle("TN-0003", "DR-003", fare, Shares::NONE)
        .await?;
    let sub = ledger
        .subsidize(
            "TN-0003",
            "PEAKDRV",
            "DR-003",
            dong(3_000),
            &BudgetGuard::Locked,
        )
        .await?;
    let entry = view(&ledger, &sub, "trip:TN-0003:subsidy:PEAKDRV").await?;
    let after = ledger.balances().await?;
    let deltas = deltas_of(&before, &after, seven_names("CU-003", "DR-003", "PEAKDRV"));
    let second = ledger
        .subsidize(
            "TN-0004",
            "PEAKDRV",
            "DR-003",
            dong(3_000),
            &BudgetGuard::Locked,
        )
        .await
        .err()
        .ok_or_else(|| anyhow::anyhow!("lần trợ giá thứ hai lẽ ra hết ngân sách"))?;
    let subsidy = SubsidyView {
        entry,
        deltas,
        second,
    };

    // --- Hàng rào thứ hai ở DB: cài trigger rồi chạy lại cuộc đua không khóa ---
    ledger.install_budget_floor().await?;
    ledger.fund_campaign("RACE-FLOOR", dong(10_000)).await?;
    let floor = race(&ledger, "RACE-FLOOR", 221, 5, dong(5_000), RaceMode::Naive).await?;

    Ok(Report {
        tn0002,
        trial,
        funders,
        locked,
        naive,
        floor,
        exhausted,
        refund,
        subsidy,
        mismatch,
    })
}

async fn key_of(ledger: &PgLedger, id: i64) -> Result<String> {
    Ok(ledger.key_of(id).await?)
}

fn fmt_lines(e: &EntryView) -> String {
    let mut s = format!("  {} {}\n", e.id, e.key);
    for (account, amount) in &e.lines {
        s.push_str(&format!("    {account:<28} {amount:>8}\n"));
    }
    s
}

fn row(label: &str, d: &Deltas) -> String {
    format!(
        "  {label:<30} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
        d[0], d[1], d[2], d[3], d[4], d[5], d[6]
    )
}

fn race_line(label: &str, r: &RaceResult) -> String {
    format!(
        "  {label:<26} thành công {}, hết ngân sách {}, lỗi DB {} {:?}, ngân sách còn {}, chi phí +{}",
        r.ok,
        r.exhausted,
        r.db_errors.len(),
        r.db_errors,
        r.budget_after,
        r.expense_delta
    )
}

impl Report {
    /// In kết quả theo đúng thứ tự bài viết.
    pub fn print(&self) {
        println!("== TN-0002: cước 25.500, mã giảm 5.000 do nền tảng chịu ==");
        for e in &self.tn0002.entries {
            print!("{}", fmt_lines(e));
        }
        println!(
            "  thay đổi số dư, ngân sách còn {}",
            self.tn0002.budget_after
        );
        println!(
            "  {:<30} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
            "", "khách", "clearing", "ngân sách", "chi phí", "ví KM", "tài xế", "doanh thu"
        );
        println!("{}", row("TN-0002", &self.tn0002.deltas));
        println!(
            "  doanh thu {} - chi phí {} = {}; đọc theo loại: bên nợ {}, bên có {}",
            self.tn0002.deltas[6],
            self.tn0002.deltas[3],
            self.tn0002.deltas[6] - self.tn0002.deltas[3],
            self.trial.0,
            self.trial.1
        );
        println!();
        println!("== Ai chịu giảm giá thì bút toán đổi thế nào ==");
        for f in &self.funders {
            println!(
                "{}  [redeem {} posting, chia tiền {} posting]",
                row(f.label, &f.deltas),
                f.postings.0,
                f.postings.1
            );
        }
        println!();
        println!("== Cuộc đua: 5 yêu cầu x 5.000 vào ngân sách 10.000 ==");
        println!("{}", race_line("có khóa (for update)", &self.locked));
        println!("{}", race_line("không khóa", &self.naive));
        println!("{}", race_line("không khóa + trigger sàn", &self.floor));
        println!();
        println!("== Hết ngân sách, nhìn từ Promotion service ==");
        println!("  {}", self.exhausted.message);
        println!(
            "  gRPC {}, mã {}, gửi lại được: {}; replay yêu cầu đã thắng vẫn Ok: {}",
            self.exhausted.grpc,
            self.exhausted.domain,
            self.exhausted.retryable,
            self.exhausted.replay_ok
        );
        println!("  tiền thu lệch cước trừ giảm: {}", self.mismatch);
        println!();
        println!("== Hoàn chuyến TN-0002 ==");
        for (id, key) in &self.refund.reversals {
            println!("  {id} {key}");
        }
        println!(
            "  gọi lần hai: toàn replay = {}; ngân sách còn {}",
            self.refund.second_call_all_replays, self.refund.budget_after
        );
        println!("{}", row("chuyến + hoàn chuyến", &self.refund.net_deltas));
        println!();
        println!("== TN-0003: trợ giá tài xế 3.000, khách trả đủ 25.500 ==");
        print!("{}", fmt_lines(&self.subsidy.entry));
        println!("{}", row("TN-0003", &self.subsidy.deltas));
        println!(
            "  trợ giá lần hai: {} ({})",
            self.subsidy.second,
            self.subsidy.second.domain_code()
        );
    }

    /// Kiểm mọi con số bài viết nêu. Sai thì trả `Err` (exit code 1).
    pub fn verify(&self) -> Result<()> {
        let t = &self.tn0002;
        ensure!(
            t.deltas == [-20_500, 0, -5_000, 5_000, -5_000, 20_400, 5_100],
            "TN-0002 deltas {:?}",
            t.deltas
        );
        ensure!(t.budget_after == 5_000, "ngân sách sau TN-0002");
        ensure!(t.entries.len() == 3, "TN-0002 có 3 bút toán");
        for e in &t.entries {
            let sum: i64 = e.lines.iter().map(|(_, a)| a).sum();
            ensure!(sum == 0, "bút toán {} lệch {sum}", e.key);
        }
        let counts: Vec<usize> = t.entries.iter().map(|e| e.lines.len()).collect();
        ensure!(counts == [2, 2, 4], "số posting {counts:?}");
        ensure!(
            t.deltas[6] - t.deltas[3] == 100,
            "doanh thu trừ chi phí phải là 100"
        );
        ensure!(
            self.trial == (35_500, 35_500),
            "đọc theo loại {:?}",
            self.trial
        );
        ensure!(self.funders[0].postings == (2, 4), "posting nền tảng chịu");
        ensure!(self.funders[1].postings == (0, 3), "posting tài xế chịu");
        ensure!(self.funders[2].postings == (2, 4), "posting chia đôi");

        ensure!(
            self.funders[1].deltas == [-20_500, 0, 0, 0, 0, 15_400, 5_100],
            "tài xế chịu {:?}",
            self.funders[1].deltas
        );
        ensure!(
            self.funders[2].deltas == [-20_500, 0, -2_500, 2_500, -2_500, 17_900, 5_100],
            "chia đôi {:?}",
            self.funders[2].deltas
        );

        ensure!(
            self.locked.ok == 2 && self.locked.exhausted == 3 && self.locked.db_errors.is_empty(),
            "cuộc đua có khóa"
        );
        ensure!(
            self.locked.budget_after == 0 && self.locked.expense_delta == 10_000,
            "ngân sách có khóa"
        );
        ensure!(
            self.naive.ok == 5 && self.naive.exhausted == 0,
            "cuộc đua không khóa lẽ ra cả 5 cùng qua"
        );
        ensure!(
            self.naive.budget_after == -15_000 && self.naive.expense_delta == 25_000,
            "không khóa phải bị tiêu quá: {}",
            self.naive.budget_after
        );
        ensure!(
            self.floor.ok == 2
                && self.floor.exhausted == 0
                && self.floor.db_errors.len() == 3
                && self.floor.db_errors.iter().all(|c| c == "23514")
                && self.floor.budget_after == 0,
            "trigger sàn: {:?}",
            self.floor
        );

        ensure!(
            self.exhausted.grpc == "FAILED_PRECONDITION"
                && self.exhausted.domain == "PROMO_BUDGET_EXHAUSTED"
                && !self.exhausted.retryable
                && self.exhausted.replay_ok,
            "lỗi hết ngân sách"
        );
        ensure!(
            matches!(
                self.mismatch,
                PostError::AmountMismatch { expected, captured, .. }
                    if expected == dong(20_500) && captured == dong(25_500)
            ),
            "lệch số: {}",
            self.mismatch
        );

        let r = &self.refund;
        ensure!(r.reversals.len() == 3, "hoàn chuyến đảo 3 bút toán");
        ensure!(
            r.net_deltas == [0; 7],
            "hoàn chuyến về nguyên trạng: {:?}",
            r.net_deltas
        );
        ensure!(r.budget_after == 10_000, "ngân sách được trả lại");
        ensure!(r.second_call_all_replays, "hoàn chuyến lần hai là replay");

        let s = &self.subsidy;
        ensure!(s.entry.lines.len() == 4, "trợ giá có 4 posting");
        ensure!(
            s.deltas == [-25_500, 0, -3_000, 3_000, -3_000, 23_400, 5_100],
            "TN-0003 deltas {:?}",
            s.deltas
        );
        ensure!(
            matches!(
                s.second,
                PostError::BudgetExhausted { requested, remaining, .. }
                    if requested == dong(3_000) && remaining == dong(2_000)
            ),
            "trợ giá lần hai: {}",
            s.second
        );
        Ok(())
    }
}
