//! `cargo run` in kết quả các kịch bản; `cargo run -- --verify` kiểm các giá trị kỳ vọng của bài.

use std::process::ExitCode;

use anyhow::Result;
use ledger_28_redis_cache::checks::{check_cache, check_lock, check_pipeline, check_rate};
use ledger_28_redis_cache::lab::Lab;
use ledger_28_redis_cache::scenarios::{
    CacheReport, LockReport, PipelineReport, RateReport, cache_scenario, lock_scenario,
    pipeline_scenario, rate_limit_scenario, source_name,
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

async fn run(verify: bool) -> Result<()> {
    let lab = Lab::start().await?;
    println!("schema {}, tiền tố key {}", lab.schema, lab.prefix);
    // Dù kịch bản lỗi hay không, vẫn dọn đúng schema và key của ca này.
    let result = scenarios(&lab, verify).await;
    let cleanup = lab.finish().await;
    result?;
    cleanup?;
    if verify {
        println!("VERIFY OK");
    }
    Ok(())
}

async fn scenarios(lab: &Lab, verify: bool) -> Result<()> {
    let cache = cache_scenario(lab).await?;
    print_cache(&cache);
    let rate = rate_limit_scenario(lab).await?;
    print_rate(&rate);
    let lock = lock_scenario(lab).await?;
    print_lock(&lock);
    let pipeline = pipeline_scenario(lab).await?;
    print_pipeline(&pipeline);
    if verify {
        check_cache(&cache)?;
        check_rate(&rate)?;
        check_lock(&lock)?;
        check_pipeline(&pipeline)?;
    }
    Ok(())
}

fn print_cache(r: &CacheReport) {
    println!("\n== Cache-aside số dư driver:DR-001 (TTL 1500 ms) ==");
    println!(
        "đọc 1: {} từ {}",
        r.first.balance,
        source_name(r.first.source)
    );
    println!(
        "đọc 2: {} từ {}",
        r.second.balance,
        source_name(r.second.source)
    );
    println!(
        "số lần đọc sổ cái sau 2 lần đọc: {}; TTL còn lại {} ms",
        r.ledger_reads_after_hit, r.ttl_ms_after_fill
    );
    println!(
        "sau ghi TN-0002 và DEL: {} từ {}",
        r.after_invalidate.balance,
        source_name(r.after_invalidate.source)
    );
    println!(
        "cửa sổ A (commit xong, chưa DEL): cache {:?}, sổ cái {}",
        r.window_cached.map(|m| m.dong()),
        r.window_ledger
    );
    println!(
        "  sau DEL: {} từ {}",
        r.window_after_del.balance,
        source_name(r.window_after_del.source)
    );
    println!(
        "cửa sổ B (nạp số cũ sau DEL): cache {:?}, sổ cái {}",
        r.race_cached.map(|m| m.dong()),
        r.race_ledger
    );
    println!(
        "  get trả {} từ {}",
        r.race_get.balance,
        source_name(r.race_get.source)
    );
    println!(
        "  đối chiếu: cache {:?}, sổ cái {}, lệch {}",
        r.reconcile.cached.map(|m| m.dong()),
        r.reconcile.ledger,
        r.reconcile.drift
    );
    println!(
        "  nạp lại số cũ rồi đợi quá TTL: cache {:?}, get trả {} từ {}",
        r.after_ttl_cached.map(|m| m.dong()),
        r.after_ttl_get.balance,
        source_name(r.after_ttl_get.source)
    );
}

fn print_rate(r: &RateReport) {
    println!(
        "\n== Rate limit: {} yêu cầu đồng thời, bucket {} token ==",
        r.requests, r.capacity
    );
    println!("GET rồi SET từ client: cho qua {}", r.naive_allowed);
    println!(
        "script Lua: cho qua {} trong {} ms",
        r.lua_allowed, r.lua_elapsed_ms
    );
    println!(
        "bucket 2 token, 3 lần liên tiếp: {:?}; sau 1,1 giây: {}",
        r.small_bucket, r.refilled_after_wait
    );
}

fn print_lock(r: &LockReport) {
    println!("\n== Lock có TTL ==");
    println!("B lấy lock khi A đang giữ: {}", r.second_try_while_held);
    println!("số worker cùng ở trong vùng găng tối đa: {}", r.max_holders);
    println!(
        "A nhả lock sau khi hết hạn: {}; B nhả: {}",
        r.stale_release_a, r.release_b
    );
    println!("hai lần ghi cùng key: {:?}", r.outcomes);
    println!(
        "bút toán mới trong sổ cái: {}; số dư driver:DR-001 tăng {} đồng",
        r.new_entries, r.driver_delta
    );
}

fn print_pipeline(r: &PipelineReport) {
    println!(
        "\n== Pipeline {} lệnh SET ... PX (trung vị 3 vòng, máy viết bài, không phải SLA) ==",
        r.commands
    );
    println!("một vòng PING: {:.2} ms", r.ping_ms);
    println!(
        "lần lượt: {:.1} ms; pipeline: {:.1} ms; nhanh hơn {:.1} lần",
        r.sequential_ms,
        r.pipelined_ms,
        r.sequential_ms / r.pipelined_ms
    );
}
