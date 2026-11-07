---
slug: location-sinh-ung-vien-diem-don-do-thi-di-bo
title: "Sinh ứng viên điểm đón: khách đi bộ tới được đâu"
summary: "Bài 33: Dijkstra có giới hạn trên đồ thị đi bộ, ứng viên theo từng chiều xe, loại cầu, hầm, vòng xuyến, đoạn quá ngắn, gom ứng viên gần nhau; đo số ứng viên ở từng bước và vì sao bán kính chim bay đánh lừa."
category: system-design
level: advanced
tags: [toi-noi, location, rust, pickup, candidate-generation, dijkstra, walking-graph, openstreetmap]
language: vi
---

Khách đứng trong một khúc đường nhỏ ở TP.HCM, cách các đoạn cầu, hầm hoặc cầu vượt khoảng 160 đến 200 mét về phía tây và tây bắc. Hàm mục tiêu `J` của [bài 32](/articles/location-ham-muc-tieu-diem-hen-khach-va-tai-xe) cần một danh sách điểm đón ứng viên để chấm. Nếu lấy mọi cạnh xe chạy trong bán kính 250 m chim bay, danh sách đó sẽ có cả đoạn đường ở bên kia một con kênh hay dải phân cách mà khách phải đi vòng cả trăm mét mới tới, và cả mặt cầu vượt nơi xe không được phép dừng. Bài này dựng bước đầu tiên của pipeline chọn điểm đón: **sinh ứng viên**, tức xác định khách đi bộ tới được những đâu và xe dừng được ở đâu trong tầm đó.

Đây là bài 33 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng đồ thị đi bộ và các profile của [bài 24](/articles/location-profile-xe-may-o-to-di-bo-trong-hem) và hướng cạnh của [bài 23](/articles/location-chi-phi-re-u-turn-do-thi-theo-canh). Mọi số đo là **trên dữ liệu mô phỏng**: kịch bản `hcm-baseline`, đồ thị TP.HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors), 300 khách đầu tiên của seed 20270001 (kịch bản chỉ sinh 209 khách). Bài này chỉ **đếm và đo thời gian**; lợi ích của danh sách ứng viên cho khách và tài xế được đo ở bài 32 và bài 34.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Với khách mẫu, có bao nhiêu cạnh xe nằm trong 250 m chim bay, và bao nhiêu trong số đó khách <em>đi bộ tới được</em> trong 250 m? Hãy đoán tỉ lệ trước khi mở đáp án.</p>
<details><summary>Xem đáp án</summary><p>Trung bình 118,4 cạnh xe nằm trong 250 m chim bay, nhưng chỉ 49,2 cạnh đi bộ tới được trong 250 m; 69,1 cạnh (58%) gần theo chim bay mà xa theo đường đi bộ. Dùng bán kính chim bay sẽ đề xuất điểm đón mà khách phải vòng qua cầu hay dải phân cách.</p></details>
</div>

## Dijkstra có giới hạn trên đồ thị đi bộ

Ý tưởng chính: không ước bằng khoảng cách, mà **duyệt đồ thị đi bộ** từ chỗ khách đứng và dừng ngay khi quãng đi vượt ngân sách. Chỗ khách đứng được snap lên cạnh đi bộ gần nhất (bằng R-tree của [bài 07](/articles/location-rstar-r-tree-snap-diem-ve-duong)), rồi chạy Dijkstra với ngân sách là 250 m chia cho tốc độ của bảng chi phí đi bộ (5 km/h):

`examples/toi-noi-location/rust/crates/pickup/src/candidates.rs`

```rust
        let view = engine.view(Profile::Foot);
        let budget = (self.cfg.max_walk_m / FOOT_TABLE_MPS * 1000.0) as u32;
        for leg in s.source_legs(view) {
            if leg.cost > budget {
                continue;
            }
            for (n, d) in self
                .searcher
                .within(view, leg.node, budget - leg.cost, true)
            {
                let tot = d + leg.cost;
                reach
                    .entry(n)
                    .and_modify(|o| *o = (*o).min(tot))
                    .or_insert(tot);
            }
        }
```

Hàm trả về tập nút tới được kèm thời gian đi. Dijkstra có giới hạn chỉ chạm những nút trong tầm: trung bình 41,9 nút trên tổng 126.699 của đồ thị (0,0331%), nên chi phí tỉ lệ với vùng nhỏ quanh khách chứ không với cả thành phố. So với «mọi cạnh trong bán kính chim bay»:

| Trên 209 khách mẫu (seed 20270001) | Trung bình | p50 | p95 | Lớn nhất |
|---|---:|---:|---:|---:|
| Cạnh xe trong 250 m chim bay | 118,4 | 118 | 209 | 307 |
| trong đó đi bộ tới được ≤ 250 m | 49,2 | 43 | 109 | 203 |
| chim bay gần nhưng đi bộ quá 250 m | 69,1 | 67 | 132 | 191 |

Hơn một nửa cạnh «gần» theo thước chim bay là cạnh xa theo đường đi bộ. Đó là lý do bài 30 và bài 32 gọi đuôi `walk_m` của baseline là **đi vòng** chứ không phải đi xa: hai điểm cách nhau 60 m trên bản đồ có thể nằm hai bên một con kênh, một mặt cầu, một dải phân cách. Đồ thị đi bộ OSM không phải thực tế (có thể thiếu lối đi tắt thật mà lab không kiểm ngoài thực địa), nhưng nó là thước đo nhất quán với thước đo của baseline.

## Từ nút tới ứng viên: theo từng chiều xe

Tập nút đi bộ tới được là điểm xuất phát. Với mỗi nút, lab duyệt các cạnh xe ô tô (hoặc xe máy) có một đầu ở đó, và coi mỗi cạnh là một **ứng viên có hướng**. Hai chiều của đường hai chiều là hai ứng viên khác nhau, vì chúng khác phía đường: cùng một tọa độ, nhưng xe chạy theo một chiều thì khách ở bên phải, theo chiều kia thì ở bên trái. Mỗi cạnh cho tối đa hai điểm dừng: giữa cạnh, và chân đường vuông góc của chỗ khách đứng nếu khách ở trong 60 m (điểm giữa của cạnh dài có thể cách khách tới nửa chiều dài cạnh, nên điểm thứ hai được thêm sau lần thử đầu).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 575" role="img" aria-label="Hình 1. Khách 193 của seed 20270001 trên đồ thị đi bộ HCM: chỗ đứng, điểm ghim lệch 13,2 m, bốn vị trí ứng viên trong tám dòng đầu danh sách và bốn vị trí cạnh bị loại vì là cầu, hầm hay cầu vượt, cách 160 đến 200 m về phía tây và tây bắc. Tọa độ lấy từ output lab. © OpenStreetMap contributors."><rect x="20" y="20" width="840" height="535" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Ứng viên quanh khách 193</text><path d="M163.0 292.9 L175.0 304.9" class="f-none s-warn" stroke-width="2.5"/><path d="M163.0 304.9 L175.0 292.9" class="f-none s-warn" stroke-width="2.5"/><path d="M226.4 248.0 L238.4 260.0" class="f-none s-warn" stroke-width="2.5"/><path d="M226.4 260.0 L238.4 248.0" class="f-none s-warn" stroke-width="2.5"/><path d="M286.4 202.9 L298.4 214.9" class="f-none s-warn" stroke-width="2.5"/><path d="M286.4 214.9 L298.4 202.9" class="f-none s-warn" stroke-width="2.5"/><path d="M219.5 85.0 L231.5 97.0" class="f-none s-warn" stroke-width="2.5"/><path d="M219.5 97.0 L231.5 85.0" class="f-none s-warn" stroke-width="2.5"/><text x="415.2" y="248.1" class="t-small">×: cạnh cầu/hầm/cầu vượt bị loại</text><text x="415.2" y="264.1" class="t-small">(4 vị trí trong 8 dòng đầu, mỗi vị trí hai chiều)</text><circle cx="638.1" cy="417.3" r="9" class="f-none s-pk" stroke-width="1.5"/><circle cx="638.1" cy="417.3" r="5" class="f-pk s-pk" stroke-width="1.5"/><text x="652.1" y="431.3" class="t-small">14,3 m</text><circle cx="590.1" cy="331.6" r="9" class="f-none s-pk" stroke-width="1.5"/><circle cx="590.1" cy="331.6" r="5" class="f-pk s-pk" stroke-width="1.5"/><text x="576.1" y="325.6" class="t-small" text-anchor="end">35,1 m</text><circle cx="737.1" cy="445.2" r="9" class="f-none s-pk" stroke-width="1.5"/><circle cx="737.1" cy="445.2" r="5" class="f-pk s-pk" stroke-width="1.5"/><text x="751.1" y="459.2" class="t-small">37,5 m</text><circle cx="588.3" cy="429.4" r="9" class="f-none s-pk" stroke-width="1.5"/><circle cx="588.3" cy="429.4" r="5" class="f-pk s-pk" stroke-width="1.5"/><text x="574.3" y="443.4" class="t-small" text-anchor="end">41,7 m</text><circle cx="660.0" cy="387.7" r="7" class="f-phys s-phys" stroke-width="1.5"/><text x="672.0" y="379.7" class="t-b">khách đứng</text><circle cx="628.4" cy="401.5" r="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="618.4" y="395.5" class="t-small" text-anchor="end">ghim (13,2 m)</text><path d="M40.0 491.0 L170.0 491.0" class="f-none s-muted" stroke-width="2"/><path d="M40.0 486.0 L40.0 496.0" class="f-none s-muted" stroke-width="2"/><path d="M170.0 486.0 L170.0 496.0" class="f-none s-muted" stroke-width="2"/><text x="180.0" y="495.0" class="t-small">50 m; gốc tại chỗ khách đứng (10,769484; 106,640061), bắc ở phía trên</text><text x="40.0" y="515.0" class="t-small">● ứng viên; vòng ngoài: xe vào được vị trí đó từ hai chiều.</text><text x="40.0" y="531.0" class="t-small">Điểm 41,7 m mang hai nhãn: một chiều Left, một chiều Right; ba điểm còn lại Centre.</text></svg>
</div>
<figcaption>Hình 1. Khách 193 của seed 20270001 (không liên quan khách 193 của seed 20261106 ở bài 32): chỗ đứng, điểm ghim lệch 13,2 m, bốn vị trí ứng viên trong tám dòng đầu của output (mỗi vị trí là hai chiều xe) với quãng đi bộ ước lượng, và bốn vị trí cạnh bị loại vì là cầu, hầm hoặc cầu vượt cách 160 đến 200 m về phía tây và tây bắc. Trên dữ liệu mô phỏng, tọa độ lấy từ output lab; hình không vẽ đường vì lab không xuất hình học đường. © OpenStreetMap contributors.</figcaption>
</figure>

Hình 1 vẽ khách 193, ca mà lab in chi tiết vì có nhiều cạnh bị loại nhất: 84 nút tới được, 222 điểm dừng thô, loại 36 (cầu, hầm), 48 (quá ngắn), 26 (quá xa), gom 6, còn 106 ứng viên. Tám dòng đầu của danh sách (xếp theo quãng đi bộ ước lượng) thực ra là bốn vị trí, mỗi vị trí hai chiều xe. Điểm cách 41,7 m là ví dụ cho nhận xét ở trên: hai nhãn `Left` và `Right` cho cùng một tọa độ.

## Ràng buộc cứng: loại hẳn, không phải phạt

Ứng viên bị loại nếu xe không được hoặc không nên dừng ở đó. Mã đặt các điều kiện này vào một hàm dùng chung cho cả ứng viên và điểm ghim:

`examples/toi-noi-location/rust/crates/pickup/src/candidates.rs`

```rust
/// Lý do không được dừng tại cạnh `e` (theo cờ và loại đường), `None` nếu được. Dùng chung cho ứng viên và điểm ghim.
pub fn stop_violation(engine: &Engine, e: EdgeId, min_edge_m: f64) -> Option<Violation> {
    let g = &engine.g;
    let i = e as usize;
    if g.flags[i] & (flag::BRIDGE | flag::TUNNEL) != 0 || g.layer[i] != 0 {
        return Some(Violation::BridgeOrTunnel);
    }
    if g.flags[i] & flag::ROUNDABOUT != 0 {
        return Some(Violation::Roundabout);
    }
    if matches!(
        HighwayClass::from_u8(g.class[i]),
        Some(
            HighwayClass::Motorway
                | HighwayClass::MotorwayLink
                | HighwayClass::TrunkLink
                | HighwayClass::PrimaryLink
                | HighwayClass::SecondaryLink
        )
    ) {
        return Some(Violation::NoStopClass);
    }
    if f64::from(g.length_cm[i]) / 100.0 < min_edge_m {
        return Some(Violation::TooShort);
    }
    None
}
```

Bốn nhóm: cầu, hầm, cầu vượt (theo cờ OSM hoặc lớp `layer` khác 0); vòng xuyến; đường cao tốc và các nhánh nối; và cạnh quá ngắn, hiểu là nằm trọn trong nút giao. Cộng thêm quá ngưỡng đi bộ 250 m. Tách «loại hẳn» khỏi «phạt» là có chủ ý (bài 32): một khoản phạt luôn có thể bị lợi ích lớn hơn lấn át, còn dừng trên cầu vượt thì không.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Số cạnh xe theo chiều quanh 209 khách mẫu (20.409 cạnh): 56,2% còn lại làm ứng viên, 23,5% bị loại vì quá ngắn (nằm trong nút giao), 14,3% vì đi bộ quá 250 m, chỉ 0,7% vì cầu, hầm hoặc cầu vượt."><rect x="20" y="20" width="840" height="290" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Phễu loại ứng viên</text><text x="40.0" y="76.0" class="t-small">20.409 cạnh xe theo chiều có một đầu trong tầm đi bộ (209 khách, seed 20270001)</text><rect x="40.0" y="90.0" width="438.3" height="30.0" rx="0" class="f-pk" stroke-width="0"/><rect x="478.3" y="90.0" width="183.3" height="30.0" rx="0" class="f-dead" stroke-width="0"/><rect x="661.7" y="90.0" width="111.4" height="30.0" rx="0" class="f-warn-soft s-warn" stroke-width="1"/><rect x="773.1" y="90.0" width="30.0" height="30.0" rx="0" class="f-phys" stroke-width="0"/><rect x="803.1" y="90.0" width="11.6" height="30.0" rx="0" class="f-warn" stroke-width="0"/><rect x="814.7" y="90.0" width="5.3" height="30.0" rx="0" class="f-muted" stroke-width="0"/><rect x="40.0" y="149.0" width="14.0" height="14.0" rx="2" class="f-pk" stroke-width="0"/><text x="62.0" y="160.0">còn lại làm ứng viên: 11.469 (56,2%)</text><rect x="40.0" y="173.0" width="14.0" height="14.0" rx="2" class="f-dead" stroke-width="0"/><text x="62.0" y="184.0">cạnh quá ngắn (&lt; 20 m): 4.797 (23,5%)</text><rect x="40.0" y="197.0" width="14.0" height="14.0" rx="2" class="f-warn-soft s-warn" stroke-width="1"/><text x="62.0" y="208.0">đi bộ quá 250 m: 2.915 (14,3%)</text><rect x="40.0" y="221.0" width="14.0" height="14.0" rx="2" class="f-phys" stroke-width="0"/><text x="62.0" y="232.0">gom vào ứng viên gần (&lt; 25 m): 786 (3,9%)</text><rect x="40.0" y="245.0" width="14.0" height="14.0" rx="2" class="f-warn" stroke-width="0"/><text x="62.0" y="256.0">cao tốc, nhánh nối, vòng xuyến: 303 (1,5%)</text><rect x="40.0" y="269.0" width="14.0" height="14.0" rx="2" class="f-muted" stroke-width="0"/><text x="62.0" y="280.0">cầu, hầm, cầu vượt: 139 (0,7%)</text></svg>
</div>
<figcaption>Hình 2. Số cạnh xe theo chiều quanh 209 khách mẫu của seed 20270001, tổng 20.409, trên dữ liệu mô phỏng, đồ thị TP.HCM. 56,2% còn lại làm ứng viên; 23,5% bị loại vì cạnh quá ngắn; 14,3% vì đi bộ quá 250 m; 3,9% bị gom; cao tốc và vòng xuyến 1,5%; cầu, hầm, cầu vượt chỉ 0,7%. Ngưỡng 20 m, 250 m và 25 m là giả định của lab.</figcaption>
</figure>

Con số bất ngờ của Hình 2: loại nhiều nhất không phải cầu hay hầm (139 cạnh, 0,7%) hay cao tốc và vòng xuyến (303 cạnh, 1,5%) mà là **cạnh quá ngắn dưới 20 m: 4.797 cạnh, 23,5%**, tức đoạn nằm trong nút giao và ngã tư. Ngưỡng 20 m là giả định của lab, chưa hiệu chỉnh. Nó có hệ quả cụ thể: ở nút giao dày đặc, nơi người thật hay đón khách, danh sách ứng viên mỏng đi đáng kể. Kiểm tra bất biến: **0 ứng viên đầu ra vi phạm** ràng buộc cứng (cầu, hầm, cấm dừng, vòng xuyến, quá ngắn) và 0 ứng viên có quãng đi bộ ước lượng quá 250 m.

## Gom ứng viên gần nhau

Hai điểm cách nhau vài mét trên cùng một đường, cùng chiều, không đáng là hai lựa chọn. Lab sắp ứng viên theo quãng đi bộ ước lượng tăng dần rồi giữ lại ứng viên đi bộ ít nhất, bỏ những điểm cùng đường, cùng chiều, nằm trong 25 m một điểm đã giữ:

`examples/toi-noi-location/rust/crates/pickup/src/candidates.rs`

```rust
        // gom: đi bộ ít nhất trước; bỏ ứng viên cùng đường, cùng chiều, quá gần một ứng viên đã giữ
        raw.sort_by(|a, b| {
            a.walk_est_m
                .total_cmp(&b.walk_est_m)
                .then(a.pos.edge.cmp(&b.pos.edge))
        });
        let mut kept: Vec<(Candidate, i64, [f64; 2])> = Vec::new();
        for c in raw {
            let way = g.way_id[c.pos.edge as usize];
            let dir = direction_at(engine, c.pos.edge, c.pos.frac);
            let dup = kept.iter_mut().find(|(k, kway, kd)| {
                *kway == way
                    && haversine_m(k.point, c.point) < self.cfg.merge_radius_m
                    && kd[0] * dir[0] + kd[1] * dir[1] > 0.0
            });
            if let Some((k, _, _)) = dup {
                k.merged += 1;
                stats.merged_away += 1;
            } else {
                kept.push((c, way, dir));
            }
        }
```

Gom bỏ trung bình 3,8 ứng viên mỗi khách (p95 11, lớn nhất 20), chiếm 3,9% số cạnh theo chiều. Còn lại trung bình **54,9 ứng viên mỗi khách** (p95 118, lớn nhất 176), nghĩa là bộ xếp hạng ở bài 35 chấm vài chục điểm (tối đa 176 trong mẫu này), không phải hàng nghìn. Bộ sinh trả điểm ghim riêng cho mỗi chiều (trung bình 1,9). Nó không bị loại mà chỉ bị gắn cờ nếu vi phạm, và không bị kiểm ngưỡng 250 m. Bộ xếp hạng ở bài 35 chỉ chấm điểm ghim hợp lệ; điểm ghim vi phạm chỉ được dùng khi không còn ứng viên nào khác (`pin_forced`).

## Ngân sách đi bộ đổi số ứng viên mạnh hơn bạn nghĩ

Ngân sách 250 m là một lựa chọn. Lab đo thử bốn mức (trên dữ liệu mô phỏng, 209 khách, seed 20270001, **thời gian đo khi máy đang chịu tải nặng** nên chỉ so tương đối):

| Ngân sách đi bộ | Ứng viên TB | Ứng viên p95 | Thời gian một lần sinh, p50 (µs) |
|---|---:|---:|---:|
| 100 m | 10,0 | 22 | 20 |
| 150 m | 20,4 | 44 | 32 |
| 250 m | 54,9 | 118 | 72 |
| 400 m | 145,5 | 310 | 163 |

Số ứng viên tăng nhanh hơn tuyến tính: ngân sách tăng 2,5 lần (100 sang 250 m) thì ứng viên tăng 5,5 lần, vì vùng tới được tăng theo diện tích. Mỗi ứng viên thêm vào là một ô cần tính `J` ở bước sau (bài 34, bài 35), nên chi phí này không miễn phí. Thời gian một lần sinh ở mức 250 m: p50 71 µs, p95 458 µs, lớn nhất 22.499 µs, so với ~2 ms CPU mỗi tuyến A\* ở tầng chấm lại của bài 35: sinh ứng viên không phải chỗ đắt.

## Ai được lợi, ai chịu thiệt

Bước sinh ứng viên không tối ưu gì một mình, nhưng nó đặt giới hạn cho mọi bước sau, nên đáng hỏi ai chịu ảnh hưởng.

- **Khách được bảo vệ.** Ngưỡng 250 m cứng nghĩa là mọi ứng viên không phải điểm ghim có quãng đi bộ ước lượng không quá 250 m trên đồ thị đi bộ, nên thuật toán phía sau không thể đẩy khách đi xa hơn mức đó chỉ để tài xế đỡ chạy. Điểm ghim là ngoại lệ, và đây là lý do `walk_m` max của `p04-v1` vẫn trên 1.000 m ở bài 32.
- **Tài xế được bảo vệ khỏi điểm dừng bất hợp lý**: không phải dừng trên cầu, cao tốc, vòng xuyến.
- **Chỗ chịu thiệt:** ràng buộc cứng có thể loại đúng điểm sẽ cho `J` tốt nhất. Ngưỡng 20 m loại gần một phần tư số cạnh; ngưỡng 250 m có thể loại điểm thuận cho tài xế mà khách sẵn lòng đi bộ thêm một chút. Lab không đo mức thiệt này, vì nó chưa chạy thử nghiệm với các ngưỡng khác trên simulator (chỉ đo số ứng viên ở bốn ngân sách).
- **Khách ở xa đường xe** (chỗ đứng cách mọi đường ô tô hơn 250 m đi bộ) có thể không có ứng viên nào ngoài điểm ghim. Lab buộc dùng ghim trong trường hợp đó (cờ `pin_forced`) và bài này không thống kê tần suất.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` (hoặc `--fixture`). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run --release -q --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc33_sinh_ung_vien
```

Cấu hình: Apple Silicon, `--release`, `GenConfig` mặc định (đi bộ ≤ 250 m, gom trong 25 m, cạnh ≥ 20 m), kịch bản `hcm-baseline`, seed 20270001. `uptime` lúc chạy: load averages 404,41 410,54 405,91, nên mọi số thời gian là cận trên bị phóng đại; số đếm tái lập từng chữ số. Output (rút gọn):

```text
-- 1. đếm ứng viên ở từng bước (ngân sách đi bộ 250 m) --
  nút đi bộ tới được                 trung bình    41.9  p50     35  p95     96  max    150
  cạnh xe có đầu trong tầm (theo chiều) trung bình    97.7  p50     85  p95    214  max    408
  loại: cầu, hầm, cầu vượt           trung bình     0.7  p50      0  p95      3  max     36
  loại: cạnh quá ngắn (< 20 m)       trung bình    23.0  p50     16  p95     64  max    184
  ứng viên còn lại                   trung bình    54.9  p50     48  p95    118  max    176
  tổng trên 209 khách: 20409 cạnh theo chiều → loại cầu/hầm 139 (0.7%), cao tốc/vòng xuyến 303 (1.5%), quá ngắn 4797 (23.5%), quá xa 2915 (14.3%), gom 786 (3.9%), còn 11469 (56.2%)
  kiểm tra: ứng viên đầu ra vi phạm ràng buộc cứng (cầu/hầm/cấm dừng/vòng xuyến/quá ngắn) = 0; ứng viên đi bộ ước lượng quá 250 m = 0
-- 2. Dijkstra có giới hạn so với «mọi cạnh trong bán kính chim bay» --
  Dijkstra có giới hạn duyệt trung bình 41.9 nút trên tổng 126699 nút của đồ thị (0.0331%)
```

## Giới hạn của ví dụ

- **Đồ thị đi bộ là OSM**, snap không biết chướng ngại: có thể thiếu lối đi tắt thật, và nhiều khả năng góp phần vào đuôi `walk_m` ở baseline (bài 32). Lab không kiểm ngoài thực địa. Quãng đi bộ đo dọc đồ thị giữa hai điểm đã snap, không gồm đoạn từ chỗ đứng ra tới đường.
- **Ba ngưỡng là giả định**: 20 m (cạnh quá ngắn), 250 m (đi bộ), 25 m (gom). Chưa hiệu chỉnh và chưa đo ảnh hưởng của chúng lên `J` hay `deadhead_m`.
- **Ứng viên chỉ ở giữa cạnh và chân đường vuông góc**, không có điểm dừng thật như lề đường, bãi đỗ, cổng (bài 37 thêm lớp cổng, sảnh bằng dữ liệu bịa).
- **Chưa có thẻ cấm dừng thật** của OSM (biển cấm, `parking:*`); chỉ có cờ cầu, hầm, vòng xuyến, loại đường.
- **Một seed, 209 khách**: số đếm cho cỡ của bài toán, không cho khoảng tin cậy. Thời gian đo khi máy chịu tải nặng.

## Bài tập

**Bài 1.** Từ Hình 2 (20.409 cạnh theo chiều), tính tỉ lệ cạnh còn lại làm ứng viên và kiểm tổng các phần bằng 20.409. Nếu hạ ngưỡng cạnh ngắn từ 20 m xuống 10 m, bạn kỳ vọng tỉ lệ nào đổi nhiều nhất, và bạn đo tác động đó thế nào?

<details><summary>Lời giải bài 1</summary>
<p>11.469 / 20.409 = 56,2%. Tổng: 11.469 + 4.797 + 2.915 + 786 + 303 + 139 = 20.409. Tỉ lệ «quá ngắn» (23,5%) đổi nhiều nhất, và một phần số cạnh chuyển sang «còn lại». Đo bằng cách chạy lại lab với <code>min_edge_m</code> khác, rồi chạy so sánh cặp trên 20 seed đánh giá (bài 32) cho <code>deadhead_m</code> và <code>u_turns</code>, vì ứng viên mới nằm trong nút giao có thể cho J tốt hơn nhưng cũng cản đường hơn; số đếm một mình không trả lời được.</p>
</details>

**Bài 2.** Vì sao ngân sách đi bộ tăng 2,5 lần (100 m lên 250 m) làm số ứng viên tăng 5,5 lần chứ không 2,5 lần?

<details><summary>Lời giải bài 2</summary>
<p>Tập nút tới được nằm trong một vùng quanh khách; số cạnh xe tỉ lệ với diện tích vùng, tức bình phương bán kính nếu mạng đường đều. 2,5² = 6,25, gần với tỉ lệ 54,9 / 10,0 = 5,5 đo được. Phần lệch đến từ việc mạng đường không đều và từ các ràng buộc cứng. Ở 400 m, 145,5 / 54,9 = 2,65 so với (400/250)² = 2,56.</p>
</details>

**Bài 3.** Bộ sinh luôn trả điểm ghim, kể cả khi nó bị gắn cờ vi phạm, và bộ xếp hạng buộc dùng nó khi không còn gì khác. Nêu một tình huống khiến điều này cần thiết và một rủi ro của nó.

<details><summary>Lời giải bài 3</summary>
<p>Cần thiết khi khách đứng giữa vùng mà mọi cạnh xe quanh đó đều bị loại hoặc quá xa (ví dụ khu có toàn cầu vượt và đoạn quá ngắn): nếu không có phương án lùi, hệ thống bỏ khách, điều lab tránh bằng cách buộc dùng ghim và không bao giờ bỏ khách. Rủi ro: ghim có thể nằm trên điểm không nên dừng (cầu, vòng xuyến); lab mới gắn cờ, chưa có cơ chế cảnh báo tài xế, nên cần một lớp dữ liệu nơi khó (bài 37) để xử lý.</p>
</details>

Bài trước: [bài 32](/articles/location-ham-muc-tieu-diem-hen-khach-va-tai-xe). Bài tiếp: [bài 34](/articles/location-phia-duong-huong-tiep-can-tranh-quay-dau).

## Tài liệu gốc

- [Dijkstra's algorithm (Wikipedia)](https://en.wikipedia.org/wiki/Dijkstra%27s_algorithm): duyệt đồ thị theo chi phí tăng dần và dừng ở ngân sách.
- [rstar trên docs.rs](https://docs.rs/rstar): R-tree dùng để snap chỗ đứng lên cạnh đi bộ.
