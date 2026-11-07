# TEST-052 · Hai test charter cho trang tài xế `/driver.html`

Định dạng mission theo CTAL-TA v4.0 mục 3.4.1: «Explore [target] With [resources] To discover [information]».

## CH-01 — Danh sách chuyến chờ và nhận chuyến

| Mục | Nội dung |
|---|---|
| Mission | **Khám phá** danh sách chuyến đang chờ và thao tác nhận chuyến trên `/driver.html` **với** nhiều mã tài xế (D101, D102 xe máy; D103 ô tô; D104 offline; mã sai), ETA biên (0, trống, 1, 60, 61), thao tác lặp (bấm đúp, tải lại trang) và danh sách để lâu **để tìm** chỗ trang hiển thị sai/thiếu, mất trạng thái hoặc thông báo không giúp tài xế hành động |
| Tổ chức | Time box 45 phút · tester: tác giả bài · ngày 05/11/2026 |
| Phạm vi | R-ACCEPT-01…07, R-STATE-02/04/05, R-UI-02/03/04. Ngoài phạm vi: trang khách, thanh toán (đã có TEST-051), hiệu năng |
| Điều kiện vào | Máy chủ lab cổng 3152, `NODE_ENV=test`, đồng hồ 09:00 giờ VN 04/11/2026, giới hạn tần suất tắt; chạy `node articles/exploratory-testing/seed-session.mjs` (3 chuyến xe máy + 1 ô tô ở SG) |
| Thông tin sản phẩm | Điều phối kiểu kéo (R-ACCEPT-06), lab không gửi offer; hệ thống tự huỷ chuyến tìm quá 5 phút, quét mỗi 15 giây |
| Lịch sử | TEST-014: thông báo `INVALID_TRANSITION` lộ mã nội bộ; TEST-051: thao tác lặp và race ở phía API |
| Giới hạn (không bao giờ được) | Hai tài xế cùng giữ một chuyến; một tài xế giữ hai chuyến (R-ACCEPT-03) |
| Công cụ | Chrome + DevTools (Console, Network), `curl` xem chuyến qua API |

## CH-02 — Thực hiện chuyến và huỷ nhận khi phiên trình duyệt không ổn định

| Mục | Nội dung |
|---|---|
| Mission | **Khám phá** các bước arrive → pickup → start → complete và «Huỷ nhận» **với** hai tab cùng một tài xế, mạng chậm (DevTools «Slow 3G»), tải lại trang giữa các bước và khách huỷ song song ở trang `/` **để tìm** trạng thái hiển thị lệch với trạng thái thật của chuyến và thao tác không còn đường quay lại |
| Tổ chức | Time box 45 phút · chưa chạy (phiên sau) |
| Phạm vi | R-STATE-02…04, R-CANCEL-05, R-ACCEPT-05/07, R-UI-02, R-NFR-04 |
| Điều kiện vào | Như CH-01 + mở thêm trang khách `/` ở cửa sổ ẩn danh |
| Lịch sử | Phát hiện S1-01 của phiên CH-01 (tải lại trang làm mất chuyến) — CH-02 đi sâu vào nhóm lỗi này |
