//! Ghi bút toán kèm một dòng hash chain và một dòng audit, rồi kiểm tra chuỗi.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use ledger_core::JournalEntry;
use sqlx::{AssertSqlSafe, PgPool, Row};
use tokio::sync::Barrier;

use crate::canonical::{CanonicalEntry, CanonicalPosting, GENESIS, entry_hash};

/// Bảng chứa chuỗi. `Naive` chỉ dùng để dựng cuộc đua làm rẽ nhánh: không khoá, không ràng buộc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainTable {
    /// `entry_chain`: có `PRIMARY KEY (seq)` và `UNIQUE (prev_hash)`.
    Real,
    /// `entry_chain_naive`: cùng cột, không ràng buộc nào.
    Naive,
}

impl ChainTable {
    /// Tên bảng (hằng số, an toàn để ghép vào SQL).
    pub fn name(self) -> &'static str {
        match self {
            ChainTable::Real => "entry_chain",
            ChainTable::Naive => "entry_chain_naive",
        }
    }
}

/// Cách nối vào đầu chuỗi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkMode {
    /// Giữ advisory lock của transaction rồi mới đọc đầu chuỗi: hai bên nối lần lượt.
    Locked,
    /// Đọc đầu chuỗi không khoá: hai bên có thể cùng thấy một đầu.
    Unlocked,
    /// Không nối chuỗi (không đọc đầu, không chèn dòng chuỗi). Chỉ để đo chi phí của chuỗi.
    Skip,
}

/// Bút toán cần ghi: khoá, bút toán bị đảo (nếu có) và các dòng `(tài khoản, số tiền có dấu)`.
#[derive(Debug, Clone)]
pub struct NewEntry {
    /// `idempotency_key`.
    pub key: String,
    /// Mã `journal_entries.id` bị đảo.
    pub reversal_of: Option<i64>,
    /// Các posting theo thứ tự; `line_no` là vị trí từ 1.
    pub postings: Vec<(String, i64)>,
}

impl NewEntry {
    /// Lấy nội dung từ `JournalEntry` của `ledger-core`.
    pub fn from_core(entry: &JournalEntry) -> NewEntry {
        NewEntry {
            key: entry.idempotency_key().to_string(),
            // `JE-000002` -> 2
            reversal_of: entry
                .reverses()
                .and_then(|id| id.to_string()[3..].parse::<i64>().ok()),
            postings: entry
                .postings()
                .iter()
                .map(|p| (p.account.as_str().to_string(), p.amount.dong()))
                .collect(),
        }
    }
}

/// Ngữ cảnh audit của một lần ghi: ai làm, vai trò gì, lý do, ai duyệt.
#[derive(Debug, Clone)]
pub struct AuditCtx {
    /// Mã người hoặc service (`staff:FIN-007`, `svc:ledger`). Chỉ mã, không tên.
    pub actor_id: String,
    /// Vai trò lúc làm.
    pub actor_role: String,
    /// Mã lý do (`TRIP_CAPTURE`, `REFUND_CUSTOMER_REQUEST`).
    pub reason_code: String,
    /// Người duyệt (maker-checker); phải khác `actor_id`.
    pub approved_by: Option<String>,
    /// Mã request để nối với log.
    pub request_id: String,
}

impl AuditCtx {
    /// Ngữ cảnh cho thao tác tự động của service ledger.
    pub fn service(reason_code: &str, request_id: &str) -> AuditCtx {
        AuditCtx {
            actor_id: "svc:ledger".to_string(),
            actor_role: "ledger_writer".to_string(),
            reason_code: reason_code.to_string(),
            approved_by: None,
            request_id: request_id.to_string(),
        }
    }
}

/// Tuỳ chọn ghi. Hai rào `Barrier` chỉ để test dựng cuộc đua; production để `None`.
#[derive(Clone)]
pub struct AppendOptions {
    /// Cách nối vào đầu chuỗi.
    pub mode: LinkMode,
    /// Bảng chuỗi.
    pub table: ChainTable,
    /// Các bên đợi nhau ở đây, sau khi đã chèn bút toán và trước khi xin khoá.
    pub before_link: Option<Arc<Barrier>>,
    /// Các bên đợi nhau ở đây, sau khi đã đọc đầu chuỗi và trước khi chèn dòng chuỗi.
    pub after_tip: Option<Arc<Barrier>>,
}

impl Default for AppendOptions {
    fn default() -> AppendOptions {
        AppendOptions {
            mode: LinkMode::Locked,
            table: ChainTable::Real,
            before_link: None,
            after_tip: None,
        }
    }
}

/// Kết quả ghi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posted {
    /// `journal_entries.id`.
    pub id: i64,
    /// Vị trí trong chuỗi; `None` nếu là ghi lại cùng khoá (không thêm dòng chuỗi).
    pub seq: Option<i64>,
    /// `entry_hash`; `None` nếu ghi lại.
    pub entry_hash: Option<[u8; 32]>,
}

impl Posted {
    /// Mã hiển thị `JE-000001`.
    pub fn display_id(&self) -> String {
        format!("JE-{:06}", self.id)
    }
}

/// Khoá advisory theo schema: advisory lock là của cả database, nên phải gắn tên schema vào khoá,
/// nếu không hai ca lab (hai schema) sẽ chờ nhau vô cớ.
pub const LOCK_SQL: &str =
    "SELECT pg_advisory_xact_lock(hashtextextended(current_schema() || ':entry_chain', 0))";

const AUDIT_SQL: &str = "INSERT INTO audit_log \
    (actor_id, actor_role, action, target, reason_code, approved_by, request_id) \
    VALUES ($1, $2, 'entry.post', $3, $4, $5, $6)";

/// Ghi một bút toán, một dòng chuỗi và một dòng audit trong CÙNG một transaction.
///
/// # Errors
/// Mọi lỗi sqlx, kể cả `23505` khi hai bên cùng nối vào một đầu chuỗi (chỉ xảy ra ở `Unlocked`).
pub async fn append(
    pool: &PgPool,
    entry: &NewEntry,
    ctx: &AuditCtx,
    opts: &AppendOptions,
) -> Result<Posted, sqlx::Error> {
    let mut tx = pool.begin().await?;

    // 1. Bút toán. Trùng khoá thì không ghi gì thêm (bài 24 đã so nội dung; lab này bỏ phần đó).
    let inserted = sqlx::query(
        "INSERT INTO journal_entries (idempotency_key, reversal_of) VALUES ($1, $2) \
         ON CONFLICT (idempotency_key) DO NOTHING \
         RETURNING id, (extract(epoch FROM posted_at) * 1000000)::bigint AS micros",
    )
    .bind(&entry.key)
    .bind(entry.reversal_of)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(row) = inserted else {
        let id: i64 =
            sqlx::query_scalar("SELECT id FROM journal_entries WHERE idempotency_key = $1")
                .bind(&entry.key)
                .fetch_one(&mut *tx)
                .await?;
        return Ok(Posted {
            id,
            seq: None,
            entry_hash: None,
        });
    };
    let id: i64 = row.get("id");
    let micros: i64 = row.get("micros");

    // 2. Posting.
    let mut canonical_postings = Vec::with_capacity(entry.postings.len());
    for (i, (account, amount)) in entry.postings.iter().enumerate() {
        let line_no = i32::try_from(i + 1).expect("ít hơn 2^31 posting");
        sqlx::query(
            "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(line_no)
        .bind(account)
        .bind(amount)
        .execute(&mut *tx)
        .await?;
        canonical_postings.push(CanonicalPosting {
            line_no,
            account_id: account.clone(),
            amount_dong: *amount,
        });
    }
    let canonical = CanonicalEntry {
        entry_id: id,
        idempotency_key: entry.key.clone(),
        reversal_of: entry.reversal_of,
        posted_at_micros: micros,
        postings: canonical_postings,
    }
    .to_bytes();

    if let Some(barrier) = &opts.before_link {
        barrier.wait().await;
    }

    // Skip: chỉ để đo chi phí của chuỗi, bỏ qua bước 3.
    if opts.mode == LinkMode::Skip {
        sqlx::query(AUDIT_SQL)
            .bind(&ctx.actor_id)
            .bind(&ctx.actor_role)
            .bind(format!("JE-{id:06}"))
            .bind(&ctx.reason_code)
            .bind(&ctx.approved_by)
            .bind(&ctx.request_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        return Ok(Posted {
            id,
            seq: None,
            entry_hash: None,
        });
    }

    // 3. Điểm tuần tự hoá: từ đây tới COMMIT chỉ một transaction được nối vào chuỗi.
    //    Xin khoá muộn nhất có thể để phần phải xếp hàng ngắn nhất.
    if opts.mode == LinkMode::Locked {
        sqlx::query(LOCK_SQL).execute(&mut *tx).await?;
    }
    // READ COMMITTED: mỗi câu lệnh có snapshot mới, nên sau khi có khoá ta thấy dòng chuỗi
    // mà transaction trước vừa commit. Ở REPEATABLE READ snapshot cũ sẽ làm hỏng điều này.
    let table = opts.table.name();
    let tip = sqlx::query(AssertSqlSafe(format!(
        "SELECT seq, entry_hash FROM {table} ORDER BY seq DESC, entry_id DESC LIMIT 1"
    )))
    .fetch_optional(&mut *tx)
    .await?;
    let (seq, prev): (i64, [u8; 32]) = match tip {
        None => (1, GENESIS),
        Some(r) => {
            let bytes: Vec<u8> = r.get("entry_hash");
            let prev = <[u8; 32]>::try_from(bytes.as_slice())
                .map_err(|_| sqlx::Error::Decode("entry_hash không đủ 32 byte".into()))?;
            (r.get::<i64, _>("seq") + 1, prev)
        }
    };
    if let Some(barrier) = &opts.after_tip {
        barrier.wait().await;
    }
    let hash = entry_hash(&prev, &canonical);
    sqlx::query(AssertSqlSafe(format!(
        "INSERT INTO {table} (seq, entry_id, prev_hash, entry_hash) VALUES ($1, $2, $3, $4)"
    )))
    .bind(seq)
    .bind(id)
    .bind(prev.as_slice())
    .bind(hash.as_slice())
    .execute(&mut *tx)
    .await?;

    // 4. Audit: ai làm gì, cùng transaction nên không có bút toán nào thiếu dòng audit.
    sqlx::query(AUDIT_SQL)
        .bind(&ctx.actor_id)
        .bind(&ctx.actor_role)
        .bind(format!("JE-{id:06}"))
        .bind(&ctx.reason_code)
        .bind(&ctx.approved_by)
        .bind(&ctx.request_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(Posted {
        id,
        seq: Some(seq),
        entry_hash: Some(hash),
    })
}

/// Loại sai lệch tìm thấy khi kiểm tra chuỗi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultKind {
    /// `seq` không phải số kế tiếp (thiếu dòng, trùng `seq`, đảo thứ tự).
    SeqMismatch,
    /// Dòng chuỗi trỏ tới bút toán không còn.
    MissingEntry,
    /// `prev_hash` khác `entry_hash` của dòng liền trước.
    PrevMismatch,
    /// `entry_hash` lưu khác giá trị tính lại từ nội dung bút toán.
    HashMismatch,
    /// Bút toán có trong `journal_entries` nhưng không có dòng chuỗi.
    Unchained,
}

/// Dòng đầu tiên sai.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault {
    /// `seq` ghi trên dòng chuỗi; `None` với `Unchained`.
    pub seq: Option<i64>,
    /// Bút toán liên quan.
    pub entry_id: i64,
    /// Loại sai lệch.
    pub kind: FaultKind,
}

/// Điểm neo: cặp `(seq, entry_hash)` của đầu chuỗi, công bố ra ngoài database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anchor {
    /// Vị trí đầu chuỗi lúc neo.
    pub seq: i64,
    /// Hash của đầu chuỗi lúc neo.
    pub hash: [u8; 32],
}

impl Anchor {
    /// Dạng một dòng để lưu ra ngoài: `seq:hex`.
    pub fn to_line(&self) -> String {
        format!("{}:{}", self.seq, hex::encode(self.hash))
    }
}

/// Kết quả kiểm tra chuỗi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainReport {
    /// Số dòng chuỗi đã duyệt cho tới khi gặp lỗi (hoặc hết chuỗi).
    pub checked: u64,
    /// Dòng đầu tiên sai, nếu có.
    pub first_fault: Option<Fault>,
}

impl ChainReport {
    /// Chuỗi nguyên vẹn?
    pub fn is_ok(&self) -> bool {
        self.first_fault.is_none()
    }
}

/// Đọc toàn bộ bút toán từ `journal_entries` + `postings` về dạng chuẩn, theo `entry_id`.
///
/// # Errors
/// Lỗi sqlx.
pub async fn load_entries(pool: &PgPool) -> Result<BTreeMap<i64, CanonicalEntry>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT e.id, e.idempotency_key, e.reversal_of, \
                (extract(epoch FROM e.posted_at) * 1000000)::bigint AS micros, \
                p.line_no, p.account_id, p.amount_dong \
         FROM journal_entries e LEFT JOIN postings p ON p.entry_id = e.id \
         ORDER BY e.id, p.line_no",
    )
    .fetch_all(pool)
    .await?;
    let mut map: BTreeMap<i64, CanonicalEntry> = BTreeMap::new();
    for r in rows {
        let id: i64 = r.get("id");
        let entry = map.entry(id).or_insert_with(|| CanonicalEntry {
            entry_id: id,
            idempotency_key: r.get("idempotency_key"),
            reversal_of: r.get("reversal_of"),
            posted_at_micros: r.get("micros"),
            postings: Vec::new(),
        });
        if let Some(line_no) = r.get::<Option<i32>, _>("line_no") {
            entry.postings.push(CanonicalPosting {
                line_no,
                account_id: r.get("account_id"),
                amount_dong: r.get("amount_dong"),
            });
        }
    }
    Ok(map)
}

fn to_hash(bytes: &[u8]) -> Option<[u8; 32]> {
    <[u8; 32]>::try_from(bytes).ok()
}

/// Kiểm tra chuỗi từ đầu: tính lại hash từng bút toán từ nội dung đang có trong bảng, so với giá
/// trị đã lưu, và trả về dòng đầu tiên sai. Chỉ đọc; không sửa gì.
///
/// # Errors
/// Lỗi sqlx.
pub async fn verify_chain(pool: &PgPool, table: ChainTable) -> Result<ChainReport, sqlx::Error> {
    let name = table.name();
    let rows = sqlx::query(AssertSqlSafe(format!(
        "SELECT seq, entry_id, prev_hash, entry_hash FROM {name} ORDER BY seq, entry_id"
    )))
    .fetch_all(pool)
    .await?;
    let entries = load_entries(pool).await?;

    let mut prev = GENESIS;
    let mut checked = 0_u64;
    let mut seen: BTreeSet<i64> = BTreeSet::new();
    // `expected_seq` đếm từ 1: dòng thứ k phải mang seq = k.
    for (expected_seq, r) in (1_i64..).zip(&rows) {
        let seq: i64 = r.get("seq");
        let entry_id: i64 = r.get("entry_id");
        let fault = |kind| ChainReport {
            checked,
            first_fault: Some(Fault {
                seq: Some(seq),
                entry_id,
                kind,
            }),
        };
        let stored_prev = to_hash(&r.get::<Vec<u8>, _>("prev_hash"));
        let stored_hash = to_hash(&r.get::<Vec<u8>, _>("entry_hash"));
        if seq != expected_seq {
            return Ok(fault(FaultKind::SeqMismatch));
        }
        if stored_prev != Some(prev) {
            return Ok(fault(FaultKind::PrevMismatch));
        }
        let Some(entry) = entries.get(&entry_id) else {
            return Ok(fault(FaultKind::MissingEntry));
        };
        // `prev` là entry_hash ĐÃ LƯU của dòng trước (không phải giá trị tính lại), nên lỗi ở dòng
        // nào báo đúng dòng đó, không lan sang các dòng sau.
        if Some(entry_hash(&prev, &entry.to_bytes())) != stored_hash {
            return Ok(fault(FaultKind::HashMismatch));
        }
        prev = stored_hash.unwrap_or(GENESIS);
        checked += 1;
        seen.insert(entry_id);
    }
    if let Some(id) = entries.keys().find(|id| !seen.contains(id)) {
        return Ok(ChainReport {
            checked,
            first_fault: Some(Fault {
                seq: None,
                entry_id: *id,
                kind: FaultKind::Unchained,
            }),
        });
    }
    Ok(ChainReport {
        checked,
        first_fault: None,
    })
}

/// Đầu chuỗi hiện tại làm điểm neo; `None` nếu chuỗi rỗng.
///
/// # Errors
/// Lỗi sqlx.
pub async fn current_anchor(pool: &PgPool) -> Result<Option<Anchor>, sqlx::Error> {
    let row = sqlx::query("SELECT seq, entry_hash FROM entry_chain ORDER BY seq DESC LIMIT 1")
        .fetch_optional(pool)
        .await?;
    Ok(row.and_then(|r| {
        Some(Anchor {
            seq: r.get("seq"),
            hash: to_hash(&r.get::<Vec<u8>, _>("entry_hash"))?,
        })
    }))
}

/// Kết quả đối chiếu với điểm neo đã công bố.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnchorCheck {
    /// Dòng `seq` của neo vẫn có đúng hash đó.
    Match,
    /// Dòng đó có nhưng hash khác: nội dung chuỗi đã bị viết lại.
    Mismatch,
    /// Không còn dòng nào ở `seq` đó: đuôi chuỗi đã bị cắt.
    Missing,
}

/// So điểm neo (lưu ở nơi kẻ tấn công không với tới) với chuỗi hiện tại.
///
/// # Errors
/// Lỗi sqlx.
pub async fn check_anchor(pool: &PgPool, anchor: &Anchor) -> Result<AnchorCheck, sqlx::Error> {
    let row = sqlx::query("SELECT entry_hash FROM entry_chain WHERE seq = $1")
        .bind(anchor.seq)
        .fetch_optional(pool)
        .await?;
    Ok(match row {
        None => AnchorCheck::Missing,
        Some(r) if to_hash(&r.get::<Vec<u8>, _>("entry_hash")) == Some(anchor.hash) => {
            AnchorCheck::Match
        }
        Some(_) => AnchorCheck::Mismatch,
    })
}

/// Kẻ có toàn quyền: tắt trigger, tính lại TOÀN BỘ chuỗi từ nội dung hiện có rồi ghi đè.
/// Có trong lab để chứng minh `verify_chain` không phát hiện được, còn điểm neo thì có.
///
/// # Errors
/// Lỗi sqlx. Cần kết nối superuser (pool `admin` của lab).
pub async fn rewrite_chain_as_superuser(admin: &PgPool) -> Result<(), sqlx::Error> {
    let entries = load_entries(admin).await?;
    let rows = sqlx::query("SELECT seq, entry_id FROM entry_chain ORDER BY seq")
        .fetch_all(admin)
        .await?;
    let mut conn = admin.acquire().await?;
    sqlx::query("ALTER TABLE entry_chain DISABLE TRIGGER USER")
        .execute(&mut *conn)
        .await?;
    let mut prev = GENESIS;
    for r in rows {
        let seq: i64 = r.get("seq");
        let entry_id: i64 = r.get("entry_id");
        let Some(entry) = entries.get(&entry_id) else {
            continue;
        };
        let hash = entry_hash(&prev, &entry.to_bytes());
        sqlx::query("UPDATE entry_chain SET prev_hash = $1, entry_hash = $2 WHERE seq = $3")
            .bind(prev.as_slice())
            .bind(hash.as_slice())
            .bind(seq)
            .execute(&mut *conn)
            .await?;
        prev = hash;
    }
    sqlx::query("ALTER TABLE entry_chain ENABLE TRIGGER USER")
        .execute(&mut *conn)
        .await?;
    Ok(())
}
