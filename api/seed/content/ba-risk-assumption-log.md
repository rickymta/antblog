---
slug: ba-risk-assumption-log
title: "Risk và assumption log: theo dõi điều có thể làm lệch quyết định"
summary: "Tách rủi ro, giả định và vấn đề đã xảy ra; ghi tín hiệu, người phụ trách, phản ứng và thời hạn kiểm chứng thay vì chỉ lập danh sách lo ngại."
category: business-analysis
level: beginner
tags: [business-analysis, ba-059, risk, assumptions, toi-noi]
language: vi
---

“Chắc tài xế sẽ đọc thông tin” là một giả định quan trọng của điểm đón. Nếu không kiểm chứng giả định này, nhóm có thể làm đúng toàn bộ chức năng phía khách mà vẫn không cải thiện việc gặp nhau.

<div class="tkh-callout"><span class="tkh-callout-title">BA-059 · Phần 8/8 · Cơ bản</span><p>Đầu ra: log có hành động cho P07. Đánh giá mức rủi ro là minh hoạ định tính; không phải xác suất được đo từ dữ liệu thật.</p></div>

## 1. Ba loại thông tin

Giả định là điều đang tạm coi là đúng để lập kế hoạch. Rủi ro là sự kiện hoặc điều kiện chưa chắc xảy ra có thể ảnh hưởng mục tiêu. Issue là vấn đề đã xảy ra hoặc đang tồn tại cần xử lý. Một giả định bị bác bỏ có thể trở thành issue và kéo theo thay đổi kế hoạch.

“Chưa có baseline guardrail đáng tin” trong P07 là khoảng trống đã biết, không phải khả năng mơ hồ trong tương lai.

## 2. Viết rủi ro theo nguyên nhân và hậu quả

Thay “rủi ro tài xế” bằng “nếu thông tin xác nhận khó tìm khi tài xế cần, họ có thể tiếp tục dùng pin cũ, làm hành trình không đạt mục tiêu”. Cách viết này giúp chọn hành động kiểm chứng và phát hiện tín hiệu.

Đừng gán lỗi cho một nhóm người trước khi quan sát. P07 có 60 chuyến xác nhận chưa có event đọc; nguyên nhân còn có thể là thiếu telemetry.

## 3. Log mẫu

| ID/loại | Nội dung | Người phụ trách đề xuất | Bước tiếp |
|---|---|---|---|
| A-01 giả định | Tài xế tìm được thông tin khi cần | UX + vận hành | Quan sát các lượt dùng với phiên bản app rõ ràng |
| R-01 rủi ro | Telemetry thiếu làm đọc sai adoption | Analytics | Đối chiếu trip mẫu từ đầu tới dashboard |
| I-01 issue | Guardrail chưa có baseline tin cậy | PO + CS | Chốt định nghĩa và thu baseline |
| R-02 rủi ro | Mở rộng vượt công suất hỗ trợ | Vận hành | Ước lượng và diễn tập trường hợp ngoại lệ |

Thêm hạn, trạng thái và bằng chứng khi áp dụng. Người phụ trách là người theo dõi tới kết luận, không nhất thiết tự làm mọi việc.

## 4. Xác suất và tác động có căn cứ

Có thể dùng thấp/trung bình/cao với định nghĩa chung. Ví dụ tác động cao khi khiến số liệu không dùng được cho quyết định mở rộng. Nếu không có dữ liệu để ước lượng xác suất, ghi chưa biết và ưu tiên kiểm chứng theo mức ảnh hưởng; đừng đặt 73% chỉ vì bảng cần số.

## 5. Chọn phản ứng cụ thể

Giảm rủi ro bằng cách kiểm tra telemetry; tránh rủi ro bằng cách chưa mở rộng khi điều kiện chưa đạt; chấp nhận một rủi ro còn lại trong thẩm quyền với trigger review. Nếu chuyển việc cho đối tác, trách nhiệm hiểu ảnh hưởng tới khách không tự biến mất. Luôn ghi ai làm gì và bằng chứng nào xác nhận hiệu quả.

## 6. Đặt trigger để hành động

“Theo dõi sát” quá mơ hồ. Với dữ liệu thiếu, trigger có thể là đối chiếu mẫu phát hiện event mất ở một phiên bản app: tạm dừng diễn giải adoption của nhóm đó và điều tra. Ngưỡng định lượng cho rollout phải được chốt với nhóm, không phát minh sau khi đã thấy kết quả.

## 7. Thực hành

Thêm giả định “CS xử lý được mọi trường hợp trong pilot bằng tra cứu thủ công”. Chỉ ra bằng chứng cần có, tín hiệu bác bỏ và quyết định phải xem lại nếu giả định sai.

<details><summary>Gợi ý</summary><p>Cần thời gian xử lý mỗi trường hợp, số trường hợp, ca trực và quyền truy cập. Nếu công suất không đủ, xem lại phạm vi pilot, công cụ hoặc lịch; không chỉ yêu cầu CS cố gắng hơn.</p></details>

## 8. Đọc tiếp

Liên hệ [quyết định khi thiếu bằng chứng](/articles/ba-quyet-dinh-khi-thieu-bang-chung). Tiếp: [gap analysis và roadmap](/articles/ba-gap-analysis-roadmap).
