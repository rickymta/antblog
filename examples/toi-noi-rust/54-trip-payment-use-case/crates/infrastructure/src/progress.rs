//! Bộ đếm tiến độ để test chờ theo ĐIỀU KIỆN có hạn chót thay vì ngủ cố định.

use std::time::Duration;

use anyhow::{Result, anyhow};
use tokio::sync::watch;

/// Các bộ đếm của một `Service`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Counters {
    /// Message đã xử lý xong và commit offset (kể cả trùng và sai nội dung).
    pub handled: u64,
    /// Message trùng: aggregate không đổi.
    pub noops: u64,
    /// Message sai nội dung, bị bỏ qua (chưa có DLQ).
    pub poison: u64,
    /// Số lần thua tranh chấp version rồi đọc lại.
    pub conflicts: u64,
    /// Bút toán mới đã ghi.
    pub posted: u64,
    /// Dòng outbox đã phát và đánh dấu.
    pub relayed: u64,
}

/// Bên phát: consumer và relay gọi `update`, test `subscribe` rồi `wait_for`.
#[derive(Debug)]
pub struct Progress {
    tx: watch::Sender<Counters>,
}

impl Progress {
    /// Bộ đếm mới, mọi số bằng 0.
    pub fn new() -> Progress {
        Progress {
            tx: watch::channel(Counters::default()).0,
        }
    }

    /// Sửa bộ đếm và đánh thức mọi bên đang chờ.
    pub fn update(&self, change: impl FnOnce(&mut Counters)) {
        self.tx.send_modify(change);
    }

    /// Ảnh chụp hiện tại.
    pub fn snapshot(&self) -> Counters {
        self.tx.borrow().clone()
    }

    /// Đầu nhận để chờ.
    pub fn subscribe(&self) -> watch::Receiver<Counters> {
        self.tx.subscribe()
    }
}

impl Default for Progress {
    fn default() -> Progress {
        Progress::new()
    }
}

/// Chờ tới khi `done` đúng hoặc hết `within`. Không có vòng ngủ: `watch` đánh thức đúng lúc bộ đếm đổi.
///
/// # Errors
/// Hết hạn, kèm mô tả điều đang chờ và bộ đếm cuối cùng.
pub async fn wait_for(
    rx: &mut watch::Receiver<Counters>,
    what: &str,
    within: Duration,
    done: impl FnMut(&Counters) -> bool,
) -> Result<Counters> {
    let waited = tokio::time::timeout(within, rx.wait_for(done))
        .await
        .map(|result| result.map(|counters| counters.clone()));
    match waited {
        Ok(Ok(counters)) => Ok(counters),
        Ok(Err(_)) => Err(anyhow!("bên phát tiến độ đã đóng khi đang chờ: {what}")),
        Err(_) => Err(anyhow!(
            "quá {within:?} mà chưa thấy: {what} (đếm cuối: {:?})",
            rx.borrow().clone()
        )),
    }
}
