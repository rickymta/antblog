//! Chạy các thí nghiệm của bài 38 và in kết quả. `--verify` kiểm thêm các giá trị bài viết nêu.

use std::process::ExitCode;

use anyhow::Result;
use ledger_38_webhook_hmac::harness::{Harness, key_a, key_b};
use ledger_38_webhook_hmac::lab::with_lab;
use ledger_38_webhook_hmac::scenarios::{
    arrival_order, compare_model, key_rotation, psp_retries_on_5xx, replay_and_duplicates,
    signature_on_raw_bytes,
};
use ledger_38_webhook_hmac::signature::KeyRing;

async fn run(verify: bool) -> Result<()> {
    with_lab(async move |lab| {
        let h = Harness::start(lab.pool.clone(), KeyRing::new(vec![key_a()])).await?;
        let r = signature_on_raw_bytes(&h).await?;
        println!("== 1. Chữ ký trên byte thô (server nghe 127.0.0.1, cổng do hệ điều hành cấp) ==");
        println!("webhook hợp lệ        : {} {}", r.happy.0, r.happy.1);
        println!(
            "bút toán / số dư      : {} / khách {}, nền tảng {}, tài xế {}",
            r.entries, r.balances.0, r.balances.1, r.balances.2
        );
        println!(
            "serialize lại JSON    : {} byte so với {} byte gốc, giống từng byte: {}",
            r.lengths.1, r.lengths.0, r.reserialized_same
        );
        for (name, status) in &r.rejections {
            println!("giả mạo: {name:<44} -> {status}");
        }
        println!(
            "sau các lần từ chối   : {} webhook, {} bút toán",
            r.after_rejections.0, r.after_rejections.1
        );
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    with_lab(async move |lab| {
        let h = Harness::start(lab.pool.clone(), KeyRing::new(vec![key_a()])).await?;
        let r = replay_and_duplicates(&h).await?;
        println!("== 2. Replay và trùng (đồng hồ cố định) ==");
        println!("lần đầu               : {} {}", r.first.0, r.first.1);
        println!(
            "gửi lại sau 120 giây  : {} {}",
            r.inside_window.0, r.inside_window.1
        );
        println!(
            "gửi lại sau 301 giây  : {} {}",
            r.outside_window.0, r.outside_window.1
        );
        println!(
            "t ở tương lai +400 giây: {} {}",
            r.future_timestamp.0, r.future_timestamp.1
        );
        println!(
            "VíXanh ký lại, cùng body: {} {}",
            r.legit_retry.0, r.legit_retry.1
        );
        println!("bút toán              : {}", r.entries);
        println!(
            "8 bản cùng lúc        : {} applied, {} duplicate; bút toán {}",
            r.concurrent.0, r.concurrent.1, r.entries_final
        );
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    with_lab(async move |lab| {
        let h = Harness::start(lab.pool.clone(), KeyRing::new(vec![key_a()])).await?;
        let r = arrival_order(&h).await?;
        println!("== 3. Webhook đến trước/sau event nội bộ ==");
        println!("webhook trước event   : {} {}", r.before.0, r.before.1);
        println!("khi đang chờ          : {} bút toán, {} pending", r.while_waiting.0, r.while_waiting.1);
        println!("event đến sau         : {:?}", r.then_event);
        println!("sau event             : {} bút toán, {} pending", r.after_event.0, r.after_event.1);
        println!("event giao lặp        : nhặt lại {} webhook", r.duplicate_event_picked);
        println!("event trước, webhook sau: {} {}", r.after.0, r.after.1);
        println!("số tiền lệch          : {} {}, bút toán thêm {}", r.mismatch.0, r.mismatch.1, r.mismatch.2);
        println!("hai external_ref/chuyến: {} {}, bút toán thêm {}", r.double_capture.0, r.double_capture.1, r.double_capture.2);
        println!(
            "đua {} chuyến          : còn pending {}, applied {} (tổng gồm 3 chuyến trên); trả 202 ở {} chuyến (đổi theo lịch chạy)",
            r.race.0, r.race.1, r.race.2, r.race.3
        );
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    with_lab(async move |lab| {
        let h = Harness::start(lab.pool.clone(), KeyRing::new(vec![key_a()])).await?;
        let r = key_rotation(&h).await?;
        println!(
            "== 4. Xoay vòng khóa (khóa lab: {} và {}) ==",
            key_a().kid,
            key_b().kid
        );
        for (name, status) in &r.steps {
            println!("{name:<40} -> {status}");
        }
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    with_lab(async move |lab| {
        let h = Harness::start(lab.pool.clone(), KeyRing::new(vec![key_a()])).await?;
        let r = psp_retries_on_5xx(&h).await?;
        println!("== 5. VíXanh thử lại khi gặp 5xx ==");
        println!("các lần thử           : {:?}", r.attempts);
        println!(
            "webhook sau lần lỗi 1 : {} dòng (transaction đã rollback)",
            r.rows_after_first_failure
        );
        println!(
            "cuối cùng             : {} applied, {} bút toán",
            r.finally.0, r.finally.1
        );
        if verify {
            r.verify()?;
        }
        Ok(())
    })
    .await?;

    let c = compare_model();
    println!("== 6. So sánh thẻ: == dừng sớm, verify_slice thì không ==");
    for (good, steps, accepted) in &c.rows {
        println!(
            "đoán đúng {good:>2} byte đầu    : == phải xét {steps:>2} byte; verify_slice nhận: {accepted}"
        );
    }
    println!("(đếm byte của mô hình ==, không phải số đo thời gian)");
    if verify {
        c.verify()?;
        println!("VERIFY OK");
    }
    Ok(())
}

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
