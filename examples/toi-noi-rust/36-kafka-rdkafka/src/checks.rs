//! Các giá trị kỳ vọng in trong bài. Sai thì trả lỗi; `main` đổi lỗi thành exit code 1.

use std::collections::BTreeSet;

use anyhow::{Result, ensure};

use crate::scenarios::{AtLeastOnceReport, GroupReport, KeysReport, NoRewindReport, PoisonReport};
use crate::service::Summary;

pub fn check_at_least_once(r: &AtLeastOnceReport) -> Result<()> {
    let first = Summary::Posted("JE-000001".to_string());
    ensure!(
        r.run1.summary == first,
        "lần 1 phải ghi JE-000001: {:?}",
        r.run1
    );
    ensure!(
        r.committed_before_run2.is_none(),
        "chưa commit nên group chưa có offset"
    );
    ensure!(
        r.run2.summary == Summary::Replayed("JE-000001".to_string()),
        "lần 2 phải nhận lại và replay: {:?}",
        r.run2
    );
    ensure!(
        (r.run1.partition, r.run1.offset) == r.delivered
            && (r.run2.partition, r.run2.offset) == r.delivered,
        "hai lần phải là cùng một thông điệp (partition, offset)"
    );
    ensure!(
        r.committed_after_run2 == Some(r.delivered.1 + 1),
        "commit phải trỏ tới offset kế tiếp"
    );
    ensure!(
        !r.redelivered_after_commit,
        "đã commit thì không được nhận lại"
    );
    ensure!(
        r.ledger_entries == 1,
        "sổ cái chỉ có một bút toán, có {}",
        r.ledger_entries
    );
    ensure!(
        r.driver_balance == 20_400,
        "số dư tài xế phải là 20.400, có {}",
        r.driver_balance
    );
    ensure!(r.output.len() == 2, "sự kiện đầu ra được phát hai lần");
    for event in &r.output {
        ensure!(event.idempotency_key == "trip:TN-0001:capture" && event.entry_id == "JE-000001");
        let amounts: Vec<i64> = event.postings.iter().map(|p| p.amount_dong).collect();
        ensure!(
            amounts == [-25_500, 5_100, 20_400],
            "posting sai: {amounts:?}"
        );
    }
    Ok(())
}

pub fn check_keys(r: &KeysReport) -> Result<()> {
    ensure!(r.by_trip_key.values().sum::<usize>() == 12);
    ensure!(
        r.by_trip_key.len() > 1,
        "key theo tripId phải trải ra nhiều partition"
    );
    ensure!(
        r.by_platform_key.len() == 1,
        "key nền tảng dồn hết vào một partition"
    );
    ensure!(r.by_platform_key.values().sum::<usize>() == 12);
    let distinct: BTreeSet<i32> = r.account_partitions.iter().map(|(_, p)| *p).collect();
    ensure!(
        distinct.len() > 1,
        "ba tài khoản của một bút toán phải rơi vào nhiều partition"
    );
    ensure!(
        r.account_partitions
            .iter()
            .all(|(_, p)| (0..r.partitions).contains(p))
    );
    let (p0, o0) = r.same_key[0];
    ensure!(
        r.same_key
            .iter()
            .enumerate()
            .all(|(i, (p, o))| *p == p0 && *o == o0 + i as i64),
        "cùng key phải cùng partition và offset tăng liên tục: {:?}",
        r.same_key
    );
    ensure!(
        r.consumed_n == [1, 2, 3, 4, 5],
        "thứ tự nhận phải giữ: {:?}",
        r.consumed_n
    );
    ensure!(
        r.hash_compare.iter().any(|(_, crc, mur)| crc != mur),
        "CRC32 và murmur2 phải khác nhau ở ít nhất một key: {:?}",
        r.hash_compare
    );
    Ok(())
}

pub fn check_group(r: &GroupReport) -> Result<()> {
    let all: Vec<i32> = (0..r.partitions as i32).collect();
    ensure!(
        r.alone == all,
        "consumer đầu phải có mọi partition: {:?}",
        r.alone
    );
    let a: BTreeSet<i32> = r.first_after_join.iter().copied().collect();
    let b: BTreeSet<i32> = r.second_after_join.iter().copied().collect();
    ensure!(
        a.is_disjoint(&b) && !a.is_empty() && !b.is_empty(),
        "hai phần phải rời nhau và khác rỗng"
    );
    ensure!(
        a.union(&b).count() == r.partitions,
        "hai phần phải phủ hết partition"
    );
    ensure!(
        r.partitions_seen.0.is_subset(&a) && r.partitions_seen.1.is_subset(&b),
        "mỗi consumer chỉ nhận từ partition của mình"
    );
    ensure!(
        r.ledger_entries == 12,
        "phải có đúng 12 bút toán, có {}",
        r.ledger_entries
    );
    ensure!(r.processed.0 + r.processed.1 >= 12, "tổng xử lý phải ≥ 12");
    ensure!(
        r.second_after_leave == all,
        "consumer 2 phải nhận lại mọi partition"
    );
    ensure!(
        !r.events.0.is_empty() && !r.events.1.is_empty(),
        "phải ghi được các lần rebalance"
    );
    Ok(())
}

pub fn check_poison(r: &PoisonReport) -> Result<()> {
    ensure!(r.resilient.len() == 5);
    for (i, step) in r.resilient.iter().enumerate() {
        ensure!(
            step.offset == i as i64 && step.partition == 0,
            "offset phải đi tuần tự: {step:?}"
        );
    }
    ensure!(r.resilient[0].summary == Summary::Posted("JE-000001".to_string()));
    ensure!(matches!(&r.resilient[1].summary, Summary::Skipped(s) if s.contains("JSON hỏng")));
    ensure!(
        matches!(&r.resilient[2].summary, Summary::Skipped(s) if s.contains("schemaVersion 2"))
    );
    ensure!(matches!(&r.resilient[3].summary, Summary::Skipped(s) if s.contains("fareDong")));
    ensure!(r.resilient[4].summary == Summary::Posted("JE-000002".to_string()));
    ensure!(
        r.resilient_committed == Some(5),
        "offset commit phải vượt qua thông điệp xấu"
    );
    ensure!(r.resilient_entries == 2);
    ensure!(
        r.naive_failures.len() == 2 && r.naive_failures.iter().all(|(o, _)| *o == 1),
        "consumer ngây thơ phải kẹt ở offset 1"
    );
    ensure!(r.naive_committed == Some(1));
    ensure!(r.naive_entries == 1, "TN-0002 không bao giờ được ghi");
    Ok(())
}

pub fn check_no_rewind(r: &NoRewindReport) -> Result<()> {
    ensure!(
        r.failed == 0 && r.next == 1,
        "không commit vẫn phải đi tiếp"
    );
    ensure!(
        r.committed == Some(2),
        "commit offset 1 che luôn offset 0: {:?}",
        r.committed
    );
    ensure!(r.after_seek == 0, "seek phải đọc lại offset 0");
    ensure!(
        r.after_restart == 2,
        "khởi động lại phải đi từ offset đã commit, có {}",
        r.after_restart
    );
    Ok(())
}

pub fn check_timeout(message: &str) -> Result<()> {
    ensure!(
        message.contains("hết") && message.contains("1.5s"),
        "thông báo timeout sai: {message}"
    );
    Ok(())
}
