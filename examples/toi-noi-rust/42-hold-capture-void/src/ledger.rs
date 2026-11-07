//! Ghi bút toán thật (không phải hold): bút toán, posting và projection `account_balances`.

use ledger_core::{AccountId, JournalEntry, Money, Posting};
use sqlx::{PgConnection, PgPool, Row};

use crate::hold::HoldError;

/// Tài khoản clearing của VíXanh.
pub const PSP_CLEARING: &str = "psp:vixanh:clearing";

/// Chèn bút toán, các posting và cộng dồn `account_balances` trong transaction của người gọi.
/// `Ok(None)`: key đã tồn tại, không ghi gì. `Ok(Some(id))`: bút toán mới.
///
/// # Errors
/// Lỗi SQL; người gọi để transaction rollback.
pub async fn insert_entry(
    conn: &mut PgConnection,
    entry: &JournalEntry,
) -> Result<Option<i64>, sqlx::Error> {
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
            "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(line_no)
        .bind(posting.account.as_str())
        .bind(posting.amount.dong())
        .execute(&mut *conn)
        .await?;
        // Projection cập nhật cùng transaction, upsert như bài 25: UPDATE trần sẽ lặng lẽ bỏ qua
        // tài khoản chưa có hàng số dư và làm projection lệch posting.
        sqlx::query(
            "INSERT INTO account_balances (account_id, balance_dong, version) VALUES ($1, $2, 1) \
             ON CONFLICT (account_id) DO UPDATE \
             SET balance_dong = account_balances.balance_dong + EXCLUDED.balance_dong, \
                 version = account_balances.version + 1, updated_at = now()",
        )
        .bind(posting.account.as_str())
        .bind(posting.amount.dong())
        .execute(&mut *conn)
        .await?;
    }
    Ok(Some(id))
}

/// Đọc bút toán đã ghi theo key (cùng thứ tự dòng). `None` nếu chưa có.
///
/// # Errors
/// Lỗi SQL, hoặc dữ liệu trong DB không thành bút toán hợp lệ.
pub async fn load_by_key(
    conn: &mut PgConnection,
    key: &str,
) -> Result<Option<(i64, JournalEntry)>, HoldError> {
    let Some(head) = sqlx::query("SELECT id FROM journal_entries WHERE idempotency_key = $1")
        .bind(key)
        .fetch_optional(&mut *conn)
        .await?
    else {
        return Ok(None);
    };
    let id: i64 = head.get("id");
    let rows = sqlx::query(
        "SELECT account_id, amount_dong FROM postings WHERE entry_id = $1 ORDER BY line_no",
    )
    .bind(id)
    .fetch_all(&mut *conn)
    .await?;
    let mut postings = Vec::with_capacity(rows.len());
    for row in rows {
        let account: String = row.get("account_id");
        let amount: i64 = row.get("amount_dong");
        postings.push(Posting::new(
            AccountId::new(&account)?,
            Money::from_dong(amount),
        ));
    }
    Ok(Some((id, JournalEntry::try_new(key, postings)?)))
}

/// Fixture của lab: nạp ví cho khách để có số dư đầu (không thuộc phạm vi bài).
/// Bút toán `fixture:topup:<tài khoản>`: ví +`amount`, clearing -`amount`. Sổ dòng tiền đóng kín
/// nên tiền từ ngoài vào hiện thành số âm ở tài khoản biên, ở đây là clearing (cách đọc: bài 41).
///
/// # Errors
/// Lỗi SQL hoặc mã tài khoản sai.
pub async fn fixture_topup(pool: &PgPool, account: &str, amount: Money) -> Result<i64, HoldError> {
    let key = format!("fixture:topup:{account}");
    let negated = amount
        .checked_neg()
        .ok_or(HoldError::InvalidAmount(amount.dong()))?;
    let entry = JournalEntry::try_new(
        &key,
        vec![
            Posting::new(AccountId::new(account)?, amount),
            Posting::new(AccountId::new(PSP_CLEARING)?, negated),
        ],
    )?;
    let mut tx = pool.begin().await?;
    let id = insert_entry(&mut tx, &entry)
        .await?
        .ok_or(HoldError::IdempotencyConflict { key })?;
    tx.commit().await?;
    Ok(id)
}
