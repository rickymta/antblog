# Giai đoạn 1–5 — Tính năng từ điển keyword

## Trạng thái triển khai (11/2026)

| Giai đoạn | Trạng thái | Mã nguồn |
|---|---|---|
| 1. Model, API, seed | Xong | `api/src/models/term.ts`, `api/src/routes/glossary.ts` (`GET /glossary`, `GET /glossary/:slug`, `/admin/terms`), bước seed trong `api/seed/lib.ts` |
| 2. Dò keyword khi render | Xong | Bộ dò `packages/shared/src/glossary.ts`, plugin `packages/ui/src/rehypeGlossary.ts`, nối vào `ArticleContent` |
| 3. Hộp thoại | Xong | `apps/web/src/components/article/GlossaryPopover.tsx`, CSS `.tkh-term` trong `packages/ui/src/article.css` |
| — Xem trước trong CMS | Xong | `useGlossary` trong `apps/admin/src/lib/queries.ts`. Bản xem trước chỉ đánh dấu keyword, chưa có hộp thoại |
| 4. Trang quản trị keyword | Xong | `apps/admin/src/pages/TermsPage.tsx` (route `/terms`, chỉ admin): danh sách có tìm kiếm và lọc, form có xem trước hộp thoại, nút đếm số bài chứa keyword (`POST /admin/terms/usage`, `api/src/services/glossaryUsage.ts`) |
| 5. Trang `/glossary`, SEO | Xong | `apps/web/src/app/glossary/page.tsx` (A–Z, nhóm theo chữ cái), `glossary/[slug]/page.tsx` (định nghĩa, giải thích dài, cách gọi khác, bài liên quan); JSON-LD `DefinedTermSet`/`DefinedTerm`, sitemap, link ở footer, link "Xem thêm" trong hộp thoại. `GET /glossary/:slug` trả tối đa 6 bài đã xuất bản có nhắc tới thuật ngữ |
| Tắt từ điển cho cả bài | Xong | Frontmatter `glossary: false` (seed) hoặc ô "Đánh dấu keyword" trong trình soạn bài CMS. Trường `glossary` chỉ lưu khi bằng `false`; trang bài và bản xem trước bỏ qua việc dò keyword. Tắt từng vùng vẫn dùng `data-no-glossary` |
| Dữ liệu từ điển | 387 thuật ngữ | Seed từ các mục `glossary: true` trong `api/seed/terminology.json`. Đợt 1 (67 mục) chọn theo số bài nhắc tới trên toàn site. Đợt 2 (320 mục) rà theo từng nhóm bài trừ PostgreSQL và MSSQL: lọc cụm tiếng Anh theo tần suất, agent đề xuất và kiểm nghĩa theo đoạn văn thật, rồi loại các từ mang nghĩa khác ở nhóm bài khác (ví dụ ownership, lifetime trong bài .NET). Cả hai đợt bỏ các từ quá chung hoặc nhiều nghĩa như index, timeout, transaction, baseline, coverage. Mục có thể khai báo thêm `aliases` và `caseSensitive` (ví dụ `test oracle` có alias `oracle` và phân biệt hoa thường để không khớp tên hệ quản trị Oracle) |

Test: `api/test/glossary.test.ts` (API, seed, đếm số bài), `glossary-matcher.test.ts` (bộ dò), `glossary-render.test.ts` (plugin chạy qua `ArticleContent`).

**Khác với thiết kế ban đầu bên dưới:**
- Dữ liệu cho hộp thoại không được truyền riêng qua `vfile.data`, mà nằm luôn trên thẻ `dfn`, trong các thuộc tính `data-term-title`, `data-term-english`, `data-term-definition`. Lý do: `react-markdown` render ở server không trả ra danh sách các keyword đã khớp. Cách này chỉ tốn thêm vài trăm byte cho mỗi keyword, vì mỗi keyword chỉ được đánh dấu lần đầu. Đổi lại, bản xem trước trong CMS có sẵn dữ liệu mà không cần làm thêm gì.
- Hộp thoại tự tính vị trí (trên hay dưới, giữ trong màn hình với lề 16 px, chọn đúng dòng khi keyword xuống dòng), không dùng thư viện `@floating-ui`.
- Bộ dò keyword đặt trong `@tkh/shared` thay vì `@tkh/ui`, để API dùng lại được ở giai đoạn 4 (đếm số bài chứa một keyword).
- Hộp thoại khi đã ghim (mở bằng nhấp, chạm, Enter hoặc Space) dùng `role="dialog"`, vì nó có nút đóng và link "Xem thêm". Khi mở bằng phím, focus được chuyển vào trong hộp thoại. Khi chỉ rê chuột hoặc focus, hộp thoại vẫn là `role="tooltip"`.
- Bộ đếm số bài chứa keyword ở API dùng `api/src/lib/contentLint.ts` (chuyển từ `api/seed/` để bản build API dùng được), không dùng bộ dò của `@tkh/shared`, vì API là gói npm riêng. Bộ đếm quét toàn bộ văn xuôi (chỉ che code, thẻ HTML, đích liên kết), nên kết quả là ước lượng trên: trang đọc còn bỏ qua chữ trong heading và link, và chỉ đánh dấu lần xuất hiện đầu tiên mỗi bài.
- Plugin bỏ qua mọi bảng nằm trong callout `.tkh-callout` (bảng "Thông tin bài" của loạt testing và vài bảng tóm tắt), vì nhãn như "Test Level", "Test Type" ở đó không phải văn xuôi. Keyword vẫn được đánh dấu ở lần xuất hiện đầu tiên trong thân bài.
- Trang bài lấy từ điển qua `serverApi.glossary()` với cache 5 phút. Nếu API lỗi, bài vẫn hiển thị bình thường, chỉ không đánh dấu keyword.

## Hành vi với người đọc

- Keyword trong bài có gạch chân chấm mờ.
- **Máy tính** (`pointer: fine`): rê chuột vào keyword khoảng 300 ms thì hộp thoại hiện ra. Rê chuột từ keyword sang hộp thoại không làm hộp thoại đóng. Tab tới keyword (focus) cũng mở hộp thoại.
- **Thiết bị cảm ứng**: chạm vào keyword để mở. Chạm ra ngoài hoặc chạm lại keyword để đóng.
- Hộp thoại tự chọn hiện **dưới hay trên** keyword tùy theo khoảng trống còn lại, và không bao giờ tràn khỏi màn hình. Màn hình hẹp hơn 480 px có thể chuyển sang dạng bottom sheet.
- Nội dung hộp thoại: tên keyword, tên tiếng Anh (nếu khác), định nghĩa ngắn khoảng 1–3 câu, và liên kết "Xem thêm" tới trang `/glossary/<slug>` hoặc tới bài liên quan.
- Phím `Esc` đóng hộp thoại. Mỗi lúc chỉ mở một hộp thoại.
- Mặc định chỉ đánh dấu **lần xuất hiện đầu tiên** của mỗi keyword trong bài.

## Dữ liệu (giai đoạn 1)

Model `Term` trong `api/src/models/term.ts`:

```ts
interface ITerm {
  slug: string;                 // 'smoke-test' — duy nhất theo language
  language: 'vi' | 'en';
  term: string;                 // dạng hiển thị chính: 'Smoke test'
  english?: string;             // tên gốc nếu `term` là tiếng Việt
  aliases: string[];            // các dạng khác cũng được dò: 'smoke testing', 'kiểm thử khói'
  shortDefinition: string;      // ≤ 300 ký tự, văn bản thuần — nội dung hộp thoại
  body?: string;                // Markdown dài cho trang /glossary/<slug>
  caseSensitive: boolean;       // true cho từ viết tắt dễ trùng: 'Go', 'CAP'
  autoLink: boolean;            // false = chỉ đánh dấu khi tác giả gắn thủ công
  categoryIds: ObjectId[];      // rỗng = áp dụng mọi chủ đề; có giá trị = chỉ dò trong bài thuộc các chủ đề này
  relatedArticleSlugs: string[];
  status: 'draft' | 'published';
}
```

Dữ liệu khởi tạo lấy từ `api/seed/terminology.json` (giai đoạn 0a). Các mục có `glossary: true` sẽ được seed thành `Term`.

API:

- `GET /api/terms?language=vi`: trả các `Term` đã xuất bản, chỉ gồm các trường cần cho việc dò keyword và hiển thị hộp thoại (không có `body`). Phía Next gọi qua `get()` trong `lib/server-api.ts`, nên dùng chung cơ chế cache sẵn có (`revalidate`, tag `api`).
- `GET /api/terms/:slug`: chi tiết một keyword, dùng cho trang `/glossary/<slug>`.
- Route admin: CRUD, cùng mẫu với `categories`.
- Bổ sung kiểu dữ liệu vào `packages/shared/src/types.ts` và cập nhật [`api-contract.md`](../api-contract.md).

## Dò keyword khi render (giai đoạn 2)

Viết plugin `rehypeGlossary` trong `packages/ui/src/glossary/`, chạy sau `rehypeRaw` và trước `rehypeHighlight` trong [`ArticleContent.tsx`](../../packages/ui/src/ArticleContent.tsx).

- **Đầu vào:** danh sách `Term` đã lọc theo ngôn ngữ và chủ đề của bài, truyền qua prop `glossary` của `ArticleContent`.
- **Matcher:** dựng một lần cho mỗi bộ term (memo theo phiên bản). Gom mọi `term` và `aliases`, sắp xếp theo độ dài giảm dần, escape, rồi ghép thành một regex có cờ `u` (và `i` cho các term không phân biệt hoa thường). Với vài nghìn term thì cách này đủ nhanh. Nếu đo thấy chậm thì chuyển sang Aho-Corasick.
- **Ranh giới từ:** không dùng `\b`, vì `\b` không hiểu chữ có dấu. Dùng `(?<![\p{L}\p{N}_])` ở đầu và `(?![\p{L}\p{N}_])` ở cuối.
- **Bỏ qua** text nằm trong: `pre`, `code`, `a`, `h1`–`h6`, `dfn`, `abbr`, `kbd`, `script`, `style`, `svg`, và mọi phần tử có `data-no-glossary`.
- **Chỉ lần đầu:** ghi nhớ các slug đã đánh dấu trong lúc duyệt cây.
- **Đầu ra:** `<dfn class="tkh-term" data-term="smoke-test" tabindex="0">smoke test</dfn>`. Giữ nguyên chữ gốc trong bài; chỉ bọc thêm thẻ.
- **Tác giả gắn thủ công:** cú pháp `<dfn data-term="cap-theorem">định lý CAP</dfn>` (HTML thô, đã được `rehypeRaw` hỗ trợ). Không cần plugin remark mới. Nếu cần tắt dò cho cả bài, thêm frontmatter `glossary: false` (đã có trong schema seed, model `Article` và form CMS).
- **Test** (vitest trong `packages/ui`): chữ có dấu, cụm chồng nhau ("CAP" và "CAP theorem"), phân biệt hoa thường, bỏ qua code và link, chỉ đánh dấu lần đầu, alias.

Trang đọc [`articles/[slug]/page.tsx`](../../apps/web/src/app/articles/[slug]/page.tsx) lấy danh sách term ở server (có cache) và truyền vào `ArticleContent`. Bản xem trước trong admin ([`ArticleEditorPage.tsx`](../../apps/admin/src/pages/ArticleEditorPage.tsx)) lấy danh sách qua React Query để tác giả thấy đúng kết quả đánh dấu.

Dữ liệu cho hộp thoại: trang bài chỉ gửi xuống client `{slug, term, english, shortDefinition, href}` của **những term thực sự xuất hiện** trong bài (plugin trả danh sách này qua `vfile.data`). Nhờ vậy hộp thoại mở ra ngay mà không cần gọi mạng.

## Giao diện hộp thoại (giai đoạn 3)

`apps/web/src/components/article/GlossaryPopover.tsx`, client component:

- Theo mẫu của [`CodeCopyOverlay.tsx`](../../apps/web/src/components/article/CodeCopyOverlay.tsx): lắng nghe sự kiện chung trên `.tkh-article-body` (`pointerover`, `pointerout`, `focusin`, `click`) và không sửa DOM do React render.
- Định vị bằng `@floating-ui/react-dom` với các middleware `offset(8)`, `flip()`, `shift({ padding: 16 })`, `size()` và `autoUpdate`. Không dùng CSS Anchor Positioning vì `browserslist` còn hỗ trợ Safari 16.4.
- Render qua portal vào `document.body`, có `role="tooltip"` và `id`. Keyword được gắn `aria-describedby` trỏ tới id đó khi hộp thoại mở.
- Style: thêm `.tkh-term` vào `reader.css` (gạch chân `dotted`, `text-underline-offset`, `cursor: help`). Hộp thoại dùng token màu sẵn có để tự hỗ trợ dark mode, rộng tối đa `min(360px, 100vw - 32px)`.
- Chuỗi giao diện ("Xem thêm", "Đóng") thêm vào `apps/web/src/i18n/vi.ts` và `en.ts`.
- Tôn trọng `prefers-reduced-motion`.

## Quản trị (giai đoạn 4)

- Trang `Terms` trong `apps/admin`: danh sách có tìm kiếm và lọc theo chủ đề, form sửa, xem trước hộp thoại.
- Nút "Bài chứa keyword này": gọi API đếm số bài khớp (dùng cùng matcher) để biên tập viên đánh giá nguy cơ bắt nhầm trước khi xuất bản.

## Mở rộng (giai đoạn 5, tùy chọn)

- Trang `/glossary` (danh sách A–Z) và `/glossary/[slug]`, có JSON-LD `DefinedTermSet` / `DefinedTerm` và được đưa vào `sitemap.ts`.
- Cho người đọc bật hoặc tắt việc đánh dấu keyword, lưu trong `localStore`.
- Ghi nhận keyword nào được mở nhiều nhất để ưu tiên viết định nghĩa dài.

## Rủi ro và cách giảm

| Rủi ro | Cách giảm |
|---|---|
| Bắt nhầm từ thông dụng ("Go", "class", "state") | `caseSensitive`, giới hạn bằng `categoryIds`, `autoLink: false`, kiểm tra số bài khớp trong admin |
| Đánh dấu quá nhiều làm khó đọc | Chỉ đánh dấu lần đầu. Không dò trong heading và link. Có thể giới hạn tối đa N keyword mỗi mục H2 |
| Render chậm | Matcher được dựng trước và memo. Trang bài vẫn dùng ISR như hiện tại |
| Sửa term nhưng trang bài vẫn hiện bản cũ | Trang bài đang dùng ISR `revalidate = 60` với tag `api` (`lib/server-api.ts`), nên thay đổi hiện ra trong khoảng 60 giây. Chỉ khi cần cập nhật tức thì mới phải thêm endpoint revalidate theo yêu cầu |
