//! `--verify`: kiểm mọi giá trị in trong bài. Không có assert nào về thời gian.

use std::borrow::Cow;
use std::time::Duration;

use optimize_measured::account::normalize;
use optimize_measured::db::{BoxError, LabDb, balances_match_postings, read_state};
use optimize_measured::experiments::{alloc_experiments, wire_run};
use optimize_measured::loadgen::{single_entry_latency, stall_comparison};
use optimize_measured::omission::{Sim, closed_loop, closed_loop_corrected, open_loop};
use optimize_measured::prepared;
use optimize_measured::strategies::{Strategy, copy_unbalanced_probe, trigger_stats, write};
use optimize_measured::workload::{make_entries, raw_account_ids};
use sqlx::Connection;

macro_rules! ensure {
    ($cond:expr, $($msg:tt)+) => {
        if !$cond {
            return Err(format!($($msg)+).into());
        }
    };
}

const N: u64 = 200;
const BATCH: usize = 100;

fn strategies() -> [Strategy; 4] {
    [
        Strategy::PerRow,
        Strategy::Unnest(BATCH),
        Strategy::Copy(BATCH),
        Strategy::Cte(BATCH),
    ]
}

pub async fn run() -> Result<(), BoxError> {
    step_alloc()?;
    step_omission()?;
    let db = LabDb::create(false).await?;
    let result = async {
        step_strategies(&db).await?;
        step_idempotency(&db).await?;
        step_cow_in_db(&db).await?;
        step_prepared(&db).await?;
        step_real_load(&db).await
    }
    .await;
    db.drop_schema().await;
    result?;
    step_trigger().await?;
    println!("VERIFY OK");
    Ok(())
}

fn step_alloc() -> Result<(), BoxError> {
    let r = alloc_experiments(10_000);
    ensure!(r.need_change == 500, "phải đổi byte: {}", r.need_change);
    ensure!(r.clone.allocs == 10_000, "clone: {}", r.clone.allocs);
    ensure!(r.cow.allocs == 500, "Cow: {}", r.cow.allocs);
    ensure!(
        r.account_id.allocs == 10_000,
        "AccountId: {}",
        r.account_id.allocs
    );
    ensure!(
        r.columns_borrow.allocs == 5,
        "cột mượn: {}",
        r.columns_borrow.allocs
    );
    ensure!(
        r.columns_clone.allocs == 70_005,
        "cột clone: {}",
        r.columns_clone.allocs
    );
    println!(
        "[1/7] cấp phát: normalize_clone {}, Cow {}, AccountId::new {}; cột mượn {}, cột clone {}",
        r.clone.allocs,
        r.cow.allocs,
        r.account_id.allocs,
        r.columns_borrow.allocs,
        r.columns_clone.allocs
    );
    Ok(())
}

fn step_omission() -> Result<(), BoxError> {
    let sim = Sim::article();
    let (mut closed, mut open, mut fixed) =
        (closed_loop(sim), open_loop(sim), closed_loop_corrected(sim));
    ensure!(
        closed.percentile(99.0) == 500,
        "vòng đóng p99 = {}",
        closed.percentile(99.0)
    );
    ensure!(closed.max() == 1_000_000, "vòng đóng max");
    ensure!(open.percentile(50.0) == 500, "vòng mở p50");
    ensure!(
        open.percentile(99.0) == 980_000,
        "vòng mở p99 = {}",
        open.percentile(99.0)
    );
    ensure!(
        fixed.percentile(99.0) == 951_000,
        "vòng đóng bù p99 = {}",
        fixed.percentile(99.0)
    );
    println!(
        "[2/7] coordinated omission (thời gian ảo): p99 vòng đóng {} us, bù {} us, vòng mở {} us",
        closed.percentile(99.0),
        fixed.percentile(99.0),
        open.percentile(99.0)
    );
    Ok(())
}

async fn step_strategies(db: &LabDb) -> Result<(), BoxError> {
    let entries = make_entries(1, N);
    let mut first_balances = None;
    for strategy in strategies() {
        let run = wire_run(db, strategy, &entries).await?;
        let expected = strategy.expected_round_trips(run.entries);
        ensure!(
            run.wire.round_trips == expected,
            "{}: {} round trip, cần {expected}",
            strategy.label(),
            run.wire.round_trips
        );
        // COPY có đúng một lần chờ CopyInResponse mỗi lô; ba cách còn lại không có.
        let expected_g = if matches!(strategy, Strategy::Copy(_)) {
            2
        } else {
            0
        };
        ensure!(
            run.wire.copy_in_response == expected_g,
            "{}: {} CopyInResponse, cần {expected_g}",
            strategy.label(),
            run.wire.copy_in_response
        );
        ensure!(
            run.wire.parse == 0,
            "{}: còn {} Parse sau khi làm ấm",
            strategy.label(),
            run.wire.parse
        );
        let s = &run.state;
        ensure!(
            s.entries == 200 && s.postings == 600,
            "{}: {} bút toán, {} posting",
            strategy.label(),
            s.entries,
            s.postings
        );
        ensure!(
            s.posting_sum == 0,
            "{}: tổng posting {}",
            strategy.label(),
            s.posting_sum
        );
        let mut conn = db.connect().await?;
        ensure!(
            balances_match_postings(&mut conn).await?,
            "{}: account_balances lệch posting",
            strategy.label()
        );
        conn.close().await?;
        // Số lần hàng số dư bị ghi: từng dòng ghi mỗi posting; lô gộp theo tài khoản mỗi lô.
        let expected_versions = if strategy == Strategy::PerRow { 199 } else { 2 };
        ensure!(
            s.balance_versions == expected_versions,
            "{}: tổng version {}, cần {expected_versions}",
            strategy.label(),
            s.balance_versions
        );
        match &first_balances {
            None => first_balances = Some(run.state.balances.clone()),
            Some(base) => ensure!(
                base == &run.state.balances,
                "{}: số dư khác cách từng dòng",
                strategy.label()
            ),
        }
        println!(
            "[3/7] {:<36} {:>5} round trip (cần {expected}), Parse 0, số dư khớp posting",
            strategy.label(),
            run.wire.round_trips
        );
    }
    Ok(())
}

async fn step_idempotency(db: &LabDb) -> Result<(), BoxError> {
    let entries = make_entries(1, 60);
    for strategy in strategies() {
        db.reset().await?;
        let mut conn = db.connect().await?;
        write(&mut conn, strategy, &entries).await?;
        let first = read_state(&mut conn).await?;
        // Ghi lại đúng các bút toán đó: không thêm gì, số dư và version giữ nguyên.
        write(&mut conn, strategy, &entries).await?;
        let replay = read_state(&mut conn).await?;
        ensure!(
            first == replay,
            "{}: ghi lại làm đổi trạng thái",
            strategy.label()
        );
        // Hai bút toán cùng key trong một lô: chỉ ghi một.
        db.reset().await?;
        let dup = [entries[0].clone(), entries[0].clone(), entries[1].clone()];
        write(&mut conn, strategy, &dup).await?;
        let s = read_state(&mut conn).await?;
        ensure!(
            s.entries == 2 && s.postings == 6 && s.posting_sum == 0,
            "{}: key trùng trong lô",
            strategy.label()
        );
        ensure!(
            balances_match_postings(&mut conn).await?,
            "{}: số dư lệch sau key trùng",
            strategy.label()
        );
        conn.close().await?;
    }
    println!("[4/7] idempotency: ghi lại và key trùng trong lô không thêm dòng, cả 4 cách");
    Ok(())
}

async fn step_cow_in_db(db: &LabDb) -> Result<(), BoxError> {
    let raw = raw_account_ids(10_000);
    let normalized: Vec<Cow<'_, str>> = raw.iter().filter_map(|r| normalize(r)).collect();
    ensure!(normalized.len() == raw.len(), "normalize bỏ mất mã");
    let raw_refs: Vec<&str> = raw.iter().map(String::as_str).collect();
    let norm_refs: Vec<&str> = normalized.iter().map(AsRef::as_ref).collect();
    let mut conn = db.connect().await?;
    let sql = "select count(*) from unnest($1::text[]) as t(id) join accounts a on a.id = t.id";
    let found_raw: i64 = sqlx::query_scalar(sql)
        .bind(&raw_refs[..])
        .fetch_one(&mut conn)
        .await?;
    let found_norm: i64 = sqlx::query_scalar(sql)
        .bind(&norm_refs[..])
        .fetch_one(&mut conn)
        .await?;
    conn.close().await?;
    ensure!(found_raw == 9_000, "mã thô khớp {found_raw}, cần 9.000");
    ensure!(
        found_norm == 10_000,
        "mã đã chuẩn hóa khớp {found_norm}, cần 10.000"
    );
    println!(
        "[5/7] Cow trong database: mã thô khớp tài khoản {}/10.000, sau normalize {}/10.000",
        crate::report::dots(found_raw.unsigned_abs()),
        crate::report::dots(found_norm.unsigned_abs())
    );
    Ok(())
}

async fn step_prepared(db: &LabDb) -> Result<(), BoxError> {
    let proxy = db.proxy().await?;
    let mut a = db.connect_via(&proxy).await?;
    let cached = prepared::same_text(&mut a, &proxy, 200, true).await?;
    ensure!(
        cached.wire.parse == 1 && cached.wire.round_trips == 201 && cached.prepared_on_server == 1,
        "cache bật: {cached:?}"
    );
    let mut b = db.connect_via(&proxy).await?;
    let unnamed = prepared::same_text(&mut b, &proxy, 200, false).await?;
    ensure!(
        unnamed.wire.parse == 200
            && unnamed.wire.round_trips == 400
            && unnamed.prepared_on_server == 0,
        "persistent(false): {unnamed:?}"
    );
    let fresh = prepared::fresh_connection(db, &proxy, 200).await?;
    ensure!(
        fresh.wire.parse == 1,
        "kết nối mới phải Parse lại: {fresh:?}"
    );
    let mut c = db.connect_via(&proxy).await?;
    let lists = prepared::in_lists(&mut c, &proxy, 50).await?;
    ensure!(
        lists.wire.parse == 50 && lists.wire.round_trips == 100 && lists.prepared_on_server == 50,
        "IN: {lists:?}"
    );
    let mut d = db.connect_via(&proxy).await?;
    let any = prepared::any_arrays(&mut d, &proxy, 50).await?;
    ensure!(
        any.wire.parse == 1 && any.wire.round_trips == 51 && any.prepared_on_server == 1,
        "ANY: {any:?}"
    );
    let small = prepared::small_cache(db, &proxy, 8, 20).await?;
    ensure!(
        small.wire.parse == 20 && small.prepared_on_server == 8,
        "cache nhỏ: {small:?}"
    );
    for conn in [a, b, c, d] {
        conn.close().await?;
    }
    let mut conn = db.connect().await?;
    let preload: String = sqlx::query_scalar("show shared_preload_libraries")
        .persistent(false)
        .fetch_one(&mut conn)
        .await?;
    conn.close().await?;
    println!(
        "[6/7] prepared: Parse cache bật {} / persistent(false) {} / IN động {} / ANY mảng {}; cache 8 chỗ giữ {} câu; pg_stat_statements nạp: {}",
        cached.wire.parse,
        unnamed.wire.parse,
        lists.wire.parse,
        any.wire.parse,
        small.prepared_on_server,
        if preload.contains("pg_stat_statements") {
            "có"
        } else {
            "không (đếm bằng proxy)"
        }
    );
    Ok(())
}

async fn step_real_load(db: &LabDb) -> Result<(), BoxError> {
    // Chỉ kiểm số lượng (đồng hồ thật nên không assert thời gian).
    let (closed, open) = stall_comparison(
        db,
        50,
        Duration::from_secs(1),
        Duration::from_millis(200),
        Duration::from_millis(300),
        2,
    )
    .await?;
    ensure!(
        open.sent == 50 && open.samples.len() == 50,
        "vòng mở phải ghi đủ 50 mẫu, có {}",
        open.samples.len()
    );
    ensure!(
        closed.sent >= 1 && closed.sent as usize == closed.samples.len(),
        "vòng đóng: số mẫu"
    );
    ensure!(open.errors == 0 && closed.errors == 0, "có request lỗi");
    let run = single_entry_latency(db, Strategy::Cte(1), 20, Duration::from_secs(1), 1).await?;
    ensure!(
        run.sent == 20 && run.samples.len() == 20 && run.errors == 0,
        "đường ghi một bút toán"
    );
    println!(
        "[7/7] tải thật: vòng mở ghi đủ {} mẫu theo lịch, vòng đóng {} mẫu, 0 lỗi (không assert thời gian)",
        open.samples.len(),
        closed.samples.len()
    );
    Ok(())
}

/// Constraint trigger cân bút toán của bài 25 vẫn chạy theo từng dòng khi ghi lô và khi `COPY`.
async fn step_trigger() -> Result<(), BoxError> {
    let entries = make_entries(1, N);
    let mut calls = Vec::new();
    for strategy in [Strategy::Unnest(BATCH), Strategy::Copy(BATCH)] {
        let db = LabDb::create(true).await?;
        let result = async {
            let mut conn = db.connect().await?;
            sqlx::raw_sql("set track_functions = 'pl'")
                .execute(&mut conn)
                .await?;
            write(&mut conn, strategy, &entries).await?;
            let (n, _ms) = trigger_stats(&mut conn).await?;
            ensure!(
                n == 4 * N as i64,
                "{}: trigger chạy {n} lần, cần {}",
                strategy.label(),
                4 * N
            );
            let state = read_state(&mut conn).await?;
            ensure!(
                state.entries == 200 && state.posting_sum == 0,
                "{}: trạng thái",
                strategy.label()
            );
            let code = copy_unbalanced_probe(&mut conn).await?;
            ensure!(
                code.as_deref() == Some("23514"),
                "{}: COPY lệch phải bị chặn, SQLSTATE {code:?}",
                strategy.label()
            );
            conn.close().await?;
            Ok::<i64, BoxError>(n)
        }
        .await;
        db.drop_schema().await;
        calls.push(result?);
    }
    println!(
        "[trigger] 200 bút toán (600 posting) chạy trigger {} và {} lần = 4 mỗi bút toán; COPY bút toán lệch bị chặn (SQLSTATE 23514)",
        calls[0], calls[1]
    );
    Ok(())
}
