//! Lab bài 48: số dư theo thời điểm, snapshot hằng ngày, đóng kỳ kế toán và cuộc đua đóng kỳ.
//!
//! `cargo run` in kết quả các thí nghiệm; `cargo run -- --verify` kiểm thêm các giá trị kỳ vọng
//! và thoát với mã 1 nếu sai. Mọi DDL nằm trong schema `lab_<uuid>` và bị xoá cuối ca.
//! Không assert thời gian: số đo in ra là số đo trên máy viết bài, không phải SLA.

use std::process::ExitCode;
use std::time::Instant;

use anyhow::{Result, ensure};
use jiff::civil::Date;
use ledger_48_snapshot_period_close::db::{LabDb, pg_error, with_lab};
use ledger_48_snapshot_period_close::period::{PeriodError, close_period, seed_months};
use ledger_48_snapshot_period_close::post::{entry, post};
use ledger_48_snapshot_period_close::race::{run_close_first, run_write_then_close};
use ledger_48_snapshot_period_close::seed::{HISTORY_DAYS, seed_history};
use ledger_48_snapshot_period_close::snapshot::{
    SNAPSHOT_DELTA_SQL, balance_full, balance_snapshot_delta, explain, explain_no_timing,
    rebuild_snapshots, snapshot_day, verify_snapshots,
};
use ledger_48_snapshot_period_close::tz::{day_bounds, ts};
use ledger_core::{EntryId, JournalEntry};
use sqlx::{AssertSqlSafe, PgPool};

const PSP: &str = "psp:vixanh:clearing";

/// Kiểm một điều kiện ở chế độ `--verify`; chế độ thường chỉ in.
fn expect(verify: bool, ok: bool, what: &str) -> Result<()> {
    if verify {
        ensure!(ok, "VERIFY FAIL: {what}");
    }
    Ok(())
}

fn day(y: i16, m: i8, d: i8) -> Date {
    Date::constant(y, m, d)
}

/// Tổng posting của `psp:vixanh:clearing` với điều kiện `where_clause` trên `e.posted_at` (hoặc cột khác).
async fn psp_sum(pool: &PgPool, where_clause: &str) -> Result<i64> {
    Ok(sqlx::query_scalar(AssertSqlSafe(format!(
        "select coalesce(sum(p.amount_dong), 0)::bigint from postings p \
           join journal_entries e on e.id = p.entry_id \
          where p.account_id = '{PSP}' and {where_clause}"
    )))
    .fetch_one(pool)
    .await?)
}

/// Phần 1 và 2, rồi đóng kỳ và điều chỉnh: một schema, bốn bút toán quanh nửa đêm ngày 05/11.
async fn story(db: LabDb, verify: bool) -> Result<()> {
    db.migrate().await?;
    seed_months(&db.pool, 2026, 10, 3).await?;
    let pool = &db.pool;

    println!("== 1. Số dư lúc 23:59 ngày 05/11/2026, giờ Việt Nam ==");
    let (from, to) = day_bounds(day(2026, 11, 5))?;
    let sql_to: String = sqlx::query_scalar(
        "select to_char(((date '2026-11-05' + 1)::timestamp at time zone 'Asia/Ho_Chi_Minh') \
                at time zone 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"')",
    )
    .fetch_one(pool)
    .await?;
    println!("jiff day_bounds(2026-11-05) = [{from}, {to})");
    println!("SQL AT TIME ZONE            = {sql_to}");
    expect(
        verify,
        sql_to == to.to_string(),
        "jiff và SQL cùng ranh giới ngày",
    )?;

    // Bốn bút toán, ghi theo thứ tự đến (recorded_at). posted_at là thời điểm xảy ra.
    // E4 xảy ra 23:59:59,5 ngày 05 nhưng tới sổ cái lúc 00:03 ngày 06.
    let e1 = entry(
        "trip:TN-0001:capture",
        &[("customer:CU-001", -25_500), (PSP, 25_500)],
    )?;
    let e3 = entry(
        "trip:TN-0002:capture",
        &[("customer:CU-002", -12_000), (PSP, 12_000)],
    )?;
    let e4 = entry(
        "trip:TN-0003:capture",
        &[("customer:CU-003", -12_000), (PSP, 12_000)],
    )?;
    // Chia sai: phí 4.500 thay vì 5.100. Sẽ được sửa sau khi đóng kỳ.
    let e2 = entry(
        "trip:TN-0001:split",
        &[
            (PSP, -25_500),
            ("platform:revenue", 4_500),
            ("driver:DR-001", 21_000),
        ],
    )?;
    let t1 = ts("2026-11-05T23:58:00+07:00")?;
    let t3 = ts("2026-11-06T00:00:00+07:00")?;
    let t4 = ts("2026-11-05T23:59:59.5+07:00")?;
    let r4 = ts("2026-11-06T00:03:00+07:00")?;
    let t2 = ts("2026-11-06T00:10:00+07:00")?;
    let id1 = post(pool, &e1, t1, t1, None).await?;
    post(pool, &e3, t3, t3, None).await?;
    post(pool, &e4, t4, r4, None).await?;
    let id2 = post(pool, &e2, t2, t2, None).await?;
    println!("đã ghi: JE-{id1:06} capture lúc {t1}, split JE-{id2:06} lúc {t2}");

    let cut = "'2026-11-06 00:00:00+07'";
    let correct = psp_sum(pool, &format!("e.posted_at < {cut}")).await?;
    let utc_date = psp_sum(pool, "e.posted_at::date <= date '2026-11-05'").await?;
    let between = psp_sum(
        pool,
        "e.posted_at between '2026-11-05 00:00:00+07' and '2026-11-05 23:59:59+07'",
    )
    .await?;
    let closed_end = psp_sum(pool, &format!("e.posted_at <= {cut}")).await?;
    println!("{PSP} cuối ngày 05/11:");
    println!("  posted_at < 06/11 00:00+07 (nửa mở, đúng) = {correct:>7}");
    println!("  posted_at::date <= '2026-11-05' (ngày UTC) = {utc_date:>7}");
    println!("  BETWEEN ... 23:59:59 (đóng, mất 23:59:59,5) = {between:>7}");
    println!("  posted_at <= 06/11 00:00+07 (đóng ở đầu phải) = {closed_end:>7}");
    expect(
        verify,
        correct == 37_500,
        "nửa mở theo giờ VN: 25.500 + 12.000",
    )?;
    expect(
        verify,
        utc_date == 24_000,
        "ngày UTC kéo cả split và bút toán 00:00 vào ngày 05",
    )?;
    expect(verify, between == 25_500, "BETWEEN mất bút toán 23:59:59,5")?;
    expect(
        verify,
        closed_end == 49_500,
        "đóng đầu phải kéo bút toán đúng 00:00 vào ngày 05",
    )?;

    println!("\n== 2. posted_at (xảy ra) và recorded_at (ghi) ==");
    let known = psp_sum(pool, &format!("e.recorded_at < {cut}")).await?;
    println!("  theo posted_at  < 06/11 00:00+07 = {correct:>7}  (sự thật về ngày 05)");
    println!("  theo recorded_at < 06/11 00:00+07 = {known:>7}  (điều hệ thống biết lúc 00:00)");
    expect(
        verify,
        known == 25_500,
        "recorded_at: bút toán trễ chưa có lúc nửa đêm",
    )?;

    println!("\n== 3. Đóng kỳ và điều chỉnh ở kỳ sau ==");
    let mut conn = pool.acquire().await?;
    let oct = close_period(&mut conn, "2026-10", ts("2026-11-01T00:05:00+07:00")?).await?;
    println!("đóng 2026-10: {oct} snapshot cuối kỳ");
    let out_of_order = close_period(&mut conn, "2026-12", ts("2026-12-31T23:00:00+07:00")?).await;
    let order_err = out_of_order.expect_err("đóng kỳ 12 khi kỳ 11 còn mở phải lỗi");
    println!("đóng 2026-12 trước 2026-11: {order_err}");
    expect(
        verify,
        matches!(order_err, PeriodError::EarlierOpen { .. }),
        "phải đóng theo thứ tự",
    )?;
    let nov = close_period(&mut conn, "2026-11", ts("2026-12-01T00:05:00+07:00")?).await?;
    println!("đóng 2026-11: {nov} snapshot cuối kỳ");
    drop(conn);

    let fix = entry(
        "trip:TN-0001:split:direct",
        &[
            (PSP, -25_500),
            ("platform:revenue", 5_100),
            ("driver:DR-001", 20_400),
        ],
    )?;
    let err = post(pool, &fix, t2, ts("2026-12-02T09:00:00+07:00")?, None)
        .await
        .expect_err("ghi vào kỳ đã đóng phải bị từ chối");
    let (code, msg) = pg_error(&err).unwrap_or_default();
    println!("ghi posted_at = {t2} vào kỳ đã đóng:\n  SQLSTATE {code}: {msg}");
    expect(
        verify,
        code == "23001" && msg.contains("2026-11 đã đóng"),
        "trigger cấm ghi vào kỳ đã đóng",
    )?;

    // Điều chỉnh ở kỳ sau: bút toán đảo có tham chiếu, rồi bút toán đúng. posted_at nằm ở kỳ 2026-12.
    let original: &JournalEntry = &e2;
    let reversal = JournalEntry::reversal_of(
        EntryId::parse(&format!("JE-{id2:06}")).expect("mã JE hợp lệ"),
        original,
    )?;
    let t_adj = ts("2026-12-02T09:00:00+07:00")?;
    let rev_id = post(pool, &reversal, t_adj, t_adj, Some(id2)).await?;
    let v2 = entry(
        "trip:TN-0001:split:v2",
        &[
            (PSP, -25_500),
            ("platform:revenue", 5_100),
            ("driver:DR-001", 20_400),
        ],
    )?;
    let v2_id = post(pool, &v2, t_adj, t_adj, None).await?;
    println!(
        "điều chỉnh ở kỳ 2026-12: JE-{rev_id:06} đảo JE-{id2:06}, JE-{v2_id:06} là bút toán đúng"
    );

    let end_nov = day_bounds(day(2026, 11, 30))?.1;
    let end_dec2 = day_bounds(day(2026, 12, 2))?.1;
    let rev_nov = balance_snapshot_delta(pool, "platform:revenue", end_nov).await?;
    let rev_dec = balance_snapshot_delta(pool, "platform:revenue", end_dec2).await?;
    println!("platform:revenue cuối kỳ 11 (đã báo cáo) = {rev_nov}; đến hết 02/12 = {rev_dec}");
    expect(
        verify,
        rev_nov == 4_500 && rev_dec == 5_100,
        "kỳ 11 giữ nguyên 4.500, số dư hiện tại 5.100",
    )?;
    let drv_dec = balance_snapshot_delta(pool, "driver:DR-001", end_dec2).await?;
    expect(
        verify,
        drv_dec == 20_400,
        "tài xế DR-001 nhận 20.400 sau điều chỉnh",
    )?;
    let mismatches = verify_snapshots(pool, None).await?;
    expect(
        verify,
        mismatches.is_empty(),
        "snapshot kỳ khớp tính lại từ posting",
    )?;
    Ok(())
}

/// Snapshot cộng phần chênh sai khi có posting ghi lùi vượt qua snapshot; trigger vô hiệu hoá sửa được.
async fn stale_demo(db: LabDb, with_trigger: bool, verify: bool) -> Result<(i64, i64, i64)> {
    db.migrate().await?;
    if !with_trigger {
        sqlx::query("alter table postings disable trigger postings_invalidate_snapshots")
            .execute(&db.pool)
            .await?;
    }
    seed_months(&db.pool, 2026, 11, 1).await?;
    let pool = &db.pool;
    let e1 = entry(
        "trip:TN-0001:capture",
        &[("customer:CU-001", -25_500), (PSP, 25_500)],
    )?;
    let t1 = ts("2026-11-05T23:58:00+07:00")?;
    post(pool, &e1, t1, t1, None).await?;
    snapshot_day(pool, day(2026, 11, 5)).await?; // job chạy lúc 00:01 ngày 06
    // Bút toán xảy ra 23:59:59,5 ngày 05 tới trễ lúc 00:03 ngày 06.
    let e4 = entry(
        "trip:TN-0003:capture",
        &[("customer:CU-003", -12_000), (PSP, 12_000)],
    )?;
    post(
        pool,
        &e4,
        ts("2026-11-05T23:59:59.5+07:00")?,
        ts("2026-11-06T00:03:00+07:00")?,
        None,
    )
    .await?;
    let cut = day_bounds(day(2026, 11, 5))?.1;
    let snap = balance_snapshot_delta(pool, PSP, cut).await?;
    let full = balance_full(pool, PSP, cut).await?;
    let left: i64 =
        sqlx::query_scalar("select count(*) from balance_snapshots where account_id = $1")
            .bind(PSP)
            .fetch_one(pool)
            .await?;
    let label = if with_trigger {
        "có trigger vô hiệu snapshot"
    } else {
        "không có trigger"
    };
    println!(
        "  {label}: snapshot + chênh = {snap}, tính lại = {full}, snapshot còn lại của {PSP} = {left}"
    );
    if with_trigger {
        expect(
            verify,
            snap == full && snap == 37_500 && left == 0,
            "trigger xoá snapshot bị vượt, số dư vẫn đúng",
        )?;
    } else {
        expect(
            verify,
            snap == 25_500 && full == 37_500,
            "không có trigger: snapshot cũ lệch 12.000",
        )?;
    }
    Ok((snap, full, left))
}

/// Xorshift để lấy mẫu ngẫu nhiên có thể lặp lại (không cần crate).
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

/// Trích «Buffers: shared hit=A read=B» đầu tiên (nút gốc, đã gồm các nút con) thành A + B.
fn buffers(plan: &[String]) -> i64 {
    plan.iter()
        .find(|l| l.trim_start().starts_with("Buffers:"))
        .map_or(0, |l| {
            l.split(|c: char| !c.is_ascii_digit())
                .filter_map(|n| n.parse::<i64>().ok())
                .sum()
        })
}

fn exec_ms(plan: &[String]) -> String {
    plan.iter()
        .find(|l| l.contains("Execution Time"))
        .cloned()
        .unwrap_or_default()
}

/// Dữ liệu lớn: so hai cách, EXPLAIN, lấy mẫu ngẫu nhiên, snapshot sai và dựng lại.
async fn bench(db: LabDb, verify: bool) -> Result<()> {
    db.migrate_base().await?;
    let pool = &db.pool;
    let trips: u32 = 32_000;
    let started = Instant::now();
    seed_history(pool, trips).await?;
    let (entries, postings): (i64, i64) = sqlx::query_as(
        "select (select count(*) from journal_entries), (select count(*) from postings)",
    )
    .fetch_one(pool)
    .await?;
    println!(
        "nạp {entries} bút toán, {postings} posting trong {:.1?} (số đo trên máy viết bài, không phải SLA)",
        started.elapsed()
    );
    expect(verify, postings >= 100_000, "ít nhất 100.000 posting")?;
    let hot: i64 = sqlx::query_scalar("select count(*) from postings where account_id = $1")
        .bind(PSP)
        .fetch_one(pool)
        .await?;
    println!(
        "{PSP} có {hot} posting; mỗi khách khoảng {} posting",
        trips / 300
    );

    let started = Instant::now();
    let rows = rebuild_snapshots(pool, day(2026, 7, 1), day(2026, 10, 31)).await?;
    sqlx::query("analyze balance_snapshots")
        .execute(pool)
        .await?;
    println!(
        "snapshot hằng ngày: {rows} dòng ({HISTORY_DAYS} ngày) dựng trong {:.1?}",
        started.elapsed()
    );
    expect(
        verify,
        rows == 400 * HISTORY_DAYS.unsigned_abs() + 2 * HISTORY_DAYS.unsigned_abs(),
        "400 + 2 tài khoản x 123 ngày",
    )?;

    // Hai cách cho cùng câu hỏi: số dư lúc 23:59 ngày 30/10 giờ Việt Nam.
    let cut = ts("2026-10-30T23:59:00+07:00")?;
    let full = balance_full(pool, PSP, cut).await?;
    let snap = balance_snapshot_delta(pool, PSP, cut).await?;
    println!("\n{PSP} lúc 23:59 ngày 30/10: tính lại toàn bộ = {full}, snapshot + chênh = {snap}");
    expect(verify, full == snap, "hai cách cho cùng số dư")?;

    let full_sql = format!(
        "select coalesce(sum(p.amount_dong), 0)::bigint from postings p join journal_entries e on e.id = p.entry_id \
          where p.account_id = '{PSP}' and e.posted_at < '{cut}'::timestamptz"
    );
    let snap_sql = SNAPSHOT_DELTA_SQL
        .replace("$1", &format!("'{PSP}'"))
        .replace("$2", &format!("'{cut}'"));
    // Chạy mỗi câu hai lần: lần đầu làm nóng cache, đọc kế hoạch lần hai.
    let (full_sql_for_timing, snap_sql_for_timing) = (full_sql.clone(), snap_sql.clone());
    explain(pool, full_sql.clone()).await?;
    let plan_full = explain(pool, full_sql).await?;
    explain(pool, snap_sql.clone()).await?;
    let plan_snap = explain(pool, snap_sql).await?;
    println!("\nEXPLAIN (ANALYZE, BUFFERS), tính lại toàn bộ:");
    for l in &plan_full {
        println!("  {l}");
    }
    println!("EXPLAIN (ANALYZE, BUFFERS), snapshot + chênh:");
    for l in &plan_snap {
        println!("  {l}");
    }
    let (bf, bs) = (buffers(&plan_full), buffers(&plan_snap));
    println!(
        "buffers (hit + read): toàn bộ = {bf}, snapshot + chênh = {bs}; {} | {}",
        exec_ms(&plan_full).trim(),
        exec_ms(&plan_snap).trim()
    );

    // Vì sao thời gian giảm mà buffer không giảm: Buffers đếm số lần chạm trang, không phải số trang.
    // Đếm số trang heap thật sự chứa phần chênh, và đo lại không có chi phí đo từng node.
    let (delta_rows, delta_pages): (i64, i64) = sqlx::query_as(
        "select count(*), count(distinct (p.ctid::text::point)[0]::bigint) \
           from postings p join journal_entries e on e.id = p.entry_id \
          where p.account_id = $1 and e.posted_at < $2::timestamptz \
            and e.posted_at >= (select max(as_of) from balance_snapshots \
                                 where account_id = $1 and as_of <= $2::timestamptz)",
    )
    .bind(PSP)
    .bind(cut.to_string())
    .fetch_one(pool)
    .await?;
    println!("phần chênh của {PSP}: {delta_rows} posting nằm trên {delta_pages} trang heap");
    expect(verify, delta_rows == 494, "phần chênh 494 posting")?;
    expect(verify, delta_pages < 100, "phần chênh nằm trên ít trang")?;
    let plan_full_quiet = explain_no_timing(pool, full_sql_for_timing).await?;
    let plan_snap_quiet = explain_no_timing(pool, snap_sql_for_timing).await?;
    println!(
        "TIMING OFF: toàn bộ {} | snapshot + chênh {}",
        exec_ms(&plan_full_quiet).trim(),
        exec_ms(&plan_snap_quiet).trim()
    );

    // Cùng câu hỏi với một khách bình thường (107 posting trong cả lịch sử).
    let cu = "customer:CU-0042";
    let cu_full = format!(
        "select coalesce(sum(p.amount_dong), 0)::bigint from postings p join journal_entries e on e.id = p.entry_id \
          where p.account_id = '{cu}' and e.posted_at < '{cut}'::timestamptz"
    );
    let cu_snap = SNAPSHOT_DELTA_SQL
        .replace("$1", &format!("'{cu}'"))
        .replace("$2", &format!("'{cut}'"));
    explain(pool, cu_full.clone()).await?;
    let cu_plan_full = explain(pool, cu_full).await?;
    explain(pool, cu_snap.clone()).await?;
    let cu_plan_snap = explain(pool, cu_snap).await?;
    println!(
        "\n{cu}: toàn bộ buffers = {}, {}",
        buffers(&cu_plan_full),
        exec_ms(&cu_plan_full).trim()
    );
    println!(
        "{cu}: snapshot + chênh buffers = {}, {}",
        buffers(&cu_plan_snap),
        exec_ms(&cu_plan_snap).trim()
    );
    for l in cu_plan_full
        .iter()
        .chain(&cu_plan_snap)
        .filter(|l| l.contains("Scan") || l.contains("Join") || l.contains("Nested"))
    {
        println!("  {l}");
    }

    // 400 mẫu ngẫu nhiên: tài khoản và mốc (một nửa số mẫu rơi đúng ranh giới ngày).
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let start = ts("2026-06-30T17:00:00Z")?.as_second();
    let span = HISTORY_DAYS * 86_400;
    let mut checked = 0;
    for i in 0..400 {
        let account = match i % 4 {
            0 => PSP.to_string(),
            1 => "platform:revenue".to_string(),
            2 => format!("customer:CU-{:04}", rng.next() % 300 + 1),
            _ => format!("driver:DR-{:03}", rng.next() % 100 + 1),
        };
        let secs = i64::try_from(rng.next() % u64::try_from(span)?)?;
        let at = if i % 2 == 0 {
            start + secs / 86_400 * 86_400
        } else {
            start + secs
        };
        let cut = jiff::Timestamp::from_second(at)?;
        let a = balance_full(pool, &account, cut).await?;
        let b = balance_snapshot_delta(pool, &account, cut).await?;
        expect(verify, a == b, "mẫu ngẫu nhiên: hai cách khớp")?;
        if a == b {
            checked += 1;
        }
    }
    println!("\n400 mẫu ngẫu nhiên (tài khoản x mốc, một nửa đúng ranh giới ngày): {checked} khớp");

    // Snapshot sai: lệch 1 đồng ở cuối ngày 29/10, phát hiện bằng so với tính lại, dựng lại từ posting.
    let bad_cut = day_bounds(day(2026, 10, 29))?.1;
    sqlx::query("update balance_snapshots set balance_dong = balance_dong + 1 where account_id = $1 and as_of = $2::timestamptz")
        .bind(PSP)
        .bind(bad_cut.to_string())
        .execute(pool)
        .await?;
    let probe = ts("2026-10-30T12:00:00+07:00")?;
    let wrong = balance_snapshot_delta(pool, PSP, probe).await?;
    let truth = balance_full(pool, PSP, probe).await?;
    let found = verify_snapshots(pool, Some(day_bounds(day(2026, 10, 28))?.0)).await?;
    println!(
        "\nsnapshot sai +1 đồng: truy vấn lúc 12:00 ngày 30/10 = {wrong}, đúng là {truth}; verify tìm thấy {} dòng lệch: {:?}",
        found.len(),
        found.first()
    );
    expect(
        verify,
        wrong == truth + 1 && found.len() == 1,
        "snapshot sai bị phát hiện",
    )?;
    let rebuilt = rebuild_snapshots(pool, day(2026, 10, 29), day(2026, 10, 31)).await?;
    let after = verify_snapshots(pool, Some(day_bounds(day(2026, 10, 25))?.0)).await?;
    let fixed = balance_snapshot_delta(pool, PSP, probe).await?;
    println!(
        "dựng lại 3 ngày từ posting ({rebuilt} dòng): còn {} dòng lệch, truy vấn = {fixed}",
        after.len()
    );
    expect(
        verify,
        after.is_empty() && fixed == truth,
        "dựng lại từ posting sửa được snapshot",
    )?;
    Ok(())
}

async fn run(verify: bool) -> Result<()> {
    with_lab(move |db| story(db, verify)).await?;

    println!("\n== 4. Snapshot cũ và posting ghi lùi (ngày 05/11, snapshot lúc 00:01) ==");
    with_lab(move |db| async move { stale_demo(db, false, verify).await.map(|_| ()) }).await?;
    with_lab(move |db| async move { stale_demo(db, true, verify).await.map(|_| ()) }).await?;

    println!("\n== 5. Ghi đang chạy trong lúc đóng kỳ 2026-11 ==");
    let naive = with_lab(|db| async move { run_write_then_close(&db, true).await }).await?;
    println!("trigger không khoá, đóng kỳ không khoá: {naive:?}");
    expect(
        verify,
        !naive.closer_waited && naive.writer_committed && naive.mismatches > 0,
        "bản sai: lần ghi lọt qua, snapshot lệch",
    )?;
    expect(
        verify,
        naive.snapshot_psp == 25_500 && naive.full_psp == 37_500,
        "bản sai: snapshot thiếu 12.000",
    )?;
    let locked = with_lab(|db| async move { run_write_then_close(&db, false).await }).await?;
    println!("FOR SHARE ở trigger, FOR UPDATE ở đóng kỳ: {locked:?}");
    expect(
        verify,
        locked.closer_waited && locked.writer_committed && locked.mismatches == 0,
        "bản đúng: đóng kỳ đợi, snapshot khớp",
    )?;
    expect(
        verify,
        locked.snapshot_psp == 37_500 && locked.full_psp == 37_500,
        "bản đúng: snapshot gồm bút toán vừa commit",
    )?;
    let first = with_lab(|db| async move { run_close_first(&db).await }).await?;
    println!("đóng kỳ giữ khoá trước, lần ghi đến sau: {first:?}");
    expect(
        verify,
        first.writer_blocked
            && first
                .writer_error
                .as_ref()
                .is_some_and(|(c, m)| c == "23001" && m.contains("đã đóng")),
        "lần ghi đến sau bị từ chối",
    )?;

    println!("\n== 6. Dữ liệu lớn: so hai cách, snapshot sai và dựng lại ==");
    with_lab(move |db| bench(db, verify)).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    let verify = std::env::args().any(|a| a == "--verify");
    match run(verify).await {
        Ok(()) => {
            if verify {
                println!("\nVERIFY OK");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e:#}");
            ExitCode::FAILURE
        }
    }
}
