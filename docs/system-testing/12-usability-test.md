# SYS-12 — Usability Test: người dùng có hoàn thành được việc họ cần không?

Trạng thái: bản nháp hoàn chỉnh để review; có study kit và supporting checks. **Chưa chạy nghiên cứu với người tham gia thật.** Bài trước: [Compatibility Test](11-compatibility-test.md). [Lộ trình](../system-testing-roadmap.md).

## 1. Tình huống và mục tiêu

API báo giá đúng 35.000, nút Đặt chuyến hoạt động trên Chrome và Edge. Nhưng người mới có thể tưởng bấm Báo giá đã gọi tài xế, hoặc thấy thông báo lỗi mà không biết sửa ở đâu. Automation thao tác đúng theo kịch bản không trả lời được người dùng có hiểu giao diện hay không.

Bài dành cho tester, BA, designer và người làm sản phẩm. Sau bài, bạn có thể chuyển câu hỏi sản phẩm thành nhiệm vụ trung lập, chuẩn bị phiên quan sát, ghi hành vi tách khỏi suy diễn, tính metric có mẫu số rõ và ưu tiên cải tiến dựa trên bằng chứng. Không cần viết code để tổ chức study; code đi kèm chỉ kiểm điều kiện hỗ trợ.

## 2. Khái niệm và phân biệt

Usability testing quan sát người dùng đại diện thực hiện nhiệm vụ trong bối cảnh cụ thể để tìm trở ngại và đánh giá kết quả. Tập trung vào hiệu quả hoàn thành, công sức/thời gian và trải nghiệm được phản hồi. Một người nói “giao diện đẹp” chưa chứng minh họ hoàn thành được công việc.

| Hoạt động | Trả lời điều gì? | Không thay thế |
|---|---|---|
| Functional test | Hệ thống xử lý đúng khi được thao tác theo test? | Khả năng người mới tự tìm đường đi |
| Usability study | Người dùng thực tế hiểu và hoàn thành nhiệm vụ ra sao? | Kiểm mọi quy tắc kỹ thuật |
| Accessibility evaluation | Người có nhu cầu truy cập khác nhau có sử dụng được, đáp ứng tiêu chí nào? | Toàn bộ usability hoặc chứng nhận chỉ bằng scanner |
| Expert/heuristic review | Chuyên gia phát hiện vấn đề theo nguyên tắc | Quan sát participant thật |
| UAT | Nghiệp vụ có chấp nhận giải pháp theo tiêu chí? | Trải nghiệm của mọi nhóm khách |

Accessibility và usability giao nhau: một người dùng bàn phím không tới được nút sẽ không hoàn thành task. Tuy nhiên aria-live tồn tại không chứng minh screen reader đọc đúng, và user study với vài người không chứng minh tuân thủ toàn bộ tiêu chuẩn accessibility.

## 3. Chọn phương pháp phù hợp câu hỏi

Moderated study có người điều phối, thích hợp khi cần hỏi lại lý do hoặc xem người dùng xử lý tình huống mới. Unmoderated có thể thu nhiều phiên hơn nhưng khó phân biệt bỏ cuộc với lỗi setup nếu thiết kế thiếu rõ ràng. Formative study tìm vấn đề để cải tiến; summative study đánh giá theo tiêu chí định trước, cần mẫu và phương pháp phù hợp hơn cho suy luận định lượng.

Bài chọn formative moderated cho ba câu hỏi: người dùng có phân biệt báo giá/đặt; có sửa được dữ liệu không hợp lệ; có nhận biết đã huỷ và phí không? Đây là phạm vi hẹp của trang khách. Không đánh giá trải nghiệm toàn nền tảng đặt xe, bản đồ, tài xế, GPS hoặc thanh toán thật.

Sáu người là đề xuất vòng học ban đầu trong study kit, không phải một định luật “sáu người đủ” hay bảo đảm tỷ lệ thành công của dân số. Tuyển theo nhóm sử dụng dự kiến, mức kinh nghiệm và nhu cầu hỗ trợ; thành viên đã xây giao diện có thể nhớ đường đi nên không đại diện người mới.

## 4. Thiết kế nhiệm vụ không dẫn đáp án

Prompt tốt nêu mục tiêu và bối cảnh, không đọc tên nút hoặc các bước cần bấm. “Hãy bấm Báo giá, rồi tìm Tổng tiền” đo khả năng làm theo chỉ dẫn. “Bạn định đi xe máy 5 km; hãy tìm số tiền cần trả, chưa yêu cầu tài xế” cho phép quan sát mô hình hiểu của người dùng.

Tách prompt participant khỏi scoring moderator. Có thể thiết lập dữ liệu trước để tập trung một nhiệm vụ, nhưng phải ghi rõ điều đó: U3 chuẩn bị chuyến searching sẵn nên không chứng minh participant tự đặt được chuyến. Với U2, nếu họ sửa quãng đường trước khi submit, không ép họ tạo lỗi chỉ để đúng script; ghi chưa có exposure với thông báo lỗi.

[Study kit](12-study-kit.md) chứa kế hoạch phiên, script mở đầu, ba task cards, quy tắc hỗ trợ, worksheet và mẫu finding có thể dùng ngay. Các bảng kết quả vẫn để trống vì chưa có phiên thật; không dùng participant hư cấu để lấp chỗ trống.

## 5. Quy trình thực hành từng bước

**Chuẩn bị:** thống nhất research questions và đối tượng, chọn build, browser và dữ liệu giả. Chuẩn bị consent, nơi lưu/hạn xoá dữ liệu, cách hỗ trợ participant và kênh ghi chép. Chạy pilot với đồng nghiệp để phát hiện prompt/setup lỗi; không trộn kết quả pilot vào cohort mà không nêu rõ.

**Mở đầu:** giải thích đang đánh giá giao diện, không chấm điểm con người; participant được nghỉ/dừng. Chỉ ghi âm/màn hình khi có đồng ý phù hợp. Không dùng dữ liệu đặt xe hay thanh toán thật cho bài tập.

**Quan sát:** đọc prompt, giữ im lặng đủ để participant tự làm, dùng câu hỏi trung lập khi cần. Think-aloud giúp hiểu cách suy nghĩ nhưng có thể đổi tốc độ thực hiện. Ghi hành động, thời điểm và lời nói trước; diễn giải để ở cột khác.

**Hỗ trợ:** nếu phải hướng dẫn, ghi nội dung/thời điểm và outcome assisted. Không xếp assisted-success vào independent-success để làm tỷ lệ đẹp. Lỗi môi trường làm task không thực hiện được cần ghi blocked, không trách participant.

**Kết thúc:** hỏi lại điểm chưa rõ, nhận phản hồi và kiểm các quan sát trái giả thuyết ban đầu. Tổng hợp giữa các phiên theo task, không chỉ nhớ tình huống ấn tượng nhất. Đề xuất sửa rồi lên retest, không mặc định mọi ý kiến “thêm nút” là giải pháp tối ưu.

## 6. Metrics và cách tránh kết luận sai

| Metric | Cách dùng trong study kit | Giới hạn |
|---|---|---|
| Task success | Independent/assisted/failed/abandoned/blocked tách riêng | Định nghĩa success trước quan sát |
| Time-on-task | Từ bắt đầu thao tác tới hoàn tất/dừng, ghi trợ giúp | Không trộn nhóm thành công với nhóm bỏ cuộc thiếu giải thích |
| Errors/recovery | Sai thao tác và cách tự sửa | Không phải mọi backtrack đều là lỗi |
| Rating sau task | Câu hỏi tự thiết kế 1 rất khó–7 rất dễ, hỏi vì sao | Không gọi là SUS hoặc điểm usability chuẩn |
| Finding frequency | Số người gặp / số có cơ hội gặp | Mẫu nhỏ không đại diện tỷ lệ thị trường |

Ví dụ minh họa cách tính, **không phải kết quả study này**: 5 người được giao task, 3 tự hoàn thành, 1 cần giúp, 1 bị lỗi môi trường. Báo các counts đầy đủ; nếu tính independent success trên 4 người có cơ hội thực hiện là 3/4, phải ghi người blocked và lý do loại. Không chỉ ghi 75% để che mẫu nhỏ.

Nếu chỉ người nhìn thấy lỗi mới được hỏi về thông báo, mẫu số của finding “không hiểu lỗi” là số exposed, không phải toàn bộ cohort. Tương tự chỉ participant đã có chuyến searching mới có cơ hội đánh giá phí huỷ. Định nghĩa exposure giúp tránh diễn giải sai.

## 7. Lab automation hỗ trợ, không thay người dùng

Config `examples/toi-noi-testing/usability.config.ts` mở server PGlite riêng 3117; spec ở `articles/system-testing-usability-test/e2e/support.spec.ts`. Từ thư mục lab, Node >=22.12 và Chrome có sẵn:

```sh
npm ci
npx playwright test --config usability.config.ts
npm run typecheck
```

Expected hai supporting checks pass. Một worker, zero retry, reset seed và đặt giờ 09:00 VN trước mỗi case. Context/DB do runner quản lý; không dùng môi trường của participant hoặc production.

| ID | Basis/hành động | Oracle | Điều chưa chứng minh |
|---|---|---|---|
| US-A01 | R-UI-03/04: tìm input bằng label; focus voucher, Tab tới payment rồi Báo giá, Enter | Focus đúng ở đoạn đó, 35.000 đ, region aria-live polite, đặt được enable | Người dùng tự tìm được, full keyboard navigation, focus nhìn rõ hoặc screen reader đọc đúng |
| US-A02 | R-UI-01/03: nhập 0,1 km → báo giá lỗi → sửa 5 km → báo giá | Alert có chữ Quãng đường, đặt disabled; sau sửa alert rỗng, 35.000, đặt enabled | Người dùng hiểu thông báo và tự chọn cách sửa |

US-A01 dùng focus để đặt điểm bắt đầu nên **không** được báo “toàn hành trình chỉ dùng bàn phím đạt”. getByLabel xác nhận liên kết label cho các field được dùng, không audit toàn form. Không chạy axe hoặc tuyên bố WCAG conformance. Kết quả thực đo nằm trong [báo cáo](12-test-report.md).

## 8. Phân tích finding và ưu tiên

Finding cần liên kết hành vi quan sát với tác động. “Nút chưa đẹp” thiếu căn cứ; “P02/P04 dừng sau Báo giá và tưởng đã gọi tài xế” có thể hữu ích **nếu thật sự đã quan sát**, kèm task/timestamp. Trong bài này ví dụ đó chỉ minh họa cấu trúc, không được ghi là finding thực tế.

Tách observation, interpretation và recommendation. Observation là người bấm gì/nói gì; interpretation là giả thuyết họ hiểu thế nào; recommendation là thay đổi cần kiểm chứng. Một finding nghiêm trọng có thể chỉ xuất hiện ở một participant nếu chặn nhiệm vụ hoặc gây mất tiền; không ưu tiên chỉ bằng tần suất.

Severity có thể dùng blocker/nghiêm trọng/vừa/nhỏ với tiêu chí về hậu quả, khả năng tự hồi phục và mức lặp. Ghi độ chắc chắn và evidence, không giả định điểm số thay quyết định sản phẩm. Retest sau thay đổi với nhiệm vụ tương đương; chú ý người cũ đã học giao diện.

## 9. Mở rộng nâng cao

### 9.1. Người dùng công nghệ hỗ trợ

Tuyển người sử dụng screen reader, bàn phím hoặc phương tiện hỗ trợ theo nhóm khách thực tế; cho dùng thiết lập quen thuộc khi phù hợp. Moderator không nên tự giả làm người dùng khiếm thị rồi xem đó là dữ liệu đại diện. Kết hợp chuyên gia accessibility, standards review và user evidence; mỗi nguồn trả lời một phần khác nhau.

### 9.2. Learning effects và đối trọng thứ tự

Người làm U1 có thể học vị trí nút giúp U2/U3 dễ hơn. Ghi thứ tự tasks; nếu so hai phiên bản, cân nhắc đổi thứ tự phiên bản giữa người để giảm learning effect, hoặc dùng nhóm độc lập. Không đổi prompt giữa chừng rồi gộp số liệu như cùng điều kiện.

### 9.3. Usability trong bối cảnh thật

Người đặt xe có thể vội, mạng yếu hoặc dùng một tay. Mô phỏng bối cảnh cần phù hợp và không gây nguy hiểm; quan sát trên thiết bị thật nếu câu hỏi liên quan bàn phím ảo hoặc thao tác di động. Lab web với mã C001 không đại diện app bản đồ production; giới hạn đó phải xuất hiện trong report.

### 9.4. Quantitative study và A/B

Nếu muốn ước lượng tỷ lệ hoặc so sánh chênh lệch nhỏ, cần thiết kế cỡ mẫu, tiêu chí, phân nhóm và phân tích bất định. Sáu phiên formative không đủ tự suy significance. A/B trên sản phẩm đo hành vi ở quy mô khác nhưng vẫn cần event instrumentation đúng và phân biệt mục tiêu người dùng với chỉ số click.

### 9.5. AI và synthetic participants

Có thể dùng AI để rà prompt, đề xuất abuse/misunderstanding hypotheses hoặc sắp xếp notes đã được phép xử lý. Không gán phản hồi AI thành participant thật, không điền success rate bằng mô phỏng. Nếu có nhận xét từ chuyên gia/AI, gắn nhãn review để không trộn với user study.

## 10. Anti-pattern và giới hạn

Đọc từng bước bấm nút rồi báo người dùng thành công; chọn toàn đồng nghiệp quen sản phẩm; moderator chỉ dẫn nhưng không ghi trợ giúp; chỉ hỏi “có dễ không”; bỏ blocked khỏi mẫu số mà không giải thích; gọi thời gian Playwright là time-on-task — đều làm evidence sai lệch.

Hiện tại participant=0. Không có success rate, rating, thời gian nhiệm vụ hoặc findings người dùng thực đo. Hai automation checks không chứng minh giao diện dễ dùng, không đủ accessibility audit, không phải UAT. Mục tiêu đã hoàn thành ở lượt viết là bài học, bộ nghiên cứu có thể dùng và supporting checks được kiểm chứng.

## 11. Bài tập và đáp án định hướng

1. **Cơ bản:** sửa prompt “Bấm Huỷ chuyến và xem phí” thành mục tiêu trung lập. Gợi ý: “Bạn đổi kế hoạch khi chưa có tài xế nhận; hãy dừng chuyến và cho biết có khoản phí nào.”
2. **Trung bình:** participant sửa 0,1 thành 5 trước submit có chứng minh hiểu thông báo lỗi? Không, đánh dấu chưa exposure; chỉ kết luận về chỉnh dữ liệu.
3. **Trung bình:** moderator nhắc tên nút rồi người dùng hoàn tất. Outcome assisted, không independent; giữ timestamp trợ giúp.
4. **Nâng cao:** so phiên bản A/B với người làm cả hai. Đáp án cần xử lý thứ tự, learning effect, nhiệm vụ tương đương và thang đo nhất quán; không chỉ so mean thời gian.
5. **Tự kiểm:** 2/2 automation pass có nghĩa usability đạt? Không. Study kit có sáu ô participant có nghĩa đã nghiên cứu sáu người? Không, phải có phiên và dữ liệu thật.

## 12. Tài liệu tham khảo

- [GOV.UK — Using moderated usability testing](https://www.gov.uk/service-manual/user-research/using-moderated-usability-testing): cơ sở cho quan sát nhiệm vụ và điều phối phiên; bộ prompt/scoring Tới Nơi là thiết kế riêng của bài.
- [W3C WAI — Involving Users in Evaluating Web Accessibility](https://www.w3.org/WAI/test-evaluate/involving-users/): kết hợp người dùng có nhu cầu truy cập với đánh giá tiêu chuẩn, không thay nhau.
- SPEC R-UI-01/03/04 và [study kit](12-study-kit.md): test basis UI và quy trình thực hành chi tiết.

Nguồn đối chiếu ngày 13/11/2026. Bài tiếp theo: Data Integrity Test — ràng buộc, giao dịch và đối soát dữ liệu qua vòng đời nghiệp vụ.
