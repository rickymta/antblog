---
slug: location-kho-vi-tri-moi-nhat-h3-redis-geo-kafka
title: "Kho vị trí mới nhất: bộ nhớ theo ô H3, Redis GEO và Kafka"
summary: "Bài 18: kho vị trí mới nhất của 25.000 tài xế trong bộ nhớ theo ô H3, TTL cho tài xế im lặng, Redis làm bản phụ, khởi động lại từ Redis, Kafka hoặc để nguội (đầy lại sau 4 giây), giảm mẫu sang lịch sử."
category: system-design
level: intermediate
tags: [toi-noi, location, rust, h3, redis, kafka, ttl, synthetic-data]
language: vi
---

Giờ tan tầm, một khách đặt xe và matcher hỏi kho: «năm tài xế gần điểm đón nhất». Câu hỏi này phải được trả lời từ bộ nhớ trong vài micro-giây, vì có hàng chục truy vấn như vậy mỗi giây (hồ sơ tham chiếu ở bài 09 là 70). Nhưng kho có hai cách sai rất đời thường. Một: `geo-engine` vừa được triển khai lại và kho còn rỗng, nên suốt vài giây không có tài xế nào để ghép dù 25.000 người đang trực tuyến. Hai: nếu kho không có hạn, một tài xế vừa vào hầm hay tắt app cách đây hai phút vẫn nằm đầu danh sách «gần nhất», và khách được hứa một chiếc xe không ai nhìn thấy nữa.

Đây là bài 18 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài ghép các mảnh trước đó: hợp đồng `location.updates.v1` từ phần Go ([bài 12](/articles/location-go-ingest-websocket-batch-gps-tai-xe) tới [bài 14](/articles/location-validate-gps-timestamp-accuracy-gia-mao)), ô H3 của [bài 08](/articles/location-h3-geohash-chi-muc-luoi-k-ring) và bài toán «tìm tài xế gần nhất» của [bài 09](/articles/location-tim-tai-xe-gan-nhat-bon-cach-co-so-do). Mọi số đo là **trên dữ liệu mô phỏng**: 25.000 tài xế do lab sinh, một broker Kafka và một Redis chạy trên chính máy viết, không có mạng thật. Thời gian đo khi máy đang chịu tải nặng nên chỉ so sánh tương đối trong cùng một lần chạy.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>25.000 tài xế, mỗi người gửi một điểm mỗi 4 giây. <code>geo-engine</code> khởi động lại với kho rỗng và <b>không nạp lại gì</b> từ Redis hay Kafka. Sau bao lâu thì kho thấy đủ 25.000 tài xế?</p>
<details><summary>Xem đáp án</summary><p><b>4 giây</b>, đúng một chu kỳ báo cáo: 25% sau 1 giây (6.207 tài xế), 49% sau 2 giây, 75% sau 3 giây, 99,7% sau 4 giây (24.924) và đủ 25.000 sau 5 giây. Vì mỗi tài xế sẽ tự gửi điểm kế tiếp trong vòng 4 giây, kho tự đầy lại; nạp từ Redis (46,0 ms) hay phát lại từ Kafka (0,73 giây kể cả kết nối) chỉ rút ngắn khoảng trống đó.</p></details>
</div>

## Kho này giữ gì, và không giữ gì

Kho giữ **một vị trí mới nhất cho mỗi tài xế**, không phải lịch sử. Lịch sử có đường đi riêng (bài 20). Người dùng của kho là các bước phía sau: điểm điều phối của bài 36 đọc vị trí qua `get` và `nearest`, ghép chuyến của P05 hỏi `nearest`. Mỗi cập nhật là một `DriverLocationUpdated` trên topic `location.updates.v1` với key `driver_id`, nên mọi điểm của một tài xế về cùng một partition và giữ nguyên thứ tự.

Hai quyết định nhỏ ở mô hình dữ liệu có hệ quả lớn. Thứ nhất, `accuracy`, `speed` và `heading` là `Option`: «vắng» khác «0» và phải giữ nguyên khác biệt đó qua Kafka và Redis (test kiểm đúng điều này). Thứ hai, TTL tính theo `received_ms`, tức giờ ingest nhận lô theo **đồng hồ server**, không theo đồng hồ điện thoại; một điện thoại có đồng hồ lệch không làm tài xế bị coi là hết hạn hay còn tươi mãi, đúng điều bài 14 chuẩn bị ở tầng validator.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 440" role="img" aria-label="Hình 1. Đường đi của một cập nhật vị trí: Kafka, kho trong bộ nhớ theo ô H3 (nhiều mảnh khóa), bản phụ Redis qua hàng đợi giới hạn, bộ giảm mẫu sang lịch sử; ba cách khởi động lại."><defs><marker id="loc18-f1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs><rect x="20.0" y="90.0" width="170.0" height="130.0" rx="10" class="f-surface s-muted" stroke-width="1.5"/><text x="105.0" y="116.0" class="t-b" text-anchor="middle">Kafka</text><text x="105.0" y="136.0" class="t-mono" text-anchor="middle">location.updates.v1</text><text x="105.0" y="156.0" class="t-small" text-anchor="middle">key = driver_id</text><text x="105.0" y="172.0" class="t-small" text-anchor="middle">lab đo: 3 partition</text><text x="105.0" y="192.0" class="t-small" text-anchor="middle">assign cả 3, không commit</text><path d="M190.0 155.0 L250.0 155.0" class="f-none s-muted" stroke-width="2" marker-end="url(#loc18-f1-a)"/><rect x="250.0" y="110.0" width="150.0" height="90.0" rx="10" class="f-surface s-muted" stroke-width="1.5"/><text x="325.0" y="136.0" class="t-b" text-anchor="middle">Ingest.handle</text><text x="325.0" y="156.0" class="t-small" text-anchor="middle">bỏ điểm cũ đến sau,</text><text x="325.0" y="172.0" class="t-small" text-anchor="middle">đếm bản ghi hỏng</text><path d="M400.0 155.0 L450.0 155.0" class="f-none s-pk" stroke-width="3" marker-end="url(#loc18-f1-a)"/><rect x="450.0" y="60.0" width="250.0" height="190.0" rx="10" class="f-pk-soft s-pk" stroke-width="2"/><text x="575.0" y="84.0" class="t-b" text-anchor="middle">LatestStore (bộ nhớ)</text><rect x="466.0" y="98.0" width="218.0" height="60.0" rx="6" class="f-surface s-muted" stroke-width="1.5"/><text x="575.0" y="118.0" class="t-mono" text-anchor="middle">drivers: 64 mảnh Mutex</text><text x="575.0" y="136.0" class="t-small" text-anchor="middle">tài xế → ô, giờ, seq</text><text x="575.0" y="150.0" class="t-small" text-anchor="middle">một tài xế qua đúng một khóa</text><rect x="466.0" y="168.0" width="218.0" height="70.0" rx="6" class="f-surface s-muted" stroke-width="1.5"/><text x="575.0" y="188.0" class="t-mono" text-anchor="middle">cells: 64 mảnh RwLock</text><text x="575.0" y="206.0" class="t-small" text-anchor="middle">ô H3 độ phân giải 9 → danh sách</text><text x="575.0" y="222.0" class="t-small" text-anchor="middle">nearest chỉ lấy khóa đọc của ô</text><rect x="740.0" y="40.0" width="120.0" height="70.0" rx="10" class="f-surface s-muted" stroke-width="1.5"/><text x="800.0" y="66.0" class="t-b" text-anchor="middle">Truy vấn</text><text x="800.0" y="84.0" class="t-small" text-anchor="middle">nearest, get</text><text x="800.0" y="98.0" class="t-small" text-anchor="middle">(bài 35, 36)</text><path d="M700.0 100.0 L740.0 75.0" class="f-none s-pk" stroke-width="2" marker-end="url(#loc18-f1-a)"/><rect x="740.0" y="130.0" width="120.0" height="80.0" rx="10" class="f-surface s-muted" stroke-width="1.5"/><text x="800.0" y="156.0" class="t-b" text-anchor="middle">MirrorWriter</text><text x="800.0" y="174.0" class="t-small" text-anchor="middle">hàng đợi giới hạn</text><text x="800.0" y="188.0" class="t-small" text-anchor="middle">try_send, đầy thì bỏ</text><path d="M325.0 200.0 L325.0 285.0" class="f-none s-muted" stroke-width="1.5" marker-end="url(#loc18-f1-a)"/><path d="M700.0 170.0 L740.0 170.0" class="f-none s-muted" stroke-width="1.5" marker-end="url(#loc18-f1-a)"/><rect x="740.0" y="230.0" width="120.0" height="70.0" rx="10" class="f-surface s-muted" stroke-width="1.5"/><text x="800.0" y="256.0" class="t-b" text-anchor="middle">Redis (bản phụ)</text><text x="800.0" y="274.0" class="t-small" text-anchor="middle">GEO + HASH 46 byte</text><path d="M800.0 210.0 L800.0 230.0" class="f-none s-muted" stroke-width="1.5" marker-end="url(#loc18-f1-a)"/><rect x="250.0" y="285.0" width="150.0" height="70.0" rx="10" class="f-surface s-muted" stroke-width="1.5"/><text x="325.0" y="311.0" class="t-b" text-anchor="middle">Sampler</text><text x="325.0" y="329.0" class="t-small" text-anchor="middle">60 m, rẽ 30°, nhịp 60 s</text><text x="325.0" y="343.0" class="t-small" text-anchor="middle">→ lịch sử (Go, bài 20)</text><text x="450.0" y="292.0" class="t-b">Khởi động lại, kho rỗng thì nạp từ:</text><text x="450.0" y="314.0">Redis HGETALL: 46,0 ms cho 25.000 tài xế</text><text x="450.0" y="334.0">Kafka offsets_for_times(now − 34 s): 0,08 s + 0,65 s phát lại</text><text x="450.0" y="354.0">không nạp gì (Latest): đầy lại sau 4 s, một chu kỳ báo cáo</text><text x="20.0" y="392.0" class="t-small">Thứ tự cố ý: kho trong bộ nhớ trước (phục vụ truy vấn), Redis và lịch sử sau và không bao giờ chặn kho.</text><text x="20.0" y="410.0" class="t-small">Các số khởi động lại là lần chạy chính, đo khi máy chịu tải nặng; xem bảng ở phần chạy ví dụ.</text></svg>
</div>
<figcaption>Hình 1. Luồng của <code>location-store</code>. Ba nhánh sau kho (truy vấn, Redis, lịch sử) cố ý không chờ nhau; Redis chậm thì cập nhật bị bỏ và đếm, Kafka không bị chặn. Kafka không chia partition giữa các bản geo-engine: mỗi bản cần toàn bộ vị trí của thành phố.</figcaption>
</figure>

## Tiêu thụ topic: assign cả partition, không commit

Bình thường một consumer trong nhóm nhận một phần các partition. Kho thì khác: mỗi bản `geo-engine` cần **toàn bộ** vị trí của thành phố để trả lời «gần nhất», nên không dùng nhóm consumer chia partition. Mỗi bản `assign` tất cả partition, `group.id` chỉ là thủ tục cấu hình chứ không dùng để chia partition, và **không commit offset**. Khởi động lại thì đọc lại từ `now − TTL` bằng `offsets_for_times` (phép đo dùng mốc `now − 34 s`, phủ hết 8 vòng dữ liệu đã phát): kho chỉ cần các điểm còn trong TTL, nên phần phát lại rất ngắn. Cái giá là mỗi bản `geo-engine` đọc cả topic (băng thông đọc nhân với số bản); lab chưa đo cái giá này. `Ingest::handle` quyết định đường đi của một cập nhật:

Tệp `examples/toi-noi-location/rust/crates/location-store/src/ingest.rs` (dòng 50–67):

```rust
    /// Xử lý một cập nhật.
    pub fn handle(&mut self, u: &Update) -> Applied {
        self.counters.messages += 1;
        let r = self.store.apply(u);
        if r == Applied::Stale {
            self.counters.stale += 1;
            return r;
        }
        self.counters.applied += 1;
        if let Some(m) = &self.mirror {
            m.upsert(Arc::from(u.driver_id.as_str()), u.pos);
        }
        if let Some(why) = self.sampler.keep(u) {
            self.sink.write(u, why);
            self.counters.history_kept += 1;
        }
        r
    }
```

Thứ tự là cố ý. Kho trong bộ nhớ trước vì nó là đường nóng phục vụ truy vấn. Điểm cũ đến sau bị kho bỏ (`Applied::Stale`) thì **không** được ghi sang Redis hay lịch sử. Redis và lịch sử sau và không bao giờ chặn kho. Bản ghi hỏng (không giải mã được hoặc thiếu trường bắt buộc) bị bỏ nhưng được đếm; test Kafka kiểm với 300 bản hợp lệ và 2 bản thêm (một điểm cũ đến sau, một bản hỏng).

## Hai bảng, nhiều mảnh khóa: đọc nhiều ghi nhiều

Kho có hai bảng, mỗi bảng chia 64 mảnh khóa. `drivers`: tài xế → (ô hiện tại, giờ, seq), mỗi mảnh một `Mutex`; một tài xế luôn qua đúng một khóa nên cập nhật của cùng một tài xế được tuần tự hóa và phát hiện điểm cũ đến sau là chính xác. `cells`: ô H3 (độ phân giải 9, cạnh ô trung bình khoảng 200,8 m theo bảng độ dài cạnh của H3) → danh sách tài xế trong ô, mỗi mảnh một `RwLock`. Truy vấn gần nhất chỉ lấy khóa đọc của các ô, không đụng `drivers`.

Tệp `examples/toi-noi-location/rust/crates/location-store/src/store.rs` (dòng 180–218):

```rust
    pub fn apply(&self, u: &Update) -> Applied {
        let slot = self.registry.slot(&u.driver_id);
        let cell = cell_of(u.pos.latlng(), self.res);
        let entry = Entry { slot, pos: u.pos };
        let rec_new = DriverRec {
            cell,
            occurred_ms: u.pos.occurred_ms,
            received_ms: u.pos.received_ms,
            seq: u.pos.seq,
        };
        let mut shard = self.drivers[slot as usize % self.drivers.len()]
            .lock()
            .expect("drivers");
        self.applied.fetch_add(1, Ordering::Relaxed);
        match shard.get_mut(&slot) {
            Some(rec) => {
                let stale = u.pos.occurred_ms < rec.occurred_ms
                    || (u.pos.occurred_ms == rec.occurred_ms && u.pos.seq <= rec.seq);
                if stale {
                    self.stale.fetch_add(1, Ordering::Relaxed);
                    return Applied::Stale;
                }
                let moved = rec.cell != cell;
                self.put_in_cell(cell, entry);
                if moved {
                    self.take_from_cell(rec.cell, slot);
                    self.cell_moves.fetch_add(1, Ordering::Relaxed);
                }
                *rec = rec_new;
                Applied::Updated { moved_cell: moved }
            }
            None => {
                self.put_in_cell(cell, entry);
                shard.insert(slot, rec_new);
                self.inserted.fetch_add(1, Ordering::Relaxed);
                Applied::Inserted
            }
        }
    }
```

Khi tài xế đổi ô, `apply` **chèn vào ô mới trước, xóa khỏi ô cũ sau**, nên truy vấn không bao giờ mất tài xế; trong khoảnh khắc đó có thể thấy hai bản, và truy vấn gộp theo `slot` rồi giữ bản mới hơn. Thứ tự khóa luôn là `drivers` rồi `cells` và không bao giờ giữ hai khóa `cells` cùng lúc, nên không có vòng chờ khóa. Truy vấn mở rộng từng vòng ô quanh điểm hỏi và dừng khi vòng kế tiếp chắc chắn xa hơn tài xế thứ `k`:

Tệp `examples/toi-noi-location/rust/crates/location-store/src/store.rs` (dòng 257–292):

```rust
        let c0 = cell_of(q, self.res);
        let edge = edge_length_avg_m(self.res);
        let mut best: Vec<(f64, Entry)> = Vec::with_capacity(k + 1);
        let max_ring = ((max_radius_m / (edge * 1.2)).ceil() as u32 + 1).min(200);
        for r in 0..=max_ring {
            for cell in Self::ring(c0, r) {
                let m = self.cells_for(cell).read().expect("cells");
                let Some(v) = m.get(&cell) else { continue };
                for e in v {
                    if e.pos.received_ms + self.cfg.ttl_ms < now_ms {
                        continue;
                    }
                    let d = equirect_m(q, e.pos.latlng());
                    if d > max_radius_m {
                        continue;
                    }
                    if let Some(old) = best.iter_mut().find(|(_, o)| o.slot == e.slot) {
                        if e.pos.occurred_ms > old.1.pos.occurred_ms {
                            *old = (d, *e);
                        }
                        continue;
                    }
                    if best.len() == k && best.last().is_some_and(|l| d >= l.0) {
                        continue;
                    }
                    let at = best.partition_point(|b| b.0 <= d);
                    best.insert(at, (d, *e));
                    best.truncate(k);
                }
            }
            // mọi điểm ở ô cách >= r+1 bước cách điểm hỏi ít nhất (1,5·r − 0,5)·cạnh; nhân 0,8 cho độ méo của lưới
            let safe = (1.5 * f64::from(r) - 0.5).max(0.0) * edge * 0.8;
            if best.len() == k && best.last().is_some_and(|l| l.0 <= safe) {
                break;
            }
        }
```

Điều kiện dừng `safe` là heuristic có hệ số an toàn 0,8 cho độ méo của lưới, nên tính đúng của nó phải kiểm bằng cách so với vét cạn: test `nearest_matches_brute_force` (proptest, 48 ca mỗi lần chạy) và lab, trong đó 0 trên 200 truy vấn cho kết quả khác vét cạn.

**Một luồng, trong bộ nhớ** (25.000 tài xế, 60% quanh trung tâm HCM): chèn tài xế mới 0,60 µs mỗi lần; cập nhật 200.000 lần mất 0,73 µs mỗi lần, trong đó 21.563 lần (10,8%) đổi ô H3; điểm cũ đến sau bị bỏ trong 0,30 µs. Truy vấn gần nhất k = 5, bán kính tối đa 2 km: p50 5,25 µs, p95 6,21 µs, p99 7,75 µs, trung bình 5,00 tài xế mỗi truy vấn. Tải tham chiếu 6.250 cập nhật mỗi giây chỉ dùng một phần rất nhỏ của một lõi. Các số µs này **đo khi máy chịu tải nặng** (tải 1 phút khoảng 90) nên đừng trích tuyệt đối.

**Khi vừa đọc vừa ghi** thì khóa quyết định. Lab chạy mỗi cấu hình 2 giây, 25.000 tài xế, ghi là cập nhật theo vòng, đọc là truy vấn k = 5 liên tục, và so một mảnh khóa với 64 mảnh, hai lần ở hai mức tải máy (90 và 28):

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Hình 2. Số cập nhật ghi mỗi giây khi có 4 luồng đọc chạy cùng: chia 64 mảnh khóa cao hơn một khóa duy nhất ở cả bốn cấu hình và hai lần chạy, từ 3 đến 134 lần."><rect x="20.0" y="20.0" width="840.0" height="360.0" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36.0" y="44.0" class="t-b">Cập nhật ghi mỗi giây khi 4 luồng đọc chạy cùng (k = 5, 25.000 tài xế, mỗi phép đo 2 s)</text><text x="36.0" y="94.0" class="t-b">lần chính, tải 90</text><text x="36.0" y="112.0" class="t-small">1 ghi + 4 đọc</text><rect x="200.0" y="80.0" width="16.4" height="18.0" rx="3" class="f-warn s-warn" stroke-width="1"/><text x="224.4" y="93.0" class="t-small">73.304 ghi/s · đọc 138.899/s</text><rect x="200.0" y="106.0" width="49.5" height="18.0" rx="3" class="f-phys s-phys" stroke-width="1"/><text x="257.5" y="119.0" class="t-small">221.557 ghi/s · đọc 154.898/s</text><text x="36.0" y="166.0" class="t-b">lần chính, tải 90</text><text x="36.0" y="184.0" class="t-small">4 ghi + 4 đọc</text><rect x="200.0" y="152.0" width="38.9" height="18.0" rx="3" class="f-warn s-warn" stroke-width="1"/><text x="246.9" y="165.0" class="t-small">174.080 ghi/s · đọc 188.216/s</text><rect x="200.0" y="178.0" width="151.9" height="18.0" rx="3" class="f-phys s-phys" stroke-width="1"/><text x="359.9" y="191.0" class="t-small">679.723 ghi/s · đọc 93.112/s</text><text x="36.0" y="238.0" class="t-b">lần ít tải, tải 28</text><text x="36.0" y="256.0" class="t-small">1 ghi + 4 đọc</text><rect x="200.0" y="224.0" width="5.6" height="18.0" rx="3" class="f-warn s-warn" stroke-width="1"/><text x="213.6" y="237.0" class="t-small">24.845 ghi/s · đọc 364.158/s</text><rect x="200.0" y="250.0" width="153.0" height="18.0" rx="3" class="f-phys s-phys" stroke-width="1"/><text x="361.0" y="263.0" class="t-small">684.690 ghi/s · đọc 515.988/s</text><text x="36.0" y="310.0" class="t-b">lần ít tải, tải 28</text><text x="36.0" y="328.0" class="t-small">4 ghi + 4 đọc</text><rect x="200.0" y="296.0" width="2.8" height="18.0" rx="3" class="f-warn s-warn" stroke-width="1"/><text x="210.8" y="309.0" class="t-small">12.440 ghi/s · đọc 391.786/s</text><rect x="200.0" y="322.0" width="371.9" height="18.0" rx="3" class="f-phys s-phys" stroke-width="1"/><text x="579.9" y="335.0" class="t-small">1.663.963 ghi/s · đọc 304.737/s</text><rect x="200.0" y="356.0" width="14.0" height="10.0" rx="2" class="f-warn s-warn" stroke-width="1"/><text x="220.0" y="365.0" class="t-small">1 mảnh khóa</text><rect x="320.0" y="356.0" width="14.0" height="10.0" rx="2" class="f-phys s-phys" stroke-width="1"/><text x="340.0" y="365.0" class="t-small">64 mảnh khóa</text></svg>
</div>
<figcaption>Hình 2. Mục 3 của output <code>loc18_kho_vi_tri</code>, hai lần chạy ở hai mức tải máy. Mỗi cặp thanh là cùng số luồng ghi và đọc, chỉ khác số mảnh khóa. Số ghi/s là số đo thật; số đọc/s ghi bên cạnh cho thấy cái giá: ở 4 ghi + 4 đọc, 64 mảnh cho <b>ít</b> truy vấn đọc hơn một mảnh ở cả hai lần chạy (93.112 so với 188.216; 304.737 so với 391.786).</figcaption>
</figure>

Bảng đủ số của Hình 2 (ghi: cập nhật/giây; đọc: truy vấn/giây):

| Cấu hình | Lần | 1 mảnh: ghi / đọc | 64 mảnh: ghi / đọc |
|---|---|---:|---:|
| 1 ghi + 4 đọc | chính (tải 90) | 73.304 / 138.899 | 221.557 / 154.898 |
| 4 ghi + 4 đọc | chính (tải 90) | 174.080 / 188.216 | 679.723 / 93.112 |
| 1 ghi + 4 đọc | ít tải (tải 28) | 24.845 / 364.158 | 684.690 / 515.988 |
| 4 ghi + 4 đọc | ít tải (tải 28) | 12.440 / 391.786 | 1.663.963 / 304.737 |

Hai điều rút ra, và một điều không. Chia mảnh làm tốc độ ghi **cao hơn trong cả bốn cấu hình**, từ 3,0 lần lên 134 lần. Nhưng chiều ngược lại cũng có thật: ở 4 ghi + 4 đọc, 64 mảnh cho **ít** truy vấn đọc hơn một mảnh ở cả hai lần chạy (93.112 so với 188.216; 304.737 so với 391.786), còn luồng đọc thì chậm lại ở đúng cấu hình đó. Điều chưa rút ra được là **cơ chế** của cả hai chiều. Cách hiểu hợp lý: với một mảnh, luồng đọc giữ `RwLock` gần như liên tục nên luồng ghi chờ rất lâu; với 64 mảnh, luồng ghi chạy tự do và tranh CPU với luồng đọc. Lab không có phép đo riêng cho điều đó, và máy 10 lõi lúc đó đang chạy ở tải 28 đến 90, nên các con số đồng thời nhiễu mạnh. Độ trễ đọc p50 vẫn 5,7 đến 7,0 µs ở mọi cấu hình, còn p99 từ 20 đến 282 µs.

## TTL cho tài xế im lặng

Tài xế im lặng quá `ttl_ms` (30 giây, tính theo `received_ms`) bị `get` và `nearest` **bỏ qua ngay**, trước cả khi bị dọn khỏi bộ nhớ: điều kiện `received_ms + ttl < now` nằm ngay trong vòng lặp truy vấn. `sweep` dọn hẳn khỏi bảng và trả danh sách mã tài xế đã xóa để xóa luôn ở Redis. Lab kiểm: 25.000 tài xế, 10% tiếp tục gửi đến giây thứ 40; ở giây 41, truy vấn bán kính 30 km thấy 2.500 tài xế (đã bỏ qua 22.500 tài xế quá TTL chưa dọn), và `sweep` xóa 22.500 tài xế trong 1,9 ms. Lab không có số cho việc «tài xế vào hầm» cụ thể; TTL 30 giây là tham số của lab, không phải kết luận rằng 30 giây đúng với Tới Nơi.

## Khởi động lại: nạp từ đâu

Kho rỗng sau khởi động, và có ba cách làm đầy nó. Lab đo cả ba với cùng 25.000 tài xế:

| Cách | Lần chính | Lần ít tải | Lần tải cao |
|---|---:|---:|---:|
| Redis `HGETALL` (bản phụ) | 46,0 ms | 81,0 ms | 60,2 ms |
| Kafka phát lại 34 giây cuối (`offsets_for_times`), 200.000 bản ghi, 3 partition | 0,08 s kết nối + 0,65 s | 0,07 s + 0,43 s | 3,05 s |
| Nguội hoàn toàn (`Start::Latest`) | đầy sau 4 s | không đo lại | không đo lại |

(«Ít tải» và «tải cao» là hai lần chạy khác được chép từ màn hình, đã ghi trong `results/`; tải 1 phút của ít tải, lần chính và tải cao lần lượt khoảng 28, 90 và 125.) Thời gian khác nhau nhiều giữa các lần chạy vì máy chịu tải thay đổi (Kafka phát lại từ 0,43 giây tới 3,05 giây, hơn bảy lần), nên chỉ so sánh được bậc độ lớn. Hình 3 vẽ cách kho đầy lại khi nguội:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Hình 3. Kho khởi động nguội: số tài xế nhìn thấy được theo thời gian, 25%, 49%, 75%, 99,7% sau 4 giây rồi đủ sau 5 giây; khởi động nóng từ Redis hoặc Kafka đã đầy ngay từ giây 0."><rect x="20.0" y="20.0" width="560.0" height="360.0" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36.0" y="44.0" class="t-b">Tài xế nhìn thấy được (điểm trong TTL 30 s)</text><path d="M94.0 320.0 L550.0 320.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="88.0" y="324.0" class="t-small" text-anchor="end">0</text><path d="M94.0 205.0 L550.0 205.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="88.0" y="209.0" class="t-small" text-anchor="end">12.500</text><path d="M94.0 90.0 L550.0 90.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="88.0" y="94.0" class="t-small" text-anchor="end">25.000</text><path d="M100.0 90.0 L550.0 90.0" class="f-none s-phys" stroke-width="3"/><text x="546.0" y="82.0" class="t-small" text-anchor="end">khởi động nóng (Redis hoặc Kafka)</text><path d="M100.0 320.0 L164.3 262.9 L228.6 207.8 L292.9 147.8 L357.1 90.7 L421.4 90.0 L485.7 90.0 L550.0 90.0" class="f-none s-warn" stroke-width="3" stroke-linejoin="round" stroke-linecap="round"/><circle cx="100.0" cy="320.0" r="4" class="f-warn s-warn" stroke-width="1"/><text x="100.0" y="340.0" class="t-small" text-anchor="middle">0 s</text><circle cx="164.3" cy="262.9" r="4" class="f-warn s-warn" stroke-width="1"/><text x="164.3" y="340.0" class="t-small" text-anchor="middle">1 s</text><circle cx="228.6" cy="207.8" r="4" class="f-warn s-warn" stroke-width="1"/><text x="228.6" y="340.0" class="t-small" text-anchor="middle">2 s</text><circle cx="292.9" cy="147.8" r="4" class="f-warn s-warn" stroke-width="1"/><text x="292.9" y="340.0" class="t-small" text-anchor="middle">3 s</text><circle cx="357.1" cy="90.7" r="4" class="f-warn s-warn" stroke-width="1"/><text x="357.1" y="340.0" class="t-small" text-anchor="middle">4 s</text><circle cx="421.4" cy="90.0" r="4" class="f-warn s-warn" stroke-width="1"/><text x="421.4" y="340.0" class="t-small" text-anchor="middle">5 s</text><circle cx="485.7" cy="90.0" r="4" class="f-warn s-warn" stroke-width="1"/><text x="485.7" y="340.0" class="t-small" text-anchor="middle">6 s</text><circle cx="550.0" cy="90.0" r="4" class="f-warn s-warn" stroke-width="1"/><text x="550.0" y="340.0" class="t-small" text-anchor="middle">7 s</text><text x="240.6" y="225.8" class="t-small">nguội: 49% sau 2 s</text><text x="371.1" y="118.7" class="t-small">99,7% sau 4 s</text><text x="325.0" y="362.0" class="t-small" text-anchor="middle">thời gian từ lúc kho bắt đầu nhận (nguội: không nạp gì)</text><rect x="600.0" y="20.0" width="260.0" height="360.0" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="616.0" y="44.0" class="t-b">Thời gian nạp 25.000 tài xế</text><text x="616.0" y="74.0" class="t-b">Redis HGETALL</text><text x="616.0" y="92.0" class="t-small">46,0 ms (lần chính)</text><text x="616.0" y="108.0" class="t-small">81,0 ms (ít tải)</text><text x="616.0" y="124.0" class="t-small">60,2 ms (tải cao)</text><text x="616.0" y="158.0" class="t-b">Kafka phát lại 34 s</text><text x="616.0" y="176.0" class="t-small">0,65 s + 0,08 s kết nối</text><text x="616.0" y="192.0" class="t-small">0,43 s + 0,07 s (ít tải)</text><text x="616.0" y="208.0" class="t-small">3,05 s (tải cao)</text><text x="616.0" y="242.0" class="t-b">Nguội, không nạp</text><text x="616.0" y="260.0" class="t-small">4 s: một chu kỳ</text><text x="616.0" y="276.0" class="t-small">báo cáo của tài xế</text><text x="616.0" y="352.0" class="t-small">Đo khi máy chịu tải nặng.</text></svg>
</div>
<figcaption>Hình 3. Mục 7(d) của output <code>loc18_kho_vi_tri --redis --kafka</code>: kho khởi động nguội (<code>Start::Latest</code>) đầy dần theo chu kỳ báo cáo 4 giây của 25.000 tài xế; số tài xế là số đếm, không phụ thuộc tải máy. Bảng bên phải gom thời gian nạp của ba lần chạy, <b>đều đo khi máy chịu tải</b> nên chỉ có giá trị so sánh bậc độ lớn. Dữ liệu tổng hợp.</figcaption>
</figure>

Kết luận chắc: **Kafka không cần Redis để khởi động nóng**, vì TTL chỉ 30 giây nên phần phát lại ngắn (200.000 bản ghi trong 0,65 giây ở lần chính). Redis có ích khi muốn khởi động dưới 0,1 giây, hoặc khi Kafka chậm hay đang gặp sự cố. Và ngay cả không nạp gì, kho tự đầy lại trong 4 giây vì nguồn của nó là luồng liên tục. Bài toán thật không nằm ở thời gian đầy lại mà ở chỗ trong 4 giây đó matcher đang nhìn một kho thiếu; nên readiness probe của bản `geo-engine` mới (bài 63) phải chờ kho đủ ấm thay vì chỉ chờ tiến trình lên.

## Redis là bản phụ

Redis không nằm trên đường đọc nóng. Nó có hai việc: khởi động lại nhanh, và cho dịch vụ khác cần `GEOSEARCH` mà không muốn giữ bản đồ vị trí trong RAM. Mỗi tài xế ở hai chỗ: một sorted set `GEO` (member là mã tài xế) cho `GEOSEARCH`, và một hash mã tài xế → bản ghi nhị phân 46 byte (đủ giờ, seq, accuracy/speed/heading và cờ, kèm mặt nạ «có mặt» để giữ vắng khác 0) cho nạp lại. `GEOSEARCH` có từ Redis 6.2.0 và chấp nhận `FROMLONLAT`, `BYRADIUS`, `ASC`, `COUNT` như lab dùng; Redis của lab là 7.4.12.

Điều quan trọng nhất là Redis **không được làm chậm kho**. `MirrorWriter` có một luồng riêng nhận cập nhật qua hàng đợi có giới hạn, gộp theo tài xế và ghi theo lô (pipeline):

Tệp `examples/toi-noi-location/rust/crates/location-store/src/mirror.rs` (dòng 281–287):

```rust
    /// Không bao giờ chặn: hàng đợi đầy thì bỏ và đếm.
    pub fn upsert(&self, id: Arc<str>, p: Position) {
        self.stats.queued.fetch_add(1, Ordering::Relaxed);
        if self.tx.try_send(Msg::Upsert(id, p)).is_err() {
            self.stats.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
```

Hàng đợi đầy thì bỏ và đếm, không bao giờ chặn luồng tiêu thụ Kafka. Số đo: ghi 25.000 tài xế mất 1,25 giây lần chính (20.011 tài xế/giây) và 0,19 giây lần ít tải (130.653 tài xế/giây), 50 lô, 0 cập nhật bị bỏ, 0 lỗi; Redis có đủ 25.000 thành viên. Gộp theo tài xế hiệu quả: 5.000 cập nhật cho 1.000 tài xế trong một lần xả chỉ ghi 1.000 lệnh. `GEOSEARCH` của Redis trả cùng tập tài xế với kho trong bộ nhớ ở 100 trên 100 truy vấn (so theo tập, vì Redis làm tròn tọa độ về geohash 52 bit nên thứ tự có thể khác khi hai xe gần ngang nhau). Cũng cần nói về TTL: sorted set không có hạn riêng cho từng member, hạn của khóa thì áp cho cả tập, nên lab **không dùng hạn của khóa Redis**. Bản phụ chỉ bị xóa khi `sweep` của kho báo, nên nếu tiến trình chết, Redis giữ bản ghi cũ; khi nạp lại, lab lọc theo `received_ms` và báo số bản ghi quá hạn và hỏng (0 và 0 trong lần chạy này).

## Giảm mẫu sang lịch sử

Không phải điểm nào của kho cũng đáng lưu. Phần Rust chỉ quyết định điểm nào đi sang lịch sử (`history::Sampler`), còn ghi Parquet là việc của Go ([bài 20](/articles/location-lich-su-gps-parquet-retention-quyen-rieng-tu)). Chính sách giống `history-writer`: luôn giữ điểm đầu và điểm có cờ; giữ khi đã di chuyển từ 60 m; giữ khi hướng đổi từ 30° sau ít nhất 4 giây; và giữ một điểm «còn sống» mỗi 60 giây khi đứng yên. Trên 60 chuyến HCM sinh có seed (10.101 điểm, 4 giây một điểm, nhiễu σ = 6 m), lab giữ **33,7%** (3.400 điểm: 2.483 vì di chuyển, 848 vì rẽ, 60 điểm đầu, 9 nhịp, 0 cờ). Con số 60,2% ở thí nghiệm Kafka trực tiếp là của luồng đi ngẫu nhiên do lab sinh (đổi hướng nhiều) và **không đại diện** cho xe chạy trên đường. Bài 20 báo 26,7% trên dữ liệu và simulator của riêng nó; ba con số đến từ ba tập dữ liệu khác nhau nên không so trực tiếp được.

## Chạy ví dụ và kiểm chứng

Redis và Kafka cần chạy bằng `docker compose` của `examples/toi-noi-location/infra` (Redis cổng 6391, Kafka cổng 9096). Bỏ `--redis --kafka` thì chỉ chạy mục 1 đến 5 trong bộ nhớ, không cần mạng. Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc18_kho_vi_tri -- --redis --kafka
```

Cấu hình đo: `--release`, Apple M1 Max, 25.000 tài xế tổng hợp (60% quanh trung tâm HCM, 40% rải khắp bbox), H3 độ phân giải 9, TTL 30 giây, 64 mảnh khóa, seed 20261106, Redis 7.4.12, Kafka `apache/kafka:4.1.0` một broker, 3 partition. Output (rút gọn; **đo khi máy chịu tải nặng**, tải 1 phút khoảng 90):

```text
uptime lúc bắt đầu: 23:27  up 35 days, 15:10, 1 user, load averages: 90.25 85.49 79.63
…
-- 1. cập nhật một luồng (apply) --
chèn 25000 tài xế mới: 0.60 µs/lần (1668613/s), 2868 ô H3 có tài xế
cập nhật 200000 lần (kể cả sinh vị trí mới): 0.73 µs/lần; đổi ô H3 21563 lần = 10.8% số cập nhật; bỏ vì cũ 0
điểm cũ đến sau bị bỏ: 0.30 µs/lần

-- 2. truy vấn gần nhất (k = 5, bán kính tối đa 2 km, một luồng) --
20000 truy vấn quanh trung tâm: p50 5.25 µs, p95 6.21 µs, p99 7.75 µs; trung bình 5.00 tài xế/truy vấn
đối chiếu với vét cạn trên 25000 vị trí: 0/200 truy vấn khác kết quả
…
-- 4. TTL cho tài xế im lặng (TTL 30 s) --
25000 tài xế, 10% còn gửi: trước khi dọn, truy vấn bán kính 30 km tại t = 41 s thấy 2500 (đã bỏ qua tài xế quá TTL); sweep xóa 22500 tài xế trong 1.9 ms; còn trong bộ nhớ 2500
…
ghi 25000 tài xế qua MirrorWriter (gộp theo tài xế, lô 500): 1.25 s = 20011 tài xế/s; 50 lô, 25000 đã ghi, 0 bị bỏ vì hàng đợi đầy, 0 lỗi
Redis có 25000 thành viên trong sorted set GEO
gộp theo tài xế: 5.000 cập nhật cho 1.000 tài xế trong một lần xả chỉ ghi 1000 lệnh
khởi động lại, nạp từ Redis: đọc 25000 bản ghi, nạp 25000, quá hạn TTL 0, hỏng 0: 46.0 ms; kho có 25000 tài xế, bất biến đúng
bản sao khớp GEOSEARCH của Redis trên 100 truy vấn k = 5: 100/100 cùng tập tài xế
…
(c) trực tiếp 10 s, 25000 tài xế gửi mỗi 4 s, kho đã khởi động nóng: nhận 62452 bản ghi (62452 áp dụng, 0 bỏ vì cũ), độ trễ gửi→kho p50 27 ms, p95 214 ms, p99 303 ms; lịch sử giữ 37575 (60.2%); kho 25000 → 25000 tài xế
    số tài xế nhìn thấy được (điểm trong TTL 30 s) theo thời gian: t=0s: 25000 (100%); t=1s: 25000 (100%); t=4s: 25000 (100%); t=7s: 25000 (100%); t=10s: 25000 (100%)
(d) trực tiếp 10 s, 25000 tài xế gửi mỗi 4 s, kho khởi động nguội (Latest): nhận 62415 bản ghi (62415 áp dụng, 0 bỏ vì cũ), độ trễ gửi→kho p50 9 ms, p95 56 ms, p99 83 ms; lịch sử giữ 37560 (60.2%); kho 0 → 25000 tài xế
    số tài xế nhìn thấy được (điểm trong TTL 30 s) theo thời gian: t=0s: 0 (0%); t=1s: 6207 (25%); t=2s: 12197 (49%); t=3s: 18720 (75%); t=4s: 24924 (100%); t=5s: 25000 (100%); t=6s: 25000 (100%); t=7s: 25000 (100%); t=8s: 25000 (100%); t=9s: 25000 (100%); t=10s: 25000 (100%)
```

Số tài xế, số bản ghi, tỉ lệ giữ và số khớp vét cạn tái lập từng chữ số; µs, ms, cập nhật/giây và truy vấn/giây thì nhiễu mạnh theo tải máy. Test: `cargo test --manifest-path examples/toi-noi-location/rust/Cargo.toml -p location-store` chạy 12 test đơn vị (đổi ô, TTL, nearest so với vét cạn, đồng thời giữ bất biến, vắng khác 0 qua proto và bản ghi nhị phân 46 byte); các test tích hợp Kafka và Redis chỉ chạy khi đặt `TOI_NOI_KAFKA=1 TOI_NOI_REDIS=1`, và khi Kafka không tới được thì **thất bại có hướng dẫn bật hạ tầng** chứ không bỏ qua âm thầm.

## Giới hạn của ví dụ

- **Chưa chạy** Redis chết giữa lúc chạy (chỉ có test hàng đợi đầy: hàng đợi 4 chỗ, 50.000 `upsert` không chặn và có bản ghi bị bỏ), **chưa chạy** Kafka rebalance, **chưa commit offset** (cố ý; xem thiết kế ở trên), chưa chạy nhiều bản `geo-engine` cùng đọc một topic, chưa thử Redis cụm.
- **Một broker, một Redis, cùng một máy, không có mạng thật.** Độ trễ gửi tới kho khi chạy trực tiếp qua Kafka là p50 5 đến 27 ms, p95 11 đến 214 ms, p99 26 đến 303 ms tùy tải máy và tùy khởi động nóng hay nguội; SLO tham chiếu (độ tươi vị trí p95 dưới 5–10 giây) còn dư rất xa, nhưng đó là điều kiện lý tưởng.
- **Cơ chế của số liệu đồng thời chưa được kiểm riêng**, và 64 mảnh làm giảm số truy vấn đọc ở 4 ghi + 4 đọc.
- **Dữ liệu do lab sinh:** vị trí tài xế là bước đi ngẫu nhiên, không phải đường đi thật; 60% quanh trung tâm là giả định. Tỉ lệ giữ 33,7% là trên 60 chuyến.
- **Độ ổn định của test tích hợp Kafka:** khi phát triển, `consume_apply_replay_ttl_against_real_kafka` thất bại 2 trong khoảng 7 lần chạy có đặt `TOI_NOI_KAFKA=1`, lúc container Kafka dùng chung đang ở trạng thái Paused (do một thí nghiệm khác). Đó là nguyên nhân hợp lý nhưng không được chứng minh riêng. Sau khi thêm thử lại 3 lần cho việc tạo topic, 6 trên 6 lần chạy liên tiếp qua.
- **Kho chỉ lưu vị trí mới nhất theo `driver_id`**, không có cơ chế chia theo thành phố (P07) hay chuyển quyền sở hữu tài xế giữa hai tiến trình.

## Bài tập

**Bài 1.** TTL là 30 giây và tài xế gửi mỗi 4 giây. Một tài xế vừa vào hầm 20 giây. Tại giây 20, tài xế đó có nằm trong kết quả `nearest` không? Còn tại giây 35?

<details><summary>Lời giải bài 1</summary>
<p>Tại giây 20 kể từ điểm cuối cùng nhận được: <code>received_ms + 30 s ≥ now</code> nên vẫn còn trong TTL, và vẫn nằm trong kết quả dù xe có thể đã đi xa hàng trăm mét; TTL chỉ đo thời gian im lặng, không đo sai lệch vị trí. Tại giây 35 thì bị bỏ qua ngay dù <code>sweep</code> chưa chạy. Nếu muốn loại sớm hơn cho tài xế đang chạy nhanh thì cần TTL động theo vận tốc, một đề xuất mà lab chưa đo.</p>
</details>

**Bài 2.** Ở 4 ghi + 4 đọc, 64 mảnh cho nhiều cập nhật hơn nhưng ít truy vấn đọc hơn một mảnh. Bạn muốn ưu tiên đọc (matcher) hơn ghi. Có thể chỉnh những gì, và làm sao biết chỉnh đúng?

<details><summary>Lời giải bài 2</summary>
<p>Có thể giảm số mảnh của <code>cells</code>, tăng nhịp gộp cập nhật theo lô, hoặc chạy các luồng đọc ở độ ưu tiên cao hơn. Để biết chỉnh đúng, thêm mục đo có hai chỉ số cùng lúc (ghi/giây và độ trễ p99 của truy vấn) rồi lặp lại nhiều lần ở cùng mức tải, vì ở tải 28 và 90 kết quả đã khác nhau tới mức một lần chạy không đủ để kết luận. Đồng thời kiểm cơ chế bằng một phép đo riêng thời gian chờ khóa.</p>
</details>

**Bài 3.** Kafka phát lại 34 giây cuối mất 0,65 giây nhưng Redis chỉ cần 46 ms. Khi nào bạn vẫn chọn Kafka làm nguồn khởi động?

<details><summary>Lời giải bài 3</summary>
<p>Khi muốn một nguồn sự thật duy nhất và không phụ thuộc bản phụ có thể cũ: Redis chỉ chứa những gì <code>MirrorWriter</code> kịp ghi, và hàng đợi đầy thì bỏ cập nhật. Phát lại từ Kafka là đúng theo luồng gốc nên không có khoảng trống do bản phụ bỏ cập nhật; còn với TTL chỉ 30 giây, 0,65 giây (lần chính) là chấp nhận được. Cần lưu ý số thời gian của hai cách đều nhiễu theo tải máy (Kafka phát lại từ 0,43 giây tới 3,05 giây giữa các lần chạy).</p>
</details>

Bài trước: [bài 17](/articles/location-map-matching-online-cua-so-cau-vuot-duong-song-song). Bài tiếp: [bài 19](/articles/location-go-fan-out-tracking-khach-noi-suy-marker).

## Tài liệu gốc

- [Redis: lệnh GEOSEARCH](https://redis.io/docs/latest/commands/geosearch/): cú pháp `FROMLONLAT`, `BYRADIUS`, `ASC`, `COUNT`; có từ Redis 6.2.0.
- [Uber H3: bảng độ phân giải](https://h3geo.org/docs/core-library/restable): độ dài cạnh trung bình theo từng độ phân giải (độ phân giải 9 khoảng 200,8 m).
- [h3o trên docs.rs](https://docs.rs/h3o/latest/h3o/): bản cài đặt H3 bằng Rust mà lab dùng.
- [rdkafka trên docs.rs](https://docs.rs/rdkafka/latest/rdkafka/): `BaseConsumer` và các kiểu liên quan, bản bọc librdkafka.
- [redis trên docs.rs](https://docs.rs/redis/latest/redis/): client Redis cho Rust.
- [Apache Kafka: tài liệu](https://kafka.apache.org/documentation/): consumer, `assign` và offset.
