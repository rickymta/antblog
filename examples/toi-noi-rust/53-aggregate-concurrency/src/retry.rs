//! Retry có giới hạn và jitter cho lỗi tạm thời (xung đột version, 40001, 40P01).

use std::future::Future;
use std::time::Duration;

use crate::store::StoreError;

/// Chính sách thử lại: tối đa `max_attempts` lần (kể cả lần đầu), chờ ngẫu nhiên trong
/// `[0, min(cap, base * 2^(lần-1))]` ms giữa hai lần («full jitter»).
#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    /// Tổng số lần thử tối đa, kể cả lần đầu.
    pub max_attempts: u32,
    /// Cận trên của lần chờ đầu tiên, ms.
    pub base_ms: u64,
    /// Cận trên của mọi lần chờ, ms.
    pub cap_ms: u64,
}

impl RetryPolicy {
    /// Mặc định của lab: 6 lần, cận 2 ms nhân đôi, trần 50 ms.
    pub const fn lab() -> RetryPolicy {
        RetryPolicy {
            max_attempts: 6,
            base_ms: 2,
            cap_ms: 50,
        }
    }

    /// Cận trên của lần chờ sau lần thử thứ `attempt` (bắt đầu từ 1).
    pub fn ceiling_ms(&self, attempt: u32) -> u64 {
        let doubled = self
            .base_ms
            .saturating_mul(1u64 << attempt.saturating_sub(1).min(20));
        doubled.min(self.cap_ms)
    }
}

/// Bộ sinh số giả ngẫu nhiên (SplitMix64). Không cần mật mã: chỉ để các task không thử lại cùng nhịp.
#[derive(Debug, Clone)]
pub struct Jitter(u64);

impl Jitter {
    /// Mỗi task một `seed` khác nhau (ví dụ số thứ tự task).
    pub fn new(seed: u64) -> Jitter {
        Jitter(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Số trong `[0, max]`.
    pub fn up_to(&mut self, max: u64) -> u64 {
        self.next_u64() % (max + 1)
    }
}

/// Kết quả một thao tác có thử lại: số lần đã thử và kết quả cuối.
#[derive(Debug)]
pub struct Run<T> {
    /// Số lần đã thử (≥ 1), kể cả lần cuối thành công hay bị từ chối.
    pub attempts: u32,
    /// Kết quả của lần cuối.
    pub result: Result<T, StoreError>,
}

/// Chạy `op(lần_thử)` cho tới khi thành công, gặp lỗi không thử lại được, hoặc hết `max_attempts`.
/// `op` phải mở transaction MỚI mỗi lần: bản đọc của lần trước đã cũ.
pub async fn with_retry<T, F, Fut>(policy: RetryPolicy, mut jitter: Jitter, mut op: F) -> Run<T>
where
    F: FnMut(u32) -> Fut,
    Fut: Future<Output = Result<T, StoreError>>,
{
    let mut attempt = 1;
    loop {
        match op(attempt).await {
            Err(err) if err.is_retryable() => {
                if attempt >= policy.max_attempts {
                    return Run {
                        attempts: attempt,
                        result: Err(StoreError::RetriesExhausted { attempts: attempt }),
                    };
                }
                let wait = jitter.up_to(policy.ceiling_ms(attempt));
                tokio::time::sleep(Duration::from_millis(wait)).await;
                attempt += 1;
            }
            other => {
                return Run {
                    attempts: attempt,
                    result: other,
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ceiling_doubles_then_stops_at_cap() {
        let p = RetryPolicy::lab();
        let ceilings: Vec<u64> = (1..=8).map(|a| p.ceiling_ms(a)).collect();
        assert_eq!(ceilings, [2, 4, 8, 16, 32, 50, 50, 50]);
    }

    #[test]
    fn jitter_stays_in_range_and_depends_on_seed() {
        let mut a = Jitter::new(1);
        let mut b = Jitter::new(2);
        let xs: Vec<u64> = (0..64).map(|_| a.up_to(50)).collect();
        let ys: Vec<u64> = (0..64).map(|_| b.up_to(50)).collect();
        assert!(xs.iter().chain(&ys).all(|v| *v <= 50));
        assert_ne!(xs, ys);
        assert_eq!(
            xs,
            (0..64)
                .map({
                    let mut again = Jitter::new(1);
                    move |_| again.up_to(50)
                })
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn gives_up_after_max_attempts_and_does_not_retry_business_errors() {
        let policy = RetryPolicy {
            max_attempts: 3,
            base_ms: 0,
            cap_ms: 0,
        };
        let run: Run<()> =
            with_retry(policy, Jitter::new(7), async |_| Err(StoreError::Conflict)).await;
        assert_eq!(run.attempts, 3);
        assert!(matches!(
            run.result,
            Err(StoreError::RetriesExhausted { attempts: 3 })
        ));

        let run: Run<()> = with_retry(policy, Jitter::new(7), async |_| {
            Err(StoreError::KeyReused("k".into()))
        })
        .await;
        assert_eq!(run.attempts, 1, "lỗi nghiệp vụ không được thử lại");
    }
}
