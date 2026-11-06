//! Test của lab 61. Cần PostgreSQL (`TOI_NOI_PG_URL`); thiếu thì FAIL, không bỏ qua.
//! Không test nào assert thời gian; thứ được kiểm là số đếm và bất biến.

use std::time::Duration;

use ledger_core::{AccountId, JournalEntry, Money, Posting};
use optimize_measured::alloc_count::measure;
use optimize_measured::columns::{Columns, dedupe};
use optimize_measured::db::{BoxError, LabDb, State, balances_match_postings, read_state};
use optimize_measured::experiments::alloc_experiments;
use optimize_measured::loadgen::open_loop;
use optimize_measured::strategies::{Strategy, copy_unbalanced_probe, write};
use optimize_measured::workload::make_entries;
use sqlx::Connection;

/// Sai thì trả `Err`, không panic: panic sẽ bỏ qua bước dọn schema của `lab!`.
macro_rules! check {
    ($left:expr, $right:expr, $what:expr) => {
        match (&$left, &$right) {
            (left, right) if left != right => {
                return Err(format!("{:?}: {left:?} khác {right:?}", $what).into());
            }
            _ => {}
        }
    };
}

/// Tạo schema lab, chạy `body`, luôn dọn schema kể cả khi `body` lỗi.
macro_rules! lab {
    ($trigger:expr, |$db:ident| $body:block) => {{
        let $db = LabDb::create($trigger)
            .await
            .expect("kết nối được PostgreSQL của lab");
        let result: Result<(), BoxError> = async {
            $body
            Ok(())
        }
        .await;
        $db.drop_schema().await;
        result.expect("ca kiểm");
    }};
}

async fn state_after(
    db: &LabDb,
    strategy: Strategy,
    entries: &[JournalEntry],
) -> Result<State, BoxError> {
    db.reset().await?;
    let mut conn = db.connect().await?;
    write(&mut conn, strategy, entries).await?;
    let state = read_state(&mut conn).await?;
    check!(balances_match_postings(&mut conn).await?, true, strategy);
    conn.close().await?;
    Ok(state)
}

#[tokio::test]
async fn four_strategies_leave_the_same_balances() {
    lab!(false, |db| {
        let entries = make_entries(1, 60);
        let base = state_after(&db, Strategy::PerRow, &entries).await?;
        check!(
            (base.entries, base.postings, base.posting_sum),
            (60, 180, 0),
            "từng dòng"
        );
        for strategy in [Strategy::Unnest(25), Strategy::Copy(25), Strategy::Cte(25)] {
            let state = state_after(&db, strategy, &entries).await?;
            check!(state.balances, base.balances, strategy);
            check!(state.postings, 180, strategy);
        }
    });
}

#[tokio::test]
async fn replay_and_duplicate_keys_add_nothing() {
    lab!(false, |db| {
        let entries = make_entries(1, 20);
        for strategy in [Strategy::Unnest(8), Strategy::Copy(8), Strategy::Cte(8)] {
            let first = state_after(&db, strategy, &entries).await?;
            let mut conn = db.connect().await?;
            write(&mut conn, strategy, &entries).await?;
            check!(read_state(&mut conn).await?, first, strategy);
            // Hai bút toán cùng key trong một lô chỉ được ghi một.
            db.reset().await?;
            let dup = [entries[0].clone(), entries[0].clone(), entries[1].clone()];
            write(&mut conn, strategy, &dup).await?;
            check!(read_state(&mut conn).await?.entries, 2, strategy);
            conn.close().await?;
        }
    });
}

#[tokio::test]
async fn copy_escapes_awkward_account_ids() {
    lab!(false, |db| {
        let odd = "odd:a\tb\\c";
        let mut conn = db.connect().await?;
        sqlx::query("insert into accounts (id, kind) values ($1, 'liability')")
            .bind(odd)
            .execute(&mut conn)
            .await?;
        let entry = JournalEntry::try_new(
            "odd:1",
            vec![
                Posting::new(AccountId::new(odd)?, Money::from_dong(-700)),
                Posting::new(AccountId::new("platform:revenue")?, Money::from_dong(700)),
            ],
        )?;
        write(&mut conn, Strategy::Copy(10), &[entry]).await?;
        let stored: String =
            sqlx::query_scalar("select account_id from postings where amount_dong = -700")
                .fetch_one(&mut conn)
                .await?;
        check!(stored, odd, "mã tài khoản lạ");
        conn.close().await?;
    });
}

#[tokio::test]
async fn trigger_still_guards_copy() {
    lab!(true, |db| {
        let mut conn = db.connect().await?;
        check!(
            copy_unbalanced_probe(&mut conn).await?.as_deref(),
            Some("23514"),
            "COPY lệch"
        );
        // Bút toán cân vẫn ghi được qua COPY khi có trigger.
        write(&mut conn, Strategy::Copy(10), &make_entries(1, 25)).await?;
        let state = read_state(&mut conn).await?;
        check!((state.entries, state.posting_sum), (25, 0), "COPY cân");
        conn.close().await?;
    });
}

#[test]
fn allocation_counts_are_exact() {
    let r = alloc_experiments(2_000);
    assert_eq!(r.need_change, 100);
    assert_eq!((r.clone.allocs, r.cow.allocs), (2_000, 100));
    assert_eq!(r.columns_borrow.allocs, 5);
    assert_eq!(r.columns_clone.allocs, 14_005);
}

#[test]
fn dedupe_keeps_first_occurrence_without_allocating_per_entry() {
    let entries = make_entries(1, 3);
    let with_dup = [entries[0].clone(), entries[0].clone(), entries[1].clone()];
    let (uniq, stats) = measure(|| dedupe(&with_dup));
    assert_eq!(uniq.len(), 2);
    // HashSet + Vec kết quả: số cấp phát không tăng theo số bút toán.
    assert!(stats.allocs <= 4, "{stats:?}");
    let cols = Columns::borrow(&uniq);
    assert_eq!(cols.keys.len(), 2);
    assert_eq!(cols.accounts.len(), 6);
}

#[tokio::test]
async fn open_loop_records_every_scheduled_request() {
    // Không database: request giả luôn thành công. Số mẫu phải đúng bằng số request theo lịch.
    let mut run = open_loop(100, Duration::from_millis(300), |_| async { Ok(()) }).await;
    assert_eq!((run.sent, run.errors), (30, 0));
    assert_eq!(run.samples.len(), 30);
    assert!(run.samples.max() < u64::MAX);
}
