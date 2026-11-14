---
slug: ba-thuc-hanh-doi-phi-sat-ngay-phat-hanh
title: "Thực hành BA: đổi chính sách phí sát ngày phát hành"
summary: "Tổng hợp ưu tiên, baseline, impact analysis và phụ thuộc để đưa ra khuyến nghị có căn cứ khi một thay đổi phí đến muộn."
category: business-analysis
level: advanced
tags: [business-analysis, ba-048, case-study, release, toi-noi]
language: vi
---

Hai ngày trước mốc phát hành giả lập, stakeholder muốn thêm CR-BA-01 vào gói đang chuẩn bị. Nhóm đã kiểm tra rule cũ, chưa chốt chuyến qua mốc và CS vẫn dùng hướng dẫn hai phút.

<div class="tkh-callout"><span class="tkh-callout-title">BA-048 · Phần 6/8 · Nâng cao</span><p>Đầu ra: decision brief, impact map và kế hoạch điều kiện. Hồ sơ P06 ghi tình huống học tập; không có quyết định triển khai thật được đưa ra trong bài.</p></div>

## 1. Đọc dữ kiện, không tự điền chỗ trống

CR chỉ đổi miễn phí accepted từ dưới hai lên dưới ba phút. Thời điểm hiệu lực chưa chốt, chưa có test cho rule mới và chưa có estimate đầy đủ. “Hai ngày” là khoảng thời gian còn lại, không phải bằng chứng cho thấy thay đổi làm kịp.

Người đề xuất cho rằng khách sẽ hài lòng hơn; đây là giả thuyết. Không có số liệu trong hồ sơ chứng minh lợi ích đó hoặc chi phí chính sách mới.

## 2. Viết quyết định cần xin

Quyết định là có đưa CR vào đợt này không, với phạm vi và điều kiện nào. Không gộp thành “có thích chính sách ba phút không”. Một người có thể đồng ý nghiên cứu chính sách nhưng phản đối triển khai khi chưa rõ cách áp dụng.

## 3. So ba phương án

| Phương án | Lợi thế | Rủi ro/điều kiện |
|---|---|---|
| Giữ baseline, hoãn CR | Bảo toàn gói đã được chuẩn bị | Lợi ích CR chưa có; gói cũ vẫn cần go/no-go riêng |
| Lùi đợt để hoàn thiện CR | Có thời gian chốt và kiểm tra | Chi phí chậm cần được chấp nhận |
| Tách pilot giới hạn | Học trên phạm vi nhỏ | Cần policy version, đo lường, hỗ trợ và điều kiện dừng |

Pilot không miễn yêu cầu tính đúng tiền. Phạm vi nhỏ giảm mức ảnh hưởng, không chữa rule mơ hồ.

## 4. Kiểm tra bằng chứng tối thiểu

Trước khi chọn triển khai CR, cần chốt rule hiệu lực, có estimate từ nhóm thực hiện, test biên 119/120/179/180 giây, arriving, ngưỡng trễ, replay và luồng driver_cancel không đổi. CS phải giải thích được chuyến qua mốc; đội đo lường phải phân biệt cohort.

Đây là danh sách cần kiểm chứng, không tuyên bố đã có kết quả pass. Nếu còn thiếu bằng chứng ảnh hưởng tiền, decision brief phải nêu rõ.

## 5. Khuyến nghị theo dữ kiện hiện tại

Với dữ kiện đề bài, khuyến nghị hoãn CR khỏi đợt gần nhất và tiếp tục làm rõ. Lý do là các quyết định trọng yếu và bằng chứng kiểm tra chưa có, không phải vì BA luôn chống thay đổi. Gói baseline cũ có phát hành được không vẫn cần người có trách nhiệm đánh giá riêng.

Nếu sau đó xuất hiện đủ quyết định và bằng chứng, có thể review lại. Ghi tín hiệu mở lại quyết định để việc hoãn không trở thành quên hẳn.

## 6. Kế hoạch tiếp theo có người phụ trách

Bộ phận nghiệp vụ chốt rule hiệu lực và giả thuyết giá trị; bộ phận kỹ thuật phân tích policy/replay; QA thiết kế case và bằng chứng; CS kiểm hướng dẫn; analytics đề xuất cohort. Mốc hoàn thành do người thực hiện xác nhận sau estimate, không ép ngược từ mong muốn hai ngày.

## 7. Bài nộp

Viết một trang quyết định cho sponsor và phụ lục cho nhóm triển khai. Trang đầu phải có đề xuất, bằng chứng thiếu, lựa chọn khác và điều kiện review lại. Phụ lục gồm delta, traceability và dependency register.

<details><summary>Điều gì khiến bài chưa đạt?</summary><p>Viết “go có điều kiện” nhưng không chỉ ra điều kiện phải hoàn thành trước phát hành, ai kiểm và ai có quyền dừng. Điều kiện quan trọng chưa đạt thì chưa phải cho phép phát hành.</p></details>

## 8. Tổng kết phần 6

Từ [backlog](/articles/ba-backlog-refinement-cau-hoi-mo) đến quyết định thay đổi, đầu ra của BA là làm rõ lựa chọn và hệ quả. Phần 7 tiếp tục nghiệm thu, chuyển đổi vận hành và đo giá trị sau phát hành.

Bắt đầu phần 7: [UAT, triển khai và đánh giá](/articles/ba-uat-vai-tro-nguoi-dung-nghiep-vu).
