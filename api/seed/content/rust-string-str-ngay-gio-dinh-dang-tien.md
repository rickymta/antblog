---
slug: rust-string-str-ngay-gio-dinh-dang-tien
title: "String, &str, ngày giờ UTC và định dạng tiền: hiển thị đúng, lưu đúng"
summary: "Bài 08: String sở hữu, &str mượn; UTF-8 với len() là byte; tách mã tài khoản bằng split_once; impl Display in 25.500 VND; jiff lưu Timestamp UTC và hiển thị giờ Asia/Ho_Chi_Minh."
category: rust
level: beginner
tags: [rust, toi-noi, ledger, string, jiff, utf-8]
language: vi
---

Màn hình thu nhập của tài xế ghi «Tài xế Nguyễn Văn An nhận 20.400 VND lúc 09:30 06/11/2026». Trong sổ cái, cùng bút toán chỉ là số nguyên `20400` và một thời điểm `2026-11-06T02:30:00Z`. Chữ có dấu, dấu chấm phân nhóm và giờ +07:00 chỉ tồn tại ở lớp hiển thị. Lẫn hai lớp này là cách nhanh nhất để lưu sai.

Đây là bài 08 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Sau [bài 07](/articles/rust-result-option-toan-tu-hoi-loi-la-gia-tri), lỗi đã là giá trị; hôm nay ta xử lý văn bản và thời gian, hai nơi dữ liệu dễ đổi nghĩa khi qua biên. Ví dụ chạy với Rust 1.98, edition 2024 và `jiff` 0.2; mỗi bài là một crate độc lập.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Chuỗi <code>"Tài xế"</code> có <code>len()</code> bằng 6 hay 9?</p>
<details><summary>Xem đáp án</summary><p><code>len()</code> là số byte UTF-8, bằng 9. Số ký tự (Unicode scalar value) là 6, lấy bằng <code>chars().count()</code>.</p></details>
</div>

## String sở hữu, &str mượn: khác nhau ở đâu?

`String` là chuỗi UTF-8 nằm trên heap, có chủ sở hữu duy nhất và co giãn được. Trên stack nó chỉ là ba từ: con trỏ, capacity và độ dài (24 byte trên máy 64-bit). `&str` là một lát cắt mượn: con trỏ và độ dài (16 byte), không sở hữu gì. Literal `"Tài xế"` có kiểu `&'static str`, dữ liệu nằm sẵn trong binary. Quy tắc ownership của [bài 04](/articles/rust-ownership-move-borrow-but-toan) áp dụng nguyên vẹn: `String` bị move, `&str` được mượn và không được sống lâu hơn chủ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 1. String trên stack gồm con trỏ, capacity và len, trỏ tới buffer 9 byte UTF-8 trên heap; một &str chỉ gồm con trỏ và len, trỏ vào cùng buffer đó."><defs><marker id="rust-string-str-ngay-gio-dinh-dang-tien-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="24" class="t-small">Stack</text><text x="320" y="24" class="t-small">Heap: 9 byte UTF-8 của «Tài xế»</text><rect x="20" y="40" width="250" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="70" class="t-b">owned: String</text><text x="36" y="95" class="t-small">ptr · cap = 9 · len = 9</text><text x="36" y="120" class="t-small">24 byte, sở hữu buffer</text><rect x="20" y="170" width="250" height="100" rx="10" class="f-pk-soft s-pk"/><text x="36" y="200" class="t-b">part: &amp;str</text><text x="36" y="225" class="t-small">ptr · len = 4</text><text x="36" y="250" class="t-small">16 byte, chỉ mượn</text><rect x="320" y="60" width="60" height="50" class="f-phys-soft s-phys"/><text x="350" y="90" class="t-mono" text-anchor="middle">54</text><rect x="380" y="60" width="60" height="50" class="f-phys-soft s-phys"/><text x="410" y="90" class="t-mono" text-anchor="middle">c3</text><rect x="440" y="60" width="60" height="50" class="f-phys-soft s-phys"/><text x="470" y="90" class="t-mono" text-anchor="middle">a0</text><rect x="500" y="60" width="60" height="50" class="f-phys-soft s-phys"/><text x="530" y="90" class="t-mono" text-anchor="middle">69</text><rect x="560" y="60" width="60" height="50" class="f-phys-soft s-phys"/><text x="590" y="90" class="t-mono" text-anchor="middle">20</text><rect x="620" y="60" width="60" height="50" class="f-phys-soft s-phys"/><text x="650" y="90" class="t-mono" text-anchor="middle">78</text><rect x="680" y="60" width="60" height="50" class="f-phys-soft s-phys"/><text x="710" y="90" class="t-mono" text-anchor="middle">e1</text><rect x="740" y="60" width="60" height="50" class="f-phys-soft s-phys"/><text x="770" y="90" class="t-mono" text-anchor="middle">ba</text><rect x="800" y="60" width="60" height="50" class="f-phys-soft s-phys"/><text x="830" y="90" class="t-mono" text-anchor="middle">bf</text><text x="350" y="130" class="t-small" text-anchor="middle">T</text><text x="440" y="130" class="t-small" text-anchor="middle">à (2 byte)</text><text x="530" y="130" class="t-small" text-anchor="middle">i</text><text x="590" y="130" class="t-small" text-anchor="middle">cách</text><text x="650" y="130" class="t-small" text-anchor="middle">x</text><text x="770" y="130" class="t-small" text-anchor="middle">ế (3 byte)</text><path d="M270 85 L316 85" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-string-str-ngay-gio-dinh-dang-tien-0-a)"/><path d="M320 160 L560 160 M320 154 L320 166 M560 154 L560 166" class="f-none s-pk" stroke-width="2"/><path d="M270 215 L318 164" class="f-none s-pk" stroke-width="2" marker-end="url(#rust-string-str-ngay-gio-dinh-dang-tien-0-a)"/><text x="330" y="190" class="t-small">part = &amp;owned[0..4] = «Tài», không cấp phát thêm</text><text x="320" y="250" class="t-small">Sáu ký tự nhưng chín byte: len() trả 9, chars().count() trả 6.</text></svg>
</div>
<figcaption>Hình 1. <code>String</code> trên stack trỏ tới buffer trên heap; <code>&amp;str</code> trỏ vào chính buffer đó, không sao chép.</figcaption>
</figure>

Mã `src/main.rs` của lab (rút gọn):

```rust
let literal: &str = "Tài xế"; // &'static str nằm trong binary
let mut owned: String = String::from(literal); // copy sang heap
let also_owned = literal.to_owned(); // cùng kết quả, nói rõ ý "tạo bản sở hữu"
let via_display = 25_500.to_string(); // mọi kiểu có Display đều có to_string()
owned.push_str(" Nguyễn Văn An"); // nối vào cuối, có thể cấp phát lại
println!("{}", greeting(literal)); // &'static str
println!("{}", greeting(&owned)); // &String tự đổi thành &str (deref coercion)
```

`String::from`, `to_owned` và `to_string` đều tạo `String` mới; `format!` làm điều đó khi cần ghép nhiều mảnh. Hàm `greeting(name: &str)` nhận được cả literal lẫn `&String` nhờ **deref coercion**: compiler tự đổi `&String` thành `&str`. Vì vậy quy tắc thực dụng là: tham số nhận `&str`, kết quả tạo mới trả `String`. Viết `name: &String` chỉ làm hàm kén đối số hơn.

## Vì sao len() không phải số ký tự, và cắt chuỗi có thể panic?

Rust lưu chuỗi dạng UTF-8: ký tự ASCII chiếm 1 byte, `à` 2 byte, `ế` 3 byte. Lab in ra với tên tài xế (hư cấu):

```text
"Tài xế Nguyễn Văn An": len() = 26 byte, chars().count() = 20 ký tự
(0,'T') (1,'à') (3,'i') (4,' ') (5,'x') (6,'ế')
get(0..2) = None, get(0..3) = Some("Tà")
```

`len()` là byte, `chars()` duyệt từng ký tự, `char_indices()` cho kèm vị trí byte, như dòng thứ hai: `i` bắt đầu ở byte 3 vì `à` chiếm byte 1 và 2. Cắt `&driver[0..2]` rơi vào giữa `à`, và Rust không trả về nửa ký tự. Nó panic ở thời gian chạy:

```text
end byte index 2 is not a char boundary; it is inside 'à' (bytes 1..3 of string)
```

Muốn không panic, dùng `get(0..2)` (trả `None`) hoặc `is_char_boundary`, hoặc tìm vị trí bằng `find("Nguyễn")` rồi mới cắt. Index bằng số nguyên thì bị từ chối ngay khi compile, vì vị trí byte không trùng thứ tự ký tự:

```text
error[E0277]: the type `str` cannot be indexed by `{integer}`
```

Hai giới hạn cần biết. Một: `chars()` đếm Unicode scalar value, không phải «chữ người đọc thấy». Hai: `à` gõ dạng một code point và `a` cộng dấu huyền là hai chuỗi byte khác nhau. Lab kiểm `"\u{e0}" != "a\u{300}"`. std không chuẩn hóa; việc đó cần crate ngoài như `unicode-normalization`, bài này không dùng, nên chuẩn hóa ở biên nhập trước khi so sánh hoặc dùng làm khóa.

## Tách mã tài khoản và đọc số mà không sao chép

Mã tài khoản `customer:CU-001` gồm loại và định danh. `split_once(':')` tách ở dấu hai chấm **đầu tiên** và trả hai `&str` trỏ vào chuỗi gốc, không cấp phát:

Trong `src/main.rs`:

```rust
fn parse_account(code: &str) -> Result<(&str, &str), String> {
    let (kind, id) = code
        .split_once(':')
        .ok_or_else(|| format!("mã tài khoản thiếu dấu ':' : {code:?}"))?;
    if kind.is_empty() || id.is_empty() {
        return Err(format!("thành phần rỗng trong {code:?}"));
    }
    Ok((kind, id))
}
```

Lỗi là giá trị `Result` như bài 07. Kết quả hai `&str` sống cùng đầu vào; cách Rust ghi nhớ điều đó (lifetime) là chủ đề của [bài 09](/articles/rust-vec-hashmap-btreemap-lifetime-so-cai-bo-nho). Với `psp:vixanh:clearing`, kết quả là `("psp", "vixanh:clearing")`: tách một lần, phần còn lại giữ nguyên. Các hàm kiểm tra cạnh nó là `starts_with`, `split_whitespace`, `trim`.

Đọc số dùng `parse::<i64>()`, trả `Result` thay vì ném exception. `"25500".parse::<i64>()` là `Ok(25500)`, còn `"25.500"` là `Err(ParseIntError { kind: InvalidDigit })`. Đó là chủ ý: chuỗi hiển thị `25.500` không phải dữ liệu. Hợp đồng giữa service là số `25500` kèm `"currency": "VND"`; chuỗi «25.500 VND» không bao giờ được parse ngược.

## In «25.500 VND» khi std không có locale

Rust không có «culture» ngầm như .NET: `{}` in số nguyên không nhóm hàng nghìn, không phụ thuộc máy chạy, nên không có chuyện cùng đoạn code in `25.500` ở máy này và `25,500` ở máy khác. Đổi lại ta tự viết định dạng, và đó là nơi dễ sai. Xem [bài C# về culture](/articles/csharp-chuoi-ngay-gio-culture) để đối chiếu cách .NET xử lý. Với Rust, bộ định dạng theo locale là crate bên ngoài, lab không dùng.

Trong `src/main.rs`:

```rust
fn format_vnd(amount: i64) -> String {
    // unsigned_abs an toàn cả với i64::MIN (abs() sẽ tràn).
    let digits = amount.unsigned_abs().to_string();
    let sign = if amount < 0 { "-" } else { "" };
    format!("{sign}{} VND", group_thousands(&digits))
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `pad` để `{:>14}` và `{:<14}` hoạt động như với kiểu chuẩn.
        f.pad(&format_vnd(self.0))
    }
}
```

`group_thousands` (đầy đủ trong lab) duyệt chữ số và chèn dấu chấm khi phần còn lại là bội của 3. Số âm dùng dấu trừ ASCII `-` cho thống nhất: `-5.100 VND`. Hai điểm nhỏ nhưng đáng làm: `unsigned_abs` thay `abs` để `i64::MIN` không tràn (bài 03), và `f.pad` để `{:>14}` có tác dụng. Kết quả của lab:

```text
    25500 -> [    25.500 VND]
    -5100 -> [    -5.100 VND]
```

## Lưu UTC, hiển thị giờ Việt Nam

Thời điểm bút toán được ghi (`posted_at`) lưu **UTC**. `jiff::Timestamp` là một thời điểm trên trục thời gian, không mang múi giờ. `Zoned` là Timestamp kèm múi giờ IANA, chỉ dùng khi in cho người đọc.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 2. Một mốc UTC 2026-11-06T02:30:00Z được to_zoned với Asia/Ho_Chi_Minh (+07:00) thành 09:30 ngày 06/11/2026; mốc 17:30Z cùng cách đổi cho ra 00:30 ngày 07/11/2026."><defs><marker id="rust-string-str-ngay-gio-dinh-dang-tien-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="250" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="60" class="t-b">Lưu: Timestamp (UTC)</text><text x="36" y="85" class="t-small">2026-11-06T02:30:00Z</text><text x="36" y="110" class="t-small">posted_at, dùng để sắp xếp</text><rect x="315" y="30" width="250" height="100" rx="10" class="f-pk-soft s-pk"/><text x="331" y="60" class="t-b">to_zoned(tz)</text><text x="331" y="85" class="t-small">Asia/Ho_Chi_Minh</text><text x="331" y="110" class="t-small">+07:00, không có giờ mùa hè</text><rect x="610" y="30" width="250" height="100" rx="10" class="f-phys-soft s-phys"/><text x="626" y="60" class="t-b">Hiển thị: Zoned</text><text x="626" y="85" class="t-small">09:30 ngày 06/11/2026</text><text x="626" y="110" class="t-small">chỉ cho người đọc</text><path d="M270 80 L311 80" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-string-str-ngay-gio-dinh-dang-tien-1-a)"/><path d="M565 80 L606 80" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-string-str-ngay-gio-dinh-dang-tien-1-a)"/><rect x="20" y="160" width="840" height="60" rx="10" class="f-warn-soft s-warn"/><text x="36" y="186" class="t-b">Ngày đổi, thời điểm không đổi</text><text x="36" y="206" class="t-small">2026-11-06T17:30:00Z hiển thị 00:30 ngày 07/11/2026; so sánh hai mốc luôn làm trên Timestamp.</text></svg>
</div>
<figcaption>Hình 2. Một mốc UTC, hiển thị ở <code>Asia/Ho_Chi_Minh</code>. Dữ liệu lưu không đổi khi đổi múi giờ.</figcaption>
</figure>

Trích `src/main.rs` (rút gọn):

```rust
let posted_at: Timestamp = "2026-11-06T02:30:00Z".parse()?;
let local = posted_at.to_zoned(vn.clone()); // vn = TimeZone::get("Asia/Ho_Chi_Minh")?
println!("Zoned: {local}");
println!("Hiển thị: {}", local.strftime("%H:%M ngày %d/%m/%Y"));
```

```text
Zoned: 2026-11-06T09:30:00+07:00[Asia/Ho_Chi_Minh]
Hiển thị: 09:30 ngày 06/11/2026
```

Bốn quy tắc rút ra từ lab:

- **Naive bị từ chối.** `"2026-11-06T09:30:00".parse::<Timestamp>()` trả `Err("failed to find offset component, which is required for parsing a timestamp")`. Chuỗi thiếu `Z` hoặc offset không biết là mốc nào, nên không thành thời điểm một cách âm thầm. API nhận ISO 8601 có offset.
- **Sắp xếp theo `Timestamp`.** `09:30+07:00` là 02:30Z, sớm hơn `03:00+00:00`. So như chuỗi thì kết quả ngược; so `Timestamp` thì đúng. Sổ cái sắp bút toán theo `posted_at` UTC, không sắp theo chuỗi hiển thị.
- **`Span` cộng trừ.** `posted_at.checked_add(90.minutes())` ra `2026-11-06T04:00:00Z`; hiệu hai mốc là `Span` tính bằng giây; `round` với đơn vị lớn nhất là giờ rồi in bằng `{:#}` (dạng «friendly» của jiff) thì đọc được `1h 30m`, còn `{}` in dạng ISO 8601 `PT1H30M`. `Timestamp` từ chối cộng «1 ngày» vì ngày không có độ dài cố định; cần `Zoned`: `local.checked_add(1.day())` ra `2026-11-07T09:30:00+07:00[Asia/Ho_Chi_Minh]`.
- **Tên múi giờ có thể thất bại.** `TimeZone::get` trả `Result`, vì dữ liệu IANA nằm trong hệ điều hành (macOS, Linux: `/usr/share/zoneinfo`) hoặc được gói vào binary (mặc định trên Windows; nơi khác bật feature `tzdb-bundle-always`).

Đối chiếu C#: `Timestamp` gần `DateTimeOffset` đã quy về UTC, và `Zoned` giống `DateTimeOffset` cộng thêm tên múi giờ IANA thay vì chỉ một offset cố định. Quy tắc giống loạt C# (đã nêu ở bài về [culture](/articles/csharp-chuoi-ngay-gio-culture)): lưu một mốc không mơ hồ, đổi sang giờ địa phương ở cửa ra.

## Bảng tra nhanh

| Việc | API | Lưu ý |
| --- | --- | --- |
| Tạo `String` từ literal | `String::from`, `to_owned`, `to_string` | Cấp phát heap; `format!` khi ghép nhiều mảnh |
| Nối chuỗi | `push_str`, `format!` | `push_str` có thể cấp phát lại |
| Nhận chuỗi trong hàm | tham số `&str` | `&String` tự đổi nhờ deref coercion |
| Độ dài | `len()` / `chars().count()` | Byte / Unicode scalar value, không phải chữ người đọc thấy |
| Cắt an toàn | `get(a..b)`, `is_char_boundary`, `find` | `s[a..b]` panic nếu sai biên UTF-8 |
| Tách mã | `split_once(':')`, `starts_with` | Kết quả mượn từ chuỗi gốc |
| Đọc số | `parse::<i64>()` | `Result`; `"25.500"` là lỗi |
| In tiền | `impl Display`, `f.pad` | std không có locale; tự nhóm ba chữ số |
| Mốc thời gian | `Timestamp`, parse ISO 8601 | Cần `Z` hoặc offset; lưu và sắp theo UTC |
| Hiển thị giờ | `to_zoned`, `strftime` | `TimeZone::get` có thể lỗi |

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository, cần `cargo` stable (bài 02) và tzdata của hệ điều hành (macOS và đa số bản Linux có sẵn):

```bash
cargo run --manifest-path examples/toi-noi-rust/08-string-ngay-gio/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/08-string-ngay-gio/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/08-string-ngay-gio/Cargo.toml
```

Lệnh thứ hai in `VERIFY OK` khi `format_vnd(25_500) == "25.500 VND"`, `format_vnd(-5_100) == "-5.100 VND"`, `len()` khác `chars().count()`, cắt `[0..2]` panic thật, và mốc `2026-11-06T02:30:00Z` ra `09:30` ngày `06/11/2026`; sai thì exit code 1. Dòng sổ cái đầu tiên của lab in `JE-000001 | customer:CU-001      |  -25.500 VND | 09:30 06/11/2026`, ba posting cộng lại bằng 0.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Lab chỉ làm VND một ngôn ngữ, không có locale, không chuẩn hóa Unicode, không xử lý múi giờ khác, và <code>Line</code> chỉ là dòng sổ rút gọn để minh họa hiển thị. Dữ liệu múi giờ đến từ hệ điều hành; môi trường container tối giản có thể thiếu tzdata, khi đó cài gói <code>tzdata</code> hoặc bật feature <code>tzdb-bundle-always</code> của <code>jiff</code> để gói dữ liệu IANA vào binary. Chuỗi hiển thị không bao giờ quay lại thành dữ liệu lưu. Tên và số liệu là hư cấu của Tới Nơi.</p>
</div>

**Bài tập:** `Money(-25_500)` in thế nào bằng `{:>14}`? Và `"2026-11-06T17:30:00Z"` hiển thị ngày nào ở Việt Nam, vì sao khác ngày UTC?

<details><summary>Xem lời giải</summary><p><code>format_vnd(-25_500)</code> là <code>-25.500 VND</code>, 11 ký tự, nên <code>{:&gt;14}</code> đệm 3 dấu cách bên trái nhờ <code>f.pad</code>. Mốc 17:30Z cộng 7 giờ thành 00:30 ngày <code>07/11/2026</code>: cùng một thời điểm, nhưng lịch Việt Nam đã sang ngày mới.</p></details>

Bài trước: [bài 07](/articles/rust-result-option-toan-tu-hoi-loi-la-gia-tri). Bài tiếp: [bài 09](/articles/rust-vec-hashmap-btreemap-lifetime-so-cai-bo-nho).

## Tài liệu gốc

- [The Rust Book: Storing UTF-8 Encoded Text with Strings](https://doc.rust-lang.org/book/ch08-02-strings.html).
- [std::string::String](https://doc.rust-lang.org/std/string/struct.String.html) và [str](https://doc.rust-lang.org/std/primitive.str.html).
- [std::fmt: Display và Formatter::pad](https://doc.rust-lang.org/std/fmt/index.html).
- [jiff 0.2 trên docs.rs](https://docs.rs/jiff/0.2/jiff/): `Timestamp`, `Zoned`, `Span`, `tz::TimeZone`, và mục crate features cho `tzdb-bundle-always`.
- [The Rust Reference: kiểu str](https://doc.rust-lang.org/reference/types/textual.html).
