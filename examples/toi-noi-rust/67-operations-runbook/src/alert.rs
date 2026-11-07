//! Điều kiện alert, mức độ và người xử lý: hàm thuần trên một ảnh chụp số liệu.
//!
//! Hàm [`evaluate`] chỉ NHẬN số liệu và TRẢ danh sách alert. Nó không có kết nối database nào
//! trong tay nên không thể sửa sổ; hành động duy nhất của một alert là báo người (ticket hoặc page).

use crate::slo::Severity;

/// Ai nhận alert. Phần tử đầu của `owners` là người xử lý chính.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Owner {
    /// Kỹ sư trực service sổ cái.
    LedgerOnCall,
    /// Nhóm vận hành tài chính (đối soát, duyệt bút toán điều chỉnh).
    FinanceOps,
}

/// Thống kê đối soát theo dòng clearing (như bài 47): số mã so được và số mã lệch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReconStats {
    /// Số `external_ref` xuất hiện ở ít nhất một phía.
    pub compared_refs: i64,
    /// Số mã mà hai phía không khớp (thiếu một phía hoặc lệch tiền).
    pub mismatched_refs: i64,
    /// Tổng chênh lệch tuyệt đối, đồng.
    pub abs_diff_dong: i64,
}

impl ReconStats {
    /// Tỷ lệ lệch, phần triệu của số mã so được. `None` khi chưa so mã nào.
    pub fn ratio_ppm(&self) -> Option<i64> {
        (self.compared_refs > 0).then(|| self.mismatched_refs * 1_000_000 / self.compared_refs)
    }
}

/// Ảnh chụp số liệu tại một thời điểm: đầu vào duy nhất của [`evaluate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Snapshot {
    /// Tuổi của dòng outbox chưa gửi cũ nhất, giây; `None` khi outbox sạch.
    pub outbox_oldest_age_secs: Option<i64>,
    /// Số message consumer chưa xử lý.
    pub consumer_lag: i64,
    /// Số message đang nằm trong DLQ.
    pub dlq_depth: i64,
    /// Tổng mọi posting trong sổ; phải bằng 0.
    pub ledger_total_dong: i64,
    /// Số tài khoản có `account_balances` khác tổng posting.
    pub balance_drift_accounts: i64,
    /// Đối soát.
    pub recon: ReconStats,
}

/// Ngưỡng. Mọi con số là gợi ý của bài, không phải của tài liệu kiến trúc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Thresholds {
    /// Outbox tồn quá số giây này: ticket.
    pub outbox_age_ticket_secs: i64,
    /// Outbox tồn quá số giây này: page.
    pub outbox_age_page_secs: i64,
    /// Lag quá số này: ticket.
    pub lag_ticket: i64,
    /// Lag quá số này: page.
    pub lag_page: i64,
    /// Tỷ lệ lệch đối soát từ mức này: page, phần triệu (10.000 = 1%).
    pub recon_page_ppm: i64,
    /// Chênh lệch tuyệt đối từ mức này: page, đồng.
    pub recon_page_abs_dong: i64,
}

impl Thresholds {
    /// Bộ ngưỡng gợi ý ban đầu.
    pub const fn suggested() -> Thresholds {
        Thresholds {
            outbox_age_ticket_secs: 60,
            outbox_age_page_secs: 300,
            lag_ticket: 1_000,
            lag_page: 10_000,
            recon_page_ppm: 10_000,
            recon_page_abs_dong: 1_000_000,
        }
    }
}

/// Một alert đang bắn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alert {
    /// Tên cố định, dùng làm khoá trong runbook.
    pub name: &'static str,
    /// Mức.
    pub severity: Severity,
    /// Người nhận, người chính đứng đầu.
    pub owners: &'static [Owner],
    /// Dòng của bảng runbook.
    pub runbook: &'static str,
    /// Mô tả một dòng kèm số liệu.
    pub summary: String,
}

const LEDGER_THEN_FINANCE: &[Owner] = &[Owner::LedgerOnCall, Owner::FinanceOps];
const FINANCE_THEN_LEDGER: &[Owner] = &[Owner::FinanceOps, Owner::LedgerOnCall];
const LEDGER_ONLY: &[Owner] = &[Owner::LedgerOnCall];

/// Đánh giá mọi điều kiện trên một ảnh chụp.
pub fn evaluate(s: &Snapshot, t: &Thresholds) -> Vec<Alert> {
    let mut alerts = Vec::new();

    if s.ledger_total_dong != 0 {
        alerts.push(Alert {
            name: "ledger-total-nonzero",
            severity: Severity::Page,
            owners: LEDGER_THEN_FINANCE,
            runbook: "R5",
            summary: format!(
                "tổng mọi posting = {} đồng, phải bằng 0",
                s.ledger_total_dong
            ),
        });
    } else if s.balance_drift_accounts > 0 {
        alerts.push(Alert {
            name: "balance-projection-drift",
            severity: Severity::Ticket,
            owners: LEDGER_ONLY,
            runbook: "R6",
            summary: format!(
                "{} tài khoản có số dư projection khác tổng posting",
                s.balance_drift_accounts
            ),
        });
    }

    if let Some(age) = s.outbox_oldest_age_secs {
        let severity = if age > t.outbox_age_page_secs {
            Some(Severity::Page)
        } else if age > t.outbox_age_ticket_secs {
            Some(Severity::Ticket)
        } else {
            None
        };
        if let Some(severity) = severity {
            alerts.push(Alert {
                name: "outbox-stale",
                severity,
                owners: LEDGER_ONLY,
                runbook: "R1",
                summary: format!("dòng outbox chưa gửi cũ nhất {age} giây"),
            });
        }
    }

    let lag_severity = if s.consumer_lag > t.lag_page {
        Some(Severity::Page)
    } else if s.consumer_lag > t.lag_ticket {
        Some(Severity::Ticket)
    } else {
        None
    };
    if let Some(severity) = lag_severity {
        alerts.push(Alert {
            name: "consumer-lag",
            severity,
            owners: LEDGER_ONLY,
            runbook: "R3",
            summary: format!("consumer chậm {} message", s.consumer_lag),
        });
    }

    if s.dlq_depth > 0 {
        alerts.push(Alert {
            name: "dlq-not-empty",
            severity: Severity::Ticket,
            owners: LEDGER_ONLY,
            runbook: "R2",
            summary: format!("{} message trong DLQ", s.dlq_depth),
        });
    }

    if s.recon.mismatched_refs > 0 {
        let ratio = s.recon.ratio_ppm().unwrap_or(0);
        let page = ratio >= t.recon_page_ppm || s.recon.abs_diff_dong >= t.recon_page_abs_dong;
        alerts.push(Alert {
            name: "reconciliation-mismatch",
            severity: if page {
                Severity::Page
            } else {
                Severity::Ticket
            },
            owners: FINANCE_THEN_LEDGER,
            runbook: "R4",
            summary: format!(
                "{} trên {} mã lệch ({} phần triệu), chênh {} đồng",
                s.recon.mismatched_refs, s.recon.compared_refs, ratio, s.recon.abs_diff_dong
            ),
        });
    }
    alerts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(alerts: &[Alert]) -> Vec<&'static str> {
        alerts.iter().map(|a| a.name).collect()
    }

    #[test]
    fn healthy_snapshot_raises_nothing() {
        assert!(evaluate(&Snapshot::default(), &Thresholds::suggested()).is_empty());
    }

    #[test]
    fn one_mismatch_in_400_is_a_ticket_for_finance() {
        let s = Snapshot {
            recon: ReconStats {
                compared_refs: 400,
                mismatched_refs: 1,
                abs_diff_dong: 25_500,
            },
            ..Snapshot::default()
        };
        let alerts = evaluate(&s, &Thresholds::suggested());
        assert_eq!(names(&alerts), ["reconciliation-mismatch"]);
        assert_eq!(alerts[0].severity, Severity::Ticket);
        assert_eq!(alerts[0].owners[0], Owner::FinanceOps);
        assert_eq!(s.recon.ratio_ppm(), Some(2_500)); // 0,25%
    }

    #[test]
    fn mismatch_ratio_at_one_percent_or_big_money_pages() {
        let t = Thresholds::suggested();
        let by_ratio = ReconStats {
            compared_refs: 400,
            mismatched_refs: 4,
            abs_diff_dong: 102_000,
        };
        let by_money = ReconStats {
            compared_refs: 100_000,
            mismatched_refs: 1,
            abs_diff_dong: 1_000_000,
        };
        for recon in [by_ratio, by_money] {
            let alerts = evaluate(
                &Snapshot {
                    recon,
                    ..Snapshot::default()
                },
                &t,
            );
            assert_eq!(alerts[0].severity, Severity::Page, "{recon:?}");
        }
    }

    #[test]
    fn outbox_age_has_two_levels() {
        let t = Thresholds::suggested();
        let at = |age| {
            evaluate(
                &Snapshot {
                    outbox_oldest_age_secs: Some(age),
                    ..Snapshot::default()
                },
                &t,
            )
        };
        assert!(at(60).is_empty());
        assert_eq!(at(61)[0].severity, Severity::Ticket);
        assert_eq!(at(301)[0].severity, Severity::Page);
    }

    #[test]
    fn nonzero_total_pages_both_teams_and_hides_the_lesser_drift_alert() {
        let alerts = evaluate(
            &Snapshot {
                ledger_total_dong: 100,
                balance_drift_accounts: 1,
                ..Snapshot::default()
            },
            &Thresholds::suggested(),
        );
        assert_eq!(names(&alerts), ["ledger-total-nonzero"]);
        assert_eq!(alerts[0].owners, [Owner::LedgerOnCall, Owner::FinanceOps]);
    }

    #[test]
    fn drift_alone_is_a_ticket_and_dlq_always_is() {
        let alerts = evaluate(
            &Snapshot {
                balance_drift_accounts: 2,
                dlq_depth: 3,
                ..Snapshot::default()
            },
            &Thresholds::suggested(),
        );
        assert_eq!(
            names(&alerts),
            ["balance-projection-drift", "dlq-not-empty"]
        );
        assert!(alerts.iter().all(|a| a.severity == Severity::Ticket));
    }
}
