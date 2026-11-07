//! Property test: với mọi chuỗi «ghi bút toán / chạy job snapshot» ngẫu nhiên (kể cả bút toán đúng
//! ranh giới nửa đêm và ghi lùi), «snapshot gần nhất + phần chênh» luôn bằng tính lại toàn bộ.

use std::sync::OnceLock;

use anyhow::Result;
use jiff::Timestamp;
use jiff::civil::Date;
use ledger_48_snapshot_period_close::db::LabDb;
use ledger_48_snapshot_period_close::period::seed_months;
use ledger_48_snapshot_period_close::post::{entry, post};
use ledger_48_snapshot_period_close::snapshot::{
    balance_full, balance_snapshot_delta, snapshot_day, verify_snapshots,
};
use ledger_48_snapshot_period_close::tz::ts;
use proptest::prelude::*;
use tokio::runtime::Runtime;

/// 00:00 ngày 01/11/2026 giờ Việt Nam, tính bằng giây UTC.
fn base() -> Result<i64> {
    Ok(ts("2026-11-01T00:00:00+07:00")?.as_second())
}

#[derive(Debug, Clone)]
enum Op {
    Post {
        a: usize,
        b: usize,
        amount: i64,
        secs: i64,
    },
    Snap {
        day: i8,
    },
}

/// Giây kể từ 00:00 ngày 01/11: ngẫu nhiên, hoặc đúng nửa đêm, hoặc 1 giây trước nửa đêm.
fn secs() -> impl Strategy<Value = i64> {
    prop_oneof![
        2 => 0..5 * 86_400i64,
        1 => (1..5i64).prop_map(|d| d * 86_400),
        1 => (1..5i64).prop_map(|d| d * 86_400 - 1),
    ]
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        3 => (0..4usize, 0..4usize, 1..50_000i64, secs())
            .prop_map(|(a, b, amount, secs)| Op::Post { a, b, amount, secs }),
        1 => (1..=5i8).prop_map(|day| Op::Snap { day }),
    ]
}

fn runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| Runtime::new().expect("tạo runtime tokio"))
}

async fn check(db: &LabDb, ops: &[Op]) -> Result<()> {
    db.migrate().await?;
    seed_months(&db.pool, 2026, 11, 1).await?;
    let base = base()?;
    let mut cuts: Vec<i64> = (0..=5).map(|d| base + d * 86_400).collect();
    for (i, o) in ops.iter().enumerate() {
        match o {
            Op::Post { a, b, amount, secs } => {
                let b = if a == b { (b + 1) % 4 } else { *b };
                let e = entry(
                    &format!("pt:{i}"),
                    &[
                        (&format!("acct:{a}"), -amount),
                        (&format!("acct:{b}"), *amount),
                    ],
                )?;
                let t = Timestamp::from_second(base + secs)?;
                post(&db.pool, &e, t, t, None).await?;
                cuts.push(base + secs);
                cuts.push(base + secs + 1);
            }
            Op::Snap { day } => {
                snapshot_day(&db.pool, Date::constant(2026, 11, *day)).await?;
            }
        }
    }
    for account in 0..4 {
        let name = format!("acct:{account}");
        for &cut in &cuts {
            let cut = Timestamp::from_second(cut)?;
            let full = balance_full(&db.pool, &name, cut).await?;
            let fast = balance_snapshot_delta(&db.pool, &name, cut).await?;
            anyhow::ensure!(
                full == fast,
                "{name} tại {cut}: tính lại {full}, snapshot + chênh {fast}"
            );
        }
    }
    let bad = verify_snapshots(&db.pool, None).await?;
    anyhow::ensure!(bad.is_empty(), "snapshot lệch: {bad:?}");
    Ok(())
}

async fn run_case(ops: Vec<Op>) -> Result<()> {
    let db = LabDb::create().await?;
    let result = check(&db, &ops).await;
    let cleaned = db.cleanup().await;
    result?;
    cleaned
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 16, failure_persistence: None, ..ProptestConfig::default() })]

    #[test]
    fn snapshot_plus_delta_equals_full_recompute(ops in proptest::collection::vec(op(), 1..14)) {
        let outcome = runtime().block_on(run_case(ops));
        prop_assert!(outcome.is_ok(), "{:?}", outcome.err());
    }
}
