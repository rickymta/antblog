# Hợp đồng API — Tech Knowledge Hub

Nguồn sự thật DUY NHẤT giữa frontend và API trong repo này:
- Frontend (`apps/web`, `apps/admin`, `packages/*`) — kiểu TypeScript tương ứng ở `packages/shared/src/types.ts`.
- Backend `api/` (Express + MongoDB).

Đổi hợp đồng ⇒ sửa file này + `packages/shared/src/types.ts` + code `api/` trong cùng một commit.

## 0. Quy ước chung

- Gốc: `/api/v1`. Frontend dev gọi qua proxy Vite (`/api` → `http://127.0.0.1:4000`) nên cùng origin — cookie hoạt động không cần CORS.
- JSON cả hai chiều. Mọi request ghi (POST/PUT/PATCH/DELETE) phải có `Content-Type: application/json` (kể cả thân rỗng `{}`) — API trả 415 nếu thiếu (chống CSRF dạng form).
- Thời gian: ISO 8601 UTC (`2026-11-03T04:12:00.000Z`). Ngày thống kê: chuỗi `YYYY-MM-DD` theo giờ Việt Nam (UTC+7).
- Id: chuỗi ObjectId 24 ký tự hex, trường tên `id` (KHÔNG trả `_id`, `__v`).
- Phân trang: query `page` (≥ 1, mặc định 1), `pageSize` (1–50, mặc định 12) ⇒ `{ items, total, page, pageSize }`.
- Lỗi: HTTP status + thân `{ "error": { "code": "SNAKE_UPPER", "message": "câu cho người đọc (tiếng Việt)", "details"?: {...} } }`.
  - 400 `VALIDATION` (`details.fields: { [field]: string }`), 401 `UNAUTHENTICATED`, 403 `FORBIDDEN`, 404 `NOT_FOUND`, 409 (mã riêng), 415 `UNSUPPORTED_MEDIA_TYPE`, 429 `RATE_LIMITED`, 500 `INTERNAL`.
- Trường rỗng/không có: bỏ khỏi JSON (không trả `null` trừ khi ghi rõ).

## 1. Xác thực

- Phiên = JWT ký HS256 (`JWT_SECRET`), lưu cookie **httpOnly** `tkh_session`, `SameSite=Lax`, `Path=/`, `Secure` khi `NODE_ENV=production`, hạn 7 ngày. Payload `{ sub: userId, role }`. Mỗi request có cookie hợp lệ ⇒ API tải lại user từ DB (user bị khoá ⇒ coi như chưa đăng nhập + xoá cookie).
- Mật khẩu băm bcrypt (bcryptjs, cost 12). Email lưu chữ thường, đã trim.
- Vai trò: `reader` (mặc định khi đăng ký), `author` (vào admin, chỉ quản bài của mình), `admin` (toàn quyền).
- Cookie khách `tkh_vid` (uuid v4, httpOnly, SameSite=Lax, 400 ngày) — API tự đặt ở request đầu tiên không có nó (middleware chung) để đếm khách duy nhất và lượt xem.
- Giới hạn: `/auth/login` + `/auth/register` 10 lần / 15 phút / IP ⇒ 429.

| Method | Path | Thân | Trả |
|---|---|---|---|
| POST | `/auth/register` | `{ email, password (8–72), displayName (2–60) }` | 201 `{ user: User }` + đặt cookie. Trùng email ⇒ 409 `EMAIL_TAKEN` |
| POST | `/auth/login` | `{ email, password }` | 200 `{ user }` + cookie. Sai ⇒ 401 `INVALID_CREDENTIALS`; bị khoá ⇒ 403 `ACCOUNT_LOCKED` |
| POST | `/auth/logout` | `{}` | 204, xoá cookie |
| GET | `/auth/me` | — | 200 `{ user }` hoặc 401 |
| PATCH | `/auth/me` | `{ displayName?, avatarUrl? (http/https ≤ 500) , bio? (≤ 300) }` | 200 `{ user }` |

`User = { id, email, displayName, avatarUrl?, bio?, role, status: 'active'|'locked', createdAt }` (email chỉ trả cho chính chủ và admin; ở chỗ công khai dùng `AuthorRef = { id, displayName, avatarUrl?, role }`).

## 2. Nội dung công khai

Chỉ trả bài `status = 'published'` và `publishedAt <= now`.

`Localized = { vi: string, en: string }`

`Category = { id, slug, name: Localized, description: Localized, icon: CategoryIcon, color: string (#RRGGBB), order: number, articleCount: number }`
`CategoryIcon = 'database' | 'code' | 'pattern' | 'architecture' | 'devops' | 'system' | 'security'`

`Level = 'beginner' | 'intermediate' | 'advanced'`

`ReactionType = 'love' | 'helpful' | 'insightful' | 'inspiring'`
`ReactionCounts = { love: number, helpful: number, insightful: number, inspiring: number }`

`ArticleStats = { views: number, comments: number, reactions: ReactionCounts }`

`ArticleCard = { id, slug, title, summary, coverImageUrl?, language: 'vi'|'en', category: { id, slug, name: Localized, icon, color }, level, tags: string[], author: AuthorRef, readingMinutes, publishedAt, updatedAt, stats: ArticleStats }`

`ArticleDetail = ArticleCard & { content: string (Markdown, có thể chứa HTML/SVG do tác giả viết), learningPath?: { id, slug, title, level, articles: { id, slug, title, readingMinutes }[] } , myReactions: ReactionType[] (rỗng khi chưa đăng nhập) }`

| Method | Path | Ghi chú |
|---|---|---|
| GET | `/site/summary` | `{ articleCount, categoryCount, learningPathCount, memberCount, totalViews }` |
| GET | `/categories` | `Category[]` sắp theo `order` |
| GET | `/categories/:slug` | `Category` hoặc 404 |
| GET | `/articles` | query: `category` (slug), `level`, `tag`, `q` (tìm toàn văn tiêu đề/tóm tắt/tag), `sort` = `latest` (mặc định, `publishedAt` giảm) \| `popular` (views giảm) \| `trending` (lượt xem 7 ngày qua giảm), `page`, `pageSize` ⇒ trang `ArticleCard` |
| GET | `/articles/trending?limit=4` | `ArticleCard[]` (lượt xem 7 ngày; thiếu dữ liệu thì bù bằng `popular`), `limit` 1–12 |
| GET | `/articles/:slug` | `ArticleDetail`; KHÔNG tự tăng lượt xem |
| GET | `/articles/:slug/related?limit=3` | `ArticleCard[]`: cùng chủ đề, ưu tiên trùng tag, bỏ chính nó |
| GET | `/tags/popular?limit=20` | `{ tag, count }[]` |
| GET | `/learning-paths` | query `level`, `q` ⇒ `LearningPathCard[]` (chỉ `published`) |
| GET | `/learning-paths/:slug` | `LearningPathDetail` |

`LearningPathCard = { id, slug, title, description, level, category?: { slug, name: Localized, color, icon }, articleCount, totalMinutes, updatedAt }`
`LearningPathDetail = LearningPathCard & { articles: ArticleCard[] (đúng thứ tự học, bỏ bài chưa xuất bản) }`

## 3. Thống kê lượt xem / truy cập

| Method | Path | Ghi chú |
|---|---|---|
| POST | `/track/visit` | `{ path }` — gọi MỘT lần mỗi phiên trình duyệt (frontend giữ cờ `sessionStorage`). Tăng `visits` ngày hôm nay; tăng `uniqueVisitors` nếu `tkh_vid` chưa ghi nhận trong ngày. 204 |
| POST | `/articles/:slug/view` | `{}` — tăng `stats.views` nếu cặp (bài, `tkh_vid`) chưa được đếm trong **30 phút** gần nhất (collection chống trùng có TTL). Trả `{ counted: boolean, views: number }`. Frontend gọi sau khi bài hiển thị 5 giây hoặc khi người đọc cuộn qua 30% bài, mỗi lần mở trang một lần |

Bot (User-Agent chứa `bot|crawler|spider|preview`) không được đếm.

## 4. Reaction

| Method | Path | Ghi chú |
|---|---|---|
| PUT | `/articles/:slug/reactions/:type` | Cần đăng nhập. Thêm reaction (idempotent). ⇒ `{ reactions: ReactionCounts, myReactions: ReactionType[] }` |
| DELETE | `/articles/:slug/reactions/:type` | Bỏ reaction (idempotent), cùng hình dạng trả về |

Mỗi người tối đa 1 reaction MỖI LOẠI mỗi bài (chọn nhiều loại được). `type` lạ ⇒ 400.

## 5. Bình luận (2 cấp)

`Comment = { id, articleId, parentId?: string, replyTo?: AuthorRef, author: AuthorRef, body: string, status: 'visible'|'deleted', likeCount: number, likedByMe: boolean, editedAt?, createdAt }`
- `status: 'deleted'` ⇒ `body` = `""` và `author` vẫn trả (frontend hiện "Bình luận đã bị xoá"). Bình luận bị admin ẩn (`hidden`) KHÔNG trả ra công khai; nếu là gốc có phản hồi thì trả như `deleted`.
`CommentThread = Comment & { replies: Comment[], replyCount: number }`

| Method | Path | Ghi chú |
|---|---|---|
| GET | `/articles/:slug/comments` | query `sort` = `newest` (mặc định) \| `oldest` \| `top` (likeCount giảm, rồi mới nhất), `page`, `pageSize` (mặc định 10) — phân trang theo **bình luận gốc**; mỗi gốc kèm TẤT CẢ phản hồi (cũ → mới). ⇒ `{ items: CommentThread[], total (số gốc), page, pageSize, commentCount (tổng bình luận đang hiện, = stats.comments) }` |
| POST | `/articles/:slug/comments` | Cần đăng nhập. `{ body (1–2000 sau trim), parentId? }`. **Luật 2 cấp:** `parentId` là bình luận gốc ⇒ tạo phản hồi; `parentId` là một phản hồi ⇒ phản hồi mới gắn vào **gốc của nó** và `replyTo` = tác giả phản hồi đó. `parentId` không thuộc bài ⇒ 400; gốc đã xoá/ẩn ⇒ 409 `PARENT_UNAVAILABLE`. Giới hạn 10 bình luận / phút / người ⇒ 429. ⇒ 201 `Comment` |
| PATCH | `/comments/:id` | Chỉ chủ bình luận. `{ body }` ⇒ đặt `editedAt` ⇒ `Comment` |
| DELETE | `/comments/:id` | Chủ bình luận hoặc admin ⇒ xoá mềm (`status: 'deleted'`, xoá `body`). 204 |
| PUT | `/comments/:id/like` | Cần đăng nhập ⇒ `{ likeCount, likedByMe: true }` |
| DELETE | `/comments/:id/like` | ⇒ `{ likeCount, likedByMe: false }` |

`stats.comments` của bài = số bình luận `visible` (gốc + phản hồi). Ghi bình luận, xoá, ẩn/hiện đều cập nhật số này trong cùng thao tác.

## 6. Đăng ký nhận tin

`POST /subscribers` `{ email }` ⇒ 201 `{ ok: true }` (trùng ⇒ 200 `{ ok: true }`, không lộ là đã có).

## 7. Quản trị (`/admin/*`)

Cần đăng nhập với vai trò `admin` hoặc `author` (`reader` ⇒ 403). `author` chỉ thấy/sửa bài do mình là tác giả; mọi mục khác (chủ đề, lộ trình, bình luận, người dùng, đăng ký tin, thống kê toàn site) chỉ `admin`.

### 7.1 Thống kê

`GET /admin/stats/overview?days=30` (7–180) ⇒
```
{
  range: { from: 'YYYY-MM-DD', to: 'YYYY-MM-DD', days },
  totals: { articles, published, drafts, users, comments, subscribers, views, visits, uniqueVisitors },   // views/visits/uniqueVisitors/comments: trong kỳ; còn lại: tổng hiện tại
  previous: { views, visits, uniqueVisitors, comments },                                                   // kỳ liền trước cùng độ dài
  series: [{ date, views, visits, uniqueVisitors, comments, reactions }],                                  // đủ mọi ngày trong kỳ, ngày trống = 0
  topArticles: [{ id, slug, title, views, comments, reactions }]                                           // 10 bài nhiều lượt xem nhất trong kỳ
}
```

### 7.2 Bài viết

`AdminArticle = ArticleDetail-không-có-myReactions & { status: 'draft'|'published'|'archived', learningPathId? }` (thêm `content`, có cả bài nháp).

| Method | Path | Ghi chú |
|---|---|---|
| GET | `/admin/articles` | query `status`, `category`, `q`, `page`, `pageSize` ⇒ trang `AdminArticle` (không kèm `content`) |
| POST | `/admin/articles` | `ArticleInput` ⇒ 201 `AdminArticle` (status `draft`, tác giả = người gọi) |
| GET | `/admin/articles/:id` | `AdminArticle` |
| PUT | `/admin/articles/:id` | `ArticleInput` ⇒ `AdminArticle`. Slug trùng ⇒ 409 `SLUG_TAKEN` |
| POST | `/admin/articles/:id/publish` | đặt `published`, `publishedAt` = now nếu chưa có |
| POST | `/admin/articles/:id/unpublish` | về `draft` |
| DELETE | `/admin/articles/:id` | chuyển `archived` (không xoá cứng); 204 |

`ArticleInput = { title (3–200), slug? (a-z0-9-, tự sinh từ tiêu đề bỏ dấu nếu trống), summary (≤ 400), content (Markdown), coverImageUrl?, language: 'vi'|'en', categoryId, level, tags: string[] (≤ 10, chữ thường) }`
`readingMinutes` do API tính: số từ của `content` (bỏ thẻ HTML/SVG, khối mã tính 1/2) ÷ 200, làm tròn lên, tối thiểu 1.

### 7.3 Chủ đề

`GET/POST /admin/categories`, `PUT/DELETE /admin/categories/:id`. `CategoryInput = { slug, name: Localized, description: Localized, icon, color, order }`. Xoá chủ đề còn bài ⇒ 409 `CATEGORY_IN_USE`.

### 7.4 Lộ trình học

`GET/POST /admin/learning-paths`, `GET/PUT/DELETE /admin/learning-paths/:id`.
`LearningPathInput = { slug, title, description, level, categoryId?, articleIds: string[] (thứ tự học), status: 'draft'|'published' }`. Một bài thuộc tối đa một lộ trình (gán vào lộ trình mới ⇒ gỡ khỏi lộ trình cũ). `totalMinutes` = tổng `readingMinutes` bài đã xuất bản.

### 7.5 Bình luận

`GET /admin/comments` query `status` (`visible`|`hidden`|`deleted`), `articleId`, `q`, `page` ⇒ trang `AdminComment = Comment & { status: 'visible'|'hidden'|'deleted', article: { id, slug, title } }`.
`PATCH /admin/comments/:id` `{ status: 'visible'|'hidden' }` ⇒ `AdminComment` (cập nhật `stats.comments`).
`DELETE /admin/comments/:id` ⇒ xoá mềm, 204.

### 7.6 Người dùng

`GET /admin/users` query `q`, `role`, `status`, `page` ⇒ trang `AdminUser = User & { commentCount, lastLoginAt? }`.
`PATCH /admin/users/:id` `{ role?, status? }`. Admin không được tự hạ vai trò hay tự khoá ⇒ 409 `CANNOT_MODIFY_SELF`.

### 7.7 Đăng ký tin

`GET /admin/subscribers?page=` ⇒ trang `{ id, email, createdAt }`.
