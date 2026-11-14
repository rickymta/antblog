---
slug: system-testing-usability-test
title: "Usability Test: người dùng có tự hoàn thành được việc họ cần không?"
summary: "Usability testing cho trang đặt xe Tới Nơi: phân biệt với functional, accessibility và UAT; chọn phương pháp formative có điều phối; viết nhiệm vụ không dẫn đáp án; ghi outcome, mẫu số và time-on-task trung thực; tách quan sát khỏi diễn giải. Kèm bộ study kit dùng được ngay và hai supporting check Playwright — chưa phải nghiên cứu người dùng thật."
category: testing
level: intermediate
tags: [testing, sys-12, usability-testing, user-research, accessibility, task-success, time-on-task, playwright, toi-noi]
language: vi
---

API báo giá đúng 35.000 đ, nút «Đặt chuyến» chạy tốt trên Chrome và Edge, mọi bài functional đều xanh. Vậy mà một người mới mở trang có thể bấm «Báo giá» rồi ngồi chờ tài xế, vì tưởng đã gọi xe; hoặc thấy dòng báo lỗi đỏ mà không biết phải sửa ô nào. Script automation luôn bấm đúng nút theo kịch bản, nên script không bao giờ gặp những nhầm lẫn đó. Bài này trả lời: **làm sao biết người dùng thật có tự hoàn thành được việc họ cần, và làm sao thu bằng chứng về điều đó mà không tự lừa mình bằng con số đẹp.** Phần lớn bài là phương pháp và tài liệu cho một phiên quan sát người dùng; phần code chỉ kiểm vài điều kiện hỗ trợ.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · SYS-12</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>Kiểm thử hệ thống Tới Nơi · Phần 2</td></tr>
<tr><th scope="row">Cấp độ</th><td>Intermediate — tester, BA, designer, product; không cần viết code để tổ chức study</td></tr>
<tr><th scope="row">Test level / hướng kiểm</th><td>Đặc tính phi chức năng (usability) ở cấp hệ thống; phương pháp formative, moderated; accessibility chỉ ở mức kiểm bổ trợ</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/system-testing-compatibility-test">SYS-11 · Compatibility Test</a> · <a href="/articles/non-functional-testing-tong-quan">Non-functional testing tổng quan</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Human testing: study kit gồm kế hoạch phiên, script mở đầu, ba task card U1–U3, quy tắc ghi số liệu, worksheet, mẫu finding. Code bổ trợ: 2 check Playwright US-A01, US-A02 (R-UI-01, R-UI-03, R-UI-04) — <code>examples/toi-noi-testing/articles/system-testing-usability-test/</code></td></tr>
<tr><th scope="row">Nguồn chính</th><td>GOV.UK Service Manual (moderated usability testing); W3C WAI (Involving Users in Evaluating Web Accessibility); <code>SPEC.md</code> của lab</td></tr>
</tbody>
</table>
</div>

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Trạng thái bằng chứng</span>
<p><b>Chưa có phiên nghiên cứu nào với người tham gia thật</b> (participant = 0). Bài không có success rate, time-on-task, rating hay finding nào rút ra từ người dùng thật. Mọi con số và trích dẫn người dùng trong bài đều được gắn nhãn <i>minh hoạ</i>. Thứ duy nhất đã chạy là hai supporting check tự động, và chúng không đo usability.</p>
</div>

## 1. Bài toán và mục tiêu

### 1.1. Tình huống

Trang khách `/` của lab Tới Nơi có một form: mã khách, dịch vụ, thành phố, loại xe, quãng đường, voucher, phương thức thanh toán; nút «Báo giá» hiện tổng tiền; nút «Đặt chuyến» tạo chuyến từ báo giá; nút «Huỷ chuyến» hiện phí huỷ (R-UI-01). Về chức năng, các bài SYS-03 tới SYS-05 đã kiểm từng quy tắc: cước, voucher, chuyển trạng thái, hành trình hai vai trò.

Nhưng có ba câu hỏi mà những bài đó không trả lời được:

- Người mới có **phân biệt được báo giá với đặt chuyến** không? Hai nút đặt cạnh nhau; nếu khách tưởng «Báo giá» là gọi xe, họ chờ mãi mà không có tài xế, rồi bỏ đi.
- Khi nhập quãng đường không hợp lệ, người dùng có **hiểu thông báo và tự sửa** được không? Theo R-FARE-02, 0,1 km bị từ chối vì nhỏ hơn 0,5 km. Hệ thống trả lỗi đúng, nhưng thông báo có giúp họ sửa không?
- Sau khi huỷ, họ có **xác nhận được đã huỷ và biết có phí hay không**? R-CANCEL-01 nói huỷ khi đang tìm tài xế thì không mất phí; giao diện có nói điều đó đủ rõ không?

Một test Playwright bấm đúng nút theo thứ tự đã viết sẵn. Test này trả lời «hệ thống có xử lý đúng khi được thao tác đúng không», không trả lời «một người chưa từng thấy trang này sẽ thao tác thế nào».

### 1.2. Đối tượng và điều kiện tiên quyết

Bài dành cho tester, BA, designer và người làm sản phẩm. Bạn cần biết luồng đặt xe của Tới Nơi ở mức người dùng (xem [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi)) và phân biệt functional với non-functional. Không cần viết code để tổ chức nghiên cứu; phần chạy Playwright chỉ dành cho ai muốn tái hiện supporting check.

### 1.3. Mục tiêu đo được

Sau bài, bạn có thể:

1. Chuyển một câu hỏi sản phẩm thành nhiệm vụ trung lập, không đọc tên nút hay các bước.
2. Chuẩn bị một phiên moderated: đồng ý tham gia, vai trò, script mở đầu, quy tắc hỗ trợ.
3. Ghi hành vi tách khỏi suy diễn, và gán outcome đúng loại (independent, assisted, failed, abandoned, blocked).
4. Tính một metric có mẫu số rõ ràng và giải thích ai bị loại, vì sao.
5. Viết finding có evidence, tác động, giả thuyết và cách kiểm lại; ưu tiên theo hậu quả, không chỉ theo tần suất.
6. Nói rõ supporting check tự động chứng minh gì và không chứng minh gì.

## 2. Khái niệm và những điều dễ nhầm

### 2.1. Usability testing là gì

Usability testing là **quan sát người dùng đại diện thực hiện nhiệm vụ cụ thể trong một bối cảnh cụ thể**, để tìm trở ngại và đánh giá mức họ đạt mục tiêu. Ba khía cạnh thường được xem xét: hiệu quả (có hoàn thành không), công sức (mất bao lâu, sai bao nhiêu lần, cần giúp không), và trải nghiệm được người dùng phản hồi.

Điểm mấu chốt nằm ở chữ *quan sát*. Một người nói «giao diện đẹp» không chứng minh được họ đặt được xe; một người im lặng bấm nhầm ba lần rồi tự sửa lại cho ta nhiều thông tin hơn. Ý kiến có giá trị, nhưng chỉ khi đặt cạnh hành vi.

### 2.2. Bảng phân biệt

| Hoạt động | Trả lời câu hỏi nào | Không thay thế được |
|---|---|---|
| Functional test | Hệ thống xử lý đúng khi được thao tác theo test? | Người mới có tự tìm được đường đi không |
| Usability study | Người dùng đại diện hiểu và hoàn thành nhiệm vụ ra sao? | Kiểm tra mọi quy tắc kỹ thuật |
| Accessibility evaluation | Người có nhu cầu truy cập khác nhau dùng được không, đạt tiêu chí nào? | Toàn bộ usability; chứng nhận chỉ bằng scanner |
| Expert / heuristic review | Chuyên gia thấy vấn đề gì theo nguyên tắc thiết kế? | Quan sát participant thật |
| UAT | Đại diện nghiệp vụ có chấp nhận giải pháp theo tiêu chí? | Trải nghiệm của mọi nhóm khách |

Accessibility và usability giao nhau. Người chỉ dùng bàn phím mà không tới được nút «Báo giá» thì không hoàn thành task. Nhưng hai chiều đều có giới hạn: thuộc tính `aria-live` tồn tại trong DOM không chứng minh screen reader đọc đúng lúc; và một study vài người không chứng minh tuân thủ toàn bộ tiêu chuẩn accessibility. Phân biệt với UAT cũng quan trọng: nghiệp vụ có thể ký chấp nhận một giao diện mà khách mới vẫn gặp khó (xem [Acceptance testing tổng quan](/articles/acceptance-testing-tong-quan) và [SYS-15 · UAT](/articles/system-testing-uat)).

### 2.3. Ưu điểm và nhược điểm

Usability study tìm ra loại lỗi mà không suite tự động nào tìm được: mô hình hiểu sai của người dùng. Một phiên 30 phút có thể chỉ ra rằng nhãn nút gây hiểu lầm, điều mà hàng trăm test xanh không thấy.

Đổi lại, phương pháp này tốn công tuyển người, cần kỹ năng điều phối, mẫu nhỏ nên không cho phép suy luận thống kê mạnh, và dễ bị thiên lệch: moderator gợi ý vô tình, participant muốn làm hài lòng người hỏi, đồng nghiệp đã quen sản phẩm. Phần lớn bài này là về cách giảm các thiên lệch đó.

## 3. Chọn phương pháp theo câu hỏi

### 3.1. Moderated hay unmoderated, formative hay summative

| Lựa chọn | Phù hợp khi | Rủi ro |
|---|---|---|
| Moderated | Cần hỏi lại lý do, quan sát cách xử lý tình huống mới | Moderator ảnh hưởng hành vi; tốn người |
| Unmoderated | Muốn nhiều phiên hơn với chi phí thấp | Khó phân biệt bỏ cuộc với lỗi setup nếu thiết kế thiếu rõ |
| Formative | Tìm vấn đề để cải tiến trong vòng thiết kế | Không cho phép ước lượng tỷ lệ cho toàn bộ người dùng |
| Summative | Đánh giá theo tiêu chí định trước, so sánh phiên bản | Cần cỡ mẫu và phương pháp phân tích phù hợp hơn |

Bài chọn **formative, moderated** cho ba câu hỏi ở mục 1.1. Phạm vi cố ý hẹp: chỉ trang khách của lab. Không đánh giá trải nghiệm toàn nền tảng, bản đồ, ứng dụng tài xế, GPS hay thanh toán thật.

### 3.2. Bao nhiêu người, là ai

Study kit đề xuất **sáu người cho vòng học ban đầu**, chia ba người mới và ba người đã quen ứng dụng đặt xe tương tự. Sáu không phải một định luật «đủ người», không bảo đảm phát hiện mọi vấn đề, càng không cho phép nói «X% khách hàng sẽ thành công». Đó là ngân sách hợp lý cho một vòng formative; sau khi sửa, ta chạy vòng tiếp theo.

Tuyển theo nhóm sử dụng dự kiến, mức kinh nghiệm, thiết bị và nhu cầu hỗ trợ. Thành viên đội đã xây giao diện thì nhớ đường đi, nên không đại diện người mới. Nếu phải pilot với đồng nghiệp — và nên pilot — thì giữ dữ liệu pilot tách riêng.

### 3.3. Test basis, rủi ro và tiêu chí bắt đầu/kết thúc

Test basis gồm R-UI-01 (chức năng trang khách), R-UI-03 (khả năng tiếp cận: mọi ô có nhãn, lỗi dùng `role="alert"`, kết quả cập nhật trong vùng `aria-live`), R-UI-04 (định dạng tiền «35.000 đ»), R-FARE-02 (khoảng quãng đường hợp lệ) và R-CANCEL-01 (phí huỷ theo trạng thái). Nhưng usability còn có một basis không nằm trong SPEC: **mục tiêu của người dùng**. SPEC nói nút huỷ phải hiện phí, nhưng không nói người dùng phải hiểu số đó là phí hay là hoàn tiền.

| Rủi ro | Hậu quả với Tới Nơi | Nhiệm vụ quan sát |
|---|---|---|
| Nhầm báo giá với đặt chuyến | Khách chờ không có xe, bỏ đi, mất chuyến | U1 |
| Không hiểu thông báo lỗi | Khách bỏ cuộc ở bước nhập liệu | U2 |
| Không chắc đã huỷ / sợ bị trừ tiền | Gọi tổng đài, mất niềm tin | U3 |

**Tiêu chí bắt đầu:** câu hỏi nghiên cứu và đối tượng đã thống nhất; build, browser và dữ liệu giả được chốt; tài liệu đồng ý và quy tắc lưu dữ liệu sẵn sàng; pilot đã chạy và prompt đã sửa.
**Tiêu chí kết thúc vòng:** đủ số phiên dự kiến hoặc đã thấy cùng một trở ngại lặp lại đủ để ra quyết định sửa; mọi phiên có worksheet đầy đủ; finding được tổng hợp theo task, có evidence. Hiện tại tiêu chí bắt đầu chưa đạt: chưa pilot, chưa tuyển.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 250" role="img" aria-label="Quy trình một vòng usability study formative: chuẩn bị gồm câu hỏi, tuyển người, đồng ý và pilot; phiên quan sát gồm mở đầu, ba nhiệm vụ U1 đến U3 và hỏi lại; tổng hợp theo task với mẫu số rõ; finding được ưu tiên rồi sửa và retest, quay về vòng mới. Supporting check tự động nằm bên dưới, chỉ kiểm điều kiện hỗ trợ và không đưa dữ liệu vào bảng kết quả người dùng.">
<defs>
<marker id="sys12-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<rect x="16" y="30" width="180" height="96" rx="8" class="f-pk-soft s-pk"/><text x="106" y="56" text-anchor="middle" class="t-b">1. Chuẩn bị</text><text x="106" y="78" text-anchor="middle" class="t-small">câu hỏi · tuyển người</text><text x="106" y="96" text-anchor="middle" class="t-small">đồng ý · pilot</text>
<rect x="246" y="30" width="190" height="96" rx="8" class="f-pk-soft s-pk"/><text x="341" y="56" text-anchor="middle" class="t-b">2. Phiên quan sát</text><text x="341" y="78" text-anchor="middle" class="t-small">mở đầu · U1 U2 U3</text><text x="341" y="96" text-anchor="middle" class="t-small">ghi hành vi · hỏi lại</text>
<rect x="486" y="30" width="190" height="96" rx="8" class="f-pk-soft s-pk"/><text x="581" y="56" text-anchor="middle" class="t-b">3. Tổng hợp</text><text x="581" y="78" text-anchor="middle" class="t-small">theo task · raw counts</text><text x="581" y="96" text-anchor="middle" class="t-small">mẫu số = có cơ hội gặp</text>
<rect x="726" y="30" width="178" height="96" rx="8" class="f-pk-soft s-pk"/><text x="815" y="56" text-anchor="middle" class="t-b">4. Finding</text><text x="815" y="78" text-anchor="middle" class="t-small">ưu tiên theo hậu quả</text><text x="815" y="96" text-anchor="middle" class="t-small">sửa → retest</text>
<line x1="196" y1="78" x2="242" y2="78" class="s-pk" stroke-width="1.6" marker-end="url(#sys12-arrow)"/>
<line x1="436" y1="78" x2="482" y2="78" class="s-pk" stroke-width="1.6" marker-end="url(#sys12-arrow)"/>
<line x1="676" y1="78" x2="722" y2="78" class="s-pk" stroke-width="1.6" marker-end="url(#sys12-arrow)"/>
<path d="M815 126 L815 146 L106 146 L106 130" fill="none" class="s-pk dash" stroke-width="1.4" marker-end="url(#sys12-arrow)"/>
<text x="460" y="140" text-anchor="middle" class="t-small">vòng formative tiếp theo</text>
<rect x="246" y="176" width="430" height="56" rx="8" class="f-warn-soft s-warn"/><text x="461" y="199" text-anchor="middle" class="t-b">Supporting check (Playwright US-A01, US-A02)</text><text x="461" y="219" text-anchor="middle" class="t-small t-warn">kiểm nhãn, một đoạn Tab, alert — không phải dữ liệu người dùng</text>
<line x1="341" y1="176" x2="341" y2="130" class="s-warn dash" stroke-width="1.4"/>
<text x="770" y="206" text-anchor="middle" class="t-small t-warn">không ghi vào bảng kết quả</text>
</svg>
</div>
<figcaption><b>Hình 1 · Một vòng usability study formative.</b> Dữ liệu chính đến từ phiên quan sát người. Supporting check tự động chỉ xác nhận vài điều kiện để phiên không bị chặn bởi lỗi kỹ thuật; nó không đi vào bảng success rate.</figcaption>
</figure>

## 4. Thiết kế nhiệm vụ không dẫn đáp án

### 4.1. Prompt nêu mục tiêu, không nêu nút

Prompt «Hãy bấm Báo giá, rồi tìm Tổng tiền» đo khả năng làm theo chỉ dẫn, không đo giao diện. Prompt tốt nêu **mục tiêu và bối cảnh** bằng ngôn ngữ của người dùng: «Bạn định đi xe máy 5 km; hãy tìm số tiền cần trả, chưa yêu cầu tài xế.» Khi đó ta quan sát được người dùng *nghĩ* nút nào làm gì.

Ba quy tắc thực hành:

- Không dùng từ trùng nhãn nút nếu có thể tránh («dừng chuyến» thay vì «Huỷ chuyến»).
- Không gợi ý thứ tự bước.
- Ghi tiêu chí thành công cho moderator, **không đọc** cho participant.

### 4.2. Setup có chủ đích và giới hạn của nó

Có thể chuẩn bị dữ liệu để tập trung vào một nhiệm vụ, nhưng phải ghi rõ hệ quả. U3 dùng một chuyến `searching` do moderator tạo trước, nên U3 **không** chứng minh participant tự đặt được chuyến. Với U2, nếu người dùng tự sửa 0,1 thành 5 trước khi bấm, đừng ép họ tạo lỗi chỉ để đúng script; ghi «error-message exposure = no» — họ đạt mục tiêu sửa dữ liệu, nhưng ta không có bằng chứng họ hiểu thông báo lỗi.

### 4.3. Ba task card

Bảng dưới lấy nguyên từ study kit. Cột prompt đọc cho participant; các cột còn lại chỉ moderator xem.

| ID | Prompt đọc cho participant | Setup riêng | Success không trợ giúp | Giới hạn quan sát |
|---|---|---|---|---|
| U1 | «Bạn định đi xe máy 5 km tại TP.HCM. Hãy tìm số tiền cần trả; chưa yêu cầu tài xế.» | Trang đầu, dữ liệu giả đã nạp | Báo đúng 35.000 đ và dừng trước booking | Tối đa 4 phút trước khi hỏi có muốn tiếp tục/nhận gợi ý |
| U2 | «Bạn đang xem báo giá nhưng thông tin quãng đường là 0,1 km. Hãy tìm cách lấy báo giá cho chuyến 5 km.» | Distance 0.1, chưa submit | Gặp/nhận biết lỗi khi áp dụng; sửa 5, báo giá 35.000 và giải thích đã sửa gì | Tối đa 4 phút; sửa trước submit thì không ép tạo lỗi, ghi đường đi |
| U3 | «Bạn đã yêu cầu chuyến nhưng đổi kế hoạch khi chưa có tài xế nhận. Hãy dừng chuyến và cho biết có khoản phí nào không.» | Seed sạch, moderator tạo chuyến `searching` riêng trước nhiệm vụ | Huỷ, xác nhận đã huỷ và phí 0 đ | Tối đa 4 phút; không nhắc tên nút huỷ |

Oracle cho tiêu chí thành công lấy từ SPEC, không từ giao diện: 35.000 đ là cước xe máy 5 km giờ thường tại SG, không mưa, không voucher (R-FARE-01…08, đã tính ở [SYS-01](/articles/system-testing-smoke-test)); 0 đ là phí huỷ khi chuyến còn `searching` (R-CANCEL-01, lý do `searching` theo R-CANCEL-06).

## 5. Chạy phiên quan sát từng bước

### 5.1. Chuẩn bị và đồng ý tham gia

Chốt câu hỏi nghiên cứu, đối tượng, build, browser/version/viewport, dữ liệu giả. Đặt lại seed và giờ 09:00 VN trước mỗi nhiệm vụ độc lập. Không nối thanh toán thật.

Giải thích cho participant: mục đích, tự nguyện, quyền bỏ câu hỏi hoặc dừng, dữ liệu nào được lưu, ai xem, lưu ở đâu và khi nào xoá. Xin đồng ý **riêng** nếu ghi âm hoặc ghi màn hình; không có đồng ý thì ghi chép tay theo phương án đã thống nhất. Dùng mã P01, P02…; không ghi tên hay số điện thoại vào worksheet. Mẫu trong bài không thay quy trình đạo đức nghiên cứu của tổ chức bạn.

Thời lượng dự kiến 30–40 phút: giới thiệu 5, ba nhiệm vụ 15–20, hỏi lại 10, dự phòng 5. Vai trò: một moderator trung lập, một note-taker, observer (nếu có) không hướng dẫn trực tiếp.

### 5.2. Script mở đầu

> «Hôm nay chúng tôi muốn tìm hiểu trang đặt xe có hỗ trợ bạn thực hiện công việc không. Chúng tôi đang đánh giá giao diện, không chấm điểm bạn. Bạn có thể nói điều mình đang nghĩ, bỏ qua câu hỏi hoặc dừng bất kỳ lúc nào. Dữ liệu và tiền trong bản thử này đều là giả.»

Giải thích riêng giới hạn lab: nhân viên đã chuẩn bị sẵn mã khách C001, thành phố SG, loại xe máy. Participant không cần hiểu mã nội bộ. Không chỉ trước nút nào là đáp án.

### 5.3. Quan sát và hỗ trợ

Đọc prompt, rồi im lặng đủ lâu để participant tự làm. Khi cần, chỉ dùng câu trung lập: «Bạn đang nghĩ gì?», «Điều gì khiến bạn chọn cách đó?». Think-aloud giúp hiểu suy nghĩ nhưng có thể làm chậm thao tác; nếu dùng thì dùng nhất quán cho mọi phiên.

Note-taker ghi **hành động, thời điểm, lời nói** trước; diễn giải để sang cột khác. Nếu moderator phải gợi ý, ghi nội dung và thời điểm, outcome là *assisted*. Nếu lỗi môi trường (server chết, trang trắng) chặn nhiệm vụ, outcome là *blocked* — không phải lỗi của participant, cũng không phải lỗi usability. Không hứa hẹn cách sửa thiết kế trong lúc quan sát.

### 5.4. Kết thúc phiên

Hỏi lại các điểm chưa rõ, nhận phản hồi chung, và chủ động hỏi về những quan sát trái với giả thuyết ban đầu của đội. Ngay sau phiên, note-taker hoàn thiện worksheet khi trí nhớ còn mới.

### 5.5. Worksheet cho từng participant

Participant ID: ___ · nhóm kinh nghiệm: ___ · browser/version/device: ___ · hỗ trợ cần thiết: ___
Consent tham gia: ___ · recording riêng: ___ · build: ___ · ngày/giờ: ___ · moderator/note-taker: ___

| Task | Start/end | Outcome | Assistance/time | Error-message exposure | Rating 1–7 | Evidence/timestamp |
|---|---|---|---|---|---|---|
| U1 | Chưa chạy | | | N/A | | |
| U2 | Chưa chạy | | | | | |
| U3 | Chưa chạy | | | N/A | | |

| Timestamp | Quan sát thực tế | Lời nói nguyên văn (nếu được ghi) | Diễn giải / giả thuyết (tách riêng) |
|---|---|---|---|
| Chưa có | | | |

Các ô để trống có chủ đích. Không điền participant hư cấu để «minh hoạ cho đầy».

## 6. Metrics và cách không tự lừa mình

### 6.1. Quy tắc ghi số liệu

| Metric | Cách ghi | Giới hạn |
|---|---|---|
| Task success | Tách independent-success / assisted-success / failed / abandoned / blocked | Định nghĩa thành công trước khi quan sát |
| Time-on-task | Từ khi bắt đầu thao tác tới khi tuyên bố xong và kiểm kết quả, hoặc dừng; ghi thời điểm trợ giúp | Không trộn nhóm thành công với nhóm bỏ cuộc; không dùng thời gian của script Playwright |
| Errors / recovery | Thao tác sai và cách tự sửa | Không phải mọi lần quay lại đều là lỗi |
| Rating sau task | «Vừa rồi việc này dễ hay khó với bạn, từ 1 rất khó đến 7 rất dễ? Vì sao?» | Câu hỏi tự thiết kế; không gọi là SUS hay thang chuẩn |
| Finding frequency | Số người gặp / số người **có cơ hội** gặp | Mẫu nhỏ không đại diện tỷ lệ thị trường |

### 6.2. Mẫu số là nơi dễ sai nhất

**Ví dụ minh hoạ cách tính — không phải kết quả của study nào:** 5 người được giao U1; 3 tự hoàn thành, 1 cần gợi ý, 1 bị blocked vì server lab dừng giữa chừng. Báo cáo đúng:

> U1 (n = 5 được giao): independent 3, assisted 1, failed 0, abandoned 0, blocked 1 (lỗi môi trường, đã loại). Independent success trên số người có cơ hội thực hiện: 3/4.

Báo cáo sai là chỉ ghi «75%». Con số đó che mất mẫu nhỏ, che người được giúp, và che người bị loại.

Tương tự với exposure: nếu chỉ người *đã thấy* thông báo lỗi ở U2 mới có thể đánh giá thông báo đó, thì mẫu số của finding «không hiểu thông báo lỗi» là số người exposed, không phải toàn bộ cohort. Người sửa 0,1 → 5 trước khi bấm không thuộc mẫu số đó.

### 6.3. Expected và observed

| Hạng mục | Expected (thiết kế) | Observed |
|---|---|---|
| Số phiên | 6 phiên formative + 1 pilot | **0 — chưa chạy** |
| Success, time-on-task, rating | Ghi theo quy tắc 6.1 | **Chưa có dữ liệu** |
| Findings | Theo mẫu mục 8 | **Chưa có finding thực tế** |
| Supporting checks | 2/2 pass | 2/2 pass (mục 7.3) |

## 7. Supporting check tự động — hỗ trợ, không thay người

### 7.1. Vì sao vẫn có code

Trước khi mời người thật, cần chắc rằng phiên không bị phá bởi lỗi kỹ thuật: ô nhập không có nhãn, nút không phản hồi bàn phím, thông báo lỗi không xuất hiện. Hai check sau kiểm đúng mấy điều kiện đó, dựa trên R-UI-01, R-UI-03, R-UI-04. Chúng giống bước «dọn đường» trước study, không phải một phần của kết quả study.

### 7.2. Cấu hình và lệnh chạy

Config `examples/toi-noi-testing/usability.config.ts` dựng server PGlite riêng ở cổng 3117, một worker, không retry, không reuse server, giờ khởi động 09:00 VN và không mưa:

```ts
export default defineConfig({
  testDir: 'articles/system-testing-usability-test/e2e', workers: 1, retries: 0,
  reporter: [['list']], outputDir: 'reports/usability',
  use: { baseURL: 'http://127.0.0.1:3117', channel: 'chrome', headless: true,
    viewport: { width: 1280, height: 800 }, trace: 'retain-on-failure', screenshot: 'only-on-failure' },
  webServer: {
    command: 'npx tsx src/server.ts', url: 'http://127.0.0.1:3117/api/health',
    reuseExistingServer: false, timeout: 60_000,
    env: { PORT: '3117', NODE_ENV: 'test', TOINOI_TEST_API: '1', TOINOI_DB: 'pglite',
      TOINOI_RAIN: '', TOINOI_START_AT: '2026-11-04T09:00:00+07:00',
      TRIP_RATE_LIMIT_MAX: '5', TRIP_RATE_LIMIT_WINDOW_MS: '60000' },
  },
});
```

Từ thư mục lab, Node ≥ 22.12 và Chrome cài sẵn:

```sh
npm ci
npx playwright test --config usability.config.ts
npm run typecheck
```

Mỗi test reset dữ liệu và đặt lại đồng hồ trước khi chạy:

```ts
test.beforeEach(async ({ request }) => {
  expect((await request.post('/api/test/reset')).status()).toBe(200);
  expect((await request.post('/api/test/clock', { data: { now: '2026-11-04T09:00:00+07:00' } })).status()).toBe(200);
});
```

| ID | Basis / hành động | Oracle | Điều **chưa** chứng minh |
|---|---|---|---|
| US-A01 | R-UI-03/04: tìm input bằng label; focus ô voucher, Tab tới Thanh toán rồi «Báo giá», Enter | Focus đúng trong đoạn đó; 35.000 đ; region «Báo giá» có `aria-live="polite"`; «Đặt chuyến» enable | Người dùng tự tìm được; toàn hành trình chỉ bằng bàn phím; focus nhìn thấy rõ; screen reader đọc đúng |
| US-A02 | R-UI-01/03: nhập 0,1 km → báo giá → lỗi; sửa 5 km → báo giá lại | Alert chứa «Quãng đường», «Đặt chuyến» disabled; sau sửa alert rỗng, 35.000 đ, «Đặt chuyến» enable | Người dùng hiểu thông báo và tự chọn được cách sửa |

US-A01 dùng automation để đặt focus vào ô voucher, sau đó mới Tab:

```ts
await page.getByLabel('Mã giảm giá (không bắt buộc)', { exact: true }).focus();
await page.keyboard.press('Tab');
await expect(page.getByLabel('Thanh toán', { exact: true })).toBeFocused();
await page.keyboard.press('Tab');
await expect(page.getByRole('button', { name: 'Báo giá', exact: true })).toBeFocused();
await page.keyboard.press('Enter');
await expect(page.getByTestId('quote-total')).toHaveText('35.000 đ');
await expect(page.getByRole('region', { name: 'Báo giá', exact: true })).toHaveAttribute('aria-live', 'polite');
```

Vì điểm bắt đầu do code đặt, kết quả **không** được báo cáo là «hành trình bàn phím đạt». `getByLabel(..., { exact: true })` xác nhận liên kết nhãn cho đúng các ô được dùng, không audit toàn form. Kiểm `aria-live` là kiểm thuộc tính, không kiểm trải nghiệm nghe.

US-A02 kiểm vòng lỗi → sửa → báo giá lại:

```ts
await page.getByLabel('Quãng đường (km)', { exact: true }).fill('0.1');
await page.getByRole('button', { name: 'Báo giá', exact: true }).click();
await expect(page.getByRole('alert')).toContainText('Quãng đường');
await expect(page.getByRole('button', { name: 'Đặt chuyến', exact: true })).toBeDisabled();
await page.getByLabel('Quãng đường (km)', { exact: true }).fill('5');
await page.getByRole('button', { name: 'Báo giá', exact: true }).click();
await expect(page.getByRole('alert')).toBeEmpty();
await expect(page.getByTestId('quote-total')).toHaveText('35.000 đ');
```

Assertion «Đặt chuyến disabled» quan trọng ngang assertion có alert: nếu nút vẫn bấm được sau một báo giá lỗi, khách có thể đặt từ báo giá cũ. `toBeEmpty()` sau khi sửa bảo đảm thông báo cũ không đứng lại gây nhầm.

### 7.3. Kết quả thực đo

Expected: 2 check pass, 0 retry.

Observed, chạy lại ngày 13/11/2026 khi viết bài (Windows 11, Node v22.18.0, Playwright 1.64.1, Chrome cài sẵn, headless, 1280×800):

```text
Running 2 tests using 1 worker

  ok 1 …support.spec.ts:8:1 › US-A01 · nhãn, đoạn tab order và kích hoạt báo giá bằng Enter (884ms)
  ok 2 …support.spec.ts:23:1 › US-A02 · lỗi có thông báo, sửa dữ liệu và báo giá lại được (795ms)

  2 passed (9.8s)
```

Báo cáo gốc cùng ngày (Node v24.15.0) ghi 2 pass trong 5,3 giây (US-A01 740 ms, US-A02 508 ms); `npm run typecheck` pass. Chênh thời gian giữa hai lượt đến từ máy và tải khác nhau, không mang ý nghĩa gì về usability. Không chạy axe, không đánh giá WCAG đầy đủ, không có review trực quan bằng ảnh.

## 8. Từ quan sát tới finding và ưu tiên

### 8.1. Tách ba lớp

- **Observation:** người dùng bấm gì, nói gì, lúc nào.
- **Interpretation:** giả thuyết về cách họ hiểu.
- **Recommendation:** thay đổi cần thử và cách kiểm lại.

«Nút chưa đẹp» thiếu cả ba. Một finding có cấu trúc trông như sau — **ví dụ minh hoạ cấu trúc, không phải quan sát thực tế**: «Ở U1, P02 và P04 dừng sau khi bấm Báo giá và hỏi "tài xế sắp tới chưa?" (timestamp 02:10, 03:45). Giả thuyết: nhãn "Báo giá" đứng cạnh "Đặt chuyến" không đủ để phân biệt xem giá với gọi xe. Đề xuất: thêm dòng "Chưa gọi tài xế" dưới tổng tiền; retest với U1 ở vòng sau.»

### 8.2. Mẫu finding

Finding ID: ___ · nhiệm vụ: ___ · build: ___ · mô tả hành vi quan sát: ___
Evidence / participant IDs: ___ · số bị ảnh hưởng / số được quan sát: ___ · tác động: ___
Giả thuyết nguyên nhân: ___ · cách kiểm giả thuyết: ___ · đề xuất thay đổi: ___
Ưu tiên: blocker / nghiêm trọng / vừa / nhỏ — giải thích bằng tác động và khả năng tự hồi phục, không chỉ số lượt.
Owner: ___ · kế hoạch retest: ___ · trạng thái: chưa có finding thực tế.

### 8.3. Ưu tiên theo hậu quả

Một vấn đề chỉ xuất hiện ở một participant vẫn có thể là blocker nếu nó chặn nhiệm vụ hoặc làm khách mất tiền — ví dụ không biết mình đã huỷ và bị tính phí ở chuyến sau. Một vấn đề nhiều người gặp nhưng tự sửa được trong vài giây có thể chỉ là nhỏ. Tiêu chí gợi ý: hậu quả (mất chuyến, mất tiền, mất thời gian), khả năng tự hồi phục, mức lặp. Ghi độ chắc chắn kèm evidence; điểm số không thay được quyết định sản phẩm.

Tổng hợp theo task và nhóm người, giữ cả trường hợp trái chiều, báo raw counts trước phần trăm. Sau khi sửa, retest với nhiệm vụ tương đương; nhớ rằng người cũ đã học giao diện.

## 9. Mở rộng nâng cao

### 9.1. Người dùng công nghệ hỗ trợ

Tuyển người thực sự dùng screen reader, chỉ dùng bàn phím, hoặc phóng to màn hình, theo nhóm khách thực tế, và cho họ dùng thiết lập quen thuộc. Việc moderator tự nhắm mắt dùng screen reader không cho ra dữ liệu đại diện. W3C WAI khuyến nghị kết hợp người dùng với đánh giá theo tiêu chuẩn và công cụ; mỗi nguồn trả lời một phần khác nhau.

### 9.2. Learning effect và thứ tự nhiệm vụ

Người làm U1 học được vị trí các nút, nên U2 và U3 dễ hơn. Ghi thứ tự nhiệm vụ. Khi so hai phiên bản giao diện, đổi thứ tự phiên bản giữa các người (counterbalancing) hoặc dùng hai nhóm độc lập. Không đổi prompt giữa chừng rồi gộp số liệu như cùng điều kiện.

### 9.3. Bối cảnh thật của người đặt xe

Người đặt xe thường vội, đứng ngoài đường, mạng chập chờn, cầm điện thoại một tay. Câu hỏi liên quan bàn phím ảo hay thao tác chạm cần thiết bị thật; mô phỏng bối cảnh phải an toàn. Lab web với mã C001 không đại diện ứng dụng bản đồ production — giới hạn đó phải nằm trong báo cáo. [SYS-11](/articles/system-testing-compatibility-test) đã nói rõ mobile emulation không thay thiết bị thật; usability còn đòi hỏi nhiều hơn thế.

### 9.4. Định lượng và A/B

Muốn ước lượng tỷ lệ thành công hoặc so chênh lệch nhỏ giữa hai thiết kế thì cần thiết kế summative: cỡ mẫu, tiêu chí, phân nhóm, phân tích bất định. Sáu phiên formative không đủ để nói «có ý nghĩa thống kê». A/B trên sản phẩm thật đo hành vi ở quy mô lớn, nhưng cần event instrumentation đúng và phải phân biệt mục tiêu người dùng (đặt được xe đúng ý) với chỉ số click.

### 9.5. AI và «synthetic participants»

AI có thể giúp rà prompt xem có dẫn đáp án không, gợi ý các kiểu hiểu lầm có thể xảy ra, hoặc sắp xếp ghi chép đã được phép xử lý. Không được gán phản hồi của AI thành participant, không điền success rate bằng mô phỏng. Nhận xét của AI hay chuyên gia phải gắn nhãn *review* để không trộn với dữ liệu user study.

## 10. Anti-pattern, giới hạn và đánh đổi

### 10.1. Anti-pattern

- Đọc từng bước bấm nút rồi báo «người dùng thành công».
- Tuyển toàn đồng nghiệp đã quen sản phẩm.
- Moderator gợi ý nhưng không ghi, rồi tính là independent success.
- Chỉ hỏi «có dễ không?» mà không quan sát.
- Bỏ người blocked khỏi mẫu số mà không giải thích.
- Gọi thời gian chạy Playwright là time-on-task.
- Điền worksheet bằng participant hư cấu cho «đủ bảng».

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn bằng chứng của bài</span>
<p>Participant = 0. Không có success rate, rating, time-on-task hay finding nào rút ra từ người dùng thật. Hai supporting check không chứng minh giao diện dễ dùng, không phải accessibility audit, không phải UAT. Thứ đã hoàn thành: bài học, bộ study kit dùng được, và hai check được chạy lại pass. Muốn kết luận về usability, cần pilot, tuyển người phù hợp, xin đồng ý và chạy phiên thật.</p>
</div>

### 10.2. Đánh đổi chi phí và phạm vi

Một vòng sáu phiên tốn khoảng một ngày quan sát cộng thời gian tuyển và tổng hợp — ít hơn nhiều so với chi phí một luồng đặt xe khiến khách bỏ đi. Nhưng không cần study cho mọi thay đổi: đổi màu chữ không cần, đổi cách hiển thị phí huỷ thì nên có. Dùng expert review để lọc vấn đề hiển nhiên trước, dành phiên với người thật cho câu hỏi mà đội thực sự không biết đáp án.

## 11. Bài tập và câu hỏi tự kiểm

### Bài tập

**Bài 1 (cơ bản).** Sửa prompt «Bấm Huỷ chuyến và xem phí» thành một nhiệm vụ trung lập.

<details>
<summary><b>Gợi ý đáp án</b></summary>
<p>«Bạn đổi kế hoạch khi chưa có tài xế nhận; hãy dừng chuyến và cho biết có khoản phí nào không.» Prompt nêu bối cảnh và mục tiêu, không dùng tên nút, không chỉ thứ tự bước. Tiêu chí thành công (huỷ, thấy đã huỷ, phí 0 đ theo R-CANCEL-01) chỉ moderator giữ.</p>
</details>

**Bài 2 (trung bình).** Ở U2, participant sửa 0,1 thành 5 rồi mới bấm «Báo giá» và thấy 35.000 đ. Phiên này có chứng minh thông báo lỗi dễ hiểu không?

<details>
<summary><b>Gợi ý đáp án</b></summary>
<p>Không. Họ đạt mục tiêu sửa dữ liệu, nhưng chưa từng thấy thông báo lỗi: ghi «error-message exposure = no» và không tính họ vào mẫu số của câu hỏi về thông báo lỗi. Không ép họ quay lại tạo lỗi chỉ để đúng script.</p>
</details>

**Bài 3 (trung bình).** Ở U3, moderator nói «nút huỷ ở dưới cùng» và participant hoàn tất. Ghi outcome thế nào?

<details>
<summary><b>Gợi ý đáp án</b></summary>
<p>Assisted-success, kèm timestamp và nội dung gợi ý. Không gộp vào independent-success. Gợi ý đó cũng là một tín hiệu: có thể nút huỷ khó tìm — ghi thành quan sát để tổng hợp, chưa phải kết luận.</p>
</details>

**Bài 4 (nâng cao).** Đội muốn so phiên bản A (hiện tại) với phiên bản B (thêm dòng «Chưa gọi tài xế» dưới tổng tiền), mỗi người làm U1 trên cả hai. Thiết kế phiên thế nào để kết quả không bị learning effect làm sai?

<details>
<summary><b>Gợi ý đáp án</b></summary>
<p>Đổi thứ tự A/B giữa các participant (một nửa A trước, một nửa B trước), hoặc dùng hai nhóm độc lập. Dùng nhiệm vụ tương đương nhưng không giống hệt (ví dụ 5 km và 8 km, oracle tính từ SPEC). Giữ prompt, thang rating và quy tắc think-aloud nhất quán. Báo raw counts theo thứ tự trình bày; với mẫu formative nhỏ, không kết luận ý nghĩa thống kê, chỉ so sánh định tính và quyết định có đáng chạy study lớn hơn không.</p>
</details>

**Bài 5 (nâng cao).** Viết lại báo cáo sai «U2: 80% thành công» thành dạng đúng, giả sử (minh hoạ) 5 người được giao, 2 tự sửa trước khi thấy lỗi, 2 thấy lỗi và tự sửa, 1 thấy lỗi và bỏ cuộc.

<details>
<summary><b>Gợi ý đáp án</b></summary>
<p>«U2 (n = 5, dữ liệu minh hoạ): đạt mục tiêu sửa dữ liệu 4/5, abandoned 1. Đánh giá thông báo lỗi: exposed 3; trong đó tự sửa sau khi đọc 2, bỏ cuộc 1. 2 người sửa trước submit không thuộc mẫu số này.» Con số 80% gộp hai câu hỏi khác nhau và che đi người bỏ cuộc sau khi thấy thông báo — đúng người mà finding cần chú ý nhất.</p>
</details>

### Câu hỏi tự kiểm

**Câu 1.** 2/2 supporting check pass có nghĩa usability đạt?

<details>
<summary><b>Đáp án</b></summary>
<p>Không. Chúng kiểm nhãn, một đoạn Tab do code đặt điểm bắt đầu, và vòng lỗi–sửa trong DOM. Không có người dùng nào được quan sát.</p>
</details>

**Câu 2.** Study kit có sáu ô participant. Vậy đội đã nghiên cứu sáu người?

<details>
<summary><b>Đáp án</b></summary>
<p>Không. Sáu là số đề xuất cho vòng formative. Hiện participant = 0; bảng để trống có chủ đích.</p>
</details>

**Câu 3.** Vì sao không gộp blocked vào failed?

<details>
<summary><b>Đáp án</b></summary>
<p>Blocked là lỗi môi trường, không phải người dùng thất bại vì giao diện. Gộp lại làm usability trông tệ hơn thực tế và che lỗi setup cần sửa. Báo riêng với lý do, và nêu rõ khi loại khỏi mẫu số.</p>
</details>

**Câu 4.** Thuộc tính `aria-live="polite"` có mặt trong DOM. Có kết luận được screen reader đọc kết quả báo giá đúng lúc không?

<details>
<summary><b>Đáp án</b></summary>
<p>Không. Thuộc tính này là điều kiện cần theo R-UI-03, không phải bằng chứng trải nghiệm. Cần kiểm với screen reader thật và, tốt hơn, với người dùng screen reader thường xuyên.</p>
</details>

**Câu 5.** Một finding chỉ có 1/6 người gặp có nên xếp blocker?

<details>
<summary><b>Đáp án</b></summary>
<p>Có thể, nếu nó chặn nhiệm vụ hoặc gây mất tiền và người dùng không tự hồi phục được. Ưu tiên theo hậu quả và khả năng hồi phục, tần suất chỉ là một yếu tố.</p>
</details>

## 12. Tài liệu tham khảo

- GOV.UK Service Manual — *Using moderated usability testing*: https://www.gov.uk/service-manual/user-research/using-moderated-usability-testing — cơ sở cho quan sát nhiệm vụ và điều phối phiên; bộ prompt và scoring Tới Nơi là thiết kế riêng của bài. Tra cứu 13/11/2026.
- W3C WAI — *Involving Users in Evaluating Web Accessibility*: https://www.w3.org/WAI/test-evaluate/involving-users/ — kết hợp người dùng có nhu cầu truy cập với đánh giá tiêu chuẩn. Tra cứu 13/11/2026.
- W3C WAI — *Evaluating Web Accessibility Overview*: https://www.w3.org/WAI/test-evaluate/ — vì sao cần cả người đánh giá và công cụ. Tra cứu 13/11/2026.
- Playwright — *Locators* (`getByLabel`, `getByRole`): https://playwright.dev/docs/locators — công cụ cho supporting check, không phải phương pháp usability. Tra cứu 13/11/2026.
- Lab Tới Nơi: `SPEC.md` (R-UI-01, R-UI-03, R-UI-04, R-FARE-02, R-CANCEL-01, R-CANCEL-06) và supporting check — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-usability-test
- Bài liên quan: [Non-functional testing tổng quan](/articles/non-functional-testing-tong-quan) · [Acceptance testing tổng quan](/articles/acceptance-testing-tong-quan) · [SYS-05 · System Test](/articles/system-testing-system-test) · [SYS-11 · Compatibility Test](/articles/system-testing-compatibility-test) · [SYS-15 · UAT](/articles/system-testing-uat)

Bài tiếp theo: [SYS-13 · Data Integrity Test](/articles/system-testing-data-integrity-test).
