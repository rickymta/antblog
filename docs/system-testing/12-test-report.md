# SYS-12 — Báo cáo chuẩn bị usability study và supporting checks

Ngày 13/11/2026. Windows, Node v24.15.0, Chrome channel cài sẵn (headless), viewport 1280×800. Working tree sau SYS-11 `1c32216`.

| Hạng mục | Trạng thái / bằng chứng |
|---|---|
| `npx playwright test --config usability.config.ts` tại lab | PASS, 2 tests, 5,3 giây, zero retry |
| US-A01 label/đoạn Tab/Enter/aria-live | PASS, 740 ms |
| US-A02 thông báo lỗi/sửa dữ liệu/báo giá lại | PASS, 508 ms |
| `npm run typecheck` tại lab | PASS, exit 0 |
| Study kit, task cards, worksheet, scoring và mẫu finding | Đã soạn; chưa pilot với người |
| Participant sessions | **0; chưa chạy** |
| Success rate / time-on-task / rating / findings người dùng | **Chưa có số liệu** |

Server PGlite riêng 3117, một worker, không reuse, reset dữ liệu/clock từng test. Nhãn input được kiểm ở các field dùng trong case; chỉ một đoạn Tab order từ voucher tới payment rồi Báo giá, với điểm bắt đầu focus bằng automation. Không tuyên bố toàn hành trình keyboard-only hoặc screen reader đạt. Error recovery chứng minh UI cho phép sửa, không chứng minh người dùng hiểu cách sửa.

Không chạy axe hoặc đánh giá WCAG đầy đủ. Không có ảnh baseline/review trực quan; trace/screenshot chỉ cấu hình lưu khi fail. Terminal cảnh báo NO_COLOR/FORCE_COLOR không gây failure.

Study kit tại `12-study-kit.md`: kế hoạch formative sáu người là đề xuất, không phải đã tuyển; bảng dữ liệu để trống. Các ví dụ tính tỷ lệ/viết finding trong bài đều là minh hoạ, không phải observation thực. Cần tổ chức pilot, tuyển participant phù hợp, xin đồng ý và thực hiện phiên trước khi kết luận usability.

Không sửa src sản phẩm, không rebuild web/admin, không chạy lại các suite không liên quan. Chưa xuất bản CMS. Kết luận: supporting checks hoạt động và bộ nghiên cứu đã chuẩn bị; **chưa đánh giá usability với người dùng thật**.
