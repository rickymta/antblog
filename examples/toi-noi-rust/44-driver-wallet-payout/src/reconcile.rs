//! Đối chiếu khoản chi của một lô với sao kê ngân hàng.

use std::collections::{HashMap, HashSet};

use anyhow::Result;
use ledger_core::Money;
use sqlx::{PgPool, Row};

use crate::bank::{Bank, LineStatus, StatementLine};
use crate::clock::Clock;
use crate::payout::{Status, advance, fail_payout, load_payouts, refresh_batch_status};

/// Kết quả đối chiếu, mỗi mục là danh sách `payout_key`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    /// Khớp mã và số tiền, sao kê báo đã tới: khoản chuyển sang `SETTLED`.
    pub settled: Vec<String>,
    /// Trong số đó, khoản ta còn `CREATED` (timeout) mà ngân hàng đã nhận: sao kê gỡ nút thắt.
    pub recovered_after_timeout: Vec<String>,
    /// Ngân hàng trả tiền lại: khoản chuyển `FAILED` và có bút toán đảo.
    pub returned: Vec<String>,
    /// Khớp mã nhưng lệch số tiền (ta, ngân hàng): không tự sửa, chờ người xem.
    pub amount_mismatch: Vec<(String, i64, i64)>,
    /// Ta đã gửi (`SENT`) nhưng sao kê không có dòng nào.
    pub missing_in_statement: Vec<String>,
    /// Sao kê có dòng mà không khoản chi nào của ta mang mã đó.
    pub unknown_in_statement: Vec<String>,
}

/// Đối chiếu các khoản `CREATED` và `SENT` của `batch_id` với sao kê của `bank`.
/// Chạy lại được: khoản đã kết thúc không bị đọc lại, nên lần hai không đổi gì thêm.
///
/// # Errors
/// Lỗi SQL hoặc lỗi lấy sao kê.
pub async fn reconcile<B: Bank>(
    pool: &PgPool,
    bank: &B,
    clock: &impl Clock,
    batch_id: i64,
) -> Result<Report> {
    let lines = bank.statement().await?;
    let mut by_ref: HashMap<&str, &StatementLine> = lines
        .iter()
        .map(|line| (line.reference.as_str(), line))
        .collect();
    let known: HashSet<String> = sqlx::query("SELECT payout_key FROM payouts")
        .fetch_all(pool)
        .await?
        .iter()
        .map(|r| r.get("payout_key"))
        .collect();

    let mut report = Report::default();
    for payout in load_payouts(pool, batch_id).await? {
        if !matches!(payout.status, Status::Created | Status::Sent) {
            continue;
        }
        let key = payout.payout_key.clone();
        let Some(line) = by_ref.remove(key.as_str()) else {
            // Khoản CREATED mà sao kê không có: chưa kết luận được, lượt gửi sau sẽ tra cứu; không báo lệch.
            if payout.status == Status::Sent {
                report.missing_in_statement.push(key);
            }
            continue;
        };
        if line.amount != Money::from_dong(payout.amount_dong) {
            report
                .amount_mismatch
                .push((key, payout.amount_dong, line.amount.dong()));
            continue;
        }
        let now = clock.now();
        match line.status {
            LineStatus::Settled => {
                if payout.status == Status::Created {
                    advance(
                        pool,
                        now,
                        payout.id,
                        Status::Created,
                        Status::Sent,
                        Some(&line.bank_ref),
                    )
                    .await?;
                    report.recovered_after_timeout.push(key.clone());
                }
                advance(pool, now, payout.id, Status::Sent, Status::Settled, None).await?;
                report.settled.push(key);
            }
            LineStatus::Returned => {
                if fail_payout(pool, now, payout.id, "ngân hàng trả tiền lại").await? {
                    report.returned.push(key);
                }
            }
        }
    }
    report.unknown_in_statement = by_ref
        .keys()
        .filter(|reference| !known.contains(**reference))
        .map(|reference| (*reference).to_string())
        .collect();
    report.unknown_in_statement.sort();

    let mut tx = pool.begin().await?;
    refresh_batch_status(&mut tx, batch_id).await?;
    tx.commit().await?;
    Ok(report)
}
