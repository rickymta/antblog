---
slug: rust-lam-tron-chia-tien-largest-remainder-khong-mat-dong
title: "Làm tròn và chia tiền không mất đồng: largest remainder"
summary: "Bài 46: vì sao làm tròn từng phần làm lệch đồng, các RoundingStrategy của rust_decimal, largest remainder tất định với số âm và i128, ghi quy tắc đồng lẻ vào bút toán, proptest tìm phản ví dụ."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, rounding, decimal, proptest]
language: vi
---

Chuyến ghép `TN-0012` có ba khách cùng chia cước 25.501 đồng. Service thanh toán lấy 25.501 chia 3 được 8.500,33, làm tròn thành 8.500 cho mỗi khách rồi gửi ba lệnh trừ ví. VíXanh đã thu 25.501, ví khách chỉ bị trừ 25.500, và sổ cái thiếu một đồng cho đến khi đối soát cuối ngày báo lệch.

Đây là bài 46 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài [19](/articles/rust-proptest-property-test-bat-bien-so-cai) đã có hàm `split` kiểu largest remainder để minh họa proptest. Bài này biến nó thành thành phần dùng thật: bên nhận có `key`, phá hòa tất định, số âm cho hoàn tiền, nối với `FeePolicy` của [bài 11](/articles/rust-trait-generics-chinh-sach-phi-lam-tron) và ghi quy tắc đã dùng cùng bút toán. [Bài 44](/articles/rust-vi-tai-xe-chia-tien-payout-theo-lo) và [bài 45](/articles/rust-khuyen-mai-tro-gia-but-toan-nhieu-ben-ngan-sach) chia hai bên bằng `FeePolicy::split`; thuật toán này dành cho lúc có nhiều bên hơn.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Chia 25.501 đồng theo tỷ lệ 80/20, làm tròn từng phần theo nửa ra xa 0. Hai phần cộng lại có bằng 25.501 không?</p>
<details><summary>Xem đáp án</summary><p><b>Có:</b> 20.401 + 5.100 = 25.501. Vì thế bug này hay lọt qua test: với hai bên, hai phần lẻ bù nhau. Chia 40/40/20 hoặc 50/50 thì tổng lệch ngay.</p></details>
</div>

## Làm tròn từng phần sai ở đâu, và vì sao test 80/20 vẫn xanh?

Cách ngây thơ: tính phần chính xác của từng bên rồi làm tròn từng phần. Lab giữ nó trong `src/naive.rs` để so sánh:

```rust
// src/naive.rs (rút gọn: bỏ chú thích)
pub fn round_each(total: i64, weights: &[u32], strategy: RoundingStrategy) -> Option<Vec<i64>> {
    let sum: u32 = weights
        .iter()
        .try_fold(0u32, |acc, &w| acc.checked_add(w))?;
    weights
        .iter()
        .map(|&w| round_dong(exact_share(total, w, sum)?, strategy))
        .collect()
}
```

Kết quả `cargo run`:

| Tổng | Trọng số | Từng phần làm tròn | Tổng các phần | Lệch |
| --- | --- | --- | --- | --- |
| 100 | `[1, 1, 1]` | 33, 33, 33 | 99 | -1 |
| 25.501 | `[80, 20]` | 20.401, 5.100 | 25.501 | 0 |
| 25.501 | `[40, 40, 20]` | 10.200, 10.200, 5.100 | 25.500 | -1 |
| 25.501 | `[50, 50]` | 12.751, 12.751 | 25.502 | +1 |

Với hai bên, hai phần lẻ cộng lại đúng bằng 1 (hoặc 0): 0,8 và 0,2 làm tròn thành 1 và 0, bù nhau. Chỉ điểm giữa (hai nửa đồng cùng làm tròn lên) làm vỡ quy luật, như dòng 50/50. Từ ba bên, các phần lẻ không còn bù nhau và lệch xuất hiện ở cả hai chiều.

`rust_decimal` không cứu được chuyện này. `Decimal` không biểu diễn hết 1/3: lab in `1/3 = 0.3333333333333333333333333333, ba lần 1/3 = 0.9999999999999999999999999999`. Và dù biểu diễn chính xác, làm tròn từng phần vẫn sai về cấu trúc: tổng các phần chưa bao giờ được so với tổng gốc.

## Bảy RoundingStrategy của rust_decimal cho kết quả khác nhau thế nào?

`RoundingStrategy` trả lời câu hỏi cho **một** giá trị. Bảng sau là output của lab; ba cột cuối là độ lệch tổng khi làm tròn từng phần theo strategy đó:

| Strategy | 2,4 | 2,5 | 3,5 | -2,5 | -2,6 | Lệch 100/`[1,1,1]` | Lệch 25.501/`[80,20]` | Lệch 25.501/`[50,50]` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `MidpointNearestEven` | 2 | 2 | 4 | -2 | -3 | -1 | 0 | -1 |
| `MidpointAwayFromZero` | 2 | 3 | 4 | -3 | -3 | -1 | 0 | +1 |
| `MidpointTowardZero` | 2 | 2 | 3 | -2 | -3 | -1 | 0 | -1 |
| `ToZero` | 2 | 2 | 3 | -2 | -2 | -1 | -1 | -1 |
| `AwayFromZero` | 3 | 3 | 4 | -3 | -3 | +2 | +1 | +1 |
| `ToNegativeInfinity` | 2 | 2 | 3 | -3 | -3 | -1 | -1 | -1 |
| `ToPositiveInfinity` | 3 | 3 | 4 | -2 | -2 | +2 | +1 | +1 |

Ba strategy `Midpoint*` chỉ khác nhau ở đúng điểm giữa; `ToZero` và `AwayFromZero` luôn cắt hoặc luôn đẩy; hai strategy hướng về vô cực phụ thuộc dấu. Không hàng nào có ba số 0 ở cột lệch. Strategy quyết định **một giá trị** được làm tròn ra sao, ví dụ phí 20% của một chuyến ([bài 3](/articles/rust-kieu-so-tien-vnd-i64-decimal) chọn `MidpointAwayFromZero`); bảo toàn tổng khi chia cho nhiều bên cần thuật toán khác. Người từ .NET gặp cùng vấn đề ở `MidpointRounding` ([bài về `decimal`](/articles/csharp-bien-kieu-du-lieu-decimal)).

## Largest remainder: phát phần nguyên trước, đồng lẻ cho phần dư lớn nhất

Ba bước: mỗi bên nhận phần nguyên của `tổng × trọng số / tổng trọng số`; đếm số đồng còn lại; cộng thêm 1 đồng cho các bên có phần dư lớn nhất. Mọi phép tính là số nguyên, không có sai số để tích lũy.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 304" role="img" aria-label="Hình 1. Largest remainder chia 25.501 đồng cho ba khách trọng số bằng nhau: mỗi bên nhận 8.500 đồng phần nguyên, còn dư 1 đồng, phần dư bằng nhau nên hòa được phá bằng key, customer:CU-001 nhận thêm 1 đồng thành 8.501."><defs><marker id="rust-lam-tron-chia-tien-largest-remainder-khong-mat-dong-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="16" y="24" width="190" height="100" rx="10" class="f-phys-soft s-phys"/><text x="30" y="50" class="t-b">1. Phần chính xác</text><text x="30" y="76" class="t-small">25.501 × 1 / 3</text><text x="30" y="98" class="t-small">= 8.500,333… mỗi bên</text><rect x="234" y="24" width="190" height="100" rx="10" class="f-phys-soft s-phys"/><text x="248" y="50" class="t-b">2. Phát phần nguyên</text><text x="248" y="76" class="t-small">8.500 × 3 = 25.500</text><text x="248" y="98" class="t-small">còn dư 1 đồng</text><rect x="452" y="24" width="190" height="100" rx="10" class="f-phys-soft s-phys"/><text x="466" y="50" class="t-b">3. Xếp theo phần dư</text><text x="466" y="76" class="t-small">ba phần dư bằng nhau</text><text x="466" y="98" class="t-small">hòa: key tăng dần</text><rect x="670" y="24" width="190" height="100" rx="10" class="f-pk-soft s-pk"/><text x="684" y="50" class="t-b">4. Cộng 1 đồng</text><text x="684" y="76" class="t-small">cho 1 bên đứng đầu</text><text x="684" y="98" class="t-small">tổng = 25.501</text><path d="M206 74 L228 74" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-lam-tron-chia-tien-largest-remainder-khong-mat-dong-0-a)"/><path d="M424 74 L446 74" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-lam-tron-chia-tien-largest-remainder-khong-mat-dong-0-a)"/><path d="M642 74 L664 74" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-lam-tron-chia-tien-largest-remainder-khong-mat-dong-0-a)"/><rect x="16" y="156" width="270" height="76" rx="10" class="f-pk-soft s-pk"/><text x="30" y="182" class="t-b">customer:CU-001</text><text x="30" y="208" class="t-small">8.500 + 1 = 8.501 VND</text><rect x="305" y="156" width="270" height="76" rx="10" class="f-phys-soft s-phys"/><text x="319" y="182" class="t-b">customer:CU-002</text><text x="319" y="208" class="t-small">8.500 VND</text><rect x="594" y="156" width="270" height="76" rx="10" class="f-phys-soft s-phys"/><text x="608" y="182" class="t-b">customer:CU-003</text><text x="608" y="208" class="t-small">8.500 VND</text><rect x="16" y="252" width="848" height="40" rx="8" class="f-none s-line dash"/><text x="32" y="277" class="t-small">Đảo thứ tự ba khách trong đầu vào không đổi kết quả: người nhận đồng lẻ do key quyết định, không do vị trí.</text></svg>
</div>
<figcaption>Hình 1. Largest remainder trên 25.501 đồng chia ba khách; đồng lẻ đi theo key nhỏ nhất.</figcaption>
</figure>

Phần lõi của `allocate`, cài trong `src/allocate.rs`:

```rust
// src/allocate.rs (rút gọn: bỏ chú thích)
pub fn allocate(total: Money, parties: &[Party]) -> Result<Vec<Share>, AllocError> {
    let weight_sum = checked_weight_sum(parties)?;
    let magnitude = i128::from(total.dong()).abs();

    let mut quotas: Vec<(i128, i128)> = parties
        .iter()
        .map(|p| {
            let exact = magnitude * i128::from(p.weight);
            (exact / weight_sum, exact % weight_sum)
        })
        .collect();
    let assigned: i128 = quotas.iter().map(|&(base, _)| base).sum();
    let leftover = usize::try_from(magnitude - assigned).map_err(|_| AllocError::Overflow)?;

    let mut order: Vec<usize> = (0..parties.len()).collect();
    order.sort_by_key(|&i| (Reverse(quotas[i].1), parties[i].key.as_str()));
    for &i in order.iter().take(leftover) {
        quotas[i].0 += 1;
    }
    // ... gắn dấu của `total` rồi đổi về Money
}
```

Ba chi tiết đáng dừng lại.

**Số nguyên rộng.** `|i64::MIN|` là 2^63 và trọng số `u64` gần 2^64, tích gần 2^127 nên vừa `i128`. Nhân bằng `i64` tràn rất sớm: lab in `25.500 × 1000000000000000: i64 = None, i128 = 25500000000000000000`.

**Số âm.** `/` và `%` của Rust cắt về 0, phần dư mang dấu số bị chia, nên chia thẳng số âm sẽ xếp sai thứ tự phần dư. `allocate` làm việc trên độ lớn rồi gắn dấu, nên `allocate(-t)` là đối xứng từng đồng của `allocate(t)`: `25.501 chia ba: 8.501 VND, 8.500 VND, 8.500 VND | hoàn: -8.501 VND, -8.500 VND, -8.500 VND`. Hoàn toàn bộ trả đồng lẻ về đúng người đã gánh nó.

**Phá hòa tất định.** Phần dư lớn nhất đứng trước, hòa thì `key` nhỏ hơn (so sánh byte) đứng trước. `key` là duy nhất (trùng thì `DuplicateKey`) nên thứ tự là toàn phần và kết quả chỉ phụ thuộc (key, trọng số, tổng): `100 chia ba, thứ tự a,b,c: 34 VND, 33 VND, 33 VND | thứ tự c,b,a: 33 VND, 33 VND, 34 VND`; key `a` nhận 34 trong cả hai. Phá hòa theo thứ tự duyệt `HashMap` là bug thật: mỗi `HashMap` có hạt giống băm riêng, qua 200 lần thử lab thấy nhiều hơn một bên nhận đồng lẻ, nên một lần thử lại có thể ra bút toán khác bản đã ghi.

Các biên đều có test: trọng số 0 không nhận đồng nào (`10 theo [1, 0, 1, 1]: 4 VND, 0 VND, 3 VND, 3 VND`); tổng trọng số 0 là `ZeroWeightSum`, danh sách rỗng là `NoParties`.

## Ai nhận đồng lẻ là quyết định nghiệp vụ: ghi vào bút toán

Toán học chỉ nói tổng phải khớp; ai nhận đồng lẻ thì kế toán và đối tác sẽ hỏi. Enum `Rule` trong `src/book.rs` có `LargestRemainder` và `Plug(key)` (các bên khác làm tròn, một bên nhận phần còn lại, như `FeePolicy::split` của `ledger-core`: phí làm tròn, tài xế là phần còn lại). Chia 6 đồng cho bốn bên bằng nhau (mỗi bên đáng 1,5 đồng):

```text
6 đồng cho 4 bên bằng nhau: phần còn lại cho d = 2 VND, 2 VND, 2 VND, 0 VND | largest remainder = 2 VND, 2 VND, 1 VND, 1 VND
```

Bên nhận «phần còn lại» chỉ được 0 đồng trong khi phần đúng là 1,5; với số liệu khác nó còn có thể âm. Largest remainder giữ mỗi bên dưới 1 đồng, nhưng không chỉ định sẵn ai chịu: quy tắc phải được nói ra.

`AllocatedFee` cắm largest remainder vào trait `FeePolicy` của bài 11, với hai key cố định `"driver"` và `"platform"`. Phí 50% trên cước 25.501 (cố ý lẻ), hai chính sách đặt cạnh nhau:

```text
50% · nửa ra xa 0                            → [-25501, 12751, 12750]
50% · largest remainder, hòa: key tăng dần   → [-25501, 12750, 12751]
```

Các posting theo thứ tự khách, nền tảng, tài xế: đồng lẻ chuyển từ nền tảng sang tài xế.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hình 2. Hai chính sách phí 50 phần trăm trên cước 25.501 đồng cho đồng lẻ đi hai hướng: PercentageFee nửa ra xa 0 cho nền tảng 12.751, AllocatedFee largest remainder cho tài xế 12.751; cả hai đều phải ghi quy tắc vào bản ghi, nếu không cùng key với nội dung khác sẽ bị IdempotencyConflict."><defs><marker id="rust-lam-tron-chia-tien-largest-remainder-khong-mat-dong-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="16" y="24" width="410" height="124" rx="10" class="f-phys-soft s-phys"/><text x="32" y="50" class="t-b">PercentageFee, nửa ra xa 0</text><text x="32" y="78" class="t-small">phí nền tảng = làm tròn(50% × 25.501)</text><text x="32" y="102" class="t-small">nền tảng 12.751, tài xế = phần còn lại 12.750</text><text x="32" y="130" class="t-b">đồng lẻ → nền tảng</text><rect x="454" y="24" width="410" height="124" rx="10" class="f-pk-soft s-pk"/><text x="470" y="50" class="t-b">AllocatedFee, largest remainder</text><text x="470" y="78" class="t-small">hai phần dư bằng nhau, hòa theo key</text><text x="470" y="102" class="t-small">driver &lt; platform nên tài xế 12.751</text><text x="470" y="130" class="t-b">đồng lẻ → tài xế</text><rect x="16" y="176" width="848" height="88" rx="10" class="f-warn-soft s-warn"/><text x="32" y="202" class="t-b">AllocationRecord ghi cùng bút toán</text><text x="32" y="226" class="t-small">rule=largest-remainder/v1 tie=key-asc total=-25501 weights=customer:CU-001:1,...</text><text x="32" y="248" class="t-small">Cùng idempotency_key mà đổi quy tắc, số tiền đổi theo: sổ từ chối bằng IdempotencyConflict.</text></svg>
</div>
<figcaption>Hình 2. Cùng một phí 50%, đồng lẻ đi hai hướng; quy tắc phải đi kèm bút toán.</figcaption>
</figure>

`AllocationRecord` của lab chuẩn hóa trọng số theo key, nên cùng quyết định cho cùng chuỗi dù đầu vào xếp thế nào:

```text
bản ghi quy tắc: rule=largest-remainder/v1 tie=key-asc total=-25501 weights=customer:CU-001:1,customer:CU-002:1,customer:CU-003:1
```

Chuỗi này nên đi cùng bút toán, ví dụ trong payload sự kiện `LedgerEntryPosted`. Đây là đề xuất của bài: schema chuẩn không có cột ghi chú, lab chỉ in chuỗi. `distribute_entry` thêm một dòng `pool` bằng âm tổng các phần nên bút toán luôn cân. Ba khách `TN-0012` ra `JE-000002: -8.501 VND, -8.500 VND, -8.500 VND`; ghi lại cùng quy tắc trả `replayed = true`. Đổi sang `Plug("customer:CU-003")` mà giữ key `trip:TN-0012:capture` thì sổ từ chối: `key trip:TN-0012:capture đã dùng cho bút toán khác nội dung`.

Hoàn toàn bộ: `allocate(+25.501)` cho đúng các posting của bút toán đảo mà `Ledger::reverse` tạo. Hoàn một phần nhiều lần phức tạp hơn, xem bài tập.

## VND không có phần lẻ, và những lỗi hay gặp

Tiền trong sổ cái là `Money(i64)` đơn vị đồng; VND không có đơn vị nhỏ hơn (bảng mã ISO 4217 ghi 0 chữ số thập phân, dùng để tham khảo), nên không có «xu» để chứa phần lẻ. Quy tắc rút ra: **làm tròn đúng một lần, ở biên, trên tổng gốc, bằng thuật toán bảo toàn tổng**. Mọi lệnh làm tròn thêm là chỗ đồng tiền rò rỉ. Lỗi hay gặp:

| Lỗi | Hậu quả | Cách tránh |
| --- | --- | --- |
| Làm tròn từng phần rồi cộng | Lệch ±1 đồng, test hai bên vẫn xanh | `allocate` một lần trên tổng gốc |
| Phá hòa theo thứ tự đầu vào hoặc `HashMap` | Thử lại ra bút toán khác, `IdempotencyConflict` | Phá hòa theo `key` cố định |
| Chia hai tầng, làm tròn ở tầng giữa | Sai số cộng dồn giữa các tầng | Gộp trọng số rồi chia một lần |
| Trọng số là `f64` hoặc phần trăm đã làm tròn | Tỷ lệ sai từ trước khi chia | Trọng số nguyên (phần vạn) lấy từ chính sách |
| Chia thẳng số âm bằng `/` và `%` | Phần dư mang dấu, xếp sai thứ tự | Chia độ lớn rồi gắn dấu |
| Nhân tổng với trọng số bằng `i64` | Tràn số ngay khi trọng số lớn | Nhân bằng `i128` |
| Một bên «nhận phần còn lại» không ghi rõ | Bên đó lệch hơn 1 đồng, có thể âm | Ghi quy tắc, hoặc dùng largest remainder |

## Property test chứng minh gì, và không chứng minh gì?

Test của lab nằm ở `tests/properties.rs`, mỗi property chạy 1.000 ca, tổng sinh đến cả `i64::MIN` và `i64::MAX` (riêng test bút toán dùng tổng cỡ ±1 tỷ, test hoàn tiền bỏ `i64::MIN` vì nó không có số đối):

| Bất biến | Test |
| --- | --- |
| Tổng các phần bằng tổng gốc | `parts_sum_to_total` |
| Mỗi phần lệch phần chính xác dưới 1 đồng (nhân chéo bằng `i128`) | `each_part_is_within_one_dong` |
| Đảo thứ tự đầu vào không đổi phần của từng key | `result_does_not_depend_on_input_order` |
| Trọng số 0 không nhận đồng nào | `zero_weight_gets_nothing` |
| Trọng số lớn hơn thì không nhận ít hơn (theo độ lớn) | `heavier_party_never_gets_less` |
| `allocate(-t)` là đối xứng của `allocate(t)` | `refund_mirrors_payment` |
| Phần chia dựng được bút toán cân, tổng toàn sổ bằng 0 | `distribution_is_a_balanced_entry` |

«Đơn điệu» cần nói đúng mức. Chứng minh được: trọng số A lớn hơn B thì phần của A không nhỏ hơn B (phần nguyên của A lớn hơn thì A đã hơn B ít nhất 1 đồng; bằng nhau thì A có phần dư lớn hơn nên đứng trước B trong hàng nhận đồng lẻ). Không chứng minh được, vì sai: tăng **tổng** thêm 1 đồng thì phần mỗi bên không giảm. Lab vét cạn và tìm ra nghịch lý Alabama:

```text
tăng tổng 3 → 4 theo trọng số [1, 3, 3]: [1, 1, 1] → [0, 2, 2]
```

Bên trọng số 1 nhận 1 đồng khi tổng là 3 nhưng 0 đồng khi tổng là 4. Test `total_monotonicity_does_not_hold` ghi sự thật này; hệ quả cho hoàn tiền một phần nằm ở bài tập.

Proptest cũng tìm phản ví dụ của bản ngây thơ. Với RNG cố định, nó bị bắt ở ca đầu `(20, [2, 2, 1, 3])`, sau shrinking còn `(9, [2, 1, 2])`: 9 đồng chia 2:1:2 ra 4, 2, 4, tổng 10. Chưa phải ca nhỏ nhất: `(1, [1, 1])` cũng sai (mỗi bên 0,5 làm tròn thành 1). Shrinking là tìm kiếm cục bộ, không vét cạn, nên có thể dừng ở cực tiểu cục bộ.

Về strategy: thử sửa tiêu chí phá hòa thành «theo vị trí trong đầu vào». Khi trọng số sinh trong khoảng rộng, test đảo thứ tự lúc xanh lúc đỏ (12 lần chạy trên máy viết bài, đỏ 5) vì phần dư hòa nhau hiếm. Với `tie_prone_parties()` (trọng số 0 đến 3, nhiều bên bằng nhau) cả 12 lần đều đỏ:

```rust
// tests/properties.rs
#[test]
fn result_does_not_depend_on_input_order(
    total in prop_oneof![-1_000i64..=1_000, total_dong()],
    (parties, shuffled) in tie_prone_parties()
        .prop_flat_map(|ps| (Just(ps.clone()), Just(ps).prop_shuffle())),
) {
    let a = allocate(Money::from_dong(total), &parties).unwrap();
    let b = allocate(Money::from_dong(total), &shuffled).unwrap();
    prop_assert_eq!(by_key(&a), by_key(&b));
}
```

Property chỉ mạnh bằng strategy; hãy thử phá code để xem test có đỏ thật. Đoạn trên đã rút gọn (bỏ dòng kiểm gọi lại cùng đầu vào).

## Chạy ví dụ và kiểm chứng

Từ gốc repository, không cần PostgreSQL:

```bash
cargo run --manifest-path examples/toi-noi-rust/46-largest-remainder/Cargo.toml
cargo run --manifest-path examples/toi-noi-rust/46-largest-remainder/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/46-largest-remainder/Cargo.toml
```

Lệnh đầu in sáu mục; mọi số trong bài lấy từ đó. `-- --verify` kiểm các giá trị này, chạy thêm 1.000 ca cho hai bất biến, rồi in `VERIFY OK` (exit code 0, sai thì exit 1). `cargo test` chạy 9 unit test và 13 test trong `tests/properties.rs`. Khi một property fail, proptest ghi `tests/properties.proptest-regressions`; dự án thật nên commit file đó.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Sổ cái ở đây là <code>Ledger</code> trong bộ nhớ của <code>ledger-core</code>: chưa có PostgreSQL, chưa lưu <code>AllocationRecord</code>. Cước 25.501 và phí 50% là số liệu chọn để thấy hiện tượng, không phải chính sách của Tới Nơi. Largest remainder không phải quy tắc hợp lệ duy nhất; điều bắt buộc là quy tắc cố định, tất định và được ghi lại.</p>
</div>

**Bài tập:** chuyến ghép được hoàn một phần hai lần, mỗi lần 10.000 đồng vào ví ba khách `a`, `b`, `c` (trọng số bằng nhau). Nếu mỗi lần gọi `allocate(10_000)` riêng, tổng mỗi khách được hoàn có bằng `allocate(20_000)` không? Cách nào làm chúng khớp?

<details><summary>Xem lời giải</summary><p>Không khớp. Mỗi lần chia riêng cho <code>[3.334, 3.333, 3.333]</code>, hai lần cộng thành <code>[6.668, 6.666, 6.666]</code>, trong khi <code>allocate(20_000)</code> cho <code>[6.667, 6.667, 6.666]</code>: khách <code>a</code> nhận cả hai đồng lẻ. Muốn khớp, tính theo lũy kế: phần lần hai bằng <code>allocate(20_000)</code> trừ phần đã hoàn lần một, tức <code>[3.333, 3.334, 3.333]</code>, tổng 10.000. Test <code>partial_refunds_are_consistent_only_when_cumulative</code> trong <code>src/allocate.rs</code> kiểm các số này. Vì nghịch lý Alabama, hiệu lũy kế có thể âm ở ca hiếm; bài này chưa giải quyết trường hợp đó.</p></details>

Bài trước: [bài 45](/articles/rust-khuyen-mai-tro-gia-but-toan-nhieu-ben-ngan-sach). Bài tiếp: [bài 47](/articles/rust-doi-soat-psp-sao-ke-csv-khop-lech-bao-cao).

## Tài liệu gốc

- [docs.rs: `rust_decimal::RoundingStrategy`](https://docs.rs/rust_decimal/latest/rust_decimal/enum.RoundingStrategy.html): bảy chiến lược làm tròn và ví dụ của từng cái.
- [docs.rs: `rust_decimal::Decimal`](https://docs.rs/rust_decimal/latest/rust_decimal/struct.Decimal.html): `round_dp_with_strategy`, thang tối đa 28 chữ số thập phân.
- [The Rust Reference: Arithmetic and Logical Binary Operators](https://doc.rust-lang.org/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators): `/` và `%` trên số âm, tràn số số nguyên.
- [std: `slice::sort_by_key`](https://doc.rust-lang.org/std/primitive.slice.html#method.sort_by_key) và [`std::cmp::Reverse`](https://doc.rust-lang.org/std/cmp/struct.Reverse.html).
- [docs.rs: `proptest`](https://docs.rs/proptest/latest/proptest/): strategy, `prop_shuffle`, `TestRunner`.
- [Proptest Book](https://proptest-rs.github.io/proptest/intro.html): shrinking và file regressions.
