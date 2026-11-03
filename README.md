# Tech Knowledge Hub — frontend

Blog chia sẻ kiến thức kỹ thuật từ cơ bản tới nâng cao: database, ngôn ngữ lập trình, design pattern, architecture pattern. Mọi ví dụ dùng chung dự án mẫu hư cấu **Tới Nơi** (đặt xe + giao hàng).

> Dự án chạy riêng trên máy này, **chưa công khai**: dev server chỉ nghe `127.0.0.1`, trang có `noindex`, git không có remote.

## Hai repo

| Repo | Nội dung |
|---|---|
| `H:\Work\tech-blog` (repo này) | Monorepo frontend: `apps/web` (trang đọc), `apps/admin` (quản trị), `packages/shared` (kiểu dữ liệu + client API + định dạng), `packages/ui` (theme Tailwind, bộ hiển thị bài viết, component dùng chung) |
| `H:\Work\tech-blog-api` | API Express + MongoDB, seed nội dung |

Hợp đồng giữa hai repo: [`docs/api-contract.md`](docs/api-contract.md) (bản chép ở repo API). Hướng giao diện tổng hợp từ ảnh mẫu: [`docs/ui-guide.md`](docs/ui-guide.md).

## Chạy trên máy

Yêu cầu: Node ≥ 22.12, MongoDB đang chạy ở `127.0.0.1:27017` (service Windows `MongoDB`).

```bash
# 1. API (cửa sổ 1)
cd H:/Work/tech-blog-api
npm install
npm run seed        # chủ đề, bài viết, lộ trình, tài khoản quản trị (đọc .env)
npm run dev         # http://127.0.0.1:4000

# 2. Frontend (cửa sổ 2, 3)
cd H:/Work/tech-blog
npm install
npm run dev:web     # trang đọc   http://127.0.0.1:5173
npm run dev:admin   # quản trị    http://127.0.0.1:5174
```

Cả hai app gọi API qua proxy Vite (`/api` → `127.0.0.1:4000`), nên cookie đăng nhập dùng chung và không cần CORS. Đổi địa chỉ API: biến môi trường `TKH_API_URL`.

Tài khoản quản trị nằm trong `H:\Work\tech-blog-api\.env` (`SEED_ADMIN_EMAIL`, `SEED_ADMIN_PASSWORD`).

## Kiểm tra trước khi commit

```bash
npm run typecheck
npm run build
```

## Quy ước

- Giao diện song ngữ vi/en (i18next), mặc định tiếng Việt; nội dung bài giữ ngôn ngữ gốc.
- Màu, font, bo góc theo `packages/ui/src/theme.css` (primary `#2563EB`, secondary `#6366F1`, nền `#F8FAFC` / `#0F172A`, Inter). Dùng các lớp ngữ nghĩa `bg-page`, `bg-surface`, `text-ink`, `text-muted`, `border-line`… thay vì mã màu rời.
- Nội dung bài viết render bằng `ArticleContent` của `@tkh/ui` ở CẢ trang đọc và bản xem trước quản trị. Bình luận của người đọc luôn hiển thị dạng văn bản thuần.
- Hình minh hoạ trong bài là SVG viết tay dùng bộ lớp cố định trong `packages/ui/src/article.css` (phys = lớp vật lý, pk = lớp ứng dụng nhìn thấy, warn = chỗ phải trả giá, dead = đã chết).
- Ví dụ trong bài luôn lấy từ dự án mẫu Tới Nơi; không đưa thông tin của dự án/công ty thật nào vào blog.
