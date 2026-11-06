//! Ghi bút toán, posting và số dư trong transaction của người gọi (rút từ lab 24, 25).

use std::collections::BTreeMap;

use anyhow::{Result, anyhow};
use jiff::Timestamp;
use ledger_core::JournalEntry;
use sqlx::{PgConnection, PgPool, Row};

/// Biến động số dư gom theo tài khoản. `BTreeMap` cho thứ tự khóa cố định: mọi transaction
/// khóa hàng `account_balances` theo cùng một thứ tự nên không chờ vòng tròn.
pub type Deltas = BTreeMap<String, i64>;

/// Cộng các posting của `entry` vào `deltas` (có kiểm tra tràn số).
///
/// # Errors
/// Tổng một tài khoản vượt `i64`.
pub fn collect_deltas(deltas: &mut Deltas, entry: &JournalEntry) -> Result<()> {
    for posting in entry.postings() {
        let slot = deltas
            .entry(posting.account.as_str().to_string())
            .or_insert(0);
        *slot = slot
            .checked_add(posting.amount.dong())
            .ok_or_else(|| anyhow!("tràn số khi gom biến động"))?;
    }
    Ok(())
}

/// Tạo tài khoản nếu chưa có.
///
/// # Errors
/// Lỗi SQL.
pub async fn ensure_account(
    conn: &mut PgConnection,
    id: &str,
    kind: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO accounts (id, kind) VALUES ($1, $2) ON CONFLICT (id) DO NOTHING")
        .bind(id)
        .bind(kind)
        .execute(conn)
        .await?;
    Ok(())
}

/// Chèn bút toán và posting, `posted_at` lấy từ đồng hồ truyền vào (không phải `now()` của DB).
/// `Ok(None)`: `idempotency_key` đã có, không chèn gì. Chưa đụng `account_balances`.
///
/// # Errors
/// Lỗi SQL, ví dụ khóa ngoại 23503 khi tài khoản không tồn tại.
pub async fn insert_entry(
    conn: &mut PgConnection,
    entry: &JournalEntry,
    at: Timestamp,
    reversal_of: Option<i64>,
) -> Result<Option<i64>, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO journal_entries (idempotency_key, reversal_of, posted_at) \
         VALUES ($1, $2, $3::timestamptz) \
         ON CONFLICT (idempotency_key) DO NOTHING RETURNING id",
    )
    .bind(entry.idempotency_key())
    .bind(reversal_of)
    .bind(at.to_string())
    .fetch_optional(&mut *conn)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let id: i64 = row.get("id");
    for (index, posting) in entry.postings().iter().enumerate() {
        let line_no = i32::try_from(index + 1).expect("bút toán không có hàng tỷ dòng");
        sqlx::query(
            "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
             VALUES ($1, $2, $3, $4)",
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

/// Cộng biến động vào `account_balances`, mỗi tài khoản một lệnh, theo thứ tự khóa của `deltas`.
///
/// # Errors
/// Lỗi SQL.
pub async fn apply_deltas(
    conn: &mut PgConnection,
    deltas: &Deltas,
    at: Timestamp,
) -> Result<(), sqlx::Error> {
    for (account, delta) in deltas {
        sqlx::query(
            "INSERT INTO account_balances (account_id, balance_dong, version, updated_at) \
             VALUES ($1, $2, 1, $3::timestamptz) \
             ON CONFLICT (account_id) DO UPDATE SET \
               balance_dong = account_balances.balance_dong + EXCLUDED.balance_dong, \
               version = account_balances.version + 1, \
               updated_at = EXCLUDED.updated_at",
        )
        .bind(account)
        .bind(delta)
        .bind(at.to_string())
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// Ghi một bút toán và cập nhật số dư ngay. `Ok(None)` nếu key đã có (số dư không đổi).
///
/// # Errors
/// Lỗi SQL hoặc tràn số.
pub async fn post(
    conn: &mut PgConnection,
    entry: &JournalEntry,
    at: Timestamp,
    reversal_of: Option<i64>,
) -> Result<Option<i64>> {
    let Some(id) = insert_entry(&mut *conn, entry, at, reversal_of).await? else {
        return Ok(None);
    };
    let mut deltas = Deltas::new();
    collect_deltas(&mut deltas, entry)?;
    apply_deltas(conn, &deltas, at).await?;
    Ok(Some(id))
}

/// Số dư trong `account_balances` (0 nếu tài khoản chưa có dòng nào).
///
/// # Errors
/// Lỗi SQL.
pub async fn balance(pool: &PgPool, account: &str) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT balance_dong FROM account_balances WHERE account_id = $1")
        .bind(account)
        .fetch_optional(pool)
        .await?;
    Ok(row.map_or(0, |r| r.get("balance_dong")))
}

/// Số dư tính lại từ posting: dùng để kiểm projection `account_balances` không lệch.
///
/// # Errors
/// Lỗi SQL.
pub async fn postings_sum(pool: &PgPool, account: &str) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "SELECT COALESCE(SUM(amount_dong), 0)::bigint AS total FROM postings WHERE account_id = $1",
    )
    .bind(account)
    .fetch_one(pool)
    .await?;
    Ok(row.get("total"))
}

/// Tổng mọi posting của sổ: phải bằng 0.
///
/// # Errors
/// Lỗi SQL.
pub async fn grand_total(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT COALESCE(SUM(amount_dong), 0)::bigint AS total FROM postings")
        .fetch_one(pool)
        .await?;
    Ok(row.get("total"))
}

/// Đếm bút toán có `idempotency_key` khớp mẫu `LIKE`.
///
/// # Errors
/// Lỗi SQL.
pub async fn count_entries(pool: &PgPool, like: &str) -> Result<i64, sqlx::Error> {
    let row =
        sqlx::query("SELECT count(*) AS n FROM journal_entries WHERE idempotency_key LIKE $1")
            .bind(like)
            .fetch_one(pool)
            .await?;
    Ok(row.get("n"))
}
