---
slug: location-chi-phi-ghep-cong-bang-tai-xe-rui-ro-huy
title: "Chi phí ghép ngoài quãng đường: công bằng cho tài xế và rủi ro hủy"
summary: "Bài 45: thêm rủi ro hủy, thưởng chờ, thưởng xe rảnh và giới hạn đón xa vào chi phí ghép. Khi ghép ngay, hai số hạng không có gì để tác động; giới hạn đón xa làm đẹp số trung bình, ngoại vi hoàn thành −0,031. Trên dữ liệu mô phỏng."
category: system-design
level: advanced
tags: [toi-noi, location, rust, dispatch, matching, fairness, policy, cancellation]
language: vi
---

Hãy hình dung (tình huống minh họa, không phải dữ liệu): tài xế DR-040 vừa trả khách xong ở rìa thành phố và đã đứng đó bốn mươi phút. Cùng lúc, một khách ở rìa bên kia đặt xe, ETA tới DR-040 là 11 phút. Một hệ thống chỉ biết ETA sẽ để DR-040 đứng tiếp và đưa khách cho một tài xế khác vừa bắt khách ở gần đó, vì chi phí chỉ là số giây. Một dispatcher tử tế sẽ muốn chi phí biết thêm điều gì đó: tài xế này đã rảnh quá lâu, khách kia đã chờ quá lâu, đón xa quá 10 phút thì khách hay hủy. Bài này thêm các số hạng đó vào hàm chi phí, đặt trọng số trước khi đo, và đo cái gì thật sự đổi.

Kết quả không như một bản ghi chú phát hành. Hai trong bốn số hạng cho nhật ký chuyến **giống hệt** baseline khi ghép ngay, và lý do là cách thí nghiệm được dựng: chúng chưa hề được thử. Khi cho chúng cơ hội, trong lô 10 s ở cao điểm, chúng cũng gần như không đổi gì. Số hạng thứ ba làm tài xế công bằng hơn bằng cách bắt khách chờ lâu hơn. Số hạng thứ tư, giới hạn đón xa, làm các con số trung bình đẹp lên và đẩy thiệt hại vào khách ở xa trung tâm.

Đây là bài 45 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi), bài cuối của chuỗi trục ghép chuyến lõi (bài 41–45). Bài dùng cách đọc so sánh có cặp 20 seed của [bài 41](/articles/location-gan-nhat-sai-o-dau-do-bang-mo-phong), phép gán Hungarian của [bài 42](/articles/location-ghep-theo-lo-hungarian-kuhn-munkres-rust), và đo lại khoản khách bị bỏ rơi mà bài 42 phát hiện. Mọi số đo là **trên dữ liệu mô phỏng**, đồ thị HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors), 20 seed đánh giá 20261106…20261125. Công bằng cho tài xế đo bằng số chuyến và khoảng rảnh, **không phải thu nhập thật**.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Thêm vào chi phí ghép một số hạng «xác suất khách hủy × 600 s» (khách hủy thì mất 10 phút tương đương). Theo bạn việc này làm đổi những cặp được ghép đến mức nào? Và nếu mô hình hủy mà chính sách tin là sai, hậu quả ra sao?</p>
<details><summary>Xem đáp án</summary><p>Không đổi gì cả. Nhật ký chuyến giống <b>hệt</b> baseline (hiệu 0,0 ở mọi chỉ số, khoảng [0,0; 0,0]), ở mọi bộ giả định đã chạy, kể cả khi simulator dùng mô hình hủy khác với mô hình chính sách tin. Lý do: khi ghép ngay, mỗi lô chỉ có một khách hoặc chỉ có một xe, và xác suất hủy chỉ phụ thuộc ETA và tăng theo ETA, nên cộng thêm nó không đổi lựa chọn nào. Điều đó nói về cách thí nghiệm được dựng, không nói rằng số hạng này an toàn để triển khai.</p></details>
</div>

## Hàm chi phí có phiên bản

Chi phí của cặp (khách `j`, tài xế `i`) tính bằng mili giây «tương đương thời gian đón», mỗi số hạng một lý do:

`examples/toi-noi-location/rust/crates/matcher/src/policy.rs`

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CostPolicy {
    pub version: String,
    pub w_eta: f64,
    pub soft_pickup_ms: i64,
    pub soft_w: f64,
    /// Phạt (ms) cho một lần huỷ; 0 = không dùng xác suất huỷ.
    pub risk_ms: f64,
    pub age_bonus: f64,
    pub idle_bonus: f64,
    /// `None` = không giới hạn.
    pub hard_pickup_ms: Option<i64>,
}
```

`examples/toi-noi-location/rust/crates/matcher/src/policy.rs`

```rust
    pub fn cost(&self, eta_ms: i64, wait_ms: i64, idle_ms: i64, p_cancel: f64) -> Option<i64> {
        if self.hard_pickup_ms.is_some_and(|h| eta_ms > h) {
            return None;
        }
        let over = (eta_ms - self.soft_pickup_ms).max(0) as f64;
        let c = self.w_eta * eta_ms as f64 + self.soft_w * over + self.risk_ms * p_cancel
            - self.age_bonus * wait_ms as f64
            - self.idle_bonus * idle_ms as f64;
        Some(c.round() as i64)
    }
```

- `w_eta · ETA`: thời gian tài xế tới điểm đón, như mọi bài trước.
- `soft_w · max(0, ETA − mềm)`: đón xa bị phạt thêm phần vượt ngưỡng mềm.
- `risk_ms · P(hủy)`: xác suất hủy là **đầu vào** lấy từ mô hình hủy của kịch bản, nhân với «giá» một lần hủy.
- `− age_bonus · (khách đã chờ)`: khách chờ lâu được ưu tiên khi thiếu xe.
- `− idle_bonus · (tài xế đã rảnh)`: tài xế rảnh lâu được ưu tiên khi thừa xe.
- Cặp bị cấm nếu `ETA > hard_pickup_ms`: giới hạn đón xa **cứng**.

Xác suất hủy lấy từ chính công thức của simulator ([bài 30](/articles/location-tong-hop-mo-phong-su-kien-roi-rac-seed)). Chiến lược «biết» mô hình, điều này làm thí nghiệm có lợi cho số hạng rủi ro; phần dưới có thí nghiệm khi chiến lược tin sai.

`examples/toi-noi-location/rust/crates/matcher/src/batch.rs`

```rust
        let cost_of = |j: usize, i: usize, e: i64| -> Option<i64> {
            if e >= FORBIDDEN / 2 {
                return None;
            }
            let p = &pending[j];
            let c = &cands[i];
            let wait_ms = ctx.now_ms.saturating_sub(p.request_ms) as i64;
            let total = e + c.lead_ms;
            // khi mô hình huỷ tính theo tổng thời gian chờ, thời gian đã chờ cũng là đầu vào của xác suất huỷ
            let basis_ms = match model.basis {
                CancelBasis::Eta => total,
                CancelBasis::TotalWait => total + wait_ms,
            };
            let pc = model.probability(basis_ms as f64 / 1000.0, p.pickup.walk_m);
            self.cfg.policy.cost(total, wait_ms, c.idle_ms, pc)
        };
```

Sáu chính sách, trọng số **đặt trước khi đo**, in ở đầu output, không chỉnh theo kết quả, và **mọi** chính sách đều được báo cáo, kể cả chính sách xấu:

`examples/toi-noi-location/rust/labs/src/bin/loc45_chi_phi_cong_bang_rui_ro_huy.rs`

```rust
        (
            "v1-age (+ thưởng chờ lâu 0,5)",
            mk("v1-age", &|p| p.age_bonus = 0.5),
        ),
        (
            "v1-idle (+ thưởng xe rảnh lâu 0,1)",
            mk("v1-idle", &|p| p.idle_bonus = 0.1),
        ),
        (
            "v1-limit (đón xa: mềm 300 s ×1, cứng 600 s)",
            mk("v1-limit", &|p| {
                p.soft_pickup_ms = 300_000;
                p.soft_w = 1.0;
                p.hard_pickup_ms = Some(600_000);
            }),
        ),
```

`v1-all` gộp mọi số hạng: rủi ro hủy ×600 s, thưởng chờ 0,5, thưởng rảnh 0,1, ngưỡng mềm 300 s, ngưỡng cứng 600 s. Đây là điều §3.4 của tài liệu lộ trình (`docs/location-master-curriculum.md`) đã đặt ra: bộ trọng số là một **chính sách có `policy_version`**, lưu cùng quyết định để sau này tái lập và so sánh. Trong lab, `version` (`v0-eta`, `v1-risk`, …) đi cùng mọi bảng kết quả. Tất cả chạy với Hungarian ghép ngay (không cửa sổ), 20 seed, ba bộ giả định: `thường` (120 xe, 3,5 khách/phút), `thưa xe` (60 xe), `huỷ nặng` (hủy tối đa 0,8 từ ETA 120 s).

## Hai số hạng cho kết quả y hệt: không phải lỗi nối dây, mà là chưa được thử

Ở bộ `thường`, `v1-risk` và `v1-age` cho hiệu **đúng 0,0** ở mọi chỉ số, khoảng tin cậy [0,0; 0,0], kể cả chỉ số công bằng (Gini, khoảng rảnh) và chỉ số theo vùng. Nhật ký chuyến không đổi một dòng. Một kết quả giống hệt đến từng byte thì phải nghi trước hết là lỗi nối dây, nên lab kiểm hai điều: hình dạng của các lô, và số hạng có đổi kết quả không khi lô có nhiều khách, nhiều xe. Hai cách đọc cần tránh là «lab hỏng» và «số hạng này vô hại, cứ triển khai».

Cờ `--hinh-dang-lo` bọc chiến lược để đếm mọi lần ghép, 20 seed, Hungarian ghép ngay, trên dữ liệu mô phỏng:

| Bộ giả định | Lô 1 khách × m xe | Lô n ≥ 2 khách × 1 xe | Lô n ≥ 2 khách × m ≥ 2 xe | Khách của lô 1 × m đã chờ tối đa |
|---|---:|---:|---:|---:|
| `thường` | 4.568 | 0 | 0 | 0 ms |
| `huỷ nặng` | 4.568 | 0 | 0 | 0 ms |
| `thưa xe` | 3.298 | 927 | 0 | 299.921 ms |
| `cao điểm` | 6.736 | 1.665 | 0 | 286.750 ms |

Không có lô nào vừa nhiều khách vừa nhiều xe, và đó là tất yếu: khi ghép ngay, mỗi sự kiện chỉ thêm một khách hoặc một xe, và mọi cặp ghép được đã ghép xong từ sự kiện trước. Mọi quyết định vì thế chỉ là chọn phần tử nhỏ nhất của một hàng hoặc một cột.

**`v1-age` nhận đầu vào bằng 0.** Số hạng `−age_bonus · wait` chỉ phụ thuộc hàng (khách), nên trong một lô 1 × m nó không đổi được xe nào. Nó chỉ có tác dụng khi phải chọn **khách nào** được phục vụ, tức lô n × 1. Ở `thường` lô đó không tồn tại, và mọi khách đều được ghép ngay lúc đặt: thời gian đã chờ luôn là 0 ms, nên `age_bonus × 0 = 0`. Ở `thưa xe` có 927 lô n × 1, và `v1-age` đổi nhẹ (`wait_s` +2,8 s [−0,3; 5,8], không phân biệt được).

**`v1-risk` là một hàm tăng của ETA.** Mô hình hủy của kịch bản có `walk_extra_per_100m = 0`: xác suất hủy chỉ phụ thuộc ETA, và tăng (rồi phẳng) theo ETA.

`examples/toi-noi-location/rust/crates/simulator/src/scenario.rs`

```rust
    pub fn probability(&self, eta_s: f64, walk_m: f64) -> f64 {
        let ramp = ((eta_s - self.eta_free_s) / (self.eta_max_s - self.eta_free_s)).clamp(0.0, 1.0);
        (self.max_prob * ramp + self.walk_extra_per_100m * walk_m / 100.0).clamp(0.0, 1.0)
    }
```

Cộng `risk_ms · P(ETA)` vào `ETA` cho một hàm **tăng nghiêm ngặt** của ETA, và hàm đó giống nhau cho mọi ô. Trong lô 1 × m, tài xế có chi phí nhỏ nhất vẫn là tài xế có ETA nhỏ nhất. Trong lô n × 1, khách có chi phí nhỏ nhất vẫn là khách có ETA nhỏ nhất. Vậy với mọi seed và mọi đường cong hủy chỉ phụ thuộc ETA, `v1-risk` khi ghép ngay **không thể** khác `v0-eta`. Kết quả ở thí nghiệm «tin sai mô hình» (simulator dùng `huỷ nặng`, chính sách tin mô hình mặc định, hiệu vẫn 0,0) cũng từ đó mà ra: mô hình sai nhưng vẫn là một hàm tăng của ETA, nên thí nghiệm đó không kiểm được gì.

**Điều này có nghĩa là gì.** Ở cấu hình ghép ngay, hai biến thể này **chưa được thử**. Không phải «an toàn khi triển khai», cũng không phải lỗi: nối dây đúng, vì mục dưới cho thấy chúng đổi kết quả ngay khi lô có nhiều khách, nhiều xe. Số hạng rủi ro còn có thể đổi quyết định khi `P(hủy)` phụ thuộc điều gì đó mà ETA không biết: quãng khách phải đi bộ (hệ số `walk_extra_per_100m` khác 0), đặc điểm của từng khách hay tài xế, giờ trong ngày. Lab chưa thử bất kỳ mô hình nào như vậy.

### Thử ở nơi có dữ liệu: lô 10 s ở cao điểm

Bài 42 đặt câu hỏi cho chính bài này: thưởng chờ lâu có chữa được khoản khách bị bỏ rơi của Hungarian không? Cờ `--thu-lo` chạy mọi chính sách với cửa sổ lô 10 s ở `cao điểm`, đúng nơi vấn đề xuất hiện, so với `v0-eta` cùng cửa sổ (`deadhead_m` 3.917,9 m, hết kiên nhẫn 0,0717, hoàn thành 313,9, km rỗng/khách 2.771,6 m), 20 seed, trên dữ liệu mô phỏng:

| Hiệu so với `v0-eta` | `v1-risk` | `v1-age` | `v1-limit` | `v1-all` |
|---|---|---|---|---|
| `deadhead_m` TB (m) | +1,4 [−45,5; 48,3] | −6,1 [−25,4; 13,2] | **−1.187,3 [−1.301,3; −1.073,4]** | **−1.194,1 [−1.300,6; −1.087,6]** |
| `wait_s` TB (s) | 0,0 [−4,6; 4,6] | +2,1 [−1,1; 5,3] | **−102,8 [−113,5; −92,1]** | **−98,5 [−108,1; −88,8]** |
| hết kiên nhẫn | +0,0036 [−0,0003; 0,0076] | −0,0025 [−0,0050; −0,0000] | **+0,1256 [0,1055; 0,1458]** | **+0,1192 [0,0982; 0,1403]** |
| hoàn thành | −0,1 [−1,3; 1,1] | +0,4 [−0,7; 1,5] | **−8,9 [−13,1; −4,8]** | **−9,2 [−14,3; −4,1]** |
| km rỗng tổng/khách (m) | −0,4 [−31,2; 30,5] | +0,6 [−19,2; 20,4] | **−902,4 [−999,0; −805,8]** | **−909,3 [−1.006,0; −812,6]** |

`v1-risk` và `v1-age` giờ khác `v0-eta` ở gần như mọi seed, tức nối dây đúng, nhưng không có hiệu nào đáng kể. Khoản −0,0025 của `v1-age` có cận trên sát 0 trong một bảng rất nhiều ô, và dù có thật thì cũng nhỏ so với +0,0568 mà Hungarian đã gây ra. Với trọng số 0,5, khách chờ đủ 300 s chỉ được thưởng 150 s, trong khi ETA trung bình 482,5 s; trọng số lớn hơn thì chưa thử. Giới hạn đón xa thì ngược hẳn: km rỗng giảm mạnh, số khách bị bỏ rơi tăng gần gấp ba, và 8,9 chuyến biến mất. Theo vùng, thiệt hại ở đây rơi vào trung tâm (hoàn thành 0,710 → 0,665) và vành trong (0,707 → 0,677), còn ngoại vi không phân biệt được (0,673 → 0,691).

## Thưởng xe rảnh: công bằng cho tài xế, khách chờ lâu hơn

`v1-idle` (thưởng 0,1 cho mỗi giây tài xế đã rảnh) có tác dụng. Ở bộ `thường`, so với `v0-eta` (Gini số chuyến mỗi tài xế 0,257):

| Chỉ số | Hiệu `v1-idle − v0-eta` [KTC 95%] | Nhận xét |
|---|---|---|
| Gini số chuyến/xe | −0,039 [−0,050; −0,029] | đều hơn |
| Tỉ lệ tài xế không có chuyến nào | −0,033 [−0,042; −0,025] | ít người bị bỏ không hơn |
| Khoảng rảnh liên tiếp dài nhất, p95 (s) | −1.052 [−1.495; −609] | ngắn hơn |
| `pickup_eta_s` trung bình (s) | +6,4 [2,9; 9,8] | khách đợi lâu hơn |
| `deadhead_m` trung bình (m) | +54,9 [23,8; 85,9] | km rỗng nhiều hơn |
| Hoàn thành | −0,8 [−1,7; 0,2] | không phân biệt được |

**Ai lợi, ai thiệt:** tài xế rảnh lâu có thêm chuyến (lợi), khách bị giao tài xế xa hơn một chút, đợi thêm 6,4 s, và km rỗng tổng tăng (thiệt). Theo vùng, khách vành trong chờ +7 s và ngoại vi +8 s (có ý nghĩa), trung tâm +3 s (không). Đổi lại trung bình 6,4 s chờ thêm, Gini giảm 0,039. Nên nhớ nó đo **số chuyến**, không phải thu nhập: một tài xế nhiều chuyến ngắn chưa chắc kiếm nhiều hơn một tài xế ít chuyến dài. Và ở `thưa xe` (ít tài xế rảnh) `v1-idle` không có tác dụng đo được (Gini +0,001 [−0,003; 0,005]): thưởng rảnh cần có dư xe để chọn.

## Giới hạn đón xa: số trung bình đẹp lên, ngoại vi trả giá

`v1-limit` phạt đón xa mềm từ 300 s (hệ số 1) và cấm cứng quá 600 s. Ở bộ `thường`, so với `v0-eta`:

| Chỉ số | `v1-limit` | `v1-all` |
|---|---|---|
| `pickup_eta_s` trung bình (s) | **−18,5 [−25,3; −11,7]** | **−14,8 [−21,6; −8,1]** |
| `wait_s` p95 (s) | **−56,8 [−85,2; −28,4]** | **−65,5 [−95,4; −35,6]** |
| `deadhead_m` trung bình (m) | **−181,0 [−255,1; −106,9]** | **−143,9 [−213,9; −73,9]** |
| Chuyến hoàn thành | −0,9 [−2,3; 0,4] | **−2,1 [−3,6; −0,7] XẤU HƠN** |
| Hủy (`cancel_rate`) | +0,0041 [−0,0020; 0,0102] | **+0,0097 [0,0032; 0,0162] XẤU HƠN** |
| Khách hết kiên nhẫn | không in riêng | **0,0000 → 0,0287 [0,0210; 0,0365] XẤU HƠN** |
| Gini số chuyến/xe | **+0,013 [0,006; 0,021]** | −0,013 [−0,023; −0,002] |

Nhìn riêng nửa trên, `v1-limit` là chính sách thắng: ETA, chờ ở đuôi, km rỗng đều giảm rõ. Nhưng các chỉ số này (`pickup_eta_s`, `wait_s`, `deadhead_m`) được tính **trên chuyến hoàn thành**. Một chính sách bỏ rơi khách ở xa làm các số này đẹp lên mà không giúp ai: những khách khó nhất không còn trong mẫu. Bảng theo vùng cho thấy ai bị bỏ:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 462" role="img" aria-label="Hình 1. Hiệu tỉ lệ khách được phục vụ theo vùng so với v0-eta, bộ thường, 20 seed. v1-idle: trung tâm trừ 0,003, vành trong trừ 0,001, ngoại vi trừ 0,009. v1-limit: trung tâm cộng 0,016, vành trong 0,000, ngoại vi trừ 0,027 có ý nghĩa. v1-all: trung tâm cộng 0,004, vành trong trừ 0,003, ngoại vi trừ 0,031 có ý nghĩa."><rect x="20" y="20" width="840" height="437" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Tỉ lệ khách hoàn thành theo vùng: chính sách nào đẩy thiệt hại ra ngoại vi</text><text x="40" y="90" class="t-b">v1-idle (thưởng xe rảnh lâu)</text><text x="52" y="119">trung tâm (&lt; 3 km)</text><rect x="504.4" y="107" width="15.6" height="16" rx="2" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="496.4" y="119" class="t-mono" text-anchor="end">−0,003</text><text x="52" y="143">vành trong (3–6 km)</text><rect x="514.8" y="131" width="5.2" height="16" rx="2" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="506.8" y="143" class="t-mono" text-anchor="end">−0,001</text><text x="52" y="167">ngoại vi (≥ 6 km)</text><rect x="473.2" y="155" width="46.8" height="16" rx="2" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="465.2" y="167" class="t-mono" text-anchor="end">−0,009</text><text x="40" y="190" class="t-b">v1-limit (giới hạn đón xa)</text><text x="52" y="219">trung tâm (&lt; 3 km)</text><rect x="520" y="207" width="83.2" height="16" rx="2" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="611.2" y="219" class="t-mono">+0,016</text><text x="52" y="243">vành trong (3–6 km)</text><rect x="520" y="231" width="1.0" height="16" rx="2" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="528.0" y="243" class="t-mono">+0,000</text><text x="52" y="267">ngoại vi (≥ 6 km)</text><rect x="379.6" y="255" width="140.4" height="16" rx="2" class="f-warn s-warn" stroke-width="1.5"/><text x="371.6" y="267" class="t-mono" text-anchor="end">−0,027*</text><text x="40" y="290" class="t-b">v1-all (mọi số hạng)</text><text x="52" y="319">trung tâm (&lt; 3 km)</text><rect x="520" y="307" width="20.8" height="16" rx="2" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="548.8" y="319" class="t-mono">+0,004</text><text x="52" y="343">vành trong (3–6 km)</text><rect x="504.4" y="331" width="15.6" height="16" rx="2" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="496.4" y="343" class="t-mono" text-anchor="end">−0,003</text><text x="52" y="367">ngoại vi (≥ 6 km)</text><rect x="358.8" y="355" width="161.2" height="16" rx="2" class="f-warn s-warn" stroke-width="1.5"/><text x="350.8" y="367" class="t-mono" text-anchor="end">−0,031*</text><path d="M520 76 L520 376" class="f-none s-muted" stroke-width="1.5"/><text x="520" y="68" class="t-small" text-anchor="middle">0</text><text x="508" y="68" class="t-small" text-anchor="end">← ít khách được phục vụ hơn</text><text x="532" y="68" class="t-small">nhiều hơn →</text><text x="40" y="426" class="t-small">Seed 20261106…25, Hungarian ghép ngay. * = KTC 95% không chứa 0; thanh nhạt = không phân biệt được.</text><text x="40" y="442" class="t-small">Vùng theo khoảng cách từ điểm ghim tới một tâm quy chiếu của lab. Trên dữ liệu mô phỏng.</text></svg>
</div>
<figcaption>Hình 1. Hiệu tỉ lệ khách hoàn thành theo vùng so với `v0-eta`, bộ thường. Giới hạn đón xa (`v1-limit`) và `v1-all` đều giảm tỉ lệ hoàn thành của khách ngoại vi có ý nghĩa (−0,027 và −0,031), trong khi trung tâm không đổi hoặc nhỉnh lên. Trên dữ liệu mô phỏng, 20 seed 20261106…25.</figcaption>
</figure>

Ở `v1-all`, tỉ lệ hoàn thành của khách ngoại vi đi từ 0,854 xuống 0,824 (−0,031, có ý nghĩa), trung tâm 0,878 → 0,882, vành trong 0,875 → 0,872. **Một chính sách cải thiện số trung bình toàn cục và làm kém đi đúng nhóm khách ở xa trung tâm.** Những khách ngoại vi vẫn được phục vụ lại được chờ ít hơn (−18,3 s) và tài xế chạy rỗng ít hơn (−235,8 m): đó chính là thiên lệch người sống sót, vì các chuyến ETA dài nhất đã bị loại ngay từ đầu.

Đây là điểm công bằng giữa các khu vực, không chỉ là chuyện kỹ thuật. Vùng ở đây định nghĩa theo khoảng cách từ điểm ghim tới một tâm quy chiếu cố định của lab (trung tâm dưới 3 km, vành trong 3–6 km, ngoại vi từ 6 km), không phải một khu vực hành chính cụ thể nào, và dữ liệu là mô phỏng. Cơ chế hợp lý là nơi xe ít hơn thì ETA dài hơn, vượt ngưỡng cứng nhiều hơn, bị cấm nhiều hơn; lab không đếm số cặp bị cấm theo vùng nên đây vẫn là suy luận. Hàm chi phí không nhận vùng làm đầu vào, vậy mà kết quả vẫn lệch theo vùng: một quy tắc đồng đều về hình thức có thể có hậu quả không đồng đều. Vì vậy chỉ số theo vùng cần là chỉ số bảo vệ cố định trong mọi thí nghiệm ghép, không phải phần phụ lục. Dữ liệu là mô phỏng nên bài không nói gì về một khu vực thật nào.

### Khi xe thưa, hiệu ứng khuếch đại, và không đi theo một hướng

Ở `thưa xe` (60 xe), `v1-limit` và `v1-all` giảm `wait_s` khoảng 150 s và `deadhead_m` khoảng 1.710–1.743 m. Đó là con số đẹp nhất của bài, và là con số cần nghi ngờ nhất:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Hình 2. Cùng chính sách v1-all so với v0-eta ở hai mật độ. Bộ thường: deadhead_m trừ 143,9 mét, khách hết kiên nhẫn cộng 0,0287, chuyến hoàn thành trừ 2,1, Gini số chuyến mỗi tài xế trừ 0,013, khoảng rảnh dài nhất lớn nhất trừ 270 giây. Bộ thưa xe: deadhead_m trừ 1.709,8 mét, khách hết kiên nhẫn cộng 0,1708, chuyến hoàn thành trừ 6,9, Gini cộng 0,057, khoảng rảnh dài nhất lớn nhất cộng 1.904 giây."><rect x="20" y="20" width="840" height="375" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Chính sách v1-all so với v0-eta: lợi và thiệt ở hai mật độ</text><text x="52" y="82" class="t-b">chỉ số (hiệu v1-all − v0-eta)</text><text x="360" y="82" class="t-b">«thường» (120 xe)</text><text x="600" y="82" class="t-b">«thưa xe» (60 xe)</text><text x="52" y="126">deadhead_m trung bình (m)</text><rect x="360" y="102" width="200" height="40" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="460" y="128" class="t-mono t-phys" text-anchor="middle">−143,9</text><rect x="600" y="102" width="200" height="40" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="700" y="128" class="t-mono t-phys" text-anchor="middle">−1.709,8</text><text x="52" y="178">khách hết kiên nhẫn (tỉ lệ)</text><rect x="360" y="154" width="200" height="40" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="460" y="180" class="t-mono t-warn" text-anchor="middle">+0,0287</text><rect x="600" y="154" width="200" height="40" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="700" y="180" class="t-mono t-warn" text-anchor="middle">+0,1708</text><text x="52" y="230">chuyến hoàn thành (mỗi seed)</text><rect x="360" y="206" width="200" height="40" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="460" y="232" class="t-mono t-warn" text-anchor="middle">−2,1</text><rect x="600" y="206" width="200" height="40" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="700" y="232" class="t-mono t-warn" text-anchor="middle">−6,9</text><text x="52" y="282">Gini số chuyến mỗi tài xế</text><rect x="360" y="258" width="200" height="40" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="460" y="284" class="t-mono t-phys" text-anchor="middle">−0,013</text><rect x="600" y="258" width="200" height="40" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="700" y="284" class="t-mono t-warn" text-anchor="middle">+0,057</text><text x="52" y="334">khoảng rảnh dài nhất, max (s)</text><rect x="360" y="310" width="200" height="40" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="460" y="336" class="t-mono t-phys" text-anchor="middle">−270</text><rect x="600" y="310" width="200" height="40" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="700" y="336" class="t-mono t-warn" text-anchor="middle">+1.904</text><text x="40" y="378" class="t-small">Seed 20261106…25. Xanh ngọc = có lợi, vàng cam = bất lợi. Mọi ô đều có KTC 95% không chứa 0. Trên dữ liệu mô phỏng.</text></svg>
</div>
<figcaption>Hình 2. Cùng chính sách `v1-all`, hai mật độ. Ở `thưa xe` khoản giảm km rỗng lớn gấp 12 lần, nhưng khoản bỏ rơi khách lớn gấp 6 lần và Gini số chuyến đảo chiều sang tệ hơn, kèm khoảng rảnh dài nhất của tài xế tăng 1.904 s. Trên dữ liệu mô phỏng.</figcaption>
</figure>

Ở `thưa xe`, khách hết kiên nhẫn đi từ 0,0733 lên 0,2441 (+0,1708 [0,1503; 0,1912], 20/20 seed), số chuyến hoàn thành giảm 6,9 trên 150,5, hủy tăng 0,0307, Gini số chuyến tăng +0,057 [0,047; 0,067], và khoảng rảnh dài nhất của tài xế tăng lên tới **+1.904 s** ở giá trị lớn nhất: xe đứng chờ trong khi khách bị từ chối vì quá xa. Không phải tài xế nào cũng được lợi từ chính sách đón xa: hai thứ tác động ngược nhau.

Theo vùng ở `thưa xe` (`v1-all`): trung tâm hoàn thành −0,028 (có ý nghĩa), **vành trong −0,039 (có ý nghĩa)**, ngoại vi −0,018 (không phân biệt được). Nên đừng nhớ máy móc «ngoại vi trả giá»: ở mật độ thường đó là ngoại vi, ở xe thưa và ở lô 10 s cao điểm thiệt hại nằm ở vùng trong, có lẽ vì ETA điển hình đã gần ngưỡng cứng 600 s ở mọi nơi (ETA trung bình `v0` ở `thưa xe` là 555,9 s). Cái nhất quán xuyên hai mật độ không phải «vùng nào», mà là: **hiệu ứng của một ngưỡng cứng phụ thuộc mật độ xe, và con số trung bình toàn cục không cho biết điều đó.**

Một nhánh nữa: ở bộ `huỷ nặng`, `v1-all` chỉ giảm `wait_s` max 120,4 s và p95 `pickup_eta_s` 35,5 s, phần lớn chỉ số trung bình không phân biệt được, nhưng khách hết kiên nhẫn tăng +0,0178 [0,0126; 0,0230] (XẤU HƠN). Nghĩa là giá phải trả còn nguyên, lợi lại nhỏ đi.

Tóm lại, **không có bộ trọng số nào ở đây là miễn phí**: `v1-idle` đổi khách lấy tài xế; `v1-limit` đổi khách ở xa xe lấy số trung bình đẹp và, khi xe thưa, đổi cả tài xế lẫn khách. Câu trả lời cho bài 42 là chưa có bản sửa: thưởng chờ lâu 0,5 gần như không đổi khoản bỏ rơi, còn giới hạn đón xa **làm nó tăng**, ở cả lô 10 s cao điểm (+0,1256) lẫn ghép ngay (`v1-all` +0,0287 ở `thường`, +0,1708 ở `thưa xe`).

## Chạy ví dụ và kiểm chứng

Từ gốc repo (cần `data/hcm-graph.bin`; thêm `--fixture` để chạy nhanh trên đồ thị tổng hợp):

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc45_chi_phi_cong_bang_rui_ro_huy
```

Tham số: `--seeds N` (20), `--jobs N`, `--hinh-dang-lo` (đếm hình dạng lô khi ghép ngay), `--thu-lo` (các chính sách với cửa sổ lô 10 s ở `cao điểm`). Output đầy đủ ở `examples/toi-noi-location/results/p05.md`. Trích bảng công bằng theo vùng ở `thường` (nguyên văn trừ dòng tiêu đề, đo khi máy chịu tải nặng; đây là tỉ lệ và hiệu của số đếm nên không bị thời gian ảnh hưởng):

```text
  chính sách                                   trung tâm (<3 km)                            vành trong (3–6 km)                          ngoại vi (≥6 km)                            
  v1-risk (+ xác suất huỷ × 600 s)             hoàn thành +0.000 · wait +0 s                hoàn thành +0.000 · wait +0 s                hoàn thành +0.000 · wait +0 s               
  v1-age (+ thưởng chờ lâu 0,5)                hoàn thành +0.000 · wait +0 s                hoàn thành +0.000 · wait +0 s                hoàn thành +0.000 · wait +0 s               
  v1-idle (+ thưởng xe rảnh lâu 0,1)           hoàn thành -0.003 · wait +3 s                hoàn thành -0.001 · wait +7 s*               hoàn thành -0.009 · wait +8 s*              
  v1-limit (đón xa: mềm 300 s ×1, cứng 600 s)  hoàn thành +0.016 · wait -3 s                hoàn thành +0.000 · wait -10 s*              hoàn thành -0.027* · wait -26 s*            
  v1-all (tất cả các số hạng trên)             hoàn thành +0.004 · wait -2 s                hoàn thành -0.003 · wait -8 s                hoàn thành -0.031* · wait -18 s*            
```

(Dòng tiêu đề của output gốc, bỏ ở đây, ghi vùng là khoảng cách từ điểm ghim tới một điểm quy chiếu cố định của lab; bài này không dùng điểm đó như dữ liệu thật.) `verify-p05.sh` chạy lab này trên fixture với 2 seed để kiểm bất biến, không tái lập các số trên.

## Giới hạn của ví dụ

- **Mô phỏng, và chiến lược biết mô hình hủy.** Xác suất hủy lấy từ chính công thức của kịch bản. Mô hình hủy không phụ thuộc quãng đi bộ, nên khi ghép ngay số hạng rủi ro bị vô hiệu hóa **do thiết kế của thí nghiệm**, không phải do ý tưởng.
- **Chưa chạy:** `cao điểm` khi ghép ngay, `dày xe` và `huỷ theo chờ` cho bài này; trọng số `age_bonus` khác 0,5; `v1-age`, `v1-idle`, `v1-limit` ở bộ `huỷ nặng` (ở đó chỉ chạy `v1-risk` và `v1-all`); mô hình hủy phụ thuộc quãng đi bộ hoặc đặc điểm khách.
- **Không thử** bản giới hạn đón xa chỉ có phần mềm (không cấm cứng), nên chưa biết phần nào của thiệt hại đến từ ngưỡng cứng.
- **Công bằng cho tài xế đo bằng số chuyến và khoảng rảnh,** không phải thu nhập, không phải mức hài lòng; vùng đo theo khoảng cách tới một tâm quy chiếu, không phải khu vực hành chính.
- **Trọng số đặt tay** (600 s, 0,5, 0,1, 300 s, 600 s), không dò lưới, nên có thể có bộ khác tốt hơn; lab chọn không dò để không chọn theo may rủi của seed.
- **Chưa cắm vào Dispatch thật.** Mọi thứ nằm trong simulator.

## Bài tập

**Bài 1.** Ma trận ETA (giây) `[[100, 200], [150, 400]]`, khách 0 đã chờ 20 s, khách 1 đã chờ 60 s, `age_bonus = 0,5`. Tính chi phí hai phép gán có thể có trước và sau khi thêm thưởng chờ lâu, và chứng tỏ phép gán tối ưu không đổi.

<details><summary>Lời giải bài 1</summary>
<p>Thưởng chờ: khách 0 trừ 0,5 × 20 = 10 s, khách 1 trừ 0,5 × 60 = 30 s, mỗi khách trừ một hằng số trên mọi ô của hàng. Đường chéo chính (0,0) và (1,1): 100 + 400 = 500, sau thưởng 500 − 10 − 30 = 460. Đường chéo phụ (0,1) và (1,0): 200 + 150 = 350, sau thưởng 350 − 40 = 310. Cả hai phép gán đều giảm đúng 40 s, nên đường chéo phụ vẫn tối ưu (350 so với 500 trước, 310 so với 460 sau). Thưởng chờ lâu chỉ có tác dụng khi không đủ xe cho mọi khách: lúc đó nó quyết định khách nào bị bỏ lại.</p>
</details>

**Bài 2.** Ma trận ETA `[[300, 700], [650, 900]]` giây, giới hạn cứng 600 s. Tính tổng ETA và ETA trung bình mỗi khách được phục vụ khi không giới hạn và khi có giới hạn cứng. Chính sách nào trông tốt hơn nếu chỉ đọc ETA trung bình, và vì sao đó là cách đọc sai?

<details><summary>Lời giải bài 2</summary>
<p>Không giới hạn: đường chéo chính 300 + 900 = 1.200, đường chéo phụ 700 + 650 = 1.350, Hungarian chọn 1.200, phục vụ cả hai khách, ETA trung bình 600 s. Giới hạn cứng 600 s: các ô 700, 650, 900 bị cấm, chỉ còn (0,0) = 300 s, nên chỉ một khách được phục vụ, ETA trung bình 300 s, khách 1 bị bỏ lại. Nhìn ETA trung bình, chính sách có giới hạn «tốt gấp đôi» (300 so với 600 s), nhưng nó phục vụ 1 khách thay vì 2. Đó là thiên lệch người sống sót: số liệu tính trên chuyến hoàn thành đẹp lên vì khách khó nhất đã rời mẫu. Phải báo cáo kèm số khách phục vụ và tỉ lệ hoàn thành theo vùng.</p>
</details>

**Bài 3.** Đề xuất một «chỉ số bảo vệ» theo vùng để chặn các chính sách như `v1-limit`, nêu ngưỡng, cách tính và thí nghiệm kiểm chứng. Nêu một hạn chế của đề xuất của bạn.

<details><summary>Lời giải bài 3</summary>
<p>Một đề xuất hợp lệ: với mỗi vùng, hiệu tỉ lệ hoàn thành giữa chính sách mới và chính sách hiện hành (có cặp, 20 seed, khoảng tin cậy 95%) phải có cận dưới không thấp hơn −0,01, nếu không chính sách bị loại dù trung bình toàn cục tốt hơn. Với bảng ở trên, `v1-limit` và `v1-all` thất bại ở ngoại vi (−0,027 và −0,031). Đáng chú ý là `v1-idle` cũng không qua: −0,009 mà khoảng chứa 0 nghĩa là nửa khoảng lớn hơn 0,009, nên cận dưới thấp hơn −0,018. Ngưỡng này với 20 seed gần như loại mọi chính sách, tức cần nhiều seed hơn cho chỉ số theo vùng. Thí nghiệm: chạy bộ `thường`, `thưa xe` và `huỷ nặng`, báo cả ba vùng, và chốt ngưỡng −0,01 **trước** khi đo. Hạn chế: ngưỡng −0,01 là chính sách, không đọc ra từ lab; vùng theo khoảng cách chim bay tới một tâm cố định bỏ sót các vùng thiệt thòi khác (ví dụ vùng có đường kém, hay vùng theo giờ); và với 20 seed khoảng tin cậy rộng, nên «không vi phạm» chưa bằng «không có tác hại».</p>
</details>

Bài trước: [bài 44](/articles/location-cat-tia-ung-vien-h3-ma-tran-thua). Bài tiếp: [bài 46](/articles/location-goi-y-sang-dispatch-csharp-grpc-lease-idempotency).

## Tài liệu gốc

- [Hungarian algorithm (Wikipedia)](https://en.wikipedia.org/wiki/Hungarian_algorithm): phép gán tối ưu mà các chính sách chi phí ở đây cắm vào.
- [`pathfinding::kuhn_munkres` (docs.rs)](https://docs.rs/pathfinding/4.16.0/pathfinding/kuhn_munkres/fn.kuhn_munkres.html): thư viện Kuhn–Munkres dùng trong lab.
- [Student's t-test (Wikipedia)](https://en.wikipedia.org/wiki/Student%27s_t-test): kiểm định mẫu ghép cặp, nền của khoảng tin cậy trong bài.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho dữ liệu OSM.
