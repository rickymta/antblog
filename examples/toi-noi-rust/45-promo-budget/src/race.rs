//! Dựng cuộc đua nhiều yêu cầu cùng tiêu một ngân sách. Lịch đua do `Barrier` quyết định,
//! không do may rủi: kết quả kiểm bằng bất biến, không kiểm thứ tự ai thắng.

use std::sync::Arc;

use anyhow::Result;
use ledger_core::Money;
use tokio::sync::Barrier;
use tokio::task::JoinSet;

use crate::promo::{Funding, PROMO_EXPENSE, PromoGrant, Shares};
use crate::store::{BudgetGuard, PgLedger, PostError, budget_account};

/// Chế độ của một cuộc đua.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaceMode {
    /// `select ... for update`.
    Locked,
    /// `select` thường, mọi bên đọc xong mới cùng ghi (lịch đua tệ nhất, dựng bằng `Barrier`).
    Naive,
}

/// Điều quan sát được sau cuộc đua.
#[derive(Debug, Clone)]
pub struct RaceResult {
    /// Số yêu cầu được ghi.
    pub ok: usize,
    /// Số yêu cầu nhận `BudgetExhausted`.
    pub exhausted: usize,
    /// Số yêu cầu lỗi database (hàng rào ở DB chặn), kèm SQLSTATE của từng cái.
    pub db_errors: Vec<String>,
    /// Mã chuyến của các yêu cầu thành công.
    pub winners: Vec<String>,
    /// Số dư ngân sách sau cuộc đua; âm nghĩa là đã tiêu quá.
    pub budget_after: i64,
    /// Chi phí khuyến mãi tăng thêm trong cuộc đua.
    pub expense_delta: i64,
}

/// Cho `n` chuyến `TN-<first..first+n>` cùng xin `amount` đồng từ `campaign` (đã cấp sẵn).
pub async fn race(
    ledger: &PgLedger,
    campaign: &str,
    first_trip: u32,
    n: usize,
    amount: Money,
    mode: RaceMode,
) -> Result<RaceResult> {
    let before = ledger.balances().await?;
    let start = Arc::new(Barrier::new(n)); // mọi tác vụ bắt đầu cùng lúc
    let guard = match mode {
        RaceMode::Locked => BudgetGuard::Locked,
        RaceMode::Naive => BudgetGuard::Unlocked {
            after_read: Some(Arc::new(Barrier::new(n))),
        },
    };

    let mut set = JoinSet::new();
    for i in 0..n {
        let ledger = ledger.clone();
        let start = Arc::clone(&start);
        let guard = guard.clone();
        let trip = format!("TN-{:04}", first_trip + u32::try_from(i)?);
        let grant = PromoGrant {
            trip: trip.clone(),
            campaign: campaign.to_string(),
            shares: Shares::from_funding(amount, Funding::Platform)?,
        };
        set.spawn(async move {
            start.wait().await;
            (trip, ledger.redeem(&grant, &guard).await)
        });
    }

    let mut result = RaceResult {
        ok: 0,
        exhausted: 0,
        db_errors: Vec::new(),
        winners: Vec::new(),
        budget_after: 0,
        expense_delta: 0,
    };
    while let Some(joined) = set.join_next().await {
        let (trip, outcome) = joined?;
        match outcome {
            Ok(_) => {
                result.ok += 1;
                result.winners.push(trip);
            }
            Err(PostError::BudgetExhausted { .. }) => result.exhausted += 1,
            Err(e) => result
                .db_errors
                .push(e.sqlstate().unwrap_or_else(|| e.to_string())),
        }
    }
    result.winners.sort();
    let after = ledger.balances().await?;
    let get = |m: &std::collections::BTreeMap<String, i64>, k: &str| m.get(k).copied().unwrap_or(0);
    let budget = budget_account(campaign);
    result.budget_after = get(&after, &budget);
    result.expense_delta = get(&after, PROMO_EXPENSE) - get(&before, PROMO_EXPENSE);
    Ok(result)
}
