---
slug: rust-bo-nho-khong-gc-stack-heap-allocator-so-voi-dotnet
title: "Bộ nhớ không GC: stack, heap, allocator và so với .NET"
summary: "Bài 59: Drop tất định và RAII, đếm allocation bằng allocator tự viết (90.026 block cho 10.000 bút toán), so mimalloc và jemalloc, chạy C# thật cùng khối lượng để so RSS và GC, cùng lifetime nâng cao."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, memory, allocator, drop]
language: vi
---

Đội vận hành hỏi: service Rust dùng bao nhiêu RAM, và bộ nhớ được trả lại khi nào? «Rust không có GC nên nhẹ» là khẩu hiệu, không phải số đo. Bài này đếm từng lần cấp phát, đo RSS, chạy một chương trình C# thật cùng khối lượng để so, rồi nói số nào nối được với ADR bài 01 và số nào không.

Đây là bài 59 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. [Bài 16](/articles/rust-box-rc-arc-refcell-smart-pointer) đã giới thiệu stack và heap; bài này đo chúng. Lab dùng `ledger-core` của [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace); phần C# cần .NET SDK 10.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một <code>Vec&lt;Money&gt;</code> tạo bằng <code>Vec::new()</code> rồi <code>push</code> 10 phần tử thì cấp phát bao nhiêu lần, tổng bao nhiêu byte?</p>
<details><summary>Xem đáp án</summary><p>3 lần: dung lượng đi 4, 8, 16 phần tử, tức 32, 64, 128 byte, tổng 224 byte; một <code>alloc</code> và hai <code>realloc</code>. <code>Vec::with_capacity(10)</code> chỉ cần một lần, 80 byte.</p></details>
</div>

## Mỗi Box, Vec và String tốn bao nhiêu lần cấp phát?

Biến cục bộ cỡ cố định nằm trên stack và biến mất khi hàm trả về. Dữ liệu cỡ chỉ biết lúc chạy, hoặc phải sống lâu hơn hàm tạo ra nó, nằm trên heap do allocator cấp. Mọi `Box`, `Vec`, `String` đều đi qua `GlobalAlloc` của chương trình, nên ta thay được nó bằng một allocator bọc để đếm.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hình 1. Stack giữ Money 8 byte tại chỗ và ba con trỏ: Box, String, Vec. Mỗi con trỏ trỏ tới một block trên heap, và mọi cấp phát đi qua bộ đếm Counting bọc allocator hệ thống."><defs><marker id="rust-bo-nho-khong-gc-stack-heap-allocator-so-voi-dotnet-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="16" width="270" height="252" rx="12" class="f-none s-line dash"/><text x="36" y="40" class="t-b">Stack: cỡ biết lúc biên dịch</text><rect x="36" y="54" width="238" height="44" rx="8" class="f-phys-soft s-phys"/><text x="50" y="73" class="t-b">Money</text><text x="50" y="90" class="t-small">8 byte tại chỗ, 0 cấp phát</text><rect x="36" y="106" width="238" height="44" rx="8" class="f-pk-soft s-pk"/><text x="50" y="125" class="t-b">Box&lt;Money&gt;</text><text x="50" y="142" class="t-small">8 byte: địa chỉ</text><rect x="36" y="158" width="238" height="44" rx="8" class="f-pk-soft s-pk"/><text x="50" y="177" class="t-b">String</text><text x="50" y="194" class="t-small">24 byte: địa chỉ, dài, sức chứa</text><rect x="36" y="210" width="238" height="44" rx="8" class="f-pk-soft s-pk"/><text x="50" y="229" class="t-b">Vec&lt;Money&gt;</text><text x="50" y="246" class="t-small">24 byte: địa chỉ, dài, sức chứa</text><rect x="334" y="54" width="212" height="200" rx="10" class="f-warn-soft s-warn"/><text x="348" y="78" class="t-b">Counting&lt;System&gt;</text><text x="348" y="104" class="t-small">alloc: block +1, byte +size</text><text x="348" y="126" class="t-small">realloc: tính thêm một block</text><text x="348" y="148" class="t-small">dealloc: frees + 1</text><text x="348" y="182" class="t-small">rồi chuyển nguyên xi</text><text x="348" y="204" class="t-small">cho allocator hệ thống</text><rect x="590" y="16" width="270" height="252" rx="12" class="f-none s-line dash"/><text x="606" y="40" class="t-b">Heap: cỡ biết lúc chạy</text><rect x="606" y="106" width="238" height="44" rx="8" class="f-phys-soft s-phys"/><text x="620" y="125" class="t-b">1 block, 8 byte</text><text x="620" y="142" class="t-small">giá trị của Box</text><rect x="606" y="158" width="238" height="44" rx="8" class="f-phys-soft s-phys"/><text x="620" y="177" class="t-b">1 block, 15 byte</text><text x="620" y="194" class="t-small">chữ customer:CU-001</text><rect x="606" y="210" width="238" height="44" rx="8" class="f-phys-soft s-phys"/><text x="620" y="229" class="t-b">3 block: 32, 64, 128 byte</text><text x="620" y="246" class="t-small">1 alloc rồi 2 realloc</text><path d="M274 128 L330 128" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-bo-nho-khong-gc-stack-heap-allocator-so-voi-dotnet-0-a)"/><path d="M274 180 L330 180" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-bo-nho-khong-gc-stack-heap-allocator-so-voi-dotnet-0-a)"/><path d="M274 232 L330 232" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-bo-nho-khong-gc-stack-heap-allocator-so-voi-dotnet-0-a)"/><path d="M546 128 L602 128" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-bo-nho-khong-gc-stack-heap-allocator-so-voi-dotnet-0-a)"/><path d="M546 180 L602 180" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-bo-nho-khong-gc-stack-heap-allocator-so-voi-dotnet-0-a)"/><path d="M546 232 L602 232" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-bo-nho-khong-gc-stack-heap-allocator-so-voi-dotnet-0-a)"/></svg>
</div>
<figcaption>Hình 1. Số byte cấp phát của từng kiểu, đo bằng bộ đếm bọc allocator hệ thống. Money nằm trọn trên stack.</figcaption>
</figure>

```rust
// src/counting.rs (rút gọn)
pub struct Counting<A> { inner: A, blocks: AtomicU64, bytes: AtomicU64, /* ... */ }

unsafe impl<A: GlobalAlloc> GlobalAlloc for Counting<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.inner.alloc(layout) };
        if !ptr.is_null() {
            self.blocks.fetch_add(1, Relaxed);
            self.bytes.fetch_add(layout.size() as u64, Relaxed);
        }
        ptr
    }
    // dealloc, realloc, alloc_zeroed tương tự
}

// src/main.rs
#[global_allocator]
static ALLOC: Counting<System> = Counting::new(System);
```

`cargo run` in bảng sau. Đây là số đếm, không phải số giờ, nên máy tải thế nào cũng ra đúng như vậy. `realloc` tính là một block, như `dhat` ở [bài 30](/articles/rust-criterion-benchmark-do-bo-nho-10k-but-toan).

| Đoạn code | Block | Byte | `realloc` |
| --- | ---: | ---: | ---: |
| `Money` trên stack | 0 | 0 | 0 |
| `Box::new(Money)` | 1 | 8 | 0 |
| `Box::new([Money; 512])` | 1 | 4.096 | 0 |
| `String::from("customer:CU-001")` | 1 | 15 | 0 |
| `AccountId::new("customer:CU-001")` | 1 | 15 | 0 |
| `Vec::new()` + 10 `push` (dung lượng 4, 8, 16) | 3 | 224 | 2 |
| `Vec::with_capacity(10)` | 1 | 80 | 0 |
| `make_entry(1)`, một bút toán ba posting | 8 | 236 | 0 |

## Drop chạy khi nào, và RAII thay finally ra sao?

Khi chủ cuối cùng ra khỏi phạm vi, compiler chèn `Drop::drop` đúng chỗ đó. Biến cục bộ bị drop theo thứ tự ngược khai báo, field theo thứ tự khai báo, `drop(x)` giải phóng sớm:

```rust
// src/raii.rs (rút gọn)
let _conn = Guard::new(&log, "conn");
let _lock = Guard::new(&log, "lock");
let early = Guard::new(&log, "early");
drop(early);                                   // giải phóng sớm
let _pair = Pair { first: Guard::new(&log, "pair.first"), second: Guard::new(&log, "pair.second") };
let leaked = Guard::new(&log, "leaked");
mem::forget(leaked);                           // an toàn, nhưng Drop không bao giờ chạy
log.push("end of scope");
```

```text
open conn
open lock
open early
drop early
open pair.first
open pair.second
open leaked
end of scope
drop pair.first
drop pair.second
drop lock
drop conn
```

`Drop` không phải bảo đảm tuyệt đối: `mem::forget`, chu trình `Rc` hay `panic = "abort"` đều bỏ qua nó. Đặt lên `Drop` việc dọn tài nguyên cục bộ, không đặt tính đúng nghiệp vụ.

Từ đó có RAII. `TxGuard` mở giao dịch ở `begin`; `commit` nhận `self` theo giá trị nên không dùng lại được; `Drop` tự `rollback` nếu chưa commit. Không có `finally`, và không đường ra nào quên rollback, kể cả `return` sớm hay panic: với `panic = "unwind"` mặc định, stack được tháo và `Drop` của mọi biến cục bộ vẫn chạy.

```rust
// src/raii.rs (rút gọn)
impl Drop for TxGuard<'_> {
    fn drop(&mut self) {
        if !self.committed { self.log.push("tx rollback"); }
    }
}
```

```text
tx begin
tx commit
tx begin
tx rollback
tx begin
tx rollback
panic caught: true
```

Ba ca: commit, trả lỗi sớm, panic giữa chừng. `MutexGuard` cùng khuôn: ra phạm vi thì khoá được nhả. Bên C#, `using` giải cùng bài toán nhưng người gọi phải nhớ viết, còn finalizer chạy theo lịch GC ([bài 27 loạt C#](/articles/dotnet10-allocation-gc-disposal)).

## Ghi 10.000 bút toán cấp phát bao nhiêu lần?

| Chỉ số | Giá trị |
| --- | ---: |
| Block đã cấp phát | 90.026 (9 mỗi bút toán) |
| Tổng byte đã cấp phát | 5.738.224 |
| Trong đó `realloc` | 12 |
| Đỉnh bộ nhớ sống | 3.389.999 byte |
| Bộ nhớ sống sau build, sau drop | 3.389.944 byte, 676 byte (bằng mức trước build) |
| Block trả về khi drop sổ cái | 60.002 |

90.026 block trùng đúng số `dhat` ở bài 30. Chín block mỗi bút toán: tám của `make_entry` (ba là chuỗi tạm do `format!`) và một cho bản sao khoá idempotency trong chỉ mục; 26 block còn lại là `Vec` và `HashMap` của `Ledger` tự tăng. Drop sổ cái trả 60.002 block trong một lời gọi và đưa bộ nhớ sống về mức trước build. «Trả» ở đây là trả cho allocator; RSS có giảm hay không còn tuỳ allocator, như mục sau đo.

## Đổi allocator có đổi được gì?

Đổi allocator là một dòng `#[global_allocator]`. Binary `alloc-bench` có hai feature `mimalloc` và `jemalloc` (crate `tikv-jemallocator`), và cố ý không có bộ đếm.

```rust
// src/bin/alloc_bench.rs (rút gọn)
#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
```

Mỗi thread ghi 20 sổ cái 10.000 bút toán rồi bỏ từng sổ. Bảng là trung vị của 7 lần chạy xen kẽ (khoảng min–max trong ngoặc), RSS đo bằng `/usr/bin/time -l`, macOS, load average khoảng 350:

| Allocator | 1 thread: thời gian | 1 thread: RSS đỉnh | 4 thread: thời gian | 4 thread: RSS đỉnh | 4 thread: RSS sau khi drop |
| --- | ---: | ---: | ---: | ---: | ---: |
| Hệ thống (macOS) | 208 ms (96–260) | 8,7 MiB | 708 ms (502–913) | 23,2 MiB | 23,2 MiB |
| `mimalloc` 0.1.52 | 95 ms (49–189) | 10,5 MiB | 290 ms (124–437) | 40,0 MiB | 40,0 MiB |
| `tikv-jemallocator` 0.7.0 | 174 ms (69–240) | 7,6 MiB | 376 ms (178–780) | 24,8 MiB | 2,8 MiB |

Đọc theo hướng: `mimalloc` nhanh nhất nhưng giữ nhiều bộ nhớ hơn; `jemalloc` nằm giữa, đỉnh RSS gần allocator hệ thống, và RSS tụt xuống sau khi sổ cái bị drop. Khoảng min–max ở lô này rộng bằng cả khoảng cách giữa các cấu hình. Lô kiểm lại lúc load khoảng 105 (5 lần) hẹp hơn hẳn và cùng thứ tự: 1 thread 83, 69, 51 ms; 4 thread 134, 76, 59 ms (hệ thống, `jemalloc`, `mimalloc`). RSS đỉnh ít đổi theo tải, trừ `mimalloc` 4 thread dao động 34 đến 53 MiB.

Hai cảnh báo. `mimalloc` và `jemalloc` đều biên dịch C, `jemalloc` chậm hơn rõ: build release sạch mất 37 giây so với 14 (hệ thống) và 11 (`mimalloc`) lúc load khoảng 110. Và mọi số allocator ở đây chỉ đo trên macOS. Trên `glibc` hay `musl` ([bài 63](/articles/rust-container-dockerfile-da-stage-cargo-chef-distroless)) allocator mặc định và cách trả bộ nhớ đều khác, nên phải đo lại trên môi trường chạy thật.

## Một tiến trình Rust so với .NET: so thế nào cho công bằng?

`dotnet-compare/` là code C# thông thường, không tinh chỉnh, chạy cùng khối lượng (10.000 bút toán ba posting, `List` và `Dictionary` theo khoá), in `GC.GetTotalAllocatedBytes`, số lần GC và RSS.

Hai chương trình **không giống hệt**. Rust kiểm tra và sao chép mọi mã tài khoản thành `String` UTF-8, sao chép khoá vào chỉ mục và giữ `Posting` ngay trong `Vec`. C# chia sẻ tham chiếu chuỗi UTF-16 (literal dùng chung, khoá không sao chép), mỗi `Posting` là một object, tính số dư bằng LINQ, và còn có JIT.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hình 2. Hai dòng thời gian: Rust giải phóng ngay tại dấu đóng phạm vi bằng Drop; .NET giải phóng khi GC chạy, sau khi object hết được tham chiếu và gen0 đầy."><text x="110" y="28" class="t-small">thời gian, từ trái sang phải</text><path d="M110 38 L860 38" class="f-none s-line dash"/><text x="20" y="78" class="t-b">Rust</text><rect x="110" y="52" width="150" height="44" rx="8" class="f-phys-soft s-phys"/><text x="122" y="79" class="t-small">Box::new, Vec::new</text><rect x="264" y="52" width="120" height="44" rx="8" class="f-phys-soft s-phys"/><text x="276" y="79" class="t-small">dùng</text><rect x="388" y="52" width="260" height="44" rx="8" class="f-pk-soft s-pk"/><text x="400" y="79" class="t-small">hết phạm vi: Drop rồi trả bộ nhớ</text><text x="662" y="79" class="t-small">ngay tại dấu }</text><text x="20" y="158" class="t-b">.NET</text><rect x="110" y="132" width="150" height="44" rx="8" class="f-phys-soft s-phys"/><text x="122" y="159" class="t-small">new</text><rect x="264" y="132" width="120" height="44" rx="8" class="f-phys-soft s-phys"/><text x="276" y="159" class="t-small">dùng</text><rect x="388" y="132" width="150" height="44" rx="8" class="f-none s-line dash"/><text x="400" y="159" class="t-small">hết tham chiếu</text><rect x="542" y="132" width="150" height="44" rx="8" class="f-phys-soft s-phys"/><text x="554" y="159" class="t-small">chờ gen0 đầy</text><rect x="696" y="132" width="164" height="44" rx="8" class="f-warn-soft s-warn"/><text x="708" y="159" class="t-small">GC thu hồi</text><rect x="20" y="200" width="840" height="64" rx="10" class="f-warn-soft s-warn"/><text x="36" y="226" class="t-small">Drop: thời điểm giải phóng nằm trong code, đọc được bằng mắt.</text><text x="36" y="248" class="t-small">GC: thời điểm do runtime chọn; đổi lại không phải theo dõi chủ sở hữu của từng object.</text></svg>
</div>
<figcaption>Hình 2. Thời điểm giải phóng bộ nhớ: Drop đặt nó trong code, GC để runtime chọn.</figcaption>
</figure>

Trước hết kiểm khối lượng: mỗi vòng Rust cấp phát 5.738.224 byte, C# 5.939.536 byte (`--rounds 1`; 20 vòng thì 5,93 đến 6,15 MB mỗi vòng). Hai số không cùng định nghĩa: Rust đếm byte xin allocator, C# gồm header object và một lần `Summarize`. Chênh 3,5% chỉ nói «cùng cỡ»: chênh lệch bộ nhớ không đến từ chuyện một bên tạo nhiều rác hơn hẳn.

Kế đến, lab chạy .NET với Workstation GC (mặc định của console), Workstation tắt concurrent (`DOTNET_gcConcurrent=0`), Server GC kèm DATAS (`DOTNET_gcServer=1`, DATAS mặc định từ .NET 9) và Server GC tắt DATAS (`DOTNET_GCDynamicAdaptationMode=0`). Bảng: 20 vòng một thread, trung vị 7 lần chạy xen kẽ, load khoảng 380. RSS đo bằng `/usr/bin/time -l` trên binary đã build, không qua `dotnet run`; dòng Rust là `alloc-bench`.

| Cấu hình | Byte cấp phát mỗi vòng | Số lần GC | RSS đỉnh | Thời gian 20 vòng |
| --- | ---: | ---: | ---: | ---: |
| Rust, allocator hệ thống | 5.738.224 | không có GC | 8,7 MiB | 402 ms (167–633) |
| .NET Workstation GC | 5.939.536 | 19 (4 gen2) | 76,8 MiB | 2.378 ms (1.314–3.485) |
| .NET Workstation, concurrent GC tắt | 5.939.536 | 5 (4 gen2) | 72,2 MiB | 1.987 ms (1.470–2.740) |
| .NET Server GC (DATAS mặc định) | 5.939.536 | 1 | 157,8 MiB | 1.447 ms (1.080–2.422) |
| .NET Server GC, tắt DATAS | 5.939.536 | 1 | 160,8 MiB | 1.525 ms (1.056–2.237) |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 252" role="img" aria-label="Hình 3. RSS đỉnh của một tiến trình chạy 20 vòng ghi 10.000 bút toán: Rust 8,7 MiB; .NET Workstation GC 76,8 MiB; Workstation tắt concurrent 72,2 MiB; Server GC 157,8 MiB. Mỗi thanh gồm phần RSS có sẵn trước khi chạy việc và phần tăng do việc."><path d="M250 30 L250 218" class="f-none s-line dash"/><text x="236" y="238" class="t-small">0 MiB</text><path d="M415 30 L415 218" class="f-none s-line dash"/><text x="401" y="238" class="t-small">50 MiB</text><path d="M580 30 L580 218" class="f-none s-line dash"/><text x="566" y="238" class="t-small">100 MiB</text><path d="M745 30 L745 218" class="f-none s-line dash"/><text x="731" y="238" class="t-small">150 MiB</text><text x="20" y="59" class="t-small">Rust, allocator hệ thống</text><rect x="250" y="40" width="5" height="28" rx="4" class="f-warn-soft s-warn"/><rect x="255" y="40" width="24" height="28" rx="4" class="f-phys-soft s-phys"/><text x="287" y="59" class="t-small">8,7 MiB</text><text x="20" y="101" class="t-small">.NET Workstation GC</text><rect x="250" y="82" width="129" height="28" rx="4" class="f-warn-soft s-warn"/><rect x="379" y="82" width="124" height="28" rx="4" class="f-phys-soft s-phys"/><text x="511" y="101" class="t-small">76,8 MiB</text><text x="20" y="143" class="t-small">.NET Workstation, concurrent GC tắt</text><rect x="250" y="124" width="129" height="28" rx="4" class="f-warn-soft s-warn"/><rect x="379" y="124" width="109" height="28" rx="4" class="f-phys-soft s-phys"/><text x="496" y="143" class="t-small">72,2 MiB</text><text x="20" y="185" class="t-small">.NET Server GC (DATAS mặc định)</text><rect x="250" y="166" width="129" height="28" rx="4" class="f-warn-soft s-warn"/><rect x="379" y="166" width="392" height="28" rx="4" class="f-phys-soft s-phys"/><text x="779" y="185" class="t-small">157,8 MiB</text><rect x="600" y="44" width="14" height="14" rx="3" class="f-warn-soft s-warn"/><text x="620" y="56" class="t-small">có sẵn trước khi chạy việc</text><rect x="600" y="68" width="14" height="14" rx="3" class="f-phys-soft s-phys"/><text x="620" y="80" class="t-small">tăng do việc</text></svg>
</div>
<figcaption>Hình 3. RSS đỉnh sau 20 vòng (<code>/usr/bin/time -l</code>). Đoạn đầu mỗi thanh là RSS lúc chương trình bắt đầu: khoảng 1,5 MiB với Rust, 39 MiB với .NET.</figcaption>
</figure>

Bốn điều thấy được, và GC không phải điểm yếu mặc nhiên:

- **Một vòng không kích hoạt GC nào.** Với `--rounds 1`, Workstation và Server GC đều báo 0 lần GC; RSS đỉnh khoảng 47 MiB, tăng khoảng 7 MiB, còn Rust 6,2 MiB, tăng khoảng 4,7 MiB. Phần tăng cùng cỡ; 37,7 MiB sau 20 vòng là heap GC giữ lại, không phải dữ liệu sống.
- **Số lần GC là núm vặn.** Workstation chạy 19 lần ở khoảng 77 MiB; Server GC chạy đúng 1 lần nhưng tới 158 MiB. DATAS không cho khác biệt đo được; tài liệu nói nó có ích nhất với tải lên xuống, còn đây là khối lượng đều.
- **RSS cao của Server GC không có nghĩa là xấu.** Theo tài liệu, Server GC nhắm vào throughput, coi tiến trình là chủ đạo trên máy, mỗi CPU logic một heap (máy này 10) và nới heap khi còn bộ nhớ. Thêm `DOTNET_GCHeapCount=2` cho khoảng 100 MiB và 4 lần GC. Trong container, giới hạn heap mặc định là 75% giới hạn bộ nhớ. Chạy hàng trăm bản thì tài liệu khuyên Workstation tắt concurrent.
- **Phần RSS có sẵn là runtime.** Trước khi chạy việc, CoreCLR và JIT đã chiếm khoảng 39 MiB, Rust khoảng 1,5 MiB. NativeAOT có thể cho số khác; lab chưa đo.

Thời gian chỉ đọc theo hướng. Cùng binary Rust cho 402 ms ở lô này, 208 ms ở lô bảng allocator và 83 ms lúc load khoảng 105; lô đó cho .NET 323, 242 và 232 ms (Workstation, tắt concurrent, Server). Chưa tách được JIT khỏi GC: `DOTNET_TieredCompilation=0` cho một vòng 34 ms so với mặc định 19 ms ở load 100. Nên tôi không nêu «Rust nhanh gấp N lần»; trong workload ngắn này Rust thấp hơn vài lần về thời gian, không suy ra cho service đã warm up. Tổng pause GC đo được là thời gian tường trên máy quá tải, không dùng làm bằng chứng.

## Lifetime nâng cao: `'static`, `Cow`, HRTB, struct tự tham chiếu và Pin

[Bài 09](/articles/rust-vec-hashmap-btreemap-lifetime-so-cai-bo-nho) dạy `'a`. Bốn chỗ hay vấp tiếp theo có code trong `src/lifetimes.rs`.

**`'static` không có nghĩa là sống mãi**, chỉ là không chứa tham chiếu nào ngắn hơn chương trình. `Vec<Money>` thoả; `&ledger` trỏ vào biến cục bộ thì không. `thread::spawn` đòi `'static`:

```text
error[E0373]: closure may outlive the current function, but it borrows `ledger`, which is owned by the current function
4 |     let handle = thread::spawn(|| ledger.len());
  |                                ^^ ------ `ledger` is borrowed here
note: function requires argument type to outlive `'static`
```

Hai lối ra: `move`, hoặc `thread::scope`, nơi mọi thread kết thúc trước khi hàm trả về nên mượn `&Ledger` được.

**`Cow`** là «mượn nếu không phải sửa»: `normalize_memo` trả `Cow::Borrowed` khi ghi chú đã sạch (0 block) và chỉ cấp phát khi phải sửa (1 block). [Bài 61](/articles/rust-toi-uu-co-so-do-clone-cow-batching-copy-p99) bàn kỹ hơn.

**HRTB**: `for<'a> Fn(&'a str) -> &'a str` nghĩa là «đúng với mọi lifetime». Hàm thường như `without_kind` thoả. Closure gán vào biến trước khi dùng thì không: compiler không nối lifetime của kết quả với lifetime của tham số, nên `'1` và `'2` tách rời. Truyền thẳng closure vào `apply_to_both` thì nó lấy chữ ký từ bound và biên dịch được.

```text
error: lifetime may not live long enough
8 |     let strip = |s: &str| s.split_once(':').map_or(s, |(_, id)| id);
  |                     -   - ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ returning this value requires that `'1` must outlive `'2`
error[E0308]: mismatched types ... one type is more general than the other
```

Bạn hiếm khi viết `for<'a>` tay: `Fn(&str) -> &str` đã là HRTB ngầm.

**Struct tự tham chiếu** bị từ chối: `first_line: &str` trỏ vào `raw: String` của cùng struct cho E0515 và E0505, vì `raw` bị move trong lúc còn bị mượn.

```text
error[E0515]: cannot return value referencing function parameter `raw`
7 |     Statement { raw, first_line }
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ returns a value referencing data owned by the current function
error[E0505]: cannot move out of `raw` because it is borrowed
```

Lối an toàn là lưu vị trí thay vì tham chiếu:

```rust
// src/lifetimes.rs
pub struct Statement { raw: String, first_line: Range<usize> }

impl Statement {
    pub fn first_line(&self) -> &str { &self.raw[self.first_line.clone()] }
}
```

**Pin**, ở mức khái niệm: khối `async` giữ tham chiếu qua điểm `await` được compiler biến thành đúng một struct tự tham chiếu, nên không được dời sau lần `poll` đầu. `Pin<&mut Self>` hứa «giá trị này không bị move nữa» (trừ kiểu `Unpin`, vốn move thoải mái), vì vậy `Future::poll` nhận nó. Lab ghim một future như thế bằng `pin!` rồi poll hai lần: `Pending`, rồi đọc đúng giá trị (`(true, -25500)`). Khi `spawn`, tokio lo việc ghim.

## Chạy ví dụ và kiểm chứng

Từ gốc repository (Rust stable, edition 2024; phần C# cần .NET SDK 10):

```bash
cargo run --manifest-path examples/toi-noi-rust/59-memory-allocator/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/59-memory-allocator/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/59-memory-allocator/Cargo.toml
cargo run --release --manifest-path examples/toi-noi-rust/59-memory-allocator/Cargo.toml -- --workload
cargo run --release --bin alloc-bench --manifest-path examples/toi-noi-rust/59-memory-allocator/Cargo.toml -- --threads 4 --rounds 20
cargo run --release --bin alloc-bench --features mimalloc --manifest-path examples/toi-noi-rust/59-memory-allocator/Cargo.toml -- --verify
cargo run --release --bin alloc-bench --features jemalloc --manifest-path examples/toi-noi-rust/59-memory-allocator/Cargo.toml -- --verify
dotnet run --project examples/toi-noi-rust/59-memory-allocator/dotnet-compare -c Release -- --rounds 20
# RSS đỉnh: đo binary đã build, không đo qua `dotnet run`
DOTNET_gcServer=1 /usr/bin/time -l examples/toi-noi-rust/59-memory-allocator/target/dotnet-compare/bin/LedgerMemory/release/LedgerMemory --rounds 20
```

`--verify` in `VERIFY OK` sau khi kiểm số block, thứ tự drop, rollback, `Cow`, HRTB, `Pin` và ba số dư; không assert thời gian hay RSS. `cargo test` có 11 test đơn vị và một test tích hợp.

## Số liệu này nói gì về ADR bài 01, và không nói gì?

Bài 01 ghi tiêu chí «Độ trễ p99 và bộ nhớ dễ đoán», gọi phần so với .NET là **xu hướng, chưa đo**, và hẹn số ở lộ trình nâng cao. Số liệu ủng hộ được hai câu hẹp, đều về nửa bộ nhớ:

- Cả hai ngôn ngữ cấp phát cỡ 6 MB mỗi vòng. Rust cho đọc thời điểm trả bộ nhớ cho allocator ngay trong code và đếm từng block; .NET đếm được tổng byte và cho tinh chỉnh số lần thu gom.
- Tiến trình Rust nhỏ hơn cỡ một bậc về RSS đỉnh (8,7 so với 72 đến 161 MiB). Ở Workstation, hơn nửa chênh lệch là runtime có sẵn, phần còn lại là heap GC chủ động giữ. Lợi ích thật khi chạy nhiều bản nhỏ; mức sàn của .NET sau tinh chỉnh thì lab chưa đo.

Số liệu **không** nói p99 của service (chưa đo độ trễ đuôi dưới tải thật), không nói .NET thiếu cho sổ cái (một lượt 6 MB nằm dưới ngân sách gen0 nên không nói gì về GC khi chạy dài), và không nói allocator nào hợp service thật. Nói gọn: nó ủng hộ nửa bộ nhớ của tiêu chí, ở mức «bộ nhớ nhỏ và đếm được», và chưa nói gì về nửa độ trễ p99. Quyết định bài 01 không đổi vì ba tiêu chí còn lại không phụ thuộc số này; điều kiện xét lại của bài 01 (.NET đáp ứng p99 và bộ nhớ theo số đo trên cùng tải) vẫn còn nguyên.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Máy dùng chung, load average tới vài trăm: số thời gian đo trên máy viết bài, không phải SLA. Mọi số đo trên một máy macOS arm64, chưa đo Linux. Workload hư cấu, thuần bộ nhớ, chưa có PostgreSQL, Kafka hay request đồng thời. Hai chương trình không giống hệt, như đã nêu.</p>
</div>

**Bài tập:** bạn muốn thử `mimalloc` trên service thật. Nêu ba việc để kết luận có nghĩa, và một kết luận không được rút từ một lần chạy.

<details><summary>Xem lời giải</summary><p>Chạy xen kẽ nhiều lần, ghi load. So cả RSS đỉnh lẫn thời gian, kiểm nghiệp vụ không đổi. Đo trên khối lượng và image gần thật nhất. Không được kết luận «mimalloc nhanh hơn X%» từ một lần chạy.</p></details>

Bài trước: [bài 58](/articles/rust-tong-hop-kien-truc-ledger-service-workspace). Bài tiếp: [bài 60](/articles/rust-profiling-flamegraph-tokio-console-dhat).

## Tài liệu gốc

- [`std::alloc::GlobalAlloc`](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html) và [`#[global_allocator]`](https://doc.rust-lang.org/reference/runtime.html#the-global_allocator-attribute).
- [`Drop` trong The Book](https://doc.rust-lang.org/book/ch15-03-drop.html) và [thứ tự drop trong Reference](https://doc.rust-lang.org/reference/destructors.html).
- [Higher-ranked trait bound](https://doc.rust-lang.org/reference/trait-bounds.html#higher-ranked-trait-bounds), [`std::borrow::Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html), [`std::pin`](https://doc.rust-lang.org/std/pin/index.html).
- [docs.rs: mimalloc](https://docs.rs/mimalloc) và [tikv-jemallocator](https://docs.rs/tikv-jemallocator).
- [Garbage collection trong .NET](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/fundamentals) và [DATAS](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/datas), [Workstation và Server GC](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/workstation-server-gc), [cấu hình GC](https://learn.microsoft.com/en-us/dotnet/core/runtime-config/garbage-collector).
