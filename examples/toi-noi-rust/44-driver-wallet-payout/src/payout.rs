//! Payout theo lô: chọn ví đủ ngưỡng, ghi bút toán, gửi ngân hàng, đảo khi thất bại.

use std::fmt;

use anyhow::{Result, anyhow, bail, ensure};
use jiff::Timestamp;
use ledger_core::{AccountId, EntryId, JournalEntry, JournalEntryBuilder, Money};
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, PgPool, Row};
use tokio::sync::Barrier;

use crate::bank::{Bank, BankError, TransferRequest};
use crate::clock::{Clock, local_day_bounds};
use crate::store::{Deltas, apply_deltas, collect_deltas, insert_entry};

/// Tài khoản tiền đang chi ra ngân hàng.
pub const CLEARING: &str = "bank:payout:clearing";

/// Trạng thái của một khoản chi và của cả lô (cùng bốn tên).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Đã trừ ví, chưa chắc ngân hàng đã nhận lệnh.
    Created,
    /// Ngân hàng đã xác nhận nhận lệnh; chưa biết kết quả xử lý, chờ sao kê.
    Sent,
    /// Sao kê xác nhận tiền đã tới.
    Settled,
    /// Thất bại: đã có bút toán đảo trả tiền về ví.
    Failed,
}

impl Status {
    /// Tên lưu trong DB.
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Created => "CREATED",
            Status::Sent => "SENT",
            Status::Settled => "SETTLED",
            Status::Failed => "FAILED",
        }
    }

    /// Đọc tên từ DB.
    ///
    /// # Errors
    /// Tên không thuộc bốn trạng thái.
    pub fn parse(raw: &str) -> Result<Status> {
        Ok(match raw {
            "CREATED" => Status::Created,
            "SENT" => Status::Sent,
            "SETTLED" => Status::Settled,
            "FAILED" => Status::Failed,
            other => bail!("trạng thái lạ: {other}"),
        })
    }

    /// Đường đi hợp lệ của một khoản chi. `Settled` và `Failed` là điểm cuối.
    pub fn can_go_to(self, next: Status) -> bool {
        matches!(
            (self, next),
            (Status::Created, Status::Sent | Status::Failed)
                | (Status::Sent, Status::Settled | Status::Failed)
        )
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.as_str())
    }
}

/// Số khoản chi theo trạng thái trong một lô.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    /// Số khoản `CREATED`.
    pub created: i64,
    /// Số khoản `SENT`.
    pub sent: i64,
    /// Số khoản `SETTLED`.
    pub settled: i64,
    /// Số khoản `FAILED`.
    pub failed: i64,
}

/// Trạng thái lô suy ra từ trạng thái các khoản: còn khoản chưa gửi thì lô vẫn `CREATED`,
/// còn khoản chưa có kết cục thì `SENT`, mọi khoản đã có kết cục thì `SETTLED`,
/// riêng lô mà mọi khoản đều thất bại là `FAILED`.
pub fn derive_batch_status(c: Counts) -> Status {
    if c.created > 0 {
        Status::Created
    } else if c.sent > 0 {
        Status::Sent
    } else if c.settled == 0 && c.failed > 0 {
        Status::Failed
    } else {
        Status::Settled
    }
}

/// Chính sách chọn ví và giới hạn rút.
#[derive(Debug, Clone, Copy)]
pub struct Policy {
    /// Ví phải có ít nhất số này mới vào lô, và khoản chi cũng không được nhỏ hơn.
    pub min_payout: Money,
    /// Tổng tối đa một tài xế được rút trong một ngày lịch giờ Việt Nam.
    pub daily_limit: Money,
    /// Số ví tối đa một lô lấy.
    pub max_items: i64,
}

impl Policy {
    /// Chính sách của lab: ngưỡng 50.000, hạn mức 150.000 mỗi ngày, tối đa 100 ví một lô.
    pub fn standard() -> Policy {
        Policy {
            min_payout: Money::from_dong(50_000),
            daily_limit: Money::from_dong(150_000),
            max_items: 100,
        }
    }
}

/// Cách chọn ví vào lô.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    /// `FOR UPDATE SKIP LOCKED`: khóa ví đã chọn, bỏ qua ví lô khác đang khóa.
    SkipLocked,
    /// `SELECT` thường: để cho thấy lô song song chọn trùng.
    Naive,
}

const SELECT_LOCKED: &str = "SELECT substr(account_id, 8) AS driver_id, balance_dong \
     FROM account_balances \
     WHERE account_id LIKE 'driver:%' AND balance_dong >= $1 \
     ORDER BY account_id LIMIT $2 FOR UPDATE SKIP LOCKED";

const SELECT_PLAIN: &str = "SELECT substr(account_id, 8) AS driver_id, balance_dong \
     FROM account_balances \
     WHERE account_id LIKE 'driver:%' AND balance_dong >= $1 \
     ORDER BY account_id LIMIT $2";

/// Yêu cầu tạo một lô.
#[derive(Debug, Clone)]
pub struct BatchRequest {
    /// Mã lô, duy nhất: chạy lại cùng mã là cùng một lô.
    pub code: String,
    /// Kỳ chi, ví dụ `2026-11-06.1`. Mỗi tài xế một khoản còn hiệu lực trong một kỳ.
    pub period: String,
    /// Chính sách chọn ví.
    pub policy: Policy,
    /// Cách chọn ví.
    pub selection: Selection,
}

/// Ví bị bỏ qua và lý do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skipped {
    /// Tài xế.
    pub driver_id: String,
    /// Số dư ví lúc xét.
    pub balance_dong: i64,
    /// Hạn mức còn lại trong ngày lúc xét.
    pub remaining_limit_dong: i64,
}

/// Kết quả `create_batch`.
#[derive(Debug, Clone)]
pub struct Created {
    /// Id lô trong DB.
    pub batch_id: i64,
    /// `true` nếu lô vừa được tạo, `false` nếu mã lô đã có từ trước.
    pub fresh: bool,
    /// Các khoản chi (tài xế, số tiền) theo thứ tự tạo.
    pub items: Vec<(String, i64)>,
    /// Ví đủ ngưỡng nhưng hạn mức trong ngày không còn đủ.
    pub skipped: Vec<Skipped>,
}

/// Mã tham chiếu và `idempotency_key` của khoản chi: một lô, một tài xế, đúng một mã.
pub fn payout_key(batch_code: &str, driver_id: &str) -> String {
    format!("payout:{batch_code}:{driver_id}")
}

fn account(raw: &str) -> Result<AccountId> {
    Ok(AccountId::new(raw)?)
}

/// Bút toán của khoản chi: ví tài xế âm, clearing chi ngân hàng dương.
///
/// # Errors
/// Mã tài khoản hoặc bút toán không hợp lệ.
pub fn payout_entry(key: &str, driver_id: &str, amount: i64) -> Result<JournalEntry> {
    Ok(JournalEntryBuilder::new(key)
        .posting(
            account(&format!("driver:{driver_id}"))?,
            Money::from_dong(-amount),
        )
        .posting(account(CLEARING)?, Money::from_dong(amount))
        .build()?)
}

/// Tổng tiền tài xế đã được tạo khoản chi (không tính khoản thất bại) trong `[from, to)`.
async fn used_in_window(
    conn: &mut PgConnection,
    driver_id: &str,
    from: Timestamp,
    to: Timestamp,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "SELECT COALESCE(SUM(amount_dong), 0)::bigint AS used FROM payouts \
         WHERE driver_id = $1 AND status <> 'FAILED' \
           AND created_at >= $2::timestamptz AND created_at < $3::timestamptz",
    )
    .bind(driver_id)
    .bind(from.to_string())
    .bind(to.to_string())
    .fetch_one(conn)
    .await?;
    Ok(row.get("used"))
}

/// Hạn mức còn lại của `driver_id` trong ngày lịch (giờ Việt Nam) chứa `now`.
///
/// # Errors
/// Lỗi SQL hoặc không có múi giờ.
pub async fn remaining_limit(
    pool: &PgPool,
    policy: &Policy,
    driver_id: &str,
    now: Timestamp,
) -> Result<i64> {
    let (from, to) = local_day_bounds(now)?;
    let mut conn = pool.acquire().await?;
    let used = used_in_window(&mut conn, driver_id, from, to).await?;
    Ok((policy.daily_limit.dong() - used).max(0))
}

/// Tạo một lô trong MỘT transaction: đăng ký mã lô, chọn ví, ghi bút toán và khoản chi.
/// `pause` chỉ để thí nghiệm: hai lô cùng chờ ở hàng rào sau bước chọn, trước bước ghi.
///
/// # Errors
/// Lỗi SQL (ví dụ 23505 khi vi phạm unique theo kỳ), lỗi múi giờ hoặc dựng bút toán.
pub async fn create_batch(
    pool: &PgPool,
    clock: &impl Clock,
    request: &BatchRequest,
    pause: Option<&Barrier>,
) -> Result<Created> {
    let now = clock.now();
    let mut tx = pool.begin().await?;

    // Bước 1: đăng ký mã lô. Hai lệnh cùng mã: lệnh sau chờ lệnh trước commit rồi thấy conflict.
    let inserted = sqlx::query(
        "INSERT INTO payout_batches (batch_code, period, status, created_at) \
         VALUES ($1, $2, 'CREATED', $3::timestamptz) \
         ON CONFLICT (batch_code) DO NOTHING RETURNING id",
    )
    .bind(&request.code)
    .bind(&request.period)
    .bind(now.to_string())
    .fetch_optional(&mut *tx)
    .await?;
    let Some(inserted) = inserted else {
        tx.rollback().await?;
        return existing_batch(pool, &request.code).await;
    };
    let batch_id: i64 = inserted.get("id");

    // Bước 2: chọn ví. Với SkipLocked, hàng được chọn bị khóa tới khi commit.
    let sql = match request.selection {
        Selection::SkipLocked => SELECT_LOCKED,
        Selection::Naive => SELECT_PLAIN,
    };
    let candidates = sqlx::query(sql)
        .bind(request.policy.min_payout.dong())
        .bind(request.policy.max_items)
        .fetch_all(&mut *tx)
        .await?;
    if let Some(barrier) = pause {
        barrier.wait().await;
    }

    // Bước 3: tính khoản chi theo hạn mức, ghi bút toán và khoản chi.
    let (day_start, day_end) = local_day_bounds(now)?;
    let mut deltas = Deltas::new();
    let mut items = Vec::new();
    let mut skipped = Vec::new();
    for row in &candidates {
        let driver_id: String = row.get("driver_id");
        let balance: i64 = row.get("balance_dong");
        let used = used_in_window(&mut tx, &driver_id, day_start, day_end).await?;
        let remaining = (request.policy.daily_limit.dong() - used).max(0);
        let amount = balance.min(remaining);
        if amount < request.policy.min_payout.dong() {
            skipped.push(Skipped {
                driver_id,
                balance_dong: balance,
                remaining_limit_dong: remaining,
            });
            continue;
        }
        let key = payout_key(&request.code, &driver_id);
        let entry = payout_entry(&key, &driver_id, amount)?;
        let entry_id = insert_entry(&mut tx, &entry, now, None)
            .await?
            .ok_or_else(|| anyhow!("key {key} đã có bút toán"))?;
        sqlx::query(
            "INSERT INTO payouts (batch_id, period, driver_id, amount_dong, status, \
                                  payout_key, entry_id, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, 'CREATED', $5, $6, $7::timestamptz, $7::timestamptz)",
        )
        .bind(batch_id)
        .bind(&request.period)
        .bind(&driver_id)
        .bind(amount)
        .bind(&key)
        .bind(entry_id)
        .bind(now.to_string())
        .execute(&mut *tx)
        .await?;
        collect_deltas(&mut deltas, &entry)?;
        items.push((driver_id, amount));
    }
    // Số dư cập nhật một lần cho mỗi tài khoản, cuối transaction: hàng clearing dùng chung
    // chỉ bị giữ khóa trong khoảng ngắn nhất.
    apply_deltas(&mut tx, &deltas, now).await?;
    refresh_batch_status(&mut tx, batch_id).await?;
    tx.commit().await?;
    Ok(Created {
        batch_id,
        fresh: true,
        items,
        skipped,
    })
}

/// Lô đã có từ trước: trả đúng nội dung đã ghi, không tạo gì thêm.
async fn existing_batch(pool: &PgPool, code: &str) -> Result<Created> {
    let batch = sqlx::query("SELECT id FROM payout_batches WHERE batch_code = $1")
        .bind(code)
        .fetch_one(pool)
        .await?;
    let batch_id: i64 = batch.get("id");
    let items = load_payouts(pool, batch_id)
        .await?
        .into_iter()
        .map(|p| (p.driver_id, p.amount_dong))
        .collect();
    Ok(Created {
        batch_id,
        fresh: false,
        items,
        skipped: Vec::new(),
    })
}

/// Một khoản chi đọc từ DB.
#[derive(Debug, Clone)]
pub struct PayoutRow {
    /// Id khoản chi.
    pub id: i64,
    /// Tài xế, ví dụ `DR-001`.
    pub driver_id: String,
    /// Số tiền chi.
    pub amount_dong: i64,
    /// Trạng thái hiện tại.
    pub status: Status,
    /// Mã tham chiếu (cũng là key của bút toán).
    pub payout_key: String,
    /// Mã giao dịch ngân hàng, có sau khi ngân hàng xác nhận.
    pub bank_ref: Option<String>,
    /// Số lần đã gửi lệnh sang ngân hàng.
    pub attempts: i32,
    /// Lỗi gần nhất.
    pub last_error: Option<String>,
    /// Bút toán trừ ví.
    pub entry_id: i64,
    /// Bút toán đảo, có khi thất bại.
    pub reversal_entry_id: Option<i64>,
}

fn payout_from_row(row: &PgRow) -> Result<PayoutRow> {
    Ok(PayoutRow {
        id: row.get("id"),
        driver_id: row.get("driver_id"),
        amount_dong: row.get("amount_dong"),
        status: Status::parse(row.get("status"))?,
        payout_key: row.get("payout_key"),
        bank_ref: row.get("bank_ref"),
        attempts: row.get("attempts"),
        last_error: row.get("last_error"),
        entry_id: row.get("entry_id"),
        reversal_entry_id: row.get("reversal_entry_id"),
    })
}

/// Mọi khoản chi của một lô, theo thứ tự tạo.
///
/// # Errors
/// Lỗi SQL.
pub async fn load_payouts(pool: &PgPool, batch_id: i64) -> Result<Vec<PayoutRow>> {
    let rows = sqlx::query("SELECT * FROM payouts WHERE batch_id = $1 ORDER BY id")
        .bind(batch_id)
        .fetch_all(pool)
        .await?;
    rows.iter().map(payout_from_row).collect()
}

/// Trạng thái hiện tại của một lô.
///
/// # Errors
/// Lỗi SQL.
pub async fn batch_status(pool: &PgPool, batch_id: i64) -> Result<Status> {
    let row = sqlx::query("SELECT status FROM payout_batches WHERE id = $1")
        .bind(batch_id)
        .fetch_one(pool)
        .await?;
    Status::parse(row.get("status"))
}

/// Tính lại trạng thái lô từ các khoản chi. Khóa hàng lô trước khi đếm, nên hai transaction
/// cùng kết thúc khoản chi của một lô đếm lần lượt, không đếm trên dữ liệu cũ của nhau.
///
/// # Errors
/// Lỗi SQL.
pub async fn refresh_batch_status(conn: &mut PgConnection, batch_id: i64) -> Result<Status> {
    sqlx::query("SELECT id FROM payout_batches WHERE id = $1 FOR UPDATE")
        .bind(batch_id)
        .fetch_one(&mut *conn)
        .await?;
    let rows = sqlx::query(
        "SELECT status, count(*) AS n FROM payouts WHERE batch_id = $1 GROUP BY status",
    )
    .bind(batch_id)
    .fetch_all(&mut *conn)
    .await?;
    let mut counts = Counts::default();
    for row in &rows {
        let n: i64 = row.get("n");
        match Status::parse(row.get("status"))? {
            Status::Created => counts.created = n,
            Status::Sent => counts.sent = n,
            Status::Settled => counts.settled = n,
            Status::Failed => counts.failed = n,
        }
    }
    let status = derive_batch_status(counts);
    sqlx::query("UPDATE payout_batches SET status = $2 WHERE id = $1")
        .bind(batch_id)
        .bind(status.as_str())
        .execute(&mut *conn)
        .await?;
    Ok(status)
}

/// Chuyển một khoản chi sang trạng thái kế tiếp bằng compare-and-set: chỉ đổi nếu hiện vẫn
/// đang ở `from`. `Ok(false)`: ai đó đã đổi trước. Không dùng cho `Failed` (cần bút toán đảo).
///
/// # Errors
/// Chuyển trạng thái không hợp lệ hoặc lỗi SQL.
pub async fn advance(
    pool: &PgPool,
    now: Timestamp,
    payout_id: i64,
    from: Status,
    to: Status,
    bank_ref: Option<&str>,
) -> Result<bool> {
    ensure!(from.can_go_to(to), "không đi được từ {from} sang {to}");
    ensure!(
        to != Status::Failed,
        "FAILED phải đi qua fail_payout để có bút toán đảo"
    );
    let done = sqlx::query(
        "UPDATE payouts SET status = $3, bank_ref = COALESCE($4, bank_ref), \
                updated_at = $5::timestamptz \
          WHERE id = $1 AND status = $2",
    )
    .bind(payout_id)
    .bind(from.as_str())
    .bind(to.as_str())
    .bind(bank_ref)
    .bind(now.to_string())
    .execute(pool)
    .await?
    .rows_affected();
    Ok(done == 1)
}

/// Đánh dấu thất bại và ghi bút toán đảo trả tiền về ví, cùng một transaction.
/// `Ok(false)`: khoản này đã kết thúc từ trước (không đảo lần hai).
///
/// # Errors
/// Lỗi SQL hoặc dựng bút toán.
pub async fn fail_payout(
    pool: &PgPool,
    now: Timestamp,
    payout_id: i64,
    reason: &str,
) -> Result<bool> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query(
        "UPDATE payouts SET status = 'FAILED', last_error = $2, updated_at = $3::timestamptz \
          WHERE id = $1 AND status IN ('CREATED', 'SENT') \
          RETURNING batch_id, driver_id, amount_dong, payout_key, entry_id",
    )
    .bind(payout_id)
    .bind(reason)
    .bind(now.to_string())
    .fetch_optional(&mut *tx)
    .await?;
    let Some(row) = row else {
        tx.rollback().await?;
        return Ok(false);
    };
    let batch_id: i64 = row.get("batch_id");
    let driver_id: String = row.get("driver_id");
    let amount: i64 = row.get("amount_dong");
    let key: String = row.get("payout_key");
    let entry_id: i64 = row.get("entry_id");

    // Bút toán gốc dựng lại từ khoản chi; reversal_of lật dấu và đặt key `<key>:reversal`.
    let original = payout_entry(&key, &driver_id, amount)?;
    let original_id = EntryId::parse(&format!("JE-{entry_id:06}"))
        .ok_or_else(|| anyhow!("id bút toán {entry_id} không đổi được sang JE-"))?;
    let reversal = JournalEntry::reversal_of(original_id, &original)?;
    let reversal_id = insert_entry(&mut tx, &reversal, now, Some(entry_id))
        .await?
        .ok_or_else(|| anyhow!("bút toán đảo của {key} đã có"))?;
    let mut deltas = Deltas::new();
    collect_deltas(&mut deltas, &reversal)?;
    apply_deltas(&mut tx, &deltas, now).await?;
    sqlx::query("UPDATE payouts SET reversal_entry_id = $2 WHERE id = $1")
        .bind(payout_id)
        .bind(reversal_id)
        .execute(&mut *tx)
        .await?;
    refresh_batch_status(&mut tx, batch_id).await?;
    tx.commit().await?;
    Ok(true)
}

/// Kết quả một lượt gửi lô sang ngân hàng.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SendReport {
    /// Khoản được ngân hàng xác nhận (`SENT`).
    pub sent: usize,
    /// Khoản bị từ chối (`FAILED`, đã đảo).
    pub rejected: usize,
    /// Khoản timeout: vẫn `CREATED`, chưa biết ngân hàng đã nhận hay chưa.
    pub timed_out: usize,
}

/// Gửi mọi khoản `CREATED` của lô sang ngân hàng bằng đúng `payout_key` làm mã tham chiếu.
/// Gọi lại bao nhiêu lần cũng được: khoản đã `SENT` không bị gửi lại. Khoản từng gửi mà chưa rõ
/// kết cục (timeout, hoặc tiến trình chết giữa chừng) phải tra cứu trước; chỉ khi ngân hàng trả
/// lời chưa có lệnh mới gửi lại, và vẫn đúng mã cũ.
///
/// # Errors
/// Lỗi SQL hoặc lỗi hạ tầng không phải kết quả nghiệp vụ của ngân hàng.
pub async fn send_batch<B: Bank>(
    pool: &PgPool,
    bank: &B,
    clock: &impl Clock,
    batch_id: i64,
) -> Result<SendReport> {
    let mut report = SendReport::default();
    for payout in load_payouts(pool, batch_id).await? {
        if payout.status != Status::Created {
            continue;
        }
        if payout.attempts > 0 {
            match bank.lookup(&payout.payout_key).await {
                Ok(Some(found)) => {
                    let done = advance(
                        pool,
                        clock.now(),
                        payout.id,
                        Status::Created,
                        Status::Sent,
                        Some(&found.bank_ref),
                    )
                    .await?;
                    if done {
                        report.sent += 1;
                    }
                    continue;
                }
                Ok(None) => {} // ngân hàng trả lời chưa có lệnh: gửi lại đúng mã cũ
                Err(_) => {
                    // Tra cứu cũng không có câu trả lời: vẫn không biết, không gửi.
                    report.timed_out += 1;
                    continue;
                }
            }
        }
        // Ghi nhận lần thử trước khi gọi: chết giữa chừng vẫn thấy đã thử.
        sqlx::query("UPDATE payouts SET attempts = attempts + 1 WHERE id = $1")
            .bind(payout.id)
            .execute(pool)
            .await?;
        let request = TransferRequest {
            reference: payout.payout_key.clone(),
            beneficiary: payout.driver_id.clone(),
            amount: Money::from_dong(payout.amount_dong),
        };
        match bank.transfer(&request).await {
            Ok(transfer) => {
                let now = clock.now();
                if advance(
                    pool,
                    now,
                    payout.id,
                    Status::Created,
                    Status::Sent,
                    Some(&transfer.bank_ref),
                )
                .await?
                {
                    report.sent += 1;
                }
            }
            Err(BankError::Rejected(reason)) => {
                if fail_payout(pool, clock.now(), payout.id, &reason).await? {
                    report.rejected += 1;
                }
            }
            Err(BankError::Timeout) => {
                sqlx::query("UPDATE payouts SET last_error = 'timeout' WHERE id = $1")
                    .bind(payout.id)
                    .execute(pool)
                    .await?;
                report.timed_out += 1;
            }
        }
    }
    let mut conn = pool.begin().await?;
    refresh_batch_status(&mut conn, batch_id).await?;
    conn.commit().await?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transitions_only_go_forward() {
        assert!(Status::Created.can_go_to(Status::Sent));
        assert!(Status::Created.can_go_to(Status::Failed));
        assert!(Status::Sent.can_go_to(Status::Settled));
        assert!(Status::Sent.can_go_to(Status::Failed));
        assert!(!Status::Created.can_go_to(Status::Settled));
        assert!(!Status::Settled.can_go_to(Status::Failed));
        assert!(!Status::Failed.can_go_to(Status::Created));
        assert!(!Status::Sent.can_go_to(Status::Created));
    }

    #[test]
    fn batch_status_follows_the_slowest_payout() {
        let c = |created, sent, settled, failed| {
            derive_batch_status(Counts {
                created,
                sent,
                settled,
                failed,
            })
        };
        assert_eq!(c(1, 3, 0, 0), Status::Created);
        assert_eq!(c(0, 2, 1, 1), Status::Sent);
        assert_eq!(c(0, 0, 2, 1), Status::Settled);
        assert_eq!(c(0, 0, 0, 3), Status::Failed);
        assert_eq!(c(0, 0, 0, 0), Status::Settled);
    }

    #[test]
    fn payout_key_is_batch_and_driver() {
        assert_eq!(
            payout_key("PB-2026-11-06.1", "DR-001"),
            "payout:PB-2026-11-06.1:DR-001"
        );
    }
}
