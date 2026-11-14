---
slug: location-ham-muc-tieu-diem-hen-khach-va-tai-xe
title: "Hàm mục tiêu điểm hẹn: đặt khách và tài xế lên cùng một cân"
summary: "Bài 32: hàm J(p) với bốn trọng số, đường đánh đổi đi bộ và km rỗng, so sánh cặp trên 20 seed, và một lần sửa quy tắc chọn trọng số sau khi thấy kết quả, có dải seed xác nhận. Khách điển hình đi bộ thêm 26,5 m."
category: location-routing
level: advanced
tags: [toi-noi, location, rust, pickup, objective-function, paired-comparison, simulation, trade-off]
language: vi
---

Trong một ví dụ nhỏ của lab, khách đang đứng ở cạnh 407 và tài xế tới đó mất 93,3 giây. Có một cạnh khác (403) mà tài xế tới nhanh hơn 13,8 giây, 79,5 giây. Nghe có lợi. Nhưng khách phải đi bộ thêm 52,0 giây, và đoạn có khách dài thêm 13,8 giây (138,8 so với 125,0 giây) vì xe xuất phát từ chỗ khác. Tính tổng, điểm «có lợi cho tài xế» đó tệ hơn 38,3 giây (349,8 so với 311,5). Câu hỏi của P04: lấy gì làm cân để so một giây của khách với một giây của tài xế, và nếu cân lệch thì ai trả giá.

Đây là bài 32 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi), và là bài trung tâm của cả phần. Với điểm hẹn ứng viên `p`, hàm mục tiêu của bài chốt ở đây và dùng lại ở P05, P07 là:

```text
J(p) = a·T_walk(khách → p) + b·max(T_walk, T_drive) + c·T_drive(tài xế → p)
     + d·T_trip(p → điểm trả, gồm quay đầu) + phạt(phía đường, hẻm, đi bộ quá xa)
```

Mọi số đo là **trên dữ liệu mô phỏng**: kịch bản `hcm-baseline` trên đồ thị TP.HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors), baseline là «đón đúng ghim» cộng «tài xế gần nhất theo chim bay» của [bài 30](/articles/location-tong-hop-mo-phong-su-kien-roi-rac-seed). Seed chỉnh trọng số là 20261201–20261210, seed đánh giá là 20270001–20270020, seed xác nhận là 20270101–20270120. Simulator không dự đoán đời thật; nó chỉ cho một thước đo lặp lại được.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Trọng số <code>a</code> (một giây khách đi bộ) giảm dần so với <code>c</code> (một giây tài xế chạy không). Theo bạn khách đi bộ nhiều dần, và km rỗng của tài xế sẽ giảm đều theo? Có cấu hình nào có thể làm cả hai giảm so với baseline không?</p>
<details><summary>Xem đáp án</summary><p>Đi bộ nhiều dần thì đúng (trung bình từ 24,3 m lên 149,7 m khi <code>a/c</code> từ 8 xuống 0,25 trên seed chỉnh). Km rỗng thì không giảm đều: tổng quãng của cả hai người nhỏ nhất ở <code>a/c</code> = 0,25 chứ không ở cực trị. Và mọi cấu hình đều có km rỗng trung bình thấp hơn baseline trên 10 seed này, kể cả <code>a/c</code> = 8 gần như đón ở ghim. Nên «km rỗng giảm» một mình không chứng minh trọng số nào tốt.</p></details>
</div>

## Đường đánh đổi: không cấu hình nào miễn phí

Lab quét `a/c` trên 10 seed chỉnh, giữ `b = c = d = 1`, và so từng cấu hình với baseline chạy cùng seed. Bảng sau là kết quả (trên dữ liệu mô phỏng, kịch bản `hcm-baseline`, seed chỉnh 20261201–20261210; **dải này không dùng để báo kết quả cuối**):

| Cấu hình | `walk_m` TB | `deadhead_m` TB | ETA TB (s) | `cancel_rate` | `u_turns` | Δ`walk_m` | Δ`deadhead_m` |
|---|---:|---:|---:|---:|---:|---:|---:|
| baseline (đón ghim) | 60,4 | 2.474,0 | 342,4 | 0,1329 | 1,652 | | |
| a = 0, b = 0 (bỏ qua khách) | 171,8 | 2.267,9 | 302,8 | 0,1126 | 1,254 | +111,4 | −206,2 |
| a/c = 0,25 | 149,7 | 2.236,6 | 299,4 | 0,1132 | 1,265 | +89,3 | −237,4 |
| a/c = 0,5 | 117,4 | 2.319,2 | 308,6 | 0,1117 | 1,255 | +57,0 | −154,9 |
| **a/c = 1** | 71,9 | 2.355,0 | 319,6 | 0,1174 | 1,331 | +11,5 | −119,0 |
| a/c = 2 | 37,1 | 2.426,6 | 331,6 | 0,1255 | 1,485 | −23,4 | −47,5 |
| a/c = 4 | 26,7 | 2.453,8 | 338,2 | 0,1288 | 1,586 | −33,7 | −20,2 |
| a/c = 8 | 24,3 | 2.439,8 | 338,2 | 0,1319 | 1,621 | −36,1 | −34,3 |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 450" role="img" aria-label="Hình 1. Đường đánh đổi trên 10 seed chỉnh 20261201–20261210: khi trọng số khách a/c giảm từ 8 xuống 0,25, khách đi bộ nhiều hơn (từ 24,3 lên 149,7 m) và tài xế chạy không ít hơn (từ 2.439,8 xuống 2.236,6 m); baseline ghim ở 60,4 m đi bộ và 2.474,0 m chạy không."><rect x="20" y="20" width="840" height="410" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Đường đánh đổi đi bộ và chạy không, seed chỉnh</text><path d="M100.0 380.0 L740.0 380.0" class="f-none s-line" stroke-width="1"/><text x="92.0" y="384.0" class="t-small" text-anchor="end">2.200</text><path d="M100.0 280.0 L740.0 280.0" class="f-none s-line" stroke-width="1"/><text x="92.0" y="284.0" class="t-small" text-anchor="end">2.300</text><path d="M100.0 180.0 L740.0 180.0" class="f-none s-line" stroke-width="1"/><text x="92.0" y="184.0" class="t-small" text-anchor="end">2.400</text><path d="M100.0 80.0 L740.0 80.0" class="f-none s-line" stroke-width="1"/><text x="92.0" y="84.0" class="t-small" text-anchor="end">2.500</text><path d="M153.3 380.0 L153.3 385.0" class="f-none s-muted" stroke-width="1"/><text x="153.3" y="400.0" class="t-small" text-anchor="middle">25</text><path d="M331.1 380.0 L331.1 385.0" class="f-none s-muted" stroke-width="1"/><text x="331.1" y="400.0" class="t-small" text-anchor="middle">75</text><path d="M508.9 380.0 L508.9 385.0" class="f-none s-muted" stroke-width="1"/><text x="508.9" y="400.0" class="t-small" text-anchor="middle">125</text><path d="M686.7 380.0 L686.7 385.0" class="f-none s-muted" stroke-width="1"/><text x="686.7" y="400.0" class="t-small" text-anchor="middle">175</text><path d="M100.0 380.0 L740.0 380.0" class="f-none s-muted" stroke-width="1.5"/><path d="M100.0 80.0 L100.0 380.0" class="f-none s-muted" stroke-width="1.5"/><text x="420.0" y="420.0" class="t-small" text-anchor="middle">walk_m trung bình (m) trên seed chỉnh</text><text x="40.0" y="66.0" class="t-small">deadhead_m trung bình (m)</text><path d="M150.8 140.2 L159.4 126.2 L196.4 153.4 L320.1 225.0 L481.9 260.8 L596.7 343.4" class="f-none s-pk" stroke-width="2"/><path d="M100.0 106.0 L740.0 106.0" class="f-none s-dead dash" stroke-width="1.5"/><circle cx="279.2" cy="106.0" r="6" class="f-dead s-dead" stroke-width="1.5"/><text x="291.2" y="98.0" class="t-small">baseline: đón đúng ghim (60,4 m; 2.474,0 m)</text><text x="740.0" y="122.0" class="t-small" text-anchor="end">baseline</text><circle cx="150.8" cy="140.2" r="5" class="f-pk s-pk" stroke-width="1.5"/><text x="141.8" y="144.2" class="t-small" text-anchor="end">a/c = 8</text><circle cx="159.4" cy="126.2" r="5" class="f-pk s-pk" stroke-width="1.5"/><text x="169.4" y="126.2" class="t-small">a/c = 4</text><circle cx="196.4" cy="153.4" r="5" class="f-pk s-pk" stroke-width="1.5"/><text x="206.4" y="145.4" class="t-small">a/c = 2</text><circle cx="320.1" cy="225.0" r="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="332.1" y="215.0" class="t-small">a/c = 1</text><circle cx="481.9" cy="260.8" r="5" class="f-pk s-pk" stroke-width="1.5"/><text x="491.9" y="252.8" class="t-small">a/c = 0,5</text><circle cx="596.7" cy="343.4" r="5" class="f-warn s-warn" stroke-width="1.5"/><text x="606.7" y="361.4" class="t-small">a/c = 0,25</text><circle cx="675.3" cy="312.1" r="5" class="f-none s-muted" stroke-width="1.5"/><text x="685.3" y="316.1" class="t-small">a = b = 0 (bỏ qua khách)</text><rect x="430.0" y="190.0" width="12.0" height="12.0" rx="6" class="f-pk" stroke-width="0"/><text x="448.0" y="200.0" class="t-small">a/c = 1 (vòng to) là p04-v1: quy tắc v2 chọn</text><rect x="430.0" y="214.0" width="12.0" height="12.0" rx="6" class="f-warn" stroke-width="0"/><text x="448.0" y="224.0" class="t-small">a/c = 0,25 là cấu hình quy tắc v1 đã chọn</text></svg>
</div>
<figcaption>Hình 1. Đường đánh đổi trên 10 seed chỉnh 20261201–20261210, kịch bản hcm-baseline, trên dữ liệu mô phỏng. Mỗi chấm là một trọng số a/c; trục ngang là quãng đi bộ của khách, trục dọc là quãng chạy không của tài xế. Đi dọc đường từ phải sang trái khách đi bộ ít dần còn tài xế chạy nhiều dần. Đường không đơn điệu ở phần giữa và mọi chấm đều nằm dưới đường baseline.</figcaption>
</figure>

Đọc đường cong này theo ba cách:

1. **Đi dọc đường là chuyển lợi ích từ bên này sang bên kia.** Từ `a/c` = 0,25 sang 8, khách bớt 125,4 m đi bộ trung bình, tài xế thêm 203,2 m chạy không. Không phép đo nào nói điểm nào «đúng»: mỗi điểm là một chính sách, và chính sách là chọn xem ai gánh.
2. **Đường không đơn điệu.** Tổng quãng `walk_m + deadhead_m` nhỏ nhất ở `a/c` = 0,25 (−148,1 m so với baseline, KTC t 95% [−221,9; −74,4] trên 10 seed chỉnh), chứ không ở `a = b = 0` (−94,8 m). Các chênh này trên 10 seed còn nhiễu: `a/c` = 8 có km rỗng thấp hơn `a/c` = 4, nên đừng đọc từng bậc của bảng như một quy luật.
3. **Mọi cấu hình đều có km rỗng trung bình thấp hơn baseline**, kể cả `a/c` = 8 khi khách gần như được đón ở ghim. Giả thuyết: một phần «tiết kiệm» không đến từ việc cân khách với tài xế mà từ việc loại các ghim xấu (cầu, hầm, đoạn đường quá ngắn) và chọn điểm gần tài xế hơn; lab không tách hai phần này.

Bảng chỉ cho thấy đánh đổi, **chưa** nói chọn điểm nào. Cách chọn có hai phiên bản, và chuyện giữa hai phiên bản đó là phần quan trọng nhất của bài.

## Hàm J, từng số hạng

`J` tính bằng **giây**, bốn số hạng và một khoản phạt. Mã chỉ là một biểu thức:

`examples/toi-noi-location/rust/crates/pickup/src/objective.rs`

```rust
    /// `J` từ các số hạng. Giá trị không hữu hạn (không có đường) cho `INFINITY`.
    pub fn j(&self, t: &Terms) -> f64 {
        let v = self.a * t.walk_s
            + self.b * t.walk_s.max(t.drive_s)
            + self.c * t.drive_s
            + self.d * t.trip_s
            + t.penalty_s;
        if v.is_finite() { v } else { f64::INFINITY }
    }
```

- `a·T_walk`: giá của việc khách đi bộ.
- `b·max(T_walk, T_drive)`: thời gian tới lúc **hai người gặp nhau**, tức người tới sau. Số hạng này cho biết khi tài xế mới là bên chậm (ví dụ tài xế mất 300 s, khách đi bộ 120 s), khách đi bộ thêm một chút không làm trễ cuộc hẹn; ngược lại khi khách chậm hơn thì mỗi giây đi bộ bị tính thêm lần nữa.
- `c·T_drive`: km rỗng của tài xế, quy ra giây.
- `d·T_trip`: đoạn có khách từ điểm hẹn tới điểm trả, gồm cả quay đầu (tính bằng A\* theo cạnh của [bài 23](/articles/location-chi-phi-re-u-turn-do-thi-theo-canh)). Điểm đón khác thì đoạn này khác, và nó quyết định cước.

Chính sách mặc định `p04-v1` đặt cả bốn bằng 1, tức «một giây của khách bằng một giây của tài xế». Đó là một **quyết định giá trị**, không phải sự thật đo được. Và nó không cân đều ở biên: khi tài xế là bên tới sau, điều xảy ra ở phần lớn yêu cầu (đi bộ không quá khoảng 190 s, ETA trung bình 307 s), một giây `T_drive` làm J tăng 2 giây (`b + c`), còn một giây `T_walk` chỉ làm J tăng 1 giây (`a`). Giây thêm của tài xế cũng là giây khách phải đợi, nên cách tính này có lý do, nhưng nó giải thích vì sao a = c vẫn đẩy khách đi bộ thêm.

Chính sách được ghi cùng `policy_version` vào một bản chụp (`PolicySnapshot`) có mã băm, để dựng lại vì sao quote chọn điểm đón này:

`examples/toi-noi-location/rust/crates/pickup/src/objective.rs`

```rust
impl Default for ObjectiveWeights {
    /// Chính sách `p04-v1`: **một giây của khách bằng một giây của tài xế** (`a = b = c = d = 1`).
    /// Quy tắc chọn ghi ở `results/p04.md`, mục LOC-32: các trọng số khác chỉ được dùng nếu qua cổng chặn trên tập seed chỉnh.
    fn default() -> Self {
        Self {
            policy_version: "p04-v1".into(),
            a: 1.0,
            b: 1.0,
            c: 1.0,
            d: 1.0,
            penalties: Penalties::default(),
            max_walk_m: 250.0,
        }
    }
}
```

**Ràng buộc cứng khác khoản phạt.** Ứng viên là cầu, hầm, cầu vượt, đường cao tốc, vòng xuyến, đoạn quá ngắn (nằm trong nút giao), quá xa ngưỡng đi bộ 250 m theo ước lượng, hoặc xe không vào được thì bị **loại**, không được tính J. Còn phía đường bất lợi, dừng trong hẻm, hay đi bộ quá 150 m chỉ **cộng giây phạt** (30 s nhân hệ số cấp đường, 20 s, và 20 s mỗi 100 m vượt thêm). Lý do tách đôi: khoản phạt luôn có thể bị một khoản lợi lớn hơn lấn át, mà dừng trên cầu vượt thì không có khoản lợi nào đủ lớn để biện minh. So với §3.4 của curriculum, lab chuyển «cấm dừng» từ phạt sang ràng buộc cứng, và chỉ có cờ cầu, hầm, vòng xuyến, loại đường, chưa có biển cấm dừng thật.

Có một ngoại lệ phải nhớ: **điểm ghim không bị kiểm ngưỡng 250 m**. Nó luôn được chấm nếu không vi phạm luật dừng, và chỉ bị thắng khi có ứng viên J nhỏ hơn. Vì thế `walk_m` max của `p04-v1` vẫn là 1.159,4 m, không phải 250 m. `T_drive` trong bài này tính với tài xế rảnh mà bộ ghép baseline sẽ gán cho từng ứng viên; [bài 35](/articles/location-xep-hang-diem-don-one-to-many-ngan-sach-do-tre) đo trường hợp chưa biết tài xế.

Ví dụ tay trên đồ thị tổng hợp nhỏ `fixtures/khu-mau.json` (khách cách ghim 35 m, đích cách 730 m chim bay, tài xế cách khách 618 m chim bay; 64 cạnh xe trong tầm đi bộ, còn 30 sau lọc và gom, chấm lại 6 bằng A\* theo cạnh):

| Ứng viên | `walk_s` | `drive_s` | `trip_s` | phạt | **J** | `walk_m` |
|---|---:|---:|---:|---:|---:|---:|
| cạnh 407 | 0,0 | 93,3 | 125,0 | 0,0 | **311,5** | 0,0 |
| ghim, cạnh 407 | 10,1 | 95,6 | 122,6 | 0,0 | 324,0 | 13,2 |
| cạnh 403 | 52,0 | 79,5 | 138,8 | 0,0 | 349,8 | 67,6 |
| cạnh 813 | 0,0 | 141,6 | 145,7 | 0,0 | 428,8 | 0,0 |
| cạnh 405 | 52,0 | 158,3 | 128,9 | 15,0 | 512,6 | 67,6 |

Kiểm bằng tay cho cạnh 403: 52,0 + max(52,0; 79,5) + 79,5 + 138,8 = 349,8. Cạnh 403 là ứng viên có thời gian tài xế ngắn nhất (79,5 s) nhưng thua, vì khách đi bộ 52,0 s và đoạn có khách dài hơn. J tốt nhất (311,5 s) hơn điểm ghim 12,5 s, đúng như bất biến «không tệ hơn ghim» mà [bài 35](/articles/location-xep-hang-diem-don-one-to-many-ngan-sach-do-tre) kiểm.

## Cách đo: so sánh cặp và ba dải seed

[Bài 30](/articles/location-tong-hop-mo-phong-su-kien-roi-rac-seed) cho thấy một seed không chứng minh gì: `deadhead_m` p50 của baseline dao động 1.874,9 ± 286,6 m giữa các seed, và seed 20261107 thấp hơn seed 20261106 tới 29,7% mà không đổi gì. Bài này dùng một cách đo chống lại nhiễu đó, và các bài sau của P04, P05 dùng lại.

**So sánh cặp.** Với mỗi seed `i`, chạy baseline và chiến lược mới trên cùng seed, lấy hiệu `Δᵢ = X_mới − X_baseline`. Cùng seed thì cùng chuỗi khách và cùng vị trí tài xế ban đầu, nên phần dao động do nhu cầu bị khử khi lấy hiệu. Khử **một phần**: sau khi hai lần chạy rẽ nhánh (ghép khác nhau), dòng ngẫu nhiên của huỷ chuyến và nhiễu tốc độ lệch nhau.

**Khoảng tin cậy t 95%.** Đơn vị phân tích là một seed (mỗi seed cho một giá trị của mỗi chỉ số), nên `n` = số seed, không phải số chuyến. Khoảng là `trung bình(Δ) ± t · sd(Δ) / √n` với `t` ở bậc tự do `n − 1` (2,093 khi n = 20):

`examples/toi-noi-location/rust/crates/pickup/src/stats.rs`

```rust
    let (b, n): (Vec<f64>, Vec<f64>) = pairs.iter().copied().unzip();
    let diffs: Vec<f64> = pairs.iter().map(|(a, c)| c - a).collect();
    let m = mean(&diffs);
    let sd = stddev(&diffs);
    let half = t_crit_975(diffs.len().saturating_sub(1)) * sd / (diffs.len() as f64).sqrt();
    let (bl, bh) = bootstrap_ci(&diffs, 4000, 0x5eed_0004);
```

Đọc một dòng của bảng: `deadhead_m` trung bình −107,0 m, khoảng [−151,6; −62,4]. Nửa độ rộng là 44,6 m, tính ngược ra độ lệch chuẩn của chênh lệch giữa các seed cỡ 95 m. Ba điều khoảng này **không** nói:

- Không chứa 0 không có nghĩa là lớn. −107 m là −4,5% của 2.357 m; có đáng hay không phải so với cái giá ở dưới.
- Chứa 0 không có nghĩa là không có tác động. `walk_m` trung bình +3,5 m [−1,1; +8,1] tương thích với cả −1 m lẫn +8 m.
- Khoảng nói về nhiễu giữa các seed của simulator này, không nói về đời thật. Mô hình sai thì khoảng hẹp vẫn sai.

Lab còn tính khoảng bootstrap 4.000 lần để đối chiếu; hai loại gần nhau, trừ vài chỉ số sát 0 (ví dụ `pickup_eta_s` p95: t chứa 0, bootstrap thì không).

**Ba dải seed không giao nhau.** Chỉnh trọng số trên 10 seed (20261201–20261210). Báo kết quả trên 20 seed đánh giá (20270001–20270020). Một dải xác nhận 20 seed (20270101–20270120) chỉ thêm sau khi quy tắc chọn trọng số phải sửa. Chọn trọng số trên cùng seed dùng để báo cáo thì kết quả lạc quan một cách hệ thống, vì ta đã chọn giá trị hợp với nhiễu của chính những seed đó.

**So nhiều chỉ số cùng lúc.** Bảng đầy đủ của lab có 19 chỉ số, mỗi chỉ số một khoảng 95%. Nếu không có hiệu ứng thật nào, trung bình một trong hai mươi khoảng vẫn tình cờ không chứa 0, nên lab đánh dấu sao không có nghĩa là đã chắc. Kiểm thô bằng Bonferroni cho 19 phép so (t = 3,459 thay vì 2,093, tính ngược từ khoảng đã in): `deadhead_m` trung bình còn [−180,7; −33,3], `pickup_eta_s` trung bình [−29,8; −13,6], `cancel_rate` [−0,0255; −0,0027], `walk_m` p50 [+20,8; +32,2]: các dòng chính vẫn không chứa 0. `meet_gap_s` p95 [−75,5; +7,5] thì không còn. Riêng trên dải xác nhận, `deadhead_m` trung bình [−177,5; −52,5] vẫn không chứa 0, còn `walk_m` trung bình +7,3 m thành [−0,4; +15,0]. Bonferroni chỉ tính 19 chỉ số của một bảng, chưa tính các cấu hình khác đã thử, nên đây là phép kiểm tối thiểu.

## Kết quả của `p04-v1`: ai được lợi, ai trả

Trên dữ liệu mô phỏng, `p04-v1` (a = b = c = d = 1), xếp hạng hai tầng, 20 seed đánh giá 20270001–20270020, kịch bản `hcm-baseline`, chênh trung bình cặp so với baseline với khoảng tin cậy t 95%:

| Chỉ số | Baseline | `p04-v1` | Chênh cặp [KTC 95%] | Ai |
|---|---:|---:|---:|---|
| `deadhead_m` trung bình | 2.357,0 | 2.250,0 | −107,0 [−151,6; −62,4] | tài xế được lợi |
| `deadhead_m` p95 | 5.699,4 | 5.599,7 | −99,7 [−344,6; +145,2] | không khác biệt rõ |
| `pickup_eta_s` trung bình | 328,7 | 307,0 | −21,7 [−26,6; −16,8] | khách chờ xe được lợi |
| `meet_gap_s` trung bình | 301,3 | 264,2 | −37,1 [−41,8; −32,3] | cả hai |
| `u_turns` mỗi chuyến | 1,6 | 1,3 | −0,3 [−0,3; −0,3] | tài xế |
| `cancel_rate` | 0,1272 | 0,1131 | −0,0141 [−0,0210; −0,0072] | khách, qua ETA ngắn hơn |
| **`walk_m` p50** | 14,3 | 40,9 | **+26,5 [+23,1; +30,0]** | **khách điển hình chịu thiệt** |
| `walk_m` trung bình | 64,7 | 68,2 | +3,5 [−1,1; +8,1] | không khác biệt rõ |
| `walk_m + deadhead_m` | 2.421,7 | 2.318,2 | −103,5 [−149,9; −57,1] | tổng quãng hai người |
| `trip_m` trung bình | 6.856,9 | 6.774,9 | −82,0 [−108,0; −56,0] | cước −368 đồng mỗi chuyến |

Ai được lợi: tài xế (km rỗng, quay đầu) và khách đợi xe (ETA ngắn hơn, huỷ ít hơn; trong mô hình huỷ gốc của lab, huỷ chỉ phụ thuộc ETA và **không** tính quãng đi bộ). Ai chịu thiệt: khách điển hình đi bộ thêm 26,5 m ở p50, và nền tảng thu trung bình ít hơn 368 đồng mỗi chuyến (cước tính từ `trip_m`) vì đoạn có khách ngắn đi.

Hai điều cần kèm theo con số, chi tiết ở [bài 40](/articles/location-cuoc-quote-giai-thich-doi-diem-don-cho-khach):

- **Độ nhạy theo số xe.** Ở bộ «60 tài xế, mật độ giảm một nửa» (kịch bản và seed đánh giá như trên), `deadhead_m` trung bình −111,5 m với khoảng [−289,6; +66,7], tức không khác biệt rõ, trên nền baseline chạy không 5.425,7 m. Giả thuyết (lab chưa kiểm): xe thưa thì đổi điểm đón khó rút ngắn quãng xe chạy, và nhiễu giữa các seed cũng lớn hơn. ETA, huỷ và quay đầu vẫn tốt hơn ở cả hai mật độ.
- **`walk_m` p50 xấu hơn ở 5 trong 6 bộ giả định**; chỉ bộ «khách đi bộ chậm» là khác, vì J dùng tốc độ đi bộ nên tự giảm đi bộ.

## Vì sao trung bình đi bộ «không đổi» mà khách điển hình vẫn thiệt

`walk_m` trung bình chỉ tăng +3,5 m [−1,1; +8,1]. Nhìn riêng dòng đó, người đọc sẽ nghĩ khách không mất gì. Phân tầng cho thấy ngược lại: hai nhóm bù trừ nhau.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Chênh walk_m cặp của p04-v1 trên 20 seed đánh giá: nhóm đuôi của baseline (178 cặp) đi bộ ít hơn 370,4 m, 3.602 khách còn lại đi bộ nhiều hơn 21,2 m, nên trung bình toàn bộ chỉ +3,5 m."><rect x="20" y="20" width="840" height="290" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Phân tầng chênh walk_m: đuôi và khách điển hình</text><path d="M560.0 70.0 L560.0 290.0" class="f-none s-muted" stroke-width="1.5"/><text x="560.0" y="306.0" class="t-small" text-anchor="middle">0</text><text x="40.0" y="76.0" class="t-small">khách đuôi của baseline (walk_m &gt; 300 m): 178 cặp</text><rect x="226.6" y="84.0" width="333.4" height="20.0" rx="2" class="f-pk" stroke-width="0"/><text x="218.6" y="99.0" class="t-b" text-anchor="end">−370,4 m</text><text x="40.0" y="132.0" class="t-small">3.602 khách còn lại</text><rect x="560.0" y="140.0" width="19.1" height="20.0" rx="2" class="f-warn" stroke-width="0"/><text x="587.1" y="155.0" class="t-b">+21,2 m</text><text x="40.0" y="188.0" class="t-small">trung bình mọi khách (KTC t 95% gồm 0)</text><rect x="560.0" y="196.0" width="3.1" height="20.0" rx="2" class="f-dead" stroke-width="0"/><text x="571.1" y="211.0" class="t-b">+3,5 m</text><text x="40.0" y="244.0" class="t-small">p50 theo seed (14,3 → 40,9 m)</text><rect x="560.0" y="252.0" width="23.9" height="20.0" rx="2" class="f-warn" stroke-width="0"/><text x="591.9" y="267.0" class="t-b">+26,5 m</text></svg>
</div>
<figcaption>Hình 2. Chênh walk_m cặp của p04-v1 trên 20 seed đánh giá, trên dữ liệu mô phỏng, theo nhóm khách: nhóm đuôi của baseline (walk_m baseline lớn hơn 300 m, 178 cặp khách) đi bộ ít hơn 370,4 m; 3.602 khách còn lại đi bộ nhiều hơn 21,2 m; trung bình chung +3,5 m; p50 theo seed từ 14,3 lên 40,9 m (+26,5 m).</figcaption>
</figure>

Nhóm đuôi của baseline (178 cặp khách, `walk_m` baseline > 300 m) đi bộ ít hơn 370,4 m. 3.602 khách còn lại đi bộ **nhiều hơn 21,2 m**; 27,6% khách đi thêm trên 50 m, 14,3% thêm trên 100 m, và 13,1% đi bộ ít hơn trên 50 m. «+3,5 m» là kết quả của hai dấu ngược nhau, không phải sự vắng mặt của tác động.

### Phần lớn đuôi đó là một artefact của baseline

Vì sao baseline có đuôi dài như vậy? Lab điều tra trên seed 20261106 (227 khách, ngoài ba dải trên): 11 khách (4,8%) có `walk_m` lớn hơn 300 m, và cả 11 đều có khoảng chim bay giữa chỗ đứng và điểm đón dưới 150 m. Đó không phải đi xa mà là **đi vòng**: ở 8 ca đầu bảng, đường đi bộ dài gấp 11 tới 59 lần chim bay (khách 76: 59,3 m chim bay, đi bộ 3.492,5 m). Mười một khách này chiếm **64,4% tổng quãng đi bộ** của baseline: `walk_m` trung bình 77,2 m, bỏ họ còn 28,9 m (phép chia khác với con số 78,0 m ở bài 30, vì bài đó chỉ tính chuyến hoàn thành).

Cách giải thích suy ra từ dữ liệu của lab: điểm ghim snap lên một đường ô tô khác với đường đi bộ gần chỗ đứng nhất, và hai đường không nối với nhau gần trong đồ thị đi bộ OSM. Chỗ đứng của khách do simulator sinh (Gauss quanh điểm nóng, chỉ yêu cầu *ghim* snap được trong 120 m), nên có thể rơi giữa khối nhà hay mặt nước. **Nguyên nhân chính xác (sông, dải phân cách, thiếu lối đi trong dữ liệu) chưa xác minh**, và **simulator không bị sửa**, vì sửa sẽ làm baseline đổi so với các bài đã xuất bản (mã băm nhật ký chuyến `100a862df76819aa` vẫn giữ nguyên).

Hệ quả cho cách đọc kết quả: `walk_m` p95 giảm 87,7 m và max giảm 379,9 m **không phải chiến thắng thật** của `p04-v1`. So cặp theo seed sau khi bỏ khách đuôi cho thấy p95 đổi dấu (lab `--walk-tail`, trên dữ liệu mô phỏng, chỉ tính khách hoàn thành ở cả hai lần chạy, nên hàng đầu lệch chút ít so với bảng 19 chỉ số):

| Dải seed | Lọc | Cặp | `walk_m` TB | `walk_m` p50 | `walk_m` p95 |
|---|---|---:|---:|---:|---:|
| Đánh giá | không lọc | 3.780 | +2,9 [−1,1; +6,8] | +25,7 [+22,6; +28,8] | −86,4 [−121,5; −51,3] |
| Đánh giá | bỏ baseline > 300 m | 3.602 | **+21,2 [+19,4; +23,1]** | +23,3 [+19,9; +26,8] | **+14,7 [+4,9; +24,5]** |
| Xác nhận | không lọc | 3.682 | +5,9 [+1,2; +10,5] | +26,4 [+22,8; +30,1] | −94,3 [−133,4; −55,3] |
| Xác nhận | bỏ baseline > 300 m | 3.518 | **+22,2 [+18,7; +25,7]** | +25,1 [+21,4; +28,7] | **+15,7 [+0,6; +30,7]** |

Bỏ đuôi đi, trung bình đi bộ tăng 21–22 m trên cả hai dải, và p95 **xấu đi** thay vì tốt lên. Lọc theo baseline có thiên lệch riêng (nó chọn đúng những khách baseline xấu), nhưng lọc cả hai bên cho cùng kết quả vì không khách nào có baseline ≤ 300 m mà `p04-v1` > 300 m. Nên «đuôi đi bộ giảm gần 88 m» không được đưa lên đầu trang: đó là báo cáo việc bỏ một lỗi của thước đo như một cải tiến của thuật toán.

## Quy tắc chọn trọng số và một lối rẽ đã xảy ra

Chọn một trọng số từ bảng đường đánh đổi cần một quy tắc, và quy tắc phải đặt trước khi nhìn kết quả đánh giá. Đây là cách nó đã diễn ra.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 3. Con đường dẫn tới p04-v1: quy tắc v1 chọn a/c = 0,25, kết quả trên seed đánh giá cho khách điển hình đi thêm 132,3 m, quy tắc được sửa sau khi đã thấy kết quả, v2 chọn a/c = 1, rồi dải seed xác nhận 20270101–20270120 đo lại."><defs><marker id="l32f" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs><rect x="20" y="20" width="840" height="290" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Quy tắc chọn trọng số và lần sửa</text><rect x="40.0" y="70.0" width="250.0" height="96.0" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="52.0" y="94.0" class="t-b">1. Quy tắc v1, đặt trước</text><text x="52.0" y="116.0" class="t-small">cổng: Δwalk p95 ≤ +25 m,</text><text x="52.0" y="133.0" class="t-small">Δcancel ≤ +0,01, hoàn thành</text><text x="52.0" y="150.0" class="t-small">không giảm quá 2%</text><path d="M292.0 118.0 L311.0 118.0" class="f-none s-muted" stroke-width="2" marker-end="url(#l32f)"/><rect x="315.0" y="70.0" width="250.0" height="96.0" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="327.0" y="94.0" class="t-b">2. v1 chọn a/c = 0,25</text><text x="327.0" y="116.0" class="t-small">trên seed chỉnh: walk+dead</text><text x="327.0" y="133.0" class="t-small">nhỏ nhất, mọi cấu hình</text><text x="327.0" y="150.0" class="t-small">đều qua cổng p95</text><path d="M567.0 118.0 L586.0 118.0" class="f-none s-muted" stroke-width="2" marker-end="url(#l32f)"/><rect x="590.0" y="70.0" width="250.0" height="96.0" rx="8" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="602.0" y="94.0" class="t-b">3. Đo trên 20 seed đánh giá</text><text x="602.0" y="116.0" class="t-small">khách điển hình đi thêm</text><text x="602.0" y="133.0" class="t-small">132,3 m (p50 14,3 → 146,7 m)</text><text x="602.0" y="150.0" class="t-small">67,6% khách thêm &gt; 50 m</text><rect x="40.0" y="196.0" width="250.0" height="96.0" rx="8" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="52.0" y="220.0" class="t-b">4. Sửa quy tắc SAU khi thấy</text><text x="52.0" y="242.0" class="t-small">thêm cổng: Δwalk trung bình</text><text x="52.0" y="259.0" class="t-small">≤ +25 m. Đây là lối rẽ</text><text x="52.0" y="276.0" class="t-small">của người phân tích</text><path d="M292.0 244.0 L311.0 244.0" class="f-none s-muted" stroke-width="2" marker-end="url(#l32f)"/><rect x="315.0" y="196.0" width="250.0" height="96.0" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="327.0" y="220.0" class="t-b">5. v2 chọn a/c = 1</text><text x="327.0" y="242.0" class="t-small">= p04-v1 (a = b = c = d = 1)</text><text x="327.0" y="259.0" class="t-small">a/c = 0,25, 0,5 và a = b = 0</text><text x="327.0" y="276.0" class="t-small">bị chặn</text><path d="M567.0 244.0 L586.0 244.0" class="f-none s-muted" stroke-width="2" marker-end="url(#l32f)"/><rect x="590.0" y="196.0" width="250.0" height="96.0" rx="8" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="602.0" y="220.0" class="t-b">6. Dải seed xác nhận</text><text x="602.0" y="242.0" class="t-small">20270101–20270120, chưa từng</text><text x="602.0" y="259.0" class="t-small">nhìn khi chọn: lặp lại,</text><text x="602.0" y="276.0" class="t-small">walk TB +7,3 m [+2,7; +12,0]</text><path d="M715.0 168 L715.0 181 L165.0 181 L165.0 192" class="f-none s-muted" stroke-width="2" marker-end="url(#l32f)"/></svg>
</div>
<figcaption>Hình 3. Con đường dẫn tới p04-v1. Quy tắc v1 đặt trước chọn a/c = 0,25, kết quả trên seed đánh giá cho khách điển hình đi thêm 132,3 m; quy tắc được sửa SAU khi đã thấy kết quả đó; v2 chọn a/c = 1; dải seed xác nhận chưa từng nhìn đo lại. Mô phỏng, kịch bản hcm-baseline.</figcaption>
</figure>

**Quy tắc v1 (đặt trước khi chạy).** Cổng chặn: chênh cặp trung bình của `walk_m` p95 không quá +25 m, `cancel_rate` không tăng quá 0,01, số chuyến hoàn thành không giảm quá 2%. Trong các cấu hình qua cổng, chọn cấu hình có `walk_m + deadhead_m` mỗi chuyến nhỏ nhất. Trên 10 seed chỉnh, v1 chọn `a/c` = 0,25.

**Kết quả của lựa chọn đó trên 20 seed đánh giá.** `walk_m` p50 tăng từ 14,3 lên 146,7 m: khách điển hình đi thêm **132,3 m** [+129,2; +135,4], 67,6% khách đi thêm trên 50 m, 51,8% thêm trên 100 m. Trong khi đó `walk_m` p95 vẫn «tốt hơn» 53,1 m. Cổng p95 không thấy gì, vì p95 của baseline do khoảng 5% khách ở các ca ghim sai quyết định, nên mọi cấu hình đều làm đuôi tốt lên và cổng luôn qua, kể cả khi khách điển hình đi thêm hơn 100 m.

**Sửa quy tắc (v2).** Lab thêm một cổng thứ hai: chênh trung bình của `walk_m` **trung bình** không quá +25 m. Với cổng này `a/c` = 0,25, 0,5 và `a = b = 0` bị chặn, còn `a/c` = 1, 2, 4, 8 qua; trong các cấu hình qua cổng, `a/c` = 1 có tổng quãng nhỏ nhất. Đó là `p04-v1`.

Điều phải nói rõ: **việc sửa này quyết định sau khi đã thấy kết quả trên seed đánh giá của cấu hình mà v1 chọn.** Đây là một ví dụ của «khu vườn những lối rẽ» (garden of forking paths): có nhiều cách hợp lý để viết quy tắc chọn, và khi chọn cách viết sau khi biết nó sẽ chọn gì, số trên seed đánh giá không còn «sạch» cho quyết định đó. Ngưỡng +25 m của cổng mới cũng vậy: lab không chứng minh được nó đặt độc lập với việc đã biết `a/c` = 1 là mặc định.

Cổng mới cũng chưa hết mù. Trung bình `walk_m` vẫn bị đuôi của baseline kéo xuống: trên seed chỉnh, `a/c` = 1 có Δ`walk_m` +11,5 m nên qua cổng, nhưng bỏ khách đuôi thì là **+25,6 m [+20,9; +30,3]**, và Δp95 là **+38,2 m [+18,2; +58,2]**. Nếu hai cổng +25 m tính trên khách không thuộc đuôi, `p04-v1` trượt cả hai, dù trượt cổng trung bình chỉ 0,6 m, nằm gọn trong nhiễu. Phép đo này làm sau khi đã chốt `p04-v1` nên không dùng để đổi chính sách ở đây; nó cho thấy kết quả của quy tắc phụ thuộc vào cách viết cổng tới mức nào.

**Dải seed xác nhận giảm rủi ro, không xoá nó.** Theo các nhật ký còn lại: lần chạy theo v1 lúc 1:40–1:48, sửa quy tắc, lần chạy theo v2 lúc 1:50–1:58, rồi lab bài 40 lúc 2:00–2:21 mới lần đầu chạy dải 20270101–20270120 trên bản đồ thật; `CONFIRM_SEEDS` không có trong lab nào khác của P04 lúc đó. Trên dải này (bài 40 báo đầy đủ), `p04-v1` lặp lại: `deadhead_m` trung bình −115,0 m [−152,8; −77,2] (so với −107,0 [−151,6; −62,4] ở dải đánh giá), `pickup_eta_s` −21,7 s, `cancel_rate` −0,0138, `u_turns` 1,6 → 1,3, `walk_m` p50 +28,4 m, và `walk_m` trung bình **+7,3 m [+2,7; +12,0], tức xấu hơn rõ**. Phân tầng: nhóm đuôi baseline −344,5 m (164 cặp), nhóm còn lại +22,2 m (3.518 cặp). Một lần lặp lại như vậy cho thấy hướng của kết quả không phụ thuộc vào 20 seed đã nhìn. Nó **không** cho biết v2 là quy tắc đúng, vì nó chỉ đo một cấu hình đã được chọn bằng con đường này, và cùng kịch bản, cùng mô hình; các lối rẽ khác (lưới `a/c`, ngưỡng 250 m, 150 m, các khoản phạt 30 s, 20 s, bộ chỉ số dùng làm cổng) vẫn còn nguyên. Dòng «trung bình đi bộ» đổi nhãn từ «không khác biệt rõ» sang khoảng loại 0, nhưng hai khoảng [−1,1; +8,1] và [+2,7; +12,0] chồng nhau rộng: nên đọc là hai lần ước lượng một mức tăng nhỏ, không phải hiệu ứng đã đổi.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` (bước dựng đồ thị ở [bài 06](/articles/location-osm-pbf-do-thi-duong-bo-xe-may-o-to)). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run --release -q --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc32_ham_muc_tieu -- --jobs 10
```

Cấu hình: Apple Silicon, `--release`, `--jobs 10`, seed chỉnh 20261201–20261210, seed đánh giá 20270001–20270020, đồ thị 126.699 nút, 319.074 cạnh. **Số thời gian không dùng trong bài vì máy đang chịu tải rất nặng** (load averages ghi ở đầu output 297,91 347,59 375,78). Mã băm nhật ký chuyến baseline sau khi thêm điểm cắm P04 vẫn là `100a862df76819aa`, giống từng byte với ghi chép ở P03. Output (rút gọn), phần quy tắc chọn và kết quả đánh giá:

```text
-- 4. quy tắc chọn trọng số (áp dụng trên seed chỉnh, KHÔNG dùng seed đánh giá) --
  a/c = 0,25                 Δwalk TB   +89.3  Δwalk p95   -23.5  Δcancel -0.0198  Δhoàn thành +2.3%  Δ(walk+dead)   -148.1  → v1: qua, v2: BỊ CHẶN
  a/c = 1 (mặc định)         Δwalk TB   +11.5  Δwalk p95   -64.1  Δcancel -0.0155  Δhoàn thành +1.8%  Δ(walk+dead)   -107.5  → v1: qua, v2: qua
  → v1 chọn: a/c = 0,25 (a = 0.25, b = 1); v2 chọn: a/c = 1 (mặc định) (a = 1, b = 1)
-- 5. đo trên seed ĐÁNH GIÁ: quy tắc v2 = p04-v1 (so sánh cặp, J hai tầng) --
  walk_m p50                             14.3       40.9      +26.5        [+23.1, +30.0]        [+23.4, +29.6]    0/20          14.4→39.7  *
  deadhead_m trung bình                2357.0     2250.0     -107.0       [-151.6, -62.4]       [-148.0, -66.0]    17/3      2335.7→2185.8  *
  pickup_eta_s trung bình               328.7      307.0      -21.7        [-26.6, -16.8]        [-26.1, -16.9]    20/0        326.4→300.5  *
  phân tầng theo baseline: khách có walk_m baseline > 300 m (đuôi): 178 cặp, chênh TB -370.4 m; còn lại: 3602 cặp, chênh TB +21.2 m
```

Bảng bỏ đuôi ở trên đến từ một lệnh riêng, chạy cả ba dải seed: thêm `-- --walk-tail --jobs 8` vào lệnh trên (output ở `results/p04.md`, mục LOC-32).

Quy tắc v1 và v2 trong mã chỉ là hai biểu thức; cổng thứ hai thêm vào ở dòng thứ hai:

`examples/toi-noi-location/rust/labs/src/bin/loc32_ham_muc_tieu.rs`

```rust
    let pass_v1 = |t: &Row| t.2 <= 25.0 && t.3 <= 0.01 && t.4 >= -0.02;
    let pass_v2 = |t: &Row| pass_v1(t) && t.7 <= 25.0;
```

## Giới hạn của ví dụ

- **Mọi số là của simulator.** Nhu cầu, huỷ chuyến, tốc độ đi bộ và vị trí đứng của khách đều là giả định của lab; không có GPS người thật. Mô hình huỷ gốc không phạt quãng đi bộ, nên «khách đi bộ thêm» không làm tăng huỷ; con số huỷ thấp hơn chỉ đến từ ETA ngắn hơn.
- **`walk_m` đo dọc đồ thị đi bộ giữa hai điểm đã snap**, không gồm đoạn từ chỗ đứng ra tới đường, ở cả baseline lẫn `p04-v1`. Con số tuyệt đối vì vậy thấp hơn quãng đi thật.
- **`p04-v1` trong simulator biết nhiều hơn một hệ thống thật**: thấy mọi tài xế rảnh lúc khách đặt và biết đúng luật của bộ ghép. Bài 34, 35 đo khi điều đó sai.
- **Trọng số `a = c` là một lập trường**, không phải đo lường. Giá trị thật của một giây của khách so với tài xế (chi phí, rủi ro huỷ, công bằng) lab không hiệu chỉnh.
- **Ca đuôi của baseline chưa tìm ra nguyên nhân cụ thể** và không được sửa; mọi so sánh về đuôi `walk_m` phải đọc kèm bảng bỏ đuôi.
- **Bảng của lab không hiệu chỉnh nhiều phép so sánh** (phép kiểm Bonferroni ở trên tính tay); ngưỡng +25 m của cổng v2 và lưới `a/c` do người chọn; dải xác nhận chỉ nhân đôi bằng chứng cho đúng một lựa chọn.
- Quét trọng số chỉ thay `a` (giữ `b = c = d = 1`; riêng cấu hình bỏ qua khách đặt cả `a` và `b` bằng 0); chưa quét đồng thời `b`, `c`, `d` hay các khoản phạt.

## Bài tập

**Bài 1.** Tính J cho một ứng viên có `walk_s` = 40, `drive_s` = 300, `trip_s` = 800, phạt 20 với `a = 2, b = 1, c = 1, d = 1`. Rồi đổi `a` về 1: J đổi bao nhiêu, và vì sao số hạng `max` không đổi?

<details><summary>Lời giải bài 1</summary>
<p>J = 2·40 + 1·max(40; 300) + 1·300 + 1·800 + 20 = 80 + 300 + 300 + 800 + 20 = 1.500. Với a = 1: 40 + 300 + 300 + 800 + 20 = 1.460, giảm 40. Số hạng max giữ 300 vì tài xế là bên tới sau, nên thời gian gặp nhau không phụ thuộc vào việc khách đi bộ 40 giây; chỉ giá trực tiếp của đi bộ (a·T_walk) thay đổi.</p>
</details>

**Bài 2.** Khoảng tin cậy t 95% của `deadhead_m` trung bình là [−151,6; −62,4] trên n = 20 seed. Tính trung bình chênh, nửa độ rộng và độ lệch chuẩn của chênh lệch giữa các seed (t = 2,093). Vì sao cỡ mẫu ở đây là số seed chứ không phải số chuyến (hàng nghìn)?

<details><summary>Lời giải bài 2</summary>
<p>Trung bình = (−151,6 − 62,4)/2 = −107,0 m; nửa độ rộng = 44,6 m; sd = 44,6 · √20 / 2,093 ≈ 95,3 m. Hàng nghìn chuyến hoàn thành trong 20 seed không phải mẫu độc lập: các chuyến trong cùng một seed không độc lập (chung dãy khách, tài xế, nhiễu tốc độ, cạnh tranh tài xế), nên coi từng chuyến là một mẫu độc lập sẽ cho khoảng tin cậy hẹp giả tạo. Một seed là một «ngày giả định» độc lập với seed khác, nên đơn vị phân tích là seed.</p>
</details>

**Bài 3.** Giả sử bạn là người đặt quy tắc v1 và đã thấy `a/c` = 0,25 làm khách điển hình đi thêm 132,3 m. Cách nào sửa quy tắc mà ít rơi vào lối rẽ nhất?

<details><summary>Lời giải bài 3</summary>
<p>Không có cách nào xoá hẳn lối rẽ, vì bạn đã biết kết quả. Giảm thiểu: (1) ghi quyết định sửa và lý do trước khi chạy lại bất cứ thứ gì; (2) chạy trên một dải seed hoàn toàn mới, chưa dùng để chọn hay đánh giá (dải xác nhận), và chỉ báo kết quả cuối từ dải đó; (3) công bố cả hai quy tắc và cả hai kết quả, như bài này; (4) thay cổng p95 bằng một thước đo phản ánh khách điển hình ngay từ đầu (p50 hoặc trung bình) thay vì thêm cổng chỉ vì thấy một kết quả khó chịu. Vẫn nên ghi rõ rằng ngưỡng mới là lựa chọn của người phân tích.</p>
</details>

Bài trước: [bài 31](/articles/location-geocoding-dia-chi-viet-nam-autocomplete). Bài tiếp: [bài 33](/articles/location-sinh-ung-vien-diem-don-do-thi-di-bo).

## Tài liệu gốc

- [Phân phối t của Student (Wikipedia)](https://en.wikipedia.org/wiki/Student%27s_t-distribution): khoảng tin cậy t cho trung bình của các chênh lệch.
- [Giảm phương sai: common random numbers (Wikipedia)](https://en.wikipedia.org/wiki/Variance_reduction): lý do dùng cùng seed khi so hai chiến lược.
- [Vấn đề so sánh nhiều phép (Wikipedia)](https://en.wikipedia.org/wiki/Multiple_comparisons_problem): vì sao nhiều khoảng 95% cùng lúc vẫn có khoảng tình cờ không chứa 0.
- [Gelman và Loken, «The garden of forking paths»](http://www.stat.columbia.edu/~gelman/research/unpublished/forking.pdf): bản thảo về việc quyết định phân tích sau khi thấy dữ liệu.
- [Pareto front (Wikipedia)](https://en.wikipedia.org/wiki/Pareto_front): đường đánh đổi giữa hai mục tiêu.
