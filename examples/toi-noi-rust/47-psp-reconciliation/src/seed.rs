//! Dữ liệu phía sổ cái: bút toán thu tiền qua VíXanh (mẫu nhỏ và sinh hàng loạt cho bench).

use anyhow::Result;
use jiff::Timestamp;
use ledger_core::{AccountId, JournalEntry, Money, Posting};
use sqlx::PgPool;

use crate::period::Period;

/// Tài khoản clearing: số dương là tiền VíXanh đã thu và đang giữ (bài 13).
pub const CLEARING: &str = "psp:vixanh:clearing";

/// Ghi một bút toán thu tiền (khách -X, clearing +X) và gắn nó với `external_ref`.
/// Bút toán đi qua `JournalEntry::try_new` của `ledger-core`, nên tổng bằng 0 được kiểm trước khi ghi.
///
/// # Errors
/// Bút toán không hợp lệ hoặc lỗi cơ sở dữ liệu (ví dụ trùng `idempotency_key`).
pub async fn post_capture(
    pool: &PgPool,
    key: &str,
    external_ref: &str,
    customer: &str,
    dong: i64,
    posted_at: Timestamp,
) -> Result<i64> {
    let entry = JournalEntry::try_new(
        key,
        vec![
            Posting::new(AccountId::new(customer)?, Money::from_dong(-dong)),
            Posting::new(AccountId::new(CLEARING)?, Money::from_dong(dong)),
        ],
    )?;
    let mut tx = pool.begin().await?;
    let entry_id: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (idempotency_key, posted_at) \
         VALUES ($1, $2::timestamptz) RETURNING id",
    )
    .bind(entry.idempotency_key())
    .bind(posted_at.to_string())
    .fetch_one(&mut *tx)
    .await?;
    for (index, posting) in entry.postings().iter().enumerate() {
        sqlx::query(
            "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(entry_id)
        .bind(i32::try_from(index + 1)?)
        .bind(posting.account.as_str())
        .bind(posting.amount.dong())
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("INSERT INTO psp_refs (entry_id, external_ref) VALUES ($1, $2)")
        .bind(entry_id)
        .bind(external_ref)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(entry_id)
}

/// Phía sổ cái của sao kê mẫu `data/vixanh_2026-11-05.csv`: (key, external_ref, khách, đồng, posted_at).
const SAMPLE_LEDGER: [(&str, &str, &str, i64, &str); 10] = [
    (
        "trip:TN-0001:capture",
        "vx_ch_0001",
        "customer:CU-001",
        25_500,
        "2026-11-05T03:00:05Z",
    ),
    (
        "trip:TN-0002:capture",
        "vx_ch_0002",
        "customer:CU-002",
        40_000,
        "2026-11-05T04:10:00Z",
    ),
    (
        "trip:TN-0003:capture",
        "vx_ch_0003",
        "customer:CU-003",
        31_500,
        "2026-11-05T05:20:00Z",
    ),
    (
        "trip:TN-0004:capture",
        "vx_ch_0004",
        "customer:CU-004",
        18_000,
        "2026-11-05T06:30:00Z",
    ),
    (
        "trip:TN-0006:capture",
        "vx_ch_0006",
        "customer:CU-006",
        22_000,
        "2026-11-05T07:40:00Z",
    ),
    // 23:58:30 giờ Việt Nam: còn trong ngày 5, nhưng VíXanh ghi 00:02 ngày 6.
    (
        "trip:TN-0007:capture",
        "vx_ch_0007",
        "customer:CU-007",
        60_000,
        "2026-11-05T16:58:30Z",
    ),
    (
        "trip:TN-0008:capture",
        "vx_ch_0008",
        "customer:CU-008",
        12_000,
        "2026-11-05T08:50:00Z",
    ),
    // 03:15 ngày 6 giờ Việt Nam: thuộc kỳ sau ở cả hai phía.
    (
        "trip:TN-0009:capture",
        "vx_ch_0009",
        "customer:CU-009",
        33_000,
        "2026-11-05T20:15:00Z",
    ),
    // Sổ ta ghi tách hai bút toán 10.000 cho một dòng 20.000 của VíXanh.
    (
        "trip:TN-0010:capture:1",
        "vx_ch_0010",
        "customer:CU-010",
        10_000,
        "2026-11-05T09:00:00Z",
    ),
    (
        "trip:TN-0010:capture:2",
        "vx_ch_0010",
        "customer:CU-010",
        10_000,
        "2026-11-05T09:00:40Z",
    ),
];

/// Nạp phía sổ cái của sao kê mẫu. Trả số bút toán đã ghi.
///
/// # Errors
/// Lỗi cơ sở dữ liệu hoặc thời điểm mẫu sai định dạng.
pub async fn seed_sample_ledger(pool: &PgPool) -> Result<usize> {
    for (key, external_ref, customer, dong, at) in SAMPLE_LEDGER {
        post_capture(pool, key, external_ref, customer, dong, at.parse()?).await?;
    }
    Ok(SAMPLE_LEDGER.len())
}

/// Số bút toán và tổng posting hiện có: dùng để chứng minh đối soát không ghi sổ.
///
/// # Errors
/// Lỗi cơ sở dữ liệu.
pub async fn ledger_fingerprint(pool: &PgPool) -> Result<(i64, i64, i64)> {
    let row: (i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM journal_entries), \
                (SELECT count(*) FROM postings), \
                (SELECT COALESCE(sum(abs(amount_dong)), 0)::bigint FROM postings)",
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Số tiền của ref thứ `i` (1..) trong bộ dữ liệu sinh: cùng công thức với SQL bên dưới.
pub fn generated_amount(i: i64) -> i64 {
    10_000 + (i % 490) * 1_000
}

/// Nạp `rows` bút toán sinh sẵn (ref `vx_ch_<i:08>`), với các lớp bất thường theo `i % 1000`:
/// 0 thiếu bên ta (không nạp), 2 lệch +500 đồng, 5 ghi sát cuối kỳ (VíXanh ghi sau nửa đêm);
/// các lớp 1, 3, 4 khác ở phía sao kê. Dùng SQL `generate_series` nên không đi qua Rust.
///
/// # Errors
/// Lỗi cơ sở dữ liệu.
pub async fn seed_generated_ledger(pool: &PgPool, rows: i64, period: &Period) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO journal_entries (idempotency_key, posted_at) \
         SELECT 'gen:' || lpad(i::text, 8, '0'), \
                CASE WHEN i % 1000 = 5 THEN $2::timestamptz - interval '10 seconds' \
                     ELSE $1::timestamptz + ((i * 7) % 80000) * interval '1 second' END \
         FROM generate_series(1::bigint, $3::bigint) AS i \
         WHERE i % 1000 <> 0 ORDER BY i",
    )
    .bind(period.start.to_string())
    .bind(period.end.to_string())
    .bind(rows)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO psp_refs (entry_id, external_ref) \
         SELECT id, 'vx_ch_' || substr(idempotency_key, 5) FROM journal_entries \
         WHERE idempotency_key LIKE 'gen:%'",
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) \
         SELECT e.id, l.n, \
                CASE l.n WHEN 1 THEN 'customer:CU-001' ELSE 'psp:vixanh:clearing' END, \
                CASE l.n WHEN 1 THEN -a.amt ELSE a.amt END \
         FROM journal_entries e \
         CROSS JOIN LATERAL (SELECT substr(e.idempotency_key, 5)::bigint AS i) k \
         CROSS JOIN LATERAL (SELECT 10000 + (k.i % 490) * 1000 \
                                    + CASE WHEN k.i % 1000 = 2 THEN 500 ELSE 0 END AS amt) a \
         CROSS JOIN (VALUES (1), (2)) AS l(n) \
         WHERE e.idempotency_key LIKE 'gen:%'",
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    sqlx::query("ANALYZE journal_entries").execute(pool).await?;
    sqlx::query("ANALYZE postings").execute(pool).await?;
    sqlx::query("ANALYZE psp_refs").execute(pool).await?;
    Ok(())
}
