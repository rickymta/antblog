---
slug: location-geocoding-dia-chi-viet-nam-autocomplete
title: "Từ địa chỉ tới toạ độ: geocoding và gợi ý địa chỉ Việt Nam"
summary: "Bài 31: chuẩn hoá chữ có dấu, số nhà và hẻm, chỉ mục địa chỉ dựng từ OSM còn thưa, gợi ý khi gõ, tìm ngược, độ tin cậy; tập thử sinh từ chính OSM nên chỉ đo độ bền trước nhiễu chữ; việc kiểm tra đổi tên đơn vị hành chính 2025 chưa đầy đủ."
category: location-routing
level: advanced
tags: [toi-noi, location, rust, geocoding, openstreetmap, unicode, autocomplete, reverse-geocoding]
language: vi
---

Khách gõ vào ô điểm đến: `Hem 45 Nguyen Trai, P12 Q10`. Không dấu, «hẻm» viết thành «hem», phường và quận dính liền với số. Người đọc tiếng Việt hiểu ngay, còn dịch vụ định vị phải đưa ra một toạ độ, vì điểm trả `dest` là số hạng `T_trip` trong hàm mục tiêu của phần P04: nếu toạ độ lệch 500 m thì đoạn có khách của mọi ứng viên điểm đón đều bị đo sai, và thứ hạng của chúng có thể đổi. Bài này đứng trước cả phần tối ưu: nó trả lời «địa chỉ gõ vào là chỗ nào» và, quan trọng hơn, «ta tin câu trả lời đó tới đâu».

Đây là bài 31 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dùng R-tree của [bài 07](/articles/location-rstar-r-tree-snap-diem-ve-duong) cho tìm ngược và dữ liệu OpenStreetMap đã đọc ở [bài 06](/articles/location-osm-pbf-do-thi-duong-bo-xe-may-o-to). Mọi con số bên dưới đo trên dữ liệu OSM của TP.HCM trong bbox của lab (lat 10,70–10,86, lng 106,60–106,78; © OpenStreetMap contributors), bản PBF có dấu thời gian `2026-11-04T21:21:03Z`, seed 20261106. **Tập thử của bài được sinh từ chính OSM**, nên nó không nói gì về độ chính xác ngoài đời; mục thứ tư giải thích thiên lệch đó trước khi đưa bất kỳ phần trăm nào.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Chữ «Nguyễn Trãi» trong một cơ sở dữ liệu có thể đến dưới hai dạng Unicode trông giống hệt nhau. Theo bạn chúng khác nhau bao nhiêu byte, và nếu so sánh chuỗi thô thì điều gì xảy ra với một người dùng gõ trên máy này và một địa chỉ lưu từ máy kia?</p>
<details><summary>Xem đáp án</summary><p>Dạng dựng sẵn (NFC) chiếm 14 byte, dạng tách rời (NFD, chữ cái cộng dấu kết hợp) chiếm 17 byte. Hai chuỗi hiển thị giống nhau nhưng so sánh byte thì khác, nên tìm kiếm thô hoàn toàn không khớp được. Lab đưa cả hai về cùng một khoá chuẩn hoá và kiểm tra bằng một dòng <code>kiểm tra: NFC và NFD cho cùng khoá chuẩn hoá = true</code>.</p></details>
</div>

## Địa chỉ Việt Nam khác địa chỉ trong sách giáo khoa

Một địa chỉ ở TP.HCM thường không có dạng «số nhà, tên đường». Nó có thể là `123/45/6 Nguyễn Trãi`: nhà số 6, trong hẻm 45, trong hẻm 123 mở ra từ đường chính (cách đọc của lab, ghi ở đầu `parse.rs`). Nó có thể là `3 Tháng 2`, nơi «3» là một phần của tên đường chứ không phải số nhà. Nó có thể thiếu dấu, viết tắt (`P.`, `Q.`, `TP.`), hoặc dùng chữ cái đơn đứng riêng như `12 A Lê Lợi`.

Bước đầu tiên của lab là chuẩn hoá. Quy trình: đưa về NFC và chữ thường, tách token (giữ `/` và `-` kẹp giữa hai ký tự chữ số), bỏ dấu, mở rộng viết tắt. Phần cốt lõi của việc bỏ dấu chỉ có bốn dòng, vì `đ` không phân rã thành chữ cái cộng dấu kết hợp và phải xử lý riêng:

`examples/toi-noi-location/rust/crates/geocode/src/norm.rs`

```rust
/// Chữ thường và bỏ dấu: NFD, xoá dấu kết hợp (U+0300 trở đi), `đ` thành `d`. Giữ nguyên các ký tự khác,
/// kể cả dấu câu. Nhận cả chuỗi NFC lẫn NFD.
pub fn strip_marks(s: &str) -> String {
    s.to_lowercase()
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .map(|c| if c == 'đ' { 'd' } else { c })
        .collect()
}
```

Phép phân tích (`parse`) đọc các đơn vị hành chính trước, rồi số nhà và hẻm, phần còn lại là tên đường. Lab in một bảng 11 câu mẫu; trích vài dòng (kết quả thật của lần chạy ghi ở mục «Chạy ví dụ»):

| Đầu vào | Khoá chuẩn hoá | Cấu trúc đọc được |
|---|---|---|
| `123/45/6 Nguyễn Trãi, P. Bến Thành, Q.1, TP.HCM` | `123/45/6 nguyen trai phuong ben thanh quan 1 thanh pho ho chi minh` | hẻm 123, 45; nhà 6; đường `nguyen trai`; phường `ben thanh`; quận 1 |
| `Số 12, hẻm 45 Nguyễn Trãi` | `12 hem 45 nguyen trai` | hẻm 45; nhà 12 |
| `H45 Nguyễn Trãi, P12 Q10` | `hem 45 nguyen trai phuong 12 quan 10` | hẻm 45; phường 12; quận 10 |
| `3 Tháng 2` | `3 thang 2` | đường `3 thang 2`, không có số nhà |
| `Huyen Tran Cong Chua` | `huyen tran cong chua` | **đọc sai** thành huyện `tran cong chua` |

Dòng cuối là một giới hạn đã biết, không phải lỗi bị giấu đi: khi bỏ dấu, «Huyền» (tên người) trùng chữ với «huyện» (đơn vị hành chính), và nếu không có chữ «đường» đứng trước thì parser chọn huyện. Có dấu (`Huyền Trân Công Chúa, Quận 1`) thì đọc đúng. Mức mơ hồ do bỏ dấu đo được trên tên đường trong chỉ mục: 247 trong 24.993 khoá tên đường không dấu (0,99%) ứng với từ hai dạng có dấu khác nhau trở lên, nhưng nguyên nhân chủ yếu hoá ra là **người vẽ bản đồ viết cùng một tên khi có dấu, khi không** (`chu van an` và `chu văn an`), chứ không phải cặp từ đồng âm thật như «Bà» và «Ba».

## OSM ở TP.HCM có bao nhiêu địa chỉ để tra cứu

Chỉ mục dựng từ ba loại bản ghi: địa chỉ có số nhà (25.725), địa điểm có tên (8.612) và địa danh (2.168), tổng cộng 36.505 bản ghi, cộng 41.987 way đường có tên để tra tên đường. Con số quyết định chất lượng của mọi thứ phía sau là **độ phủ `addr:*`**, đếm trực tiếp từ PBF:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 360" role="img" aria-label="Hình 1. Độ phủ addr:* của OSM trong bbox của lab: trong 79.839 toà nhà chỉ 7.567 có số nhà (9,48%), và thẻ phường, quận, tỉnh thưa hơn nhiều so với số nhà."><rect x="20" y="20" width="840" height="320" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">Độ phủ addr:* trong bbox của lab</text><text x="40.0" y="80.0" class="t-b">79.839 toà nhà (building=*) trong bbox</text><rect x="40.0" y="94.0" width="36.0" height="26.0" rx="0" class="f-pk" stroke-width="0"/><rect x="76.0" y="94.0" width="4.6" height="26.0" rx="0" class="f-warn" stroke-width="0"/><rect x="80.6" y="94.0" width="339.4" height="26.0" rx="0" class="f-dead" stroke-width="0"/><rect x="40.0" y="94.0" width="380.0" height="26.0" rx="0" class="f-none s-line" stroke-width="1"/><rect x="40.0" y="140.0" width="12.0" height="12.0" rx="2" class="f-pk" stroke-width="0"/><text x="60.0" y="150.0" class="t-small">có addr:housenumber: 7.567 (9,48%)</text><rect x="40.0" y="166.0" width="12.0" height="12.0" rx="2" class="f-warn" stroke-width="0"/><text x="60.0" y="176.0" class="t-small">có addr:* khác, không có số nhà: 963 (tính từ 8.530 − 7.567)</text><rect x="40.0" y="192.0" width="12.0" height="12.0" rx="2" class="f-dead" stroke-width="0"/><text x="60.0" y="202.0" class="t-small">không có thẻ addr:* nào: 71.309 (tính từ 79.839 − 8.530)</text><text x="40.0" y="245.0" class="t-small">Cả bbox: 25.725 đối tượng có số nhà (nút 17.206, way 8.519),</text><text x="40.0" y="261.0" class="t-small">21.610 có cả số nhà lẫn tên đường, 4.115 có số nhà nhưng không có đường.</text><text x="40.0" y="285.0" class="t-small">Đếm trực tiếp từ PBF 2026-11-04, © OpenStreetMap contributors.</text><text x="460.0" y="80.0" class="t-b">Số đối tượng theo khoá addr:*</text><text x="460.0" y="122.0" class="t-mono">addr:housenumber</text><rect x="590.0" y="110.0" width="190.0" height="16.0" rx="2" class="f-pk" stroke-width="0"/><text x="786.0" y="122.0">25.725</text><text x="460.0" y="152.0" class="t-mono">addr:street</text><rect x="590.0" y="140.0" width="188.1" height="16.0" rx="2" class="f-pk" stroke-width="0"/><text x="784.1" y="152.0">25.468</text><text x="460.0" y="182.0" class="t-mono">addr:subdistrict</text><rect x="590.0" y="170.0" width="55.5" height="16.0" rx="2" class="f-warn" stroke-width="0"/><text x="651.5" y="182.0">7.521</text><text x="460.0" y="212.0" class="t-mono">addr:district</text><rect x="590.0" y="200.0" width="52.1" height="16.0" rx="2" class="f-warn" stroke-width="0"/><text x="648.1" y="212.0">7.051</text><text x="460.0" y="242.0" class="t-mono">addr:city</text><rect x="590.0" y="230.0" width="45.7" height="16.0" rx="2" class="f-warn" stroke-width="0"/><text x="641.7" y="242.0">6.188</text><text x="460.0" y="272.0" class="t-mono">addr:province</text><rect x="590.0" y="260.0" width="29.0" height="16.0" rx="2" class="f-warn" stroke-width="0"/><text x="625.0" y="272.0">3.922</text><text x="460.0" y="302.0" class="t-mono">addr:ward</text><rect x="590.0" y="290.0" width="1.5" height="16.0" rx="2" class="f-warn" stroke-width="0"/><text x="597.5" y="302.0">126</text></svg>
</div>
<figcaption>Hình 1. Độ phủ addr:* của OSM trong bbox của lab, đếm trực tiếp từ bản PBF 2026-11-04. Trong 79.839 toà nhà chỉ 7.567 có addr:housenumber (9,48%); thẻ phường, quận, tỉnh thưa hơn nhiều so với số nhà. Hai số 963 và 71.309 ở chú giải là phép trừ từ các số lab in ra. © OpenStreetMap contributors.</figcaption>
</figure>

Hơn 9 trên 10 toà nhà trong OSM không có số nhà. Điều đó có nghĩa là địa chỉ thật mà khách gõ thường **không có trong chỉ mục**, và hệ thống phải lùi về «số nhà gần nhất cùng đường» (`HouseNearest`) hoặc mức đường. Thẻ hành chính còn thưa hơn: 7.521 điểm có `addr:subdistrict` và 126 có `addr:ward` so với 25.725 số nhà, nên không thể suy ra phường từ toạ độ; lab không có ranh giới hành chính, phường chỉ là thẻ gắn trên điểm. Khoá `addr:interpolation` chỉ có 16 đối tượng, quá ít để nội suy số nhà theo hình học. Dữ liệu cũng nhiễu: `addr:city` có `Ho Chi Minh City` ×1.233, `HCMC` ×179 và cả `Phường An Lạc`; `addr:district` có `Phường An Lạc` ×349 và quận không có chữ «Quận» (`Gò Vấp`).

## Chỉ mục, mức khớp và độ tin cậy

Mỗi truy vấn đi qua khoá chuẩn hoá rồi được xếp vào một **mức khớp**, từ chắc chắn nhất tới yếu nhất: `HouseExact` (đúng số nhà trên đúng đường), `AlleyRoot` (chỉ khớp lối vào hẻm, số đầu của chuỗi `123/45/6`), `HouseNearest` (số nhà không có trong dữ liệu, lấy số gần nhất cùng đường), `Poi` (tên địa điểm), `Street`, `Place`, và `Ward` (chỉ có phường). Mức khớp quyết định điểm cơ sở (0,90; 0,60; 0,50; 0,85; 0,40; 0,45; 0,20), là tham số của lab chứ không đo từ dữ liệu. Sửa lỗi gõ dùng khoảng cách Damerau–Levenshtein với ngân sách theo độ dài khoá: tên ngắn phải khớp đúng, vì cho phép sửa trên tên ngắn sẽ khớp bừa:

`examples/toi-noi-location/rust/crates/geocode/src/index.rs`

```rust
/// Ngân sách sửa chữ theo độ dài khoá: ≤ 3 ký tự phải khớp đúng, 4–7 cho 1 lần sửa, từ 8 cho 2 lần.
pub fn fuzzy_budget(len: usize) -> usize {
    match len {
        0..=3 => 0,
        4..=7 => 1,
        _ => 2,
    }
}
```

Từ điểm cơ sở, lab nhân thêm các hệ số (độ giống chữ, khớp phường, dấu, số nhà gần nhất) rồi giới hạn bằng **trần hành chính**. Công thức nằm ở đầu `index.rs`:

`examples/toi-noi-location/rust/crates/geocode/src/index.rs`

```text
//! ```text
//! score      = base(mức khớp) × text_sim × admin_factor × diacritic_factor × nearest_factor
//! confidence = min( score × ambiguity × dup , trần hành chính )
//! ```
```

Hệ số `dup = 1/√G` đáng có một đoạn riêng, vì nó xuất phát từ một điều bất ngờ. Tên đường lặp rất nhiều ở thành phố này (Đường số 1, 6, 8, 27, …, mỗi tên có hàng chục cụm cách xa nhau). Một khớp số nhà đúng ở cụm này không chứng minh người dùng nghĩ tới cụm đó. Truy vấn `16 Đường số 6` trả `14;16 Đường số 6` cách đáp án 19.907,5 m với điểm 0,873, tức trông rất tự tin nếu chỉ nhìn điểm; nhân thêm hệ số này thì độ tin cậy còn 0,098. Hệ số được thêm sau lần chạy đầu trên tập thử, chưa kiểm trên tập khác.

**Gợi ý khi gõ** là bài toán khác: mỗi ký tự gõ vào là một truy vấn tiền tố trên khoá chuẩn hoá đã sắp xếp. Cài đặt là tìm nhị phân vị trí bắt đầu rồi duyệt các khoá cùng tiền tố, giữ top `limit` bằng một heap nhỏ:

`examples/toi-noi-location/rust/crates/geocode/src/index.rs`

```rust
        let start = self.ac.partition_point(|e| e.key.as_str() < rest.as_str());
        let mut heap: BinaryHeap<(Reverse<u32>, usize)> = BinaryHeap::with_capacity(limit + 1);
        for (off, e) in self.ac[start..].iter().enumerate() {
            if !e.key.starts_with(rest.as_str()) {
                break;
            }
```

Số đo trên 1.000 khoá đích (19.812 lần gọi, mỗi ký tự một lần): p50 mỗi lần gọi 4,1 µs, p99 300,8 µs, đo khi máy đang chịu tải nặng nên chỉ so sánh tương đối. Điều đáng chú ý là chất lượng: ở tiền tố 5 ký tự khoá đích chỉ lọt vào top 5 ở 22,7% trường hợp, còn ở 11–15 ký tự là 60,9%; trung bình phải gõ 53,6% độ dài khoá để đích vào top 5 lần đầu, và 103 trên 1.000 đích không bao giờ vào top 5, phần lớn là nhãn bắt đầu bằng `Hẻm N`, vì số đầu bị đọc là hẻm. Gợi ý chỉ khớp tiền tố của cả tên (không khớp giữa từ, không sửa lỗi gõ).

## Tập thử sinh từ chính OSM: thiên lệch cần nói thẳng

Muốn biết geocoder đúng bao nhiêu phần trăm thì phải có câu hỏi và đáp án. Lab chọn cách rẻ nhất: lấy 21.610 điểm có cả số nhà lẫn tên đường, tách ngẫu nhiên 20% (4.322 điểm) làm tập thử, dựng câu hỏi từ thẻ OSM rồi thêm nhiễu (bỏ dấu, 1 hoặc 2 lỗi gõ, viết tắt, thiếu phường), đáp án là toạ độ OSM. Mỗi nhóm dùng tối đa 2.000 điểm.

**Thiên lệch ở đây mang tính cấu trúc, không phải chi tiết.** Câu hỏi dựng từ OSM, đáp án lấy từ OSM, chỉ mục cũng dựng từ OSM. Vì vậy tập thử đo **độ bền trước nhiễu chữ viết** và cơ chế của hệ thống; nó **không** đo: người dùng gõ địa danh hay tên thân mật, số hẻm không có trong OSM, tên phường cũ, hoặc chuyện OSM thưa đúng ở chỗ họ cần. Hai chế độ đo cố gắng bù một phần. Chế độ GIỮ để đối tượng thử nằm trong chỉ mục: đo thuần nhiễu chữ. Chế độ GỠ gỡ đối tượng thử khỏi chỉ mục trước khi tra, mô phỏng «địa chỉ không có trong OSM»: không bao giờ khớp lại đúng điểm đã gỡ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 420" role="img" aria-label="Hình 2. top1 trong 50 m của geocoder theo kiểu nhiễu chữ: chế độ GIỮ (đối tượng thử vẫn trong chỉ mục) và chế độ GỠ (đã gỡ khỏi chỉ mục). Lỗi gõ làm tụt nhiều hơn bỏ dấu; địa chỉ không có trong OSM chỉ trúng khoảng một nửa."><rect x="20" y="20" width="840" height="380" rx="10" class="f-bg s-line" stroke-width="1.5"/><text x="40" y="46" class="t-b">top1@50 theo kiểu nhiễu</text><text x="40.0" y="76.0" class="t-small">Tập thử sinh từ chính OSM, nên chỉ đo độ bền trước nhiễu chữ, không đo độ chính xác ngoài đời</text><rect x="40.0" y="90.0" width="12.0" height="12.0" rx="2" class="f-pk" stroke-width="0"/><text x="58.0" y="100.0" class="t-small">GIỮ: đối tượng thử còn trong chỉ mục</text><rect x="330.0" y="90.0" width="12.0" height="12.0" rx="2" class="f-warn" stroke-width="0"/><text x="348.0" y="100.0" class="t-small">GỠ: đối tượng thử bị gỡ (địa chỉ không có trong OSM)</text><text x="40.0" y="144.0" class="t-b">sạch</text><rect x="170.0" y="122.0" width="529.2" height="16.0" rx="2" class="f-pk" stroke-width="0"/><text x="705.2" y="134.0">88,2%</text><rect x="170.0" y="142.0" width="303.6" height="16.0" rx="2" class="f-warn" stroke-width="0"/><text x="479.6" y="154.0">50,6%</text><text x="40.0" y="190.0" class="t-b">bỏ dấu</text><rect x="170.0" y="168.0" width="521.4" height="16.0" rx="2" class="f-pk" stroke-width="0"/><text x="697.4" y="180.0">86,9%</text><rect x="170.0" y="188.0" width="298.2" height="16.0" rx="2" class="f-warn" stroke-width="0"/><text x="474.2" y="200.0">49,7%</text><text x="40.0" y="236.0" class="t-b">viết tắt</text><rect x="170.0" y="214.0" width="538.2" height="16.0" rx="2" class="f-pk" stroke-width="0"/><text x="714.2" y="226.0">89,7%</text><rect x="170.0" y="234.0" width="310.8" height="16.0" rx="2" class="f-warn" stroke-width="0"/><text x="486.8" y="246.0">51,8%</text><text x="40.0" y="282.0" class="t-b">1 lỗi gõ</text><rect x="170.0" y="260.0" width="445.8" height="16.0" rx="2" class="f-pk" stroke-width="0"/><text x="621.8" y="272.0">74,3%</text><rect x="170.0" y="280.0" width="250.8" height="16.0" rx="2" class="f-warn" stroke-width="0"/><text x="426.8" y="292.0">41,8%</text><text x="40.0" y="328.0" class="t-b">2 lỗi gõ</text><rect x="170.0" y="306.0" width="405.0" height="16.0" rx="2" class="f-pk" stroke-width="0"/><text x="581.0" y="318.0">67,5%</text><rect x="170.0" y="326.0" width="240.0" height="16.0" rx="2" class="f-warn" stroke-width="0"/><text x="416.0" y="338.0">40,0%</text><text x="40.0" y="374.0" class="t-b">gộp các nhiễu</text><rect x="170.0" y="352.0" width="441.0" height="16.0" rx="2" class="f-pk" stroke-width="0"/><text x="617.0" y="364.0">73,5%</text><rect x="170.0" y="372.0" width="247.2" height="16.0" rx="2" class="f-warn" stroke-width="0"/><text x="423.2" y="384.0">41,2%</text></svg>
</div>
<figcaption>Hình 2. top1 trong 50 m theo kiểu nhiễu, 2.000 câu hỏi mỗi nhóm trừ bỏ dấu (1.989) và viết tắt (1.501). Cột xanh là chế độ GIỮ, cột cam là chế độ GỠ. Seed 20261106, dữ liệu OSM TP.HCM, tập thử sinh từ chính OSM.</figcaption>
</figure>

Ba điều đọc được, mỗi điều kèm giới hạn:

- **Lỗi gõ làm tụt nhiều hơn bỏ dấu.** Ở chế độ GIỮ, câu sạch trúng 88,2% trong 50 m, bỏ dấu còn 86,9% (khoá tìm kiếm đã bỏ dấu nên gần như không đổi), một lỗi gõ 74,3%, hai lỗi 67,5%. Phần lớn lượng hụt là các câu «không có kết quả» (408 trên 2.000 ca ở một lỗi gõ), do tên đường ngắn chỉ cho 0–1 lần sửa.
- **Địa chỉ không có trong OSM chỉ trúng khoảng một nửa.** Ở chế độ GỠ câu sạch trúng 50,6% trong 50 m và 72,5% trong 200 m. Ứng viên đầu cách đáp án trung vị 44 m nhưng p90 tới 5.267 m: số nhà gần nhất cùng đường thường nằm sát, nhưng khi đường dài hoặc trùng tên ở nhiều nơi thì sai hàng kilômét.
- **Độ tin cậy có ý nghĩa ở nửa trên, không đơn điệu ở nửa dưới.** Chế độ GIỮ: nhóm tin cậy [0,9; 1,0] đúng 99,3% (trong 50 m), nhóm [0,5; 0,6) đúng 95,4%. Chế độ GỠ: nhóm [0,7; 0,8) chỉ đúng 55,8%, vì mô hình không thể biết địa chỉ thật đã thiếu trong dữ liệu.

Một điều cần nói thẳng nữa: sau lần chạy đầu (trúng 79,7% trong 50 m ở chế độ GIỮ, câu sạch), lab đã sửa parser (tách theo dấu phẩy, nhận biết dấu khi tách tên đường, số nhà gần nhất theo cụm, hệ số `dup`) **trên chính tập thử này**. Kết quả cuối (88,2%) vì thế lạc quan hơn so với một tập thử mới chưa từng nhìn. Hệ số `dup` chưa được kiểm trên tập khác.

## Tìm ngược: từ toạ độ về nhãn

Chiều ngược lại dùng R-tree: tìm điểm địa chỉ gần nhất trong bán kính, nếu không có thì đường có tên gần nhất. Lab hỏi bằng toạ độ OSM của 3.000 điểm địa chỉ đã gỡ khỏi chỉ mục, nên kết quả luôn là một địa chỉ **khác**, và «sai số» là khoảng cách từ điểm thật tới nhãn trả về. Với GPS hoàn hảo (σ = 0 m), 2.519 truy vấn ra mức địa chỉ (p50 7,7 m, p95 40,5 m), 475 ra mức đường (p50 12,0 m), 6 không có kết quả; đúng tên đường 85,0%. Thêm nhiễu GPS σ = 10 m: p50 14,8 m, p95 43,3 m, đúng tên đường 78,9%. Hiểu đúng các số này: chúng cho biết nhãn hàng xóm gần nhất nằm sát tới đâu và độ nhạy với nhiễu GPS, chứ không phải «độ chính xác của ứng dụng» ngoài đời.

## Đổi tên hành chính 2025: đã kiểm tra gì, chưa kiểm tra gì

Địa chỉ Việt Nam có thêm một lớp bất ổn: đơn vị hành chính đã đổi năm 2025. Lab ghi lại những gì kiểm được ngày 07/11/2026, và nói rõ kiểm bằng cách nào. Mọi nguồn dưới đây là trang tổng hợp luật và báo, đọc qua công cụ tìm kiếm và tải trang có tóm tắt, **chưa đối chiếu với văn bản gốc trên vbpl.vn hay Công báo**. Ba ý chính dưới đây khớp nhau giữa ít nhất hai nguồn độc lập; chi tiết ở `results/p04.md`.

- Nghị quyết **202/2025/QH15** của Quốc hội: số hiệu và ngày 12/6/2025 thấy trên luatvietnam.vn; trang đó ghi Điều 1 khoản 16 nhập TP.HCM, tỉnh Bà Rịa - Vũng Tàu và tỉnh Bình Dương thành Thành phố Hồ Chí Minh mới. Hai trang báo xác nhận cùng nội dung nhưng không ghi số hiệu.
- Nghị quyết **203/2025/QH15** (sửa đổi Hiến pháp), thông qua 16/6/2025, chấm dứt đơn vị hành chính cấp huyện trên cả nước từ 01/7/2025 (theo trang pbgdpl.cantho.gov.vn).
- Nghị quyết **1685/NQ-UBTVQH15** ngày 16/6/2025 của Uỷ ban Thường vụ Quốc hội về sắp xếp đơn vị hành chính cấp xã của TP.HCM: còn 168 đơn vị cấp xã (113 phường, 54 xã, 1 đặc khu Côn Đảo; luatvietnam.vn, Báo Đấu thầu và Báo Lao Động cho cùng con số), chính quyền mới hoạt động từ 01/7/2025.

Phần **chưa kiểm** quan trọng không kém: bảng ánh xạ phường cũ sang phường mới trong lab chỉ có 60 dòng, lấy từ khoản 1–20 và 78 của Điều 1 NQ 1685 (21 trên 168 khoản); các phường của Bình Dương và Bà Rịa - Vũng Tàu cũ không có dòng nào; hai luật về tổ chức chính quyền địa phương chỉ xuất hiện trong bản tóm tắt tìm kiếm, **chưa** thấy trong trang nào đã tải nên bài không trích; mục «Ngày áp dụng» của NQ 1685 nằm sau đăng nhập; và diện tích, dân số TP.HCM mới chỉ thấy ở một trang báo, bài không dùng. Cần kiểm lại các điểm này trước khi đưa vào sản phẩm thật.

OSM trong bbox gắn tên phường **lẫn cả hai thời kỳ**: `addr:subdistrict` có `Phường 12` ×125, `Phường 14` ×123 (kiểu cũ) cạnh `Phường Xóm Chiếu` ×183 (tên chỉ có sau sáp nhập). Các quan hệ ranh giới `boundary=administrative` trong file PBF toàn Việt Nam có `Phường Sài Gòn`, `Bàn Cờ`, `Bến Thành`, `Nhiêu Lộc`… và không có `Phường Bến Nghé`, `Phường Thảo Điền`, `Quận 1`, `Quận 3`, `Quận 4`, tức ranh giới trong PBF đã theo mô hình mới. Hệ quả: tên phường trong truy vấn và trong thẻ có thể thuộc hai thời kỳ, và hệ thống chỉ so chuỗi.

Cách lab xử lý: câu hỏi mang tên cũ phải được ánh xạ hoặc gắn cờ, **không bao giờ trả lời sai với độ tin cậy cao**. Mỗi trạng thái hành chính có một trần độ tin cậy, và phường cũ bị tách vào nhiều phường mới (như Đa Kao vào Sài Gòn và Tân Định) được gắn `OldSplit` thay vì chọn bừa:

`examples/toi-noi-location/rust/crates/geocode/src/admin.rs`

```rust
    /// Trần độ tin cậy cho kết quả geocode khi trạng thái này xảy ra: không bao giờ tin cậy cao khi tên hành chính đã cũ.
    pub fn confidence_cap(self) -> f64 {
        match self {
            Self::OldUnmapped | Self::OldSplit | Self::ProvinceMerged => 0.5,
            Self::OldMapped => 0.8,
            Self::NoAdmin | Self::NotChecked | Self::NewWard => 1.0,
        }
    }
}

fn lookup(district: Option<&str>, ward: &str) -> Option<&'static WardMapping> {
    WARD_MAPPINGS
        .iter()
        .find(|r| r.ward == ward && r.district.is_none_or(|d| Some(d) == district))
}
```

Kết quả in ra của lab cho thấy hiệu ứng: `3 Đại lộ 2` (không nêu hành chính) tin cậy 0,900; thêm `Phường Sài Gòn` còn 0,855; thêm `Phường Bến Nghé, Quận 1` (ánh xạ được sang Sài Gòn) tin cậy 0,800 kèm cảnh báo; thêm `Phường 12, Quận Tân Bình` (không có trong bảng) chỉ 0,500 kèm cảnh báo «câu hỏi dùng tên quận/phường số kiểu cũ». Tên phường trong PBF không phải thứ để tin vô điều kiện, vì cả dữ liệu lẫn luật đều đang đổi.

## Chạy ví dụ và kiểm chứng

Từ gốc repo (cần `data/vietnam-latest.osm.pbf`; thêm `-- --fixture` để chạy offline trên `fixtures/geocode-mau.json`, dữ liệu **bịa**, dưới 1 giây; thêm `-- --rebuild` để đọc lại PBF thay vì nạp chỉ mục đã lưu):

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc31_geocode
```

Cấu hình: Apple Silicon, `--release`, seed 20261106, nguồn `data/vietnam-latest.osm.pbf` (PBF ngày 2026-11-04), chỉ mục lưu ở `data/geocode-index.bin` (không commit). **Các số đo thời gian lấy khi máy đang chịu tải nặng** (`uptime` lúc bắt đầu: load averages 363,18 365,92 323,88); nạp chỉ mục mất 6,4 s, tổng lab 84,6 s. Số đếm, tỉ lệ và sai số mét tái lập từng chữ số với cùng seed và cùng PBF. Kết quả in ra (rút gọn):

```text
-- 4. Đánh giá trên tập thử sinh từ chính OSM --
  THIÊN LỆCH: câu hỏi dựng từ thẻ OSM, đáp án là toạ độ OSM, chỉ mục cũng dựng từ OSM.
  [chế độ GIỮ]
  nhóm  nhiễu               n không KQ  top1@50 top1@200  top3@50 top3@200
  addr  clean           2.000      69    88.2%    90.5%    92.0%    93.9%
  addr  no_diacritics   1.989      71    86.9%    89.5%    90.7%    92.8%
  addr  typo_1          2.000     408    74.3%    76.2%    75.5%    76.9%
  addr  typo_2          2.000     581    67.5%    68.7%    68.5%    69.2%
  [chế độ GỠ]
  addr  clean           2.000      73    50.6%    72.5%    51.9%    74.4%
  chế độ GỠ, câu hỏi sạch: khoảng cách từ ứng viên đầu tới đáp án: p50 44 m, p90 5267 m, p95 11749 m, max 20775 m (n=1.927)
-- 7. Tìm ngược --
  σ = 0 m     3.000        6            2.519              7.7/40.5/49.8             12.0/51.3/92.7        85.0%
  σ = 10 m    3.000        5            2.518             14.8/43.3/81.6            19.0/55.7/102.7        78.9%
```

Test của hai crate chạy bằng `bash examples/toi-noi-location/verify-p04.sh` (dùng fixture bịa, không cần mạng): `geocode` có 21 test đơn vị, 18 test fixture và 7 test proptest. Proptest tìm ra một lỗi thật: dấu kết hợp đứng giữa hai dấu `-` làm `normalize` không idempotent trên chuỗi `0-\u{363}-Ù`; lỗi đã sửa và ca đó nằm trong `props.proptest-regressions`.

## Giới hạn của ví dụ

- **Tập thử sinh từ OSM** (mục trên): phần trăm là độ bền trước nhiễu chữ, không phải độ chính xác với người dùng thật. Không so với Nominatim, Pelias hay dịch vụ nào khác.
- **Không có nội suy số nhà, không có ranh giới hành chính**, địa chỉ ngoài bbox TP.HCM cũ không được phủ.
- **Chưa hỗ trợ**: số nhà có tiền tố chữ (`B2-04`, `B124`), dải nhiều số (`464-466-468`), `Nguyễn Trãi 12` (số đứng sau tên); lỗi gõ ngay trong chữ «đường» không được sửa.
- **Bảng đổi tên hành chính chỉ một phần** và nguồn pháp lý chỉ là trang tổng hợp; xem mục trên.
- Hệ số độ tin cậy là tham số của lab; hiệu chỉnh độ tin cậy ở chế độ GỠ không đơn điệu.

## Bài tập

**Bài 1.** Trong bbox, 7.567 trên 79.839 toà nhà có `addr:housenumber`. Hai con số «88,2%» (chế độ GIỮ) và «50,6%» (chế độ GỠ) cho câu hỏi sạch: con số nào gần với trải nghiệm của khách thật hơn, và vì sao cả hai đều chưa đại diện?

<details><summary>Lời giải bài 1</summary>
<p>9,48% là tỉ lệ toà nhà có số nhà, nên địa chỉ khách gõ phần lớn rơi vào trường hợp «không có trong OSM», gần với chế độ GỠ hơn. Nhưng chế độ GỠ chỉ gỡ những điểm vốn có đủ thẻ địa chỉ trong OSM, tức nơi OSM đã khá đầy đủ, nên đường lân cận vẫn nhiều số nhà để «gần nhất» có nghĩa; ở khu vực OSM hoàn toàn thiếu, kết quả còn tệ hơn. Chế độ GIỮ thì đo thuần nhiễu chữ và bỏ qua hoàn toàn việc thiếu dữ liệu. Cả hai đều dựa trên câu hỏi sinh từ thẻ OSM nên không có địa danh, tên thân mật hay tên phường cũ.</p>
</details>

**Bài 2.** Parser không đọc được `Nguyễn Trãi 12` (số đứng sau tên đường). Đề xuất cách mở rộng mà không làm hỏng ca `3 Tháng 2`, và nêu một ca thử khiến bạn nghi ngờ cách đó.

<details><summary>Lời giải bài 2</summary>
<p>Có thể thử: nếu phần còn lại kết thúc bằng một token số (hoặc số cộng hậu tố) và trước nó là tên đường đã có trong chỉ mục, coi token đó là số nhà. Ca `3 Tháng 2` vẫn đúng vì «3 thang» đứng trước số cuối, và «2» ở cuối là một phần của khoá tên đường `3 thang 2` có trong chỉ mục; cần kiểm tên đường trước khi tách số. Ca đáng nghi: `Đường số 7` hay `Đại lộ 2`, nơi số cuối chính là tên đường, và `Lê Lợi 12` có thể là số nhà 12 hoặc không phải. Cần đo lại trên tập thử có nhãn chứ không chỉ chạy ví dụ.</p>
</details>

**Bài 3.** Vì sao lab gắn trần độ tin cậy 0,5 cho `OldUnmapped` thay vì từ chối trả kết quả? Khi nào từ chối mới là lựa chọn đúng?

<details><summary>Lời giải bài 3</summary>
<p>Tên phường cũ thường chỉ là một trong nhiều tín hiệu: tên đường và số nhà vẫn có giá trị, nên bỏ hẳn kết quả là mất thông tin; trần 0,5 báo cho tầng trên biết phải hỏi lại khách hoặc dùng ghim trên bản đồ. Từ chối là đúng khi truy vấn chỉ có đơn vị hành chính đã bị bỏ (ví dụ chỉ «Quận Bình Thạnh») và không còn gì để xác định một điểm, hay khi hệ quả của việc đón sai chỗ nặng hơn việc hỏi thêm một bước; ngưỡng cụ thể là chính sách sản phẩm, lab không hiệu chỉnh.</p>
</details>

Bài trước: [bài 30](/articles/location-tong-hop-mo-phong-su-kien-roi-rac-seed). Bài tiếp: [bài 32](/articles/location-ham-muc-tieu-diem-hen-khach-va-tai-xe).

## Tài liệu gốc

- [Unicode Standard Annex #15: Unicode Normalization Forms](https://unicode.org/reports/tr15/): NFC, NFD và vì sao hai chuỗi giống nhau có thể khác byte.
- [unicode-normalization trên docs.rs](https://docs.rs/unicode-normalization): crate dùng để đưa chuỗi về NFC, NFD.
- [strsim trên docs.rs](https://docs.rs/strsim): khoảng cách Damerau–Levenshtein dùng cho sửa lỗi gõ.
- [Khoảng cách Damerau–Levenshtein (Wikipedia)](https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance): định nghĩa của phép đo.
- [Geocoding (Wikipedia)](https://en.wikipedia.org/wiki/Geocoding): tổng quan geocoding thuận và ngược.
- [Nghị quyết 202/2025/QH15 trên luatvietnam.vn](https://luatvietnam.vn/hanh-chinh/nghi-quyet-202-2025-qh15-cua-quoc-hoi-ve-viec-sap-xep-don-vi-hanh-chinh-cap-tinh-402728-d1.html): trang tổng hợp, không phải văn bản gốc.
- [Nghị quyết 1685/NQ-UBTVQH15 trên luatvietnam.vn](https://luatvietnam.vn/hanh-chinh/nghi-quyet-1685-nq-ubtvqh15-2025-sap-xep-don-vi-hanh-chinh-cap-xa-tp-ho-chi-minh-403012-d1.html): trang tổng hợp, không phải văn bản gốc.
- [Nghị quyết sửa đổi Hiến pháp trên pbgdpl.cantho.gov.vn](https://pbgdpl.cantho.gov.vn/nghi-quyet-sua-doi-bo-sung-mot-so-dieu-cua-hien-phap-nuoc-cong-hoa-xa-hoi-chu-nghia-viet-nam): trang tổng hợp, không phải văn bản gốc.
