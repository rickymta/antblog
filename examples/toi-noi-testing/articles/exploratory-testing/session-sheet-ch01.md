# Session sheet — CH-01 · Danh sách chuyến chờ và nhận chuyến (`/driver.html`)

| Mục | Ghi nhận |
|---|---|
| Charter | CH-01 (`charters.md`) |
| Tester | tác giả bài TEST-052 |
| Ngày, giờ (giờ máy) | 05/11/2026 · 23:34:45 → 23:40:27 (đồng hồ của lab bắt đầu ở 09:00 giờ VN 04/11/2026) |
| Time box | 45 phút — phiên kết thúc sớm khi hết ý tưởng trong charter. Trình duyệt được điều khiển bằng công cụ tự động hoá theo từng thao tác do tester quyết định (không có kịch bản viết trước), nên nhanh hơn tester thao tác tay |
| Môi trường | Máy chủ lab cổng 3152, kho trong bộ nhớ, `NODE_ENV=test`, giới hạn tần suất tắt; Chrome, khung nhìn hẹp 436 px |
| Dữ liệu | `seed-session.mjs` lúc 23:34:42: `2762e89d…` C001 xe máy 7,4 km 46.000 đ tiền mặt · `e818d086…` C002 xe máy 2 km 21.000 đ VíXanh · `35ffbd83…` C003 đồ ăn 3 km 116.000 đ · `c062ce24…` C001 ô tô 10 km 135.000 đ |

## Nhật ký (theo thứ tự)

| # | Giờ | Thao tác | Quan sát | Ghi |
|---|---|---|---|---|
| 1 | 23:35 | D101 «Tải chuyến đang chờ» | 3 dòng xe máy, đúng thứ tự `requestedAt` (R-ACCEPT-06). Hiện mã `ride`/`food`, «7.4 km» dấu chấm, đơn đồ ăn hiện **116.000 đ** (gồm 90.000 đ tiền món) | Q-1 |
| 2 | 23:35 | ETA = 0, nhận chuyến 1 | «Dữ liệu gửi lên không hợp lệ.» — không nói ô nào sai | S1-04 |
| 3 | 23:35 | ETA trống, nhận | Như #2 (kiểm lại bằng ES-05) | S1-04 |
| 4 | 23:35 | ETA 45, nhận chuyến 1 | Nhận được, «Tài xế đã nhận», danh sách tự tải lại còn 2 dòng | đạt |
| 5 | 23:36 | D101 đang bận bấm nhận chuyến 2 | «Tài xế đang có chuyến khác.» | đạt |
| 6 | 23:36 | Tải lại trang (F5), tải danh sách | Khối «Chuyến đang chạy» biến mất; chuyến 1 không có ở danh sách; không ô nào nhập được mã chuyến ⇒ không hoàn thành được chuyến đang giữ | **S1-01** |
| 7 | 23:36 | D102 nhận chuyến 2, bấm «Hoàn thành» ngay | `Không thể thực hiện "complete" khi chuyến đang ở trạng thái "accepted".` | đã biết (TEST-014) |
| 8 | 23:36 | Đổi ô mã sang D101, bấm «Đang tới đón» | «Không tìm thấy chuyến.» cho chính chuyến đang hiện | S1-03 |
| 9 | 23:36 | D102: «Đang tới đón», rồi «Huỷ nhận» | Không hỏi xác nhận, không hỏi lý do; danh sách không làm mới. Tải lại: chuyến vừa huỷ đứng ĐẦU và D102 nhận lại được ngay | Q-2 |
| 10 | 23:36 | D104 (offline) tải danh sách, bấm nhận | Thấy đủ danh sách; bấm mới báo «Tài xế đang ngoại tuyến.»; trang không có cách chuyển online | Q-3 |
| 11 | 23:36 | Mã «d101␣» | «Thiếu hoặc sai header x-user.» — câu kỹ thuật | S1-06 |
| 12 | 23:37 | D102 tải danh sách, **bấm đúp** «Nhận chuyến» chuyến 2 | Nhận thành công (lệnh thứ hai idempotent, R-ACCEPT-07) nhưng danh sách hiện chuyến đồ ăn **hai lần**, cùng `data-testid` | S1-05 |
| 13 | 23:38 | Đi hết arrive → pickup → start → complete, bấm «Hoàn thành» lần nữa | «Hoàn thành»; khối vẫn hiện đủ 5 nút; lần bấm thêm báo mã nội bộ (đã biết). Trang không nói chuyến trả tiền mặt hay VíXanh (chuyến 2 là VíXanh, API ghi `paid`) | Q-4 |
| 14 | 23:39 | Đổi sang D103 (ô tô), bấm nhận chuyến xe máy còn trong danh sách cũ | «Chuyến cần loại xe khác.»; khối «Chuyến đang chạy» vẫn hiện chuyến của D102 dưới mã D103 | ghi chú cho CH-02 |
| 15 | 23:39 | D103 tải danh sách, ETA 61 rồi 60, nhận chuyến ô tô 10 km | 61 ⇒ thông báo chung (ô báo `validity.valid = false` nhưng trang vẫn gửi); 60 ⇒ nhận được — khách chỉ được huỷ miễn phí do trễ sau 70 phút (R-CANCEL-04) | S1-04, Q-5 |
| 16 | 23:39:47 | (hệ thống) chuyến đồ ăn bị huỷ `no_driver` sau 5 phút | Danh sách của D102 tải lúc 23:39:2x vẫn còn dòng đó | — |
| 17 | 23:40 | D102 bấm «Nhận chuyến» dòng cũ | `Không thể thực hiện "accept" khi chuyến đang ở trạng thái "cancelled".`; dòng cũ vẫn ở danh sách | **S1-02** |

## Tổng kết cho debrief

- **Lỗi (bug):** S1-01 (cao) · S1-02 (trung bình) · S1-03, S1-04, S1-05, S1-06 (thấp).
- **Câu hỏi (issue) cho PO:** Q-1 số tiền hiện cho tài xế ở đơn đồ ăn · Q-2 tài xế huỷ rồi nhận lại ngay · Q-3 tài xế offline thấy danh sách · Q-4 tài xế không biết thu tiền mặt hay không · Q-5 ETA không bị kiểm tính hợp lý so với quãng đường.
- **Đạt:** thứ tự danh sách, lọc theo loại xe, chặn tài xế bận, chặn sai loại xe, idempotent khi bấm đúp «Nhận chuyến» (phía API).
- **Phủ:** 7/7 tài nguyên trong mission (D101, D102, D103, D104, mã sai, ETA biên, bấm đúp/tải lại/danh sách để lâu); chưa thử ETA 1 và hai tab — chuyển CH-02.
- **Tái hiện tự động:** S1-01, S1-02, Q-2, S1-03, S1-04, S1-05 ⇒ `e2e/phat-hien-phien-driver.spec.ts` (ES-01…ES-06).
