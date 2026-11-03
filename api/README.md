# tech-blog-api — API cho Tech Knowledge Hub

Backend Express 5 + MongoDB (mongoose 8) cho blog kỹ thuật riêng **Tech Knowledge Hub**. Hợp đồng API (nguồn sự thật duy nhất):
[`docs/api-contract.md`](docs/api-contract.md). Frontend (monorepo `tech-blog`) gọi qua proxy Vite `/api` → `http://127.0.0.1:4000`
nên cùng origin — API **không bật CORS**.

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
| `NODE_ENV` | `production` ⇒ cookie có cờ `Secure` |
| `HOST` | Địa chỉ nghe, mặc định `127.0.0.1` (chỉ máy cục bộ) |
| `PORT` | Cổng, mặc định `4000` |
| `MONGODB_URI` | Ví dụ `mongodb://127.0.0.1:27017/tech_knowledge_hub` |
| `JWT_SECRET` | Khoá ký phiên HS256, chuỗi ngẫu nhiên ≥ 48 byte |
| `SEED_ADMIN_EMAIL` / `SEED_ADMIN_PASSWORD` / `SEED_ADMIN_NAME` | Tài khoản admin do `npm run seed` tạo (chỉ tạo khi chưa có, không bao giờ đặt lại mật khẩu) |
| `TEST_MONGODB_URI` | (tuỳ chọn) DB cho test, mặc định `…/tech_knowledge_hub_test` — tên phải kết thúc `_test` |

## Scripts

| Lệnh | Việc |
|---|---|
| `npm run dev` | Chạy dev (tsx watch) |
| `npm run build` / `npm start` | Biên dịch ra `dist/` / chạy bản build |
| `npm run typecheck` | Kiểm kiểu toàn repo (src, seed, test) |
| `npm test` | Vitest + supertest trên MongoDB thật, DB `tech_knowledge_hub_test` (tự xoá sau khi chạy) |
| `npm run seed` | Nạp dữ liệu từ `seed/` — idempotent, chạy lại không nhân đôi, giữ `publishedAt` và số liệu |
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
                   Subscriber, ArticleView (TTL 30'), DailyVisitor (TTL), DailySiteStat, DailyArticleStat
  routes/          auth, public (site/chủ đề/tag/lộ trình/track/subscribers), articles (+ view, reaction),
                   comments (+ like), admin
  services/        serializers (JSON theo hợp đồng), stats, counters, comments, articles
  middleware/      auth (cookie phiên, vai trò), common (415, cookie khách, lỗi), rateLimit
  lib/             errors, validate (zod, thông điệp tiếng Việt), dates (giờ VN), text (slug, thời gian đọc)
seed/              run.ts, demo.ts, lib.ts, demoLib.ts + dữ liệu
test/              *.test.ts, helpers.ts, fixtures/
```

## Ghi chú kỹ thuật

- Phiên: JWT HS256 trong cookie httpOnly `tkh_session` (SameSite=Lax, 7 ngày). Mỗi request tải lại user; user bị khoá ⇒ coi như chưa đăng nhập.
- Cookie khách `tkh_vid` (uuid v4, 400 ngày) đặt ở request đầu tiên — dùng đếm khách duy nhất và chống trùng lượt xem.
- Bộ đếm phi chuẩn hoá (`category.articleCount`, `article.stats.*`, `comment.likeCount`) cập nhật bằng `$inc` nguyên tử trong cùng thao tác;
  index unique (reaction, like, lượt xem, khách/ngày) chặn bấm đúp đếm hai lần.
- Ngày thống kê theo giờ Việt Nam (UTC+7). Bot (User-Agent chứa `bot|crawler|spider|preview`) không được đếm.
- Giới hạn tần suất dùng bộ nhớ trong tiến trình (một bản chạy).
