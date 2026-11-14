---
slug: location-khoang-cach-haversine-equirectangular-sai-so
title: "Khoảng cách: haversine, equirectangular và sai số đo được"
summary: "Bài 04: công thức haversine và equirectangular, sai số đo ở vĩ độ Việt Nam, sai số của mô hình mặt cầu so với ellipsoid, tốc độ (đo khi máy tải nặng) và hệ số vòng đường đi trên chim bay ở TP.HCM."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, haversine, equirectangular, geodesic, routing]
language: vi
---

Ở bài mở đầu có lúc tài xế chạy ngang qua đúng chỗ ghim, chỉ cách khách một dải phân cách. Nếu app hiện «tài xế cách bạn …», con số đó là mấy chục mét hay 1.200 m? Cả hai đều đúng, tuỳ bạn hỏi đường chim bay hay đường xe đi. Service định vị cần cả hai: chim bay để lọc nhanh, đường đi để quyết định. Và mỗi phép đo có một độ sai số riêng mà ta nên biết bằng số, không bằng cảm giác.

Bài này tính khoảng cách giữa hai toạ độ bằng hai công thức, đo xem chúng sai bao nhiêu mét ở vĩ độ Việt Nam và độ dài vài kilômét, đo tốc độ, rồi so với quãng đường xe thật sự đi trên đồ thị đường bộ TP.HCM. Đây là bài 04 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng `LatLng` của [bài 03](/articles/location-toa-do-wgs84-newtype-lat-lng-do-chinh-xac) và đồ thị đường bộ của bài [từ OpenStreetMap tới đồ thị](/articles/location-osm-pbf-do-thi-duong-bo-xe-may-o-to); nếu chưa có đồ thị, lab có cờ `--fixture` để chạy trên đồ thị tổng hợp.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Hai cách xấp xỉ khoảng cách hai điểm cách nhau 10 km quanh TP.HCM: coi vùng đó là mặt phẳng (equirectangular) hoặc coi Trái Đất là mặt cầu hoàn hảo (haversine). Theo bạn cách nào sai nhiều hơn so với con số thật trên ellipsoid WGS84, và sai cỡ bao nhiêu mét?</p>
<details><summary>Xem đáp án</summary><p>Cả hai cách <b>sai gần như bằng nhau</b>, vì cùng dùng bán kính cầu 6.371.008,8 m: lệch tới <b>52,371 m</b> ở 10 km (0,526%) so với trắc địa ellipsoid. Còn chênh lệch giữa equirectangular và haversine chỉ <b>0,00029 m</b> ở cùng khoảng cách. Chọn công thức nào gần như không quan trọng bằng việc chấp nhận mô hình mặt cầu.</p></details>
</div>

## Haversine: khoảng cách trên mặt cầu

Với hai điểm có vĩ độ φ₁, φ₂ và kinh độ λ₁, λ₂ (đổi sang radian), công thức haversine tính `h`, tức sin² của nửa góc ở tâm giữa chúng, rồi suy ra cung tròn lớn:

```text
h = sin²(Δφ/2) + cos φ₁ · cos φ₂ · sin²(Δλ/2)
d = 2R · asin(√h)
```

`R` là bán kính cầu. Lab dùng bán kính trung bình của Trái Đất theo IUGG, 6.371.008,8 m:

`examples/toi-noi-location/rust/crates/geo-core/src/distance.rs`

```rust
/// Bán kính trung bình của Trái Đất (IUGG), mét.
pub const EARTH_RADIUS_M: f64 = 6_371_008.8;
```

`examples/toi-noi-location/rust/crates/geo-core/src/distance.rs`

```rust
/// Haversine: khoảng cách cung tròn lớn, ổn định số học với khoảng cách ngắn.
pub fn haversine_m(a: LatLng, b: LatLng) -> f64 {
    let (p1, p2) = (a.lat().to_radians(), b.lat().to_radians());
    let dphi = p2 - p1;
    let dlam = (b.lng() - a.lng()).to_radians();
    let h = (dphi / 2.0).sin().powi(2) + p1.cos() * p2.cos() * (dlam / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS_M * h.sqrt().min(1.0).asin()
}
```

Haversine dùng `sin²` thay vì `acos` của luật cosin cầu. Khi hai điểm rất gần nhau, `acos` nhận số sát 1 và mất nhiều chữ số có nghĩa: với `f64`, luật cosin vẫn tạm dùng được tới cỡ vài mét rồi sai số tương đối tăng nhanh. Còn `sin²` của một góc nhỏ giữ nguyên độ chính xác, nên haversine ổn ở mọi khoảng cách ngắn. `min(1.0)` chặn sai số làm tròn đẩy `√h` vượt 1 và sinh `NaN`. Kết quả là mét ở kiểu `f64`; ở biên công khai lab làm tròn về mét nguyên (`Meters = u32`) theo quy ước của cả loạt.

## Equirectangular: trải mặt cầu ra mặt phẳng

Nếu hai điểm đủ gần, có thể bỏ qua độ cong: trải vùng quanh vĩ độ trung bình ra mặt phẳng, với trục ngang co lại theo cos của vĩ độ (vì kinh tuyến hội tụ), rồi dùng định lý Pythagoras:

`examples/toi-noi-location/rust/crates/geo-core/src/distance.rs`

```rust
/// Equirectangular: coi mặt cầu như mặt phẳng quanh vĩ độ trung bình. Nhanh hơn, đủ chính xác ở vài km.
pub fn equirect_m(a: LatLng, b: LatLng) -> f64 {
    let (p1, p2) = (a.lat().to_radians(), b.lat().to_radians());
    let x = (b.lng() - a.lng()).to_radians() * ((p1 + p2) / 2.0).cos();
    let y = p2 - p1;
    EARTH_RADIUS_M * x.hypot(y)
}
```

Không có `asin`, không có `sqrt` lồng, chỉ một `cos` và một `hypot`. Đổi lại, sai số tăng theo khoảng cách và theo vĩ độ. Lab đo bằng cách dựng điểm B cách A đúng `d` mét theo hướng ngẫu nhiên trên mặt cầu (hàm `destination`), nên haversine(A, B) bằng `d`, rồi so equirect với haversine, 20.000 cặp mỗi ô, seed 20261106:

| Vĩ độ A | d (m) | Trung bình (m) | Lớn nhất (m) | Lớn nhất (%) |
|---:|---:|---:|---:|---:|
| 10,77 | 1.000 | 0,00000 | 0,00000 | 0,0000000 |
| 10,77 | 5.000 | 0,00002 | 0,00004 | 0,0000007 |
| 10,77 | 10.000 | 0,00016 | 0,00029 | 0,0000029 |
| 10,77 | 50.000 | 0,01951 | 0,03690 | 0,0000738 |
| 23,00 | 50.000 | 0,03350 | 0,05632 | 0,0001126 |

Ở TP.HCM, equirectangular lệch haversine tối đa 0,00029 m ở 10 km và 0,037 m ở 50 km; ngay cả ở vĩ độ 23, gần cực bắc lãnh thổ, chỉ 0,056 m ở 50 km. Sai số mét tăng gần như theo lập phương khoảng cách: từ 10 km lên 50 km là gấp 5 lần, còn sai số gấp 127 lần (5³ = 125). Vì thế ở vài km nó nhỏ hơn nhiễu GPS hàng nghìn lần, và ở 50 km vẫn nhỏ hơn vài chục lần.

Một biến thể gặp ở R-tree và ở bài chiếu điểm lên đường là **mặt phẳng mét cố định**: chọn một điểm gốc (tâm bbox TP.HCM), cố định `cos φ₀`, và đổi mọi toạ độ sang (x, y) mét bằng một lần nhân:

`examples/toi-noi-location/rust/crates/geo-core/src/projection.rs`

```rust
impl LocalProjection {
    pub fn new(origin: LatLng) -> Self {
        Self {
            origin,
            cos_lat0: origin.lat().to_radians().cos(),
        }
    }

    pub fn origin(&self) -> LatLng {
        self.origin
    }

    /// (x về đông, y về bắc), đơn vị mét.
    pub fn to_xy(&self, p: LatLng) -> [f64; 2] {
        let x = (p.lng() - self.origin.lng()).to_radians() * self.cos_lat0 * EARTH_RADIUS_M;
        let y = (p.lat() - self.origin.lat()).to_radians() * EARTH_RADIUS_M;
        [x, y]
    }
```

Vì `cos φ₀` không đổi nên điểm càng xa vĩ độ gốc thì trục đông tây càng sai. Lab đo điểm A ngẫu nhiên trong bbox lat 10,70–10,86, lng 106,60–106,78:

| d (m) | Lớn nhất (m) | Lớn nhất (%) |
|---:|---:|---:|
| 500 | 0,1328 | 0,02655 |
| 2.000 | 0,5339 | 0,02670 |
| 5.000 | 1,3578 | 0,02716 |
| 10.000 | 2,8492 | 0,02849 |

Sai số tương đối gần như không đổi theo `d`, khoảng 0,027%, vì nó đến từ một hệ số co sai chứ không đến từ độ cong: ở mép bbox, cách vĩ độ tâm 0,08° (0,0014 radian), cos thật lệch `cos φ₀` một lượng tương đối tan(10,78°) × 0,0014 ≈ 0,027%, và lượng đó nhân vào mọi đoạn theo hướng đông tây. Ở 10 km là 2,85 m, không vượt nhiễu GPS vài mét, nên đủ cho R-tree và chiếu điểm, nhưng lớn hơn equirect «cos trung bình hai đầu» cỡ mười nghìn lần. Đó là cái giá của việc dùng một hằng số cho cả thành phố.

## Sai số lớn nằm ở mô hình mặt cầu

Hai công thức trên đều coi Trái Đất là hình cầu. Thực tế nó là ellipsoid, dẹt ở hai cực, nên bán kính cong theo hướng bắc nam ở vĩ độ thấp nhỏ hơn bán kính trung bình. Lab so haversine và equirect với `geo::Geodesic` (khoảng cách trắc địa trên ellipsoid WGS84), 20.000 cặp, A ở vĩ độ 10,77:

| d (m) | Haversine lớn nhất (m) | Haversine lớn nhất (%) | Equirect lớn nhất (m) |
|---:|---:|---:|---:|
| 500 | 2,6172 | 0,52619 | 2,6172 |
| 2.000 | 10,4696 | 0,52624 | 10,4696 |
| 5.000 | 26,1783 | 0,52632 | 26,1783 |
| 10.000 | 52,3710 | 0,52647 | 52,3710 |

Hai cột equirect và haversine trùng nhau tới 4 chữ số thập phân, vì cùng bán kính cầu. Sai số khoảng 0,526%, lớn nhất theo hướng bắc nam, tức khoảng 2,6 m ở 500 m và 52 m ở 10 km. Hình 1 đặt ba loại sai số cạnh nhau trên một thang.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 322" role="img" aria-label="Hình 1. Sai số lớn nhất của ba cách xấp xỉ khoảng cách, thang logarit. Ở 5.000 mét: equirectangular so với haversine 0,00004 mét, mặt phẳng mét cố định tại tâm bbox so với haversine 1,3578 mét, haversine so với ellipsoid WGS84 26,1783 mét. Ở 10.000 mét: 0,00029 mét, 2,8492 mét và 52,3710 mét."><text x="20" y="46" class="t-b">Khoảng cách 5.000 m</text><text x="290" y="72" class="t-small" text-anchor="end">equirect (cos hai đầu) so với haversine</text><rect x="300" y="58" width="45.2" height="20" class="f-phys s-phys"/><text x="353.2" y="72" class="t-mono">0,00004 m</text><text x="290" y="100" class="t-small" text-anchor="end">mặt phẳng mét cố định so với haversine</text><rect x="300" y="86" width="385" height="20" class="f-pk s-pk"/><text x="693" y="100" class="t-mono">1,3578 m</text><text x="290" y="128" class="t-small" text-anchor="end">haversine so với ellipsoid WGS84</text><rect x="300" y="114" width="481.3" height="20" class="f-warn s-warn"/><text x="789.3" y="128" class="t-mono">26,1783 m</text><text x="20" y="168" class="t-b">Khoảng cách 10.000 m</text><text x="290" y="194" class="t-small" text-anchor="end">equirect (cos hai đầu) so với haversine</text><rect x="300" y="180" width="109.7" height="20" class="f-phys s-phys"/><text x="417.7" y="194" class="t-mono">0,00029 m</text><text x="290" y="222" class="t-small" text-anchor="end">mặt phẳng mét cố định so với haversine</text><rect x="300" y="208" width="409.1" height="20" class="f-pk s-pk"/><text x="717.1" y="222" class="t-mono">2,8492 m</text><text x="290" y="250" class="t-small" text-anchor="end">haversine so với ellipsoid WGS84</text><rect x="300" y="236" width="503.9" height="20" class="f-warn s-warn"/><text x="811.9" y="250" class="t-mono">52,3710 m</text><path d="M300 282 L825 282" class="f-none s-muted" stroke-width="1"/><path d="M300 282 L300 288" class="f-none s-muted" stroke-width="1"/><text x="300" y="304" class="t-small" text-anchor="middle">0,00001</text><path d="M375 282 L375 288" class="f-none s-muted" stroke-width="1"/><text x="375" y="304" class="t-small" text-anchor="middle">0,0001</text><path d="M450 282 L450 288" class="f-none s-muted" stroke-width="1"/><text x="450" y="304" class="t-small" text-anchor="middle">0,001</text><path d="M525 282 L525 288" class="f-none s-muted" stroke-width="1"/><text x="525" y="304" class="t-small" text-anchor="middle">0,01</text><path d="M600 282 L600 288" class="f-none s-muted" stroke-width="1"/><text x="600" y="304" class="t-small" text-anchor="middle">0,1</text><path d="M675 282 L675 288" class="f-none s-muted" stroke-width="1"/><text x="675" y="304" class="t-small" text-anchor="middle">1</text><path d="M750 282 L750 288" class="f-none s-muted" stroke-width="1"/><text x="750" y="304" class="t-small" text-anchor="middle">10</text><path d="M825 282 L825 288" class="f-none s-muted" stroke-width="1"/><text x="825" y="304" class="t-small" text-anchor="middle">100</text><text x="837" y="304" class="t-small">m</text><text x="20" y="304" class="t-small">sai số lớn nhất</text></svg>
</div>
<figcaption>Hình 1. Thang ngang là logarit, mỗi vạch cách nhau một bậc mười. Sai số của hai cách xấp xỉ phẳng không vượt nhiễu GPS vài mét; sai số lớn hơn nhiều đến từ việc coi Trái Đất là mặt cầu thay vì ellipsoid. Số từ <code>loc04_khoang_cach</code>, mục 1, 1b và 2 trong <code>results/p01.md</code>.</figcaption>
</figure>

Kết luận cho thiết kế: **chọn haversine hay equirect không đáng kể so với việc chấp nhận mô hình cầu**. Nếu cần độ chính xác trắc địa thật, dùng `Geodesic`, nhưng trả giá bằng tốc độ, như phần sau cho thấy. Một ghi chú nhỏ: haversine của `geo-core` lệch haversine của crate `geo` tối đa 1,07·10⁻¹⁰ m, nên hai bản cài đặt cho cùng kết quả.

## Tốc độ: đo khi máy tải nặng, nên chỉ tin vào thứ bậc

Lab đo 1.000.000 cặp quanh TP.HCM cách nhau 50 m đến 5 km, mỗi công thức 5 lần, `--release`, Apple M1 Max, seed 20261106. **Máy viết khi đó đang chịu tải rất nặng** (tải trung bình 1 phút từ khoảng 74 tới hơn 200 trên 10 lõi, theo `uptime`), nên con số thời gian bị phóng đại và dao động mạnh giữa các lần chạy. Cách đúng là nhìn thứ bậc và tỉ lệ trong cùng một lần chạy:

| Lần chạy | Haversine nhanh nhất (ns/cặp) | Equirect nhanh nhất (ns/cặp) | Haversine / equirect (trung vị) | Geodesic / haversine (trung vị) |
|---|---:|---:|---:|---:|
| Lô đo chính (tải 1 phút 74–89) | 27,35 | 5,89 | 1,17 | 54,7 |
| Chạy lại 1 (tải ≈ 200) | 50,26 | 6,95 | 3,08 | 8,8 |
| Chạy lại 2 (tải ≈ 192) | 35,41 | 6,30 | 4,63 | 23,6 |
| Chạy lại 3 (tải ≈ 194) | 30,85 | 5,91 | 4,29 | 37,4 |
| Lúc phát triển (máy ít tải hơn, không có `uptime`) | 17,55 | 5,80 | 2,86 | 21,2 |

Cùng một mã nguồn, số nhanh nhất của haversine đi từ 17,55 ns tới 50,26 ns; tỉ lệ Geodesic trên haversine nhảy từ 8,8 đến 54,7 lần. Điều đọc được một cách đáng tin: (1) equirect luôn nhanh hơn haversine, từ 1,17 đến 4,63 lần theo trung vị; (2) `Geodesic` chậm hơn haversine ít nhất hàng chục lần trong đa số lần chạy; (3) ở tốc độ này mỗi phép khoảng cách là vài chục nano giây, nên chi phí khoảng cách không phải chỗ cần tối ưu đầu tiên của service. Đừng ghi «haversine 17,55 ns» vào tài liệu như một số đo của công thức. Lab cũng có `cargo bench` dùng criterion cho số đo chặt hơn, nhưng cả hai lần chạy của nó cũng diễn ra khi máy tải nặng và khoảng tin cậy rất rộng.

## Chim bay khác đường đi: hệ số vòng

Mọi công thức trên đo đường thẳng trên mặt cầu. Xe không đi như vậy. Lab đo **hệ số vòng**, tức đường đi ngắn nhất theo đường chia khoảng cách chim bay, trên đồ thị OSM TP.HCM (126.699 nút, 319.074 cạnh có hướng; `--release`, seed 20261106): chọn nguồn ngẫu nhiên trong thành phần liên thông mạnh lớn nhất của từng profile, chạy Dijkstra đầy đủ, lấy tối đa 15 đích ngẫu nhiên mỗi dải.

`examples/toi-noi-location/rust/labs/src/bin/loc04_khoang_cach.rs`

```rust
        for _ in 0..sources {
            let s = members[rng.usize(members.len())];
            let dist = sp::distances_cm(&g, mode, s);
            let sll = g.node_latlng(s);
            let mut pool: Vec<Vec<(u32, f64)>> = vec![Vec::new(); bands.len()];
            for &t in &members {
                let d = dist[t as usize];
                if d == sp::INF || t == s {
                    continue;
                }
                let straight = equirect_m(sll, g.node_latlng(t));
                if let Some(b) = bands
                    .iter()
                    .position(|&(lo, hi)| straight >= lo && straight < hi)
                {
                    pool[b].push((t, f64::from(d) / 100.0));
                }
            }
            for (b, p) in pool.iter().enumerate() {
                for _ in 0..15.min(p.len()) {
                    let (t, road) = p[rng.usize(p.len())];
                    let straight = haversine_m(sll, g.node_latlng(t));
                    samples[b].push(road / straight);
                }
            }
        }
```

Kết quả (200 nút nguồn mỗi profile, 3.000 cặp mỗi ô):

| Profile | Dải chim bay | Trung bình | Trung vị | p90 | Lớn nhất | Từ 2,0 trở lên |
|---|---:|---:|---:|---:|---:|---:|
| Ô tô | 1–2 km | 1,840 | 1,456 | 2,481 | 13,581 | 17,7% |
| Ô tô | 2–5 km | 1,492 | 1,339 | 1,926 | 5,771 | 8,8% |
| Ô tô | 5–10 km | 1,291 | 1,253 | 1,488 | 2,794 | 0,7% |
| Xe máy | 1–2 km | 1,756 | 1,421 | 2,390 | 12,093 | 15,5% |
| Xe máy | 2–5 km | 1,438 | 1,311 | 1,786 | 6,172 | 6,9% |
| Xe máy | 5–10 km | 1,274 | 1,236 | 1,456 | 2,703 | 0,4% |
| Đi bộ | 1–2 km | 1,603 | 1,346 | 1,969 | 11,220 | 9,7% |
| Đi bộ | 2–5 km | 1,378 | 1,263 | 1,658 | 6,252 | 5,2% |
| Đi bộ | 5–10 km | 1,237 | 1,203 | 1,381 | 2,496 | 0,4% |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 350" role="img" aria-label="Hình 2. Hệ số vòng, tức đường đi ngắn nhất theo đường chia khoảng cách chim bay, trên đồ thị OSM TP.HCM của lab. Dải 1 đến 2 km: ô tô trung bình 1,840, xe máy 1,756, đi bộ 1,603. Dải 2 đến 5 km: 1,492, 1,438, 1,378. Dải 5 đến 10 km: 1,291, 1,274, 1,237. Vạch ngang trên mỗi cột là p90: 2,481, 2,390, 1,969; 1,926, 1,786, 1,658; 1,488, 1,456, 1,381."><text x="20" y="24" class="t-small">đường đi / chim bay</text><path d="M100 172 L840 172" class="f-none s-dead dash" stroke-width="1"/><text x="92" y="176" class="t-small" text-anchor="end">1,0 = chim bay</text><path d="M100 92 L840 92" class="f-none s-line" stroke-width="1"/><text x="92" y="96" class="t-small" text-anchor="end">2,0</text><path d="M100 252 L840 252" class="f-none s-muted" stroke-width="1"/><rect x="100" y="104.8" width="52" height="147.2" class="f-pk s-pk"/><path d="M97 53.5 L155 53.5" class="f-none s-muted" stroke-width="3"/><text x="126" y="270" class="t-mono" text-anchor="middle">1,840</text><text x="126" y="286" class="t-small" text-anchor="middle">2,481</text><rect x="164" y="111.5" width="52" height="140.5" class="f-phys s-phys"/><path d="M161 60.8 L219 60.8" class="f-none s-muted" stroke-width="3"/><text x="190" y="270" class="t-mono" text-anchor="middle">1,756</text><text x="190" y="286" class="t-small" text-anchor="middle">2,390</text><rect x="228" y="123.8" width="52" height="128.2" class="f-warn s-warn"/><path d="M225 94.5 L283 94.5" class="f-none s-muted" stroke-width="3"/><text x="254" y="270" class="t-mono" text-anchor="middle">1,603</text><text x="254" y="286" class="t-small" text-anchor="middle">1,969</text><text x="196" y="310" class="t-b" text-anchor="middle">chim bay 1–2 km</text><rect x="370" y="132.6" width="52" height="119.4" class="f-pk s-pk"/><path d="M367 97.9 L425 97.9" class="f-none s-muted" stroke-width="3"/><text x="396" y="270" class="t-mono" text-anchor="middle">1,492</text><text x="396" y="286" class="t-small" text-anchor="middle">1,926</text><rect x="434" y="137" width="52" height="115" class="f-phys s-phys"/><path d="M431 109.1 L489 109.1" class="f-none s-muted" stroke-width="3"/><text x="460" y="270" class="t-mono" text-anchor="middle">1,438</text><text x="460" y="286" class="t-small" text-anchor="middle">1,786</text><rect x="498" y="141.8" width="52" height="110.2" class="f-warn s-warn"/><path d="M495 119.4 L553 119.4" class="f-none s-muted" stroke-width="3"/><text x="524" y="270" class="t-mono" text-anchor="middle">1,378</text><text x="524" y="286" class="t-small" text-anchor="middle">1,658</text><text x="466" y="310" class="t-b" text-anchor="middle">chim bay 2–5 km</text><rect x="640" y="148.7" width="52" height="103.3" class="f-pk s-pk"/><path d="M637 133 L695 133" class="f-none s-muted" stroke-width="3"/><text x="666" y="270" class="t-mono" text-anchor="middle">1,291</text><text x="666" y="286" class="t-small" text-anchor="middle">1,488</text><rect x="704" y="150.1" width="52" height="101.9" class="f-phys s-phys"/><path d="M701 135.5 L759 135.5" class="f-none s-muted" stroke-width="3"/><text x="730" y="270" class="t-mono" text-anchor="middle">1,274</text><text x="730" y="286" class="t-small" text-anchor="middle">1,456</text><rect x="768" y="153" width="52" height="99" class="f-warn s-warn"/><path d="M765 141.5 L823 141.5" class="f-none s-muted" stroke-width="3"/><text x="794" y="270" class="t-mono" text-anchor="middle">1,237</text><text x="794" y="286" class="t-small" text-anchor="middle">1,381</text><text x="736" y="310" class="t-b" text-anchor="middle">chim bay 5–10 km</text><text x="92" y="270" class="t-small" text-anchor="end">trung bình</text><text x="92" y="286" class="t-small" text-anchor="end">p90</text><rect x="620" y="14" width="12" height="12" class="f-pk s-pk"/><text x="638" y="25" class="t-small">ô tô</text><rect x="705" y="14" width="12" height="12" class="f-phys s-phys"/><text x="723" y="25" class="t-small">xe máy</text><rect x="790" y="14" width="12" height="12" class="f-warn s-warn"/><text x="808" y="25" class="t-small">đi bộ</text><path d="M430 20 L454 20" class="f-none s-muted" stroke-width="3"/><text x="460" y="25" class="t-small">vạch ngang = p90</text></svg>
</div>
<figcaption>Hình 2. Hệ số vòng giảm khi khoảng cách tăng, nhưng ở dải ngắn có đuôi dài: p90 của ô tô ở dải 1–2 km là 2,481. Số của đồ thị OSM TP.HCM trong khung lab, bỏ qua cấm rẽ.</figcaption>
</figure>

Ba điều rút ra. **Một**, hệ số giảm khi khoảng cách tăng: ô tô trung bình 1,840 ở dải 1–2 km và 1,291 ở dải 5–10 km. Nên «chim bay nhân một hằng số» là phương án lùi tồi: hằng số đúng ở 7 km sai nhiều ở 1,5 km. **Hai**, ở dải ngắn đuôi rất dài: 17,7% số cặp ô tô trong dải 1–2 km có đường đi dài gấp hai lần chim bay trở lên, lớn nhất tới 13,6. Đó là loại cặp có thể làm tài xế «gần nhất» theo chim bay hoá ra xa hơn theo đường (xem bài mở đầu). **Ba**, đi bộ thấp nhất vì đi được hai chiều và có lối đi bộ, xe máy ở giữa, ô tô cao nhất.

Ba giới hạn bắt buộc khi đọc con số này. Phép đo **bỏ qua cấm rẽ** và cho quay đầu ở mọi nút, nên trên cùng đồ thị, đường đi hợp lệ chỉ có thể dài hơn. Đồ thị bị cắt ở biên bbox, nên một cặp gần biên có thể bị thổi cao vì lối tắt thật nằm ngoài khung. Và nó đo độ dài ngắn nhất, không phải thời gian nhanh nhất; tốc độ chưa được xét. Những con số này mô tả đồ thị OSM trong khung lab, không phải thành phố nói chung.

## Khi nào xấp xỉ là đủ

| Việc | Công cụ | Lý do |
|---|---|---|
| Lọc ứng viên theo bán kính (R-tree, H3) | Mặt phẳng mét cục bộ hoặc equirect | Sai 0,027% nhỏ hơn nhiễu GPS; nhanh |
| Hiển thị «cách bạn X m» ở đời thường | Haversine | Sai 0,5% so với ellipsoid không ai nhận ra |
| Ranh giới cần độ chính xác thật (geofence, đo đạc) | `Geodesic` | 0,5% là 52 m ở 10 km; chậm hơn hàng chục lần nhưng không nằm trên đường nóng |
| Cước, ETA, chọn tài xế | Quãng đường theo đồ thị đường bộ | Chim bay có thể sai gấp đôi (bảng trên); khoảng cách thẳng chỉ để lọc |
| Phương án lùi khi route engine lỗi | Chim bay nhân hệ số vòng **theo dải khoảng cách** | Hằng số chung sai ở dải ngắn |

Quy tắc ngắn: dùng chim bay để **lọc**, dùng đường đi để **quyết định**.

## Chạy ví dụ và kiểm chứng

Từ gốc repo (cần `data/hcm-graph.bin` do bài về OSM sinh ra; thêm `--fixture` để chạy trên đồ thị tổng hợp nhỏ, khi đó phần hệ số vòng ra con số khác):

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc04_khoang_cach
```

Cấu hình đo: `--release`, seed 20261106, 20.000 cặp mỗi ô sai số, 200 nút nguồn ngẫu nhiên mỗi profile, đồ thị HCM 126.699 nút. **Đo khi máy đang chịu tải nặng**: phần sai số và hệ số vòng không phụ thuộc thời gian nên không bị ảnh hưởng, phần tốc độ thì bị. Nguyên văn từ `results/p01.md`, mục tốc độ rồi hệ số vòng:

```text
-- 3. tốc độ (đồng hồ thô: 1.000.000 cặp quanh TP.HCM, cách nhau tối đa vài km, 5 lần chạy, --release) --
  haversine (geo-core)         nhanh nhất    27.35 ns/cặp   trung vị    50.22   chậm nhất   120.04
  equirectangular (geo-core)   nhanh nhất     5.89 ns/cặp   trung vị    42.82   chậm nhất    62.34
  geo::Haversine               nhanh nhất    73.33 ns/cặp   trung vị   102.35   chậm nhất   149.47
  geo::Geodesic (ellipsoid)    nhanh nhất  2024.41 ns/cặp   trung vị  2745.28   chậm nhất  3674.29
  tỉ lệ thời gian trung vị: haversine / equirect = 1.17, geodesic / haversine = 54.7, geodesic / equirect = 64.1
```

```text
-- 4. hệ số vòng = đường đi ngắn nhất theo đường / khoảng cách chim bay (haversine) --
đồ thị: examples/toi-noi-location/data/hcm-graph.bin: 126699 nút, 319074 cạnh
200 nút nguồn ngẫu nhiên trong thành phần liên thông mạnh lớn nhất của từng profile; mỗi nguồn lấy tối đa 15 đích mỗi dải; Dijkstra đầy đủ; bỏ qua cấm rẽ
profile    dải chim bay  số cặp trung bình trung vị      p90 lớn nhất     >= 2,0
ô tô      1000–2000  m    3000    1.840    1.456    2.481   13.581      17.7%
ô tô      2000–5000  m    3000    1.492    1.339    1.926    5.771       8.8%
ô tô      5000–10000 m    3000    1.291    1.253    1.488    2.794       0.7%
xe máy    1000–2000  m    3000    1.756    1.421    2.390   12.093      15.5%
xe máy    2000–5000  m    3000    1.438    1.311    1.786    6.172       6.9%
xe máy    5000–10000 m    3000    1.274    1.236    1.456    2.703       0.4%
đi bộ     1000–2000  m    3000    1.603    1.346    1.969   11.220       9.7%
đi bộ     2000–5000  m    3000    1.378    1.263    1.658    6.252       5.2%
đi bộ     5000–10000 m    3000    1.237    1.203    1.381    2.496       0.4%
```

## Giới hạn của ví dụ

Mọi sai số được đo với điểm do chính lab sinh ra bằng công thức mặt cầu, nên kiểm tra haversine và equirect khớp nhau là kiểm tra nhất quán chứ không phải kiểm tra với dữ liệu thật; so với ellipsoid thì có `geo::Geodesic` làm chuẩn. Sai số của mô hình cầu đo ở vĩ độ 10,77; ở vĩ độ khác sẽ khác (chưa đo trong bài này). Hệ số vòng bỏ qua cấm rẽ, tốc độ và thời gian, chỉ đúng với đồ thị OSM trong khung TP.HCM của lab. Số tốc độ đo khi máy tải nặng nên chỉ so sánh tương đối, và chưa có lần đo nào trên máy rảnh được lưu đầy đủ. Lab chưa tính VN-2000.

## Bài tập

1. Dựa vào bảng hệ số vòng, một tài xế ô tô cách ghim 1.500 m chim bay thì quãng đường xe đi có thể là bao nhiêu? Cho một khoảng hợp lý và nói vì sao không nên dùng một con số.

<details><summary>Lời giải</summary>

Ở dải 1–2 km của ô tô, trung vị 1,456 cho khoảng 2.184 m, trung bình 1,840 cho khoảng 2.760 m, p90 2,481 cho khoảng 3.722 m. Tức một khoảng rộng từ khoảng 1,5 km (hệ số gần 1) tới hơn 3,7 km ở p90, và 17,7% cặp dài gấp đôi chim bay trở lên. Một hằng số duy nhất che mất đuôi dài này, nên phương án lùi phải dùng theo dải khoảng cách và nên chuyển sang hiển thị khoảng ước lượng thay vì một con số.

</details>

2. Vì sao nhìn bảng sai số, ta có thể dùng equirect cho R-tree nhưng không nên coi nó là «chính xác hơn haversine»?

<details><summary>Lời giải</summary>

Hai công thức cùng dùng bán kính cầu, nên cùng sai 0,526% so với ellipsoid; equirect chỉ khác haversine 0,00029 m ở 10 km. Nó nhanh hơn vài lần nhưng không chính xác hơn. Chọn equirect cho việc lọc là chọn tốc độ mà không mất gì có thể thấy được, còn độ chính xác phụ thuộc mô hình cầu, không phụ thuộc công thức.

</details>

3. Thiết kế một phép đo để biết thứ tự hai tài xế theo chim bay bị đảo bao nhiêu lần so với thứ tự theo đường đi. Cần dữ liệu và chỉ số gì, và bài này đã có những phần nào của phép đo đó?

<details><summary>Lời giải</summary>

Cần đồ thị đường (đã có), một mô hình đội xe có seed (chưa có; là simulator ở bài LOC-30), và với mỗi chuyến lấy hai tài xế ứng viên, so thứ tự theo chim bay với thứ tự theo đường đi, đếm tỉ lệ đảo và km rỗng tăng thêm khi chọn theo chim bay. Bài này mới có hệ số vòng của từng cặp (điểm, điểm) độc lập, chưa có đội xe và chưa có tỉ lệ đảo. Vì vậy bài chỉ kết luận «đường đi dài gấp đôi chim bay không hiếm», không kết luận tần suất đảo thứ tự.

</details>

Bài trước: [bài 03](/articles/location-toa-do-wgs84-newtype-lat-lng-do-chinh-xac). Bài tiếp: [bài 05](/articles/location-geometry-crate-geo-chieu-diem-polyline-bearing).

## Tài liệu gốc

- [Haversine formula (Wikipedia)](https://en.wikipedia.org/wiki/Haversine_formula): dẫn xuất và độ ổn định số học.
- [Movable Type Scripts: Calculate distance, bearing and more between latitude/longitude points](https://www.movable-type.co.uk/scripts/latlong.html): haversine, equirectangular và bearing với ví dụ.
- [GeographicLib](https://geographiclib.sourceforge.io/): thuật toán trắc địa trên ellipsoid của Karney, nền của `geo::Geodesic`.
- [docs.rs: crate geo](https://docs.rs/geo/latest/geo/): `Haversine`, `Geodesic`, `Distance`.
- [docs.rs: crate criterion](https://docs.rs/criterion/latest/criterion/): bench thống kê.
- `examples/toi-noi-location/results/p01.md`, mục LOC-04.
