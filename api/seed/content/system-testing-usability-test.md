---
slug: system-testing-usability-test
title: "Usability Test: người dùng có tự hoàn thành được việc họ cần không?"
summary: "Usability testing cho trang đặt xe Tới Nơi: phân biệt với functional, accessibility và UAT; phương pháp formative có điều phối; nhiệm vụ không dẫn đáp án; ghi outcome, mẫu số, time-on-task trung thực. Kèm study kit và hai supporting check Playwright — chưa phải nghiên cứu người dùng thật."
category: testing
level: intermediate
tags: [testing, sys-12, usability-testing, user-research, accessibility, task-success, time-on-task, playwright, toi-noi]
language: vi
---

API báo giá đúng 35.000 đ, nút «Đặt chuyến» chạy tốt trên Chrome và Edge, các bài functional đều xanh. Vậy mà người mới mở trang có thể bấm «Báo giá» rồi ngồi chờ tài xế vì tưởng đã gọi xe, hoặc thấy dòng lỗi đỏ mà không biết sửa ô nào. Script automation luôn bấm đúng nút theo kịch bản nên không gặp những nhầm lẫn đó. Bài này trả lời: **làm sao biết người dùng thật có tự hoàn thành được việc họ cần, và thu bằng chứng về điều đó mà không tự lừa mình bằng con số đẹp.**

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · SYS-12</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>Kiểm thử hệ thống Tới Nơi · Phần 2</td></tr>
<tr><th scope="row">Cấp độ</th><td>Intermediate — tester, BA, designer, product; không cần viết code để tổ chức study</td></tr>
<tr><th scope="row">Test level / hướng kiểm</th><td>Đặc tính phi chức năng (usability) ở cấp hệ thống; phương pháp formative, moderated; accessibility chỉ ở mức kiểm bổ trợ</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/system-testing-compatibility-test">SYS-11 · Compatibility Test</a> · <a href="/articles/non-functional-testing-tong-quan">Non-functional testing tổng quan</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Human testing: study kit gồm kế hoạch phiên, script mở đầu, ba task card U1–U3, quy tắc ghi số liệu, worksheet, mẫu finding. Code bổ trợ: 2 check Playwright US-A01, US-A02 (R-UI-01, R-UI-03, R-UI-04) — <code>examples/toi-noi-testing/articles/system-testing-usability-test/</code> · Minh hoạ: CurlStudio với collection <code>examples/toi-noi-testing/curlstudio/</code></td></tr>
<tr><th scope="row">Nguồn chính</th><td>GOV.UK Service Manual (moderated usability testing); W3C WAI (Involving Users in Evaluating Web Accessibility); <code>SPEC.md</code> của lab</td></tr>
</tbody>
</table>
</div>

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Trạng thái bằng chứng</span>
<p><b>Chưa có phiên nghiên cứu nào với người tham gia thật</b> (participant = 0). Bài không có success rate, time-on-task, rating hay finding nào rút ra từ người dùng thật. Mọi con số và trích dẫn người dùng trong bài đều được gắn nhãn <i>minh hoạ</i>. Thứ duy nhất đã chạy là hai supporting check tự động; chúng không đo usability, không phải accessibility audit hay UAT. Muốn kết luận về usability, cần pilot, tuyển người phù hợp, xin đồng ý và chạy phiên thật.</p>
</div>

## 1. Bài toán và mục tiêu

### 1.1. Tình huống

Trang khách `/` của lab Tới Nơi có một form (mã khách, dịch vụ, thành phố, loại xe, quãng đường, voucher, phương thức thanh toán) và ba nút: «Báo giá» hiện tổng tiền, «Đặt chuyến» tạo chuyến từ báo giá, «Huỷ chuyến» hiện phí huỷ (R-UI-01). SYS-03 tới SYS-05 đã kiểm chức năng: cước, voucher, trạng thái, hành trình hai vai trò.

Những bài đó không trả lời được ba câu hỏi:

- Người mới có **phân biệt được báo giá với đặt chuyến** không? Hai nút nằm cạnh nhau; ai tưởng «Báo giá» là gọi xe sẽ chờ mãi rồi bỏ đi.
- Khi nhập quãng đường không hợp lệ, người dùng có **hiểu thông báo và tự sửa** được không? Theo R-FARE-02, 0,1 km bị từ chối vì nhỏ hơn 0,5 km; hệ thống trả lỗi đúng, nhưng thông báo có đủ rõ?
- Sau khi huỷ, họ có **xác nhận được đã huỷ và biết có phí hay không**? R-CANCEL-01 nói huỷ khi đang tìm tài xế thì không mất phí; giao diện có nói đủ rõ không?

Test Playwright trả lời «hệ thống có xử lý đúng khi được thao tác đúng không», không phải «một người chưa từng thấy trang này sẽ thao tác thế nào».

### 1.2. Đối tượng và điều kiện tiên quyết

Cần biết luồng đặt xe Tới Nơi ở mức người dùng (xem [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi)) và phân biệt functional với non-functional. Không cần viết code để tổ chức nghiên cứu; phần Playwright dành cho ai muốn tái hiện supporting check.

### 1.3. Mục tiêu đo được

Sau bài, bạn có thể:

1. Chuyển câu hỏi sản phẩm thành nhiệm vụ trung lập, không đọc tên nút hay các bước.
2. Chuẩn bị một phiên moderated: đồng ý tham gia, vai trò, script mở đầu, quy tắc hỗ trợ.
3. Ghi hành vi tách khỏi suy diễn, gán đúng outcome (independent, assisted, failed, abandoned, blocked).
4. Tính metric có mẫu số rõ ràng, giải thích ai bị loại và vì sao.
5. Viết finding có bằng chứng, tác động, giả thuyết và cách kiểm lại; ưu tiên theo hậu quả, không chỉ theo tần suất.
6. Nói rõ supporting check tự động chứng minh gì và không chứng minh gì.

## 2. Khái niệm và những điều dễ nhầm

### 2.1. Usability testing là gì

Usability testing là **quan sát người dùng đại diện thực hiện nhiệm vụ cụ thể trong bối cảnh cụ thể** để tìm trở ngại và đánh giá mức họ đạt mục tiêu. Ba khía cạnh: hiệu quả (có hoàn thành không), công sức (mất bao lâu, sai mấy lần, cần giúp không) và trải nghiệm người dùng phản hồi.

Mấu chốt là chữ *quan sát*: một người nói «giao diện đẹp» không chứng minh họ đặt được xe; một người im lặng bấm nhầm ba lần rồi tự sửa cho nhiều thông tin hơn. Ý kiến chỉ có giá trị khi đặt cạnh hành vi.

### 2.2. Bảng phân biệt

| Hoạt động | Trả lời câu hỏi nào | Không thay thế được |
|---|---|---|
| Functional test | Hệ thống xử lý đúng khi được thao tác theo test? | Người mới có tự tìm được đường đi không |
| Usability study | Người dùng đại diện hiểu và hoàn thành nhiệm vụ ra sao? | Kiểm tra mọi quy tắc kỹ thuật |
| Accessibility evaluation | Người có nhu cầu truy cập khác nhau dùng được không, đạt tiêu chí nào? | Toàn bộ usability; chứng nhận chỉ bằng scanner |
| Expert / heuristic review | Chuyên gia thấy vấn đề gì theo nguyên tắc thiết kế? | Quan sát participant thật |
| UAT | Đại diện nghiệp vụ có chấp nhận giải pháp theo tiêu chí? | Trải nghiệm của mọi nhóm khách |

Accessibility và usability giao nhau (người chỉ dùng bàn phím mà không tới được nút «Báo giá» thì không hoàn thành task), nhưng `aria-live` có trong DOM không chứng minh screen reader đọc đúng lúc, và một study vài người không chứng minh tuân thủ tiêu chuẩn accessibility. Nghiệp vụ cũng có thể ký UAT cho một giao diện mà khách mới vẫn gặp khó (xem [Acceptance testing tổng quan](/articles/acceptance-testing-tong-quan) và [SYS-15 · UAT](/articles/system-testing-uat)).

### 2.3. Ưu điểm và nhược điểm

Usability study tìm ra loại lỗi suite tự động không thấy: người dùng hiểu sai giao diện. Một phiên 30 phút có thể lộ ra nhãn nút gây hiểu lầm mà hàng trăm test xanh bỏ qua.

Đổi lại, phương pháp này tốn công tuyển người, cần kỹ năng điều phối, mẫu nhỏ nên không cho suy luận thống kê mạnh, và dễ thiên lệch: moderator vô tình gợi ý, participant muốn làm hài lòng người hỏi, đồng nghiệp đã quen sản phẩm.

## 3. Chọn phương pháp theo câu hỏi

### 3.1. Moderated hay unmoderated, formative hay summative

| Lựa chọn | Phù hợp khi | Rủi ro |
|---|---|---|
| Moderated | Cần hỏi lại lý do, quan sát cách xử lý tình huống mới | Moderator ảnh hưởng hành vi; tốn người |
| Unmoderated | Muốn nhiều phiên hơn với chi phí thấp | Khó phân biệt bỏ cuộc với lỗi setup nếu thiết kế thiếu rõ |
| Formative | Tìm vấn đề để cải tiến trong vòng thiết kế | Không cho phép ước lượng tỷ lệ cho toàn bộ người dùng |
| Summative | Đánh giá theo tiêu chí định trước, so sánh phiên bản | Cần cỡ mẫu và phương pháp phân tích phù hợp hơn |

Bài chọn **formative, moderated** cho ba câu hỏi ở mục 1.1, phạm vi cố ý hẹp: chỉ trang khách của lab, không gồm trải nghiệm toàn nền tảng, bản đồ, ứng dụng tài xế, GPS hay thanh toán thật.

### 3.2. Bao nhiêu người, là ai

Study kit đề xuất **sáu người cho vòng học ban đầu**: ba người mới và ba người đã quen ứng dụng đặt xe tương tự. Sáu không phải định luật «đủ người», không bảo đảm phát hiện mọi vấn đề và không cho phép nói «X% khách hàng sẽ thành công»; đó là ngân sách hợp lý cho một vòng formative.

Tuyển theo nhóm sử dụng dự kiến, kinh nghiệm, thiết bị và nhu cầu hỗ trợ. Người trong đội đã xây giao diện nhớ đường đi nên không đại diện người mới. Nên pilot, kể cả với đồng nghiệp, nhưng giữ dữ liệu pilot tách riêng.

### 3.3. Test basis, rủi ro, entry criteria và exit criteria

Test basis: R-UI-01 (chức năng trang khách), R-UI-03 (khả năng tiếp cận: mọi ô có nhãn, lỗi dùng `role="alert"`, kết quả cập nhật trong vùng `aria-live`), R-UI-04 (định dạng tiền «35.000 đ»), R-FARE-02 (khoảng quãng đường hợp lệ) và R-CANCEL-01 (phí huỷ theo trạng thái). Usability còn một basis ngoài SPEC: **mục tiêu của người dùng**. SPEC nói nút huỷ phải hiện phí, không nói người dùng phải hiểu số đó là phí hay hoàn tiền.

| Rủi ro | Hậu quả với Tới Nơi | Nhiệm vụ quan sát |
|---|---|---|
| Nhầm báo giá với đặt chuyến | Khách chờ không có xe, bỏ đi, mất chuyến | U1 |
| Không hiểu thông báo lỗi | Khách bỏ cuộc ở bước nhập liệu | U2 |
| Không chắc đã huỷ / sợ bị trừ tiền | Gọi tổng đài, mất niềm tin | U3 |

**Entry criteria:** câu hỏi nghiên cứu và đối tượng đã thống nhất; build, browser, dữ liệu giả đã chốt; tài liệu đồng ý và quy tắc lưu dữ liệu sẵn sàng; đã pilot và sửa prompt.
**Exit criteria của vòng:** đủ số phiên dự kiến hoặc đã thấy cùng một trở ngại lặp lại đủ để quyết định sửa; mọi phiên có worksheet đầy đủ; finding được tổng hợp theo task, có bằng chứng. Hiện entry criteria chưa đạt: chưa pilot, chưa tuyển.

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

Prompt «Hãy bấm Báo giá, rồi tìm Tổng tiền» đo khả năng làm theo chỉ dẫn, không đo giao diện. Prompt tốt nêu **mục tiêu và bối cảnh** bằng ngôn ngữ của người dùng: «Bạn định đi xe máy 5 km; hãy tìm số tiền cần trả, chưa yêu cầu tài xế.» Khi đó ta thấy người dùng *nghĩ* nút nào làm gì.

Ba quy tắc:

- Không dùng từ trùng nhãn nút nếu có thể tránh («dừng chuyến» thay vì «Huỷ chuyến»).
- Không gợi ý thứ tự bước.
- Ghi tiêu chí thành công cho moderator, **không đọc** cho participant.

### 4.2. Setup có chủ đích và giới hạn của nó

Được chuẩn bị dữ liệu để tập trung vào một nhiệm vụ, nhưng phải ghi rõ hệ quả. U3 dùng một chuyến `searching` do moderator tạo trước, nên U3 **không** chứng minh participant tự đặt được chuyến. Ở U2, người tự sửa 0,1 thành 5 trước khi bấm thì không bị ép tạo lỗi; ghi «error-message exposure = no» (xem mục 6.2).

### 4.3. Ba task card

Bảng dưới lấy nguyên từ study kit. Cột prompt đọc cho participant; các cột khác chỉ moderator xem.

| ID | Prompt đọc cho participant | Setup riêng | Success không trợ giúp | Giới hạn quan sát |
|---|---|---|---|---|
| U1 | «Bạn định đi xe máy 5 km tại TP.HCM. Hãy tìm số tiền cần trả; chưa yêu cầu tài xế.» | Trang đầu, dữ liệu giả đã nạp | Báo đúng 35.000 đ và dừng trước booking | Tối đa 4 phút trước khi hỏi có muốn tiếp tục/nhận gợi ý |
| U2 | «Bạn đang ở màn hình đặt xe và ô quãng đường đang ghi 0,1 km. Hãy tìm cách lấy báo giá cho chuyến 5 km.» | Distance 0.1, chưa submit | Gặp/nhận biết lỗi khi áp dụng; sửa 5, báo giá 35.000 và giải thích đã sửa gì | Tối đa 4 phút; sửa trước submit thì không ép tạo lỗi, ghi đường đi |
| U3 | «Bạn đã yêu cầu chuyến nhưng đổi kế hoạch khi chưa có tài xế nhận. Hãy dừng chuyến và cho biết có khoản phí nào không.» | Seed sạch, moderator tạo chuyến `searching` riêng trước nhiệm vụ | Huỷ, xác nhận đã huỷ và phí 0 đ | Tối đa 4 phút; không nhắc tên nút huỷ |

Oracle cho tiêu chí thành công lấy từ SPEC, không từ giao diện: 35.000 đ là cước xe máy 5 km giờ thường tại SG, không mưa, không voucher (R-FARE-01…08, đã tính ở [SYS-01 · Smoke Test](/articles/system-testing-smoke-test)); 0 đ là phí huỷ khi chuyến còn `searching` (R-CANCEL-01, lý do `searching` theo R-CANCEL-06).

## 5. Chạy phiên quan sát từng bước

### 5.1. Chuẩn bị và đồng ý tham gia

Chốt câu hỏi, đối tượng, build, browser/version/viewport, dữ liệu giả. Đặt lại seed và giờ 09:00 VN trước mỗi nhiệm vụ độc lập; không nối thanh toán thật.

Giải thích cho participant: mục đích, tính tự nguyện, quyền bỏ câu hỏi hoặc dừng, dữ liệu nào được lưu, ai xem, ở đâu, khi nào xoá. Xin đồng ý **riêng** nếu ghi âm hoặc ghi màn hình; không có đồng ý thì ghi chép tay theo phương án đã thống nhất. Dùng mã P01, P02…, không ghi tên hay số điện thoại vào worksheet. Mẫu này không thay quy trình đạo đức nghiên cứu của tổ chức.

Thời lượng dự kiến 30–40 phút: giới thiệu 5, ba nhiệm vụ 15–20, hỏi lại 10, dự phòng 5. Vai trò: một moderator trung lập, một note-taker; observer (nếu có) không hướng dẫn trực tiếp.

### 5.2. Script mở đầu

> «Hôm nay chúng tôi muốn tìm hiểu trang đặt xe có hỗ trợ bạn thực hiện công việc không. Chúng tôi đang đánh giá giao diện, không chấm điểm bạn. Bạn có thể nói điều mình đang nghĩ, bỏ qua câu hỏi hoặc dừng bất kỳ lúc nào. Dữ liệu và tiền trong bản thử này đều là giả.»

Nói riêng giới hạn lab: mã khách C001, thành phố SG, loại xe máy đã được chuẩn bị sẵn; participant không cần hiểu mã nội bộ. Không chỉ trước nút nào là đáp án.

### 5.3. Quan sát và hỗ trợ

Đọc prompt rồi im lặng để participant tự làm. Khi cần, chỉ dùng câu trung lập: «Bạn đang nghĩ gì?», «Điều gì khiến bạn chọn cách đó?». Think-aloud giúp hiểu suy nghĩ nhưng làm chậm thao tác; nếu dùng thì dùng cho mọi phiên.

Note-taker ghi **hành động, thời điểm, lời nói**; diễn giải để cột khác. Moderator phải gợi ý thì ghi nội dung, thời điểm; outcome là *assisted*. Nếu lỗi môi trường (server chết, trang trắng) chặn nhiệm vụ, outcome là *blocked* — không phải lỗi của participant hay lỗi usability. Không hứa cách sửa thiết kế trong lúc quan sát.

### 5.4. Kết thúc phiên

Hỏi lại điểm chưa rõ, nhận phản hồi chung, chủ động hỏi về quan sát trái với giả thuyết của đội. Note-taker hoàn thiện worksheet ngay sau phiên.

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

Các ô để trống có chủ đích; không điền participant hư cấu để «minh hoạ cho đầy».

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

Báo cáo sai là chỉ ghi «75%»: con số đó che mẫu nhỏ, người được giúp và người bị loại.

Tương tự với exposure: mẫu số của finding «không hiểu thông báo lỗi» ở U2 là số người *đã thấy* thông báo (exposed), không phải toàn bộ cohort.
### 6.3. Mong đợi và thực tế

| Hạng mục | Mong đợi (thiết kế) | Thực tế |
|---|---|---|
| Số phiên | 6 phiên formative + 1 pilot | **0 — chưa chạy** |
| Success, time-on-task, rating | Ghi theo quy tắc 6.1 | **Chưa có dữ liệu** |
| Findings | Theo mẫu mục 8 | **Chưa có finding thực tế** |
| Supporting checks | 2/2 pass | 2/2 pass (mục 7.3) |

## 7. Supporting check tự động — hỗ trợ, không thay người

### 7.1. Vì sao vẫn có code

Trước khi mời người thật, cần chắc phiên không bị phá bởi lỗi kỹ thuật: ô nhập thiếu nhãn, nút không phản hồi bàn phím, thông báo lỗi không hiện. Hai check sau kiểm các điều kiện đó (R-UI-01, R-UI-03, R-UI-04) — bước «dọn đường», không phải kết quả study.

### 7.2. Cấu hình và lệnh chạy

Config `examples/toi-noi-testing/usability.config.ts` dựng server PGlite riêng ở cổng 3117, một worker, không retry, giờ khởi động 09:00 VN, không mưa:

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

US-A01 dùng automation đặt focus vào ô voucher rồi mới Tab:

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

Vì điểm bắt đầu do code đặt, **không** báo cáo kết quả là «hành trình bàn phím đạt». `getByLabel(..., { exact: true })` chỉ xác nhận liên kết nhãn của các ô được dùng, không audit toàn form. Kiểm `aria-live` là kiểm thuộc tính, không kiểm trải nghiệm nghe.

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

Assertion «Đặt chuyến disabled» quan trọng ngang assertion có alert: nút còn bấm được sau báo giá lỗi thì khách có thể đặt từ báo giá cũ. `toBeEmpty()` sau khi sửa bảo đảm thông báo cũ không còn gây nhầm.

### 7.3. Kết quả thực đo

**Mong đợi:** 2 check pass, 0 retry.

**Thực tế — chạy lại khi biên tập (13/11/2026)** (Windows 11, Node v22.18.0, Playwright 1.64.1, Chrome cài sẵn, headless, 1280×800):

```text
Running 2 tests using 1 worker

  ok 1 …support.spec.ts:8:1 › US-A01 · nhãn, đoạn tab order và kích hoạt báo giá bằng Enter (884ms)
  ok 2 …support.spec.ts:23:1 › US-A02 · lỗi có thông báo, sửa dữ liệu và báo giá lại được (795ms)

  2 passed (9.8s)
```

**Thực tế — báo cáo gốc (13/11/2026)** (Node v24.15.0): 2 pass trong 5,3 giây (US-A01 740 ms, US-A02 508 ms); `npm run typecheck` pass. Chênh thời gian do máy và tải khác nhau, không nói gì về usability. Không chạy axe, không đánh giá WCAG đầy đủ, không review trực quan bằng ảnh.

### 7.4. Minh hoạ với CurlStudio

US-A02 chỉ đòi alert *chứa* chữ «Quãng đường». Nội dung đầy đủ của thông điệp mà API trả cho task U2 có thể xem thêm bằng API client, với collection đã giới thiệu ở [SYS-01 · Smoke Test](/articles/system-testing-smoke-test). Trong folder «SYS-12 · Thông điệp lỗi cho người dùng», mở request «UX · Quãng đường 0,1 km» (POST `{{baseUrl}}/api/quotes`, quãng đường 0,1 km) và bấm **Gửi**.

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/system-testing/sys-12/01-thong-diep-loi.webp" alt="CurlStudio mở request UX · Quãng đường 0,1 km trong folder SYS-12 · Thông điệp lỗi cho người dùng, gửi POST tới {{baseUrl}}/api/quotes. Thanh response ghi Trạng thái 422 Unprocessable Entity, thời gian 4.9 ms, kết quả test 4/4. Thân JSON có error.code là DISTANCE_OUT_OF_RANGE và error.message là: Quãng đường phải từ 0.5 đến 100 km (sau khi làm tròn 0,1 km)." width="2000" height="1125" loading="lazy"></div>
<figcaption><b>Hình 2 · Thông điệp lỗi API cho task U2.</b> Response 422 với mã <code>DISTANCE_OUT_OF_RANGE</code>, bốn test của request đều đạt. Chú ý cách viết số trong <code>message</code>: «0.5» dùng dấu chấm, còn «0,1» trong cùng câu dùng dấu phẩy.</figcaption>
</figure>

Đọc ảnh: trạng thái **422 Unprocessable Entity**, **Kết quả test 4/4**. Bốn test của request kiểm HTTP 422, mã lỗi `DISTANCE_OUT_OF_RANGE`, thông điệp nêu khoảng hợp lệ «0.5 đến 100 km», và thông điệp không lộ chi tiết kỹ thuật. Số 4,9 ms trên thanh response là thời gian phản hồi của API — không phải time-on-task, đúng như anti-pattern ở mục 10.1.

**Ảnh này chứng minh gì.** Ở tầng API, thông điệp cho U2 có nội dung người dùng cần để tự sửa: nói rõ khoảng hợp lệ và giá trị hệ thống đã nhận, không kèm stack hay tên nội bộ. Đây là một kiểm bổ trợ thêm vào bước «dọn đường» của mục 7.1, cùng loại với US-A01 và US-A02.

**Ảnh này không chứng minh gì.** Nó **không** chứng minh người dùng hiểu thông điệp hay tự sửa được: chưa có user study nào, participant vẫn bằng 0, và 4/4 không đi vào bảng kết quả ở mục 6.3. Ảnh cũng không cho biết trang khách hiển thị nguyên văn thông điệp này hay không; muốn biết thì nhìn trên trang, trong chính phiên quan sát.

Ảnh còn gợi ý một điểm đáng mang vào phiên. Thông điệp viết «0.5» bằng dấu chấm, trong khi cách viết số kiểu Việt Nam dùng dấu phẩy cho phần thập phân và dấu chấm cho hàng nghìn — như «35.000 đ» (R-UI-04), như «0,1» ngay trong cùng câu và trong prompt U2. Đây là **giả thuyết để quan sát**, chưa phải finding: ghi vào cột «Diễn giải / giả thuyết» của worksheet, rồi để hành vi của người tham gia đã thấy thông điệp (exposed) xác nhận hoặc bác bỏ — ví dụ họ gõ gì vào ô quãng đường ngay sau khi đọc lỗi.

## 8. Từ quan sát tới finding và ưu tiên

### 8.1. Tách ba lớp

- **Observation:** người dùng bấm gì, nói gì, lúc nào.
- **Interpretation:** giả thuyết về cách họ hiểu.
- **Recommendation:** thay đổi cần thử và cách kiểm lại.

«Nút chưa đẹp» thiếu cả ba. Một finding có cấu trúc trông như sau — **ví dụ minh hoạ cấu trúc, không phải quan sát thực tế**: «Ở U1, P02 và P04 dừng sau khi bấm Báo giá và hỏi ‹tài xế sắp tới chưa?› (timestamp 02:10, 03:45). Giả thuyết: nhãn ‹Báo giá› đứng cạnh ‹Đặt chuyến› không đủ để phân biệt xem giá với gọi xe. Đề xuất: thêm dòng ‹Chưa gọi tài xế› dưới tổng tiền; retest với U1 ở vòng sau.»

### 8.2. Mẫu finding

Finding ID: ___ · nhiệm vụ: ___ · build: ___ · mô tả hành vi quan sát: ___
Evidence / participant IDs: ___ · số bị ảnh hưởng / số được quan sát: ___ · tác động: ___
Giả thuyết nguyên nhân: ___ · cách kiểm giả thuyết: ___ · đề xuất thay đổi: ___
Ưu tiên: blocker / nghiêm trọng / vừa / nhỏ — giải thích bằng tác động và khả năng tự hồi phục, không chỉ số lượt.
Owner: ___ · kế hoạch retest: ___ · trạng thái: chưa có finding thực tế.

### 8.3. Ưu tiên theo hậu quả

Vấn đề chỉ một participant gặp vẫn có thể là blocker nếu chặn nhiệm vụ hoặc làm khách mất tiền — ví dụ không biết mình đã huỷ và bị tính phí ở chuyến sau. Vấn đề nhiều người gặp nhưng tự sửa trong vài giây có thể chỉ là nhỏ. Tiêu chí gợi ý: hậu quả (mất chuyến, mất tiền, mất thời gian), khả năng tự hồi phục, mức lặp. Ghi độ chắc chắn kèm bằng chứng.

Tổng hợp theo task và nhóm người, giữ trường hợp trái chiều, báo raw counts trước phần trăm. Sau khi sửa, retest bằng nhiệm vụ tương đương; người cũ đã học giao diện.

## 9. Mở rộng nâng cao

### 9.1. Người dùng công nghệ hỗ trợ

Tuyển người thực sự dùng screen reader, chỉ dùng bàn phím hoặc phóng to màn hình, với thiết lập quen thuộc của họ. Moderator tự nhắm mắt dùng screen reader không cho dữ liệu đại diện; W3C WAI khuyến nghị kết hợp người dùng với đánh giá theo tiêu chuẩn và công cụ.

### 9.2. Learning effect và thứ tự nhiệm vụ

Làm U1 xong, người dùng đã biết vị trí các nút nên U2, U3 dễ hơn; hãy ghi thứ tự nhiệm vụ. Khi so hai phiên bản giao diện, đổi thứ tự phiên bản giữa các người (counterbalancing) hoặc dùng hai nhóm độc lập. Không đổi prompt giữa chừng rồi gộp số liệu như cùng điều kiện.

### 9.3. Bối cảnh thật của người đặt xe

Người đặt xe thường vội, đứng ngoài đường, mạng yếu, cầm điện thoại một tay. Bàn phím ảo hay thao tác chạm cần thiết bị thật; mô phỏng bối cảnh phải an toàn. Lab web với mã C001 không đại diện ứng dụng bản đồ production. [SYS-11 · Compatibility Test](/articles/system-testing-compatibility-test) đã nói mobile emulation không thay thiết bị thật; usability còn đòi hỏi nhiều hơn.

### 9.4. Định lượng và A/B

Ước lượng tỷ lệ thành công hay so chênh lệch nhỏ giữa hai thiết kế cần thiết kế summative (cỡ mẫu, tiêu chí, phân nhóm, phân tích bất định); sáu phiên formative không đủ để nói «có ý nghĩa thống kê». A/B trên sản phẩm thật đo hành vi ở quy mô lớn, nhưng cần instrumentation đúng và phân biệt mục tiêu người dùng (đặt được xe đúng ý) với chỉ số click.

### 9.5. AI và «synthetic participants»

AI có thể giúp rà prompt có dẫn đáp án không, gợi ý kiểu hiểu lầm, hoặc sắp xếp ghi chép đã được phép xử lý. Không gán phản hồi AI thành participant, không điền success rate bằng mô phỏng; nhận xét của AI hay chuyên gia gắn nhãn *review*.

## 10. Anti-pattern, giới hạn và đánh đổi

### 10.1. Anti-pattern

- Đọc từng bước bấm nút rồi báo «người dùng thành công».
- Tuyển toàn đồng nghiệp đã quen sản phẩm.
- Moderator gợi ý nhưng không ghi, rồi tính là independent success.
- Chỉ hỏi «có dễ không?» mà không quan sát.
- Bỏ người blocked khỏi mẫu số mà không giải thích.
- Gọi thời gian chạy Playwright là time-on-task.
- Điền worksheet bằng participant hư cấu cho «đủ bảng».

Giới hạn bằng chứng nằm ở callout đầu bài: participant = 0; thứ đã hoàn thành là bài học, bộ study kit dùng được và hai supporting check chạy lại pass.

### 10.2. Đánh đổi chi phí và phạm vi

Một vòng sáu phiên tốn khoảng một ngày quan sát cộng thời gian tuyển và tổng hợp — ít hơn nhiều chi phí của một luồng đặt xe khiến khách bỏ đi. Không phải thay đổi nào cũng cần study: đổi màu chữ thì không, đổi cách hiển thị phí huỷ thì nên; expert review lọc vấn đề hiển nhiên trước.

## 11. Bài tập và câu hỏi tự kiểm

### Bài tập

**Bài 1 (cơ bản).** Sửa prompt «Bấm Huỷ chuyến và xem phí» thành một nhiệm vụ trung lập.

<details>
<summary><b>Gợi ý đáp án</b></summary>
<p>«Bạn đổi kế hoạch khi chưa có tài xế nhận; hãy dừng chuyến và cho biết có khoản phí nào không.» Prompt nêu bối cảnh và mục tiêu, không tên nút, không thứ tự bước. Tiêu chí thành công (huỷ, thấy đã huỷ, phí 0 đ theo R-CANCEL-01) chỉ moderator giữ.</p>
</details>

**Bài 2 (trung bình).** Ở U2, participant sửa 0,1 thành 5 rồi mới bấm «Báo giá» và thấy 35.000 đ. Phiên này có chứng minh thông báo lỗi dễ hiểu không?

<details>
<summary><b>Gợi ý đáp án</b></summary>
<p>Không. Họ đạt mục tiêu sửa dữ liệu nhưng chưa từng thấy thông báo lỗi: ghi «error-message exposure = no» và không tính họ vào mẫu số của câu hỏi về thông báo lỗi. Không ép họ quay lại tạo lỗi.</p>
</details>

**Bài 3 (trung bình).** Ở U3, moderator nói «nút huỷ ở dưới cùng» và participant hoàn tất. Ghi outcome thế nào?

<details>
<summary><b>Gợi ý đáp án</b></summary>
<p>Assisted-success, kèm timestamp và nội dung gợi ý; không gộp vào independent-success. Gợi ý đó cũng cho thấy nút huỷ có thể khó tìm — ghi thành quan sát, chưa phải kết luận.</p>
</details>

**Bài 4 (nâng cao).** Đội muốn so phiên bản A (hiện tại) với phiên bản B (thêm dòng «Chưa gọi tài xế» dưới tổng tiền), mỗi người làm U1 trên cả hai. Thiết kế phiên thế nào để kết quả không bị learning effect làm sai?

<details>
<summary><b>Gợi ý đáp án</b></summary>
<p>Đổi thứ tự A/B giữa các participant (một nửa A trước, một nửa B trước) hoặc dùng hai nhóm độc lập. Dùng nhiệm vụ tương đương, không giống hệt (ví dụ 5 km và 8 km, oracle tính từ SPEC). Giữ prompt, thang rating, quy tắc think-aloud nhất quán; báo raw counts theo thứ tự trình bày; mẫu formative nhỏ chỉ cho so sánh định tính và quyết định có nên chạy study lớn hơn, không cho kết luận ý nghĩa thống kê.</p>
</details>

**Bài 5 (nâng cao).** Viết lại báo cáo sai «U2: 80% thành công» thành dạng đúng, giả sử (minh hoạ) 5 người được giao, 2 tự sửa trước khi thấy lỗi, 2 thấy lỗi và tự sửa, 1 thấy lỗi và bỏ cuộc.

<details>
<summary><b>Gợi ý đáp án</b></summary>
<p>«U2 (n = 5, dữ liệu minh hoạ): đạt mục tiêu sửa dữ liệu 4/5, abandoned 1. Đánh giá thông báo lỗi: exposed 3; trong đó tự sửa sau khi đọc 2, bỏ cuộc 1. 2 người sửa trước submit không thuộc mẫu số này.» Con số 80% gộp hai câu hỏi khác nhau và che người bỏ cuộc sau khi thấy thông báo — đúng người finding cần chú ý nhất.</p>
</details>

### Câu hỏi tự kiểm

**Câu 1.** 2/2 supporting check pass có nghĩa usability đạt?

<details>
<summary><b>Đáp án</b></summary>
<p>Không. Chúng kiểm nhãn, một đoạn Tab do code đặt điểm bắt đầu và vòng lỗi–sửa trong DOM. Không người dùng nào được quan sát.</p>
</details>

**Câu 2.** Study kit có sáu ô participant. Vậy đội đã nghiên cứu sáu người?

<details>
<summary><b>Đáp án</b></summary>
<p>Không. Sáu là số đề xuất cho vòng formative; hiện participant = 0, bảng để trống có chủ đích.</p>
</details>

**Câu 3.** Vì sao không gộp blocked vào failed?

<details>
<summary><b>Đáp án</b></summary>
<p>Blocked là lỗi môi trường, không phải người dùng thất bại vì giao diện. Gộp lại làm usability trông tệ hơn thực tế và che lỗi setup cần sửa. Báo riêng kèm lý do, nêu rõ khi loại khỏi mẫu số.</p>
</details>

**Câu 4.** Thuộc tính `aria-live="polite"` có mặt trong DOM. Có kết luận được screen reader đọc kết quả báo giá đúng lúc không?

<details>
<summary><b>Đáp án</b></summary>
<p>Không. Thuộc tính này là điều kiện cần theo R-UI-03, không phải bằng chứng trải nghiệm. Cần kiểm với screen reader thật, tốt hơn là với người dùng screen reader thường xuyên.</p>
</details>

**Câu 5.** Một finding chỉ có 1/6 người gặp có nên xếp blocker?

<details>
<summary><b>Đáp án</b></summary>
<p>Có thể, nếu nó chặn nhiệm vụ hoặc gây mất tiền và người dùng không tự hồi phục được. Ưu tiên theo hậu quả và khả năng hồi phục; tần suất chỉ là một yếu tố.</p>
</details>

## 12. Tài liệu tham khảo

- GOV.UK Service Manual — *Using moderated usability testing*: https://www.gov.uk/service-manual/user-research/using-moderated-usability-testing — cơ sở cho quan sát nhiệm vụ và điều phối phiên; bộ prompt và scoring Tới Nơi là thiết kế riêng của bài. Tra cứu 13/11/2026.
- W3C WAI — *Involving Users in Evaluating Web Accessibility*: https://www.w3.org/WAI/test-evaluate/involving-users/ — kết hợp người dùng có nhu cầu truy cập với đánh giá tiêu chuẩn. Tra cứu 13/11/2026.
- W3C WAI — *Evaluating Web Accessibility Overview*: https://www.w3.org/WAI/test-evaluate/ — vì sao cần cả người đánh giá và công cụ. Tra cứu 13/11/2026.
- Playwright — *Locators* (`getByLabel`, `getByRole`): https://playwright.dev/docs/locators — công cụ cho supporting check, không phải phương pháp usability. Tra cứu 13/11/2026.
- Lab Tới Nơi: `SPEC.md` (R-UI-01, R-UI-03, R-UI-04, R-FARE-02, R-CANCEL-01, R-CANCEL-06) và supporting check — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-usability-test
- CurlStudio 0.2.1 — API client và load test (mã nguồn, tra cứu 23/11/2026): https://github.com/rickymta/curl-studio · collection minh hoạ: `examples/toi-noi-testing/curlstudio/`
- Bài liên quan: [Non-functional testing tổng quan](/articles/non-functional-testing-tong-quan) · [Acceptance testing tổng quan](/articles/acceptance-testing-tong-quan) · [SYS-05 · System Test](/articles/system-testing-system-test) · [SYS-11 · Compatibility Test](/articles/system-testing-compatibility-test) · [SYS-15 · UAT](/articles/system-testing-uat)

Bài tiếp theo: [SYS-13 · Data Integrity Test](/articles/system-testing-data-integrity-test).
