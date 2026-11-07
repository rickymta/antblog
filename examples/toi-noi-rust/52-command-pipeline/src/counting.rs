//! Bộ đếm cấp phát cho phần đo: bọc allocator hệ thống, đếm theo từng thread.
//! Đây là chỗ DUY NHẤT của lab dùng `unsafe` (hợp đồng của `GlobalAlloc` buộc như vậy).
#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    // `const` init và kiểu không có Drop: truy cập không tự cấp phát.
    static ALLOCS: Cell<u64> = const { Cell::new(0) };
}

/// Allocator đếm số lần `alloc` trên thread hiện tại, còn lại giao cho `System`.
pub struct Counting;

// SAFETY: chỉ chuyển tiếp mọi lời gọi sang `System` nguyên vẹn; phần đếm không cấp phát.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = ALLOCS.try_with(|c| c.set(c.get() + 1));
        // SAFETY: cùng `layout` mà người gọi đã bảo đảm hợp lệ.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` do chính `System` cấp với `layout` này (qua `alloc` ở trên).
        unsafe { System.dealloc(ptr, layout) }
    }
}

/// Số lần cấp phát đã xảy ra trên thread hiện tại tính đến giờ.
pub fn allocations() -> u64 {
    ALLOCS.try_with(Cell::get).unwrap_or(0)
}
