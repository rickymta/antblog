---
slug: location-toa-do-wgs84-newtype-lat-lng-do-chinh-xac
title: "Toạ độ: WGS84, newtype LatLng và lỗi đảo lat/lng"
summary: "Bài 03: WGS84 và VN-2000, newtype Lat/Lng chặn đảo thứ tự lúc biên dịch, GeoJSON ngược thứ tự, mỗi chữ số thập phân là bao nhiêu mét, f64 so với i32 E7 và vì sao float ổn với toạ độ mà không ổn với tiền."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, wgs84, geojson, newtype, e7, precision]
language: vi
---

Một điểm đón của chuyến TN-0001 nằm gần chợ Bến Thành: vĩ độ 10,7725, kinh độ 106,698 (toạ độ xấp xỉ để minh hoạ, không phải số đo khảo sát). Ở app, điểm này là cặp `(lat, lng)`. Khi xuất ra GeoJSON để xem trên bản đồ, cùng điểm đó phải là `[lng, lat]`: kinh độ đứng trước. Chỉ cần một dòng code quên đổi chỗ, ghim sẽ rơi sang một nơi khác, và nếu may thì nó rơi ra ngoài khoảng giá trị hợp lệ để hệ thống kêu lên.

May mắn đó chỉ có ở một số vùng. Ở Việt Nam kinh độ khoảng 102–110, lớn hơn 90, nên một «vĩ độ» 106,698 bị bắt ngay. Ở Abuja (vĩ độ 9,07, kinh độ 7,40) đảo thứ tự cho ra một điểm vẫn hợp lệ ở cả hai chiều, và không ai bắt được nó. Bài này dựng kiểu `LatLng` để việc bắt lỗi không còn dựa vào may mắn của vùng địa lý, rồi đo xem một chữ số thập phân là bao nhiêu mét, và vì sao lưu toạ độ bằng số nguyên `i32` là hợp lý. Đây là bài 03 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi), đọc sau bài [ADR về ranh giới các ngôn ngữ](/articles/location-adr-rust-go-python-csharp-ranh-gioi-service). Bài dùng crate `geo-core` của lab và giả định bạn đã biết struct, trait và `Result` của Rust.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Có hàm <code>LatLng::try_new(lat, lng)</code> kiểm tra vĩ độ trong [−90, 90] và kinh độ trong [−180, 180]. Gọi <code>try_new(7.40, 9.07)</code>, tức toạ độ Abuja (9,07; 7,40) bị đảo thứ tự, thì hàm trả lỗi hay trả điểm?</p>
<details><summary>Xem đáp án</summary><p><b>Trả điểm</b>, không lỗi: 7,40 là vĩ độ hợp lệ và 9,07 là kinh độ hợp lệ. Kiểm tra khoảng giá trị chỉ bắt được đảo thứ tự ở vùng mà kinh độ vượt 90, như Việt Nam. Thứ chặn lỗi một cách tổng quát là hai kiểu khác nhau <code>Lat</code> và <code>Lng</code>, nên <code>LatLng::new(Lng, Lat)</code> không biên dịch được.</p></details>
</div>

## WGS84, và VN-2000 thì sao

GPS trong điện thoại trả về vị trí theo **WGS84**: một ellipsoid chuẩn của Trái Đất, kèm quy ước vĩ độ là góc từ xích đạo (từ −90° ở cực nam tới +90° ở cực bắc, dương về phía bắc) và kinh độ là góc từ kinh tuyến gốc (từ −180° tới +180°, dương về phía đông). OpenStreetMap cũng lưu toạ độ theo hệ này, nên dữ liệu bản đồ của lab và GPS của tài xế nói cùng một thứ tiếng. Mã định danh quen thuộc của nó là EPSG:4326. Khi cần hiển thị lên bản đồ web, thư viện hiển thị thường chiếu tiếp sang Web Mercator; phần đó nằm ở lớp hiển thị và bài này không đụng tới.

**VN-2000** là hệ toạ độ quốc gia của Việt Nam. Dữ liệu đo đạc hoặc địa chính của cơ quan nhà nước có thể đến ở hệ này. Để đặt chúng lên cùng bản đồ với GPS phải biến đổi datum và lưới chiếu, việc cần một thư viện như PROJ. Lab **chưa chạy phép biến đổi WGS84 sang VN-2000**, nên bài này không có số nào về độ lệch giữa hai hệ và không khẳng định hai hệ lệch nhau bao nhiêu mét. Điều bài rút ra chỉ là một quy tắc thiết kế: mọi toạ độ đi vào hệ thống phải ghi rõ nó thuộc hệ nào, và biến đổi ở **biên**, đúng một chỗ, không để lẫn bên trong.

Phạm vi lab dùng một khung hình chữ nhật quanh Việt Nam: vĩ độ 8,0 đến 23,4 và kinh độ 102,1 đến 109,5, cùng bbox nội thành TP.HCM (vĩ độ 10,70–10,86, kinh độ 106,60–106,78).

## Newtype: để trình biên dịch giữ thứ tự thay con người

Cách ngây thơ nhất là truyền hai `f64`: `fn dist(lat1: f64, lng1: f64, lat2: f64, lng2: f64)`. Bốn số cùng kiểu, nên đảo hai số bất kỳ vẫn biên dịch. Newtype bọc mỗi số vào một kiểu riêng, miễn phí lúc chạy:

`examples/toi-noi-location/rust/crates/geo-core/src/coord.rs`

```rust
/// Vĩ độ, độ thập phân, trong [-90, 90].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Lat(f64);

/// Kinh độ, độ thập phân, trong [-180, 180].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Lng(f64);

impl Lat {
    pub fn new(deg: f64) -> Result<Self, GeoError> {
        if !deg.is_finite() {
            return Err(GeoError::NotFinite);
        }
        if !(-90.0..=90.0).contains(&deg) {
            return Err(GeoError::LatOutOfRange(deg));
        }
        Ok(Self(deg))
    }
    pub fn deg(self) -> f64 {
        self.0
    }
}
```

Trường bên trong là riêng tư nên bên ngoài module không thể viết `Lat(500.0)`; muốn có một `Lat` phải đi qua `Lat::new` và bị kiểm tra: số hữu hạn, nằm trong [−90, 90]. `Lng` làm tương tự với [−180, 180]. Hai kiểu này gộp thành `LatLng`, và hàm tạo nhận chúng theo đúng thứ tự:

`examples/toi-noi-location/rust/crates/geo-core/src/coord.rs`

```rust
impl LatLng {
    /// Không thể truyền nhầm thứ tự: tham số đầu là `Lat`, tham số sau là `Lng`.
    ///
    /// ```
    /// use geo_core::{Lat, LatLng, Lng};
    /// let p = LatLng::new(Lat::new(10.7725).unwrap(), Lng::new(106.6980).unwrap());
    /// assert_eq!(p.lat(), 10.7725);
    /// ```
    ///
    /// Đảo thứ tự thì không biên dịch được:
    ///
    /// ```compile_fail
    /// use geo_core::{Lat, LatLng, Lng};
    /// let _ = LatLng::new(Lng::new(106.6980).unwrap(), Lat::new(10.7725).unwrap());
    /// ```
    pub fn new(lat: Lat, lng: Lng) -> Self {
        Self { lat, lng }
    }

    /// Từ số thô, có kiểm tra khoảng giá trị. Ở Việt Nam (lat 8–24, lng 102–110) việc đảo thứ tự
    /// luôn bị bắt vì kinh độ ~106 vượt quá 90.
    pub fn try_new(lat_deg: f64, lng_deg: f64) -> Result<Self, GeoError> {
        Ok(Self::new(Lat::new(lat_deg)?, Lng::new(lng_deg)?))
    }
```

Doctest `compile_fail` **khẳng định rằng đoạn code đảo thứ tự không biên dịch được**, và nếu một ngày ai đó đổi API làm đoạn đó biên dịch, test sẽ đỏ. `try_new` là cửa vào cho số thô, ví dụ vừa parse từ JSON hay từ file OSM; nó kiểm khoảng giá trị và là chỗ duy nhất dữ liệu bẩn bị chặn.

Hai lớp bảo vệ này khác nhau về bản chất, và Hình 1 cho thấy tại sao cần cả hai.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 262" role="img" aria-label="Hình 1. Khoảng giá trị hợp lệ của vĩ độ là từ trừ 90 đến 90, của kinh độ là từ trừ 180 đến 180. Toạ độ TP.HCM là vĩ độ 10,7725 và kinh độ 106,698: nếu đảo, giá trị vĩ độ thành 106,698 vượt 90 nên bị bắt. Toạ độ Abuja là vĩ độ 9,07 và kinh độ 7,40: nếu đảo, cả hai giá trị vẫn nằm trong cả hai khoảng nên không bị bắt."><rect x="242" y="38" width="396" height="176" class="f-pk-soft s-pk" stroke-width="1"/><text x="440" y="56" class="t-b" text-anchor="middle">khoảng hợp lệ của vĩ độ: −90° … +90°</text><path d="M44 214 L44 222" class="f-none s-muted" stroke-width="1"/><text x="44" y="238" class="t-small" text-anchor="middle">−180°</text><path d="M242 214 L242 222" class="f-none s-muted" stroke-width="1"/><text x="242" y="238" class="t-small" text-anchor="middle">−90°</text><path d="M440 214 L440 222" class="f-none s-muted" stroke-width="1"/><text x="440" y="238" class="t-small" text-anchor="middle">0°</text><path d="M638 214 L638 222" class="f-none s-muted" stroke-width="1"/><text x="638" y="238" class="t-small" text-anchor="middle">90°</text><path d="M836 214 L836 222" class="f-none s-muted" stroke-width="1"/><text x="836" y="238" class="t-small" text-anchor="middle">180°</text><path d="M44 214 L836 214" class="f-none s-muted" stroke-width="1"/><text x="834" y="254" class="t-small" text-anchor="end">kinh độ hợp lệ: −180° … +180° (toàn trục)</text><text x="52" y="96" class="t-b">TP.HCM</text><circle cx="463.7" cy="90" r="5" class="f-pk s-pk" stroke-width="1"/><circle cx="674.7" cy="106" r="5" class="f-phys s-phys" stroke-width="1"/><text x="473.7" y="94" class="t-small">vĩ độ 10,7725</text><text x="684.7" y="110" class="t-small">kinh độ 106,698</text><text x="252" y="134" class="t-small">đảo thứ tự: «vĩ độ» thành 106,698, lớn hơn 90 nên try_new bắt ngay</text><text x="52" y="176" class="t-b">Abuja</text><circle cx="460" cy="170" r="5" class="f-pk s-pk" stroke-width="1"/><circle cx="456.3" cy="186" r="5" class="f-phys s-phys" stroke-width="1"/><text x="470" y="174" class="t-small">vĩ độ 9,07</text><text x="446.3" y="190" class="t-small" text-anchor="end">kinh độ 7,40</text><text x="252" y="206" class="t-small">đảo thứ tự: (7,40; 9,07) vẫn hợp lệ ở cả hai khoảng, không ai bắt</text></svg>
</div>
<figcaption>Hình 1. Đảo thứ tự lat/lng ở Việt Nam bị bắt vì kinh độ (khoảng 102–110) lớn hơn 90. Ở nơi khác thì không: Abuja nằm trong khoảng hợp lệ cả hai chiều. Số lấy từ <code>loc03_toa_do</code>; điểm xanh dương là vĩ độ, xanh lục là kinh độ, vẽ đúng vị trí trên trục.</figcaption>
</figure>

Lớp khoảng giá trị (`try_new`) hữu ích nhưng phụ thuộc địa lý: nó bắt được đảo thứ tự ở TP.HCM vì 106,698 vượt 90, còn ở Abuja thì không. Lớp kiểu (`Lat` khác `Lng`) không phụ thuộc gì: nó chặn lỗi ở mọi nơi trên Trái Đất, ngay lúc biên dịch, trước khi có dữ liệu nào chảy qua. Output của lab minh hoạ đúng điều đó:

```text
try_new(106.6980, 10.7725)     = Err(vĩ độ 106.698 ngoài khoảng [-90, 90] (có thể đã đảo lat/lng))
Abuja (lat 9.07, lng 7.40) đảo thành try_new(7.40, 9.07) = LatLng(7.4000000, 9.0700000): khoảng giá trị không bắt được, chỉ kiểu mới chặn
```

Quy tắc dùng: số thô chỉ tồn tại ở biên (parse, đọc file, nhận request), đi qua `try_new` một lần, và từ đó trở vào trong mọi hàm chỉ truyền `LatLng`.

## GeoJSON ngược thứ tự, và một chỗ đổi duy nhất

Đặc tả GeoJSON (RFC 7946) viết toạ độ theo thứ tự kinh độ rồi vĩ độ, ngược với thói quen `(lat, lng)` của GPS và của hầu hết app. Hai quy ước trái nhau là nguồn lỗi kinh điển. Cách đối phó của lab là chỉ có **một** nơi trong codebase đổi thứ tự:

`examples/toi-noi-location/rust/crates/geo-core/src/coord.rs`

```rust
    /// Dạng GeoJSON `[lng, lat]`. Đây là chỗ duy nhất của codebase đổi thứ tự sang lng trước.
    pub fn to_lnglat(self) -> [f64; 2] {
        [self.lng.0, self.lat.0]
    }

    /// Ngược lại của [`LatLng::to_lnglat`]: nhận `[lng, lat]` của GeoJSON.
    pub fn from_lnglat(pos: [f64; 2]) -> Result<Self, GeoError> {
        Self::try_new(pos[1], pos[0])
    }
```

Mọi xuất GeoJSON đi qua `to_lnglat` và mọi nhập đi qua `from_lnglat`. Đây cũng là cách để kiểm: tìm trong mã nguồn mọi chỗ tự viết `[x, y]` mà không qua hàm này. Lab xác nhận vòng đi về: `from_lnglat(to_lnglat(p)) == p` cho kết quả `true`, và GeoJSON của điểm Bến Thành ra đúng `"coordinates":[106.698,10.7725]`.

## Mỗi chữ số thập phân là bao nhiêu mét

Toạ độ lưu bằng độ thập phân, và bao nhiêu chữ số sau dấu phẩy là đủ? Lab đo bằng haversine, với bước một đơn vị ở chữ số thứ d (số lấy từ `results/p01.md`, tính trên mặt cầu bán kính trung bình, nên khác ellipsoid thật một chút; chênh lệch đó được đo ở bài kế tiếp):

| Chữ số thứ d | Theo vĩ độ (m) | Theo kinh độ, TP.HCM (m) | Theo kinh độ, Hà Nội (m) |
|---:|---:|---:|---:|
| 3 | 111,1951 | 109,2355 | 103,7897 |
| 4 | 11,1195 | 10,9235 | 10,3790 |
| 5 | 1,1120 | 1,0924 | 1,0379 |
| 6 | 0,1112 | 0,1092 | 0,1038 |
| 7 | 0,0111 | 0,0109 | 0,0104 |

Hình 2 vẽ cả tám chữ số theo vĩ độ trên thang logarit.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 372" role="img" aria-label="Hình 2. Mỗi chữ số thập phân của độ ứng với bao nhiêu mét theo vĩ độ, trên thang logarit. Chữ số thứ nhất là 11.119,5080 mét, thứ hai 1.111,9508, thứ ba 111,1951, thứ tư 11,1195, thứ năm 1,1120, thứ sáu 0,1112, thứ bảy 0,0111 và thứ tám 0,0011 mét."><text x="20" y="24" class="t-small">chữ số</text><text x="130" y="24" class="t-small">mét theo vĩ độ</text><text x="20" y="58" class="t-b">thứ 1</text><text x="250" y="58" class="t-mono" text-anchor="end">11.119,5080 m</text><rect x="290" y="44" width="563.7" height="20" class="f-warn s-warn"/><text x="20" y="90" class="t-b">thứ 2</text><text x="250" y="90" class="t-mono" text-anchor="end">1.111,9508 m</text><rect x="290" y="76" width="483.7" height="20" class="f-warn s-warn"/><text x="20" y="122" class="t-b">thứ 3</text><text x="250" y="122" class="t-mono" text-anchor="end">111,1951 m</text><rect x="290" y="108" width="403.7" height="20" class="f-warn s-warn"/><text x="20" y="154" class="t-b">thứ 4</text><text x="250" y="154" class="t-mono" text-anchor="end">11,1195 m</text><rect x="290" y="140" width="323.7" height="20" class="f-warn s-warn"/><text x="20" y="186" class="t-b">thứ 5</text><text x="250" y="186" class="t-mono" text-anchor="end">1,1120 m</text><rect x="290" y="172" width="243.7" height="20" class="f-phys s-phys"/><text x="541.7" y="186" class="t-small">polyline precision 5: sai tối đa 0,7577 m sau vòng mã hoá</text><text x="20" y="218" class="t-b">thứ 6</text><text x="250" y="218" class="t-mono" text-anchor="end">0,1112 m</text><rect x="290" y="204" width="163.7" height="20" class="f-phys s-phys"/><text x="461.7" y="218" class="t-small">polyline precision 6: sai tối đa 0,0749 m</text><text x="20" y="250" class="t-b">thứ 7</text><text x="250" y="250" class="t-mono" text-anchor="end">0,0111 m</text><rect x="290" y="236" width="83.6" height="20" class="f-dead s-dead"/><text x="381.6" y="250" class="t-small">i32 E7: độ × 10⁷, lệch tối đa 7,8 mm trên 1 triệu điểm</text><text x="20" y="282" class="t-b">thứ 8</text><text x="250" y="282" class="t-mono" text-anchor="end">0,0011 m</text><rect x="290" y="268" width="3.3" height="20" class="f-dead s-dead"/><path d="M290 304 L850 304" class="f-none s-muted" stroke-width="1"/><path d="M290 304 L290 310" class="f-none s-muted" stroke-width="1"/><text x="290" y="326" class="t-small" text-anchor="middle">1 mm</text><path d="M370 304 L370 310" class="f-none s-muted" stroke-width="1"/><text x="370" y="326" class="t-small" text-anchor="middle">1 cm</text><path d="M450 304 L450 310" class="f-none s-muted" stroke-width="1"/><text x="450" y="326" class="t-small" text-anchor="middle">10 cm</text><path d="M530 304 L530 310" class="f-none s-muted" stroke-width="1"/><text x="530" y="326" class="t-small" text-anchor="middle">1 m</text><path d="M610 304 L610 310" class="f-none s-muted" stroke-width="1"/><text x="610" y="326" class="t-small" text-anchor="middle">10 m</text><path d="M690 304 L690 310" class="f-none s-muted" stroke-width="1"/><text x="690" y="326" class="t-small" text-anchor="middle">100 m</text><path d="M770 304 L770 310" class="f-none s-muted" stroke-width="1"/><text x="770" y="326" class="t-small" text-anchor="middle">1 km</text><path d="M850 304 L850 310" class="f-none s-muted" stroke-width="1"/><text x="850" y="326" class="t-small" text-anchor="middle">10 km</text><rect x="20" y="340" width="12" height="12" class="f-warn s-warn"/><text x="38" y="350" class="t-small">thô hơn nhiễu GPS vài mét</text><rect x="250" y="340" width="12" height="12" class="f-phys s-phys"/><text x="268" y="350" class="t-small">cùng cỡ hoặc mịn hơn nhiễu GPS</text><rect x="520" y="340" width="12" height="12" class="f-dead s-dead"/><text x="538" y="350" class="t-small">mịn hơn mọi thứ điện thoại đo được</text></svg>
</div>
<figcaption>Hình 2. Số mét ứng với một đơn vị ở chữ số thập phân thứ d, theo vĩ độ (bảng của <code>loc03_toa_do</code>, mục 2). Thang ngang là logarit nên mỗi vạch cách nhau một bậc mười. Cách tô màu là nhận định của bài dựa trên câu «GPS điện thoại thường sai vài mét» trong chính lab, không phải số đo.</figcaption>
</figure>

Hai điều rút ra. Thứ nhất, **kinh độ ngắn dần khi lên bắc**: một độ kinh ở TP.HCM dài 109.235 m, ở Hà Nội 103.790 m, vì các kinh tuyến hội tụ về cực (độ dài bằng độ dài một độ vĩ nhân với cos của vĩ độ). Cà Mau, ở vĩ độ thấp hơn TP.HCM, cho 109.772 m. Vì vậy không có hằng số «mỗi độ bao nhiêu mét» chung cho cả hai trục. Thứ hai, **chữ số thứ 5 tương ứng khoảng 1,1 m, thứ 6 là 0,11 m, thứ 7 là 1,1 cm**. Định dạng polyline mã hoá của Google dùng 5 chữ số, một số dịch vụ dùng 6; GPS điện thoại thường sai vài mét, nên từ chữ số thứ 7 trở đi là độ chính xác mà điện thoại không đo được.

## `f64`, `f32` và `i32` E7

Với 5–7 chữ số có nghĩa, có ba cách lưu một số toạ độ. Lab thử cả ba trên 1 triệu điểm ngẫu nhiên trong khung Việt Nam (seed 20261106), đo sai số khi lưu rồi đọc lại:

| Cách lưu | Sai số lớn nhất | Sai số trung bình |
|---|---:|---:|
| `f64` (gốc) | 0 m | 0 m |
| `i32` E7 (độ × 10⁷) | 0,0078 m | 0,00417 m |
| `i32` E6 (độ × 10⁶) | 0,078048 m | 0,041693 m |
| `f32` | 0,422926 m | 0,212028 m |

`f32` sai tới **42 cm**: nó chỉ có khoảng bảy chữ số có nghĩa cho cả phần nguyên lẫn phần thập phân của một số như 106,698, nên không dùng được cho toạ độ. E7 sai tối đa 7,8 mm, thấp hơn nhiều mọi nhiễu GPS. E8 không vừa `i32`, vì 180° × 10⁸ = 18.000.000.000 lớn hơn `i32::MAX` = 2.147.483.647; E7 thì tối đa 1.800.000.000, còn dư chỗ. Dạng E7 được định nghĩa như sau:

`examples/toi-noi-location/rust/crates/geo-core/src/coord.rs`

```rust
    /// Nén về hai `i32` (độ × 10⁷, làm tròn gần nhất). Sai số tối đa 0,5·10⁻⁷ độ mỗi trục, khoảng 5,6 mm; theo đường chéo hai trục khoảng 7,8 mm.
    pub fn to_e7(self) -> LatLngE7 {
        LatLngE7 {
            lat_e7: (self.lat.0 * 1e7).round() as i32,
            lng_e7: (self.lng.0 * 1e7).round() as i32,
        }
    }
}

/// Dạng gọn 8 byte: độ × 10⁷ kiểu `i32`. Dùng trong đồ thị, chỉ mục, file nhị phân.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct LatLngE7 {
    lat_e7: i32,
    lng_e7: i32,
}
```

Về bộ nhớ, `LatLng` (hai `f64`) là 16 byte và `LatLngE7` (hai `i32`) là 8 byte, tiết kiệm 50%. Nhưng con số quan trọng phải đặt đúng quy mô: 25.000 tài xế trực tuyến tốn 0,40 MB so với 0,20 MB, nên bộ nhớ **không phải lý do chính** ở quy mô này. Với đồ thị TP.HCM 126.699 nút thì 2,03 MB so với 1,01 MB, cũng nhỏ. Lý do thật của E7 nằm ở nơi khác:

- **Hợp đồng và file nhị phân.** `lat_e7` và `lng_e7` kiểu `sint32` là cách toạ độ đi trên dây trong hợp đồng protobuf của lab; đồ thị lưu dưới dạng cột mảng liền nhau.
- **`Eq` và `Hash`.** `LatLngE7` dẫn xuất được `Eq` và `Hash`, nên làm khoá của `HashMap` được; một `f64` thì không (có NaN và hai cách viết số không).
- **`Pod`.** Struct hai `i32` ghi thẳng ra file và đọc lại bằng `bytemuck` không cần chuyển đổi.

Comment của `to_e7` ghi hai con số vì sai số có hai cách đo. 5,6 mm là sai số theo **từng trục** (0,5·10⁻⁷ độ nhân khoảng 111.195 m mỗi độ). 7,8 mm là khoảng cách haversine giữa điểm gốc và điểm đã lượng tử hoá, khi cả hai trục cùng lệch tối đa: đường chéo của ô E7, tức 5,6 mm nhân khoảng √2 (ít hơn một chút vì trục kinh độ co theo cos của vĩ độ).

Quy ước của lab: tính toán dùng `LatLng` (`f64`), lưu trữ và truyền dùng `LatLngE7`, và chuyển ở biên bằng `to_e7` và `to_latlng`.

## Vì sao float ổn với toạ độ mà không ổn với tiền

Hai câu hỏi hay đi liền nhau trong code review: sao toạ độ dùng `f64` được, còn tiền thì phải là số nguyên? Lab trả lời bằng số. Khoảng cách giữa hai số `f64` liền kề ở quanh kinh độ 106,698 là 1,42·10⁻¹⁴ độ, tức khoảng 1,56·10⁻⁹ m, cỡ nano mét. GPS điện thoại sai vài mét, nên sai số do float nằm dưới nhiễu hàng tỉ lần và không có ai quan tâm.

Với tiền, sai số bé nhưng **bị ép về một nấc rời rạc**:

`examples/toi-noi-location/rust/labs/src/bin/loc03_toa_do.rs`

```rust
    println!(
        "0.29 * 100.0 = {:?}, ép về i64 = {} (đáng ra 29 đồng)",
        0.29_f64 * 100.0,
        (0.29_f64 * 100.0) as i64
    );
    println!(
        "4.35 * 100.0 = {:?}, ép về i64 = {} (đáng ra 435 đồng)",
        4.35_f64 * 100.0,
        (4.35_f64 * 100.0) as i64
    );
```

Kết quả: `0.29 * 100.0` ra `28.999999999999996` và ép về `i64` thành 28 đồng thay vì 29; `4.35 * 100.0` ra 434,99999999999994 và thành 434 thay vì 435. Chênh một đồng là đủ để hai hệ thống đối soát không khớp. Cước của Tới Nơi (12.000 đồng cho 2 km đầu, 4.500 đồng mỗi km vượt) vì thế luôn là số nguyên đồng; loạt Rust qua Tới Nơi có kiểu `Money` cho việc đó. Còn toạ độ không có nấc rời rạc nào bị ép: hiếm khi ta so sánh hai toạ độ bằng `==`, và khi so thì so theo khoảng cách có dung sai.

## Chạy ví dụ và kiểm chứng

Từ gốc repo (trên máy viết `cargo` không có sẵn trên PATH, nên thêm đường dẫn toolchain trước; lab không cần PBF, Redis hay mạng):

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc03_toa_do
```

Cấu hình đo: `--release`, 1.000.000 điểm ngẫu nhiên, seed 20261106, Apple M1 Max. Mọi số trong bài này không phụ thuộc thời gian (sai số, số mét, kích thước), nên không bị ảnh hưởng bởi việc máy viết lúc đó chịu tải nặng. Output mục 1 và mục 3 (nguyên văn từ `results/p01.md`):

```text
-- 1. newtype chặn đảo lat/lng --
LatLng::new(Lat, Lng)          = LatLng(10.7725000, 106.6980000)
LatLng::new(Lng, Lat)          = lỗi biên dịch (doctest compile_fail trong geo-core/src/coord.rs)
try_new(106.6980, 10.7725)     = Err(vĩ độ 106.698 ngoài khoảng [-90, 90] (có thể đã đảo lat/lng))
Abuja (lat 9.07, lng 7.40) đảo thành try_new(7.40, 9.07) = LatLng(7.4000000, 9.0700000): khoảng giá trị không bắt được, chỉ kiểu mới chặn
to_lnglat() (thứ tự GeoJSON)   = [106.698, 10.7725]
GeoJSON                        = {"type":"FeatureCollection","features":[{"type":"Feature","geometry":{"type":"Point","coordinates":[106.698,10.7725]},"properties":{"tên":"Bến Thành"}}]}
from_lnglat(to_lnglat(p)) == p : true
```

```text
-- 3. f64, f32 và i32 E7: sai số và bộ nhớ --
mẫu: 1000000 điểm ngẫu nhiên trong khung Việt Nam (lat 8.0–23.4, lng 102.1–109.5), seed 20261106
  f64 (gốc)      sai số lớn nhất     0.000000 m   trung bình     0.000000 m
  i32 E7         sai số lớn nhất     0.007800 m   trung bình     0.004170 m
  i32 E6         sai số lớn nhất     0.078048 m   trung bình     0.041693 m
  f32            sai số lớn nhất     0.422926 m   trung bình     0.212028 m
E8 không vừa i32: 180° × 10⁸ = 18000000000 > i32::MAX = 2147483647
E7 lớn nhất: 180° × 10⁷ = 1800000000 (còn dư 347483647 so với i32::MAX)
size_of LatLng (2×f64) = 16 byte, size_of LatLngE7 (2×i32) = 8 byte, tiết kiệm 50%
  1 triệu điểm               f64:    16.00 MB   E7:     8.00 MB
  25.000 tài xế trực tuyến   f64:     0.40 MB   E7:     0.20 MB
  đồ thị HCM 126.699 nút     f64:     2.03 MB   E7:     1.01 MB
```

Dòng cuối của lab cũng cho biết việc **chưa làm**:

```text
[chưa chạy] WGS84 → VN-2000: cần phép biến đổi datum và lưới chiếu (thư viện proj); lab này chưa tính, nên bài chỉ nêu khái niệm.
```

Kiểm tra toàn phần (`bash examples/toi-noi-location/verify-p01.sh`) chạy `cargo test`, trong đó có doctest `compile_fail` của `LatLng::new`, rồi chạy lại lab này; kết quả ghi `PASS`.

## Giới hạn của ví dụ

Lab chưa chạy phép biến đổi WGS84 sang VN-2000 nên không có số nào về nó. Newtype chỉ chặn đảo thứ tự và giá trị ngoài khoảng: nó không biết một cặp toạ độ có nằm ở Việt Nam hay không, và không biết dữ liệu đến có thực sự là WGS84. Kiểu `LatLng` chưa có độ cao, chưa xử lý đường đổi ngày (kinh độ ±180°), và các bảng mét-trên-chữ-số tính trên mặt cầu bán kính trung bình thay vì ellipsoid WGS84. Toạ độ «Bến Thành» là xấp xỉ để minh hoạ. Hệ quả cho bài sau: mọi khoảng cách mà bài này nhắc tới đều là haversine trên mặt cầu, sai số của chính lựa chọn đó được đo ở bài khoảng cách.

## Bài tập

1. Một service nhận toạ độ qua JSON dạng `{"pos": [10.7725, 106.698]}` và một service khác gửi GeoJSON `{"coordinates": [106.698, 10.7725]}`. Hãy nêu ba chỗ lỗi đảo thứ tự có thể xảy ra, và bạn chặn từng chỗ bằng lớp nào (kiểu hay khoảng giá trị).

<details><summary>Lời giải</summary>

(1) Parse JSON của service thứ nhất rồi gán vào hàm nhận hai số: chặn bằng cách parse thẳng sang `Lat` và `Lng` có tên trường, không giữ mảng số thô. (2) Đọc GeoJSON bằng cách lấy phần tử 0 làm vĩ độ: chặn bằng `from_lnglat` là chỗ duy nhất đọc `[lng, lat]`. (3) Xuất ra GeoJSON bằng cách tự viết `[lat, lng]`: chặn bằng `to_lnglat`. Ở Việt Nam lớp khoảng giá trị bắt được cả ba, nhưng chỉ lớp kiểu mới bắt được ở Abuja.

</details>

2. Tính nhẩm: một tài xế báo vị trí với 5 chữ số thập phân. Sai số làm tròn theo vĩ độ tối đa là bao nhiêu mét? Có đủ để phân biệt hai làn của một đại lộ có dải phân cách (hai làn cách nhau vài chục mét) hay không?

<details><summary>Lời giải</summary>

Một đơn vị ở chữ số thứ 5 là khoảng 1,11 m theo vĩ độ, nên làm tròn tới số gần nhất sai tối đa khoảng một nửa, 0,56 m. Con số này nhỏ hơn nhiều khoảng cách giữa hai làn, nên làm tròn toạ độ không phải thứ khiến xe bị bám nhầm làn; nhiễu GPS vài mét mới là nguyên nhân, sẽ được xử lý ở các bài bám đường.

</details>

3. Viết (trên giấy) một proptest cho `to_e7` rồi `to_latlng`. Tính chất cần kiểm là gì, dung sai chọn bao nhiêu và vì sao không dùng `==` giữa hai `f64`?

<details><summary>Lời giải</summary>

Tính chất: với mọi `LatLng` hợp lệ `p`, `p` và `p.to_e7().to_latlng()` lệch nhau không quá 0,5·10⁻⁷ độ theo mỗi trục, tức khoảng 5,6 mm mỗi trục và dưới 8 mm theo đường chéo. Dung sai lấy từ bước lượng tử (0,5·10⁻⁷ độ) cộng một ít bù cho làm tròn `f64`, giống test `e7_round_trip_within_6mm` trong `coord.rs`. Không dùng `==` vì làm tròn về số nguyên rồi chia lại không trả đúng giá trị `f64` gốc, chỉ trả giá trị gần nó trong dung sai.

</details>

Bài trước: [bài 02](/articles/location-adr-rust-go-python-csharp-ranh-gioi-service). Bài tiếp: [bài 04](/articles/location-khoang-cach-haversine-equirectangular-sai-so).

## Tài liệu gốc

- [RFC 7946: The GeoJSON Format](https://datatracker.ietf.org/doc/html/rfc7946): thứ tự kinh độ rồi vĩ độ.
- [EPSG:4326, WGS 84](https://epsg.io/4326): định nghĩa hệ toạ độ địa lý WGS84.
- [PROJ](https://proj.org/): thư viện biến đổi hệ toạ độ, cần cho phép đổi sang VN-2000 mà lab chưa chạy.
- [Rust API Guidelines: Type safety](https://rust-lang.github.io/api-guidelines/type-safety.html): newtype tạo ra sự phân biệt tĩnh.
- [OpenStreetMap: bản quyền và giấy phép dữ liệu](https://www.openstreetmap.org/copyright).
- `examples/toi-noi-location/results/p01.md`, mục LOC-03; `examples/toi-noi-location/proto/README.md`: quy ước `lat_e7`, `lng_e7`.
