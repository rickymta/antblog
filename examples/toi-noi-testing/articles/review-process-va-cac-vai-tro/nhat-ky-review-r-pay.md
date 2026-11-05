# Nhật ký phát hiện — RV-PAY-01

## Mẫu trống (individual review)

| ID | Vị trí (mã R-… / đoạn) | Loại (defect nghi ngờ / câu hỏi / đề xuất) | Mô tả | Người nêu | Quyết định (điền trong họp) | Mức | Phụ trách |
|---|---|---|---|---|---|---|---|
| | | | | | | | |

Quy ước quyết định: **Defect** (sửa đặc tả; Major ⇒ có defect report) · **Câu hỏi ⇒ việc mới** (giao người + hạn) · **Bác bỏ** (không phải defect, ghi lý do) · **Hoãn** (ngoài phạm vi lần này, ghi nơi theo dõi).

Mức: Major = có thể dẫn tới sai tiền hoặc sai trạng thái nghiệp vụ; Minor = câu chữ hoặc thiếu sót không đổi hành vi đã cài đặt.

## Nhật ký sau họp (Thứ Ba 15:45 — kết quả mô phỏng)

| ID | Vị trí | Loại | Mô tả | Người nêu | Quyết định | Mức | Phụ trách |
|---|---|---|---|---|---|---|---|
| A-01 | R-PAY-03 ↔ R-API-07 | Defect nghi ngờ | «Thu đúng số tiền `fare`», nhưng R-API-07 định nghĩa `quote.fare` là bảng chi tiết; với `food`, `fare.total` chỉ là phí giao ⇒ có thể chỉ thu phí giao (kịch bản S3: 18.000 đ thay vì 114.000 đ) | Dev Booking | Defect: ghi «số tiền khách phải trả = `quote.total`, lưu ở trường `fare` của chuyến (R-QUOTE-04)» + định nghĩa trường chuyến | Major | BA |
| A-02 | R-PAY-02 ↔ R-PAY-05 | Defect nghi ngờ | `cash` vừa nghĩa «thu tiền mặt, không gọi ví» vừa dùng cho phí huỷ tiền mặt chưa thu | Đối soát | Defect câu chữ: `cash` = «xử lý ngoài hệ thống, không có nghĩa đã thu»; trạng thái nợ ⇒ backlog hệ đích | Minor | BA |
| A-03 | R-PAY-04 | Câu hỏi | «Lỗi tạm thời (cổng không trả lời được)» trùng nghĩa hết giờ; lỗi rõ ràng (vd 503) có thử lại? R-TEST-03 coi `error` là lỗi tạm thời | Dev Payment | Defect: liệt kê lỗi tạm thời = hết giờ, không kết nối được, lỗi tạm thời do cổng trả về | Minor | BA |
| A-04 | R-PAY-04 | Câu hỏi | Không nêu thời gian chờ giữa các lần thử; tệ nhất `/complete` chờ 3 × 3 = 9 giây | SRE | Câu hỏi ⇒ việc mới: nêu thời gian xử lý tối đa của `/complete` và lệnh huỷ, trước sprint sau | — | PO, SRE |
| A-05 | R-PAY-03 | Defect nghi ngờ | Không nói gì khi số tiền bằng 0 | Dev Payment | Bác bỏ: R-FARE-07 (cước tối thiểu) và R-FOOD-03/04 bảo đảm số tiền > 0 | — | — |
| A-06 | R-PAY-05 ↔ R-STATE-05 | Câu hỏi | «Phí 0 ⇒ `none`» nằm trong đoạn khách huỷ; chuyến `vixanh` bị hệ thống huỷ (`no_driver`) thì `paymentStatus` là gì? | Dev Booking | Defect (thiếu): mọi lần huỷ có phí 0 ⇒ `none` | Minor | BA |
| A-07 | R-PAY-06 | Câu hỏi | `failed` «cần xử lý sau» — ai, bằng cách nào? | Đối soát | Hoãn: ngoài phạm vi lab (R-ARCH-02); backlog Payment & Wallet Service (đối soát định kỳ, kiến trúc §8.3) | — | EM Payment |

Kết luận chất lượng trong họp: **chấp nhận sau khi sửa** (có 1 Major ⇒ cần follow-up).

## Báo cáo review (Thứ Năm — kết quả mô phỏng)

```text
REVIEW R-PAY · SPEC v1.1 mục 11
Người tham gia : 7 (4 reviewer) · nỗ lực thực tế ≈ 9,5 giờ công
Anomaly        : 7 → 4 defect (1 Major, 3 Minor) · 1 việc mới · 1 bác bỏ · 1 hoãn
Defect report  : 1 (A-01, Major) — đã sửa trong bản mới, follow-up đạt
Exit criteria  : (1) đạt  (2) đạt — 0 Major mở  (3) đạt — A-04 có hạn chót  (4) đạt
Kết luận       : chấp nhận đặc tả bản sửa; mở lại R-PAY khi thêm webhook/đối soát
Phản hồi       : gửi BA và squad Booking; đề xuất thêm mục «trường dữ liệu của chuyến» vào SPEC
```
