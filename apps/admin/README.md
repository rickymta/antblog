# @tkh/admin — trang quản trị Tech Knowledge Hub (Vite + React)

```bash
npm run dev -w @tkh/admin         # http://127.0.0.1:5174 (/api, /uploads proxy tới API)
npm run typecheck -w @tkh/admin
npm run build -w @tkh/admin       # output: apps/admin/dist
```

## Biến môi trường (nhúng LÚC BUILD — Vite `import.meta.env`)

| Biến | Mặc định | Ghi chú |
|---|---|---|
| `VITE_SITE_URL` | `http://127.0.0.1:5173` (không đặt) | Địa chỉ trang đọc để mở bài viết; đặt rỗng (bản Docker) ⇒ cùng origin. |
| `VITE_ENABLE_I18N` | (trống = tắt) | Đúng chuỗi `true` ⇒ hiện nút VI/EN ở thanh đầu và màn đăng nhập (lựa chọn lưu localStorage `tkh-admin-locale`). Tắt (mặc định, tạm ẩn theo yêu cầu): không có nút VI/EN, giao diện luôn tiếng Việt (bỏ qua lựa chọn cũ đã lưu). Tài nguyên tiếng Anh vẫn giữ để bật lại. |
| `TKH_ADMIN_BASE` | `/` | `base` của Vite (bản Docker: `/admin/`). |
