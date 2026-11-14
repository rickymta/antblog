---
slug: location-chi-phi-re-u-turn-do-thi-theo-canh
title: "Chi phí rẽ và quay đầu: đồ thị theo cạnh"
summary: "Bài 23: vì sao Dijkstra theo nút bỏ sót quay đầu và cấm rẽ, đồ thị theo cạnh với chi phí rẽ, đọc cấm rẽ OSM (bỏ qua 784 có điều kiện và 112 có via là way), lối qua dải phân cách, đo 300 truy vấn HCM."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, routing, turn-cost, u-turn, edge-based, openstreetmap]
language: vi
---

Trên đồ thị HCM, một xe ô tô đang chạy hướng 178° trên đường Lê Văn Thọ, đã đi qua 80% cạnh 73056. Khách đứng ở phía sau, cách đó hơn một kilômét theo chiều ngược lại. Dijkstra theo nút trả lời «đi hết đoạn phía trước, quay ngược trên chính đường này, rồi rẽ trái vào Phạm Văn Chiêu»: 120,5 s. Tuyến đó cho xe quay đầu giữa đường, miễn phí, ở một nút giao đang chạy xe cả hai chiều. Khi cấm quay đầu giữa đường, tuyến hợp lệ phải đi vòng qua ba đường nối ngắn không tên, rẽ phải ba lần, rồi quay lại: 160,8 s và 1.145 m. Câu hỏi «tài xế mất bao lâu để tới khách» có hai đáp án cách nhau 40,3 s, tuỳ vào việc đồ thị có biết xe đang quay mặt đi đâu hay không.

Đây là bài 23 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng CSR và Dijkstra của [bài 21](/articles/location-dijkstra-a-star-rust-do-thi-csr) và quan hệ cấm rẽ mà [bài 06](/articles/location-osm-pbf-do-thi-duong-bo-xe-may-o-to) đã đọc từ OpenStreetMap. Chi phí rẽ trong bài (rẽ phải 3 s, rẽ trái 6 s, quay đầu 20 s) là **giả định của lab**, chưa hiệu chuẩn bằng dữ liệu thật. Toàn bộ dữ liệu chuyến là mô phỏng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Lab lấy 300 truy vấn: xe ở giữa một cạnh có hướng, khách ở giữa một cạnh khác, trên bản đồ HCM. Với mỗi truy vấn, tính tuyến bằng mô hình theo nút (không cấm rẽ, quay đầu miễn phí) rồi kiểm lại bằng luật đầy đủ. Bạn đoán bao nhiêu tuyến vi phạm luật?</p>
<details><summary>Xem đáp án</summary><p><b>149/300 tuyến (49,7%)</b> vi phạm cấm rẽ hoặc quay đầu giữa đường, và 181/300 tuyến (60,3%) có đoạn quay ngược ngay trên cùng một đoạn đường. Phần lớn là do giả định mạnh «xe không được quay đầu giữa đường»; riêng cấm rẽ OSM chỉ làm thời gian tăng trung bình 1,3 s. Dù vậy, một mô hình sai ở một nửa số truy vấn không dùng được cho bài toán chọn phía đường.</p></details>
</div>

## Nút không có hướng tới: ba thứ bị bỏ sót

Dijkstra ở bài 21 nhớ **nút** đã chốt và khoảng cách tới nó. Nút không cho biết xe vào nó từ cạnh nào, nên ba điều sau không thể biểu diễn:

1. **Quay đầu miễn phí.** Xe tới nút `v` theo cạnh `u → v` có thể đi tiếp bằng cạnh `v → u` (cạnh `twin`, cùng đoạn đường chiều ngược lại) và thuật toán không thấy gì bất thường. Ngoài đời, quay đầu giữa đường hai chiều thường bị cấm hoặc rất tốn thời gian.
2. **Cấm rẽ là quan hệ giữa hai cạnh.** «Cấm rẽ trái từ đường A vào đường B» chỉ đúng với cặp (cạnh vào A, cạnh ra B). Một nút giao có nhiều cạnh vào và nhiều cạnh ra; cấm một cặp không có chỗ để ghi trên nút.
3. **Chi phí rẽ.** Rẽ trái cắt dòng ngược chiều và mất thời gian hơn đi thẳng. Rẽ trái hai lần qua một lối mở dải phân cách (cách quay đầu phổ biến ở TP.HCM) không tốn thêm gì nếu chỉ cộng trọng số cạnh.

Cả ba cùng một gốc: trạng thái của tìm kiếm thiếu thông tin «tôi vừa đến từ đâu». Cách sửa là cho trạng thái chứa thông tin đó.

## Đồ thị theo cạnh: trạng thái là cạnh có hướng

Trong **đồ thị theo cạnh** (edge-based), một trạng thái là một cạnh có hướng, tức «xe đang đi trên cạnh `e`». Chuyển từ `e1` sang `e2` chỉ hợp lệ khi `head(e1) = tail(e2)` và cặp đó không bị cấm; chi phí là `trọng số của e2` cộng `chi phí rẽ(e1, e2)`. Chi phí rẽ bây giờ là thuộc tính của **chuyển trạng thái**, đúng chỗ của nó.

Đồ thị này **không được dựng ra bộ nhớ**: các chuyển trạng thái sinh ngay lúc duyệt từ CSR theo nút của bài 21. Số trạng thái bằng số cạnh (319.074), số chuyển trạng thái bằng tổng, trên mọi nút, của (số cạnh vào) × (số cạnh ra); không cần lưu. Phần lõi của A* theo cạnh chỉ khác bài 21 hai chỗ: khoá nhãn là chỉ số cạnh, và mỗi lần nới có thêm `turn_ms`.

`examples/toi-noi-location/rust/crates/route-engine/src/turn.rs`

```rust
            for e2 in g.out_edges(v) {
                let c = w[e2 as usize];
                if c == INF {
                    continue;
                }
                let Some(tc) = tm.turn_ms(e, e2) else {
                    continue;
                };
                st.relaxed += 1;
                let nd = d.saturating_add(tc).saturating_add(c);
                if nd < self.labels.dist(e2) {
                    self.labels.set(e2, nd, e);
                    self.heap.push(nd.saturating_add(h(e2)), e2);
                }
            }
```

`turn_ms` trả `None` khi cặp bị cấm, nên phép «bỏ qua chuyển trạng thái» chỉ là một `continue`. Heuristic chim bay của bài 21 vẫn hợp lệ vì chi phí rẽ không âm: `h(e)` tính từ nút cuối của `e` tới đích và vẫn chặn dưới tổng chi phí còn lại.

Hình 1 vẽ ca A của lab trên fixture «khu mẫu» (đồ thị giả lập): xe và khách ở cùng một chỗ nhưng ngược làn. Theo nút, xe đi tới nút phía trước rồi quay ngược: 12,7 s, 99 m. Theo cạnh với luật «không quay đầu giữa đường», xe phải đi vòng một khối: 98,7 s, 489 m, ba lần rẽ trái và một lần rẽ phải.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 372" role="img" aria-label="Hình 1. Ca A trên fixture: xe đang chạy hướng đông, khách ở làn ngược chiều ngay chỗ xe đứng. Mô hình theo nút cho quay ngược miễn phí tại nút phía trước: 12,7 s, 99 m. Mô hình theo cạnh cấm quay đầu giữa đường nên đi vòng một khối: 98,7 s, 489 m, ba lần rẽ trái và một lần rẽ phải."><rect x="20" y="20" width="410" height="332" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36" y="46" class="t-b">Theo nút: quay ngược miễn phí</text><rect x="450" y="20" width="410" height="332" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="466" y="46" class="t-b">Theo cạnh: cấm quay đầu giữa đường</text><path d="M50 180 L400 180" class="f-none s-dead" stroke-width="16" stroke-linejoin="round" stroke-linecap="round"/><path d="M90 170 L351.0 170.0" class="f-none s-phys" stroke-width="3" stroke-linecap="round"/><path d="M360 170 L351.0 174.5 L351.0 165.5 z" class="f-phys"/><path d="M360 190 L99.0 190.0" class="f-none s-phys" stroke-width="3" stroke-linecap="round"/><path d="M90 190 L99.0 185.5 L99.0 194.5 z" class="f-phys"/><text x="235" y="160" class="t-small" text-anchor="middle">Đường Ngang 2, làn đông</text><circle cx="130" cy="170" r="8" class="f-pk s-pk" stroke-width="1.5"/><text x="130" y="148" class="t-b" text-anchor="middle">xe</text><circle cx="130" cy="190" r="8" class="f-warn s-warn" stroke-width="1.5"/><text x="130" y="222" class="t-b" text-anchor="middle">khách (làn tây)</text><text x="235" y="246" class="t-small" text-anchor="middle">đi hết đoạn tới nút phía trước, quay ngược</text><text x="235" y="282" class="t-mono" text-anchor="middle">12,7 s · 99 m · 2 cạnh</text><text x="40" y="326" class="t-small">Nút không có hướng tới nên «quay ngược tại nút» không tốn gì.</text><path d="M480 120 L840 120" class="f-none s-dead" stroke-width="12" stroke-linejoin="round" stroke-linecap="round"/><path d="M590 120 L590 270" class="f-none s-dead" stroke-width="12" stroke-linejoin="round" stroke-linecap="round"/><path d="M590 270 L750 270" class="f-none s-dead" stroke-width="12" stroke-linejoin="round" stroke-linecap="round"/><path d="M750 270 L750 120" class="f-none s-dead" stroke-width="12" stroke-linejoin="round" stroke-linecap="round"/><path d="M530 111 L590 111 L590 270 L750 270 L750 121 L550 121" class="f-none s-phys" stroke-width="3" stroke-linejoin="round" stroke-linecap="round"/><circle cx="530" cy="111" r="8" class="f-pk s-pk" stroke-width="1.5"/><text x="530" y="92" class="t-b" text-anchor="middle">xe</text><circle cx="550" cy="121" r="7" class="f-warn s-warn" stroke-width="1.5"/><text x="510" y="142" class="t-b" text-anchor="middle">khách</text><text x="562" y="192" class="t-small" text-anchor="end">Dọc 1</text><text x="560" y="208" class="t-small" text-anchor="end">93 m, rẽ phải</text><text x="670" y="296" class="t-small" text-anchor="middle">Ngang 1: 97 m, rẽ trái</text><text x="762" y="200" class="t-small">Dọc 2: 98 m,</text><text x="762" y="216" class="t-small">rẽ trái</text><text x="665" y="98" class="t-small" text-anchor="middle">Ngang 2: 49 m đi, 151 m về (rẽ trái)</text><text x="466" y="326" class="t-mono">98,7 s · 489 m · 6 cạnh · rẽ trái 3, rẽ phải 1</text><text x="466" y="342" class="t-small">Hình tự dựng theo kết quả ca A; không theo tỉ lệ.</text></svg>
</div>
<figcaption>Hình 1. Ca A trên fixture (đồ thị giả lập, không phải dữ liệu OSM): xe chạy hướng đông trên Đường Ngang 2, khách ở làn ngược chiều ngay chỗ xe đứng. Trái: mô hình theo nút, 12,7 s, 99 m. Phải: mô hình theo cạnh, 98,7 s, 489 m (49 + 93 + 97 + 98 + 151 m, làm tròn). Hình không theo tỉ lệ.</figcaption>
</figure>

Có thể kiểm chứng rằng «mô hình theo nút» của lab là mô hình theo nút thật chứ không phải một phiên bản tự chế: bằng cách tắt mọi luật (chi phí rẽ bằng 0, quay đầu miễn phí, không cấm rẽ), tìm kiếm theo cạnh phải cho đúng chi phí của Dijkstra theo nút. Test `no_cost_model_equals_node_dijkstra_when_nothing_is_restricted` kiểm điều đó trên 100 cặp nút ngẫu nhiên của fixture, và `proptest_vs_dijkstra.rs` kiểm lại ở cả ba profile.

Điểm đầu và điểm cuối cũng khác nút. Khách và xe đứng ở **giữa** một cạnh có hướng, nên API có `Start::Pos(Position)` (phải đi hết phần còn lại của cạnh theo hướng đó rồi mới rẽ) và `Goal::Pos(Position)` (phải đi vào điểm đó theo chiều của cạnh, tức đúng phía đường). Đây là nền cho bài chọn điểm đón theo phía đường ở phần P04.

## Chi phí rẽ và quay đầu

Một lần rẽ được phân loại theo góc giữa hướng lúc rời `e1` (đoạn cuối của cạnh) và hướng lúc vào `e2` (đoạn đầu của cạnh). `bear_in` và `bear_out` tính một lần cho mọi cạnh.

`examples/toi-noi-location/rust/crates/route-engine/src/turn.rs`

```rust
    pub fn classify(&self, e1: EdgeId, e2: EdgeId) -> TurnKind {
        let g = self.view.g;
        if g.twin[e1 as usize] == e2 {
            return TurnKind::UTurn;
        }
        let d = (self.geom.bear_out[e2 as usize] - self.geom.bear_in[e1 as usize] + 540.0)
            .rem_euclid(360.0)
            - 180.0;
        let a = d.abs();
        if a >= self.costs.reversal_deg {
            TurnKind::UTurn
        } else if a < self.costs.straight_deg {
            TurnKind::Straight
        } else if d > 0.0 {
            TurnKind::Right
        } else {
            TurnKind::Left
        }
    }
```

Góc có dấu `d` nằm trong [−180°, 180°). Dưới 35° là đi thẳng; từ 150° trở lên hoặc đi vào cạnh `twin` là quay đầu; còn lại dương là rẽ phải (xe đi bên phải đường, hướng quay theo chiều kim đồng hồ), âm là rẽ trái. Chi phí từng loại là bảng `TurnCosts::urban()`: thẳng 0 s, rẽ phải 3 s, rẽ trái 6 s, quay đầu 20 s, cộng thêm 4 s khi đi vào lối qua dải phân cách. Đó là **giả định** của lab: rẽ trái đắt hơn vì cắt dòng ngược chiều, nhưng 3 s và 6 s là số đặt, không đo.

Quyết định lớn hơn là **quay đầu ở đâu**. Có ba chính sách:

`examples/toi-noi-location/rust/crates/route-engine/src/turn.rs`

```rust
        let mut cost = match kind {
            TurnKind::Straight => c.straight_ms,
            TurnKind::Right => c.right_ms,
            TurnKind::Left => c.left_ms,
            TurnKind::UTurn => match c.u_turn_policy {
                UTurnPolicy::Free => 0,
                UTurnPolicy::Costed => c.u_turn_ms,
                UTurnPolicy::DeadEndOnly => {
                    if self.dead_end(self.view.g.head[e1 as usize]) {
                        c.u_turn_ms
                    } else {
                        return None;
                    }
                }
            },
        };
```

`Free` là điều mô hình theo nút ngầm giả định. `Costed` cho quay đầu mọi nơi với 20 s. `DeadEndOnly`, mặc định của `urban()`, chỉ cho quay đầu trực tiếp ở ngõ cụt (nút mà mọi cạnh ra đều quay về nút vừa tới); muốn quay đầu giữa đường, xe phải đi vòng hoặc qua lối mở dải phân cách. Đây là giả định **mạnh**: xe máy thật quay đầu dễ hơn nhiều, và một số đường hai chiều ở TP.HCM có biển cho phép quay đầu. Lab giữ `Costed` làm phương án D để đo mức nhạy.

## Cấm rẽ trong OSM: dùng được hơn một phần tư

OSM ghi cấm rẽ bằng relation `type=restriction`: một member `from` (way vào), một member `to` (way ra) và một member `via` (nút hoặc way), với `restriction=no_left_turn`, `only_straight_on` và các giá trị tương tự. Quy ước `no_*` cấm cặp đó, `only_*` chỉ cho phép đúng cặp đó từ cạnh vào. Lab đọc chúng thành `banned` (tập cặp bị cấm) và `only` (từ cạnh vào, danh sách cạnh ra duy nhất được phép), rồi `forbids` kiểm trong lúc duyệt.

`examples/toi-noi-location/rust/crates/route-engine/src/turn.rs`

```rust
        for rs in &g.restrictions {
            r.stats.relations += 1;
            if rs.condition.is_some() || rs.key.ends_with(":conditional") {
                r.stats.conditional_skipped += 1;
                continue;
            }
            if !applies_key(&rs.key) {
                r.stats.other_mode_skipped += 1;
                continue;
            }
            if !rs.via_ways.is_empty() {
                r.stats.via_way_skipped += 1;
                continue;
            }
            if rs.from_edges.is_empty() || rs.to_edges.is_empty() {
                r.stats.unresolved_skipped += 1;
                continue;
            }
            if excepts(&rs.tags) {
                r.stats.excepted_skipped += 1;
                continue;
            }
            r.stats.applied += 1;
```

Những dòng `continue` trong khối này là **những điều lab không làm**, và chúng nhiều hơn điều lab làm. Trong bbox TP.HCM có 1.262 quan hệ. Với ô tô, lab chỉ áp dụng **345**: 327 quan hệ `no_*` (ra 327 cặp bị cấm) và 18 quan hệ `only_*`. Số còn lại bị bỏ qua có chủ đích:

- **784 quan hệ (62%) có điều kiện thời gian** (`restriction:conditional`, ví dụ cấm rẽ vào giờ cao điểm). Lab không có đồng hồ trong lúc tìm đường, nên bỏ qua hết. Hệ quả: một chỗ chỉ cấm rẽ trong khung giờ vẫn được coi là rẽ được cả ngày.
- **112 quan hệ có `via` là way thay vì nút** (cấm đi từ `from` qua cả đoạn `via` rồi vào `to`). Đồ thị theo cạnh với trạng thái một cạnh không biểu diễn được chúng; muốn làm cần trạng thái là một chuỗi cạnh. Con số 112 là số còn lại theo thứ tự lab kiểm (điều kiện thời gian trước, `via` sau); bài 06 đếm chung 270 relation có `via` là way, kể cả phần có điều kiện, nên hai số không cộng được.
- 11 quan hệ có thẻ `except` loại ô tô (199 loại xe máy), 7 quan hệ không khớp được với cạnh, 3 quan hệ dành cho loại phương tiện khác.

Với xe máy, lab áp dụng 159 quan hệ (141 `no_*`, 18 `only_*`), cặp bị cấm 141. Hình 3 vẽ phân bổ này. Kết luận phải nói thẳng: lab chỉ áp dụng 27% số quan hệ cho ô tô (345/1.262), nên cấm rẽ trong bài là **một phần của bức tranh**. Một tuyến mà lab báo «không vi phạm cấm rẽ» có thể vẫn vi phạm cấm rẽ theo giờ hoặc cấm rẽ có `via` là way ngoài đời.

## Dải phân cách: rẽ trái hai lần

Ở TP.HCM, nhiều đại lộ có dải phân cách chỉ cắt ở một số điểm. Muốn sang làn bên kia mà không quay đầu giữa đường, xe rẽ trái vào một lối mở rồi rẽ trái lần nữa. Dữ liệu OSM chuẩn không đánh dấu lối mở. Lab nhận diện bằng quy tắc hình học: cạnh ngắn (không quá 40 m) mà nút đầu và nút cuối đều nằm trên hai chiều **ngược hướng nhau** của đường lớn một chiều thì là lối qua dải phân cách; vào cạnh đó cộng thêm 4 s. Trên HCM, quy tắc này gắn nhãn 1.619 cạnh. Đó là quy tắc của lab, **chưa đối chiếu thủ công** với thực địa nên có thể nhận nhầm hoặc bỏ sót.

Ca B trên fixture cho thấy chi phí này lớn thế nào. Xe chạy chiều đông trên đại lộ, khách ở làn chiều tây cách khoảng 300 m. Cả hai mô hình chọn cùng một đường: 346 m chiều đông, rẽ trái qua «Đường Dọc 10» (lối qua dải phân cách, 20 m), rẽ trái lần nữa vào 44 m chiều tây, tổng 410 m. Nhưng mô hình theo nút tính 39,1 s, còn mô hình theo cạnh tính 55,1 s, trong đó 16,0 s là hai lần rẽ trái (2 × 6 s) và 4 s của lối qua dải phân cách. Chấm lại tuyến của mô hình theo nút bằng chi phí rẽ ra đúng 55,1 s. Cùng đường, hai đáp án khác nhau 16 s: mô hình theo nút ước lượng thấp thời gian đón.

## Đo trên bản đồ HCM

Phần 2 của lab chạy 300 truy vấn trên đồ thị HCM (126.699 nút, 319.074 cạnh), seed 20261106, ô tô. Bốn mô hình: **A** theo nút (không cấm rẽ, không chi phí rẽ, quay đầu miễn phí); **B** là A cộng cấm rẽ OSM; **C** là B cộng chi phí rẽ và cấm quay đầu giữa đường; **D** như C nhưng quay đầu được mọi nơi với 20 s.

| Đo | Kết quả |
|---|---|
| Tuyến của A vi phạm luật của C (cấm rẽ hoặc quay đầu giữa đường) | 149/300 (49,7%) |
| Tuyến của A có quay ngược ngay trên cùng một đoạn (vào cạnh twin) | 181/300 (60,3%) |
| Thời gian của B so với A (giây, dương = A ước thấp) | trung bình +1,3; trung vị 0,0; p95 +3,3; lớn nhất +64,3 |
| Thời gian của C so với A | trung bình +80,7; trung vị +78,4; p95 +137,0; lớn nhất +386,7 |
| Thời gian của D so với A | trung bình +72,0; trung vị +69,6; p95 +118,0; lớn nhất +147,9 |
| C so với A, tách theo tuyến của A | 151 tuyến hợp lệ: +64,5; 149 tuyến vi phạm: +97,1 |
| 151 tuyến của A hợp lệ, chấm lại bằng chi phí rẽ | cộng thêm trung bình 68,3 s |
| Số quay đầu hình học (tổng 300 truy vấn) | A 398 · C 343 · D 368 |
| Trạng thái cạnh đã duyệt | A 74.607 · C 75.953 |

Ba điều đáng đọc kỹ. **Một**: cấm rẽ OSM riêng lẻ gần như vô hại ở trung bình (+1,3 s, trung vị 0) nhưng đuôi dài: một truy vấn tăng 64,3 s. **Hai**: phần lớn chênh lệch (+80,7 s) đến từ **chi phí rẽ cộng dồn**, không phải từ luật quay đầu. Cho quay đầu mọi nơi với 20 s (mô hình D) vẫn +72,0 s, và ngay 151 tuyến của A vốn không vi phạm gì cũng chênh +64,5 s; riêng việc cấm hẳn quay đầu giữa đường chỉ thêm 8,7 s ở trung bình, nhưng đổi đuôi nhiều (lớn nhất +386,7 s so với +147,9 s). Mà chi phí rẽ 3 s và 6 s là số lab đặt, nên +80,7 s là hệ quả của bảng giả định, không phải ước lượng đời thật. **Ba**: thêm luật gần như không đổi số trạng thái phải duyệt (74.607 so với 75.953), vì cả A và C đều là A* trên trạng thái cạnh. Cái giá của sự đúng nằm ở độ phức tạp cài đặt, và ở chỗ tìm theo cạnh không dùng được CH của bài 25.

Về quay đầu, số «hình học» đếm trường hợp tổng góc rẽ đạt từ 150° trở lên trong một chuỗi nút cách nhau không quá 60 m; nó bắt được cả quay đầu trực tiếp lẫn «rẽ trái hai lần qua dải phân cách». Số này là xấp xỉ hình học, không phải nhận diện biển quay đầu, và nó là cái `u_turns` mà simulator ở bài 30 sẽ đo. Mô hình A có 398 lần: 199 đi ngược trực tiếp, 40 qua lối dải phân cách, 159 «khác» (chuỗi nút gần nhau cộng góc). Mô hình C có 343 lần: 116, 39 và 188. Con số «đi ngược trực tiếp» của C vẫn khác 0 vì quay đầu ở ngõ cụt được phép.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Ca thật trên HCM: xe ở 80% cạnh 73056 hướng 178°, khách ở 16% cạnh 6486. Mô hình theo nút đi ngược ngay trên Lê Văn Thọ: 120,5 s, 1.104 m. Mô hình theo cạnh đi vòng qua ba đường nối ngắn không tên, rẽ phải ba lần: 160,8 s, 1.145 m, chậm hơn 40,3 s."><rect x="20" y="20" width="840" height="290" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36" y="46" class="t-b">Một ca thật trên đồ thị HCM (ô tô, chi phí rẽ là giả định của lab)</text><text x="36" y="80" class="t-b">Theo nút (quay ngược miễn phí): 120,5 s · 1.104 m</text><rect x="36" y="92" width="250" height="46" rx="5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="161" y="112" class="t-small" text-anchor="middle">Lê Văn Thọ</text><text x="161" y="128" class="t-mono" text-anchor="middle">1.101 m</text><rect x="294" y="92" width="120" height="46" rx="5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="354" y="112" class="t-small" text-anchor="middle">Phạm Văn Chiêu</text><text x="354" y="128" class="t-mono" text-anchor="middle">4 m, rẽ trái</text><text x="426" y="120" class="t-b"></text><text x="36" y="180" class="t-b">Theo cạnh (cấm quay đầu giữa đường): 160,8 s · 1.145 m</text><rect x="36" y="192" width="70" height="46" rx="5" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="71" y="212" class="t-small" text-anchor="middle">Lê Văn Thọ</text><text x="71" y="228" class="t-mono" text-anchor="middle">8 m</text><rect x="114" y="192" width="100" height="46" rx="5" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="164" y="212" class="t-small" text-anchor="middle">đường không tên</text><text x="164" y="228" class="t-mono" text-anchor="middle">21 m, phải</text><rect x="222" y="192" width="100" height="46" rx="5" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="272" y="212" class="t-small" text-anchor="middle">đường không tên</text><text x="272" y="228" class="t-mono" text-anchor="middle">37 m, phải</text><rect x="330" y="192" width="100" height="46" rx="5" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="380" y="212" class="t-small" text-anchor="middle">đường không tên</text><text x="380" y="228" class="t-mono" text-anchor="middle">20 m, phải</text><rect x="438" y="192" width="120" height="46" rx="5" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="498" y="212" class="t-small" text-anchor="middle">Lê Văn Thọ</text><text x="498" y="228" class="t-mono" text-anchor="middle">1.056 m, trái</text><rect x="566" y="192" width="110" height="46" rx="5" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="621" y="212" class="t-small" text-anchor="middle">Phạm Văn Chiêu</text><text x="621" y="228" class="t-mono" text-anchor="middle">4 m, trái</text><text x="688" y="220" class="t-b"></text><text x="36" y="284" class="t-small">Các ô cùng độ rộng cho dễ đọc, không theo tỉ lệ. Chênh lệch +40,3 s; đường đi tăng 41 m. Nguồn: results/p03.md, LOC-23 mục 2.</text><text x="36" y="300" class="t-small">Tên đường © OpenStreetMap contributors (ODbL); way không tên được ghi theo mã way trong lab.</text></svg>
</div>
<figcaption>Hình 2. Ca thật trên HCM: xe ở 80% cạnh 73056 (hướng 178°), khách ở 16% cạnh 6486. Theo nút: 120,5 s, 1.104 m, một quay đầu miễn phí. Theo cạnh: 160,8 s, 1.145 m. Các ô cùng độ rộng, không theo tỉ lệ. Tên đường © OpenStreetMap contributors (ODbL).</figcaption>
</figure>

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 372" role="img" aria-label="Hình 3. Trên: 1.262 quan hệ cấm rẽ OSM trong bbox được xử lý ra sao: ô tô áp dụng 345, bỏ qua 784 có điều kiện thời gian, 112 có via là way. Dưới: 149/300 tuyến của mô hình theo nút (49,7%) vi phạm luật rẽ đầy đủ."><rect x="20" y="20" width="840" height="332" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36" y="46" class="t-b">1.262 quan hệ cấm rẽ OSM trong bbox: lab dùng được bao nhiêu</text><text x="36" y="81" class="t-b">ô tô</text><rect x="120" y="64" width="197.924" height="26" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="218.962" y="81" class="t-small" text-anchor="middle">áp dụng 345</text><rect x="317.924" y="64" width="449.775" height="26" rx="2" class="f-warn-soft s-warn" stroke-width="1"/><text x="542.811" y="81" class="t-small" text-anchor="middle">điều kiện thời gian 784: bỏ qua</text><rect x="767.699" y="64" width="64.2536" height="26" rx="2" class="f-dead s-dead" stroke-width="1"/><text x="799.826" y="81" class="t-small" text-anchor="middle">112</text><rect x="831.952" y="64" width="6.31062" height="26" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><rect x="838.263" y="64" width="5.73693" height="26" rx="2" class="f-muted s-muted" stroke-width="1"/><text x="36" y="121" class="t-b">xe máy</text><rect x="120" y="104" width="91.2171" height="26" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="165.609" y="121" class="t-small" text-anchor="middle">159</text><rect x="211.217" y="104" width="449.775" height="26" rx="2" class="f-warn-soft s-warn" stroke-width="1"/><text x="436.105" y="121" class="t-small" text-anchor="middle">điều kiện thời gian 784: bỏ qua</text><rect x="660.992" y="104" width="64.2536" height="26" rx="2" class="f-dead s-dead" stroke-width="1"/><text x="693.119" y="121" class="t-small" text-anchor="middle">112</text><rect x="725.246" y="104" width="114.165" height="26" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="782.328" y="121" class="t-small" text-anchor="middle">199</text><rect x="839.41" y="104" width="4.58954" height="26" rx="2" class="f-muted s-muted" stroke-width="1"/><rect x="36" y="141" width="16" height="11" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="60" y="150" class="t-small">áp dụng được (cặp cạnh vào, cạnh ra bị cấm hoặc chỉ được)</text><rect x="36" y="159" width="16" height="11" rx="2" class="f-warn-soft s-warn" stroke-width="1"/><text x="60" y="168" class="t-small">có điều kiện thời gian (restriction:conditional): bỏ qua</text><rect x="36" y="177" width="16" height="11" rx="2" class="f-dead s-dead" stroke-width="1"/><text x="60" y="186" class="t-small">via là way: bỏ qua (112 sau khi đã tách nhóm điều kiện)</text><rect x="36" y="195" width="16" height="11" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="60" y="204" class="t-small">có thẻ except loại trừ phương tiện đó: ô tô 11, xe máy 199</text><rect x="36" y="213" width="16" height="11" rx="2" class="f-muted s-muted" stroke-width="1"/><text x="60" y="222" class="t-small">không khớp cạnh hoặc loại phương tiện khác: ô tô 7 + 3, xe máy 7 + 1</text><text x="36" y="262" class="t-b">Mô hình theo nút vi phạm luật rẽ đầy đủ ở bao nhiêu tuyến (300 truy vấn)</text><text x="36" y="288" class="t-small">vi phạm cấm rẽ hoặc quay đầu giữa đường</text><rect x="330" y="274" width="159.04" height="20" rx="3" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="497.04" y="288" class="t-mono">149/300 (49,7%)</text><text x="36" y="316" class="t-small">có quay ngược ngay trên cùng một đoạn</text><rect x="330" y="302" width="192.96" height="20" rx="3" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="530.96" y="316" class="t-mono">181/300 (60,3%)</text><text x="36" y="342" class="t-small">Số đếm, tái lập với seed 20261106. Bản đồ © OpenStreetMap contributors (ODbL).</text></svg>
</div>
<figcaption>Hình 3. Trên: 1.262 quan hệ cấm rẽ trong bbox HCM được dùng ra sao, cho ô tô và xe máy; tỉ lệ theo số quan hệ. Dưới: tỉ lệ tuyến của mô hình theo nút vi phạm luật đầy đủ. Số đếm tái lập từng chữ số với seed 20261106. © OpenStreetMap contributors (ODbL).</figcaption>
</figure>

Ca thật ở Hình 2 là ví dụ cần nhớ. Xe đang chạy theo hướng 178° và khách ở phía sau. Mô hình theo nút đi hết 20% cạnh còn lại rồi quay ngược trên Lê Văn Thọ. Mô hình theo cạnh phải luồn qua ba cạnh nối ngắn, không tên, rẽ phải ba lần để quay về đúng đường rồi mới đi ngược. 40,3 s chênh lệch là thời gian đón khách chênh nhau giữa hai mô hình, và nếu km rỗng của tài xế tính theo mô hình theo nút thì sẽ thấp hơn thực tế.

## Hệ quả cho các bài sau

Mô hình theo cạnh đúng hơn nhưng có giá: mỗi trạng thái là một cạnh, và **Contraction Hierarchies của bài 25 là theo nút**, nên nó và ma trận chi phí many-to-many dựa trên nó không có chi phí rẽ. Chỉ A* theo cạnh của bài này (và simulator) có chi phí rẽ, cấm rẽ và quay đầu. Bài 25 nói rõ hệ quả ấy; ở đây chỉ cần nhớ rằng chọn «mô hình theo nút nhanh» hay «theo cạnh đúng» là một lựa chọn có chủ đích ở mỗi bài.

## Chạy ví dụ và kiểm chứng

Phần 1 luôn chạy trên fixture «khu mẫu»; phần 2 cần `data/hcm-graph.bin` (hoặc thêm `-- --fixture`). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc23_re_u_turn
```

Cấu hình đo: `--release`, Apple M1 Max, 300 truy vấn, seed 20261106, ô tô. Output (rút gọn) của lần chạy duy nhất. Số đếm và tỉ lệ không phụ thuộc thời gian; máy lúc đó chịu tải nặng (tải 1 phút 21,34), nên bỏ qua cột µs:

```text
-- 1. ca dựng sẵn trên fixture «khu mẫu» (ô tô) --
  ca A. Xe đang chạy trên đường hai chiều, khách đứng ở làn ngược chiều ngay chỗ xe đang đứng
    mô hình theo nút (quay đầu miễn phí): 12.7 s, 99 m, 2 cạnh, quay đầu 1, rẽ trái/phải/quay [0, 0, 1]
    theo cạnh, cấm quay đầu giữa đường: 98.7 s, 489 m, 6 cạnh, quay đầu 0, rẽ trái/phải/quay [3, 1, 0]
  ca B. Xe chạy chiều đông trên đại lộ có dải phân cách, khách đứng ở làn chiều tây (cách khoảng 300 m về phía trước)
    mô hình theo nút: 39.1 s (trong đó chi phí rẽ 0.0 s), 410 m, rẽ trái 2 phải 0 quay 0, quay đầu 1
    theo cạnh, có chi phí rẽ: 55.1 s (trong đó chi phí rẽ 16.0 s), 410 m, rẽ trái 2 phải 0 quay 0, quay đầu 1
    chấm lại tuyến của mô hình theo nút bằng chi phí rẽ: 55.1 s thay vì 39.1 s
  ca C. Cấm rẽ OSM: xe đi trên cạnh vào, khách ở trên cạnh ra bị cấm (mô hình theo nút rẽ thẳng vào đó)
    cấm Đường Dọc 1 (nam) → Đại lộ Trung Tâm (chiều đông): theo nút 14.1 s (RẼ VÀO CHỖ BỊ CẤM) ; có cấm rẽ 63.2 s (+49.1 s, +371 m)
    cấm Đại lộ Trung Tâm (chiều tây) → Đường Dọc 7 (bắc): theo nút 13.7 s (RẼ VÀO CHỖ BỊ CẤM) ; có cấm rẽ 68.5 s (+54.8 s, +383 m)

-- 2. thống kê trên bản đồ: examples/toi-noi-location/data/hcm-graph.bin --
  cấm rẽ cho ô tô: 1262 quan hệ → áp dụng 345 (no_* 327, only_* 18), cặp bị cấm 327; bỏ qua: điều kiện thời gian 784, không khớp cạnh 7, via-way 112, loại phương tiện khác 3, ngoại lệ 11
  cấm rẽ cho xe máy: 1262 quan hệ → áp dụng 159 (no_* 141, only_* 18), cặp bị cấm 141; bỏ qua: điều kiện thời gian 784, không khớp cạnh 7, via-way 112, loại phương tiện khác 1, ngoại lệ 199
  cạnh nhận diện là lối qua dải phân cách (nút đầu và cuối đều nằm trên hai chiều ngược nhau của đường lớn một chiều): 1.619
  tuyến của A vi phạm cấm rẽ hoặc quay đầu giữa đường theo luật của C: 149/300 (49.7%)
  tuyến của A có quay ngược ngay trên cùng một đoạn (đi vào cạnh twin): 181/300 (60.3%)
    B (có cấm rẽ)  trung bình +1.3, trung vị +0.0, p95 +3.3, lớn nhất +64.3
    C (đầy đủ)     trung bình +80.7, trung vị +78.4, p95 +137.0, lớn nhất +386.7
    D (20 s)       trung bình +72.0, trung vị +69.6, p95 +118.0, lớn nhất +147.9
  C so với A, tách theo tuyến của A: hợp lệ (151) trung bình +64.5 s; vi phạm (149) trung bình +97.1 s
  một ca thật trên bản đồ này: xe ở 80% cạnh 73056 (hướng 178°), khách ở 16% cạnh 6486; theo nút đi ngược ngay còn theo cạnh phải đi vòng 1145 m, chậm hơn 40.3 s
```

Ca C nói riêng ghi lại điều mô hình theo nút làm sai nhất: nó cho xe **rẽ thẳng vào chỗ bị cấm** ở cả hai cặp cấm của fixture (14,1 s và 13,7 s), trong khi tuân thủ cấm tốn thêm 49,1 s (371 m) và 54,8 s (383 m).

Kiểm tự động trong `turn.rs`: A* theo cạnh cho cùng chi phí với Dijkstra theo cạnh khi có chi phí rẽ, chi phí tìm được khớp chi phí chấm lại cả tuyến bằng `path_cost`, chi phí rẽ không bao giờ làm tuyến rẻ đi, cặp bị cấm không bao giờ được dùng, chính sách `DeadEndOnly` cấm quay đầu giữa đường.

## Giới hạn của ví dụ

- **Chi phí rẽ là giả định.** 3 s, 6 s, 20 s và 4 s qua dải phân cách do lab đặt; chưa đo từ dữ liệu thật hay hiệu chuẩn bằng simulator.
- **Luật quay đầu quá chặt.** `DeadEndOnly` cấm quay đầu giữa đường ở mọi nơi, kể cả rẽ gắt từ 150° trở lên ở nút giao hình chữ Y; đó là lý do đuôi +386,7 s dài như vậy. Đường thật có chỗ cho phép quay đầu, xe máy quay đầu dễ hơn.
- **Cấm rẽ chỉ dùng được một phần.** Bỏ qua 784 trên 1.262 quan hệ có điều kiện thời gian và 112 quan hệ có `via` là way; không có đồng hồ trong lúc tìm đường, và trạng thái một cạnh không biểu diễn được `via` là way.
- **Nhận diện dải phân cách bằng hình học, chưa đối chiếu thủ công.** 1.619 cạnh có thể gồm nhầm và sót.
- **Quay đầu hình học là xấp xỉ.** Ngưỡng 150° và 60 m là lựa chọn của lab, không phải nhận diện biển.
- **Hướng xe là đầu vào.** Lab giả định xe đi hết phần còn lại của cạnh theo hướng đang chạy; hướng lấy từ GPS nhiễu mà sai thì tuyến đổi theo (bài bám đường 16, 17). Phần 2 chỉ đo ô tô.

## Bài tập

**Bài 1.** Ở ca B, mô hình theo nút tính 39,1 s còn mô hình theo cạnh tính 55,1 s dù đi cùng đường. Giải thích con số chênh 16,0 s từ bảng chi phí rẽ.

<details><summary>Lời giải bài 1</summary>
<p>Tuyến có hai lần rẽ trái: 2 × 6 s = 12 s. Đi vào lối qua dải phân cách (Đường Dọc 10) cộng thêm 4 s. Tổng 16 s, khớp «chi phí rẽ 16,0 s» của lab. Mô hình theo nút chỉ cộng trọng số cạnh nên bỏ cả 16 s đó. Nếu một lần rẽ trái là 4 s, chênh lệch sẽ còn 12 s: kết luận phụ thuộc vào bảng giả định.</p>
</details>

**Bài 2.** Vì sao mô hình theo nút của lab (A) có thể kiểm bằng cách so với Dijkstra thông thường, trong khi mô hình C thì không? Kiểm C bằng cách nào?

<details><summary>Lời giải bài 2</summary>
<p>Khi tắt hết luật (chi phí rẽ bằng 0, quay đầu miễn phí, không cấm rẽ), tìm kiếm theo cạnh phải trả đúng chi phí Dijkstra theo nút, nên có thể so trực tiếp (test <code>no_cost_model_equals_node_dijkstra_when_nothing_is_restricted</code>). Với C không có thuật toán nút tương đương. Lab kiểm C bằng các tính chất: A* và Dijkstra theo cạnh cùng chi phí; chi phí tìm được bằng chi phí chấm lại cả tuyến bằng <code>path_cost</code>; chi phí rẽ không bao giờ làm tuyến rẻ đi so với A; cặp bị cấm không xuất hiện trong tuyến.</p>
</details>

**Bài 3.** Một quan hệ cấm rẽ có `restriction:conditional = no_left_turn @ (Mo-Fr 16:00-19:00)`. Lab xử lý nó ra sao và sai lệch nào có thể xảy ra với chuyến đón lúc 17:30 thứ Hai? Cần thêm gì để xử lý đúng?

<details><summary>Lời giải bài 3</summary>
<p>Lab bỏ qua hẳn quan hệ này (đếm vào nhóm 784 «điều kiện thời gian»), nên tuyến được phép rẽ trái ở chỗ đó cả ngày; ở 17:30 thứ Hai tuyến này vi phạm cấm rẽ, và thời gian ước tính có thể thấp hơn thực tế vì đoạn đi vòng không được tính. Xử lý đúng cần: đọc điều kiện thời gian của OSM, truyền giờ truy vấn vào tìm kiếm, và chỉ áp dụng cặp bị cấm khi giờ rơi vào khung. Việc đó đụng tới bài 27 (tốc độ theo giờ) vì hai thứ cùng phụ thuộc đồng hồ.</p>
</details>

Bài trước: [bài 22](/articles/location-bidirectional-alt-landmark-tang-toc-truy-van). Bài tiếp: [bài 24](/articles/location-profile-xe-may-o-to-di-bo-trong-hem).

## Tài liệu gốc

- [Turn restrictions trên OSM wiki](https://wiki.openstreetmap.org/wiki/Turn_restrictions): quy ước cấm rẽ trong OpenStreetMap.
- [Relation:restriction trên OSM wiki](https://wiki.openstreetmap.org/Relation:restriction): các giá trị `no_*`, `only_*`, thành viên `from`, `via`, `to`, và `via` là một hoặc nhiều way.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho hình có dữ liệu OSM.
