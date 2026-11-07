//! Metric đúng/sai của sổ cái, đo bằng SQL thật. Mỗi metric là một truy vấn chỉ đọc.
//! Thời gian đi qua tham số `as_of` do người gọi truyền, không đọc đồng hồ máy.

use anyhow::Result;
use sqlx::PgPool;

/// Bút toán có tổng khác 0, hoặc ít hơn 2 dòng. Kết quả đúng là rỗng.
pub const UNBALANCED_SQL: &str = "\
SELECT e.idempotency_key,
       COALESCE(SUM(p.amount_dong), 0)::bigint AS total_dong,
       COUNT(p.entry_id)::bigint               AS lines
FROM journal_entries e
LEFT JOIN postings p ON p.entry_id = e.id
GROUP BY e.id, e.idempotency_key
HAVING COALESCE(SUM(p.amount_dong), 0) <> 0 OR COUNT(p.entry_id) < 2
ORDER BY e.id";

/// Tổng toàn sổ theo postings. Kết quả đúng là 0.
pub const GRAND_TOTAL_SQL: &str = "SELECT COALESCE(SUM(amount_dong), 0)::bigint FROM postings";

/// Tổng toàn sổ theo projection `account_balances`: rẻ hơn nhiều, cũng phải bằng 0.
pub const PROJECTION_TOTAL_SQL: &str =
    "SELECT COALESCE(SUM(balance_dong), 0)::bigint FROM account_balances";

/// Số dư âm bất thường theo loại tài khoản. Cả bốn loại đều bình thường ở chiều dương.
/// `$1 = true`: ví khách được phép âm (mô hình bỏ qua nạp ví, ví khách là tài khoản biên).
/// Mô hình nạp ví đổi biên sang clearing: truy vấn này cần ngoại lệ mới, lab không viết.
pub const NEGATIVE_SQL: &str = "\
SELECT a.id, a.kind, COALESCE(SUM(p.amount_dong), 0)::bigint AS balance_dong
FROM accounts a
LEFT JOIN postings p ON p.account_id = a.id
GROUP BY a.id, a.kind
HAVING COALESCE(SUM(p.amount_dong), 0) < 0
   AND NOT ($1::bool AND a.id LIKE 'customer:%')
ORDER BY a.id";

/// Tiền nằm ở tài khoản `*:clearing` quá `$2` ngày kể từ bút toán đầu của cùng một tham chiếu
/// (`trip:TN-0003`, `delivery:DL-0001`). Tham chiếu có tổng clearing bằng 0 là đã chia xong.
pub const STUCK_CLEARING_SQL: &str = "\
SELECT p.account_id,
       split_part(e.idempotency_key, ':', 1) || ':' || split_part(e.idempotency_key, ':', 2) AS reference,
       SUM(p.amount_dong)::bigint AS stuck_dong,
       FLOOR(EXTRACT(EPOCH FROM ($1::text::timestamptz - MIN(e.posted_at))) / 86400)::bigint AS age_days
FROM postings p
JOIN journal_entries e ON e.id = p.entry_id
WHERE p.account_id LIKE '%:clearing'
GROUP BY p.account_id, reference
HAVING SUM(p.amount_dong) <> 0
   AND MIN(e.posted_at) <= $1::text::timestamptz - make_interval(days => $2)
ORDER BY age_days DESC, reference";

/// Tham số của một lần đo.
pub struct Policy<'a> {
    /// Thời điểm đo, ISO 8601 (truyền vào, không đọc đồng hồ).
    pub as_of: &'a str,
    /// Clearing còn tiền quá số ngày này là tồn đọng.
    pub clearing_max_days: i32,
    /// Ví khách được phép âm hay không.
    pub customer_may_be_negative: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unbalanced {
    pub key: String,
    pub total_dong: i64,
    pub lines: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbnormalBalance {
    pub account: String,
    pub kind: String,
    pub balance_dong: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StuckClearing {
    pub account: String,
    pub reference: String,
    pub stuck_dong: i64,
    pub age_days: i64,
}

/// Kết quả một lần đo cả bốn metric. Sổ khỏe thì mọi danh sách rỗng và hai tổng bằng 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub unbalanced: Vec<Unbalanced>,
    pub postings_total: i64,
    pub projection_total: i64,
    pub abnormal_negative: Vec<AbnormalBalance>,
    pub stuck_clearing: Vec<StuckClearing>,
}

impl Report {
    /// Sổ khỏe khi không metric nào báo gì.
    pub fn is_healthy(&self) -> bool {
        self.unbalanced.is_empty()
            && self.postings_total == 0
            && self.projection_total == 0
            && self.abnormal_negative.is_empty()
            && self.stuck_clearing.is_empty()
    }
}

/// Đo cả bốn metric.
pub async fn measure(pool: &PgPool, policy: &Policy<'_>) -> Result<Report> {
    let unbalanced = sqlx::query_as::<_, (String, i64, i64)>(UNBALANCED_SQL)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|(key, total_dong, lines)| Unbalanced {
            key,
            total_dong,
            lines,
        })
        .collect();
    let postings_total = sqlx::query_scalar(GRAND_TOTAL_SQL).fetch_one(pool).await?;
    let projection_total = sqlx::query_scalar(PROJECTION_TOTAL_SQL)
        .fetch_one(pool)
        .await?;
    let abnormal_negative = sqlx::query_as::<_, (String, String, i64)>(NEGATIVE_SQL)
        .bind(policy.customer_may_be_negative)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|(account, kind, balance_dong)| AbnormalBalance {
            account,
            kind,
            balance_dong,
        })
        .collect();
    let stuck_clearing = sqlx::query_as::<_, (String, String, i64, i64)>(STUCK_CLEARING_SQL)
        .bind(policy.as_of)
        .bind(policy.clearing_max_days)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|(account, reference, stuck_dong, age_days)| StuckClearing {
            account,
            reference,
            stuck_dong,
            age_days,
        })
        .collect();
    Ok(Report {
        unbalanced,
        postings_total,
        projection_total,
        abnormal_negative,
        stuck_clearing,
    })
}
