---
slug: ba-dashboard-do-ket-qua-sau-phat-hanh
title: "Đo kết quả sau phát hành: dashboard có trả lời đúng câu hỏi?"
summary: "Định nghĩa cohort, tử số, mẫu số và cửa sổ đo; phân biệt exposure, adoption và outcome qua dữ liệu pilot điểm đón giả lập."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-054, metrics, dashboard, toi-noi]
language: vi
---

Dashboard hiện 720 lượt mở tính năng. PO hỏi bao nhiêu chuyến đã được hỗ trợ và tỷ lệ huỷ có giảm không. Con số lượt mở chưa trả lời được hai câu đó.

<div class="tkh-callout"><span class="tkh-callout-title">BA-054 · Phần 7/8 · Trung bình</span><p>Đầu ra: metric dictionary và bản đọc số liệu P07. Toàn bộ số pilot là dữ liệu giả lập mới, không phải kết quả của website hoặc lab.</p></div>

## 1. Xác định câu hỏi trước biểu đồ

Nhóm cần biết khách có thấy tính năng, có xác nhận, tài xế có đọc và outcome có thay đổi không. Đây là các bước khác nhau. Nếu chỉ tăng số click, nhóm có thể tối ưu một màn hình mà không cải thiện việc gặp nhau ở điểm đón.

## 2. Chốt cohort và cửa sổ

P07 dùng 1.000 chuyến đủ điều kiện có lần accepted đầu tiên trong tuần B theo Asia/Ho_Chi_Minh. Snapshot lấy sau cuối tuần bảy ngày; giả định tất cả đã có outcome quan sát đầy đủ. Huỷ trước pickup tính một lần cho mỗi trip trong cohort. Event xem/xác nhận trước accepted được nối về cùng trip.

Định nghĩa này loại chuyến chưa từng accepted. Vì vậy không dùng tỷ lệ của cohort để kết luận về mọi booking hoặc việc tính năng có ảnh hưởng khả năng được nhận hay không.

## 3. Đọc funnel đúng mẫu số

| Metric | Số giả lập | Tỷ lệ |
|---|---:|---:|
| Chuyến thấy tính năng | 600/1.000 | 60% cohort |
| Chuyến xác nhận | 240/600 | 40% chuyến đã thấy |
| Chuyến có tài xế đọc sau xác nhận | 180/240 | 75% chuyến xác nhận |
| Chuyến huỷ trước pickup | 118/1.000 | 11,8% cohort |

240/1.000 = 24% là adoption trên toàn cohort, khác 40%. 720 event mở thuộc 600 trip, nên không dùng 720 làm số chuyến thấy. Tài xế mở màn hình cũng chưa chứng minh họ hiểu hoặc làm theo thông tin.

## 4. Metric dictionary tối thiểu

Ghi tên, mục đích, đơn vị, grain, tử số, mẫu số, bộ lọc, timezone, thời gian quan sát, nguồn và owner. Event schema cần version và quy tắc dedup. Nếu có late event, mô tả dashboard cập nhật lại số cũ hay chốt snapshot để người đọc hiểu chênh lệch.

## 5. Kiểm chất lượng dữ liệu

Đối chiếu vài trip từ log đến dashboard; tìm event thiếu tripId, duplicate, timestamp sai và sự kiện phát trước khi dữ liệu được lưu. Phân biệt không có sự kiện đọc với chưa đo được sự kiện đọc. Một tỷ lệ đẹp từ telemetry lỗi không phải bằng chứng sản phẩm tốt.

## 6. So baseline có giới hạn

P01 tuần A có 120/1.000 = 12%; tuần B là 11,8%, thấp hơn 0,2 điểm phần trăm. Hai tuần không được phân nhóm ngẫu nhiên, có thể khác thời tiết, khu vực hoặc nguồn tài xế. Chưa thể quy chênh lệch cho tính năng. Guardrail khiếu nại chưa có baseline đáng tin thì ghi chưa đủ đánh giá.

## 7. Thực hành

Viết ba chú thích cho dashboard để tránh đọc nhầm 720 event, 40% adoption và mức giảm 0,2 điểm phần trăm.

<details><summary>Gợi ý</summary><p>Hiển thị 600 unique trip thay vì gọi event là chuyến; ghi 40% trong nhóm đã thấy và 24% toàn cohort; mô tả so tuần chỉ là quan sát, chưa phải ước lượng tác động nhân quả.</p></details>

## 8. Nguồn và bài tiếp

[GOV.UK — Measuring success](https://www.gov.uk/service-manual/measuring-success/measuring-the-success-of-your-service) khuyến khích kết hợp metric với nghiên cứu người dùng. Tiếp: [pilot và A/B test](/articles/ba-pilot-ab-test-tuong-quan-tac-dong).
