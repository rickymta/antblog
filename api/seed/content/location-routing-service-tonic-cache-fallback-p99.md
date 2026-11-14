---
slug: location-routing-service-tonic-cache-fallback-p99
title: "Dịch vụ định tuyến: gRPC với tonic, deadline, cache H3, dự phòng và p99"
summary: "Bài 29: bọc engine thành dịch vụ gRPC Route và Matrix bằng tonic, deadline và giấy phép tính toán, cache theo ô H3 và khung giờ, dự phòng chim bay, client C# chạy thật, đo p50 và p99 khi máy chịu tải."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, tonic, grpc, cache, h3, fallback, csharp]
language: vi
---

Màn hình đặt xe của khách đang chờ báo giá. Backend C# gọi sang `geo-engine` để biết ETA, và nó gọi với một deadline: quá 250 ms thì khách thấy vòng xoay. Cùng lúc đó hàng chục khách khác ở quanh Chợ Bến Thành hỏi những cặp điểm gần như giống nhau. Engine tính một tuyến mất chừng một phần mười mili giây, nên tính toán không phải chỗ nghẽn. Chỗ nghẽn là hàng đợi, và câu hỏi của bài này là dịch vụ nên làm gì khi hàng đợi dài hơn deadline: báo lỗi, hay trả lời một con số kém chính xác nhưng nói rõ nó kém chính xác.

Đây là bài 29 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài bọc engine của [bài 26](/articles/location-ma-tran-many-to-many-bucket-ch) và [bài 27](/articles/location-toc-do-theo-gio-customizable-ch-ket-xe) bằng gRPC, dùng lại bộ ước lượng chim bay của [bài 28](/articles/location-self-host-osrm-valhalla-hay-map-provider) và ô H3 của [bài 08](/articles/location-h3-geohash-chi-muc-luoi-k-ring). Dịch vụ gọi thẳng `Engine`, chưa đi qua trait `RoutingProvider`. Mọi số đo là trên đồ thị HCM (© OpenStreetMap contributors), máy chủ và bộ tạo tải **cùng một máy đang chịu tải nặng**, nên chỉ so sánh tương đối trong cùng một lần chạy.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Gọi thẳng hàm tuyến trong tiến trình mất p50 0,105 ms. Với 64 client song song gọi qua gRPC và chỉ 4 tính toán đồ thị đồng thời, p50 của một yêu cầu Route sẽ gần 0,1 ms, hay lớn hơn nhiều? Nếu lớn hơn, thời gian đó đi đâu?</p>
<details><summary>Xem đáp án</summary><p>Ở lần chạy chính, p50 là 2,86 ms và p99 là 4,68 ms, với 21.076 yêu cầu mỗi giây. Chênh lệch gần ba mươi lần so với 0,105 ms chủ yếu là <b>xếp hàng</b>, không phải tính toán. Theo luật Little, 64 client chia cho 21.076 req/s ra khoảng 3,04 ms thời gian trung bình trong hệ thống, gần với p50 đo được. Qua ba lần chạy thông lượng dao động từ 11.392 tới 21.076 req/s vì tải máy.</p></details>
</div>

## Hợp đồng protobuf: chất lượng câu trả lời là một trường

Hợp đồng nằm ở `proto/location/v1/routing.proto`, dùng chung cho Rust (tonic), C# (`Grpc.Net.Client`) và Go. Quy ước giống `location.proto`: toạ độ E7 (`sint32`), quãng đường mét nguyên, thời gian mili giây. Điều đáng chú ý nhất là trường `quality`: câu trả lời nói rõ nó từ đâu tới, để bên gọi quyết định tin tới mức nào.

`examples/toi-noi-location/proto/location/v1/routing.proto`

```protobuf
// Kết quả đến từ đâu, để bên gọi biết nên tin tới mức nào.
enum RoutingQuality {
  ROUTING_QUALITY_UNSPECIFIED = 0;
  // Tính trên đồ thị đường ngay cho yêu cầu này.
  ROUTING_QUALITY_ROUTED = 1;
  // Lấy từ bộ nhớ đệm theo ô H3 và khung giờ: có thể lệch vài chục mét so với tính mới.
  ROUTING_QUALITY_CACHED = 2;
  // Ước lượng chim bay × hệ số vòng vì quá hạn, quá tải hoặc không có đường.
  ROUTING_QUALITY_ESTIMATED = 3;
}

message RouteRequest {
  LatLngE7 from = 1;
  LatLngE7 to = 2;
  RoutingProfile profile = 3;
  // Giây trong ngày lúc khởi hành (0..86399) để dùng hồ sơ tốc độ theo giờ. Vắng = tốc độ tự do.
  optional uint32 depart_sec_of_day = 4;
  // Có trả hình học (polyline precision 6) hay không. Tuyến có hình học không dùng bộ nhớ đệm.
  bool want_geometry = 5;
}
```

`Route` và `Matrix` là hai RPC. Ma trận trả mảng `duration_ms` hàng-chính, `4294967295` (u32::MAX) nghĩa là không tới được. Profile là trường bắt buộc: thiếu thì dịch vụ trả `InvalidArgument` thay vì đoán, vì một câu trả lời giả cho ô tô khi khách hỏi xe máy tệ hơn một lỗi rõ ràng.

## Luồng xử lý và deadline

Hình 1 là đường đi của một yêu cầu Route. Deadline của client tới qua header `grpc-timeout`; máy chủ đọc nó, trừ 3 ms dự trữ để gửi trả lời, và dùng phần còn lại làm **ngân sách** cho mọi thứ phía sau. Không có header thì dùng mặc định 250 ms.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 1. Luồng xử lý một yêu cầu Route: kiểm tra đầu vào, tính ngân sách từ grpc-timeout, tra cache H3, xin giấy phép tính toán, tính trên engine. Trúng cache trả CACHED; hết ngân sách khi chờ giấy phép trả ESTIMATED; tính xong kịp hạn trả ROUTED; quá hạn hoặc không có đường trả ESTIMATED."><defs><marker id="loc29-f1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs><rect x="20" y="20" width="840" height="305" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Một yêu cầu Route đi qua những bước nào trong geo-engine</text><rect x="40" y="66" width="140" height="68" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="50" y="88" class="t-b">1. Đầu vào</text><text x="50" y="108" class="t-small">toạ độ, profile,</text><text x="50" y="124" class="t-small">→ InvalidArgument</text><path d="M182 100 L198 100" class="f-none s-muted" stroke-width="2" marker-end="url(#loc29-f1-a)"/><rect x="200" y="66" width="140" height="68" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="210" y="88" class="t-b">2. Ngân sách</text><text x="210" y="108" class="t-small">grpc-timeout − 3 ms</text><text x="210" y="124" class="t-small">hoặc 250 ms mặc định</text><path d="M342 100 L358 100" class="f-none s-muted" stroke-width="2" marker-end="url(#loc29-f1-a)"/><rect x="360" y="66" width="140" height="68" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="370" y="88" class="t-b">3. Cache H3</text><text x="370" y="108" class="t-small">chỉ Route,</text><text x="370" y="124" class="t-small">không kèm hình học</text><path d="M502 100 L518 100" class="f-none s-muted" stroke-width="2" marker-end="url(#loc29-f1-a)"/><rect x="520" y="66" width="140" height="68" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="530" y="88" class="t-b">4. Giấy phép</text><text x="530" y="108" class="t-small">semaphore, chờ tối</text><text x="530" y="124" class="t-small">đa bằng ngân sách</text><path d="M662 100 L678 100" class="f-none s-muted" stroke-width="2" marker-end="url(#loc29-f1-a)"/><rect x="680" y="66" width="140" height="68" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="690" y="88" class="t-b">5. Tính toán</text><text x="690" y="108" class="t-small">spawn_blocking</text><text x="690" y="124" class="t-small">engine.route</text><path d="M430 134 L430 168" class="f-none s-muted" stroke-width="2" marker-end="url(#loc29-f1-a)"/><rect x="360" y="170" width="140" height="52" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="370" y="192" class="t-b">CACHED</text><text x="370" y="210" class="t-small">trúng khoá cache</text><path d="M590 134 L590 168" class="f-none s-muted" stroke-width="2" marker-end="url(#loc29-f1-a)"/><rect x="520" y="170" width="140" height="52" rx="8" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="530" y="192" class="t-b">ESTIMATED</text><text x="530" y="210" class="t-small">chờ quá ngân sách</text><path d="M713 134 L713 168" class="f-none s-muted" stroke-width="2" marker-end="url(#loc29-f1-a)"/><rect x="680" y="170" width="66" height="52" rx="8" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="688" y="192" class="t-b">ROUTED</text><text x="688" y="210" class="t-small">kịp hạn</text><path d="M798 134 L798 168" class="f-none s-muted" stroke-width="2" marker-end="url(#loc29-f1-a)"/><rect x="756" y="170" width="84" height="52" rx="8" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="764" y="192" class="t-b">ESTIMATED</text><text x="764" y="210" class="t-small">quá hạn</text><text x="40" y="262" class="t-small">Không có đường (điểm snap nằm ngoài thành phần liên thông) cũng trả ESTIMATED, source «straight-line:no-route».</text><text x="40" y="284" class="t-small">Tác vụ tính đã bắt đầu không huỷ được khi quá hạn: nó chạy tiếp tới cùng và giữ giấy phép của nó.</text><text x="40" y="306" class="t-small">ESTIMATED = chim bay × hệ số vòng, nói rõ qua trường quality; bên gọi quyết định có tin hay không.</text></svg>
</div>
<figcaption>Hình 1. Luồng xử lý một yêu cầu Route. Mọi nhánh trong dịch vụ đều có câu trả lời, chỉ lỗi đầu vào mới trả lỗi gRPC; nhưng tầng truyền vẫn có thể cắt yêu cầu khi hết deadline, như phần dưới cho thấy.</figcaption>
</figure>

Phần cốt lõi là hàm `compute`. Chờ giấy phép tối đa bằng ngân sách (xếp hàng có hạn); hết ngân sách mà chưa có giấy phép thì coi là quá tải và bỏ yêu cầu («shed load»). Có giấy phép rồi thì chờ kết quả tối đa phần ngân sách còn lại.

`examples/toi-noi-location/rust/crates/geo-engine/src/service.rs`

```rust
        if budget.is_zero() {
            return Err(Fallback::Deadline);
        }
        let started = std::time::Instant::now();
        let permit =
            match tokio::time::timeout(budget, Arc::clone(&self.permits).acquire_owned()).await {
                Ok(Ok(p)) => p,
                _ => return Err(Fallback::Overload),
            };
        let remaining = budget.saturating_sub(started.elapsed());
        if remaining.is_zero() {
            return Err(Fallback::Deadline);
        }
        let handle = tokio::task::spawn_blocking(move || {
            let r = work();
            drop(permit);
            r
        });
        match tokio::time::timeout(remaining, handle).await {
            Ok(Ok(Ok(r))) => Ok(r),
            Ok(Ok(Err(_))) => Err(Fallback::NoRoute),
            Ok(Err(_)) => Err(Fallback::NoRoute),
            Err(_) => Err(Fallback::Deadline),
        }
```

Có một điều trung thực cần nói về đoạn này: tác vụ tính toán đã bắt đầu thì **không huỷ được** khi quá hạn. Nó chạy tiếp tới cùng và giữ giấy phép của nó. Nếu quá hạn nhiều, các giấy phép cạn và yêu cầu sau bị shed. Đó là cơ chế bảo vệ, không phải miễn phí. Lab cấu hình tối đa 4 tính toán đồng thời, máy chủ 4 luồng tokio.

### Deadline chặt: đổi chính xác lấy khả năng trả lời đúng hạn

Lab chạy Route lạnh (hầu như mọi yêu cầu trượt cache) với ba deadline:

| Deadline client | n | req/s | p50 (ms) | p99 (ms) | ROUTED | ESTIMATED | lỗi gRPC |
|---|---:|---:|---:|---:|---:|---:|---:|
| 50 ms | 1.500 | 21.805 | 2,85 | 4,00 | 99,3% | 0,6% | 0,0% |
| 10 ms | 1.500 | 24.125 | 2,54 | 3,80 | 98,7% | 0,9% | 0,0% |
| 4 ms | 1.500 | 41.392 | 1,43 | 2,80 | 40,6% | 59,3% | 0,0% |

Với 50 ms và 10 ms gần như mọi yêu cầu được tính; phần ESTIMATED nhỏ là các yêu cầu không có đường (điểm snap vào đoạn nằm ngoài thành phần liên thông mạnh của ô tô, máy chủ trả `straight-line:no-route`). Ở 4 ms, chỉ còn 1 ms ngân sách sau dự trữ 3 ms, nên 59,3% yêu cầu nhận ước lượng chim bay, p99 vẫn 2,80 ms và **0 lỗi gRPC**. Dịch vụ không báo lỗi vì chậm ở mức này mà hạ chất lượng và nói rõ qua trường `quality`.

«0 lỗi gRPC» chỉ đúng khi máy còn lập lịch kịp. Một lần chạy lại khi máy bị quá tải nặng hơn nhiều (tải 1 phút khoảng 365 trên 10 lõi, so với 24 ở bảng trên) cho kết quả khác hẳn: deadline 10 ms có 42,5% yêu cầu lỗi gRPC, phần lớn là `CANCELLED`, và deadline 4 ms có 27,1%. Cờ hiệu suy giảm không cứu được khi chính luồng phục vụ không được chạy trước hạn, nên phía client vẫn phải xử lý `CANCELLED`, không chỉ đọc `quality`.

Ma trận lớn cho thấy điều ngược lại. Với ma trận 500×500 và deadline 12 ms, **100% yêu cầu bị tầng truyền của tonic cắt** bằng mã `CANCELLED` («Timeout expired», không phải `DEADLINE_EXCEEDED` như client có thể đoán), vì dựng 250.000 ô ước lượng và mã hoá khoảng 1 MB mất hơn phần deadline còn lại. Fallback cũng phải nằm trong ngân sách. Dịch vụ giữ thêm 1 ms mỗi 100.000 ô, chưa đủ ở cỡ này. Deadline 40 ms thì ma trận 500×500 vẫn được tính xong (p50 17,29 ms).

## Cache theo ô H3 và khung giờ

Khoá cache gồm ô H3 độ phân giải 9 của điểm đi (cạnh ô cỡ 200 m, h3o báo 201 m), ô H3 của điểm đến, profile, và khung giờ 15 phút. Hai yêu cầu cùng cặp ô và cùng khung giờ dùng chung một đáp án.

`examples/toi-noi-location/rust/crates/geo-engine/src/cache.rs`

```rust
    pub fn key(
        &self,
        from: LatLng,
        to: LatLng,
        profile: u8,
        depart_sec: Option<u32>,
    ) -> Option<CacheKey> {
        let r = resolution(self.res);
        let (a, b) = (u64::from(cell_of(from, r)), u64::from(cell_of(to, r)));
        if a == b {
            self.bypass.fetch_add(1, Ordering::Relaxed);
            return None;
        }
        Some(CacheKey {
            from_cell: a,
            to_cell: b,
            profile,
            bucket: depart_sec.map_or(u16::MAX, |s| ((s % 86_400) / self.bucket_s) as u16),
        })
    }
```

Hai trường hợp không dùng cache: cặp điểm cùng một ô (sai số có thể lớn hơn chính quãng đường) và yêu cầu có hình học (cache chỉ lưu thời gian và quãng đường). Cache có 16 phân đoạn, mỗi phân đoạn một khoá riêng, đuổi theo thứ tự vào trước ra trước (FIFO), dung lượng 200.000 mục.

Đo với nhu cầu lệch: 150 điểm đi và 150 điểm đến phổ biến chọn theo luật Zipf, mỗi yêu cầu lệch tối đa 40 m.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Độ trễ p50 và p99 (ms) và thông lượng theo pha, lần chạy chính. Gọi thẳng engine 0,105 và 0,213. Route lạnh 2,86 và 4,68, 21.076 req/s. Route lệch cache ấm dần 2,53 và 7,80, 30.065 req/s. Route lệch cache ấm 0,80 và 1,53, 76.146 req/s. Deadline 4 ms 1,43 và 2,80, 41.392 req/s."><rect x="20" y="20" width="840" height="295" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Độ trễ theo pha: p50 (xanh lục) và p99 (cam), ms; máy chủ và tải cùng một máy</text><text x="40" y="92" class="t-small">Gọi thẳng engine.route (không gRPC)</text><rect x="300" y="74" width="4.0" height="14" rx="3" class="f-phys s-phys" stroke-width="1.5"/><rect x="300" y="92" width="8.2" height="14" rx="3" class="f-warn s-warn" stroke-width="1.5"/><text x="316.2" y="103" class="t-mono">0,105 / 0,213 ms</text><text x="40" y="138" class="t-small">Route lạnh, deadline 5 s</text><rect x="300" y="120" width="110.1" height="14" rx="3" class="f-phys s-phys" stroke-width="1.5"/><rect x="300" y="138" width="180.2" height="14" rx="3" class="f-warn s-warn" stroke-width="1.5"/><text x="488.2" y="149" class="t-mono">2,86 / 4,68 ms, 21.076 req/s</text><text x="40" y="184" class="t-small">Route lệch, cache ấm dần</text><rect x="300" y="166" width="97.4" height="14" rx="3" class="f-phys s-phys" stroke-width="1.5"/><rect x="300" y="184" width="300.3" height="14" rx="3" class="f-warn s-warn" stroke-width="1.5"/><text x="608.3" y="195" class="t-mono">2,53 / 7,80 ms, 30.065 req/s</text><text x="40" y="230" class="t-small">Route lệch, cache ấm</text><rect x="300" y="212" width="30.8" height="14" rx="3" class="f-phys s-phys" stroke-width="1.5"/><rect x="300" y="230" width="58.9" height="14" rx="3" class="f-warn s-warn" stroke-width="1.5"/><text x="366.9" y="241" class="t-mono">0,80 / 1,53 ms, 76.146 req/s</text><text x="40" y="276" class="t-small">Route lạnh, deadline 4 ms</text><rect x="300" y="258" width="55.1" height="14" rx="3" class="f-phys s-phys" stroke-width="1.5"/><rect x="300" y="276" width="107.8" height="14" rx="3" class="f-warn s-warn" stroke-width="1.5"/><text x="415.8" y="287" class="t-mono">1,43 / 2,80 ms, 41.392 req/s</text><text x="40" y="304" class="t-small">Hàng cuối: 59,3% câu trả lời là ESTIMATED, 0 lỗi gRPC. Máy chịu tải nặng, chỉ so sánh tương đối trong cùng một lần chạy.</text></svg>
</div>
<figcaption>Hình 2. p50 và p99 theo pha. Cache ấm cho 76.146 req/s và p99 1,53 ms, so với 21.076 req/s của đường tính mới; hạ deadline xuống 4 ms đổi 59,3% câu trả lời thành ESTIMATED. Máy chịu tải nặng.</figcaption>
</figure>

Lượt đầu với cache còn lạnh đã trúng 42,3% (1.270 trên 3.000; cache còn chứa các mục do pha Route lạnh tạo ra), p99 7,80 ms. Chạy lại đúng chuỗi đó trên cache ấm trúng 99,8%, 76.146 req/s, p50 0,80 ms, p99 1,53 ms. Theo nguồn: CACHED p50 0,26 ms, p99 1,27 ms (n = 1.270) so với ROUTED p50 3,10 ms, p99 7,95 ms (n = 1.722).

Cái giá của cache là sai số. Với 400 mẫu, đáp án CACHED so với tính mới đúng điểm lệch trung bình +0,1%, |sai số| trung vị 0,0%, p95 2,0%, nhưng **lớn nhất 37,2%**. Ô cỡ 200 m và điểm lệch tới 40 m, nên có thể có ca hai điểm cùng ô nhưng nằm hai bên một đường lớn, hoặc hai phía một con sông; lab chưa xem từng ca. Hai điều cần nhớ: đáp án trong cache là của yêu cầu đến **trước**, nên phụ thuộc thứ tự đến; và bài 28 đã nêu rằng nếu kết quả đến từ một provider bên ngoài thì việc được cache hay không phải kiểm tra theo điều khoản hiện hành của provider đó.

## Dự phòng chim bay và ma trận

Khi quá hạn, quá tải hoặc không có đường, dịch vụ trả `StraightLineProvider::estimate`: chim bay nhân hệ số vòng 1,3, chia cho tốc độ 35 km/h cho ô tô (hiệu chỉnh ở bài 28: |sai số| trung vị cỡ 9,6% so với engine). Trường `source` ghi lý do: `straight-line:deadline`, `straight-line:overload` hay `straight-line:no-route`. Với ma trận lớn, dự phòng dùng `estimate_fast` (equirectangular, rẻ hơn haversine và sai rất nhỏ trong một thành phố, xem [bài 04](/articles/location-khoang-cach-haversine-equirectangular-sai-so)) vì phải dựng hàng trăm nghìn ô.

## Ma trận qua gRPC

Số đo ở lần chạy chính, chung 4 giấy phép tính toán:

| Hình dạng | n | req/s | p50 (ms) | p99 (ms) |
|---|---:|---:|---:|---:|
| 20×20 (1 client) | 60 | 634 | 1,51 | 2,26 |
| 100×100 (1 client) | 40 | 220 | 4,51 | 5,65 |
| 100×100 (8 client) | 80 | 330 | 23,01 | 36,77 |
| 300×300 (1 client) | 20 | 79 | 11,67 | 23,17 |
| 300×300 (8 client) | 40 | 126 | 60,51 | 87,49 |
| 500×500, deadline 2.000 ms | 12 | 54 | 18,08 | 20,37 |

Với một client, 100×100 mất 4,51 ms, so với 2,40–2,81 ms của riêng ma trận ở bài 26; phần thêm là snap, mã hoá và gRPC, lab chưa đo tách. Với 8 client chung 4 giấy phép, p50 tăng lên 23,01 ms: thời gian chờ, không phải thời gian tính. Mẫu nhỏ (12–80 yêu cầu mỗi hàng) nên p99 của các hàng này chỉ là mức lớn nhất gần đó, không phải ước lượng ổn định.

## Client C# gọi được máy chủ thật

Client C# nằm ở `dotnet/GeoEngineClient`. Hợp đồng được nhập thẳng từ cùng file `.proto` của repo, không sao chép:

`examples/toi-noi-location/dotnet/GeoEngineClient/GeoEngineClient.csproj`

```xml
    <Protobuf Include="../../proto/location/v1/routing.proto" ProtoRoot="../../proto" GrpcServices="Client" Link="Protos/routing.proto" />
```

Bước 2 của client kiểm đúng hành vi cache: gọi lần đầu, rồi gọi lần hai lệch khoảng 2 m và 2 phút, hy vọng trúng cache với cùng đáp án.

`examples/toi-noi-location/dotnet/GeoEngineClient/Program.cs`

```csharp
    var first = await client.RouteAsync(new RouteRequest { From = P(a.Lat, a.Lng), To = P(b.Lat, b.Lng), Profile = RoutingProfile.Car, DepartSecOfDay = 8 * 3600 });
    var second = await client.RouteAsync(new RouteRequest { From = P(a.Lat + 0.00002, a.Lng), To = P(b.Lat, b.Lng + 0.00002), Profile = RoutingProfile.Car, DepartSecOfDay = 8 * 3600 + 120 });
    Console.WriteLine($"2. Route {a.Name} → {b.Name} lúc 08:00: lần 1 {first.DurationMs / 60000.0:F1} phút ({first.Quality}, {first.Source}); lần 2 lệch ~2 m và 2 phút: {second.DurationMs / 60000.0:F1} phút ({second.Quality}, {second.Source})");
    ok &= first.Quality == RoutingQuality.Routed && second.Quality == RoutingQuality.Cached && first.DurationMs == second.DurationMs;
```

Client này **đã chạy thật** với máy chủ `geo-engine` thật (đồ thị HCM), hai lần, thoát với kết quả «các bước 1-4 đúng như mong đợi»: Route có hình học (polyline precision 6), lần hai trúng cache với cùng thời gian, Matrix 4×4, và thiếu profile thì trả `InvalidArgument`. Nó **không nằm trong `verify-p03.sh`**, vì cần .NET SDK và NuGet (cần mạng ở lần khôi phục gói đầu), trong khi script verify cố ý không dùng mạng ngoài. Output lần 1:

```text
1. Route Chợ Bến Thành → Sân bay Tân Sơn Nhất: 12,8 phút, 7,09 km, chất lượng Routed, nguồn «ch», polyline6 776 ký tự, khứ hồi 72,0 ms
2. Route Chợ Bến Thành → Phú Mỹ Hưng lúc 08:00: lần 1 11,7 phút (Routed, ch); lần 2 lệch ~2 m và 2 phút: 11,7 phút (Cached, cache)
3. Matrix 4 × 4 (phút), chất lượng Routed, nguồn «ch-bucket»:
   Chợ Bến Thành            0,0  12,8  10,5  11,7
   Sân bay Tân Sơn Nhất    14,1   0,0  16,3  23,5
   Landmark 81             10,2  15,8   0,0  17,5
   Phú Mỹ Hưng             10,8  21,4  16,2   0,0
4. Thiếu profile → InvalidArgument: cần profile (ô tô, xe máy hoặc đi bộ)
5. Matrix 400 × 400 với deadline 30 ms: chất lượng Routed, nguồn «ch-bucket», 18,7 ms
6. 300 Route tuần tự từ C#: p50 0,29 ms, p99 0,75 ms, lớn nhất 1,05 ms (lần đầu có chi phí mở kết nối)
```

Đường chéo ma trận bằng 0,0 phút nhờ xử lý cặp cùng cạnh. Các bước 5 và 6 chỉ được in ra chứ không được kiểm trong mã thoát. Ở bước 5, ma trận 400×400 với deadline 30 ms máy chủ kịp tính ở cả hai lần chạy (18,7 và 17,4 ms), nên đường dự phòng không được thấy từ C#; nó được kiểm bằng test và bằng lab ở trên. Bước 6 là độ trễ khứ hồi nhìn từ C# trên loopback: 300 Route tuần tự p50 0,29–0,32 ms, p99 0,57–0,75 ms, **không có hàng đợi** vì chỉ một client.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin`. Lab đo tải chạy máy chủ và bộ tạo tải trong cùng một tiến trình, trên hai runtime tokio riêng, qua TCP loopback. Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc29_dich_vu_dinh_tuyen
```

Chạy máy chủ riêng và client C# (cần .NET SDK):

```bash
cargo build -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p geo-engine
examples/toi-noi-location/rust/target/release/geo-engine --addr 127.0.0.1:50071   # in ra «READY 127.0.0.1:50071»
dotnet run --project examples/toi-noi-location/dotnet/GeoEngineClient -c Release -- http://127.0.0.1:50071
```

Cấu hình đo: 64 client song song trên 4 kết nối, 3.000 yêu cầu mỗi pha, máy chủ 4 luồng tokio và tối đa 4 tính toán đồ thị đồng thời, client 2 luồng, deadline mặc định 250 ms, dự trữ 3 ms. Đã chạy 3 lần; khối dưới là lần có «Route lạnh» đạt req/s cao nhất. **Đo khi máy chịu tải nặng** (tải 1 phút khoảng 24 trên 10 lõi):

```text
  đối chứng gọi thẳng engine.route trong tiến trình (một luồng, không gRPC, 400 cặp đầu): p50 0.105 ms, p99 0.213 ms, trung bình 0.108 ms
  Route lạnh (deadline 5 s)             3000    21076     2.86     3.75     4.68     5.31
  Route lệch, chạy lại (cache ấm)       3000    76146     0.80     1.19     1.53     2.11
  sai số của đáp án CACHED so với tính mới đúng điểm (400 mẫu): trung bình +0.1%, |sai số| trung vị 0.0%, p95 2.0%, lớn nhất 37.2%
  4 ms (dự trữ 3 ms)                    1500    41392     1.43     1.99     2.80     3.24
  500×500, deadline 12 ms                 12       70        -        -        -        -
tổng cộng phía máy chủ: 13900 Route, 276 Matrix, 0 đầu vào sai; cache 8.231 mục, tỉ lệ trúng toàn phiên 33.6%
```

Các số đếm (tỉ lệ ROUTED, CACHED, ESTIMATED, sai số của cache) tái lập; req/s và độ trễ thì không: qua ba lần chạy, Route lạnh là 11.392–21.076 req/s và p99 4,68–18,93 ms. Test gRPC trong tiến trình (qua 127.0.0.1) nằm trong `verify-p03.sh`.

## Giới hạn của ví dụ

- **Máy chủ và bộ tạo tải cùng một máy đang chịu tải nặng**, tranh nhau CPU; p99 và tỉ lệ lỗi gRPC đổi hẳn theo tải (xem lần chạy lại ở tải 365). Chưa có lần đo trên máy rảnh hay hai máy riêng.
- **Không TLS, không xác thực, một bản đồ**; CH nạp từ cache nên không đo thời gian khởi động; hồ sơ giờ không bật khi đo tải (khung giờ chỉ nằm trong khoá cache).
- **Tính toán đã bắt đầu không huỷ được** khi quá hạn; quá hạn dày sẽ làm cạn giấy phép.
- **Fallback phải nằm trong ngân sách**: ma trận 500×500 với deadline 12 ms bị cắt bằng `CANCELLED`.
- **Đáp án cache phụ thuộc thứ tự đến**, lệch tới 37,2% ở ca xấu nhất đã đo. Hình học không đi qua cache.
- **FIFO, không phải LRU**: mục nóng vẫn có thể bị đuổi sớm.
- **Client C# không nằm trong `verify-p03.sh`**, cần .NET SDK và NuGet; bước 5 và 6 của nó không được kiểm trong mã thoát.
- **p99 với vài nghìn mẫu** (3.000 yêu cầu mỗi pha) là giá trị của khoảng ba mươi yêu cầu chậm nhất, dao động giữa các lần chạy; với ma trận chỉ 12–80 mẫu nó chỉ là mức lớn nhất gần đó.

## Bài tập

**Bài 1.** Dùng luật Little, ước lượng thời gian trung bình một yêu cầu Route lạnh nằm trong hệ thống ở lần chạy chính, và so với p50 đo được. Số này nói gì về chỗ nghẽn?

<details><summary>Lời giải bài 1</summary>
<p>64 client chia cho 21.076 req/s ra khoảng 0,00304 s, tức 3,04 ms, so với p50 2,86 ms. Hai số gần nhau, trong khi gọi thẳng engine mất 0,105 ms, và client C# gọi tuần tự, không có hàng đợi, thấy p50 0,29–0,32 ms cả gRPC. Vậy phần lớn thời gian của một yêu cầu là chờ giấy phép và chờ lịch của các luồng, không phải tính toán. Tăng số tính toán đồng thời hay số luồng chỉ hữu ích nếu còn lõi CPU rảnh; trên máy đang quá tải thì không.</p>
</details>

**Bài 2.** Vì sao yêu cầu có `want_geometry = true` không dùng cache, và điều đó ảnh hưởng thế nào tới thiết kế client?

<details><summary>Lời giải bài 2</summary>
<p>Cache chỉ lưu `duration_ms` và `distance_m` cho một cặp ô; hình học của hai điểm khác nhau trong cùng ô là khác nhau, nên dùng lại sẽ trả đường không đi qua điểm đón thật. Client chỉ nên xin hình học khi cần vẽ (ví dụ sau khi gán tài xế) và dùng yêu cầu không hình học cho ETA lúc báo giá, để hưởng cache.</p>
</details>

**Bài 3.** Đề xuất một cách để ma trận 500×500 với deadline 12 ms không bị tonic cắt bằng `CANCELLED`, và nêu một điều kiện cần đo để biết cách đó có đủ.

<details><summary>Lời giải bài 3</summary>
<p>Cách đề xuất, chưa được lab thử: kiểm tra cỡ ma trận và ngân sách ngay từ đầu, và nếu số ô vượt ngưỡng mà ngân sách không đủ cho dựng ước lượng thì trả lỗi rõ ràng ngay (ví dụ `DeadlineExceeded` kèm thông báo deadline quá chặt) thay vì chạy fallback quá dài; hoặc tăng khoản dự trữ theo số ô. Cần đo thời gian dựng và mã hoá một ma trận ước lượng theo số ô trên cùng máy, để biết khoản dự trữ mỗi 100.000 ô phải là bao nhiêu.</p>
</details>

Bài trước: [bài 28](/articles/location-self-host-osrm-valhalla-hay-map-provider). Bài tiếp: [bài 30](/articles/location-tong-hop-mo-phong-su-kien-roi-rac-seed).

## Tài liệu gốc

- [gRPC: Deadlines](https://grpc.io/docs/guides/deadlines/): vì sao client nên luôn đặt deadline và máy chủ nên tôn trọng nó.
- [tonic trên docs.rs](https://docs.rs/tonic): máy chủ và client gRPC cho Rust (phiên bản ghim trong lab: 0.14.9).
- [Little's law (Wikipedia)](https://en.wikipedia.org/wiki/Little%27s_law): quan hệ giữa số yêu cầu đồng thời, thông lượng và thời gian trong hệ thống.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho dữ liệu OSM.
