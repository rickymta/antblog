//! Job đối soát định kỳ: đồng hồ do người gọi truyền vào, job tự tìm file của «hôm qua».

use std::path::{Path, PathBuf};

use anyhow::Result;
use jiff::civil::Date;
use sqlx::PgPool;

use crate::period::{Clock, Period, previous_business_date};
use crate::reconcile::{RunOutcome, RunRequest, reconcile_file};

/// Tên PSP của lab.
pub const PROVIDER: &str = "vixanh";

/// Đường dẫn theo quy ước của lab: `<thư mục>/vixanh_<yyyy-mm-dd>.csv`.
pub fn statement_path(dir: &Path, date: Date) -> PathBuf {
    dir.join(format!("{PROVIDER}_{date}.csv"))
}

/// Kết quả một lần job chạy.
#[derive(Debug, Clone)]
pub enum JobOutcome {
    /// VíXanh chưa giao file của ngày đó: không phải lỗi, lần sau chạy lại.
    NoStatement {
        /// Ngày nghiệp vụ cần đối soát.
        date: Date,
        /// File mong đợi.
        expected: PathBuf,
    },
    /// Đã đối soát (mới hoặc đã làm từ trước, theo hash).
    Reconciled {
        /// Ngày nghiệp vụ.
        date: Date,
        /// Kết quả.
        outcome: RunOutcome,
    },
}

/// Một lần chạy của job: hôm qua theo giờ `Asia/Ho_Chi_Minh` (tính từ `clock`), tìm file, đối soát.
/// Chạy lại bao nhiêu lần cũng được: cùng file thì `RunOutcome::AlreadyDone`.
///
/// # Errors
/// Lỗi tính ngày, đọc file hoặc cơ sở dữ liệu.
pub async fn run_daily_job(
    pool: &PgPool,
    clock: &dyn Clock,
    statements_dir: &Path,
) -> Result<JobOutcome> {
    let now = clock.now();
    let date = previous_business_date(now)?;
    let expected = statement_path(statements_dir, date);
    if !expected.is_file() {
        return Ok(JobOutcome::NoStatement { date, expected });
    }
    let period = Period::business_day(date)?;
    let req = RunRequest {
        provider: PROVIDER,
        path: &expected,
        period,
        run_at: now,
    };
    let outcome = reconcile_file(pool, &req).await?;
    Ok(JobOutcome::Reconciled { date, outcome })
}
