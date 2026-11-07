//! Test tích hợp với PostgreSQL thật. Mỗi ca có schema `lab_<uuid>` riêng (xem `lab.rs`).
//! Không test nào assert thời gian chạy hay dùng `sleep`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use jiff::Timestamp;
use ledger_47_psp_reconciliation::bench::{check_summary, cross_check, expected_counts};
use ledger_47_psp_reconciliation::demo::{sample_dir, sample_file, sample_period};
use ledger_47_psp_reconciliation::job::{JobOutcome, run_daily_job};
use ledger_47_psp_reconciliation::kind::Kind;
use ledger_47_psp_reconciliation::lab::with_lab;
use ledger_47_psp_reconciliation::period::{FixedClock, Period};
use ledger_47_psp_reconciliation::reconcile::{
    MAX_BAD_ITEMS, RunOutcome, RunRequest, load_items, reconcile_expecting, reconcile_file,
};
use ledger_47_psp_reconciliation::seed::{ledger_fingerprint, post_capture, seed_sample_ledger};
use ledger_47_psp_reconciliation::statement::sha256_file;
use sqlx::PgPool;
use tokio::sync::Barrier;
use uuid::Uuid;

/// Thư mục tạm, tự xoá khi hết phạm vi.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> TempDir {
        let dir = std::env::temp_dir().join(format!("lab47_test_{}", Uuid::new_v4().simple()));
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }

    fn write(&self, name: &str, text: &str) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, text).unwrap();
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn request(path: &Path) -> RunRequest<'_> {
    RunRequest {
        provider: "vixanh",
        path,
        period: sample_period().unwrap(),
        run_at: "2026-11-06T01:30:00+07:00".parse().unwrap(),
    }
}

async fn runs(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM reconciliation_runs")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn sample_statement_is_classified() -> Result<()> {
    with_lab(async move |lab| {
        seed_sample_ledger(&lab.pool).await?;
        let file = sample_file();
        let outcome = reconcile_file(&lab.pool, &request(&file)).await?;
        let s = outcome.summary();
        assert_eq!((s.statement_rows, s.bad_lines, s.ledger_rows), (11, 4, 10));
        assert_eq!(
            (s.refs_matched, s.refs_matched_split, s.refs_out_of_period),
            (4, 2, 1)
        );
        let items = load_items(&lab.pool, s.run_id).await?;
        let found: Vec<(Kind, Option<&str>, Option<i64>)> = items
            .iter()
            .map(|i| (i.kind, i.external_ref.as_deref(), i.diff_dong))
            .collect();
        assert_eq!(
            found,
            [
                (Kind::MissingInLedger, Some("vx_ch_0005"), Some(52_000)),
                (Kind::MissingInPsp, Some("vx_ch_0004"), Some(-18_000)),
                (Kind::AmountMismatch, Some("vx_ch_0003"), Some(-500)),
                (Kind::Duplicate, Some("vx_ch_0006"), Some(22_000)),
                (Kind::DateShift, Some("vx_ch_0007"), Some(0)),
                (Kind::BadLine, None, None),
                (Kind::BadLine, None, None),
                (Kind::BadLine, None, None),
                (Kind::BadLine, None, None),
            ]
        );
        let lines: Vec<Option<i64>> = items.iter().skip(5).map(|i| i.line_no).collect();
        assert_eq!(lines, [Some(10), Some(14), Some(15), Some(16)]);
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn same_file_twice_is_one_run_and_edited_file_is_a_new_run() -> Result<()> {
    with_lab(async move |lab| {
        seed_sample_ledger(&lab.pool).await?;
        let file = sample_file();
        let first = reconcile_file(&lab.pool, &request(&file)).await?;
        let second = reconcile_file(&lab.pool, &request(&file)).await?;
        assert!(matches!(first, RunOutcome::Created { .. }));
        assert!(matches!(second, RunOutcome::AlreadyDone(_)));
        assert_eq!(first.summary(), second.summary());
        assert_eq!(runs(&lab.pool).await, 1);

        let tmp = TempDir::new();
        let mut text = std::fs::read_to_string(&file)?;
        text.push_str("vx_ch_0013,8000,2026-11-05T11:00:00Z\n");
        let edited = tmp.write("edited.csv", &text);
        let third = reconcile_file(&lab.pool, &request(&edited)).await?;
        assert!(matches!(third, RunOutcome::Created { .. }));
        assert_ne!(third.summary().file_sha256, first.summary().file_sha256);
        assert_eq!(
            third.summary().exceptions.get(&Kind::MissingInLedger),
            Some(&2)
        );
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrent_runs_of_one_file_create_exactly_one_run() -> Result<()> {
    with_lab(async move |lab| {
        seed_sample_ledger(&lab.pool).await?;
        let racers = 6;
        let barrier = Arc::new(Barrier::new(racers));
        let mut tasks = Vec::new();
        for _ in 0..racers {
            let (pool, barrier) = (lab.pool.clone(), barrier.clone());
            tasks.push(tokio::spawn(async move {
                barrier.wait().await;
                let file = sample_file();
                reconcile_file(&pool, &request(&file)).await
            }));
        }
        let mut created = 0;
        let mut ids = Vec::new();
        for task in tasks {
            let outcome = task.await??;
            created += usize::from(matches!(outcome, RunOutcome::Created { .. }));
            ids.push(outcome.summary().run_id);
        }
        assert_eq!(created, 1);
        assert!(ids.windows(2).all(|w| w[0] == w[1]));
        assert_eq!(runs(&lab.pool).await, 1);
        let items: i64 = sqlx::query_scalar("SELECT count(*) FROM reconciliation_items")
            .fetch_one(&lab.pool)
            .await?;
        assert_eq!(items, 9);
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn reconciliation_never_writes_the_ledger_and_opens_tasks() -> Result<()> {
    with_lab(async move |lab| {
        seed_sample_ledger(&lab.pool).await?;
        let before = ledger_fingerprint(&lab.pool).await?;
        let file = sample_file();
        reconcile_file(&lab.pool, &request(&file)).await?;
        assert_eq!(ledger_fingerprint(&lab.pool).await?, before);
        let not_open: i64 =
            sqlx::query_scalar("SELECT count(*) FROM reconciliation_items WHERE status <> 'OPEN'")
                .fetch_one(&lab.pool)
                .await?;
        assert_eq!(not_open, 0);
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn file_changed_between_hash_and_load_is_rolled_back() -> Result<()> {
    with_lab(async move |lab| {
        seed_sample_ledger(&lab.pool).await?;
        let file = sample_file();
        let wrong = "0".repeat(64);
        let err = reconcile_expecting(&lab.pool, &request(&file), &wrong)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("file đổi"), "{err}");
        assert_eq!(runs(&lab.pool).await, 0);
        // Sau lỗi, chạy đúng vẫn thành công: không có gì kẹt lại.
        let good = sha256_file(&file)?;
        let ok = reconcile_expecting(&lab.pool, &request(&file), &good).await?;
        assert!(matches!(ok, RunOutcome::Created { .. }));
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn wrong_header_rejects_the_whole_file_without_a_run() -> Result<()> {
    with_lab(async move |lab| {
        let tmp = TempDir::new();
        let file = tmp.write("bad_header.csv", "ref,amount\nvx_ch_1,100\n");
        let err = reconcile_file(&lab.pool, &request(&file))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("tiêu đề"), "{err}");
        assert_eq!(runs(&lab.pool).await, 0);
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_line_details_are_capped_but_counted() -> Result<()> {
    with_lab(async move |lab| {
        let tmp = TempDir::new();
        let mut text = String::from("external_ref,amount_vnd,settled_at\n");
        text.push_str("vx_ch_1,100,2026-11-05T01:00:00Z\n");
        for _ in 0..MAX_BAD_ITEMS + 500 {
            text.push_str("vx_ch_x,abc,2026-11-05T01:00:00Z\n");
        }
        let file = tmp.write("many_bad.csv", &text);
        let outcome = reconcile_file(&lab.pool, &request(&file)).await?;
        let s = outcome.summary();
        assert_eq!(s.bad_lines, i64::try_from(MAX_BAD_ITEMS + 500)?);
        let kept: i64 =
            sqlx::query_scalar("SELECT count(*) FROM reconciliation_items WHERE kind = 'BAD_LINE'")
                .fetch_one(&lab.pool)
                .await?;
        assert_eq!(kept, i64::try_from(MAX_BAD_ITEMS)?);
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn duplicate_posting_on_our_side_is_told_apart_from_a_mismatch() -> Result<()> {
    with_lab(async move |lab| {
        let at: Timestamp = "2026-11-05T05:00:00Z".parse()?;
        // Ta ghi hai lần cùng giao dịch (cùng ref, cùng số tiền); VíXanh chỉ có một dòng.
        post_capture(&lab.pool, "k:1", "vx_ch_77", "customer:CU-001", 9_000, at).await?;
        post_capture(&lab.pool, "k:2", "vx_ch_77", "customer:CU-001", 9_000, at).await?;
        let tmp = TempDir::new();
        let file = tmp.write(
            "s.csv",
            "external_ref,amount_vnd,settled_at\nvx_ch_77,9000,2026-11-05T05:00:05Z\n",
        );
        let outcome = reconcile_file(&lab.pool, &request(&file)).await?;
        let items = load_items(&lab.pool, outcome.summary().run_id).await?;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, Kind::Duplicate);
        assert_eq!(items[0].diff_dong, Some(-9_000));
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn job_follows_the_injected_clock() -> Result<()> {
    with_lab(async move |lab| {
        seed_sample_ledger(&lab.pool).await?;
        let dir = sample_dir();
        let now: Timestamp = "2026-11-06T01:30:00+07:00".parse()?;
        let first = run_daily_job(&lab.pool, &FixedClock(now), &dir).await?;
        let again = run_daily_job(&lab.pool, &FixedClock(now), &dir).await?;
        assert!(matches!(
            first,
            JobOutcome::Reconciled {
                outcome: RunOutcome::Created { .. },
                ..
            }
        ));
        assert!(matches!(
            again,
            JobOutcome::Reconciled {
                outcome: RunOutcome::AlreadyDone(_),
                ..
            }
        ));
        let tomorrow = FixedClock("2026-11-07T01:30:00+07:00".parse()?);
        assert!(matches!(
            run_daily_job(&lab.pool, &tomorrow, &dir).await?,
            JobOutcome::NoStatement { .. }
        ));
        let stored: i64 = sqlx::query_scalar(
            "SELECT extract(epoch FROM run_at)::bigint FROM reconciliation_runs",
        )
        .fetch_one(&lab.pool)
        .await?;
        assert_eq!(stored, now.as_second());
        // 16:30 UTC ngày 5 vẫn là ngày 5 ở Việt Nam, nên «hôm qua» là ngày 4: chưa có file.
        let early = FixedClock("2026-11-05T16:30:00Z".parse()?);
        assert!(matches!(
            run_daily_job(&lab.pool, &early, &dir).await?,
            JobOutcome::NoStatement { .. }
        ));
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn sql_and_rust_agree_on_generated_data() -> Result<()> {
    let file = std::env::temp_dir().join(format!("lab47_gen_test_{}.csv", Uuid::new_v4().simple()));
    let path = file.clone();
    let result = with_lab(async move |lab| cross_check(&lab.pool, &path, 30_000).await).await;
    let _ = std::fs::remove_file(&file);
    let cc = result?;
    assert!(cc.identical);
    check_summary(&cc.summary, &expected_counts(30_000))?;
    Ok(())
}

#[test]
fn period_of_sample_is_the_vietnam_business_day() {
    let p: Period = sample_period().unwrap();
    assert_eq!(p.start.to_string(), "2026-11-04T17:00:00Z");
}

#[tokio::test(flavor = "multi_thread")]
async fn date_shift_shows_up_on_both_sides_of_the_cut() -> Result<()> {
    with_lab(async move |lab| {
        seed_sample_ledger(&lab.pool).await?;
        let file = sample_file();
        let day5 = reconcile_file(&lab.pool, &request(&file)).await?;
        assert_eq!(
            day5.summary().exceptions.get(&Kind::DateShift),
            Some(&1),
            "kỳ ngày 5 báo vx_ch_0007"
        );
        // Cùng file, nhưng đối soát kỳ 2026-11-06: giao dịch 23:58 ngày 5 hiện ra từ phía VíXanh.
        // Khóa idempotent gồm cả kỳ, nên đây là run mới chứ không phải kết quả cũ của kỳ ngày 5.
        let next = Period::business_day(jiff::civil::date(2026, 11, 6))?;
        let req = RunRequest {
            period: next,
            ..request(&file)
        };
        let outcome = reconcile_file(&lab.pool, &req).await?;
        assert!(matches!(outcome, RunOutcome::Created { .. }));
        assert_eq!(runs(&lab.pool).await, 2);
        let items = load_items(&lab.pool, outcome.summary().run_id).await?;
        let shifts: Vec<&str> = items
            .iter()
            .filter(|i| i.kind == Kind::DateShift)
            .filter_map(|i| i.external_ref.as_deref())
            .collect();
        assert_eq!(shifts, ["vx_ch_0007"]);
        // vx_ch_0009 nằm trong kỳ 6 ở cả hai phía nên khớp bình thường; các ref còn lại thuộc kỳ 5.
        assert_eq!(outcome.summary().refs_matched, 1);
        assert_eq!(outcome.summary().refs_out_of_period, 8);
        Ok(())
    })
    .await
}
