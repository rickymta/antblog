//! Allocator đếm: bọc `System`, đếm số lần cấp phát và số byte theo từng thread.
//!
//! Đếm theo thread (`thread_local!`) nên hai test chạy song song không làm sai số của nhau.
//! Số đếm là chỉ số không phụ thuộc tải máy: chạy lại ra đúng số cũ.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::ops::Sub;

thread_local! {
    // `const` init và kiểu không có Drop: truy cập không cấp phát, an toàn dùng trong allocator.
    static ALLOCS: Cell<u64> = const { Cell::new(0) };
    static REALLOCS: Cell<u64> = const { Cell::new(0) };
    static BYTES: Cell<u64> = const { Cell::new(0) };
}

fn bump(counter: &'static std::thread::LocalKey<Cell<u64>>, by: u64) {
    counter.with(|c| c.set(c.get() + by));
}

/// Allocator toàn cục của lab: cấp phát thật vẫn do `System`.
pub struct CountingAlloc;

// SAFETY: mọi thao tác cấp phát giao nguyên cho `System`; ta chỉ cộng bộ đếm cục bộ của thread.
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        bump(&ALLOCS, 1);
        bump(&BYTES, layout.size() as u64);
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        bump(&ALLOCS, 1);
        bump(&BYTES, layout.size() as u64);
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        bump(&REALLOCS, 1);
        bump(&BYTES, new_size as u64);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

/// Số lần cấp phát của một đoạn code.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AllocStats {
    /// Lần cấp phát block mới (`alloc`, `alloc_zeroed`).
    pub allocs: u64,
    /// Lần xin đổi kích thước block cũ (`realloc`), thường do `Vec`/`String` tăng dung lượng.
    pub reallocs: u64,
    /// Tổng byte đã xin, cộng cả hai loại trên.
    pub bytes: u64,
}

impl Sub for AllocStats {
    type Output = AllocStats;

    fn sub(self, rhs: AllocStats) -> AllocStats {
        AllocStats {
            allocs: self.allocs - rhs.allocs,
            reallocs: self.reallocs - rhs.reallocs,
            bytes: self.bytes - rhs.bytes,
        }
    }
}

/// Bộ đếm hiện tại của thread này.
pub fn snapshot() -> AllocStats {
    AllocStats {
        allocs: ALLOCS.with(Cell::get),
        reallocs: REALLOCS.with(Cell::get),
        bytes: BYTES.with(Cell::get),
    }
}

/// Chạy `f` và trả kèm số cấp phát xảy ra trên thread này trong lúc `f` chạy.
pub fn measure<T>(f: impl FnOnce() -> T) -> (T, AllocStats) {
    let before = snapshot();
    let out = f();
    (out, snapshot() - before)
}
