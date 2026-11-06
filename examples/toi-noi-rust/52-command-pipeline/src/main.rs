//! Chạy các thí nghiệm của bài 52 và in kết quả. `--verify` kiểm thêm các giá trị bài viết nêu.

mod counting;
mod measure;

use std::process::ExitCode;

use anyhow::{Result, ensure};
use ledger_52_command_pipeline::lab::with_lab;
use ledger_52_command_pipeline::scenarios::{Outcome, describe, run_all};
use ledger_52_command_pipeline::{DynBus, Ping, VoidHold};

#[global_allocator]
static ALLOC: counting::Counting = counting::Counting;

fn print_trace<T>(label: &str, outcome: &Outcome<T>) {
    println!("-- {label}: {}", describe(&outcome.result));
    for line in &outcome.trace {
        println!("   {line}");
    }
    println!(
        "   (DB: {} bút toán, {} dòng outbox)",
        outcome.entries, outcome.outbox
    );
}

async fn run(verify: bool) -> Result<()> {
    let report = with_lab(async move |lab| {
        let report = run_all(&lab).await?;
        if verify {
            report.verify()?;
        }
        Ok(report)
    })
    .await?;

    println!("== 1. Thứ tự lớp: validate -> trace -> transaction -> handler ==");
    print_trace("thu cước TN-0001", &report.happy);
    print_trace("gửi lại đúng command đó", &report.replay);

    println!("== 2. Lỗi ở từng lớp ==");
    print_trace("cước 0, validate ngoài cùng", &report.invalid);
    print_trace("cước 0, trace ngoài cùng", &report.invalid_trace_first);
    print_trace("khách CU-404 (FK trong handler)", &report.handler_fail);
    print_trace("bút toán lệch (lỗi lúc COMMIT)", &report.commit_fail);

    println!("== 3. Sự kiện không đi trước commit ==");
    print_trace("pipeline phát sự kiện trước commit", &report.naive);
    println!(
        "   sự kiện đã phát ra ngoài: {} (entryId {})",
        report.naive_sink.len(),
        match report.naive_sink.first() {
            Some(ledger_52_command_pipeline::events::DomainEvent::EntryPosted(e)) =>
                e.entry_id.as_str(),
            None => "-",
        }
    );
    println!("   outbox cuối cùng: {} dòng", report.outbox.len());
    for row in &report.outbox {
        println!(
            "   {} key={} published={}",
            row.topic, row.key, row.published
        );
        println!("   {}", serde_json::to_string(&row.payload)?);
    }

    println!("== 4. Thiếu handler: lúc biên dịch và lúc chạy ==");
    let mut bus = DynBus::new();
    bus.register::<Ping, _, _>(|cmd: Ping| async move {
        Ok::<u32, ledger_52_command_pipeline::AppError>(cmd.n + 1)
    });
    let missing = bus.send(VoidHold { hold_id: 7 }).await;
    println!(
        "DynBus.send(VoidHold) biên dịch được, chạy ra: {}",
        match &missing {
            Ok(()) => "ok".to_string(),
            Err(e) => e.to_string(),
        }
    );
    println!("pipeline generic: cùng lời gọi là E0277, xem doc test ở src/lib.rs");

    println!("== 5. Chi phí (số đếm, rustc 1.98.1) ==");
    println!(
        "future của CapturePayment: pipeline {} byte | hàm thẳng capture_direct {} byte",
        report.happy.future_bytes, report.direct_future_bytes
    );
    let m = measure::measure();
    println!(
        "size_of future: Echo {} | Validated<Echo> {} | Validated<Traced<Echo>> {} | DynBus::send {} | Pin<Box<dyn Future>> {}",
        m.echo, m.validated, m.validated_traced, m.dyn_send, m.pin_box
    );
    println!(
        "cấp phát trong {} lần gọi: generic {} | DynBus {}",
        m.calls, m.generic_allocs, m.dyn_allocs
    );

    if verify {
        ensure!(missing.is_err(), "DynBus phải báo NoHandler lúc chạy");
        ensure!(
            m.pin_box == 2 * std::mem::size_of::<usize>(),
            "Pin<Box<dyn>> là con trỏ béo"
        );
        ensure!(
            m.echo <= m.validated && m.validated <= m.validated_traced,
            "future lớn dần theo lớp"
        );
        ensure!(
            m.generic_allocs == 0,
            "đường generic không cấp phát: {}",
            m.generic_allocs
        );
        ensure!(
            m.dyn_allocs == m.calls,
            "DynBus: đúng một Box mỗi lần gọi: {}",
            m.dyn_allocs
        );
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
