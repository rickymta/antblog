//! Giữ guard qua `.await` ở nơi KHÔNG đòi `Send`, nên compiler không chặn: `join!` trong
//! `block_on` trên runtime một luồng. Nhánh đầu khóa rồi nhường thread; nhánh sau gọi khóa.
//!
//! `std::sync::Mutex::lock` chặn cả thread, nên nhánh đầu không được chạy tiếp để nhả khóa.
//! `tokio::sync::Mutex::lock` chỉ nhường, nên hai nhánh đều xong.

use std::sync::{Mutex, mpsc};
use std::time::Duration;

use tokio::runtime::Builder;
use tokio::time::sleep;

const YIELD: Duration = Duration::from_millis(1);

// Clippy cũng báo đúng lỗi này (`await_holding_lock`); ở đây ta cố ý dựng nó.
#[allow(clippy::await_holding_lock)]
async fn two_branches_std_mutex() {
    let lock = Mutex::new(0_u32);
    tokio::join!(
        biased;
        async {
            let mut guard = lock.lock().expect("khóa không bị poison");
            sleep(YIELD).await; // nhường thread, guard vẫn sống
            *guard += 1;
        },
        async {
            *lock.lock().expect("khóa không bị poison") += 1; // chặn chính thread đó
        },
    );
}

async fn two_branches_tokio_mutex() {
    let lock = tokio::sync::Mutex::new(0_u32);
    tokio::join!(
        biased;
        async {
            let mut guard = lock.lock().await;
            sleep(YIELD).await;
            *guard += 1;
        },
        async {
            *lock.lock().await += 1;
        },
    );
}

// Chạy trên một thread riêng để còn canh được. Thread kẹt bị bỏ lại, process thoát là hết.
fn finishes_within<F, Fut>(wait: Duration, make: F) -> bool
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()>,
{
    let (done, finished) = mpsc::channel();
    std::thread::spawn(move || {
        let runtime = Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("dựng được runtime");
        runtime.block_on(make());
        let _ = done.send(());
    });
    finished.recv_timeout(wait).is_ok()
}

/// Bản `std::sync::Mutex` có xong trong `wait` không. Luôn `false`: std chỉ cam kết `lock()` lần
/// hai trên cùng thread không trả về (kẹt hoặc panic).
pub fn std_mutex_finishes(wait: Duration) -> bool {
    finishes_within(wait, two_branches_std_mutex)
}

/// Bản `tokio::sync::Mutex` có xong trong `wait` không.
pub fn tokio_mutex_finishes(wait: Duration) -> bool {
    finishes_within(wait, two_branches_tokio_mutex)
}
