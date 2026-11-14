---
slug: ba-backlog-refinement-cau-hoi-mo
title: "Backlog và refinement: biến ý tưởng thành công việc đủ rõ"
summary: "Tổ chức backlog theo vấn đề và giá trị, dùng câu hỏi mở cùng ví dụ để chuẩn bị trao đổi thay vì coi mọi ticket là yêu cầu đã duyệt."
category: business-analysis
level: beginner
tags: [business-analysis, ba-041, backlog, refinement, toi-noi]
language: vi
---

Backlog Tới Nơi có ba dòng: “thêm chat”, “xác nhận điểm đón”, “tăng miễn phí huỷ”. Nếu chỉ xếp chúng theo ngày người dùng gửi, nhóm sẽ thiếu căn cứ để chọn việc tiếp theo.

<div class="tkh-callout"><span class="tkh-callout-title">BA-041 · Phần 6/8 · Cơ bản</span><p>Đọc gói yêu cầu P04. Đầu ra: ba mục backlog có mục tiêu, phạm vi, ví dụ và câu hỏi. Toàn bộ trạng thái ưu tiên trong bài là đề xuất học tập.</p></div>

## 1. Backlog giữ công việc và lý do

Một mục backlog nên cho biết vấn đề gì, ai hưởng lợi và điều gì cần tìm hiểu. Không phải mọi mục đều phải là user story. Việc khảo sát, sửa lỗi hoặc kiểm chứng một rủi ro cũng cần được nhìn thấy và gắn với mục tiêu.

Trong Scrum, Product Owner chịu trách nhiệm quản lý Product Backlog hiệu quả; refinement là hoạt động liên tục làm rõ và chia nhỏ công việc. BA có thể hỗ trợ phân tích, nhưng tên vai trò BA không tự tạo quyền quyết định ưu tiên.

## 2. Viết lại ba yêu cầu đầu vào

| Mục | Lý do xem xét | Điều chưa đủ rõ |
|---|---|---|
| BL-01 Xác nhận điểm đón | Giả thuyết giảm hiểu sai vị trí | Tài xế có đọc được thông tin đúng revision? |
| BL-02 Chat | Đề xuất hỗ trợ trao đổi | Vấn đề nào chat giải quyết tốt hơn ghi chú? |
| BL-03 Miễn huỷ dưới ba phút | CR-BA-01 về chính sách | Ai chịu chi phí, đo hành vi thế nào? |

BL-02 chưa có đủ bằng chứng để đứng đầu chỉ vì stakeholder nhắc nhiều. BL-03 không kế thừa mục tiêu giảm huỷ của BL-01 nếu chưa giải thích quan hệ nhân quả.

## 3. Refinement bằng ví dụ

Với BL-01, BA mang tình huống khách mở revision 3, người khác cập nhật thành revision 4 trước khi khách xác nhận. Nhóm bàn outcome khi stale, quyền sửa và trạng thái chuyến. Một ví dụ cụ thể thường bộc lộ điều thiếu rõ nhanh hơn câu “đảm bảo đồng bộ”.

Kết quả buổi trao đổi có thể là tách công việc nghiên cứu, bổ sung AC hoặc dừng phương án. Không bắt buộc biến mọi ý tưởng thành ticket triển khai ngay.

## 4. Chia nhỏ theo kết quả dùng được

“Backend trước, frontend sau” có thể hữu ích cho kế hoạch kỹ thuật, nhưng chưa diễn tả kết quả người dùng nhận được. Thử lát cắt khách xem và xác nhận điểm đón trong một phạm vi pilot đã thống nhất, bao gồm quyền, revision và phản hồi lỗi cần thiết.

Không bỏ bước kiểm tra quyền để lát cắt nhỏ hơn. Phần tối thiểu vẫn phải đáp ứng các điều kiện chất lượng và an toàn đã cam kết.

## 5. Điều kiện sẵn sàng trao đổi

Nhóm có thể tự chọn mẫu sau: có mục tiêu, actor, phạm vi, ví dụ chính/ngoại lệ, phụ thuộc và câu hỏi có người phụ trách. Mẫu này hỗ trợ trao đổi, không phải một cổng phê duyệt bắt buộc của Scrum. Câu hỏi còn mở được phép tồn tại nếu không khiến nhóm hiểu khác nhau về phần sắp làm.

## 6. Sau refinement cần cập nhật gì?

Ghi quyết định vào nguồn yêu cầu, nối ID vào backlog và giữ câu hỏi chưa trả lời. Không để kết luận chỉ nằm trong chat. Nếu thay đổi ảnh hưởng tới ước lượng hoặc thứ tự, người có trách nhiệm lập kế hoạch phải tham gia cập nhật.

## 7. Thực hành

Chọn BL-03, viết ba câu hỏi về thời điểm áp chính sách và một ví dụ tại đúng 180 giây. Nêu điều cần làm rõ trước khi nhóm ước lượng.

<details><summary>Một câu hỏi đủ cụ thể</summary><p>Chuyến accepted trước thời điểm bật chính sách nhưng huỷ sau đó dùng phiên bản nào? Câu trả lời ảnh hưởng dữ liệu phiên bản, thông báo và test, nên cần người phụ trách quyết định.</p></details>

## 8. Nguồn và bài tiếp

[Scrum Guide 2020](https://scrumguides.org/scrum-guide.html) cho trách nhiệm Product Backlog và refinement. Tiếp: [MoSCoW](/articles/ba-moscow-uu-tien-theo-muc-tieu).
