//! Lab bài 25: schema sổ cái, kiểm tổng bằng 0 ở DB, projection số dư, index và partition.
//!
//! `cargo run` in kết quả các thí nghiệm; `cargo run -- --verify` kiểm thêm các giá trị
//! kỳ vọng và thoát với mã 1 nếu sai. Mọi DDL chạy trong schema `lab_<uuid>` và bị xoá cuối ca.

use std::collections::BTreeSet;
use std::process::ExitCode;

use anyhow::{Result, ensure};
use ledger_25_schema_so_cai::bench::{self, ACCOUNT_SUM_SQL, Plan};
use ledger_25_schema_so_cai::db::{LabDb, pg_error, with_lab};
use ledger_25_schema_so_cai::ledger::{self, display_id};
use ledger_25_schema_so_cai::partition::{self, ACCOUNT_MONTH_SQL, ALL_MONTHS_SQL, ONE_MONTH_SQL};
use ledger_core::{
    AccountId, JournalEntryBuilder, Money, TripCharge, capture_entry, default_policy, fare_for_km,
};
use sqlx::PgPool;
use tokio::task::JoinSet;

/// Số bút toán sinh cho thí nghiệm index: 60.000 bút toán x 2 dòng = 120.000 posting.
const BENCH_ENTRIES: i64 = 60_000;

/// Kiểm một điều kiện ở chế độ `--verify`; chế độ thường chỉ in.
fn expect(verify: bool, ok: bool, what: &str) -> Result<()> {
    if verify {
        ensure!(ok, "VERIFY FAIL: {what}");
    }
    Ok(())
}

fn acct(raw: &str) -> Result<AccountId> {
    Ok(AccountId::new(raw)?)
}

async fn count(pool: &PgPool, sql: &'static str) -> Result<i64> {
    Ok(sqlx::query_scalar(sql).fetch_one(pool).await?)
}

/// In (SQLSTATE, thông điệp) của lỗi PostgreSQL và trả lại cặp đó.
fn show_rejection(label: &str, err: &sqlx::Error) -> (String, String) {
    let pair = pg_error(err).unwrap_or_default();
    println!("{label}\n  SQLSTATE {}: {}", pair.0, pair.1);
    pair
}

/// Ghi một posting vào bút toán tạm trong transaction rồi bỏ (rollback khi drop).
/// Trả lỗi mà CHECK hoặc FK ném ngay ở câu INSERT, trước cả COMMIT.
async fn rejected_posting(pool: &PgPool, account: &str, amount: i64) -> Result<sqlx::Error> {
    let mut tx = pool.begin().await?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (idempotency_key) VALUES ('demo:tmp') RETURNING id",
    )
    .fetch_one(&mut *tx)
    .await?;
    let err = sqlx::query(
        "INSERT INTO postings (entry_id, line_no, account_id, amount_dong) VALUES ($1, 1, $2, $3)",
    )
    .bind(id)
    .bind(account)
    .bind(amount)
    .execute(&mut *tx)
    .await
    .expect_err("posting này phải bị từ chối");
    Ok(err)
}

fn print_balances(rows: &[(String, i64, i64)]) {
    for (account, balance, version) in rows {
        println!("  {account:<22} {balance:>8}  (version {version})");
    }
}

async fn balance_rows(pool: &PgPool) -> Result<Vec<(String, i64, i64)>> {
    Ok(sqlx::query_as(
        "SELECT account_id, balance_dong, version FROM account_balances ORDER BY account_id",
    )
    .fetch_all(pool)
    .await?)
}

/// Phần 1: bốn bảng, CHECK, kiểm cân bằng constraint trigger, projection số dư.
async fn ledger_part(db: LabDb, verify: bool) -> Result<()> {
    let pool = &db.pool;
    db.migrate_base().await?;
    ledger::seed_accounts(pool).await?;

    println!("== 1. Bốn bảng của schema chuẩn ==");
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT table_name::text FROM information_schema.tables \
         WHERE table_schema = current_schema() ORDER BY 1",
    )
    .fetch_all(pool)
    .await?;
    println!("{}", tables.join(", "));
    expect(
        verify,
        tables
            == [
                "account_balances",
                "accounts",
                "journal_entries",
                "postings",
            ],
        "đủ 4 bảng",
    )?;

    println!("\n== 2. Ghi chuyến TN-0001: bút toán, posting và số dư cùng một transaction ==");
    let trip = TripCharge {
        trip_id: "TN-0001".to_string(),
        customer: acct("customer:CU-001")?,
        driver: acct("driver:DR-001")?,
        fare: fare_for_km(5),
    };
    let entry = capture_entry(&trip, &default_policy())?;
    let id = ledger::post_entry(pool, &entry).await?;
    println!("{} {}", display_id(id), entry.idempotency_key());
    let rows = balance_rows(pool).await?;
    print_balances(&rows);
    let sum: i64 = rows.iter().map(|r| r.1).sum();
    println!("  tổng số dư: {sum}");
    expect(
        verify,
        display_id(id) == "JE-000001",
        "bút toán đầu là JE-000001",
    )?;
    expect(
        verify,
        rows == [
            ("customer:CU-001".to_string(), -25_500, 1),
            ("driver:DR-001".to_string(), 20_400, 1),
            ("platform:revenue".to_string(), 5_100, 1),
        ],
        "số dư TN-0001: -25.500 / 20.400 / 5.100",
    )?;
    expect(verify, sum == 0, "tổng số dư bằng 0")?;
    expect(
        verify,
        ledger::reconcile(pool).await?.is_empty(),
        "projection khớp posting",
    )?;

    println!("\n== 3. CHECK và FK từ chối ngay ở câu INSERT ==");
    let err = sqlx::query("INSERT INTO accounts (id, kind) VALUES ('x:1', 'equity')")
        .execute(pool)
        .await
        .expect_err("kind lạ phải bị từ chối");
    let (code_kind, _) = show_rejection("accounts.kind = 'equity'", &err);
    let err = rejected_posting(pool, "customer:CU-001", 0).await?;
    let (code_zero, _) = show_rejection("postings.amount_dong = 0", &err);
    let err = rejected_posting(pool, "driver:DR-999", 500).await?;
    let (code_fk, _) = show_rejection(
        "postings.account_id = 'driver:DR-999' (chưa có tài khoản)",
        &err,
    );
    expect(verify, code_kind == "23514", "kind lạ: 23514")?;
    expect(verify, code_zero == "23514", "amount 0: 23514")?;
    expect(verify, code_fk == "23503", "tài khoản lạ: 23503")?;

    println!("\n== 4. Bút toán lệch: từng câu INSERT chạy được, DB từ chối lúc COMMIT ==");
    let err = ledger::post_unbalanced(pool)
        .await
        .expect_err("bút toán lệch phải bị từ chối");
    let (code_sum, msg_sum) = show_rejection("posting -100 và +90, rồi COMMIT", &err);
    expect(
        verify,
        code_sum == "23514" && msg_sum.contains("lệch: tổng posting = -10"),
        "lệch -10 bị từ chối lúc commit",
    )?;
    let err = sqlx::query("INSERT INTO journal_entries (idempotency_key) VALUES ('demo:empty')")
        .execute(pool)
        .await
        .expect_err("bút toán không có posting phải bị từ chối");
    let (code_empty, msg_empty) = show_rejection("bút toán không có posting", &err);
    expect(
        verify,
        code_empty == "23514" && msg_empty.contains("có 0 posting"),
        "bút toán rỗng bị từ chối",
    )?;
    let err = ledger::post_one_line(pool)
        .await
        .expect_err("bút toán một dòng phải bị từ chối");
    let (code_one, msg_one) = show_rejection("bút toán một dòng", &err);
    expect(
        verify,
        code_one == "23514" && msg_one.contains("có 1 posting"),
        "bút toán một dòng bị từ chối",
    )?;
    let entries = count(pool, "SELECT count(*) FROM journal_entries").await?;
    let postings = count(pool, "SELECT count(*) FROM postings").await?;
    println!("  sau ba lần thử: {entries} bút toán, {postings} posting");
    expect(
        verify,
        entries == 1 && postings == 3,
        "ba lần thử không để lại dòng nào",
    )?;

    println!("\n== 5. Transaction hỏng không để lại số dư lệch ==");
    let rows = balance_rows(pool).await?;
    print_balances(&rows);
    expect(
        verify,
        rows.iter().map(|r| r.1).collect::<Vec<_>>() == [-25_500, 20_400, 5_100],
        "số dư không đổi sau lần ghi hỏng",
    )?;
    expect(
        verify,
        ledger::reconcile(pool).await?.is_empty(),
        "vẫn khớp posting",
    )?;

    println!("\n== 6. Đối chiếu projection với SUM(postings) ==");
    sqlx::query("UPDATE account_balances SET balance_dong = balance_dong - 500 WHERE account_id = 'driver:DR-001'")
        .execute(pool)
        .await?;
    let drift = ledger::reconcile(pool).await?;
    for d in &drift {
        println!(
            "  LỆCH {}: projection {}, SUM(postings) {}",
            d.account_id, d.projected, d.actual
        );
    }
    expect(
        verify,
        drift.len() == 1
            && drift[0].account_id == "driver:DR-001"
            && drift[0].projected == 19_900
            && drift[0].actual == 20_400,
        "phát hiện đúng lệch của driver:DR-001",
    )?;
    let rebuilt = ledger::rebuild_balances(pool).await?;
    let after = ledger::reconcile(pool).await?;
    println!("  dựng lại {rebuilt} dòng số dư, còn lệch: {}", after.len());
    expect(
        verify,
        rebuilt == 3 && after.is_empty(),
        "dựng lại xong thì hết lệch",
    )?;

    println!("\n== 7. 40 bút toán đồng thời trên cùng ba tài khoản ==");
    let mut tasks = JoinSet::new();
    for i in 1..=40_i64 {
        let pool = pool.clone();
        tasks.spawn(async move {
            let k = i % 5 + 1;
            let b = JournalEntryBuilder::new(&format!("demo:concurrent:{i:02}"));
            // Đổi thứ tự dòng giữa các bút toán; mã ghi vẫn khoá theo thứ tự tên tài khoản.
            let b = if i % 2 == 0 {
                b.posting(acct("customer:CU-001")?, Money::from_dong(-1_000 * k))
                    .posting(acct("driver:DR-001")?, Money::from_dong(800 * k))
                    .posting(acct("platform:revenue")?, Money::from_dong(200 * k))
            } else {
                b.posting(acct("platform:revenue")?, Money::from_dong(200 * k))
                    .posting(acct("driver:DR-001")?, Money::from_dong(800 * k))
                    .posting(acct("customer:CU-001")?, Money::from_dong(-1_000 * k))
            };
            let entry = b.build()?;
            Ok::<_, anyhow::Error>(ledger::post_entry(&pool, &entry).await?)
        });
    }
    while let Some(done) = tasks.join_next().await {
        done??;
    }
    let rows = balance_rows(pool).await?;
    print_balances(&rows);
    let drift = ledger::reconcile(pool).await?;
    let sum: i64 = rows.iter().map(|r| r.1).sum();
    println!("  tổng số dư: {sum}; tài khoản lệch: {}", drift.len());
    let max_id: i64 = count(pool, "SELECT max(id) FROM journal_entries").await?;
    let n_entries = count(pool, "SELECT count(*) FROM journal_entries").await?;
    println!("  {n_entries} bút toán, id lớn nhất {max_id} (identity có lỗ sau các lần ghi hỏng)");
    expect(
        verify,
        drift.is_empty() && sum == 0,
        "đồng thời vẫn khớp và tổng bằng 0",
    )?;
    expect(
        verify,
        rows.iter().all(|r| r.2 == 42),
        "version = 1 (TN-0001) + 1 (dựng lại) + 40",
    )?;
    expect(
        verify,
        n_entries == 41 && max_id > 41,
        "41 bút toán, id có lỗ",
    )?;
    Ok(())
}

fn print_plan(title: &str, plan: &Plan) {
    println!("{title}");
    for line in &plan.lines {
        println!("  {line}");
    }
}

/// Tên partition xuất hiện trong kế hoạch, theo thứ tự tên.
fn scanned_partitions(plan: &Plan) -> Vec<String> {
    let mut names = BTreeSet::new();
    for line in &plan.lines {
        for word in line.split(|c: char| !(c.is_alphanumeric() || c == '_')) {
            // Tên partition dài 18 ký tự (`postings_p_2026_03`); tên index mở rộng thêm đuôi.
            if word.starts_with("postings_p_2026_") && word.len() >= 18 {
                names.insert(word[..18].to_string());
            }
        }
    }
    names.into_iter().collect()
}

/// Phần 2: 120.000 posting, index, rồi bản partition.
async fn bench_part(db: LabDb, verify: bool) -> Result<()> {
    let pool = &db.pool;
    db.migrate_base().await?;

    println!("== 8. Sinh {BENCH_ENTRIES} bút toán x 2 dòng bằng generate_series ==");
    let seeded = bench::seed_bulk(pool, BENCH_ENTRIES, true).await?;
    let postings = count(pool, "SELECT count(*) FROM postings").await?;
    println!(
        "  {postings} posting, nạp + kiểm cân lúc commit: {:.2?}",
        seeded
    );
    expect(verify, postings == 2 * BENCH_ENTRIES, "đủ 120.000 posting")?;

    println!("\n== 9. Cộng posting của một tài khoản: không index, rồi có index ==");
    let before = bench::explain(pool, ACCOUNT_SUM_SQL).await?;
    print_plan("EXPLAIN (ANALYZE, BUFFERS), chưa có index:", &before);
    let built = bench::create_index(pool).await?;
    println!("  dựng index postings_account_entry_idx: {built:.2?}");
    let after = bench::explain(pool, ACCOUNT_SUM_SQL).await?;
    print_plan("EXPLAIN (ANALYZE, BUFFERS), có index:", &after);
    println!(
        "  tóm tắt: không index {:.3} ms, {} page; có index {:.3} ms, {} page",
        before.execution_ms,
        before.pages(),
        after.execution_ms,
        after.pages()
    );
    println!(
        "  kích thước postings kèm index: {} KB",
        bench::postings_size_kb(pool).await?
    );
    expect(
        verify,
        before.mentions("Seq Scan on postings"),
        "không index: Seq Scan",
    )?;
    expect(
        verify,
        after.mentions("postings_account_entry_idx"),
        "có index: dùng postings_account_entry_idx",
    )?;
    expect(
        verify,
        !after.mentions("Seq Scan"),
        "có index: không còn Seq Scan",
    )?;
    expect(
        verify,
        after.pages() * 5 < before.pages(),
        "index giảm ít nhất 5 lần số page đọc",
    )?;
    expect(
        verify,
        after.execution_ms < before.execution_ms,
        "index nhanh hơn",
    )?;

    println!("\n== 10. Partition theo tháng: hệ quả với khóa và ràng buộc ==");
    let pk = partition::pk_without_partition_key(pool).await?;
    println!("PRIMARY KEY (entry_id, line_no) trên bảng PARTITION BY RANGE (posted_at):");
    println!("  {pk:?}");
    expect(
        verify,
        pk.as_ref()
            .is_some_and(|(c, m)| c == "0A000" && m.contains("partitioning column")),
        "PK thiếu partition key bị từ chối",
    )?;

    let copied = partition::create_and_fill(pool).await?;
    println!("đã sao chép {copied} posting sang postings_p (PK gồm posted_at)");
    for (part, n) in partition::rows_per_partition(pool).await? {
        println!("  {part}: {n}");
    }
    expect(
        verify,
        copied.cast_signed() == 2 * BENCH_ENTRIES,
        "sao chép đủ posting",
    )?;

    let out = partition::insert_out_of_range(pool).await?;
    println!("posted_at = 2026-11-06 (ngoài mọi partition):\n  {out:?}");
    expect(
        verify,
        out.as_ref().is_some_and(|(c, _)| c == "23514"),
        "ngoài khoảng partition bị từ chối",
    )?;

    let probe = partition::probe_uniqueness(pool).await?;
    println!(
        "trùng (entry_id, line_no, posted_at) bị từ chối: {}",
        probe.same_month_rejected
    );
    println!(
        "trùng (entry_id, line_no) nhưng khác tháng được nhận: {}",
        probe.other_month_accepted
    );
    expect(
        verify,
        probe.same_month_rejected && probe.other_month_accepted,
        "unique chỉ còn trong phạm vi partition key",
    )?;

    let fk2 = partition::fk_to_partitioned_two_columns(pool).await?;
    println!("FK (entry_id, line_no) -> postings_p:\n  {fk2:?}");
    let fk3 = partition::fk_to_partitioned_with_key(pool).await?;
    println!(
        "FK (entry_id, line_no, posted_at) -> postings_p: {}",
        if fk3.is_none() {
            "được chấp nhận"
        } else {
            "bị từ chối"
        }
    );
    expect(
        verify,
        fk2.is_some() && fk3.is_none(),
        "FK vào bảng partition phải gồm partition key",
    )?;

    let key = partition::unique_key_without_partition_key(pool).await?;
    println!("UNIQUE (idempotency_key) trên bảng PARTITION BY RANGE (posted_at):\n  {key:?}");
    let two_months = partition::same_key_two_months(pool).await?;
    println!(
        "UNIQUE (idempotency_key, posted_at), cùng key khác posted_at: {}",
        if two_months.is_none() {
            "cả hai dòng được nhận"
        } else {
            "bị từ chối"
        }
    );
    expect(
        verify,
        key.as_ref().is_some_and(|(c, _)| c == "0A000") && two_months.is_none(),
        "idempotency_key không còn unique toàn cục khi partition theo posted_at",
    )?;

    println!("\n== 11. Partition pruning trong EXPLAIN ==");
    let one = bench::explain(pool, ONE_MONTH_SQL).await?;
    print_plan("Một tháng (posted_at trong 2026-03):", &one);
    let all = bench::explain(pool, ALL_MONTHS_SQL).await?;
    let acct_month = bench::explain(pool, ACCOUNT_MONTH_SQL).await?;
    print_plan("Một tài khoản trong một tháng:", &acct_month);
    println!(
        "  partition bị quét, một tháng: {:?}",
        scanned_partitions(&one)
    );
    println!(
        "  partition bị quét, không lọc posted_at: {}",
        scanned_partitions(&all).len()
    );
    println!(
        "  số page: một tháng {}, toàn bộ {}; một tài khoản một tháng {}",
        one.pages(),
        all.pages(),
        acct_month.pages()
    );
    expect(
        verify,
        scanned_partitions(&one) == ["postings_p_2026_03"],
        "chỉ quét partition tháng 3",
    )?;
    expect(
        verify,
        scanned_partitions(&all).len() == 6,
        "không lọc thì quét đủ 6 partition",
    )?;
    expect(
        verify,
        scanned_partitions(&acct_month) == ["postings_p_2026_03"],
        "tài khoản + tháng: chỉ partition tháng 3",
    )?;
    expect(verify, one.pages() < all.pages(), "pruning giảm số page")?;
    Ok(())
}

/// Phần 3: giá của constraint trigger khi nạp hàng loạt (tắt trigger rồi nạp cùng dữ liệu).
async fn trigger_cost_part(db: LabDb, verify: bool) -> Result<()> {
    db.migrate_base().await?;
    println!("== 12. Giá của kiểm cân lúc commit khi nạp hàng loạt ==");
    let without = bench::seed_bulk(&db.pool, BENCH_ENTRIES, false).await?;
    println!(
        "  nạp {} posting, tắt hai trigger: {without:.2?}",
        2 * BENCH_ENTRIES
    );
    expect(
        verify,
        count(&db.pool, "SELECT count(*) FROM postings").await? == 2 * BENCH_ENTRIES,
        "nạp đủ khi tắt trigger",
    )?;
    Ok(())
}

async fn run(verify: bool) -> Result<()> {
    with_lab(move |db| ledger_part(db, verify)).await?;
    println!();
    with_lab(move |db| bench_part(db, verify)).await?;
    println!();
    with_lab(move |db| trigger_cost_part(db, verify)).await?;
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
