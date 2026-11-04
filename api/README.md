# api/ — API cho Tech Knowledge Hub

Backend Express 5 + MongoDB (mongoose 8) cho blog kỹ thuật **Tech Knowledge Hub**. Hợp đồng API (nguồn sự thật duy nhất):
[`../docs/api-contract.md`](../docs/api-contract.md). Frontend (`apps/web`, `apps/admin`) gọi qua proxy Vite `/api` → `http://127.0.0.1:4000`
khi dev và qua nginx khi chạy Docker, nên cùng origin — API **không bật CORS**. Thư mục này có `package.json`/`package-lock.json`
riêng (không thuộc npm workspaces của gốc repo): chạy lệnh npm **trong `api/`**.

## Yêu cầu

- Node 22, npm 10.
- MongoDB chạy ở `127.0.0.1:27017` (không cần replica set).

## Chạy nhanh

```bash
npm install
cp .env.example .env        # rồi điền JWT_SECRET + SEED_ADMIN_PASSWORD ngẫu nhiên
npm run seed                # tạo admin + nạp chủ đề, bài viết, lộ trình từ seed/
npm run dev                 # http://127.0.0.1:4000/api/v1
```

Đăng nhập admin bằng `SEED_ADMIN_EMAIL` / `SEED_ADMIN_PASSWORD` trong `.env`.

## Biến môi trường

| Biến | Ý nghĩa |
|---|---|
| `NODE_ENV` | `production` ⇒ cookie có cờ `Secure` (trừ khi `COOKIE_SECURE=false`) |
| `COOKIE_SECURE` | Ghi đè cờ `Secure` của cookie (`true`/`false`) |
| `TRUST_PROXY` | `trust proxy` của Express — mặc định `loopback` (proxy Vite); Docker đặt subnet mạng compose |
| `HOST` | Địa chỉ nghe, mặc định `127.0.0.1` (chỉ máy cục bộ) |
| `PORT` | Cổng, mặc định `4000` |
| `MONGODB_URI` | Ví dụ `mongodb://127.0.0.1:27017/tech_knowledge_hub` |
| `JWT_SECRET` | Khoá ký phiên HS256, chuỗi ngẫu nhiên ≥ 48 byte |
| `SEED_ADMIN_EMAIL` / `SEED_ADMIN_PASSWORD` / `SEED_ADMIN_NAME` | Tài khoản admin do `npm run seed` tạo (chỉ tạo khi chưa có, không bao giờ đặt lại mật khẩu) |
| `UPLOAD_DIR` | Thư mục lưu ảnh thư viện media, mặc định `./uploads` (tính từ thư mục `api/`). API tự phục vụ `GET /uploads/*` từ đây (dev); Docker gắn volume và nginx phục vụ thẳng |
| `TEST_MONGODB_URI` | (tuỳ chọn) DB cho test, mặc định `…/tech_knowledge_hub_test` — tên phải kết thúc `_test` |

## Scripts

| Lệnh | Việc |
|---|---|
| `npm run dev` | Chạy dev (tsx watch) |
| `npm run build` / `npm start` | Biên dịch ra `dist/` / chạy bản build |
| `npm run typecheck` | Kiểm kiểu toàn repo (src, seed, test) |
| `npm test` | Vitest + supertest trên MongoDB thật, DB `tech_knowledge_hub_test` (tự xoá sau khi chạy) |
| `npm run seed` | Nạp dữ liệu từ `seed/` — idempotent, chạy lại không nhân đôi, giữ `publishedAt` và số liệu. Bài đã sửa trong CMS (`editedInCms`) được giữ nguyên (log "bỏ qua (đã sửa trong CMS)") |
| `npm run seed -- --force-content` | Như trên nhưng ghi đè cả bài đã sửa trong CMS bằng nội dung tệp `.md` (bỏ cờ `editedInCms`) |
| `npm run seed:demo` | Tạo hoạt động demo (8 tài khoản `@demo.local`, bình luận, like, reaction, số liệu truy cập 30 ngày), mọi thứ gắn `isDemo` |
| `npm run seed:demo -- --remove` | Gỡ sạch dữ liệu demo và tính lại bộ đếm |

## Dữ liệu seed

- `seed/categories.json`: `[{ slug, name: {vi,en}, description: {vi,en}, icon, color, order }]`
- `seed/learning-paths.json`: `[{ slug, title, description, level, category?, articles: [slug…], status }]`
- `seed/content/*.md`: Markdown có frontmatter `slug, title, summary, category, level, tags, language` (tuỳ chọn `coverImageUrl`, `publishedAt`).

Thiếu tệp nào thì seed ghi log và bỏ qua phần đó. Bài lỗi (frontmatter sai, chủ đề không tồn tại) bị bỏ qua kèm log.

## Cấu trúc

```
src/
  server.ts        khởi động: kết nối DB, nghe HOST:PORT
  app.ts           createApp(): middleware chung + gắn router dưới /api/v1
  config.ts        đọc .env
  db.ts            kết nối mongoose + tạo index (unique, TTL, text)
  models/          User, Category, Article, LearningPath, Comment, CommentLike, Reaction,
                   Subscriber, ArticleView (TTL 30'), DailyVisitor (TTL), DailySiteStat, DailyArticleStat, Media
  routes/          auth, public (site/sitemap/chủ đề/tag/lộ trình/track/subscribers), articles (+ view, reaction),
                   comments (+ like), admin, media (thư viện ảnh)
  services/        serializers (JSON theo hợp đồng), stats, counters, comments, articles
  middleware/      auth (cookie phiên, vai trò), common (415, cookie khách, lỗi), rateLimit
  lib/             errors, validate (zod, thông điệp tiếng Việt), dates (giờ VN), text (slug, thời gian đọc)
seed/              run.ts, demo.ts, lib.ts, demoLib.ts + dữ liệu
test/              *.test.ts, helpers.ts, fixtures/
```

## Thư viện media, SEO, lên lịch (hợp đồng §8)

- `POST /api/v1/admin/media` (multipart, trường `file` + `alt`) — ngoại lệ duy nhất của luật "ghi phải là JSON". Nhận PNG/JPEG/WebP/GIF ≤ 5 MB,
  loại ảnh xác định bằng magic bytes (không tin đuôi tệp/Content-Type), KHÔNG nhận SVG. Tệp lưu `UPLOAD_DIR/YYYY/MM/<24 hex>.<ext>`,
  bản ghi ở collection `media`. Xoá ảnh = xoá bản ghi + tệp (bài đang dùng ảnh sẽ mất ảnh — CMS cảnh báo trước khi xoá).
- `GET /uploads/*` (ngoài `/api/v1`): `express.static`, `Cache-Control: public, max-age=31536000, immutable`, `X-Content-Type-Options: nosniff`,
  `Content-Security-Policy: default-src 'none'; sandbox`, không liệt kê thư mục, không phục vụ dotfile.
- `seo` của bài (title ≤ 70, description ≤ 170, ogImageUrl http(s) hoặc `/uploads/...`, noindex); `coverImageUrl` cũng nhận `/uploads/...`.
- `POST /admin/articles/:id/publish` `{ publishAt? }`: tương lai ⇒ lên lịch (công khai tự ẩn tới giờ đó), quá khứ ⇒ 400. Danh sách admin lọc
  thêm `status=scheduled` (đã lên lịch) và `status=published` chỉ còn bài đang công khai. `articleCount` của chủ đề tính tại lúc đọc.
- `GET /site/sitemap`: bài công khai (kèm `noindex`), mọi chủ đề, lộ trình đã xuất bản.

## Ghi chú kỹ thuật

- Phiên: JWT HS256 trong cookie httpOnly `tkh_session` (SameSite=Lax, 7 ngày). Mỗi request tải lại user; user bị khoá ⇒ coi như chưa đăng nhập.
- Cookie khách `tkh_vid` (uuid v4, 400 ngày) đặt ở request đầu tiên — dùng đếm khách duy nhất và chống trùng lượt xem.
- Bộ đếm phi chuẩn hoá (`category.articleCount`, `article.stats.*`, `comment.likeCount`) cập nhật bằng `$inc` nguyên tử trong cùng thao tác;
  index unique (reaction, like, lượt xem, khách/ngày) chặn bấm đúp đếm hai lần.
- Ngày thống kê theo giờ Việt Nam (UTC+7). Bot (User-Agent chứa `bot|crawler|spider|preview`) không được đếm.
- Giới hạn tần suất dùng bộ nhớ trong tiến trình (một bản chạy).
