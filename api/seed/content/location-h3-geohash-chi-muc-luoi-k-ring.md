---
slug: location-h3-geohash-chi-muc-luoi-k-ring
title: "H3 và geohash: chia thành phố thành ô để tìm nhanh"
summary: "Bài 08: chia TP.HCM thành ô bằng geohash và H3 (h3o), đo lỗi biên ô của geohash, k-ring của H3, chọn độ phân giải theo bán kính tìm kiếm và biết khi nào lưới tốt hơn cây."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, h3, geohash, spatial-index, k-ring]
language: vi
---

Một tài xế đứng ở vĩ độ 10,722650, kinh độ 106,690000. Một khách đứng ở 10,722700, 106,690000, phía bắc anh ta **5,56 m**. Hệ thống chia thành phố thành ô geohash 6 ký tự (ô rộng 1,2 km, cao 610 m) và tìm tài xế «cùng ô với khách». Kết quả: không thấy ai. Hai người nằm ở hai ô khác nhau, vì đường biên ô nằm đúng giữa họ.

Lỗi biên ô là lý do không ai nên tìm tài xế bằng cách so sánh tiền tố geohash. Mọi lưới đều có biên ô, kể cả H3 của Uber; điều H3 cải thiện được là cách quét láng giềng đều hơn, và H3 cũng có giới hạn khi áp dụng. Đây là bài 08 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng khoảng cách của [bài 04](/articles/location-khoang-cach-haversine-equirectangular-sai-so) và là một trong hai nền tảng của [bài 09](/articles/location-tim-tai-xe-gan-nhat-bon-cach-co-so-do), nơi lưới H3 được đặt cạnh R-tree và Redis. Mọi con số trong bài là kết quả đo trên dữ liệu mô phỏng, seed 20261106.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hai điểm cách nhau 5,56 m có geohash độ dài 8 là <code>w3gugzcr</code> và <code>w3gv5b12</code>. Chúng chung bao nhiêu ký tự đầu? Nếu tìm bằng «cùng tiền tố 6 ký tự» thì hai điểm có thấy nhau không?</p>
<details><summary>Xem đáp án</summary><p>Chung <b>3 ký tự</b> (<code>w3g</code>), nên không thấy nhau ở độ dài 4 trở lên. Muốn thấy phải quét thêm 8 ô láng giềng: cả ở độ dài 5, 6 và 7, điểm B đều nằm trong 8 láng giềng của ô chứa A.</p></details>
</div>

## Geohash: đan xen bit, và đường biên không tránh được

Geohash mã hoá một điểm thành chuỗi ký tự bằng cách luân phiên chia đôi khoảng kinh độ rồi khoảng vĩ độ: bit 1 nếu điểm nằm ở nửa trên, bit 0 nếu nửa dưới, gom năm bit thành một ký tự base32. Chuỗi càng dài, ô càng nhỏ; hai điểm trong cùng một ô dài thì chung tiền tố. Đó là ưu điểm (tiền tố làm khoá, làm chỉ mục theo văn bản) và cũng là nhược điểm: hai điểm sát nhau nhưng nằm hai bên một đường chia **ở mức thô** (một trong những bit đầu của chuỗi) có thể chung rất ít ký tự.

`examples/toi-noi-location/rust/crates/geo-core/src/geohash.rs`

```rust
pub fn encode(p: LatLng, precision: usize) -> String {
    let (mut lat_lo, mut lat_hi) = (-90.0, 90.0);
    let (mut lng_lo, mut lng_hi) = (-180.0, 180.0);
    let mut out = String::with_capacity(precision);
    let (mut bits, mut nbits, mut even) = (0u8, 0u8, true);
    while out.len() < precision {
        let (val, lo, hi) = if even {
            (p.lng(), &mut lng_lo, &mut lng_hi)
        } else {
            (p.lat(), &mut lat_lo, &mut lat_hi)
        };
        let mid = (*lo + *hi) / 2.0;
        bits <<= 1;
        if val >= mid {
            bits |= 1;
            *lo = mid;
        } else {
            *hi = mid;
        }
        even = !even;
        nbits += 1;
        if nbits == 5 {
            out.push(BASE32[bits as usize] as char);
            bits = 0;
            nbits = 0;
        }
    }
    out
}
```

Kích thước ô tại Bến Thành (lat 10,7725), đo bằng haversine trên hộp bao của ô:

| Độ dài | Rộng (m) | Cao (m) |
|---:|---:|---:|
| 5 | 4.800,4 | 4.886,5 |
| 6 | 1.200,1 | 610,8 |
| 7 | 150,0 | 152,7 |
| 8 | 37,5 | 19,1 |

Lab quét hai đường trong bbox (lat = 10,78 theo kinh độ, và lng = 106,69 theo vĩ độ) với bước 0,00005° (cỡ 5,5 m), mã hoá geohash độ dài 8 cho từng cặp điểm liền kề và đếm số lần hai điểm khác ô. Có **1.456** lần, phân theo độ dài tiền tố chung: 3 ký tự 1 lần, 4 ký tự 7 lần, 5 ký tự 39 lần, 6 ký tự 201 lần, 7 ký tự 1.208 lần. Tức là có 47 lần hai điểm cách nhau chừng 5,5 m mà chung dưới 6 ký tự. Cặp tệ nhất là cặp đầu bài.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 430" role="img" aria-label="Hình 1. Hai điểm A và B cách nhau 5,56 mét nhưng nằm ở hai ô geohash 6 khác nhau, w3gugz ở phía nam và w3gv5b ở phía bắc, chỉ chung ba ký tự đầu của geohash độ dài 8."><defs><marker id="location-h3-geohash-chi-muc-luoi-k-ring-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs><rect x="20" y="20" width="400" height="390" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36.0" y="46.0" class="t-b">Hai ô geohash 6 chồng lên nhau</text><rect x="69.9631" y="62" width="300.074" height="152.703" class="f-pk-soft s-pk" stroke-width="1.5"/><rect x="69.9631" y="214.703" width="300.074" height="152.703" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="80.0" y="84.0" class="t-b">ô w3gv5b</text><text x="80.0" y="100.0" class="t-small">cao 610,8 m, rộng khoảng 1,2 km</text><text x="80.0" y="345.4" class="t-b">ô w3gugz</text><text x="80.0" y="361.4" class="t-small">ô phía nam, cùng kích thước</text><circle cx="118.2" cy="214.7" r="9" class="f-none s-pk" stroke-width="1.5"/><path d="M127.2 209.7 L230.0 164.7" class="f-none s-muted" stroke-width="1"/><text x="232.0" y="162.7" class="t-b">cặp A–B</text><text x="232.0" y="178.7" class="t-small">cách nhau 5,56 m</text><text x="36.0" y="396.0" class="t-small">Tỉ lệ 1 px = 4 m; hai điểm chồng lên nhau ở tỉ lệ này.</text><rect x="440" y="20" width="420" height="390" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="456.0" y="46.0" class="t-b">Phóng to quanh đường biên (1 m = 20 px)</text><rect x="446" y="60" width="408" height="180" class="f-pk-soft" stroke-width="0"/><rect x="446" y="240" width="408" height="105" class="f-warn-soft" stroke-width="0"/><path d="M446.0 240.0 L854.0 240.0" class="f-none s-muted" stroke-width="2"/><text x="460.0" y="80.0" class="t-b">ô w3gv5b (phía bắc)</text><text x="460.0" y="337.0" class="t-b">ô w3gugz (phía nam)</text><circle cx="600.0" cy="253.9" r="6" class="f-warn s-warn" stroke-width="1.5"/><circle cx="600.0" cy="142.7" r="6" class="f-pk s-pk" stroke-width="1.5"/><text x="616.0" y="257.9" class="t-mono">A  geohash 8: w3gugzcr</text><text x="616.0" y="146.7" class="t-mono">B  geohash 8: w3gv5b12</text><path d="M800.0 150.7 L800.0 245.9" class="f-none s-muted" stroke-width="1.5" marker-start="url(#location-h3-geohash-chi-muc-luoi-k-ring-1-a)" marker-end="url(#location-h3-geohash-chi-muc-luoi-k-ring-1-a)"/><text x="792.0" y="202.3" class="t-b" text-anchor="end">5,56 m</text><text x="460.0" y="372.0" class="t-small">Chung 3 ký tự đầu (w3g): tìm «cùng tiền tố» không thấy nhau.</text><text x="460.0" y="392.0" class="t-small">Biên ở vĩ độ 10,72265625: A cách biên 0,7 m, B cách 4,9 m.</text></svg>
</div>
<figcaption>Hình 1. Hai ô geohash 6 (w3gugz phía nam, w3gv5b phía bắc) và cặp điểm A, B cách nhau 5,56 m ở hai bên đường biên. Vẽ từ <code>loc08-geohash-bien.geojson</code>; chỉ có ô geohash và hai điểm, không có dữ liệu bản đồ.</figcaption>
</figure>

Cách chữa chuẩn là quét thêm **tám ô láng giềng**. Với cặp trên, B nằm trong 8 láng giềng của A ở độ dài 5, 6 và 7, nên quét chín ô thì thấy. Lab cũng thấy H3 res 8, 9, 10 xếp A và B vào cùng một ô, nhưng đó chỉ là may của riêng cặp này: H3 cũng có biên ô, và bảng dưới cho thấy chỉ quét một ô H3 thì bỏ sót nhiều không kém. Chín ô cũng chưa đủ khi bán kính tìm lớn hơn cạnh ô; phần bảng đo bên dưới có số.

## H3: ô lục giác và k-ring

H3 chia địa cầu thành các ô lục giác (cộng 12 ô ngũ giác ở mỗi độ phân giải, do hình học của mặt cầu) ở 16 mức độ phân giải, mỗi ô là một số 64 bit. Lab dùng `h3o`, bản H3 thuần Rust. Với lục giác, sáu ô kề có tâm cách đều nhau (xấp xỉ, vì lưới được chiếu lên mặt cầu), không có «ô chéo» như lưới vuông, nên khoảng cách theo số bước (khoảng cách lưới) gần với khoảng cách thật hơn. Gần hơn chứ không bằng: cạnh ô `e`, tâm của ô cách k bước nằm cách tâm gốc từ 1,5·k·e (theo hướng giữa hai góc của đĩa) tới 1,73·k·e (theo hướng góc đĩa), và đĩa k-ring là một hình lục giác răng cưa chứ không phải hình tròn. `k_ring(cell, k)` trả ô trung tâm và mọi ô cách tối đa k bước; số ô là 3k² + 3k + 1: 7, 19, 37, 61 cho k = 1…4.

`examples/toi-noi-location/rust/crates/geo-core/src/h3.rs`

```rust
/// Ô H3 chứa điểm `p` ở độ phân giải `res`.
pub fn cell_of(p: LatLng, res: Resolution) -> CellIndex {
    to_h3(p).to_cell(res)
}

/// Tâm ô.
pub fn cell_center(cell: CellIndex) -> LatLng {
    from_h3(h3o::LatLng::from(cell))
}

/// Các đỉnh của ô (6, hoặc 5 với ô ngũ giác).
pub fn cell_boundary(cell: CellIndex) -> Vec<LatLng> {
    cell.boundary().iter().map(|&v| from_h3(v)).collect()
}

/// k-ring: ô trung tâm và mọi ô cách tối đa `k` bước. `k = 1` cho 7 ô (6 với ô ngũ giác lân cận).
pub fn k_ring(cell: CellIndex, k: u32) -> Vec<CellIndex> {
    cell.grid_disk::<Vec<_>>(k)
}
```

Hình 2 vẽ k-ring 2 ở độ phân giải 9 quanh Bến Thành, từ chính file GeoJSON mà lab xuất ra. Với riêng điểm hỏi này, đường tròn bán kính 300 m nằm trọn trong 7 ô của vòng 0 và vòng 1. Điểm hỏi sát góc ô thì không, nên trên toàn mẫu k = 1 vẫn bỏ sót 0,20%.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 520" role="img" aria-label="Hình 2. K-ring của H3 độ phân giải 9 quanh Bến Thành: 1 ô ở vòng 0, 6 ô ở vòng 1, 12 ô ở vòng 2, tổng 19 ô; đường tròn bán kính 300 mét quanh điểm hỏi nằm trong 7 ô của vòng 0 và vòng 1."><rect x="20" y="20" width="560" height="480" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36.0" y="46.0" class="t-b">19 ô H3 độ phân giải 9 quanh điểm hỏi (10,7725; 106,698)</text><path d="M473.4 311.8 L463.2 266.9 L419.9 253.3 L386.7 284.7 L396.9 329.6 L440.2 343.2 L473.4 311.8 Z" class="f-phys-soft s-phys" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M496.4 235.6 L486.2 190.6 L442.9 177.0 L409.7 208.4 L419.9 253.3 L463.2 266.9 L496.4 235.6 Z" class="f-phys-soft s-phys" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M442.9 177.0 L432.7 132.1 L389.3 118.5 L356.2 149.9 L366.3 194.8 L409.7 208.4 L442.9 177.0 Z" class="f-phys-soft s-phys" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M389.3 118.5 L379.1 73.6 L335.8 60.0 L302.6 91.3 L312.8 136.3 L356.2 149.9 L389.3 118.5 Z" class="f-phys-soft s-phys" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M312.8 136.3 L302.6 91.3 L259.3 77.8 L226.1 109.1 L236.3 154.0 L279.6 167.6 L312.8 136.3 Z" class="f-phys-soft s-phys" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M236.3 154.0 L226.1 109.1 L182.7 95.5 L149.6 126.9 L159.8 171.8 L203.1 185.4 L236.3 154.0 Z" class="f-phys-soft s-phys" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M213.3 230.3 L203.1 185.4 L159.8 171.8 L126.6 203.2 L136.8 248.1 L180.1 261.7 L213.3 230.3 Z" class="f-phys-soft s-phys" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M190.3 306.6 L180.1 261.7 L136.8 248.1 L103.6 279.4 L113.8 324.4 L157.1 338.0 L190.3 306.6 Z" class="f-phys-soft s-phys" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M243.9 365.1 L233.7 320.2 L190.3 306.6 L157.1 338.0 L167.3 382.9 L210.7 396.5 L243.9 365.1 Z" class="f-phys-soft s-phys" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M297.4 423.7 L287.2 378.7 L243.9 365.1 L210.7 396.5 L220.9 441.4 L264.2 455.0 L297.4 423.7 Z" class="f-phys-soft s-phys" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M373.9 405.9 L363.7 361.0 L320.4 347.4 L287.2 378.7 L297.4 423.7 L340.7 437.2 L373.9 405.9 Z" class="f-phys-soft s-phys" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M450.4 388.1 L440.2 343.2 L396.9 329.6 L363.7 361.0 L373.9 405.9 L417.3 419.5 L450.4 388.1 Z" class="f-phys-soft s-phys" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M419.9 253.3 L409.7 208.4 L366.3 194.8 L333.2 226.2 L343.4 271.1 L386.7 284.7 L419.9 253.3 Z" class="f-pk-soft s-pk" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M366.3 194.8 L356.2 149.9 L312.8 136.3 L279.6 167.6 L289.8 212.6 L333.2 226.2 L366.3 194.8 Z" class="f-pk-soft s-pk" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M289.8 212.6 L279.6 167.6 L236.3 154.0 L203.1 185.4 L213.3 230.3 L256.7 243.9 L289.8 212.6 Z" class="f-pk-soft s-pk" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M266.8 288.8 L256.7 243.9 L213.3 230.3 L180.1 261.7 L190.3 306.6 L233.7 320.2 L266.8 288.8 Z" class="f-pk-soft s-pk" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M320.4 347.4 L310.2 302.4 L266.8 288.8 L233.7 320.2 L243.9 365.1 L287.2 378.7 L320.4 347.4 Z" class="f-pk-soft s-pk" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M396.9 329.6 L386.7 284.7 L343.4 271.1 L310.2 302.4 L320.4 347.4 L363.7 361.0 L396.9 329.6 Z" class="f-pk-soft s-pk" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M343.4 271.1 L333.2 226.2 L289.8 212.6 L256.7 243.9 L266.8 288.8 L310.2 302.4 L343.4 271.1 Z" class="f-warn-soft s-warn" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/><text x="382.7" y="245.7" class="t-small" text-anchor="middle">1</text><text x="335.0" y="170.0" class="t-small" text-anchor="middle">1</text><text x="252.7" y="204.9" class="t-small" text-anchor="middle">1</text><text x="229.7" y="281.2" class="t-small" text-anchor="middle">1</text><text x="283.2" y="339.7" class="t-small" text-anchor="middle">1</text><text x="359.7" y="322.0" class="t-small" text-anchor="middle">1</text><text x="436.3" y="304.2" class="t-small" text-anchor="middle">2</text><text x="459.2" y="227.9" class="t-small" text-anchor="middle">2</text><text x="405.7" y="169.4" class="t-small" text-anchor="middle">2</text><text x="352.2" y="110.9" class="t-small" text-anchor="middle">2</text><text x="275.6" y="128.6" class="t-small" text-anchor="middle">2</text><text x="199.1" y="146.4" class="t-small" text-anchor="middle">2</text><text x="176.1" y="222.7" class="t-small" text-anchor="middle">2</text><text x="153.2" y="299.0" class="t-small" text-anchor="middle">2</text><text x="206.7" y="357.5" class="t-small" text-anchor="middle">2</text><text x="260.2" y="416.0" class="t-small" text-anchor="middle">2</text><text x="336.8" y="398.2" class="t-small" text-anchor="middle">2</text><text x="413.3" y="380.5" class="t-small" text-anchor="middle">2</text><circle cx="290.4" cy="242.0" r="63.5682" class="f-none s-muted dash" stroke-width="1.5"/><circle cx="290.4" cy="242.0" r="5" class="f-warn s-warn" stroke-width="1.5"/><path d="M36.0 484.0 L57.2 484.0" class="f-none s-muted" stroke-width="2"/><text x="65.2" y="488.0" class="t-small">100 m</text><rect x="600" y="20" width="260" height="480" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="616.0" y="46.0" class="t-b">Vòng quét và số ô</text><rect x="616" y="62" width="22" height="16" rx="3" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="648.0" y="75.0">k = 0: 1 ô</text><rect x="616" y="88" width="22" height="16" rx="3" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="648.0" y="101.0">k = 1: 7 ô (thêm 6)</text><rect x="616" y="114" width="22" height="16" rx="3" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="648.0" y="127.0">k = 2: 19 ô (thêm 12)</text><circle cx="627.0" cy="152.0" r="5" class="f-warn s-warn" stroke-width="1.5"/><text x="648.0" y="156.0">điểm hỏi</text><circle cx="627.0" cy="178.0" r="9" class="f-none s-muted dash" stroke-width="1.5"/><text x="648.0" y="182.0">đường tròn r = 300 m</text><text x="616.0" y="218.0" class="t-small">Bỏ sót đích ở r = 300 m (200.000 mẫu):</text><text x="616.0" y="240.0" class="t-mono">k = 0: 46,26%</text><text x="616.0" y="260.0" class="t-mono">k = 1:  0,20%</text><text x="616.0" y="280.0" class="t-mono">k = 2:  0,00%</text><text x="616.0" y="322.0" class="t-small">Cạnh ô đo tại Bến Thành: 215,7 m,</text><text x="616.0" y="337.0" class="t-small">trung bình toàn cầu của h3o: 200,8 m.</text><text x="616.0" y="372.0" class="t-small">k-ring k có 3k² + 3k + 1 ô</text><text x="616.0" y="387.0" class="t-small">(7, 19, 37, 61 với k = 1…4).</text><text x="616.0" y="422.0" class="t-small">Ở 300 m, vòng 1 vẫn bỏ sót 0,20%</text><text x="616.0" y="437.0" class="t-small">số đích; vòng 2 không bỏ sót mẫu nào.</text></svg>
</div>
<figcaption>Hình 2. 19 ô H3 độ phân giải 9 quanh điểm (10,7725; 106,698), tô theo khoảng cách lưới 0, 1, 2. Vẽ từ <code>loc08-h3-kring.geojson</code>; chỉ có ô H3, không có dữ liệu bản đồ.</figcaption>
</figure>

Độ phân giải quyết định cạnh ô. Bảng dưới lấy từ lab: cạnh trung bình theo bảng của H3, cạnh đo trên ô thật tại Bến Thành (lớn hơn trung bình toàn cầu 7–8%, vì kích thước ô H3 thay đổi theo vị trí trên địa cầu), và số ô phủ bbox (chỉ là diện tích bbox 349,8 km² chia diện tích ô trung bình, không đếm ô thật):

| Res | Cạnh TB (m) | Cạnh đo tại HCM (m) | Diện tích TB (km²) | Số ô phủ bbox (ước tính) |
|---:|---:|---:|---:|---:|
| 7 | 1.406,5 | 1.510,1 | 5,1613 | 68 |
| 8 | 531,4 | 570,8 | 0,7373 | 474 |
| 9 | 200,8 | 215,7 | 0,1053 | 3.321 |
| 10 | 75,9 | 81,5 | 0,0150 | 23.247 |
| 11 | 28,7 | 30,8 | 0,0021 | 162.729 |

## Chọn độ phân giải theo bán kính tìm

Câu hỏi thực tế: tìm đích trong bán kính r quanh điểm hỏi, cần quét mấy vòng? Lab thử bằng cách lấy ngẫu nhiên 200.000 cặp (điểm hỏi, đích), đích cách điểm hỏi một khoảng cách phân bố đều trong [0, r] theo hướng ngẫu nhiên, rồi đếm số lần đích **không nằm** trong tập ô được quét. Với H3, «k cần thiết» của một cặp chính là khoảng cách lưới giữa hai ô:

`examples/toi-noi-location/rust/labs/src/bin/loc08_h3.rs`

```rust
                let (cq, ct) = (cell_of(q, res), cell_of(t, res));
                // khoảng cách lưới giữa hai ô: ô nhỏ nhất k mà ct nằm trong k-ring(cq)
                let need = cq.grid_distance(ct).map_or(usize::MAX, |d| d as usize);
                for (k, m) in miss.iter_mut().enumerate() {
                    if need > k {
                        *m += 1;
                    }
                }
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 422" role="img" aria-label="Hình 3. Tỉ lệ bỏ sót đích cách điểm hỏi tối đa 300 mét: geohash 6 một ô bỏ sót 22,08 phần trăm, chín ô 0; geohash 7 chín ô vẫn bỏ sót 17,86; H3 độ phân giải 9 cần k bằng 2 để về 0."><text x="20.0" y="28.0" class="t-b">Bỏ sót khi tìm đích trong bán kính 300 m (200.000 mẫu mỗi dòng, mô phỏng, seed 20261106)</text><text x="20.0" y="64.0">geohash 6 · 1 ô</text><rect x="230" y="52" width="114.816" height="15" class="f-warn" stroke-width="0"/><text x="352.8" y="64.0" class="t-mono">22,08%</text><text x="20.0" y="90.0">geohash 6 · 9 ô</text><rect x="230" y="78" width="1.5" height="15" class="f-warn" stroke-width="0"/><text x="239.5" y="90.0" class="t-mono">0,00%</text><text x="20.0" y="116.0">geohash 7 · 1 ô</text><rect x="230" y="104" width="395.564" height="15" class="f-warn" stroke-width="0"/><text x="633.6" y="116.0" class="t-mono">76,07%</text><text x="20.0" y="142.0">geohash 7 · 9 ô</text><rect x="230" y="130" width="92.872" height="15" class="f-warn" stroke-width="0"/><text x="330.9" y="142.0" class="t-mono">17,86%</text><text x="20.0" y="168.0">H3 res 8 · k=0 (1 ô)</text><rect x="230" y="156" width="97.292" height="15" class="f-pk" stroke-width="0"/><text x="335.3" y="168.0" class="t-mono">18,71%</text><text x="20.0" y="194.0">H3 res 8 · k=1 (7 ô)</text><rect x="230" y="182" width="1.5" height="15" class="f-pk" stroke-width="0"/><text x="239.5" y="194.0" class="t-mono">0,00%</text><text x="20.0" y="220.0">H3 res 9 · k=0 (1 ô)</text><rect x="230" y="208" width="240.552" height="15" class="f-pk" stroke-width="0"/><text x="478.6" y="220.0" class="t-mono">46,26%</text><text x="20.0" y="246.0">H3 res 9 · k=1 (7 ô)</text><rect x="230" y="234" width="1.5" height="15" class="f-pk" stroke-width="0"/><text x="239.5" y="246.0" class="t-mono">0,20%</text><text x="20.0" y="272.0">H3 res 9 · k=2 (19 ô)</text><rect x="230" y="260" width="1.5" height="15" class="f-pk" stroke-width="0"/><text x="239.5" y="272.0" class="t-mono">0,00%</text><text x="20.0" y="298.0">H3 res 10 · k=1 (7 ô)</text><rect x="230" y="286" width="187.408" height="15" class="f-pk" stroke-width="0"/><text x="425.4" y="298.0" class="t-mono">36,04%</text><text x="20.0" y="324.0">H3 res 10 · k=2 (19 ô)</text><rect x="230" y="312" width="15.964" height="15" class="f-pk" stroke-width="0"/><text x="254.0" y="324.0" class="t-mono">3,07%</text><text x="20.0" y="350.0">H3 res 10 · k=3 (37 ô)</text><rect x="230" y="338" width="1.5" height="15" class="f-pk" stroke-width="0"/><text x="239.5" y="350.0" class="t-mono">0,00%</text><path d="M230.0 44.0 L230.0 362.0" class="f-none s-line" stroke-width="1"/><text x="230.0" y="380.0" class="t-small" text-anchor="middle">0%</text><text x="750.0" y="380.0" class="t-small" text-anchor="middle">100%</text><rect x="560" y="370" width="14" height="10" class="f-warn" stroke-width="0"/><text x="580.0" y="380.0" class="t-small">geohash</text><rect x="650" y="370" width="14" height="10" class="f-pk" stroke-width="0"/><text x="670.0" y="380.0" class="t-small">H3</text></svg>
</div>
<figcaption>Hình 3. Tỉ lệ bỏ sót khi đích cách điểm hỏi tối đa 300 m, theo cách chia ô và số ô quét. Số liệu lấy từ kết quả chạy của <code>loc08_h3</code>.</figcaption>
</figure>

Hình 3 cho một quy tắc đơn giản: **ô cần lớn hơn nhiều so với bán kính, hoặc vòng quét phải rộng ra**. Geohash 6 (ô 1,2 km × 0,6 km) bỏ sót 7,69% đích cách 100 m, 22,08% cách 300 m, 60,88% cách 1 km nếu chỉ quét một ô; quét chín ô đưa về 0% ở 100 m và 300 m nhưng vẫn bỏ sót 3,93% ở 1 km. Geohash 7 (ô 150 m) quét chín ô vẫn bỏ sót 17,86% ở 300 m và 74,93% ở 1 km. H3 thì:

| Res | r (m) | k nhỏ nhất | Số ô | Diện tích ô quét / hình tròn |
|---:|---:|---:|---:|---:|
| 8 | 100 | 1 | 7 | 164,3 |
| 8 | 300 | 1 | 7 | 18,3 |
| 8 | 1.000 | 2 | 19 | 4,5 |
| 9 | 100 | 1 | 7 | 23,5 |
| 9 | 300 | 2 | 19 | 7,1 |
| 9 | 1.000 | 4 | 61 | 2,0 |
| 10 | 100 | 2 | 19 | 9,1 |
| 10 | 300 | 3 | 37 | 2,0 |
| 10 | 1.000 | quá 8 vòng | – | – |

(«k nhỏ nhất» là k nhỏ nhất cho 0 bỏ sót trong 200.000 mẫu, thử tới k = 8.) Cột này không phải ngẫu nhiên, và có thể tính trước. Đích nằm ở ô cách từ k + 1 bước trở lên thì tâm hai ô cách nhau ít nhất 1,5·(k + 1)·e; điểm hỏi và đích mỗi bên lệch khỏi tâm ô của mình tối đa `e`. Vậy mọi đích gần hơn **(1,5·k − 0,5)·e** chắc chắn nằm trong k-ring k. Với `e` đo tại HCM, k nhỏ nhất thoả (1,5·k − 0,5)·e ≥ r khớp đúng với mọi dòng của bảng: res 9 có k = 1 bảo đảm 216 m, k = 2 bảo đảm 539 m, nên r = 300 m cần k = 2; res 10 với r = 1 km cần k = 9, tức quá 8 vòng. Đây là ước lượng trên lưới phẳng đều, đối chiếu với mẫu đo chứ không phải chứng minh chặt trên mặt cầu.

Đọc bảng: res 9 với r = 300 m quét 19 ô, diện tích gấp 7,1 lần hình tròn, bỏ sót 0,20% nếu chỉ lấy vòng 1. Res 10 vừa khít hơn (2,0 lần) nhưng cần 37 ô, và ở 1 km quá 8 vòng. Độ phân giải tốt là độ phân giải có cạnh ô cỡ bán kính tìm kiếm thường dùng.

## Chi phí tính ô, và khi nào lưới tốt hơn cây

Tính ô cho một điểm không đắt, nhưng cũng không miễn phí. Trung vị 5 lần của 1.000.000 điểm, `--release`: ở lần chạy cuối `H3 cell_of` res 9 tốn 456,7 ns mỗi điểm, `geohash::encode` độ dài 7 tốn 281,8 ns, `k_ring(1)` 176,1 ns mỗi ô. **Các con số này đo khi máy chịu tải nặng** và dao động mạnh: qua các lần chạy `cell_of` từ 401,6 đến 1.217,9 ns, `encode` từ 281,8 đến 1.008,5 ns. Chỉ có thể kết luận rằng cả hai cùng cỡ vài trăm ns. Geohash của lab cấp phát một `String` mỗi lần mã hoá nên không nhanh hơn H3 rõ rệt dù công thức đơn giản hơn.

Khi nào lưới tốt hơn cây? `H3Buckets` của bài 09 giữ một danh sách tài xế cho mỗi ô; truy vấn mở rộng từng vòng quanh điểm hỏi tới khi đủ k người và người thứ k đã gần hơn khoảng cách an toàn của vòng tiếp theo.

`examples/toi-noi-location/rust/crates/driver-store/src/h3_buckets.rs`

```rust
    fn nearest(&self, q: LatLng, k: usize) -> Vec<Neighbor> {
        let c0 = cell_of(q, self.res);
        let edge = edge_length_avg_m(self.res);
        let mut best: Vec<Neighbor> = Vec::with_capacity(k + 1);
        for r in 0..=MAX_RING {
            for cell in Self::ring(c0, r) {
                if let Some(ids) = self.buckets.get(&cell) {
                    for &id in ids {
                        let d = equirect_m(q, self.pos[id as usize].to_latlng());
                        push_top_k(&mut best, k, Neighbor { id, dist_m: d });
                    }
                }
            }
            // Mọi điểm ở ô cách >= r+1 bước cách điểm hỏi ít nhất (1,5·r − 0,5)·cạnh; nhân 0,8 cho độ méo của lưới.
            let safe = (1.5 * f64::from(r) - 0.5).max(0.0) * edge * 0.8;
            if best.len() == k && best.last().is_some_and(|l| l.dist_m <= safe) {
                break;
            }
            if best.len() == self.count.min(k) && best.len() == self.count {
                break; // đã lấy hết tài xế
            }
        }
        best
    }
```

Khoảng cách an toàn chính là công thức vừa tính. Hệ số 0,8 là biên do lab chọn cho độ méo của lưới, không có chứng minh trong bài, nhưng kết quả truy vấn khớp quét toàn bộ 100,0% ở cả ba quy mô của bảng bài 09. Tài xế di chuyển, nên chi phí cập nhật quan trọng: đổi ô nghĩa là chuyển tài xế sang giỏ của ô mới, và chỉ khoảng 7,1–7,4% số lần cập nhật ở res 9 làm tài xế đổi ô (xe đi 0–40 km/h, báo vị trí 4 giây một lần). Phần còn lại chỉ ghi vị trí mới. Cây phải xoá rồi chèn lại mỗi lần. Đổi lại lưới cần chọn độ phân giải khớp mật độ: ở 250.000 tài xế, res 8 có p50 truy vấn 81,0 µs còn res 9 chỉ 18,0 µs (đo khi máy chịu tải nặng), vì ô res 8 chứa nhiều tài xế hơn (bài 09).

Tóm lại: lưới tốt hơn cây khi điểm di chuyển liên tục và câu hỏi là «ai ở quanh đây» (tài xế). Cây tốt hơn lưới khi phần tử là đoạn đường cố định và câu hỏi là «đoạn nào gần nhất» (bài 07). Ô còn là khoá tự nhiên để chia kho vị trí và bộ nhớ đệm theo vùng, điều mà các bài sau dùng tới.

## Chạy ví dụ và kiểm chứng

Không cần PBF, Redis hay mạng. Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc08_h3
```

Cấu hình: `--release`, Apple M1 Max, seed 20261106, 200.000 mẫu mỗi ô trong bảng, điểm hỏi trong lat 10,73–10,83 / lng 106,63–106,75. Lab ghi `results/geojson/loc08-geohash-bien.geojson` và `loc08-h3-kring.geojson`. Kết quả in ra, rút gọn:

```text
cặp tệ nhất (tiền tố chung ngắn nhất = 3 ký tự):
  A = 10.722650,106.690000  geohash w3gugzcr
  B = 10.722700,106.690000  geohash w3gv5b12
  khoảng cách A–B = 5.56 m
  độ dài 5: A = w3gug, B = w3gv5; cùng ô: false; B nằm trong 8 ô láng giềng của A: true
  H3 res 9: cùng ô: true; B trong k-ring(1) của A: true
  độ dài 6      300     22.08%      0.00%
  độ dài 7     1000     92.81%     74.93%
       9      300     46.26%      0.20%      0.00%           2       19        7.1
```

Các tỉ lệ bỏ sót và kích thước ô là số đếm và số hình học, tái lập từng chữ số với cùng seed. Chỉ phần chi phí tính ô là số đo thời gian. Test của `geo-core` có `k_ring_sizes` (kiểm tra 1, 7, 19 ô) và `boundary_has_six_vertices_in_hcm`; test của `driver-store` kiểm tra H3 res 8 và 9 khớp quét toàn bộ trên 3.000 điểm và 200 truy vấn ngẫu nhiên; tất cả chạy trong `bash examples/toi-noi-location/verify-p01.sh`.

## Giới hạn của ví dụ

- **Geohash tối giản.** `neighbors` lấy tâm ô cộng trừ kích thước rồi mã hoá lại; chỉ đúng ở vùng xa cực và xa kinh tuyến 180, đủ cho Việt Nam.
- **Độ phủ k-ring đo bằng mẫu.** «k nhỏ nhất» là thực nghiệm trên 200.000 mẫu và các bán kính 100 m, 300 m, 1 km; công thức (1,5·k − 0,5)·e chỉ là ước lượng trên lưới phẳng khớp với mẫu, và chưa thử các bán kính khác.
- **Hệ số 0,8 của `H3Buckets` là kinh nghiệm**, chưa có chứng minh; truy vấn gần ô ngũ giác dùng cách dự phòng (đĩa trừ đĩa) mà bài không kiểm riêng.
- **Mật độ đều và tải đơn giản.** Điểm hỏi rải đều trong bbox; thực tế tài xế dồn về vài điểm nóng, khiến ô res 8 chứa nhiều người hơn mức trung bình.
- **Chưa phủ vùng đa giác bằng H3.** Chưa dùng H3 để phủ một vùng (khu vực cấm đón, sân bay), chỉ dùng cho điểm.
- **Chi phí tính ô** đo khi máy tải nặng, và có phần cấp phát `String` của geohash.

## Bài tập

**Bài 1.** Vòng k = 5 của H3 có bao nhiêu ô? Quét ô trung tâm cùng 5 vòng thì cần bao nhiêu ô?

<details><summary>Lời giải bài 1</summary>
<p>«Vòng» (ring) k = 5 là các ô cách đúng 5 bước: 6·5 = 30 ô. Tổng k-ring 5 (cả ô trung tâm) là 3k² + 3k + 1 = 75 + 15 + 1 = 91 ô. Cẩn thận phân biệt <code>k_ring(cell, k)</code> (đĩa, gồm mọi ô tới k bước) với <code>grid_ring_fast</code> (chỉ vòng ngoài), điều mà <code>H3Buckets::ring</code> phân biệt.</p>
</details>

**Bài 2.** Giả sử 250.000 tài xế rải đều trong bbox 349,8 km². Mỗi ô res 8 và res 9 trung bình chứa bao nhiêu tài xế? Tìm trong bán kính 1 km, cần k = 2 ở res 8 và k = 4 ở res 9: mỗi cách phải xét tối đa khoảng bao nhiêu tài xế?

<details><summary>Lời giải bài 2</summary>
<p>Số ô phủ bbox (ước tính) là 474 ở res 8 và 3.321 ở res 9, nên trung bình khoảng 527 và 75 tài xế mỗi ô. Res 8, k = 2 là 19 ô: xấp xỉ 10.000 tài xế. Res 9, k = 4 là 61 ô: xấp xỉ 4.600. Res 9 xét ít hơn, phù hợp với số đo ở bài 09 (p50 18,0 µs so với 81,0 µs). Đây là tính số học với giả định đều; tài xế dồn về điểm nóng thì số ở ô đông còn lớn hơn.</p>
</details>

**Bài 3.** Vì sao chỉ cắt ngắn geohash (dùng độ dài 5 thay vì 8) không xoá được lỗi biên?

<details><summary>Lời giải bài 3</summary>
<p>Mọi độ dài đều có đường biên ô; cắt ngắn chỉ dời biên và làm lỗi hiếm hơn, đồng thời làm ô to hơn nên mỗi ô chứa nhiều người hơn. Ở độ dài 5 (ô khoảng 4,8 km), quét một ô vẫn bỏ sót 3,89% đích cách 300 m. Chữa tận gốc là quét cả ô láng giềng, hoặc dùng lưới có k-ring như H3 và chọn độ phân giải theo bán kính.</p>
</details>

Bài trước: [bài 07](/articles/location-rstar-r-tree-snap-diem-ve-duong). Bài tiếp: [bài 09](/articles/location-tim-tai-xe-gan-nhat-bon-cach-co-so-do).

## Tài liệu gốc

- [h3o trên docs.rs](https://docs.rs/h3o): thư viện H3 thuần Rust của lab.
- [Tài liệu H3](https://h3geo.org/): mô hình lưới lục giác, các độ phân giải và ô ngũ giác.
- [Geohash trên Wikipedia](https://en.wikipedia.org/wiki/Geohash): cách mã hoá và hạn chế ở biên ô.
