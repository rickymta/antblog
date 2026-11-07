//! Ghi bút toán xuống PostgreSQL: bút toán thường, bút toán đảo, và thu cước hai bước của chuyến.

use std::collections::BTreeMap;

use ledger_core::{
    AccountId, EntryId, FeePolicy, JournalEntry, LedgerError, Money, Posting, default_policy,
};
use sqlx::{PgConnection, PgPool, Row};

use crate::error::RefundError;

/// Kết quả ghi một bút toán.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posted {
    /// Mã bút toán (identity).
    pub id: i64,
    /// `true` nếu key đã có và nội dung trùng: không ghi gì thêm.
    pub replayed: bool,
}

impl Posted {
    /// Mã hiển thị `JE-000001`.
    pub fn display_id(&self) -> String {
        format!("JE-{:06}", self.id)
    }
}

fn lines(entry: &JournalEntry) -> Vec<(String, i64)> {
    entry
        .postings()
        .iter()
        .map(|p| (p.account.as_str().to_string(), p.amount.dong()))
        .collect()
}

/// Ghi bút toán trong transaction của người gọi: header, posting, rồi `account_balances`.
/// Trùng `idempotency_key`: cùng nội dung là replay, khác nội dung là `IdempotencyConflict`.
/// Đảo một bút toán đã đảo vi phạm chỉ mục duy nhất `journal_entries_one_reversal`
/// và được dịch thành `AlreadyReversed`.
pub async fn post_entry(
    tx: &mut PgConnection,
    entry: &JournalEntry,
) -> Result<Posted, RefundError> {
    let reversal_of = entry
        .reverses()
        .and_then(|id| id.to_string()[3..].parse::<i64>().ok());
    let inserted: Result<Option<i64>, sqlx::Error> = sqlx::query_scalar(
        "insert into journal_entries (idempotency_key, reversal_of) values ($1, $2) \
         on conflict (idempotency_key) do nothing returning id",
    )
    .bind(entry.idempotency_key())
    .bind(reversal_of)
    .fetch_optional(&mut *tx)
    .await;
    let inserted = match inserted {
        Ok(v) => v,
        Err(e)
            if e.as_database_error()
                .is_some_and(|d| d.constraint() == Some("journal_entries_one_reversal")) =>
        {
            return Err(RefundError::AlreadyReversed(
                reversal_of.unwrap_or_default(),
            ));
        }
        Err(e) => return Err(e.into()),
    };

    let Some(id) = inserted else {
        let old_id: i64 =
            sqlx::query_scalar("select id from journal_entries where idempotency_key = $1")
                .bind(entry.idempotency_key())
                .fetch_one(&mut *tx)
                .await?;
        let stored = entry_lines(tx, old_id).await?;
        if stored != lines(entry) {
            return Err(LedgerError::IdempotencyConflict {
                key: entry.idempotency_key().to_string(),
            }
            .into());
        }
        return Ok(Posted {
            id: old_id,
            replayed: true,
        });
    };

    for (i, (account, amount)) in lines(entry).iter().enumerate() {
        let line_no = i32::try_from(i + 1).expect("ít hơn 2^31 posting");
        sqlx::query(
            "insert into postings (entry_id, line_no, account_id, amount_dong) values ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(line_no)
        .bind(account)
        .bind(amount)
        .execute(&mut *tx)
        .await?;
    }
    // Cập nhật theo thứ tự account_id để hai bút toán đồng thời không khoá chéo nhau.
    let mut deltas = lines(entry);
    deltas.sort();
    for (account, amount) in &deltas {
        sqlx::query(
            "insert into account_balances (account_id, balance_dong, version) values ($1, $2, 1) \
             on conflict (account_id) do update set \
             balance_dong = account_balances.balance_dong + excluded.balance_dong, \
             version = account_balances.version + 1, updated_at = now()",
        )
        .bind(account)
        .bind(amount)
        .execute(&mut *tx)
        .await?;
    }
    Ok(Posted {
        id,
        replayed: false,
    })
}

/// Các dòng `(account_id, số tiền)` của một bút toán, theo `line_no`.
pub async fn entry_lines(
    conn: &mut PgConnection,
    entry_id: i64,
) -> Result<Vec<(String, i64)>, sqlx::Error> {
    let rows = sqlx::query(
        "select account_id, amount_dong from postings where entry_id = $1 order by line_no",
    )
    .bind(entry_id)
    .fetch_all(conn)
    .await?;
    Ok(rows
        .iter()
        .map(|r| {
            (
                r.get::<String, _>("account_id"),
                r.get::<i64, _>("amount_dong"),
            )
        })
        .collect())
}

/// Đảo toàn phần một bút toán: cùng tài khoản, mọi số tiền đổi dấu, `reversal_of` trỏ về bút toán gốc.
/// Key suy ra từ bút toán gốc (`<key>:reversal`), nên gọi lại là replay, không tạo bản thứ hai.
/// Không đảo bút toán đảo.
pub async fn reverse_entry(tx: &mut PgConnection, original_id: i64) -> Result<Posted, RefundError> {
    let row = sqlx::query("select idempotency_key, reversal_of from journal_entries where id = $1")
        .bind(original_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(LedgerError::UnknownEntry {
            id: EntryId::parse(&format!("JE-{original_id:06}")).ok_or(LedgerError::Overflow)?,
        })?;
    let key: String = row.get("idempotency_key");
    let reversal_of: Option<i64> = row.get("reversal_of");
    if reversal_of.is_some() {
        return Err(RefundError::CannotReverseReversal(original_id));
    }
    let postings = entry_lines(tx, original_id)
        .await?
        .into_iter()
        .map(|(account, amount)| {
            Ok(Posting::new(
                AccountId::new(&account)?,
                Money::from_dong(amount),
            ))
        })
        .collect::<Result<Vec<_>, LedgerError>>()?;
    let original = JournalEntry::try_new(&key, postings)?;
    let eid = EntryId::parse(&format!("JE-{original_id:06}")).ok_or(LedgerError::Overflow)?;
    let reversal = JournalEntry::reversal_of(eid, &original)?;
    post_entry(tx, &reversal).await
}

/// Ghi nhật ký audit trong transaction của người gọi: audit cùng sống hoặc cùng chết với thay đổi.
pub async fn audit(
    conn: &mut PgConnection,
    actor: &str,
    action: &str,
    subject: &str,
    detail: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("insert into audit_log (actor, action, subject, detail) values ($1, $2, $3, $4)")
        .bind(actor)
        .bind(action)
        .bind(subject)
        .bind(detail)
        .execute(conn)
        .await?;
    Ok(())
}

/// Dữ liệu một chuyến cần thu.
pub struct Trip<'a> {
    /// Mã chuyến, ví dụ `TN-0001`.
    pub id: &'a str,
    /// Mã khách, ví dụ `CU-001`.
    pub customer: &'a str,
    /// Mã tài xế, ví dụ `DR-001`.
    pub driver: &'a str,
    /// Cước.
    pub fare: Money,
    /// Mã giao dịch VíXanh.
    pub external_ref: &'a str,
}

/// Mã hai bút toán của một chuyến đã thu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Captured {
    /// Bút toán `trip:<id>:capture`.
    pub capture: Posted,
    /// Bút toán `trip:<id>:split`.
    pub split: Posted,
}

/// Dựng tài khoản và ghi dòng tiền hai bước của chuyến trong một transaction:
/// capture (khách → clearing), chia (clearing → doanh thu + tài xế), rồi dòng `payments` trạng thái `CAPTURED`.
pub async fn capture_trip(pool: &PgPool, trip: &Trip<'_>) -> Result<Captured, RefundError> {
    let split_amounts = default_policy().split(trip.fare)?;
    let customer = format!("customer:{}", trip.customer);
    let driver = format!("driver:{}", trip.driver);
    let mut tx = pool.begin().await?;
    for (id, kind) in [
        (customer.as_str(), "liability"),
        (driver.as_str(), "liability"),
        ("platform:revenue", "revenue"),
        ("psp:vixanh:clearing", "asset"),
    ] {
        sqlx::query("insert into accounts (id, kind) values ($1, $2) on conflict (id) do nothing")
            .bind(id)
            .bind(kind)
            .execute(&mut *tx)
            .await?;
    }
    let fare = trip.fare;
    let neg = |m: Money| m.checked_neg().ok_or(LedgerError::Overflow);
    let capture = JournalEntry::try_new(
        &format!("trip:{}:capture", trip.id),
        vec![
            Posting::new(AccountId::new(&customer)?, neg(fare)?),
            Posting::new(AccountId::new("psp:vixanh:clearing")?, fare),
        ],
    )?;
    let split = JournalEntry::try_new(
        &format!("trip:{}:split", trip.id),
        vec![
            Posting::new(AccountId::new("psp:vixanh:clearing")?, neg(fare)?),
            Posting::new(
                AccountId::new("platform:revenue")?,
                split_amounts.platform_fee,
            ),
            Posting::new(AccountId::new(&driver)?, split_amounts.driver_net),
        ],
    )?;
    let capture = post_entry(&mut tx, &capture).await?;
    let split = post_entry(&mut tx, &split).await?;
    sqlx::query(
        "insert into payments (trip_id, customer_id, driver_id, external_ref, captured_dong, fee_dong, status) \
         values ($1, $2, $3, $4, $5, $6, 'CAPTURED') on conflict (trip_id) do nothing",
    )
    .bind(trip.id)
    .bind(&customer)
    .bind(&driver)
    .bind(trip.external_ref)
    .bind(fare.dong())
    .bind(split_amounts.platform_fee.dong())
    .execute(&mut *tx)
    .await?;
    audit(
        &mut tx,
        "system:payment",
        "TRIP_CAPTURED",
        &format!("trip:{}", trip.id),
        &format!("{fare}"),
    )
    .await?;
    tx.commit().await?;
    Ok(Captured { capture, split })
}

/// Số dư mọi tài khoản từ `account_balances`.
pub async fn balances(pool: &PgPool) -> Result<BTreeMap<String, i64>, sqlx::Error> {
    let rows = sqlx::query("select account_id, balance_dong from account_balances")
        .fetch_all(pool)
        .await?;
    Ok(rows
        .iter()
        .map(|r| {
            (
                r.get::<String, _>("account_id"),
                r.get::<i64, _>("balance_dong"),
            )
        })
        .collect())
}

/// Số dư tính lại từ `postings` (nguồn sự thật), để so với projection.
pub async fn balances_from_postings(pool: &PgPool) -> Result<BTreeMap<String, i64>, sqlx::Error> {
    let rows = sqlx::query(
        "select account_id, sum(amount_dong)::bigint as total from postings group by account_id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .iter()
        .map(|r| (r.get::<String, _>("account_id"), r.get::<i64, _>("total")))
        .collect())
}

/// Tổng mọi posting của cả sổ: phải bằng 0.
pub async fn grand_total(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("select coalesce(sum(amount_dong), 0)::bigint from postings")
        .fetch_one(pool)
        .await
}
