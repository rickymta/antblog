---
slug: location-chain-dispatch-tai-xe-sap-tra-khach
title: "Ghép nối chuyến: nhận chuyến kế khi tài xế sắp trả khách"
summary: "Bài 47: tài xế đang chở khách cũng là ứng viên, ETA còn lại có độ bất định. Trên dữ liệu mô phỏng, chain giảm khách bị bỏ rơi và thêm chuyến, nhưng khi xe thưa thì khách điển hình chờ lâu hơn; biên rủi ro không thấy có tác dụng."
category: location-routing
level: advanced
tags: [toi-noi, location, rust, dispatch, chain-dispatch, matching, eta-uncertainty, simulation]
language: vi
---

Mưa chiều, khách đặt xe liên tục và mọi tài xế gần đó đều đang chở khách. Trong số đó có một chiếc còn khoảng hai phút nữa là trả khách, và điểm trả cách điểm đón mới vài trăm mét. Theo baseline của loạt bài này, khách mới phải đợi một tài xế **rảnh** chạy từ xa tới. Hệ thống biết rõ có một chiếc xe sắp rảnh ngay cạnh điểm đón mà không cho nó vào cuộc.

Ghép nối chuyến (chain dispatch) cho phép chiếc xe đó làm ứng viên: ghép khách với nó từ bây giờ, khách xếp hàng chờ, xe đón ngay khi trả khách xong. Nghe có vẻ chỉ có lợi. Bài này đo xem có thật vậy không, và cái giá nằm ở đâu: ETA «còn lại» của một chuyến đang chạy là một ước lượng, mà ước lượng thì có thể sai.

Đây là bài 47 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài này xây dựng trên bộ ghép theo lô của [bài 42](/articles/location-ghep-theo-lo-hungarian-kuhn-munkres-rust) và trường `busy_remaining_ms` của hợp đồng gRPC ở [bài 46](/articles/location-goi-y-sang-dispatch-csharp-grpc-lease-idempotency). Mọi số đo trong bài là **trên dữ liệu mô phỏng**: simulator của [bài 30](/articles/location-tong-hop-mo-phong-su-kien-roi-rac-seed), đồ thị HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors), 20 seed đánh giá 20261106…20261125. Cách so sánh có cặp theo seed và khoảng tin cậy (KTC) 95% đã giải thích ở [bài 41](/articles/location-gan-nhat-sai-o-dau-do-bang-mo-phong); bài này dùng nguyên.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bộ «thưa xe»: 60 tài xế, 3,5 chuyến mỗi phút. Cho phép tài xế sắp trả khách trong 120 giây tới làm ứng viên. Theo bạn thời gian chờ trung bình của khách (<code>wait_s</code>) sẽ giảm, tăng hay không đổi, và số chuyến hoàn thành thay đổi ra sao?</p>
<details><summary>Xem đáp án</summary><p>Trên dữ liệu mô phỏng, <code>wait_s</code> trung bình <b>tăng</b> 26,9 s (KTC [8,9; 44,8]) trong khi số chuyến hoàn thành <b>tăng</b> 3,2 chuyến mỗi seed (KTC [1,6; 4,8]) và tỉ lệ khách hết kiên nhẫn chưa có xe giảm từ 0,0733 xuống 0,0415. Khoản chờ thêm không chỉ đến từ những khách được cứu: trung vị <code>wait_s</code> cũng tăng 33,8 s. Mục kết quả giải thích vì sao.</p></details>
</div>

## Tài xế bận cũng là ứng viên

Cơ chế chỉ thêm một thứ vào ma trận chi phí của bài 42: mỗi tài xế đang chở khách mà sẽ rảnh trong tầm nhìn `H` giây được thêm thành một cột. Vị trí của cột là **điểm trả khách** của chuyến hiện tại, và chi phí cộng thêm một khoảng dẫn đầu `lead_ms`: thời gian còn lại của chuyến trước.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 1. Hai ứng viên cho cùng một điểm đón. Tài xế B đang bận nhưng tới điểm đón sớm hơn tài xế rảnh A ở xa. Thuật toán Hungarian xử lý hai cột này như hai tài xế bình thường; chi phí của B thêm khoảng dẫn đầu."><rect x="10" y="10" width="860" height="310" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="30" y="40" class="t-b">Hai ứng viên cho cùng một điểm đón (độ dài các đoạn chỉ minh hoạ cơ chế, không phải số đo)</text><text x="80" y="74" class="t-b">A. tài xế rảnh nhưng ở xa</text><rect x="80" y="84" width="560" height="34" rx="6" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="92" y="106" class="t-small">ETA(vị trí hiện tại → điểm đón)</text><text x="80" y="152" class="t-b">B. tài xế đang chở khách, sắp tới điểm trả</text><rect x="80" y="162" width="250" height="34" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="92" y="184" class="t-small">remaining_est (+ dừng trả khách)</text><rect x="330" y="162" width="190" height="34" rx="6" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="342" y="184" class="t-small">ETA(điểm trả → điểm đón)</text><rect x="520" y="162" width="70" height="34" rx="6" class="f-none s-warn dash" stroke-width="1.5"/><text x="530" y="184" class="t-small">risk_k·sd</text><text x="30" y="240" class="t-mono">lead_ms = remaining_est + risk_k × sd</text><text x="30" y="260" class="t-small">chi phí của cặp (khách, B) = lead_ms + ETA(điểm trả → điểm đón), rồi vào ma trận Hungarian như một tài xế thường</text><text x="30" y="280" class="t-small">sd ≈ remaining_est × (e^σ − 1), xấp xỉ một độ lệch chuẩn, σ = speed_jitter; khách ghép với B thì xếp hàng chờ B trả khách xong</text><text x="30" y="304" class="t-small">risk_k = 0: tin ước lượng tuyệt đối; risk_k = 1: cộng một độ lệch chuẩn vào chi phí (nét đứt)</text></svg>
</div>
<figcaption>Hình 1. Hai ứng viên cho cùng một điểm đón. Tài xế B đang bận nhưng tới điểm đón sớm hơn tài xế rảnh A ở xa. Thuật toán Hungarian xử lý hai cột này như hai tài xế bình thường; chi phí của B thêm khoảng dẫn đầu.</figcaption>
</figure>

Phía simulator, hàm `finishing_views` chọn những tài xế đủ điều kiện. Chú ý chỗ hàm này **không biết gì về nhiễu tốc độ** của chuyến thật: ước lượng thời gian còn lại lấy từ kế hoạch, không phải từ tương lai.

`examples/toi-noi-location/rust/crates/simulator/src/sim.rs`

```rust
    fn finishing_views(&self) -> Vec<FinishingView> {
        let Some(horizon) = self.matcher.chain_horizon_ms() else {
            return Vec::new();
        };
        let dwell = (self.sc.driver.dwell_dropoff_s * 1000.0) as u64;
        let mut out = Vec::new();
        for (i, d) in self.drivers.iter().enumerate() {
            let DState::WithPassenger(tid) = d.state else {
                continue;
            };
            if d.queued.is_some() {
                continue;
            }
            let Some(a) = self.trips[tid as usize].trip.as_ref() else {
                continue;
            };
            let left_true = d.free_at_ms.saturating_sub(self.now_ms);
            let dwell_left = left_true.min(dwell);
            let drive_true = (left_true - dwell_left) as f64;
            let est = (drive_true * a.planned_ms as f64 / a.total_ms.max(1) as f64).round() as u64
                + dwell_left;
            if est <= horizon {
                let end_pos = a.end_position();
                out.push(FinishingView {
                    id: i as u32,
                    end_pos,
                    end_latlng: self.latlng(end_pos),
                    remaining_est_ms: est,
                });
            }
        }
        out
    }
```

Phía bộ ghép, tài xế sắp trả khách được cộng vào danh sách ứng viên với `lead_ms`. Dòng comment ở đầu vòng lặp là toàn bộ mô hình bất định của bài này:

`examples/toi-noi-location/rust/crates/matcher/src/batch.rs`

```rust
        if let Some(ch) = self.cfg.chain {
            for f in finishing {
                // chuyến trước có thể kéo dài: log-chuẩn σ nên một độ lệch chuẩn ≈ remaining × (e^σ − 1)
                let sd = f.remaining_est_ms as f64 * (sigma.exp() - 1.0);
                cands.push(Cand {
                    id: f.id,
                    pos: f.end_pos,
                    latlng: f.end_latlng,
                    lead_ms: f.remaining_est_ms as i64 + (ch.risk_k * sd).round() as i64,
                    idle_ms: 0,
                });
            }
        }
```

Nếu cặp (khách, tài xế bận) thắng, simulator xếp khách vào hàng chờ của xe đó (`queue_chain`), mỗi xe tối đa một khách xếp hàng. Xe đi đón ngay khi trả khách xong.

## ETA còn lại không chắc: độ lệch chuẩn và biên rủi ro

`remaining_est_ms` là phần còn lại **theo kế hoạch** cộng thời gian dừng trả khách. Trong simulator, mỗi chặng đường bị nhân một nhiễu log-chuẩn (`speed_jitter`, σ = 0,1 ở baseline) mà chiến lược không biết, nên chuyến thật có thể dài hơn kế hoạch. Code lấy `sd ≈ remaining × (e^σ − 1)`, tức khoảng cách từ trung vị tới phân vị 84% của nhiễu log-chuẩn: σ = 0,1 cho 0,105 × remaining, σ = 0,3 cho 0,350 × remaining. Đây chỉ là xấp xỉ của độ lệch chuẩn thật `e^(σ²/2) · √(e^(σ²) − 1)` (0,101 và 0,321), đủ gần để làm biên an toàn.

Cấu hình `ChainConfig` có hai tham số: tầm nhìn và biên rủi ro `risk_k`.

`examples/toi-noi-location/rust/crates/matcher/src/batch.rs`

```rust
pub struct ChainConfig {
    /// Chỉ xét tài xế sẽ rảnh trong chừng này mili giây.
    pub horizon_ms: u64,
    /// Số độ lệch chuẩn của «chuyến trước kéo dài» cộng vào chi phí (0 = tin ước lượng tuyệt đối).
    pub risk_k: f64,
}
```

`risk_k = 0` tin ước lượng tuyệt đối. `risk_k = 1` cộng một độ lệch chuẩn vào chi phí của tài xế bận, tức là xe sắp trả khách phải «thắng» ngay cả khi đã bị cộng thêm một biên an toàn. Cần nhớ rằng biên này chỉ đổi **ai được chọn**. Biên này không rút ngắn chuyến trước. Phần kết quả sẽ cho thấy điều đó.

## Khách xếp hàng cũng có thể huỷ

Khách xếp hàng chờ chưa có xe nào đang chạy tới, nhưng mô hình huỷ của simulator vẫn tính cho họ. Xác suất huỷ tính theo ETA tổng (phần còn lại của chuyến trước cộng quãng đón), và khách huỷ được cả khi còn đang xếp hàng:

`examples/toi-noi-location/rust/crates/simulator/src/sim.rs`

```rust
        let eta_planned_s = (fv.remaining_est_ms + planned) as f64 / 1000.0 + waited_s;
        // mô hình huỷ như start_trip: rút (u, phản ứng) ngay lúc gán, vì khách huỷ được cả khi còn xếp hàng
        let (u, react) = (self.rng_cancel.f64(), self.rng_cancel.f64());
        let cn = &self.sc.cancel;
        let p = cn.probability(eta_planned_s, pickup.walk_m);
        let cancel_at = now
            + ((cn.reaction_min_s + react * (cn.reaction_max_s - cn.reaction_min_s)) * 1000.0)
                .round() as u64;
        let expected_arrival = now + fv.remaining_est_ms + planned;
        {
            let cu = &mut self.custs[c as usize];
            cu.status = CStatus::Queued;
            cu.queued_ms = now;
            cu.queued_driver = Some(d);
        }
        self.drivers[d as usize].queued = Some(c);
        self.extras.chained += 1;
        let at = self.latlng(fv.end_pos);
        self.trace("chain_queue", Some(c), Some(d), at);
        if u < p && cancel_at < expected_arrival {
            self.push(cancel_at, Ev::CancelQueued(c));
        }
```

Điều này quan trọng với cách đọc số liệu. Trên dữ liệu mô phỏng, ở bộ «thưa xe» với chain-120 có khoảng 89 khách xếp hàng mỗi seed, trong đó khoảng 28 khách huỷ trước khi xe tới: khoảng 31% khách xếp hàng. Những khách ở lại thì chờ một chuyến chưa kết thúc.

Có thêm một chỗ cần cẩn thận, đã nêu từ bài 41: các chỉ số `wait_s`, `pickup_eta_s`, `deadhead_m` của simulator tính **trên chuyến hoàn thành**. Chain cứu thêm những khách lẽ ra bị bỏ rơi, và những khách đó có thời gian chờ dài. Trung bình `wait_s` có thể tăng dù chính sách làm lợi cho khách. Vì vậy mọi bảng dưới đây đi kèm tỉ lệ huỷ, khách hết kiên nhẫn và số chuyến hoàn thành.

## Kết quả trên bộ «thưa xe»

Baseline là Hungarian ghép ngay, không chain. Trên dữ liệu mô phỏng (60 xe, 3,5 chuyến/phút, 20 seed 20261106…20261125), trung bình mỗi seed baseline có 228,4 khách đặt, 150,5 chuyến hoàn thành, `cancel_rate` 0,3385, tỉ lệ hết kiên nhẫn 0,0733, `wait_s` 576,6 (p95 1.297,4) và `deadhead_m` 4.689,1. Bảng dưới là **hiệu** chain − không chain, KTC 95% trong ngoặc, dấu `*` khi KTC không chứa 0:

| Cấu hình | Xếp hàng | Huỷ khi chờ | `wait_s` TB (s) | `deadhead_m` TB (m) | `cancel_rate` | Hoàn thành |
|---|---:|---:|---|---|---|---|
| chain-60 | 66,3 | 13,1 | +2,9 [−8,7; 14,5] | −74,1 [−185,3; 37,2] | −0,0021 [−0,0076; 0,0034] | +0,6 [−0,7; 1,8] |
| chain-120 | 89,3 | 27,9 | +26,9 [8,9; 44,8]* | +11,1 [−192,5; 214,8] | −0,0139 [−0,0208; −0,0070]* | +3,2 [1,6; 4,8]* |
| chain-300 | 106,2 | 39,8 | +22,9 [2,0; 43,9]* | −381,2 [−578,4; −184,1]* | −0,0293 [−0,0356; −0,0229]* | +6,8 [5,2; 8,4]* |
| chain-120+r1 | 86,8 | 26,6 | +23,3 [4,1; 42,5]* | −3,7 [−216,4; 208,9] | −0,0118 [−0,0173; −0,0062]* | +2,7 [1,4; 4,0]* |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 360" role="img" aria-label="Hình 2. Hiệu chain − không chain trên bộ thưa xe: chờ lâu hơn (vàng cam) đổi lấy nhiều chuyến hoàn thành hơn (xanh ngọc). Chain-60 không phân biệt được với baseline ở cả hai chỉ số; tầm nhìn dài hơn kéo thêm cả hai."><rect x="10" y="10" width="860" height="340" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="30" y="38" class="t-small">Trên dữ liệu mô phỏng, bộ «thưa xe» (60 xe, 3,5 chuyến/phút), hiệu = chain − không chain, 20 seed có cặp, vạch = KTC 95%</text><text x="30" y="70" class="t-b">wait_s trung bình (giây), sang phải là xấu</text><text x="470" y="70" class="t-b">chuyến hoàn thành thêm mỗi seed, sang phải là tốt</text><text x="30" y="104">chain-60</text><text x="30" y="154">chain-120</text><text x="30" y="204">chain-300</text><text x="30" y="254">chain-120+r1</text><path d="M150 284 L400 284" class="f-none s-muted" stroke-width="1.5"/><path d="M150.0 284 L150.0 289" class="f-none s-muted" stroke-width="1.5"/><text x="150.0" y="304" class="t-small" text-anchor="middle">−20</text><path d="M221.4 284 L221.4 289" class="f-none s-muted" stroke-width="1.5"/><text x="221.4" y="304" class="t-small" text-anchor="middle">0</text><path d="M310.7 284 L310.7 289" class="f-none s-muted" stroke-width="1.5"/><text x="310.7" y="304" class="t-small" text-anchor="middle">25</text><path d="M400.0 284 L400.0 289" class="f-none s-muted" stroke-width="1.5"/><text x="400.0" y="304" class="t-small" text-anchor="middle">50</text><path d="M221.4 76 L221.4 284" class="f-none s-muted dash" stroke-width="1.5"/><path d="M190.4 100 L273.2 100" class="f-none s-muted" stroke-width="2"/><path d="M190.4 95 L190.4 105" class="f-none s-muted" stroke-width="2"/><path d="M273.2 95 L273.2 105" class="f-none s-muted" stroke-width="2"/><circle cx="231.8" cy="100" r="6" class="f-surface s-muted" stroke-width="2"/><path d="M253.2 150 L381.4 150" class="f-none s-muted" stroke-width="2"/><path d="M253.2 145 L253.2 155" class="f-none s-muted" stroke-width="2"/><path d="M381.4 145 L381.4 155" class="f-none s-muted" stroke-width="2"/><circle cx="317.5" cy="150" r="6" class="f-warn s-muted" stroke-width="1.5"/><path d="M228.6 200 L378.2 200" class="f-none s-muted" stroke-width="2"/><path d="M228.6 195 L228.6 205" class="f-none s-muted" stroke-width="2"/><path d="M378.2 195 L378.2 205" class="f-none s-muted" stroke-width="2"/><circle cx="303.2" cy="200" r="6" class="f-warn s-muted" stroke-width="1.5"/><path d="M236.1 250 L373.2 250" class="f-none s-muted" stroke-width="2"/><path d="M236.1 245 L236.1 255" class="f-none s-muted" stroke-width="2"/><path d="M373.2 245 L373.2 255" class="f-none s-muted" stroke-width="2"/><circle cx="304.6" cy="250" r="6" class="f-warn s-muted" stroke-width="1.5"/><path d="M520 284 L720 284" class="f-none s-muted" stroke-width="1.5"/><path d="M520.0 284 L520.0 289" class="f-none s-muted" stroke-width="1.5"/><text x="520.0" y="304" class="t-small" text-anchor="middle">−2</text><path d="M556.4 284 L556.4 289" class="f-none s-muted" stroke-width="1.5"/><text x="556.4" y="304" class="t-small" text-anchor="middle">0</text><path d="M629.1 284 L629.1 289" class="f-none s-muted" stroke-width="1.5"/><text x="629.1" y="304" class="t-small" text-anchor="middle">4</text><path d="M720.0 284 L720.0 289" class="f-none s-muted" stroke-width="1.5"/><text x="720.0" y="304" class="t-small" text-anchor="middle">9</text><path d="M556.4 76 L556.4 284" class="f-none s-muted dash" stroke-width="1.5"/><path d="M543.6 100 L589.1 100" class="f-none s-muted" stroke-width="2"/><path d="M543.6 95 L543.6 105" class="f-none s-muted" stroke-width="2"/><path d="M589.1 95 L589.1 105" class="f-none s-muted" stroke-width="2"/><circle cx="567.3" cy="100" r="6" class="f-surface s-muted" stroke-width="2"/><path d="M585.5 150 L643.6 150" class="f-none s-muted" stroke-width="2"/><path d="M585.5 145 L585.5 155" class="f-none s-muted" stroke-width="2"/><path d="M643.6 145 L643.6 155" class="f-none s-muted" stroke-width="2"/><circle cx="614.5" cy="150" r="6" class="f-phys s-muted" stroke-width="1.5"/><path d="M650.9 200 L709.1 200" class="f-none s-muted" stroke-width="2"/><path d="M650.9 195 L650.9 205" class="f-none s-muted" stroke-width="2"/><path d="M709.1 195 L709.1 205" class="f-none s-muted" stroke-width="2"/><circle cx="680.0" cy="200" r="6" class="f-phys s-muted" stroke-width="1.5"/><path d="M581.8 250 L629.1 250" class="f-none s-muted" stroke-width="2"/><path d="M581.8 245 L581.8 255" class="f-none s-muted" stroke-width="2"/><path d="M629.1 245 L629.1 255" class="f-none s-muted" stroke-width="2"/><circle cx="605.5" cy="250" r="6" class="f-phys s-muted" stroke-width="1.5"/><text x="408" y="104" class="t-small">+2,9 [−8,7; 14,5]</text><text x="408" y="154" class="t-small">+26,9 [8,9; 44,8]</text><text x="408" y="204" class="t-small">+22,9 [2,0; 43,9]</text><text x="408" y="254" class="t-small">+23,3 [4,1; 42,5]</text><text x="730" y="104" class="t-small">+0,6 [−0,7; 1,8]</text><text x="730" y="154" class="t-small">+3,2 [1,6; 4,8]</text><text x="730" y="204" class="t-small">+6,8 [5,2; 8,4]</text><text x="730" y="254" class="t-small">+2,7 [1,4; 4,0]</text><text x="30" y="336" class="t-small">Chấm đặc = KTC không chứa 0 (phân biệt được); chấm rỗng = KTC chứa 0 (không phân biệt được).</text></svg>
</div>
<figcaption>Hình 2. Hiệu chain − không chain trên bộ thưa xe: chờ lâu hơn (vàng cam) đổi lấy nhiều chuyến hoàn thành hơn (xanh ngọc). Chain-60 không phân biệt được với baseline ở cả hai chỉ số; tầm nhìn dài hơn kéo thêm cả hai.</figcaption>
</figure>

Bốn điều đọc từ bảng và hình:

- **Chain giảm khách bị bỏ rơi nhưng khách điển hình chờ lâu hơn.** Chain-120 thêm 3,2 chuyến hoàn thành, tỉ lệ huỷ giảm 0,0139, nhưng `wait_s` tăng 26,9 s và `pickup_eta_s` tăng 34,7 s. Thiên lệch người sống sót không đủ giải thích khoản này. Thử tính: muốn riêng 3,2 chuyến thêm kéo trung bình từ 576,6 s lên 603,4 s thì mỗi chuyến đó phải chờ khoảng 1.864 s. Và **trung vị** `wait_s` cũng tăng 33,8 s [14,6; 53,0], điều mà vài chuyến thêm không làm được. Một giả thuyết khớp với code, chưa có thí nghiệm riêng: khi thiếu xe, baseline để khách chờ rồi mỗi lần có xe rảnh chọn khách rẻ nhất cho xe đó; chain gán khách vào xe sắp rảnh ngay lúc đặt, nên xe bị giữ chỗ sớm và mất quyền chọn khách gần hơn về sau.
- **Tầm nhìn dài giảm km rỗng của chuyến hoàn thành, đổi lại là thời gian chờ lâu hơn.** Chain-300 giảm `deadhead_m` 381,2 m và thêm 6,8 chuyến, nhưng `wait_s` vẫn tăng 22,9 s. Km rỗng tổng trên mỗi khách đặt giảm 151,5 m, tuy KTC của số đó [−308,0; 4,9] chứa 0, nên không khẳng định được.
- **Ở xe thưa, chain-60 không phân biệt được với baseline** ở cả `wait_s`, `deadhead_m` lẫn số chuyến. Điều này không có nghĩa nó vô tác dụng: 20 seed có thể chưa đủ để thấy mức chênh lệch cỡ đó.
- **Số chuyến phân bố giữa các tài xế lệch hơn một chút** khi tầm nhìn dài: chỉ số Gini số chuyến mỗi xe tăng 0,015 ở chain-300 (KTC [0,005; 0,025]). Lab không điều tra nguyên nhân; có thể là các xe nằm gần những nơi có nhiều khách xuống xe được nối chuyến nhiều hơn, nhưng đó mới là giả thuyết.

## Mật độ khác, và khi chuyến trước kéo dài

Chain không có tác dụng như nhau ở mọi mật độ. Cùng 20 seed, hiệu so với không chain (`*` = KTC không chứa 0):

| Bộ giả định (trên dữ liệu mô phỏng) | Cấu hình | `wait_s` TB (s) | `deadhead_m` TB (m) | Hoàn thành |
|---|---|---|---|---|
| «cao điểm»: 120 xe, 7,0 chuyến/phút | chain-60 | +9,9 [3,6; 16,1]* | −8,5 [−81,1; 64,1] | +4,7 [2,8; 6,6]* |
| | chain-120 | +16,7 [3,9; 29,5]* | −57,6 [−178,2; 63,0] | +6,8 [3,7; 9,8]* |
| | chain-300 | +19,5 [−5,0; 44,1] | −399,8 [−637,0; −162,7]* | +12,0 [9,3; 14,7]* |
| «thường»: 120 xe, 3,5 chuyến/phút | chain-120 | −3,1 [−5,9; −0,4]* | −96,3 [−128,3; −64,4]* | +1,2 [0,2; 2,2]* |
| | chain-300 | −3,2 [−6,5; 0,0] | −181,1 [−232,7; −129,4]* | +2,3 [1,3; 3,3]* |

Ở bộ «thường», xe không khan nên lợi ích nhỏ, và vẫn phải trả giá: `wait_s` giảm 3,1 s, `deadhead_m` giảm 96,3 m, km rỗng tổng mỗi khách giảm 74,6 m [−103,1; −46,1], nhưng Gini số chuyến tăng 0,013 [0,007; 0,019] và 3,6 trong 25,1 khách xếp hàng mỗi seed huỷ khi chờ. Ở «cao điểm», chain-60 thêm 4,7 chuyến, tỉ lệ huỷ −0,0105, đổi lấy `wait_s` +9,9 s và p95 +42,8 s [3,2; 82,3]. Chain-300 thêm 12,0 chuyến, giảm `deadhead_m` 399,8 m và km rỗng tổng 206,0 m [−384,4; −27,7], đổi lấy Gini +0,016 và 72,7 khách huỷ khi xếp hàng mỗi seed; `wait_s` p95 không phân biệt được (−9,2 s, KTC [−98,3; 79,8]). Lab không đặt tiêu chí chọn trước cho tầm nhìn, nên bài không tuyên bố cấu hình nào thắng. Bốn cấu hình nhân bốn bộ giả định và hơn mười cột tạo ra nhiều phép so sánh: dấu sao đơn lẻ sát 0 nên đọc dè dặt, mẫu hình lặp lại qua nhiều bộ thì đáng tin hơn.

**Chuyến trước kéo dài thất thường** là rủi ro cần thử. Ta cho nhiễu tốc độ σ = 0,3 (thay vì 0,1) ở bộ «thưa xe», tức chuyến thật dao động quanh kế hoạch rộng hơn. Baseline không chain có `wait_s` 605,3 (p95 1.435,7) ở bộ này.

| Cấu hình (σ = 0,3, thưa xe, trên dữ liệu mô phỏng) | `wait_s` TB (s) | `wait_s` p95 (s) | Hoàn thành |
|---|---|---|---|
| chain-120 | +29,9 [10,3; 49,6]* | +82,7 [2,3; 163,0]* | +4,5 [2,2; 6,9]* |
| chain-120+r1 | +30,0 [10,0; 50,1]* | +112,6 [13,6; 211,5]* | +3,9 [1,3; 6,5]* |

Phải đọc bảng này thật chậm. p95 tăng 82,7 s, nhưng cận dưới chỉ 2,3 s. So với chain-120 ở σ = 0,1 (p95 +69,7 s, KTC [−33,4; 172,7]) hai ước lượng gần như trùng nhau: một khoảng vượt 0 còn khoảng kia không, nhưng điều đó **không** có nghĩa nhiễu lớn làm chain tệ hơn, và lab không so trực tiếp hai mức σ. Biên `risk_k = 1` cho +112,6 s [13,6; 211,5], chồng lấn gần hết với +82,7 s: không thấy biên này giúp được gì, cũng chưa nói được nó làm tệ hơn. Điều đọc chắc được là từ code: cộng biên vào chi phí chỉ đổi người được chọn, không làm chuyến trước ngắn lại, nên khi chuyến trước kéo dài khách xếp hàng vẫn chờ hết nó. Muốn đo đúng rủi ro này cần chỉ số trực tiếp, như giờ xe tới thực so với giờ dự kiến của khách xếp hàng; lab chưa in chỉ số đó.

## Ai lợi, ai chịu thiệt, và điều chưa chạy

**Lợi:** khách suýt bị bỏ rơi. Ở bộ «thưa xe» số chuyến hoàn thành tăng, huỷ giảm, và khách hết kiên nhẫn giảm gần một nửa. **Thiệt:** khách điển hình chờ lâu hơn (`wait_s` +26,9 s, trung vị +33,8 s ở chain-120). Tài xế thì nhận được chuyến nối, nhưng ở tầm nhìn dài, số chuyến phân bố lệch hơn giữa các tài xế.

Những thứ **chưa chạy**, để không ai đọc nhầm là đã kiểm:

- Không phải mọi cấu hình đều chạy ở mọi bộ giả định: chain-60 và chain-300 không chạy ở σ = 0,3; chain-60 và `risk_k = 1` không chạy ở «thường»; «cao điểm» không chạy với σ = 0,3.
- Tài xế luôn nhận chuyến nối. Chưa thử trường hợp tài xế từ chối hoặc đổi ý khi chuyến trước kéo dài, và chưa thử chain cho chuyến ghép chung.
- ETA của bộ ghép là ETA CH theo nút, không có chi phí rẽ, trong khi simulator có (bài 23). Chưa thử giao thông tổng hợp theo giờ.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin`; `--fixture` chạy offline trên đồ thị tổng hợp để test. Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc47_chain_dispatch
```

Tham số: `--seeds N` (mặc định 20), `--jobs N`, `--fixture`. Lab chạy khoảng 28 phút trên máy của người viết bài khi `uptime` ghi load average trên 400; số đếm, tỉ lệ và hiệu có cặp tái lập được, còn thời gian chạy thì không đáng tin. Kết quả chạy của bộ «thưa xe» (nguyên văn từ `results/p05.md`):

```text
=== bộ giả định «thưa xe»: 60 xe, 3,5 chuyến/phút ===
  không chain (tuyệt đối, TB 20 seed): khách đặt 228.4, hoàn thành 150.5, cancel 0.3, unserved 0.1, wait_s 576.6 (p95 1297.4), pickup_eta_s 555.9, deadhead_m 4689.1
  hiệu = chain − không chain (Hungarian ghép ngay), có cặp theo seed, KTC 95%; * = KTC không chứa 0
  cấu hình                xếp hàng huỷ khi chờ            wait_s TB             wait_s p95             wait_s max          deadhead_m TB        cancel_rate           hoàn thành
  chain-60                    66.3      13.1   +2.9 [-8.7, 14.5]   -45.8 [-115.1, 23.4]  -146.1 [-339.9, 47.8]   -74.1 [-185.3, 37.2]  -0.0 [-0.0, 0.0]     +0.6 [-0.7, 1.8] 
  chain-120                   89.3      27.9   +26.9 [8.9, 44.8]*  +69.7 [-33.4, 172.7]   +5.0 [-147.3, 157.4]  +11.1 [-192.5, 214.8]  -0.0 [-0.0, -0.0]*     +3.2 [1.6, 4.8]*
  chain-300                  106.2      39.8   +22.9 [2.0, 43.9]*    -1.7 [-76.2, 72.9]  -127.1 [-321.1, 66.9]  -381.2 [-578.4, -184.1]* -0.0 [-0.0, -0.0]*     +6.8 [5.2, 8.4]*
  chain-120+r1                86.8      26.6   +23.3 [4.1, 42.5]*  +41.1 [-51.8, 134.0]   -45.9 [-172.7, 80.9]   -3.7 [-216.4, 208.9]  -0.0 [-0.0, -0.0]*     +2.7 [1.4, 4.0]*
  (xếp hàng, huỷ khi chờ: trung bình mỗi seed)
  cấu hình                    pickup_eta_s TB      km rỗng/khách (m)         Gini chuyến/xe    rảnh dài nhất p95 (s)
  chain-60                 +7.5 [-4.4, 19.4]   -54.2 [-130.3, 22.0]  +0.005 [-0.001, 0.012]          -196 [-386, -7]*
  chain-120               +34.7 [15.0, 54.3]*  +49.3 [-72.8, 171.5]  +0.002 [-0.006, 0.010]          -14 [-182, 154] 
  chain-300               +40.1 [17.1, 63.1]*  -151.5 [-308.0, 4.9]  +0.015 [0.005, 0.025]*         -24 [-241, 193] 
  chain-120+r1            +32.0 [11.0, 52.9]* +26.6 [-106.0, 159.2]  +0.003 [-0.005, 0.011]         -219 [-425, -13]*
```

`bash examples/toi-noi-location/verify-p05.sh` chạy lại lab với `--fixture --seeds 2`, không phải 20 seed trên HCM.

## Giới hạn của ví dụ

- **Mọi thứ là mô phỏng.** Nhu cầu, huỷ, nhiễu tốc độ, hành vi tài xế đều là giả định của kịch bản; không có số nào nói về một công ty thật.
- **Chiến lược không dùng mô hình huỷ.** Chi phí ở đây chỉ là ETA cộng `lead_ms` (`risk_ms = 0`), nên việc khách xếp hàng dễ huỷ hơn không được đưa vào quyết định. Huỷ khi xếp hàng là điều simulator đo được, không phải điều chiến lược cân nhắc.
- **Hai mức nhiễu tốc độ** (σ = 0,1 và σ = 0,3). Không biết chuyến thật có kéo dài giống log-chuẩn hay không; kẹt xe, tai nạn, khách đổi điểm trả không có trong mô hình.
- **Cỡ mẫu là 20 seed**, nên KTC rộng ở nhiều chỉ số đuôi (`wait_s` p95 và max đều không phân biệt được ở nhiều dòng). Những chỗ ghi «không phân biệt được» đừng đọc thành «không có tác dụng».

## Bài tập

**Bài 1.** Một tài xế còn 300 s nữa là trả khách. Tính một độ lệch chuẩn của phần còn lại cho σ = 0,1 và σ = 0,3. Với `risk_k = 1`, chi phí của tài xế đó tăng thêm bao nhiêu giây ở mỗi mức?

<details><summary>Lời giải bài 1</summary>
<p>sd ≈ 300 × (e^σ − 1). Với σ = 0,1: 300 × 0,10517 ≈ 31,6 s. Với σ = 0,3: 300 × 0,34986 ≈ 105,0 s. Với <code>risk_k = 1</code> chi phí cặp tăng đúng chừng đó giây, nên biên rủi ro lớn gấp 3,3 lần trong mô phỏng nhiễu hơn, trong khi chuyến thật cũng dài thất thường hơn.</p>
</details>

**Bài 2.** Chain-120 làm `wait_s` giảm 3,1 s ở bộ «thường» nhưng tăng 26,9 s ở bộ «thưa xe». Hãy nêu hai lý do, rồi chỉ ra chỉ số nào trong bảng cho thấy lý do thứ hai.

<details><summary>Lời giải bài 2</summary>
<p>Một là ở «thường» xe không khan, nên ít khách xếp hàng (25,1 mỗi seed so với 89,3 ở thưa xe) và ít phải đợi chuyến trước. Hai, một giả thuyết: ở «thường» mọi lô chỉ có một khách nên baseline chẳng có quyền chọn gì để mất, còn ở thưa xe chain lấy mất quyền chọn khách rẻ nhất khi xe vừa rảnh. Chỉ số ủng hộ lý do hai là <code>pickup_eta_s</code> +34,7 s và trung vị <code>wait_s</code> +33,8 s: cả phân phối dịch sang phải, không chỉ thêm vài khách chờ lâu vào mẫu. Lab chưa thí nghiệm riêng để xác nhận.</p>
</details>

**Bài 3.** Đề xuất một chỉ số bổ sung để người đọc không nhầm «`wait_s` tăng» với «khách thật sự bị thiệt», và nói vì sao KTC của `deadhead_m` ở chain-120 chứa 0 mà ở chain-300 thì không.

<details><summary>Lời giải bài 3</summary>
<p>Có thể báo `wait_s` tính trên mọi khách đặt, gán thời gian chờ của khách bị bỏ rơi bằng hạn kiên nhẫn, hoặc báo riêng nhóm khách chỉ được phục vụ nhờ chain. Về `deadhead_m`: một cách đọc là với tầm nhìn 120 s, số khách xếp hàng ít hơn (89,3 so với 106,2 mỗi seed) nên tác động lên km rỗng nhỏ so với nhiễu giữa các seed (hiệu +11,1 m, KTC [−192,5; 214,8]), còn tầm nhìn 300 s tác động đủ lớn để vượt khỏi nhiễu (−381,2 m, KTC [−578,4; −184,1]). Lab không kiểm giả thuyết này riêng.</p>
</details>

Bài trước: [bài 46](/articles/location-goi-y-sang-dispatch-csharp-grpc-lease-idempotency). Bài tiếp: [bài 48](/articles/location-lech-tuyen-dinh-tuyen-lai-ghep-lai).

## Tài liệu gốc

- [Hungarian algorithm (Wikipedia)](https://en.wikipedia.org/wiki/Hungarian_algorithm): bài toán phân công mà bộ ghép giải trên ma trận có thêm cột tài xế bận.
- [Log-normal distribution (Wikipedia)](https://en.wikipedia.org/wiki/Log-normal_distribution): trung vị, phân vị và độ lệch chuẩn của phân phối log-chuẩn, nền của xấp xỉ dùng cho `risk_k`.
- [Survivorship bias (Wikipedia)](https://en.wikipedia.org/wiki/Survivorship_bias): thiên lệch người sống sót khi chỉ đo chuyến hoàn thành.
