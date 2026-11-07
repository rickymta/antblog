//! In số đo ra màn hình. Số đo trên máy viết bài, không phải SLA.

use std::process::Command;
use std::time::{Duration, Instant};

use optimize_measured::db::{BoxError, LabDb};
use optimize_measured::experiments::{alloc_experiments, wire_run};
use optimize_measured::hist::Samples;
use optimize_measured::loadgen::{RealRun, single_entry_latency, stall_comparison};
use optimize_measured::omission::{Sim, closed_loop, closed_loop_corrected, open_loop};
use optimize_measured::prepared;
use optimize_measured::strategies::{Strategy, trigger_stats, write};
use optimize_measured::workload::{make_entries, make_entry};

/// Số nguyên có dấu chấm ngăn nhóm nghìn: 10000 thành 10.000.
pub fn dots(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(c);
    }
    out
}

/// Dòng load average của máy (macOS/Linux), để ghi kèm số đo thời gian.
pub fn load_average() -> String {
    Command::new("uptime")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            s.split("load average")
                .nth(1)
                .map(|t| t.trim_start_matches(['s', ':', ' ']).trim().to_string())
        })
        .unwrap_or_else(|| "không đọc được".to_string())
}

pub fn demo() {
    let r = alloc_experiments(10_000);
    println!(
        "== Chuẩn hóa {} mã tài khoản (phải đổi byte: {}) ==",
        dots(r.raw as u64),
        dots(r.need_change as u64)
    );
    println!(
        "normalize_clone  : {:>7} cấp phát, {:>9} byte",
        dots(r.clone.allocs),
        dots(r.clone.bytes)
    );
    println!(
        "normalize (Cow)  : {:>7} cấp phát, {:>9} byte",
        dots(r.cow.allocs),
        dots(r.cow.bytes)
    );
    println!(
        "AccountId::new   : {:>7} cấp phát, {:>9} byte",
        dots(r.account_id.allocs),
        dots(r.account_id.bytes)
    );
    println!(
        "== Dựng cột cho {} bút toán ({} dòng) ==",
        dots(r.entries as u64),
        dots(3 * r.entries as u64)
    );
    println!(
        "mượn &str        : {:>7} cấp phát, {:>3} cấp phát lại, {:>9} byte",
        dots(r.columns_borrow.allocs),
        dots(r.columns_borrow.reallocs),
        dots(r.columns_borrow.bytes)
    );
    println!(
        "clone String     : {:>7} cấp phát, {:>3} cấp phát lại, {:>9} byte",
        dots(r.columns_clone.allocs),
        dots(r.columns_clone.reallocs),
        dots(r.columns_clone.bytes)
    );
    println!("== Coordinated omission, thời gian ảo (1.000 request/giây, kẹt 1 giây) ==");
    let sim = Sim::article();
    for (name, mut s) in [
        ("vòng đóng", closed_loop(sim)),
        ("vòng đóng + bù", closed_loop_corrected(sim)),
        ("vòng mở", open_loop(sim)),
    ] {
        println!("{name:<16} {}", s.summary());
    }
    println!(
        "Các chế độ cần PostgreSQL: --verify, --wire, --prepared, --pg-bench, --latency, --breakdown"
    );
}

pub async fn wire() -> Result<(), BoxError> {
    const N: usize = 1_000;
    const BATCH: usize = 500;
    let db = LabDb::create(false).await?;
    let result = async {
        let entries = make_entries(1, N as u64);
        println!("{N} bút toán (1 bút toán làm ấm), lô {BATCH}; đếm message qua proxy:");
        println!(
            "{:<38} {:>10} {:>5} {:>4} {:>17} {:>10} {:>10}",
            "cách ghi", "round trip", "Parse", "Q", "byte lên/bút toán", "Bind", "CopyData"
        );
        for strategy in [
            Strategy::PerRow,
            Strategy::Unnest(BATCH),
            Strategy::Copy(BATCH),
            Strategy::Cte(BATCH),
        ] {
            let run = wire_run(&db, strategy, &entries).await?;
            println!(
                "{:<38} {:>10} {:>5} {:>4} {:>17} {:>10} {:>10}",
                strategy.label(),
                dots(run.wire.round_trips),
                run.wire.parse,
                dots(run.wire.query),
                dots(run.wire.bytes_up / run.entries as u64),
                dots(run.wire.bind_bytes),
                dots(run.wire.copy_data_bytes)
            );
        }
        Ok::<(), BoxError>(())
    }
    .await;
    db.drop_schema().await;
    result
}

fn print_prepared(r: &prepared::Report) {
    println!(
        "{:<36} lần chạy {:>4}, Parse {:>3}, round trip {:>4}, pg_prepared_statements {:>3}",
        r.label, r.executions, r.wire.parse, r.wire.round_trips, r.prepared_on_server
    );
}

pub async fn prepared() -> Result<(), BoxError> {
    let db = LabDb::create(false).await?;
    let result = async {
        let proxy = db.proxy().await?;
        let mut conn = db.connect_via(&proxy).await?;
        println!(
            "shared_preload_libraries = '{}'",
            shared_preload(&mut conn).await?
        );
        // Mỗi thí nghiệm một kết nối mới, để pg_prepared_statements chỉ phản ánh thí nghiệm đó.
        print_prepared(&prepared::same_text(&mut conn, &proxy, 200, true).await?);
        let mut conn = db.connect_via(&proxy).await?;
        print_prepared(&prepared::same_text(&mut conn, &proxy, 200, false).await?);
        print_prepared(&prepared::fresh_connection(&db, &proxy, 200).await?);
        let mut conn = db.connect_via(&proxy).await?;
        print_prepared(&prepared::in_lists(&mut conn, &proxy, 50).await?);
        let mut conn = db.connect_via(&proxy).await?;
        print_prepared(&prepared::any_arrays(&mut conn, &proxy, 50).await?);
        print_prepared(&prepared::small_cache(&db, &proxy, 8, 20).await?);
        Ok::<(), BoxError>(())
    }
    .await;
    db.drop_schema().await;
    result
}

pub async fn shared_preload(conn: &mut sqlx::PgConnection) -> Result<String, sqlx::Error> {
    sqlx::query_scalar("show shared_preload_libraries")
        .persistent(false)
        .fetch_one(conn)
        .await
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// Mục tiêu đặt trước khi đo (giả định của bài, không phải SLA).
const TARGET_PER_SEC: f64 = 10_000.0;

pub async fn pg_bench(n: u64) -> Result<(), BoxError> {
    println!(
        "Số đo thời gian trên máy viết bài, không phải SLA. Load average: {}",
        load_average()
    );
    println!(
        "Mục tiêu đặt trước: nhập lô >= {} bút toán/giây, có idempotency và account_balances.",
        dots(TARGET_PER_SEC as u64)
    );
    let entries = make_entries(1, n);
    for with_trigger in [false, true] {
        let db = LabDb::create(with_trigger).await?;
        let result = async {
            let mut conn = db.connect().await?;
            let version: String = sqlx::query_scalar("select split_part(version(), ' on ', 1)")
                .fetch_one(&mut conn)
                .await?;
            if with_trigger {
                // Thống kê hàm của server, để đếm số lần trigger chạy và thời gian trong hàm.
                sqlx::raw_sql("set track_functions = 'pl'")
                    .execute(&mut conn)
                    .await?;
            }
            println!(
                "\nPostgreSQL: {version}; trigger cân bút toán: {}",
                if with_trigger { "có" } else { "không" }
            );
            for (strategy, count, runs) in [
                (Strategy::PerRow, (n / 10) as usize, 1),
                (Strategy::Bare(500), n as usize, 3),
                (Strategy::Unnest(500), n as usize, 3),
                (Strategy::Copy(500), n as usize, 3),
                (Strategy::Cte(500), n as usize, 3),
            ] {
                let mut rates = Vec::new();
                let mut in_trigger = (0, 0.0);
                for _ in 0..runs {
                    db.reset().await?;
                    let before = if with_trigger {
                        trigger_stats(&mut conn).await?
                    } else {
                        (0, 0.0)
                    };
                    let started = Instant::now();
                    write(&mut conn, strategy, &entries[..count]).await?;
                    let elapsed = started.elapsed();
                    rates.push(count as f64 / elapsed.as_secs_f64());
                    if with_trigger {
                        let after = trigger_stats(&mut conn).await?;
                        in_trigger = (after.0 - before.0, after.1 - before.1);
                    }
                }
                let rate = median(rates);
                let trigger_note = if with_trigger {
                    format!(
                        " | trigger {} lần, {:.0} ms trong hàm (lượt cuối)",
                        dots(in_trigger.0.unsigned_abs()),
                        in_trigger.1
                    )
                } else {
                    String::new()
                };
                println!(
                    "{:<38} {:>8} bút toán, {:>9.0} bút toán/giây  {}{trigger_note}",
                    strategy.label(),
                    dots(count as u64),
                    rate,
                    if rate >= TARGET_PER_SEC {
                        "đạt mục tiêu"
                    } else {
                        "chưa đạt"
                    }
                );
            }
            Ok::<(), BoxError>(())
        }
        .await;
        db.drop_schema().await;
        result?;
    }
    println!("\nLoad average khi xong: {}", load_average());
    Ok(())
}

/// Thống kê WAL của chính backend này (PostgreSQL 18): `(số lần fsync, tổng ms fsync)`.
async fn backend_wal(conn: &mut sqlx::PgConnection) -> Result<(i64, f64), sqlx::Error> {
    sqlx::query("select pg_stat_force_next_flush()")
        .execute(&mut *conn)
        .await?;
    sqlx::query_as(
        "select coalesce(sum(fsyncs), 0)::bigint, coalesce(sum(fsync_time), 0)::float8
         from pg_stat_get_backend_io(pg_backend_pid()) where object = 'wal'",
    )
    .fetch_one(conn)
    .await
}

async fn time_one(
    conn: &mut sqlx::PgConnection,
    strategy: Strategy,
    i: u64,
) -> Result<u64, sqlx::Error> {
    let entry = make_entry(i);
    let started = Instant::now();
    write(conn, strategy, std::slice::from_ref(&entry)).await?;
    Ok(u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX))
}

/// Vì sao 9 round trip không chậm gấp 9 lần 1 round trip: tách phần chờ mạng khỏi phần server làm.
/// Một kết nối, không tranh khóa. Mỗi vòng chạy xen kẽ đủ các trường hợp để cùng chịu một nhiễu.
pub async fn breakdown(n: u64) -> Result<(), BoxError> {
    let db = LabDb::create(false).await?;
    let result = async {
        println!("Load average: {}", load_average());
        let mut conn = db.connect().await?;
        sqlx::raw_sql("set track_wal_io_timing = on")
            .execute(&mut conn)
            .await?;
        let mut rtt = Samples::new();
        let mut rows = [Samples::new(), Samples::new()]; // chỉ số 0: on, 1: off
        let mut ones = [Samples::new(), Samples::new()];
        let mut fsync = (0i64, 0.0f64);
        let mut next = 1u64;
        for _ in 0..n {
            let started = Instant::now();
            sqlx::query("select 1").execute(&mut conn).await?;
            rtt.record(u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX));
            for (k, sql) in [
                "set synchronous_commit = on",
                "set synchronous_commit = off",
            ]
            .into_iter()
            .enumerate()
            {
                sqlx::raw_sql(sql).execute(&mut conn).await?;
                let before = backend_wal(&mut conn).await?;
                ones[k].record(time_one(&mut conn, Strategy::Cte(1), next).await?);
                rows[k].record(time_one(&mut conn, Strategy::PerRow, next + 1).await?);
                let after = backend_wal(&mut conn).await?;
                next += 2;
                if k == 0 {
                    fsync = (fsync.0 + after.0 - before.0, fsync.1 + after.1 - before.1);
                }
            }
        }
        println!(
            "select 1, một round trip gần như không việc: {}",
            rtt.summary()
        );
        for (k, mode) in ["on", "off"].into_iter().enumerate() {
            println!("synchronous_commit = {mode}:");
            println!("  từng dòng, 9 round trip: {}", rows[k].summary());
            println!("  một câu CTE, 1 round trip: {}", ones[k].summary());
        }
        #[allow(clippy::cast_precision_loss)]
        let per_commit = fsync.1 / fsync.0.max(1) as f64;
        println!(
            "fsync WAL của backend khi on: {} lần, trung bình {per_commit:.2} ms mỗi commit",
            fsync.0
        );
        println!("Load average khi xong: {}", load_average());
        Ok::<(), BoxError>(())
    }
    .await;
    db.drop_schema().await;
    result
}

fn show(name: &str, run: &mut RealRun) {
    println!(
        "{name:<34} gửi {:>4}, lỗi {}, {}",
        run.sent,
        run.errors,
        run.samples.summary()
    );
}

pub async fn latency(rate: u64) -> Result<(), BoxError> {
    let db = LabDb::create(false).await?;
    let result = async {
        println!("Load average: {}", load_average());
        let (mut closed, mut open) = stall_comparison(
            &db,
            100,
            Duration::from_secs(4),
            Duration::from_secs(1),
            Duration::from_millis(500),
            2,
        )
        .await?;
        println!("\nCùng một lần kẹt thật 500 ms (khóa bảng) lúc giây thứ 1:");
        show("vòng đóng, 2 luồng", &mut closed);
        show("vòng mở, 100 request/giây", &mut open);

        println!("\nĐộ trễ một bút toán, vòng mở {rate} request/giây, 5 giây:");
        let mut first = 1u64;
        for round in 1..=2 {
            for strategy in [Strategy::PerRow, Strategy::Cte(1)] {
                let mut run =
                    single_entry_latency(&db, strategy, rate, Duration::from_secs(5), first)
                        .await?;
                first += 1_000_000;
                show(&format!("lượt {round}: {}", strategy.label()), &mut run);
            }
        }
        println!("\nLoad average khi xong: {}", load_average());
        Ok::<(), BoxError>(())
    }
    .await;
    db.drop_schema().await;
    result
}
