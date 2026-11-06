//! Lock phân tán có TTL: `SET key token NX PX ttl` để lấy, Lua so token để nhả.

use anyhow::Result;
use redis::aio::ConnectionManager;
use redis::{ExistenceCheck, Script, SetExpiry, SetOptions};
use uuid::Uuid;

// Chỉ xoá khi token còn là của mình; nếu lock đã hết hạn và người khác giữ thì không đụng vào.
const RELEASE: &str = r"
if redis.call('GET', KEYS[1]) == ARGV[1] then
  return redis.call('DEL', KEYS[1])
else
  return 0
end
";

/// Lock đang giữ. Token ngẫu nhiên chứng minh quyền nhả.
#[derive(Debug)]
pub struct Lock {
    key: String,
    token: String,
}

impl Lock {
    /// Thử lấy lock; `None` nếu người khác đang giữ. Hết `ttl_ms` thì Redis tự xoá key.
    pub async fn acquire(
        conn: &mut ConnectionManager,
        key: &str,
        ttl_ms: u64,
    ) -> Result<Option<Lock>> {
        let token = Uuid::new_v4().to_string();
        let options = SetOptions::default()
            .conditional_set(ExistenceCheck::NX)
            .with_expiration(SetExpiry::PX(ttl_ms));
        let reply: Option<String> = redis::cmd("SET")
            .arg(key)
            .arg(&token)
            .arg(options)
            .query_async(conn)
            .await?;
        Ok(reply.map(|_| Lock {
            key: key.to_string(),
            token,
        }))
    }

    /// Nhả lock; `false` nghĩa là lock không còn là của mình (đã hết hạn).
    pub async fn release(self, conn: &mut ConnectionManager) -> Result<bool> {
        let deleted: i64 = Script::new(RELEASE)
            .key(&self.key)
            .arg(&self.token)
            .invoke_async(conn)
            .await?;
        Ok(deleted == 1)
    }
}
