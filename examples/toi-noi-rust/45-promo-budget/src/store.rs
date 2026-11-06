//! Adapter PostgreSQL cho sổ cái: ghi bút toán idempotent, cập nhật `account_balances`
//! và (điểm chính của bài) kiểm ngân sách campaign trong cùng transaction.

use std::collections::BTreeMap;
use std::sync::Arc;

use ledger_core::{JournalEntry, LedgerError, Money};
use sqlx::{PgPool, Row};
use thiserror::Error;
use tokio::sync::Barrier;

/// Kết quả ghi một bút toán.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posted {
    /// Mã bút toán trong schema (identity, bắt đầu từ 1).
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

/// Mọi lý do một thao tác khuyến mãi bị từ chối. Lỗi nghiệp vụ tách khỏi lỗi hạ tầng.
#[derive(Debug, Error)]
pub enum PostError {
    /// Ngân sách campaign không đủ. Đây là thứ Ledger trả về cho Promotion service.
    #[error("campaign {campaign} hết ngân sách: cần {requested}, còn {remaining}")]
    BudgetExhausted {
        /// Mã campaign.
        campaign: String,
        /// Số tiền được yêu cầu.
        requested: Money,
        /// Ngân sách còn lại lúc kiểm.
        remaining: Money,
    },
    /// Tiền khách đã trả không khớp cước trừ giảm giá: có service nói sai số.
    #[error("cước {fare} trừ giảm {discount} phải là {expected}, nhưng VíXanh đã thu {captured}")]
    AmountMismatch {
        /// Cước gốc.
        fare: Money,
        /// Mức giảm Promotion đã quyết.
        discount: Money,
        /// Số lẽ ra phải thu.
        expected: Money,
        /// Số bút toán capture đã ghi.
        captured: Money,
    },
    /// Chưa có bút toán capture của chuyến.
    #[error("chuyến {trip} chưa có bút toán capture")]
    NotCaptured {
        /// Mã chuyến.
        trip: String,
    },
    /// Phần nền tảng chịu không khớp số đã redeem: thiếu bút toán redeem hoặc service nói sai số.
    #[error("chuyến {trip}: nền tảng chịu {expected} nhưng đã redeem {redeemed}")]
    NotRedeemed {
        /// Mã chuyến.
        trip: String,
        /// Phần nền tảng chịu theo Promotion.
        expected: Money,
        /// Tổng chi phí đã ghi qua bút toán redeem (đã trừ bút toán đảo).
        redeemed: Money,
    },
    /// Phần chịu giảm giá không hợp lệ (âm, hoặc vượt phần tài xế được nhận).
    #[error("phần chịu giảm giá không hợp lệ: {0}")]
    InvalidShares(String),
    /// Lỗi nghiệp vụ từ `ledger-core` (ví dụ `IdempotencyConflict`).
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    /// Lỗi từ PostgreSQL hoặc sqlx.
    #[error("lỗi database: {0}")]
    Db(#[from] sqlx::Error),
}

impl PostError {
    /// SQLSTATE nếu lỗi đến từ PostgreSQL.
    pub fn sqlstate(&self) -> Option<String> {
        match self {
            PostError::Db(e) => e
                .as_database_error()
                .and_then(|d| d.code())
                .map(|c| c.into_owned()),
            _ => None,
        }
    }

    /// Mã gRPC trả cho Promotion service (C#).
    pub fn grpc_code(&self) -> &'static str {
        match self {
            PostError::BudgetExhausted { .. }
            | PostError::AmountMismatch { .. }
            | PostError::NotCaptured { .. }
            | PostError::NotRedeemed { .. } => "FAILED_PRECONDITION",
            PostError::InvalidShares(_) => "INVALID_ARGUMENT",
            PostError::Ledger(LedgerError::IdempotencyConflict { .. }) => "ALREADY_EXISTS",
            PostError::Ledger(_) => "INVALID_ARGUMENT",
            PostError::Db(_) => "INTERNAL",
        }
    }

    /// Mã nghiệp vụ ổn định để Promotion `switch` theo, không phải phân tích câu chữ.
    pub fn domain_code(&self) -> &'static str {
        match self {
            PostError::BudgetExhausted { .. } => "PROMO_BUDGET_EXHAUSTED",
            PostError::AmountMismatch { .. } => "PROMO_AMOUNT_MISMATCH",
            PostError::NotCaptured { .. } => "TRIP_NOT_CAPTURED",
            PostError::NotRedeemed { .. } => "PROMO_NOT_REDEEMED",
            PostError::InvalidShares(_) => "PROMO_SHARES_INVALID",
            PostError::Ledger(LedgerError::IdempotencyConflict { .. }) => "IDEMPOTENCY_CONFLICT",
            PostError::Ledger(_) => "ENTRY_INVALID",
            PostError::Db(_) => "LEDGER_INTERNAL",
        }
    }

    /// Gửi lại y nguyên có ích không? Hết ngân sách thì không: số dư không tự đầy lên.
    /// Chưa capture thì có, sau khi `PaymentCaptured` đến: hai sự kiện có thể lệch thứ tự.
    pub fn retryable(&self) -> bool {
        matches!(self, PostError::Db(_) | PostError::NotCaptured { .. })
    }
}

/// Cách kiểm ngân sách. Chỉ khác nhau đúng một mệnh đề `for update`.
#[derive(Clone)]
pub enum BudgetGuard {
    /// `select ... for update`: giữ khóa dòng ngân sách tới lúc commit; người sau chờ rồi đọc số mới.
    Locked,
    /// `select` thường: đọc xong là hết ràng buộc. `after_read` cho lab ép mọi bên đọc xong mới cùng ghi.
    Unlocked {
        /// Rào chắn đặt giữa lúc đọc và lúc ghi, chỉ để dựng lịch đua; production không có.
        after_read: Option<Arc<Barrier>>,
    },
}

/// Một lần tiêu ngân sách đi kèm bút toán.
#[derive(Clone)]
pub struct BudgetSpend {
    /// Mã campaign (tài khoản là `promo:budget:<campaign>`).
    pub campaign: String,
    /// Số tiền sẽ tiêu.
    pub amount: Money,
    /// Cách kiểm.
    pub guard: BudgetGuard,
}

/// Tên tài khoản ngân sách của một campaign.
pub fn budget_account(campaign: &str) -> String {
    format!("promo:budget:{campaign}")
}

/// Sổ cái trên PostgreSQL. Dùng pool đã đặt `search_path` nên không nhắc tên schema.
#[derive(Clone)]
pub struct PgLedger {
    pool: PgPool,
}

impl PgLedger {
    /// Bọc một pool.
    pub fn new(pool: PgPool) -> PgLedger {
        PgLedger { pool }
    }

    /// Tạo tài khoản nếu chưa có.
    pub async fn ensure_account(&self, id: &str, kind: &str) -> Result<(), sqlx::Error> {
        sqlx::query("insert into accounts (id, kind) values ($1, $2) on conflict (id) do nothing")
            .bind(id)
            .bind(kind)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Cài hàng rào thứ hai: trigger chặn số dư ngân sách âm (xem `sql/budget_floor.sql`).
    pub async fn install_budget_floor(&self) -> Result<(), sqlx::Error> {
        sqlx::raw_sql(include_str!("../sql/budget_floor.sql"))
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Ghi bút toán không kèm kiểm ngân sách.
    pub async fn post(&self, entry: &JournalEntry) -> Result<Posted, PostError> {
        self.post_with(entry, None).await
    }

    /// Ghi bút toán trong một transaction: header, kiểm ngân sách (nếu có), posting, số dư.
    /// Key đã có thì không ghi và không kiểm ngân sách: replay không được tiêu thêm đồng nào.
    pub async fn post_with(
        &self,
        entry: &JournalEntry,
        spend: Option<&BudgetSpend>,
    ) -> Result<Posted, PostError> {
        let mut tx = self.pool.begin().await?;
        let reversal_of = entry
            .reverses()
            .and_then(|id| id.to_string()[3..].parse::<i64>().ok());
        let inserted: Option<i64> = sqlx::query_scalar(
            "insert into journal_entries (idempotency_key, reversal_of) values ($1, $2) \
             on conflict (idempotency_key) do nothing returning id",
        )
        .bind(entry.idempotency_key())
        .bind(reversal_of)
        .fetch_optional(&mut *tx)
        .await?;

        let Some(id) = inserted else {
            // Key đã có: đọc lại bút toán cũ và so nội dung.
            let old_id: i64 =
                sqlx::query_scalar("select id from journal_entries where idempotency_key = $1")
                    .bind(entry.idempotency_key())
                    .fetch_one(&mut *tx)
                    .await?;
            let stored = entry_lines_in(&mut tx, old_id).await?;
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

        if let Some(spend) = spend {
            check_budget(&mut tx, spend).await?; // Err: tx bị drop, header vừa chèn được rollback
        }

        for (i, (account, amount)) in lines(entry).iter().enumerate() {
            let line_no = i32::try_from(i + 1).expect("bút toán có ít hơn 2^31 posting");
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
        // Cập nhật số dư theo một thứ tự cố định để hai bút toán đồng thời không khóa chéo nhau:
        // ngân sách trước (kiểm ngân sách đã khóa dòng này đầu tiên), còn lại theo account_id.
        let mut deltas = lines(entry);
        deltas.sort_by_key(|(account, _)| (!account.starts_with("promo:budget:"), account.clone()));
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
        tx.commit().await?;
        Ok(Posted {
            id,
            replayed: false,
        })
    }

    /// Số dư một tài khoản; `None` nếu chưa có bút toán nào chạm tới nó.
    pub async fn balance(&self, account: &str) -> Result<Option<Money>, sqlx::Error> {
        let row = sqlx::query("select balance_dong from account_balances where account_id = $1")
            .bind(account)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| Money::from_dong(r.get("balance_dong"))))
    }

    /// Ảnh chụp mọi số dư (đơn vị đồng), để so trước/sau một thao tác.
    pub async fn balances(&self) -> Result<BTreeMap<String, i64>, sqlx::Error> {
        let rows = sqlx::query("select account_id, balance_dong from account_balances")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows
            .iter()
            .map(|r| (r.get("account_id"), r.get("balance_dong")))
            .collect())
    }

    /// Các posting của bút toán theo key: `(tài khoản, số tiền có dấu)`.
    pub async fn entry_by_key(&self, key: &str) -> Result<Option<Vec<(String, i64)>>, sqlx::Error> {
        let id: Option<i64> =
            sqlx::query_scalar("select id from journal_entries where idempotency_key = $1")
                .bind(key)
                .fetch_optional(&self.pool)
                .await?;
        let Some(id) = id else { return Ok(None) };
        let mut conn = self.pool.acquire().await?;
        Ok(Some(entry_lines_in(&mut conn, id).await?))
    }

    /// Đọc số dư theo loại tài khoản như bài 41: `(bên nợ, bên có)`, đơn vị đồng.
    pub async fn trial_by_kind(&self) -> Result<(i64, i64), sqlx::Error> {
        let rows = sqlx::query(
            "select a.kind, b.balance_dong from account_balances b \
             join accounts a on a.id = b.account_id",
        )
        .fetch_all(&self.pool)
        .await?;
        let (mut debit, mut credit) = (0, 0);
        for r in &rows {
            let kind: String = r.get("kind");
            let balance: i64 = r.get("balance_dong");
            let debit_normal = matches!(kind.as_str(), "asset" | "expense");
            if (balance >= 0) == debit_normal {
                debit += balance.abs();
            } else {
                credit += balance.abs();
            }
        }
        Ok((debit, credit))
    }

    /// Bút toán gốc (không phải bút toán đảo) của một chuyến, mới nhất trước: `(id, key)`.
    pub async fn trip_entries(&self, trip: &str) -> Result<Vec<(i64, String)>, sqlx::Error> {
        let rows = sqlx::query(
            "select id, idempotency_key from journal_entries \
             where idempotency_key like $1 and reversal_of is null order by id desc",
        )
        .bind(format!("trip:{trip}:%"))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .iter()
            .map(|r| (r.get("id"), r.get("idempotency_key")))
            .collect())
    }

    /// Key của bút toán theo mã.
    pub async fn key_of(&self, id: i64) -> Result<String, sqlx::Error> {
        sqlx::query_scalar("select idempotency_key from journal_entries where id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }

    /// Số VíXanh đã thu của chuyến theo bút toán capture; `None` nếu chưa có.
    pub async fn captured_amount(&self, trip: &str) -> Result<Option<Money>, sqlx::Error> {
        let key = format!("trip:{trip}:capture");
        let v: Option<i64> = sqlx::query_scalar(
            "select p.amount_dong from postings p join journal_entries e on e.id = p.entry_id \
             where e.idempotency_key = $1 and p.account_id = 'psp:vixanh:clearing'",
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(v.map(Money::from_dong))
    }

    /// Phần nền tảng đã chịu qua bút toán redeem của chuyến, đã trừ bút toán đảo.
    pub async fn redeemed_amount(&self, trip: &str) -> Result<Money, sqlx::Error> {
        let v: i64 = sqlx::query_scalar(
            "select coalesce(sum(p.amount_dong), 0)::bigint \
             from postings p join journal_entries e on e.id = p.entry_id \
             where e.idempotency_key like $1 and p.account_id = 'platform:promo-expense'",
        )
        .bind(format!("trip:{trip}:promo:%"))
        .fetch_one(&self.pool)
        .await?;
        Ok(Money::from_dong(v))
    }
}

// Kiểm ngân sách trong transaction đang mở. Chỉ có hai khác biệt giữa hai chế độ:
// `for update` và rào chắn của lab.
async fn check_budget(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    spend: &BudgetSpend,
) -> Result<(), PostError> {
    let account = budget_account(&spend.campaign);
    let sql = match &spend.guard {
        BudgetGuard::Locked => {
            "select balance_dong from account_balances where account_id = $1 for update"
        }
        BudgetGuard::Unlocked { .. } => {
            "select balance_dong from account_balances where account_id = $1"
        }
    };
    let remaining: i64 = sqlx::query_scalar(sql)
        .bind(&account)
        .fetch_optional(&mut **tx)
        .await?
        .unwrap_or(0);
    if let BudgetGuard::Unlocked {
        after_read: Some(gate),
    } = &spend.guard
    {
        gate.wait().await; // mọi bên đã đọc xong số cũ rồi mới cùng đi tiếp
    }
    if remaining < spend.amount.dong() {
        return Err(PostError::BudgetExhausted {
            campaign: spend.campaign.clone(),
            requested: spend.amount,
            remaining: Money::from_dong(remaining),
        });
    }
    Ok(())
}

// Posting của bút toán đã ghi, theo thứ tự dòng.
async fn entry_lines_in(
    conn: &mut sqlx::PgConnection,
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
        .map(|r| (r.get("account_id"), r.get("amount_dong")))
        .collect())
}

// (account_id, số tiền có dấu) theo thứ tự posting của bút toán.
fn lines(entry: &JournalEntry) -> Vec<(String, i64)> {
    entry
        .postings()
        .iter()
        .map(|p| (p.account.as_str().to_string(), p.amount.dong()))
        .collect()
}
