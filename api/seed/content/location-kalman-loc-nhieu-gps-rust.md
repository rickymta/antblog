---
slug: location-kalman-loc-nhieu-gps-rust
title: "Lọc nhiễu GPS bằng Kalman: giúp được bao nhiêu và trả giá gì"
summary: "Bài 15: bộ lọc Kalman với mô hình vận tốc không đổi cho điểm GPS, nhiễu đo lấy từ accuracy, cổng NIS cho điểm nhảy, làm mượt RTS; đo với ground truth của simulator: giảm 23,2% RMSE khi nhiễu trắng nhưng chỉ 3,7% khi nhiễu tương quan 8 s."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, kalman, gps, filtering, rts-smoother, synthetic-data]
language: vi
---

Xe của tài xế dừng đèn đỏ ở một ngã tư. Khách đang nhìn marker trên bản đồ: nó run lên, lệch sang trái 8 m, về giữa, lệch sang phải 14 m, dù chiếc xe không nhúc nhích. Chạy tiếp, marker còn lúc nằm trên vỉa hè, lúc nằm trong một căn nhà. Cách chữa quen thuộc là lọc: không tin từng điểm đo, mà tin một mô hình chuyển động rồi dùng điểm đo để chỉnh nó. Bộ lọc Kalman là cách làm sạch kinh điển nhất cho việc này.

Đây là bài 15 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài tiếp nối phần nhận GPS ở [bài 14](/articles/location-validate-gps-timestamp-accuracy-gia-mao), nơi `accuracy` còn là một trường để kiểm tra; ở đây nó trở thành độ tin cậy của từng điểm đo. Mọi con số trong bài đều **tính trên dữ liệu mô phỏng**: xe chạy trên lộ trình thật của đồ thị TP.HCM, nhưng nhiễu GPS do chính lab sinh ra, không phải GPS người thật. Điều đó làm lệch kết quả theo hướng mà mô hình nhiễu của chúng ta cho phép, và các mục dưới nói rõ chỗ nào bị ảnh hưởng. Chuyện marker nhảy giữa cầu vượt và đường bên dưới không thuộc bài này: Kalman không biết đường ở đâu, việc đó thuộc về bài 16 và 17.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Cùng độ lệch chuẩn σ = 6 m mỗi trục, ta làm hỏng vị trí thật theo hai cách: sai số <b>độc lập</b> giữa các điểm (nhiễu trắng), và sai số <b>tương quan theo thời gian</b> (hằng số 8 s, gần với cách GPS thật trôi chậm). Bộ lọc Kalman giảm RMSE so với vị trí thật được bao nhiêu phần trăm ở mỗi loại?</p>
<details><summary>Xem đáp án</summary><p>Nhiễu trắng: giảm <b>23,2%</b> (9,20 m xuống 7,07 m). Nhiễu tương quan 8 s: chỉ giảm <b>3,7%</b> (9,22 m xuống 8,87 m). Bộ lọc trung bình hoá rất nhanh những sai số độc lập, nhưng không xoá được độ lệch chậm, vì nhiều điểm liên tiếp cùng lệch một phía thì trung bình của chúng vẫn lệch phía đó.</p></details>
</div>

## Mô hình vận tốc không đổi, từng bước

Trạng thái của mỗi trục (đông, bắc) gồm hai số: vị trí và vận tốc, tính bằng mét trên mặt phẳng cục bộ quanh điểm đầu tiên của chuyến (phép chiếu `LocalProjection` của `geo-core`, xem [bài 04](/articles/location-khoang-cach-haversine-equirectangular-sai-so) và [bài 05](/articles/location-geometry-crate-geo-chieu-diem-polyline-bearing)). Mô hình cho rằng sau `dt` giây, vị trí tăng thêm `vận tốc × dt`, còn vận tốc đổi do một gia tốc ngẫu nhiên có độ lệch chuẩn `accel_sigma` (m/s²). Gia tốc ngẫu nhiên này là chỗ ta thừa nhận xe không chạy đều: nó là «nhiễu quá trình». Nếu giá trị này lớn thì bộ lọc bám sát điểm đo, nếu nhỏ thì bộ lọc mượt nhưng chậm bám theo khi xe tăng tốc, phanh hay rẽ.

Mỗi điểm GPS mới đi qua hai bước. **Dự đoán**: đẩy trạng thái tới thời điểm điểm đo, độ bất định của vị trí tăng lên. **Cập nhật**: so điểm đo với dự đoán; phần chênh gọi là «đổi mới» (innovation) ν. Hệ số khuếch đại `K = P' / (P' + R)` quyết định bộ lọc tin điểm đo bao nhiêu: `P'` là phương sai dự đoán, `R` là phương sai nhiễu đo. Nếu `R` lớn so với `P'`, `K` nhỏ và điểm đo chỉ nhích ước lượng một chút. Mã dưới đây thể hiện đúng hai bước đó cho một điểm; hai trục có cùng mô hình và cùng nhiễu đo (accuracy là bán kính nên đều hướng), nên ma trận hiệp phương sai 2×2 giống nhau ở cả hai trục và được tính một lần.

Tệp `examples/toi-noi-location/rust/crates/map-match/src/kalman.rs` (dòng 124–158):

```rust
        // dự đoán: x' = F x, P' = F P Fᵀ + Q (gia tốc trắng)
        let q = self.cfg.accel_sigma.powi(2);
        let [pp, pv, vv] = self.p;
        let p_pred = [
            pp + 2.0 * dt * pv + dt * dt * vv + q * dt.powi(4) / 4.0,
            pv + dt * vv + q * dt.powi(3) / 2.0,
            vv + q * dt * dt,
        ];
        let mut x_pred = self.x;
        for a in &mut x_pred {
            a[0] += dt * a[1];
        }
        // cập nhật; cổng NIS phóng to R nếu đổi mới quá lớn so với dự đoán
        let nu = [z[0] - x_pred[0][0], z[1] - x_pred[1][0]];
        let mut r_eff = r;
        if let Some(gate) = self.cfg.gate_nis {
            let nis = (nu[0] * nu[0] + nu[1] * nu[1]) / (p_pred[0] + r);
            if nis > gate {
                r_eff = r * nis / gate;
            }
        }
        let s = p_pred[0] + r_eff;
        let k = [p_pred[0] / s, p_pred[1] / s];
        let mut x = x_pred;
        for a in 0..2 {
            x[a][0] += k[0] * nu[a];
            x[a][1] += k[1] * nu[a];
        }
        let p_filt = [
            (1.0 - k[0]) * p_pred[0],
            (1.0 - k[0]) * p_pred[1],
            p_pred[2] - k[1] * p_pred[1],
        ];
        self.x = x;
        self.p = p_filt;
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 346" role="img" aria-label="Hình 1. Một bước cập nhật Kalman trên một trục, số minh hoạ tự chọn: dự đoán, điểm đo, ước lượng sau cập nhật; hàng dưới là điểm đo nhảy xa, có cổng NIS và không có cổng."><defs><marker id="loc15-f1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs><path d="M200.0 24.0 L200.0 310.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="200.0" y="320.0" class="t-small" text-anchor="middle">40 m</text><path d="M320.0 24.0 L320.0 310.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="320.0" y="320.0" class="t-small" text-anchor="middle">50 m</text><path d="M440.0 24.0 L440.0 310.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="440.0" y="320.0" class="t-small" text-anchor="middle">60 m</text><path d="M560.0 24.0 L560.0 310.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="560.0" y="320.0" class="t-small" text-anchor="middle">70 m</text><path d="M680.0 24.0 L680.0 310.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="680.0" y="320.0" class="t-small" text-anchor="middle">80 m</text><path d="M800.0 24.0 L800.0 310.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="800.0" y="320.0" class="t-small" text-anchor="middle">90 m</text><text x="680.0" y="337.0" class="t-small" text-anchor="middle">vị trí dọc đường (mét), mỗi đoạn ngang là ±1σ</text><text x="20.0" y="38.0" class="t-b">Điểm đo cách dự đoán 8 m</text><text x="20.0" y="56.0">dự đoán x'</text><path d="M296.0 52.0 L392.0 52.0" class="f-none s-phys" stroke-width="3"/><circle cx="344.0" cy="52.0" r="5" class="f-phys s-phys" stroke-width="1.5"/><text x="20.0" y="86.0">điểm đo z</text><path d="M368.0 82.0 L512.0 82.0" class="f-none s-dead" stroke-width="3"/><circle cx="440.0" cy="82.0" r="5" class="f-dead s-dead" stroke-width="1.5"/><text x="20.0" y="116.0">ước lượng</text><path d="M333.6 112.0 L413.5 112.0" class="f-none s-pk" stroke-width="3"/><circle cx="373.5" cy="112.0" r="5" class="f-pk s-pk" stroke-width="1.5"/><text x="560.0" y="56.0" class="t-small">P' = 16, R = 36 (σ 4 m và 6 m)</text><text x="560.0" y="86.0" class="t-small">K = P'/(P'+R) = 0,31</text><text x="560.0" y="116.0" class="t-small">x = x' + K·ν = 54,5 m; NIS = 1,23 &lt; 9,21</text><text x="20.0" y="168.0" class="t-b">Điểm đo nhảy xa 30 m</text><text x="20.0" y="186.0">dự đoán x'</text><path d="M296.0 182.0 L392.0 182.0" class="f-none s-phys" stroke-width="3"/><circle cx="344.0" cy="182.0" r="5" class="f-phys s-phys" stroke-width="1.5"/><text x="20.0" y="216.0">điểm đo z</text><path d="M632.0 212.0 L776.0 212.0" class="f-none s-dead" stroke-width="3"/><circle cx="704.0" cy="212.0" r="5" class="f-dead s-dead" stroke-width="1.5"/><text x="20.0" y="246.0">ước lượng, có cổng</text><path d="M369.7 242.0 L456.0 242.0" class="f-none s-pk" stroke-width="3"/><circle cx="412.9" cy="242.0" r="5" class="f-pk s-pk" stroke-width="1.5"/><text x="20.0" y="276.0">ước lượng, không cổng</text><path d="M414.8 272.0 L494.7 272.0" class="f-none s-warn" stroke-width="3"/><circle cx="454.8" cy="272.0" r="5" class="f-warn s-warn" stroke-width="1.5"/><text x="560.0" y="186.0" class="t-small">NIS = ν²/(P'+R) = 17,3 &gt; 9,21</text><text x="560.0" y="200.0" class="t-small">R_eff = R·NIS/9,21 = 67,7</text><text x="500.0" y="246.0" class="t-small">có cổng: K = 0,191, nhích 5,7 m</text><text x="500.0" y="276.0" class="t-small">không cổng: K = 0,308, nhích 9,2 m</text></svg>
</div>
<figcaption>Hình 1. Một bước Kalman trên một trục, <b>số minh hoạ tự chọn</b> (không phải output của lab): dự đoán x' = 52 m với σ = 4 m, điểm đo có σ = 6 m. Hàng trên: K = 0,31, ước lượng nằm gần dự đoán hơn điểm đo. Hàng dưới: điểm nhảy 30 m vượt cổng NIS 9,21 nên R bị phóng to và ước lượng chỉ nhích 5,7 m thay vì 9,2 m.</figcaption>
</figure>

Hình 1 minh hoạ đúng công thức này với số tự chọn. Hàng trên: dự đoán 52 m (σ = 4 m), điểm đo cách 8 m (σ = 6 m). `K` = 16 / 52 = 0,31, nên ước lượng nhích 2,5 m về phía điểm đo thay vì nhảy hết 8 m, và độ bất định sau cập nhật nhỏ hơn cả hai nguồn. Đó là toàn bộ «phép màu»: một trung bình có trọng số, trọng số lấy từ độ tin cậy của từng bên.

## Nhiễu đo lấy từ accuracy của từng điểm

`R` không thể là một hằng số: điểm báo accuracy 5 m và điểm báo 40 m không đáng tin như nhau. Android định nghĩa accuracy là bán kính tin cậy 68% của vị trí; iOS chỉ gọi là bán kính bất định, không nêu mức tin cậy, nên lab dùng chung cách hiểu của Android. Với sai số hai chiều đều hướng và độ lệch chuẩn σ mỗi trục, xác suất sai số nằm trong bán kính ρ là `1 − exp(−ρ²/2σ²)`; đặt bằng 0,68 thì ρ = σ·√(−2·ln 0,32) ≈ 1,51σ. Vì vậy `NoiseModel` đổi accuracy thành σ bằng hệ số 0,67 (khoảng 1/1,5), kẹp vào đoạn từ 3 m đến 40 m để một giá trị báo sai không khiến bộ lọc tin hẳn vào điểm đo hay bỏ qua hẳn điểm đo, và dùng 8 m khi app không báo gì.

Tệp `examples/toi-noi-location/rust/crates/map-match/src/lib.rs` (dòng 63–94):

```rust
/// Đổi accuracy báo cáo thành độ lệch chuẩn mỗi trục (mét). Dùng chung cho Kalman (nhiễu đo) và HMM (xác suất phát).
///
/// Bán kính 68% của phân phối hai chiều đều hướng bằng khoảng 1,5 sigma mỗi trục, nên `accuracy_to_sigma` mặc định 0,67.
/// Khi vắng accuracy dùng `default_sigma_m`; kẹp vào `[floor_m, cap_m]` để một giá trị báo sai không làm bộ lọc tin tuyệt đối hay bỏ qua điểm.
#[derive(Debug, Clone, Copy)]
pub struct NoiseModel {
    pub accuracy_to_sigma: f64,
    pub default_sigma_m: f64,
    pub floor_m: f64,
    pub cap_m: f64,
}

impl Default for NoiseModel {
    fn default() -> Self {
        Self {
            accuracy_to_sigma: 0.67,
            default_sigma_m: 8.0,
            floor_m: 3.0,
            cap_m: 40.0,
        }
    }
}

impl NoiseModel {
    pub fn sigma_m(&self, accuracy_m: Option<f32>) -> f64 {
        accuracy_m
            .map_or(self.default_sigma_m, |a| {
                f64::from(a) * self.accuracy_to_sigma
            })
            .clamp(self.floor_m, self.cap_m)
    }
}
```

Hai chi tiết nhỏ nhưng quan trọng. Thứ nhất: vắng accuracy là `None`, **khác** với `Some(0.0)`; test `missing_accuracy_differs_from_zero` kiểm tra đúng điều này (vắng cho 8 m, 0 bị kẹp về sàn 3 m), nối với quy ước «vắng khác 0» của hợp đồng protobuf ở các bài ingest. Thứ hai: cùng `NoiseModel` này sẽ dùng để tính xác suất phát của HMM ở bài 16, nên hai bài dùng chung một cách hiểu về accuracy.

Vì accuracy trong lab do chính simulator sinh (bằng 1,5σ nhân một hệ số lognormal với độ lệch chuẩn 0,25, và vắng ở 10% điểm), nó **được hiệu chuẩn tốt** hơn mức ta nên mong ở điện thoại thật. Lab thử bỏ hẳn accuracy: RMSE chỉ đổi từ 8,87 m sang 8,85 m. Kết luận «accuracy không quan trọng» sẽ sai ngoài đời, nơi nhiều máy báo accuracy lệch hẳn so với sai số thật; ở đây nó chỉ phản ánh mô hình nhiễu của ta.

## Cổng NIS: điểm nhảy xa không được kéo cả bộ lọc

Đa đường (tín hiệu dội qua nhà cao tầng) tạo ra điểm nhảy xa vài chục mét. Một điểm như vậy có «đổi mới» rất lớn so với dự đoán. Nếu cứ tin theo `K`, ước lượng bị kéo đi. Cách xử lý trong mã: tính `NIS = ν² / (P' + R)`, tức bình phương khoảng cách Mahalanobis của đổi mới với hai bậc tự do. Dưới giả định nhiễu Gaussian, NIS vượt 9,21 chỉ xảy ra khoảng 1% số lần (phân phối χ² hai bậc tự do), nên vượt ngưỡng là dấu hiệu điểm lạ. Khi đó bộ lọc phóng to `R` lên `R · NIS / 9,21`, làm điểm đó ít ảnh hưởng chứ không loại hẳn.

Hàng dưới của Hình 1 minh hoạ: điểm nhảy 30 m cho NIS = 17,3 nên `R` hiệu dụng tăng lên 67,7 và ước lượng chỉ nhích 5,7 m thay vì 9,2 m. Đo thật trên 3% điểm nhảy xa 25–70 m: RMSE thô 12,05 m; Kalman không cổng 10,98 m; Kalman có cổng NIS 9,21 là 9,99 m. Cổng cho hiệu quả rõ ràng ở đúng chỗ nó được thiết kế. Cổng **không** giúp khi nhiễu lớn đều chứ không nhảy: ở σ = 20 m, có cổng và không cổng cho 29,23 m và 29,24 m.

## Đo trên nhiễu tổng hợp có ground truth

Ground truth ở đây do simulator của lab cho: xe chạy trên lộ trình ngắn nhất của đồ thị HCM với gia tốc 1,8 m/s², phanh 2,5 m/s², chậm ở khúc cua, dừng đèn ở khoảng 8% nút giao từ 8 đến 35 giây; mỗi giây lấy một mẫu. Sau đó điểm bị làm hỏng bằng sai số Gaussian mỗi trục tương quan theo thời gian (tiến trình AR(1) với hằng số 8 giây), 0,5% điểm nhảy xa 25–70 m, 1% điểm vắng ngẫu nhiên, accuracy báo cáo như đã nói ở trên. Đây là mô hình nhiễu **của chúng ta**: phân phối đuôi dày, lỗi phụ thuộc thiết bị và accuracy báo sai của GPS thật chưa được kiểm tra. Kết quả đúng cho mô hình này, và chỉ cho thấy thuật toán chạy đúng chứ không cho biết độ chính xác ngoài đời.

Lab tách việc chọn tham số khỏi phần báo cáo. `accel_sigma` và cổng NIS được chọn trên 40 chuyến seed 1106 (29.734 điểm thật, σ = 6 m tương quan 8 s); mọi bảng báo cáo dưới đây dùng 40 chuyến khác seed 20261106 (26.392 điểm). Bảng chọn tham số gần như hoà giữa `accel_sigma` = 2 và 4 (8,820 m và 8,810 m); lab chọn 4 và đó cũng là giá trị mặc định trong `KalmanConfig::default()`. Tham số chỉ chọn một lần ở σ = 6 m, rồi dùng nguyên cho mọi σ khác.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Hình 2. Mức giảm RMSE của Kalman so với điểm GPS thô theo độ lệch chuẩn nhiễu: nhiễu tương quan 8 s giảm ít và đảo dấu ở σ = 20 m, nhiễu trắng giảm 23,2% và 26,1%."><rect x="20.0" y="20.0" width="840.0" height="360.0" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36.0" y="44.0" class="t-b">Giảm RMSE so với điểm thô (%, âm là tệ hơn); 26.392 điểm, 40 chuyến, seed 20261106</text><path d="M70.0 300.0 L840.0 300.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="62.0" y="304.0" class="t-small" text-anchor="end">0%</text><path d="M70.0 223.3 L840.0 223.3" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="62.0" y="227.3" class="t-small" text-anchor="end">10%</text><path d="M70.0 146.7 L840.0 146.7" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="62.0" y="150.7" class="t-small" text-anchor="end">20%</text><path d="M70.0 70.0 L840.0 70.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="62.0" y="74.0" class="t-small" text-anchor="end">30%</text><rect x="100.0" y="114.5" width="78.0" height="185.5" rx="3" class="f-phys s-phys" stroke-width="1.5"/><text x="139.0" y="106.5" class="t-b" text-anchor="middle">24,2%</text><text x="139.0" y="352.0" text-anchor="middle">σ 2 m</text><rect x="200.0" y="231.0" width="78.0" height="69.0" rx="3" class="f-phys s-phys" stroke-width="1.5"/><text x="239.0" y="223.0" class="t-b" text-anchor="middle">9,0%</text><text x="239.0" y="352.0" text-anchor="middle">σ 4 m</text><rect x="300.0" y="271.6" width="78.0" height="28.4" rx="3" class="f-phys s-phys" stroke-width="1.5"/><text x="339.0" y="263.6" class="t-b" text-anchor="middle">3,7%</text><text x="339.0" y="352.0" text-anchor="middle">σ 6 m</text><rect x="400.0" y="298.5" width="78.0" height="1.5" rx="3" class="f-phys s-phys" stroke-width="1.5"/><text x="439.0" y="290.5" class="t-b" text-anchor="middle">0,2%</text><text x="439.0" y="352.0" text-anchor="middle">σ 10 m</text><rect x="500.0" y="300.0" width="78.0" height="16.9" rx="3" class="f-warn s-warn" stroke-width="1.5"/><text x="539.0" y="332.9" class="t-b" text-anchor="middle">−2,2%</text><text x="539.0" y="352.0" text-anchor="middle">σ 20 m</text><rect x="640.0" y="122.1" width="78.0" height="177.9" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="679.0" y="114.1" class="t-b" text-anchor="middle">23,2%</text><text x="679.0" y="352.0" text-anchor="middle">σ 6 m</text><rect x="740.0" y="99.9" width="78.0" height="200.1" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="779.0" y="91.9" class="t-b" text-anchor="middle">26,1%</text><text x="779.0" y="352.0" text-anchor="middle">σ 10 m</text><text x="339.0" y="374.0" class="t-small" text-anchor="middle">nhiễu tương quan theo thời gian, hằng số 8 s</text><text x="729.0" y="374.0" class="t-small" text-anchor="middle">nhiễu trắng (độc lập)</text><path d="M100.0 360.0 L578.0 360.0" class="f-none s-muted" stroke-width="1"/><path d="M640.0 360.0 L818.0 360.0" class="f-none s-muted" stroke-width="1"/></svg>
</div>
<figcaption>Hình 2. Mức giảm RMSE của Kalman so với điểm thô, lấy từ mục 2 của output <code>loc15_kalman</code> (tham số đã chọn trên seed 1106, báo cáo trên seed 20261106). Cùng σ = 6 m: nhiễu trắng giảm 23,2%, nhiễu tương quan 8 s chỉ giảm 3,7%; ở σ = 20 m tương quan, Kalman làm tệ hơn 2,2%. Nhiễu do lab sinh, không phải GPS thật.</figcaption>
</figure>

Hình 2 cho thấy hai điều. Thứ nhất, **sai số tương quan là kẻ thù của bộ lọc**. Cùng σ = 6 m, nhiễu trắng giảm 23,2% còn nhiễu tương quan 3,7%; σ = 10 m tương quan chỉ giảm 0,2%; σ = 20 m tương quan thì Kalman làm **tệ hơn** 2,2% (28,59 m lên 29,23 m). Kalman giả định các lần đo sai độc lập; khi sai số trôi chậm, bộ lọc coi chuỗi điểm cùng lệch là bằng chứng và bám theo độ lệch đó. Thứ hai, **mức giảm lớn nhất không đáng tin hoàn toàn**: ở σ = 2 m tương quan, giảm 24,2% nghe rất tốt. Nhưng ở mức này, 0,5% điểm nhảy xa 25–70 m đã đóng góp khoảng 12 trong tổng khoảng 20 m² bình phương sai số (ước tính nhanh: bình phương trung bình của khoảng 25–70 m là cỡ 2.400 m², nhân 0,005), và p95 lại **xấu đi** nhẹ (5,01 m lên 5,17 m). Nên có khả năng phần lớn mức giảm đến từ việc dập điểm nhảy chứ không phải từ làm mượt. Giả thuyết này **chưa được kiểm chứng** bằng một lần chạy tắt điểm nhảy; bài tập 3 là cách kiểm chứng.

Cũng lưu ý p95 xấu đi ở cả năm mức nhiễu tương quan, kể cả những mức RMSE tốt hơn (σ = 4 m: 10,03 m lên 10,23 m; σ = 6 m: 15,04 m lên 15,35 m), trong khi với nhiễu trắng p95 cải thiện rõ (σ = 6 m: 15,00 m xuống 12,08 m). Với nhiễu tương quan, đuôi sai số, tức những điểm tệ nhất, không được cải thiện; đó là loại điểm dễ gây lỗi ở bước bám đường.

## Cái giá phải trả: lệch theo hướng đi và làm mượt RTS

Bộ lọc không làm vị trí «đến muộn»: nó trả ước lượng ngay khi nhận điểm. Cái giá nằm ở chỗ mô hình giả định vận tốc không đổi, nên khi xe đổi vận tốc, ước lượng chạy lệch so với xe. Lab đo sai số **dọc hướng đi** (ước lượng trừ vị trí thật, chiếu lên hướng xe; âm là nằm sau xe), trung bình theo pha chuyển động, σ = 6 m tương quan 8 s, trên nhiễu mô phỏng của lab:

| Pha | Số điểm | RMSE thô (m) | RMSE Kalman (m) | Dọc hướng, Kalman | Dọc hướng, thô |
|---|---:|---:|---:|---:|---:|
| Tăng tốc | 1.242 | 9,58 | 8,95 | −1,09 m | 0,03 m |
| Phanh | 543 | 8,60 | 8,42 | 0,98 m | 0,41 m |
| Chạy đều | 16.428 | 9,24 | 8,91 | 0,08 m | 0,15 m |
| Dừng | 5.686 | 9,29 | 8,83 | −0,28 m | −0,44 m |
| Rẽ | 2.089 | 8,76 | 8,76 | −0,65 m | −0,09 m |

Các số này nhỏ so với nhiễu (cỡ 9 m), nhưng đây là sai số **có hướng**: điểm thô lệch ngẫu nhiên hai phía, còn Kalman lệch có hệ thống ra sau xe khi tăng tốc và ra trước khi phanh. Khi GPS tốt (σ = 2 m, nhiễu trắng), con số tương ứng là −0,54 m, +0,58 m và −0,29 m ở ba pha đó. Hình 3 vẽ khúc cua gắt nhất của chuyến báo cáo số 0 từ chính output của lab; trên 15 điểm này, RMSE sau Kalman (11,63 m, tính lại từ CSV) còn **cao hơn** điểm thô (10,84 m), và Kalman gần đường thật hơn ở 6 trên 15 điểm. Đó chỉ là một ví dụ để nhìn cơ chế, không phải thống kê, nhưng nó nhắc rằng số trung bình 8,87 so với 9,22 che đi những đoạn lọc làm xấu hơn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 420" role="img" aria-label="Hình 3. Khúc cua gắt nhất của chuyến báo cáo số 0: đường thật, điểm GPS thô và điểm sau Kalman, 15 giây liên tiếp, xe chạy 3,8 m/s, σ = 6 m tương quan 8 s."><rect x="20.0" y="20.0" width="470.0" height="380.0" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36.0" y="44.0" class="t-b">Khúc cua (mét, quanh điểm đầu)</text><path d="M349.5 66.0 L354.6 98.0 L359.6 130.0 L364.6 162.1 L344.6 176.5 L312.5 182.5 L280.4 188.3 L248.3 194.1 L216.1 200.1 L184.1 205.9 L152.0 211.8 L123.5 212.1 L116.5 180.5 L109.5 148.8 L102.5 117.1" class="f-none s-phys" stroke-width="3" stroke-linejoin="round" stroke-linecap="round"/><path d="M388.1 150.5 L368.4 138.8 L377.4 213.5 L375.8 305.4 L346.1 325.0 L280.3 296.7 L236.2 269.7 L172.3 258.1 L175.5 245.3 L155.5 223.2 L129.2 241.8 L132.6 288.8 L119.8 301.7 L88.3 235.9 L56.0 140.3" class="f-none s-dead dash" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M380.3 151.5 L371.0 154.6 L373.5 200.0 L373.8 274.3 L353.0 325.5 L316.4 344.0 L254.8 306.3 L183.1 271.0 L162.1 247.7 L140.9 225.1 L126.4 235.5 L126.1 274.8 L119.3 300.3 L95.9 264.3 L67.2 195.7" class="f-none s-pk" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"/><circle cx="388.1" cy="150.5" r="3.5" class="f-dead s-dead" stroke-width="1"/><circle cx="368.4" cy="138.8" r="3.5" class="f-dead s-dead" stroke-width="1"/><circle cx="377.4" cy="213.5" r="3.5" class="f-dead s-dead" stroke-width="1"/><circle cx="375.8" cy="305.4" r="3.5" class="f-dead s-dead" stroke-width="1"/><circle cx="346.1" cy="325.0" r="3.5" class="f-dead s-dead" stroke-width="1"/><circle cx="280.3" cy="296.7" r="3.5" class="f-dead s-dead" stroke-width="1"/><circle cx="236.2" cy="269.7" r="3.5" class="f-dead s-dead" stroke-width="1"/><circle cx="172.3" cy="258.1" r="3.5" class="f-dead s-dead" stroke-width="1"/><circle cx="175.5" cy="245.3" r="3.5" class="f-dead s-dead" stroke-width="1"/><circle cx="155.5" cy="223.2" r="3.5" class="f-dead s-dead" stroke-width="1"/><circle cx="129.2" cy="241.8" r="3.5" class="f-dead s-dead" stroke-width="1"/><circle cx="132.6" cy="288.8" r="3.5" class="f-dead s-dead" stroke-width="1"/><circle cx="119.8" cy="301.7" r="3.5" class="f-dead s-dead" stroke-width="1"/><circle cx="88.3" cy="235.9" r="3.5" class="f-dead s-dead" stroke-width="1"/><circle cx="56.0" cy="140.3" r="3.5" class="f-dead s-dead" stroke-width="1"/><circle cx="380.3" cy="151.5" r="3.5" class="f-pk s-pk" stroke-width="1"/><circle cx="371.0" cy="154.6" r="3.5" class="f-pk s-pk" stroke-width="1"/><circle cx="373.5" cy="200.0" r="3.5" class="f-pk s-pk" stroke-width="1"/><circle cx="373.8" cy="274.3" r="3.5" class="f-pk s-pk" stroke-width="1"/><circle cx="353.0" cy="325.5" r="3.5" class="f-pk s-pk" stroke-width="1"/><circle cx="316.4" cy="344.0" r="3.5" class="f-pk s-pk" stroke-width="1"/><circle cx="254.8" cy="306.3" r="3.5" class="f-pk s-pk" stroke-width="1"/><circle cx="183.1" cy="271.0" r="3.5" class="f-pk s-pk" stroke-width="1"/><circle cx="162.1" cy="247.7" r="3.5" class="f-pk s-pk" stroke-width="1"/><circle cx="140.9" cy="225.1" r="3.5" class="f-pk s-pk" stroke-width="1"/><circle cx="126.4" cy="235.5" r="3.5" class="f-pk s-pk" stroke-width="1"/><circle cx="126.1" cy="274.8" r="3.5" class="f-pk s-pk" stroke-width="1"/><circle cx="119.3" cy="300.3" r="3.5" class="f-pk s-pk" stroke-width="1"/><circle cx="95.9" cy="264.3" r="3.5" class="f-pk s-pk" stroke-width="1"/><circle cx="67.2" cy="195.7" r="3.5" class="f-pk s-pk" stroke-width="1"/><circle cx="349.5" cy="66.0" r="5" class="f-phys s-phys" stroke-width="1.5"/><circle cx="102.5" cy="117.1" r="5" class="f-phys s-phys" stroke-width="1.5"/><text x="357.5" y="58.0" class="t-small">t = 70 s</text><text x="94.5" y="133.1" class="t-small" text-anchor="end">t = 84 s</text><rect x="510.0" y="20.0" width="350.0" height="380.0" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="526.0" y="44.0" class="t-b">Chú giải</text><path d="M526.0 66.0 L556.0 66.0" class="f-none s-phys" stroke-width="3"/><text x="566.0" y="70.0">đường thật (mô phỏng)</text><circle cx="541.0" cy="94.0" r="3.5" class="f-dead s-dead" stroke-width="1"/><text x="566.0" y="98.0">điểm GPS thô</text><circle cx="541.0" cy="122.0" r="3.5" class="f-pk s-pk" stroke-width="1"/><text x="566.0" y="126.0">sau Kalman</text><text x="526.0" y="166.0" class="t-b">Sai số của riêng 15 điểm này</text><text x="526.0" y="190.0" class="t-mono">RMSE thô:    10,84 m</text><text x="526.0" y="208.0" class="t-mono">RMSE Kalman: 11,63 m</text><text x="526.0" y="244.0" class="t-small">Không phải thống kê: 15 điểm của một</text><text x="526.0" y="258.0" class="t-small">chuyến. Số cả tập (26.392 điểm):</text><text x="526.0" y="272.0" class="t-small">9,22 m thô và 8,87 m Kalman.</text><text x="526.0" y="308.0" class="t-small">Kalman gần đường thật hơn điểm thô</text><text x="526.0" y="322.0" class="t-small">ở 6 trong 15 điểm; ở các điểm còn lại</text><text x="526.0" y="336.0" class="t-small">nó kém hơn hoặc ngang: bám theo sai số</text><text x="526.0" y="350.0" class="t-small">chậm của GPS chứ không xoá nó.</text><path d="M30.0 376.0 L116.5 376.0" class="f-none s-muted" stroke-width="2"/><text x="124.5" y="380.0" class="t-small">10 m</text><text x="850.0" y="390.0" class="t-small" text-anchor="end">Vẽ từ output mục 6 của loc15_kalman; mặt phẳng mét cục bộ.</text></svg>
</div>
<figcaption>Hình 3. Khúc cua gắt nhất của chuyến báo cáo số 0 (xe 3,8 m/s, σ = 6 m, tương quan 8 s), vẽ từ 15 dòng CSV mà lab in ra. Trên đúng 15 điểm này RMSE thô 10,84 m, sau Kalman 11,63 m; đây là một ví dụ để nhìn cơ chế, không phải thống kê.</figcaption>
</figure>

Nếu chấp nhận chờ, có cách đỡ lệch hơn: **làm mượt** (smoothing). Bộ lọc chỉ dùng các điểm đã qua; bộ làm mượt Rauch–Tung–Striebel quay lại sau khi đã biết thêm vài điểm tương lai, và sửa ước lượng cũ bằng thông tin đó. Cơ chế: sau khi lọc xuôi, đi ngược từng bước; ở bước `j`, ước lượng làm mượt là ước lượng đã lọc cộng một hiệu chỉnh `C · (x_mượt[j+1] − x_dự_đoán[j+1])`, với `C = P_lọc[j] · Fᵀ · (P_dự_đoán[j+1])⁻¹`. Hiệu chỉnh này lớn khi bước sau lệch nhiều so với dự đoán và phần lớn độ bất định của dự đoán đến từ chính trạng thái `j`, chứ không từ nhiễu quá trình. Bản của lab có độ trễ cố định `lag`: chỉ đi ngược `lag` bước, và dừng ở chỗ khởi tạo lại bộ lọc, vì sau một khoảng mất tín hiệu dài không còn liên hệ giữa hai bên.

Tệp `examples/toi-noi-location/rust/crates/map-match/src/kalman.rs` (dòng 206–227):

```rust
/// Làm mượt Rauch–Tung–Striebel có trễ cố định: vị trí (đông, bắc) mét của bước `k` khi đã biết thêm `lag` bước sau nó.
/// `lag = 0` trả đúng kết quả lọc. Chuỗi dừng ở bước khởi tạo lại (`reset`), vì khi đó không có liên hệ giữa hai bên.
pub fn rts_fixed_lag(steps: &[Step], k: usize, lag: usize) -> [f64; 2] {
    let mut end = k;
    while end < k + lag && end + 1 < steps.len() && !steps[end + 1].reset {
        end += 1;
    }
    let mut xs = steps[end].x_filt;
    for j in (k..end).rev() {
        let next = &steps[j + 1];
        let ft = [[1.0, 0.0], [next.dt, 1.0]];
        let c = mul(mul(mat(steps[j].p_filt), ft), inv(mat(next.p_pred)));
        for (a, x) in xs.iter_mut().enumerate() {
            let d = [x[0] - next.x_pred[a][0], x[1] - next.x_pred[a][1]];
            *x = [
                steps[j].x_filt[a][0] + c[0][0] * d[0] + c[0][1] * d[1],
                steps[j].x_filt[a][1] + c[1][0] * d[0] + c[1][1] * d[1],
            ];
        }
    }
    [xs[0][0], xs[1][0]]
}
```

Kết quả đo (σ = 6 m tương quan 8 s, bộ báo cáo, nhiễu mô phỏng):

| Trễ L (giây, 1 Hz) | RMSE (m) | So với lọc thường |
|---:|---:|---:|
| 0 | 8,870 | 0,0% |
| 1 | 8,214 | −7,4% |
| 2 | 8,041 | −9,3% |
| 3 | 8,046 | −9,3% |
| 5 | 8,064 | −9,1% |
| 12 | 8,059 | −9,2% |

Chờ 2 giây là đủ; chờ thêm không có lợi. Lý do nằm ngay trong tham số. Tính tay từ công thức, với `accel_sigma` = 4 và `R` = 36 ở 1 Hz, `K` ổn định cỡ 0,68, và trọng số của điểm đo sau 1, 2, 3 bước lên vị trí hiện tại chỉ còn cỡ 0,22, 0,07, 0,02. Điểm xa hơn gần như không được dùng, dù nhiễu thật là trắng hay tương quan. Phần RMSE còn lại (8,04 m) thì lab chưa tách được bao nhiêu là do sai số tương quan. So với điểm thô 9,22 m, mức 8,04 m là giảm khoảng 12,8% (tính từ hai số trong output). Nhưng cái giá là hiển thị chậm 2 giây, nên làm mượt hợp với việc ghi lịch sử hoặc bước bám đường offline, không hợp với marker đang chạy trên màn hình khách. Bài 17 gặp lại đúng sự đánh đổi này ở dạng «độ trễ quyết định» của HMM.

## Khi nào không cần Kalman

Mục 5 của lab gom những tình huống thử bộ lọc ở chỗ nó yếu. RMSE tính bằng mét, từ điểm thô sang Kalman, mọi dòng trên nhiễu mô phỏng của lab:

| Tình huống | RMSE thô | RMSE Kalman | Thay đổi |
|---|---:|---:|---:|
| GPS tốt: σ = 2 m, nhiễu trắng | 4,53 | 3,10 | −31,6% |
| σ = 6 m, tương quan 8 s (mặc định) | 9,22 | 8,87 | −3,7% |
| σ = 20 m, tương quan 8 s | 28,59 | 29,23 | +2,2% |
| lệch ổn định 15 m suốt 40% chuyến (hẻm cao tầng), accuracy không đổi | 13,23 | 12,99 | −1,8% |
| lấy mẫu thưa 5 giây (cùng tham số) | 9,42 | 9,40 | −0,2% |
| `accel_sigma` = 0,25 (quá mượt so với gia tốc thật) | 9,22 | 13,19 | +43,1% |
| `accel_sigma` = 8 (bám sát đo) | 9,22 | 9,04 | −1,9% |

Hãy đọc bảng này như danh sách những điều **không** nên kỳ vọng ở Kalman. Kalman không sửa được độ lệch ổn định (khu nhà cao tầng đẩy cả đoạn về một phía, giảm 1,8%). Kalman không giúp khi lấy mẫu thưa (suy từ công thức, chưa kiểm riêng: với `accel_sigma` = 4, sau 5 giây riêng nhiễu quá trình đã cho phương sai vị trí 2.500 m², so với `R` cỡ 36 m², nên `K` gần 1 và bộ lọc gần như chép lại điểm đo). Và Kalman rất nhạy với `accel_sigma`: chọn mô hình quá mượt (0,25) làm RMSE tệ hơn 43,1%, nên tham số này phải được chọn bằng dữ liệu, không đoán.

Trong loạt này, Kalman là bước tuỳ chọn trước khi bám đường. Ở bài 16, chạy HMM trên điểm đã lọc chỉ thêm 0,4 điểm phần trăm cạnh nới nút (87,26% so với 86,87% ở σ = 10 m) và 1,1 điểm cạnh chặt (70,21% so với 69,09%), trên simulator của chính lab. Muốn dùng Kalman để làm marker mượt hơn cho khách thì phải chấp nhận sai số có hướng ở trên; muốn dùng để giảm sai cạnh thì lợi ích đo được rất nhỏ. Quyết định có đưa Kalman vào đường nóng nên dựa trên dữ liệu GPS thật, thứ ta chưa có.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` do lab bài 06 ghi (hoặc thêm `--fixture` để chạy trên đồ thị tổng hợp nhỏ). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc15_kalman
```

Cấu hình đo: `--release`, Apple M1 Max, đồ thị HCM 126.699 nút và 319.074 cạnh có hướng (OSM, © OpenStreetMap contributors), 40 chuyến mỗi bộ, 1 Hz, seed báo cáo 20261106, seed chọn tham số 1106. Output (rút gọn: bảng chọn tham số lược bớt, giữ phần chọn và mục 2):

```text
chuyến chọn tham số: 29734 điểm thật; chuyến báo cáo: 26392 điểm thật

-- 1. chọn gia tốc nhiễu quá trình trên bộ chọn tham số (σ 6 m, tương quan 8 s, có điểm nhảy ngẫu nhiên) --
…
chọn: accel_sigma = 4, gate_nis = Some(9.21) (RMSE tune 8.810 m); mặc định trong KalmanConfig::default(): accel_sigma = 4, gate_nis = Some(9.21)

-- 2. RMSE so với vị trí thật, theo độ lệch chuẩn nhiễu mỗi trục (bộ báo cáo, tham số đã chọn ở mục 1) --
     σ (m) tương quan   RMSE thô  RMSE Kalman       giảm      p95 thô p95 Kalman
         2      8 s       4.52         3.43      24.2%       5.01       5.17
         4      8 s       6.68         6.08       9.0%      10.03      10.23
         6      8 s       9.22         8.87       3.7%      15.04      15.35
        10      8 s      14.62        14.59       0.2%      25.05      25.41
        20      8 s      28.59        29.23      -2.2%      49.78      51.07
         6    trắng       9.20         7.07      23.2%      15.00      12.08
        10    trắng      14.58        10.77      26.1%      24.98      18.68

```

Các số trong bài này là RMSE, tỉ lệ và số đếm nên tái lập từng chữ số với cùng seed; bài **không có số thời gian nào**, nên không bị nhiễu do máy chịu tải (lúc chạy chính thức tải 1 phút khoảng 69 trên 10 lõi, ghi trong output). Kiểm tra mã: `cargo test --manifest-path examples/toi-noi-location/rust/Cargo.toml -p map-match` chạy các test của `kalman.rs` (bám vận tốc không đổi mà không lệch, bỏ điểm đến muộn và khởi tạo lại sau khoảng trống, vắng accuracy khác 0, làm mượt có trễ không tệ hơn lọc thường) và property test «Kalman theo đúng một đường thẳng không nhiễu». Chạy toàn bộ: `bash examples/toi-noi-location/verify-p02-rust.sh`.

## Giới hạn của ví dụ

- **Không có GPS người thật.** Nhiễu AR(1) có hằng số 8 giây, accuracy hiệu chuẩn tốt và điểm nhảy xa đều do lab sinh. Mọi tỉ lệ giảm RMSE là trên mô hình này; đừng đem con số 3,7% hay 23,2% ra làm kỳ vọng cho Tới Nơi.
- **Kalman không biết đường.** Ước lượng đã lọc vẫn có thể nằm trên vỉa hè hay ngoài đường; việc ép vào mạng đường là bám đường (bài 16, 17).
- **`accel_sigma` chọn một lần** ở σ = 6 m tương quan, rồi dùng cho mọi σ; chưa chọn lại theo từng mức nhiễu.
- **Chưa dùng speed và heading do app báo**, chưa thử xe máy (tăng, phanh và dừng khác ô tô), chưa thử bộ lọc đa mô hình.
- **Chưa kiểm chứng** rằng phần lớn mức giảm ở σ = 2 m đến từ việc dập điểm nhảy (suy luận từ số học, xem bài tập 3).
- **Không đo thời gian chạy của bộ lọc** trong bài này; lab không có số µs cho Kalman.

## Bài tập

**Bài 1.** Dự đoán vị trí là 52 m với phương sai `P'` = 16, điểm đo cách dự đoán 8 m với phương sai `R` = 36. Tính `K` và vị trí ước lượng. Nếu `R` tăng lên 144 thì ước lượng nằm ở đâu?

<details><summary>Lời giải bài 1</summary>
<p><code>K</code> = 16 / (16 + 36) = 0,308; ước lượng = 52 + 0,308 × 8 = 54,46 m. Với <code>R</code> = 144: <code>K</code> = 16 / 160 = 0,1; ước lượng = 52 + 0,1 × 8 = 52,8 m. Điểm đo kém tin cậy hơn thì bộ lọc gần như bỏ qua nó, nên accuracy của từng điểm là đầu vào chứ không phải trang trí.</p>
</details>

**Bài 2.** Làm mượt RTS giảm 9,3% RMSE ở độ trễ 2 giây rồi không giảm thêm, dù có thêm điểm sau. Vì sao? Nếu nhiễu là trắng mà giữ nguyên tham số, bạn dự đoán chỗ dừng đó thay đổi thế nào (đây là dự đoán, lab chưa đo)?

<details><summary>Lời giải bài 2</summary>
<p>Trọng số của điểm đo tương lai giảm theo cấp số nhân, và chỉ tham số bộ lọc quyết định nó: với <code>accel_sigma</code> = 4, <code>R</code> = 36 là cỡ 0,22, 0,07, 0,02, 0,007 ở bước 1, 2, 3, 4 (tính tay). Trọng số không phụ thuộc nhiễu thật, nên với nhiễu trắng và cùng tham số, đường cong cũng phẳng ở cỡ 2 đến 3 bước; chỉ mức giảm có thể lớn hơn, vì các điểm sau mang thông tin độc lập. Muốn chỗ phẳng đến muộn hơn phải giảm <code>accel_sigma</code> (0,25 cho cỡ 0,19, 0,14, 0,11, 0,08), đổi lại lệch nhiều hơn khi xe đổi vận tốc. Muốn biết thật, thêm một dòng thử nhiễu trắng vào mục 4 của lab và chạy lại.</p>
</details>

**Bài 3.** Đề xuất một lần chạy để kiểm chứng giả thuyết «mức giảm 24,2% ở σ = 2 m chủ yếu là nhờ dập điểm nhảy». Bạn kỳ vọng thấy gì nếu giả thuyết đúng, và thấy gì nếu sai?

<details><summary>Lời giải bài 3</summary>
<p>Chạy lại mục 2 với <code>p_outlier = 0</code> trong <code>NoiseConfig</code> (hàm <code>noise</code> của lab). Nếu giả thuyết đúng, mức giảm ở σ = 2 m tương quan 8 s phải tụt mạnh, có thể về cỡ vài phần trăm như σ = 6 m. Nếu mức giảm vẫn gần 24%, thì phần lớn lợi ích đến từ làm mượt và giả thuyết sai. Ghi kết quả vào <code>results/</code> kèm lệnh, đúng quy tắc «số trong bài phải có output sinh ra nó».</p>
</details>

Bài trước: [bài 14](/articles/location-validate-gps-timestamp-accuracy-gia-mao). Bài tiếp: [bài 16](/articles/location-map-matching-hmm-viterbi-rust).

## Tài liệu gốc

- R. E. Kalman, «A New Approach to Linear Filtering and Prediction Problems», Journal of Basic Engineering, tập 82, số 1, 1960, trang 35–45 (DOI 10.1115/1.3662552): bài báo gốc của bộ lọc Kalman.
- H. E. Rauch, F. Tung, C. T. Striebel, «Maximum Likelihood Estimates of Linear Dynamic Systems», AIAA Journal, tập 3, số 8, 1965, trang 1445–1450 (DOI 10.2514/3.3166): nguồn của bộ làm mượt RTS dùng ở trên.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho dữ liệu đồ thị HCM.
