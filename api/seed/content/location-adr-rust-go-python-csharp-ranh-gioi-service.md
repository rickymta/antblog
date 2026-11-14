---
slug: location-adr-rust-go-python-csharp-ranh-gioi-service
title: "ADR: Rust, Go, Python và C# mỗi bên làm gì trong service định vị"
summary: "Bài 02: ADR phân vai bốn ngôn ngữ. Rust không kém về thuật toán; Go và Python được giữ ở những chỗ chúng có lợi thật; Rust đề xuất, C# quyết định; cái giá của bốn ngôn ngữ và khi nào nên gộp lại."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, go, python, csharp, adr, grpc]
language: vi
---

Tài xế DR-001 bật app. Một điểm GPS đi từ điện thoại vào `location-ingest` viết bằng Go, qua Kafka tới `geo-engine` viết bằng Rust. Khi khách CU-001 đặt xe, `geo-engine` tính ra ba chỗ đón khả dĩ và gửi về `Dispatch` viết bằng C#, nơi một chuyến TN-0001 được gán đúng một tài xế và ghi xuống database. Tối hôm đó, một pipeline Python đọc lịch sử GPS của cả ngày để cập nhật mô hình ETA mà `geo-engine` sẽ nạp vào sáng mai. Một chuyến đi qua bốn ngôn ngữ và ba ranh giới mạng, còn mọi bên dùng chung một file `.proto`.

Có người sẽ hỏi ngay: sao không viết tất cả bằng một ngôn ngữ cho đỡ phiền? Đó là câu hỏi chính đáng, và câu trả lời cũng phải được viết ra thay vì để ngầm hiểu. Bài này là một ADR (architecture decision record): ghi bối cảnh, quyết định, các phương án đã loại, hệ quả và điều kiện để xem lại. Đây là bài 02 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi), đọc sau bài [bài toán quãng đường của hai người](/articles/location-bai-toan-quang-duong-khach-va-tai-xe).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Đồ thị đường bộ TP.HCM của lab (126.699 nút, 319.074 cạnh có hướng) nằm trong một file nhị phân 18,4 MB. Bạn nghĩ nạp file này vào bộ nhớ mất khoảng bao lâu, và điều đó nói gì về việc có cần tách bản đồ ra một service riêng không?</p>
<details><summary>Xem đáp án</summary><p>Theo <code>results/p01.md</code>, nạp lại file mất <b>9–10 ms</b> ở bốn trong năm lần chạy có ghi lại và 24 ms ở lần còn lại, đều đo khi máy đang chịu tải nặng. Vì vậy một process có thể giữ nguyên bản đồ cả thành phố trong RAM và nạp lại gần như tức thì khi khởi động. Đó là một trong các lý do để đặt bản đồ vào một service duy nhất sở hữu nó (<code>geo-engine</code>) thay vì để mỗi service tự giữ một bản.</p></details>
</div>

## Bối cảnh

Hồ sơ hư cấu của Tới Nơi đưa ra các con số làm cơ sở cân nhắc: 6 thành phố, 25.000 tài xế trực tuyến lúc cao điểm, mỗi người gửi một điểm GPS mỗi 4 giây, tức khoảng 6.250 điểm mỗi giây chỉ để ghi vị trí. Số này không phải số của một công ty thật mà là số được chọn để khớp với nhau trong cả blog. Service định vị phải làm bốn việc khác tính chất:

- **Hứng** vị trí từ rất nhiều kết nối dài hạn, mỗi kết nối gửi ít nhưng đều.
- **Tính** trên bản đồ: bám đường, định tuyến, chọn điểm đón, ghép chuyến. Việc này ngốn CPU, cần cả đồ thị trong bộ nhớ, và độ trễ đuôi (p99) ảnh hưởng thẳng tới trải nghiệm đặt xe.
- **Học** từ dữ liệu cũ: ETA, dự báo nhu cầu, cụm điểm đón hay dùng. Việc này chạy theo lô, không chặn khách.
- **Quyết định** ai lái chuyến nào: làm theo giao dịch, truy vết được, không được gán đôi.

Ngoài ra còn hai ràng buộc không thuộc về kỹ thuật. Blog đã có loạt C# về các service nghiệp vụ chính của Tới Nơi, nên phần quyết định đã có chủ. Và loạt này, cùng các loạt Rust và Python qua Tới Nơi, muốn dùng đúng công cụ mà mỗi ngôn ngữ làm tốt, thay vì chứng minh một ngôn ngữ làm được mọi thứ.

## Quyết định

| Thành phần | Ngôn ngữ | Sở hữu gì | Vì sao ở đây |
|---|---|---|---|
| `geo-engine`: bám đường, định tuyến, ma trận chi phí, chọn điểm đón, gợi ý ghép chuyến | **Rust** | Bản đồ đã biên dịch, vị trí mới nhất | Nhạy với độ trễ và bộ nhớ, đồ thị lớn trong RAM, kiểu toạ độ bắt lỗi lúc biên dịch |
| `location-ingest`, `tracking-hub` | **Go** | Kết nối, hàng đợi có giới hạn | Rất nhiều kết nối, goroutine, backpressure; mất một điểm GPS không làm lệch tiền |
| Pipeline offline: ETA, dự báo nhu cầu, phân cụm điểm đón, phân tích thí nghiệm | **Python** | Dữ liệu huấn luyện, mô hình ONNX, bảng điểm đón học được | Hệ sinh thái dữ liệu và solver, không nằm trên đường nóng |
| `Dispatch`, `Trip` | **C#** (đã có) | Nguồn dữ liệu gốc của assignment | Nghiệp vụ và transaction vẫn thuộc backend chính |
| App khách, app tài xế | **Flutter** (đã có) | Hiển thị | Dùng lại thành quả của loạt Flutter |

Ranh giới quan trọng nhất không nằm giữa hai ngôn ngữ mà nằm giữa hai động từ: **Rust đề xuất, C# quyết định**. Hình 1 vẽ ranh giới đó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 372" role="img" aria-label="Hình 1. Ranh giới giữa bốn ngôn ngữ. App tài xế gửi GPS tới location-ingest viết bằng Go, qua topic Kafka location.updates.v1 tới geo-engine viết bằng Rust, và tới tracking-hub viết bằng Go cho app khách. Dispatch viết bằng C# gọi geo-engine qua gRPC để xin đề xuất và là bên quyết định assignment, ghi vào kho của Dispatch. Pipeline Python chạy offline trên lịch sử GPS, xuất mô hình ONNX và bảng điểm đón cho geo-engine nạp."><defs><marker id="location-adr-rust-go-python-csharp-ranh-gioi-service-0-m0" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker><marker id="location-adr-rust-go-python-csharp-ranh-gioi-service-0-m1" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="location-adr-rust-go-python-csharp-ranh-gioi-service-0-m2" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="50" width="130" height="62" rx="8" class="f-surface s-line"/><text x="85" y="76" class="t-b" text-anchor="middle">App tài xế</text><text x="85" y="95" class="t-small" text-anchor="middle">Flutter</text><rect x="185" y="50" width="150" height="62" rx="8" class="f-surface s-line"/><text x="260" y="76" class="t-b" text-anchor="middle">location-ingest</text><text x="260" y="95" class="t-small" text-anchor="middle">Go</text><rect x="370" y="50" width="170" height="62" rx="8" class="f-surface s-line"/><text x="455" y="76" class="t-b" text-anchor="middle">location.updates.v1</text><text x="455" y="95" class="t-small" text-anchor="middle">Kafka, key driver_id</text><rect x="575" y="50" width="130" height="62" rx="8" class="f-phys-soft s-phys"/><text x="640" y="76" class="t-b" text-anchor="middle">geo-engine</text><text x="640" y="95" class="t-small" text-anchor="middle">Rust</text><rect x="740" y="50" width="120" height="62" rx="8" class="f-pk-soft s-pk"/><text x="800" y="76" class="t-b" text-anchor="middle">Dispatch</text><text x="800" y="95" class="t-small" text-anchor="middle">C#</text><path d="M150 81 L183 81" class="f-none s-muted" stroke-width="2" marker-end="url(#location-adr-rust-go-python-csharp-ranh-gioi-service-0-m0)"/><path d="M335 81 L368 81" class="f-none s-muted" stroke-width="2" marker-end="url(#location-adr-rust-go-python-csharp-ranh-gioi-service-0-m0)"/><path d="M540 81 L573 81" class="f-none s-muted" stroke-width="2" marker-end="url(#location-adr-rust-go-python-csharp-ranh-gioi-service-0-m0)"/><path d="M707 81 L738 81" class="f-none s-pk" stroke-width="2" marker-end="url(#location-adr-rust-go-python-csharp-ranh-gioi-service-0-m1)" marker-start="url(#location-adr-rust-go-python-csharp-ranh-gioi-service-0-m1)"/><path d="M722 48 L722 262" class="f-none s-muted dash" stroke-width="1"/><text x="714" y="18" class="t-b" text-anchor="end">Rust đề xuất</text><text x="730" y="18" class="t-b">C# quyết định</text><text x="722" y="42" class="t-small" text-anchor="middle">gRPC: SuggestPickup, SuggestMatch (dự kiến)</text><rect x="185" y="168" width="150" height="62" rx="8" class="f-surface s-line"/><text x="260" y="194" class="t-b" text-anchor="middle">App khách</text><text x="260" y="213" class="t-small" text-anchor="middle">Flutter</text><rect x="370" y="168" width="170" height="62" rx="8" class="f-surface s-line"/><text x="455" y="194" class="t-b" text-anchor="middle">tracking-hub</text><text x="455" y="213" class="t-small" text-anchor="middle">Go, fan-out theo chuyến</text><rect x="740" y="168" width="120" height="62" rx="8" class="f-pk-soft s-pk"/><text x="800" y="194" class="t-b" text-anchor="middle">Kho assignment</text><text x="800" y="213" class="t-small" text-anchor="middle">của Dispatch</text><path d="M455 114 L455 166" class="f-none s-muted" stroke-width="2" marker-end="url(#location-adr-rust-go-python-csharp-ranh-gioi-service-0-m0)"/><path d="M368 199 L337 199" class="f-none s-muted" stroke-width="2" marker-end="url(#location-adr-rust-go-python-csharp-ranh-gioi-service-0-m0)"/><path d="M800 114 L800 166" class="f-none s-pk" stroke-width="2" marker-end="url(#location-adr-rust-go-python-csharp-ranh-gioi-service-0-m1)"/><text x="20" y="270" class="t-b">Offline, không nằm trên đường nóng</text><rect x="185" y="284" width="150" height="62" rx="8" class="f-surface s-line"/><text x="260" y="310" class="t-b" text-anchor="middle">Lịch sử GPS</text><text x="260" y="329" class="t-small" text-anchor="middle">Parquet</text><rect x="370" y="284" width="170" height="62" rx="8" class="f-surface s-line"/><text x="455" y="310" class="t-b" text-anchor="middle">Pipeline Python</text><text x="455" y="329" class="t-small" text-anchor="middle">ETA, nhu cầu, cụm điểm đón</text><rect x="575" y="284" width="170" height="62" rx="8" class="f-surface s-line"/><text x="660" y="310" class="t-b" text-anchor="middle">ONNX + bảng điểm đón</text><text x="660" y="329" class="t-small" text-anchor="middle">nạp vào geo-engine</text><path d="M335 315 L368 315" class="f-none s-muted" stroke-width="2" marker-end="url(#location-adr-rust-go-python-csharp-ranh-gioi-service-0-m0)"/><path d="M540 315 L573 315" class="f-none s-muted" stroke-width="2" marker-end="url(#location-adr-rust-go-python-csharp-ranh-gioi-service-0-m0)"/><path d="M640 282 L640 114" class="f-none s-phys dash" stroke-width="2" marker-end="url(#location-adr-rust-go-python-csharp-ranh-gioi-service-0-m2)"/></svg>
</div>
<figcaption>Hình 1. Ranh giới giữa các ngôn ngữ. Đường đứt dọc là ranh giới quan trọng nhất: bên trái geo-engine chỉ đề xuất, bên phải Dispatch quyết định và ghi vào kho của mình. Nét đứt màu xanh ngọc là đường offline: Python huấn luyện, Rust nạp kết quả, Python không nằm trên đường nóng. Tên RPC là kế hoạch của loạt, chưa có trong file <code>.proto</code> hiện tại.</figcaption>
</figure>

Phần còn lại của bài giải thích từng dòng của bảng, trước hết là câu hay bị hiểu nhầm nhất.

## Rust không yếu về thuật toán

Để không ai đọc nhầm bảng trên: **Go và Python không được dùng vì Rust kém thuật toán.** Chỉ mục không gian, đồ thị đường bộ, contraction hierarchies, ghép cặp Kuhn–Munkres đều có thư viện và chạy tốt trên Rust; nhiều thư viện trong số đó, như `rstar`, `h3o`, `fast_paths`, `pathfinding`, là thư viện loạt bài này dùng hoặc đem ra đối chiếu. Vì vậy thuật toán trên đường nóng ở lại Rust.

Lý do cụ thể để `geo-engine` là Rust:

- **Đồ thị lớn trong bộ nhớ, bố cục dự đoán được.** Đồ thị TP.HCM của lab là các cột mảng liền nhau (CSR). Theo `results/p01.md`, file nhị phân 18,4 MB, nạp lại 9–10 ms (24 ms một lần; đo khi máy đang chịu tải nặng); mảng cột ước tính 17,5 MB.
- **Độ trễ đuôi.** Khi cả bản đồ đã ở trong RAM, p99 phụ thuộc vào thuật toán chứ không phụ thuộc vào một bộ thu gom rác. Đây là lý do thiết kế; lab **chưa** đo so với ngôn ngữ khác và các bài định tuyến ở P03 mới là nơi đo p99 thật.
- **Sai kiểu toạ độ bị chặn lúc biên dịch.** Bài 03 cho thấy `LatLng::new(Lng, Lat)` không biên dịch.
- Một lý do thành thật hơn: loạt này chọn Rust cho phần lõi vì mục tiêu của nó là dạy cách viết engine bản đồ bằng Rust. Nếu mục tiêu là sản phẩm, C# hay Go hoàn toàn có thể giữ đồ thị trong bộ nhớ; chênh lệch nằm ở mức kiểm soát bố cục dữ liệu và độ ổn định của p99, thứ cần đo chứ không nên khẳng định.

Phần đã đo tới lúc này còn khiêm tốn: một phép khoảng cách haversine tốn vài chục nano giây trên máy của tác giả (đo khi máy chịu tải nặng, nên chỉ nên nhớ thứ bậc chứ không nhớ con số), và việc nạp đồ thị như trên. Chưa có số nào cho Dijkstra hay contraction hierarchies, vì chúng nằm ở các bài sau.

## Vì sao Go ở ingest và tracking-hub

Ingest có hình dạng rất riêng: 25.000 kết nối WebSocket sống lâu, mỗi kết nối chỉ gửi một lô vài điểm sau mỗi vài giây, gần như không tính toán. Mất một điểm GPS không làm lệch tiền; điểm mới hơn sẽ thay điểm cũ. Mô hình goroutine cho mỗi kết nối, kênh có giới hạn và `context` để huỷ là cách diễn đạt gần nhất với bài toán. Bài LOC-13 về backpressure sẽ nói kỹ cách bỏ có chủ đích khi quá tải.

Lab đã dựng bản Go này. Các số dưới đây lấy từ `results/p02-go.md`, mục LOC-12 (Apple M1 Max 10 lõi, Go 1.27.1, 25.000 kết nối, seed 1), kèm các hạn chế đi theo chúng:

| Mục | Kết quả |
|---|---|
| Tải 6.250 điểm/s, lô 3 điểm, Kafka thật, 25.000 kết nối | đạt 6.250 điểm/s, 0 lỗi, 0 điểm bị bỏ |
| Ack p50, p95, p99 (phía client) | 0,28 ms, 6,25 ms, 10,25 ms; lớn nhất 51,38 ms |
| CPU của ingest | 19% của **một** lõi |
| Bộ nhớ | RSS đỉnh 1.020 MB cho 25.012 goroutine, khoảng 41 KB mỗi kết nối |
| Cùng tải nhưng lô 1 điểm (không batching) | CPU 29%, p99 ack 30,38 ms |
| Tăng dần tải (sink bỏ Kafka) | theo kịp tới 50.000 điểm/s (p99 219 ms); ở 100.000 điểm/s p99 vọt lên 6.141 ms và có lỗi |

Hình 2 vẽ lại cột p99 theo mức tải.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 380" role="img" aria-label="Hình 2. Độ trễ ack p99 của location-ingest khi tăng tải, trục dọc thang logarit từ 10 mili giây, một lần chạy trên một máy với 25.000 kết nối. Với sink discard: 23,15 mili giây ở 6.250 điểm mỗi giây, 16,39 ở 12.500, 31,92 ở 25.000, 219,02 ở 50.000 và 6.140,54 ở 100.000. Với Kafka thật: 24,32; 18,54; 49,78; 396,15 và 9.115,67. Ở 100.000 điểm mỗi giây ingest không còn theo kịp."><path d="M100 320 L830 320" class="f-none s-line" stroke-width="1"/><text x="92" y="324" class="t-small" text-anchor="end">10 ms</text><path d="M100 236.7 L830 236.7" class="f-none s-line" stroke-width="1"/><text x="92" y="240.7" class="t-small" text-anchor="end">100 ms</text><path d="M100 153.3 L830 153.3" class="f-none s-line" stroke-width="1"/><text x="92" y="157.3" class="t-small" text-anchor="end">1 s</text><path d="M100 70 L830 70" class="f-none s-line" stroke-width="1"/><text x="92" y="74" class="t-small" text-anchor="end">10 s</text><path d="M100 153.3 L830 153.3" class="f-none s-warn dash" stroke-width="1"/><text x="108" y="147.3" class="t-small">ngưỡng «theo kịp»: p99 dưới 1 giây</text><rect x="98" y="289.6" width="48" height="30.4" class="f-phys s-phys"/><rect x="154" y="287.8" width="48" height="32.2" class="f-pk s-pk"/><text x="122" y="283.6" class="t-small" text-anchor="middle">23,15</text><text x="178" y="281.8" class="t-small" text-anchor="middle">24,32</text><text x="150" y="340" class="t-b" text-anchor="middle">6.250</text><rect x="238" y="302.1" width="48" height="17.9" class="f-phys s-phys"/><rect x="294" y="297.7" width="48" height="22.3" class="f-pk s-pk"/><text x="262" y="296.1" class="t-small" text-anchor="middle">16,39</text><text x="318" y="291.7" class="t-small" text-anchor="middle">18,54</text><text x="290" y="340" class="t-b" text-anchor="middle">12.500</text><rect x="378" y="278" width="48" height="42" class="f-phys s-phys"/><rect x="434" y="261.9" width="48" height="58.1" class="f-pk s-pk"/><text x="402" y="272" class="t-small" text-anchor="middle">31,92</text><text x="458" y="255.9" class="t-small" text-anchor="middle">49,78</text><text x="430" y="340" class="t-b" text-anchor="middle">25.000</text><rect x="518" y="208.3" width="48" height="111.7" class="f-phys s-phys"/><rect x="574" y="186.8" width="48" height="133.2" class="f-pk s-pk"/><text x="542" y="202.3" class="t-small" text-anchor="middle">219,02</text><text x="598" y="180.8" class="t-small" text-anchor="middle">396,15</text><text x="570" y="340" class="t-b" text-anchor="middle">50.000</text><rect x="658" y="87.6" width="48" height="232.4" class="f-warn s-warn"/><rect x="714" y="73.4" width="48" height="246.6" class="f-warn s-warn"/><text x="682" y="81.6" class="t-small" text-anchor="middle">6.140,54</text><text x="738" y="67.4" class="t-small" text-anchor="middle">9.115,67</text><text x="710" y="340" class="t-b" text-anchor="middle">100.000</text><text x="430" y="360" class="t-small" text-anchor="middle">điểm GPS mỗi giây (mục tiêu của bộ tạo tải)</text><rect x="100" y="22" width="12" height="12" class="f-phys s-phys"/><text x="118" y="32" class="t-small">sink discard (bỏ Kafka ra)</text><rect x="300" y="22" width="12" height="12" class="f-pk s-pk"/><text x="318" y="32" class="t-small">Kafka thật, một broker trong Docker</text><rect x="560" y="22" width="12" height="12" class="f-warn s-warn"/><text x="578" y="32" class="t-small">không theo kịp</text></svg>
</div>
<figcaption>Hình 2. Độ trễ ack p99 của <code>location-ingest</code> (Go) theo mức tải. Số lấy từ <code>results/p02-go.md</code>, mục LOC-12, mỗi cột là <b>một lần chạy</b> trên một máy, loadgen và Kafka dùng chung máy đó, dữ liệu do simulator sinh. Thang dọc là logarit, bắt đầu từ 10 ms, nên chiều cao cột không tỉ lệ tuyến tính với giá trị. Ở mức tải 100.000, cả hai cột đều tô cam vì không theo kịp.</figcaption>
</figure>

Có bốn lưu ý bắt buộc khi trích các số này. **Một**, mỗi con số là một lần chạy trên một máy, chưa lặp, chưa có khoảng tin cậy. **Hai**, `loadgen`, ingest và Kafka (một broker trong Docker) chạy chung một máy qua loopback và chia sẻ CPU, nên đây không phải số của riêng ingest và không gồm độ trễ mạng thật. **Ba**, kết quả này không ghi lại tải của máy lúc đo, nên phần thời gian chỉ nên đọc như thứ bậc. **Bốn**, dữ liệu do simulator sinh, không có GPS người thật. Nguyên nhân hệ thống gãy ở 100.000 điểm/s cũng chưa được tách bạch trọn vẹn.

Và có một điều các số này **không** nói: chưa có bản ingest bằng Rust nào để so. Không thể kết luận Go nhanh hơn hay tiết kiệm hơn Rust ở việc này. Rust với `tokio` làm được đúng loại việc ấy. Điều giữ Go lại ở đây là mô hình lập trình khớp với bài toán (goroutine viết theo kiểu đồng bộ, đọc như mã tuần tự), một ranh giới service nhỏ gọn mà một người có thể nắm trọn, và một lý do thứ hai cũng thành thật như ở Rust: blog chưa có loạt Go, nên bài nhập môn LOC-11 mở đường. Phần «Khi nào gộp lại» ở dưới đề ra thí nghiệm có thể đổi quyết định này.

## Vì sao Python ở phía offline

Việc của Python là mọi thứ chạy theo lô trên dữ liệu đã nằm yên: huấn luyện ETA, dự báo nhu cầu theo ô H3, phân cụm điểm đón từ lịch sử, phân tích thí nghiệm hai phía, và dựng baseline bằng solver (OR-Tools) để so với heuristic của Rust. Với những việc này, theo đánh giá của loạt bài, hệ sinh thái dữ liệu và solver của Python trưởng thành hơn và người đọc dễ tự mở rộng hơn. Pipeline này không phục vụ yêu cầu của khách nên độ trễ của một lần chạy tính bằng phút không thành vấn đề.

Cách hai bên gặp nhau cố ý hẹp: Python ghi **mô hình ONNX** và **bảng điểm đón học được**, Rust nạp. Python không nằm trên đường nóng, còn Rust không cần biết cách huấn luyện. Bài LOC-53 sẽ làm khâu ONNX này; ở thời điểm viết ADR, khâu đó **chưa chạy** và chưa có số đo. Hai điều cần nhớ: mô hình trong loạt này huấn luyện trên dữ liệu simulator, chỉ chứng minh **phương pháp**, không chứng minh độ chính xác ngoài đời; và điểm bất thường GPS do Go gắn cờ chỉ là tín hiệu, còn kết luận gian lận là việc của Fraud service Python, không phải của ingest.

Trên máy của tác giả, Python hệ thống là 3.9 nên không dùng; pipeline chạy trong project `uv` với Python 3.13. Đó là một ví dụ nhỏ của cái giá sẽ nói ở phần sau.

## Rust đề xuất, C# quyết định

`geo-engine` trả lời câu hỏi «chỗ nào nên đón, ghép ai với ai», ví dụ ba điểm đón xếp hạng kèm `policy_version` đã dùng, qua gRPC. Hợp đồng dự kiến có `Route`, `Matrix`, `SuggestPickup` và `SuggestMatch`; tới lúc này `.proto` mới có các message GPS và tracking, chưa có service gRPC nào, nên tên RPC chỉ là kế hoạch. Dispatch bằng C# nhận danh sách đó, kiểm tra lại, và **chỉ Dispatch** ghi assignment chính thức.

Lý do của ranh giới này có bốn.

1. **Chủ sở hữu trạng thái.** Theo kiến trúc tổng thể (mục 4.5), một booking chỉ có một assignment hợp lệ tại một thời điểm, offer có TTL và `offer_id`, thao tác accept/reject phải idempotent. Đó là transaction và invariant nghiệp vụ, thứ loạt C# đã dạy. Một engine bản đồ không nên phải biết chúng.
2. **Đề xuất có thể cũ.** Khi gói tin gRPC tới nơi, tài xế đầu danh sách có thể vừa nhận chuyến khác. Nếu `geo-engine` tự ghi assignment thì nó phải biết hết luật tranh chấp; nếu chỉ đề xuất thì Dispatch từ chối một đề xuất cũ là chuyện bình thường.
3. **Trạng thái của `geo-engine` dựng lại được.** Vị trí mới nhất đến từ Kafka và bản đồ đến từ file; mất tiến trình thì nạp lại. Dữ liệu dựng lại được thì không nên làm nguồn dữ liệu gốc.
4. **Nguyên tắc chung của dự án:** service sở hữu dữ liệu của mình, không service nào đọc trực tiếp kho của service khác.

Hệ quả thực tế: nếu `geo-engine` chậm hay lỗi, Dispatch vẫn gán được chuyến bằng phương án lùi (một bài sau sẽ làm: khoảng cách chim bay nhân hệ số vòng) với chất lượng kém hơn; còn nếu cho nó tự ghi assignment thì lỗi của bản đồ sẽ trở thành lỗi về tiền.

## Hợp đồng chung và cái giá của bốn ngôn ngữ

Các service nói chuyện với nhau qua một file duy nhất, `examples/toi-noi-location/proto/location/v1/location.proto`, và một topic Kafka `location.updates.v1` với key là `driver_id`. File `examples/toi-noi-location/proto/README.md` ghi các quy ước số mà **mọi ngôn ngữ phải hiểu giống nhau**: toạ độ là `lat_e7` và `lng_e7` kiểu `sint32` (độ nhân 10⁷, 1 đơn vị khoảng 1,1 cm), thời gian là `int64` mili giây UTC, độ chính xác tính bằng 0,1 m. Nguyên tắc tương thích theo mục 6.6 của kiến trúc: không đổi ý nghĩa hay số thứ tự field đã có, field mới thêm ở cuối, đổi không tương thích thì tạo `location.updates.v2`.

Có hợp đồng rõ không có nghĩa là bốn ngôn ngữ miễn phí. Chi phí cụ thể:

| Chi phí | Biểu hiện trong lab | Cách giảm |
|---|---|---|
| Bốn bộ công cụ | Rust 1.98.1 (edition 2024), Go 1.27.1, Python 3.13 qua `uv`, .NET SDK; phiên bản pin trong lockfile hoặc script | CI chạy riêng từng ngôn ngữ; phiên bản ghi trong README của lab |
| Môi trường không đồng đều | Trên máy của tác giả, `cargo` không có sẵn trên PATH, Python hệ thống 3.9 không dùng được, và không có `protoc` nên phải dùng `buf` cùng `protoc-gen-go` do script cài | Script `generate.sh` ghim phiên bản; file sinh ra được commit |
| Một nghĩa, bốn cách hiểu | Đơn vị (E7, 0,1 m, mili giây) phải khớp ở cả bốn nơi; một chỗ nhầm đơn vị là lỗi không bị trình biên dịch nào bắt | Một file `.proto`, bảng quy ước số, test hợp đồng giữa các ngôn ngữ (chưa viết) |
| Truy vết xuyên ngôn ngữ | Một chuyến đi qua WebSocket, Kafka, gRPC; mỗi nơi truyền trace theo một cách riêng | Bài LOC-62 sẽ làm W3C propagation xuyên Go, Rust, C# |
| Con người | Người trực ca phải đọc được cả bốn mã nguồn khi có sự cố | Giữ mỗi service nhỏ, một chủ, ranh giới rõ |
| Mô hình lỗi | Timeout, retry và mã lỗi mỗi nơi một kiểu | Quy ước gRPC chung: deadline bắt buộc, retry chỉ với thao tác an toàn |

Không dòng nào trong bảng là rào cản chặn đường, nhưng tất cả đều tốn công mỗi tuần làm việc. Đó là lý do ADR này có phần dưới cùng.

## Các phương án đã cân nhắc

| Phương án | Được | Mất | Kết luận |
|---|---|---|---|
| Tất cả bằng Rust | Một toolchain, hợp đồng kiểu chung | Phải tự dựng phần dữ liệu và solver; mất cơ hội dạy Go và Python; kéo logic giao dịch ra khỏi C# | Loại |
| Tất cả bằng C# | Một nơi giữ nghiệp vụ và bản đồ | C# làm được, nhưng đồ thị lớn cần kiểm soát bố cục bộ nhớ và độ ổn định p99, và loạt này chọn đo phần đó ở Rust | Loại cho `geo-engine`; giữ cho Dispatch |
| `geo-engine` bằng Go | Cùng ngôn ngữ với ingest | Chưa có số đo nào cho thấy lợi thế; lõi bản đồ của loạt này viết bằng Rust | Loại, nhưng không phải vì Go yếu thuật toán |
| Chia theo trách nhiệm (đề xuất) | Mỗi nơi dùng công cụ hợp việc, ranh giới rõ | Bốn ngôn ngữ, chi phí ở bảng trên | **Chọn** |

## Khi nào gộp lại

Quyết định nào cũng cần điều kiện để đảo ngược. Ba tín hiệu sau khiến ADR này đáng xem lại, và mỗi tín hiệu gắn với một việc có thể kiểm.

1. **Go cho ingest không còn lợi thế đo được.** Thí nghiệm: viết `location-ingest` bằng Rust, cùng giao thức, cùng `loadgen`, cùng chuỗi mức tải (6.250, 12.500, 25.000, 50.000, 100.000 điểm/s), cùng Kafka, chạy mỗi cấu hình **nhiều lần** trên máy rảnh và ghi `uptime`. So sánh p99 ở 6.250 và 50.000, CPU, RSS mỗi kết nối. Ngưỡng chấp nhận phải viết ra **trước** khi chạy. Nếu bản Rust ngang bản Go thì lý do còn lại của Go chỉ là thói quen, và gộp vào Rust bớt được một toolchain. Chưa ai làm thí nghiệm này, nên chưa có kết luận.
2. **Chi phí hợp đồng vượt lợi ích.** Nếu nhiều lần liên tiếp phải sửa cả bốn ngôn ngữ vì một thay đổi không liên quan tới nghiệp vụ, hoặc không còn đủ người trực ca đọc nổi mã của một trong bốn nơi, thì thu hẹp số ngôn ngữ.
3. **Python chỉ còn làm việc đơn giản.** Nếu ETA là một bảng tra theo ô H3 và giờ, hoặc một hồi quy tuyến tính, thì Rust sinh được bảng đó và không cần Python ở bước này. Ngược lại, mô hình cây hay solver lớn thì Python ở lại.

Có hai thứ không nên gộp. C# không gộp vào Rust, vì ranh giới đề xuất/quyết định chính là điểm cần giữ. Và nếu ở giai đoạn đầu các thành phần được gộp thành một deployable, kiến trúc tổng thể (mục 18) cho phép làm vậy miễn là vẫn giữ ranh giới module, quyền sở hữu schema và hợp đồng rõ: **gộp tiến trình thì được, bỏ hợp đồng thì không**.

## Giới hạn của ví dụ

Bài này không có code. Các con số về Go lấy từ lab LOC-12, mỗi con số là một lần chạy trên một máy chia sẻ với `loadgen` và Kafka, không có bản Rust để đối chiếu, nên chúng minh hoạ chi phí của lựa chọn Go chứ không chứng minh Go là lựa chọn tốt nhất. Phần Rust mới có số cho khoảng cách và việc nạp đồ thị; chưa có số cho định tuyến. Phần Python và hợp đồng gRPC chưa chạy: `.proto` hiện chưa có service nào, chưa có test hợp đồng giữa các ngôn ngữ, chưa có trace xuyên suốt. Quy mô tải là số hư cấu của hồ sơ dự án mẫu. Mọi việc ở trên là việc của các bài sau.

## Bài tập

1. Viết trước tiêu chí cho thí nghiệm «ingest bằng Rust» ở mục trên: phải giữ cố định những gì, đo những gì, ghi thêm gì, và vì sao ngưỡng phải viết trước khi chạy.

<details><summary>Lời giải</summary>

Giữ cố định: giao thức và hợp đồng protobuf, `loadgen` và seed, số kết nối (25.000), kích thước lô, Kafka và số partition, máy chạy. Đo: p50, p95, p99 ack ở từng mức tải, tốc độ đạt được so với mục tiêu, CPU của tiến trình, RSS mỗi kết nối, số lỗi. Ghi thêm: `uptime` lúc chạy, số lần lặp, chạy cả sink bỏ Kafka lẫn Kafka thật. Ngưỡng phải viết trước vì nếu chọn ngưỡng sau khi thấy kết quả thì ta sẽ vô thức chọn ngưỡng sao cho phương án mình muốn sẽ thắng; viết trước biến thí nghiệm thành một phép kiểm có thể thất bại.

</details>

2. `geo-engine` đề xuất chỗ đón và tài xế DR-001 cho chuyến TN-0001. Hai giây sau khi gói gRPC được gửi đi, DR-001 nhận một chuyến khác. Dispatch phải làm gì, và vì sao đây là lý do để `geo-engine` không ghi assignment?

<details><summary>Lời giải</summary>

Dispatch kiểm tra lại trạng thái hiện tại, thấy DR-001 đã bận, từ chối đề xuất đó và xin đề xuất khác (hoặc lấy ứng viên kế tiếp trong danh sách đã nhận). Đề xuất là thông tin có thời hạn, không phải lệnh. Nếu `geo-engine` tự ghi assignment, nó phải biết luật tranh chấp, idempotency và TTL của offer, tức là một nửa nghiệp vụ Dispatch, và hai nơi cùng ghi sẽ dẫn tới gán đôi.

</details>

3. Một đồng nghiệp đề nghị gộp `tracking-hub` (Go) vào `geo-engine` (Rust) cho đỡ một service. Hãy nêu một lý do ủng hộ, một lý do phản đối, và cho biết bạn cần số đo gì để quyết.

<details><summary>Lời giải</summary>

Ủng hộ: bớt một process, một lần truyền qua Kafka, bớt một ngôn ngữ. Phản đối: tracking-hub giữ rất nhiều kết nối dài hạn và có chính sách riêng (chỉ mở sau khi gán, coalesce theo client chậm); gộp thì tải đẩy dữ liệu qua các kết nối sẽ làm ảnh hưởng độ trễ tính toán của engine, và hai thứ có nhịp deploy, scale khác nhau. Số cần đo: độ trễ p99 của truy vấn điểm đón khi có và không có tải fan-out cùng process, cùng một máy, lặp nhiều lần.

</details>

Bài trước: [bài 01](/articles/location-bai-toan-quang-duong-khach-va-tai-xe). Bài tiếp: [bài 03](/articles/location-toa-do-wgs84-newtype-lat-lng-do-chinh-xac).

## Tài liệu gốc

- [Architecture Decision Records](https://adr.github.io/): cách ghi và lưu một ADR.
- [Effective Go: Concurrency](https://go.dev/doc/effective_go#concurrency): goroutine và channel.
- [Tokio tutorial](https://tokio.rs/tokio/tutorial): mô hình bất đồng bộ của Rust, để so với goroutine.
- [Protocol Buffers: Overview](https://protobuf.dev/overview/) và [gRPC: Deadlines](https://grpc.io/docs/guides/deadlines/): hợp đồng và deadline giữa các service.
- [ONNX](https://onnx.ai/): định dạng mô hình dùng để Python giao cho Rust.
- `docs/toi-noi-overall-architecture.md`, mục 4.5, 6.6, 7 và 18; `docs/location-master-curriculum.md`, mục 2; `examples/toi-noi-location/results/p02-go.md`, mục LOC-12.
