---
slug: location-phia-duong-huong-tiep-can-tranh-quay-dau
title: "Phía đường và hướng tiếp cận: bớt một vòng quay đầu"
summary: "Bài 34: ứng viên có hướng, phạt phía đường, chi phí quay đầu, xếp hạng hai tầng vì ma trận không có chi phí rẽ (đo ma trận sai bao nhiêu). Phạt phía đường chỉ có cơ chế, simulator không đo được lợi ích của nó."
category: location-routing
level: advanced
tags: [toi-noi, location, rust, pickup, u-turn, side-of-road, turn-cost, ranking]
language: vi
---

Một đại lộ có dải phân cách giữa. Tài xế đang chạy về phía đông, khách đứng ở vỉa hè phía nam. Điểm đón A nằm ở làn đi về đông, sát chỗ khách: xe tới, dừng, khách bước lên từ bên tay phải. Điểm đón B cách đó 60 m, ở làn đi về tây: khách phải băng cả mặt đường, và tài xế phải đi hết đoạn, tìm lối qua dải phân cách, quay đầu rồi quay lại. Trên bản đồ A và B gần như cùng một chỗ. Trong đời thực chúng khác nhau cỡ một vòng quay đầu và một lần băng qua đường lớn. Bài này làm cho `J` thấy được sự khác biệt đó, và nói thẳng phần nào của nó đo được, phần nào không.

Đây là bài 34 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dựng trên đồ thị theo cạnh có chi phí rẽ của [bài 23](/articles/location-chi-phi-re-u-turn-do-thi-theo-canh), ma trận many-to-many của [bài 26](/articles/location-ma-tran-many-to-many-bucket-ch) và danh sách ứng viên của [bài 33](/articles/location-sinh-ung-vien-diem-don-do-thi-di-bo). Mọi số đo là **trên dữ liệu mô phỏng**: kịch bản `hcm-baseline`, đồ thị TP.HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors), chính sách `p04-v1` (a = b = c = d = 1). Phần 1 và 2 đo trên 120 yêu cầu của seed 20270001; phần 3 trên 20 seed đánh giá 20270001–20270020 (so sánh cặp, như giải thích ở [bài 32](/articles/location-ham-muc-tieu-diem-hen-khach-va-tai-xe)).

**Hãy đọc bài này như một bài về cơ chế.** Phạt phía đường là số hạng của `J` mà simulator không thể đo lợi ích, vì simulator không có mô hình băng đường. Bài không chứng minh nó đáng giá; nó chỉ cho thấy nó được cài thế nào và cái giá đo được.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ma trận chi phí many-to-many của bài 26 trả lời «tài xế tới điểm đón mất bao lâu» rất nhanh. Nếu dùng nó một mình để chọn ứng viên có J nhỏ nhất trong 120 yêu cầu, theo bạn trong bao nhiêu phần trăm yêu cầu nó chọn đúng ứng viên mà tính chính xác cũng chọn?</p>
<details><summary>Xem đáp án</summary><p>40,0%. Ma trận và CH tính theo nút và không có chi phí rẽ, nên nó không thấy việc quay đầu giữa đường bị cấm hay tốn 20 giây. Số hạng tài xế bị đánh thấp trung bình 49,8 giây, và chọn sai trung bình mất 19,4 giây giá trị J (p95 87,9 giây). Chấm lại 4 ứng viên đầu cộng điểm ghim bằng A* theo cạnh đưa tỉ lệ lên 82,5%.</p></details>
</div>

## Một vị trí, hai ứng viên

Một cạnh đường hai chiều là hai cạnh có hướng. Ứng viên điểm đón vì thế là **vị trí có hướng**: cùng một tọa độ nhưng xe vào theo chiều nào thì khách ở bên phải hay bên trái của hướng xe. Xe ở Việt Nam chạy bên phải, nên khách ở bên phải hướng xe thì bước lên xe không phải băng đường, còn khách ở bên trái thì phải băng qua cả mặt đường. Hàm `side_of` quy mối quan hệ đó về một dấu: chiếu vị trí khách lên pháp tuyến bên phải của hướng cạnh.

`examples/toi-noi-location/rust/crates/pickup/src/candidates.rs`

```rust
/// Khách `cust` ở phía nào của cạnh có hướng `e` tại `frac`.
pub fn side_of(engine: &Engine, e: EdgeId, frac: f32, cust: LatLng) -> (Side, f64) {
    let p = point_on_edge(&engine.g, e, frac);
    let d = direction_at(engine, e, frac);
    let c = LocalProjection::new(p).to_xy(cust);
    // pháp tuyến bên phải của hướng đi (d.y, -d.x)
    let lat_right = c[0] * d[1] - c[1] * d[0];
    let side = if lat_right.abs() < CENTRE_M {
        Side::Centre
    } else if lat_right > 0.0 {
        Side::Right
    } else {
        Side::Left
    };
    (side, lat_right)
}
```

Nếu khách cách tim đường dưới 4 m (`CENTRE_M`) thì không phân biệt được phía và không phạt. Còn lại, phía trái bị phạt, và mức phạt phụ thuộc cấp đường: băng qua đường lớn nguy hiểm và mất thời gian, băng qua hẻm thì không đáng kể.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Hình 1. Sơ đồ minh họa, không phải kết quả đo: đường hai chiều, xe chạy bên phải. Khách đứng ở vỉa hè phía nam. Ứng viên A ở làn chạy về đông có khách bên tay phải, không phạt; ứng viên B ở làn chạy về tây có khách bên tay trái, bị phạt phía đường 30 s ở đường lớn, và tài xế đi từ phía tây còn phải quay đầu."><defs><marker id="l34a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs><rect x="20" y="20" width="840" height="360" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Phía đường</text><text x="40.0" y="70.0" class="t-small">Sơ đồ minh họa (không phải kết quả đo). Số phạt lấy từ Penalties::default.</text><rect x="60.0" y="150.0" width="760.0" height="40.0" rx="0" class="f-surface s-line" stroke-width="1"/><rect x="60.0" y="190.0" width="760.0" height="40.0" rx="0" class="f-surface s-line" stroke-width="1"/><path d="M60.0 190.0 L820.0 190.0" class="f-none s-muted dash" stroke-width="1.5"/><text x="70.0" y="144.0" class="t-small">vỉa hè phía bắc, làn chạy về tây (←)</text><text x="70.0" y="244.0" class="t-small">làn chạy về đông (→), vỉa hè phía nam</text><circle cx="430.0" cy="272.0" r="8" class="f-phys s-phys" stroke-width="1.5"/><text x="444.0" y="276.0" class="t-b">khách đứng</text><rect x="560.0" y="198.0" width="24.0" height="24.0" rx="4" class="f-pk" stroke-width="0"/><text x="572.0" y="214.0" class="t-b">A</text><rect x="560.0" y="158.0" width="24.0" height="24.0" rx="4" class="f-warn" stroke-width="0"/><text x="572.0" y="174.0" class="t-b">B</text><circle cx="110.0" cy="205.0" r="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="100.0" y="178.0" class="t-small">tài xế chạy về đông</text><path d="M120.0 202.0 L552.0 202.0" class="f-none s-pk" stroke-width="2" marker-end="url(#l34a)"/><path d="M120 224 L720 224 L720 170 L592 170" class="f-none s-warn dash" stroke-width="2" marker-end="url(#l34a)"/><text x="620.0" y="130.0" class="t-small">quay đầu: 20 s trong lab,</text><text x="620.0" y="144.0" class="t-small">hoặc phải vòng qua lối dải phân cách</text><path d="M438.0 264.0 L560.0 226.0" class="f-none s-phys" stroke-width="1.5"/><path d="M440 262 L566 184" class="f-none s-warn dash" stroke-width="1.5"/><text x="40.0" y="306.0" class="t-small">A: khách ở bên phải của hướng xe, lên xe không băng đường, phạt phía đường 0 s.</text><text x="40.0" y="324.0" class="t-small">B: khách ở bên trái, phải băng cả mặt đường, phạt +30 s nhân hệ số cấp đường.</text><text x="40.0" y="342.0" class="t-small">Hệ số cấp đường (side_scale): đường lớn 1,0; tertiary 0,5; đường nhỏ 0 (xem hàm bên dưới).</text></svg>
</div>
<figcaption>Hình 1. Sơ đồ minh họa, không phải kết quả đo. Đường hai chiều, xe chạy bên phải, khách đứng ở vỉa hè phía nam. Ứng viên A ở làn chạy về đông: khách ở bên phải hướng xe, lên xe không băng đường, không phạt. Ứng viên B ở làn chạy về tây: khách ở bên trái, phải băng mặt đường, bị phạt phía đường 30 s ở đường lớn, và tài xế đi từ phía tây còn phải quay đầu. Con số phạt lấy từ Penalties::default.</figcaption>
</figure>

Phạt phía đường là một khoản cộng vào `J`, không phải ràng buộc cứng. Hệ số theo cấp đường và khoản phạt gộp trong `penalty`:

`examples/toi-noi-location/rust/crates/pickup/src/rank.rs`

```rust
/// Hệ số phạt phía đường theo cấp đường: đường lớn băng khó, hẻm thì không đáng kể.
pub fn side_scale(class: HighwayClass, lanes: u8) -> f64 {
    use HighwayClass::*;
    match class {
        Trunk | Primary | Secondary | TrunkLink | PrimaryLink | SecondaryLink => 1.0,
        Tertiary | TertiaryLink => 0.5,
        _ if lanes >= 3 => 1.0,
        _ if lanes == 2 => 0.5,
        _ => 0.0,
    }
}
```

`examples/toi-noi-location/rust/crates/pickup/src/rank.rs`

```rust
    fn penalty(&self, engine: &Engine, c: &Candidate, walk_m: f64) -> f64 {
        let p = &self.weights.penalties;
        let mut s = self.weights.tail_walk_penalty(walk_m);
        if self.cfg.use_side && c.side == Side::Left {
            let lanes = engine.g.lanes[c.pos.edge as usize];
            s += p.wrong_side_s * side_scale(c.class, lanes);
        }
        if self.cfg.profile == Profile::Car && c.flags & flag::ALLEY != 0 {
            s += p.alley_stop_s;
        }
        s
    }
```

Mọi số này là **giả định của lab**, chưa hiệu chỉnh: `wrong_side_s` = 30 s nhân hệ số cấp đường (1,0 cho trunk, primary, secondary hoặc đường từ ba làn; 0,5 cho tertiary hoặc đường hai làn; 0 cho đường nhỏ), 20 s cho dừng trong hẻm, và phạt đi bộ vượt 150 m. Ba khoản này cộng thẳng vào `J` mà không có đo đạc nào đứng sau.

## Quay đầu: vì sao ma trận một mình sai

Chi phí quay đầu nằm ở đồ thị theo cạnh. Cấu hình đô thị của lab: rẽ phải 3 s, rẽ trái 6 s, qua lối dải phân cách 4 s, quay đầu 20 s, và quay đầu giữa đường chỉ được phép ở đường cụt (còn lại phải dùng lối dải phân cách hoặc vòng qua đường nối):

`examples/toi-noi-location/rust/crates/route-engine/src/turn.rs`

```rust
    pub fn urban() -> Self {
        Self {
            straight_ms: 0,
            right_ms: 3_000,
            left_ms: 6_000,
            u_turn_ms: 20_000,
            median_ms: 4_000,
            u_turn_policy: UTurnPolicy::DeadEndOnly,
            straight_deg: 35.0,
            reversal_deg: 150.0,
        }
    }
```

Nhưng CH và ma trận many-to-many là theo **nút** và không có chi phí rẽ. Chúng không thấy rằng một xe đang chạy ngược phía phải quay đầu mất 20 giây hoặc bị cấm. Chỉ A\* theo cạnh có chi phí rẽ, và nó đắt hơn. Đó là lý do xếp hạng hai tầng:

- **Tầng 1:** ma trận rẻ cho mọi ứng viên, tính `J1`, cắt xuống `refine_top` ứng viên.
- **Tầng 2:** chỉ ứng viên đứng đầu và điểm ghim (luôn luôn) được chấm lại bằng A\* theo cạnh, ra `J2`, con số mà simulator thực thi.

`examples/toi-noi-location/rust/crates/pickup/src/rank.rs`

```rust
        // tầng 2
        let mut all: Vec<Scored> = Vec::new();
        let mut rest: Vec<Scored> = Vec::new();
        if self.cfg.stage2 {
            let tr = Instant::now();
            let (pins, others): (Vec<Indexed>, Vec<Indexed>) =
                stage1.into_iter().partition(|s| s.1.cand.is_pin);
            let mut order: Vec<Indexed> = pins;
            let mut n_other = 0;
            let mut tail: Vec<Scored> = Vec::new();
            for s in others {
                if n_other < self.cfg.refine_top {
                    order.push(s);
                    n_other += 1;
                } else {
                    tail.push(s.1);
                }
            }
```

### Ma trận một mình sai bao nhiêu

Lab đo thẳng. Trên 120 yêu cầu của seed 20270001 (mỗi yêu cầu 56,7 ứng viên hợp lệ, gồm hai chiều của điểm ghim; 120 tài xế rảnh), «đáp án đúng» là chấm A\* theo cạnh **mọi** ứng viên. Rồi so với các cách rẻ hơn:

| Cách chọn | Chọn đúng đáp án | Hối tiếc TB (s) | p95 (s) | max (s) | Tuyến A\*/yêu cầu |
|---|---:|---:|---:|---:|---:|
| chỉ ma trận | 40,0% | 19,39 | 87,89 | 243,69 | 0 |
| top 1 + ghim | 50,8% | 15,14 | 68,73 | 239,26 | 9 |
| top 2 + ghim | 65,8% | 10,02 | 49,77 | 239,26 | 12 |
| top 4 + ghim (mặc định) | 82,5% | 4,67 | 24,20 | 239,26 | 18 |
| top 8 + ghim | 92,5% | 0,69 | 4,38 | 30,15 | 30 |
| top 16 + ghim | 99,2% | 0,08 | 0,00 | 9,14 | 54 |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 370" role="img" aria-label="Hình 2. Ma trận một mình chọn đúng ứng viên có J nhỏ nhất ở 40,0% trong 120 yêu cầu; chấm lại bằng A* theo cạnh top 4 cùng điểm ghim lên 82,5%, top 8 lên 92,5%, top 16 lên 99,2%. Mô phỏng, seed 20270001."><rect x="20" y="20" width="840" height="330" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Hai tầng: chấm lại bao nhiêu là đủ</text><text x="40.0" y="76.0" class="t-small">Tỉ lệ chọn đúng ứng viên có J chính xác nhỏ nhất (120 yêu cầu, seed 20270001)</text><text x="640.0" y="76.0" class="t-small">hối tiếc TB</text><text x="740.0" y="76.0" class="t-small">tuyến A*/yêu cầu</text><text x="40.0" y="106.0" class="t-b">chỉ ma trận</text><rect x="160.0" y="92.0" width="160.0" height="22.0" rx="2" class="f-dead" stroke-width="0"/><text x="326.0" y="108.0">40,0%</text><text x="640.0" y="108.0" class="t-mono">19,39 s</text><text x="780.0" y="108.0" class="t-mono">0</text><text x="40.0" y="144.0" class="t-b">top 1 + ghim</text><rect x="160.0" y="130.0" width="203.2" height="22.0" rx="2" class="f-dead" stroke-width="0"/><text x="369.2" y="146.0">50,8%</text><text x="640.0" y="146.0" class="t-mono">15,14 s</text><text x="780.0" y="146.0" class="t-mono">9</text><text x="40.0" y="182.0" class="t-b">top 2 + ghim</text><rect x="160.0" y="168.0" width="263.2" height="22.0" rx="2" class="f-dead" stroke-width="0"/><text x="429.2" y="184.0">65,8%</text><text x="640.0" y="184.0" class="t-mono">10,02 s</text><text x="780.0" y="184.0" class="t-mono">12</text><text x="40.0" y="220.0" class="t-b">top 4 + ghim</text><rect x="160.0" y="206.0" width="330.0" height="22.0" rx="2" class="f-pk" stroke-width="0"/><text x="496.0" y="222.0">82,5%</text><text x="640.0" y="222.0" class="t-mono">4,67 s</text><text x="780.0" y="222.0" class="t-mono">18</text><text x="40.0" y="258.0" class="t-b">top 8 + ghim</text><rect x="160.0" y="244.0" width="370.0" height="22.0" rx="2" class="f-dead" stroke-width="0"/><text x="536.0" y="260.0">92,5%</text><text x="640.0" y="260.0" class="t-mono">0,69 s</text><text x="780.0" y="260.0" class="t-mono">30</text><text x="40.0" y="296.0" class="t-b">top 16 + ghim</text><rect x="160.0" y="282.0" width="396.8" height="22.0" rx="2" class="f-dead" stroke-width="0"/><text x="562.8" y="298.0">99,2%</text><text x="640.0" y="298.0" class="t-mono">0,08 s</text><text x="780.0" y="298.0" class="t-mono">54</text><text x="40.0" y="335.0" class="t-small">Cột xanh: cấu hình mặc định của lab (top 4). Hối tiếc = J chính xác của ứng viên được chọn trừ J chính xác nhỏ nhất.</text></svg>
</div>
<figcaption>Hình 2. Tỉ lệ chọn đúng ứng viên có J chính xác nhỏ nhất theo số ứng viên được chấm lại bằng A* theo cạnh, trên 120 yêu cầu của seed 20270001, đồ thị TP.HCM. Hối tiếc là J chính xác của ứng viên được chọn trừ J chính xác nhỏ nhất. Cột xanh là cấu hình mặc định (top 4 cộng điểm ghim). Trên dữ liệu mô phỏng.</figcaption>
</figure>

Nguyên nhân thấy rõ ở sai số từng số hạng (6.733 cặp ứng viên đã chấm lại): số hạng tài xế bị ma trận **đánh thấp trung bình 49,8 s** (p5 +11,7 s, p50 +46,7 s, p95 +101,6 s, max +201,6 s); số hạng đoạn có khách đánh thấp trung bình 60,9 s (p95 +112,3 s, max +264,5 s). Chỉ 4 trong 13.466 số hạng là A\* *rẻ hơn* ma trận quá 1 s, đều là trường hợp đích cùng cạnh, phía trước xe: ma trận phải vòng qua nút, A\* đi thẳng. Vậy ma trận **gần như luôn lạc quan**, vì nó bỏ cả chi phí rẽ (3 s, 6 s) lẫn quay đầu, và mức lạc quan khác nhau giữa các ứng viên, nên xếp hạng bị đảo.

Ở đây không có bữa trưa miễn phí: top 4 chọn đúng 82,5% với hối tiếc 4,7 s, và 18 tuyến A\* mỗi yêu cầu; top 8 chọn đúng 92,5% với 30 tuyến. Chi phí độ trễ của các lựa chọn đó là chủ đề của [bài 35](/articles/location-xep-hang-diem-don-one-to-many-ngan-sach-do-tre).

## Hướng tiếp cận và hai chiều của một vị trí

Khi ứng viên tốt nhất là một vị trí trên đường hai chiều, simulator cần biết xe được phép vào theo chiều nào. `JPickup` đưa chiều tốt nhất vào danh sách `approach`, và thêm chiều còn lại **nếu `J` của nó hơn kém không quá 20 giây** (`commit_margin_s`): nếu hai chiều gần như ngang nhau thì để tài xế chọn chiều nào tới sớm hơn thay vì ép một chiều.

`examples/toi-noi-location/rust/crates/pickup/src/strategy.rs`

```rust
        // chiều thứ hai của cùng vị trí, nếu `J` không tệ hơn quá `commit_margin_s`
        let g = &engine.g;
        let mut approach = vec![best.cand.pos];
        let tw = g.twin[best.cand.pos.edge as usize];
        if tw != road_graph::model::NONE
            && let Some(o) = ranked.all.iter().find(|s| s.cand.pos.edge == tw)
            && o.j - best.j <= self.cfg.commit_margin_s
        {
            approach.push(o.cand.pos);
        }
```

Hướng của tài xế trong simulator là một vị trí có hướng (cạnh, tỉ lệ dọc cạnh), thay cho hướng mà [bài 16](/articles/location-map-matching-hmm-viterbi-rust) và [bài 17](/articles/location-map-matching-online-cua-so-cau-vuot-duong-song-song) sẽ cho từ map matching trong hệ thống thật; lab không đo độ nhiễu hướng từ GPS thật.

Đếm trực tiếp quay đầu trên tuyến của tài xế gần nhất theo chim bay tới khách, so điểm ghim với điểm chọn (118 yêu cầu có đủ tuyến, một seed, không có khoảng tin cậy): **96 lần quay đầu tới điểm ghim, 88 lần tới điểm chọn** (đi ngược trực tiếp 20 so với 22; qua lối dải phân cách 25 so với 24; loại «khác» 51 so với 42). Thời gian tài xế tới điểm hẹn trung bình 224,8 s so với 207,5 s. Chênh này nhỏ, đo trên một seed và 118 yêu cầu, và không được kiểm định.

## Thí nghiệm trên simulator

Năm biến thể, mỗi cái so với baseline (đón đúng ghim, tài xế gần nhất theo chim bay) trên dữ liệu mô phỏng, 20 seed đánh giá 20270001–20270020, kịch bản `hcm-baseline`. Trong ngoặc là khoảng tin cậy t 95% của chênh cặp; cột `walk_m` p50 là chênh cặp, mét:

| Biến thể | `deadhead_m` TB | `u_turns` mỗi chuyến | `walk_m` p50 | `cancel_rate` |
|---|---:|---:|---:|---:|
| (a) J đầy đủ, hai tầng, phạt phía đường | −107,0 [−151,6; −62,4] | 1,6 → 1,3 | +26,5 | −0,0141 [−0,0210; −0,0072] |
| (b) bỏ phạt phía đường | −125,3 [−175,7; −74,8] | 1,6 → 1,3 | +30,6 | −0,0129 [−0,0198; −0,0061] |
| (c) chỉ ma trận, không A\* theo cạnh | −95,9 [−140,0; −51,8] | 1,6 → 1,4 | +20,2 | −0,0109 [−0,0174; −0,0045] |
| (d) chỉ một hướng tiếp cận | −108,2 [−159,1; −57,2] | 1,6 → 1,3 | +26,6 | −0,0144 [−0,0211; −0,0076] |
| (e) J giả định sai bộ ghép | −70,5 [−111,6; −29,3] | 1,6 → 1,4 | +18,2 | −0,0053 [−0,0114; +0,0007] |

Ba điều đọc được, và một điều không đọc được.

**Phạt phía đường làm mất khoảng 18 m km rỗng, để đổi lấy lợi ích mà simulator không đo.** So (a) với (b): bỏ phạt cho `deadhead_m` −125,3 m thay vì −107,0 m, chênh 18,3 m. Đây là chênh của hai *ước lượng*, các khoảng tin cậy chồng nhau, và hai cấu hình chưa được so trực tiếp với nhau, nên «18 m» là một thứ tự độ lớn chứ không phải kết quả kiểm định. Điều khách được lợi, tức khỏi băng đường lớn, không hiện ra ở chỉ số nào: **simulator không tính thời gian băng đường**. Gần như chỉ có chi phí hiện ra. Một dấu hiệu gián tiếp: ở (a) `walk_m` p50 thấp hơn (b) khoảng 4 m (+26,5 so với +30,6), tức phạt có thể hơi kéo ứng viên về phía khách; cũng chưa kiểm định trực tiếp.

**Không thấy phạt phía đường làm giảm quay đầu.** `u_turns` từ 1,6 xuống 1,3 cả ở (a) lẫn (b). Giả thuyết: phần giảm đến từ việc chọn điểm gần tài xế hơn và để A\* theo cạnh thấy chi phí quay đầu thật; lab không tách riêng hai phần này. Một dấu hiệu ủng hộ: bỏ A\* và chỉ dùng ma trận (c) thì `u_turns` còn 1,4 và `deadhead_m` kém hơn (a) cỡ 11 m (−95,9 so với −107,0), cũng là chênh của hai ước lượng, chưa kiểm định.

**Cam kết một hướng không thêm gì cho bộ ghép «gần nhất theo chim bay».** (d) gần như trùng (a): `deadhead_m` −108,2 m so với −107,0 m. Danh sách hai hướng không giúp, ít nhất với bộ ghép này.

**J phải khớp bộ ghép phía sau.** Ở (e), nếu J giả định bộ ghép chọn tài xế nhanh nhất (`BestTime`) trong khi bộ ghép thật chọn tài xế gần nhất theo chim bay, `deadhead_m` chỉ −70,5 m và `cancel_rate` **không khác biệt rõ** (khoảng tin cậy chứa 0). Một mô hình sai về người thực thi phía sau có vẻ làm mất khoảng một phần ba lợi ích của (a) (−70,5 so với −107,0 m, chưa so trực tiếp), trong khi khách lại đi bộ ít hơn (`walk_m` p50 +18,2 so với +26,5 m).

Một điều lab **không** đọc được: lợi ích của phạt phía đường cho khách. Khách khỏi băng đường lớn là một lợi ích thật nếu nó thật, nhưng chúng ta đã đặt giá cho nó bằng một con số 30 s đoán, và simulator không có gì để xác nhận hay bác bỏ. Quyết định giữ hay bỏ khoản phạt này là chính sách phải dựa trên thứ khác (số vụ băng đường nguy hiểm, khảo sát khách), không phải trên bảng này.

**Ai được lợi, ai chịu thiệt.** Tài xế được lợi ở `u_turns` (1,6 → 1,3) và `deadhead_m`; khách chờ xe được lợi ở ETA và hủy. Khách điển hình chịu thiệt ở `walk_m` p50 (+26,5 m với (a)). Khoản phạt phía đường đẩy thêm cỡ 18 m km rỗng sang tài xế (chênh của hai ước lượng) để đổi lấy lợi ích cho khách mà chưa đo được; nếu lợi ích đó không có thật, tài xế đang trả cho nó mà không ai nhận.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` (hoặc `--fixture`). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run --release -q --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc34_phia_duong -- --jobs 6
```

Cấu hình: Apple Silicon, `--release`, `--jobs 6`; phần 1–2 trên 120 yêu cầu và 120 tài xế rảnh của seed 20270001; phần 3 trên 20 seed đánh giá. **Máy đang chịu tải nặng** (load averages 324,57 331,86 359,44 lúc bắt đầu), nên bài không dùng số thời gian; số đếm và tỉ lệ tái lập được. Output (rút gọn):

```text
-- 1. ma trận một mình sai bao nhiêu so với chấm lại bằng A* theo cạnh --
     A* chấm lại chọn đúng đáp án  hối tiếc TB (s)        p95 (s)     max (s) tuyến A*/yêu cầu
  0 (chỉ ma trận)          40.0%            19.39          87.89      243.69              0
    top 4 + ghim           82.5%             4.67          24.20      239.26             18
    top 8 + ghim           92.5%             0.69           4.38       30.15             30
  sai số số hạng tài xế (A* theo cạnh trừ ma trận, giây): trung bình +49.8  p5 +11.7  p50 +46.7  p95 +101.6  max +201.6  (6733 cặp ứng viên đã chấm lại)
-- 2. quay đầu trên tuyến tài xế: điểm ghim so với điểm chọn (tài xế gần nhất theo chim bay) --
  118 yêu cầu: quay đầu trên tuyến tài xế tới điểm ghim = 96 (đi ngược trực tiếp 20, qua lối dải phân cách 25, khác 51); tới điểm chọn = 88 (đi ngược trực tiếp 22, qua lối dải phân cách 24, khác 42)
-- 3. trên simulator: phía đường và hướng tiếp cận (so sánh cặp với baseline, seed ĐÁNH GIÁ) --
  (a) J đầy đủ ...   deadhead_m trung bình   2357.0   2250.0   -107.0   [-151.6, -62.4]   17/3
  (b) bỏ phạt phía đường  deadhead_m trung bình   2357.0   2231.8   -125.3   [-175.7, -74.8]   18/2
```

## Giới hạn của ví dụ

- **Phạt phía đường không đo được lợi ích** trong simulator (không có mô hình băng đường), và các khoản phạt 30 s, 20 s là đoán. Bài chỉ cho thấy cơ chế và cái giá.
- **Các so sánh giữa biến thể là chênh của hai ước lượng**, các khoảng tin cậy chồng nhau; lab chưa so trực tiếp (a) với (b), (c), (d).
- **Phần 1 và 2 chỉ một seed, 120 yêu cầu**, không có khoảng tin cậy; đếm quay đầu là tổng số lần trên các tuyến chứ không phải kiểm định.
- **Ma trận và CH vẫn là theo nút**; hai tầng là cách vòng qua chứ không giải quyết tận gốc (một CH theo cạnh có thể thay thế, lab không làm).
- **J phải khớp bộ ghép:** `DriverModel::NearestCrow` mô hình hóa đúng bộ ghép baseline; P05 đổi bộ ghép thì phải đổi giả định này.
- Hướng tài xế trong simulator là hướng thật của vị trí có hướng, không phải hướng suy ra từ GPS qua map matching.

## Bài tập

**Bài 1.** Từ bảng thí nghiệm, tính chênh `deadhead_m` giữa (a) và (b), giải thích vì sao nó không đủ để kết luận «phạt phía đường làm mất 18 m».

<details><summary>Lời giải bài 1</summary>
<p>(b) − (a) = −125,3 − (−107,0) = −18,3 m. Hai khoảng tin cậy [−175,7; −74,8] và [−151,6; −62,4] chồng nhau rộng, và chênh giữa hai ước lượng có phương sai riêng. Muốn kết luận phải so cặp trực tiếp (b) với (a) trên cùng 20 seed (lấy hiệu từng seed rồi tính khoảng tin cậy), điều lab chưa làm. Vì vậy 18 m chỉ là thứ tự độ lớn.</p>
</details>

**Bài 2.** Vì sao một mô hình tính J theo tài xế nhanh nhất (e) làm giảm lợi ích khi bộ ghép thật chọn tài xế gần nhất theo chim bay? Cho một ví dụ cụ thể về cặp ứng viên bị xếp ngược.

<details><summary>Lời giải bài 2</summary>
<p>J chấm điểm với một tài xế giả định sẽ tới điểm hẹn, nhưng người thực sự tới là tài xế mà bộ ghép chọn. Ví dụ (tưởng tượng, không lấy từ output): ứng viên P gần một tài xế X theo chim bay nhưng X ở bên kia dải phân cách (ETA thật lớn); ứng viên Q xa X hơn một chút nhưng cùng phía đường. Bộ ghép «gần nhất theo chim bay» gán X cho P, nên P tốn nhiều km rỗng, còn mô hình «nhanh nhất» tưởng một tài xế khác sẽ tới P nhanh nên cho P một J rất thấp. J chọn P, và bộ ghép thật làm tốn km rỗng hơn dự tính.</p>
</details>

**Bài 3.** Đề xuất một cách đo lợi ích của phạt phía đường mà simulator hiện tại không làm được, và nêu một giả định bạn phải thêm vào.

<details><summary>Lời giải bài 3</summary>
<p>Thêm vào simulator một đoạn «băng đường» cho khách ở phía trái: thời gian băng tăng theo cấp đường và số làn, và có thể kéo theo xác suất hủy hoặc rủi ro, rồi so J có và không có phạt phía đường trên 20 seed. Giả định cần thêm: thời gian băng đường theo cấp đường (số giây), và có thể cả tương quan giữa băng đường với hủy chuyến; hai thứ đó đều không có trong dữ liệu của lab, nên kết quả sẽ phụ thuộc hoàn toàn vào giả định. Nguồn dữ liệu thật (đo thực địa, số liệu tai nạn) cần thiết nếu muốn kết luận ngoài mô phỏng.</p>
</details>

Bài trước: [bài 33](/articles/location-sinh-ung-vien-diem-don-do-thi-di-bo). Bài tiếp: [bài 35](/articles/location-xep-hang-diem-don-one-to-many-ngan-sach-do-tre).

## Tài liệu gốc

- [Contraction hierarchies (Wikipedia)](https://en.wikipedia.org/wiki/Contraction_hierarchies): tổng quan phương pháp, nền của ma trận theo nút mà bài này chỉ ra giới hạn.
- [Pareto front (Wikipedia)](https://en.wikipedia.org/wiki/Pareto_front): đánh đổi giữa hai bên khi chỉ số mới có người được người mất.
