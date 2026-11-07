//! Các thí nghiệm có kết quả xác định (đếm, không đo giờ), dùng chung cho `main`, `--verify` và test.

use std::borrow::Cow;

use ledger_core::{AccountId, JournalEntry};
use sqlx::PgConnection;

use crate::account::{normalize, normalize_clone};
use crate::alloc_count::{AllocStats, measure};
use crate::columns::{Columns, ColumnsOwned};
use crate::db::{BoxError, LabDb, State, read_state};
use crate::proxy::Wire;
use crate::strategies::{Strategy, write};
use crate::workload::{make_entries, raw_account_ids};

/// Số cấp phát khi chuẩn hóa mã tài khoản và khi dựng cột.
#[derive(Debug)]
pub struct AllocReport {
    /// Số mã tài khoản đầu vào.
    pub raw: usize,
    /// Trong đó bao nhiêu mã phải đổi byte (kết quả của `normalize` là `Cow::Owned`).
    pub need_change: usize,
    /// `normalize_clone`: luôn tạo `String`.
    pub clone: AllocStats,
    /// `normalize`: trả `Cow`.
    pub cow: AllocStats,
    /// `AccountId::new` của `ledger-core`: luôn tạo `String` bên trong.
    pub account_id: AllocStats,
    /// Số bút toán đã dựng cột.
    pub entries: usize,
    /// Dựng cột bằng cách mượn.
    pub columns_borrow: AllocStats,
    /// Dựng cột bằng cách sao chép chuỗi.
    pub columns_clone: AllocStats,
}

/// Đếm cấp phát: chuẩn hóa `n` mã tài khoản (ba cách) và dựng cột cho `n` bút toán (hai cách).
pub fn alloc_experiments(n: usize) -> AllocReport {
    let raws = raw_account_ids(n);
    let need_change = raws
        .iter()
        .filter(|r| matches!(normalize(r), Some(Cow::Owned(_))))
        .count();

    // Cộng độ dài để compiler không bỏ cả vòng lặp; không gom vào Vec nên không tính cấp phát của Vec.
    let (len_clone, clone) = measure(|| {
        raws.iter()
            .map(|r| normalize_clone(r).map_or(0, |s| s.len()))
            .sum::<usize>()
    });
    let (len_cow, cow) = measure(|| {
        raws.iter()
            .map(|r| normalize(r).map_or(0, |s| s.len()))
            .sum::<usize>()
    });
    assert_eq!(len_clone, len_cow, "hai cách phải ra cùng chuỗi");
    let (_, account_id) = measure(|| {
        raws.iter()
            .map(|r| AccountId::new(r).map_or(0, |a| a.as_str().len()))
            .sum::<usize>()
    });

    let entries = make_entries(1, n as u64);
    let refs: Vec<&JournalEntry> = entries.iter().collect();
    let (borrowed, columns_borrow) = measure(|| Columns::borrow(&refs));
    let (owned, columns_clone) = measure(|| ColumnsOwned::clone_from(&refs));
    assert_eq!(borrowed.accounts.len(), owned.accounts.len());
    assert!(
        borrowed
            .accounts
            .iter()
            .zip(&owned.accounts)
            .all(|(a, b)| a == b),
        "hai cách dựng cột phải cho cùng nội dung"
    );
    AllocReport {
        raw: raws.len(),
        need_change,
        clone,
        cow,
        account_id,
        entries: n,
        columns_borrow,
        columns_clone,
    }
}

/// Kết quả đếm message ở tầng dây cho một cách ghi.
#[derive(Debug)]
pub struct WireRun {
    /// Cách ghi.
    pub strategy: Strategy,
    /// Số bút toán mới trong lượt đếm.
    pub entries: usize,
    /// Message đi qua proxy (không tính khởi động kết nối và lượt ghi làm ấm).
    pub wire: Wire,
    /// Trạng thái cuối trong database.
    pub state: State,
}

/// Ghi `entries` bằng `strategy` qua proxy đếm. Một bút toán đầu dùng để làm ấm cache prepared
/// statement của kết nối và không được đếm; phần còn lại là lượt đếm.
///
/// # Errors
/// Lỗi kết nối hoặc SQL.
pub async fn wire_run(
    db: &LabDb,
    strategy: Strategy,
    entries: &[JournalEntry],
) -> Result<WireRun, BoxError> {
    db.reset().await?;
    let proxy = db.proxy().await?;
    let mut conn = db.connect_via(&proxy).await?;
    let (warm, rest) = entries.split_at(1);
    write(&mut conn, strategy, warm).await?;
    let before = proxy.stats.snapshot();
    write(&mut conn, strategy, rest).await?;
    let wire = proxy.stats.snapshot() - before;
    let state = read_state(&mut conn).await?;
    sqlx::Connection::close(conn).await?;
    Ok(WireRun {
        strategy,
        entries: rest.len(),
        wire,
        state,
    })
}

/// Ghi lại `entries` một lần nữa trên `conn` (replay): idempotency phải giữ nguyên trạng thái.
///
/// # Errors
/// Lỗi SQL.
pub async fn replay_state(
    conn: &mut PgConnection,
    strategy: Strategy,
    entries: &[JournalEntry],
) -> Result<State, sqlx::Error> {
    write(conn, strategy, entries).await?;
    read_state(conn).await
}
