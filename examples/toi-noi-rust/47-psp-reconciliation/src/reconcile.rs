//! Một lần đối soát: băm file, đọc streaming vào bảng tạm bằng COPY, khớp bằng SQL, ghi kết quả.

use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use futures_util::TryStreamExt;
use jiff::Timestamp;
use sqlx::{AssertSqlSafe, PgConnection, PgPool, Row as _};

use crate::kind::Kind;
use crate::memory::{Exception, MatchResult, Matcher, Row};
use crate::period::Period;
use crate::statement::{HashingReader, StatementReader, StatementRow, sha256_file};

const OURS_SELECT: &str = include_str!("../sql/ours_rows.sql");
const JOINED_SELECT: &str = include_str!("../sql/joined.sql");
const ITEMS_INSERT: &str = include_str!("../sql/items.sql");

/// Gửi sang PostgreSQL mỗi khi đệm COPY đủ ngần này byte.
const COPY_CHUNK: usize = 64 * 1024;
/// Giữ chi tiết tối đa ngần này dòng hỏng; số dòng hỏng thật vẫn được đếm đủ.
pub const MAX_BAD_ITEMS: usize = 1_000;

/// Đầu vào một lần đối soát.
#[derive(Debug, Clone, Copy)]
pub struct RunRequest<'a> {
    /// Tên PSP, ví dụ `vixanh`.
    pub provider: &'a str,
    /// File sao kê.
    pub path: &'a Path,
    /// Kỳ cần đối soát.
    pub period: Period,
    /// Thời điểm chạy, lấy từ đồng hồ truyền vào (không gọi `now()` trong hàm này).
    pub run_at: Timestamp,
}

/// Kết quả đã lưu của một run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSummary {
    /// Mã run.
    pub run_id: i64,
    /// SHA-256 của file.
    pub file_sha256: String,
    /// Số dòng sao kê hợp lệ.
    pub statement_rows: i64,
    /// Số dòng hỏng.
    pub bad_lines: i64,
    /// Số dòng sổ cái trong cửa sổ đọc (kỳ ± 24 giờ).
    pub ledger_rows: i64,
    /// Số ref khớp.
    pub refs_matched: i64,
    /// Trong đó số ref có nhiều dòng ở một phía.
    pub refs_matched_split: i64,
    /// Số ref nằm ngoài kỳ ở cả hai phía (bị bỏ qua).
    pub refs_out_of_period: i64,
    /// Số ngoại lệ theo loại (không gồm dòng hỏng).
    pub exceptions: BTreeMap<Kind, i64>,
}

/// Thời gian các pha, chỉ để đo; không dùng để quyết định gì.
#[derive(Debug, Clone, Copy, Default)]
pub struct RunTimings {
    /// Băm SHA-256 cả file.
    pub hash: Duration,
    /// Đọc CSV, định dạng và gửi COPY.
    pub load: Duration,
    /// Dựng bảng sổ cái, khớp và ghi ngoại lệ.
    pub matching: Duration,
    /// Cả run, tính từ lúc băm.
    pub total: Duration,
}

/// Kết quả gọi đối soát.
#[derive(Debug, Clone)]
pub enum RunOutcome {
    /// Run mới vừa được tạo.
    Created {
        /// Kết quả.
        summary: RunSummary,
        /// Thời gian các pha.
        timings: RunTimings,
    },
    /// File này (theo hash) đã được đối soát cho kỳ này; trả kết quả cũ, không làm lại.
    AlreadyDone(RunSummary),
}

impl RunOutcome {
    /// Kết quả, dù mới hay cũ.
    pub fn summary(&self) -> &RunSummary {
        match self {
            RunOutcome::Created { summary, .. } | RunOutcome::AlreadyDone(summary) => summary,
        }
    }
}

/// Đối soát một file: băm trước (để chạy lại cùng file thì dừng sớm), rồi mới đọc.
///
/// # Errors
/// File không đọc được, tiêu đề sai, hoặc lỗi cơ sở dữ liệu. Lỗi nào cũng rollback toàn bộ.
pub async fn reconcile_file(pool: &PgPool, req: &RunRequest<'_>) -> Result<RunOutcome> {
    let started = Instant::now();
    let path = req.path.to_path_buf();
    let sha = tokio::task::spawn_blocking(move || sha256_file(&path))
        .await?
        .with_context(|| format!("không băm được {}", req.path.display()))?;
    let hash = started.elapsed();
    let mut outcome = reconcile_expecting(pool, req, &sha).await?;
    if let RunOutcome::Created { timings, .. } = &mut outcome {
        timings.hash = hash;
        timings.total += hash;
    }
    Ok(outcome)
}

/// Như [`reconcile_file`] nhưng nhận hash đã tính. Lượt đọc chính băm lại và so với `expected_sha`:
/// file đổi giữa hai lượt đọc thì lỗi và rollback, không lưu run nào.
///
/// # Errors
/// Như [`reconcile_file`], thêm lỗi khi hash lượt đọc khác `expected_sha`.
pub async fn reconcile_expecting(
    pool: &PgPool,
    req: &RunRequest<'_>,
    expected_sha: &str,
) -> Result<RunOutcome> {
    if let Some(run_id) = find_run(pool, expected_sha, &req.period).await? {
        return Ok(RunOutcome::AlreadyDone(load_summary(pool, run_id).await?));
    }

    let started = Instant::now();
    let mut tx = pool.begin().await?;

    // Chốt idempotency: unique theo (file_sha256, kỳ). Hai lần chạy đồng thời thì lần sau chờ lần trước commit
    // rồi DO NOTHING trả 0 dòng. Run, bảng tạm, ngoại lệ cùng một transaction: hoặc có đủ, hoặc không có gì.
    let file_name = req
        .path
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let run_id: Option<i64> = sqlx::query_scalar(
        "INSERT INTO reconciliation_runs \
             (provider, file_name, file_sha256, period_start, period_end, run_at) \
         VALUES ($1, $2, $3, $4::timestamptz, $5::timestamptz, $6::timestamptz) \
         ON CONFLICT (file_sha256, period_start, period_end) DO NOTHING RETURNING id",
    )
    .bind(req.provider)
    .bind(&file_name)
    .bind(expected_sha)
    .bind(req.period.start.to_string())
    .bind(req.period.end.to_string())
    .bind(req.run_at.to_string())
    .fetch_optional(&mut *tx)
    .await?;
    let Some(run_id) = run_id else {
        drop(tx);
        let existing = find_run(pool, expected_sha, &req.period)
            .await?
            .context("run vừa trùng khóa nhưng không tìm thấy")?;
        return Ok(RunOutcome::AlreadyDone(load_summary(pool, existing).await?));
    };

    create_params(&mut tx, run_id, &req.period).await?;
    sqlx::query(
        "CREATE TEMP TABLE psp_stmt (\
             line_no bigint NOT NULL, external_ref text NOT NULL, \
             amount_dong bigint NOT NULL, settled_at timestamptz NOT NULL) ON COMMIT DROP",
    )
    .execute(&mut *tx)
    .await?;

    // Pha 1: đọc CSV từng dòng, gửi COPY theo khối 64 KiB. Không có Vec nào giữ cả file.
    let load_started = Instant::now();
    let file =
        File::open(req.path).with_context(|| format!("không mở được {}", req.path.display()))?;
    let mut reader = StatementReader::new(HashingReader::new(file), MAX_BAD_ITEMS)?;
    let mut copy = tx
        .copy_in_raw("COPY psp_stmt (line_no, external_ref, amount_dong, settled_at) FROM STDIN (FORMAT csv)")
        .await?;
    let mut buf: Vec<u8> = Vec::with_capacity(COPY_CHUNK + 4096);
    while let Some(row) = reader.next_row()? {
        push_copy_line(&mut buf, &row);
        if buf.len() >= COPY_CHUNK {
            let chunk = std::mem::replace(&mut buf, Vec::with_capacity(COPY_CHUNK + 4096));
            copy.send(chunk).await?;
        }
    }
    if !buf.is_empty() {
        copy.send(buf).await?;
    }
    let copied = copy.finish().await?;
    let good = reader.good();
    if copied != good {
        bail!("COPY nhận {copied} dòng nhưng đã đọc {good} dòng hợp lệ");
    }
    let bad_total = reader.bad_total();
    let bad_kept = reader.bad_lines().to_vec();
    let actual_sha = reader.into_inner().finish_hex();
    if actual_sha != expected_sha {
        bail!("file đổi giữa hai lượt đọc (hash {expected_sha} rồi {actual_sha}), hủy đối soát");
    }
    let load = load_started.elapsed();

    // Pha 2: khớp bằng SQL. Bảng tạm không có thống kê, nên ANALYZE trước khi nối.
    let match_started = Instant::now();
    sqlx::query("ANALYZE psp_stmt").execute(&mut *tx).await?;
    sqlx::query(AssertSqlSafe(format!(
        "CREATE TEMP TABLE ours_rows ON COMMIT DROP AS {OURS_SELECT}"
    )))
    .execute(&mut *tx)
    .await?;
    sqlx::query("ANALYZE ours_rows").execute(&mut *tx).await?;
    sqlx::query(AssertSqlSafe(format!(
        "CREATE TEMP TABLE joined ON COMMIT DROP AS {JOINED_SELECT}"
    )))
    .execute(&mut *tx)
    .await?;

    let ledger_rows: i64 = sqlx::query_scalar("SELECT count(*) FROM ours_rows")
        .fetch_one(&mut *tx)
        .await?;
    let counts = sqlx::query(
        "SELECT kind, count(*)::bigint, \
                count(*) FILTER (WHERE shape IS DISTINCT FROM '1-1')::bigint \
         FROM joined GROUP BY kind",
    )
    .fetch_all(&mut *tx)
    .await?;
    let (mut matched, mut matched_split, mut out_of_period) = (0_i64, 0_i64, 0_i64);
    for row in &counts {
        let kind: String = row.get(0);
        let n: i64 = row.get(1);
        match kind.as_str() {
            "MATCHED" => {
                matched = n;
                matched_split = row.get(2);
            }
            "OUT_OF_PERIOD" => out_of_period = n,
            _ => {}
        }
    }

    // Ngoại lệ thành việc mở (status OPEN) cho Finance. Không có bút toán nào được ghi ở đây.
    sqlx::query(ITEMS_INSERT)
        .bind(run_id)
        .execute(&mut *tx)
        .await?;
    let lines: Vec<i64> = bad_kept
        .iter()
        .map(|b| i64::try_from(b.line_no).unwrap_or(i64::MAX))
        .collect();
    let reasons: Vec<String> = bad_kept.iter().map(|b| b.reason.clone()).collect();
    sqlx::query(
        "INSERT INTO reconciliation_items (run_id, kind, line_no, detail) \
         SELECT $1, 'BAD_LINE', l, d FROM unnest($2::bigint[], $3::text[]) AS t(l, d)",
    )
    .bind(run_id)
    .bind(&lines)
    .bind(&reasons)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE reconciliation_runs SET statement_rows = $2, bad_lines = $3, ledger_rows = $4, \
             refs_matched = $5, refs_matched_split = $6, refs_out_of_period = $7 WHERE id = $1",
    )
    .bind(run_id)
    .bind(i64::try_from(good)?)
    .bind(i64::try_from(bad_total)?)
    .bind(ledger_rows)
    .bind(matched)
    .bind(matched_split)
    .bind(out_of_period)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    let matching = match_started.elapsed();

    let summary = load_summary(pool, run_id).await?;
    Ok(RunOutcome::Created {
        summary,
        timings: RunTimings {
            hash: Duration::ZERO,
            load,
            matching,
            total: started.elapsed(),
        },
    })
}

/// Thêm một dòng định dạng CSV của PostgreSQL `COPY` vào `buf`. `external_ref` đến từ file ngoài
/// nên được bọc nháy kép khi chứa dấu phẩy, nháy kép hoặc xuống dòng.
fn push_copy_line(buf: &mut Vec<u8>, row: &StatementRow) {
    buf.extend_from_slice(row.line_no.to_string().as_bytes());
    buf.push(b',');
    if row.external_ref.contains([',', '"', '\n', '\r']) {
        buf.push(b'"');
        buf.extend_from_slice(row.external_ref.replace('"', "\"\"").as_bytes());
        buf.push(b'"');
    } else {
        buf.extend_from_slice(row.external_ref.as_bytes());
    }
    buf.push(b',');
    buf.extend_from_slice(row.amount.dong().to_string().as_bytes());
    buf.push(b',');
    buf.extend_from_slice(row.settled_at.to_string().as_bytes());
    buf.push(b'\n');
}

/// Bảng tạm một dòng giữ tham số của run, để các câu `CREATE TABLE AS` (không nhận `$n`) đọc được.
async fn create_params(conn: &mut PgConnection, run_id: i64, period: &Period) -> Result<()> {
    let (window_start, window_end) = period.window()?;
    sqlx::query(
        "CREATE TEMP TABLE run_params (run_id bigint NOT NULL, \
             period_start timestamptz NOT NULL, period_end timestamptz NOT NULL, \
             window_start timestamptz NOT NULL, window_end timestamptz NOT NULL) ON COMMIT DROP",
    )
    .execute(&mut *conn)
    .await?;
    sqlx::query(
        "INSERT INTO run_params VALUES \
             ($1, $2::timestamptz, $3::timestamptz, $4::timestamptz, $5::timestamptz)",
    )
    .bind(run_id)
    .bind(period.start.to_string())
    .bind(period.end.to_string())
    .bind(window_start.to_string())
    .bind(window_end.to_string())
    .execute(&mut *conn)
    .await?;
    Ok(())
}

async fn find_run(pool: &PgPool, sha: &str, period: &Period) -> Result<Option<i64>> {
    Ok(sqlx::query_scalar(
        "SELECT id FROM reconciliation_runs WHERE file_sha256 = $1 \
         AND period_start = $2::timestamptz AND period_end = $3::timestamptz",
    )
    .bind(sha)
    .bind(period.start.to_string())
    .bind(period.end.to_string())
    .fetch_optional(pool)
    .await?)
}

/// Đọc lại kết quả đã lưu của một run.
///
/// # Errors
/// Run không tồn tại hoặc lỗi cơ sở dữ liệu.
pub async fn load_summary(pool: &PgPool, run_id: i64) -> Result<RunSummary> {
    let row = sqlx::query(
        "SELECT file_sha256, statement_rows, bad_lines, ledger_rows, refs_matched, \
                refs_matched_split, refs_out_of_period \
         FROM reconciliation_runs WHERE id = $1",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await?;
    let mut exceptions = BTreeMap::new();
    let kinds = sqlx::query(
        "SELECT kind, count(*)::bigint FROM reconciliation_items \
         WHERE run_id = $1 AND kind <> 'BAD_LINE' GROUP BY kind",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await?;
    for k in &kinds {
        let name: String = k.get(0);
        let kind = Kind::parse(&name).with_context(|| format!("loại lạ: {name}"))?;
        exceptions.insert(kind, k.get::<i64, _>(1));
    }
    Ok(RunSummary {
        run_id,
        file_sha256: row.get(0),
        statement_rows: row.get(1),
        bad_lines: row.get(2),
        ledger_rows: row.get(3),
        refs_matched: row.get(4),
        refs_matched_split: row.get(5),
        refs_out_of_period: row.get(6),
        exceptions,
    })
}

/// Một dòng `reconciliation_items` đọc lại để in báo cáo hoặc so sánh.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// Loại.
    pub kind: Kind,
    /// Mã giao dịch (trống với dòng hỏng).
    pub external_ref: Option<String>,
    /// Số dòng trong file (chỉ dòng hỏng).
    pub line_no: Option<i64>,
    /// Tổng phía ta.
    pub ours_dong: Option<i64>,
    /// Tổng phía VíXanh.
    pub psp_dong: Option<i64>,
    /// Chênh lệch `psp - ours`: dương nghĩa là VíXanh báo nhiều hơn sổ ta.
    pub diff_dong: Option<i64>,
    /// Số dòng phía ta.
    pub ours_rows: Option<i32>,
    /// Số dòng phía VíXanh.
    pub psp_rows: Option<i32>,
    /// Lý do.
    pub detail: String,
    /// Trạng thái việc: `OPEN` khi mới mở cho Finance.
    pub status: String,
}

/// Đọc ngoại lệ của một run, theo thứ tự báo cáo (loại, rồi `external_ref`, rồi số dòng).
///
/// # Errors
/// Lỗi cơ sở dữ liệu.
pub async fn load_items(pool: &PgPool, run_id: i64) -> Result<Vec<Item>> {
    let rows = sqlx::query(
        "SELECT kind, external_ref, line_no, ours_dong, psp_dong, diff_dong, \
                ours_rows, psp_rows, detail, status \
         FROM reconciliation_items WHERE run_id = $1 \
         ORDER BY CASE kind WHEN 'MISSING_IN_LEDGER' THEN 1 WHEN 'MISSING_IN_PSP' THEN 2 \
                  WHEN 'AMOUNT_MISMATCH' THEN 3 WHEN 'DUPLICATE' THEN 4 \
                  WHEN 'DATE_SHIFT' THEN 5 ELSE 6 END, \
                  external_ref COLLATE \"C\", line_no, id",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await?;
    rows.iter()
        .map(|r| {
            let name: String = r.get(0);
            Ok(Item {
                kind: Kind::parse(&name).with_context(|| format!("loại lạ: {name}"))?,
                external_ref: r.get(1),
                line_no: r.get(2),
                ours_dong: r.get(3),
                psp_dong: r.get(4),
                diff_dong: r.get(5),
                ours_rows: r.get(6),
                psp_rows: r.get(7),
                detail: r.get(8),
                status: r.get(9),
            })
        })
        .collect()
}

/// Thời gian các pha của khớp trong Rust, chỉ để đo.
#[derive(Debug, Clone, Copy)]
pub struct MemoryTimings {
    /// Đọc CSV vào bộ nhớ.
    pub read_statement: Duration,
    /// Đọc dòng sổ cái từ PostgreSQL vào bộ nhớ.
    pub read_ledger: Duration,
    /// Phân loại.
    pub classify: Duration,
}

/// Khớp trong Rust: nạp CẢ HAI phía vào `HashMap`, rồi phân loại. Không ghi gì vào cơ sở dữ liệu.
/// Có để so kết quả với khớp bằng SQL và để đo cái giá bộ nhớ.
///
/// # Errors
/// File lỗi hoặc lỗi cơ sở dữ liệu.
pub async fn match_in_memory(
    pool: &PgPool,
    path: &Path,
    period: &Period,
) -> Result<(MatchResult, MemoryTimings)> {
    let mut matcher = Matcher::default();

    let t = Instant::now();
    let mut reader = StatementReader::new(File::open(path)?, 0)?;
    while let Some(row) = reader.next_row()? {
        matcher.add_psp(
            &row.external_ref,
            Row {
                amount: row.amount.dong(),
                at: row.settled_at,
            },
        );
    }
    let read_statement = t.elapsed();

    let t = Instant::now();
    let mut tx = pool.begin().await?;
    create_params(&mut tx, 0, period).await?;
    let sql = format!(
        "SELECT external_ref, amount_dong, (extract(epoch FROM posted_at) * 1000000)::bigint \
         FROM ({OURS_SELECT}) s"
    );
    {
        let mut rows = sqlx::query(AssertSqlSafe(sql)).fetch(&mut *tx);
        while let Some(r) = rows.try_next().await? {
            let external_ref: String = r.get(0);
            let micros: i64 = r.get(2);
            matcher.add_ours(
                &external_ref,
                Row {
                    amount: r.get(1),
                    at: Timestamp::from_microsecond(micros)?,
                },
            );
        }
    }
    tx.rollback().await?;
    let read_ledger = t.elapsed();

    let t = Instant::now();
    let result = matcher.finish(period);
    let classify = t.elapsed();
    Ok((
        result,
        MemoryTimings {
            read_statement,
            read_ledger,
            classify,
        },
    ))
}

/// Chuyển ngoại lệ đã lưu thành dạng của [`Exception`] để so với khớp trong Rust.
/// Bỏ dòng hỏng (khớp trong Rust không có khái niệm đó).
pub fn items_as_exceptions(items: &[Item]) -> Vec<Exception> {
    let mut list: Vec<Exception> = items
        .iter()
        .filter(|i| i.kind != Kind::BadLine)
        .filter_map(|i| {
            Some(Exception {
                kind: i.kind,
                external_ref: i.external_ref.clone()?,
                ours_dong: i.ours_dong,
                psp_dong: i.psp_dong,
                ours_rows: i.ours_rows.unwrap_or(0),
                psp_rows: i.psp_rows.unwrap_or(0),
            })
        })
        .collect();
    list.sort_by(|a, b| a.external_ref.cmp(&b.external_ref));
    list
}

#[cfg(test)]
mod tests {
    use ledger_core::Money;

    use super::*;

    #[test]
    fn copy_line_quotes_only_when_needed() {
        let mut row = StatementRow {
            line_no: 7,
            external_ref: "vx_ch_1".to_string(),
            amount: Money::from_dong(-500),
            settled_at: "2026-11-05T01:02:03Z".parse().unwrap(),
        };
        let mut buf = Vec::new();
        push_copy_line(&mut buf, &row);
        assert_eq!(buf, b"7,vx_ch_1,-500,2026-11-05T01:02:03Z\n");
        row.external_ref = "a,\"b\"".to_string();
        buf.clear();
        push_copy_line(&mut buf, &row);
        assert_eq!(buf, b"7,\"a,\"\"b\"\"\",-500,2026-11-05T01:02:03Z\n");
    }
}
