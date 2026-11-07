//! Ghi đang chạy trong lúc đóng kỳ: hai thứ tự có thể xảy ra, dựng bằng `Barrier` và kênh,
//! không bằng `sleep`. Kết quả kiểm bằng bất biến «snapshot khớp tính lại từ posting».

use std::sync::Arc;

use anyhow::{Result, bail};
use jiff::Timestamp;
use sqlx::PgPool;
use tokio::sync::{Barrier, oneshot};

use crate::db::LabDb;
use crate::period::{close_period, close_period_naive, seed_months};
use crate::post::{entry, post, post_on};
use crate::snapshot::{balance_full, verify_snapshots};
use crate::tz::ts;

/// Kết quả một lần dựng cuộc đua «ghi dở dang, rồi đóng kỳ».
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaceOutcome {
    /// Lần đóng kỳ có phải đợi khoá (PostgreSQL báo `wait_event_type = Lock`) không.
    pub closer_waited: bool,
    /// Lần ghi dở dang có commit được không.
    pub writer_committed: bool,
    /// Số snapshot cuối kỳ lệch so với tính lại từ posting.
    pub mismatches: usize,
    /// Số dư `psp:vixanh:clearing` trong snapshot cuối kỳ.
    pub snapshot_psp: i64,
    /// Số dư cùng mốc, tính lại toàn bộ từ posting.
    pub full_psp: i64,
}

/// Nạp một bút toán nền và tạo kỳ `2026-11` đang mở.
async fn prepare(pool: &PgPool) -> Result<()> {
    seed_months(pool, 2026, 11, 1).await?;
    let base = entry(
        "trip:TN-0001:capture",
        &[
            ("customer:CU-001", -25_500),
            ("psp:vixanh:clearing", 25_500),
        ],
    )?;
    let t = ts("2026-11-10T10:00:00+07:00")?;
    post(pool, &base, t, t, None).await?;
    Ok(())
}

/// Số lần kiểm tra tối đa khi đợi một điều kiện của PostgreSQL; mỗi lần là một vòng truy vấn.
const POLL_LIMIT: u32 = 200_000;

/// Đợi tới khi backend `pid` đang chờ một khoá. Không `sleep`: mỗi vòng là một truy vấn.
async fn wait_until_blocked(pool: &PgPool, pid: i32) -> Result<()> {
    for _ in 0..POLL_LIMIT {
        let waiting: bool = sqlx::query_scalar(
            "select exists (select 1 from pg_stat_activity where pid = $1 and wait_event_type = 'Lock')",
        )
        .bind(pid)
        .fetch_one(pool)
        .await?;
        if waiting {
            return Ok(());
        }
        tokio::task::yield_now().await;
    }
    bail!("backend {pid} không bị chặn sau {POLL_LIMIT} lần kiểm tra")
}

/// Thứ tự 1: lần ghi vào kỳ đã bắt đầu (chưa commit) thì việc đóng kỳ xen vào.
/// `naive = true` dùng trigger không khoá và đóng kỳ không khoá; `false` dùng bản đúng.
///
/// # Errors
/// Lỗi PostgreSQL hoặc task bị huỷ.
pub async fn run_write_then_close(db: &LabDb, naive: bool) -> Result<RaceOutcome> {
    db.migrate().await?;
    if naive {
        db.use_naive_guard().await?;
    }
    prepare(&db.pool).await?;

    // Writer báo «đã chèn, chưa commit» qua barrier, rồi đợi tín hiệu để commit.
    let barrier = Arc::new(Barrier::new(2));
    let (release_tx, release_rx) = oneshot::channel::<()>();
    let writer = {
        let pool = db.pool.clone();
        let barrier = Arc::clone(&barrier);
        tokio::spawn(async move {
            let mut tx = pool.begin().await?;
            let late = entry(
                "trip:TN-0099:capture",
                &[
                    ("customer:CU-009", -12_000),
                    ("psp:vixanh:clearing", 12_000),
                ],
            )?;
            // posted_at nằm trong kỳ 2026-11, lúc này kỳ còn mở nên trigger cho qua.
            let at = ts("2026-11-30T23:59:00+07:00")?;
            post_on(&mut tx, &late, at, at, None).await?;
            barrier.wait().await;
            release_rx.await?;
            tx.commit().await?;
            anyhow::Ok(())
        })
    };

    barrier.wait().await; // writer đã chèn và đang giữ giao dịch mở
    let (pid_tx, pid_rx) = oneshot::channel::<i32>();
    let closer = {
        let pool = db.pool.clone();
        tokio::spawn(async move {
            let mut conn = pool.acquire().await?;
            let pid: i32 = sqlx::query_scalar("select pg_backend_pid()")
                .fetch_one(&mut *conn)
                .await?;
            let _ = pid_tx.send(pid);
            let closed_at: Timestamp = ts("2026-12-01T00:05:00+07:00")?;
            let rows = if naive {
                close_period_naive(&mut conn, "2026-11", closed_at).await
            } else {
                close_period(&mut conn, "2026-11", closed_at).await
            }?;
            anyhow::Ok(rows)
        })
    };
    let pid = pid_rx.await?;

    // Đợi tới khi closer hoặc đã xong (bản sai) hoặc đang bị chặn (bản đúng), rồi mới cho writer commit.
    let mut closer_waited = false;
    for _ in 0..POLL_LIMIT {
        if closer.is_finished() {
            break;
        }
        let waiting: bool = sqlx::query_scalar(
            "select exists (select 1 from pg_stat_activity where pid = $1 and wait_event_type = 'Lock')",
        )
        .bind(pid)
        .fetch_one(&db.pool)
        .await?;
        if waiting {
            closer_waited = true;
            break;
        }
        tokio::task::yield_now().await;
    }
    let _ = release_tx.send(());
    closer.await??;
    let writer_committed = writer.await?.is_ok();

    let mismatches = verify_snapshots(&db.pool, None).await?.len();
    let cut = ts("2026-11-30T17:00:00Z")?; // 00:00 ngày 01/12 giờ Việt Nam
    let snapshot_psp: i64 = sqlx::query_scalar(
        "select balance_dong from balance_snapshots \
          where account_id = 'psp:vixanh:clearing' and as_of = $1::timestamptz",
    )
    .bind(cut.to_string())
    .fetch_one(&db.pool)
    .await?;
    let full_psp = balance_full(&db.pool, "psp:vixanh:clearing", cut).await?;
    Ok(RaceOutcome {
        closer_waited,
        writer_committed,
        mismatches,
        snapshot_psp,
        full_psp,
    })
}

/// Kết quả thứ tự 2: việc đóng kỳ giữ khoá trước, lần ghi đến sau.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseFirstOutcome {
    /// Lần ghi có bị chặn khi kỳ đang được đóng không.
    pub writer_blocked: bool,
    /// `(SQLSTATE, thông điệp)` lần ghi nhận được sau khi việc đóng kỳ commit.
    pub writer_error: Option<(String, String)>,
}

/// Thứ tự 2: giao dịch đóng kỳ đã khoá dòng kỳ nhưng chưa commit; lần ghi đến sau phải đợi,
/// và khi việc đóng kỳ commit thì nó thấy `CLOSED` và bị từ chối.
///
/// # Errors
/// Lỗi PostgreSQL ngoài dự kiến hoặc task bị huỷ.
pub async fn run_close_first(db: &LabDb) -> Result<CloseFirstOutcome> {
    db.migrate().await?;
    prepare(&db.pool).await?;

    let mut closing = db.pool.begin().await?;
    sqlx::query(
        "update accounting_periods set status = 'CLOSED', closed_at = '2026-12-01T00:05:00+07:00'::timestamptz \
          where label = '2026-11'",
    )
    .execute(&mut *closing)
    .await?;

    let (pid_tx, pid_rx) = oneshot::channel::<i32>();
    let writer = {
        let pool = db.pool.clone();
        tokio::spawn(async move {
            let mut conn = pool.acquire().await?;
            let pid: i32 = sqlx::query_scalar("select pg_backend_pid()")
                .fetch_one(&mut *conn)
                .await?;
            let _ = pid_tx.send(pid);
            let late = entry(
                "trip:TN-0099:capture",
                &[
                    ("customer:CU-009", -12_000),
                    ("psp:vixanh:clearing", 12_000),
                ],
            )?;
            let at = ts("2026-11-30T23:59:00+07:00")?;
            let mut tx = sqlx::Acquire::begin(&mut *conn).await?;
            let result = post_on(&mut tx, &late, at, at, None).await;
            if result.is_ok() {
                tx.commit().await?;
            }
            anyhow::Ok(result)
        })
    };
    let pid = pid_rx.await?;
    wait_until_blocked(&db.pool, pid).await?;
    closing.commit().await?;
    let result = writer.await??;
    let writer_error = result.err().and_then(|e| crate::db::pg_error(&e));
    Ok(CloseFirstOutcome {
        writer_blocked: true,
        writer_error,
    })
}
