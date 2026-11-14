---
slug: location-tong-hop-mo-phong-su-kien-roi-rac-seed
title: "Tổng hợp P03: bộ mô phỏng sự kiện rời rạc làm thước đo cho mọi tối ưu"
summary: "Bài 30: mô phỏng sự kiện rời rạc cho khách và tài xế với mọi giả định nêu rõ, cùng seed cho kết quả giống từng byte, baseline đón đúng điểm ghim và chọn tài xế gần nhất, độ nhiễu giữa 5 seed, vì sao một seed không đủ chứng minh cải thiện."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, simulation, discrete-event, baseline, benchmark, determinism]
language: vi
---

Hãy tưởng tượng một bài ở phần P04 viết: «đổi cách chọn điểm đón, km rỗng giảm 15%». Người đọc có quyền hỏi ba câu. Giảm so với cái gì? Chạy lại có ra đúng con số đó không? Và 15% có lớn hơn mức dao động tự nhiên giữa hai ngày mô phỏng khác nhau không? Nếu loạt bài này không trả lời được ba câu đó trước khi viết dòng tối ưu đầu tiên, mọi con số «giảm x%» phía sau chỉ là lời hứa. Bài này dựng cái thước đo: một bộ mô phỏng thành phố chạy lại ra đúng từng byte, một baseline cố định, và con số cho biết thước đo này dao động bao nhiêu.

Đây là bài 30 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi), bài tổng hợp phần P03. Bài dựng trên engine định tuyến của [bài 21](/articles/location-dijkstra-a-star-rust-do-thi-csr) đến [bài 29](/articles/location-routing-service-tonic-cache-fallback-p99), và trả lời câu hỏi xuyên suốt đã đặt ở [bài 01](/articles/location-bai-toan-quang-duong-khach-va-tai-xe): tổng quãng đường và thời gian của **cả khách lẫn tài xế**. Mọi số đo trong bài đều **lấy từ dữ liệu mô phỏng**, kịch bản `hcm-baseline`, đồ thị HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors), seed 20261106. Không có GPS người thật.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Chạy baseline hai lần với cùng seed: kết quả giống nhau tới đâu? Rồi đổi seed từ 20261106 sang 20261107, không đổi bất cứ thứ gì khác. Theo bạn <code>deadhead_m</code> p50 (km rỗng trung vị) của baseline thay đổi cỡ nào?</p>
<details><summary>Xem đáp án</summary><p>Cùng seed thì báo cáo, nhật ký chuyến (16.628 byte) và nhật ký sự kiện (46.671 byte, 1.073 dòng) <b>giống từng byte</b>. Đổi seed sang 20261107 thì <code>deadhead_m</code> p50 từ 1.955,0 m xuống 1.374,6 m: thấp hơn <b>29,7%</b> dù chiến lược không đổi. Cả năm seed 20261106–20261110 cho 1.874,9 ± 286,6 m. Một «cải thiện» cỡ 15% đo trên một seed nằm hoàn toàn trong khoảng dao động này.</p></details>
</div>

## Phần 3 đã dựng gì

Chín bài trước cung cấp cho simulator ba thứ: đồ thị theo cạnh có chi phí rẽ (bài 23), ba profile (bài 24), và một engine trả lời được cả đường ô tô lẫn đường đi bộ (bài 21–29). Simulator chỉ dùng những phần cần độ chính xác cao: đường chạy của xe đi qua A* theo cạnh, không qua CH hay ma trận, vì ở đây ta cần chi phí đúng theo luật rẽ của lab hơn là trả lời nhanh. Bài 30 gắn các mảnh đó thành một thế giới có khách, tài xế, thời gian và huỷ chuyến.

## Mô hình sự kiện: sáu loại sự kiện và một hàng đợi

Mô phỏng sự kiện rời rạc không bước theo từng giây. Bộ mô phỏng giữ một hàng đợi gồm các sự kiện sắp xảy ra, lấy sự kiện sớm nhất ra, nhảy đồng hồ tới thời điểm đó, và xử lý. Ở baseline, hàng đợi chỉ có sáu loại sự kiện: `Request` (khách đặt), `DriverArrives`, `PickupDone`, `TripEnds`, và hai sự kiện huỷ `Cancel` (khách huỷ sau khi gán) và `Patience` (khách hết kiên nhẫn khi chưa có xe). Việc gán tài xế (hàm `start_trip`) không phải một sự kiện riêng: nó chạy ngay bên trong xử lý của `Request`, hoặc của một sự kiện giải phóng tài xế, và hẹn `DriverArrives` (cùng `Cancel` nếu khách sẽ huỷ).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 310" role="img" aria-label="Hình 1. Mô hình sự kiện của simulator: Request, bước gán xe, DriverArrives, PickupDone, TripEnds nối tiếp nhau; Patience huỷ chuyến của khách hết kiên nhẫn khi chưa có xe; Cancel huỷ chuyến của khách sau khi gán nếu tài xế chưa tới. Hàng đợi sự kiện xếp theo thời điểm rồi theo số thứ tự thêm vào."><defs><marker id="loc30-f1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs><rect x="20" y="20" width="840" height="275" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Sáu loại sự kiện trong hàng đợi (và bước gán xe nằm giữa chúng)</text><rect x="40" y="66" width="140" height="70" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="50" y="88" class="t-b">Request</text><text x="50" y="108" class="t-small">chọn điểm đón,</text><text x="50" y="124" class="t-small">đặt hạn kiên nhẫn</text><path d="M182 101 L198 101" class="f-none s-muted" stroke-width="2" marker-end="url(#loc30-f1-a)"/><rect x="200" y="66" width="140" height="70" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="210" y="88" class="t-b">Gán xe</text><text x="210" y="108" class="t-small">start_trip, lập</text><text x="210" y="124" class="t-small">tuyến đón</text><path d="M342 101 L358 101" class="f-none s-muted" stroke-width="2" marker-end="url(#loc30-f1-a)"/><rect x="360" y="66" width="140" height="70" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="370" y="88" class="t-b">DriverArrives</text><text x="370" y="108" class="t-small">xe tới điểm đón,</text><text x="370" y="124" class="t-small">đợi khách, +30 s</text><path d="M502 101 L518 101" class="f-none s-muted" stroke-width="2" marker-end="url(#loc30-f1-a)"/><rect x="520" y="66" width="140" height="70" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="530" y="88" class="t-b">PickupDone</text><text x="530" y="108" class="t-small">lập tuyến có</text><text x="530" y="124" class="t-small">khách tới điểm trả</text><path d="M662 101 L678 101" class="f-none s-muted" stroke-width="2" marker-end="url(#loc30-f1-a)"/><rect x="680" y="66" width="140" height="70" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="690" y="88" class="t-b">TripEnds</text><text x="690" y="108" class="t-small">ghi chỉ số, trả xe,</text><text x="690" y="124" class="t-small">thử ghép tiếp</text><path d="M110 138 L110 176" class="f-none s-warn dash" stroke-width="1.5" marker-end="url(#loc30-f1-a)"/><rect x="40" y="178" width="140" height="60" rx="8" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="50" y="200" class="t-b">Patience</text><text x="50" y="218" class="t-small">300 s chưa có xe</text><text x="50" y="232" class="t-small">thì khách huỷ</text><path d="M270 138 L270 176" class="f-none s-warn dash" stroke-width="1.5" marker-end="url(#loc30-f1-a)"/><rect x="200" y="178" width="140" height="60" rx="8" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="210" y="200" class="t-b">Cancel</text><text x="210" y="218" class="t-small">khách huỷ nếu xe</text><text x="210" y="232" class="t-small">chưa tới (mô hình)</text><rect x="380" y="178" width="460" height="86" rx="8" class="f-surface s-muted" stroke-width="1.5"/><text x="392" y="200" class="t-b">Hàng đợi sự kiện</text><text x="392" y="220" class="t-small">khoá: (thời điểm tính bằng ms, số thứ tự lúc thêm vào)</text><text x="392" y="236" class="t-small">cùng thời điểm thì sự kiện thêm trước chạy trước</text><text x="392" y="252" class="t-small">một luồng, không duyệt HashMap, nên thứ tự luôn như nhau</text></svg>
</div>
<figcaption>Hình 1. Sáu loại sự kiện trong hàng đợi và bước gán xe nằm giữa chúng. Chuỗi chính đi từ Request tới TripEnds; Patience và Cancel là hai nhánh huỷ. Hàng đợi sắp theo thời điểm rồi theo số thứ tự lúc thêm vào.</figcaption>
</figure>

Khi `Request` xảy ra, chiến lược chọn điểm đón (`PickupStrategy`) trả `PickupChoice`, khách chuyển sang trạng thái chờ, một sự kiện `Patience` được hẹn sau 300 s, và simulator thử ghép. Mỗi khi có khách mới hoặc có tài xế rảnh, `try_match` gọi chiến lược ghép (`MatchStrategy`) với danh sách khách đang chờ và tài xế rảnh. Hai trait này là chỗ các bài sau thay chiến lược mà không động tới vòng lặp sự kiện.

### Baseline: đón đúng ghim, tài xế gần nhất

Baseline gồm hai cài đặt của hai trait trên. `NearestStraightLine` lấy tài xế rảnh gần điểm đón nhất theo **đường chim bay**, hoà thì mã nhỏ hơn; đó là cách tìm tài xế của [bài 09](/articles/location-tim-tai-xe-gan-nhat-bon-cach-co-so-do) và là thứ phần P05 sẽ cố vượt. `AtPin` đón khách đúng điểm ghim đã snap lên đường; khách đi bộ từ chỗ đứng thật tới đó.

`examples/toi-noi-location/rust/crates/simulator/src/strategy.rs`

```rust
    fn choose(&mut self, ctx: &Ctx, c: &CustomerView) -> Option<PickupChoice> {
        let snapped = ctx.snap_car(c.pin)?;
        let view = ctx.engine.view(Profile::Car);
        let approach = snapped.positions(view);
        if approach.is_empty() {
            return None;
        }
        let (walk_m, walk_s) = ctx.walk(c.true_pos, snapped.point);
        Some(PickupChoice {
            stop_point: snapped.point,
            approach,
            walk_m,
            walk_s,
            pin_snap_m: snapped.snap_dist_m,
        })
    }
```

## Mọi giả định của thế giới này, và chúng nằm ở đâu

Không con số nào của mô phỏng nói về đời thật. Chúng là hệ quả của các giả định dưới đây, mỗi giả định ở một chỗ nhìn thấy được: kịch bản `fixtures/scenario-hcm-baseline.toml` hoặc hằng số trong mã. Đổi giả định thì đổi số.

| Giả định | Giá trị trong baseline | Ở đâu |
|---|---|---|
| Đồ thị và tốc độ | HCM; tốc độ tự do theo loại đường (giả định của lab) | `profile.rs` |
| Chi phí rẽ | Rẽ phải 3 s, rẽ trái 6 s, quay đầu giữa đường bị cấm (bài 23) | `turn.rs` |
| Giao thông | Tự do (`free_flow`); chế độ `synthetic` dùng hồ sơ kẹt xe tổng hợp của bài 27 | kịch bản |
| Giờ bắt đầu | 08:00 | kịch bản |
| Nhu cầu | Khách đặt trong 3.600 s, tốc độ cơ sở 3,5 chuyến/phút; 600 s đầu ×0,6, 1.200 s cuối ×1,4; quá trình Poisson được làm thưa dần | kịch bản, `world.rs` |
| Điểm nóng | 8 tâm (toạ độ gần đúng của địa danh, chỉ làm tâm nhu cầu), độ lệch 700–1.000 m; 30% khách đặt đều trên mạng đường | kịch bản |
| Chuyến | Điểm trả cách điểm đón 1.500–8.000 m chim bay, hướng ngẫu nhiên | kịch bản |
| Khách | Ghim lệch khỏi chỗ đứng thật σ = 12 m mỗi trục; đi bộ 1,3 m/s; kiên nhẫn 300 s; bắt đầu đi bộ ngay lúc đặt | kịch bản |
| Tài xế | 120 người; 40% đặt đều trên mạng đường, còn lại quanh điểm nóng; không rong ruổi, không rời ca; đứng yên ở điểm trả cho tới khi được gán | kịch bản, `sim.rs` |
| Thời gian dừng | Đón 30 s, trả 10 s | kịch bản |
| Nhiễu chạy | Nhân mỗi chặng với nhiễu log-chuẩn σ = 0,10 | kịch bản |
| Quyết định ghép | Tức thời: mỗi lần có khách mới hoặc tài xế rảnh; chưa có cửa sổ lô | `decision = "immediate"` |
| Huỷ chuyến | Mô hình đơn giản bên dưới, **chưa hiệu chuẩn** | kịch bản, `scenario.rs` |

### Mô hình huỷ chuyến là một giả định, không phải đo đạc

Lúc gán tài xế, lab rút một số ngẫu nhiên phân bố đều `u` và tính xác suất huỷ:

`examples/toi-noi-location/rust/crates/simulator/src/scenario.rs`

```rust
    pub fn probability(&self, eta_s: f64, walk_m: f64) -> f64 {
        let ramp = ((eta_s - self.eta_free_s) / (self.eta_max_s - self.eta_free_s)).clamp(0.0, 1.0);
        (self.max_prob * ramp + self.walk_extra_per_100m * walk_m / 100.0).clamp(0.0, 1.0)
    }
```

Với baseline, `max_prob = 0,5`, `eta_free_s = 180`, `eta_max_s = 900`, `walk_extra_per_100m = 0`. Nếu `u < p`, khách huỷ sau một khoảng phản ứng phân bố đều trong 10–60 s, miễn là tài xế chưa tới. Ngoài ra khách huỷ khi hết 300 s mà chưa có xe.

Đừng đọc `cancel_rate = 0,1366` như tỉ lệ huỷ của một dịch vụ có thật: nó là đầu ra của công thức trên cho kịch bản này. Hai hệ quả cần nhớ. Một: vì `walk_extra_per_100m = 0`, **khách đi bộ xa hơn không làm tăng xác suất huỷ**. Một bài P04 bắt khách đi bộ nhiều hơn không bị phạt ở chỉ số này, nên mọi bài tối ưu phải báo `walk_m` trực tiếp. Hai: huỷ phụ thuộc ETA, nên một chiến lược ghép nhanh hơn sẽ có `cancel_rate` thấp hơn một phần vì chính giả định này.

## Vì sao cùng seed ra đúng từng byte

Tính tất định không tự có sẵn mà đến từ năm lựa chọn, mỗi lựa chọn chặn một nguồn sai khác không mong muốn.

**1. Bộ sinh số ngẫu nhiên tự cài.** Simulator dùng xoshiro256** khởi tạo bằng SplitMix64 thay vì crate `rand`, để kết quả không đổi khi nâng phiên bản thư viện. Mỗi nguồn ngẫu nhiên có một `stream` riêng từ cùng một seed: 1 cho khách, 2 cho tài xế, 3 cho huỷ chuyến, 4 cho nhiễu tốc độ. Nhờ vậy đổi số tài xế không làm đổi dãy khách (test `changing_driver_count_does_not_change_the_customer_stream`).

**2. Hàng đợi có khoá đầy đủ.** Khoá là `(thời điểm ms, số thứ tự lúc thêm vào)`, nên hai sự kiện cùng thời điểm luôn chạy theo thứ tự được thêm vào, không phụ thuộc thứ tự nội bộ của heap.

`examples/toi-noi-location/rust/crates/simulator/src/sim.rs`

```rust
    fn push(&mut self, t_ms: u64, ev: Ev) {
        let seq = self.events.len() as u64;
        self.events.push(ev);
        self.heap.push(Reverse((t_ms, seq)));
    }
```

**3. Một luồng, không duyệt `HashMap`.** Khách chờ được sắp theo `(request_ms, mã khách)` mỗi lần ghép; hoà khoảng cách thì chọn tài xế có mã nhỏ hơn.

**4. Luôn rút cùng số lần.** Nhiễu tốc độ luôn rút một số kể cả khi nhiễu tắt, huỷ chuyến luôn rút hai số cho mỗi chuyến, để dãy số không phụ thuộc vào việc bật hay tắt một tính năng. Bật `trace` cũng không đổi số (test `trace_does_not_change_the_numbers`).

**5. Đầu ra có định dạng cố định và mã băm.** Báo cáo và các nhật ký in số thực với số chữ số cố định. Mã băm FNV-1a của nhật ký chuyến (`100a862df76819aa` ở baseline) là con số đại diện để so «giống từng byte».

Test `same_seed_gives_byte_identical_output` so báo cáo, nhật ký chuyến, nhật ký sự kiện và JSON tổng hợp của hai lần chạy trên fixture. Lab trên HCM cũng `assert!` lúc chạy: báo cáo, nhật ký chuyến 16.628 byte và nhật ký sự kiện 46.671 byte (1.073 dòng) giống từng byte; đổi seed sang 20261107 cho nhật ký chuyến khác (15.082 byte).

### Tính tất định này hứa gì và không hứa gì

Tính tất định hứa: cùng mã, cùng kịch bản, cùng seed, cùng chiến lược thì cho kết quả giống từng byte **trên máy và bản build đã kiểm**. Nó không hứa hai điều.

- **Khác máy hay khác bộ biên dịch**: kết quả dùng số thực (logarit, mũ, cosin trong Box–Muller) có thể khác ở chữ số cuối; lab chỉ kiểm trên một máy.
- **Đổi chiến lược**: khi hai chiến lược ghép cho ra thứ tự gán khác nhau, stream 3 và 4 được tiêu thụ theo thứ tự sự kiện, nên số ngẫu nhiên thứ k rơi vào một chuyến khác. Dòng khách và vị trí tài xế ban đầu (stream 1 và 2) không phụ thuộc chiến lược, nên cùng seed thì hai chiến lược thấy **cùng một dãy khách**, nhưng không thấy cùng một dãy huỷ.

## Cách đọc bảng baseline

Chạy `loc30_mo_phong` (không tham số) cho baseline. Chỉ số §3.4 tính trên **chuyến hoàn thành** (n = 196 trên 227 khách đặt):

| Chỉ số | Đơn vị | Trung bình | p50 | p95 | Lớn nhất | Ý nghĩa |
|---|---|---:|---:|---:|---:|---|
| `walk_m` | mét | 78,0 | 11,0 | 348,3 | 3.492,5 | khách đi bộ tới điểm đón |
| `deadhead_m` | mét | 2.211,9 | 1.955,0 | 4.920,7 | 12.733,7 | tài xế chạy không tới điểm đón |
| `pickup_eta_s` | giây | 309,9 | 281,0 | 632,7 | 1.496,2 | từ lúc gán tới lúc tài xế tới |
| `meet_gap_s` | giây | 295,2 | 254,3 | 589,8 | 2.546,7 | chênh lệch giữa giờ tài xế và giờ khách tới điểm đón |
| `trip_m` | mét | 6.901,6 | 6.392,2 | 12.133,4 | 16.303,0 | đoạn có khách |

Ngoài bảng: `u_turns` tổng 265 trên 196 chuyến, 1,352 mỗi chuyến; `cancel_rate` 0,1366 (31 huỷ trên 227 khách đặt, đều là huỷ sau khi gán, 0 trường hợp hết kiên nhẫn); mô phỏng kết thúc ở 5.913 s vì các chuyến cuối chạy tiếp sau 3.600 s đặt xe.

Vài chỗ dễ đọc nhầm:

- **`pickup_eta_s` và `wait_s`** (chỉ số phụ) bằng nhau ở baseline vì không khách nào phải chờ xe rảnh; `wait_s` đếm từ lúc đặt, `pickup_eta_s` từ lúc gán.
- **`meet_gap_s`** là |giờ tài xế tới − giờ khách tới|. Baseline cho khách đi bộ ngay từ lúc đặt, nên khách thường tới trước và chỉ số này xấp xỉ thời gian chờ tài xế.
- **`deadhead_m`** của chuyến hoàn thành là độ dài tuyến đón. Quãng chạy không **bỏ phí** do khách huỷ (4.064 m) tính riêng ở `wasted_deadhead_m`. Tổng chạy không của chuyến hoàn thành là 433.535 m so với 1.352.715 m có khách, tỉ lệ chạy không 24,27%.
- **Mô phỏng này đang nhẹ tải**: 120 tài xế bận trung bình 32,5% thời gian, không khách nào hết kiên nhẫn. Một thí nghiệm về tranh chấp tài xế (như gom lô ở P05) có thể cho kết quả khác ở kịch bản đông hơn.

### Một chỉ số khó hiểu: `walk_m` p50 11 m nhưng lớn nhất 3.492,5 m

`walk_m` có đuôi rất dài: p50 11,0 m, p95 348,3 m, lớn nhất 3.492,5 m, và 10 trong 196 chuyến hoàn thành vượt 300 m. Cả hai đầu đến từ cùng một phép tính: `AtPin` gọi `ctx.walk` từ chỗ đứng thật tới điểm đón.

`examples/toi-noi-location/rust/crates/simulator/src/strategy.rs`

```rust
    pub fn walk(&self, from: LatLng, to: LatLng) -> (f64, f64) {
        let speed = self.scenario.customer.walk_speed_mps;
        match self.engine.route(Profile::Foot, from, to, None) {
            Ok(r) => (r.distance_m, r.distance_m / speed),
            Err(_) => {
                let d = haversine_m(from, to) * 1.3;
                (d, d / speed)
            }
        }
    }
```

`Engine::route` snap **cả hai đầu** lên cạnh đi bộ gần nhất rồi trả độ dài tuyến giữa hai điểm đã snap, không cộng đoạn từ chỗ đứng tới tim đường. Mục 6 của lab so sánh đường đó với đường chim bay, cho cả 227 khách đặt:

| Mét | p50 | p95 | Lớn nhất |
|---|---:|---:|---:|
| Chim bay, chỗ đứng → điểm đón | 15,8 | 56,8 | 109,3 |
| Đường đi bộ trong đồ thị | 10,7 | 293,2 | 3.492,5 |

**Đầu nhỏ.** Ở 136 trong 227 khách, hai đầu snap về cùng một cạnh đi bộ, nên đường đi bộ chỉ là đoạn dọc cạnh giữa hai chân chiếu. Vì vậy p50 còn thấp hơn cả chim bay: `walk_m` ở đây là cận dưới của quãng khách thật phải đi.

**Đầu lớn.** Không ai đứng xa: chim bay lớn nhất chỉ 109,3 m. Khách 76 cách điểm đón 59,3 m chim bay mà đường đi bộ dài 3.492,5 m, vòng qua Tân Sơn và Quang Trung. Đuôi đến từ tính **liên thông của đồ thị đi bộ**: hai điểm rất gần nhau nhưng nằm trên hai nhánh đường chỉ nối với nhau ở xa. Lab in tên đường của tám ca dài nhất: khách 126 (chim bay 31,6 m, đi bộ 1.057,6 m) phải qua Cầu Văn Thánh 1; khách 85 (52,4 m và 578,9 m) đi dọc Cầu Thủ Thiêm; khách 86 (14,1 m và 532,4 m) vòng từ Võ Nguyên Giáp sang đường song hành. Trong `world.rs`, chỉ **điểm ghim** phải snap được vào đường ô tô trong 120 m; chỗ đứng thật không được kiểm tra gì với đồ thị đi bộ, nên có thể snap vào một mặt cầu hay một đường song song. Đó là điều đồ thị ghi; lab không kiểm tra ngoài thực địa xem có lối đi tắt thật hay không.

Và `AtPin` không có ngưỡng đi bộ. Ở khách 76, xe tới sau 139,8 s, còn khách theo mô hình đi bộ hết 2.686,5 s, nên `meet_gap_s` lớn nhất 2.546,7 s chính là ca này: tài xế chờ hơn 42 phút và không ai huỷ, vì mô hình huỷ không biết gì về quãng đi bộ. Vậy đừng đọc 3.492,5 m như «khách phải đi bộ 3,5 km»; hãy đọc nó như một ca mà baseline chọn điểm đón sai phía so với chỗ khách đứng. Đó là dạng lỗi P04 phải giảm, và là lý do §3.4 có ràng buộc cứng về ngưỡng đi bộ.

## Nhiễu giữa các seed: vì sao một seed không chứng minh được gì

Giờ tới phần quan trọng nhất của bài. Lab chạy cùng một baseline trên năm seed liên tiếp:

| Seed | Khách | Hoàn thành | `cancel_rate` | `walk_m` p50 | `deadhead_m` p50 | `pickup_eta_s` p50 | `pickup_eta_s` p95 | `trip_m` p50 | `u_turns` mỗi chuyến |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 20261106 | 227 | 196 | 0,1366 | 11,0 | 1.955,0 | 281,0 | 632,7 | 6.392,2 | 1,352 |
| 20261107 | 207 | 191 | 0,0773 | 16,4 | 1.374,6 | 248,9 | 547,9 | 6.273,1 | 1,628 |
| 20261108 | 225 | 198 | 0,1200 | 19,5 | 1.921,6 | 288,9 | 835,3 | 6.747,0 | 1,515 |
| 20261109 | 231 | 186 | 0,1948 | 15,5 | 2.062,7 | 320,4 | 721,9 | 7.473,0 | 1,769 |
| 20261110 | 246 | 210 | 0,1463 | 14,1 | 2.060,7 | 318,5 | 861,7 | 6.822,4 | 1,538 |

Trung bình qua 5 seed (độ lệch chuẩn mẫu): `cancel_rate` 0,1350 (0,0426); `deadhead_m` p50 1.874,9 (± 286,6) m; `pickup_eta_s` p50 291,6 (± 29,6) s; `u_turns` mỗi chuyến 1,560 (± 0,153).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 2. deadhead_m p50 của baseline trên năm seed, cùng một chiến lược: 1.955,0; 1.374,6; 1.921,6; 2.062,7; 2.060,7 mét. Trung bình 1.874,9, độ lệch chuẩn 286,6. Seed 20261107 thấp hơn seed 20261106 tới 29,7% dù không đổi gì."><defs><marker id="loc30-f3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs><rect x="20" y="20" width="840" height="305" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">deadhead_m p50 của baseline trên năm seed (cùng chiến lược, chỉ đổi seed)</text><rect x="371.8" y="66" width="401.2" height="196" rx="4" class="f-pk-soft" stroke-width="1"/><path d="M572.4 66 L572.4 262" class="f-none s-pk dash" stroke-width="1.5"/><text x="100.0" y="282" class="t-small" text-anchor="middle">1.200</text><text x="240.0" y="282" class="t-small" text-anchor="middle">1.400</text><text x="380.0" y="282" class="t-small" text-anchor="middle">1.600</text><text x="520.0" y="282" class="t-small" text-anchor="middle">1.800</text><text x="660.0" y="282" class="t-small" text-anchor="middle">2.000</text><text x="800.0" y="282" class="t-small" text-anchor="middle">2.200</text><text x="40" y="96" class="t-mono">20261106</text><circle cx="628.5" cy="92" r="7" class="f-warn s-warn" stroke-width="1.5"/><text x="642.5" y="96" class="t-mono">1.955,0</text><text x="40" y="134" class="t-mono">20261107</text><circle cx="222.2" cy="130" r="7" class="f-pk s-pk" stroke-width="1.5"/><text x="236.2" y="134" class="t-mono">1.374,6</text><text x="40" y="172" class="t-mono">20261108</text><circle cx="605.1" cy="168" r="7" class="f-pk s-pk" stroke-width="1.5"/><text x="619.1" y="172" class="t-mono">1.921,6</text><text x="40" y="210" class="t-mono">20261109</text><circle cx="703.9" cy="206" r="7" class="f-pk s-pk" stroke-width="1.5"/><text x="717.9" y="210" class="t-mono">2.062,7</text><text x="40" y="248" class="t-mono">20261110</text><circle cx="702.5" cy="244" r="7" class="f-pk s-pk" stroke-width="1.5"/><text x="716.5" y="248" class="t-mono">2.060,7</text><text x="40" y="306" class="t-small">Vùng xanh nhạt: trung bình 1.874,9 m ± 286,6 m (độ lệch chuẩn 5 seed). Chấm cam: seed chính của baseline. Đơn vị: mét, trên dữ liệu mô phỏng.</text></svg>
</div>
<figcaption>Hình 2. deadhead_m p50 của baseline trên năm seed: cùng chiến lược, cùng kịch bản, chỉ đổi seed. Dải xanh nhạt là trung bình ± một độ lệch chuẩn (1.874,9 ± 286,6 m). Trên dữ liệu mô phỏng.</figcaption>
</figure>

**Dao động tương đối rất lớn.** Độ lệch chuẩn so với trung bình là 15,3% với `deadhead_m` p50, 10,2% với `pickup_eta_s` p50, 9,8% với `u_turns` mỗi chuyến, 31,6% với `cancel_rate` (tính từ các số trên). Seed 20261107 cho `deadhead_m` p50 thấp hơn seed 20261106 tới **29,7%**, **mà không đổi gì về chiến lược**. Nếu một bài P04 chạy baseline trên 20261106 và «chiến lược mới» trên 20261107, nó sẽ khoe giảm 29,7% km rỗng chỉ do may rủi.

**Vì sao lại lớn thế?** Mỗi seed là một «ngày giả định» với dãy khách và vị trí tài xế khác nhau, và mỗi p50 chỉ tính từ khoảng 200 chuyến. Chênh lệch giữa hai «ngày» là thật trong mô hình, nhưng không liên quan gì tới chiến lược.

**Seed chính không phải seed điển hình.** `walk_m` p50 của seed 20261106 là 11,0 m, thấp nhất trong năm seed (trung bình 15,3 m, độ lệch chuẩn 3,1 m, tính từ cột của bảng). Chọn một seed «cho đẹp» là một dạng chọn lọc kết quả. Nhiễu của phân vị đuôi như p95 chưa được đo đủ; cột `pickup_eta_s` p95 đã dao động từ 547,9 đến 861,7 s.

**Kể cả so cùng seed, một seed vẫn chỉ cho một giá trị hiệu.** Một con số `Δ` không cho biết nó là hiệu ứng của chiến lược hay may rủi của đúng dãy khách đó. Muốn biết phải có nhiều giá trị `Δ` để ước lượng nhiễu của chính nó.

### Quy ước cho P04 và P05

Lab đã **đo** nhiễu; phần dưới là **quy ước đề xuất** dựa trên phép đo đó, chưa phải kết quả đã kiểm.

1. **Không kết luận từ một seed.** Chỉ viết «giảm x%» khi có nhiều seed và chênh lệch lớn hơn nhiễu.
2. **So sánh theo cặp.** Cùng một seed, chạy baseline và chiến lược mới, tính `Δ = X_mới − X_baseline` cho từng seed, rồi báo trung bình và sai số chuẩn của `Δ` trên N seed. Cùng seed thì hai chiến lược thấy cùng dãy khách và cùng vị trí tài xế ban đầu, nên phần dao động do nhu cầu được khử khi lấy hiệu; đây là kỹ thuật «số ngẫu nhiên chung» (common random numbers). Hai giới hạn: việc khử chỉ **một phần**, vì dòng huỷ và nhiễu tốc độ lệch khi thứ tự gán khác nhau; và lab chưa đo việc so theo cặp giúp giảm nhiễu bao nhiêu lần. Việc đo đó là của bài P04 đầu tiên.
3. **Tách seed dùng để chỉnh và seed dùng để đánh giá.** Bộ trọng số `a, b, c, d` ở §3.4 sẽ được chọn bằng cách thử nhiều giá trị trên simulator. Chọn và đánh giá trên cùng seed thì kết quả tốt hơn thực tế một cách hệ thống, vì ta đã chọn giá trị hợp với nhiễu của chính những seed đó. Dùng một dải seed để chỉnh, một dải **khác**, chốt trước, để báo con số cuối.
4. **Báo cáo kèm cỡ mẫu.** Mẫu: `Δ deadhead_m p50 = −X m (−Y%) ± Z (sai số chuẩn, N seed cặp), trên dữ liệu mô phỏng, kịch bản hcm-baseline`.

Cần bao nhiêu seed? Một phép tính thô: nếu so hai nhóm seed **độc lập**, sai số chuẩn của hiệu hai trung bình là `sd × √(2/N)`. Với `sd = 286,6 m`, muốn sai số chuẩn nhỏ hơn một phần ba hiệu cần phát hiện, thì với mục tiêu phát hiện cải thiện 5% (khoảng 94 m), phải cỡ `2 × (286,6 / 31,3)² ≈ 168` seed mỗi nhóm. `sd` chỉ tính từ 5 seed nên đây là ước lượng thô, nhưng nó cho thấy cỡ của vấn đề, và vì sao so theo cặp đáng để đo.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` (hoặc `--fixture` cho kịch bản `scenario-khu-mau.toml`, chỉ để test, không phải baseline). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc30_mo_phong
```

Thêm `-- --csv thu-muc` để ghi `trips.csv`, `events.csv`, `summary.json`; `-- --seeds N` đổi số seed phụ. Kết quả in ra (rút gọn), kịch bản `fixtures/scenario-hcm-baseline.toml`, seed 20261106:

```text
khách đặt 227: hoàn thành 196, huỷ sau khi gán 31, huỷ vì hết kiên nhẫn chưa có xe 0, huỷ vì không có đường 0
mô phỏng kết thúc ở 5913 s; tài xế bận trung bình 32.5% thời gian
  walk_m        trung bình     78.0  p50     11.0  p95    348.3  max   3492.5  (n = 196)
  deadhead_m    trung bình   2211.9  p50   1955.0  p95   4920.7  max  12733.7  (n = 196)
  pickup_eta_s  trung bình    309.9  p50    281.0  p95    632.7  max   1496.2  (n = 196)
  cancel_rate   0.1366  (31 huỷ trên 227 khách đặt)
nhật ký chuyến: 227 dòng, FNV-1a = 100a862df76819aa
  báo cáo giống từng byte: true; nhật ký chuyến (16628 byte) giống từng byte: true; nhật ký sự kiện (46671 byte, 1073 dòng) giống từng byte: true
  đổi seed 20261106 → 20261107: nhật ký chuyến khác (15082): true
  trung bình qua 5 seed: cancel 0.1350 (độ lệch chuẩn 0.0426); deadhead_m p50 1874.9 (± 286.6); pickup_eta_s p50 291.6 (± 29.6); u_turns/chuyến 1.560 (± 0.153)
  đường đi bộ p50 10.7, p95 293.2, lớn nhất 3492.5 m; chim bay chỗ đứng → điểm đón p50 15.8, p95 56.8, lớn nhất 109.3 m; hai đầu snap về cùng một cạnh đi bộ: 136/227
  khách 76: đi bộ 3492.5 m, chim bay 59.3 m, qua: Tân Sơn > Quang Trung
```

Tất cả các số này tái lập từng chữ số với cùng seed. Lab còn in thời gian chạy một mô phỏng (2,19 s, đo khi máy đang chịu tải nặng); bài này không dùng số đó. `verify-p03.sh` chạy test tất định trên đồ thị tổng hợp, không chạy lab trên HCM.

## Giới hạn của ví dụ

**Kết quả của simulator không dự đoán thế giới thật.** Chúng là thước đo lặp lại được để so các chiến lược với nhau trong cùng một thế giới giả định. Một chiến lược tốt hơn ở đây có thể không tốt hơn ngoài đời, và con số tuyệt đối (km rỗng, ETA, tỉ lệ huỷ) không có ý nghĩa ngoài mô phỏng. Những chỗ cụ thể:

- **Nhu cầu** là Poisson quanh tám điểm nóng đặt thủ công; **huỷ chuyến** là công thức tuyến tính chưa hiệu chuẩn và không phụ thuộc quãng đi bộ; **kẹt xe** ở baseline là giao thông tự do.
- **Tài xế** không rong ruổi tìm khách, không rời ca, không từ chối chuyến; **ghép chuyến** tức thời, chưa có cửa sổ lô.
- **Đường đi bộ** dựa trên OSM và snap không biết chướng ngại, nên có thể thiếu lối đi tắt thật; đó là nguồn của đuôi `walk_m`.
- **Tốc độ chạy, chi phí rẽ, nhiễu thời gian** đều là giả định của lab, và **tải nhẹ**: tài xế bận 32,5% thời gian.
- **Một kịch bản, một thành phố**; chưa có kịch bản đông hơn, đêm hay mưa. Chỉ có hai chiến lược baseline, đều «ngây thơ có chủ ý».
- **Nhiễu chỉ đo cho vài chỉ số trên năm seed**, cỡ mẫu nhỏ. Chưa có cơ chế so sánh theo cặp trong mã.
- **Tính tất định chỉ kiểm trên một máy**, chưa kiểm giữa các nền tảng.

## Bài tập

**Bài 1.** Tính độ giảm phần trăm của `pickup_eta_s` p50 giữa seed 20261106 (281,0 s) và 20261107 (248,9 s), và so với độ lệch chuẩn tương đối của chỉ số đó qua 5 seed. Kết luận gì về một «cải thiện ETA 10%» đo trên một seed?

<details><summary>Lời giải bài 1</summary>
<p>(281,0 − 248,9) / 281,0 ≈ 11,4%. Độ lệch chuẩn tương đối của `pickup_eta_s` p50 là 29,6 / 291,6 ≈ 10,2%. Hai con số cùng cỡ, nên «cải thiện 10%» đo trên một seed không phân biệt được với việc đổi seed. Cần nhiều seed và so sánh theo cặp.</p>
</details>

**Bài 2.** Vì sao cùng seed mà hai chiến lược ghép khác nhau không cho cùng một dãy huỷ chuyến, dù dãy khách giống nhau? Đề xuất một thay đổi nhỏ trong mô phỏng để các chiến lược rút đúng cùng số ngẫu nhiên cho cùng một khách.

<details><summary>Lời giải bài 2</summary>
<p>Huỷ chuyến và nhiễu tốc độ dùng các dòng số (stream 3 và 4) được rút theo thứ tự các sự kiện gán và lập tuyến. Khi hai chiến lược gán khác nhau, số ngẫu nhiên thứ k rơi vào hai chuyến khác nhau, nên dãy huỷ khác dù dãy khách (stream 1) và vị trí tài xế ban đầu (stream 2) giống nhau. Đề xuất, chưa được lab cài: sinh trước, cho mỗi khách, hai số dành cho huỷ (xác suất và độ trễ phản ứng) và một nhiễu dành cho mỗi chặng của khách đó (dòng riêng theo mã khách), để cùng một khách luôn rút cùng số bất kể chiến lược nào phục vụ.</p>
</details>

**Bài 3.** Baseline có `walk_extra_per_100m = 0`. Một bài P04 đề xuất điểm đón làm `walk_m` p50 tăng từ 11,0 lên 60 m nhưng giảm `deadhead_m`. Theo mô hình hiện tại `cancel_rate` có phản ánh việc khách đi bộ nhiều hơn không? Nên báo cáo gì?

<details><summary>Lời giải bài 3</summary>
<p>Không: xác suất huỷ phụ thuộc ETA dự tính chứ không phụ thuộc quãng đi bộ, vì hệ số `walk_extra_per_100m` bằng 0. Nên bài phải báo `walk_m` (p50 và p95, kể cả đuôi) cạnh `deadhead_m` để người đọc thấy ai được lợi và ai chịu thiệt, và có thể thử thêm một kịch bản với `walk_extra_per_100m` khác 0 như một phân tích độ nhạy, nói rõ đó là giả định.</p>
</details>

Bài trước: [bài 29](/articles/location-routing-service-tonic-cache-fallback-p99). Bài tiếp: [bài 31](/articles/location-geocoding-dia-chi-viet-nam-autocomplete).

## Tài liệu gốc

- [Discrete-event simulation (Wikipedia)](https://en.wikipedia.org/wiki/Discrete-event_simulation): mô hình mô phỏng bằng dòng sự kiện tức thời.
- [Variance reduction (Wikipedia)](https://en.wikipedia.org/wiki/Variance_reduction): mục «common random numbers» cho so sánh theo cặp.
- [xoshiro / xoroshiro generators](https://prng.di.unimi.it/): trang của S. Vigna và D. Blackman về họ bộ sinh số ngẫu nhiên mà simulator tự cài.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho dữ liệu OSM.
