---
slug: location-dijkstra-a-star-rust-do-thi-csr
title: "Dijkstra và A* trên đồ thị CSR: tuyến đường đầu tiên"
summary: "Bài 21: cài Dijkstra và A* trên đồ thị CSR bằng BinaryHeap, kiểm heuristic chim bay hợp lệ trên mọi cạnh, đếm nút đã duyệt trên đồ thị HCM 126.699 nút và đối chiếu Vec<Vec>, petgraph."
category: system-design
level: intermediate
tags: [toi-noi, location, rust, routing, dijkstra, a-star, csr, petgraph, openstreetmap]
language: vi
---

Tài xế DR-001 nhận chuyến TN-0001, khách ở bên kia thành phố. Service cần một con số trước khi báo giờ đón: đường ô tô từ chỗ tài xế tới chỗ khách dài bao nhiêu mét và mất bao nhiêu giây. Ở đồ thị HCM của lab, một cặp như vậy là nút 120147 tới nút 94147: chim bay 19.263 m, nhưng đường ô tô dài 23.687 m, đi qua 408 cạnh, ước tính 2.193,9 s. Để tìm ra tuyến đó, A* phải chốt 81.881 nút và xét 181.241 cạnh, tức 73% số nút của thành phần liên thông lớn nhất của ô tô.

Đây là bài 21 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi), và là bài đầu tiên của phần P03 (định tuyến). Bài dùng đồ thị CSR của [bài 06](/articles/location-osm-pbf-do-thi-duong-bo-xe-may-o-to). Mọi tốc độ ở đây là **giả định của lab** theo loại đường, không đo từ xe thật; mọi số thời gian đo khi máy đang chịu tải rất nặng, nên bài ưu tiên số nút đã duyệt và tỉ lệ, là những số tái lập được từng chữ số.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>A* dùng khoảng cách chim bay để «nhìn trước» về phía đích. Bạn kỳ vọng nó giảm số nút phải duyệt bao nhiêu lần so với Dijkstra? Và kỳ vọng đó khác thế nào khi trọng số cạnh là <b>khoảng cách</b> so với khi là <b>thời gian</b>?</p>
<details><summary>Xem đáp án</summary><p>Trên 300 truy vấn ô tô ở HCM, với trọng số khoảng cách A* duyệt ít hơn Dijkstra <b>3,50 đến 6,66 lần</b> tùy nhóm khoảng cách. Với trọng số thời gian chỉ còn <b>1,43 đến 2,15 lần</b>. Heuristic vẫn hợp lệ, nhưng nó phải giả định xe đi nhanh nhất mạng đường (80 km/h trên cao tốc) nên «nhìn trước» rất lỏng. Phần dưới giải thích bằng số.</p></details>
</div>

## Đồ thị CSR: vì sao không dùng đối tượng con trỏ

Đồ thị HCM có 126.699 nút và 319.074 cạnh có hướng, trong đó 251.687 cạnh ô tô đi được. CSR giữ chúng trong ba mảng phẳng: `first_out` (dài số nút cộng một), `head` (nút đích của mỗi cạnh) và một mảng trọng số cùng chỉ số cạnh. Các cạnh ra khỏi nút `u` nằm liền nhau ở đoạn `first_out[u]..first_out[u+1]`, nên duyệt láng giềng là đọc một đoạn bộ nhớ liên tục. Đồ thị kiểu đối tượng (mỗi nút một đối tượng, mỗi cạnh một con trỏ) thì mỗi bước duyệt là một lần nhảy tới vùng nhớ khác, và mỗi cạnh tốn thêm phần đầu đối tượng.

`examples/toi-noi-location/rust/crates/route-engine/src/net.rs`

```rust
    /// Cạnh đi ra khỏi `u` mà profile đi được: `(cạnh, nút đích, trọng số)`.
    #[inline]
    pub fn out(&self, u: NodeId) -> impl Iterator<Item = (EdgeId, NodeId, u32)> + 'a {
        let (g, w) = (self.g, self.w);
        g.out_edges(u).filter_map(move |e| {
            let c = w[e as usize];
            (c != INF).then(|| (e, g.head[e as usize], c))
        })
    }
```

`View` chỉ là ba tham chiếu (đồ thị, CSR chiều ngược, mảng trọng số), nên sao chép gần như miễn phí. Cạnh mà profile không đi được có trọng số `INF` và bị bỏ qua ngay trong iterator. Nhờ vậy ô tô, xe máy và đi bộ chạy chung một bản đồ trong bộ nhớ (bài 24 đi sâu hơn).

Lab so CSR với hai cách làm quen thuộc, cùng 300 truy vấn Dijkstra trọng số thời gian:

| Cách làm | Bộ nhớ | Thời gian so với CSR |
|---|---:|---:|
| CSR (`first_out` + `head` + trọng số, giữ cả cạnh ô tô không đi được) | 2,92 MB | 1,00 |
| `Vec<Vec<(đích, trọng số)>>`, chỉ giữ cạnh đi được | 6,39 MB | 1,17–1,19 |
| `petgraph::algo::dijkstra` trên `DiGraph<(), u32>` | không đo | 1,38–1,41 |

Cột cuối là tỉ lệ thời gian trung bình, giá trị nhỏ nhất đến lớn nhất qua ba lần chạy, đo khi máy chịu tải nặng. Cả ba cho đúng chi phí ở 300/300 truy vấn. Hãy đọc con số cho đúng cỡ: CSR **không** làm Dijkstra nhanh hơn một bậc độ lớn, chỉ nhanh hơn dưới 1,5 lần. Lý do giữ CSR là nó gọn (chưa tới một nửa bộ nhớ, dù còn giữ cả cạnh ô tô không đi được) và là nền tự nhiên cho những thứ sau: CSR chiều ngược cho tìm hai chiều (bài 22), nhãn theo hạng cho Contraction Hierarchies (bài 25).

## Dijkstra: hàng đợi ưu tiên và nhãn có phiên bản

Dijkstra giữ một tập nút «đã chốt» (khoảng cách chắc chắn nhỏ nhất) và một hàng đợi ưu tiên các nút «đã chạm nhưng chưa chốt». Mỗi vòng lặp: lấy nút `u` có khoảng cách tạm nhỏ nhất ra khỏi hàng đợi, chốt nó, rồi với mỗi cạnh `u → v` có trọng số `c`, nếu `d(u) + c < d(v)` thì cập nhật `d(v)` và đẩy `v` vào hàng đợi. Nó dừng khi chốt được nút đích. Tính đúng đến từ trọng số không âm: nút nhỏ nhất trong hàng đợi không thể được cải thiện bằng đường đi qua một nút còn lớn hơn.

`BinaryHeap` của Rust là max-heap, nên lab bọc khóa trong `Reverse` và đóng gói khóa với nút vào một `u64`: 32 bit cao là khóa, 32 bit thấp là nút. So sánh một số nguyên 64 bit đơn giản hơn so sánh một cặp, và khi hai nút bằng khóa thì nút có chỉ số nhỏ hơn đi trước, làm kết quả tất định.

`examples/toi-noi-location/rust/crates/route-engine/src/net.rs`

```rust
/// Hàng đợi ưu tiên nhỏ nhất ở đầu: khoá và nút đóng gói trong một `u64`.
#[derive(Debug, Default, Clone)]
pub struct MinHeap(BinaryHeap<Reverse<u64>>);

impl MinHeap {
    #[inline]
    pub fn push(&mut self, key: u32, node: u32) {
        self.0
            .push(Reverse((u64::from(key) << 32) | u64::from(node)));
    }
    #[inline]
    pub fn pop(&mut self) -> Option<(u32, u32)> {
        self.0.pop().map(|Reverse(x)| ((x >> 32) as u32, x as u32))
    }
}
```

`BinaryHeap` không có thao tác «giảm khóa». Cách thông thường, và cách lab dùng, là **xóa lười**: khi khoảng cách của `v` giảm, đẩy thêm một mục mới và để mục cũ nằm lại; lúc lấy ra, mục nào có nút đã chốt thì bỏ qua. Hàng đợi vì thế có thể chứa vài mục cho cùng một nút, nhưng mỗi nút chỉ được chốt một lần.

Chuyện thứ hai là khởi tạo. Cấp một mảng khoảng cách `vec![INF; 126_699]` cho mỗi truy vấn là việc tốn hơn cả truy vấn nhỏ. `Labels` giải quyết bằng số phiên bản: mỗi nút có `seen[v]` và `closed[v]`, và nhãn chỉ hợp lệ nếu bằng `cur` của truy vấn hiện tại. Đặt lại cho truy vấn kế tiếp chỉ là `cur += 1`.

`examples/toi-noi-location/rust/crates/route-engine/src/net.rs`

```rust
    pub fn reset(&mut self) {
        self.cur = self.cur.wrapping_add(1);
        if self.cur == 0 {
            self.seen.fill(0);
            self.closed.fill(0);
            self.cur = 1;
        }
        self.touched.clear();
    }
    #[inline]
    pub fn dist(&self, v: NodeId) -> u32 {
        if self.seen[v as usize] == self.cur {
            self.dist[v as usize]
        } else {
            INF
        }
    }
```

Chỉ khi bộ đếm 32 bit quay về 0 (sau hơn bốn tỉ truy vấn) mới phải xóa thật. Tất cả `Searcher` dùng lại bộ nhớ này giữa các truy vấn, nên một luồng phục vụ chỉ cần một `Searcher`.

## A*: cùng một vòng lặp với khóa cộng thêm h

Dijkstra thực chất là A* với heuristic bằng 0. Vì vậy lab chỉ có **một** hàm lõi, `Searcher::run`, nhận heuristic `h` là một closure. Khóa trong hàng đợi là `d(v) + h(v)` thay vì `d(v)`, còn mọi thứ khác giữ nguyên.

`examples/toi-noi-location/rust/crates/route-engine/src/search.rs`

```rust
        self.labels.set(s, 0, crate::net::NONE);
        self.heap.push(h(s), s);
        while let Some((_, u)) = self.heap.pop() {
            if self.labels.is_closed(u) {
                continue;
            }
            self.labels.close(u);
            st.settled += 1;
            let d = self.labels.dist(u);
            if u == t {
                let edges = trace_back(view.g, &self.labels, t);
                return (Some(Path { cost: d, edges }), st);
            }
            for (e, v, c) in view.out(u) {
                st.relaxed += 1;
                let nd = d.saturating_add(c);
                if nd < self.labels.dist(v) {
                    self.labels.set(v, nd, e);
                    self.heap.push(nd.saturating_add(h(v)), v);
                }
            }
        }
```

Hai dòng quan trọng nhất: `is_closed(u)` bỏ mục cũ của xóa lười, và `st.settled += 1` đếm số nút đã chốt, con số mà mọi so sánh trong bài dựa vào. Để dựng lại đường, mỗi nút lưu cạnh đã cải thiện nó (`pred`); `trace_back` đi ngược từ đích về nguồn rồi đảo mảng.

Tính đúng của A* phụ thuộc vào hai tính chất của `h`. **Hợp lệ** (admissible): `h(u)` không vượt chi phí thật từ `u` tới đích. **Nhất quán** (consistent): `h(u) ≤ w(u,v) + h(v)` trên mọi cạnh. Vòng lặp ở trên chốt mỗi nút một lần và không mở lại, nên nó cần tính mạnh hơn là nhất quán: với `h(t) = 0`, cộng dồn bất đẳng thức dọc đường đi sẽ ra hợp lệ, và nút lấy ra khỏi hàng đợi đã có khoảng cách đúng. Chỉ hợp lệ mà không nhất quán thì phải cho phép mở lại nút, nếu không đáp án có thể sai.

Với chim bay, nhất quán suy ra từ bất đẳng thức tam giác miễn là `h` đổi từ mét sang đơn vị trọng số bằng cách chia cho một tốc độ **không nhỏ hơn** tốc độ nhanh nhất của đồ thị: `h(u) − h(v)` không vượt chim bay từ `u` tới `v` chia tốc độ đó, và số này không vượt `w(u,v)`. Làm tròn xuống bằng `as u32` không phá tính chất này, vì `w` là số nguyên.

Tốc độ đó là `vmax`: số cm đi được trong một đơn vị trọng số, lớn nhất trên mọi cạnh đi được. Lab lấy nó **từ chính dữ liệu**, nhân thêm hệ số an toàn 0,99 vì đoạn thẳng trên mặt phẳng cục bộ có thể dài hơn đường thật cỡ 0,1%.

`examples/toi-noi-location/rust/crates/route-engine/src/search.rs`

```rust
    /// Hệ số đổi mét chim bay thành đơn vị trọng số: `100 * an_toàn / vmax` (vmax tính theo cm trên đơn vị).
    pub fn scale(vmax_cm_per_unit: f64) -> f64 {
        100.0 * HEURISTIC_SAFETY / vmax_cm_per_unit
    }

    /// A* với heuristic chim bay `xy` và hằng số `vmax` của profile (xem [`NodeXy::scale`]).
    pub fn astar(
        &mut self,
        view: View,
        xy: &NodeXy,
        vmax_cm_per_unit: f64,
        s: NodeId,
        t: NodeId,
    ) -> (Option<Path>, Stats) {
        let scale = NodeXy::scale(vmax_cm_per_unit);
        let tp = xy.xy[t as usize];
        self.run(view, s, t, move |u| {
            (xy.dist_to_point_m(u, tp) * scale) as u32
        })
    }
```

Hình 1 vẽ cơ chế trên một lưới 9×5 nút dựng tay (không phải dữ liệu OSM), mỗi cạnh 100 m, hai đoạn bị chặn nằm chắn giữa xe và khách. Số trong nút là thứ tự chốt, tính bằng đúng hai quy tắc ở trên, kể cả quy tắc hòa theo chỉ số nút. Dijkstra lan thành một vòng tròn quanh xe và chốt 28 nút trước khi chạm khách. A* đi về phía khách, chốt 15 nút và vẫn trả đúng chi phí 700 m.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 416" role="img" aria-label="Hình 1. Dijkstra và A* trên lưới 9×5 nút dựng tay, mỗi cạnh 100 m. Số trong nút là thứ tự chốt. Dijkstra chốt 28 nút, A* với heuristic chim bay chốt 15 nút, cùng chi phí 700 m."><rect x="20" y="20" width="410" height="340" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36" y="46" class="t-b">Dijkstra: lan đều mọi hướng</text><path d="M50 100 L93.5 100" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M50 100 L50 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M93.5 100 L137 100" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M93.5 100 L93.5 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M137 100 L180.5 100" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M137 100 L137 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M180.5 100 L224 100" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M180.5 100 L180.5 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M224 100 L267.5 100" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M224 100 L224 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M267.5 100 L311 100" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M267.5 100 L267.5 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M311 100 L354.5 100" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M311 100 L311 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M354.5 100 L398 100" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M354.5 100 L354.5 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M398 100 L398 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M50 143.5 L93.5 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M50 143.5 L50 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M93.5 143.5 L137 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M93.5 143.5 L93.5 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M137 143.5 L137 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M180.5 143.5 L224 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M180.5 143.5 L180.5 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M224 143.5 L267.5 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M224 143.5 L224 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M267.5 143.5 L311 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M267.5 143.5 L267.5 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M311 143.5 L354.5 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M311 143.5 L311 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M354.5 143.5 L398 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M354.5 143.5 L354.5 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M398 143.5 L398 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M50 187 L93.5 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M50 187 L50 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M93.5 187 L137 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M93.5 187 L93.5 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M137 187 L137 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M180.5 187 L224 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M180.5 187 L180.5 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M224 187 L267.5 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M224 187 L224 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M267.5 187 L311 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M267.5 187 L267.5 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M311 187 L354.5 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M311 187 L311 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M354.5 187 L398 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M354.5 187 L354.5 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M398 187 L398 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M50 230.5 L93.5 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M50 230.5 L50 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M93.5 230.5 L137 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M93.5 230.5 L93.5 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M137 230.5 L180.5 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M137 230.5 L137 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M180.5 230.5 L224 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M180.5 230.5 L180.5 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M224 230.5 L267.5 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M224 230.5 L224 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M267.5 230.5 L311 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M267.5 230.5 L267.5 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M311 230.5 L354.5 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M311 230.5 L311 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M354.5 230.5 L398 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M354.5 230.5 L354.5 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M398 230.5 L398 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M50 274 L93.5 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M93.5 274 L137 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M137 274 L180.5 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M180.5 274 L224 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M224 274 L267.5 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M267.5 274 L311 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M311 274 L354.5 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M354.5 274 L398 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M137 143.5 L180.5 143.5" class="f-none s-warn dash" stroke-width="1.5" stroke-linecap="round"/><path d="M137 187 L180.5 187" class="f-none s-warn dash" stroke-width="1.5" stroke-linecap="round"/><path d="M267.5 187 L224 187 L180.5 187 L180.5 230.5 L137 230.5 L137 187 L93.5 187 L50 187" class="f-none s-phys" stroke-width="4.5" stroke-linejoin="round" stroke-linecap="round"/><circle cx="50" cy="100" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="50" y="103.5" class="t-small" text-anchor="middle">5</text><circle cx="93.5" cy="100" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="93.5" y="103.5" class="t-small" text-anchor="middle">10</text><circle cx="137" cy="100" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="137" y="103.5" class="t-small" text-anchor="middle">14</text><circle cx="180.5" cy="100" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="180.5" y="103.5" class="t-small" text-anchor="middle">17</text><circle cx="224" cy="100" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="224" y="103.5" class="t-small" text-anchor="middle">21</text><circle cx="267.5" cy="100" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="267.5" y="103.5" class="t-small" text-anchor="middle">26</text><circle cx="311" cy="100" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="354.5" cy="100" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="398" cy="100" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="50" cy="143.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="50" y="147" class="t-small" text-anchor="middle">2</text><circle cx="93.5" cy="143.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="93.5" y="147" class="t-small" text-anchor="middle">6</text><circle cx="137" cy="143.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="137" y="147" class="t-small" text-anchor="middle">11</text><circle cx="180.5" cy="143.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="180.5" y="147" class="t-small" text-anchor="middle">22</text><circle cx="224" cy="143.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="224" y="147" class="t-small" text-anchor="middle">27</text><circle cx="267.5" cy="143.5" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="311" cy="143.5" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="354.5" cy="143.5" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="398" cy="143.5" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="50" cy="187" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="50" y="190.5" class="t-small" text-anchor="middle">1</text><circle cx="93.5" cy="187" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="93.5" y="190.5" class="t-small" text-anchor="middle">3</text><circle cx="137" cy="187" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="137" y="190.5" class="t-small" text-anchor="middle">7</text><circle cx="180.5" cy="187" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="180.5" y="190.5" class="t-small" text-anchor="middle">18</text><circle cx="224" cy="187" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="224" y="190.5" class="t-small" text-anchor="middle">23</text><circle cx="267.5" cy="187" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="267.5" y="190.5" class="t-small" text-anchor="middle">28</text><circle cx="311" cy="187" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="354.5" cy="187" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="398" cy="187" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="50" cy="230.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="50" y="234" class="t-small" text-anchor="middle">4</text><circle cx="93.5" cy="230.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="93.5" y="234" class="t-small" text-anchor="middle">8</text><circle cx="137" cy="230.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="137" y="234" class="t-small" text-anchor="middle">12</text><circle cx="180.5" cy="230.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="180.5" y="234" class="t-small" text-anchor="middle">15</text><circle cx="224" cy="230.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="224" y="234" class="t-small" text-anchor="middle">19</text><circle cx="267.5" cy="230.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="267.5" y="234" class="t-small" text-anchor="middle">24</text><circle cx="311" cy="230.5" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="354.5" cy="230.5" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="398" cy="230.5" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="50" cy="274" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="50" y="277.5" class="t-small" text-anchor="middle">9</text><circle cx="93.5" cy="274" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="93.5" y="277.5" class="t-small" text-anchor="middle">13</text><circle cx="137" cy="274" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="137" y="277.5" class="t-small" text-anchor="middle">16</text><circle cx="180.5" cy="274" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="180.5" y="277.5" class="t-small" text-anchor="middle">20</text><circle cx="224" cy="274" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="224" y="277.5" class="t-small" text-anchor="middle">25</text><circle cx="267.5" cy="274" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="311" cy="274" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="354.5" cy="274" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="398" cy="274" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="50" cy="187" r="15.5" class="f-none s-warn" stroke-width="2.5"/><circle cx="267.5" cy="187" r="15.5" class="f-none s-warn" stroke-width="2.5"/><text x="36" y="330" class="t-small">chốt 28 trên 45 nút; chi phí 700 m</text><rect x="450" y="20" width="410" height="340" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="466" y="46" class="t-b">A*: nghiêng về phía khách</text><path d="M480 100 L523.5 100" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M480 100 L480 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M523.5 100 L567 100" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M523.5 100 L523.5 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M567 100 L610.5 100" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M567 100 L567 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M610.5 100 L654 100" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M610.5 100 L610.5 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M654 100 L697.5 100" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M654 100 L654 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M697.5 100 L741 100" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M697.5 100 L697.5 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M741 100 L784.5 100" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M741 100 L741 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M784.5 100 L828 100" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M784.5 100 L784.5 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M828 100 L828 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M480 143.5 L523.5 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M480 143.5 L480 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M523.5 143.5 L567 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M523.5 143.5 L523.5 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M567 143.5 L567 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M610.5 143.5 L654 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M610.5 143.5 L610.5 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M654 143.5 L697.5 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M654 143.5 L654 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M697.5 143.5 L741 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M697.5 143.5 L697.5 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M741 143.5 L784.5 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M741 143.5 L741 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M784.5 143.5 L828 143.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M784.5 143.5 L784.5 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M828 143.5 L828 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M480 187 L523.5 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M480 187 L480 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M523.5 187 L567 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M523.5 187 L523.5 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M567 187 L567 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M610.5 187 L654 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M610.5 187 L610.5 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M654 187 L697.5 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M654 187 L654 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M697.5 187 L741 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M697.5 187 L697.5 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M741 187 L784.5 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M741 187 L741 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M784.5 187 L828 187" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M784.5 187 L784.5 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M828 187 L828 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M480 230.5 L523.5 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M480 230.5 L480 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M523.5 230.5 L567 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M523.5 230.5 L523.5 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M567 230.5 L610.5 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M567 230.5 L567 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M610.5 230.5 L654 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M610.5 230.5 L610.5 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M654 230.5 L697.5 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M654 230.5 L654 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M697.5 230.5 L741 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M697.5 230.5 L697.5 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M741 230.5 L784.5 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M741 230.5 L741 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M784.5 230.5 L828 230.5" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M784.5 230.5 L784.5 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M828 230.5 L828 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M480 274 L523.5 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M523.5 274 L567 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M567 274 L610.5 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M610.5 274 L654 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M654 274 L697.5 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M697.5 274 L741 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M741 274 L784.5 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M784.5 274 L828 274" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M567 143.5 L610.5 143.5" class="f-none s-warn dash" stroke-width="1.5" stroke-linecap="round"/><path d="M567 187 L610.5 187" class="f-none s-warn dash" stroke-width="1.5" stroke-linecap="round"/><path d="M697.5 187 L654 187 L654 230.5 L610.5 230.5 L567 230.5 L567 187 L523.5 187 L480 187" class="f-none s-phys" stroke-width="4.5" stroke-linejoin="round" stroke-linecap="round"/><circle cx="480" cy="100" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="523.5" cy="100" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="567" cy="100" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="610.5" cy="100" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="654" cy="100" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="697.5" cy="100" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="741" cy="100" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="784.5" cy="100" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="828" cy="100" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="480" cy="143.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="480" y="147" class="t-small" text-anchor="middle">4</text><circle cx="523.5" cy="143.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="523.5" y="147" class="t-small" text-anchor="middle">6</text><circle cx="567" cy="143.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="567" y="147" class="t-small" text-anchor="middle">8</text><circle cx="610.5" cy="143.5" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="654" cy="143.5" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="697.5" cy="143.5" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="741" cy="143.5" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="784.5" cy="143.5" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="828" cy="143.5" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="480" cy="187" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="480" y="190.5" class="t-small" text-anchor="middle">1</text><circle cx="523.5" cy="187" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="523.5" y="190.5" class="t-small" text-anchor="middle">2</text><circle cx="567" cy="187" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="567" y="190.5" class="t-small" text-anchor="middle">3</text><circle cx="610.5" cy="187" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="610.5" y="190.5" class="t-small" text-anchor="middle">12</text><circle cx="654" cy="187" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="654" y="190.5" class="t-small" text-anchor="middle">13</text><circle cx="697.5" cy="187" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="697.5" y="190.5" class="t-small" text-anchor="middle">15</text><circle cx="741" cy="187" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="784.5" cy="187" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="828" cy="187" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="480" cy="230.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="480" y="234" class="t-small" text-anchor="middle">5</text><circle cx="523.5" cy="230.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="523.5" y="234" class="t-small" text-anchor="middle">7</text><circle cx="567" cy="230.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="567" y="234" class="t-small" text-anchor="middle">9</text><circle cx="610.5" cy="230.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="610.5" y="234" class="t-small" text-anchor="middle">10</text><circle cx="654" cy="230.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="654" y="234" class="t-small" text-anchor="middle">11</text><circle cx="697.5" cy="230.5" r="11.5" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="697.5" y="234" class="t-small" text-anchor="middle">14</text><circle cx="741" cy="230.5" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="784.5" cy="230.5" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="828" cy="230.5" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="480" cy="274" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="523.5" cy="274" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="567" cy="274" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="610.5" cy="274" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="654" cy="274" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="697.5" cy="274" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="741" cy="274" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="784.5" cy="274" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="828" cy="274" r="5" class="f-bg s-dead" stroke-width="1.5"/><circle cx="480" cy="187" r="15.5" class="f-none s-warn" stroke-width="2.5"/><circle cx="697.5" cy="187" r="15.5" class="f-none s-warn" stroke-width="2.5"/><text x="466" y="330" class="t-small">chốt 15 trên 45 nút; chi phí 700 m</text><text x="24" y="384" class="t-small">Vòng cam: xe (ngoài cùng bên trái) và khách (bên phải). Nét xanh: đường tìm được. Nét cam đứt: hai đoạn bị chặn.</text><text x="24" y="400" class="t-small">Vòng tròn đầy: nút đã chốt, số là thứ tự chốt. Chấm nhỏ: nút chưa chốt.</text></svg>
</div>
<figcaption>Hình 1. Cùng một truy vấn trên lưới 9×5 dựng tay, mỗi cạnh 100 m, heuristic là chim bay nhân 0,99. Số trong nút là thứ tự chốt. Dijkstra chốt 28 nút, A* chốt 15 nút, cả hai ra đường dài 700 m. Hình minh họa cơ chế, không phải dữ liệu OSM.</figcaption>
</figure>

## Số nút đã duyệt trên đồ thị thật

Trên đồ thị HCM, lab lấy 300 cặp nút ngẫu nhiên trong thành phần liên thông mạnh lớn nhất của ô tô (111.441 nút, 88,0% đồ thị), seed 20261106, để mọi cặp đều có đường. Trước hết là kiểm heuristic: lab duyệt **mọi** 251.687 cạnh ô tô và kiểm chim bay quy đổi giữa hai đầu cạnh không vượt `w`. Kiểm từng cạnh là đủ, vì bất đẳng thức này cộng với bất đẳng thức tam giác cho nhất quán trên cả đồ thị, và nhất quán cho hợp lệ. Số vi phạm bằng 0 ở cả hai loại trọng số; tỉ lệ `h/w` lớn nhất là 0,9981 (khoảng cách) và 0,9898 (thời gian), tức chim bay gần như chạm chi phí thật ở một số cạnh rất thẳng (với thời gian là cạnh thẳng chạy đúng tốc độ `vmax`).

Rồi A* và Dijkstra trả cùng chi phí ở 300/300 truy vấn, cả hai loại trọng số. Điều khác nhau là số nút phải chốt (Hình 2):

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 448" role="img" aria-label="Hình 2. Số nút Dijkstra và A* chốt theo nhóm khoảng cách chim bay trên đồ thị HCM, 300 truy vấn ô tô. Trọng số khoảng cách: A* ít hơn 3,50 đến 6,66 lần. Trọng số thời gian: chỉ 1,43 đến 2,15 lần."><rect x="20" y="20" width="840" height="408" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36" y="46" class="t-b">Số nút đã chốt trung bình mỗi truy vấn (nhóm theo khoảng cách chim bay giữa hai đầu)</text><text x="36" y="84" class="t-b">Trọng số thời gian (ms)</text><text x="36" y="109" class="t-small">&lt; 2 km</text><rect x="96" y="96" width="7.08611" height="15" rx="2" class="f-dead s-dead" stroke-width="1"/><text x="109.086" y="108" class="t-mono">2.551</text><rect x="96" y="114" width="3.50278" height="15" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="105.503" y="126" class="t-mono">1.261  (2,02 lần ít hơn)</text><text x="36" y="169" class="t-small">2–5 km</text><rect x="96" y="156" width="52.8806" height="15" rx="2" class="f-dead s-dead" stroke-width="1"/><text x="154.881" y="168" class="t-mono">19.037</text><rect x="96" y="174" width="24.5917" height="15" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="126.592" y="186" class="t-mono">8.853  (2,15 lần ít hơn)</text><text x="36" y="229" class="t-small">5–10 km</text><rect x="96" y="216" width="147.939" height="15" rx="2" class="f-dead s-dead" stroke-width="1"/><text x="249.939" y="228" class="t-mono">53.258</text><rect x="96" y="234" width="78.0583" height="15" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="180.058" y="246" class="t-mono">28.101  (1,90 lần ít hơn)</text><text x="36" y="289" class="t-small">&gt; 10 km</text><rect x="96" y="276" width="240.711" height="15" rx="2" class="f-dead s-dead" stroke-width="1"/><text x="342.711" y="288" class="t-mono">86.656</text><rect x="96" y="294" width="168.528" height="15" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="270.528" y="306" class="t-mono">60.670  (1,43 lần ít hơn)</text><text x="456" y="84" class="t-b">Trọng số khoảng cách (cm)</text><text x="456" y="109" class="t-small">&lt; 2 km</text><rect x="516" y="96" width="6.76111" height="15" rx="2" class="f-dead s-dead" stroke-width="1"/><text x="528.761" y="108" class="t-mono">2.434</text><rect x="516" y="114" width="2" height="15" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="524" y="126" class="t-mono">366  (6,66 lần ít hơn)</text><text x="456" y="169" class="t-small">2–5 km</text><rect x="516" y="156" width="53.8861" height="15" rx="2" class="f-dead s-dead" stroke-width="1"/><text x="575.886" y="168" class="t-mono">19.399</text><rect x="516" y="174" width="8.13611" height="15" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="530.136" y="186" class="t-mono">2.929  (6,62 lần ít hơn)</text><text x="456" y="229" class="t-small">5–10 km</text><rect x="516" y="216" width="149.281" height="15" rx="2" class="f-dead s-dead" stroke-width="1"/><text x="671.281" y="228" class="t-mono">53.741</text><rect x="516" y="234" width="27.3361" height="15" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="549.336" y="246" class="t-mono">9.841  (5,46 lần ít hơn)</text><text x="456" y="289" class="t-small">&gt; 10 km</text><rect x="516" y="276" width="242.544" height="15" rx="2" class="f-dead s-dead" stroke-width="1"/><text x="764.544" y="288" class="t-mono">87.316</text><rect x="516" y="294" width="69.2278" height="15" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="591.228" y="306" class="t-mono">24.922  (3,50 lần ít hơn)</text><rect x="36" y="372" width="22" height="12" rx="2" class="f-dead s-dead" stroke-width="1"/><text x="66" y="382" class="t-small">Dijkstra</text><rect x="150" y="372" width="22" height="12" rx="2" class="f-pk-soft s-pk" stroke-width="1"/><text x="180" y="382" class="t-small">A* với heuristic chim bay</text><text x="36" y="412" class="t-small">Cùng thang đo ở hai nửa. Nguồn: results/p03.md, LOC-21 mục 3; seed 20261106, 300 truy vấn; © OpenStreetMap contributors (ODbL).</text></svg>
</div>
<figcaption>Hình 2. Số nút đã chốt trung bình mỗi truy vấn theo nhóm khoảng cách chim bay, ô tô, HCM, 300 truy vấn, seed 20261106. Xám: Dijkstra. Xanh: A*. Trọng số thời gian cho A* ít hơn 1,43 đến 2,15 lần, trọng số khoảng cách ít hơn 3,50 đến 6,66 lần. © OpenStreetMap contributors (ODbL).</figcaption>
</figure>

Vì sao trọng số thời gian kém đến vậy? `vmax` lấy từ dữ liệu là 2,2229 cm/ms, tức 80 km/h của đường cao tốc trong bảng tốc độ giả định, trong khi phần lớn đường nội thành được gán 12 đến 38 km/h. Hai nút cách nhau 1 km chim bay cho `h = 44,5 s` (100.000 cm × 0,99 / 2,2229 cm/ms), còn đi thật ở 28 km/h mất 128,6 s. Heuristic hợp lệ nhưng chỉ «biết» khoảng một phần ba chi phí, nên A* vẫn phải duyệt phần lớn vùng mà Dijkstra duyệt. Với trọng số khoảng cách thì `vmax` là 1 cm/đơn vị, chim bay sát đường thật hơn nhiều, và A* thu hẹp được nhiều.

Muốn heuristic thời gian sát hơn mà vẫn đúng thì phải thay chim bay bằng một chặn dưới rút ra từ chính đồ thị; bài 22 làm việc đó bằng landmark. Đặt `vmax` nhỏ hơn tốc độ thật để «cho nhanh» thì **không** an toàn: heuristic mất tính hợp lệ và A* có thể trả đường dài hơn tối ưu mà không báo lỗi. Đó là lý do bài tập 3 bắt phải có một kiểm tra tự động.

## Đối chiếu petgraph

`petgraph` là thư viện đồ thị phổ biến nhất của Rust, nên lab dựng cùng đồ thị (`DiGraph<(), u32>`, chỉ cạnh ô tô đi được) và chạy `algo::dijkstra` và `algo::astar` cùng 300 truy vấn. Hai thuật toán đó cho đúng chi phí của lab ở 300/300 truy vấn; đây là kiểm tra chéo giữa hai cài đặt độc lập, mạnh hơn việc chỉ so lab với chính nó.

Về tốc độ, bảng ở đầu bài đã nêu: `petgraph::dijkstra` chậm hơn CSR của lab 1,38–1,41 lần. Hai điều cần nói thẳng. Một: `petgraph` là thư viện tổng quát (kiểu nút, kiểu cạnh, `HashMap` cho nhãn), còn lab chuyên biệt hóa cho nút số nguyên và nhãn có phiên bản; lab không «hơn» petgraph, chỉ nhỏ gọn hơn cho đúng một bài toán. Hai: `petgraph::dijkstra` trả bản đồ nhãn trung bình 58.539 nút mỗi truy vấn, gồm cả nút đã chạm nhưng chưa chốt, nên không so trực tiếp được với số «nút đã chốt» ở trên.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` do lab bài 06 ghi (hoặc thêm `-- --fixture` để chạy trên đồ thị tổng hợp nhỏ). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc21_dijkstra_astar
```

Cấu hình đo: `--release`, Apple M1 Max, đồ thị HCM 126.699 nút và 319.074 cạnh, ô tô, 300 cặp nút, seed 20261106. Dưới đây là phần output của lần chạy cho Dijkstra CSR nhanh nhất trong ba lần, **đo khi máy đang chịu tải rất nặng** (tải 1 phút 27,25 trên 10 lõi): chỉ so sánh tương đối trong cùng bảng.

```text
-- 2. heuristic chim bay có hợp lệ không (kiểm trên mọi cạnh ô tô) --
  trọng số khoảng cách (cm): vmax = 1.0000 cm/đơn vị; max h/w trên một cạnh = 0.9981 (cạnh 227473); số cạnh vi phạm h > w = 0
  trọng số thời gian (ms): vmax = 2.2229 cm/đơn vị; max h/w trên một cạnh = 0.9898 (cạnh 106995); số cạnh vi phạm h > w = 0

-- 3. Dijkstra so với A* theo khoảng cách chim bay (ô tô) --
  trọng số thời gian; chi phí khác nhau giữa Dijkstra và A*: 0/300
  nhóm           n   Dijkstra nút         A* nút    tỉ lệ   Dijkstra µs         A* µs
  < 2 km        16           2551           1261    2.02x           156            88
  2–5 km        46          19037           8853    2.15x          1233           675
  5–10 km      126          53258          28101    1.90x          3532          2342
  > 10 km      112          86656          60670    1.43x          5676          5142
  trọng số khoảng cách; chi phí khác nhau giữa Dijkstra và A*: 0/300
  < 2 km        16           2434            366    6.66x           123            30
  2–5 km        46          19399           2929    6.62x          1053           244
  5–10 km      126          53741           9841    5.46x          2992           815
  > 10 km      112          87316          24922    3.50x          4779          2139

-- 4. cùng truy vấn: CSR so với Vec<Vec<..>> so với petgraph (ô tô, thời gian) --
  chi phí khác Dijkstra CSR: Vec<Vec>: 0, petgraph dijkstra: 0, petgraph astar: 0 (trên 300 truy vấn)

-- 5. một tuyến cụ thể --
  nút 120147 → nút 94147: chim bay 19263 m; thời gian 2193.9 s; độ dài 23687 m; 408 cạnh; A* duyệt 81.881 nút, xét 181.241 cạnh
```

Các số đếm (nút đã duyệt, số cạnh vi phạm, số khác biệt) tái lập từng chữ số với cùng seed; các cột µs thì không, và dao động vài lần giữa các lần chạy. Kiểm tra tự động nằm trong `search.rs`: test `heuristic_never_exceeds_edge_cost_for_both_metrics` kiểm `h ≤ w` trên mọi cạnh fixture, test `astar_equals_dijkstra_equals_sssp` so A*, Dijkstra và Dijkstra một nguồn tới mọi nút trên 200 cặp ngẫu nhiên ở cả hai loại trọng số.

## Giới hạn của ví dụ

- **Tốc độ là giả định.** Chi phí thời gian dựa trên bảng tốc độ theo loại đường của lab (bài 24), chưa hiệu chuẩn bằng GPS thật. «2.193,9 s» là ước tính của mô hình, không phải thời gian một chuyến thật.
- **Theo nút, chưa có chi phí rẽ.** Một cạnh nối tiếp cạnh nào cũng miễn phí, kể cả quay đầu. Bài 23 cho thấy điều này sai bao nhiêu.
- **Heuristic thời gian lỏng.** 1,43–2,15 lần là cái giá của `vmax` lấy từ cao tốc. Bài 22 dùng landmark để thắt lại.
- **Đo thời gian khi máy tải nặng.** Chưa có lần đo nào trên máy rảnh; các cột µs chỉ dùng so sánh tương đối.
- **Một bản đồ, đồ thị cố định.** Cập nhật OSM nghĩa là dựng lại đồ thị; chưa có trọng số theo giờ (bài 27).

## Bài tập

**Bài 1.** Với `vmax = 2,2229 cm/ms` và hệ số 0,99, tính `h` cho hai nút cách nhau 1 km chim bay. So với thời gian đi thật khi mọi cạnh là đường 28 km/h, tỉ lệ `h` trên chi phí thật là bao nhiêu?

<details><summary>Lời giải bài 1</summary>
<p><code>h = 1.000 m × 100 × 0,99 / 2,2229 ≈ 44.536 ms</code>, tức 44,5 s. Đi thật ở 28 km/h mất <code>1.000 / (28 / 3,6) ≈ 128,6 s</code> (nếu đường thẳng). Tỉ lệ khoảng 0,35: heuristic chỉ «biết» khoảng một phần ba chi phí thật, nên A* theo thời gian chỉ giảm 1,43–2,15 lần số nút. Với trọng số khoảng cách, tỉ lệ là 0,99 nên giảm 3,50–6,66 lần.</p>
</details>

**Bài 2.** Vì sao `Searcher::run` phải có dòng `if self.labels.is_closed(u) { continue; }` ngay sau khi lấy nút khỏi hàng đợi? Điều gì xảy ra nếu bỏ nó?

<details><summary>Lời giải bài 2</summary>
<p>Lab dùng xóa lười: khi khoảng cách của một nút giảm, mục mới được đẩy thêm và mục cũ ở lại trong <code>BinaryHeap</code>. Mục cũ có khóa lớn hơn nên đến sau, lúc nút đã chốt. Không bỏ qua nó thì nút bị chốt lần thứ hai (làm <code>settled</code> sai) và các cạnh của nó bị xét lại vô ích; kết quả vẫn đúng nhưng số nút đã duyệt, thứ mà bài này đo, không còn ý nghĩa.</p>
</details>

**Bài 3.** Đồng nghiệp đề xuất đặt `vmax` bằng 40 km/h thay vì lấy từ dữ liệu «cho A* nhanh hơn». Chuyện gì có thể sai, và kiểm tra tự động nào trong lab bắt được?

<details><summary>Lời giải bài 3</summary>
<p>Cạnh nào nhanh hơn 40 km/h (cao tốc 80, trục chính 50) sẽ có <code>h &gt; w</code>, heuristic không còn chặn dưới chi phí thật, nên A* có thể chốt một nút trước khi tìm được đường tối ưu tới nó và trả đường dài hơn Dijkstra mà không báo lỗi. Lab bắt được bằng các kiểm tra sẵn có: lab in «số cạnh vi phạm h &gt; w» trên mọi cạnh của đồ thị HCM, test <code>heuristic_never_exceeds_edge_cost_for_both_metrics</code> kiểm cùng điều đó trên fixture, và test <code>astar_equals_dijkstra_equals_sssp</code> so chi phí A* với Dijkstra trên các cặp ngẫu nhiên. Tất cả phải xanh trước khi đổi hằng số.</p>
</details>

Bài trước: [bài 20](/articles/location-lich-su-gps-parquet-retention-quyen-rieng-tu). Bài tiếp: [bài 22](/articles/location-bidirectional-alt-landmark-tang-toc-truy-van).

## Tài liệu gốc

- E. W. Dijkstra, «A note on two problems in connexion with graphs», Numerische Mathematik 1, 1959, trang 269–271: [doi.org/10.1007/BF01386390](https://doi.org/10.1007/BF01386390).
- P. E. Hart, N. J. Nilsson, B. Raphael, «A formal basis for the heuristic determination of minimum cost paths», IEEE Transactions on Systems Science and Cybernetics 4(2), 1968, trang 100–107: [doi.org/10.1109/TSSC.1968.300136](https://doi.org/10.1109/TSSC.1968.300136).
- [BinaryHeap trong thư viện chuẩn Rust](https://doc.rust-lang.org/std/collections/struct.BinaryHeap.html): max-heap, dùng `Reverse` để thành min-heap.
- [petgraph trên docs.rs](https://docs.rs/petgraph): `DiGraph`, `algo::dijkstra`, `algo::astar`.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho hình có dữ liệu OSM.
