//! Chạy các thí nghiệm của bài 50 và in kết quả.
//! `--verify` kiểm các giá trị bài viết nêu; `--measure` đo thông lượng (số đo, không phải SLA).

use std::process::ExitCode;

use anyhow::Result;
use ledger_audit::canonical::{GENESIS, demo_entry, entry_hash};
use ledger_audit::chain::{ChainReport, LinkMode};
use ledger_audit::lab::{leftover_roles, schema_exists, with_lab, with_lab_value};
use ledger_audit::scenarios::{
    Outcome, PROBES, anchor_demo, chain_basics, layers, locked_chain, measure, naive_fork,
    permission_matrix, pii, structural_faults, tamper_detect, unique_backstop, verify_matrix,
};

// Tên schema đổi mỗi lần chạy; rút gọn để output dễ đọc và so sánh.
fn short(msg: &str, schema: &str) -> String {
    msg.replace(schema, "lab_<uuid>")
}

fn fault_text(report: &ChainReport) -> String {
    match &report.first_fault {
        None => format!("chuỗi nguyên vẹn, đã duyệt {} dòng", report.checked),
        Some(f) => format!(
            "dòng đầu tiên sai: seq={:?} entry_id={} {:?} (đã duyệt {} dòng trước nó)",
            f.seq, f.entry_id, f.kind, report.checked
        ),
    }
}

async fn run(verify: bool) -> Result<()> {
    println!("== 0. Dạng byte chuẩn và hash (hàm thuần) ==");
    let canonical = demo_entry().to_bytes();
    println!(
        "JE-000001 canonical ({} byte): {}",
        canonical.len(),
        hex::encode(&canonical)
    );
    let h = entry_hash(&GENESIS, &canonical);
    println!(
        "entry_hash = SHA-256(32 byte 0 || canonical): {}",
        hex::encode(h)
    );

    println!("== 1. Ba lớp bảo vệ: REVOKE, trigger, chủ bảng ==");
    with_lab(async move |lab| {
        let r = layers(&lab).await?;
        println!(
            "bảng nháp, còn quyền       : UPDATE được {} dòng",
            r.scratch_update_rows_before
        );
        println!(
            "sau REVOKE, UPDATE         : {} {}",
            r.scratch_update_after_revoke.0, r.scratch_update_after_revoke.1
        );
        println!(
            "sau REVOKE, DELETE         : {} {}",
            r.scratch_delete_after_revoke.0, r.scratch_delete_after_revoke.1
        );
        println!(
            "sau REVOKE, TRUNCATE       : {} {}",
            r.scratch_truncate_after_revoke.0, r.scratch_truncate_after_revoke.1
        );
        println!(
            "role ứng dụng, UPDATE      : {} {}",
            r.writer_update.0, r.writer_update.1
        );
        println!(
            "role ứng dụng, DELETE      : {} {}",
            r.writer_delete.0, r.writer_delete.1
        );
        println!(
            "role ứng dụng, TRUNCATE    : {} {}",
            r.writer_truncate.0, r.writer_truncate.1
        );
        println!(
            "chủ bảng, UPDATE           : {} {}",
            r.owner_blocked_by_trigger.0,
            short(&r.owner_blocked_by_trigger.1, &lab.schema)
        );
        println!(
            "chủ bảng tự REVOKE         : {} {}",
            r.owner_after_self_revoke.0, r.owner_after_self_revoke.1
        );
        println!(
            "chủ bảng tự GRANT lại      : {} {}",
            r.owner_after_regrant.0,
            short(&r.owner_after_regrant.1, &lab.schema)
        );
        println!(
            "chủ bảng tắt trigger       : UPDATE được {} dòng",
            r.owner_rows_after_disable
        );
        println!(
            "dòng audit trước/sau DDL   : {} / {}",
            r.audit_rows_before_after.0, r.audit_rows_before_after.1
        );
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    println!("== 2. Hash chain và audit_log ==");
    with_lab(async move |lab| {
        let r = chain_basics(&lab).await?;
        for p in &r.posted {
            println!(
                "{} seq={} entry_hash={}…",
                p.display_id(),
                p.seq.unwrap_or(0),
                &hex::encode(p.entry_hash.unwrap_or_default())[..16]
            );
        }
        println!("kiểm chuỗi                 : {}", fault_text(&r.report));
        println!(
            "ghi lại cùng khoá          : seq={:?}, dòng chuỗi/audit = {:?}",
            r.replay_seq, r.rows_after_replay
        );
        println!(
            "audit của JE-000003        : actor={} approved_by={:?} reason={}",
            r.refund_audit.0, r.refund_audit.1, r.refund_audit.2
        );
        println!(
            "người tạo tự duyệt         : SQLSTATE {}",
            r.self_approval_sqlstate
        );
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    println!("== 3. Cuộc đua ở điểm nối chuỗi ==");
    with_lab(async move |lab| {
        let r = naive_fork(&lab).await?;
        println!(
            "không khoá, không ràng buộc: {} bên báo thành công, seq={:?}, cùng prev_hash={}",
            r.succeeded, r.seqs, r.same_prev
        );
        println!("  verify_chain             : {:?}", r.fault);
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;
    with_lab(async move |lab| {
        let r = unique_backstop(&lab).await?;
        println!(
            "không khoá, bảng thật      : {} thành công, lỗi {:?}, bút toán còn lại {}",
            r.succeeded, r.failed_sqlstates, r.entries_in_ledger
        );
        println!("  verify_chain             : {}", fault_text(&r.report));
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;
    with_lab(async move |lab| {
        let r = locked_chain(&lab).await?;
        println!(
            "advisory lock, 8 bên       : {} thành công, seq={:?}",
            r.succeeded, r.seqs
        );
        println!("  verify_chain             : {}", fault_text(&r.report));
        println!(
            "  xin khoá khi có bên giữ  : {}; sau khi bên đó kết thúc: {}",
            r.try_lock_while_held, r.try_lock_after_release
        );
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    println!("== 4. Phát hiện dòng bị sửa trực tiếp ==");
    with_lab(async move |lab| {
        let r = tamper_detect(&lab).await?;
        println!("nguyên vẹn                 : {}", fault_text(&r.clean));
        println!(
            "superuser UPDATE, trigger bật: {} {}",
            r.plain_update_by_superuser.0,
            short(&r.plain_update_by_superuser.1, &lab.schema)
        );
        println!(
            "sửa bút toán 4, tắt trigger: {}",
            fault_text(&r.after_entry_4)
        );
        println!(
            "sửa thêm bút toán 2        : {}",
            fault_text(&r.after_entry_2)
        );
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;
    with_lab(async move |lab| {
        let r = structural_faults(&lab).await?;
        println!("bút toán lách chuỗi        : {}", fault_text(&r.ghost));
        println!(
            "xoá dòng chuỗi thứ 3       : {}",
            fault_text(&r.deleted_row)
        );
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    println!("== 5. Neo ra ngoài ==");
    let r = anchor_demo().await?;
    println!("điểm neo (seq:hash)        : {}", r.anchor_line);
    println!("neo, dữ liệu sạch          : {:?}", r.clean_check);
    println!(
        "tính lại cả chuỗi          : {}",
        fault_text(&r.after_rewrite)
    );
    println!("  đối chiếu neo            : {:?}", r.rewrite_check);
    println!(
        "cắt đuôi chuỗi             : {}",
        fault_text(&r.after_truncate)
    );
    println!("  đối chiếu neo            : {:?}", r.truncate_check);
    if verify {
        r.verify()?;
    }

    println!("== 6. Phân quyền §11.2 bằng role thật ==");
    let schema = with_lab_value(move |lab| async move {
        let m = permission_matrix(&lab).await?;
        print!("{:<10}", "");
        for (label, _) in PROBES {
            print!(" {label:<17}");
        }
        println!();
        for (role, row) in &m {
            print!("{:<10}", role.suffix());
            for o in row {
                let cell = match o {
                    Outcome::Allowed => "ok".to_string(),
                    Outcome::Denied(c) => format!("từ chối {c}"),
                };
                print!(" {cell:<17}");
            }
            println!();
        }
        if verify {
            verify_matrix(&m)?;
        }
        Ok(lab.schema.clone())
    })
    .await?;
    // Sau khi dọn: schema và role của ca phải biến mất.
    let gone = !schema_exists(&schema).await? && leftover_roles(&schema).await? == 0;
    println!("dọn cuối ca                : schema và 5 role đã xoá = {gone}");
    if verify {
        anyhow::ensure!(gone, "còn schema hoặc role sót lại sau ca {schema}");
    }

    println!("== 7. PII tối thiểu và mask trong log ==");
    with_lab(async move |lab| {
        let r = pii(&lab).await?;
        for line in r.log.lines() {
            println!("log: {line}");
        }
        println!("chuỗi nhạy cảm lọt vào log : {:?}", r.leaked_in_log);
        println!(
            "chuỗi nhạy cảm lọt vào bảng: {:?} (tìm thấy mã khách: {})",
            r.leaked_in_db, r.id_found_in_db
        );
        println!(
            "đối chứng derive(Debug)    : bộ quét bắt được {:?}",
            r.leaky_control
        );
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;
    Ok(())
}

async fn run_measure() -> Result<()> {
    // Máy dùng chung đang tải nặng nên mỗi ô đo 3 lần và lấy lần nhanh nhất (ít bị nhiễu nhất).
    const N: usize = 100;
    println!(
        "== Đo ghi {N} bút toán, tốt nhất trong 3 lần (số đo trên máy viết bài, không phải SLA) =="
    );
    for (label, mode, workers) in [
        ("không chuỗi, 1 task", LinkMode::Skip, 1),
        ("không chuỗi, 8 task", LinkMode::Skip, 8),
        ("có chuỗi + khoá, 1 task", LinkMode::Locked, 1),
        ("có chuỗi + khoá, 8 task", LinkMode::Locked, 8),
    ] {
        let mut best = f64::MAX;
        for _ in 0..3 {
            let secs = with_lab_value(move |lab| async move {
                Ok(measure(&lab, mode, workers, N).await?.as_secs_f64())
            })
            .await?;
            best = best.min(secs);
        }
        println!(
            "{label:<26}: {:>7.1} bút toán/s ({best:.2} s)",
            N as f64 / best
        );
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let verify = args.iter().any(|a| a == "--verify");
    let result = if args.iter().any(|a| a == "--measure") {
        run_measure().await
    } else {
        run(verify).await
    };
    match result {
        Ok(()) => {
            if verify {
                println!("VERIFY OK");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("LỖI: {e:#}");
            ExitCode::FAILURE
        }
    }
}
