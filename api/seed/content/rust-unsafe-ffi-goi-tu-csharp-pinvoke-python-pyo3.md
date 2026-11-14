---
slug: rust-unsafe-ffi-goi-tu-csharp-pinvoke-python-pyo3
title: "unsafe và FFI: gọi Rust từ C# (P/Invoke) và Python (PyO3)"
summary: "Bài 62: thư viện tính phí thành cdylib với hàm extern C, header cbindgen, client C# P/Invoke có SafeHandle, module Python bằng PyO3 và maturin, panic và double free ở ranh giới unsafe, và vì sao sổ cái vẫn đi qua gRPC."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, unsafe, ffi, pyo3]
language: vi
---

Ứng dụng đặt chuyến viết bằng C# và công cụ phân tích của đội dữ liệu viết bằng Python đều phải chia cước 25.500 đồng thành phí nền tảng 5.100 và phần tài xế 20.400, đúng như sổ cái chia. Chép công thức sang thêm hai ngôn ngữ là thêm hai chỗ có thể lệch một đồng. Với phép tính thuần, không I/O, không trạng thái, nạp thẳng thư viện Rust vào tiến trình là lựa chọn có lý. Cái giá là `unsafe` ở ranh giới.

Đây là bài 62 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab bọc chính sách phí 20% của `ledger-core` ([bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace)) và đối chiếu với quyết định dùng gRPC cho sổ cái ở [bài 35](/articles/rust-tonic-grpc-get-balance-post-entry-interop-dotnet). Đề cương nối bài này với loạt Python qua Tới Nơi; ở đây Python chỉ là người gọi module Rust.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một hàm <code>extern "C"</code> của Rust 1.98 (edition 2024) bị <code>panic!</code> và không ai chặn. Tiến trình C# hoặc Python đang gọi nó nhận exception, hay chuyện khác xảy ra?</p>
<details><summary>Xem đáp án</summary><p>Không có exception. Rust không cho unwinding đi qua ranh giới <code>extern "C"</code> nên abort cả tiến trình bằng SIGABRT. Lab dựng đúng phép thử này ở tiến trình con và in thông điệp thật ở mục «Panic, double free và ai giải phóng».</p></details>
</div>

## Khi nào `unsafe` là hợp lý?

`unsafe` không tắt borrow checker. Nó mở một nhóm thao tác mà compiler không chứng minh được, trong đó có hai thao tác ta cần ở đây: dereference con trỏ thô và gọi hàm mà người viết đã cam kết hợp đồng bằng lời. Ranh giới FFI là chỗ không tránh được, vì compiler không nhìn thấy code C# hay Python ở bên kia.

Tới Nơi giữ ba nguyên tắc. Thu nhỏ: lõi `ledger-core` đặt `unsafe_code = "forbid"` (bài 20) nên không chứa `unsafe` nào, còn vỏ FFI là crate riêng `fee-ffi`, 8 khối `unsafe` trong khoảng 290 dòng. Ghi lý do: mỗi khối có `// SAFETY:` và `clippy::undocumented_unsafe_blocks` bắt khối thiếu lời giải thích. Đặt hàng rào: `#![deny(unsafe_op_in_unsafe_fn)]` buộc thân một `unsafe fn` cũng phải viết khối `unsafe` rõ ràng, không được ngầm hiểu cả hàm là vùng cho phép.

```text
error[E0133]: dereference of raw pointer is unsafe and requires unsafe block
 --> b.rs:2:61
  |
2 | pub unsafe extern "C" fn read_first(p: *const i64) -> i64 { *p }
  |                                                             ^^ dereference of raw pointer
  |
  = note: raw pointers may be null, dangling or unaligned; they can violate aliasing rules and cause data races: all of these are undefined behavior
note: an unsafe function restricts its caller, but its body is safe by default
```

Đoạn trên là thông điệp rustc thật (rút gọn) cho một hàm nháp ngoài lab, khi có `deny(unsafe_op_in_unsafe_fn)`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Hình 1. C# và Python gọi qua hai vỏ mỏng có unsafe, cả hai cùng bọc ledger-core là crate cấm unsafe."><defs><marker id="rust-unsafe-ffi-goi-tu-csharp-pinvoke-python-pyo3-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="230" height="80" rx="10" class="f-phys-soft s-phys"/><text x="36" y="58" class="t-b">C# · .NET 10</text><text x="36" y="80" class="t-small">LibraryImport + SafeHandle</text><rect x="20" y="150" width="230" height="80" rx="10" class="f-phys-soft s-phys"/><text x="36" y="178" class="t-b">Python 3</text><text x="36" y="200" class="t-small">import toi_noi_fee</text><rect x="340" y="30" width="230" height="80" rx="10" class="f-warn-soft s-warn"/><text x="356" y="58" class="t-b">fee-ffi (cdylib)</text><text x="356" y="80" class="t-small">extern C, 8 khối unsafe</text><rect x="340" y="150" width="230" height="80" rx="10" class="f-warn-soft s-warn"/><text x="356" y="178" class="t-b">fee-py (PyO3)</text><text x="356" y="200" class="t-small">PyO3 dựng ranh giới kiểu</text><rect x="650" y="70" width="210" height="120" rx="10" class="f-pk-soft s-pk"/><text x="666" y="108" class="t-b">ledger-core</text><text x="666" y="130" class="t-small">unsafe_code = forbid</text><text x="666" y="148" class="t-small">chính sách phí 20%</text><path d="M250 70 L336 70" class="f-none s-phys dash" stroke-width="2" marker-end="url(#rust-unsafe-ffi-goi-tu-csharp-pinvoke-python-pyo3-0-a)"/><path d="M250 190 L336 190" class="f-none s-phys dash" stroke-width="2" marker-end="url(#rust-unsafe-ffi-goi-tu-csharp-pinvoke-python-pyo3-0-a)"/><path d="M570 70 L646 110" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-unsafe-ffi-goi-tu-csharp-pinvoke-python-pyo3-0-a)"/><path d="M570 190 L646 150" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-unsafe-ffi-goi-tu-csharp-pinvoke-python-pyo3-0-a)"/><text x="262" y="60" class="t-small">C ABI</text><text x="258" y="180" class="t-small">C API CPython</text></svg>
</div>
<figcaption>Hình 1. Nét đứt là ranh giới ngôn ngữ, nơi <code>unsafe</code> được phép. Lõi bên phải không bao giờ thấy con trỏ thô.</figcaption>
</figure>

## Hàm `extern "C"` và header: những gì phải khớp

Ranh giới C chỉ hiểu số nguyên, con trỏ và struct có bố cục cố định. Tiền đi qua dưới dạng `i64` đồng, kết quả là struct `#[repr(C)]`. Không có float, không có `String`, không có `Result`. Lỗi trở thành một `FeeStatus` có kiểu `i32`.

```rust
// fee-ffi/src/lib.rs (rút gọn: bỏ doc comment và derive)
#[repr(C)]
pub struct FeeSplit {
    pub fare_dong: i64,
    pub platform_fee_dong: i64,
    pub driver_net_dong: i64,
}

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
```

Có bốn chi tiết. Thiếu `#[repr(C)]`, Rust được quyền sắp xếp lại trường. Tên hàm phải giữ nguyên để C# tìm thấy: ở edition 2024 phải viết `#[unsafe(no_mangle)]`, còn `#[no_mangle]` trần là lỗi biên dịch (`error: unsafe attribute used without unsafe`). Hàm nhận con trỏ ghi nên là `unsafe fn`, và mục `# Safety` trong doc comment là hợp đồng mà người gọi phải giữ. Thân hàm nằm trong `guard`, mục sau giải thích. Crate khai báo `crate-type = ["cdylib", "rlib"]`: `cdylib` cho `libfee_ffi.dylib` (`.so` trên Linux), `rlib` để binary `--verify` và test gọi như Rust thường.

Header C do `cbindgen` sinh trong `build.rs` (cbindgen 0.29.4, đọc được cả `unsafe(no_mangle)` của edition 2024):

```c
// fee-ffi/include/fee_ffi.h (rút gọn: bỏ mục Safety trong comment)
// Chia cước `fare_dong` và ghi kết quả vào `*out`.
FeeStatus fee_split(int64_t fare_dong,
                    struct FeeSplit *out);
```

cbindgen đọc source, không biên dịch, nên header có thể lệch với code thật mà không ai biết. Lab chặn bằng bốn lớp trong bảng dưới; chữ ký hàm là phép ép kiểu `let _: unsafe extern "C" fn(i64, *mut FeeSplit) -> FeeStatus = fee_split;`, lệch là lỗi biên dịch.

| Điều cần khớp | Kiểm bằng |
|---|---|
| Bố cục struct (24 byte, offset 0, 8, 16) | `size_of`, `offset_of!` trong `--verify`; `Marshal.SizeOf` ở C# |
| Chữ ký hàm | ép kiểu hàm trong `--verify` |
| Tên symbol không bị mangle | `nm -g` trên `libfee_ffi.dylib` |
| Header khớp code | so `HEADER` sinh lúc build với `include/fee_ffi.h` |

## C# gọi qua P/Invoke: chuỗi, struct và SafeHandle

`LibraryImport` là source generator: marshal được sinh lúc build thay vì lúc chạy như `DllImport`, nên project phải bật `AllowUnsafeBlocks`. Client nằm ở `dotnet-client/` và nhận đường dẫn thư viện qua biến `FEE_FFI_LIB`.

```csharp
// dotnet-client/Native.cs (rút gọn)
[StructLayout(LayoutKind.Sequential)]
internal struct FeeSplit { public long FareDong; public long PlatformFeeDong; public long DriverNetDong; }

internal sealed class QuoteHandle : SafeHandleZeroOrMinusOneIsInvalid
{
    public QuoteHandle() : base(ownsHandle: true) { }
    protected override bool ReleaseHandle() => Native.fee_quote_free(handle) == FeeStatus.Ok;
}

internal static partial class Native
{
    [LibraryImport("fee_ffi")]
    internal static partial FeeStatus fee_split(long fareDong, out FeeSplit result);

    [LibraryImport("fee_ffi", StringMarshalling = StringMarshalling.Utf8)]
    internal static partial QuoteHandle fee_quote_new(string? tripId, uint km, out FeeStatus status);

    // Trả IntPtr, không trả string: chuỗi do Rust cấp, .NET không được tự giải phóng nó.
    [LibraryImport("fee_ffi")]
    internal static partial IntPtr fee_quote_describe(QuoteHandle quote);

    [LibraryImport("fee_ffi")]
    internal static partial FeeStatus fee_string_free(IntPtr text);
}
```

Struct chỉ gồm `long` nên blittable: `out FeeSplit` là con trỏ tới biến cục bộ, không có bản sao nào. `StringMarshalling.Utf8` đổi `string` .NET thành byte UTF-8 kết thúc NUL cho `const char *`. Marshal sai không báo lỗi: lab cố ý nhập thêm bản `Utf16` của cùng hàm, và Rust chỉ thấy chữ «T» vì byte thứ hai của `T` trong UTF-16 là NUL. Kết quả là dữ liệu sai lặng lẽ, không phải crash.

Handle tới `FeeQuote` nằm trên heap của Rust (Hình 2). `SafeHandle` bảo đảm `fee_quote_free` chạy đúng một lần, qua `Dispose` hoặc qua finalizer nếu quên. Chuỗi mô tả cũng do Rust cấp, nên C# đọc bằng `Marshal.PtrToStringUTF8` rồi trả bằng `fee_string_free`. Gọi `Marshal.FreeHGlobal` hay `FreeCoTaskMem` trên con trỏ đó là sai: mỗi bên có allocator riêng ([bài 59](/articles/rust-bo-nho-khong-gc-stack-heap-allocator-so-voi-dotnet)).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Bốn bước đời sống của một báo giá: Rust cấp handle, Rust cấp chuỗi, C# chép chuỗi rồi trả cho Rust, Dispose trả handle cho Rust."><defs><marker id="rust-unsafe-ffi-goi-tu-csharp-pinvoke-python-pyo3-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="15" y="30" width="190" height="110" rx="10" class="f-phys-soft s-phys"/><text x="29" y="58" class="t-b">1. fee_quote_new</text><text x="29" y="82" class="t-small">Rust cấp FeeQuote</text><text x="29" y="100" class="t-small">Box::into_raw + sổ</text><text x="29" y="118" class="t-small">SafeHandle giữ con trỏ</text><rect x="235" y="30" width="190" height="110" rx="10" class="f-phys-soft s-phys"/><text x="249" y="58" class="t-b">2. fee_quote_describe</text><text x="249" y="82" class="t-small">Rust cấp char* UTF-8</text><text x="249" y="100" class="t-small">CString::into_raw + sổ</text><text x="249" y="118" class="t-small">C# chỉ được đọc</text><rect x="455" y="30" width="190" height="110" rx="10" class="f-phys-soft s-phys"/><text x="469" y="58" class="t-b">3. fee_string_free</text><text x="469" y="82" class="t-small">C# chép sang string</text><text x="469" y="100" class="t-small">trả char* cho Rust</text><text x="469" y="118" class="t-small">Rust gỡ khỏi sổ rồi drop</text><rect x="675" y="30" width="190" height="110" rx="10" class="f-phys-soft s-phys"/><text x="689" y="58" class="t-b">4. Dispose</text><text x="689" y="82" class="t-small">hoặc finalizer</text><text x="689" y="100" class="t-small">fee_quote_free(handle)</text><text x="689" y="118" class="t-small">lần hai: BadHandle</text><rect x="15" y="180" width="850" height="60" rx="10" class="f-warn-soft s-warn"/><text x="29" y="206" class="t-b">Quy tắc</text><text x="29" y="226" class="t-small">Bộ nhớ do Rust cấp luôn do Rust giải phóng. .NET chỉ giữ con trỏ và bản sao của nó.</text><path d="M205 85 L231 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-unsafe-ffi-goi-tu-csharp-pinvoke-python-pyo3-1-a)"/><path d="M425 85 L451 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-unsafe-ffi-goi-tu-csharp-pinvoke-python-pyo3-1-a)"/><path d="M645 85 L671 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-unsafe-ffi-goi-tu-csharp-pinvoke-python-pyo3-1-a)"/></svg>
</div>
<figcaption>Hình 2. Mọi ô là một lời gọi sang Rust; chỉ có bản sao string là nằm trong heap của .NET.</figcaption>
</figure>

Client C# thật in (rút gọn) như sau; `dotnet run` chạy trên .NET SDK 10.0.401:

```text
== 2. Handle và chuỗi UTF-8 ==
  ok  fee_quote_new trả handle hợp lệ
      TN-0001: 5 km, cước 25.500 VND, phí nền tảng 5.100 VND, tài xế nhận 20.400 VND
  ok  hết khối using: Dispose đã gọi fee_quote_free
      marshal nhầm UTF-16: T: 5 km, cước 25.500 VND, phí nền tảng 5.100 VND, tài xế nhận 20.400 VND
== 3. Double free và dùng sau free ==
  ok  free thủ công lần hai trên con trỏ đã giải phóng -> BadHandle
  ok  dùng handle đã Dispose -> ObjectDisposedException ở phía .NET, không gọi sang Rust
== 5. Quên Dispose: finalizer của SafeHandle ==
  ok  1.000 handle bị bỏ quên đều được finalizer trả về cho Rust
DOTNET OK
```

`Dispose` lần hai không gọi lại Rust, và gọi hàm trên handle đã đóng làm .NET ném `ObjectDisposedException` trước khi chạm vào con trỏ.

## Python gọi qua PyO3 và maturin

PyO3 không đi qua C ABI của ta. Nó dùng C API của CPython để dựng một extension module, và lo thay ta phần ranh giới: đổi `int` Python sang `i64`, đổi `Result` thành exception, chặn panic. `fee-py` bọc cùng `ledger-core`, cùng phép chia cước, với vỏ khác.

```rust
// fee-py/src/lib.rs (rút gọn)
#[pyclass(frozen, eq, get_all, module = "toi_noi_fee")]
#[derive(Debug, PartialEq, Eq)]
struct FeeSplit { fare_dong: i64, platform_fee_dong: i64, driver_net_dong: i64 }

#[pyfunction]
fn split_fare(fare_dong: i64) -> PyResult<FeeSplit> {
    let split = default_policy()
        .split(Money::from_dong(fare_dong))
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(FeeSplit {
        fare_dong,
        platform_fee_dong: split.platform_fee.dong(),
        driver_net_dong: split.driver_net.dong(),
    })
}

#[pymodule]
mod toi_noi_fee {
    #[pymodule_export]
    use super::{FeeSplit, fare_dong_for_km, panic_for_test, split_fare};
}
```

Crate không có dòng `unsafe` nào do ta viết; phần đó nằm trong macro của PyO3 0.30.1. `maturin develop` biên dịch rồi cài module vào đúng venv đang dùng, và `--with-python` làm trọn chuỗi: tạo venv, `pip install maturin` (cần mạng, chỉ vào venv), `maturin develop`, rồi chạy `python-client/check_fee.py` bằng `python -B` để không sinh `__pycache__`. Output (rút gọn):

```text
Python 3.9.6, module toi_noi_fee
      FeeSplit(fare_dong=25500, platform_fee_dong=5100, driver_net_dong=20400)
  ok  cước 25.500, phí 5.100, tài xế 20.400
  ok  cước âm -> ValueError
  ok  int Python vượt i64 -> OverflowError, không bị cắt lặng lẽ
  ok  float vào chỗ i64 -> TypeError (không có đường cho tiền float lọt vào)
  ok  panic Rust -> PanicException
  ok  PanicException kế thừa BaseException, `except Exception` không nuốt nó
PYTHON OK
```

Ba hành vi nên nhớ. `int` Python không giới hạn nên `2**63` bị từ chối bằng `OverflowError` thay vì bị cắt. Một `float` đưa vào chỗ `i64` bị `TypeError`, nên tiền float không lọt vào đường này. Panic trong Rust thành `PanicException` kế thừa `BaseException`: `except Exception` không bắt được, và tiến trình Python vẫn sống để gọi tiếp. Python cũng nạp được `cdylib` của phần trước bằng `ctypes`; lab không chạy hướng đó.

## Panic, double free và ai giải phóng

Panic đi qua ranh giới `extern "C"` từng là hành vi không xác định. Từ Rust 1.81, nó abort tiến trình. Cái chết đó vẫn giết cả tiến trình chủ, kể cả .NET hay Python, nên vỏ FFI phải tự chặn bằng `catch_unwind` và trả mã lỗi:

```rust
pub fn guard<T>(on_panic: T, f: impl FnOnce() -> T) -> T {
    // AssertUnwindSafe: sau panic ta không dùng lại trạng thái nào của closure, chỉ trả mã lỗi.
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(on_panic)
}
```

`--verify` kiểm cả hai nhánh. Với `guard`, một `panic!` giả lập trở thành `FeeStatus::Panic`. Không có `guard`, nó chạy lại chính binary ở chế độ `--child-unguarded-panic`, đọc stderr tới EOF và xác nhận tiến trình con chết vì SIGABRT (6) với thông điệp:

```text
panic in a function that cannot unwind
thread caused non-unwinding panic. aborting.
```

`guard` chỉ có tác dụng khi profile dùng `panic = "unwind"` (mặc định). Đặt `panic = "abort"` trong `Cargo.toml` thì `catch_unwind` không còn bắt được gì; `--verify` kiểm `cfg!(panic = "unwind")` để lỗi cấu hình hiện ra sớm.

Về giải phóng, quy tắc là ai cấp phát thì người đó giải phóng, và `fee_ffi.h` có sẵn cặp `fee_quote_new` / `fee_quote_free`, `fee_quote_describe` / `fee_string_free`. Để dựng double free mà lab không crash, thư viện ghi địa chỉ mọi con trỏ nó cấp vào một sổ có khoá. `fee_quote_free` gỡ địa chỉ khỏi sổ rồi mới `Box::from_raw`; lần gọi thứ hai không thấy địa chỉ và trả `BadHandle` mà không chạm vào bộ nhớ. Test `concurrent_free_has_exactly_one_winner` cho tám luồng cùng free một handle: đúng một luồng nhận `Ok`, bảy luồng nhận `BadHandle`.

Ba hàm `fee_quote_describe`, `fee_quote_free`, `fee_string_free` nhận con trỏ mà không phải `unsafe fn`, nên mọi giá trị truyền vào, kể cả từ code Rust an toàn, đều không được gây UB. Tra sổ thôi chưa đủ: bản đầu của lab tra đúng địa chỉ rồi dereference chính con trỏ phía gọi đưa vào, và Miri bắt UB với một con trỏ chỉ có địa chỉ (`ptr::without_provenance_mut`, gọi được không cần `unsafe`). Bản hiện tại chỉ lấy `.addr()` của tham số; lúc cấp phát ghi sổ bằng `expose_provenance`, lúc dùng dựng con trỏ bằng `with_exposed_provenance`. Test `free_trusts_only_the_address_not_the_callers_pointer` giữ điều đó.

Sổ này là lưới an toàn, không thay được kỷ luật sở hữu. Nó có khoá toàn cục nên không dành cho đường nóng. Nếu địa chỉ đã free được cấp lại cho một báo giá khác thì free con trỏ cũ giải phóng báo giá mới: an toàn về bộ nhớ nhưng sai về logic, và `SafeHandle` ở phía C# ngăn chuyện đó.

| Hiện tượng | Kiểm tra trước |
|---|---|
| Chuỗi nhận bên Rust chỉ còn một chữ cái | marshal UTF-16 thay vì UTF-8 |
| Tiến trình chủ chết SIGABRT, stderr «cannot unwind» | hàm `extern "C"` không qua `guard`, hoặc profile `panic = "abort"` |
| Crash ngay lúc free | free bằng allocator khác, hoặc double free |
| Số lạ trong struct | thiếu `#[repr(C)]`, sai thứ tự hay kiểu trường |
| `fee_live_allocations()` khác 0 sau khi xong | thiếu `Dispose`, hoặc quên `fee_string_free` |

Miri diễn dịch MIR của Rust và bắt use-after-free, truy cập lệch căn lề, vi phạm quy tắc con trỏ. Mười test của `fee-ffi` xanh dưới `cargo miri test -p fee-ffi --lib` (cần nightly có component `miri`; Miri cảnh báo `integer-to-pointer cast` ở các chỗ dùng `with_exposed_provenance`, đúng thiết kế). Miri không chạy được code native hay CPython, nên phía bên kia dựa vào client thật, `--verify` và việc đọc từng `// SAFETY:`.

## Vì sao sổ cái dùng gRPC mà không dùng FFI?

Cùng một ranh giới ngôn ngữ, hai cách nối cho hai loại việc khác nhau:

| | FFI trong tiến trình | gRPC (bài 35) |
|---|---|---|
| Lỗi nặng | abort hoặc UB kéo cả tiến trình chủ | `Status` trả về, hai tiến trình tách biệt |
| Trạng thái | không có: hàm thuần | PostgreSQL riêng, transaction, idempotency |
| Triển khai | cùng lịch với tiến trình gọi | độc lập |
| Hợp đồng | ABI: bố cục, cấp phát, ai free | proto sinh code cho hai phía |
| Đổi hợp đồng | tăng `fee_abi_version`, build lại mọi bên gọi | thêm trường, tương thích ngược |

Sổ cái giữ transaction, khoá số dư và idempotency key trên database mà chỉ nó được ghi (kiến trúc §4.9). Một lỗi nặng ở đó phải chết trong tiến trình của nó, không kéo theo service gọi. Đó là lý do bài 35 chọn gRPC. FFI chỉ hợp với thư viện tính phí: thuần, không I/O, chạy trong tiến trình gọi. Ngay cả khi ấy, mỗi lần đổi chính sách phí là một lần build lại và phát hành lại ở mọi tiến trình nạp thư viện; số `fee_abi_version` có để bên gọi từ chối nạp bản lệch. Lab không đo độ trễ nên bài không khẳng định FFI nhanh hơn gRPC bao nhiêu.

## Chạy ví dụ và kiểm chứng

Từ gốc repository, `--verify` chỉ cần Rust. Máy không có `nm` thì bước kiểm symbol in dòng `--  BỎ QUA` và các bước khác vẫn chạy; `nm` có mà chạy lỗi thì FAIL.

```bash
cargo run --manifest-path examples/toi-noi-rust/62-ffi-csharp-python/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/62-ffi-csharp-python/Cargo.toml -p fee-ffi
cargo run --manifest-path examples/toi-noi-rust/62-ffi-csharp-python/Cargo.toml -- --with-dotnet
cargo run --manifest-path examples/toi-noi-rust/62-ffi-csharp-python/Cargo.toml -- --with-python
```

Hai cờ cuối cần .NET SDK 10 và Python 3 có `venv`; cả hai thoát mã khác 0 khi lỗi. Đặt `FEE_PY_VENV` nếu muốn venv nằm ngoài `target/`. Phần kết của `--verify` (rút gọn):

```text
== 5. Panic ==
  ok  guard biến panic thành FeeStatus::Panic
  ok  profile dùng panic = unwind (catch_unwind có tác dụng)
  ok  extern "C" panic không có catch_unwind: tiến trình con không thoát bình thường
  ok  tiến trình con chết vì SIGABRT (6)
  ok  stderr có thông điệp «cannot unwind»
== 6. Header cbindgen ==
  ok  header sinh lúc build = fee-ffi/include/fee_ffi.h
  ok  header khai báo 8 hàm
  ok  fee_split xuất ra với tên nguyên trong libfee_ffi.dylib
VERIFY OK
```

`cargo test` chạy 10 test, `cargo clippy --workspace --all-targets -- -D warnings` sạch. Thư mục `bin/`, `obj/`, `target/`, `__pycache__`, `.dylib` và `.so` không commit.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Đây là một thư viện hàm thuần với một chính sách phí, không có I/O. Sổ đăng ký handle dùng khoá toàn cục và chưa được đo dưới tải. Lab chỉ chạy trên macOS arm64 (Python 3.9.6, .NET 10.0.401); chưa chạy trên Linux hay Windows, chưa đóng gói NuGet hay wheel, chưa kiểm tương thích ABI giữa các phiên bản ngoài một số nguyên. Miri không phủ phần bên kia ranh giới.</p>
</div>

**Bài tập:** thêm hàm `fee_quote_trip_id(quote)` trả mã chuyến cho phía gọi, không cấp phát chuỗi mới. Ai phải giữ cho con trỏ trả về còn hợp lệ, và hợp đồng của hàm nên ghi gì?

<details><summary>Xem lời giải</summary>
<p>Con trỏ trỏ vào bộ nhớ của <code>FeeQuote</code>, nên chỉ hợp lệ tới khi <code>fee_quote_free</code> chạy; phía gọi không được giữ lâu hơn handle, không được ghi hay free nó. Hợp đồng ghi «chuỗi mượn, hết hạn khi free handle». Cần lưu mã chuyến dạng <code>CString</code> trong <code>FeeQuote</code> (hiện là <code>String</code>, không có NUL cuối), trả <code>as_ptr()</code> qua <code>with_quote</code>, cập nhật header và bản commit, thêm test dùng sau free. Cách mượn bớt một lần cấp phát nhưng đẩy trách nhiệm thời hạn sống sang người gọi; lab chọn chuỗi cấp mới kèm <code>fee_string_free</code> để C# đọc xong là trả ngay.</p>
</details>

Bài trước: [bài 61](/articles/rust-toi-uu-co-so-do-clone-cow-batching-copy-p99). Bài tiếp: [bài 63](/articles/rust-container-dockerfile-da-stage-cargo-chef-distroless).

## Tài liệu gốc

- [The Rust Reference: Unsafety](https://doc.rust-lang.org/reference/unsafety.html) và [Behavior considered undefined](https://doc.rust-lang.org/reference/behavior-considered-undefined.html).
- [The Rustonomicon: FFI](https://doc.rust-lang.org/nomicon/ffi.html), [`std::panic::catch_unwind`](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html), [Rust 1.81 release notes](https://blog.rust-lang.org/2024/09/05/Rust-1.81.0/) và [`std::ptr::with_exposed_provenance`](https://doc.rust-lang.org/std/ptr/fn.with_exposed_provenance.html).
- [Edition Guide 2024: unsafe attributes](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-attributes.html) và [`unsafe_op_in_unsafe_fn`](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html).
- [Crate cbindgen](https://docs.rs/cbindgen) và [crate pyo3](https://docs.rs/pyo3).
- [P/Invoke source generation (`LibraryImport`)](https://learn.microsoft.com/dotnet/standard/native-interop/pinvoke-source-generation).
