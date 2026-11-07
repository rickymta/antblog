//! Tải thật: vòng đóng và vòng mở. Đo bằng đồng hồ thật nên chỉ in ra, không assert thời gian.
//!
//! Vòng mở gửi request theo lịch (giờ dự kiến) và tính độ trễ từ giờ dự kiến, kể cả khi hệ thống
//! chưa kịp nhận request. Vòng đóng chờ request trước xong rồi mới gửi tiếp, nên khi hệ thống kẹt
//! nó cũng ngừng gửi và không ghi lại những request lẽ ra đã phải gửi.

use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use sqlx::pool::PoolConnection;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, PgPool, Postgres};
use tokio::task::JoinSet;
use tokio::time::Instant;

use crate::db::{BoxError, LabDb};
use crate::hist::Samples;
use crate::strategies::{Strategy, write};
use crate::workload::make_entry;

/// Kết quả một lượt chạy.
#[derive(Debug)]
pub struct RealRun {
    /// Số request đã gửi (vòng mở: đúng số theo lịch).
    pub sent: u64,
    /// Số request lỗi (vẫn được ghi độ trễ).
    pub errors: u64,
    /// Độ trễ từng request, micro giây.
    pub samples: Samples,
}

fn micros(d: Duration) -> u64 {
    u64::try_from(d.as_micros()).unwrap_or(u64::MAX)
}

/// Vòng mở: `rate` request mỗi giây trong `duration`; độ trễ tính từ giờ dự kiến của request.
pub async fn open_loop<F, Fut>(rate: u64, duration: Duration, make: F) -> RealRun
where
    F: Fn(u64) -> Fut,
    Fut: Future<Output = Result<(), sqlx::Error>> + Send + 'static,
{
    let total = rate * duration.as_millis().try_into().unwrap_or(0u64) / 1000;
    let period = Duration::from_micros(1_000_000 / rate);
    let start = Instant::now();
    let mut set = JoinSet::new();
    for i in 0..total {
        let intended = start + period * u32::try_from(i).expect("ít request");
        tokio::time::sleep_until(intended).await;
        let work = make(i);
        set.spawn(async move {
            let ok = work.await.is_ok();
            (micros(intended.elapsed()), ok)
        });
    }
    let mut run = RealRun {
        sent: total,
        errors: 0,
        samples: Samples::new(),
    };
    while let Some(done) = set.join_next().await {
        let (latency, ok) = done.expect("task không panic");
        run.samples.record(latency);
        run.errors += u64::from(!ok);
    }
    run
}

/// Vòng đóng: `workers` luồng, mỗi luồng gửi request kế tiếp ngay khi request trước xong.
pub async fn closed_loop<F, Fut>(workers: usize, duration: Duration, make: F) -> RealRun
where
    F: Fn(u64) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), sqlx::Error>> + Send + 'static,
{
    let make = Arc::new(make);
    let next = Arc::new(AtomicU64::new(0));
    let deadline = Instant::now() + duration;
    let mut set = JoinSet::new();
    for _ in 0..workers {
        let (make, next) = (Arc::clone(&make), Arc::clone(&next));
        set.spawn(async move {
            let (mut latencies, mut errors) = (Vec::new(), 0u64);
            while Instant::now() < deadline {
                let id = next.fetch_add(1, Ordering::SeqCst);
                let began = Instant::now();
                errors += u64::from(make(id).await.is_err());
                latencies.push(micros(began.elapsed()));
            }
            (latencies, errors)
        });
    }
    let mut run = RealRun {
        sent: 0,
        errors: 0,
        samples: Samples::new(),
    };
    while let Some(done) = set.join_next().await {
        let (latencies, errors) = done.expect("task không panic");
        run.sent += latencies.len() as u64;
        run.errors += errors;
        for l in latencies {
            run.samples.record(l);
        }
    }
    run
}

async fn pool_of(db: &LabDb, size: u32) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(size)
        // Tắt ping trước mỗi lần lấy kết nối: nó thêm một round trip vào mỗi request.
        .test_before_acquire(false)
        .connect_with(db.options.clone())
        .await
}

/// Giữ khóa độc quyền bảng `probe` trong `hold`, bắt đầu sau `after`: mô phỏng một lần server kẹt.
async fn stall(db: &LabDb, after: Duration, hold: Duration) -> Result<(), sqlx::Error> {
    let mut conn = db.connect().await?;
    tokio::time::sleep(after).await;
    let mut tx = conn.begin().await?;
    sqlx::raw_sql("lock table probe in access exclusive mode")
        .execute(&mut *tx)
        .await?;
    tokio::time::sleep(hold).await;
    tx.commit().await
}

/// Cùng một lần kẹt thật, đo bằng vòng đóng rồi vòng mở. Trả `(vòng đóng, vòng mở)`.
///
/// # Errors
/// Lỗi kết nối hoặc SQL của phần dựng.
pub async fn stall_comparison(
    db: &LabDb,
    rate: u64,
    duration: Duration,
    stall_after: Duration,
    stall_for: Duration,
    workers: u32,
) -> Result<(RealRun, RealRun), BoxError> {
    let pool = Arc::new(pool_of(db, 8).await?);
    let probe = {
        let pool = Arc::clone(&pool);
        move |_i: u64| {
            let pool = Arc::clone(&pool);
            async move {
                sqlx::query("insert into probe default values")
                    .execute(&*pool)
                    .await
                    .map(|_| ())
            }
        }
    };
    let (blocker, closed) = tokio::join!(
        stall(db, stall_after, stall_for),
        closed_loop(workers as usize, duration, probe.clone())
    );
    blocker?;
    let (blocker, open) = tokio::join!(
        stall(db, stall_after, stall_for),
        open_loop(rate, duration, probe)
    );
    blocker?;
    pool.close().await;
    Ok((closed, open))
}

/// Mở sẵn `n` kết nối và chạy một ghi thử trên mỗi kết nối để cache prepared statement đã ấm.
async fn warm_pool(pool: &PgPool, strategy: Strategy, n: u32) -> Result<(), sqlx::Error> {
    let mut held: Vec<PoolConnection<Postgres>> = Vec::new();
    for k in 0..u64::from(n) {
        held.push(pool.acquire().await?);
        let entry = make_entry(9_000_000 + k);
        write(held.last_mut().expect("vừa push"), strategy, &[entry]).await?;
    }
    Ok(())
}

/// Độ trễ từng bút toán của một cách ghi dưới tải vòng mở (mỗi request một bút toán mới).
/// `first` là số chuyến đầu tiên, để hai lượt không trùng `idempotency_key`.
///
/// # Errors
/// Lỗi kết nối hoặc SQL của phần dựng.
pub async fn single_entry_latency(
    db: &LabDb,
    strategy: Strategy,
    rate: u64,
    duration: Duration,
    first: u64,
) -> Result<RealRun, BoxError> {
    let pool = Arc::new(pool_of(db, 8).await?);
    warm_pool(&pool, strategy, 8).await?;
    let run = open_loop(rate, duration, |i| {
        let pool = Arc::clone(&pool);
        async move {
            let entry = make_entry(first + i);
            let mut conn = pool.acquire().await?;
            write(&mut conn, strategy, std::slice::from_ref(&entry)).await
        }
    })
    .await;
    pool.close().await;
    Ok(run)
}
