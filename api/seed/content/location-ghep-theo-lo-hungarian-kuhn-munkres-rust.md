---
slug: location-ghep-theo-lo-hungarian-kuhn-munkres-rust
title: "Ghép theo lô: bài toán phân công và thuật toán Hungarian"
summary: "Bài 42: ghép theo lô bằng Hungarian trên ma trận ETA. Giảm 911,7 m km rỗng ở cao điểm nhưng khách bị bỏ rơi tăng 0,0149 lên 0,0717; ở mật độ thường lô chỉ 1,6 khách nên không có gì để chọn. Trên dữ liệu mô phỏng."
category: location-routing
level: advanced
tags: [toi-noi, location, rust, dispatch, matching, hungarian, assignment, simulation]
language: vi
---

Giờ cao điểm, hai khách cùng đặt xe trong vài giây ở hai đầu một con đường, và chỉ có hai tài xế rảnh, một người đứng giữa. Nếu hệ thống giao ngay cho khách đặt trước chiếc xe gần nhất, khách đến sau chỉ còn chiếc xe ở xa. [Bài 41](/articles/location-gan-nhat-sai-o-dau-do-bang-mo-phong) đã cho thấy lô này: tham lam tốn 502 s, đổi chéo chỉ 247 s. Ghép theo lô là câu trả lời hiển nhiên: gom các khách đang chờ lại vài giây, dựng ma trận ETA giữa mọi khách và mọi xe rảnh, rồi giải bài toán phân công tối ưu cho cả lô.

Bài này làm cả hai việc: cài đặt phép gán tối ưu bằng crate `pathfinding`, và đo xem nó có đáng không trên simulator. Câu trả lời không gọn. Ở cao điểm km rỗng giảm hàng trăm mét, nhưng tỉ lệ khách bị bỏ rơi tăng gần gấp năm lần. Ở mật độ thường, lô trung bình chỉ 1,64 khách và không chỉ số nào phân biệt được với tham lam, vì gần như không có gì để chọn.

Đây là bài 42 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dựng trên ma trận ETA của [bài 26](/articles/location-ma-tran-many-to-many-bucket-ch) và baseline của bài 41, và dùng đúng cách đọc so sánh có cặp 20 seed đã giải thích ở bài 41: mọi hiệu là «B trừ A, cùng seed», khoảng tin cậy 95% trên hiệu từng seed, «TỐT HƠN» và «XẤU HƠN» chỉ ghi khi khoảng không chứa 0. Mọi số đo là **trên dữ liệu mô phỏng**, đồ thị HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors), seed đánh giá 20261106…20261125.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Gom khách vào lô 10 s rồi giải tối ưu bằng Hungarian thay vì tham lam theo thứ tự. Ở mật độ cao điểm (7 khách/phút), bạn đoán km rỗng giảm, và có chỉ số nào xấu đi không? Ở mật độ thường (3,5 khách/phút) thì sao?</p>
<details><summary>Xem đáp án</summary><p>Ở cao điểm, <code>deadhead_m</code> trung bình giảm 911,7 m, khoảng [−1.143,2; −680,3], 20/20 seed. Nhưng tỉ lệ khách hết kiên nhẫn (chưa có xe sau 300 s) tăng từ 0,0149 lên 0,0717. Ở mật độ thường, hai cách <b>không phân biệt được</b> ở mọi chỉ số: lô trung bình chỉ 1,64 khách, nên gần như không có gì để tối ưu.</p></details>
</div>

## Mô hình hoá: ma trận chi phí và nhiều hơn một cách sai

Quy ước của crate `matcher`: **hàng là khách, cột là tài xế, ô là ETA đường bộ tính bằng mili giây** (số nguyên, lấy từ `bucket_matrix` trên CH ở bài 26). Ô bị cấm hoặc không tới được mang giá trị `FORBIDDEN`. Mọi hàm phân công trả danh sách cặp `(khách, tài xế)`, mỗi khách và mỗi tài xế tối đa một lần, và không bao giờ chứa cặp bị cấm.

Phép gán tối ưu cần một thứ mà tham lam không có: nhìn cả lô. Ba cách giải trong crate:

- `greedy_fifo`: khách theo thứ tự đặt, mỗi khách lấy xe rẻ nhất còn trống (chính là `nearest-eta` của bài 41).
- `greedy_global`: lặp lại việc lấy cặp rẻ nhất trong mọi cặp còn lại. Mạnh hơn, vẫn không tối ưu.
- `hungarian`: tối đa số cặp được phép trước, rồi tối thiểu tổng chi phí.

Crate `pathfinding` có `kuhn_munkres`, nhưng hàm đó **tối đa hoá** tổng trọng số và đòi số hàng không vượt số cột. Ma trận ETA là chi phí cần tối thiểu, thường chữ nhật theo cả hai chiều, và có ô cấm. Ba điều chỉnh trong `hungarian`: chuyển ma trận khi thừa khách; đổi chi phí `v` thành trọng số `big − (v − lo)`; ô cấm nhận trọng số 0 và bị loại khỏi kết quả. Hằng `big` đủ lớn để **thêm một cặp luôn thắng mọi khoản tiết kiệm chi phí**, nhờ đó thứ tự ưu tiên là «ghép được nhiều khách nhất» rồi mới «rẻ nhất».

`examples/toi-noi-location/rust/crates/matcher/src/assign.rs`

```rust
/// Hungarian: tối đa số cặp được phép trước, rồi tối thiểu tổng chi phí. Ma trận chữ nhật bất kỳ (thừa khách hoặc thừa xe).
///
/// `pathfinding::kuhn_munkres` *tối đa hoá* tổng trọng số và đòi số hàng ≤ số cột, nên: chuyển ma trận nếu thừa khách; đổi
/// chi phí thành trọng số `M − chi phí` với `M` lớn hơn mọi tổng chi phí (một cặp thêm luôn thắng mọi tiết kiệm chi phí), ô bị cấm
/// nhận trọng số 0 và bị loại khỏi kết quả. Độ phức tạp O(r²·c) cho r ≤ c (r = min(hàng, cột)).
pub fn hungarian(costs: &Dense) -> Vec<(usize, usize)> {
    if costs.rows == 0 || costs.cols == 0 {
        return Vec::new();
    }
    let transposed = costs.rows > costs.cols;
    let m = if transposed {
        costs.transpose()
    } else {
        costs.clone()
    };
    let (lo, hi) = m
        .data
        .iter()
        .filter(|&&v| allowed(v))
        .fold((i64::MAX, i64::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    if lo > hi {
        return Vec::new(); // không cặp nào được phép
    }
    // M > r · (khoảng dao động chi phí) để thêm một cặp luôn có lợi hơn mọi tiết kiệm chi phí
    let span = hi - lo + 1;
    let big = (m.rows as i64 + 1) * span + 1;
    let weights: Vec<i64> = m
        .data
        .iter()
        .map(|&v| if allowed(v) { big - (v - lo) } else { 0 })
        .collect();
    let w = Matrix::from_vec(m.rows, m.cols, weights).expect("kích thước khớp");
    let (_, cols_of_row) = kuhn_munkres(&w);
    let mut out: Vec<(usize, usize)> = cols_of_row
        .iter()
        .enumerate()
        .filter(|&(r, &c)| allowed(m.get(r, c)))
        .map(|(r, &c)| if transposed { (c, r) } else { (r, c) })
        .collect();
    out.sort_unstable();
    out
}
```

Bài toán phân công này có lời giải tối ưu trong thời gian đa thức. Kuhn công bố phương pháp ở Naval Research Logistics Quarterly, tập 2, trang 83–97, năm 1955, dưới tên «Hungarian method»; Munkres trình bày lại thuật toán ở Journal of the Society for Industrial and Applied Mathematics, tập 5, trang 32–38, năm 1957. Tên «Kuhn–Munkres» của crate là từ hai bài báo đó. Tài liệu của `pathfinding` nêu độ phức tạp O(n³).

Một ví dụ nhỏ trong test cho thấy vì sao cần nhìn cả lô: khách 0 đến trước và gần tài xế 0 (10 so với 12); khách 1 chỉ gần tài xế 0 (11 so với 100).

`examples/toi-noi-location/rust/crates/matcher/src/assign.rs`

```rust
    #[test]
    fn greedy_steals_the_driver_hungarian_does_not() {
        // khách 0 đến trước và «gần» tài xế 0 (10 so với 12); khách 1 chỉ có tài xế 0 là gần (11 so với 100)
        let c = m(&[&[10, 12], &[11, 100]]);
        let g = greedy_fifo(&c);
        let h = hungarian(&c);
        assert_eq!(g, vec![(0, 0), (1, 1)]);
        assert_eq!(total_cost(&c, &g), 110);
        assert_eq!(h, vec![(0, 1), (1, 0)]);
        assert_eq!(total_cost(&c, &h), 23);
    }
```

Cách kiểm đúng: trên 300 ma trận ngẫu nhiên 1–6 × 1–6 có 10% ô bị cấm, `hungarian` và vét cạn (`brute_force`) cho cùng số cặp và cùng tổng chi phí, **0 khác biệt**. Thêm các test proptest trong `tests/assignment_props.rs`: lời giải hợp lệ, chi phí Hungarian không bao giờ lớn hơn tham lam trên cùng ma trận, khớp vét cạn.

## Thừa khách, thừa xe: Hungarian rẻ hơn bao nhiêu trên cùng ma trận

Mục 1 của lab dựng lô ngẫu nhiên từ thế giới mô phỏng 300 tài xế, 200 lô mỗi cỡ (seed 42001), tính ETA đường bộ thật bằng CH, rồi so tổng ETA của các cặp được ghép. «Tham lam TT» xếp khách theo thứ tự ngẫu nhiên (đại diện cho thứ tự đặt); «tham lam TC» là toàn cục.

| Khách × xe | Cặp ghép | Tham lam TT (s) | Tham lam TC (s) | Hungarian (s) | Hungarian − TT | Hungarian − TC | ETA max TT → H (s) |
|---|---:|---:|---:|---:|---|---|---|
| 5 × 5 | 5 | 3.755,3 | 3.560,8 | 3.398,8 | −356,5 (−9,5%) | −162,1 (−4,6%) | 1.289,7 → 1.180,4 |
| 10 × 10 | 10 | 6.598,9 | 6.175,6 | 5.741,2 | −857,7 (−13,0%) | −434,4 (−7,0%) | 1.419,5 → 1.225,8 |
| 20 × 20 | 20 | 11.277,6 | 10.490,2 | 9.616,0 | −1.661,6 (−14,7%) | −874,2 (−8,3%) | 1.540,4 → 1.230,1 |
| 30 × 50 | 30 | 9.066,5 | 8.779,1 | 8.475,7 | −590,8 (−6,5%) | −303,4 (−3,5%) | 810,3 → 778,4 |
| 50 × 30 | 30 | 15.569,1 | 8.858,8 | 8.541,8 | −7.027,3 (−45,1%) | −317,0 (−3,6%) | 1.592,8 → 829,9 |
| 10 × 60 | 10 | 2.415,4 | 2.399,0 | 2.383,0 | −32,4 (−1,3%) | −16,0 (−0,7%) | 540,4 → 539,5 |
| 60 × 10 | 10 | 6.610,4 | 2.314,5 | 2.290,2 | −4.320,2 (−65,4%) | −24,3 (−1,0%) | 1.436,9 → 520,7 |

Mọi khoảng tin cậy 95% đều không chứa 0. Ba cách đọc.

**Khi lô cân bằng, lợi 9,5–14,7% so với tham lam theo thứ tự.** Lô càng lớn, lợi càng nhiều (5 × 5: 9,5%; 20 × 20: 14,7%), vì càng nhiều khả năng có «khách cướp xe của khách sau». So với tham lam toàn cục, lợi chỉ 4,6–8,3%: nếu chọn cách giải rẻ hơn để đỡ tốn CPU, tham lam toàn cục đã lấy khoảng một nửa phần lợi (47–55% tuỳ cỡ, tính từ bảng).

**Khi thừa khách, con số 45,1% và 65,4% không nên đọc như «tối ưu hơn một nửa».** Ở 50 × 30, cả hai cách ghép đúng 30 khách. Khác biệt là **ai** được phục vụ: tham lam theo thứ tự phục vụ 30 khách đầu danh sách, còn Hungarian, vì tối thiểu tổng ETA, tự chọn 30 khách rẻ nhất trong 50. Phần lớn khoản «tiết kiệm» là do chọn lại tập khách được phục vụ. Đó là điều thuật toán làm theo đúng định nghĩa của nó, và là mầm của kết quả ở mục «Cái giá» bên dưới. Còn so với tham lam toàn cục, lợi chỉ 3,6%.

**ETA lớn nhất trong lô cũng thấp hơn** ở cả bảy cỡ lô. Lab đã ghi chú trước rằng cái giá của việc tối ưu tổng có thể là một cặp đơn lẻ xa hơn; trong các lô ngẫu nhiên này điều đó không xảy ra. Đừng suy ra rằng nó không bao giờ xảy ra: đây là lô ngẫu nhiên, không phải lô của dispatch.

## Độ phức tạp: lý thuyết O(n³), số đo nhiễu

Mục 2 đo Hungarian trên ma trận vuông ngẫu nhiên n × n (số nguyên), trung vị của 5 lần chạy, so với tham lam theo thứ tự. **Máy đo chịu tải rất nặng (load average từ khoảng 350 trên 10 lõi), nên số thời gian nhiễu và phóng đại; chỉ đọc tỉ lệ.**

| n | Hungarian (ms) | Tham lam (ms) | H(n) / H(n/2) |
|---:|---:|---:|---:|
| 25 | 0,014 | 0,001 | — |
| 50 | 0,133 | 0,009 | 9,8 |
| 100 | 0,986 | 0,018 | 7,4 |
| 200 | 1,660 | 0,072 | 1,7 |
| 400 | 6,300 | 0,287 | 3,8 |
| 800 | 59,417 | 1,149 | 9,4 |

Độ dốc log–log là 2,23, thấp hơn 3 của trường hợp xấu nhất; ma trận ngẫu nhiên thường dễ hơn trường hợp xấu nhất. Cột cuối nhảy từ 9,8 xuống 1,7 rồi lên 9,4: nhiều khả năng là nhiễu từ tải máy, nên đừng ngoại suy một con số chắc chắn từ bảng này (bài tập 3). Điều bảng cho thấy: Hungarian chậm hơn tham lam từ 14 đến 55 lần trên cùng ma trận, không đều theo n, và phần lớn sự không đều đó cũng là nhiễu.

Trong simulator, kích thước thực của một lô nhỏ hơn nhiều so với bảng trên (xem mục kế tiếp): thời gian giải p50 chỉ 3,0 µs và p99 8,2 µs ở mật độ thường (đo khi máy chịu tải nặng). Chi phí chính của một lần ghép không nằm ở thuật toán mà ở việc dựng ma trận ETA, và đó là chuyện của [bài 44](/articles/location-cat-tia-ung-vien-h3-ma-tran-thua).

## Trong simulator: khi nào lô có gì để chọn

Mục 3 chạy simulator với cửa sổ lô cố định 10 s: khi có khách chờ và có tài xế rảnh mà chưa có lô nào đang mở, simulator mở lô, và đúng 10 s sau gọi chiến lược ghép **một lần** cho mọi khách đang chờ. Hai chiến lược so sánh dùng **cùng cửa sổ, cùng lô**, chỉ khác thuật toán trong lô: A tham lam theo thứ tự, B Hungarian. Kích thước lô thực đo trong 20 seed:

| Bộ giả định | Lần ghép | Khách mỗi lô | Xe rảnh mỗi lô | Lô lớn nhất |
|---|---:|---:|---:|---:|
| `thường`: 120 xe, 3,5 khách/phút | 2.781 | 1,64 | 67,78 | 6 khách |
| `thưa xe`: 60 xe, 3,5 khách/phút | 2.697 | 3,87 | 15,86 | 32 khách |
| `cao điểm`: 120 xe, 7,0 khách/phút | 3.934 | 6,71 | 30,90 | 47 khách |

Ở `thường`, một lô trung bình chưa tới hai khách. Với một khách, bài toán phân công suy biến thành «chọn phần tử nhỏ nhất trong một hàng», và Hungarian lẫn tham lam chọn đúng cùng một xe. Hai khách chỉ cho kết quả khác nhau khi họ tranh cùng một xe, mà với 67,78 xe rảnh điều đó hiếm. Kết quả đo đúng như vậy: hai cách cho `deadhead_m` trung bình khác nhau +10,1 m, khoảng [−18,6; 38,9] chứa 0, và chỉ 2/20 seed cho đúng cùng giá trị: ở 18 seed còn lại có ít nhất một lô mà hai cách chọn khác nhau, và sai khác nhỏ đó lan sang các lô sau.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 306" role="img" aria-label="Hình 1. Hiệu deadhead_m trung bình giữa Hungarian và tham lam theo thứ tự, cùng cửa sổ lô 10 giây, 20 seed. Bộ thường, lô trung bình 1,64 khách: cộng 10,1 mét, khoảng từ trừ 18,6 đến cộng 38,9, không phân biệt được. Bộ thưa xe, lô 3,87 khách: trừ 974,8 mét. Bộ cao điểm, lô 6,71 khách: trừ 911,7 mét."><rect x="20" y="20" width="840" height="281" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Hungarian trừ tham lam, cùng lô 10 s: hiệu deadhead_m trung bình theo cỡ lô</text><path d="M417.9 86 L417.9 232" class="f-none s-line" stroke-width="1"/><text x="417.9" y="248" class="t-small" text-anchor="middle">−1.200</text><path d="M471.4 86 L471.4 232" class="f-none s-line" stroke-width="1"/><text x="471.4" y="248" class="t-small" text-anchor="middle">−900</text><path d="M525.0 86 L525.0 232" class="f-none s-line" stroke-width="1"/><text x="525.0" y="248" class="t-small" text-anchor="middle">−600</text><path d="M578.6 86 L578.6 232" class="f-none s-line" stroke-width="1"/><text x="578.6" y="248" class="t-small" text-anchor="middle">−300</text><path d="M632.1 86 L632.1 232" class="f-none s-line" stroke-width="1"/><text x="632.1" y="248" class="t-small" text-anchor="middle">0</text><path d="M632.1 86 L632.1 232" class="f-none s-muted dash" stroke-width="1.5"/><text x="400" y="72" class="t-small">← ít km rỗng hơn</text><text x="650" y="72" class="t-small" text-anchor="end">nhiều hơn →</text><text x="670" y="72" class="t-small">hiệu [KTC 95%]</text><text x="40" y="120.0" class="t-b">deadhead_m trung bình (mét)</text><text x="52" y="152.0">«thường»: lô 1,64 khách × 67,78 xe</text><path d="M628.8 148.0 L639.1 148.0" class="f-none s-dead" stroke-width="2.5"/><path d="M628.8 143.0 L628.8 153.0 M639.1 143.0 L639.1 153.0" class="f-none s-dead" stroke-width="2"/><circle cx="633.9" cy="148.0" r="5" class="f-dead s-dead" stroke-width="1"/><text x="670" y="152.0" class="t-mono">+10,1 [−18,6; 38,9]</text><text x="52" y="184.0">«thưa xe»: lô 3,87 khách × 15,86 xe</text><path d="M409.1 180.0 L507.0 180.0" class="f-none s-phys" stroke-width="2.5"/><path d="M409.1 175.0 L409.1 185.0 M507.0 175.0 L507.0 185.0" class="f-none s-phys" stroke-width="2"/><circle cx="458.1" cy="180.0" r="5" class="f-phys s-phys" stroke-width="1"/><text x="670" y="184.0" class="t-mono">−974,8 [−1.248,8; −700,8]</text><text x="52" y="216.0">«cao điểm»: lô 6,71 khách × 30,90 xe</text><path d="M428.0 212.0 L510.7 212.0" class="f-none s-phys" stroke-width="2.5"/><path d="M428.0 207.0 L428.0 217.0 M510.7 207.0 L510.7 217.0" class="f-none s-phys" stroke-width="2"/><circle cx="469.3" cy="212.0" r="5" class="f-phys s-phys" stroke-width="1"/><text x="670" y="216.0" class="t-mono">−911,7 [−1.143,2; −680,3]</text><circle cx="52" cy="266" r="5" class="f-phys s-phys" stroke-width="1"/><text x="64" y="270" class="t-small">TỐT HƠN (KTC không chứa 0)</text><circle cx="262" cy="266" r="5" class="f-warn s-warn" stroke-width="1"/><text x="274" y="270" class="t-small">XẤU HƠN (KTC không chứa 0)</text><circle cx="480" cy="266" r="5" class="f-dead s-dead" stroke-width="1"/><text x="492" y="270" class="t-small">không phân biệt được</text><text x="40" y="286" class="t-small">Seed 20261106…25, cửa sổ lô 10 s. Kích thước lô là trung bình thực đo trong simulator. Trên dữ liệu mô phỏng.</text></svg>
</div>
<figcaption>Hình 1. Hiệu km rỗng giữa Hungarian và tham lam, cùng cửa sổ lô 10 s, theo kích thước lô thực. Lợi xuất hiện ở hai bộ có lô 3,87 và 6,71 khách. Ở mật độ thường, lô 1,64 khách: không phân biệt được. Trên dữ liệu mô phỏng, 20 seed.</figcaption>
</figure>

| Chỉ số (B = Hungarian, A = tham lam, lô 10 s) | `thường` | `thưa xe` | `cao điểm` |
|---|---|---|---|
| `deadhead_m` trung bình | +10,1 [−18,6; 38,9] | −974,8 [−1.248,8; −700,8] | −911,7 [−1.143,2; −680,3] |
| `pickup_eta_s` trung bình | +1,8 [−1,0; 4,7] | −90,1 [−116,0; −64,3] | −87,1 [−108,5; −65,7] |
| `wait_s` p95 | +4,7 [−11,6; 21,0] | −271,9 [−382,5; −161,3] | −315,1 [−403,2; −227,0] |
| km rỗng tổng / khách đặt (m) | +13,3 [−13,2; 39,8] | −636,3 [−819,0; −453,6] | −632,1 [−801,6; −462,5] |
| chuyến hoàn thành | +0,5 [−0,2; 1,1] | +0,1 [−1,4; 1,6] | +1,1 [−3,3; 5,5] |
| huỷ (`cancel_rate`) | −0,0019 [−0,0049; 0,0011] | −0,0005 [−0,0072; 0,0063] | −0,0023 [−0,0119; 0,0073] |
| **khách hết kiên nhẫn** | 0,0000 → 0,0000 | **0,0341 → 0,0796 XẤU HƠN** | **0,0149 → 0,0717 XẤU HƠN** |

Số khách ở một lô cho biết trước kết quả. Ở `thưa xe` và `cao điểm` (lô 3,9 và 6,7 khách) Hungarian giảm km rỗng khoảng 900–975 m, ETA đón khoảng 87–90 s, `wait_s` p95 hơn 270 s, ở 16 đến 20 trên 20 seed tuỳ chỉ số. Ở `thường`, không chỉ số nào phân biệt được với 0, và đây không phải một dấu sao đơn lẻ mà là cả một khối chỉ số cùng hướng. Chỉ số ít bị thiên lệch nhất là km rỗng tổng trên mỗi khách đặt, vì mẫu số là mọi khách: −632,1 m ở `cao điểm`, nhỏ hơn con số 911,7 m tính trên chuyến hoàn thành.

## Cái giá: số khách bị bỏ rơi tăng gần gấp năm lần

Dòng in đậm cuối bảng là dòng không được bỏ khi kể câu chuyện về 911,7 m. Ở `cao điểm`, tỉ lệ khách chờ tới hết kiên nhẫn 300 s mà chưa có xe tăng từ 0,0149 lên 0,0717: hiệu +0,0568, khoảng [0,0411; 0,0724], 19/20 seed theo chiều xấu. Ở `thưa xe`: 0,0341 → 0,0796, +0,0455 [0,0310; 0,0599], 17/20 seed.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Lợi và cái giá của Hungarian so với tham lam trong lô 10 giây. Bộ cao điểm: deadhead_m trung bình từ 4.829,6 xuống 3.917,9 mét, nhưng tỉ lệ khách hết kiên nhẫn từ 0,0149 lên 0,0717. Bộ thưa xe: deadhead_m từ 5.457,0 xuống 4.482,2 mét, tỉ lệ hết kiên nhẫn từ 0,0341 lên 0,0796."><rect x="20" y="20" width="840" height="305" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Km rỗng giảm, khách bị bỏ rơi tăng: cùng một thay đổi, cùng 20 seed</text><text x="40" y="76" class="t-b">«cao điểm»: 120 xe, 7,0 khách/phút</text><text x="40" y="96" class="t-b">deadhead_m trung bình (m)</text><text x="40" y="120" class="t-small">tham lam</text><rect x="150" y="108" width="144.9" height="16" rx="2" class="f-dead"/><text x="300.9" y="121" class="t-mono">4.829,6</text><text x="40" y="146" class="t-small">Hungarian</text><rect x="150" y="134" width="117.5" height="16" rx="2" class="f-phys"/><text x="273.5" y="147" class="t-mono">3.917,9</text><text x="450" y="96" class="t-b">khách hết kiên nhẫn (tỉ lệ)</text><text x="450" y="120" class="t-small">tham lam</text><rect x="560" y="108" width="28.5" height="16" rx="2" class="f-dead"/><text x="594.5" y="121" class="t-mono">0,0149</text><text x="450" y="146" class="t-small">Hungarian</text><rect x="560" y="134" width="136.9" height="16" rx="2" class="f-warn"/><text x="702.9" y="147" class="t-mono">0,0717</text><text x="40" y="196" class="t-b">«thưa xe»: 60 xe, 3,5 khách/phút</text><text x="40" y="216" class="t-b">deadhead_m trung bình (m)</text><text x="40" y="240" class="t-small">tham lam</text><rect x="150" y="228" width="163.7" height="16" rx="2" class="f-dead"/><text x="319.7" y="241" class="t-mono">5.457,0</text><text x="40" y="266" class="t-small">Hungarian</text><rect x="150" y="254" width="134.5" height="16" rx="2" class="f-phys"/><text x="290.5" y="267" class="t-mono">4.482,2</text><text x="450" y="216" class="t-b">khách hết kiên nhẫn (tỉ lệ)</text><text x="450" y="240" class="t-small">tham lam</text><rect x="560" y="228" width="65.1" height="16" rx="2" class="f-dead"/><text x="631.1" y="241" class="t-mono">0,0341</text><text x="450" y="266" class="t-small">Hungarian</text><rect x="560" y="254" width="152.0" height="16" rx="2" class="f-warn"/><text x="718.0" y="267" class="t-mono">0,0796</text><text x="40" y="312" class="t-small">Seed 20261106…25, lô 10 s. Hiệu có cặp: unserved +0,0568 [0,0411; 0,0724] ở cao điểm, +0,0455 [0,0310; 0,0599] ở thưa xe. Trên dữ liệu mô phỏng.</text></svg>
</div>
<figcaption>Hình 2. Cùng một thay đổi, hai kết quả ngược chiều. Km rỗng của chuyến hoàn thành giảm (xanh ngọc), nhưng tỉ lệ khách hết kiên nhẫn tăng gấp 4,8 lần ở cao điểm và 2,3 lần ở thưa xe (vàng cam). Hai bộ dùng chung một thang. Trên dữ liệu mô phỏng, 20 seed 20261106…25, lô 10 s.</figcaption>
</figure>

**Vì sao.** Hungarian tối thiểu tổng ETA của lô và không có số hạng nào nói «khách này đã chờ lâu». Khi thừa khách, nó chọn những khách rẻ nhất để phục vụ và để lại khách xa xe nhất; lô sau, khi một xe rảnh trở lại, nó lại chọn khách rẻ nhất trong danh sách chờ. Tham lam theo thứ tự thì phục vụ khách đặt sớm nhất trước.

Lab kiểm trực tiếp giải thích này bằng cờ `--bo-roi`: bọc chiến lược để đếm, với mỗi khách hết kiên nhẫn, có lần nào một khách đặt **sau** họ được gán xe trong lô họ đang chờ hay không.

| Trên dữ liệu mô phỏng, lô 10 s, tổng 20 seed | Khách hết kiên nhẫn | Từng bị khách đặt sau vượt | Lần bị bỏ qua mỗi khách |
|---|---:|---:|---:|
| `cao điểm`, tham lam theo thứ tự | 140 | 0 (0,0%) | 15,2 |
| `cao điểm`, Hungarian | 659 | 659 (100,0%) | 14,2 |
| `thưa xe`, tham lam theo thứ tự | 161 | 0 (0,0%) | 9,4 |
| `thưa xe`, Hungarian | 372 | 372 (100,0%) | 8,6 |

Dưới tham lam, khách hết kiên nhẫn đơn giản là vì thiếu xe, không ai chen trước họ. Dưới Hungarian, không có khách hết kiên nhẫn nào mà chưa từng bị khách đến sau vượt. Đối chiếu thêm cùng bộ `cao điểm`, cùng seed: Hungarian ghép tức thì, không cửa sổ, đã có tỉ lệ hết kiên nhẫn 0,0660 ([bài 43](/articles/location-cua-so-lo-do-tre-doi-chat-luong-ghep)), còn tham lam ghép ngay chỉ 0,0126 ([bài 41](/articles/location-gan-nhat-sai-o-dau-do-bang-mo-phong)). Vậy nguồn gốc là quyền chọn khách của Hungarian, không phải cửa sổ lô. Đây vẫn là đo trong mô phỏng, nhưng không còn là phỏng đoán từ công thức.

**Phép trừ cho thấy câu chuyện không đơn giản là «mất thêm khách».** `cancel_rate` của simulator đếm mọi chuyến không hoàn thành (huỷ sau khi gán lẫn hết kiên nhẫn). Ở `cao điểm` nó gần như không đổi (0,3047 → 0,3024, −0,0023, khoảng chứa 0), số chuyến hoàn thành cũng không đổi (+1,1, khoảng chứa 0). Vậy khoản +0,0568 hết kiên nhẫn phải được bù bằng khoảng −0,0591 huỷ sau khi gán (phép trừ trên các giá trị trung bình, chưa phải bảng riêng của lab). Nói cách khác, Hungarian không bỏ rơi thêm khách trên tổng thể: nó chuyển một số khách từ «được giao một tài xế quá xa rồi huỷ» sang «không được giao, chờ cho hết kiên nhẫn». Cả hai đều không đi được xe. Điều khác nhau:

- **Khách ở xa:** trước đây nhận được lời chào xe sau vài giây rồi tự huỷ khi thấy ETA dài; nay phải chờ trọn 300 s mới biết không có xe. Lab chưa đo khách nào tệ hơn trong hai kịch bản, và mô hình huỷ là giả định chưa hiệu chuẩn.
- **Tài xế:** không còn chạy vô ích tới khách sẽ huỷ. Đó là một nguồn của khoản giảm km rỗng tổng trên mỗi khách đặt (−632,1 m): tài xế được lợi.
- **Khách còn lại:** ETA đón trung bình −87,1 s, nhưng con số này tính trên chuyến hoàn thành, trong khi những khách khó nhất đã rời khỏi mẫu. Một phần khoản lợi 87,1 s là thiên lệch người sống sót, chưa biết bao nhiêu.

Theo vùng, ở `cao điểm` cả ba vùng đều xấu đi: trung tâm 0,0172 → 0,0685, vành trong 0,0139 → 0,0644, ngoại vi 0,0147 → 0,0867; ngoại vi tăng nhiều nhất (+0,0720 [0,0545; 0,0896]) nhưng khoảng chồng lên hai vùng kia. «Xa» ở đây là xa xe rảnh lúc đó, không phải xa trung tâm thành phố. [Bài 45](/articles/location-chi-phi-ghep-cong-bang-tai-xe-rui-ro-huy) thử thêm thưởng chờ lâu và giới hạn đón xa vào hàm chi phí, kể cả ở đúng cấu hình này (lô 10 s, `cao điểm`): thưởng chờ lâu 0,5 chỉ giảm tỉ lệ hết kiên nhẫn 0,0025, còn giới hạn đón xa đẩy nó lên 0,1973. Chưa có bản sửa nào trong lab.

## Chạy ví dụ và kiểm chứng

Từ gốc repo (cần `data/hcm-graph.bin`; thêm `--fixture` để chạy nhanh trên đồ thị tổng hợp):

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc42_ghep_theo_lo_hungarian
```

Tham số: `--seeds N` (20), `--batches N` (200), `--jobs N`, `--bo-roi` (chỉ chạy phép đo «ai bị bỏ rơi» ở `cao điểm` và `thưa xe`). Kết quả đầy đủ nằm ở `examples/toi-noi-location/results/p05.md`. Trích (đo khi máy chịu tải nặng; số đếm và tỉ lệ tái lập được, thời gian thì không):

```text
  đối chiếu vét cạn (ma trận ngẫu nhiên 1–6 × 1–6, 10% ô bị cấm, 300 ma trận): số lần Hungarian khác vét cạn = 0
  độ dốc log–log của thời gian theo n (ma trận vuông): 2.23 (lý thuyết O(n³) → 3; ma trận ngẫu nhiên thường thấp hơn trường hợp xấu nhất)

=== bộ giả định «cao điểm»: 120 xe, 7,0 chuyến/phút (cầu ×2) ===
  kích thước lô thực tế (Hungarian): 3934 lần ghép trên 20 seed; trung bình 6.71 khách × 30.90 xe mỗi lô, lô lớn nhất 47 khách; thời gian giải p50 3.7 µs, p99 10.2 µs (đo khi máy chịu tải)
  deadhead_m trung bình                4829.6     3917.9     -911.7  [-1143.2, -680.3]            4868.3     3884.6  20/0    TỐT HƠN
  unserved_rate (hết kiên nhẫn)        0.0149     0.0717     0.0568  [0.0411, 0.0724]             0.0023     0.0640   0/19   XẤU HƠN
```

`verify-p05.sh` chạy lab này trên fixture với 2 seed để kiểm bất biến, không tái lập các số trên.

## Giới hạn của ví dụ

- **Mô phỏng.** Mô hình huỷ là công thức chưa hiệu chuẩn; kiên nhẫn 300 s là giả định; khách ở đây không có «ưu tiên» nào ngoài thứ tự đặt.
- **Chỉ ba bộ giả định** (`thường`, `cao điểm`, `thưa xe`); `dày xe`, `huỷ nặng` và `huỷ theo chờ` chưa chạy cho bài này.
- **Phép đo «bị vượt» là đếm trong cùng một lần chạy**, không phải thí nghiệm phản thực: nó không nói nếu không bị vượt thì khách đó có được phục vụ hay không.
- **Thời gian giải** (µs, ms) đo khi máy chịu tải nặng, nên không có giá trị tuyệt đối; độ dốc 2,23 cũng nhiễu.
- **ETA ma trận theo nút, không có chi phí rẽ**; Hungarian tối ưu trên một ước lượng, simulator đo thế giới có chi phí rẽ và nhiễu tốc độ.
- **Hungarian ở cỡ thành phố** (hàng nghìn khách) chưa chạy ở bài này; bài 44 đo cỡ đó.

## Bài tập

**Bài 1.** Với ma trận `[[10, 12], [11, 100]]` (khách là hàng), tính `lo`, `hi`, `big` và các trọng số mà `hungarian` đưa vào `kuhn_munkres`. Phép gán nào có tổng trọng số lớn nhất, và tổng chi phí tương ứng là bao nhiêu?

<details><summary>Lời giải bài 1</summary>
<p>Các ô đều được phép: `lo = 10`, `hi = 100`, `span = 91`, `big = (2 + 1) × 91 + 1 = 274`. Trọng số `big − (v − lo)` lần lượt là 274, 272 cho hàng 0 và 273, 184 cho hàng 1. Đường chéo chính (ô (0,0) và (1,1)) có tổng 274 + 184 = 458; đường chéo phụ (ô (0,1) và (1,0)) có tổng 272 + 273 = 545 lớn hơn. Vậy `kuhn_munkres` chọn chéo phụ: khách 0 ← tài xế 1 (12), khách 1 ← tài xế 0 (11), tổng chi phí 23, đúng như test `greedy_steals_the_driver_hungarian_does_not`. Tham lam chọn đường chéo chính: 10 + 100 = 110.</p>
</details>

**Bài 2.** Ba khách, hai xe, hàng theo thứ tự đặt: `[[100, 110], [1, 50], [2, 60]]`. Tham lam theo thứ tự ghép những ai, tổng bao nhiêu? Hungarian ghép những ai, tổng bao nhiêu? Khách nào bị bỏ lại ở mỗi cách, và điều đó liên hệ gì tới tỉ lệ khách hết kiên nhẫn ở cao điểm?

<details><summary>Lời giải bài 2</summary>
<p>Tham lam theo thứ tự: khách 0 lấy cột 0 (100), khách 1 lấy cột 1 (50), khách 2 không còn xe: tổng 150, khách 2 bị bỏ lại. Hungarian chọn hai trong ba khách và hai cột khác nhau sao cho tổng nhỏ nhất: khách 1 ← cột 1 (50) và khách 2 ← cột 0 (2) cho tổng 52; mọi phương án có khách 0 đều ≥ 111. Vậy Hungarian bỏ lại **khách 0**, khách đặt sớm nhất nhưng ở xa mọi xe. Tổng chi phí 52 so với 150 là «lợi rất lớn», và cái giá chính là khách 0. Nếu lô sau cũng có khách rẻ hơn khách 0 thì khách 0 lại bị hoãn tiếp: đó là cơ chế mà phép đo `--bo-roi` xác nhận trong simulator.</p>
</details>

**Bài 3.** Từ bảng độ phức tạp, ước lượng thời gian Hungarian ở n = 3.000 bằng hai cách: độ dốc đo được 2,23 và độ dốc lý thuyết 3, lấy n = 800 (59,417 ms) làm mốc. Vì sao hai ước lượng khác nhau tới mức đó, và vì sao không nên tin cả hai?

<details><summary>Lời giải bài 3</summary>
<p>Hệ số n tăng là 3.000 / 800 = 3,75. Với độ dốc 2,23: 3,75^2,23 ≈ 19,1, thời gian cỡ 1,1 giây. Với độ dốc 3: 3,75³ ≈ 52,7, thời gian cỡ 3,1 giây. Hai ước lượng chênh gần ba lần vì số mũ tác động mạnh khi nhân qua hệ số lớn. Không nên tin cả hai: các tỉ lệ H(n)/H(n/2) trong bảng (9,8; 7,4; 1,7; 3,8; 9,4) không đơn điệu, nên độ dốc 2,23 đo trên dữ liệu nhiễu tải máy; ma trận ngẫu nhiên khác ma trận ETA thật (có cấu trúc không gian); và lab chưa chạy Hungarian ở cỡ đó. Muốn biết phải chạy, hoặc dùng cắt tỉa để khỏi phải chạy, như bài 44.</p>
</details>

Bài trước: [bài 41](/articles/location-gan-nhat-sai-o-dau-do-bang-mo-phong). Bài tiếp: [bài 43](/articles/location-cua-so-lo-do-tre-doi-chat-luong-ghep).

## Tài liệu gốc

- [Hungarian algorithm (Wikipedia)](https://en.wikipedia.org/wiki/Hungarian_algorithm): bài toán phân công và nguồn gốc thuật toán; trang liệt kê bài báo của Kuhn (1955) và Munkres (1957) đã trích ở trên.
- [`pathfinding::kuhn_munkres` (docs.rs)](https://docs.rs/pathfinding/4.16.0/pathfinding/kuhn_munkres/fn.kuhn_munkres.html): chữ ký, quy ước tối đa hoá, yêu cầu số hàng không vượt số cột, độ phức tạp O(n³).
- [Student's t-test (Wikipedia)](https://en.wikipedia.org/wiki/Student%27s_t-test): kiểm định mẫu ghép cặp, nền của khoảng tin cậy trong bài.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho dữ liệu OSM.
