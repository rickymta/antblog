---
slug: location-hem-chung-cu-tttm-san-bay-cong-sanh
title: "Hẻm, chung cư, TTTM, sân bay: nơi GPS vô dụng và dữ liệu cổng gần như trống"
summary: "Bài 37: lớp cổng OSM gần như trống ở TP.HCM (0,7% khách mẫu có cổng trong 80 m), hẻm khác nhau cho xe máy và ô tô, vùng đón bắt buộc là dữ liệu BỊA, GPS kém và chuỗi lùi không âm thầm. Trên dữ liệu mô phỏng."
category: system-design
level: advanced
tags: [toi-noi, location, rust, openstreetmap, entrance, alley, gps, fallback, simulation]
language: vi
---

Hình dung ba ca đặt xe. Ca thứ nhất: khách đặt xe từ tầng hầm của một chung cư. GPS báo anh đang đứng giữa đường, lệch chỗ anh thật sự đứng cả một đoạn, và ghim anh thả trên bản đồ thì nằm ở đường lớn phía sau tòa nhà. Ca thứ hai: khách ở sâu trong hẻm, xe máy vào được tới tận cửa nhưng ô tô thì dừng ở miệng hẻm. Ca thứ ba: khách ở khu đón khách riêng của một nơi đông người, nơi xe chỉ được đón tại một vài điểm do đơn vị quản lý quy định, bất kể bản đồ cho phép dừng ở đâu. Cả ba có chung một điều: thuật toán chọn điểm đón của các bài trước, vốn dựa vào tọa độ khách và mạng đường, sẽ chọn sai hoặc không có gì để chọn.

Bài này làm ba việc. Một: đo xem **dữ liệu cổng và sảnh** của OpenStreetMap, thứ ta hy vọng sẽ cứu những ca này, có thật sự có ở TP.HCM không. Câu trả lời ngắn là gần như không. Hai: đo hẻm và GPS kém làm khách đi bộ thêm bao nhiêu. Ba: dựng **chuỗi lùi** để khi dữ liệu thiếu thì hệ thống nói rõ nó đang dùng gì thay vì lùi âm thầm. **Dữ liệu vùng đón bắt buộc trong bài này là dữ liệu bịa**, và mục riêng bên dưới nhắc lại điều đó.

Đây là bài 37 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dựng trên bộ sinh ứng viên của [bài 33](/articles/location-sinh-ung-vien-diem-don-do-thi-di-bo) và profile xe máy, ô tô, đi bộ của [bài 24](/articles/location-profile-xe-may-o-to-di-bo-trong-hem). Đồ thị được biên dịch từ OSM ở [bài 06](/articles/location-osm-pbf-do-thi-duong-bo-xe-may-o-to). Mọi số trong bài là **trên dữ liệu mô phỏng** và dữ liệu OSM trong bbox của lab (lat 10,70–10,86, lng 106,60–106,78), đồ thị HCM 126.699 nút, 319.074 cạnh, © OpenStreetMap contributors. Simulator không dự đoán thế giới thật.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn định dùng thẻ <code>entrance=*</code> của OpenStreetMap để đưa khách ra đúng cổng khi GPS trong nhà kém. Trong bbox nội thành TP.HCM của lab, bạn đoán có bao nhiêu nút <code>entrance=*</code>, và trong 150 khách mẫu có bao nhiêu khách có một cổng công khai trong vòng 80 m?</p>
<details><summary>Xem đáp án</summary><p>Có <b>178</b> nút <code>entrance=*</code> trong khoảng 350 km², tức 0,51 nút mỗi km². 130 nút trong đó chỉ ghi <code>yes</code>. Trong 150 khách mẫu của seed 20270001, chỉ <b>1 khách (0,7%)</b> có cổng công khai trong 80 m. Lớp cổng chỉ dùng được khi dữ liệu được bổ sung, nên phần quan trọng của bài là chuỗi lùi khi dữ liệu thiếu.</p></details>
</div>

## Bốn nơi, bốn cách GPS làm khổ bộ chọn điểm đón

Tọa độ không sai theo cùng một cách ở mọi nơi. Bảng này tách bốn nơi theo thứ làm hỏng và thứ lab có để xử lý.

| Nơi | Cái gì hỏng | Dữ liệu sẽ giúp | Lab có gì |
|---|---|---|---|
| Hẻm | Xe máy vào được, ô tô không; điểm dừng gần nhất xa hơn nhiều cho ô tô | Loại xe, quyền đi của từng cạnh | Profile theo loại xe, `alley_stats`, so điểm dừng gần nhất |
| Chung cư, tầng hầm | GPS lệch hàng chục mét, ghim và chỗ đứng khác nhau | Cổng, sảnh, lối ra | Lớp `entrance=*` của OSM, vùng GPS kém (dữ liệu bịa) |
| Trung tâm thương mại | Chỉ đón ở cổng đã định, hay có khu đón riêng | Điểm đón quy định của nơi đó | Vùng đón bắt buộc (dữ liệu bịa) |
| Sân bay, khu đón riêng | Điểm đón bắt buộc, có khi xa chỗ khách ra | Điểm đón quy định của đơn vị quản lý | Vùng đón bắt buộc (dữ liệu bịa) |

Một nguyên tắc cho cả bài: **bài này không nêu quy định đón khách của bất kỳ sân bay, trung tâm thương mại hay tòa nhà thật nào như một sự thật.** Quy định đón khách là của đơn vị quản lý từng nơi và thay đổi theo thời gian; muốn dùng, phải lấy từ chính nguồn đó và kiểm lại. Những gì bài này đo về «vùng đón bắt buộc» là chi phí của một quy tắc áp lên tọa độ bịa, không phải số đo về nơi nào.

## Lớp cổng từ OSM: gần như trống

OpenStreetMap có thẻ `entrance=*` đặt trên nút nằm ở tường tòa nhà, ghi cổng ra vào; giá trị thường gặp là `yes`, `main`, `secondary`. Đồ thị của lab giữ các nút này ngay khi đọc PBF (bài 06) nhưng chưa gắn vào cạnh đường. Lớp cổng dựng một R-tree trên chúng và trả cổng công khai gần nhất trong một bán kính.

`examples/toi-noi-location/rust/crates/pickup/src/hard.rs`

```rust
    pub fn nearest(&self, p: LatLng, radius_m: f64) -> Option<(LatLng, f64, String)> {
        let q = self.proj.to_xy(p);
        let mut best: Option<(f64, &Ent)> = None;
        for e in self.tree.locate_within_distance(q, radius_m * radius_m) {
            if e.private {
                continue;
            }
            let d = haversine_m(p, e.p) - if e.kind == "main" { 25.0 } else { 0.0 };
            if best.is_none_or(|(b, _)| d < b) {
                best = Some((d, e));
            }
        }
        best.map(|(_, e)| (e.p, haversine_m(p, e.p), e.kind.clone()))
    }
```

Hai quy tắc nằm trong đoạn này. Cổng có `access=private`, `no` hoặc `staff` (cờ `private`) bị bỏ qua, vì điểm hẹn công cộng không thể là cổng nhân viên. Và cổng `main` được ưu tiên: nó thắng cổng thường nếu xa hơn không quá 25 m. Con số 25 m là lựa chọn của lab, không phải chuẩn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 412" role="img" aria-label="Hình 1. Lớp cổng OSM entrance=* trong bbox lab TP.HCM: 178 nút (0,51 nút mỗi km2), trong đó 130 nút chỉ ghi yes và 39 nút main. 128 nút (71,9%) cách đường xe tối đa 30 m, 131 nút (73,6%) cách đường bộ tối đa 30 m. Trong 150 khách mẫu chỉ 1 khách (0,7%) có cổng công khai trong 80 m."><text x="36" y="28" class="t-b">178 nút entrance=* trong bbox lab, theo giá trị thẻ</text><text x="36" y="57" class="t-mono">yes</text><rect x="130" y="44" width="390" height="16" rx="2" class="f-phys-soft s-phys" stroke-width="1.2"/><text x="528" y="56" class="t-mono">130</text><text x="36" y="79" class="t-mono">main</text><rect x="130" y="66" width="117" height="16" rx="2" class="f-phys-soft s-phys" stroke-width="1.2"/><text x="255" y="78" class="t-mono">39</text><text x="36" y="101" class="t-mono">secondary</text><rect x="130" y="88" width="15" height="16" rx="2" class="f-phys-soft s-phys" stroke-width="1.2"/><text x="153" y="100" class="t-mono">5</text><text x="36" y="123" class="t-mono">door</text><rect x="130" y="110" width="3" height="16" rx="2" class="f-phys-soft s-phys" stroke-width="1.2"/><text x="141" y="122" class="t-mono">1</text><text x="36" y="145" class="t-mono">entrance</text><rect x="130" y="132" width="3" height="16" rx="2" class="f-phys-soft s-phys" stroke-width="1.2"/><text x="141" y="144" class="t-mono">1</text><text x="36" y="167" class="t-mono">garage</text><rect x="130" y="154" width="3" height="16" rx="2" class="f-phys-soft s-phys" stroke-width="1.2"/><text x="141" y="166" class="t-mono">1</text><text x="36" y="189" class="t-mono">gate</text><rect x="130" y="176" width="3" height="16" rx="2" class="f-phys-soft s-phys" stroke-width="1.2"/><text x="141" y="188" class="t-mono">1</text><text x="560" y="78" class="t-big t-warn">0,51</text><text x="560" y="100" class="t-small">nút entrance=* mỗi km², trên khoảng 350 km²</text><text x="560" y="118" class="t-small">130 nút chỉ ghi «yes»: không nói cổng nào</text><text x="560" y="136" class="t-small">3 nút access=private/no/staff: bị loại</text><text x="36" y="232" class="t-b">Cổng có dùng được cho điểm hẹn không?</text><path d="M360 244 L360 340" class="f-none s-line" stroke-width="1" stroke-linecap="round"/><path d="M760 244 L760 340" class="f-none s-line dash" stroke-width="1" stroke-linecap="round"/><text x="760" y="358" class="t-small" text-anchor="middle">100%</text><text x="360" y="358" class="t-small" text-anchor="middle">0</text><text x="36" y="264">nút cách đường xe ≤ 30 m (128/178)</text><rect x="360" y="250" width="287.6" height="20" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><text x="655.6" y="264" class="t-mono">71,9%</text><text x="36" y="294">nút cách đường bộ ≤ 30 m (131/178)</text><rect x="360" y="280" width="294.4" height="20" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><text x="662.4" y="294" class="t-mono">73,6%</text><text x="36" y="324">khách mẫu có cổng công khai ≤ 80 m (1/150)</text><rect x="360" y="310" width="2.8" height="20" rx="2" class="f-warn-soft s-warn" stroke-width="1.2"/><text x="370.8" y="324" class="t-mono">0,7%</text><text x="36" y="382" class="t-small">© OpenStreetMap contributors (ODbL). Đếm trong bbox lab lat 10,70–10,86, lng 106,60–106,78.</text><text x="36" y="398" class="t-small">Khách mẫu: 150 yêu cầu của seed 20270001, trên dữ liệu mô phỏng.</text></svg>
</div>
<figcaption>Hình 1. Lớp cổng OSM trong bbox lab: theo giá trị thẻ (trên) và độ dùng được (dưới). 178 nút, 0,51 nút mỗi km². Chỉ 1 trong 150 khách mẫu có cổng công khai trong 80 m. © OpenStreetMap contributors (ODbL); khách mẫu là dữ liệu mô phỏng.</figcaption>
</figure>

Đọc Hình 1 cho đúng. Một: mật độ 0,51 nút mỗi km² trên khoảng 350 km² là mật độ **trung bình của bbox**, không phải của từng khu: phân bố có thể không đều và nội thành đông có thể dày hơn. Lab không tách theo quận. Hai: 130 trong 178 nút chỉ ghi `yes`. Chúng chứng minh có cổng, không nói đó là cổng chính hay cổng hầm, cho người đi bộ hay cho xe. Ba: trong số này 128 nút (71,9%) cách đường xe tối đa 30 m và 131 nút (73,6%) cách đường bộ tối đa 30 m, nên phần lớn dữ liệu có sẵn dùng được về mặt hình học. Vấn đề không phải chất lượng mà là **số lượng**: khi lấy 150 khách mẫu, chỉ 1 người (0,7%) có cổng công khai trong 80 m, và cổng duy nhất đó cách khách 79 m.

Hệ quả thực tế: tuyến «đưa khách ra đúng cổng» không thể dựa vào OSM một mình ở thành phố này. Muốn có, phải bổ sung dữ liệu (đối tác tòa nhà, đóng góp của tài xế sau mỗi chuyến, hoặc bản đồ riêng), và phải coi mọi cổng là dữ liệu có độ tin cậy. Đó cũng là lý do mục sau là mục quan trọng nhất của bài: hệ thống phải chạy tốt khi cổng không có.

## Hẻm: xe máy vào được, ô tô thì không

Bài 24 cho ba profile, trong đó xe máy đi được những cạnh mà ô tô không đi được. Lab đếm trên đồ thị HCM: các cạnh hẻm hay đường nhỏ (`service=alley`, `service`, `living_street`).

| Đếm trên đồ thị HCM (126.699 nút, 319.074 cạnh) | Số cạnh |
|---|---:|
| Cạnh hẻm hoặc đường nhỏ | 111.448 |
| Ô tô vào được | 99.635 |
| Xe máy vào được | 109.550 |
| Chỉ xe máy vào được | 10.107 |
| Ô tô vào được nhưng hẹp (< 4 m) | 26 |

Nguồn: `results/p04.md`, LOC-37 mục 2. Con số 26 cạnh hẹp nhỏ bất thường so với 99.635 cạnh ô tô vào được. Đó là dấu hiệu thẻ `width` thưa trong dữ liệu (đây là suy luận; lab không đếm độ phủ của thẻ này). Nghĩa là «ô tô vào được» ở đây phản ánh quy tắc profile của lab, không phải kích thước hẻm thật.

Cạnh xe máy đi được mà ô tô không, đi bộ được, dài ít nhất 15 m có 15.256. Lab lấy mẫu 150 điểm giữa các cạnh đó (khách đứng trong hẻm mô phỏng), và hỏi: điểm dừng hợp lệ gần nhất, theo từng loại xe, cách bao xa, trong tầm đi bộ 250 m?

| Loại xe | Có ứng viên | Đi bộ tới điểm dừng gần nhất, TB / p95 (m) | Ứng viên TB | Không có |
|---|---:|---:|---:|---:|
| Ô tô | 150 | 45 / 106 | 72,7 | 0 |
| Xe máy | 150 | 3 / 22 | 103,2 | 0 |

Nguồn: `results/p04.md`, LOC-37 mục 2, 150 điểm mẫu, dữ liệu mô phỏng trên đồ thị OSM thật.

Ai trả: với **ô tô**, khách trong hẻm đi bộ trung bình 45 m (p95 106 m) tới chỗ xe dừng được; với **xe máy**, 3 m (p95 22 m). Chênh khoảng 42 m ở trung bình là chi phí của việc phải đón ô tô ở miệng hẻm hoặc trên đường lớn. Ai được lợi: tài xế ô tô không phải rẽ vào đường hẹp, nhưng đó là lợi cho tài xế và cho dòng xe, không phải cho khách. Lý do loại xe là đầu vào của bộ chọn điểm đón là vậy: cùng một khách ở cùng một chỗ, ứng viên và quãng đi bộ khác hẳn.

## Vùng đón bắt buộc: dữ liệu bịa, nói to

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Dữ liệu bịa: đọc trước khi dùng bất kỳ số nào ở mục này</span>
<p><code>fixtures/hard-places-mau.json</code> là <b>dữ liệu bịa</b>, viết riêng cho lab. Không phải OSM, không phải khu vực thật, không phải quy định của bất kỳ sân bay, trung tâm thương mại hay chung cư nào. Mỗi vùng có cờ <code>invented: true</code>. Tọa độ chỉ được đặt gần các tâm nhu cầu của kịch bản mô phỏng để có khách mô phỏng rơi vào. Các số «khách đi thêm +415 m» và «+303 m» bên dưới là hệ quả của hình học bịa; chúng <b>không</b> là số đo về sân bay hay trung tâm thương mại thật.</p>
</div>

Mỗi vùng có một loại: `MandatoryPickup` (chỉ được đón ở các điểm `bays` của vùng) hoặc `IndoorPoorGps` (GPS trong nhà kém, vị trí khách chỉ đáng tin tới `gps_sigma_m`).

`examples/toi-noi-location/rust/crates/pickup/src/hard.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoneKind {
    /// Chỉ được đón ở các `bays` (sân bay, TTTM có khu đón riêng).
    MandatoryPickup,
    /// GPS trong nhà/tầng hầm kém: vị trí khách chỉ đáng tin tới `gps_sigma_m`.
    IndoorPoorGps,
}
```

`examples/toi-noi-location/fixtures/hard-places-mau.json`

```json
    {
      "name": "Khu đón sân bay giả (bịa)",
      "kind": "mandatory_pickup",
      "center": [10.8150, 106.6650],
      "radius_m": 700.0,
      "bays": [[10.8135, 106.6642], [10.8163, 106.6671]],
      "gps_sigma_m": 0.0,
      "invented": true
    },
```

Với khách trong vùng `MandatoryPickup`, chuỗi lùi bỏ hẳn bộ sinh ứng viên thường và chỉ cho các điểm đón của vùng (đã snap lên đường xe). Trong lab này, quy tắc đó là **ràng buộc cứng**, nằm trên `J`; nó chưa được cắm vào `JPickup` (xem «Giới hạn của ví dụ»). Lab đo chi phí của nó trên khách mô phỏng rơi vào hai vùng bịa, qua 6 seed đánh giá đầu tiên:

| Vùng (bịa) | Khách trong vùng | Đi bộ tới điểm `J` chọn tự do, TB / p95 (m) | Đi bộ tới điểm đón bắt buộc gần nhất, TB / p95 / max (m) | Chi phí của quy tắc (TB) |
|---|---:|---:|---:|---:|
| Khu đón sân bay giả (bán kính 700 m, 2 điểm đón) | 19 | 137 / 1.286 | 552 / 952 / 952 | **+415 m** |
| Trung tâm thương mại giả (bán kính 350 m, 1 điểm đón) | 15 | 82 / 243 | 385 / 793 / 793 | **+303 m** |

Nguồn: `results/p04.md`, LOC-37 mục 3. Số khách rất nhỏ (19 và 15), không có khoảng tin cậy. Dữ liệu mô phỏng và vùng bịa.

Ai trả, ai được lợi, nói trong khuôn khổ giả định: khách trả bằng quãng đi bộ thêm; bên được lợi là đơn vị quản lý nơi đó (giao thông ở khu đón gọn hơn, nếu quy tắc có thật) và có thể cả tài xế (không phải tìm chỗ dừng). Điều cần thấy ở số đo: trong mô hình, ràng buộc cứng của vùng **thắng `J`**, và ngưỡng đi bộ 250 m **không** loại điểm đón bắt buộc, nên khách có thể đi tới 952 m (max). Đó là quyết định thiết kế có chủ ý của lab (một quy tắc bắt buộc là bắt buộc), nhưng nó đặt ra câu hỏi sản phẩm mà lab không trả lời: có nên đặt trần quãng đi bộ cho vùng bắt buộc, và khi vượt trần thì làm gì (gọi xe đón ở xa, hay thông báo)?

## GPS kém trong nhà: tin GPS hay lùi về ghim

Khi khách ở sâu trong tòa nhà, tọa độ GPS lệch hàng chục mét. Có hai cách dùng: tin GPS như báo, hoặc coi GPS là kém khi sai số σ vượt ngưỡng (30 m ở lab) và lùi về chỗ khác. Chuỗi lùi cho **chỗ khách đứng**:

`examples/toi-noi-location/rust/crates/pickup/src/hard.rs`

```rust
pub fn resolve_origin(
    layer: Option<&EntranceLayer>,
    gps: LatLng,
    sigma_m: f64,
    pin: LatLng,
    poor_m: f64,
) -> Origin {
    if sigma_m <= poor_m {
        return Origin {
            at: gps,
            source: OriginSource::Gps,
            warnings: Vec::new(),
        };
    }
    if let Some((p, d, kind)) = layer.and_then(|l| l.nearest(gps, sigma_m)) {
        return Origin {
            at: p,
            source: OriginSource::Entrance,
            warnings: vec![format!(
                "GPS kém (±{sigma_m:.0} m): dùng cổng «{kind}» cách {d:.0} m làm chỗ khách đứng"
            )],
        };
    }
    Origin {
        at: pin,
        source: OriginSource::Pin,
        warnings: vec![format!(
            "GPS kém (±{sigma_m:.0} m) và không có cổng trong dữ liệu: lùi về điểm ghim"
        )],
    }
}
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 320" role="img" aria-label="Hình 2. Hai chuỗi lùi khi dữ liệu thiếu. Ứng viên điểm đón: vùng bắt buộc có điểm đón snap được, rồi sinh từ đồ thị đi bộ, rồi điểm ghim đã snap, rồi không có phương án. Vị trí khách đứng: GPS đủ tốt dùng GPS, GPS kém và có cổng thì dùng cổng, GPS kém và không có cổng thì lùi về điểm ghim. Mọi bước lùi ghi nguồn và cảnh báo."><text x="36" y="38" class="t-b">Ứng viên điểm đón (resolve_candidates)</text><rect x="20" y="52" width="180" height="76" rx="8" class="f-surface s-line" stroke-width="1.5"/><text x="110" y="74" class="t-b" text-anchor="middle">1. Vùng bắt buộc</text><text x="110" y="92" class="t-small" text-anchor="middle">có điểm đón snap được</text><text x="110" y="110" class="t-small" text-anchor="middle">→ chỉ các điểm đó</text><path d="M204 90 L227.0 90.0" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><path d="M236 90 L227.0 94.5 L227.0 85.5 z" class="f-muted"/><rect x="240" y="52" width="180" height="76" rx="8" class="f-surface s-line" stroke-width="1.5"/><text x="330" y="74" class="t-b" text-anchor="middle">2. Sinh từ đồ thị</text><text x="330" y="92" class="t-small" text-anchor="middle">đi bộ như bình thường</text><text x="330" y="110" class="t-small" text-anchor="middle">(mặc định)</text><path d="M424 90 L447.0 90.0" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><path d="M456 90 L447.0 94.5 L447.0 85.5 z" class="f-muted"/><rect x="460" y="52" width="180" height="76" rx="8" class="f-surface s-line" stroke-width="1.5"/><text x="550" y="74" class="t-b" text-anchor="middle">3. Điểm ghim</text><text x="550" y="92" class="t-small" text-anchor="middle">đã snap lên đường</text><text x="550" y="110" class="t-small" text-anchor="middle">(PinSnap)</text><path d="M644 90 L667.0 90.0" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><path d="M676 90 L667.0 94.5 L667.0 85.5 z" class="f-muted"/><rect x="680" y="52" width="180" height="76" rx="8" class="f-surface s-line" stroke-width="1.5"/><text x="770" y="74" class="t-b" text-anchor="middle">4. Không phương án</text><text x="770" y="92" class="t-small" text-anchor="middle">kèm cảnh báo</text><text x="770" y="110" class="t-small" text-anchor="middle">(None)</text><text x="36" y="170" class="t-b">Vị trí khách đứng (resolve_origin)</text><rect x="20" y="184" width="180" height="76" rx="8" class="f-surface s-line" stroke-width="1.5"/><text x="110" y="206" class="t-b" text-anchor="middle">GPS đủ tốt</text><text x="110" y="224" class="t-small" text-anchor="middle">σ ≤ 30 m</text><text x="110" y="242" class="t-small" text-anchor="middle">→ dùng GPS như báo</text><path d="M204 222 L227.0 222.0" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><path d="M236 222 L227.0 226.5 L227.0 217.5 z" class="f-muted"/><rect x="240" y="184" width="180" height="76" rx="8" class="f-surface s-line" stroke-width="1.5"/><text x="330" y="206" class="t-b" text-anchor="middle">GPS kém, có cổng</text><text x="330" y="224" class="t-small" text-anchor="middle">trong vòng σ</text><text x="330" y="242" class="t-small" text-anchor="middle">→ dùng cổng</text><path d="M424 222 L447.0 222.0" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><path d="M456 222 L447.0 226.5 L447.0 217.5 z" class="f-muted"/><rect x="460" y="184" width="180" height="76" rx="8" class="f-surface s-line" stroke-width="1.5"/><text x="550" y="206" class="t-b" text-anchor="middle">GPS kém, không cổng</text><text x="550" y="224" class="t-small" text-anchor="middle">→ lùi về điểm ghim</text><text x="550" y="242" class="t-small" text-anchor="middle">kèm cảnh báo</text><rect x="680" y="184" width="180" height="76" rx="8" class="f-warn-soft s-warn dash" stroke-width="1.5"/><text x="770" y="206" class="t-b" text-anchor="middle">Không lùi âm thầm</text><text x="770" y="224" class="t-small" text-anchor="middle">mỗi bước ghi nguồn</text><text x="770" y="242" class="t-small" text-anchor="middle">và cảnh báo</text><text x="36" y="306" class="t-small">Mỗi hộp là một giá trị của enum trong crate pickup::hard. Hình minh họa luồng, không theo tỉ lệ.</text></svg>
</div>
<figcaption>Hình 2. Hai chuỗi lùi của <code>pickup::hard</code>. Mỗi bước dùng được đều được ghi vào <code>source</code> và <code>warnings</code>; không có bước nào lùi mà không để lại dấu vết. Hình minh họa luồng, không theo tỉ lệ.</figcaption>
</figure>

Lab thử với 150 yêu cầu của seed 20270001. «Hối tiếc» là `J` thật (tính ở vị trí thật của khách) của điểm được chọn trừ `J` thật nhỏ nhất; «đi bộ thật» là quãng khách đi từ chỗ đứng **thật** tới điểm đã chọn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 396" role="img" aria-label="Hình 3. Quãng khách đi bộ thật theo độ nhiễu GPS. Tin GPS như báo: 78,8; 86,2; 134,8; 210,8 mét ở σ bằng 5, 15, 40, 80 mét. Lùi về điểm ghim khi GPS kém: 78,8; 86,2; 87,8; 91,8 mét. Hối tiếc của hai cách: 15,24 và 15,24; 27,67 và 27,67; 74,71 và 37,18; 166,21 và 41,36 giây."><text x="36" y="28" class="t-b">Quãng khách đi bộ thật (m) và hối tiếc (giây) theo độ nhiễu GPS σ</text><rect x="36" y="42" width="14" height="12" rx="2" class="f-warn-soft s-warn" stroke-width="1.2"/><text x="58" y="52" class="t-small">tin GPS như báo</text><rect x="220" y="42" width="14" height="12" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><text x="242" y="52" class="t-small">lùi về cổng/ghim khi GPS kém (σ &gt; 30 m)</text><path d="M110 66 L110 350" class="f-none s-line" stroke-width="1" stroke-linecap="round"/><text x="36" y="98" class="t-b">σ = 5 m</text><rect x="110" y="74" width="118.2" height="20" rx="2" class="f-warn-soft s-warn" stroke-width="1.2"/><text x="236.2" y="88" class="t-mono">78,8 m, hối tiếc 15,24</text><rect x="110" y="100" width="118.2" height="20" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><text x="236.2" y="114" class="t-mono">78,8 m, hối tiếc 15,24</text><text x="840" y="98" class="t-mono" text-anchor="end">150/0/0</text><text x="36" y="166" class="t-b">σ = 15 m</text><rect x="110" y="142" width="129.3" height="20" rx="2" class="f-warn-soft s-warn" stroke-width="1.2"/><text x="247.3" y="156" class="t-mono">86,2 m, hối tiếc 27,67</text><rect x="110" y="168" width="129.3" height="20" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><text x="247.3" y="182" class="t-mono">86,2 m, hối tiếc 27,67</text><text x="840" y="166" class="t-mono" text-anchor="end">150/0/0</text><text x="36" y="234" class="t-b">σ = 40 m</text><rect x="110" y="210" width="202.2" height="20" rx="2" class="f-warn-soft s-warn" stroke-width="1.2"/><text x="320.2" y="224" class="t-mono">134,8 m, hối tiếc 74,71</text><rect x="110" y="236" width="131.7" height="20" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><text x="249.7" y="250" class="t-mono">87,8 m, hối tiếc 37,18</text><text x="840" y="234" class="t-mono" text-anchor="end">0/0/150</text><text x="36" y="302" class="t-b">σ = 80 m</text><rect x="110" y="278" width="316.2" height="20" rx="2" class="f-warn-soft s-warn" stroke-width="1.2"/><text x="434.2" y="292" class="t-mono">210,8 m, hối tiếc 166,21</text><rect x="110" y="304" width="137.7" height="20" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><text x="255.7" y="318" class="t-mono">91,8 m, hối tiếc 41,36</text><text x="840" y="302" class="t-mono" text-anchor="end">0/4/146</text><text x="840" y="66" class="t-small" text-anchor="end">nguồn GPS/cổng/ghim</text><text x="36" y="370" class="t-small">Một seed (20270001), 150 yêu cầu, không có khoảng tin cậy. Trên dữ liệu mô phỏng.</text><text x="36" y="386" class="t-small">Ghim của simulator lệch chỗ đứng thật σ = 12 m mỗi trục, nên lùi về ghim ở đây là lựa chọn dễ.</text></svg>
</div>
<figcaption>Hình 3. Quãng khách đi bộ thật (mét) và hối tiếc (giây) khi GPS nhiễu σ. Cam: tin GPS như báo. Xanh: lùi về cổng hoặc ghim khi σ vượt 30 m. Cột phải: nguồn vị trí được dùng (GPS/cổng/ghim). Một seed, 150 yêu cầu, không có khoảng tin cậy; trên dữ liệu mô phỏng.</figcaption>
</figure>

Đọc Hình 3: ở σ = 5 và 15 m, hai cách giống hệt nhau vì GPS được coi là đủ tốt (nguồn 150/0/0). Từ σ = 40 m, tin GPS làm khách đi bộ thật 134,8 m (so với 78,8 m ở σ = 5 m) và tới 210,8 m ở σ = 80 m, trong khi lùi về ghim giữ ở 87,8 m và 91,8 m. Hối tiếc cũng giảm: 74,71 xuống 37,18 giây ở σ = 40 m; 166,21 xuống 41,36 giây ở σ = 80 m. Lớp cổng gần như không giúp được: ở σ = 80 m chỉ 4 trong 150 ca dùng được cổng, vì dữ liệu cổng thưa.

**Một điều phải nói thẳng để không đọc quá tay.** Lùi về ghim trông tốt ở đây phần lớn vì **simulator định nghĩa ghim lệch chỗ đứng thật chỉ σ = 12 m mỗi trục** (bài 30). Trong mô hình đó ghim luôn là một ước lượng chỗ đứng tốt hơn GPS lệch 40 hay 80 m. Ngoài đời, ghim do khách thả hoặc do ứng dụng gợi ý có thể lệch hơn thế rất nhiều, nhất là trong tòa nhà lớn hay khu vực khách không rành đường. Kết luận «lùi về ghim tốt hơn tin GPS kém» vì thế phụ thuộc vào độ chính xác của ghim, một giả định của lab. Cái bài này chứng minh là cơ chế (có bước lùi, có ghi nguồn, có đo), không phải con số 87,8 m.

## Chuỗi lùi cho ứng viên: không bao giờ lùi âm thầm

Chuỗi thứ hai quyết định **danh sách ứng viên**: vùng bắt buộc, rồi sinh từ đồ thị đi bộ, rồi điểm ghim đã snap, rồi không phương án (Hình 2, hàng trên). Lab chạy năm trường hợp và in nguồn cùng cảnh báo; đây là output thật:

```text
  bình thường (không có vùng): nguồn = Generated, 42 ứng viên, cảnh báo: []
  vùng bắt buộc nhưng điểm đón không snap được: nguồn = Generated, 42 ứng viên, cảnh báo: ["điểm đón «vùng giả có điểm đón nằm ngoài bản đồ (bịa)» không snap được: không snap được điểm đón vùng lên đường của profile (cách 62853 m, ngưỡng 250 m)", "vùng «vùng giả có điểm đón nằm ngoài bản đồ (bịa)» bắt buộc điểm đón riêng nhưng không có điểm nào dùng được: lùi về ứng viên thường"]
  khách ở ngoài bản đồ (không đường nào): nguồn = None, 0 ứng viên, cảnh báo: ["không sinh được ứng viên từ đồ thị đi bộ: lùi về điểm ghim", "không snap được điểm ghim lên đường xe: không có phương án"]
  GPS ±60 m, không có lớp cổng: nguồn vị trí = Pin, cảnh báo: ["GPS kém (±60 m) và không có cổng trong dữ liệu: lùi về điểm ghim"]
  GPS ±60 m, có lớp cổng OSM (khách đầu tiên): nguồn vị trí = Pin, cảnh báo: ["GPS kém (±60 m) và không có cổng trong dữ liệu: lùi về điểm ghim"]
```

Dòng cuối rất đáng nhìn: **có** lớp cổng OSM nhưng kết quả vẫn là `Pin`, vì khách đầu tiên không có cổng trong vòng ±60 m. Đúng với số đo ở mục trên (0,7% khách có cổng trong 80 m). Cảnh báo đi theo kết quả để tầng trên quyết định: ghi log, hiển thị cho tài xế, hoặc hỏi khách xác nhận ghim.

**Khi dữ liệu thiếu thì lùi về đâu** (đề xuất thiết kế, lab chưa đo):

- Có cổng công khai trong vòng sai số: dùng cổng, ghi nguồn.
- Không có cổng, GPS kém: lùi về ghim, kèm cảnh báo, và **hiện ghim cho khách xác nhận** chứ không đón thầm lặng ở đó.
- Vùng bắt buộc nhưng điểm đón không dùng được (không snap được, hay dữ liệu hỏng): lùi về ứng viên thường kèm cảnh báo nghiêm trọng, vì đang vi phạm một quy tắc có thể có thật.
- Không có đường nào: trả «không có phương án», không đoán.

Điều lab **đã** cài là chuỗi và việc ghi cảnh báo. Việc hiện ghim cho khách xác nhận là chức năng của ứng dụng, chưa có trong lab.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` (đã dựng ở bài 06; hoặc thêm `-- --fixture` để chạy trên đồ thị tổng hợp). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run --release -q --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc37_hem_cong_sanh
```

Output thật (rút gọn, seed 20270001, 150 yêu cầu, kịch bản `hcm-baseline`):

```text
-- 1. lớp cổng/sảnh từ OSM entrance=* (có sẵn trong đồ thị, chưa gắn vào cạnh) --
  178 nút entrance=* trong bbox của đồ thị (~350 km²: 0.51 nút/km²). Theo loại: {"door": 1, "entrance": 1, "garage": 1, "gate": 1, "main": 39, "secondary": 5, "yes": 130}
  trong đó cách đường xe ≤ 30 m: 128 (71.9%); cách đường bộ ≤ 30 m: 131 (73.6%); access=private/no/staff (không dùng làm điểm hẹn): 3
  150 khách mẫu: 1 (0.7%) có cổng công khai trong 80 m (khoảng cách p50 79 m); còn lại không có dữ liệu cổng, chuỗi lùi ở mục 5 xử lý

   σ (m) |            tin GPS như báo             |       dùng cổng/ghim khi GPS kém       | nguồn vị trí (GPS/cổng/ghim)
         |  hối tiếc TB     đi bộ TB        p95 |  hối tiếc TB     đi bộ TB        p95 |
       5 |        15.24         78.8      213.0 |        15.24         78.8      213.0 |    150/0/0
      15 |        27.67         86.2      258.2 |        27.67         86.2      258.2 |    150/0/0
      40 |        74.71        134.8      348.8 |        37.18         87.8      333.8 |      0/0/150
      80 |       166.21        210.8      592.5 |        41.36         91.8      333.8 |      0/4/146
```

Phần vùng bắt buộc và hẻm đã trích ở các bảng trên. `bash examples/toi-noi-location/verify-p04.sh` chạy cả lab này trên fixture (30 yêu cầu) để kiểm đường ống, kèm test đơn vị của `pickup::hard`, nhưng không tái hiện các bảng trên đồ thị HCM.

## Giới hạn của ví dụ

- **Dữ liệu vùng đón bắt buộc và vùng GPS kém là bịa.** Tọa độ, bán kính, số điểm đón và σ trong `hard-places-mau.json` đều là giá trị đặt tay. Kết luận duy nhất rút được là về **cơ chế** (ràng buộc cứng thắng `J`, có chi phí, có chuỗi lùi), không phải số mét của nơi nào.
- **Số khách nhỏ.** 19 và 15 khách trong hai vùng, 150 yêu cầu của một seed cho phần GPS kém. Lab này **không** so sánh cặp 20 seed và không có khoảng tin cậy; các số là mô tả, không phải kiểm định.
- **Chuỗi lùi và vùng bắt buộc chưa được cắm vào chiến lược `JPickup` của simulator.** `resolve_origin`, `resolve_candidates` và `ZoneTable` chỉ chạy trong lab này; các số tổng hợp P04 ở [bài 40](/articles/location-cuoc-quote-giai-thich-doi-diem-don-cho-khach) **không** có vùng bắt buộc hay xử lý GPS kém. Điểm trả (bài 38) dùng hàm `nearest_gate` đọc thẳng nút `entrance=*` thô, không qua lớp cổng này. Cổng cũng chưa được gắn vào cạnh của đồ thị.
- **Chưa có thẻ cấm dừng thật** của OSM (như `parking:*`, biển cấm) trong đồ thị; chỉ có cờ cầu, hầm, vòng xuyến và loại đường.
- **Hẻm** đo bằng quy tắc profile của lab trên OSM, thẻ `width` thưa; không biết hẻm thật có xe ô tô vào được hay không.
- **GPS trong nhà** chỉ được mô phỏng bằng nhiễu Gauss σ; GPS thật trong tòa nhà còn có lệch hệ thống (đa đường, mất tín hiệu, nhảy tầng) mà mô hình này không có.
- **Ghim chính xác σ = 12 m** là giả định có lợi cho phương án «lùi về ghim» (xem trên).
- **Quy định đón khách của từng nơi thật** không có trong bài này, và không nên suy ra từ bài này.
- **Máy chịu tải rất nặng khi đo** (load average hơn 300). Bài không dùng số thời gian.

## Bài tập

**Bài 1.** Từ bảng hẻm, tính chênh quãng đi bộ trung bình giữa ô tô và xe máy, và nêu một chỉ số nữa mà một bài về hẻm cần báo thêm ngoài quãng đi bộ.

<details><summary>Lời giải bài 1</summary>
<p>45 − 3 = 42 m ở trung bình, và p95 là 106 − 22 = 84 m. Chỉ số nên báo thêm: thời gian tài xế mất để vào hoặc ra khỏi hẻm (km rỗng của tài xế xe máy khi vào sâu hơn), vì quãng khách đi bộ ít hơn có thể đổi lấy quãng đường tài xế dài hơn; lab chỉ đo phía khách ở mục này.</p>
</details>

**Bài 2.** Lab bỏ qua ngưỡng đi bộ 250 m cho điểm đón bắt buộc. Đề xuất hai chính sách cho trường hợp quãng đi bộ tới điểm bắt buộc vượt trần, và nói mỗi chính sách trao cái giá cho bên nào.

<details><summary>Lời giải bài 2</summary>
<p>(a) Thông báo trước khi đặt xe rằng ở vùng này khách phải đi tới điểm đón quy định: khách trả bằng thời gian và quãng đi bộ, nhưng biết trước và có thể chọn không đặt. (b) Cho phép tài xế đón ở điểm gần hơn khi quy tắc cho phép ngoại lệ (người già, hành lý): cái giá chuyển sang đơn vị quản lý hoặc tài xế bị phạt nếu vi phạm, cần quy tắc rõ từ chính đơn vị đó. Lab không đo chính sách nào; đây là hướng thiết kế.</p>
</details>

**Bài 3.** Kết luận «lùi về ghim khi GPS kém giảm đi bộ từ 210,8 xuống 91,8 m ở σ = 80 m» phụ thuộc vào giả định nào của simulator? Thí nghiệm nào sẽ cho biết kết luận bền đến đâu?

<details><summary>Lời giải bài 3</summary>
<p>Phụ thuộc vào việc ghim lệch chỗ đứng thật chỉ σ = 12 m mỗi trục. Thí nghiệm: lặp lại phép đo với nhiễu ghim tăng dần (ví dụ 12, 30, 60 m) và tìm σ của ghim mà tại đó lùi về ghim không còn tốt hơn tin GPS. Đây cũng là phép đo độ nhạy kiểu bài 40, và cần chạy trên nhiều seed có khoảng tin cậy chứ không chỉ một seed như lab này.</p>
</details>

Bài trước: [bài 36](/articles/location-diem-hen-dong-cap-nhat-khi-hai-ben-di-chuyen). Bài tiếp: [bài 38](/articles/location-toi-uu-diem-tra-cong-vao-chuyen-ke-tiep).

## Tài liệu gốc

- [Key:entrance trên OSM wiki](https://wiki.openstreetmap.org/wiki/Key:entrance): ý nghĩa thẻ `entrance=*` và các giá trị `yes`, `main`, `secondary`.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho dữ liệu OSM của đồ thị HCM.
