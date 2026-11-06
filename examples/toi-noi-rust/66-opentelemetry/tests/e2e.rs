//! Một trace_id đi qua gRPC thật và Kafka thật (cần `TOI_NOI_KAFKA`), exporter in-memory.
//! Mỗi file trong `tests/` là một tiến trình riêng, nên được đặt subscriber toàn cục của nó.

use ledger_66_opentelemetry::checks::run_stack_suite;
use ledger_66_opentelemetry::stack::Stack;
use ledger_66_opentelemetry::telemetry::shared_in_memory;
use opentelemetry::metrics::MeterProvider as _;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn one_trace_id_from_client_through_grpc_and_kafka() {
    let t = shared_in_memory();
    let stack = Stack::start(&t.meter_provider.meter("ledger"))
        .await
        .expect("dựng stack (cần Kafka đang chạy)");
    let run = run_stack_suite(t, &stack).await;
    // Dọn trước, kiểm kết quả sau: không để lại topic khi kịch bản lỗi.
    stack.stop().await.expect("dọn stack");
    run.expect("bộ kịch bản");
}
