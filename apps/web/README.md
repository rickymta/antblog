# @tkh/web — trang đọc Tech Knowledge Hub (Next.js App Router)

Trang đọc công khai (antblog.xyz): Server Components lấy dữ liệu thẳng từ API (cache 60 giây), phần tương tác
(phiên đăng nhập, reaction, bình luận, lượt xem, đánh dấu bài, sáng/tối, VI/EN, đăng ký tin) chạy ở trình duyệt qua `api` của `@tkh/shared`.

## Lệnh (chạy từ gốc repo)

```bash
npm run dev -w @tkh/web        # http://127.0.0.1:5173 (cần API ở API_INTERNAL_URL; /api và /uploads được rewrite sang API)
npm run typecheck -w @tkh/web
npm run build -w @tkh/web      # output: 'standalone'
npm run start -w @tkh/web      # next start, cổng 3000 (chỉ để thử; production dùng bản standalone bên dưới)
```

Production (Docker): sau `next build`, chép `apps/web/.next/static` vào `apps/web/.next/standalone/apps/web/.next/static`
(và `apps/web/public` nếu có tệp) rồi chạy từ thư mục `apps/web/.next/standalone`:

```bash
PORT=3000 HOSTNAME=0.0.0.0 node apps/web/server.js
```

## Biến môi trường

| Biến | Mặc định | Ghi chú |
|---|---|---|
| `API_INTERNAL_URL` | `http://127.0.0.1:4000` | Địa chỉ API mà **máy chủ Next** gọi (không có `/api/v1`). Docker: tên service API trong mạng compose. Đọc lúc chạy. |
| `NEXT_PUBLIC_SITE_URL` | `https://antblog.xyz` | URL gốc cho canonical, Open Graph, JSON-LD, sitemap. **Nhúng lúc build** — đặt khi build ảnh. |
| `SITE_NOINDEX` | (trống) | `true` ⇒ `robots.txt` chặn toàn bộ + mọi trang `noindex, nofollow` (staging). Đọc lúc chạy. |

## SEO

- Metadata từng trang (`generateMetadata`): title mẫu `%s · Tech Knowledge Hub`, description, canonical, Open Graph, Twitter `summary_large_image`;
  bài viết dùng `seo.title` / `seo.description` / `seo.ogImageUrl` / `seo.noindex` khi có.
- Canonical danh sách bài/chủ đề: bỏ mọi tham số lọc, chỉ giữ `?page=N` (N > 1) khi trang không lọc; trang tìm kiếm (`q`) là `noindex, follow`.
- JSON-LD: `WebSite` + `SearchAction` + `Organization` (trang chủ), `BlogPosting` + `BreadcrumbList` (bài), `BreadcrumbList` (chủ đề, lộ trình) + `ItemList` (lộ trình).
- `sitemap.xml` từ `GET /site/sitemap` (bỏ bài `noindex`; API cũ chưa có endpoint ⇒ dựng từ các danh sách công khai), `robots.txt`, `manifest.webmanifest`,
  `icon.svg`, `logo.png`, ảnh OG mặc định `/opengraph-image` và ảnh OG sinh từ tiêu đề bài `/og/articles/[slug]` (bài không có ảnh bìa/ảnh SEO; font Inter ở `assets/fonts`).

## Ngôn ngữ & giao diện

- Ngôn ngữ giao diện theo cookie `tkh-locale` (`vi` mặc định, 1 năm); URL không có mã ngôn ngữ. Nút VI/EN ghi cookie rồi `router.refresh()`.
- Sáng/tối: script nhỏ trong `<head>` áp class `.dark` từ localStorage `tkh-theme` trước khi trang hiển thị.
