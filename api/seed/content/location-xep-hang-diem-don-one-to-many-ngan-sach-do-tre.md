---
slug: location-xep-hang-diem-don-one-to-many-ngan-sach-do-tre
title: "Xếp hạng điểm đón trong ngân sách độ trễ"
summary: "Bài 35: tính J cho mọi ứng viên từ ma trận, trả top 3, chấm lại bằng A* theo cạnh; ngân sách 120 ms cho p99 bị vượt (145,9 ms CPU); chưa biết tài xế thì dùng phân bố quanh đó; bất biến không tệ hơn điểm ghim theo J."
category: system-design
level: advanced
tags: [toi-noi, location, rust, pickup, latency, p99, proptest, ranking]
language: vi
---

Khách bấm «Đặt xe». Trước khi app hiện giá và điểm đón, service phải xếp hạng vài chục ứng viên điểm đón bằng `J` và trả về ba điểm tốt nhất. Quote có ngân sách p95 dưới 500 ms (kiến trúc §14.1), và việc xếp hạng chỉ là một phần của ngân sách đó, trước cả bước định giá, gọi Dispatch và mạng di động. Bài này đặt cho riêng việc xếp hạng một ngân sách, đo nó, và kết luận thẳng thắn: **ngân sách đó đã bị vượt**.

Đây là bài 35 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài ghép các mảnh của bài trước: hàm `J` của [bài 32](/articles/location-ham-muc-tieu-diem-hen-khach-va-tai-xe), danh sách ứng viên của [bài 33](/articles/location-sinh-ung-vien-diem-don-do-thi-di-bo), xếp hạng hai tầng của [bài 34](/articles/location-phia-duong-huong-tiep-can-tranh-quay-dau), và ma trận many-to-many của [bài 26](/articles/location-ma-tran-many-to-many-bucket-ch). Mọi số đo trên **dữ liệu mô phỏng**: kịch bản `hcm-baseline`, đồ thị TP.HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors), 150 yêu cầu và 120 tài xế rảnh của seed 20270001, chính sách `p04-v1`, `RankConfig` mặc định (mỗi ứng viên một tài xế rảnh gần nó nhất theo chim bay, đúng như bộ ghép baseline; chấm lại 4 ứng viên). **Số thời gian đo khi máy đang chịu tải rất nặng**, nên chúng nhiễu và phóng đại.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bộ xếp hạng hai tầng chấm 4 ứng viên đầu và điểm ghim bằng A* theo cạnh. Theo bạn phần nào chiếm phần lớn thời gian: ma trận cho mọi ứng viên, hay A* cho chỉ một ít ứng viên? Và chế độ mặc định có nằm trong ngân sách p99 120 ms không?</p>
<details><summary>Xem đáp án</summary><p>A* theo cạnh chiếm phần lớn: mỗi tuyến A* mất khoảng 2 ms CPU, trong khi cả ma trận của một yêu cầu mất 1,9 ms ở p50. Chế độ mặc định có p99 thời gian CPU là 145,9 ms, <b>vượt</b> ngân sách 120 ms.</p></details>
</div>

## Từ ma trận tới top 3

Mỗi yêu cầu đi qua bốn bước. Một, sinh ứng viên bằng Dijkstra trên đồ thị đi bộ (bài 33). Hai, hai ma trận many-to-many: tài xế tới từng ứng viên và từng ứng viên tới điểm trả. Ba, tính `J1` cho mọi ứng viên và cắt xuống `refine_top`. Bốn, chấm lại ứng viên đứng đầu và điểm ghim bằng A\* theo cạnh, ra `J2`. Kết quả là `top_n` = 3 vị trí khác nhau, `J` tăng dần. Một yêu cầu mẫu (khách ở 10,764460; 106,682161; ghim 10,764698; 106,682173; 120 tài xế rảnh, 44 ứng viên hợp lệ, 6 chấm lại):

| Hạng | Đi bộ | Tài xế | Đoạn có khách | Phạt | **J** | Đi bộ (m) |
|---|---:|---:|---:|---:|---:|---:|
| #1 | 47,9 s | 253,8 s | 491,8 s | 0,0 s | **1.047,4** | 62 |
| #2 | 52,7 s | 259,4 s | 486,3 s | 0,0 s | 1.057,8 | 69 |
| #3 | 63,5 s | 239,9 s | 537,3 s | 0,0 s | 1.080,5 | 83 |
| điểm ghim | | | | | 1.137,3 | |

Kiểm #1 bằng tay: 47,9 + max(47,9; 253,8) + 253,8 + 491,8 = 1.047,3, đúng với 1.047,4 khi làm tròn. Hạng #3 có thời gian tài xế ngắn nhất (239,9 s) nhưng đoạn có khách dài nhất (537,3 s), nên không thắng. Ở yêu cầu này điểm ghim kém điểm tốt nhất 89,9 s trong đơn vị của J.

Hai tầng không phải ý tưởng tối ưu hóa chung chung; nó là câu trả lời cho một ràng buộc cụ thể: ma trận không có chi phí rẽ (bài 34), A\* theo cạnh thì có nhưng đắt. Phần còn lại của bài đo «đắt» là bao nhiêu.

## Đặt ngân sách, rồi đo

**Ngân sách:** 120 ms cho p99 của một lần xếp hạng, tính theo **thời gian CPU của luồng**. Lý do chọn CPU chứ không phải đồng hồ treo tường: máy đo dùng chung nhiều việc và chịu tải rất nặng, nên đồng hồ treo tường bị tính cả thời gian luồng chờ lượt chạy. Lab đo cả hai bằng cách bọc một lần xếp hạng:

`examples/toi-noi-location/rust/labs/src/bin/loc35_xep_hang.rs`

```rust
        for rq in &reqs {
            let (s, c) = (Instant::now(), ThreadTime::now());
            let r = run_one(e, &mut ranker, &es, rq, &live);
            tc.push(c.elapsed().as_micros() as f64);
            t.push(s.elapsed().as_micros() as f64);
            routes += r.stats.n_edge_routes;
```

Bốn chế độ, mỗi chế độ 150 yêu cầu liên tiếp trên một luồng (trên dữ liệu mô phỏng, seed 20270001, **đo khi máy đang chịu tải nặng**; thời gian tính theo ms):

| Chế độ | CPU p50 | CPU p95 | **CPU p99** | Tường p99 | Tuyến A\*/yêu cầu |
|---|---:|---:|---:|---:|---:|
| chỉ ma trận (tầng 1) | 1,9 | 4,0 | 5,5 | 73,6 | 0,0 |
| hai tầng, top 4 + ghim (mặc định) | 23,7 | 100,0 | **145,9** | 473,0 | 11,5 |
| hai tầng, top 8 + ghim | 36,8 | 152,9 | 243,0 | 916,0 | 19,3 |
| top 8 có hạn chót 120 ms cho tầng 2 | 27,9 | 66,3 | 87,4 | 278,1 | 15,0 |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 430" role="img" aria-label="Hình 1. Thời gian CPU của luồng cho một lần xếp hạng, 150 yêu cầu seed 20270001, đo khi máy chịu tải nặng: chế độ mặc định hai tầng có p99 145,9 ms, vượt ngân sách 120 ms; chỉ khi có hạn chót cho tầng 2 thì p99 xuống 87,4 ms nhưng 64 trên 150 yêu cầu bị cắt."><rect x="20" y="20" width="840" height="390" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Độ trễ CPU của một lần xếp hạng</text><text x="40.0" y="72.0" class="t-small">Thời gian CPU của luồng (ms); mỗi chế độ ba cột: p50 (nhạt), p95 (xanh), p99 (đậm, cam nếu vượt ngân sách)</text><path d="M438.0 100.0 L438.0 380.0" class="f-none s-warn dash" stroke-width="1.5"/><text x="444.0" y="96.0" class="t-small">ngân sách 120 ms cho p99</text><text x="40.0" y="142.0">chỉ ma trận (tầng 1)</text><rect x="270.0" y="112.0" width="2.7" height="14.0" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="680.0" y="124.0" class="t-mono">p50 1,9 ms</text><rect x="270.0" y="130.0" width="5.6" height="14.0" rx="2" class="f-pk" stroke-width="0"/><text x="680.0" y="142.0" class="t-mono">p95 4,0 ms</text><rect x="270.0" y="148.0" width="7.7" height="14.0" rx="2" class="f-phys" stroke-width="0"/><text x="680.0" y="160.0" class="t-mono">p99 5,5 ms</text><text x="40.0" y="210.0">hai tầng, top 4 + ghim (mặc định)</text><rect x="270.0" y="180.0" width="33.2" height="14.0" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="680.0" y="192.0" class="t-mono">p50 23,7 ms</text><rect x="270.0" y="198.0" width="140.0" height="14.0" rx="2" class="f-pk" stroke-width="0"/><text x="680.0" y="210.0" class="t-mono">p95 100,0 ms</text><rect x="270.0" y="216.0" width="204.3" height="14.0" rx="2" class="f-warn" stroke-width="0"/><text x="680.0" y="228.0" class="t-mono">p99 145,9 ms</text><text x="40.0" y="278.0">hai tầng, top 8 + ghim</text><rect x="270.0" y="248.0" width="51.5" height="14.0" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="680.0" y="260.0" class="t-mono">p50 36,8 ms</text><rect x="270.0" y="266.0" width="214.1" height="14.0" rx="2" class="f-pk" stroke-width="0"/><text x="680.0" y="278.0" class="t-mono">p95 152,9 ms</text><rect x="270.0" y="284.0" width="340.2" height="14.0" rx="2" class="f-warn" stroke-width="0"/><text x="680.0" y="296.0" class="t-mono">p99 243,0 ms</text><text x="40.0" y="346.0">top 8 + hạn chót 120 ms cho tầng 2</text><rect x="270.0" y="316.0" width="39.1" height="14.0" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="680.0" y="328.0" class="t-mono">p50 27,9 ms</text><rect x="270.0" y="334.0" width="92.8" height="14.0" rx="2" class="f-pk" stroke-width="0"/><text x="680.0" y="346.0" class="t-mono">p95 66,3 ms</text><rect x="270.0" y="352.0" width="122.4" height="14.0" rx="2" class="f-phys" stroke-width="0"/><text x="680.0" y="364.0" class="t-mono">p99 87,4 ms</text></svg>
</div>
<figcaption>Hình 1. Thời gian CPU của luồng cho một lần xếp hạng, 150 yêu cầu seed 20270001, đo khi máy chịu tải nặng. Chế độ mặc định hai tầng có p99 145,9 ms, vượt ngân sách 120 ms (đường đứt nét); chỉ ma trận chỉ 5,5 ms. Chế độ có hạn chót cho tầng 2 giữ p99 ở 87,4 ms nhưng cắt tầng 2 ở 64 trên 150 yêu cầu.</figcaption>
</figure>

**Kết luận: ngân sách không đạt ở chế độ mặc định.** p99 CPU 145,9 ms vượt 120 ms cỡ 22%. Đo với 6 luồng chạy cùng lúc (6 × 150 yêu cầu) cho p99 CPU 138,8 ms, cũng vượt. Bài giữ nguyên 120 ms (`BUDGET_MS` trong mã) thay vì nới theo số đo: kết quả là chưa đạt.

Những gì kèm theo con số này, và làm nó khó nói chắc:

- **Máy đang chịu tải rất nặng** (load averages 323,21 343,67 371,99 lúc bắt đầu). Thời gian CPU của luồng ít nhiễu hơn đồng hồ treo tường, nhưng vẫn phụ thuộc cache và xung nhịp khi các tiến trình khác tranh chấp. Tường p99 473 ms (hai tầng) so với 73,6 ms (ma trận) là cận trên bị phóng đại. **Chưa đo trên máy rảnh.**
- **p99 CPU chưa bao gồm chờ I/O hay mạng** của một dịch vụ thật, nên con số của một dịch vụ thật chỉ có thể lớn hơn.
- **A\* theo cạnh chưa được tối ưu**: mỗi tuyến chạy lại từ đầu. Phần đắt là A\*, không phải ma trận: ~2 ms CPU mỗi tuyến so với 1,9 ms cho cả ma trận của một yêu cầu. Với 11,5 tuyến trung bình mỗi yêu cầu, p50 là 23,7 ms. Có chỗ để tối ưu (dùng chung kết quả giữa các tuyến cùng nguồn, gom các tuyến cùng đích), nhưng lab chưa làm và bài không hứa nó đạt ngân sách.
- **Chế độ có hạn chót** giữ p99 CPU ở 87,4 ms, trong ngân sách, nhưng đổi lại: tầng 2 bị cắt ở 64 trên 150 yêu cầu (43%). Hạn chót tính bằng đồng hồ treo tường nên tải máy có thể làm tăng số lần cắt; lab chưa đo trên máy rảnh nên chưa biết giảm bao nhiêu. Lab **chưa đo** chất lượng ứng viên khi tầng 2 bị cắt (hối tiếc ở các yêu cầu bị cắt), nên không biết «trong ngân sách» này có làm điểm đón tệ đi bao nhiêu.

Không có ai được lợi ở đây theo nghĩa quãng đường; đây là **chi phí của việc tính**. Ai trả: khách chờ lâu hơn trước khi thấy điểm đón (nếu dịch vụ không cắt), hoặc nhận điểm đón chất lượng kém hơn (nếu cắt bằng hạn chót). Chọn giữa hai điều này là quyết định sản phẩm.

## Chưa biết tài xế thì tính thế nào

Số hạng `T_drive` cần biết tài xế nào sẽ tới, nhưng khi khách vừa bấm đặt xe, bộ ghép chưa chạy. Rust chỉ **đề xuất**; Dispatch bằng C# **quyết định** gán tài xế. Trong simulator, `JPickup` thấy mọi tài xế rảnh lúc khách đặt (`Live`), và chỉ khi không còn ai rảnh mới dùng phân bố quanh đó. Ở 120 xe điều đó không xảy ra lần nào trên 50 seed; ở 60 xe xảy ra ở 26% lượt chọn (1.158/4.408). Bộ xếp hạng có ba mức hiểu biết về tài xế:

`examples/toi-noi-location/rust/crates/pickup/src/rank.rs`

```rust
pub enum DriverBelief {
    /// Các tài xế đang rảnh (vị trí có hướng). Tài xế sẽ nhận chuyến được ước bằng `k_drivers` người gần nhất.
    Live(Vec<DriverRef>),
    /// Chưa biết tài xế nào: dùng `n` tài xế ảo rải đều quanh điểm ghim ở bán kính `radius_m` (phân bố quanh đó).
    Prior { n: usize, radius_m: f64 },
    /// Không có thông tin: số hạng tài xế bằng 0 cho mọi ứng viên.
    Unknown,
}
```

- `Live`: biết các tài xế rảnh và vị trí có hướng của họ. Với `DriverModel::NearestCrow` mặc định, mỗi ứng viên lấy đúng một tài xế rảnh gần nó nhất theo chim bay; `k_drivers` chỉ dùng cho `BestTime`.
- `Prior`: chưa biết, dùng 8 tài xế ảo rải đều quanh điểm ghim ở bán kính `radius_m`. Tầng 1 lấy trung bình cả 8; tầng 2 lấy trung bình 4 người cách đều (0°, 90°, 180°, 270°).
- `Unknown`: bỏ số hạng tài xế (`T_drive` = 0 cho mọi ứng viên).

Lab so các mức này trên 80 yêu cầu. «J thật» của một ứng viên là J chính xác tính với tài xế mà bộ ghép baseline sẽ gán cho điểm đó (rảnh gần nhất theo chim bay), và hối tiếc là J thật của điểm được chọn trừ J thật nhỏ nhất. Trên dữ liệu mô phỏng, seed 20270001:

| Mức hiểu biết | Chọn đúng | Hối tiếc TB (s) | p95 | max |
|---|---:|---:|---:|---:|
| biết tài xế rảnh, bộ ghép = gần điểm nhất (mặc định) | 86,2% | 3,23 | 30,15 | 65,69 |
| biết tài xế rảnh, **giả định sai** ghép nhanh nhất | 76,2% | 19,65 | 65,69 | 681,49 |
| biết đúng 1 tài xế gần khách nhất | 82,5% | 15,70 | 49,77 | 681,49 |
| phân bố quanh đó, bán kính 800 m | 48,8% | 36,31 | 153,42 | 681,49 |
| phân bố quanh đó, bán kính 1.500 m | 56,2% | 33,77 | 153,42 | 681,49 |
| phân bố quanh đó, bán kính 3.000 m | 56,2% | 33,39 | 153,42 | 681,49 |
| không biết gì (`T_drive` = 0) | 17,5% | 74,65 | 211,23 | 723,50 |
| đối chứng: luôn chọn điểm ghim | 10,0% | 64,78 | 225,96 | 811,26 |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Hình 2. Chưa biết tài xế: chọn đúng ứng viên tốt nhất trên 80 yêu cầu. Biết tài xế rảnh và mô hình hóa đúng bộ ghép: 86,2%; phân bố quanh đó: 48,8 đến 56,2%; không biết gì về tài xế: 17,5%, tệ hơn điểm ghim ở hối tiếc (74,7 s so với 64,8 s). Mô phỏng, seed 20270001."><rect x="20" y="20" width="840" height="360" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Chưa biết tài xế: phân bố quanh đó</text><text x="40.0" y="76.0" class="t-small">Tỉ lệ chọn đúng (80 yêu cầu, seed 20270001) và hối tiếc trung bình theo J thật</text><text x="690.0" y="76.0" class="t-small">hối tiếc TB</text><text x="40.0" y="104.0" class="t-small">biết tài xế, ghép gần nhất (mặc định)</text><rect x="300.0" y="90.0" width="344.8" height="20.0" rx="2" class="f-pk" stroke-width="0"/><text x="650.8" y="105.0">86,2%</text><text x="700.0" y="105.0" class="t-mono">3,23 s</text><text x="40.0" y="137.0" class="t-small">biết tài xế, giả định sai: ghép nhanh nhất</text><rect x="300.0" y="123.0" width="304.8" height="20.0" rx="2" class="f-dead" stroke-width="0"/><text x="610.8" y="138.0">76,2%</text><text x="700.0" y="138.0" class="t-mono">19,65 s</text><text x="40.0" y="170.0" class="t-small">biết đúng 1 tài xế gần khách nhất</text><rect x="300.0" y="156.0" width="330.0" height="20.0" rx="2" class="f-dead" stroke-width="0"/><text x="636.0" y="171.0">82,5%</text><text x="700.0" y="171.0" class="t-mono">15,70 s</text><text x="40.0" y="203.0" class="t-small">phân bố quanh đó, bán kính 800 m</text><rect x="300.0" y="189.0" width="195.2" height="20.0" rx="2" class="f-dead" stroke-width="0"/><text x="501.2" y="204.0">48,8%</text><text x="700.0" y="204.0" class="t-mono">36,31 s</text><text x="40.0" y="236.0" class="t-small">phân bố quanh đó, bán kính 1.500 m</text><rect x="300.0" y="222.0" width="224.8" height="20.0" rx="2" class="f-dead" stroke-width="0"/><text x="530.8" y="237.0">56,2%</text><text x="700.0" y="237.0" class="t-mono">33,77 s</text><text x="40.0" y="269.0" class="t-small">phân bố quanh đó, bán kính 3.000 m</text><rect x="300.0" y="255.0" width="224.8" height="20.0" rx="2" class="f-dead" stroke-width="0"/><text x="530.8" y="270.0">56,2%</text><text x="700.0" y="270.0" class="t-mono">33,39 s</text><text x="40.0" y="302.0" class="t-small">không biết gì (T_drive = 0)</text><rect x="300.0" y="288.0" width="70.0" height="20.0" rx="2" class="f-warn" stroke-width="0"/><text x="376.0" y="303.0">17,5%</text><text x="700.0" y="303.0" class="t-mono">74,65 s</text><text x="40.0" y="335.0" class="t-small">đối chứng: luôn chọn điểm ghim</text><rect x="300.0" y="321.0" width="40.0" height="20.0" rx="2" class="f-dead" stroke-width="0"/><text x="346.0" y="336.0">10,0%</text><text x="700.0" y="336.0" class="t-mono">64,78 s</text></svg>
</div>
<figcaption>Hình 2. Chọn đúng ứng viên có J thật nhỏ nhất theo mức hiểu biết về tài xế, 80 yêu cầu của seed 20270001, trên dữ liệu mô phỏng. Biết tài xế và mô hình hóa đúng bộ ghép: 86,2%. Phân bố quanh đó: 48,8 đến 56,2%. Không biết gì về tài xế: 17,5%, hối tiếc 74,7 s, tệ hơn cả luôn chọn điểm ghim (64,8 s).</figcaption>
</figure>

Ba điều:

- **Phân bố quanh đó tốt hơn điểm ghim trên mẫu này**: chọn đúng 48,8–56,2% so với 10,0%, hối tiếc 33–36 s so với 64,8 s; đổi bán kính 800, 1.500 hay 3.000 m chỉ làm kết quả đổi ít (80 yêu cầu một seed, không có khoảng tin cậy). Khi chưa có tài xế, một «tài xế trung bình» đã loại được phần nào các điểm tệ. Cái giá: J này không biết hướng tài xế thật tới.
- **Không biết gì về tài xế tệ hơn cả điểm ghim**: chọn đúng 17,5%, hối tiếc 74,65 s so với 64,78 s. Nếu J bỏ hẳn số hạng tài xế, nó tối ưu cho đi bộ và đoạn có khách, và có thể chọn điểm khiến mọi tài xế phải chạy xa. Lab không tách hối tiếc theo bên; giả thuyết hợp lý nhất là tài xế gánh phần thiệt đó.
- **Giả định sai bộ ghép vẫn làm mất chất lượng dù biết đủ tài xế.** Biết tài xế rảnh nhưng giả định ghép «nhanh nhất» trong khi bộ ghép thật chọn «gần nhất theo chim bay»: chọn đúng giảm từ 86,2% xuống 76,2% và hối tiếc trung bình tăng từ 3,23 lên 19,65 s (gấp hơn sáu lần). Cùng kết luận với bài 34: J phải khớp bộ ghép phía sau.

## Bất biến: không bao giờ tệ hơn điểm ghim, theo J

Hợp đồng của bộ xếp hạng: ứng viên tốt nhất có `J` không lớn hơn `J` của điểm ghim. Bất biến này **đúng theo cách xây bộ xếp hạng**, không phải một phát hiện. Điểm ghim luôn có mặt trong tầng 2, nên được chấm bằng đúng thước đo `J2` như mọi ứng viên, và `top` được sắp theo `J2` tăng dần, nên phần tử đầu không thể lớn hơn điểm ghim. Test kiểm điều đó trên đồ thị fixture:

`examples/toi-noi-location/rust/crates/pickup/tests/rank_props.rs`

```rust
        if let (Some(best), Some(pin_s)) = (r.top.first(), r.pin)
            && pin_s.cand.violation.is_none()
            && !r.stats.pin_forced
            && pin_s.j.is_finite()
        {
            prop_assert!(best.j <= pin_s.j + 1e-9, "best {} > pin {}", best.j, pin_s.j);
        }
```

Điều kiện đi kèm: điểm ghim không vi phạm ràng buộc cứng, không bị buộc dùng (`pin_forced`), và `J` hữu hạn. Với proptest (`cargo test -p pickup`, `tests/rank_props.rs`, 40 ca mỗi thuộc tính, tọa độ khách, điểm đến và tài xế sinh ngẫu nhiên, cả chế độ có và không có tầng 2), cùng bất biến được kiểm: J không giảm dọc danh sách, các vị trí khác nhau, ứng viên không phải ghim luôn qua ràng buộc cứng và nằm trong ngưỡng đi bộ ước lượng. Trên đồ thị TP.HCM, lab kiểm lại trên 150 yêu cầu: 147 có điểm ghim hợp lệ, **0 trường hợp** ứng viên tốt nhất có J lớn hơn J điểm ghim.

**Phải đọc bất biến này đúng cỡ của nó.** Nó nói «không tệ hơn theo `J`», và vì cách xây dựng nên hiển nhiên, nó là lưới an toàn chống lỗi lập trình (nếu ai đó sửa mã và làm điểm ghim rơi khỏi tầng 2, test đỏ), **không phải bằng chứng về lợi ích ngoài đời**. `J` là mô hình. Ở bài 32, `p04-v1` làm khách điển hình đi bộ thêm 26,5 m ở p50 và J vẫn «không tệ hơn điểm ghim» ở mọi yêu cầu; J của điểm ghim lớn hơn J điểm chọn trung bình 85,9 giây (trên 4.403 lượt chọn của 20 seed đánh giá, chỉ tính khi cả hai hữu hạn, và đó là giây quy đổi của J chứ không phải giây thật), và ở 12,5% lượt chọn điểm tốt nhất đúng là điểm ghim. Bất biến không kiểm tra cái J không đo (băng đường, an toàn, sự khó chịu của đi bộ thêm).

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` (hoặc `--fixture`). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run --release -q --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc35_xep_hang
```

Thêm `-- --belief-only` để chỉ chạy mục 3. Cấu hình: Apple Silicon, `--release`, một luồng cho phần đo từng chế độ, 6 luồng cho phần đo dưới tải, 150 yêu cầu; **`uptime` lúc bắt đầu: load averages 323,21 343,67 371,99** (máy đang chịu tải nặng). Số đếm và tỉ lệ tái lập được; thời gian chỉ so tương đối. Ngân sách đặt trong mã: `const BUDGET_MS: u64 = 120;`. Output (rút gọn):

```text
-- 1. độ trễ một lần xếp hạng, một luồng (µs) --
  chế độ                                       tường p50       p95       p99       max |   CPU p50       p95       p99       max |  tuyến A*
  chỉ ma trận (tầng 1)                              2000     45708     73593     88201 |      1862      3972      5495      5622 |       0.0
  hai tầng, A* top 4 + ghim (mặc định)             55917    289628    472996    537202 |     23748     99957    145864    150758 |      11.5
  hai tầng, top 8 có hạn chót 120 ms cho tầng 2    104033    216991    278132    326478 |     27894     66300     87375    110265 |      15.0   (bị cắt hạn chót: 64/150)
-- 2. đo dưới tải: nhiều luồng cùng xếp hạng (hai tầng mặc định) --
  6 luồng × 150 yêu cầu, tường: p50 66180 µs, p95 346627 µs, p99 504151 µs, max 662273 µs; CPU luồng: p50 22972 µs, p95 91593 µs, p99 138774 µs, max 153666 µs
  ngân sách đặt cho tầng xếp hạng: 120 ms cho p99 của một lần xếp hạng ... p99 CPU đo được VƯỢT ngân sách; p99 tường VƯỢT ngân sách
-- 4. bất biến: kết quả không bao giờ tệ hơn điểm ghim theo J --
  147 yêu cầu có điểm ghim hợp lệ: số lần ứng viên tốt nhất có J lớn hơn J điểm ghim = 0
```

Test: `cargo test --manifest-path examples/toi-noi-location/rust/Cargo.toml -p pickup` hoặc `bash examples/toi-noi-location/verify-p04.sh` chạy cả `rank_props.rs` trên fixture; `pickup` có 20 test đơn vị và 4 test trong `rank_props.rs`, đều qua ở lần chạy cuối.

## Giới hạn của ví dụ

- **Ngân sách không đạt** ở chế độ mặc định (p99 CPU 145,9 ms so với 120 ms); chế độ đạt ngân sách đánh đổi bằng việc cắt tầng 2 ở 43% yêu cầu, và chất lượng khi bị cắt chưa đo.
- **Máy đo tải rất nặng, chưa đo trên máy rảnh**; p99 CPU chưa gồm I/O, gRPC hay mạng. Một seed, 150 yêu cầu: p99 của 150 mẫu chỉ dựa trên hai giá trị lớn nhất.
- **A\* theo cạnh chưa tối ưu**; chưa thử CH theo cạnh hay chia sẻ kết quả giữa tuyến.
- **«Chưa biết tài xế» dùng 8 tài xế ảo** rải đều quanh điểm ghim; chưa thử phân bố theo mật độ thật hay dự báo từ lịch sử (P06). Lần chạy đầu của lab có lỗi: tầng 2 chỉ lấy hai hướng đầu (0°, 45°); đã sửa và đo lại mục này (`--belief-only`, xem `results/p04.md`), các số thời gian không đo lại vì chế độ mặc định không đi qua nhánh đó.
- **Bất biến đúng theo cấu tạo của bộ xếp hạng**, không chứng minh lợi ích thật; không kiểm thứ J không đo.
- **J phải khớp bộ ghép phía sau**; đổi bộ ghép ở P05 là đổi giả định này.

## Bài tập

**Bài 1.** Tính J của hạng #2 trong bảng yêu cầu mẫu bằng công thức `a = b = c = d = 1`, và kiểm số làm tròn.

<details><summary>Lời giải bài 1</summary>
<p>J = 52,7 + max(52,7; 259,4) + 259,4 + 486,3 + 0 = 52,7 + 259,4 + 259,4 + 486,3 = 1.057,8, đúng với giá trị in ra.</p>
</details>

**Bài 2.** Vì sao lab đo ngân sách bằng thời gian CPU của luồng thay vì đồng hồ treo tường, và con số nào của kết quả bạn vẫn không nên tin tuyệt đối?

<details><summary>Lời giải bài 2</summary>
<p>Máy dùng chung nhiều việc, tải trung bình hơn 300 trên 10 lõi, nên đồng hồ treo tường tính cả thời gian luồng bị chuyển khỏi CPU, làm p99 phình lên (473 ms so với 145,9 ms CPU ở chế độ mặc định). Thời gian CPU loại phần chờ đó, nhưng vẫn phụ thuộc cache chung và xung nhịp, và không gồm I/O hay mạng. Vì thế p99 CPU 145,9 ms là xấp xỉ, chưa phải con số của máy rảnh hay của dịch vụ thật; nên chỉ dùng nó để so tương đối giữa các chế độ và kết luận «chưa đạt ngân sách», không để cam kết SLO.</p>
</details>

**Bài 3.** Thiết kế một kiểm tra cho thấy bất biến «không tệ hơn điểm ghim theo J» *không đủ* để bảo đảm khách không bị thiệt. Mô tả đầu vào và điều bạn đo.

<details><summary>Lời giải bài 3</summary>
<p>Dựng một yêu cầu mà điểm ghim ở ngay chỗ khách đứng (đi bộ 0 m) còn ứng viên có J nhỏ hơn vài giây vì tài xế tới sớm hơn nhưng khách phải đi bộ 200 m. Bộ xếp hạng chọn ứng viên đó vì J(ứng viên) ≤ J(ghim), đúng bất biến, nhưng khách đi bộ thêm 200 m trong khi lợi ích là vài giây của tài xế. Đo: chênh <code>walk_m</code> theo từng khách (như phân tầng ở bài 32) và tỉ lệ khách đi thêm trên 50 m, thay vì chỉ so J.</p>
</details>

Bài trước: [bài 34](/articles/location-phia-duong-huong-tiep-can-tranh-quay-dau). Bài tiếp: [bài 36](/articles/location-diem-hen-dong-cap-nhat-khi-hai-ben-di-chuyen).

## Tài liệu gốc

- [Dean và Barroso, «The Tail at Scale»](https://research.google/pubs/the-tail-at-scale/): vì sao p99 quan trọng hơn trung bình ở dịch vụ nhiều tầng.
- [proptest trên docs.rs](https://docs.rs/proptest): thư viện kiểm thử thuộc tính dùng cho bất biến.
- [cpu-time trên docs.rs](https://docs.rs/cpu-time): `ThreadTime`, thời gian CPU của luồng hiện tại.
