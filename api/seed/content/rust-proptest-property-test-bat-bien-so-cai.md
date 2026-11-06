---
slug: rust-proptest-property-test-bat-bien-so-cai
title: "Property-based test với proptest: chứng minh bất biến sổ cái"
summary: "Bài 19: dùng proptest kiểm bất biến sổ cái Tới Nơi: strategy sinh bút toán luôn cân, tổng bằng 0, idempotency, chia tiền không mất đồng, và shrinking thu nhỏ phản ví dụ về còn 1 đồng."
category: rust
level: beginner
tags: [rust, toi-noi, ledger, proptest, property-test]
language: vi
---

Bút toán thu cước `TN-0001` có test xanh từ bài 10: khách -25.500, nền tảng +5.100, tài xế +20.400, tổng bằng 0. Test đó chỉ đúng với **một** bút toán do ta chọn. Còn bút toán 7 posting, hai posting cùng tài khoản, hay chia 10 đồng cho ba bên thì sao?

Đây là bài 19 của lộ trình **Rust cơ bản qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài này dùng crate `proptest` để máy tự sinh hàng nghìn đầu vào và kiểm các **bất biến** của sổ cái ở [bài 13](/articles/rust-double-entry-journal-entry-posting). Lab xuất phát từ mô hình [bài 10](/articles/rust-module-crate-cargo-test-clippy), thêm hàm chia tiền và bộ strategy.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Chia 10 đồng cho 3 bên đều nhau bằng cách lấy <code>10 / 3</code> cho mỗi bên (chia nguyên). Tổng ba phần là bao nhiêu?</p>
<details><summary>Xem đáp án</summary><p><b>9 đồng.</b> <code>10 / 3 = 3</code> với <code>i64</code>, ba phần cộng lại 9, mất 1 đồng.</p></details>
</div>

## Example test và property test khác nhau ở đâu?

**Example test** ghi sẵn đầu vào lẫn kết quả: `split(10, [1, 1, 1])` phải ra `[4, 3, 3]`. **Property test** chỉ ghi một tính chất đúng với mọi đầu vào hợp lệ, ví dụ "tổng các phần luôn bằng tổng gốc"; máy chọn đầu vào.

Example test mạnh ở các ca nghiệp vụ có tên như `TN-0001`; property test mạnh ở biên và tổ hợp ta chưa nghĩ tới. Mỗi property mặc định chạy 256 ca; các bất biến sổ cái trong lab nâng lên 1.000. Một property chạy như sau:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 1. Chu trình của một property test: strategy mô tả dữ liệu, runner sinh 1.000 ca, thân test kiểm tính chất; nếu có ca fail thì shrinking thu nhỏ đầu vào và proptest in phản ví dụ nhỏ nhất kèm seed."><defs><marker id="rust-proptest-property-test-bat-bien-so-cai-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="16" y="30" width="140" height="96" rx="10" class="f-phys-soft s-phys"/><text x="30" y="56" class="t-b">Strategy</text><text x="30" y="80" class="t-small">mô tả dữ liệu</text><text x="30" y="100" class="t-small">any::&lt;i64&gt;(), vec</text><rect x="194" y="30" width="140" height="96" rx="10" class="f-phys-soft s-phys"/><text x="208" y="56" class="t-b">1.000 ca</text><text x="208" y="80" class="t-small">RNG sinh đầu vào</text><text x="208" y="100" class="t-small">ProptestConfig</text><rect x="372" y="30" width="140" height="96" rx="10" class="f-pk-soft s-pk"/><text x="386" y="56" class="t-b">Thân test</text><text x="386" y="80" class="t-small">gọi hàm cần kiểm</text><text x="386" y="100" class="t-small">prop_assert_eq!</text><rect x="550" y="30" width="140" height="96" rx="10" class="f-warn-soft s-warn"/><text x="564" y="56" class="t-b">Shrinking</text><text x="564" y="80" class="t-small">thu nhỏ đầu vào</text><text x="564" y="100" class="t-small">vẫn fail thì giữ</text><rect x="728" y="30" width="140" height="96" rx="10" class="f-warn-soft s-warn"/><text x="742" y="56" class="t-b">Phản ví dụ</text><text x="742" y="80" class="t-small">nhỏ nhất, kèm seed</text><text x="742" y="100" class="t-small">ghi vào regressions</text><path d="M156 78 L188 78" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-proptest-property-test-bat-bien-so-cai-0-a)"/><path d="M334 78 L366 78" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-proptest-property-test-bat-bien-so-cai-0-a)"/><path d="M512 78 L544 78" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-proptest-property-test-bat-bien-so-cai-0-a)"/><path d="M690 78 L722 78" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-proptest-property-test-bat-bien-so-cai-0-a)"/><text x="556" y="146" class="t-small">chỉ khi có ca fail</text><rect x="16" y="168" width="848" height="48" rx="8" class="f-none s-line dash"/><text x="32" y="190" class="t-small">Cả 1.000 ca qua: test xanh. Có ca fail: runner không báo ngay mà thu nhỏ trước,</text><text x="32" y="207" class="t-small">rồi mới báo, để người đọc thấy ca nhỏ nhất thay vì ca ngẫu nhiên khổng lồ.</text></svg>
</div>
<figcaption>Hình 1. Một property test: ca fail đi qua shrinking trước khi được báo.</figcaption>
</figure>

## `proptest!` và strategy: máy sinh dữ liệu thế nào?

**Strategy** là giá trị mô tả cách sinh và cách thu nhỏ một kiểu dữ liệu. Macro `proptest!` nhận danh sách `tên in strategy` và chạy thân hàm nhiều lần như một `#[test]`. Hai tính chất của `Money` trong `src/money.rs` dùng `any::<i64>()`, strategy sinh `i64` trên toàn khoảng từ `i64::MIN` đến `i64::MAX`:

```rust
// src/money.rs (trong #[cfg(test)] mod tests, rút gọn)
proptest! {
    #[test]
    fn dong_round_trips(x in any::<i64>()) {
        prop_assert_eq!(Money::from_dong(x).dong(), x);
    }

    #[test]
    fn checked_add_is_commutative(a in any::<i64>(), b in any::<i64>()) {
        let (a, b) = (Money::from_dong(a), Money::from_dong(b));
        prop_assert_eq!(a.checked_add(b), b.checked_add(a));
    }
}
```

Dùng `prop_assert_eq!` thay `assert_eq!`: khi sai nó trả `Err` thay vì panic, nên lúc thu nhỏ không sinh hàng loạt thông điệp panic (`assert_eq!` vẫn hoạt động, vì proptest bắt panic). Ngoài `any::<T>()` còn có khoảng (`-MAX..=MAX`), `prop::sample::select` (chọn từ danh sách), `prop::collection::vec(strategy, 2..=8)` và tuple của các strategy.

Một lưu ý: `any::<i64>()` chia đều trên 2^64 giá trị, nên gần như không bao giờ ra số nhỏ hay đầu mút (thử 200.000 mẫu: không mẫu nào có trị tuyệt đối dưới 1 tỷ, không mẫu nào là `i64::MIN`). Muốn thử biên phải nói rõ, ta làm ở mục chia tiền. Hai bộ biến đổi quan trọng: `prop_map` đổi giá trị sinh ra thành giá trị khác (thu nhỏ vẫn chạy trên dữ liệu gốc rồi map lại), và `prop_filter` loại giá trị không thỏa điều kiện. Loại quá nhiều thì proptest dừng báo lỗi. Với điều kiện như "bút toán cân", đừng lọc, hãy xây.

## Làm sao sinh bút toán luôn cân?

Sinh ngẫu nhiên rồi giữ lại bút toán có tổng bằng 0 gần như không bao giờ thành công. Cách đúng: sinh n posting ngẫu nhiên, rồi **ép posting cuối bằng âm tổng của n-1 posting đầu**.

```rust
// src/strategies.rs
pub fn balanced_postings() -> impl Strategy<Value = Vec<Posting>> {
    prop::collection::vec((account(), amount()), 2..=8)
        .prop_map(|mut lines| {
            let last = lines.len() - 1;
            let sum: i64 = lines[..last].iter().map(|(_, dong)| *dong).sum();
            lines[last].1 = -sum;
            lines
                .into_iter()
                .map(|(account, dong)| Posting::new(account, Money::from_dong(dong)))
                .collect::<Vec<Posting>>()
        })
        // Loại bút toán toàn số 0 (hợp lệ nhưng vô nghĩa để thử).
        .prop_filter("có ít nhất một posting khác 0", |postings| {
            postings.iter().any(|p| p.amount != Money::ZERO)
        })
}
```

`amount()` giới hạn mỗi posting trong ±1.000 tỷ đồng để tổng tối đa 7 posting không tràn `i64`; tràn số đã có test riêng từ bài 10. Một bút toán mẫu (RNG cố định, `cargo run` in ra):

```text
Một bút toán sinh ngẫu nhiên (3 posting):
  psp:vixanh:clearing   -882.825.331.594 VND
  platform:revenue       778.690.109.276 VND
  customer:CU-001        104.135.222.318 VND
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 236" role="img" aria-label="Hình 2. Dựng bút toán luôn cân: hai posting đầu sinh ngẫu nhiên, posting cuối được đặt bằng âm tổng của các posting trước nên tổng cả bút toán bằng 0."><defs><marker id="rust-proptest-property-test-bat-bien-so-cai-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="24" width="250" height="84" rx="10" class="f-phys-soft s-phys"/><text x="36" y="50" class="t-b">posting 1 (ngẫu nhiên)</text><text x="36" y="72" class="t-small">psp:vixanh:clearing</text><text x="36" y="94" class="t-b">-882.825.331.594 VND</text><rect x="310" y="24" width="250" height="84" rx="10" class="f-phys-soft s-phys"/><text x="326" y="50" class="t-b">posting 2 (ngẫu nhiên)</text><text x="326" y="72" class="t-small">platform:revenue</text><text x="326" y="94" class="t-b">+778.690.109.276 VND</text><rect x="600" y="24" width="260" height="84" rx="10" class="f-warn-soft s-warn"/><text x="616" y="50" class="t-b">posting cuối (ép cân)</text><text x="616" y="72" class="t-small">customer:CU-001</text><text x="616" y="94" class="t-b">+104.135.222.318 VND</text><rect x="20" y="146" width="540" height="62" rx="10" class="f-pk-soft s-pk"/><text x="36" y="172" class="t-b">Tổng các posting trước = -104.135.222.318 VND</text><text x="36" y="194" class="t-small">posting cuối = -(tổng đó), nên tổng cả bút toán = 0</text><path d="M145 108 L145 142" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-proptest-property-test-bat-bien-so-cai-1-a)"/><path d="M435 108 L435 142" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-proptest-property-test-bat-bien-so-cai-1-a)"/><path d="M560 177 L730 177 L730 112" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-proptest-property-test-bat-bien-so-cai-1-a)"/></svg>
</div>
<figcaption>Hình 2. Dựng bút toán cân: n-1 posting ngẫu nhiên, posting cuối bằng âm tổng của chúng. Không cần thử rồi loại.</figcaption>
</figure>

## Bất biến nào của sổ cái đáng chứng minh?

Sáu tính chất trong `tests/ledger_properties.rs`, chỉ dùng API `pub`:

| Bất biến | Test |
| --- | --- |
| Mọi bút toán sinh ra cân và `try_new` nhận | `generated_entries_sum_to_zero` |
| Sau mọi dãy `post`, tổng toàn sổ và tổng các số dư bằng 0 | `ledger_stays_balanced_after_any_sequence` |
| Số dư từng tài khoản khớp mô hình tham chiếu (`HashMap` + `i128`) | `balances_match_a_reference_model` |
| Ghi lại cả dãy cùng key: không thêm bút toán, số dư không đổi | `posting_twice_is_the_same_as_posting_once` |
| Key trùng tự do: mỗi key một bút toán, xung đột bị từ chối, sổ vẫn cân | `colliding_keys_never_corrupt_the_ledger` |
| Lệch một posting `delta` đồng thì bị từ chối đúng `Unbalanced { sum: delta }` | `perturbed_entry_is_rejected` |

`ProptestConfig { cases: 1000 }` đặt một lần cho cả khối `proptest!`. Hai trong số đó:

```rust
// tests/ledger_properties.rs (rút gọn)
proptest! {
    #![proptest_config(ProptestConfig { cases: 1000, ..ProptestConfig::default() })]

    #[test]
    fn ledger_stays_balanced_after_any_sequence(sequence in strategies::entry_sequence()) {
        let ledger = post_all(&sequence);
        prop_assert_eq!(ledger.len(), sequence.len());
        prop_assert_eq!(ledger.grand_total(), Ok(Money::ZERO));
    }

    #[test]
    fn posting_twice_is_the_same_as_posting_once(sequence in strategies::entry_sequence()) {
        let mut ledger = post_all(&sequence);
        for (i, postings) in sequence.iter().enumerate() {
            let entry = JournalEntry::try_new(&format!("k-{i}"), postings.clone()).unwrap();
            prop_assert!(ledger.post(entry).unwrap().replayed);
        }
        prop_assert_eq!(ledger.len(), sequence.len());
    }
}
```

Mô hình tham chiếu đáng giá nhất: viết lại phép tính số dư theo cách đơn giản nhất rồi so với code thật.

Một cái bẫy: `matches!(e, LedgerError::IdempotencyConflict { .. })` đặt thẳng trong `prop_assert!` không biên dịch được, vì macro chèn text biểu thức vào chuỗi định dạng:

```text
error: invalid format string: expected `}`, found `.`
   --> tests/ledger_properties.rs:100:21
    |
100 |                     prop_assert!(matches!(e, LedgerError::IdempotencyConflict { .. }));
```

Sửa: `let is_conflict = matches!(…);` rồi `prop_assert!(is_conflict);`.

## Chia tiền cho nhiều bên mà không mất đồng

Lab cài `split` theo largest remainder: mỗi bên nhận phần nguyên trước, số đồng còn dư cộng thêm 1 đồng cho các bên có phần dư lớn nhất, hòa thì bên đứng trước được ưu tiên.

```rust
// src/split.rs
pub fn split(total: Money, weights: &[u32]) -> Result<Vec<Money>, SplitError> {
    let sum = weight_sum(weights)?;
    let magnitude = i128::from(total.dong()).abs();
    let mut shares = floor_shares(magnitude, weights, sum);
    let assigned: i128 = shares.iter().map(|&(base, _)| base).sum();
    let leftover = usize::try_from(magnitude - assigned).expect("số đồng dư nhỏ hơn số bên");

    // Phần dư lớn nhất đứng trước; hòa thì chỉ số nhỏ hơn đứng trước.
    let mut order: Vec<usize> = (0..shares.len()).collect();
    order.sort_by(|&a, &b| shares[b].1.cmp(&shares[a].1).then(a.cmp(&b)));
    for &i in order.iter().take(leftover) {
        shares[i].0 += 1;
    }
    Ok(to_money(&shares, total.dong() < 0))
}
```

Tính trung gian dùng `i128` để `tổng × trọng số` không tràn. Example test: `25.500` chia `[80, 20]` ra `20.400` và `5.100`; `10` chia `[1, 1, 1]` ra `4, 3, 3`. Property test khẳng định nhiều hơn:

```rust
// src/split.rs (trong #[cfg(test)], rút gọn)
proptest! {
    #[test]
    fn parts_sum_to_total((total, weights) in strategies::split_input()) {
        let parts = split(Money::from_dong(total), &weights).unwrap();
        prop_assert_eq!(parts.len(), weights.len());
        let sum: i128 = parts.iter().map(|m| i128::from(m.dong())).sum();
        prop_assert_eq!(sum, i128::from(total));
    }
}
```

`split_input()` ghép một tổng tiền với một vector 1 đến 8 trọng số `0..=1000`, qua `prop_filter` để tổng trọng số khác 0. Tổng tiền lấy từ `total_dong()`, trộn bốn nhóm bằng `prop_oneof!`:

```rust
// src/strategies.rs
pub fn total_dong() -> impl Strategy<Value = i64> {
    prop_oneof![
        3 => -100_000i64..=100_000, // cỡ tiền thật của một chuyến
        2 => any::<i64>(),          // mọi độ lớn
        1 => Just(i64::MIN),
        1 => Just(i64::MAX),
    ]
}
```

Hai tính chất khác: mỗi bên lệch phần chính xác dưới 1 đồng (`each_part_is_within_one_dong`), bên có trọng số 0 không nhận đồng nào (`zero_weight_gets_nothing`). Chạy `cargo run`:

```text
Chia 25.500 VND theo [80, 20]: 20.400 VND, 5.100 VND
Chia 10 VND theo [1, 1, 1]: 4 VND, 3 VND, 3 VND
```

## Shrinking: cố ý sai để xem phản ví dụ nhỏ nhất

Muốn tin một property test, hãy nhìn nó đỏ. Lab giữ thêm `split_floor_only`, bản **cố ý sai**: làm tròn xuống rồi quên phát số đồng dư. Test kiểm cùng bất biến nằm ở `tests/shrinking_demo.rs`, gắn `#[ignore]` để `cargo test` thường vẫn xanh. Chạy riêng bằng `cargo test --test shrinking_demo -- --ignored`:

```text
proptest: Saving this and future failures in .../tests/shrinking_demo.proptest-regressions
cc 7a65285b554c1f715ac1d6a8c4cb587d8cda5a5307dffead3946f3bce1145eab

thread 'floor_only_split_keeps_total' (…) panicked at tests/shrinking_demo.rs:8:1:
Test failed: assertion failed: `(left == right)`
  left: `0`,
 right: `1` at tests/shrinking_demo.rs:25.
minimal failing input: (total, weights) = (
    1,
    [
        1,
        1,
    ],
)
```

Phản ví dụ nhỏ nhất là chia **1 đồng cho hai bên bằng nhau**: mỗi bên được 0, tổng 0 thay vì 1. Output trên là của một lần chạy, đã rút gọn. RNG đổi theo lần chạy nên chuỗi `cc …` của bạn sẽ khác, và proptest có thể dừng ở `total = -1`, bản đối xứng của cùng lỗi (sáu lần thử: ba lần `1`, ba lần `-1`). Ca fail đầu tiên dài hơn nhiều; `cargo run` in cả hai nhờ RNG cố định:

```text
split_floor_only, phản ví dụ thô: total = 17332, weights = [188, 948, 321, 973, 341]
split_floor_only, sau shrinking:  total = 1, weights = [1, 1]
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 3. Shrinking thu nhỏ phản ví dụ của split_floor_only từ total 17332 với năm trọng số xuống total bằng 1 với hai trọng số bằng 1."><defs><marker id="rust-proptest-property-test-bat-bien-so-cai-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="16" y="28" width="440" height="104" rx="10" class="f-warn-soft s-warn"/><text x="32" y="54" class="t-b">Phản ví dụ thô (ca fail đầu tiên)</text><text x="32" y="82" class="t-mono">total = 17332</text><text x="32" y="104" class="t-mono">weights = [188, 948, 321, 973, 341]</text><rect x="604" y="28" width="260" height="104" rx="10" class="f-pk-soft s-pk"/><text x="620" y="54" class="t-b">Sau shrinking</text><text x="620" y="82" class="t-mono">total = 1</text><text x="620" y="104" class="t-mono">weights = [1, 1]</text><path d="M456 80 L598 80" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-proptest-property-test-bat-bien-so-cai-2-a)"/><text x="486" y="70" class="t-small">thu nhỏ dần</text><rect x="16" y="160" width="848" height="62" rx="8" class="f-none s-line dash"/><text x="32" y="184" class="t-small">Mỗi bước: bỏ bớt phần tử của weights hoặc đưa số về gần 0, rồi chạy lại test.</text><text x="32" y="204" class="t-small">Vẫn fail thì giữ bước đó, hết fail thì lùi. Dừng khi không còn bước nhỏ hơn mà vẫn fail.</text></svg>
</div>
<figcaption>Hình 3. Shrinking dẫn phản ví dụ thô về ca nhỏ nhất, đủ ngắn để đọc bằng mắt và suy ra nguyên nhân.</figcaption>
</figure>

Dòng `cc 7a65…` là seed của ca fail; proptest ghi nó vào file regressions, phần sau `#` chỉ là chú thích về ca đã thu nhỏ:

```text
cc 7a65285b554c1f715ac1d6a8c4cb587d8cda5a5307dffead3946f3bce1145eab # shrinks to (total, weights) = (1, [1, 1])
```

Lần chạy sau, proptest dựng lại ca đó từ seed và chạy nó **trước** các ca ngẫu nhiên, nên bug đã gặp không âm thầm quay lại; dự án thật nên commit file này. Ở lab, bản sai chỉ để làm mẫu nên ta xoá file sau khi xem. Bản sửa chính là `split` ở mục trước, thêm bước phát số đồng dư.

Để `cargo test` thường cũng kiểm shrinking, test `shrinking_reaches_a_tiny_counterexample` dùng `TestRunner` với RNG xác định và tắt `failure_persistence` nên không ghi file.

## Test đặt ở đâu: `#[cfg(test)]` hay `tests/`?

Quy tắc của [bài 10](/articles/rust-module-crate-cargo-test-clippy) vẫn đúng:

| Vị trí | Lab đặt gì | Lý do |
| --- | --- | --- |
| `#[cfg(test)] mod tests` trong `money.rs`, `split.rs` | Tính chất của một hàm hoặc một kiểu | Gần code, thấy được item private |
| `tests/ledger_properties.rs` | Bất biến của cả sổ cái qua API `pub` | Giống người dùng thật của thư viện |
| `tests/shrinking_demo.rs` | Demo shrinking, `#[ignore]` | Cố ý fail nên tách file riêng |
| `src/strategies.rs` (module `pub`) | Strategy dùng chung | Cả `main.rs` (`--verify`) lẫn `tests/` cùng gọi |

`proptest` nằm ở `[dependencies]` và strategy ở module `pub` chỉ vì binary của lab cũng chạy property trong `--verify`. Dự án thật để `proptest` ở `[dev-dependencies]`, strategy trong `#[cfg(test)]` hoặc `tests/common/mod.rs`.

Vị trí file regressions cần chú ý. Mặc định là `FileFailurePersistence::SourceParallel("proptest-regressions")`: từ file nguồn của test, proptest đi ngược lên cây thư mục tới thư mục đầu tiên chứa `lib.rs` hoặc `main.rs`, rồi ghi vào thư mục `proptest-regressions/` đặt **cạnh thư mục đó**. Test trong `src/split.rs` vì thế ghi `proptest-regressions/split.txt` ở gốc crate. Test trong `tests/` thì khác: gốc crate không có hai file đó nên proptest đi tiếp lên các thư mục cha. Không thấy, nó in cảnh báo `failed to find lib.rs or main.rs` rồi ghi cạnh file test, đuôi `.proptest-regressions`; một thư mục cha nào đó có `main.rs` thì file nằm ngoài crate. Vì vậy `shrinking_demo.rs` khai báo rõ `FileFailurePersistence::WithSource("proptest-regressions")`: luôn ghi cạnh file test.

Hai cách điều khiển lúc chạy: `cargo test -- --nocapture` hiện `println!` của test đang xanh (lab in vài bút toán mẫu bằng `show_sample_entries`); `PROPTEST_CASES=20000` đổi số ca mà không sửa code, ghi đè cả `cases: 1000` trong khối `proptest!` (macro đọc biến môi trường sau khi nhận config).

**Với người từ C#.** Công cụ tương ứng là FsCheck; test ở [bài về unit test và test double](/articles/csharp-unit-test-test-double) là example test.

| Ý | FsCheck (.NET) | proptest (Rust) |
| --- | --- | --- |
| Mô tả cách sinh dữ liệu | `Gen`, `Arbitrary` | `Strategy` |
| Khai báo tính chất | `[Property]` (gói FsCheck.Xunit) | `proptest! { #[test] fn … }` |
| Số ca mặc định | 100 | 256 |
| Thu nhỏ phản ví dụ | Có | Có |

Cột FsCheck theo tài liệu của FsCheck; lab này không chạy nó.

## Chạy ví dụ và kiểm chứng

Từ gốc repository:

```bash
cargo run --manifest-path examples/toi-noi-rust/19-proptest-bat-bien/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/19-proptest-bat-bien/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/19-proptest-bat-bien/Cargo.toml
cargo test --manifest-path examples/toi-noi-rust/19-proptest-bat-bien/Cargo.toml -- --ignored
```

Lệnh đầu in các khối đã trích, mở đầu bằng:

```text
Ghi 2 lần cùng key: replayed = false rồi true, sổ có 1 bút toán, tổng = 0 VND
```

`-- --verify` kiểm các giá trị trên và chạy 1.000 ca cho hai bất biến, rồi in `VERIFY OK` (exit code 0); sai thì thoát với mã 1. `cargo test` báo 24 unit test, 12 integration test, 3 doc test pass và 1 test `ignored`. Lệnh cuối cố ý fail và ghi đúng một file, `examples/toi-noi-rust/19-proptest-bat-bien/tests/shrinking_demo.proptest-regressions`; xoá nó sau khi xem.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Property test không phải chứng minh toán học: 1.000 ca vẫn có thể bỏ sót ca hiếm, và nó chỉ mạnh bằng strategy. Strategy của lab giới hạn ±1.000 tỷ đồng, tối đa 8 posting, 4 tài khoản. RNG cố định trong <code>shrink_demo</code> chỉ để bài in được kết quả lặp lại; test thật nên để RNG ngẫu nhiên và commit file regressions. Sổ cái vẫn nằm trong bộ nhớ, chưa có database, chưa có nhiều luồng ghi.</p>
</div>

**Bài tập:** viết thêm tính chất «chia đều cho n bên (n từ 1 đến 8), hai phần bất kỳ chênh nhau không quá 1 đồng». Strategy nên là gì? Có cần `prop_filter` không?

<details><summary>Xem lời giải</summary><p>Không cần filter: <code>total in any::&lt;i64&gt;()</code> và <code>n in 1usize..=8</code> đã luôn hợp lệ, trọng số là <code>vec![1; n]</code>. Test đã nằm trong <code>src/split.rs</code> (khối <code>proptest!</code> trong <code>#[cfg(test)]</code>):</p>
<pre><code class="language-rust">#[test]
fn equal_weights_differ_by_at_most_one(total in any::&lt;i64&gt;(), n in 1usize..=8) {
    let parts = split(Money::from_dong(total), &amp;vec![1; n]).unwrap();
    let dongs: Vec&lt;i128&gt; = parts.iter().map(|m| i128::from(m.dong())).collect();
    let (min, max) = (dongs.iter().min().unwrap(), dongs.iter().max().unwrap());
    prop_assert!(max - min &lt;= 1);
}</code></pre>
<p>Thử thay <code>split</code> bằng <code>split_floor_only</code>: test này vẫn xanh, vì bản sai chia đều thì mọi phần bằng nhau. Chỉ bất biến «tổng bằng tổng gốc» mới bắt được bug đó; vì thế cần nhiều bất biến.</p></details>

Bài trước: [bài 18](/articles/rust-serde-json-but-toan-decimal-thoi-gian). Bài tiếp: [bài 20](/articles/rust-tong-hop-ledger-core-cargo-doc-workspace).

## Tài liệu gốc

- [Proptest Book](https://proptest-rs.github.io/proptest/intro.html): hướng dẫn chính thức của proptest.
- [docs.rs: proptest](https://docs.rs/proptest/latest/proptest/): API, `ProptestConfig`, `FileFailurePersistence`, `TestRunner`.
- [docs.rs: proptest::strategy::Strategy](https://docs.rs/proptest/latest/proptest/strategy/trait.Strategy.html): `prop_map`, `prop_filter`, `prop_flat_map`.
- [The Rust Book: Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html).
- [Cargo Book: cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html).
