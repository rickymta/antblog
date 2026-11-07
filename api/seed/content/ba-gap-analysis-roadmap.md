---
slug: ba-gap-analysis-roadmap
title: "Gap analysis và roadmap: từ khoảng trống đến đợt thay đổi"
summary: "Nối hiện trạng, trạng thái đích và năng lực còn thiếu vào roadmap có điều kiện, tránh biến roadmap thành danh sách tính năng kèm ngày hứa."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-060, gap-analysis, roadmap, toi-noi]
language: vi
---

Roadmap ghi “tháng tới chat, tháng sau AI” chưa giải thích vì sao các việc đó giúp khách gặp tài xế dễ hơn. Phân tích khoảng trống bắt đầu từ kết quả cần đạt và điều hiện trạng còn thiếu.

<div class="tkh-callout"><span class="tkh-callout-title">BA-060 · Phần 8/8 · Trung bình</span><p>Đầu ra: roadmap ba đợt có điều kiện qua cổng. Dùng dữ liệu giả lập P07; không đặt lịch phát hành thật hoặc tự phê duyệt đầu tư.</p></div>

## 1. Hiện trạng phải có nguồn

P07 cho thấy adoption toàn cohort 24%, telemetry cần xác minh và guardrail thiếu baseline. Đây là dữ kiện/giới hạn của bài tập. “Người dùng không thích tính năng” là diễn giải chưa có bằng chứng; không đưa vào cột hiện trạng như sự thật.

## 2. Mô tả đích bằng năng lực và outcome

Đích đề xuất: người đủ điều kiện hiểu cách xác nhận; tài xế tìm được thông tin đúng; CS xử lý ngoại lệ; nhóm đo được tác động và rủi ro. Mục tiêu 9% của P01 vẫn cần review về tính khả thi và thiết kế đo, không tự trở thành điều kiện chắc chắn đạt sau roadmap.

## 3. Bảng khoảng trống

| Hiện trạng giả lập | Đích đề xuất | Khoảng trống | Việc cần làm |
|---|---|---|---|
| Event có thể thiếu/lặp | Metric có thể kiểm chứng | Chất lượng đo | Đối chiếu và sửa instrumentation |
| Chưa biết vì sao không xác nhận | Hiểu rào cản chính | Bằng chứng hành vi | Quan sát nhóm không dùng |
| CS chưa chứng minh công suất | Hỗ trợ trong giới hạn pilot | Năng lực vận hành | Diễn tập, đo thời gian xử lý |
| So trước/sau chưa có đối chứng | Đánh giá effect phù hợp | Thiết kế thử nghiệm | Review với analytics/statistics |

Một khoảng trống có thể cần nhiều hoạt động và không phải hoạt động nào cũng là phát triển phần mềm.

## 4. Roadmap theo đợt học và thay đổi

Đợt A làm dữ liệu đáng tin và xác định rào cản. Đợt B thử cải thiện ưu tiên cùng quy trình hỗ trợ. Đợt C đánh giá để quyết định mở rộng, chỉnh hoặc dừng. Không hứa C chắc chắn là rollout: nó là một quyết định dựa trên bằng chứng từ A/B.

Trong từng đợt, owner và estimate phải được nhóm thực hiện xác nhận. “Now/Next/Later” cũng cần mục tiêu và điều kiện chuyển, nếu không chỉ là lịch mơ hồ.

## 5. Sắp thứ tự theo phụ thuộc

Đo hiệu quả cải thiện trước khi telemetry đáng tin tạo ra học sai. Đào tạo CS trước khi chốt ý nghĩa trạng thái có thể phải làm lại. Dùng dependency register P06 để tìm điều kiện tiên quyết, đồng thời cho phép các việc độc lập như nghiên cứu người dùng và kiểm pipeline chạy song song.

## 6. Cập nhật roadmap có lý do

Nếu quan sát cho thấy cổng đóng là nguyên nhân chính, roadmap có thể chuyển sang cập nhật thông tin vận hành thay vì chat. Ghi bằng chứng mới, tác động và quyết định bỏ/hoãn. Thay roadmap không phải thất bại nếu nó phản ánh điều học được và được truyền đạt rõ.

## 7. Thực hành

Viết điều kiện kết thúc từng đợt A/B/C, gồm bằng chứng cần có và người quyết định. Thêm một nhánh dừng nếu lợi ích không đủ bù chi phí hoặc guardrail xấu đi.

<details><summary>Điểm tự kiểm</summary><p>Mỗi hoạt động phải nối được về một khoảng trống và outcome. Nếu chỉ có tên tính năng, hãy hỏi cần năng lực gì, bằng chứng thiếu gì và lựa chọn không xây là gì.</p></details>

## 8. Đọc tiếp

Ôn [capability map](/articles/ba-capability-map-nang-luc-to-chuc). Tiếp: [trình bày bằng bằng chứng](/articles/ba-facilitation-ke-chuyen-bang-chung).
