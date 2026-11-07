//! Các thí nghiệm của bài 56. Mỗi hàm chạy trong một `Lab` (schema riêng), trả về một báo cáo
//! có `verify()` kiểm các giá trị bài viết nêu. Không thí nghiệm nào assert thời gian: chờ khóa
//! được quan sát qua `pg_stat_activity` và `pg_locks`, không đoán bằng `sleep`.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail, ensure};
use sqlx::{PgConnection, PgPool, Row};
use tokio::sync::Mutex;

use crate::app::{
    AppError, AppV1, AppV2, Decision, WriteMode, balances_match_postings, count_nulls,
    digest_up_to, event_versions, max_entry_id, sample_trip, seed_v1, sqlstate, trip_km,
};
use crate::config::{Config, ConfigError, DEFAULT_JSON, legacy_fee_rule};
use crate::flags::{self, Flag, FlagError, FlagName, Rollout, UnstableRollout, bucket};
use crate::lab::Lab;
use crate::migrate::{
    self, ADD_CHECKS, Batch, EVENTS_VERSION, FEE_PERCENT_FROM_VERSION, POSTINGS_CURRENCY, Probed,
    Step, VALIDATE_EVENTS, backfill_all, backfill_batch, checkpoint, contract_a, contract_b,
    expand, finish_a, held_modes, retry_on_lock_timeout, run_tx, uncontract_b,
};
use crate::policy::PolicyVersion;
use crate::replay::{ReplayMode, ReplayReport, read_events, replay};

/// Trần chờ một điều kiện (số lần thăm dò, mỗi lần 10 ms): chỉ để khỏi treo vô hạn khi có lỗi.
const POLL_LIMIT: u32 = 12_000;
/// Trần chờ một tác vụ chạy xong.
const JOIN_LIMIT: Duration = Duration::from_secs(120);

async fn exec(conn: &mut PgConnection, sql: &'static str) -> Result<()> {
    sqlx::query(sql).execute(&mut *conn).await?;
    Ok(())
}

async fn backend_pid(conn: &mut PgConnection) -> Result<i32> {
    Ok(sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *conn)
        .await?)
}

/// Chờ tới khi có phiên bị chặn (đang chờ khóa) bởi `blocker`; trả pid các phiên đó.
/// `pg_blocking_pids` tính cả phiên xếp trước trong hàng đợi khóa, không chỉ phiên đang giữ khóa.
async fn wait_blocked_by(observer: &mut PgConnection, blocker: i32) -> Result<Vec<i32>> {
    for _ in 0..POLL_LIMIT {
        let rows = sqlx::query(
            "SELECT pid FROM pg_stat_activity \
             WHERE wait_event_type = 'Lock' AND $1 = ANY(pg_blocking_pids(pid)) ORDER BY pid",
        )
        .bind(blocker)
        .fetch_all(&mut *observer)
        .await?;
        if !rows.is_empty() {
            return Ok(rows.iter().map(|r| r.get("pid")).collect());
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    bail!("không thấy phiên nào bị chặn bởi backend {blocker}")
}

async fn join<T>(handle: tokio::task::JoinHandle<T>) -> Result<T> {
    tokio::time::timeout(JOIN_LIMIT, handle)
        .await
        .context("tác vụ không chạy xong (bị chặn?)")?
        .context("tác vụ panic")
}

fn ok_or_state(result: &Result<impl Sized, AppError>) -> Option<String> {
    match result {
        Ok(_) => None,
        Err(e) => Some(e.sqlstate().unwrap_or_else(|| e.to_string())),
    }
}

// ---------------------------------------------------------------------------------------------
// 1. Đi hết expand/contract với hai phiên bản code cùng chạy
// ---------------------------------------------------------------------------------------------

/// Hai cột mới của một chuyến ghi giữa chừng, theo từng phiên bản code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidRow {
    /// Mã chuyến.
    pub trip_id: String,
    /// Code đã ghi: "v1" hoặc "v2".
    pub writer: &'static str,
    /// Các giá trị `currency` của posting chuyến này.
    pub currencies: Vec<String>,
    /// Cột cũ.
    pub fee_percent: Option<i32>,
    /// Cột mới.
    pub policy_version: Option<i32>,
}

/// Báo cáo của [`walkthrough`].
#[derive(Debug)]
pub struct Walkthrough {
    /// Số chuyến seed bằng code cũ.
    pub seeded_trips: u32,
    /// Số posting lúc seed.
    pub seeded_postings: i64,
    /// Số posting và sự kiện còn NULL ngay sau expand.
    pub nulls_after_expand: (i64, i64),
    /// SQLSTATE khi cố sửa số tiền của posting sau khi guard được nới (phải là `TN001`).
    pub amount_update_state: Option<String>,
    /// Hai chuyến ghi giữa chừng.
    pub mid: Vec<MidRow>,
    /// Các lô backfill `postings.currency` (cả hai đoạn trước và sau khi "chết").
    pub currency_batches: Vec<Batch>,
    /// Checkpoint đọc lại lúc tiến trình khởi động lại.
    pub resumed_from: i64,
    /// Số lô đã chạy trước khi "chết".
    pub batches_before_crash: usize,
    /// Các lô backfill `fee_events.policy_version`.
    pub events_batches: Vec<Batch>,
    /// Còn NULL trước contract.
    pub nulls_before_contract: (i64, i64),
    /// SQLSTATE khi sửa currency sau contract A (guard đã chặt lại).
    pub guard_after_state: Option<String>,
    /// Kết quả code cũ ghi sau contract B.
    pub v1_after_state: Option<String>,
    /// Code mới (chỉ ghi cột mới) ghi sau contract B có được không.
    pub v2_after_ok: bool,
    /// Dấu vân tay bốn cột gốc trước và sau.
    pub digest: (String, String),
    /// `account_balances` khớp tổng posting.
    pub balances_ok: bool,
    /// Phân bố `policy_version` cuối cùng.
    pub versions: Vec<(i32, i64)>,
}

async fn mid_row(pool: &PgPool, trip_id: &str, writer: &'static str) -> Result<MidRow> {
    let currencies: Vec<String> = sqlx::query(
        "SELECT DISTINCT COALESCE(p.currency, 'NULL') AS c FROM postings p \
         JOIN journal_entries e ON e.id = p.entry_id WHERE e.idempotency_key LIKE $1 ORDER BY 1",
    )
    .bind(format!("trip:{trip_id}:%"))
    .fetch_all(pool)
    .await?
    .iter()
    .map(|r| r.get("c"))
    .collect();
    let event =
        sqlx::query("SELECT fee_percent, policy_version FROM fee_events WHERE trip_id = $1")
            .bind(trip_id)
            .fetch_one(pool)
            .await?;
    Ok(MidRow {
        trip_id: trip_id.to_string(),
        writer,
        currencies,
        fee_percent: event.get("fee_percent"),
        policy_version: event.get("policy_version"),
    })
}

/// Đi từ schema 1 tới schema cuối, có ghi từ cả code cũ và code mới ở giữa.
///
/// # Errors
/// Lỗi hạ tầng. Sai kỳ vọng được kiểm ở [`Walkthrough::verify`].
pub async fn walkthrough(lab: &Lab, cfg: &Config) -> Result<Walkthrough> {
    let pool = &lab.pool;
    let m = cfg.migration;
    let seeded_trips = 120;
    seed_v1(pool, seeded_trips).await?;
    let seeded_postings: i64 = sqlx::query_scalar("SELECT count(*) FROM postings")
        .fetch_one(pool)
        .await?;
    let seeded_max = max_entry_id(pool).await?;
    let digest_before = digest_up_to(pool, seeded_max).await?;

    // Expand: chỉ thêm. Từ đây hai phiên bản code cùng chạy được.
    expand(pool, m.lock_timeout_ms).await?;
    let nulls_after_expand = (
        count_nulls(pool, "postings", "currency").await?,
        count_nulls(pool, "fee_events", "policy_version").await?,
    );
    let amount_update_state = sqlx::query("UPDATE postings SET amount_dong = 1 WHERE entry_id = 1")
        .execute(pool)
        .await
        .err()
        .and_then(|e| sqlstate(&e));

    // Giữa chừng: code cũ và code mới cùng ghi.
    AppV1::record_trip(pool, &sample_trip(121)?).await?;
    AppV2::record_trip(
        pool,
        &sample_trip(122)?,
        PolicyVersion::V2,
        WriteMode::Dual,
        Decision::Frozen,
    )
    .await?;
    let mid = vec![
        mid_row(pool, "TN-0121", "v1").await?,
        mid_row(pool, "TN-0122", "v2").await?,
    ];

    // Backfill theo lô, vừa chạy vừa có bút toán mới. Chạy hai lô rồi "chết".
    let mut currency_batches = Vec::new();
    for (i, n) in [(0, 123), (1, 124)] {
        let batch = backfill_batch(pool, &POSTINGS_CURRENCY, m.batch_size)
            .await?
            .context("hết khóa quá sớm")?;
        currency_batches.push(batch);
        if i == 0 {
            AppV1::record_trip(pool, &sample_trip(n)?).await?;
        } else {
            AppV2::record_trip(
                pool,
                &sample_trip(n)?,
                PolicyVersion::V1,
                WriteMode::Dual,
                Decision::Frozen,
            )
            .await?;
        }
    }
    let batches_before_crash = currency_batches.len();
    // Tiến trình mới: không nhớ gì, chỉ đọc checkpoint rồi chạy tiếp.
    let resumed_from = checkpoint(pool, &POSTINGS_CURRENCY).await?;
    currency_batches.extend(backfill_all(pool, &POSTINGS_CURRENCY, m.batch_size).await?);
    let events_batches = backfill_all(pool, &EVENTS_VERSION, m.batch_size).await?;
    let nulls_before_contract = (
        count_nulls(pool, "postings", "currency").await?,
        count_nulls(pool, "fee_events", "policy_version").await?,
    );

    // Contract A: NOT NULL an toàn. Còn quay lại được.
    contract_a(pool, m.lock_timeout_ms).await?;
    let guard_after_state = sqlx::query("UPDATE postings SET currency = 'USD' WHERE entry_id = 1")
        .execute(pool)
        .await
        .err()
        .and_then(|e| sqlstate(&e));

    // Gỡ code cũ, code mới chuyển sang chỉ ghi cột mới, rồi contract B.
    AppV2::record_trip(
        pool,
        &sample_trip(125)?,
        PolicyVersion::V1,
        WriteMode::NewOnly,
        Decision::Frozen,
    )
    .await?;
    contract_b(pool, m.lock_timeout_ms).await?;
    let v1_after_state = ok_or_state(&AppV1::record_trip(pool, &sample_trip(126)?).await);
    let v2_after_ok = AppV2::record_trip(
        pool,
        &sample_trip(127)?,
        PolicyVersion::V2,
        WriteMode::NewOnly,
        Decision::Frozen,
    )
    .await
    .is_ok();

    let digest_after = digest_up_to(pool, seeded_max).await?;
    Ok(Walkthrough {
        seeded_trips,
        seeded_postings,
        nulls_after_expand,
        amount_update_state,
        mid,
        currency_batches,
        resumed_from,
        batches_before_crash,
        events_batches,
        nulls_before_contract,
        guard_after_state,
        v1_after_state,
        v2_after_ok,
        digest: (digest_before, digest_after),
        balances_ok: balances_match_postings(pool).await?,
        versions: event_versions(pool).await?,
    })
}

impl Walkthrough {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không khớp.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.seeded_postings == 600, "{self:?}");
        ensure!(self.nulls_after_expand == (600, 120), "{self:?}");
        ensure!(
            self.amount_update_state.as_deref() == Some("TN001"),
            "{self:?}"
        );
        // Chuyến của code cũ: trigger điền VND và policy_version 1. Chuyến của code mới: ghi đúp.
        ensure!(
            self.mid[0].currencies == ["VND"]
                && self.mid[0].fee_percent == Some(20)
                && self.mid[0].policy_version == Some(1),
            "{:?}",
            self.mid[0]
        );
        ensure!(
            self.mid[1].currencies == ["VND"]
                && self.mid[1].fee_percent == Some(15)
                && self.mid[1].policy_version == Some(2),
            "{:?}",
            self.mid[1]
        );
        ensure!(self.batches_before_crash == 2, "{self:?}");
        ensure!(
            self.resumed_from == self.currency_batches[1].checkpoint,
            "{self:?}"
        );
        // Mỗi dòng cũ được điền đúng một lần: tổng số dòng cập nhật bằng số dòng seed.
        let filled: u64 = self.currency_batches.iter().map(|b| b.rows).sum();
        ensure!(filled == 600, "điền {filled} dòng, cần 600: {self:?}");
        let events: u64 = self.events_batches.iter().map(|b| b.rows).sum();
        ensure!(events == 120, "{self:?}");
        // Checkpoint chỉ tăng.
        ensure!(
            self.currency_batches
                .windows(2)
                .all(|w| w[0].checkpoint < w[1].checkpoint),
            "{self:?}"
        );
        ensure!(self.nulls_before_contract == (0, 0), "{self:?}");
        ensure!(
            self.guard_after_state.as_deref() == Some("TN001"),
            "{self:?}"
        );
        ensure!(self.v1_after_state.as_deref() == Some("23502"), "{self:?}");
        ensure!(self.v2_after_ok, "{self:?}");
        ensure!(
            self.digest.0 == self.digest.1,
            "bút toán cũ đã đổi: {self:?}"
        );
        ensure!(self.balances_ok, "{self:?}");
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------
// 2. Ma trận tương thích: ba bản code x bốn giai đoạn
// ---------------------------------------------------------------------------------------------

/// Ma trận: mỗi hàng là một giai đoạn, ba ô là kết quả của `AppV1`, `AppV2 Dual`, `AppV2 NewOnly`
/// (`None` là ghi được, `Some` là SQLSTATE của lỗi).
#[derive(Debug)]
pub struct Matrix {
    /// Tên giai đoạn và ba ô.
    pub rows: Vec<(&'static str, [Option<String>; 3])>,
}

async fn try_three(pool: &PgPool, base: u32) -> Result<[Option<String>; 3]> {
    let v1 = AppV1::record_trip(pool, &sample_trip(base)?).await;
    let dual = AppV2::record_trip(
        pool,
        &sample_trip(base + 1)?,
        PolicyVersion::V2,
        WriteMode::Dual,
        Decision::Frozen,
    )
    .await;
    let new_only = AppV2::record_trip(
        pool,
        &sample_trip(base + 2)?,
        PolicyVersion::V2,
        WriteMode::NewOnly,
        Decision::Frozen,
    )
    .await;
    Ok([ok_or_state(&v1), ok_or_state(&dual), ok_or_state(&new_only)])
}

/// Chạy cả ba bản code ở từng giai đoạn của cuộc chuyển.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn compat_matrix(lab: &Lab, cfg: &Config) -> Result<Matrix> {
    let pool = &lab.pool;
    let m = cfg.migration;
    let mut rows = Vec::new();
    rows.push(("trước expand", try_three(pool, 1_000).await?));
    expand(pool, m.lock_timeout_ms).await?;
    rows.push(("sau expand", try_three(pool, 1_010).await?));
    backfill_all(pool, &POSTINGS_CURRENCY, m.batch_size).await?;
    backfill_all(pool, &EVENTS_VERSION, m.batch_size).await?;
    contract_a(pool, m.lock_timeout_ms).await?;
    rows.push(("sau contract A", try_three(pool, 1_020).await?));
    contract_b(pool, m.lock_timeout_ms).await?;
    rows.push(("sau contract B", try_three(pool, 1_030).await?));
    Ok(Matrix { rows })
}

impl Matrix {
    /// Kiểm ma trận bài viết nêu.
    ///
    /// # Errors
    /// Có ô khác kỳ vọng.
    pub fn verify(&self) -> Result<()> {
        let s = |x: &str| Some(x.to_string());
        let expected: [(&str, [Option<String>; 3]); 4] = [
            ("trước expand", [None, s("42703"), s("42703")]),
            ("sau expand", [None, None, s("23502")]),
            ("sau contract A", [None, None, None]),
            ("sau contract B", [s("23502"), s("42703"), None]),
        ];
        ensure!(self.rows.len() == expected.len(), "{self:?}");
        for ((name, got), (ename, want)) in self.rows.iter().zip(expected) {
            ensure!(
                *name == ename && *got == want,
                "{name}: {got:?}, cần {want:?}"
            );
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------
// 3. Khóa thật: đường an toàn
// ---------------------------------------------------------------------------------------------

/// Báo cáo đường an toàn (`CHECK ... NOT VALID`, `VALIDATE`, `SET NOT NULL`).
#[derive(Debug)]
pub struct SafePath {
    /// Quan sát từng bước DDL trên `postings` (khóa mới lấy, số lần quét).
    pub steps: Vec<Probed>,
    /// Khóa của phiên đang `VALIDATE` (giữ mở).
    pub validate_modes: Vec<String>,
    /// Số lần quét của `VALIDATE`.
    pub validate_scans: i64,
    /// Một bút toán mới ghi được trong lúc `VALIDATE` còn mở.
    pub writer_ok_during_validate: bool,
    /// `ADD COLUMN` không viết lại bảng: file dữ liệu (`pg_relation_filenode`) giữ nguyên.
    pub filenode_unchanged: bool,
    /// Đưa `VALIDATE` khi còn dòng NULL: SQLSTATE lỗi (phải là `23514`).
    pub validate_with_nulls_state: Option<String>,
}

/// Đường an toàn, quan sát khóa thật.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn safe_path(lab: &Lab, cfg: &Config) -> Result<SafePath> {
    let pool = &lab.pool;
    let m = cfg.migration;
    seed_v1(pool, 120).await?;
    let filenode_before = filenode(pool, "postings").await?;
    let mut steps = expand(pool, m.lock_timeout_ms).await?;
    let filenode_unchanged = filenode(pool, "postings").await? == filenode_before;

    // Còn dòng NULL thì VALIDATE từ chối: đó là hàng rào chặn contract sớm.
    steps.extend(run_tx(pool, m.lock_timeout_ms, &ADD_CHECKS).await?);
    let validate_with_nulls = run_tx(pool, m.lock_timeout_ms, &[migrate::VALIDATE_POSTINGS]).await;
    let validate_with_nulls_state = validate_with_nulls.err().and_then(|e| sqlstate(&e));

    backfill_all(pool, &POSTINGS_CURRENCY, m.batch_size).await?;
    backfill_all(pool, &EVENTS_VERSION, m.batch_size).await?;

    // VALIDATE giữ mở: một phiên khác vẫn ghi được.
    let mut holder = lab.connect("holder-validate").await?;
    exec(&mut holder, "BEGIN").await?;
    let scans_before = migrate::xact_scans(&mut holder, "postings").await?;
    exec(&mut holder, migrate::VALIDATE_POSTINGS.sql).await?;
    let validate_modes = held_modes(&mut holder, "postings").await?;
    let validate_scans = migrate::xact_scans(&mut holder, "postings").await? - scans_before;
    let writer = {
        let pool = pool.clone();
        tokio::spawn(async move {
            AppV2::record_trip(
                &pool,
                &sample_trip(500)?,
                PolicyVersion::V1,
                WriteMode::Dual,
                Decision::Frozen,
            )
            .await
        })
    };
    // Nếu VALIDATE chặn ghi, dòng này treo tới hết trần và báo lỗi thay vì chạy tiếp.
    let writer_ok_during_validate = join(writer).await?.is_ok();
    exec(&mut holder, "COMMIT").await?;
    steps.push(Probed {
        label: migrate::VALIDATE_POSTINGS.label,
        rel: Some("postings"),
        modes: validate_modes.clone(),
        scans: validate_scans,
    });

    steps.extend(run_tx(pool, m.lock_timeout_ms, &[VALIDATE_EVENTS]).await?);
    steps.extend(finish_a(pool, m.lock_timeout_ms).await?);
    steps.retain(|p| p.rel == Some("postings"));
    Ok(SafePath {
        steps,
        validate_modes,
        validate_scans,
        writer_ok_during_validate,
        filenode_unchanged,
        validate_with_nulls_state,
    })
}

async fn filenode(pool: &PgPool, table: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT pg_relation_filenode(to_regclass($1))::bigint")
        .bind(table)
        .fetch_one(pool)
        .await
}

impl SafePath {
    fn step(&self, prefix: &str) -> Result<&Probed> {
        self.steps
            .iter()
            .find(|p| p.label.starts_with(prefix))
            .with_context(|| format!("thiếu bước `{prefix}`"))
    }

    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không khớp.
    pub fn verify(&self) -> Result<()> {
        let access_exclusive = ["AccessExclusiveLock".to_string()];
        ensure!(
            self.step("ADD COLUMN")?.modes == access_exclusive,
            "{:?}",
            self.steps
        );
        ensure!(self.step("ADD COLUMN")?.scans == 0, "{:?}", self.steps);
        ensure!(self.filenode_unchanged, "{self:?}");
        // CREATE TRIGGER lấy khóa yếu hơn, nhưng ADD COLUMN cùng transaction đã giữ khóa mạnh nhất.
        ensure!(
            self.step("ADD CONSTRAINT")?.modes == access_exclusive,
            "{:?}",
            self.steps
        );
        ensure!(self.step("ADD CONSTRAINT")?.scans == 0, "{:?}", self.steps);
        ensure!(
            self.validate_modes
                .iter()
                .any(|m| m == "ShareUpdateExclusiveLock")
                && !self
                    .validate_modes
                    .iter()
                    .any(|m| m == "AccessExclusiveLock"),
            "{:?}",
            self.validate_modes
        );
        ensure!(self.validate_scans == 1, "{self:?}");
        ensure!(self.writer_ok_during_validate, "{self:?}");
        ensure!(
            self.validate_with_nulls_state.as_deref() == Some("23514"),
            "{self:?}"
        );
        // Đã có CHECK hợp lệ: SET NOT NULL vẫn xin khóa mạnh nhưng không quét bảng.
        let set = self.step("SET NOT NULL")?;
        ensure!(set.modes == access_exclusive && set.scans == 0, "{set:?}");
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------
// 3b. Hai đường ngắn hơn: DEFAULT hằng và NOT NULL ... NOT VALID
// ---------------------------------------------------------------------------------------------

const ADD_CONST_DEFAULT: Step = Step {
    label: "ADD COLUMN source NOT NULL DEFAULT hằng",
    sql: "ALTER TABLE postings ADD COLUMN source text NOT NULL DEFAULT 'ledger'",
    rel: Some("postings"),
};
const ADD_VOLATILE_DEFAULT: Step = Step {
    label: "ADD COLUMN noted_at DEFAULT clock_timestamp()",
    sql: "ALTER TABLE postings ADD COLUMN noted_at timestamptz DEFAULT clock_timestamp()",
    rel: Some("postings"),
};
const ADD_NULLABLE_NOTE: Step = Step {
    label: "ADD COLUMN fee_events.note (nullable)",
    sql: "ALTER TABLE fee_events ADD COLUMN note text",
    rel: Some("fee_events"),
};
const ADD_NOT_NULL_NOT_VALID: Step = Step {
    label: "ADD CONSTRAINT note_nn NOT NULL note NOT VALID",
    sql: "ALTER TABLE fee_events ADD CONSTRAINT note_nn NOT NULL note NOT VALID",
    rel: Some("fee_events"),
};
const VALIDATE_NOT_NULL: Step = Step {
    label: "VALIDATE CONSTRAINT note_nn",
    sql: "ALTER TABLE fee_events VALIDATE CONSTRAINT note_nn",
    rel: Some("fee_events"),
};

/// Báo cáo hai đường ngắn hơn expand/backfill khi điều kiện cho phép.
#[derive(Debug)]
pub struct Alternatives {
    /// `ADD COLUMN ... NOT NULL DEFAULT` với hằng.
    pub const_default: Probed,
    /// Hằng: file dữ liệu giữ nguyên, không viết lại bảng.
    pub const_default_same_file: bool,
    /// Số dòng cũ đọc ra giá trị mặc định mà không có `UPDATE` nào.
    pub rows_seeing_default: i64,
    /// Số dòng của bảng.
    pub rows_total: i64,
    /// `DEFAULT` volatile: file dữ liệu đổi, tức bảng bị viết lại.
    pub volatile_default_rewrites: bool,
    /// `ADD CONSTRAINT ... NOT NULL ... NOT VALID`.
    pub not_null_not_valid: Probed,
    /// SQLSTATE khi ghi dòng `NULL` mới lúc ràng buộc còn chưa `VALIDATE`.
    pub null_insert_state: Option<String>,
    /// `VALIDATE` ràng buộc not-null.
    pub validate: Probed,
}

fn single(mut probes: Vec<Probed>) -> Result<Probed> {
    probes.pop().context("run_tx không trả quan sát nào")
}

/// `DEFAULT` hằng (không viết lại bảng, từ PostgreSQL 11), `DEFAULT` volatile (viết lại bảng),
/// và ràng buộc not-null `NOT VALID` của PostgreSQL 18, đo bằng khóa và bộ đếm quét thật.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn alternatives(lab: &Lab, cfg: &Config) -> Result<Alternatives> {
    let pool = &lab.pool;
    let timeout = cfg.migration.lock_timeout_ms;
    seed_v1(pool, 30).await?;

    let file_before = filenode(pool, "postings").await?;
    let const_default = single(run_tx(pool, timeout, &[ADD_CONST_DEFAULT]).await?)?;
    let file_after_const = filenode(pool, "postings").await?;
    let rows_seeing_default: i64 =
        sqlx::query_scalar("SELECT count(*) FROM postings WHERE source = 'ledger'")
            .fetch_one(pool)
            .await?;
    let rows_total: i64 = sqlx::query_scalar("SELECT count(*) FROM postings")
        .fetch_one(pool)
        .await?;
    run_tx(pool, timeout, &[ADD_VOLATILE_DEFAULT]).await?;
    let file_after_volatile = filenode(pool, "postings").await?;

    run_tx(pool, timeout, &[ADD_NULLABLE_NOTE]).await?;
    let not_null_not_valid = single(run_tx(pool, timeout, &[ADD_NOT_NULL_NOT_VALID]).await?)?;
    // Dòng cũ còn NULL vẫn được giữ, nhưng dòng mới đã bị kiểm.
    let null_insert = sqlx::query(
        "INSERT INTO fee_events (trip_id, fare_dong, fee_percent) VALUES ('TN-9999', 25500, 20)",
    )
    .execute(pool)
    .await;
    let null_insert_state = null_insert.err().and_then(|e| sqlstate(&e));
    sqlx::query("UPDATE fee_events SET note = 'da-dien'")
        .execute(pool)
        .await?;
    let validate = single(run_tx(pool, timeout, &[VALIDATE_NOT_NULL]).await?)?;

    Ok(Alternatives {
        const_default,
        const_default_same_file: file_after_const == file_before,
        rows_seeing_default,
        rows_total,
        volatile_default_rewrites: file_after_volatile != file_after_const,
        not_null_not_valid,
        null_insert_state,
        validate,
    })
}

impl Alternatives {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không khớp.
    pub fn verify(&self) -> Result<()> {
        let access_exclusive = ["AccessExclusiveLock".to_string()];
        ensure!(
            self.const_default.modes == access_exclusive && self.const_default.scans == 0,
            "{self:?}"
        );
        ensure!(self.const_default_same_file, "{self:?}");
        ensure!(
            self.rows_total > 0 && self.rows_seeing_default == self.rows_total,
            "{self:?}"
        );
        ensure!(self.volatile_default_rewrites, "{self:?}");
        ensure!(
            self.not_null_not_valid.modes == access_exclusive && self.not_null_not_valid.scans == 0,
            "{self:?}"
        );
        ensure!(
            self.null_insert_state.as_deref() == Some("23502"),
            "{self:?}"
        );
        ensure!(
            self.validate
                .modes
                .iter()
                .any(|m| m == "ShareUpdateExclusiveLock")
                && !self
                    .validate
                    .modes
                    .iter()
                    .any(|m| m == "AccessExclusiveLock")
                && self.validate.scans == 1,
            "{self:?}"
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------
// 4. Khóa thật: đường naive
// ---------------------------------------------------------------------------------------------

/// Báo cáo `SET NOT NULL` không có CHECK hợp lệ.
#[derive(Debug)]
pub struct NaivePath {
    /// Khóa của phiên chạy `SET NOT NULL` (đang giữ mở).
    pub modes: Vec<String>,
    /// Số lần quét bảng trong lệnh đó.
    pub scans: i64,
    /// Một bút toán mới bị chặn trong lúc `SET NOT NULL` còn mở.
    pub writer_blocked: bool,
    /// Sau khi phiên kia commit, bút toán ghi được.
    pub writer_done_after_commit: bool,
}

/// `SET NOT NULL` thẳng, giữ transaction mở và quan sát người ghi bị chặn.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn naive_path(lab: &Lab, cfg: &Config) -> Result<NaivePath> {
    let pool = &lab.pool;
    let m = cfg.migration;
    seed_v1(pool, 120).await?;
    expand(pool, m.lock_timeout_ms).await?;
    backfill_all(pool, &POSTINGS_CURRENCY, m.batch_size).await?;

    let mut ddl = lab.connect("naive-ddl").await?;
    let mut observer = lab.connect("naive-observer").await?;
    let ddl_pid = backend_pid(&mut ddl).await?;
    exec(&mut ddl, "BEGIN").await?;
    let scans_before = migrate::xact_scans(&mut ddl, "postings").await?;
    exec(&mut ddl, migrate::SET_NOT_NULL_POSTINGS.sql).await?;
    let modes = held_modes(&mut ddl, "postings").await?;
    let scans = migrate::xact_scans(&mut ddl, "postings").await? - scans_before;

    let writer = {
        let pool = pool.clone();
        tokio::spawn(async move {
            AppV2::record_trip(
                &pool,
                &sample_trip(500)?,
                PolicyVersion::V1,
                WriteMode::Dual,
                Decision::Frozen,
            )
            .await
        })
    };
    let blocked = wait_blocked_by(&mut observer, ddl_pid).await?;
    let writer_blocked = !blocked.is_empty();
    exec(&mut ddl, "COMMIT").await?;
    let writer_done_after_commit = join(writer).await?.is_ok();
    Ok(NaivePath {
        modes,
        scans,
        writer_blocked,
        writer_done_after_commit,
    })
}

impl NaivePath {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không khớp.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.modes == ["AccessExclusiveLock"], "{self:?}");
        ensure!(self.scans == 1, "{self:?}");
        ensure!(
            self.writer_blocked && self.writer_done_after_commit,
            "{self:?}"
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------
// 5. Hàng đợi khóa: một ALTER đang chờ chặn cả truy vấn đọc đến sau
// ---------------------------------------------------------------------------------------------

/// Báo cáo hàng đợi khóa.
#[derive(Debug)]
pub struct LockQueue {
    /// Phiên `ALTER` bị chặn bởi phiên đọc dài (A).
    pub ddl_blocked_by_reader: bool,
    /// Truy vấn đọc ngắn (C) bị chặn bởi chính `ALTER`.
    pub reader_blocked_by_ddl: bool,
    /// Các yêu cầu khóa chưa được cấp: (vai, chế độ), theo thứ tự `ALTER`, đọc.
    pub waiting: Vec<(&'static str, String)>,
    /// Truy vấn C đọc được bao nhiêu dòng sau khi hàng đợi thông.
    pub reader_rows: i64,
    /// Cột đã được thêm khi hàng đợi thông.
    pub column_added: bool,
}

/// Dựng hàng đợi: A giữ khóa đọc, B (`ALTER`) chờ A, C (đọc) chờ B. Quan sát rồi nhả A.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn lock_queue(lab: &Lab) -> Result<LockQueue> {
    seed_v1(&lab.pool, 10).await?;
    let mut a = lab.connect("queue-a").await?;
    let mut b = lab.connect("queue-b").await?;
    let mut c = lab.connect("queue-c").await?;
    let mut observer = lab.connect("queue-observer").await?;
    let (pid_a, pid_b, pid_c) = (
        backend_pid(&mut a).await?,
        backend_pid(&mut b).await?,
        backend_pid(&mut c).await?,
    );

    // A: một báo cáo chạy dài, đang giữ AccessShareLock trên postings.
    exec(&mut a, "BEGIN").await?;
    exec(&mut a, "SELECT count(*) FROM postings").await?;

    let ddl =
        tokio::spawn(
            async move { exec(&mut b, "ALTER TABLE postings ADD COLUMN currency text").await },
        );
    let ddl_blocked_by_reader = wait_blocked_by(&mut observer, pid_a)
        .await?
        .contains(&pid_b);

    // C: truy vấn đọc ngắn tới sau B. Không xung đột với A, nhưng xếp sau B.
    let reader = tokio::spawn(async move {
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM postings")
            .fetch_one(&mut c)
            .await?;
        anyhow::Ok(n)
    });
    let reader_blocked_by_ddl = wait_blocked_by(&mut observer, pid_b)
        .await?
        .contains(&pid_c);

    let waiting_rows = sqlx::query(
        "SELECT pid, mode FROM pg_locks WHERE locktype = 'relation' AND NOT granted \
         AND relation = to_regclass('postings') ORDER BY pid",
    )
    .fetch_all(&mut observer)
    .await?;
    let mut waiting = Vec::new();
    for row in &waiting_rows {
        let pid: i32 = row.get("pid");
        let role = if pid == pid_b { "ALTER" } else { "đọc" };
        waiting.push((role, row.get::<String, _>("mode")));
    }
    waiting.sort_by_key(|(role, _)| u8::from(*role != "ALTER"));

    exec(&mut a, "COMMIT").await?;
    join(ddl).await??;
    let reader_rows = join(reader).await??;
    let column_added = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.columns \
         WHERE table_schema = current_schema() AND table_name = 'postings' AND column_name = 'currency')",
    )
    .fetch_one(&mut observer)
    .await?;
    Ok(LockQueue {
        ddl_blocked_by_reader,
        reader_blocked_by_ddl,
        waiting,
        reader_rows,
        column_added,
    })
}

impl LockQueue {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không khớp.
    pub fn verify(&self) -> Result<()> {
        ensure!(
            self.ddl_blocked_by_reader && self.reader_blocked_by_ddl,
            "{self:?}"
        );
        ensure!(
            self.waiting
                == [
                    ("ALTER", "AccessExclusiveLock".to_string()),
                    ("đọc", "AccessShareLock".to_string())
                ],
            "{self:?}"
        );
        ensure!(self.reader_rows == 50 && self.column_added, "{self:?}");
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------
// 6. lock_timeout và thử lại
// ---------------------------------------------------------------------------------------------

/// Báo cáo `lock_timeout` và thử lại.
#[derive(Debug)]
pub struct TimeoutRetry {
    /// Số lần thử đã dùng.
    pub attempts: u32,
    /// SQLSTATE của lần thử đầu.
    pub first_state: Option<String>,
    /// Sau lần thất bại, cột chưa được thêm (transaction hoàn tác sạch).
    pub column_absent_after_failure: bool,
    /// Sau lần thất bại, người ghi không còn bị chặn (không còn yêu cầu khóa nào xếp hàng).
    pub writer_ok_after_timeout: bool,
    /// Lần thử cuối thành công và cột có mặt.
    pub column_present_at_end: bool,
}

/// `expand` với `lock_timeout` ngắn trong lúc một phiên đọc dài giữ bảng; phiên đọc xong thì thử lại.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn timeout_retry(lab: &Lab, cfg: &Config) -> Result<TimeoutRetry> {
    let pool = lab.pool.clone();
    let m = cfg.migration;
    seed_v1(&pool, 10).await?;
    let mut reader = lab.connect("timeout-reader").await?;
    exec(&mut reader, "BEGIN").await?;
    exec(&mut reader, "SELECT count(*) FROM postings").await?;
    let reader = Arc::new(Mutex::new(reader));

    let first_state = Arc::new(Mutex::new(None::<String>));
    let column_absent = Arc::new(Mutex::new(false));
    let writer_ok = Arc::new(Mutex::new(false));
    let (_, attempts) = retry_on_lock_timeout(m.max_attempts, |n| {
        let (pool, reader) = (pool.clone(), reader.clone());
        let (first_state, column_absent, writer_ok) = (
            first_state.clone(),
            column_absent.clone(),
            writer_ok.clone(),
        );
        async move {
            if n == 2 {
                // Lần 1 đã hết hạn chờ. Kiểm trạng thái rồi cho phiên đọc dài kết thúc.
                *column_absent.lock().await = !has_column(&pool, "postings", "currency").await?;
                let trip = sample_trip(500).map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
                *writer_ok.lock().await = AppV1::record_trip(&pool, &trip).await.is_ok();
                sqlx::query("COMMIT")
                    .execute(&mut *reader.lock().await)
                    .await?;
            }
            let result = expand(&pool, m.lock_timeout_ms).await;
            if n == 1 {
                *first_state.lock().await = result.as_ref().err().and_then(sqlstate);
            }
            result
        }
    })
    .await?;
    let column_present_at_end = has_column(&pool, "postings", "currency").await?;
    let report = TimeoutRetry {
        attempts,
        first_state: first_state.lock().await.clone(),
        column_absent_after_failure: *column_absent.lock().await,
        writer_ok_after_timeout: *writer_ok.lock().await,
        column_present_at_end,
    };
    Ok(report)
}

async fn has_column(pool: &PgPool, table: &str, column: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.columns \
         WHERE table_schema = current_schema() AND table_name = $1 AND column_name = $2)",
    )
    .bind(table)
    .bind(column)
    .fetch_one(pool)
    .await
}

impl TimeoutRetry {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không khớp.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.attempts == 2, "{self:?}");
        ensure!(self.first_state.as_deref() == Some("55P03"), "{self:?}");
        ensure!(self.column_absent_after_failure, "{self:?}");
        ensure!(self.writer_ok_after_timeout, "{self:?}");
        ensure!(self.column_present_at_end, "{self:?}");
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------
// 7. Feature flag, policy_version và replay
// ---------------------------------------------------------------------------------------------

/// Báo cáo của [`fee_flag`].
#[derive(Debug)]
pub struct FeeFlag {
    /// Cờ đọc lúc khởi động.
    pub startup_flag: Flag,
    /// Phí và phiên bản của bốn chuyến: (mã chuyến, cước, phiên bản, phí, phần tài xế).
    pub trips: Vec<(String, i64, PolicyVersion, i64, i64)>,
    /// Dấu vân tay bút toán cũ trước và sau khi bật cờ.
    pub digest: (String, String),
    /// Các sự kiện, dạng JSON.
    pub events_json: Vec<String>,
    /// Replay theo `policy_version` trong sự kiện.
    pub pinned: ReplayReport,
    /// Replay áp quy tắc hiện hành cho mọi sự kiện.
    pub current: ReplayReport,
    /// Giao lại chuyến `TN-0004` sau khi tắt cờ, quyết định đóng băng: bút toán cũ được trả về.
    pub redelivery_frozen_replayed: bool,
    /// Phiên bản dùng cho lần giao lại đóng băng.
    pub redelivery_version: PolicyVersion,
    /// Cùng việc đó nhưng hỏi lại cờ: lỗi idempotency.
    pub redelivery_redecide_error: Option<String>,
    /// Hai worker xử lý cùng chuyến `TN-0005` cùng lúc, mỗi bên đề xuất một phiên bản khác nhau:
    /// (hai bên cùng phiên bản, số bên ghi mới, số bút toán chia cước của chuyến).
    pub concurrent: (bool, usize, i64),
    /// Số chuyến được chọn trong `TN-0001..TN-1000` theo từng tỷ lệ.
    pub selected: Vec<(u8, usize)>,
    /// Cùng tỷ lệ, hai lần đếm cho cùng số.
    pub selected_stable: bool,
    /// Tăng tỷ lệ chỉ thêm chuyến.
    pub monotonic: bool,
    /// Nhóm của bốn chuyến đầu.
    pub first_buckets: Vec<u8>,
    /// Bộ chọn ngẫu nhiên hỏi hai lần ở 50%: số chuyến đổi ý.
    pub unstable_flips: usize,
}

fn selected_count(flag: &Flag) -> usize {
    (1..=1_000)
        .filter(|n| flag.selects(&format!("TN-{n:04}")))
        .count()
}

/// Cờ phí mới, từ khởi động tới rollback bằng cờ, có replay.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn fee_flag(lab: &Lab, cfg: &Config) -> Result<FeeFlag> {
    let pool = &lab.pool;
    let m = cfg.migration;
    migrate::migrate_all(pool, m.batch_size, m.lock_timeout_ms).await?;
    let mut conn = lab.connect("flag-admin").await?;

    // Khởi động: kiểm cờ trước khi nhận chuyến nào.
    let startup_flag = flags::validate_all(&mut conn)
        .await?
        .into_iter()
        .find(|f| f.name == FlagName::FeeRuleV2)
        .context("thiếu cờ")?;

    let specs = [
        ("TN-0001", "customer:CU-001", "driver:DR-001", 5),
        ("TN-0002", "customer:CU-002", "driver:DR-002", 1),
        ("TN-0003", "customer:CU-003", "driver:DR-003", 1),
        ("TN-0004", "customer:CU-001", "driver:DR-001", 5),
    ];
    let mut trips = Vec::new();
    let mut handle = async |idx: usize, conn: &mut PgConnection| -> Result<()> {
        let (id, customer, driver, km) = specs[idx];
        let trip = trip_km(id, customer, driver, km)?;
        // Đọc cờ MỘT lần cho chuyến này; kết quả đi vào sự kiện.
        let flag = flags::load(conn, FlagName::FeeRuleV2).await?;
        let proposed = flag.fee_rule_for(id);
        let out =
            AppV2::record_trip(pool, &trip, proposed, WriteMode::NewOnly, Decision::Frozen).await?;
        let split = crate::policy::split_for(out.version, trip.fare)?;
        trips.push((
            id.to_string(),
            trip.fare.dong(),
            out.version,
            split.platform_fee.dong(),
            split.driver_net.dong(),
        ));
        Ok(())
    };

    // Cờ tắt: hai chuyến đầu theo V1.
    handle(0, &mut conn).await?;
    handle(1, &mut conn).await?;
    let old_max = max_entry_id(pool).await?;
    let digest_before = digest_up_to(pool, old_max).await?;

    // Bật cờ 100%: hai chuyến sau theo V2.
    flags::set(
        &mut conn,
        FlagName::FeeRuleV2,
        true,
        Rollout::new(100).context("tỷ lệ")?,
    )
    .await?;
    handle(2, &mut conn).await?;
    handle(3, &mut conn).await?;
    let digest_after = digest_up_to(pool, old_max).await?;

    let events_json = read_events(pool)
        .await?
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?;
    let pinned = replay(pool, ReplayMode::Pinned).await?;
    let current = replay(pool, ReplayMode::CurrentRule).await?;

    // Tắt cờ (rollback bằng cờ), rồi Kafka giao lại thông điệp của TN-0004.
    flags::set(
        &mut conn,
        FlagName::FeeRuleV2,
        false,
        Rollout::new(0).context("tỷ lệ")?,
    )
    .await?;
    let off = flags::load(&mut conn, FlagName::FeeRuleV2).await?;
    let (id, customer, driver, km) = specs[3];
    let trip = trip_km(id, customer, driver, km)?;
    let frozen = AppV2::record_trip(
        pool,
        &trip,
        off.fee_rule_for(id),
        WriteMode::NewOnly,
        Decision::Frozen,
    )
    .await?;
    let redecide = AppV2::record_trip(
        pool,
        &trip,
        off.fee_rule_for(id),
        WriteMode::NewOnly,
        Decision::Redecide,
    )
    .await;

    // Hai instance thấy cờ khác nhau (V1 và V2) và xử lý cùng một chuyến cùng lúc.
    let trip5 = trip_km("TN-0005", "customer:CU-002", "driver:DR-002", 3)?;
    let (left, right) = tokio::join!(
        AppV2::record_trip(
            pool,
            &trip5,
            PolicyVersion::V1,
            WriteMode::NewOnly,
            Decision::Frozen
        ),
        AppV2::record_trip(
            pool,
            &trip5,
            PolicyVersion::V2,
            WriteMode::NewOnly,
            Decision::Frozen
        ),
    );
    let (left, right) = (left?, right?);
    let split_entries: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM journal_entries WHERE idempotency_key = 'trip:TN-0005:split'",
    )
    .fetch_one(pool)
    .await?;
    let concurrent = (
        left.version == right.version,
        [left.split, right.split]
            .iter()
            .filter(|o| matches!(o, crate::app::PostOutcome::Posted(_)))
            .count(),
        split_entries,
    );

    // Chọn theo tỷ lệ: tất định theo tripId.
    let mut selected = Vec::new();
    let mut monotonic = true;
    let mut previous: Vec<bool> = vec![false; 1_000];
    let mut selected_stable = true;
    for percent in [0u8, 10, 30, 50, 100] {
        let flag = Flag {
            name: FlagName::FeeRuleV2,
            enabled: true,
            rollout: Rollout::new(percent).context("tỷ lệ")?,
        };
        let now: Vec<bool> = (1..=1_000)
            .map(|n| flag.selects(&format!("TN-{n:04}")))
            .collect();
        monotonic &= previous.iter().zip(&now).all(|(was, is)| !*was || *is);
        selected_stable &= selected_count(&flag) == now.iter().filter(|x| **x).count();
        selected.push((percent, now.iter().filter(|x| **x).count()));
        previous = now;
    }
    let unstable = UnstableRollout::new(7);
    let pass1: Vec<bool> = (1..=1_000)
        .map(|n| unstable.selects(&format!("TN-{n:04}"), 50))
        .collect();
    let pass2: Vec<bool> = (1..=1_000)
        .map(|n| unstable.selects(&format!("TN-{n:04}"), 50))
        .collect();

    Ok(FeeFlag {
        startup_flag,
        trips,
        digest: (digest_before, digest_after),
        events_json,
        pinned,
        current,
        redelivery_frozen_replayed: matches!(
            (frozen.capture, frozen.split),
            (
                crate::app::PostOutcome::Replayed(_),
                crate::app::PostOutcome::Replayed(_)
            )
        ),
        redelivery_version: frozen.version,
        redelivery_redecide_error: redecide.err().map(|e| e.to_string()),
        concurrent,
        selected,
        selected_stable,
        monotonic,
        first_buckets: (1..=4)
            .map(|n| bucket(FlagName::FeeRuleV2, &format!("TN-{n:04}")))
            .collect(),
        unstable_flips: pass1.iter().zip(&pass2).filter(|(a, b)| a != b).count(),
    })
}

impl FeeFlag {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không khớp.
    pub fn verify(&self) -> Result<()> {
        ensure!(
            !self.startup_flag.enabled && self.startup_flag.rollout.percent() == 0,
            "{self:?}"
        );
        let want = [
            ("TN-0001", 25_500, PolicyVersion::V1, 5_100, 20_400),
            ("TN-0002", 12_000, PolicyVersion::V1, 2_400, 9_600),
            ("TN-0003", 12_000, PolicyVersion::V2, 3_000, 9_000),
            ("TN-0004", 25_500, PolicyVersion::V2, 3_825, 21_675),
        ];
        ensure!(self.trips.len() == want.len(), "{self:?}");
        for (got, want) in self.trips.iter().zip(want) {
            ensure!(
                (got.0.as_str(), got.1, got.2, got.3, got.4) == want,
                "{got:?} khác {want:?}"
            );
        }
        ensure!(
            self.digest.0 == self.digest.1,
            "bút toán cũ đã đổi: {self:?}"
        );
        ensure!(self.events_json.len() == 4, "{self:?}");
        ensure!(
            self.events_json[0]
                == r#"{"schemaVersion":1,"tripId":"TN-0001","fareDong":25500,"policyVersion":1}"#,
            "{:?}",
            self.events_json[0]
        );
        ensure!(
            self.pinned.matches() && self.pinned.trips == 4,
            "{:?}",
            self.pinned
        );
        let ids: Vec<&str> = self
            .current
            .diffs
            .iter()
            .map(|d| d.trip_id.as_str())
            .collect();
        ensure!(ids == ["TN-0001", "TN-0002"], "{:?}", self.current);
        ensure!(
            self.current.diffs[0].expected_fee == 3_825
                && self.current.diffs[0].actual_fee == 5_100,
            "{:?}",
            self.current
        );
        ensure!(
            self.redelivery_frozen_replayed && self.redelivery_version == PolicyVersion::V2,
            "{self:?}"
        );
        ensure!(
            self.redelivery_redecide_error
                .as_deref()
                .is_some_and(|e| e.contains("trip:TN-0004:split")),
            "{self:?}"
        );
        ensure!(self.concurrent == (true, 1, 1), "{:?}", self.concurrent);
        ensure!(self.selected_stable && self.monotonic, "{self:?}");
        ensure!(self.selected.first() == Some(&(0, 0)), "{self:?}");
        ensure!(self.selected.last() == Some(&(100, 1_000)), "{self:?}");
        ensure!(
            self.first_buckets == flags::GOLDEN_FIRST_FOUR,
            "{:?}",
            self.first_buckets
        );
        ensure!(self.selected == GOLDEN_SELECTED, "{:?}", self.selected);
        ensure!(self.unstable_flips > 0, "{self:?}");
        Ok(())
    }
}

/// Số chuyến được chọn trong `TN-0001..TN-1000` ở các tỷ lệ 0, 10, 30, 50, 100 (giá trị vàng).
pub const GOLDEN_SELECTED: [(u8, usize); 5] =
    [(0, 0), (10, 115), (30, 309), (50, 491), (100, 1_000)];

// ---------------------------------------------------------------------------------------------
// 8. Rollback: trước và sau contract
// ---------------------------------------------------------------------------------------------

/// Báo cáo của [`rollback`].
#[derive(Debug)]
pub struct Rollback {
    /// Trước contract, code cũ đọc `fee_percent` của chuyến ghi bởi code mới:
    /// (mã chuyến, tỷ lệ code cũ đọc được, tỷ lệ thực = phí / cước, phần trăm nguyên).
    pub v1_reads_v2_rows: Vec<(String, i32, i64)>,
    /// Code cũ ghi lại được trước contract.
    pub v1_before_contract_ok: bool,
    /// Sau contract B: code cũ ghi lỗi.
    pub v1_write_after_state: Option<String>,
    /// Sau contract B: code cũ đọc báo cáo lỗi.
    pub v1_report_after_state: Option<String>,
    /// Sau `uncontract_b`: code cũ ghi lại được.
    pub v1_write_after_uncontract_ok: bool,
    /// Sau `uncontract_b`: cột cũ rỗng, số dòng NULL / tổng.
    pub fee_percent_nulls: (i64, i64),
    /// Báo cáo của code cũ trước khi suy lại cột: lỗi vì NULL.
    pub v1_report_before_refill_failed: bool,
    /// Sau khi suy lại cột cũ từ cột mới: các dòng ghi đúp trước đó khớp giá trị ban đầu.
    pub refill_matches_original: bool,
    /// Dấu vân tay bút toán trước/sau toàn bộ chuỗi (bút toán đã ghi không đổi).
    pub digest: (String, String),
}

/// Lùi code trước contract, sau contract B và cách dựng lại đường lùi.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn rollback(lab: &Lab, cfg: &Config) -> Result<Rollback> {
    let pool = &lab.pool;
    let m = cfg.migration;
    seed_v1(pool, 10).await?;
    expand(pool, m.lock_timeout_ms).await?;

    // Hai chuyến theo V2 ghi đúp: chuyến ngắn chạm sàn 3.000 (tỷ lệ thực 25%), chuyến dài thì
    // đúng 15%. Cột cũ `fee_percent` luôn ghi tỷ lệ danh nghĩa 15.
    let mut real_percent = Vec::new();
    for (id, km) in [("TN-0011", 1), ("TN-0012", 5)] {
        let trip = trip_km(id, "customer:CU-001", "driver:DR-001", km)?;
        AppV2::record_trip(
            pool,
            &trip,
            PolicyVersion::V2,
            WriteMode::Dual,
            Decision::Frozen,
        )
        .await?;
        let fee = crate::policy::split_for(PolicyVersion::V2, trip.fare)?.platform_fee;
        real_percent.push((id, fee.dong() * 100 / trip.fare.dong()));
    }
    let max_id = max_entry_id(pool).await?;
    let digest_before = digest_up_to(pool, max_id).await?;

    // Lùi code về bản cũ trước contract: chạy được, nhưng cột cũ chỉ là gần đúng cho chuyến V2.
    let v1_before_contract_ok = AppV1::record_trip(pool, &sample_trip(13)?).await.is_ok();
    let report = AppV1::fee_report(pool).await?;
    let v1_reads_v2_rows = report
        .iter()
        .filter_map(|(id, percent)| {
            real_percent
                .iter()
                .find(|(rid, _)| rid == id)
                .map(|(_, real)| (id.clone(), *percent, *real))
        })
        .collect();

    // Chốt snapshot cột cũ rồi contract.
    let original: Vec<(String, i32)> = AppV1::fee_report(pool).await?;
    backfill_all(pool, &POSTINGS_CURRENCY, m.batch_size).await?;
    backfill_all(pool, &EVENTS_VERSION, m.batch_size).await?;
    contract_a(pool, m.lock_timeout_ms).await?;
    AppV2::record_trip(
        pool,
        &sample_trip(14)?,
        PolicyVersion::V2,
        WriteMode::NewOnly,
        Decision::Frozen,
    )
    .await?;
    contract_b(pool, m.lock_timeout_ms).await?;

    let v1_write_after_state = ok_or_state(&AppV1::record_trip(pool, &sample_trip(15)?).await);
    let v1_report_after_state = ok_or_state(&AppV1::fee_report(pool).await);

    // Dựng lại đường lùi: cột cũ trở lại nhưng rỗng.
    uncontract_b(pool, m.lock_timeout_ms).await?;
    let v1_write_after_uncontract_ok = AppV1::record_trip(pool, &sample_trip(16)?).await.is_ok();
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM fee_events")
        .fetch_one(pool)
        .await?;
    let nulls = count_nulls(pool, "fee_events", "fee_percent").await?;
    let v1_report_before_refill_failed = AppV1::fee_report(pool).await.is_err();
    backfill_all(pool, &FEE_PERCENT_FROM_VERSION, m.batch_size).await?;
    let restored = AppV1::fee_report(pool).await?;
    let refill_matches_original = original
        .iter()
        .all(|(id, percent)| restored.iter().any(|(rid, rp)| rid == id && rp == percent));
    let digest_after = digest_up_to(pool, max_id).await?;
    Ok(Rollback {
        v1_reads_v2_rows,
        v1_before_contract_ok,
        v1_write_after_state,
        v1_report_after_state,
        v1_write_after_uncontract_ok,
        fee_percent_nulls: (nulls, total),
        v1_report_before_refill_failed,
        refill_matches_original,
        digest: (digest_before, digest_after),
    })
}

impl Rollback {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không khớp.
    pub fn verify(&self) -> Result<()> {
        ensure!(
            self.v1_reads_v2_rows
                == [
                    ("TN-0011".to_string(), 15, 25),
                    ("TN-0012".to_string(), 15, 15)
                ],
            "{self:?}"
        );
        ensure!(self.v1_before_contract_ok, "{self:?}");
        ensure!(
            self.v1_write_after_state.as_deref() == Some("23502"),
            "{self:?}"
        );
        ensure!(
            self.v1_report_after_state.as_deref() == Some("42703"),
            "{self:?}"
        );
        ensure!(self.v1_write_after_uncontract_ok, "{self:?}");
        // Chỉ chuyến vừa ghi bằng code cũ sau khi dựng lại có giá trị; mọi dòng trước đó rỗng.
        ensure!(
            self.fee_percent_nulls.0 + 1 == self.fee_percent_nulls.1,
            "{self:?}"
        );
        ensure!(self.v1_report_before_refill_failed, "{self:?}");
        ensure!(self.refill_matches_original, "{self:?}");
        ensure!(self.digest.0 == self.digest.1, "bút toán đã đổi: {self:?}");
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------
// 9. Cấu hình kiểu an toàn
// ---------------------------------------------------------------------------------------------

/// Báo cáo của [`typed_config`].
#[derive(Debug)]
pub struct TypedConfig {
    /// Cấu hình mẫu đọc được.
    pub default_ok: bool,
    /// Các cấu hình hỏng và lỗi nhận được lúc khởi động: (mô tả, lỗi).
    pub rejected: Vec<(&'static str, String)>,
    /// Cách chuỗi: giá trị nhận được cho `"v2"`, `"V2"`, `"v3"`.
    pub legacy: [PolicyVersion; 3],
    /// Lỗi khi DB có dòng cờ lạ.
    pub unknown_flag: Option<String>,
    /// Lỗi khi DB thiếu dòng cờ đã biết.
    pub missing_flag: Option<String>,
}

/// Cấu hình đúng và sai, cờ trong DB đúng và sai.
///
/// # Errors
/// Lỗi hạ tầng.
pub async fn typed_config(lab: &Lab) -> Result<TypedConfig> {
    let default_ok = Config::from_json(DEFAULT_JSON).is_ok();
    let cases: [(&'static str, String); 4] = [
        (
            "defaultFeeRule = \"v3\"",
            DEFAULT_JSON.replace("\"v1\"", "\"v3\""),
        ),
        (
            "batchSise (gõ sai tên trường)",
            DEFAULT_JSON.replace("batchSize", "batchSise"),
        ),
        ("batchSize = 0", DEFAULT_JSON.replace("100", "0")),
        (
            "lockTimeoutMs = 60000",
            DEFAULT_JSON.replace("\"lockTimeoutMs\": 200", "\"lockTimeoutMs\": 60000"),
        ),
    ];
    let rejected = cases
        .into_iter()
        .map(|(what, json)| {
            let err = Config::from_json(&json).err().map_or_else(
                || "(được chấp nhận)".to_string(),
                |e: ConfigError| e.to_string(),
            );
            (what, err)
        })
        .collect();

    let mut conn = lab.connect("config-check").await?;
    sqlx::query("INSERT INTO feature_flags (name, enabled, rollout_percent) VALUES ('fee-rule-v3', true, 50)")
        .execute(&mut conn)
        .await?;
    let unknown_flag = flags::validate_all(&mut conn)
        .await
        .err()
        .map(|e: FlagError| e.to_string());
    sqlx::query("DELETE FROM feature_flags")
        .execute(&mut conn)
        .await?;
    let missing_flag = flags::validate_all(&mut conn)
        .await
        .err()
        .map(|e: FlagError| e.to_string());

    Ok(TypedConfig {
        default_ok,
        rejected,
        legacy: [
            legacy_fee_rule("v2"),
            legacy_fee_rule("V2"),
            legacy_fee_rule("v3"),
        ],
        unknown_flag,
        missing_flag,
    })
}

impl TypedConfig {
    /// Kiểm các giá trị bài viết nêu.
    ///
    /// # Errors
    /// Giá trị không khớp.
    pub fn verify(&self) -> Result<()> {
        ensure!(self.default_ok, "{self:?}");
        ensure!(
            self.rejected.iter().all(|(_, e)| e != "(được chấp nhận)"),
            "{self:?}"
        );
        ensure!(
            self.legacy == [PolicyVersion::V2, PolicyVersion::V1, PolicyVersion::V1],
            "{self:?}"
        );
        ensure!(
            self.unknown_flag
                .as_deref()
                .is_some_and(|e| e.contains("fee-rule-v3")),
            "{self:?}"
        );
        ensure!(
            self.missing_flag
                .as_deref()
                .is_some_and(|e| e.contains("thiếu trong DB")),
            "{self:?}"
        );
        Ok(())
    }
}
