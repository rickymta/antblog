//! `cargo run` in kết quả các kịch bản; `cargo run -- --verify` kiểm các giá trị kỳ vọng của bài.

use std::process::ExitCode;

use anyhow::Result;
use ledger_36_kafka_rdkafka::checks::{
    check_at_least_once, check_group, check_keys, check_no_rewind, check_poison, check_timeout,
};
use ledger_36_kafka_rdkafka::kafka::Lab;
use ledger_36_kafka_rdkafka::scenarios::{
    AtLeastOnceReport, GroupReport, KeysReport, NoRewindReport, PoisonReport, at_least_once,
    group_rebalance, idle_timeout, keys_and_order, no_commit_does_not_rewind,
    poison_messages_scenario,
};

#[tokio::main]
async fn main() -> ExitCode {
    let verify = std::env::args().any(|a| a == "--verify");
    match run(verify).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("LỖI: {err:#}");
            ExitCode::FAILURE
        }
    }
}

// Mỗi kịch bản một `Lab` riêng: topic/group riêng, dọn xong mới sang kịch bản kế.
async fn run(verify: bool) -> Result<()> {
    let lab = Lab::start().await?;
    println!("== 1. At-least-once: xử lý xong, chưa commit, khởi động lại ==");
    let result = at_least_once(&lab).await;
    let cleanup = lab.finish().await;
    let report = result?;
    cleanup?;
    print_at_least_once(&report);
    if verify {
        check_at_least_once(&report)?;
    }

    let lab = Lab::start().await?;
    println!("\n== 2. Thứ tự theo partition và chọn key ==");
    let result = keys_and_order(&lab).await;
    let cleanup = lab.finish().await;
    let report = result?;
    cleanup?;
    print_keys(&report);
    if verify {
        check_keys(&report)?;
    }

    let lab = Lab::start().await?;
    println!("\n== 3. Consumer group và rebalance ==");
    let result = group_rebalance(&lab).await;
    let cleanup = lab.finish().await;
    let report = result?;
    cleanup?;
    print_group(&report);
    if verify {
        check_group(&report)?;
    }

    let lab = Lab::start().await?;
    println!("\n== 4. Thông điệp hỏng và version lạ ==");
    let result = poison_messages_scenario(&lab).await;
    let cleanup = lab.finish().await;
    let report = result?;
    cleanup?;
    print_poison(&report);
    if verify {
        check_poison(&report)?;
    }

    let lab = Lab::start().await?;
    println!("\n== 5. Không commit không có nghĩa là đọc lại ==");
    let result = no_commit_does_not_rewind(&lab).await;
    let cleanup = lab.finish().await;
    let report = result?;
    cleanup?;
    print_no_rewind(&report);
    if verify {
        check_no_rewind(&report)?;
    }

    let lab = Lab::start().await?;
    println!("\n== 6. Hết giờ chờ ==");
    let result = idle_timeout(&lab).await;
    let cleanup = lab.finish().await;
    let message = result?;
    cleanup?;
    println!("topic rỗng: {message}");
    if verify {
        check_timeout(&message)?;
        println!("\nVERIFY OK");
    }
    Ok(())
}

fn print_at_least_once(r: &AtLeastOnceReport) {
    println!(
        "gửi TripCompleted TN-0001: partition {}, offset {}",
        r.delivered.0, r.delivered.1
    );
    println!(
        "lần chạy 1 (partition {}, offset {}): {}; chết trước khi commit",
        r.run1.partition, r.run1.offset, r.run1.summary
    );
    println!(
        "offset đã commit của group trước lần chạy 2: {:?}",
        r.committed_before_run2
    );
    println!(
        "lần chạy 2 (partition {}, offset {}): {}; commit",
        r.run2.partition, r.run2.offset, r.run2.summary
    );
    println!(
        "offset đã commit sau lần chạy 2: {:?}",
        r.committed_after_run2
    );
    println!(
        "lần chạy 3 nhận lại sau commit: {}",
        r.redelivered_after_commit
    );
    println!(
        "bút toán trong sổ: {}; số dư driver:DR-001: {} đồng",
        r.ledger_entries, r.driver_balance
    );
    println!(
        "LedgerEntryPosted trên topic đầu ra: {} sự kiện",
        r.output.len()
    );
    for e in &r.output {
        println!(
            "  {} key={} postings={:?}",
            e.entry_id,
            e.idempotency_key,
            e.postings.iter().map(|p| p.amount_dong).collect::<Vec<_>>()
        );
    }
}

fn print_keys(r: &KeysReport) {
    println!("topic {} partition", r.partitions);
    println!("key = tripId, 12 chuyến: {:?}", r.by_trip_key);
    println!(
        "key = platform:revenue, 12 bút toán: {:?}",
        r.by_platform_key
    );
    println!("ba tài khoản của bút toán trip:TN-0001:capture, key = tài khoản:");
    for (account, partition) in &r.account_partitions {
        println!("  {account} -> partition {partition}");
    }
    println!("5 thông điệp cùng key TN-0001: {:?}", r.same_key);
    println!("thứ tự nhận lại: {:?}", r.consumed_n);
    for (key, crc, murmur) in &r.hash_compare {
        println!(
            "key {key}: consistent_random -> partition {crc}, murmur2_random -> partition {murmur}"
        );
    }
}

fn print_group(r: &GroupReport) {
    println!(
        "{} partition. Consumer 1 một mình: {:?}",
        r.partitions, r.alone
    );
    println!(
        "consumer 2 vào group: consumer 1 giữ {:?}, consumer 2 nhận {:?}",
        r.first_after_join, r.second_after_join
    );
    println!(
        "12 chuyến, key = tripId: consumer 1 xử lý {}, consumer 2 xử lý {}; bút toán trong sổ: {}",
        r.processed.0, r.processed.1, r.ledger_entries
    );
    println!(
        "consumer 1 rời group: consumer 2 nhận lại {:?}",
        r.second_after_leave
    );
    println!("rebalance của consumer 1: {:?}", r.events.0);
    println!("rebalance của consumer 2: {:?}", r.events.1);
}

fn print_no_rewind(r: &NoRewindReport) {
    println!("offset {}: lỗi giả lập, không commit", r.failed);
    println!("thông điệp kế tiếp vẫn đến: offset {}; commit nó", r.next);
    println!("offset đã commit: {:?}", r.committed);
    println!("seek về offset {}: nhận offset {}", r.failed, r.after_seek);
    println!("khởi động lại không seek: nhận offset {}", r.after_restart);
}

fn print_poison(r: &PoisonReport) {
    println!("consumer bỏ qua thông điệp xấu và vẫn commit:");
    for s in &r.resilient {
        println!("  offset {}: {}", s.offset, s.summary);
    }
    println!(
        "offset đã commit: {:?}; bút toán trong sổ: {}",
        r.resilient_committed, r.resilient_entries
    );
    println!("consumer ngây thơ (dừng khi gặp lỗi), khởi động hai lần:");
    for (offset, why) in &r.naive_failures {
        println!("  dừng ở offset {offset}: {why}");
    }
    println!(
        "offset đã commit: {:?}; bút toán trong sổ: {}",
        r.naive_committed, r.naive_entries
    );
}
