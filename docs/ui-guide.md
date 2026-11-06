# Hướng dẫn giao diện — Tech Knowledge Hub

Bản tổng hợp từ 9 ảnh mẫu người dùng gửi (03/11/2026). Mỗi mục ghi rõ lấy ý tưởng từ ảnh nào và **đổi gì** cho khớp sản phẩm. Ảnh là nguồn cảm hứng, không chép pixel.

Ảnh tham chiếu:
- **A1** Tech Knowledge Hub — trang chủ (hero, ô chủ đề, bài nổi bật, lộ trình, cộng đồng/đăng ký tin).
- **A2** Tech Knowledge Hub — trang bài viết (breadcrumb, thanh reaction, mục lục trái, bài liên quan + tag + chia sẻ phải, khối "Bạn thấy bài viết này thế nào?", bình luận).
- **A3** DevPath — hero nền xanh đậm, chữ trắng cực lớn, 3 con số thống kê, sơ đồ lộ trình nét đứt bên phải.
- **A4** Fullstack Svelte (Newline) — bố cục học: menu chương thu/mở bên trái, nội dung giữa, "trên trang này" bên phải, khối mã nền tối, công tắc sáng/tối.
- **A5** Bình luận tối — tab Newest/Oldest/Top dạng viên thuốc, ô "What are your thoughts?", nút Comment, "Hide 1 replies", đường dọc nối phản hồi.
- **A6** Explore Learning Paths — lưới thẻ: nhãn chủ đề + nhãn cấp độ, tiêu đề lớn, mô tả, số chương, ngày, nút hành động rộng hết thẻ, ô tìm kiếm rộng.
- **A7** Reaction — 4 thẻ ngang: emoji lớn, tên tiếng Anh in đậm, nghĩa tiếng Việt nhỏ.
- **A8** Bình luận 2 cấp — `Bình luận (42)` + "Mới nhất trước", 👍 số + Reply, phản hồi thụt vào có **vạch dọc xanh**, ghi chú "Comment → Reply. Không hỗ trợ Reply tiếp vào Reply".
- **A9** Bảng màu (người dùng chốt).

## 1. Nền tảng thị giác (A9 — bắt buộc)

| Token | Giá trị | Dùng cho |
|---|---|---|
| `primary` | Blue `#2563EB` (Tailwind `blue-600`) | Nút chính, liên kết, mục đang chọn, vạch phản hồi |
| `secondary` | Indigo `#6366F1` (`indigo-500`) | Điểm nhấn phụ: nhãn "Intermediate", gradient hero (blue → indigo), biểu đồ thứ hai |
| `bg` sáng | `#F8FAFC` (`slate-50`) | Nền trang |
| `bg` tối | `#0F172A` (`slate-900`) | Nền trang chế độ tối; thẻ tối dùng `slate-800` `#1E293B` |
| Chữ | `slate-900` / tối `slate-100`; phụ `slate-500` / tối `slate-400` | |
| Viền | `slate-200` / tối `slate-700` | |
| Font | **Inter** (cài local `@fontsource/inter`, có tiếng Việt), mã: JetBrains Mono | |
| Bo góc | 8px (nút, ô nhập, nhãn) · 12px (thẻ, khối) | Không bo tròn hết cỡ trừ avatar và tab dạng viên thuốc |
| Theme | Sáng / tối, nút mặt trời–mặt trăng trên thanh đầu; mặc định theo hệ điều hành, nhớ lựa chọn | Tailwind `dark:` theo class `.dark` trên `<html>` |

Màu ngữ nghĩa cấp độ (giữ thống nhất mọi nơi): **Beginner** = emerald, **Intermediate** = indigo (secondary), **Advanced** = violet/purple. Màu chủ đề lấy từ dữ liệu danh mục (`category.color`).

Không dùng: tím làm màu chính như A6 (thay bằng primary xanh), cam như A5 (thay bằng primary), nền đen tuyền như A5/A7/A8 ở chế độ tối (dùng `slate-900`).

## 2. Khung chung

- **Thanh đầu** (A1 + A3): logo `</>` + "Tech Knowledge Hub" + dòng nhỏ "Learn • Share • Grow"; menu Trang chủ · Bài viết · Lộ trình học · Chủ đề (menu thả) · Giới thiệu; ô tìm kiếm có gợi ý phím tắt `Ctrl K`; nút chuyển ngôn ngữ **VI/EN**; nút sáng/tối; avatar (đã đăng nhập) hoặc nút **Đăng nhập** nền primary (A3). Dính trên cùng, nền trắng/`slate-900` mờ nhẹ, viền dưới.
- Trên điện thoại: menu gập vào nút ☰ (ngăn kéo trái); ô tìm kiếm thành biểu tượng.
- **Chân trang**: 4 cột (giới thiệu ngắn, chủ đề, tài nguyên, liên hệ) + dòng bản quyền.

## 3. Trang chủ `/` (A1 + A3)

Thứ tự khối:
1. **Hero** (A3 chủ đạo, A1 bổ sung): nền gradient xanh đậm → indigo đậm (`#0B1B4D → #1E3A8A → #312E81`, có vầng sáng mờ), chữ trắng cỡ rất lớn 2 dòng ("Hiểu sâu công nghệ. / Xây giải pháp tốt hơn."), đoạn mô tả, nút **Bắt đầu học →** (primary) + **Xem bài viết** (chữ trắng, viền mờ). Dưới là **3 con số** kiểu A3 có biểu tượng nét xanh: số chủ đề · số lộ trình · số thành viên (từ `GET /site/summary`). Bên phải: **sơ đồ lộ trình** kiểu A3 — các thẻ trắng nhỏ nối bằng đường nét đứt có mũi tên: Cơ bản → Database → Ngôn ngữ lập trình / Design Pattern → Architecture → Nâng cao (SVG tự vẽ, không ảnh). Ẩn sơ đồ khi màn hẹp.
2. **Ô chủ đề** (A1): hàng thẻ chủ đề — biểu tượng màu, tên, "N bài viết"; cuộn ngang trên điện thoại.
3. **Bài viết nổi bật** 🔥 (A1): 4 thẻ — ảnh bìa (hoặc khối gradient màu chủ đề + biểu tượng nếu không có ảnh), nhãn chủ đề + nhãn cấp độ, tiêu đề 2 dòng, ⏱ phút đọc · 👁 lượt xem, ❤ số reaction, nút lưu. "Xem tất cả →".
4. **Lộ trình học** (A1 + A6): 3 thẻ theo cấp độ (Beginner/Intermediate/Advanced): nhãn cấp độ, tên, mô tả, "N bài · Xh Ym", thanh tiến độ (đã đọc, lưu ở máy người đọc), "Xem tất cả lộ trình →".
5. **Bài mới nhất**: danh sách gọn 6 bài.
6. **Cộng đồng + Đăng ký tin** (A1): hai khối nền xanh đậm cạnh nhau; khối trái "Tham gia cộng đồng" + chồng avatar + "+N thành viên" + nút Đăng ký tài khoản; khối phải ô email + nút Đăng ký (`POST /subscribers`).

## 4. Danh sách bài viết `/articles` và chủ đề `/topics/:slug`

- Thanh lọc: chủ đề, cấp độ, tag, sắp xếp (Mới nhất / Xem nhiều / Thịnh hành), ô tìm kiếm rộng như A6.
- Lưới thẻ bài như mục 3.3; phân trang dưới.
- Trang chủ đề **lá**: banner chủ đề (biểu tượng, tên, mô tả, số bài), rồi bộ lọc + lưới bài lọc sẵn.
- Trang chủ đề **cha** (có chủ đề con): banner + lưới thẻ chủ đề con, KHÔNG hiện bài viết nào (không gọi API bài viết/thẻ; bài chỉ gắn vào chủ đề lá). Nếu không tải được cây chủ đề thì trang rơi về danh sách bài như trang lá.

### Banner chủ đề

- **Ảnh**: mỗi chủ đề một banner `apps/web/public/images/categories/<slug>.svg` + `.webp` (1600×400), vẽ bằng `apps/web/scripts/generate-category-banners.mjs` (hình minh hoạ vector gốc, không dùng logo/nhãn hiệu thật, không đóng chữ vào ảnh vì chữ được địa phương hoá ở HTML). Dựng lại: `npm run banners:generate -w @tkh/web`; kiểm: `npm run banners:check -w @tkh/web` (mọi slug trong `api/seed/categories.json` phải có đủ cặp tệp 1600×400; không có `api/` thì chỉ kiểm tệp ảnh). Thêm/đổi chủ đề trong seed ⇒ thêm minh hoạ + màu tương ứng trong script (màu trùng `color` của seed) rồi chạy lại.
- **Bố cục**: nửa trái (x < 720) để trống vì chữ phủ lên đó; toàn bộ hình nằm trong x ≥ 780 để còn nguyên khi thẻ chủ đề con (16:7, neo phải) cắt ảnh.
- **Hiển thị**: `CategoryBanner` (đầu trang) và `ChildCategoryCard` (thẻ chủ đề con) dùng chung nền CSS nhiều lớp `url(ảnh), linear-gradient(màu chủ đề → #0F172A)`, cover, neo phải-giữa. Chủ đề tạo từ CMS chưa có ảnh ⇒ ảnh 404, tự hiện gradient (không cần JS). Chữ trắng cố định ở cả hai theme, có lớp phủ tối (đặc hơn trên điện thoại) để đạt tương phản ≥ 4.5:1.
- **Triển khai**: ảnh nằm trong `apps/web/public` nên được đóng vào image `web` — thêm/đổi banner phải build lại và deploy lại image `web` (không đủ chỉ seed DB).

## 5. Lộ trình học `/learning-paths` (A6) và `/learning-paths/:slug` (A4)

- **Danh sách** (A6): tiêu đề "Khám phá lộ trình học" + mô tả, ô tìm kiếm rộng có biểu tượng kính lúp, lưới 4 cột (2 cột tablet, 1 cột điện thoại). Thẻ: nhãn chủ đề (chữ hoa, nền nhạt) trái + nhãn cấp độ phải, tiêu đề đậm lớn, mô tả, "📘 N bài" + ngày cập nhật, nút **Xem lộ trình** rộng hết thẻ (primary). Thẻ cùng hàng cao bằng nhau, nút dính đáy thẻ.
- **Chi tiết lộ trình**: đầu trang (cấp độ, tổng thời lượng, số bài) + danh sách bài đánh số theo thứ tự học, dấu ✓ bài đã đọc.

## 6. Trang bài viết `/articles/:slug` (A2 + A4 + A7 + A8)

Bố cục 3 cột (≥ 1280px), giữa rộng ~ 720–760px chữ:
- **Trái — điều hướng** (A4): nếu bài thuộc lộ trình: tên lộ trình + danh sách bài của lộ trình (bài hiện tại tô nền primary nhạt, chữ primary), các nhóm thu/mở có mũi tên `›`. Nếu không thuộc lộ trình: **Mục lục** của bài (A2) ở cột trái.
- **Giữa**: breadcrumb (🏠 › Bài viết › Chủ đề › bài), tiêu đề rất lớn, dòng tác giả (avatar, tên, ✓) · phút đọc · ngày; **thanh reaction nhỏ** dưới tiêu đề (A2: ❤ 892 👍 324 💡 121 🚀 87, nút Lưu, Chia sẻ); ảnh bìa; nội dung Markdown: H2/H3 rõ, đoạn rộng thoáng, **khối mã nền tối** kể cả ở chế độ sáng (A4) có nút Sao chép, `code` nội dòng nền xám nhạt, khối "Điểm nổi bật" có ✓ (A2), hình SVG trong khung viền bo 12px cuộn ngang được.
- **Phải** (A2 + A4): **"Trên trang này"** (mục lục bám cuộn, mục đang đọc tô primary — A4), rồi **Bài liên quan** (ảnh nhỏ + tiêu đề + phút đọc), **Tags**, **Chia sẻ** (sao chép liên kết).
- ≤ 1280px: bỏ cột trái, mục lục chuyển lên cột phải; ≤ 1024px: một cột, mục lục thành khối thu gọn đầu bài.
- Cuối bài — **"Bạn thấy bài viết này thế nào?"** (A7): **4 thẻ reaction** ngang bằng nhau: emoji lớn, tên tiếng Anh đậm (Love / Helpful / Insightful / Inspiring), nghĩa tiếng Việt nhỏ (Yêu thích / Hữu ích / Thú vị / Truyền cảm hứng) + **số đếm**. Đã chọn: viền + nền primary nhạt; bấm lần nữa để bỏ. Chưa đăng nhập ⇒ mời đăng nhập. Một người có thể chọn nhiều loại. Nhãn hiển thị theo ngôn ngữ đang chọn (EN: dòng phụ ẩn hoặc giữ tiếng Việt — giữ song song như A7).
- Điều hướng bài trước / bài sau trong lộ trình.

### Bình luận (A5 + A8)

- Đầu khối: **"Bình luận (42)"** trái (A8) + tab dạng viên thuốc **Mới nhất · Cũ nhất · Nổi bật** phải (A5, tab đang chọn nền primary chữ trắng).
- Ô soạn (A5): avatar + textarea "Bạn nghĩ gì về bài viết này?", nút **Bình luận** (primary, mờ khi trống); chưa đăng nhập ⇒ khối mời đăng nhập thay ô soạn.
- Mỗi bình luận (A8): avatar tròn, **tên đậm** · thời gian tương đối ("2 giờ trước"), nội dung, hàng hành động: **👍 số** (bấm để thích/bỏ) · **Trả lời**; chủ bình luận thấy thêm Sửa/Xoá trong menu `⋯`. Bình luận đã xoá: "Bình luận đã bị xoá" chữ nghiêng nhạt, vẫn giữ phản hồi bên dưới.
- **Phản hồi (cấp 2)**: thụt vào, **vạch dọc primary** bên trái (A8); khi > 2 phản hồi thì thu gọn, nút "Xem N phản hồi" / "Ẩn N phản hồi" chữ primary (A5).
- **Luật 2 cấp** (A8): Comment → Reply, **không lồng sâu hơn**. Bấm "Trả lời" trên một phản hồi ⇒ mở ô soạn ở cuối luồng của bình luận gốc, điền sẵn `@Tên` người được trả lời; phản hồi mới vẫn ở cấp 2 (API tự gắn vào bình luận gốc). Ghi chú nhỏ cuối khối như A8.
- Ngăn cách các luồng bằng đường kẻ mảnh.

## 7. Tài khoản

- `/login`, `/register`: thẻ giữa màn trên nền gradient nhạt của hero; email, mật khẩu, (tên hiển thị khi đăng ký); lỗi hiện dưới ô.
- `/me`: sửa tên hiển thị, avatar URL, giới thiệu; danh sách bình luận của tôi (đợt sau).

## 8. Trang quản trị (app riêng `apps/admin`)

Phong cách bảng điều khiển gọn, cùng token màu; khung: thanh bên trái (Tổng quan, Bài viết, Chủ đề, Lộ trình, Bình luận, Người dùng, Đăng ký tin) — mục đang chọn nền primary như A6 cột trái nhưng màu xanh; thanh đầu có tên người dùng, VI/EN, sáng/tối, đăng xuất.
- **Tổng quan**: 4–6 ô số (bài đã xuất bản, lượt xem, lượt truy cập, khách duy nhất, bình luận, thành viên) có so với kỳ trước; biểu đồ đường 30 ngày (lượt xem + truy cập + khách duy nhất); cột bình luận theo ngày; bảng "Bài xem nhiều".
- **Bài viết**: bảng (tiêu đề, chủ đề, cấp độ, trạng thái, lượt xem, bình luận, cập nhật) + lọc; trang soạn: hai cột — form trái (tiêu đề, slug tự sinh, tóm tắt, chủ đề, cấp độ, tag, ảnh bìa, lộ trình) + soạn Markdown với **xem trước trực tiếp dùng CHÍNH bộ hiển thị của trang đọc**; Lưu nháp / Xuất bản / Gỡ xuất bản.
- **Bình luận**: bảng có lọc trạng thái + bài; Ẩn/Hiện/Xoá; xem luồng.
- **Người dùng**: đổi vai trò, khoá/mở khoá (không tự hạ quyền mình).

## 9. Chữ trên giao diện (song ngữ)

Mọi chữ giao diện đi qua i18n (`vi` mặc định, `en`); nút **VI/EN** ở thanh đầu, nhớ trong `localStorage`. Nội dung bài viết giữ ngôn ngữ gốc của bài (trường `language`); tên chủ đề/mô tả chủ đề song ngữ từ API. Định dạng số theo locale (`24,5 N` / `24.5K`), thời gian tương đối theo locale.

## 10. Khả năng truy cập & chất lượng

Tương phản chữ ≥ 4.5:1 ở cả hai theme; focus ring primary rõ; mọi nút biểu tượng có `aria-label`; tôn trọng `prefers-reduced-motion`; ảnh có `alt`; không cuộn ngang trang trên điện thoại (bảng/khối mã/hình cuộn trong khung riêng).
