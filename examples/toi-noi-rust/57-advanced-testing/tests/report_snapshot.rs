//! Snapshot báo cáo đối soát bằng insta: một bản tất định hoàn toàn, một bản có redaction.

use ledger_57_advanced_testing::clock::{StepClock, SystemClock};
use ledger_57_advanced_testing::fixture;
use ledger_57_advanced_testing::report::{new_run_id, reconcile};

#[test]
fn report_text_with_fake_clock() {
    // Đồng hồ giả và mã chạy cố định: không cần redaction, snapshot giống nhau mọi lần.
    let clock = StepClock::new("2026-11-05T17:00:00Z".parse().unwrap(), 250);
    let report = reconcile(&fixture::ledger(), fixture::STATEMENT, &clock, "run-0001");
    insta::assert_snapshot!(report.render_text());
}

#[test]
fn report_json_with_redactions() {
    // Đồng hồ thật và UUID ngẫu nhiên: ba trường đổi mỗi lần chạy phải được che trước khi so.
    let report = reconcile(
        &fixture::ledger(),
        fixture::STATEMENT,
        &SystemClock,
        &new_run_id(),
    );
    insta::assert_json_snapshot!(report, {
        ".run_id" => "[run_id]",
        ".generated_at" => "[timestamp]",
        ".duration_ms" => "[ms]",
    });
}
