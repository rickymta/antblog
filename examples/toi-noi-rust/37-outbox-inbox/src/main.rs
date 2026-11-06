//! Chạy các thí nghiệm của bài 37 và in kết quả. `--verify` kiểm thêm các giá trị bài viết nêu.
//!
//! Tiến trình con (`child <mode> <schema> <topic>`) do chính lab sinh ra, tự thoát bằng
//! `process::exit(137)` giữa hai bước để giả lập bị kill.

use std::process::ExitCode;

use anyhow::Result;
use ledger_37_outbox_inbox::lab::with_env;
use ledger_37_outbox_inbox::scenarios::{
    child_main, dlq, dual_write, inbox_dedupe, inbox_race, ordering, outbox_atomic,
    parallel_relays, redelivery, relay_crash,
};

async fn run(verify: bool) -> Result<()> {
    let exe = std::env::current_exe()?;

    with_env(exe.clone(), async move |env| {
        let r = dual_write(&env).await?;
        println!("== 1. Dual-write ==");
        println!("-- ghi DB, commit, rồi mới gửi Kafka (tiến trình con chết giữa hai bước)");
        println!("mã thoát của con        : {:?}", r.child_exit_code);
        println!("bút toán trong DB       : {}", r.entries_after_crash);
        println!("thông điệp trong Kafka  : {}", r.kafka_after_crash);
        println!("-- gửi Kafka (đã xác nhận), rồi mới ghi DB (DB lỗi)");
        println!(
            "SQLSTATE ghi DB         : {}",
            r.db_error.as_deref().unwrap_or("?")
        );
        println!("thông điệp trong Kafka  : {}", r.kafka_ghost);
        println!("bút toán trong DB       : {}", r.ghost_entries);
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    with_env(exe.clone(), async move |env| {
        let r = outbox_atomic(&env).await?;
        println!("== 2. Outbox cùng transaction ==");
        println!(
            "ghi thành công          : {} bút toán, outbox {} dòng ({} chưa gửi)",
            r.entries_after_commit, r.outbox_after_commit.0, r.outbox_after_commit.1
        );
        println!(
            "ghi lỗi giữa chừng      : SQLSTATE {}, vẫn {} bút toán, {} dòng outbox",
            r.failed_sqlstate.as_deref().unwrap_or("?"),
            r.after_failure.0,
            r.after_failure.1
        );
        println!(
            "Kafka trước khi relay   : {} thông điệp",
            r.kafka_before_relay
        );
        println!(
            "relay một lượt          : gửi dòng {:?}, outbox còn {} chưa gửi",
            r.relayed, r.outbox_after_relay.1
        );
        println!(
            "Kafka sau relay         : key={} event-id={}",
            r.message.key.as_deref().unwrap_or("?"),
            r.message.header("event-id").unwrap_or("?")
        );
        println!("payload                 : {}", r.message.payload);
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    for use_inbox in [false, true] {
        with_env(exe.clone(), async move |env| {
            let r = relay_crash(&env, use_inbox).await?;
            println!(
                "== 3{}. Relay chết sau khi gửi, consumer {} inbox ==",
                if use_inbox { "b" } else { "a" },
                if use_inbox { "CÓ" } else { "KHÔNG có" }
            );
            println!("mã thoát của relay con  : {:?}", r.child_exit_code);
            println!(
                "sau khi con chết        : Kafka {} thông điệp, outbox {} dòng chưa đánh dấu",
                r.kafka_after_crash, r.unpublished_after_crash
            );
            println!(
                "sau khi relay chạy lại  : Kafka {} thông điệp, {} event-id khác nhau",
                r.kafka_after_retry, r.distinct_event_ids
            );
            println!("thứ tự outbox id        : {:?}", r.order);
            println!(
                "consumer                : applied {}, duplicates {}",
                r.stats.applied, r.stats.duplicates
            );
            println!(
                "số dư                   : driver:DR-001 {}, customer:CU-001 {}, platform:revenue {}",
                r.driver_balance, r.customer_balance, r.platform_balance
            );
            if verify {
                r.verify()?;
            }
            Ok(())
        })
        .await?;
    }

    for lock in [false, true] {
        with_env(exe.clone(), async move |env| {
            let r = parallel_relays(&env, lock).await?;
            println!(
                "== 4{}. Hai relay song song, {} ==",
                if lock { "b" } else { "a" },
                if lock { "FOR UPDATE SKIP LOCKED" } else { "SELECT không khóa" }
            );
            println!("relay A nhận            : {:?}", r.a_ids);
            println!("relay B nhận            : {:?}", r.b_ids);
            println!(
                "Kafka                   : {} thông điệp, {} event-id khác nhau",
                r.messages, r.distinct
            );
            if let Some((sent, messages, distinct)) = r.drain {
                println!(
                    "xả thêm 100 dòng        : hai worker gửi {sent}, Kafka thêm {messages} thông điệp, {distinct} event-id mới"
                );
            }
            if verify {
                r.verify()?;
            }
            Ok(())
        })
        .await?;
    }

    for sharded in [false, true] {
        with_env(exe.clone(), async move |env| {
            let r = ordering(&env, sharded).await?;
            println!(
                "== 5{}. Thứ tự với hai worker, {} ==",
                if sharded { "b" } else { "a" },
                if sharded {
                    "chia theo md5(key)"
                } else {
                    "không chia"
                }
            );
            println!("hai key                 : {} / {}", r.keys.0, r.keys.1);
            println!("thứ tự outbox id        : {:?}", r.order);
            println!("mỗi key đúng thứ tự     : {}", r.per_key_ordered);
            println!("toàn topic đúng thứ tự  : {}", r.globally_ordered);
            if verify {
                r.verify()?;
            }
            Ok(())
        })
        .await?;
    }

    with_env(exe.clone(), async move |env| {
        let r = inbox_dedupe(&env).await?;
        println!("== 6. Inbox: A, A, B (B cùng chuyến TN-0001 nhưng event_id khác) ==");
        println!(
            "consumer                : applied {}, duplicates {}, entry_replays {}",
            r.stats.applied, r.stats.duplicates, r.stats.entry_replays
        );
        println!(
            "DB                      : {} bút toán, outbox {} dòng, inbox {} dòng",
            r.entries, r.outbox.0, r.inbox
        );
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    with_env(exe.clone(), async move |env| {
        let r = inbox_race(&env).await?;
        println!("== 7. Tám task cùng một event_id ==");
        println!(
            "applied {}, duplicates {}; {} bút toán, {} dòng inbox",
            r.applied, r.duplicates, r.entries, r.inbox
        );
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    with_env(exe.clone(), async move |env| {
        let r = redelivery(&env).await?;
        println!("== 8. Chết trước khi commit offset ==");
        println!(
            "lần 1 (không commit)    : applied {}, offset đã commit {}",
            r.first.applied, r.committed_after_first
        );
        println!(
            "lần 2 (cùng group)      : applied {}, duplicates {}, offset đã commit {}",
            r.second.applied, r.second.duplicates, r.committed_after_second
        );
        println!("bút toán                : {}", r.entries);
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    with_env(exe, async move |env| {
        let r = dlq(&env).await?;
        println!("== 9. DLQ: tốt, JSON hỏng, tốt, tài xế không tồn tại, tốt ==");
        println!(
            "consumer                : applied {}, dead_lettered {}, tổng lần thử {}",
            r.stats.applied, r.stats.dead_lettered, r.stats.attempts
        );
        println!(
            "DB                      : {} bút toán, {} dòng inbox",
            r.entries, r.inbox
        );
        println!(
            "offset đã commit        : {} (topic có 5 thông điệp)",
            r.committed
        );
        for message in &r.dlq {
            println!(
                "DLQ                     : x-source={} x-attempts={} event-id={}",
                message.header("x-source").unwrap_or("?"),
                message.header("x-attempts").unwrap_or("?"),
                message.header("event-id").unwrap_or("?")
            );
            println!(
                "                          x-error={}",
                message.header("x-error").unwrap_or("?")
            );
        }
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    if verify {
        println!("VERIFY OK");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("child") if args.len() == 4 => child_main(&args[1], &args[2], &args[3]).await,
        _ => run(args.iter().any(|a| a == "--verify")).await,
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("LỖI: {err:#}");
            ExitCode::FAILURE
        }
    }
}
