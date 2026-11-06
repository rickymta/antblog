//! Lab bài 30: chạy workload, kiểm kết quả nghiệp vụ (`--verify`), đo nhanh (`--timing`)
//! và đo ghi PostgreSQL (`--pg-bench`). Số đo chính xác hơn lấy từ `cargo bench`.

use std::process::ExitCode;
use std::time::Duration;

use ledger_bench::pg::{self, BoxError, LabDb, Strategy};
use ledger_bench::workload::{flatten_grow, flatten_with_capacity};
use ledger_bench::{IndexedLedger, make_entry, make_ledger, timing};
use ledger_core::{AccountId, JournalEntry, Money};

fn acct(raw: &str) -> AccountId {
    AccountId::new(raw).expect("mã tài khoản hợp lệ")
}

fn demo() {
    let book = {
        let mut book = IndexedLedger::new();
        for i in 1..=10_000 {
            book.post(make_entry(i)).expect("key không trùng");
        }
        book
    };
    let driver = acct("driver:DR-001");
    let scan = book.ledger().balance(&driver).expect("không tràn");
    println!("Sổ có {} bút toán", book.ledger().len());
    println!("Quét toàn sổ : {} = {scan}", driver.as_str());
    println!(
        "Cộng dồn     : {} = {}",
        driver.as_str(),
        book.balance(&driver)
    );
}

fn timing_report() {
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let t = timing::measure_default();
    println!("Build: {profile}");
    println!(
        "Tạo + ghi {} bút toán: {:.1} ms",
        t.n,
        t.create_and_post.as_secs_f64() * 1e3
    );
    println!(
        "Một lần balance() quét toàn sổ: {:.1} us",
        t.balance_scan_each.as_secs_f64() * 1e6
    );
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

async fn pg_bench() -> Result<(), BoxError> {
    const N: u64 = 10_000;
    const RUNS: usize = 3;
    let entries: Vec<JournalEntry> = (1..=N).map(make_entry).collect();
    let db = LabDb::create(16).await?;
    let result = async {
        println!("PostgreSQL: {}", pg::server_info(&db).await?);
        db.seed_accounts(&entries).await?;
        let rtt = pg::round_trip(&db.pool, 2_000).await?;
        println!(
            "Một round trip `select 1` (Docker Desktop, loopback): {:.0} us",
            rtt.as_secs_f64() * 1e6
        );
        println!("Ghi {N} bút toán, {RUNS} lượt mỗi cách (bút toán/giây):");
        let strategies = [
            Strategy::PerEntryTx,
            Strategy::Batched(500),
            Strategy::BatchedUnnest(500),
            Strategy::ConcurrentPerEntryTx(16),
        ];
        for strategy in strategies {
            let mut rates = Vec::new();
            for _ in 0..RUNS {
                let run = pg::run(&db, strategy, &entries).await?;
                rates.push(run.per_second());
            }
            let shown: Vec<String> = rates.iter().map(|r| format!("{r:.0}")).collect();
            println!(
                "  {:<40} {}  (trung vị {:.0})",
                strategy.label(),
                shown.join(" / "),
                median(rates)
            );
        }
        // Cách 1 lặp lại với synchronous_commit=off: phần chênh lệch là chi phí chờ fsync.
        let no_fsync = db.pool_with_sync_commit(1, "off").await?;
        let mut rates = Vec::new();
        for _ in 0..RUNS {
            let run = pg::run_on(&db, &no_fsync, Strategy::PerEntryTx, &entries).await?;
            rates.push(run.per_second());
        }
        let shown: Vec<String> = rates.iter().map(|r| format!("{r:.0}")).collect();
        println!(
            "  {:<40} {}  (trung vị {:.0})",
            "1 transaction / bút toán, commit off",
            shown.join(" / "),
            median(rates)
        );
        no_fsync.close().await;
        Ok::<(), BoxError>(())
    }
    .await;
    db.drop_schema().await;
    result
}

/// Kiểm kết quả nghiệp vụ. Không kiểm số đo tuyệt đối vì chúng đổi theo máy và theo lượt.
async fn verify() -> Result<(), BoxError> {
    // 1. Chuyến TN-0001 đúng số liệu cố định của loạt bài.
    let first = make_entry(1);
    assert_eq!(first.idempotency_key(), "trip:TN-0001:capture");
    let amounts: Vec<i64> = first.postings().iter().map(|p| p.amount.dong()).collect();
    assert_eq!(amounts, [-25_500, 5_100, 20_400]);

    // 2. Số dư quét toàn sổ trùng số dư cộng dồn; tổng toàn sổ bằng 0.
    let mut book = IndexedLedger::new();
    for i in 1..=10_000 {
        assert!(book.post(make_entry(i))?);
    }
    for (account, want) in [
        ("driver:DR-001", 50 * 20_400),
        ("customer:CU-001", -10 * 25_500),
        ("platform:revenue", 10_000 * 5_100),
        ("driver:DR-999", 0),
    ] {
        let a = acct(account);
        let scan = book.ledger().balance(&a)?;
        assert_eq!(scan, Money::from_dong(want), "{account}");
        assert_eq!(book.balance(&a), scan, "{account}");
    }
    assert_eq!(book.ledger().grand_total()?, Money::ZERO);

    // 3. Ghi lại cùng bút toán là replay: số dư cộng dồn không đổi.
    assert!(!book.post(make_entry(1))?);
    assert_eq!(
        book.balance(&acct("driver:DR-001")),
        Money::from_dong(1_020_000)
    );

    // 4. Hai cách thu Vec cho cùng kết quả (chỉ khác số lần cấp phát).
    let ledger = make_ledger(1_000);
    let (a, b) = (flatten_grow(&ledger), flatten_with_capacity(&ledger));
    assert_eq!(a.len(), 3_000);
    assert!(a.iter().zip(&b).all(|(x, y)| x == y));

    // 5. Bộ đo chạy được: thời gian dương, không so với ngưỡng nào.
    let t = timing::measure(1_000, 2);
    assert!(t.create_and_post > Duration::ZERO && t.balance_scan_each > Duration::ZERO);

    // 6. PostgreSQL thật: 4 cách ghi cho cùng kết quả nghiệp vụ (200 bút toán, schema riêng).
    const N: u64 = 200;
    let entries: Vec<JournalEntry> = (1..=N).map(make_entry).collect();
    let mut small = IndexedLedger::new();
    for e in &entries {
        small.post(e.clone())?;
    }
    let expected = small.balance(&acct("driver:DR-001"));
    let db = LabDb::create(4).await?;
    let outcome = async {
        db.seed_accounts(&entries).await?;
        for strategy in [
            Strategy::PerEntryTx,
            Strategy::Batched(50),
            Strategy::BatchedUnnest(50),
            Strategy::ConcurrentPerEntryTx(4),
        ] {
            let run = pg::run(&db, strategy, &entries).await?;
            assert!(run.elapsed > Duration::ZERO);
            pg::check(&db, entries.len(), "driver:DR-001", expected).await?;
        }
        Ok::<(), BoxError>(())
    }
    .await;
    db.drop_schema().await;
    outcome
}

#[tokio::main]
async fn main() -> ExitCode {
    let arg = std::env::args().nth(1);
    let result: Result<(), BoxError> = match arg.as_deref() {
        Some("--verify") => match verify().await {
            Ok(()) => {
                println!("VERIFY OK");
                Ok(())
            }
            Err(e) => Err(e),
        },
        Some("--timing") => {
            timing_report();
            Ok(())
        }
        Some("--pg-bench") => pg_bench().await,
        _ => {
            demo();
            Ok(())
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("LỖI: {e}");
            ExitCode::FAILURE
        }
    }
}
