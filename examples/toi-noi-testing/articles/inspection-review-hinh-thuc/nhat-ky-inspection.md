# Hồ sơ inspection INS-2026-07 — Đặc tả phí huỷ, bản nháp v0.3 (LỜI GIẢI)

> Lời giải mẫu của bài thực hành TEST-026. Hãy tự làm inspection trên `dac-ta-phi-huy-ban-nhap-v0.3.md`
> với `checklist-inspection.md` trước khi đọc tệp này. Mọi tên người, thời gian, số đo là giả định.

## 1. Planning

| Mục | Nội dung |
|---|---|
| Work product | `dac-ta-phi-huy-ban-nhap-v0.3.md` — 7 mục, ≈ 1 trang |
| Mục đích | Tìm tối đa anomaly trước khi bản nháp thay mục R-CANCEL của SPEC |
| Đặc tính chất lượng cần đánh giá | Đầy đủ, nhất quán (nội bộ và với SPEC), không mơ hồ, kiểm chứng được |
| Vùng tập trung | Mốc thời gian, nguồn dữ liệu tính tiền, thu phí qua ví |
| Tài liệu hỗ trợ | SPEC v1.1 (R-GEN, R-STATE, R-PAY, R-IDEM), checklist CL-SPEC v1.0 |
| Kỹ thuật review | Checklist-based + perspective-based (mỗi inspector một góc nhìn) |
| Nỗ lực dự kiến | ≤ 60 phút chuẩn bị mỗi inspector, họp 45 phút |

**Vai trò**

| Vai trò (CTFL 3.2.3) | Người | Ghi chú |
|---|---|---|
| Manager | Trưởng nhóm kỹ thuật Booking | Quyết định inspect, dành thời gian cho nhóm |
| Review leader + moderator | QA Lead | Một người giữ hai vai |
| Author | BA nhóm Booking | Không được làm leader hay scribe |
| Scribe | Tester 2 | Không review chính |
| Reviewer I1 | Tester 1 | Góc nhìn kiểm thử: lập bảng quyết định từ đặc tả |
| Reviewer I2 | Developer Booking | Góc nhìn cài đặt: viết được hàm tính phí không? |
| Reviewer I3 | Operator vận hành | Góc nhìn vận hành/CSKH: giải thích được phí cho khách không? |

**Tiêu chí vào (entry)** — QA Lead kiểm trước khi khởi động

| Mã | Tiêu chí | Kết quả |
|---|---|---|
| V1 | Có số phiên bản, tác giả, danh sách tài liệu liên quan | Đạt |
| V2 | Tác giả đã tự soát chính tả, đánh số mục | Đạt |
| V3 | ≤ 2 trang (một phiên chuẩn bị ≤ 60 phút) | Đạt (1 trang) |
| V4 | SPEC v1.1 và checklist đã gửi cho mọi người | Đạt |
| V5 | ≥ 3 reviewer với 3 góc nhìn khác nhau đã nhận lời | Đạt |

**Tiêu chí ra (exit)**

| Mã | Tiêu chí |
|---|---|
| E1 | Mọi anomaly có trạng thái (defect / câu hỏi / đề xuất) và người chịu trách nhiệm |
| E2 | 0 defect critical/major còn mở; review leader kiểm từng sửa đổi |
| E3 | Mật độ critical + major > 5 / trang ⇒ bản sửa phải qua thêm một vòng review (leader + 1 reviewer) |

## 2. Review initiation (15 phút, 6 người)

QA Lead nhắc mục đích, vai trò, góc nhìn của từng reviewer, cách ghi anomaly (mã mục, vị trí, mô tả, mức độ
đề xuất), hạn nộp nhật ký cá nhân trước buổi họp một ngày.

## 3. Individual review — nhật ký cá nhân

| Reviewer | Thời gian chuẩn bị | Anomaly ghi | Trong đó |
|---|---|---|---|
| I1 Tester | 40 phút | 8 | 8 nghi defect |
| I2 Developer | 35 phút | 6 | 6 nghi defect |
| I3 Operator | 30 phút | 6 | 4 nghi defect, 1 câu hỏi, 1 đề xuất |
| **Cộng** | **105 phút** | **20** | |

Bảng quyết định I1 dựng từ bản nháp (ô «?» = đặc tả không trả lời được):

| Trạng thái | t = thời gian từ lúc nhận | Tài xế trễ? | Phí | Lý do |
|---|---|---|---|---|
| `searching` | — | — | 0 đ | `searching` |
| `accepted` | t < 2 phút | — | 0 đ | `grace_period` hay `grace`? (D-09) |
| `accepted` | t = 2:00 đúng | — | ? (D-01) | ? |
| `accepted` | t > 2 phút | «đáng kể»? (D-07) | phí / 0 đ? | ? |
| `arriving` | bất kỳ | «đáng kể»? (D-07) | phí / 0 đ? | ? |
| `picked_up` | bất kỳ | — | gấp đôi — nhưng R-STATE cấm khách huỷ (D-02) | ? |
| `requested`, `in_transit`, `completed`, `cancelled` | — | — | ? (D-03) | ? |
| hệ thống huỷ (`no_driver`) | — | — | ? (D-08) | ? |

## 4. Communication and analysis — họp 45 phút, 6 người

Gộp 20 anomaly thô ⇒ **14 mục duy nhất** (6 trùng). Kết luận từng mục:

| Mã | Mục | Mô tả | Loại | Mức | Checklist | Tìm bởi | Quyết định |
|---|---|---|---|---|---|---|---|
| D-01 | N-01 | «trong vòng 2 phút» / «sau 2 phút»: đúng 2:00 thuộc dòng nào? | Mơ hồ | Major | CL-1 | I1, I2 | Defect — ghi rõ `now − acceptedAt < 2 phút` |
| D-02 | N-01 | Dòng `picked_up` mâu thuẫn R-STATE-02/03: khách không được huỷ sau khi đã đón | Mâu thuẫn | Major | CL-4 | I1, I2 | Defect — bỏ dòng |
| D-03 | N-01 | Thiếu kết quả cho `requested`, `in_transit`, `completed`, `cancelled` | Thiếu | Major | CL-2 | I1 | Defect — thêm «trạng thái khác: không được huỷ, `INVALID_TRANSITION`» |
| D-04 | N-02 | Tính giờ theo đồng hồ điện thoại tài xế: trái R-GEN-02 và là dữ liệu client tự khai (chỉnh giờ để thu phí) | Mâu thuẫn | Critical | CL-4, CL-8 | I1, I2 | Defect — dùng `acceptedAt` do hệ thống ghi |
| D-05 | N-02 | «lúc tài xế bấm Nhận» khác thời điểm hệ thống ghi nhận (`acceptedAt`, R-STATE-07) | Thuật ngữ | Minor | CL-3 | I2 | Defect |
| D-06 | N-03 | «bảng phí theo thành phố» không tồn tại | Thiếu | Major | CL-6 | I3 | Defect — PO xác nhận phí giống nhau mọi thành phố, bỏ câu |
| D-07 | N-04 | «trễ đáng kể» không kiểm chứng được; trễ so với mốc nào? | Mơ hồ | Major | CL-5 | I1, I3 | Defect — `now > acceptedAt + etaMinutes + 10 phút` |
| D-08 | N-05 | Không nói phí khi hệ thống huỷ vì hết thời gian tìm (R-STATE-05) | Thiếu | Major | CL-2 | I1 | Defect |
| D-09 | N-06 | Mã `grace`, `late` khác `grace_period` ở N-01 | Thuật ngữ | Minor | CL-3 | I1 | Defect — thống nhất `grace_period`, `driver_late` |
| D-10 | N-07 | Chuyến tiền mặt không có ví để trừ (R-PAY-01, R-PAY-05) | Mâu thuẫn | Major | CL-4 | I3 | Defect |
| D-11 | N-07 | «Mỗi yêu cầu huỷ tạo một khoản phí»: gửi lại ⇒ thu hai lần, trái R-IDEM-02 | Mâu thuẫn | Critical | CL-4, CL-7 | I1, I2 | Defect — thu theo khoá `trip:<id>:cancel-fee` |
| D-12 | N-07 | Thu phí thất bại thì huỷ có bị chặn không? | Thiếu | Major | CL-7 | I2, I3 | Defect — theo R-PAY-06: không chặn, `paymentStatus = failed` |
| Q-01 | N-03 | Đơn đồ ăn tính phí xe máy hay ô tô? | Câu hỏi | — | — | I3 | Đã giải đáp: R-GEN-04, đồ ăn luôn xe máy ⇒ 10.000 đ. Không phải defect |
| R-01 | — | Hiện phí dự kiến trước khi khách bấm «Huỷ» | Đề xuất | — | — | I3 | Chuyển PO (backlog giao diện), ngoài phạm vi đặc tả này |

Kết luận chất lượng: **chưa đạt** — 10 defect critical/major trên 1 trang ⇒ E3 kích hoạt.

## 5. Fixing and reporting

- Author sửa thành v0.4 (60 phút). Review leader + I1 kiểm lại từng sửa đổi (2 × 20 phút) — đạt E2.
- 12 defect report được đóng sau khi kiểm; Q-01 đóng; R-01 chuyển backlog.

## 6. Số đo (metric)

| Số đo | Giá trị | Cách tính |
|---|---|---|
| Kích thước | 1 trang (7 mục) | |
| Thời gian chuẩn bị | 105 phút-người | 40 + 35 + 30 |
| Tốc độ chuẩn bị trung bình | ≈ 1,7 trang/giờ | 1 trang ÷ 35 phút |
| Anomaly thô / duy nhất / trùng | 20 / 14 / 6 (30%) | |
| Defect | 12 — 2 critical, 8 major, 2 minor | |
| Mật độ defect | 12 defect/trang (10 critical + major/trang) | 12 ÷ 1 |
| Defect theo loại | Mâu thuẫn 4 · Thiếu 4 · Mơ hồ 2 · Thuật ngữ 2 | |
| Hiệu suất chuẩn bị | ≈ 6,9 defect/giờ chuẩn bị | 12 ÷ (105/60) |
| Tốc độ ghi trong họp | ≈ 3,2 phút/mục | 45 ÷ 14 |
| Defect chỉ một góc nhìn thấy | 6/12 — kiểm thử 3, cài đặt 1, vận hành 2 | |
| Tổng nỗ lực | 620 phút-người ≈ 10,3 giờ-người | 40 + 90 + 105 + 270 + 60 + 40 + 15 |
| Nỗ lực mỗi defect | ≈ 52 phút-người | 620 ÷ 12 |

Chi tiết nỗ lực: planning 40 (QA Lead 30 + Manager 10) · initiation 90 (15 × 6) · individual review 105 ·
họp 270 (45 × 6) · sửa 60 · kiểm lại 40 (2 × 20) · báo cáo 15.

## 7. Đề xuất cải tiến quy trình (dùng metric — không đánh giá cá nhân)

1. 4/12 defect là mâu thuẫn với mục khác của SPEC ⇒ gói khởi động lần sau đính kèm sẵn đoạn SPEC liên quan,
   thêm vào checklist câu «Đã đọc chéo từng mục tham chiếu chưa?» cho từng mã R-… cụ thể.
2. Góc nhìn vận hành tìm 2 defect không ai khác thấy (D-06, D-10) ⇒ đặc tả có tiền luôn mời Operator.
3. Mật độ 10 critical/major mỗi trang ở bản nháp ⇒ đề xuất BA tự soát bằng CL-SPEC trước khi nộp (thêm vào V2).
