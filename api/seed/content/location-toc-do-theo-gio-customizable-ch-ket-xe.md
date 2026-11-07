---
slug: location-toc-do-theo-gio-customizable-ch-ket-xe
title: "Tốc độ theo giờ và kẹt xe: Customizable CH, đổi trọng số không dựng lại"
summary: "Bài 27: hồ sơ kẹt xe theo giờ (tổng hợp, không phải dữ liệu thật), Customizable CH tách thứ tự khỏi trọng số, kết quả đúng nhưng truy vấn chậm hơn CH thường 31–35 lần, và ETA tĩnh sai bao nhiêu."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, routing, customizable-ch, traffic, time-dependent]
language: vi
---

Lúc 3 giờ sáng, chuyến từ góc này của thành phố sang góc kia mất 37,0 phút. Lúc 8 giờ sáng, chính chuyến đó mất 67,9 phút, gấp 1,83 lần. Nếu engine luôn báo 37 phút cho khách đặt xe lúc cao điểm, lời hứa đó sai cả nửa giờ. Số liệu ở trên là của hồ sơ kẹt xe **tổng hợp** mà lab tự đặt; nó cho thấy cỡ của hiệu ứng nếu giả định đúng, chứ không nói thành phố thật kẹt thế nào. Bài này nói về cách để engine đổi trọng số theo giờ mà không phải dựng lại toàn bộ CH mỗi lần, và về giá phải trả cho cách đó, vốn không nhỏ.

Đây là bài 27 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng ma trận của [bài 26](/articles/location-ma-tran-many-to-many-bucket-ch) và CH của [bài 25](/articles/location-contraction-hierarchies-tu-cai-dat-fast-paths). Mọi số đo trên đồ thị HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors), hồ sơ ô tô.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>CH của cả thành phố dựng trong khoảng 1,1–1,3 giây (bài 25). Trọng số đổi theo 24 giờ. Bạn chọn cách nào: dựng sẵn 24 CH, hay dùng Customizable CH để chỉ tính lại trọng số khi giờ đổi? Dự đoán Customizable CH nhanh hơn bao nhiêu lần ở bước đổi trọng số và chậm hơn bao nhiêu ở truy vấn.</p>
<details><summary>Xem đáp án</summary><p>Ở lab này, tùy biến trọng số mất trung bình 355 ms mỗi giờ, so với 1.614 ms dựng lại CH thường: nhanh hơn 3,7–4,5 lần qua hai lần chạy, không phải vài bậc độ lớn. Cái giá: truy vấn khoảng cách chậm hơn CH thường <b>31,1–34,8 lần</b> (234,2 µs so với 7,5 µs ở lần chạy chính). Với bản đồ cỡ này và 24 giờ cố định, dựng sẵn một CH cho mỗi giờ đơn giản hơn và truy vấn nhanh hơn nhiều; Customizable CH chỉ đáng dùng khi trọng số đổi liên tục và có thứ tự nút tốt hơn thứ tự của lab.</p></details>
</div>

## Hồ sơ tốc độ theo giờ, và vì sao nó là tổng hợp

Lab không có dữ liệu giao thông thật, nên `traffic.rs` tự đặt một mô hình để có thứ mà thử thuật toán cập nhật trọng số. Mỗi cạnh có tốc độ tự do theo loại đường (bảng của LOC-24). Tốc độ ở giờ `h` bằng tốc độ tự do nhân một hệ số, hệ số này là `1 − chỉ_số_kẹt(h) × độ_nặng`, kẹp tối thiểu 0,2. Chỉ số kẹt có hai đỉnh sáng và chiều:

`examples/toi-noi-location/rust/crates/route-engine/src/traffic.rs`

```rust
pub const CONGESTION_BY_HOUR: [f64; SLOTS] = [
    0.05, 0.0, 0.0, 0.0, 0.0, 0.1, 0.3, 0.7, 1.0, 0.7, 0.4, 0.35, 0.4, 0.4, 0.35, 0.4, 0.6, 0.9,
    1.0, 0.8, 0.5, 0.3, 0.15, 0.1,
];
```

Độ nặng phụ thuộc loại đường và việc cạnh có nằm trong vùng trung tâm (bán kính 5 km quanh Chợ Bến Thành, 109.905 trên 319.074 cạnh) hay không. Xe máy chịu ảnh hưởng bằng 0,7 lần ô tô «vì luồn lách», đi bộ không bị ảnh hưởng. Đó là các **giả định của lab**, không đo từ đâu cả:

`examples/toi-noi-location/rust/crates/route-engine/src/traffic.rs`

```rust
    fn severity(&self, class: u8, edge: usize, profile: Profile) -> f64 {
        use HighwayClass::*;
        let c = self.central[edge];
        let base = match HighwayClass::from_u8(class) {
            Some(
                Motorway | MotorwayLink | Trunk | TrunkLink | Primary | PrimaryLink | Secondary
                | SecondaryLink,
            ) => {
                if c {
                    0.65
                } else {
                    0.45
                }
            }
            Some(Tertiary | TertiaryLink | Unclassified) => {
                if c {
                    0.55
                } else {
                    0.35
                }
            }
            _ => {
                if c {
                    0.35
                } else {
                    0.20
                }
            }
        };
        match profile {
            Profile::Car => base,
            Profile::Motorbike => base * 0.7, // xe máy luồn lách nên ít bị ảnh hưởng hơn
            Profile::Foot => 0.0,
        }
    }
```

Tốc độ trung bình theo độ dài trên mọi cạnh ô tô (kể cả hẻm, nên chỉ khoảng 16 km/h ở giờ thông thoáng) đi từ 16,04 km/h lúc 1–4 giờ xuống 11,58 km/h lúc 8 giờ và 18 giờ, tức 72% so với giờ thông thoáng. Hình 1 vẽ thời gian của một chuyến cố định (nút 47715 tới nút 17989, chim bay 20.575 m, chuyến xa nhất trong mẫu) theo giờ khởi hành.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 1. Thời gian một chuyến cố định (nút 47715 đến nút 17989, chim bay 20.575 m) theo giờ khởi hành trong hồ sơ kẹt xe tổng hợp: 37,0 phút lúc 1 đến 4 giờ sáng, 67,9 phút lúc 8 giờ và 18 giờ."><rect x="20" y="20" width="840" height="300" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Một chuyến cố định theo giờ khởi hành (phút), hồ sơ kẹt xe TỔNG HỢP của lab</text><rect x="52.0" y="172.3" width="22" height="97.7" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="63.0" y="166.3" class="t-small" text-anchor="middle">38,0</text><text x="63.0" y="288" class="t-small" text-anchor="middle">00</text><rect x="84.5" y="174.9" width="22" height="95.1" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="95.5" y="168.9" class="t-small" text-anchor="middle">37,0</text><text x="95.5" y="288" class="t-small" text-anchor="middle">01</text><rect x="117.0" y="174.9" width="22" height="95.1" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="128.0" y="168.9" class="t-small" text-anchor="middle">37,0</text><text x="128.0" y="288" class="t-small" text-anchor="middle">02</text><rect x="149.5" y="174.9" width="22" height="95.1" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="160.5" y="168.9" class="t-small" text-anchor="middle">37,0</text><text x="160.5" y="288" class="t-small" text-anchor="middle">03</text><rect x="182.0" y="174.9" width="22" height="95.1" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="193.0" y="168.9" class="t-small" text-anchor="middle">37,0</text><text x="193.0" y="288" class="t-small" text-anchor="middle">04</text><rect x="214.5" y="169.5" width="22" height="100.5" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="225.5" y="163.5" class="t-small" text-anchor="middle">39,1</text><text x="225.5" y="288" class="t-small" text-anchor="middle">05</text><rect x="247.0" y="158.4" width="22" height="111.6" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="258.0" y="152.4" class="t-small" text-anchor="middle">43,4</text><text x="258.0" y="288" class="t-small" text-anchor="middle">06</text><rect x="279.5" y="128.1" width="22" height="141.9" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="290.5" y="122.1" class="t-small" text-anchor="middle">55,2</text><text x="290.5" y="288" class="t-small" text-anchor="middle">07</text><rect x="312.0" y="95.4" width="22" height="174.6" rx="3" class="f-warn s-warn" stroke-width="1.5"/><text x="323.0" y="89.4" class="t-small" text-anchor="middle">67,9</text><text x="323.0" y="288" class="t-small" text-anchor="middle">08</text><rect x="344.5" y="128.1" width="22" height="141.9" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="355.5" y="122.1" class="t-small" text-anchor="middle">55,2</text><text x="355.5" y="288" class="t-small" text-anchor="middle">09</text><rect x="377.0" y="152.0" width="22" height="118.0" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="388.0" y="146.0" class="t-small" text-anchor="middle">45,9</text><text x="388.0" y="288" class="t-small" text-anchor="middle">10</text><rect x="409.5" y="155.3" width="22" height="114.7" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="420.5" y="149.3" class="t-small" text-anchor="middle">44,6</text><text x="420.5" y="288" class="t-small" text-anchor="middle">11</text><rect x="442.0" y="152.0" width="22" height="118.0" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="453.0" y="146.0" class="t-small" text-anchor="middle">45,9</text><text x="453.0" y="288" class="t-small" text-anchor="middle">12</text><rect x="474.5" y="152.0" width="22" height="118.0" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="485.5" y="146.0" class="t-small" text-anchor="middle">45,9</text><text x="485.5" y="288" class="t-small" text-anchor="middle">13</text><rect x="507.0" y="155.3" width="22" height="114.7" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="518.0" y="149.3" class="t-small" text-anchor="middle">44,6</text><text x="518.0" y="288" class="t-small" text-anchor="middle">14</text><rect x="539.5" y="152.0" width="22" height="118.0" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="550.5" y="146.0" class="t-small" text-anchor="middle">45,9</text><text x="550.5" y="288" class="t-small" text-anchor="middle">15</text><rect x="572.0" y="136.8" width="22" height="133.2" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="583.0" y="130.8" class="t-small" text-anchor="middle">51,8</text><text x="583.0" y="288" class="t-small" text-anchor="middle">16</text><rect x="604.5" y="107.7" width="22" height="162.3" rx="3" class="f-warn s-warn" stroke-width="1.5"/><text x="615.5" y="101.7" class="t-small" text-anchor="middle">63,1</text><text x="615.5" y="288" class="t-small" text-anchor="middle">17</text><rect x="637.0" y="95.4" width="22" height="174.6" rx="3" class="f-warn s-warn" stroke-width="1.5"/><text x="648.0" y="89.4" class="t-small" text-anchor="middle">67,9</text><text x="648.0" y="288" class="t-small" text-anchor="middle">18</text><rect x="669.5" y="118.5" width="22" height="151.5" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="680.5" y="112.5" class="t-small" text-anchor="middle">58,9</text><text x="680.5" y="288" class="t-small" text-anchor="middle">19</text><rect x="702.0" y="144.8" width="22" height="125.2" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="713.0" y="138.8" class="t-small" text-anchor="middle">48,7</text><text x="713.0" y="288" class="t-small" text-anchor="middle">20</text><rect x="734.5" y="158.4" width="22" height="111.6" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="745.5" y="152.4" class="t-small" text-anchor="middle">43,4</text><text x="745.5" y="288" class="t-small" text-anchor="middle">21</text><rect x="767.0" y="166.9" width="22" height="103.1" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="778.0" y="160.9" class="t-small" text-anchor="middle">40,1</text><text x="778.0" y="288" class="t-small" text-anchor="middle">22</text><rect x="799.5" y="169.5" width="22" height="100.5" rx="3" class="f-pk s-pk" stroke-width="1.5"/><text x="810.5" y="163.5" class="t-small" text-anchor="middle">39,1</text><text x="810.5" y="288" class="t-small" text-anchor="middle">23</text><text x="40" y="310" class="t-small">Giờ khởi hành (0–23). Cột cam: từ 60 phút trở lên. Hệ số kẹt xe do lab tự đặt, không đo từ xe thật.</text></svg>
</div>
<figcaption>Hình 1. Thời gian một chuyến cố định theo giờ khởi hành, tính trên hồ sơ kẹt xe tổng hợp: 37,0 phút lúc 1–4 giờ, 67,9 phút lúc 8 giờ và 18 giờ. Đồ thị © OpenStreetMap contributors; hệ số kẹt xe do lab tự đặt.</figcaption>
</figure>

| Giờ | Tốc độ TB ô tô (km/h) | So với giờ thông thoáng |
|---:|---:|---:|
| 3 | 16,04 | 100% |
| 8 | 11,58 | 72% |
| 13 | 14,34 | 89% |
| 18 | 11,58 | 72% |
| 22 | 15,41 | 96% |

Giờ 8 và giờ 18 ra cùng số vì chỉ số kẹt của hai giờ này đều bằng 1,0 trong bảng giả định. Đừng đọc bảng này như hình dạng kẹt xe thật của TP.HCM.

Một điều nữa cần nói rõ: «theo giờ» ở đây nghĩa là **cả chuyến dùng trọng số của giờ khởi hành**. Chuyến 67,9 phút xuất phát lúc 8 giờ vẫn được tính bằng hệ số của giờ 8 cho phần chạy sau 9 giờ. Đó là một đồ thị tĩnh cho mỗi khung giờ, chưa phải định tuyến phụ thuộc thời gian thật sự, nơi trọng số của một cạnh phụ thuộc lúc xe tới cạnh đó.

## Vì sao CH thường không hợp với trọng số đổi

Thứ tự co và tập shortcut của CH được chọn dựa trên trọng số: tìm kiếm nhân chứng quyết định shortcut nào cần thiết, và thứ tự ưu tiên dùng số shortcut sinh ra. Đổi trọng số nghĩa là các nhân chứng cũ có thể không còn là nhân chứng, nên cách chắc chắn là dựng lại. Với bản đồ này, dựng lại cho cùng 24 giờ mất trung bình 1.614 ms mỗi giờ (nhỏ nhất 1.156, lớn nhất 5.367 ms) trong lần chạy chính; qua hai lần chạy trung bình mỗi giờ là 1.341–1.614 ms.

Chi phí đó nhỏ, nên phương án «dựng sẵn một CH cho mỗi khung giờ» khả thi: mỗi CH ô tô của bài 25 nặng 8,53 MB, nên 24 CH cỡ 205 MB (ước tính nhân thẳng, chưa đo). Khi trọng số đổi liên tục, ví dụ cập nhật kẹt xe trực tiếp mỗi phút, không thể dựng sẵn nữa, mà phải dựng lại trong lúc chạy. Đó là chỗ Customizable CH muốn giải quyết.

## Customizable CH: tách thứ tự khỏi trọng số

Customizable Contraction Hierarchies (CCH) tách việc dựng thành ba pha, và chỉ pha cuối phụ thuộc trọng số:

1. **Thứ tự** (một lần, chỉ phụ thuộc hình dạng đồ thị): chia đệ quy theo tọa độ, kiểu chia lồng nhau đơn giản. Mỗi lần cắt đôi theo trục dài, rồi lấy một «vách ngăn» là tập phủ đỉnh nhỏ nhất của các cạnh cắt (tìm bằng ghép cặp cực đại và định lý Kőnig) và xếp vách ngăn *sau* hai nửa. Bỏ vách ngăn thì hai nửa không còn cạnh nối nhau, nên co nút trong một nửa không bao giờ sinh shortcut sang nửa kia; shortcut chỉ dồn về vách ngăn. Vì vậy cỡ vách ngăn quyết định số cung.
2. **Cấu trúc** (một lần): co nút theo thứ tự đó **không cần tìm nhân chứng**, nên tập shortcut không phụ thuộc trọng số.
3. **Tùy biến** (mỗi lần đổi trọng số): duyệt các tam giác theo hạng và cập nhật `w(y,z) = min(w(y,z), w(y,x) + w(x,z))`.

Pha 3 là vòng lặp ba tầng sau: với mỗi nút `x`, mỗi cặp hàng xóm đi lên `(y, z)` tạo một «tam giác» `x, y, z`, và cung `y–z` có thể được cải thiện qua `x`. Vì không bỏ shortcut nào, cung `y–z` luôn tồn tại. Vì duyệt `x` theo hạng tăng dần, khi tới `x` thì `w(y,x)` và `w(x,z)` đã nhận đủ cải thiện từ mọi nút thấp hơn. Sau vòng lặp, mỗi cung mang chi phí tốt nhất qua các nút hạng thấp hơn nó, đúng điều truy vấn đi lên rồi đi xuống của bài 25 cần.

`examples/toi-noi-location/rust/crates/route-engine/src/cch.rs`

```rust
        let n = self.up_first.len() - 1;
        for x in 0..n as u32 {
            let (s, t) = (
                self.up_first[x as usize] as usize,
                self.up_first[x as usize + 1] as usize,
            );
            for i in s..t {
                let y = self.up_head[i];
                let (dn_xy, up_xy) = (w_dn[i], w_up[i]);
                for j in i + 1..t {
                    let z = self.up_head[j];
                    let a = self.find(y, z);
                    // y → x → z
                    let c1 = dn_xy.saturating_add(w_up[j]);
                    if c1 < w_up[a] {
                        w_up[a] = c1;
                        mid_up[a] = x;
                        e_up[a] = NONE;
                    }
                    // z → x → y
                    let c2 = w_dn[j].saturating_add(up_xy);
                    if c2 < w_dn[a] {
                        w_dn[a] = c2;
                        mid_dn[a] = x;
                        e_dn[a] = NONE;
                    }
                }
            }
        }
```

Kết quả của `customize` là một `Ch` thường, dùng được với mọi truy vấn và ma trận của bài 25, 26. Phần đắt được làm một lần: dựng thứ tự và cấu trúc mất 0,08 s (thứ tự 0,06 s, cấu trúc 0,02 s), ra 1.069.409 cung đi lên (8,44 cung mỗi nút), bậc đi lên lớn nhất 299, bộ nhớ cấu trúc 7,1 MB. Mỗi lần đổi trọng số phải duyệt 39.194.426 tam giác.

Engine dùng nó thế nào: `enable_time_of_day` dựng CCH một lần; mỗi lần có yêu cầu cho một giờ chưa có trong bộ nhớ đệm nhỏ (giữ tối đa 4 giờ gần nhất), engine tính trọng số của giờ đó và tùy biến CCH.

`examples/toi-noi-location/rust/crates/route-engine/src/engine.rs`

```rust
        if let (Some(tod), Some(slot)) = (&self.tod, slot)
            && tod.profile == p
        {
            let slot = slot % SLOTS;
            let mut recent = tod.recent.lock().expect("khoá không bị hỏng");
            if let Some((_, r)) = recent.iter().find(|(s, _)| *s == slot) {
                return Arc::clone(r);
            }
            let costs = tod.model.costs(&self.g, p, slot);
            let ch = tod.cch.customize(&costs.w);
```

Kết quả đúng: sau mỗi lần tùy biến, CH cho đúng chi phí của Dijkstra trên trọng số giờ đó, **0 khác biệt trên 300 truy vấn** ở mỗi giờ 3, 8, 13 và 18. Đường giải nén dùng lại code của bài 25.

## Cái giá: truy vấn chậm hơn 31–35 lần

Đây là phần cần đọc thẳng, không làm đẹp. Trong cài đặt này CCH **chậm hơn CH thường 31,1–34,8 lần** ở truy vấn khoảng cách: 234,2 µs so với 7,5 µs ở lần chạy chính (255,9 so với 7,3 µs ở lần hai). Số nút chốt mỗi truy vấn là 789 so với 173, và số cung đi lên sau tùy biến là 1,78 triệu (14,06 cung mỗi nút) so với 0,44 triệu của CH thường (3,49 cung mỗi nút).

Ba nguyên nhân chồng nhau. Thứ nhất, và không tránh được ở mọi CCH: thứ tự và cấu trúc không biết trọng số, không có nhân chứng để bỏ shortcut, nên phải giữ mọi shortcut mà thứ tự sinh ra: 899.270 shortcut ở giờ 8, so với 191.118 của CH thường. Thứ hai, thứ tự chia theo tọa độ không bằng các cách chia đồ thị tốt (lab chưa dùng InertialFlow hay METIS), nên vách ngăn lớn và đồ thị đi lên dày hơn mức cần. Thứ ba, chưa có bước «tùy biến hoàn hảo» để bỏ các cung thừa sau khi đã biết trọng số. Lab không tách được phần nào đóng góp bao nhiêu. Từ hai con số ở trên: chốt nhiều nút hơn 4,6 lần và mỗi nút chốt tốn khoảng 6,8 lần thời gian (234,2/789 so với 7,5/173, tính từ chính các số đo), một phần vì đồ thị đi lên dày hơn (14,06 so với 3,49 cung mỗi nút, gấp khoảng 4 lần).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 2. Đánh đổi của Customizable CH so với CH thường: đổi trọng số một giờ mất 355 ms thay vì 1.614 ms dựng lại, nhưng một truy vấn khoảng cách mất 234,2 µs thay vì 7,5 µs."><rect x="20" y="20" width="840" height="255" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Customizable CH đổi lấy gì và trả giá gì (cùng một lần chạy, HCM, ô tô)</text><text x="40" y="80" class="t-b">Đổi trọng số cho một giờ (trung bình 24 giờ)</text><text x="40" y="106" class="t-mono">CH dựng lại</text><rect x="190" y="92" width="500" height="20" rx="3" class="f-warn s-warn" stroke-width="1.5"/><text x="700" y="106" class="t-mono">1.614 ms</text><text x="40" y="136" class="t-mono">CCH tùy biến</text><rect x="190" y="122" width="110" height="20" rx="3" class="f-phys s-phys" stroke-width="1.5"/><text x="310" y="136" class="t-mono">355 ms (nhanh hơn 4,5 lần)</text><text x="40" y="180" class="t-b">Một truy vấn khoảng cách (trung bình)</text><text x="40" y="206" class="t-mono">CH thường</text><rect x="190" y="192" width="16" height="20" rx="3" class="f-phys s-phys" stroke-width="1.5"/><text x="216" y="206" class="t-mono">7,5 µs (173 nút chốt)</text><text x="40" y="236" class="t-mono">CCH</text><rect x="190" y="222" width="500" height="20" rx="3" class="f-warn s-warn" stroke-width="1.5"/><text x="700" y="236" class="t-mono">234,2 µs (789 nút)</text><text x="40" y="262" class="t-small">Hai nhóm cột có thang riêng. Máy chịu tải nặng: chỉ tỉ lệ có nghĩa. CCH chậm hơn 31,14 lần ở lần này (31,1–34,8 lần qua hai lần chạy).</text></svg>
</div>
<figcaption>Hình 2. Customizable CH đổi trọng số nhanh hơn 4,5 lần so với dựng lại CH ở lần chạy chính (3,7–4,5 lần qua hai lần), nhưng mỗi truy vấn chậm hơn 31 lần. Đo khi máy chịu tải nặng.</figcaption>
</figure>

Hình 2 gói lại sự đánh đổi. Nên chọn thế nào cho Tới Nơi? Theo số đo ở đây:

| Tình huống | Chọn | Lý do |
|---|---|---|
| Trọng số đổi theo 24 khung giờ cố định, bản đồ một thành phố | Dựng sẵn một CH cho mỗi giờ (hoặc dựng khi giờ đổi) | Dựng 1,1–1,6 s mỗi lần, truy vấn nhanh hơn 31 lần |
| Trọng số đổi liên tục (kẹt xe trực tiếp), ít truy vấn giữa hai lần đổi | CCH, kèm đầu tư vào thứ tự nút tốt hơn | Tùy biến 355 ms; hòa vốn với dựng lại ở khoảng 5.500 truy vấn giữa hai lần đổi (bài tập 2); truy vấn sẽ nhanh lên khi thứ tự tốt hơn, chưa đo |
| Trọng số đổi liên tục, nhiều truy vấn giữa hai lần đổi | Dựng lại CH trong nền | Bản đồ cỡ này dựng lại cỡ 1–2 s; truy vấn nhanh hơn 31 lần |
| Mô phỏng và lô nhỏ, ít truy vấn | Tìm kiếm theo cạnh trọng số của giờ (không cần CH) | Simulator của bài 30 tính trước 24 vector trọng số và dùng A* theo cạnh |

Dòng cuối là đúng cách simulator làm khi bật `traffic = "synthetic"`: `Router::new` tính 24 vector trọng số theo giờ, chọn vector theo giờ mô phỏng. Baseline của bài 30 dùng `free_flow`, nên lúc đó không dùng hồ sơ giờ.

## ETA tĩnh sai bao nhiêu

Câu hỏi thực tế: nếu engine luôn dùng tốc độ tự do thì sai ở đâu? Lab lấy 500 cặp nút ngẫu nhiên (seed 20261106), tính tuyến tối ưu theo tốc độ của giờ 3 sáng («tĩnh»), rồi chấm lại tuyến đó bằng trọng số của giờ đang xét. Hai thước đo: ETA tĩnh thấp hơn thời gian thật bao nhiêu phần trăm, và tuyến tĩnh chậm hơn tuyến tối ưu của giờ đó bao nhiêu.

| Giờ | ETA tĩnh thấp hơn (TB) | p95 | Lớn nhất | Tuyến tĩnh chậm hơn tối ưu (TB) | p95 | Tuyến khác |
|---:|---:|---:|---:|---:|---:|---:|
| 8 | 51,9% | 62,8% | 64,7% | 3,38% | 15,97% | 76,2% |
| 13 | 20,1% | 24,7% | 25,8% | 0,13% | 0,78% | 24,4% |
| 18 | 51,9% | 62,8% | 64,7% | 3,38% | 15,97% | 76,2% |
| 22 | 7,5% | 9,3% | 9,7% | 0,01% | 0,07% | 10,0% |

Đọc bảng: ở giờ cao điểm trong hồ sơ tổng hợp, ETA tĩnh thiếu trung bình 51,9% so với thời gian thật, nhưng tuyến tĩnh chỉ chậm hơn tuyến tối ưu 3,38% (p95 15,97%), dù 76,2% số tuyến là tuyến khác. Nói gọn: **dùng tốc độ tĩnh chủ yếu làm ETA sai, ít làm tuyến tệ đi**. Tách theo khoảng cách chim bay ở giờ 8: dưới 2 km (30 cặp) 44,9% và 1,96%; 2–5 km (82 cặp) 50,3% và 2,27%; 5–10 km (202 cặp) 52,8% và 2,96%; trên 10 km (186 cặp) 52,8% và 4,56%.

Hệ quả cho các bài sau: ETA ghi vào hàm mục tiêu và vào lời hứa với khách phải dùng trọng số theo giờ; còn muốn chỉ so thứ hạng tuyến thì tốc độ tĩnh chưa tệ. Nhưng toàn bộ các tỉ lệ trên phụ thuộc vào độ nặng do lab đặt (0,20–0,65). Với hệ số khác, các con số sẽ khác; chúng là **cỡ của hiệu ứng dưới giả định**, không phải đo thực tế.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` (hoặc `--fixture`). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc27_toc_do_theo_gio
```

Cấu hình: 500 cặp nút (300 cặp đầu để kiểm đúng), seed 20261106, hai lần chạy. Output (rút gọn, lần có thời gian tùy biến thấp nhất), **đo khi máy chịu tải nặng**:

```text
DỮ LIỆU KẸT XE Ở ĐÂY LÀ TỔNG HỢP: hệ số do lab tự đặt (xem crates/route-engine/src/traffic.rs), không đo từ xe thật.
  dựng CCH (thứ tự bằng chia đệ quy theo toạ độ + cấu trúc không nhân chứng): 0.08 s (thứ tự 0.06 s, cấu trúc 0.02 s)
  tuỳ biến lại trọng số cho 24 giờ: mỗi lần trung bình 355 ms (nhỏ nhất 330, lớn nhất 447); tổng 8.53 s
  dựng lại CH thường từ đầu cho cùng 24 giờ: mỗi lần trung bình 1614 ms (nhỏ nhất 1156, lớn nhất 5367); tổng 38.74 s
  dựng lại / tuỳ biến: trung bình 4.54x, nhỏ nhất 2.67x (cùng máy, cùng lần chạy, luân phiên theo giờ)
  giờ  8: CCH tuỳ biến so với Dijkstra: 0 khác biệt trên 300 truy vấn
  truy vấn khoảng cách, trung bình: CH thường 7.5 µs (173 nút chốt), CCH 234.2 µs (789 nút chốt); CCH chậm hơn 31.14x
```

Các số đếm và tỉ lệ ETA tái lập từng chữ số; số thời gian thì không (lần hai: tùy biến 358 ms, dựng lại 1.341 ms, CCH chậm hơn 34,84 lần). `verify-p03.sh` chạy phần kiểm đúng trên đồ thị tổng hợp.

## Giới hạn của ví dụ

- **Kẹt xe là tổng hợp.** Hệ số do lab đặt, không có dữ liệu xe thật, không có sự cố, mưa, sự kiện. Mọi con số «ETA sai x%» là cỡ hiệu ứng dưới giả định.
- **Khung giờ, chưa phụ thuộc thời gian.** Cả chuyến dùng trọng số của giờ khởi hành, nên chuyến dài vắt qua hai khung giờ bị tính sai phần sau; chưa có thuật toán phụ thuộc thời gian thật sự.
- **CCH ở mức học.** Thứ tự theo tọa độ, chưa có tùy biến hoàn hảo; số đo chậm hơn 31–35 lần là của cài đặt này chứ không phải của phương pháp CCH nói chung. Bài này không so với một cài đặt CCH tốt hơn.
- **Tùy biến toàn bộ.** Mỗi lần tùy biến tính lại mọi cung; chưa có cập nhật cục bộ cho một nhóm cạnh bị kẹt.
- **Chỉ một thành phố, ô tô.** Xe máy dùng cùng cơ chế với độ nặng 0,7 lần, chưa đo riêng.
- **Dịch vụ `geo-engine` chỉ bật hồ sơ giờ khi chạy với `--time-of-day`**; lab đo tải của bài 29 không bật. Simulator baseline ở bài 30 dùng giao thông tự do.
- **Đo khi máy chịu tải nặng**, hai lần chạy; chưa có lần đo trên máy rảnh.

## Bài tập

**Bài 1.** Tùy biến nhanh hơn dựng lại 4,54 lần ở lần chạy chính, nhỏ nhất 2,67 lần. Vì sao «nhỏ nhất» quan trọng hơn «trung bình» khi lên kế hoạch dung lượng?

<details><summary>Lời giải bài 1</summary>
<p>Kế hoạch dung lượng phải chịu được trường hợp bất lợi cho lựa chọn của mình. Trung bình 4,54 lần bị kéo lên bởi một lần dựng lại 5.367 ms, hơn ba lần trung bình; ở lần chạy thứ hai không có giá trị như vậy và trung bình chỉ còn 3,74 lần. Con số 2,67 gần hơn với điều nên trông đợi. Hai lần chạy dưới tải nặng không đủ để kết luận CCH «ổn định hơn»: lần hai, dựng lại cũng chỉ dao động 1.197–1.560 ms. Dù sao lợi thế vẫn cỡ vài lần, chưa phải vài bậc độ lớn.</p>
</details>

**Bài 2.** Nếu trọng số chỉ đổi tám lần mỗi ngày (theo bảng cố định) và truy vấn là phần lớn tải, bạn chọn CCH hay CH theo giờ? Tính từ số đo.

<details><summary>Lời giải bài 2</summary>
<p>CH theo giờ: mỗi lần đổi dựng lại 1,1–1,6 s, truy vấn 7,5 µs. CCH: mỗi lần đổi 355 ms, truy vấn 234,2 µs (chậm hơn khoảng 31 lần). Mỗi truy vấn CH nhanh hơn CCH khoảng 227 µs (234,2 trừ 7,5), còn mỗi lần đổi trọng số CCH rẻ hơn dựng lại khoảng 1.259 ms (1.614 trừ 355). Hòa vốn khi có khoảng 5.500 truy vấn giữa hai lần đổi (1.259 chia 0,227); nhiều hơn thế thì CH theo giờ thắng. Tám lần đổi mỗi ngày với tải truy vấn lớn chắc chắn vượt ngưỡng đó, nên CH theo giờ thắng; cái giá là bộ nhớ nhân lên theo số khung giờ.</p>
</details>

**Bài 3.** Vì sao hai chuyến cùng 20.575 m chim bay lúc 3 giờ và 8 giờ có thể đi theo hai tuyến khác nhau?

<details><summary>Lời giải bài 3</summary>
<p>Hệ số kẹt phụ thuộc loại đường và vùng: đường lớn ở trung tâm chịu hệ số nặng nhất (độ nặng 0,65), nên lúc cao điểm một đường nhỏ vòng ngoài có thể nhanh hơn. Lab đo 76,2% tuyến tối ưu của giờ 8 khác tuyến tối ưu của giờ 3, dù tuyến tĩnh chỉ chậm hơn trung bình 3,38%.</p>
</details>

Bài trước: [bài 26](/articles/location-ma-tran-many-to-many-bucket-ch). Bài tiếp: [bài 28](/articles/location-self-host-osrm-valhalla-hay-map-provider).

## Tài liệu gốc

- J. Dibbelt, B. Strasser, D. Wagner, «Customizable Contraction Hierarchies»: [arXiv:1402.0402](https://arxiv.org/abs/1402.0402). Tách thứ tự theo chia lồng nhau khỏi trọng số.
- R. Geisberger, P. Sanders, D. Schultes, D. Delling, «Contraction Hierarchies: Faster and Simpler Hierarchical Routing in Road Networks», WEA 2008: [trang bài báo ở KIT](https://ae.iti.kit.edu/999.php).
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho dữ liệu OSM.
