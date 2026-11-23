# Minh hoạ series SYS bằng CurlStudio

Bộ collection Postman v2.1 dùng để minh hoạ các bài «Kiểm thử hệ thống Tới Nơi» (SYS-01…17) bằng [CurlStudio](https://github.com/rickymta/curl-studio) — API client kiểu Postman, có Collection Runner, `pm.test` và load test open/closed model. Collection cũng mở được bằng Postman/Newman.

Đây là công cụ **minh hoạ**: bằng chứng chính của từng bài vẫn là bộ test Vitest/Playwright trong `articles/system-testing-*`.

## Tệp

| Tệp | Nội dung |
|---|---|
| `toi-noi-lab.postman_collection.json` | Folder theo bài: 00 · Chuẩn bị lab, SYS-01 Smoke, SYS-02 Sanity voucher, SYS-03 Functional theo bảng cước, SYS-04 Tích hợp ví, SYS-10 Phân quyền, SYS-12 Thông điệp lỗi, SYS-13 Idempotency, SYS-14 Ví mất phản hồi kéo dài (RC-03), SYS-15 Supporting check UA-02, SYS-16 Post-deploy smoke, request cho kịch bản tải SYS-07…09 |
| `toi-noi-lab-local.postman_environment.json` | `baseUrl = http://localhost:3100` — máy chủ lab có API kiểm thử |
| `toi-noi-prod-like.postman_environment.json` | `baseUrl = http://localhost:3101` — bản «triển khai» tắt API kiểm thử |
| `data/bang-cuoc.csv` | 6 dòng cho SYS-03 (giá trị mong đợi tính tay theo R-FARE) |
| `data/che-do-vi.json` | 3 chế độ ví giả cho SYS-04 |
| `data/huy-chuyen.json` | 2 mốc huỷ (1 phút, 3 phút) cho SYS-15 |
| `build-collection.mjs` | Sinh lại các tệp trên: `node curlstudio/build-collection.mjs` |

## Chạy

Từ `examples/toi-noi-testing`, hai máy chủ (PowerShell):

```powershell
$env:NODE_ENV='test'; $env:TOINOI_DB='pglite'; $env:TOINOI_START_AT='2026-11-04T09:00:00+07:00'; $env:PORT='3100'; npm start
# cửa sổ khác — bản prod-like cho SYS-16
$env:NODE_ENV='production'; $env:TOINOI_DB='pglite'; $env:TOINOI_START_AT='2026-11-04T09:00:00+07:00'; $env:PORT='3101'; npm start
```

Mở CurlStudio (bản cài đặt, hoặc từ mã nguồn: `pnpm install` rồi `pnpm tauri dev` trong repo curl-studio), **Nhập** collection và hai environment, chọn environment «Tới Nơi lab (local, API test bật)». Chuột phải một folder → **Chạy thư mục** (Collection Runner) hoặc **Load test thư mục**. Folder SYS-03, SYS-04, SYS-15 cần chọn tệp dữ liệu trong runner (**Chọn file**).

Mỗi folder tự reset dữ liệu và đặt đồng hồ 09:00 ngày 04/11/2026 (không cao điểm, không mưa) trước khi kiểm. Lab giới hạn 5 lần tạo chuyến/phút cho mỗi khách (R-API-13): chạy một folder nhiều lần liên tiếp có thể gặp 429 — đợi một phút hoặc chạy folder `00 · Chuẩn bị lab`.

Ảnh trong bài chụp ngày 23/11/2026: CurlStudio 0.2.1 chạy bằng `pnpm tauri dev` trên Windows 11, giao diện tiếng Việt, theme sáng, cửa sổ 1600×900; workspace riêng qua `CURLSTUDIO_HOME` (không dùng dữ liệu cá nhân). Chỉ chạy collection và load test trên lab cô lập, không trỏ vào hệ thống thật.
