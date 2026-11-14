---
slug: da-analysis-brief-cau-hoi-kiem-chung
title: "Từ ‘huỷ chuyến tăng’ tới câu hỏi phân tích kiểm chứng được"
summary: "Biến yêu cầu làm báo cáo huỷ chuyến thành analysis brief: xác định quyết định, phạm vi, câu hỏi, bằng chứng, hạn trả lời và tiêu chí hoàn thành qua tình huống Tới Nơi."
category: data-analysis
level: intermediate
tags: [data-analysis, da-004, analysis-brief, scope, toi-noi]
language: vi
---

“Tuần tới họp rồi, em tìm nguyên nhân huỷ tăng và đề xuất xử lý nhé.” Yêu cầu nghe rõ ràng, nhưng ít nhất ba việc đang nằm trong cùng một câu: kiểm tra số huỷ, giải thích biến động và lựa chọn can thiệp. Mỗi việc cần bằng chứng khác nhau và có thể không hoàn thành cùng lúc.

DA cần biến lời đề nghị thành một thoả thuận có thể thực hiện. Tài liệu dùng cho việc đó là **analysis brief**: bản mô tả ngắn về câu hỏi, cách dùng câu trả lời, phạm vi và đầu ra cần giao.

<div class="tkh-callout">
<span class="tkh-callout-title">DA-004 · Phần 1/8 · Trung bình</span>
<p>Đọc trước: <a href="/articles/da-vong-doi-bai-toan-phan-tich">DA-003: vòng đời phân tích</a>. Đầu ra: analysis brief một trang cho ca huỷ chuyến. Sau bài này, bạn tách được câu hỏi mô tả và câu hỏi nhân quả, thu hẹp phạm vi và đặt điều kiện nghiệm thu có thể kiểm tra.</p>
</div>

## 1. Hỏi về quyết định trước khi hỏi về biểu đồ

Người vận hành có thể cần tăng nhân sự xử lý ca, sản phẩm có thể cần chọn cải tiến điểm đón, còn kỹ thuật có thể cần điều tra sự cố trong khâu ghi nhận dữ liệu. Một biểu đồ huỷ chung không tự trả lời được cả ba.

Ba câu hỏi mở đầu hữu ích là: “Anh/chị sẽ chọn hành động nào sau khi nhận kết quả?”, “Nếu số không tăng thì quyết định có khác không?” và “Chúng ta cần biết đến mức mô tả, hay cần chứng minh hiệu quả của một thay đổi?”.

Ví dụ, nếu mục tiêu là bố trí CSKH, số lượt huỷ chỉ là dữ liệu liên quan. Ta còn cần biết tỷ lệ ca cần hỗ trợ, số lần liên hệ và thời gian xử lý. Nếu mục tiêu là đánh giá chat, phải có thiết kế đo tác động; không thể suy ra hiệu quả chat từ hai tổng số trước khi thử tính năng.

[Government Data Quality Framework](https://www.gov.uk/government/publications/the-government-data-quality-framework/the-government-data-quality-framework) đặt chất lượng dữ liệu trong quan hệ với mục đích sử dụng. Trong bài này, brief giúp định nghĩa mục đích đó trước khi nhóm tuyên bố dữ liệu “đủ tốt”.

## 2. Viết câu hỏi có đối tượng, phép đo và phạm vi

Câu “vì sao khách huỷ nhiều?” thiếu đơn vị và tập so sánh. Câu hỏi đầu tiên của DA-P01 có thể viết lại:

> Trong các lượt đặt xe ở khu vực A được nhận lần đầu vào W1 hoặc W2, số và tỷ lệ huỷ trước đón khác nhau thế nào giữa hai kỳ và hai nhóm giờ? Nhóm nào đáng ưu tiên xác minh trước khi chọn can thiệp?

Đơn vị là lượt đặt xe, không phải khách duy nhất. Phạm vi là dịch vụ `ride` ở A, không phải cả ba dịch vụ hay toàn hệ thống. Mốc để xếp cohort là thời điểm nhận lần đầu, không phải ngày phát sinh huỷ. Đầu ra giúp xếp thứ tự ưu tiên điều tra, chưa xác nhận nguyên nhân.

W1/W2 và nhóm cao điểm vẫn theo hồ sơ giả lập P01: chưa có ngày hoặc lịch giờ cụ thể. Brief cho dữ liệu thực tế phải yêu cầu người có trách nhiệm chốt các giá trị này. Một chỗ trống được đánh dấu rõ tốt hơn một con số tự điền mà không ai biết là giả định.

## 3. Tách câu hỏi lớn thành các câu kiểm chứng được

| Mã | Câu hỏi | Bằng chứng cần | Có thể trả lời với P01 hiện tại? |
|---|---|---|---|
| A-01 | Số huỷ và tỷ lệ chung có tăng không? | Tổng được nhận và huỷ cùng cohort | Có, trong giả định bảng tổng hợp |
| A-02 | Diễn biến giữa các khung giờ có khác nhau không? | Bảng phân nhóm đối soát được | Có, ở mức mô tả |
| A-03 | Thay đổi ghi nhận có làm số sai lệch không? | Lịch thay đổi, độ trễ, đối soát nguồn | Chưa |
| A-04 | Thời gian đón liên quan tới huỷ như thế nào? | Lịch sử sự kiện và thiết kế đo không bỏ sót lượt huỷ | Chưa |
| A-05 | Chat có làm giảm huỷ không? | Dữ liệu can thiệp và thiết kế đánh giá tác động | Chưa |

Thứ tự này có chủ đích: kiểm tra cách đo trước khi giải thích hiện tượng. Nếu A-03 phát hiện thiếu dữ liệu ở W2, phải sửa mô tả A-01/A-02 trước khi dùng chúng làm nền cho câu chuyện nguyên nhân.

A-04 cũng cần cẩn thận: chỉ những lượt đã đón mới có “thời gian tới lúc đón” hoàn chỉnh. Lấy trung bình của nhóm đó rồi áp sang nhóm huỷ sẽ bỏ qua các lượt không bao giờ được đón. Brief nên ghi vấn đề đo này để người thực hiện thiết kế đúng cách đo, thay vì yêu cầu một cột mà dữ liệu không thể có.

## 4. Một analysis brief mẫu

Đây là **đề xuất học tập**, chưa có quyết định triển khai hoặc ngày họp thật.

| Trường | Nội dung cho Tới Nơi |
|---|---|
| Người sử dụng | Đại diện vận hành phụ trách A; cần gán người cụ thể khi triển khai |
| Quyết định | Có dành nguồn lực xác minh nhóm cao điểm, và cần xác minh điều gì trước? |
| Câu hỏi | A-01/A-02; lập kế hoạch kiểm tra A-03/A-04 |
| Phạm vi | Cohort nhận lần đầu W1/W2, dịch vụ ride, khu vực A |
| Ngoài phạm vi | Chứng minh hiệu quả chat; dự báo toàn hệ thống; tính doanh thu; đổi database |
| Đầu vào | Bảng tổng hợp DA-P01 và danh sách câu hỏi mở Q-01–Q-05 |
| Đầu ra | Bảng đối soát, nhận định một trang, giới hạn và kế hoạch dữ liệu tiếp theo |
| Hạn đề xuất | T+3 ngày làm việc sau khi xác nhận phạm vi và khả năng cung cấp dữ liệu |
| Điểm kiểm tra | Cuối T+1 rà soát phần dữ liệu còn thiếu; báo lại về phạm vi nếu dữ liệu không đủ |
| Tiêu chí hoàn thành | Phép tính kiểm tra được; tách kết luận và giả thuyết; mỗi thiếu hụt có người phối hợp và bước tiếp |
| Trạng thái | Bản nháp để người có thẩm quyền xác nhận |

T là mốc giả lập dùng để lập kế hoạch, không phải cam kết hoàn thành mọi phân tích trong ba ngày. Nếu nguồn chưa sẵn sàng, nhóm có thể giao báo cáo hiện trạng và kế hoạch bổ sung trước, nhưng phải nói rõ phần nào chưa trả lời.

## 5. Tiêu chí nghiệm thu phải kiểm tra được

“Có insight hay” là tiêu chí khó dùng. Hãy thay bằng các điều kiện cụ thể: số tổng khớp số phân nhóm; mẫu số được ghi; nhận định chỉ ra phạm vi; nguồn và giả định có thể tra lại; câu hỏi chưa trả lời được không bị che đi.

Trong ca này, một nhận định đạt yêu cầu phải nêu được: W1 là 120/1.000, W2 là 180/1.500, cùng 12%; cao điểm từ 16% lên 18%; chưa có bằng chứng giải thích nguyên nhân. Chỉ viết “huỷ tăng 50%” là thiếu câu trả lời về tỷ lệ và phân nhóm.

Việc nghiệm thu sản phẩm phân tích không có nghĩa là đồng ý với mọi khuyến nghị. Vận hành có thể chấp nhận báo cáo mô tả nhưng chưa duyệt bước điều tra vì nguồn lực. Cần ghi hai trạng thái riêng.

## 6. Xử lý yêu cầu phát sinh

Giữa chừng, người nhận đề nghị “thêm giao đồ ăn, sáu thành phố và tính chi phí huỷ”. DA cần xác định việc đó có đổi quyết định đang hỗ trợ không. Nếu có, lập phiên bản brief mới với nguồn, định nghĩa và thời gian tương ứng.

Không cộng `food` và `parcel` vào `ride` rồi tiếp tục dùng cùng câu kết luận về điểm đón khách. Những dịch vụ có hành trình, lý do huỷ và chi phí khác nhau. Cũng không coi giá chuyến là thiệt hại: phải định nghĩa đó là chi phí nào, phát sinh ở đâu và có thực sự tránh được bằng can thiệp hay không.

Mẫu ghi thay đổi ngắn: **CR-DA-01 — đề nghị mở rộng phạm vi; tác động: thêm nguồn và định nghĩa; lựa chọn: giữ thời hạn với phạm vi cũ hoặc đổi thời hạn, phạm vi; trạng thái: chờ xác nhận.** Việc ghi nhận yêu cầu không tự động biến nó thành yêu cầu đã duyệt.

## 7. Bài tập và gợi ý review

Viết brief tối đa một trang từ lời đề nghị: “Chứng minh chat sẽ giảm 20% huỷ cao điểm để tuần tới xin triển khai.” Giữ nhu cầu ra quyết định, sửa phần kết luận áp đặt, ghi ít nhất hai câu hỏi có thể trả lời ngay và hai khoảng trống bằng chứng.

<details><summary>Gợi ý lời giải</summary><p>Viết lại quyết định thành việc đánh giá xem có nên dành nguồn lực tìm hiểu hoặc thử chat hay không. Câu hỏi trả lời ngay được: số/tỷ lệ huỷ thay đổi thế nào; nhóm giờ nào cần xác minh. Khoảng trống: nguyên nhân huỷ chưa xác nhận; chưa có dữ liệu can thiệp hay thiết kế đánh giá. Mốc 20% là giả định hoặc mục tiêu cần thảo luận, không phải kết quả phải chứng minh. Đầu ra có thể là kế hoạch đo cùng phương án thay thế, chưa phải khuyến nghị rollout.</p></details>

**Câu hỏi ôn tập:** Nếu người nhận bảo “cứ lấy tất cả dữ liệu rồi tìm điều thú vị”, DA nên làm gì?

<details><summary>Gợi ý trả lời</summary><p>Đề xuất một phạm vi khám phá có giới hạn, mục đích, thời gian và đầu ra. Ghi rằng các phát hiện là khám phá cần kiểm chứng tiếp. Không giả vờ đã có câu hỏi xác nhận từ trước, cũng không coi việc tìm nhiều phân nhóm là bằng chứng cho nhân quả.</p></details>

Checklist: biết người dùng kết quả; biết quyết định; có cohort/thời gian; có phạm vi trong và ngoài; có nguồn và giới hạn; có thời hạn gắn với điều kiện; có tiêu chí nghiệm thu và trạng thái xác nhận.

## 8. Tham khảo và bước tiếp

- [Government Data Quality Framework](https://www.gov.uk/government/publications/the-government-data-quality-framework/the-government-data-quality-framework): dữ liệu phù hợp mục đích và nhu cầu người sử dụng.
- [DA-003: vòng đời phân tích](/articles/da-vong-doi-bai-toan-phan-tich).

Mẫu nộp bài: `examples/toi-noi-da/p01-ke-hoach-phan-tich.md`. Bài tiếp theo: [DA-005: KPI, metric và chỉ số bảo vệ](/articles/da-kpi-metric-chi-so-bao-ve).
