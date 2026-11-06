//! Kịch bản dùng chung cho `main`, `--verify` và test: chạy service với subscriber ghi vào bộ đệm.

use tracing::info;

use crate::service::{
    CaptureRequest, LedgerService, Outcome, SpanPropagation, audit_leaky, audit_safe,
    hold_enter_guard_across_await, instrument_the_future,
};
use crate::telemetry::{LogBuffer, json_subscriber};
use ledger_core::LedgerError;

pub const PHONE: &str = "0900000001";
pub const CARD: &str = "4111111111111111";

pub fn request(request_id: &str, trip_id: &str, km: u32) -> CaptureRequest {
    CaptureRequest {
        request_id: request_id.to_string(),
        trip_id: trip_id.to_string(),
        km,
        customer_phone: PHONE.to_string(),
        card_number: CARD.to_string(),
    }
}

/// Ba request liên tiếp: ghi mới, replay cùng key, rồi cùng key nhưng khác nội dung (xung đột).
pub async fn three_requests(service: &LedgerService) -> Vec<Result<Outcome, LedgerError>> {
    vec![
        service.handle(request("req-0001", "TN-0001", 5)).await,
        service.handle(request("req-0002", "TN-0001", 5)).await,
        service.handle(request("req-0003", "TN-0001", 6)).await,
    ]
}

/// Chạy `body` trên runtime một luồng với subscriber JSON ghi vào bộ đệm.
/// Một luồng + `set_default` (theo luồng) đủ để `tokio::spawn` vẫn thấy subscriber này.
pub fn with_buffer<F: std::future::Future<Output = ()>>(filter: &str, body: F) -> LogBuffer {
    let buffer = LogBuffer::default();
    let subscriber = json_subscriber(filter, buffer.clone()).expect("filter hợp lệ");
    let _guard = tracing::subscriber::set_default(subscriber);
    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("dựng runtime")
        .block_on(body);
    buffer
}

pub fn capture_flow(filter: &str, propagation: SpanPropagation) -> LogBuffer {
    with_buffer(filter, async {
        let service = LedgerService::new(propagation);
        three_requests(&service).await;
    })
}

/// Giữ guard `span.enter()` qua `.await`, rồi một việc không liên quan chạy xen vào.
pub fn enter_guard_bug() -> LogBuffer {
    with_buffer("info", async {
        tokio::join!(hold_enter_guard_across_await("req-A"), async {
            info!("việc không liên quan");
        });
    })
}

/// Cùng kịch bản nhưng dùng `.instrument(span)`.
pub fn instrument_fix() -> LogBuffer {
    with_buffer("info", async {
        tokio::join!(instrument_the_future("req-A"), async {
            info!("việc không liên quan");
        });
    })
}

/// `leaky = true` dùng `#[instrument]` trần.
pub fn audit(leaky: bool) -> LogBuffer {
    with_buffer("info", async move {
        let req = request("req-0001", "TN-0001", 5);
        if leaky {
            audit_leaky(&req);
        } else {
            audit_safe(&req);
        }
    })
}
