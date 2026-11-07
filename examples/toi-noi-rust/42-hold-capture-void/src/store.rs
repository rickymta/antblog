//! Thao tác hold trên PostgreSQL: tạo, capture, void, hết hạn và đọc số dư khả dụng.

use jiff::SignedDuration;
use ledger_core::Money;
use sqlx::postgres::PgPool;
use sqlx::{PgConnection, Row};
use tokio::sync::{Barrier, Notify};

use crate::clock::Clock;
use crate::hold::{self, HoldError, HoldRow, HoldStatus, Locked, to_pg};
use crate::ledger::load_by_key;

/// Cách đọc số dư khi tạo hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockMode {
    /// `SELECT ... FOR UPDATE` trên `account_balances`: các hold cùng tài khoản xếp hàng.
    ForUpdate,
    /// Đọc thường. Chỉ để chứng minh vì sao thiếu khóa là sai.
    None,
}

/// Điểm dừng trong `create_hold`, sau khi tính khả dụng và trước khi chèn hold.
/// Chỉ dùng để dựng cuộc đua có kiểm soát trong lab; production không có tham số này.
pub enum Gate<'a> {
    /// Không dừng.
    Open,
    /// Đợi đủ số bên tới điểm này rồi cùng đi tiếp.
    Barrier(&'a Barrier),
    /// Báo «đã tới», rồi đợi được cho đi tiếp.
    Pause(&'a Pause),
}

/// Cặp tín hiệu cho [`Gate::Pause`].
#[derive(Default)]
pub struct Pause {
    /// Phát khi `create_hold` đã tới điểm dừng.
    pub reached: Notify,
    /// Người điều khiển phát để cho `create_hold` đi tiếp.
    pub resume: Notify,
}

impl Gate<'_> {
    async fn reach(&self) {
        match self {
            Gate::Open => {}
            Gate::Barrier(barrier) => {
                barrier.wait().await;
            }
            Gate::Pause(pause) => {
                pause.reached.notify_one();
                pause.resume.notified().await;
            }
        }
    }
}

/// Yêu cầu giữ tiền.
#[derive(Debug, Clone)]
pub struct NewHold<'a> {
    /// Ví cần giữ tiền, ví dụ `customer:CU-001`.
    pub account: &'a str,
    /// Số cần giữ, dương.
    pub amount: Money,
    /// Key idempotent, ví dụ `trip:TN-0001:hold`.
    pub key: &'a str,
    /// Hold sống bao lâu kể từ `clock.now()`.
    pub ttl: SignedDuration,
}

/// Kết quả tạo hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldOutcome {
    /// Hold mới.
    Created {
        /// Id hold.
        id: i64,
    },
    /// Key đã có và nội dung trùng: trả hold cũ.
    Replayed {
        /// Id hold cũ.
        id: i64,
    },
}

impl HoldOutcome {
    /// Id hold, mới hay cũ.
    pub fn id(self) -> i64 {
        match self {
            HoldOutcome::Created { id } | HoldOutcome::Replayed { id } => id,
        }
    }
}

/// Ba con số của một ví tại một snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Funds {
    /// Số dư sổ cái (`account_balances`).
    pub balance: Money,
    /// Tổng phần còn lại của hold ACTIVE.
    pub held: Money,
    /// `balance - held`.
    pub available: Money,
}

/// Số dư, phần đang giữ và khả dụng trong MỘT câu lệnh (một snapshot, không lệch giữa hai lần đọc).
///
/// # Errors
/// `NoBalanceRow` hoặc lỗi SQL.
pub async fn funds(pool: &PgPool, account: &str) -> Result<Funds, HoldError> {
    let row = sqlx::query(
        "SELECT b.balance_dong, COALESCE((SELECT SUM(h.amount_dong - h.captured_dong) \
         FROM holds h WHERE h.account_id = b.account_id AND h.status = 'ACTIVE'), 0)::bigint AS held \
         FROM account_balances b WHERE b.account_id = $1",
    )
    .bind(account)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| HoldError::NoBalanceRow(account.to_string()))?;
    let balance: i64 = row.get("balance_dong");
    let held: i64 = row.get("held");
    Ok(Funds {
        balance: Money::from_dong(balance),
        held: Money::from_dong(held),
        available: Money::from_dong(balance - held),
    })
}

/// Đọc một hold (không khóa).
///
/// # Errors
/// `NotFound` hoặc lỗi SQL.
pub async fn get_hold(pool: &PgPool, id: i64) -> Result<HoldRow, HoldError> {
    let row = sqlx::query(
        "SELECT id, account_id, amount_dong, captured_dong, status, expires_at \
         FROM holds WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(HoldError::NotFound(id))?;
    HoldRow::from_row(&row)
}

/// Tạo hold nếu số dư khả dụng đủ. Hold không ghi bút toán và không đổi `balance_dong`.
///
/// Với [`LockMode::ForUpdate`], bước khóa hàng số dư đi TRƯỚC bước tính tổng hold. PostgreSQL ở
/// READ COMMITTED cho mỗi câu lệnh một snapshot riêng, nên câu `SUM` chạy sau khi giữ khóa
/// thấy mọi hold đã commit của các giao dịch đứng trước.
///
/// `FOR UPDATE` trên một hàng chưa tồn tại không khóa gì, nên ví chưa có hàng số dư bị từ chối
/// bằng `NoBalanceRow`, không coi là số dư 0 rồi đi tiếp.
///
/// # Errors
/// `NoBalanceRow`, `InsufficientAvailable`, `IdempotencyConflict`, `InvalidAmount` hoặc lỗi SQL.
pub async fn create_hold(
    pool: &PgPool,
    clock: &impl Clock,
    req: &NewHold<'_>,
    mode: LockMode,
    gate: Gate<'_>,
) -> Result<HoldOutcome, HoldError> {
    if req.amount.dong() <= 0 {
        return Err(HoldError::InvalidAmount(req.amount.dong()));
    }
    let now = clock.now();
    let expires_at = now
        .checked_add(req.ttl)
        .map_err(|e| HoldError::Time(e.to_string()))?;
    let mut tx = pool.begin().await?;

    // 1. Khóa (hoặc không) hàng số dư của ví.
    let balance_sql = match mode {
        LockMode::ForUpdate => {
            "SELECT balance_dong FROM account_balances WHERE account_id = $1 FOR UPDATE"
        }
        LockMode::None => "SELECT balance_dong FROM account_balances WHERE account_id = $1",
    };
    let balance: i64 = sqlx::query(balance_sql)
        .bind(req.account)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| HoldError::NoBalanceRow(req.account.to_string()))?
        .get(0);

    // 2. Replay theo key: cùng ví, cùng số tiền thì trả hold cũ.
    if let Some(row) = sqlx::query(
        "SELECT id, account_id, amount_dong, captured_dong, status, expires_at \
         FROM holds WHERE idempotency_key = $1",
    )
    .bind(req.key)
    .fetch_optional(&mut *tx)
    .await?
    {
        let old = HoldRow::from_row(&row)?;
        tx.rollback().await?;
        return if old.account == req.account && old.amount == req.amount {
            Ok(HoldOutcome::Replayed { id: old.id })
        } else {
            Err(HoldError::IdempotencyConflict {
                key: req.key.to_string(),
            })
        };
    }

    // 3. Phần đang giữ của các hold ACTIVE.
    let held: i64 = sqlx::query(
        "SELECT COALESCE(SUM(amount_dong - captured_dong), 0)::bigint FROM holds \
         WHERE account_id = $1 AND status = 'ACTIVE'",
    )
    .bind(req.account)
    .fetch_one(&mut *tx)
    .await?
    .get(0);
    let available = balance - held;

    gate.reach().await; // chỉ lab: dừng giữa «kiểm» và «ghi»

    if req.amount.dong() > available {
        tx.rollback().await?;
        return Err(HoldError::InsufficientAvailable {
            available: Money::from_dong(available),
            requested: req.amount,
        });
    }

    // 4. Ghi hold. `created_at` lấy từ clock truyền vào, không dùng now() của DB.
    let id: i64 = sqlx::query(
        "INSERT INTO holds (account_id, amount_dong, status, idempotency_key, expires_at, created_at) \
         VALUES ($1, $2, 'ACTIVE', $3, $4, $5) RETURNING id",
    )
    .bind(req.account)
    .bind(req.amount.dong())
    .bind(req.key)
    .bind(to_pg(expires_at)?)
    .bind(to_pg(now)?)
    .fetch_one(&mut *tx)
    .await?
    .get("id");
    tx.commit().await?;
    Ok(HoldOutcome::Created { id })
}

/// Kết quả capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureOutcome {
    /// Bút toán capture mới.
    Captured {
        /// Id bút toán.
        entry_id: i64,
    },
    /// Hold đã capture đúng số này với đúng key: trả bút toán cũ, không ghi gì thêm.
    Replayed {
        /// Id bút toán đã ghi trước đó.
        entry_id: i64,
    },
}

/// Capture `amount` của hold. Idempotent theo `key`.
///
/// # Errors
/// `NotActive`, `Expired`, `CaptureExceedsHold`, `IdempotencyConflict` hoặc lỗi SQL.
pub async fn capture_hold(
    pool: &PgPool,
    clock: &impl Clock,
    hold_id: i64,
    amount: Money,
    key: &str,
) -> Result<CaptureOutcome, HoldError> {
    let mut tx = pool.begin().await?;
    match hold::lock(&mut tx, hold_id).await? {
        Locked::Active(active) => {
            let entry_id = active.capture(&mut tx, clock.now(), amount, key).await?;
            tx.commit().await?;
            Ok(CaptureOutcome::Captured { entry_id })
        }
        Locked::Closed(row) => {
            let replay = replay_capture(&mut tx, &row, amount, key).await;
            tx.rollback().await?;
            replay
        }
    }
}

// Hold đã đóng: chỉ «CAPTURED với đúng bút toán của key này» mới là replay hợp lệ.
async fn replay_capture(
    conn: &mut PgConnection,
    row: &HoldRow,
    amount: Money,
    key: &str,
) -> Result<CaptureOutcome, HoldError> {
    if row.status != HoldStatus::Captured {
        return Err(HoldError::NotActive {
            id: row.id,
            status: row.status,
        });
    }
    match load_by_key(conn, key).await? {
        Some((entry_id, old))
            if row.captured == amount && old == hold::capture_entry(&row.account, amount, key)? =>
        {
            Ok(CaptureOutcome::Replayed { entry_id })
        }
        Some(_) => Err(HoldError::IdempotencyConflict {
            key: key.to_string(),
        }),
        None => Err(HoldError::NotActive {
            id: row.id,
            status: row.status,
        }),
    }
}

/// Kết quả void.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoidOutcome {
    /// Hold vừa bị hủy.
    Voided,
    /// Hold đã bị hủy từ trước: gọi lặp vẫn thành công.
    AlreadyVoided,
}

/// Hủy hold (khách hủy chuyến). Gọi lặp là `AlreadyVoided`; hold đã capture hoặc hết hạn thì lỗi.
///
/// # Errors
/// `NotActive` hoặc lỗi SQL.
pub async fn void_hold(pool: &PgPool, hold_id: i64) -> Result<VoidOutcome, HoldError> {
    let mut tx = pool.begin().await?;
    match hold::lock(&mut tx, hold_id).await? {
        Locked::Active(active) => {
            active.void(&mut tx).await?;
            tx.commit().await?;
            Ok(VoidOutcome::Voided)
        }
        Locked::Closed(row) => {
            tx.rollback().await?;
            if row.status == HoldStatus::Voided {
                Ok(VoidOutcome::AlreadyVoided)
            } else {
                Err(HoldError::NotActive {
                    id: row.id,
                    status: row.status,
                })
            }
        }
    }
}

/// Job hết hạn: chuyển hold ACTIVE có `expires_at <= clock.now()` sang EXPIRED, tối đa `batch` hold.
/// `FOR UPDATE SKIP LOCKED` bỏ qua hold đang bị transaction khác giữ khóa (ví dụ capture đang chạy)
/// thay vì chờ; lần chạy sau sẽ nhặt lại nếu hold vẫn ACTIVE. Trả id các hold vừa hết hạn, tăng dần.
///
/// # Errors
/// Lỗi SQL hoặc thời điểm ngoài miền.
pub async fn expire_due(
    pool: &PgPool,
    clock: &impl Clock,
    batch: i64,
) -> Result<Vec<i64>, HoldError> {
    let rows = sqlx::query(
        "WITH due AS (SELECT id FROM holds WHERE status = 'ACTIVE' AND expires_at <= $1 \
         ORDER BY expires_at, id LIMIT $2 FOR UPDATE SKIP LOCKED) \
         UPDATE holds h SET status = 'EXPIRED' FROM due WHERE h.id = due.id RETURNING h.id",
    )
    .bind(to_pg(clock.now())?)
    .bind(batch)
    .fetch_all(pool)
    .await?;
    let mut ids: Vec<i64> = rows.iter().map(|r| r.get("id")).collect();
    ids.sort_unstable();
    Ok(ids)
}
