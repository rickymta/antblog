---
slug: ba-baseline-version-approval
title: "Baseline, phiên bản và phê duyệt: biết nhóm đã chốt điều gì"
summary: "Tách bản nháp, bản được chấp nhận và phiên bản có hiệu lực; lưu quyết định đủ rõ để triển khai, nghiệm thu và giải thích thay đổi."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-044, baseline, versioning, toi-noi]
language: vi
---

Tài liệu có tên final-v3 không đảm bảo mọi người đang dùng cùng một chính sách. Với phí huỷ, nhầm phiên bản có thể khiến khách nhìn một điều kiện còn hệ thống tính theo điều kiện khác.

<div class="tkh-callout"><span class="tkh-callout-title">BA-044 · Phần 6/8 · Trung bình</span><p>Đầu ra: hồ sơ baseline và decision log cho CR-BA-01. Mốc áp dụng trong bài là phương án để review, chưa thay đổi SPEC lab.</p></div>

## 1. Ba điều cần tách

Phiên bản nhận diện nội dung cụ thể. Phê duyệt ghi người có thẩm quyền đã chấp nhận điều gì, với điều kiện nào. Baseline là tập thông tin được chọn làm mốc tham chiếu cho công việc tiếp theo. Một tài liệu có thể có phiên bản mới nhưng chưa được duyệt, hoặc được duyệt nhưng chưa có hiệu lực vận hành.

Git commit giúp nhận diện và xem diff nội dung; tự nó không thay thế chữ ký hoặc quyết định nghiệp vụ cần thiết.

## 2. Mẫu baseline nhỏ

| Trường | Ví dụ học tập |
|---|---|
| Baseline | BL-P06-01, bản phân tích hiện hành |
| Rule nền | accepted miễn phí khi elapsed < 120 giây |
| Thay đổi đang xét | CR-BA-01 đề xuất < 180 giây |
| Tài liệu đi cùng | Decision table, state model, thông báo, trường hợp biên |
| Trạng thái CR | Chưa duyệt |
| Người quyết định | Đại diện nghiệp vụ có thẩm quyền, cần xác nhận danh tính thực tế |

Không ghi CR vào baseline đang dùng rồi chỉ chú thích nhỏ “draft”. Người triển khai cần phân biệt ngay trạng thái từng phần.

## 3. Phê duyệt phải có nội dung

“OK nhé” dưới một cuộc chat dài không rõ duyệt giá, phạm vi hay ngày phát hành. Bản ghi nên có ID quyết định, nội dung/diff, phạm vi, điều kiện, người quyết định, thời điểm và bằng chứng xác nhận. Nếu chỉ duyệt phân tích tiếp, đừng đánh dấu sẵn sàng triển khai.

## 4. Thời điểm hiệu lực là một rule

Phương án A áp chính sách theo thời điểm accepted; phương án B theo thời điểm khách huỷ. Chuyến nhận trước mốc bật và huỷ sau mốc sẽ cho kết quả khác nhau giữa hai phương án. Cần chọn và mô tả rõ, không để developer tự quyết từ cách code tiện nhất.

Nếu chọn snapshot theo lần accepted, phải phân tích trường hợp tài xế rời, chuyến về searching rồi được nhận lại. Có snapshot mới không? Đó là câu hỏi thiết kế chưa được baseline hiện tại trả lời.

## 5. Giữ lịch sử và replay

Kết quả đã trả cho yêu cầu huỷ theo cơ chế idempotency của lab phải được hiểu theo hợp đồng replay hiện hành. Không tự tính lại vì chính sách mới vừa bật. Lưu đủ dấu vết để giải thích kết quả cũ; trường policyVersion cho thay đổi tương lai là đề xuất cần thiết kế, không tuyên bố đã có trong lab.

## 6. Khi rút lại quyết định

Rollback phần mềm không tự thu hồi tiền hay thông báo đã gửi. Decision log cần phân biệt tắt áp dụng cho giao dịch mới và xử lý giao dịch đã chịu chính sách. Muốn điều chỉnh khoản tiền cũ phải có phạm vi, quyền và quy trình riêng.

## 7. Thực hành

Viết decision log cho tình huống PO đồng ý thử chính sách nhưng chưa chốt cách xử lý chuyến đang accepted. Đánh dấu phần nào được tiếp tục và phần nào còn chờ.

<details><summary>Gợi ý</summary><p>Có thể tiếp tục phân tích/ước lượng các phương án. Chưa coi rule hiệu lực đủ rõ để triển khai. Gán người phụ trách và hạn chốt cách xử lý chuyến qua mốc, thay vì diễn giải sự đồng ý chung thành phê duyệt toàn bộ.</p></details>

## 8. Đọc tiếp

Baseline nguồn: [thực hành chính sách huỷ](/articles/ba-thuc-hanh-thay-doi-chinh-sach-huy). Tiếp: [change request và impact analysis](/articles/ba-change-request-impact-analysis).
