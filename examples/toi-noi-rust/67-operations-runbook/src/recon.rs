//! Đối soát theo dòng clearing (bản rút gọn của bài 47) và bút toán điều chỉnh có tham chiếu.
//!
//! Hàm ở đây ĐỌC để báo lệch. Duy nhất [`post_adjustment`] ghi, và chỉ ghi bút toán MỚI.

use jiff::Timestamp;
use sqlx::{PgPool, Row};

use crate::alert::ReconStats;
use crate::store::{CLEARING, NewEntry, post_entry};

/// Nạp các dòng sao kê VíXanh `(external_ref, số tiền)`.
///
/// # Errors
/// Lỗi SQL.
pub async fn load_statement(pool: &PgPool, rows: &[(&str, i64)]) -> Result<(), sqlx::Error> {
    for (external_ref, amount) in rows {
        sqlx::query("INSERT INTO psp_statement (external_ref, amount_dong) VALUES ($1, $2)")
            .bind(external_ref)
            .bind(amount)
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// Hai phía theo mã: bên ta là tổng dòng clearing dương của bút toán có `psp_refs`,
/// bên VíXanh là tổng sao kê. Khớp khi hai tổng bằng nhau.
const JOINED: &str = "\
    WITH ours AS ( \
        SELECT r.external_ref, sum(p.amount_dong)::bigint AS amt \
        FROM psp_refs r JOIN postings p \
          ON p.entry_id = r.entry_id AND p.account_id = 'psp:vixanh:clearing' AND p.amount_dong > 0 \
        GROUP BY r.external_ref), \
    psp AS ( \
        SELECT external_ref, sum(amount_dong)::bigint AS amt FROM psp_statement GROUP BY external_ref), \
    j AS ( \
        SELECT COALESCE(o.external_ref, p.external_ref) AS external_ref, o.amt AS ours, p.amt AS psp \
        FROM ours o FULL JOIN psp p ON p.external_ref = o.external_ref)";

/// Thống kê đối soát: số mã so được, số mã lệch, tổng chênh tuyệt đối. Chưa có sao kê thì toàn 0.
///
/// # Errors
/// Lỗi SQL.
pub async fn stats(pool: &PgPool) -> Result<ReconStats, sqlx::Error> {
    // Chưa nạp sao kê thì chưa đối soát: coi mọi bút toán là «thiếu bên PSP» sẽ là báo động giả.
    let loaded: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM psp_statement)")
        .fetch_one(pool)
        .await?;
    if !loaded {
        return Ok(ReconStats::default());
    }
    let sql = format!(
        "{JOINED} SELECT count(*) AS compared, \
                count(*) FILTER (WHERE ours IS DISTINCT FROM psp) AS mismatched, \
                COALESCE(sum(abs(COALESCE(psp, 0) - COALESCE(ours, 0))) \
                         FILTER (WHERE ours IS DISTINCT FROM psp), 0)::bigint AS abs_diff \
         FROM j"
    );
    let row = sqlx::query(sqlx::AssertSqlSafe(sql))
        .fetch_one(pool)
        .await?;
    Ok(ReconStats {
        compared_refs: row.get("compared"),
        mismatched_refs: row.get("mismatched"),
        abs_diff_dong: row.get("abs_diff"),
    })
}

/// Một mã lệch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mismatch {
    /// Mã giao dịch VíXanh.
    pub external_ref: String,
    /// Số tiền bên ta; `None` là thiếu bút toán.
    pub ours_dong: Option<i64>,
    /// Số tiền sao kê; `None` là sao kê không có.
    pub psp_dong: Option<i64>,
}

/// Danh sách mã lệch, sắp theo mã.
///
/// # Errors
/// Lỗi SQL.
pub async fn mismatches(pool: &PgPool) -> Result<Vec<Mismatch>, sqlx::Error> {
    let sql = format!(
        "{JOINED} SELECT external_ref, ours, psp FROM j \
         WHERE ours IS DISTINCT FROM psp ORDER BY external_ref"
    );
    let rows = sqlx::query(sqlx::AssertSqlSafe(sql))
        .fetch_all(pool)
        .await?;
    Ok(rows
        .iter()
        .map(|row| Mismatch {
            external_ref: row.get("external_ref"),
            ours_dong: row.get("ours"),
            psp_dong: row.get("psp"),
        })
        .collect())
}

/// Bút toán điều chỉnh cho một mã bị thiếu bên ta.
#[derive(Debug, Clone)]
pub struct Adjustment {
    /// Mã giao dịch VíXanh cần bù.
    pub external_ref: String,
    /// Ví khách của giao dịch (đã xác minh với nguồn, không đoán).
    pub customer: String,
    /// Số tiền theo sao kê, đồng.
    pub amount_dong: i64,
    /// Kỳ của giao dịch gốc, ví dụ `T10`.
    pub original_period: String,
    /// Lý do, ghi vào tham chiếu.
    pub reason: String,
}

/// Ghi bút toán MỚI `recon:<ref>:capture-adjust` (khách ra, clearing vào), kèm `psp_refs` để lần
/// đối soát sau thấy khớp, và `recon_adjustments` để lần ngược về kỳ và lý do. Không sửa dòng cũ nào.
/// `at` phải nằm trong kỳ đang mở; nếu rơi vào kỳ đã đóng thì trigger từ chối.
/// `Ok(None)` khi bút toán điều chỉnh này đã có (chạy lại không tạo bản thứ hai).
///
/// # Errors
/// Lỗi SQL, gồm `23001` khi `at` thuộc kỳ đã đóng.
pub async fn post_adjustment(
    pool: &PgPool,
    adj: &Adjustment,
    at: Timestamp,
    out_topic: &str,
) -> Result<Option<i64>, sqlx::Error> {
    let entry = NewEntry {
        key: format!("recon:{}:capture-adjust", adj.external_ref),
        reversal_of: None,
        postings: vec![
            (adj.customer.clone(), -adj.amount_dong),
            (CLEARING.to_string(), adj.amount_dong),
        ],
        external_ref: Some(adj.external_ref.clone()),
    };
    let mut tx = pool.begin().await?;
    let id = post_entry(&mut tx, &entry, at, out_topic).await?;
    if let Some(id) = id {
        sqlx::query(
            "INSERT INTO recon_adjustments (entry_id, external_ref, original_period, reason) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(&adj.external_ref)
        .bind(&adj.original_period)
        .bind(&adj.reason)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(id)
}
