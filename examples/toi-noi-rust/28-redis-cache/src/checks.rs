//! Các giá trị kỳ vọng in trong bài; dùng chung cho `--verify` và `cargo test`.

use anyhow::{Result, ensure};

use crate::cache::Source;
use crate::scenarios::{CacheReport, LockReport, PipelineReport, RateReport};
use crate::store::PostOutcome;

pub fn check_cache(r: &CacheReport) -> Result<()> {
    ensure!(
        r.first.source == Source::Ledger && r.first.balance.dong() == 20_400,
        "đọc 1 phải miss, 20.400"
    );
    ensure!(
        r.second.source == Source::Cache && r.second.balance.dong() == 20_400,
        "đọc 2 phải hit"
    );
    ensure!(r.ledger_reads_after_hit == 1, "hit không được chạm sổ cái");
    ensure!(
        r.ttl_ms_after_fill > 0 && r.ttl_ms_after_fill <= 1_500,
        "key phải có TTL"
    );
    ensure!(
        r.after_invalidate.source == Source::Ledger && r.after_invalidate.balance.dong() == 33_600,
        "sau DEL phải thấy 33.600"
    );
    ensure!(
        r.window_cached.map(|m| m.dong()) == Some(33_600),
        "cửa sổ A: cache còn số cũ"
    );
    ensure!(
        r.window_ledger.dong() == 50_400,
        "cửa sổ A: sổ cái là 50.400"
    );
    ensure!(
        r.window_after_del.balance.dong() == 50_400,
        "sau DEL phải thấy 50.400"
    );
    ensure!(
        r.race_cached.map(|m| m.dong()) == Some(50_400) && r.race_ledger.dong() == 60_000,
        "cửa sổ B: cache cũ, sổ cái mới"
    );
    ensure!(
        r.race_get.source == Source::Cache && r.race_get.balance.dong() == 50_400,
        "get vẫn trả số cũ trong cửa sổ B"
    );
    ensure!(
        r.reconcile.drift && r.reconcile.ledger.dong() == 60_000,
        "đối chiếu phải phát hiện lệch"
    );
    ensure!(r.after_ttl_cached.is_none(), "hết TTL key phải biến mất");
    ensure!(
        r.after_ttl_get.source == Source::Ledger && r.after_ttl_get.balance.dong() == 60_000,
        "sau TTL đọc lại từ sổ cái"
    );
    Ok(())
}

pub fn check_rate(r: &RateReport) -> Result<()> {
    // Tất định nhờ barrier: mọi GET xong mới có SET, nên không yêu cầu nào thấy xô đã vơi.
    ensure!(
        r.naive_allowed == r.requests,
        "bản GET/SET phải cho lọt cả {} yêu cầu",
        r.requests
    );
    ensure!(
        r.lua_allowed >= r.capacity,
        "Lua phải cho đủ {} yêu cầu",
        r.capacity
    );
    // Sai số duy nhất là token hoàn lại trong lúc chạy: tối đa 1 token/giây.
    let refill_allowed = u32::try_from(r.lua_elapsed_ms / 1_000 + 1)?;
    ensure!(
        r.lua_allowed <= r.capacity + refill_allowed,
        "Lua cho lọt {} > {} + hoàn",
        r.lua_allowed,
        r.capacity
    );
    ensure!(
        r.small_bucket == vec![true, true, false],
        "bucket 2 token: T, T, F"
    );
    ensure!(r.refilled_after_wait, "sau 1,1 giây phải hoàn được 1 token");
    ensure!(r.bucket_ttl_ms > 0, "bucket phải có TTL");
    Ok(())
}

pub fn check_lock(r: &LockReport) -> Result<()> {
    ensure!(
        !r.second_try_while_held,
        "lock còn hạn thì người thứ hai không lấy được"
    );
    ensure!(r.max_holders == 2, "lock hết hạn: hai worker cùng chạy");
    ensure!(!r.stale_release_a, "A nhả sai token phải thất bại");
    ensure!(r.release_b, "B nhả đúng token phải thành công");
    let posted = r
        .outcomes
        .iter()
        .filter(|o| matches!(o, PostOutcome::Posted { .. }))
        .count();
    ensure!(
        posted == 1 && r.outcomes.len() == 2,
        "đúng một lần ghi mới, một lần phát lại"
    );
    ensure!(
        r.new_entries == 1 && r.driver_delta == 20_400,
        "sổ cái thêm đúng một bút toán, tài xế +20.400"
    );
    Ok(())
}

/// Chỉ kiểm dữ liệu pipeline đã ghi đủ. Thời gian phụ thuộc tải máy nên chỉ in, không kiểm.
pub fn check_pipeline(r: &PipelineReport) -> Result<()> {
    ensure!(r.pipelined_values.len() == r.commands, "thiếu giá trị");
    for (i, value) in r.pipelined_values.iter().enumerate() {
        ensure!(
            *value == Some(i64::try_from(i)?),
            "key {i} ghi sai: {value:?}"
        );
    }
    Ok(())
}
