---
slug: location-rstar-r-tree-snap-diem-ve-duong
title: "R-tree với rstar: snap điểm GPS về đúng cạnh đường"
summary: "Bài 07: dựng R-tree các đoạn đường bằng rstar, snap điểm GPS về cạnh gần nhất, chọn ứng viên theo bán kính, đo tỉ lệ snap sai sang đường song song trên đồ thị HCM và so tốc độ với quét toàn bộ."
category: system-design
level: intermediate
tags: [toi-noi, location, rust, rstar, r-tree, snap, map-matching, openstreetmap]
language: vi
---

Lấy một điểm trên đường Song Hành Xa Lộ Hà Nội, rồi cộng nhiễu GPS mô phỏng: điểm đo lệch 12,0 m so với chỗ đứng thật. Engine hỏi «đường nào gần điểm này nhất?» và nhận về **Xa lộ Hà Nội**, đường chính cách điểm đo chỉ 6,9 m, trong khi đường Song Hành thật cách 11,4 m. Hai đường chạy song song, cùng hướng, cách nhau 17,9 m. Snap sai một lần như vậy là điểm đón nằm sai cả một con đường, và tài xế có thể phải đi một vòng mới sang được đường kia.

Đây là bài 07 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng đồ thị CSR của [bài 06](/articles/location-osm-pbf-do-thi-duong-bo-xe-may-o-to) và các phép chiếu điểm lên đoạn của `geo-core` (bài 05). Mọi số đo ở đây là **trên dữ liệu mô phỏng**: vị trí thật là điểm ngẫu nhiên trên cạnh có thật của OSM, còn nhiễu GPS do lab sinh, không phải GPS người thật.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Lấy một điểm ngẫu nhiên trên đường thật của đồ thị HCM, cộng nhiễu Gaussian σ = 10 m mỗi trục, rồi chọn cạnh gần điểm đo nhất. Theo bạn, bao nhiêu phần trăm số lần cạnh đó thuộc đúng way thật?</p>
<details><summary>Xem đáp án</summary><p>Chỉ <b>73,39%</b>: cạnh gần nhất thuộc sai way ở <b>26,61%</b> số điểm (20.000 điểm, σ = 10 m). Thêm hướng đi vào điểm số kéo tỉ lệ sai xuống 10,97%. Đường song song cùng hướng là ca mà cả hai cách đều thua; phần dưới có ví dụ đo được.</p></details>
</div>

## Vì sao cần chỉ mục: 306.571 đoạn thẳng và mỗi điểm GPS một câu hỏi

Đồ thị HCM có 319.074 cạnh có hướng. Mỗi cạnh là một polyline (nút đầu, các điểm hình dạng, nút cuối), nên để snap chính xác phải xét từng **đoạn thẳng** giữa hai điểm liên tiếp. Cặp hai chiều của một đoạn đường (`twin`) chỉ lập chỉ mục một lần, và kết quả mang theo cả cạnh `twin` để người gọi chọn chiều theo hướng đi; nhờ vậy tổng cộng có **306.571 đoạn thẳng** trong chỉ mục. Quét hết từng đoạn cho mỗi điểm GPS là tuyến tính theo số đoạn.

**R-tree** tổ chức các đoạn theo hộp bao nhỏ nhất (AABB) lồng nhau: mỗi nút cha giữ hộp bao của các con. `rstar` yêu cầu hai thứ: `RTreeObject` cho biết hộp bao của phần tử, và `PointDistance` cho biết bình phương khoảng cách từ một điểm tới phần tử. Với đoạn thẳng, khoảng cách là tới điểm gần nhất trên đoạn, không phải tới hộp.

`examples/toi-noi-location/rust/crates/road-graph/src/snap.rs`

```rust
fn dist2_point_segment(p: &[f64; 2], a: &[f64; 2], b: &[f64; 2]) -> f64 {
    let d = [b[0] - a[0], b[1] - a[1]];
    let len2 = d[0] * d[0] + d[1] * d[1];
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / len2).clamp(0.0, 1.0)
    };
    let q = [a[0] + t * d[0], a[1] + t * d[1]];
    (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)
}

impl RTreeObject for Seg {
    type Envelope = AABB<[f64; 2]>;
    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(self.a, self.b)
    }
}

impl PointDistance for Seg {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        dist2_point_segment(point, &self.a, &self.b)
    }
}
```

R-tree dùng khoảng cách Euclid, nên tọa độ phải nằm trên mặt phẳng mét. `SnapIndex` chiếu mọi điểm lên mặt phẳng cục bộ `LocalProjection` quanh tâm đồ thị (sai số dưới 0,1% trên đường chéo bbox TP.HCM, kiểm bằng test `distance_matches_haversine_within_city` của `geo-core`). Kết quả cuối cùng của mỗi ứng viên không lấy từ mặt phẳng này mà tính lại chính xác bằng `project_to_segment`. Cây được dựng một lần bằng `RTree::bulk_load`; tài liệu của `rstar` nói khi việc chèn chỉ diễn ra một lần cho nhiều phần tử thì cách này nhanh hơn đáng kể so với chèn lần lượt. Bước dựng cho 306.571 đoạn mất **117 ms** trong lần cuối của lô đo và 822 ms ở một lần chạy lại lúc tải máy tăng; cả hai đều đo khi máy đang chịu tải nặng.

`examples/toi-noi-location/rust/crates/road-graph/src/snap.rs`

```rust
    pub fn build(g: &Graph) -> Self {
        let proj = LocalProjection::new(g.center());
        let mut segs = Vec::new();
        for e in 0..g.edge_count() as u32 {
            let twin = g.twin[e as usize];
            if twin != NONE && twin < e {
                continue; // đã lập chỉ mục qua cạnh twin có chỉ số nhỏ hơn
            }
            let modes = g.modes[e as usize]
                | if twin != NONE {
                    g.modes[twin as usize]
                } else {
                    0
                };
```

Truy vấn «cạnh gần nhất cho profile này» là `nearest_neighbor_iter`, trả phần tử theo thứ tự tăng dần của khoảng cách, nên chỉ cần lấy phần tử đầu tiên thỏa điều kiện profile:

`examples/toi-noi-location/rust/crates/road-graph/src/snap.rs`

```rust
    /// Cạnh gần nhất mà profile `mode` đi được (ít nhất một chiều), dùng duyệt láng giềng của R-tree.
    pub fn nearest(&self, g: &Graph, p: LatLng, mode: Mode) -> Option<SnapCandidate> {
        let xy = self.proj.to_xy(p);
        let s = self
            .tree
            .nearest_neighbor_iter(xy)
            .find(|s| s.modes & mode.bit() != 0)?;
        Some(self.make_candidate(g, s, p))
    }
```

Hình 1 vẽ cơ chế trên một ví dụ tự dựng (không phải dữ liệu OSM): ba nhóm hộp bao, điểm truy vấn nằm trong nhóm G2. Heap ưu tiên lấy G2 trước (khoảng cách tới hộp 0,0 m), xem ba đoạn bên trong, rồi thấy đoạn G2/3 cách 10,0 m còn nhỏ hơn khoảng cách tới G1 (55,0 m) và G3 (68,0 m), nên dừng. Hai nhóm kia không bao giờ bị mở. Đó là lý do truy vấn không phụ thuộc tuyến tính vào số đoạn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 370" role="img" aria-label="Hình 1. R-tree tìm đường gần nhất: ba nhóm hộp bao, điểm truy vấn nằm trong nhóm G2; heap chỉ mở G2, đoạn gần nhất nằm trong G2, hai nhóm còn lại không bao giờ được mở."><rect x="20" y="20" width="580" height="340" rx="10" class="f-bg s-line" stroke-width="1.5"/><rect x="56" y="71" width="168" height="182" rx="6" class="f-none s-dead dash" stroke-width="1.5"/><text x="58.0" y="66.0" class="t-small">G1</text><rect x="262" y="141" width="172" height="128" rx="6" class="f-none s-pk dash" stroke-width="1.5"/><text x="264.0" y="136.0" class="t-b">G2</text><rect x="462" y="121" width="104" height="158" rx="6" class="f-none s-dead dash" stroke-width="1.5"/><text x="464.0" y="116.0" class="t-small">G3</text><path d="M60.0 125.0 L150.0 75.0" class="f-none s-dead" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M150.0 75.0 L220.0 109.0" class="f-none s-dead" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M70.0 225.0 L200.0 249.0" class="f-none s-dead" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M266.0 265.0 L410.0 241.0" class="f-none s-pk" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M270.0 145.0 L400.0 161.0" class="f-none s-pk" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M346.0 213.0 L430.0 205.0" class="f-none s-phys" stroke-width="4" stroke-linejoin="round" stroke-linecap="round"/><path d="M466.0 275.0 L554.0 225.0" class="f-none s-dead" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M474.0 125.0 L562.0 155.0" class="f-none s-dead" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M494.0 205.0 L518.0 145.0" class="f-none s-dead" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round"/><circle cx="330.0" cy="201.0" r="20" class="f-none s-warn dash" stroke-width="1.5"/><circle cx="330.0" cy="201.0" r="5" class="f-warn s-warn" stroke-width="1.5"/><text x="330.0" y="237.0" text-anchor="middle">điểm GPS</text><rect x="36" y="282" width="22" height="10" rx="3" class="f-none s-pk dash" stroke-width="1.5"/><text x="66.0" y="296.0" class="t-small">nhóm được mở</text><rect x="36" y="307" width="22" height="10" rx="3" class="f-none s-dead dash" stroke-width="1.5"/><text x="66.0" y="316.0" class="t-small">nhóm bị bỏ qua</text><path d="M36.0 338.0 L58.0 338.0" class="f-none s-phys" stroke-width="4" stroke-linejoin="round" stroke-linecap="round"/><text x="66.0" y="342.0" class="t-small">đoạn gần nhất</text><circle cx="280.0" cy="292.0" r="6" class="f-none s-warn dash" stroke-width="1.5"/><text x="292.0" y="296.0" class="t-small">bán kính = khoảng cách tốt nhất hiện có</text><text x="300.0" y="40.0" class="t-small" text-anchor="middle">Minh họa cơ chế, hình học tự dựng (đơn vị mét), không phải dữ liệu OSM</text><rect x="620" y="20" width="240" height="340" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="636.0" y="46.0" class="t-b">Thứ tự lấy ra khỏi heap</text><text x="636.0" y="76.0" class="t-mono">1. nhóm G2: 0,0 m</text><text x="636.0" y="92.0" class="t-small">   mở, đọc 3 đoạn</text><text x="636.0" y="120.0" class="t-mono">2. đoạn G2/3: 10,0 m</text><text x="636.0" y="136.0" class="t-small">   gần nhất, dừng</text><text x="636.0" y="170.0" class="t-small">Còn trong heap, không mở:</text><text x="636.0" y="190.0" class="t-mono">nhóm G1: 55,0 m</text><text x="636.0" y="210.0" class="t-mono">nhóm G3: 68,0 m</text><text x="636.0" y="338.0" class="t-small">Đọc 3/9 đoạn là đủ.</text></svg>
</div>
<figcaption>Hình 1. Duyệt R-tree theo khoảng cách tới hộp bao. Chín đoạn chia ba nhóm, chỉ nhóm G2 chứa điểm được mở; 3 trên 9 đoạn được tính khoảng cách. Hình minh họa tự dựng, đơn vị mét.</figcaption>
</figure>

## Ứng viên theo bán kính: lấy bao nhiêu cạnh quanh điểm GPS

Snap bằng một cạnh gần nhất là quyết định quá sớm. Bám đường (map matching, bài 16) cần **nhiều ứng viên** quanh mỗi điểm GPS, rồi chọn bằng ngữ cảnh. `candidates` lấy mọi đoạn trong bán kính bằng `locate_within_distance`, giữ cho mỗi cạnh một ứng viên tốt nhất (đoạn gần nhất của cạnh đó), sắp xếp từ gần ra xa.

`examples/toi-noi-location/rust/crates/road-graph/src/snap.rs`

```rust
    /// Mọi cạnh trong bán kính `radius_m` (mỗi cạnh một ứng viên tốt nhất), gần nhất trước, tối đa `limit`.
    pub fn candidates(
        &self,
        g: &Graph,
        p: LatLng,
        radius_m: f64,
        mode: Mode,
        limit: usize,
    ) -> Vec<SnapCandidate> {
        let xy = self.proj.to_xy(p);
        let r = radius_m * 1.002;
        let mut best: std::collections::BTreeMap<EdgeId, SnapCandidate> =
            std::collections::BTreeMap::new();
        for s in self.tree.locate_within_distance(xy, r * r) {
            if s.modes & mode.bit() == 0 {
                continue;
            }
            let c = self.make_candidate(g, s, p);
            if c.dist_m > radius_m {
                continue;
            }
            match best.get(&c.edge) {
                Some(old) if old.dist_m <= c.dist_m => {}
                _ => {
                    best.insert(c.edge, c);
                }
            }
        }
        let mut v: Vec<SnapCandidate> = best.into_values().collect();
        v.sort_by(|a, b| a.dist_m.total_cmp(&b.dist_m).then(a.edge.cmp(&b.edge)));
        v.truncate(limit);
        v
    }
```

Bán kính nên bao nhiêu? Lab đo trên 5.000 điểm, σ = 10 m, và đếm xem cạnh của way thật có nằm trong tập ứng viên không:

| Bán kính (m) | Ứng viên trung bình | Ứng viên lớn nhất | Có cạnh thật |
|---:|---:|---:|---:|
| 10 | 1,32 | 10 | 67,1% |
| 20 | 2,79 | 22 | 95,3% |
| 30 | 4,40 | 22 | 99,7% |
| 50 | 8,65 | 39 | 100,0% |
| 100 | 24,91 | 102 | 100,0% |

Bán kính 10 m, bằng σ, bỏ mất đường thật ở một phần ba số điểm. Ba con số 67,1%, 95,3%, 99,7% gần đúng quy tắc 68–95–99,7 của phân phối chuẩn, và đó không phải trùng hợp: khoảng cách từ điểm đo tới đường thật chính là thành phần nhiễu theo một trục, vuông góc với đường. Quy tắc rút ra: lấy bán kính từ khoảng 3σ; lab dùng `max(30, 3σ)` mét. Khoảng cách từ điểm GPS tới cạnh gần nhất (σ = 10 m, 1.000 mẫu) có p50 5,2 m, p90 13,4 m, p99 21,7 m. Trong lab σ cho trước; ở service thật, độ chính xác phải đến từ trường accuracy mà thiết bị báo (bài 14), vì nhiễu thay đổi theo từng điểm.

## R-tree nhanh hơn quét toàn bộ bao nhiêu

`nearest_scan` quét hết mọi đoạn với cùng công thức khoảng cách, để so công bằng. Bảng dưới gom mọi lần đo có trong `results/p01.md`, cùng cấu hình `--release`. **Trừ dòng đầu, tất cả đều đo khi máy đang chịu tải nặng**, nên đừng trích số µs tuyệt đối:

| Cách đo | Tải máy (1 phút) | R-tree (µs/điểm) | Quét (µs/điểm) | Quét / R-tree |
|---|---|---:|---:|---:|
| lab, lúc phát triển | không có `uptime` | 1,86 | 1.080,1 | 579 |
| lab, lần cuối của lô đo | 114–124 | 6,62 | 3.187,3 | 481 |
| lab, chạy lại 1 | 104–139 | 13,70 | 4.083,5 | 298 |
| lab, chạy lại 2 | 136–154 | 2,32 | 1.380,5 | 596 |
| criterion, 256 điểm, lần 1 | 267–279 | 5,65 | 6.237,5 | 1.104 |
| criterion, 256 điểm, lần 2 | 137–145 | 2,69 | 3.596,8 | 1.338 |
| xen kẽ, 256 điểm cố định, trung vị 7 lượt | 136–154 | 2,71 | 1.581,2 | 582 |
| xen kẽ, điểm mới mỗi lượt, trung vị 7 lượt | 136–154 | 2,51 | 1.680,6 | 669 |

(Hai dòng criterion tính từ ước lượng giữa cho cả lô 256 điểm: 1,4462 ms và 1,5968 s; 688,16 µs và 920,78 ms.)

Lab và criterion khác nhau ở hai chỗ: lab đo hai bên ở hai thời điểm khác nhau, trên điểm mới; criterion lặp lại 256 điểm cố định sát nút giao, cache đã nóng. Để biết bộ điểm có phải nguyên nhân không, cờ `--ratio-rounds 7` của lab đo xen kẽ từng lượt, để R-tree và quét chịu cùng một mức tải. Hai bộ điểm cho trung vị gần nhau (582 và 669), còn tỉ lệ từng lượt đi từ 429 tới 1.422 ngay trong một lần chạy. Vậy chênh lệch chủ yếu là nhiễu của máy, không phải do bộ điểm, và không dòng nào trong bảng là «con số đúng». Kết luận chắc chắn chỉ là bậc độ lớn: **R-tree nhanh hơn quét cỡ vài trăm đến hơn một nghìn lần**, phần lớn các lần đo quanh 500–700 lần.

## 999/1000: một truy vấn chọn cạnh khác, và đó là một ca hòa

Lab cho R-tree và quét chạy trên 1.000 điểm đầu của cùng bộ truy vấn rồi so cạnh gần nhất tìm được: **cùng cạnh ở 999/1000 truy vấn**. Lab in ra truy vấn còn lại (số đếm và số hình học, không phụ thuộc tải máy):

```text
  truy vấn khác cạnh, điểm 10.753043,106.743459:
    R-tree: cạnh 267663 (nút 101269 → 101271), đoạn 0, t = 0.000000, cách 28.406593709 m, chân 10.753104,106.743207
    quét  : cạnh 267660 (nút 101268 → 101269), đoạn 5, t = 1.000000, cách 28.406593709 m, chân 10.753104,106.743207
```

Hai cạnh gặp nhau ở nút 101269. Điểm hỏi nằm ở chỗ mà điểm gần nhất của cả hai cạnh chính là nút đó, nên chân chiếu trùng nhau (`t = 1` ở cuối cạnh này, `t = 0` ở đầu cạnh kia) và khoảng cách bằng nhau tới 9 chữ số thập phân: một ca **hòa đúng nghĩa**, không phải lỗi chỉ mục. Tài liệu `rstar` 0.14.0 ghi rằng phần tử nào được trả trước trong số các phần tử cách đều là không xác định, còn `nearest_scan` giữ phần tử gặp đầu tiên (so sánh `<` chặt), nên hai bên có thể chọn khác nhau. Hệ quả thực tế: khi chân snap rơi đúng vào nút giao, «cạnh gần nhất» không xác định duy nhất, và tầng trên phải xét cả các cạnh chung nút thay vì tin vào một mã cạnh.

Vì vậy test `rtree_equals_scan_for_every_mode` của `snap.rs` so **khoảng cách** `dist_m` của hai kết quả (chênh dưới 0,01 m) trên 500 điểm cho mỗi profile, không so mã cạnh.

`examples/toi-noi-location/rust/crates/road-graph/src/snap.rs`

```rust
    #[test]
    fn rtree_equals_scan_for_every_mode() {
        let (g, idx) = setup();
        let mut seed = 42;
        for mode in Mode::ALL {
            for _ in 0..500 {
                let q = random_near(&g, &mut seed);
                let a = idx.nearest(&g, q, mode).unwrap();
                let b = idx.nearest_scan(&g, q, mode).unwrap();
                assert!(
                    (a.dist_m - b.dist_m).abs() < 0.01,
                    "{mode:?}: {} vs {}",
                    a.dist_m,
                    b.dist_m
                );
            }
        }
    }
```

## Snap sai sang đường song song: số đo và hai ca thật

Cạnh gần nhất «sai» ở đây nghĩa là **không thuộc way thật**. Lab thêm một cách chọn thứ hai: lấy ứng viên trong bán kính `max(30, 3σ)` rồi chọn điểm nhỏ nhất của `khoảng cách + 0,5 m × độ lệch hướng`, hướng đi là hướng thật cộng nhiễu 10°.

`examples/toi-noi-location/rust/labs/src/bin/loc07_snap.rs`

```rust
/// Góc lệch nhỏ nhất giữa hướng đi và các chiều ô tô được phép của ứng viên.
fn heading_penalty(g: &Graph, c: &SnapCandidate, heading: f64) -> f64 {
    let car = Mode::Car.bit();
    let mut best = 180.0f64;
    if g.modes[c.edge as usize] & car != 0 {
        best = best.min(bearing_diff_deg(heading, c.bearing_deg));
    }
    if let Some(t) = c.twin
        && g.modes[t as usize] & car != 0
    {
        best = best.min(bearing_diff_deg(heading, (c.bearing_deg + 180.0) % 360.0));
    }
    best
}
```

`examples/toi-noi-location/rust/labs/src/bin/loc07_snap.rs`

```rust
            let hd = (t.heading + rng.gauss() * 10.0).rem_euclid(360.0);
            let radius = (3.0 * sigma).max(30.0);
            let best = index
                .candidates(&g, q, radius, Mode::Car, usize::MAX)
                .into_iter()
                .min_by(|a, b| {
                    (a.dist_m + 0.5 * heading_penalty(&g, a, hd))
                        .total_cmp(&(b.dist_m + 0.5 * heading_penalty(&g, b, hd)))
                });
```

Tỉ lệ sai (20.000 điểm mỗi hàng, đồ thị ô tô HCM, seed 20261106):

| σ (m) | Gần nhất: sai | Có hướng: sai |
|---:|---:|---:|
| 3 | 6,68% | 1,60% |
| 5 | 12,18% | 3,67% |
| **10** | **26,61%** | **10,97%** |
| 15 | 39,87% | 20,10% |
| 20 | 50,51% | 29,89% |
| 30 | 64,60% | 45,51% |

Ở σ = 10 m có 5.323 ca sai, trong đó **2.912** ca chân snap cách vị trí thật 15–40 m (đường song song hoặc sát nhau) và 90 ca liên quan cầu hoặc hầm. Hướng đi chỉ cứu được các ca khác hướng; nó không cứu được hai đường cùng hướng. Hình 2 vẽ hai trong ba ca lab xuất ra GeoJSON, từ chính `results/geojson/`:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 526" role="img" aria-label="Hình 2. Hai ca snap sai đo được trên đồ thị TP.HCM với nhiễu mô phỏng σ = 10 m. Ca 1: Hoàng Diệu bị snap sang Vĩnh Khánh, chọn theo hướng đi sửa được. Ca 3: Song Hành Xa Lộ Hà Nội bị snap sang Xa lộ Hà Nội, hai đường song song cùng hướng, hướng đi không sửa được."><rect x="20" y="20" width="410" height="440" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36.0" y="46.0" class="t-b">Ca 1: Hoàng Diệu (thật) và Vĩnh Khánh (snap sai)</text><path d="M155.0 202.0 L332.5 76.0" class="f-none s-phys" stroke-width="4" stroke-linejoin="round" stroke-linecap="round"/><path d="M326.3 374.0 L142.5 249.3 L117.5 228.7" class="f-none s-warn" stroke-width="4" stroke-linejoin="round" stroke-linecap="round"/><path d="M233.7 146.2 L250.7 232.7" class="f-none s-muted dash" stroke-width="1.5"/><path d="M250.7 232.7 L208.9 294.4" class="f-none s-warn dash" stroke-width="1.5"/><circle cx="233.7" cy="146.2" r="6" class="f-phys s-phys" stroke-width="1.5"/><circle cx="250.7" cy="232.7" r="6" class="f-pk s-pk" stroke-width="1.5"/><circle cx="208.9" cy="294.4" r="5" class="f-warn s-warn" stroke-width="1.5"/><text x="229.8" y="131.0" class="t-phys t-b" text-anchor="end">Hoàng Diệu</text><text x="220.4" y="335.7" class="t-warn t-b" text-anchor="end">Vĩnh Khánh</text><text x="36.0" y="406.0" class="t-small">GPS lệch 16,1 m; cách cạnh thật 14,7 m, cách cạnh snap 13,6 m</text><text x="36.0" y="422.0" class="t-small">hai đường cách nhau 27,5 m; chọn có hướng: đúng</text><path d="M36.0 446.0 L90.6 446.0" class="f-none s-muted" stroke-width="2"/><path d="M36.0 442.0 L36.0 450.0" class="f-none s-muted" stroke-width="1.5"/><path d="M90.6 442.0 L90.6 450.0" class="f-none s-muted" stroke-width="1.5"/><text x="98.6" y="450.0" class="t-small">10 m</text><rect x="450" y="20" width="410" height="440" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="466.0" y="46.0" class="t-b">Ca 3: Song Hành Xa Lộ Hà Nội (thật) và Xa lộ Hà Nội</text><path d="M632.7 298.6 L671.4 262.3 L735.2 210.6 L789.9 160.7" class="f-none s-phys" stroke-width="4" stroke-linejoin="round" stroke-linecap="round"/><path d="M495.5 374.0 L527.4 339.0 L653.5 213.5 L814.5 76.0" class="f-none s-warn" stroke-width="4" stroke-linejoin="round" stroke-linecap="round"/><path d="M634.8 296.6 L600.9 293.5" class="f-none s-muted dash" stroke-width="1.5"/><path d="M600.9 293.5 L587.1 279.6" class="f-none s-warn dash" stroke-width="1.5"/><circle cx="634.8" cy="296.6" r="6" class="f-phys s-phys" stroke-width="1.5"/><circle cx="600.9" cy="293.5" r="6" class="f-pk s-pk" stroke-width="1.5"/><circle cx="587.1" cy="279.6" r="5" class="f-warn s-warn" stroke-width="1.5"/><text x="669.3" y="280.4" class="t-phys t-b">Song Hành Xa Lộ Hà Nội</text><text x="574.4" y="270.2" class="t-warn t-b" text-anchor="end">Xa lộ Hà Nội</text><text x="466.0" y="406.0" class="t-small">GPS lệch 12,0 m; cách cạnh thật 11,4 m, cách cạnh snap 6,9 m</text><text x="466.0" y="422.0" class="t-small">hai đường cách nhau 17,9 m; chọn có hướng: vẫn sai</text><path d="M466.0 446.0 L494.4 446.0" class="f-none s-muted" stroke-width="2"/><path d="M466.0 442.0 L466.0 450.0" class="f-none s-muted" stroke-width="1.5"/><path d="M494.4 442.0 L494.4 450.0" class="f-none s-muted" stroke-width="1.5"/><text x="502.4" y="450.0" class="t-small">10 m</text><path d="M24.0 492.0 L50.0 492.0" class="f-none s-phys" stroke-width="4"/><text x="58.0" y="496.0" class="t-small">cạnh thật</text><path d="M140.0 492.0 L166.0 492.0" class="f-none s-warn" stroke-width="4"/><text x="174.0" y="496.0" class="t-small">cạnh bị snap sai</text><circle cx="300.0" cy="492.0" r="6" class="f-phys s-phys" stroke-width="1.5"/><text x="312.0" y="496.0" class="t-small">vị trí thật</text><circle cx="410.0" cy="492.0" r="6" class="f-pk s-pk" stroke-width="1.5"/><text x="422.0" y="496.0" class="t-small">điểm GPS có nhiễu σ = 10 m (mô phỏng)</text><circle cx="670.0" cy="492.0" r="5" class="f-warn s-warn" stroke-width="1.5"/><text x="682.0" y="496.0" class="t-small">chân snap gần nhất</text><text x="860.0" y="514.0" class="t-small" text-anchor="end">© OpenStreetMap contributors</text></svg>
</div>
<figcaption>Hình 2. Trái: ca 1, snap gần nhất sai sang Vĩnh Khánh, chọn theo hướng đúng lại Hoàng Diệu. Phải: ca 3, Song Hành Xa Lộ Hà Nội bị snap sang Xa lộ Hà Nội, hai đường cùng hướng nên chọn theo hướng vẫn sai. Vẽ từ <code>loc07-hcm-snap-sai-1.geojson</code> và <code>-3.geojson</code>; nhiễu mô phỏng σ = 10 m. © OpenStreetMap contributors (ODbL).</figcaption>
</figure>

| Ca | Đường thật | Cạnh gần nhất | GPS tới đường thật | GPS tới cạnh snap | Hai đường cách nhau | Chọn theo hướng |
|---|---|---|---:|---:|---:|---|
| 1 | Hoàng Diệu (`secondary`) | Vĩnh Khánh (`tertiary`) | 14,7 m | 13,6 m | 27,5 m | đúng |
| 2 | Hẻm 150 Bến Vân Đồn (`secondary`) | Hẻm 151 Bến Vân Đồn (`secondary`) | 21,7 m | 1,7 m | 24,1 m | sai, chọn Cầu Ông Lãnh |
| 3 | Song Hành Xa Lộ Hà Nội (`secondary`) | Xa lộ Hà Nội (`trunk`) | 11,4 m | 6,9 m | 17,9 m | vẫn sai |

Ca 3 cho thấy giới hạn của cách này: hai đường cùng hướng nên độ lệch hướng của hai ứng viên gần bằng nhau, chỉ còn khoảng cách quyết định, mà nhiễu đã đẩy điểm đo về phía đường chính. Ca 2 cho thấy dữ liệu có thể làm mọi thứ khó thêm: cả hai «Hẻm» đều bị gắn `highway=secondary`, khác với giả định hẻm là `service`. Có một ca cầu: lab ghi một ca mà đường thật là Đào Sư Tích (`residential`) bị snap lên Cầu Phước Lộc (`tertiary`, cầu), cách GPS 4,1 m trong khi đường thật 10,8 m. R-tree hai chiều không biết cầu nằm trên đường, nên cần dữ liệu `layer` và ngữ cảnh chuyển động (bài 17).

Hai lưu ý để đọc đúng bảng. Một: «sai» được định nghĩa bằng mã way, nên một con đường bị OSM tách thành nhiều way mà bị snap sang way kế bên vẫn tính là sai; tỉ lệ này vì vậy là cận trên của «snap sai đường», và lab chưa tách riêng phần đó. Hai: nhiễu của lab là Gaussian độc lập giữa các điểm; GPS thật ở khu nhà cao tầng thường lệch có hệ thống và liên tục, nên số đo này không thể thay cho đo trên dữ liệu thật.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` do lab bài 06 ghi. Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc07_snap
```

Cấu hình đo: `--release`, Apple M1 Max, đồ thị HCM 126.699 nút, hồ sơ ô tô, seed 20261106, 20.000 truy vấn R-tree, 1.000 truy vấn quét, 5.000 điểm cho bảng ứng viên, 20.000 điểm mỗi mức σ. Lab ghi ba file `results/geojson/loc07-hcm-snap-sai-{1,2,3}.geojson`. Output (rút gọn), **đo khi máy đang chịu tải nặng** (tải 1 phút 114–124):

```text
  306571 đoạn thẳng trong R-tree (cặp hai chiều chỉ lập một lần), bulk_load 117 ms
  R-tree (nearest_neighbor_iter): 20000 truy vấn, 20000 có kết quả, 6.62 µs/truy vấn
  quét toàn bộ 306571 đoạn:           1000 truy vấn, 3187.3 µs/truy vấn
  tỉ lệ thời gian quét / R-tree = 481; cùng cạnh gần nhất ở 999/1000 truy vấn
  σ = 10 m: 5323 ca sai; trong đó chân snap cách điểm thật 15–40 m (đường song song hoặc gần nhau): 2912; liên quan cầu/hầm: 90
  khoảng cách GPS tới cạnh gần nhất (σ = 10 m, 1000 mẫu): p50 5.2 m, p90 13.4 m, p99 21.7 m
```

Các số đếm và tỉ lệ (số đoạn, tỉ lệ snap sai, số ứng viên) tái lập từng chữ số với cùng seed; số thời gian thì không. Thêm `-- --ratio-rounds 7` để đo xen kẽ như bảng ở trên. Đo bằng criterion: `cargo bench --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bench loc07_snap`. Không có bản đồ thật, chạy được trên đồ thị tổng hợp bằng cờ `--fixture`, cũng là lệnh mà `verify-p01.sh` gọi (`--fixture --queries 2000 --scan-queries 200`).

## Giới hạn của ví dụ

- **Một profile, chưa có chuỗi điểm.** Snap từng điểm riêng lẻ; chưa dùng thứ tự các điểm GPS. Đường song song cùng hướng chỉ giải được bằng ngữ cảnh: HMM và Viterbi ở bài 16.
- **Hướng đi là hướng thật cộng nhiễu.** Hướng từ GPS thật không có khi xe đứng yên, và nhiễu hướng ở xe chậm lớn hơn 10°. Số 10,97% của «có hướng» là cận dưới lạc quan.
- **Nhiễu mô phỏng** là Gaussian độc lập; chưa có GPS người thật.
- **Cầu, hầm** chưa được xử lý; `layer` được lưu nhưng chưa dùng khi snap.
- **Chỉ mục không sửa được.** Đồ thị đọc xong là cố định; cập nhật bản đồ nghĩa là dựng lại cây.
- **Hòa ở nút giao chưa có quy tắc phá.** Ca 1/1.000 là hòa đúng nghĩa; lab báo cạnh nào tùy thứ tự duyệt.

## Bài tập

**Bài 1.** Lab chọn bán kính ứng viên là `max(30, 3σ)`. Tính bán kính cho σ = 5 m và σ = 20 m. Bảng ứng viên đo ở σ nào, và điều đó nói gì về mức tin cậy của sàn 30 m?

<details><summary>Lời giải bài 1</summary>
<p>σ = 5 m cho 3σ = 15 m nên bán kính là 30 m (sàn); σ = 20 m cho 60 m. Bảng ứng viên chỉ đo ở σ = 10 m: nó xác nhận 3σ = 30 m cho 99,7% ở σ đó, còn sàn 30 m khi σ nhỏ hơn 10 m là lựa chọn của lab, chưa được đo riêng. Muốn chắc hơn phải chạy lại bảng với các σ khác.</p>
</details>

**Bài 2.** Vì sao `heading_penalty` thử cả `c.bearing_deg` và `c.bearing_deg + 180`?

<details><summary>Lời giải bài 2</summary>
<p>Chỉ mục giữ mỗi cặp hai chiều một lần, với hướng của cạnh đại diện. Ứng viên mang thêm cạnh <code>twin</code>, hướng ngược lại lệch 180°. Hàm tính độ lệch hướng nhỏ nhất giữa hướng đi và các chiều ô tô thật sự được phép của ứng viên: nếu chỉ thử một chiều, tài xế đi chiều ngược sẽ bị phạt oan. Với đường một chiều (<code>twin</code> là <code>None</code>) chỉ có một chiều để thử.</p>
</details>

**Bài 3.** Ở ca 999/1000, vì sao một bên báo `t = 1` còn bên kia báo `t = 0`? Muốn `nearest` trả kết quả tất định ở nút giao thì thêm quy tắc gì?

<details><summary>Lời giải bài 3</summary>
<p>Nút 101269 là nút cuối của cạnh 267660 và nút đầu của cạnh 267663, nên cùng một điểm là <code>t = 1</code> ở đoạn cuối của cạnh này và <code>t = 0</code> ở đoạn đầu của cạnh kia. Quy tắc phá hòa: tiếp tục đọc <code>nearest_neighbor_iter</code> chừng nào khoảng cách còn bằng phần tử đầu, rồi chọn chỉ số cạnh nhỏ nhất; quét toàn bộ thì so theo cặp (khoảng cách, chỉ số cạnh). Nếu hai khoảng cách khác nhau rõ rệt thì mới là lỗi thật trong chỉ mục hoặc trong lọc profile.</p>
</details>

Bài trước: [bài 06](/articles/location-osm-pbf-do-thi-duong-bo-xe-may-o-to). Bài tiếp: [bài 08](/articles/location-h3-geohash-chi-muc-luoi-k-ring).

## Tài liệu gốc

- [rstar trên docs.rs](https://docs.rs/rstar): `RTree`, `bulk_load`, `nearest_neighbor_iter`, `PointDistance`.
- [Kho mã rstar của georust](https://github.com/georust/rstar).
- A. Guttman, «R-trees: a dynamic index structure for spatial searching», ACM SIGMOD, 1984: bài báo gốc về R-tree.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho hình có dữ liệu OSM.
