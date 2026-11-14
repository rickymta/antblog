---
slug: ba-pilot-ab-test-tuong-quan-tac-dong
title: "Pilot và A/B test: phân biệt tương quan với tác động"
summary: "Chọn câu hỏi và đơn vị thử nghiệm, nhận diện tự chọn và ảnh hưởng chéo; đọc chênh lệch tỷ lệ với giới hạn thay vì tuyên bố nhân quả quá sớm."
category: business-analysis
level: advanced
tags: [business-analysis, ba-055, experiment, causality, toi-noi]
language: vi
---

Trong dữ liệu giả lập P07, nhóm xác nhận điểm đón huỷ ít hơn nhóm không xác nhận. Có thể tính năng giúp họ, nhưng cũng có thể những người vốn cẩn thận hơn thường xác nhận và ít huỷ hơn.

<div class="tkh-callout"><span class="tkh-callout-title">BA-055 · Phần 7/8 · Nâng cao</span><p>Đầu ra: experiment brief, không phải thiết kế thống kê đã được duyệt. Cần người có chuyên môn thống kê review đơn vị phân nhóm và cỡ mẫu trước thử nghiệm thật.</p></div>

## 1. Tính chênh lệch quan sát

Trong 240 chuyến xác nhận có 18 huỷ: 7,5%. Trong 760 chuyến còn lại có 100 huỷ: khoảng 13,16%. Tổng vẫn là 118/1.000. Chênh lệch khoảng 5,66 điểm phần trăm không phải tác động được chứng minh, vì hai nhóm được hình thành bởi hành vi người dùng.

Không dùng chính sự hoàn thành tính năng để chọn nhóm treatment sau khi kết quả đã bị ảnh hưởng. Nhóm xác nhận có thể khác về mạng, khu vực, mức quen thuộc hoặc độ khó điểm đón.

## 2. Pilot giải đáp điều gì?

Pilot giới hạn giúp tìm vấn đề vận hành, khả năng dùng và chất lượng dữ liệu. Nó có thể cung cấp tín hiệu giá trị, nhưng không tự là thí nghiệm có đối chứng. Muốn ước lượng tác động cần thiết kế so sánh phù hợp và nêu giả định nhận diện nhân quả.

## 3. Chọn đơn vị phân nhóm trước

Đề xuất phân nhóm khách đủ điều kiện trước exposure, giữ phân nhóm ổn định trong thời gian thử. Phân tích theo nhóm được gán giúp tránh chỉ chọn người đã dùng thành công. Nếu đo theo trip, phải xét một khách có nhiều trip và phụ thuộc giữa các quan sát.

Với marketplace, một tài xế phục vụ cả hai nhóm có thể học thói quen mới rồi áp sang nhóm đối chứng. Randomize theo khách không tự loại ảnh hưởng chéo này. Có thể cần thiết kế theo cụm hoặc thời gian, với phân tích phù hợp và đánh đổi về công suất thống kê.

## 4. Chọn outcome không bị đổi mẫu số tuỳ tiện

Nếu tính năng xuất hiện trước accepted, chỉ phân tích chuyến đã accepted có thể bỏ qua tác động lên khả năng được nhận. Experiment brief nên chốt cohort từ lúc đủ điều kiện, rồi theo dõi cả accepted rate và huỷ. Có thể giữ metric P01 cho mục đích so vận hành nhưng phải nêu giới hạn chọn mẫu.

## 5. Metric, guardrail và thời gian

Chọn metric chính, hiệu ứng tối thiểu đáng quan tâm, guardrail và kế hoạch phân tích trước. Cỡ mẫu phụ thuộc baseline, mức biến thiên, đơn vị phân nhóm, công suất mong muốn và thiết kế; không mặc định 1.000 chuyến là đủ. Cần thời gian bao phủ chu kỳ hành vi và độ trễ outcome.

Kiểm phân bổ nhóm và telemetry trước khi đọc effect. Không dừng tuỳ hứng ngay khi đồ thị có màu xanh; quy tắc dừng phải tương thích phương pháp phân tích đã chọn.

## 6. Đọc kết quả có độ bất định

Trình bày effect cùng khoảng bất định và ý nghĩa thực tiễn. Kết quả chưa phân biệt được với không hiệu ứng không chứng minh chắc chắn tính năng vô dụng. Ngược lại, khác biệt có ý nghĩa thống kê có thể quá nhỏ để bù chi phí vận hành.

## 7. Thực hành

Viết brief gồm giả thuyết, eligibility, đơn vị phân nhóm, metric, guardrail, rủi ro ảnh hưởng chéo và điều kiện chất lượng dữ liệu. Gạch chân mọi mục cần chuyên gia xác nhận.

<details><summary>Vì sao không so riêng người đã xác nhận?</summary><p>Họ tự chọn và thường khác nhóm còn lại trước khi dùng tính năng. So như vậy hữu ích để khám phá giả thuyết, nhưng không cô lập tác động của việc cung cấp tính năng.</p></details>

## 8. Nguồn và bài tiếp

[Practical Guide to Controlled Experiments](https://ai.stanford.edu/~ronnyk/GuideControlledExperiments.pdf) về thiết kế thí nghiệm; [nghiên cứu interference](https://arxiv.org/abs/2211.03262) về ảnh hưởng chéo. Tiếp: [UAT đạt nhưng chưa có giá trị](/articles/ba-thuc-hanh-uat-dat-chua-co-gia-tri).
