//! Chạy các thí nghiệm của bài 24 và in kết quả. `--verify` kiểm thêm các giá trị bài viết nêu.

use std::process::ExitCode;

use anyhow::Result;
use ledger_24_transaction_idempotency::lab::with_lab;
use ledger_24_transaction_idempotency::scenarios::{
    append_only, atomicity, idempotency, limit_race, on_conflict_after_snapshot, on_conflict_waits,
};
use ledger_24_transaction_idempotency::store::{Isolation, display_id};

async fn run(verify: bool) -> Result<()> {
    // Mỗi thí nghiệm có schema riêng nên id bút toán luôn bắt đầu từ 1.
    with_lab(async move |lab| {
        let r = atomicity(&lab).await?;
        println!("== 1. Transaction ==");
        println!(
            "commit TN-0001      : {:?} ({})",
            r.committed,
            display_id(r.committed.id())
        );
        println!(
            "sau commit          : {} bút toán, {} posting",
            r.after_commit.0, r.after_commit.1
        );
        println!(
            "lỗi ở posting 3     : SQLSTATE {} ràng buộc {}",
            r.failure_sqlstate.as_deref().unwrap_or("?"),
            r.failure_constraint.as_deref().unwrap_or("?")
        );
        println!(
            "sau lỗi giữa chừng  : {} bút toán, {} posting",
            r.after_failure.0, r.after_failure.1
        );
        println!(
            "trong tx chưa commit: {} bút toán, {} posting",
            r.inside_open_tx.0, r.inside_open_tx.1
        );
        println!(
            "sau khi drop tx     : {} bút toán, {} posting",
            r.after_drop.0, r.after_drop.1
        );
        println!(
            "ghi lại key đã drop : {:?} (id có khoảng trống)",
            r.rewrite_after_drop
        );
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    with_lab(async move |lab| {
        let r = idempotency(&lab).await?;
        println!("== 2. Idempotency đồng thời ==");
        println!("tokio::join! 2 task : {:?} / {:?}", r.pair.0, r.pair.1);
        println!(
            "8 task cùng key     : {} Created, {} Replayed, cùng id: {}",
            r.created_of_8, r.replayed_of_8, r.same_id
        );
        println!(
            "số dòng             : {} bút toán, {} posting",
            r.rows.0, r.rows.1
        );
        println!("cùng key, cước khác : {}", r.conflict);
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    println!("== 3. ON CONFLICT đợi giao dịch kia ==");
    for first_commits in [true, false] {
        with_lab(async move |lab| {
            let r = on_conflict_waits(&lab, first_commits).await?;
            println!(
                "giao dịch đầu {:8}: phiên hai chờ {}/{}, bị chặn: {}, kết quả: {}",
                if first_commits { "commit" } else { "rollback" },
                r.wait_event_type.as_deref().unwrap_or("-"),
                r.wait_event.as_deref().unwrap_or("-"),
                r.blocked_while_open,
                match r.second_result {
                    None => "không chèn (đọc lại bút toán cũ)".to_string(),
                    Some(id) => format!("chèn được, id {id}"),
                }
            );
            if verify {
                r.verify(first_commits)?;
            }
            Ok(())
        })
        .await?;
    }

    for isolation in [
        Isolation::ReadCommitted,
        Isolation::RepeatableRead,
        Isolation::Serializable,
    ] {
        with_lab(async move |lab| {
            let r = on_conflict_after_snapshot(&lab, isolation).await?;
            println!(
                "snapshot cũ {:14}: {}, đọc lại thấy id {}",
                format!("{:?}", r.isolation),
                match &r.insert_sqlstate {
                    None => "0 dòng".to_string(),
                    Some(code) => format!("SQLSTATE {code}, thử giao dịch mới: 0 dòng"),
                },
                r.seen_id.unwrap_or_default()
            );
            if verify {
                r.verify()?;
            }
            Ok(())
        })
        .await?;
    }

    println!("== 4. Hạn mức rút 30.000, hai yêu cầu 20.400 cùng lúc ==");
    for (isolation, driver) in [
        (Isolation::ReadCommitted, "driver:DR-002"),
        (Isolation::RepeatableRead, "driver:DR-002"),
        (Isolation::Serializable, "driver:DR-003"),
    ] {
        with_lab(async move |lab| {
            let r = limit_race(&lab, isolation, driver).await?;
            println!(
                "{:14}: {} ok, {} bị từ chối, {} lần 40001, số lần thử {:?}, tổng đã rút {} đồng",
                format!("{:?}", r.isolation),
                r.ok,
                r.rejected,
                r.serialization_failures,
                r.attempts,
                r.total_out
            );
            if verify {
                r.verify()?;
            }
            Ok(())
        })
        .await?;
    }

    with_lab(async move |lab| {
        let r = append_only(&lab).await?;
        println!("== 5. Append-only ==");
        for item in &r.rejected {
            println!(
                "{} -> SQLSTATE {}: {}",
                item.statement,
                item.sqlstate.as_deref().unwrap_or("?"),
                item.message
            );
        }
        println!(
            "sau khi thử phá: {} bút toán, {} posting, tổng {} đồng",
            r.after.0, r.after.1, r.after.2
        );
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
    let verify = std::env::args().any(|arg| arg == "--verify");
    match run(verify).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("LỖI: {err:#}");
            ExitCode::FAILURE
        }
    }
}
