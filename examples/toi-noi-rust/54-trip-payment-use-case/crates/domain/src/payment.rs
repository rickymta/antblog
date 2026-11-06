//! Aggregate `TripPayment`: gom ba loại dữ kiện của một chuyến (chuyến xong, VíXanh đã thu, khuyến
//! mãi) và chỉ chốt sổ khi đủ và khớp nhau. Dữ kiện đến thứ tự nào kết quả cũng như nhau, vì
//! quyết định «đã đủ chưa» chỉ nhìn trạng thái, không nhìn thứ tự đến.

use std::fmt;

use ledger_core::{FeePolicy, JournalEntry, LedgerError, Money, default_policy};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::events::{PaymentCaptured, TripCompleted};
use crate::flows;

/// Dữ kiện từ `TripCompleted`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripFacts {
    /// Mã khách.
    pub customer_id: String,
    /// Mã tài xế.
    pub driver_id: String,
    /// Cước, đồng.
    pub fare_dong: i64,
    /// Thời điểm hoàn thành.
    pub occurred_at: String,
}

/// Dữ kiện từ `PaymentCaptured`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureFacts {
    /// Mã thanh toán phía Payment.
    pub payment_id: String,
    /// Số tiền VíXanh đã thu thật.
    pub amount_dong: i64,
    /// Mã giao dịch của VíXanh.
    pub external_ref: String,
    /// Thời điểm thu.
    pub occurred_at: String,
}

/// Quyết định của Promotion cho một chuyến: ai chịu bao nhiêu (bài 45).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromoGrant {
    /// Mã campaign, ví dụ `WELCOME5K`.
    pub campaign: String,
    /// Phần nền tảng chịu, đồng.
    pub platform_dong: i64,
    /// Phần tài xế chịu, đồng.
    pub driver_dong: i64,
}

/// Lý do aggregate từ chối. Mọi lỗi ở đây là lỗi nghiệp vụ: thử lại bao nhiêu lần cũng vẫn sai.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    /// Dữ kiện cùng loại đến lần hai với nội dung khác bản đã nhận.
    #[error("{fact} của {trip} đến lần hai với nội dung khác bản đã nhận")]
    Conflict {
        /// Mã chuyến.
        trip: String,
        /// Loại dữ kiện.
        fact: &'static str,
    },
    /// Dữ liệu sai nội dung.
    #[error("dữ liệu không hợp lệ: {0}")]
    Invalid(String),
    /// Khuyến mãi đến sau khi chuyến đã chốt sổ.
    #[error("khuyến mãi của {trip} đến sau khi chuyến đã chốt sổ")]
    PromoTooLate {
        /// Mã chuyến.
        trip: String,
    },
    /// Hoàn tiền khi chuyến chưa chốt sổ.
    #[error("{trip} chưa chốt sổ nên chưa hoàn được")]
    NotSettled {
        /// Mã chuyến.
        trip: String,
    },
    /// Lần hoàn nhảy cóc.
    #[error("lần hoàn {got} của {trip} sai thứ tự, đang chờ lần {expected}")]
    RefundOutOfSequence {
        /// Mã chuyến.
        trip: String,
        /// Lần hoàn tiếp theo hợp lệ.
        expected: u32,
        /// Lần hoàn nhận được.
        got: u32,
    },
    /// Tổng hoàn vượt số đã thu.
    #[error("hoàn {asked} vượt phần còn hoàn được {remaining}")]
    RefundExceeds {
        /// Phần còn hoàn được.
        remaining: Money,
        /// Số yêu cầu hoàn.
        asked: Money,
    },
    /// Lab chưa hoàn chuyến có khuyến mãi (cần đảo cả ngân sách, bài 45).
    #[error("{trip} có khuyến mãi, lab chưa hoàn chuyến này")]
    RefundOnPromoTrip {
        /// Mã chuyến.
        trip: String,
    },
    /// Lỗi của `ledger-core`.
    #[error(transparent)]
    Ledger(#[from] LedgerError),
}

/// Yêu cầu có đủ số dư ở một tài khoản trước khi ghi (ngân sách campaign không được âm).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Funds {
    /// Tài khoản cần kiểm.
    pub account: String,
    /// Số dư tối thiểu cần có, đồng.
    pub at_least: i64,
}

/// Bút toán aggregate muốn ghi, kèm hai chỉ dẫn cho adapter lưu trữ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedEntry {
    /// Bút toán đã cân.
    pub entry: JournalEntry,
    /// Mã giao dịch VíXanh gắn với dòng clearing của bút toán (chỉ bút toán capture có).
    pub external_ref: Option<String>,
    /// Kiểm số dư trong cùng transaction với việc ghi.
    pub requires_funds: Option<Funds>,
}

impl PlannedEntry {
    fn plain(entry: JournalEntry) -> PlannedEntry {
        PlannedEntry {
            entry,
            external_ref: None,
            requires_funds: None,
        }
    }
}

/// Sự kiện domain: điều vừa xảy ra trong aggregate. Rỗng nghĩa là lệnh không đổi gì (trùng).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    /// Đã nhận `TripCompleted`.
    TripRecorded,
    /// Đã nhận `PaymentCaptured`.
    CaptureRecorded,
    /// Đã nhận quyết định khuyến mãi; `redeem` là bút toán dùng ngân sách nếu nền tảng có chịu.
    PromoGranted {
        /// Mã campaign.
        campaign: String,
        /// Bút toán dùng ngân sách.
        redeem: Option<PlannedEntry>,
    },
    /// Đủ dữ kiện và khớp: chốt sổ bằng hai bút toán capture và split.
    Settled {
        /// Hai bút toán theo thứ tự ghi.
        entries: Vec<PlannedEntry>,
    },
    /// Một lần hoàn một phần.
    Refunded {
        /// Số thứ tự lần hoàn.
        n: u32,
        /// Bút toán hoàn.
        entry: PlannedEntry,
    },
}

impl DomainEvent {
    /// Tên ngắn để log và báo cáo.
    pub fn name(&self) -> &'static str {
        match self {
            DomainEvent::TripRecorded => "TripRecorded",
            DomainEvent::CaptureRecorded => "CaptureRecorded",
            DomainEvent::PromoGranted { .. } => "PromoGranted",
            DomainEvent::Settled { .. } => "Settled",
            DomainEvent::Refunded { .. } => "Refunded",
        }
    }

    /// Các bút toán sự kiện này yêu cầu ghi, theo thứ tự.
    pub fn entries(&self) -> Vec<PlannedEntry> {
        match self {
            DomainEvent::TripRecorded | DomainEvent::CaptureRecorded => Vec::new(),
            DomainEvent::PromoGranted { redeem, .. } => redeem.iter().cloned().collect(),
            DomainEvent::Settled { entries } => entries.clone(),
            DomainEvent::Refunded { entry, .. } => vec![entry.clone()],
        }
    }
}

/// Điều aggregate còn chờ trước khi chốt sổ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Waiting {
    /// Chưa có `TripCompleted`.
    TripCompleted,
    /// Chưa có `PaymentCaptured`.
    PaymentCaptured,
    /// Có cả hai nhưng số thu không bằng cước trừ khuyến mãi.
    AmountMatch {
        /// Cước.
        fare: i64,
        /// Mức giảm đã biết.
        discount: i64,
        /// Số VíXanh đã thu.
        captured: i64,
    },
}

impl fmt::Display for Waiting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Waiting::TripCompleted => write!(f, "chờ TripCompleted"),
            Waiting::PaymentCaptured => write!(f, "chờ PaymentCaptured"),
            Waiting::AmountMatch {
                fare,
                discount,
                captured,
            } => write!(
                f,
                "số thu {} khác cước {} trừ giảm {}: không ghi gì, chờ dữ kiện sửa",
                Money::from_dong(*captured),
                Money::from_dong(*fare),
                Money::from_dong(*discount)
            ),
        }
    }
}

/// Trạng thái thanh toán của một chuyến phía sổ cái. Đây là aggregate được lưu (cột `state`, kèm `version`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripPayment {
    /// Mã chuyến, cũng là định danh của aggregate.
    pub trip_id: String,
    /// Dữ kiện chuyến, nếu đã nhận.
    pub trip: Option<TripFacts>,
    /// Dữ kiện thu tiền, nếu đã nhận.
    pub capture: Option<CaptureFacts>,
    /// Khuyến mãi, nếu có.
    pub promo: Option<PromoGrant>,
    /// Đã chốt sổ (capture và split đã được yêu cầu ghi).
    pub settled: bool,
    /// Số tiền của từng lần hoàn, theo thứ tự lần 1, 2, ...
    pub refunds: Vec<i64>,
}

impl TripPayment {
    /// Aggregate rỗng của một chuyến chưa có dữ kiện nào.
    pub fn new(trip_id: &str) -> TripPayment {
        TripPayment {
            trip_id: trip_id.to_string(),
            trip: None,
            capture: None,
            promo: None,
            settled: false,
            refunds: Vec::new(),
        }
    }

    /// Nhận `TripCompleted`. Trùng hệt thì không đổi gì, khác nội dung thì từ chối.
    ///
    /// # Errors
    /// [`DomainError::Conflict`] hoặc [`DomainError::Invalid`].
    pub fn record_trip(&mut self, ev: &TripCompleted) -> Result<Vec<DomainEvent>, DomainError> {
        if ev.fare_dong <= 0 {
            return Err(DomainError::Invalid(format!(
                "fareDong {} <= 0",
                ev.fare_dong
            )));
        }
        let facts = TripFacts {
            customer_id: ev.customer_id.clone(),
            driver_id: ev.driver_id.clone(),
            fare_dong: ev.fare_dong,
            occurred_at: ev.occurred_at.clone(),
        };
        match &self.trip {
            Some(known) if *known == facts => return Ok(Vec::new()),
            Some(_) => return Err(self.conflict("TripCompleted")),
            None => {}
        }
        self.trip = Some(facts);
        let mut events = vec![DomainEvent::TripRecorded];
        self.advance(&mut events)?;
        Ok(events)
    }

    /// Nhận `PaymentCaptured`. Trùng hệt thì không đổi gì, khác nội dung thì từ chối.
    ///
    /// # Errors
    /// [`DomainError::Conflict`] hoặc [`DomainError::Invalid`].
    pub fn record_capture(
        &mut self,
        ev: &PaymentCaptured,
    ) -> Result<Vec<DomainEvent>, DomainError> {
        if ev.amount_dong <= 0 {
            return Err(DomainError::Invalid(format!(
                "amountDong {} <= 0",
                ev.amount_dong
            )));
        }
        let facts = CaptureFacts {
            payment_id: ev.payment_id.clone(),
            amount_dong: ev.amount_dong,
            external_ref: ev.external_ref.clone(),
            occurred_at: ev.occurred_at.clone(),
        };
        match &self.capture {
            Some(known) if *known == facts => return Ok(Vec::new()),
            Some(_) => return Err(self.conflict("PaymentCaptured")),
            None => {}
        }
        self.capture = Some(facts);
        let mut events = vec![DomainEvent::CaptureRecorded];
        self.advance(&mut events)?;
        Ok(events)
    }

    /// Nhận quyết định khuyến mãi. Bút toán dùng ngân sách do `redeem` trả về, kèm `Funds`
    /// để adapter kiểm ngân sách còn đủ trong cùng transaction.
    ///
    /// # Errors
    /// Quyết định sai, khác bản đã nhận, hoặc đến sau khi chuyến đã chốt sổ.
    pub fn grant_promo(&mut self, grant: &PromoGrant) -> Result<Vec<DomainEvent>, DomainError> {
        if self.promo.as_ref() == Some(grant) {
            return Ok(Vec::new());
        }
        if self.promo.is_some() {
            return Err(self.conflict("PromoGrant"));
        }
        if self.settled {
            return Err(DomainError::PromoTooLate {
                trip: self.trip_id.clone(),
            });
        }
        if grant.platform_dong < 0
            || grant.driver_dong < 0
            || grant.platform_dong + grant.driver_dong == 0
        {
            return Err(DomainError::Invalid(format!(
                "phần chịu không hợp lệ: nền tảng {}, tài xế {}",
                grant.platform_dong, grant.driver_dong
            )));
        }
        if let Some(trip) = &self.trip
            && grant.platform_dong + grant.driver_dong > trip.fare_dong
        {
            return Err(DomainError::Invalid(format!(
                "mức giảm {} vượt cước {}",
                grant.platform_dong + grant.driver_dong,
                trip.fare_dong
            )));
        }
        let redeem = if grant.platform_dong > 0 {
            Some(PlannedEntry {
                entry: flows::redeem_entry(
                    &self.trip_id,
                    &grant.campaign,
                    Money::from_dong(grant.platform_dong),
                )?,
                external_ref: None,
                requires_funds: Some(Funds {
                    account: flows::budget_account(&grant.campaign),
                    at_least: grant.platform_dong,
                }),
            })
        } else {
            None
        };
        self.promo = Some(grant.clone());
        let mut events = vec![DomainEvent::PromoGranted {
            campaign: grant.campaign.clone(),
            redeem,
        }];
        self.advance(&mut events)?;
        Ok(events)
    }

    /// Hoàn một phần, lần thứ `n` (1, 2, ...). Lần đã nhận cùng số tiền thì không đổi gì.
    ///
    /// # Errors
    /// Chưa chốt sổ, nhảy cóc, vượt số đã thu, chuyến có khuyến mãi, hoặc khác bản đã nhận.
    pub fn refund(&mut self, n: u32, amount: Money) -> Result<Vec<DomainEvent>, DomainError> {
        let (Some(trip), Some(capture)) = (&self.trip, &self.capture) else {
            return Err(DomainError::NotSettled {
                trip: self.trip_id.clone(),
            });
        };
        if !self.settled {
            return Err(DomainError::NotSettled {
                trip: self.trip_id.clone(),
            });
        }
        if amount <= Money::ZERO {
            return Err(DomainError::Invalid(format!("số hoàn {amount} <= 0")));
        }
        let index = usize::try_from(n).unwrap_or(usize::MAX);
        if index >= 1 && index <= self.refunds.len() {
            return if self.refunds[index - 1] == amount.dong() {
                Ok(Vec::new())
            } else {
                Err(self.conflict("Refund"))
            };
        }
        let expected = u32::try_from(self.refunds.len() + 1).unwrap_or(u32::MAX);
        if n != expected {
            return Err(DomainError::RefundOutOfSequence {
                trip: self.trip_id.clone(),
                expected,
                got: n,
            });
        }
        if self.promo.is_some() {
            return Err(DomainError::RefundOnPromoTrip {
                trip: self.trip_id.clone(),
            });
        }
        let done: i64 = self.refunds.iter().sum();
        let remaining = Money::from_dong(capture.amount_dong - done);
        if amount > remaining {
            return Err(DomainError::RefundExceeds {
                remaining,
                asked: amount,
            });
        }
        let captured = Money::from_dong(capture.amount_dong);
        let fee = default_policy()
            .split(Money::from_dong(trip.fare_dong))?
            .platform_fee;
        let (fee_part, driver_part) = flows::refund_split(captured, fee, done, amount)?;
        let entry = flows::refund_entry(
            &self.trip_id,
            n,
            &trip.customer_id,
            &trip.driver_id,
            amount,
            fee_part,
            driver_part,
        )?;
        self.refunds.push(amount.dong());
        Ok(vec![DomainEvent::Refunded {
            n,
            entry: PlannedEntry::plain(entry),
        }])
    }

    /// Điều còn chờ trước khi chốt sổ; rỗng khi đã chốt.
    pub fn waiting_for(&self) -> Vec<Waiting> {
        if self.settled {
            return Vec::new();
        }
        match (&self.trip, &self.capture) {
            (None, None) => vec![Waiting::TripCompleted, Waiting::PaymentCaptured],
            (None, Some(_)) => vec![Waiting::TripCompleted],
            (Some(_), None) => vec![Waiting::PaymentCaptured],
            (Some(trip), Some(capture)) => vec![Waiting::AmountMatch {
                fare: trip.fare_dong,
                discount: self.discount(),
                captured: capture.amount_dong,
            }],
        }
    }

    fn discount(&self) -> i64 {
        self.promo
            .as_ref()
            .map_or(0, |p| p.platform_dong + p.driver_dong)
    }

    fn conflict(&self, fact: &'static str) -> DomainError {
        DomainError::Conflict {
            trip: self.trip_id.clone(),
            fact,
        }
    }

    // Chốt sổ khi đủ chuyến và số thu, và số thu đúng bằng cước trừ khuyến mãi đã biết. Chỉ nhìn
    // trạng thái hiện tại, nên gọi sau dữ kiện nào cũng cho cùng kết quả.
    fn advance(&mut self, events: &mut Vec<DomainEvent>) -> Result<(), DomainError> {
        if self.settled {
            return Ok(());
        }
        let (Some(trip), Some(capture)) = (&self.trip, &self.capture) else {
            return Ok(());
        };
        if capture.amount_dong != trip.fare_dong - self.discount() {
            return Ok(());
        }
        let fare = Money::from_dong(trip.fare_dong);
        let collected = Money::from_dong(capture.amount_dong);
        let capture_entry = PlannedEntry {
            entry: flows::capture_entry(&self.trip_id, &trip.customer_id, collected)?,
            external_ref: Some(capture.external_ref.clone()),
            requires_funds: None,
        };
        let split_entry = PlannedEntry::plain(flows::split_entry(
            &self.trip_id,
            &trip.driver_id,
            fare,
            collected,
            self.promo.as_ref(),
        )?);
        self.settled = true;
        events.push(DomainEvent::Settled {
            entries: vec![capture_entry, split_entry],
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn trip() -> TripCompleted {
        TripCompleted::sample("TN-0001", "CU-001", "DR-001", 25_500)
    }

    fn paid(amount: i64) -> PaymentCaptured {
        PaymentCaptured::sample("TN-0001", "PAY-0001", "vx_ch_0001", amount)
    }

    fn promo() -> PromoGrant {
        PromoGrant {
            campaign: "WELCOME5K".to_string(),
            platform_dong: 5_000,
            driver_dong: 0,
        }
    }

    fn entries(events: &[DomainEvent]) -> Vec<PlannedEntry> {
        events.iter().flat_map(DomainEvent::entries).collect()
    }

    fn shape(planned: &[PlannedEntry]) -> BTreeMap<String, Vec<(String, i64)>> {
        planned
            .iter()
            .map(|p| {
                (
                    p.entry.idempotency_key().to_string(),
                    p.entry
                        .postings()
                        .iter()
                        .map(|x| (x.account.as_str().to_string(), x.amount.dong()))
                        .collect(),
                )
            })
            .collect()
    }

    #[test]
    fn settles_only_when_both_facts_are_in() {
        let mut p = TripPayment::new("TN-0001");
        let first = p.record_trip(&trip()).unwrap();
        assert_eq!(first, vec![DomainEvent::TripRecorded]);
        assert_eq!(p.waiting_for(), vec![Waiting::PaymentCaptured]);
        let second = p.record_capture(&paid(25_500)).unwrap();
        assert_eq!(
            second.iter().map(DomainEvent::name).collect::<Vec<_>>(),
            ["CaptureRecorded", "Settled"]
        );
        assert!(p.settled && p.waiting_for().is_empty());
        let keys: Vec<_> = entries(&second)
            .iter()
            .map(|e| e.entry.idempotency_key().to_string())
            .collect();
        assert_eq!(keys, ["trip:TN-0001:capture", "trip:TN-0001:split"]);
    }

    // Sáu thứ tự đến của ba dữ kiện: trạng thái cuối và tập bút toán phải y hệt nhau.
    #[test]
    fn every_arrival_order_settles_the_same() {
        let orders = [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ];
        let mut outcomes = Vec::new();
        for order in orders {
            let mut p = TripPayment::new("TN-0001");
            let mut planned = Vec::new();
            for step in order {
                let events = match step {
                    0 => p.record_trip(&trip()).unwrap(),
                    1 => p.record_capture(&paid(20_500)).unwrap(),
                    _ => p.grant_promo(&promo()).unwrap(),
                };
                planned.extend(entries(&events));
            }
            assert!(p.settled, "thứ tự {order:?} phải chốt sổ");
            outcomes.push((p, shape(&planned)));
        }
        assert!(outcomes.windows(2).all(|w| w[0] == w[1]));
        let keys: Vec<_> = outcomes[0].1.keys().cloned().collect();
        assert_eq!(
            keys,
            [
                "trip:TN-0001:capture",
                "trip:TN-0001:promo:WELCOME5K",
                "trip:TN-0001:split"
            ]
        );
    }

    #[test]
    fn exact_duplicates_change_nothing_but_changed_ones_conflict() {
        let mut p = TripPayment::new("TN-0001");
        p.record_trip(&trip()).unwrap();
        assert!(p.record_trip(&trip()).unwrap().is_empty());
        let mut other = trip();
        other.fare_dong = 26_000;
        assert_eq!(
            p.record_trip(&other),
            Err(DomainError::Conflict {
                trip: "TN-0001".into(),
                fact: "TripCompleted"
            })
        );
        p.record_capture(&paid(25_500)).unwrap();
        assert!(p.record_capture(&paid(25_500)).unwrap().is_empty());
        let mut second_payment = paid(25_500);
        second_payment.payment_id = "PAY-0002".into();
        assert!(matches!(
            p.record_capture(&second_payment),
            Err(DomainError::Conflict { .. })
        ));
    }

    #[test]
    fn mismatched_amount_waits_and_writes_nothing() {
        let mut p = TripPayment::new("TN-0001");
        p.record_trip(&trip()).unwrap();
        let events = p.record_capture(&paid(24_000)).unwrap();
        assert!(entries(&events).is_empty());
        assert!(!p.settled);
        assert_eq!(
            p.waiting_for(),
            vec![Waiting::AmountMatch {
                fare: 25_500,
                discount: 0,
                captured: 24_000
            }]
        );
    }

    #[test]
    fn promo_after_settlement_is_too_late() {
        let mut p = TripPayment::new("TN-0001");
        p.record_trip(&trip()).unwrap();
        p.record_capture(&paid(25_500)).unwrap();
        assert!(matches!(
            p.grant_promo(&promo()),
            Err(DomainError::PromoTooLate { .. })
        ));
    }

    #[test]
    fn refunds_follow_the_cumulative_split() {
        let mut p = TripPayment::new("TN-0001");
        p.record_trip(&trip()).unwrap();
        assert!(matches!(
            p.refund(1, Money::from_dong(1)),
            Err(DomainError::NotSettled { .. })
        ));
        p.record_capture(&paid(25_500)).unwrap();
        let first = p.refund(1, Money::from_dong(10_000)).unwrap();
        let lines = shape(&entries(&first));
        assert_eq!(
            lines["trip:TN-0001:refund:1"],
            [
                ("customer:CU-001".to_string(), 10_000),
                ("platform:revenue".to_string(), -2_000),
                ("driver:DR-001".to_string(), -8_000)
            ]
        );
        assert!(p.refund(1, Money::from_dong(10_000)).unwrap().is_empty());
        assert!(matches!(
            p.refund(1, Money::from_dong(9_000)),
            Err(DomainError::Conflict { .. })
        ));
        assert!(matches!(
            p.refund(3, Money::from_dong(1_000)),
            Err(DomainError::RefundOutOfSequence { expected: 2, .. })
        ));
        assert!(matches!(
            p.refund(2, Money::from_dong(16_000)),
            Err(DomainError::RefundExceeds { .. })
        ));
        p.refund(2, Money::from_dong(15_500)).unwrap();
        assert_eq!(p.refunds, [10_000, 15_500]);
    }

    #[test]
    fn promo_trips_are_not_refundable_in_the_lab() {
        let mut p = TripPayment::new("TN-0001");
        p.grant_promo(&promo()).unwrap();
        p.record_trip(&trip()).unwrap();
        p.record_capture(&paid(20_500)).unwrap();
        assert!(matches!(
            p.refund(1, Money::from_dong(1_000)),
            Err(DomainError::RefundOnPromoTrip { .. })
        ));
    }

    #[test]
    fn state_round_trips_through_json() {
        let mut p = TripPayment::new("TN-0001");
        p.record_trip(&trip()).unwrap();
        let json = serde_json::to_value(&p).unwrap();
        assert_eq!(json["tripId"], "TN-0001");
        assert_eq!(json["trip"]["fareDong"], 25_500);
        let back: TripPayment = serde_json::from_value(json).unwrap();
        assert_eq!(back, p);
    }
}
