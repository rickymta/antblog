---
slug: ba-thuc-hanh-review-goi-yeu-cau
title: "Thực hành: review một gói yêu cầu còn mơ hồ"
summary: "Tìm và sửa mâu thuẫn giữa story, rule, use case, prototype và NFR trong gói điểm đón; lập defect log, traceability và kết luận phần nào đủ để phát triển."
category: business-analysis
level: advanced
tags: [business-analysis, ba-032, requirements-review, case-study, toi-noi]
language: vi
---

“Mọi khách sửa điểm đón bất kỳ lúc nào. App lưu ngay, tài xế thấy tức thì. Xác nhận chuyển chuyến sang picked_up. Tải phải nhanh, an toàn. Nếu lỗi thì retry tới khi được.” Đoạn nháp có vẻ đầy đủ mong muốn nhưng chứa các quyết định sai phạm vi, mơ hồ và mâu thuẫn.

<div class="tkh-callout"><span class="tkh-callout-title">BA-032 · Phần 4/8 · Nâng cao · Thực hành</span><p>Đọc BA-025–BA-031. Đầu ra: findings, bản sửa gọn, traceability và kết luận review. Dùng <code>examples/toi-noi-ba/p04-goi-yeu-cau.md</code>; dành khoảng 60–90 phút làm bài. Đây là thiết kế, không phải sửa code lab.</p></div>

## 1. Xác định chuẩn để review

Hồ sơ P04 là đề xuất chung cho bài tập: chỉ chủ lượt ghi khi searching, xác nhận gắn revision, thành công dựa nguồn chuẩn, không đổi trip state. Nó chưa phải baseline sản phẩm được duyệt. SPEC lab xác nhận điểm đón chưa được triển khai; P03 có đề xuất ba phút chưa duyệt, ngoài phạm vi P04.

Review phải phân biệt lỗi trái giả định thiết kế của bài, câu hỏi chưa chốt và vấn đề cần người có quyền quyết định. Không sửa hết bằng ý kiến cá nhân rồi ghi “đã duyệt”.

## 2. Ghi finding có trigger và ảnh hưởng

| Finding | Trigger / vấn đề | Sửa đề xuất |
|---|---|---|
| F01 | “Mọi khách” có thể ghi lượt người khác | Chỉ chủ lượt; kiểm quyền đọc/ghi |
| F02 | “Bất kỳ lúc nào” trái scope searching | Nêu state được phép và kiểm lại lúc ghi |
| F03 | Xác nhận r4 khi server r5 | Dùng revision; không xác nhận bản cũ |
| F04 | App tự báo thành công | Chờ kết quả chuẩn; timeout là chưa biết |
| F05 | “Tài xế thấy tức thì” chưa có hợp đồng | Tách story đọc và tiêu chí đồng bộ cần chốt |
| F06 | Xác nhận → picked_up | Không đổi vòng đời chuyến |
| F07 | “Nhanh, an toàn” không đo được | Quality scenario và rule quyền cụ thể |
| F08 | Retry vô hạn | Chốt giới hạn, định danh, đối soát và phản hồi |

Thêm F09 nếu nháp đưa phí ba phút vào: không có approval và ngoài scope. Thêm F10 cho cách chia API trước/UI sau: kế hoạch kỹ thuật chưa chứng minh một lát cắt có thể nghiệm thu theo story khách.

## 3. Bản sửa tối thiểu có thể review

US-P04-01 giữ mục tiêu khách biết thông tin nào mình đã xác nhận. UC-P04-01 đọc dữ liệu chuẩn, khách xem, hệ thống kiểm quyền/state/revision rồi ghi và trả kết quả. Nhánh stale yêu cầu xem lại; state đổi từ chối ghi; timeout kiểm lại kết quả bằng định danh cần thiết kế.

Không tự điền hành vi ghi chú rỗng vì OPEN-01 còn mở. Không tuyên bố NFR sẵn sàng nghiệm thu khi thiết bị/mạng/cache chưa chốt. Bản sửa tốt làm rõ cả điều đã biết và điều còn thiếu.

## 4. Truy vết để tránh sửa một nơi

| Rule | Use case | AC | UI / kiểm tra |
|---|---|---|---|
| RULE-P04-01 | Bước kiểm quyền/state | 03, 04 | Không cho ghi ngoài scope; kiểm trực tiếp ngoài UI |
| RULE-P04-02 | Kiểm revision | 01, 02 | Nội dung mới phải được xem lại |
| RULE-P04-03 | Nhận kết quả chuẩn | 05 | Trạng thái đang gửi/chưa biết/thành công |
| NFR-P04-01 | Mở chi tiết | Quality scenario riêng | Đo đầu cuối và tỷ lệ lỗi |

Traceability không phải chỉ đánh dấu có liên kết. Đọc nội dung hai đầu để kiểm cùng nghĩa và version. AC trỏ đúng ID nhưng mô tả sai state vẫn là lỗi.

## 5. Walkthrough các trường hợp quyết định

Đi qua chủ lượt r4 khớp; r4 cũ; accepted thắng trước ghi; sai quyền; request đã lưu nhưng response mất. Với mỗi ca nêu trạng thái cuối, dữ liệu được ghi và điều người dùng biết.

Một reviewer đóng vai khách, một người làm kỹ thuật/test nếu có thể. Nếu tự học, thay vai theo lượt và ghi câu hỏi còn mở. Không tự dựng kết quả “người dùng thích” khi chưa có phiên nghiên cứu.

## 6. Kết luận review có phạm vi

Ví dụ: “Các rule quyền/state/revision đã được diễn đạt nhất quán trong bản đề xuất. Chưa đủ chốt triển khai vì nội dung bắt buộc, hợp đồng retry và môi trường NFR còn mở. Đề nghị review ba mục với owner trước khi baseline; có thể chuẩn bị prototype và hợp đồng nháp song song.”

Không dùng “tài liệu xong 100%” khi điều kiện ảnh hưởng kết quả còn thiếu. Cũng không chặn mọi công việc chỉ vì một câu hỏi không liên quan chưa được trả lời; nêu phụ thuộc cụ thể.

## 7. Rubric tự đánh giá

Chấm 0–2 cho: phát hiện đúng trigger; sửa có căn cứ; nhất quán liên sản phẩm; nêu câu hỏi/thẩm quyền còn mở; kế hoạch kiểm có luồng lỗi. Mục tiêu tự học từ 8/10, không mục nào 0.

**Câu hỏi:** Có cần thêm code để hoàn thành bài này không?

<details><summary>Gợi ý trả lời</summary><p>Không. Đầu ra là gói yêu cầu có thể review và chỉ rõ điều chưa chốt. Việc xây hoặc tự động hoá kiểm thử là bước khác, cần phạm vi và hợp đồng phù hợp.</p></details>

## 8. Tài liệu và bước tiếp theo

- `examples/toi-noi-ba/p04-goi-yeu-cau.md` chứa nguồn, nháp lỗi và mẫu quyết định.
- [Use case](/articles/ba-use-case-xac-nhan-diem-don), [NFR](/articles/ba-yeu-cau-phi-chuc-nang-do-duoc).

Phần 5 tiếp tục dữ liệu, API và tích hợp cho BA; các trường dữ liệu mới của P04 vẫn là đề xuất cần review, không mặc định đã tồn tại trong lab.

Bắt đầu phần 5: [glossary và mô hình dữ liệu](/articles/ba-domain-glossary-mo-hinh-du-lieu).
