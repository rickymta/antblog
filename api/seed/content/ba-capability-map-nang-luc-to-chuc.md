---
slug: ba-capability-map-nang-luc-to-chuc
title: "Capability map: tổ chức cần làm tốt việc gì?"
summary: "Phân biệt năng lực nghiệp vụ với quy trình và hệ thống, dùng bản đồ năng lực để tìm khoảng trống về người, cách làm và công cụ."
category: business-analysis
level: beginner
tags: [business-analysis, ba-058, capability-map, strategy, toi-noi]
language: vi
---

Tới Nơi có màn hình hoàn tiền chưa chắc đã có năng lực xử lý hoàn tiền đáng tin cậy. Nếu không ai đối soát ngoại lệ và không có quyền quyết định, công cụ chỉ giải quyết một phần vấn đề.

<div class="tkh-callout"><span class="tkh-callout-title">BA-058 · Phần 8/8 · Cơ bản</span><p>Đầu ra: capability map nhỏ và bảng khoảng trống. Mức trưởng thành dưới đây là đánh giá giả lập, không phải audit tổ chức thật.</p></div>

## 1. Năng lực mô tả điều cần làm được

Capability là khả năng tổ chức thực hiện một công việc nghiệp vụ để tạo kết quả. “Quản lý hoàn tiền” là năng lực; quy trình mô tả các bước từ yêu cầu tới đối soát; hệ thống là công cụ hỗ trợ các bước. Tên một phần mềm không đủ để mô tả năng lực.

Phân biệt này giúp nhóm cân nhắc đào tạo hoặc đổi quyền quyết định thay vì luôn mua thêm công cụ.

## 2. Bản đồ rút gọn cho Tới Nơi

| Nhóm năng lực | Năng lực con minh họa |
|---|---|
| Phục vụ chuyến | Tiếp nhận nhu cầu, điều phối, hỗ trợ điểm đón |
| Quản lý giao dịch | Báo giá, thu tiền, xử lý hoàn tiền đề xuất |
| Hỗ trợ khách | Tiếp nhận ca, truy vết, giải quyết ngoại lệ |
| Cải tiến dịch vụ | Đo outcome, nghiên cứu người dùng, quản lý thay đổi |

Bản đồ không quy định thứ tự xử lý. Cùng một năng lực có thể xuất hiện trong nhiều quy trình và được nhiều đội cùng hỗ trợ.

## 3. Đánh giá bằng bằng chứng

Đề xuất thang học tập: chưa xác định, làm được nhờ cá nhân, có quy trình lặp lại, có đo lường và cải tiến. Muốn chấm “có quy trình” phải có bằng chứng như owner, hướng dẫn và ca diễn tập; không chỉ vì tài liệu đã được viết.

Không cộng các mức thành một điểm tổng chính xác giả tạo. Mục đích là tìm chỗ cần đầu tư và căn cứ đánh giá.

## 4. Tách người, quy trình và công cụ

Với hoàn tiền P05, có thể thiết kế contract tốt nhưng chưa có người xử lý RF-C không khớp đối tác. Khoảng trống là ownership và quy trình ngoại lệ. Ngược lại, có người giỏi nhưng thiếu khóa nối refundId/providerRef thì công cụ và dữ liệu cũng cần thay đổi.

Viết mỗi khoảng trống theo bằng chứng: “chưa có owner nhận giao dịch chờ quá hạn”, thay vì nhận xét chung “vận hành yếu”.

## 5. Đặt ưu tiên theo chiến lược

Nếu mục tiêu gần là pilot điểm đón, năng lực hỗ trợ và đo lường có thể cần trước hoàn tiền tự động. Nếu mở giao hàng nhiều điểm, điều phối và xử lý giao một phần trở nên quan trọng. Bản đồ ổn định tương đối, nhưng thứ tự đầu tư phụ thuộc mục tiêu.

## 6. Tránh biến bản đồ thành sơ đồ tổ chức

Một đội có thể sở hữu nhiều năng lực, một năng lực có thể đi qua nhiều đội. Ghi accountable owner để tránh bỏ trống, nhưng không chia năng lực chỉ theo phòng ban hiện tại. Nếu tái tổ chức, công việc nghiệp vụ vẫn cần được thực hiện.

## 7. Thực hành

Chọn năng lực giải quyết ngoại lệ hoàn tiền. Ghi outcome, ba yếu tố hỗ trợ, bằng chứng hiện có và khoảng trống ưu tiên. Chỉ dùng hồ sơ P05, đánh dấu mọi điều chưa biết.

<details><summary>Gợi ý</summary><p>Outcome là ngoại lệ được xác minh và xử lý có dấu vết. Cần owner có quyền, quy trình đối soát và dữ liệu liên kết. Hồ sơ đề xuất chưa chứng minh bất kỳ năng lực nào đã vận hành thật.</p></details>

## 8. Đọc tiếp

[IIBA KnowledgeHub](https://www.iiba.org/knowledgehub/the-business-analysis-standard/) liệt kê business capability analysis trong các kỹ thuật BA; bản đồ trên là ví dụ riêng. Tiếp: [risk và assumption log](/articles/ba-risk-assumption-log).
