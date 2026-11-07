# Tech Knowledge Hub (antblog)

Blog chia sẻ kiến thức kỹ thuật từ cơ bản tới nâng cao: database, ngôn ngữ lập trình, design pattern, architecture pattern. Mọi ví dụ dùng chung dự án mẫu hư cấu **Tới Nơi** (đặt xe + giao hàng).

- Trang đọc: `https://antblog.xyz` · Quản trị: `https://admin.antblog.xyz`
- Trang đọc cho phép máy tìm kiếm lập chỉ mục (tắt bằng `SITE_NOINDEX=true`); trang quản trị luôn `noindex`.

## Cấu trúc

| Thư mục | Nội dung |
|---|---|
| `apps/web` | Trang đọc (Next.js, render phía server cho SEO) |
| `apps/admin` | Trang quản trị / CMS (React + Vite): soạn Markdown, thư viện ảnh, SEO, lên lịch đăng |
| `packages/shared` | Kiểu dữ liệu + client API + định dạng, dùng chung hai app |
| `packages/ui` | Theme Tailwind, bộ hiển thị bài viết, component dùng chung |
| `api` | API Express + MongoDB, seed nội dung — `package.json` riêng, không thuộc npm workspaces |
| `deploy` | nginx, certbot, MongoDB init, WireGuard, script sinh `.env` |
| `docs` | [`api-contract.md`](docs/api-contract.md) (hợp đồng frontend ↔ API), [`ui-guide.md`](docs/ui-guide.md) (hướng giao diện) |

## Khung nội dung Database

[PostgreSQL qua Tới Nơi](docs/postgresql-master-curriculum.md): 6 phần, tối đa 10 bài/phần, đủ cấp độ cơ bản, trung bình và nâng cao. Các series MySQL, MongoDB, MSSQL, Oracle và Cassandra sẽ có khung riêng dưới chuyên mục Database.

## Chạy trên máy (dev)

Yêu cầu: Node ≥ 22.12, MongoDB đang chạy ở `127.0.0.1:27017`.

```bash
# 1. API (cửa sổ 1)
cd api
npm install
cp .env.example .env   # điền JWT_SECRET, SEED_ADMIN_PASSWORD ngẫu nhiên
npm run seed           # chủ đề, bài viết, lộ trình, tài khoản quản trị
npm run dev            # http://127.0.0.1:4000

# 2. Frontend (cửa sổ 2, 3) — ở gốc repo
npm install
npm run dev:web        # trang đọc   http://127.0.0.1:5173
npm run dev:admin      # quản trị    http://127.0.0.1:5174
```

Cả hai app gọi API qua proxy của dev server (`/api`, `/uploads` → `127.0.0.1:4000`), nên cookie đăng nhập cùng origin và không cần CORS. Đổi địa chỉ API: biến môi trường `TKH_API_URL`.

## Triển khai lên server (Docker)

`docker-compose.yml` ở gốc repo chạy toàn bộ stack: nginx (cổng vào duy nhất 80/443: `antblog.xyz` trang đọc, `admin.antblog.xyz` quản trị, `/api/` ở cả hai),
API, MongoDB có xác thực, certbot (chứng chỉ Let's Encrypt qua DNS Cloudflare) và WireGuard (VPN vào database).
Hướng dẫn từng bước: [`deploy/README.md`](deploy/README.md).

## Kiểm tra trước khi commit

```bash
npm run typecheck && npm run build          # frontend, ở gốc repo
cd api && npm run typecheck && npm test     # API (cần MongoDB cục bộ)
```

## Quy ước

- Giao diện song ngữ vi/en (i18next), mặc định tiếng Việt; nội dung bài giữ ngôn ngữ gốc.
- Màu, font, bo góc theo `packages/ui/src/theme.css` (primary `#2563EB`, secondary `#6366F1`, nền `#F8FAFC` / `#0F172A`, Inter). Dùng các lớp ngữ nghĩa `bg-page`, `bg-surface`, `text-ink`, `text-muted`, `border-line`… thay vì mã màu rời.
- Nội dung bài viết render bằng `ArticleContent` của `@tkh/ui` ở CẢ trang đọc và bản xem trước quản trị. Bình luận của người đọc luôn hiển thị dạng văn bản thuần.
- Hình minh hoạ trong bài là SVG viết tay dùng bộ lớp cố định trong `packages/ui/src/article.css` (phys = lớp vật lý, pk = lớp ứng dụng nhìn thấy, warn = chỗ phải trả giá, dead = đã chết).
- Ví dụ trong bài luôn lấy từ dự án mẫu Tới Nơi; không đưa thông tin của dự án/công ty thật nào vào blog.
- Đổi hợp đồng API ⇒ sửa `docs/api-contract.md`, `packages/shared/src/types.ts` và `api/` trong cùng một commit.
