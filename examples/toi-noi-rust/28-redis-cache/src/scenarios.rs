//! Các kịch bản của bài: mỗi hàm chạy thật trên PostgreSQL và Redis, trả về số đo/giá trị
//! để `main` in ra và `--verify`/test kiểm tra.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use ledger_core::{
    AccountId, JournalEntry, Money, TripCharge, capture_entry, default_policy, fare_for_km,
};
use redis::{AsyncCommands, SetExpiry, SetOptions};
use tokio::sync::{Barrier, Notify};
use tokio::time::{sleep, timeout};

use crate::cache::{BalanceCache, Read, Reconcile, Source};
use crate::lab::Lab;
use crate::lock::Lock;
use crate::rate_limit::{TokenBucket, naive_try_acquire};
use crate::store::{self, PostOutcome};

const DRIVER: &str = "driver:DR-001";

/// Bút toán thu cước của một chuyến `TN-xxxx` đi `km` km (key `trip:<id>:capture`).
pub fn capture(trip_id: &str, km: u32) -> Result<JournalEntry> {
    let charge = TripCharge {
        trip_id: trip_id.to_string(),
        customer: AccountId::new("customer:CU-001")?,
        driver: AccountId::new(DRIVER)?,
        fare: fare_for_km(km),
    };
    Ok(capture_entry(&charge, &default_policy())?)
}

/// Ghi vào sổ cái rồi MỚI xoá key cache: thứ tự này là quy tắc, không đảo được.
async fn post_and_invalidate(
    cache: &BalanceCache,
    lab: &Lab,
    entry: &JournalEntry,
) -> Result<PostOutcome> {
    let outcome = store::post_entry(&lab.pool, entry).await?;
    for posting in entry.postings() {
        cache.invalidate(posting.account.as_str()).await?;
    }
    Ok(outcome)
}

/// Kết quả kịch bản cache-aside.
#[derive(Debug)]
pub struct CacheReport {
    pub first: Read,
    pub second: Read,
    pub ledger_reads_after_hit: u64,
    pub ttl_ms_after_fill: i64,
    pub after_invalidate: Read,
    pub window_cached: Option<Money>,
    pub window_ledger: Money,
    pub window_after_del: Read,
    pub race_cached: Option<Money>,
    pub race_ledger: Money,
    pub race_get: Read,
    pub reconcile: Reconcile,
    pub after_ttl_cached: Option<Money>,
    pub after_ttl_get: Read,
}

/// Cache-aside, xoá key sau commit, hai cửa sổ lệch và cách xử lý.
pub async fn cache_scenario(lab: &Lab) -> Result<CacheReport> {
    let cache = BalanceCache::new(lab.redis.clone(), lab.pool.clone(), &lab.prefix, 1_500);

    // 1. Ghi TN-0001 (5 km): tài xế nhận 20.400. Đọc lần 1 miss, lần 2 hit.
    post_and_invalidate(&cache, lab, &capture("TN-0001", 5)?).await?;
    let first = cache.get(DRIVER).await?;
    let second = cache.get(DRIVER).await?;
    let ledger_reads_after_hit = cache.ledger_reads();
    let ttl_ms_after_fill = cache
        .ttl_ms(DRIVER)
        .await?
        .ok_or_else(|| anyhow!("key không có TTL"))?;

    // 2. Ghi TN-0002 (3 km) rồi xoá key: lần đọc kế tiếp miss và thấy số mới.
    post_and_invalidate(&cache, lab, &capture("TN-0002", 3)?).await?;
    let after_invalidate = cache.get(DRIVER).await?;

    // 3. Cửa sổ A: đã commit (TN-0003, 4 km) nhưng chưa kịp DEL. Cache còn số cũ.
    store::post_entry(&lab.pool, &capture("TN-0003", 4)?).await?;
    let window_cached = cache.peek(DRIVER).await?;
    let window_ledger = store::balance(&lab.pool, DRIVER).await?;
    cache.invalidate(DRIVER).await?;
    let window_after_del = cache.get(DRIVER).await?;

    // 4. Cửa sổ B: người đọc miss, đọc sổ cái xong thì bị người ghi (TN-0004, 2 km)
    //    commit và DEL chen vào, rồi người đọc mới nạp số cũ vào cache.
    cache.invalidate(DRIVER).await?;
    let seen_by_reader = cache.read_ledger(DRIVER).await?;
    post_and_invalidate(&cache, lab, &capture("TN-0004", 2)?).await?;
    cache.fill(DRIVER, seen_by_reader).await?;
    let race_cached = cache.peek(DRIVER).await?;
    let race_ledger = store::balance(&lab.pool, DRIVER).await?;
    let race_get = cache.get(DRIVER).await?;

    // 5. Lệch thì tin sổ cái: đối chiếu và ghi đè cache bằng số của sổ cái.
    let reconcile = cache.reconcile(DRIVER).await?;

    // 6. Không có ai đối chiếu thì TTL là lưới an toàn cuối: số cũ tự biến mất.
    cache.fill(DRIVER, seen_by_reader).await?;
    sleep(Duration::from_millis(1_600)).await;
    let after_ttl_cached = cache.peek(DRIVER).await?;
    let after_ttl_get = cache.get(DRIVER).await?;

    Ok(CacheReport {
        first,
        second,
        ledger_reads_after_hit,
        ttl_ms_after_fill,
        after_invalidate,
        window_cached,
        window_ledger,
        window_after_del,
        race_cached,
        race_ledger,
        race_get,
        reconcile,
        after_ttl_cached,
        after_ttl_get,
    })
}

/// Kết quả kịch bản rate limit.
#[derive(Debug)]
pub struct RateReport {
    pub requests: u32,
    pub capacity: u32,
    pub naive_allowed: u32,
    pub lua_allowed: u32,
    pub lua_elapsed_ms: u128,
    pub small_bucket: Vec<bool>,
    pub refilled_after_wait: bool,
    pub bucket_ttl_ms: i64,
}

/// 100 yêu cầu đồng thời vào bucket dung lượng 10, mỗi giây hoàn 1 token.
pub async fn rate_limit_scenario(lab: &Lab) -> Result<RateReport> {
    let requests = 100_u32;
    let capacity = 10_u32;

    // Bản sai: GET rồi SET từ client; barrier giữ mọi SET lại cho tới khi mọi GET xong.
    let naive_key = lab.key("rl:naive:CU-001");
    let gate = Arc::new(Barrier::new(usize::try_from(requests)?));
    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..requests {
        let mut conn = lab.redis.clone();
        let (key, gate) = (naive_key.clone(), Arc::clone(&gate));
        tasks.spawn(
            async move { naive_try_acquire(&mut conn, &key, i64::from(capacity), &gate).await },
        );
    }
    let mut naive_allowed = 0;
    while let Some(joined) = tasks.join_next().await {
        if joined?? {
            naive_allowed += 1;
        }
    }

    // Bản đúng: script Lua nguyên tử.
    let bucket = Arc::new(TokenBucket::new(capacity, 1.0));
    let lua_key = lab.key("rl:lua:CU-001");
    let started = Instant::now();
    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..requests {
        let mut conn = lab.redis.clone();
        let (bucket, key) = (Arc::clone(&bucket), lua_key.clone());
        tasks.spawn(async move { bucket.try_acquire(&mut conn, &key).await });
    }
    let mut lua_allowed = 0;
    while let Some(joined) = tasks.join_next().await {
        if joined??.allowed {
            lua_allowed += 1;
        }
    }
    let lua_elapsed_ms = started.elapsed().as_millis();

    // Hoàn token: bucket 2 token, 1 token/giây. Lấy 3 lần liên tiếp, đợi 1,1 giây, lấy lại.
    let small = TokenBucket::new(2, 1.0);
    let small_key = lab.key("rl:small:CU-001");
    let mut conn = lab.redis.clone();
    let mut small_bucket = Vec::new();
    for _ in 0..3 {
        small_bucket.push(small.try_acquire(&mut conn, &small_key).await?.allowed);
    }
    sleep(Duration::from_millis(1_100)).await;
    let refilled_after_wait = small.try_acquire(&mut conn, &small_key).await?.allowed;
    let bucket_ttl_ms: i64 = conn.pttl(&small_key).await?;

    Ok(RateReport {
        requests,
        capacity,
        naive_allowed,
        lua_allowed,
        lua_elapsed_ms,
        small_bucket,
        refilled_after_wait,
        bucket_ttl_ms,
    })
}

/// Kết quả kịch bản lock.
#[derive(Debug)]
pub struct LockReport {
    pub second_try_while_held: bool,
    pub max_holders: u32,
    pub stale_release_a: bool,
    pub release_b: bool,
    pub outcomes: Vec<PostOutcome>,
    pub new_entries: i64,
    pub driver_delta: i64,
}

/// Lock có TTL không phải mutual exclusion tuyệt đối; unique key của PostgreSQL mới là chốt chặn.
pub async fn lock_scenario(lab: &Lab) -> Result<LockReport> {
    let lock_key = lab.key("lock:trip:TN-0099:capture");
    let holders = Arc::new(AtomicU32::new(0));
    let max_holders = Arc::new(AtomicU32::new(0));
    let b_entered = Arc::new(Notify::new());
    // Chuyến riêng của kịch bản này; đo số bút toán và số dư trước để lấy chênh lệch.
    let entry = capture("TN-0099", 5)?;
    let entries_before = store::entry_count(&lab.pool).await?;
    let balance_before = store::balance(&lab.pool, DRIVER).await?;

    // Worker A lấy lock với TTL 200 ms rồi "chạy chậm" ít nhất 500 ms (GC pause, mạng nghẽn, swap...),
    // và chưa ra khỏi vùng găng khi B chưa vào: kết quả không phụ thuộc máy nhanh hay chậm.
    let mut conn_a = lab.redis.clone();
    let lock_a = Lock::acquire(&mut conn_a, &lock_key, 200)
        .await?
        .ok_or_else(|| anyhow!("A phải lấy được lock"))?;
    let mut conn_probe = lab.redis.clone();
    let second_try_while_held = Lock::acquire(&mut conn_probe, &lock_key, 200)
        .await?
        .is_some();

    let worker_a = {
        let (holders, max_holders, pool, entry, b_entered) = (
            Arc::clone(&holders),
            Arc::clone(&max_holders),
            lab.pool.clone(),
            entry.clone(),
            Arc::clone(&b_entered),
        );
        tokio::spawn(async move {
            let now = holders.fetch_add(1, Ordering::SeqCst) + 1;
            max_holders.fetch_max(now, Ordering::SeqCst);
            sleep(Duration::from_millis(500)).await;
            // Giới hạn chờ: nếu B lỗi thì A không kẹt, và `check_lock` sẽ báo sai.
            let _ = timeout(Duration::from_secs(10), b_entered.notified()).await;
            let outcome = store::post_entry(&pool, &entry).await;
            holders.fetch_sub(1, Ordering::SeqCst);
            let released = lock_a.release(&mut conn_a).await;
            (outcome, released)
        })
    };

    // Worker B chờ đến khi lock của A hết hạn thì lấy được và chạy song song với A.
    let worker_b = {
        let (holders, max_holders, pool, entry) = (
            Arc::clone(&holders),
            Arc::clone(&max_holders),
            lab.pool.clone(),
            entry.clone(),
        );
        let mut conn_b = lab.redis.clone();
        let key = lock_key.clone();
        tokio::spawn(async move {
            let lock_b = loop {
                if let Some(lock) = Lock::acquire(&mut conn_b, &key, 2_000).await? {
                    break lock;
                }
                sleep(Duration::from_millis(20)).await;
            };
            let now = holders.fetch_add(1, Ordering::SeqCst) + 1;
            max_holders.fetch_max(now, Ordering::SeqCst);
            b_entered.notify_one();
            sleep(Duration::from_millis(100)).await;
            let outcome = store::post_entry(&pool, &entry).await?;
            holders.fetch_sub(1, Ordering::SeqCst);
            // Giữ lock thêm một chút để A nhả (sai token) trong lúc B còn giữ.
            sleep(Duration::from_millis(600)).await;
            let released = lock_b.release(&mut conn_b).await?;
            anyhow::Ok((outcome, released))
        })
    };

    let (outcome_a, released_a) = worker_a.await?;
    let (outcome_b, release_b) = worker_b.await??;
    Ok(LockReport {
        second_try_while_held,
        max_holders: max_holders.load(Ordering::SeqCst),
        stale_release_a: released_a?,
        release_b,
        outcomes: vec![outcome_b, outcome_a?],
        new_entries: store::entry_count(&lab.pool).await? - entries_before,
        driver_delta: store::balance(&lab.pool, DRIVER).await?.dong() - balance_before.dong(),
    })
}

/// Kết quả đo pipeline.
#[derive(Debug)]
pub struct PipelineReport {
    pub commands: usize,
    pub ping_ms: f64,
    pub sequential_ms: f64,
    pub pipelined_ms: f64,
    pub pipelined_values: Vec<Option<i64>>,
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

/// 200 lệnh `SET ... PX` gọi lần lượt (mỗi lệnh chờ một vòng mạng) so với một pipeline.
pub async fn pipeline_scenario(lab: &Lab) -> Result<PipelineReport> {
    let commands = 200_usize;
    let mut conn = lab.redis.clone();
    // Hai bộ key riêng: đọc lại bộ của pipeline mới chứng minh pipeline đã ghi thật.
    let seq_key = |i: usize| lab.key(&format!("seq:{i}"));
    let pipe_key = |i: usize| lab.key(&format!("pipe:{i}"));
    let options = SetOptions::default().with_expiration(SetExpiry::PX(30_000));

    let mut sequential = Vec::new();
    let mut pipelined = Vec::new();
    // Một vòng PING làm đường cơ sở: thời gian một vòng mạng tới Redis trên máy này.
    let mut pings = Vec::new();
    for _ in 0..20 {
        let started = Instant::now();
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;
        pings.push(started.elapsed().as_secs_f64() * 1_000.0);
    }
    // Vòng 0 để làm nóng, 3 vòng sau lấy trung vị.
    for round in 0..4 {
        let started = Instant::now();
        for i in 0..commands {
            let _: () = conn.set_options(seq_key(i), i, options.clone()).await?;
        }
        let seq = started.elapsed().as_secs_f64() * 1_000.0;

        let started = Instant::now();
        let mut pipe = redis::pipe();
        for i in 0..commands {
            pipe.set_options(pipe_key(i), i, options.clone()).ignore();
        }
        let _: () = pipe.query_async(&mut conn).await?;
        let pip = started.elapsed().as_secs_f64() * 1_000.0;
        if round > 0 {
            sequential.push(seq);
            pipelined.push(pip);
        }
    }
    let pipe_keys: Vec<String> = (0..commands).map(pipe_key).collect();
    let pipelined_values: Vec<Option<i64>> = conn.mget(pipe_keys).await?;
    Ok(PipelineReport {
        commands,
        ping_ms: median(pings),
        sequential_ms: median(sequential),
        pipelined_ms: median(pipelined),
        pipelined_values,
    })
}

/// Tiện ích cho `Source` khi in.
pub fn source_name(source: Source) -> &'static str {
    match source {
        Source::Cache => "cache",
        Source::Ledger => "sổ cái",
    }
}
