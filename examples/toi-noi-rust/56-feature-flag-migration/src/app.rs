//! Hai phiên bản code ghi sổ chạy trên cùng một database: `AppV1` (code cũ) và `AppV2` (code mới).
//! Cả hai ghi bút toán thu cước của một chuyến theo dạng hai bước (capture rồi chia) và sự kiện
//! quyết định phí. Khác nhau ở cột nào chúng biết tới.

use ledger_core::{
    AccountId, FeeSplit, JournalEntry, LedgerError, Money, Posting, TripCharge, fare_for_km,
};
use sqlx::{PgConnection, PgPool, Row};
use thiserror::Error;

use crate::policy::{PolicyVersion, split_for};

/// Lỗi ghi sổ.
#[derive(Debug, Error)]
pub enum AppError {
    /// Lỗi SQL.
    #[error("lỗi SQL: {0}")]
    Db(#[from] sqlx::Error),
    /// Bút toán dựng sai (không cân, rỗng...).
    #[error("lỗi sổ cái: {0}")]
    Ledger(#[from] LedgerError),
    /// Cùng `idempotency_key` nhưng nội dung khác: không thể "ghi lại" mà phải báo.
    #[error("khóa `{key}` đã có một bút toán khác nội dung")]
    IdempotencyConflict {
        /// Khóa idempotency bị trùng.
        key: String,
    },
    /// DB có `policy_version` mà code không biết.
    #[error("policy_version {0} trong DB không có trong enum PolicyVersion")]
    UnknownVersion(i32),
}

/// Mã SQLSTATE của lỗi DB, nếu có (ví dụ `23502` là vi phạm NOT NULL).
pub fn sqlstate(err: &sqlx::Error) -> Option<String> {
    err.as_database_error()
        .and_then(|e| e.code())
        .map(|c| c.to_string())
}

impl AppError {
    /// SQLSTATE của lỗi DB bên trong, nếu đây là lỗi DB.
    pub fn sqlstate(&self) -> Option<String> {
        match self {
            AppError::Db(e) => sqlstate(e),
            _ => None,
        }
    }
}

/// Kết quả ghi một bút toán.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostOutcome {
    /// Bút toán mới.
    Posted(i64),
    /// Khóa đã có, nội dung y hệt: trả bút toán cũ.
    Replayed(i64),
}

impl PostOutcome {
    /// `journal_entries.id`.
    pub fn entry_id(self) -> i64 {
        match self {
            PostOutcome::Posted(id) | PostOutcome::Replayed(id) => id,
        }
    }
}

/// Kết quả ghi một chuyến.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TripOutcome {
    /// Bút toán thu cước.
    pub capture: PostOutcome,
    /// Bút toán chia cước.
    pub split: PostOutcome,
    /// Phiên bản quy tắc phí thực sự dùng.
    pub version: PolicyVersion,
    /// Phí nền tảng.
    pub fee: Money,
}

/// Chuyến thứ `n` của dữ liệu mẫu: khách và tài xế xoay vòng ba người, quãng đường 3 đến 7 km.
/// `TN-0001` là chuyến 5 km, cước 25.500.
pub fn sample_trip(n: u32) -> Result<TripCharge, LedgerError> {
    let who = (n - 1) % 3 + 1;
    trip_km(
        &format!("TN-{n:04}"),
        &format!("customer:CU-{who:03}"),
        &format!("driver:DR-{who:03}"),
        3 + (2 * n) % 5,
    )
}

/// Một chuyến đi `km` km của khách và tài xế cho trước.
///
/// # Errors
/// Mã tài khoản rỗng.
pub fn trip_km(
    trip_id: &str,
    customer: &str,
    driver: &str,
    km: u32,
) -> Result<TripCharge, LedgerError> {
    Ok(TripCharge {
        trip_id: trip_id.to_string(),
        customer: AccountId::new(customer)?,
        driver: AccountId::new(driver)?,
        fare: fare_for_km(km),
    })
}

/// Bước 1: khách trả cước vào clearing của VíXanh.
fn capture_entry(trip: &TripCharge) -> Result<JournalEntry, LedgerError> {
    let charge = trip.fare.checked_neg().ok_or(LedgerError::Overflow)?;
    JournalEntry::try_new(
        &format!("trip:{}:capture", trip.trip_id),
        vec![
            Posting::new(trip.customer.clone(), charge),
            Posting::new(AccountId::new("psp:vixanh:clearing")?, trip.fare),
        ],
    )
}

/// Bước 2: chia từ clearing cho nền tảng và tài xế.
fn split_entry(trip: &TripCharge, split: FeeSplit) -> Result<JournalEntry, LedgerError> {
    let out = trip.fare.checked_neg().ok_or(LedgerError::Overflow)?;
    JournalEntry::try_new(
        &format!("trip:{}:split", trip.trip_id),
        vec![
            Posting::new(AccountId::new("psp:vixanh:clearing")?, out),
            Posting::new(AccountId::new("platform:revenue")?, split.platform_fee),
            Posting::new(trip.driver.clone(), split.driver_net),
        ],
    )
}

/// Ghi một bút toán, idempotent theo khóa. `with_currency` chọn câu INSERT: code cũ không biết
/// cột `currency` (`false`), code mới ghi `'VND'` tường minh (`true`).
async fn post_entry(
    conn: &mut PgConnection,
    entry: &JournalEntry,
    with_currency: bool,
) -> Result<PostOutcome, AppError> {
    let inserted: Option<i64> = sqlx::query_scalar(
        "INSERT INTO journal_entries (idempotency_key) VALUES ($1) \
         ON CONFLICT (idempotency_key) DO NOTHING RETURNING id",
    )
    .bind(entry.idempotency_key())
    .fetch_optional(&mut *conn)
    .await?;

    let Some(id) = inserted else {
        // Khóa đã có: cùng nội dung thì là ghi lại, khác nội dung thì là xung đột.
        let existing: i64 =
            sqlx::query_scalar("SELECT id FROM journal_entries WHERE idempotency_key = $1")
                .bind(entry.idempotency_key())
                .fetch_one(&mut *conn)
                .await?;
        let stored: Vec<(String, i64)> = sqlx::query(
            "SELECT account_id, amount_dong FROM postings WHERE entry_id = $1 ORDER BY line_no",
        )
        .bind(existing)
        .fetch_all(&mut *conn)
        .await?
        .iter()
        .map(|r| (r.get("account_id"), r.get("amount_dong")))
        .collect();
        let wanted: Vec<(String, i64)> = entry
            .postings()
            .iter()
            .map(|p| (p.account.as_str().to_string(), p.amount.dong()))
            .collect();
        return if stored == wanted {
            Ok(PostOutcome::Replayed(existing))
        } else {
            Err(AppError::IdempotencyConflict {
                key: entry.idempotency_key().to_string(),
            })
        };
    };

    for (index, posting) in entry.postings().iter().enumerate() {
        let line_no = i32::try_from(index + 1).expect("bút toán không có hàng tỷ dòng");
        let sql = if with_currency {
            "INSERT INTO postings (entry_id, line_no, account_id, amount_dong, currency) \
             VALUES ($1, $2, $3, $4, 'VND')"
        } else {
            "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
             VALUES ($1, $2, $3, $4)"
        };
        sqlx::query(sql)
            .bind(id)
            .bind(line_no)
            .bind(posting.account.as_str())
            .bind(posting.amount.dong())
            .execute(&mut *conn)
            .await?;
        sqlx::query(
            "INSERT INTO account_balances (account_id, balance_dong, version) VALUES ($1, $2, 1) \
             ON CONFLICT (account_id) DO UPDATE SET \
             balance_dong = account_balances.balance_dong + EXCLUDED.balance_dong, \
             version = account_balances.version + 1, updated_at = now()",
        )
        .bind(posting.account.as_str())
        .bind(posting.amount.dong())
        .execute(&mut *conn)
        .await?;
    }
    Ok(PostOutcome::Posted(id))
}

/// Code cũ: không biết `currency` và `policy_version`. Luôn dùng quy tắc `V1`, ghi `fee_percent`.
pub struct AppV1;

impl AppV1 {
    /// Ghi một chuyến theo cách của code cũ.
    ///
    /// # Errors
    /// Lỗi SQL (ví dụ khi schema đã contract mà code cũ còn chạy), hoặc bút toán không hợp lệ.
    pub async fn record_trip(pool: &PgPool, trip: &TripCharge) -> Result<TripOutcome, AppError> {
        let split = split_for(PolicyVersion::V1, trip.fare)?;
        let mut tx = pool.begin().await?;
        let capture = post_entry(&mut tx, &capture_entry(trip)?, false).await?;
        let split_outcome = post_entry(&mut tx, &split_entry(trip, split)?, false).await?;
        sqlx::query(
            "INSERT INTO fee_events (trip_id, fare_dong, fee_percent) VALUES ($1, $2, $3) \
             ON CONFLICT (trip_id) DO NOTHING",
        )
        .bind(&trip.trip_id)
        .bind(trip.fare.dong())
        .bind(PolicyVersion::V1.nominal_percent())
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(TripOutcome {
            capture,
            split: split_outcome,
            version: PolicyVersion::V1,
            fee: split.platform_fee,
        })
    }

    /// Báo cáo của code cũ: tỷ lệ phí từng chuyến, đọc từ cột `fee_percent`.
    ///
    /// # Errors
    /// Lỗi SQL, ví dụ cột đã bị bỏ.
    pub async fn fee_report(pool: &PgPool) -> Result<Vec<(String, i32)>, AppError> {
        let rows = sqlx::query("SELECT trip_id, fee_percent FROM fee_events ORDER BY seq")
            .fetch_all(pool)
            .await?;
        // `try_get`: cột còn NULL (đường lùi chưa suy lại dữ liệu) là lỗi, không phải panic.
        rows.iter()
            .map(|r| Ok((r.try_get("trip_id")?, r.try_get("fee_percent")?)))
            .collect()
    }
}

/// Giai đoạn của code mới trong cuộc chuyển.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteMode {
    /// Ghi cả cột cũ `fee_percent` (cho code cũ còn chạy) và cột mới `policy_version`.
    Dual,
    /// Chỉ ghi cột mới. Chỉ an toàn khi không còn ai đọc cột cũ và cột cũ đã nới NOT NULL.
    NewOnly,
}

/// Cách chốt phiên bản quy tắc phí của một chuyến.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Đúng: sự kiện đã có thì dùng phiên bản trong sự kiện, bỏ qua cờ hiện tại.
    Frozen,
    /// SAI để đối chứng: hỏi lại cờ mỗi lần và dùng kết quả mới.
    Redecide,
}

/// Code mới: biết `currency` và `policy_version`, đọc cột mới và chịu được dòng cũ chưa backfill.
pub struct AppV2;

impl AppV2 {
    /// Ghi một chuyến. `proposed` là phiên bản do cờ chọn cho lần xử lý này.
    ///
    /// # Errors
    /// Lỗi SQL, xung đột idempotency hoặc bút toán không hợp lệ.
    pub async fn record_trip(
        pool: &PgPool,
        trip: &TripCharge,
        proposed: PolicyVersion,
        mode: WriteMode,
        decision: Decision,
    ) -> Result<TripOutcome, AppError> {
        let mut tx = pool.begin().await?;
        // Chốt quyết định phí trước: sự kiện đầu tiên của chuyến thắng.
        let created: Option<i32> = match mode {
            WriteMode::Dual => {
                sqlx::query_scalar(
                    "INSERT INTO fee_events (trip_id, fare_dong, fee_percent, policy_version) \
                     VALUES ($1, $2, $3, $4) ON CONFLICT (trip_id) DO NOTHING \
                     RETURNING policy_version",
                )
                .bind(&trip.trip_id)
                .bind(trip.fare.dong())
                .bind(proposed.nominal_percent())
                .bind(proposed.number())
                .fetch_optional(&mut *tx)
                .await?
            }
            WriteMode::NewOnly => {
                sqlx::query_scalar(
                    "INSERT INTO fee_events (trip_id, fare_dong, policy_version) \
                     VALUES ($1, $2, $3) ON CONFLICT (trip_id) DO NOTHING \
                     RETURNING policy_version",
                )
                .bind(&trip.trip_id)
                .bind(trip.fare.dong())
                .bind(proposed.number())
                .fetch_optional(&mut *tx)
                .await?
            }
        };
        let version = match (created, decision) {
            (Some(_), _) | (None, Decision::Redecide) => proposed,
            (None, Decision::Frozen) => stored_version(&mut tx, &trip.trip_id).await?,
        };
        let split = split_for(version, trip.fare)?;
        let capture = post_entry(&mut tx, &capture_entry(trip)?, true).await?;
        let split_outcome = post_entry(&mut tx, &split_entry(trip, split)?, true).await?;
        tx.commit().await?;
        Ok(TripOutcome {
            capture,
            split: split_outcome,
            version,
            fee: split.platform_fee,
        })
    }

    /// Báo cáo của code mới: phiên bản quy tắc từng chuyến. Dòng chưa backfill (`NULL`)
    /// được hiểu là `V1`, vì trước khi có cột này chỉ có một quy tắc.
    ///
    /// # Errors
    /// Lỗi SQL hoặc `policy_version` lạ.
    pub async fn fee_report(pool: &PgPool) -> Result<Vec<(String, PolicyVersion)>, AppError> {
        let rows = sqlx::query(
            "SELECT trip_id, COALESCE(policy_version, 1) AS v FROM fee_events ORDER BY seq",
        )
        .fetch_all(pool)
        .await?;
        rows.iter()
            .map(|r| {
                let n: i32 = r.get("v");
                PolicyVersion::from_number(n)
                    .map(|v| (r.get("trip_id"), v))
                    .ok_or(AppError::UnknownVersion(n))
            })
            .collect()
    }
}

/// Phiên bản đã lưu của một chuyến (`NULL` của dòng chưa backfill được hiểu là `V1`).
async fn stored_version(conn: &mut PgConnection, trip_id: &str) -> Result<PolicyVersion, AppError> {
    let n: i32 =
        sqlx::query_scalar("SELECT COALESCE(policy_version, 1) FROM fee_events WHERE trip_id = $1")
            .bind(trip_id)
            .fetch_one(&mut *conn)
            .await?;
    PolicyVersion::from_number(n).ok_or(AppError::UnknownVersion(n))
}

/// Ghi `count` chuyến mẫu bằng code cũ (dữ liệu có sẵn trước khi chuyển).
///
/// # Errors
/// Lỗi ghi sổ.
pub async fn seed_v1(pool: &PgPool, count: u32) -> Result<(), AppError> {
    for n in 1..=count {
        AppV1::record_trip(pool, &sample_trip(n)?).await?;
    }
    Ok(())
}

/// `journal_entries.id` lớn nhất (0 nếu sổ trống).
///
/// # Errors
/// Lỗi SQL.
pub async fn max_entry_id(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COALESCE(max(id), 0) FROM journal_entries")
        .fetch_one(pool)
        .await
}

/// Dấu vân tay (md5) của bốn cột gốc của mọi posting thuộc bút toán có `id <= up_to`.
/// Không gồm `currency`: câu hỏi là "số tiền và tài khoản có đổi không".
///
/// # Errors
/// Lỗi SQL.
pub async fn digest_up_to(pool: &PgPool, up_to: i64) -> Result<String, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT COALESCE(md5(string_agg(\
            entry_id::text || ':' || line_no::text || ':' || account_id || ':' || amount_dong::text, \
            ',' ORDER BY entry_id, line_no)), '') \
         FROM postings WHERE entry_id <= $1",
    )
    .bind(up_to)
    .fetch_one(pool)
    .await
}

/// Số posting hoặc sự kiện còn `NULL` ở cột mới (đếm cho `postings.currency` và
/// `fee_events.policy_version`).
///
/// # Errors
/// Lỗi SQL.
pub async fn count_nulls(pool: &PgPool, table: &str, column: &str) -> Result<i64, sqlx::Error> {
    // Chỉ gọi với hằng của lab, không với dữ liệu ngoài.
    let sql = format!("SELECT count(*) FROM {table} WHERE {column} IS NULL");
    sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .fetch_one(pool)
        .await
}

/// Số dư `account_balances` có khớp tổng posting tính trực tiếp không (hai bảng không lệch nhau).
///
/// # Errors
/// Lỗi SQL.
pub async fn balances_match_postings(pool: &PgPool) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT NOT EXISTS ( \
           SELECT 1 FROM account_balances b \
           FULL JOIN (SELECT account_id, sum(amount_dong)::bigint AS s FROM postings GROUP BY 1) p \
             ON p.account_id = b.account_id \
           WHERE COALESCE(b.balance_dong, 0) <> COALESCE(p.s, 0))",
    )
    .fetch_one(pool)
    .await
}

/// Số phiên bản `policy_version` hiện có và số trong từng phiên bản, để in ra.
///
/// # Errors
/// Lỗi SQL.
pub async fn event_versions(pool: &PgPool) -> Result<Vec<(i32, i64)>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT COALESCE(policy_version, 0) AS v, count(*) AS n FROM fee_events GROUP BY 1 ORDER BY 1",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.iter().map(|r| (r.get("v"), r.get("n"))).collect())
}
