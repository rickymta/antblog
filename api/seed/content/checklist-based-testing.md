---
slug: checklist-based-testing
title: "Checklist-based Testing: checklist smoke 20 mục cho bốn luồng quan trọng của Tới Nơi"
summary: "Checklist-based testing là kỹ thuật dựa trên kinh nghiệm dùng checklist khi chạy phần mềm. Bài soạn checklist smoke 20 mục cho đặt xe, bưu kiện, đồ ăn, huỷ có phí, chạy thật trên web UI lab và cập nhật lên v1.1."
category: testing
level: advanced
tags: [testing, istqb, ctfl, ctal-ta, test-053, experience-based-testing, checklist, smoke-test, playwright, toi-noi]
language: vi
---

Ngày mai Tới Nơi phát hành bản `rc-2026.11.2`. Bộ test tự động đã xanh, nhưng trưởng nhóm muốn một người dùng giao diện thật đi qua bốn luồng kiếm ra tiền — đặt xe, bưu kiện, đồ ăn, huỷ có phí — trong 30 phút. Không có thời gian viết test case chi tiết. Câu hỏi của bài: dùng một **checklist** thế nào để 30 phút đó có hệ thống, lặp lại được giữa các lần phát hành, và không mục nào trở thành thói quen tick cho xong?

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-053</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>S05 · White-box &amp; Experience-based Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation — CTFL v4.0.1; Advanced — CTAL-TA v4.0</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Advanced</td></tr>
<tr><th scope="row">Test Level</th><td>System</td></tr>
<tr><th scope="row">Test Type</th><td>Black-box (chức năng và vài đặc tính phi chức năng)</td></tr>
<tr><th scope="row">Test Technique</th><td>Checklist-based Testing</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/checklist-based-review">TEST-028 Checklist-based Review</a>, <a href="/articles/error-guessing">TEST-051 Error Guessing</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Checklist CL-SMOKE 20 mục chạy smoke 4 luồng trên web UI lab trước bản phát hành giả định — lab <code>articles/checklist-based-testing</code> (checklist + spec Playwright ghi bằng chứng)</td></tr>
<tr><th scope="row">Tham chiếu ISTQB</th><td>CTFL v4.0.1 · Chương 4 · mục 4.4.3 · FL-4.4.3 (K2). CTAL-TA v4.0 · Chương 3 · mục 3.4.2 · TA-3.4.2 (K3)</td></tr>
</tbody>
</table>
</div>

## 1. Giới thiệu

**Checklist-based testing** (kiểm thử theo checklist) là kỹ thuật kiểm thử **dựa trên kinh nghiệm**: tester thiết kế, thực hiện và chạy test để phủ các **test condition** ghi trong một checklist. Mỗi mục là một câu hỏi ngắn: «Huỷ sau 3 phút có thu đúng 10.000 đ không?». Tester tự quyết thao tác chi tiết, checklist chỉ bảo phải nhìn vào đâu.

Kỹ thuật giải quyết khoảng giữa hai thái cực: test case viết sẵn từng bước (chắc chắn, nhưng tốn công viết và bảo trì) và khám phá tự do (nhanh, nhưng mỗi người một kiểu, dễ bỏ sót cái quan trọng). Dùng khi:

- Không có (hoặc không kịp có) test case chi tiết, nhưng cần **độ nhất quán** giữa các lần chạy và giữa các tester.
- Có một tập ý **lặp lại** mỗi lần phát hành: smoke trước release, kiểm thử sau khi deploy, kiểm thử một màn hình mới.
- Muốn đóng gói **kinh nghiệm** — lỗi từng gặp, điều khách hay phàn nàn — thành thứ người mới dùng được.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Khác TEST-028</span>
<p><a href="/articles/checklist-based-review">TEST-028</a> dùng checklist để <b>đọc</b> đặc tả và code — kiểm thử tĩnh, không chạy gì. Bài này dùng checklist khi <b>chạy</b> phần mềm — kiểm thử động. Cùng một ý «khách bị trừ tiền», bản review hỏi «đặc tả có định nghĩa không?», bản kiểm thử hỏi «màn hình có cho khách thấy không?».</p>
</div>

## 2. Lý thuyết theo ISTQB

### 2.1. CTFL v4.0.1 mục 4.4.3 — bảy ý cần nhớ

CTFL xếp checklist-based testing cùng error guessing (4.4.1) và exploratory testing (4.4.2) trong nhóm **experience-based test techniques**. Diễn đạt lại nội dung mục 4.4.3:

| # | Ý của syllabus | Hệ quả khi soạn checklist |
|---|---|---|
| 1 | Checklist được xây từ kinh nghiệm, từ hiểu biết điều gì quan trọng với người dùng, từ hiểu biết phần mềm hỏng thế nào, vì sao | Nguồn mục: lịch sử defect, phàn nàn của khách, rủi ro nghiệp vụ |
| 2 | **Không** chứa mục máy kiểm được, mục hợp làm tiêu chí vào/ra, mục quá chung | Lọc trước khi đưa vào (Hình 1) |
| 3 | Mục thường viết dạng **câu hỏi**; mỗi mục kiểm được **riêng lẻ và trực tiếp** | Một ý mỗi mục, có kết quả quan sát được |
| 4 | Mục có thể chỉ yêu cầu, thuộc tính giao diện, đặc tính chất lượng hay test condition khác; dùng cho cả kiểm thử chức năng và phi chức năng | Ví dụ syllabus nêu: 10 heuristic usability của Nielsen |
| 5 | Một số mục **kém hiệu quả dần** vì developer học được cách tránh lỗi đó; cần thêm mục cho defect nghiêm trọng mới | Cập nhật định kỳ **dựa trên phân tích defect** |
| 6 | Đừng để checklist **quá dài** | Giới hạn theo thời lượng phiên |
| 7 | Không có test case chi tiết thì checklist cho **hướng dẫn và một mức nhất quán**; checklist càng chung thì kết quả càng biến thiên — phủ có thể rộng hơn, nhưng **khó lặp lại** hơn | Chọn độ chi tiết theo mục đích |

Ý 7 là đánh đổi cốt lõi: mục «Báo giá xe máy 7,4 km có ra 46.000 đ?» lặp lại hoàn hảo nhưng hẹp; mục «Thông báo lỗi có dễ hiểu?» mỗi người chạy một kiểu, phủ rộng hơn nhưng hai lần chạy khó so sánh.

### 2.2. CTAL-TA v4.0 mục 3.4.2 — soạn checklist ở mức áp dụng (K3)

TA-3.4.2 yêu cầu **chuẩn bị** checklist hỗ trợ experience-based testing. Syllabus nêu lợi ích (giữ được các khía cạnh thiết yếu, nhất quán giữa các vòng test và giữa các tester, là kho ghi lại kinh nghiệm về failure và defect, gợi ý khi tester cạn ý — kể cả lúc khám phá, giảm công viết test case khi yêu cầu đổi liên tục) và một quy trình ba bước, diễn đạt lại:

1. **Xác định phạm vi, mục tiêu, định dạng** — vì chúng quyết định độ sâu. Hai định dạng:
   - **read-do checklist**: liệt kê các yếu tố chính phải làm cho một quy trình, mục có thể ghi sẵn giá trị cụ thể (ví dụ các đầu vào không hợp lệ cho một ô nhập). Đọc mục nào làm mục đó.
   - **do-confirm checklist**: hỗ trợ dòng suy nghĩ, gợi ý ý tưởng để khám phá thêm (ví dụ «kết quả tìm kiếm có liên quan không?»). Làm trước theo kinh nghiệm, rồi dùng checklist để xác nhận.
2. **Thu thập thông tin** cho các mục: hỏi người có kinh nghiệm, xem thư viện và **taxonomy defect**, đọc tài liệu, phân tích rủi ro, test case, kịch bản. Mục phải rõ, cụ thể, không mơ hồ, nhất quán, liên quan, bảo trì được, hành động được, đo được; viết thành câu hỏi trả lời **có / không / không áp dụng**; có **độ ưu tiên** theo tầm quan trọng, tác động, mức rủi ro. Checklist không phải hướng dẫn từng bước — nó là công cụ nhanh cho người đã thạo.
3. **Cấu trúc**: nhóm mục theo vùng chức năng, vai người dùng, test level… Dùng mẫu, chuẩn có sẵn khi được.

Và một câu đáng thuộc: checklist **không bao giờ hoàn tất** — luôn được xem lại theo phát hiện mới, ưu tiên mới, góp ý của tester khác, bài học từ vòng trước; chia sẻ checklist giúp cả nhóm nhất quán.

### 2.3. Thuật ngữ Anh – Việt

| Thuật ngữ | Tiếng Việt trong bài | Ghi chú |
|---|---|---|
| checklist-based testing | kiểm thử theo checklist | CTFL 4.4.3 — kiểm thử động |
| experience-based test technique | kỹ thuật kiểm thử dựa trên kinh nghiệm | nhóm cùng error guessing, exploratory |
| test condition | điều kiện kiểm thử | mỗi mục checklist là một test condition |
| read-do / do-confirm checklist | checklist đọc-làm / làm-xác nhận | CTAL-TA 3.4.2 |
| entry / exit criteria | tiêu chí vào / ra | CTFL 5.1.3 — không đưa vào checklist |
| smoke test | kiểm thử khói | CTFL 5.1.3 nêu «mọi smoke test đã đạt» làm ví dụ tiêu chí vào |
| defect taxonomy | phân loại (taxonomy) defect | một nguồn mục checklist — xem TEST-054 |

<div class="tkh-callout">
<span class="tkh-callout-title">Ngoài syllabus ISTQB — «smoke» trong bài</span>
<p>CTFL v4.0.1 chỉ nhắc smoke test như ví dụ tiêu chí vào (5.1.3) và đặt nó ở góc phần tư Q4, «thường được tự động hoá» (5.1.7). Trong bài, <b>smoke</b> được hiểu theo nghĩa thông dụng của ngành: một lượt kiểm thử nhanh, nông nhưng rộng các luồng sống còn, để quyết định bản build có đáng kiểm thử chuyên sâu hay phát hành tiếp không. Hai cụm từ read-do / do-confirm vốn xuất phát từ ngành hàng không và y tế, được Gawande (2009) phổ biến; CTAL-TA dùng lại cho kiểm thử.</p>
</div>

## 3. Tình huống kiểm thử tại Tới Nơi

### 3.1. Yêu cầu nghiệp vụ và business rules

Bốn luồng chiếm gần hết doanh thu và khiếu nại. Quy tắc liên quan trong SPEC lab:

| Luồng | Quy tắc chính | Điều phải thấy trên màn hình |
|---|---|---|
| Đặt xe | R-FARE-01…08, R-VOUCHER-02/06, R-IDEM-02, R-STATE-02 | Tổng tiền đúng; bấm lại không tạo chuyến trùng; trạng thái tự cập nhật |
| Bưu kiện | R-PARCEL-02/03 | Phụ phí khối lượng; > 20 kg phải ô tô |
| Đồ ăn | R-FOOD-01…06 | Voucher trừ vào tiền món, không trừ phí giao; tối thiểu 30.000 đ |
| Huỷ có phí | R-CANCEL-01…06, R-ACCEPT-05 | 0 đ khi đang tìm hoặc trong 2 phút đầu; 10.000 đ (xe máy) sau đó; tài xế về `online` |
| Chung | R-UI-01…04, R-API-02, R-GEN-02 | Có nhãn, có `role="alert"`, tiền dạng «52.000 đ», thông điệp tiếng Việt |

### 3.2. System context

```text
Kiến trúc đích:  Customer App / Driver App ─► API Gateway/BFF ─► Booking, Pricing, Dispatch, Trip, Payment …
                 (smoke sau deploy ở pipeline §17 bước 9 — trên staging trước khi promote production)
Lab (MVP):       trình duyệt ─► /  và /driver.html ─► express (một tiến trình) ─► kho bộ nhớ + ví VíXanh giả
                 đồng hồ dời về 09:00 giờ VN qua TOINOI_START_AT; /api/test/* bật (NODE_ENV=test)
```

Kiến trúc đặt **smoke test** ở bước 9 của pipeline (§17), ngay sau deploy. Checklist phục vụ phần **cần mắt người**; phần máy kiểm được thuộc bộ test tự động. Lab không có realtime gateway (R-ARCH-02): trang khách hỏi lại mỗi 2 giây, trang tài xế không tự làm mới.

### 3.3. Rủi ro

Khách thấy giá khác giá bị thu; bấm đúp tạo hai chuyến; khách huỷ bị thu phí sai mốc 2 phút; tài xế không biết chuyến đã bị huỷ và chạy tới đón; thông báo lỗi khó hiểu làm khách bỏ đơn; giờ hiển thị lệch khi máy khách ở múi giờ khác.

## 4. Phân tích kỹ thuật

### 4.1. Bước 1 — phạm vi, mục tiêu, định dạng

| Quyết định | Lựa chọn | Lý do |
|---|---|---|
| Mục tiêu | Phát hiện failure **chặn phát hành** ở 4 luồng; ghi nhận vấn đề dùng được | Smoke trước release |
| Thời lượng | ≤ 30 phút, một người | ⇒ trần khoảng 20 mục |
| Định dạng | Trộn: **read-do** cho số tiền, trạng thái (ghi sẵn giá trị mong đợi); **do-confirm** cho câu chữ, khả năng hiểu | Số tiền cần lặp lại chính xác; câu chữ cần phán đoán |
| Kết quả mỗi mục | Đạt / Không đạt / Không áp dụng + bằng chứng | TA-3.4.2: trả lời có/không/không áp dụng |
| Ưu tiên | P1 dừng phát hành · P2 ghi defect, trưởng nhóm quyết · P3 ghi nhận | TA-3.4.2: mục có ưu tiên theo rủi ro |

Giá trị mong đợi của mục read-do tính trước từ SPEC (09:00, không mưa): xe máy 7,4 km = 12.000 + 4.500 × 7,4 = 45.300 ⇒ **46.000 đ**; với `GIAM20` giảm 20% × 45.300 = 9.060 ⇒ 36.240 ⇒ **37.000 đ**; bưu kiện 6,1 kg cộng 2.000 × ⌈1,1⌉ = 4.000 ⇒ 49.300 ⇒ **50.000 đ**; đồ ăn 60.000 đ với `FOOD15` giảm 9.000, phí giao 46.000 ⇒ **97.000 đ**.

### 4.2. Bước 2 — nguồn mục và bộ lọc của CTFL

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Soạn checklist smoke: bốn nguồn ý tưởng đi qua ba câu hỏi lọc. Máy kiểm được thì sang bộ test tự động; là tiêu chí vào hoặc ra thì sang kế hoạch test; quá chung thì viết lại hoặc tách. Phần còn lại thành checklist CL-SMOKE 20 mục chia 5 nhóm.">
<defs>
<marker id="t053-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
<marker id="t053-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
</defs>
<text x="105" y="24" text-anchor="middle" class="t-b">Nguồn ý tưởng</text>
<rect x="20" y="36" width="170" height="40" rx="6" class="f-surface s-line"/><text x="105" y="61" text-anchor="middle" class="t-small">SPEC R-FARE, R-CANCEL, R-UI</text>
<rect x="20" y="86" width="170" height="40" rx="6" class="f-surface s-line"/><text x="105" y="111" text-anchor="middle" class="t-small">lịch sử defect (TEST-054)</text>
<rect x="20" y="136" width="170" height="40" rx="6" class="f-surface s-line"/><text x="105" y="161" text-anchor="middle" class="t-small">đặc tính chất lượng</text>
<rect x="20" y="186" width="170" height="40" rx="6" class="f-surface s-line"/><text x="105" y="211" text-anchor="middle" class="t-small">phàn nàn của khách</text>
<line x1="190" y1="131" x2="236" y2="131" class="s-muted" stroke-width="1.6" marker-end="url(#t053-a)"/>
<text x="400" y="24" text-anchor="middle" class="t-b">Ba câu hỏi lọc (CTFL 4.4.3)</text>
<rect x="240" y="36" width="320" height="44" rx="6" class="f-warn-soft s-warn"/><text x="400" y="63" text-anchor="middle" class="t-small">Máy kiểm được? ⇒ test tự động (axe, unit)</text>
<rect x="240" y="96" width="320" height="44" rx="6" class="f-warn-soft s-warn"/><text x="400" y="123" text-anchor="middle" class="t-small">Là tiêu chí vào/ra? ⇒ kế hoạch test</text>
<rect x="240" y="156" width="320" height="44" rx="6" class="f-warn-soft s-warn"/><text x="400" y="183" text-anchor="middle" class="t-small">Quá chung? ⇒ viết lại thành câu hỏi đóng</text>
<line x1="560" y1="131" x2="606" y2="131" class="s-muted" stroke-width="1.6" marker-end="url(#t053-a)"/>
<text x="750" y="24" text-anchor="middle" class="t-b">CL-SMOKE v1.0 · 20 mục</text>
<rect x="610" y="36" width="290" height="190" rx="8" class="f-pk-soft s-pk"/>
<text x="625" y="62" class="t-small">Đặt xe · SM-01…05</text><text x="885" y="62" text-anchor="end" class="t-mono t-small">4 P1</text>
<text x="625" y="90" class="t-small">Bưu kiện · SM-06…08</text><text x="885" y="90" text-anchor="end" class="t-mono t-small">2 P1</text>
<text x="625" y="118" class="t-small">Đồ ăn · SM-09…12</text><text x="885" y="118" text-anchor="end" class="t-mono t-small">2 P1</text>
<text x="625" y="146" class="t-small">Huỷ có phí · SM-13…16</text><text x="885" y="146" text-anchor="end" class="t-mono t-small">3 P1</text>
<text x="625" y="174" class="t-small">Chung · SM-17…20</text><text x="885" y="174" text-anchor="end" class="t-mono t-small">0 P1</text>
<text x="625" y="208" class="t-small t-pk">11 read-do · 9 do-confirm</text>
<path d="M755 226 L755 270 L105 270 L105 230" class="f-none s-warn dash" stroke-width="1.4" marker-end="url(#t053-w)"/>
<text x="430" y="262" text-anchor="middle" class="t-small t-warn">kết quả phiên + defect mới ⇒ v1.1 (bỏ mục đã tự động hoá, thêm mục từ phát hiện)</text>
</svg>
</div>
<figcaption><b>Hình 1 · Từ nguồn ý tưởng tới checklist smoke.</b> Bộ lọc là ba loại mục mà CTFL 4.4.3 khuyên không đưa vào checklist. Đường đứt nét: checklist không bao giờ «xong» — mỗi phiên trả phát hiện về nguồn để ra phiên bản sau (CTAL-TA 3.4.2).</figcaption>
</figure>

Năm ứng viên bị loại khi soạn: «máy chủ chạy, dữ liệu mẫu đã nạp?» (**tiêu chí vào**), «≥ 95% test case đạt?» (**tiêu chí ra**), «axe-core không báo vi phạm?» (**máy kiểm được** — để cho test tự động, [TEST-091](/articles/accessibility-testing)), «mọi quy tắc R-FARE đúng?» và «ứng dụng dễ dùng?» (**quá chung** — cái đầu để cho unit test Pricing ở Series 06, cái sau tách thành SM-17…SM-20).

### 4.3. Bước 3 — viết, nhóm, xếp ưu tiên

Mục yếu và cách viết lại (đúng tiêu chí «rõ, cụ thể, đo được» của TA-3.4.2):

| Mục yếu | Vấn đề | Viết lại |
|---|---|---|
| «Kiểm tra voucher» | Không nói kiểm gì, không có kết quả | SM-02: «Nhập « giam20 » (chữ thường, có khoảng trắng): giảm 9.060 đ, tổng 37.000 đ?» |
| «Huỷ chuyến OK?» | Huỷ lúc nào? phí bao nhiêu? | SM-13 (đang tìm ⇒ 0 đ) và SM-14 (nhận + 3 phút ⇒ 10.000 đ) — mỗi mục một ý |
| «Giao diện đẹp» | Không trả lời có/không được | Bỏ; thay bằng SM-19 (định dạng số kiểu Việt Nam) |

Nhóm theo **luồng** (khớp cách tester đi qua ứng dụng), mỗi nhóm read-do trước, do-confirm sau; nhóm «Chung» ở cuối vì quan sát được suốt phiên. Checklist đầy đủ ở mục 5.1.

## 5. Thực hành

Mọi tệp ở `examples/toi-noi-testing/articles/checklist-based-testing/` ([lab trên GitHub](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing)): `checklist-smoke-v1.0.md` (checklist + tiêu chí vào + mục bị loại), `e2e/smoke-checklist.spec.ts`, `playwright.config.ts` (cổng riêng 3153).

### 5.1. Checklist CL-SMOKE v1.0

| Mã | Câu hỏi (rút gọn) | Kiểu | Ưu tiên |
|---|---|---|---|
| SM-01 | Xe máy 7,4 km TP.HCM, không voucher: «46.000 đ», cao điểm/mưa «Không»? | read-do | P1 |
| SM-02 | « giam20 »: giảm «9.060 đ», tổng «37.000 đ»? | read-do | P1 |
| SM-03 | Bấm đúp «Đặt chuyến» (VíXanh): D101 thấy đúng 1 chuyến? | read-do | P1 |
| SM-04 | D101 đi hết 4 bước: trang khách tự hiện «Hoàn thành» ≤ 3 giây, nút huỷ khoá? | read-do | P1 |
| SM-05 | Chuyến VíXanh xong: khách biết bị thu bao nhiêu, thu thành công chưa? | do-confirm | P2 |
| SM-06 | Bưu kiện 6,1 kg, xe máy, 7,4 km: «50.000 đ»? | read-do | P1 |
| SM-07 | 25 kg + xe máy: thông báo nói rõ phải dùng ô tô, không đặt được? | do-confirm | P1 |
| SM-08 | Sửa khối lượng sau báo giá: «Đặt chuyến» khoá, báo giá cũ ẩn? | read-do | P2 |
| SM-09 | Đồ ăn 60.000 đ + FOOD15: giảm «9.000 đ», tổng «97.000 đ»? | read-do | P1 |
| SM-10 | «Giao đồ ăn»: ô «Loại xe» khoá? | read-do | P2 |
| SM-11 | Tiền món 25.000 đ: thông báo nêu mức 30.000 đ? | do-confirm | P1 |
| SM-12 | FOOD15 cho đặt xe: từ chối với thông báo khách hiểu được? | do-confirm | P2 |
| SM-13 | Huỷ khi đang tìm tài xế: «Phí huỷ: 0 đ»? | read-do | P1 |
| SM-14 | Nhận (ETA 5) + 3 phút rồi khách huỷ: «Phí huỷ: 10.000 đ», «Đã huỷ»? | read-do | P1 |
| SM-15 | Sau khi khách huỷ, D101 nhận được chuyến mới? | read-do | P1 |
| SM-16 | Trang tài xế cho biết chuyến vừa bị khách huỷ? | do-confirm | P2 |
| SM-17 | Mọi thông báo lỗi là tiếng Việt, không lộ mã/tên kỹ thuật? | do-confirm | P2 |
| SM-18 | Giờ hiển thị (hạn báo giá) rõ là giờ Việt Nam, kể cả máy khách ở múi giờ khác? | do-confirm | P2 |
| SM-19 | Tiền, quãng đường hiện kiểu Việt Nam («52.000 đ», «7,4 km»)? | do-confirm | P2 |
| SM-20 | Chỉ dùng bàn phím (Tab, Enter) báo giá và đặt được chuyến? | do-confirm | P3 |

SM-18 không tự nghĩ ra: mục này đến từ cụm defect «giờ VN/UTC» trong lịch sử lỗi của Tới Nơi — cách tìm cụm đó ở bài sau, [TEST-054](/articles/defect-based-testing). SM-17 đến từ phát hiện của [TEST-014](/articles/system-testing).

### 5.2. Chạy phiên — người kiểm thử có ghi chép

Phiên được chạy trên trình duyệt thật (Chrome) bằng Playwright, đúng thao tác từng mục, để ai cũng tái hiện được. Spec **không** biến checklist thành test case: nó làm thao tác và **ghi lại điều quan sát được**. Mục read-do có giá trị mong đợi ⇒ khẳng định cứng (P1 sai là dừng). Mục do-confirm ⇒ chỉ ghi bằng chứng, **người** đánh giá.

```ts
await quote(customer, { kind: 'food', km: '7.4', food: '60000', voucher: 'FOOD15' });
await expect(customer.getByTestId('quote-total')).toHaveText('97.000 đ');          // read-do: so cứng
note('SM-09', 'auto', 'tiền món 60.000 đ + FOOD15 ⇒ giảm 9.000 đ, tổng 97.000 đ');

await quote(customer, { food: '25000', voucher: '' });
note('SM-11', 'judge', `tiền món 25.000 đ ⇒ thông báo «${await alertText(customer)}»`); // do-confirm: chỉ ghi
```

Mốc «3 phút sau khi nhận» (SM-14) không chờ thật: spec đọc `acceptedAt` của chuyến rồi dời đồng hồ máy chủ bằng móc kiểm thử `POST /api/test/clock` (R-TEST-02). SM-18 mở thêm một phiên trình duyệt đặt múi giờ `Asia/Tokyo`.

```bash
npx playwright test -c articles/checklist-based-testing/playwright.config.ts
```

```text
SM-03 [auto] bấm đúp ⇒ tài xế thấy 1 chuyến: «ride · 7.4 km · 37.000 đ»
SM-04 [auto] trang khách tự hiện «Hoàn thành» sau 1870 ms, nút huỷ khoá
SM-05 [judge] API: paymentStatus=paid, fare=37000; trang khách hiện: «Chuyến của bạn Mã chuyến: 7431f9f2-… Trạng thái: Hoàn thành Tài xế: D101 Huỷ chuyến»
SM-07 [judge] 25 kg + xe máy ⇒ thông báo «Bưu kiện trên 20 kg phải giao bằng ô tô.», nút Đặt chuyến khoá
SM-11 [judge] tiền món 25.000 đ ⇒ thông báo «Tiền món tối thiểu 30.000 đ.»
SM-12 [judge] FOOD15 cho chuyến đặt xe ⇒ thông báo «Voucher FOOD15 không áp dụng cho chuyến này.»
SM-16 [judge] 3 giây sau khi khách huỷ, trang tài xế vẫn ghi «Tài xế đã nhận»; bấm «Đang tới đón» ⇒ «Không thể thực hiện "arrive" khi chuyến đang ở trạng thái "cancelled".»
SM-18 [judge] hạn báo giá: máy giờ VN «09:05:02», máy giờ Tokyo «11:05:02» (không ghi múi giờ)
  5 passed (14.0s)
```

(Trích; nhật ký đủ 20 dòng in khi chạy. Node 22.18, Playwright 1.64, Chrome cài sẵn, máy chủ lab 09:00 giờ VN 04/11/2026, không mưa. Mã chuyến là uuid ngẫu nhiên, thời gian lan truyền đổi theo lần chạy — trang khách hỏi lại mỗi 2 giây nên quanh 1,9–2,5 giây.)

### 5.3. Kết quả — người đánh giá

| Mục | Kết quả | Bằng chứng / nhận xét |
|---|---|---|
| SM-01…04, 06, 08, 09, 10, 13, 14, 15 | **Đạt** (11 mục read-do) | Đúng mọi giá trị tính ở mục 4.1 |
| SM-07, 11 | Đạt | Thông báo tiếng Việt, nêu đúng ngưỡng; SM-07 nói «phải giao bằng ô tô» nhưng không gợi ý đổi loại xe — chấp nhận |
| SM-12 | Đạt (ghi chú) | Bị từ chối, nhưng không nói **vì sao** («chỉ áp dụng giao đồ ăn») — gợi ý cải tiến, không phải defect |
| SM-20 | Đạt | Thứ tự Tab hợp lý, Enter gửi form |
| SM-05 | **Không đạt · F-1** | Ví đã thu (`paymentStatus=paid`) nhưng trang khách không hiện số tiền đã thu hay trạng thái thanh toán. R-UI-01 không yêu cầu ⇒ thiếu sót **đặc tả**, câu hỏi cho PO |
| SM-16 | **Không đạt · F-2** | Trang tài xế không biết chuyến đã bị huỷ; chỉ biết khi bấm bước tiếp theo và nhận lỗi. R-UI-02 không yêu cầu tự làm mới ⇒ thiếu sót đặc tả; ở hệ đích là việc của realtime gateway |
| SM-17 | Không đạt · đã biết | `"arrive"`, `"cancelled"` lộ trong thông báo — trùng phát hiện của TEST-014, đã có defect report, chưa sửa |
| SM-18 | **Không đạt · F-3** | Cùng một báo giá: «09:05:02» trên máy giờ VN, «11:05:02» trên máy giờ Tokyo, không ghi múi giờ. Khách du lịch đọc nhầm hạn báo giá |
| SM-19 | **Không đạt · F-4** | Tiền đúng «37.000 đ»; nhưng danh sách tài xế ghi «7.4 km» (dấu chấm), trái quy ước SPEC «7,4 km» |
| *ngoài checklist* | **Phát hiện O-1** | Danh sách tài xế ghi `ride` — mã dịch vụ nội bộ, không phải «Đặt xe» |

**Tổng:** 15 Đạt · 5 Không đạt (4 mới, 1 đã biết) · 1 phát hiện ngoài checklist. **Mọi mục P1 đạt** ⇒ checklist không chặn phát hành; năm vấn đề P2 được ghi defect hoặc câu hỏi, trưởng nhóm quyết. O-1 nhắc lại điều CTAL-TA nói: checklist hướng sự chú ý, không giới hạn nó — tester vẫn ghi mọi thứ mình thấy.

### 5.4. Cập nhật lên v1.1

| Thay đổi | Mục | Lý do |
|---|---|---|
| **Chuyển ra test tự động** | SM-01…04, 06, 08…10, 13…15 (11 mục) | Spec đã khẳng định cứng chúng ⇒ đúng loại «máy kiểm được» mà CTFL khuyên không để trong checklist. Smoke tự động chạy ở pipeline; checklist dành cho người không lặp lại các mục này |
| **Gộp, viết lại** | SM-17 + O-1 ⇒ «Mọi chữ hiện cho người dùng (lỗi, danh sách, nhãn) là tiếng Việt, không lộ mã kỹ thuật?» | Phát hiện ngoài checklist cho thấy mục cũ hẹp |
| **Thêm** | SM-21 «Khách thấy chuyến đã được thu tiền chưa (F-1)?»; SM-22 «Tài xế được báo khi chuyến bị huỷ (F-2)?» | Giữ tới khi defect đóng; sau đó xem lại có còn đáng giữ |
| **Giữ** | SM-05 → SM-21, SM-07, 11, 12, 16 → SM-22, 18, 19, 20 | Cần phán đoán của người |

v1.1 còn **11 mục do-confirm**, chạy khoảng 15 phút thay vì 30 — thời gian còn lại dành cho một phiên khám phá ([TEST-052](/articles/exploratory-testing)) quanh các phát hiện F-1…F-4.

## 6. Góc nhìn nâng cao

**Checklist mòn.** CTFL nói thẳng: mục kém hiệu quả dần vì developer học cách tránh lỗi đó — giống nguyên lý «tests wear out» ở [TEST-003](/articles/bay-nguyen-ly-kiem-thu). Một tiêu chí thực dụng (giả định của bài): mục nào **Đạt 10 bản phát hành liên tiếp** và đã có test tự động phủ thì đưa ra họp xem lại. Ngược lại, mỗi defect nghiêm trọng lọt production là ứng viên một mục mới — đó là đầu vào chính của [TEST-054](/articles/defect-based-testing).

**Độ chi tiết là đánh đổi.** Mục read-do chi tiết (ghi sẵn «46.000 đ») cho kết quả lặp lại được nhưng chỉ phủ đúng một điểm; mục do-confirm chung («thông báo có dễ hiểu?») để mỗi tester đi một đường — phủ rộng hơn, khó lặp lại hơn (CTFL 4.4.3 ý 7). Smoke cần cả hai.

**So với hai kỹ thuật kinh nghiệm còn lại.** Error guessing ([TEST-051](/articles/error-guessing)) nhắm vào một danh sách lỗi dự đoán; exploratory ([TEST-052](/articles/exploratory-testing)) thiết kế và chạy cùng lúc theo charter, khó lặp lại nhất; checklist-based lặp lại tốt nhất, hợp cho việc kiểm mỗi lần phát hành. Ba kỹ thuật bổ trợ nhau: checklist là nguồn gợi ý khi cạn ý trong phiên khám phá (CTAL-TA 3.4.2), và fault attack tốt thường thành mục checklist.

**Sai lầm thường gặp:**

- *Test case trá hình* — 200 mục, mỗi mục 6 bước: mất lợi thế nhẹ và nhanh.
- *Giữ mục máy kiểm được* — người so 11 con số mỗi lần phát hành vừa tốn công vừa dễ sai hơn máy.
- *«Đạt» không bằng chứng* — không ai kiểm lại hay so được giữa hai bản.
- *Không ghi ngoài checklist* — O-1 sẽ biến mất nếu tester chỉ tick đúng 20 dòng.
- *Một checklist cho mọi mục đích* — smoke 30 phút khác hồi quy thanh toán; gộp lại thì quá dài.

**Giới hạn.** Checklist chỉ tốt bằng kinh nghiệm của người soạn: nó không tìm được loại lỗi chưa ai nghĩ tới. Độ phủ đo theo **số mục đã kiểm**, không phải độ phủ đặc tả hay code — 20/20 mục không có nghĩa luồng không còn lỗi (nguyên lý «testing shows the presence of defects»).

## 7. Bài tập và câu hỏi ôn tập

### Bài tập 1 — Checklist cho trang tài xế (K3)

Soạn 5 mục checklist (ghi kiểu read-do/do-confirm và ưu tiên) để smoke trang `/driver.html` theo R-UI-02, R-ACCEPT-01…07, R-STATE-04.

<details><summary><b>Lời giải mẫu</b></summary><ul><li><b>DR-01 · read-do · P1</b> D104 (offline) bấm «Nhận chuyến»: bị từ chối với thông báo đang offline? (R-ACCEPT-02)</li><li><b>DR-02 · read-do · P1</b> D101 (xe máy SG) không thấy chuyến ô tô; D103 (ô tô SG) thấy? (R-ACCEPT-01, R-ACCEPT-06)</li><li><b>DR-03 · read-do · P1</b> Bấm «Nhận chuyến» hai lần cho cùng chuyến: lần hai không báo lỗi, trạng thái giữ «Tài xế đã nhận»? (R-ACCEPT-07)</li><li><b>DR-04 · read-do · P1</b> «Huỷ nhận»: chuyến biến khỏi «Chuyến đang chạy» và quay lại danh sách chờ? (R-STATE-04)</li><li><b>DR-05 · do-confirm · P2</b> Bấm sai thứ tự bước: thông báo dùng nhãn nút/trạng thái tiếng Việt? (SM-17 v1.1)</li></ul><p>Không nên có mục «Nút có <code>data-testid</code>?» — máy kiểm được.</p></details>

### Bài tập 2 — Lọc ứng viên (K2)

Mục nào nên giữ trong checklist smoke, mục nào chuyển đi đâu? (a) «Trang khách tải trong dưới 2 giây?»; (b) «Bản build đã qua review code?»; (c) «Khách hiểu vì sao voucher bị từ chối?»; (d) «Mọi API trả JSON hợp lệ?».

<details><summary><b>Lời giải mẫu</b></summary><ul><li>(a) Máy đo được ⇒ test hiệu năng tự động hoặc giám sát (Series 09).</li><li>(b) Tiêu chí vào của phiên test, không phải mục checklist.</li><li>(c) Giữ — do-confirm, cần phán đoán của người (gần SM-12).</li><li>(d) Máy kiểm được (schema validation, Series 07) và quá chung cho smoke giao diện.</li></ul></details>

### Bài tập 3 — Mục mới từ defect (K3)

Production báo: «khách huỷ đúng lúc 2 phút 0 giây sau khi tài xế nhận, không bị thu phí». Viết mục checklist cho v1.2 và cho biết mục đó thuộc read-do hay do-confirm.

<details><summary><b>Lời giải mẫu</b></summary><p>«Nhận chuyến, dời đồng hồ đúng 2 phút 0 giây, khách huỷ: «Phí huỷ: 10.000 đ»?» — read-do, P1, nguồn R-CANCEL-02 (đúng 2 phút tính là có phí) + defect production. Vì giá trị mong đợi rõ và máy kiểm được, mục này nên ở checklist <b>tạm thời</b>, rồi chuyển thành test tự động ở mức component (phí huỷ, Series 06) — chạy mãi mãi, rẻ hơn mắt người.</p></details>

### Câu hỏi trắc nghiệm

**Câu 1 (K2).** Theo CTFL v4.0.1, loại mục nào **không** nên đưa vào checklist?

- a) Mục viết dưới dạng câu hỏi
- b) Mục về đặc tính chất lượng như usability
- c) Mục mà công cụ kiểm tự động được
- d) Mục xây từ hiểu biết phần mềm hỏng thế nào

<details><summary><b>Đáp án: c</b></summary><ul><li>a) Sai — syllabus khuyến khích viết mục thành câu hỏi.</li><li>b) Sai — checklist dùng được cho kiểm thử phi chức năng (ví dụ heuristic usability).</li><li>c) Đúng — mục 4.4.3: không chứa mục kiểm tự động được, mục hợp làm tiêu chí vào/ra, mục quá chung.</li><li>d) Sai — đó là một nguồn hợp lệ để xây checklist.</li></ul></details>

**Câu 2 (K2).** Vì sao checklist cần được cập nhật định kỳ dựa trên phân tích defect?

- a) Vì syllabus quy định mỗi checklist tối đa 20 mục
- b) Vì developer học cách tránh lỗi cũ nên một số mục kém hiệu quả dần, và defect nghiêm trọng mới cần có mục
- c) Vì checklist phải trùng khớp test case
- d) Vì checklist cũ không dùng được cho kiểm thử phi chức năng

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — syllabus không đặt con số, chỉ cảnh báo đừng để quá dài.</li><li>b) Đúng — mục 4.4.3.</li><li>c) Sai — checklist dùng chính khi thiếu test case chi tiết.</li><li>d) Sai — không liên quan tới tuổi của checklist.</li></ul></details>

**Câu 3 (K2).** Nhóm dùng một checklist rất chung («Màn hình có ổn không?») thay cho test case. Hệ quả nào **đúng nhất** theo CTFL?

- a) Kết quả hoàn toàn lặp lại được giữa các tester
- b) Có thể phủ rộng hơn nhưng kém lặp lại hơn
- c) Độ phủ code đạt 100%
- d) Không còn cần tester có kinh nghiệm

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — checklist chung tăng độ biến thiên.</li><li>b) Đúng — ý cuối mục 4.4.3.</li><li>c) Sai — checklist không đo độ phủ code.</li><li>d) Sai — ngược lại, mục càng chung càng dựa vào kinh nghiệm.</li></ul></details>

**Câu 4 (K3).** Tester cần một mục để kiểm 6 đầu vào không hợp lệ cụ thể của ô «Quãng đường» (0,44; 100,05; −1; chữ; trống; 1e9). Theo CTAL-TA v4.0, định dạng nào hợp nhất?

- a) Do-confirm checklist
- b) Read-do checklist
- c) Test charter
- d) Tiêu chí ra

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — do-confirm dùng để gợi ý dòng suy nghĩ, xác nhận sau khi làm.</li><li>b) Đúng — mục 3.4.2: read-do chứa yếu tố chính phải làm, ví dụ đầu vào không hợp lệ cụ thể cho một ô nhập.</li><li>c) Sai — charter thuộc session-based exploratory testing (3.4.1).</li><li>d) Sai — không phải mục kiểm thử.</li></ul></details>

**Câu 5 (K3).** Mục checklist nào đáp ứng tốt nhất tiêu chí «rõ, cụ thể, đo được, trả lời có/không/không áp dụng» của CTAL-TA?

- a) «Kiểm tra chức năng huỷ chuyến»
- b) «Huỷ chuyến có hợp lý không?»
- c) «Khách huỷ khi tài xế đã nhận được 3 phút (xe máy): màn hình hiện «Phí huỷ: 10.000 đ»?»
- d) «Huỷ chuyến theo mọi quy tắc R-CANCEL»

<details><summary><b>Đáp án: c</b></summary><ul><li>a) Sai — không có câu hỏi, không có kết quả.</li><li>b) Sai — «hợp lý» không đo được.</li><li>c) Đúng — một ý, điều kiện rõ, kết quả quan sát được.</li><li>d) Sai — quá chung, không kiểm trực tiếp được.</li></ul></details>

**Câu 6 (K2).** Phát biểu nào đúng về quan hệ giữa checklist-based testing và các kỹ thuật kinh nghiệm khác?

- a) Checklist không bao giờ được dùng trong exploratory testing
- b) Checklist có thể là nguồn ý tưởng khi tester cạn ý trong một phiên khám phá
- c) Checklist-based testing là kỹ thuật hộp trắng
- d) Checklist-based testing thay được error guessing

<details><summary><b>Đáp án: b</b></summary><ul><li>a) Sai — CTAL-TA 3.4.2 nêu đúng điều ngược lại.</li><li>b) Đúng — checklist là lời nhắc, nguồn cảm hứng khi cạn ý.</li><li>c) Sai — là kỹ thuật dựa trên kinh nghiệm (CTFL 4.4).</li><li>d) Sai — ba kỹ thuật bổ trợ, không thay nhau.</li></ul></details>

## 8. Tài liệu tham khảo

- ISTQB, *Certified Tester Foundation Level Syllabus* v4.0.1 (2024): mục 4.4 (4.4.3 · FL-4.4.3), 5.1.3 (tiêu chí vào/ra), 5.1.7 (góc phần tư) — https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- ISTQB, *Certified Tester Advanced Level Test Analyst Syllabus* v4.0: mục 3.4, 3.4.2 (TA-3.4.2) — https://www.istqb.org/wp-content/uploads/sdm-uploads/ISTQB-CTAL-TA-Syllabus-v4.0-EN-4.pdf
- ISTQB Glossary — https://glossary.istqb.org/ (mục *checklist-based testing*, *experience-based test technique*)
- Gawande, A. (2009), *The Checklist Manifesto: How to Get Things Right*, Metropolitan Books — CTFL v4.0.1 dẫn về độ dài checklist.
- Brykczynski, B. (1999), «A survey of software inspection checklists», *ACM SIGSOFT Software Engineering Notes* — CTFL dẫn về loại mục không nên đưa vào.
- Nielsen, J. (1994), 10 heuristic usability — CTFL dẫn làm ví dụ checklist phi chức năng.
- Lab Tới Nơi: `SPEC.md` (R-FARE, R-PARCEL, R-FOOD, R-CANCEL, R-UI, R-TEST-02), `articles/checklist-based-testing/` — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [TEST-028 Checklist-based Review](/articles/checklist-based-review) · [TEST-051 Error Guessing](/articles/error-guessing) · [TEST-052 Exploratory Testing](/articles/exploratory-testing) · [TEST-014 System Testing](/articles/system-testing) · [TEST-003 Bảy nguyên lý](/articles/bay-nguyen-ly-kiem-thu) · [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi)

Bài tiếp theo: [TEST-054 · Defect-based Testing](/articles/defect-based-testing)
