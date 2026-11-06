---
slug: rust-kieu-so-tien-vnd-i64-decimal
title: "Kiểu số và tiền VND: i64, checked arithmetic và rust_decimal, không dùng float"
summary: "Bài 03: các kiểu số của Rust, vì sao 0.1 + 0.2 != 0.3, tràn số debug và release, checked_add, as và try_from, rồi tính phí 20% bằng rust_decimal và chốt lưu i64 đồng."
category: programming
level: beginner
tags: [rust, toi-noi, ledger, integer, decimal]
language: vi
---

Đây là bài 03 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Ở [bài 02](/articles/rust-cargo-tu-source-code-den-but-toan-dau-tien), sổ cái ghi bút toán đầu tiên 25.500 VND. Bài này xét con số đó nằm trong kiểu nào và tính phí nền tảng 20% thế nào để luôn khớp từng đồng.

Chuyến `TN-0001` cước 25.500 đồng, phí 5.100, tài xế `DR-001` nhận 20.400. Một chuyến cước 25.503 đồng thì 20% là 5.100,6 đồng, mà VND không có đồng lẻ. Ai làm tròn, theo quy tắc nào, bằng kiểu gì?

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Phép cộng <code>i32::MAX + 1</code>, với giá trị chỉ biết lúc chạy, cho kết quả gì ở bản debug và ở bản release?</p>
<details><summary>Xem đáp án</summary><p>Bản debug panic với «attempt to add with overflow». Bản release mặc định quay vòng về <code>-2147483648</code>. (Viết thẳng hằng số thì <code>rustc</code> từ chối ngay lúc biên dịch.) Sổ cái không được dựa vào mặc định của profile.</p></details>
</div>

## Vì sao 25.500 đồng không nên là số thực?

`f32` và `f64` lưu số theo cơ số 2. Số thập phân như 0,1 không có biểu diễn hữu hạn trong cơ số 2, giống 1/3 trong cơ số 10. Trích `src/main.rs` (rút gọn):

```rust
let sum = 0.1_f64 + 0.2_f64;
println!("0.1 + 0.2 = {sum}");
println!("0.1 + 0.2 == 0.3 ? {}", sum == 0.3);
```

```text
0.1 + 0.2 = 0.30000000000000004
0.1 + 0.2 == 0.3 ? false
```

Tổng posting của bút toán phải đúng bằng 0, không phải «gần bằng 0». Vì vậy tiền không dùng `f32`/`f64`. Float vẫn đúng chỗ cho tọa độ hay điểm thống kê, nơi sai số nhỏ chấp nhận được.

## Rust có những kiểu số nào, và literal suy ra kiểu gì?

Số nguyên có dấu: `i8`, `i16`, `i32`, `i64`, `i128`, `isize`. Không dấu: `u8` … `u128`, `usize`. Con số là số bit; `isize`/`usize` rộng bằng con trỏ máy (8 byte trên máy chạy lab). Số thực có `f32` và `f64`.

Literal nhận kiểu từ ngữ cảnh; nếu ngữ cảnh không nói gì, số nguyên mặc định là `i32`, số thực là `f64`. Ba cách sau cùng giá trị 25.500, khác kiểu (`show_integer_types`, rút gọn):

```rust
let fare = 25_500; // không có gì ép kiểu, mặc định i32
let fare_vnd = 25_500i64; // hậu tố i64
let fare_typed: i64 = 25_500; // chú thích kiểu
```

```text
25_500 mặc định: 4 byte; hậu tố i64: 8 byte; chú thích i64: 8 byte
```

Dấu `_` chỉ để nhóm chữ số. Rust không tự ép kiểu giữa các kiểu số: cộng `i64` với `i32` là lỗi biên dịch. Lab giữ đoạn lỗi dưới dạng comment (`let km: i32 = 5;` rồi `let total = fare_vnd + km;`); bỏ comment thì `rustc` báo (rút gọn):

```text
error[E0308]: mismatched types
  --> src/main.rs:43:28
   |
43 |     let total = fare_vnd + km;
   |                            ^^ expected `i64`, found `i32`

error[E0277]: cannot add `i32` to `i64`
```

`E0277` nghĩa là `i64` không implement `Add<i32>`. Muốn cộng, đổi kiểu tường minh: `fare_vnd + i64::from(km)`.

## Điều gì xảy ra khi i32 tràn?

`i32` dừng ở 2.147.483.647, tức hơn 2,1 tỷ đồng; một tổng đối soát cuối ngày có thể vượt. `i64` tới 9.223.372.036.854.775.807 đồng, rất xa mọi tổng thực tế.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 1. Phạm vi i32 và i64 quanh số 0; i32::MAX + 1 vượt phạm vi."><defs><marker id="rs03-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="40" y="82" class="t-b">i32 · 32 bit</text><rect x="290" y="50" width="300" height="50" rx="8" class="f-phys-soft s-phys"/><text x="300" y="80" class="t-small">-2.147.483.648</text><text x="580" y="80" text-anchor="end" class="t-small">2.147.483.647</text><path d="M590 75 L608 75" class="f-none s-phys" stroke-width="2" marker-end="url(#rs03-0-a)"/><rect x="610" y="45" width="250" height="60" rx="8" class="f-warn-soft s-warn"/><text x="624" y="68" class="t-b">MAX + 1 vượt phạm vi</text><text x="624" y="90" class="t-small">debug: panic · release: wrap</text><text x="40" y="132" class="t-b">i64 · 64 bit</text><text x="840" y="132" text-anchor="end" class="t-small">không vẽ đúng tỷ lệ</text><rect x="40" y="140" width="800" height="50" rx="8" class="f-pk-soft s-pk"/><text x="52" y="170" class="t-small">-9.223.372.036.854.775.808</text><text x="828" y="170" text-anchor="end" class="t-small">9.223.372.036.854.775.807</text><path d="M440 190 L440 204" class="f-none s-phys" stroke-width="2"/><text x="440" y="226" text-anchor="middle" class="t-small">0 · cước 25.500 đồng nằm sát giữa</text></svg>
</div>
<figcaption>Hình 1. Phạm vi i32 so với i64. Vượt MAX là tràn số; hành vi phụ thuộc profile build.</figcaption>
</figure>

Hành vi khi tràn phụ thuộc cách build. Hàm dưới chỉ chạy với `--overflow`. `black_box` che giá trị khỏi compiler, vì viết thẳng `i32::MAX + 1` thì `rustc` từ chối lúc biên dịch: `error: this arithmetic operation will overflow`.

```rust
fn show_overflow() {
    println!("debug_assertions = {}", cfg!(debug_assertions));
    let max = black_box(i32::MAX);
    let one = black_box(1);
    let total = max + one;
    println!("i32::MAX + 1 = {total}");
}
```

Bản debug (`cargo run`), exit code 101 (số trong ngoặc sau `'main'` là mã thread, đổi theo mỗi lần chạy):

```text
debug_assertions = true

thread 'main' (…) panicked at src/main.rs:133:17:
attempt to add with overflow
```

Bản release (`cargo run --release`), exit code 0:

```text
debug_assertions = false
i32::MAX + 1 = -2147483648
```

Tùy chọn cargo `overflow-checks` quyết định điều này: mặc định bật ở profile `dev`, tắt ở `release`. Khi tắt, tràn số không phải hành vi không xác định; kết quả quay vòng theo bù hai (wrap). Nhưng với sổ cái, số dư âm bất ngờ là lỗi tiền thật. Thêm `[profile.release]` với `overflow-checks = true` vào `Cargo.toml` (lab để sẵn dạng comment) thì bản release cũng panic với cùng thông điệp. Panic chỉ là lưới an toàn cuối; lớp chính là các phép toán tường minh ở mục sau.

## checked, saturating, wrapping: chọn cái nào cho tiền?

Mỗi kiểu nguyên có bốn họ phương thức nói rõ ý định về tràn:

| Phương thức | Khi tràn | Dùng khi |
| --- | --- | --- |
| `checked_add`, `checked_mul`, `checked_sub` | trả `None` | Tiền: tràn là lỗi phải xử lý |
| `saturating_add` | kẹt ở `MAX`/`MIN` | Bộ đếm, thanh tiến độ; không dùng cho số dư |
| `wrapping_add` | quay vòng | Hash, checksum |
| `overflowing_add` | trả `(giá trị quay vòng, true)` | Cần cả kết quả lẫn cờ tràn |

Trích `show_checked_arithmetic` (rút gọn), kết quả thật ghi ở comment:

```rust
i64::MAX.checked_add(1)      // None
25_500i64.checked_add(100)   // Some(25600)
i64::MAX.saturating_add(1)   // 9223372036854775807
i64::MAX.wrapping_add(1)     // -9223372036854775808
let big = i128::from(i64::MAX) + i128::from(i64::MAX); // 18446744073709551614
i64::try_from(big)           // Err(TryFromIntError(PosOverflow))
```

`checked_*` trả `Option<i64>`: `Some(giá trị)` hoặc `None`, và chương trình phải xử lý `None` trước khi dùng giá trị. `saturating_add` cho tổng sai mà không báo. Khi cần cộng nhiều khoản lớn, cộng trong `i128` rồi `try_from` để biết kết quả có vừa `i64` không.

## `as` khác `try_from` ở đâu?

`as` không bao giờ báo lỗi: thu hẹp số nguyên thì cắt bớt bit cao, đổi dấu cùng cỡ thì giữ nguyên bit (`-1` thành `u64::MAX`), từ số thực sang nguyên thì kẹt ở biên. `TryFrom`/`try_into` trả `Result`, lỗi khi giá trị không vừa. Trích `show_casts` (rút gọn), kết quả thật ghi ở comment:

```rust
let large: i64 = 5_000_000_000;
large as i32                 // 705032704
300i64 as u8                 // 44
-1i64 as u64                 // 18446744073709551615
1e20_f64 as i64              // 9223372036854775807
i32::try_from(large)         // Err(TryFromIntError(PosOverflow))
```

Năm tỷ đồng thành 705.032.704 bằng `as i32` mà không một dòng cảnh báo. Quy tắc cho sổ cái: thu hẹp hoặc đổi dấu thì dùng `try_from`; nới rộng thì dùng `i128::from(x)`, không mất dữ liệu; `as` chỉ khi bạn chứng minh giá trị luôn vừa.

## Tính phí 20% bằng rust_decimal rồi quay về i64 thế nào?

Tỷ lệ 20% cần phần thập phân chính xác, số tiền thì không. `rust_decimal` (`cargo add rust_decimal`, bản 1.43.0 trong lab) có kiểu `Decimal` lưu một số nguyên 96 bit kèm số chữ số thập phân (scale), nên 0,20 được biểu diễn chính xác. Luồng: đưa `i64` vào `Decimal`, nhân tỷ lệ, làm tròn về đồng, lấy lại `i64`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 190" role="img" aria-label="Hình 2. i64 đồng, Decimal nhân 0,20, làm tròn, rồi i64 đồng."><defs><marker id="rs03-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="235" y="25" width="450" height="120" rx="12" class="f-none s-pk dash"/><rect x="20" y="50" width="165" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="80" class="t-b">i64 đồng</text><text x="34" y="104" class="t-small">cước 25.500</text><path d="M185 85 L243 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rs03-1-a)"/><rect x="245" y="50" width="165" height="70" rx="10" class="f-pk-soft s-pk"/><text x="259" y="80" class="t-b">Decimal × 0,20</text><text x="259" y="104" class="t-small">5100.00 hoặc 5100.60</text><path d="M410 85 L468 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rs03-1-a)"/><rect x="470" y="50" width="165" height="70" rx="10" class="f-pk-soft s-pk"/><text x="484" y="80" class="t-b">Làm tròn</text><text x="484" y="104" class="t-small">nửa đồng ra xa 0</text><path d="M635 85 L693 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rs03-1-a)"/><rect x="695" y="50" width="165" height="70" rx="10" class="f-phys-soft s-phys"/><text x="709" y="80" class="t-b">i64 đồng</text><text x="709" y="104" class="t-small">to_i64() = 5.100</text><text x="460" y="170" text-anchor="middle" class="t-small">Decimal chỉ sống trong hàm tính phí; vào và ra luôn là i64</text></svg>
</div>
<figcaption>Hình 2. Luồng tiền của phí nền tảng: i64 đồng, Decimal cho tỷ lệ, làm tròn, rồi i64 đồng.</figcaption>
</figure>

Mã trích từ `src/main.rs`:

```rust
use rust_decimal::prelude::*;

fn platform_fee(fare: i64, rate: Decimal) -> Option<i64> {
    let raw = Decimal::from(fare).checked_mul(rate)?;
    // Làm tròn về đồng: nửa đồng làm tròn ra xa số 0 (ROUND_HALF_UP).
    let rounded = raw.round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero);
    rounded.to_i64()
}

fn fee(fare: i64) -> Option<i64> {
    platform_fee(fare, Decimal::new(20, 2)) // 20 * 10^-2 = 0.20
}

fn driver_share(fare: i64) -> Option<i64> {
    fare.checked_sub(fee(fare)?)
}
```

`Decimal::new(20, 2)` là 20 × 10⁻², tức 0,20. `to_i64()` thuộc trait `ToPrimitive`, nạp bởi `rust_decimal::prelude::*`, trả `Option<i64>`. Dấu `?` nghĩa là «gặp `None` thì dừng và trả `None`»: phép nhân tràn hoặc kết quả không vừa `i64` đều ra `None`. Kết quả thật:

```text
20% của 25500 = 5100.00 -> làm tròn Some(5100)
20% của 25503 = 5100.60 -> làm tròn Some(5101)
Tài xế nhận từ 25_500: Some(20400)
5% của 25_530 = 1276.50; ra xa số 0 -> 1277; về số chẵn -> 1276
```

`TN-0001` ra phí 5.100, tài xế nhận 20.400. Ca 25.503 đồng ra 5.100,60 và làm tròn thành 5.101. Dòng cuối cho thấy làm tròn là quyết định nghiệp vụ: với đúng nửa đồng, `MidpointAwayFromZero` cho 1.277, `MidpointNearestEven` (về số chẵn) cho 1.276. Hai bên đối soát phải thống nhất quy tắc; Tới Nơi chọn ra xa số 0.

## Sổ cái chọn kiểu nào để lưu tiền?

Quyết định của sổ cái: **lưu `i64` đơn vị đồng; chỉ dùng `Decimal` cho tỷ lệ, và đưa kết quả về `i64` ngay sau khi làm tròn.** VND không có phần lẻ nên số nguyên là biểu diễn tự nhiên: cộng trừ chính xác, và tổng posting bằng 0 kiểm tra được tuyệt đối.

| Kiểu | Dùng cho | Không dùng cho |
| --- | --- | --- |
| `i64` | Số tiền VND đơn vị đồng, số dư, posting có dấu | Tỷ lệ phần trăm, giá trị có phần lẻ |
| `i128` | Tổng trung gian rất lớn khi đối soát nhiều giao dịch | Kiểu lưu mặc định khi `i64` đã đủ |
| `u64`, `u32` | Số lượng, bộ đếm không âm | Tiền có thể âm (posting ghi nợ) |
| `usize` | Chỉ số mảng, độ dài | Tiền, mã lưu DB (kích thước đổi theo máy) |
| `i32` | Số km nguyên, bộ đếm nhỏ | Tổng tiền (tràn ở hơn 2,1 tỷ đồng) |
| `f64` | Tọa độ, khoảng cách đo, điểm thống kê | Tiền, số dư, tỷ lệ phải khớp từng đồng |
| `Decimal` | Tỷ lệ phí, khuyến mãi; tính trung gian | Cột số dư của sổ cái (đã chọn `i64`) |

Đối chiếu C#: [bài 02 của loạt C#](/articles/csharp-bien-kieu-du-lieu-decimal) dùng `decimal` cho tiền tính toán rồi `Math.Round` với `MidpointRounding.AwayFromZero` ra số tiền phải trả. Đó là lựa chọn hợp lý khi `decimal` có sẵn trong .NET. Ở sổ cái ta thu hẹp hơn: kiểu lưu không cho phép phần lẻ, `Decimal` chỉ để tính tỷ lệ. Quy tắc làm tròn giống nhau, nên cùng cước ra cùng phí.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Lab chỉ chứng minh kiểu số: chưa có newtype <code>Money</code>, cột PostgreSQL, số âm hay hoàn tiền; tỷ lệ 20% đặt cứng trong mã. Production cần kiểu miền có kiểm tra bất biến, quy tắc làm tròn ghi trong chính sách và test sinh dữ liệu. <code>Cargo.lock</code> pin phiên bản <code>rust_decimal</code>; đọc changelog trước khi nâng.</p>
</div>

## Chạy ví dụ và kiểm chứng

Từ gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/03-kieu-so-tien-vnd/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/03-kieu-so-tien-vnd/Cargo.toml -- --verify
cargo run --manifest-path examples/toi-noi-rust/03-kieu-so-tien-vnd/Cargo.toml -- --overflow
cargo run --release --manifest-path examples/toi-noi-rust/03-kieu-so-tien-vnd/Cargo.toml -- --overflow
```

Lệnh hai in `VERIFY OK` và thoát mã 0 khi mọi assert đúng; sai thì panic, mã 101. Nó kiểm bẫy float, `checked_add`, `as`/`try_from`, `fee(25_500) == Some(5_100)`, `fee(25_503) == Some(5_101)`, tài xế nhận 20.400 và hai chiến lược làm tròn nửa đồng. Hai lệnh `--overflow` tái hiện panic ở debug và wrap ở release.

**Bài tập:** (1) Dự đoán rồi chạy thử `fee(25_502)` và `driver_share(25_502)`. (2) Vì sao `u32` không hợp để lưu số tiền của một posting?

<details><summary>Xem lời giải</summary><p>(1) 20% của 25.502 là 5.100,4, làm tròn thành 5.100, nên tài xế nhận 25.502 - 5.100 = 20.402 (<code>--verify</code> của lab đã kiểm hai giá trị này). (2) <code>u32</code> không có số âm trong khi posting ghi nợ phải âm; nó còn tràn ở khoảng 4,29 tỷ đồng. Dùng <code>i64</code>.</p></details>

Bài trước: [bài 02](/articles/rust-cargo-tu-source-code-den-but-toan-dau-tien). Bài tiếp: [bài 04](/articles/rust-ownership-move-borrow-but-toan).

## Tài liệu gốc

- [The Rust Book: Data Types](https://doc.rust-lang.org/book/ch03-02-data-types.html), gồm phần integer overflow.
- [Rust Reference: Numeric types và `as` cast](https://doc.rust-lang.org/reference/expressions/operator-expr.html#numeric-cast).
- [std: `i64`](https://doc.rust-lang.org/std/primitive.i64.html) (checked, saturating, wrapping), [`TryFrom`](https://doc.rust-lang.org/std/convert/trait.TryFrom.html).
- [Cargo Book: Profiles, `overflow-checks`](https://doc.rust-lang.org/cargo/reference/profiles.html#overflow-checks).
- [docs.rs: `rust_decimal`](https://docs.rs/rust_decimal), gồm `RoundingStrategy` và `Decimal`.
