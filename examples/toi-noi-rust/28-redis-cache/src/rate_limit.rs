//! Token bucket trong Redis: một script Lua nên đọc, tính, ghi là một bước nguyên tử.

use anyhow::Result;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, RedisResult, Script, SetExpiry, SetOptions};
use tokio::sync::Barrier;

// KEYS[1] = bucket; ARGV = capacity, token/giây, chi phí mỗi yêu cầu.
// Giờ lấy từ chính Redis (TIME), không tin đồng hồ của client.
const TOKEN_BUCKET: &str = r"
local t = redis.call('TIME')
local now_ms = t[1] * 1000 + math.floor(t[2] / 1000)
local capacity = tonumber(ARGV[1])
local rate = tonumber(ARGV[2])
local cost = tonumber(ARGV[3])
local data = redis.call('HMGET', KEYS[1], 'tokens', 'ts')
local tokens = tonumber(data[1])
local ts = tonumber(data[2])
if tokens == nil then
  tokens = capacity
  ts = now_ms
end
tokens = math.min(capacity, tokens + math.max(0, now_ms - ts) * rate / 1000)
local allowed = 0
if tokens >= cost then
  tokens = tokens - cost
  allowed = 1
end
redis.call('HSET', KEYS[1], 'tokens', tokens, 'ts', now_ms)
redis.call('PEXPIRE', KEYS[1], math.ceil(capacity / rate * 1000) + 1000)
return {allowed, math.floor(tokens)}
";

/// Kết quả một lần xin phép.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decision {
    pub allowed: bool,
    pub tokens_left: i64,
}

/// Token bucket dùng chung cho nhiều tiến trình vì trạng thái nằm trong Redis.
pub struct TokenBucket {
    script: Script,
    capacity: u32,
    refill_per_sec: f64,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_per_sec: f64) -> TokenBucket {
        TokenBucket {
            script: Script::new(TOKEN_BUCKET),
            capacity,
            refill_per_sec,
        }
    }

    /// Xin một token cho `key`. Script chạy trọn vẹn trong Redis, không ai chen giữa.
    pub async fn try_acquire(&self, conn: &mut ConnectionManager, key: &str) -> Result<Decision> {
        let (allowed, tokens_left): (i64, i64) = self
            .script
            .key(key)
            .arg(self.capacity)
            .arg(self.refill_per_sec)
            .arg(1)
            .invoke_async(conn)
            .await?;
        Ok(Decision {
            allowed: allowed == 1,
            tokens_left,
        })
    }
}

/// Bản SAI để đối chiếu: GET rồi SET từ client. Hai yêu cầu cùng đọc `10` thì cùng ghi `9`.
/// `gate` ép mọi yêu cầu đọc xong mới có ai được ghi: thứ tự xấu nhất, để kết quả lặp lại được.
pub async fn naive_try_acquire(
    conn: &mut ConnectionManager,
    key: &str,
    capacity: i64,
    gate: &Barrier,
) -> Result<bool> {
    let read: RedisResult<Option<i64>> = conn.get(key).await;
    gate.wait().await; // đợi cả khi GET lỗi, để các yêu cầu khác không kẹt ở đây
    let tokens = read?.unwrap_or(capacity);
    if tokens < 1 {
        return Ok(false);
    }
    let options = SetOptions::default().with_expiration(SetExpiry::PX(60_000));
    let _: () = conn.set_options(key, tokens - 1, options).await?;
    Ok(true)
}
