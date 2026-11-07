---
slug: location-lech-tuyen-dinh-tuyen-lai-ghep-lai
title: "Lệch tuyến, định tuyến lại và ghép lại: khi nào đáng làm phiền cả khách lẫn tài xế"
summary: "Bài 48: phát hiện lệch tuyến từ bám đường bằng hysteresis, định tuyến lại phía server, ngân sách thông báo và biên chỉ ghép lại khi lợi đủ lớn. Lab độc lập vì simulator chưa có mô hình lệch tuyến, nên không có số toàn thành phố."
category: system-design
level: advanced
tags: [toi-noi, location, rust, map-matching, off-route, rerouting, hysteresis, dispatch]
language: vi
---

Tài xế `DR-014` đang chạy tới điểm đón theo tuyến đã hứa với khách, rồi rẽ nhầm vào một nhánh đường khác. Hai việc xấu có thể xảy ra, và chúng ngược nhau. Nếu hệ thống không phát hiện, khách nhìn chấm xe trên bản đồ đi xa dần điểm đón mà ETA vẫn ghi «3 phút». Nếu hệ thống phát hiện quá nhạy, một điểm GPS nhảy sang đường song song là đủ để báo lệch tuyến, tính lại tuyến, đổi ETA và có khi đổi luôn tài xế, và cả khách lẫn tài xế bị làm phiền bằng những thông báo vô nghĩa.

Bài này đi qua bốn việc: phát hiện lệch tuyến từ kết quả bám đường, định tuyến lại, thông báo cho hai bên, và quyết định có đáng đổi tài xế hay không. Điều cần nói ngay từ đầu: **đây là một thí nghiệm độc lập, không cắm vào simulator**, vì simulator của [bài 30](/articles/location-tong-hop-mo-phong-su-kien-roi-rac-seed) không có mô hình lệch tuyến. Vết GPS, nhiễu, đoạn lệch, xác suất tài xế rảnh và nhiễu ETA đều do lab sinh ra. Kết quả vì thế không cho ra con số «km rỗng toàn thành phố» nào, và bài sẽ không viết câu nào như vậy.

Đây là bài 48 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng bộ bám đường online của [bài 17](/articles/location-map-matching-online-cua-so-cau-vuot-duong-song-song) (HMM của [bài 16](/articles/location-map-matching-hmm-viterbi-rust)), engine định tuyến CH của [bài 25](/articles/location-contraction-hierarchies-tu-cai-dat-fast-paths) và hợp đồng gợi ý của [bài 46](/articles/location-goi-y-sang-dispatch-csharp-grpc-lease-idempotency). Số đo là trên dữ liệu mô phỏng, đồ thị HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Cấu hình đơn giản nhất: báo lệch tuyến ngay khi một điểm GPS đã bám đường nằm ngoài tuyến. Trong 200 chuyến mô phỏng đi <b>đúng tuyến</b> với nhiễu GPS σ = 6 m, theo bạn bao nhiêu chuyến bị báo lệch nhầm ít nhất một lần?</p>
<details><summary>Xem đáp án</summary><p>Cả 200 chuyến (100,0%), và trung bình mỗi chuyến đi đúng đổi trạng thái giữa «trên tuyến» và «ngoài tuyến» khoảng 252 lần. Nhiễu GPS làm bộ bám đường thỉnh thoảng gán điểm vào cạnh lân cận, và cấu hình ngây thơ coi mỗi lần như vậy là một lần lệch. Cấu hình lab chọn (cần 3 điểm liên tiếp, cách tuyến ít nhất 30 m) báo nhầm 2,5% số chuyến.</p></details>
</div>

## Thí nghiệm độc lập: cái được sinh và cái không

Lab sinh vết GPS tổng hợp có nhiễu (σ = 6 m, tự tương quan τ = 8 s) trên tuyến đón thật của đồ thị HCM, chạy qua bộ bám đường HMM online với cấu hình mặc định của bài 16, lấy mẫu 1 giây. Có ba loại ca:

- **Ca đi đúng tuyến**, để đo báo nhầm.
- **Ca đi dọc đường có đường song song sát bên**, lấy từ danh mục 2.162 cặp đường song song của đồ thị: chỗ bám đường dễ gán nhầm nhất.
- **Ca lệch tuyến**: «nhẹ» (rẽ nhầm rồi điều hướng lại ngay) và «nặng» (chạy sai hướng 400 đến 900 m rồi mới định tuyến lại). Chỉ những ca có đoạn lạc tuyến liên tục từ 150 m trở lên mới được tính là lệch «đáng kể» để đo tỉ lệ phát hiện.

Dải seed được tách đôi, như mọi bài tối ưu của P05: tham số chỉ được chọn trên 100 seed (48001…48100, gồm 98 ca lệch, 100 ca đúng, 40 ca song song), số báo cáo lấy trên 200 seed khác (48501…48700, gồm 199 ca lệch trong đó 146 đáng kể, 200 ca đúng, 40 ca song song). Quy tắc chọn đặt trước khi chạy: trong các cấu hình có tỉ lệ báo nhầm không quá 2% (cả ca đúng lẫn ca song song), chọn cấu hình có số mét p50 sau điểm lệch nhỏ nhất.

Vì lệch tuyến mô phỏng chỉ là «rẽ sang một cạnh khác», lab **không** có quay đầu giữa đường, cấm rẽ, hay lệch cố ý. Đọc mọi con số phía dưới với giới hạn đó.

## Hysteresis: một điểm lạc không phải là lệch tuyến

`RouteMonitor` là một máy trạng thái nhỏ, thuần (không đọc đồng hồ, không I/O) nên test được bằng chuỗi điểm viết tay. Điểm chỉ tính là «ngoài tuyến» nếu cạnh khớp không thuộc tuyến **và** nó cách polyline của tuyến quá ngưỡng `off_dist_m`; ngưỡng này nuốt nhầm lẫn giữa hai đường song song sát nhau. Muốn chuyển sang `OffRoute` phải có `k_off` điểm liên tiếp ngoài tuyến; muốn quay lại `OnRoute` phải có `k_on` điểm liên tiếp trên tuyến.

`examples/toi-noi-location/rust/crates/matcher/src/deviation.rs`

```rust
impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            k_off: 3,
            off_dist_m: 30.0,
            k_on: 3,
            cooldown_ms: 30_000,
            max_alerts: 3,
        }
    }
}
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 1. Hysteresis trên 15 điểm GPS đã bám đường. Điểm 4 là điểm lạc do nhiễu: k_off = 1 báo nhầm, k_off = 3 bỏ qua. Đoạn lệch thật từ điểm 7 đến 12 làm k_off = 3 báo ở điểm 9, và đủ ba điểm trên tuyến thì mới trở về. Sơ đồ minh họa cơ chế, không phải số đo."><rect x="10" y="10" width="860" height="320" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="30" y="38" class="t-b">Hysteresis: 15 điểm GPS đã bám đường, điểm 4 là một điểm lạc do nhiễu, điểm 7 đến 12 là đoạn lệch thật</text><text x="30" y="94">điểm bám đường</text><circle cx="200" cy="90" r="11" class="f-phys s-phys" stroke-width="1.5"/><text x="200" y="124" class="t-small" text-anchor="middle">1</text><circle cx="242" cy="90" r="11" class="f-phys s-phys" stroke-width="1.5"/><text x="242" y="124" class="t-small" text-anchor="middle">2</text><circle cx="284" cy="90" r="11" class="f-phys s-phys" stroke-width="1.5"/><text x="284" y="124" class="t-small" text-anchor="middle">3</text><circle cx="326" cy="90" r="11" class="f-warn s-warn" stroke-width="1.5"/><text x="326" y="124" class="t-small" text-anchor="middle">4</text><circle cx="368" cy="90" r="11" class="f-phys s-phys" stroke-width="1.5"/><text x="368" y="124" class="t-small" text-anchor="middle">5</text><circle cx="410" cy="90" r="11" class="f-phys s-phys" stroke-width="1.5"/><text x="410" y="124" class="t-small" text-anchor="middle">6</text><circle cx="452" cy="90" r="11" class="f-warn s-warn" stroke-width="1.5"/><text x="452" y="124" class="t-small" text-anchor="middle">7</text><circle cx="494" cy="90" r="11" class="f-warn s-warn" stroke-width="1.5"/><text x="494" y="124" class="t-small" text-anchor="middle">8</text><circle cx="536" cy="90" r="11" class="f-warn s-warn" stroke-width="1.5"/><text x="536" y="124" class="t-small" text-anchor="middle">9</text><circle cx="578" cy="90" r="11" class="f-warn s-warn" stroke-width="1.5"/><text x="578" y="124" class="t-small" text-anchor="middle">10</text><circle cx="620" cy="90" r="11" class="f-warn s-warn" stroke-width="1.5"/><text x="620" y="124" class="t-small" text-anchor="middle">11</text><circle cx="662" cy="90" r="11" class="f-warn s-warn" stroke-width="1.5"/><text x="662" y="124" class="t-small" text-anchor="middle">12</text><circle cx="704" cy="90" r="11" class="f-phys s-phys" stroke-width="1.5"/><text x="704" y="124" class="t-small" text-anchor="middle">13</text><circle cx="746" cy="90" r="11" class="f-phys s-phys" stroke-width="1.5"/><text x="746" y="124" class="t-small" text-anchor="middle">14</text><circle cx="788" cy="90" r="11" class="f-phys s-phys" stroke-width="1.5"/><text x="788" y="124" class="t-small" text-anchor="middle">15</text><text x="30" y="182">k_off = 1 (ngây thơ)</text><rect x="308" y="164" width="36" height="26" rx="5" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="326" y="182" class="t-small" text-anchor="middle">báo</text><rect x="434" y="164" width="36" height="26" rx="5" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="452" y="182" class="t-small" text-anchor="middle">báo</text><text x="30" y="252">k_off = 3 (đã chọn)</text><rect x="518" y="234" width="36" height="26" rx="5" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="536" y="252" class="t-small" text-anchor="middle">báo</text><rect x="758" y="234" width="60" height="26" rx="5" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="788" y="252" class="t-small" text-anchor="middle">về tuyến</text><path d="M438 208 L550 208" class="f-none s-warn" stroke-width="2"/><text x="494.0" y="224" class="t-small" text-anchor="middle">3 điểm liên tiếp</text><text x="30" y="296" class="t-small">Điểm 4: k_off = 1 báo nhầm, k_off = 3 bỏ qua vì chuỗi ngoài tuyến chưa đủ 3. Điểm 15: đủ k_on = 3 điểm</text><text x="30" y="314" class="t-small">liên tiếp trên tuyến (13, 14, 15) mới trở về OnRoute. Sơ đồ minh họa cơ chế, không phải số đo.</text></svg>
</div>
<figcaption>Hình 1. Hysteresis trên 15 điểm GPS đã bám đường. Điểm 4 là điểm lạc do nhiễu: k_off = 1 báo nhầm, k_off = 3 bỏ qua. Đoạn lệch thật từ điểm 7 đến 12 làm k_off = 3 báo ở điểm 9, và đủ ba điểm trên tuyến thì mới trở về. Sơ đồ minh họa cơ chế, không phải số đo.</figcaption>
</figure>

Cơ chế nằm trong `observe`. Có hai lớp chặn thêm ngoài `k_off`: cooldown giữa hai lần báo, và trần số lần báo mỗi chuyến, để việc định tuyến lại không thể lặp vô hạn.

`examples/toi-noi-location/rust/crates/matcher/src/deviation.rs`

```rust
    pub fn observe(&mut self, o: &MatchedObs) -> Option<MonitorEvent> {
        let off = self.classify(o)?;
        if off {
            if self.off_run == 0 {
                self.first_off_t_ms = o.t_ms;
            }
            self.off_run += 1;
            self.on_run = 0;
            if self.state == Track::OnRoute && self.off_run >= self.cfg.k_off {
                self.state = Track::OffRoute;
                self.toggles += 1;
                self.pending = true;
            }
            // báo bị cooldown chặn thì chờ tới khi hết cooldown mà vẫn còn ngoài tuyến; hết ngân sách thì bỏ
            if self.pending && self.state == Track::OffRoute && self.off_run >= self.cfg.k_off {
                if self.alerts >= self.cfg.max_alerts {
                    self.pending = false;
                } else if self
                    .last_alert_ms
                    .is_none_or(|t| o.now_ms - t >= self.cfg.cooldown_ms)
                {
                    self.pending = false;
                    self.alerts += 1;
                    self.last_alert_ms = Some(o.now_ms);
                    return Some(MonitorEvent::Alert {
                        now_ms: o.now_ms,
                        first_off_t_ms: self.first_off_t_ms,
                        off_points: self.off_run,
                    });
                }
            }
        } else {
            self.last_on_pos = Some(o.pos);
            self.last_on_ms = Some(o.t_ms);
            self.pending = false;
            self.on_run += 1;
            self.off_run = 0;
            if self.state == Track::OffRoute && self.on_run >= self.cfg.k_on {
                self.state = Track::OnRoute;
                self.toggles += 1;
                return Some(MonitorEvent::Recovered { now_ms: o.now_ms });
            }
        }
        None
    }
```

## Quét tham số, và vì sao «báo nhầm» phụ thuộc giả định nhiễu

Lab quét `k_off` ∈ {1, 2, 3, 5}, `off_dist_m` ∈ {15, 30, 50} và độ trễ chốt 0 hoặc 3 điểm, cộng hai cấu hình ngây thơ làm đối chứng. Cấu hình được chọn trên tập chỉnh là `k_off = 3`, ngưỡng 30 m, không trễ. Trên 200 seed báo cáo (nhiễu chuẩn, trên dữ liệu mô phỏng), một phần của bảng:

```text
  cấu hình                         phát hiện%  trễ s p50/p95   mét p50/p95/max      báo nhầm% đi đúng/song song  đổi trạng thái/chuyến đúng  báo/chuyến lệch  báo trước lệch%
  k_off=1 d=30 m trễ 0                 89.0     6.0/62.0        34/   232/1491       80.0 /35.0               4.760                    2.43            54.3 
  k_off=2 d=30 m trễ 0                 95.2     7.0/37.0        39/   186/822         7.0 /0.0                0.230                    1.45             3.5 
  k_off=3 d=30 m trễ 0                 95.2     8.0/38.0        45/   195/857         2.5 /0.0                0.070                    1.40             1.0 <== chọn
  k_off=5 d=50 m trễ 0                 87.0    15.0/65.0        79/   266/1334        0.0 /0.0                0.000                    1.09             0.0 
  k_off=1 d= 0 m trễ 0 ngây thơ       100.0     1.0/18.0         6/   101/200       100.0 /100.0            252.175                  137.95           100.0 
đọc: cấu hình ngây thơ báo nhầm 100.0% ca đi đúng và 100.0% ca song song, đổi trạng thái 252.175 lần mỗi chuyến đi đúng; cấu hình chọn 2.5%, 0.0%, 0.070; đổi lại độ trễ phát hiện p50 45 m so với 6 m
```

Đọc đúng cột: «báo nhầm %» là phần trăm chuyến đi đúng tuyến (cột trái) và chuyến đi dọc đường song song (cột phải) bị báo ít nhất một lần; «đổi trạng thái/chuyến đúng» là số lần `OnRoute` ↔ `OffRoute` trung bình mỗi chuyến đi đúng. Cấu hình chọn phát hiện **95,2%** lệch đáng kể, báo nhầm 2,5% ca đi đúng và 0,0% ca song song, đổi lại độ trễ phát hiện p50 là 8 s (45 m sau điểm lệch) thay vì 1 s (6 m) của cấu hình ngây thơ.

Các tỉ lệ này có độ bất định riêng. 2,5% là 5/200 chuyến, khoảng Wilson 95% [1,1%; 5,7%]; 95,2% là 139/146, khoảng [90,4%; 97,7%]; 0/40 ca song song vẫn tương thích với tỉ lệ thật tới 8,8% (tính tay từ số đếm). Cấu hình được chọn vì báo nhầm 1,0% và 0,0% trên tập chỉnh, nhưng 2,5% trên tập báo cáo đã vượt ngưỡng 2% đặt trước. Đó là chuyện thường gặp khi chọn cấu hình tốt nhất trong 26 cấu hình trên mẫu nhỏ, và là lý do phải báo cáo trên tập riêng.

Tỉ lệ phát hiện theo độ dài đoạn lạc cũng đáng nhìn:

```text
  chọn      đoạn lạc    0–50     m:   0/12  =   0.0%
  chọn      đoạn lạc   50–150    m:  32/41  =  78.0%
  chọn      đoạn lạc  150–400    m:  57/60  =  95.0%
  chọn      đoạn lạc  400–∞      m:  82/86  =  95.3%
```

Đoạn lạc dưới 50 m không bao giờ bị cấu hình chọn báo (0/12), và 50 đến 150 m chỉ phát hiện 78,0%. Đây là hệ quả của hysteresis và ngưỡng 30 m: đoạn lạc ngắn thường chưa đủ ba điểm liên tiếp cách tuyến quá ngưỡng. Vì vậy lab chỉ tính tỉ lệ phát hiện trên lệch «đáng kể» (đoạn lạc từ 150 m); có coi lệch dưới 150 m là đáng báo hay không là một quyết định sản phẩm mà lab không đưa ra.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 380" role="img" aria-label="Hình 2. Tỉ lệ báo nhầm trên 200 chuyến đi đúng tuyến, và tỉ lệ phát hiện ở cột phải. Cấu hình đã chọn báo nhầm 2,5% với nhiễu chuẩn nhưng 80,5% với nhiễu thô; trong ba cấu hình thử ở nhiễu thô, chỉ cấu hình chặt nhất giữ báo nhầm gần 0, trả bằng phát hiện 87,0%."><rect x="10" y="10" width="860" height="360" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="30" y="38" class="t-b">Tỉ lệ báo nhầm trên 200 chuyến đi đúng tuyến (tập báo cáo, trên dữ liệu mô phỏng)</text><text x="800" y="62" class="t-small" text-anchor="middle">phát hiện</text><text x="30" y="95">ngây thơ (k_off=1, d=0)</text><rect x="270" y="79" width="420.0" height="22" rx="4" class="f-warn" stroke-width="1.5"/><text x="698.0" y="95" class="t-b">100,0%</text><text x="30" y="131">k_off=1, d=30 m</text><rect x="270" y="115" width="336.0" height="22" rx="4" class="f-pk" stroke-width="1.5"/><text x="614.0" y="131" class="t-b">80,0%</text><text x="30" y="167">k_off=2, d=30 m</text><rect x="270" y="151" width="29.4" height="22" rx="4" class="f-pk" stroke-width="1.5"/><text x="307.4" y="167" class="t-b">7,0%</text><text x="30" y="203">k_off=3, d=30 m (đã chọn)</text><rect x="270" y="187" width="10.5" height="22" rx="4" class="f-pk" stroke-width="1.5"/><text x="288.5" y="203" class="t-b">2,5%</text><text x="30" y="239">k_off=5, d=50 m (chặt nhất)</text><rect x="270" y="223" width="2" height="22" rx="4" class="f-pk" stroke-width="1.5"/><text x="280" y="239" class="t-b">0,0%</text><text x="30" y="275">đã chọn, nhiễu thô σ 12 m</text><rect x="270" y="259" width="338.1" height="22" rx="4" class="f-warn" stroke-width="1.5"/><text x="616.1" y="275" class="t-b">80,5%</text><text x="30" y="311">chặt nhất, nhiễu thô σ 12 m</text><rect x="270" y="295" width="2.1" height="22" rx="4" class="f-pk" stroke-width="1.5"/><text x="280.1" y="311" class="t-b">0,5%</text><text x="800" y="95" class="t-small" text-anchor="middle">100,0%</text><text x="800" y="131" class="t-small" text-anchor="middle">89,0%</text><text x="800" y="167" class="t-small" text-anchor="middle">95,2%</text><text x="800" y="203" class="t-small" text-anchor="middle">95,2%</text><text x="800" y="239" class="t-small" text-anchor="middle">87,0%</text><text x="800" y="275" class="t-small" text-anchor="middle">91,8%</text><text x="800" y="311" class="t-small" text-anchor="middle">87,0%</text><text x="30" y="360" class="t-small">Vàng cam: cấu hình ngây thơ, và cấu hình đã chọn ở nhiễu thô (σ 12 m, 2% điểm nhảy xa, 3% rớt điểm). Xanh dương: còn lại.</text></svg>
</div>
<figcaption>Hình 2. Tỉ lệ báo nhầm trên 200 chuyến đi đúng tuyến, và tỉ lệ phát hiện ở cột phải. Cấu hình đã chọn báo nhầm 2,5% với nhiễu chuẩn nhưng 80,5% với nhiễu thô; trong ba cấu hình thử ở nhiễu thô, chỉ cấu hình chặt nhất giữ báo nhầm gần 0, trả bằng phát hiện 87,0%.</figcaption>
</figure>

Hình 2 cho thấy chỗ yếu nhất của kết quả. **Cấu hình «đã chọn» chỉ tốt với giả định nhiễu mà nó được chọn trên đó.** Khi nhiễu thô (σ 12 m, 2% điểm nhảy xa, 3% rớt điểm), cùng cấu hình báo nhầm 80,5% ca đi đúng; trong ba cấu hình thử, chỉ cấu hình chặt nhất (`k_off = 5`, 50 m) giữ 0,5%, với phát hiện 87,0% và độ trễ p50 16 s. Nhiễu GPS ngoài đời có thể khác xa cả hai mô hình này, nên con số 2,5% không mang sang được. Điều mang sang được là cơ chế: hysteresis biến một báo động không dùng được (đổi trạng thái 252 lần mỗi chuyến) thành một báo động dùng được (0,07 lần), và độ chặt phải hiệu chỉnh trên nhiễu của dữ liệu thật.

## Định tuyến lại phía server, và ngân sách thông báo

Phát hiện xong thì ai định tuyến lại? Lab đo phía server, nơi `Engine::route` chạy trên đồ thị CH của bài 25:

```text
    tài xế: gửi 279/279 cập nhật tuyến; khách: gửi 5/279 cập nhật ETA (đối chứng ngây thơ: khách nhận 279 thông báo, mỗi lần báo một thông báo)
  thay đổi ETA do lệch tuyến (ETA từ điểm lệch tới điểm đón trừ ETA nếu còn trên tuyến): p50 6 s, p95 42 s, max 166 s, tỉ lệ ≥ 60 s: 1.8% (n = 279)
  một truy vấn định tuyến lại phía server (Engine::route, CH, gồm snap và giải nén): p50 0.03 ms, p95 0.10 ms, max 0.31 ms (n = 558); Dijkstra theo độ dài của map-match: p50 0.22 ms, p95 1.00 ms (n = 279). Đo khi máy chịu tải nặng, chỉ so tương đối.
  chi phí phát lại giám sát + định tuyến lại: 1.69 µs mỗi điểm đã bám đường (51685 điểm, một luồng)
```

Một truy vấn định tuyến lại (gồm snap và giải nén đường) có p50 0,03 ms và p95 0,10 ms; Dijkstra theo độ dài mà bộ bám đường dùng nội bộ có p50 0,22 ms. **Thời gian này đo khi máy đang chịu tải nặng** (load average khoảng 121 trên 10 lõi), nên đọc tương đối: định tuyến lại phía server rẻ tới mức chi phí tính toán không phải lý do để đẩy việc xuống ứng dụng. Phần giám sát tốn 1,69 µs mỗi điểm đã bám đường, một luồng.

Phía ứng dụng thì lab **không đo**: không có số về độ trễ mạng di động, pin, hay chất lượng định tuyến cục bộ khi mất mạng. Lý lẽ nghiêng về server là bản đồ và chính sách nằm ở server và hai bên cần cùng một tuyến, nhưng đó là lập luận thiết kế, không phải kết quả đo.

Phần quan trọng hơn là **ai nhận thông báo nào**. Tuyến mới là thứ tài xế cần mọi lần; còn khách chỉ cần biết khi ETA đổi đáng kể. `Notifier` có ngân sách: tài xế tối đa 3 cập nhật tuyến mỗi chuyến, khách tối đa 2 cập nhật ETA, cách nhau ít nhất 30 giây, và khách chỉ được báo khi ETA đổi từ 60 giây. Trên 279 lần báo của cấu hình chọn: tài xế nhận 279/279 cập nhật tuyến, khách chỉ nhận **5/279** cập nhật ETA, vì thay đổi ETA do lệch tuyến có p50 6 s, p95 42 s, và chỉ 1,8% lần báo đổi ETA từ 60 giây. Đối chứng ngây thơ cho khách 279 thông báo, mỗi lần báo một thông báo.

## Khi nào được đổi tài xế

Đây là phần dễ làm hại nhất. ETA xấu đi thì khách muốn xe khác, nhưng tài xế đã lệch **đã chạy một đoạn rỗng**, và đổi người nghĩa là bỏ phí quãng đó. `RematchPolicy` chặn đổi trừ khi hội đủ điều kiện:

`examples/toi-noi-location/rust/crates/matcher/src/deviation.rs`

```rust
    /// Quyết định theo ETA đọc được lúc `now_ms`: `eta_orig_s` là ETA đã hứa với khách lúc gán, `eta_cur_s` của tài xế hiện tại,
    /// `eta_alt_s` của ứng viên tốt nhất khác. Gọi `Switch` thì state tự ghi nhận một lần đổi.
    pub fn decide(
        &mut self,
        p: &RematchPolicy,
        now_ms: i64,
        eta_orig_s: f64,
        eta_cur_s: f64,
        eta_alt_s: f64,
    ) -> RematchDecision {
        use RematchDecision::{Keep as K, Switch};
        if self.changes >= p.max_changes {
            return K(Keep::MaxChanges);
        }
        if eta_cur_s <= p.no_rematch_within_s {
            return K(Keep::TooCloseToArrival);
        }
        if eta_cur_s - eta_orig_s < p.min_worsening_s {
            return K(Keep::NotWorse);
        }
        if self
            .last_change_ms
            .is_some_and(|t| (now_ms - t) as f64 / 1000.0 < p.cooldown_s)
        {
            return K(Keep::Cooldown);
        }
        let gain = eta_cur_s - eta_alt_s;
        if gain > 0.0 && gain >= p.abs_margin_s.max(p.rel_margin * eta_cur_s) {
            self.changes += 1;
            self.last_change_ms = Some(now_ms);
            Switch
        } else {
            K(Keep::MarginTooSmall)
        }
    }
```

Chính sách `guarded` dùng: biên lợi tối thiểu `max(120 s, 25% ETA hiện tại)`, không đổi khi tài xế hiện tại còn dưới 90 giây là tới, chỉ xét khi ETA đã xấu đi ít nhất 60 giây, tối đa một lần đổi mỗi chuyến, cooldown 60 giây. Lab đánh giá bốn chính sách trên 139 sự kiện (lệch đáng kể đã được báo), ETA của tài xế khác đọc chính xác, các tài xế còn lại ở vị trí khởi tạo của kịch bản `hcm-baseline` và rảnh với xác suất 0,67; ETA được đọc lại ở mốc +0, +30, +60, +90 s sau lần báo. Có một giới hạn quyết định cách đọc bảng dưới: **tuyến đón trong lab sinh ngẫu nhiên, tài xế của chuyến không được chọn bằng ghép**, nên ngay từ đầu thường đã có xe khác gần hơn. Các cột lợi vì thế bị thổi lên; phần so được là hành vi tương đối giữa các chính sách.

```text
  chính sách (ETA đọc chính xác)     đổi%  sau báo s   khách lợi s mean/p50/p95    do lệch s    km rỗng thêm   tài xế mất chặng m  mất ≥1km%
  ngây thơ (đổi nếu nhanh hơn)      64.0%          1          148/  122/  329           28     -1.491 km/sk                 1997       80.9
  lỏng (60 s hoặc 10%)              43.9%         30          184/  180/  334           62     -1.867 km/sk                 2044       88.5
  có biên (120 s hoặc 25%)          12.2%         37          226/  243/  351           99     -2.309 km/sk                 2434      100.0
  chặt (240 s hoặc 50%)              5.8%         34          276/  250/  351           85     -2.734 km/sk                 2686      100.0
```

Cột `đổi%` ứng với 89, 61, 17 và 8 lần đổi trên 139 sự kiện (tính từ tỉ lệ), nên các số của hai chính sách sau dựa trên rất ít ca. Cột `km rỗng thêm` có nhãn `km/sk` trong output nhưng code chia cho số lần đổi, tức km mỗi lần đổi. Bốn điều đọc ra, trên dữ liệu mô phỏng:

- **Biên quyết định số lần đổi, và phần lớn «lợi» không đến từ lệch tuyến.** Chính sách có biên đổi tài xế ở 12,2% sự kiện, chính sách ngây thơ ở 64,0%. Ở 59,0% sự kiện, một tài xế rảnh khác đã nhanh hơn chính tài xế được gán **ngay từ lúc gán**, trước cả khi có lệch tuyến. Đó là hệ quả của cách lab sinh chuyến, không phải bằng chứng về dispatch thật. Cột «do lệch» tách phần ETA xấu đi vì lệch tuyến: 99 s trong mức lợi trung bình 226 s của chính sách có biên.
- **Lợi trung bình mỗi lần đổi tăng theo độ chặt** (148 s ở ngây thơ, 276 s ở chính sách chặt), nhưng đây chủ yếu là tác dụng chọn lọc: biên càng cao thì chỉ ca lợi lớn mới lọt qua, và chính sách chặt chỉ có 8 ca.
- **Tài xế đã lệch chịu thiệt.** Mỗi lần đổi, tài xế mất chặng đã chạy: trung bình 2.434 m ở chính sách có biên, và cả 17 và 8 ca đổi ở hai chính sách chặt đều mất từ 1 km trở lên (tuyến đón trung bình 4.429 m). Cột «km rỗng thêm» âm (−2,309 km mỗi lần đổi ở chính sách có biên) nghĩa là phần đường còn lại ngắn hơn khi đổi, cũng vì xe được chọn thay thế gần hơn ngay từ đầu. Lab **không cộng** hai cột này với nhau, nên bài không có con số tổng cho cả hệ thống.
- **Biên chặn dao động.** Khi ETA đọc có nhiễu log-chuẩn (đọc mỗi 15 s trong 300 s, hai tài xế cùng tiến lại điểm đón):

```text
  chính sách                          σ    đổi/sự kiện   đổi ≥ 2 lần%       đổi tối đa đổi sang người chậm hơn thật/sự kiện
  ngây thơ (đổi nếu nhanh hơn)     0.15          2.712          46.8%               13              0.655
  có biên (120 s hoặc 25%)         0.15          0.439           0.0%                1              0.000
  ngây thơ (đổi nếu nhanh hơn)     0.30          3.813          60.4%               14              1.201
  có biên (120 s hoặc 25%)         0.30          0.619           0.0%                1              0.072
```

Chính sách ngây thơ đổi 2,7 lần mỗi sự kiện ở σ = 0,15 và 3,8 lần ở σ = 0,30, có lúc đổi sang người chậm hơn thật (0,655 và 1,201 lần mỗi sự kiện); chính sách có biên không bao giờ đổi quá một lần. Đó là lý do biên phải lớn hơn nhiễu ETA, không chỉ lớn hơn 0.

**Bên nào lợi, bên nào thiệt.** Khách lợi khi tài xế khác thật sự gần hơn. Tài xế đã lệch bị rút chuyến sau khi đã chạy rỗng, và cả hai bên bị làm phiền nếu hysteresis và ngân sách thông báo quá lỏng. Lab không đủ để nói lợi của khách có bù được thiệt của tài xế trong vận hành thật không.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin`; `--fixture` chạy offline. Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc48_lech_tuyen_dinh_tuyen_lai
```

Lab chạy khoảng một phút. Tham số cho bản nhanh trên đồ thị tổng hợp: `--fixture --tune 10 --eval 20 --hard 5`; đó là cách `verify-p05.sh` chạy lại lab, nên số của bài (200 seed trên HCM) chỉ tái lập bằng lệnh đầy đủ ở trên. Phần mở đầu của output:

```text
uptime lúc bắt đầu: 0:32  up 35 days, 16:15, 1 user, load averages: 121.70 119.32 112.55
đồ thị: data/hcm-graph.bin: 126.699 nút, 319.074 cạnh; 100 seed chọn tham số 48001…48100, 200 seed báo cáo 48501…48700
lưu ý: trên dữ liệu mô phỏng. Vết GPS, nhiễu, đoạn lệch tuyến, xác suất tài xế rảnh và nhiễu ETA đều do lab sinh (có seed); không có GPS người thật. Không cắm vào simulator.
khởi động CH ô tô (nạp từ cache hoặc dựng): 0.01 s
danh mục ca khó: 2162 cặp đường song song (quét 0.5 s)
bám đường: HMM online (MatchConfig::default() của LOC-16), lấy mẫu 1 s; nhiễu chuẩn: chuẩn (σ 6 m, τ 8 s)
giám sát: k_on = 3, cooldown 30 s, tối đa 3 lần báo mỗi chuyến, sau mỗi lần báo server định tuyến lại và theo dõi tuyến mới (adopt)
lệch tuyến «đáng kể» = đoạn lạc tuyến liên tục ≥ 150 m; tỉ lệ phát hiện chỉ tính trên các ca này
sinh và bám đường 238 + 439 + 439 vết: 4.5 s
```

Các số về độ trễ phát hiện, tỉ lệ báo nhầm và đổi trạng thái tái lập được từng chữ số với cùng seed; các số thời gian (p50 0,03 ms…) thì không.

## Giới hạn của ví dụ

- **Không cắm vào simulator.** Không có số km rỗng, ETA hay tỉ lệ hủy toàn thành phố từ bài này, và không so với baseline của bài 30.
- **Lệch tuyến mô phỏng chỉ là rẽ sang cạnh khác** (nhẹ hoặc nặng 400 đến 900 m). Không có quay đầu giữa đường, cấm rẽ, hay lệch cố ý; lab cũng không phân biệt được lệch cố ý với điều hướng chỉ khác.
- **Nhiễu GPS là tổng hợp**, hai mức (chuẩn và thô). Tỉ lệ báo nhầm là hàm của mức nhiễu; ngoài đời cần hiệu chỉnh lại.
- **Tài xế của chuyến không được chọn bằng ghép**: tuyến đón sinh ngẫu nhiên, nên số «khách lợi» và «km rỗng thêm» không ước lượng được giá trị của ghép lại trong vận hành.
- **ETA ở phần ghép lại đọc từ vị trí thật của tài xế**, không qua bám đường, nên là cận trên của độ chính xác phía server. Các tài xế còn lại ở vị trí khởi tạo và rảnh ngẫu nhiên 67%; nhiễu ETA là log-chuẩn đơn giản.
- **Chưa đo phía ứng dụng** (định tuyến cục bộ, độ trễ mạng, pin), và chưa đo cảm nhận của khách khi tuyến đổi.

## Bài tập

**Bài 1.** Giả sử mỗi điểm bám đường bị gán nhầm «ngoài tuyến» độc lập với xác suất 0,2 khi xe đi đúng. Xác suất để k điểm liên tiếp đều nhầm là bao nhiêu với k = 1, 3, 5? Vì sao con số thật trong lab khác?

<details><summary>Lời giải bài 1</summary>
<p>0,2 với k = 1; 0,2³ = 0,008 với k = 3; 0,2⁵ = 0,00032 với k = 5. Đây là phép tính giả định, không phải số đo của lab. Con số thật khác vì nhiễu GPS tự tương quan (τ = 8 s): các điểm liên tiếp nhầm cùng nhau, nên chuỗi k điểm nhầm xảy ra nhiều hơn so với giả định độc lập. Tương quan này là một cách giải thích có thể cho việc `k_off = 3` vẫn báo nhầm 2,5% ca ở nhiễu chuẩn và 80,5% ở nhiễu thô; lab không tách riêng nguyên nhân.</p>
</details>

**Bài 2.** Với chính sách `guarded`: ETA gốc 300 s, ETA hiện tại 420 s, một tài xế khác có ETA 270 s thì có đổi không? Nếu tài xế khác có ETA 330 s thì sao? Chỉ ra điều kiện nào quyết định.

<details><summary>Lời giải bài 2</summary>
<p>Cả hai ca đều qua các điều kiện đầu: tài xế hiện tại còn hơn 90 s, ETA xấu đi 120 s (≥ 60 s), chưa đổi lần nào. Biên cần đạt: <code>max(120, 0,25 × 420) = 120</code> s. Với ETA 270 s, lợi là 420 − 270 = 150 s ≥ 120, nên <b>đổi</b>. Với ETA 330 s, lợi là 90 s &lt; 120, nên giữ nguyên với lý do <code>MarginTooSmall</code>. Biên ở đây quan trọng hơn mức xấu đi của ETA.</p>
</details>

**Bài 3.** Cột «mất ≥ 1 km %» là 100,0 ở chính sách có biên. Bài nói lab không cộng «km rỗng thêm» với «chặng tài xế đã mất». Hãy nêu một cách tính tổng hợp hợp lý và những gì còn thiếu trong lab để tính được.

<details><summary>Lời giải bài 3</summary>
<p>Một cách: tổng km rỗng của hệ thống = quãng tài xế cũ đã chạy rỗng trước lúc đổi (mất hẳn) + quãng tài xế mới phải chạy tới điểm đón; so với kịch bản không đổi, tức tài xế cũ chạy hết quãng còn lại. Lab chỉ in hiệu «quãng tài xế mới phải chạy tới điểm đón trừ quãng tài xế đã lệch còn phải chạy» và quãng đã mất như hai cột riêng, nên chưa kết luận được tổng hợp. Còn thiếu mô hình việc tài xế cũ làm sau khi bị rút chuyến, điều mà lab không có.</p>
</details>

Bài trước: [bài 47](/articles/location-chain-dispatch-tai-xe-sap-tra-khach). Bài tiếp: [bài 49](/articles/location-tai-xe-ranh-di-dau-dieu-huong-theo-nhu-cau).

## Tài liệu gốc

- [Hysteresis (Wikipedia)](https://en.wikipedia.org/wiki/Hysteresis): khái niệm hai ngưỡng khác nhau để vào và ra một trạng thái, nền của `k_off` và `k_on`.
- [gRPC: Deadlines](https://grpc.io/docs/guides/deadlines/): ngân sách thời gian khi server định tuyến lại trong một lời gọi.
