//! Global allocator tự viết: bọc một allocator khác và đếm mọi lần cấp phát.
//! Đây là cách đo thật không cần công cụ ngoài; bài 60 dùng `dhat` cho phần còn lại.

use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

/// Ảnh chụp bộ đếm tại một thời điểm. Lấy hiệu hai ảnh để đo một đoạn code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stats {
    /// Số block đã cấp phát (mỗi `alloc`, `alloc_zeroed` và `realloc` tính một block, như dhat).
    pub blocks: u64,
    /// Tổng byte đã cấp phát (với `realloc` là kích thước MỚI).
    pub bytes: u64,
    /// Số lần `realloc` (đã nằm trong `blocks`).
    pub reallocs: u64,
    /// Số lần trả bộ nhớ (`dealloc`; `realloc` không tính).
    pub frees: u64,
    /// Số byte đang sống ngay lúc chụp.
    pub live_bytes: u64,
    /// Đỉnh của `live_bytes` từ lúc bắt đầu chương trình hoặc lần `reset_peak` gần nhất.
    pub peak_bytes: u64,
}

impl Stats {
    /// Phần chênh của đoạn code nằm giữa hai ảnh chụp (`live_bytes` và `peak_bytes` lấy từ ảnh sau).
    pub fn since(self, before: Stats) -> Stats {
        Stats {
            blocks: self.blocks - before.blocks,
            bytes: self.bytes - before.bytes,
            reallocs: self.reallocs - before.reallocs,
            frees: self.frees - before.frees,
            live_bytes: self.live_bytes,
            peak_bytes: self.peak_bytes,
        }
    }
}

/// Bộ đếm bọc allocator `A`. Khai báo bằng `#[global_allocator] static ALLOC: Counting<System>`.
pub struct Counting<A> {
    inner: A,
    blocks: AtomicU64,
    bytes: AtomicU64,
    reallocs: AtomicU64,
    frees: AtomicU64,
    live: AtomicU64,
    peak: AtomicU64,
}

impl<A> Counting<A> {
    /// Bọc `inner`; `const` để dùng được trong `static`.
    pub const fn new(inner: A) -> Counting<A> {
        Counting {
            inner,
            blocks: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            reallocs: AtomicU64::new(0),
            frees: AtomicU64::new(0),
            live: AtomicU64::new(0),
            peak: AtomicU64::new(0),
        }
    }

    /// Chụp bộ đếm.
    pub fn stats(&self) -> Stats {
        Stats {
            blocks: self.blocks.load(Relaxed),
            bytes: self.bytes.load(Relaxed),
            reallocs: self.reallocs.load(Relaxed),
            frees: self.frees.load(Relaxed),
            live_bytes: self.live.load(Relaxed),
            peak_bytes: self.peak.load(Relaxed),
        }
    }

    /// Đặt lại đỉnh về mức đang sống, để đo đỉnh của riêng một đoạn code.
    pub fn reset_peak(&self) {
        self.peak.store(self.live.load(Relaxed), Relaxed);
    }

    fn grow_live(&self, size: u64) {
        let live = self.live.fetch_add(size, Relaxed) + size;
        self.peak.fetch_max(live, Relaxed);
    }

    fn on_alloc(&self, size: usize) {
        self.blocks.fetch_add(1, Relaxed);
        self.bytes.fetch_add(size as u64, Relaxed);
        self.grow_live(size as u64);
    }
}

// SAFETY: mọi lời gọi chuyển nguyên xi sang `inner` với cùng `Layout`; ta chỉ cộng bộ đếm
// nguyên tử, không cấp phát và không giữ con trỏ, nên hợp đồng của `GlobalAlloc` do `inner` bảo đảm.
unsafe impl<A: GlobalAlloc> GlobalAlloc for Counting<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.inner.alloc(layout) };
        if !ptr.is_null() {
            self.on_alloc(layout.size());
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.inner.alloc_zeroed(layout) };
        if !ptr.is_null() {
            self.on_alloc(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.inner.dealloc(ptr, layout) };
        self.frees.fetch_add(1, Relaxed);
        self.live.fetch_sub(layout.size() as u64, Relaxed);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { self.inner.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            self.reallocs.fetch_add(1, Relaxed);
            self.blocks.fetch_add(1, Relaxed);
            self.bytes.fetch_add(new_size as u64, Relaxed);
            self.live.fetch_sub(layout.size() as u64, Relaxed);
            self.grow_live(new_size as u64);
        }
        new_ptr
    }
}
