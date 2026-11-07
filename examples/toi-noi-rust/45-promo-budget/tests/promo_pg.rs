//! Integration test trên PostgreSQL thật. Mỗi ca một schema `lab_<uuid>`; ca trả `Result`
//! thay vì panic, nên `with_db` luôn kịp dọn schema. Không ca nào assert thời gian:
//! cuộc đua dựng bằng `Barrier` và chỉ kiểm bất biến.

use std::collections::BTreeMap;

use anyhow::{Result, bail, ensure};
use ledger_core::{LedgerError, Money};
use ledger_promo::PostError;
use ledger_promo::db::{TestDb, lab_url, with_db};
use ledger_promo::promo::{
    CLEARING, Funding, PROMO_EXPENSE, PROMO_WALLET, PromoGrant, REVENUE, Shares, TREASURY,
};
use ledger_promo::race::{RaceMode, race};
use ledger_promo::scenarios::run_trip;
use ledger_promo::store::{BudgetGuard, budget_account};

macro_rules! eq {
    ($left:expr, $right:expr) => {{
        let (l, r) = (&$left, &$right);
        ensure!(l == r, "{} = {:?}, mong đợi {:?}", stringify!($left), l, r);
    }};
}

fn dong(n: i64) -> Money {
    Money::from_dong(n)
}

fn grant(trip: &str, campaign: &str, platform: i64) -> Result<PromoGrant> {
    Ok(PromoGrant {
        trip: trip.to_string(),
        campaign: campaign.to_string(),
        shares: Shares::from_funding(dong(platform), Funding::Platform)?,
    })
}

fn get(m: &BTreeMap<String, i64>, k: &str) -> i64 {
    m.get(k).copied().unwrap_or(0)
}

#[tokio::test]
async fn tn0002_platform_funded_balances_and_entry_shapes() -> Result<()> {
    with_db(3, async |db| {
        let ledger = db.ledger();
        ledger.fund_campaign("WELCOME5K", dong(10_000)).await?;
        let run = run_trip(
            &ledger,
            "TN-0002",
            "CU-002",
            "DR-002",
            dong(25_500),
            "WELCOME5K",
            dong(5_000),
            Funding::Platform,
        )
        .await?;
        // khách, clearing, ngân sách, chi phí, ví khuyến mãi, tài xế, doanh thu
        eq!(
            run.deltas,
            [-20_500, 0, -5_000, 5_000, -5_000, 20_400, 5_100]
        );
        eq!(run.budget_after, 5_000);
        let shapes: Vec<usize> = run.entries.iter().map(|e| e.lines.len()).collect();
        eq!(shapes, vec![2, 2, 4]);
        // Phí tính trên cước gốc: 20% của 25.500 là 5.100, không phải 20% của 20.500.
        eq!(run.deltas[6], 5_100);
        eq!(run.deltas[5], 25_500 - 5_100);
        // Lãi của nền tảng trên chuyến: doanh thu trừ chi phí.
        eq!(run.deltas[6] - run.deltas[3], 100);
        let b = ledger.balances().await?;
        // Cấp ngân sách không đụng doanh thu: đã cấp = đã tiêu + còn lại.
        eq!(get(&b, REVENUE), 5_100);
        eq!(
            get(&b, TREASURY) + get(&b, &budget_account("WELCOME5K")) + get(&b, PROMO_EXPENSE),
            0
        );
        // Sau chuyến, ví khuyến mãi đã trả thay đúng bằng chi phí.
        eq!(get(&b, PROMO_WALLET) + get(&b, PROMO_EXPENSE), 0);
        // Đọc số dư theo loại như bài 41: hai phía bằng nhau khi clearing về 0.
        eq!(ledger.trial_by_kind().await?, (35_500, 35_500));
        Ok(())
    })
    .await
}

#[tokio::test]
async fn every_entry_in_the_database_sums_to_zero() -> Result<()> {
    with_db(3, async |db| {
        let ledger = db.ledger();
        ledger.fund_campaign("SHARED", dong(10_000)).await?;
        run_trip(
            &ledger,
            "TN-0103",
            "CU-103",
            "DR-103",
            dong(25_500),
            "SHARED",
            dong(5_000),
            Funding::Shared {
                platform: dong(2_500),
            },
        )
        .await?;
        ledger
            .subsidize(
                "TN-0103",
                "SHARED",
                "DR-103",
                dong(1_000),
                &BudgetGuard::Locked,
            )
            .await?;
        let unbalanced: i64 = sqlx::query_scalar(
            "select count(*) from (select entry_id from postings group by entry_id \
             having sum(amount_dong) <> 0) t",
        )
        .fetch_one(&db.pool)
        .await?;
        eq!(unbalanced, 0);
        // Tổng mọi số dư cũng bằng 0: tiền không tự sinh ra hay biến mất.
        let all: i64 = sqlx::query_scalar(
            "select coalesce(sum(balance_dong), 0)::bigint from account_balances",
        )
        .fetch_one(&db.pool)
        .await?;
        eq!(all, 0);
        Ok(())
    })
    .await
}

#[tokio::test]
async fn locked_race_never_overspends() -> Result<()> {
    with_db(12, async |db| {
        let ledger = db.ledger();
        ledger.ensure_account(PROMO_EXPENSE, "expense").await?;
        // 8 yêu cầu x 3.000 vào ngân sách 10.000: chỉ 3 yêu cầu (9.000) vừa.
        ledger.fund_campaign("RACE", dong(10_000)).await?;
        let r = race(&ledger, "RACE", 201, 8, dong(3_000), RaceMode::Locked).await?;
        eq!(r.ok, 3);
        eq!(r.exhausted, 5);
        ensure!(r.db_errors.is_empty(), "lỗi DB: {:?}", r.db_errors);
        eq!(r.budget_after, 1_000);
        // Bất biến: đã tiêu + còn lại = ngân sách, và không bao giờ âm.
        eq!(r.expense_delta + r.budget_after, 10_000);
        ensure!(r.budget_after >= 0, "ngân sách âm");
        Ok(())
    })
    .await
}

#[tokio::test]
async fn unlocked_race_overspends_when_everyone_reads_before_anyone_writes() -> Result<()> {
    with_db(8, async |db| {
        let ledger = db.ledger();
        ledger.ensure_account(PROMO_EXPENSE, "expense").await?;
        ledger.fund_campaign("RACE", dong(10_000)).await?;
        let r = race(&ledger, "RACE", 211, 5, dong(5_000), RaceMode::Naive).await?;
        // Cả 5 cùng đọc 10.000 nên cả 5 cùng thấy "đủ".
        eq!(r.ok, 5);
        eq!(r.exhausted, 0);
        eq!(r.budget_after, -15_000);
        eq!(r.expense_delta, 25_000);
        Ok(())
    })
    .await
}

#[tokio::test]
async fn unlocked_race_with_eight_waiters_spends_twenty_four_thousand() -> Result<()> {
    with_db(12, async |db| {
        let ledger = db.ledger();
        ledger.ensure_account(PROMO_EXPENSE, "expense").await?;
        ledger.fund_campaign("RACE", dong(10_000)).await?;
        // Cùng tải với ca có khóa (8 x 3.000 vào 10.000), nhưng không khóa: cả 8 cùng qua.
        let r = race(&ledger, "RACE", 231, 8, dong(3_000), RaceMode::Naive).await?;
        eq!(r.ok, 8);
        eq!(r.budget_after, 10_000 - 24_000);
        eq!(r.expense_delta, 24_000);
        Ok(())
    })
    .await
}

#[tokio::test]
async fn budget_floor_trigger_turns_overspend_into_an_opaque_db_error() -> Result<()> {
    with_db(8, async |db| {
        let ledger = db.ledger();
        ledger.ensure_account(PROMO_EXPENSE, "expense").await?;
        ledger.install_budget_floor().await?;
        ledger.fund_campaign("RACE", dong(10_000)).await?;
        let r = race(&ledger, "RACE", 221, 5, dong(5_000), RaceMode::Naive).await?;
        eq!(r.ok, 2);
        eq!(r.exhausted, 0); // không ai nhận BudgetExhausted: lỗi đến từ trigger, không từ code kiểm
        eq!(r.db_errors, vec!["23514".to_string(); 3]);
        eq!(r.budget_after, 0);
        Ok(())
    })
    .await
}

#[tokio::test]
async fn exhausted_budget_is_a_domain_error_and_leaves_no_trace() -> Result<()> {
    with_db(3, async |db| {
        let ledger = db.ledger();
        ledger.ensure_account(PROMO_EXPENSE, "expense").await?;
        ledger.fund_campaign("SMALL", dong(3_000)).await?;
        let before = ledger.balances().await?;
        let Err(err) = ledger
            .redeem(&grant("TN-0401", "SMALL", 5_000)?, &BudgetGuard::Locked)
            .await
        else {
            bail!("lẽ ra hết ngân sách");
        };
        ensure!(
            matches!(
                err,
                PostError::BudgetExhausted { requested, remaining, .. }
                    if requested == dong(5_000) && remaining == dong(3_000)
            ),
            "{err}"
        );
        eq!(err.grpc_code(), "FAILED_PRECONDITION");
        eq!(err.domain_code(), "PROMO_BUDGET_EXHAUSTED");
        ensure!(!err.retryable(), "hết ngân sách không được gợi ý gửi lại");
        // Transaction bị rollback: không còn header, không đổi số dư.
        eq!(ledger.entry_by_key("trip:TN-0401:promo:SMALL").await?, None);
        eq!(ledger.balances().await?, before);
        Ok(())
    })
    .await
}

#[tokio::test]
async fn replay_does_not_spend_twice_and_changed_content_conflicts() -> Result<()> {
    with_db(3, async |db| {
        let ledger = db.ledger();
        ledger.ensure_account(PROMO_EXPENSE, "expense").await?;
        ledger.fund_campaign("ONE", dong(5_000)).await?;
        let g = grant("TN-0402", "ONE", 5_000)?;
        let first = ledger.redeem(&g, &BudgetGuard::Locked).await?;
        eq!(first.as_ref().map(|p| p.replayed), Some(false));
        // Ngân sách đã cạn, nhưng gửi lại cùng yêu cầu vẫn là replay thành công.
        let again = ledger.redeem(&g, &BudgetGuard::Locked).await?;
        eq!(again.as_ref().map(|p| p.replayed), Some(true));
        eq!(ledger.balance(&budget_account("ONE")).await?, Some(dong(0)));
        // Cùng key nhưng mức tiền khác: xung đột, không phải replay.
        let changed = grant("TN-0402", "ONE", 4_000)?;
        let Err(err) = ledger.redeem(&changed, &BudgetGuard::Locked).await else {
            bail!("lẽ ra xung đột idempotency");
        };
        ensure!(
            matches!(
                err,
                PostError::Ledger(LedgerError::IdempotencyConflict { .. })
            ),
            "{err}"
        );
        eq!(err.grpc_code(), "ALREADY_EXISTS");
        Ok(())
    })
    .await
}

#[tokio::test]
async fn reversing_a_trip_returns_the_budget_and_reverses_the_expense() -> Result<()> {
    with_db(3, async |db| {
        let ledger = db.ledger();
        ledger.fund_campaign("WELCOME5K", dong(10_000)).await?;
        let before = ledger.balances().await?;
        let run = run_trip(
            &ledger,
            "TN-0002",
            "CU-002",
            "DR-002",
            dong(25_500),
            "WELCOME5K",
            dong(5_000),
            Funding::Platform,
        )
        .await?;
        eq!(run.budget_after, 5_000);
        let reversed = ledger.reverse_trip("TN-0002").await?;
        eq!(reversed.len(), 3);
        let after = ledger.balances().await?;
        eq!(get(&after, &budget_account("WELCOME5K")), 10_000);
        eq!(get(&after, PROMO_EXPENSE), 0);
        eq!(get(&after, REVENUE), get(&before, REVENUE));
        eq!(get(&after, PROMO_WALLET), 0);
        eq!(get(&after, CLEARING), 0);
        eq!(get(&after, "customer:CU-002"), 0);
        eq!(get(&after, "driver:DR-002"), 0);
        // Bút toán đảo trỏ về bút toán gốc.
        let linked: i64 = sqlx::query_scalar(
            "select count(*) from journal_entries where reversal_of is not null",
        )
        .fetch_one(&db.pool)
        .await?;
        eq!(linked, 3);
        // Hoàn lần hai là replay, không đảo thêm.
        let again = ledger.reverse_trip("TN-0002").await?;
        ensure!(again.iter().all(|p| p.replayed), "lần hai phải là replay");
        Ok(())
    })
    .await
}

#[tokio::test]
async fn settle_refuses_when_captured_amount_disagrees_with_the_discount() -> Result<()> {
    with_db(3, async |db| {
        let ledger = db.ledger();
        ledger.open_trip_accounts("CU-301", "DR-301", None).await?;
        let shares = Shares::from_funding(dong(5_000), Funding::Platform)?;
        // Chưa capture: không chia được.
        let Err(not_captured) = ledger
            .settle("TN-0301", "DR-301", dong(25_500), shares)
            .await
        else {
            bail!("lẽ ra chưa capture");
        };
        ensure!(
            matches!(not_captured, PostError::NotCaptured { .. }),
            "{not_captured}"
        );
        // Thu đủ 25.500 dù Promotion nói giảm 5.000.
        ledger.capture("TN-0301", "CU-301", dong(25_500)).await?;
        let Err(err) = ledger
            .settle("TN-0301", "DR-301", dong(25_500), shares)
            .await
        else {
            bail!("lẽ ra lệch số");
        };
        ensure!(
            matches!(err, PostError::AmountMismatch { expected, captured, .. }
                if expected == dong(20_500) && captured == dong(25_500)),
            "{err}"
        );
        eq!(ledger.entry_by_key("trip:TN-0301:split").await?, None);
        Ok(())
    })
    .await
}

#[tokio::test]
async fn settle_refuses_when_the_platform_share_was_not_redeemed() -> Result<()> {
    with_db(3, async |db| {
        let ledger = db.ledger();
        ledger.open_trip_accounts("CU-302", "DR-302", None).await?;
        // VíXanh thu đúng 20.500, nhưng chưa có bút toán redeem nào cho phần 5.000.
        ledger.capture("TN-0302", "CU-302", dong(20_500)).await?;
        let shares = Shares::from_funding(dong(5_000), Funding::Platform)?;
        let Err(err) = ledger
            .settle("TN-0302", "DR-302", dong(25_500), shares)
            .await
        else {
            bail!("lẽ ra thiếu redeem");
        };
        eq!(err.domain_code(), "PROMO_NOT_REDEEMED");
        ensure!(!err.retryable(), "thiếu redeem không tự hết khi gửi lại");
        eq!(ledger.entry_by_key("trip:TN-0302:split").await?, None);
        Ok(())
    })
    .await
}

#[tokio::test]
async fn release_before_capture_returns_the_budget() -> Result<()> {
    with_db(3, async |db| {
        let ledger = db.ledger();
        ledger.open_trip_accounts("CU-303", "DR-303", None).await?;
        ledger.fund_campaign("ONE", dong(5_000)).await?;
        ledger
            .redeem(&grant("TN-0303", "ONE", 5_000)?, &BudgetGuard::Locked)
            .await?;
        // Chuyến hủy trước capture: chỉ có bút toán promo để đảo.
        eq!(ledger.reverse_trip("TN-0303").await?.len(), 1);
        eq!(
            ledger.balance(&budget_account("ONE")).await?,
            Some(dong(5_000))
        );
        eq!(ledger.redeemed_amount("TN-0303").await?, dong(0));
        Ok(())
    })
    .await
}

#[tokio::test]
async fn driver_subsidy_tn0003_pays_the_driver_from_the_budget_not_revenue() -> Result<()> {
    with_db(3, async |db| {
        let ledger = db.ledger();
        ledger.fund_campaign("PEAKDRV", dong(5_000)).await?;
        ledger
            .open_trip_accounts("CU-003", "DR-003", Some("PEAKDRV"))
            .await?;
        ledger.capture("TN-0003", "CU-003", dong(25_500)).await?;
        ledger
            .settle("TN-0003", "DR-003", dong(25_500), Shares::NONE)
            .await?;
        ledger
            .subsidize(
                "TN-0003",
                "PEAKDRV",
                "DR-003",
                dong(3_000),
                &BudgetGuard::Locked,
            )
            .await?;
        let b = ledger.balances().await?;
        eq!(get(&b, "customer:CU-003"), -25_500); // khách vẫn trả đủ
        eq!(get(&b, "driver:DR-003"), 23_400); // 20.400 + 3.000
        eq!(get(&b, REVENUE), 5_100); // trợ giá không đụng doanh thu
        eq!(get(&b, PROMO_EXPENSE), 3_000);
        eq!(get(&b, PROMO_WALLET), -3_000);
        eq!(get(&b, TREASURY), -5_000);
        eq!(get(&b, &budget_account("PEAKDRV")), 2_000);
        // Lần thứ hai vượt phần còn lại: hết ngân sách, không phải lỗi hạ tầng.
        let Err(err) = ledger
            .subsidize(
                "TN-0004",
                "PEAKDRV",
                "DR-003",
                dong(3_000),
                &BudgetGuard::Locked,
            )
            .await
        else {
            bail!("lẽ ra hết ngân sách");
        };
        eq!(err.domain_code(), "PROMO_BUDGET_EXHAUSTED");
        Ok(())
    })
    .await
}

#[tokio::test]
async fn schema_is_dropped_even_when_the_body_fails() -> Result<()> {
    let mut seen = String::new();
    let outcome: Result<()> = with_db(2, async |db| {
        seen = db.schema.clone();
        bail!("ca cố tình lỗi");
    })
    .await;
    ensure!(outcome.is_err(), "ca lẽ ra phải lỗi");
    ensure!(seen.starts_with("lab_"), "tên schema {seen}");
    ensure!(
        !TestDb::schema_exists(&lab_url()?, &seen).await?,
        "schema {seen} còn sót"
    );
    Ok(())
}
