---
slug: ba-thuc-hanh-uat-dat-chua-co-gia-tri
title: "Thực hành BA: tính năng đạt UAT nhưng chưa chứng minh giá trị"
summary: "Chẩn đoán adoption, chất lượng đo và giả thuyết giải pháp từ pilot giả lập; đề xuất bước tiếp có bằng chứng thay vì vội xây thêm tính năng."
category: business-analysis
level: advanced
tags: [business-analysis, ba-056, case-study, solution-evaluation, toi-noi]
language: vi
---

Giả sử bộ UAT đã được nghiệp vụ chấp nhận theo phạm vi điểm đón. Sau một tuần pilot, PO hỏi vì sao chưa thấy tỷ lệ huỷ về 9%. BA cần phân tích khoảng cách từ khả năng hoạt động đến outcome mong muốn.

<div class="tkh-callout"><span class="tkh-callout-title">BA-056 · Phần 7/8 · Nâng cao</span><p>Đầu ra: evaluation brief và kế hoạch học tiếp. UAT pass và dữ liệu pilot đều là giả định trong hồ sơ P07, không phải kết quả triển khai thật.</p></div>

## 1. Tập hợp bằng chứng và giới hạn

Tuần B có 1.000 chuyến trong cohort accepted, 600 thấy tính năng, 240 xác nhận, 180 có tài xế mở thông tin sau xác nhận, 118 huỷ trước pickup. Mục tiêu P01 là đề xuất 9% sau bốn tuần, chưa phải dự báo được bảo đảm. Một tuần quan sát không đủ để kết luận đạt hoặc thất bại theo thiết kế chưa hoàn thiện.

## 2. Kiểm dữ liệu trước giải thích

720 event mở nhưng chỉ 600 trip là do có lần mở lặp trong bộ giả lập. Kiểm pipeline tương tự trước khi nói khách “không dùng”. Nếu event đọc của tài xế chưa được ghi trên một phiên bản app, con số 180 có thể phản ánh thiếu đo; cần xác minh thay vì đi thẳng tới kết luận tài xế bỏ qua.

## 3. Tách các giả thuyết

| Quan sát | Giả thuyết cần kiểm | Cách học tiếp |
|---|---|---|
| 400 chuyến không thấy tính năng | Điều kiện hiển thị hoặc telemetry thiếu | Đối chiếu rule và log mẫu |
| 360 thấy nhưng không xác nhận | Không hiểu lợi ích hoặc điểm đã rõ | Quan sát/phỏng vấn theo nhóm |
| 60 xác nhận chưa có event tài xế đọc | Khó tìm thông tin hoặc thiếu đo | Kiểm app version và hành trình |
| Huỷ vẫn 11,8% | Nguyên nhân ngoài điểm đón | Review lý do và mẫu tình huống |

Mỗi hàng là giả thuyết, không phải chẩn đoán đã xác nhận. Đừng chỉ phỏng vấn người dùng hài lòng và bỏ những người không dùng.

## 4. Kiểm giới hạn của giải pháp

Xác nhận giúp hai bên có cùng thông tin, nhưng không chữa được cổng bị đóng, đường cấm hoặc khách đặt nhầm khu vực. Nếu các nguyên nhân này phổ biến, cần đổi quy trình hoặc thông tin vận hành. Xây thêm chat không tự là bước đúng; phải so với phương án giải quyết nguyên nhân đã được kiểm chứng.

## 5. Đừng đọc tương quan thành thành công

Nhóm xác nhận huỷ 7,5%, nhóm còn lại khoảng 13,16%. Chênh lệch bị ảnh hưởng tự chọn, nên chưa thể báo cáo “tính năng giảm huỷ 5,66 điểm phần trăm”. So tuần A 12% với tuần B 11,8% cũng không cô lập tác động. Guardrail khiếu nại thiếu baseline thì chưa đủ căn cứ mở rộng.

## 6. Khuyến nghị có điều kiện

Ưu tiên kiểm telemetry, quan sát người không xác nhận và xác minh nguyên nhân huỷ; đồng thời hoàn thiện thiết kế đánh giá tác động. Chưa khuyến nghị mở rộng chỉ dựa trên số hiện có. Nếu phát hiện lỗi quyền hoặc ảnh hưởng vận hành nghiêm trọng, xử lý theo điều kiện dừng đã thống nhất, không đợi hết kế hoạch nghiên cứu.

## 7. Bài nộp

Nộp một trang cho PO gồm ba kết luận được dữ liệu hỗ trợ, ba điều chưa biết và hai hoạt động học tiếp có owner. Kèm metric dictionary và câu hỏi nghiên cứu; không tự thêm số liệu để làm khuyến nghị chắc chắn hơn.

<details><summary>Kết luận mẫu</summary><p>Tính năng đã được chấp nhận trong giả định UAT, nhưng adoption toàn cohort mới 24% và dữ liệu chưa chứng minh tác động nhân quả. Cần xác minh chất lượng đo và rào cản sử dụng trước quyết định mở rộng.</p></details>

## 8. Tổng kết phần 7

Ôn [dashboard](/articles/ba-dashboard-do-ket-qua-sau-phat-hanh) và [pilot](/articles/ba-pilot-ab-test-tuong-quan-tac-dong). Phần cuối đưa các bằng chứng vào business case, roadmap và hồ sơ capstone để bảo vệ quyết định đầu tư.
