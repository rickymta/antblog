---
slug: rust-newtype-value-object-money-account-id
title: "Newtype và value object: Money, AccountId, EntryId không nhầm được"
summary: "Bài 12: bọc i64 và chuỗi thành Money, AccountId, EntryId bằng newtype; dựng từ chuỗi bằng FromStr/TryFrom, chọn derive đúng, cộng trừ tiền trả Result và chặn cộng khác loại tiền bằng phantom type."
category: programming
level: beginner
tags: [rust, toi-noi, ledger, newtype, value-object, traits]
language: vi
---

Trong sổ cái, nhiều thứ khác nhau lại chung một kiểu thô: số tiền `25500` và số km `5` đều là `i64`, mã bút toán `JE-000001` và mã tài khoản `driver:DR-001` đều là `String`. Nếu hàm ghi sổ nhận các kiểu thô đó, đổi chỗ hai đối số cùng kiểu vẫn biên dịch, và `25500 + 5` ra `25505` mà không báo gì cho đến lúc đối soát lệch. Bài này dời loại lỗi đó từ lúc chạy sang lúc biên dịch.

Đây là bài 12 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài này dùng lại trait và generics của [bài 11](/articles/rust-trait-generics-chinh-sach-phi-lam-tron): `Add`, `FromStr`, `TryFrom` là trait của thư viện chuẩn, `Money<C>` là kiểu generic. Lab đi tiếp từ lab bài 10, chỉ dùng thư viện chuẩn, và cố ý đổi vài quyết định của lab đó (mục «Lab này đổi gì»).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Cộng <code>Money&lt;Vnd&gt;</code> với <code>Money&lt;Usd&gt;</code> thì lỗi xảy ra lúc nào, và <code>Money&lt;Usd&gt;</code> tốn thêm bao nhiêu byte so với <code>i64</code>?</p>
<details><summary>Xem đáp án</summary><p>Lỗi xảy ra lúc <b>biên dịch</b> (E0308, <code>mismatched types</code>), chương trình không hề chạy. Loại tiền chỉ nằm ở kiểu nên không tốn byte nào: <code>size_of::&lt;Money&lt;Usd&gt;&gt;()</code> bằng <code>size_of::&lt;i64&gt;()</code>, test trong lab kiểm điều này.</p></details>
</div>

## Newtype là gì, và vì sao `Money` khác `i64`?

Newtype là một struct tuple chỉ có một trường, ví dụ `struct Km(u32);`. Lúc chạy nó có cùng kích thước với trường bên trong, không thêm chi phí. Với trình biên dịch nó là một kiểu hoàn toàn mới: mọi phép toán của `u32` biến mất, trừ những phép ta viết lại. Lab có ba newtype giá trị: `Money`, `EntryId` và `Km`, cộng `AccountId` bọc chuỗi. Trường để private, nên ngoài module không dựng được giá trị mà bỏ qua hàm tạo (bài 10 đã dùng điều này).

Trong thiết kế hướng miền, kiểu như vậy là **value object**: bất biến, hợp lệ ngay từ lúc tạo, so sánh theo giá trị; khác với entity như chuyến `TN-0001`, nơi danh tính mới quan trọng. Chữ ký hàm tính cước đã nói hết về đơn vị:

```rust
// src/km.rs (rút gọn)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Km(u32);

/// Cước: 12.000 đồng cho 2 km đầu, 4.500 đồng mỗi km vượt. Nhận `Km`, trả `Money`.
pub fn trip_fare(distance: Km) -> Money {
    let extra_km = i64::from(distance.get().saturating_sub(2));
    Money::from_dong(12_000 + 4_500 * extra_km)
}
```

Trong một crate nháp phụ thuộc vào lab, các phép trộn kiểu đều bị từ chối. Lỗi cho `Money::from_dong(12_000) + Km::new(5)` (rút gọn):

```text
error[E0308]: mismatched types
3 |     let _ = Money::from_dong(12_000) + Km::new(5);
  |             ------------------------   ^^^^^^^^^^ expected `Money`, found `Km`
```

Cộng `Money` với `i64` trần hay với `EntryId` cũng cho E0308; gán `EntryId` vào chỗ cần `AccountId` cũng vậy. Lab giữ bốn ca thành doc test `compile_fail,E0308`: `Money + Km`, `Money + i64`, `Money<Vnd> + Money<Usd>` và gán `EntryId` vào `AccountId`. Trên stable, rustdoc chỉ kiểm đoạn code không biên dịch được, không đối chiếu mã lỗi (đổi thành `compile_fail,E0599` vẫn pass); mã E0308 trong bài lấy từ thông điệp thật của rustc trong crate nháp.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 1. Ba newtype Money, EntryId, Km đều là số nguyên trong bộ nhớ nhưng là ba kiểu khác nhau; trộn chúng trong phép cộng, so sánh là lỗi biên dịch E0308."><rect x="20" y="30" width="260" height="96" rx="10" class="f-phys-soft s-phys"/><text x="36" y="60" class="t-b">Money&lt;Vnd&gt;</text><text x="36" y="84" class="t-small">i64 đồng · Copy</text><text x="36" y="106" class="t-small">trường private, loại tiền ở kiểu</text><rect x="310" y="30" width="260" height="96" rx="10" class="f-pk-soft s-pk"/><text x="326" y="60" class="t-b">EntryId</text><text x="326" y="84" class="t-small">u64 số thứ tự · Copy</text><text x="326" y="106" class="t-small">in ra dạng JE-000001</text><rect x="600" y="30" width="260" height="96" rx="10" class="f-phys-soft s-phys"/><text x="616" y="60" class="t-b">Km</text><text x="616" y="84" class="t-small">u32 quãng đường · Copy</text><text x="616" y="106" class="t-small">không cộng được với Money</text><text x="295" y="84" text-anchor="middle" class="t-b">≠</text><text x="585" y="84" text-anchor="middle" class="t-b">≠</text><rect x="20" y="150" width="840" height="64" rx="10" class="f-warn-soft s-warn"/><text x="36" y="176" class="t-b">Trong bộ nhớ đều là số nguyên (size_of::&lt;Money&gt;() == 8)</text><text x="36" y="198" class="t-small">Phép +, == và &lt; giữa hai loại khác nhau không biên dịch: E0308 mismatched types</text></svg>
</div>
<figcaption>Hình 1. Ba newtype cùng là số nguyên khi chạy nhưng là ba kiểu khác nhau khi biên dịch. Ranh giới nằm ở trình biên dịch, không tốn byte nào.</figcaption>
</figure>

## `From`, `TryFrom` hay `FromStr`: dựng giá trị từ bên ngoài thế nào?

Quy tắc chọn: `From` khi mọi đầu vào đều hợp lệ; `TryFrom` khi đầu vào có thể sai; `FromStr` khi đầu vào là chuỗi và ta muốn gọi `.parse()`.

| Chuyển đổi | Trait | Lý do |
| --- | --- | --- |
| `&str` → `AccountId` | `FromStr`, `TryFrom<&str>` | Chuỗi từ API có thể sai dạng |
| `u64` → `EntryId` | `TryFrom<u64>` | Số thứ tự `0` bị từ chối |
| `i128` → `Money` | `TryFrom<i128>` | Kết quả tính rộng có thể vượt `i64` (bài 03) |
| `AccountId` → `String` | `From` | Bỏ lớp bọc không bao giờ lỗi |
| `i64` → `Money` | cố ý không có | `5_i64.into()` che mất đơn vị; dùng `Money::from_dong(5)` |

`AccountId` kiểm bốn điều: không rỗng, không quá 64 ký tự, chỉ gồm `A-Z a-z 0-9 - :`, và có dạng `kind:tên` với mọi đoạn không rỗng. Lab từ chối khoảng trắng thừa thay vì cắt đi như lab bài 10: hai chuỗi khác nhau cùng cho ra một mã sẽ che lỗi của bên gọi.

```rust
// src/account.rs (rút gọn: bỏ hai nhánh Empty và TooLong)
fn from_str(s: &str) -> Result<AccountId, IdError> {
    if let Some(ch) = s
        .chars()
        .find(|c| !(c.is_ascii_alphanumeric() || *c == '-' || *c == ':'))
    {
        return Err(IdError::InvalidChar { ch });
    }
    let segments = s.split(':').count();
    if segments < 2 || s.split(':').any(str::is_empty) {
        return Err(IdError::BadShape);
    }
    Ok(AccountId(s.to_string()))
}
```

`EntryId` lưu số thứ tự `u64` và chỉ nhận **dạng chuẩn**: `JE-000001` parse được, còn `JE-1`, `JE-0000001` hay `JE-+00001` thì không, vì `u64::from_str` nhận cả dấu `+` và số 0 đầu. Bước cuối của `from_str` in lại id rồi so với chuỗi gốc:

```rust
// src/entry_id.rs (rút gọn)
let id = EntryId::try_from(sequence)?;
// Chỉ nhận dạng chuẩn: "JE-1" parse được số nhưng không phải mã đã in ra.
if id.to_string() != s {
    return Err(IdError::BadEntryId);
}
```

Lưu `u64` thay vì chuỗi cũng cho thứ tự đúng: so sánh số thì `JE-999999` đứng trước `JE-1000000`, so sánh chuỗi cho kết quả ngược.

## `derive` nào cho kiểu nào, và `Copy` khi nào?

| Trait | `Money<C>` | `AccountId` | `EntryId` |
| --- | --- | --- | --- |
| `Clone` | có | có | có |
| `Copy` | có | **không** (chứa `String`) | có |
| `PartialEq`, `Eq` | có | có | có |
| `Hash` | có | có | có |
| `PartialOrd`, `Ord` | có | có | có |
| `Debug` | viết tay | derive | derive |

`Eq` được vì không có `f64`, nên không có NaN phá quan hệ bằng nhau. `Hash` đi cùng `Eq`: hai giá trị bằng nhau phải cho cùng hash, nên `AccountId` làm khóa `HashMap` được. `Ord` cho phép `BTreeMap<AccountId, Money>` (`Ledger::balances` trả số dư xếp theo mã tài khoản) và sắp xếp `EntryId`.

`Copy` nên dành cho kiểu nhỏ, không sở hữu tài nguyên, sao chép bằng vài byte: `Money` và `EntryId` được. `AccountId` giữ `String` trên heap nên chỉ `Clone`; thử dùng sau khi move thì gặp đúng lỗi của bài 04 (rút gọn):

```text
error[E0382]: borrow of moved value: `id`
  |     let id: AccountId = "driver:DR-001".parse().unwrap();
  |         -- move occurs because `id` has type `AccountId`, which does not implement the `Copy` trait
  |     let moved = id;
  |                 -- value moved here
```

Đừng derive `Copy` cho đủ bộ: gỡ nó đi sau này làm vỡ mọi chỗ đã dùng bản sao ngầm. Hai chi tiết riêng của `Money<C>`: `derive` chỉ sinh impl khi `C` cũng có trait đó, nên `Vnd` và `Usd` phải derive đủ bộ; và `Debug` viết tay để in `Money(25500 VND)` thay vì cả `PhantomData<…>`.

## `Add` và `Sub`: panic, `Option` hay `Result`?

Viết `Add` ngây thơ (`Money(self.0 + rhs.0)`) là cách ngắn nhất, và sai nhất với sổ cái. Trong một crate nháp, `Money(i64::MAX) + Money(1)` ở debug panic `attempt to add with overflow`; ở release âm thầm quay vòng ra `Money(-9223372036854775808)`. Panic dừng luồng xử lý giữa chừng một bút toán; quay vòng ghi sai tiền mà không ai biết.

Lab chọn: `Add` và `Sub` có `Output = Result<Money<C>, MoneyError>`, cài bằng `checked_add`/`checked_sub`. `Option` bị loại vì không nói lý do; hàm `checked_*` trả `Option` vẫn có, như thư viện chuẩn.

```rust
// src/money.rs
impl<C: Currency> Add for Money<C> {
    type Output = Result<Money<C>, MoneyError>;

    fn add(self, rhs: Money<C>) -> Self::Output {
        self.checked_add(rhs).ok_or(MoneyError::Overflow)
    }
}
```

Giá phải trả là `a + b + c` không biên dịch, vì `Result<Money, _>` không biết cộng với `Money` (E0369: cannot add `Money` to `Result<Money, MoneyError>`). Cũng không có `+=` (`AddAssign`) vì nó không báo lỗi được. Cộng nhiều số thì viết từng bước với `?`, như hàm `total` của sổ cái:

```rust
// src/entry.rs (rút gọn)
for p in postings {
    sum = (sum + p.amount)?;
}
```

`?` ở đây đổi `MoneyError` thành `LedgerError` nhờ `impl From<MoneyError> for LedgerError` (bài 17 dùng `thiserror` thay cho các `impl Display` và `Error` viết tay). Một số thư viện chỉ cung cấp `checked_add`, không impl `Add`; hướng đó cũng ổn, điều cần tránh là panic hoặc quay vòng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Phép a + b trên hai Money cùng loại tiền gọi checked_add; kết quả là Ok(Money) hoặc Err(MoneyError::Overflow), và dấu ? đổi lỗi tiền thành LedgerError::Overflow nhờ impl From."><defs><marker id="rust-newtype-value-object-money-account-id-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="105" width="200" height="76" rx="10" class="f-phys-soft s-phys"/><text x="36" y="136" class="t-b">a + b</text><text x="36" y="160" class="t-small">hai Money cùng loại tiền</text><rect x="290" y="105" width="200" height="76" rx="10" class="f-phys-soft s-phys"/><text x="306" y="136" class="t-b">checked_add</text><text x="306" y="160" class="t-small">i64::checked_add bên trong</text><rect x="570" y="30" width="290" height="70" rx="10" class="f-pk-soft s-pk"/><text x="586" y="60" class="t-b">Ok(Money)</text><text x="586" y="84" class="t-small">tổng nằm trong i64</text><rect x="570" y="130" width="290" height="70" rx="10" class="f-warn-soft s-warn"/><text x="586" y="160" class="t-b">Err(MoneyError::Overflow)</text><text x="586" y="184" class="t-small">tràn: không panic, không quay vòng</text><rect x="570" y="230" width="290" height="56" rx="10" class="f-none s-warn dash"/><text x="586" y="254" class="t-b">? + impl From</text><text x="586" y="274" class="t-small">thành LedgerError::Overflow</text><path d="M220 143 L290 143" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-newtype-value-object-money-account-id-0-a)"/><path d="M490 130 L570 68" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-newtype-value-object-money-account-id-0-a)"/><path d="M490 156 L570 165" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-newtype-value-object-money-account-id-0-a)"/><path d="M715 200 L715 230" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-newtype-value-object-money-account-id-0-a)"/></svg>
</div>
<figcaption>Hình 2. Phép cộng không còn hai kết cục "panic hoặc quay vòng". Tràn số là một giá trị <code>Err</code> mà người gọi phải xử lý hoặc đẩy lên bằng <code>?</code>.</figcaption>
</figure>

## Phantom type: loại tiền nằm ở kiểu

`Money<Vnd>` và `Money<Usd>` là hai kiểu khác nhau. Tham số `C` chỉ là nhãn: `Vnd` và `Usd` là `enum` rỗng (không bao giờ tạo được giá trị) và `PhantomData<C>` là trường không chiếm byte nào để báo cho compiler rằng struct "dùng" `C`. Nói chặt, `Money<C>` có hai trường có tên nên không còn là struct tuple, nhưng lúc chạy vẫn chỉ là một `i64`.

```rust
// src/money.rs (rút gọn)
pub trait Currency {
    const CODE: &'static str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Vnd {}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Money<C: Currency = Vnd> {
    amount: i64,
    currency: PhantomData<C>,
}
```

Một: ràng buộc `C: Currency` ngay trên struct nên `Money<String>` bị E0277 (the trait bound `String: Currency` is not satisfied). Hai: tham số mặc định `= Vnd` giữ nguyên các chữ ký cũ: `Posting { amount: Money }` vẫn nghĩa là VND, và `Money::from_dong` chỉ có trong `impl Money<Vnd>`. Ba: `Add` được cài cho `Money<C>` với cùng `C` ở cả hai vế, nên `vnd + usd` ra E0308 (rút gọn):

```text
error[E0308]: mismatched types
5 |     let _ = vnd + usd;
  |                   ^^^ expected `Money`, found `Money<Usd>`
  = note: expected struct `Money<Vnd>`
             found struct `Money<Usd>`
```

So sánh `vnd < usd` cũng vậy, vì `derive(PartialOrd)` chỉ sinh `PartialOrd` giữa hai `Money<C>` cùng `C`.

Cái giá là thêm tham số generic trong chữ ký. Nếu hệ thống chỉ có VND, một `Money` không generic đã đủ; phantom type đáng dùng khi loại tiền thật sự có nguy cơ lẫn nhau.

## Lab này đổi gì so với lab 10, và các bài sau giữ gì?

Mỗi lab là một crate độc lập. Bốn thay đổi dưới đây là lựa chọn của riêng lab này, không phải API chung của loạt bài:

| Điểm | Lab 10 | Lab 12 | Lab 13 và `ledger-core` (bài 20) |
| --- | --- | --- | --- |
| Dựng `AccountId` | `AccountId::new`, cắt khoảng trắng | `parse`, `TryFrom<&str>`, từ chối khoảng trắng, kiểm dạng | `AccountId::new` như lab 10 |
| `Posted.id` | `String` | `EntryId` | `EntryId` |
| `Money` | Không generic | `Money<C>`, mặc định `Vnd` | Không generic, chỉ VND |
| Cộng trừ tiền | `checked_add` trả `Option` | Thêm `Add`, `Sub` trả `Result` | `checked_*` trả `Option`, không có `Add` |

[Bài 13](/articles/rust-double-entry-journal-entry-posting) và [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace) giữ dạng gọn vì lõi sổ cái chỉ có VND, nên phantom type không đáng cái giá của nó. `EntryId` là thứ đi tiếp.

## Với người từ C#: `record Money` và newtype

Bài [Record, struct và immutability](/articles/csharp-record-struct-money-quote) dựng `Money` là `sealed record` với `Amount`, `Currency`, và `DistanceKm` là `readonly record struct`. Đối chiếu:

| Ý | C# | Rust trong lab |
| --- | --- | --- |
| Value object | `sealed record Money` (class) hoặc `readonly record struct` | `struct Money<C>` bọc `i64`, trường private |
| So sánh theo giá trị | compiler sinh `Equals`, `==`, `GetHashCode` | `derive(PartialEq, Eq, Hash)` |
| Loại tiền | trường `string`, constructor kiểm lúc chạy và ném exception | tham số kiểu, kiểm lúc biên dịch |
| Toán tử `+` khi tràn | `decimal` luôn ném `OverflowException`; `long` chỉ ném trong ngữ cảnh `checked` | trả `Result`, không ném |
| Giá trị mặc định | struct: `default` luôn tồn tại, đi vòng qua constructor; class: `default` là `null` | không có `Default`, chỉ tạo qua hàm tạo |
| Chuyển đổi | `implicit/explicit operator`, `Parse`/`TryParse` | `From`, `TryFrom`, `FromStr` |

C# cũng dựng được `Money<TCurrency>` bằng generic. Khác biệt đáng nhớ là giá trị mặc định: với struct C#, `default` luôn dựng được giá trị không qua kiểm tra (bài trên đã thử với `DistanceKm`), còn Rust chỉ có `Default` khi ta tự khai báo. `decimal` của bài [kiểu dữ liệu và decimal](/articles/csharp-bien-kieu-du-lieu-decimal) được thay bằng `i64` đồng, vì VND không có phần lẻ.

## Chạy ví dụ và kiểm chứng

Từ gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/12-newtype-money-account-id/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/12-newtype-money-account-id/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/12-newtype-money-account-id/Cargo.toml
```

Lệnh đầu in:

```text
== Mã tài khoản: parse có kiểm tra ==
  "customer:CU-001" -> hợp lệ: customer:CU-001
  "psp:vixanh:clearing" -> hợp lệ: psp:vixanh:clearing
  "customer" -> mã tài khoản phải dạng kind:tên, không đoạn rỗng
  " driver:DR-001" -> ký tự ' ' không được phép
  "customer::CU-001" -> mã tài khoản phải dạng kind:tên, không đoạn rỗng
  "" -> mã rỗng
== Tiền: kiểu riêng, cộng trừ có kiểm tra ==
  5.100 VND + 20.400 VND = 25.500 VND
  cước chuyến 5 km = 25.500 VND
  9.223.372.036.854.775.807 VND + 1 VND -> lỗi: phép tính tiền vượt giới hạn i64
  9223372036854775808 -> lỗi: giá trị nằm ngoài phạm vi i64
  Debug: Money(25500 VND) và Money(100 USD) là hai kiểu khác nhau
== Mã bút toán ==
  JE-000001 có số thứ tự 1
  "JE-000042" -> JE-000042
  "JE-42" -> mã bút toán phải dạng JE-000001, số thứ tự từ 1
  "TN-000001" -> mã bút toán phải dạng JE-000001, số thứ tự từ 1
== Sổ cái ==
  JE-000001 trip:TN-0001:capture (3 posting, replayed = false)
  JE-000001 ghi lại: replayed = true
  Số dư (xếp theo mã tài khoản):
    customer:CU-001    -25.500 VND
    driver:DR-001       20.400 VND
    platform:revenue     5.100 VND
  Tổng toàn sổ: 0 VND
```

`-- --verify` kiểm các giá trị trên cùng các ca lỗi và in `VERIFY OK` với exit code 0. `cargo test` báo 30 unit test, 4 integration test và 5 doc test (một ví dụ chạy được, bốn `compile_fail`) đều pass; clippy (`--all-targets -- -D warnings`) và `cargo fmt --check` sạch.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Sổ cái vẫn nằm trong bộ nhớ. <code>Money</code> chỉ có cộng trừ: chưa có nhân chia (phí 20% dùng <code>rust_decimal</code>, xem bài 03 và bài 11), chưa có đổi tiền theo tỷ giá, và <code>Usd</code> chỉ để minh họa kỹ thuật: số trong <code>Money&lt;Usd&gt;</code> là cent mà <code>Display</code> chưa biết. Quy tắc <code>AccountId</code> chỉ kiểm hình dạng, chưa kiểm loại tài khoản có tồn tại. <code>Add</code> trả <code>Result</code> là lựa chọn của lab, không phải chuẩn của hệ sinh thái.</p>
</div>

**Bài tập:** viết `checked_neg` cho `Money<C>` (lab 11 có bản cho `Money` không generic) để dựng bút toán đảo ở bài sau. Vì sao `Money::from_dong(i64::MIN)` không phủ định được, và kiểu trả về phải là gì?

<details><summary>Xem lời giải</summary><p>Trong số nguyên bù hai, <code>i64::MIN</code> (−9.223.372.036.854.775.808) không có số đối trong <code>i64</code>, vì <code>i64::MAX</code> chỉ là 9.223.372.036.854.775.807. Kiểu trả về phải có chỗ cho thất bại: <code>Option&lt;Money&lt;C&gt;&gt;</code>. <code>m.amount().checked_neg().map(Money::new)</code> cho <code>Some(Money(-25500 VND))</code> với 25.500 và <code>None</code> với <code>i64::MIN</code>.</p></details>

Bài trước: [bài 11](/articles/rust-trait-generics-chinh-sach-phi-lam-tron). Bài tiếp: [bài 13](/articles/rust-double-entry-journal-entry-posting).

## Tài liệu gốc

- [The Rust Book: Type Safety and Abstraction with the Newtype Pattern](https://doc.rust-lang.org/book/ch20-03-advanced-types.html).
- [std::ops::Add](https://doc.rust-lang.org/std/ops/trait.Add.html).
- [std::convert::TryFrom](https://doc.rust-lang.org/std/convert/trait.TryFrom.html).
- [std::str::FromStr](https://doc.rust-lang.org/std/str/trait.FromStr.html).
- [std::marker::PhantomData](https://doc.rust-lang.org/std/marker/struct.PhantomData.html).
- [The Rust Reference: Derive](https://doc.rust-lang.org/reference/attributes/derive.html).
