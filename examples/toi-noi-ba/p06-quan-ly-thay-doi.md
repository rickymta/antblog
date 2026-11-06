# P06 · Quản lý backlog và thay đổi

Hồ sơ học tập, không phải approval hoặc kế hoạch phát hành thật. Kế thừa P03 CR-BA-01 và P04 điểm đón; không sửa SPEC lab.

## Tình huống cuối phần

Hai ngày trước mốc phát hành giả lập, stakeholder muốn đổi grace ở accepted từ elapsed < 120 giây thành < 180 giây. Gói hiện tại chỉ đã kiểm rule cũ trong giả định đề bài; không có báo cáo test thật. Rule chuyến qua mốc chưa chốt, CS vẫn dùng hướng dẫn hai phút, estimate đầy đủ chưa có. Lợi ích hài lòng hơn là giả thuyết.

Giữ arriving, ngưỡng driver_late, mức phí, driver_cancel và replay theo baseline. Không suy CR giảm tỷ lệ huỷ từ target P01.

## Artifact cần nộp

| Artifact | Điều phải thể hiện |
|---|---|
| Backlog | BL-01 điểm đón, BL-02 chat, BL-03 CR; mục tiêu và câu hỏi mở |
| MoSCoW | Phạm vi/timebox và lý do; ai quyết định |
| Traceability | Nhu cầu → rule/story → case, trạng thái chưa chạy |
| Baseline | BL-P06-01 tham chiếu rule cũ; CR vẫn chưa duyệt |
| Impact map | Rule, dữ liệu, API, UI, CS, đo lường |
| Dependency register | Provider, consumer, artifact, hạn cần, tiêu chí bàn giao |
| Decision brief | Hoãn CR, lùi đợt hoặc pilot; điều kiện và rủi ro |

## Case phải review

accepted không thuộc miễn do trễ tại 119/120/179/180 giây; arriving dưới 180 giây; đúng và vượt ngưỡng trễ; tài xế rời rồi nhận lại; chuyến qua mốc hiệu lực; replay cùng key/payload đã có kết quả. Việc chọn policyVersion cho CR còn mở, không tuyên bố field đã tồn tại.

## Bảng điểm minh họa độc lập BA-046

S = reach × impact × confidence / effort. A = 1.000 × 1 × 0,6 / 2 = 300. B = 1.000 × 2 × 0,3 / 5 = 120. A hòa B tại confidence 0,24 khi effort 2, hoặc effort 5 khi confidence 0,6. Không coi điểm là tiền/lợi ích thật.

## Kết luận mẫu

Theo dữ kiện đề bài, hoãn CR khỏi đợt gần nhất và tiếp tục phân tích. Phát hành baseline cũ là quyết định riêng, chưa được bài tập tự phê duyệt. Mở lại review khi có rule hiệu lực, estimate, bằng chứng kiểm tra, readiness CS và đo lường. Người có thẩm quyền thực tế phải được xác nhận trước approval.
