//! Ghi bút toán vào PostgreSQL và giữ `account_balances` làm projection.

use std::collections::BTreeMap;

use ledger_core::JournalEntry;
use sqlx::{PgPool, Postgres, Transaction};

/// Bốn tài khoản của Tới Nơi và loại của chúng.
pub const ACCOUNTS: [(&str, &str); 4] = [
    ("customer:CU-001", "liability"),
    ("driver:DR-001", "liability"),
    ("platform:revenue", "revenue"),
    ("psp:vixanh:clearing", "asset"),
];

/// Mã hiển thị của bút toán: `JE-000001`.
pub fn display_id(id: i64) -> String {
    format!("JE-{id:06}")
}

pub async fn seed_accounts(pool: &PgPool) -> Result<(), sqlx::Error> {
    for (id, kind) in ACCOUNTS {
        sqlx::query("INSERT INTO accounts (id, kind) VALUES ($1, $2)")
            .bind(id)
            .bind(kind)
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// Cộng `delta` vào số dư một tài khoản: tạo dòng nếu chưa có, cộng dồn nếu đã có.
/// Một câu lệnh duy nhất, không có bước "đọc rồi ghi" ở phía ứng dụng.
pub const UPSERT_BALANCE: &str = "\
INSERT INTO account_balances (account_id, balance_dong, version)
VALUES ($1, $2, 1)
ON CONFLICT (account_id) DO UPDATE
SET balance_dong = account_balances.balance_dong + EXCLUDED.balance_dong,
    version      = account_balances.version + 1,
    updated_at   = now()";

/// Ghi bút toán và cập nhật số dư trong transaction của người gọi (chưa commit).
///
/// Trả về `id` của `journal_entries`.
pub async fn post_entry_in(
    tx: &mut Transaction<'_, Postgres>,
    entry: &JournalEntry,
) -> Result<i64, sqlx::Error> {
    let entry_id: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (idempotency_key) VALUES ($1) RETURNING id",
    )
    .bind(entry.idempotency_key())
    .fetch_one(&mut **tx)
    .await?;

    // Gộp delta theo tài khoản; BTreeMap duyệt theo tên nên mọi bút toán khoá số dư theo cùng thứ tự.
    // Một transaction ghi nhiều bút toán thì không còn bảo đảm này (xem test deadlock).
    let mut deltas: BTreeMap<&str, i64> = BTreeMap::new();
    for (index, posting) in entry.postings().iter().enumerate() {
        let line_no = i32::try_from(index + 1).expect("bút toán không có hàng tỷ dòng");
        sqlx::query(
            "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(entry_id)
        .bind(line_no)
        .bind(posting.account.as_str())
        .bind(posting.amount.dong())
        .execute(&mut **tx)
        .await?;
        *deltas.entry(posting.account.as_str()).or_insert(0) += posting.amount.dong();
    }
    for (account, delta) in deltas {
        sqlx::query(UPSERT_BALANCE)
            .bind(account)
            .bind(delta)
            .execute(&mut **tx)
            .await?;
    }
    Ok(entry_id)
}

/// Ghi một bút toán trong transaction riêng; posting, bút toán và số dư cùng commit hoặc cùng mất.
pub async fn post_entry(pool: &PgPool, entry: &JournalEntry) -> Result<i64, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let id = post_entry_in(&mut tx, entry).await?;
    tx.commit().await?;
    Ok(id)
}

/// Mô phỏng code lỗi: ghi hai posting lệch (-100 và +90) rồi cập nhật số dư như bình thường.
/// Từng câu lệnh đều chạy được; DB chỉ từ chối ở `COMMIT`.
pub async fn post_unbalanced(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let entry_id: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (idempotency_key) VALUES ('demo:unbalanced') RETURNING id",
    )
    .fetch_one(&mut *tx)
    .await?;
    for (line_no, account, amount) in [
        (1_i32, "customer:CU-001", -100_i64),
        (2, "driver:DR-001", 90),
    ] {
        sqlx::query(
            "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(entry_id)
        .bind(line_no)
        .bind(account)
        .bind(amount)
        .execute(&mut *tx)
        .await?;
        sqlx::query(UPSERT_BALANCE)
            .bind(account)
            .bind(amount)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await
}

/// Bút toán chỉ có một dòng: CHECK từng dòng không bắt được, trigger từ chối lúc `COMMIT`.
pub async fn post_one_line(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let entry_id: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (idempotency_key) VALUES ('demo:one-line') RETURNING id",
    )
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
         VALUES ($1, 1, 'customer:CU-001', -100)",
    )
    .bind(entry_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

/// Số dư theo projection; 0 nếu tài khoản chưa có dòng.
pub async fn projected_balance(pool: &PgPool, account: &str) -> Result<i64, sqlx::Error> {
    let balance: Option<i64> =
        sqlx::query_scalar("SELECT balance_dong FROM account_balances WHERE account_id = $1")
            .bind(account)
            .fetch_optional(pool)
            .await?;
    Ok(balance.unwrap_or(0))
}

/// Một tài khoản mà projection khác tổng posting.
#[derive(Debug, PartialEq, Eq)]
pub struct Drift {
    pub account_id: String,
    pub projected: i64,
    pub actual: i64,
}

/// Đối chiếu `account_balances` với `SUM(postings)`; trả các tài khoản lệch.
pub async fn reconcile(pool: &PgPool) -> Result<Vec<Drift>, sqlx::Error> {
    let rows: Vec<(String, i64, i64)> = sqlx::query_as(
        "SELECT COALESCE(b.account_id, p.account_id) AS account_id,
                COALESCE(b.balance_dong, 0)           AS projected,
                COALESCE(p.total, 0)                  AS actual
         FROM account_balances b
         FULL JOIN (SELECT account_id, SUM(amount_dong)::bigint AS total
                    FROM postings GROUP BY account_id) p USING (account_id)
         WHERE COALESCE(b.balance_dong, 0) <> COALESCE(p.total, 0)
         ORDER BY 1",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(account_id, projected, actual)| Drift {
            account_id,
            projected,
            actual,
        })
        .collect())
}

/// Dựng lại projection từ posting (sửa lệch). Trả số tài khoản đã ghi.
pub async fn rebuild_balances(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO account_balances (account_id, balance_dong, version)
         SELECT account_id, SUM(amount_dong)::bigint, 1 FROM postings GROUP BY account_id
         ON CONFLICT (account_id) DO UPDATE
         SET balance_dong = EXCLUDED.balance_dong,
             version      = account_balances.version + 1,
             updated_at   = now()",
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
