---
slug: location-tai-xe-ranh-di-dau-dieu-huong-theo-nhu-cau
title: "Tài xế rảnh nên đi đâu: gợi ý theo nhu cầu, và ai trả cho km rỗng"
summary: "Bài 49: gợi ý di chuyển cho tài xế rảnh theo bảng nhu cầu H3, chống bầy đàn, chỉ gợi ý không ép. Trên dữ liệu mô phỏng khách chờ đón ít hơn 36,1 s nhưng km rỗng tổng tăng 1.428,9 m mỗi khách đặt: tài xế gánh chi phí."
category: system-design
level: advanced
tags: [toi-noi, location, rust, h3, reposition, demand, herding, simulation]
language: vi
---

Tài xế `DR-052` vừa trả khách xong ở một khu vắng. Hệ thống biết có một ô cách đó ba phút xe chạy đang có nhiều khách đặt hơn hẳn. Gợi ý «đi sang đó» nghe hiển nhiên, cho tới khi hai mươi tài xế rảnh khác nhận cùng một gợi ý, kéo nhau về cùng một ô, và ô vừa nóng lại thừa xe trong khi khu họ rời đi thành trống.

Hai việc phải làm đúng cùng lúc. Một là gợi ý **đúng chỗ**, tức dựa trên nhu cầu. Hai là gợi ý **không dồn một chỗ**, tức tự phân bổ. Và có một câu hỏi mà bài này không né: tài xế chạy theo gợi ý là chạy không, tốn xăng và thời gian. Khách chờ ít hơn, nhưng **ai trả cho những km đó**?

Đây là bài 49 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng chỉ mục H3 của [bài 08](/articles/location-h3-geohash-chi-muc-luoi-k-ring) lẫn [bài 44](/articles/location-cat-tia-ung-vien-h3-ma-tran-thua), và simulator của [bài 30](/articles/location-tong-hop-mo-phong-su-kien-roi-rac-seed). **Mọi số trong bài là trên dữ liệu mô phỏng**: đồ thị HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors), 20 seed đánh giá 20261106…20261125, so sánh có cặp theo seed như đã giải thích ở [bài 41](/articles/location-gan-nhat-sai-o-dau-do-bang-mo-phong).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bộ «thường» (120 xe, 3,5 chuyến mỗi phút), tài xế rảnh nghe theo gợi ý theo nhu cầu với xác suất 70%. Thời gian chờ đón trung bình giảm 36,1 s. Theo bạn <b>km rỗng tổng mỗi khách đặt</b> (km chạy không của chuyến hoàn thành, của chuyến hủy sau khi gán, và quãng chạy theo gợi ý) giảm, tăng hay không đổi?</p>
<details><summary>Xem đáp án</summary><p>Tăng, và tăng mạnh: <b>+1.428,9 m</b> mỗi khách đặt (KTC 95% [1.285,5; 1.572,3], tăng ở cả 20/20 seed). Quãng chạy theo gợi ý là 1.595,6 m mỗi khách, lớn hơn nhiều phần km đón tiết kiệm được. Khách được lợi, tài xế chịu chi phí. So với baseline là Hungarian ghép ngay, không gợi ý.</p></details>
</div>

## Bảng nhu cầu cố định theo ô H3

Gợi ý theo nhu cầu cần một bản đồ nhu cầu. Lab dựng một **bảng cố định**: H3 độ phân giải 8 (cạnh ô khoảng 531 m), mỗi ô giữ số yêu cầu kỳ vọng mỗi giờ, lấy từ 10 seed **chỉnh** 9001…9010, không dùng seed đánh giá. Kết quả là 326 ô có nhu cầu (có đường ô tô gần tâm ô), tổng 217,9 yêu cầu mỗi giờ, và 10 ô đầu chiếm 18,9%.

```text
  326 ô có nhu cầu (có đường ô tô gần tâm ô); tổng 217.9 yêu cầu/giờ; 10 ô đầu chiếm 18.9%; cạnh ô H3 độ phân giải 8 ≈ 531 m
  độ khớp của bảng với số khách thực theo ô trên 10 seed đánh giá đầu: Pearson trung bình 0.726 (từng seed 0.684…0.797). Bảng là MỘT DỰ BÁO LÝ TƯỞNG vì cùng phân bố sinh khách với simulator; dự báo thật (LOC-55) sẽ kém hơn.
```

Hai điều cần ghi nhớ về bảng này. **Nó là một dự báo lý tưởng**, vì dựng từ cùng phân bố sinh khách với simulator; hệ số Pearson giữa bảng và số khách thực theo ô chỉ 0,726 ngay cả khi vậy, có lẽ vì ô nào cũng chỉ có vài khách trong một giờ nên số đếm của một seed rất nhiễu. Dự báo thật sẽ kém hơn, và bảng được thay bằng dự báo ở LOC-55, bài chưa viết của loạt. Và **nhu cầu chỉ phụ thuộc vị trí, không theo giờ trong ngày**, nên bài không nói gì về ngày thường so với cao điểm hay giờ khuya.

## Luật gợi ý: lợi kỳ vọng giảm dần theo số xe đã về ô

Để không kéo mọi người về một ô, luật không dùng hạn ngạch cứng. Nó dùng một ước lượng thời gian chờ khách của một tài xế rảnh trong ô `c`: `3600 / (λ_c / số_xe_rảnh_c)` giây, với λ là số yêu cầu mỗi giờ. Mỗi tài xế thêm vào ô làm số này tăng lên, tức lợi của ô giảm. Hàm ước lượng, cùng trần 3 giờ:

`examples/toi-noi-location/rust/crates/matcher/src/reposition.rs`

```rust
const WAIT_CAP_S: f64 = 3.0 * 3600.0;

fn wait_s(per_hour: f64, supply: f64) -> f64 {
    if per_hour <= 0.0 {
        return WAIT_CAP_S;
    }
    (3600.0 * supply.max(1.0) / per_hour).min(WAIT_CAP_S)
```

Gọi mỗi 120 giây, xét 40 ô nhu cầu cao nhất, tài xế chỉ đi nếu ETA tới ô đích không quá 420 s và lợi ròng đạt lề 60 s. Phần quyết định:

`examples/toi-noi-location/rust/crates/matcher/src/reposition.rs`

```rust
            let mut best: Option<(f64, usize)> = None; // (lợi ròng, chỉ số trong `order`)
            match self.cfg.mode {
                Mode::HerdToTop => {
                    let k = 0;
                    let e = eta.get(i, k);
                    if Some(order[k]) != here
                        && e < FORBIDDEN / 2
                        && e as f64 / 1000.0 <= self.cfg.max_move_s
                    {
                        best = Some((f64::MAX, k));
                    }
                }
                Mode::Marginal => {
                    for (k, &c) in order.iter().enumerate() {
                        if Some(c) == here {
                            continue;
                        }
                        let e = eta.get(i, k);
                        if e >= FORBIDDEN / 2 {
                            continue;
                        }
                        let travel_s = e as f64 / 1000.0;
                        if travel_s > self.cfg.max_move_s {
                            continue;
                        }
                        let wait_there = wait_s(self.table.cells[c].per_hour, supply[c] + 1.0);
                        let net = wait_here - wait_there - travel_s;
                        if net >= self.cfg.margin_s && best.is_none_or(|(b, _)| net > b) {
                            best = Some((net, k));
                        }
                    }
                }
            }
```

Có ba chi tiết trung thực cần nêu. Một, `wait_s` ở đây là **thước xếp hạng**, không phải ETA thật: các giá trị lên hàng nghìn giây chỉ để so ô này với ô kia. Hai, tài xế đứng ngoài mọi ô trong bảng được coi là chờ tối đa 3 giờ (`WAIT_CAP_S`), nên luôn có động cơ đi. Ba, `supply[c] + 1.0` ở dòng `wait_there` là chỗ chống bầy đàn: lợi của ô đích được tính **sau khi thêm chính tài xế này**, và các tài xế đã được gợi ý về ô trong 420 s gần nhất vẫn được đếm là «đang về».

Điểm đích là điểm trên đường ô tô gần tâm ô, không xét chỗ đỗ hợp lệ; chưa có kiểm tra về việc có chỗ dừng đợi khách tại đó.

## Thử bầy đàn: mọi xe về ô nóng nhất

Muốn biết chống bầy đàn có đáng không, lab có sẵn bản đối chứng `HerdToTop`: mọi tài xế trong tầm với (ETA không quá 420 s) đều được gợi ý về ô nhu cầu cao nhất. Đo bằng chỉ số Herfindahl (HHI) của đích đến, trong đó 1 nghĩa là mọi xe về một ô, và số xe lớn nhất về cùng ô trong một lần gọi:

```text
  cấu hình                                       HHI đích đến phần gợi ý về ô nhiều nhất số xe về cùng ô/lần gọi (max)
  gợi ý theo nhu cầu, nghe 70%                          0.033                   6.4%                  5
  bầy đàn: mọi xe về ô nóng nhất, nghe 70%              1.000                 100.0%                 33
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 410" role="img" aria-label="Hình 1. Ai được gì, ai trả gì ở bộ thường (nghe 70%, trên dữ liệu mô phỏng). Ba hàng trên là phần lợi: khách chờ ít hơn, km đón ngắn hơn, nhiều chuyến hoàn thành hơn. Hàng dưới là chi phí: km rỗng tổng mỗi khách đặt tăng 1.428,9 m."><rect x="10" y="10" width="860" height="390" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="30" y="38" class="t-b">Trên dữ liệu mô phỏng, bộ «thường» (120 xe, 3,5 chuyến/phút), gợi ý theo nhu cầu, tài xế nghe 70%</text><text x="30" y="58" class="t-small">hiệu = có gợi ý − không gợi ý, trung bình 20 seed có cặp, vạch = KTC 95%, mỗi hàng một thang riêng</text><text x="30" y="109">chờ đón: pickup_eta_s (giây)</text><path d="M330 127 L640 127" class="f-none s-muted" stroke-width="1.5"/><path d="M330.0 127 L330.0 132" class="f-none s-muted" stroke-width="1.5"/><text x="330.0" y="146" class="t-small" text-anchor="middle">−50</text><path d="M588.3 127 L588.3 132" class="f-none s-muted" stroke-width="1.5"/><text x="588.3" y="146" class="t-small" text-anchor="middle">0</text><path d="M640.0 127 L640.0 132" class="f-none s-muted" stroke-width="1.5"/><text x="640.0" y="146" class="t-small" text-anchor="middle">10</text><path d="M588.3 87 L588.3 127" class="f-none s-muted dash" stroke-width="1.5"/><path d="M376.0 105 L427.65 105" class="f-none s-muted" stroke-width="2"/><path d="M376.0 100 L376.0 110" class="f-none s-muted" stroke-width="2"/><path d="M427.65 100 L427.65 110" class="f-none s-muted" stroke-width="2"/><circle cx="401.8" cy="105" r="7" class="f-phys s-muted" stroke-width="1.5"/><text x="660" y="109" class="t-b">−36,1 [−41,1; −31,1]</text><text x="30" y="179">km rỗng lúc đón: deadhead_m (mét)</text><path d="M330 197 L640 197" class="f-none s-muted" stroke-width="1.5"/><path d="M330.0 197 L330.0 202" class="f-none s-muted" stroke-width="1.5"/><text x="330.0" y="216" class="t-small" text-anchor="middle">−400</text><path d="M578.0 197 L578.0 202" class="f-none s-muted" stroke-width="1.5"/><text x="578.0" y="216" class="t-small" text-anchor="middle">0</text><path d="M640.0 197 L640.0 202" class="f-none s-muted" stroke-width="1.5"/><text x="640.0" y="216" class="t-small" text-anchor="middle">100</text><path d="M578.0 157 L578.0 197" class="f-none s-muted dash" stroke-width="1.5"/><path d="M389.272 175 L450.032 175" class="f-none s-muted" stroke-width="2"/><path d="M389.272 170 L389.272 180" class="f-none s-muted" stroke-width="2"/><path d="M450.032 170 L450.032 180" class="f-none s-muted" stroke-width="2"/><circle cx="419.7" cy="175" r="7" class="f-phys s-muted" stroke-width="1.5"/><text x="660" y="179" class="t-b">−255,4 [−304,4; −206,4]</text><text x="30" y="249">chuyến hoàn thành thêm mỗi seed</text><path d="M330 267 L640 267" class="f-none s-muted" stroke-width="1.5"/><path d="M330.0 267 L330.0 272" class="f-none s-muted" stroke-width="1.5"/><text x="330.0" y="286" class="t-small" text-anchor="middle">−2</text><path d="M381.7 267 L381.7 272" class="f-none s-muted" stroke-width="1.5"/><text x="381.7" y="286" class="t-small" text-anchor="middle">0</text><path d="M640.0 267 L640.0 272" class="f-none s-muted" stroke-width="1.5"/><text x="640.0" y="286" class="t-small" text-anchor="middle">10</text><path d="M381.7 227 L381.7 267" class="f-none s-muted dash" stroke-width="1.5"/><path d="M497.9 245 L598.7 245" class="f-none s-muted" stroke-width="2"/><path d="M497.9 240 L497.9 250" class="f-none s-muted" stroke-width="2"/><path d="M598.7 240 L598.7 250" class="f-none s-muted" stroke-width="2"/><circle cx="549.6" cy="245" r="7" class="f-phys s-muted" stroke-width="1.5"/><text x="660" y="249" class="t-b">+6,5 [4,5; 8,4]</text><text x="30" y="319">km rỗng TỔNG mỗi khách đặt (mét)</text><path d="M330 337 L640 337" class="f-none s-muted" stroke-width="1.5"/><path d="M330.0 337 L330.0 342" class="f-none s-muted" stroke-width="1.5"/><text x="330.0" y="356" class="t-small" text-anchor="middle">−200</text><path d="M361.0 337 L361.0 342" class="f-none s-muted" stroke-width="1.5"/><text x="361.0" y="356" class="t-small" text-anchor="middle">0</text><path d="M640.0 337 L640.0 342" class="f-none s-muted" stroke-width="1.5"/><text x="640.0" y="356" class="t-small" text-anchor="middle">1.800</text><path d="M361.0 297 L361.0 337" class="f-none s-muted dash" stroke-width="1.5"/><path d="M560.3 315 L604.7 315" class="f-none s-muted" stroke-width="2"/><path d="M560.3 310 L560.3 320" class="f-none s-muted" stroke-width="2"/><path d="M604.7 310 L604.7 320" class="f-none s-muted" stroke-width="2"/><circle cx="582.5" cy="315" r="7" class="f-warn s-muted" stroke-width="1.5"/><text x="660" y="319" class="t-b">+1.428,9 [1.285,5; 1.572,3]</text><text x="30" y="392" class="t-small">Xanh ngọc: phần lợi (khách chờ ít hơn, nhiều chuyến hơn). Vàng cam: chi phí, cộng cả quãng tài xế chạy theo gợi ý.</text></svg>
</div>
<figcaption>Hình 1. Ai được gì, ai trả gì ở bộ thường (nghe 70%, trên dữ liệu mô phỏng). Ba hàng trên là phần lợi: khách chờ ít hơn, km đón ngắn hơn, nhiều chuyến hoàn thành hơn. Hàng dưới là chi phí: km rỗng tổng mỗi khách đặt tăng 1.428,9 m.</figcaption>
</figure>

Luật chống bầy đàn có HHI 0,033, tức tương đương khoảng 30 ô «có hiệu lực» trong 40 ô ứng viên (1/HHI), và nhiều nhất 5 xe về cùng ô một lần. Bản đối chứng có HHI 1,000 và tới 33 xe về cùng ô. Hệ quả đo được: ở «thường», bầy đàn làm `pickup_eta_s` +1,4 s (không phân biệt được) và km rỗng tổng +272,6 m; ở «dày xe» (240 xe), nó làm `pickup_eta_s` **tăng +15,5 s** (KTC [12,5; 18,5]) và `deadhead_m` +147,2 m, tức xấu hơn rõ so với không gợi ý gì.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 360" role="img" aria-label="Hình 2. Số xe lớn nhất về cùng một ô trong một lần gọi. Luật chống bầy đàn giữ ở 5 xe (7 ở dày xe); bản đối chứng dồn 33 xe (59 ở dày xe). Dự báo sai (σ = 1) lên 18 xe nhưng vẫn không tới mức bầy đàn."><rect x="10" y="10" width="860" height="340" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="30" y="38" class="t-b">Bầy đàn: số xe lớn nhất được gợi ý về cùng một ô trong một lần gọi (cột max của lab)</text><text x="30" y="58" class="t-small">Trên dữ liệu mô phỏng, nghe 70%, 20 seed. Chữ nhỏ bên phải: HHI của đích đến (1 = mọi xe về một ô).</text><text x="30" y="105">«thường»: luật chống bầy đàn</text><rect x="330" y="89" width="30.0" height="22" rx="4" class="f-pk" stroke-width="1.5"/><text x="368.0" y="105" class="t-b">5</text><text x="790" y="105" class="t-small" text-anchor="middle">HHI 0,033</text><text x="30" y="141">«thường»: dự báo nhiễu σ = 1</text><rect x="330" y="125" width="108.0" height="22" rx="4" class="f-pk" stroke-width="1.5"/><text x="446.0" y="141" class="t-b">18</text><text x="790" y="141" class="t-small" text-anchor="middle">HHI 0,049</text><text x="30" y="177">«thường»: bảng đều, không dự báo</text><rect x="330" y="161" width="6.0" height="22" rx="4" class="f-pk" stroke-width="1.5"/><text x="344.0" y="177" class="t-b">1</text><text x="790" y="177" class="t-small" text-anchor="middle">HHI 0,133</text><text x="30" y="213">«thường»: mọi xe về ô nóng nhất</text><rect x="330" y="197" width="198.0" height="22" rx="4" class="f-warn" stroke-width="1.5"/><text x="536.0" y="213" class="t-b">33</text><text x="790" y="213" class="t-small" text-anchor="middle">HHI 1,000</text><text x="30" y="249">«dày xe» 240 xe: luật chống bầy đàn</text><rect x="330" y="233" width="42.0" height="22" rx="4" class="f-pk" stroke-width="1.5"/><text x="380.0" y="249" class="t-b">7</text><text x="790" y="249" class="t-small" text-anchor="middle">HHI 0,030</text><text x="30" y="285">«dày xe» 240 xe: mọi xe về ô nóng nhất</text><rect x="330" y="269" width="354.0" height="22" rx="4" class="f-warn" stroke-width="1.5"/><text x="692.0" y="285" class="t-b">59</text><text x="790" y="285" class="t-small" text-anchor="middle">HHI 1,000</text></svg>
</div>
<figcaption>Hình 2. Số xe lớn nhất về cùng một ô trong một lần gọi. Luật chống bầy đàn giữ ở 5 xe (7 ở dày xe); bản đối chứng dồn 33 xe (59 ở dày xe). Dự báo sai (σ = 1) lên 18 xe nhưng vẫn không tới mức bầy đàn.</figcaption>
</figure>

## Chỉ gợi ý, không ép

Tài xế là người, không phải robot: họ có thể từ chối. Phía simulator, mỗi gợi ý rút một số ngẫu nhiên; chỉ khi số đó nhỏ hơn xác suất nghe theo của kịch bản thì tài xế mới đi.

`examples/toi-noi-location/rust/crates/simulator/src/sim.rs`

```rust
            self.extras.reposition_suggested += 1;
            let u = self.rng_repo.f64();
            if u >= self.reposition_accept_prob {
                continue;
            }
```

Quãng chạy theo gợi ý được tính vào km rỗng. Thử nghe theo 30%, 70% (mặc định) và 100%, trên dữ liệu mô phỏng, bộ «thường», hiệu so với Hungarian ghép ngay không gợi ý, KTC 95% trong ngoặc, `*` khi không chứa 0:

| Tài xế nghe theo | Chạy theo gợi ý mỗi khách (m) | `pickup_eta_s` TB (s) | `deadhead_m` TB (m) | km rỗng tổng mỗi khách (m) | Hoàn thành |
|---|---:|---|---|---|---|
| 30% | 971,0 | −28,7 [−33,7; −23,7]* | −215,9 [−265,3; −166,5]* | +825,6 [730,7; 920,6]* | +4,8 [3,2; 6,4]* |
| 70% | 1.595,6 | −36,1 [−41,1; −31,1]* | −255,4 [−304,4; −206,4]* | +1.428,9 [1.285,5; 1.572,3]* | +6,5 [4,5; 8,4]* |
| 100% | 1.977,9 | −45,2 [−51,1; −39,2]* | −317,9 [−371,7; −264,2]* | +1.762,6 [1.585,3; 1.940,0]* | +7,2 [5,9; 8,5]* |

Lợi và chi phí cùng tăng theo mức nghe theo. Chia thô km rỗng tổng thêm cho số giây chờ đón giảm được cho khoảng 29 m, 40 m và 39 m mỗi giây ở ba mức (tỉ số thô, hai mẫu số khác nhau: chuyến hoàn thành và khách đặt). Hai kiểm tra nữa trên cùng bộ, nghe 70%: với bảng nhiễu σ = 1 (dự báo sai có chủ đích) `pickup_eta_s` vẫn giảm 22,0 s, km rỗng tổng +1.713,0 m; với **bảng đều** (không có thông tin nhu cầu), `pickup_eta_s` +1,6 s [−0,1; 3,4] không phân biệt được với 0, còn km rỗng tổng +115,4 m, và chỉ có 15,1 gợi ý mỗi seed. Dự báo sai vẫn có lợi, nên lab chưa tách được phần lợi nào đến từ thông tin nhu cầu, phần nào đến từ việc xe rảnh được dàn về vùng đông nói chung.

## Ai trả cho km rỗng?

Đây là bảng quan trọng nhất của bài, ba bộ giả định, nghe 70%, trên dữ liệu mô phỏng:

| Bộ giả định | Gợi ý mỗi seed | `pickup_eta_s` TB (s) | km rỗng tổng mỗi khách (m) |
|---|---:|---|---|
| «thường»: 120 xe | 267,8 | −36,1 [−41,1; −31,1]* | +1.428,9 [1.285,5; 1.572,3]* |
| «thưa xe»: 60 xe | 88,1 | −12,5 [−27,8; 2,7] | +385,8 [264,0; 507,7]* |
| «dày xe»: 240 xe | 450,4 | −16,8 [−20,9; −12,7]* | +3.139,7 [2.962,6; 3.316,8]* |

Xe càng nhiều thì càng nhiều gợi ý và càng nhiều quãng chạy không. Lợi cho khách thì không đi theo: lớn nhất ở «thường», ở «dày xe» khách chờ ít hơn 16,8 s trong khi km rỗng tổng mỗi khách tăng 3.139,7 m. Ở «thưa xe», lợi cho khách không phân biệt được với 0, còn chi phí vẫn có thật (+385,8 m).

Ai trả? Quy về cả đội ở bộ «thường» (tính từ các số trên, mỗi seed là một giờ mô phỏng): 228,4 khách × 1.428,9 m là khoảng 326 km rỗng thêm, tức 2,7 km mỗi xe. Đổi lại, 204,7 chuyến hoàn thành × 36,1 s là khoảng 2,05 giờ khách đỡ chờ, cộng 6,5 chuyến thêm cho cả đội 120 xe. Phần tăng thêm là 1.595,6 m chạy theo gợi ý trừ 166,7 m km đón tiết kiệm được mỗi khách đặt. Trong **mô hình này tài xế không có phần thưởng nào cho quãng chạy theo gợi ý**: nhiên liệu, hao mòn và thời gian không chở khách do tài xế gánh, phần lớn lợi do khách hưởng. Lab không có đơn vị chung để nói 326 km có đáng 2,05 giờ không; đó là quyết định chính sách, và nó phải đi cùng câu hỏi ai trả tiền. Nói «gợi ý theo nhu cầu giảm thời gian chờ» mà bỏ phần này là kể nửa câu chuyện.

Theo vùng (khoảng cách từ ghim của khách tới một tâm quy chiếu cố định của lab), «thường», nghe 70%:

| Vùng | Hoàn thành, không gợi ý → có gợi ý | `wait_s` TB hiệu (s) |
|---|---|---|
| Trung tâm (dưới 3 km) | 0,878 → 0,920* | −68,6 [−78,9; −58,2]* |
| Vành trong (3–6 km) | 0,875 → 0,911* | −41,3 [−49,1; −33,5]* |
| Ngoại vi (từ 6 km) | 0,854 → 0,859 | +0,1 [−7,9; 8,1] |

Lợi dồn về trung tâm và vành trong, nơi nhu cầu dày, còn **ngoại vi gần như không đổi**: không bị làm xấu đi, nhưng cũng không được giúp. Ở «dày xe» có dấu hiệu ngoại vi xấu đi: hoàn thành 0,938 → 0,926 (KTC không chứa 0) và `deadhead_m` +50,8 m [4,9; 96,8]; hai dấu sao sát 0 trong một bảng lớn, nên cần kiểm thêm. Một chính sách «điều hướng theo nhu cầu» đi theo nơi đông khách thì có cơ chế để rời khu ít khách, và đó là lý do phải giữ chỉ số theo vùng. Điều này đụng thẳng tới quy tắc của loạt: không tối ưu cho một vùng bằng cách bỏ rơi vùng khác.

Vài hướng chia sẻ chi phí có thể thử, **chưa chạy** trong lab: thưởng theo km chạy theo gợi ý; giới hạn số gợi ý mỗi tài xế mỗi giờ; nâng lề lợi ròng (hiện cố định 60 s) hoặc hạ trần ETA (420 s) để chỉ gợi ý khi chắc có lợi. Các tham số `margin_s`, `max_move_s`, chu kỳ 120 s và số ô xét 40 chưa được quét; kết quả ở trên chỉ là một điểm trong không gian đó.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin`; `--fixture` chạy offline trên đồ thị tổng hợp. Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc49_tai_xe_ranh_di_dau
```

Tham số: `--seeds N` (mặc định 20), `--jobs N`, `--fixture`. Lab chạy khoảng 18 phút khi máy chịu tải nặng (load average trên 400); số đếm, tỉ lệ và hiệu có cặp tái lập được, thời gian chạy thì không. Output cho bộ «thường» (nguyên văn từ `results/p05.md`):

```text
  cấu hình                                     gợi ý/seed  nghe/seed chạy theo gợi ý/khách (m)      pickup_eta_s TB            wait_s TB          deadhead_m TB km rỗng TỔNG/khách (m)       hoàn thành
  gợi ý theo nhu cầu, nghe 70%                      267.8      185.5             1595.6 -36.1 [-41.1, -31.1]* -36.1 [-41.1, -31.1]* -255.4 [-304.4, -206.4]* +1428.9 [1285.5, 1572.3]* +6.5 [4.5, 8.4]*
  bầy đàn: mọi xe về ô nóng nhất, nghe 70%           40.4       28.6              248.3    +1.4 [-4.0, 6.7]     +1.4 [-4.0, 6.7]    +34.2 [-18.9, 87.2]  +272.6 [191.7, 353.5]* -0.6 [-1.7, 0.5] 
```

`bash examples/toi-noi-location/verify-p05.sh` chạy lại lab với `--fixture --seeds 2`.

## Giới hạn của ví dụ

- **Dự báo lý tưởng.** Bảng nhu cầu cùng phân bố với simulator, và Pearson chỉ 0,726 dù vậy. Dự báo thật kém hơn; bài này chưa tích hợp dự báo của LOC-55.
- **Chưa chạy hết các bộ giả định.** «Cao điểm» không chạy cho bài này; nghe 30%, 100%, bảng nhiễu σ = 1 và bảng đều chỉ chạy ở «thường»; «thưa xe» và «dày xe» chỉ có luật chống bầy đàn và bản bầy đàn ở nghe 70%.
- **Tài xế nghe theo với xác suất cố định, độc lập**; nhu cầu không theo giờ; tài xế không có phần thưởng, không rong ruổi tự do, không rời ca.
- **Đích gợi ý là điểm trên đường gần tâm ô**, không xét chỗ đỗ hợp lệ hay tắc đường theo giờ.
- **Chưa quét tham số** của luật (lề, trần ETA, chu kỳ, số ô xét).

## Bài tập

**Bài 1.** Ô A có λ = 4,4 yêu cầu mỗi giờ. Tính ước lượng thời gian chờ `wait_s` khi ô có 1, 2 và 3 xe rảnh. Một tài xế đang ở ô B (λ = 2,0, có 3 xe rảnh kể cả anh ta) cân nhắc đi sang A (đang có 1 xe) với ETA 300 s: lợi ròng là bao nhiêu, và ràng buộc nào chặn gợi ý trước khi lề 60 s kịp chặn?

<details><summary>Lời giải bài 1</summary>
<p>3600 × s / 4,4 cho s = 1, 2, 3 là 818,2 s, 1.636,4 s và 2.454,5 s. Ở B, <code>wait_here</code> = 3600 × 3 / 2,0 = 5.400 s. Ở A sau khi thêm anh ta, supply = 2 nên <code>wait_there</code> = 1.636,4 s. Lợi ròng = 5.400 − 1.636,4 − 300 = 3.463,6 s, lớn hơn lề 60 s nhiều, nên đi. Lề 60 s chỉ chặn khi ETA đạt khoảng 3.703 s; nhưng trần <code>max_move_s</code> = 420 s chặn từ xa trước đó. Trong luật này trần ETA mới là ràng buộc thực tế, không phải lề. (Tính tay từ công thức, không phải số đo của lab.)</p>
</details>

**Bài 2.** `deadhead_m` trung bình giảm 255,4 m mà km rỗng tổng mỗi khách đặt lại tăng 1.428,9 m. Giải thích vì sao hai số không mâu thuẫn.

<details><summary>Lời giải bài 2</summary>
<p><code>deadhead_m</code> chỉ đo quãng xe chạy không tới điểm đón, trên chuyến hoàn thành. Km rỗng tổng cộng thêm quãng chạy theo gợi ý (1.595,6 m mỗi khách đặt) và quãng chạy không của chuyến hủy sau khi gán, rồi chia cho số khách đặt. Quãng đón ngắn đi nhưng quãng di chuyển dọn chỗ lớn hơn nhiều, nên tổng tăng. Đây là lý do bài báo cáo cả hai, không chọn một.</p>
</details>

**Bài 3.** Đề xuất một cách chia sẻ chi phí km rỗng cho tài xế và hai chỉ số cần đo thêm để biết nó có làm mất lợi của khách không.

<details><summary>Lời giải bài 3</summary>
<p>Gợi ý, chưa thử trong lab: thưởng cho tài xế theo số km chạy theo gợi ý (bù một phần nhiên liệu), hoặc giới hạn số gợi ý mỗi giờ. Chỉ số cần đo thêm: (1) `pickup_eta_s` và hoàn thành theo vùng, để xem lợi của khách còn bao nhiêu khi gợi ý ít đi; (2) km rỗng tổng mỗi khách đặt và phần thưởng đã chi, để thấy phần thưởng có lớn hơn lợi không. Cần chạy lại lab với từng mức giới hạn, và chọn tham số trên dải seed chỉnh, báo cáo trên dải seed đánh giá.</p>
</details>

Bài trước: [bài 48](/articles/location-lech-tuyen-dinh-tuyen-lai-ghep-lai). Bài tiếp: [bài 50](/articles/location-nhieu-diem-dung-chen-2-opt-doi-chieu-ortools).

## Tài liệu gốc

- [H3: tài liệu chính thức](https://h3geo.org/docs/): chỉ mục lưới lục giác, độ phân giải 8 dùng cho bảng nhu cầu.
- [Herfindahl–Hirschman index (Wikipedia)](https://en.wikipedia.org/wiki/Herfindahl%E2%80%93Hirschman_index): chỉ số tập trung dùng đo mức bầy đàn của đích đến.
