# SYS-11 — Báo cáo Compatibility Test

Ngày 13/11/2026. Windows, Node v24.15.0; working tree sau SYS-10 `841e39f`. Lệnh tại lab: `npx playwright test --config compatibility.config.ts`.

| Project | Browser thực đo | Cấu hình | Kết quả |
|---|---|---|---|
| chrome-desktop-vi | Chrome 157.0.8103.51 | 1280×800, vi-VN, Asia/Ho_Chi_Minh | CP-01/02 PASS, 984/597 ms |
| edge-desktop-en | Edge 157.0.4341.46 | 1280×800, en-US, America/New_York | CP-01/02 PASS, 1,1 s/618 ms |
| chrome-narrow-touch | Chrome 157.0.8103.51 | 390×844, mobile/touch emulation, vi-VN/VN | CP-01/02 PASS, 930/645 ms |

Tổng 6 executions pass trong 10,8 giây, không retry. `npm run typecheck` tại lab đạt, exit 0. Có terminal warning NO_COLOR/FORCE_COLOR, không làm test fail.

Quan sát: cả ba cấu hình hiện đúng 35.000 đ cho ride và 95.000 đ cho food, báo giá/đặt/huỷ hoạt động, phí huỷ searching 0, field động và trạng thái disabled đúng; lỗi quãng đường hiển thị. Không có horizontal overflow vượt tolerance 1 CSS pixel tại các điểm kiểm. Không phát hiện defect sản phẩm trong phạm vi đó.

Server PGlite riêng 3116, không reuse, một worker, reset DB/clock trước mỗi execution. Browser headless; context/metadata ghi theo config. Trace/screenshot chỉ lưu khi lỗi nên không có visual baseline hoặc ảnh được review ở lượt pass này.

Chrome và Edge đều Chromium; narrow-touch không phải Android/iOS thật. Chưa kiểm Firefox/WebKit/Safari, OS khác, zoom/orientation, keyboard ảo, screen reader, mọi viewport hay API version compatibility. Edge đổi đồng thời locale/timezone/browser nên nếu có lỗi cần thí nghiệm tách yếu tố. Không suy ra support policy rộng hơn matrix đã chạy.

Không sửa src, không rebuild web/admin, chưa xuất bản CMS. Lệnh config mặc định E2E không tương đương matrix này; tái hiện bằng đúng compatibility.config.ts và browser versions tương ứng nếu cần so sánh chính xác.
