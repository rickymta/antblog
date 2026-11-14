---
slug: location-contraction-hierarchies-tu-cai-dat-fast-paths
title: "Contraction Hierarchies: tự cài đặt rồi đối chiếu fast_paths"
summary: "Bài 25: tự cài Contraction Hierarchies bằng Rust (thứ tự co, shortcut, truy vấn hai chiều đi lên), đối chiếu fast_paths và Dijkstra trên đồ thị HCM, và vì sao CH theo nút không mang chi phí rẽ."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, routing, contraction-hierarchies, fast-paths, openstreetmap]
language: vi
---

Ghép chuyến cần biết chi phí đường từ **mỗi** tài xế rảnh tới **mỗi** khách đang chờ. Với 100 khách và 100 tài xế, đó là 10.000 cặp. Trung vị một truy vấn Dijkstra trên đồ thị HCM của lab là 3.355,9 µs; 10.000 truy vấn là khoảng 34 giây. Cùng 10.000 cặp, bằng Contraction Hierarchies (CH) tự cài của lab với trung vị 6,2 µs mỗi cặp, là khoảng 0,06 giây. Hai phép nhân đơn giản từ số đo, và cả hai số đo lấy khi máy đang chịu tải rất nặng, nên chỉ cho biết cỡ: cách nhau gần ba bậc độ lớn, và đó là lý do ghép chuyến ở phần P05 dựa trên CH.

Đây là bài 25 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài tiếp nối [bài 22](/articles/location-bidirectional-alt-landmark-tang-toc-truy-van) (tìm hai chiều và điều kiện dừng đúng) và dùng đồ thị theo nút của [bài 21](/articles/location-dijkstra-a-star-rust-do-thi-csr). Ý tưởng là của Geisberger, Sanders, Schultes và Delling trong «Contraction Hierarchies: Faster and Simpler Hierarchical Routing in Road Networks» (WEA 2008): đánh số nút theo độ quan trọng tăng dần rồi dựng phân cấp bằng cách co lần lượt từng nút. Bài này cài lại **theo ý tưởng đó**, với hàm ưu tiên đơn giản hơn bản gốc, để học; engine của lab không được tuyên bố thay thế OSRM hay Valhalla (bài 28 bàn chuyện đó).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Dựng CH cho cả đồ thị HCM mất khoảng một giây (một luồng). Mỗi truy vấn sau đó nhanh hơn Dijkstra khoảng ba nghìn µs. Bạn đoán cần bao nhiêu truy vấn thì thời gian dựng được «hoàn vốn»?</p>
<details><summary>Xem đáp án</summary><p>Khoảng <b>336 truy vấn</b> (320 đến 369 qua ba lần chạy): 1,13 s chia cho 3.361 µs tiết kiệm mỗi truy vấn. Một lần dựng đủ cho cả một ngày truy vấn, nên CH gần như luôn đáng, miễn là bản đồ và trọng số không đổi liên tục. Với bản đồ đổi mỗi phút (kẹt xe), câu trả lời khác, và bài 27 bàn tới.</p></details>
</div>

## Ý tưởng: đường tắt đã tính sẵn, nút quan trọng ở trên cùng

Hãy nghĩ tới việc đi từ một hẻm này sang một hẻm khác cách đó 15 km. Ta không cần xét mọi ngõ nhỏ trên đường: ta đi từ hẻm ra đường nhỏ, ra đường lớn, lên trục chính, rồi ngược lại xuống. Đồ thị có sẵn **thứ bậc** như vậy; CH khai thác nó. Mỗi nút được gán một hạng: hẻm cụt có hạng thấp, ngã tư trục chính có hạng cao. Rồi nút được **co** (contract) từ hạng thấp lên cao: nhấc nút `v` ra khỏi đồ thị, và nếu đường ngắn nhất giữa hai láng giềng `u`, `x` của nó từng đi qua `v`, thêm một cạnh tắt (shortcut) `u → x` với trọng số `w(u,v) + w(v,x)` để không mất đường đó.

Điều kiện «từng đi qua `v`» được kiểm bằng **tìm kiếm nhân chứng** (witness search): chạy một Dijkstra nhỏ từ `u`, bỏ qua `v`. Nếu tìm được đường tới `x` không dài hơn `w(u,v) + w(v,x)`, nhân chứng chứng tỏ shortcut thừa. Ngược lại phải thêm.

Hình 1 chạy tay trên một đồ thị bảy cạnh dựng tay (không phải dữ liệu OSM): a và b là hai hẻm, c là ngã ba, d và e là hai đường nhỏ, f là điểm đón. Thứ tự co là a, d, e, b, f, c, do chính `Ch::build` của lab cho ra trên đồ thị này (test `article_toy_graph_gives_the_order_and_shortcut_in_the_text` trong `ch.rs` kiểm thứ tự, shortcut và truy vấn ở Hình 2). Co a và co d không tạo shortcut vì có nhân chứng; co e tạo đúng một shortcut hai chiều c–f = 7, vì không có đường nào khác từ c tới f dài không quá 7 (đường qua d dài 8 và d cũng đã co).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 392" role="img" aria-label="Hình 1. Contraction Hierarchies trên đồ thị bảy cạnh dựng tay, co theo thứ tự a, d, e, b, f, c. Co a và d không cần shortcut vì có nhân chứng (b–c = 5 ≤ 6; c–e–f = 7 ≤ 8). Co e cần shortcut c–f = 7 vì không có nhân chứng."><rect x="20" y="20" width="420" height="352" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36" y="46" class="t-b">Đồ thị (cạnh hai chiều, trọng số là giây) và hạng</text><path d="M80 125 L195 205" class="f-none s-dead" stroke-width="3" stroke-linecap="round"/><path d="M80 285 L195 205" class="f-none s-dead" stroke-width="3" stroke-linecap="round"/><path d="M195 205 L310 125" class="f-none s-dead" stroke-width="3" stroke-linecap="round"/><path d="M195 205 L310 285" class="f-none s-dead" stroke-width="3" stroke-linecap="round"/><path d="M310 125 L405 205" class="f-none s-dead" stroke-width="3" stroke-linecap="round"/><path d="M310 285 L405 205" class="f-none s-dead" stroke-width="3" stroke-linecap="round"/><path d="M80 125 L80 285" class="f-none s-dead" stroke-width="3" stroke-linecap="round"/><path d="M195 205 L405 205" class="f-none s-warn dash" stroke-width="2.5" stroke-linecap="round"/><text x="140" y="150" class="t-mono" text-anchor="middle">2</text><text x="126" y="278" class="t-mono" text-anchor="middle">5</text><text x="236" y="150" class="t-mono" text-anchor="middle">5</text><text x="236" y="262" class="t-mono" text-anchor="middle">2</text><text x="375" y="148" class="t-mono" text-anchor="middle">3</text><text x="375" y="262" class="t-mono" text-anchor="middle">5</text><text x="62" y="206" class="t-mono" text-anchor="middle">4</text><text x="300" y="196" class="t-small" text-anchor="middle">7, nút giữa e</text><circle cx="80" cy="125" r="15" class="f-phys-soft s-phys" stroke-width="2"/><text x="80" y="129" class="t-b" text-anchor="middle">a</text><text x="80" y="101" class="t-small" text-anchor="middle">hạng 1</text><circle cx="80" cy="285" r="15" class="f-phys-soft s-phys" stroke-width="2"/><text x="80" y="289" class="t-b" text-anchor="middle">b</text><text x="80" y="317" class="t-small" text-anchor="middle">hạng 4</text><circle cx="195" cy="205" r="15" class="f-phys-soft s-phys" stroke-width="2"/><text x="195" y="209" class="t-b" text-anchor="middle">c</text><text x="195" y="181" class="t-small" text-anchor="middle">hạng 6</text><circle cx="310" cy="125" r="15" class="f-phys-soft s-phys" stroke-width="2"/><text x="310" y="129" class="t-b" text-anchor="middle">d</text><text x="310" y="101" class="t-small" text-anchor="middle">hạng 2</text><circle cx="310" cy="285" r="15" class="f-warn-soft s-warn" stroke-width="2"/><text x="310" y="289" class="t-b" text-anchor="middle">e</text><text x="310" y="317" class="t-small" text-anchor="middle">hạng 3</text><circle cx="405" cy="205" r="15" class="f-phys-soft s-phys" stroke-width="2"/><text x="405" y="209" class="t-b" text-anchor="middle">f</text><text x="405" y="237" class="t-small" text-anchor="middle">hạng 5</text><text x="36" y="360" class="t-small">Nét cam đứt: shortcut c–f thêm khi co e.</text><rect x="460" y="20" width="400" height="352" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="476" y="46" class="t-b">Từng bước co</text><rect x="476" y="64" width="368" height="80" rx="6" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="488" y="84" class="t-b">Bước 1: co a (hạng 1)</text><text x="488" y="103" class="t-small">b–c qua a = 4 + 2 = 6;</text><text x="488" y="118" class="t-small">nhân chứng b–c trực tiếp = 5 ≤ 6</text><text x="488" y="133" class="t-small">→ không thêm shortcut</text><rect x="476" y="152" width="368" height="80" rx="6" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="488" y="172" class="t-b">Bước 2: co d (hạng 2)</text><text x="488" y="191" class="t-small">c–f qua d = 5 + 3 = 8;</text><text x="488" y="206" class="t-small">nhân chứng c–e–f = 2 + 5 = 7 ≤ 8</text><text x="488" y="221" class="t-small">→ không thêm shortcut</text><rect x="476" y="240" width="368" height="80" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="488" y="260" class="t-b">Bước 3: co e (hạng 3)</text><text x="488" y="279" class="t-small">c–f qua e = 2 + 5 = 7; d đã co,</text><text x="488" y="294" class="t-small">không có nhân chứng dài ≤ 7</text><text x="488" y="309" class="t-small">→ THÊM shortcut c–f = 7 (nút giữa e)</text><text x="476" y="342" class="t-small">Bước 4–6: co b, f, c: mỗi nút còn nhiều nhất một láng giềng</text><text x="476" y="358" class="t-small">nên không có cặp nào để xét. Thêm 1 shortcut cho 7 cạnh.</text></svg>
</div>
<figcaption>Hình 1. Co nút trên đồ thị bảy cạnh dựng tay (cạnh hai chiều, trọng số giây). Trái: đồ thị, hạng của từng nút, shortcut c–f nét cam đứt. Phải: ba bước đầu. Bước 1 và 2 có nhân chứng (b–c = 5 ≤ 6; c–e–f = 7 ≤ 8), bước 3 không có nhân chứng nên thêm shortcut. Đồ thị dựng tay; thứ tự và shortcut là output của <code>Ch::build</code>, kiểm bằng test trong <code>ch.rs</code>.</figcaption>
</figure>

Kết quả của cả quá trình là hai đồ thị «chỉ đi lên»: với mỗi nút, các cung tới láng giềng có hạng **cao hơn** (cạnh gốc hoặc shortcut). `UpGraph` lưu chúng ở dạng CSR, kèm nút giữa của shortcut để giải nén sau.

`examples/toi-noi-location/rust/crates/route-engine/src/ch.rs`

```rust
pub struct UpGraph {
    pub first: Vec<u32>,
    /// Đầu kia của cung (hạng cao hơn).
    pub head: Vec<u32>,
    pub w: Vec<u32>,
    /// Nút bị co tạo ra shortcut (hạng), hoặc [`NONE`] nếu là cạnh gốc.
    pub mid: Vec<u32>,
    /// Cạnh gốc của đồ thị đường nếu `mid == NONE`.
    pub edge: Vec<u32>,
}
```

Mọi chỉ số nút bên trong `Ch` là **hạng**, không phải chỉ số nút gốc; `new_id` đổi nút gốc sang hạng và `old_id` đổi ngược lại. Chọn hạng làm chỉ số cho phép «cung đi lên» chỉ là so sánh hai số nguyên.

## Co một nút: tìm kiếm nhân chứng

Co `v` nghĩa là xét mọi cặp (`u` có cạnh vào `v`, `x` có cạnh ra `v`, `u ≠ x`). Với mỗi `u`, lab chạy **một** Dijkstra nhân chứng từ `u`, bỏ qua `v`, giới hạn chi phí bằng `w(u,v) + w(v, x)` lớn nhất, rồi so với từng `x`.

`examples/toi-noi-location/rust/crates/route-engine/src/ch.rs`

```rust
    /// Dijkstra bỏ qua `skip`, dừng khi vượt `max_cost` hoặc đã chốt `limit` nút.
    fn witness(&mut self, u: u32, skip: u32, max_cost: u32, limit: u32) {
        self.ws.reset();
        self.heap.clear();
        self.ws.set(u, 0, NONE);
        self.heap.push(0, u);
        let mut settled = 0;
        while let Some((d, x)) = self.heap.pop() {
            if self.ws.is_closed(x) {
                continue;
            }
            self.ws.close(x);
            settled += 1;
            if d > max_cost || settled > limit {
                break;
            }
            for a in &self.out[x as usize] {
                if a.to == skip {
                    continue;
                }
                let nd = d.saturating_add(a.w);
                if nd <= max_cost && nd < self.ws.dist(a.to) {
                    self.ws.set(a.to, nd, x);
                    self.heap.push(nd, a.to);
                }
            }
        }
    }

    /// Các shortcut cần thêm khi co `v`: `(u, x, trọng số)`.
    fn shortcuts_for(&mut self, v: u32, limit: u32) -> Vec<(u32, u32, u32)> {
        let mut res = Vec::new();
        let max_out = self.out[v as usize].iter().map(|a| a.w).max().unwrap_or(0);
        let ins: Vec<Arc> = self.inn[v as usize].clone();
        for ain in &ins {
            let u = ain.to;
            if self.out[v as usize].iter().all(|a| a.to == u) {
                continue;
            }
            let max_cost = ain.w.saturating_add(max_out);
            self.witness(u, v, max_cost, limit);
            for aout in &self.out[v as usize] {
                let x = aout.to;
                if x == u {
                    continue;
                }
                let s = ain.w.saturating_add(aout.w);
                if self.ws.dist(x) > s {
                    res.push((u, x, s));
                }
            }
        }
        res
    }
```

Điểm cốt lõi là `self.ws.dist(x) > s`: chỉ khi nhân chứng **không** tìm được đường tới `x` với chi phí không quá `s` mới thêm shortcut `(u, x, s)`. Có hai chỗ cần để ý. Một: `limit` chặn số nút nhân chứng được chốt (lab dùng 500 khi co thật, 60 khi chỉ mô phỏng để tính độ ưu tiên). Nếu nhân chứng bị cắt sớm, ta có thể **thêm shortcut thừa** nhưng không bao giờ thiếu shortcut: thiếu mới là sai, thừa chỉ tốn bộ nhớ. Hai: `witness` dùng `Labels` có phiên bản của bài 21 nên chạy rất nhiều lần không phải khởi tạo lại mảng.

## Thứ tự co: độ ưu tiên và cập nhật lười

Chất lượng CH phụ thuộc gần như hoàn toàn vào **thứ tự co**. Co một nút «xấu» (nhiều láng giềng, mọi cặp đều cần shortcut) sớm sẽ đẻ ra rất nhiều shortcut. Lab đo mỗi nút bằng một con số, càng nhỏ càng co sớm:

`examples/toi-noi-location/rust/crates/route-engine/src/ch.rs`

```rust
        let priority = |d: &mut Dyn, v: u32, dn: i32, lvl: u32| -> i32 {
            let sc = d.shortcuts_for(v, SIM_LIMIT).len() as i32;
            let removed = (d.out[v as usize].len() + d.inn[v as usize].len()) as i32;
            2 * (sc - removed) + dn + lvl as i32
        };
```

`sc − removed` là «chênh lệch cạnh» (edge difference): số shortcut sẽ thêm trừ số cung bị bỏ khi co `v`. Co nút có chênh lệch âm làm đồ thị **nhỏ đi**. `dn` là số láng giềng đã bị co (nhằm trải đều việc co, tránh dồn vào một vùng) và `lvl` là độ sâu phân cấp (nút có láng giềng đã co ở mức cao thì nên co muộn). Hệ số `2` ưu tiên chênh lệch cạnh hơn hai số kia. Đây là hàm **đơn giản** của lab, không phải hàm ưu tiên đầy đủ của bài báo.

Áp lên đồ thị bảy cạnh ở Hình 1, tính tay theo công thức trên, độ ưu tiên ban đầu của a, b, c, d, e, f là −8, −8, 4, −8, −4, −8. Nút c (ngã ba nối nhiều đường) có độ ưu tiên cao nhất nên co cuối cùng; các nút có độ ưu tiên −8 (hai láng giềng, co không cần shortcut) co trước, rồi tới e (−4), nút duy nhất cần shortcut. Hạng cuối cùng khớp cách vẽ ở Hình 1: c cao nhất.

Vì co một nút làm đổi độ ưu tiên của nút khác, lab làm hai việc: tính lại ngay độ ưu tiên của các láng giềng vừa bị ảnh hưởng, và dùng **cập nhật lười** cho phần còn lại: khi lấy nút ra, tính lại độ ưu tiên; nếu tệ hơn đỉnh hàng đợi thì đẩy lại, chưa co.

`examples/toi-noi-location/rust/crates/route-engine/src/ch.rs`

```rust
            // cập nhật lười: tính lại, nếu tệ hơn đỉnh hàng đợi thì đẩy lại
            let np = priority(&mut d, v, deleted_nb[vi], level[vi]);
            if np > p
                && let Some(&Reverse((top, _))) = heap.peek()
                && np > top
            {
                prio[vi] = np;
                heap.push(Reverse((np, v)));
                continue;
            }
```

Trên đồ thị HCM, quá trình này mất **1,13 s** (một luồng; 1,13 đến 1,31 s qua ba lần chạy, đo khi máy chịu tải nặng). Đồ thị có 250.580 cạnh gốc đi được (sau khi gộp cạnh song song) và CH thêm **181.535 shortcut**, bằng 0,72 lần số cạnh gốc, với độ sâu phân cấp lớn nhất là 72. Hai đồ thị đi lên có 215.975 cung thuận và 216.140 cung ngược, tổng 432.115 (3,41 cung mỗi nút), 8,53 MB.

## Truy vấn: hai chiều, chỉ đi lên

Đây là chỗ CH có lợi. Với mỗi cặp nút, đồ thị gốc cộng shortcut luôn có **một** đường ngắn nhất dạng «đi lên theo hạng tới một nút cao nhất, rồi đi xuống».

Vì sao có đường như vậy? Lấy một đường ngắn nhất bất kỳ, và giả sử nó có một «đáy» `u → v → x`, tức `v` có hạng thấp hơn cả `u` và `x`. Lúc co `v`, cả `u` và `x` còn trong đồ thị, nên chỉ có hai khả năng: lab thêm shortcut `u → x` với trọng số `w(u,v) + w(v,x)`, hoặc nhân chứng đã tìm được một đường từ `u` tới `x` không dài hơn, đi qua toàn nút chưa co, tức hạng cao hơn `v`. Thay `u → v → x` bằng đường đó thì độ dài không tăng và đáy biến mất. Mỗi lần thay, các nút trên đường chỉ có thể thay bằng nút hạng cao hơn, nên quá trình phải dừng, và đường lúc dừng không còn đáy. Lý lẽ này chỉ cần «không bao giờ thiếu shortcut», nên thừa shortcut do nhân chứng bị cắt sớm không làm hỏng nó.

Vì vậy truy vấn chạy hai Dijkstra: chiều **xuôi** từ nguồn trên `fwd`, chiều **ngược** từ đích trên `bwd`, cả hai chỉ đi lên. Chúng gặp nhau ở nút hạng cao nhất của đường đó. Không gian đi lên rất nhỏ so với cả đồ thị: một truy vấn chỉ chốt trung bình 172 nút tính cả hai chiều (con số đo ở phần dưới).

`examples/toi-noi-location/rust/crates/route-engine/src/ch.rs`

```rust
        loop {
            let kf = top(&mut ws.hf, &ws.f).filter(|&k| k < mu);
            let kb = top(&mut ws.hb, &ws.b).filter(|&k| k < mu);
            let forward = match (kf, kb) {
                (None, None) => break,
                (Some(_), None) => true,
                (None, Some(_)) => false,
                (Some(a), Some(b)) => a <= b,
            };
```

Chú ý điều kiện dừng. Bài 22 dừng khi `top_f + top_b ≥ μ`. Ở CH, mỗi chiều chạy trên **một đồ thị đi lên riêng** và nút gặp nhau là nút cao nhất của đường, nên chiều xuôi không đi qua phần «đi xuống» của đường. Tổng hai khoá vì thế không còn chặn dưới được đường chưa thấy: nút gặp có thể đã được chiều xuôi chốt từ lâu với khoảng cách nhỏ, trong khi chiều ngược chưa tới. Lab dùng điều kiện đơn giản hơn và đúng: mỗi chiều chỉ tiếp tục chừng nào khoá nhỏ nhất của nó còn nhỏ hơn μ (`filter(|&k| k < mu)`); khoá đã đạt μ không thể cho đường ngắn hơn vì trọng số không âm. Khi một nút được chốt, nếu chiều kia đã có nhãn của nó thì cập nhật μ và nút gặp.

Hình 2 chạy truy vấn a tới f trên đồ thị đi lên của ví dụ ở Hình 1. Chiều xuôi từ a thấy a→c (2) và a→b (4); chiều ngược từ f thấy f→c (7), cung này chính là shortcut c–f. Khi chiều xuôi chốt c với khoảng cách 2, chiều ngược đã có nhãn c bằng 7, nên μ = 9. Hai chiều còn xử lý b và c phía ngược, không cải thiện, và dừng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 372" role="img" aria-label="Hình 2. Truy vấn a tới f trên đồ thị đi lên: chiều xuôi chỉ đi lên từ a, chiều ngược chỉ đi lên từ f, gặp nhau ở nút hạng cao nhất c với tổng 2 + 7 = 9. Giải nén shortcut c–f (nút giữa e) cho đường a, c, e, f dài 9."><rect x="20" y="20" width="420" height="332" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36" y="46" class="t-b">Đồ thị đi lên (cung từ hạng thấp lên hạng cao)</text><path d="M100 294 L100.0 175.0" class="f-none s-phys" stroke-width="3" stroke-linecap="round"/><path d="M100 166 L104.5 175.0 L95.5 175.0 z" class="f-phys"/><path d="M108.426 296.398 L226.8 105.3" class="f-none s-phys" stroke-width="3" stroke-linecap="round"/><path d="M231.574 97.6017 L230.7 107.6 L223.0 102.9 z" class="f-phys"/><path d="M114.472 143.177 L217.4 94.7" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M225.528 90.8227 L219.3 98.7 L215.5 90.6 z" class="f-dead"/><path d="M240 220 L240.0 109.0" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M240 100 L244.5 109.0 L235.5 109.0 z" class="f-dead"/><path d="M253.633 227.625 L358.7 163.1" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M366.367 158.375 L361.1 166.9 L356.3 159.3 z" class="f-dead"/><path d="M369.16 224.231 L256.9 102.4" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M250.84 95.7687 L260.2 99.3 L253.6 105.4 z" class="f-dead"/><path d="M380 220 L380.0 175.0" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M380 166 L384.5 175.0 L375.5 175.0 z" class="f-dead"/><path d="M365.528 143.177 L262.6 94.7" class="f-none s-warn" stroke-width="3" stroke-linecap="round"/><path d="M254.472 90.8227 L264.5 90.6 L260.7 98.7 z" class="f-warn"/><circle cx="240" cy="84" r="15" class="f-pk-soft s-pk" stroke-width="2"/><text x="240" y="88" class="t-b" text-anchor="middle">c</text><text x="218" y="88" class="t-small" text-anchor="end">hạng 6</text><circle cx="380" cy="150" r="15" class="f-warn-soft s-warn" stroke-width="2"/><text x="380" y="154" class="t-b" text-anchor="middle">f</text><text x="402" y="154" class="t-small">hạng 5</text><circle cx="100" cy="150" r="15" class="f-bg s-muted" stroke-width="2"/><text x="100" y="154" class="t-b" text-anchor="middle">b</text><text x="78" y="154" class="t-small" text-anchor="end">hạng 4</text><circle cx="380" cy="236" r="15" class="f-bg s-muted" stroke-width="2"/><text x="380" y="240" class="t-b" text-anchor="middle">e</text><text x="402" y="240" class="t-small">hạng 3</text><circle cx="240" cy="236" r="15" class="f-bg s-muted" stroke-width="2"/><text x="240" y="240" class="t-b" text-anchor="middle">d</text><text x="218" y="240" class="t-small" text-anchor="end">hạng 2</text><circle cx="100" cy="310" r="15" class="f-phys-soft s-phys" stroke-width="2"/><text x="100" y="314" class="t-b" text-anchor="middle">a</text><text x="78" y="314" class="t-small" text-anchor="end">hạng 1</text><text x="113" y="234" class="t-mono" text-anchor="middle">4</text><text x="181.051" y="207.846" class="t-mono" text-anchor="middle">2</text><text x="175.543" y="132.759" class="t-mono" text-anchor="middle">5</text><text x="253" y="164" class="t-mono" text-anchor="middle">5</text><text x="316.804" y="208.077" class="t-mono" text-anchor="middle">3</text><text x="319.562" y="155.193" class="t-mono" text-anchor="middle">2</text><text x="393" y="197" class="t-mono" text-anchor="middle">5</text><text x="338" y="102" class="t-mono" text-anchor="middle">7, qua e</text><rect x="460" y="20" width="400" height="332" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="476" y="46" class="t-b">Truy vấn a → f (F: xuôi từ a, B: ngược từ f)</text><text x="476" y="76">1. F chốt a (0); B chốt f (0)</text><text x="476" y="100">2. F chốt c (2): B đã có nhãn c = 7</text><text x="476" y="124">    → μ = 2 + 7 = 9, nút gặp c</text><text x="476" y="148">3. F chốt b (4): b → c = 9, không tốt hơn 2</text><text x="476" y="172">4. B chốt c (7): 7 + 2 = 9, không &lt; μ</text><text x="476" y="196">5. không còn khoá nào &lt; μ: dừng, μ = 9</text><rect x="476" y="226" width="368" height="96" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="488" y="248" class="t-b">Giải nén: f → c là shortcut qua e</text><text x="488" y="268" class="t-small">c → f = c–e (2) + e–f (5)</text><text x="488" y="286" class="t-small">Đường gốc: a–c (2), c–e (2), e–f (5) = 9</text><text x="488" y="304" class="t-small">Năm lần chốt (a, f, c, b, c); c được cả hai chiều chốt.</text></svg>
</div>
<figcaption>Hình 2. Truy vấn a tới f trên đồ thị đi lên của ví dụ ở Hình 1. Chiều xuôi (xanh lục) đi lên từ a, chiều ngược (cam) đi lên từ f, gặp nhau ở c với μ = 2 + 7 = 9. Cung f → c là shortcut qua e, giải nén thành c–e (2) và e–f (5). Ví dụ dựng tay.</figcaption>
</figure>

Đường tìm được là các cung đi lên, trong đó có shortcut. Muốn trả về dãy cạnh gốc, phải **giải nén** shortcut đệ quy qua `mid` (nút bị co đã tạo ra nó) cho tới khi chỉ còn cạnh gốc.

`examples/toi-noi-location/rust/crates/route-engine/src/ch.rs`

```rust
    /// Giải nén cung `a → b` (hạng) thành các cạnh gốc, đệ quy qua nút giữa.
    fn unpack_arc(&self, a: u32, b: u32, out: &mut Vec<EdgeId>) {
        let (g, idx) = if a < b {
            let i = self.fwd.find(a, b).expect("cung thuận phải tồn tại");
            (&self.fwd, i)
        } else {
            let i = self.bwd.find(b, a).expect("cung ngược phải tồn tại");
            (&self.bwd, i)
        };
        let mid = g.mid[idx];
        if mid == NONE {
            out.push(g.edge[idx]);
        } else {
            self.unpack_arc(a, mid, out);
            self.unpack_arc(mid, b, out);
        }
    }
```

Trong Hình 2, shortcut `c → f` lưu ở `bwd` của f, có `mid = e`, nên giải nén thành `c → e` rồi `e → f`, cả hai là cạnh gốc, cho đường a, c, e, f dài 9, đúng bằng Dijkstra. Hai nhánh `a < b` và `a > b` cho biết cung nằm ở đồ thị thuận hay ngược, vì cả hai chỉ lưu cung hướng lên.

## Đúng: so với Dijkstra, fast_paths và truy vấn ngẫu nhiên

CH thay gần như toàn bộ cách tìm đường của Dijkstra, nên lập luận trên giấy chưa đủ, phải đối chiếu từng đáp án. Lab kiểm ba lớp trên đồ thị HCM, seed 20261106, ô tô, trọng số thời gian:

- **So với Dijkstra**: 0 khác biệt chi phí trên 500 truy vấn đầu.
- **So với `fast_paths`**: 0 khác biệt trên 2.000 truy vấn ngẫu nhiên. Đây là so với một cài đặt CH **độc lập** của người khác, dựng trên đúng cùng đầu vào.
- **Đường giải nén**: 0 đường sai (không liền mạch, sai tổng, hoặc sai nút đầu cuối) trên 2.000 truy vấn. Kiểm: tổng trọng số cạnh bằng chi phí báo cáo, nút cuối của cạnh trước là nút đầu của cạnh sau, và hai đầu đúng là nguồn và đích.

Test tự động nằm ở `crates/route-engine/tests/proptest_vs_dijkstra.rs` (CH, ALT, hai chiều, A* so với Dijkstra trên 400 trường hợp ngẫu nhiên, cả ba profile của fixture) và trong `ch.rs`.

## Nhanh bao nhiêu, và trả giá bao nhiêu

Truy vấn 2.000 cặp ngẫu nhiên trong thành phần liên thông mạnh lớn nhất; Dijkstra và A* chỉ chạy 500 cặp đầu vì chậm; mỗi truy vấn chạy nóng một lần trước khi bấm giờ.

| Cách làm | Trung vị (µs) | p99 (µs) |
|---|---:|---:|
| CH của lab, chỉ khoảng cách | 6,2 | 18,2 |
| CH của lab, khoảng cách và giải nén đường | 10,6 | 34,9 |
| `fast_paths::calc_path` (có đường) | 9,9 | 30,8 |
| A* chim bay | 2.016,2 | 7.424,9 |
| Dijkstra CSR | 3.355,9 | 7.101,1 |

Bảng lấy từ lần chạy có thời gian dựng thấp nhất, **đo khi máy chịu tải nặng** (tải 1 phút 20,36); qua ba lần, trung vị CH có đường là 10,6–10,9 µs và Dijkstra 3,4–3,7 ms. Hãy đọc tỉ lệ, đừng đọc số tuyệt đối. Mỗi truy vấn CH chốt trung bình 172 nút (cả hai chiều), Dijkstra chốt 56.142 nút: **ít hơn 326 lần**. Thời gian trung bình của Dijkstra trên CH cộng giải nén là 311 lần. Đuôi p99 của CH dài hơn trung vị nhiều lần (31,9 đến 46,4 µs), nên khi đặt ngân sách độ trễ (bài 29) phải nhìn p99 chứ không chỉ trung vị.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 440" role="img" aria-label="Hình 3. Trên đồ thị HCM, 2.000 truy vấn ô tô: trung vị thời gian một truy vấn (µs, thang log) của CH tự cài, fast_paths, A* chim bay và Dijkstra; đo khi máy chịu tải nặng. CH khoảng cách 6,2; CH cộng đường 10,6; fast_paths 9,9; A* 2016,2; Dijkstra 3355,9."><rect x="20" y="20" width="840" height="404" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36" y="46" class="t-b">Trung vị một truy vấn (µs, thang log) — đo khi máy chịu tải nặng (tải 1 phút 20,36)</text><text x="36" y="84" class="t-small">CH lab: chỉ khoảng cách</text><rect x="250" y="70" width="103.011" height="20" rx="3" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="361.011" y="84" class="t-mono">6,2</text><text x="36" y="116" class="t-small">CH lab: cộng giải nén đường</text><rect x="250" y="102" width="133.29" height="20" rx="3" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="391.29" y="116" class="t-mono">10,6</text><text x="36" y="148" class="t-small">fast_paths: calc_path</text><rect x="250" y="134" width="129.433" height="20" rx="3" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="387.433" y="148" class="t-mono">9,9</text><text x="36" y="180" class="t-small">A* chim bay (500 cặp)</text><rect x="250" y="166" width="429.589" height="20" rx="3" class="f-dead s-dead" stroke-width="1.5"/><text x="687.589" y="180" class="t-mono">2.016,2</text><text x="36" y="212" class="t-small">Dijkstra CSR (500 cặp)</text><rect x="250" y="198" width="458.355" height="20" rx="3" class="f-dead s-dead" stroke-width="1.5"/><text x="716.355" y="212" class="t-mono">3.355,9</text><path d="M250 230 L250 236" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><text x="250" y="250" class="t-small" text-anchor="middle">1</text><path d="M380 230 L380 236" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><text x="380" y="250" class="t-small" text-anchor="middle">10</text><path d="M510 230 L510 236" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><text x="510" y="250" class="t-small" text-anchor="middle">100</text><path d="M640 230 L640 236" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><text x="640" y="250" class="t-small" text-anchor="middle">1.000</text><path d="M770 230 L770 236" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><text x="770" y="250" class="t-small" text-anchor="middle">10.000</text><path d="M250 230 L770 230" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><text x="36" y="276" class="t-b">Tiền xử lý và cấu trúc (một luồng, cùng đầu vào)</text><text x="36" y="298" class="t-b"></text><text x="250" y="298" class="t-b">CH của lab</text><text x="520" y="298" class="t-b">fast_paths</text><text x="36" y="318" class="t-small">thời gian dựng</text><text x="250" y="318" class="t-mono">1,13 s</text><text x="520" y="318" class="t-mono">0,77 s</text><text x="36" y="338" class="t-small">cung đi lên</text><text x="250" y="338" class="t-mono">432.115 (3,41/nút)</text><text x="520" y="338" class="t-mono">394.394 (3,11/nút)</text><text x="36" y="358" class="t-small">bộ nhớ</text><text x="250" y="358" class="t-mono">8,53 MB (đo)</text><text x="520" y="358" class="t-mono">17,94 MB (ước tính)</text><text x="36" y="378" class="t-small">nút chốt mỗi truy vấn</text><text x="250" y="378" class="t-mono">172</text><text x="520" y="378" class="t-mono">—</text><text x="36" y="404" class="t-small">© OpenStreetMap contributors (ODbL). Nguồn: results/p03.md, LOC-25. Hai số bộ nhớ không cùng phương pháp đo.</text></svg>
</div>
<figcaption>Hình 3. Trên: trung vị thời gian một truy vấn ô tô trên đồ thị HCM (µs, thang log), đo khi máy chịu tải nặng. Dưới: so sánh tiền xử lý và cấu trúc giữa CH của lab và fast_paths. Hai số bộ nhớ không cùng phương pháp đo (một là đo, một là ước tính từ cấu trúc). © OpenStreetMap contributors (ODbL).</figcaption>
</figure>

Tiền xử lý hoà vốn sau 336 truy vấn so với Dijkstra (320 đến 369 qua ba lần chạy), cùng máy và cùng tải. Cỡ một giây cho cả thành phố nghĩa là dựng lại CH khi bản đồ đổi là chấp nhận được; khi **trọng số** đổi liên tục (kẹt xe theo giờ) thì không, và đó là câu hỏi của bài 27.

## So với fast_paths

[`fast_paths`](https://docs.rs/fast_paths) là một cài đặt CH hoàn chỉnh trên crates.io. Lab dựng nó trên đúng cùng đầu vào (cùng cạnh, cùng trọng số `u32`) để so thẳng.

| | CH của lab | `fast_paths` 1.0.0 |
|---|---:|---:|
| Thời gian dựng | 1,13 s (1,13–1,31) | 0,77 s (0,77–0,85) |
| Cung đi lên (tổng vào + ra) | 432.115 (3,41 mỗi nút) | 394.394 (3,11 mỗi nút) |
| Bộ nhớ | 8,53 MB (tính từ mảng thật) | 17,94 MB (ước tính từ cấu trúc) |
| Trung vị truy vấn có đường | 10,6 µs | 9,9 µs |

Lab chậm hơn 1,46 lần (1,46 đến 1,54) khi dựng và có nhiều hơn 9,6% số cung. Vì sao? Lab tách ảnh hưởng của hai yếu tố bằng một thí nghiệm: cho `fast_paths` chạy với **đúng thứ tự nút của lab**.

`examples/toi-noi-location/rust/labs/src/bin/loc25_contraction_hierarchies.rs`

```rust
    let order: Vec<usize> = ch.old_id.iter().map(|&v| v as usize).collect();
    let (fg2, fsecs2) =
        timed(|| fast_paths::prepare_with_order(&input, &order).expect("thứ tự hợp lệ"));
```

Kết quả: `fast_paths` với thứ tự của lab ra 431.244 cung, gần như bằng 432.115 của lab, và co xong chỉ trong 0,18 s. Hai điều rút ra. Một: chênh lệch số cung đến từ **thứ tự nút** (hàm ưu tiên của lab đơn giản hơn), không phải từ tìm kiếm nhân chứng hay cách lưu. Hai: với `fast_paths`, phần lớn thời gian dựng là **tìm thứ tự**, không phải co: 0,18 s co so với 0,77 s tổng cộng. Phép đo này không tách được phần tương ứng của lab. Truy vấn thì gần như ngang nhau: thời gian trung bình của `fast_paths` bằng 0,95 lần của CH lab (khoảng cách cộng đường).

Hai con số cần đọc thận trọng. Bộ nhớ 8,53 so với 17,94 MB so hai thứ khác phương pháp (lab đo mảng `u32` thật; `fast_paths` ước tính từ cấu trúc: 3 `usize` mỗi nút, 40 byte mỗi cung), nên chỉ biết cỡ. Còn «chậm hơn 1,46 lần» không phải so sánh công bằng về chất lượng cài đặt.

## CH này là theo nút: không mang chi phí rẽ

Đây là hệ quả quan trọng nhất của bài, và các bài sau dựa vào nó. CH của lab dựng trên `View`, tức đồ thị **theo nút** của bài 21 (comment đầu hàm `Ch::build` ghi rõ: «theo nút, bỏ qua chi phí rẽ»). Một shortcut `u → x` qua `v` lưu một con số: `w(u,v) + w(v,x)`. Nhưng chi phí rẽ và cấm rẽ ở bài 23 phụ thuộc vào **cặp cạnh** (cạnh vào `v`, cạnh ra `v`), mà shortcut đã gộp hai cạnh thành một cung mất thông tin đó.

Hệ quả cụ thể:

- Truy vấn CH có thể trả đường quay ngược miễn phí tại một nút, hoặc rẽ vào chỗ OSM cấm, vì nó không biết gì về chuyện đó.
- **Chỉ có A\* theo cạnh của bài 23, và simulator dùng nó, mới có chi phí rẽ, cấm rẽ và quay đầu.** CH và ma trận many-to-many dựng trên CH (bài 26) là theo nút. Ma trận «đúng phía đường» ở bài 26 chỉ mô hình việc đi vòng để vào đúng chiều của cạnh, không phải chi phí rẽ đầy đủ.
- Khoảng cách giữa hai mô hình là con số đã đo ở bài 23: trên 300 truy vấn HCM, tuyến theo nút vi phạm luật rẽ đầy đủ ở 49,7% và ước thời gian thấp hơn trung bình 80,7 s, phần lớn là chi phí rẽ giả định cộng dồn. Con số phụ thuộc bảng giả định; cái cần nhớ là CH trả lời nhanh một mô hình thô hơn, không cho chi phí đúng tới từng chỗ rẽ.

CH dựng trên đồ thị theo cạnh là hướng khác; lab **không** làm nó. Mỗi bài sau phải nêu rõ dùng mô hình nào, vì lựa chọn đó đổi cả `u_turns` lẫn km rỗng đo được.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` (hoặc thêm `-- --fixture`). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc25_contraction_hierarchies
```

Cấu hình đo: `--release`, Apple M1 Max, đồ thị HCM 126.699 nút, ô tô, trọng số thời gian (ms), seed 20261106, một luồng cho tiền xử lý, `fast_paths` 1.0.0. Output (rút gọn) của lần chạy có thời gian dựng CH thấp nhất trong ba lần, **đo khi máy đang chịu tải nặng** (tải 1 phút 20,36):

```text
-- 1. tiền xử lý: CH của lab --
  thời gian dựng: 1.13 s (một luồng)
  cạnh gốc đi được (đã gộp song song): 250.580; shortcut thêm: 181.535 (0.72x số cạnh gốc); độ sâu phân cấp lớn nhất (level): 72
  cung trong hai đồ thị đi lên: thuận 215.975, ngược 216.140, tổng 432.115 (3.41 cung/nút); bộ nhớ 8.53 MB

-- 2. tiền xử lý: fast_paths trên cùng đầu vào --
  thời gian dựng: 0.77 s; nút 126.699, cung đi lên: vào 197.274, ra 197.120, tổng 394.394 (3.11 cung/nút); bộ nhớ ước tính từ cấu trúc (3 usize/nút + 40 byte/cung) 17.94 MB
  fast_paths với đúng THỨ TỰ NÚT của lab (tách ảnh hưởng của thứ tự khỏi tìm kiếm nhân chứng): 0.18 s, tổng cung đi lên 431.244 so với 432.115 của lab
  so sánh thời gian dựng: lab / fast_paths = 1.46x

-- 3. tính đúng --
  CH của lab so với Dijkstra: 0 khác biệt trên 500 truy vấn
  CH của lab so với fast_paths: 0 khác biệt trên 2000 truy vấn
  đường giải nén sai (không liền mạch / sai tổng / sai đầu cuối): 0 trên 2000

-- 4. tốc độ truy vấn (2000 cặp ngẫu nhiên trong SCC lớn nhất; Dijkstra/A* chỉ 500 cặp đầu) --
  cách làm                           trung bình µs  trung vị µs       p99 µs
  CH lab: chỉ khoảng cách                     6.4          6.2         18.2
  CH lab: khoảng cách + giải nén đường         10.8         10.6         34.9
  fast_paths: calc_path (có đường)           10.3          9.9         30.8
  Dijkstra CSR                             3372.1       3355.9       7101.1
  A* chim bay                              2499.2       2016.2       7424.9
  nút đã chốt mỗi truy vấn: CH lab trung bình 172 (cả hai chiều), Dijkstra trung bình 56142 (trên 500 cặp đầu)
  Dijkstra / CH lab (khoảng cách + đường) trung bình = 311x; fast_paths / CH lab = 0.95x

-- 5. tiền xử lý đổi lấy truy vấn --
  mỗi truy vấn CH nhanh hơn Dijkstra 3361 µs; tiền xử lý 1.13 s hoà vốn sau 336 truy vấn (so với Dijkstra, cùng máy, cùng tải)
```

Các số đếm (cung, shortcut, nút chốt, số khác biệt) tái lập từng chữ số với cùng seed; thời gian dựng và µs thì không. Lệnh có thêm cờ `--save-ch PATH` để ghi CH ra file nhị phân dùng lại, và `Ch::save` và `Ch::load` làm việc đó bằng `postcard`.

## Giới hạn của ví dụ

- **Theo nút, không có chi phí rẽ.** Điều đã nói ở phần trước; không dùng CH này để báo thời gian đón tới từng lần rẽ.
- **Trọng số tĩnh.** CH dựng cho một bộ trọng số. Đổi tốc độ theo giờ (bài 27) cần dựng lại hoặc dùng Customizable CH.
- **Hàm ưu tiên đơn giản, nhân chứng giới hạn 500 nút.** Chưa so các hàm khác, chưa tinh chỉnh `limit`; lab dựng chậm hơn `fast_paths` 1,46 lần.
- **Một luồng tiền xử lý, chưa có CH cho đi bộ** (đi bộ dùng A* và tìm kiếm có giới hạn).
- **Đo thời gian khi máy chịu tải nặng.** Chỉ tỉ lệ trong cùng một lần chạy đáng tin; chưa có lần đo trên máy rảnh.
- **Đây là engine để học.** Không tuyên bố thay được OSRM hay Valhalla: lưu lượng, kẹt xe thật và cập nhật bản đồ chưa được đo (bài 28).

## Bài tập

**Bài 1.** Trên đồ thị bảy cạnh ở Hình 1, đổi thứ tự co thành a, e, d, b, f, c (co e trước d). Có mấy shortcut và trọng số bao nhiêu? Vì sao kết quả giống hay khác thứ tự trong bài?

<details><summary>Lời giải bài 1</summary>
<p>Co a: như cũ, không shortcut (nhân chứng b–c = 5 ≤ 6). Co e: c–f qua e = 2 + 5 = 7; d chưa bị co nên nhân chứng c–d–f = 8 không ngắn hơn 7 (8 &gt; 7), nên <b>thêm shortcut c–f = 7</b> qua e. Co d: c–f qua d = 8, nhưng bây giờ đã có shortcut c–f = 7 ≤ 8 làm nhân chứng, nên không thêm. Tổng vẫn đúng một shortcut c–f = 7, cùng trọng số như trong bài. Khác nhau chỉ ở chỗ nhân chứng của bước co d là shortcut vừa tạo, không phải đường c–e–f. Ở đồ thị nhỏ này thứ tự không đổi số shortcut; ở đồ thị lớn thì có (432.115 cung của lab so với 394.394 của <code>fast_paths</code>), nhưng không bao giờ đổi đáp án.</p>
</details>

**Bài 2.** Chi phí dựng 1,13 s và mỗi truy vấn tiết kiệm 3.361 µs so với Dijkstra. Nếu bản đồ được dựng lại mỗi lần cập nhật OSM, và mỗi ngày service chạy 5.000 truy vấn định tuyến, thì tỉ lệ thời gian dựng trên thời gian tiết kiệm trong một ngày là bao nhiêu? Kết luận về việc dựng lại CH hằng ngày.

<details><summary>Lời giải bài 2</summary>
<p>5.000 truy vấn tiết kiệm <code>5.000 × 3.361 µs ≈ 16,8 s</code>; thời gian dựng 1,13 s, tức khoảng 7% thời gian tiết kiệm. Dựng lại hằng ngày hoàn toàn chấp nhận được, và hoà vốn sau 336 truy vấn. Số này dựa trên số đo khi máy chịu tải nặng và số truy vấn 5.000 là số giả định của bài tập, nên chỉ cho thấy cỡ. Khi trọng số đổi từng phút thì cần cách khác (bài 27).</p>
</details>

**Bài 3.** Vì sao điều kiện dừng `top_f + top_b ≥ μ` của bài 22 không được dùng nguyên xi cho truy vấn CH, và lab dùng điều kiện nào?

<details><summary>Lời giải bài 3</summary>
<p>Chứng minh ở bài 22 dựa vào việc mọi nút có khoảng cách nhỏ hơn khoá nhỏ nhất đã được chốt với đúng khoảng cách, trên cùng một đồ thị. Ở CH, hai chiều chạy trên hai đồ thị đi lên khác nhau và chỉ gặp nhau ở nút hạng cao nhất của đường; chiều xuôi không bao giờ đi qua phần đường «đi xuống», nên nút gặp có thể đã chốt ở một chiều trong khi chiều kia còn xa, và tổng hai khoá có thể vượt μ trước khi đường tốt nhất lộ ra. Điều kiện hợp lệ cho cấu trúc này là mỗi chiều chỉ dừng khi khoá nhỏ nhất của nó đã không nhỏ hơn μ, và đó là điều lab làm bằng <code>filter(|&k| k &lt; mu)</code>. Tính đúng của cách đó được kiểm bằng so sánh với Dijkstra trên 500 truy vấn và với <code>fast_paths</code> trên 2.000 truy vấn, 0 khác biệt.</p>
</details>

Bài trước: [bài 24](/articles/location-profile-xe-may-o-to-di-bo-trong-hem). Bài tiếp: [bài 26](/articles/location-ma-tran-many-to-many-bucket-ch).

## Tài liệu gốc

- R. Geisberger, P. Sanders, D. Schultes, D. Delling, «Contraction Hierarchies: Faster and Simpler Hierarchical Routing in Road Networks», Experimental Algorithms (WEA 2008), LNCS 5038, trang 319–333: [doi.org/10.1007/978-3-540-68552-4_24](https://doi.org/10.1007/978-3-540-68552-4_24). Bài báo gốc của CH.
- [fast_paths trên docs.rs](https://docs.rs/fast_paths) và [kho mã easbar/fast_paths](https://github.com/easbar/fast_paths): cài đặt CH bằng Rust dùng để đối chiếu.
- E. W. Dijkstra, «A note on two problems in connexion with graphs», Numerische Mathematik 1, 1959: [doi.org/10.1007/BF01386390](https://doi.org/10.1007/BF01386390).
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho hình có dữ liệu OSM.
