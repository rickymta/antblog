//! Mỗi test dựng một `Lab` riêng (schema và tiền tố key riêng) nên chạy song song được.
//! Thiếu `TOI_NOI_PG_URL`/`TOI_NOI_REDIS` hoặc không kết nối được thì test FAIL, không bỏ qua.

use anyhow::Result;
use ledger_28_redis_cache::cache::Source;
use ledger_28_redis_cache::checks;
use ledger_28_redis_cache::lab::Lab;
use ledger_28_redis_cache::scenarios;

/// Chạy `body` rồi luôn dọn schema và key, kể cả khi `body` lỗi.
async fn with_lab(body: impl AsyncFnOnce(&Lab) -> Result<()>) -> Result<()> {
    let lab = Lab::start().await?;
    let result = body(&lab).await;
    lab.finish().await?;
    result
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cache_aside_stale_windows_and_ttl() -> Result<()> {
    with_lab(async |lab| {
        let report = scenarios::cache_scenario(lab).await?;
        checks::check_cache(&report)?;
        // Cửa sổ B: `get` vẫn tin cache cho tới khi đối chiếu hoặc hết TTL.
        assert_eq!(report.race_get.source, Source::Cache);
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn token_bucket_is_atomic_under_100_concurrent_requests() -> Result<()> {
    with_lab(async |lab| {
        let report = scenarios::rate_limit_scenario(lab).await?;
        checks::check_rate(&report)
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn expired_lock_allows_two_holders_but_ledger_stays_correct() -> Result<()> {
    with_lab(async |lab| {
        let report = scenarios::lock_scenario(lab).await?;
        checks::check_lock(&report)
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn pipeline_writes_every_key() -> Result<()> {
    with_lab(async |lab| {
        let report = scenarios::pipeline_scenario(lab).await?;
        checks::check_pipeline(&report)
    })
    .await
}
