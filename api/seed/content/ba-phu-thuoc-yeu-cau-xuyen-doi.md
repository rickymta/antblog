---
slug: ba-phu-thuoc-yeu-cau-xuyen-doi
title: "Yêu cầu xuyên nhiều đội: quản lý phụ thuộc bằng hợp đồng bàn giao"
summary: "Làm rõ đầu ra, owner và điều kiện sẵn sàng giữa các đội; xử lý vòng phụ thuộc, thay đổi hợp đồng và việc tích hợp đến muộn."
category: business-analysis
level: advanced
tags: [business-analysis, ba-047, dependency, collaboration, toi-noi]
language: vi
---

Backend báo đã xong, mobile báo đang chờ backend, CS chưa biết ngày đào tạo. Mỗi đội có thể đúng theo định nghĩa riêng của mình, trong khi hành trình khách vẫn chưa dùng được.

<div class="tkh-callout"><span class="tkh-callout-title">BA-047 · Phần 6/8 · Nâng cao</span><p>Đầu ra: dependency register cho đổi chính sách phí. Các đội và lịch trong bài là tình huống giả lập, không mô tả tổ chức thật.</p></div>

## 1. Phụ thuộc là đầu ra cần nhận

Thay “chờ đội API” bằng “cần response mô tả policy áp dụng và ví dụ chuyến qua mốc để mobile hiển thị đúng”. Ghi provider, consumer, artifact, ngày cần và tiêu chí chấp nhận. Tên đội đơn lẻ không cho biết khi nào phụ thuộc được giải toả.

## 2. Bảng bàn giao

| Provider → consumer | Đầu ra cần bàn giao | Điều kiện kiểm tra |
|---|---|---|
| Nghiệp vụ → API/QA | Rule hiệu lực được chốt | Có ca trước/sau/qua mốc |
| API → Mobile | Contract có version và ví dụ lỗi | Mobile đọc được trên môi trường chung |
| QA → release owner | Kết quả case biên theo build | Có bằng chứng, không chỉ số pass |
| Nghiệp vụ/API → CS | Cách tra policy của chuyến | CS giải thích được ca mẫu |
| Analytics → PO | Metric theo cohort policy | Đối chiếu dữ liệu mẫu đúng |

Các trường contract mới vẫn là đề xuất, cần đội kỹ thuật thống nhất; BA không tự thêm tên field rồi coi như hợp đồng đã tồn tại.

## 3. Phân biệt mock và tích hợp thật

Mock giúp mobile phát triển khi API chưa xong, nhưng không chứng minh xác thực, dữ liệu và timing thật hoạt động đúng. Định nghĩa mốc contract-ready, implementation-ready và integration-verified riêng. Một dependency chưa đóng chỉ vì demo với dữ liệu cứng thành công.

## 4. Vòng phụ thuộc cần quyết định

API đợi UI chốt thông báo, UI đợi API chốt mã lỗi. BA có thể tổ chức buổi review cùng ví dụ nghiệp vụ: lỗi nào cần người dùng sửa, lỗi nào phải tra cứu. Chốt outcome và contract tối thiểu trước rồi hai đội triển khai song song; ghi điều còn mở và compatibility cần giữ.

Không giải vòng bằng cách để mỗi đội tự giả định. Hai giả định riêng dễ tạo lỗi chỉ xuất hiện gần ngày phát hành.

## 5. Tác động của thay đổi muộn

Nếu rule hiệu lực đổi, thông báo, analytics và bộ test phải được thông báo cùng phiên bản. Dependency register nối vào traceability để biết ai cần review. Một change log ở API mà consumer không biết chưa đủ để bảo vệ hành trình end-to-end.

## 6. Escalation có lựa chọn

Khi analytics không kịp, trình bày ảnh hưởng: không phân biệt được cohort policy, khó đánh giá kết quả. Đề xuất lùi pilot, thu hẹp phạm vi hoặc phương án đo tạm có thể kiểm chứng. Người quyết định cần thấy rủi ro và điều kiện, không chỉ nhận câu “đội khác đang block”.

## 7. Thực hành

CS chưa có công cụ xem policy của chuyến, nhưng API và mobile đã pass test. Viết hai phương án bàn giao và tiêu chí chọn; đánh giá công suất nếu tra cứu thủ công.

<details><summary>Gợi ý</summary><p>Có thể trì hoãn hoặc dùng quy trình tra cứu giới hạn với owner, quyền truy cập, thời gian đáp ứng và giới hạn số ca rõ. Không coi thủ công là khả thi nếu chưa thử một ca và kiểm năng lực trực.</p></details>

## 8. Đọc tiếp

Ôn [traceability](/articles/ba-traceability-muc-tieu-yeu-cau-test). Tiếp: [bài thực hành đổi phí sát ngày phát hành](/articles/ba-thuc-hanh-doi-phi-sat-ngay-phat-hanh).
