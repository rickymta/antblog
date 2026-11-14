---
slug: location-osm-pbf-do-thi-duong-bo-xe-may-o-to
title: "Từ OpenStreetMap tới đồ thị đường bộ: một chiều, xe máy, ô tô"
summary: "Bài 06: đọc PBF OpenStreetMap bằng osmpbf, lọc bbox TP.HCM, đọc thẻ highway, oneway, motorcycle, tách way thành cạnh CSR cho ô tô, xe máy, đi bộ; đo chất lượng dữ liệu hẻm và thành phần liên thông."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, openstreetmap, osm-pbf, graph, csr, routing]
language: vi
---

Khách đặt xe ô tô từ trong một hẻm ở TP.HCM. Engine gợi ý tài xế chạy vào tận cửa hẻm đón, vì trên bản đồ đó là một đường `highway=service`, `service=alley`, nối thẳng ra đường lớn. Giả sử hẻm chỉ rộng 2,2 m: xe máy vào được, ô tô thì không. Muốn engine biết điều đó, dữ liệu phải có chiều rộng. Trong bản trích OpenStreetMap của nội thành mà lab này đọc có **22.365 way hẻm, và chỉ 26 way có thẻ `width`**.

Đó là chỗ bắt đầu của mọi thứ phía sau: trước khi snap, bám đường hay tìm đường, phải có một đồ thị đường bộ, và đồ thị đó chỉ tốt bằng dữ liệu nguồn. Đây là bài 06 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng crate `geo-core` đã dựng ở [bài 03](/articles/location-toa-do-wgs84-newtype-lat-lng-do-chinh-xac), [bài 04](/articles/location-khoang-cach-haversine-equirectangular-sai-so) và [bài 05](/articles/location-geometry-crate-geo-chieu-diem-polyline-bearing).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một way có <code>highway=service</code>, <code>service=alley</code>, <code>width=2.2</code> và <code>motorcycle=yes</code>. Ô tô, xe máy, người đi bộ, ai đi được? Nếu way này không có thẻ <code>width</code> thì kết quả đổi thế nào?</p>
<details><summary>Xem đáp án</summary><p>Với <code>width=2.2</code>: xe máy và người đi bộ đi được, ô tô không, vì lab chặn ô tô khi <code>width</code> nhỏ hơn 3,0 m. Bỏ thẻ <code>width</code>: lab không đoán, nên <b>ô tô được vào</b>. Vì 99,9% hẻm thiếu <code>width</code>, đồ thị dành cho ô tô sẽ cho phép đi vào cả những hẻm quá hẹp; mục «Chất lượng dữ liệu» bên dưới có con số.</p></details>
</div>

## Dữ liệu từ đâu, giấy phép nào, và vì sao có file `hcm-graph.bin`

Bản đồ của lab là OpenStreetMap, dữ liệu **© OpenStreetMap contributors**, giấy phép **ODbL** (Open Database License). Mọi hình trong bài có dữ liệu OSM đều ghi dòng ghi công này. Script `data/fetch.sh` tải bản trích Việt Nam từ Geofabrik (`vietnam-latest.osm.pbf`, 332.178.522 byte) về `examples/toi-noi-location/data/`; file này không được commit. ODbL có hai nghĩa vụ: ghi công và nói rõ dữ liệu thuộc giấy phép này ở mọi sản phẩm dùng nó, và chia sẻ theo cùng giấy phép khi công bố một cơ sở dữ liệu dẫn xuất. File đồ thị `hcm-graph.bin` sinh ra từ dữ liệu OSM thuộc diện cơ sở dữ liệu dẫn xuất, nên trước khi phân phối hay dùng nó công khai phải đọc kỹ điều khoản; bài này không phải tư vấn pháp lý. Repo không commit PBF lẫn `hcm-graph.bin`. Thứ được commit là mã, đồ thị tổng hợp nhỏ (bài 10), số thống kê trong `results/p01.md`, và ba đoạn đường rất ngắn trích từ OSM trong `results/geojson/loc07-hcm-snap-sai-*.geojson` để vẽ hình ở bài 07; hình đó ghi công và nêu ODbL.

Định dạng PBF (Protocolbuffer Binary Format) chia dữ liệu thành các khối (blob) nén, giải mã được song song. Crate `osmpbf` **chỉ đọc PBF, không ghi được**, nên chữ «cắt theo bbox» trong lab có nghĩa khác với công cụ `osmium extract`: lab không tạo ra `hcm.pbf`. Lab đọc file Việt Nam, giữ phần nằm trong bbox lat 10,70–10,86, lng 106,60–106,78 (nội thành TP.HCM), biên dịch thành đồ thị rồi ghi **`data/hcm-graph.bin`**, một file nhị phân có phiên bản (magic `TNGRAPH`). Service chạy thật chỉ cần nạp file nhỏ này.

## Ba lượt đọc PBF

Một way OSM chỉ liệt kê mã các nút, không chứa toạ độ. Vì vậy muốn biết way có nằm trong bbox hay không phải biết toạ độ nút trước. Hàm `read_pbf` đọc file ba lượt, mỗi lượt giải mã song song theo khối bằng `rayon`: lượt 1 giữ nút trong bbox, lượt 2 giữ way `highway` có ít nhất một nút trong tập đó, lượt 3 giữ relation cấm rẽ (`type=restriction`) chạm vào các way đã giữ.

`examples/toi-noi-location/rust/crates/road-graph/src/pbf.rs`

```rust
    // Lượt 2: way.
    let t = Instant::now();
    let mut way_parts: Vec<(usize, u64, Vec<RawWay>)> = all_blobs
        .par_iter()
        .filter_map(|(idx, blob)| {
            let BlobDecode::OsmData(block) = blob.decode().ok()? else {
                return None;
            };
            let mut total_hw = 0u64;
            let mut ways = Vec::new();
            for group in block.groups() {
                for w in group.ways() {
                    if !w.tags().any(|(k, _)| k == "highway") {
                        continue;
                    }
                    total_hw += 1;
                    let refs: Vec<i64> = w.refs().collect();
                    if refs.iter().any(|r| node_map.contains_key(r)) {
                        ways.push(RawWay {
                            id: w.id(),
                            nodes: refs,
                            tags: tags_vec(w.tags()),
                        });
                    }
                }
            }
            Some((*idx, total_hw, ways))
        })
        .collect();
```

Số đếm của lần chạy trên bản trích đã tải (đây là số đếm nên tái lập được từng chữ số): quét **46.566.872** nút, **800.623** nút nằm trong bbox; trong **3.157.498** way `highway` của cả file chỉ **80.629** way chạm bbox; **3.547** relation cấm rẽ toàn file. Bbox cắt ngang **729** way, các way đó chỉ giữ phần trong bbox.

## Từ thẻ OSM tới quyền đi của ba profile

Mỗi cạnh mang một mặt nạ ba bit: ô tô, xe máy, đi bộ, riêng cho từng chiều. Hàm `classify` suy mặt nạ đó từ thẻ của way theo thứ tự từ chung tới riêng: loại đường cho mặt nạ gốc, rồi `access` kiểu cấm tắt cả ba profile và `vehicle` kiểu cấm tắt hai profile cơ giới, rồi `motor_vehicle`, `motorcar`, `motorcycle`, `foot` bật hoặc tắt từng profile, cuối cùng `width` dưới 3,0 m tắt ô tô.

`examples/toi-noi-location/rust/crates/road-graph/src/tags.rs`

```rust
    use HighwayClass::*;
    let mut m: u8 = match class {
        Motorway | MotorwayLink => Mode::CAR_BIT,
        Pedestrian | Footway | Path | Steps | Corridor => Mode::FOOT_BIT,
        Cycleway => 0,
        _ => Mode::CAR_BIT | Mode::MOTORBIKE_BIT | Mode::FOOT_BIT,
    };
    let motor = Mode::CAR_BIT | Mode::MOTORBIKE_BIT;

    // Thứ tự từ chung tới riêng, thẻ riêng ghi đè thẻ chung.
    if let Some(v) = get(tags, "access") {
        *stats.entry(format!("tag.access={v}")).or_default() += 1;
        if denies(v) {
            m = 0;
        }
    }
    if let Some(v) = get(tags, "vehicle")
        && denies(v)
    {
        m &= !motor;
    }
    let mut apply = |key: &str, bits: u8| {
        if let Some(v) = get(tags, key) {
            if denies(v) {
                m &= !bits;
            } else if allows(v) {
                m |= bits;
            }
        }
    };
    apply("motor_vehicle", motor);
    apply("motorcar", Mode::CAR_BIT);
    apply("motorcycle", Mode::MOTORBIKE_BIT);
    apply("foot", Mode::FOOT_BIT);
    if let Some(v) = get(tags, "motorcycle") {
        *stats.entry(format!("tag.motorcycle={v}")).or_default() += 1;
    }

    let width_dm = get(tags, "width")
        .and_then(parse_metres)
        .map_or(0, |w| (w * 10.0).round() as u16);
    if width_dm != 0 && width_dm < CAR_MIN_WIDTH_DM {
        m &= !Mode::CAR_BIT;
    }
```

Đây là những giả định có chủ đích của lab, ghi ở đầu `tags.rs` để đối chiếu: đường `motorway` chỉ cho ô tô (lab coi xe máy bị cấm đường cao tốc), đường đi bộ chỉ cho người đi bộ, `cycleway` không ai trong ba profile đi được trừ khi có thẻ riêng cho phép, như `foot=yes`. Giá trị lạ không nằm trong danh sách cho phép hay cấm thì bị bỏ qua. Ở dữ liệu HCM, thẻ `motorcycle` có `yes` 5.294 way, `no` 922 way, và cả giá trị bẩn: `yes|yes` 2 way và `|yes|yes` 11 way, những giá trị mà parser không nhận ra.

Đường một chiều là chỗ xe máy khác ô tô. Lab tính hai hướng đi riêng, mỗi hướng dò từ thẻ riêng tới thẻ chung rồi mới tới mặc định; người đi bộ luôn hai chiều.

`examples/toi-noi-location/rust/crates/road-graph/src/tags.rs`

```rust
    // Một chiều: mặc định theo loại đường rồi thẻ chung rồi thẻ riêng từng phương tiện.
    let roundabout = matches!(get(tags, "junction"), Some("roundabout" | "circular"));
    let default_dir = if roundabout || matches!(class, Motorway | MotorwayLink) {
        Dir::Forward
    } else {
        Dir::Both
    };
    let dir_for = |specific: &[&str]| -> Dir {
        specific
            .iter()
            .chain(["oneway:vehicle", "oneway"].iter())
            .find_map(|k| get(tags, k).and_then(parse_dir))
            .unwrap_or(default_dir)
    };
    let car_dir = dir_for(&["oneway:motorcar", "oneway:motor_vehicle"]);
    let moto_dir = dir_for(&["oneway:motorcycle", "oneway:motor_vehicle"]);
```

Kết quả: **8.310** way một chiều với ô tô, **8.105** với xe máy, **205** way khác nhau giữa hai loại (thẻ `oneway:motorcycle=no` xuất hiện ở 204 way). Hai profile cần có đồ thị riêng: **17.513 cạnh (5,5%)** xe máy đi được mà ô tô không, và 69.260 cạnh (21,7%) không đủ cả ba profile.

## Tách way thành cạnh và xếp CSR

Một way OSM có thể dài hàng km và đi qua nhiều ngã tư, nên không thể coi cả way là một cạnh. Lab coi một nút là **nút giao** nếu nó là đầu mút của một đoạn way hoặc xuất hiện từ hai chỗ trở lên (đầu mút tính 2, nút giữa tính 1, tổng từ 2 trở lên là nút giao). Cạnh là đoạn giữa hai nút giao liên tiếp; các nút còn lại trở thành điểm hình dạng, lưu riêng để vẽ và để snap.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 450" role="img" aria-label="Hình 1. Một way OSM có sáu nút được tách thành các cạnh giữa các nút giao; đồ thị bốn nút được xếp thành mảng first_out, head và twin của CSR."><defs><marker id="location-osm-pbf-do-thi-duong-bo-xe-may-o-to-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs><rect x="20" y="20" width="420" height="415" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36.0" y="46.0" class="t-b">Way OSM tách thành cạnh</text><path d="M60.0 240.0 L120.0 206.0 L180.0 224.0 L250.0 178.0" class="f-none s-phys" stroke-width="3.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M250.0 178.0 L330.0 222.0 L400.0 196.0" class="f-none s-warn" stroke-width="3.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M250.0 92.0 L250.0 178.0" class="f-none s-pk" stroke-width="3.5" stroke-linejoin="round" stroke-linecap="round"/><path d="M250.0 178.0 L250.0 300.0" class="f-none s-pk" stroke-width="3.5" stroke-linejoin="round" stroke-linecap="round"/><circle cx="60.0" cy="240.0" r="7" class="f-pk s-pk" stroke-width="1.5"/><circle cx="120.0" cy="206.0" r="5" class="f-bg s-muted" stroke-width="2"/><circle cx="180.0" cy="224.0" r="5" class="f-bg s-muted" stroke-width="2"/><circle cx="250.0" cy="178.0" r="7" class="f-pk s-pk" stroke-width="1.5"/><circle cx="330.0" cy="222.0" r="5" class="f-bg s-muted" stroke-width="2"/><circle cx="400.0" cy="196.0" r="7" class="f-pk s-pk" stroke-width="1.5"/><circle cx="250.0" cy="92.0" r="7" class="f-pk s-pk" stroke-width="1.5"/><circle cx="250.0" cy="300.0" r="7" class="f-pk s-pk" stroke-width="1.5"/><text x="60.0" y="266.0" class="t-mono" text-anchor="middle">n1</text><text x="120.0" y="188.0" class="t-mono" text-anchor="middle">n2</text><text x="180.0" y="248.0" class="t-mono" text-anchor="middle">n3</text><text x="270.0" y="172.0" class="t-mono">n4</text><text x="330.0" y="246.0" class="t-mono" text-anchor="middle">n5</text><text x="400.0" y="220.0" class="t-mono" text-anchor="middle">n6</text><text x="250.0" y="76.0" class="t-mono" text-anchor="middle">m1</text><text x="250.0" y="326.0" class="t-mono" text-anchor="middle">m3</text><circle cx="52.0" cy="352.0" r="7" class="f-pk s-pk" stroke-width="1.5"/><text x="66.0" y="356.0" class="t-small">nút giao: thành một nút của CSR</text><circle cx="52.0" cy="372.0" r="5" class="f-bg s-muted" stroke-width="2"/><text x="66.0" y="376.0" class="t-small">điểm hình dạng: vào cột shape, không thành nút</text><text x="36.0" y="410.0" class="t-small">Way thứ nhất là n1…n6. Way thứ hai là m1, n4, m3,</text><text x="36.0" y="425.0" class="t-small">cắt way thứ nhất tại n4.</text><rect x="460" y="20" width="400" height="415" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="476.0" y="46.0" class="t-b">CSR của một đồ thị bốn nút</text><path d="M535.4 105.7 L604.6 86.3" class="f-none s-pk" stroke-width="2" marker-start="url(#location-osm-pbf-do-thi-duong-bo-xe-may-o-to-1-a)" marker-end="url(#location-osm-pbf-do-thi-duong-bo-xe-may-o-to-1-a)"/><path d="M534.8 116.2 L605.2 145.8" class="f-none s-pk" stroke-width="2" marker-start="url(#location-osm-pbf-do-thi-duong-bo-xe-may-o-to-1-a)" marker-end="url(#location-osm-pbf-do-thi-duong-bo-xe-may-o-to-1-a)"/><path d="M620.0 98.0 L620.0 136.0" class="f-none s-pk" stroke-width="2" marker-end="url(#location-osm-pbf-do-thi-duong-bo-xe-may-o-to-1-a)"/><path d="M636.0 152.0 L724.0 152.0" class="f-none s-pk" stroke-width="2" marker-start="url(#location-osm-pbf-do-thi-duong-bo-xe-may-o-to-1-a)" marker-end="url(#location-osm-pbf-do-thi-duong-bo-xe-may-o-to-1-a)"/><circle cx="520.0" cy="110.0" r="15" class="f-pk-soft s-pk" stroke-width="2"/><text x="520.0" y="114.0" class="t-b" text-anchor="middle">0</text><circle cx="620.0" cy="82.0" r="15" class="f-pk-soft s-pk" stroke-width="2"/><text x="620.0" y="86.0" class="t-b" text-anchor="middle">1</text><circle cx="620.0" cy="152.0" r="15" class="f-pk-soft s-pk" stroke-width="2"/><text x="620.0" y="156.0" class="t-b" text-anchor="middle">2</text><circle cx="740.0" cy="152.0" r="15" class="f-pk-soft s-pk" stroke-width="2"/><text x="740.0" y="156.0" class="t-b" text-anchor="middle">3</text><text x="628.0" y="121.0" class="t-small">một chiều</text><text x="476.0" y="204.0" class="t-small">first_out (nút 0…4, dài = số nút + 1)</text><rect x="476" y="212" width="40" height="26" class="f-surface s-line" stroke-width="1.5"/><text x="496.0" y="230.0" class="t-mono" text-anchor="middle">0</text><text x="496.0" y="252.0" class="t-small" text-anchor="middle">0</text><rect x="516" y="212" width="40" height="26" class="f-surface s-line" stroke-width="1.5"/><text x="536.0" y="230.0" class="t-mono" text-anchor="middle">2</text><text x="536.0" y="252.0" class="t-small" text-anchor="middle">1</text><rect x="556" y="212" width="40" height="26" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="576.0" y="230.0" class="t-mono" text-anchor="middle">4</text><text x="576.0" y="252.0" class="t-small" text-anchor="middle">2</text><rect x="596" y="212" width="40" height="26" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="616.0" y="230.0" class="t-mono" text-anchor="middle">6</text><text x="616.0" y="252.0" class="t-small" text-anchor="middle">3</text><rect x="636" y="212" width="40" height="26" class="f-surface s-line" stroke-width="1.5"/><text x="656.0" y="230.0" class="t-mono" text-anchor="middle">7</text><text x="656.0" y="252.0" class="t-small" text-anchor="middle">4</text><text x="476.0" y="274.0" class="t-small">head (cạnh e0…e6, nút đích)</text><rect x="476" y="282" width="40" height="26" class="f-surface s-line" stroke-width="1.5"/><text x="496.0" y="300.0" class="t-mono" text-anchor="middle">1</text><text x="496.0" y="322.0" class="t-small" text-anchor="middle">e0</text><rect x="516" y="282" width="40" height="26" class="f-surface s-line" stroke-width="1.5"/><text x="536.0" y="300.0" class="t-mono" text-anchor="middle">2</text><text x="536.0" y="322.0" class="t-small" text-anchor="middle">e1</text><rect x="556" y="282" width="40" height="26" class="f-surface s-line" stroke-width="1.5"/><text x="576.0" y="300.0" class="t-mono" text-anchor="middle">0</text><text x="576.0" y="322.0" class="t-small" text-anchor="middle">e2</text><rect x="596" y="282" width="40" height="26" class="f-surface s-line" stroke-width="1.5"/><text x="616.0" y="300.0" class="t-mono" text-anchor="middle">2</text><text x="616.0" y="322.0" class="t-small" text-anchor="middle">e3</text><rect x="636" y="282" width="40" height="26" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="656.0" y="300.0" class="t-mono" text-anchor="middle">0</text><text x="656.0" y="322.0" class="t-small" text-anchor="middle">e4</text><rect x="676" y="282" width="40" height="26" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="696.0" y="300.0" class="t-mono" text-anchor="middle">3</text><text x="696.0" y="322.0" class="t-small" text-anchor="middle">e5</text><rect x="716" y="282" width="40" height="26" class="f-surface s-line" stroke-width="1.5"/><text x="736.0" y="300.0" class="t-mono" text-anchor="middle">2</text><text x="736.0" y="322.0" class="t-small" text-anchor="middle">e6</text><text x="476.0" y="344.0" class="t-small">twin (cạnh cùng đoạn, chiều ngược)</text><rect x="476" y="352" width="40" height="26" class="f-surface s-line" stroke-width="1.5"/><text x="496.0" y="370.0" class="t-mono" text-anchor="middle">2</text><text x="496.0" y="392.0" class="t-small" text-anchor="middle">e0</text><rect x="516" y="352" width="40" height="26" class="f-surface s-line" stroke-width="1.5"/><text x="536.0" y="370.0" class="t-mono" text-anchor="middle">4</text><text x="536.0" y="392.0" class="t-small" text-anchor="middle">e1</text><rect x="556" y="352" width="40" height="26" class="f-surface s-line" stroke-width="1.5"/><text x="576.0" y="370.0" class="t-mono" text-anchor="middle">0</text><text x="576.0" y="392.0" class="t-small" text-anchor="middle">e2</text><rect x="596" y="352" width="40" height="26" class="f-surface s-line" stroke-width="1.5"/><text x="616.0" y="370.0" class="t-mono" text-anchor="middle">–</text><text x="616.0" y="392.0" class="t-small" text-anchor="middle">e3</text><rect x="636" y="352" width="40" height="26" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="656.0" y="370.0" class="t-mono" text-anchor="middle">1</text><text x="656.0" y="392.0" class="t-small" text-anchor="middle">e4</text><rect x="676" y="352" width="40" height="26" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="696.0" y="370.0" class="t-mono" text-anchor="middle">6</text><text x="696.0" y="392.0" class="t-small" text-anchor="middle">e5</text><rect x="716" y="352" width="40" height="26" class="f-surface s-line" stroke-width="1.5"/><text x="736.0" y="370.0" class="t-mono" text-anchor="middle">5</text><text x="736.0" y="392.0" class="t-small" text-anchor="middle">e6</text><text x="476.0" y="410.0" class="t-small">Ra khỏi nút 2: first_out[2]..first_out[3] = 4..6,</text><text x="476.0" y="425.0" class="t-small">tức cạnh e4, e5, đến nút 0 và nút 3.</text></svg>
</div>
<figcaption>Hình 1. Trái: way n1…n6 cắt way m1, n4, m3 tại n4, nên n4 trở thành nút giao cùng các đầu mút n1, n6, m1, m3; n2, n3, n5 trở thành điểm hình dạng. Phải: đồ thị bốn nút xếp thành ba mảng của CSR. Hình minh hoạ cơ chế, số trong mảng khớp với đồ thị vẽ bên trên.</figcaption>
</figure>

`examples/toi-noi-location/rust/crates/road-graph/src/builder.rs`

```rust
    // 3. Nút giao: đầu mút của đoạn, hoặc nút có mặt từ hai lần trở lên.
    let mut uses: FxHashMap<i64, u8> = FxHashMap::default();
    for (_, _, r) in &runs {
        for (i, n) in r.iter().enumerate() {
            let add = if i == 0 || i + 1 == r.len() { 2 } else { 1 };
            let e = uses.entry(*n).or_insert(0);
            *e = e.saturating_add(add);
        }
    }
```

`examples/toi-noi-location/rust/crates/road-graph/src/builder.rs`

```rust
    for (ai, way_id, r) in &runs {
        let a = &attrs_table[*ai];
        let mut seg_start = 0usize;
        for i in 1..r.len() {
            let is_junction = uses.get(&r[i]).copied().unwrap_or(0) >= 2;
            if !is_junction && i + 1 != r.len() {
                continue;
            }
```

Danh sách cạnh được xếp thành **CSR** (compressed sparse row): các cạnh sắp theo nút đầu, mảng `first_out` cho biết cạnh ra khỏi nút `n` nằm ở đoạn `first_out[n]..first_out[n+1]`. Mọi thuộc tính khác (độ dài, mặt nạ profile, loại đường, `twin`) là các mảng cột song song, dùng chung chỉ số cạnh. `twin` trỏ tới cạnh cùng đoạn đường theo chiều ngược lại, và là `NONE` nếu đường một chiều.

`examples/toi-noi-location/rust/crates/road-graph/src/model.rs`

```rust
#[derive(Debug, Clone, Default)]
pub struct Graph {
    pub node_ll: Vec<LatLngE7>,
    pub node_osm: Vec<i64>,
    /// `first_out[n]..first_out[n+1]` là các cạnh ra khỏi nút `n`. Dài `node_count + 1`.
    pub first_out: Vec<u32>,
    pub tail: Vec<NodeId>,
    pub head: Vec<NodeId>,
```

`examples/toi-noi-location/rust/crates/road-graph/src/model.rs`

```rust
    /// Chỉ số các cạnh ra khỏi nút `n`.
    pub fn out_edges(&self, n: NodeId) -> Range<u32> {
        self.first_out[n as usize]..self.first_out[n as usize + 1]
    }
```

Duyệt các cạnh ra khỏi một nút là đọc một đoạn liền của mảng, không phải đuổi theo con trỏ. Kết quả trên HCM: **126.699 nút, 319.074 cạnh có hướng**, 99,8% cạnh có `twin`, 151.182 điểm hình dạng. Độ dài cạnh: trung vị 35,8 m, p90 97,7 m, p99 229,0 m, dài nhất 2.197,2 m (một đoạn đường dài không có nút giao nào). 216 cạnh ngắn hơn 1 m, ngắn nhất 0,20 m: dấu vết của những nút OSM đặt sát nhau.

## Chất lượng dữ liệu hẻm ở TP.HCM: thẻ nào có, thẻ nào thiếu

Lab đếm theo way đã mô hình hoá (79.630 trên 80.629 way chạm bbox; 999 way bị bỏ vì là `area`, loại `highway` chưa mô hình hoá, hoặc không profile nào đi được). Bức tranh như sau:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Tỉ lệ way có từng thẻ trong dữ liệu OpenStreetMap quanh TP.HCM: tên đường 57,8 phần trăm, thẻ service 62,5 trong số way service, hướng một chiều 11,9, số làn 12,1, tốc độ tối đa 5,1, bề rộng 0,1; hẻm có bề rộng 0,1 phần trăm."><text x="20.0" y="28.0" class="t-b">Tỉ lệ way mô hình hoá có thẻ (đường cơ giới: 72.534 way; hẻm service=alley: 22.365 way)</text><text x="20.0" y="65.0">có name (đường cơ giới)</text><rect x="320" y="53" width="196.52" height="16" class="f-phys" stroke-width="0"/><text x="524.5" y="66.0" class="t-mono">57,8%  41.911 way</text><text x="20.0" y="97.0">có thẻ service=* (highway=service)</text><rect x="320" y="85" width="212.5" height="16" class="f-phys" stroke-width="0"/><text x="540.5" y="98.0" class="t-mono">62,5%  23.008 / 36.816</text><text x="20.0" y="129.0">có lanes (đường cơ giới)</text><rect x="320" y="117" width="41.14" height="16" class="f-phys" stroke-width="0"/><text x="369.1" y="130.0" class="t-mono">12,1%  8.807 way</text><text x="20.0" y="161.0">có thẻ oneway (đường cơ giới)</text><rect x="320" y="149" width="40.46" height="16" class="f-phys" stroke-width="0"/><text x="368.5" y="162.0" class="t-mono">11,9%  8.662 way</text><text x="20.0" y="193.0">có maxspeed (đường cơ giới)</text><rect x="320" y="181" width="17.34" height="16" class="f-phys" stroke-width="0"/><text x="345.3" y="194.0" class="t-mono">5,1%  3.674 way</text><text x="20.0" y="225.0">có width (đường cơ giới)</text><rect x="320" y="213" width="2" height="16" class="f-phys" stroke-width="0"/><text x="330.0" y="226.0" class="t-mono">0,1%  66 way</text><text x="20.0" y="257.0">có width (hẻm service=alley)</text><rect x="320" y="245" width="2" height="16" class="f-phys" stroke-width="0"/><text x="330.0" y="258.0" class="t-mono">0,1%  26 / 22.365</text><path d="M320.0 44.0 L320.0 272.0" class="f-none s-line" stroke-width="1"/><text x="320.0" y="290.0" class="t-small" text-anchor="middle">0%</text><text x="660.0" y="290.0" class="t-small" text-anchor="middle">100%</text><text x="860.0" y="290.0" class="t-small" text-anchor="end">© OpenStreetMap contributors</text></svg>
</div>
<figcaption>Hình 2. Tỉ lệ way có từng thẻ trong dữ liệu OSM quanh TP.HCM, tính từ kết quả chạy của <code>loc06_osm_do_thi</code>. © OpenStreetMap contributors (ODbL).</figcaption>
</figure>

- **Chiều rộng gần như không có trong dữ liệu.** «Hẻm» ở đây là way gắn `service=alley`; hẻm gắn `residential` hay `service` không kèm `service=alley` không nằm trong 22.365. Chỉ 26 trong số đó (0,1%) và 66 trên 72.534 đường cơ giới (0,1%) có `width` đọc được. Một `width` ghi sai định dạng cũng bị tính là thiếu, nên 26 là cận dưới. Quy tắc «ô tô không vào hẻm dưới 3 m» gần như không có dữ liệu để chạy.
- **`service` đông hơn `residential`.** `highway=service` có 36.816 way, còn `residential` 26.023. Đường «dịch vụ» trong dữ liệu này phần lớn là hẻm và đường nội bộ; 62,5% way `service` có thẻ `service=*` để phân biệt.
- **Thẻ tốc độ, làn hầu như không có:** `maxspeed` ở 5,1%, `lanes` ở 12,1%, `oneway` ở 11,9% đường cơ giới; `name` ở 57,8%. Tốc độ theo loại đường (bài 24) sẽ phải dựa vào `highway`, không dựa được vào `maxspeed`.
- **Không thể dựa vào tên để suy ra loại đường.** Một số way mang tên «Hẻm …» lại gắn `highway=secondary` (xem bài 07), nên không thể suy hẻm từ tên.
- **Dữ liệu cấm rẽ có nhưng chưa dùng được hết.** Trong 1.283 relation chạm đồ thị, lab giữ 1.262: 980 giải được thành cạnh vào và ra, 12 không khớp, 270 có `via` là một way (chưa giải), và 784 là cấm theo giờ (`restriction:conditional`). Cổng `entrance=*` có 178 nút, 130 trong đó chỉ ghi `yes`. Cả hai loại dữ liệu này được giữ nguyên thẻ gốc cho các bài sau.

Cũng có chi tiết bẩn khiến parser phải sửa: `kind` của một số relation theo giờ ghi dạng `no_left_turn @ (06:00-22:00)` hoặc dính liền `no_left_turn@(...)`, nên lab tách ở dấu `@`.

## Thành phần liên thông: cái gì nằm ngoài, cái gì kẹt bên trong

Một đồ thị đường bộ cắt từ bản đồ thật luôn có «đảo»: đoạn đường nằm riêng vì dữ liệu thiếu nút nối, hay bị bbox cắt lìa. Lab dùng union-find (`weak_components`) để tìm thành phần liên thông vô hướng và chỉ giữ thành phần lớn nhất.

`examples/toi-noi-location/rust/crates/road-graph/src/components.rs`

```rust
/// Thành phần liên thông vô hướng của danh sách cạnh `(u, v)` trên `n` nút.
pub fn weak_components(n: usize, edges: &[(u32, u32)]) -> Components {
    let mut d = Dsu::new(n);
    for &(u, v) in edges {
        d.union(u, v);
    }
    let mut compact = vec![u32::MAX; n];
    let mut count = 0u32;
    let mut label = vec![0u32; n];
    for (i, slot) in label.iter_mut().enumerate() {
        let r = d.find(i as u32) as usize;
        if compact[r] == u32::MAX {
            compact[r] = count;
            count += 1;
        }
        *slot = compact[r];
    }
    Components {
        label,
        count: count as usize,
    }
}
```

Có **329** thành phần; thành phần lớn nhất có 126.699 nút, thành phần hạng hai chỉ có 431 nút, rồi 153, 114, 57. Có 1.890 nút (1,5% của 128.589) nằm ngoài thành phần lớn nhất và bị bỏ.

Nhưng liên thông vô hướng chưa đủ: một nút có thể đi tới được mà không có đường quay về. Đường một chiều, vùng `access=private` và cắt theo bbox đều tạo ra «túi» như thế, và mỗi profile một khác. Lab chạy thuật toán Tarjan cài theo dạng lặp trên đồ thị con của từng profile để tìm thành phần liên thông mạnh (SCC) lớn nhất:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 3. Phần nút và cạnh nằm trong thành phần liên thông mạnh lớn nhất theo từng profile: ô tô 88,0 phần trăm nút và 78,9 phần trăm cạnh, xe máy 91,9 và 83,9, đi bộ 99,8 và 99,8."><text x="20.0" y="28.0" class="t-b">Thành phần liên thông mạnh lớn nhất (đồ thị HCM đã lọc: 126.699 nút, 319.074 cạnh)</text><text x="20.0" y="76.0" class="t-b">ô tô</text><rect x="120" y="56" width="457.6" height="16" class="f-pk" stroke-width="0"/><text x="585.6" y="69.0" class="t-mono">nút 88,0%</text><rect x="120" y="78" width="410.28" height="16" class="f-phys" stroke-width="0"/><text x="538.3" y="91.0" class="t-mono">cạnh 78,9%</text><text x="860.0" y="76.0" class="t-small" text-anchor="end">908 SCC &gt; 1 nút</text><text x="20.0" y="132.0" class="t-b">xe máy</text><rect x="120" y="112" width="477.88" height="16" class="f-pk" stroke-width="0"/><text x="605.9" y="125.0" class="t-mono">nút 91,9%</text><rect x="120" y="134" width="436.28" height="16" class="f-phys" stroke-width="0"/><text x="564.3" y="147.0" class="t-mono">cạnh 83,9%</text><text x="860.0" y="132.0" class="t-small" text-anchor="end">149 SCC &gt; 1 nút</text><text x="20.0" y="188.0" class="t-b">đi bộ</text><rect x="120" y="168" width="518.96" height="16" class="f-pk" stroke-width="0"/><text x="647.0" y="181.0" class="t-mono">nút 99,8%</text><rect x="120" y="190" width="518.96" height="16" class="f-phys" stroke-width="0"/><text x="647.0" y="203.0" class="t-mono">cạnh 99,8%</text><text x="860.0" y="188.0" class="t-small" text-anchor="end">19 SCC &gt; 1 nút</text><path d="M120.0 48.0 L120.0 208.0" class="f-none s-line" stroke-width="1"/><text x="120.0" y="224.0" class="t-small" text-anchor="middle">0%</text><text x="640.0" y="224.0" class="t-small" text-anchor="middle">100%</text><text x="860.0" y="224.0" class="t-small" text-anchor="end">© OpenStreetMap contributors</text></svg>
</div>
<figcaption>Hình 3. Phần nút và cạnh nằm trong SCC lớn nhất của từng profile, sau khi đã bỏ các đảo; số SCC lớn hơn một nút ghi bên phải. © OpenStreetMap contributors (ODbL).</figcaption>
</figure>

**Ô tô chỉ có 88,0% số nút (78,9% số cạnh đi được) trong SCC lớn nhất; xe máy 91,9% (83,9%); đi bộ 99,8% (99,8%).** Ô tô có 908 SCC lớn hơn một nút, xe máy 149, đi bộ 19. Hệ quả cho Tới Nơi: nếu điểm đón snap vào cạnh nằm ngoài SCC lớn nhất của profile, ít nhất một chiều tìm đường có thể không có lời giải, dù bản đồ trông liên thông. Lab chưa tách riêng nguyên nhân nào đóng góp bao nhiêu; việc đó thuộc phần định tuyến.

## Chạy ví dụ và kiểm chứng

Cần `data/vietnam-latest.osm.pbf` (chạy `bash examples/toi-noi-location/data/fetch.sh`), từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo build -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc06_osm_do_thi
/usr/bin/time -l examples/toi-noi-location/rust/target/release/loc06_osm_do_thi
```

Cấu hình đo: Apple M1 Max 10 lõi, `--release`, Rust 1.98.1, bbox lat 10,70–10,86 / lng 106,60–106,78. Kết quả của lần chạy cuối (rút gọn):

```text
đọc PBF (3 lượt, song song): 6.95 s
biên dịch (tách cạnh, liên thông, CSR, cấm rẽ): 0.23 s
ghi examples/toi-noi-location/data/hcm-graph.bin : 18.4 MB trong 0.01 s
nạp lại file: 9 ms, 126699 nút, 319074 cạnh (khớp: true)
  nút = 126699
  cạnh có hướng = 319074
  thành phần hạng 1: 126699 nút
  ô tô         251687     78.9%       111441      88.0%       908
  xe máy       267578     83.9%       116430      91.9%       149
  đi bộ        318446     99.8%       126485      99.8%        19
  hẻm (service=alley) = 22365, trong đó có width = 26 (0.1%)
```

**Các số liệu thời gian được đo khi máy đang chịu tải nặng** (tải trung bình 1 phút từ 45 đến hơn 300 trên 10 lõi), nên chỉ đọc tương đối. Số đếm ở trên (nút, cạnh, tỉ lệ) không phụ thuộc thời gian và tái lập từng chữ số. Trong cùng lần chạy cuối: đọc PBF 6,95 s, biên dịch 0,23 s, tức đọc PBF chiếm phần lớn; nạp lại file đồ thị mất 9 ms, nhanh hơn dựng lại từ PBF cỡ 800 lần (tính từ 7,18 s và 9 ms). Các lần chạy khác ghi lại đọc PBF từ 4,66 s (lúc phát triển, máy ít tải) đến 27,62 s (máy tải nặng nhất). Bộ nhớ đỉnh khi biên dịch khoảng 618–640 MB (`/usr/bin/time -l`), còn các cột của đồ thị chỉ ước tính 17,5 MB.

Không có PBF thì chạy được trên đồ thị tổng hợp: `cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc06_osm_do_thi -- --fixture`. Test của `road-graph` (32 test, trong đó có `narrow_alley_blocks_car_only` và `motorbike_may_differ_from_car` ở `tags.rs`) chạy cùng `bash examples/toi-noi-location/verify-p01.sh`.

## Giới hạn của ví dụ

- **Dữ liệu là một ảnh chụp.** Mọi số liệu về chất lượng thẻ lấy từ bản trích Geofabrik trên máy viết; OSM thay đổi liên tục, và bài không so sánh các thời điểm.
- **Quy tắc quyền đi là giả định.** Cấm xe máy trên `motorway`, `access=destination` coi như được phép, không có `width` thì cho ô tô vào: đều là lựa chọn của lab, chưa đối chiếu với biển báo ngoài đời.
- **Cấm rẽ mới được đọc, chưa dùng.** 270 relation `via` là way chưa giải; 784 cấm theo giờ giữ thô. Tìm đường chưa biết đến chúng (bài 23).
- **Cổng toà nhà (`entrance=*`) mới lưu thô**, chưa gắn vào đồ thị (bài 37).
- **Tốc độ chưa có.** Cạnh mới có độ dài, chưa có thời gian đi; bài 24 và bài 27 lo phần đó.
- **Một thành phố, một bbox.** Chưa thử bbox lớn hơn hay nhiều thành phố; RSS 618–640 MB khi biên dịch cho thấy cần đọc PBF theo luồng nếu mở rộng.

## Bài tập

**Bài 1.** Way `highway=residential`, `oneway=yes`, `oneway:motorcycle=no`. Với từng chiều của way, profile nào đi được?

<details><summary>Lời giải bài 1</summary>
<p>Loại đường <code>residential</code> cho cả ba bit. <code>oneway=yes</code> đặt hướng chung là chiều xuôi, nhưng <code>oneway:motorcycle=no</code> cụ thể hơn nên xe máy hai chiều. Chiều xuôi: ô tô, xe máy, đi bộ. Chiều ngược: xe máy và đi bộ, ô tô không. Kiểu way này chính là loại mà 205 way trong dữ liệu HCM có hướng một chiều khác nhau giữa ô tô và xe máy.</p>
</details>

**Bài 2.** Cho `first_out = [0, 2, 4, 6, 7]` và `head = [1, 2, 0, 2, 0, 3, 2]` như Hình 1. Liệt kê các cạnh ra khỏi nút 2 và nút đến của chúng. Cạnh nào là một chiều?

<details><summary>Lời giải bài 2</summary>
<p>Cạnh ra khỏi nút 2 nằm ở <code>first_out[2]..first_out[3]</code>, tức 4..6: cạnh e4 và e5, có <code>head</code> là 0 và 3. Cạnh một chiều là cạnh có <code>twin = NONE</code>: chỉ e3 (nút 1 tới nút 2). Chiều ngược lại, từ nút 2 tới nút 1, không tồn tại.</p>
</details>

**Bài 3.** Vì sao SCC lớn nhất của ô tô nhỏ hơn của xe máy, còn của đi bộ gần như phủ hết?

<details><summary>Lời giải bài 3</summary>
<p>Đi bộ luôn hai chiều và hầu như không bị <code>access</code> chặn, nên gần như mọi nút liên thông mạnh với nhau (99,8%). Xe máy chịu đường một chiều và một số hạn chế. Ô tô chịu thêm giới hạn <code>width</code> dưới 3,0 m và <code>access</code> kiểu cấm, nên có nhiều «túi» hơn (908 SCC lớn hơn một nút). Lab chưa tách riêng đóng góp của từng nguyên nhân, nên đây là lời giải thích khả dĩ chứ chưa phải kết luận đo được.</p>
</details>

Bài trước: [bài 05](/articles/location-geometry-crate-geo-chieu-diem-polyline-bearing). Bài tiếp: [bài 07](/articles/location-rstar-r-tree-snap-diem-ve-duong).

## Tài liệu gốc

- [osmpbf trên docs.rs](https://docs.rs/osmpbf): crate đọc PBF của lab.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công «© OpenStreetMap contributors» và ODbL.
- [Trang Geofabrik cho Việt Nam](https://download.geofabrik.de/asia/vietnam.html): nguồn bản trích PBF.
- [OSM Wiki: Key:highway](https://wiki.openstreetmap.org/wiki/Key:highway) và [Key:oneway](https://wiki.openstreetmap.org/wiki/Key:oneway): ý nghĩa các thẻ.
- R. E. Tarjan, «Depth-first search and linear graph algorithms», SIAM Journal on Computing, 1972: thuật toán SCC mà `strong_components` cài lại theo dạng lặp.
