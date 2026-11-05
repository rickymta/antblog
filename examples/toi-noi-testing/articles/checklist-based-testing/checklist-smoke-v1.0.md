# CL-SMOKE v1.0 — checklist smoke 4 luồng quan trọng (web UI lab Tới Nơi)

Bài TEST-053 «Checklist-based Testing». Chủ checklist: Test Analyst nhóm Booking. Dùng trước mỗi bản phát hành (giả định: `rc-2026.11.2`).

- **Phạm vi**: trang khách `/` và trang tài xế `/driver.html`; 4 luồng: đặt xe, bưu kiện, đồ ăn, huỷ có phí; cộng 4 mục chung về đặc tính chất lượng.
- **Thời lượng**: một phiên ≤ 30 phút, một người.
- **Định dạng mục**: câu hỏi đóng, trả lời Đạt / Không đạt / Không áp dụng; mỗi câu trả lời kèm bằng chứng (chữ trên màn hình, mã chuyến).
- **Kiểu mục** (CTAL-TA 3.4.2): `read-do` = làm đúng thao tác ghi trong mục rồi so với kết quả ghi sẵn; `do-confirm` = tự thao tác, rồi xác nhận một đánh giá cần phán đoán.
- **Ưu tiên**: P1 = không đạt ⇒ dừng phát hành; P2 = không đạt ⇒ ghi defect, trưởng nhóm quyết; P3 = ghi nhận.

**Tiêu chí vào (KHÔNG nằm trong checklist):** máy chủ trả `GET /api/health` 200; dữ liệu mẫu SPEC Phụ lục A đã nạp (`POST /api/test/reset`); đồng hồ 09:00 giờ VN (ngoài cao điểm), không mưa; ví VíXanh giả ở chế độ `none`; bộ test tự động (`npm test`) xanh.

| Mã | Nhóm | Câu hỏi | Kiểu | Ưu tiên | Nguồn |
|---|---|---|---|---|---|
| SM-01 | Đặt xe | Báo giá xe máy 7,4 km TP.HCM, không voucher: tổng «46.000 đ», cao điểm «Không», mưa «Không»? | read-do | P1 | R-FARE-01/08, R-UI-01 |
| SM-02 | Đặt xe | Nhập « giam20 » (chữ thường, có khoảng trắng): giảm «9.060 đ», tổng «37.000 đ»? | read-do | P1 | R-VOUCHER-02/06 |
| SM-03 | Đặt xe | Bấm đúp «Đặt chuyến» (VíXanh): tài xế D101 chỉ thấy đúng 1 chuyến? | read-do | P1 | R-UI-01, R-IDEM-02 |
| SM-04 | Đặt xe | D101 nhận và bấm lần lượt 4 bước: trang khách tự hiện «Hoàn thành» trong ≤ 3 giây, nút huỷ khoá? | read-do | P1 | R-STATE-02, R-NFR-04 |
| SM-05 | Đặt xe | Sau khi chuyến VíXanh hoàn thành, khách có biết đã bị thu bao nhiêu và thu thành công chưa? | do-confirm | P2 | R-PAY-03, R-UI-01 |
| SM-06 | Bưu kiện | Bưu kiện 6,1 kg, xe máy, 7,4 km: tổng «50.000 đ»? | read-do | P1 | R-PARCEL-03 |
| SM-07 | Bưu kiện | 25 kg + xe máy: thông báo nói rõ phải dùng ô tô, không đặt được? | do-confirm | P1 | R-PARCEL-02 |
| SM-08 | Bưu kiện | Sửa khối lượng sau khi báo giá: «Đặt chuyến» khoá, báo giá cũ ẩn? | read-do | P2 | R-UI-01 |
| SM-09 | Đồ ăn | Tiền món 60.000 đ, FOOD15, 7,4 km: giảm «9.000 đ», tổng «97.000 đ»? | read-do | P1 | R-FOOD-01/04/05 |
| SM-10 | Đồ ăn | Chọn «Giao đồ ăn»: ô «Loại xe» bị khoá? | read-do | P2 | R-FOOD-06 |
| SM-11 | Đồ ăn | Tiền món 25.000 đ: thông báo nêu đúng mức tối thiểu 30.000 đ? | do-confirm | P1 | R-FOOD-03 |
| SM-12 | Đồ ăn | FOOD15 cho chuyến đặt xe: bị từ chối với thông báo khách hiểu được? | do-confirm | P2 | R-VOUCHER-05 |
| SM-13 | Huỷ | Huỷ khi «Đang tìm tài xế»: «Phí huỷ: 0 đ»? | read-do | P1 | R-CANCEL-01 |
| SM-14 | Huỷ | D101 nhận (ETA 5), sau 3 phút khách huỷ: «Phí huỷ: 10.000 đ», «Đã huỷ», nút huỷ khoá? | read-do | P1 | R-CANCEL-01/03 |
| SM-15 | Huỷ | Sau khi khách huỷ, D101 nhận được chuyến mới (đã về `online`)? | read-do | P1 | R-ACCEPT-05 |
| SM-16 | Huỷ | Trang tài xế cho tài xế biết chuyến vừa bị khách huỷ? | do-confirm | P2 | R-UI-02 |
| SM-17 | Chung | Mọi thông báo lỗi gặp trong phiên là tiếng Việt, không lộ mã/tên kỹ thuật? | do-confirm | P2 | R-API-02; TEST-014 |
| SM-18 | Chung | Giờ hiển thị cho khách (hạn báo giá) rõ là giờ Việt Nam, kể cả khi máy khách đặt múi giờ khác? | do-confirm | P2 | R-GEN-02; lịch sử defect giờ VN/UTC |
| SM-19 | Chung | Mọi số tiền, quãng đường hiển thị theo kiểu Việt Nam («52.000 đ», «7,4 km»)? | do-confirm | P2 | R-UI-04; quy ước SPEC |
| SM-20 | Chung | Chỉ dùng bàn phím (Tab, Enter) báo giá và đặt được một chuyến? | do-confirm | P3 | R-UI-03 |

**Mục bị loại khi soạn (và lý do — CTFL 4.4.3):**

| Mục ứng viên | Lý do loại | Đi đâu |
|---|---|---|
| «Máy chủ chạy, dữ liệu mẫu đã nạp?» | Là tiêu chí vào | Tiêu chí vào ở trên |
| «axe-core không báo vi phạm WCAG?» | Máy kiểm được hoàn toàn | Test tự động (TEST-091) |
| «Mọi quy tắc R-FARE đúng?» | Quá chung, không kiểm trực tiếp được | Bộ unit test Pricing (Series 06) |
| «≥ 95% ca test đạt?» | Là tiêu chí ra | Kế hoạch test |
| «Ứng dụng dễ dùng?» | Quá chung | Tách thành SM-17…SM-20 |
