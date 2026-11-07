---
slug: location-diem-hen-dong-cap-nhat-khi-hai-ben-di-chuyen
title: "Điểm hẹn động: cập nhật khi hai bên đang đi, và kết quả rỗng của hysteresis"
summary: "Bài 36: tính lại điểm hẹn khi khách và tài xế đang di chuyển. «Đổi ngay» có lợi nhưng khách đi thêm và đổi qua đổi lại; hysteresis đặt trước gần như không đổi gì. Kết quả rỗng, trên dữ liệu mô phỏng."
category: system-design
level: advanced
tags: [toi-noi, location, rust, rendezvous, hysteresis, simulation, optimization, meet-gap]
language: vi
---

Hình dung một tình huống đặt xe. Khách đặt xe ở một đoạn đường lớn, điểm hẹn được chọn lúc đặt cách chỗ khách đứng vài chục mét, và tài xế còn cách đó vài cây số. Ba phút sau, tài xế vẫn chưa nhúc nhích vì đang kẹt sau một xe tải trả hàng. Trên bản đồ của khách, điểm hẹn đứng yên; khách đã đi bộ tới nơi và đứng chờ giữa lề đường nóng. Lúc đặt xe, điểm hẹn là lựa chọn tốt nhất cho **thế giới lúc đó**. Giờ thế giới đã khác, và một điểm hẹn khác có thể đem lại cho cả hai một cuộc gặp tốt hơn, với giá là khách phải đi thêm. Câu hỏi của bài này là: khi nào hệ thống nên tự đề nghị dịch điểm hẹn, và khi nào nên để yên.

Nghe thì hiển nhiên: cứ tính lại `J` mỗi vài giây, nếu có chỗ tốt hơn thì đổi. Nhưng đổi điểm hẹn là việc hỏi khách đứng lên đi tiếp, và nếu GPS nhiễu thì điểm tốt nhất nhảy qua lại giữa hai vỉa hè. Bài này đo cả hai mặt. Trước hết là «đổi ngay», có lợi thật. Rồi là chính sách có trễ (hysteresis) với ngưỡng đặt từ trước. Chính sách đó **gần như không làm gì**, và bài này báo điều đó như một kết quả, giải thích vì sao từ chính mô hình, và nói rõ kết quả này có chứng minh được gì và không chứng minh được gì về ý tưởng.

Đây là bài 36 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng hàm mục tiêu `J` và cách xếp hạng ứng viên của [bài 32](/articles/location-ham-muc-tieu-diem-hen-khach-va-tai-xe) đến [bài 35](/articles/location-xep-hang-diem-don-one-to-many-ngan-sach-do-tre), và kịch bản của simulator [bài 30](/articles/location-tong-hop-mo-phong-su-kien-roi-rac-seed), nhưng chạy trong một mô phỏng riêng cho từng cặp (mục sau). Đường đưa vị trí tới app đã có ở [bài 19](/articles/location-go-fan-out-tracking-khach-noi-suy-marker); lab này không chạy nó. Phương pháp so sánh cặp theo seed (20 seed đánh giá, khoảng tin cậy t 95%) đã được định nghĩa ở bài 32 và không nhắc lại ở đây. Mọi số trong bài là **trên dữ liệu mô phỏng**, seed 20270001…20270020, đồ thị HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors). Simulator không dự đoán thế giới thật.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Có hai chính sách cho điểm hẹn động. A: tính lại mỗi 15 giây và đổi ngay khi <code>J</code> mới nhỏ hơn dù chỉ một chút. B: chỉ tính lại khi dự báo giờ gặp trễ hơn kế hoạch quá 30 s, chỉ đổi khi <code>J</code> giảm ít nhất 25 s và 10%, và chỉ khi khách đồng ý. Trong tình huống tài xế trễ 120 s, bạn đoán B đổi điểm hẹn bao nhiêu lần mỗi ca so với A, và B có lấy được lợi ích của A không?</p>
<details><summary>Xem đáp án</summary><p>A đổi 2,12 lần mỗi ca, trong đó 0,64 lần là quay lại chỗ đã bỏ. B đổi <b>0,03</b> lần mỗi ca, tức là gần như không bao giờ. Vì vậy B gần như trùng chính sách tĩnh: khác biệt so với tĩnh không có chỉ số nào có khoảng tin cậy loại 0. A thì có: giờ cả hai có mặt sớm hơn 22,5 s, tài xế chạy ít hơn 93,5 m, nhưng khách đi bộ thêm 59,0 m. Hysteresis chặn được đổi qua đổi lại, và chặn luôn cả lợi ích.</p></details>
</div>

## Điểm hẹn động là gì, và lab đo nó thế nào

Ở các bài trước, điểm hẹn được chọn **một lần** lúc khách đặt xe. Điểm hẹn động thêm một vòng lặp: trong lúc khách đi bộ tới điểm hẹn và tài xế chạy tới, nền tảng quan sát cả hai, ước tính lại giờ tới của từng người, và khi cần thì chấm lại các ứng viên, rồi đề nghị đổi.

Lab dựng một mô phỏng riêng, theo bước 5 s, cho từng cặp (khách, tài xế). Mỗi seed lấy 8 cặp: khách trong danh sách của seed, ghép với tài xế rảnh gần nhất chưa dùng, cách 700–4.000 m chim bay để có đủ thời gian di chuyển. Vậy mỗi tình huống có 20 × 8 = 160 ca. Mô phỏng này **không** thay thế simulator sự kiện rời rạc của bài 30; nó chạy riêng vì chỉ cần hai người đi trên đồ thị, không cần ghép chuyến.

Điều quan trọng là nền tảng không biết sự thật. Nó chỉ thấy vị trí quan sát (khách có thể nhiễu GPS) và tốc độ ước từ quãng đã đi trong 30 s gần nhất. Hệ số tốc độ thật của khách (ví dụ khách chậm 0,5 lần so với dự kiến) nằm ngoài tầm nhìn. Ba chính sách được so sánh:

- **Tĩnh**: giữ điểm hẹn chọn lúc đầu. Điểm đó được chọn khi **đã biết** vị trí tài xế, nên đây là cận tốt nhất của «không đổi», không phải một baseline yếu.
- **Đổi ngay**: tính lại mỗi 15 s khi dự báo giờ gặp đã trễ quá 5 s, và đổi ngay khi `J` mới nhỏ hơn `J` cũ, không hỏi khách.
- **Có trễ + đồng ý**: hysteresis. Chỉ tính lại khi dự báo trễ hơn kế hoạch quá 30 s, cách lần đổi trước ít nhất 45 s, khi cả hai còn cách điểm hẹn hơn 60 s; chỉ đổi nếu `J` giảm ít nhất 25 s **và** ít nhất 10%, khách đi thêm không quá 120 m, và khách đồng ý.

Năm tình huống: bình thường (đối chứng), khách đi chậm (0,5 lần tốc độ dự kiến), tài xế trễ 120 s (đứng yên 120 s rồi chạy bình thường), cả hai cùng lúc, và GPS khách nhiễu σ = 10 m với tốc độ đúng. Chỉ số chính là `t_pick_s` (giờ cả hai có mặt tại điểm hẹn cuối cùng), `meet_gap_s` (lệch giờ tới của hai người), `walk_m` và `drive_m` (tổng quãng khách đi bộ và tài xế chạy, **kể cả quãng bỏ dở** sau khi đổi).

## Ba cổng của hysteresis

Hysteresis ở đây là một chuỗi cổng nối tiếp. Cổng đầu quyết định có **tính lại** hay không, cổng sau quyết định có **đổi** hay không. Mặc định trong mã để `naive_interval_s` = 10 s; lab bài này đặt 15 s cho chính sách «đổi ngay».

`examples/toi-noi-location/rust/crates/pickup/src/dynamic.rs`

```rust
impl Default for DynConfig {
    fn default() -> Self {
        Self {
            tick_s: 5.0,
            naive_interval_s: 10.0,
            trigger_dev_s: 30.0,
            cooldown_s: 45.0,
            lock_s: 60.0,
            min_gain_s: 25.0,
            min_gain_frac: 0.10,
            max_extra_walk_m: 120.0,
            speed_window_s: 30.0,
            planned_walk_mps: 1.3,
            max_time_s: 1_500.0,
        }
    }
}
```

Cổng thứ nhất (tính lại) nằm trong vòng lặp theo bước 5 s:

`examples/toi-noi-location/rust/crates/pickup/src/dynamic.rs`

```rust
            let eligible = match policy {
                Policy::Static => false,
                Policy::Naive => t - last_replan >= cfg.naive_interval_s && dev > 5.0,
                Policy::Hysteresis => {
                    dev > cfg.trigger_dev_s
                        && t - last_change >= cfg.cooldown_s
                        && t - last_replan >= cfg.cooldown_s * 0.5
                        && eta_c.max(eta_d) > cfg.lock_s
                }
            };
```

`dev` là độ lệch giữa giờ gặp dự báo bây giờ và giờ gặp đã dự báo lúc lập kế hoạch. Bốn điều kiện nối bằng `&&`: lệch quá 30 s, đã nguội 45 s sau lần đổi trước, cách lần tính lại trước ít nhất nửa thời gian nguội, và còn hơn 60 s trước khi một trong hai tới (không đổi khi sắp gặp). Qua được cổng này mới tới cổng thứ hai, nơi `J` được chấm lại:

`examples/toi-noi-location/rust/crates/pickup/src/dynamic.rs`

```rust
                    let moved = haversine_m(cand.cand.point, cur.cand.point) > 5.0;
                    let extra_walk = cand.walk_m - rem_walk_m;
                    let gain = j_cur - j_new;
                    let ok_gain = match policy {
                        Policy::Naive => gain > 0.0,
                        _ => gain >= cfg.min_gain_s && gain >= cfg.min_gain_frac * j_cur,
                    };
                    let ok_walk = policy == Policy::Naive || extra_walk <= cfg.max_extra_walk_m;
                    if moved && ok_gain && ok_walk {
                        let accept = match policy {
                            Policy::Hysteresis => coin.f64() < consent_probability(extra_walk),
                            _ => true,
                        };
```

Và cổng thứ ba, mô hình khách đồng ý, **là giả định của lab**, không phải đo đạc:

`examples/toi-noi-location/rust/crates/pickup/src/dynamic.rs`

```rust
/// Mô hình khách đồng ý đổi điểm (giả định của lab): xác suất giảm theo quãng đi thêm.
pub fn consent_probability(extra_walk_m: f64) -> f64 {
    (0.95 - 0.004 * extra_walk_m.max(0.0)).clamp(0.05, 0.95)
}
```

Khách đi thêm 0 m đồng ý với xác suất 0,95; đi thêm 100 m thì còn 0,55; đi thêm 120 m, mức tối đa mà cổng đi bộ cho phép, thì còn 0,47. Không ai đo hành vi này. Lab chọn một đường dốc hợp lý để có thứ mà mô phỏng chạy được, và mọi con số về «khách đồng ý» chỉ đáng tin tới mức đường dốc này đáng tin.

## Kết quả: «đổi ngay» có lợi, có giá, và hysteresis đứng im

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 380" role="img" aria-label="Hình 1. Số lần đổi điểm hẹn mỗi ca trong năm tình huống. Chính sách đổi ngay đổi 0,42; 0,61; 2,12; 1,97 và 2,55 lần mỗi ca, trong đó đổi qua đổi lại 0,05; 0,08; 0,64; 0,92 và 1,52 lần. Chính sách có trễ cộng đồng ý đổi 0,00; 0,06; 0,03; 0,03 và 0,06 lần."><text x="36" y="28" class="t-b">Số lần đổi điểm hẹn mỗi ca, 160 cặp khách–tài xế mỗi tình huống</text><rect x="36" y="42" width="14" height="12" rx="2" class="f-warn-soft s-warn" stroke-width="1.2"/><text x="58" y="52" class="t-small">đổi ngay (kèm số lần đổi qua đổi lại, flip)</text><rect x="360" y="42" width="14" height="12" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><text x="382" y="52" class="t-small">có trễ + khách đồng ý</text><path d="M300 70 L300 340" class="f-none s-line" stroke-width="1" stroke-linecap="round"/><text x="36" y="102">bình thường (đối chứng)</text><rect x="300" y="78" width="63" height="18" rx="2" class="f-warn-soft s-warn" stroke-width="1.2"/><text x="371" y="91" class="t-mono">0,42 (flip 0,05)</text><rect x="300" y="100" width="2" height="18" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><text x="310" y="113" class="t-mono">0,00</text><text x="36" y="154">khách đi chậm 0,5×</text><rect x="300" y="130" width="91.5" height="18" rx="2" class="f-warn-soft s-warn" stroke-width="1.2"/><text x="399.5" y="143" class="t-mono">0,61 (flip 0,08)</text><rect x="300" y="152" width="9" height="18" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><text x="317" y="165" class="t-mono">0,06</text><text x="36" y="206">tài xế trễ 120 s</text><rect x="300" y="182" width="318" height="18" rx="2" class="f-warn-soft s-warn" stroke-width="1.2"/><text x="626" y="195" class="t-mono">2,12 (flip 0,64)</text><rect x="300" y="204" width="4.5" height="18" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><text x="312.5" y="217" class="t-mono">0,03</text><text x="36" y="258">cả hai: chậm 0,5×, trễ 120 s</text><rect x="300" y="234" width="295.5" height="18" rx="2" class="f-warn-soft s-warn" stroke-width="1.2"/><text x="603.5" y="247" class="t-mono">1,97 (flip 0,92)</text><rect x="300" y="256" width="4.5" height="18" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><text x="312.5" y="269" class="t-mono">0,03</text><text x="36" y="310">GPS khách nhiễu σ = 10 m</text><rect x="300" y="286" width="382.5" height="18" rx="2" class="f-warn-soft s-warn" stroke-width="1.2"/><text x="690.5" y="299" class="t-mono">2,55 (flip 1,52)</text><rect x="300" y="308" width="9" height="18" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><text x="317" y="321" class="t-mono">0,06</text><text x="36" y="364" class="t-small">Trên dữ liệu mô phỏng, seed 20270001…20270020, kịch bản hcm-baseline. flip: đổi sang chỗ cách điểm đã bỏ dưới 40 m.</text></svg>
</div>
<figcaption>Hình 1. Số lần đổi điểm hẹn mỗi ca trong năm tình huống, từ output của <code>loc36_diem_hen_dong</code>. Cam: «đổi ngay», kèm số lần đổi qua đổi lại (flip). Xanh: hysteresis có khách đồng ý. Trên dữ liệu mô phỏng, 20 seed, 160 ca mỗi tình huống.</figcaption>
</figure>

Hình 1 là kết quả chính. «Đổi ngay» đổi từ 0,42 lần mỗi ca (bình thường) tới 2,55 lần (GPS nhiễu), còn hysteresis đổi từ 0,00 tới 0,06 lần. Trong khi đó hysteresis **vẫn tính lại** nhiều: 3,1 tới 12,9 lần mỗi ca. Tức là cổng thứ nhất mở thường xuyên, còn cổng thứ hai gần như luôn đóng. Ở tình huống tài xế trễ 120 s: 12,79 lần tính lại cho 0,03 lần đổi, cỡ một lần đổi trên 400 lần tính lại (số làm tròn hai chữ số thập phân, nên chỉ nên đọc là «vài trăm»).

Bảng dưới là chênh lệch trung bình so với chính sách tĩnh, chênh lệch cặp theo seed, trong ngoặc là khoảng tin cậy t 95% trên 20 seed. Dấu `*`: khoảng không chứa 0. Trên dữ liệu mô phỏng.

| Tình huống | Chính sách | `t_pick_s` | `meet_gap_s` | `walk_m` | `drive_m` |
|---|---|---:|---:|---:|---:|
| Bình thường | đổi ngay | −3,0 [−5,1; −0,9]* | −7,5 [−13,9; −1,1]* | +5,1 [−0,8; +11,0] | −17,5 [−31,1; −3,9]* |
| Bình thường | có trễ | +0,0 [+0,0; +0,0] | +0,0 [+0,0; +0,0] | +0,0 [+0,0; +0,0] | +0,0 [+0,0; +0,0] |
| Khách đi chậm 0,5× | đổi ngay | −6,8 [−19,2; +5,6] | **+22,5** [+12,2; +32,8]* | −22,9 [−34,3; −11,5]* | **+24,8** [+3,2; +46,4]* |
| Khách đi chậm 0,5× | có trễ | −10,0 [−22,5; +2,6] | +1,2 [−4,3; +6,7] | −10,4 [−19,3; −1,6]* | +11,7 [−6,1; +29,5] |
| Tài xế trễ 120 s | đổi ngay | −22,5 [−32,1; −12,9]* | −80,5 [−99,6; −61,3]* | **+59,0** [+40,8; +77,1]* | −93,5 [−121,5; −65,5]* |
| Tài xế trễ 120 s | có trễ | −4,9 [−12,6; +2,8] | −4,5 [−11,6; +2,6] | −5,0 [−15,3; +5,3] | −5,8 [−11,6; +0,1] |
| Cả hai | đổi ngay | −6,5 [−19,6; +6,6] | −17,6 [−30,5; −4,8]* | +4,8 [−10,1; +19,8] | −30,5 [−54,6; −6,4]* |
| Cả hai | có trễ | −7,5 [−19,7; +4,7] | +0,7 [−3,5; +4,8] | −4,7 [−12,7; +3,3] | −6,3 [−14,3; +1,6] |
| GPS nhiễu σ = 10 m | đổi ngay | −4,7 [−14,4; +5,1] | −120,6 [−149,0; −92,1]* | **+40,5** [+22,9; +58,0]* | −12,6 [−28,4; +3,2] |
| GPS nhiễu σ = 10 m | có trễ | −4,0 [−12,0; +4,1] | −7,2 [−15,6; +1,3] | −2,2 [−13,8; +9,3] | −7,8 [−17,6; +1,9] |

Nguồn: `results/p04.md`, mục LOC-36. Chênh `walk_m` và `drive_m` tính cả quãng bỏ dở sau khi đổi.

Có ba điều cần đọc cho đúng.

**«Đổi ngay» có lợi, nhưng không đều và có người trả.** Ở tình huống tài xế trễ, tài xế chạy ít hơn 93,5 m và hai người gặp nhau sớm hơn 22,5 s, nhưng khách đi bộ thêm 59,0 m, và 0,64 lần mỗi ca là đổi qua đổi lại. Khách là bên trả. Ở tình huống khách đi chậm thì ngược lại: khách đi bộ ít hơn 22,9 m, nhưng tài xế chạy thêm 24,8 m, và `meet_gap_s` **xấu đi** 22,5 s. Số đo phù hợp với cách hiểu này: khi tài xế là bên tới sau, kéo điểm hẹn về phía khách làm người tới trước (khách) đứng chờ lâu hơn. Đây là cách giải thích hợp lý, lab không tách riêng cơ chế để kiểm chứng.

**GPS nhiễu cho thấy giá trị thật của hysteresis.** «Đổi ngay» đổi 2,55 lần mỗi ca, trong đó 1,52 lần là quay lại chỗ đã bỏ, và khách đi thêm 40,5 m. Hysteresis đổi 0,06 lần và flip làm tròn còn 0,00. Đây là bằng chứng yếu: một chính sách gần như không đổi gì thì tất nhiên không đổi qua đổi lại. Bằng chứng tốt hơn nằm ở phép quét dưới đây, nhưng phép quét đó chỉ chạy ở tình huống tài xế trễ, không chạy ở GPS nhiễu. Có một con số nhìn đẹp nhưng nên đọc cẩn thận: `meet_gap_s` của «đổi ngay» giảm 120,6 s ở tình huống này. Nó đi kèm hai lần đổi mỗi ca trở lên và 40,5 m đi bộ thêm; lab không tách vì sao `meet_gap_s` giảm mạnh như vậy, nên không nên dùng nó như lợi ích.

**Một khoảng tin cậy «có ý nghĩa» của hysteresis không đáng tin.** Duy nhất một ô của hysteresis loại được 0: `walk_m` −10,4 m [−19,3; −1,6] ở tình huống khách đi chậm. Chính sách đó đổi 0,06 lần mỗi ca, khoảng 0,06 × 160 ≈ 10 lần đổi trong cả 160 ca (ước tính từ số làm tròn). Mười sự kiện không đủ chống đỡ một kết luận. Hơn nữa, hysteresis có 4 tình huống thật sự đổi × 4 chỉ số = 16 khoảng tin cậy ở bảng trên (tình huống đối chứng toàn 0); khi mỗi khoảng là 95%, kỳ vọng gần một khoảng loại 0 chỉ vì may rủi. Không nên dùng ô này làm bằng chứng chính sách có ích.

## Vì sao hysteresis đứng im: giải thích từ mô hình

Kết quả rỗng cần một giải thích, không chỉ một câu «ngưỡng quá chặt». Số liệu cho ba manh mối.

Một: cổng thứ nhất **không** là thủ phạm. Hysteresis tính lại 3,10 tới 12,86 lần mỗi ca, nên điều kiện lệch quá 30 s thỏa thường xuyên, nhất là khi tài xế trễ hay GPS nhiễu.

Hai: cổng `J` giảm ít nhất 25 s **và** ít nhất 10% là chặn chính. Lab có một lần quét ngưỡng ở tình huống tài xế trễ 120 s, bỏ cổng 10% và chỉ giữ cổng giây:

| `min_gain_s` | đổi/ca | flip/ca | `t_pick_s` | `walk_m` |
|---:|---:|---:|---:|---:|
| (mặc định, kèm cổng 10%) | 0,03 | 0,00 | 405,78 | 65,95 |
| 60 | 0,29 | 0,00 | 400,97 | 83,36 |
| 25 | 0,60 | 0,01 | 397,88 | 97,54 |
| 5 | 0,86 | 0,02 | 396,41 | 107,09 |
| tĩnh | 0,00 | 0,00 | 410,66 | 70,94 |
| đổi ngay | 2,12 | 0,64 | 388,16 | 129,92 |

Nguồn: `results/p04.md`, mục LOC-36, tình huống tài xế trễ 120 s, trên dữ liệu mô phỏng.

Chỉ cần bỏ cổng 10% và giữ nguyên ngưỡng 25 s, số lần đổi nhảy từ 0,03 lên 0,60 mỗi ca, trong khi flip vẫn 0,01; đổi lại khách đi bộ 97,5 m so với 70,9 m của tĩnh. Hình 2 vẽ đường đánh đổi này.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Hình 2. Đánh đổi giữa số lần đổi điểm hẹn mỗi ca và giờ cả hai có mặt, tình huống tài xế trễ 120 s. Tĩnh: 0,00 lần, 410,7 s. Có trễ mặc định: 0,03 lần, 405,8 s. Quét ngưỡng bỏ cổng 10%: ngưỡng 60 s 0,29 lần, 401,0 s; 25 s 0,60 lần, 397,9 s; 5 s 0,86 lần, 396,4 s. Đổi ngay: 2,12 lần, 388,2 s."><text x="36" y="28" class="t-b">Tài xế trễ 120 s: đổi nhiều hơn thì cả hai có mặt sớm hơn</text><path d="M100 290 L800 290" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><path d="M100 70 L100 290" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><path d="M100 290 L100 295" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><text x="100" y="310" class="t-small" text-anchor="middle">0,0</text><path d="M259.091 290 L259.091 295" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><text x="259.091" y="310" class="t-small" text-anchor="middle">0,5</text><path d="M418.182 290 L418.182 295" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><text x="418.182" y="310" class="t-small" text-anchor="middle">1,0</text><path d="M577.273 290 L577.273 295" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><text x="577.273" y="310" class="t-small" text-anchor="middle">1,5</text><path d="M736.364 290 L736.364 295" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><text x="736.364" y="310" class="t-small" text-anchor="middle">2,0</text><path d="M95 253.333 L100 253.333" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><path d="M100 253.333 L800 253.333" class="f-none s-line dash" stroke-width="0.8" stroke-linecap="round"/><text x="90" y="257.333" class="t-small" text-anchor="end">390</text><path d="M95 180 L100 180" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><path d="M100 180 L800 180" class="f-none s-line dash" stroke-width="0.8" stroke-linecap="round"/><text x="90" y="184" class="t-small" text-anchor="end">400</text><path d="M95 106.667 L100 106.667" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><path d="M100 106.667 L800 106.667" class="f-none s-line dash" stroke-width="0.8" stroke-linecap="round"/><text x="90" y="110.667" class="t-small" text-anchor="end">410</text><text x="36" y="56" class="t-small">giờ cả hai có mặt (t_pick_s, giây)</text><text x="450" y="332" class="t-small" text-anchor="middle">số lần đổi điểm hẹn mỗi ca</text><path d="M100 101.827 L109.545 137.613" class="f-none s-line dash" stroke-width="1.2" stroke-linecap="round"/><path d="M109.545 137.613 L192.273 172.887" class="f-none s-line dash" stroke-width="1.2" stroke-linecap="round"/><path d="M192.273 172.887 L290.909 195.547" class="f-none s-line dash" stroke-width="1.2" stroke-linecap="round"/><path d="M290.909 195.547 L373.636 206.327" class="f-none s-line dash" stroke-width="1.2" stroke-linecap="round"/><path d="M373.636 206.327 L774.545 266.827" class="f-none s-line dash" stroke-width="1.2" stroke-linecap="round"/><circle cx="100" cy="101.827" r="6" class="f-dead s-dead" stroke-width="1.5"/><text x="112" y="105.827" class="t-small">tĩnh: 410,7 s</text><circle cx="109.545" cy="137.613" r="6" class="f-pk s-pk" stroke-width="1.5"/><text x="121.545" y="141.613" class="t-small">có trễ mặc định: 405,8 s</text><circle cx="192.273" cy="172.887" r="6" class="f-phys s-phys" stroke-width="1.5"/><text x="204.273" y="164.887" class="t-small">ngưỡng 60 s: 401,0 s</text><circle cx="290.909" cy="195.547" r="6" class="f-phys s-phys" stroke-width="1.5"/><text x="294.909" y="183.547" class="t-small">ngưỡng 25 s: 397,9 s</text><circle cx="373.636" cy="206.327" r="6" class="f-phys s-phys" stroke-width="1.5"/><text x="377.636" y="228.327" class="t-small">ngưỡng 5 s: 396,4 s</text><circle cx="774.545" cy="266.827" r="6" class="f-warn s-warn" stroke-width="1.5"/><text x="762.545" y="254.827" class="t-small" text-anchor="end">đổi ngay: 388,2 s</text><text x="36" y="352" class="t-small">Chấm xanh ngọc: quét ngưỡng «J giảm ít nhất», bỏ cổng 10%. Đây là quét khám phá: không có kiểm định cặp,</text><text x="36" y="368" class="t-small">ngưỡng chưa được chọn bằng quy tắc đặt trước. Trên dữ liệu mô phỏng, seed 20270001…20270020, 160 cặp.</text></svg>
</div>
<figcaption>Hình 2. Tình huống tài xế trễ 120 s: số lần đổi mỗi ca (trục ngang) và giờ cả hai có mặt (trục dọc, thấp hơn là tốt). Quét ngưỡng bỏ cổng 10% (xanh ngọc) lấy được khoảng nửa quãng giữa «tĩnh» và «đổi ngay» với flip không quá 0,02 mỗi ca, và khách trả bằng quãng đi bộ 83,4 đến 107,1 m so với 70,9 m. Đây là quét khám phá, không kiểm định cặp, trên dữ liệu mô phỏng.</figcaption>
</figure>

Ba: vì sao cổng 10% nặng đến vậy? Đây là **suy luận từ cấu trúc của `J`, lab không in `J` hiện tại nên không có con số trực tiếp**. `J` cộng cả `d·T_trip`, thời gian đoạn có khách, mà đoạn đó dài 1.500–8.000 m chim bay. Số hạng này gần như không đổi khi điểm hẹn dịch vài chục mét, nên `J` hiện tại luôn lớn, và 10% của nó lớn hơn nhiều so với lợi ích mà một lần dịch điểm hẹn có thể đem lại, trong khi lợi ích đó chỉ nằm ở hai số hạng đi bộ và chạy xe. Cổng tỉ lệ so với toàn bộ `J` là cổng sai thước: nó so lợi ích cục bộ với một tổng bị chi phối bởi phần không đổi. Một cổng đúng hơn sẽ so với phần có thể thay đổi, chẳng hạn `T_walk + T_drive`. Lab không thử biến thể đó.

## Kết quả này chứng minh gì, và không chứng minh gì

Nó chứng minh một điều hẹp: **bộ ngưỡng đặt trước này, trong simulator này, không hành động**, và vì vậy không có lợi ích đo được. Nó cũng cho thấy hai điều có thể tin hơn. Một: «đổi ngay» không phải miễn phí, vì khách đi thêm hàng chục mét và có hiện tượng đổi qua đổi lại (bằng chứng ở Hình 1). Hai: ở GPS nhiễu, chính sách có cổng có 0,00 flip mỗi ca so với 1,52 của «đổi ngay», nhưng phần lớn vì nó gần như không đổi.

Nó **không** chứng minh ý tưởng điểm hẹn động vô ích. Khoảng tin cậy rộng của các ô hysteresis là «chưa biết», không phải «bằng 0». Phép quét ngưỡng gợi ý một chính sách có trễ ít chặt hơn lấy được khoảng một nửa lợi ích của «đổi ngay» (giờ có mặt 396,4 tới 401,0 s so với 410,7 s của tĩnh và 388,2 s của «đổi ngay») với gần như không đổi qua đổi lại, và khách đi bộ nhiều hơn tĩnh 12,4 đến 36,2 m. Nhưng quét đó **không có kiểm định cặp**, cổng 10% bị bỏ chứ không được thay bằng một cổng đúng hơn, ngưỡng chưa được chọn bằng quy tắc đặt trước trên dải seed chỉnh, và chỉ chạy trên một tình huống. Nên coi nó là giả thuyết cho lần sau, không phải kết quả.

Ai được lợi và ai chịu thiệt, theo thứ tự đọc từ bảng: hysteresis mặc định không lợi ai và không hại ai (đúng như một chính sách đứng im). «Đổi ngay» lợi tài xế (km rỗng ít hơn) và lợi khách ở chỗ xe tới sớm hơn khi tài xế trễ, nhưng khách trả bằng quãng đi bộ thêm và sự bất ổn của điểm hẹn. Khi khách đi chậm, bên trả là tài xế.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` (hoặc thêm `-- --fixture` để chạy nhanh trên đồ thị tổng hợp, kết quả chỉ để kiểm đường ống). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run --release -q --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc36_diem_hen_dong -- --jobs 6
```

Kết quả thật của tình huống tài xế trễ 120 s và phần quét ngưỡng, kịch bản `hcm-baseline`, 20 seed 20270001…20270020 (rút gọn từ `results/p04.md`):

```text
  tình huống: tài xế trễ 120 s
    chính sách            t_pick_s  meet_gap_s      walk_m     drive_m      đổi/ca     flip/ca  từ chối/ca tính lại/ca        xong
    tĩnh                    410.66      358.19       70.94     2047.15        0.00        0.00        0.00        0.00        1.00
    đổi ngay                388.16      277.72      129.92     1953.63        2.12        0.64        0.00       10.04        1.00
    có trễ + đồng ý         405.78      353.69       65.95     2041.39        0.03        0.00        0.01       12.79        1.00
    đổi ngay − tĩnh     t_pick_s -22.5 [-32.1, -12.9]*  meet_gap_s -80.5 [-99.6, -61.3]*  walk_m +59.0 [+40.8, +77.1]*  drive_m -93.5 [-121.5, -65.5]*
    có trễ + đồng ý − tĩnh  t_pick_s -4.9 [-12.6, +2.8]  meet_gap_s -4.5 [-11.6, +2.6]  walk_m -5.0 [-15.3, +5.3]  drive_m -5.8 [-11.6, +0.1]

  quét ngưỡng «J giảm ít nhất» của chính sách có trễ, tình huống tài xế trễ 120 s (đổi nhiều thì flip tăng, đổi ít thì không hưởng được):
    min_gain_s            t_pick_s  meet_gap_s      walk_m     drive_m      đổi/ca     flip/ca  từ chối/ca tính lại/ca        xong
    5                       396.41      309.31      107.09     1990.17        0.86        0.02        0.28        7.72        1.00
    25                      397.88      319.31       97.54     2002.37        0.60        0.01        0.24        9.10        1.00
    60                      400.97      334.28       83.36     2018.26        0.29        0.00        0.12       10.67        1.00
```

Máy đang chịu tải rất nặng khi đo, nên đừng so số giây chạy của lab giữa các lần. Số mô phỏng ở trên (giờ có mặt, quãng đường, số lần đổi) tái lập được vì chúng không phụ thuộc đồng hồ của máy. Lệnh `bash examples/toi-noi-location/verify-p04.sh` chạy lab này trên fixture với 3 seed để kiểm đường ống, không tái hiện bảng trên.

Cũng nên nhìn cột `từ chối/ca` của dòng quét: ở ngưỡng 5 s, 0,28 lần mỗi ca là khách từ chối đề nghị. Đây là chi phí thật của một chính sách đổi nhiều: mỗi lần từ chối là một lần làm phiền khách mà không thu được gì, và con số đó phụ thuộc hoàn toàn vào mô hình đồng ý giả định.

## Giới hạn của ví dụ

- **Mô hình đồng ý của khách** (`0,95 − 0,004 × mét đi thêm`) là giả định của lab. Không có dữ liệu hành vi thật, và việc khách có bị làm phiền bởi lời đề nghị hay không không nằm trong mô hình.
- **«Tài xế trễ»** là tài xế đứng yên 120 s rồi chạy bình thường; nền tảng thấy điều đó qua tốc độ ước trong cửa sổ 30 s. «Khách đi chậm» là hệ số 0,5 trên tốc độ đi bộ 1,3 m/s. Cả hai là kịch bản dựng sẵn, không phải phân bố thật.
- **8 cặp khách–tài xế mỗi seed**, 160 ca mỗi tình huống; tài xế rảnh gần nhất cách 700–4.000 m chim bay. Đây không phải các cặp mà bộ ghép của simulator tạo ra.
- **Mô phỏng riêng một cặp**, không có tranh chấp với khách khác, không có đổi tài xế (đó là việc của [bài 48](/articles/location-lech-tuyen-dinh-tuyen-lai-ghep-lai)).
- **Không có cơ chế truyền đề nghị tới app.** Lab không mô phỏng độ trễ gRPC hay hiển thị trên Flutter; một đề nghị đổi điểm hẹn thật còn phải đi qua Dispatch C# (xem [bài 46](/articles/location-goi-y-sang-dispatch-csharp-grpc-lease-idempotency)).
- **Số thời gian** chạy trên máy chịu tải rất nặng (load average khoảng 330 lúc đo). Bài này không dùng số thời gian chạy của lab, chỉ dùng số mô phỏng.
- **Ngưỡng 5 s của vòng lặp** là một chọn lựa của lab; sự kiện thật (GPS tới, tài xế dừng) có thể kích hoạt tính lại sớm hơn.

## Bài tập

**Bài 1.** Từ output ở tình huống tài xế trễ 120 s, tính tỉ lệ «lần tính lại» của hysteresis dẫn tới một lần đổi. Con số này gợi ý điều gì về cổng nào cần nới trước?

<details><summary>Lời giải bài 1</summary>
<p>0,03 / 12,79 ≈ 0,0023, cỡ 1 lần đổi trên 400 lần tính lại (số đầu vào làm tròn nên chỉ nên đọc là vài trăm). Cổng tính lại mở thường xuyên (12,79 lần mỗi ca), cổng đổi gần như luôn đóng. Phép quét ngưỡng cho thấy nơi nên nới là cổng tỉ lệ 10%: bỏ cổng này và giữ 25 s đưa số lần đổi từ 0,03 lên 0,60 mỗi ca, flip vẫn 0,01.</p>
</details>

**Bài 2.** Bạn muốn chọn ngưỡng `min_gain_s` mà không «chỉnh tới khi đẹp». Thiết kế quy trình dùng các dải seed của P04.

<details><summary>Lời giải bài 2</summary>
<p>Đặt trước quy tắc chọn (ví dụ: ngưỡng nhỏ nhất làm <code>t_pick_s</code> giảm có ý nghĩa mà flip mỗi ca không vượt 0,05), chạy quét trên dải seed chỉnh 20261201…20261210, chọn một ngưỡng theo quy tắc, rồi báo kết quả cặp trên dải đánh giá 20270001…20270020, và nếu quy tắc phải sửa sau khi thấy kết quả thì đo lại trên dải xác nhận chưa từng nhìn (20270101…20270120 như ở bài 40). Quét hiện tại của lab chạy thẳng trên dải đánh giá và bỏ kiểm định cặp nên chưa đạt các điều này.</p>
</details>

**Bài 3.** Ở tình huống khách đi chậm, «đổi ngay» làm `walk_m` giảm 22,9 m nhưng `meet_gap_s` tăng 22,5 s. Giải thích bằng ai tới trước, và nói ai chịu thiệt. Một thí nghiệm nhỏ nào sẽ kiểm tra giải thích của bạn?

<details><summary>Lời giải bài 3</summary>
<p>Khi khách chậm, tài xế là bên tới sau, nên `meet_gap_s` gần với thời gian khách đứng chờ. Dịch điểm hẹn về phía khách rút ngắn quãng đi bộ (−22,9 m) nhưng kéo dài đường của tài xế (+24,8 m), làm bên tới trước chờ lâu hơn; tài xế là bên chịu thiệt (chạy thêm), khách có lợi ở quãng đi bộ nhưng đứng chờ lâu hơn. Để kiểm tra, ghi lại ai là người tới trước ở điểm hẹn cuối và tách <code>meet_gap_s</code> theo hai nhóm; nếu giải thích đúng, tăng <code>meet_gap_s</code> sẽ nằm ở nhóm «khách tới trước».</p>
</details>

Bài trước: [bài 35](/articles/location-xep-hang-diem-don-one-to-many-ngan-sach-do-tre). Bài tiếp: [bài 37](/articles/location-hem-chung-cu-tttm-san-bay-cong-sanh).

## Tài liệu gốc

- [Hysteresis (Wikipedia)](https://en.wikipedia.org/wiki/Hysteresis): khái niệm trễ, nền của ý tưởng «chỉ đổi khi chênh đủ lớn».
- [Schmitt trigger (Wikipedia)](https://en.wikipedia.org/wiki/Schmitt_trigger): ví dụ kinh điển dùng hai ngưỡng để không nhảy qua lại khi tín hiệu nhiễu.
- [Student's t-test (Wikipedia)](https://en.wikipedia.org/wiki/Student%27s_t-test): mục kiểm định mẫu ghép cặp, nền của khoảng tin cậy trên hiệu từng seed.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho dữ liệu OSM của đồ thị HCM.
