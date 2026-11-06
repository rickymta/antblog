//! Cache-aside cho số dư: đọc cache trước, miss thì đọc sổ cái rồi nạp lại; có TTL.

use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;
use ledger_core::Money;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, SetExpiry, SetOptions};
use sqlx::PgPool;

use crate::store;

/// Số dư đến từ đâu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Cache,
    Ledger,
}

/// Một lần đọc số dư.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Read {
    pub balance: Money,
    pub source: Source,
}

/// Kết quả đối chiếu cache với sổ cái.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reconcile {
    pub cached: Option<Money>,
    pub ledger: Money,
    pub drift: bool,
}

/// Cache số dư theo tài khoản.
pub struct BalanceCache {
    redis: ConnectionManager,
    pool: PgPool,
    prefix: String,
    ttl_ms: u64,
    ledger_reads: AtomicU64,
}

impl BalanceCache {
    pub fn new(redis: ConnectionManager, pool: PgPool, prefix: &str, ttl_ms: u64) -> BalanceCache {
        BalanceCache {
            redis,
            pool,
            prefix: prefix.to_string(),
            ttl_ms,
            ledger_reads: AtomicU64::new(0),
        }
    }

    /// Key của một tài khoản: `<tiền tố ca>balance:<tài khoản>`.
    pub fn key(&self, account: &str) -> String {
        format!("{}balance:{account}", self.prefix)
    }

    /// Số lần đã đọc sổ cái (để chứng minh cache hit không chạm PostgreSQL).
    pub fn ledger_reads(&self) -> u64 {
        self.ledger_reads.load(Ordering::Relaxed)
    }

    /// Cache-aside: hit trả ngay, miss đọc sổ cái rồi nạp lại với TTL.
    pub async fn get(&self, account: &str) -> Result<Read> {
        if let Some(balance) = self.peek(account).await? {
            return Ok(Read {
                balance,
                source: Source::Cache,
            });
        }
        let balance = self.read_ledger(account).await?;
        self.fill(account, balance).await?;
        Ok(Read {
            balance,
            source: Source::Ledger,
        })
    }

    /// Chỉ nhìn cache, không chạm sổ cái.
    pub async fn peek(&self, account: &str) -> Result<Option<Money>> {
        let mut conn = self.redis.clone();
        let raw: Option<i64> = conn.get(self.key(account)).await?;
        Ok(raw.map(Money::from_dong))
    }

    /// Đọc sổ cái (bước 1 của một lần miss).
    pub async fn read_ledger(&self, account: &str) -> Result<Money> {
        self.ledger_reads.fetch_add(1, Ordering::Relaxed);
        store::balance(&self.pool, account).await
    }

    /// Nạp cache với TTL (bước 2 của một lần miss): `SET key value PX ttl`.
    pub async fn fill(&self, account: &str, balance: Money) -> Result<()> {
        let mut conn = self.redis.clone();
        let options = SetOptions::default().with_expiration(SetExpiry::PX(self.ttl_ms));
        let _: () = conn
            .set_options(self.key(account), balance.dong(), options)
            .await?;
        Ok(())
    }

    /// Xoá key sau khi transaction ghi đã commit.
    pub async fn invalidate(&self, account: &str) -> Result<()> {
        let mut conn = self.redis.clone();
        let _: () = conn.del(self.key(account)).await?;
        Ok(())
    }

    /// Thời gian sống còn lại (ms) của key; `None` nếu key không tồn tại.
    pub async fn ttl_ms(&self, account: &str) -> Result<Option<i64>> {
        let mut conn = self.redis.clone();
        let ms: i64 = conn.pttl(self.key(account)).await?;
        Ok((ms >= 0).then_some(ms))
    }

    /// Đối chiếu: khi cache và sổ cái khác nhau thì sổ cái đúng, ghi đè cache bằng số của sổ cái.
    pub async fn reconcile(&self, account: &str) -> Result<Reconcile> {
        let cached = self.peek(account).await?;
        let ledger = self.read_ledger(account).await?;
        let drift = cached.is_some_and(|c| c != ledger);
        if drift {
            self.fill(account, ledger).await?;
        }
        Ok(Reconcile {
            cached,
            ledger,
            drift,
        })
    }
}
