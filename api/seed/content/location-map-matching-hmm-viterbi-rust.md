---
slug: location-map-matching-hmm-viterbi-rust
title: "Map matching bằng HMM và Viterbi: tài xế đang ở đường nào"
summary: "Bài 16: bám đường Newson–Krumm trên HCM, nhiễu mô phỏng: σ = 10 m, HMM 86,87% cạnh nới nút (69,09% chặt), snap 66,06% (49,85%); trên fixture σ = 3 m snap nhỉnh ở thước nới (99,57% so với 99,16%), thua ở thước chặt."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, map-matching, hmm, viterbi, newson-krumm, synthetic-data]
language: vi
---

Một đại lộ ở TP.HCM có dải phân cách ở giữa. Mỗi chiều xe chạy là một nhánh một chiều riêng, và sát bên là đường gom. Ba con đường cách nhau chưa tới hai chục mét, trong khi điểm GPS của tài xế lệch cỡ 10 m. Tài xế đang đi đúng chiều trên đại lộ; một điểm GPS nằm sát nhánh ngược chiều, hai điểm sau nằm sát đường gom. Nếu mỗi điểm chọn «đường gần nhất», marker của khách nhảy qua lại giữa ba đường, và điểm đón có thể bị đặt sang phía đường mà tài xế phải quay đầu cả trăm mét mới tới được.

Đây là bài 16 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng ứng viên từ R-tree của [bài 07](/articles/location-rstar-r-tree-snap-diem-ve-duong) và `NoiseModel` của [bài 15](/articles/location-kalman-loc-nhieu-gps-rust). Mọi độ chính xác trong bài là **trên dữ liệu mô phỏng**: xe chạy trên lộ trình thật của đồ thị HCM, nhưng nhiễu GPS do chính lab sinh ra, nên kết quả bị chi phối bởi mô hình nhiễu của chúng ta và không phải số đo trên GPS người thật.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Trên đồ thị HCM, nhiễu σ = 10 m mỗi trục (tương quan 8 giây), lấy mẫu 1 giây. Cách đơn giản nhất là snap mỗi điểm về cạnh gần nhất đi được bằng ô tô. Theo bạn bao nhiêu phần trăm điểm được gán đúng cạnh, và HMM với Viterbi cải thiện được mấy điểm phần trăm?</p>
<details><summary>Xem đáp án</summary><p>Snap đúng <b>66,06%</b> số điểm theo thước «cạnh nới nút» (49,85% theo thước chặt); HMM đúng <b>86,87%</b> (69,09% chặt), tức hơn khoảng 20,8 điểm phần trăm (19,2 điểm theo thước chặt), trên simulator của chính lab. Về lộ trình, RMF của snap là 1,310 (lộ trình sai nhiều hơn độ dài đúng), HMM là 0,098. Nhưng HMM không thắng mọi nơi: trên đồ thị fixture nhỏ ở σ = 3 m, snap có cạnh nới nút 99,57% cao hơn HMM (99,16%). Hai thước «chặt» và «nới nút» được giải thích ở dưới.</p></details>
</div>

## Từ snap từng điểm tới bài toán chuỗi

Snap xét mỗi điểm một mình, nên nó mù với thứ tự thời gian: nó không biết điểm trước đó nằm ở nhánh nào, và không biết xe không thể sang nhánh bên kia dải phân cách chỉ trong một giây. Bám đường (map matching) xử lý cả chuỗi: tìm dãy cạnh hợp lý nhất, trong đó mỗi điểm GPS vừa gần cạnh được gán, vừa nối được với điểm trước bằng một đoạn đường hợp lý.

Newson và Krumm (Microsoft Research, ACM SIGSPATIAL GIS 2009) mô hình hoá việc này bằng mô hình Markov ẩn. Trạng thái ẩn là «xe đang ở đoạn đường nào»; quan sát là điểm GPS. Hai xác suất làm nên mô hình, theo bản bài báo mà bài này đối chiếu:

- **Xác suất phát**: Gaussian theo khoảng cách chim bay từ điểm GPS tới điểm gần nhất trên đoạn đường ứng viên. Đoạn càng xa càng ít có khả năng sinh ra điểm đo.
- **Xác suất chuyển**: hàm mũ theo chênh lệch tuyệt đối giữa khoảng cách chim bay của hai điểm GPS liên tiếp và quãng đường đi trên mạng đường giữa hai điểm chiếu. Hai khoảng này gần bằng nhau với các cặp khớp đúng, và lệch nhiều với cặp sai.

Chuỗi trạng thái tốt nhất được tìm bằng thuật toán Viterbi. Bài báo ước lượng hai tham số (độ lệch chuẩn nhiễu σ_z và thang β của phân phối mũ) từ dữ liệu có nhãn, đo trên một chuyến lái xe dài 80 km bằng dữ liệu thật, và đo độ sai bằng phần lộ trình sai (đoạn thừa cộng đoạn thiếu, chia cho độ dài lộ trình đúng). Chỉ số RMF trong bài này lấy đúng tinh thần đó.

Lab **không** chép nguyên bài báo; nó khác ở các điểm sau. Trạng thái là **cạnh có hướng** (cạnh và cạnh ngược `twin` là hai trạng thái), nên hướng đi cũng được suy ra từ chuỗi điểm. σ của từng điểm lấy từ accuracy báo cáo (qua `NoiseModel`), không phải một hằng số. Thêm ràng buộc vận tốc tối đa và xử lý «gãy chuỗi» khi không còn đường nối. Bán kính ứng viên là 50 m và tối đa 8 cạnh vật lý gần nhất (tới 16 trạng thái có hướng), còn bài báo cắt ở 200 m. Chỉ profile ô tô, bỏ qua cấm rẽ và chi phí rẽ.

## Xác suất phát: điểm GPS cách mỗi cạnh bao xa

`states_for` hỏi R-tree lấy các cạnh trong bán kính quanh điểm GPS, rồi tạo một trạng thái cho mỗi chiều đi được. Điểm số phát là log của Gaussian bỏ hằng số: `−½ · (khoảng cách / σ)²`. Với điểm báo accuracy 10 m, σ = 6,7 m, nên cạnh cách 12 m có điểm −1,6 còn cạnh cách 5 m có −0,28. Nếu bật số hạng hướng đi (`heading_sigma_deg`, mặc định tắt) thì điểm của trạng thái còn bị trừ theo độ lệch giữa heading báo cáo và hướng cạnh.

Tệp `examples/toi-noi-location/rust/crates/map-match/src/hmm.rs` (dòng 157–194):

```rust
    fn states_for(&self, obs: &Obs) -> Vec<State> {
        let g = self.g;
        let bit = self.cfg.mode.bit();
        let sigma = self.cfg.noise.sigma_m(obs.accuracy_m);
        let cands = self.idx.candidates(
            g,
            obs.pos,
            self.cfg.radius_m,
            self.cfg.mode,
            self.cfg.max_candidates,
        );
        let use_heading = self.cfg.heading_sigma_deg.is_some()
            && obs.heading_deg.is_some()
            && obs.speed_mps.is_none_or(|s| s > 1.5);
        let mut out = Vec::with_capacity(cands.len() * 2);
        for c in cands {
            let len_e = f64::from(g.length_cm[c.edge as usize]) / 100.0;
            let a_rep = c.along_m.min(len_e).max(0.0);
            let mut push = |edge: EdgeId, offset: f64, bearing: f64| {
                let len = f64::from(g.length_cm[edge as usize]) / 100.0;
                let mut score = -0.5 * (c.dist_m / sigma).powi(2);
                if use_heading
                    && let (Some(h), Some(hs)) = (obs.heading_deg, self.cfg.heading_sigma_deg)
                {
                    score -= 0.5 * (bearing_diff_deg(f64::from(h), bearing) / hs).powi(2);
                }
                out.push(State {
                    m: MatchedPoint {
                        edge,
                        offset_m: offset.clamp(0.0, len) as f32,
                        pos: c.point,
                        dist_m: c.dist_m as f32,
                    },
                    len_m: len as f32,
                    score,
                    back: NO_BACK,
                });
            };
```

Điểm chiếu và vị trí dọc cạnh (`offset_m`) được lưu cùng trạng thái. Chúng cần cho bước chuyển: hai điểm cùng trên một cạnh thì quãng đường đi chỉ là hiệu hai vị trí dọc cạnh.

## Xác suất chuyển: quãng đường đi phải khớp đường chim bay

Với hai điểm GPS liên tiếp, `link` tính khoảng cách chim bay `straight`, thang `β = beta_m + beta_frac · straight` (mặc định `beta_m` = 1 m, `beta_frac` = 0), rồi với mỗi cặp trạng thái tìm quãng đường ngắn nhất trên mạng từ điểm chiếu này tới điểm chiếu kia. Dijkstra có giới hạn (`Router::distances_to`, bài 21 sẽ đào sâu thuật toán) chỉ tìm tới `limit = straight + 6β + 40 m` và tối đa 4.000 nút, để mỗi điểm GPS chỉ tốn vài chục nút duyệt. Cặp nào không tới được trong giới hạn hoặc cần đi nhanh hơn 38 m/s thì bị loại.

Tệp `examples/toi-noi-location/rust/crates/map-match/src/hmm.rs` (dòng 248–252):

```rust
        let cfg = &self.cfg;
        let straight = haversine_m(p.obs.pos, obs.pos);
        let beta = cfg.beta_m + cfg.beta_frac * straight;
        let reach = cfg.max_speed_mps * dt + cfg.slack_m;
        let limit_m = (straight + 6.0 * beta + cfg.slack_m).min(reach);
```

Tệp `examples/toi-noi-location/rust/crates/map-match/src/hmm.rs` (dòng 283–305), vòng trong của `link`:

```rust
            for (j, s) in states.iter().enumerate() {
                let a2 = f64::from(s.m.offset_m);
                let route = if s.m.edge == e1 && a2 + cfg.backward_tol_m >= a1 {
                    (a2 - a1).max(0.0)
                } else {
                    if row[j] == INF {
                        continue;
                    }
                    (f64::from(ps.len_m) - a1) + f64::from(row[j]) / 100.0 + a2
                };
                if route > reach {
                    continue;
                }
                let back = if s.m.edge == e1 {
                    (a1 - a2).max(0.0)
                } else {
                    0.0
                };
                let sc = ps.score - (route - straight).abs() / beta - back / cfg.backward_beta_m;
                if sc > best[j].0 {
                    best[j] = (sc, i as u32);
                }
            }
```

Dòng quan trọng là `sc = ps.score - (route - straight).abs() / beta - back / cfg.backward_beta_m`: log của phân phối mũ, cộng thêm một hình phạt cho việc đi lùi trên cùng cạnh (mỗi mét lùi bị phạt `1/backward_beta_m` nat). Hình 1 vẽ hậu quả trên ví dụ đại lộ. Điểm GPS thứ hai nằm gần nhánh ngược chiều hơn nhánh đúng, nên snap gần nhất chọn nhánh ngược. Nhưng muốn đi từ nhánh đúng sang nhánh ngược chiều của một dải phân cách, xe phải quay đầu ở chỗ hở xa nhất, quãng đường vượt giới hạn, nên chuyển trạng thái đó bị loại và không có chuỗi nào đi qua nhánh ngược ở điểm 2.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 430" role="img" aria-label="Hình 1. Lưới Viterbi cho ba điểm GPS và ba ứng viên (đúng chiều, ngược chiều, đường gom), số minh hoạ tự chọn: điểm 2 gần chiều ngược nhất nhưng đường đi qua chiều ngược bị loại, chuỗi tốt nhất đi hết chiều đúng."><defs><marker id="loc16-f1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs><text x="230.0" y="52.0" class="t-b">điểm GPS 1</text><text x="450.0" y="52.0" class="t-b">điểm GPS 2</text><text x="670.0" y="52.0" class="t-b">điểm GPS 3</text><text x="20.0" y="34.0" class="t-small">ε = −½·(d/σ)², σ = 6 m;  chuyển = −|đường đi − chim bay|/β, β = 1 m</text><text x="20.0" y="124.0" class="t-b">A: đại lộ, đúng chiều</text><rect x="230.0" y="92.0" width="150.0" height="56.0" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="305.0" y="114.0" text-anchor="middle">d = 4 m</text><text x="305.0" y="134.0" class="t-mono" text-anchor="middle">ε = −0,22</text><rect x="450.0" y="92.0" width="150.0" height="56.0" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="525.0" y="114.0" text-anchor="middle">d = 14 m</text><text x="525.0" y="134.0" class="t-mono" text-anchor="middle">ε = −2,72</text><rect x="670.0" y="92.0" width="150.0" height="56.0" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="745.0" y="114.0" text-anchor="middle">d = 3 m</text><text x="745.0" y="134.0" class="t-mono" text-anchor="middle">ε = −0,12</text><text x="20.0" y="224.0" class="t-b">B: nhánh xe chạy ngược</text><rect x="230.0" y="192.0" width="150.0" height="56.0" rx="8" class="f-surface s-muted" stroke-width="1.5"/><text x="305.0" y="214.0" text-anchor="middle">d = 15 m</text><text x="305.0" y="234.0" class="t-mono" text-anchor="middle">ε = −3,12</text><rect x="450.0" y="192.0" width="150.0" height="56.0" rx="8" class="f-surface s-muted" stroke-width="1.5"/><text x="525.0" y="214.0" text-anchor="middle">d = 6 m</text><text x="525.0" y="234.0" class="t-mono" text-anchor="middle">ε = −0,50</text><rect x="670.0" y="192.0" width="150.0" height="56.0" rx="8" class="f-surface s-muted" stroke-width="1.5"/><text x="745.0" y="214.0" text-anchor="middle">d = 17 m</text><text x="745.0" y="234.0" class="t-mono" text-anchor="middle">ε = −4,01</text><text x="20.0" y="324.0" class="t-b">C: đường gom bên cạnh</text><rect x="230.0" y="292.0" width="150.0" height="56.0" rx="8" class="f-surface s-muted" stroke-width="1.5"/><text x="305.0" y="314.0" text-anchor="middle">d = 22 m</text><text x="305.0" y="334.0" class="t-mono" text-anchor="middle">ε = −6,72</text><rect x="450.0" y="292.0" width="150.0" height="56.0" rx="8" class="f-surface s-muted" stroke-width="1.5"/><text x="525.0" y="314.0" text-anchor="middle">d = 12 m</text><text x="525.0" y="334.0" class="t-mono" text-anchor="middle">ε = −2,00</text><rect x="670.0" y="292.0" width="150.0" height="56.0" rx="8" class="f-surface s-muted" stroke-width="1.5"/><text x="745.0" y="314.0" text-anchor="middle">d = 20 m</text><text x="745.0" y="334.0" class="t-mono" text-anchor="middle">ε = −5,56</text><path d="M380.0 120.0 L450.0 120.0" class="f-none s-pk" stroke-width="3" marker-end="url(#loc16-f1-a)"/><text x="415.0" y="112.0" class="t-small" text-anchor="middle">−1,0</text><path d="M600.0 120.0 L670.0 120.0" class="f-none s-pk" stroke-width="3" marker-end="url(#loc16-f1-a)"/><text x="635.0" y="112.0" class="t-small" text-anchor="middle">−1,0</text><path d="M380.0 220.0 L450.0 220.0" class="f-none s-muted" stroke-width="1.5" marker-end="url(#loc16-f1-a)"/><text x="415.0" y="212.0" class="t-small" text-anchor="middle">−1,0</text><path d="M600.0 220.0 L670.0 220.0" class="f-none s-muted" stroke-width="1.5" marker-end="url(#loc16-f1-a)"/><text x="635.0" y="212.0" class="t-small" text-anchor="middle">−1,0</text><path d="M380.0 320.0 L450.0 320.0" class="f-none s-muted" stroke-width="1.5" marker-end="url(#loc16-f1-a)"/><text x="415.0" y="312.0" class="t-small" text-anchor="middle">−1,0</text><path d="M600.0 320.0 L670.0 320.0" class="f-none s-muted" stroke-width="1.5" marker-end="url(#loc16-f1-a)"/><text x="635.0" y="312.0" class="t-small" text-anchor="middle">−1,0</text><path d="M380.0 130.0 L450.0 206.0" class="f-none s-warn dash" stroke-width="1.5"/><text x="20.0" y="172.0" class="t-small">A→B (nét đứt): muốn sang nhánh</text><text x="20.0" y="186.0" class="t-small">kia phải quay đầu ở chỗ hở dải</text><text x="20.0" y="200.0" class="t-small">phân cách, quá giới hạn: bị loại</text><rect x="445.0" y="187.0" width="160.0" height="66.0" rx="10" class="f-none s-warn dash" stroke-width="2"/><text x="525.0" y="272.0" class="t-small" text-anchor="middle">snap gần nhất chọn B ở đây</text><text x="20.0" y="394.0">Tổng log xác suất của chuỗi: A–A–A = −5,07; B–B–B = −9,64; C–C–C = −16,28. Chuỗi A–B–A không tồn tại vì A→B bị loại.</text><text x="20.0" y="414.0">Viterbi giữ chuỗi cao nhất (A–A–A), nên điểm 2 được gán A dù điểm đo gần B hơn.</text></svg>
</div>
<figcaption>Hình 1. Lưới Viterbi tối giản: mỗi cột là một điểm GPS, mỗi ô là một cạnh có hướng ứng viên. <b>Số minh hoạ tự chọn</b> (khoảng cách d, σ, β), không phải output của lab; công thức giống mã trong <code>hmm.rs</code>. Nhiễu đẩy điểm 2 gần chiều ngược hơn, nhưng ngữ cảnh (không có đường nối hợp lý từ A sang B) giữ chuỗi ở chiều đúng.</figcaption>
</figure>

Hình 1 dùng số tự chọn (d, σ = 6 m, β = 1 m) để thấy cơ chế, không phải output của lab. Điều cần nhớ là thông tin nằm ở **cạnh nối** giữa các ô, không ở từng ô: chính ngữ cảnh loại nhánh ngược chiều, kể cả khi điểm đo gần nó hơn.

## Viterbi: chọn cả chuỗi rồi truy ngược

Viterbi là quy hoạch động trên lưới các lớp (một lớp cho mỗi điểm GPS). Mỗi trạng thái giữ điểm số tốt nhất của mọi chuỗi kết thúc ở nó, và con trỏ `back` về trạng thái trước trong chuỗi đó. `extend` dựng một lớp và trừ điểm tốt nhất của lớp khỏi mọi điểm số (chuẩn hoá) để điểm không trôi về −∞ trên vết dài. Bản offline dựng cả lưới rồi truy ngược từ trạng thái tốt nhất của lớp cuối:

Tệp `examples/toi-noi-location/rust/crates/map-match/src/hmm.rs` (dòng 348–364):

```rust
        for (si, &end) in seg_end.iter().enumerate() {
            let mut pos = end;
            let Some(mut s) = layers[nonempty[pos]].best() else {
                continue;
            };
            loop {
                let l = &layers[nonempty[pos]];
                points[l.seq as usize] = Some(l.states[s].m);
                seg_of[l.seq as usize] = si as u32;
                let b = l.states[s].back;
                if b == NO_BACK || pos == 0 {
                    break;
                }
                pos -= 1;
                s = b as usize;
            }
        }
```

«Gãy chuỗi» xảy ra khi không có đường nối hợp lệ giữa hai lớp liên tiếp, hoặc khoảng cách thời gian vượt `gap_break_s` (90 giây). Khi đó đoạn cũ được chốt, đoạn mới bắt đầu, và lab đếm số lần gãy; gãy là một tín hiệu chất lượng đáng quan sát ở production. Lộ trình suy ra (`route`) nối các điểm đã bám bằng đường ngắn nhất giữa hai điểm liên tiếp (`path_between`).

## Đo độ chính xác: hai thước, và vì sao thước chặt thấp

Lab chấm bằng ground truth của simulator và dùng ba thước đo từng điểm cộng hai thước đo lộ trình. **Cạnh chặt**: cạnh gán trùng cạnh thật hoặc `twin` của nó (bỏ qua chiều). **Cạnh nới nút**: như trên, hoặc cạnh kề chung nút mà điểm chiếu cách vị trí thật không quá 12 m. **Chiều nới**: như «nới nút» nhưng phải đúng chiều xe chạy. **Recall** là tỉ lệ độ dài cạnh thật mà lộ trình suy ra có; **RMF** = (độ dài thiếu + độ dài thừa) / độ dài thật, 0 là hoàn hảo và trên 1 là lộ trình sai nhiều hơn đúng.

Tệp `examples/toi-noi-location/rust/crates/map-match/src/eval.rs` (dòng 114–128):

```rust
                let near = haversine_m(m.pos, t.pos) <= JUNCTION_TOL_M;
                let (mt, mh, tt, th) = (
                    g.tail[m.edge as usize],
                    g.head[m.edge as usize],
                    g.tail[t.edge as usize],
                    g.head[t.edge as usize],
                );
                let shares = mt == tt || mt == th || mh == tt || mh == th;
                let consecutive = mt == th || mh == tt;
                if canon(g, m.edge) == canon(g, t.edge) || (near && shares) {
                    s.edge_near += 1;
                }
                if m.edge == t.edge || (near && consecutive) {
                    s.dir_near += 1;
                }
```

Vì sao cần thước «nới nút»? Cạnh của đồ thị HCM rất ngắn: nhiều cạnh chỉ dài 5–40 m, vì OSM tách đường tại mỗi nút giao và mỗi chỗ đổi thuộc tính. Khi xe đi qua nút, điểm thật nằm cách nút vài mét, và việc gán nó sang cạnh kề là hợp lý chứ không phải lỗi bám đường. Thước chặt coi đó là sai, nên bị trần thấp ngay cả khi thuật toán rất tốt: HMM ở σ = 3 m chỉ đạt 83,65% chặt nhưng 99,15% nới. **Thước nới được thêm sau khi xem kết quả**, không đặt trước; lab nói rõ điều này và bài cũng vậy. Vì thế bảng dưới đây hiện cả hai thước, không chỉ thước dễ nhìn. Bộ chọn tham số (seed 1106, 40 chuyến khác hẳn) và bộ báo cáo (seed 20261106) tách nhau; bảng chọn tham số cho thấy mặt phẳng rất phẳng, mọi tổ hợp β và hệ số accuracy trong lưới 60 ô cho cạnh nới nút từ 87,44% đến 88,93%, nên giá trị mặc định (`beta_m` 1, `beta_frac` 0, hệ số accuracy 0,67) không phải một đỉnh nhọn.

Kết quả trên đồ thị HCM, 26.146 điểm mỗi mức σ, lấy mẫu 1 Hz, **nhiễu mô phỏng do lab sinh** (tương quan 8 giây, không phải GPS thật), `MatchConfig::default()`:

| σ (m) | Cách | Cạnh chặt | Cạnh nới nút | Chiều nới | Recall | RMF | Sai số vị trí (m) |
|---:|---|---:|---:|---:|---:|---:|---:|
| 3 | snap | 75,11% | 96,96% | 87,00% | 98,89% | 0,635 | 3,04 |
| 3 | HMM | 83,65% | 99,15% | 98,08% | 99,97% | 0,028 | 2,58 |
| 6 | snap | 62,51% | 85,58% | 74,03% | 96,37% | 0,991 | 6,32 |
| 6 | HMM | 76,99% | 96,44% | 93,61% | 99,61% | 0,053 | 5,00 |
| 10 | snap | 49,85% | 66,06% | 55,16% | 93,47% | 1,310 | 10,99 |
| 10 | HMM | 69,09% | 86,87% | 82,96% | 98,68% | 0,098 | 8,33 |
| 15 | snap | 38,71% | 48,23% | 39,09% | 89,47% | 1,642 | 17,34 |
| 15 | HMM | 57,33% | 71,28% | 66,73% | 95,67% | 0,236 | 13,17 |
| 25 | snap | 25,24% | 29,75% | 23,09% | 83,78% | 2,229 | 30,10 |
| 25 | HMM | 35,80% | 44,55% | 40,15% | 89,65% | 0,742 | 25,86 |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 440" role="img" aria-label="Hình 2. Độ chính xác gán cạnh của snap và HMM theo σ trên đồ thị HCM, hai thước chặt và nới nút, cùng cạnh nới nút trên fixture ở σ = 3 m, nơi snap nhỉnh hơn HMM."><rect x="20.0" y="20.0" width="660.0" height="400.0" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36.0" y="44.0" class="t-b">Đồ thị HCM, 26.146 điểm mỗi mức σ (%)</text><path d="M58.0 340.0 L670.0 340.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="52.0" y="344.0" class="t-small" text-anchor="end">0</text><path d="M58.0 280.0 L670.0 280.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="52.0" y="284.0" class="t-small" text-anchor="end">25</text><path d="M58.0 220.0 L670.0 220.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="52.0" y="224.0" class="t-small" text-anchor="end">50</text><path d="M58.0 160.0 L670.0 160.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="52.0" y="164.0" class="t-small" text-anchor="end">75</text><path d="M58.0 100.0 L670.0 100.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="52.0" y="104.0" class="t-small" text-anchor="end">100</text><rect x="66.0" y="159.7" width="24.0" height="180.3" rx="2" class="f-warn-soft s-warn" stroke-width="1"/><text x="78.0" y="154.7" class="t-small" text-anchor="middle">75,1</text><rect x="92.0" y="139.2" width="24.0" height="200.8" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="104.0" y="134.2" class="t-small" text-anchor="middle">83,7</text><rect x="118.0" y="107.3" width="24.0" height="232.7" rx="2" class="f-warn s-warn" stroke-width="1"/><text x="130.0" y="102.3" class="t-small" text-anchor="middle">97,0</text><rect x="144.0" y="102.0" width="24.0" height="238.0" rx="2" class="f-phys s-phys" stroke-width="1"/><text x="156.0" y="97.0" class="t-small" text-anchor="middle">99,2</text><text x="118.0" y="358.0" text-anchor="middle">σ = 3 m</text><rect x="186.0" y="190.0" width="24.0" height="150.0" rx="2" class="f-warn-soft s-warn" stroke-width="1"/><text x="198.0" y="185.0" class="t-small" text-anchor="middle">62,5</text><rect x="212.0" y="155.2" width="24.0" height="184.8" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="224.0" y="150.2" class="t-small" text-anchor="middle">77,0</text><rect x="238.0" y="134.6" width="24.0" height="205.4" rx="2" class="f-warn s-warn" stroke-width="1"/><text x="250.0" y="129.6" class="t-small" text-anchor="middle">85,6</text><rect x="264.0" y="108.5" width="24.0" height="231.5" rx="2" class="f-phys s-phys" stroke-width="1"/><text x="276.0" y="103.5" class="t-small" text-anchor="middle">96,4</text><text x="238.0" y="358.0" text-anchor="middle">σ = 6 m</text><rect x="306.0" y="220.4" width="24.0" height="119.6" rx="2" class="f-warn-soft s-warn" stroke-width="1"/><text x="318.0" y="215.4" class="t-small" text-anchor="middle">49,9</text><rect x="332.0" y="174.2" width="24.0" height="165.8" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="344.0" y="169.2" class="t-small" text-anchor="middle">69,1</text><rect x="358.0" y="181.5" width="24.0" height="158.5" rx="2" class="f-warn s-warn" stroke-width="1"/><text x="370.0" y="176.5" class="t-small" text-anchor="middle">66,1</text><rect x="384.0" y="131.5" width="24.0" height="208.5" rx="2" class="f-phys s-phys" stroke-width="1"/><text x="396.0" y="126.5" class="t-small" text-anchor="middle">86,9</text><text x="358.0" y="358.0" text-anchor="middle">σ = 10 m</text><rect x="426.0" y="247.1" width="24.0" height="92.9" rx="2" class="f-warn-soft s-warn" stroke-width="1"/><text x="438.0" y="242.1" class="t-small" text-anchor="middle">38,7</text><rect x="452.0" y="202.4" width="24.0" height="137.6" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="464.0" y="197.4" class="t-small" text-anchor="middle">57,3</text><rect x="478.0" y="224.2" width="24.0" height="115.8" rx="2" class="f-warn s-warn" stroke-width="1"/><text x="490.0" y="219.2" class="t-small" text-anchor="middle">48,2</text><rect x="504.0" y="168.9" width="24.0" height="171.1" rx="2" class="f-phys s-phys" stroke-width="1"/><text x="516.0" y="163.9" class="t-small" text-anchor="middle">71,3</text><text x="478.0" y="358.0" text-anchor="middle">σ = 15 m</text><rect x="546.0" y="279.4" width="24.0" height="60.6" rx="2" class="f-warn-soft s-warn" stroke-width="1"/><text x="558.0" y="274.4" class="t-small" text-anchor="middle">25,2</text><rect x="572.0" y="254.1" width="24.0" height="85.9" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="584.0" y="249.1" class="t-small" text-anchor="middle">35,8</text><rect x="598.0" y="268.6" width="24.0" height="71.4" rx="2" class="f-warn s-warn" stroke-width="1"/><text x="610.0" y="263.6" class="t-small" text-anchor="middle">29,8</text><rect x="624.0" y="233.1" width="24.0" height="106.9" rx="2" class="f-phys s-phys" stroke-width="1"/><text x="636.0" y="228.1" class="t-small" text-anchor="middle">44,6</text><text x="598.0" y="358.0" text-anchor="middle">σ = 25 m</text><rect x="700.0" y="20.0" width="160.0" height="400.0" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="710.0" y="44.0" class="t-b">Fixture, σ = 3 m</text><text x="710.0" y="60.0" class="t-small">3.710 điểm</text><rect x="712.0" y="138.8" width="30.0" height="201.2" rx="2" class="f-warn-soft s-warn" stroke-width="1"/><text x="727.0" y="133.8" class="t-small" text-anchor="middle">83,85</text><rect x="748.0" y="124.7" width="30.0" height="215.3" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="763.0" y="119.7" class="t-small" text-anchor="middle">89,70</text><rect x="784.0" y="101.0" width="30.0" height="239.0" rx="2" class="f-warn s-warn" stroke-width="1"/><text x="799.0" y="96.0" class="t-small" text-anchor="middle">99,57</text><rect x="820.0" y="102.0" width="30.0" height="238.0" rx="2" class="f-phys s-phys" stroke-width="1"/><text x="835.0" y="97.0" class="t-small" text-anchor="middle">99,16</text><text x="780.0" y="358.0" text-anchor="middle">σ = 3 m</text><rect x="60.0" y="384.0" width="14.0" height="10.0" rx="2" class="f-warn-soft s-warn" stroke-width="1"/><text x="80.0" y="393.0" class="t-small">snap, chặt</text><rect x="160.0" y="384.0" width="14.0" height="10.0" rx="2" class="f-phys-soft s-phys" stroke-width="1"/><text x="180.0" y="393.0" class="t-small">HMM, chặt</text><rect x="260.0" y="384.0" width="14.0" height="10.0" rx="2" class="f-warn s-warn" stroke-width="1"/><text x="280.0" y="393.0" class="t-small">snap, nới nút</text><rect x="370.0" y="384.0" width="14.0" height="10.0" rx="2" class="f-phys s-phys" stroke-width="1"/><text x="390.0" y="393.0" class="t-small">HMM, nới nút</text><text x="60.0" y="412.0" class="t-small">Nhiễu mô phỏng do lab sinh (tương quan 8 s), không phải GPS thật. Thứ tự cột trong mỗi nhóm như chú giải.</text></svg>
</div>
<figcaption>Hình 2. Tỉ lệ điểm được gán đúng cạnh, từ mục 2 của output <code>loc16_hmm</code> (bộ báo cáo, seed 20261106). Hai thước cùng hiện: «chặt» và «nới nút». Cột phải: trên fixture ở σ = 3 m, với tham số mặc định chọn trên HCM, snap có cạnh nới nút <b>99,57%</b>, cao hơn HMM (99,16%); HMM hơn ở thước chặt (89,70% so với 83,85%). Nhiễu do lab sinh.</figcaption>
</figure>

Trên đồ thị HCM, HMM thắng snap ở mọi mức nhiễu, và khoảng cách rộng ra khi nhiễu tăng: RMF 0,098 so với 1,310 ở σ = 10 m. Ở σ = 25 m cả hai đều tệ (cạnh nới nút 44,55% so với 29,75%, cạnh chặt 35,80% so với 25,24%; RMF 0,742 so với 2,229); HMM không cứu được nhiễu quá lớn. Hình 3 là một lát cắt cụ thể: bảy điểm liên tiếp trên đường Thành Thái ở σ = 10 m, nơi theo thước chặt snap sai cả bảy điểm và HMM đúng cả bảy. Lab chọn đoạn này đúng theo tiêu chí đó, nên nó minh hoạ cơ chế chứ không đại diện cho cả tập.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 3. Bảy điểm liên tiếp trên đường Thành Thái, σ = 10 m: snap gán sai cạnh ở cả bảy điểm, HMM đúng cả bảy."><rect x="20.0" y="20.0" width="840.0" height="290.0" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36.0" y="46.0" class="t-b">Đường Thành Thái, mã cạnh đồ thị (chuyến báo cáo, σ = 10 m)</text><text x="36.0" y="114.0">cạnh thật</text><text x="36.0" y="174.0">snap gần nhất</text><text x="36.0" y="234.0">HMM + Viterbi</text><text x="236.0" y="82.0" class="t-small" text-anchor="middle">t = 391 s</text><rect x="190.0" y="90.0" width="92.0" height="40.0" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="236.0" y="114.0" class="t-mono" text-anchor="middle">86296</text><rect x="190.0" y="150.0" width="92.0" height="40.0" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="236.0" y="174.0" class="t-mono" text-anchor="middle">86294</text><rect x="190.0" y="210.0" width="92.0" height="40.0" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="236.0" y="234.0" class="t-mono" text-anchor="middle">86296</text><text x="236.0" y="276.0" class="t-small" text-anchor="middle">3,4 m</text><text x="332.0" y="82.0" class="t-small" text-anchor="middle">t = 392 s</text><rect x="286.0" y="90.0" width="92.0" height="40.0" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="332.0" y="114.0" class="t-mono" text-anchor="middle">86293</text><rect x="286.0" y="150.0" width="92.0" height="40.0" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="332.0" y="174.0" class="t-mono" text-anchor="middle">86291</text><rect x="286.0" y="210.0" width="92.0" height="40.0" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="332.0" y="234.0" class="t-mono" text-anchor="middle">86293</text><text x="332.0" y="276.0" class="t-small" text-anchor="middle">1,8 m</text><text x="428.0" y="82.0" class="t-small" text-anchor="middle">t = 393 s</text><rect x="382.0" y="90.0" width="92.0" height="40.0" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="428.0" y="114.0" class="t-mono" text-anchor="middle">86290</text><rect x="382.0" y="150.0" width="92.0" height="40.0" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="428.0" y="174.0" class="t-mono" text-anchor="middle">86292</text><rect x="382.0" y="210.0" width="92.0" height="40.0" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="428.0" y="234.0" class="t-mono" text-anchor="middle">86290</text><text x="428.0" y="276.0" class="t-small" text-anchor="middle">3,7 m</text><text x="524.0" y="82.0" class="t-small" text-anchor="middle">t = 394 s</text><rect x="478.0" y="90.0" width="92.0" height="40.0" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="524.0" y="114.0" class="t-mono" text-anchor="middle">86290</text><rect x="478.0" y="150.0" width="92.0" height="40.0" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="524.0" y="174.0" class="t-mono" text-anchor="middle">86287</text><rect x="478.0" y="210.0" width="92.0" height="40.0" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="524.0" y="234.0" class="t-mono" text-anchor="middle">86290</text><text x="524.0" y="276.0" class="t-small" text-anchor="middle">0,4 m</text><text x="620.0" y="82.0" class="t-small" text-anchor="middle">t = 395 s</text><rect x="574.0" y="90.0" width="92.0" height="40.0" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="620.0" y="114.0" class="t-mono" text-anchor="middle">86290</text><rect x="574.0" y="150.0" width="92.0" height="40.0" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="620.0" y="174.0" class="t-mono" text-anchor="middle">86287</text><rect x="574.0" y="210.0" width="92.0" height="40.0" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="620.0" y="234.0" class="t-mono" text-anchor="middle">86290</text><text x="620.0" y="276.0" class="t-small" text-anchor="middle">1,0 m</text><text x="716.0" y="82.0" class="t-small" text-anchor="middle">t = 396 s</text><rect x="670.0" y="90.0" width="92.0" height="40.0" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="716.0" y="114.0" class="t-mono" text-anchor="middle">86289</text><rect x="670.0" y="150.0" width="92.0" height="40.0" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="716.0" y="174.0" class="t-mono" text-anchor="middle">260123</text><rect x="670.0" y="210.0" width="92.0" height="40.0" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="716.0" y="234.0" class="t-mono" text-anchor="middle">86289</text><text x="716.0" y="276.0" class="t-small" text-anchor="middle">6,3 m</text><text x="812.0" y="82.0" class="t-small" text-anchor="middle">t = 397 s</text><rect x="766.0" y="90.0" width="92.0" height="40.0" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="812.0" y="114.0" class="t-mono" text-anchor="middle">86289</text><rect x="766.0" y="150.0" width="92.0" height="40.0" rx="6" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="812.0" y="174.0" class="t-mono" text-anchor="middle">260123</text><rect x="766.0" y="210.0" width="92.0" height="40.0" rx="6" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="812.0" y="234.0" class="t-mono" text-anchor="middle">86289</text><text x="812.0" y="276.0" class="t-small" text-anchor="middle">13,2 m</text><text x="36.0" y="280.0" class="t-small">thô → HMM</text><text x="36.0" y="296.0" class="t-small">(dịch chuyển điểm)</text></svg>
</div>
<figcaption>Hình 3. Bảy điểm GPS liên tiếp trên đường Thành Thái (σ = 10 m), mục 6 của output <code>loc16_hmm</code>. Ô màu cảnh báo là cạnh sai theo thước chặt: snap sai ở cả bảy điểm, HMM đúng cả bảy; lab chọn đoạn này theo đúng tiêu chí đó nên không đại diện cho cả tập. Hàng dưới là khoảng cách giữa điểm thô và điểm đã bám (tính lại từ toạ độ trong output). Mã cạnh tra được trong <code>data/hcm-graph.bin</code>. Nhiễu do lab sinh.</figcaption>
</figure>

## Chỗ snap không thua, và chỗ HMM yếu

Đây là phần không nên giấu. Lab cũng chạy trên fixture, đồ thị tổng hợp nhỏ (234 nút, 840 cạnh, 20 chuyến dài 0,4–1,5 km, 3.710 điểm mỗi mức σ), vẫn với nhiễu mô phỏng và ground truth của chính simulator. Bảng này dùng `MatchConfig::default()` **đã chọn trên HCM**, không chọn lại cho fixture:

| σ (m) | Cách | Cạnh chặt | Cạnh nới nút | Chiều nới | RMF |
|---:|---|---:|---:|---:|---:|
| 3 | snap | 83,85% | **99,57%** | 91,40% | 0,721 |
| 3 | HMM | 89,70% | 99,16% | 99,03% | 0,109 |
| 10 | snap | 71,59% | 83,91% | 71,91% | 1,057 |
| 10 | HMM | 84,69% | 94,12% | 93,72% | 0,136 |

**Ở σ = 3 m trên fixture, snap có cạnh nới nút cao hơn HMM (99,57% so với 99,16%).** HMM vẫn hơn ở thước chặt, ở chiều và ở RMF, nên đúng cách đọc là: khi nhiễu nhỏ hơn khoảng cách giữa các đường thì «đang ở đường nào» đã gần như được snap giải xong, và chỗ HMM còn hơn là **chiều đi** và **lộ trình**. Lab chưa kiểm nguyên nhân của chênh lệch 0,41 điểm. Có hai nghi vấn hợp lý. Một: bảng này dùng `MatchConfig::default()` đã chọn trên HCM, còn bảng chọn tham số của chính fixture chọn giá trị khác hẳn (`beta_m` = 6, `beta_frac` = 0,3, hệ số accuracy = 1,5), nên HMM chạy với tham số không dành cho đồ thị này; **tham số không mang sang đồ thị khác**. Hai: HMM gãy chuỗi 11 lần trên 3.710 điểm, và mỗi lần gãy bắt đầu lại từ điểm không có ngữ cảnh. Chưa chạy lại fixture bằng tham số của chính nó, nên chưa nói được nghi vấn nào đúng.

Những điểm yếu khác có số đo:

- **Lấy mẫu thưa là chỗ snap hỏng nhất**, không phải HMM. Ở σ = 6 m, chu kỳ 10 giây: snap có recall lộ trình 70,12% (bỏ sót cạnh vì không nối khoảng trống) và RMF 0,595; HMM 99,12% và 0,023. HMM chỉ tụt từ 96,44% (1 giây) xuống 95,56% (10 giây) theo cạnh nới nút, và từ 76,99% xuống 75,80% theo cạnh chặt. Bài báo gốc cũng báo HMM chịu lấy mẫu thưa tốt (0,11% lộ trình sai ở chu kỳ 30 giây trên dữ liệu lái xe thật của họ), nhưng hai con số không so sánh được: dữ liệu, bản đồ, nhiễu và cách tính khác nhau.
- **Điểm nhảy xa làm gãy chuỗi.** Khi tỉ lệ điểm nhảy 25–70 m tăng từ 0% lên 0,5% rồi 3%, số lần gãy chuỗi đi từ 15 lên 38 rồi 183, và HMM đi từ 87,42% xuống 86,87% rồi 82,60% cạnh nới nút (70,55%, 69,09%, 63,87% cạnh chặt); snap gần như không đổi ở hai đầu (66,16% và 65,92% nới, 50,14% và 49,81% chặt) vì vốn đã sai nhiều. Lab chưa có cơ chế «bỏ điểm nhảy rồi nối tiếp».
- **Tham số của mô hình quan trọng không đều.** Đổi từng thành phần ở σ = 10 m, mặc định 86,87% nới nút và 69,09% chặt (nới / chặt): chỉ 3 ứng viên mỗi điểm còn 78,49% / 61,52%; bán kính 25 m, nhỏ hơn nhiễu, còn 84,46% / 66,86%; `beta_m` = 30 (quá lỏng) còn 84,03% / 65,54%. Thêm số hạng hướng đi **tăng** lên 87,91% / 71,71%; chạy trên điểm đã lọc Kalman của bài 15 tăng lên 87,26% / 70,21%; bỏ accuracy gần như không đổi (87,07% / 69,26%, nhưng accuracy trong lab hiệu chuẩn tốt, như bài 15 đã nói).

Một lỗi đã gặp và sửa, đáng ghi lại vì nó không lộ ra trong số tổng. Bản đầu cho phép đi lùi một đoạn ngắn trên cùng cạnh với quãng đường tính bằng 0 và miễn phí, nên đi ngược chiều cạnh rẻ gần bằng đổi sang cạnh `twin`. Lỗi được phát hiện nhờ soi một chỗ gãy chuỗi ở σ = 3 m, không nhờ số tổng. Sau khi sửa (phạt mỗi mét lùi), hàng «không phạt đi lùi» cho 87,06% so với 86,87% mặc định (cạnh chặt 69,40% so với 69,09%): ảnh hưởng nhỏ trên tập này, nhưng đây là loại lỗi chỉ kiểm thử trên ca cụ thể mới thấy.

## Chi phí: vài micro-giây mỗi điểm

Mỗi điểm GPS sinh trung bình 11,5 ứng viên và 6,1 lần Dijkstra có giới hạn, duyệt khoảng 24,8 nút. Một luồng, σ = 10 m: HMM offline mất 9,1 µs mỗi điểm, so với 1,20 µs cho snap về cạnh gần nhất; hạ xuống 3 ứng viên còn 7,2 µs (nhưng mất 8,4 điểm cạnh nới nút và 7,6 điểm cạnh chặt). Hai số này **đo khi máy đang chịu tải nặng** (tải 1 phút khoảng 59 trên 10 lõi, ghi trong output); đừng trích µs tuyệt đối, chỉ rút ra tỉ lệ: trong lần đo này HMM tốn cỡ 7,6 lần snap. Bài 17 đo lại chi phí này cho trường hợp 25.000 tài xế chạy online.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin`. Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc16_hmm
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc16_hmm -- --fixture --trips 20
```

Cấu hình đo: `--release`, Apple M1 Max, đồ thị HCM 126.699 nút và 319.074 cạnh có hướng (© OpenStreetMap contributors), 40 chuyến báo cáo mỗi mức nhiễu (cùng tập chuyến, nhiễu khác nhau), seed báo cáo 20261106, seed chọn tham số 1106. Output (rút gọn, mục chọn tham số và mục 2 với σ = 3 và 10 m):

```text
chọn (tối đa cạnh nới% − 10·RMF): beta_m = 1, beta_frac = 0, accuracy_to_sigma = 0.67; mặc định trong MatchConfig::default(): 1, 0, 0.67
…
-- 2. độ chính xác theo độ lệch chuẩn nhiễu (bộ báo cáo, lấy mẫu 1 Hz, tương quan 8 s, MatchConfig::default()) --
                                        cạnh chặt  cạnh nới  chiều nới  recall    RMF  sai số vị trí (m)
σ = 3 m: 26146 điểm, RMSE thô 5.53 m
  snap về cạnh gần nhất                   75.11%    96.96%    87.00%    98.89%   0.635      3.04
  HMM + Viterbi offline                   83.65%    99.15%    98.08%    99.97%   0.028      2.58  (gãy chuỗi 27)
…
σ = 10 m: 26146 điểm, RMSE thô 14.62 m
  snap về cạnh gần nhất                   49.85%    66.06%    55.16%    93.47%   1.310     10.99
  HMM + Viterbi offline                   69.09%    86.87%    82.96%    98.68%   0.098      8.33  (gãy chuỗi 38)
```

Số đếm và tỉ lệ tái lập từng chữ số với cùng seed; chỉ phần µs/điểm là nhiễu theo tải máy. Test hồi quy trên ca khó của fixture (cầu vượt, hai chiều đại lộ có dải phân cách): `cargo test --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p map-match --test hard_cases -- --nocapture`, in `snap 58.6%/65.4%  offline 85.5%/93.9%` (chặt/nới nút) và RMF 0,97 so với 0,14. Ngưỡng của test (offline hơn snap ít nhất 5 điểm, RMF offline thấp hơn) được đặt rộng hơn số đo và **đặt sau khi xem một lần chạy**, nên đây là kiểm tra hồi quy, không phải chứng minh. Toàn phần: `bash examples/toi-noi-location/verify-p02-rust.sh`.

## Giới hạn của ví dụ

- **Không có GPS người thật.** Nhiễu có hằng số tương quan 8 giây, accuracy hiệu chuẩn tốt, điểm nhảy xa do lab sinh; mọi tỉ lệ đúng cạnh chỉ đúng cho mô hình này.
- **Chỉ profile ô tô.** Chưa bám đường cho xe máy (hẻm hẹp là chỗ xe máy khác ô tô), và đồ thị cho ô tô vào cả những hẻm không có thẻ cấm, nên một chuyến 2–4,5 km có nhiều đoạn hẻm.
- **Bỏ qua cấm rẽ và chi phí rẽ** (dùng Dijkstra không có chi phí rẽ; bài 23 mới thêm). Việc quay đầu ở dải phân cách vì thế chỉ được mô hình hoá qua độ dài đường đi.
- **Chưa có «bỏ điểm nhảy rồi nối tiếp»**: mỗi điểm đa đường làm gãy chuỗi.
- **Tham số không mang sang đồ thị khác**; chọn lại cho từng bản đồ.
- **Chưa chạy lại fixture bằng tham số của chính nó**, nên chênh lệch 99,57% so với 99,16% chưa có nguyên nhân được kiểm.
- Mô hình chỉ dùng heading nếu bật `heading_sigma_deg`; vì chưa kiểm với heading thật nên mặc định tắt.

## Bài tập

**Bài 1.** Cho `σ` = 6 m, β = 1 m. Hai trạng thái ở điểm 2: A cách điểm đo 14 m, B cách 6 m. Giả sử có chuyển hợp lệ từ A và từ B ở điểm 1, đều có |đường đi − chim bay| = 1 m. Chênh lệch điểm số phát giữa A và B là bao nhiêu? Phải chênh lệch quãng đường đi tối thiểu bao nhiêu để B bị A vượt qua?

<details><summary>Lời giải bài 1</summary>
<p>Điểm phát A = −½ · (14/6)² = −2,72; B = −½ · (6/6)² = −0,5; B hơn 2,22 nat. Mỗi mét chênh lệch quãng đường đi trừ 1/β = 1 nat, nên nếu chuyển vào B có |đường đi − chim bay| lớn hơn chuyển vào A khoảng 2,22 m trở lên thì A thắng. Đó là lý do thang β quyết định mô hình tin hình học mạng đường hay tin điểm đo hơn.</p>
</details>

**Bài 2.** Vì sao thước «nới nút» không thể dùng để so sánh hai thuật toán nếu một bên được phép gán sang cạnh kề tuỳ ý? Lab chặn điều đó bằng cách nào?

<details><summary>Lời giải bài 2</summary>
<p>Nếu cạnh kề nào cũng tính đúng thì một thuật toán chỉ cần gán sang cạnh bất kỳ chung nút là «đúng», nên thước mất khả năng phân biệt. Lab chặn bằng hai điều kiện đồng thời: cạnh phải chung nút với cạnh thật <i>và</i> điểm chiếu phải cách vị trí thật không quá 12 m (<code>JUNCTION_TOL_M</code>); hai thuật toán chịu cùng quy tắc. Vì thước được thêm sau khi xem kết quả nên bài luôn hiện cả thước chặt để người đọc tự kiểm.</p>
</details>

**Bài 3.** Cạnh nới nút của snap trên fixture ở σ = 3 m là 99,57%, cao hơn HMM (99,16%), dù thua ở cạnh chặt (83,85% so với 89,70%). Đề xuất hai lần chạy phân biệt hai nghi vấn trong bài.

<details><summary>Lời giải bài 3</summary>
<p>Nghi vấn «tham số không hợp»: chạy lại fixture với <code>MatchConfig</code> mang giá trị mà bảng chọn tham số của fixture chọn (<code>beta_m</code> = 6, <code>beta_frac</code> = 0,3, hệ số accuracy 1,5) và so cạnh nới nút. Nghi vấn «gãy chuỗi»: giữ tham số mặc định nhưng tăng <code>gap_break_s</code> và nới giới hạn tìm đường (<code>slack_m</code>, <code>max_settled</code>) để số lần gãy giảm, rồi xem chênh lệch có thu hẹp không. Nếu cả hai không đổi gì, nguyên nhân nằm ở chỗ khác và phải soi từng điểm snap thắng HMM. Ghi mọi lần chạy vào <code>results/</code>.</p>
</details>

Bài trước: [bài 15](/articles/location-kalman-loc-nhieu-gps-rust). Bài tiếp: [bài 17](/articles/location-map-matching-online-cua-so-cau-vuot-duong-song-song).

## Tài liệu gốc

- Paul Newson, John Krumm, [«Hidden Markov Map Matching Through Noise and Sparseness»](https://www.microsoft.com/en-us/research/publication/hidden-markov-map-matching-noise-sparseness/), ACM SIGSPATIAL GIS 2009 (Seattle, 4–6/11/2009): bài báo gốc của xác suất phát, xác suất chuyển và chỉ số lộ trình sai.
- A. J. Viterbi, «Error bounds for convolutional codes and an asymptotically optimum decoding algorithm», IEEE Transactions on Information Theory, tập 13, số 2, 4/1967, trang 260–269 (DOI 10.1109/TIT.1967.1054010): nguồn của thuật toán Viterbi.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho dữ liệu đồ thị HCM.
