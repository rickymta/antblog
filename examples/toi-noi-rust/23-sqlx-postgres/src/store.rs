//! Ghi và đọc bút toán. Mọi câu SQL ở đây dùng `query!`/`query_as!`:
//! tên cột, kiểu tham số và kiểu cột trả về được kiểm lúc biên dịch.

use anyhow::{Context, Result, ensure};
use ledger_core::{AccountId, EntryId, JournalEntry, Money, Posting};
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::mapping::{account_kind, entry_id_from_pk, entry_pk, to_jiff};

/// Bút toán đọc lại từ database.
#[derive(Debug)]
pub struct StoredEntry {
    /// Mã `JE-xxxxxx`.
    pub id: EntryId,
    /// Bút toán đã qua kiểm tra bất biến của `ledger-core`.
    pub entry: JournalEntry,
    /// Thời điểm ghi (UTC).
    pub posted_at: jiff::Timestamp,
}

// Hàng của `postings`; tên và kiểu trường phải khớp cột, nếu không `query_as!` không biên dịch.
struct PostingRow {
    line_no: i32,
    account_id: String,
    amount_dong: i64,
}

/// Ghi một bút toán (đầu bút toán, các dòng, tài khoản chưa có) trong một transaction.
///
/// # Errors
/// Lỗi database, kể cả vi phạm unique `idempotency_key` (xem bài 24 về cách xử lý).
pub async fn insert_entry(pool: &PgPool, entry: &JournalEntry) -> Result<EntryId> {
    let mut tx = pool.begin().await?;

    for posting in entry.postings() {
        let id = posting.account.as_str();
        sqlx::query!(
            "insert into accounts (id, kind) values ($1, $2) on conflict (id) do nothing",
            id,
            account_kind(id),
        )
        .execute(&mut *tx)
        .await?;
    }

    // `returning id`: cột `generated always as identity` do PostgreSQL cấp, ta không tự đặt.
    let raw_id = sqlx::query_scalar!(
        "insert into journal_entries (idempotency_key) values ($1) returning id",
        entry.idempotency_key(),
    )
    .fetch_one(&mut *tx)
    .await?;

    for (index, posting) in entry.postings().iter().enumerate() {
        let line_no = i32::try_from(index + 1).context("quá nhiều dòng")?;
        sqlx::query!(
            "insert into postings (entry_id, line_no, account_id, amount_dong) \
             values ($1, $2, $3, $4)",
            raw_id,
            line_no,
            posting.account.as_str(),
            posting.amount.dong(),
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    entry_id_from_pk(raw_id)
}

/// Đọc lại bút toán theo mã; `None` nếu không có.
///
/// # Errors
/// Lỗi database, hoặc dữ liệu đọc ra vi phạm bất biến của `ledger-core`.
pub async fn load_entry(pool: &PgPool, id: EntryId) -> Result<Option<StoredEntry>> {
    let pk = entry_pk(id)?;

    let Some(head) = sqlx::query!(
        "select idempotency_key, posted_at from journal_entries where id = $1",
        pk
    )
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    let rows = sqlx::query_as!(
        PostingRow,
        "select line_no, account_id, amount_dong from postings \
         where entry_id = $1 order by line_no",
        pk
    )
    .fetch_all(pool)
    .await?;

    let mut postings = Vec::with_capacity(rows.len());
    for (expected, row) in (1..).zip(&rows) {
        ensure!(row.line_no == expected, "line_no không liên tục");
        postings.push(Posting::new(
            AccountId::new(&row.account_id)?,
            Money::from_dong(row.amount_dong),
        ));
    }
    // Dựng lại qua `try_new`: bút toán đọc từ DB cũng phải cân bằng.
    let entry = JournalEntry::try_new(&head.idempotency_key, postings)?;

    Ok(Some(StoredEntry {
        id,
        entry,
        // `posted_at` là TIMESTAMPTZ NOT NULL nên macro cho `time::OffsetDateTime`, không phải `Option`.
        posted_at: to_jiff(head.posted_at)?,
    }))
}

/// Số dư từng tài khoản trong schema hiện tại: tổng các posting, theo mã tài khoản.
///
/// # Errors
/// Lỗi database hoặc mã tài khoản không hợp lệ.
pub async fn balances(pool: &PgPool) -> Result<Vec<(AccountId, Money)>> {
    // `sum(bigint)` trong PostgreSQL trả NUMERIC, nên ép `::bigint` để nhận `i64`;
    // hậu tố `!` nói «cột này không NULL» (macro không biết với biểu thức).
    let rows = sqlx::query!(
        r#"select account_id, sum(amount_dong)::bigint as "balance!"
           from postings group by account_id order by account_id"#
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|r| Ok((AccountId::new(&r.account_id)?, Money::from_dong(r.balance))))
        .collect()
}

/// Tổng giá trị tuyệt đối của mọi posting, đọc thẳng NUMERIC thành `rust_decimal::Decimal`.
///
/// # Errors
/// Lỗi database.
pub async fn gross_volume(pool: &PgPool) -> Result<Decimal> {
    // `sum(bigint)` là NUMERIC; không ép kiểu thì macro suy ra `Decimal` nhờ feature `rust_decimal`.
    let volume = sqlx::query_scalar!(
        r#"select coalesce(sum(abs(amount_dong)), 0) as "volume!" from postings"#
    )
    .fetch_one(pool)
    .await?;
    Ok(volume)
}
