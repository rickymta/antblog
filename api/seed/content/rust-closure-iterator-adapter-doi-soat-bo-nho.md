---
slug: rust-closure-iterator-adapter-doi-soat-bo-nho
title: "Closure, iterator adapter và lazy evaluation: đối soát trong bộ nhớ"
summary: "Bài 15: đối soát sổ cái với sao kê VíXanh giả theo external_ref bằng closure Fn/FnMut/FnOnce, iterator adapter, group-by thủ công và collect; chứng minh iterator lazy bằng inspect, đối chiếu LINQ."
category: programming
level: beginner
tags: [rust, toi-noi, ledger, closure, iterator, reconciliation]
language: vi
---

Cuối ngày, VíXanh gửi sao kê năm dòng; sổ cái của Tới Nơi có bốn bút toán thu tiền qua VíXanh. Hai bên phải khớp, và khi không khớp phải nói được lệch ở đâu: sổ ta thiếu, sao kê thiếu, hay hai bên ghi số tiền khác nhau. Đó là đối soát (reconciliation), và là chỗ tốt để học closure và iterator, vì cả bài toán là biến hai danh sách thành một báo cáo.

Đây là bài 15 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài dùng `HashMap`, `BTreeMap` của [bài 09](/articles/rust-vec-hashmap-btreemap-lifetime-so-cai-bo-nho) và trait bound của [bài 11](/articles/rust-trait-generics-chinh-sach-phi-lam-tron). Lab chỉ dùng thư viện chuẩn.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Đoạn <code>records.iter().map(|r| println!("{}", r.external_ref));</code> có in gì ra màn hình không?</p>
<details><summary>Xem đáp án</summary><p><b>Không in gì.</b> <code>map</code> chỉ dựng một iterator mới, chưa ai kéo phần tử nào. Compiler cảnh báo <code>unused `Map` that must be used</code>, ghi chú <code>iterators are lazy and do nothing unless consumed</code>. Đổi sang <code>for_each</code>, như compiler gợi ý, thì <code>println!</code> mới chạy.</p></details>
</div>

## Đối soát là so hai danh sách theo khóa, không theo vị trí

Khóa khớp là `external_ref`, mã giao dịch của VíXanh. Lab đặt nó vào `idempotency_key` của bút toán thu qua VíXanh, dạng `psp:vixanh:<external_ref>`, để có sẵn khóa mà chưa cần thêm trường mới. Đây là quy ước riêng của lab đối soát, không thay `trip:TN-0001:capture` của các bài khác; sổ thật giữ key theo chuyến và lưu `external_ref` ở trường riêng. Dấu giữ như bước capture của [bài 13](/articles/rust-double-entry-journal-entry-posting): ví khách âm, `psp:vixanh:clearing` dương. Bút toán chia `trip:TN-0001:split` (bước 2 của bài 13) là chuyển nội bộ, sao kê VíXanh không có, nên không đối soát.

Thuật toán có ba bước (Hình 1): cộng tiền theo khóa ở từng phía, hợp hai tập khóa, phân loại từng khóa. Phải cộng trước khi so vì sao kê có thể chia một khóa thành nhiều dòng: `VX-1002` là 25.000 và 15.000.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 1. Đối soát gồm ba bước: cộng tiền theo external_ref ở từng phía, hợp hai tập khóa rồi phân loại; kết quả có hai ref khớp, một ref lệch số tiền, một ref thiếu bên PSP và một ref thiếu bên ta."><defs><marker id="rust-closure-iterator-adapter-doi-soat-bo-nho-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="230" height="140" rx="10" class="f-phys-soft s-phys"/><text x="36" y="46" class="t-b">Sổ cái · 4 dòng</text><text x="36" y="72" class="t-mono">VX-1001  25.500</text><text x="36" y="92" class="t-mono">VX-1002  40.000</text><text x="36" y="112" class="t-mono">VX-1003  31.500</text><text x="36" y="132" class="t-mono">VX-1004  18.000</text><rect x="20" y="175" width="230" height="140" rx="10" class="f-phys-soft s-phys"/><text x="36" y="201" class="t-b">Sao kê VíXanh · 5 dòng</text><text x="36" y="224" class="t-mono">VX-1001  25.500</text><text x="36" y="242" class="t-mono">VX-1002  25.000</text><text x="36" y="260" class="t-mono">VX-1002  15.000</text><text x="36" y="278" class="t-mono">VX-1003  31.000</text><text x="36" y="296" class="t-mono">VX-1005  52.000</text><rect x="300" y="20" width="210" height="140" rx="10" class="f-surface s-phys"/><text x="316" y="46" class="t-b">Theo ref · sổ cái</text><text x="316" y="72" class="t-mono">VX-1001 → 25.500</text><text x="316" y="92" class="t-mono">VX-1002 → 40.000</text><text x="316" y="112" class="t-mono">VX-1003 → 31.500</text><text x="316" y="132" class="t-mono">VX-1004 → 18.000</text><rect x="300" y="175" width="210" height="140" rx="10" class="f-surface s-phys"/><text x="316" y="201" class="t-b">Theo ref · sao kê</text><text x="316" y="224" class="t-mono">VX-1001 → 25.500</text><text x="316" y="242" class="t-mono">VX-1002 → 40.000</text><text x="316" y="260" class="t-mono">VX-1003 → 31.000</text><text x="316" y="278" class="t-mono">VX-1005 → 52.000</text><text x="316" y="302" class="t-small">VX-1002 gộp từ 2 dòng</text><rect x="560" y="20" width="300" height="295" rx="10" class="f-pk-soft s-pk"/><text x="576" y="44" class="t-b">Phân loại từng ref → BTreeMap</text><rect x="576" y="58" width="268" height="58" rx="8" class="f-warn-soft s-warn"/><text x="590" y="80" class="t-b">VX-1003 · lệch số tiền</text><text x="590" y="100" class="t-small">sổ 31.500, sao kê 31.000, chênh 500</text><rect x="576" y="126" width="268" height="58" rx="8" class="f-warn-soft s-warn"/><text x="590" y="148" class="t-b">VX-1004 · thiếu bên PSP</text><text x="590" y="168" class="t-small">sổ ghi 18.000, sao kê không có</text><rect x="576" y="194" width="268" height="58" rx="8" class="f-warn-soft s-warn"/><text x="590" y="216" class="t-b">VX-1005 · thiếu bên ta</text><text x="590" y="236" class="t-small">sao kê có 52.000, sổ chưa ghi</text><text x="576" y="282" class="t-small">VX-1001, VX-1002: bằng nhau, 2 ref khớp</text><path d="M250 90 L298 90" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-closure-iterator-adapter-doi-soat-bo-nho-0-a)"/><path d="M250 245 L298 245" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-closure-iterator-adapter-doi-soat-bo-nho-0-a)"/><path d="M510 90 L558 140" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-closure-iterator-adapter-doi-soat-bo-nho-0-a)"/><path d="M510 245 L558 190" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-closure-iterator-adapter-doi-soat-bo-nho-0-a)"/></svg>
</div>
<figcaption>Hình 1. Cộng theo khóa ở từng phía, rồi phân loại. <code>VX-1002</code> khớp sau khi cộng hai dòng sao kê.</figcaption>
</figure>

## `Fn`, `FnMut`, `FnOnce`: closure làm gì với biến nó bắt?

Closure là hàm vô danh dùng được biến ở quanh nó (biến được bắt, captured). Compiler cài trait cho closure tùy cách nó dùng các biến đó:

| Trait | Closure làm gì với biến bắt được | Gọi được | Trong lab |
| --- | --- | --- | --- |
| `Fn` | Chỉ đọc | Nhiều lần, kể cả cùng lúc | `\|r\| r.amount >= min` |
| `FnMut` | Sửa biến bắt được | Nhiều lần, lần lượt | `\|_, l, p\| if l != p { mismatched += 1 }` |
| `FnOnce` | Chuyển (move) biến bắt được ra ngoài | Đúng một lần | `move \|\| note` (trả `note` đi) |

Mọi closure đều là `FnOnce`; không move biến ra thì là `FnMut`; không sửa biến nữa thì là `Fn`. Hàm nhận closure nên đòi ít nhất có thể: `visit_common` gọi closure nhiều lần nên không nhận `FnOnce`, nhưng không gọi đồng thời nên chỉ đòi `FnMut`; nhờ đó người gọi đếm được ref lệch.

```rust
// src/reconcile.rs
pub fn visit_common<F>(ours: &HashMap<String, Money>, theirs: &HashMap<String, Money>, mut visit: F)
where
    F: FnMut(&str, Money, Money),
{
    ours.iter()
        .filter_map(|(key, &ledger)| theirs.get(key).map(|&psp| (key, ledger, psp)))
        .for_each(|(key, ledger, psp)| visit(key, ledger, psp));
}
```

```rust
// src/main.rs (rút gọn)
let mut mismatched = 0;
visit_common(&ours, &theirs, |_, l, p| {
    if l != p {
        mismatched += 1;
    }
});
println!("FnMut   : {mismatched} ref có ở cả hai phía nhưng lệch số tiền");
```

`visit` phải khai báo `mut` vì gọi một `FnMut` là mượn nó mutably. Truyền closure sửa biến vào chỗ đòi `Fn` (như `key_of` của `group_by` ở phần sau) là lỗi E0594 (đã thử, rút gọn):

```text
error[E0594]: cannot assign to `seen`, as it is a captured variable in a `Fn` closure
5 |     let groups = group_by(["a", "b"], |s| {
  |                  --------             --- in this closure
  |                  |
  |                  expects `Fn` instead of `FnMut`
6 |         seen += 1;
  |         ^^^^^^^^^ cannot assign
```

Closure `FnOnce` tiêu thụ biến của nó, nên gọi lần hai là lỗi E0382:

```text
error[E0382]: use of moved value: `finish`
note: closure cannot be invoked more than once because it moves the variable `note` out of its environment
note: this value implements `FnOnce`, which causes it to be moved when called
```

## Bắt theo tham chiếu hay `move`?

Mặc định closure bắt biến theo cách nhẹ nhất đủ dùng: đọc thì mượn `&`, sửa thì mượn `&mut`, move ra thì lấy luôn. Từ khóa `move` ép mọi biến thành bản sở hữu của closure. Nó **không** quyết định `Fn`, `FnMut` hay `FnOnce`, chỉ quyết định biến được mượn hay được lấy đi. Cần `move` khi closure sống lâu hơn hàm tạo ra nó, như khi trả closure về:

```rust
// src/reconcile.rs
pub fn at_least(min: Money) -> impl Fn(&LedgerRecord) -> bool {
    move |record| record.amount >= min
}
```

Bỏ `move`, closure mượn `min`, mà `min` mất khi hàm kết thúc. Compiler từ chối (E0373) và chỉ chỗ sửa:

```text
error[E0373]: closure may outlive the current function, but it borrows `min`, which is owned by the current function
help: to force the closure to take ownership of `min` (and any other referenced variables), use the `move` keyword
```

`Money` là `Copy` nên `move` chỉ sao chép 8 byte; với `String` hay `Vec`, `move` chuyển quyền sở hữu. Kiểu closure không có tên, nên hàm trả về `impl Fn(&LedgerRecord) -> bool`.

## Iterator adapter: dựng đường ống từ từng bước nhỏ

Adapter là phương thức của `Iterator` nhận một iterator và trả iterator khác. Cuối đường ống là một bước tiêu thụ (`collect`, `sum`, `fold`, `for`). `map`, `filter`, `fold` đã có ở [bài 06](/articles/rust-ham-dieu-kien-vong-lap-iterator-so-du); lab thêm:

| Adapter | Việc | Chỗ dùng trong lab |
| --- | --- | --- |
| `filter_map` | Lọc và đổi cùng lúc, bỏ `None` | Bỏ bút toán không qua VíXanh |
| `take_while` | Lấy đến khi điều kiện sai lần đầu | Dừng ở dòng `---` |
| `zip` | Ghép hai iterator theo vị trí | Ví dụ sai ở phần sau |

Đọc sao kê là một đường ống:

```rust
// src/statement.rs
pub fn parse_statement(text: &str) -> Result<Vec<PspRecord>, StatementError> {
    text.lines()
        .enumerate() // đánh số trước khi bỏ tiêu đề, để số dòng báo lỗi đúng
        .skip(1) // bỏ dòng tiêu đề
        .take_while(|(_, line)| !line.starts_with("---")) // dừng ở chân trang
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| parse_line(index + 1, line))
        .collect()
}
```

Thứ tự adapter có nghĩa: đổi chỗ `enumerate` và `skip` thì số dòng báo lỗi lệch một. Phía sổ cái, `LedgerRecord::from_entry` trả `Result<Option<_>, _>`, còn `filter_map` cần `Option`. `transpose` đổi `Result<Option<T>, E>` thành `Option<Result<T, E>>`: `None` bị bỏ, `Some(Err(..))` đi tiếp cho `collect` bắt:

```rust
// src/reconcile.rs
pub fn ledger_records(ledger: &Ledger) -> Result<Vec<LedgerRecord>, LedgerError> {
    ledger
        .entries_with_id()
        .filter_map(|(id, entry)| LedgerRecord::from_entry(id, entry).transpose())
        .collect()
}
```

`collect` vào `Result<Vec<_>, _>` chạy hết nếu mọi phần tử là `Ok`, và dừng ở `Err` đầu tiên rồi trả đúng lỗi đó. Sao kê có dòng `VX-2,12a` cho `dòng 3: số tiền không hợp lệ ("12a")`, không phải một `Vec` thiếu dòng.

## Group-by thủ công và ba đích của `collect`

`Iterator` không có `group_by` (`slice::chunk_by` chỉ gom các dòng liền kề); viết tay chỉ vài dòng với `entry` của bài 09, dùng `or_default()` thay `or_insert`:

```rust
// src/reconcile.rs
pub fn group_by<T, K, F>(items: impl IntoIterator<Item = T>, key_of: F) -> HashMap<K, Vec<T>>
where
    K: Eq + Hash,
    F: Fn(&T) -> K,
{
    let mut groups: HashMap<K, Vec<T>> = HashMap::new();
    for item in items {
        // `entry().or_default()` tạo Vec rỗng nếu khóa chưa có, rồi trả `&mut Vec`.
        groups.entry(key_of(&item)).or_default().push(item);
    }
    groups
}
```

`key_of` chỉ đọc `item` nên `Fn` là đủ. Cộng từng nhóm dùng `try_fold` (như `fold` nhưng dừng khi gặp `Err`) để kiểm tràn số, rồi `collect` vào `Result<HashMap<_, _>, _>`:

```rust
// src/reconcile.rs
fn sum_groups<T>(
    groups: HashMap<String, Vec<T>>,
    amount_of: impl Fn(&T) -> Money,
) -> Result<HashMap<String, Money>, LedgerError> {
    groups
        .into_iter()
        .map(|(key, rows)| {
            let sum = rows.iter().try_fold(Money::ZERO, |acc, row| {
                acc.checked_add(amount_of(row)).ok_or(LedgerError::Overflow)
            })?;
            Ok((key, sum))
        })
        .collect()
}
```

Lab có ba đích `collect`: `Result<Vec<_>, _>` cho kết quả có thể lỗi, `Result<HashMap<_, _>, _>` cho tổng theo ref, `BTreeMap` cho báo cáo cần thứ tự ổn định. Báo cáo dùng `BTreeSet` hợp hai tập khóa nên luôn sắp theo `external_ref`:

```rust
// src/reconcile.rs (rút gọn)
let findings: BTreeMap<String, Finding> = refs
    .iter()
    .filter_map(|&key| {
        let finding = match (ours.get(key).copied(), theirs.get(key).copied()) {
            (Some(l), Some(p)) if l == p => return None,
            (Some(l), Some(p)) => Finding::AmountMismatch { ledger: l, psp: p },
            (Some(l), None) => Finding::MissingInPsp { ledger: l },
            (None, Some(p)) => Finding::MissingInLedger { psp: p },
            (None, None) => return None,
        };
        Some((key.to_string(), finding))
    })
    .collect();
```

`collect` vào `HashMap` từ dữ liệu có khóa trùng thì giá trị sau âm thầm thay giá trị trước, không có lỗi. Test `collect_into_hashmap_keeps_last_duplicate_key` ghi lại hành vi này; vì vậy ta gom nhóm rồi cộng, không `collect` thẳng dòng sao kê vào map.

Còn `zip`. Hàm `positional_ref_clashes` ghép sổ và sao kê theo vị trí để cho thấy cách làm sai: sao kê có thêm một dòng `VX-1002` nên các cặp sau đó lệch một bước, và `VX-1005` không có cặp vì `zip` dừng ở danh sách ngắn hơn:

```text
  ghép VX-1003 với VX-1002
  ghép VX-1004 với VX-1003
```

Hai cặp này là báo động giả, còn `VX-1005` thiếu bên ta không xuất hiện. Chỉ dùng `zip` khi hai danh sách thật sự thẳng hàng.

## Lazy: iterator không chạy cho đến khi có người kéo

Adapter chỉ ghi "sẽ làm gì" vào một giá trị iterator. Việc chỉ xảy ra khi bước tiêu thụ gọi `next()`, và mỗi `next()` kéo ngược từ cuối đường ống về đầu, dừng khi có một phần tử qua hết các bước. Lab chứng minh bằng `inspect`, adapter chạy closure cho mỗi phần tử đi qua mà không đổi nó:

```rust
// src/main.rs (rút gọn)
let big = at_least(Money::from_dong(30_000));
println!("Dựng chuỗi inspect → filter → map (chưa tiêu thụ):");
let mut iter = records
    .iter()
    .inspect(|r| println!("    inspect {}", r.external_ref))
    .filter(|r| big(r))
    .map(|r| r.external_ref.as_str());
println!("  dựng xong, chưa có dòng inspect nào ở trên");
for call in 1..=3 {
    println!("  next() lần {call}:");
    println!("    -> {:?}", iter.next());
}
```

Output thật:

```text
Dựng chuỗi inspect → filter → map (chưa tiêu thụ):
  dựng xong, chưa có dòng inspect nào ở trên
  next() lần 1:
    inspect VX-1001
    inspect VX-1002
    -> Some("VX-1002")
  next() lần 2:
    inspect VX-1003
    -> Some("VX-1003")
  next() lần 3:
    inspect VX-1004
    -> None
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 256" role="img" aria-label="Hình 2. Đường ống iterator gồm iter, inspect, filter, map và bước tiêu thụ. Dựng chuỗi mà không gọi next thì inspect chạy 0 lần; lấy phần tử khớp đầu tiên chạy 2 lần; collect toàn bộ chạy 4 lần."><defs><marker id="rust-closure-iterator-adapter-doi-soat-bo-nho-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="150" height="50" rx="8" class="f-phys-soft s-phys"/><text x="36" y="50" class="t-mono">records.iter()</text><rect x="200" y="20" width="150" height="50" rx="8" class="f-phys-soft s-phys"/><text x="216" y="50" class="t-mono">inspect(…)</text><rect x="380" y="20" width="150" height="50" rx="8" class="f-phys-soft s-phys"/><text x="396" y="50" class="t-mono">filter(|r| big(r))</text><rect x="560" y="20" width="150" height="50" rx="8" class="f-phys-soft s-phys"/><text x="576" y="50" class="t-mono">map(…ref)</text><rect x="740" y="20" width="120" height="50" rx="8" class="f-pk-soft s-pk"/><text x="756" y="50" class="t-mono">next()/collect</text><path d="M170 45 L198 45" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-closure-iterator-adapter-doi-soat-bo-nho-1-a)"/><path d="M350 45 L378 45" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-closure-iterator-adapter-doi-soat-bo-nho-1-a)"/><path d="M530 45 L558 45" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-closure-iterator-adapter-doi-soat-bo-nho-1-a)"/><path d="M710 45 L738 45" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-closure-iterator-adapter-doi-soat-bo-nho-1-a)"/><path d="M800 72 L800 92 L95 92 L95 74" class="f-none s-pk dash" stroke-width="2" marker-end="url(#rust-closure-iterator-adapter-doi-soat-bo-nho-1-a)"/><text x="330" y="86" class="t-small">pull: mỗi next() kéo ngược từ phải sang trái</text><rect x="20" y="112" width="840" height="38" rx="8" class="f-phys-soft s-phys"/><text x="36" y="136" class="t-b">Chỉ dựng chuỗi</text><text x="260" y="136" class="t-small">không ai gọi next() · inspect chạy 0 lần</text><rect x="20" y="158" width="840" height="38" rx="8" class="f-pk-soft s-pk"/><text x="36" y="182" class="t-b">next() lần 1</text><text x="260" y="182" class="t-small">inspect VX-1001, VX-1002 · dừng ở VX-1002 · chạy 2 lần</text><rect x="20" y="204" width="840" height="38" rx="8" class="f-warn-soft s-warn"/><text x="36" y="228" class="t-b">collect() toàn bộ</text><text x="260" y="228" class="t-small">duyệt cả 4 dòng sổ · inspect chạy 4 lần, kết quả nằm sẵn trong một Vec</text></svg>
</div>
<figcaption>Hình 2. Cùng một đường ống, ba cách tiêu thụ cho ba số lần <code>inspect</code> khác nhau. Con số 0, 2, 4 do <code>lazy_probe</code> trả về và <code>--verify</code> kiểm.</figcaption>
</figure>

Lazy cho hai điều: chỉ trả công cho phần cần dùng (`next()` đầu tiên duyệt hai dòng thay vì bốn), và không tạo `Vec` trung gian. Quy tắc: `collect` một lần ở cuối, sát chỗ tiêu thụ.

`lazy_probe` đếm bằng closure `FnMut` sửa một biến đếm. Iterator còn được dùng về sau thì vẫn đang mượn mutably biến đó, nên đọc biến ở giữa là lỗi E0502 (đã thử):

```text
error[E0502]: cannot borrow `seen` as immutable because it is also borrowed as mutable
4 |     let it = amounts.iter().inspect(|_| seen += 1);
  |                                     --- ---- first borrow occurs due to use of `seen` in closure
  |                                     |
  |                                     mutable borrow occurs here
5 |     println!("đã thấy {seen}");
  |                        ^^^^ immutable borrow occurs here
6 |     let total: i64 = it.sum();
  |                      -- mutable borrow later used here
```

Trong lab, iterator được bỏ đi (`drop(unused)`) trước khi đọc bộ đếm nên không vướng. Một nét nữa: `Ledger::entries_with_id` trả `impl Iterator` thay vì kiểu thật `Map<Enumerate<Iter<…>>, …>`:

```rust
// src/ledger.rs
pub fn entries_with_id(&self) -> impl Iterator<Item = (String, &JournalEntry)> {
    self.entries
        .iter()
        .enumerate()
        .map(|(index, entry)| (entry_id(index), entry))
}
```

Người gọi chỉ thấy "một iterator của `(String, &JournalEntry)`", nên cài đặt đổi được mà API không đổi. Từ edition 2024, `impl Trait` ở vị trí trả về tự bắt mọi lifetime trong phạm vi, gồm của `&self`, nên không cần `+ '_`.

## Với người từ C#: LINQ và iterator adapter

LINQ và iterator adapter cùng một ý tưởng. Phía C#, xem [bài generics và collections](/articles/csharp-generics-collections).

| Việc | LINQ (C#) | Iterator Rust trong lab |
| --- | --- | --- |
| Lọc | `Where(r => r.Amount >= min)` | `.filter(\|r\| r.amount >= min)` |
| Biến đổi | `Select` | `.map` |
| Gom nhóm | `GroupBy(r => r.ExternalRef)`, hoãn chạy | `group_by` viết tay, chạy ngay, trả `HashMap<K, Vec<T>>` |
| Ra từ điển | `ToDictionary`, ném `ArgumentException` khi trùng khóa | `collect::<HashMap<_, _>>()`, giữ giá trị cuối |
| Gom một giá trị | `Aggregate` | `fold`, `try_fold` |

Giống nhau: `Where`/`Select` hoãn chạy đến `foreach` hay `ToList()`, như adapter đến `collect`. Khác nhau: Rust bắt chọn mượn hay `move`, còn lambda C# bắt chính biến nên mọi nơi cùng thấy thay đổi; iterator Rust đi một lượt (bước tiêu thụ nhận `self`), còn truy vấn LINQ duyệt lại sẽ chạy lại từ đầu; lỗi trong đường ống là `Result`, không phải exception.

## Chạy ví dụ và kiểm chứng

Từ gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/15-closure-iterator-doi-soat/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/15-closure-iterator-doi-soat/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/15-closure-iterator-doi-soat/Cargo.toml
```

Lệnh đầu in (khối `--- Lazy ---` đã xem ở trên):

```text
Phía sổ cái (5 bút toán, 4 dòng qua VíXanh):
  JE-000001 VX-1001   25.500 VND
  JE-000003 VX-1002   40.000 VND
  JE-000004 VX-1003   31.500 VND
  JE-000005 VX-1004   18.000 VND
Phía sao kê VíXanh (5 dòng):
  VX-1001   25.500 VND
  VX-1002   25.000 VND
  VX-1002   15.000 VND
  VX-1003   31.000 VND
  VX-1005   52.000 VND
Kết quả đối soát: 2 ref khớp, 3 chênh lệch
  VX-1003: lệch số tiền: sổ 31.500 VND, sao kê 31.000 VND, chênh 500 VND
  VX-1004: thiếu bên PSP: sổ ghi 18.000 VND, sao kê không có
  VX-1005: thiếu bên ta: sao kê có 52.000 VND, sổ chưa ghi
Tóm tắt: thiếu bên PSP = 1, thiếu bên ta = 1, lệch số tiền = 1
--- Closure ---
Fn      : 2 dòng sổ từ 30.000 VND trở lên
FnMut   : 1 ref có ở cả hai phía nhưng lệch số tiền
FnOnce  : đã đối soát với sao kê VíXanh
--- zip ghép theo vị trí (cách sai) ---
  ghép VX-1003 với VX-1002
  ghép VX-1004 với VX-1003
```

Sau khối Lazy, hai dòng cuối là:

```text
inspect chạy: dựng = 0, next đầu tiên = 2, collect = 4
--- Sao kê hỏng ---
  dòng 3: số tiền không hợp lệ ("12a")
```

`JE-000002` (`trip:TN-0001:split`) vắng ở phía sổ cái vì `filter_map` bỏ nó: khóa không có tiền tố `psp:vixanh:`. `-- --verify` kiểm các giá trị trên, in `VERIFY OK` với exit code 0; sai thì thoát mã 1. `cargo test` chạy 30 unit test, 4 integration test và 2 doc test.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Đối soát thật đọc sao kê hàng trăm nghìn dòng, so theo ngày chốt và múi giờ, xử lý phí PSP, hoàn tiền, chargeback, dung sai làm tròn, và chạy lại được khi sao kê đến muộn. Lab chỉ có một ngày, một PSP, khớp đúng từng đồng, dữ liệu trong bộ nhớ, và chưa xử lý chênh lệch.</p>
</div>

**Bài tập:** viết hàm `within_tolerance(tolerance: Money)` trả closure `impl Fn(Money, Money) -> bool`, coi hai số tiền là khớp nếu lệch không quá `tolerance`. Với dung sai 500, `VX-1003` (31.500 so với 31.000) có còn bị báo lệch không? Vì sao không viết `(a.dong() - b.dong()).abs()`?

<details><summary>Xem lời giải</summary><p><code>move |a, b| a.checked_sub(b).is_some_and(|d| d.dong().unsigned_abs() &lt;= tolerance.dong().unsigned_abs())</code>, bọc trong <code>impl Fn(Money, Money) -> bool</code>; <code>move</code> là bắt buộc vì <code>tolerance</code> là tham số cục bộ. Chênh của <code>VX-1003</code> đúng bằng 500 nên dung sai 500 coi là khớp, dung sai 499 thì vẫn báo (bản chạy thử cho <code>true</code> và <code>false</code>). Phép trừ trần có thể tràn <code>i64</code>, và <code>abs()</code> của <code>i64::MIN</code> cũng tràn (panic ở debug); <code>checked_sub</code> cộng <code>unsigned_abs</code> tránh cả hai, còn <code>None</code> thì trả <code>false</code>.</p></details>

Bài trước: [bài 14](/articles/rust-typestate-may-trang-thai-thanh-toan). Bài tiếp: [bài 16](/articles/rust-box-rc-arc-refcell-smart-pointer).

## Tài liệu gốc

- [The Rust Book: Closures](https://doc.rust-lang.org/book/ch13-01-closures.html).
- [The Rust Book: Processing a Series of Items with Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html).
- [std::iter: Module và trait Iterator](https://doc.rust-lang.org/std/iter/index.html).
- [std::collections::hash_map::Entry](https://doc.rust-lang.org/std/collections/hash_map/enum.Entry.html).
- [The Rust Reference: Closure types](https://doc.rust-lang.org/reference/types/closure.html).
- [The Rust Reference: Impl trait type](https://doc.rust-lang.org/reference/types/impl-trait.html).
