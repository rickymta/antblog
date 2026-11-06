//! Test với PostgreSQL thật; mỗi ca có schema `lab_<uuid>` riêng và được dọn kể cả khi lỗi.

use anyhow::Result;
use jiff::civil::Date;
use ledger_48_snapshot_period_close::db::{pg_error, with_lab};
use ledger_48_snapshot_period_close::period::{PeriodError, close_period, seed_months};
use ledger_48_snapshot_period_close::post::{entry, post};
use ledger_48_snapshot_period_close::race::{run_close_first, run_write_then_close};
use ledger_48_snapshot_period_close::snapshot::{
    balance_full, balance_snapshot_delta, rebuild_snapshots, snapshot_day, verify_snapshots,
};
use ledger_48_snapshot_period_close::tz::{day_bounds, ts};

const PSP: &str = "psp:vixanh:clearing";

fn day(d: i8) -> Date {
    Date::constant(2026, 11, d)
}

#[tokio::test]
async fn guard_rejects_posting_into_closed_period() -> Result<()> {
    with_lab(|db| async move {
        db.migrate().await?;
        seed_months(&db.pool, 2026, 11, 2).await?;
        let e = entry(
            "trip:TN-0001:capture",
            &[("customer:CU-001", -25_500), (PSP, 25_500)],
        )?;
        let t = ts("2026-11-05T10:00:00+07:00")?;
        post(&db.pool, &e, t, t, None).await?;
        let mut conn = db.pool.acquire().await?;
        close_period(&mut conn, "2026-11", ts("2026-12-01T00:05:00+07:00")?).await?;
        drop(conn);

        let late = entry(
            "trip:TN-0001:late",
            &[("customer:CU-001", -1_000), (PSP, 1_000)],
        )?;
        let err = post(&db.pool, &late, t, t, None)
            .await
            .expect_err("phải bị từ chối");
        let (code, msg) = pg_error(&err).expect("lỗi từ PostgreSQL");
        assert_eq!(code, "23001");
        assert!(msg.contains("kỳ 2026-11 đã đóng"), "{msg}");

        // Chèn thẳng posting vào bút toán cũ của kỳ đã đóng cũng bị chặn.
        let err = sqlx::query(
            "insert into postings (entry_id, line_no, account_id, amount_dong) \
             select id, 3, 'psp:vixanh:clearing', 1 from journal_entries limit 1",
        )
        .execute(&db.pool)
        .await
        .expect_err("thêm posting vào kỳ đã đóng phải bị từ chối");
        let (code, msg) = pg_error(&err).expect("lỗi từ PostgreSQL");
        assert_eq!(code, "23001");
        assert!(msg.contains("kỳ 2026-11 đã đóng"), "{msg}");

        // Kỳ sau vẫn ghi được; ranh giới đúng 00:00 ngày 01/12 thuộc kỳ 12.
        let edge = ts("2026-12-01T00:00:00+07:00")?;
        post(&db.pool, &late, edge, edge, None).await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn posting_outside_any_period_is_rejected() -> Result<()> {
    with_lab(|db| async move {
        db.migrate().await?;
        seed_months(&db.pool, 2026, 11, 1).await?;
        let e = entry("k", &[("a", -1), ("b", 1)])?;
        let t = ts("2027-01-05T10:00:00+07:00")?;
        let err = post(&db.pool, &e, t, t, None)
            .await
            .expect_err("chưa có kỳ");
        let (code, msg) = pg_error(&err).expect("lỗi từ PostgreSQL");
        assert_eq!(code, "23001");
        assert!(msg.contains("chưa có kỳ kế toán"), "{msg}");
        Ok(())
    })
    .await
}

#[tokio::test]
async fn periods_close_in_order_and_cannot_reopen_or_overlap() -> Result<()> {
    with_lab(|db| async move {
        db.migrate().await?;
        seed_months(&db.pool, 2026, 10, 3).await?;
        let mut conn = db.pool.acquire().await?;
        let now = ts("2026-12-31T00:00:00+07:00")?;
        let err = close_period(&mut conn, "2026-11", now).await.expect_err("kỳ 10 còn mở");
        assert!(matches!(err, PeriodError::EarlierOpen { ref earlier, .. } if earlier == "2026-10"));
        close_period(&mut conn, "2026-10", now).await?;
        let err = close_period(&mut conn, "2026-10", now).await.expect_err("đã đóng");
        assert!(matches!(err, PeriodError::AlreadyClosed(_)));
        let err = close_period(&mut conn, "2027-01", now).await.expect_err("không có kỳ");
        assert!(matches!(err, PeriodError::NotFound(_)));
        drop(conn);

        let reopen = sqlx::query("update accounting_periods set status = 'OPEN', closed_at = null where label = '2026-10'")
            .execute(&db.pool)
            .await
            .expect_err("không mở lại kỳ đã đóng");
        assert_eq!(pg_error(&reopen).expect("lỗi PG").0, "23001");

        // EXCLUDE: kỳ chồng lên kỳ đang có bị từ chối (SQLSTATE 23P01).
        let overlap = sqlx::query(
            "insert into accounting_periods (label, starts_at, ends_at) \
             values ('x', '2026-11-15T00:00:00+07', '2026-12-15T00:00:00+07')",
        )
        .execute(&db.pool)
        .await
        .expect_err("kỳ chồng nhau");
        assert_eq!(pg_error(&overlap).expect("lỗi PG").0, "23P01");
        Ok(())
    })
    .await
}

#[tokio::test]
async fn adjustment_in_next_period_leaves_closed_snapshot_untouched() -> Result<()> {
    with_lab(|db| async move {
        db.migrate().await?;
        seed_months(&db.pool, 2026, 11, 2).await?;
        let split = entry(
            "trip:TN-0001:split",
            &[(PSP, -25_500), ("platform:revenue", 4_500), ("driver:DR-001", 21_000)],
        )?;
        let t = ts("2026-11-06T00:10:00+07:00")?;
        // Cần số dư dương ở psp để split hợp lý; ghi capture trước.
        let cap = entry("trip:TN-0001:capture", &[("customer:CU-001", -25_500), (PSP, 25_500)])?;
        post(&db.pool, &cap, t, t, None).await?;
        let id = post(&db.pool, &split, t, t, None).await?;
        let mut conn = db.pool.acquire().await?;
        close_period(&mut conn, "2026-11", ts("2026-12-01T00:05:00+07:00")?).await?;
        drop(conn);

        let adj = ts("2026-12-02T09:00:00+07:00")?;
        let rev = ledger_core::JournalEntry::reversal_of(
            ledger_core::EntryId::parse(&format!("JE-{id:06}")).expect("mã hợp lệ"),
            &split,
        )?;
        post(&db.pool, &rev, adj, adj, Some(id)).await?;
        let v2 = entry(
            "trip:TN-0001:split:v2",
            &[(PSP, -25_500), ("platform:revenue", 5_100), ("driver:DR-001", 20_400)],
        )?;
        post(&db.pool, &v2, adj, adj, None).await?;

        let end_nov = ts("2026-11-30T17:00:00Z")?;
        let now = ts("2026-12-03T00:00:00+07:00")?;
        assert_eq!(balance_snapshot_delta(&db.pool, "platform:revenue", end_nov).await?, 4_500);
        assert_eq!(balance_snapshot_delta(&db.pool, "platform:revenue", now).await?, 5_100);
        assert_eq!(balance_full(&db.pool, "platform:revenue", now).await?, 5_100);
        let reversal_of: Option<i64> = sqlx::query_scalar(
            "select reversal_of from journal_entries where idempotency_key = 'trip:TN-0001:split:reversal'",
        )
        .fetch_one(&db.pool)
        .await?;
        assert_eq!(reversal_of, Some(id));
        assert!(verify_snapshots(&db.pool, None).await?.is_empty());
        Ok(())
    })
    .await
}

#[tokio::test]
async fn daily_job_and_window_rebuild_produce_identical_snapshots() -> Result<()> {
    with_lab(|db| async move {
        db.migrate().await?;
        seed_months(&db.pool, 2026, 11, 1).await?;
        // Bút toán rải trên 5 ngày, có cái đúng ranh giới 00:00.
        let times = [
            "2026-11-01T00:00:00+07:00",
            "2026-11-01T23:59:59.5+07:00",
            "2026-11-02T00:00:00+07:00",
            "2026-11-03T12:00:00+07:00",
            "2026-11-05T23:59:59+07:00",
        ];
        for (i, t) in times.iter().enumerate() {
            let t = ts(t)?;
            let amount = 1_000 * i64::try_from(i + 1)?;
            let e = entry(
                &format!("k{i}"),
                &[("customer:CU-001", -amount), (PSP, amount)],
            )?;
            post(&db.pool, &e, t, t, None).await?;
        }
        for d in 1..=5 {
            snapshot_day(&db.pool, day(d)).await?;
        }
        let read =
            "select account_id, as_of::text, balance_dong from balance_snapshots order by 1, 2";
        let incremental: Vec<(String, String, i64)> =
            sqlx::query_as(read).fetch_all(&db.pool).await?;
        sqlx::query("delete from balance_snapshots")
            .execute(&db.pool)
            .await?;
        rebuild_snapshots(&db.pool, day(1), day(5)).await?;
        let rebuilt: Vec<(String, String, i64)> = sqlx::query_as(read).fetch_all(&db.pool).await?;
        assert_eq!(incremental.len(), 10);
        assert_eq!(incremental, rebuilt);
        // Mốc cuối ngày 01 là 00:00 ngày 02: bút toán đúng 00:00 ngày 02 chưa tính vào ngày 01.
        let end_day1 = day_bounds(day(1))?.1;
        assert_eq!(balance_full(&db.pool, PSP, end_day1).await?, 1_000 + 2_000);
        Ok(())
    })
    .await
}

#[tokio::test]
async fn race_unlocked_close_leaks_a_write_and_locked_close_does_not() -> Result<()> {
    let naive = with_lab(|db| async move { run_write_then_close(&db, true).await }).await?;
    assert!(!naive.closer_waited && naive.writer_committed);
    assert_eq!((naive.snapshot_psp, naive.full_psp), (25_500, 37_500));
    assert!(naive.mismatches > 0);

    let locked = with_lab(|db| async move { run_write_then_close(&db, false).await }).await?;
    assert!(locked.closer_waited && locked.writer_committed);
    assert_eq!((locked.snapshot_psp, locked.full_psp), (37_500, 37_500));
    assert_eq!(locked.mismatches, 0);
    Ok(())
}

#[tokio::test]
async fn race_close_first_rejects_the_waiting_writer() -> Result<()> {
    let out = with_lab(|db| async move { run_close_first(&db).await }).await?;
    assert!(out.writer_blocked);
    let (code, msg) = out.writer_error.expect("lần ghi đến sau phải bị từ chối");
    assert_eq!(code, "23001");
    assert!(msg.contains("đã đóng"), "{msg}");
    Ok(())
}
