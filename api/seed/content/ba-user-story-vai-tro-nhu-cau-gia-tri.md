---
slug: ba-user-story-vai-tro-nhu-cau-gia-tri
title: "User story: vai trò, nhu cầu và giá trị"
summary: "Viết user story từ nhu cầu điểm đón, tách task kỹ thuật khỏi kết quả người dùng và chuẩn bị cuộc trao đổi cùng tiêu chí xác nhận."
category: business-analysis
level: beginner
tags: [business-analysis, ba-025, user-story, requirements, toi-noi]
language: vi
---

“Là người dùng, tôi muốn một API cập nhật pickupRevision để hệ thống lưu dữ liệu.” Câu này có đủ khuôn vai trò–mong muốn–lợi ích, nhưng chưa nói người dùng làm việc gì tốt hơn. Thêm mẫu câu vào task kỹ thuật không tự biến nó thành story có giá trị.

<div class="tkh-callout"><span class="tkh-callout-title">BA-025 · Phần 4/8 · Cơ bản</span><p>Đọc trước: <a href="/articles/ba-thuc-hanh-thay-doi-chinh-sach-huy">Thực hành phần 3</a>. Đầu ra: story có mục tiêu và câu hỏi refinement. P04 là thiết kế giả lập mới; lab hiện không có điểm đón/đến theo R-GEN-06.</p></div>

## 1. Story bắt đầu từ việc người dùng muốn hoàn thành

US-P04-01 đề xuất: “Là khách đặt xe, tôi muốn xem và xác nhận thông tin điểm đón mình cung cấp để biết thông tin nào đang được dùng cho lượt đặt xe.” Giá trị ở đây là sự rõ ràng, chưa phải lời hứa giảm huỷ hoặc bảo đảm tài xế đã đọc.

Vai trò cần đủ cụ thể để xét quyền và bối cảnh. “Người dùng” có thể là khách, tài xế hoặc CSKH với quyền khác nhau. “Hệ thống” đôi lúc xuất hiện trong công việc nền, nhưng không nên dùng vai trò này để che câu hỏi ai nhận lợi ích.

## 2. Card, Conversation, Confirmation

[Agile Alliance mô tả ba C của user story](https://agilealliance.org/glossary/three-cs/): thẻ ghi ngắn, trao đổi để làm rõ và xác nhận kỳ vọng. Một câu story là điểm bắt đầu, không thay toàn bộ yêu cầu.

| Thành phần | Áp dụng US-P04-01 |
|---|---|
| Card | Khách xem/xác nhận đúng thông tin của lượt mình |
| Conversation | Khi nào được sửa, xác nhận nghĩa là gì, dữ liệu cũ thì sao? |
| Confirmation | Chủ lượt, searching, revision khớp thì ghi nhận đúng bản; state/revision đổi thì không xác nhận bản cũ |

Đừng hoàn thiện câu văn rồi coi như refinement đã xong. Nếu developer và tester còn hiểu “xác nhận” thành hai trạng thái khác nhau, story vẫn chưa đủ rõ.

## 3. Phân biệt story, task và rule

| Nội dung | Vai trò trong gói yêu cầu |
|---|---|
| Khách xác nhận thông tin mình đã xem | Story |
| Thêm cột revision, thiết kế API, dựng màn hình | Task/thiết kế để thực hiện story |
| Chỉ chủ lượt ghi khi searching | Rule đề xuất |
| Màn hình r4 nhưng server r5 thì yêu cầu xem lại | Acceptance criterion và ví dụ |

Task kỹ thuật có giá trị trong kế hoạch, không cần ép tất cả vào câu “Là người dùng”. Nhóm có thể quản lý chúng dưới story hoặc dưới công việc hỗ trợ riêng, miễn truy vết được lý do và cách kiểm.

## 4. Làm rõ phạm vi đầu tiên

P04 giới hạn ghi trước khi tài xế nhận. Sau accepted, việc sửa điểm đón chưa thuộc lát cắt đầu; việc tài xế được phân công xem dữ liệu là story liên quan. Đây là đề xuất phạm vi để học đặc tả, chưa được duyệt sản phẩm.

Xác nhận thông tin không đổi chuyến sang picked_up. Thao tác này chỉ ghi lại khách đã xác nhận revision nào. Nếu khách sửa, revision mới cần được xem/xác nhận theo rule mới; không giữ dấu xác nhận cũ cho dữ liệu khác.

## 5. Kiểm story có đủ để trao đổi không

Hỏi: ai hưởng lợi; kết quả quan sát được là gì; có thể trình diễn một luồng trọn vẹn không; điều gì nằm ngoài phạm vi; câu hỏi nào chặn ước lượng hoặc kiểm thử? Không dùng checklist như cách chấm điểm cơ học để ép mọi story độc lập tuyệt đối.

Ví dụ quyền truy cập, dữ liệu chuẩn và xử lý state đổi là các phụ thuộc thật. Story vẫn hữu ích khi phụ thuộc được nêu, nhưng không nên tuyên bố “sẵn sàng phát triển” nếu chưa có quyết định về ý nghĩa xác nhận.

## 6. Những lời hứa cần tránh

“Để không bao giờ nhầm điểm đón” vượt bằng chứng P02. “Để tài xế nhìn thấy ngay” trộn kỳ vọng đồng bộ chưa được định nghĩa. Viết lợi ích sát với kết quả kiểm được của story, rồi nối tới giả thuyết giá trị rộng hơn trong brief.

Với các nhóm stakeholder khác nhau, có thể cần story riêng. Không dùng một story “mọi người quản lý điểm đón” vì như vậy quyền của khách, tài xế và hỗ trợ sẽ bị gộp.

## 7. Bài tập và tự kiểm tra

Sửa câu mở đầu bài thành một story khách và một task kỹ thuật. Thêm ba câu hỏi còn mở, hai tiêu chí xác nhận và một điều ngoài scope. Dùng hồ sơ `p04-goi-yeu-cau.md`, không giả định API đã tồn tại.

**Câu hỏi:** Story có đúng khuôn “Là… tôi muốn… để…” là đã tốt chưa?

<details><summary>Gợi ý trả lời</summary><p>Chưa. Cần vai trò, nhu cầu và giá trị có nghĩa, trao đổi các tình huống và cách xác nhận. Khuôn câu không tự giải quyết mơ hồ hoặc thiếu bằng chứng.</p></details>

## 8. Tham khảo và bài tiếp theo

- [Agile Alliance — Three C’s](https://agilealliance.org/glossary/three-cs/), [User Stories](https://agilealliance.org/glossary/user-stories/).
- Hồ sơ: `examples/toi-noi-ba/p04-goi-yeu-cau.md`.

Bài tiếp theo: [Acceptance criteria và Given–When–Then](/articles/ba-acceptance-criteria-given-when-then).
