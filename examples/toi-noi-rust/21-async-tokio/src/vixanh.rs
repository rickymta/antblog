//! VíXanh giả: một cổng thanh toán ở xa. Mọi độ trễ dùng `tokio::time::sleep`,
//! nên test có thể dùng đồng hồ ảo và cho kết quả tất định.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ledger_core::Money;
use tokio::sync::oneshot;
use tokio::time::sleep;

/// Mã giữ tiền (authorization) do VíXanh trả về.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthId(String);

impl AuthId {
    /// Mã giữ tiền của chuyến, dùng khi bước authorize đã xong từ trước.
    pub fn for_trip(trip_id: &str) -> AuthId {
        AuthId(format!("VX-AUTH-{trip_id}"))
    }

    /// Chuỗi mã, ví dụ `VX-AUTH-TN-0001`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Biên lai thu tiền (capture).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    /// Mã thu tiền, ví dụ `VX-CAP-0001`.
    pub capture_id: String,
    /// Mã giữ tiền mà lần thu này dùng.
    pub auth_id: String,
    /// Idempotency key khách gửi lên.
    pub key: String,
    /// Số tiền đã thu.
    pub amount: Money,
    /// `true` nếu VíXanh nhận ra key đã xử lý và trả lại biên lai cũ, không trừ thêm.
    pub replayed: bool,
}

/// Lỗi từ VíXanh, hoặc từ đường truyền tới VíXanh.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PspError {
    /// VíXanh từ chối giao dịch.
    #[error("VíXanh từ chối: {0}")]
    Declined(String),
    /// VíXanh tạm thời không phục vụ.
    #[error("VíXanh tạm thời không phục vụ")]
    Unavailable,
    /// Mất kết nối trước khi nhận phản hồi.
    #[error("mất kết nối trước khi nhận phản hồi")]
    ConnectionLost,
}

// Kịch bản của một chuyến: authorize mất bao lâu, có lỗi không.
#[derive(Debug, Clone)]
struct Script {
    latency: Duration,
    failure: Option<PspError>,
}

// Hai mốc của một lần capture: lúc VíXanh trừ tiền thật và lúc phản hồi tới khách.
#[derive(Debug, Clone, Copy)]
struct Timing {
    effect_after: Duration,
    reply_after: Duration,
}

struct Inner {
    scripts: HashMap<String, Script>,
    default_latency: Duration,
    slow_captures: u32,
    slow: Timing,
    fast: Timing,
    authorize_calls: AtomicU32,
    capture_calls: AtomicU32,
    // Sổ phía VíXanh: key -> biên lai. Khóa chỉ giữ trong lúc đọc/ghi, không bao giờ qua `.await`.
    charges: Mutex<HashMap<String, Receipt>>,
}

/// Cổng thanh toán giả. Clone rẻ (chia sẻ một `Arc`), nên đưa được vào `tokio::spawn`.
#[derive(Clone)]
pub struct Vixanh {
    inner: Arc<Inner>,
}

/// Bộ dựng cấu hình cho [`Vixanh`].
pub struct VixanhBuilder {
    scripts: HashMap<String, Script>,
    slow_captures: u32,
}

const fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

impl VixanhBuilder {
    /// Authorize của chuyến `trip_id` mất `latency`.
    pub fn authorize_latency(mut self, trip_id: &str, latency: Duration) -> Self {
        self.scripts.insert(
            trip_id.to_string(),
            Script {
                latency,
                failure: None,
            },
        );
        self
    }

    /// Authorize của chuyến `trip_id` lỗi `failure` sau `latency`.
    pub fn authorize_failure(
        mut self,
        trip_id: &str,
        latency: Duration,
        failure: PspError,
    ) -> Self {
        self.scripts.insert(
            trip_id.to_string(),
            Script {
                latency,
                failure: Some(failure),
            },
        );
        self
    }

    /// `n` lần capture đầu tiên chậm (trừ tiền sau 100 ms, phản hồi sau 400 ms);
    /// các lần sau nhanh (20 ms và 50 ms).
    pub fn slow_captures(mut self, n: u32) -> Self {
        self.slow_captures = n;
        self
    }

    /// Dựng cổng thanh toán.
    pub fn build(self) -> Vixanh {
        Vixanh {
            inner: Arc::new(Inner {
                scripts: self.scripts,
                default_latency: ms(300),
                slow_captures: self.slow_captures,
                slow: Timing {
                    effect_after: ms(100),
                    reply_after: ms(400),
                },
                fast: Timing {
                    effect_after: ms(20),
                    reply_after: ms(50),
                },
                authorize_calls: AtomicU32::new(0),
                capture_calls: AtomicU32::new(0),
                charges: Mutex::new(HashMap::new()),
            }),
        }
    }
}

impl Vixanh {
    /// Bắt đầu cấu hình.
    pub fn builder() -> VixanhBuilder {
        VixanhBuilder {
            scripts: HashMap::new(),
            slow_captures: 0,
        }
    }

    /// Cổng thanh toán mặc định: authorize 300 ms, capture luôn nhanh.
    pub fn standard() -> Vixanh {
        Vixanh::builder().build()
    }

    /// Giữ tiền cho một chuyến. Không trừ tiền thật.
    ///
    /// # Errors
    /// Lỗi theo kịch bản của chuyến, hoặc `Declined` nếu số tiền không dương.
    pub async fn authorize(&self, trip_id: &str, amount: Money) -> Result<AuthId, PspError> {
        // Thân hàm chỉ chạy khi future được poll: tạo future mà không `.await` thì bộ đếm vẫn là 0.
        self.inner.authorize_calls.fetch_add(1, Ordering::SeqCst);
        let script = self.inner.scripts.get(trip_id).cloned().unwrap_or(Script {
            latency: self.inner.default_latency,
            failure: None,
        });
        sleep(script.latency).await;
        if let Some(failure) = script.failure {
            return Err(failure);
        }
        if amount <= Money::ZERO {
            return Err(PspError::Declined("số tiền không hợp lệ".to_string()));
        }
        Ok(AuthId(format!("VX-AUTH-{trip_id}")))
    }

    /// Thu tiền đã giữ. Phía VíXanh xử lý trong một task riêng: khách bỏ chờ (drop future)
    /// thì tiền vẫn bị trừ, chỉ có phản hồi là mất.
    ///
    /// Cùng `key` gọi lại thì VíXanh trả biên lai cũ (`replayed = true`), không trừ thêm.
    ///
    /// # Errors
    /// `ConnectionLost` nếu phía VíXanh không gửi được phản hồi.
    pub async fn capture(
        &self,
        auth: &AuthId,
        key: &str,
        amount: Money,
    ) -> Result<Receipt, PspError> {
        let n = self.inner.capture_calls.fetch_add(1, Ordering::SeqCst);
        let timing = if n < self.inner.slow_captures {
            self.inner.slow
        } else {
            self.inner.fast
        };
        let (reply_tx, reply_rx) = oneshot::channel();
        let server = self.clone();
        let (auth, key) = (auth.clone(), key.to_string());
        // «Máy chủ VíXanh»: chạy độc lập với future của khách.
        tokio::spawn(async move {
            sleep(timing.effect_after).await;
            let receipt = server.apply_charge(&auth, &key, amount);
            sleep(timing.reply_after - timing.effect_after).await;
            // Khách đã bỏ chờ thì `send` lỗi; máy chủ không quan tâm.
            let _ = reply_tx.send(receipt);
        });
        reply_rx.await.map_err(|_| PspError::ConnectionLost)
    }

    fn apply_charge(&self, auth: &AuthId, key: &str, amount: Money) -> Receipt {
        let mut charges = self.inner.charges.lock().expect("khóa không bị poison");
        if let Some(old) = charges.get(key) {
            return Receipt {
                replayed: true,
                ..old.clone()
            };
        }
        let receipt = Receipt {
            capture_id: format!("VX-CAP-{:04}", charges.len() + 1),
            auth_id: auth.as_str().to_string(),
            key: key.to_string(),
            amount,
            replayed: false,
        };
        charges.insert(key.to_string(), receipt.clone());
        receipt
    }

    /// Tổng tiền VíXanh đã thực sự trừ.
    pub fn charged_total(&self) -> Money {
        let charges = self.inner.charges.lock().expect("khóa không bị poison");
        Money::from_dong(charges.values().map(|r| r.amount.dong()).sum())
    }

    /// Số lần VíXanh đã trừ tiền (số key khác nhau).
    pub fn charge_count(&self) -> usize {
        self.inner
            .charges
            .lock()
            .expect("khóa không bị poison")
            .len()
    }

    /// Số lần `authorize` đã thực sự chạy.
    pub fn authorize_calls(&self) -> u32 {
        self.inner.authorize_calls.load(Ordering::SeqCst)
    }

    /// Số lần `capture` đã thực sự chạy.
    pub fn capture_calls(&self) -> u32 {
        self.inner.capture_calls.load(Ordering::SeqCst)
    }
}
