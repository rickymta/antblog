---
slug: location-tong-hop-geo-core-kiem-thu-hinh-hoc
title: "Tổng hợp P01: thư viện geo-core và kiểm thử hình học"
summary: "Bài 10: tổng kết Phần 1: crate geo-core có gì và chưa có gì, kiểm thử thuộc tính bằng proptest, đồ thị tổng hợp «khu mẫu» cho CI, và danh sách việc chưa làm hoặc chưa tra của P01."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, geo-core, proptest, testing, fixture, ci]
language: vi
---

Trong «khu mẫu» tổng hợp của lab có một đại lộ hai chiều ngăn bằng dải phân cách. Hai khách ghim hai bên đại lộ, cách nhau **56,0 m** chim bay. Tài xế ô tô đi từ cạnh đường gần khách phía bắc sang cạnh gần khách phía nam mất **305,5 m** theo một chiều và **114,8 m** theo chiều ngược lại, gấp khoảng 5,5 và 2 lần đường chim bay. Đó là bài toán «ghim sai phía đường» của cả loạt bài, thu nhỏ vào vài trăm nút giao mà chạy được trong CI không cần mạng.

Đây là bài 10 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi), bài cuối của Phần 1. Bài này không thêm thuật toán mới. Nó đóng sổ: crate `geo-core` đã có gì, cách kiểm các phép hình học bằng kiểm thử thuộc tính, vì sao có một đồ thị tổng hợp được commit, và quan trọng nhất, những gì P01 **chưa** làm hoặc chưa tra được. Mã nguồn được dựng dần từ [bài 03](/articles/location-toa-do-wgs84-newtype-lat-lng-do-chinh-xac) tới [bài 09](/articles/location-tim-tai-xe-gan-nhat-bon-cach-co-so-do). Tên đường trong khu mẫu là giả lập, không phải dữ liệu OSM.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Haversine trên mặt cầu thoả bất đẳng thức tam giác: <code>d(a, c) ≤ d(a, b) + d(b, c)</code>. Khoảng cách equirectangular (xấp xỉ mặt phẳng) có luôn thoả không? Quét 200.000 bộ ba điểm trong bbox HCM, phần vượt lớn nhất là bao nhiêu?</p>
<details><summary>Xem đáp án</summary><p>Không luôn thoả: equirectangular vượt tối đa <b>0,0007 m</b>, tức 4,64·10⁻⁸ của <code>d(a, c)</code>, còn haversine không vượt (phần vượt lớn nhất là −0,0000 m). Vì vậy test của equirect phải có dung sai nhỏ thay vì đòi đúng tuyệt đối.</p></details>
</div>

## `geo-core` có gì

`geo-core` là crate nền của P01. Mọi crate khác dựa vào nó, và nó không biết gì về bản đồ hay mạng. Phần đầu `lib.rs` liệt kê chín mô-đun và những kiểu được xuất lại:

`examples/toi-noi-location/rust/crates/geo-core/src/lib.rs`

```rust
pub mod bearing;
pub mod coord;
pub mod distance;
pub mod geohash;
pub mod geojson;
pub mod h3;
pub mod polyline;
pub mod projection;
pub mod segment;

pub use bearing::{bearing_deg, bearing_diff_deg};
pub use coord::{GeoError, Lat, LatLng, LatLngE7, Lng};
pub use distance::{EARTH_RADIUS_M, Meters, destination, equirect_m, haversine_m, to_meters};
pub use projection::LocalProjection;
pub use segment::{PolylineProjection, SegmentProjection, project_to_polyline, project_to_segment};
```

| Mô-đun | Có gì |
|---|---|
| `coord` | `Lat`, `Lng`, `LatLng` (đảo lat/lng là lỗi biên dịch), `LatLngE7` (số nguyên i32 nhân 10⁷), chuyển sang và từ thứ tự GeoJSON |
| `distance` | `haversine_m`, `equirect_m`, `destination`, `to_meters` (mét nguyên ở biên) |
| `projection` | `LocalProjection`: mặt phẳng mét quanh một điểm, cho R-tree và chiếu điểm |
| `segment` | chiếu điểm lên đoạn và polyline, vị trí dọc tuyến, `point_at` |
| `bearing` | `bearing_deg`, `bearing_diff_deg` |
| `polyline` | mã hoá và giải mã Encoded Polyline, precision 5 hoặc 6 |
| `geojson` | `point_feature`, `line_feature`, `polygon_feature`: xuất GeoJSON, đổi thứ tự qua `to_lnglat` |
| `h3`, `geohash` | `h3o` bọc theo kiểu `LatLng`, và geohash tối giản của bài 08 |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 380" role="img" aria-label="Hình 1. Ba crate của phần P01: geo-core ở dưới, road-graph và driver-store dựa vào nó; các lab loc03 đến loc10 dùng từng crate; số test của mỗi crate: 28 test đơn vị và 16 thuộc tính cho geo-core, 32 cho road-graph, 6 cho driver-store."><defs><marker id="location-tong-hop-geo-core-kiem-thu-hinh-hoc-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs><rect x="20" y="60" width="260" height="170" rx="10" class="f-phys-soft s-phys" stroke-width="2"/><text x="34.0" y="86.0" class="t-b">geo-core</text><text x="34.0" y="112.0" class="t-mono">coord</text><text x="158.0" y="112.0" class="t-mono">distance</text><text x="34.0" y="130.0" class="t-mono">projection</text><text x="158.0" y="130.0" class="t-mono">segment</text><text x="34.0" y="148.0" class="t-mono">bearing</text><text x="158.0" y="148.0" class="t-mono">polyline</text><text x="34.0" y="166.0" class="t-mono">geojson</text><text x="158.0" y="166.0" class="t-mono">h3</text><text x="34.0" y="184.0" class="t-mono">geohash</text><text x="34.0" y="216.0" class="t-small">28 test + 16 thuộc tính proptest</text><rect x="330" y="60" width="260" height="170" rx="10" class="f-pk-soft s-pk" stroke-width="2"/><text x="344.0" y="86.0" class="t-b">road-graph</text><text x="344.0" y="112.0" class="t-mono">model (CSR)</text><text x="468.0" y="112.0" class="t-mono">tags</text><text x="344.0" y="130.0" class="t-mono">builder</text><text x="468.0" y="130.0" class="t-mono">pbf</text><text x="344.0" y="148.0" class="t-mono">components</text><text x="468.0" y="148.0" class="t-mono">io</text><text x="344.0" y="166.0" class="t-mono">snap</text><text x="468.0" y="166.0" class="t-mono">fixture</text><text x="344.0" y="184.0" class="t-mono">sp</text><text x="344.0" y="216.0" class="t-small">32 test</text><rect x="640" y="60" width="220" height="170" rx="10" class="f-warn-soft s-warn" stroke-width="2"/><text x="654.0" y="86.0" class="t-b">driver-store</text><text x="654.0" y="112.0" class="t-mono">scan</text><text x="758.0" y="112.0" class="t-mono">rtree</text><text x="654.0" y="130.0" class="t-mono">h3_buckets</text><text x="758.0" y="130.0" class="t-mono">redis_geo</text><text x="654.0" y="216.0" class="t-small">6 test; 1 test cần Redis</text><path d="M280.0 145.0 L328.0 145.0" class="f-none s-muted" stroke-width="2" marker-end="url(#location-tong-hop-geo-core-kiem-thu-hinh-hoc-1-a)"/><path d="M150.0 60.0 L150.0 42.0 L750.0 42.0 L750.0 58.0" class="f-none s-muted" stroke-width="2" stroke-linejoin="round" stroke-linecap="round" marker-end="url(#location-tong-hop-geo-core-kiem-thu-hinh-hoc-1-a)"/><text x="450.0" y="36.0" class="t-small" text-anchor="middle">driver-store cũng chỉ dựa vào geo-core</text><text x="20.0" y="28.0" class="t-b">Cargo workspace của P01</text><text x="20.0" y="262.0" class="t-b">Lab dùng crate nào</text><rect x="20" y="274" width="168" height="50" rx="8" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="32.0" y="294.0" class="t-mono">loc03, loc05</text><text x="32.0" y="312.0" class="t-small">geo-core</text><rect x="200" y="274" width="188" height="50" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="212.0" y="294.0" class="t-mono">loc04, loc06, loc07</text><text x="212.0" y="312.0" class="t-small">road-graph + geo-core</text><rect x="400" y="274" width="198" height="50" rx="8" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="412.0" y="294.0" class="t-mono">loc08</text><text x="412.0" y="312.0" class="t-small">geo-core (h3, geohash)</text><rect x="610" y="274" width="118" height="50" rx="8" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="622.0" y="294.0" class="t-mono">loc09</text><text x="622.0" y="312.0" class="t-small">driver-store</text><rect x="740" y="274" width="120" height="50" rx="8" class="f-surface s-line" stroke-width="1.5"/><text x="752.0" y="294.0" class="t-mono">loc10</text><text x="752.0" y="312.0" class="t-small">road-graph, geo-core</text><text x="20.0" y="352.0" class="t-small">Test chạy bằng cargo test --workspace; kiểm tra toàn phần bằng verify-p01.sh.</text></svg>
</div>
<figcaption>Hình 1. Cargo workspace của P01: <code>road-graph</code> (bài 06, 07) và <code>driver-store</code> (bài 09) đều chỉ dựa vào <code>geo-core</code>. Mỗi lab <code>locNN</code> chạy trên crate nó cần.</figcaption>
</figure>

Một quyết định lặp lại xuyên suốt: mọi phép đổi thứ tự toạ độ nằm ở **một chỗ**. `LatLng` luôn là lat rồi lng; GeoJSON và Redis là lng rồi lat; `to_lnglat` và `from_lnglat` là cầu nối duy nhất, và test `geojson_feature_contains_lng_first` giữ nó đúng.

## Kiểm thử thuộc tính: điều kiện nào luôn đúng

Test theo ví dụ («từ Bến Thành tới Landmark 81 là bấy nhiêu mét») chỉ chứng minh vài điểm. Hình học có những **tính chất** phải đúng với mọi đầu vào: khoảng cách đối xứng, bất đẳng thức tam giác, chiếu một điểm lên đoạn thì chân chiếu nằm trên đoạn. `proptest` sinh đầu vào ngẫu nhiên có kiểm soát, kiểm tính chất, và khi tìm ra phản ví dụ thì thu nhỏ nó về dạng nhỏ nhất. Lab có 16 thuộc tính, mỗi thuộc tính 2.000 ca, sinh điểm trên toàn lãnh thổ Việt Nam (lat 8–24, lng 102–110) hoặc trong bbox TP.HCM.

`examples/toi-noi-location/rust/crates/geo-core/tests/properties.rs`

```rust
    #[test]
    fn haversine_triangle_inequality(a in vn_point(), b in vn_point(), c in vn_point()) {
        prop_assert!(haversine_m(a, c) <= haversine_m(a, b) + haversine_m(b, c) + 1e-6);
    }

    /// Equirectangular không phải khoảng cách thật trên mặt cầu, nên bất đẳng thức tam giác chỉ đúng
    /// trong sai số: ở quy mô thành phố sai số tương đối dưới 0,1%.
    #[test]
    fn equirect_triangle_inequality_within_tolerance(a in hcm_point(), b in hcm_point(), c in hcm_point()) {
        let tol = 0.001 * equirect_m(a, c) + 0.01;
        prop_assert!(equirect_m(a, c) <= equirect_m(a, b) + equirect_m(b, c) + tol);
    }

    #[test]
    fn equirect_close_to_haversine_in_city(a in hcm_point(), b in hcm_point()) {
        let h = haversine_m(a, b);
        prop_assert!((equirect_m(a, b) - h).abs() <= 0.001 * h + 0.01);
    }
```

Chú ý chỗ khác nhau: haversine là khoảng cách thật trên mặt cầu nên đòi đúng (chỉ trừ sai số làm tròn 1 µm). Equirect thì mỗi cặp điểm dùng một cos vĩ độ riêng, nên ba cạnh của tam giác không nằm trên cùng một mặt phẳng, và tính chất chỉ đúng **trong sai số**, với dung sai 0,1% cộng 1 cm. Dung sai không được chọn tuỳ tiện: nó phải khớp sai số đã biết của phép xấp xỉ.

`examples/toi-noi-location/rust/crates/geo-core/tests/properties.rs`

```rust
    #[test]
    fn projection_falls_on_segment(p in hcm_point(), a in hcm_point(), bearing in 0.0f64..360.0, len in 1.0f64..2000.0) {
        let b = destination(a, bearing, len);
        let r = project_to_segment(p, a, b);
        prop_assert!((0.0..=1.0).contains(&r.t));
        // chân nằm trên đoạn: tổng hai nửa bằng cả đoạn
        let seg = haversine_m(a, b);
        prop_assert!((haversine_m(a, r.point) + haversine_m(r.point, b) - seg).abs() < 0.01 + 1e-6 * seg);
        // không xa hơn hai đầu mút
        // (mặt phẳng gốc tại p lệch tới ~0,1% khi đầu mút xa p hàng chục km, nên dùng sai số tương đối)
        let tol = |d: f64| 0.001 * d + 0.01;
        prop_assert!(r.dist_m <= haversine_m(p, a) + tol(haversine_m(p, a)));
        prop_assert!(r.dist_m <= haversine_m(p, b) + tol(haversine_m(p, b)));
        // dist_m khớp khoảng cách thật tới chân
        prop_assert!((r.dist_m - haversine_m(p, r.point)).abs() < tol(r.dist_m));
    }
```

Thuộc tính chiếu đoạn này **từng thất bại**. Phiên bản đầu kiểm «chân chiếu không xa hơn đầu mút» với dung sai tuyệt đối 1 cm, thất bại khi điểm hỏi cách đầu mút hàng chục km: mặt phẳng cục bộ gốc tại điểm hỏi lệch tới 0,1% ở khoảng cách đó. Sửa là dùng sai số tương đối 0,1% cộng 1 cm. Từ đó suy ra: chiếu điểm chỉ nên dùng gần đoạn, như khi snap, và đúng với cách `road-graph` dùng nó.

Cùng tính chất, lab còn chạy một đợt quét 200.000 ca bằng `loc10_tong_hop` (seed 20261106) và in **con số lớn nhất** thay vì chỉ đạt hay không:

| Tính chất | Lớn nhất đo được |
|---|---|
| haversine đối xứng, 200.000 bộ ba (cả Việt Nam) | lệch 0 m |
| equirect đối xứng (bbox HCM) | lệch 0 m |
| tam giác, `d(a,c) − (d(a,b) + d(b,c))`, haversine | −0,0000 m (luôn thoả) |
| tam giác, equirect | 0,0007 m (4,64·10⁻⁸ của `d(a,c)`) |
| polyline precision 5, 2.000 tuyến 1–60 điểm | 0,7768 m |
| polyline precision 6 | 0,0777 m |
| lượng tử hoá E7 | 0,0078 m |
| chiếu lên đoạn, 200.000 ca: `t` ngoài [0, 1] | 0 ca |
| chân chiếu lệch khỏi đoạn, hoặc xa hơn hai đầu mút | 0,0000 m, 0 ca |
| bearing sau `destination` rồi `bearing_deg` (100–5.000 m) | lệch 0,0000° |

Những con số này là **số đo, không phải thời gian**, nên tái lập từng chữ số với cùng seed, không phụ thuộc máy đang tải. Hai điều cần đọc đúng. Một: precision 5 của polyline sai tới 0,78 m, E7 sai 7,8 mm, tức đủ cho hiển thị và lưu trữ nhưng không phải đủ cho mọi việc; chọn precision theo việc. Hai: thuộc tính chỉ kiểm tính chất **toán học** của công thức trên các mẫu đã sinh, không chứng minh công thức khớp mặt đất. Thuộc tính «đảo lat/lng luôn bị từ chối» (`swapped_lat_lng_always_rejected_in_vietnam`) đúng trong Việt Nam vì khi đảo, kinh độ 102–110 thành «vĩ độ» vượt khoảng cho phép; ở nơi như Abuja (lat 9,07; lng 7,40) đảo thứ tự vẫn ra toạ độ hợp lệ, khoảng giá trị không bắt được, và chỉ kiểu `LatLng` mới chặn.

## Đồ thị tổng hợp cho CI

CI không thể phụ thuộc file PBF 332 MB và mạng. Lab có `fixtures/khu-mau.json`, một đồ thị tổng hợp do `gen_khu_mau` sinh có seed: lưới 16 cột × 13 hàng, bước 100 m, lệch ngẫu nhiên theo seed. File JSON có dạng OSM thu nhỏ (nút, way kèm thẻ, quan hệ cấm rẽ, cổng) và **đi qua đúng đường biên dịch của bản đồ thật** (`compile`), nên test chạy trên khu mẫu kiểm chính mã chạy trên HCM. Nó phải sinh lại được từng byte, và lab kiểm điều đó:

`examples/toi-noi-location/rust/labs/src/bin/loc10_tong_hop.rs`

```rust
    let path = labs::fixture_path();
    let on_disk = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| labs::die(&format!("thiếu fixture: {e}")));
    let regenerated = to_json(&generate());
    println!(
        "  file {} byte; sinh lại từ seed {} cho {} byte; trùng từng byte: {}",
        on_disk.len(),
        road_graph::fixture::FIXTURE_SEED,
        regenerated.len(),
        on_disk == regenerated
    );
```

Kết quả: file 50.678 byte, sinh lại từ seed 20261106 cho đúng 50.678 byte, trùng từng byte. File có 1.041 nút OSM-mini, 48 way, 3 quan hệ cấm rẽ, 3 cổng; biên dịch ra 234 nút giao, 840 cạnh có hướng, 806 điểm hình dạng, với 2 thành phần liên thông trước khi lọc. Khu mẫu được thiết kế **chứa sẵn các tình huống khó** để bài sau có chỗ thử:

| Tình huống | Có trong khu mẫu (đếm từ output) |
|---|---|
| Cầu ô tô được đi, cầu chỉ cho người đi bộ | 2 cạnh, 2 cạnh |
| Hẻm xe máy và đi bộ vào được, ô tô không (cả hai chiều) | 4 cạnh (hẻm `width=2.2` và hẻm `motor_vehicle=no`) |
| Hẻm cụt rộng 4 m ô tô vào được | 2 cạnh |
| Đường một chiều cho ô tô; xe máy đi ngược chiều ô tô | 101 cạnh; 16 cạnh |
| Cầu vượt cắt đại lộ mà không có nút chung | dài 196 m, 0 cạnh đại lộ chạm đầu cầu |
| Quan hệ cấm rẽ giải được thành cạnh vào và ra | 3 |
| Mọi profile: SCC lớn nhất phủ bao nhiêu nút | 100,0% cả ba profile |

Hình 2 vẽ đại lộ có dải phân cách từ chính `results/geojson/loc10-khu-mau-dai-lo.geojson`. Mỗi chiều xe là một way một chiều, cách nhau cỡ 20 m, chỉ cho cắt ngang ở cột 0, 5, 10, 15.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 440" role="img" aria-label="Hình 2. Khu mẫu tổng hợp: hai khách ghim hai phía một đại lộ có dải phân cách cách nhau 56,0 mét chim bay; cạnh gần nhất của khách phía bắc là chiều tây, của khách phía nam là chiều đông."><defs><marker id="location-tong-hop-geo-core-kiem-thu-hinh-hoc-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs><rect x="20" y="20" width="600" height="400" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36.0" y="46.0" class="t-b">Đại lộ Trung Tâm (khu mẫu tổng hợp, tên giả lập)</text><rect x="83.6167" y="202.53" width="472.767" height="57.7749" rx="4" class="f-surface s-dead dash" stroke-width="1"/><path d="M556.4 166.3 L398.5 172.4 L241.0 183.4 L83.6 196.5" class="f-none s-warn" stroke-width="4" stroke-linejoin="round" stroke-linecap="round" marker-end="url(#location-tong-hop-geo-core-kiem-thu-hinh-hoc-2-a)"/><path d="M83.6 296.6 L241.1 285.4 L399.0 280.5 L556.4 266.3" class="f-none s-pk" stroke-width="4" stroke-linejoin="round" stroke-linecap="round" marker-end="url(#location-tong-hop-geo-core-kiem-thu-hinh-hoc-2-a)"/><circle cx="329.7" cy="90.0" r="7" class="f-warn s-warn" stroke-width="1.5"/><circle cx="329.7" cy="370.0" r="7" class="f-pk s-pk" stroke-width="1.5"/><path d="M359.7 90.0 L359.7 370.0" class="f-none s-muted" stroke-width="1.5" marker-start="url(#location-tong-hop-geo-core-kiem-thu-hinh-hoc-2-a)" marker-end="url(#location-tong-hop-geo-core-kiem-thu-hinh-hoc-2-a)"/><text x="367.7" y="234.0" class="t-b">56,0 m chim bay</text><text x="317.7" y="94.0" text-anchor="end">ghim phía bắc</text><text x="317.7" y="374.0" text-anchor="end">ghim phía nam</text><text x="508.0" y="235.4" class="t-small" text-anchor="middle">dải phân cách</text><text x="75.6" y="154.3" class="t-small">chiều tây, hướng 266°, cách ghim 17,4 m</text><text x="75.6" y="318.6" class="t-small">chiều đông, hướng 88°, cách ghim 17,5 m</text><path d="M36.0 402.0 L136.0 402.0" class="f-none s-muted" stroke-width="2"/><text x="144.0" y="406.0" class="t-small">20 m</text><rect x="640" y="20" width="220" height="400" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="656.0" y="46.0" class="t-b">Quãng đường thật</text><text x="656.0" y="76.0" class="t-small">ô tô, bắc → nam</text><text x="656.0" y="104.0" class="t-big">305,5 m</text><text x="656.0" y="132.0" class="t-small">ô tô, nam → bắc</text><text x="656.0" y="160.0" class="t-big">114,8 m</text><text x="656.0" y="188.0" class="t-small">đi bộ, bắc → nam</text><text x="656.0" y="216.0" class="t-big">114,8 m</text><text x="656.0" y="244.0" class="t-small">chim bay</text><text x="656.0" y="272.0" class="t-big">56,0 m</text><text x="656.0" y="312.0" class="t-small">đo giữa hai nút cuối của cạnh</text><text x="656.0" y="327.0" class="t-small">snap, xấp xỉ: không tính phần</text><text x="656.0" y="342.0" class="t-small">cạnh còn lại tới chân snap.</text><text x="656.0" y="372.0" class="t-small">Mỗi chiều đại lộ là một way</text><text x="656.0" y="387.0" class="t-small">một chiều, chỉ cắt ngang ở</text><text x="656.0" y="402.0" class="t-small">cột 0, 5, 10, 15.</text></svg>
</div>
<figcaption>Hình 2. Hai khách ghim hai phía đại lộ trong khu mẫu tổng hợp: cạnh gần nhất của khách phía bắc là chiều tây (hướng 266°, cách 17,4 m), của khách phía nam là chiều đông (hướng 88°, cách 17,5 m). Vẽ từ <code>loc10-khu-mau-dai-lo.geojson</code>; dữ liệu tổng hợp, không phải OSM.</figcaption>
</figure>

Khoảng cách hai ghim là 56,0 m, nhưng ô tô đi từ nút cuối cạnh phía bắc tới nút cuối cạnh phía nam mất 305,5 m, còn chiều ngược lại và người đi bộ đều 114,8 m (305,5 − 114,8 = 190,7 m dài hơn). Lab không in ra tuyến cụ thể, nên bài chỉ nói đúng điều đo được: vì mỗi chiều đại lộ là một chiều và chỉ cắt ngang ở vài cột, **đường chim bay không nói gì về quãng đường thật**. Số đo là giữa hai nút cuối của cạnh snap, xấp xỉ, chưa tính phần cạnh còn lại tới chân snap.

## Kiểm tra toàn phần và cái nó cố ý bỏ qua

`verify-p01.sh` chạy tuần tự: `cargo fmt --check`; `cargo clippy` với `-D warnings` trên mọi target; `cargo test` trên cả workspace; sinh lại fixture và so mã băm với file đã commit; rồi chạy mọi lab không cần PBF lẫn Redis (`loc03`, `loc04 --fixture`, `loc05`, `loc06 --fixture`, `loc07 --fixture`, `loc08`, `loc09 --no-redis`, `loc10`). In `PASS` nếu tất cả qua. Tổng số test của lần chạy cuối: `geo-core` 28 test đơn vị và 16 thuộc tính, `road-graph` 32, `driver-store` 6, cộng doctest `compile_fail` của `geo-core` kiểm đảo lat/lng; test cần Redis thật chạy riêng và có 1 test qua.

Cái script **cố ý không chạy** cũng quan trọng không kém: `loc06` trên PBF thật, `loc04` và `loc07` trên đồ thị HCM, `loc09` với Redis, và `cargo bench`. Các kết quả của chúng nằm trong `results/p01.md` và được nêu trong bài tương ứng. Nguyên tắc chung của lab: mọi lab dùng dữ liệu thiếu mà không có cờ tường minh thì thoát mã 2 kèm thông báo, không bao giờ bỏ qua âm thầm.

## Chạy ví dụ và kiểm chứng

Không cần PBF, Redis hay mạng. Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc10_tong_hop
cargo test --manifest-path examples/toi-noi-location/rust/Cargo.toml --workspace
bash examples/toi-noi-location/verify-p01.sh
```

Cấu hình: `--release` cho lab, `debug` cho test; seed 20261106; `proptest` 2.000 ca mỗi thuộc tính, `failure_persistence` tắt để không sinh file. Lab còn ghi `results/geojson/loc10-khu-mau-dai-lo.geojson`. Output rút gọn:

```text
  file 50678 byte; sinh lại từ seed 20261106 cho 50678 byte; trùng từng byte: true
  đồ thị CSR: 234 nút (nút giao), 840 cạnh có hướng, 806 điểm hình dạng; thành phần liên thông trước lọc: 2
  ghim phía bắc: gần nhất «Đại lộ Trung Tâm (chiều tây)» cách 17.4 m, bearing 266°
  ghim phía nam: gần nhất «Đại lộ Trung Tâm (chiều đông)» cách 17.5 m, bearing 88°
  chim bay giữa hai ghim: 56.0 m
  ô tô, giữa hai nút đầu cạnh snap (xấp xỉ, nút cuối cạnh): 223 → 211: Some(305.46) m; chiều ngược lại: Some(114.75) m
    tam giác: d(a,c) - (d(a,b) + d(b,c)) lớn nhất: haversine -0.0000 m (≤ 0 nghĩa là luôn thỏa); equirect 0.0007 m (4.64e-8 của d(a,c))
```

Mọi số ở đây là số hình học và số đếm, tái lập từng chữ số; bài không có số thời gian để cảnh báo. `cargo clippy --workspace --all-targets -- -D warnings` và `cargo fmt --check` đều thoát mã 0.

## Giới hạn của ví dụ: những gì P01 chưa có

- **Định tuyến.** Chỉ có Dijkstra tối giản trong `road_graph::sp`, dùng để đo hệ số vòng; không A*, không Contraction Hierarchies, không chi phí rẽ. Phần định tuyến bắt đầu ở LOC-21.
- **Cấm rẽ mới đọc, chưa dùng.** 270 relation có `via` là way chưa giải; 784 cấm theo giờ giữ thô (LOC-23).
- **Cổng toà nhà (`entrance=*`)** mới lưu thô, chưa gắn vào đồ thị; chưa có hệ toạ độ trong nhà, nơi GPS vô dụng (LOC-37).
- **Bám đường, lọc Kalman, tốc độ theo giờ** thuộc P02 và P03. Snap hiện tại là snap từng điểm.
- **Hệ toạ độ và hình học.** Chỉ WGS84; VN-2000 chưa hỗ trợ. Khoảng cách dùng mặt cầu bán kính 6.371.008,8 m, lệch tới 0,526% so với ellipsoid WGS84 ở vĩ độ 10,77 (bài 04); `geo-core` chưa có hàm trắc địa. Geohash và mặt phẳng cục bộ chỉ kiểm cho vùng quanh Việt Nam. `geo-core` cũng chưa có phép toán trên đa giác (vùng cấm đón, geofence).
- **Hạ tầng.** Redis GEO chỉ đo qua Docker loopback; PostGIS không chạy.
- **Không có GPS người thật.** Mọi vị trí tài xế, nhiễu GPS đều do lab sinh.
- **Số thời gian.** Hầu hết số thời gian của P01 đo khi máy tải nặng, và chưa có lần đo trên máy rảnh được lưu đầy đủ; chỉ nên đọc tỉ lệ trong cùng một lần chạy.
- **Chưa tách riêng:** tỉ lệ «snap sai» của bài 07 đếm theo mã way nên là cận trên. Ca R-tree và quét chọn khác cạnh ở 1/1.000 truy vấn đã kiểm: hoà đúng nghĩa ở nút giao (bài 07), nhưng `nearest` chưa có quy tắc phá hoà.

## Bài tập

**Bài 1.** Vì sao `haversine_triangle_inequality` chỉ cho dung sai 1e-6 m, còn bản equirect cần 0,1% cộng 1 cm?

<details><summary>Lời giải bài 1</summary>
<p>Haversine là khoảng cách thật trên mặt cầu nên đúng bất đẳng thức tam giác, và 1e-6 m chỉ bù sai số làm tròn số thực. Equirect là xấp xỉ trên mặt phẳng nên có thể vượt một chút (lab đo tối đa 0,0007 m, 4,64·10⁻⁸ của <code>d(a, c)</code>). Dung sai 0,1% khớp mức sai số của mặt phẳng cục bộ trong bbox thành phố, còn 1 cm lo các khoảng cách rất ngắn.</p>
</details>

**Bài 2.** Fixture sinh lại phải trùng từng byte với file đã commit. Việc đó bảo vệ điều gì mà chỉ commit file thì không?

<details><summary>Lời giải bài 2</summary>
<p>Nó buộc generator tất định: cùng seed ra cùng byte, không phụ thuộc thứ tự duyệt bảng băm hay thời gian. Nó cũng phát hiện sửa tay file JSON mà không sửa generator, hay ngược lại. Nhờ đó test và các bài sau có thể trích số từ khu mẫu mà người đọc chạy lại ra đúng số.</p>
</details>

**Bài 3.** Từ ví dụ đại lộ, vì sao hàm chọn điểm đón không thể xếp hạng ứng viên theo khoảng cách chim bay giữa khách và điểm đón?

<details><summary>Lời giải bài 3</summary>
<p>Hai ghim chỉ cách nhau 56,0 m nhưng ô tô đi 305,5 m một chiều và 114,8 m chiều kia; điểm đón ở phía nào của dải phân cách làm quãng đường chênh gấp 2,7 lần (305,5 chia 114,8). Khoảng cách chim bay không biết dải phân cách, một chiều và nút chặn. Đây là số đo trên một khu mẫu tổng hợp, nên chỉ minh hoạ cơ chế; mức độ ngoài đời phải đo bằng engine định tuyến và simulator ở các phần sau.</p>
</details>

Bài trước: [bài 09](/articles/location-tim-tai-xe-gan-nhat-bon-cach-co-so-do). Bài tiếp: [bài 11](/articles/location-go-goroutine-channel-nhap-mon-cho-ingest).

## Tài liệu gốc

- [proptest trên docs.rs](https://docs.rs/proptest) và [kho mã proptest](https://github.com/proptest-rs/proptest): kiểm thử thuộc tính, thu nhỏ phản ví dụ.
- K. Claessen và J. Hughes, «QuickCheck: a lightweight tool for random testing of Haskell programs», ICFP 2000: nguồn gốc ý tưởng kiểm thử thuộc tính.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): dữ liệu OSM ở các bài trước của phần này.
- [Tài liệu Cargo về workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html).
