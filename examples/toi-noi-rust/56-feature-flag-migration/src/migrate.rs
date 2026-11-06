//! Expand/contract: các bước DDL và backfill, mỗi bước có thể quan sát khóa thật qua `pg_locks`.
//!
//! Quy trình: `expand` (thêm cột nullable, trigger "ghi hộ" cho code cũ) -> triển khai code mới
//! -> `backfill_batch` theo lô có checkpoint -> `contract_a` (NOT NULL an toàn, còn quay lại được)
//! -> gỡ code cũ, đổi code mới sang chỉ ghi cột mới -> `contract_b` (bỏ cột cũ và trigger,
//! không quay lại được nếu không dựng lại dữ liệu).

use std::future::Future;

use sqlx::{AssertSqlSafe, PgConnection, PgPool, Row};

use crate::app::sqlstate;

/// SQLSTATE của "lock_not_available": hết `lock_timeout` khi chờ khóa.
pub const LOCK_NOT_AVAILABLE: &str = "55P03";

/// Một câu DDL, kèm bảng cần quan sát khóa (nếu có).
#[derive(Debug, Clone, Copy)]
pub struct Step {
    /// Tên hiển thị.
    pub label: &'static str,
    /// Câu SQL (hằng, không ghép dữ liệu ngoài).
    pub sql: &'static str,
    /// Bảng để đọc `pg_locks` và số lần quét sau câu này.
    pub rel: Option<&'static str>,
}

/// Quan sát sau một bước: khóa MỚI mà bước này lấy trên bảng, và số lần quét tuần tự do chính
/// bước này gây ra (hiệu của `pg_stat_get_xact_numscans` trước và sau câu lệnh).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Probed {
    /// Tên bước.
    pub label: &'static str,
    /// Bảng được quan sát.
    pub rel: Option<&'static str>,
    /// Chế độ khóa mới giữ trên bảng sau bước này.
    pub modes: Vec<String>,
    /// Số lần quét của bảng do bước này gây ra.
    pub scans: i64,
}

const fn step(label: &'static str, sql: &'static str, rel: Option<&'static str>) -> Step {
    Step { label, sql, rel }
}

/// Bước expand: chỉ thêm, không bỏ gì. Cột nullable không mặc định chỉ đổi catalog, không viết lại bảng.
pub const EXPAND: [Step; 7] = [
    step(
        "ADD COLUMN postings.currency (nullable)",
        "ALTER TABLE postings ADD COLUMN currency text",
        Some("postings"),
    ),
    step(
        "hàm điền currency cho code cũ",
        "CREATE FUNCTION postings_fill_currency() RETURNS trigger LANGUAGE plpgsql AS $$ \
         BEGIN NEW.currency := COALESCE(NEW.currency, 'VND'); RETURN NEW; END; $$",
        None,
    ),
    step(
        "CREATE TRIGGER postings_fill_currency",
        "CREATE TRIGGER postings_fill_currency BEFORE INSERT ON postings \
         FOR EACH ROW EXECUTE FUNCTION postings_fill_currency()",
        Some("postings"),
    ),
    // Chỉ cho phép đúng một kiểu UPDATE: điền currency từ NULL sang 'VND', bốn cột gốc giữ nguyên.
    step(
        "nới postings_guard: chỉ cho điền currency",
        "CREATE OR REPLACE FUNCTION postings_guard() RETURNS trigger LANGUAGE plpgsql AS $$ \
         BEGIN \
           IF TG_OP = 'UPDATE' AND OLD.currency IS NULL AND NEW.currency = 'VND' \
              AND (OLD.entry_id, OLD.line_no, OLD.account_id, OLD.amount_dong) \
                  IS NOT DISTINCT FROM (NEW.entry_id, NEW.line_no, NEW.account_id, NEW.amount_dong) \
           THEN RETURN NEW; END IF; \
           RAISE EXCEPTION 'sổ cái append-only: % trên postings bị cấm', TG_OP USING ERRCODE = 'TN001'; \
         END; $$",
        None,
    ),
    step(
        "ADD COLUMN fee_events.policy_version (nullable)",
        "ALTER TABLE fee_events ADD COLUMN policy_version int",
        Some("fee_events"),
    ),
    step(
        "hàm điền policy_version cho code cũ",
        "CREATE FUNCTION fee_events_fill_version() RETURNS trigger LANGUAGE plpgsql AS $$ \
         BEGIN NEW.policy_version := COALESCE(NEW.policy_version, 1); RETURN NEW; END; $$",
        None,
    ),
    step(
        "CREATE TRIGGER fee_events_fill_version",
        "CREATE TRIGGER fee_events_fill_version BEFORE INSERT ON fee_events \
         FOR EACH ROW EXECUTE FUNCTION fee_events_fill_version()",
        Some("fee_events"),
    ),
];

/// Thêm ràng buộc `NOT VALID`: không quét bảng, chỉ áp cho dòng ghi từ giờ.
pub const ADD_CHECKS: [Step; 2] = [
    step(
        "ADD CONSTRAINT postings_currency_nn CHECK (...) NOT VALID",
        "ALTER TABLE postings ADD CONSTRAINT postings_currency_nn \
         CHECK (currency IS NOT NULL) NOT VALID",
        Some("postings"),
    ),
    step(
        "ADD CONSTRAINT fee_events_version_nn CHECK (...) NOT VALID",
        "ALTER TABLE fee_events ADD CONSTRAINT fee_events_version_nn \
         CHECK (policy_version IS NOT NULL) NOT VALID",
        Some("fee_events"),
    ),
];

/// `VALIDATE` quét bảng nhưng chỉ lấy khóa không chặn ghi.
pub const VALIDATE_POSTINGS: Step = step(
    "VALIDATE CONSTRAINT postings_currency_nn",
    "ALTER TABLE postings VALIDATE CONSTRAINT postings_currency_nn",
    Some("postings"),
);
/// Như trên, cho `fee_events`.
pub const VALIDATE_EVENTS: Step = step(
    "VALIDATE CONSTRAINT fee_events_version_nn",
    "ALTER TABLE fee_events VALIDATE CONSTRAINT fee_events_version_nn",
    Some("fee_events"),
);

/// `SET NOT NULL` khi đã có CHECK hợp lệ chứng minh không còn NULL: PostgreSQL bỏ qua bước quét.
pub const SET_NOT_NULL_POSTINGS: Step = step(
    "SET NOT NULL postings.currency",
    "ALTER TABLE postings ALTER COLUMN currency SET NOT NULL",
    Some("postings"),
);
/// Như trên, cho `fee_events.policy_version`.
pub const SET_NOT_NULL_EVENTS: Step = step(
    "SET NOT NULL fee_events.policy_version",
    "ALTER TABLE fee_events ALTER COLUMN policy_version SET NOT NULL",
    Some("fee_events"),
);

const FINISH_A: [Step; 4] = [
    step(
        "DROP CONSTRAINT postings_currency_nn",
        "ALTER TABLE postings DROP CONSTRAINT postings_currency_nn",
        Some("postings"),
    ),
    step(
        "DROP CONSTRAINT fee_events_version_nn",
        "ALTER TABLE fee_events DROP CONSTRAINT fee_events_version_nn",
        Some("fee_events"),
    ),
    // Cột cũ không còn bắt buộc: code mới (NewOnly) sẽ ngừng ghi nó.
    step(
        "nới NOT NULL của fee_events.fee_percent",
        "ALTER TABLE fee_events ALTER COLUMN fee_percent DROP NOT NULL",
        Some("fee_events"),
    ),
    // Backfill xong thì trả lại bất biến chặt: không còn UPDATE nào được phép.
    step(
        "trả postings_guard về append-only chặt",
        "CREATE OR REPLACE FUNCTION postings_guard() RETURNS trigger LANGUAGE plpgsql AS $$ \
         BEGIN RAISE EXCEPTION 'sổ cái append-only: % trên postings bị cấm', TG_OP \
         USING ERRCODE = 'TN001'; END; $$",
        None,
    ),
];

/// Bước contract không thể quay lại nếu không dựng lại dữ liệu: bỏ trigger ghi hộ và cột cũ.
pub const CONTRACT_B: [Step; 5] = [
    step(
        "DROP TRIGGER postings_fill_currency",
        "DROP TRIGGER postings_fill_currency ON postings",
        Some("postings"),
    ),
    step(
        "DROP FUNCTION postings_fill_currency",
        "DROP FUNCTION postings_fill_currency()",
        None,
    ),
    step(
        "DROP TRIGGER fee_events_fill_version",
        "DROP TRIGGER fee_events_fill_version ON fee_events",
        Some("fee_events"),
    ),
    step(
        "DROP FUNCTION fee_events_fill_version",
        "DROP FUNCTION fee_events_fill_version()",
        None,
    ),
    step(
        "DROP COLUMN fee_events.fee_percent",
        "ALTER TABLE fee_events DROP COLUMN fee_percent",
        Some("fee_events"),
    ),
];

/// Dựng lại đường lùi sau `contract_b`: trigger ghi hộ và cột cũ (nullable, rỗng; dữ liệu cột
/// cũ KHÔNG tự quay lại, phải suy ra lại bằng [`FEE_PERCENT_FROM_VERSION`]).
pub const UNCONTRACT_B: [Step; 5] = [
    EXPAND[1],
    EXPAND[2],
    EXPAND[5],
    EXPAND[6],
    step(
        "ADD COLUMN fee_events.fee_percent (nullable, rỗng)",
        "ALTER TABLE fee_events ADD COLUMN fee_percent int",
        Some("fee_events"),
    ),
];

/// Một lần backfill: bảng, cột khóa tăng dần, vế gán và cột `NULL` cần điền.
#[derive(Debug, Clone, Copy)]
pub struct BackfillSpec {
    /// Tên checkpoint trong `migration_checkpoints`.
    pub name: &'static str,
    /// Bảng.
    pub table: &'static str,
    /// Cột khóa, tăng theo thứ tự ghi.
    pub key: &'static str,
    /// Vế gán của `UPDATE ... SET`.
    pub assign: &'static str,
    /// Cột đang `NULL` cần điền; điều kiện `IS NULL` làm lô idempotent.
    pub null_col: &'static str,
}

/// `postings.currency = 'VND'`, theo `entry_id`.
pub const POSTINGS_CURRENCY: BackfillSpec = BackfillSpec {
    name: "postings.currency",
    table: "postings",
    key: "entry_id",
    assign: "currency = 'VND'",
    null_col: "currency",
};

/// `fee_events.policy_version = 1` (mọi chuyến trước đây dùng quy tắc duy nhất), theo `seq`.
pub const EVENTS_VERSION: BackfillSpec = BackfillSpec {
    name: "fee_events.policy_version",
    table: "fee_events",
    key: "seq",
    assign: "policy_version = 1",
    null_col: "policy_version",
};

/// Suy lại cột cũ từ cột mới khi cần lùi: tỷ lệ danh nghĩa của từng phiên bản.
pub const FEE_PERCENT_FROM_VERSION: BackfillSpec = BackfillSpec {
    name: "fee_events.fee_percent",
    table: "fee_events",
    key: "seq",
    assign: "fee_percent = CASE policy_version WHEN 1 THEN 20 WHEN 2 THEN 15 END",
    null_col: "fee_percent",
};

/// Kết quả một lô backfill.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Batch {
    /// Số dòng thật sự được cập nhật (dòng đã có giá trị thì không tính).
    pub rows: u64,
    /// Checkpoint sau lô.
    pub checkpoint: i64,
}

/// Checkpoint hiện tại của một backfill (0 nếu chưa chạy).
///
/// # Errors
/// Lỗi SQL.
pub async fn checkpoint(pool: &PgPool, spec: &BackfillSpec) -> Result<i64, sqlx::Error> {
    let last: Option<i64> =
        sqlx::query_scalar("SELECT last_key FROM migration_checkpoints WHERE name = $1")
            .bind(spec.name)
            .fetch_optional(pool)
            .await?;
    Ok(last.unwrap_or(0))
}

/// Chạy MỘT lô: lấy `batch_size` khóa kế tiếp sau checkpoint, điền dòng còn `NULL`, dời checkpoint.
/// Điền và dời checkpoint cùng một transaction nên chết giữa chừng thì không mất vị trí và không
/// làm dở. `None` khi không còn khóa nào sau checkpoint.
///
/// # Errors
/// Lỗi SQL.
pub async fn backfill_batch(
    pool: &PgPool,
    spec: &BackfillSpec,
    batch_size: i64,
) -> Result<Option<Batch>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO migration_checkpoints (name) VALUES ($1) ON CONFLICT (name) DO NOTHING",
    )
    .bind(spec.name)
    .execute(&mut *tx)
    .await?;
    // Khóa dòng checkpoint: hai tiến trình backfill cùng tên xếp hàng, không giẫm lên nhau.
    let last: i64 =
        sqlx::query_scalar("SELECT last_key FROM migration_checkpoints WHERE name = $1 FOR UPDATE")
            .bind(spec.name)
            .fetch_one(&mut *tx)
            .await?;

    // Tên bảng và cột lấy từ hằng của lab, không từ dữ liệu ngoài.
    let upper_sql = format!(
        "SELECT max(k) FROM (SELECT DISTINCT {key} AS k FROM {table} WHERE {key} > $1 \
         ORDER BY k LIMIT $2) AS s",
        key = spec.key,
        table = spec.table
    );
    let upper: Option<i64> = sqlx::query_scalar(AssertSqlSafe(upper_sql))
        .bind(last)
        .bind(batch_size)
        .fetch_one(&mut *tx)
        .await?;
    let Some(upper) = upper else {
        return Ok(None);
    };

    let update_sql = format!(
        "UPDATE {table} SET {assign} WHERE {key} > $1 AND {key} <= $2 AND {null_col} IS NULL",
        table = spec.table,
        assign = spec.assign,
        key = spec.key,
        null_col = spec.null_col
    );
    let rows = sqlx::query(AssertSqlSafe(update_sql))
        .bind(last)
        .bind(upper)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    sqlx::query(
        "UPDATE migration_checkpoints SET last_key = $2, updated_at = now() WHERE name = $1",
    )
    .bind(spec.name)
    .bind(upper)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Some(Batch {
        rows,
        checkpoint: upper,
    }))
}

/// Chế độ khóa quan hệ mà backend này đang giữ trên `rel` (tên không kèm schema, tìm theo `search_path`).
///
/// # Errors
/// Lỗi SQL.
pub async fn held_modes(conn: &mut PgConnection, rel: &str) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT mode FROM pg_locks WHERE locktype = 'relation' AND granted \
         AND pid = pg_backend_pid() AND relation = to_regclass($1) ORDER BY mode",
    )
    .bind(rel)
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows.iter().map(|r| r.get("mode")).collect())
}

/// Bộ đếm quét của `rel` của backend này. Nó gồm cả phần chưa được đẩy lên thống kê chung từ các
/// transaction trước trên cùng kết nối, nên chỉ có nghĩa khi lấy HIỆU trước và sau một câu lệnh.
///
/// # Errors
/// Lỗi SQL.
pub async fn xact_scans(conn: &mut PgConnection, rel: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COALESCE(pg_stat_get_xact_numscans(to_regclass($1)), 0)::bigint")
        .bind(rel)
        .fetch_one(&mut *conn)
        .await
}

/// Chạy các bước trong MỘT transaction với `lock_timeout`, quan sát khóa mới và số lần quét
/// sau mỗi bước. Hết thời gian chờ khóa thì lỗi `55P03` và cả transaction được hoàn tác.
///
/// # Errors
/// Lỗi SQL của bất kỳ bước nào.
pub async fn run_tx(
    pool: &PgPool,
    lock_timeout_ms: u32,
    steps: &[Step],
) -> Result<Vec<Probed>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    // Số nguyên do ta truyền, không phải dữ liệu ngoài.
    sqlx::query(AssertSqlSafe(format!(
        "SET LOCAL lock_timeout = '{lock_timeout_ms}ms'"
    )))
    .execute(&mut *tx)
    .await?;
    let mut probes = Vec::new();
    for step in steps {
        let (before, scans_before) = match step.rel {
            Some(rel) => (
                held_modes(&mut tx, rel).await?,
                xact_scans(&mut tx, rel).await?,
            ),
            None => (Vec::new(), 0),
        };
        sqlx::query(step.sql).execute(&mut *tx).await?;
        let (modes, scans) = match step.rel {
            Some(rel) => {
                let after = held_modes(&mut tx, rel).await?;
                let added = after.into_iter().filter(|m| !before.contains(m)).collect();
                (added, xact_scans(&mut tx, rel).await? - scans_before)
            }
            None => (Vec::new(), 0),
        };
        probes.push(Probed {
            label: step.label,
            rel: step.rel,
            modes,
            scans,
        });
    }
    tx.commit().await?;
    Ok(probes)
}

/// Thử `attempt` tối đa `max_attempts` lần, chỉ thử lại khi lỗi là hết `lock_timeout` (`55P03`).
/// Trả giá trị và số lần đã thử. Lỗi khác, hoặc hết lượt, trả về ngay.
///
/// # Errors
/// Lỗi cuối cùng.
pub async fn retry_on_lock_timeout<T, F, Fut>(
    max_attempts: u32,
    mut attempt: F,
) -> Result<(T, u32), sqlx::Error>
where
    F: FnMut(u32) -> Fut,
    Fut: Future<Output = Result<T, sqlx::Error>>,
{
    let mut n = 1;
    loop {
        match attempt(n).await {
            Ok(value) => return Ok((value, n)),
            Err(err)
                if n < max_attempts && sqlstate(&err).as_deref() == Some(LOCK_NOT_AVAILABLE) =>
            {
                n += 1;
            }
            Err(err) => return Err(err),
        }
    }
}

/// Expand trong một transaction.
///
/// # Errors
/// Lỗi SQL, hoặc `55P03` nếu hết thời gian chờ khóa.
pub async fn expand(pool: &PgPool, lock_timeout_ms: u32) -> Result<Vec<Probed>, sqlx::Error> {
    run_tx(pool, lock_timeout_ms, &EXPAND).await
}

/// Contract giai đoạn A: NOT NULL an toàn cho hai cột mới, nới cột cũ, trả append-only chặt.
/// Còn quay lại được. Gọi khi không còn dòng `NULL` (kiểm bằng `count_nulls`); còn thì
/// `VALIDATE` lỗi `23514` và không đổi gì.
///
/// # Errors
/// Lỗi SQL, `55P03` hoặc `23514` (còn dòng chưa backfill).
pub async fn contract_a(pool: &PgPool, lock_timeout_ms: u32) -> Result<Vec<Probed>, sqlx::Error> {
    let mut probes = run_tx(pool, lock_timeout_ms, &ADD_CHECKS).await?;
    probes.extend(run_tx(pool, lock_timeout_ms, &[VALIDATE_POSTINGS]).await?);
    probes.extend(run_tx(pool, lock_timeout_ms, &[VALIDATE_EVENTS]).await?);
    probes.extend(finish_a(pool, lock_timeout_ms).await?);
    Ok(probes)
}

/// Phần cuối của contract A, sau khi cả hai ràng buộc đã `VALIDATE`: `SET NOT NULL` (không quét),
/// bỏ ràng buộc tạm, nới cột cũ, trả append-only chặt. Cả bốn việc trong một transaction ngắn.
///
/// # Errors
/// Lỗi SQL hoặc `55P03`.
pub async fn finish_a(pool: &PgPool, lock_timeout_ms: u32) -> Result<Vec<Probed>, sqlx::Error> {
    let mut finish = vec![SET_NOT_NULL_POSTINGS, SET_NOT_NULL_EVENTS];
    finish.extend(FINISH_A);
    run_tx(pool, lock_timeout_ms, &finish).await
}

/// Chạy backfill từng lô tới hết, trả các lô đã chạy.
///
/// # Errors
/// Lỗi SQL.
pub async fn backfill_all(
    pool: &PgPool,
    spec: &BackfillSpec,
    batch_size: i64,
) -> Result<Vec<Batch>, sqlx::Error> {
    let mut batches = Vec::new();
    while let Some(batch) = backfill_batch(pool, spec, batch_size).await? {
        batches.push(batch);
    }
    Ok(batches)
}

/// Chuyển hết, không dữ liệu giữa chừng: expand, backfill, contract A rồi B. Dùng để dựng
/// schema cuối cho các thí nghiệm không quan tâm tới đoạn chuyển.
///
/// # Errors
/// Lỗi SQL.
pub async fn migrate_all(
    pool: &PgPool,
    batch_size: i64,
    lock_timeout_ms: u32,
) -> Result<(), sqlx::Error> {
    expand(pool, lock_timeout_ms).await?;
    backfill_all(pool, &POSTINGS_CURRENCY, batch_size).await?;
    backfill_all(pool, &EVENTS_VERSION, batch_size).await?;
    contract_a(pool, lock_timeout_ms).await?;
    contract_b(pool, lock_timeout_ms).await?;
    Ok(())
}

/// Contract giai đoạn B: bỏ trigger ghi hộ và cột cũ. Chỉ gọi khi không còn code cũ.
///
/// # Errors
/// Lỗi SQL hoặc `55P03`.
pub async fn contract_b(pool: &PgPool, lock_timeout_ms: u32) -> Result<Vec<Probed>, sqlx::Error> {
    run_tx(pool, lock_timeout_ms, &CONTRACT_B).await
}

/// Dựng lại đường lùi sau `contract_b` (xem [`UNCONTRACT_B`]).
///
/// # Errors
/// Lỗi SQL hoặc `55P03`.
pub async fn uncontract_b(pool: &PgPool, lock_timeout_ms: u32) -> Result<Vec<Probed>, sqlx::Error> {
    run_tx(pool, lock_timeout_ms, &UNCONTRACT_B).await
}
