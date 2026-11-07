---
slug: ba-uat-vai-tro-nguoi-dung-nghiep-vu
title: "UAT là gì? BA và người dùng nghiệp vụ tham gia ra sao?"
summary: "Xác định mục tiêu, đại diện nghiệp vụ và bằng chứng UAT; phân biệt hỗ trợ nghiệm thu với tự quyết định thay người sử dụng."
category: business-analysis
level: beginner
tags: [business-analysis, ba-049, uat, acceptance, toi-noi]
language: vi
---

Một màn hình chạy đúng trên máy developer chưa trả lời liệu CS có xử lý được ca khách xác nhận nhầm cổng hay không. UAT — user acceptance testing — đưa hành trình và nhu cầu nghiệp vụ vào hoạt động kiểm tra chấp nhận.

<div class="tkh-callout"><span class="tkh-callout-title">BA-049 · Phần 7/8 · Cơ bản</span><p>Đầu ra: UAT charter cho điểm đón đề xuất P04. Giả định phục vụ bài học là đã có một bản triển khai để nghiệm thu; repository không cung cấp tính năng đó và bài không tuyên bố đã chạy UAT thật.</p></div>

## 1. UAT trả lời câu hỏi nào?

Hệ thống có hỗ trợ các tình huống nghiệp vụ đã thống nhất để người có thẩm quyền chấp nhận không? Câu hỏi này khác với việc code có đúng từng hàm, cũng khác với việc tính năng làm giảm huỷ sau phát hành. Các hoạt động bổ sung cho nhau; UAT không thay toàn bộ kiểm thử kỹ thuật hoặc đo giá trị.

## 2. Chọn người đại diện có căn cứ

Với điểm đón, cần góc nhìn khách đặt chuyến, tài xế được phân công và CS. Chọn người hiểu bối cảnh sử dụng; một quản lý chưa từng xử lý ca không tự đại diện cho mọi nhân viên. Ghi những nhóm chưa tham gia, chẳng hạn người dùng thiết bị cũ hoặc điều kiện mạng yếu, để biết giới hạn bằng chứng.

## 3. Phân công trong bài tập

| Vai trò | Trách nhiệm đề xuất |
|---|---|
| Đại diện nghiệp vụ | Xác nhận outcome phù hợp, nêu tác động thực tế |
| BA | Chuẩn bị scenario, làm rõ baseline, truy vết phản hồi |
| QA | Hỗ trợ dữ liệu, môi trường, bằng chứng và phân loại lỗi |
| Dev | Điều tra, sửa và cung cấp bản mới có thể nhận diện |
| Người có thẩm quyền nghiệm thu | Quyết định chấp nhận theo phạm vi được giao |

Một người có thể kiêm vai trò, nhưng quyền nghiệm thu phải rõ. BA điều phối không có nghĩa BA tự ký thay nghiệp vụ.

## 4. Viết charter ngắn

Mục tiêu: kiểm hành trình khách xác nhận đúng điểm trước accepted và tài xế được phân công đọc được. Phạm vi gồm revision cũ, actor sai, chuyến đổi trạng thái và timeout. Ngoài phạm vi là chat, chỉnh điểm sau accepted và chứng minh giảm huỷ ngoài thực tế.

Đầu vào cần có baseline, build, tài khoản/role, dữ liệu và môi trường đủ đại diện. Thiếu điều kiện đầu vào thì ghi blocked, không đánh dấu fail sản phẩm một cách tùy tiện.

## 5. Hướng dẫn vừa đủ

Cho người tham gia mục tiêu nghiệp vụ và dữ kiện, tránh đọc từng cú click khiến họ chỉ làm theo kịch bản. Quan sát chỗ họ hiểu sai, hỏi lý do sau hành động. Một scenario có thể pass về dữ liệu nhưng vẫn tạo phản hồi usability đáng xem xét.

## 6. Bằng chứng có thể dùng lại

Lưu scenario ID, người thực hiện, build, dữ liệu đầu vào, actual outcome và tài liệu đính kèm phù hợp. Với ảnh/log, dùng dữ liệu giả và bỏ thông tin không cần thiết. Một câu “đã test OK” thiếu ngữ cảnh không đủ cho quyết định phát hành sau khi build đổi.

## 7. Thực hành

Viết charter mười dòng, chỉ định người chấp nhận và một nhóm người dùng chưa được đại diện. Nêu hạn chế đó ảnh hưởng mức tự tin thế nào.

<details><summary>Có phải UAT pass là chắc chắn tạo giá trị?</summary><p>Không. UAT kiểm khả năng chấp nhận theo phạm vi và điều kiện đã chọn. Adoption, tác động lên hành vi và outcome cần được đo ngoài thực tế với thiết kế phù hợp.</p></details>

## 8. Đọc tiếp

Nguồn tình huống: [gói yêu cầu P04](/articles/ba-thuc-hanh-review-goi-yeu-cau). Tiếp: [thiết kế UAT scenario](/articles/ba-thiet-ke-uat-scenario).
