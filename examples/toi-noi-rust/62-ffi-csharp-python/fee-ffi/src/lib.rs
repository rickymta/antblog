//! `fee-ffi`: vỏ C ABI mỏng quanh `ledger-core` để C#, Python hay C gọi thư viện tính phí trong tiến trình.
//!
//! Quy ước của lớp vỏ này (ghi cả trong README và bài 62):
//! - Mọi hàm xuất ra là `extern "C"`, tên giữ nguyên bằng `#[unsafe(no_mangle)]`, kiểu tham số chỉ gồm số nguyên,
//!   con trỏ và struct `#[repr(C)]`. Tiền là `i64` đồng, không float.
//! - Không hàm nào để panic thoát ra ngoài: thân hàm đi qua [`guard`], panic thành [`FeeStatus::Panic`].
//! - Ai cấp phát thì người đó giải phóng: `FeeQuote` và chuỗi do Rust cấp, trả về bằng
//!   [`fee_quote_free`] và [`fee_string_free`]. Phía gọi không bao giờ `free` chúng bằng allocator của mình.
//! - Mỗi con trỏ do Rust cấp được ghi vào một sổ đăng ký; con trỏ lạ hoặc đã giải phóng nhận
//!   [`FeeStatus::BadHandle`] thay vì chạm vào bộ nhớ. Con trỏ phía gọi đưa vào chỉ được dùng để lấy
//!   địa chỉ tra sổ, không bao giờ bị dereference; con trỏ thật dựng lại từ provenance đã expose lúc cấp.
//!   Nhờ vậy `fee_quote_describe`, `fee_quote_free`, `fee_string_free` không cần là `unsafe fn`.
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(clippy::undocumented_unsafe_blocks, clippy::missing_safety_doc)]

use std::collections::BTreeSet;
use std::ffi::{CStr, CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use std::sync::{Mutex, MutexGuard, PoisonError};

use ledger_core::{FeePolicy, Money, default_policy, fare_for_km};

/// Header C do cbindgen sinh lúc build (`build.rs`), để `--verify` so với bản commit ở `include/fee_ffi.h`.
pub const HEADER: &str = include_str!(concat!(env!("OUT_DIR"), "/fee_ffi.h"));

/// Phiên bản hợp đồng ABI. Đổi bố cục struct hay chữ ký hàm thì tăng số này.
pub const FEE_ABI_VERSION: u32 = 1;

/// Mã kết quả của mọi hàm xuất ra. `i32` cố định để C# ánh xạ sang `int`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeeStatus {
    /// Thành công.
    Ok = 0,
    /// Một con trỏ bắt buộc là null.
    NullPointer = 1,
    /// Giá trị không hợp lệ (cước âm, quãng đường 0, mã chuyến rỗng).
    InvalidArgument = 2,
    /// Chuỗi đầu vào không phải UTF-8.
    InvalidUtf8 = 3,
    /// Con trỏ không phải handle đang sống do thư viện này cấp (lạ, hoặc đã giải phóng).
    BadHandle = 4,
    /// Có panic bên trong và đã bị chặn lại ở ranh giới.
    Panic = 99,
}

/// Kết quả chia cước, bố cục C: ba `i64` liền nhau, 24 byte, căn lề 8.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeeSplit {
    /// Cước khách trả, đồng.
    pub fare_dong: i64,
    /// Phí nền tảng, đồng.
    pub platform_fee_dong: i64,
    /// Phần tài xế nhận, đồng.
    pub driver_net_dong: i64,
}

/// Báo giá một chuyến, nằm trên heap của Rust. Phía gọi chỉ giữ con trỏ mờ (opaque), không biết bố cục.
pub struct FeeQuote {
    trip_id: String,
    km: u32,
    split: FeeSplit,
}

// Sổ đăng ký địa chỉ các handle đang sống, ghi bằng `expose_provenance` lúc cấp. Khóa được giữ suốt
// lúc dùng handle, nên `fee_quote_free` không thể chạy chen vào giữa lúc một luồng khác đang đọc nó.
static LIVE_QUOTES: Mutex<BTreeSet<usize>> = Mutex::new(BTreeSet::new());
static LIVE_STRINGS: Mutex<BTreeSet<usize>> = Mutex::new(BTreeSet::new());

fn lock(set: &'static Mutex<BTreeSet<usize>>) -> MutexGuard<'static, BTreeSet<usize>> {
    // Khóa bị nhiễm (poison) không làm mất tính đúng của tập địa chỉ: lấy lại, không panic.
    set.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Chặn panic ở ranh giới FFI: có panic thì trả `on_panic`.
///
/// Cần `panic = "unwind"` (mặc định). Với `panic = "abort"` thì `catch_unwind` không còn tác dụng:
/// tiến trình chủ chết ngay ở panic đầu tiên.
pub fn guard<T>(on_panic: T, f: impl FnOnce() -> T) -> T {
    // AssertUnwindSafe: sau panic ta không dùng lại trạng thái nào của closure, chỉ trả mã lỗi.
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(on_panic)
}

/// Phần an toàn, không `unsafe`: chia cước theo chính sách mặc định 20% của `ledger-core`.
///
/// # Errors
/// [`FeeStatus::InvalidArgument`] khi cước âm hoặc chính sách từ chối.
pub fn split_dong(fare_dong: i64) -> Result<FeeSplit, FeeStatus> {
    let split = default_policy()
        .split(Money::from_dong(fare_dong))
        .map_err(|_| FeeStatus::InvalidArgument)?;
    Ok(FeeSplit {
        fare_dong,
        platform_fee_dong: split.platform_fee.dong(),
        driver_net_dong: split.driver_net.dong(),
    })
}

/// Số `FeeQuote` và chuỗi do thư viện cấp mà phía gọi chưa trả lại. Dùng để kiểm rò rỉ.
#[unsafe(no_mangle)]
pub extern "C" fn fee_live_allocations() -> u64 {
    (lock(&LIVE_QUOTES).len() + lock(&LIVE_STRINGS).len()) as u64
}

/// Phiên bản ABI, để phía gọi từ chối nạp bản thư viện lệch.
#[unsafe(no_mangle)]
pub extern "C" fn fee_abi_version() -> u32 {
    FEE_ABI_VERSION
}

/// Chia cước `fare_dong` và ghi kết quả vào `*out`.
///
/// # Safety
/// `out` phải null (trả `NullPointer`) hoặc trỏ tới vùng ghi được, căn lề 8, đủ chỗ cho một `FeeSplit`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fee_split(fare_dong: i64, out: *mut FeeSplit) -> FeeStatus {
    guard(FeeStatus::Panic, || {
        if out.is_null() {
            return FeeStatus::NullPointer;
        }
        match split_dong(fare_dong) {
            Ok(split) => {
                // SAFETY: `out` khác null (vừa kiểm) và hợp đồng của hàm buộc người gọi
                // đưa vùng ghi được, căn lề đúng, đủ chỗ cho một FeeSplit.
                unsafe { out.write(split) };
                FeeStatus::Ok
            }
            Err(status) => status,
        }
    })
}

/// Tạo báo giá cho chuyến `trip_id` đi `km` km. Trả handle (con trỏ mờ), hoặc null khi lỗi.
/// Mã lỗi ghi vào `*status` nếu `status` khác null. Phải giải phóng bằng [`fee_quote_free`].
///
/// # Safety
/// `trip_id` phải null (trả `NullPointer`) hoặc trỏ tới chuỗi kết thúc bằng NUL, đọc được suốt lời gọi.
/// `status` phải null hoặc trỏ tới vùng ghi được, căn lề 4, đủ chỗ cho một `FeeStatus`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fee_quote_new(
    trip_id: *const c_char,
    km: u32,
    status: *mut FeeStatus,
) -> *mut FeeQuote {
    let (handle, code) = guard((ptr::null_mut(), FeeStatus::Panic), || {
        // SAFETY: hợp đồng của hàm này về `trip_id` được chuyển nguyên cho `build_quote`.
        match unsafe { build_quote(trip_id, km) } {
            Ok(handle) => (handle, FeeStatus::Ok),
            Err(code) => (ptr::null_mut(), code),
        }
    });
    if !status.is_null() {
        // SAFETY: `status` khác null (vừa kiểm), hợp đồng buộc nó ghi được và căn lề đúng.
        unsafe { status.write(code) };
    }
    handle
}

/// # Safety
/// `trip_id` null hoặc trỏ tới chuỗi kết thúc bằng NUL, đọc được suốt lời gọi.
unsafe fn build_quote(trip_id: *const c_char, km: u32) -> Result<*mut FeeQuote, FeeStatus> {
    if trip_id.is_null() {
        return Err(FeeStatus::NullPointer);
    }
    // SAFETY: khác null (vừa kiểm); hợp đồng bảo đảm có NUL kết thúc và đọc được suốt lời gọi.
    let text = unsafe { CStr::from_ptr(trip_id) }
        .to_str()
        .map_err(|_| FeeStatus::InvalidUtf8)?;
    if text.is_empty() || km == 0 {
        return Err(FeeStatus::InvalidArgument);
    }
    let split = split_dong(fare_for_km(km).dong())?;
    let raw = Box::into_raw(Box::new(FeeQuote {
        trip_id: text.to_owned(),
        km,
        split,
    }));
    lock(&LIVE_QUOTES).insert(raw.expose_provenance());
    Ok(raw)
}

// Chạy `f` trên handle nếu và chỉ nếu handle còn sống. Giữ khóa suốt lúc `f` chạy.
fn with_quote<T>(quote: *const FeeQuote, f: impl FnOnce(&FeeQuote) -> T) -> Result<T, FeeStatus> {
    if quote.is_null() {
        return Err(FeeStatus::NullPointer);
    }
    let addr = quote.addr();
    let live = lock(&LIVE_QUOTES);
    if !live.contains(&addr) {
        return Err(FeeStatus::BadHandle);
    }
    // SAFETY: con trỏ dựng lại từ provenance mà `build_quote` đã expose, không dùng con trỏ của phía gọi.
    // Địa chỉ có trong sổ nghĩa là Box chưa free; ta giữ khóa nên không lời gọi free nào chạy chen vào.
    let quote = unsafe { &*ptr::with_exposed_provenance::<FeeQuote>(addr) };
    let out = f(quote);
    drop(live);
    Ok(out)
}

/// Chép phần chia cước của báo giá vào `*out`.
///
/// # Safety
/// `out` phải null (trả `NullPointer`) hoặc trỏ tới vùng ghi được, căn lề 8, đủ chỗ cho một `FeeSplit`.
/// `quote` có thể là giá trị bất kỳ: handle không còn sống nhận `BadHandle`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fee_quote_split(quote: *const FeeQuote, out: *mut FeeSplit) -> FeeStatus {
    guard(FeeStatus::Panic, || {
        if out.is_null() {
            return FeeStatus::NullPointer;
        }
        match with_quote(quote, |q| q.split) {
            Ok(split) => {
                // SAFETY: `out` khác null (vừa kiểm), hợp đồng buộc nó ghi được và căn lề đúng.
                unsafe { out.write(split) };
                FeeStatus::Ok
            }
            Err(status) => status,
        }
    })
}

/// Mô tả báo giá thành chuỗi UTF-8 do Rust cấp. Trả null khi handle không còn sống.
/// Phía gọi phải trả chuỗi bằng [`fee_string_free`], không `free` bằng allocator của mình.
#[unsafe(no_mangle)]
pub extern "C" fn fee_quote_describe(quote: *const FeeQuote) -> *mut c_char {
    guard(ptr::null_mut(), || {
        let text = with_quote(quote, |q| {
            format!(
                "{}: {} km, cước {}, phí nền tảng {}, tài xế nhận {}",
                q.trip_id,
                q.km,
                Money::from_dong(q.split.fare_dong),
                Money::from_dong(q.split.platform_fee_dong),
                Money::from_dong(q.split.driver_net_dong),
            )
        });
        let Ok(text) = text else {
            return ptr::null_mut();
        };
        // Chuỗi vừa format không chứa NUL (mã chuyến đến từ CStr), nên CString::new không lỗi.
        let Ok(c_text) = CString::new(text) else {
            return ptr::null_mut();
        };
        let raw = c_text.into_raw();
        lock(&LIVE_STRINGS).insert(raw.expose_provenance());
        raw
    })
}

/// Giải phóng chuỗi do [`fee_quote_describe`] trả về. Gọi lần hai trên cùng con trỏ trả `BadHandle`.
#[unsafe(no_mangle)]
pub extern "C" fn fee_string_free(text: *mut c_char) -> FeeStatus {
    guard(FeeStatus::Panic, || {
        if text.is_null() {
            return FeeStatus::NullPointer;
        }
        // `remove` trả true cho đúng một lời gọi: chỉ lời gọi đó được đi tiếp.
        let addr = text.addr();
        if !lock(&LIVE_STRINGS).remove(&addr) {
            return FeeStatus::BadHandle;
        }
        // SAFETY: địa chỉ vừa được gỡ khỏi sổ nên nó do `CString::into_raw` ở `fee_quote_describe` cấp,
        // chưa free, không luồng nào khác lấy lại được nó; con trỏ dựng lại từ provenance đã expose.
        drop(unsafe { CString::from_raw(ptr::with_exposed_provenance_mut(addr)) });
        FeeStatus::Ok
    })
}

/// Giải phóng báo giá. Gọi lần hai trên cùng handle trả `BadHandle`, không chạm vào bộ nhớ.
#[unsafe(no_mangle)]
pub extern "C" fn fee_quote_free(quote: *mut FeeQuote) -> FeeStatus {
    guard(FeeStatus::Panic, || {
        if quote.is_null() {
            return FeeStatus::NullPointer;
        }
        let addr = quote.addr();
        if !lock(&LIVE_QUOTES).remove(&addr) {
            return FeeStatus::BadHandle;
        }
        // SAFETY: địa chỉ vừa được gỡ khỏi sổ nên nó do `Box::into_raw` ở `build_quote` cấp, chưa free,
        // và `with_quote` không còn đọc nó (khóa đã được lấy khi gỡ); con trỏ dựng lại từ provenance đã expose.
        drop(unsafe { Box::from_raw(ptr::with_exposed_provenance_mut::<FeeQuote>(addr)) });
        FeeStatus::Ok
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::sync::{Arc, Barrier};

    // Các test đụng tới bộ đếm toàn cục phải chạy lần lượt.
    static SERIAL: Mutex<()> = Mutex::new(());

    fn serial() -> MutexGuard<'static, ()> {
        SERIAL.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn new_quote(id: &str, km: u32) -> (*mut FeeQuote, FeeStatus) {
        let id = CString::new(id).expect("không có NUL");
        let mut status = FeeStatus::Panic;
        // SAFETY: `id` là CString sống hết lời gọi; `status` là biến cục bộ ghi được, căn lề đúng.
        let handle = unsafe { fee_quote_new(id.as_ptr(), km, &raw mut status) };
        (handle, status)
    }

    #[test]
    fn layout_matches_the_header_contract() {
        assert_eq!(size_of::<FeeSplit>(), 24);
        assert_eq!(align_of::<FeeSplit>(), 8);
        assert_eq!(std::mem::offset_of!(FeeSplit, fare_dong), 0);
        assert_eq!(std::mem::offset_of!(FeeSplit, platform_fee_dong), 8);
        assert_eq!(std::mem::offset_of!(FeeSplit, driver_net_dong), 16);
        assert_eq!(size_of::<FeeStatus>(), 4);
    }

    #[test]
    fn split_matches_the_business_numbers() {
        let mut out = FeeSplit {
            fare_dong: 0,
            platform_fee_dong: 0,
            driver_net_dong: 0,
        };
        // SAFETY: `out` là biến cục bộ ghi được, căn lề đúng.
        let status = unsafe { fee_split(25_500, &raw mut out) };
        assert_eq!(status, FeeStatus::Ok);
        assert_eq!(
            out,
            FeeSplit {
                fare_dong: 25_500,
                platform_fee_dong: 5_100,
                driver_net_dong: 20_400
            }
        );
    }

    #[test]
    fn split_reports_errors_as_codes() {
        // SAFETY: null là giá trị hợp lệ theo hợp đồng của `fee_split`.
        let null_status = unsafe { fee_split(25_500, ptr::null_mut()) };
        assert_eq!(null_status, FeeStatus::NullPointer);
        let mut out = FeeSplit {
            fare_dong: 7,
            platform_fee_dong: 7,
            driver_net_dong: 7,
        };
        // SAFETY: `out` là biến cục bộ ghi được, căn lề đúng.
        let negative_status = unsafe { fee_split(-1, &raw mut out) };
        assert_eq!(negative_status, FeeStatus::InvalidArgument);
        assert_eq!(out.fare_dong, 7, "lỗi thì không ghi vào out");
    }

    #[test]
    fn split_never_loses_a_dong() -> Result<(), FeeStatus> {
        for fare in [0, 1, 3, 1_010, 25_500, i64::MAX] {
            let s = split_dong(fare)?;
            assert_eq!(s.platform_fee_dong + s.driver_net_dong, fare);
        }
        Ok(())
    }

    #[test]
    fn quote_lifecycle_and_double_free() {
        let _guard = serial();
        let before = fee_live_allocations();
        let (handle, status) = new_quote("TN-0001", 5);
        assert_eq!(status, FeeStatus::Ok);
        assert_eq!(fee_live_allocations(), before + 1);

        let text = fee_quote_describe(handle);
        assert!(!text.is_null());
        // SAFETY: `text` do `fee_quote_describe` trả về, còn sống, có NUL kết thúc.
        let shown = unsafe { CStr::from_ptr(text) }
            .to_str()
            .expect("UTF-8")
            .to_owned();
        assert_eq!(
            shown,
            "TN-0001: 5 km, cước 25.500 VND, phí nền tảng 5.100 VND, tài xế nhận 20.400 VND"
        );
        assert_eq!(fee_string_free(text), FeeStatus::Ok);
        assert_eq!(
            fee_string_free(text),
            FeeStatus::BadHandle,
            "free chuỗi lần hai"
        );

        assert_eq!(fee_quote_free(handle), FeeStatus::Ok);
        assert_eq!(
            fee_quote_free(handle),
            FeeStatus::BadHandle,
            "double free bị chặn"
        );
        let mut out = FeeSplit {
            fare_dong: 0,
            platform_fee_dong: 0,
            driver_net_dong: 0,
        };
        // SAFETY: `out` ghi được, căn lề đúng; `handle` đã free nên theo hợp đồng sẽ nhận BadHandle.
        let stale_status = unsafe { fee_quote_split(handle, &raw mut out) };
        assert_eq!(stale_status, FeeStatus::BadHandle, "dùng sau free");
        assert!(fee_quote_describe(handle).is_null());
        assert_eq!(fee_live_allocations(), before);
    }

    #[test]
    fn bad_input_gives_codes_not_crashes() {
        let _guard = serial();
        let mut status = FeeStatus::Ok;
        // SAFETY: null hợp lệ cho `trip_id`; `status` ghi được, căn lề đúng.
        let handle = unsafe { fee_quote_new(ptr::null(), 5, &raw mut status) };
        assert!(handle.is_null());
        assert_eq!(status, FeeStatus::NullPointer);

        let bad_utf8 = [0xFF_u8, 0xFE, 0x00];
        // SAFETY: mảng kết thúc bằng NUL và sống hết lời gọi; `status` hợp lệ.
        let handle = unsafe { fee_quote_new(bad_utf8.as_ptr().cast(), 5, &raw mut status) };
        assert!(handle.is_null());
        assert_eq!(status, FeeStatus::InvalidUtf8);

        assert_eq!(new_quote("", 5).1, FeeStatus::InvalidArgument);
        assert_eq!(new_quote("TN-0001", 0).1, FeeStatus::InvalidArgument);
        assert_eq!(fee_quote_free(ptr::null_mut()), FeeStatus::NullPointer);
        let stray = 0xDEAD_BEEF_usize as *mut FeeQuote;
        assert_eq!(
            fee_quote_free(stray),
            FeeStatus::BadHandle,
            "con trỏ lạ không bị chạm tới"
        );
    }

    #[test]
    fn guard_turns_panic_into_a_code() {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let status = guard(FeeStatus::Panic, || -> FeeStatus {
            panic!("lỗi giả lập")
        });
        std::panic::set_hook(previous);
        assert_eq!(status, FeeStatus::Panic);
        let unwinds = cfg!(panic = "unwind");
        assert!(unwinds, "catch_unwind chỉ có tác dụng khi panic = unwind");
    }

    #[test]
    fn concurrent_free_has_exactly_one_winner() {
        let _guard = serial();
        let before = fee_live_allocations();
        let (handle, _) = new_quote("TN-0001", 5);
        let addr = handle.expose_provenance();
        let barrier = Arc::new(Barrier::new(8));
        let threads: Vec<_> = (0..8)
            .map(|_| {
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    fee_quote_free(ptr::with_exposed_provenance_mut(addr))
                })
            })
            .collect();
        let results: Vec<FeeStatus> = threads
            .into_iter()
            .map(|t| t.join().expect("luồng không panic"))
            .collect();
        assert_eq!(results.iter().filter(|s| **s == FeeStatus::Ok).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter(|s| **s == FeeStatus::BadHandle)
                .count(),
            7
        );
        assert_eq!(fee_live_allocations(), before);
    }

    #[test]
    fn free_trusts_only_the_address_not_the_callers_pointer() {
        let _guard = serial();
        let before = fee_live_allocations();
        // Con trỏ chỉ có địa chỉ, không có provenance: gọi từ code an toàn, không có khối `unsafe` nào.
        // Hàm không phải `unsafe fn` thì đầu vào này cũng không được gây UB (Miri kiểm điều đó).
        let (handle, _) = new_quote("TN-0001", 5);
        let forged = ptr::without_provenance_mut::<FeeQuote>(handle.addr());
        let text = fee_quote_describe(forged);
        assert!(!text.is_null());
        assert_eq!(
            fee_string_free(ptr::without_provenance_mut(text.addr())),
            FeeStatus::Ok
        );
        assert_eq!(fee_quote_free(forged), FeeStatus::Ok);
        assert_eq!(fee_live_allocations(), before);
    }

    #[test]
    fn committed_header_matches_cbindgen_output() {
        let committed = include_str!("../include/fee_ffi.h");
        assert_eq!(
            HEADER, committed,
            "chạy lại: cargo run -- --print-header > fee-ffi/include/fee_ffi.h"
        );
    }
}
