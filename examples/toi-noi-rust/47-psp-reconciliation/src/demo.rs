//! Kịch bản chạy thật của bài 47; `--verify` kiểm các giá trị bài viết nêu.

use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use jiff::Timestamp;
use jiff::civil::date;
use sqlx::PgPool;
use uuid::Uuid;

use crate::bench::{check_summary, cross_check, expected_counts};
use crate::job::{JobOutcome, run_daily_job};
use crate::kind::Kind;
use crate::lab::with_lab;
use crate::memory::Exception;
use crate::period::{FixedClock, Period};
use crate::reconcile::{
    Item, RunOutcome, RunRequest, RunSummary, items_as_exceptions, load_items, match_in_memory,
    reconcile_file,
};
use crate::report::write_report;
use crate::seed::{ledger_fingerprint, seed_sample_ledger};
use crate::statement::{HashingReader, StatementReader};

/// Sao kê mẫu nằm trong lab.
pub fn sample_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

/// Đường dẫn file sao kê mẫu.
pub fn sample_file() -> PathBuf {
    sample_dir().join("vixanh_2026-11-05.csv")
}

/// Kỳ của sao kê mẫu: ngày 2026-11-05 giờ Việt Nam.
///
/// # Errors
/// Lỗi tzdb.
pub fn sample_period() -> Result<Period> {
    Ok(Period::business_day(date(2026, 11, 5))?)
}

/// Gom kiểm tra: chỉ ném lỗi khi `--verify`.
struct Check {
    verify: bool,
}

impl Check {
    fn that(&self, ok: bool, what: &str) -> Result<()> {
        if self.verify && !ok {
            bail!("VERIFY FAIL: {what}");
        }
        Ok(())
    }
}

fn describe_exception(e: &Exception) -> String {
    format!(
        "{:<17} {} ours={:?} psp={:?}",
        e.kind.as_str(),
        e.external_ref,
        e.ours_dong,
        e.psp_dong
    )
}

fn print_summary(s: &RunSummary) {
    println!(
        "run {}: {} dòng sao kê hợp lệ, {} dòng hỏng, {} dòng sổ trong cửa sổ",
        s.run_id, s.statement_rows, s.bad_lines, s.ledger_rows
    );
    println!(
        "  khớp {} ref (trong đó {} có tách nhiều dòng), ngoài kỳ {} ref",
        s.refs_matched, s.refs_matched_split, s.refs_out_of_period
    );
    for (kind, n) in &s.exceptions {
        println!("  {:<17}: {n}", kind.as_str());
    }
}

fn print_item(i: &Item) {
    match i.kind {
        Kind::BadLine => println!(
            "  BAD_LINE          dòng {}: {}",
            i.line_no.unwrap_or(0),
            i.detail
        ),
        _ => println!(
            "  {:<17} {} ours={:?} psp={:?} diff={:?} [{}]",
            i.kind.as_str(),
            i.external_ref.as_deref().unwrap_or("?"),
            i.ours_dong,
            i.psp_dong,
            i.diff_dong,
            i.status
        ),
    }
}

/// Ghi bản sao của file mẫu kèm thêm một dòng, để có hash khác.
fn variant(dir: &Path, name: &str, extra: &str) -> Result<PathBuf> {
    let mut text = std::fs::read_to_string(sample_file())?;
    text.push_str(extra);
    let path = dir.join(name);
    std::fs::write(&path, text)?;
    Ok(path)
}

async fn count(pool: &PgPool, table: &'static str) -> Result<i64> {
    let sql = match table {
        "runs" => "SELECT count(*) FROM reconciliation_runs",
        _ => "SELECT count(*) FROM reconciliation_items",
    };
    Ok(sqlx::query_scalar(sql).fetch_one(pool).await?)
}

/// Chạy mọi phần của bài. `verify` bật kiểm giá trị.
///
/// # Errors
/// Lỗi hạ tầng, hoặc (khi `verify`) giá trị khác bài viết.
pub async fn run(verify: bool) -> Result<()> {
    let check = Check { verify };
    reconcile_sample(&check).await?;
    job_demo(&check).await?;
    large_cross_check(&check).await?;
    if verify {
        println!("VERIFY OK");
    }
    Ok(())
}

async fn reconcile_sample(check: &Check) -> Result<()> {
    with_lab(|lab| {
        let check = Check {
            verify: check.verify,
        };
        async move {
            let pool = lab.pool.clone();
            let n = seed_sample_ledger(&pool).await?;
            let before = ledger_fingerprint(&pool).await?;
            let file = sample_file();
            let period = sample_period()?;
            let run_at: Timestamp = "2026-11-06T01:30:00+07:00".parse()?;

            println!("== 1. Đọc streaming sao kê mẫu (chưa chạm PostgreSQL) ==");
            let mut reader = StatementReader::new(HashingReader::new(File::open(&file)?), 100)?;
            let mut refs = Vec::new();
            while let Some(row) = reader.next_row()? {
                refs.push((row.line_no, row.external_ref));
            }
            println!(
                "dòng hợp lệ {}, dòng hỏng {}; dòng hợp lệ cuối: {:?}",
                reader.good(),
                reader.bad_total(),
                refs.last()
            );
            for bad in reader.bad_lines() {
                println!("  dòng {}: {}", bad.line_no, bad.reason);
            }
            let bad_lines: Vec<u64> = reader.bad_lines().iter().map(|b| b.line_no).collect();
            let sha = reader.into_inner().finish_hex();
            println!("SHA-256 của file: {sha}");
            check.that(refs.len() == 11, "11 dòng hợp lệ")?;
            check.that(bad_lines == [10, 14, 15, 16], "dòng hỏng 10, 14, 15, 16")?;
            check.that(
                refs.last().is_some_and(|(l, r)| *l == 13 && r == "vx_ch_0010"),
                "dòng hợp lệ cuối là dòng 13",
            )?;

            println!("== 2. Kỳ đối soát ==");
            println!(
                "ngày 2026-11-05 giờ Asia/Ho_Chi_Minh = [{}, {})",
                period.start, period.end
            );
            check.that(
                period.start.to_string() == "2026-11-04T17:00:00Z"
                    && period.end.to_string() == "2026-11-05T17:00:00Z",
                "kỳ 17:00Z đến 17:00Z",
            )?;
            println!("phía sổ cái mẫu: {n} bút toán thu tiền qua VíXanh");

            println!("== 3. Khớp bằng SQL: COPY vào bảng tạm, FULL JOIN theo external_ref ==");
            let req = RunRequest {
                provider: "vixanh",
                path: &file,
                period,
                run_at,
            };
            let first = reconcile_file(&pool, &req).await?;
            let RunOutcome::Created { summary, .. } = &first else {
                bail!("lần đầu phải tạo run mới");
            };
            print_summary(summary);
            let items = load_items(&pool, summary.run_id).await?;
            for item in &items {
                print_item(item);
            }
            check.that(summary.file_sha256 == sha, "hash khi băm trước = hash khi đọc")?;
            check.that(
                summary.refs_matched == 4 && summary.refs_matched_split == 2,
                "4 ref khớp, 2 ref tách dòng",
            )?;
            check.that(summary.refs_out_of_period == 1, "1 ref ngoài kỳ")?;
            let kinds: Vec<(Kind, i64)> = summary.exceptions.iter().map(|(k, v)| (*k, *v)).collect();
            check.that(
                kinds
                    == [
                        (Kind::MissingInLedger, 1),
                        (Kind::MissingInPsp, 1),
                        (Kind::AmountMismatch, 1),
                        (Kind::Duplicate, 1),
                        (Kind::DateShift, 1),
                    ],
                "mỗi loại lệch đúng một ref",
            )?;
            let mismatch = items.iter().find(|i| i.kind == Kind::AmountMismatch);
            check.that(
                mismatch.is_some_and(|i| {
                    i.external_ref.as_deref() == Some("vx_ch_0003")
                        && i.ours_dong == Some(31_500)
                        && i.psp_dong == Some(31_000)
                        && i.diff_dong == Some(-500)
                }),
                "vx_ch_0003 lệch -500",
            )?;
            check.that(items.len() == 9, "5 ngoại lệ + 4 dòng hỏng")?;

            println!("== 4. Khớp trong Rust (HashMap) so với SQL ==");
            let (memory, _) = match_in_memory(&pool, &file, &period).await?;
            for e in &memory.exceptions {
                println!("  {}", describe_exception(e));
            }
            let same = items_as_exceptions(&items) == memory.exceptions;
            println!(
                "khớp {} ref; danh sách ngoại lệ giống SQL: {same}",
                memory.matched
            );
            check.that(same, "SQL và Rust cho cùng ngoại lệ")?;
            check.that(memory.matched == 4, "Rust cũng khớp 4 ref")?;

            println!("== 5. Chạy lại cùng file, sửa file, chạy đồng thời ==");
            let again = reconcile_file(&pool, &req).await?;
            let again_same = matches!(&again, RunOutcome::AlreadyDone(s) if s.run_id == summary.run_id);
            println!(
                "chạy lại cùng file    : AlreadyDone run {} ; runs = {}, items = {}",
                again.summary().run_id,
                count(&pool, "runs").await?,
                count(&pool, "items").await?
            );
            check.that(again_same, "chạy lại trả run cũ")?;
            check.that(
                count(&pool, "runs").await? == 1 && count(&pool, "items").await? == 9,
                "vẫn 1 run, 9 item",
            )?;

            let dir = std::env::temp_dir().join(format!("lab47_{}", Uuid::new_v4().simple()));
            std::fs::create_dir_all(&dir)?;
            let edited = variant(&dir, "edited.csv", "vx_ch_0013,8000,2026-11-05T11:00:00Z\n")?;
            let req_edited = RunRequest { path: &edited, ..req };
            let third = reconcile_file(&pool, &req_edited).await?;
            let edited_new = matches!(third, RunOutcome::Created { .. });
            println!(
                "file thêm 1 dòng      : {} run {} (hash khác nên là run mới) ; runs = {}",
                if edited_new { "Created" } else { "AlreadyDone" },
                third.summary().run_id,
                count(&pool, "runs").await?
            );
            check.that(edited_new, "file đổi một dòng là run mới")?;

            let racy = variant(&dir, "racy.csv", "vx_ch_0014,9000,2026-11-05T12:00:00Z\n")?;
            let racers = 4;
            let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(racers));
            let mut tasks = Vec::new();
            for _ in 0..racers {
                let (pool, barrier, racy) = (pool.clone(), barrier.clone(), racy.clone());
                tasks.push(tokio::spawn(async move {
                    barrier.wait().await;
                    let req = RunRequest {
                        provider: "vixanh",
                        path: &racy,
                        period,
                        run_at,
                    };
                    reconcile_file(&pool, &req).await
                }));
            }
            let (mut created, mut already) = (0, 0);
            for task in tasks {
                match task.await?? {
                    RunOutcome::Created { .. } => created += 1,
                    RunOutcome::AlreadyDone(_) => already += 1,
                }
            }
            let runs_now = count(&pool, "runs").await?;
            println!(
                "4 lần chạy cùng lúc   : {created} Created, {already} AlreadyDone ; runs = {runs_now}"
            );
            check.that(created == 1 && already == 3, "đúng một lần tạo run")?;
            check.that(runs_now == 3, "tổng 3 run (mẫu, thêm 1 dòng, chạy đồng thời)")?;
            std::fs::remove_dir_all(&dir)?;

            // Mọi lần chạy trên là bản đọc/ghi trong bảng đối soát, không đụng sổ.
            println!("== 6. Lệch không tự sửa, chỉ mở việc ==");
            let after = ledger_fingerprint(&pool).await?;
            let open: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM reconciliation_items WHERE status = 'OPEN'",
            )
            .fetch_one(&pool)
            .await?;
            let total_items = count(&pool, "items").await?;
            println!(
                "sổ cái trước/sau 3 run: {before:?} / {after:?} (bút toán, posting, tổng |đồng|)"
            );
            println!("việc mở cho Finance   : {open}/{total_items} item ở trạng thái OPEN");
            check.that(before == after, "đối soát không ghi bút toán")?;
            check.that(open == total_items, "mọi item đều OPEN")?;

            println!("== 7. Báo cáo CSV cho Finance (run {}) ==", summary.run_id);
            let mut buf = Vec::new();
            write_report(&mut buf, &items)?;
            let text = String::from_utf8(buf)?;
            print!("{text}");
            check.that(text.lines().count() == 10, "báo cáo có tiêu đề + 9 dòng")?;
            Ok(())
        }
    })
    .await
}

async fn job_demo(check: &Check) -> Result<()> {
    with_lab(|lab| {
        let check = Check {
            verify: check.verify,
        };
        async move {
            let pool = lab.pool.clone();
            seed_sample_ledger(&pool).await?;
            println!("== 8. Job định kỳ với đồng hồ truyền vào ==");
            let dir = sample_dir();
            let monday: Timestamp = "2026-11-06T01:30:00+07:00".parse()?;
            let clock = FixedClock(monday);
            let first = run_daily_job(&pool, &clock, &dir).await?;
            let second = run_daily_job(&pool, &clock, &dir).await?;
            let next_day = FixedClock("2026-11-07T01:30:00+07:00".parse()?);
            let third = run_daily_job(&pool, &next_day, &dir).await?;
            let label = |o: &JobOutcome| match o {
                JobOutcome::NoStatement { date, expected } => format!(
                    "chưa có file cho {date} (cần {})",
                    expected
                        .file_name()
                        .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
                ),
                JobOutcome::Reconciled { date, outcome } => format!(
                    "{date}: {} run {}",
                    if matches!(outcome, RunOutcome::Created { .. }) {
                        "Created"
                    } else {
                        "AlreadyDone"
                    },
                    outcome.summary().run_id
                ),
            };
            println!("đồng hồ {monday}, lần 1: {}", label(&first));
            println!("đồng hồ {monday}, lần 2: {}", label(&second));
            println!("đồng hồ 2026-11-06T18:30:00Z, lần 3: {}", label(&third));
            let run_at: String = sqlx::query_scalar(
                "SELECT to_char(run_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') \
                 FROM reconciliation_runs",
            )
            .fetch_one(&pool)
            .await?;
            println!("run_at đã lưu          : {run_at} (đúng thời điểm của đồng hồ truyền vào)");
            check.that(
                matches!(
                    &first,
                    JobOutcome::Reconciled {
                        outcome: RunOutcome::Created { .. },
                        ..
                    }
                ),
                "lần 1 tạo run",
            )?;
            check.that(
                matches!(
                    &second,
                    JobOutcome::Reconciled {
                        outcome: RunOutcome::AlreadyDone(_),
                        ..
                    }
                ),
                "lần 2 là AlreadyDone",
            )?;
            check.that(
                matches!(&third, JobOutcome::NoStatement { .. }),
                "ngày sau chưa có file",
            )?;
            check.that(
                run_at == "2026-11-05T18:30:00Z",
                "run_at là giờ của đồng hồ giả",
            )?;
            check.that(count(&pool, "runs").await? == 1, "job chỉ tạo một run")?;
            Ok(())
        }
    })
    .await
}

async fn large_cross_check(check: &Check) -> Result<()> {
    let rows = 50_000;
    let file = std::env::temp_dir().join(format!("lab47_gen_{}.csv", Uuid::new_v4().simple()));
    let path = file.clone();
    let result = with_lab(async move |lab| cross_check(&lab.pool, &path, rows).await).await;
    let _ = std::fs::remove_file(&file);
    let cc = result?;
    println!("== 9. {rows} ref sinh sẵn: khớp bằng SQL và bằng Rust ==");
    print_summary(&cc.summary);
    println!("ngoại lệ giống hệt nhau giữa SQL và Rust: {}", cc.identical);
    check.that(cc.identical, "SQL và Rust giống hệt trên dữ liệu sinh")?;
    if check.verify {
        check_summary(&cc.summary, &expected_counts(rows))?;
    }
    Ok(())
}
