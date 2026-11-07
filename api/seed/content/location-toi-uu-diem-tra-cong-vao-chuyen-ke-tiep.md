---
slug: location-toi-uu-diem-tra-cong-vao-chuyen-ke-tiep
title: "Tối ưu điểm trả: gần đích thật, đúng phía, dễ bắt chuyến kế, và vì sao số đo nhỏ"
summary: "Bài 38: hàm mục tiêu cho điểm trả với chi phí thóat của tài xế. Riêng điểm trả không có ý nghĩa thống kê (km rỗng −16,3 m [−51,5; +18,9]) và khách đi thêm 10,6 m ở cuối chuyến. Trên dữ liệu mô phỏng."
category: location-routing
level: advanced
tags: [toi-noi, location, rust, dropoff, objective, simulation, deadhead, paired-evaluation]
language: vi
---

Khách xuống xe ở một con đường lớn có dải phân cách cứng. Đích của anh là căn nhà bên kia đường. Tài xế trả khách đúng điểm đích đã snap lên đường, nghĩa là ở làn bên này, nên anh phải đi qua cầu vượt dành cho người đi bộ. Hoặc ngược lại: tài xế trả bên kia đường, rồi mất cả một vòng quay đầu để quay ra, trong khi khách hàng kế tiếp đang đứng chờ ở hướng ngược lại. Điểm trả trông như phần dễ nhất của chuyến, nhưng nó là chỗ mà hai bên lại giành nhau: khách muốn xuống gần cửa, tài xế muốn thóat ra thuận hướng để bắt chuyến kế.

Kết luận của bài này khiêm tốn như bài 36 và 37: **riêng điểm trả không cải thiện km rỗng đến mức chứng minh được**, và khách phải đi bộ thêm ở cuối chuyến. Bài nói rõ vì sao, và giá trị của ý tưởng, nếu có, nằm ở đâu mà mô hình này có thể không nhìn thấy.

Đây là bài 38 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng hàm mục tiêu của [bài 32](/articles/location-ham-muc-tieu-diem-hen-khach-va-tai-xe), xếp hạng ứng viên của [bài 35](/articles/location-xep-hang-diem-don-one-to-many-ngan-sach-do-tre) và dữ liệu cổng OSM đã đo ở [bài 37](/articles/location-hem-chung-cu-tttm-san-bay-cong-sanh) (hàm `nearest_gate` đọc thẳng nút `entrance=*`, không qua lớp cổng của bài đó). Phương pháp so sánh cặp theo 20 seed đánh giá (20270001…20270020), khoảng tin cậy t 95% được định nghĩa ở bài 32. Mọi số trong bài là **trên dữ liệu mô phỏng**, kịch bản `hcm-baseline`, đồ thị HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors), baseline là đón đúng ghim cộng tài xế gần nhất theo chim bay, trả đúng điểm đích đã snap. Simulator không dự đoán thế giới thật.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn thêm một hàm mục tiêu riêng cho điểm trả (khách đi bộ tới đích thật, xe tới điểm trả, tài xế thóat ra thuận hướng) rồi chạy 20 seed ghép cặp, giữ nguyên điểm đón ở ghim. Theo bạn km rỗng trung bình thay đổi thế nào, và khoảng tin cậy có loại được 0 không?</p>
<details><summary>Xem đáp án</summary><p>Km rỗng trung bình đổi <b>−16,3 m</b> với khoảng tin cậy t 95% <b>[−51,5; +18,9]</b>: chứa 0, tức không khác biệt rõ. Bù lại khách đi bộ từ điểm trả tới đích thật thêm <b>+10,6 m [+9,0; +12,2]</b>, và con số này loại được 0. Ở thí nghiệm này, bên chịu thiệt có khoảng tin cậy loại 0, bên được lợi thì không; điều đó không có nghĩa là tài xế không được lợi gì.</p></details>
</div>

## Hàm mục tiêu điểm trả: cùng dạng, đổi vai

Bài 32 đặt khách và tài xế lên một cân cho điểm đón. Điểm trả dùng cùng ý nhưng đổi vai: khách không còn đi bộ **tới** điểm hẹn mà đi bộ **từ** điểm trả tới đích thật, còn tài xế đã có khách nên chi phí «km rỗng» thành chi phí **thóat ra** để bắt chuyến kế.

```text
J_trả(p) = a·T_bộ(p → đích thật hoặc cổng) + d·T_trip(xe → p) + e·T_thoát(p) + phạt(phía đường, đi bộ quá xa)
```

Trong lab, `T_trip` là thời gian xe tới `p` (đoạn có khách), `T_thoát` là thời gian A* theo cạnh (có quay đầu, có cấm rẽ) từ `p` tới tâm nhu cầu gần đích nhất, và phạt gồm phạt đi bộ đuôi dài cùng phạt phía đường bất lợi. Điểm trả mặc định của baseline, đích đã snap, luôn là một ứng viên, nên hàm không bao giờ buộc phải đổi.

`examples/toi-noi-location/rust/crates/pickup/src/dropoff.rs`

```rust
impl Default for DropoffPolicy {
    fn default() -> Self {
        Self {
            a_walk: 1.0,
            d_trip: 1.0,
            e_exit: 1.0,
            max_walk_m: 150.0,
            refine_top: 6,
            anchors_k: 2,
            gate_radius_m: 80.0,
            use_exit: true,
            use_side: true,
        }
    }
}
```

Vòng chấm điểm cho từng ứng viên đã qua lọc ma trận:

`examples/toi-noi-location/rust/crates/pickup/src/dropoff.rs`

```rust
            let exit = if self.policy.use_exit {
                self.exit_s(ctx, c.pos, d.dest_true)
            } else {
                0.0
            };
            let mut pen = self.weights.tail_walk_penalty(walk_m);
            if self.policy.use_side {
                let (side, _) = side_of(engine, c.pos.edge, c.pos.frac, target);
                if side == Side::Left {
                    pen += self.weights.penalties.wrong_side_s
                        * side_scale(c.class, engine.g.lanes[c.pos.edge as usize]);
                }
            }
            let j = self.policy.a_walk * walk_m / speed
                + self.policy.d_trip * trip
                + self.policy.e_exit * exit
                + pen;
```

Phần đáng chú ý nhất là `exit`, chi phí thóat, vì nó là cách duy nhất lab biết cho «chuyến kế». Simulator **không biết** khách kế tiếp là ai lúc trả khách, nên lab dùng một đại diện:

`examples/toi-noi-location/rust/crates/pickup/src/dropoff.rs`

```rust
    /// Chi phí thoát (giây) từ `p`: A* theo cạnh tới `anchors_k` tâm nhu cầu gần đích nhất, lấy nhỏ nhất.
    fn exit_s(&mut self, ctx: &Ctx, p: Position, dest: LatLng) -> f64 {
        let k = self.policy.anchors_k;
        let mut near: Vec<(f64, Position)> = self
            .anchors(ctx)
            .iter()
            .map(|(ll, pos)| (haversine_m(*ll, dest), *pos))
            .collect();
        near.sort_by(|a, b| a.0.total_cmp(&b.0));
        near.truncate(k);
        let mut best = f64::INFINITY;
        for (_, pos) in near {
            if let Some(r) = ctx.route_car(Start::Pos(p), Goal::Pos(pos)) {
                best = best.min(f64::from(r.cost) / 1000.0);
            }
        }
        if best.is_finite() { best } else { 0.0 }
    }
```

Chuyến kế được đại diện bằng **hai tâm điểm nóng của kịch bản gần đích nhất** (8 tâm tất cả). Đây là một giả định thô; mục «Vì sao số đo nhỏ» quay lại chuyện này.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 1. Ba số hạng của hàm mục tiêu điểm trả. Xe chở khách tới điểm trả: thời gian xe T_trip. Khách đi bộ từ điểm trả tới đích thật: T_bộ. Tài xế rời điểm trả để đi tìm khách kế: T_thóat. Điểm trả baseline là điểm đích đã snap lên đường. Hình minh họa, không theo tỉ lệ."><text x="36" y="28" class="t-b">Ba số hạng của J_trả quanh một điểm trả</text><path d="M60 180 L830 180" class="f-none s-dead" stroke-width="16" stroke-linecap="round"/><circle cx="110" cy="180" r="8" class="f-pk s-pk" stroke-width="1.5"/><text x="110" y="158" class="t-b" text-anchor="middle">xe + khách</text><path d="M126 180 L431.0 180.0" class="f-none s-pk" stroke-width="2" stroke-linecap="round"/><path d="M440 180 L431.0 184.5 L431.0 175.5 z" class="f-pk"/><text x="285" y="214" class="t-small" text-anchor="middle">T_trip: xe chở khách tới điểm trả</text><circle cx="470" cy="70" r="8" class="f-warn s-warn" stroke-width="1.5"/><text x="486" y="74" class="t-small">đích thật (chưa snap, có thể có cổng)</text><path d="M470 80 L470 170" class="f-none s-dead dash" stroke-width="1.2" stroke-linecap="round"/><circle cx="470" cy="180" r="8" class="f-phys s-phys" stroke-width="1.5"/><text x="470" y="238" class="t-small" text-anchor="middle">đích đã snap lên đường: baseline trả khách ở đây</text><circle cx="640" cy="180" r="9" class="f-pk s-pk" stroke-width="1.5"/><text x="640" y="258" class="t-small" text-anchor="middle">điểm J_trả chọn (một ứng viên khác)</text><path d="M632 172 L478 78" class="f-none s-warn dash" stroke-width="2" stroke-linecap="round"/><text x="600" y="112" class="t-small">T_bộ: khách đi bộ</text><text x="600" y="128" class="t-small">từ điểm trả tới đích thật</text><path d="M656 180 L811.0 180.0" class="f-none s-phys" stroke-width="2" stroke-linecap="round"/><path d="M820 180 L811.0 184.5 L811.0 175.5 z" class="f-phys"/><text x="740" y="160" class="t-small" text-anchor="middle">T_thóat: tài xế đi tìm khách kế</text></svg>
</div>
<figcaption>Hình 1. Ba số hạng của hàm mục tiêu điểm trả quanh một điểm trả: xe chở khách tới (T_trip), khách đi bộ tới đích thật (T_bộ), tài xế rời đi tìm khách kế (T_thóat). Hình minh họa, không theo tỉ lệ; số đo thật ở Hình 2.</figcaption>
</figure>

## Cái cắm vào simulator: hai điểm cắm mới

Để chiến lược chọn được điểm trả, simulator cần hai thứ mà baseline không có. Một là điểm đến thật của khách (chưa snap), vì baseline chỉ giữ điểm đã snap lên đường. Hai là một lời gọi cho chiến lược ngay khi xe đón xong khách.

`examples/toi-noi-location/rust/crates/simulator/src/strategy.rs`

```rust
    fn observe_idle(&mut self, _idle: &[DriverView]) {}
    /// P04, LOC-38: chọn điểm trả khi tài xế đã đón khách. `None` (mặc định) = baseline: xe tới đúng điểm đích đã snap.
    /// Trả về các vị trí có hướng mà xe được phép vào; mô phỏng chọn đường rẻ nhất tới một trong chúng.
    fn choose_dropoff(&mut self, _ctx: &Ctx, _d: &DropView) -> Option<Vec<Position>> {
        None
    }
```

Mặc định `None`, nên baseline không đổi từng byte (mã băm FNV-1a của nhật ký chuyến seed 20261106 vẫn là `100a862df76819aa`, in ở lab bài 32). Bên cạnh đó có `BaselineLogged`: một baseline y hệt `AtPin` nhưng ghi nhật ký quãng khách đi bộ từ điểm trả tới đích thật, để có số so sánh mà không đổi bất kỳ số nào của mô phỏng. Quãng đi bộ **sau khi xuống xe** không nằm trong chỉ số §3.4 của simulator nên phải đo riêng như vậy.

Bốn biến thể được so với baseline, mỗi biến thể trên 20 seed ghép cặp:

- **(a)** chỉ điểm trả theo `J_trả` đầy đủ, điểm đón vẫn là ghim.
- **(b)** điểm trả không tính chi phí thóat.
- **(c)** chỉ tối thiểu quãng khách đi bộ từ điểm trả (bỏ trip, thóat, phía đường).
- **(d)** điểm đón `J` (`p04-v1`) cộng điểm trả `J_trả`.

## Kết quả: lợi ích nhỏ, không chứng minh được, và khách trả giá

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 612" role="img" aria-label="Hình 2. Chênh cặp so với baseline trên 20 seed, kèm khoảng tin cậy t 95%. Trên: km rỗng trung bình. Chỉ điểm đón: −107,0 mét [−151,6; −62,4]. Chỉ điểm trả J_trả: −16,3 [−51,5; +18,9]. Bỏ chi phí thóat: −1,0 [−13,0; +11,1]. Chỉ tối thiểu đi bộ: +5,8 [−13,5; +25,1]. Điểm đón cộng điểm trả: −129,4 [−171,1; −87,8]. Dưới: quãng khách đi bộ từ điểm trả tới đích thật: J_trả +10,6 [+9,0; +12,2]; bỏ thóat −2,3 [−3,9; −0,7]; chỉ đi bộ −3,8 [−5,3; −2,3]; điểm đón cộng điểm trả +11,9 [+9,4; +14,4]."><text x="36" y="30" class="t-b">Km rỗng trung bình (deadhead_m), chênh so với baseline, mét</text><path d="M748.333 44 L748.333 264" class="f-none s-muted dash" stroke-width="1.2" stroke-linecap="round"/><text x="36" y="56">chỉ điểm đón J (p04-v1), đo ở bài 32 và 40</text><text x="36" y="72" class="t-mono">−107,0 [−151,6; −62,4]</text><path d="M470.4 60 L633.933 60" class="f-none s-pk" stroke-width="3" stroke-linecap="round"/><path d="M470.4 55 L470.4 65" class="f-none s-pk" stroke-width="2" stroke-linecap="round"/><path d="M633.933 55 L633.933 65" class="f-none s-pk" stroke-width="2" stroke-linecap="round"/><circle cx="552.167" cy="60" r="5" class="f-pk s-pk" stroke-width="1.5"/><text x="36" y="100">(a) chỉ điểm trả, J_trả đầy đủ</text><text x="36" y="116" class="t-mono">−16,3 [−51,5; +18,9]</text><path d="M653.917 104 L782.983 104" class="f-none s-dead" stroke-width="3" stroke-linecap="round"/><path d="M653.917 99 L653.917 109" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M782.983 99 L782.983 109" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><circle cx="718.45" cy="104" r="5" class="f-dead s-dead" stroke-width="1.5"/><text x="36" y="144">(b) điểm trả bỏ chi phí thóat</text><text x="36" y="160" class="t-mono">−1,0 [−13,0; +11,1]</text><path d="M724.5 148 L768.683 148" class="f-none s-dead" stroke-width="3" stroke-linecap="round"/><path d="M724.5 143 L724.5 153" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M768.683 143 L768.683 153" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><circle cx="746.5" cy="148" r="5" class="f-dead s-dead" stroke-width="1.5"/><text x="36" y="188">(c) điểm trả chỉ tối thiểu đi bộ</text><text x="36" y="204" class="t-mono">+5,8 [−13,5; +25,1]</text><path d="M723.583 192 L794.35 192" class="f-none s-dead" stroke-width="3" stroke-linecap="round"/><path d="M723.583 187 L723.583 197" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><path d="M794.35 187 L794.35 197" class="f-none s-dead" stroke-width="2" stroke-linecap="round"/><circle cx="758.967" cy="192" r="5" class="f-dead s-dead" stroke-width="1.5"/><text x="36" y="232">(d) điểm đón J + điểm trả J_trả</text><text x="36" y="248" class="t-mono">−129,4 [−171,1; −87,8]</text><path d="M434.65 236 L587.367 236" class="f-none s-pk" stroke-width="3" stroke-linecap="round"/><path d="M434.65 231 L434.65 241" class="f-none s-pk" stroke-width="2" stroke-linecap="round"/><path d="M587.367 231 L587.367 241" class="f-none s-pk" stroke-width="2" stroke-linecap="round"/><circle cx="511.1" cy="236" r="5" class="f-pk s-pk" stroke-width="1.5"/><path d="M400 264 L840 264" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><path d="M473.333 264 L473.333 269" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><text x="473.333" y="284" class="t-small" text-anchor="middle">−150</text><path d="M565 264 L565 269" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><text x="565" y="284" class="t-small" text-anchor="middle">−100</text><path d="M656.667 264 L656.667 269" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><text x="656.667" y="284" class="t-small" text-anchor="middle">−50</text><path d="M748.333 264 L748.333 269" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><text x="748.333" y="284" class="t-small" text-anchor="middle">0</text><path d="M840 264 L840 269" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><text x="840" y="284" class="t-small" text-anchor="middle">+50</text><text x="36" y="306" class="t-small">xanh: khoảng không chứa 0 và có lợi; xám: khoảng chứa 0 (không khác biệt rõ)</text><text x="36" y="346" class="t-b">Khách đi bộ từ điểm trả tới đích thật hoặc cổng, chênh so với baseline, mét</text><path d="M546.667 360 L546.667 536" class="f-none s-muted dash" stroke-width="1.2" stroke-linecap="round"/><text x="36" y="372">(a) chỉ điểm trả, J_trả đầy đủ</text><text x="36" y="388" class="t-mono">+10,6 [+9,0; +12,2]</text><path d="M711.667 376 L770.333 376" class="f-none s-warn" stroke-width="3" stroke-linecap="round"/><path d="M711.667 371 L711.667 381" class="f-none s-warn" stroke-width="2" stroke-linecap="round"/><path d="M770.333 371 L770.333 381" class="f-none s-warn" stroke-width="2" stroke-linecap="round"/><circle cx="741" cy="376" r="5" class="f-warn s-warn" stroke-width="1.5"/><text x="36" y="416">(b) điểm trả bỏ chi phí thóat</text><text x="36" y="432" class="t-mono">−2,3 [−3,9; −0,7]</text><path d="M475.167 420 L533.833 420" class="f-none s-pk" stroke-width="3" stroke-linecap="round"/><path d="M475.167 415 L475.167 425" class="f-none s-pk" stroke-width="2" stroke-linecap="round"/><path d="M533.833 415 L533.833 425" class="f-none s-pk" stroke-width="2" stroke-linecap="round"/><circle cx="504.5" cy="420" r="5" class="f-pk s-pk" stroke-width="1.5"/><text x="36" y="460">(c) điểm trả chỉ tối thiểu đi bộ</text><text x="36" y="476" class="t-mono">−3,8 [−5,3; −2,3]</text><path d="M449.5 464 L504.5 464" class="f-none s-pk" stroke-width="3" stroke-linecap="round"/><path d="M449.5 459 L449.5 469" class="f-none s-pk" stroke-width="2" stroke-linecap="round"/><path d="M504.5 459 L504.5 469" class="f-none s-pk" stroke-width="2" stroke-linecap="round"/><circle cx="477" cy="464" r="5" class="f-pk s-pk" stroke-width="1.5"/><text x="36" y="504">(d) điểm đón J + điểm trả J_trả</text><text x="36" y="520" class="t-mono">+11,9 [+9,4; +14,4]</text><path d="M719 508 L810.667 508" class="f-none s-warn" stroke-width="3" stroke-linecap="round"/><path d="M719 503 L719 513" class="f-none s-warn" stroke-width="2" stroke-linecap="round"/><path d="M810.667 503 L810.667 513" class="f-none s-warn" stroke-width="2" stroke-linecap="round"/><circle cx="764.833" cy="508" r="5" class="f-warn s-warn" stroke-width="1.5"/><path d="M400 536 L840 536" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><path d="M455 536 L455 541" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><text x="455" y="556" class="t-small" text-anchor="middle">−5</text><path d="M546.667 536 L546.667 541" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><text x="546.667" y="556" class="t-small" text-anchor="middle">0</text><path d="M638.333 536 L638.333 541" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><text x="638.333" y="556" class="t-small" text-anchor="middle">+5</text><path d="M730 536 L730 541" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><text x="730" y="556" class="t-small" text-anchor="middle">+10</text><path d="M821.667 536 L821.667 541" class="f-none s-muted" stroke-width="1.2" stroke-linecap="round"/><text x="821.667" y="556" class="t-small" text-anchor="middle">+15</text><text x="36" y="578" class="t-small">cam: khách đi thêm (thiệt); xanh: khách đi ít hơn</text><text x="36" y="596" class="t-small">Trên dữ liệu mô phỏng, seed 20270001…20270020, kịch bản hcm-baseline. Số lấy từ results/p04.md, mục LOC-38.</text></svg>
</div>
<figcaption>Hình 2. Chênh cặp so với baseline (trung bình và khoảng tin cậy t 95%, 20 seed). Trên: km rỗng trung bình; dòng đầu là điểm đón riêng, đo ở bài 32 và 40, đặt vào để so. Dưới: quãng khách đi bộ từ điểm trả tới đích thật. Trên dữ liệu mô phỏng, seed 20270001…20270020.</figcaption>
</figure>

Bảng chi tiết cho biến thể (a), cặp theo seed, trên dữ liệu mô phỏng:

| Chỉ số | Baseline | (a) chỉ điểm trả | Chênh cặp [KTC 95% t] | Ý nghĩa |
|---|---:|---:|---:|---|
| `deadhead_m` trung bình | 2.357,0 | 2.340,7 | −16,3 [−51,5; +18,9] | không khác biệt rõ |
| `deadhead_m` p95 | 5.699,4 | 5.590,5 | −108,9 [−242,2; +24,5] | không khác biệt rõ |
| `pickup_eta_s` trung bình | 328,7 | 323,1 | −5,6 [−9,3; −1,9] | khoảng loại 0, nhỏ |
| `trip_m` trung bình | 6.856,9 | 6.839,9 | −17,0 [−31,2; −2,7] | khoảng loại 0, nhỏ |
| `cancel_rate` | 0,1272 | 0,1247 | −0,0025 [−0,0061; +0,0011] | không khác biệt rõ |
| Cước trung bình (đồng) | 33.857,4 | 33.781,2 | −76,2 [−140,5; −11,8] | nền tảng, tài xế thu ít hơn |
| Khách đi bộ ở điểm trả (m, TB theo seed) | 25,9 | 36,5 | **+10,6** [+9,0; +12,2] | **khách trả giá** |

Nguồn: `results/p04.md`, LOC-38 (a). Các khoảng loại 0 ở `pickup_eta_s` và `trip_m` có chênh nhỏ. Lab không có cơ chế tách riêng cho chúng, và bốn biến thể × 19 chỉ số là 76 khoảng 95%, nên kỳ vọng vài khoảng loại 0 chỉ vì tình cờ. Nên coi đây là giả thuyết.

Đọc từng biến thể:

- **(a)** km rỗng trung bình −16,3 m, khoảng chứa 0. Khoảng 28,7% chuyến đổi điểm trả (71,3% chọn đúng điểm baseline). Tính trên mọi chuyến, khách đi bộ ở cuối thêm trung bình 10,6 m; chênh theo từng khách ở p95 là +88,0 m, còn p50 là +0,0 m vì đa số chuyến không đổi.
- **(b)** bỏ chi phí thóat: 89,4% chuyến chọn đúng điểm baseline, km rỗng −1,0 m [−13,0; +11,1], khách đi bộ ở điểm trả −2,3 m [−3,9; −0,7]. Gần như trùng baseline. **Nếu lợi ích nhỏ của (a) có thật, nó có vẻ đến từ chi phí thóat**: bỏ nó đi thì hàm hầu như không còn lý do rời điểm đã snap. Đây là so sánh gián tiếp, (a) và (b) chưa so cặp với nhau.
- **(c)** chỉ tối thiểu quãng đi bộ: khách đi bộ ít hơn 3,8 m (25,9 xuống 22,1 m) và chỉ có một khoảng loại 0 ở `u_turns` với chênh +0,0 sau khi làm tròn, không có ý nghĩa thực tế. Đây là cận trên của «phần khách» trong mô hình này: dù tối ưu hóa chỉ vì khách, mỗi khách được lợi chừng bốn mét.
- **(d)** điểm đón cộng điểm trả: km rỗng −129,4 m [−171,1; −87,8], so với −107,0 m [−151,6; −62,4] của điểm đón riêng. Hai khoảng tin cậy chồng nhau và hai cấu hình chưa so trực tiếp với nhau, nên phần cộng thêm của điểm trả (khoảng 22 m) **chưa chứng minh được**. Khách đi bộ ở điểm trả thêm 11,9 m [+9,4; +14,4].

## Vì sao số đo nhỏ, và nó nói gì về ý tưởng

Có ít nhất bốn lý do, đều thuộc về mô hình chứ không nhất thiết thuộc về ý tưởng.

**Thước đi bộ để lại rất ít dư địa.** Baseline trả khách ở điểm đích đã snap, tức điểm trên đường gần đích thật nhất. Quãng đi bộ của lab đo dọc đồ thị đi bộ giữa hai điểm đã snap, **không gồm đoạn từ đường vào tới đích thật** (đích có thể cách đường tới 120 m). Vì vậy p50 bằng 0,0 m: ở hơn nửa số chuyến hai đầu snap vào cùng một chỗ. Trung bình là 25,8 m, và biến thể (c), tối ưu thuần cho khách, chỉ giảm 3,8 m. Con số đó nói về thước đo này, không nói khách thật hết chỗ để được lợi.

**Chuyến kế là đại diện thô.** Chi phí thóat trung bình của điểm đã chọn là 506 s: tâm nhu cầu gần nhất cách điểm trả cả hơn tám phút lái xe, nên tín hiệu «dễ bắt chuyến kế» yếu. Giả thuyết: phần lớn ứng viên quanh một đích đều cách tâm đó xa tương tự, nên chênh lệch giữa chúng nhỏ so với 506 s; lab không in phân bố chi phí thóat giữa các ứng viên. Simulator không biết khách kế tiếp ở đâu lúc trả khách, nên lab không có thứ tốt hơn.

**Mô hình không đo cái điểm trả muốn tối ưu.** Phạt phía đường chỉ nằm trong `J`, không có trong số đo nào: simulator không mô phỏng thời gian băng đường, và quãng đi bộ sau khi xuống xe không nằm trong chỉ số §3.4. Nếu giá trị thật của «trả đúng phía» là khách khỏi phải qua đường, mô hình này không có cách để thấy.

**Đuôi dài không phải chuyện điểm trả.** Dù ở baseline hay mọi chiến lược, quãng khách đi bộ từ điểm trả tới đích có max **4.167,6 m**: chính là loại ca đi vòng đã gặp ở bài 32 (hai điểm gần nhau nhưng đồ thị đi bộ nối ở rất xa). Chọn lại điểm trả trong tầm 150 m không sửa được liên thông của đồ thị đi bộ; max không đổi trong mọi biến thể. Cổng OSM trong 80 m quanh đích chỉ có ở 0,8% chuyến, khớp với bài 37.

Vậy giá trị của ý tưởng, nếu có, nằm ở đâu? Có hai chỗ đáng thử, và **chưa chỗ nào được chứng minh ở đây**.

- **Phía khách:** trả khách đúng phía, đúng cổng (nhất là khi có dữ liệu cổng do đối tác hay tài xế đóng góp, không chỉ OSM thưa). Mô hình không đo thời gian băng đường và chỉ có 0,8% chuyến có cổng, nên không thể hiện ra.
- **Phía cơ chế:** tài xế thóat ra thuận hướng để bắt chuyến kế, giảm quay đầu. Nó cần tín hiệu «chuyến kế» tốt hơn tâm điểm nóng, chẳng hạn bản đồ nhu cầu theo ô H3 ([bài 49](/articles/location-tai-xe-ranh-di-dau-dieu-huong-theo-nhu-cau)), hoặc chuyến kế đã được ghép sẵn khi chuyến này sắp xong ([bài 47](/articles/location-chain-dispatch-tai-xe-sap-tra-khach)).

Kết quả của bài là «chưa thấy lợi ích trong mô hình này», không phải «điểm trả không có lợi». Khoảng [−51,5; +18,9] vẫn chứa cả một mức giảm vài chục mét.

## Ai được lợi và ai trả giá

| Bên | Trong biến thể (a) | Bằng chứng |
|---|---|---|
| Khách đang đi | Đi bộ thêm ở cuối chuyến: +10,6 m trung bình, p95 theo từng khách +88,0 m | khoảng tin cậy loại 0 |
| Khách đang chờ xe | ETA ngắn hơn một chút: −5,6 s | khoảng loại 0, nhỏ |
| Tài xế | Km rỗng ít hơn 16,3 m | không khác biệt rõ |
| Nền tảng và tài xế (cước) | Cước trung bình thấp hơn 76,2 đồng mỗi chuyến vì quãng có khách ngắn đi 17,0 m | khoảng loại 0, nhỏ |

Bên chịu thiệt có bằng chứng; bên được lợi chủ yếu không. Khi bảng cân bằng lệch như vậy, đây là lý do bình thường để **không** bật tính năng chỉ vì nó «có lý».

**Đổi đích khi đang đi.** Đề cương của bài nêu cả việc khách đổi đích giữa chuyến. Lab **chưa** có: nó không có sự kiện đổi đích, cũng không có đồng ý của khách hay tính lại cước. Khi làm, nó đụng thẳng vào bài 40: đổi đích là đổi quãng có khách, nên đổi cước, và cùng quy tắc «không tăng giá lén» phải áp dụng.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` (hoặc thêm `-- --fixture` để chạy nhanh trên đồ thị tổng hợp). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run --release -q --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc38_diem_tra -- --jobs 6
```

Output thật của biến thể (a), kịch bản `hcm-baseline`, 20 seed 20270001…20270020 (rút gọn từ `results/p04.md`):

```text
  chỉ số                             baseline chiến lược   chênh TB           KTC 95% (t)   KTC 95% (bootstrap)     ↓/↑  trung vị seed b→n
  deadhead_m trung bình                2357.0     2340.7      -16.3        [-51.5, +18.9]        [-49.4, +14.8]    13/7          2335.7→2298.7
  deadhead_m p95                       5699.4     5590.5     -108.9       [-242.2, +24.5]        [-232.4, +6.1]    10/6          5522.2→5332.1
  pickup_eta_s trung bình               328.7      323.1       -5.6          [-9.3, -1.9]          [-9.0, -2.3]    16/4          326.4→317.4  *
  trip_m trung bình                    6856.9     6839.9      -17.0         [-31.2, -2.7]         [-29.6, -3.4]    16/4      6874.5→6842.2  *
  cancel_rate                          0.1272     0.1247    -0.0025    [-0.0061, +0.0011]    [-0.0059, +0.0007]     8/4      0.1261→0.1262
  cước trung bình (đồng)              33857.4    33781.2      -76.2       [-140.5, -11.8]       [-133.0, -14.4]    16/4    33936.6→33790.1  *
  điểm trả: quãng khách đi bộ từ điểm trả tới đích thật/cổng (3815 chuyến ghép cặp theo khách)
    baseline (trả đúng điểm đích đã snap): TB 25.8  p50 0.0  p95 90.6  max 4167.6 m
    chiến lược:                            TB 36.3  p50 0.0  p95 114.9  max 4167.6 m; chênh theo khách p5 +0.0  p50 +0.0  p95 +88.0  max +150.0 m
    chọn đúng điểm baseline: 71.3% chuyến; có cổng OSM trong 80 m quanh đích: 0.8% chuyến; chi phí thoát (A* theo cạnh tới tâm nhu cầu gần nhất) của điểm chọn, trung bình 506 s
    đi bộ ở điểm trả TB theo seed: baseline 25.9 → 36.5 m, chênh cặp +10.6 [+9.0, +12.2] (KTC t 95%, 20 seed) *
```

Dấu `*` là khoảng tin cậy t không chứa 0. Lệnh `bash examples/toi-noi-location/verify-p04.sh` chạy lab này trên fixture với 3 seed để kiểm đường ống, kèm test động cho điểm trả; nó không tái hiện bảng trên.

## Giới hạn của ví dụ

- **«Đích thật» là điểm sinh ngẫu nhiên** cách điểm đón 1.500–8.000 m chim bay rồi snap lên đường (cách đường tới 120 m). Không có địa chỉ, không có cổng nhà thật, chỉ có cổng OSM trong 80 m (0,8% chuyến).
- **Chuyến kế = tâm điểm nóng.** Mô phỏng không biết khách kế tiếp lúc trả khách; chi phí thóat trung bình 506 s là tín hiệu yếu.
- **Quãng đi bộ sau khi xuống xe** không nằm trong chỉ số §3.4; đo riêng bằng nhật ký của chiến lược.
- **Không đo thời gian băng đường** hay an toàn của khách; phạt phía đường chỉ nằm trong `J`.
- **Chưa so trực tiếp** các biến thể (a) và (d) (chỉ so từng biến thể với baseline); khoảng cách 22 m giữa −129,4 và −107,0 chưa kiểm định.
- **Không có đổi đích giữa chuyến.**
- **Số thời gian** không dùng: lab chạy trên máy chịu tải rất nặng (load average hơn 300 lúc đo).
- **Tính ngẫu nhiên:** hai lần chạy (baseline và chiến lược) rẽ nhánh khi ghép khác nhau, nên cặp theo seed giảm phương sai nhưng không loại hết nhiễu.

## Bài tập

**Bài 1.** Từ biến thể (a) và (b), giải thích bằng một câu vì sao lợi ích nhỏ của điểm trả có vẻ đến từ chi phí thóat, và nói con số nào trong output gợi ý điều đó.

<details><summary>Lời giải bài 1</summary>
<p>Khi bỏ chi phí thóat (biến thể b), 89,4% chuyến chọn đúng điểm baseline (so với 71,3% ở biến thể a) và km rỗng chỉ đổi −1,0 m [−13,0; +11,1]; tức là hầu hết lý do rời điểm đã snap biến mất cùng chi phí thóat. Số 71,3% so với 89,4% là chỉ báo tốt nhất lab có; muốn chắc phải so cặp (a) với (b).</p>
</details>

**Bài 2.** Biến thể (d) cho km rỗng −129,4 m còn điểm đón riêng cho −107,0 m. Một đồng nghiệp viết «điểm trả cộng thêm 22 m». Phản biện bằng số liệu và nêu cách đo đúng.

<details><summary>Lời giải bài 2</summary>
<p>Hai khoảng tin cậy [−171,1; −87,8] và [−151,6; −62,4] chồng nhau rộng, và hai cấu hình chưa được so trực tiếp với nhau, nên chênh 22 m có thể là nhiễu. Cách đo đúng: chạy cặp (d) trừ «điểm đón riêng» trên cùng 20 seed, báo chênh cặp và khoảng tin cậy của hiệu đó; chỉ khi khoảng loại 0 mới được viết có phần cộng thêm.</p>
</details>

**Bài 3.** Đề xuất một đại diện tốt hơn cho «chuyến kế» so với hai tâm điểm nóng gần nhất, dùng được trong simulator, và nói rủi ro của nó.

<details><summary>Lời giải bài 3</summary>
<p>Dùng mật độ nhu cầu lịch sử theo ô H3 quanh điểm trả (bản đồ nhu cầu như [bài 49](/articles/location-tai-xe-ranh-di-dau-dieu-huong-theo-nhu-cau)), tính thời gian từ ứng viên tới các ô nhu cầu cao trong bán kính vài trăm mét, có trọng số nhu cầu. Rủi ro: nếu bản đồ nhu cầu học từ chính nơi tài xế đang đón thì lặp lại thiên lệch «đón ở đâu thì học ở đó» của bài 39, và nếu simulator sinh nhu cầu từ chính các tâm điểm nóng đó thì kết quả là vòng tròn.</p>
</details>

Bài trước: [bài 37](/articles/location-hem-chung-cu-tttm-san-bay-cong-sanh). Bài tiếp: [bài 39](/articles/location-hoc-diem-don-tu-lich-su-dbscan-python).

## Tài liệu gốc

- [Key:entrance trên OSM wiki](https://wiki.openstreetmap.org/wiki/Key:entrance): thẻ `entrance=*`, nguồn cổng mà lab dùng khi có trong vòng 80 m quanh đích.
- [Student's t-test (Wikipedia)](https://en.wikipedia.org/wiki/Student%27s_t-test): mục kiểm định mẫu ghép cặp, nền của khoảng tin cậy trên hiệu từng seed.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho dữ liệu OSM của đồ thị HCM.
