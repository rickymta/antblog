---
slug: location-cua-so-lo-do-tre-doi-chat-luong-ghep
title: "Cửa sổ lô: đợi bao lâu là đáng"
summary: "Bài 43: quét cửa sổ lô 5–60 s theo mật độ. Ngày thường cửa sổ chỉ làm khách chờ thêm, kết quả rỗng; cao điểm 30 s giảm cả thời gian chờ 13,7 s lẫn km rỗng 327,9 m. Cửa sổ thích ứng không thắng được cửa sổ cố định. Trên dữ liệu mô phỏng."
category: location-routing
level: advanced
tags: [toi-noi, location, rust, dispatch, matching, batching, simulation, latency]
language: vi
---

Gom khách vào lô nghĩa là cố ý bắt họ đợi. Một khách đặt xe lúc 08:00:00 không được báo «đã có tài xế» ngay, mà phải đợi thêm 5, 10, hay 30 giây để hệ thống chờ xem còn ai đặt cùng lúc không. [Bài 42](/articles/location-ghep-theo-lo-hungarian-kuhn-munkres-rust) cho thấy phần thưởng: khi lô có đủ khách, phép gán tối ưu giảm km rỗng hàng trăm mét. Nhưng cũng cho thấy rằng ở mật độ thường lô chỉ có 1,64 khách, tức là chẳng có gì để gom. Câu hỏi của bài này là cái giá: đợi bao lâu thì thời gian chờ thêm của khách được bù lại, và bù bằng gì.

Câu trả lời trung thực gồm hai nửa. Ngày thường: đợi chỉ làm khách chờ thêm, còn km rỗng không phân biệt được với ghép tức thì (trừ cửa sổ 60 s). Cao điểm: đợi 30 s làm giảm cả thời gian chờ lẫn km rỗng. Và một cửa sổ «thông minh» tự co giãn theo lượng khách đến không thắng nổi một con số cố định. Bài này báo cáo cả ba điều, kể cả điều không như mong đợi.

Đây là bài 43 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng phép gán Hungarian của bài 42, chạy theo lô, và cách đọc so sánh có cặp của [bài 41](/articles/location-gan-nhat-sai-o-dau-do-bang-mo-phong): 20 seed đánh giá 20261106…20261125, hiệu «cấu hình trừ ghép tức thì», khoảng tin cậy 95% trên hiệu từng seed, «TỐT HƠN» hay «XẤU HƠN» chỉ khi khoảng không chứa 0. Tham số chọn trên dải seed chỉnh 9001…9010, tách hẳn. Mọi số đo đều lấy **trên dữ liệu mô phỏng**, với đồ thị HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors), kịch bản `hcm-baseline` và các biến thể mật độ ghi ở từng bảng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Cửa sổ lô 30 s làm khách đợi thêm tới 30 s cho mỗi lô trước khi được ghép. Ở mật độ thường (3,5 khách/phút), thời gian chờ trung bình của khách thay đổi thế nào? Ở cao điểm (7,0 khách/phút) thì sao: còn đáng đợi không?</p>
<details><summary>Xem đáp án</summary><p>Ở mật độ thường, <code>wait_s</code> trung bình <b>tăng</b> 20,4 s, khoảng [17,6; 23,1], còn km rỗng không phân biệt được với 0 (−4,7 m, khoảng [−38,3; 28,9]). Cửa sổ chỉ thêm thời gian chờ. Ở cao điểm, <code>wait_s</code> <b>giảm</b> 13,7 s [−20,3; −7,2] và <code>deadhead_m</code> giảm 327,9 m [−407,6; −248,1]: khách đợi trong cửa sổ nhưng được ghép với tài xế gần hơn nhiều, nên tổng thời gian chờ ngắn đi.</p></details>
</div>

## Cửa sổ lô hoạt động thế nào trong simulator

Theo hợp đồng `window_ms` của simulator: khi có khách chờ và có tài xế rảnh mà chưa có lô nào đang mở, simulator mở lô và gọi chiến lược ghép **một lần**, sau đúng `ms` mili giây, cho mọi khách đang chờ. Khách chưa được ghép ở lô đó chờ lô sau. Chiến lược trả `None` thì ghép ngay mỗi lần có khách mới hoặc có xe rảnh. Đó là «tức thì», điểm so sánh của cả bài, và trong bài này nó dùng cùng Hungarian, nên chênh lệch chỉ đến từ việc gom.

`BatchMatcher` cài đặt hợp đồng này với hai kiểu cửa sổ: cố định (`Fixed(ms)`) và thích ứng (`Adaptive`), kẹp trong `[min_ms, max_ms]` theo tốc độ khách đến quan sát trong `horizon_s` giây gần nhất.

`examples/toi-noi-location/rust/crates/matcher/src/batch.rs`

```rust
/// Cửa sổ thích ứng, đặt **trước** khi đo (không chỉnh trên seed đánh giá): đợi cỡ `k` khách dự kiến tới,
/// kẹp vào `[min_ms, max_ms]`; thiếu xe (rảnh ≤ khách chờ) thì không đợi thêm, dùng `min_ms`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdaptiveWindow {
    pub k: f64,
    pub min_ms: u64,
    pub max_ms: u64,
    /// Chiều dài cửa sổ trượt để ước lượng tốc độ khách đặt, giây.
    pub horizon_s: f64,
}
```

`examples/toi-noi-location/rust/crates/matcher/src/batch.rs`

```rust
    fn window_ms(
        &mut self,
        ctx: &Ctx,
        pending: &[PendingView],
        idle: &[DriverView],
    ) -> Option<u64> {
        self.observe(pending);
        let w = match self.cfg.window {
            Window::Immediate => return None,
            Window::Fixed(ms) => ms,
            Window::Adaptive(a) => {
                if idle.len() <= pending.len() {
                    a.min_ms
                } else {
                    let lambda = self.arrival_rate_per_s(ctx.now_ms, a.horizon_s);
                    ((a.k / lambda) * 1000.0)
                        .round()
                        .clamp(a.min_ms as f64, a.max_ms as f64) as u64
                }
            }
        };
        self.stats.lock().expect("khoá").windows.push(w);
        (w > 0).then_some(w)
    }
```

Quy tắc thích ứng được đặt **trước khi đo** và không chỉnh theo kết quả: đợi cỡ `k` khách dự kiến tới (`k / λ` giây, với `λ` là tốc độ khách đến), kẹp trong khoảng; nếu xe rảnh không nhiều hơn khách đang chờ (thiếu xe) thì không đợi thêm, dùng `min_ms`. Lưới thử: `k ∈ {1, 3}`, trần 10, 20 hoặc 40 s, sàn 2 s, cửa sổ trượt 300 s để ước `λ`.

Có một giả định của simulator quyết định cách đọc kết quả. Mô hình huỷ mặc định tính xác suất huỷ từ ETA **kể từ lúc gán xe**, nên thời gian đợi trong cửa sổ không làm tăng xác suất huỷ trực tiếp; chỉ hết kiên nhẫn 300 s mới tính từ lúc đặt. Nếu thực tế khách bỏ ngay khi đợi lâu, kết luận sẽ khác. Lab có thử bộ `huỷ theo chờ` (xác suất huỷ tính theo tổng thời gian chờ từ lúc đặt) và kết quả nằm ở mục «Mô hình huỷ làm đổi kết luận».

## Quy trình đo và tiêu chí chọn trước

Tham số của cửa sổ thích ứng chọn trên 10 seed chỉnh 9001…9010 theo **tiêu chí ghi trước khi đo**: cấu hình có `wait_s` trung bình thấp nhất (thời gian chờ từ lúc đặt tới lúc xe tới, đã gồm thời gian đợi trong cửa sổ). Km rỗng được báo cạnh đó nhưng không dùng để chọn, để một cấu hình không thể «mua» km rỗng bằng thời gian chờ mà lọt qua bộ lọc. Cũng theo tiêu chí đó, lab chọn một cửa sổ cố định để làm bản so sánh chi tiết.

Điểm đáng nhìn ngay trên seed chỉnh, trước khi đụng vào seed đánh giá: ở mật độ thường, `wait_s` trung bình của «tức thì» (307,7 s) đã thấp hơn mọi cấu hình có cửa sổ (từ 313,1 s ở 5 s đến 342,1 s ở 60 s). Ở cao điểm thì ngược lại: «tức thì» 489,4 s, còn cửa sổ cố định 30 s chỉ 481,1 s, thấp nhất lưới. Lab đã thấy hai chế độ khác nhau ngay trên dải chỉnh.

## Ngày thường: kết quả rỗng

Bộ `thường`: 120 xe, 3,5 khách/phút. «Tức thì» (Hungarian không gom), trung bình 20 seed: `wait_s` 324,1 s, `deadhead_m` 2.373,0 m, `wait_s` p95 671,0 s, hoàn thành 198,2/228,4 khách. Hiệu của từng cửa sổ cố định so với «tức thì»:

| Cửa sổ | `wait_s` trung bình | `deadhead_m` trung bình | `wait_s` p95 | km rỗng / khách (m) | huỷ | hoàn thành |
|---|---|---|---|---|---|---|
| 5 s | **+6,6 [3,8; 9,4]** | +10,8 [−15,4; 37,0] | +23,0 [−0,2; 46,1] | +9,9 [−18,2; 37,9] | −0,0005 [−0,0047; 0,0037] | +0,1 [−0,9; 1,1] |
| 10 s | **+9,5 [6,3; 12,8]** | +10,0 [−24,9; 44,8] | **+23,4 [8,0; 38,9]** | +11,0 [−21,2; 43,1] | −0,0016 [−0,0049; 0,0018] | +0,3 [−0,4; 1,1] |
| 20 s | **+13,4 [10,8; 15,9]** | −5,6 [−34,5; 23,3] | +11,6 [−5,1; 28,4] | −9,5 [−37,8; 18,8] | +0,0014 [−0,0029; 0,0058] | −0,3 [−1,4; 0,7] |
| 30 s | **+20,4 [17,6; 23,1]** | −4,7 [−38,3; 28,9] | **+26,0 [13,7; 38,3]** | −1,9 [−31,6; 27,7] | −0,0011 [−0,0052; 0,0029] | +0,2 [−0,7; 1,2] |
| 60 s | **+30,7 [27,3; 34,1]** | **−56,6 [−88,6; −24,6]** | **+38,7 [18,3; 59,1]** | **−39,8 [−70,5; −9,2]** | **−0,0050 [−0,0095; −0,0005]** | **+1,1 [0,1; 2,2]** |

Chữ in đậm là khoảng không chứa 0. Cách đọc đơn giản: **cột `wait_s` có dấu cộng và khoảng không chứa 0 ở cả năm hàng, cột km rỗng không có ý nghĩa thống kê ở bốn hàng đầu.** Khách đợi lâu hơn thật, từ 6,6 s tới 30,7 s trung bình, và không thấy lợi ích nào về km rỗng. Chỉ ở 60 s km rỗng mới giảm 56,6 m (và km rỗng tổng trên mỗi khách giảm 39,8 m), kèm huỷ −0,0050 và +1,1 chuyến, đổi lại phải trả 30,7 s chờ thêm trung bình. Tách theo vùng cho cửa sổ 5 s (cấu hình chọn trên seed chỉnh): khách ở cả ba vùng đều chờ thêm 5–7 s (trung tâm +7,4 [1,8; 12,9], vành trong +7,0 [2,9; 11,1], ngoại vi +5,2 [0,4; 10,1]) và km rỗng không đổi ở vùng nào. Bảng chi tiết của cửa sổ 5 s có một dòng XẤU HƠN ở `deadhead_m` p95 (+147,5 m [25,9; 269,1]), nhưng cửa sổ 10 s dài hơn lại cho +2,0 m [−176,2; 180,3]; với hàng chục ô trong các bảng này, một dấu sao không lặp lại như vậy nên được coi là may rủi cho tới khi chạy lại.

Đây là kết quả rỗng, và kết quả này khớp với bài 42: lô 1,64 khách thì gần như không có gì để tối ưu, nên mấy giây đợi thêm không mang lại lợi ích nào đo được. Bảng chọn trên seed chỉnh cũng đã cho dấu hiệu này (cửa sổ cố định chọn được là 5 s, cửa sổ ngắn nhất).

**Ai lợi, ai thiệt:** khách đều thiệt, mức thiệt khác nhau tuỳ độ dài cửa sổ; tài xế chỉ có lợi nhỏ ở cửa sổ 60 s. Cửa sổ 60 s đổi trung bình 30,7 s chờ của khách lấy 56,6 m km rỗng; bài tập 2 tính tỉ giá đó.

## Cao điểm: cả hai cùng giảm

Bộ `cao điểm`: 120 xe, 7,0 khách/phút. «Tức thì»: `wait_s` 515,4 s, `deadhead_m` 4.052,8 m, `wait_s` p95 1.152,9 s, km rỗng/khách 2.862,3 m, hoàn thành 313,4/450,4, hết kiên nhẫn 0,0660.

| Cửa sổ | `wait_s` trung bình | `deadhead_m` trung bình | `pickup_eta_s` trung bình | `wait_s` p95 | km rỗng / khách (m) |
|---|---|---|---|---|---|
| 5 s | −9,0 [−15,0; −3,0] | −163,9 [−230,7; −97,0] | −15,0 [−21,7; −8,4] | −26,7 [−64,1; 10,7] | −95,4 [−138,7; −52,1] |
| 10 s | −5,3 [−9,7; −0,9] | −134,9 [−190,7; −79,1] | −13,1 [−18,2; −7,9] | −23,8 [−57,2; 9,6] | −90,8 [−125,7; −55,8] |
| 20 s | −12,2 [−19,4; −4,9] | −270,2 [−345,8; −194,7] | −25,3 [−33,0; −17,6] | −56,2 [−99,5; −12,9] | −180,6 [−236,1; −125,0] |
| **30 s** | **−13,7 [−20,3; −7,2]** | **−327,9 [−407,6; −248,1]** | −32,0 [−39,9; −24,0] | −58,9 [−89,9; −27,9] | **−226,9 [−274,2; −179,6]** |
| 60 s | −17,2 [−24,6; −9,8] | −511,7 [−586,4; −437,1] | −50,4 [−58,6; −42,2] | −83,7 [−123,2; −44,2] | −357,8 [−410,6; −305,1] |

Mọi ô đều có khoảng không chứa 0, trừ `wait_s` p95 ở 5 s và 10 s. Cửa sổ 30 s, cấu hình chọn trên seed chỉnh, cho `wait_s` trung bình −13,7 s (17/20 seed), `deadhead_m` −327,9 m (20/20 seed), `pickup_eta_s` −32,0 s.

**Vì sao đợi mà lại chờ ít hơn?** Thời gian chờ của khách gồm hai phần: đợi trong cửa sổ và ETA của tài xế được giao. Lấy hiệu của `wait_s` trừ hiệu của `pickup_eta_s` sẽ ra phần giữ thêm trong cửa sổ: 30 s ở cao điểm giữ thêm −13,7 − (−32,0) ≈ 18,3 s (phép trừ từ hai cột trên), ít hơn 30 s vì cửa sổ mở khi có khách đầu tiên nên khách đến sau chỉ đợi phần còn lại. ETA lại giảm tới 32,0 s; lời giải thích hợp lý là lô lớn cho Hungarian nhiều lựa chọn hơn, dù lab không tách riêng. Tính chung, phần lợi vượt phần thiệt 13,7 s. Ở mật độ thường, cùng phép trừ cho 30 s giữ thêm +20,4 − 0,3 ≈ 20,1 s trong khi ETA gần như không giảm (+0,3 s): hết lợi, chỉ còn thiệt.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 420" role="img" aria-label="Hình 1. Đánh đổi giữa thời gian chờ thêm và km rỗng theo độ dài cửa sổ lô, so với ghép tức thì. Bộ thường: mọi cửa sổ làm khách chờ thêm từ 6,6 đến 30,7 giây, km rỗng chỉ giảm có ý nghĩa ở 60 giây (−56,6 mét). Bộ cao điểm: cả hai cùng giảm, cửa sổ 30 giây giảm 13,7 giây chờ và 327,9 mét, cửa sổ 60 giây giảm 17,2 giây và 511,7 mét."><rect x="20" y="20" width="840" height="395" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Mỗi chấm là một độ dài cửa sổ (5, 10, 20, 30, 60 s), hiệu so với ghép tức thì</text><rect x="110" y="114.2" width="203.8" height="225.8" class="f-phys-soft"/><path d="M150.8 90 L150.8 340" class="f-none s-line" stroke-width="1"/><text x="150.8" y="356" class="t-small" text-anchor="middle">−20</text><path d="M232.3 90 L232.3 340" class="f-none s-line" stroke-width="1"/><text x="232.3" y="356" class="t-small" text-anchor="middle">−10</text><path d="M313.8 90 L313.8 340" class="f-none s-line" stroke-width="1"/><text x="313.8" y="356" class="t-small" text-anchor="middle">0</text><path d="M395.4 90 L395.4 340" class="f-none s-line" stroke-width="1"/><text x="395.4" y="356" class="t-small" text-anchor="middle">+10</text><path d="M476.9 90 L476.9 340" class="f-none s-line" stroke-width="1"/><text x="476.9" y="356" class="t-small" text-anchor="middle">+20</text><path d="M558.5 90 L558.5 340" class="f-none s-line" stroke-width="1"/><text x="558.5" y="356" class="t-small" text-anchor="middle">+30</text><path d="M640.0 90 L640.0 340" class="f-none s-line" stroke-width="1"/><text x="640.0" y="356" class="t-small" text-anchor="middle">+40</text><path d="M110 315.8 L640 315.8" class="f-none s-line" stroke-width="1"/><text x="102" y="319.8" class="t-small" text-anchor="end">−500</text><path d="M110 275.5 L640 275.5" class="f-none s-line" stroke-width="1"/><text x="102" y="279.5" class="t-small" text-anchor="end">−400</text><path d="M110 235.2 L640 235.2" class="f-none s-line" stroke-width="1"/><text x="102" y="239.2" class="t-small" text-anchor="end">−300</text><path d="M110 194.8 L640 194.8" class="f-none s-line" stroke-width="1"/><text x="102" y="198.8" class="t-small" text-anchor="end">−200</text><path d="M110 154.5 L640 154.5" class="f-none s-line" stroke-width="1"/><text x="102" y="158.5" class="t-small" text-anchor="end">−100</text><path d="M110 114.2 L640 114.2" class="f-none s-line" stroke-width="1"/><text x="102" y="118.2" class="t-small" text-anchor="end">0</text><path d="M313.8 90 L313.8 340" class="f-none s-muted dash" stroke-width="1.5"/><path d="M110 114.2 L640 114.2" class="f-none s-muted dash" stroke-width="1.5"/><text x="375.0" y="376" class="t-small" text-anchor="middle">Δ wait_s trung bình (giây): âm = khách chờ ít hơn</text><text x="36" y="80" class="t-small">Δ deadhead_m (mét)</text><text x="118" y="130.2" class="t-small t-phys">cả hai tốt hơn</text><path d="M367.7 109.8 L391.3 110.2 L423.1 116.5 L480.2 116.1 L564.2 137.0" class="f-none s-warn" stroke-width="1.5"/><circle cx="367.7" cy="109.8" r="6" class="f-bg s-warn" stroke-width="2"/><text x="367.7" y="97.8" class="t-mono" text-anchor="middle">5 s</text><circle cx="391.3" cy="110.2" r="6" class="f-bg s-warn" stroke-width="2"/><text x="391.3" y="132.2" class="t-mono" text-anchor="middle">10 s</text><circle cx="423.1" cy="116.5" r="6" class="f-bg s-warn" stroke-width="2"/><text x="423.1" y="104.5" class="t-mono" text-anchor="middle">20 s</text><circle cx="480.2" cy="116.1" r="6" class="f-bg s-warn" stroke-width="2"/><text x="480.2" y="138.1" class="t-mono" text-anchor="middle">30 s</text><circle cx="564.2" cy="137.0" r="6" class="f-warn s-warn" stroke-width="2"/><text x="564.2" y="125.0" class="t-mono" text-anchor="middle">60 s</text><path d="M240.5 180.3 L270.6 168.6 L214.4 223.1 L202.1 246.4 L173.6 320.5" class="f-none s-phys" stroke-width="1.5"/><circle cx="240.5" cy="180.3" r="6" class="f-phys s-phys" stroke-width="2"/><text x="230.5" y="184.3" class="t-mono" text-anchor="end">5 s</text><circle cx="270.6" cy="168.6" r="6" class="f-phys s-phys" stroke-width="2"/><text x="280.6" y="172.6" class="t-mono">10 s</text><circle cx="214.4" cy="223.1" r="6" class="f-phys s-phys" stroke-width="2"/><text x="224.4" y="227.1" class="t-mono">20 s</text><circle cx="202.1" cy="246.4" r="6" class="f-phys s-phys" stroke-width="2"/><text x="212.1" y="250.4" class="t-mono">30 s</text><circle cx="173.6" cy="320.5" r="6" class="f-phys s-phys" stroke-width="2"/><text x="183.6" y="324.5" class="t-mono">60 s</text><text x="665" y="100" class="t-b">Chú giải</text><circle cx="673" cy="124" r="6" class="f-phys s-phys" stroke-width="2"/><text x="687" y="128">cao điểm (7,0 khách/phút)</text><text x="687" y="146" class="t-small">cả 5 cửa sổ: km rỗng giảm</text><text x="687" y="160" class="t-small">có ý nghĩa (KTC không chứa 0)</text><circle cx="673" cy="190" r="6" class="f-warn s-warn" stroke-width="2"/><text x="687" y="194">thường, km rỗng khác 0</text><text x="687" y="212" class="t-small">(chỉ cửa sổ 60 s)</text><circle cx="673" cy="240" r="6" class="f-bg s-warn" stroke-width="2"/><text x="687" y="244">thường, km rỗng không</text><text x="687" y="262" class="t-small">phân biệt được với 0</text><text x="665" y="300" class="t-small">Chưa vẽ: bộ thưa xe, huỷ theo chờ</text><text x="665" y="314" class="t-small">(xem bảng trong bài).</text><text x="40" y="400" class="t-small">Seed 20261106…25, Hungarian trong lô; hiệu có cặp so với Hungarian không gom. Trên dữ liệu mô phỏng.</text></svg>
</div>
<figcaption>Hình 1. Mỗi chấm là một độ dài cửa sổ, toạ độ là hiệu so với ghép tức thì (trục ngang: chờ thêm, trục dọc: km rỗng thêm). Ngày thường (vàng cam) nằm bên phải, tức mọi cửa sổ làm khách chờ thêm; chỉ chấm 60 s đặc có km rỗng giảm có ý nghĩa. Cao điểm (xanh ngọc) nằm trong vùng cả hai cùng tốt hơn, và đường nối các chấm đi xuống khi cửa sổ dài hơn. Trên dữ liệu mô phỏng, 20 seed 20261106…25.</figcaption>
</figure>

Ba lưu ý trước khi coi cao điểm là chiến thắng. Thứ nhất, `wait_s` chỉ tính trên chuyến hoàn thành, và ở cửa sổ 30 s tỉ lệ hết kiên nhẫn đi từ 0,0660 lên 0,0798 (+0,0138, khoảng [−0,0003; 0,0279]): không phân biệt được với 0, nhưng khoảng gần như nằm trọn phía xấu, nên đừng đọc nó là «không đổi». Một phần của −13,7 s có thể là thiên lệch người sống sót; chỉ số không dính thiên lệch đó là km rỗng tổng trên mỗi khách (−226,9 m). Hai, huỷ (−0,0016) và số chuyến hoàn thành (+0,7) đều có khoảng chứa 0: không thấy việc đợi lâu hơn giữ thêm được khách nào. Ba, cửa sổ 60 s trông còn tốt hơn 30 s (−17,2 s; −511,7 m), nhưng nó không được chọn trên seed chỉnh (486,3 s so với 481,1 s), và đổi sang nó sau khi đã nhìn seed đánh giá chính là cách tự lừa mình mà việc tách hai dải seed muốn tránh. Lab cũng không in tỉ lệ hết kiên nhẫn của cửa sổ 60 s ở cao điểm, trong khi ở `thưa xe` chính cửa sổ 60 s làm tỉ lệ đó tăng rõ.

### Khi xe thưa mà cầu không cao

Bộ `thưa xe` (60 xe, 3,5 khách/phút): cửa sổ 10, 20, 30 s giảm `deadhead_m` 206,9, 279,1 và 306,9 m, khoảng không chứa 0, mà `wait_s` không phân biệt được với 0 (−9,6; −10,4; −7,8 s). Cửa sổ 60 s giảm `deadhead_m` 502,0 m và `pickup_eta_s` 46,8 s, nhưng cái giá hiện ra: huỷ tăng +0,0076 [0,0015; 0,0137], số chuyến hoàn thành −1,8 [−3,1; −0,4], và tỉ lệ hết kiên nhẫn 0,0733 → 0,1112 (+0,0379 [0,0241; 0,0517]), XẤU HƠN. Vậy cửa sổ càng dài không phải lúc nào cũng càng tốt: ở bộ xe thưa, 60 s đã vượt quá điểm mà cái lợi còn bù được cái hại. Tức thì ở bộ này: `wait_s` 576,6 s, `deadhead_m` 4.689,1 m.

## Cửa sổ thích ứng không thắng cửa sổ cố định

Quy tắc «đợi cỡ k khách dự kiến tới» nghe hợp lý: đông khách thì đợi ngắn, vắng thì đợi dài hơn nhưng có trần. Kết quả:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 370" role="img" aria-label="Hình 2. Hiệu wait_s trung bình so với ghép tức thì. Bộ thường: cửa sổ cố định 5 giây cộng 6,6 giây, thích ứng k=3 trần 10 giây cộng 9,5 giây. Bộ cao điểm: cửa sổ cố định 30 giây trừ 13,7 giây, khoảng từ trừ 20,3 đến trừ 7,2; thích ứng k=1 trần 40 giây trừ 3,0 giây, khoảng chứa 0."><rect x="20" y="20" width="840" height="345" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Cửa sổ thích ứng không thắng cửa sổ cố định: Δ wait_s trung bình so với ghép tức thì</text><path d="M431.2 86 L431.2 296" class="f-none s-line" stroke-width="1"/><text x="431.2" y="312" class="t-small" text-anchor="middle">−20</text><path d="M493.8 86 L493.8 296" class="f-none s-line" stroke-width="1"/><text x="493.8" y="312" class="t-small" text-anchor="middle">−10</text><path d="M556.2 86 L556.2 296" class="f-none s-line" stroke-width="1"/><text x="556.2" y="312" class="t-small" text-anchor="middle">0</text><path d="M618.8 86 L618.8 296" class="f-none s-line" stroke-width="1"/><text x="618.8" y="312" class="t-small" text-anchor="middle">10</text><path d="M556.2 86 L556.2 296" class="f-none s-muted dash" stroke-width="1.5"/><text x="400" y="72" class="t-small">← khách chờ ít hơn</text><text x="650" y="72" class="t-small" text-anchor="end">chờ nhiều hơn →</text><text x="670" y="72" class="t-small">hiệu [KTC 95%]</text><text x="40" y="120.0" class="t-b">«thường»: 120 xe, 3,5 khách/phút</text><text x="52" y="152.0">cố định 5 s (chọn trên seed chỉnh)</text><path d="M580.0 148.0 L615.0 148.0" class="f-none s-warn" stroke-width="2.5"/><path d="M580.0 143.0 L580.0 153.0 M615.0 143.0 L615.0 153.0" class="f-none s-warn" stroke-width="2"/><circle cx="597.5" cy="148.0" r="5" class="f-warn s-warn" stroke-width="1"/><text x="670" y="152.0" class="t-mono">+6,6 [3,8; 9,4]</text><text x="52" y="184.0">thích ứng k=3, trần 10 s (đã chọn)</text><path d="M595.6 180.0 L636.2 180.0" class="f-none s-warn" stroke-width="2.5"/><path d="M595.6 175.0 L595.6 185.0 M636.2 175.0 L636.2 185.0" class="f-none s-warn" stroke-width="2"/><circle cx="615.6" cy="180.0" r="5" class="f-warn s-warn" stroke-width="1"/><text x="670" y="184.0" class="t-mono">+9,5 [6,3; 12,8]</text><text x="40" y="216.0" class="t-b">«cao điểm»: 120 xe, 7,0 khách/phút</text><text x="52" y="248.0">cố định 30 s (chọn trên seed chỉnh)</text><path d="M429.4 244.0 L511.2 244.0" class="f-none s-phys" stroke-width="2.5"/><path d="M429.4 239.0 L429.4 249.0 M511.2 239.0 L511.2 249.0" class="f-none s-phys" stroke-width="2"/><circle cx="470.6" cy="244.0" r="5" class="f-phys s-phys" stroke-width="1"/><text x="670" y="248.0" class="t-mono">−13,7 [−20,3; −7,2]</text><text x="52" y="280.0">thích ứng k=1, trần 40 s (đã chọn)</text><path d="M503.1 276.0 L571.9 276.0" class="f-none s-dead" stroke-width="2.5"/><path d="M503.1 271.0 L503.1 281.0 M571.9 271.0 L571.9 281.0" class="f-none s-dead" stroke-width="2"/><circle cx="537.5" cy="276.0" r="5" class="f-dead s-dead" stroke-width="1"/><text x="670" y="280.0" class="t-mono">−3,0 [−8,5; 2,5]</text><circle cx="52" cy="330" r="5" class="f-phys s-phys" stroke-width="1"/><text x="64" y="334" class="t-small">TỐT HƠN (KTC không chứa 0)</text><circle cx="262" cy="330" r="5" class="f-warn s-warn" stroke-width="1"/><text x="274" y="334" class="t-small">XẤU HƠN (KTC không chứa 0)</text><circle cx="480" cy="330" r="5" class="f-dead s-dead" stroke-width="1"/><text x="492" y="334" class="t-small">không phân biệt được</text><text x="40" y="350" class="t-small">Tham số chọn trên seed 9001…10 theo wait_s thấp nhất, báo cáo trên seed 20261106…25. Trên dữ liệu mô phỏng.</text></svg>
</div>
<figcaption>Hình 2. Hiệu `wait_s` trung bình so với ghép tức thì của cấu hình cố định và cấu hình thích ứng, mỗi cái chọn trên seed chỉnh. Ở bộ thường, cả hai làm khách chờ thêm. Ở cao điểm, cố định 30 s giảm 13,7 s có ý nghĩa; thích ứng chỉ giảm 3,0 s, khoảng chứa 0. Trên dữ liệu mô phỏng.</figcaption>
</figure>

- **Mật độ thường:** cấu hình thích ứng chọn trên seed chỉnh (`k = 3`, trần 10 s) cho đúng các số của cửa sổ cố định 10 s: `wait_s` +9,5 [6,3; 12,8], `deadhead_m` +10,0 [−24,9; 44,8]. Từ công thức: `λ` ≈ 3,5 khách/phút ≈ 0,058 khách/s, nên `k / λ ≈ 51 s`; kể cả khi tốc độ khách đổi từ 0,6× đến 1,4× trong giờ, `k / λ` vẫn nằm trong khoảng 37–86 s, luôn bị cắt về trần 10 s. Quy tắc thích ứng ở đây suy biến thành cửa sổ cố định 10 s (suy ra từ công thức; lab không in tỉ lệ bị cắt).
- **Cao điểm:** cấu hình đã chọn (`k = 1`, trần 40 s) cho `wait_s` −3,0 [−8,5; 2,5], không phân biệt được; `deadhead_m` −93,5 [−155,7; −31,3], và `u_turns` mỗi chuyến tăng nhẹ +0,021 [0,002; 0,040] (XẤU HƠN). Cố định 30 s tốt hơn rõ rệt về km rỗng (−327,9 m [−407,6; −248,1], hai khoảng không chồng nhau); ở `wait_s` (−13,7 s) hai khoảng chồng nhau và lab không so trực tiếp hai cấu hình, nên chỉ có thể nói là thích ứng không thắng được.
- **`thưa xe`:** thích ứng (`k = 1`, trần 10 s) `wait_s` −4,6 [−17,7; 8,6], `deadhead_m` −128,5 [−277,8; 20,7]: không phân biệt được.

Vì sao quy tắc không có lợi thế đo được? Lab không thí nghiệm riêng, nhưng công thức gợi ý hai nguyên nhân. Ở cao điểm `λ` ≈ 7 khách/phút ≈ 0,117 khách/s, nên `k / λ ≈ 8,6 s` với `k = 1` (khoảng 6 tới 14 s tuỳ pha trong giờ): cửa sổ thích ứng dài cỡ 9 s, tức cỡ các cửa sổ ngắn 5–10 s (−163,9 và −134,9 m), chứ không phải cỡ 30–60 s, nơi có lợi lớn. Quy tắc đếm **số khách đến**, trong khi lợi của gom lô phụ thuộc **khách nhiều hơn xe tới mức nào**. Và nhánh `idle.len() <= pending.len()` rút cửa sổ về sàn 2 s đúng lúc thiếu xe, trong khi số đo ở trên cho thấy lúc đó đợi lại có lợi. Một quy tắc dựa trên tỉ lệ khách chờ trên xe rảnh có thể khác, nhưng đó là giả thuyết chưa đo.

Phải nói thẳng giới hạn của kết luận này: lab thử **một** họ quy tắc thích ứng (6 cấu hình trong lưới), không phải mọi quy tắc. «Không thắng cố định» là kết quả của họ quy tắc này, không phải chứng minh mọi cửa sổ thích ứng đều vô ích. Thêm nữa, tiêu chí chọn trên seed chỉnh (chỉ `wait_s` trung bình) khó phân biệt các cấu hình ở cao điểm: toàn lưới nằm trong khoảng 481 tới 496 s.

## Mô hình huỷ làm đổi kết luận

Bộ `huỷ theo chờ`: 120 xe, 3,5 khách/phút, xác suất huỷ tính theo tổng thời gian chờ từ lúc đặt, dùng cùng đường cong (0,5 từ 180 s tới 900 s). Ở mô hình này thời gian đợi trong cửa sổ cộng thẳng vào xác suất huỷ.

| Cửa sổ | `wait_s` trung bình | huỷ | hoàn thành |
|---|---|---|---|
| 5 s | +5,9 [3,6; 8,1] | −0,0007 [−0,0047; 0,0033] | +0,1 [−0,8; 1,1] |
| 10 s | +6,3 [2,7; 9,8] | −0,0002 [−0,0039; 0,0035] | +0,1 [−0,8; 0,9] |
| 20 s | +9,6 [6,4; 12,9] | **+0,0052 [0,0017; 0,0086]** | **−1,2 [−2,0; −0,4]** |
| 30 s | +16,7 [12,7; 20,6] | **+0,0054 [0,0011; 0,0097]** | **−1,2 [−2,3; −0,2]** |
| 60 s | +26,1 [22,7; 29,5] | **+0,0095 [0,0049; 0,0141]** | **−2,1 [−3,2; −1,1]** |

Với cửa sổ từ 20 s trở lên, huỷ tăng và số chuyến hoàn thành giảm, có ý nghĩa thống kê: khách bỏ đi vì đợi lâu. Trong mô hình gốc (huỷ tính từ lúc gán) cùng các cửa sổ này không tăng huỷ. Nói cách khác, **việc đáng đợi 30 s ở cao điểm chỉ được chứng minh dưới giả định huỷ-từ-lúc-gán.** Lab chưa chạy `huỷ theo chờ` ở mật độ cao điểm, nơi cửa sổ có lợi; nên chưa biết kết luận «đáng» ở đó có còn đứng vững khi khách bỏ vì đợi hay không. Bộ `huỷ theo chờ` ở mật độ thường lại cho đúng kết luận rỗng của mục trên, và thậm chí xấu hơn một chút. Đây là chỗ cần thí nghiệm tiếp, không phải chỗ để kết luận.

## Chạy ví dụ và kiểm chứng

Từ gốc repo (cần `data/hcm-graph.bin`; thêm `--fixture` để chạy nhanh trên đồ thị tổng hợp):

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc43_cua_so_lo
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc43_cua_so_lo -- --variant 'thưa xe,huỷ theo chờ'
```

Lệnh đầu chạy hai bộ `thường` và `cao điểm`; lệnh sau chạy riêng `thưa xe` và `huỷ theo chờ` bằng cờ `--variant`. Tham số khác: `--seeds N` (20), `--tuning-seeds N` (10), `--jobs N`. Kết quả đầy đủ nằm ở `examples/toi-noi-location/results/p05.md`. Trích (máy chịu tải nặng; các số trong bảng là hiệu của số đếm và tỉ lệ nên không bị thời gian ảnh hưởng):

```text
seed chỉnh (không dùng để báo cáo): 9001…9010 (10 seed); seed đánh giá: 20261106…20261125 (20 seed)
  → chọn theo tiêu chí đặt trước (wait_s TB thấp nhất trên seed chỉnh): thích ứng = «thích ứng k=1 max=40s», cố định = «cố định 30 s»
  «tức thì» (tuyệt đối, trung bình 20 seed): wait_s 515.4, deadhead_m 4052.8, pickup_eta_s 495.6, wait_s p95 1152.9, km rỗng/khách 2862.3, cancel 0.3037, hoàn thành 313.4/450.4
  cố định 30 s                 -13.7 [-20.3, -7.2]* -327.9 [-407.6, -248.1]* -32.0 [-39.9, -24.0]* -58.9 [-89.9, -27.9]* -226.9 [-274.2, -179.6]* -0.0016 [-0.0089, 0.0057]       +0.7 [-2.7, 4.0]
```

`verify-p05.sh` chạy lab này trên fixture với 2 seed chính và 2 seed chỉnh để kiểm tra các bất biến, không tái lập các số trên.

## Giới hạn của ví dụ

- **Mô phỏng.** Mô hình huỷ, ngưỡng kiên nhẫn 300 s và nhu cầu đều là giả định. Kết luận về cửa sổ nhạy với mô hình huỷ, như bảng `huỷ theo chờ` cho thấy.
- **Chưa chạy:** `huỷ theo chờ` ở cao điểm; `dày xe` và `huỷ nặng` cho bài này; mật độ cao hơn 7 khách/phút với 120 xe.
- **Một họ quy tắc thích ứng.** Kết luận «không thắng» chỉ cho họ này.
- **Lô nhỏ ở mật độ thường làm kết quả rỗng gần như tất yếu.** Nếu thành phố thật có mật độ tập trung hơn ở từng vùng, kết luận vùng đó có thể khác; lab chưa chia theo vùng cho cao điểm.
- **Tiêu chí chọn chỉ nhìn `wait_s`.** Tiêu chí này bỏ qua km rỗng và hết kiên nhẫn; một tiêu chí khác (chẳng hạn hàm mục tiêu §3.4 có trọng số) sẽ chọn khác.
- **Không có tiền, không có thu nhập tài xế**; km rỗng là chỉ số thay thế.
- **Thời gian** (µs, ms) không dùng trong bài này.

## Bài tập

**Bài 1.** Từ bảng cao điểm, dùng hiệu `wait_s` trừ hiệu `pickup_eta_s` để ước lượng khách trung bình bị giữ thêm bao lâu trong cửa sổ 60 s. Vì sao con số nhỏ hơn 60 s?

<details><summary>Lời giải bài 1</summary>
<p>−17,2 − (−50,4) = +33,2 s. Nhỏ hơn 60 s vì cửa sổ mở khi khách đầu tiên của lô tới và đóng sau đúng 60 s: khách đầu đợi trọn 60 s, khách đến sau chỉ đợi phần còn lại. Trung bình nhỏ hơn độ dài cửa sổ. Cùng phép trừ ở mật độ thường cho 30,7 − (−5,5) = 36,2 s cho cửa sổ 60 s, cùng cỡ, nhưng ở đó phần ETA giảm nhỏ hơn phần giữ nên tổng là chờ thêm.</p>
</details>

**Bài 2.** Ở mật độ thường, cửa sổ 60 s giảm `deadhead_m` 56,6 m trên mỗi chuyến hoàn thành, đổi lấy `wait_s` +30,7 s cho mỗi khách. Với khoảng 198,2 chuyến hoàn thành mỗi seed, tính tổng giây chờ thêm và tổng mét km rỗng tiết kiệm mỗi seed, rồi suy ra «tỉ giá» giây-khách trên km-tài xế mà hai bên hoà. Đây là con số mô phỏng có quyết định thay bạn không?

<details><summary>Lời giải bài 2</summary>
<p>Chờ thêm: 198,2 × 30,7 ≈ 6.085 s. Km rỗng tiết kiệm: 198,2 × 56,6 ≈ 11.219 m, tức 11,2 km. Hoà ở khoảng 6.085 / 11,2 ≈ 543 s chờ của khách cho mỗi km rỗng của tài xế (xấp xỉ, vì làm tròn và vì số chuyến hoàn thành của hai cấu hình hơi khác nhau). Mô phỏng chỉ cho biết hai phía của phép đổi; việc 543 s khách đáng giá hơn hay kém 1 km tài xế là một lựa chọn chính sách, chính là các trọng số `a, b, c, d` ở §3.4, không đọc ra được từ lab.</p>
</details>

**Bài 3.** Thiết kế một thí nghiệm đủ chặt để kết luận cửa sổ lô thích ứng có thắng cố định hay không, nêu seed, tiêu chí chọn, các bộ giả định và chỉ số bảo vệ.

<details><summary>Lời giải bài 3</summary>
<p>Một phác thảo hợp lệ: định nghĩa quy tắc mới (chẳng hạn cửa sổ phụ thuộc tỉ lệ khách chờ trên xe rảnh) và tiêu chí chọn trước, ví dụ tối thiểu một hàm có trọng số của `wait_s` trung bình và km rỗng tổng trên mỗi khách đặt; chọn tham số chỉ trên seed 9001…9010; báo cáo trên seed 20261106…20261125 so với cửa sổ cố định tốt nhất cùng chọn trên seed chỉnh, có cặp, khoảng tin cậy 95%. Chạy ở tối thiểu `thường`, `cao điểm`, `thưa xe` và cả hai mô hình huỷ (từ lúc gán và theo tổng chờ), vì kết luận nhạy với huỷ. Chỉ số bảo vệ: `unserved_rate`, `cancel_rate`, số chuyến hoàn thành, và `wait_s` p95; bất kỳ chỉ số bảo vệ nào XẤU HƠN thì quy tắc không thắng dù km rỗng tốt hơn.</p>
</details>

Bài trước: [bài 42](/articles/location-ghep-theo-lo-hungarian-kuhn-munkres-rust). Bài tiếp: [bài 44](/articles/location-cat-tia-ung-vien-h3-ma-tran-thua).

## Tài liệu gốc

- [Hungarian algorithm (Wikipedia)](https://en.wikipedia.org/wiki/Hungarian_algorithm): thuật toán phân công dùng trong mỗi lô.
- [Student's t-test (Wikipedia)](https://en.wikipedia.org/wiki/Student%27s_t-test): kiểm định mẫu ghép cặp, nền của khoảng tin cậy trong bài.
- [Variance reduction (Wikipedia)](https://en.wikipedia.org/wiki/Variance_reduction): mục «common random numbers», lý do cùng seed giúp so sánh các cấu hình.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho dữ liệu OSM.
