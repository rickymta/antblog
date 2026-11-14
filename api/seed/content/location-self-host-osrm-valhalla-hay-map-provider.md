---
slug: location-self-host-osrm-valhalla-hay-map-provider
title: "ADR: tự viết engine, self-host OSRM/Valhalla hay dùng map provider?"
summary: "Bài 28: ADR thứ hai của loạt. So ba hướng định tuyến theo tiêu chí, chưa chạy OSRM hay Valhalla thật, port RoutingProvider để thay adapter, dự phòng chim bay có số đo, và kế hoạch đo trước khi chọn."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, adr, routing, osrm, valhalla, architecture]
language: vi
---

Sau bài 27, ai đọc loạt này cũng sẽ hỏi cùng một câu: engine định tuyến tự viết thì hay, nhưng sao không chạy OSRM, hoặc gọi thẳng một dịch vụ bản đồ? Câu hỏi này không có đáp án chung, và một bài viết không nên giả vờ có. Tới Nơi cần ba thứ từ lớp định tuyến: tuyến và ETA cho từng chuyến, ma trận cho ghép chuyến (bài 26), và mô phỏng chạy hàng vạn lượt gọi không tốn tiền, không cần mạng. Mỗi hướng trong ba hướng dưới đây đáp ứng được vài thứ trong đó và để hở những thứ khác.

Đây là bài 28 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi), và là ADR (bản ghi quyết định kiến trúc) thứ hai của loạt, sau [ADR phân vai ngôn ngữ ở bài 02](/articles/location-adr-rust-go-python-csharp-ranh-gioi-service). Bài này **không** tuyên bố hướng nào thắng ở production. Nó ghi tiêu chí, ghi những gì lab đã đo và chưa đo, và chốt một quyết định nhỏ hơn: giữ một port để về sau đổi hướng chủ yếu là đổi adapter.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Adapter OSRM của lab đọc JSON, dựng URL, và qua bộ kiểm tra hợp đồng với 0 vi phạm trên 50 cặp. Theo bạn, điều đó có nghĩa là Tới Nơi dùng OSRM được chưa?</p>
<details><summary>Xem đáp án</summary><p>Chưa. Adapter chỉ chạy qua một <b>transport giả</b> sinh JSON đúng schema OSRM từ chính engine của lab, nên nó chỉ kiểm tra mã dựng URL và đọc JSON. Lab chưa chạy với server OSRM hay Valhalla thật: máy viết không có server nào trong hai cái đó và không có bản đồ đã biên dịch cho chúng. Lưu lượng, độ trễ, chi phí vận hành của hai hệ thống đó là các điều chưa đo.</p></details>
</div>

## Bối cảnh: Tới Nơi cần gì ở lớp định tuyến

Từ các bài trước, danh sách nhu cầu đã khá cụ thể:

- **Route và Matrix** nhanh cho lô ghép chuyến và xếp hạng điểm đón (bài 26). Hồ sơ tải tham chiếu ở bài 09 có 70 truy vấn mỗi giây.
- **Phía đường và chi phí rẽ.** Điểm đón nằm trên cạnh có hướng, quay đầu và dải phân cách đổi đáp án (bài 23, và phần P04). Lớp định tuyến phải trả lời «tài xế tới đúng phía này mất bao lâu».
- **Tốc độ theo giờ và kẹt xe thật.** Lab chỉ có kẹt xe tổng hợp (bài 27). Muốn ETA đúng ngoài đời cần nguồn kẹt xe thật, mà lab không có.
- **Hồ sơ xe máy.** Phần lớn chuyến ở Việt Nam là xe máy; hồ sơ xe máy của lab (bài 24) là giả định.
- **Mô phỏng**: hàng vạn lượt gọi cho mỗi lần chạy, tất định, không phụ thuộc mạng.
- **Cache và dự phòng.** Bài 29 đặt cache theo ô H3 và khung giờ; việc có được cache kết quả hay không phụ thuộc vào ai trả lời.

Engine của lab trả lời được vài nhu cầu đầu, nhưng không cùng lúc: CH và ma trận thì nhanh mà không có chi phí rẽ, A* theo cạnh thì có chi phí rẽ, phía đường mà chậm hơn nhiều bậc. Còn dữ liệu tốc độ, kẹt xe và hồ sơ xe máy đều là giả định. Đó là lý do quyết định này cần được ghi lại thay vì ngầm hiểu.

## Ba hướng, nói bằng điều đã biết

**A. Engine tự viết (lab này).** Có sẵn: CH, ma trận bucket, A* theo cạnh với chi phí rẽ, ba profile, dịch vụ gRPC. Kiểm soát hoàn toàn hình dạng API và có thể thêm bất cứ thứ gì P04 cần. Chưa có: dữ liệu kẹt xe thật, xử lý cấm rẽ đầy đủ (trong bbox HCM có 784 trên 1.262 quan hệ cấm rẽ có điều kiện thời gian và 112 quan hệ có `via` là way bị bỏ qua), nhiều thành phố, vận hành ở quy mô lớn, nhiều năm sửa lỗi dữ liệu bản đồ mà các dự án lâu đời đã làm.

**B. Self-host OSRM hoặc Valhalla.** Cả hai là engine định tuyến mã nguồn mở có API HTTP. OSRM có dịch vụ route và table (ma trận); trong lab, yêu cầu Valhalla được dựng thành thân JSON để gửi bằng POST. Tài liệu Route API của Valhalla liệt kê các costing `motor_scooter` và `motorcycle`; OSRM tính tốc độ theo file profile Lua nên hồ sơ xe máy là việc phải tự viết hoặc tự chọn. Bạn tự vận hành, tự chọn bản đồ OSM và cập nhật nó. Điều chưa biết với Tới Nơi: độ trễ và thông lượng trên cấu hình thật, chất lượng hồ sơ xe máy trên đường Việt Nam, khả năng trả lời đúng phía đường, và công sức vận hành. Lab chưa chạy cái nào trong hai cái này.

**C. Map provider qua API.** Dịch vụ do bên thứ ba vận hành, có thể có dữ liệu kẹt xe thực mà các hướng kia không có. Điều cần xem kỹ: điều khoản cache, giới hạn lưu kết quả, giới hạn tốc độ gọi, toạ độ khách và tài xế được gửi ra ngoài, và tính phụ thuộc vào một nhà cung cấp. Bài này cố ý không nêu giá tiền và không nêu điều khoản cụ thể của provider nào.

Điều cần nhấn: ba hướng không loại trừ nhau. Một kiến trúc hợp lý có thể dùng hướng C cho ETA hiển thị cho khách, hướng A cho mô phỏng và lô ghép, và hướng B làm đối chiếu. Port ở mục sau nhằm làm kiểu kết hợp đó rẻ hơn.

## Tiêu chí so sánh và cách đo trước khi chọn

Bảng dưới là danh sách câu hỏi cho từng tiêu chí, không phải bảng chấm điểm. Chấm điểm trước khi đo là cách chắc chắn nhất để chọn sai.

| Tiêu chí | Câu hỏi cần trả lời | Cách đo hoặc kiểm |
|---|---|---|
| Lưu lượng và độ trễ | Ở tải của Tới Nơi, p50 và p99 của Route và Matrix là bao nhiêu, có giữ được ngân sách của bài 35? | Chạy cùng một bộ truy vấn có seed qua từng hướng, ghi cấu hình máy và tải |
| Kẹt xe thật | ETA lúc cao điểm lệch bao nhiêu so với thời gian thực tế? | Cần chuyến thật có giờ; lab không có, nên chỉ đo được sau khi có dữ liệu chạy thật |
| Điều khoản cache và lưu kết quả | Được lưu kết quả bao lâu, cho mục đích nào? | Đọc điều khoản **hiện hành** của provider; bài này không nêu cụ thể |
| Cập nhật bản đồ | Đường mới, hẻm mới, cấm rẽ mới vào dữ liệu sau bao lâu và ai làm? | Hỏi nhà cung cấp, hoặc đo công sức tự cập nhật OSM |
| Phía đường, chi phí rẽ, quay đầu | Có trả lời được «tới đúng phía này» không? | Chạy bộ kiểm tra hợp đồng mở rộng với đích có hướng |
| Hồ sơ xe máy | Xe máy được xử lý riêng hay xấp xỉ bằng ô tô? | So ETA và tuyến xe máy với dữ liệu chạy thật |
| Quyền riêng tư | Toạ độ khách và tài xế có rời hệ thống của Tới Nơi không? | Xem luồng dữ liệu; với vị trí là dữ liệu cá nhân, đây có thể là ràng buộc cứng |
| Công sức vận hành | Ai cập nhật, giám sát, nâng cấp? | Ước lượng theo số người và số giờ, không phải một con số công khai |
| Tính phụ thuộc | Đổi sang hướng khác mất bao lâu? | Chính là lý do cho port bên dưới |

Cột «cách đo» có ba dòng mà lab chưa làm được: kẹt xe thật, hồ sơ xe máy với dữ liệu chạy thật, và mọi phép đo với server OSRM hay Valhalla. Dòng điều khoản cache cần một người đọc điều khoản hiện hành của đúng provider đang cân nhắc, vì điều khoản thay đổi và khác nhau giữa các nhà cung cấp. Lưu ý quan trọng với bài 29: cache theo ô H3 của dịch vụ gRPC lưu kết quả của engine của lab. Nếu sau này adapter provider cắm vào cùng đường, việc cache kết quả của provider phải được kiểm tra lại với điều khoản trước khi dùng.

## Port RoutingProvider: đường để đổi adapter

Port là một trait: bên gọi chỉ thấy trait, không biết engine nào ở sau.

`examples/toi-noi-location/rust/crates/route-engine/src/provider.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    /// Tính trên đồ thị đường.
    Routed,
    /// Ước lượng chim bay × hệ số vòng.
    Estimated,
}
```

`examples/toi-noi-location/rust/crates/route-engine/src/provider.rs`

```rust
pub trait RoutingProvider: Send + Sync {
    fn name(&self) -> &str;
    fn route(&self, q: &RouteQuery) -> Result<RouteAnswer, ProviderError>;
    fn matrix(&self, q: &MatrixQuery) -> Result<MatrixAnswer, ProviderError>;
}
```

Trường `quality` trong câu trả lời để bên gọi biết đáp án là tính thật hay ước lượng. Hình 1 vẽ các adapter có trong crate và trạng thái của từng cái.

Phải nói thẳng hiện trạng: trong lab, chỉ `loc28_routing_provider` đi qua trait này. Simulator và dịch vụ gRPC còn gọi thẳng `Engine`, vì simulator cần A* theo cạnh với đích có hướng, mà `RouteQuery` chưa có chỗ cho đích có hướng. Muốn đổi hướng thật thì phải mở rộng trait và chuyển hai bên gọi đó sang trait trước; phần này chưa làm.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 1. Port RoutingProvider: bên gọi chỉ biết trait (hiện simulator và dịch vụ gRPC còn gọi thẳng Engine); sau trait có bốn adapter. own-engine và straight-line đã chạy thật; osrm-http mới chạy qua transport giả; Valhalla mới chỉ dựng thân yêu cầu JSON và không gửi đi đâu. FallbackProvider ghép hai adapter thành chuỗi suy giảm."><defs><marker id="loc28-f1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs><rect x="20" y="20" width="840" height="305" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Port RoutingProvider và trạng thái từng adapter</text><rect x="40" y="110" width="200" height="90" rx="8" class="f-surface s-muted" stroke-width="1.5"/><text x="56" y="136" class="t-b">Bên gọi (dự kiến)</text><text x="56" y="158" class="t-small">simulator, dịch vụ gRPC,</text><text x="56" y="176" class="t-small">chọn điểm đón</text><path d="M240 155 L292 155" class="f-none s-muted" stroke-width="2" marker-end="url(#loc28-f1-a)"/><rect x="300" y="110" width="210" height="90" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="316" y="136" class="t-b">trait RoutingProvider</text><text x="316" y="158" class="t-mono">name()</text><text x="316" y="176" class="t-mono">route(q), matrix(q)</text><text x="300" y="228" class="t-small">FallbackProvider(primary, secondary)</text><text x="300" y="244" class="t-small">cũng là một RoutingProvider</text><path d="M510 155 L572 86" class="f-none s-muted" stroke-width="1.5" marker-end="url(#loc28-f1-a)"/><rect x="580" y="62" width="260" height="48" rx="8" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="596" y="82" class="t-b">own-engine</text><text x="596" y="100" class="t-small">chạy thật: CH của lab</text><path d="M510 155 L572 146" class="f-none s-muted" stroke-width="1.5" marker-end="url(#loc28-f1-a)"/><rect x="580" y="122" width="260" height="48" rx="8" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="596" y="142" class="t-b">straight-line</text><text x="596" y="160" class="t-small">chạy thật: chim bay × hệ số vòng</text><path d="M510 155 L572 206" class="f-none s-muted" stroke-width="1.5" marker-end="url(#loc28-f1-a)"/><rect x="580" y="182" width="260" height="48" rx="8" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="596" y="202" class="t-b">osrm-http</text><text x="596" y="220" class="t-small">chỉ chạy qua transport giả</text><path d="M510 155 L572 266" class="f-none s-muted" stroke-width="1.5" marker-end="url(#loc28-f1-a)"/><rect x="580" y="242" width="260" height="48" rx="8" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="596" y="262" class="t-b">Valhalla</text><text x="596" y="280" class="t-small">chỉ dựng thân JSON, không gửi đi</text><rect x="40" y="288" width="22" height="12" rx="3" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="70" y="298" class="t-small">đã chạy thật trên máy viết</text><rect x="300" y="288" width="22" height="12" rx="3" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="330" y="298" class="t-small">chưa chạy với server OSRM hoặc Valhalla thật</text></svg>
</div>
<figcaption>Hình 1. Port RoutingProvider và bốn adapter. Chỉ hai adapter đã chạy thật; adapter OSRM chỉ chạy qua transport giả, phần Valhalla mới chỉ dựng thân yêu cầu. Bên gọi ở đây là dự kiến: hiện simulator và dịch vụ gRPC còn gọi thẳng <code>Engine</code>.</figcaption>
</figure>

Adapter OSRM dựng URL theo hình dạng của OSRM: toạ độ theo thứ tự `lng,lat`, các điểm cách nhau bằng `;`, ma trận qua `table` với `sources` và `destinations`.

`examples/toi-noi-location/rust/crates/route-engine/src/provider.rs`

```rust
    pub fn table_url(&self, q: &MatrixQuery) -> String {
        let mut all = q.sources.clone();
        all.extend(q.targets.iter().copied());
        let src: Vec<String> = (0..q.sources.len()).map(|i| i.to_string()).collect();
        let dst: Vec<String> = (q.sources.len()..all.len())
            .map(|i| i.to_string())
            .collect();
        format!(
            "{}/table/v1/{}/{}?sources={}&destinations={}&annotations=duration",
            self.base_url,
            Self::profile_name(q.profile),
            Self::coords(&all),
            src.join(";"),
            dst.join(";")
        )
    }
```

Còn Valhalla thì lab chỉ dựng thân yêu cầu để thấy hình dạng khác nhau; hàm này không gửi đi đâu cả:

`examples/toi-noi-location/rust/crates/route-engine/src/provider.rs`

```rust
/// Valhalla dùng POST JSON thay vì URL: chỉ dựng thân yêu cầu để thấy hình dạng khác nhau (không gửi đi đâu cả).
pub fn valhalla_route_request(q: &RouteQuery) -> Value {
    let costing = match q.profile {
        Profile::Car => "auto",
        Profile::Motorbike => "motor_scooter",
        Profile::Foot => "pedestrian",
    };
    json!({
        "locations": [
            {"lat": q.from.lat(), "lon": q.from.lng()},
            {"lat": q.to.lat(), "lon": q.to.lng()},
        ],
        "costing": costing,
        "units": "kilometers",
    })
}
```

Hai chỗ khác nhau về hình dạng hiện ra ngay khi viết hai adapter: OSRM dùng URL với `lng,lat`, Valhalla dùng POST JSON với `lat` và `lon`; và adapter OSRM ánh xạ xe máy sang `car`, ghi chú là xấp xỉ (theo chú thích trong code, vì OSRM không có profile xe máy mặc định; cần kiểm lại với bản OSRM và profile bạn dùng). Những khác biệt kiểu này là lý do cần một lớp adapter chứ không phải gọi trực tiếp.

Bộ kiểm tra hợp đồng `contract_violations` là thứ giữ port trung thực: nó chạy cho mọi adapter, kiểm thời gian dương, quãng đường không ngắn hơn chim bay quá dung sai snap 60 m, hình học bắt đầu gần điểm đi, và ma trận 1×1 khớp `route` trong 5%. Dung sai snap có lý do: điểm đầu vào được snap lên đường nên tuyến có thể ngắn hơn chim bay giữa hai điểm thô.

## Đã chạy gì, chưa chạy gì

Lab lấy 300 cặp điểm GPS có seed (cách nút 5–30 m nên phải snap), hồ sơ ô tô, và chạy:

- **Hợp đồng cho từng adapter**: `own-engine`, `osrm-http` (qua transport giả) và `straight-line` đều có **0 vi phạm trên 50 cặp**.
- **Adapter OSRM qua transport giả so với engine của lab** (298 cặp): chênh thời gian trung vị 0,006 s, lớn nhất 4,120 s; chênh quãng đường trung vị 0,027 m, lớn nhất 19,738 m; điểm đầu hình học lệch tối đa 0,76 m. Không trùng tuyệt đối vì toạ độ đi qua URL bị làm tròn 6 chữ số thập phân (khoảng 0,1 m) và hình học qua polyline precision 5. Transport giả được gọi 400 lần.

Đọc đúng ý nghĩa của hai dòng trên: **transport giả sinh JSON từ chính engine của lab**, nên việc khớp gần tuyệt đối chỉ chứng minh mã dựng URL và đọc JSON hoạt động; nó không nói gì về OSRM thật. Không có con số nào ở đây nói về độ trễ, thông lượng, độ chính xác ETA, hay chi phí vận hành của OSRM hay Valhalla.

## Dự phòng chim bay: số đo cho đường suy giảm

Dù chọn hướng nào, một nhà cung cấp có lúc chậm hoặc hỏng. `FallbackProvider` thử adapter chính, lỗi thì dùng adapter phụ; adapter phụ rẻ nhất là chim bay nhân hệ số vòng. Lab đo hệ số này trên 298 cặp có chim bay từ 300 m trở lên:

- hệ số vòng quãng đường ô tô (đường chia chim bay): trung bình 1,367, trung vị 1,296, p5 1,155, p95 1,727;
- hiệu chỉnh trên 149 cặp đầu cho hệ số vòng trung vị 1,307 và tốc độ trên đường trung vị 35,3 km/h; sau đó đo trên 149 cặp sau.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 310" role="img" aria-label="Hình 2. Sai số của ước lượng chim bay nhân hệ số vòng so với engine của lab trên 149 cặp sau. Đoán tay hệ số vòng 1,4 và 22 km/h: sai trung vị 73,5%, p95 114,7%. Hiệu chỉnh trên nửa đầu mẫu: 9,2% và 33,3%. Mặc định của StraightLineProvider: 9,6% và 33,0%."><rect x="20" y="20" width="840" height="275" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Sai số tuyệt đối của dự phòng chim bay so với engine của lab (149 cặp)</text><text x="40" y="78" class="t-b">Đoán tay: hệ số vòng 1,4; 22 km/h</text><text x="40" y="101" class="t-small">trung vị</text><rect x="110" y="90" width="307.6" height="14" rx="3" class="f-warn s-warn" stroke-width="1.5"/><text x="425.6" y="101" class="t-mono">73,5%</text><text x="40" y="121" class="t-small">p95</text><rect x="110" y="110" width="480.0" height="14" rx="3" class="f-warn s-warn" stroke-width="1.5"/><text x="598.0" y="121" class="t-mono">114,7%</text><text x="40" y="150" class="t-b">Hiệu chỉnh trên nửa đầu mẫu: hệ số vòng 1,307; 35,3 km/h</text><text x="40" y="173" class="t-small">trung vị</text><rect x="110" y="162" width="38.5" height="14" rx="3" class="f-phys s-phys" stroke-width="1.5"/><text x="156.5" y="173" class="t-mono">9,2%</text><text x="40" y="193" class="t-small">p95</text><rect x="110" y="182" width="139.4" height="14" rx="3" class="f-phys s-phys" stroke-width="1.5"/><text x="257.4" y="193" class="t-mono">33,3%</text><text x="40" y="222" class="t-b">Mặc định StraightLineProvider: 1,3; 35 km/h</text><text x="40" y="245" class="t-small">trung vị</text><rect x="110" y="234" width="40.2" height="14" rx="3" class="f-phys s-phys" stroke-width="1.5"/><text x="158.2" y="245" class="t-mono">9,6%</text><text x="40" y="265" class="t-small">p95</text><rect x="110" y="254" width="138.1" height="14" rx="3" class="f-phys s-phys" stroke-width="1.5"/><text x="256.1" y="265" class="t-mono">33,0%</text><text x="40" y="284" class="t-small">Engine cũng là mô hình với tốc độ giả định, nên đây là độ lệch giữa hai mô hình, không phải so với đường thật.</text></svg>
</div>
<figcaption>Hình 2. Sai số của dự phòng chim bay so với engine của lab trên 149 cặp sau. Đoán tay (1,4; 22 km/h) lệch trung bình +71,4%; hiệu chỉnh theo mẫu thì |sai số| trung vị cỡ 9–10%.</figcaption>
</figure>

Đoán tay «hệ số vòng 1,4 và 22 km/h» lệch trung bình **+71,4%**, vì 22 km/h thấp hơn nhiều tốc độ chạy trên đường của engine. Hiệu chỉnh trên nửa đầu mẫu rồi đo ngoài mẫu: sai số trung bình −0,3%, |sai số| trung vị 9,2%, p95 33,3%. Mặc định của `StraightLineProvider` (1,3 và 35 km/h, chọn từ cùng mẫu này nên không hoàn toàn ngoài mẫu) cho |sai số| trung vị 9,6%, p95 33,0%.

Hai lưu ý. Một: engine của lab cũng chạy trên tốc độ giả định, nên đây là độ lệch giữa **hai mô hình**, không phải so với đường thật; nếu đổi sang provider, phải hiệu chỉnh lại dự phòng theo provider đó. Hai: bài học thực tế là bộ kiểm tra hợp đồng của dự phòng nên gồm cả một bước hiệu chỉnh, vì con số đoán tay sai tới 71%.

Lab cũng thử chuỗi: adapter chính giả lập «down» lỗi thì trả lời đến từ `straight-line` với chất lượng `Estimated` (2.040 s, 19.833 m cho cặp ví dụ); adapter chính `own-engine` sống thì trả lời `Routed` (1.964 s).

## Quyết định (bản ghi ADR)

- **Trạng thái**: đề xuất, chưa kết luận hướng cho production.
- **Bối cảnh**: xem các mục trên.
- **Quyết định**:
  1. Hướng đi: mã cần tuyến hoặc ma trận (simulator, dịch vụ gRPC, chọn điểm đón) nên chỉ phụ thuộc trait `RoutingProvider`. Hiện chưa đạt: simulator và dịch vụ gRPC còn gọi thẳng `Engine`, và trait chưa có đích có hướng.
  2. Engine của lab là hướng dùng để học, để mô phỏng, để chạy test và để làm nền cho phần cần phía đường và chi phí rẽ. Lab **không tuyên bố** nó thay được OSRM hay Valhalla.
  3. Trước khi chọn hướng cho production, chạy cùng một bộ truy vấn có seed và cùng bộ kiểm tra hợp đồng qua cả ba hướng, ghi cấu hình máy, tải và điều kiện đo; đọc điều khoản cache và lưu trữ hiện hành của provider; ghi lại ai cập nhật bản đồ.
  4. Thêm một adapter mới nghĩa là thêm mã adapter và chạy `contract_violations`, không sửa bên gọi.
- **Hệ quả**: có một lớp trừu tượng để bảo trì; adapter phải ánh xạ khác biệt giữa các hệ thống (thứ tự toạ độ, hồ sơ xe máy, đơn vị) và bộ kiểm tra hợp đồng phải đủ chặt để bắt các khác biệt đó.
- **Khi nào xem lại**: khi có dữ liệu chuyến thật để đo ETA, khi lưu lượng vượt ngân sách đã đo, hoặc khi điều khoản của provider đang cân nhắc thay đổi.

ADR này cố ý không nói «hướng nào tốt hơn», vì câu trả lời phụ thuộc vào dữ liệu mà lab chưa có: kẹt xe thật, chuyến thật, điều khoản hiện hành và công sức vận hành của đội.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` (hoặc `--fixture`). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc28_routing_provider
```

Output (rút gọn). Mọi số trong lab này không phụ thuộc thời gian, chạy một lần:

```text
CHƯA CHẠY với OSRM hay Valhalla thật (máy viết không có server OSRM/Valhalla và không có bản đồ đã biên dịch cho chúng).
  own-engine     cặp vi phạm hợp đồng: 0/50
  osrm-http      cặp vi phạm hợp đồng: 0/50
  straight-line  cặp vi phạm hợp đồng: 0/50
  298 cặp: chênh lệch thời gian trung vị 0.006 s, lớn nhất 4.120 s; chênh quãng đường trung vị 0.027 m, lớn nhất 19.738 m; điểm đầu hình học lệch tối đa 0.76 m
  298 cặp có chim bay ≥ 300 m. Hệ số vòng quãng đường (đường ô tô / chim bay): trung bình 1.367, p5 1.155, trung vị 1.296, p95 1.727
  đoán tay (hệ số vòng 1,4; 22 km/h)             trên 149 cặp sau: sai số trung bình +71.4%, |sai số| trung vị 73.5%, p95 114.7%
  primary «down» lỗi → trả lời bởi «straight-line», chất lượng Estimated, 2040 s, 19833 m
```

Mọi số ở đây là số đếm hoặc sai số nên tái lập từng chữ số với cùng seed.

## Giới hạn của ví dụ

- **Chưa chạy với OSRM hay Valhalla thật.** Adapter OSRM chỉ qua `MockOsrmTransport`; Valhalla chỉ dựng thân yêu cầu. Không có số đo lưu lượng, độ trễ hay chi phí vận hành của hai hệ thống này.
- **Chưa có adapter cho map provider nào.** Hướng C chưa được viết hay thử; bài không nêu giá hay điều khoản của provider.
- **Số hiệu chỉnh dự phòng là giữa hai mô hình**, không phải so với đường thật; 1,3 và 35 km/h chọn từ chính mẫu 300 cặp nên không hoàn toàn ngoài mẫu.
- **Port chưa được dùng bởi simulator hay dịch vụ gRPC.** Chúng gọi thẳng `Engine`; chỉ lab của bài này đi qua trait.
- **Ma trận qua adapter chỉ được kiểm bằng ô 1×1** trong bộ kiểm tra hợp đồng; chưa có kiểm tra ma trận lớn qua adapter bên ngoài.
- **Không có đo thời gian** trong lab này; các lab khác của P03 đo khi máy chịu tải nặng.

## Bài tập

**Bài 1.** Lập kế hoạch đo cho hướng B (self-host OSRM): ghi ba chỉ số bạn đo, và ba điều kiện đo phải ghi kèm.

<details><summary>Lời giải bài 1</summary>
<p>Chỉ số: p50 và p99 của Route; p50 và p99 của Matrix ở các cỡ 50×50, 100×100, 300×300; thông lượng tối đa trước khi p99 vượt ngân sách. Điều kiện: cấu hình máy chủ (CPU, RAM) và phiên bản OSRM cùng profile dùng; kích thước và ngày của bản đồ OSM đã biên dịch; tải của máy trong lúc đo và việc client với server có chung máy hay không. Bộ truy vấn phải có seed để chạy lại được, và chạy qua cùng bộ kiểm tra hợp đồng như hai adapter đã có.</p>
</details>

**Bài 2.** Vì sao bộ kiểm tra hợp đồng dùng dung sai snap 60 m cho quãng đường tối thiểu, thay vì đòi quãng đường không ngắn hơn chim bay?

<details><summary>Lời giải bài 2</summary>
<p>Điểm đầu vào được snap lên đường trước khi tính tuyến, nên tuyến có thể ngắn hơn chim bay giữa hai điểm thô khoảng bằng tổng hai khoảng snap. Đòi nghiêm ngặt sẽ báo vi phạm giả. Dung sai 60 m cho phép hai hướng snap mà vẫn bắt được adapter trả quãng đường ngắn phi lý.</p>
</details>

**Bài 3.** Dự phòng đoán tay (1,4; 22 km/h) lệch +71,4%. Nếu bạn chỉ thấy số này lúc hệ thống đã chạy, triệu chứng sẽ là gì với khách đặt xe, và cách phát hiện sớm hơn?

<details><summary>Lời giải bài 3</summary>
<p>ETA hiển thị lớn hơn đáp án của engine khoảng bảy phần mười mỗi khi dự phòng được dùng, nên khách thấy thời gian chờ dài bất thường đúng lúc dịch vụ chính có sự cố. Phát hiện sớm: đưa bước so sánh dự phòng với engine chính trên mẫu cặp cố định vào bộ kiểm tra hợp đồng, báo lỗi khi |sai số| trung vị vượt một ngưỡng đã định, và hiệu chỉnh lại mỗi khi đổi bản đồ hoặc đổi adapter chính.</p>
</details>

Bài trước: [bài 27](/articles/location-toc-do-theo-gio-customizable-ch-ket-xe). Bài tiếp: [bài 29](/articles/location-routing-service-tonic-cache-fallback-p99).

## Tài liệu gốc

- [Tài liệu API của OSRM](https://project-osrm.org/docs/v5.24.0/api/): dịch vụ Route và Table. Phiên bản và hồ sơ xe mà bạn dùng phải kiểm lại trong tài liệu hiện hành.
- [Route API của Valhalla](https://valhalla.github.io/valhalla/api/route/api-reference/): thân yêu cầu JSON, costing.
- Điều khoản sử dụng, cache và lưu trữ của map provider: đọc trực tiếp bản **hiện hành** của provider đang cân nhắc; bài này cố ý không trích.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho dữ liệu OSM.
