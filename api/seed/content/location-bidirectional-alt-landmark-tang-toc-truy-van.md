---
slug: location-bidirectional-alt-landmark-tang-toc-truy-van
title: "Bidirectional và ALT: giảm số nút duyệt mà vẫn đúng"
summary: "Bài 22: Dijkstra hai chiều với điều kiện dừng đúng, lỗi dừng ở nút gặp đầu tiên cho chi phí sai ở 29/300 truy vấn HCM, ALT với landmark và bất đẳng thức tam giác, tiền xử lý đổi lấy số nút đã duyệt."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, routing, bidirectional, alt, landmark, a-star, openstreetmap]
language: vi
---

Hai nút cách nhau 335 m chim bay trên đồ thị HCM: nút 97043 và nút 2643. Dijkstra cho đáp án 78,7 s. Một bản Dijkstra hai chiều «gặp nhau ở giữa là dừng» cho 83,1 s, chậm hơn 5,69%, và không có dòng log nào báo lỗi. Trên 300 cặp ngẫu nhiên, bản dừng sai trả chi phí sai ở 29 cặp, gần một phần mười. Đây là lỗi kinh điển của tìm hai chiều, và nó chỉ lộ ra khi so từng đáp án với Dijkstra một chiều.

Đây là bài 22 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài tiếp nối [bài 21](/articles/location-dijkstra-a-star-rust-do-thi-csr): từ Dijkstra và A* trên CSR, giảm số nút phải duyệt bằng hai cách (tìm từ cả hai đầu, và heuristic landmark) mà vẫn giữ đúng chi phí. Số liệu là số đếm nút đã duyệt trên đồ thị HCM; số thời gian đo khi máy đang chịu tải nặng nên chỉ dùng làm tỉ lệ trong cùng một lần chạy.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Tìm hai chiều chạy Dijkstra từ xe tới khách và từ khách ngược về xe. Khi một nút lần đầu được <b>cả hai</b> chiều chốt, ta dừng và trả tổng hai khoảng cách tại nút đó. Cách dừng này có luôn cho đường ngắn nhất không?</p>
<details><summary>Xem đáp án</summary><p>Không. Trên đồ thị năm nút của lab, đường tối ưu dài 12 nhưng nút m được cả hai chiều chốt trước và cách dừng này trả 13. Trên bản đồ HCM nó sai ở 29/300 truy vấn (9,7%). Lỗi nằm ở chỗ trả tổng tại nút gặp: lúc đó lab đã biết một đường 12. Điều kiện chuẩn là dừng khi <code>top_f + top_b ≥ μ</code> rồi trả μ, với μ là đường tốt nhất đã thấy; phần dưới chứng minh vì sao.</p></details>
</div>

## Tìm hai chiều: hai vòng tròn nhỏ thay cho một vòng lớn

Dijkstra từ xe lan thành một «vòng tròn» chi phí tăng dần quanh xe và chỉ dừng khi chạm khách. Nếu khách cách xe khoảng cách `D`, vòng đó phủ diện tích cỡ `D²`. Tìm hai chiều chạy hai Dijkstra cùng lúc: một từ xe theo chiều cạnh, một từ khách theo **chiều ngược** của cạnh, mỗi bên chỉ cần lan tới khoảng `D/2`. Hai vòng bán kính `D/2` có tổng diện tích cỡ một nửa vòng lớn, tức phép tính thô hứa hẹn giảm khoảng 2 lần số nút. Phần đo bên dưới sẽ kiểm lại con số này.

Chiều ngược cần CSR ngược: với mỗi nút `v`, danh sách cạnh đi **vào** `v`. `Rev` dựng nó một lần, không phụ thuộc profile, còn `View::inn` trả cạnh vào đã lọc theo trọng số của profile (bài 21). Hai bên có hai bộ nhãn và hai hàng đợi riêng; mỗi vòng lặp mở rộng bên có khoá nhỏ nhất ở đỉnh hàng đợi.

`examples/toi-noi-location/rust/crates/route-engine/src/bidir.rs`

```rust
            // mở rộng chiều có khoá nhỏ hơn
            if kf <= kb {
                let (_, u) = self.hf.pop().expect("đã kiểm tra");
                self.fwd.close(u);
                st.settled += 1;
                let d = self.fwd.dist(u);
                if self.bwd.is_closed(u) && first_meet == NONE {
                    first_meet = u;
                }
                for (e, v, c) in view.out(u) {
                    st.relaxed += 1;
                    let nd = d.saturating_add(c);
                    if nd < self.fwd.dist(v) {
                        self.fwd.set(v, nd, e);
                        self.hf.push(nd, v);
                    }
                    let db = self.bwd.dist(v);
                    if db != INF && nd.saturating_add(db) < mu {
                        mu = nd + db;
                        meet = v;
                    }
                }
            }
```

Hai thứ nằm ngoài vòng lặp mở rộng quen thuộc. Một: `mu`, chi phí của đường s→t tốt nhất đã thấy, được cập nhật **mỗi khi nới một cạnh chạm vào nút mà chiều kia đã có nhãn**, không đợi tới lúc chốt. Hai: `first_meet`, nút đầu tiên được cả hai chiều chốt, chỉ để bản dừng sai dùng. Phía ngược lại (`else`) là hình gương: lấy từ hàng đợi ngược, duyệt `view.inn(u)`, so với nhãn xuôi.

Còn một chi tiết cài đặt: sau khi nhãn của một nút giảm, hàng đợi giữ cả mục cũ. `clean_top` bỏ các mục đã chốt hoặc có khoá khác nhãn hiện tại, để «khoá nhỏ nhất ở đỉnh» luôn là khoá thật. Điều này quan trọng vì điều kiện dừng đọc đúng hai khoá này.

## Lỗi kinh điển: dừng ở nút gặp đầu tiên

Trực giác sai là «khi hai bên gặp nhau thì đáp án nằm ở chỗ gặp». Nhưng «gặp nhau» có nghĩa là một nút được **chốt ở cả hai chiều**, và nút đó không nhất thiết nằm trên đường ngắn nhất. Biến thể `FirstMeeting` của lab dừng ngay lúc đó rồi trả `d_F(m) + d_B(m)`. Phản ví dụ năm nút dưới đây cho thấy nó sai:

`examples/toi-noi-location/rust/crates/route-engine/src/bidir.rs`

```rust
    let es: [(u32, u32, u32); 5] = [(0, 1, 1), (1, 3, 10), (3, 4, 1), (0, 2, 6), (2, 4, 7)];
```

Đặt tên cho dễ nhớ: `s` là xe, `t` là khách, `a` và `b` là hai ngã ba trên một đường dài, `m` là đường vòng qua nút giữa. Đường s→a→b→t có cạnh ở giữa dài 10, tổng 12. Đường s→m→t tổng 13. Chạy từng bước (Hình 1): xuôi chốt `s`, ngược chốt `t` và chạm `m` (μ = 13), xuôi chốt `a` và chạm `b` (μ = 12), ngược chốt `b`. Đến đây khoá nhỏ nhất của hai bên là 6 (nút `m` chiều xuôi) và 7 (nút `m` chiều ngược). Bản dừng sai tiếp tục: xuôi chốt `m` với khoảng cách 6, ngược chốt `m` với khoảng cách 7, và bây giờ `m` đã chốt ở cả hai chiều. Nó dừng và trả `6 + 7 = 13`, đường qua `m`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 388" role="img" aria-label="Hình 1. Ví dụ năm nút của lab: đường tối ưu s, a, b, t dài 12, đường qua m dài 13. Dừng ở nút đầu tiên được cả hai chiều chốt (m) trả 13. Điều kiện đúng top_f + top_b ≥ μ dừng sau bước 4 và trả 12."><rect x="20" y="20" width="420" height="348" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36" y="46" class="t-b">Đồ thị (trọng số là giây)</text><path d="M82.3412 198.308 L146.1 137.9" class="f-none s-phys" stroke-width="3" stroke-linecap="round"/><path d="M152.659 131.692 L149.2 141.1 L143.0 134.6 z" class="f-phys"/><path d="M182 120 L274.0 120.0" class="f-none s-phys" stroke-width="3" stroke-linecap="round"/><path d="M283 120 L274.0 124.5 L274.0 115.5 z" class="f-phys"/><path d="M312.341 131.692 L376.1 192.1" class="f-none s-phys" stroke-width="3" stroke-linecap="round"/><path d="M382.659 198.308 L373.0 195.4 L379.2 188.9 z" class="f-phys"/><path d="M84.8607 218.256 L209.3 287.4" class="f-none s-warn" stroke-width="3" stroke-linecap="round"/><path d="M217.139 291.744 L207.1 291.3 L211.5 283.4 z" class="f-warn"/><path d="M246.882 291.783 L372.2 222.6" class="f-none s-warn" stroke-width="3" stroke-linecap="round"/><path d="M380.118 218.217 L374.4 226.5 L370.1 218.6 z" class="f-warn"/><circle cx="70" cy="210" r="16" class="f-phys-soft s-phys" stroke-width="2"/><text x="70" y="214" class="t-b" text-anchor="middle">s</text><circle cx="165" cy="120" r="16" class="f-phys-soft s-phys" stroke-width="2"/><text x="165" y="124" class="t-b" text-anchor="middle">a</text><circle cx="300" cy="120" r="16" class="f-phys-soft s-phys" stroke-width="2"/><text x="300" y="124" class="t-b" text-anchor="middle">b</text><circle cx="395" cy="210" r="16" class="f-phys-soft s-phys" stroke-width="2"/><text x="395" y="214" class="t-b" text-anchor="middle">t</text><circle cx="232" cy="300" r="16" class="f-warn-soft s-warn" stroke-width="2"/><text x="232" y="304" class="t-b" text-anchor="middle">m</text><text x="104" y="150" class="t-mono" text-anchor="middle">1</text><text x="232" y="106" class="t-mono" text-anchor="middle">10</text><text x="362" y="150" class="t-mono" text-anchor="middle">1</text><text x="128" y="268" class="t-mono" text-anchor="middle">6</text><text x="342" y="268" class="t-mono" text-anchor="middle">7</text><text x="36" y="350" class="t-small">Đúng: s, a, b, t = 12. Đường qua m dài 13.</text><rect x="460" y="20" width="400" height="348" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="476" y="46" class="t-b">Thứ tự chốt (F = chiều xuôi từ s, B = chiều ngược từ t)</text><circle cx="488" cy="74" r="10" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="488" y="78" class="t-small" text-anchor="middle">1</text><text x="508" y="78">F chốt s (0)</text><circle cx="488" cy="102" r="10" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="488" y="106" class="t-small" text-anchor="middle">2</text><text x="508" y="106">B chốt t (0); chạm m: μ = 13</text><circle cx="488" cy="130" r="10" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="488" y="134" class="t-small" text-anchor="middle">3</text><text x="508" y="134">F chốt a (1); chạm b: μ = 12</text><circle cx="488" cy="158" r="10" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="488" y="162" class="t-small" text-anchor="middle">4</text><text x="508" y="162">B chốt b (1)</text><circle cx="488" cy="186" r="10" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="488" y="190" class="t-small" text-anchor="middle">5</text><text x="508" y="190">F chốt m (6)</text><circle cx="488" cy="214" r="10" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="488" y="218" class="t-small" text-anchor="middle">6</text><text x="508" y="218">B chốt m (7): m chốt ở cả hai chiều</text><rect x="476" y="250" width="368" height="50" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="488" y="270" class="t-b">Dừng đúng sau bước 4: 6 + 7 = 13 ≥ μ = 12, trả 12.</text><text x="488" y="288" class="t-small">Sau bước 4 mọi đường chưa thấy dài ít nhất 13.</text><rect x="476" y="310" width="368" height="50" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="488" y="330" class="t-b">Dừng sai sau bước 6: d_F(m) + d_B(m) = 6 + 7 = 13.</text><text x="488" y="348" class="t-small">Trả đường s, m, t dài 13 trong khi có đường 12.</text></svg>
</div>
<figcaption>Hình 1. Đồ thị năm nút của lab (s=0, a=1, m=2, b=3, t=4), trọng số tính bằng giây. Trái: đường đúng s, a, b, t = 12 (xanh) và đường qua m = 13 (cam). Phải: sáu bước chốt theo đúng thứ tự lab. Dừng đúng sau bước 4 vì 6 + 7 ≥ 12; dừng sai sau bước 6 vì m chốt ở cả hai chiều. Ví dụ dựng tay.</figcaption>
</figure>

Mấu chốt là đường tối ưu **vắt qua biên** của hai vùng đã chốt bằng một cạnh dài (a→b). Hai đầu của cạnh đó là hai nút mà mỗi chiều mới chỉ thấy một: `a` được xuôi chốt, `b` được ngược chốt, không nút nào được cả hai chốt. Nút `m`, rẻ hơn ở hai đầu nhưng tổng đắt hơn, được cả hai chốt trước.

Lưu ý chỗ sai thật sự. Sau bước 6, `μ` đã bằng 12 từ bước 3, nên nếu dừng ở đó mà trả `μ` thì đáp án vẫn đúng: một nút `m` đã chốt ở cả hai chiều có `top_f ≥ d_F(m)` và `top_b ≥ d_B(m)`, mà `μ ≤ d_F(m) + d_B(m)`, tức điều kiện đúng ở mục sau đã thoả. Thời điểm đó không sai, chỉ muộn hơn cần thiết; cái sai là đáp án lấy từ nút gặp thay vì từ `μ`. Bản không giữ `μ` thì không có gì khác để trả.

Lab chạy cùng lỗi trên bản đồ thật: 300 truy vấn ngẫu nhiên, seed 20261106. Bản trả tổng tại nút gặp đầu tiên cho chi phí sai ở **29/300 truy vấn (9,7%)**. Đường sai dài hơn đường đúng trung bình 0,71%, trung vị 0,20%, lớn nhất 5,69%: ca tệ nhất là cặp mở đầu bài, 83,1 s thay vì 78,7 s. Đây chính là loại lỗi nguy hiểm: sai ít, ở khá nhiều truy vấn, và trung vị 0,20% đủ nhỏ để chẳng ai thấy khi chạy thử bằng mắt.

## Điều kiện dừng đúng: top_f + top_b ≥ μ

Điều kiện đúng đọc: giữ `μ` là chi phí đường tốt nhất đã thấy; gọi `top_f` và `top_b` là khoá nhỏ nhất còn trong hai hàng đợi. **Dừng khi `top_f + top_b ≥ μ`, rồi trả μ.**

`examples/toi-noi-location/rust/crates/route-engine/src/bidir.rs`

```rust
            match stop {
                Stop::Correct => {
                    if mu != INF && u64::from(kf) + u64::from(kb) >= u64::from(mu) {
                        break;
                    }
                }
                Stop::FirstMeeting => {
                    if first_meet != NONE {
                        break;
                    }
                }
            }
```

Vì sao đúng? Gọi `P` là đường ngắn nhất, dài `L*`, và giả sử lúc dừng `L* < μ`, tức `top_f + top_b ≥ μ > L*`. Trong Dijkstra, mọi nút có khoảng cách thật nhỏ hơn khoá nhỏ nhất của hàng đợi đều đã được chốt. Đi dọc `P` từ `s`, gọi `x` là nút cuối cùng có `d(s, x) < top_f`, và `y` là nút kế tiếp, có `d(s, y) ≥ top_f`. Vì `P` là đường ngắn nhất nên `d(y, t) = L* − d(s, y) ≤ L* − top_f < top_b`, nên `y` đã được chiều ngược chốt, tức có nhãn ngược bằng `d(y, t)`. Còn `x` đã được chiều xuôi chốt, nên cạnh `x → y` đã được nới và `y` có nhãn xuôi bằng `d(s, y)`. Nhãn nào đến sau thì lần nới gắn nhãn đó thấy nhãn kia đã có, và dòng `if db != INF && nd + db < mu` (hoặc hình gương của nó ở chiều ngược) đã cập nhật `μ ≤ d(s, y) + d(y, t) = L*`, mâu thuẫn với `L* < μ`. Chứng minh này giải thích vì sao `μ` phải cập nhật lúc **nới cạnh** chứ không đợi một nút được chốt ở cả hai chiều: cạnh `a → b` ở Hình 1 là chính cạnh làm lộ ra đường 12.

Trên 300 truy vấn HCM, điều kiện đúng cho **0 khác biệt** so với Dijkstra một chiều. Trong ví dụ năm nút, `top_f + top_b = 6 + 7 = 13 ≥ 12` đúng sau bước 4, hai bước trước khi bản sai dừng. Rút ra quy tắc cho mọi thuật toán «tìm từ hai đầu» về sau: **đáp án là `μ`, không phải nút chạm nhau; dừng khi tổng hai khoá nhỏ nhất không còn nhỏ hơn `μ`**.

## ALT: A* với landmark và bất đẳng thức tam giác

Bài 21 để lại một vấn đề: với trọng số thời gian, heuristic chim bay lỏng (A* giảm chỉ 1,43–2,15 lần số nút) vì phải giả định xe chạy 80 km/h khắp nơi. ALT (A*, Landmarks, Triangle inequality) thay chim bay bằng một chặn dưới **học từ chính đồ thị**.

Chọn `k` nút làm landmark `L`. Với mỗi landmark, tính trước hai mảng: `d(v, L)` (từ mọi nút tới `L`) và `d(L, v)` (từ `L` tới mọi nút), bằng hai Dijkstra, một trên CSR xuôi và một trên CSR ngược. Bất đẳng thức tam giác cho `d(u, L) ≤ d(u, t) + d(t, L)`, suy ra `d(u, t) ≥ d(u, L) − d(t, L)`; tương tự `d(u, t) ≥ d(L, t) − d(L, u)`. Hình 2 minh hoạ với số tự đặt: nếu `u` cách `L` 900 s còn `t` cách `L` 700 s thì đường từ `u` tới `t` không thể ngắn hơn 200 s.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 2. Bất đẳng thức tam giác với một landmark L: nếu d(u, L) = 900 s và d(t, L) = 700 s thì d(u, t) không thể nhỏ hơn 200 s, vì đi từ u qua t tới L không thể ngắn hơn d(u, L)."><rect x="20" y="20" width="840" height="210" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36" y="46" class="t-b">Chặn dưới bằng landmark (số tự đặt để minh hoạ công thức)</text><path d="M170 150 L800 150" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><circle cx="170" cy="150" r="14" class="f-phys-soft s-phys" stroke-width="2"/><text x="170" y="154" class="t-b" text-anchor="middle">u</text><circle cx="310" cy="150" r="14" class="f-pk-soft s-pk" stroke-width="2"/><text x="310" y="154" class="t-b" text-anchor="middle">t</text><circle cx="800" cy="150" r="14" class="f-warn-soft s-warn" stroke-width="2"/><text x="800" y="154" class="t-b" text-anchor="middle">L</text><path d="M188 184 L782 184" class="f-none s-warn" stroke-width="2" stroke-linecap="round"/><path d="M188 179 L188 189" class="f-none s-warn" stroke-width="2" stroke-linecap="round"/><path d="M782 179 L782 189" class="f-none s-warn" stroke-width="2" stroke-linecap="round"/><text x="485" y="203" class="t-mono" text-anchor="middle">d(u, L) = 900 s</text><path d="M328 208 L782 208" class="f-none s-warn" stroke-width="2" stroke-linecap="round"/><path d="M328 203 L328 213" class="f-none s-warn" stroke-width="2" stroke-linecap="round"/><path d="M782 203 L782 213" class="f-none s-warn" stroke-width="2" stroke-linecap="round"/><text x="555" y="227" class="t-mono" text-anchor="middle">d(t, L) = 700 s</text><path d="M188 100 L292 100" class="f-none s-phys" stroke-width="2" stroke-linecap="round"/><path d="M188 95 L188 105" class="f-none s-phys" stroke-width="2" stroke-linecap="round"/><path d="M292 95 L292 105" class="f-none s-phys" stroke-width="2" stroke-linecap="round"/><text x="240" y="119" class="t-mono" text-anchor="middle">chặn dưới d(u, t) ≥ 900 − 700 = 200 s</text><text x="188" y="76" class="t-small">đường thật từ u tới t có thể dài hơn, nhưng không thể ngắn hơn</text><text x="600" y="64" class="t-mono">d(u, L) ≤ d(u, t) + d(t, L)</text><text x="600" y="82" class="t-mono">⇒ d(u, t) ≥ d(u, L) − d(t, L)</text></svg>
</div>
<figcaption>Hình 2. Chặn dưới bằng một landmark L. Số 900 s và 700 s là số tự đặt để minh hoạ công thức, không phải số đo trên bản đồ. Với k landmark, lấy lớn nhất của các chặn dưới (cả hai vế) thì được một heuristic hợp lệ.</figcaption>
</figure>

Heuristic của ALT là lớn nhất của các chặn dưới trên mọi landmark và cả hai vế. Mỗi chặn dưới riêng lẻ là một heuristic nhất quán, mà lớn nhất của các heuristic nhất quán vẫn nhất quán, nên A* vẫn tối ưu và mỗi nút vẫn chốt một lần.

`examples/toi-noi-location/rust/crates/route-engine/src/alt.rs`

```rust
    /// Chặn dưới của chi phí `u → t`.
    #[inline]
    pub fn lower_bound(&self, u: NodeId, t: NodeId) -> u32 {
        let (bu, bt) = (u as usize * self.k * 2, t as usize * self.k * 2);
        let mut best = 0u32;
        for i in 0..self.k {
            let (u_to, u_from) = (self.data[bu + i * 2], self.data[bu + i * 2 + 1]);
            let (t_to, t_from) = (self.data[bt + i * 2], self.data[bt + i * 2 + 1]);
            // d(u,t) >= d(u,l) - d(t,l)
            if u_to != INF && t_to != INF && u_to > t_to {
                best = best.max(u_to - t_to);
            }
            // d(u,t) >= d(l,t) - d(l,u)
            if u_from != INF && t_from != INF && t_from > u_from {
                best = best.max(t_from - u_from);
            }
        }
        best
    }
```

Dữ liệu xếp xen kẽ theo nút: `data[(v * k + i) * 2]` là `d(v → Lᵢ)` và phần tử kế là `d(Lᵢ → v)`, nên mỗi nút giữ `2k` số `u32` liền nhau và một lần đọc cache đủ cho mọi landmark của nút đó. Gọi vào A* chỉ là truyền một closure, vì `Searcher::run` của bài 21 đã nhận heuristic bất kỳ:

`examples/toi-noi-location/rust/labs/src/bin/loc22_hai_chieu_alt.rs`

```rust
        for (i, (_, lm)) in alts.iter().enumerate() {
            let t0 = Instant::now();
            let (x, xs) = uni.run(view, a, b, |u| lm.lower_bound(u, b));
```

Chọn landmark thế nào cho tốt? Lab dùng chiến lược «xa nhất» (farthest): landmark đầu là nút xa nhất so với một nút hạt giống, các landmark sau là nút xa nhất so với tập đã chọn, tất cả trong thành phần liên thông mạnh lớn nhất của ô tô. Landmark ở rìa thành phố hay nằm «phía sau» một đầu của truy vấn: khi đường ngắn nhất từ `u` tới `L` đi qua `t`, chặn dưới `d(u, L) − d(t, L)` bằng đúng `d(u, t)`, còn lệch hướng bao nhiêu thì lỏng bấy nhiêu. Bài này không so các chiến lược chọn khác, và `k` cũng chưa tinh chỉnh ngoài ba giá trị 4, 8 và 16.

Tiền xử lý là `2k` Dijkstra toàn đồ thị, mỗi landmark chạy song song hai chiều bằng `rayon::join`, rồi `par_chunks_mut` xếp dữ liệu. Với `k = 4`, đó là tám Dijkstra, chưa kể một lần từ nút hạt giống.

## Số nút duyệt, bộ nhớ và tiền xử lý

Trên cùng 300 truy vấn ô tô, trọng số thời gian, lab chạy sáu thuật toán và so chi phí với Dijkstra: **sáu thuật toán cho 0 khác biệt**. Số nút đã chốt trung bình, nhóm trên 10 km chim bay (112 truy vấn):

| Thuật toán | Nút đã chốt | Ít hơn Dijkstra |
|---|---:|---:|
| Dijkstra | 86.656 | 1,00 |
| A* chim bay | 60.670 | 1,43 |
| hai chiều (điều kiện đúng) | 49.007 | 1,77 |
| ALT k=4 | 16.464 | 5,26 |
| ALT k=8 | 9.638 | 8,99 |
| ALT k=16 | 7.393 | 11,72 |

Tính trên mọi nhóm khoảng cách, lab báo A* ít hơn Dijkstra 1,61 lần, hai chiều 1,97 lần, ALT 6,39 lần (k=4), 10,72 lần (k=8) và 13,76 lần (k=16). Có hai điều bất ngờ nên nói rõ.

**Một: tìm hai chiều chỉ đáng khoảng 2 lần.** 1,97 lần gần đúng bằng phép tính diện tích ở đầu bài, không hơn. Theo nhóm khoảng cách, tỉ lệ này giảm dần: 2,97 lần (dưới 2 km), 2,61, 2,25, rồi 1,77 lần (trên 10 km), vì truy vấn dài thì vòng của Dijkstra đã chạm biên thành phố (86.656 trên 111.441 nút) nên không còn nhiều để tiết kiệm. Phần lớn lợi ích tới từ landmark, không phải từ việc chạy từ hai đầu. Lab không kết hợp hai kỹ thuật (ALT hai chiều); bài này không đo thứ chưa chạy.

**Hai: landmark không miễn phí.** Mỗi nút giữ `8k` byte: 32 byte/nút ở k=4, 128 byte/nút ở k=16. Với 126.699 nút, bộ nhớ là 3,9 MB, 7,7 MB và 15,5 MB, so với đồ thị gốc 2,9 MB: k=16 tốn gấp khoảng 5 lần chính đồ thị. Tiền xử lý mất 0,04 s (k=4), 0,08 s (k=8) và 0,12–0,14 s (k=16) trên 10 lõi, giá trị nhỏ nhất đến lớn nhất qua ba lần chạy, đo khi máy chịu tải.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Hình 3. Trên 300 truy vấn ô tô HCM, nhóm trên 10 km: số nút đã chốt của Dijkstra 86.656, A* chim bay 60.670, hai chiều 49.007, ALT k=4 16.464, k=8 9.638, k=16 7.393. Bộ nhớ landmark: 3,9; 7,7; 15,5 MB so với đồ thị 2,9 MB."><rect x="20" y="20" width="840" height="360" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36" y="46" class="t-b">Nhóm trên 10 km chim bay (112 truy vấn): nút đã chốt</text><text x="36" y="78" class="t-small">Dijkstra</text><rect x="116" y="64" width="317.739" height="20" rx="3" class="f-dead s-dead" stroke-width="1.5"/><text x="441.739" y="78" class="t-mono">86.656</text><text x="36" y="110" class="t-small">A* chim bay</text><rect x="116" y="96" width="222.457" height="20" rx="3" class="f-dead s-dead" stroke-width="1.5"/><text x="346.457" y="110" class="t-mono">60.670</text><text x="36" y="142" class="t-small">hai chiều</text><rect x="116" y="128" width="179.692" height="20" rx="3" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="303.692" y="142" class="t-mono">49.007</text><text x="36" y="174" class="t-small">ALT k=4</text><rect x="116" y="160" width="60.368" height="20" rx="3" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="184.368" y="174" class="t-mono">16.464</text><text x="36" y="206" class="t-small">ALT k=8</text><rect x="116" y="192" width="35.3393" height="20" rx="3" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="159.339" y="206" class="t-mono">9.638</text><text x="36" y="238" class="t-small">ALT k=16</text><rect x="116" y="224" width="27.1077" height="20" rx="3" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="151.108" y="238" class="t-mono">7.393</text><text x="36" y="274" class="t-small">Tỉ lệ ít hơn Dijkstra (mọi nhóm): A* 1,61x · hai chiều 1,97x · ALT 6,39x, 10,72x, 13,76x</text><text x="36" y="292" class="t-small">Cả sáu thuật toán cho chi phí đúng bằng Dijkstra ở 300/300 truy vấn.</text><text x="36" y="310" class="t-small">Nút đã chốt là số đếm, tái lập từng chữ số với seed 20261106.</text><text x="560" y="46" class="t-b">Landmark trả giá bằng bộ nhớ (MB)</text><text x="560" y="78" class="t-small">đồ thị gốc</text><rect x="630" y="64" width="20.3" height="20" rx="3" class="f-dead s-dead" stroke-width="1.5"/><text x="656.3" y="78" class="t-mono">2,9</text><text x="560" y="110" class="t-small">k = 4</text><rect x="630" y="96" width="27.3" height="20" rx="3" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="663.3" y="110" class="t-mono">3,9 · 0,04 s</text><text x="560" y="142" class="t-small">k = 8</text><rect x="630" y="128" width="53.9" height="20" rx="3" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="689.9" y="142" class="t-mono">7,7 · 0,08 s</text><text x="560" y="174" class="t-small">k = 16</text><rect x="630" y="160" width="108.5" height="20" rx="3" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="744.5" y="174" class="t-mono">15,5 · 0,14 s</text><text x="560" y="206" class="t-small">Số sau dấu · là thời gian tiền xử lý</text><text x="560" y="224" class="t-small">(10 lõi, máy chịu tải).</text><text x="36" y="362" class="t-small">© OpenStreetMap contributors (ODbL). Nguồn: results/p03.md, LOC-22 mục 3 và 4.</text></svg>
</div>
<figcaption>Hình 3. Trái: số nút đã chốt của sáu thuật toán trong nhóm trên 10 km (số đếm, không phụ thuộc tải máy). Phải: bộ nhớ landmark và thời gian tiền xử lý so với đồ thị gốc. Đồ thị HCM 126.699 nút, ô tô, trọng số thời gian, seed 20261106. © OpenStreetMap contributors (ODbL).</figcaption>
</figure>

Nút đã chốt ít hơn không tự động là nhanh hơn: mỗi lần nới được nhãn, ALT còn phải gọi `lower_bound`, đọc `4k` số của `u` và `t`. Trong nhóm trên 10 km, ALT k=16 duyệt ít hơn Dijkstra 11,72 lần nhưng thời gian trung bình chỉ giảm khoảng 5 lần (5.348 µs xuống 1.064 µs, cùng một lần chạy, khi máy tải 1 phút 24,26). Số µs này nhiễu và chỉ đáng tin ở cỡ tỉ lệ.

Còn một giới hạn quan trọng của ALT: landmark học từ **trọng số lúc tính**. Khi tốc độ theo giờ đổi (kẹt xe, bài 27), điều gì xảy ra tuỳ chiều thay đổi. Nếu mọi trọng số chỉ tăng so với lúc tính (học ở giờ thông thoáng, dùng ở cao điểm), chặn dưới vẫn hợp lệ, vì nó không vượt chi phí cũ mà chi phí cũ không vượt chi phí mới, nhưng lỏng dần. Nếu trọng số giảm (học ở cao điểm, dùng ban đêm), chặn dưới có thể vượt chi phí thật và A* trả đường sai mà không báo lỗi. Đây là cái giá chung của mọi thuật toán có tiền xử lý; bài 25 và bài 27 quay lại câu hỏi dựng lại hay chỉnh trọng số.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` (hoặc thêm `-- --fixture`). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc22_hai_chieu_alt
```

Cấu hình đo: `--release`, Apple M1 Max, 10 lõi, đồ thị HCM 126.699 nút, ô tô, trọng số thời gian, 300 truy vấn, seed 20261106, landmark chọn «xa nhất». Output (rút gọn) của lần chạy có Dijkstra nhanh nhất trong ba lần, **đo khi máy đang chịu tải nặng** (tải 1 phút 24,26):

```text
-- 1. lỗi dừng sớm: ví dụ nhỏ dựng tay (s=0, a=1, m=2, b=3, t=4) --
  cạnh: s→a 1, a→b 10, b→t 1, s→m 6, m→t 7
  Dijkstra một chiều        : chi phí 12
  hai chiều, dừng đúng      : chi phí 12
  hai chiều, dừng ở nút gặp đầu tiên: chi phí 13 (qua nút m, đi [0, 2, 4])

-- 2. cùng lỗi đó trên bản đồ thật: 300 truy vấn ngẫu nhiên --
  dừng đúng khác Dijkstra: 0/300
  dừng ở nút gặp đầu tiên cho chi phí sai: 29/300 truy vấn (9.7%)
  đường sai dài hơn đường đúng: trung bình 0.71%, trung vị 0.20%, lớn nhất 5.69%
  ca tệ nhất: nút 97043 → nút 2643, chim bay 335 m: đúng 78.7 s, sai 83.1 s

-- 3. tiền xử lý ALT --
  k = 4 : tiền xử lý 0.04 s (2×4 lần Dijkstra song song), bộ nhớ 3.9 MB (32 byte/nút), đồ thị gốc 2.9 MB
  k = 8 : tiền xử lý 0.08 s (2×8 lần Dijkstra song song), bộ nhớ 7.7 MB (64 byte/nút), đồ thị gốc 2.9 MB
  k = 16: tiền xử lý 0.14 s (2×16 lần Dijkstra song song), bộ nhớ 15.5 MB (128 byte/nút), đồ thị gốc 2.9 MB

-- 4. số nút đã duyệt và thời gian trung bình theo nhóm khoảng cách --
  nhóm             Dijkstra  A* chim bay    hai chiều      ALT k=4      ALT k=8     ALT k=16
  > 10 km nút         86656        60670        49007        16464         9638         7393   (n = 112)
     µs                5348         4759         4582         1592         1111         1064
  khác Dijkstra            0            0            0            0            0            0
  tỉ lệ nút duyệt so với Dijkstra (mọi nhóm): A* 1.61x, hai chiều 1.97x, ALT k=4 6.39x, ALT k=8 10.72x, ALT k=16 13.76x ít hơn
```

Số nút đã duyệt, số truy vấn sai và các tỉ lệ tái lập từng chữ số với cùng seed; thời gian tiền xử lý và cột µs thì không. Kiểm tự động: `crates/route-engine/tests/proptest_vs_dijkstra.rs` chạy A*, ALT (k=8), hai chiều với điều kiện dừng đúng và CH, so chi phí với Dijkstra trên 400 trường hợp ngẫu nhiên, ở cả ba profile trên đồ thị fixture.

## Giới hạn của ví dụ

- **Chỉ có đồ thị theo nút, chưa có chi phí rẽ.** Hai chiều và ALT không biết quay đầu hay cấm rẽ (bài 23).
- **Landmark học từ trọng số tĩnh.** Lab chưa chạy ALT với hồ sơ tốc độ theo giờ, nên chưa có số về việc chặn dưới lỏng đi bao nhiêu.
- **Một chiến lược chọn landmark, ba giá trị k.** Chưa so «xa nhất» với ngẫu nhiên hay tránh trùng lặp, chưa chọn k theo ngân sách bộ nhớ.
- **Chưa kết hợp hai chiều với ALT**, nên không có số cho tổ hợp đó.
- **Đo thời gian khi máy tải nặng.** Chỉ tỉ lệ số nút đáng tin; các cột µs nhiễu, chưa có lần đo trên máy rảnh.
- **Bộ nhớ ALT tăng tuyến tính theo k và số nút.** Với thành phố lớn hơn, 128 byte/nút có thể không chấp nhận được.

## Bài tập

**Bài 1.** Đổi trọng số cạnh `m → t` của đồ thị năm nút từ 7 thành 5. Đường nào ngắn nhất? Bản dừng sai có còn trả sai không? Vì sao?

<details><summary>Lời giải bài 1</summary>
<p>Đường s→m→t dài 6 + 5 = 11, ngắn hơn đường 12. Bản dừng sai trả 11, tức đúng: nút gặp đầu tiên (m, chốt ở khoảng cách 6 từ xuôi và 5 từ ngược) lần này nằm ngay trên đường tối ưu. Lỗi chỉ xảy ra khi nút gặp đầu tiên nằm ngoài đường tối ưu, nên một test chỉ có vài trường hợp nhỏ rất dễ bỏ sót nó; đó là lý do lab so với Dijkstra trên hàng trăm truy vấn ngẫu nhiên.</p>
</details>

**Bài 2.** Vì sao `mu` phải được cập nhật lúc nới cạnh chạm nút có nhãn của chiều kia, mà không phải lúc chốt nút? Chỉ ra cạnh nào ở Hình 1 sẽ bị bỏ sót nếu chỉ cập nhật lúc chốt.

<details><summary>Lời giải bài 2</summary>
<p>Đường tối ưu 12 đi qua cạnh a→b. Nút a chỉ được chiều xuôi chốt, nút b chỉ được chiều ngược chốt, không nút nào được cả hai chốt trước khi điều kiện dừng đúng thoả. Nếu μ chỉ cập nhật khi một nút được chốt ở cả hai chiều, đường 12 không bao giờ được ghi nhận; μ sẽ kẹt ở 13 (qua m). Nới cạnh a→b lúc chốt a thấy b đã có nhãn ngược, và μ được hạ xuống 12 ngay.</p>
</details>

**Bài 3.** Tính bộ nhớ landmark nếu dùng k = 32 trên đồ thị 126.699 nút (mỗi giá trị là `u32`, hai chiều cho mỗi landmark). Gấp bao nhiêu lần đồ thị gốc 2,9 MB?

<details><summary>Lời giải bài 3</summary>
<p>Mỗi nút giữ <code>2 × 32</code> số <code>u32</code> = 256 byte. <code>126.699 × 256 = 32.434.944</code> byte, khoảng 30,9 MB, tức khoảng 10,7 lần đồ thị gốc. Kiểm lại với k = 16: 128 byte × 126.699 = 15,5 MB, khớp với lab. Tuyến tính theo k, nên k = 32 đã vượt xa mức đáng chấp nhận nếu chỉ để giảm tiếp số nút từ 13,76 lần.</p>
</details>

Bài trước: [bài 21](/articles/location-dijkstra-a-star-rust-do-thi-csr). Bài tiếp: [bài 23](/articles/location-chi-phi-re-u-turn-do-thi-theo-canh).

## Tài liệu gốc

- A. V. Goldberg, C. Harrelson, «Computing the shortest path: A* search meets graph theory», ACM-SIAM SODA 2005, trang 156–165: [bản tại Microsoft Research](https://www.microsoft.com/en-us/research/publication/computing-the-shortest-path-a-search-meets-graph-theory). Bài báo gốc của ALT.
- P. E. Hart, N. J. Nilsson, B. Raphael, «A formal basis for the heuristic determination of minimum cost paths», IEEE Transactions on Systems Science and Cybernetics 4(2), 1968: [doi.org/10.1109/TSSC.1968.300136](https://doi.org/10.1109/TSSC.1968.300136). Điều kiện hợp lệ và nhất quán của heuristic.
- E. W. Dijkstra, «A note on two problems in connexion with graphs», Numerische Mathematik 1, 1959: [doi.org/10.1007/BF01386390](https://doi.org/10.1007/BF01386390).
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho hình có dữ liệu OSM.
