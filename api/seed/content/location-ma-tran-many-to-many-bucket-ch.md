---
slug: location-ma-tran-many-to-many-bucket-ch
title: "Ma trận many-to-many bằng bucket trên CH: nền của ghép chuyến"
summary: "Bài 26: tính ma trận thời gian N×M bằng N+M tìm kiếm đi lên trên CH thay cho N×M truy vấn, kiểm đúng từng ô, đo rayon, ngân sách độ trễ, và nói rõ ma trận theo nút nên chưa có chi phí rẽ."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, routing, contraction-hierarchies, rayon, matrix, dispatch]
language: vi
---

Một lô ghép chuyến lúc cao điểm có thể gồm 100 khách đang chờ và 100 tài xế rảnh. Để chọn cặp tốt nhất, thuật toán ghép cần biết thời gian chạy từ từng tài xế tới từng điểm đón: 10.000 con số. Gọi 10.000 lần truy vấn tuyến đường thì mỗi lần đều đi lại những đoạn lên cao tốc, qua cầu mà 9.999 lần trước đã đi. Bài này tính cả bảng 10.000 ô một lần, trên chính đồ thị CH của bài trước, và đo xem nó nhanh hơn bao nhiêu mà vẫn ra đúng từng ô.

Đây là bài 26 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng CH của [bài 25](/articles/location-contraction-hierarchies-tu-cai-dat-fast-paths) và vị trí có hướng trên cạnh của [bài 23](/articles/location-chi-phi-re-u-turn-do-thi-theo-canh). Mọi số đo là trên đồ thị HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors), hồ sơ ô tô, trọng số thời gian, với tốc độ đường do lab giả định chứ không đo từ xe thật.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ma trận 100 nguồn × 100 đích có 10.000 ô. Theo bạn, tính nó bằng 10.000 truy vấn CH hai chiều chạy lặp thì nhanh hơn hay chậm hơn bao nhiêu lần so với cách chỉ chạy 100 + 100 tìm kiếm đi lên?</p>
<details><summary>Xem đáp án</summary><p>Ở lần chạy chính của lab, bucket một luồng tính 100×100 trong 2,40 ms, còn 10.000 truy vấn lặp mất 92,78 ms: chậm hơn <b>38,7 lần</b>. Qua ba lần chạy tỉ lệ là 32,5–38,7 lần. Ở 200×200 là 62,3–64,7 lần, còn ở 10×10 chỉ khoảng 2 lần. Lợi thế lớn dần theo kích thước vì N+M tìm kiếm tăng tuyến tính, còn N×M truy vấn tăng theo tích. Hai cách ra cùng từng ô.</p></details>
</div>

## Vì sao ghép chuyến cần một ma trận, không cần từng tuyến

Bài toán tham lam của baseline chỉ hỏi «tài xế nào gần điểm đón này nhất». Chim bay trả lời được câu đó rất rẻ, nhưng sai ở sông, cầu, đường một chiều (bài toán mở đầu của [bài 01](/articles/location-bai-toan-quang-duong-khach-va-tai-xe)). Muốn xếp hạng theo thời gian chạy thật, mỗi cặp (tài xế, điểm đón) cần một con số. Có ba tình huống dùng ma trận:

- **Xếp hạng điểm đón**: một khách, vài chục đến vài trăm ứng viên điểm hẹn, mỗi ứng viên cần thời gian từ tài xế tới đó (LOC-35).
- **Ghép theo lô**: N khách × M tài xế, đưa ma trận chi phí cho thuật toán phân công (LOC-42).
- **Tìm tài xế quanh một khách**: một nguồn, nhiều đích, dạng suy biến N = 1.

Vì vậy phần P04 (điểm đón) và P05 (ghép chuyến) đều đọc bài này trước. Một ma trận chậm chặn cả hai phần; một ma trận nhanh nhưng sai ô nào đó thì làm sai quyết định ghép mà không ai thấy lỗi. Hai yêu cầu của bài: **nhanh** và **đúng từng ô**.

## Ý tưởng bucket: N + M tìm kiếm thay cho N × M truy vấn

CH cho một tính chất: với mỗi cặp nút có một đường ngắn nhất dạng đi lên rồi đi xuống, nên từ hai đầu đi *lên* theo thứ hạng đều tới được nút hạng cao nhất của đường đó. Truy vấn hai chiều của bài 25 tận dụng điều này: một tìm kiếm thuận đi lên từ nguồn, một tìm kiếm ngược đi lên từ đích, rồi lấy nút gặp nhau cho tổng nhỏ nhất. Mỗi truy vấn chốt trung bình 172 nút trên cả hai chiều (số đo ở LOC-25), thay vì 56.142 nút của Dijkstra.

Điểm then chốt: tìm kiếm đi lên từ một đích **không phụ thuộc nguồn**. Nên với M đích ta chạy M tìm kiếm ngược đi lên đúng một lần, và ghi lại những gì chúng tìm thấy vào các «bucket» theo nút:

1. Với mỗi đích `Tⱼ`, chạy tìm kiếm ngược đi lên. Mỗi nút `v` được chốt với khoảng cách `d(v → Tⱼ)`; thêm cặp `(j, d)` vào bucket của `v`.
2. Với mỗi nguồn `Sᵢ`, chạy tìm kiếm thuận đi lên. Mỗi nút `v` được chốt với `d(Sᵢ → v)`; quét bucket của `v`, và với mỗi `(j, d)` cập nhật `ô(i, j) = min(ô(i, j), d(Sᵢ → v) + d)`.

Tổng cộng N + M tìm kiếm nhỏ. Công việc quét bucket tỉ lệ với số lần một cặp (nguồn, đích) cùng chốt một nút: mỗi cặp ít nhất một lần nếu có đường, thường vài lần. Phần này vẫn tăng theo N×M, nhưng là phép cộng-so sánh trên mảng liên tục, không phải truy vấn đồ thị. Tính đúng theo cùng lý lẽ với CH hai chiều: với mỗi cặp, nút hạng cao nhất của đường ngắn nhất nằm trong cả hai tập nút đã chốt, và các nút gặp nhau khác chỉ cho tổng không nhỏ hơn.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 1. Cơ chế bucket: hai nguồn S1, S2 đi lên qua nút u; hai đích T1, T2 đi lên qua nút w; hai nhánh gặp nhau ở nút hạng cao v, nơi bucket lưu khoảng cách từ v tới từng đích và mỗi nguồn quét bucket đó để điền ô của mình."><defs><marker id="loc26-f1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="loc26-f1-b" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="840" height="310" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Minh hoạ cơ chế, đồ thị tự dựng (không phải dữ liệu OSM): N + M tìm kiếm đi lên</text><text x="40" y="118" class="t-small">hạng cao</text><text x="40" y="268" class="t-small">hạng thấp</text><path d="M137 251 L193 183" class="f-none s-pk" stroke-width="2" marker-end="url(#loc26-f1-a)"/><path d="M263 251 L207 183" class="f-none s-pk" stroke-width="2" marker-end="url(#loc26-f1-a)"/><path d="M209 170 L430 118" class="f-none s-pk" stroke-width="2" marker-end="url(#loc26-f1-a)"/><path d="M613 251 L669 183" class="f-none s-phys" stroke-width="2" marker-end="url(#loc26-f1-b)"/><path d="M747 251 L691 183" class="f-none s-phys" stroke-width="2" marker-end="url(#loc26-f1-b)"/><path d="M671 170 L450 118" class="f-none s-phys" stroke-width="2" marker-end="url(#loc26-f1-b)"/><circle cx="130" cy="260" r="9" class="f-pk s-pk" stroke-width="1.5"/><circle cx="270" cy="260" r="9" class="f-pk s-pk" stroke-width="1.5"/><circle cx="610" cy="260" r="9" class="f-phys s-phys" stroke-width="1.5"/><circle cx="750" cy="260" r="9" class="f-phys s-phys" stroke-width="1.5"/><circle cx="200" cy="176" r="7" class="f-pk s-pk" stroke-width="1.5"/><circle cx="680" cy="176" r="7" class="f-phys s-phys" stroke-width="1.5"/><circle cx="440" cy="112" r="10" class="f-warn s-warn" stroke-width="1.5"/><text x="130" y="288" class="t-b" text-anchor="middle">S1</text><text x="270" y="288" class="t-b" text-anchor="middle">S2</text><text x="610" y="288" class="t-b" text-anchor="middle">T1</text><text x="750" y="288" class="t-b" text-anchor="middle">T2</text><text x="440" y="88" class="t-b" text-anchor="middle">nút v</text><path d="M449 106 L500 96" class="f-none s-warn dash" stroke-width="1.5"/><rect x="500" y="58" width="330" height="62" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="512" y="80" class="t-mono">bucket[v] = { (T1, d1), (T2, d2) }</text><text x="512" y="104" class="t-small">S1 chốt v: ô(S1,Tj) = min(d(S1,v) + dj)</text><text x="40" y="312" class="t-small">mũi tên xanh dương: tìm kiếm thuận đi lên từ mỗi nguồn</text><text x="460" y="312" class="t-small">mũi tên xanh lục: tìm kiếm ngược đi lên từ mỗi đích</text></svg>
</div>
<figcaption>Hình 1. Mỗi đích chạy một tìm kiếm ngược đi lên và để lại khoảng cách vào bucket của từng nút đã chốt; mỗi nguồn chạy một tìm kiếm thuận đi lên và quét bucket ở từng nút đã chốt. Hình minh hoạ tự dựng.</figcaption>
</figure>

## Cài đặt: tìm kiếm đi lên đầy đủ và bucket dạng CSR

Hàm `upward_search` là tìm kiếm Dijkstra trên đồ thị «chỉ đi lên» của CH, **không dừng sớm** (khác truy vấn hai chiều), và gọi `visit(hạng, khoảng cách)` ở mỗi nút chốt. Nguồn và đích là danh sách *chân* (`Leg`): nút kèm chi phí đi nốt phần cạnh, nên điểm nằm giữa một cạnh vẫn dùng được.

`examples/toi-noi-location/rust/crates/route-engine/src/matrix.rs`

```rust
fn upward_search(
    ch: &Ch,
    forward: bool,
    legs: &[Leg],
    ws: &mut UpWorkspace,
    mut visit: impl FnMut(u32, u32),
) {
    let g = if forward { &ch.fwd } else { &ch.bwd };
    ws.labels.reset();
    ws.heap.clear();
    for l in legs {
        let r = ch.new_id[l.node as usize];
        if l.cost < ws.labels.dist(r) {
            ws.labels.set(r, l.cost, NONE);
            ws.heap.push(l.cost, r);
        }
    }
    while let Some((k, u)) = ws.heap.pop() {
        if ws.labels.is_closed(u) || k != ws.labels.dist(u) {
            continue;
        }
        ws.labels.close(u);
        visit(u, k);
        for a in g.arcs(u) {
            let nd = k.saturating_add(g.w[a]);
            let v = g.head[a];
            if nd < ws.labels.dist(v) {
                ws.labels.set(v, nd, u);
                ws.heap.push(nd, v);
            }
        }
    }
}
```

Bucket được xếp thành CSR: `first[node]` cho biết đoạn của `entries` thuộc nút đó, mỗi phần tử là `(chỉ số đích, khoảng cách)`. Dựng hai lượt (đếm rồi điền) cho một mảng liên tục, quét nhanh và không cấp phát từng bucket.

`examples/toi-noi-location/rust/crates/route-engine/src/matrix.rs`

```rust
    let mut first = vec![0u32; n + 1];
    for v in &per_target {
        for &(node, _) in v {
            first[node as usize + 1] += 1;
        }
    }
    for i in 0..n {
        first[i + 1] += first[i];
    }
    let mut fill = first.clone();
    let mut entries = vec![(0u32, 0u32); first[n] as usize];
    for (j, v) in per_target.iter().enumerate() {
        for &(node, d) in v {
            entries[fill[node as usize] as usize] = (j as u32, d);
            fill[node as usize] += 1;
        }
    }
```

Mỗi hàng của ma trận là một tìm kiếm thuận đi lên; ở mỗi nút chốt nó quét bucket và hạ ô nào có tổng nhỏ hơn.

`examples/toi-noi-location/rust/crates/route-engine/src/matrix.rs`

```rust
    let row = |i: usize, ws: &mut UpWorkspace| -> Vec<u32> {
        let mut out = vec![INF; cols];
        upward_search(ch, true, &sources[i], ws, |node, d| {
            let s =
                buckets.first[node as usize] as usize..buckets.first[node as usize + 1] as usize;
            for &(j, dt) in &buckets.entries[s] {
                let c = d.saturating_add(dt);
                if c < out[j as usize] {
                    out[j as usize] = c;
                }
            }
        });
        out
    };
```

`INF` là «không tới được». Các hàng độc lập nhau nên chạy song song được; đó là chỗ `rayon` vào cuộc ở mục sau.

## Đúng từng ô: bốn cách tính phải ra cùng một bảng

Lab tính mỗi ma trận bằng bốn cách: bucket một luồng, bucket `rayon`, N×M truy vấn CH hai chiều lặp (một luồng và `rayon`). Cột «khác nhau» đếm số ô khác với bucket một luồng, cộng ba cách còn lại. Ngoài ra ma trận 20×20 được đối chiếu với 400 truy vấn A* độc lập và với 20 lần Dijkstra một-nguồn-tới-tất-cả. Cả ba lần chạy đều cho **0 ô khác** ở mọi cỡ, và `khác A* 0, khác sssp 0`.

Test đơn vị `bucket_matrix_equals_repeated_queries_and_dijkstra` của `matrix.rs` làm việc tương tự trên đồ thị tổng hợp nhỏ, và `legs_with_offsets_add_to_the_cost` kiểm rằng chân có chi phí lệch được cộng đúng (1.000 ở nguồn cộng 250 ở đích ra đúng 1.250 ms thêm).

## Số đo: lợi thế lớn dần theo kích thước

Cấu hình: Apple M1 Max, `--release`, HCM, hồ sơ ô tô, CH nạp từ cache, nguồn và đích là nút ngẫu nhiên có seed 20261106 trong thành phần liên thông mạnh lớn nhất, một lần chạy cho mỗi cỡ. **Đo khi máy đang chịu tải nặng** (tải 1 phút khoảng 20 trên 10 lõi), nên đọc tỉ lệ trong cùng một dòng, không đọc ms tuyệt đối.

| N×M | bucket 1 luồng (ms) | bucket rayon (ms) | N×M truy vấn (ms) | N×M truy vấn rayon (ms) | lặp / bucket |
|---|---:|---:|---:|---:|---:|
| 10×10 | 0,46 | 1,09 | 0,96 | 0,68 | 2,1x |
| 25×25 | 0,79 | 1,51 | 5,84 | 2,05 | 7,4x |
| 50×50 | 1,35 | 2,33 | 22,72 | 7,80 | 16,8x |
| 100×100 | 2,40 | 6,84 | 92,78 | 19,24 | 38,7x |
| 200×200 | 6,34 | 6,22 | 410,07 | 68,28 | 64,7x |

Hình 2 vẽ cột cuối. Số liệu ở cột này là *một* lần chạy; qua ba lần chạy 100×100 nằm trong 32,5–38,7 lần và 200×200 trong 62,3–64,7 lần, còn 10×10 chỉ 2,1 lần ở lần chính.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 2. Tỉ lệ thời gian N×M truy vấn chạy lặp so với bucket một luồng: 10×10 là 2,1 lần, 25×25 là 7,4 lần, 50×50 là 16,8 lần, 100×100 là 38,7 lần, 200×200 là 64,7 lần."><rect x="20" y="20" width="840" height="250" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Chạy lặp N×M truy vấn chậm hơn bucket bao nhiêu lần (cùng một lần chạy, một luồng)</text><text x="40" y="84" class="t-mono">10×10</text><rect x="130" y="70" width="16" height="20" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="156" y="84" class="t-mono">2,1x   (0,46 ms so với 0,96 ms)</text><text x="40" y="124" class="t-mono">25×25</text><rect x="130" y="110" width="55" height="20" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="195" y="124" class="t-mono">7,4x   (0,79 ms so với 5,84 ms)</text><text x="40" y="164" class="t-mono">50×50</text><rect x="130" y="150" width="125" height="20" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="265" y="164" class="t-mono">16,8x   (1,35 ms so với 22,72 ms)</text><text x="40" y="204" class="t-mono">100×100</text><rect x="130" y="190" width="288" height="20" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="428" y="204" class="t-mono">38,7x   (2,40 ms so với 92,78 ms)</text><text x="40" y="244" class="t-mono">200×200</text><rect x="130" y="230" width="480" height="20" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="620" y="244" class="t-mono">64,7x   (6,34 ms so với 410,07 ms)</text></svg>
</div>
<figcaption>Hình 2. Tỉ lệ «lặp / bucket» của lần chạy chính, độ dài cột tỉ lệ với số lần. Đo trên HCM, ô tô, khi máy chịu tải nặng; chỉ tỉ lệ trong cùng một lần chạy là đáng đọc. Hai cách ra cùng từng ô (0 ô khác).</figcaption>
</figure>

Hai điều rút ra. Thứ nhất, với 10×10 lợi thế chỉ cỡ 2 lần vì chi phí cố định của mỗi tìm kiếm đi lên chiếm hết phần tiết kiệm (cách giải thích ghi trong `results/p03.md`; lab chưa đo riêng chi phí này); bucket đáng giá từ cỡ vài chục nguồn trở lên. Thứ hai, ở 200×200 mỗi ô tốn 0,16 µs (lần chính), vì chi phí tìm kiếm chỉ còn N + M lần, chia cho N×M ô, cộng phần quét bucket. So với đối chứng ngoài CH: 400 truy vấn A* cho ma trận 20×20 mất 804,7 ms trong cùng lần chạy.

## rayon: song song hoá ở đâu thì có ích

Cả `build_buckets` lẫn các hàng đều viết sẵn nhánh `parallel` bằng `into_par_iter().map_init(...)`, mỗi luồng giữ một `UpWorkspace` riêng để không cấp phát lại. Nhưng số đo cho thấy song song hoá **bên trong một ma trận nhỏ không có lợi**: ở 100×100, bucket `rayon` mất 6,84 ms so với 2,40 ms một luồng (3,96–6,84 ms qua ba lần chạy so với 2,40–2,81 ms). Mỗi tìm kiếm đi lên chỉ cỡ vài chục µs, nên chi phí điều phối và tranh lõi (máy đang tải) lớn hơn phần việc được chia. Ở 200×200 hai cách gần ngang nhau.

`rayon` chỉ thắng rõ ở hai chỗ: khi chạy lặp N×M truy vấn (100×100: 19,24 ms thay vì 92,78 ms), và khi ma trận đủ lớn. Bảng ngân sách độ trễ dưới đây cho thấy điểm chuyển ở quanh 300×300.

## Ngân sách độ trễ theo hình dạng yêu cầu

Mỗi hình dạng được đo 15 lần với mẫu nút mới mỗi lần, ghi p50 và p95 (ms), lần chạy chính, vẫn khi máy chịu tải:

| Hình dạng | p50 1 luồng | p95 1 luồng | p50 rayon | p95 rayon |
|---|---:|---:|---:|---:|
| 1 khách × 50 tài xế | 0,91 | 2,17 | 5,65 | 26,02 |
| 1 khách × 200 ứng viên điểm đón | 2,26 | 13,62 | 21,40 | 50,31 |
| 50 tài xế × 50 khách | 1,52 | 3,16 | 3,86 | 14,65 |
| 100 × 100 | 2,78 | 4,49 | 5,37 | 14,26 |
| 300 × 300 | 11,20 | 13,02 | 8,96 | 15,15 |
| 500 × 500 | 24,80 | 37,93 | 18,07 | 32,23 |

Kết luận thực dụng: với mọi cỡ không quá 200×200, **dùng một luồng cho mỗi ma trận và song song hoá ở mức nhiều yêu cầu** (bài 29 làm việc đó ở tầng dịch vụ); `rayon` trong một ma trận chỉ đáng bật từ khoảng 300×300 trở lên. Một lô ghép cỡ vài trăm trên vài trăm nằm ở mức vài chục ms ở p50 trên máy này, nhưng p95 của lô lớn dao động nhiều theo tải, nên đừng chốt ngân sách p99 từ các số này: LOC-35 sẽ đặt và đo ngân sách đó trên đường chạy thật.

## Giới hạn quan trọng nhất: ma trận này không có chi phí rẽ

CH của bài 25 dựng trên đồ thị **theo nút**: chi phí nằm ở cạnh, đi từ cạnh này sang cạnh kia luôn miễn phí. Vì ma trận bucket nằm trên CH đó, nó thừa hưởng hạn chế này. Cụ thể, ma trận **không có** chi phí rẽ trái, rẽ phải, **không có** phạt quay đầu, và **không áp** cấm rẽ của OSM (kể cả phần cấm rẽ đã đọc được, vốn chỉ được áp trong A* theo cạnh của bài 23). Bản thân phần cấm rẽ đọc được cũng chưa đủ: trong bbox HCM, 784 trên 1.262 quan hệ cấm rẽ có điều kiện thời gian và bị bỏ qua, 112 quan hệ có `via` là way cũng bỏ qua (bài 23).

Hệ quả cho Tới Nơi cụ thể: một tài xế đang chạy ngược chiều so với điểm đón sẽ thấy ma trận báo thời gian *quá lạc quan*, vì nó không tính việc phải quay đầu hay vòng qua dải phân cách. Lab có một phép đo nhỏ cho đúng phần này. Với 100 nguồn × 86 đích đường hai chiều, đi tới điểm đúng phía bất lợi tốn thêm so với «phía nào cũng được» trong 89,5% số ô: trung bình 10,9 s, p95 32,0 s, lớn nhất 115,0 s. Con số này mới chỉ là chi phí đi vòng để vào đúng chiều cạnh; **chưa tính chi phí quay đầu**.

Vậy các bài sau phải làm gì? Đây là việc các bài sau phải làm, chưa phải kết quả đã đo:

- **Dùng ma trận để sàng, không để chốt.** Ma trận trả lời nhanh «ứng viên nào đáng xét», sau đó tính lại chính xác cho vài ứng viên đầu (top-k) bằng tìm kiếm theo cạnh có chi phí rẽ của bài 23 (`EdgeSearcher` với đích `Goal::Pos`, vị trí trên cạnh có hướng). Chính simulator của bài 30 đã tính quãng tài xế chạy bằng bộ định tuyến theo cạnh có chi phí rẽ; chỉ phần ghép chuyến baseline là dùng chim bay.
- **Đừng tin thứ hạng gần nhau.** Hai ứng viên chênh nhau ít hơn sai số do thiếu chi phí rẽ có thể bị xếp ngược; vì vậy lấy k lớn hơn 1 rồi chấm lại.
- **Đích đúng phía đường** (LOC-34) phải thêm chi phí rẽ và quay đầu lên trên cấu trúc `Leg` đã có, nên ma trận hiện tại là nền chứ chưa phải đáp án.
- **Đo lại bằng simulator.** Bất cứ tuyên bố «xếp hạng bằng ma trận tốt hơn gần nhất» phải kèm số đo so với baseline của bài 30, không suy từ ms ở bài này.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` do lab bài 06 ghi (hoặc thêm `--fixture` cho đồ thị tổng hợp). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc26_ma_tran
```

Output (rút gọn), ba lần chạy cho các số đếm giống hệt nhau, **đo khi máy chịu tải nặng**:

```text
  N×M        bucket 1 luồng  bucket rayon  N×M truy vấn N×M truy vấn rayon µs/ô bucket lặp/bucket khác nhau
  100×100               2.40          6.84         92.78          19.24        0.24     38.7x         0
  200×200               6.34          6.22        410.07          68.28        0.16     64.7x         0
  cột «khác nhau»: số ô khác với bucket 1 luồng, cộng ba cách còn lại; tổng = 0
  400 ô: khác A* 0, khác sssp 0
  100 nguồn × 86 đích hai chiều: «hai phía» = min(phía 0, phía 1) đúng ở mọi ô: 0 ô vi phạm trên 8600
  đi tới điểm đúng phía bất lợi ... tốn thêm so với «phía nào cũng được»: trong 89.5% ô; trung bình 10.9 s, p95 32.0 s, lớn nhất 115.0 s
```

Số ô khác, số ô vi phạm và tỉ lệ 89,5% tái lập từng chữ số với cùng seed; các số ms thì không. `bash examples/toi-noi-location/verify-p03.sh` chạy cả test của ma trận trên đồ thị tổng hợp mà không cần bản đồ thật.

## Giới hạn của ví dụ

- **Không có chi phí rẽ và cấm rẽ**: xem mục trên; đây là hạn chế lớn nhất và đã được nêu ở `results/p03.md`.
- **Nguồn và đích là nút ngẫu nhiên đều** trong thành phần liên thông lớn nhất. Tài xế thật tụ ở điểm nóng; phân bố đó có thể làm tìm kiếm đi lên chốt nhiều nút hơn hoặc ít hơn, chưa đo.
- **Một lần chạy cho mỗi cỡ ở bảng 1**, ba lần chạy tổng cộng; chưa có lần đo trên máy rảnh. Chỉ tỉ lệ trong cùng một lần chạy là đáng đọc.
- **CH nạp từ cache** (thời gian dựng gốc 1,94 s), nên bài này không đo thời gian dựng; bài 27 hỏi khi nào phải dựng lại.
- **Đi bộ không có CH**: hồ sơ đi bộ dùng A* cho ma trận, không dùng bucket.
- **Chưa có ma trận thưa hay cắt tỉa**: lab tính ô đủ, không bỏ ô xa; việc cắt tỉa theo H3 là bài của P05.

## Bài tập

**Bài 1.** Với 1 nguồn × M đích, bucket có còn lợi so với M truy vấn CH hai chiều không? Giải thích bằng số tìm kiếm cần chạy.

<details><summary>Lời giải bài 1</summary>
<p>M truy vấn hai chiều chạy M tìm kiếm thuận và M tìm kiếm ngược (mỗi truy vấn một cặp). Bucket chạy M tìm kiếm ngược để dựng bucket và đúng một tìm kiếm thuận cho nguồn duy nhất, tức là M + 1 tìm kiếm thay cho 2M. Theo số tìm kiếm thì lợi thế chỉ khoảng hai lần, và còn phải cộng chi phí dựng bucket; lab chưa đo riêng so sánh này ở hình dạng 1 × M (riêng bucket 1 khách × 50 tài xế mất 0,91 ms ở p50). Lợi thế lớn xuất hiện khi cả N và M đều lớn, vì tìm kiếm được dùng lại cho cả hàng và cột.</p>
</details>

**Bài 2.** Vì sao `upward_search` không dừng khi chốt được nút đích như truy vấn hai chiều?

<details><summary>Lời giải bài 2</summary>
<p>Ở truy vấn hai chiều, ta biết đúng một cặp (nguồn, đích) nên có thể dừng khi hai bên không còn cải thiện được. Với ma trận, tìm kiếm của một nguồn phải gặp bucket của mọi đích, và tìm kiếm của một đích phải ghi vào bucket của mọi nút nó chốt; ta không biết trước nút nào là nút gặp nhau cho cặp nào. Vì vậy mỗi tìm kiếm đi lên chạy cạn không gian đi lên của nó (nhỏ nhờ CH), rồi mọi nút gặp nhau đều được xét.</p>
</details>

**Bài 3.** Một nhà phát triển muốn dùng ma trận này để quyết định «tài xế nào tới điểm đón trong thời gian ngắn nhất» cho cả lô và chốt luôn. Chỉ ra hai loại sai lệch có thể xảy ra, và cách giảm.

<details><summary>Lời giải bài 3</summary>
<p>Loại một: ma trận theo nút không có chi phí rẽ, quay đầu, cấm rẽ, nên tài xế phải vòng qua dải phân cách hoặc quay đầu sẽ bị đánh giá quá nhanh (lab đo đi vào đúng phía bất lợi tốn thêm trung bình 10,9 s, trong 89,5% số ô có tốn thêm, chưa tính quay đầu). Loại hai: hai ứng viên chênh nhau ít có thể bị xếp ngược. Cách giảm: lấy top-k theo ma trận rồi chấm lại bằng tìm kiếm theo cạnh có chi phí rẽ, và đo tác động bằng simulator so với baseline.</p>
</details>

Bài trước: [bài 25](/articles/location-contraction-hierarchies-tu-cai-dat-fast-paths). Bài tiếp: [bài 27](/articles/location-toc-do-theo-gio-customizable-ch-ket-xe).

## Tài liệu gốc

- S. Knopp, P. Sanders, D. Schultes, F. Schulz, D. Wagner, «Computing Many-to-Many Shortest Paths Using Highway Hierarchies», ALENEX 2007: [trang bài báo ở KIT](https://ae.iti.kit.edu/english/1044.php). Ý tưởng bảng khoảng cách dựa trên phân cấp; lab dùng cùng ý tưởng bucket trên CH.
- R. Geisberger, P. Sanders, D. Schultes, D. Delling, «Contraction Hierarchies: Faster and Simpler Hierarchical Routing in Road Networks», WEA 2008: [trang bài báo ở KIT](https://ae.iti.kit.edu/999.php).
- [rayon trên docs.rs](https://docs.rs/rayon): `into_par_iter`, `map_init`.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho dữ liệu OSM.
