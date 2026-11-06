//! Lưu webhook và ghi bút toán. Hai cửa vào: webhook của VíXanh và `TripCompleted` nội bộ.
//! Cả hai nối nhau qua `trip_id`, và có thể đến theo thứ tự nào cũng được.

use anyhow::{Context, Result};
use ledger_core::{AccountId, JournalEntry, Money, TripCharge, capture_entry, default_policy};
use serde::Deserialize;
use sqlx::{PgConnection, PgPool, Row};

/// Webhook đã qua xác thực chữ ký, rút gọn còn phần sổ cái cần.
#[derive(Debug, Clone)]
pub struct Webhook {
    /// Mã giao dịch phía VíXanh; duy nhất cho mỗi lần thu tiền.
    pub external_ref: String,
    /// Chuyến được thanh toán.
    pub trip_id: String,
    /// Số tiền VíXanh báo đã thu, đơn vị đồng.
    pub amount_dong: i64,
}

/// Event nội bộ `TripCompleted` (JSON camelCase, `schemaVersion: 1`), rút gọn các trường bài cần.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripCompleted {
    /// Phiên bản schema, hiện là 1.
    pub schema_version: u32,
    /// Mã chuyến.
    pub trip_id: String,
    /// Mã khách, ví dụ `CU-001`.
    pub customer_id: String,
    /// Mã tài xế, ví dụ `DR-001`.
    pub driver_id: String,
    /// Cước phải thu, đơn vị đồng.
    pub fare_dong: i64,
}

/// Kết quả xử lý một webhook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Đã ghi bút toán `entry_id`.
    Applied {
        /// Id bút toán vừa ghi.
        entry_id: i64,
    },
    /// Chưa có payment cho chuyến: webhook được lưu, chờ event nội bộ.
    Pending,
    /// `external_ref` đã xử lý trước đó: không làm gì thêm.
    Duplicate,
    /// Số tiền webhook khác cước đã biết: lưu để đối soát, không ghi sổ.
    Mismatch,
    /// Chuyến đã có bút toán thu tiền từ `external_ref` khác: VíXanh có thể đã thu hai lần.
    DoubleCapture,
}

impl Outcome {
    /// Nhãn trả cho VíXanh trong body JSON.
    pub fn label(self) -> &'static str {
        match self {
            Outcome::Applied { .. } => "applied",
            Outcome::Pending => "pending",
            Outcome::Duplicate => "duplicate",
            Outcome::Mismatch => "mismatch",
            Outcome::DoubleCapture => "double_capture",
        }
    }
}

struct Payment {
    customer: String,
    driver: String,
    fare_dong: i64,
}

// Khóa advisory theo chuyến, tự nhả khi transaction kết thúc. Nó xếp hàng hai cửa vào của CÙNG
// một chuyến để bên đến sau luôn thấy bản ghi đã commit của bên đến trước.
async fn lock_trip(conn: &mut PgConnection, trip_id: &str) -> Result<()> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(trip_id)
        .execute(conn)
        .await?;
    Ok(())
}

/// Xử lý một webhook đã xác thực. Lỗi hạ tầng trả `Err` và transaction tự rollback:
/// khi đó không còn dấu vết nào, nên VíXanh gửi lại là xử lý lại từ đầu.
///
/// # Errors
/// Lỗi cơ sở dữ liệu hoặc dựng bút toán.
pub async fn receive_webhook(pool: &PgPool, hook: &Webhook) -> Result<Outcome> {
    let mut tx = pool.begin().await?;

    // Chốt «chỉ một lần»: khóa chính external_ref. Hai bản đến cùng lúc thì bản sau chờ
    // bản trước commit, rồi DO NOTHING trả 0 dòng.
    let inserted = sqlx::query(
        "INSERT INTO psp_webhooks (external_ref, trip_id, amount_dong, status) \
         VALUES ($1, $2, $3, 'pending') \
         ON CONFLICT (external_ref) DO NOTHING RETURNING external_ref",
    )
    .bind(&hook.external_ref)
    .bind(&hook.trip_id)
    .bind(hook.amount_dong)
    .fetch_optional(&mut *tx)
    .await?;
    if inserted.is_none() {
        return Ok(Outcome::Duplicate); // drop tx: không có gì để commit
    }

    lock_trip(&mut tx, &hook.trip_id).await?;
    let payment = find_payment(&mut tx, &hook.trip_id).await?;
    let outcome = match payment {
        // Webhook đến TRƯỚC event: giữ dòng 'pending', on_trip_completed sẽ nhặt.
        None => Outcome::Pending,
        Some(p) => settle(&mut tx, hook, &p).await?,
    };
    tx.commit().await?;
    Ok(outcome)
}

/// Xử lý `TripCompleted`: ghi payment, rồi nhặt webhook đã đến trước (nếu có).
/// Trả về kết quả của từng webhook được nhặt.
///
/// # Errors
/// Lỗi cơ sở dữ liệu hoặc dựng bút toán.
pub async fn on_trip_completed(pool: &PgPool, event: &TripCompleted) -> Result<Vec<Outcome>> {
    let mut tx = pool.begin().await?;
    lock_trip(&mut tx, &event.trip_id).await?;

    let inserted = sqlx::query(
        "INSERT INTO payments (trip_id, customer_id, driver_id, fare_dong) \
         VALUES ($1, $2, $3, $4) ON CONFLICT (trip_id) DO NOTHING RETURNING trip_id",
    )
    .bind(&event.trip_id)
    .bind(&event.customer_id)
    .bind(&event.driver_id)
    .bind(event.fare_dong)
    .fetch_optional(&mut *tx)
    .await?;
    if inserted.is_none() {
        return Ok(Vec::new()); // event giao lặp (at-least-once): đã xử lý rồi
    }

    let payment = Payment {
        customer: event.customer_id.clone(),
        driver: event.driver_id.clone(),
        fare_dong: event.fare_dong,
    };
    let waiting = sqlx::query(
        "SELECT external_ref, amount_dong FROM psp_webhooks \
         WHERE trip_id = $1 AND status = 'pending' ORDER BY received_at, external_ref FOR UPDATE",
    )
    .bind(&event.trip_id)
    .fetch_all(&mut *tx)
    .await?;
    let mut outcomes = Vec::new();
    for row in waiting {
        let hook = Webhook {
            external_ref: row.get("external_ref"),
            trip_id: event.trip_id.clone(),
            amount_dong: row.get("amount_dong"),
        };
        outcomes.push(settle(&mut tx, &hook, &payment).await?);
    }
    tx.commit().await?;
    Ok(outcomes)
}

async fn find_payment(conn: &mut PgConnection, trip_id: &str) -> Result<Option<Payment>> {
    let row =
        sqlx::query("SELECT customer_id, driver_id, fare_dong FROM payments WHERE trip_id = $1")
            .bind(trip_id)
            .fetch_optional(conn)
            .await?;
    Ok(row.map(|r| Payment {
        customer: r.get("customer_id"),
        driver: r.get("driver_id"),
        fare_dong: r.get("fare_dong"),
    }))
}

// Đối chiếu số tiền rồi ghi bút toán thu cước và đánh dấu webhook. Chạy trong transaction của người gọi.
async fn settle(conn: &mut PgConnection, hook: &Webhook, payment: &Payment) -> Result<Outcome> {
    if hook.amount_dong != payment.fare_dong {
        mark(conn, &hook.external_ref, "mismatch", None).await?;
        return Ok(Outcome::Mismatch);
    }
    let charge = TripCharge {
        trip_id: hook.trip_id.clone(),
        customer: AccountId::new(&format!("customer:{}", payment.customer))?,
        driver: AccountId::new(&format!("driver:{}", payment.driver))?,
        fare: Money::from_dong(payment.fare_dong),
    };
    let entry = capture_entry(&charge, &default_policy()).context("dựng bút toán thu cước")?;
    match insert_entry(conn, &entry).await? {
        Some(entry_id) => {
            mark(conn, &hook.external_ref, "applied", Some(entry_id)).await?;
            Ok(Outcome::Applied { entry_id })
        }
        None => {
            // idempotency_key `trip:<id>:capture` đã có, nhưng từ một external_ref khác.
            mark(conn, &hook.external_ref, "double_capture", None).await?;
            Ok(Outcome::DoubleCapture)
        }
    }
}

async fn mark(
    conn: &mut PgConnection,
    external_ref: &str,
    status: &str,
    entry_id: Option<i64>,
) -> Result<()> {
    sqlx::query(
        "UPDATE psp_webhooks SET status = $2, entry_id = $3, applied_at = now() WHERE external_ref = $1",
    )
    .bind(external_ref)
    .bind(status)
    .bind(entry_id)
    .execute(conn)
    .await?;
    Ok(())
}

// Giống bài 24: INSERT ... ON CONFLICT DO NOTHING RETURNING id; None nghĩa là key đã có.
async fn insert_entry(conn: &mut PgConnection, entry: &JournalEntry) -> Result<Option<i64>> {
    let inserted = sqlx::query(
        "INSERT INTO journal_entries (idempotency_key) VALUES ($1) \
         ON CONFLICT (idempotency_key) DO NOTHING RETURNING id",
    )
    .bind(entry.idempotency_key())
    .fetch_optional(&mut *conn)
    .await?;
    let Some(row) = inserted else {
        return Ok(None);
    };
    let id: i64 = row.get("id");
    for (index, posting) in entry.postings().iter().enumerate() {
        let line_no = i32::try_from(index + 1).expect("bút toán không có hàng tỷ dòng");
        sqlx::query(
            "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(line_no)
        .bind(posting.account.as_str())
        .bind(posting.amount.dong())
        .execute(&mut *conn)
        .await?;
    }
    Ok(Some(id))
}

/// Số bút toán trong sổ (để lab kiểm).
///
/// # Errors
/// Lỗi cơ sở dữ liệu.
pub async fn count_entries(pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query_scalar("SELECT count(*) FROM journal_entries")
        .fetch_one(pool)
        .await?)
}

/// Số webhook theo trạng thái (`None` là mọi trạng thái).
///
/// # Errors
/// Lỗi cơ sở dữ liệu.
pub async fn count_webhooks(pool: &PgPool, status: Option<&str>) -> Result<i64> {
    Ok(sqlx::query_scalar(
        "SELECT count(*) FROM psp_webhooks WHERE $1::text IS NULL OR status = $1",
    )
    .bind(status)
    .fetch_one(pool)
    .await?)
}

/// Số dư của một tài khoản = tổng posting (dương vào, âm ra).
///
/// # Errors
/// Lỗi cơ sở dữ liệu.
pub async fn balance(pool: &PgPool, account: &str) -> Result<i64> {
    Ok(sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_dong), 0)::bigint FROM postings WHERE account_id = $1",
    )
    .bind(account)
    .fetch_one(pool)
    .await?)
}
