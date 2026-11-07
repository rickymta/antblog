//! Kiểm ba cách cài cho cùng đáp án, bộ đếm theo công thức, và hai kiểu handler trên runtime một luồng.

use ledger_core::Money;
use ledger_profiling::books::{Book, IndexedBook, ScanBook, SlowBook};
use ledger_profiling::runtime::{CHUNK, Mode, run_scenario};
use ledger_profiling::workload::{
    customer_of, driver_of, expected_scanned, make_entry, make_workload, post_and_read_balances,
};

#[test]
fn trip_one_matches_the_fixed_numbers() {
    let amounts: Vec<i64> = make_entry(1)
        .postings()
        .iter()
        .map(|p| p.amount.dong())
        .collect();
    assert_eq!(amounts, [-25_500, 5_100, 20_400]);
}

#[test]
fn three_books_give_the_same_answer() {
    let slow = post_and_read_balances(&mut SlowBook::new(), make_workload(150));
    let scan = post_and_read_balances(&mut ScanBook::new(), make_workload(150));
    let indexed = post_and_read_balances(&mut IndexedBook::new(), make_workload(150));
    assert_eq!(slow.checksum, scan.checksum);
    assert_eq!(slow.checksum, indexed.checksum);
}

#[test]
fn scanned_postings_follow_the_formula() {
    let n = 120;
    let slow = post_and_read_balances(&mut SlowBook::new(), make_workload(n));
    let scan = post_and_read_balances(&mut ScanBook::new(), make_workload(n));
    let indexed = post_and_read_balances(&mut IndexedBook::new(), make_workload(n));
    assert_eq!(slow.balance_calls, 2 * n);
    assert_eq!(slow.scanned_postings, expected_scanned(n));
    assert_eq!(scan.scanned_postings, expected_scanned(n));
    assert_eq!(indexed.scanned_postings, 0);
}

#[test]
fn balances_of_one_customer_and_one_driver() {
    // 1.000 khách và 200 tài xế xoay vòng: sau 1.200 chuyến CU-001 có 2 chuyến (chuyến 1 và 1.001),
    // DR-001 có 6 chuyến (1, 201, 401, 601, 801, 1.001).
    let mut book = IndexedBook::new();
    for entry in make_workload(1_200).entries {
        book.post(entry);
    }
    assert_eq!(book.balance(&customer_of(1)), Money::from_dong(-2 * 25_500));
    assert_eq!(book.balance(&driver_of(1)), Money::from_dong(6 * 20_400));
}

fn small_scan_book() -> ScanBook {
    let mut book = ScanBook::new();
    // 4 đoạn CHUNK bút toán.
    for entry in make_workload(4 * CHUNK as u64).entries {
        book.post(entry);
    }
    book
}

#[tokio::test(flavor = "current_thread")]
async fn blocking_handler_starves_the_heartbeat() {
    let run = run_scenario(Mode::Blocking, small_scan_book(), customer_of(1), 3).await;
    assert_eq!(
        run.ticks_at_finish, 0,
        "không có .await nên heartbeat chưa được chạy lần nào"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn chunked_handler_lets_the_heartbeat_run() {
    let blocking = run_scenario(Mode::Blocking, small_scan_book(), customer_of(1), 3).await;
    let chunked = run_scenario(Mode::Chunked, small_scan_book(), customer_of(1), 3).await;
    assert!(chunked.ticks_at_finish > 0);
    assert_eq!(
        chunked.balance, blocking.balance,
        "chia đoạn không đổi đáp án"
    );
}
