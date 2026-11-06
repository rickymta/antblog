---
slug: ba-traceability-muc-tieu-yeu-cau-test
title: "Traceability: nối mục tiêu, yêu cầu và bằng chứng kiểm tra"
summary: "Lập ma trận truy vết hai chiều để tìm yêu cầu thiếu lý do, test thiếu căn cứ và phạm vi bị ảnh hưởng khi thay đổi."
category: business-analysis
level: beginner
tags: [business-analysis, ba-043, traceability, requirements, toi-noi]
language: vi
---

Một test màu xanh chỉ chứng minh điều đã được kiểm trong điều kiện cụ thể. Muốn biết test đó bảo vệ nhu cầu nào, nhóm cần đường nối về yêu cầu và mục tiêu.

<div class="tkh-callout"><span class="tkh-callout-title">BA-043 · Phần 6/8 · Cơ bản</span><p>Đọc P04 về điểm đón. Đầu ra: ma trận truy vết nhỏ, có loại quan hệ và trạng thái bằng chứng. Các test UAT dưới đây mới là ca đề xuất.</p></div>

## 1. Truy vết hai chiều

Đi xuôi từ mục tiêu tới yêu cầu và test giúp hỏi: mục tiêu này được giải quyết bằng gì? Đi ngược từ một field hay test giúp hỏi: ai cần nó, rule nào đòi hỏi? Cả hai chiều đều cần khi quyết định bỏ hoặc sửa một yêu cầu.

Liên kết không nhất thiết một-một. Một rule có nhiều test, một test có thể đi qua nhiều yêu cầu. Đừng ép mọi thứ vào một dòng rồi mất quan hệ.

## 2. Bảng mẫu có giới hạn rõ

| Nhu cầu/giả thuyết | Yêu cầu P04 | Ví dụ kiểm đề xuất | Trạng thái |
|---|---|---|---|
| Giảm hiểu sai điểm đón | US-P04-01 xác nhận | UAT-P07-01 đúng revision | Chưa chạy |
| Bảo vệ quyền chủ chuyến | RULE-P04-01 | UAT-P07-02 người khác xác nhận | Chưa chạy |
| Không xác nhận thông tin cũ | RULE-P04-02 | UAT-P07-03 stale revision | Chưa chạy |
| Tài xế biết điểm đã xác nhận | US-P04-02 | UAT-P07-04 tài xế đọc | Chưa chạy |

Hàng đầu chỉ thể hiện giả thuyết đóng góp vào giá trị. Test pass chưa chứng minh tỷ lệ huỷ giảm; cần dữ liệu sau phát hành và thiết kế đánh giá riêng.

## 3. Ghi đúng loại liên kết

Các từ “xuất phát từ”, “triển khai”, “kiểm tra” và “phụ thuộc” khác nhau. Nối mọi thứ bằng một từ “related” khiến ma trận khó dùng khi thay đổi. Tối thiểu giữ ID nguồn, ID đích, loại quan hệ, phiên bản và người duy trì.

Có thể bắt đầu bằng Markdown hoặc bảng tính. Công cụ chuyên dụng chỉ hữu ích khi quan hệ được cập nhật đúng.

## 4. Tìm mục bị mồ côi

Một nút “chia sẻ mọi ghi chú với CS” chưa có nhu cầu được chấp nhận không nên tự trở thành phạm vi. Ngược lại, yêu cầu kiểm revision chưa có ca kiểm chứng là lỗ hổng coverage. Báo cáo hai loại này riêng vì cách xử lý khác nhau: tìm căn cứ hoặc bổ sung kiểm chứng.

## 5. Áp dụng khi đổi rule

Nếu đề xuất cho phép sửa sau accepted, từ RULE-P04-01 lần theo use case, wireframe, API, quyền đọc của tài xế, test và hướng dẫn CS. Đánh dấu từng liên kết cần review, không sửa toàn bộ một cách máy móc. Một số tài liệu chỉ cần xác nhận không ảnh hưởng.

## 6. Tránh tỷ lệ coverage gây hiểu nhầm

“100% yêu cầu có test” có thể chỉ nghĩa đã tạo tên test. Phân biệt designed, executed, passed và accepted; gắn phiên bản build/dữ liệu khi có kết quả. Ngoài ra kiểm chiều sâu: một happy path không bao phủ đầy đủ rule cạnh tranh hoặc ngoại lệ.

## 7. Thực hành

Thêm AC-P04-05 về timeout vào ma trận. Tạo hai ca: response mất nhưng xác nhận đã lưu, và yêu cầu chưa tới server. Ghi điều UI có thể nói trước khi tra cứu.

<details><summary>Gợi ý tự kiểm</summary><p>Cả hai ca đều có thể trông giống timeout ở client. UI chưa được tuyên bố thất bại chắc chắn; traceability phải nối hành vi tra cứu với yêu cầu xử lý kết quả chưa rõ.</p></details>

## 8. Đọc tiếp

Gói [yêu cầu P04](/articles/ba-thuc-hanh-review-goi-yeu-cau) là nguồn ID trong bài; ma trận là ví dụ tự xây của series. Tiếp: [baseline, version và approval](/articles/ba-baseline-version-approval).
