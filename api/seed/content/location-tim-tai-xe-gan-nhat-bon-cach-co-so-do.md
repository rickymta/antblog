---
slug: location-tim-tai-xe-gan-nhat-bon-cach-co-so-do
title: "Tìm tài xế gần nhất: quét, R-tree, H3 và Redis GEO có số đo"
summary: "Bài 09: dựng tải 6.250 cập nhật/giây và 70 truy vấn/giây theo hồ sơ Tới Nơi, đo quét toàn bộ, R-tree, ô H3 và Redis GEO (Docker, loopback); PostGIS chưa chạy nên chỉ đối chiếu khái niệm."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, rstar, h3, redis, geo, benchmark, nearest-neighbor]
language: vi
---

Giờ tan tầm, mưa. Mỗi giây có khoảng 70 khách bấm đặt xe, và mỗi khách cần danh sách 10 tài xế gần nhất để chào chuyến. Cùng lúc, 25.000 tài xế đang trực tuyến gửi vị trí mỗi 4 giây một lần, tức 6.250 cập nhật mỗi giây. Hai con số này là hồ sơ của [Tới Nơi](/articles/du-an-mau-toi-noi): một luồng ghi liên tục và một luồng đọc theo khoảng cách. Câu hỏi của bài: giữ «vị trí mới nhất của mọi tài xế» trong cấu trúc nào để cả hai luồng chạy đủ nhanh, và nhanh hơn bao nhiêu so với cách ngây thơ nhất là duyệt hết?

«Gần nhất» ở đây là gần theo đường chim bay, chỉ là bước lọc thô; km rỗng thật phải tính theo đường đi (các bài sau). Đây là bài 09 của lộ trình **Nền tảng định vị** qua Tới Nơi. Bài đặt R-tree của [bài 07](/articles/location-rstar-r-tree-snap-diem-ve-duong) cạnh lưới H3 của [bài 08](/articles/location-h3-geohash-chi-muc-luoi-k-ring), quét toàn bộ và Redis GEO. **Mọi số đo là trên dữ liệu mô phỏng**; vị trí tài xế do lab sinh, không có GPS người thật. **Redis chạy trong Docker, qua mạng loopback của Docker Desktop**, nên số của nó chỉ có giá trị tương đối. **PostGIS không được chạy** trong lab; phần đó chỉ là đối chiếu khái niệm, không có số.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Duyệt tuyến tính cả 25.000 tài xế cho mỗi trong 70 truy vấn mỗi giây, cộng 6.250 cập nhật mỗi giây, một luồng CPU: dùng bao nhiêu phần trăm một lõi?</p>
<details><summary>Xem đáp án</summary><p>Rất ít: cả một giây tải tốn <b>2,7 ms</b> CPU ở lần chạy lúc phát triển (máy ít tải), và 3,0–11,0 ms khi máy chịu tải nặng, tức dưới 2% một lõi. Quét toàn bộ <b>không cần chỉ mục</b> ở đúng quy mô hồ sơ; chỉ mục có giá trị khi số tài xế hoặc tần suất truy vấn tăng, như phần ×10 bên dưới.</p></details>
</div>

## Dựng tải theo hồ sơ và cách đo

Lab sinh tải giống hồ sơ: 25.000 tài xế, mỗi người báo vị trí 4 giây một lần nên mỗi giây có N/4 tài xế báo (quay vòng), và 70 truy vấn «10 tài xế gần nhất» mỗi giây (k = 10). Quy mô 2.500 và 250.000 (×0,1 và ×10) co dãn cả hai theo tỉ lệ. Vị trí: 70% quanh 8 điểm nóng (σ 1,5 km), 30% rải đều trong bbox lat 10,70–10,86, lng 106,60–106,78; tài xế đi 0–40 km/h và đổi hướng ngẫu nhiên σ 20° mỗi lần báo.

`examples/toi-noi-location/rust/labs/src/bin/loc09_gan_nhat.rs`

```rust
/// 70% quanh 8 điểm nóng (σ ≈ 1,5 km mỗi trục), 30% rải đều trong bbox.
fn sample_point(rng: &mut Rand) -> LatLng {
    loop {
        let p = if rng.f64() < 0.7 {
            let (la, ln) = HOTSPOTS[rng.usize(HOTSPOTS.len())];
            let c = LatLng::try_new(la, ln).expect("hợp lệ");
            let east = destination(c, 90.0, rng.gauss() * 1500.0);
            destination(east, 0.0, rng.gauss() * 1500.0)
        } else {
            LatLng::try_new(rng.range(BBOX[0], BBOX[1]), rng.range(BBOX[2], BBOX[3]))
                .expect("hợp lệ")
        };
        if in_bbox(p) {
            return p.to_e7().to_latlng();
        }
    }
}
```

Mọi cách giữ vị trí dùng chung một giao diện, để lab đo cùng một việc:

`examples/toi-noi-location/rust/crates/driver-store/src/lib.rs`

```rust
/// Giao diện chung của các chỉ mục trong bộ nhớ.
pub trait DriverIndex {
    /// Tên hiển thị trong bảng đo.
    fn name(&self) -> String;
    /// Thêm hoặc cập nhật vị trí.
    fn upsert(&mut self, id: DriverId, pos: LatLng);
    /// Xóa tài xế (offline). Trả `true` nếu có.
    fn remove(&mut self, id: DriverId) -> bool;
    /// `k` tài xế gần `q` nhất, gần trước.
    fn nearest(&self, q: LatLng, k: usize) -> Vec<Neighbor>;
    /// Áp dụng các thay đổi còn treo (chỉ [`RTreeBulk`] dùng); các cách khác không làm gì.
    fn flush(&mut self) {}
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
```

Mỗi cách được đo qua bốn pha, một luồng, không khóa, không tranh chấp: nạp ban đầu; 8 giây mô phỏng chỉ cập nhật (đo ns mỗi cập nhật); chỉ truy vấn trên trạng thái cuối (2.000 điểm hỏi, đo p50 và p99 từng truy vấn, 1.000 điểm đầu được đối chiếu với quét toàn bộ); và 8 giây mô phỏng xen kẽ như thật.

`examples/toi-noi-location/rust/labs/src/bin/loc09_gan_nhat.rs`

```rust
    // Pha 3: xen kẽ như hồ sơ thật: mỗi giây các cập nhật rồi các truy vấn, đo thời gian cả giây.
    let t = Instant::now();
    let mut sink = 0usize;
    for s in half..w.updates.len() {
        for &(id, p) in &w.updates[s] {
            idx.upsert(id, p);
        }
        idx.flush();
        for &q in &w.queries[s] {
            sink += idx.nearest(q, K).len();
        }
    }
    std::hint::black_box(sink);
    let sec_ms = t.elapsed().as_secs_f64() * 1e3 / (w.updates.len() - half) as f64;
```

Cột **«1 giây tải»** là thời gian CPU của một luồng để xử lý trọn một giây tải (cập nhật, `flush` rồi các truy vấn). Dưới 1.000 ms nghĩa là kịp; 10 ms là 1% một lõi. Ngân sách mỗi cập nhật nếu chỉ để cập nhật là 1 s chia 6.250, tức 160 µs.

**Điều kiện đo.** Apple M1 Max, `--release`, seed 20261106. Máy dùng chung với nhiều phiên làm việc khác, **tải trung bình 1 phút từ khoảng 120 đến 180 trong lô đo chính** (theo `uptime`, ghi trong `results/p01.md`). Mọi số thời gian bị phóng đại và dao động; số đếm, tỉ lệ khớp thì không. Vì vậy bài chỉ so tương đối trong cùng một lần chạy và ghi khoảng giá trị qua nhiều lần chạy.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Hình 1. Bốn cách giữ vị trí tài xế: quét toàn bộ, R-tree và ô H3 nằm trong tiến trình Rust; Redis GEO nằm ngoài tiến trình nên mỗi lệnh GEOADD hoặc GEOSEARCH là một vòng mạng."><text x="20.0" y="28.0" class="t-b">Mỗi giây ở quy mô hồ sơ: 6.250 cập nhật vị trí và 70 truy vấn «10 tài xế gần nhất»</text><text x="190.0" y="56.0" class="t-small">cập nhật (ghi vị trí mới)</text><text x="400.0" y="56.0" class="t-small">truy vấn (k = 10)</text><text x="20.0" y="104.0" class="t-b">Quét toàn bộ</text><rect x="180" y="70" width="190" height="58" rx="8" class="f-surface s-phys" stroke-width="1.5"/><text x="192.0" y="94.0" class="t-small">ghi một ô nhớ trong mảng</text><text x="192.0" y="112.0" class="t-small">chỉ số theo mã tài xế</text><rect x="390" y="70" width="190" height="58" rx="8" class="f-surface s-phys" stroke-width="1.5"/><text x="402.0" y="94.0" class="t-small">duyệt cả 25.000 vị trí</text><text x="402.0" y="112.0" class="t-small">giữ 10 gần nhất</text><text x="600.0" y="104.0" class="t-small">trong tiến trình, làm mốc đúng</text><text x="20.0" y="178.0" class="t-b">R-tree (rstar)</text><rect x="180" y="144" width="190" height="58" rx="8" class="f-surface s-phys" stroke-width="1.5"/><text x="192.0" y="168.0" class="t-small">xóa rồi chèn, hoặc</text><text x="192.0" y="186.0" class="t-small">dựng lại theo lô mỗi giây</text><rect x="390" y="144" width="190" height="58" rx="8" class="f-surface s-phys" stroke-width="1.5"/><text x="402.0" y="168.0" class="t-small">nearest_neighbor_iter</text><text x="402.0" y="186.0" class="t-small">lấy 10 phần tử đầu</text><text x="600.0" y="178.0" class="t-small">trong tiến trình, cây phải sửa</text><text x="20.0" y="252.0" class="t-b">Ô H3</text><rect x="180" y="218" width="190" height="58" rx="8" class="f-surface s-phys" stroke-width="1.5"/><text x="192.0" y="242.0" class="t-small">đổi giỏ khi đổi ô</text><text x="192.0" y="260.0" class="t-small">(khoảng 7% số lần cập nhật)</text><rect x="390" y="218" width="190" height="58" rx="8" class="f-surface s-phys" stroke-width="1.5"/><text x="402.0" y="242.0" class="t-small">mở rộng vòng ô quanh</text><text x="402.0" y="260.0" class="t-small">điểm hỏi tới khi đủ</text><text x="600.0" y="252.0" class="t-small">trong tiến trình, giỏ theo ô</text><text x="20.0" y="326.0" class="t-b">Redis GEO</text><rect x="180" y="292" width="190" height="58" rx="8" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="192.0" y="316.0" class="t-small">GEOADD: một vòng mạng</text><text x="192.0" y="334.0" class="t-small">mỗi lệnh, hoặc pipeline 500</text><rect x="390" y="292" width="190" height="58" rx="8" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="402.0" y="316.0" class="t-small">GEOSEARCH bán kính 3000 m,</text><text x="402.0" y="334.0" class="t-small">ASC COUNT 10: một vòng mạng</text><text x="600.0" y="326.0" class="t-small">ngoài tiến trình, qua Docker</text><path d="M20.0 284.0 L860.0 284.0" class="f-none s-warn dash" stroke-width="1.5"/><text x="860.0" y="279.0" class="t-small" text-anchor="end">ranh giới tiến trình</text><text x="600.0" y="366.0" class="t-small">Mỗi lệnh tốn tối thiểu một PING:</text><text x="600.0" y="381.0" class="t-small">354,8 µs đến 3.450,3 µs tùy lần đo.</text></svg>
</div>
<figcaption>Hình 1. Bốn cách giữ vị trí tài xế. Ba cách đầu nằm trong tiến trình Rust; Redis nằm ngoài, nên mỗi lệnh là một vòng mạng, dù chỉ qua loopback của Docker.</figcaption>
</figure>

## Bốn cách và giá của việc sửa vị trí

**Quét toàn bộ** giữ một mảng phẳng chỉ số theo mã tài xế; cập nhật chỉ ghi một ô nhớ, truy vấn duyệt hết và giữ 10 gần nhất.

`examples/toi-noi-location/rust/crates/driver-store/src/scan.rs`

```rust
    fn upsert(&mut self, id: DriverId, pos: LatLng) {
        let slot = &mut self.pos[id as usize];
        if slot.is_absent() {
            self.count += 1;
        }
        *slot = pos.to_e7();
    }
```

**R-tree** (`rstar`, trên mặt phẳng mét của bài 07) có hai biến thể; chính tài liệu của `rstar` nhận xét R-tree hợp nhất khi có nhiều truy vấn và ít lần chèn, đúng chỗ mà tải của tài xế (ghi liên tục) thử nó. Bản «xóa + chèn» sửa cây ngay: tìm và xóa mục cũ rồi chèn mục mới. Bản «dựng lại theo lô» chỉ ghi vị trí, rồi mỗi giây dựng lại cây bằng `bulk_load`; đổi lại, truy vấn giữa hai lần `flush` thấy trạng thái cũ tối đa một giây.

`examples/toi-noi-location/rust/crates/driver-store/src/rtree.rs`

```rust
    fn upsert(&mut self, id: DriverId, pos: LatLng) {
        let i = id as usize;
        if !self.pos[i].is_absent() {
            self.tree.remove(&Entry::new(self.xy[i], id));
        }
        let e7 = pos.to_e7();
        self.pos[i] = e7;
        self.xy[i] = self.proj.to_xy(e7.to_latlng());
        self.tree.insert(Entry::new(self.xy[i], id));
    }
```

`examples/toi-noi-location/rust/crates/driver-store/src/rtree.rs`

```rust
    fn flush(&mut self) {
        let entries: Vec<Entry> = self
            .pos
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.is_absent())
            .map(|(id, p)| Entry::new(self.proj.to_xy(p.to_latlng()), id as DriverId))
            .collect();
        self.tree = RTree::bulk_load(entries);
    }
```

**Ô H3** giữ một danh sách tài xế cho mỗi ô; cập nhật không đổi ô thì chỉ ghi vị trí mới, đổi ô thì chuyển giỏ.

`examples/toi-noi-location/rust/crates/driver-store/src/h3_buckets.rs`

```rust
    fn upsert(&mut self, id: DriverId, pos: LatLng) {
        let i = id as usize;
        let e7 = pos.to_e7();
        let new_cell = cell_of(e7.to_latlng(), self.res);
        match self.cell[i] {
            Some(old) if old == new_cell => {}
            Some(old) => {
                self.detach(id, old);
                self.buckets.entry(new_cell).or_default().push(id);
                self.cell[i] = Some(new_cell);
                self.cell_changes += 1;
            }
            None => {
                self.buckets.entry(new_cell).or_default().push(id);
                self.cell[i] = Some(new_cell);
                self.count += 1;
            }
        }
        self.pos[i] = e7;
    }
```

Chi phí cập nhật ở quy mô hồ sơ, đơn vị ns mỗi cập nhật, qua năm lần chạy (lần phát triển không có `uptime`; bốn lần sau chạy khi máy tải nặng, tải 1 phút lần lượt 122–178, 122–138, 138–168, 53–136):

| Cách | Lúc phát triển | Lần 1 | Lần 2 | Lần 3 | Lần 4 |
|---|---:|---:|---:|---:|---:|
| Quét toàn bộ | 1 | 1,6 | 11,3 | 3,3 | 3,9 |
| R-tree xóa + chèn | 706 | 2.648,7 | 2.174,3 | 2.631,9 | 777,4 |
| R-tree dựng lại theo lô | 509 | 657,7 | 660,8 | 1.195,9 | 507,8 |
| H3 res 8 | 291 | 371,7 | 334,6 | 930,5 | 280,4 |
| H3 res 9 | 300 | 403,6 | 393,3 | 2.744,6 | 298,6 |
| Redis GEO, 1 lệnh/vòng mạng | 782.309 | 3.182.126,5 | 3.143.026,0 | 2.963.746,1 | 817.925,4 |
| Redis GEO, pipeline 500/lô | 4.853 | 7.024,2 | 24.858,0 | 22.535,2 | 4.544,0 |

Quét chỉ ghi một ô nhớ nên rẻ nhất. R-tree xóa + chèn **đắt hơn H3 từ 2,4 đến 7,1 lần** ở bốn trong năm lần chạy; ở lần 3, H3 res 8 vẫn rẻ hơn (930,5 ns, 2,8 lần) nhưng H3 res 9 đo ra 2.744,6 ns, không rẻ hơn R-tree (2.631,9 ns): nhiễu của máy tải nặng đủ lớn để đảo thứ tự, nên đừng đọc con số riêng lẻ. Nguyên nhân hợp lý của chiều hướng chung là chỉ khoảng 7,1–7,4% cập nhật làm tài xế đổi ô res 9, còn cây phải xóa và chèn mọi lần. R-tree dựng lại theo lô rẻ hơn mỗi cập nhật (508–1.196 ns, đã gồm dựng lại mỗi giây) nhưng đổi bằng dữ liệu cũ tối đa một giây; vị trí vốn đã cũ tới 4 giây theo chu kỳ báo, nên một giây trễ thêm có thể chấp nhận được. Các cách trong bộ nhớ đều nhỏ hơn nhiều ngân sách 160 µs; Redis từng lệnh thì không.

## Số đo ở quy mô hồ sơ và ở ×10

Bảng dưới là **lần 1** của lô đo chính, N = 25.000 (6.250 cập nhật/giây, 70 truy vấn/giây), một luồng. Cột «khớp quét» so khoảng cách của 10 tài xế trả về với quét toàn bộ (lệch dưới 1 m); riêng Redis so tập mã tài xế trong bán kính 3.000 m.

| Cách | Nạp (ms) | Cập nhật (ns) | Hỏi p50 (µs) | Hỏi p99 (µs) | 1 giây tải (ms) | Khớp quét |
|---|---:|---:|---:|---:|---:|---:|
| Quét toàn bộ | 0,1 | 1,6 | 37,9 | 2.970,2 | 9,4 | 100,0% |
| R-tree xóa + chèn | 70,7 | 2.648,7 | 2,6 | 23,7 | 9,6 | 100,0% |
| R-tree dựng lại theo lô | 3,4 | 657,7 | 2,0 | 3,3 | 5,1 | 100,0% |
| H3 res 8 | 13,3 | 371,7 | 8,9 | 20,5 | 2,5 | 100,0% |
| H3 res 9 | 10,4 | 403,6 | 3,7 | 7,9 | 2,2 | 100,0% |
| Redis GEO, 1 lệnh/vòng mạng | 256,1 | 3.182.126,5 | 3.130,3 | 27.787,2 | 19.807,1 | 99,0% |
| Redis GEO, pipeline 500/lô | 157,6 | 7.024,2 | 3.104,2 | 14.985,6 | 380,4 | 99,0% |

**Đọc bảng.** Ở đúng quy mô hồ sơ, mọi cách trong bộ nhớ đều thừa sức: p50 truy vấn của quét ổn định quanh 37 µs (37,0–37,9 qua cả năm lần chạy), R-tree 1,8–2,8 µs, H3 res 9 3,6–4,1 µs, H3 res 8 8,8–9,3 µs; một giây tải tốn 2,2–4,3 ms lúc phát triển và tối đa khoảng 20 ms khi máy tải nặng, tức dưới 2% một lõi. Cột p99 của quét (2.970,2 µs) là nhiễu của máy tải, không phải bản chất: các lần chạy khác cho 505,5, 213,1 và 67,4 µs, lúc phát triển 45,6 µs. Quét toàn bộ **không cần chỉ mục** nếu chỉ phục vụ một thành phố ở tải này.

Chỉ mục có giá trị khi tải tăng. Ở ×10 (250.000 tài xế, 62.500 cập nhật/giây, 700 truy vấn/giây), cùng lần chạy:

| Cách | Hỏi p50 (µs) | 1 giây tải (ms) | Lúc phát triển (ms) |
|---|---:|---:|---:|
| Quét toàn bộ | 341,5 | 357,0 | 269,0 |
| R-tree xóa + chèn | 4,0 | 241,8 | 85,9 |
| R-tree dựng lại theo lô | 3,0 | 68,6 | 45,1 |
| H3 res 8 | 81,0 | 215,0 | 65,7 |
| H3 res 9 | 18,0 | 99,5 | 28,1 |

Quét tốn 269–357 ms mỗi giây tải, tức 27–36% một lõi, và truy vấn p50 chậm hơn R-tree cỡ 100 lần (341,5 µs so với 3,0–4,0 µs). H3 res 8 kém res 9 ở đây vì mỗi ô chứa nhiều tài xế hơn: chia đều, 250.000 tài xế trên 349,8 km² là khoảng 527 người mỗi ô res 8 và 75 người mỗi ô res 9, chưa tính việc tài xế dồn về điểm nóng. **Độ phân giải H3 phải khớp mật độ.**

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 434" role="img" aria-label="Hình 2. CPU cần cho một giây tải, thang logarit: ở 25.000 tài xế mọi cách trong bộ nhớ dưới 10 mili giây, Redis từng lệnh 19.807,1 mili giây vượt xa một giây; ở 250.000 tài xế quét tốn 357,0 mili giây còn R-tree dựng theo lô 68,6."><text x="20.0" y="28.0" class="t-b">Một giây tải = 6.250 cập nhật + 70 truy vấn ở N = 25.000 (và ×10 ở N = 250.000), ms, thang logarit</text><path d="M250.0 52.0 L250.0 350.0" class="f-none s-line" stroke-width="1"/><text x="250.0" y="366.0" class="t-small" text-anchor="middle">1</text><path d="M362.0 52.0 L362.0 350.0" class="f-none s-line" stroke-width="1"/><text x="362.0" y="366.0" class="t-small" text-anchor="middle">10</text><path d="M474.0 52.0 L474.0 350.0" class="f-none s-line" stroke-width="1"/><text x="474.0" y="366.0" class="t-small" text-anchor="middle">100</text><path d="M586.0 52.0 L586.0 350.0" class="f-none s-line" stroke-width="1"/><text x="586.0" y="366.0" class="t-small" text-anchor="middle">1.000</text><path d="M698.0 52.0 L698.0 350.0" class="f-none s-line" stroke-width="1"/><text x="698.0" y="366.0" class="t-small" text-anchor="middle">10.000</text><text x="228.0" y="366.0" class="t-small" text-anchor="end">ms</text><path d="M586.0 52.0 L586.0 350.0" class="f-none s-warn dash" stroke-width="1.5"/><text x="592.0" y="48.0" class="t-small">ngân sách: 1 giây CPU mỗi giây</text><text x="20.0" y="78.0">Quét toàn bộ</text><rect x="250" y="60" width="108.99" height="12" class="f-phys" stroke-width="0"/><text x="365.0" y="71.0" class="t-mono">9,4</text><rect x="250" y="76" width="285.899" height="12" class="f-pk" stroke-width="0"/><text x="541.9" y="87.0" class="t-mono">357,0</text><text x="20.0" y="120.0">R-tree (xóa + chèn)</text><rect x="250" y="102" width="110.014" height="12" class="f-phys" stroke-width="0"/><text x="366.0" y="113.0" class="t-mono">9,6</text><rect x="250" y="118" width="266.947" height="12" class="f-pk" stroke-width="0"/><text x="522.9" y="129.0" class="t-mono">241,8</text><text x="20.0" y="162.0">R-tree (dựng lại theo lô)</text><rect x="250" y="144" width="79.2479" height="12" class="f-phys" stroke-width="0"/><text x="335.2" y="155.0" class="t-mono">5,1</text><rect x="250" y="160" width="205.668" height="12" class="f-pk" stroke-width="0"/><text x="461.7" y="171.0" class="t-mono">68,6</text><text x="20.0" y="204.0">Ô H3 độ phân giải 8</text><rect x="250" y="186" width="44.5693" height="12" class="f-phys" stroke-width="0"/><text x="300.6" y="197.0" class="t-mono">2,5</text><rect x="250" y="202" width="261.233" height="12" class="f-pk" stroke-width="0"/><text x="517.2" y="213.0" class="t-mono">215,0</text><text x="20.0" y="246.0">Ô H3 độ phân giải 9</text><rect x="250" y="228" width="38.3513" height="12" class="f-phys" stroke-width="0"/><text x="294.4" y="239.0" class="t-mono">2,2</text><rect x="250" y="244" width="223.756" height="12" class="f-pk" stroke-width="0"/><text x="479.8" y="255.0" class="t-mono">99,5</text><text x="20.0" y="288.0">Redis GEO, 1 lệnh/vòng mạng</text><rect x="250" y="270" width="481.244" height="12" class="f-phys" stroke-width="0"/><text x="737.2" y="281.0" class="t-mono">19.807,1</text><text x="20.0" y="330.0">Redis GEO, pipeline 500/lô</text><rect x="250" y="312" width="288.987" height="12" class="f-phys" stroke-width="0"/><text x="545.0" y="323.0" class="t-mono">380,4</text><rect x="250" y="378" width="14" height="10" class="f-phys" stroke-width="0"/><text x="270.0" y="388.0" class="t-small">25.000 tài xế (hồ sơ)</text><rect x="430" y="378" width="14" height="10" class="f-pk" stroke-width="0"/><text x="450.0" y="388.0" class="t-small">250.000 tài xế (×10, Redis không đo)</text><text x="860.0" y="388.0" class="t-small" text-anchor="end">máy đang tải nặng</text></svg>
</div>
<figcaption>Hình 2. CPU cho một giây tải, thang logarit, lần 1 của lô đo chính (máy tải nặng). Redis từng lệnh vượt xa đường 1 giây; ở ×10 chỉ đo quét và bốn cách trong bộ nhớ (không đo Redis).</figcaption>
</figure>

## Redis GEO: vòng mạng quyết định tất cả

Redis GEO lưu vị trí trong một sorted set với geohash 52 bit làm score: `GEOADD` ghi, `GEOSEARCH ... BYRADIUS ... ASC COUNT k` tìm. Điều khác biệt nằm ở vị trí của nó: ngoài tiến trình. Thứ tự tham số của Redis là **kinh độ trước, vĩ độ sau**, ngược với `LatLng`, nên lab đổi bằng `to_lnglat` ở một chỗ duy nhất.

`examples/toi-noi-location/rust/crates/driver-store/src/redis_geo.rs`

```rust
    /// Thêm/cập nhật một tài xế: một vòng mạng.
    pub fn upsert(&mut self, id: DriverId, pos: LatLng) -> Result<(), StoreError> {
        let [lng, lat] = pos.to_lnglat();
        redis::cmd("GEOADD")
            .arg(&self.key)
            .arg(lng)
            .arg(lat)
            .arg(id)
            .exec(&mut self.con)?;
        Ok(())
    }
```

`examples/toi-noi-location/rust/crates/driver-store/src/redis_geo.rs`

```rust
    /// `k` tài xế gần nhất trong bán kính `radius_m` (`GEOSEARCH ... BYRADIUS ... ASC COUNT k WITHDIST`).
    /// Bán kính là bắt buộc của lệnh: nếu trong bán kính ít hơn `k` người thì chỉ trả bấy nhiêu.
    pub fn nearest(
        &mut self,
        q: LatLng,
        k: usize,
        radius_m: u32,
    ) -> Result<Vec<Neighbor>, StoreError> {
        let [lng, lat] = q.to_lnglat();
        let rows: Vec<(String, f64)> = redis::cmd("GEOSEARCH")
            .arg(&self.key)
            .arg("FROMLONLAT")
            .arg(lng)
            .arg(lat)
            .arg("BYRADIUS")
            .arg(radius_m)
            .arg("m")
            .arg("ASC")
            .arg("COUNT")
            .arg(k)
            .arg("WITHDIST")
            .query(&mut self.con)?;
        Ok(rows
            .into_iter()
            .filter_map(|(m, d)| {
                Some(Neighbor {
                    id: m.parse().ok()?,
                    dist_m: d,
                })
            })
            .collect())
    }
```

Container Redis 7.4.12 (`redis:7`) chạy bằng `docker compose` ở `127.0.0.1:6391`, qua máy ảo Linux của Docker Desktop. Chi phí tối thiểu của **mọi** lệnh được đo bằng `PING`: trung vị 354,8 µs lúc phát triển, 1.218,4 µs ở lần 1, và tới 3.450,3 µs ở lần 3 (p99 từ 1.135,2 µs tới 27.426,8 µs). Hệ quả:

- **Một lệnh một vòng mạng không kịp.** `GEOADD` lẻ tốn 782–3.182 µs mỗi lệnh, nên một giây tải (6.250 cập nhật và 70 truy vấn) mất **3,4 đến 20,4 giây**, quá một giây, không theo kịp trên một kết nối.
- **Pipeline cứu phần cập nhật.** Gộp 500 lệnh mỗi lô đưa cập nhật về 4,5–25 µs mỗi lệnh, một giây tải còn 125–732 ms.
- **Truy vấn vẫn chậm.** `GEOSEARCH` p50 từ 1,3 đến 6,5 ms, so với 2–3 µs của R-tree: chậm hơn cỡ nghìn lần, ví dụ 680 lần ở lần phát triển (1.283,9 µs so với 1,9 µs) và khoảng 1.600 lần ở lần 1 (3.130,3 µs so với 2,0 µs); tỉ lệ tính từ bảng. Chi phí tăng theo bán kính tìm: p50 ở bán kính 300, 1.000, 3.000 m là 458,6, 662,3 và 1.569,8 µs lúc phát triển, 2.061,5, 2.217,2 và 2.935,5 µs ở lần 1. Tài liệu của Redis ghi độ phức tạp của `GEOSEARCH` là O(N + log M), với N là số phần tử trong hộp bao lưới quanh hình tìm, nên `COUNT 10` không rẻ khi vùng đông.
- **Kết quả khớp quét 99,0%**, không phải 100%, tức 10 trên 1.000 truy vấn được đối chiếu. Lab in thêm khoảng cách giữa người thứ 10 và thứ 11 theo quét ở đúng 10 truy vấn đó: trung vị 0,107 m, lớn nhất 0,437 m. Cả 10 ca đều là **gần hòa** ở ranh giới tốp 10, không phải Redis bỏ sót người gần. Điều này khớp với việc Redis lưu vị trí trong geohash 52 bit, mỗi ô cỡ 0,6 m × 0,3 m ở vĩ độ này, và dùng công thức, bán kính khác; lab không tách riêng phần của từng yếu tố. Bộ nhớ Redis tăng 2,0–2,3 MB cho 25.000 tài xế.

Những điều này **không** nói Redis dở. Nó đo một thứ cụ thể: giá của một vòng mạng, trên Docker Desktop, qua loopback. Redis ngoài Docker, hoặc kết nối Unix socket, sẽ nhanh hơn; điều đó chưa đo. Redis còn giải quyết việc khác mà ba cách trong bộ nhớ không làm được, như chia sẻ trạng thái giữa nhiều tiến trình; bài này chưa đo phần đó, và để lại cho bài về kho vị trí mới nhất.

## PostGIS: đối chiếu khái niệm, chưa đo

**PostGIS không chạy trong lab** (image chưa có sẵn trên máy viết, và kéo image mới từ Docker Hub có thể treo), nên không có con số nào cho nó và bài không ước lượng thay. Phần này chỉ để đặt nó vào bản đồ khái niệm.

PostGIS đặt chỉ mục không gian GiST (một biến thể của R-tree) lên cột hình học, hỗ trợ lọc theo bán kính (`ST_DWithin`) và sắp xếp theo khoảng cách bằng toán tử `<->` kèm `LIMIT`. Về truy vấn, đó là cùng họ với R-tree của bài 07. Khác biệt nằm ở **cập nhật**: mỗi lần sửa một dòng, PostgreSQL ghi một phiên bản dòng mới, và khi cột có chỉ mục đổi giá trị thì cả mục chỉ mục cũng phải đổi, đều đi qua nhật ký ghi trước (WAL). Giả thuyết của loạt bài, **chưa được đo**, là ở 6.250 lần sửa mỗi giây một bảng «vị trí mới nhất» như vậy là gánh nặng không cần thiết cho dữ liệu chỉ cần tươi vài giây. Hồ sơ Tới Nơi coi bảng `driver_locations` là bảng chỉ chèn (6.250 dòng mỗi giây, chủ yếu đọc vài phút gần nhất), không phải bảng «vị trí mới nhất» để sửa tại chỗ. PostGIS vẫn là lựa chọn tự nhiên cho các truy vấn không gian phức tạp trên dữ liệu bền như vùng cấm đón.

## Chạy ví dụ và kiểm chứng

Cần Redis ở `127.0.0.1:6391`; từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
docker compose -f examples/toi-noi-location/infra/docker-compose.yml up -d --wait redis
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc09_gan_nhat
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc09_gan_nhat -- --scales 25000
```

Lệnh cuối chỉ chạy quy mô theo hồ sơ, để xem độ dao động giữa các lần. Thêm `--no-redis` để bỏ Redis một cách tường minh; không có cờ này mà Redis thiếu thì lab báo lỗi và dừng với mã 2, không bỏ qua âm thầm (đã xảy ra thật trong lô đo đầu, khi container bị một tiến trình khác gỡ). Cấu hình: seed 20261106, mỗi pha 8 giây mô phỏng, k = 10. Output rút gọn của lần 1, máy tải nặng:

```text
Redis 7.4.12 tại redis://127.0.0.1:6391/: PING trung vị 1218.4 µs, p99 11340.5 µs (2000 lần): chi phí tối thiểu của MỌI lệnh, vì đi qua mạng loopback/Docker
---- N = 25000 tài xế (×1.0 so với hồ sơ): 6250 cập nhật/giây, 70 truy vấn/giây ----
quét toàn bộ                         0.1          1.6       37.9     2970.2      172.5              9.4    100.0%  
R-tree (xóa + chèn)                 70.7       2648.7        2.6       23.7       13.0              9.6    100.0%  
ô H3 độ phân giải 9                 10.4        403.6        3.7        7.9        4.0              2.2    100.0%  
Redis GEO, 1 lệnh/vòng mạng        256.1    3182126.5     3130.3    27787.2     5128.0          19807.1     99.0%  (Redis used_memory tăng 2.3 MB sau nạp, khớp = cùng tập id, bán kính 3000 m)
PostGIS: không chạy (lab không có PostGIS; image chưa có sẵn), nên không có số đo. Không ước lượng thay.
```

Số **thời gian** đo khi máy tải nặng, nên chỉ so tương đối trong cùng lần chạy; số **đếm** (khớp quét, tỉ lệ đổi ô) tái lập. Test của `driver-store` (6 test) kiểm cả ba cách trong bộ nhớ khớp quét toàn bộ trên 3.000 điểm và 200 truy vấn ngẫu nhiên, và kiểm cập nhật, xóa. Test cần Redis thật chạy riêng: `cargo test --manifest-path examples/toi-noi-location/rust/Cargo.toml -p driver-store --features redis -- --ignored` (1 test, qua).

## Giới hạn của ví dụ

- **Một luồng, không khóa.** Chưa đo ghi và đọc cùng lúc từ nhiều luồng; một service thật sẽ cần khóa hoặc cấu trúc đọc nhiều ghi nhiều, và con số sẽ khác.
- **Không đo bộ nhớ trong tiến trình** của ba cách trong bộ nhớ; chỉ đo `used_memory` của Redis.
- **Khoảng cách chim bay, k cố định bằng 10**, tài xế đang trực tuyến mọi lúc; chưa có tài xế im lặng, TTL, hay lọc theo trạng thái nhận chuyến.
- **Redis qua Docker loopback.** Số của Redis không thay cho Redis riêng, socket Unix hay cụm.
- **PostGIS không chạy**; mọi phát biểu về nó là khái niệm hoặc giả thuyết, không phải kết quả.
- **Máy đo bị tải nặng**, số thời gian nhiễu; p99 đặc biệt không đáng tin.
- **Tải mô phỏng**: điểm nóng, tốc độ, hướng đi do lab chọn; không có GPS người thật.

## Bài tập

**Bài 1.** R-tree xóa + chèn tốn 2.648,7 ns mỗi cập nhật (lần 1). Nếu chỉ tính riêng việc cập nhật trên một lõi, nó đỡ nổi bao nhiêu cập nhật mỗi giây? So với 6.250.

<details><summary>Lời giải bài 1</summary>
<p>1 giây chia 2.648,7 ns là xấp xỉ 377.000 cập nhật mỗi giây cho một lõi, cỡ 60 lần nhu cầu 6.250, chưa tính truy vấn, <code>flush</code> và tranh chấp khóa. Đây là cận trên lý thuyết trên máy đo, trong lúc máy bị tải nặng; không phải năng lực của service.</p>
</details>

**Bài 2.** Vì sao R-tree dựng lại theo lô, dữ liệu cũ tối đa một giây, vẫn có thể chấp nhận cho việc chọn 10 tài xế gần nhất?

<details><summary>Lời giải bài 2</summary>
<p>Vị trí của mỗi tài xế vốn cũ tới 4 giây theo chu kỳ báo, và xe đi tối đa 40 km/h (khoảng 11 m/s) trong mô phỏng, nên một giây cũ thêm là cỡ 11 m ở tốc độ lớn nhất. Danh sách 10 tài xế chỉ là bước lọc thô trước khi tính đường đi thật. Đổi lại cách này rẻ hơn mỗi cập nhật (508–1.196 ns) và truy vấn nhanh nhất (p50 2,0 µs). Nó sẽ không phù hợp nếu cần thấy tài xế vừa nhận chuyến ngay lập tức, tức nhu cầu loại tài xế khỏi danh sách tức thì.</p>
</details>

**Bài 3.** Dùng «hỏi TB» và «cập nhật ns» của dòng Redis pipeline ở lần 1 để ước lượng một giây tải, rồi so với 380,4 ms đã đo.

<details><summary>Lời giải bài 3</summary>
<p>70 truy vấn nhân 3.890,7 µs là khoảng 272 ms; 6.250 cập nhật nhân 7.024,2 ns là khoảng 44 ms; tổng khoảng 316 ms, thấp hơn 380,4 ms đã đo khoảng 64 ms. Phần chênh <b>chưa được giải thích</b>: có thể do pha đo xen kẽ chịu nhiễu tải khác pha đo riêng từng việc, nhưng bài chưa kiểm. Ý nghĩa thực tế: ngay cả với pipeline, truy vấn mới là phần lớn chi phí của Redis.</p>
</details>

Bài trước: [bài 08](/articles/location-h3-geohash-chi-muc-luoi-k-ring). Bài tiếp: [bài 10](/articles/location-tong-hop-geo-core-kiem-thu-hinh-hoc).

## Tài liệu gốc

- [Lệnh GEOSEARCH của Redis](https://redis.io/docs/latest/commands/geosearch/) và [GEOADD](https://redis.io/docs/latest/commands/geoadd/): cú pháp và độ phức tạp.
- [redis trên docs.rs](https://docs.rs/redis): client Redis của lab.
- [rstar trên docs.rs](https://docs.rs/rstar) và [h3o trên docs.rs](https://docs.rs/h3o).
- [Hàm ST_DWithin của PostGIS](https://postgis.net/docs/ST_DWithin.html) và [toán tử KNN `<->`](https://postgis.net/docs/geometry_distance_knn.html): chỉ để đối chiếu khái niệm, bài này không chạy PostGIS.
