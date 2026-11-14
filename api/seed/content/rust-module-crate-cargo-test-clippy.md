---
slug: rust-module-crate-cargo-test-clippy
title: "Module, crate, cargo test và clippy: tổ chức lab sổ cái"
summary: "Bài 10: tách lab sổ cái thành crate thư viện và binary, chia module, dùng pub/use, viết unit, doc và integration test, chạy clippy và fmt; tổng kết điều phần P01 đã chứng minh và chưa có."
category: rust-beginner
level: beginner
tags: [rust, toi-noi, ledger, module, cargo-test, clippy]
language: vi
---

Chín bài đầu, mỗi lab là một `main.rs` chạy ra đúng số dư. Một sổ cái thật thì khác: mô hình `Money`, `Posting`, `JournalEntry` phải nằm ở nơi nhiều chương trình cùng gọi được, và không ai tin nó chỉ vì "chạy ra đúng một lần". Bài này gom mô hình của bài 03–09 vào một thư viện có cấu trúc, rồi viết test và chạy `clippy` để máy kiểm thay mắt người.

Đây là bài 10 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Lab chỉ dùng thư viện chuẩn, không database, không mạng. Bài liền trước là [bài 09](/articles/rust-vec-hashmap-btreemap-lifetime-so-cai-bo-nho); không cần thuộc hết, nhưng nên đã quen `Result` và `?` ở [bài 07](/articles/rust-result-option-toan-tu-hoi-loi-la-gia-tri).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>File <code>tests/ledger.rs</code> có gọi được hàm <code>pub(crate) fn total</code> của thư viện không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Mỗi file trong <code>tests/</code> là một crate riêng, chỉ thấy item <code>pub</code>. Thử <code>use ledger_lab::entry::total</code> sẽ gặp lỗi E0603: <code>function `total` is private</code>.</p></details>
</div>

## Package, crate và module khác nhau thế nào?

Ba từ này hay bị dùng lẫn. Mỗi từ ở một tầng khác nhau.

| Khái niệm | Là gì | Trong lab |
| --- | --- | --- |
| Package | Thứ có `Cargo.toml`; chứa một hoặc nhiều crate | `ledger_10_module_cargo_test` |
| Crate | Đơn vị `rustc` biên dịch: binary (có `main`) hoặc library | Thư viện `ledger_lab`, binary `ledger-cli`, và mỗi file `tests/*.rs` |
| Module | Cách chia tên và quyền nhìn thấy bên trong một crate | `money`, `account`, `entry`, `error`, `ledger` |

Nếu quen C#: crate gần với một assembly (`.dll` hoặc `.exe`), package gần với một project `.csproj`, module gần với namespace nhưng gắn chặt với file. So sánh chỉ để định hướng, không khớp từng chi tiết.

Tên package và tên crate không bắt buộc trùng. Lab đặt tên thư viện riêng để người gọi viết `ledger_lab::…` cho gọn:

```toml
# Cargo.toml
[lib]
name = "ledger_lab"
path = "src/lib.rs"

[[bin]]
name = "ledger-cli"
path = "src/main.rs"
```

Cả hai mục đều có thể bỏ: cargo tự nhận `src/lib.rs` và `src/main.rs`. Ta ghi ra để nhìn thấy rằng một package có thể chứa cả thư viện lẫn binary.

## Cây module của lab: mỗi `mod` trỏ tới một file

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 310" role="img" aria-label="Hình 1. Một package có ba loại crate: thư viện ledger_lab với năm module mỗi module một file, binary ledger-cli và crate integration test; binary và test chỉ dùng phần pub của thư viện."><defs><marker id="rust-module-crate-cargo-test-clippy-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="10" y="8" width="860" height="294" rx="12" class="f-none s-line dash"/><text x="24" y="30" class="t-b">Package ledger_10_module_cargo_test (một Cargo.toml)</text><rect x="24" y="46" width="520" height="244" rx="10" class="f-phys-soft s-phys"/><text x="40" y="72" class="t-b">Library crate ledger_lab · src/lib.rs</text><rect x="40" y="88" width="150" height="56" rx="8" class="f-surface s-phys"/><text x="52" y="112" class="t-b">money</text><text x="52" y="132" class="t-small">src/money.rs</text><rect x="206" y="88" width="150" height="56" rx="8" class="f-surface s-phys"/><text x="218" y="112" class="t-b">account</text><text x="218" y="132" class="t-small">src/account.rs</text><rect x="372" y="88" width="150" height="56" rx="8" class="f-surface s-phys"/><text x="384" y="112" class="t-b">entry</text><text x="384" y="132" class="t-small">src/entry.rs</text><rect x="40" y="160" width="150" height="56" rx="8" class="f-surface s-phys"/><text x="52" y="184" class="t-b">error</text><text x="52" y="204" class="t-small">src/error.rs</text><rect x="206" y="160" width="150" height="56" rx="8" class="f-surface s-phys"/><text x="218" y="184" class="t-b">ledger</text><text x="218" y="204" class="t-small">src/ledger.rs</text><rect x="372" y="160" width="150" height="56" rx="8" class="f-none s-muted dash"/><text x="384" y="184" class="t-b">pub use</text><text x="384" y="204" class="t-small">re-export ở lib.rs</text><text x="40" y="244" class="t-small">lib.rs khai báo: pub mod money; pub mod account; pub mod entry;</text><text x="40" y="264" class="t-small">pub mod error; pub mod ledger;  (edition 2024: không cần mod.rs)</text><rect x="620" y="46" width="236" height="104" rx="10" class="f-pk-soft s-pk"/><text x="636" y="72" class="t-b">Binary crate ledger-cli</text><text x="636" y="96" class="t-small">src/main.rs · cargo run</text><text x="636" y="118" class="t-small">use ledger_lab::{…};</text><text x="636" y="138" class="t-small">tên crate, không dùng mod</text><rect x="620" y="186" width="236" height="104" rx="10" class="f-warn-soft s-warn"/><text x="636" y="212" class="t-b">Integration test crate</text><text x="636" y="236" class="t-small">tests/ledger.rs</text><text x="636" y="258" class="t-small">use ledger_lab::{…};</text><text x="636" y="278" class="t-small">chỉ thấy item pub</text><path d="M620 98 L546 98" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-module-crate-cargo-test-clippy-0-a)"/><path d="M620 238 L546 238" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-module-crate-cargo-test-clippy-0-a)"/></svg>
</div>
<figcaption>Hình 1. Một package, ba loại crate. Mũi tên là "dùng": binary và test gọi thư viện qua tên crate và chỉ thấy phần <code>pub</code>.</figcaption>
</figure>

`src/lib.rs` là gốc của cây module. Mỗi dòng `pub mod tên;` bảo compiler đọc nội dung từ `src/tên.rs`. Không cần `mod.rs`: cách đặt file này có từ edition 2018 và vẫn là kiểu khuyên dùng ở edition 2024; module con của `money` nếu có sẽ nằm ở `src/money/<tên>.rs`. Đây là `src/lib.rs` rút gọn (bỏ chú thích):

```rust
pub mod account;
pub mod entry;
pub mod error;
pub mod ledger;
pub mod money;

pub use account::AccountId;
pub use entry::{JournalEntry, Posting};
pub use error::LedgerError;
pub use ledger::{Ledger, Posted};
pub use money::Money;
```

Phần `pub use` là re-export: người dùng viết `ledger_lab::Money` thay vì `ledger_lab::money::Money`. Đường dẫn đầy đủ vẫn dùng được.

`LedgerError` được tách ra `error.rs` riêng, thành năm module. Nếu để trong `ledger.rs`, `entry.rs` phải `use` ngược từ `ledger.rs` trong khi `ledger.rs` lại dùng `entry.rs`. Rust cho phép vòng như vậy, nhưng khó đọc. Tách riêng thì chiều phụ thuộc đi một hướng: `money` → `error` → `account` → `entry` → `ledger`.

## `pub`, `pub(crate)` và `use`: ai nhìn thấy gì?

Mặc định mọi thứ là private trong module chứa nó. `pub` mở ra cho mọi crate. `pub(crate)` chỉ mở cho các module trong cùng crate. Hàm `total` trong `src/entry.rs` là ví dụ: `JournalEntry::try_new` dùng nó, và `Ledger::grand_total` ở module khác cũng dùng, nhưng người dùng thư viện thì không cần thấy.

```rust
// src/entry.rs
pub(crate) fn total(postings: &[Posting]) -> Result<Money, LedgerError> {
    let mut sum = Money::ZERO;
    for p in postings {
        sum = sum.checked_add(p.amount).ok_or(LedgerError::Overflow)?;
    }
    Ok(sum)
}
```

Trong `src/ledger.rs`, dòng `use crate::entry::{self, JournalEntry};` cho phép gọi `entry::total(e.postings())?`. `use crate::…` là đường dẫn tuyệt đối từ gốc crate, không phụ thuộc vị trí file hiện tại. `use super::*;` (trong `mod tests`) kéo mọi tên của module cha vào. `use ledger_lab::…` trong `main.rs` là đường dẫn đến crate khác.

Quyền nhìn thấy cũng là công cụ bảo vệ bất biến. `JournalEntry` có trường private, nên bên ngoài chỉ tạo được qua `try_new`, tức chỉ tạo được bút toán đã cân. `Money` cũng giấu trường `i64`: viết `Money(5)` từ crate khác là lỗi biên dịch, và lab ghi lại điều này bằng một doc test `compile_fail`.

`src/main.rs` dùng thư viện như người ngoài. Nó không có dòng `mod money;`, vì làm vậy sẽ biên dịch lại các file kia thành một bản sao riêng trong binary:

```rust
// src/main.rs
use ledger_lab::{AccountId, JournalEntry, Ledger, LedgerError, Money, Posting};
```

## Ba tầng kiểm thử: unit, doc và integration

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Hình 2. Ba tầng test của lab: 16 unit test trong module tests, 2 doc test trong comment ba gạch chéo, 4 integration test trong thư mục tests; càng sang phải càng giống người dùng thật."><rect x="20" y="20" width="270" height="150" rx="10" class="f-phys-soft s-phys"/><text x="36" y="50" class="t-b">Unit test · 16 test</text><text x="36" y="76" class="t-mono">#[cfg(test)] mod tests</text><text x="36" y="98" class="t-small">cùng file với code cần kiểm</text><text x="36" y="120" class="t-small">thấy cả item private</text><text x="36" y="142" class="t-small">use super::*;</text><rect x="305" y="20" width="270" height="150" rx="10" class="f-pk-soft s-pk"/><text x="321" y="50" class="t-b">Doc test · 2 test</text><text x="321" y="76" class="t-mono">/// ``` … ```</text><text x="321" y="98" class="t-small">ví dụ trong comment tài liệu</text><text x="321" y="120" class="t-small">cargo test --doc</text><text x="321" y="142" class="t-small">gọi từ ngoài, gồm compile_fail</text><rect x="590" y="20" width="270" height="150" rx="10" class="f-warn-soft s-warn"/><text x="606" y="50" class="t-b">Integration test · 4 test</text><text x="606" y="76" class="t-mono">tests/ledger.rs</text><text x="606" y="98" class="t-small">mỗi file là một crate riêng</text><text x="606" y="120" class="t-small">chỉ API pub của thư viện</text><text x="606" y="142" class="t-small">use ledger_lab::…;</text><rect x="20" y="192" width="840" height="52" rx="8" class="f-none s-line dash"/><text x="36" y="214" class="t-small">Từ trái sang phải: càng xa bên trong, càng giống người dùng thật của thư viện.</text><text x="36" y="232" class="t-small">cargo test chạy cả ba tầng; doc test chạy sau cùng.</text></svg>
</div>
<figcaption>Hình 2. Ba tầng test. Unit test kiểm chi tiết bên trong, doc test kiểm ví dụ trong tài liệu, integration test kiểm hợp đồng công khai.</figcaption>
</figure>

**Unit test** nằm cạnh code trong `#[cfg(test)] mod tests`, chỉ được biên dịch khi chạy `cargo test`. Hàm gắn `#[test]` là một ca. Đây là hai ca trong `src/entry.rs`:

```rust
#[test]
fn balanced_entry_is_accepted() -> Result<(), LedgerError> {
    let entry = JournalEntry::try_new(
        "trip:TN-0001:capture",
        vec![
            posting("customer:CU-001", -25_500),
            posting("platform:revenue", 5_100),
            posting("driver:DR-001", 20_400),
        ],
    )?;
    assert_eq!(entry.postings().len(), 3);
    assert_eq!(entry.idempotency_key(), "trip:TN-0001:capture");
    Ok(())
}

// Đây là panic thật vì `unwrap` trên `Err`; thông điệp chứa Debug của lỗi.
#[test]
#[should_panic(expected = "Unbalanced")]
fn unwrap_on_unbalanced_entry_panics() {
    JournalEntry::try_new("k", vec![posting("a", 1), posting("b", 2)]).unwrap();
}
```

Test có thể trả `Result<(), E>`: `?` dùng được và ca fail nếu trả `Err`. `assert!` kiểm điều kiện, `assert_eq!` và `assert_ne!` so hai giá trị (kiểu phải có `PartialEq` và `Debug`). `#[should_panic(expected = …)]` chỉ khớp một đoạn thông điệp panic, nên khá lỏng. Với lỗi nghiệp vụ, hãy ưu tiên `assert_eq!(result, Err(LedgerError::Unbalanced { … }))` như test `unbalanced_entry_is_rejected`: nó kiểm đúng loại lỗi và đúng số dư lệch. Dành `should_panic` cho bug thật.

**Doc test** là khối code trong comment `///`. Ví dụ trên `JournalEntry::try_new` chạy như một chương trình nhỏ gọi thư viện từ bên ngoài, nên tài liệu không thể lỗi thời mà không bị phát hiện. Khối ```` ```compile_fail ```` đòi hỏi đoạn code phải không biên dịch được:

```rust
/// ```compile_fail
/// let _m = ledger_lab::Money(5);
/// ```
```

Lưu ý: `compile_fail` pass với bất kỳ lỗi biên dịch nào, nên đừng tin nó khi chưa thấy lỗi thật. Với `Money(5)`, rustc báo E0423: `cannot initialize a tuple struct which contains private fields`, và gợi ý dùng `Money::from_dong`.

**Integration test** nằm trong `tests/`. Mỗi file là một crate riêng, chỉ dùng được API `pub`, đúng như chương trình khác dùng thư viện của ta:

```rust
// tests/ledger.rs
#[test]
fn replay_does_not_double_count() -> Result<(), LedgerError> {
    let mut ledger = Ledger::new();
    let lines = [("customer:CU-001", -25_500), ("driver:DR-001", 25_500)];
    ledger.post(entry("trip:TN-0001:capture", &lines)?)?;
    let again = ledger.post(entry("trip:TN-0001:capture", &lines)?)?;
    assert!(again.replayed);
    assert_eq!(ledger.len(), 1);
    assert_eq!(
        ledger.balance(&acct("driver:DR-001"))?,
        Money::from_dong(25_500)
    );
    Ok(())
}
```

Đối chiếu xUnit trong [bài về unit test và test double](/articles/csharp-unit-test-test-double): `[Fact]` tương ứng `#[test]`. C# thường để test ở project riêng và chỉ thấy phần `public` (trừ khi bật `InternalsVisibleTo`); Rust cho unit test sống cạnh code và thấy cả private, còn tầng "project riêng" chính là `tests/`.

## Các lệnh hằng ngày: chạy cái nào, khi nào?

| Lệnh | Làm gì | Khi nào chạy |
| --- | --- | --- |
| `cargo check` | Biên dịch kiểm lỗi, không sinh binary | Mỗi lần sửa, cần phản hồi nhanh |
| `cargo test` | Chạy unit, integration và doc test | Trước mỗi commit, trong CI |
| `cargo test ledger::` | Chỉ chạy test có tên chứa chuỗi đó | Đang sửa một module |
| `cargo test -- --nocapture` | Hiện `println!` của test đang pass | Gỡ lỗi một ca |
| `cargo test --doc` | Chỉ chạy doc test | Sau khi sửa comment `///` |
| `cargo test --release` | Chạy test với tối ưu hoá | Trước khi phát hành |
| `cargo clippy --all-targets -- -D warnings` | Lint, mọi cảnh báo thành lỗi | Trước commit, trong CI |
| `cargo fmt --check` | Báo nếu code chưa đúng định dạng (không sửa) | CI; ở máy dùng `cargo fmt` |
| `cargo doc --open` | Sinh tài liệu từ `///` và mở trình duyệt | Xem API `pub` mà người dùng thấy |

Lọc theo tên: `cargo test --lib ledger::` cho kết quả `5 passed; 0 failed; … 11 filtered out`. Output của test pass bị ẩn; thêm `-- --nocapture` thì dòng `tổng toàn sổ = 0 VND` do `println!` trong `tests/ledger.rs` hiện ra. Khi test fail, thông điệp nêu hai vế của `assert_eq!`. Đổi tạm kỳ vọng thành `"25500 VND"` cho ra:

```text
  left: "25.500 VND"
 right: "25500 VND"
```

`--release` đáng chạy riêng vì overflow của `i64` panic ở chế độ debug nhưng quay vòng ở release (bài 03). Một test `#[should_panic]` dựa vào panic tràn số (`i64::MAX + 1`, giá trị đi qua `std::hint::black_box` để compiler không phát hiện sớm) pass ở `cargo test` và fail ở `cargo test --release` với `test did not panic as expected`; ta đã thử trong một project nháp. Vì vậy lab dùng `checked_add` và `Err(Overflow)`, hành vi giống nhau ở cả hai chế độ.

Ghim phiên bản công cụ cho cả nhóm bằng file `rust-toolchain.toml` (khoá `channel`, `components`). Lab không dùng file này vì không bắt buộc.

## clippy bắt gì mà compiler để qua?

`clippy` là bộ lint chính thức: code biên dịch được nhưng có cách viết rõ hơn hoặc dễ sai. Lần chạy `cargo clippy -- -D warnings` đầu tiên trên lab này fail ngay, vì một dòng ta viết tay trong `Money::fmt` (nhóm ba chữ số):

```text
error: manual implementation of `.is_multiple_of()`
  --> src/money.rs:39:25
   |
39 |             if i > 0 && (digits.len() - i) % 3 == 0 {
   |                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: replace with: `(digits.len() - i).is_multiple_of(3)`
```

Sau khi sửa, ta cố ý viết `if ledger.len() == 0` trong `main.rs` và clippy báo tiếp:

```text
error: length comparison to zero
  --> src/main.rs:26:8
   |
26 |     if ledger.len() == 0 {
   |        ^^^^^^^^^^^^^^^^^ help: using `is_empty` is clearer and more explicit: `ledger.is_empty()`
```

Lab giữ bản đã sửa: `(digits.len() - i).is_multiple_of(3)` và `ledger.is_empty()`. Cờ `-D warnings` biến cảnh báo thành lỗi để CI đỏ khi có vấn đề; bỏ cờ thì clippy chỉ in cảnh báo. Nếu thật sự cần tắt một lint, dùng `#[allow(clippy::tên_lint)]` kèm lý do trong comment, đừng tắt cả crate. `cargo fmt` thì không tranh luận: nó viết lại code theo một kiểu chung, nên review không mất thời gian vào dấu cách.

## Chạy ví dụ và kiểm chứng

Từ gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/10-module-cargo-test/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/10-module-cargo-test/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/10-module-cargo-test/Cargo.toml
```

Lệnh đầu in sổ cái sau khi ghi bút toán `JE-000001`, rồi ghi lại cùng `idempotency_key`:

```text
Sổ cái mới, chưa có bút toán.
JE-000001 trip:TN-0001:capture (3 posting, replayed = false)
JE-000001 ghi lại: replayed = true
Số bút toán trong sổ: 1
Số dư:
  customer:CU-001    -25.500 VND
  platform:revenue     5.100 VND
  driver:DR-001       20.400 VND
Tổng toàn sổ: 0 VND
```

`-- --verify` kiểm các giá trị trên cùng hai lỗi (lệch 10 đồng, tràn `i64`) và in `VERIFY OK` với exit code 0; sai thì in `VERIFY FAIL: …` và thoát với mã 1. `cargo test` báo 16 unit test, 4 integration test và 2 doc test đều pass.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Sổ cái này sống trong bộ nhớ: tắt process là mất bút toán và mất luôn bảng idempotency. <code>balance</code> quét toàn bộ bút toán mỗi lần gọi. Chưa có ngày giờ ghi, chưa có đồng thời nhiều luồng, chưa có phân quyền. Phí 5.100 viết cứng, không tính từ tỷ lệ. Đây là mô hình để học tổ chức crate và test, không phải service chạy được trên production.</p>
</div>

**Bài tập:** viết thêm một integration test: ghi hai bút toán giống hệt nhau (khách -25.500, tài xế +25.500) nhưng dùng hai key `trip:TN-0001:capture` và `trip:TN-0001:capture-2`. Số bút toán, mã bút toán thứ hai và số dư tài xế là bao nhiêu? Vì sao khác trường hợp replay?

<details><summary>Xem lời giải</summary><p>Sổ có 2 bút toán, bút toán thứ hai là <code>JE-000002</code>, số dư tài xế là 51.000 VND. Idempotency của lab xét theo <code>idempotency_key</code>, không theo nội dung: key khác nghĩa là yêu cầu khác, nên ghi mới. Replay chỉ xảy ra khi key trùng. Vì vậy key phải sinh từ sự kiện nghiệp vụ (<code>trip:TN-0001:capture</code>), không phải từ số ngẫu nhiên mỗi lần gọi; bài tập này đã chạy trong một test nháp và cho đúng kết quả trên.</p></details>

## Tổng kết phần P01: điều lab đã chứng minh và chưa có

Một crate nhỏ gom lại đa số điều của P01:

- **Cargo và cấu trúc (bài 02, 10):** một package với thư viện, binary và test; `--verify` và `cargo test` cùng chạy từ gốc repo.
- **Kiểu số (bài 03):** tiền là `i64` đơn vị đồng, cộng bằng `checked_add`; tràn số thành `Err(Overflow)`, không panic, không quay vòng.
- **Ownership (bài 04):** `post` nhận quyền sở hữu bút toán; `postings()` chỉ cho mượn `&[Posting]`, nên bút toán đã ghi không sửa được qua alias.
- **Struct và enum (bài 05):** newtype `Money(i64)` và `AccountId`, `JournalEntry` có trường private; `LedgerError` là enum, `Display` dùng `match` vét cạn.
- **Hàm, vòng lặp, iterator (bài 06):** `balance` và `total` duyệt bằng `for` kèm `checked_add`; test dựng posting bằng `iter().map(…).collect()`.
- **Lỗi là giá trị (bài 07):** `try_new` và `post` trả `Result`, `?` đẩy lỗi lên; bút toán lệch không thể tồn tại. `LedgerError` ở đây có 6 biến thể thay cho 5 của bài 07: ghi lại cùng key và cùng nội dung là replay thành công, chỉ khác nội dung mới là `IdempotencyConflict`.
- **Chuỗi và hiển thị (bài 08):** `AccountId::new` nhận `&str`, lưu `String`; `Display` cho `Money` ra `25.500 VND` và `-25.500 VND`, cùng cách định dạng với bài 08.
- **Tập hợp và lifetime (bài 09):** `Vec<JournalEntry>` kèm `HashMap` theo key; `Posted<'a>` mượn bút toán từ sổ.
- **Kiểm thử (bài 10):** 22 test chia ba tầng, clippy và fmt sạch.

Điều lab **chưa có**: database (mất dữ liệu khi tắt), mạng (gRPC, Kafka), async và concurrency, ngày giờ trên bút toán (`jiff` mới dùng ở bài 08), máy trạng thái thanh toán sáu trạng thái của bài 05, `rust_decimal` cho tỷ lệ phí (chỉ có ở bài 03; ở đây 5.100 là số cố định), kiểu riêng cho `EntryId`, trait cho chính sách phí, lỗi dựng bằng `thiserror`, serialize JSON, property test. Số dư tính lại từ bút toán mỗi lần, chưa phải projection được lưu.

Bài trước: [bài 09](/articles/rust-vec-hashmap-btreemap-lifetime-so-cai-bo-nho). Bài tiếp: [bài 11](/articles/rust-trait-generics-chinh-sach-phi-lam-tron).

## Tài liệu gốc

- [The Rust Book: Managing Growing Projects with Packages, Crates, and Modules](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html).
- [The Rust Book: Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html).
- [Cargo Book: Cargo Targets](https://doc.rust-lang.org/cargo/reference/cargo-targets.html).
- [Rustdoc Book: Documentation tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html).
- [The Rust Reference: Visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html).
- [Clippy documentation](https://doc.rust-lang.org/clippy/).
