//! Chargeback từ PSP: tiền rời `psp:vixanh:clearing` dù nền tảng có thể đã chia cho tài xế.

use ledger_core::{AccountId, JournalEntry, Money, Posting};
use sqlx::{PgPool, Row};

use crate::error::RefundError;
use crate::ledger::{Posted, audit, post_entry, reverse_entry};
use crate::model::{ReasonCode, ReasonKind, refund_split};

/// Ai chịu khoản chargeback khi VíXanh kết luận nền tảng thua.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LossPolicy {
    /// Thu lại phần tài xế đã nhận và phần phí, theo tỷ lệ của cước gốc.
    RecoverFromDriver,
    /// Nền tảng chịu hết: doanh thu bị trừ đủ số PSP đã rút.
    PlatformAbsorbs,
}

/// Một chargeback đã ghi nhận.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opened {
    /// Mã nội bộ.
    pub id: i64,
    /// Bút toán lúc PSP rút tiền.
    pub entry_id: i64,
    /// `true` nếu mã vụ việc của PSP đã nhận từ trước (webhook gửi lại).
    pub replayed: bool,
}

/// Dịch vụ chargeback.
#[derive(Clone)]
pub struct ChargebackService {
    pool: PgPool,
}

impl ChargebackService {
    /// Bọc một pool.
    pub fn new(pool: PgPool) -> ChargebackService {
        ChargebackService { pool }
    }

    /// PSP rút tiền: khách được ghi nhận lại số tiền, `psp:vixanh:clearing` đi âm.
    /// Khoản âm đó là chỗ nền tảng chịu tạm: nó là tiền nền tảng nợ VíXanh, trong khi
    /// ví tài xế vẫn giữ phần đã chia. Ghi theo `psp_case_ref` nên webhook gửi lại không nhân đôi.
    pub async fn open(
        &self,
        psp_case_ref: &str,
        trip_id: &str,
        amount: Money,
        reason: ReasonCode,
    ) -> Result<Opened, RefundError> {
        if reason.kind() != ReasonKind::Chargeback {
            return Err(RefundError::WrongReasonKind(reason.to_string()));
        }
        if amount <= Money::ZERO {
            return Err(RefundError::NonPositiveAmount(amount));
        }
        let mut tx = self.pool.begin().await?;
        // Khoá dòng thanh toán: chargeback và hoàn tiền của cùng chuyến xếp hàng.
        let row = sqlx::query(
            "select customer_id, captured_dong from payments where trip_id = $1 for update",
        )
        .bind(trip_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| RefundError::UnknownTrip(trip_id.to_string()))?;
        let customer: String = row.get("customer_id");
        let captured: i64 = row.get("captured_dong");
        if let Some(old) =
            sqlx::query("select id, entry_id from chargebacks where psp_case_ref = $1")
                .bind(psp_case_ref)
                .fetch_optional(&mut *tx)
                .await?
        {
            return Ok(Opened {
                id: old.get("id"),
                entry_id: old.get("entry_id"),
                replayed: true,
            });
        }
        if amount.dong() > captured {
            return Err(RefundError::ChargebackExceedsCaptured {
                amount,
                captured: Money::from_dong(captured),
            });
        }
        let entry = JournalEntry::try_new(
            &format!("trip:{trip_id}:chargeback:{psp_case_ref}"),
            vec![
                Posting::new(
                    AccountId::new("psp:vixanh:clearing")?,
                    Money::from_dong(-amount.dong()),
                ),
                Posting::new(AccountId::new(&customer)?, amount),
            ],
        )?;
        let posted = post_entry(&mut tx, &entry).await?;
        let id: i64 = sqlx::query_scalar(
            "insert into chargebacks (psp_case_ref, trip_id, amount_dong, reason_code, entry_id) \
             values ($1, $2, $3, $4, $5) returning id",
        )
        .bind(psp_case_ref)
        .bind(trip_id)
        .bind(amount.dong())
        .bind(reason.as_str())
        .bind(posted.id)
        .fetch_one(&mut *tx)
        .await?;
        audit(
            &mut tx,
            "psp:vixanh",
            "CHARGEBACK_OPENED",
            &format!("chargeback:{psp_case_ref}"),
            &format!("{trip_id} {amount} {reason}"),
        )
        .await?;
        tx.commit().await?;
        Ok(Opened {
            id,
            entry_id: posted.id,
            replayed: false,
        })
    }

    /// Nền tảng thắng tranh chấp: PSP trả lại tiền, nên đảo đúng bút toán chargeback (`reversal_of`).
    pub async fn resolve_won(
        &self,
        psp_case_ref: &str,
        actor: &str,
    ) -> Result<Posted, RefundError> {
        let mut tx = self.pool.begin().await?;
        let (_, entry_id, _, _) = lock_open_case(&mut tx, psp_case_ref).await?;
        let posted = reverse_entry(&mut tx, entry_id).await?;
        sqlx::query("update chargebacks set status = 'WON' where psp_case_ref = $1")
            .bind(psp_case_ref)
            .execute(&mut *tx)
            .await?;
        audit(
            &mut tx,
            actor,
            "CHARGEBACK_WON",
            &format!("chargeback:{psp_case_ref}"),
            &format!("JE-{:06}", posted.id),
        )
        .await?;
        tx.commit().await?;
        Ok(posted)
    }

    /// Nền tảng thua: đưa `psp:vixanh:clearing` về 0 bằng bút toán thu hồi theo `LossPolicy`.
    pub async fn resolve_lost(
        &self,
        psp_case_ref: &str,
        policy: LossPolicy,
        actor: &str,
    ) -> Result<Posted, RefundError> {
        let mut tx = self.pool.begin().await?;
        let (trip_id, _, amount, _) = lock_open_case(&mut tx, psp_case_ref).await?;
        let pay = sqlx::query(
            "select driver_id, captured_dong, fee_dong from payments where trip_id = $1 for update",
        )
        .bind(&trip_id)
        .fetch_one(&mut *tx)
        .await?;
        let driver: String = pay.get("driver_id");
        let (captured, fee): (i64, i64) = (pay.get("captured_dong"), pay.get("fee_dong"));
        let mut postings = vec![Posting::new(
            AccountId::new("psp:vixanh:clearing")?,
            Money::from_dong(amount),
        )];
        match policy {
            LossPolicy::RecoverFromDriver => {
                let s = refund_split(
                    Money::from_dong(captured),
                    Money::from_dong(fee),
                    Money::ZERO,
                    Money::from_dong(amount),
                );
                if s.fee_part > 0 {
                    postings.push(Posting::new(
                        AccountId::new("platform:revenue")?,
                        Money::from_dong(-s.fee_part),
                    ));
                }
                if s.driver_part > 0 {
                    postings.push(Posting::new(
                        AccountId::new(&driver)?,
                        Money::from_dong(-s.driver_part),
                    ));
                }
            }
            LossPolicy::PlatformAbsorbs => {
                postings.push(Posting::new(
                    AccountId::new("platform:revenue")?,
                    Money::from_dong(-amount),
                ));
            }
        }
        let entry = JournalEntry::try_new(
            &format!("trip:{trip_id}:chargeback:{psp_case_ref}:recovery"),
            postings,
        )?;
        let posted = post_entry(&mut tx, &entry).await?;
        sqlx::query("update chargebacks set status = 'LOST' where psp_case_ref = $1")
            .bind(psp_case_ref)
            .execute(&mut *tx)
            .await?;
        audit(
            &mut tx,
            actor,
            "CHARGEBACK_LOST",
            &format!("chargeback:{psp_case_ref}"),
            &format!("{policy:?} JE-{:06}", posted.id),
        )
        .await?;
        tx.commit().await?;
        Ok(posted)
    }
}

// Khoá dòng chargeback và bảo đảm còn `OPEN`; trả (trip, entry_id, amount, status).
async fn lock_open_case(
    tx: &mut sqlx::PgConnection,
    psp_case_ref: &str,
) -> Result<(String, i64, i64, String), RefundError> {
    let row = sqlx::query("select trip_id, entry_id, amount_dong, status from chargebacks where psp_case_ref = $1 for update")
        .bind(psp_case_ref)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| RefundError::UnknownChargeback(psp_case_ref.to_string()))?;
    let status: String = row.get("status");
    if status != "OPEN" {
        return Err(RefundError::AlreadyDecided {
            what: format!("chargeback {psp_case_ref}"),
            status,
        });
    }
    Ok((
        row.get("trip_id"),
        row.get("entry_id"),
        row.get("amount_dong"),
        "OPEN".to_string(),
    ))
}
