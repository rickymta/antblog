# @tkh/web — trang đọc Tech Knowledge Hub (Next.js App Router)

Trang đọc công khai (antblog.xyz): Server Components lấy dữ liệu thẳng từ API (cache 60 giây), phần tương tác
(phiên đăng nhập, reaction, bình luận, lượt xem, đánh dấu bài, sáng/tối, đăng ký tin; VI/EN khi bật cờ) chạy ở trình duyệt qua `api` của `@tkh/shared`.

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
| `NEXT_PUBLIC_ENABLE_I18N` | (trống = tắt) | Đúng chuỗi `true` ⇒ bật giao diện song ngữ (nút VI/EN ở thanh đầu + ngăn kéo điện thoại, cookie `tkh-locale`). **Nhúng lúc build.** Tắt (mặc định, tạm ẩn theo yêu cầu): không có nút VI/EN, ngôn ngữ luôn `vi`, máy chủ KHÔNG đọc cookie ⇒ trang bài/lộ trình là ISR; tài nguyên tiếng Anh không vào bundle trình duyệt. Bật ⇒ trang bài/lộ trình trở lại trang động (đọc cookie). |
| `GOOGLE_SITE_VERIFICATION` | (trống) | Mã xác minh Google Search Console ⇒ `<meta name="google-site-verification">`. Đọc lúc chạy. |
| `BING_SITE_VERIFICATION` | (trống) | Mã xác minh Bing Webmaster ⇒ `<meta name="msvalidate.01">`. Đọc lúc chạy. |

## SEO

- Metadata từng trang (`generateMetadata`): title mẫu `%s · Tech Knowledge Hub`, description, canonical, Open Graph, Twitter `summary_large_image`;
  bài viết dùng `seo.title` / `seo.description` / `seo.ogImageUrl` / `seo.noindex` khi có.
- Canonical danh sách bài/chủ đề: bỏ mọi tham số lọc, chỉ giữ `?page=N` (N > 1) khi trang không lọc; trang tìm kiếm (`q`) là `noindex, follow`.
- `robots` mặc định `index, follow` + googleBot `max-image-preview: large`, `max-snippet: -1`, `max-video-preview: -1` (`SITE_NOINDEX` / noindex từng trang ghi đè).
- JSON-LD: `WebSite` + `SearchAction` + `Organization` (trang chủ), `BlogPosting` + `BreadcrumbList` (bài), `BreadcrumbList` (chủ đề, lộ trình) + `ItemList` (lộ trình).
  `BlogPosting`: `image` là ImageObject URL tuyệt đối (ảnh SEO › ảnh bìa › ảnh OG sinh `/og/articles/<slug>` 1200×630), `publisher.logo` ImageObject 512×512,
  `wordCount` (phần chữ, bỏ khối mã/Markdown), `timeRequired` ISO 8601 (`PT7M`), `articleSection`, `inLanguage` (ngôn ngữ nội dung bài, mặc định `vi`).
- RSS 2.0 `/rss.xml` (50 bài mới nhất: title, link, guid, pubDate, description = tóm tắt, category; cache 10 phút) + `<link rel="alternate" type="application/rss+xml">` ở mọi trang.
- `apple-touch-icon` 180×180 (`app/apple-icon.tsx`), `theme-color` sáng/tối (export `viewport`).
- Slug bài / chủ đề / lộ trình không tồn tại: `src/proxy.ts` hỏi API (`HEAD`, nhớ 60 giây / 15 giây) rồi rewrite về trang 404 gốc ⇒ mã 404 **và** nội dung 404
  có sẵn trong HTML (không cần JavaScript). Lý do: `notFound()` ném trong page lồng nhau chỉ cho khung HTML rỗng (`__next_error__`), nội dung dựng ở trình duyệt.
- `sitemap.xml` từ `GET /site/sitemap` (bỏ bài `noindex`; API cũ chưa có endpoint ⇒ dựng từ các danh sách công khai), `robots.txt`, `manifest.webmanifest`,
  `icon.svg`, `logo.png`, ảnh OG mặc định `/opengraph-image` và ảnh OG sinh từ tiêu đề bài `/og/articles/[slug]` (bài không có ảnh bìa/ảnh SEO; font Inter ở `assets/fonts`).

## Hiệu năng & cache

- Trang bài `/articles/[slug]` và lộ trình `/learning-paths/[slug]` là **ISR** (`revalidate = 60`, `generateStaticParams` trả `[]`): không dựng gì lúc `next build`
  (ảnh Docker build không có API), dựng lần đầu khi có request rồi cache ở máy chủ (`Cache-Control: s-maxage=60, stale-while-revalidate`).
  Trang chủ, danh sách, chủ đề (đọc `searchParams`) vẫn là trang động — dữ liệu API vẫn cache 60 giây ở tầng fetch. Lúc build, `serverApi` gọi `connection()`
  để trang nào chạm API bị đánh dấu động thay vì gọi API; lúc chạy thì không (gọi API động trong trang ISR là lỗi).
- Phông: Inter **biến thiên** (`@fontsource-variable/inter`) qua `next/font/local` (`src/app/fonts.ts`) — 3 họ tách theo unicode-range (latin, vietnamese, latin-ext),
  trang thường chỉ tải 2 tệp woff2 (~59 KB); tệp latin được preload; phông dự phòng Arial chỉnh số đo chống nhảy chữ. JetBrains Mono chỉ nạp ở trang bài.
- Bình luận nạp mã + dữ liệu khi người đọc cuộn tới gần (`LazyComments`); số reaction/lượt xem vẫn có sẵn trong HTML máy chủ.
- Phiên đăng nhập: `GET /api/v1/auth/session` (luôn 200) — khách vãng lai không sinh request 4xx.
- `browserslist` (package.json): Chrome/Edge/Firefox 111, Safari 16.4.

## Ngôn ngữ & giao diện

- Song ngữ đang **tạm ẩn** (cờ `NEXT_PUBLIC_ENABLE_I18N`, mặc định tắt): `<html lang="vi">`, không nút VI/EN. Bật cờ (build lại) ⇒ hành vi dưới đây.
- Ngôn ngữ giao diện theo cookie `tkh-locale` (`vi` mặc định, 1 năm); URL không có mã ngôn ngữ. Nút VI/EN ghi cookie rồi `router.refresh()`.
- Sáng/tối: script nhỏ trong `<head>` áp class `.dark` từ localStorage `tkh-theme` trước khi trang hiển thị.
