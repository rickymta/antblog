//! Dựng server thật trên `127.0.0.1:0`, đồng hồ cố định và các hàm gọi/ký dùng chung cho lab và test.

use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use anyhow::Result;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::app::{AppState, router};
use crate::httpc::{self, HttpResponse};
use crate::signature::{FixedClock, KeyRing, SIGNATURE_HEADER, SecretKey, header_value};
use crate::store::{Outcome, TripCompleted, on_trip_completed};

/// 2026-11-06T03:00:00Z. Đồng hồ lab bắt đầu ở đây và chỉ nhích khi lab gọi `advance`.
pub const T0: i64 = 1_793_934_000;

/// Khóa lab A. Giá trị lab, không phải bí mật thật.
pub fn key_a() -> SecretKey {
    SecretKey::new("lab-2026-11-a", "lab-only-secret-a")
}

/// Khóa lab B, dùng cho xoay vòng. Giá trị lab.
pub fn key_b() -> SecretKey {
    SecretKey::new("lab-2026-11-b", "lab-only-secret-b")
}

/// Body webhook đúng thứ tự trường như VíXanh (giả định) gửi, gọn, không khoảng trắng.
pub fn captured_body(external_ref: &str, trip_id: &str, amount_dong: i64) -> Vec<u8> {
    format!(
        r#"{{"eventId":"evt_{external_ref}","type":"payment.captured","externalRef":"{external_ref}","tripId":"{trip_id}","amountDong":{amount_dong},"occurredAt":"2026-11-06T03:00:00Z"}}"#
    )
    .into_bytes()
}

/// Server axum thật + đồng hồ cố định + danh sách khóa sửa được lúc chạy.
pub struct Harness {
    /// Pool của ca.
    pub pool: PgPool,
    /// Địa chỉ server đang nghe (cổng do hệ điều hành cấp).
    pub addr: SocketAddr,
    /// Đồng hồ cố định.
    pub clock: Arc<FixedClock>,
    /// Khóa hợp lệ của server.
    pub keys: Arc<RwLock<KeyRing>>,
    server: JoinHandle<()>,
}

impl Harness {
    /// Bind `127.0.0.1:0`, đọc cổng thật rồi chạy server nền.
    ///
    /// # Errors
    /// Không bind được cổng.
    pub async fn start(pool: PgPool, ring: KeyRing) -> Result<Harness> {
        let clock = Arc::new(FixedClock::new(T0));
        let keys = Arc::new(RwLock::new(ring));
        let state = AppState {
            pool: pool.clone(),
            keys: Arc::clone(&keys),
            clock: clock.clone(),
        };
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let server = tokio::spawn(async move {
            // Server chạy tới khi `Harness` bị drop (abort); lỗi serve ở đây chỉ có thể là lỗi socket.
            let _ = axum::serve(listener, router(state)).await;
        });
        Ok(Harness {
            pool,
            addr,
            clock,
            keys,
            server,
        })
    }

    /// Header chữ ký hợp lệ tại thời điểm đồng hồ hiện tại.
    pub fn sign_now(&self, key: &SecretKey, body: &[u8]) -> String {
        use crate::signature::Clock;
        let t = self.clock.now_epoch();
        header_value(t, &[key.sign(t, body)])
    }

    /// Gửi body với header (hoặc không có header).
    ///
    /// # Errors
    /// Lỗi kết nối.
    pub async fn send(&self, body: &[u8], header: Option<&str>) -> Result<HttpResponse> {
        let headers: Vec<(&str, &str)> =
            header.map(|h| (SIGNATURE_HEADER, h)).into_iter().collect();
        httpc::post(self.addr, "/webhooks/vixanh", &headers, body).await
    }

    /// Ký bằng `key` ở thời điểm hiện tại rồi gửi.
    ///
    /// # Errors
    /// Lỗi kết nối.
    pub async fn send_signed(&self, key: &SecretKey, body: &[u8]) -> Result<HttpResponse> {
        let header = self.sign_now(key, body);
        self.send(body, Some(&header)).await
    }

    /// Giả lập consumer nhận `TripCompleted` cho khách `CU-001`, tài xế `DR-001`.
    ///
    /// # Errors
    /// JSON hoặc cơ sở dữ liệu lỗi.
    pub async fn trip_completed(&self, trip_id: &str, fare_dong: i64) -> Result<Vec<Outcome>> {
        let json = format!(
            r#"{{"schemaVersion":1,"tripId":"{trip_id}","customerId":"CU-001","driverId":"DR-001","fareDong":{fare_dong},"occurredAt":"2026-11-06T02:55:00Z"}}"#
        );
        let event: TripCompleted = serde_json::from_str(&json)?;
        debug_assert_eq!(event.schema_version, 1);
        on_trip_completed(&self.pool, &event).await
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.server.abort();
    }
}
