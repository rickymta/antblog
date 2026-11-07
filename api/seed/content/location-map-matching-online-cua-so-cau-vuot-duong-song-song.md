---
slug: location-map-matching-online-cua-so-cau-vuot-duong-song-song
title: "Map matching online: cửa sổ trượt, cầu vượt và đường song song"
summary: "Bài 17: bám đường theo luồng với độ trễ quyết định, ca khó cầu vượt, đường gom, hầm, mất tín hiệu và ngân sách CPU cho 25.000 tài xế; offline không phải lúc nào cũng chính xác hơn online (đường gom)."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, map-matching, online, sliding-window, overpass, synthetic-data]
language: vi
---

Tài xế chạy lên một cầu vượt; bên dưới, một con đường mặt đất chạy cắt ngang, cách mặt cầu vài mét theo chiều cao nhưng chỉ vài mét theo bản đồ phẳng. Điểm GPS lệch 12 m rơi vào chỗ gần đường dưới hơn cầu. Trong ba giây marker của khách nhảy xuống đường dưới, rồi bật lên lại cầu. Giải pháp rõ ràng là đợi thêm vài điểm cho chắc, nhưng mỗi giây đợi là một giây marker đứng sau xe. Bài này đo chính xác chuyện đó: chờ bao lâu thì đáng, và ở những chỗ khó nhất của TP.HCM (cầu vượt, đường song song, đường gom, hầm) thì chờ có cứu được không.

Đây là bài 17 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài tiếp nối HMM offline của [bài 16](/articles/location-map-matching-hmm-viterbi-rust): cùng mô hình, cùng tham số, chỉ khác là điểm đến theo luồng. Mọi độ chính xác là **trên dữ liệu mô phỏng**: nhiễu GPS do chính lab sinh, và các «ca khó» được tìm bằng cách quét hình học và thẻ của đồ thị OSM chứ chưa duyệt từng ca bằng mắt.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bám đường với độ trễ 0: mỗi điểm mới đến thì chốt ngay và không bao giờ sửa. Trên chuyến thường (σ = 8 m), nó gán cạnh y hệt snap về cạnh gần nhất. Theo bạn lộ trình nó dựng ra tốt hơn hay tệ hơn snap?</p>
<details><summary>Xem đáp án</summary><p><b>Tệ hơn</b>: RMF 2,683 so với 1,152 của snap (cùng 75,73% cạnh nới nút và 55,76% cạnh chặt). Cạnh gán giống nhau, chỉ khác cách dựng lộ trình: snap không lấp khoảng trống giữa hai cạnh, còn online nối hai điểm liên tiếp bằng đường ngắn nhất, nên mỗi lần cạnh nhảy sang đường bên cạnh là một đoạn đi vòng bị tính vào lộ trình. Chỉ cần chờ <b>1 điểm</b>, RMF rơi xuống 0,572.</p></details>
</div>

## Online khác offline ở chỗ nào

Offline nhìn cả chuyến rồi truy ngược một lần (bài báo của Newson và Krumm cũng trình bày thuật toán theo kiểu xử lý cả lô). Online nhận từng điểm, và phải trả lời «xe đang ở cạnh nào» trong khi tương lai chưa tới. `OnlineMatcher` giữ **cùng lưới Viterbi** như bài 16, nhưng chia các lớp thành hai nhóm: các lớp đã chốt (không đổi nữa) và **cửa sổ** các lớp chưa chốt, nơi mỗi điểm chỉ có một ước lượng tạm có thể bị sửa. `Lag::Points(n)` quy định một điểm được chốt khi đã có thêm `n` điểm sau nó (hoặc `Lag::Millis` theo thời gian). Trễ 0 trong lab thực chất là snap: lớp vừa dựng được chốt ngay nên cửa sổ rỗng, điểm kế tiếp không có lớp trước để nối và chỉ còn điểm số phát. Bản tham lam thật, vẫn nối tiếp từ trạng thái vừa chốt, lab chưa đo. Trễ vô hạn thì chính là offline, và property test `online_with_unbounded_lag_equals_offline` kiểm đúng điều đó trên các vết ngẫu nhiên của fixture: kết quả từng điểm trùng bit với `match_offline`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 1. Cửa sổ trượt của bám đường online với trễ 3 điểm: các điểm cũ đã chốt, điểm đầu cửa sổ được chốt khi điểm mới đến, ba điểm sau còn ước lượng tạm và có thể bị sửa."><defs><marker id="loc17-f1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs><rect x="40.0" y="56.0" width="100.0" height="56.0" rx="8" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="90.0" y="80.0" class="t-b" text-anchor="middle">k−6</text><text x="90.0" y="98.0" class="t-small" text-anchor="middle">đã chốt</text><rect x="160.0" y="56.0" width="100.0" height="56.0" rx="8" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="210.0" y="80.0" class="t-b" text-anchor="middle">k−5</text><text x="210.0" y="98.0" class="t-small" text-anchor="middle">đã chốt</text><rect x="280.0" y="56.0" width="100.0" height="56.0" rx="8" class="f-phys-soft s-phys" stroke-width="1.5"/><text x="330.0" y="80.0" class="t-b" text-anchor="middle">k−4</text><text x="330.0" y="98.0" class="t-small" text-anchor="middle">đã chốt</text><rect x="400.0" y="56.0" width="100.0" height="56.0" rx="8" class="f-pk-soft s-pk" stroke-width="2.5"/><text x="450.0" y="80.0" class="t-b" text-anchor="middle">k−3</text><text x="450.0" y="98.0" class="t-small" text-anchor="middle">chốt bây giờ</text><rect x="520.0" y="56.0" width="100.0" height="56.0" rx="8" class="f-surface s-muted dash" stroke-width="1.5"/><text x="570.0" y="80.0" class="t-b" text-anchor="middle">k−2</text><text x="570.0" y="98.0" class="t-small" text-anchor="middle">ước lượng tạm</text><rect x="640.0" y="56.0" width="100.0" height="56.0" rx="8" class="f-surface s-muted dash" stroke-width="1.5"/><text x="690.0" y="80.0" class="t-b" text-anchor="middle">k−1</text><text x="690.0" y="98.0" class="t-small" text-anchor="middle">ước lượng tạm</text><rect x="760.0" y="56.0" width="100.0" height="56.0" rx="8" class="f-surface s-muted dash" stroke-width="1.5"/><text x="810.0" y="80.0" class="t-b" text-anchor="middle">k</text><text x="810.0" y="98.0" class="t-small" text-anchor="middle">ước lượng tạm</text><rect x="512.0" y="28.0" width="356.0" height="112.0" rx="10" class="f-none s-muted dash" stroke-width="1.5"/><text x="520.0" y="44.0" class="t-small">cửa sổ: 3 lớp chưa chốt</text><text x="802.0" y="134.0" class="t-small" text-anchor="end">lớp mới nhất</text><path d="M810.0 112.0 L810.0 142.0" class="f-none s-pk" stroke-width="2"/><path d="M810.0 142.0 L456.0 142.0" class="f-none s-pk" stroke-width="2.5" marker-end="url(#loc17-f1-a)"/><text x="630.0" y="162.0" text-anchor="middle">truy ngược từ trạng thái tốt nhất của lớp mới nhất</text><text x="40.0" y="214.0">1. extend: dựng một lớp ứng viên cho điểm mới và nối nó với lớp trước (Viterbi một bước).</text><text x="40.0" y="236.0">2. backtrack: truy ngược ra ước lượng tạm của mọi điểm trong cửa sổ; đổi cạnh so với lần trước thì tăng bộ đếm sửa.</text><text x="40.0" y="258.0">3. finalize_front: chốt lớp đầu cửa sổ, rồi đặt −∞ cho trạng thái lớp sau không xuất phát từ lựa chọn đó (cam kết).</text><text x="40.0" y="296.0" class="t-small">Trễ 3 điểm là 3 giây ở 1 Hz, và 12 giây ở nhịp 4 giây của app. Trễ 0: không có lớp trước để nối, chỉ còn điểm số phát như snap.</text></svg>
</div>
<figcaption>Hình 1. Cơ chế của <code>OnlineMatcher</code> với <code>Lag::Points(3)</code>. Sau cam kết, chuỗi đã chốt luôn liền mạch (không «dịch chuyển tức thời» giữa hai điểm), nhưng điểm đã chốt không sửa lại được: đó là cái giá của việc trả kết quả sớm.</figcaption>
</figure>

Mỗi điểm mới đi qua ba việc (Hình 1): dựng một lớp và nối với lớp trước (như bài 16), truy ngược để cập nhật ước lượng tạm, rồi chốt lớp đầu cửa sổ nếu cửa sổ đã dài hơn `lag`. Vòng chốt trong `push` là:

Tệp `examples/toi-noi-location/rust/crates/map-match/src/online.rs` (dòng 156–170):

```rust
        // chốt các điểm đã đủ cũ
        let mut bt = bt;
        loop {
            let finalize = match self.lag {
                Lag::Points(n) => self.window.len() > n,
                Lag::Millis(ms) => self
                    .window
                    .front()
                    .is_some_and(|f| obs.t_ms - f.layer.obs.t_ms >= ms),
            };
            if !finalize || self.window.is_empty() {
                break;
            }
            out.push(self.finalize_front(&mut bt, obs.t_ms));
        }
```

Mỗi điểm đã chốt được trả dưới dạng `Decision`: cạnh đã chốt, cạnh được gán lần đầu (`first_guess`), số lần ước lượng tạm đổi cạnh trước khi chốt (`revisions`) và độ trễ thực tế `lag_ms`. Hai trường giữa cho phép đo xem app sẽ phải **sửa lại** marker bao nhiêu lần nếu hiển thị ước lượng tạm.

## Chốt điểm và cam kết

Chốt một điểm không chỉ là ghi kết quả. Nếu lớp đầu được chốt ở trạng thái `c`, thì các trạng thái ở lớp sau mà không xuất phát từ `c` phải bị loại, nếu không chuỗi đã chốt có thể không liền với chuỗi tương lai (marker «dịch chuyển tức thời» giữa hai điểm). `finalize_front` đặt điểm số −∞ cho chúng rồi lan sự loại bỏ về phía sau:

Tệp `examples/toi-noi-location/rust/crates/map-match/src/online.rs` (dòng 189–227):

```rust
    fn finalize_front(&mut self, bt: &mut Vec<Option<u32>>, now_ms: i64) -> Decision {
        let chosen = bt.remove(0);
        let slot = self.window.pop_front().expect("cửa sổ không rỗng");
        let point = chosen.map(|s| slot.layer.states[s as usize].m);
        if let Some(c) = chosen {
            // loại các trạng thái ở lớp không rỗng kế tiếp không xuất phát từ `c`, rồi lan tỏa về phía sau
            let mut alive_prev: Vec<bool> = Vec::new();
            let mut first = true;
            for sl in self.window.iter_mut() {
                if sl.layer.is_empty() {
                    continue;
                }
                let states = &mut sl.layer.states;
                let mut now: Vec<bool> = Vec::with_capacity(states.len());
                for st in states.iter_mut() {
                    let ok = if first {
                        st.back == c
                    } else {
                        st.back != NO_BACK
                            && alive_prev.get(st.back as usize).copied().unwrap_or(false)
                    };
                    if !ok {
                        st.score = f64::NEG_INFINITY;
                    }
                    now.push(ok && st.alive());
                }
                alive_prev = now;
                first = false;
            }
        }
        Decision {
            seq: slot.layer.seq,
            t_ms: slot.layer.obs.t_ms,
            point,
            first_guess: slot.first,
            revisions: slot.revisions,
            lag_ms: now_ms - slot.layer.obs.t_ms,
        }
    }
```

Cam kết là cái giá của việc trả kết quả sớm: một khi đã chốt, sai cũng không sửa được. Đó là lý do trễ càng ngắn thì rủi ro chốt sai càng cao, và là lý do ước lượng tạm (hàm `provisional`) chỉ nên dùng để **hiển thị**, không để ra quyết định kinh doanh như đặt điểm đón hay tính cước.

## Trễ đổi lấy độ chính xác

Lab quét độ trễ trên 40 chuyến thường (26.146 điểm, σ = 8 m, 1 Hz, nhiễu mô phỏng, tham số HMM mặc định đã chọn ở bài 16). «%sửa» là tỉ lệ điểm có ước lượng tạm đổi cạnh ít nhất một lần trước khi chốt; «%đầu-sai» là tỉ lệ điểm mà cạnh gán lần đầu khác cạnh chốt.

| Cách | Cạnh chặt | Cạnh nới nút | RMF | %sửa | %đầu-sai |
|---|---:|---:|---:|---:|---:|
| snap về cạnh gần nhất | 55,76% | 75,73% | 1,152 | | |
| online, trễ 0 | 55,76% | 75,73% | 2,683 | 0,0% | 0,0% |
| online, trễ 1 điểm | 61,17% | 77,23% | 0,572 | 7,2% | 5,7% |
| online, trễ 2 điểm | 64,69% | 81,71% | 0,504 | 11,2% | 9,3% |
| online, trễ 3 điểm | 66,40% | 84,19% | 0,454 | 13,7% | 11,5% |
| online, trễ 5 điểm | 69,18% | 86,85% | 0,290 | 16,9% | 14,0% |
| online, trễ 8 điểm | 70,09% | 88,68% | 0,219 | 20,1% | 16,3% |
| online, trễ 12 điểm | 70,79% | 89,87% | 0,195 | 23,4% | 19,0% |
| online, trễ 20 điểm | 71,51% | 90,87% | 0,177 | 26,7% | 21,4% |
| offline (nhìn cả chuyến) | 73,47% | 92,53% | 0,073 | | |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 380" role="img" aria-label="Hình 2. Độ trễ quyết định so với độ chính xác: cạnh nới nút tăng từ 75,73% ở trễ 0 lên 90,87% ở trễ 20 điểm (offline 92,53%); RMF của lộ trình giảm từ 2,683 xuống 0,177, và trễ 0 tệ hơn snap (1,152)."><rect x="20.0" y="20.0" width="420.0" height="340.0" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36.0" y="44.0" class="t-b">Cạnh nới nút (%), chuyến thường σ = 8 m</text><path d="M74.0 300.0 L424.0 300.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="68.0" y="304.0" class="t-small" text-anchor="end">70</text><path d="M74.0 256.0 L424.0 256.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="68.0" y="260.0" class="t-small" text-anchor="end">75</text><path d="M74.0 212.0 L424.0 212.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="68.0" y="216.0" class="t-small" text-anchor="end">80</text><path d="M74.0 168.0 L424.0 168.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="68.0" y="172.0" class="t-small" text-anchor="end">85</text><path d="M74.0 124.0 L424.0 124.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="68.0" y="128.0" class="t-small" text-anchor="end">90</text><path d="M74.0 80.0 L424.0 80.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="68.0" y="84.0" class="t-small" text-anchor="end">95</text><path d="M74.0 249.6 L424.0 249.6" class="f-none s-warn dash" stroke-width="1.5"/><path d="M74.0 101.7 L424.0 101.7" class="f-none s-phys dash" stroke-width="1.5"/><text x="84.0" y="263.6" class="t-small">snap 75,73</text><text x="84.0" y="95.7" class="t-small">offline 92,53</text><path d="M80.0 249.6 L128.6 236.4 L177.1 197.0 L225.7 175.1 L274.3 151.7 L322.9 135.6 L371.4 125.1 L420.0 116.3" class="f-none s-pk" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round"/><circle cx="80.0" cy="249.6" r="4" class="f-pk s-pk" stroke-width="1"/><text x="80.0" y="320.0" class="t-small" text-anchor="middle">0</text><circle cx="128.6" cy="236.4" r="4" class="f-pk s-pk" stroke-width="1"/><text x="128.6" y="320.0" class="t-small" text-anchor="middle">1</text><circle cx="177.1" cy="197.0" r="4" class="f-pk s-pk" stroke-width="1"/><text x="177.1" y="320.0" class="t-small" text-anchor="middle">2</text><circle cx="225.7" cy="175.1" r="4" class="f-pk s-pk" stroke-width="1"/><text x="225.7" y="320.0" class="t-small" text-anchor="middle">3</text><circle cx="274.3" cy="151.7" r="4" class="f-pk s-pk" stroke-width="1"/><text x="274.3" y="320.0" class="t-small" text-anchor="middle">5</text><circle cx="322.9" cy="135.6" r="4" class="f-pk s-pk" stroke-width="1"/><text x="322.9" y="320.0" class="t-small" text-anchor="middle">8</text><circle cx="371.4" cy="125.1" r="4" class="f-pk s-pk" stroke-width="1"/><text x="371.4" y="320.0" class="t-small" text-anchor="middle">12</text><circle cx="420.0" cy="116.3" r="4" class="f-pk s-pk" stroke-width="1"/><text x="420.0" y="320.0" class="t-small" text-anchor="middle">20</text><text x="225.7" y="195.1" class="t-small" text-anchor="middle">84,19</text><text x="420.0" y="136.3" class="t-small" text-anchor="middle">90,87</text><text x="250.0" y="338.0" class="t-small" text-anchor="middle">độ trễ chốt, tính bằng số điểm (1 điểm = 1 s)</text><rect x="460.0" y="20.0" width="400.0" height="340.0" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="476.0" y="44.0" class="t-b">RMF của lộ trình (0 là hoàn hảo)</text><path d="M514.0 300.0 L844.0 300.0" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="508.0" y="304.0" class="t-small" text-anchor="end">0</text><path d="M514.0 224.1 L844.0 224.1" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="508.0" y="228.1" class="t-small" text-anchor="end">1</text><path d="M514.0 148.3 L844.0 148.3" class="f-none s-line" stroke-width="1" stroke-dasharray="2 4"/><text x="508.0" y="152.3" class="t-small" text-anchor="end">2</text><path d="M514.0 212.6 L844.0 212.6" class="f-none s-warn dash" stroke-width="1.5"/><path d="M514.0 294.5 L844.0 294.5" class="f-none s-phys dash" stroke-width="1.5"/><text x="836.0" y="206.6" class="t-small" text-anchor="end">snap 1,152</text><text x="524.0" y="305.5" class="t-small">offline 0,073</text><path d="M520.0 96.5 L565.7 256.6 L611.4 261.8 L657.1 265.6 L702.9 278.0 L748.6 283.4 L794.3 285.2 L840.0 286.6" class="f-none s-pk" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round"/><circle cx="520.0" cy="96.5" r="4" class="f-pk s-pk" stroke-width="1"/><text x="520.0" y="320.0" class="t-small" text-anchor="middle">0</text><circle cx="565.7" cy="256.6" r="4" class="f-pk s-pk" stroke-width="1"/><text x="565.7" y="320.0" class="t-small" text-anchor="middle">1</text><circle cx="611.4" cy="261.8" r="4" class="f-pk s-pk" stroke-width="1"/><text x="611.4" y="320.0" class="t-small" text-anchor="middle">2</text><circle cx="657.1" cy="265.6" r="4" class="f-pk s-pk" stroke-width="1"/><text x="657.1" y="320.0" class="t-small" text-anchor="middle">3</text><circle cx="702.9" cy="278.0" r="4" class="f-pk s-pk" stroke-width="1"/><text x="702.9" y="320.0" class="t-small" text-anchor="middle">5</text><circle cx="748.6" cy="283.4" r="4" class="f-pk s-pk" stroke-width="1"/><text x="748.6" y="320.0" class="t-small" text-anchor="middle">8</text><circle cx="794.3" cy="285.2" r="4" class="f-pk s-pk" stroke-width="1"/><text x="794.3" y="320.0" class="t-small" text-anchor="middle">12</text><circle cx="840.0" cy="286.6" r="4" class="f-pk s-pk" stroke-width="1"/><text x="840.0" y="320.0" class="t-small" text-anchor="middle">20</text><text x="528.0" y="88.5" class="t-small">trễ 0: 2,683</text><text x="663.1" y="253.6" class="t-small" text-anchor="middle">0,454</text><text x="680.0" y="338.0" class="t-small" text-anchor="middle">độ trễ chốt, tính bằng số điểm</text></svg>
</div>
<figcaption>Hình 2. Quét độ trễ trên 40 chuyến thường (26.146 điểm, σ = 8 m, 1 Hz, tham số HMM mặc định), mục 1 của output <code>loc17_online</code>. Phần lớn lợi ích nằm trong 5–8 điểm đầu; trễ 0 đúng cạnh bằng snap (75,73%) nhưng lộ trình tệ hơn snap (RMF 2,683 so với 1,152). Nhiễu do lab sinh.</figcaption>
</figure>

Phần lớn lợi ích nằm ở 5–8 điểm đầu: cạnh nới nút từ 75,73% lên 86,85% sau 5 giây và 88,68% sau 8 giây (cạnh chặt 55,76%, 69,18%, 70,09%), trong khi từ 8 lên 12 điểm chỉ thêm 1,2 điểm phần trăm, và từ 8 lên 20 điểm thêm tổng cộng 2,2 cạnh nới nút. Cái giá có hai mặt. Một mặt là marker chậm đúng bằng độ trễ. Mặt khác, ước lượng tạm bị sửa nhiều: ở trễ 3 điểm, 13,7% điểm bị đổi cạnh ít nhất một lần và 11,5% có cạnh gán lần đầu khác cạnh chốt. Với app, đó là lý do hiển thị vị trí bám đường cho khách phải dùng điểm đã chốt, hoặc chấp nhận một cú nhảy khi ước lượng bị sửa. Với app gửi mỗi 4 giây như hồ sơ Tới Nơi, trễ 3 điểm là 12 giây thực tế, nên con số «điểm» chỉ chuyển sang «giây» được khi biết nhịp gửi.

## Cầu vượt, đường song song: ca khó của đô thị

Vì sao cầu vượt khó? Trong đồ thị, cầu và đường bên dưới là hai cạnh **không chung nút**: về mặt tô-pô chúng không nối với nhau, dù trên bản đồ phẳng chúng cắt nhau. Snap chỉ nhìn khoảng cách nên coi hai đường là hàng xóm; HMM thì thấy muốn đi từ cầu xuống đường dưới phải vòng tới nút gần nhất, một quãng đường lệch xa so với đường chim bay giữa hai điểm GPS. Đó là cơ chế phân biệt, và nó chỉ phát huy nếu ngữ cảnh đủ dài.

Lab quét đồ thị HCM tìm ca khó bằng hình học và thẻ OSM: **729** giao cắt mà cầu vượt cắt đường ô tô bên dưới, **2.162** cặp đường song song (cách 6–35 m, chạy chung từ 100 m, cùng hạng), **131** cặp đường chính với đường gom, và **22** hầm ô tô dài từ 80 m. Danh mục này có hai điều cần đọc kỹ: «cầu vượt» gồm cả cầu qua kênh và có thể có lỗi vẽ bản đồ, và phần lớn cặp «song song» là hẻm. Từ danh mục, mỗi loại lấy tối đa 12 ca ở chỉ số lẻ (chỉ số chẵn để dành cho chọn tham số về sau và **chưa được dùng**), mỗi ca chạy 5 lần nhiễu σ = 6 m cộng vùng lệch ổn định 12 m quanh điểm khó (mô phỏng hẻm cao tầng, accuracy báo cáo chỉ tăng 1,2 lần). Tham số của ca khó (bán kính 45 m, lệch 12 m) được đặt trước khi xem kết quả. Chấm các điểm trong 45 m quanh điểm khó.

Kết quả «cạnh nới nút», kèm «cạnh chặt» trong ngoặc (nhiễu mô phỏng, cộng vùng lệch 12 m do lab đặt):

| Ca (12 ca × 5 nhiễu) | Số điểm | snap | online trễ 0 | online trễ 3 | online trễ 8 | offline |
|---|---:|---:|---:|---:|---:|---:|
| chạy trên cầu vượt | 688 | 26,60% (25,44%) | 26,60% (25,44%) | 58,58% (57,41%) | 60,47% (58,28%) | 76,60% (75,87%) |
| chạy dưới cầu vượt | 1.384 | 35,77% (24,86%) | 35,77% (24,78%) | 54,55% (39,09%) | 57,23% (43,50%) | 61,27% (46,53%) |
| hai đường song song | 948 | 61,29% (53,90%) | 61,29% (53,90%) | 81,65% (73,84%) | 86,81% (79,64%) | 92,30% (85,02%) |

Cả ba ca đều cho cùng một bức tranh: snap yếu nhất (cầu vượt chỉ 26,60%), chờ 3 điểm tăng rõ (từ 1,3 lần ở đường song song tới 2,2 lần ở cầu vượt, theo cạnh nới nút), offline cao nhất. Ca khó nhất là **chạy dưới cầu vượt**, không phải trên cầu: offline chỉ đạt 61,27% (46,53% chặt). Lab chưa kiểm nguyên nhân; hai giả thuyết là điểm lệch nằm gần cầu hơn gần đường dưới, và xe còn dừng đèn dưới chân cầu (khoảng 22% số điểm của các chuyến thường là lúc xe dừng: 5.686 trên 25.988 điểm đã phân pha ở bài 15). Bài viết không khẳng định nguyên nhân nào.

Hình 3 vẽ một ca cụ thể: cầu vượt An Sương, cạnh 28226, trên cạnh 44669, lấy từ output mục 4 của lab. Ca này được chọn bằng một bộ lọc (tên cầu bắt đầu bằng «Cầu vượt», đường dưới từ hạng tertiary trở lên, cầu dài từ 100 m), không chọn theo kết quả, nhưng vẫn chỉ là **một ca**, không phải thống kê.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 440" role="img" aria-label="Hình 3. Cầu vượt An Sương trên đường bên dưới: 21 điểm GPS của lần nhiễu đầu tiên; snap gán sai cạnh ở 14 điểm, online trễ 3 điểm và offline đúng cả 21."><rect x="20.0" y="20.0" width="560.0" height="390.0" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="36.0" y="44.0" class="t-b">Quanh giao điểm (mét), nhiễu σ = 6 m + lệch 12 m</text><path d="M36.0 354.4 L360.7 63.2" class="f-none s-phys" stroke-width="5"/><path d="M179.6 174.8 L179.9 188.8" class="f-none s-dead" stroke-width="5"/><path d="M179.9 188.8 L185.2 202.9" class="f-none s-dead" stroke-width="5"/><path d="M185.2 202.9 L196.0 215.1" class="f-none s-dead" stroke-width="5"/><path d="M196.0 215.1 L209.5 218.2" class="f-none s-dead" stroke-width="5"/><path d="M209.5 218.2 L223.7 217.8" class="f-none s-dead" stroke-width="5"/><circle cx="80.2" cy="341.8" r="4.5" class="f-warn s-warn" stroke-width="1"/><circle cx="93.4" cy="326.4" r="4.5" class="f-warn s-warn" stroke-width="1"/><circle cx="100.3" cy="312.3" r="4.5" class="f-warn s-warn" stroke-width="1"/><circle cx="104.5" cy="291.8" r="4.5" class="f-dead s-dead" stroke-width="1"/><circle cx="118.9" cy="281.7" r="4.5" class="f-dead s-dead" stroke-width="1"/><circle cx="127.2" cy="270.4" r="4.5" class="f-dead s-dead" stroke-width="1"/><circle cx="134.7" cy="261.6" r="4.5" class="f-dead s-dead" stroke-width="1"/><circle cx="123.2" cy="248.0" r="4.5" class="f-warn s-warn" stroke-width="1"/><circle cx="143.4" cy="229.1" r="4.5" class="f-warn s-warn" stroke-width="1"/><circle cx="158.4" cy="208.1" r="4.5" class="f-warn s-warn" stroke-width="1"/><circle cx="174.0" cy="186.9" r="4.5" class="f-warn s-warn" stroke-width="1"/><circle cx="181.6" cy="177.4" r="4.5" class="f-warn s-warn" stroke-width="1"/><circle cx="200.6" cy="163.9" r="4.5" class="f-warn s-warn" stroke-width="1"/><circle cx="213.4" cy="163.1" r="4.5" class="f-warn s-warn" stroke-width="1"/><circle cx="222.2" cy="151.7" r="4.5" class="f-warn s-warn" stroke-width="1"/><circle cx="226.9" cy="141.2" r="4.5" class="f-warn s-warn" stroke-width="1"/><circle cx="254.3" cy="139.4" r="4.5" class="f-warn s-warn" stroke-width="1"/><circle cx="267.0" cy="129.1" r="4.5" class="f-warn s-warn" stroke-width="1"/><circle cx="279.4" cy="123.8" r="4.5" class="f-dead s-dead" stroke-width="1"/><circle cx="302.8" cy="111.1" r="4.5" class="f-dead s-dead" stroke-width="1"/><circle cx="316.5" cy="100.2" r="4.5" class="f-dead s-dead" stroke-width="1"/><circle cx="196.2" cy="210.8" r="6" class="f-none s-muted dash" stroke-width="1.5"/><text x="42.0" y="408.0" class="t-small">© OpenStreetMap contributors</text><path d="M440.0 402.0 L475.4 402.0" class="f-none s-muted" stroke-width="2"/><text x="483.4" y="406.0" class="t-small">20 m</text><rect x="600.0" y="20.0" width="260.0" height="390.0" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="616.0" y="44.0" class="t-b">Chú giải</text><path d="M616.0 66.0 L646.0 66.0" class="f-none s-phys" stroke-width="5"/><text x="656.0" y="70.0">cầu vượt (cạnh 28226)</text><path d="M616.0 92.0 L646.0 92.0" class="f-none s-dead" stroke-width="5"/><text x="656.0" y="96.0">đường dưới (cạnh 44669)</text><circle cx="631.0" cy="120.0" r="4.5" class="f-warn s-warn" stroke-width="1"/><text x="656.0" y="124.0">điểm snap gán sai cạnh</text><circle cx="631.0" cy="146.0" r="4.5" class="f-dead s-dead" stroke-width="1"/><text x="656.0" y="150.0">điểm snap gán đúng</text><circle cx="631.0" cy="172.0" r="6" class="f-none s-muted dash" stroke-width="1.5"/><text x="656.0" y="176.0">giao điểm hình học</text><text x="616.0" y="218.0" class="t-b">Lần nhiễu đầu tiên (21 điểm)</text><text x="616.0" y="240.0" class="t-mono">snap đúng:        7/21</text><text x="616.0" y="258.0" class="t-mono">online trễ 3 đúng: 21/21</text><text x="616.0" y="276.0" class="t-mono">offline đúng:     21/21</text><text x="616.0" y="312.0" class="t-b">Cả 20 lần nhiễu (204 điểm):</text><text x="616.0" y="332.0" class="t-small">snap 32,35% · online 3: 86,27%</text><text x="616.0" y="348.0" class="t-small">online 8: 90,20% · offline 100,00%</text><text x="616.0" y="380.0" class="t-small">Một ca chọn theo bộ lọc, không</text><text x="616.0" y="394.0" class="t-small">phải thống kê.</text></svg>
</div>
<figcaption>Hình 3. Cầu vượt An Sương (cạnh 28226) trên cạnh 44669, vẽ từ hình học và 21 điểm GPS mà mục 4 của output <code>loc17_online</code> in ra; xe chạy trên cầu. Điểm màu cảnh báo là điểm snap gán sang đường dưới hoặc cạnh khác. Cạnh 44669 chỉ dài vài chục mét; đường dưới còn nối tiếp bằng các cạnh khác (snap gán 44666, 44671, 75529…) mà output không in hình học nên hình không vẽ. Nhiễu mô phỏng. © OpenStreetMap contributors (ODbL).</figcaption>
</figure>

Trên 20 lần nhiễu của ca này (204 điểm), snap đúng 32,35%, online trễ 3 điểm đạt 86,27%, trễ 8 điểm 90,20%, offline 100,00%; ở ca này cạnh chặt và cạnh nới nút trùng nhau, và ground truth vẫn là của simulator. Ở lần nhiễu đầu tiên (21 điểm), snap đúng 7 điểm và gán sai 14 điểm sang cạnh khác; online trễ 3 điểm giữ trên cầu ở cả 21 điểm. Điểm đáng nói là một mô hình không có thông tin độ cao nào («cầu» hay «đường dưới») vẫn tìm ra đúng nhờ chuỗi điểm và hình học mạng.

## Offline không phải lúc nào cũng chính xác hơn: đường gom

Hai ca đường gom và ca hầm (cùng nhiễu mô phỏng như bảng trên) đem lại một kết quả ngược với trực giác «nhìn càng nhiều càng đúng»:

| Ca (12 ca × 5 nhiễu) | Số điểm | snap | online trễ 3 | online trễ 8 | offline |
|---|---:|---:|---:|---:|---:|
| đường chính, có đường gom bên cạnh | 662 | 55,14% (44,26%) | **81,57%** (72,81%) | 76,89% (68,13%) | 76,28% (66,31%) |
| đường gom bên cạnh đường chính | 1.239 | 65,21% (59,00%) | 78,69% (68,28%) | **83,86%** (76,43%) | 81,28% (74,90%) |
| hầm (mất tín hiệu) | 224 | 59,82% (36,16%) | 66,96% (54,91%) | 67,86% (46,43%) | 78,12% (58,93%) |

Với đường chính có đường gom bên cạnh, **offline (76,28%) thấp hơn online trễ 3 điểm (81,57%)**, và thậm chí thấp hơn trễ 8 điểm (76,89%). Với đường gom bên đường chính, online trễ 8 (83,86%) hơn offline (81,28%). Ở hầm, thước chặt của trễ 3 (54,91%) cao hơn của trễ 8 (46,43%).

Vì sao? Viterbi tìm chuỗi **có xác suất cao nhất theo mô hình**, không phải chuỗi gần sự thật nhất. Mô hình coi sai số mỗi điểm là độc lập, nhưng vùng lệch ổn định 12 m của lab (và có thể cả của hẻm cao tầng thật) đẩy **cả đoạn** về cùng một phía. Nếu hướng lệch đưa các điểm về phía đường kia, càng nhiều điểm trong cửa sổ thì càng nhiều bằng chứng nhỏ cộng dồn cho đường sai, và chuỗi xác suất cao nhất trở thành chuỗi sai. Cơ chế này hợp lý nhưng **chưa được kiểm riêng** (lab không có lần chạy tắt độ lệch ổn định cho riêng ca này). Cũng cần cẩn thận với cỡ mẫu: mỗi loại chỉ có 12 ca và 5 lần nhiễu của cùng một ca không độc lập, nên chênh lệch vài điểm phần trăm trên 662 điểm không đủ để kết luận rằng trễ 3 «tốt hơn» offline. Điều có thể kết luận chắc là **offline không bảo đảm chính xác hơn online** ở những ca mà lỗi là hệ thống chứ không phải ngẫu nhiên; ở hai ca cầu vượt và đường song song, offline vẫn thắng rõ.

Bài học thiết kế: đừng gộp «chờ lâu hơn» với «đúng hơn». Với những ca mà nhiễu có tương quan mạnh, cần nguồn thông tin khác (heading, tốc độ, ràng buộc cầu vượt qua `layer` của OSM) chứ không phải thêm cửa sổ. Lab có số hạng hướng đi nhưng tắt mặc định, vì chưa kiểm với heading thật.

## Mất tín hiệu: gãy chuỗi và nội suy theo đường

Khi xe vào hầm hay tín hiệu rơi vài chục giây, không có điểm nào để nối. `extend` quyết định: khoảng cách thời gian không quá `gap_break_s` (90 giây) thì thử nối, nhưng đường đi phải nằm trong giới hạn `min(chim bay + 6β + 40 m, vận tốc tối đa × thời gian + 40 m)` của bài 16. Vượt `gap_break_s`, hoặc không cặp trạng thái nào lọt giới hạn đó, thì gãy chuỗi, chốt hết đoạn cũ và bắt đầu đoạn mới. Giới hạn đầu khá chặt: đường đi vòng hơn chim bay quá 46 m thì không nối được, kể cả khi thời gian còn dư.

Tệp `examples/toi-noi-location/rust/crates/map-match/src/hmm.rs` (dòng 215–226):

```rust
        if !states.is_empty()
            && let Some(p) = prev.filter(|p| p.states.iter().any(State::alive))
        {
            let dt = ((obs.t_ms - p.obs.t_ms) as f64 / 1000.0).max(0.2);
            if dt > self.cfg.gap_break_s || !self.link(p, &obs, &mut states, dt) {
                broke = true;
                self.stats.breaks += 1;
                for s in &mut states {
                    s.back = NO_BACK;
                }
            }
        }
```

Chuỗi gãy có hệ quả đo được. Ở ca hầm (11 ca × 5 nhiễu), recall lộ trình của offline chỉ 71,55% (gãy 18 lần); nếu nâng `gap_break_s` lên 300 giây (**chỉ để biết độ nhạy, không dùng để chọn tham số**) thì recall lên 76,70% và gãy còn 8 lần. Với mất tín hiệu 25 giây trên chuyến thường, online trễ 8 điểm đạt 94,21% cạnh nới nút và RMF 0,139.

Vị trí xe **trong** khoảng trống thì nội suy được, nhưng chỉ sau khi tín hiệu quay lại, vì cần điểm sau. `interpolate_gap` đi dọc đường ngắn nhất giữa hai điểm đã bám, đều theo thời gian:

Tệp `examples/toi-noi-location/rust/crates/map-match/src/hmm.rs` (dòng 417–449):

```rust
    pub fn interpolate_gap(
        &mut self,
        a: &MatchedPoint,
        b: &MatchedPoint,
        fracs: &[f64],
    ) -> Vec<LatLng> {
        let edges = self.path_between(a, b);
        let mut pts: Vec<LatLng> = Vec::new();
        let mut before_last = 0.0;
        for (k, &e) in edges.iter().enumerate() {
            let ep = self.g.edge_points(e);
            if k + 1 == edges.len() {
                before_last = geo_core::segment::polyline_length_m(&pts);
            }
            for p in ep {
                if pts.last() != Some(&p) {
                    pts.push(p);
                }
            }
        }
        let start = f64::from(a.offset_m);
        let end = if edges.len() == 1 {
            f64::from(b.offset_m)
        } else {
            before_last + f64::from(b.offset_m)
        };
        fracs
            .iter()
            .filter_map(|&f| {
                geo_core::segment::point_at(&pts, start + (end - start) * f.clamp(0.0, 1.0))
            })
            .collect()
    }
```

Lab so sai số nội suy theo đường với cách nối thẳng hai điểm thô, trên các khoảng trống từ 6 giây (kết quả online trễ 8 điểm), mét, p50 / p95 / lớn nhất:

| Loại | Số khoảng | Số điểm | Nội suy theo đường | Nối thẳng |
|---|---:|---:|---:|---:|
| hầm | 38 | 2.161 | 7,8 / 153,7 / 252,4 | 32,6 / 202,2 / 234,0 |
| mất tín hiệu 25 giây | 59 | 1.477 | 6,4 / 47,1 / 97,2 | 10,0 / 40,5 / 85,3 |

Nội suy theo đường tốt hơn rõ ở trung vị (7,8 m so với 32,6 m ở hầm), nhưng **ở đuôi thì không chắc**: p95 ở hầm có tốt hơn (153,7 m so với 202,2 m), còn p95 và giá trị lớn nhất của mất tín hiệu 25 giây thậm chí kém hơn nối thẳng (47,1 m so với 40,5 m; 97,2 m so với 85,3 m), và lớn nhất ở hầm cũng kém (252,4 m so với 234,0 m). Hầm dài vẫn lệch cả trăm mét ở p95. Nội suy theo đường chọn đúng đường nhưng chia thời gian đều dọc đường, trong khi xe có thể dừng hay chạy nhanh trong lúc đó; chưa có mô hình tốc độ trong khoảng trống.

## Ngân sách CPU và bộ nhớ theo tài xế

Hồ sơ Tới Nơi là 25.000 tài xế, mỗi 4 giây một điểm, tức 6.250 điểm/giây. Lab cho 400 tài xế chạy xen kẽ từng điểm trên **một luồng**, mỗi tài xế một `OnlineMatcher`, và đo từng lần `push`. Bộ nhớ mỗi tài xế là ước lượng theo `capacity` của cửa sổ và các lớp (`mem_bytes`), không phải RSS đo từ hệ điều hành:

| Trễ (điểm) | Bộ nhớ/tài xế | Bộ nhớ lớn nhất | 25.000 tài xế |
|---:|---:|---:|---:|
| 0 | 504 B | 504 B | 12,6 MB |
| 3 | 2.785 B | 3.072 B | 69,6 MB |
| 8 | 7.850 B | 8.696 B | 196,2 MB |
| 20 | 18.420 B | 20.760 B | 460,5 MB |

Độ trễ quyết định tính bằng điểm trả bằng bộ nhớ gần tuyến tính, vì mỗi điểm chờ giữ một lớp ứng viên. Thời gian thì **rất nhiễu**: lab chạy mục này nhiều lần cạnh nhau, và tải máy lúc đó từ 40 tới hơn 169 trên 10 lõi. Dưới đây là trễ 3 điểm, `max_settled` 4.000, qua bốn lần:

| Lần chạy | Tải 1 phút | µs/điểm (trung bình) | p50 (µs) | p99 (µs) | Lớn nhất (µs) |
|---|---:|---:|---:|---:|---:|
| chính thức | 67,87 | 53,96 | 10,83 | 131,50 | 75.052,5 |
| lần 2 | 166,28 | 33,00 | 11,00 | 109,79 | 102.582,3 |
| lần 3 | 169,09 | 11,50 | 10,25 | 27,67 | 5.008,0 |
| lúc phát triển | 39,78 | 11,37 | 9,50 | 29,62 | 11.602,0 |

Trung bình đi từ 11,4 tới 54 µs, và «lớn nhất» tới hơn 100 ms. Không có quan hệ đơn giản giữa tải và số đo: lần tải 169 lại có «lớn nhất» thấp nhất (5,0 ms) còn lần tải 68 có trung bình cao nhất, nên các cực đại hơn 100 ms khó là của thuật toán; ta quy cho nhiễu hệ điều hành nhưng chưa kiểm riêng. Trung vị ổn định hơn nhiều, 9,5 tới 11,0 µs ở trễ 3 qua các lần. Với số trung bình của lần nhanh nhất, 6.250 điểm/giây × 11,37 µs = 0,071 lõi cho 25.000 tài xế; với lần chậm nhất là 0,337 lõi. Kết luận chắc chỉ là bậc độ lớn: **từ khoảng 0,07 đến 0,34 lõi** trên máy này, chưa tính I/O Kafka, giữ đồ thị trong RAM và tranh chấp cache khi chạy cùng nhiều luồng khác.

Trần ngân sách `max_settled` (số nút tối đa mỗi lần Dijkstra) **không phải nút thắt** ở mức nhiễu này: ngay cả khi chỉ cho 4 nút (chạm trần 651.362 lần) cạnh nới nút vẫn 83,25%, so với 83,24% khi cho 4.000 (mục này của lab chỉ in cạnh nới nút, không in cạnh chặt). Lab chưa thử nhiễu lớn hơn hay đồ thị thưa hơn, nên đây chưa phải kết luận chung.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin`. Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc17_online -- --reps 5
```

Cấu hình đo: `--release`, Apple M1 Max, đồ thị HCM 126.699 nút và 319.074 cạnh có hướng (© OpenStreetMap contributors), seed báo cáo 20261106, `MatchConfig::default()`. Output (rút gọn: danh mục ca khó, hai hàng đầu và hàng offline của bảng độ trễ, bảng CPU):

```text
-- 0. ca khó tìm được trên đồ thị (quét hình học và thẻ, 0.5 s) --
cầu vượt cắt đường ô tô bên dưới: 729; cặp đường song song (6–35 m, ≥100 m, cùng hạng): 2162; cặp đường chính + đường gom: 131; hầm cho ô tô (≥80 m): 22
…
-- 1. độ trễ quyết định (chuyến thường, σ = 8 m, 1 Hz, 26146 điểm): chờ thêm L điểm thì chốt chắc hơn --
                               cạnh chặt  cạnh nới  chiều nới  recall    RMF   số điểm   %sửa  %đầu-sai  trễ p50/p95 (s)
snap về cạnh gần nhất             55.76%    75.73%    64.25%    94.85%   1.152    26146
online, trễ 0 điểm                55.76%    75.73%    64.25%    99.74%   2.683    26146    0.0%     0.0%    0.0 /  0.0
…
offline (nhìn cả chuyến)          73.47%    92.53%    88.84%    99.20%   0.073    26146
(%sửa: điểm có ước lượng tạm đổi cạnh ít nhất một lần trước khi chốt; %đầu-sai: cạnh gán lần đầu khác cạnh chốt)
…
-- 3. ngân sách CPU theo tài xế (online, 400 tài xế xen kẽ từng điểm, σ = 8 m, 1 luồng) --
uptime: 23:24  up 35 days, 15:08, 1 user, load averages: 67.87 67.34 72.42
     trễ  max_settled      µs/điểm   p50 (µs)   p99 (µs) tối đa (µs)   cạnh nới    chạm trần bộ nhớ/xe (TB, tối đa)
       0         4000        35.01       7.12      53.96     36573.9     76.75%            0       504 B, 504 B
       3         4000        53.96      10.83     131.50     75052.5     83.24%            0      2785 B, 3072 B
       8         4000        39.87      12.25      98.58     41500.9     88.25%            0      7850 B, 8696 B
      20         4000        21.31      15.29      96.46     45994.1     90.58%            0     18420 B, 20760 B
```

Số đếm và tỉ lệ (độ chính xác, RMF, số khoảng, số lần gãy) tái lập từng chữ số với cùng seed; số µs và «tối đa» thì nhiễu mạnh, và bài này luôn ghi tải máy cạnh mỗi lần đo. Mục «độ nhạy» `gap_break_s = 300` không dùng để chọn tham số. Kiểm tra mã: `cargo test --manifest-path examples/toi-noi-location/rust/Cargo.toml -p map-match` gồm test «mỗi điểm được chốt đúng một lần theo thứ tự», «lộ trình liền mạch khi không gãy chuỗi» và «độ trễ vô hạn trùng offline». Test hồi quy `hard_cases` kiểm HMM thắng snap trên ca khó của fixture (ngưỡng đặt rộng, đặt sau khi xem một lần chạy). Toàn phần: `bash examples/toi-noi-location/verify-p02-rust.sh`.

## Giới hạn của ví dụ

- **Không có GPS người thật**, và lệch ổn định 12 m là mô hình của lab về hẻm cao tầng, không phải số đo. Kết luận «offline không hơn online ở đường gom» là kết luận về mô hình này.
- **Cỡ mẫu nhỏ ở ca khó**: tối đa 12 ca mỗi loại, 5 lần nhiễu của một ca không độc lập; hầm chỉ có 11 ca. Danh mục ca có thể có lỗi vẽ bản đồ, và chưa duyệt từng ca bằng mắt.
- **Chưa kiểm** nguyên nhân vì sao «chạy dưới cầu vượt» khó nhất, và nguyên nhân của chênh lệch online/offline ở đường gom.
- **Chỉ profile ô tô; bỏ qua cấm rẽ.** Chưa có bám đường xe máy, nơi hẻm hẹp và đường song song khác ô tô.
- **Chưa dùng `layer` của OSM** (cầu, hầm) để ngăn nhảy giữa cầu và đường dưới; mô hình chỉ dựa vào chuỗi điểm và tô-pô mạng.
- **Chưa chạy** (không có trong lab này): Kafka rebalance giữa chừng, chuyển quyền sở hữu tài xế giữa hai tiến trình geo-engine khi cửa sổ đang mở, và đo trên nhiều luồng; chỉ có đo một luồng, 400 tài xế xen kẽ.
- **Thời gian CPU nhiễu** do máy chịu tải; chỉ có giá trị bậc độ lớn.

## Bài tập

**Bài 1.** App gửi mỗi 4 giây thay vì 1 giây. Độ trễ quyết định 3 điểm là bao nhiêu giây, và bộ nhớ mỗi tài xế có đổi không? Bạn chọn trễ nào nếu muốn marker chậm không quá 8 giây?

<details><summary>Lời giải bài 1</summary>
<p>Trễ 3 điểm ở nhịp 4 giây là 12 giây; bộ nhớ không đổi vì tính theo số lớp (2.785 B mỗi tài xế), không theo giây. Để chậm không quá 8 giây chỉ có thể chờ 1 điểm (4 giây) hoặc 2 điểm (8 giây). Ở nhịp 1 giây, trễ 2 điểm cho cạnh nới nút 81,71%, cạnh chặt 64,69%; con số ở nhịp 4 giây phải đo lại vì mỗi điểm cách xa hơn 4 lần, nên không suy từ bảng này được.</p>
</details>

**Bài 2.** Ở đường chính có đường gom, vì sao không nên kết luận «trễ 3 điểm tốt hơn offline» dù 81,57% lớn hơn 76,28% (cạnh chặt 72,81% so với 66,31%)?

<details><summary>Lời giải bài 2</summary>
<p>Có hai lý do: mỗi loại chỉ có 12 ca và 5 lần nhiễu của cùng một ca không độc lập, nên cỡ mẫu hiệu dụng gần 12; và lab chưa kiểm cơ chế (độ lệch ổn định đẩy cả đoạn về phía đường kia). Một kết luận an toàn hơn là «offline không đảm bảo hơn online khi lỗi là hệ thống». Muốn kiểm, tăng số ca bằng cách dùng cả danh mục chỉ số chẵn (hiện chưa dùng) và so với một lần chạy tắt độ lệch ổn định.</p>
</details>

**Bài 3.** Nội suy theo đường ở hầm có p50 tốt hơn nối thẳng nhưng p95 và giá trị lớn nhất kém hơn ở ca 25 giây. Đề xuất một thay đổi nhỏ và cách đo.

<details><summary>Lời giải bài 3</summary>
<p>Thay việc chia thời gian đều dọc đường bằng một mô hình tốc độ trong khoảng trống (ví dụ nội suy theo vận tốc trước và sau khoảng trống, có tính dừng đèn). Đo bằng đúng bảng 2b của lab: p50/p95/lớn nhất so với nối thẳng, trên cùng seed và cùng các khoảng trống, và ghi cả khoảng nào nội suy kém hơn nối thẳng. Chỉ khi p95 và lớn nhất cũng không kém hơn mới nên dùng thay thế.</p>
</details>

Bài trước: [bài 16](/articles/location-map-matching-hmm-viterbi-rust). Bài tiếp: [bài 18](/articles/location-kho-vi-tri-moi-nhat-h3-redis-geo-kafka).

## Tài liệu gốc

- Paul Newson, John Krumm, [«Hidden Markov Map Matching Through Noise and Sparseness»](https://www.microsoft.com/en-us/research/publication/hidden-markov-map-matching-noise-sparseness/), ACM SIGSPATIAL GIS 2009: mô hình HMM cho bám đường. Bài báo trình bày thuật toán như bài toán xử lý cả lô sau khi đã thu hết dữ liệu và chỉ phỏng đoán rằng một phiên bản cửa sổ trượt sẽ hợp với thời gian thực; cơ chế cửa sổ, cam kết và các số đo ở bài này là phần của lab, trên dữ liệu mô phỏng.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho dữ liệu đồ thị HCM.
