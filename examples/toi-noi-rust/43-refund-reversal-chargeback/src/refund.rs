//! Hoàn tiền theo maker-checker: tạo yêu cầu, duyệt (ghi bút toán đúng một lần), từ chối, đảo toàn phần.

use std::sync::Arc;

use ledger_core::{
    AccountId, JournalEntry, Money, PaymentEvent, PaymentState, Posting, transition,
};
use sqlx::{PgConnection, PgPool, Row};
use tokio::sync::Barrier;

use crate::error::RefundError;
use crate::ledger::{Posted, audit, post_entry, reverse_entry};
use crate::model::{ReasonCode, ReasonKind, refund_split};

/// Có khoá dòng `payments` khi duyệt hay không. `Unlocked` chỉ để thí nghiệm cho thấy
/// chốt chặn `CHECK` làm gì khi thiếu khoá; production luôn dùng `Locked`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Guard {
    /// `SELECT ... FOR UPDATE` trên dòng `payments`.
    Locked,
    /// Đọc không khoá.
    Unlocked,
}

/// Điểm dừng cho test đồng thời: mọi task đến `Barrier` rồi mới đi tiếp.
#[derive(Clone, Default)]
pub enum SyncPoint {
    /// Không dừng.
    #[default]
    None,
    /// Chờ ngay khi vào hàm, trước khi mở transaction.
    Start(Arc<Barrier>),
    /// Chờ ngay sau khi đọc `payments`, trước khi ghi (chỉ có nghĩa với `Guard::Unlocked`).
    AfterRead(Arc<Barrier>),
}

/// Kết quả duyệt một yêu cầu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Approval {
    /// Mã yêu cầu.
    pub request_id: i64,
    /// Bút toán hoàn tiền.
    pub entry_id: i64,
    /// `true` nếu yêu cầu đã được duyệt từ trước: không ghi gì thêm.
    pub replayed: bool,
    /// Trạng thái thanh toán sau khi duyệt.
    pub status: PaymentState,
}

/// Hai bút toán đảo của một chuyến.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TripReversal {
    /// Đảo của `trip:<id>:split`.
    pub split: Posted,
    /// Đảo của `trip:<id>:capture`.
    pub capture: Posted,
}

struct PaymentRow {
    customer: String,
    driver: String,
    captured: i64,
    fee: i64,
    refunded: i64,
    status: PaymentState,
}

// Hai câu lệnh tĩnh, chỉ khác `for update`: không ghép chuỗi SQL.
async fn load_payment(
    conn: &mut PgConnection,
    trip_id: &str,
    lock: bool,
) -> Result<PaymentRow, RefundError> {
    let sql = if lock {
        "select customer_id, driver_id, captured_dong, fee_dong, refunded_dong, status \
         from payments where trip_id = $1 for update"
    } else {
        "select customer_id, driver_id, captured_dong, fee_dong, refunded_dong, status \
         from payments where trip_id = $1"
    };
    let row = sqlx::query(sql)
        .bind(trip_id)
        .fetch_optional(conn)
        .await?
        .ok_or_else(|| RefundError::UnknownTrip(trip_id.to_string()))?;
    let status: String = row.get("status");
    Ok(PaymentRow {
        customer: row.get("customer_id"),
        driver: row.get("driver_id"),
        captured: row.get("captured_dong"),
        fee: row.get("fee_dong"),
        refunded: row.get("refunded_dong"),
        status: status.parse()?,
    })
}

struct RequestRow {
    trip_id: String,
    amount: i64,
    status: String,
    created_by: String,
    entry_id: Option<i64>,
}

async fn lock_request(conn: &mut PgConnection, id: i64) -> Result<RequestRow, RefundError> {
    let row = sqlx::query(
        "select trip_id, amount_dong, status, created_by, entry_id \
         from refund_requests where id = $1 for update",
    )
    .bind(id)
    .fetch_optional(conn)
    .await?
    .ok_or(RefundError::RequestNotFound(id))?;
    Ok(RequestRow {
        trip_id: row.get("trip_id"),
        amount: row.get("amount_dong"),
        status: row.get("status"),
        created_by: row.get("created_by"),
        entry_id: row.get("entry_id"),
    })
}

async fn pending_left(conn: &mut PgConnection, trip_id: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "select count(*) from refund_requests where trip_id = $1 and status = 'PENDING'",
    )
    .bind(trip_id)
    .fetch_one(conn)
    .await
}

/// Dịch vụ hoàn tiền trên một pool đã đặt `search_path`.
#[derive(Clone)]
pub struct RefundService {
    pool: PgPool,
}

impl RefundService {
    /// Bọc một pool.
    pub fn new(pool: PgPool) -> RefundService {
        RefundService { pool }
    }

    /// Maker tạo yêu cầu. Chưa có bút toán nào; thanh toán chuyển `CAPTURED` → `REFUND_PENDING`.
    pub async fn create_request(
        &self,
        trip_id: &str,
        amount: Money,
        reason: ReasonCode,
        maker: &str,
    ) -> Result<i64, RefundError> {
        if reason.kind() != ReasonKind::Refund {
            return Err(RefundError::WrongReasonKind(reason.to_string()));
        }
        if amount <= Money::ZERO {
            return Err(RefundError::NonPositiveAmount(amount));
        }
        let mut tx = self.pool.begin().await?;
        let pay = load_payment(&mut tx, trip_id, true).await?;
        let next = match pay.status {
            PaymentState::Captured => transition(pay.status, PaymentEvent::RequestRefund)?,
            // Đã có yêu cầu khác đang chờ: vẫn nhận thêm yêu cầu.
            PaymentState::RefundPending => PaymentState::RefundPending,
            other => {
                return Err(RefundError::PaymentNotRefundable {
                    status: other.to_string(),
                });
            }
        };
        let refundable = pay.captured - pay.refunded;
        if amount.dong() > refundable {
            return Err(RefundError::ExceedsRefundable {
                requested: amount,
                refundable: Money::from_dong(refundable),
            });
        }
        let id: i64 = sqlx::query_scalar(
            "insert into refund_requests (trip_id, amount_dong, reason_code, created_by) \
             values ($1, $2, $3, $4) returning id",
        )
        .bind(trip_id)
        .bind(amount.dong())
        .bind(reason.as_str())
        .bind(maker)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query("update payments set status = $2 where trip_id = $1")
            .bind(trip_id)
            .bind(next.as_str())
            .execute(&mut *tx)
            .await?;
        audit(
            &mut tx,
            maker,
            "REFUND_REQUESTED",
            &format!("refund_request:{id}"),
            &format!("{trip_id} {amount} {reason}"),
        )
        .await?;
        tx.commit().await?;
        Ok(id)
    }

    /// Checker duyệt: ghi bút toán hoàn đúng một lần.
    pub async fn approve(&self, request_id: i64, checker: &str) -> Result<Approval, RefundError> {
        self.approve_with(request_id, checker, Guard::Locked, SyncPoint::None)
            .await
    }

    /// Như [`approve`](Self::approve) nhưng chọn `Guard` và điểm dừng cho thí nghiệm đồng thời.
    pub async fn approve_with(
        &self,
        request_id: i64,
        checker: &str,
        guard: Guard,
        sync: SyncPoint,
    ) -> Result<Approval, RefundError> {
        if let SyncPoint::Start(b) = &sync {
            b.wait().await;
        }
        let mut tx = self.pool.begin().await?;
        // Khoá dòng yêu cầu trước: hai lần duyệt cùng một yêu cầu xếp hàng ở đây.
        let req = lock_request(&mut tx, request_id).await?;
        if req.created_by == checker {
            tx.rollback().await?;
            // Lần bị từ chối vẫn phải để lại dấu vết: ghi ngoài transaction đã rollback.
            // Hai bước không nguyên tử: chết giữa chừng thì tiền không đổi nhưng mất dòng audit này.
            let mut conn = self.pool.acquire().await?;
            audit(
                &mut conn,
                checker,
                "APPROVAL_DENIED",
                &format!("refund_request:{request_id}"),
                "maker-checker: người tạo không tự duyệt",
            )
            .await?;
            return Err(RefundError::SelfApproval {
                actor: checker.to_string(),
            });
        }
        let pay = load_payment(&mut tx, &req.trip_id, guard == Guard::Locked).await?;
        match req.status.as_str() {
            "APPROVED" => {
                // Duyệt lặp lại: trả kết quả cũ, không ghi gì.
                return Ok(Approval {
                    request_id,
                    entry_id: req.entry_id.unwrap_or_default(),
                    replayed: true,
                    status: pay.status,
                });
            }
            "PENDING" => {}
            other => {
                return Err(RefundError::AlreadyDecided {
                    what: format!("yêu cầu {request_id}"),
                    status: other.to_string(),
                });
            }
        }
        if let SyncPoint::AfterRead(b) = &sync {
            b.wait().await;
        }
        let refundable = pay.captured - pay.refunded;
        if req.amount > refundable {
            return Err(RefundError::ExceedsRefundable {
                requested: Money::from_dong(req.amount),
                refundable: Money::from_dong(refundable),
            });
        }
        let split = refund_split(
            Money::from_dong(pay.captured),
            Money::from_dong(pay.fee),
            Money::from_dong(pay.refunded),
            Money::from_dong(req.amount),
        );
        // Khách nhận lại đủ số hoàn; nền tảng và tài xế mỗi bên trả phần của mình (bỏ dòng 0 đồng).
        let mut postings = vec![Posting::new(
            AccountId::new(&pay.customer)?,
            Money::from_dong(req.amount),
        )];
        if split.fee_part > 0 {
            postings.push(Posting::new(
                AccountId::new("platform:revenue")?,
                Money::from_dong(-split.fee_part),
            ));
        }
        if split.driver_part > 0 {
            postings.push(Posting::new(
                AccountId::new(&pay.driver)?,
                Money::from_dong(-split.driver_part),
            ));
        }
        let entry = JournalEntry::try_new(
            &format!("trip:{}:refund:{request_id}", req.trip_id),
            postings,
        )?;
        let posted = post_entry(&mut tx, &entry).await?;

        sqlx::query(
            "update refund_requests set status = 'APPROVED', decided_by = $2, decided_at = now(), entry_id = $3 \
             where id = $1",
        )
        .bind(request_id)
        .bind(checker)
        .bind(posted.id)
        .execute(&mut *tx)
        .await?;
        let still_pending = pending_left(&mut tx, &req.trip_id).await?;
        let next = if pay.refunded + req.amount == pay.captured {
            transition(pay.status, PaymentEvent::CompleteRefund)?
        } else if still_pending > 0 {
            PaymentState::RefundPending
        } else {
            // Cạnh RefundPending → Captured do lab tự thêm: `transition` của lab 20 không có, và lab 20 giữ nguyên.
            PaymentState::Captured
        };
        // Cộng tương đối, không ghi đè bằng số đã đọc: CHECK refund_within_captured là chốt chặn cuối.
        sqlx::query("update payments set refunded_dong = refunded_dong + $2, status = $3 where trip_id = $1")
            .bind(&req.trip_id)
            .bind(req.amount)
            .bind(next.as_str())
            .execute(&mut *tx)
            .await?;
        audit(
            &mut tx,
            checker,
            "REFUND_APPROVED",
            &format!("refund_request:{request_id}"),
            &format!("JE-{:06} {}", posted.id, Money::from_dong(req.amount)),
        )
        .await?;
        tx.commit().await?;
        Ok(Approval {
            request_id,
            entry_id: posted.id,
            replayed: false,
            status: next,
        })
    }

    /// Checker từ chối; trả `true` nếu yêu cầu đã bị từ chối từ trước (lặp lại không đổi gì).
    pub async fn reject(&self, request_id: i64, checker: &str) -> Result<bool, RefundError> {
        let mut tx = self.pool.begin().await?;
        let req = lock_request(&mut tx, request_id).await?;
        if req.created_by == checker {
            tx.rollback().await?;
            // Như `approve_with`: ghi ngoài transaction đã rollback, cùng giới hạn khi chết giữa chừng.
            let mut conn = self.pool.acquire().await?;
            audit(
                &mut conn,
                checker,
                "REJECTION_DENIED",
                &format!("refund_request:{request_id}"),
                "maker-checker: người tạo không tự quyết định",
            )
            .await?;
            return Err(RefundError::SelfApproval {
                actor: checker.to_string(),
            });
        }
        match req.status.as_str() {
            "REJECTED" => return Ok(true),
            "PENDING" => {}
            other => {
                return Err(RefundError::AlreadyDecided {
                    what: format!("yêu cầu {request_id}"),
                    status: other.to_string(),
                });
            }
        }
        let pay = load_payment(&mut tx, &req.trip_id, true).await?;
        sqlx::query("update refund_requests set status = 'REJECTED', decided_by = $2, decided_at = now() where id = $1")
            .bind(request_id)
            .bind(checker)
            .execute(&mut *tx)
            .await?;
        if pending_left(&mut tx, &req.trip_id).await? == 0
            && pay.status == PaymentState::RefundPending
        {
            // Cùng cạnh RefundPending → Captured do lab tự thêm, không qua `transition`.
            sqlx::query("update payments set status = 'CAPTURED' where trip_id = $1")
                .bind(&req.trip_id)
                .execute(&mut *tx)
                .await?;
        }
        audit(
            &mut tx,
            checker,
            "REFUND_REJECTED",
            &format!("refund_request:{request_id}"),
            "",
        )
        .await?;
        tx.commit().await?;
        Ok(false)
    }

    /// Đảo toàn phần cả chuyến: đảo `split` rồi `capture`. Chỉ khi chưa hoàn đồng nào và không có yêu cầu đang chờ.
    pub async fn reverse_trip(
        &self,
        trip_id: &str,
        actor: &str,
    ) -> Result<TripReversal, RefundError> {
        let mut tx = self.pool.begin().await?;
        let pay = load_payment(&mut tx, trip_id, true).await?;
        match pay.status {
            PaymentState::Captured => {}
            PaymentState::Refunded => {
                let capture_id: i64 =
                    sqlx::query_scalar("select id from journal_entries where idempotency_key = $1")
                        .bind(format!("trip:{trip_id}:capture"))
                        .fetch_one(&mut *tx)
                        .await?;
                return Err(RefundError::AlreadyReversed(capture_id));
            }
            other => {
                return Err(RefundError::PaymentNotRefundable {
                    status: other.to_string(),
                });
            }
        }
        if pay.refunded != 0 {
            return Err(RefundError::PartialRefundExists {
                refunded: Money::from_dong(pay.refunded),
            });
        }
        let mut ids = [0_i64; 2];
        for (slot, step) in ids.iter_mut().zip(["split", "capture"]) {
            *slot = sqlx::query_scalar("select id from journal_entries where idempotency_key = $1")
                .bind(format!("trip:{trip_id}:{step}"))
                .fetch_one(&mut *tx)
                .await?;
        }
        let split = reverse_entry(&mut tx, ids[0]).await?;
        let capture = reverse_entry(&mut tx, ids[1]).await?;
        // Hai bước của máy trạng thái bài 20: CAPTURED → REFUND_PENDING → REFUNDED.
        let pending = transition(pay.status, PaymentEvent::RequestRefund)?;
        let done = transition(pending, PaymentEvent::CompleteRefund)?;
        sqlx::query(
            "update payments set refunded_dong = captured_dong, status = $2 where trip_id = $1",
        )
        .bind(trip_id)
        .bind(done.as_str())
        .execute(&mut *tx)
        .await?;
        audit(
            &mut tx,
            actor,
            "TRIP_REVERSED",
            &format!("trip:{trip_id}"),
            &format!("JE-{:06} JE-{:06}", split.id, capture.id),
        )
        .await?;
        tx.commit().await?;
        Ok(TripReversal { split, capture })
    }
}
