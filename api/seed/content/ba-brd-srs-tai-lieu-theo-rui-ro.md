---
slug: ba-brd-srs-tai-lieu-theo-rui-ro
title: "BRD, SRS hay tài liệu gọn: chọn theo rủi ro quyết định"
summary: "Chọn cấu trúc tài liệu đủ giúp review và thay đổi, giữ một nguồn chuẩn cho rule, version và approval; tránh vừa thiếu thông tin vừa sao chép nhiều bản mâu thuẫn."
category: business-analysis
level: advanced
tags: [business-analysis, ba-031, brd, srs, documentation, toi-noi]
language: vi
---

Một bên muốn BRD 40 trang; bên kia nói Agile chỉ cần story. Cả hai chưa trả lời ai sẽ dùng tài liệu để quyết định gì. Với gói điểm đón, một câu story thiếu rule/revision không đủ, nhưng sao chép rule vào năm file cũng làm thay đổi khó hơn.

<div class="tkh-callout"><span class="tkh-callout-title">BA-031 · Phần 4/8 · Nâng cao</span><p>Đọc trước: <a href="/articles/ba-yeu-cau-phi-chuc-nang-do-duoc">NFR đo được</a>. Đầu ra: cấu trúc gói yêu cầu, nguồn chuẩn và cơ chế phê duyệt. Không áp đặt tên tài liệu chung cho mọi tổ chức.</p></div>

## 1. Hỏi quyết định trước khi chọn mẫu

BRD thường tập trung lý do, mục tiêu và phạm vi kinh doanh; SRS thường đi sâu yêu cầu giải pháp. Cách gọi và nội dung khác nhau giữa tổ chức, nên phải thống nhất người đọc, mục đích và thông tin cần có thay vì chỉ giao tên file.

P04 cần người nghiệp vụ review ý nghĩa xác nhận, kỹ thuật review tính nhất quán/retry và tester review cách kiểm. Họ có thể dùng một gói liên kết thay vì một tài liệu khổng lồ.

## 2. Thiết kế gói tài liệu theo thông tin

| Thành phần | Nguồn chuẩn đề xuất | Người dùng chính |
|---|---|---|
| Mục tiêu, scope, câu hỏi mở | Brief P04 | PO/nghiệp vụ |
| Story, use case, AC | Requirement catalog | Cả nhóm |
| Rule quyền/state/revision | Rule catalog có ID | Nghiệp vụ, dev, tester |
| Trạng thái UI | Wireframe liên kết AC | UX, dev, tester |
| Hợp đồng dữ liệu/API | Tài liệu kỹ thuật liên kết rule | Dev, tích hợp, tester |
| Approval và thay đổi | Decision/change log | Người chốt và người bị ảnh hưởng |

Không ghi “chi tiết theo họp” nếu người nhận không có bản ghi quyết định. Cuộc trao đổi cần được phản ánh vào nguồn chuẩn đủ để người không dự họp hiểu kết quả.

## 3. Tăng chi tiết ở chỗ chi phí hiểu sai cao

Revision, quyền và timeout cần ví dụ rõ vì hiểu khác gây ghi sai hoặc truy cập sai. Màu nhãn có thể để design system quyết định nếu không có yêu cầu đặc biệt. Mức tài liệu nên tương xứng rủi ro và số nhóm phụ thuộc.

Một tích hợp nhiều đội, chính sách tiền hoặc nghĩa vụ đã xác nhận có thể cần baseline, review chính thức và bằng chứng nhiều hơn thay đổi câu chữ nhỏ. Không mặc định tài liệu dài hơn luôn an toàn hơn; cần kiểm khả năng tìm thông tin và giữ nhất quán.

## 4. Version phải có ý nghĩa

Đặt version gói, ngày, người sửa, phạm vi thay đổi và trạng thái: draft, đang review, được chốt trong phạm vi cụ thể, bị thay thế. “Final-v7” không cho biết người nào đã phê duyệt cái gì.

Ví dụ PO đồng ý scope không có nghĩa NFR được chốt khi môi trường đo còn trống. Ghi phê duyệt có phạm vi; không để người đọc suy ra mọi ô trong cùng file đều approved.

## 5. Tránh sao chép rule thành nhiều nguồn

Wireframe có thể hiển thị thông báo theo RULE-P04-01 nhưng không tự định nghĩa quyền khác. AC tham chiếu rule và đưa ví dụ; nếu rule đổi thì cập nhật các AC bị ảnh hưởng. Dùng traceability để tìm phụ thuộc, không chỉ nhớ bằng trí nhớ tác giả.

CR-BA-01 về ba phút vẫn chưa duyệt và ngoài P04. Việc người viết chép nó vào “yêu cầu mới” không tạo hiệu lực chính sách. Nguồn và trạng thái phải đi cùng nội dung.

## 6. Chọn cách review và bàn giao

Gửi người review các câu hỏi cụ thể: có đúng mục tiêu, có chấp nhận scope, rule có nhất quán, NFR có đo được? Ghi issue chưa đóng, owner, hạn và ảnh hưởng. Nếu phải phát triển một phần trước, chỉ rõ phần nào đủ quyết định và phần nào chưa được cam kết.

Tài liệu được duyệt vẫn có thể sai. Phê duyệt là cơ chế chịu trách nhiệm quyết định, không thay walkthrough, kiểm thử hay đánh giá giá trị sau triển khai.

## 7. Bài tập và câu hỏi ôn tập

Thiết kế mục lục gói P04 tối đa một trang, ghi nơi lưu chuẩn cho mỗi loại thông tin. Thêm hai ví dụ approval có phạm vi và một change log khi sửa hành vi timeout.

**Câu hỏi:** Có thể giữ rule ở chat rồi chỉ liên kết chat từ story không?

<details><summary>Gợi ý trả lời</summary><p>Cần bảo đảm quyết định cuối được tìm thấy, truy cập, version và duy trì. Chat có thể là nguồn lịch sử, nhưng thường cần tổng hợp kết luận vào nơi nhóm dùng làm baseline.</p></details>

## 8. Tham khảo và bài tiếp theo

- Hồ sơ: `examples/toi-noi-ba/p04-goi-yeu-cau.md`; [review liên mô hình](/articles/ba-kiem-tra-nhat-quan-mo-hinh).

Bài tiếp theo: [Thực hành: review một gói yêu cầu còn mơ hồ](/articles/ba-thuc-hanh-review-goi-yeu-cau).
