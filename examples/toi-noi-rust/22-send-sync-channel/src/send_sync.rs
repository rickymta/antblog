//! `Send` và `Sync` là trait đánh dấu; compiler tự suy ra, ta chỉ kiểm.
//!
//! Ba lỗi dưới đây là lỗi biên dịch thật, nằm trong doc test `compile_fail`: `cargo test` xanh
//! nghĩa là đoạn code đó bị từ chối. Rustc stable không kiểm mã lỗi trong `compile_fail`, nên lý do
//! cụ thể (thông điệp rustc) được kiểm riêng bằng tay và ghi trong bài viết.
//!
//! `Rc` đưa vào `tokio::spawn` (future không `Send`):
//!
//! ```compile_fail
//! use std::rc::Rc;
//! use std::sync::Mutex;
//! use ledger_lab::Ledger;
//!
//! async fn demo() {
//!     let shared = Rc::new(Mutex::new(Ledger::new()));
//!     tokio::spawn(async move {
//!         let _ = shared.lock().map(|l| l.len());
//!     });
//! }
//! ```
//!
//! `std::sync::MutexGuard` giữ qua `.await`:
//!
//! ```compile_fail
//! use std::sync::{Arc, Mutex};
//! use std::time::Duration;
//! use ledger_lab::Ledger;
//!
//! async fn demo(shared: Arc<Mutex<Ledger>>) {
//!     tokio::spawn(async move {
//!         let ledger = shared.lock().unwrap();
//!         tokio::time::sleep(Duration::from_millis(1)).await;
//!         let _ = ledger.len();
//!     });
//! }
//! ```
//!
//! `Arc<RefCell<_>>` đưa sang luồng khác (`RefCell` không `Sync`):
//!
//! ```compile_fail
//! use std::cell::RefCell;
//! use std::sync::Arc;
//! use ledger_lab::Ledger;
//!
//! fn demo() {
//!     let shared = Arc::new(RefCell::new(Ledger::new()));
//!     std::thread::spawn(move || {
//!         let _ = shared.borrow().len();
//!     });
//! }
//! ```

use std::sync::{Arc, Mutex, RwLock};

use ledger_core::Ledger;

const fn assert_send<T: Send>() {}
const fn assert_sync<T: Sync>() {}

// Kiểm lúc biên dịch: nếu `Ledger` mất `Send` hay `Sync` (ví dụ ai đó thêm một `Rc` vào trong),
// crate này không build được. Không có gì chạy lúc runtime.
const _: () = {
    assert_send::<Ledger>();
    assert_sync::<Ledger>();
    assert_send::<Arc<Mutex<Ledger>>>();
    assert_sync::<Arc<Mutex<Ledger>>>();
    assert_send::<Arc<RwLock<Ledger>>>();
    assert_send::<tokio::sync::mpsc::Sender<crate::actor::Command>>();
    assert_sync::<tokio::sync::mpsc::Sender<crate::actor::Command>>();
    assert_send::<crate::actor::Command>();
};
