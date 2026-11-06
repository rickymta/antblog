//! Các kịch bản chạy trong tiến trình test. Mỗi test có schema, topic và group riêng nên chạy song song được.

use anyhow::Result;
use ledger_39_graceful_shutdown::checks::{
    check_backpressure, check_graceful, check_http_drain, check_poll, check_probes, check_timeout,
};
use ledger_39_graceful_shutdown::scenarios;

#[tokio::test(flavor = "multi_thread")]
async fn graceful_shutdown_drains_commits_and_resumes_without_duplicates() -> Result<()> {
    check_graceful(&scenarios::graceful().await?)
}

#[tokio::test(flavor = "multi_thread")]
async fn in_flight_http_request_finishes_after_shutdown_begins() -> Result<()> {
    check_http_drain(&scenarios::http_drain().await?)
}

#[tokio::test(flavor = "multi_thread")]
async fn drain_timeout_rolls_back_the_open_transaction() -> Result<()> {
    check_timeout(&scenarios::timeout().await?)
}

#[tokio::test(flavor = "multi_thread")]
async fn bounded_queue_pauses_consumer_and_resumes() -> Result<()> {
    check_backpressure(&scenarios::backpressure().await?)
}

#[tokio::test(flavor = "multi_thread")]
async fn pause_with_polling_avoids_max_poll_interval_but_not_polling_does_not() -> Result<()> {
    check_poll(&scenarios::poll_interval().await?)
}

#[tokio::test(flavor = "multi_thread")]
async fn healthz_stays_up_while_readyz_follows_dependencies() -> Result<()> {
    check_probes(&scenarios::probes().await?)
}
