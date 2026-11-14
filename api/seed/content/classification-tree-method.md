---
slug: classification-tree-method
title: "Classification Tree Method: mô hình hoá đầu vào báo giá bằng cây phân loại"
summary: "Classification Tree Method cho POST /api/quotes của Tới Nơi: dựng cây 7 classification, biểu diễn ba loại ràng buộc, đếm 360 tổ hợp, chọn bộ each-choice 9 dòng và base choice 12 dòng, chạy thật trên lab."
category: testing
level: intermediate
tags: [testing, istqb, ctal-ta, test-036, black-box, classification-tree, combinatorial-testing, bao-gia]
language: vi
---

Báo giá của Tới Nơi phụ thuộc bảy yếu tố: dịch vụ, khối lượng bưu kiện, loại xe, thành phố, mưa, cao điểm, voucher. Ghép đủ mọi lựa chọn được 360 tổ hợp — không ai chạy hết bằng tay, và chọn ngẫu hứng thì không biết mình đã bỏ sót gì. Bài này trả lời: làm sao vẽ không gian đầu vào thành một cây phân loại, biểu diễn ràng buộc giữa các lựa chọn, và chọn tổ hợp test **có kiểm soát** — biết chính xác mình phủ gì và chưa phủ gì.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-036</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S04 · Black-box Test Design Techniques</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Advanced — CTAL-TA v4.0 (each choice: Foundation — CTFL v4.0.1)</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L2 · Intermediate</td></tr>
<tr><th scope="row">Test Level</th><td>System; Component</td></tr>
<tr><th scope="row">Test Type</th><td>Functional; Black-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Classification Tree Method</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/equivalence-partitioning">TEST-031 · Equivalence Partitioning</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Dựng classification tree cho <code>POST /api/quotes</code>: dịch vụ, loại xe, thành phố, mưa, cao điểm, voucher (không/hợp lệ/sai phạm vi), ràng buộc «đồ ăn chỉ xe máy» (R-FOOD-06) và «&gt; 20 kg phải ô tô» (R-PARCEL-02); chọn tổ hợp each-choice; mô hình và 25 test chạy thật ở <code>articles/classification-tree-method/</code></td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTAL-TA v4.0 · Chương 3 · mục 3.1.2 · TA-3.1.2 (K3). CTFL v4.0.1 · Chương 4 · mục 4.2.1 (Each Choice coverage) · FL-4.2.1 (K3)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

**Classification Tree Method** (CTM, phương pháp cây phân loại) là cách mô hình hoá miền đầu vào của test object bằng một cây: mỗi nhánh là một **khía cạnh** ảnh hưởng tới hành vi (classification), mỗi lá là một **phân vùng** của khía cạnh đó (class). Dưới cây là **bảng tổ hợp**: mỗi dòng chọn một lá ở mỗi nhánh và trở thành một test case.

Vấn đề CTM giải quyết:

- **Nhiều tham số, mỗi tham số nhiều giá trị.** EP ([TEST-031](/articles/equivalence-partitioning)) chia từng tham số thành phân vùng; CTM sắp các phân vùng đó lên một hình duy nhất để thấy toàn cảnh.
- **Tham số phụ thuộc nhau.** Khối lượng chỉ có nghĩa với bưu kiện; đồ ăn không đi ô tô. Cây diễn đạt quan hệ cha–con, còn ràng buộc ghi bên cạnh cây.
- **Cần chọn tổ hợp có lý do.** Bảng tổ hợp cho biết dòng nào phủ class nào; đổi tiêu chí (each choice, base choice, pairwise) thì số dòng thay đổi theo cách giải thích được.

Nên dùng khi hành vi phụ thuộc nhiều yếu tố đầu vào hoặc môi trường cùng lúc: báo giá, cấu hình thiết bị × hệ điều hành × phương thức thanh toán, điều kiện voucher. CTM là **nền** cho combinatorial testing: [TEST-037](/articles/pairwise-testing) và [TEST-038](/articles/combinatorial-testing) dùng lại mô hình này.

## 2. Lý thuyết theo ISTQB

### 2.1. CTM nằm ở đâu trong syllabus

| Syllabus | Nội dung liên quan |
|---|---|
| CTFL v4.0.1, mục 4.2.1 | Nhiều tập phân vùng (nhiều tham số) ⇒ tiêu chí đơn giản nhất là **Each Choice coverage**: mỗi phân vùng của mỗi tập được thực thi ít nhất một lần, không xét tổ hợp |
| CTAL-TA v4.0, mục 3.1.2 | **Combinatorial testing** (nhóm data-based). Tham số nhiều giá trị thì áp EP trước; ghi tham số và giá trị vào **classification tree** hoặc feature model hỗ trợ việc đó. Số test case cuối cùng chịu ảnh hưởng của ràng buộc, tổ hợp thêm tay vì biết hay gây lỗi, và tổ hợp không hợp lệ/không khả thi |
| CTAL-TA v4.0, phụ lục thay đổi | LO cũ về classification tree diagrams (K2) và pairwise testing đã **gộp** vào TA-3.1.2 (K3) |

Nghĩa là ISTQB coi cây phân loại là **công cụ mô hình hoá** phục vụ combinatorial testing, không phải một tiêu chí coverage riêng. Các tiêu chí CTAL-TA nêu cho combinatorial testing là **base choice** và **pairwise**; each choice đến từ CTFL.

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB</span>
<p>CTM do M. Grochtmann và K. Grimm công bố năm 1993 («Classification Trees for Partition Testing»). Ký hiệu đồ hoạ (gốc, classification hình chữ nhật, class là lá, bảng tổ hợp dạng chấm dưới cây) và các công cụ vẽ cây — Classification Tree Editor và hậu duệ thương mại TESTONA — thuộc thực hành ngành. Tài liệu về CTM hay gọi «mỗi class ít nhất một lần» là <i>minimal coverage</i> và «mọi tổ hợp» là <i>maximal coverage</i>; bài dùng tên each choice của CTFL và «mọi tổ hợp».</p>
</div>

### 2.2. Các thành phần của cây

| Thành phần | Ý nghĩa | Ví dụ Tới Nơi |
|---|---|---|
| Gốc (test object) | Thứ đang được test | `POST /api/quotes` |
| Classification | Một khía cạnh có thể ảnh hưởng tới kết quả; con trực tiếp của gốc hoặc của một class | Dịch vụ, Thành phố, Mưa |
| Class | Một phân vùng tương đương của classification. Các class của cùng classification phải **rời nhau** và **phủ đủ** | `SG`, `HN`, mã không hỗ trợ |
| Classification con | Làm mịn một class; chỉ tồn tại khi class cha được chọn | Khối lượng dưới class `parcel` |
| Bảng tổ hợp | Mỗi dòng chọn đúng một class ở mỗi classification đang mở | E1…E9 trong hình 1 |

Classification không nhất thiết là một trường của request. «Mưa» và «Cao điểm» là **điều kiện môi trường** (dịch vụ thời tiết, đồng hồ hệ thống — R-GEN-02, R-GEN-05), khách không gửi, nhưng làm đổi giá. Bỏ chúng khỏi cây vì «API không có trường này» là sai lầm phổ biến.

### 2.3. Tiêu chí chọn tổ hợp

| Tiêu chí | Nguồn | Yêu cầu | Số dòng với cây của bài |
|---|---|---|---|
| Each choice | CTFL 4.2.1 | Mỗi class xuất hiện ít nhất một lần | tối thiểu 5 dòng hợp lệ (giải thích ở 4.5); bài chọn 9 |
| Base choice | CTAL-TA 3.1.2 | Một dòng gốc; mỗi class không gốc thay vào dòng gốc đúng một lần | 12 |
| Pairwise | CTAL-TA 3.1.2 | Mọi cặp giá trị của mọi cặp tham số | xem [TEST-037](/articles/pairwise-testing) |
| Mọi tổ hợp | — | Mọi tổ hợp dựng được | 300 |

### 2.4. Ba loại ràng buộc

| Loại | Đặc điểm | Xử lý | Ví dụ |
|---|---|---|---|
| **Cấu trúc** | Diễn đạt ngay bằng hình dạng cây | Classification con chỉ mở dưới class cha | Khối lượng chỉ có khi dịch vụ = `parcel` |
| **Loại trừ** (infeasible) | Tổ hợp không dựng được trong thực tế | Loại khỏi mọi bộ test | «Đang mưa» ở thành phố không hỗ trợ: dịch vụ thời tiết không có trạng thái cho mã đó |
| **Quy tắc nghiệp vụ** | Tổ hợp **dựng được** nhưng hệ thống phải từ chối | **Giữ lại** dưới dạng test case âm, mỗi quy tắc ít nhất một dòng | đồ ăn + ô tô (R-FOOD-06); > 20 kg + xe máy (R-PARCEL-02) |

Nhầm loại thứ ba thành loại thứ hai là lỗi thiết kế nguy hiểm: loại «đồ ăn + ô tô» khỏi bảng tổ hợp đồng nghĩa với việc không bao giờ kiểm R-FOOD-06.

### 2.5. Thuật ngữ Anh – Việt

| Thuật ngữ | Nghĩa tiếng Việt dùng trong bài |
|---|---|
| classification tree | cây phân loại |
| classification | khía cạnh phân loại (nhánh) |
| class | lớp / phân vùng (lá) |
| combinatorial testing | kiểm thử tổ hợp |
| interaction failure | failure do tương tác giữa các tham số |
| each choice coverage | độ phủ mỗi lựa chọn |
| base choice coverage | độ phủ lựa chọn gốc |
| constraint | ràng buộc |

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu và business rules

`POST /api/quotes` (R-API-07) trả báo giá có hạn 5 phút. Những quy tắc làm kết quả khác nhau theo tổ hợp đầu vào:

| Yếu tố | Quy tắc | Ảnh hưởng |
|---|---|---|
| Dịch vụ, loại xe | R-GEN-04, R-FARE-01, R-FOOD-04, R-FOOD-06 | Bảng giá; đồ ăn tính phí giao theo công thức xe máy, chỉ nhận `bike` |
| Khối lượng | R-PARCEL-01…04 | Phụ phí 2.000 đ/kg bắt đầu trên 5 kg, **bị nhân** hệ số cao điểm; > 20 kg bắt buộc ô tô |
| Thành phố | R-GEN-03 | 6 mã; mã khác ⇒ `CITY_NOT_SUPPORTED` |
| Mưa | R-FARE-05 | +8.000 đ sau hệ số cao điểm (không bị nhân) |
| Cao điểm | R-FARE-04 | × 1,25 trong [11:00; 13:00) và [17:00; 19:00) giờ VN |
| Voucher | R-VOUCHER-05, -06, R-FOOD-05 | Sai dịch vụ/thành phố ⇒ `VOUCHER_NOT_APPLICABLE`; với đồ ăn giảm trên tiền món |
| Thứ tự lỗi | R-VALID-01 | Định dạng (400) → thành phố → khối lượng → loại xe → tiền món → voucher |

### 3.2. System context

Trong kiến trúc đích, client gọi Customer BFF; **Pricing & Quote Service** tính giá, lấy quãng đường từ MapViet, trạng thái mưa từ dịch vụ thời tiết, điều kiện voucher từ **Promotion Service**, rồi lưu quote có `quote_id`, `expires_at`, `pricing_version` (kiến trúc §4.3). Booking gọi Pricing qua gRPC khi tạo chuyến. Classification môi trường của cây ứng với **phụ thuộc** của service — muốn test được chúng thì phải điều khiển được đồng hồ và thời tiết (testability hooks §17.2).

Lab (SPEC R-ARCH-03) có sẵn các móc đó: `FixedClock` cho giờ, `FakeWeather` cho mưa; quãng đường do client gửi (R-GEN-06).

### 3.3. Rủi ro

- **Interaction failure:** phụ phí khối lượng phải bị nhân 1,25 (R-PARCEL-04) còn phụ phí trời mưa thì không (R-FARE-05). Lỗi chỉ lộ khi bưu kiện nặng **và** cao điểm cùng có mặt.
- **Ràng buộc bị kiểm thiếu:** đồ ăn đi ô tô, bưu kiện 25 kg đi xe máy lọt qua ⇒ đơn không thể giao.
- **Voucher sai phạm vi vẫn giảm:** lộ tiền khuyến mãi (kiến trúc §4.10).
- **Lỗi bị che (masking):** một test case chứa hai giá trị sai chỉ chứng minh được lỗi báo đầu tiên.

## 4. Phân tích kỹ thuật

### 4.1. Chọn classification và class

Cơ sở của từng classification là bảng 3.1; Mưa và Cao điểm là classification môi trường, còn lại là đầu vào. Ba quyết định cần giải thích:

- **Chỉ `SG` và `HN` trong 6 thành phố.** Bốn mã còn lại tương đương `SG` với mọi quy tắc ở 3.1; `HN` tách riêng vì voucher `HN10` chỉ áp ở đó.
- **Quãng đường không là classification.** Phân vùng và biên của nó thuộc [TEST-031](/articles/equivalence-partitioning), [TEST-032](/articles/boundary-value-analysis); ở đây cố định giá trị đại diện (xe 7,4 km; bưu kiện 10 km; đồ ăn 3 km) để cây tập trung vào **tương tác**.
- **Khối lượng không có class không hợp lệ.** Biên 0 và 30 kg thuộc BVA; cây chỉ giữ ba vùng hợp lệ có hành vi khác nhau (không phụ phí / phụ phí / phụ phí + bắt buộc ô tô).

### 4.2. Cây và bảng tổ hợp

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 400" role="img" aria-label="Cây phân loại của POST /api/quotes: gốc nối tới sáu classification Dịch vụ, Loại xe, Thành phố, Mưa, Cao điểm, Voucher; class parcel có classification con Khối lượng với ba class; bên dưới là bảng tổ hợp chín dòng E1 tới E9, năm dòng đầu hợp lệ chấm tím, bốn dòng sau là test case âm chấm cam.">
<rect x="400" y="6" width="180" height="26" rx="6" class="f-pk-soft s-pk" stroke-width="2"/>
<text x="490" y="24" text-anchor="middle" class="t-b t-mono">POST /api/quotes</text>
<rect x="98" y="52" width="230" height="24" rx="4" class="f-surface s-line"/>
<text x="213" y="69" text-anchor="middle" class="t-b">Dịch vụ</text>
<rect x="330.5" y="52" width="90.5" height="24" rx="4" class="f-surface s-line"/>
<text x="375.8" y="69" text-anchor="middle" class="t-b">Loại xe</text>
<rect x="423.5" y="52" width="137" height="24" rx="4" class="f-surface s-line"/>
<text x="492" y="69" text-anchor="middle" class="t-b">Thành phố</text>
<rect x="563" y="52" width="90.5" height="24" rx="4" class="f-surface s-line"/>
<text x="608.2" y="69" text-anchor="middle" class="t-b">Mưa</text>
<rect x="656" y="52" width="90.5" height="24" rx="4" class="f-surface s-line"/>
<text x="701.2" y="69" text-anchor="middle" class="t-b">Cao điểm</text>
<rect x="749" y="52" width="137" height="24" rx="4" class="f-surface s-line"/>
<text x="817.5" y="69" text-anchor="middle" class="t-b">Voucher</text>
<rect x="144.5" y="118" width="137" height="24" rx="4" class="f-surface s-line"/>
<text x="213" y="135" text-anchor="middle" class="t-b">Khối lượng (kg)</text>
<path d="M490 32L213 52 M490 32L375.8 52 M490 32L492 52 M490 32L608.2 52 M490 32L701.2 52 M490 32L817.5 52 M213 76L120 91 M213 76L213 91 M213 76L306 91 M213 109V118 M213 142L166.5 155 M213 142L213 155 M213 142L259.5 155 M375.8 76L352.5 91 M375.8 76L399 91 M492 76L445.5 91 M492 76L492 91 M492 76L538.5 91 M608.2 76L585 91 M608.2 76L631.5 91 M701.2 76L678 91 M701.2 76L724.5 91 M817.5 76L771 91 M817.5 76L817.5 91 M817.5 76L864 91" class="f-none s-muted"/>
<text x="120" y="104" text-anchor="middle">ride</text>
<text x="213" y="104" text-anchor="middle" class="t-b">parcel</text>
<text x="306" y="104" text-anchor="middle">food</text>
<text x="166.5" y="168" text-anchor="middle">≤ 5</text>
<text x="213" y="168" text-anchor="middle">5–20</text>
<text x="259.5" y="168" text-anchor="middle">20–30</text>
<text x="352.5" y="104" text-anchor="middle">bike</text>
<text x="399" y="104" text-anchor="middle">car</text>
<text x="445.5" y="104" text-anchor="middle">SG</text>
<text x="492" y="104" text-anchor="middle">HN</text>
<text x="538.5" y="104" text-anchor="middle">VT</text>
<text x="585" y="104" text-anchor="middle">có</text>
<text x="631.5" y="104" text-anchor="middle">không</text>
<text x="678" y="104" text-anchor="middle">có</text>
<text x="724.5" y="104" text-anchor="middle">không</text>
<text x="771" y="104" text-anchor="middle">không</text>
<text x="817.5" y="104" text-anchor="middle">GIAM20</text>
<text x="864" y="104" text-anchor="middle">sai PV</text>
<path d="M120 110V392 M166.5 174V392 M213 174V392 M259.5 174V392 M306 110V392 M352.5 110V392 M399 110V392 M445.5 110V392 M492 110V392 M538.5 110V392 M585 110V392 M631.5 110V392 M678 110V392 M724.5 110V392 M771 110V392 M817.5 110V392 M864 110V392" class="f-none s-line dash"/>
<path d="M10 186H910" class="s-muted"/>
<text x="14" y="181" class="t-small">Bảng tổ hợp</text>
<text x="20" y="208" class="t-b">E1</text>
<circle cx="120" cy="204" r="5.5" class="f-pk"/>
<circle cx="352.5" cy="204" r="5.5" class="f-pk"/>
<circle cx="445.5" cy="204" r="5.5" class="f-pk"/>
<circle cx="631.5" cy="204" r="5.5" class="f-pk"/>
<circle cx="724.5" cy="204" r="5.5" class="f-pk"/>
<circle cx="771" cy="204" r="5.5" class="f-pk"/>
<text x="20" y="230" class="t-b">E2</text>
<circle cx="166.5" cy="226" r="5.5" class="f-pk"/>
<circle cx="352.5" cy="226" r="5.5" class="f-pk"/>
<circle cx="492" cy="226" r="5.5" class="f-pk"/>
<circle cx="585" cy="226" r="5.5" class="f-pk"/>
<circle cx="678" cy="226" r="5.5" class="f-pk"/>
<circle cx="817.5" cy="226" r="5.5" class="f-pk"/>
<text x="20" y="252" class="t-b">E3</text>
<circle cx="213" cy="248" r="5.5" class="f-pk"/>
<circle cx="399" cy="248" r="5.5" class="f-pk"/>
<circle cx="445.5" cy="248" r="5.5" class="f-pk"/>
<circle cx="585" cy="248" r="5.5" class="f-pk"/>
<circle cx="724.5" cy="248" r="5.5" class="f-pk"/>
<circle cx="771" cy="248" r="5.5" class="f-pk"/>
<text x="20" y="274" class="t-b">E4</text>
<circle cx="259.5" cy="270" r="5.5" class="f-pk"/>
<circle cx="399" cy="270" r="5.5" class="f-pk"/>
<circle cx="492" cy="270" r="5.5" class="f-pk"/>
<circle cx="631.5" cy="270" r="5.5" class="f-pk"/>
<circle cx="678" cy="270" r="5.5" class="f-pk"/>
<circle cx="771" cy="270" r="5.5" class="f-pk"/>
<text x="20" y="296" class="t-b">E5</text>
<circle cx="306" cy="292" r="5.5" class="f-pk"/>
<circle cx="352.5" cy="292" r="5.5" class="f-pk"/>
<circle cx="445.5" cy="292" r="5.5" class="f-pk"/>
<circle cx="631.5" cy="292" r="5.5" class="f-pk"/>
<circle cx="678" cy="292" r="5.5" class="f-pk"/>
<circle cx="817.5" cy="292" r="5.5" class="f-pk"/>
<text x="20" y="318" class="t-b t-warn">E6</text>
<circle cx="120" cy="314" r="5.5" class="f-warn"/>
<circle cx="352.5" cy="314" r="5.5" class="f-warn"/>
<circle cx="538.5" cy="314" r="5.5" class="f-warn"/>
<circle cx="631.5" cy="314" r="5.5" class="f-warn"/>
<circle cx="724.5" cy="314" r="5.5" class="f-warn"/>
<circle cx="771" cy="314" r="5.5" class="f-warn"/>
<text x="20" y="340" class="t-b t-warn">E7</text>
<circle cx="120" cy="336" r="5.5" class="f-warn"/>
<circle cx="352.5" cy="336" r="5.5" class="f-warn"/>
<circle cx="445.5" cy="336" r="5.5" class="f-warn"/>
<circle cx="631.5" cy="336" r="5.5" class="f-warn"/>
<circle cx="724.5" cy="336" r="5.5" class="f-warn"/>
<circle cx="864" cy="336" r="5.5" class="f-warn"/>
<text x="20" y="362" class="t-b t-warn">E8</text>
<circle cx="306" cy="358" r="5.5" class="f-warn"/>
<circle cx="399" cy="358" r="5.5" class="f-warn"/>
<circle cx="445.5" cy="358" r="5.5" class="f-warn"/>
<circle cx="631.5" cy="358" r="5.5" class="f-warn"/>
<circle cx="724.5" cy="358" r="5.5" class="f-warn"/>
<circle cx="771" cy="358" r="5.5" class="f-warn"/>
<text x="20" y="384" class="t-b t-warn">E9</text>
<circle cx="259.5" cy="380" r="5.5" class="f-warn"/>
<circle cx="352.5" cy="380" r="5.5" class="f-warn"/>
<circle cx="445.5" cy="380" r="5.5" class="f-warn"/>
<circle cx="631.5" cy="380" r="5.5" class="f-warn"/>
<circle cx="724.5" cy="380" r="5.5" class="f-warn"/>
<circle cx="771" cy="380" r="5.5" class="f-warn"/>
</svg>
</div>
<figcaption><b>Hình 1 · Cây phân loại và bảng tổ hợp each-choice.</b> Mỗi cột là một class (lá); mỗi dòng chọn đúng một chấm ở mỗi classification đang mở. «Khối lượng» chỉ mở dưới <code>parcel</code>, nên dòng <code>ride</code>/<code>food</code> không có chấm ở ba cột đó. E1–E5 (tím) dùng toàn class hợp lệ; E6, E7 (cam) mỗi dòng chứa đúng một class không hợp lệ; E8, E9 (cam) vi phạm một ràng buộc quy tắc.</figcaption>
</figure>

### 4.3. Ràng buộc của cây

| Mã | Ràng buộc | Loại | Xử lý trong bộ test |
|---|---|---|---|
| RB-1 | Khối lượng chỉ có khi Dịch vụ = `parcel` | cấu trúc | Hình dạng cây |
| RB-2 | Mưa = có ⇒ Thành phố ∈ {SG, HN} | loại trừ | Bỏ 60 tổ hợp |
| RB-3 | Dịch vụ = `food` ⇒ Loại xe = `bike` (R-FOOD-06) | quy tắc | E8: mong đợi 400 `VALIDATION_ERROR` |
| RB-4 | Khối lượng (20; 30] ⇒ Loại xe = `car` (R-PARCEL-02) | quy tắc | E9: mong đợi 422 `VEHICLE_TOO_SMALL` |

Class «voucher sai phạm vi» cần chọn mã cụ thể theo tổ hợp: `HN10` (chỉ `ride` ở HN) cho mọi tổ hợp trừ `ride` ở HN, nơi dùng `FOOD15` (chỉ `food`). Ánh xạ class ⇒ giá trị cụ thể là một phần của mô hình (hàm `toRequest` ở 5.2).

### 4.4. Đếm tổ hợp

Theo cây: (1 `ride` + 3 khối lượng của `parcel` + 1 `food`) × 2 loại xe × 3 thành phố × 2 mưa × 2 cao điểm × 3 voucher = **360**. Bỏ 60 tổ hợp vi phạm RB-2 còn **300** tổ hợp dựng được. Phân theo kết quả mong đợi (oracle theo thứ tự R-VALID-01, đếm bằng code ở 5.3):

| Kết quả | Số tổ hợp |
|---|---|
| 200 — có báo giá | 128 |
| 422 `VOUCHER_NOT_APPLICABLE` | 64 |
| 422 `CITY_NOT_SUPPORTED` | 54 |
| 400 `VALIDATION_ERROR` (đồ ăn + ô tô, kể cả ở mã không hỗ trợ — kiểm định dạng đứng trước) | 30 |
| 422 `VEHICLE_TOO_SMALL` | 24 |

172 trong 300 tổ hợp chỉ kiểm **đường lỗi**, phần lớn lặp lại cùng một lỗi: chạy hết cũng không có nghĩa là «test kỹ hơn».

### 4.5. Chọn bộ each-choice

**Cận dưới.** Classification nhiều class nhất có 3 class, nên nếu nghĩ ngây thơ thì 3 dòng là đủ. Nhưng ba class khối lượng chỉ xuất hiện trong dòng `parcel` ⇒ cần 3 dòng `parcel`, cộng 1 dòng `ride` và 1 dòng `food` ⇒ **ít nhất 5 dòng hợp lệ**. Cấu trúc cây làm tăng cận dưới.

**Class không hợp lệ đi một mình.** Theo R-VALID-01, hệ thống dừng ở lỗi đầu tiên. Dòng chứa thành phố `VT` sẽ dừng ở `CITY_NOT_SUPPORTED`; mọi class khác trong dòng đó **không được xử lý**. Vì vậy mỗi class không hợp lệ cần dòng riêng, các class còn lại lấy giá trị hợp lệ đã phủ ⇒ +2 dòng (E6, E7). Mỗi ràng buộc quy tắc thêm một dòng âm ⇒ +2 dòng (E8, E9). Tổng **9 dòng**.

| Dòng | Tổ hợp | Mong đợi | Tính tay |
|---|---|---|---|
| E1 | ride · bike · SG · khô · thường · không voucher | 200, **46.000 đ** | 12.000 + 4.500 × 7,4 = 45.300 ⇒ 46.000 |
| E2 | parcel 3 kg · bike · HN · mưa · cao điểm · GIAM20 | 200, **64.000 đ** | (12.000 + 45.000) × 1,25 = 71.250; + 8.000 = 79.250; − 15.850 = 63.400 ⇒ 64.000 |
| E3 | parcel 12 kg · car · SG · mưa · thường · không | 200, **157.000 đ** | 25.000 + 110.000 + 2.000 × ⌈7⌉ = 149.000; + 8.000 = 157.000 |
| E4 | parcel 25 kg · car · HN · khô · cao điểm · không | 200, **219.000 đ** | (135.000 + 40.000) × 1,25 = 218.750 ⇒ 219.000 |
| E5 | food 60.000 đ · bike · SG · khô · cao điểm · GIAM20 | 200, **80.000 đ** | phí giao 25.500 × 1,25 = 31.875 ⇒ 32.000; giảm 12.000 trên tiền món; 60.000 − 12.000 + 32.000 |
| E6 | ride · bike · **VT** · khô · thường · không | 422 `CITY_NOT_SUPPORTED` | — |
| E7 | ride · bike · SG · khô · thường · **HN10** | 422 `VOUCHER_NOT_APPLICABLE` | — |
| E8 | **food · car** · SG · khô · thường · không | 400 `VALIDATION_ERROR` | — |
| E9 | **parcel 25 kg · bike** · SG · khô · thường · không | 422 `VEHICLE_TOO_SMALL` | — |

E4 tình cờ ghép bưu kiện nặng với cao điểm nên bắt được rủi ro R-PARCEL-04: nếu phụ phí không bị nhân, kết quả là 135.000 × 1,25 + 40.000 = 208.750 ⇒ 209.000 đ. Nhưng «tình cờ» là điểm yếu của each choice — xem mục 6.

## 5. Thực hành

### 5.1. Đề bài và lời giải

Đề bài: *dựng classification tree cho `POST /api/quotes` (dịch vụ, loại xe, thành phố, mưa, cao điểm, voucher không/hợp lệ/sai phạm vi) với ràng buộc «đồ ăn chỉ xe máy» và «> 20 kg phải ô tô», chọn tổ hợp theo each choice.* Lời giải: cây ở 4.1–4.2, ràng buộc ở 4.3, bộ E1–E9 ở 4.5. Phần dưới biến mô hình thành code và chạy trên lab.

### 5.2. Mô hình là dữ liệu

`articles/classification-tree-method/cay-phan-loai.ts` khai báo cây thành mảng `TREE` (classification, class, quan hệ cha), cùng các hàm thuần: `allCombos` sinh 360 tổ hợp theo cây, `isInfeasible` (RB-2), `expectedOutcome` (oracle loại kết quả theo R-VALID-01, gồm RB-3 và RB-4), `uncoveredClasses` kiểm each choice, `baseChoice` sinh bộ base choice, `toRequest` đổi tổ hợp thành thân yêu cầu + giờ + thành phố đang mưa. Hàm `uncoveredEffective` là bản chặt hơn của kiểm each choice: một class chỉ tính là đã phủ ở dòng trả 200, hoặc ở dòng mà chính class đó gây ra lỗi.

`cay-phan-loai.test.ts` có 25 test: 4 test về mô hình, 9 dòng E1–E9 chạy trên API (kiểm mã kết quả và tổng tiền tính tay ở 4.5), 12 dòng base choice chạy trên API (kiểm mã kết quả). Mỗi dòng dựng hệ thống riêng với `FixedClock` lúc 09:00 hoặc 11:30 giờ VN ngày 04/11/2026 và `rainingCities` theo class Mưa.

### 5.3. Lần chạy thật

Chạy trong `examples/toi-noi-testing` (Node 22.18, vitest 3.2.7):

```bash
npx vitest run articles/classification-tree-method
```

```text
stdout | articles/classification-tree-method/cay-phan-loai.test.ts > Mô hình cây phân loại > đếm tổ hợp: 360 theo cây, 60 bị loại trừ, 300 dựng được — phân theo loại kết quả
Tổ hợp theo cây: 360 · loại trừ: 60 · dựng được: 300 {
  OK: 128,
  VOUCHER_NOT_APPLICABLE: 64,
  CITY_NOT_SUPPORTED: 54,
  VEHICLE_TOO_SMALL: 24,
  VALIDATION_ERROR: 30
}

stdout | articles/classification-tree-method/cay-phan-loai.test.ts > Mô hình cây phân loại > bộ «ngây thơ» 5 dòng đủ mặt mọi class nhưng hai class bị lỗi thành phố che
Bộ ngây thơ — class bị che: weight=vua, voucher=sai-pham-vi

stdout | articles/classification-tree-method/cay-phan-loai.test.ts > Mô hình cây phân loại > base choice từ dòng gốc E1: 12 dòng, đúng một dòng rơi vào ràng buộc «> 20 kg phải ô tô»
B01 ride bike SG khô thường voucher:khong ⇒ OK
… (B02–B04 lược bớt: đều OK)
B05 parcel/nang bike SG khô thường voucher:khong ⇒ VEHICLE_TOO_SMALL
… (B06–B11 lược bớt: OK, riêng B08 thành phố VT ⇒ CITY_NOT_SUPPORTED)
B12 ride bike SG khô thường voucher:sai-pham-vi ⇒ VOUCHER_NOT_APPLICABLE

 ✓ articles/classification-tree-method/cay-phan-loai.test.ts (25 tests) 174ms

 Test Files  1 passed (1)
      Tests  25 passed (25)
```

Năm tổng tiền tính tay ở 4.5 khớp lab; bốn dòng âm trả đúng mã lỗi; 12 dòng base choice trả đúng loại kết quả.

### 5.4. Hai điều lần chạy cho thấy

**Đủ mặt chưa chắc đã phủ.** Bộ «ngây thơ» 5 dòng trong test có mọi class xuất hiện — kiểm each choice kiểu đếm đơn thuần báo 100%. Nhưng dòng thứ ba gói `parcel` 12 kg, voucher sai phạm vi và thành phố `VT` vào một chỗ: hệ thống dừng ở `CITY_NOT_SUPPORTED`, hai class kia chưa bao giờ được xử lý. Bản kiểm chặt báo đúng hai class bị che. Đây là lý do E6, E7 đứng riêng.

**Base choice vấp ràng buộc.** Thay khối lượng (20; 30] vào dòng gốc (xe máy) sinh ra B05 vi phạm RB-4: dòng đó thành test case âm, còn class (20; 30] thì **chưa** được kiểm ở đường thành công (`uncoveredEffective` của bộ base choice trả `weight=nang`). Người thiết kế phải thêm dòng «sửa» `parcel 25 kg · car` — chính là E4. CTAL-TA nói đúng hiện tượng này: số test case cuối cùng chịu ảnh hưởng của ràng buộc và tổ hợp không hợp lệ.

## 6. Góc nhìn nâng cao

**Each choice không phủ tương tác.** E4 bắt được lỗi «phụ phí khối lượng không bị nhân cao điểm» chỉ vì tình cờ ghép (20; 30] với cao điểm. Đổi cao điểm từ E4 sang E3 vẫn đạt each choice (E3 có phụ phí 14.000 đ nên vẫn bắt được), nhưng nếu dòng cao điểm duy nhất của bưu kiện là E2 (≤ 5 kg, phụ phí 0) thì lỗi lọt hoàn toàn. Muốn **bảo đảm** mọi cặp (khối lượng, cao điểm) được thử, cần pairwise — [TEST-037](/articles/pairwise-testing); bộ ba trở lên thì t-wise — [TEST-038](/articles/combinatorial-testing). CTAL-TA dẫn một nghiên cứu giới hạn cho thấy phần lớn failure do một hoặc hai điều kiện tương tác gây ra — lý do pairwise hay được chọn làm mức mặc định.

**Thêm tay các tổ hợp đã biết hay gây lỗi** — CTAL-TA nêu đây là một yếu tố quyết định số test case. Ví dụ: bưu kiện nặng + cao điểm + mưa (hai phụ phí cùng lúc, kiểm thứ tự R-FARE-03); đồ ăn + `FOOD15` khi tiền món dưới 50.000 đ.

**So với kỹ thuật khác.**

| Kỹ thuật | Khi nào hợp hơn CTM |
|---|---|
| EP ([TEST-031](/articles/equivalence-partitioning)) | Một tham số, cần phân vùng kỹ; CTM dùng kết quả EP làm class |
| Decision table ([TEST-033](/articles/decision-table-testing)) | Quy tắc logic «nếu các điều kiện thì hành động», cần phủ đủ mọi quy tắc (vd phí huỷ); CTM mạnh khi tham số nhiều giá trị và cần chọn tổ hợp đại diện |
| Domain testing ([TEST-040](/articles/data-based-test-techniques-domain-testing)) | Biên phụ thuộc nhiều biến cùng lúc (vd khối lượng × loại xe ở đúng 20,0 kg) |
| Scenario-based ([TEST-035](/articles/use-case-testing-scenario-based-testing)) | Hành vi phụ thuộc chuỗi bước, không chỉ đầu vào của một lần gọi |

**Sai lầm thường gặp:** dựng classification theo **trường API** thay vì khía cạnh ảnh hưởng hành vi (bỏ sót mưa, cao điểm); class chồng nhau hoặc thiếu (quên «không voucher»); đưa mọi giá trị biên vào cây (để biên cho BVA); coi ràng buộc quy tắc là loại trừ; dồn nhiều class không hợp lệ vào một dòng; cây sâu quá ba tầng (thường là dấu hiệu nên tách test object).

**Cây cũng là tài liệu review.** Vẽ cây buộc ta hỏi về từng cặp class «đã có quy tắc chưa?». Ví dụ «đồ ăn + mưa: phí giao có cộng 8.000 đ không?» — R-FOOD-04 trả lời có. Câu hỏi nào **không** tìm được câu trả lời trong đặc tả là một anomaly cần gửi BA trước khi viết test ([TEST-021](/articles/static-testing-la-gi)).

## 7. Bài tập và câu hỏi ôn tập

### Bài tập 1 — Thêm classification «Tiền món»

Thêm classification con «Tiền món» dưới class `food` với ba class: < 30.000 đ, 30.000–49.999 đ, ≥ 50.000 đ, và đổi class «voucher hợp lệ» của đồ ăn thành `FOOD15` (tối thiểu 50.000 đ). (a) Có ràng buộc mới nào? Thuộc loại gì? (b) Cận dưới của số dòng hợp lệ trong bộ each-choice là bao nhiêu?

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(a) Class «&lt; 30.000 đ» là class <b>không hợp lệ</b> (R-FOOD-03 ⇒ <code>SUBTOTAL_TOO_LOW</code>) — cần dòng riêng. Tổ hợp «30.000–49.999 đ + FOOD15» dựng được nhưng bị từ chối (<code>VOUCHER_MIN_AMOUNT</code>, R-VOUCHER-06) ⇒ ràng buộc <b>quy tắc</b>, giữ một dòng âm; muốn class 30.000–49.999 được phủ ở đường thành công thì dòng đó dùng «không voucher» hoặc GIAM20. (b) Dòng hợp lệ: 1 <code>ride</code> + 3 <code>parcel</code> + 2 <code>food</code> (hai class tiền món hợp lệ) = <b>6</b>.</p>
</details>

### Bài tập 2 — Phân loại ràng buộc

Xếp loại (cấu trúc / loại trừ / quy tắc): (a) «Khối lượng» chỉ có với bưu kiện; (b) chọn `food` và `car`; (c) giả sử cây có thêm classification «Khung giờ» (00:00–06:00, 06:00–24:00): Cao điểm = có cùng Khung giờ 00:00–06:00; (d) voucher `HN10` cho chuyến ở SG.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>(a) cấu trúc — classification con dưới <code>parcel</code>. (b) quy tắc — dựng được, phải nhận 400 (R-FOOD-06). (c) loại trừ — giờ cao điểm không nằm trong 00:00–06:00 (R-FARE-04), hai class mâu thuẫn, không dựng được. (d) không phải ràng buộc mà là class «voucher sai phạm vi» được hiện thực bằng mã cụ thể — mong đợi <code>VOUCHER_NOT_APPLICABLE</code>.</p>
</details>

### Câu hỏi trắc nghiệm

**Câu 1 (K1).** Theo CTAL-TA v4.0, classification tree được dùng để làm gì trong combinatorial testing?

- a) Thay thế equivalence partitioning.
- b) Ghi lại tham số và giá trị của chúng, hỗ trợ việc giảm số giá trị và tổ hợp.
- c) Đo statement coverage của code tính giá.
- d) Sinh dữ liệu ngẫu nhiên theo operational profile.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — class chính là kết quả của EP; cây dùng EP chứ không thay nó. <b>b đúng</b> — CTAL-TA 3.1.2. <b>c sai</b> — đó là white-box. <b>d sai</b> — đó là random testing (3.1.3).</p>
</details>

**Câu 2 (K2).** Các class của cùng một classification phải thoả điều kiện nào?

- a) Chồng nhau để chắc không bỏ sót.
- b) Rời nhau và cùng nhau phủ đủ classification.
- c) Có đúng hai class.
- d) Chỉ gồm giá trị hợp lệ.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — chồng nhau làm một giá trị thuộc hai class, coverage mất nghĩa. <b>b đúng</b> — class là phân vùng tương đương. <b>c sai</b> — số class tuỳ khía cạnh (Thành phố có 3). <b>d sai</b> — class không hợp lệ (vd mã thành phố không hỗ trợ) là một phần của mô hình.</p>
</details>

**Câu 3 (K3).** Một cây có Hệ điều hành (Android, iOS, Web), Phương thức (cash, vixanh), Ngôn ngữ (vi, en) và classification con «Phiên bản» (12, 13, 14, 15) chỉ dưới class Android. Không có ràng buộc khác. Số dòng tối thiểu để đạt each choice là:

- a) 3.
- b) 4.
- c) 6.
- d) 48.

<details>
<summary><b>Đáp án: c</b></summary>
<p>Bốn class Phiên bản chỉ xuất hiện trong dòng Android ⇒ 4 dòng Android, cộng 1 dòng iOS và 1 dòng Web ⇒ 6; hai classification còn lại (2 class mỗi cái) phủ được trong 6 dòng đó. <b>a sai</b> — bỏ qua classification con. <b>b sai</b> — 4 dòng Android không phủ iOS và Web. <b>d sai</b> — 48 = 3 × 4 × 2 × 2, như thể mọi hệ điều hành đều có classification Phiên bản; số tổ hợp đầy đủ đúng là (4 + 1 + 1) × 2 × 2 = 24.</p>
</details>

**Câu 4 (K2).** Vì sao bộ each-choice của bài đặt thành phố `VT` và voucher sai phạm vi ở hai dòng khác nhau?

- a) Vì each choice yêu cầu mỗi dòng chỉ có một class không hợp lệ.
- b) Vì hệ thống dừng ở lỗi đầu tiên; class không hợp lệ thứ hai trong cùng dòng không được xử lý.
- c) Vì hai class đó vi phạm ràng buộc loại trừ.
- d) Để tăng số dòng cho đủ tiêu chí.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — each choice chỉ đòi mỗi class xuất hiện; luật «một class không hợp lệ mỗi dòng» là thực hành chống che lỗi. <b>b đúng</b> — R-VALID-01: dừng ở <code>CITY_NOT_SUPPORTED</code>, voucher không bao giờ được kiểm. <b>c sai</b> — tổ hợp đó dựng được. <b>d sai</b> — tăng dòng không có mục đích là lãng phí.</p>
</details>

**Câu 5 (K3).** Dòng gốc là (ride, bike, SG, khô, thường, không voucher). Theo base choice, dòng nào **thuộc** bộ test?

- a) (parcel 25 kg, car, HN, mưa, thường, không voucher).
- b) (ride, car, SG, khô, cao điểm, không voucher).
- c) (ride, bike, SG, khô, thường, voucher sai phạm vi).
- d) (food, bike, HN, khô, thường, GIAM20).

<details>
<summary><b>Đáp án: c</b></summary>
<p>Base choice thay <b>đúng một</b> class của dòng gốc mỗi lần. <b>a sai</b> — đổi nhiều classification. <b>b sai</b> — đổi hai (loại xe, cao điểm). <b>c đúng</b> — chỉ đổi Voucher. <b>d sai</b> — đổi ba.</p>
</details>

**Câu 6 (K2).** Đội loại tổ hợp «đồ ăn + ô tô» khỏi bảng tổ hợp vì «không ai đặt đồ ăn bằng ô tô». Hệ quả là gì?

- a) Không sao — tổ hợp không khả thi thì phải loại.
- b) R-FOOD-06 không bao giờ được kiểm; client gửi `car` vẫn có thể lọt nếu backend thiếu kiểm tra.
- c) Bộ test vi phạm each choice.
- d) Số tổ hợp dựng được tăng lên.

<details>
<summary><b>Đáp án: b</b></summary>
<p><b>a sai</b> — tổ hợp này <b>dựng được</b> (gửi <code>vehicle: "car"</code>), là ràng buộc quy tắc chứ không phải loại trừ. <b>b đúng</b>. <b>c sai</b> — each choice vẫn đạt vì <code>car</code> có ở dòng khác. <b>d sai</b> — loại bớt thì giảm.</p>
</details>

## 8. Tài liệu tham khảo

- ISTQB® Certified Tester Advanced Level — Test Analyst Syllabus **v4.0** (2025): mục 3.1 Data-Based Test Techniques, 3.1.2 Combinatorial Testing; phụ lục thay đổi LO; LO TA-3.1.2 (K3). PDF: https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB® Certified Tester Foundation Level Syllabus **v4.0.1** (2024): mục 4.2.1 Equivalence Partitioning (Each Choice coverage); LO FL-4.2.1 (K3). PDF: https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary (tra *classification tree*, *combinatorial testing*, *equivalence partition*) — https://glossary.istqb.org/
- M. Grochtmann, K. Grimm, «Classification Trees for Partition Testing», *Software Testing, Verification and Reliability*, 1993 — bài gốc của CTM (ngoài ISTQB).
- ISO/IEC/IEEE 29119-4 — Software testing, Part 4: Test techniques (có mô tả classification tree method và combinatorial techniques).
- Đặc tả và mã của lab Tới Nơi (`SPEC.md` nhóm R-FARE, R-PARCEL, R-FOOD, R-VOUCHER, R-VALID; test của bài): https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [TEST-031 · Equivalence Partitioning](/articles/equivalence-partitioning) · [TEST-033 · Decision Table Testing](/articles/decision-table-testing) · [TEST-035 · Use Case Testing và Scenario-based Testing](/articles/use-case-testing-scenario-based-testing) · [TEST-038 · Combinatorial Testing](/articles/combinatorial-testing)

Bài tiếp theo: [Pairwise Testing](/articles/pairwise-testing).
