# Nhật ký review mẫu — Đặc tả cước bản nháp v0.3 (lời giải TEST-021)

Đối tượng review: `ban-nhap-dac-ta-cuoc-v0.3.md`. Nguồn đối chiếu: `SPEC.md` v1.1 (R-FARE-01…09, R-PARCEL-03/04, R-GEN-01/02, R-NFR-03).
Cách làm: đọc tĩnh, không chạy mã. Số tiền trong cột «Hậu quả nếu lọt» tính tay theo công thức của SPEC.

| # | Dòng | Loại | Phát hiện | Đối chiếu SPEC | Hậu quả nếu lọt (ví dụ) |
|---|---|---|---|---|---|
| F1 | D-02 | Mơ hồ | Không nói kiểu làm tròn 0,1 km (half-up? half-even? cắt bỏ?) | R-FARE-02: half-up (0,45 ⇒ 0,5) | 0,45 km bị cắt thành 0,4 ⇒ báo lỗi thay vì nhận chuyến |
| F2 | D-02, D-03 | Thiếu | Không nói làm tròn trước hay sau khi kiểm khoảng | R-FARE-02: làm tròn **trước** mọi bước | 100,04 km: hợp lệ hay lỗi tuỳ người code |
| F3 | D-03 ↔ D-13 | Mâu thuẫn | «dưới 100 km» vs «tối đa 100 km» | R-FARE-02: [0,5; 100], tính cả hai đầu | Chuyến đúng 100,0 km bị từ chối |
| F4 | D-04 | Mơ hồ + thiếu | Biên giờ (13:00 có là cao điểm?) và múi giờ (giờ máy chủ hay giờ VN?) không rõ | R-FARE-04: [11:00; 13:00) và [17:00; 19:00); R-GEN-02: giờ VN UTC+7 | Máy chủ chạy UTC ⇒ cao điểm lệch 7 giờ |
| F5 | D-04, D-05 | Thiếu | Không có thứ tự cao điểm – mưa: phụ phí mưa có bị nhân 1,25? | R-FARE-03 bước 2→3; R-FARE-05: mưa cộng sau, không bị nhân | Ví dụ R-FARE-09: 54.000 đ thay vì 52.000 đ (+2.000 đ) |
| F6 | D-06 | Mơ hồ | «phần trăm của cước» — cước ở bước nào? | R-FARE-06: trên số tiền sau bước 3 (đã gồm cao điểm, mưa) | Giảm trên cước cơ bản ⇒ khách được giảm ít hơn |
| F7 | D-07 | Sai | Áp cước tối thiểu **trước** voucher cho phép voucher kéo cước xuống dưới tối thiểu | R-FARE-07: tối thiểu áp **sau** voucher | Xe máy 0,5 km, GIAM20, ngoài cao điểm: 12.000 đ thay vì 15.000 đ |
| F8 | D-08 | Mơ hồ + thiếu | Không nói làm tròn lên/xuống/gần nhất, và làm ở bước nào | R-GEN-01: không làm tròn giữa chừng; R-FARE-08: chỉ bước cuối, làm tròn **lên** | Làm tròn gần nhất ở mọi bước: ví dụ R-FARE-09 ra 51.000 đ |
| F9 | D-09 | Mơ hồ + thiếu | «mỗi kg vượt»: kg chưa trọn tính sao? Phụ phí đứng ở bước nào? | R-PARCEL-03: mỗi kg **bắt đầu** (5,1 kg ⇒ 2.000 đ); R-PARCEL-04: cộng vào cước cơ bản nên bị nhân cao điểm | 5,1 kg tính 0 đ |
| F10 | D-01 ↔ D-10 | Trùng lặp + mâu thuẫn | Cước tối thiểu ô tô ghi hai nơi, hai giá trị (30.000 / 35.000) | R-FARE-01: 30.000 đ | Hai developer chọn hai số khác nhau |
| F11 | D-11 | Không kiểm thử được | «nhanh» không có số đo, không có phân vị, không có cửa sổ đo | R-NFR-03: p95 `POST /api/quotes` dưới 500 ms | Không viết được tiêu chí đạt/không đạt |
| F12 | D-12 | Sai (ví dụ) | 64.625 − 20% = 51.700, không phải 51.900 | R-FARE-09 | Ví dụ sai làm người đọc nghi ngờ công thức đúng; không test nào chạy được một ví dụ trong tài liệu |

Ghi chú:
- Kết quả 52.000 đ ở D-12 vẫn đúng vì làm tròn lên che mất lỗi số học — một lý do nữa để kiểm từng bước trung gian.
- Một số phát hiện ban đầu là **câu hỏi** (anomaly), chỉ thành defect sau khi đối chiếu nguồn cấp trên. Ví dụ «voucher có điều kiện đơn tối thiểu không?» — D-06 không nhắc, nhưng R-VOUCHER-06 nằm ở đặc tả voucher riêng ⇒ ghi «câu hỏi, đã đóng», không tính là defect của bản này.
