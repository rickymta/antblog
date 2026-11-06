---
slug: rust-tong-hop-ledger-core-cargo-doc-workspace
title: "Bài tổng hợp cấp cơ bản: thư viện ledger-core có bất biến, test và tài liệu"
summary: "Bài 20: gom P02 thành workspace ledger-core và ledger-cli; bảng bất biến double-entry và test chứng minh; cargo doc với deny(missing_docs), clippy pedantic có chọn lọc; điều còn thiếu trước khi thành service."
category: programming
level: beginner
tags: [rust, toi-noi, ledger, workspace, cargo-doc, clippy]
language: vi
---

Chín bài vừa qua (11–19), mỗi bài thêm một mảnh vào sổ cái. Nếu nhóm Payment hỏi "dùng được chưa?", thư viện cần trả lời được: cái gì công khai, cái gì được bảo đảm, test nào chứng minh, tài liệu nào để đọc.

Đây là bài 20 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài gom phần P02 thành thư viện `ledger-core` trong một Cargo workspace, kèm binary `ledger-cli` chạy kịch bản chuyến `TN-0001`. Lab dùng `rust_decimal`, `serde`, `serde_json`, `thiserror` và `proptest`; không database, không mạng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Workspace có hai package: thư viện <code>ledger-core</code> và binary <code>ledger-cli</code>. Chạy <code>cargo run --manifest-path …/Cargo.toml</code> từ gốc repo mà không có <code>-p</code>, cargo chạy package nào, và vì sao không báo lỗi?</p>
<details><summary>Xem đáp án</summary><p>Cargo chạy <code>ledger-cli</code>. Gốc workspace khai báo <code>default-members</code> gồm cả hai package, nhưng chỉ <code>ledger-cli</code> có binary; <code>ledger-core</code> là thư viện nên không có gì để chạy. Nếu thêm một binary nữa vào <code>ledger-core</code>, cargo sẽ báo <code>could not determine which binary to run</code> và đòi <code>--bin</code> hoặc <code>default-run</code>.</p></details>
</div>

## Vì sao tách workspace hai package?

Thư viện chỉ cung cấp kiểu và quy tắc; chương trình chạy được là việc của binary. Nhờ vậy service ledger sau này là package thứ ba, dùng lại `ledger-core` mà không sao chép.

Một **workspace** là tập package dùng chung một `Cargo.lock` và một thư mục `target/`. `Cargo.toml` ở gốc lab không có mục `[package]`, chỉ khai báo những thứ dùng chung (Hình 1):

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Workspace gồm package ledger-core (thư viện, tám module) và package ledger-cli (binary); binary dùng thư viện, cả hai dùng chung phiên bản dependency và lint khai báo ở Cargo.toml gốc."><defs><marker id="rust-tong-hop-ledger-core-cargo-doc-workspace-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="10" y="8" width="860" height="284" rx="12" class="f-none s-line dash"/><text x="24" y="30" class="t-b">Workspace · Cargo.toml gốc chỉ khai báo, không có code</text><rect x="24" y="46" width="520" height="236" rx="10" class="f-phys-soft s-phys"/><text x="40" y="72" class="t-b">Package ledger-core · crate thư viện</text><rect x="40" y="88" width="112" height="46" rx="8" class="f-surface s-phys"/><text x="50" y="108" class="t-b">money</text><text x="50" y="125" class="t-small">Money</text><rect x="162" y="88" width="112" height="46" rx="8" class="f-surface s-phys"/><text x="172" y="108" class="t-b">account</text><text x="172" y="125" class="t-small">AccountId</text><rect x="284" y="88" width="112" height="46" rx="8" class="f-surface s-phys"/><text x="294" y="108" class="t-b">entry</text><text x="294" y="125" class="t-small">JournalEntry</text><rect x="406" y="88" width="112" height="46" rx="8" class="f-surface s-phys"/><text x="416" y="108" class="t-b">policy</text><text x="416" y="125" class="t-small">FeePolicy</text><rect x="40" y="144" width="112" height="46" rx="8" class="f-surface s-phys"/><text x="50" y="164" class="t-b">state</text><text x="50" y="181" class="t-small">PaymentState</text><rect x="162" y="144" width="112" height="46" rx="8" class="f-surface s-phys"/><text x="172" y="164" class="t-b">error</text><text x="172" y="181" class="t-small">LedgerError</text><rect x="284" y="144" width="112" height="46" rx="8" class="f-surface s-phys"/><text x="294" y="164" class="t-b">ledger</text><text x="294" y="181" class="t-small">Ledger</text><rect x="406" y="144" width="112" height="46" rx="8" class="f-surface s-phys"/><text x="416" y="164" class="t-b">capture</text><text x="416" y="181" class="t-small">TripCharge</text><text x="40" y="224" class="t-small">#![deny(missing_docs)] · README.md là trang đầu của cargo doc</text><text x="40" y="244" class="t-small">tests/: ledger.rs (integration), properties.rs (proptest)</text><text x="40" y="264" class="t-small">deps: rust_decimal, serde, thiserror · dev: proptest, serde_json</text><rect x="620" y="46" width="236" height="104" rx="10" class="f-pk-soft s-pk"/><text x="636" y="72" class="t-b">ledger-cli · binary</text><text x="636" y="96" class="t-small">src/main.rs · cargo run</text><text x="636" y="116" class="t-small">use ledger_core::{…};</text><text x="636" y="136" class="t-small">dependencies: serde_json</text><path d="M620 98 L546 98" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-ledger-core-cargo-doc-workspace-0-a)"/><rect x="620" y="172" width="236" height="110" rx="10" class="f-warn-soft s-warn"/><text x="636" y="198" class="t-b">Khai báo một lần ở gốc</text><text x="636" y="222" class="t-small">[workspace.dependencies]</text><text x="636" y="242" class="t-small">[workspace.lints.clippy]</text><text x="636" y="262" class="t-small">resolver 3 · default-members</text></svg>
</div>
<figcaption>Hình 1. Workspace gồm hai package. <code>ledger-cli</code> dùng <code>ledger-core</code>; phiên bản dependency và lint khai báo một lần ở gốc.</figcaption>
</figure>

```toml
# Cargo.toml (gốc workspace, rút gọn)
[workspace]
members = ["ledger-core", "ledger-cli"]
default-members = ["ledger-core", "ledger-cli"]
resolver = "3"

[workspace.dependencies]
rust_decimal = "1.43"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
proptest = "1"
ledger-core = { path = "ledger-core" }
```

`default-members` ghi rõ lệnh không kèm `-p` chạy trên package nào. Workspace ảo (gốc không có `[package]`) mà không khai thì mặc định đã là mọi member, nên dòng này chỉ làm rõ ý định; `-p ledger-core` thu hẹp về một package. Chạy được từ gốc repo là nhờ `--manifest-path`. `resolver = "3"` là mặc định của edition 2024, nhưng workspace ảo phải tự khai: thiếu dòng này cargo cảnh báo và lùi về `resolver = "1"`. Lab không khai `rust-version` vì mới kiểm trên Rust 1.98.

Package con không lặp lại số phiên bản, chỉ trỏ về gốc:

```toml
# ledger-core/Cargo.toml (rút gọn)
[dependencies]
rust_decimal.workspace = true
serde.workspace = true
thiserror.workspace = true

[dev-dependencies]
proptest.workspace = true
serde_json.workspace = true

[lints]
workspace = true
```

`proptest` và `serde_json` nằm ở `dev-dependencies` vì chỉ test của thư viện cần chúng. `ledger-cli` in JSON nên khai `serde_json` trong `[dependencies]` của nó.

## Chín bài của P02 còn lại gì trong `ledger-core`?

| Bài | Chủ đề | Trong `ledger-core` |
| --- | --- | --- |
| [11](/articles/rust-trait-generics-chinh-sach-phi-lam-tron) | Trait, generics | `FeePolicy` (default method `split`), `RoundingPolicy` (`HalfAwayFromZero`, `NearestEven`), `PercentageFee<R>`, `FlatFee`; `capture_entry` nhận cả generic lẫn `&dyn` |
| [12](/articles/rust-newtype-value-object-money-account-id) | Newtype, value object | `Money`, `AccountId`, `EntryId` với trường private; `EntryId::parse` chỉ nhận dạng chuẩn. Lõi **không lấy** `Money<C>`. Thay đổi: `AccountId::new` thay `FromStr`, `checked_*` trả `Option` thay `Add`/`Sub` trả `Result` |
| [13](/articles/rust-double-entry-journal-entry-posting) | Double-entry | `try_new`, `JournalEntryBuilder`, `reversal_of`, `Ledger::reverse`, `balances` là projection. `Flow`, `AccountKind` ở lại bài 13: lõi chỉ cần số tiền có dấu |
| [14](/articles/rust-typestate-may-trang-thai-thanh-toan) | Typestate, máy trạng thái | `PaymentState`, `PaymentEvent`, `transition` bằng enum. Lõi **không lấy** typestate `Payment<S>` |
| [15](/articles/rust-closure-iterator-adapter-doi-soat-bo-nho) | Closure, iterator | `reconcile` và sao kê VíXanh ở lại bài 15; lõi dùng cùng công cụ: `flat_map`, `filter`, `try_fold` trong `balance`, `balances`, `grand_total` |
| [16](/articles/rust-box-rc-arc-refcell-smart-pointer) | Smart pointer | `Box<dyn FeePolicy>` ở CLI; `Rc`, `Weak`, `RefCell`, `Arc<Mutex>` không vào lõi vì sổ có một chủ, ghi qua `&mut self` |
| [17](/articles/rust-thiserror-anyhow-xu-ly-loi-thuc-te) | `thiserror`, `anyhow` | `LedgerError` (9 biến thể) và `PaymentError` dựng bằng `#[derive(Error)]`; CLI chỉ cần `Box<dyn Error>`, `anyhow` ở lại bài 17 |
| [18](/articles/rust-serde-json-but-toan-decimal-thoi-gian) | `serde` | JSON camelCase, `deny_unknown_fields`, `EntryWire`, `Money` là số nguyên; `schemaVersion`, `LedgerEvent` và `jiff` ở lại bài 18 |
| [19](/articles/rust-proptest-property-test-bat-bien-so-cai) | `proptest` | `tests/properties.rs`: 6 property, 3 trong đó trên bút toán dựng cho cân. Hàm `split` chia nhiều bên ở lại bài 19; `FeePolicy::split` của lõi là hàm khác, chia cước làm hai |

**Lõi cố ý không lấy hai thứ.** `Money<C>`: hệ thống mới có VND, nên như bài 12 đã nói, một `Money` không generic là đủ. Typestate `Payment<S>`: trạng thái ở lõi đến từ bên ngoài (sự kiện Kafka, cột `state` trong database) nên chỉ biết lúc chạy; như bài 14 đã chốt, ở biên dùng enum cộng `transition`, và `match` chỉ cho đúng sáu cạnh đi qua.

Nếu quen C#: trait ↔ interface; newtype ↔ [record struct `Money`](/articles/csharp-record-struct-money-quote); `flat_map`, `try_fold` ↔ LINQ `SelectMany`, `Aggregate`; `thiserror` ↔ exception có kiểu, nhưng lỗi là giá trị trả về; `serde` ↔ `System.Text.Json`; `proptest` ↔ FsCheck, bổ sung cho [unit test](/articles/csharp-unit-test-test-double); workspace ↔ `.sln` cùng `Directory.Packages.props`; `#![deny(missing_docs)]` ↔ cảnh báo CS1591, nhưng ở đây thiếu tài liệu là lỗi biên dịch.

## Bất biến nào được bảo đảm, và test nào chứng minh?

Ba con đường tạo bút toán (`try_new` mà builder gọi, đọc JSON, bút toán đảo) đều gọi `JournalEntry::validated`; không có đường thứ tư vì trường của `JournalEntry` là private (Hình 2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hình 2. Ba đường vào bút toán (try_new cùng builder, đọc JSON, bút toán đảo) cùng đi qua một cổng kiểm tra duy nhất; chỉ sau đó bút toán mới vào sổ append-only, và số dư là projection tính lại từ sổ."><defs><marker id="rust-tong-hop-ledger-core-cargo-doc-workspace-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="24" y="40" width="236" height="56" rx="8" class="f-phys-soft s-phys"/><text x="40" y="64" class="t-b">try_new, builder</text><text x="40" y="83" class="t-small">try_new(key, postings)</text><rect x="24" y="116" width="236" height="56" rx="8" class="f-phys-soft s-phys"/><text x="40" y="140" class="t-b">Đọc JSON</text><text x="40" y="159" class="t-small">serde: try_from EntryWire</text><rect x="24" y="192" width="236" height="56" rx="8" class="f-phys-soft s-phys"/><text x="40" y="216" class="t-b">Bút toán đảo</text><text x="40" y="235" class="t-small">reversal_of(id, &amp;entry)</text><rect x="330" y="36" width="240" height="208" rx="10" class="f-warn-soft s-warn"/><text x="342" y="62" class="t-b">JournalEntry::validated</text><text x="342" y="92" class="t-small">key không rỗng</text><text x="342" y="116" class="t-small">có ít nhất 2 posting</text><text x="342" y="140" class="t-small">tổng có dấu bằng 0</text><text x="342" y="164" class="t-small">không tràn i64</text><text x="342" y="200" class="t-small">sai thì Err(LedgerError)</text><text x="342" y="220" class="t-small">không có đường vòng</text><path d="M260 68 L328 100" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-ledger-core-cargo-doc-workspace-1-a)"/><path d="M260 144 L328 140" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-ledger-core-cargo-doc-workspace-1-a)"/><path d="M260 220 L328 180" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-ledger-core-cargo-doc-workspace-1-a)"/><rect x="620" y="36" width="236" height="56" rx="8" class="f-pk-soft s-pk"/><text x="636" y="60" class="t-b">JournalEntry</text><text x="636" y="79" class="t-small">trường private, luôn cân</text><rect x="620" y="112" width="236" height="56" rx="8" class="f-phys-soft s-phys"/><text x="636" y="136" class="t-b">Ledger::post</text><text x="636" y="155" class="t-small">trùng key: replay hoặc Conflict</text><rect x="620" y="188" width="236" height="56" rx="8" class="f-phys-soft s-phys"/><text x="636" y="212" class="t-b">Vec append-only</text><text x="636" y="231" class="t-small">balances() tính lại từ sổ</text><path d="M570 64 L618 64" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-ledger-core-cargo-doc-workspace-1-a)"/><path d="M738 92 L738 110" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-ledger-core-cargo-doc-workspace-1-a)"/><path d="M738 168 L738 186" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-tong-hop-ledger-core-cargo-doc-workspace-1-a)"/></svg>
</div>
<figcaption>Hình 2. Ba cửa vào, một cổng kiểm tra. Bút toán chỉ vào sổ sau khi qua cổng; số dư luôn tính lại từ sổ, không lưu riêng.</figcaption>
</figure>

```rust
// ledger-core/src/entry.rs (rút gọn)
// Cổng kiểm tra duy nhất: try_new, builder, bút toán đảo và deserialize cùng đi qua đây.
fn validated(
    idempotency_key: &str,
    postings: Vec<Posting>,
    reverses: Option<EntryId>,
) -> Result<JournalEntry, LedgerError> {
    if idempotency_key.trim().is_empty() {
        return Err(LedgerError::EmptyIdempotencyKey);
    }
    if postings.len() < 2 {
        return Err(LedgerError::TooFewPostings { count: postings.len() });
    }
    let sum = total(&postings)?;
    if sum != Money::ZERO {
        return Err(LedgerError::Unbalanced { sum });
    }
    Ok(JournalEntry { idempotency_key: idempotency_key.to_string(), postings, reverses })
}
```

`#[serde(try_from = "EntryWire")]` trên `JournalEntry` bảo serde đọc ra `EntryWire` rồi gọi `validated`: JSON lệch bị từ chối ngay khi đọc. Mười bất biến, cơ chế giữ chúng và test chứng minh ("unit" là test cạnh code, "integration" ở `tests/ledger.rs`, "property" ở `tests/properties.rs`):

| # | Bất biến | Cơ chế | Test chứng minh |
| --- | --- | --- | --- |
| 1 | Bút toán luôn cân, có ít nhất 2 posting, key không rỗng | `validated` là cổng duy nhất, trường private | unit `unbalanced_entry_is_rejected`, `single_posting_is_too_few`; integration `unbalanced_entry_never_reaches_the_ledger`; property `unbalanced_entry_is_always_rejected` |
| 2 | JSON không đưa được bút toán lệch, mã tài khoản rỗng hay trường lạ vào | `try_from` trong `serde`, `deny_unknown_fields` | unit `json_unbalanced_entry_is_rejected_on_read`, `json_blank_id_is_rejected`, `json_unknown_field_is_rejected` |
| 3 | Tràn `i64` là lỗi, không quay vòng | `checked_*` rồi đổi `None` thành `Overflow` | unit `checked_ops_report_overflow_as_none`, `overflow_is_reported_not_wrapped` |
| 4 | Tổng mọi posting của sổ bằng 0 | Mọi bút toán cân, chỉ thêm vào cuối | unit `balance_and_grand_total`; property `grand_total_stays_zero` |
| 5 | Ghi lại cùng key không nhân đôi, khác nội dung thì báo xung đột | Bảng `by_key` và so sánh nội dung | unit `post_same_key_is_idempotent`, `same_key_different_content_conflicts`; property `replay_never_changes_balances` |
| 6 | Bút toán đã ghi không sửa được | Không có `&mut` ra ngoài; `entries()` chỉ trả `&[JournalEntry]` | doc test `compile_fail` của `Ledger`; integration `reversal_restores_balances_and_keeps_history` |
| 7 | Đảo bút toán đưa số dư về 0, bản gốc còn nguyên, đảo hai lần chỉ là replay | `reversal_of` suy key từ bản gốc | unit `reverse_brings_every_balance_back_to_zero`, `reversing_twice_is_a_replay`; property `reversal_cancels_the_entry` |
| 8 | Chia cước không mất, không thừa đồng nào; phí nằm trong 0..=cước | `split` kiểm hợp đồng rồi lấy `fare - fee` | unit `split_rejects_a_fee_above_the_fare`; property `fare_split_never_loses_a_dong` |
| 9 | Chỉ chuyển trạng thái qua sáu cạnh cho phép | `transition` với `match` | unit `exactly_six_edges_are_valid` (duyệt đủ 30 cặp), `terminal_states_go_nowhere` |
| 10 | Không viết được `Money(5)`, tiền là số nguyên | Trường private | doc test `compile_fail` của `Money`; unit `serde_is_a_plain_integer` |

Bảng chỉ có giá trị nếu test thật sự bắt được lỗi. Thử phá: tạm đổi dòng kiểm tổng trong `validated` thành `if false && sum != Money::ZERO` rồi chạy `cargo test -p ledger-core --no-fail-fast`. Bốn test đỏ: hai unit, một integration, một property (`unbalanced_entry_is_always_rejected`). `grand_total_stays_zero` vẫn xanh, và điều đó đúng: nó chỉ sinh bút toán đã cân, nên chứng minh `Ledger` không làm hỏng tổng, không chứng minh `validated` đúng. Sau thí nghiệm, khôi phục dòng đó và xoá `ledger-core/tests/properties.proptest-regressions` mà proptest vừa ghi.

## `deny(missing_docs)` và `cargo doc`: tài liệu cũng phải biên dịch

Hai thuộc tính ở đầu `src/lib.rs`:

```rust
// ledger-core/src/lib.rs (rút gọn)
// Mọi item `pub` phải có tài liệu: thiếu là lỗi biên dịch, không chỉ cảnh báo.
#![deny(missing_docs)]
// README của crate cũng là trang đầu của `cargo doc`; các khối code `rust` trong đó chạy như doc test.
#![doc = include_str!("../README.md")]
```

Xoá thử dòng chú thích `/// Số đồng bên trong.` của `Money::dong` rồi chạy `cargo doc --no-deps`, rustdoc dừng với lỗi:

```text
error: missing documentation for a method
  --> ledger-core/src/money.rs:39:5
   |
39 |     pub const fn dong(self) -> i64 {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
note: the lint level is defined here
  --> ledger-core/src/lib.rs:2:9
   |
 2 | #![deny(missing_docs)]
   |         ^^^^^^^^^^^^
```

Thuộc tính `doc` nối `ledger-core/README.md` vào làm trang đầu của tài liệu. Ví dụ trong README là khối `rust` thật nên `cargo test` chạy nó như doc test; các ví dụ `///` cũng vậy. Dòng ẩn `# Ok::<(), ledger_core::LedgerError>(())` ở cuối cho phép dùng `?` trong ví dụ mà người đọc không thấy.

Mỗi hàm trả `Result` có mục `# Errors` liệt kê biến thể lỗi có thể gặp, ví dụ `AccountId::new`:

```rust
// ledger-core/src/account.rs (rút gọn)
/// Tạo mã tài khoản.
///
/// # Errors
/// [`LedgerError::EmptyAccountId`] nếu chuỗi rỗng hoặc toàn khoảng trắng.
pub fn new(raw: &str) -> Result<AccountId, LedgerError> { /* … */ }
```

Dấu ngoặc vuông là liên kết nội bộ; để liên kết hỏng cũng làm CI đỏ, chạy `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`.

## Clippy pedantic: bật cả nhóm, tắt có chọn lọc

Nhóm `pedantic` của clippy nghiêm hơn mặc định và cố ý ồn ào. Lab bật cả nhóm ở mức `warn` trong gốc workspace, rồi tắt riêng một lint, kèm lý do ngay trong manifest:

```toml
# Cargo.toml (gốc workspace)
[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
# Getter nhỏ (`dong()`, `as_str()`) sẽ đều bị gợi ý #[must_use].
must_use_candidate = "allow"
```

`priority = -1` để nhóm `pedantic` áp dụng trước, dòng `allow` bên dưới thắng. Bỏ dòng đó thì `ledger-core` có 18 cảnh báo, phần lớn là getter và constructor nhỏ; ta không thêm `#[must_use]` cho mọi hàm chỉ để im lặng clippy. Phần còn lại của `pedantic` vẫn bật và có ích: `missing_errors_doc` buộc ta viết mục `# Errors` ở trên. Xoá mục đó của `AccountId::new` thì clippy báo:

```text
warning: docs for function returning `Result` missing `# Errors` section
  --> ledger-core/src/account.rs:26:5
   |
26 |     pub fn new(raw: &str) -> Result<AccountId, LedgerError> {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#missing_errors_doc
   = note: `-W clippy::missing-errors-doc` implied by `-W clippy::pedantic`
   = help: to override `-W clippy::pedantic` add `#[allow(clippy::missing_errors_doc)]`
```

Quy tắc: tắt từng lint, ghi lý do cạnh dòng tắt. `unsafe_code = "forbid"` đảm bảo code của workspace không chứa `unsafe`. Cả hai package kế thừa bằng `[lints] workspace = true`.

## Chạy ví dụ và kiểm chứng

Từ gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/20-ledger-core-workspace/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/20-ledger-core-workspace/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/20-ledger-core-workspace/Cargo.toml
cargo doc --no-deps --manifest-path examples/toi-noi-rust/20-ledger-core-workspace/Cargo.toml
```

Lệnh đầu chạy kịch bản trong `ledger-cli/src/main.rs`:

```text
Cước TN-0001 (5 km): 25.500 VND
Phí nền tảng 20%: 5.100 VND
Tài xế DR-001 nhận: 20.400 VND
Thanh toán: INITIATED -> AUTHORIZED -> CAPTURED
JSON: {"idempotencyKey":"trip:TN-0001:capture","postings":[{"account":"customer:CU-001","amount":-25500},{"account":"platform:revenue","amount":5100},{"account":"driver:DR-001","amount":20400}]}
JE-000001 trip:TN-0001:capture (3 posting, replayed = false)
JE-000001 ghi lại: replayed = true
Số dư sau thu cước:
  customer:CU-001    -25.500 VND
  driver:DR-001       20.400 VND
  platform:revenue     5.100 VND
JE-000002 trip:TN-0001:capture:reversal (đảo JE-000001)
Số dư sau bút toán đảo:
  customer:CU-001          0 VND
  driver:DR-001            0 VND
  platform:revenue         0 VND
Số bút toán: 2, tổng toàn sổ: 0 VND
```

Chính sách phí là `Box<dyn FeePolicy>`; đổi sang `FlatFee` không phải sửa dòng tính bút toán, vì `capture_entry` chỉ biết trait:

```rust
// ledger-core/src/capture.rs
pub fn capture_entry<F>(trip: &TripCharge, policy: &F) -> Result<JournalEntry, LedgerError>
where
    F: FeePolicy + ?Sized,
{
    let split = policy.split(trip.fare)?;
    let platform = AccountId::new("platform:revenue")?;
    let charge = trip.fare.checked_neg().ok_or(LedgerError::Overflow)?;
    JournalEntry::try_new(
        &format!("trip:{}:capture", trip.trip_id),
        vec![
            Posting::new(trip.customer.clone(), charge),
            Posting::new(platform, split.platform_fee),
            Posting::new(trip.driver.clone(), split.driver_net),
        ],
    )
}
```

`-- --verify` kiểm cước, phí, số dư, replay, bút toán đảo, bút toán lệch và chuyển trạng thái sai, rồi in `VERIFY OK` với exit code 0; sai thì thoát mã 1. `cargo test` chạy 67 ca: 2 ở `ledger-cli`, và ở `ledger-core` có 41 unit, 7 integration, 6 property cùng 11 doc test (trong đó 2 `compile_fail`). Phần kiểm tĩnh:

```bash
cd examples/toi-noi-rust/20-ledger-core-workspace
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

Cả ba lệnh đều sạch trên lab.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Sổ cái này sống trong bộ nhớ: tắt process là mất bút toán lẫn bảng idempotency. <code>balance</code> quét toàn bộ bút toán mỗi lần gọi; <code>grand_total</code> cộng tuần tự nên có thể báo <code>Overflow</code> dù tổng cuối bằng 0 nếu một tổng riêng phần vượt <code>i64</code>. Property test chạy 256 ca mỗi property (mặc định của <code>proptest</code>): đủ để bắt lỗi, không phải chứng minh hình thức.</p></div>

## Còn thiếu gì trước khi thành service?

Thư viện đủ để học, chưa đủ cho production. Chưa có:

- **Bền vững:** database; số dư được tính lại mỗi lần gọi.
- **Mạng:** gRPC cho Payment và Admin BFF, Kafka để nhận `PaymentCaptured` và phát `LedgerEntryPosted`.
- **Async và đồng thời:** async, nhiều luồng ghi, khoá, transaction.
- **Mô hình đầy đủ:** thời điểm ghi trên bút toán (`jiff` chưa vào lõi), kiểm định dạng `AccountId`, thuế, khuyến mãi, payout, chargeback.
- **Vận hành:** log có cấu trúc, cấu hình, đo hiệu năng.

Các mục này mở sang lộ trình cấp trung bình.

**Bài tập:** sau kịch bản trên, đảo chính bút toán đảo `JE-000002`. Mã và key của bút toán mới là gì, `ledger.len()` bằng mấy, số dư của `driver:DR-001` là bao nhiêu? Gọi lại `reverse` với `JE-000001` thì sao? Có cần sửa `ledger-core` không?

<details><summary>Xem lời giải</summary><p>Bút toán mới là <code>JE-000003</code> với key <code>trip:TN-0001:capture:reversal:reversal</code>; <code>ledger.len()</code> bằng 3 và số dư tài xế trở lại 20.400 VND, tổng toàn sổ vẫn bằng 0. Gọi lại <code>reverse</code> với <code>JE-000001</code> chỉ là replay, vì key suy ra từ bản gốc nên trùng bản đã ghi; sổ vẫn có 3 bút toán. Không cần sửa thư viện: bút toán đảo cũng là một bút toán cân bình thường, nên <code>reverse</code> áp dụng được cho nó.</p></details>

Bài trước: [bài 19](/articles/rust-proptest-property-test-bat-bien-so-cai). Bài tiếp: [bài 21](/articles/rust-async-await-tokio-task-timeout-cancellation), đầu lộ trình Rust trung bình.

## Tài liệu gốc

- [The Cargo Book: Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html).
- [The Cargo Book: The `[lints]` section](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section).
- [The Rust Book: Making Useful Documentation Comments](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments).
- [Rustdoc Book: Documentation tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html).
- [Clippy documentation: lint groups](https://doc.rust-lang.org/clippy/lints.html).
- [docs.rs: thiserror](https://docs.rs/thiserror) và [proptest](https://docs.rs/proptest).
