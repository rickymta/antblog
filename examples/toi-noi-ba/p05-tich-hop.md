# P05 · Dữ liệu và tích hợp

Baseline đọc: SPEC lab R-QUOTE, R-PAY, R-IDEM. Hoàn tiền, callback và trạng thái refund dưới đây là **đề xuất học tập**, chưa có trong lab. Không gọi cổng thanh toán thật.

## Hợp đồng đề xuất REF-P05

- Một refundId thuộc một paymentId đã được xác nhận thu thành công; amountVnd nguyên dương, tiền VND. Không dùng fare làm bằng chứng đã thu.
- Tổng đã hoàn + khoản hoàn đang được giữ chỗ không vượt số tiền đã thu; kiểm đồng thời, không chỉ kiểm từng request riêng.
- proposed states: requested → processing → succeeded hoặc rejected; unknown biểu diễn chưa biết kết quả từ bên ngoài, không coi là thất bại chắc chắn. Cần chốt đầy đủ transition trước triển khai.
- Retry cùng ý định dùng cùng key và cùng payload; key khác payload bị từ chối. Phạm vi key, retention, mã lỗi, timeout và retry budget còn phải chốt; không sao chép retention của lab thành hợp đồng refund.
- Callback cần xác thực, liên kết refundId/providerRef, chống trùng event và đối chiếu nội dung. Callback cũ không làm succeeded quay về processing; sự kiện mâu thuẫn đưa vào hàng đợi đối soát có owner.
- Không tạo refund mới chỉ vì timeout. Người xử lý sau phải kiểm kết quả nhà cung cấp trước quyết định tiếp.

## Dữ liệu SQL tổng hợp độc lập

Ba lượt T1/T2/T3 đều đã được nhận trong cùng cohort; T1 huỷ trước đón, T2/T3 không. Ticket S1/S2 cùng T1, S3 thuộc T2. Đúng: 1/3 lượt huỷ; 2/3 lượt có ít nhất một ticket. Không phải 2/4 hay 3/4 do join tạo nhiều dòng.

## Case hợp đồng refund

| ID | Đầu vào/sự kiện | Kỳ vọng đề xuất |
|---|---|---|
| RF01 | Thu 100.000, hoàn 30.000 hợp lệ | Một ý định hoàn; chờ kết quả chuẩn |
| RF02 | Cùng key/payload gửi lại | Không tạo thêm refund |
| RF03 | Cùng key, đổi amount | Từ chối, không đổi ý định cũ |
| RF04 | Timeout rồi callback thành công | Đối chiếu và cập nhật cùng refund, không tạo mới |
| RF05 | Callback thành công lặp | Không hoàn/ghi nhận hai lần |
| RF06 | Hai yêu cầu 60.000 và 50.000 trên 100.000 | Không cho tổng cam kết vượt 100.000 |
| RF07 | Callback thiếu/sai xác thực | Không cập nhật theo payload chưa tin cậy |
| RF08 | Succeeded rồi nhận processing cũ | Không lùi trạng thái; lưu dấu vết phù hợp |

Nộp glossary, dictionary, mapping, state/decision table, các case, câu hỏi mở và quyền chốt. Đây là hồ sơ thiết kế để review, không phải tích hợp đã chạy.
