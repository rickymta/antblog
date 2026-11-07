---
slug: location-nhieu-diem-dung-chen-2-opt-doi-chieu-ortools
title: "Nhiều điểm dừng: chèn và 2-opt trong Rust, đối chiếu OR-Tools, tổng hợp P05"
summary: "Bài 50: một người giao hàng, n cặp lấy trước giao sau; chèn rẻ nhất và 2-opt/Or-opt trong Rust cách OR-Tools 0,39 đến 2,77% ở cực tiểu cục bộ. Thời gian, ngân sách, giới hạn của mốc so sánh và tổng hợp P05."
category: location-routing
level: advanced
tags: [toi-noi, location, rust, python, ortools, 2-opt, insertion, pickup-delivery, vrp]
language: vi
---

Một tài xế giao hàng nhận bốn gói trong cùng một lượt. Mỗi gói có điểm lấy và điểm giao, và gói nào cũng phải **lấy trước rồi mới giao**. Chín điểm cần sắp thứ tự, kể cả điểm xuất phát. Với bốn gói, thử hết các thứ tự hợp lệ còn làm được; với ba mươi gói thì không. Loạt bài này cần một thứ trung gian: đủ nhanh để chạy trên đường nóng khi tài xế vừa nhận thêm gói, đủ tốt để không bỏ phí quãng đường, và có một mốc để biết «đủ tốt» là bao nhiêu.

Bài này dựng thứ trung gian đó trong Rust (chèn rẻ nhất, rồi 2-opt, Or-opt và dời cặp), đặt cạnh nó hai mốc: quy hoạch động chính xác cho cỡ nhỏ, và OR-Tools chạy offline bằng Python cho mọi cỡ. Đây cũng là bài chốt phần P05, nên cuối bài có bảng tổng hợp cả mười bài.

Đây là bài 50 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng ma trận ETA bất đối xứng của [bài 26](/articles/location-ma-tran-many-to-many-bucket-ch) và các điểm lấy, giao do simulator của [bài 30](/articles/location-tong-hop-mo-phong-su-kien-roi-rac-seed) sinh ra, nên mọi số là **trên dữ liệu mô phỏng**: đồ thị HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors), seed 50001…50020. Cách so sánh có cặp ở đây khác bài 41 một chút: cặp là **instance**, và gap tính trên từng instance.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một instance 4 cặp (9 đỉnh, seed 50001). Chèn rẻ nhất rồi 2-opt, Or-opt và dời cặp dừng ở một tuyến mà không phép dời nào cải thiện thêm được. Theo bạn tuyến tối ưu thật (quy hoạch động) rẻ hơn tuyến đó bao nhiêu phần trăm?</p>
<details><summary>Xem đáp án</summary><p>Tuyến heuristic tốn 4.948.489 ms, tuyến tối ưu 4.745.839 ms: heuristic đắt hơn <b>4,27%</b>, và đây cũng là instance tệ nhất trong 20 instance cỡ 4 cặp. Trung bình 20 instance, gap chỉ +0,39%, và OR-Tools trùng với tối ưu ở mọi instance cỡ 4 và 8.</p></details>
</div>

## Bài toán và ma trận chi phí

Một người giao hàng, `n` cặp (lấy → giao), lấy trước giao, **tuyến mở** (không về điểm xuất phát), tối thiểu tổng thời gian chạy ô tô. Lab **không** có tải trọng, khung giờ hay nhiều xe: chỉ có ràng buộc thứ tự. Đỉnh 0 là điểm xuất phát, đỉnh `1 + 2k` là điểm lấy của cặp `k`, đỉnh `2 + 2k` là điểm giao.

Ma trận chi phí `(2n + 1)²` lấy từ CH (bài 26), tính bằng mili giây nguyên, và **bất đối xứng** vì có đường một chiều (ma trận CH không có chi phí rẽ). Điều này quan trọng cho 2-opt ở dưới. Instance sinh từ thế giới mô phỏng của từng seed: điểm lấy và giao là ghim và điểm trả của `n` khách đầu tiên, điểm xuất phát là tài xế số 0. Năm cỡ n ∈ {4, 8, 12, 20, 30}, 20 instance mỗi cỡ, tổng 100 instance; sinh cả 100 instance cùng ma trận mất 1,5 s.

## Chèn rẻ nhất

Heuristic chèn đi từ tuyến chỉ có điểm xuất phát. Mỗi bước, với mỗi cặp chưa vào tuyến, thử **mọi vị trí lấy rồi mọi vị trí giao sau nó** và tính chi phí tăng thêm; chọn cặp rẻ nhất rồi chèn đúng chỗ đó. Chi phí tăng khi chèn đỉnh `x` giữa `a` và `b` là `c(a,x) + c(x,b) − c(a,b)`, tính trong O(1):

`examples/toi-noi-location/rust/crates/matcher/src/multistop.rs`

```rust
/// Vị trí chèn tốt nhất của cặp `k` vào `route` (không chứa cặp đó): `(delta, i, m)`.
/// Điểm lấy chèn ngay sau `route[i]`; điểm giao chèn ngay sau điểm lấy nếu `m == i`, ngược lại ngay sau `route[m]` (`m > i`).
fn best_pair_insertion(inst: &Instance, route: &[usize], k: usize) -> (i64, usize, usize) {
    let (p, d) = (Instance::pickup(k), Instance::delivery(k));
    let l = route.len();
    let mut best = (i64::MAX, 0, 0);
    for i in 0..l {
        let a = route[i];
        let b = route.get(i + 1).copied();
        let pick = ins(inst, a, p, b);
        // giao liền sau điểm lấy: a → p → d → b
        let direct = match b {
            Some(b) => inst.c(a, p) + inst.c(p, d) + inst.c(d, b) - inst.c(a, b),
            None => inst.c(a, p) + inst.c(p, d),
        };
        if direct < best.0 {
            best = (direct, i, i);
        }
        for m in (i + 1)..l {
            let delta = pick + ins(inst, route[m], d, route.get(m + 1).copied());
            if delta < best.0 {
                best = (delta, i, m);
            }
        }
    }
    best
}
```

Độ phức tạp là `O(n² · L²)` với `L = 2n + 1`, và mọi tuyến sinh ra đều hợp lệ vì điểm giao luôn chèn sau điểm lấy. Chèn rất rẻ nhưng tham lam: quyết định ở bước đầu không bao giờ được xét lại.

## 2-opt, Or-opt và dời cặp dưới ràng buộc thứ tự

Tìm kiếm cục bộ sửa những quyết định tham lam đó, nhưng ràng buộc thứ tự làm các phép dời quen thuộc phải cẩn thận. 2-opt đảo một đoạn `route[i..=j]`. **Đảo đoạn lật thứ tự mọi cặp nằm trọn trong đoạn**, nên đoạn chứa cả điểm lấy lẫn điểm giao của một cặp là không hợp lệ. Vì ma trận bất đối xứng, đảo đoạn còn đổi chi phí từng cạnh bên trong; code giữ hai tổng chạy (`fwd` theo chiều cũ, `bwd` theo chiều đảo) để mỗi `j` tính delta trong O(1):

`examples/toi-noi-location/rust/crates/matcher/src/multistop.rs`

```rust
/// 2-opt tốt nhất: đảo đoạn `route[i..=j]`. Đảo làm lật thứ tự mọi cặp nằm trọn trong đoạn nên đoạn chứa cả điểm lấy lẫn
/// điểm giao của một cặp là không hợp lệ; với `i` cố định, `j` tăng dần thì gặp điểm giao có điểm lấy đã trong đoạn là dừng.
fn best_two_opt(inst: &Instance, r: &[usize]) -> Option<(i64, Move)> {
    let l = r.len();
    let mut best: Option<(i64, Move)> = None;
    let mut in_seg = vec![false; inst.nodes];
    for i in 1..l.saturating_sub(1) {
        in_seg.iter_mut().for_each(|x| *x = false);
        let (mut fwd, mut bwd) = (0i64, 0i64);
        in_seg[r[i]] = true;
        for j in (i + 1)..l {
            let v = r[j];
            if !Instance::is_pickup(v) && in_seg[Instance::partner(v)] {
                break;
            }
            in_seg[v] = true;
            fwd += inst.c(r[j - 1], v);
            bwd += inst.c(v, r[j - 1]);
            let prev = r[i - 1];
            let next = r.get(j + 1).copied();
            let old = inst.c(prev, r[i]) + fwd + next.map_or(0, |n| inst.c(v, n));
            let new = inst.c(prev, v) + bwd + next.map_or(0, |n| inst.c(r[i], n));
            let delta = new - old;
            if delta < 0 && best.is_none_or(|b| delta < b.0) {
                best = Some((delta, Move::TwoOpt { i, j }));
            }
        }
    }
    best
}
```

Hàm này dừng mở rộng `j` ngay khi gặp một điểm giao mà điểm lấy của nó đã nằm trong đoạn. Hai phép dời còn lại cùng cơ chế: **Or-opt** dời một đoạn 1 đến 3 đỉnh liên tiếp (giữ chiều) tới chỗ khác, kèm kiểm tra thứ tự lấy trước giao còn đúng; **dời cặp** gỡ cả hai đỉnh của một cặp rồi chèn lại ở chỗ tốt nhất. Tìm kiếm cục bộ chạy theo thứ tự đó, mỗi lần áp dụng bước tốt nhất của loại đầu tiên có cải thiện, và dừng khi không loại nào cải thiện (hoặc hết hạn):

`examples/toi-noi-location/rust/crates/matcher/src/multistop.rs`

```rust
/// Tìm kiếm cục bộ từ `start` (tuyến hợp lệ): lần lượt 2-opt, Or-opt, dời cặp; áp dụng bước **tốt nhất** của loại đầu tiên
/// có cải thiện rồi quay lại từ đầu, tới khi không loại nào cải thiện (hoặc hết hạn).
pub fn local_search(
    inst: &Instance,
    start: Vec<usize>,
    deadline: Option<Instant>,
) -> (Vec<usize>, usize, bool) {
    let mut r = start;
    let mut moves = 0;
    loop {
        if deadline.is_some_and(|d| Instant::now() >= d) {
            return (r, moves, true);
        }
        let step = best_two_opt(inst, &r)
            .or_else(|| best_or_opt(inst, &r))
            .or_else(|| best_pair_move(inst, &r));
        match step {
            Some((_, mv)) => {
                r = apply_move(&r, mv);
                moves += 1;
            }
            None => return (r, moves, false),
        }
    }
}
```

Mọi bước chỉ được nhận khi **giảm** tổng chi phí, nên vòng lặp luôn dừng. Điểm dừng là một cực tiểu cục bộ, và hình dưới là một ví dụ thật của nó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 1. Instance n04-00 (seed 50001). Chèn rẻ nhất cho tuyến ở hàng trên và tìm kiếm cục bộ không cải thiện thêm được; tuyến tối ưu (DP, trùng OR-Tools) ở hàng dưới gom L2 L3 trước. Chênh lệch +4,27%. Chi phí lấy nguyên văn từ output của lab."><rect x="10" y="10" width="860" height="250" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="30" y="38" class="t-b">Instance n04-00 (seed 50001): L = lấy, G = giao, S = xuất phát, tuyến mở</text><text x="30" y="82">chèn rẻ nhất,</text><text x="30" y="98" class="t-small">cả khi thêm LS</text><rect x="200" y="70" width="56" height="30" rx="6" class="f-surface s-muted" stroke-width="1.5"/><text x="228.0" y="90" class="t-mono" text-anchor="middle">S</text><rect x="262" y="70" width="56" height="30" rx="6" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="290.0" y="90" class="t-mono" text-anchor="middle">L0</text><rect x="324" y="70" width="56" height="30" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="352.0" y="90" class="t-mono" text-anchor="middle">G0</text><rect x="386" y="70" width="56" height="30" rx="6" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="414.0" y="90" class="t-mono" text-anchor="middle">L1</text><rect x="448" y="70" width="56" height="30" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="476.0" y="90" class="t-mono" text-anchor="middle">G1</text><rect x="510" y="70" width="56" height="30" rx="6" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="538.0" y="90" class="t-mono" text-anchor="middle">L2</text><rect x="572" y="70" width="56" height="30" rx="6" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="600.0" y="90" class="t-mono" text-anchor="middle">L3</text><rect x="634" y="70" width="56" height="30" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="662.0" y="90" class="t-mono" text-anchor="middle">G3</text><rect x="696" y="70" width="56" height="30" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="724.0" y="90" class="t-mono" text-anchor="middle">G2</text><text x="775" y="90" class="t-mono">4.948.489 ms</text><text x="30" y="152">DP tối ưu,</text><text x="30" y="168" class="t-small">OR-Tools trùng</text><rect x="200" y="140" width="56" height="30" rx="6" class="f-surface s-muted" stroke-width="1.5"/><text x="228.0" y="160" class="t-mono" text-anchor="middle">S</text><rect x="262" y="140" width="56" height="30" rx="6" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="290.0" y="160" class="t-mono" text-anchor="middle">L2</text><rect x="324" y="140" width="56" height="30" rx="6" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="352.0" y="160" class="t-mono" text-anchor="middle">L3</text><rect x="386" y="140" width="56" height="30" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="414.0" y="160" class="t-mono" text-anchor="middle">G2</text><rect x="448" y="140" width="56" height="30" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="476.0" y="160" class="t-mono" text-anchor="middle">G3</text><rect x="510" y="140" width="56" height="30" rx="6" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="538.0" y="160" class="t-mono" text-anchor="middle">L0</text><rect x="572" y="140" width="56" height="30" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="600.0" y="160" class="t-mono" text-anchor="middle">G0</text><rect x="634" y="140" width="56" height="30" rx="6" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="662.0" y="160" class="t-mono" text-anchor="middle">L1</text><rect x="696" y="140" width="56" height="30" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="724.0" y="160" class="t-mono" text-anchor="middle">G1</text><text x="775" y="160" class="t-mono">4.745.839 ms</text><text x="30" y="226" class="t-small">Chèn ra tuyến đi lần lượt từng cặp rồi gom bốn điểm cuối; không phép 2-opt, Or-opt hay dời cặp nào làm giảm chi phí,</text><text x="30" y="244" class="t-small">nên đây là cực tiểu cục bộ. Tuyến tối ưu gom L2 L3 trước rồi mới giao; tuyến hàng trên đắt hơn nó +4,27%.</text></svg>
</div>
<figcaption>Hình 1. Instance n04-00 (seed 50001). Chèn rẻ nhất cho tuyến ở hàng trên và tìm kiếm cục bộ không cải thiện thêm được; tuyến tối ưu (DP, trùng OR-Tools) ở hàng dưới gom L2 L3 trước. Chênh lệch +4,27%. Chi phí lấy nguyên văn từ output của lab.</figcaption>
</figure>

## Mốc so sánh: DP chính xác và OR-Tools offline

Nói «Rust kém bao nhiêu» cần một mốc. Lab dùng hai.

**Quy hoạch động chính xác** theo tập đỉnh đã đi, `O(2^(2n) · (2n)²)` thời gian và `2^(2n) · 2n` ô bộ nhớ, nên chỉ chạy với n ≤ 9 và chỉ chạy ở lab cho n = 4 và 8. Đây là mốc duy nhất **chứng minh được** tối ưu. Công thức là cận trên: code chỉ mở rộng các trạng thái thỏa lấy trước giao, nên số trạng thái thật đi qua ít hơn nhiều, dù bảng vẫn cấp phát đủ ô.

**OR-Tools** (Python 3.13, `ortools` 9.15.6755) chạy offline, một lần, ghi vào `fixtures/loc50/ortools-results.json`; lab Rust đọc file đó. Mô hình khớp bài toán Rust: tuyến mở bằng một đỉnh kết thúc giả có chi phí 0 từ mọi đỉnh, lấy trước giao bằng `AddPickupAndDelivery`, cùng xe, thời điểm tích lũy của lấy không muộn hơn của giao:

`examples/toi-noi-location/python/loc50/loc50_ortools.py`

```python
    for k in range(n):
        p, d = mgr.NodeToIndex(1 + 2 * k), mgr.NodeToIndex(2 + 2 * k)
        routing.AddPickupAndDelivery(p, d)
        solver.Add(routing.VehicleVar(p) == routing.VehicleVar(d))
        solver.Add(time_dim.CumulVar(p) <= time_dim.CumulVar(d))
    params = pywrapcp.DefaultRoutingSearchParameters()
    params.first_solution_strategy = routing_enums_pb2.FirstSolutionStrategy.PARALLEL_CHEAPEST_INSERTION
    params.local_search_metaheuristic = routing_enums_pb2.LocalSearchMetaheuristic.GUIDED_LOCAL_SEARCH
    params.time_limit.FromMilliseconds(int(limit_s * 1000))
```

Điều cần nói thẳng về mốc này: **OR-Tools là mốc tham chiếu trong giới hạn thời gian của nó, không phải tối ưu đã chứng minh**, trừ khi solver tự nói vậy. Trạng thái `SUCCESS` của routing chỉ có nghĩa là tìm được một lời giải, không có nghĩa tối ưu. Guided Local Search còn không có điều kiện dừng nào khác ngoài giới hạn thời gian (2 s, 2 s, 3 s, 5 s, 8 s theo cỡ n); trong lab mọi instance đều dùng gần đúng hết giới hạn đó, dù có còn cải thiện hay không. Chỉ ở n = 4 và 8, nơi DP kiểm được, ta biết OR-Tools đạt đúng tối ưu (gap 0,00% ở cả 40 instance). Ở n ≥ 12, tối ưu thật không đắt hơn lời giải của OR-Tools, nên «gap so với OR-Tools» chỉ là **cận dưới** của «gap so với tối ưu»: Rust có thể kém tối ưu nhiều hơn con số trong bảng, không thể ít hơn. Instance n = 20 mà Rust thắng OR-Tools cho thấy OR-Tools không phải lúc nào cũng tối ưu ở cỡ đó. Ngoài ra OR-Tools chỉ chạy một lần, không lặp nhiều seed của solver.

Kiểm chéo: mọi tuyến OR-Tools hợp lệ (lấy trước giao) và chi phí Python bằng chi phí Rust tính lại trên cùng ma trận: 0 instance sai, 0 kết quả ứng với ma trận khác (băm FNV).

## Kết quả: gap theo cỡ n

Gap = (chi phí Rust − chi phí mốc) / chi phí mốc, ghép cặp theo instance; KTC 95% theo phân phối t trên các instance cùng cỡ. Trên dữ liệu mô phỏng, 20 instance mỗi cỡ, seed 50001…50020:

| n (cặp) | Chỉ chèn: gap TB | Chèn + LS: gap TB [KTC] | Chèn + LS so với OR-Tools: tốt hơn / bằng / kém | Gap lớn nhất | Rust chèn + LS, thời gian | OR-Tools, thời gian tường |
|---:|---|---|---|---|---:|---:|
| 4 | +2,38% | +0,39% [−0,10; +0,89] | 0 / 15 / 5 | +4,27% | 2 µs | 2,006 s (giới hạn 2 s) |
| 8 | +6,41% | +2,77% [+1,27; +4,27] | 0 / 6 / 14 | +13,06% | 8 µs | 2,005 s (giới hạn 2 s) |
| 12 | +5,00% | +1,60% [+0,82; +2,38] | 0 / 3 / 17 | +5,25% | 45 µs | 3,006 s (giới hạn 3 s) |
| 20 | +7,50% | +2,27% [+0,83; +3,72] | 1 / 2 / 17 | +8,15% | 525 µs | 5,005 s (giới hạn 5 s) |
| 30 | +7,28% | +2,13% [+1,28; +2,98] | 0 / 3 / 17 | +5,35% | 646 µs | 8,006 s (giới hạn 8 s) |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 380" role="img" aria-label="Hình 2. Gap so với OR-Tools theo số cặp n. Tìm kiếm cục bộ kéo gap từ 2,38–7,50% (chỉ chèn) xuống 0,39–2,77%; vạch là KTC 95% của cột chèn + LS. Cỡ n = 4 có KTC chứa 0, các cỡ còn lại thì không."><rect x="10" y="10" width="860" height="360" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="30" y="38" class="t-b">Gap = (chi phí Rust − chi phí OR-Tools) / chi phí OR-Tools, trung bình 20 instance mỗi cỡ</text><text x="30" y="58" class="t-small">Dữ liệu mô phỏng, ghép cặp theo instance; vạch trên cột «chèn + LS» là KTC 95%.</text><path d="M110 288.0 L840 288.0" class="f-none s-muted" stroke-width="1.5"/><text x="100" y="292.0" class="t-small" text-anchor="end">0%</text><path d="M110 244.0 L840 244.0" class="f-none s-line" stroke-width="1.5"/><text x="100" y="248.0" class="t-small" text-anchor="end">2%</text><path d="M110 200.0 L840 200.0" class="f-none s-line" stroke-width="1.5"/><text x="100" y="204.0" class="t-small" text-anchor="end">4%</text><path d="M110 156.0 L840 156.0" class="f-none s-line" stroke-width="1.5"/><text x="100" y="160.0" class="t-small" text-anchor="end">6%</text><path d="M110 112.0 L840 112.0" class="f-none s-line" stroke-width="1.5"/><text x="100" y="116.0" class="t-small" text-anchor="end">8%</text><rect x="560" y="66" width="14" height="14" rx="3" class="f-warn" stroke-width="1.5"/><text x="580" y="78" class="t-small">chỉ chèn rẻ nhất</text><rect x="700" y="66" width="14" height="14" rx="3" class="f-pk" stroke-width="1.5"/><text x="720" y="78" class="t-small">chèn + LS</text><rect x="129.0" y="235.6" width="44" height="52.4" rx="3" class="f-warn" stroke-width="1.5"/><text x="151.0" y="229.6" class="t-small" text-anchor="middle">2,38%</text><rect x="193.0" y="279.4" width="44" height="8.6" rx="3" class="f-pk" stroke-width="1.5"/><path d="M215.0 290.2 L215.0 268.4" class="f-none s-muted" stroke-width="2"/><path d="M210.0 268.4 L220.0 268.4" class="f-none s-muted" stroke-width="2"/><path d="M210.0 290.2 L220.0 290.2" class="f-none s-muted" stroke-width="2"/><text x="215.0" y="260.4" class="t-small" text-anchor="middle">0,39%</text><text x="183.0" y="336" text-anchor="middle">n = 4 cặp</text><rect x="275.0" y="147.0" width="44" height="141.0" rx="3" class="f-warn" stroke-width="1.5"/><text x="297.0" y="141.0" class="t-small" text-anchor="middle">6,41%</text><rect x="339.0" y="227.1" width="44" height="60.9" rx="3" class="f-pk" stroke-width="1.5"/><path d="M361.0 260.1 L361.0 194.1" class="f-none s-muted" stroke-width="2"/><path d="M356.0 194.1 L366.0 194.1" class="f-none s-muted" stroke-width="2"/><path d="M356.0 260.1 L366.0 260.1" class="f-none s-muted" stroke-width="2"/><text x="361.0" y="186.1" class="t-small" text-anchor="middle">2,77%</text><text x="329.0" y="336" text-anchor="middle">n = 8 cặp</text><rect x="421.0" y="178.0" width="44" height="110.0" rx="3" class="f-warn" stroke-width="1.5"/><text x="443.0" y="172.0" class="t-small" text-anchor="middle">5,00%</text><rect x="485.0" y="252.8" width="44" height="35.2" rx="3" class="f-pk" stroke-width="1.5"/><path d="M507.0 270.0 L507.0 235.6" class="f-none s-muted" stroke-width="2"/><path d="M502.0 235.6 L512.0 235.6" class="f-none s-muted" stroke-width="2"/><path d="M502.0 270.0 L512.0 270.0" class="f-none s-muted" stroke-width="2"/><text x="507.0" y="227.6" class="t-small" text-anchor="middle">1,60%</text><text x="475.0" y="336" text-anchor="middle">n = 12 cặp</text><rect x="567.0" y="123.0" width="44" height="165.0" rx="3" class="f-warn" stroke-width="1.5"/><text x="589.0" y="117.0" class="t-small" text-anchor="middle">7,50%</text><rect x="631.0" y="238.1" width="44" height="49.9" rx="3" class="f-pk" stroke-width="1.5"/><path d="M653.0 269.7 L653.0 206.2" class="f-none s-muted" stroke-width="2"/><path d="M648.0 206.2 L658.0 206.2" class="f-none s-muted" stroke-width="2"/><path d="M648.0 269.7 L658.0 269.7" class="f-none s-muted" stroke-width="2"/><text x="653.0" y="198.2" class="t-small" text-anchor="middle">2,27%</text><text x="621.0" y="336" text-anchor="middle">n = 20 cặp</text><rect x="713.0" y="127.8" width="44" height="160.2" rx="3" class="f-warn" stroke-width="1.5"/><text x="735.0" y="121.8" class="t-small" text-anchor="middle">7,28%</text><rect x="777.0" y="241.1" width="44" height="46.9" rx="3" class="f-pk" stroke-width="1.5"/><path d="M799.0 259.8 L799.0 222.4" class="f-none s-muted" stroke-width="2"/><path d="M794.0 222.4 L804.0 222.4" class="f-none s-muted" stroke-width="2"/><path d="M794.0 259.8 L804.0 259.8" class="f-none s-muted" stroke-width="2"/><text x="799.0" y="214.4" class="t-small" text-anchor="middle">2,13%</text><text x="767.0" y="336" text-anchor="middle">n = 30 cặp</text></svg>
</div>
<figcaption>Hình 2. Gap so với OR-Tools theo số cặp n. Tìm kiếm cục bộ kéo gap từ 2,38–7,50% (chỉ chèn) xuống 0,39–2,77%; vạch là KTC 95% của cột chèn + LS. Cỡ n = 4 có KTC chứa 0, các cỡ còn lại thì không. Ở n = 4 gap thật không thể âm (mốc là tối ưu), nên cận dưới −0,10 chỉ là hệ quả của công thức t trên mẫu nhỏ.</figcaption>
</figure>

Một đoạn output nguyên văn cho n = 8, nơi cả hai mốc cùng có mặt, và cho n = 30:

```text
  n = 8 cặp (17 đỉnh), 20 instance; chi phí trung bình sau +LS 9171.5 s, số bước cải thiện trung bình 2.2
    chèn vs OR-Tools       gap trung bình   +6.41% [  +4.47%,   +8.34%]  trung vị   +6.46%  lớn nhất  +13.76%
    +LS vs OR-Tools        gap trung bình   +2.77% [  +1.27%,   +4.27%]  trung vị   +2.07%  lớn nhất  +13.06%
    +LS so với OR-Tools trên 20 instance: Rust tốt hơn 0, bằng 6, kém hơn 14
    chèn vs DP (tối ưu)    gap trung bình   +6.41% [  +4.47%,   +8.34%]  trung vị   +6.46%  lớn nhất  +13.76%
    +LS vs DP (tối ưu)     gap trung bình   +2.77% [  +1.27%,   +4.27%]  trung vị   +2.07%  lớn nhất  +13.06%
    OR-Tools vs DP         gap trung bình   +0.00% [  +0.00%,   +0.00%]  trung vị   +0.00%  lớn nhất   +0.00%
    thời gian (µs, trung vị mỗi instance, trung bình qua instance): chèn 2; chèn+LS 8; DP chính xác 9941; OR-Tools (tường thời gian) 2005216 (lớn nhất 2044268)
    tỉ lệ thời gian OR-Tools / Rust(+LS) ≈ 242570×
    trạng thái OR-Tools: {"SUCCESS": 20}
```

```text
  n = 30 cặp (61 đỉnh), 20 instance; chi phí trung bình sau +LS 23321.5 s, số bước cải thiện trung bình 13.2
    chèn vs OR-Tools       gap trung bình   +7.28% [  +5.74%,   +8.82%]  trung vị   +6.67%  lớn nhất  +15.58%
    +LS vs OR-Tools        gap trung bình   +2.13% [  +1.28%,   +2.98%]  trung vị   +2.03%  lớn nhất   +5.35%
    +LS so với OR-Tools trên 20 instance: Rust tốt hơn 0, bằng 3, kém hơn 17
    thời gian (µs, trung vị mỗi instance, trung bình qua instance): chèn 193; chèn+LS 646; OR-Tools (tường thời gian) 8005819 (lớn nhất 8039759)
    tỉ lệ thời gian OR-Tools / Rust(+LS) ≈ 12399×
    trạng thái OR-Tools: {"SUCCESS": 20}
```

Đọc ra ba điều. Một, **chèn rẻ nhất + tìm kiếm cục bộ kém OR-Tools khoảng 0,4 đến 2,8%** (gap trung bình +0,39% đến +2,77% tùy cỡ), và chỉ chèn thì kém 2,4 đến 7,5%. Trên cả 100 instance, Rust tốt hơn OR-Tools 1 lần, bằng 29 lần, kém hơn 70 lần. Hai, với n nhỏ, gap so với OR-Tools cũng là gap so với tối ưu thật (OR-Tools bằng DP): +0,39% ở n = 4 và +2,77% ở n = 8, tệ nhất +13,06% ở một instance n = 8. Ba, gap không tăng đều theo n; ở n = 30 là +2,13%, thấp hơn n = 8. Lab không giải thích được điều này; với 20 instance mỗi cỡ, các KTC chồng lên nhau nên chưa nói được gap đổi theo cỡ.

Phần gap còn lại có thể đến từ việc Rust không có xáo trộn hay khởi động lại ngẫu nhiên, trong khi Guided Local Search có cơ chế thoát cực tiểu cục bộ. Đó là giả thuyết, **chưa thử** (thêm xáo trộn sẽ là bước tiếp theo, không phải kết quả của bài).

## Thời gian, và đường nóng có ngân sách

Rust chạy chèn + tìm kiếm cục bộ mất 2 µs ở n = 4 đến 646 µs ở n = 30; DP chính xác mất 8 µs ở n = 4 và khoảng 10 ms ở n = 8. Mỗi số là trung vị các lần đo lặp của một instance, rồi lấy trung bình qua 20 instance (đúng cách lab in); cả Rust lẫn OR-Tools đều không tính thời gian dựng ma trận. OR-Tools dùng hết giới hạn thời gian 2 đến 8 s. **Tất cả đo khi máy đang chịu tải nặng** (load average khoảng 121 trên 10 lõi), nên chỉ đọc tương đối. Có một cái bẫy khi đọc tỉ lệ: ở n = 30 OR-Tools tốn gấp khoảng 12.400 lần, nhưng đó là giới hạn thời gian ta *đặt* cho nó, vì GLS không dừng sớm. Con số này **không** nói OR-Tools cần 8 s để đạt lời giải của nó, và cũng không nói Rust «nhanh gấp 12.400 lần» ở cùng chất lượng, vì chất lượng thấp hơn khoảng 2%.

Điều thực dụng cho đường nóng là **ngân sách thời gian**: `solve` nhận một hạn, và khi hết hạn nó vẫn trả tuyến hợp lệ (phần chưa chèn được nối vào cuối theo kiểu lấy rồi giao). Lab đo ở cỡ lớn nhất:

```text
    ngân sách     0 µs: hết hạn 20/20, chi phí chênh trung bình +125.79% (lớn nhất +146.22%)
    ngân sách    50 µs: hết hạn 20/20, chi phí chênh trung bình +75.75% (lớn nhất +96.02%)
    ngân sách   200 µs: hết hạn 20/20, chi phí chênh trung bình +36.31% (lớn nhất +117.78%)
    ngân sách  1000 µs: hết hạn 3/20, chi phí chênh trung bình +0.17% (lớn nhất +2.30%)
    ngân sách  5000 µs: hết hạn 1/20, chi phí chênh trung bình +2.48% (lớn nhất +49.60%)
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 3. Chi phí chênh so với chạy không giới hạn theo ngân sách ở n = 30. Ngân sách 0 µs vẫn trả tuyến hợp lệ nhưng đắt hơn 125,79%; từ 1.000 µs thì 17 trong 20 instance chạy tới cực tiểu cục bộ. Cột 5.000 µs bị một instance kéo lên, nhiều khả năng do tải máy."><rect x="10" y="10" width="860" height="310" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="30" y="38" class="t-b">n = 30 cặp: chi phí chênh so với chạy không giới hạn, theo ngân sách thời gian (20 instance)</text><text x="30" y="58" class="t-small">Dữ liệu mô phỏng. Thời gian đo khi máy chịu tải nặng, chỉ đọc tương đối.</text><text x="30" y="105">ngân sách 0 µs</text><rect x="220" y="89" width="402.5" height="22" rx="4" class="f-warn" stroke-width="1.5"/><text x="630.5" y="105" class="t-b">+125,79%</text><text x="740" y="105" class="t-small">hết hạn 20/20</text><text x="30" y="143">ngân sách 50 µs</text><rect x="220" y="127" width="242.4" height="22" rx="4" class="f-warn" stroke-width="1.5"/><text x="470.4" y="143" class="t-b">+75,75%</text><text x="740" y="143" class="t-small">hết hạn 20/20</text><text x="30" y="181">ngân sách 200 µs</text><rect x="220" y="165" width="116.2" height="22" rx="4" class="f-warn" stroke-width="1.5"/><text x="344.2" y="181" class="t-b">+36,31%</text><text x="740" y="181" class="t-small">hết hạn 20/20</text><text x="30" y="219">ngân sách 1.000 µs</text><rect x="220" y="203" width="3" height="22" rx="4" class="f-pk" stroke-width="1.5"/><text x="231" y="219" class="t-b">+0,17%</text><text x="740" y="219" class="t-small">hết hạn 3/20</text><text x="30" y="257">ngân sách 5.000 µs</text><rect x="220" y="241" width="7.9" height="22" rx="4" class="f-dead" stroke-width="1.5"/><text x="235.9" y="257" class="t-b">+2,48%</text><text x="740" y="257" class="t-small">hết hạn 1/20</text><text x="30" y="304" class="t-small">Hàng 5.000 µs (xám): +2,48% kéo bởi một instance (+49,60%), nhiều khả năng do tải máy; không chỉnh, chưa đo lại.</text></svg>
</div>
<figcaption>Hình 3. Chi phí chênh so với chạy không giới hạn theo ngân sách ở n = 30. Ngân sách 0 µs vẫn trả tuyến hợp lệ nhưng đắt hơn 125,79%; từ 1.000 µs thì 17 trong 20 instance chạy tới cực tiểu cục bộ. Cột 5.000 µs bị một instance kéo lên (+49,60%), nhiều khả năng do tải máy: ngân sách tính bằng thời gian thực nên hai lần chạy dưới tải có thể dừng ở hai chỗ khác nhau. Đó là phỏng đoán, lab chưa đo lại.</figcaption>
</figure>

Ngân sách 1 ms đã đủ cho 17/20 instance cỡ 30 chạy tới cực tiểu cục bộ, với chi phí chênh trung bình +0,17%. Ngân sách 5 ms lại có một instance +49,60%: nhiều khả năng máy bị giành CPU đúng lúc đó, vì ngân sách tính bằng thời gian thực, nhưng lab chưa đo lại nên đây chỉ là phỏng đoán. Điều đo được chắc chắn là phần an toàn: dừng ở đâu cũng có tuyến hợp lệ. «Cho thêm thời gian thì tốt dần» đúng với xu hướng chung, không đúng với từng lần chạy dưới tải.

## Tổng hợp P05: cái gì đáng làm trước

Các bài dùng simulator của P05 cùng một thước: so với baseline P03 (`NearestStraightLine`, ghép ngay) hoặc bước trước, 20 seed đánh giá, có cặp, **trên dữ liệu mô phỏng**; «không phân biệt được» nghĩa là KTC 95% chứa 0. Bảng dưới là các dòng tổng hợp trong `results/p05.md`:

| Bài | Can thiệp, kết quả chính (mật độ «thường» nếu không ghi) | Ai lợi | Ai chịu thiệt | Kết quả rỗng hoặc xấu |
|---|---|---|---|---|
| [41](/articles/location-gan-nhat-sai-o-dau-do-bang-mo-phong) | Gần nhất theo ETA thay chim bay: `deadhead_m` TB −66,2 m, p95 −533,0 m, max −3.703,1 m, ETA −12,9 s, hủy −0,0093 | Khách ngoại vi, khách ở đuôi | Không rõ: +145,3 m ở trung tâm không lặp lại trên 40 seed mới (+32,6 m, khoảng chứa 0) | p50 `deadhead_m` và km rỗng tổng không phân biệt được; «thưa xe», «cao điểm» cũng vậy ở `deadhead_m` |
| [42](/articles/location-ghep-theo-lo-hungarian-kuhn-munkres-rust) | Hungarian trên lô: rẻ hơn tham lam 9,5–14,7% trên cùng ma trận; trong sim chỉ có lợi khi lô từ 2 khách (cao điểm: `deadhead_m` −911,7 m) | Khách gần xe, tài xế (km rỗng tổng −632,1 m ở cao điểm) | Khách xa xe (hết kiên nhẫn 0,0149 → 0,0717 ở cao điểm; cả 659 khách đều từng bị khách đặt sau vượt) | «thường»: không phân biệt được |
| [43](/articles/location-cua-so-lo-do-tre-doi-chat-luong-ghep) | Cửa sổ lô: cao điểm, cửa sổ 30 s giảm cả chờ (−13,7 s) lẫn `deadhead_m` (−327,9 m) | Khách và tài xế khi cầu dày | Khách khi cầu thường (+6,6…+30,7 s chờ) | «thường»: cửa sổ chỉ thêm chờ; thích ứng không thắng cố định |
| [44](/articles/location-cat-tia-ung-vien-h3-ma-tran-thua) | Cắt tỉa H3 và ma trận thưa: xe gấp 4 lần khách, +2,43…+4,58% ETA, thường nhanh hơn (1,9–18,3 lần, số rất nhiễu) | Độ trễ máy chủ | Chất lượng ghép 1–13% | Xe bằng khách: mất khách; ở 3.000×3.000 một vùng khổng lồ, bộ giải đặc nên không rút được khâu giải; cắt tỉa + lượt bù đắt hơn tối ưu 14,9% trên một lô |
| [45](/articles/location-chi-phi-ghep-cong-bang-tai-xe-rui-ro-huy) | Chi phí có công bằng, giới hạn đón xa, rủi ro hủy: `v1-idle` Gini −0,039; `v1-limit` ETA −18,5 s | Tài xế rảnh lâu; khách gần | Khách ngoại vi (hoàn thành −0,027…−0,031), tài xế ở «thưa xe» (rảnh dài nhất +1.904 s) | `v1-risk`, `v1-age` khi ghép ngay chưa được thử (y hệt `v0-eta` do hình dạng lô); trong lô 10 s ở cao điểm không có hiệu đáng kể |
| [46](/articles/location-goi-y-sang-dispatch-csharp-grpc-lease-idempotency) | `SuggestMatch` gRPC và stub Dispatch: 23/23 kiểm tra hợp đồng qua | Ranh giới Rust/C# rõ | Một vòng gọi thêm; gợi ý cũ bị bỏ phí (chưa đo tần suất) | Stub trong bộ nhớ: không chứng minh Dispatch thật |
| [47](/articles/location-chain-dispatch-tai-xe-sap-tra-khach) | Chain (xe sắp trả khách): «thưa xe» +3,2 chuyến, hết kiên nhẫn 0,0733 → 0,0415 | Khách suýt bị bỏ rơi | Khách còn lại (`wait_s` +26,9 s) | Chuyến trước kéo dài (σ 0,3): p95 +82,7 s, không phân biệt được với mức σ 0,1 (+69,7 s); `risk_k = 1` không thấy giúp |
| [48](/articles/location-lech-tuyen-dinh-tuyen-lai-ghep-lai) | Phát hiện lệch tuyến, hysteresis, ghép lại: phát hiện 95,2%, báo nhầm 2,5%; khách nhận 5/279 cập nhật | Khách (ít thông báo vô nghĩa) | Tài xế bị rút chuyến (17 ca đổi đều mất ≥ 1 km đã chạy) | Nhiễu thô: 80,5% báo nhầm; lợi của ghép lại bị thổi lên vì tài xế trong lab không được chọn bằng ghép |
| [49](/articles/location-tai-xe-ranh-di-dau-dieu-huong-theo-nhu-cau) | Gợi ý di chuyển theo nhu cầu H3: `pickup_eta_s` −36,1 s | Khách trung tâm và vành trong | Tài xế: km rỗng tổng +1.428,9 m mỗi khách | Ngoại vi không đổi; bầy đàn làm xấu đi («dày xe» +15,5 s) |
| 50 | Chèn + 2-opt/Or-opt so với OR-Tools: gap +0,39…+2,77% | Đường nóng (thời gian 2–646 µs, máy chịu tải) | Chất lượng khoảng 2% | Rust tốt hơn OR-Tools 1 trong 100 instance |

Bài 46, 48 và 50 không chạy trong simulator (lab độc lập), nên ba dòng đó không so được với baseline của bài 30. Việc nên thử trước theo các số này, chỉ trong mô phỏng: **một**, ETA thay chim bay là bước đơn giản nhất, cắt đuôi rõ ở `thường` và `dày xe`, lặp lại trên seed mới, tuy chi phí dựng ma trận chưa được cân ở bài 41; **hai**, ghép lô chỉ thấy lợi khi cầu dày hơn cung, và cái giá hết kiên nhẫn vẫn chưa có lời giải: thưởng chờ lâu 0,5 không chữa được, giới hạn đón xa làm nặng thêm (bài 45); **ba**, giới hạn đón xa và công bằng cho tài xế phải đo theo vùng, vì con số toàn cục che mất ai chịu thiệt; **bốn**, gợi ý di chuyển cho tài xế rảnh đổi 36,1 s chờ lấy 1.428,9 m rỗng mỗi khách, hai đơn vị không so trực tiếp được, nên trước hết phải trả lời ai trả cho quãng đó.

## Chạy ví dụ và kiểm chứng

OR-Tools chạy riêng, vì tốn khoảng 7 phút. Kết quả nằm sẵn trong `fixtures/loc50/ortools-results.json`; lab Rust chỉ đọc file đó. Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc50_nhieu_diem_dung -- --write-instances
cd examples/toi-noi-location/python/loc50 && uv sync --python 3.13 && uv run python loc50_ortools.py
cd - && cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc50_nhieu_diem_dung
```

Lệnh thứ nhất sinh `fixtures/loc50/instances.json`, lệnh thứ hai chạy OR-Tools (cần `uv` và mạng lần đầu để tải gói), lệnh thứ ba chạy so sánh. Output, đầu file (nguyên văn từ `results/p05.md`), kèm dòng kiểm chéo:

```text
OR-Tools: đọc fixtures/loc50/ortools-results.json (100 kết quả), ortools 9.15.6755; first_solution = PARALLEL_CHEAPEST_INSERTION, metaheuristic = GUIDED_LOCAL_SEARCH, giới hạn thời gian theo cỡ n = {"12":3.0,"20":5.0,"30":8.0,"4":2.0,"8":2.0}, Python 3.13.15
  kiểm tra chéo: tuyến OR-Tools hợp lệ (lấy trước giao) và chi phí Python = chi phí Rust tính lại: 0 instance sai; kết quả OR-Tools ứng với ma trận khác (băm FNV lệch): 0
```

`bash examples/toi-noi-location/verify-p05.sh` chạy lab với `--fixture` trên đồ thị tổng hợp (không có OR-Tools), và chỉ kiểm rằng `ortools` nạp được ngoại tuyến; nó **không** giải lại OR-Tools. Các số trong bài đến từ lần chạy đầy đủ trên HCM ghi ở `results/p05.md`.

## Giới hạn của ví dụ

- **Bài toán rút gọn**: một người giao hàng, không tải trọng, không khung giờ, không nhiều xe. Giao hàng thật có thêm cả ba; kết quả ở đây không tự mang sang được.
- **Instance mô phỏng**: điểm lấy và giao rải khắp bản đồ, không phải bưu kiện cùng một quận, nên các cạnh dài hơn giao hàng đô thị thật. Chi phí là thời gian chạy từ CH, không có chi phí rẽ.
- **OR-Tools bị chặn bởi giới hạn thời gian**, chạy một lần, một cấu hình (`PARALLEL_CHEAPEST_INSERTION` + `GUIDED_LOCAL_SEARCH`). Chất lượng của nó phụ thuộc giới hạn đó và tải máy; nó là mốc tham chiếu, không phải tối ưu, trừ n = 4 và 8.
- **Rust không có xáo trộn hay khởi động lại ngẫu nhiên**, và chỉ có ba loại phép dời. Gap khoảng 2% là của cấu hình này.
- **Thời gian đo khi máy chịu tải nặng.** Đừng kết luận gì về thời gian tuyệt đối; đọc tỉ lệ trong cùng một lần chạy.
- **Tổng hợp P05 gom kết quả của các bài trước**, mỗi bài chạy hai đến bốn bộ giả định; các bộ còn lại chưa chạy (ghi ở từng bài). Mười bài cộng lại là hàng trăm phép so sánh, nên một dấu sao đơn lẻ ở bất kỳ bài nào cũng cần được chạy lại trước khi tin, như bài 41 đã làm. Chưa thử tải cao hơn «cao điểm», chưa so với dispatch thật của một công ty (không có dữ liệu), chưa tích hợp dự báo thật cho bài 49.

## Bài tập

**Bài 1.** Kiểm lại gap +4,27% của instance n04-00 từ hai chi phí trong output.

<details><summary>Lời giải bài 1</summary>
<p>(4.948.489 − 4.745.839) / 4.745.839 = 202.650 / 4.745.839 ≈ 0,0427, tức +4,27%. Trùng với «gap lớn nhất» của cỡ n = 4 trong output, nên đó chính là instance tệ nhất cỡ này.</p>
</details>

**Bài 2.** Vì sao 2-opt thông thường không được đảo một đoạn chứa cả điểm lấy lẫn điểm giao của cùng một cặp? Và vì sao với ma trận bất đối xứng, code phải giữ hai tổng `fwd` và `bwd`?

<details><summary>Lời giải bài 2</summary>
<p>Đảo đoạn đảo thứ tự các đỉnh trong đoạn, nên nếu cả L<sub>k</sub> và G<sub>k</sub> nằm trong đoạn thì sau khi đảo G<sub>k</sub> đứng trước L<sub>k</sub>, vi phạm lấy trước giao. Với ma trận bất đối xứng, <code>c(a,b)</code> khác <code>c(b,a)</code>: đảo đoạn làm mỗi cạnh trong đoạn chạy theo chiều ngược lại, nên chi phí đoạn sau khi đảo là tổng <code>c(v, trước)</code> (<code>bwd</code>) chứ không phải tổng <code>c(trước, v)</code> (<code>fwd</code>). Giữ hai tổng chạy cho phép tính delta mỗi <code>j</code> trong O(1).</p>
</details>

**Bài 3.** Ở n = 12, chèn + LS kém OR-Tools +1,60% trung bình, 17/20 instance kém hơn. Có thể kết luận Rust kém tối ưu thật +1,60% không? Vì sao?

<details><summary>Lời giải bài 3</summary>
<p>Không. Ở n = 12 DP không chạy (chỉ chạy tới n ≤ 9 trong lab), và OR-Tools chỉ là mốc tham chiếu trong giới hạn 3 s, không được chứng minh tối ưu. Gap thật so với tối ưu có thể lớn hơn +1,60% nếu OR-Tools chưa tối ưu; chỉ ở n = 4 và 8, nơi OR-Tools trùng DP, gap so với OR-Tools mới đồng nghĩa với gap so với tối ưu.</p>
</details>

Bài trước: [bài 49](/articles/location-tai-xe-ranh-di-dau-dieu-huong-theo-nhu-cau).

## Tài liệu gốc

- [OR-Tools: Vehicle Routing with Pickups and Deliveries](https://developers.google.com/optimization/routing/pickup_delivery): cách mô hình lấy trước giao bằng `AddPickupAndDelivery` và ràng buộc thời điểm tích lũy.
- [OR-Tools: Routing Options](https://developers.google.com/optimization/routing/routing_options): chiến lược lời giải đầu, metaheuristic (Guided Local Search) và giới hạn thời gian.
- [2-opt (Wikipedia)](https://en.wikipedia.org/wiki/2-opt): phép đảo đoạn cơ bản của tìm kiếm cục bộ.
- [Held–Karp algorithm (Wikipedia)](https://en.wikipedia.org/wiki/Held%E2%80%93Karp_algorithm): quy hoạch động theo tập đỉnh, nền của mốc chính xác cho cỡ nhỏ.
- [Guided local search (Wikipedia)](https://en.wikipedia.org/wiki/Guided_local_search): metaheuristic mà OR-Tools dùng trong lab này.
