---
slug: location-geometry-crate-geo-chieu-diem-polyline-bearing
title: "Hình học với crate geo: chiếu điểm, polyline, bearing và GeoJSON"
summary: "Bài 05: chiếu điểm GPS lên đoạn và polyline, vị trí dọc tuyến, bearing, hệ độ so với hệ mét trong crate geo, mã hóa polyline và xuất GeoJSON để xem kết quả trên bản đồ như một thói quen debug."
category: system-design
level: intermediate
tags: [toi-noi, location, rust, geo, polyline, bearing, geojson, projection]
language: vi
---

Tài xế DR-001 gửi một điểm GPS nằm lệch tuyến đường 25 m. Hai thứ cần biết ngay: chỗ nào trên đường là chỗ xe đang ở, và xe đã đi được bao xa dọc tuyến, để app khách vẽ marker chạy mượt trên đường thay vì nhảy sang vỉa hè. Cả hai đều là một phép **chiếu**: kéo điểm GPS vuông góc xuống đường, rồi đo quãng đường từ đầu tuyến tới chân vừa tìm ra. Thêm một thứ nữa là **hướng**: xe đang quay mặt về đâu, thứ mà bài chọn phía đường sau này cần để biết khách đứng bên phải hay bên trái xe.

Bài này làm bốn việc hình học nhỏ trên một đường gấp khúc (polyline): chiếu điểm lên đoạn và lên cả polyline, tính bearing, mã hóa polyline để gửi gọn qua mạng, và xuất GeoJSON để nhìn kết quả trên bản đồ. Đây là bài 05 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi), đọc sau bài [khoảng cách và sai số](/articles/location-khoang-cach-haversine-equirectangular-sai-so). Bài dùng crate `geo-core` của lab và so sánh với crate `geo` ở một chỗ mà chúng cho kết quả khác nhau.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Crate <code>geo</code> có sẵn hàm tìm điểm gần nhất trên đường gấp khúc (<code>ClosestPoint</code>). Nếu đưa vào tọa độ lat/lng trực tiếp (coi độ như x, y phẳng), điểm nó trả về sai bao nhiêu mét so với chân đường vuông góc đo theo mét, ở TP.HCM và ở Hà Nội?</p>
<details><summary>Xem đáp án</summary><p>Lab đo 2.000 đoạn 200 m hướng ngẫu nhiên với điểm GPS cách trung điểm 30 m: ở TP.HCM chân lệch <b>trung bình 0,212 m, lớn nhất 0,533 m</b>; ở Hà Nội <b>trung bình 0,821 m, lớn nhất 2,069 m</b>. Lệch tăng theo vĩ độ vì 1° kinh độ chỉ dài bằng 1° vĩ độ nhân cos của vĩ độ. Chân lệch cỡ này làm sai <code>along_m</code> và marker dọc tuyến ngay cả khi vẫn chọn đúng đường.</p></details>
</div>

## Chiếu điểm lên một đoạn thẳng

Cho đoạn AB và điểm Q. Gọi `d = B − A`. Điểm trên đường thẳng AB gần Q nhất là `A + t·d` với

```text
t = (Q − A)·d / |d|²
```

`t` là tỉ lệ vị trí: 0 ở A, 1 ở B. Nhưng đoạn thẳng không dài vô hạn: nếu `t` ra âm hoặc lớn hơn 1, chân vuông góc nằm ngoài đoạn và điểm gần nhất thực sự là đầu mút. Vì vậy `t` phải bị **kẹp** vào [0, 1]. Hình 1 vẽ ba trường hợp.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 1. Chiếu một điểm Q lên đoạn thẳng AB. Điểm Q1 chiếu xuống giữa đoạn với t khoảng 0,44. Điểm Q2 chiếu ra ngoài đầu A với t âm nên chân bị kẹp về A. Điểm Q3 chiếu ra ngoài đầu B với t lớn hơn 1 nên chân bị kẹp về B."><path d="M120 230 L420 150" class="f-none s-phys" stroke-width="4"/><circle cx="120" cy="230" r="5" class="f-phys s-phys"/><circle cx="420" cy="150" r="5" class="f-phys s-phys"/><text x="112" y="252" class="t-b" text-anchor="end">A</text><text x="430" y="170" class="t-b">B</text><circle cx="230" cy="110" r="5" class="f-pk s-pk"/><path d="M230 110 L252.6 194.6" class="f-none s-pk" stroke-width="2"/><circle cx="252.6" cy="194.6" r="4" class="f-warn s-warn"/><circle cx="70" cy="170" r="5" class="f-pk s-pk"/><path d="M70 170 L88.3 238.5" class="f-none s-dead dash" stroke-width="1"/><circle cx="88.3" cy="238.5" r="4" class="f-bg s-dead"/><path d="M70 170 L120 230" class="f-none s-pk" stroke-width="2"/><circle cx="120" cy="230" r="4" class="f-warn s-warn"/><circle cx="480" cy="80" r="5" class="f-pk s-pk"/><path d="M480 80 L493.4 130.4" class="f-none s-dead dash" stroke-width="1"/><circle cx="493.4" cy="130.4" r="4" class="f-bg s-dead"/><path d="M480 80 L420 150" class="f-none s-pk" stroke-width="2"/><circle cx="420" cy="150" r="4" class="f-warn s-warn"/><text x="240" y="96" class="t-b">Q1</text><text x="250" y="112" class="t-small">t = 0,44, chân trong đoạn</text><text x="82" y="166" class="t-b">Q2</text><text x="24" y="142" class="t-small">t = −0,11, kẹp về A</text><text x="490" y="78" class="t-b">Q3</text><text x="380" y="46" class="t-small">t = 1,24, kẹp về B</text><text x="20" y="312" class="t-small">Chấm xanh: Q. Chấm cam: chân sau khi kẹp. Vòng rỗng: chân vuông góc nằm ngoài đoạn.</text><text x="560" y="60" class="t-b">Trong project_to_segment</text><text x="560" y="90" class="t-mono">d = B − A</text><text x="560" y="114" class="t-mono">t = clamp((Q − A)·d / |d|², 0, 1)</text><text x="560" y="138" class="t-mono">chân = A + t·d</text><text x="560" y="162" class="t-mono">dist_m = |Q − chân|</text><text x="560" y="206" class="t-small">Gốc tọa độ đặt tại Q (Q = (0, 0)), đơn vị mét,</text><text x="560" y="222" class="t-small">nên sai số do Trái Đất cong là không đáng kể</text><text x="560" y="238" class="t-small">ở độ dài đoạn đường phố.</text></svg>
</div>
<figcaption>Hình 1. Chiếu điểm lên đoạn thẳng. Hình minh họa cơ chế, tọa độ vẽ tay, các giá trị t tính từ chính tọa độ vẽ. Chân vuông góc của Q2 và Q3 nằm ngoài đoạn nên kết quả bị kẹp về đầu mút gần nhất, và khoảng cách báo về là khoảng cách tới đầu mút, không phải tới đường kéo dài.</figcaption>
</figure>

Q2 và Q3 là chỗ người mới hay làm sai: họ chiếu lên đường thẳng rồi quên rằng đoạn có hai đầu. Hệ quả trên bản đồ: một điểm GPS ở ngã tư có thể «dính» vào một đoạn đường chỉ vì đường thẳng kéo dài của đoạn đó đi qua sát điểm. Khoảng cách `dist_m` mà hàm trả về là khoảng cách tới điểm đã kẹp, không phải tới đường kéo dài.

Code của lab đặt gốc mặt phẳng mét **tại chính điểm Q**, nên `Q = (0, 0)` và công thức gọn lại:

`examples/toi-noi-location/rust/crates/geo-core/src/segment.rs`

```rust
/// Chiếu `p` lên đoạn `a`–`b`. Làm việc trong mặt phẳng mét gốc tại `p` nên sai số do cong Trái Đất
/// là không đáng kể ở độ dài đoạn đường phố.
pub fn project_to_segment(p: LatLng, a: LatLng, b: LatLng) -> SegmentProjection {
    let proj = LocalProjection::new(p);
    let (a2, b2) = (proj.to_xy(a), proj.to_xy(b));
    let d = [b2[0] - a2[0], b2[1] - a2[1]];
    let len2 = d[0] * d[0] + d[1] * d[1];
    let t = if len2 == 0.0 {
        0.0
    } else {
        (-(a2[0] * d[0] + a2[1] * d[1]) / len2).clamp(0.0, 1.0)
    };
    let foot = [a2[0] + t * d[0], a2[1] + t * d[1]];
    SegmentProjection {
        point: proj.to_latlng(foot),
        t,
        dist_m: foot[0].hypot(foot[1]),
    }
}
```

Chú ý `LocalProjection::new(p)`: gốc ở điểm hỏi nên khoảng cách quanh điểm hỏi gần như chính xác, và mặt phẳng này đã được đo ở bài trước (sai tương đối khoảng 0,027% trong bbox TP.HCM khi gốc ở tâm bbox, nhỏ hơn nữa khi gốc ở sát điểm hỏi). `dist_m` ở dòng cuối tính bằng `hypot` của chân trong mặt phẳng mét đó.

## Chiếu lên polyline, và `along_m`

Đường đi là chuỗi đoạn nối nhau. Chiếu lên polyline là chiếu lên từng đoạn rồi lấy đoạn có `dist_m` nhỏ nhất. Thông tin thứ hai, `along_m`, là quãng đường từ đầu polyline tới chân, đo **dọc tuyến**: tổng độ dài các đoạn trước đoạn trúng, cộng `t` nhân độ dài đoạn trúng.

`examples/toi-noi-location/rust/crates/geo-core/src/segment.rs`

```rust
/// Chiếu `p` lên polyline: chọn đoạn gần nhất rồi tính `along_m`. `None` nếu polyline rỗng.
pub fn project_to_polyline(p: LatLng, points: &[LatLng]) -> Option<PolylineProjection> {
    match points {
        [] => None,
        [only] => Some(PolylineProjection {
            segment: 0,
            t: 0.0,
            point: *only,
            dist_m: haversine_m(p, *only),
            along_m: 0.0,
        }),
        _ => {
            let mut best: Option<(usize, SegmentProjection)> = None;
            for (i, w) in points.windows(2).enumerate() {
                let sp = project_to_segment(p, w[0], w[1]);
                if best.as_ref().is_none_or(|(_, b)| sp.dist_m < b.dist_m) {
                    best = Some((i, sp));
                }
            }
            let (i, sp) = best?;
            let before: f64 = points[..=i]
                .windows(2)
                .map(|w| haversine_m(w[0], w[1]))
                .sum();
            let seg_len = haversine_m(points[i], points[i + 1]);
            Some(PolylineProjection {
                segment: i,
                t: sp.t,
                point: sp.point,
                dist_m: sp.dist_m,
                along_m: before + sp.t * seg_len,
            })
        }
    }
}
```

`along_m` là đại lượng mà app khách cần để đặt marker dọc tuyến: nội suy theo `along_m` thay vì tọa độ thô thì marker chạy trên đường. Hợp đồng theo dõi chuyến của lab cũng có trường `route_offset_m` đúng với ý này: vị trí đã bám đường, tính bằng mét từ đầu route (lab hiện để trống, vì bám đường là việc của phần sau). Hàm ngược của nó, `point_at`, cho điểm cách đầu tuyến `along_m` mét.

Lab thử trên một polyline minh họa bốn đỉnh nối ba địa danh ở trung tâm TP.HCM bằng tọa độ gần đúng; đây **không phải tuyến đường thật**. Lab dựng điểm GPS cách đoạn 1 đúng 25 m theo hướng vuông góc, tại 40% đoạn, rồi kiểm tra hàm tìm lại đúng vị trí đó (nguyên văn từ `results/p01.md`):

```text
-- 2. chiếu một điểm GPS lên polyline --
  điểm trên đoạn 1 tại 40%: 10.778640,106.697600
  điểm GPS dựng cách đó 25 m vuông góc: 10.778812,106.697452
  kết quả: đoạn 1, t = 0.4000, dist = 25.001 m, along = 1019.101 m
  kỳ vọng:  đoạn 1, t = 0.4000, dist = 25.000 m, along = 1019.101 m
  chân đường vuông góc: 10.778640,106.697600
```

Đoạn 1 dài 500,15 m và đoạn 0 dài 819,04 m, nên vị trí dọc tuyến kỳ vọng là 819,04 + 0,4 × 500,15 = 1.019,10 m, khớp với `along = 1019.101`. `dist` ra 25,001 m thay vì 25,000 m: chênh một mili mét do chiếu phẳng cục bộ. Hình 2 vẽ lại kết quả này từ file GeoJSON mà lab đã xuất.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 470" role="img" aria-label="Hình 2. Bên trái: đường gấp khúc minh họa bốn đỉnh P0 đến P3 vẽ từ results/geojson/loc05-hinh-hoc.geojson, hướng bắc ở trên, tỉ lệ 0,4 pixel mỗi mét, với ba đoạn dài 819,04, 500,15 và 888,48 mét. Bên phải: phóng to quanh điểm GPS, cách đoạn 1 đúng 25,001 mét, chân đường vuông góc nằm cách đầu polyline 1.019,101 mét dọc tuyến."><defs><marker id="location-geometry-crate-geo-chieu-diem-polyline-bearing-1-m0" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs><path d="M165.2 370.7 L208.9 46 L56 175 L305.1 428.5" class="f-none s-phys" stroke-width="3"/><circle cx="165.2" cy="370.7" r="5" class="f-phys s-phys"/><text x="175.2" y="386.7" class="t-b">P0</text><circle cx="208.9" cy="46" r="5" class="f-phys s-phys"/><text x="218.9" y="38" class="t-b">P1</text><circle cx="56" cy="175" r="5" class="f-phys s-phys"/><text x="46" y="167" class="t-b" text-anchor="end">P2</text><circle cx="305.1" cy="428.5" r="5" class="f-phys s-phys"/><text x="315.1" y="444.5" class="t-b">P3</text><text x="190" y="296" class="t-small">đoạn 0: 819,04 m</text><text x="118" y="100" class="t-small" text-anchor="end">đoạn 1: 500,15 m</text><text x="240" y="346" class="t-small">đoạn 2: 888,48 m</text><circle cx="147.8" cy="97.6" r="4" class="f-pk s-pk"/><rect x="135.8" y="85.6" width="24" height="24" class="f-none s-warn" stroke-width="1"/><rect x="430" y="36" width="432" height="404" rx="8" class="f-surface s-line"/><path d="M159.8 85.6 L430 36" class="f-none s-warn dash" stroke-width="1"/><path d="M159.8 109.6 L430 440" class="f-none s-warn dash" stroke-width="1"/><path d="M810.5 114.6 L489.5 385.4" class="f-none s-phys" stroke-width="4"/><path d="M537.2 116.2 L650 250" class="f-none s-pk dash" stroke-width="2"/><circle cx="537.2" cy="116.2" r="6" class="f-pk s-pk"/><circle cx="650" cy="250" r="5" class="f-warn s-warn"/><text x="549.2" y="112.2" class="t-b">điểm GPS</text><text x="585.6" y="183.1" class="t-small" text-anchor="end">dist = 25,001 m</text><text x="662" y="272" class="t-small">chân vuông góc</text><text x="662" y="288" class="t-small">along = 1.019,101 m</text><text x="444" y="60" class="t-b">Phóng to quanh điểm GPS (7 px mỗi mét)</text><text x="848" y="384" class="t-small" text-anchor="end">đoạn 1 (P1 → P2), bearing 229,85°</text><path d="M470 416 L540 416" class="f-none s-muted" stroke-width="2"/><text x="548" y="420" class="t-small">10 m</text><path d="M56 446 L96 446" class="f-none s-muted" stroke-width="2"/><text x="104" y="450" class="t-small">100 m</text><path d="M380 440 L380 410" class="f-none s-muted" stroke-width="2" marker-end="url(#location-geometry-crate-geo-chieu-diem-polyline-bearing-1-m0)"/><text x="380" y="404" class="t-small" text-anchor="middle">bắc</text></svg>
</div>
<figcaption>Hình 2. Kết quả thật của lab, vẽ từ <code>results/geojson/loc05-hinh-hoc.geojson</code>. Đường gấp khúc là tọa độ gần đúng của ba địa danh ở trung tâm TP.HCM để minh họa, <b>không phải tuyến đường thật</b>. Điểm GPS được dựng cách đoạn 1 đúng 25 m vuông góc, tại 40% đoạn. Khung cam bên trái là vùng được phóng to ở khung bên phải.</figcaption>
</figure>

## Bearing: xe đang quay mặt về đâu

Bearing là hướng từ điểm này tới điểm kia, đo bằng độ, 0 là bắc, quay theo kim đồng hồ (90 là đông, 180 là nam, 270 là tây):

`examples/toi-noi-location/rust/crates/geo-core/src/bearing.rs`

```rust
/// Hướng ban đầu từ `a` tới `b`, độ trong [0, 360): 0 là bắc, 90 là đông.
pub fn bearing_deg(a: LatLng, b: LatLng) -> f64 {
    let (p1, p2) = (a.lat().to_radians(), b.lat().to_radians());
    let dlam = (b.lng() - a.lng()).to_radians();
    let y = dlam.sin() * p2.cos();
    let x = p1.cos() * p2.sin() - p1.sin() * p2.cos() * dlam.cos();
    (y.atan2(x).to_degrees() + 360.0).rem_euclid(360.0)
}

/// Chênh lệch nhỏ nhất giữa hai hướng, độ trong [0, 180]. 350 và 10 chênh nhau 20, không phải 340.
pub fn bearing_diff_deg(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}
```

Điểm cần cẩn thận không nằm ở công thức mà ở chỗ **góc là đại lượng vòng tròn**: 350° và 10° chỉ cách nhau 20°, không phải 340°. Hàm `bearing_diff_deg` xử lý chuyện đó và luôn trả giá trị trong [0, 180]. Lab xác nhận `bearing_diff_deg(350, 10) = 20` và `bearing_diff_deg(90, 270) = 180`. Ba đoạn của polyline minh họa có bearing như sau:

| Đoạn | Dài (m) | Bearing | Hướng |
|---:|---:|---:|---|
| 0 | 819,04 | 7,66° | bắc |
| 1 | 500,15 | 229,85° | tây nam |
| 2 | 888,48 | 135,51° | đông nam |

Ở các bài sau bearing có hai việc. Thứ nhất, so hướng của tài xế (từ hai điểm GPS liên tiếp) với bearing của từng cạnh đường để loại cạnh đi ngược chiều: bài bám đường dùng cách đó, và `bearing_diff_deg` là thước đo. Thứ hai, quyết định điểm đón nằm bên nào của xe: cùng một tọa độ, xe đi hướng này thì khách ở lề phải, xe đi hướng ngược lại thì phải quay đầu. Phần đó là việc của các bài điểm đón ở P04, bài này chưa làm.

## Hệ độ hay hệ mét: chỗ crate `geo` cho kết quả khác

Crate `geo` rất đầy đủ, nhưng nhiều thuật toán hình học của nó làm việc trong **mặt phẳng của dữ liệu đầu vào**, tức coi `x` và `y` là hai trục cùng đơn vị. Nếu bạn đưa vào lng làm `x` và lat làm `y` thì phép «gần nhất» đo trong mặt phẳng độ, nơi 1° kinh độ ngắn hơn 1° vĩ độ ở mọi nơi trừ xích đạo. Lab đo độ lệch bằng cách dựng đoạn 200 m hướng ngẫu nhiên, đặt điểm GPS cách trung điểm 30 m, rồi so chân do `geo::ClosestPoint` trả với chân của `project_to_polyline`:

`examples/toi-noi-location/rust/labs/src/bin/loc05_hinh_hoc.rs`

```rust
            let mid = destination(a, bearing_deg(a, b), 100.0);
            let q = destination(mid, rng.range(0.0, 360.0), 30.0);
            let ours = project_to_polyline(q, &[a, b]).expect("hợp lệ").point;
            let line = LineString::new(vec![
                Coord {
                    x: a.lng(),
                    y: a.lat(),
                },
                Coord {
                    x: b.lng(),
                    y: b.lat(),
                },
            ]);
            let theirs = match line.closest_point(&Point::new(q.lng(), q.lat())) {
                Closest::SinglePoint(p) | Closest::Intersection(p) => ll(p.y(), p.x()),
                Closest::Indeterminate => continue,
            };
            let off = haversine_m(ours, theirs);
```

| Thành phố | Số đoạn thử | Lệch trung bình | Lệch lớn nhất |
|---|---:|---:|---:|
| TP.HCM | 2.000 | 0,212 m | 0,533 m |
| Hà Nội | 2.000 | 0,821 m | 2,069 m |

Chân đường vuông góc ở hệ độ lệch dưới một mét ở TP.HCM, hơn hai mét ở Hà Nội. Lệch này chủ yếu nằm dọc theo đoạn, nên nó làm sai `along_m` nhiều hơn là làm sai khoảng cách tới đường. Nhưng nếu xếp hạng ứng viên bằng khoảng cách trong hệ độ thì lỗi còn đi xa hơn: cùng 30 m, hướng đông tây được tính dài hơn hướng bắc nam 1,8% ở TP.HCM và 7% ở Hà Nội (1 / cos của vĩ độ), đủ để đổi thứ tự hai cạnh gần như cách đều. Lab không đo số lần đổi thứ tự. Quy tắc của lab: **chiếu sang mét trước, hình học sau**.

## Mã hóa polyline: gửi gọn một tuyến

Một tuyến 442 điểm viết bằng JSON `[lng, lat]` tốn hàng chục nghìn byte. Định dạng *Encoded Polyline* của Google nén bằng ba ý: nhân tọa độ với `10^precision` làm tròn thành số nguyên, lưu **hiệu** so với điểm trước (số nhỏ), rồi mã hóa hiệu kèm bit dấu thành các khối 5 bit, mỗi khối một ký tự in được.

`examples/toi-noi-location/rust/crates/geo-core/src/polyline.rs`

```rust
/// Mã hóa danh sách điểm. `precision` thường là 5 hoặc 6.
pub fn encode(points: &[LatLng], precision: u32) -> String {
    let factor = 10f64.powi(precision as i32);
    let mut out = String::new();
    let (mut prev_lat, mut prev_lng) = (0i64, 0i64);
    for p in points {
        let lat = (p.lat() * factor).round() as i64;
        let lng = (p.lng() * factor).round() as i64;
        encode_value(lat - prev_lat, &mut out);
        encode_value(lng - prev_lng, &mut out);
        prev_lat = lat;
        prev_lng = lng;
    }
    out
}
```

`precision` thường là 5 (khoảng 1 m, chuẩn của Google) hoặc 6 (khoảng 0,1 m). Lab dày hóa tuyến minh họa thành 442 điểm, mỗi khoảng 5 m, rồi thêm nhiễu 0–1 m (seed 20261106) và đo vòng mã hóa rồi giải mã:

| Cách lưu 442 điểm | Kích thước | Sai số lớn nhất sau vòng đi về |
|---|---:|---:|
| Chuỗi polyline precision 5 | 892 byte | 0,7577 m |
| Chuỗi polyline precision 6 | 1.614 byte | 0,0749 m |
| Nhị phân E7 (2×`i32`) | 3.536 byte | 0,0078 m (theo bài 03) |
| Nhị phân `f64` thô | 7.072 byte | 0 |
| JSON `[lng, lat]` | khoảng 17.511 byte | 0 |

Chuỗi precision 5 nhỏ hơn JSON khoảng 20 lần, nhưng cái giá là sai số gần 0,76 m. Với việc **vẽ tuyến** trên app thì quá thừa; với việc **tính toán** (chiếu, bám đường) thì dùng dữ liệu gốc, đừng dùng bản đã mã hóa. Hàm của lab được kiểm bằng ví dụ có trong tài liệu của Google và so khớp từng ký tự với crate `polyline`.

## GeoJSON: nhìn thấy mới tin

Các lab hình học của loạt đều xuất GeoJSON. Đó là thói quen debug đáng giữ nhất của cả phần: lỗi tọa độ hiếm khi lộ ra ở con số nhưng lộ ngay ở hình. Hàm xuất cũng đi qua `to_lnglat` của bài 03, nên thứ tự `[lng, lat]` chỉ được tạo ra ở một chỗ:

`examples/toi-noi-location/rust/crates/geo-core/src/geojson.rs`

```rust
/// Feature kiểu LineString.
pub fn line_feature(points: &[LatLng], properties: &[(&str, JsonValue)]) -> Feature {
    Feature {
        bbox: None,
        geometry: Some(Geometry::new_line_string(
            points.iter().map(|p| p.to_lnglat()),
        )),
        id: None,
        properties: props(properties),
        foreign_members: None,
    }
}
```

Lab ghi `results/geojson/loc05-hinh-hoc.geojson` gồm bốn feature: polyline minh họa, đoạn chiếu (kèm `do_dai_m`), điểm GPS và chân vuông góc (kèm `along_m`). Mở file bằng geojson.io hoặc QGIS là thấy ngay đoạn chiếu thẳng góc với đường. Vì vậy các lab sau (snap, ô H3) cũng xuất GeoJSON vào `results/geojson/`.

## Chạy ví dụ và kiểm chứng

Từ gốc repo (không cần PBF, Redis hay mạng; lab ghi file GeoJSON ở trên):

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc05_hinh_hoc
```

Cấu hình: `--release`, seed 20261106, Apple M1 Max. Mọi con số trong bài là đại lượng hình học, độ dài và sai số, không phụ thuộc thời gian, nên không bị ảnh hưởng khi máy chịu tải nặng lúc chạy lô đo. Output, nguyên văn từ `results/p01.md`:

```text
-- 1. độ dài và bearing từng đoạn --
  đoạn 0: dài   819.04 m, bearing   7.66° (B)
  đoạn 1: dài   500.15 m, bearing 229.85° (TN)
  đoạn 2: dài   888.48 m, bearing 135.51° (ĐN)
  tổng độ dài polyline: 2207.67 m
  bearing_diff_deg(350, 10) = 20, bearing_diff_deg(90, 270) = 180
```

```text
-- 4. mã hóa polyline (Google Encoded Polyline) --
  tuyến dày: 442 điểm, mỗi ~5 m, nhiễu 0–1 m (seed 20261106)
  ví dụ nhỏ (4 đỉnh): precision 5 = "c_w`AomfjSsl@gEbQzTrb@sb@"
                       precision 6 = "g`opS_piojEggMo}@ftDvyEfcJgcJ"
  precision 5:    892 byte chuỗi, sai số lớn nhất sau vòng mã hóa/giải mã 0.7577 m
  precision 6:   1614 byte chuỗi, sai số lớn nhất sau vòng mã hóa/giải mã 0.0749 m
  so sánh kích thước: JSON [lng,lat] f64 ≈ 17511 byte, nhị phân E7 = 3536 byte, f64 thô = 7072 byte
```

Kiểm tra toàn phần `bash examples/toi-noi-location/verify-p01.sh` chạy các test của `geo-core` (chiếu, bearing, polyline theo ví dụ của Google, GeoJSON) và chạy lại lab; kết quả ghi `PASS`.

## Giới hạn của ví dụ

Đường gấp khúc minh họa dùng tọa độ gần đúng của ba địa danh, không phải tuyến đường thật, và không có dữ liệu đường OSM trong bài này. Phép chiếu chọn đoạn gần nhất rồi kết thúc: nếu tuyến tự cắt hoặc vòng lại (cầu vượt, vòng xoay), `along_m` của một điểm nằm giữa hai nhánh gần nhau có thể sai nhánh. Việc chọn đúng nhánh là bài toán bám đường (map matching) của phần sau. Bearing mới có hướng ban đầu của cung tròn lớn giữa hai điểm; chưa có quyết định trái hay phải. Lab chưa nội suy marker cho app khách và chưa tính VN-2000. Con số nhiễu 0–1 m dùng cho tuyến dày là nhiễu tổng hợp của lab, không phải số đo từ GPS thật.

## Bài tập

1. Điểm Q ở ngay ngoài đầu mút B của đoạn AB, cách B 10 m dọc theo hướng kéo dài của AB. `t` trả về bao nhiêu và `dist_m` là bao nhiêu? Nếu bạn quên kẹp `t` thì kết quả khác thế nào?

<details><summary>Lời giải</summary>

Chân vuông góc của Q trên đường thẳng AB trùng chính Q, nên `t` thô lớn hơn 1 và `dist_m` thô bằng 0. Sau khi kẹp, `t = 1`, chân là B, `dist_m = 10`. Quên kẹp thì hàm báo Q nằm **trên** đoạn (`dist_m = 0`), điều sai: Q cách đoạn 10 m.

</details>

2. Một đoạn đường chạy đúng hướng tây có bearing bao nhiêu? Một xe đi hướng 350° trên cạnh có bearing 10° thì hai hướng chênh nhau bao nhiêu độ, và vì sao không tính bằng phép trừ thường?

<details><summary>Lời giải</summary>

Bearing hướng tây là 270°. Hai hướng 350° và 10° chênh nhau 20°, không phải 340°, vì góc quay vòng 360°. `bearing_diff_deg` lấy phần dư `(a − b) mod 360` rồi chọn giá trị nhỏ hơn giữa nó và `360 − nó`. Phép trừ thường cho 340°, làm một xe đang đi gần như đúng hướng bị xem là đi ngược chiều.

</details>

3. Nếu bạn dùng polyline precision 5 để vẽ marker xe trên app khách, sai số lớn nhất gần 0,76 m có chấp nhận được không? Còn nếu dùng nó làm đầu vào cho bám đường thì sao?

<details><summary>Lời giải</summary>

Để vẽ marker trên màn hình điện thoại thì chấp nhận được, vì nhiễu GPS vài mét lớn hơn nhiều và 0,76 m chỉ bằng vài điểm ảnh ở mức zoom đường phố. Làm đầu vào cho bám đường thì không nên: sai số cộng thêm vào nhiễu GPS, làm xấu việc xếp hạng đường song song. Quy tắc: tính toán dùng dữ liệu gốc (`f64` hoặc E7), chỉ mã hóa polyline ở khâu gửi cho giao diện.

</details>

Bài trước: [bài 04](/articles/location-khoang-cach-haversine-equirectangular-sai-so). Bài tiếp: [bài 06](/articles/location-osm-pbf-do-thi-duong-bo-xe-may-o-to).

## Tài liệu gốc

- [Encoded Polyline Algorithm Format (Google)](https://developers.google.com/maps/documentation/utilities/polylinealgorithm): định dạng và ví dụ chuẩn.
- [RFC 7946: The GeoJSON Format](https://datatracker.ietf.org/doc/html/rfc7946).
- [geojson.io](https://geojson.io/) và [QGIS](https://qgis.org/): xem file GeoJSON do lab xuất.
- [docs.rs: crate geo](https://docs.rs/geo/latest/geo/): có `ClosestPoint`.
- [docs.rs: crate polyline](https://docs.rs/polyline/latest/polyline/).
- `examples/toi-noi-location/results/p01.md`, mục LOC-05; `examples/toi-noi-location/results/geojson/loc05-hinh-hoc.geojson`.
