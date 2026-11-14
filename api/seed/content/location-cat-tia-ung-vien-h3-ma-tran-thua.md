---
slug: location-cat-tia-ung-vien-h3-ma-tran-thua
title: "Cắt tỉa ứng viên: ma trận thưa ở quy mô thành phố"
summary: "Bài 44: cắt tỉa ứng viên bằng H3 k-ring và k gần nhất. Khi xe gấp 4 lần khách, tổng ETA tệ đi 2,4–4,6%; khi xe bằng khách, mất 4–16% số khách mỗi lô. Bộ giải làm việc trên ma trận đặc nên vùng khổng lồ vẫn chậm. Trên dữ liệu mô phỏng."
category: location-routing
level: advanced
tags: [toi-noi, location, rust, dispatch, matching, h3, sparse-matrix, latency]
language: vi
---

Trong simulator của loạt bài, lô ghép chỉ có vài khách và ma trận ETA vài trăm ô. Nếu quy mô lớn hơn nhiều, một lô có thể tới hàng nghìn khách và hàng nghìn tài xế, và ma trận đầy đủ 3.000 × 3.000 có 9 triệu ô. Lab đo: dựng ma trận ETA đầy đủ cỡ đó mất 3,46 s trên máy đang quá tải; Hungarian chạy trên ma trận đầy đủ cỡ đó, ở một lô riêng, mất 4,50 s, cũng khi máy quá tải. Cắt tỉa là cách quen thuộc để giảm tải: với mỗi khách chỉ xét vài tài xế ở gần, bỏ phần còn lại.

Nhưng cắt tỉa là một phép cắt có hướng, và bài này đo cái giá. Khi xe gấp 4 lần khách, cấu hình ở giữa làm tổng ETA tệ đi 2,4–4,6% và trong lần đo này nhanh hơn 1,9–18,3 lần (khoảng rộng đó phần lớn là nhiễu tải máy). Khi số xe bằng số khách, cắt tỉa làm mất 4–16% số khách của mỗi lô, và lượt bù để lấy lại số khách lại khiến kết quả tệ hơn tối ưu tới 13%. Ở cỡ thành phố, cắt tỉa không rút ngắn được khâu giải, vì bộ giải trong lab vẫn làm việc trên ma trận đặc.

Đây là bài 44 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng H3 và k-ring của [bài 08](/articles/location-h3-geohash-chi-muc-luoi-k-ring), ma trận ETA của [bài 26](/articles/location-ma-tran-many-to-many-bucket-ch), phép gán Hungarian của [bài 42](/articles/location-ghep-theo-lo-hungarian-kuhn-munkres-rust), và cách đọc so sánh có cặp 20 seed của [bài 41](/articles/location-gan-nhat-sai-o-dau-do-bang-mo-phong). Mọi số đo đều lấy **trên dữ liệu mô phỏng**, với đồ thị HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors). **Mọi số thời gian (ms, s) đo khi máy chịu tải rất nặng** (load average khoảng 370–430 trên 10 lõi): số đếm và sai số tổng ETA tái lập được, còn thời gian thì không. Có những chỗ ngay cả tỉ lệ cũng nhiễu, bài sẽ chỉ ra.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Mỗi khách chỉ giữ 8 tài xế gần nhất (trong H3 k-ring 2). Với 200 khách và 200 tài xế, có 200 cặp để ghép. Theo bạn, trung bình có bao nhiêu khách được ghép khi cắt tỉa, và có lô nào ghép đủ 200 không?</p>
<details><summary>Xem đáp án</summary><p>Trung bình 181,1 trên 200 khách, và <b>0/10 lô ghép đủ</b>. Khi mọi tài xế phải được dùng, một khách mà cả 8 ứng viên đều đã bị người khác lấy sẽ không còn xe nào để ghép. Lượt bù (ghép nốt phần thừa trên ma trận đầy đủ) khôi phục đủ 200 cặp, nhưng tổng ETA tệ hơn tối ưu +10,25%.</p></details>
</div>

## Vì sao phải cắt tỉa: chi phí nằm ở hai nơi

Một lần ghép gồm hai khâu: dựng ma trận ETA (bài 26) và giải phân công (bài 42). Lab đo cả hai ở các cỡ vuông, trên lô lấy từ bộ sinh của simulator (kho 3.000 tài xế, 5.208 khách). Trung vị của 10 lô, đo khi máy chịu tải:

| Cỡ | ETA ma trận (ms) | Hungarian (ms) | Tham lam toàn cục / Hungarian (tổng ETA) |
|---|---:|---:|---:|
| 200 × 200 | 16,9 | 19,2 | 1,122 |
| 500 × 500 | 103,8 | 108,1 | 1,129 |
| 800 × 800 | 310,3 | 573,1 | 1,130 |
| 100 × 400 | 72,6 | 0,2 | 1,012 |
| 250 × 1.000 | 268,4 | 1,5 | 1,015 |
| 400 × 1.600 | 382,9 | 22,6 | 1,012 |

Hai điều đáng nhìn. Khi số khách bằng số xe, cả hai khâu đều tốn kém, và Hungarian chiếm gần hai phần ba thời gian ở 800 × 800. Khi xe gấp 4 lần khách, **Hungarian chỉ còn 0,2–22,6 ms** (ma trận chữ nhật có ít hàng hơn cột), còn khâu dựng ma trận ETA mới là chi phí chính. Cột cuối cho một mốc so sánh cần nhớ về sau: ở xe gấp 4 lần khách, tham lam toàn cục chỉ tệ hơn tối ưu 1,2–1,5%, trong khi các phép cắt tỉa dưới đây còn tệ hơn mức đó.

## Cắt tỉa: k-ring, k gần nhất, tách vùng độc lập

Ý tưởng gồm ba bước, đều nằm trong crate `matcher::prune`.

1. **Ứng viên theo khách.** Lập chỉ mục H3 các tài xế (độ phân giải 8, cạnh ô cỡ 530 m; độ phân giải 7 cỡ 1,4 km). Với mỗi khách, lấy các tài xế trong k-ring quanh ô của điểm đón; nếu chưa đủ `k_nearest` thì nới thêm một vòng, tới `k_ring_max`; rồi giữ `k_nearest` tài xế gần nhất theo khoảng cách chim bay.
2. **Chỉ tính ETA cho cặp ứng viên.** Ma trận trở thành thưa: danh sách cạnh `(khách, tài xế, ETA)`.
3. **Tách vùng độc lập.** Đồ thị hai phía thưa tách thành các thành phần liên thông (union-find); mỗi thành phần giải Hungarian riêng trên ma trận đặc nhỏ.

`examples/toi-noi-location/rust/crates/matcher/src/prune.rs`

```rust
pub struct PruneConfig {
    /// Độ phân giải H3 của chỉ mục tài xế (8: cạnh ô ~ 530 m; 7: ~ 1,4 km).
    pub res: u8,
    /// Bán kính k-ring ban đầu quanh ô của điểm đón.
    pub k_ring: u32,
    /// Giữ tối đa chừng này tài xế gần nhất cho mỗi khách.
    pub k_nearest: usize,
    /// Nếu k-ring không đủ `k_nearest` ứng viên thì nới thêm 1 vòng, tới `k_ring_max`; vẫn thiếu thì lấy những gì có.
    pub k_ring_max: u32,
    /// Lượt bù: sau khi giải theo vùng, khách và tài xế còn thừa được ghép lại trên ma trận đầy đủ của riêng họ.
    pub leftover: bool,
}
```

`examples/toi-noi-location/rust/crates/matcher/src/prune.rs`

```rust
pub fn candidates(drivers: &[LatLng], pickups: &[LatLng], cfg: PruneConfig) -> Vec<Vec<usize>> {
    let res = resolution(cfg.res);
    let mut index: FxHashMap<u64, Vec<usize>> = FxHashMap::default();
    for (i, &d) in drivers.iter().enumerate() {
        index.entry(u64::from(cell_of(d, res))).or_default().push(i);
    }
    pickups
        .iter()
        .map(|&p| {
            let cell = cell_of(p, res);
            let mut k = cfg.k_ring;
            let mut found: Vec<usize>;
            loop {
                found = k_ring(cell, k)
                    .into_iter()
                    .flat_map(|c| index.get(&u64::from(c)).cloned().unwrap_or_default())
                    .collect();
                if found.len() >= cfg.k_nearest || k >= cfg.k_ring_max {
                    break;
                }
                k += 1;
            }
            found.sort_by(|&a, &b| {
                haversine_m(drivers[a], p)
                    .total_cmp(&haversine_m(drivers[b], p))
                    .then(a.cmp(&b))
            });
            found.truncate(cfg.k_nearest);
            found
        })
        .collect()
}
```

Một bất biến được kiểm bằng cả proptest lẫn `assert!` ngay trong lab: **cắt tỉa không bao giờ rẻ hơn tối ưu đầy đủ khi cùng số cặp**. Lý do nằm ở quan hệ tập hợp: mọi phép gán trên ma trận thưa cũng là phép gán trên ma trận đầy đủ, nên nếu nó ghép đủ số cặp tối đa thì tổng ETA của nó không thể nhỏ hơn tối ưu. Bất biến chỉ đúng khi cùng số cặp: cắt tỉa có thể ghép **ít hơn**, và khi đó tổng ETA nhỏ hơn không có nghĩa gì. Chuyện đó có xảy ra, như mục kế tiếp cho thấy. Proptest còn kiểm rằng giải từng vùng độc lập cho đúng tối ưu của ma trận thưa viết dưới dạng đặc: tách vùng không làm mất tối ưu, chỉ cắt tỉa mới làm mất.

Năm cấu hình được ghi trước khi đo, đều trên H3 độ phân giải 8 trừ khi ghi khác: `k-ring 1 + 4 gần nhất`; `k-ring 2 + 8 gần nhất`; `k-ring 2 + 8 gần nhất + lượt bù`; `k-ring 3 + 16 gần nhất`; và `độ phân giải 7, k-ring 1 + 8 gần nhất`. «Lượt bù» ghép lại khách và xe còn thừa sau khi giải theo vùng, trên ma trận đầy đủ của riêng nhóm này.

## Khi xe gấp 4 lần khách: giá 2,4–4,6% tổng ETA

Đây là tình huống gần thực tế nhất trong một giờ bình thường: xe rảnh nhiều hơn khách chờ. Mục A của lab đo 10 lô mỗi cỡ (seed 44001), sai số = tổng ETA cắt tỉa chia tổng ETA tối ưu đầy đủ trừ 1, trên các lô mà cắt tỉa ghép đủ số cặp (ở các cỡ này gần như mọi lô).

| Cỡ | Cấu hình | Cạnh/lô | Mật độ | Sai số tổng ETA [KTC 95%] | Nhanh hơn đầy đủ |
|---|---|---:|---:|---|---:|
| 100 × 400 | k-ring 1, 4 gần nhất | 400 | 1,00% | +5,64% [+3,60; +7,67] | 35,4× |
| | k-ring 2, 8 gần nhất | 800 | 2,00% | +2,43% [+0,69; +4,18] | 18,3× |
| | k-ring 3, 16 gần nhất | 1.600 | 4,00% | +0,68% [+0,14; +1,23] | 0,9× |
| 250 × 1.000 | k-ring 1, 4 gần nhất | 1.000 | 0,40% | +12,75% [+9,80; +15,70] | 3,3× |
| | k-ring 2, 8 gần nhất | 2.000 | 0,80% | +3,70% [+1,73; +5,67] | 3,7× |
| | k-ring 3, 16 gần nhất | 4.000 | 1,60% | +1,04% [+0,15; +1,92] | 1,4× |
| 400 × 1.600 | k-ring 1, 4 gần nhất | 1.600 | 0,25% | +13,20% [+12,00; +14,40] | 4,9× |
| | k-ring 2, 8 gần nhất | 3.200 | 0,50% | +4,58% [+3,28; +5,89] | 1,9× |
| | k-ring 3, 16 gần nhất | 6.400 | 1,00% | +1,62% [+0,73; +2,50] | 1,1× |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 464" role="img" aria-label="Hình 1. Sai số tổng ETA của ghép cắt tỉa so với Hungarian đầy đủ, khi số xe gấp 4 lần số khách. k-ring 1 với 4 gần nhất sai từ 5,64 đến 13,20 phần trăm; k-ring 2 với 8 gần nhất sai từ 2,43 đến 4,58 phần trăm; k-ring 3 với 16 gần nhất sai từ 0,68 đến 1,62 phần trăm nhưng thời gian gần bằng bản đầy đủ."><rect x="20" y="20" width="840" height="439" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Sai số tổng ETA so với tối ưu đầy đủ, khi xe gấp 4 lần khách (H3 độ phân giải 8)</text><text x="40" y="84" class="t-b">100 khách × 400 xe</text><text x="52" y="115">k-ring 1, 4 gần nhất</text><rect x="290" y="103" width="97.8" height="16" rx="2" class="f-warn"/><text x="395.8" y="115" class="t-mono">+5,64%   (35,4× so với đầy đủ)</text><text x="52" y="141">k-ring 2, 8 gần nhất</text><rect x="290" y="129" width="42.1" height="16" rx="2" class="f-pk"/><text x="340.1" y="141" class="t-mono">+2,43%   (18,3× so với đầy đủ)</text><text x="52" y="167">k-ring 3, 16 gần nhất</text><rect x="290" y="155" width="11.8" height="16" rx="2" class="f-phys"/><text x="309.8" y="167" class="t-mono">+0,68%   (0,9× so với đầy đủ)</text><text x="40" y="192" class="t-b">250 khách × 1.000 xe</text><text x="52" y="223">k-ring 1, 4 gần nhất</text><rect x="290" y="211" width="221.0" height="16" rx="2" class="f-warn"/><text x="519.0" y="223" class="t-mono">+12,75%   (3,3× so với đầy đủ)</text><text x="52" y="249">k-ring 2, 8 gần nhất</text><rect x="290" y="237" width="64.1" height="16" rx="2" class="f-pk"/><text x="362.1" y="249" class="t-mono">+3,70%   (3,7× so với đầy đủ)</text><text x="52" y="275">k-ring 3, 16 gần nhất</text><rect x="290" y="263" width="18.0" height="16" rx="2" class="f-phys"/><text x="316.0" y="275" class="t-mono">+1,04%   (1,4× so với đầy đủ)</text><text x="40" y="300" class="t-b">400 khách × 1.600 xe</text><text x="52" y="331">k-ring 1, 4 gần nhất</text><rect x="290" y="319" width="228.8" height="16" rx="2" class="f-warn"/><text x="526.8" y="331" class="t-mono">+13,20%   (4,9× so với đầy đủ)</text><text x="52" y="357">k-ring 2, 8 gần nhất</text><rect x="290" y="345" width="79.4" height="16" rx="2" class="f-pk"/><text x="377.4" y="357" class="t-mono">+4,58%   (1,9× so với đầy đủ)</text><text x="52" y="383">k-ring 3, 16 gần nhất</text><rect x="290" y="371" width="28.1" height="16" rx="2" class="f-phys"/><text x="326.1" y="383" class="t-mono">+1,62%   (1,1× so với đầy đủ)</text><path d="M290 70 L290 394" class="f-none s-muted" stroke-width="1.5"/><rect x="40" y="412" width="12" height="12" rx="2" class="f-warn"/><text x="58" y="422" class="t-small">k-ring 1, 4 gần nhất</text><rect x="208" y="412" width="12" height="12" rx="2" class="f-pk"/><text x="226" y="422" class="t-small">k-ring 2, 8 gần nhất</text><rect x="376" y="412" width="12" height="12" rx="2" class="f-phys"/><text x="394" y="422" class="t-small">k-ring 3, 16 gần nhất</text><text x="40" y="444" class="t-small">10 lô mỗi cỡ, seed 44001. Cột trong ngoặc: tốc độ dựng ma trận + giải so với đầy đủ, đo khi máy chịu tải nặng, chỉ đọc tỉ lệ. Trên dữ liệu mô phỏng.</text></svg>
</div>
<figcaption>Hình 1. Sai số tổng ETA so với tối ưu đầy đủ theo cấu hình cắt tỉa, ba cỡ lô với xe gấp 4 lần khách. Cắt càng mạnh càng sai. Tốc độ đo khi máy chịu tải nên nhiễu: ở 250 × 1.000, k-ring 1 không nhanh hơn k-ring 2. Trên dữ liệu mô phỏng.</figcaption>
</figure>

Ba cách đọc. **Cột sai số tái lập được, cột tốc độ thì không tin được từng ô.** Ở 250 × 1.000, `k-ring 1` với 1.000 cạnh lại chậm hơn `k-ring 2` với 2.000 cạnh (3,3× so với 3,7×), điều này chỉ có thể do nhiễu; và 18,3× ở 100 × 400 so với 3,7× ở 250 × 1.000 không nên đọc thành quy luật. **`k-ring 2 + 8 gần nhất` là điểm giữa**: sai 2,4–4,6%, thường nhanh hơn bản đầy đủ. `k-ring 1` sai 5,6–13,2%; `k-ring 3` sai chỉ 0,7–1,6% mà thời gian ngang bản đầy đủ (0,9–1,4×).

**Và đừng quên mốc tham lam.** Ở các cỡ này, tham lam toàn cục trên ma trận đầy đủ chỉ tệ hơn tối ưu 1,2–1,5% (bảng ở trên), tức là **tốt hơn cả `k-ring 2` (+2,43% đến +4,58%)** về chất lượng. Điều đó không có nghĩa là tham lam rẻ hơn: nó vẫn cần ma trận đầy đủ (72,6–382,9 ms ở các cỡ này), còn cắt tỉa tránh được khâu đó. Nhưng nếu thứ bạn tiết kiệm là Hungarian (1,5 ms, 22,6 ms) thì cắt tỉa không đáng; nếu thứ bạn tiết kiệm là ETA thì đáng. Phải biết mình tiết kiệm cái gì trước khi chọn cấu hình.

**Ai lợi, ai thiệt:** máy chủ được lợi về độ trễ; khách và tài xế chịu thêm 0,7–13,2% tổng thời gian đón của lô, vì ghép kém tối ưu hơn.

## Khi xe bằng khách: cắt tỉa làm mất khách

Tình huống ngược lại, xe bằng khách, là lúc mọi tài xế phải được dùng, và mục A cũng đo nó (200, 500 và 800 khách, mỗi cỡ 10 lô).

| Cỡ | Cấu hình | Khách ghép TB | Số lô đủ cặp | Sai số tổng ETA [KTC 95%] | Nhanh hơn đầy đủ |
|---|---|---:|:-:|---|---:|
| 200 × 200 | k-ring 2, 8 gần nhất | 181,1 / 200 | 0/10 | n/a | 1,3× |
| | k-ring 2, 8 + lượt bù | 200,0 / 200 | 10/10 | +10,25% [+8,77; +11,72] | 0,5× |
| | k-ring 3, 16 gần nhất | 190,8 / 200 | 2/10 | +8,50% [−11,32; +28,33] (chỉ 2 lô) | 0,8× |
| 500 × 500 | k-ring 2, 8 gần nhất | 462,4 / 500 | 0/10 | n/a | 1,6× |
| | k-ring 2, 8 + lượt bù | 500,0 / 500 | 10/10 | +12,52% [+10,91; +14,13] | 1,0× |
| 800 × 800 | k-ring 2, 8 gần nhất | 724,1 / 800 | 0/10 | n/a | 2,1× |
| | k-ring 2, 8 + lượt bù | 800,0 / 800 | 10/10 | +12,97% [+11,64; +14,30] | 1,1× |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 464" role="img" aria-label="Hình 2. Số khách không được ghép trung bình mỗi lô khi số xe bằng số khách. Ở 200 khách, k-ring 1 mất 32,4 khách, k-ring 2 mất 18,9, k-ring 3 mất 9,2. Ở 500 khách mất 71,7, 37,6 và 18,3. Ở 800 khách mất 126,3, 75,9 và 47,9. Lượt bù ghép đủ mọi khách nhưng sai số tổng ETA tăng 10,25 đến 12,97 phần trăm."><rect x="20" y="20" width="840" height="439" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Khách không được ghép mỗi lô khi số xe bằng số khách (trung bình 10 lô)</text><text x="40" y="84" class="t-b">200 khách × 200 xe</text><text x="52" y="115">k-ring 1, 4 gần nhất</text><rect x="290" y="103" width="57.6" height="16" rx="2" class="f-warn"/><text x="355.6" y="115" class="t-mono">32,4   (16,2% số khách; 0/10 lô đủ cặp)</text><text x="52" y="141">k-ring 2, 8 gần nhất</text><rect x="290" y="129" width="33.6" height="16" rx="2" class="f-pk"/><text x="331.6" y="141" class="t-mono">18,9   (9,5% số khách; 0/10 lô đủ cặp)</text><text x="52" y="167">k-ring 3, 16 gần nhất</text><rect x="290" y="155" width="16.4" height="16" rx="2" class="f-phys"/><text x="314.4" y="167" class="t-mono">9,2   (4,6% số khách; 2/10 lô đủ cặp)</text><text x="40" y="192" class="t-b">500 khách × 500 xe</text><text x="52" y="223">k-ring 1, 4 gần nhất</text><rect x="290" y="211" width="127.5" height="16" rx="2" class="f-warn"/><text x="425.5" y="223" class="t-mono">71,7   (14,3% số khách; 0/10 lô đủ cặp)</text><text x="52" y="249">k-ring 2, 8 gần nhất</text><rect x="290" y="237" width="66.8" height="16" rx="2" class="f-pk"/><text x="364.8" y="249" class="t-mono">37,6   (7,5% số khách; 0/10 lô đủ cặp)</text><text x="52" y="275">k-ring 3, 16 gần nhất</text><rect x="290" y="263" width="32.5" height="16" rx="2" class="f-phys"/><text x="330.5" y="275" class="t-mono">18,3   (3,7% số khách; 0/10 lô đủ cặp)</text><text x="40" y="300" class="t-b">800 khách × 800 xe</text><text x="52" y="331">k-ring 1, 4 gần nhất</text><rect x="290" y="319" width="224.5" height="16" rx="2" class="f-warn"/><text x="522.5" y="331" class="t-mono">126,3   (15,8% số khách; 0/10 lô đủ cặp)</text><text x="52" y="357">k-ring 2, 8 gần nhất</text><rect x="290" y="345" width="134.9" height="16" rx="2" class="f-pk"/><text x="432.9" y="357" class="t-mono">75,9   (9,5% số khách; 0/10 lô đủ cặp)</text><text x="52" y="383">k-ring 3, 16 gần nhất</text><rect x="290" y="371" width="85.2" height="16" rx="2" class="f-phys"/><text x="383.2" y="383" class="t-mono">47,9   (6,0% số khách; 0/10 lô đủ cặp)</text><path d="M290 70 L290 394" class="f-none s-muted" stroke-width="1.5"/><rect x="40" y="412" width="12" height="12" rx="2" class="f-warn"/><text x="58" y="422" class="t-small">k-ring 1, 4 gần nhất</text><rect x="208" y="412" width="12" height="12" rx="2" class="f-pk"/><text x="226" y="422" class="t-small">k-ring 2, 8 gần nhất</text><rect x="376" y="412" width="12" height="12" rx="2" class="f-phys"/><text x="394" y="422" class="t-small">k-ring 3, 16 gần nhất</text><text x="40" y="444" class="t-small">10 lô mỗi cỡ. Với «lượt bù» số khách mất bằng 0 nhưng tổng ETA tệ hơn tối ưu từ +10,25% đến +12,97%. Trên dữ liệu mô phỏng.</text></svg>
</div>
<figcaption>Hình 2. Số khách không được ghép trung bình mỗi lô khi xe bằng khách. Cắt càng mạnh càng mất nhiều khách (k-ring 1 mất khoảng 14–16%); ngay cả k-ring 3 với 16 ứng viên vẫn mất 4–6%. Trên dữ liệu mô phỏng.</figcaption>
</figure>

Cơ chế dễ thấy: khi số xe bằng số khách, lời giải tối ưu là một **ghép hoàn hảo**, tức khách nào cũng phải có xe riêng. Cắt tỉa giữ cho mỗi khách 8 xe gần, nhưng nhiều khách chia nhau cùng vài xe ở chỗ đông; xe ở nơi vắng thì không là ứng viên của ai trong tầm gần. Những khách còn lại hết ứng viên khả dụng, dù trên ma trận đầy đủ họ vẫn có xe ở xa hơn. Ở ba cỡ này cắt tỉa theo vùng ghép đủ cặp ở **0/10 lô** (với k-ring 2 + 8 gần nhất); chỉ k-ring 3 với 16 ứng viên thỉnh thoảng đủ (2/10 ở 200 × 200, 0/10 ở hai cỡ còn lại).

Lượt bù chữa được số khách nhưng đắt: lượt bù khôi phục đủ cặp nhưng tổng ETA tệ hơn tối ưu +10,25% đến +12,97%, và tổng thời gian không còn nhanh hơn bản đầy đủ (0,5× đến 1,1×). Nghĩa là **cắt tỉa ở tỉ lệ 1:1 không thắng cả về chất lượng lẫn tốc độ.**

Hạn chế của mục này: lab chưa chạy trường hợp ngược, **thừa khách** (nhiều khách hơn xe, như một đợt tan sự kiện), nên chưa biết cắt tỉa ứng xử ra sao ở đó. Trong simulator, lô lớn nhất đo được là 47 khách ở cao điểm, nên các cỡ 200–800 là thử nghiệm chịu tải, không phải tình huống thường gặp.

## Cỡ thành phố 3.000 × 3.000: phần chậm không nằm ở nơi ta cắt

Cỡ cuối là 3.000 khách × 3.000 xe, mỗi cấu hình đúng 1 lô. Lần chạy chính không có Hungarian trên ma trận đầy đủ ở cỡ này, nên mốc so sánh của bảng là tham lam toàn cục trên ma trận đầy đủ: 9.000.000 ô ETA dựng mất 3,46 s, tham lam ghép 3.000 cặp với tổng 475.535 s (158,5 s mỗi cặp). Đây là mốc tham lam, không phải mốc tối ưu.

| Cấu hình | Cạnh | Số vùng | Vùng lớn nhất | Cặp ghép | s/cặp | Cắt tỉa (ms) | ETA cặp (ms) | Giải (ms) | Tổng (s) |
|---|---:|---:|---|---:|---:|---:|---:|---:|---:|
| k-ring 1, 4 gần nhất | 12.000 | 48 | 2.686 × 2.285 | 2.348 | 83,1 | 479,8 | 100,5 | 3.563,3 | 4,14 |
| k-ring 2, 8 gần nhất | 24.000 | 3 | 2.993 × 2.948 | 2.634 | 89,5 | 945,8 | 195,2 | 15.016,6 | 16,16 |
| k-ring 2, 8 + lượt bù | 24.000 | 3 | 2.993 × 2.948 | 3.000 | 160,7 | 1.343,4 | 241,5 | 23.513,5 | 25,10 |
| k-ring 3, 16 gần nhất | 48.000 | 1 | 3.000 × 3.000 | 2.765 | 96,2 | 2.458,2 | 367,9 | 3.271,3 | 6,10 |
| H3 r7, k-ring 1, 8 gần nhất | 24.000 | 3 | 2.993 × 2.948 | 2.634 | 89,5 | 2.429,3 | 238,2 | 3.643,4 | 6,31 |

Ba điều cần đọc kỹ, vì bảng này dễ gây hiểu lầm.

**Tách vùng độc lập gần như vô dụng ở mật độ này.** Với k-ring từ 2 trở lên, đồ thị ứng viên gần như thành một khối (1 đến 3 vùng, vùng lớn nhất chứa gần như mọi khách và xe). Chỉ `k-ring 1` tách được 48 vùng, và đó là cấu hình sai nhiều nhất và ghép được ít nhất (2.348/3.000). Ở cỡ nhỏ hơn cũng thế: ở 250 × 1.000 và 400 × 1.600, `k-ring 1` có 68–117 vùng rất nhỏ nhưng sai 12,75–13,20%, còn `k-ring 2` chỉ còn 8–15 vùng, trong đó vùng lớn nhất chứa khoảng 62% số nút. Càng tách được, càng mất chất lượng.

**Lợi ở khâu dựng ETA, không ở khâu giải.** Truy vấn ETA chỉ cho 24.000 cặp mất 195,2 ms thay vì 3,46 s của ma trận đầy đủ (khoảng 18 lần). Khâu giải thì khác. `solve_sparse` dựng một ma trận **đặc** cho từng vùng rồi gọi `hungarian`, nên vùng 2.993 × 2.948 tốn gần như Hungarian đầy đủ, dù chỉ có 24.000 ô hợp lệ. Một bộ giải tận dụng độ thưa có thể khác, nhưng lab chưa thử. Đừng đọc con số 15,0 s như thuộc tính của thuật toán: dòng `H3 r7` có cùng 24.000 cạnh, cùng vùng lớn nhất, cùng 2.634 cặp mà giải trong 3,6 s, và `k-ring 3` với 48.000 cạnh giải trong 3,3 s. Chênh gấp bốn lần trên hai bài toán cùng cỡ cho thấy độ nhiễu của phép đo này.

**Cột `s/cặp` là cái bẫy thiên lệch người sống sót.** Các cấu hình cắt tỉa có `s/cặp` 83–96 s, thấp hơn nhiều mốc tham lam 158,5 s. Đừng đọc đó là «ETA mỗi khách giảm gần một nửa»: cắt tỉa chỉ ghép 2.348–2.765 trong 3.000 khách, và những khách bị bỏ lại phần nhiều là khách khó, nên trung bình trên các khách được ghép tự nhiên đẹp hơn. Cấu hình ghép đủ 3.000 khách (lượt bù) có `s/cặp` 160,7 s, tệ hơn mốc tham lam 158,5 s, và tốn 25,1 s. Không có cấu hình cắt tỉa nào ở đây thắng tham lam toàn cục trên ma trận đầy đủ về mọi mặt.

**Mốc tối ưu thật thì chạy được.** Con số n³ ≈ 2,7 × 10¹⁰ là trường hợp xấu nhất, và bảng trên đã giải Hungarian trên ma trận đặc 3.000 × 3.000 trong vài giây, nên lab có một lần chạy riêng (`--chi-thanh-pho`, một lô khác, 3.000 khách × 3.000 xe). Hungarian trên ma trận đầy đủ cho tổng 427.470 s (142,5 s mỗi cặp), giải trong 4,50 s, cộng 1,04 s dựng ma trận; tham lam toàn cục đắt hơn 13,0% (161,1 s mỗi cặp). Cắt tỉa kèm lượt bù ghép đủ 3.000 cặp với 163,8 s mỗi cặp, tức đắt hơn tối ưu 14,9%, trong 5,54 s. Trên lô này cắt tỉa vừa không nhanh hơn bản đầy đủ, vừa kém cả tham lam. Đây chỉ là một lô, thời gian đo dưới tải (lần chạy trước của chính lô đó ghi 14,06 s cho Hungarian), nên điều giữ lại là tỉ lệ chi phí, không phải số giây.

## Trong simulator và ngân sách độ trễ

Mục B chạy Hungarian với cửa sổ 10 s trong simulator, 20 seed ghép cặp, **không cắt tỉa (A)** so với **cắt tỉa `k-ring 2 + 8 gần nhất` (B)**. Số ô ETA phải tính mỗi lần ghép giảm mạnh:

| Bộ giả định | Ô ETA mỗi lần ghép: không cắt → cắt | p50 / p99 mỗi lần ghép (ms): không cắt → cắt |
|---|---|---|
| `thường` | 108,5 → 13,0 | 1,00 / 106,71 → 0,23 / 51,10 |
| `cao điểm` | 70,8 → 12,6 | 0,71 / 86,51 → 0,26 / 48,40 |
| `thưa xe` | 27,4 → 5,5 | 0,55 / 86,74 → 0,19 / 41,76 |

(Số lần ghép của hai cấu hình khác nhau ở `cao điểm` và `thưa xe` vì quỹ đạo mô phỏng khác nhau; thời gian đo khi máy chịu tải.) Về chất lượng, hiệu B − A:

| Chỉ số | `thường` | `cao điểm` | `thưa xe` |
|---|---|---|---|
| `deadhead_m` trung bình | −15,0 [−44,4; 14,4] không phân biệt được | **−155,2 [−218,5; −91,9] TỐT HƠN** | **−187,7 [−293,5; −81,8] TỐT HƠN** |
| `deadhead_m` max | **+859,6 [17,8; 1.701,5] XẤU HƠN** | −2.810,3 [−4.414,8; −1.205,8] | −3.996,7 [−5.606,3; −2.387,0] |
| `wait_s` max | **+125,4 [5,2; 245,6] XẤU HƠN** | −235,0 [−376,7; −93,3] | −279,1 [−466,8; −91,5] |
| khách hết kiên nhẫn | 0,0000 → 0,0000 | +0,0007 [−0,0055; 0,0069] | +0,0050 [−0,0082; 0,0182] |
| km rỗng tổng / khách đặt (m) | −14,7 [−47,3; 17,8] | −102,8 [−145,3; −60,4] | −101,1 [−163,6; −38,6] |

Ở mật độ thường, chất lượng trung bình không phân biệt được với không cắt tỉa. Hai chỉ số `max` xấu đi (`deadhead_m` +859,6 m, `wait_s` +125,4 s), nhưng cận dưới sát 0 trong khi p95 không phân biệt được (+9,6 m), và mỗi bảng có 17 chỉ số, nên đây là dấu hiệu cần kiểm tra thêm hơn là kết luận. Nếu dấu hiệu đó có thật, một giả thuyết hợp lý là ứng viên chọn theo khoảng cách chim bay gây ra đúng lỗi của [bài 41](/articles/location-gan-nhat-sai-o-dau-do-bang-mo-phong): cả 8 xe gần nhất đều phải đi vòng. Ở cao điểm và thưa xe, cắt tỉa lại **cải thiện** km rỗng 155–188 m, 17 đến 19 trên 20 seed. Điều này nghe có vẻ mâu thuẫn với bất biến «cắt tỉa không bao giờ rẻ hơn tối ưu», nhưng bất biến chỉ nói về **một ma trận**, còn tối ưu theo lô không phải tối ưu cả hệ thống: một lô tối ưu có thể giao một tài xế đi đón khách rất xa, và quyết định đó làm các lô sau xấu đi. Lab chưa kiểm cơ chế; giả thuyết của lab là cắt tỉa vô tình không giao khách cho tài xế rất xa. Đừng coi đây là lý do để cắt tỉa: một lợi ích chưa hiểu cơ chế là lợi ích có thể biến mất khi mô hình đổi. Khi thêm lượt bù, lợi này mất: ở `thưa xe` `deadhead_m` +88,9 m [17,1; 160,8] XẤU HƠN, `wait_s` +7,7 s XẤU HƠN; ở `cao điểm` không phân biệt được.

### Ngân sách độ trễ

Mục C đo thời gian của một lần ghép (dựng ma trận + giải) với ngân sách **giả định** 100 ms ở p99, trên HCM. Máy chịu tải nặng, nên chỉ đọc tỉ lệ giữa hai đường ống trong cùng lần chạy:

| Cỡ đường ống | p50 (ms) | p95 (ms) | p99 (ms) | Trong 100 ms? |
|---|---:|---:|---:|:-:|
| 50 × 50, đầy đủ | 1,58 | 1,95 | 8,98 | có |
| 50 × 50, cắt tỉa | 2,66 | 3,41 | 4,25 | có |
| 200 × 200, đầy đủ | 9,70 | 14,88 | 15,19 | có |
| 200 × 200, cắt tỉa | 8,48 | 10,69 | 10,84 | có |
| 1.000 × 1.000, cắt tỉa | 668,19 | 1.356,22 | 1.361,07 | không |

Mỗi cỡ đo 30 lần, nên «p99» ở đây thực ra là giá trị lớn nhất. Ở 50 × 50 cắt tỉa **chậm hơn ở p50** (2,66 so với 1,58 ms). Lab không tách khâu, nhưng số ở bài tập 1 gợi ý lý do: một truy vấn cặp đắt hơn một ô ma trận bucket cỡ 21 lần, nên 400 cặp không rẻ hơn 2.500 ô. Ở 200 × 200, cắt tỉa chỉ nhanh hơn 1,4 lần ở p99. Ở 1.000 × 1.000 ngay cả cắt tỉa cũng vượt ngân sách giả định (lab không chạy đường ống đầy đủ ở cỡ này trong mục C). Con số 100 ms là giả định của lab; ngân sách thật phải đặt từ SLO của Dispatch.

## Chạy ví dụ và kiểm chứng

Từ gốc repo (cần `data/hcm-graph.bin`; thêm `--fixture` để chạy nhanh):

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc44_cat_tia_h3_ma_tran_thua
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc44_cat_tia_h3_ma_tran_thua -- --only-b --variant 'thưa xe'
```

Lệnh đầu chạy đủ mục A, B (thường, cao điểm) và C; lệnh sau chỉ chạy mục B cho `thưa xe`. Tham số khác: `--seeds N`, `--instances N` (số lô mỗi cỡ cho mục A, mặc định 10), `--jobs N`, `--chi-thanh-pho` (chỉ chạy cỡ 3.000 × 3.000 trên một lô riêng, có Hungarian đầy đủ). Kết quả đầy đủ nằm ở `examples/toi-noi-location/results/p05.md`. Đoạn trích (cỡ 250 × 1.000, nguyên văn):

```text
  cỡ 250 khách × 1000 xe: đầy đủ: ghép TB 250.0/250, tổng ETA TB 20561 s, ETA ma trận trung vị 268.4 ms + Hungarian trung vị 1.5 ms; tham lam toàn cục / Hungarian = 1.015
  H3 r8, k-ring 2, 8 gần nhất         2000     0.80       8.4       779.3     250.0     10/10      +3.70% [+1.73, +5.67]   +10.44%                 73.6       3.7×
  H3 r8, k-ring 3, 16 gần nhất        4000     1.60       1.3      1186.9     250.0     10/10      +1.04% [+0.15, +1.92]    +3.26%                187.7       1.4×
```

`verify-p05.sh` chạy lab này trên fixture với 2 seed và 2 lô để kiểm tra các bất biến (cắt tỉa không rẻ hơn tối ưu đầy đủ, vùng độc lập), không tái lập các số trên.

## Giới hạn của ví dụ

- **Mô phỏng.** Vị trí tài xế và khách được rút ngẫu nhiên từ bộ sinh của simulator (phân bố điểm nóng cộng nền đều), nên mật độ cục bộ phụ thuộc vào đó. Đồ thị thật có sông, kênh khiến k-ring theo khoảng cách chim bay chọn sai ứng viên; sai số đo ở trên đã gồm hiệu ứng đó trên bản đồ HCM, nhưng k-ring theo đường bộ chưa được thử.
- **Ở 3.000 × 3.000 chỉ có một lô cho mỗi lần chạy**, và mốc tối ưu chỉ có ở lô riêng của `--chi-thanh-pho`. Chưa đủ để nói «cắt tỉa cách tối ưu bao xa» ở cỡ này ngoài một điểm dữ liệu.
- **Lô rút có hoàn lại** từ kho 3.000 tài xế và 5.208 khách, nên ở cỡ lớn nhiều xe trùng vị trí. Điều đó làm mật độ cục bộ khác một đội xe thật.
- **Bộ giải theo vùng là Hungarian trên ma trận đặc**; chưa thử bộ giải dành cho ma trận thưa.
- **Chưa thử trường hợp thừa khách** ở mục A (chỉ khách = xe và xe gấp 4 lần khách), chưa thử `thưa xe` ở mục A, và chưa thử `dày xe`, `huỷ nặng` cho mục B.
- **Cơ chế của lợi ích km rỗng khi cắt tỉa trong simulator chưa được kiểm.**
- **Số thời gian đo khi máy chịu tải rất nặng**, kể cả ngân sách 100 ms giả định; chỉ tỉ lệ có nghĩa.
- **10 lô mỗi cỡ** cho mục A, ít lô nên khoảng tin cậy rộng, và ở cấu hình chỉ có 2 lô đủ cặp (k-ring 3, 200 × 200), khoảng tin cậy [−11,32; +28,33] không nói được gì.

## Bài tập

**Bài 1.** Với 3.000 khách, `k_nearest = 8` và `k_nearest = 16`, tính số cạnh tối đa của ma trận thưa và so với 9.000.000 ô đầy đủ. Từ thời gian ETA đã đo (3,46 s cho ma trận đầy đủ, 195,2 ms cho 24.000 cặp), suy ra truy vấn một cặp tốn kém hơn một ô của ma trận đầy đủ bao nhiêu lần.

<details><summary>Lời giải bài 1</summary>
<p>Số cạnh tối đa bằng số khách nhân `k_nearest`: 3.000 × 8 = 24.000 (đúng cột «Cạnh» của bảng) và 3.000 × 16 = 48.000. So với 9.000.000, ít hơn 375 lần và 187,5 lần. Thời gian mỗi ô của ma trận đầy đủ: 3,46 s / 9.000.000 ≈ 0,38 µs; mỗi cặp ứng viên: 195,2 ms / 24.000 ≈ 8,1 µs. Truy vấn một cặp đắt gấp khoảng 21 lần một ô của ma trận bucket, vì bucket CH dùng chung công việc giữa các ô. Thời gian đo khi máy chịu tải, nên tỉ lệ ~21 chỉ là cỡ độ lớn. Đây là lý do cắt tỉa giảm số ô ETA 375 lần mà thời gian ETA chỉ giảm khoảng 18 lần.</p>
</details>

**Bài 2.** Dựng ví dụ nhỏ cho thấy cắt tỉa ghép ít cặp hơn bản đầy đủ khi số khách bằng số xe: hai khách A, B, hai tài xế D1, D2, `k_nearest = 1`. ETA của A: D1 = 10, D2 = 50; của B: D1 = 20, D2 = 40.

<details><summary>Lời giải bài 2</summary>
<p>Bản đầy đủ: A ← D1 (10) và B ← D2 (40) tổng 50; hoặc A ← D2 (50) và B ← D1 (20) tổng 70. Hungarian chọn 50, ghép đủ 2 cặp. Cắt tỉa với `k_nearest = 1`: giả sử chim bay xếp hạng giống ETA, nhìn từ A hay từ B thì xe gần nhất đều là D1. Ma trận thưa chỉ có hai cạnh (A, D1) và (B, D1), cùng chung D1, nên chỉ ghép được 1 cặp (chọn cạnh rẻ hơn: A ← D1, 10). D2 không là ứng viên của ai và khách B bị bỏ. Đó là cơ chế của cột «Khách ghép TB» dưới 200 ở bảng xe bằng khách: nhiều khách chia nhau cùng vài xe ở gần.</p>
</details>

**Bài 3.** Ở `cao điểm`, cắt tỉa cho `deadhead_m` thấp hơn không cắt tỉa 155,2 m, dù bất biến nói cắt tỉa không bao giờ rẻ hơn tối ưu trên một ma trận. Đề xuất một thí nghiệm để phân biệt giả thuyết «cắt tỉa tránh giao khách cho tài xế rất xa» với các giải thích khác.

<details><summary>Lời giải bài 3</summary>
<p>Nếu giả thuyết đúng thì việc **cấm cứng** các cặp có ETA quá một ngưỡng trong Hungarian không cắt tỉa cũng phải cho lợi tương tự. Thí nghiệm: giữ Hungarian đầy đủ nhưng đặt ô ETA vượt ngưỡng (ví dụ 600 s, như `v1-limit` của bài 45) thành `FORBIDDEN`, chạy 20 seed ghép cặp ở `cao điểm` và so với không cắt tỉa. Đồng thời in phân phối ETA của các cặp được ghép trong hai cấu hình (p99, max) và số khách bị bỏ lại. Nếu lợi biến mất khi chỉ cấm cặp xa, giả thuyết sai; nếu lợi tương tự nhưng kèm khách bị bỏ nhiều hơn, thì đó chính là cái giá của bài 45 chứ không phải món quà miễn phí. Mọi thí nghiệm này phải chạy trên seed đánh giá, tham số ngưỡng chọn trước trên seed chỉnh.</p>
</details>

Bài trước: [bài 43](/articles/location-cua-so-lo-do-tre-doi-chat-luong-ghep). Bài tiếp: [bài 45](/articles/location-chi-phi-ghep-cong-bang-tai-xe-rui-ro-huy).

## Tài liệu gốc

- [`h3o::CellIndex` (docs.rs)](https://docs.rs/h3o/0.11.1/h3o/struct.CellIndex.html): kiểu chỉ số ô H3 trong phiên bản h3o 0.11.1; có phương thức `grid_disk` lấy các ô trong khoảng cách k, tương ứng với k-ring.
- [`pathfinding::kuhn_munkres` (docs.rs)](https://docs.rs/pathfinding/4.16.0/pathfinding/kuhn_munkres/fn.kuhn_munkres.html): phép gán tối ưu dùng trong từng vùng, độ phức tạp O(n³).
- [Hungarian algorithm (Wikipedia)](https://en.wikipedia.org/wiki/Hungarian_algorithm): bài toán phân công và nguồn gốc.
- [Student's t-test (Wikipedia)](https://en.wikipedia.org/wiki/Student%27s_t-test): kiểm định mẫu ghép cặp, nền của khoảng tin cậy trong bài.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho dữ liệu OSM.
