---
slug: location-profile-xe-may-o-to-di-bo-trong-hem
title: "Profile xe máy, ô tô, đi bộ: cùng thành phố, ba đồ thị"
summary: "Bài 24: một đồ thị CSR dùng chung, ba mảng trọng số cho ô tô, xe máy, đi bộ; quyền đi lấy từ thẻ OSM, tốc độ theo loại đường là giả định, xử lý hẻm, và đo tuyến ô tô khác tuyến xe máy trên 300 cặp nút HCM."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, routing, profile, motorbike, openstreetmap, hem]
language: vi
---

Hai khách đứng ở cùng một chỗ trong một khu dân cư nội thành, một người đặt ô tô, một người đặt xe máy. Với đồ thị HCM của lab, cặp nút 90276 và 111467 cách nhau 3.534 m chim bay. Tuyến ô tô dài 6.166 m và mất 772,4 s. Tuyến xe máy dài 5.362 m và mất 599,4 s, đi qua tám cạnh mà ô tô không vào được; năm cạnh đầu được lab liệt kê thuộc «Hẻm 451 Đường Phạm Thế Hiển» và «Hẻm 1305 Phạm Thế Hiển». Nếu bắt xe máy chỉ chạy trên đường ô tô đi được, thời gian là 666,5 s: riêng quyền vào hẻm đã giúp nhanh hơn 10,1%.

Đây là bài 24 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài nối [bài 06](/articles/location-osm-pbf-do-thi-duong-bo-xe-may-o-to) (đọc thẻ OSM) với [bài 21](/articles/location-dijkstra-a-star-rust-do-thi-csr) (Dijkstra, A*): thêm ba cách di chuyển vào đồ thị. Cần nói rõ ngay từ đầu, vì mọi số trong bài đều chịu ràng buộc này: **tốc độ ở đây được giả định theo loại đường**, không đo từ GPS xe thật. «599,4 s» và «xe máy nhanh hơn 10,1%» là số của mô hình lab, không phải thời gian một chuyến xe máy ngoài đời.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Lab lấy 300 cặp nút ngẫu nhiên ở HCM (cùng đi được bằng cả ô tô và xe máy) và tìm tuyến nhanh nhất cho từng profile. Bạn đoán bao nhiêu phần trăm cặp có tuyến ô tô khác tuyến xe máy? Và bao nhiêu phần trăm sẽ vẫn khác nếu cho hai loại xe cùng tốc độ, chỉ khác quyền đi?</p>
<details><summary>Xem đáp án</summary><p><b>74,8%</b> cặp có tuyến khác nhau (khác tốc độ lẫn quyền đi). Khi cho xe máy chạy cùng tốc độ trên mạng ô tô đi được và trên toàn mạng, vẫn có <b>50,3%</b> cặp khác tuyến chỉ vì quyền đi. Tỉ lệ khác tăng theo khoảng cách, từ 53,3% (dưới 2 km) lên 91,1% (trên 10 km). Con số cụ thể phụ thuộc bảng tốc độ giả định.</p></details>
</div>

## Một đồ thị, ba mảng trọng số

Cách làm đơn giản nhất là dựng ba đồ thị: giữ cạnh ô tô đi được cho đồ thị ô tô, cạnh xe máy đi được cho xe máy, cạnh đi bộ cho đi bộ. Ba bản đồ gần như trùng nhau (ô tô đi được 78,9% số cạnh, xe máy 83,9%, đi bộ 99,8%) và mỗi bản kèm một chỉ mục không gian riêng.

Lab làm khác: hình dạng đồ thị, gồm CSR xuôi, CSR ngược, `tail`, `head` và độ dài cạnh, **chỉ giữ một bản** (5,84 MB). Mỗi profile chỉ thêm một mảng trọng số `u32` dài bằng số cạnh (1,22 MB). Cạnh mà profile không đi được nhận trọng số `INF`, và `View::out` của bài 21 bỏ qua nó. Một `SnapIndex` (bài 07) dùng cho cả ba, vì chỉ mục giữ mặt nạ profile của từng đoạn.

`examples/toi-noi-location/rust/crates/route-engine/src/profile.rs`

```rust
        let bit = profile.mode().bit();
        let mut w = vec![INF; g.edge_count()];
        let mut vmax = 0.0f64;
        let mut allowed = 0;
        for e in 0..g.edge_count() {
            if g.modes[e] & bit == 0 {
                continue;
            }
            allowed += 1;
            let len = g.length_cm[e];
            let c = match metric {
                Metric::Distance => len.max(1),
                Metric::Time => {
                    let kmh = f64::from(speed_kmh(e as u32)).max(1.0);
                    // t(ms) = len_cm * 36 / kmh
                    ((f64::from(len) * 36.0 / kmh).round() as u32).max(1)
                }
            };
            w[e] = c;
            vmax = vmax.max(f64::from(len) / f64::from(c));
        }
```

Mỗi profile có `vmax` riêng (số cm đi được trong một đơn vị trọng số, nhanh nhất), nên heuristic chim bay của bài 21 tính riêng cho từng profile. A* của xe máy dùng `vmax` của xe máy, không dùng của ô tô.

Tiết kiệm bộ nhớ không lớn: ba profile dùng chung tốn 9,49 MB, ba đồ thị riêng (chỉ giữ cạnh đi được, CSR cộng độ dài cộng trọng số) tốn 11,04 MB, ít hơn 14%, vì ba đồ thị rất giống nhau. Lợi ích thật là **một bản đồ trong bộ nhớ và một chỉ mục không gian**, và thêm một profile chỉ là thêm một mảng. Hình 1 minh hoạ bằng năm cạnh dựng tay, với trọng số tính từ đúng công thức `t(ms) = độ dài(cm) × 36 / km/h`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 416" role="img" aria-label="Hình 1. Ba profile dùng chung một bản đồ: hình dạng đồ thị 5,84 MB giữ một bản, mỗi profile chỉ thêm một mảng trọng số 1,22 MB; tổng 9,49 MB so với 11,04 MB nếu dựng ba đồ thị riêng. Bảng minh hoạ năm cạnh dựng tay, trọng số tính bằng công thức của lab; INF là profile không đi được."><rect x="20" y="20" width="840" height="376" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36" y="46" class="t-b">Bộ nhớ trên đồ thị HCM (MB)</text><text x="36" y="76" class="t-small">dùng chung</text><rect x="150" y="62" width="338.72" height="22" rx="3" class="f-dead s-dead" stroke-width="1"/><text x="319.36" y="77" class="t-small" text-anchor="middle">hình dạng 5,84</text><rect x="488.72" y="62" width="70.76" height="22" rx="3" class="f-pk-soft s-pk" stroke-width="1"/><rect x="559.48" y="62" width="70.76" height="22" rx="3" class="f-pk-soft s-pk" stroke-width="1"/><rect x="630.24" y="62" width="70.76" height="22" rx="3" class="f-pk-soft s-pk" stroke-width="1"/><text x="711" y="77" class="t-mono">+ 3 × 1,22 = 9,49</text><text x="36" y="106" class="t-small">ba đồ thị riêng</text><rect x="150" y="92" width="640.32" height="22" rx="3" class="f-warn-soft s-warn" stroke-width="1"/><text x="800.32" y="107" class="t-mono">11,04</text><text x="36" y="142" class="t-small">Một bản đồ trong bộ nhớ, ba mảng trọng số u32 theo chỉ số cạnh. Tiết kiệm 14%: ba đồ thị rất giống nhau.</text><text x="36" y="174" class="t-b">Năm cạnh dựng tay, trọng số tính bằng t(ms) = độ dài(cm) × 36 / tốc độ(km/h)</text><text x="36" y="198" class="t-b">cạnh</text><text x="92" y="198" class="t-b">loại đường</text><text x="300" y="198" class="t-b">dài</text><text x="370" y="198" class="t-b">ô tô (ms)</text><text x="480" y="198" class="t-b">xe máy (ms)</text><text x="610" y="198" class="t-b">đi bộ (ms)</text><text x="36" y="223">e0</text><text x="92" y="223">primary</text><text x="300" y="223" class="t-mono">200 m</text><text x="370" y="223" class="t-mono">18.947</text><text x="480" y="223" class="t-mono">18.000</text><text x="610" y="223" class="t-mono">144.000</text><text x="36" y="251">e1</text><text x="92" y="251">residential</text><text x="300" y="251" class="t-mono">100 m</text><text x="370" y="251" class="t-mono">18.000</text><text x="480" y="251" class="t-mono">13.846</text><text x="610" y="251" class="t-mono">72.000</text><text x="36" y="279">e2</text><text x="92" y="279">hẻm service=alley, width 2,2 m</text><text x="300" y="279" class="t-mono">80 m</text><rect x="364" y="264" width="56" height="22" rx="3" class="f-warn-soft s-warn" stroke-width="1"/><text x="370" y="279" class="t-mono">INF</text><text x="480" y="279" class="t-mono">16.000</text><text x="610" y="279" class="t-mono">57.600</text><text x="36" y="307">e3</text><text x="92" y="307">motorway</text><text x="300" y="307" class="t-mono">500 m</text><text x="370" y="307" class="t-mono">22.500</text><rect x="474" y="292" width="56" height="22" rx="3" class="f-warn-soft s-warn" stroke-width="1"/><text x="480" y="307" class="t-mono">INF</text><rect x="604" y="292" width="56" height="22" rx="3" class="f-warn-soft s-warn" stroke-width="1"/><text x="610" y="307" class="t-mono">INF</text><text x="36" y="335">e4</text><text x="92" y="335">footway</text><text x="300" y="335" class="t-mono">60 m</text><rect x="364" y="320" width="56" height="22" rx="3" class="f-warn-soft s-warn" stroke-width="1"/><text x="370" y="335" class="t-mono">INF</text><rect x="474" y="320" width="56" height="22" rx="3" class="f-warn-soft s-warn" stroke-width="1"/><text x="480" y="335" class="t-mono">INF</text><text x="610" y="335" class="t-mono">43.200</text><text x="36" y="370" class="t-small">Tốc độ ô tô 38, 20, 10 (hẻm bị chặn), 80; xe máy 40, 26, 18 (hẻm), không đi cao tốc; đi bộ 5 km/h. Mọi tốc độ là giả định của lab.</text><text x="36" y="388" class="t-small">Hẻm e2: ô tô INF vì width &lt; 3 m (luật của lab); xe máy chạy 18 km/h (trần hẻm của lab).</text></svg>
</div>
<figcaption>Hình 1. Trên: bộ nhớ khi dùng chung hình dạng đồ thị (5,84 MB + 3 × 1,22 MB = 9,49 MB) so với dựng ba đồ thị riêng (11,04 MB), đồ thị HCM 126.699 nút. Dưới: năm cạnh dựng tay với ba mảng trọng số; INF là profile không đi được. Tốc độ và luật hẻm là giả định của lab.</figcaption>
</figure>

## Quyền đi: đọc từ thẻ OSM

Profile nào được đi cạnh nào sẽ được quyết định **một lần**, lúc biên dịch đồ thị (bài 06), bằng `tags::classify`. Quy tắc là giả định có chủ đích và được ghi ở đầu file để đối chiếu. Mặc định theo loại đường:

`examples/toi-noi-location/rust/crates/road-graph/src/tags.rs`

```rust
    let mut m: u8 = match class {
        Motorway | MotorwayLink => Mode::CAR_BIT,
        Pedestrian | Footway | Path | Steps | Corridor => Mode::FOOT_BIT,
        Cycleway => 0,
        _ => Mode::CAR_BIT | Mode::MOTORBIKE_BIT | Mode::FOOT_BIT,
    };
```

Đường cao tốc chỉ cho ô tô (xe máy không được đi cao tốc ở Việt Nam, trừ khi dữ liệu có `motorcycle=yes`); đường đi bộ chỉ cho đi bộ; làn xe đạp không cho profile nào trong ba profile; mọi loại còn lại cho cả ba. Sau đó các thẻ cụ thể ghi đè theo thứ tự từ chung tới riêng: `access`, `vehicle`, `motor_vehicle`, `motorcar`, `motorcycle`, `foot`. Quy tắc bề rộng:

`examples/toi-noi-location/rust/crates/road-graph/src/tags.rs`

```rust
    let width_dm = get(tags, "width")
        .and_then(parse_metres)
        .map_or(0, |w| (w * 10.0).round() as u16);
    if width_dm != 0 && width_dm < CAR_MIN_WIDTH_DM {
        m &= !Mode::CAR_BIT;
    }
```

Hẻm hẹp hơn 3,0 m thì ô tô không vào. **Không có thẻ `width` thì không suy đoán và mặc định cho vào**, một lựa chọn dễ phản tác dụng, như phần dưới sẽ đo. Một chiều tính riêng cho ô tô và xe máy, còn đi bộ luôn hai chiều:

`examples/toi-noi-location/rust/crates/road-graph/src/tags.rs`

```rust
    let car_dir = dir_for(&["oneway:motorcar", "oneway:motor_vehicle"]);
    let moto_dir = dir_for(&["oneway:motorcycle", "oneway:motor_vehicle"]);
```

`add(Mode::FOOT_BIT, Dir::Both)` ghi chiều đi bộ. Nhờ vậy khách đi bộ có thể đi ngược đường một chiều, điều mà bài 33 (sinh ứng viên điểm đón) cần. Kết quả trên bản đồ HCM: ô tô đi được 251.687 cạnh (78,9%), xe máy 267.578 (83,9%), đi bộ 318.446 (99,8%) trong 319.074 cạnh.

## Tốc độ theo loại đường: một bảng giả định

Trọng số thời gian cần tốc độ, và lab không có dữ liệu tốc độ thật. Lab đặt một bảng tốc độ theo loại đường (Hình 2), rồi hiệu chỉnh theo `maxspeed` và hẻm:

`examples/toi-noi-location/rust/crates/route-engine/src/profile.rs`

```rust
pub fn edge_speed_kmh(g: &Graph, e: u32, p: Profile, table: &SpeedTable) -> f32 {
    let i = e as usize;
    let mut v = table.kmh[g.class[i] as usize];
    if p == Profile::Foot {
        return v;
    }
    if g.maxspeed_kmh[i] != 0 {
        v = v.min(f32::from(g.maxspeed_kmh[i]));
    }
    if g.flags[i] & flag::ALLEY != 0 {
        v = v.min(if p == Profile::Car { 10.0 } else { 18.0 });
    }
    let width_dm = g.width_dm[i];
    if width_dm != 0 && width_dm < 40 && p == Profile::Car {
        v = v.min(10.0);
    }
    v.max(3.0)
}
```

Đi bộ chạy 5 km/h ở hầu hết loại đường (bậc thang 3, đường mòn `track` 4,5), bỏ qua `maxspeed`. Ô tô và xe máy lấy tốc độ theo loại đường rồi không được vượt quá `maxspeed` nếu có. Hẻm `service=alley` cho tối đa 10 km/h (ô tô) hoặc 18 km/h (xe máy); đường hẹp dưới 4 m chặn ô tô ở 10 km/h. Xe máy nhanh hơn ô tô trên đường nhỏ (26 so với 20 km/h ở `residential`, 20 so với 12 ở `service`), một giả định về việc xe máy luồn lách mà **chưa hiệu chuẩn bằng GPS xe máy thật**.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 396" role="img" aria-label="Hình 2. Bảng tốc độ giả định của lab theo loại đường (km/h): ô tô và xe máy; đi bộ là 5 km/h ở hầu hết loại đường. Tốc độ không đo từ xe thật."><rect x="20" y="20" width="840" height="356" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36" y="46" class="t-b">Tốc độ giả định của lab theo loại đường (km/h) — không phải số đo</text><text x="36" y="75" class="t-mono">motorway</text><rect x="160" y="62" width="336" height="12" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="502" y="72" class="t-mono">80</text><text x="160" y="87" class="t-small">xe máy: không đi (trừ khi có motorcycle=yes)</text><text x="36" y="106" class="t-mono">trunk</text><rect x="160" y="93" width="210" height="12" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="376" y="103" class="t-mono">50</text><rect x="160" y="107" width="210" height="12" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="376" y="117" class="t-mono">50</text><text x="36" y="137" class="t-mono">primary</text><rect x="160" y="124" width="159.6" height="12" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="325.6" y="134" class="t-mono">38</text><rect x="160" y="138" width="168" height="12" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="334" y="148" class="t-mono">40</text><text x="36" y="168" class="t-mono">secondary</text><rect x="160" y="155" width="138.6" height="12" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="304.6" y="165" class="t-mono">33</text><rect x="160" y="169" width="151.2" height="12" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="317.2" y="179" class="t-mono">36</text><text x="36" y="199" class="t-mono">tertiary</text><rect x="160" y="186" width="117.6" height="12" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="283.6" y="196" class="t-mono">28</text><rect x="160" y="200" width="134.4" height="12" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="300.4" y="210" class="t-mono">32</text><text x="36" y="230" class="t-mono">unclassified</text><rect x="160" y="217" width="105" height="12" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="271" y="227" class="t-mono">25</text><rect x="160" y="231" width="126" height="12" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="292" y="241" class="t-mono">30</text><text x="36" y="261" class="t-mono">residential</text><rect x="160" y="248" width="84" height="12" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="250" y="258" class="t-mono">20</text><rect x="160" y="262" width="109.2" height="12" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="275.2" y="272" class="t-mono">26</text><text x="36" y="292" class="t-mono">living_street</text><rect x="160" y="279" width="50.4" height="12" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="216.4" y="289" class="t-mono">12</text><rect x="160" y="293" width="75.6" height="12" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="241.6" y="303" class="t-mono">18</text><text x="36" y="323" class="t-mono">service</text><rect x="160" y="310" width="50.4" height="12" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="216.4" y="320" class="t-mono">12</text><rect x="160" y="324" width="84" height="12" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="250" y="334" class="t-mono">20</text><rect x="620" y="70" width="22" height="12" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="650" y="80" class="t-small">ô tô</text><rect x="620" y="92" width="22" height="12" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="650" y="102" class="t-small">xe máy</text><text x="620" y="134" class="t-small">Đi bộ: 5 km/h ở hầu hết loại</text><text x="620" y="150" class="t-small">đường đi bộ được (bậc thang 3,</text><text x="620" y="166" class="t-small">đường mòn track 4,5).</text><text x="620" y="198" class="t-small">Bị chặn thêm bởi thẻ maxspeed,</text><text x="620" y="214" class="t-small">hẻm service=alley: ô tô 10, xe máy 18;</text><text x="620" y="230" class="t-small">width &lt; 4 m: ô tô tối đa 10.</text><text x="620" y="262" class="t-small">Xe máy nhanh hơn ô tô ở đường nhỏ</text><text x="620" y="278" class="t-small">vì lách được; đây là giả định, chưa</text><text x="620" y="294" class="t-small">hiệu chuẩn bằng GPS xe máy thật.</text><text x="36" y="366" class="t-small">Nguồn: bảng SpeedTable trong profile.rs và results/p03.md, LOC-24 mục 2.</text></svg>
</div>
<figcaption>Hình 2. Bảng tốc độ giả định theo loại đường của lab (km/h), ô tô và xe máy. Đi bộ 5 km/h ở hầu hết loại đường. Đây là tham số đặt tay trong <code>SpeedTable</code>, không phải số đo; thay bảng này sẽ đổi mọi tỉ lệ ở phần dưới.</figcaption>
</figure>

## Hẻm: dữ liệu thiếu chứ không phải thuật toán thiếu

Bài 06 đã nêu hẻm là chỗ dữ liệu yếu nhất. Số đếm trong bài này: HCM có 111.318 cạnh `highway=service`, trong đó 69.462 gắn `service=alley`. Nhưng chỉ **438 cạnh (0,14% tổng số cạnh)** có thẻ `width`, và chỉ **64 cạnh** có bề rộng dưới 3,0 m để luật «ô tô không vào hẻm hẹp» chặn được.

Vậy tại sao xe máy đi được ở nơi ô tô không? Lab tách 17.513 cạnh (5,5%) xe máy đi được mà ô tô không:

| Lý do | Số cạnh |
|---|---:|
| Bề rộng dưới 3 m | 46 |
| Một chiều chỉ với ô tô (chiều ngược còn cho ô tô: cạnh `twin` đi được) | 838 |
| Lý do khác: thẻ `access`, `motor_vehicle`, `motorcycle`... | 16.629 |

Phần lớn không phải bề rộng. Nguyên nhân chính là thẻ truy cập: theo loại đường, cạnh chỉ xe máy gồm `service` 10.105, `residential` 6.498, `primary` 368, `tertiary` 260. Chiều ngược lại, 1.622 cạnh ô tô đi được mà xe máy không (`trunk` 765, `primary` 276, `service` 192, `tertiary` 114), và 49.874 cạnh chỉ đi bộ. Hệ quả cho bài toán điểm đón: «hẻm ô tô không vào» ở TP.HCM trong dữ liệu OSM thường không phải do người lập bản đồ ghi `width=2.2`, mà do thẻ truy cập, và những hẻm không có thẻ nào sẽ bị coi là ô tô vào được, chỉ bị giới hạn tốc độ ở 10 km/h nếu có `service=alley`. Đo mức sai lệch đó cần dữ liệu thực địa mà lab không có.

## Tuyến ô tô khác tuyến xe máy bao nhiêu

Lab lấy 300 cặp nút ngẫu nhiên trong **giao** của hai thành phần liên thông mạnh lớn nhất (ô tô và xe máy), gồm 110.548 nút, seed 20261106, để mọi cặp đều có đường cho cả hai profile. Lọc theo từng profile riêng sẽ cho hai tập nút khác nhau nên phép so sánh trở nên vô nghĩa. Với mỗi cặp, A* chạy bốn lần: ô tô, xe máy, đi bộ, và một lần thứ tư của **xe máy chỉ trên cạnh ô tô đi được** (cùng tốc độ xe máy, chỉ khác quyền đi) để tách hai nguyên nhân.

- Tuyến ô tô khác tuyến xe máy (khác dãy cạnh) ở **74,8%** cặp, gồm cả khác do tốc độ và do quyền đi.
- Khác **chỉ vì quyền đi** ở **50,3%** cặp; và đúng bằng 50,3% tuyến xe máy dùng ít nhất một cạnh ô tô không vào được.
- Trong 150 cặp khác vì quyền đi, xe máy nhanh hơn 6,8% trung bình, trung vị 3,6%, p95 24,0% so với xe máy bị giới hạn trên mạng ô tô.
- Tỉ lệ thời gian xe máy so với ô tô: trung bình 0,925 (trung vị 0,918, p5 0,828, p95 1,053). Độ dài: trung bình 0,979, trung vị 0,996.

Hình 3 chia theo khoảng cách chim bay: càng xa, tuyến càng khác (91,1% ở trên 10 km). Có thể hiểu là tuyến dài có nhiều cơ hội đổi đường hơn, nhưng lab chưa kiểm chứng giả thuyết đó. Tỉ lệ thời gian xe máy so với ô tô tăng từ 0,857 lên 0,955: lợi thế của xe máy giảm dần theo khoảng cách. Một cách giải thích có thể là đoạn đường lớn chiếm nhiều hơn ở tuyến dài, nơi hai bảng tốc độ gần nhau hơn (xe máy 40 so với ô tô 38 km/h ở `primary`), nhưng đó cũng chỉ là giả thuyết.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 3. 300 cặp nút ngẫu nhiên trong giao hai thành phần liên thông lớn nhất của ô tô và xe máy trên HCM. Tỉ lệ tuyến ô tô khác tuyến xe máy: 53,3%; 60,7%; 71,1%; 91,1% theo nhóm khoảng cách. Thời gian xe máy trên ô tô: 0,857; 0,882; 0,930; 0,955."><rect x="20" y="20" width="840" height="260" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36" y="46" class="t-b">Tuyến ô tô khác tuyến xe máy (%)</text><text x="456" y="46" class="t-b">Thời gian xe máy / ô tô (trung bình)</text><text x="36" y="80" class="t-small">&lt; 2 km (15)</text><rect x="130" y="66" width="127.92" height="22" rx="3" class="f-warn-soft s-warn" stroke-width="1"/><text x="265.92" y="81" class="t-mono">53,3%</text><text x="456" y="80" class="t-small">&lt; 2 km (15)</text><rect x="556" y="66" width="233.727" height="22" rx="3" class="f-pk-soft s-pk" stroke-width="1"/><text x="781.727" y="81" class="t-mono" text-anchor="end">0,857</text><text x="36" y="120" class="t-small">2–5 km (61)</text><rect x="130" y="106" width="145.68" height="22" rx="3" class="f-warn-soft s-warn" stroke-width="1"/><text x="283.68" y="121" class="t-mono">60,7%</text><text x="456" y="120" class="t-small">2–5 km (61)</text><rect x="556" y="106" width="240.545" height="22" rx="3" class="f-pk-soft s-pk" stroke-width="1"/><text x="788.545" y="121" class="t-mono" text-anchor="end">0,882</text><text x="36" y="160" class="t-small">5–10 km (121)</text><rect x="130" y="146" width="170.64" height="22" rx="3" class="f-warn-soft s-warn" stroke-width="1"/><text x="308.64" y="161" class="t-mono">71,1%</text><text x="456" y="160" class="t-small">5–10 km (121)</text><rect x="556" y="146" width="253.636" height="22" rx="3" class="f-pk-soft s-pk" stroke-width="1"/><text x="801.636" y="161" class="t-mono" text-anchor="end">0,930</text><text x="36" y="200" class="t-small">&gt; 10 km (101)</text><rect x="130" y="186" width="218.64" height="22" rx="3" class="f-warn-soft s-warn" stroke-width="1"/><text x="356.64" y="201" class="t-mono">91,1%</text><text x="456" y="200" class="t-small">&gt; 10 km (101)</text><rect x="556" y="186" width="260.455" height="22" rx="3" class="f-pk-soft s-pk" stroke-width="1"/><text x="808.455" y="201" class="t-mono" text-anchor="end">0,955</text><path d="M828.727 60 L828.727 214" class="f-none s-muted dash" stroke-width="1.5" stroke-linecap="round"/><text x="850" y="228" class="t-small" text-anchor="end">bằng nhau = 1,0</text><text x="36" y="258" class="t-small">«Khác tuyến» gồm cả khác do tốc độ lẫn do quyền đi. Tốc độ giả định, nên 0,925 trung bình không phải số đo ngoài đời.</text><text x="36" y="274" class="t-small">© OpenStreetMap contributors (ODbL). Nguồn: results/p03.md, LOC-24 mục 4; seed 20261106.</text></svg>
</div>
<figcaption>Hình 3. Theo nhóm khoảng cách chim bay (số cặp trong ngoặc): tỉ lệ tuyến ô tô khác tuyến xe máy và thời gian xe máy trên ô tô. Tốc độ giả định, nên tỉ lệ thời gian là số của mô hình. 300 cặp nút HCM, seed 20261106. © OpenStreetMap contributors (ODbL).</figcaption>
</figure>

Hai ca thật giúp hình dung rõ hơn. **Ca hẻm** (đã nói ở đầu bài): nút 90276 tới nút 111467, ô tô 772,4 s trên 6.166 m, xe máy 599,4 s trên 5.362 m, xe máy chỉ trên cạnh ô tô vào được 666,5 s trên 6.129 m. Xe máy đi qua tám cạnh không vào được cho ô tô; lab liệt kê năm cạnh đầu, đều thuộc «Hẻm 451 Đường Phạm Thế Hiển» hoặc «Hẻm 1305 Phạm Thế Hiển». Trong dữ liệu chúng là `highway=residential`, không có `width`, không có `service=alley`, và chiều ngược cũng không cho ô tô. Đọc thẳng file PBF: way 1322052085 ghi `motorcar=no`, way 686305068 ghi `motor_vehicle=no` và `motorcycle=yes`, đúng kiểu thẻ truy cập ở mục trên.

**Ca lớn nhất** (nhanh hơn 58,7%) là nút 17055 tới nút 100680: tuyến xe máy dùng một way tên «Cầu Sài Gòn 2» (1.100 m, `primary`) và một đoạn Điện Biên Phủ, hai cạnh ô tô không đi được. Đây không phải lỗi luật của lab. Trong PBF, way 306073180 và way 192289472 đều ghi `motorcar=no`, `motorcycle=yes`, `oneway=yes`; còn way 306073179 cùng tên là `highway=trunk` với `motorcycle=no`. Tức dữ liệu OSM tách phần đường cho xe máy khỏi phần đường cho ô tô, và lab đọc đúng như vậy; lab không kiểm chứng ngoài thực địa. Vì thế 58,7% **không** nói xe máy nhanh hơn ô tô: ở cặp này ô tô mất 298,4 s, xe máy 312,1 s. Nó chỉ cho biết rằng nếu đuổi xe máy khỏi phần đường dành riêng cho xe máy mà phần đường ô tô lại cấm xe máy, thì xe máy phải vòng 755,9 s trên 7.861 m. Đó là hệ quả của phép so sánh phản thực tế, không nên dùng làm ví dụ ngoài đời thực.

Đi bộ thì khác hẳn: tuyến đi bộ ngắn hơn tuyến ô tô trong 96,6% cặp (đi ngược đường một chiều, qua cầu bộ hành, hẻm), độ dài đi bộ trên ô tô trung bình 0,920, trung vị 0,934, nhỏ nhất 0,261. Thời gian đi bộ gấp 4,81 lần (dưới 2 km) đến 7,07 lần (trên 10 km) thời gian ô tô, kết quả trực tiếp của 5 km/h so với tốc độ ô tô giả định.

## Đi bộ: tìm kiếm có giới hạn

Khách đi bộ không cần «đường tới một nút» mà cần «mọi nơi tới được trong N phút». `Searcher::within` chạy Dijkstra trên đồ thị đi bộ và dừng khi vượt ngân sách, nên không quét cả đồ thị. Lab lấy 60 điểm xuất phát ngẫu nhiên có seed:

| Ngân sách đi bộ | Nút tới được (TB) | Nút p95 | Chim bay xa nhất (TB) | Xe máy cùng thời gian (nút TB) |
|---:|---:|---:|---:|---:|
| 2 phút | 20 | 39 | 135 m | 506 |
| 5 phút | 109 | 267 | 360 m | 4.341 |
| 10 phút | 451 | 925 | 736 m | 19.666 |

Trong 5 phút (khoảng 417 m ở 5 km/h) khách tới trung bình 109 nút, trong khi một xe máy cùng 5 phút tới 4.341 nút. Mỗi lần tìm chỉ mất vài tới vài chục µs vì chỉ duyệt nút trong ngân sách; số µs này đo khi máy chịu tải nặng nên chỉ cho biết bậc độ lớn. Đây là nền cho bài 33: sinh ứng viên điểm đón là chọn trong tập nút khách tới được trong ngưỡng đi bộ.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` (hoặc `-- --fixture`). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc24_profile
```

Cấu hình đo: `--release`, Apple M1 Max, 300 cặp nút trong giao hai thành phần liên thông lớn nhất (110.548 nút), seed 20261106, A* theo thời gian. Output (rút gọn) của lần chạy duy nhất; các số dưới đây là số đếm hoặc tỉ lệ nên không phụ thuộc tải máy (lúc chạy tải 1 phút 21,70):

```text
-- 1. cấu trúc dữ liệu dùng chung --
  hình dạng đồ thị dùng chung (CSR thuận + ngược + độ dài): 5.84 MB, giữ một bản; mỗi profile chỉ thêm mảng trọng số 1.22 MB
  tổng ba profile dùng chung: 9.49 MB; nếu dựng ba đồ thị riêng (chỉ giữ cạnh đi được, CSR + độ dài + trọng số): 11.04 MB
  car        cạnh đi được   251.687  (78.9% cạnh)
  motorbike  cạnh đi được   267.578  (83.9% cạnh)
  foot       cạnh đi được   318.446  (99.8% cạnh)

-- 3. hẻm và những chỗ ba profile khác nhau trên dữ liệu thật --
  cạnh highway=service: 111.318; trong đó service=alley: 69.462; có thẻ width: 438 (0.14% cạnh); width < 3,0 m (ô tô bị chặn): 64
  cạnh xe máy đi được mà ô tô không: 17.513 (5.5%): do width < 3 m 46, do một chiều chỉ với ô tô (chiều ngược còn cho ô tô: cạnh twin đi được) 838, lý do khác (thẻ access, motor_vehicle, đường dành cho xe máy...) 16.629

-- 4. tuyến ô tô so với tuyến xe máy: 300 cặp ngẫu nhiên trong giao của hai SCC lớn nhất (110.548 nút) --
  tuyến ô tô khác tuyến xe máy (khác dãy cạnh): 74.8% (gồm cả khác do tốc độ và do quyền đi)
  khác chỉ vì quyền đi (xe máy trên toàn mạng so với xe máy chỉ trên cạnh ô tô đi được, cùng tốc độ): 50.3%; tuyến xe máy dùng ít nhất một cạnh ô tô không vào được: 50.3%
  trong các ca khác vì quyền đi (150): xe máy nhanh hơn 6.8% trung bình, trung vị 3.6%, p95 24.0%
  xe máy / ô tô: thời gian trung bình 0.925 (trung vị 0.918, p5 0.828, p95 1.053); độ dài trung bình 0.979 (trung vị 0.996)
    nhóm           n khác tuyến t xe máy/ô tô t đi bộ/ô tô  d đi bộ/ô tô
    < 2 km        15      53.3%        0.857         4.81         0.846
    2–5 km        61      60.7%        0.882         5.82         0.898
    5–10 km      121      71.1%        0.930         6.47         0.925
    > 10 km      101      91.1%        0.955         7.07         0.939
  đi bộ so với ô tô: đường đi bộ ngắn hơn đường ô tô trong 96.6% cặp (đi ngược đường một chiều, qua cầu bộ hành, hẻm); độ dài đi bộ/ô tô trung bình 0.920, trung vị 0.934, nhỏ nhất 0.261
```

Kiểm thử tự động: test `disallowed_edges_are_inf_and_allowed_are_positive` (mọi cạnh profile không đi được có trọng số `INF`, mọi cạnh đi được có trọng số dương), `vmax_bounds_every_edge` (heuristic chim bay của từng profile hợp lệ), `foot_is_slower_than_car_on_an_avenue` (đi bộ chậm hơn ô tô trên đường lớn); và `proptest_vs_dijkstra.rs` chạy mọi thuật toán nhanh trên cả ba profile.

## Giới hạn của ví dụ

- **Tốc độ là giả định.** Bảng theo loại đường chưa hiệu chuẩn bằng GPS; mọi tỉ lệ thời gian (0,925; 10,1%...) là của mô hình. Xe máy 26 km/h trên `residential` là điều lab đặt, không phải điều lab đo.
- **Chênh lệch giữa ô tô và xe máy là số do mô hình tạo ra.** 74,8% tuyến khác nhau và 50,3% do quyền đi phụ thuộc cả luật quyền đi (đọc từ thẻ OSM) lẫn bảng tốc độ; đổi một trong hai là đổi con số.
- **Hẻm không có `width` được coi là ô tô vào được.** Chỉ 0,14% cạnh có thẻ này, nên luật bề rộng chỉ chặn 64 cạnh; thực tế ô tô không vào được nhiều hẻm hơn thế.
- **Ca 58,7% là so sánh phản thực tế.** Thẻ OSM đã kiểm trong file PBF, nhưng chưa kiểm ngoài thực địa, và xe máy ở cặp đó vẫn chậm hơn ô tô; không dùng làm ví dụ ngoài đời.
- **Không có thời gian chờ, đèn đỏ, chi phí rẽ.** Chi phí rẽ ở bài 23 chỉ áp cho A* theo cạnh; ba profile ở bài này là theo nút.
- **Chưa có CH cho đi bộ.** Đi bộ dùng A* và tìm kiếm có giới hạn; ô tô và xe máy có CH ở bài 25.
- **Một bản đồ, một thành phố.** Đồ thị cố định, bbox nội thành TP.HCM.

## Bài tập

**Bài 1.** Một cạnh `residential` dài 150 m. Tính trọng số thời gian (ms) của nó cho ô tô (20 km/h) và xe máy (26 km/h) theo công thức của lab. Tỉ lệ xe máy trên ô tô là bao nhiêu?

<details><summary>Lời giải bài 1</summary>
<p><code>len = 15.000 cm</code>. Ô tô: <code>15.000 × 36 / 20 = 27.000 ms</code> (27 s). Xe máy: <code>15.000 × 36 / 26 ≈ 20.769 ms</code>. Tỉ lệ 20.769 / 27.000 ≈ 0,77. Đây là tỉ lệ trên một cạnh đơn lẻ; trên cả tuyến, tỉ lệ trung bình là 0,925; một phần có thể vì đường lớn chiếm nhiều quãng đường và hai bảng tốc độ gần nhau hơn ở đó (0,95 ở <code>primary</code>), điều lab chưa tách riêng để kiểm.</p>
</details>

**Bài 2.** Vì sao lab lấy mẫu 300 cặp nút trong **giao** của hai thành phần liên thông mạnh lớn nhất của ô tô và xe máy, thay vì mỗi profile một thành phần riêng?

<details><summary>Lời giải bài 2</summary>
<p>So sánh tuyến ô tô với tuyến xe máy chỉ có nghĩa nếu mỗi cặp có đường cho cả hai profile. Nếu mỗi profile lấy mẫu riêng, một số cặp có đường cho ô tô mà không có cho xe máy (hoặc ngược lại), và các tỉ lệ trung bình tính trên hai tập khác nhau. Giao hai thành phần (110.548 nút, ít hơn 111.441 nút của riêng ô tô) đảm bảo mọi cặp đều có đường ở cả hai profile.</p>
</details>

**Bài 3.** Một hẻm `service=alley` không có thẻ `width`. Luật hiện tại cho ô tô đi vào với tốc độ tối đa 10 km/h. Chỉ ra rủi ro cho điểm đón và đề xuất một cách giảm thiểu mà không cần dữ liệu thực địa.

<details><summary>Lời giải bài 3</summary>
<p>Ô tô có thể được gợi ý vào một hẻm mà ngoài đời ô tô không vào được, vì chỉ 0,14% cạnh có thẻ <code>width</code>. Tuyến được tính là chậm (10 km/h) nhưng vẫn «đi được», nên thời gian đón bị ước thấp và tài xế bị gọi vào chỗ không có lối. Cách giảm thiểu: cho ô tô đi vào hẻm không thẻ <code>width</code> chỉ khi hẻm là đích hoặc điểm đầu (không đi xuyên qua), hoặc phạt nặng cạnh hẻm không thẻ bề rộng khi tính điểm đón ở P04. Cả hai cách cần kiểm bằng simulator, vì chúng đổi <code>deadhead_m</code>.</p>
</details>

Bài trước: [bài 23](/articles/location-chi-phi-re-u-turn-do-thi-theo-canh). Bài tiếp: [bài 25](/articles/location-contraction-hierarchies-tu-cai-dat-fast-paths).

## Tài liệu gốc

- [Wiki OpenStreetMap: Key:access](https://wiki.openstreetmap.org/wiki/Key:access): thẻ truy cập theo phương tiện.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho hình có dữ liệu OSM.
