---
slug: location-gan-nhat-sai-o-dau-do-bang-mo-phong
title: "Tài xế gần nhất sai ở đâu: đo bằng mô phỏng"
summary: "Bài 41: chim bay so với ETA đường bộ, phản ví dụ tham lam cướp tài xế, cách đọc so sánh có cặp 20 seed. ETA thắng rõ ở đuôi, trung vị không phân biệt được, dấu sao ở trung tâm không lặp lại trên seed mới. Trên dữ liệu mô phỏng."
category: location-routing
level: advanced
tags: [toi-noi, location, rust, dispatch, matching, simulation, baseline, statistics]
language: vi
---

Trong 2.000 khách mà lab lấy mẫu, ca tệ nhất trông như sau. Khách ghim ở (10,71574; 106,62485). Trong 80 tài xế rảnh của lần rút đó, DR-079 cách điểm đón 1.315 m theo đường chim bay, còn DR-002 cách 3.169 m. Cách chọn «gần nhất» của baseline sẽ giao khách cho DR-079, và ETA đường bộ của DR-079 là 1.430 s. DR-002, xa gấp hơn hai lần trên bản đồ, tới sau 462 s. Chênh 968 s, tức 16,1 phút khách đứng chờ, vì tuyến thật của DR-079 dài 15.742 m cho 1.315 m chim bay. Một phần con số này là do chính bản đồ của lab gây ra: DR-079 đứng cách mép cắt phía nam của đồ thị chừng 450 m, và trên đồ thị cắt rộng hơn cùng cặp điểm còn 11.280 m, 1.018 s. Đường vòng vẫn có thật, nhưng ca tệ nhất của một mẫu thường là ca mà giả định của phép đo yếu nhất.

Vậy có nên thay hết «gần nhất theo chim bay» bằng «gần nhất theo ETA» ngay không? Bài này không đưa ra một con số «giảm x%» cho câu hỏi đó. Thay vào đó, bài dựng baseline cho cả phần P05 và trả lời bằng cách đo: sai ở đâu, sai bao nhiêu, ai được lợi, ai chịu thiệt, và mức nào là nhiễu. Kết quả có ích hơn một con số đẹp: lợi ích có thật, nhỏ ở trung bình, rõ ở đuôi; còn dấu hiệu «một nhóm tài xế trả giá» thì biến mất khi chạy lại trên seed mới, và đó là bài học thống kê của cả phần.

Đây là bài 41 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi), bài mở đầu phần ghép chuyến và giảm km rỗng. Bài dựng trên simulator của [bài 30](/articles/location-tong-hop-mo-phong-su-kien-roi-rac-seed), ma trận ETA của [bài 26](/articles/location-ma-tran-many-to-many-bucket-ch) và câu hỏi xuyên suốt của [bài 01](/articles/location-bai-toan-quang-duong-khach-va-tai-xe). Phần này luôn dùng `AtPin` (đón đúng điểm ghim) làm cách chọn điểm đón, nên chỉ thay cách chọn tài xế. Mọi số đo là **trên dữ liệu mô phỏng**: kịch bản `hcm-baseline`, đồ thị HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors), 20 seed đánh giá 20261106…20261125. Không có GPS người thật.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Đổi «gần nhất theo chim bay» thành «gần nhất theo ETA đường bộ», giữ nguyên mọi thứ khác, chạy 20 seed ghép cặp. Bạn đoán <code>deadhead_m</code> (km rỗng) trung vị, trung bình và p95 thay đổi thế nào? Và có nhóm khách hay tài xế nào bị xấu đi không?</p>
<details><summary>Xem đáp án</summary><p>Trung bình giảm 66,2 m, khoảng tin cậy 95% là [−112,1; −20,3]. p95 giảm 533,0 m. <b>Nhưng p50 không phân biệt được</b>: +37,8 m, khoảng [−25,6; 101,1] chứa 0. Lợi nằm ở đuôi, không ở số đông. Có một nhóm trông như xấu đi: chuyến đón khách vùng trung tâm chạy rỗng thêm 145,3 m, khoảng [43,0; 247,6]. Trên 40 seed mới con số này còn +32,6 m, khoảng chứa 0.</p></details>
</div>

## Baseline của cả phần: hai cách chọn, một so sánh

Baseline P03 là `NearestStraightLine`: duyệt khách theo thứ tự đặt, mỗi khách lấy tài xế rảnh gần điểm đón nhất theo haversine (hoà thì mã nhỏ hơn). Đó cũng là cách tìm tài xế ở [bài 09](/articles/location-tim-tai-xe-gan-nhat-bon-cach-co-so-do).

`examples/toi-noi-location/rust/crates/simulator/src/strategy.rs`

```rust
pub fn nearest_straight_line(pending: &[PendingView], idle: &[DriverView]) -> Vec<Assignment> {
    let mut taken = vec![false; idle.len()];
    let mut out = Vec::new();
    for p in pending {
        let mut best: Option<(f64, usize)> = None;
        for (i, d) in idle.iter().enumerate() {
            if taken[i] {
                continue;
            }
            let dist = haversine_m(d.latlng, p.pickup.stop_point);
            if best.is_none_or(|(b, j)| dist < b || (dist == b && d.id < idle[j].id)) {
                best = Some((dist, i));
            }
        }
        if let Some((_, i)) = best {
            taken[i] = true;
            out.push(Assignment {
                customer: p.customer,
                driver: idle[i].id,
            });
        }
    }
    out
}
```

Cách thứ hai, `nearest-eta`, giữ nguyên vòng lặp «khách đặt trước chọn trước», nhưng thay khoảng cách chim bay bằng ETA đường bộ lấy từ ma trận CH của bài 26. Cách này nằm trong crate `matcher` dưới tên `greedy_fifo`, chạy trên ma trận mà hàng là khách, cột là tài xế, ô là ETA tính bằng mili giây:

`examples/toi-noi-location/rust/crates/matcher/src/assign.rs`

```rust
/// Tham lam theo thứ tự hàng (khách đặt trước chọn trước): mỗi khách lấy tài xế rẻ nhất còn trống. Đây là «gần nhất theo ETA»
/// khi hàng xếp theo giờ đặt; hoà thì chọn cột nhỏ hơn.
pub fn greedy_fifo(costs: &Dense) -> Vec<(usize, usize)> {
    let mut taken = vec![false; costs.cols];
    let mut out = Vec::new();
    for r in 0..costs.rows {
        let mut best: Option<(i64, usize)> = None;
        for (c, &is_taken) in taken.iter().enumerate() {
            let v = costs.get(r, c);
            if !is_taken && allowed(v) && best.is_none_or(|(b, _)| v < b) {
                best = Some((v, c));
            }
        }
        if let Some((_, c)) = best {
            taken[c] = true;
            out.push((r, c));
        }
    }
    out
}
```

Hai điều cần nhớ khi đọc kết quả. Một, ma trận ETA tính theo **nút**, không có chi phí rẽ và cấm quay đầu, còn simulator định tuyến theo cạnh, có chi phí rẽ ([bài 23](/articles/location-chi-phi-re-u-turn-do-thi-theo-canh)) và nhiễu tốc độ log-chuẩn σ = 0,1 mà chiến lược không biết. Chiến lược tối ưu trên một ước lượng; số đo là của «thế giới thật» trong mô phỏng. Hai, `nearest-eta` vẫn là tham lam theo thứ tự: nó sửa thước đo khoảng cách, chưa sửa cấu trúc quyết định. Phần sau của bài cho thấy cấu trúc đó cũng có lỗi riêng.

## Cách đọc một so sánh có cặp (đọc một lần, dùng cho cả phần)

[Bài 30](/articles/location-tong-hop-mo-phong-su-kien-roi-rac-seed) đo được `deadhead_m` p50 của baseline dao động 1.874,9 ± 286,6 m giữa các seed, nên một seed không chứng minh gì. Từ bài này, mọi khẳng định «tốt hơn» hay «xấu hơn» trong phần P05 đều theo đúng quy trình sau, và các bài 42–45 chỉ nhắc lại tên các cột.

1. **Cùng seed, hai chiến lược.** Với seed `s`, chạy baseline A và chiến lược B trên cùng dãy khách và vị trí tài xế ban đầu. Hiệu của seed đó là `Δ_s = B_s − A_s`.
2. **20 seed đánh giá**: 20261106…20261125 (gồm 5 seed của bài 30). Tham số của chiến lược, nếu có, chỉ được chọn trên **dải seed chỉnh tham số** 9001…9010, hai dải không giao nhau (có test `seed_ranges_do_not_overlap`). Các bài sau không báo cáo kết luận trên dải seed chỉnh.
3. **Trung bình hiệu và khoảng tin cậy 95%** theo phân phối t Student trên 20 giá trị `Δ_s`: nửa khoảng bằng `t · sd / √20`, với `t = 2,093` cho 19 bậc tự do. Khoảng này nói về **hiệu trung bình** của kịch bản nếu rút seed mới. Điều đó không có nghĩa là 95% số seed nằm trong khoảng. Vì sd tính trên từng cặp, nhiễu chung giữa các seed không làm khoảng rộng ra, đó là lợi của việc ghép cặp.
4. **Nhận xét chỉ có ba mức.** «TỐT HƠN» hoặc «XẤU HƠN» khi khoảng tin cậy **không chứa 0**; ngược lại là «không phân biệt được». Cột `B<A/B>A` đếm số seed mà B thấp hơn hoặc cao hơn A, để thấy hiệu có đều giữa các seed hay chỉ do vài seed.

`examples/toi-noi-location/rust/crates/matcher/src/eval.rs`

```rust
    let mut d: Vec<f64> = pairs.iter().map(|(x, y)| y - x).collect();
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
    let md = mean(&d);
    let sd = if n > 1 {
        (d.iter().map(|x| (x - md).powi(2)).sum::<f64>() / (n - 1) as f64).sqrt()
    } else {
        f64::NAN
    };
    let half = t_crit_975(n.saturating_sub(1)) * sd / (n as f64).sqrt();
    Paired {
        n,
        mean_a: mean(&xa),
        mean_b: mean(&xb),
        mean_diff: md,
        ci_lo: md - half,
        ci_hi: md + half,
```

Bốn lời cảnh báo đi kèm, vì quy trình này vẫn có thể bị đọc sai.

- **«Không phân biệt được» không có nghĩa là «bằng nhau».** Điều này nghĩa là 20 seed không đủ để thấy hiệu; hiệu thật có thể nhỏ hơn độ phân giải của phép đo.
- **«Khác 0» không có nghĩa là «lớn» hay «đáng làm».** `cancel_rate` −0,0093 có khoảng không chứa 0, nhưng trên 228,4 khách đặt mỗi seed chỉ là chừng 2 khách. Độ lớn phải đọc riêng.
- **Mỗi bảng có 17 chỉ số.** Với mức tin cậy 95%, kể cả khi hai chiến lược hoàn toàn giống nhau, kỳ vọng vẫn có chừng một chỉ số «có ý nghĩa thống kê» do may rủi. Vì vậy hãy đọc **mẫu hình**: nhiều chỉ số liên quan cùng hướng (km rỗng trung bình, p95, max, ETA, chờ) đáng tin hơn một dấu sao đơn lẻ. Phần sau của bài có đúng một ví dụ.
- **Cùng seed chỉ khử một phần nhiễu.** Dòng khách và vị trí tài xế ban đầu giống nhau, nhưng dòng huỷ chuyến và nhiễu tốc độ bị lệch khi hai chiến lược gán khác thứ tự (bài 30 đã nêu).
- **Khoảng t giả định rằng hiệu giữa các seed xấp xỉ phân phối chuẩn.** Với 20 seed, giả định này chưa được kiểm; đó là lý do các bài ghi thêm trung vị hai bên và số seed theo từng hướng.

## Chim bay sai ở đâu: mẫu 2.000 khách

Mục 1 của lab không dùng simulator mà hỏi một câu hẹp: với mỗi khách, nếu có 80 tài xế rảnh ngẫu nhiên (trong 120), chọn theo chim bay và chọn theo ETA có khác nhau không, và khác thì thiệt bao nhiêu giây? Mẫu 2.000 khách, seed 41001. «Thiệt hại» là ETA của lựa chọn chim bay trừ ETA nhỏ nhất.

| Đại lượng (giây, trên dữ liệu mô phỏng) | Giá trị |
|---|---:|
| Hai cách chọn khác nhau | 689/2.000 khách (34,5%) |
| Thiệt hại trên mọi khách: trung bình | 27,2 |
| p50 | 0,0 |
| p95 | 144,9 |
| p99 | 430,9 |
| lớn nhất | 967,9 |
| Chỉ trên 689 khách bị khác: thiệt hại p50 / p95 | 41,1 / 348,7 |
| ETA của lựa chọn chim bay so với ETA tốt nhất (trung vị, chỉ tính các khách bị khác) | 1,23× |

Chim bay chọn khác ở hơn một phần ba khách, và **p50 của thiệt hại là 0**: 65,5% khách hai cách chọn cùng một tài xế; ở 689 ca còn lại chim bay đều tệ hơn. Cái hại nằm ở đuôi: p95 144,9 s, p99 430,9 s, ca tệ nhất 967,9 s. Các ETA này là ETA CH theo nút; với tuyến có hướng và chi phí rẽ như simulator đo, ca tệ nhất là 1.500 s so với 511 s.

Ca tệ nhất đáng soi vì nó mở đầu bài này. Cờ `--ca-te` in tuyến thật: DR-079 ở vĩ độ 10,70400, tuyến đi sát 10,70252, trong khi đồ thị bị cắt ở 10,70. Dựng lại đồ thị với mép cắt lùi về 10,62 và 106,54, DR-079 còn 11.280 m, 1.018 s, còn DR-002 giữ nguyên 3.542 m, 462 s. Vậy 412 s trong 968 s là do mép bản đồ của lab; phần còn lại là đường vòng có thật trên dữ liệu OSM. Lab không phân tích vì sao phải vòng (kênh rạch, dải phân cách, đường một chiều đều là giả thuyết). Chỉ có ca này được kiểm; lab chưa đo mép cắt ảnh hưởng tới bao nhiêu phần đuôi còn lại.

Đây là lý do một baseline «ngây thơ có chủ ý» đáng giữ: nó đúng ở số đông và sai nặng ở số ít. Bất kỳ cải tiến nào chỉ nhìn trung vị sẽ kết luận «không có gì để sửa».

## Phản ví dụ: tham lam cướp tài xế của khách sau

Đổi sang ETA không sửa được lỗi thứ hai, lỗi của chính thứ tự «ai đặt trước chọn trước». Mục 2 của lab dựng lô gồm **hai khách đặt gần nhau** (ghim cách nhau tối đa 1.500 m) và **ba tài xế rảnh gần nhất** quanh trung điểm, rồi so tham lam theo thứ tự đặt với phép gán tối ưu cả lô (Hungarian, bài 42). Mẫu 2.000 lô, seed 41002. Lô tệ nhất:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 385" role="img" aria-label="Hình 1. Lô tệ nhất của phản ví dụ: khách A đặt trước, khách B đặt sau, hai tài xế DR-055 và DR-092. ETA (giây): A tới DR-055 189, A tới DR-092 222, B tới DR-055 25, B tới DR-092 313. Tham lam theo thứ tự đặt chọn A với DR-055 và B với DR-092, tổng 502 giây. Hungarian chọn A với DR-092 và B với DR-055, tổng 247 giây."><rect x="20" y="20" width="840" height="360" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Hai khách đặt cách nhau 666 m, hai tài xế: tham lam cướp tài xế của khách sau</text><path d="M150 120 L520 120" class="f-none s-warn" stroke-width="3.5"/><path d="M150 270 L520 270" class="f-none s-warn" stroke-width="3.5"/><path d="M150 120 L520 270" class="f-none s-phys" stroke-width="3.5"/><path d="M150 270 L520 120" class="f-none s-phys" stroke-width="3.5"/><text x="335" y="108" class="t-mono t-warn" text-anchor="middle">A–DR-055: 189 s</text><text x="335" y="296" class="t-mono t-warn" text-anchor="middle">B–DR-092: 313 s</text><text x="262" y="158" class="t-mono t-phys">A–DR-092: 222 s</text><text x="262" y="246" class="t-mono t-phys">B–DR-055: 25 s</text><rect x="70" y="96" width="100" height="48" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="80" y="116" class="t-b">Khách A</text><text x="80" y="133" class="t-small">đặt trước</text><rect x="70" y="246" width="100" height="48" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="80" y="266" class="t-b">Khách B</text><text x="80" y="283" class="t-small">đặt sau</text><rect x="520" y="96" width="100" height="48" rx="8" class="f-surface s-muted" stroke-width="1.5"/><text x="532" y="116" class="t-b">DR-055</text><text x="532" y="133" class="t-small">tài xế rảnh</text><rect x="520" y="246" width="100" height="48" rx="8" class="f-surface s-muted" stroke-width="1.5"/><text x="532" y="266" class="t-b">DR-092</text><text x="532" y="283" class="t-small">tài xế rảnh</text><rect x="650" y="70" width="190" height="120" rx="8" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="662" y="92" class="t-b">Tham lam (A chọn trước)</text><text x="662" y="112" class="t-mono">A ← DR-055  189 s</text><text x="662" y="130" class="t-mono">B ← DR-092  313 s</text><text x="662" y="156" class="t-b">Tổng 502 s</text><text x="662" y="176" class="t-small">B bị cướp xe ở ngay sát</text><rect x="650" y="205" width="190" height="120" rx="8" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="662" y="227" class="t-b">Hungarian (cả lô)</text><text x="662" y="247" class="t-mono">A ← DR-092  222 s</text><text x="662" y="265" class="t-mono">B ← DR-055   25 s</text><text x="662" y="291" class="t-b">Tổng 247 s</text><text x="662" y="311" class="t-small">A +33 s, B −287 s</text><text x="40" y="350" class="t-small">Nguồn: output loc41, mục 2, lô tệ nhất trong 2.000 lô (seed 41002).</text><text x="40" y="366" class="t-small">Tài xế thứ ba DR-035 (ETA 461 và 477 s) không được chọn. Trên dữ liệu mô phỏng.</text></svg>
</div>
<figcaption>Hình 1. Lô tệ nhất trong 2.000 lô: khách A đặt trước chọn DR-055 (189 s) vì đó là xe gần nhất của A, nhưng DR-055 chỉ cách khách B 25 s. B phải nhận DR-092 (313 s). Phép gán tối ưu đổi chéo: tổng 247 s thay vì 502 s. A thiệt 33 s, B lợi 287 s (ETA làm tròn giây nên hiệu có thể lệch 1 s). Trên dữ liệu mô phỏng.</figcaption>
</figure>

Con số tổng hợp: tham lam theo thứ tự tốn thêm ETA ở **339/2.000 lô (16,9%)**, và trong các lô đó, phần tốn thêm có p50 38,8 s, p95 181,1 s, lớn nhất 254,5 s. Lô trong hình là lô cực đoan nhất của mẫu.

Có hai điều cần đọc kỹ.

**16,9% không phải tần suất trong vận hành.** Lô được dựng để có xung đột: hai khách gần nhau, chỉ ba xe. Trong simulator ở bộ `thường` với ghép ngay, hai khách chờ cùng lúc không xảy ra lần nào: bài 45 đếm 4.568 lô trên 20 seed, đều chỉ một khách. Đây là cách kiểm tra xem lỗi có tồn tại không, không phải ước lượng chuyện này xảy ra bao nhiêu lần trong một ngày.

**Ai được lợi, ai thiệt.** Khách A đặt trước mất thêm 33 s; khách B đặt sau được giảm 287 s. Tổng lợi lớn, nhưng người thiệt là người đã chờ lâu hơn, nên quy tắc «đặt trước chọn trước» bảo vệ A bằng cái giá B phải trả. Một chính sách phân công tối ưu không đương nhiên «công bằng»: bài 45 quay lại điểm này.

## Trong simulator: lợi nhỏ ở trung bình, rõ ở đuôi

Mục 3 chạy hai chiến lược trên simulator, 20 seed ghép cặp 20261106…20261125, kịch bản `hcm-baseline`, bộ giả định `thường` (120 tài xế, 3,5 khách/phút), trên dữ liệu mô phỏng. Lab đối chiếu lại seed 20261106 của baseline: nhật ký chuyến 227 dòng, FNV-1a `100a862df76819aa`, khớp từng byte với con số bài 30 đã công bố. A là chim bay, B là ETA.

| Chỉ số | A | B | Hiệu B − A [KTC 95%] | B<A/B>A | Nhận xét |
|---|---:|---:|---|:-:|---|
| `deadhead_m` trung bình | 2.439,2 | 2.373,0 | −66,2 [−112,1; −20,3] | 16/4 | TỐT HƠN |
| `deadhead_m` p50 | 1.915,5 | 1.953,2 | +37,8 [−25,6; 101,1] | 6/14 | không phân biệt được |
| `deadhead_m` p95 | 6.115,2 | 5.582,2 | −533,0 [−774,4; −291,6] | 17/3 | TỐT HƠN |
| `deadhead_m` max | 12.581,1 | 8.878,0 | −3.703,1 [−4.820,1; −2.586,1] | 20/0 | TỐT HƠN |
| `pickup_eta_s` trung bình | 337,0 | 324,1 | −12,9 [−18,1; −7,7] | 17/3 | TỐT HƠN |
| `wait_s` p50 | 287,1 | 290,2 | +3,1 [−4,4; 10,7] | 9/11 | không phân biệt được |
| `wait_s` p95 | 746,7 | 671,0 | −75,8 [−103,7; −47,8] | 19/1 | TỐT HƠN |
| `walk_m` trung bình | 66,73 | 66,79 | +0,06 [−1,67; 1,79] | 11/9 | không phân biệt được |
| `u_turns` mỗi chuyến | 1,603 | 1,576 | −0,027 [−0,047; −0,008] | 15/5 | TỐT HƠN |
| `cancel_rate` | 0,1407 | 0,1314 | −0,0093 [−0,0146; −0,0040] | 13/4 | TỐT HƠN |
| chuyến hoàn thành | 196,0 | 198,2 | +2,2 [0,9; 3,5] | 4/13 | TỐT HƠN |
| km rỗng tổng / khách đặt (m) | 2.113,5 | 2.078,9 | −34,6 [−80,5; 11,3] | 14/6 | không phân biệt được |

Cách đọc mẫu hình thay vì từng dòng: mọi chỉ số đuôi (p95, max của km rỗng; p95, max của chờ) cùng hướng và rõ; trung vị không phân biệt được; km rỗng tổng trên mỗi khách đặt, chỉ số gộp cả chuyến huỷ sau khi gán, giảm 34,6 m và khoảng chứa 0. Kết luận trung thực: **lợi nhỏ ở trung bình (66,2 m trên mức 2.439,2 m), lớn ở đuôi.** `walk_m` không phân biệt được, đúng như chờ đợi vì cả hai chiến lược đón đúng ghim.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 434" role="img" aria-label="Hình 2. Hiệu giữa gần nhất theo ETA và gần nhất theo chim bay, 20 seed đánh giá, bộ thường. deadhead_m trung bình giảm 66,2 mét, khoảng tin cậy từ trừ 112,1 đến trừ 20,3. deadhead_m p50 tăng 37,8 mét, khoảng chứa 0. p95 giảm 533,0 mét. Theo vùng: trung tâm tăng 145,3 mét, vành trong giảm 76,7, ngoại vi giảm 221,2."><rect x="20" y="20" width="840" height="409" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Gần nhất theo ETA trừ gần nhất theo chim bay (20 seed ghép cặp, bộ «thường»)</text><path d="M400.0 86 L400.0 360" class="f-none s-line" stroke-width="1"/><text x="400.0" y="376" class="t-small" text-anchor="middle">−800</text><path d="M445.5 86 L445.5 360" class="f-none s-line" stroke-width="1"/><text x="445.5" y="376" class="t-small" text-anchor="middle">−600</text><path d="M490.9 86 L490.9 360" class="f-none s-line" stroke-width="1"/><text x="490.9" y="376" class="t-small" text-anchor="middle">−400</text><path d="M536.4 86 L536.4 360" class="f-none s-line" stroke-width="1"/><text x="536.4" y="376" class="t-small" text-anchor="middle">−200</text><path d="M581.8 86 L581.8 360" class="f-none s-line" stroke-width="1"/><text x="581.8" y="376" class="t-small" text-anchor="middle">0</text><path d="M627.3 86 L627.3 360" class="f-none s-line" stroke-width="1"/><text x="627.3" y="376" class="t-small" text-anchor="middle">200</text><path d="M581.8 86 L581.8 360" class="f-none s-muted dash" stroke-width="1.5"/><text x="400" y="72" class="t-small">← ít km rỗng hơn</text><text x="650" y="72" class="t-small" text-anchor="end">nhiều hơn →</text><text x="670" y="72" class="t-small">hiệu [KTC 95%]</text><text x="40" y="120.0" class="t-b">Toàn hệ thống: deadhead_m (mét)</text><text x="52" y="152.0">trung bình</text><path d="M556.3 148.0 L577.2 148.0" class="f-none s-phys" stroke-width="2.5"/><path d="M556.3 143.0 L556.3 153.0 M577.2 143.0 L577.2 153.0" class="f-none s-phys" stroke-width="2"/><circle cx="566.8" cy="148.0" r="5" class="f-phys s-phys" stroke-width="1"/><text x="670" y="152.0" class="t-mono">−66,2 [−112,1; −20,3]</text><text x="52" y="184.0">p50 (trung vị)</text><path d="M576.0 180.0 L604.8 180.0" class="f-none s-dead" stroke-width="2.5"/><path d="M576.0 175.0 L576.0 185.0 M604.8 175.0 L604.8 185.0" class="f-none s-dead" stroke-width="2"/><circle cx="590.4" cy="180.0" r="5" class="f-dead s-dead" stroke-width="1"/><text x="670" y="184.0" class="t-mono">+37,8 [−25,6; 101,1]</text><text x="52" y="216.0">p95 (đuôi)</text><path d="M405.8 212.0 L515.5 212.0" class="f-none s-phys" stroke-width="2.5"/><path d="M405.8 207.0 L405.8 217.0 M515.5 207.0 L515.5 217.0" class="f-none s-phys" stroke-width="2"/><circle cx="460.7" cy="212.0" r="5" class="f-phys s-phys" stroke-width="1"/><text x="670" y="216.0" class="t-mono">−533,0 [−774,4; −291,6]</text><text x="40" y="248.0" class="t-b">Theo vùng của khách: deadhead_m trung bình (mét)</text><text x="52" y="280.0">trung tâm (&lt; 3 km)</text><path d="M591.6 276.0 L638.1 276.0" class="f-none s-warn" stroke-width="2.5"/><path d="M591.6 271.0 L591.6 281.0 M638.1 271.0 L638.1 281.0" class="f-none s-warn" stroke-width="2"/><circle cx="614.8" cy="276.0" r="5" class="f-warn s-warn" stroke-width="1"/><text x="670" y="280.0" class="t-mono">+145,3 [43,0; 247,6]</text><text x="52" y="312.0">vành trong (3–6 km)</text><path d="M548.7 308.0 L580.1 308.0" class="f-none s-phys" stroke-width="2.5"/><path d="M548.7 303.0 L548.7 313.0 M580.1 303.0 L580.1 313.0" class="f-none s-phys" stroke-width="2"/><circle cx="564.4" cy="308.0" r="5" class="f-phys s-phys" stroke-width="1"/><text x="670" y="312.0" class="t-mono">−76,7 [−145,7; −7,7]</text><text x="52" y="344.0">ngoại vi (≥ 6 km)</text><path d="M509.3 340.0 L553.8 340.0" class="f-none s-phys" stroke-width="2.5"/><path d="M509.3 335.0 L509.3 345.0 M553.8 335.0 L553.8 345.0" class="f-none s-phys" stroke-width="2"/><circle cx="531.5" cy="340.0" r="5" class="f-phys s-phys" stroke-width="1"/><text x="670" y="344.0" class="t-mono">−221,2 [−319,2; −123,1]</text><circle cx="52" cy="394" r="5" class="f-phys s-phys" stroke-width="1"/><text x="64" y="398" class="t-small">TỐT HƠN (KTC không chứa 0)</text><circle cx="262" cy="394" r="5" class="f-warn s-warn" stroke-width="1"/><text x="274" y="398" class="t-small">XẤU HƠN (KTC không chứa 0)</text><circle cx="480" cy="394" r="5" class="f-dead s-dead" stroke-width="1"/><text x="492" y="398" class="t-small">không phân biệt được</text><text x="40" y="414" class="t-small">Seed 20261106…25, kịch bản hcm-baseline, 120 xe, 3,5 khách/phút. Số âm = ít km rỗng hơn. Trên dữ liệu mô phỏng.</text></svg>
</div>
<figcaption>Hình 2. Hiệu của gần nhất theo ETA so với chim bay, vẽ từ bảng ở trên và bảng theo vùng ở dưới. Đoạn ngang là khoảng tin cậy 95%; đoạn chạm hoặc bao trùm đường 0 nghĩa là không phân biệt được. Dấu sao ở trung tâm không lặp lại trên 40 seed mới (mục dưới). Trên dữ liệu mô phỏng, 20 seed 20261106…25.</figcaption>
</figure>

### Ai lợi, ai thiệt theo vùng, và một dấu sao không lặp lại

Khách được chia theo khoảng cách từ điểm ghim tới một tâm quy chiếu cố định của lab: trung tâm (dưới 3 km), vành trong (3–6 km), ngoại vi (từ 6 km). Hiệu B − A, có cặp:

| Vùng | Khách/seed | Tỉ lệ hoàn thành A → B | `wait_s` | `deadhead_m` |
|---|---:|---|---|---|
| trung tâm | 53,6 | 0,871 → 0,878 | +7,4 [−3,1; 18,0] | **+145,3 [43,0; 247,6] XẤU HƠN** |
| vành trong | 107,3 | 0,871 → 0,875 | −12,0 [−20,1; −3,9] | −76,7 [−145,7; −7,7] TỐT HƠN |
| ngoại vi | 67,5 | 0,833 → 0,854 (có ý nghĩa thống kê) | −31,0 [−43,9; −18,2] | −221,2 [−319,2; −123,1] TỐT HƠN |

Khách ngoại vi được lợi nhất: chờ ít hơn 31,0 s, tỉ lệ hoàn thành tăng 0,833 → 0,854. Chuyến đón khách ở trung tâm lại chạy rỗng **thêm 145,3 m** và khách ở đó không được chờ ít hơn (+7,4 s, khoảng chứa 0). Thật dễ viết câu «ETA chuyển lợi từ trung tâm ra ngoại vi».

Nhưng đây là một dấu sao trong 26 phép so sánh (17 chỉ số và 9 ô theo vùng). Từ khoảng tin cậy suy ra t ≈ 145,3 / (102,3 / 2,093) ≈ 2,97; muốn giữ mức 5% cho cả 26 phép so sánh theo Bonferroni cần t khoảng 3,6. Dòng này không vượt ngưỡng đó, trong khi p95 (4,6), max (6,9) và ngoại vi (4,7) đều vượt. Cách kiểm tra tốt hơn là chạy lại, nên lab lặp thí nghiệm trên 40 seed chưa dùng, 20261126…20261165:

| Hiệu B − A (trên dữ liệu mô phỏng) | 20 seed đánh giá | 40 seed mới |
|---|---|---|
| `deadhead_m` trung bình | −66,2 [−112,1; −20,3] | −105,0 [−134,4; −75,6] |
| `deadhead_m` p50 | +37,8 [−25,6; 101,1] | +28,0 [−4,9; 61,0] |
| `deadhead_m` p95 | −533,0 [−774,4; −291,6] | −339,3 [−521,8; −156,9] |
| `wait_s` p95 | −75,8 [−103,7; −47,8] | −67,2 [−84,5; −49,9] |
| km rỗng tổng / khách đặt (m) | −34,6 [−80,5; 11,3] | **−69,9 [−99,2; −40,5]** |
| `deadhead_m`, khách trung tâm | **+145,3 [43,0; 247,6]** | +32,6 [−26,6; 91,8] |
| `deadhead_m`, khách vành trong | −76,7 [−145,7; −7,7] | −104,3 [−149,6; −59,0] |
| `deadhead_m`, khách ngoại vi | −221,2 [−319,2; −123,1] | −231,1 [−300,1; −162,2] |

Đuôi và ngoại vi lặp lại. Khoản phạt ở trung tâm thì không: đừng trích nó như một kết quả. Ngược lại, km rỗng tổng từ «không phân biệt được» thành có hiệu, đúng tinh thần của cảnh báo đầu tiên ở trên. Cũng đừng đọc bảng này như chứng cứ rằng trung tâm không thiệt: +32,6 m với khoảng rộng tới 91,8 m chưa loại được một khoản phạt nhỏ.

### Độ nhạy: ở đâu hiệu biến mất

Cùng so sánh, thử thêm các bộ giả định (20 seed, ghép cặp):

| Bộ giả định | `deadhead_m` trung bình | Các chỉ số khác đáng chú ý |
|---|---|---|
| `thường` | −66,2 [−112,1; −20,3] TỐT HƠN | như bảng trên |
| `dày xe` (240 xe) | −165,0 [−196,5; −133,6] TỐT HƠN | huỷ −0,0167, chuyến hoàn thành +3,8, `wait_s` max −655,5 s |
| `thưa xe` (60 xe) | −226,0 [−465,0; 13,0] không phân biệt được | `pickup_eta_s` −28,0 s TỐT HƠN; khách hết kiên nhẫn 0,0466 → 0,0302 |
| `cao điểm` (7,0 khách/phút) | +65,7 [−6,8; 138,3] không phân biệt được | km rỗng tổng/khách +64,3 m [9,6; 118,9] **XẤU HƠN**; `wait_s` p50 +8,1 s XẤU HƠN |
| `huỷ nặng` | −13,4 [−54,5; 27,6] không phân biệt được | `deadhead_m` p50 +59,7 m XẤU HƠN; huỷ −0,0210, hoàn thành +4,8 TỐT HƠN |
| `huỷ theo chờ` | cho đúng kết quả của `thường` | ghép ngay thì thời gian chờ chính là ETA, bộ này chưa thêm thông tin ở bài này |

Hiệu rõ nhất khi dư xe. Khi xe quá ít hoặc cầu quá cao, ai cũng phải chạy xa, nên chọn xe nào gần hơn gần như không đổi gì; đó là cách đọc của lab, không phải kết luận đã kiểm riêng. Ở `cao điểm` có hai dòng XẤU HƠN với cận dưới sát 0 trong 17 chỉ số: có thể là may rủi, chưa được chạy lại. Bộ `huỷ theo chờ` giống hệt `thường` không phải vì cẩu thả: với ghép ngay, thời gian chờ tính từ lúc đặt bằng ETA tính từ lúc gán, nên hai mô hình huỷ trùng nhau. Bộ này chỉ có ý nghĩa khi có cửa sổ lô, như ở bài 43.

Kết luận cho baseline của phần P05: **ở mật độ thường, ETA thay chim bay không làm chỉ số toàn hệ thống nào xấu đi một cách phân biệt được, và giá trị chính của nó là cắt đuôi.** ETA không phải thuốc chữa km rỗng trung bình. Cái giá về tính toán (ma trận ETA thay cho phép tính haversine) không đo ở bài này; [bài 44](/articles/location-cat-tia-ung-vien-h3-ma-tran-thua) đo chi phí dựng ma trận. Từ đây, mọi bài sau so với **`nearest-eta` ghép ngay** hoặc nêu rõ so với chiến lược nào.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` (hoặc thêm `--fixture` để chạy trên đồ thị tổng hợp nhỏ, chỉ để kiểm cấu trúc). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc41_gan_nhat_sai_o_dau
```

Tham số: `--seeds N` (mặc định 20), `--trials N` (mẫu cho mục 1–2, mặc định 2.000), `--jobs N`, `--ca-te` (in tuyến của ca tệ nhất, gồm phép thử mép bản đồ), `--variant thường --skip-seeds 20 --seeds 40` (lần chạy lại trên seed mới). Kết quả đầy đủ, kể cả tất cả bộ giả định và `uptime` đầu và cuối, nằm trong `examples/toi-noi-location/results/p05.md`. Trích mục 1 và 2 (HCM, đo khi máy đang chịu tải nặng, load average khoảng 300 trên 10 lõi; các số này là số đếm và tỉ lệ nên không bị ảnh hưởng):

```text
-- 1. chọn theo chim bay hay theo ETA đường bộ (mẫu 2000 khách, mỗi khách 80 tài xế rảnh ngẫu nhiên trong 120) --
  hai cách chọn KHÁC nhau ở 689/2000 khách (34.5%)
  thiệt hại ETA của «gần nhất chim bay» so với «gần nhất ETA» (giây, trên mọi khách): trung bình 27.2  p50 0.0  p95 144.9  p99 430.9  max 967.9
  ca tệ nhất trong mẫu: khách ghim (10.71574, 106.62485); chim bay chọn tài xế DR-079 cách 1315 m chim bay, ETA 1430 s; ETA chọn DR-002 cách 3169 m chim bay, ETA 462 s; thiệt 968 s (16.1 phút), tỉ lệ ETA 3.09×

-- 2. phản ví dụ: tham lam theo thứ tự đặt «cướp» tài xế của khách sau --
  tham lam theo thứ tự đặt tốn thêm ETA ở 339/2000 lô (16.9%); phần tốn thêm trong các lô đó (s): p50 38.8  p95 181.1  max 254.5
    tham lam (A chọn trước): A ← DR-055 (189 s), B ← DR-092 (313 s); tổng 502 s
    Hungarian (tối ưu cả lô): A ← DR-092 (222 s), B ← DR-055 (25 s); tổng 247 s
    chênh 255 s (102.9%): A mất +33 s, B được -287 s
```

Lab chạy hai lần, lần đầu có năm bộ giả định, lần sau thêm bộ `huỷ theo chờ`; mọi số trừ thời gian giống nhau. `verify-p05.sh` chạy lab này trên fixture (`--seeds 2 --trials 100`) để kiểm tra các bất biến, không tái lập các số trong bài; các số trên HCM với 20 seed mất vài phút đến hàng chục phút mỗi lab.

## Giới hạn của ví dụ

- **Tất cả là mô phỏng.** Nhu cầu quanh tám điểm nóng đặt tay, mô hình huỷ là công thức chưa hiệu chuẩn, tài xế không từ chối chuyến, không tự di chuyển đi tìm vùng có nhu cầu. Bài này không nói một hệ thống thật nên có thay đổi nào.
- **ETA của matcher là ETA CH theo nút**, không có chi phí rẽ, và chưa thử với giao thông tổng hợp theo giờ: CH của matcher dùng tốc độ tự do.
- **Mẫu 1 và 2 là mẫu dựng**, không phải phân phối của vận hành. Mẫu 2 cố tình tạo xung đột.
- **Chỉ 20 seed** và khoảng t giả định gần chuẩn; chỉ bộ `thường` được chạy lại trên seed mới, các bộ khác chưa.
- **Đồ thị bị cắt theo bbox.** Điểm gần mép có thể phải đi vòng do đường bị cắt, như ca tệ nhất cho thấy; ảnh hưởng tới phần còn lại của đuôi chưa được đo.
- **Chưa phân tích** vì sao ca tệ nhất vẫn vòng 11 km trên đồ thị rộng hơn.
- **Chưa so với dispatch của bất kỳ công ty nào**: không có dữ liệu để so.

## Bài tập

**Bài 1.** Trong lô ở Hình 1, thêm tài xế thứ ba DR-035 với ETA 461 s cho A và 477 s cho B. Tham lam theo thứ tự có đổi lựa chọn không? Hungarian có đổi không?

<details><summary>Lời giải bài 1</summary>
<p>Không đổi cả hai. Với tham lam, A vẫn chọn DR-055 (189 s) vì rẻ nhất, B còn DR-092 (313 s) và DR-035 (477 s) nên chọn DR-092: tổng 502 s. Hungarian cũng vẫn chọn A ← DR-092 (222), B ← DR-055 (25): tổng 247 s. Tài xế thứ ba quá xa cả hai khách nên không bao giờ nằm trong lời giải rẻ nhất. Bài học: phản ví dụ này đến từ hai tài xế đứng giữa hai khách, không đến từ việc thiếu xe.</p>
</details>

**Bài 2.** Khoảng tin cậy của `deadhead_m` trung bình ở bộ `thường` là [−112,1; −20,3] với 20 seed. Suy ra độ lệch chuẩn của các hiệu theo từng seed, rồi ước lượng cần bao nhiêu seed để nửa khoảng nhỏ đi một nửa.

<details><summary>Lời giải bài 2</summary>
<p>Nửa khoảng là (−20,3 − (−112,1)) / 2 = 45,9 m. Từ `t · sd / √20 = 45,9` với `t = 2,093` suy ra `sd ≈ 45,9 × 4,472 / 2,093 ≈ 98,1 m`. Nửa khoảng tỉ lệ với 1/√n, nên muốn giảm một nửa cần chừng gấp bốn số seed, tức khoảng 80 seed (t khi đó còn khoảng 1,99 nên nửa khoảng thực tế cỡ 21,8 m, còn nhỏ hơn một nửa một chút). Đổi lại là gấp bốn thời gian chạy. Bộ `thường` chạy lại với 40 seed mới cho khoảng ±29,4 m, hẹp hơn đúng theo quy luật đó.</p>
</details>

**Bài 3.** Mục 2 báo «16,9% lô tham lam tốn thêm». Giải thích vì sao không được trích con số này như «16,9% chuyến bị giao sai tài xế», và đề xuất cách đo đúng tần suất trong simulator.

<details><summary>Lời giải bài 3</summary>
<p>Lô được dựng có điều kiện: hai khách ghim cách nhau tối đa 1.500 m và chỉ ba tài xế gần trung điểm. Điều kiện này tạo xung đột, nên tỉ lệ là tỉ lệ trong tập lô đã chọn, không phải trên mọi chuyến. Cách đo đúng: trong simulator, mỗi lần chiến lược được gọi, ghi số khách chờ và số xe rảnh, rồi đếm bao nhiêu lần có từ hai khách chờ trở lên mà tham lam theo thứ tự cho tổng ETA lớn hơn Hungarian trên chính lô đó. Với ghép ngay và 120 xe, lần gọi có hai khách trở lên không xảy ra (bài 45 đếm được 0 trong 4.568 lô); muốn có lô thật cần cửa sổ, như cửa sổ 10 s ở bài 42 cho trung bình 1,64 khách mỗi lô.</p>
</details>

Bài trước: [bài 40](/articles/location-cuoc-quote-giai-thich-doi-diem-don-cho-khach). Bài tiếp: [bài 42](/articles/location-ghep-theo-lo-hungarian-kuhn-munkres-rust).

## Tài liệu gốc

- [Student's t-test (Wikipedia)](https://en.wikipedia.org/wiki/Student%27s_t-test): mục kiểm định cho mẫu ghép cặp (dependent samples), nền của khoảng tin cậy trên hiệu từng seed.
- [Variance reduction (Wikipedia)](https://en.wikipedia.org/wiki/Variance_reduction): mục «common random numbers», lý do cùng seed giúp so sánh hai chiến lược.
- [Hungarian algorithm (Wikipedia)](https://en.wikipedia.org/wiki/Hungarian_algorithm): thuật toán phân công tối ưu mà bài 42 dùng cho phép gán cả lô.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho dữ liệu OSM của đồ thị HCM.
