---
slug: defect-based-testing
title: "Defect-based Testing: phân loại 30 defect lịch sử và thiết kế test cho cụm lỗi hay tái diễn"
summary: "Thiết kế test từ taxonomy và lịch sử defect: phân loại 30 defect của Tới Nơi theo nhiều trục, tìm cụm lỗi tái diễn, dựng mô hình lỗi cho làm tròn, giờ Việt Nam, trạng thái chuyến và chứng minh test bắt được defect cũ."
category: testing
level: advanced
tags: [testing, istqb, ctal-ta, ctfl, test-054, defect-based-testing, defect-taxonomy, root-cause-analysis, vitest, toi-noi]
language: vi
---

Sáu tháng chạy bản MVP, Tới Nơi sửa 30 defect. Sửa xong cái nào đóng cái đó. Rồi D-09 «tiền giảm bưu kiện lệch 1.000 đ» lọt lên production — đúng kiểu lỗi D-06 đã sửa ở nhánh đặt xe. Rồi D-18 «chuyến hoàn thành quay về đang tìm tài xế» — họ hàng của D-08. Bài này trả lời: làm sao biến **lịch sử defect** thành **test nhắm đúng loại lỗi hay tái diễn**, và vì sao cách phân loại quyết định bạn có thấy cụm lỗi hay không.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-054</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S05 · White-box &amp; Experience-based Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Advanced — CTAL-TA v4.0 (mở rộng); Foundation — CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Advanced</td></tr>
<tr><th scope="row">Test Level</th><td>Component; System</td></tr>
<tr><th scope="row">Test Type</th><td>Black-box</td></tr>
<tr><th scope="row">Test Technique</th><td>Defect-based Testing</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/error-defect-failure-root-cause">TEST-004 Error, Defect, Failure, Root Cause</a>, <a href="/articles/error-guessing">TEST-051 Error Guessing</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Phân loại 30 defect lịch sử (giả định) theo taxonomy, thiết kế test cho 3 cụm hay tái diễn — lab <code>articles/defect-based-testing</code> (CSV + 17 test vitest)</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTAL-TA v4.0 · Chương 5 · mục 5.3.2 · TA-5.3.2 (K2); liên quan mục 5.3.1 · TA-5.3.1 (K4), mục 3.4.2. CTFL v4.0.1 · mục 4.4.1 · FL-4.4.1 (K2); mục 1.3 · FL-1.3.1 (K2)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

**Defect-based testing** (kiểm thử dựa trên defect) là cách thiết kế test bắt đầu từ **loại defect** chứ không từ đặc tả hay cấu trúc code: «chỗ nào có làm tròn tiền?», «chỗ nào có so ngày?» — rồi viết test nhắm đúng cách mà loại lỗi đó từng xảy ra. Nguyên liệu là **taxonomy** (bảng phân loại defect) và **lịch sử defect** của chính dự án.

Vấn đề mà kỹ thuật này giải quyết: bộ test thiết kế theo đặc tả phủ đều mọi quy tắc, nhưng defect thì không rải đều. Chúng **tụ thành cụm** và **lặp lại** — cùng một kiểu sai xuất hiện ở module khác, ở người khác, sau bản sửa chỉ vá một chỗ. Dùng khi:

- Đã có dữ liệu defect đủ nhiều (vài chục trở lên) và nhất quán.
- Một loại lỗi lặp lại hoặc lọt production dù đã «sửa».
- Cần chọn chỗ đầu tư test thêm có lý do, không chia đều.

## 2. Lý thuyết theo ISTQB

### 2.1. ISTQB nói gì — và không nói gì

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Nội dung mở rộng — đọc trước</span>
<p>CTAL-TA v4.0 <b>không</b> có learning objective nào tên «defect-based test technique». Bài ghép ba mảnh có trong syllabus hiện hành rồi thêm phần thiết kế test từ mô hình lỗi (mở rộng, gắn nhãn ở 2.3). Không viện dẫn syllabus cũ.</p>
</div>

| Mảnh | Syllabus | Nội dung (diễn đạt lại) | Dùng trong bài |
|---|---|---|---|
| Lỗi hay gặp | CTFL 4.4.1 (FL-4.4.1) | Error, defect, failure thường liên quan tới: **input, output, logic, computation, interfaces, data**. Fault attack: lập danh sách lỗi có thể xảy ra **từ kinh nghiệm, dữ liệu defect và failure**, hiểu biết chung vì sao phần mềm hỏng | Một trục phân loại; danh sách giả thuyết lỗi |
| Defect tụ cụm, test mòn | CTFL 1.3 (FL-1.3.1) | Số ít thành phần chứa phần lớn defect; cụm dự đoán và cụm thực tế là đầu vào của risk-based testing. Test lặp lại mãi thì kém tìm defect mới | Chọn cụm, viết test mới |
| Phân tích kết quả test | CTAL-TA 5.3.1 (TA-5.3.1, K4) | So cụm defect **dự đoán** với cụm **thực tế**; khi xác định cụm, **mức độ nghiêm trọng** phải có vai trò lớn — cụm nhỏ nhưng critical quan trọng hơn cụm lớn toàn lỗi nhẹ. Thêm: DDP, coverage, test gap, mô hình xuất hiện defect | Trọng số mức độ, cụm lọt production |
| Phân loại hỗ trợ RCA | CTAL-TA 5.3.2 (TA-5.3.2, K2) | RCA từng defect quá tốn ⇒ **phân loại defect rồi làm RCA theo loại**. Phân loại biến thông tin trong defect thành số đo quá trình, nối thống kê định lượng với RCA định tính; phải **phân loại thống nhất xuyên SDLC**. Ví dụ phương pháp: ODC, IEEE 1044, theo mức độ, mô hình taxonomy (Beizer 1990, Catolino 2019); có thể ánh xạ defect vào đặc tính chất lượng ISO/IEC 25010 hay FURPS | Bộ trục phân loại, RCA theo cụm |
| Nguồn của checklist | CTAL-TA 3.4.2 | Thư viện defect và **defect taxonomy** là nguồn mục cho checklist | Đầu ra sang TEST-053 |

### 2.2. Thuật ngữ Anh – Việt

| Thuật ngữ | Tiếng Việt trong bài | Ghi chú |
|---|---|---|
| defect taxonomy | bảng phân loại defect | danh mục có cấp bậc các loại defect |
| defect classification | phân loại defect | gán thuộc tính cho từng defect (TA 5.3.2) |
| orthogonal defect classification (ODC) | phân loại defect trực giao | mỗi thuộc tính độc lập, giá trị loại trừ nhau |
| defect cluster | cụm defect | FL-1.3.1 nguyên lý 4 |
| fault attack | tấn công lỗi | CTFL 4.4.1 |
| root cause analysis (RCA) | phân tích nguyên nhân gốc | TA 5.3.2; đi sâu ở TEST-153 (Series 12) |
| defect detection percentage (DDP) | tỉ lệ phát hiện defect | TA 5.3.1 |
| fault model | mô hình lỗi | ngoài syllabus — xem 2.3 |

### 2.3. Từ taxonomy tới test — phần mở rộng

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB — ODC và mô hình lỗi</span>
<ul>
<li><b>ODC</b> (Chillarege và cộng sự, 1992 — syllabus chỉ nêu tên) có tám <i>defect type</i>: Function, Interface, Checking, Assignment, Timing/Serialization, Build/Package/Merge, Documentation, Algorithm; cùng các thuộc tính khác như trigger, impact. Bài dùng phần defect type.</li>
<li><b>Mô hình lỗi</b> (fault model) là một giả thuyết có cấu trúc về cách một loại defect sinh ra failure — ví dụ «quy tắc theo giờ dùng giờ máy thay vì UTC+7 ⇒ sai đúng 7 tiếng quanh mốc». Từ mô hình suy ra <b>luật thiết kế test</b>: kiểm ở đâu, giá trị nào, oracle là gì.</li>
<li>Kiểm chứng bộ test bằng cách <b>tái tạo defect cũ</b> và xem test có bắt được không — gần với fault seeding và mutation testing (<a href="/articles/mutation-testing">TEST-063</a>).</li>
</ul>
</div>

Quy trình bài dùng: **phân loại** (nhiều trục độc lập) → **gom cụm, xếp hạng** theo số lượng, mức độ, lọt production, tái diễn → với mỗi cụm chọn, viết **mô hình lỗi** → suy ra **test case** và **oracle** → chạy với mã hiện tại và với **bản tái tạo defect cũ** → đưa kết quả sang **RCA** và **checklist**.

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu và dữ liệu

Trưởng nhóm QA giao: «Lấy 30 defect đã đóng của bản MVP, phân loại theo taxonomy, chỉ ra ba nhóm lỗi hay tái diễn và thiết kế test mới cho chúng». Danh sách ban đầu do mọi người tự gắn nhãn: «làm tròn tiền», «giờ VN/UTC», «trạng thái chuyến», «voucher». Toàn bộ 30 defect là **giả định** của bài, đặt sao cho khớp các quy tắc SPEC lab:

| Quy tắc dễ sai | Mã SPEC | Vì sao dễ sai |
|---|---|---|
| Không làm tròn giữa chừng, làm tròn **lên** 1.000 đ ở bước cuối | R-GEN-01, R-FARE-08 | Dấu phẩy động, nhiều nơi tính tiền |
| Làm tròn 0,1 km/kg half-up **trước** khi kiểm biên | R-FARE-02, R-PARCEL-01 | 7,35 trong máy là 7,3499… |
| Giờ nghiệp vụ là UTC+7 bất kể múi giờ máy chủ | R-GEN-02, R-FARE-04, R-VOUCHER-04/07 | Máy dev ở Việt Nam che lỗi |
| Bảng chuyển trạng thái, trạng thái kết thúc | R-STATE-02/03/04 | Nhiều actor, nhiều lối ra |

### 3.2. System context

Defect nằm rải ở các module của lab (R-ARCH-01): Pricing (`fare.ts`, `parcel.ts`, `food.ts`), Promotion (`voucher.ts`), Trip lifecycle (`trip-state.ts`), Dispatch, giao diện web, cộng một báo cáo doanh thu và pipeline CI ngoài lab. Ở kiến trúc đích, cùng các loại lỗi này nhân lên theo số service: mỗi service tự so ngày, tự làm tròn, tự giữ máy trạng thái (§9.1) — và bảng rủi ro §14 (trùng giao dịch, nhận cuốc tranh chấp, event sai thứ tự) là những loại defect mà taxonomy cần có chỗ chứa.

### 3.3. Rủi ro

Sửa từng defect mà không sửa loại lỗi ⇒ cùng kiểu sai quay lại ở module khác; phân loại lẫn trục (module trộn với kiểu lỗi) ⇒ cụm cắt ngang không ai thấy; mỗi người gắn nhãn một kiểu ⇒ thống kê vô nghĩa; chỉ đếm số lượng ⇒ bỏ qua cụm ít nhưng critical.

## 4. Phân tích kỹ thuật

### 4.1. Lỗi đầu tiên: nhãn ban đầu trộn hai trục

«Voucher» là **nơi** defect nằm (module); «làm tròn», «giờ VN» là **kiểu** sai. Defect D-05 «voucher hết hạn sớm 7 tiếng» vừa là voucher vừa là giờ VN — gắn một nhãn thì mất một nửa thông tin. Theo tinh thần ODC, mỗi defect được gán **một giá trị trên mỗi trục độc lập**:

| Trục (cột CSV) | Câu hỏi | Giá trị dùng |
|---|---|---|
| `module` | Defect nằm ở đâu? | Pricing, Voucher, Trip, Dispatch, Booking, Web, Report, CI |
| `cum` | Thuộc mô hình lỗi nào? | lam-tron, gio-vn, trang-thai, bien-dau-vao, khac |
| `odc` | Loại defect kiểu ODC? | Algorithm, Checking, Assignment, Timing/Serialization, Interface, Function, Build/Package |
| `ctfl` | Nhóm theo CTFL 4.4.1? | input, output, logic, computation, interfaces, data |
| `muc_do` | Nghiêm trọng tới đâu? | critical, major, minor, trivial |
| `dua_vao` / `phat_hien` | Đưa vào ở pha nào / phát hiện ở đâu? | requirement, design, code / component, system, uat, ci, production |
| `tai_dien_cua` | Lặp lại defect nào? | mã defect trước hoặc trống |

Bốn nhãn ban đầu được giữ, nhưng đúng chỗ: «voucher» thành giá trị của `module`, ba nhãn còn lại thành giá trị của `cum`. Để phân loại nhất quán giữa người (TA 5.3.2 đòi thống nhất xuyên SDLC), mỗi giá trị có một câu định nghĩa và ví dụ trong README của lab.

### 4.2. Hai bức tranh từ cùng 30 defect

Điểm = Σ trọng số mức độ (critical 10, major 5, minor 2, trivial 1 — giả định của bài, theo ý TA-5.3.1 rằng mức độ phải nặng ký).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Hai biểu đồ cột ngang từ cùng 30 defect giả định. Bên trái theo module, số lượng: Pricing 10, Trip 5, Dispatch 4, Voucher 4, Web 3, Booking 2, CI 1, Report 1. Bên phải theo mô hình lỗi, điểm có trọng số: trang-thai 55, gio-vn 28, lam-tron 27, bien-dau-vao 11, khac 10. Cụm gio-vn trải trên 4 module nên không hiện ra ở biểu đồ bên trái.">
<text x="230" y="24" text-anchor="middle" class="t-b">Theo module — số defect</text>
<text x="690" y="24" text-anchor="middle" class="t-b">Theo mô hình lỗi — điểm có trọng số</text>
<text x="110" y="56" text-anchor="end" class="t-small">Pricing</text><rect x="120" y="44" width="250" height="16" rx="3" class="f-pk-soft s-pk"/><text x="376" y="57" class="t-mono t-small">10</text>
<text x="110" y="82" text-anchor="end" class="t-small">Trip</text><rect x="120" y="70" width="125" height="16" rx="3" class="f-pk-soft s-pk"/><text x="251" y="83" class="t-mono t-small">5</text>
<text x="110" y="108" text-anchor="end" class="t-small">Dispatch</text><rect x="120" y="96" width="100" height="16" rx="3" class="f-pk-soft s-pk"/><text x="226" y="109" class="t-mono t-small">4</text>
<text x="110" y="134" text-anchor="end" class="t-small">Voucher</text><rect x="120" y="122" width="100" height="16" rx="3" class="f-pk-soft s-pk"/><text x="226" y="135" class="t-mono t-small">4</text>
<text x="110" y="160" text-anchor="end" class="t-small">Web</text><rect x="120" y="148" width="75" height="16" rx="3" class="f-pk-soft s-pk"/><text x="201" y="161" class="t-mono t-small">3</text>
<text x="110" y="186" text-anchor="end" class="t-small">Booking</text><rect x="120" y="174" width="50" height="16" rx="3" class="f-pk-soft s-pk"/><text x="176" y="187" class="t-mono t-small">2</text>
<text x="110" y="212" text-anchor="end" class="t-small">CI</text><rect x="120" y="200" width="25" height="16" rx="3" class="f-pk-soft s-pk"/><text x="151" y="213" class="t-mono t-small">1</text>
<text x="110" y="238" text-anchor="end" class="t-small">Report</text><rect x="120" y="226" width="25" height="16" rx="3" class="f-pk-soft s-pk"/><text x="151" y="239" class="t-mono t-small">1</text>
<line x1="120" y1="38" x2="120" y2="248" class="s-muted"/>
<line x1="460" y1="36" x2="460" y2="250" class="s-line dash"/>
<text x="580" y="56" text-anchor="end" class="t-small">trang-thai</text><rect x="590" y="44" width="220" height="16" rx="3" class="f-warn-soft s-warn"/><text x="816" y="57" class="t-mono t-small t-warn">55</text>
<text x="580" y="82" text-anchor="end" class="t-small">gio-vn</text><rect x="590" y="70" width="112" height="16" rx="3" class="f-warn-soft s-warn"/><text x="708" y="83" class="t-mono t-small t-warn">28</text>
<text x="580" y="108" text-anchor="end" class="t-small">lam-tron</text><rect x="590" y="96" width="108" height="16" rx="3" class="f-warn-soft s-warn"/><text x="704" y="109" class="t-mono t-small t-warn">27</text>
<text x="580" y="134" text-anchor="end" class="t-small">bien-dau-vao</text><rect x="590" y="122" width="44" height="16" rx="3" class="f-phys-soft s-phys"/><text x="640" y="135" class="t-mono t-small">11</text>
<text x="580" y="160" text-anchor="end" class="t-small">khac</text><rect x="590" y="148" width="40" height="16" rx="3" class="f-phys-soft s-phys"/><text x="636" y="161" class="t-mono t-small">10</text>
<line x1="590" y1="38" x2="590" y2="170" class="s-muted"/>
<text x="490" y="196" class="t-small">trang-thai: 4/4 defect critical · 3 lần tái diễn</text>
<text x="490" y="216" class="t-small">gio-vn: 4/7 lọt production · trải 4 module</text>
<text x="490" y="236" class="t-small">lam-tron: 8 defect · 3 lần tái diễn</text>
<text x="460" y="284" text-anchor="middle" class="t-small">Bên trái gợi ý «dồn sức cho Pricing»; bên phải cho thấy cụm giờ VN cắt ngang Pricing, Voucher, Web, CI</text>
</svg>
</div>
<figcaption><b>Hình 1 · Cùng dữ liệu, hai góc phân loại.</b> Số liệu giả định, in ra từ <code>phan-loai.test.ts</code>. Góc module là góc của nguyên lý «defect tụ cụm» ở <a href="/articles/bay-nguyen-ly-kiem-thu">TEST-003</a>; góc mô hình lỗi mới chỉ ra <i>cần viết loại test nào</i>.</figcaption>
</figure>

Ba quan sát từ bảng in ra (mục 5.2):

1. **Đếm số lượng** thì Pricing đứng đầu (10). **Tính trọng số** thì Trip vượt lên (40 điểm so với 38) vì chứa 3 defect critical — đúng cảnh báo của TA-5.3.1.
2. **gio-vn** chỉ có 7 defect, nhưng 4 lọt production (cao nhất) và trải trên 4 module. Lọt nhiều nghĩa là bộ test hiện tại **không có khả năng** bắt loại lỗi này: chúng chạy trên máy đặt giờ Việt Nam.
3. **bien-dau-vao** 4 defect, 0 lọt production, 0 tái diễn — test biên hiện có ([TEST-032](/articles/boundary-value-analysis)) đang làm tốt. Không cần đầu tư thêm.

### 4.3. Chọn ba cụm và viết mô hình lỗi

| Cụm | Lý do chọn | Mô hình lỗi (giả thuyết) | Luật thiết kế test | Oracle |
|---|---|---|---|---|
| **FM-R làm tròn** | 8 defect, 3 tái diễn (D-06 → D-09, D-01 → D-22, D-01 → D-17) | (R1) làm tròn đầu vào sai ở x,x5; (R2) làm tròn giữa chừng; (R3) làm tròn cuối sai chiều | Quét **toàn bộ** miền: mọi x,x5 hợp lệ; mọi quãng đường 0,5…100 km × loại xe × cao điểm × mưa × voucher | Cài đặt **độc lập** bằng BigInt, đơn vị 1/4000 đồng — không dấu phẩy động |
| **FM-T giờ VN** | 4/7 lọt production, chạy đúng trên máy dev | (T1) dùng giờ máy thay UTC+7; (T2) sai đầu mút [a, b); (T3) ranh giới ngày VN = 17:00 UTC | Mỗi mốc giờ của quy tắc ±1 giây, ở nhiều ngày (đầu năm, giữa năm, cuối năm); mốc 00:00, 06:59:59, 07:00 VN | Giá trị tính tay từ SPEC; thời điểm dựng bằng `vnTime` (không phụ thuộc múi giờ máy) |
| **FM-S trạng thái** | 4 defect critical, 3 tái diễn | (S1) chuyển sai được chấp nhận; (S2) chuyển đúng tới sai trạng thái; (S3) trạng thái kết thúc «sống lại»; (S4) rời trạng thái mà không dọn dữ liệu | **Đủ 72 cặp** (8 trạng thái × 9 sự kiện); mọi lối ra khỏi `accepted` | Bảng R-STATE-02 chép tay từ SPEC (không import từ `src`) |

Hai điều cần thấy ở bảng: luật thiết kế đến từ **mô hình lỗi**, không từ đặc tả — đặc tả chỉ nói «cao điểm là [11:00; 13:00)», còn mô hình T1 bảo phải thử ở **nhiều ngày** và quanh **07:00 VN** (nửa đêm UTC), chỗ không ai thử nếu chỉ đọc R-FARE-04. Và oracle phải **độc lập**: so `calculateFare` với chính công thức của nó thì lỗi R2 không bao giờ lộ.

Ba defect critical D-03, D-14, D-28 thuộc cụm trạng thái nhưng là lỗi **đồng thời** (Timing/Serialization). Mô hình S1–S4 kiểm từng thao tác tuần tự nên không bắt được — chúng cần kiểm thử đồng thời ở Series 08 ([TEST-085](/articles/concurrent-testing-hai-tai-xe-cung-nhan), [TEST-086](/articles/race-condition-testing)). Đây là lý do trục ODC có giá trị: trục này tách một cụm «trạng thái» thành hai mô hình lỗi cần hai cách test khác nhau.

## 5. Thực hành

Tệp ở `examples/toi-noi-testing/articles/defect-based-testing/` ([lab trên GitHub](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing)): `lich-su-defect.csv` (30 defect, mở bằng bảng tính), `phan-loai.ts`, `phan-loai.test.ts`, `mo-hinh-loi.test.ts`, `ban-loi.ts`.

### 5.1. Trích lịch sử defect

| ID | Tóm tắt | module | cum | ODC | mức độ | phát hiện | tái diễn của |
|---|---|---|---|---|---|---|---|
| D-01 | 7,35 km ra 45.000 đ (toFixed làm tròn 7,3) | Pricing | lam-tron | Algorithm | major | system | |
| D-02 | Cao điểm lấy giờ máy chủ UTC: 18:30 VN không nhân 1,25 | Pricing | gio-vn | Algorithm | major | production | |
| D-05 | Voucher hết hạn sớm 7 tiếng (ngày UTC) | Voucher | gio-vn | Checking | major | production | |
| D-06 | Tiền giảm làm tròn tới 1.000 đ trước khi trừ (đặt xe) | Voucher | lam-tron | Algorithm | minor | uat | |
| D-08 | Khách huỷ được chuyến đã `picked_up` | Trip | trang-thai | Checking | major | system | |
| D-09 | Tiền giảm bưu kiện vẫn làm tròn (bản sửa D-06 chỉ vá nhánh đặt xe) | Pricing | lam-tron | Algorithm | major | production | D-06 |
| D-14 | Hai tài xế cùng nhận thành công một chuyến | Dispatch | trang-thai | Timing/Serialization | critical | production | |
| D-18 | Chuyến `completed` về `searching` khi bấm «Huỷ nhận» muộn | Trip | trang-thai | Checking | critical | production | D-08 |
| D-29 | Test cao điểm xanh trên máy dev, đỏ trên CI chạy UTC | CI | gio-vn | Build/Package | trivial | ci | D-02 |

### 5.2. Phân loại

```bash
npx vitest run articles/defect-based-testing/phan-loai.test.ts
```

```text
== Theo mô hình lỗi (cum, sắp theo điểm) ==
nhóm                  SL  điểm  crit  prod  tái_diễn  module
trang-thai             7    55     4     2         3  Dispatch, Trip
gio-vn                 7    28     0     4         2  CI, Pricing, Voucher, Web
lam-tron               8    27     0     2         3  Pricing, Report, Voucher
bien-dau-vao           4    11     0     0         0  Booking, Dispatch, Pricing
khac                   4    10     0     1         0  Booking, Dispatch, Web
== Theo nhóm CTFL 4.4.1 ==
logic                 10    66     4     4         3  CI, Dispatch, Pricing, Trip, Voucher
computation            9    29     0     2         3  Booking, Pricing, Report, Voucher
data                   4    20     0     2         2  Dispatch, Trip, Voucher
input                  5    12     0     0         0  Booking, Dispatch, Pricing, Web
output                 2     4     0     1         0  Web
```

(Bảng theo module và theo ODC cũng được in; Node 22.18, vitest 3.2.7.) Trục CTFL cho câu trả lời thô hơn: «logic» gom cả trạng thái lẫn giờ VN — đủ để nói «lỗi logic nhiều nhất», chưa đủ để biết **viết test gì**. Đó là lý do bài thêm trục `cum` mịn hơn.

### 5.3. Test cho ba cụm — và bằng chứng chúng bắt được lỗi cũ

Mỗi mô hình là một hàm `check…(impl)` trả về danh sách ca sai. Oracle làm tròn, độc lập với `src/`:

```ts
// đơn vị 1/4000 đồng: ×1,25 = 5/4, percent/100 và km/10 đều chia hết ⇒ số học chính xác tuyệt đối
const base = open * U + perKm * BigInt(tenths) * 400n;
const afterPeak = peak ? (base * 5n) / 4n : base;
const before = afterPeak + (rain ? 8_000n * U : 0n);
// … tiền giảm min(percent% × before, trần), nâng lên cước tối thiểu, rồi làm tròn LÊN 1.000 đ
```

Mô hình giờ VN dựng thời điểm bằng `vnTime(y, m, d, h, mi, s)` và kiểm cả hai phía mỗi mốc; với voucher hiệu lực đúng một ngày, test case «ngoài» là 1 giây trước 00:00:00 và 1 giây sau 23:59:59 giờ VN.

```bash
npx vitest run articles/defect-based-testing --reporter=verbose
```

```text
src: 0 ca sai                                   (FM-R: 996 giá trị x,x5 + 31.872 tổ hợp cước)
D-01 toFixed: 398 ca sai — vd R1 0.85 km ⇒ 0.8 | R1 0.95 km ⇒ 0.9
D-04 làm tròn gần nhất: 13366 ca sai — vd R2 bike 0.5 km mưa: 22000 ≠ 23000 | …
D-06/D-09 làm tròn tiền giảm: 1078 ca sai — vd R2 bike 0.5 km mưa −20%: 19000 ≠ 18000 | …
làm tròn về đồng sau ×1,25: 0 ca sai
src: 0 ca sai                                   (FM-T: 48 mốc cao điểm + 24 mốc ngày voucher)
D-02 giờ máy chủ UTC: 20 ca sai — vd T-peak 1/1 11:00:00 VN ⇒ false | …
D-24 đầu mút <=: 8 ca sai — vd T-peak 1/1 13:00:00 VN ⇒ true | T-peak 1/1 19:00:00 VN ⇒ true
D-05 ngày theo toISOString: 12 ca sai — vd T-day 2025-12-31T17:00:00.000Z trong ngày VN 2026-01-01 bị coi là hết hạn | …
src: 0 ca sai                                   (FM-S: 72 cặp trạng thái × sự kiện)
D-08 huỷ sau khi đón: 2 ca sai — vd picked_up + customer_cancel ⇒ cancelled (đặc tả: INVALID) | …
D-18 huỷ nhận sau hoàn thành: 1 ca sai — vd completed + driver_cancel ⇒ searching (đặc tả: INVALID)

 Test Files  2 passed (2)
      Tests  17 passed (17)
```

(Chú thích trong ngoặc là của bài. Thêm hai ca use case S4: tài xế huỷ hoặc khách huỷ chuyến `accepted` ⇒ không còn `acceptedAt`/ETA cũ, tài xế nhận được chuyến kế tiếp — đều đạt trên `src`.)

| Kết quả | Ý nghĩa |
|---|---|
| `src` 0 ca sai ở cả ba mô hình | Mã lab hiện tại sạch với ba loại lỗi — giờ đây có bằng chứng, không chỉ là «đã sửa» |
| 8 bản tái tạo defect cũ đều bị bắt | Bộ test nhắm **đúng loại** lỗi: lần sau D-09 quay lại ở nhánh nào cũng đỏ |
| D-02 bị bắt dù máy chạy test đặt giờ Việt Nam | Test case không dựa vào múi giờ máy — sửa luôn nguyên nhân của D-29 |
| Biến thể «làm tròn về đồng sau ×1,25» 0 ca sai | Xem 5.4 |

### 5.4. Một biến thể không bắt được — và vì sao đó là tin tốt

Lúc soạn bài, D-09 ban đầu được mô tả là «làm tròn về đồng ngay sau ×1,25 (17.812,5 ⇒ 17.813)». Tái tạo đúng mô tả đó, bộ FM-R cho **0 ca sai** trên 31.872 tổ hợp. Không phải test yếu: sau ×1,25 phần lẻ chỉ có thể là 0,5 đồng, và bước cuối luôn làm tròn **lên** 1.000 đ, nên làm tròn nửa đồng không bao giờ đổi tổng — một **biến thể tương đương** (thuật ngữ của mutation testing, [TEST-063](/articles/mutation-testing)). Kết luận ngược lại mới quan trọng: mô tả cũ **không thể** là nguyên nhân của failure «lệch 1.000 đ» trên production. Đội quay lại hồ sơ và thấy nguyên nhân thật là tiền giảm bị làm tròn (đã sửa lại D-09 trong CSV). Phân loại sai nguyên nhân thì RCA sai — đúng điều TA-5.3.2 cảnh báo.

### 5.5. Từ cụm tới RCA và phòng ngừa (TA-5.3.2)

Phân loại cho phép làm RCA **một lần cho cả cụm** thay vì 30 lần:

| Cụm | Nguyên nhân gốc chung (giả định sau buổi RCA) | Biện pháp ở code / test / quy trình |
|---|---|---|
| gio-vn | Không có quy ước «giờ nghiệp vụ đi qua một module»; máy dev và CI khác múi giờ | Mọi quy tắc theo giờ chỉ gọi `src/domain/time.ts`; luật static analysis cấm `getHours`/`toISOString().slice` trong `domain/` ([TEST-029](/articles/static-analysis-phan-tich-code-tinh)); CI chạy test hai lần với `TZ=UTC` và `TZ=Asia/Ho_Chi_Minh`; FM-T vào bộ hồi quy |
| lam-tron | Logic tiền chép ở nhiều nhánh; bản sửa chỉ vá một chỗ | Một module tiền duy nhất (`money.ts`); FM-R chạy ở mọi thay đổi Pricing; mục review «tiền giảm có làm tròn không?» cho checklist review ([TEST-028](/articles/checklist-based-review)) |
| trang-thai | Kiểm điều kiện rải rác trong từng use case | Máy trạng thái khai báo một bảng; test đủ 72 cặp; tách lỗi đồng thời sang kiểm thử đồng thời |

Kết quả còn quay về kỹ thuật kinh nghiệm: mục SM-18 «giờ hiển thị rõ là giờ Việt Nam?» trong checklist smoke của [TEST-053](/articles/checklist-based-testing) và các fault attack về thời gian ở [TEST-051](/articles/error-guessing) đều rút từ cụm gio-vn.

## 6. Góc nhìn nâng cao

**Chất lượng dữ liệu quyết định tất cả.** 30 defect là mẫu nhỏ; hai người gắn nhãn khác nhau là thống kê vô nghĩa. Cách giảm: định nghĩa từng giá trị kèm ví dụ, gắn nhãn hai lần (lúc báo và lúc sửa — ODC làm vậy), đo mức đồng thuận giữa người phân loại trên một mẫu.

**Cụm dự đoán và cụm thực tế (TA-5.3.1).** Trước khi đếm, nhóm dự đoán Pricing và Payment là vùng nhiều lỗi ([TEST-003](/articles/bay-nguyen-ly-kiem-thu)). Thực tế Payment không có defect nào trong mẫu — hoặc Payment tốt, hoặc test chưa chạm tới Payment (lab chưa có webhook, đối soát). Chênh lệch dự đoán – thực tế là câu hỏi cần trả lời, không phải tin tốt mặc nhiên.

**Giới hạn.**

- Defect-based testing nhìn **ra sau**: loại lỗi chưa từng xảy ra không có trong lịch sử. Bù bằng taxonomy chung (Beizer, ODC) và các kỹ thuật dựa trên đặc tả.
- Bộ test cụm cũng **mòn** (FL-1.3.1): khi developer đã quen dùng `time.ts`, FM-T ít bắt lỗi mới — chi phí thấp nên vẫn giữ làm hồi quy, nhưng đừng coi nó là tìm lỗi.
- Một mô hình lỗi chỉ phủ những gì nó giả thuyết. FM-S không bắt được D-14 — cần một mô hình khác.
- Biến thể tương đương làm «không bắt được» trở nên mơ hồ: phải chứng minh nó tương đương trước khi kết luận test yếu.

**So với các kỹ thuật gần.** Error guessing ([TEST-051](/articles/error-guessing)) dùng kinh nghiệm cá nhân, có thể chưa có dữ liệu; defect-based dùng **dữ liệu có hệ thống** của dự án. Checklist ([TEST-053](/articles/checklist-based-testing)) là một **đầu ra** của phân loại. Mutation testing ([TEST-063](/articles/mutation-testing)) sinh biến thể máy móc từ toán tử; ở đây biến thể được **chọn** từ lỗi thật đã xảy ra. RCA đầy đủ (5 Whys, Ishikawa, thống kê cải tiến quy trình) là TEST-153 ở Series 12.

**Sai lầm thường gặp:**

- Một trục duy nhất trộn «ở đâu» và «sai kiểu gì».
- Chỉ đếm, không trọng số mức độ và lọt production.
- Viết test cho **defect** (một test case tái hiện) thay vì cho **loại defect** (cả miền). Test tái hiện D-09 chỉ thử bưu kiện; FM-R thử mọi nhánh.
- Oracle mượn từ chính code đang kiểm.
- Phân loại xong để đó, không làm RCA, không cập nhật checklist.

## 7. Bài tập và câu hỏi ôn tập

### Bài tập 1 — Phân loại 4 defect mới (K2)

Gán `module`, `cum`, `odc`, `ctfl`, mức độ: (a) Khách C002 dùng `FOOD15` lần thứ ba trong ngày vẫn được (R-VOUCHER-07: 2 lần/ngày). (b) Chuyến ô tô huỷ có phí thu 10.000 đ thay vì 20.000 đ. (c) Lượt voucher của khách huỷ chuyến lúc 23:30 VN không được trả lại cho ngày hôm đó. (d) Tài xế D103 hai tab cùng nhận hai chuyến khác nhau, cả hai thành công.

<details><summary><b>Lời giải mẫu</b></summary><ul><li>(a) Voucher · khac (hoặc bien-dau-vao: so sánh <code>&lt;</code> với <code>&lt;=</code>) · Checking · logic · major.</li><li>(b) Trip · khac · Assignment (sai hằng theo loại xe) · computation · major.</li><li>(c) Voucher · <b>gio-vn</b> (23:30 VN là 16:30 UTC cùng ngày — nhưng nếu đếm theo ngày UTC sẽ sai từ 07:00 sáng; cần điều tra) · Checking · data · major. Một defect nữa cho cụm giờ VN ⇒ thêm ca «trả lượt quanh 17:00 UTC» vào FM-T.</li><li>(d) Dispatch · trang-thai · <b>Timing/Serialization</b> · logic · critical — thuộc phần đồng thời, FM-S tuần tự không bắt được (R-ACCEPT-03).</li></ul></details>

### Bài tập 2 — Mô hình lỗi cho cụm mới (K3)

Giả sử 6 tháng sau có cụm «trùng lặp»: thu tiền hai lần, tạo chuyến hai lần, lượt voucher trừ hai lần. Viết mô hình lỗi và ba luật thiết kế test.

<details><summary><b>Lời giải mẫu</b></summary><p><b>Mô hình:</b> (U1) thao tác có tác dụng phụ không có khoá idempotency hoặc khoá đổi giữa các lần thử; (U2) kiểm-rồi-ghi không nguyên tử; (U3) kết quả không rõ (hết giờ) bị coi là thất bại rồi làm lại. <b>Luật:</b> (1) mọi endpoint có tác dụng phụ: gửi lại cùng khoá, cùng thân ⇒ đúng một tác dụng (R-IDEM-02); (2) mỗi lần gọi bên thứ ba: ép hết giờ ở mọi lần thử, kiểm số tiền ví đã trừ (R-PAY-04, ví giả <code>lost_response</code>); (3) hai yêu cầu đồng thời cho cùng tài nguyên ⇒ một thắng (R-QUOTE-03, R-ACCEPT-03) — cần môi trường đồng thời thật. Đi sâu ở TEST-134 (Series 11).</p></details>

### Bài tập 3 — Phân tích (K4)

Một vòng test mới có 12 defect: 9 minor ở Web, 3 critical ở Payment. Trưởng nhóm muốn dồn sức cho Web «vì nhiều lỗi nhất». Phân tích và đề xuất.

<details><summary><b>Lời giải mẫu</b></summary><p>Theo số lượng Web chiếm 75%; theo trọng số bài (minor 2, critical 10) Web 18 điểm, Payment 30 điểm. TA-5.3.1: cụm nhỏ nhưng critical cần kiểm nghiêm ngặt hơn cụm lớn lỗi nhẹ. Đề xuất: phân loại 3 defect Payment theo ODC — nếu là Timing/Serialization (trùng giao dịch) thì cần kiểm thử đồng thời và hồi quy idempotency; Web xử lý bằng checklist do-confirm và một luật static analysis nếu các lỗi minor cùng một kiểu.</p></details>

### Câu hỏi trắc nghiệm

**Câu 1 (K2).** Theo CTAL-TA v4.0, vì sao phân loại defect hỗ trợ RCA?

- a) Vì phân loại thay được RCA
- b) Vì RCA từng defect quá tốn; phân loại cho phép làm RCA theo loại defect và biến thông tin defect thành số đo quá trình
- c) Vì phân loại chỉ cần làm ở production
- d) Vì phân loại xác định người gây lỗi

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — phân loại nối thống kê với RCA, không thay RCA.</li><li>b) Đúng — mục 5.3.2.</li><li>c) Sai — syllabus đòi phân loại thống nhất xuyên SDLC, từ test sớm tới production.</li><li>d) Sai — mục tiêu là cải tiến quá trình, không quy lỗi cá nhân.</li></ul></details>

**Câu 2 (K2).** Nhóm nào **không** nằm trong các nhóm lỗi mà CTFL v4.0.1 mục 4.4.1 nêu?

- a) Input
- b) Computation
- c) Interfaces
- d) Hardware

<details><summary><b>Đáp án: d</b></summary><ul><li>a, b, c) Sai — đều có trong danh sách input, output, logic, computation, interfaces, data.</li><li>d) Đúng — không có trong danh sách của mục 4.4.1.</li></ul></details>

**Câu 3 (K2).** Phương pháp nào là ví dụ phân loại defect mà CTAL-TA v4.0 nêu?

- a) Orthogonal defect classification (ODC)
- b) Equivalence partitioning
- c) Pairwise testing
- d) Test pyramid

<details><summary><b>Đáp án: a</b></summary><ul><li>a) Đúng — cùng IEEE 1044, phân loại theo mức độ, mô hình taxonomy.</li><li>b, c) Sai — kỹ thuật thiết kế test hộp đen.</li><li>d) Sai — mô hình phân bổ test (CTFL 5.1.6).</li></ul></details>

**Câu 4 (K4).** Sau vòng test: cụm A có 12 defect minor, cụm B có 3 defect critical, cụm C có 5 defect major nhưng 4 lọt production. Thứ tự đầu tư test thêm nào hợp lý nhất theo tinh thần TA-5.3.1?

- a) A → C → B, theo số lượng
- b) B và C trước, A sau
- c) Chia đều cho ba cụm
- d) Chỉ C, vì lọt production là duy nhất quan trọng

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — bỏ qua mức độ, trái ý «cụm nhỏ nhưng critical quan trọng hơn».</li><li>b) Đúng — B nghiêm trọng; C cho thấy test hiện tại không bắt được (DDP thấp ở cấp test trước).</li><li>c) Sai — dữ liệu cho phép tập trung, chia đều là bỏ phí.</li><li>d) Sai — bỏ qua B có 3 critical.</li></ul></details>

**Câu 5 (K3).** Cụm «giờ VN» có lỗi dùng giờ máy chủ. Bộ test case nào nhắm đúng mô hình lỗi này nhất?

- a) Một test case lúc 12:00 giờ VN chạy trên máy dev
- b) Các mốc 10:59:59, 11:00:00, 12:59:59, 13:00:00 giờ VN dựng từ UTC, ở nhiều ngày, không phụ thuộc múi giờ máy chạy test
- c) Một test case với giờ ngẫu nhiên
- d) Chạy lại bộ test hiện có nhiều lần

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — máy dev ở Việt Nam che đúng lỗi này (D-29).</li><li>b) Đúng — mỗi mốc ±1 giây, dựng tường minh theo UTC+7.</li><li>c) Sai — khó trúng mốc, không lặp lại được.</li><li>d) Sai — test lặp lại mãi sẽ mòn; chạy lại không thêm khả năng phát hiện.</li></ul></details>

**Câu 6 (K2).** Bộ test thiết kế từ mô hình lỗi không bắt được một biến thể tái tạo defect cũ. Kết luận nào cần kiểm trước?

- a) Bộ test chắc chắn yếu
- b) Biến thể có thể tương đương — không đổi kết quả nào trong miền hợp lệ — và mô tả defect cũ có thể sai nguyên nhân
- c) Nên xoá bộ test
- d) Nên thêm nhiều ca ngẫu nhiên

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — chưa loại trừ biến thể tương đương.</li><li>b) Đúng — như mục 5.4.</li><li>c) Sai — không có cơ sở.</li><li>d) Sai — nếu biến thể tương đương thì không ca nào bắt được.</li></ul></details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Advanced Level Test Analyst Syllabus* v4.0: mục 5.3, 5.3.1 (TA-5.3.1), 5.3.2 (TA-5.3.2), 3.4.2 — https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB, *Certified Tester Foundation Level Syllabus* v4.0.1 (2024): mục 1.3 (FL-1.3.1), 4.4.1 (FL-4.4.1) — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (mục *defect taxonomy*, *root cause analysis*)
- Chillarege, R. và cộng sự (1992), «Orthogonal Defect Classification — A Concept for In-Process Measurements», *IEEE Transactions on Software Engineering* — CTAL-TA dẫn.
- IEEE Std 1044-2009, *IEEE Standard Classification for Software Anomalies* — CTAL-TA dẫn.
- Beizer, B. (1990), *Software Testing Techniques*, 2nd ed. — CTAL-TA dẫn làm mô hình taxonomy; Catolino và cộng sự (2019) — CTAL-TA dẫn.
- ISO/IEC 25010:2023 — ánh xạ defect vào đặc tính chất lượng (CTAL-TA 5.3.2 nêu).
- Lab Tới Nơi: `SPEC.md` (R-GEN-01/02, R-FARE, R-VOUCHER-04/07, R-STATE), `articles/defect-based-testing/` — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TEST-004](/articles/error-defect-failure-root-cause) · [TEST-003](/articles/bay-nguyen-ly-kiem-thu) · [TEST-051](/articles/error-guessing) · [TEST-053](/articles/checklist-based-testing) · [TEST-019](/articles/confirmation-testing-va-regression-testing) · [TEST-063](/articles/mutation-testing) · TEST-153 Defect Root Cause Analysis (Series 12, sắp ra mắt) · [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi)

Bài tiếp theo: [TEST-055 · Unit Testing Fundamentals](/articles/unit-testing-fundamentals) — mở đầu Series 06 «Unit Testing & Component Testing».
