# P07 · Nghiệm thu và đánh giá giá trị

Hồ sơ giả lập riêng cho học tập. P04 chưa triển khai trong lab. Bài thực hành BA-056 giả sử có bản triển khai đã được nghiệm thu; không phải báo cáo UAT hay pilot thật. P06 hoãn CR phí không ngăn bài tập riêng về điểm đón; hai phạm vi khác nhau.

## UAT charter và traceability

Phạm vi: chủ chuyến xác nhận điểm trước accepted, đúng revision; tài xế được phân công đọc được; xử lý stale, sai actor, đổi state và timeout. Ngoài phạm vi: chat, chỉnh sau accepted, chứng minh giảm huỷ.

| Case | Nối P04 | Expected |
|---|---|---|
| UAT-P07-01 | US-P04-01, AC-P04-01 | Xác nhận đúng revision, tồn tại sau reload |
| UAT-P07-02 | RULE-P04-01, AC-P04-04 | Sai actor không đổi dữ liệu |
| UAT-P07-03 | RULE-P04-02, AC-P04-02 | Stale yêu cầu đọc lại |
| UAT-P07-04 | US-P04-02 | Tài xế được phân công thấy thông tin phù hợp |
| UAT-P07-05 | AC-P04-05 | Timeout chưa kết luận thất bại, tra cứu theo thiết kế |
| UAT-P07-06 | RULE-P04-01, AC-P04-03 | accepted ghi trước thì từ chối xác nhận |

Mẫu execution cần build, actor, dữ liệu, thời điểm, actual và evidence. Bảng này chưa có kết quả thực thi. BA-052 dùng tình huống lỗi quyền riêng để học no-go; BA-056 giả định lỗi chặn đã được giải quyết và nghiệm thu trong phạm vi bài tập.

## Dataset tuần B

Cohort: 1.000 trip đủ điều kiện, accepted lần đầu trong tuần B theo Asia/Ho_Chi_Minh. Snapshot cuối tuần + 7 ngày, giả định mọi trip có outcome đầy đủ. Nối event trước accepted theo tripId; không bao gồm booking chưa từng accepted.

- 720 event mở thuộc 600 trip duy nhất; exposure 60% cohort.
- 240 trip xác nhận, nằm trong 600 trip đã thấy: 40% exposed, 24% cohort.
- 180 trip có tài xế được phân công đọc sau xác nhận, nằm trong 240: 75% confirmed.
- 118 trip huỷ trước pickup: 18 trong nhóm 240 confirmed, 100 trong nhóm 760 còn lại.
- Tỷ lệ huỷ toàn cohort 11,8%; confirmed 7,5%; còn lại 13,1579%. Không phân nhóm ngẫu nhiên, không suy nhân quả.
- So tuần A P01: 120/1.000 = 12%, chênh -0,2 điểm phần trăm. Không có baseline guardrail khiếu nại đáng tin trong hồ sơ; không tự suy từ 80 cuộc gọi.

## Sản phẩm cuối phần

UAT charter/scenario; triage log; decision record; transition checklist; metric dictionary; experiment brief; evaluation brief. Bằng chứng cần phân biệt dữ kiện giả lập, giả thuyết, đề xuất và quyết định. Kế hoạch thử nghiệm phải chốt đơn vị phân nhóm, eligibility trước exposure, metric, guardrail, cỡ mẫu và cách xử lý ảnh hưởng chéo.
