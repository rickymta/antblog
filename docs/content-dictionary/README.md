# Rà soát nội dung và Từ điển keyword — lộ trình triển khai

Tài liệu gốc cho hai việc liên quan chặt với nhau:

1. **Rà soát ngôn ngữ toàn bộ bài viết**: sửa thuật ngữ bị dịch sai nghĩa (ví dụ "kiểm sâu") và câu chữ chưa thuần Việt.
2. **Từ điển keyword**: khi người đọc rê chuột hoặc chạm vào một keyword trong bài, một hộp thoại nhỏ hiện ra ngay trên hoặc dưới keyword để giải thích nghĩa.

Hai việc dùng chung một nền: **bảng thuật ngữ chuẩn**. Bảng này quyết định thuật ngữ nào giữ nguyên tiếng Anh, thuật ngữ nào dịch và dịch ra sao. Bảng là chuẩn để rà soát bài viết, đồng thời là dữ liệu khởi tạo cho từ điển.

## Tài liệu

| Tệp | Nội dung |
|---|---|
| [`01-content-review.md`](01-content-review.md) | Quy trình và quy tắc rà soát nội dung (giai đoạn 0) |
| [`review-prompt.md`](review-prompt.md) | Mẫu prompt cho agent rà soát một bài, và các lệnh áp dụng/duyệt |
| [`technical-notes.md`](technical-notes.md) | Các chỗ nghi sai kỹ thuật phát hiện khi rà soát, chờ tác giả xử lý |
| [`edits/`](edits/) | Hồ sơ đề xuất đã áp theo từng đợt (kèm lý do của từng chỗ sửa) |
| [`style-guide.md`](style-guide.md) | Văn phong tiếng Việt cho bài kỹ thuật, kèm ví dụ sai → đúng |
| [`terminology.md`](terminology.md) | Cách dùng và quy tắc của bảng thuật ngữ; dữ liệu nằm ở `api/seed/terminology.json` |
| [`02-glossary-feature.md`](02-glossary-feature.md) | Đặc tả kỹ thuật tính năng từ điển keyword (giai đoạn 1–5) |
| [`survey-2026-11.md`](survey-2026-11.md) | Kết quả khảo sát chất lượng ngôn ngữ ban đầu |

## Thứ tự giai đoạn

Thứ tự này được chọn để không làm hai lần cùng một việc:

| # | Giai đoạn | Vì sao ở vị trí này | Kết quả bàn giao |
|---|---|---|---|
| **0a** | Khảo sát và chốt bảng thuật ngữ, style guide | Mọi lần sửa bài và mọi keyword đều dựa vào chuẩn này. Chốt sai thì phải sửa lại cả 606 bài. | `terminology.json`, `style-guide.md`, báo cáo khảo sát |
| **0b** | Công cụ kiểm tra tự động (`content:lint`) | Dò nhanh trên toàn bộ bài các thuật ngữ bị cấm hoặc không chuẩn. Dùng để đo tiến độ, và chặn lỗi tái phát ở bài mới. | `api/scripts/content-lint.ts` |
| **0c** | Rà soát và sửa bài theo từng đợt, theo nhóm chủ đề | Đây là phần tốn công nhất. Làm trước từ điển để keyword khớp với câu chữ đã chuẩn hoá. | Các commit sửa nội dung trong `api/seed/content`, mỗi đợt một nhóm |
| 1 | Model `Term` và API | Bảng thuật ngữ đã có, chỉ cần nạp vào DB | Model, route công khai và route admin, test |
| 2 | Plugin `rehypeGlossary` | Phần lõi: dò keyword khi render | Plugin trong `@tkh/ui`, kèm unit test |
| 3 | Giao diện popover | Cần dữ liệu từ giai đoạn 2 | `GlossaryPopover`, CSS, i18n |
| 4 | Trang quản trị keyword và bổ sung định nghĩa | Biên tập viên tự quản lý từ điển | Trang CRUD trong admin, seed định nghĩa |
| 5 | Mở rộng: trang `/glossary`, SEO, thống kê | Không chặn các giai đoạn khác | Tùy chọn |

Giai đoạn 0c và 1–2 có thể chạy song song **sau khi 0a đã chốt**, vì chúng không sửa cùng tệp.

## Trạng thái

| Giai đoạn | Trạng thái |
|---|---|
| 0a | Đã khảo sát xong ([báo cáo](survey-2026-11.md)). Đã chốt: chính tả kiểu huỷ/khoá, "test case", "phiên bản" |
| 0b | Đã có `npm run content:lint`, kèm `--fix` cho các rule chắc chắn |
| 0c | Đã sửa mọi lỗi mức cao lặp lại trên toàn site và chuẩn hoá chính tả huỷ/khoá. **Nhóm testing đã rà xong toàn bộ 74/74 bài**: đợt thử 8 bài, 137 chỗ sửa ([hồ sơ](edits/batch-01-testing-pilot/)); đợt 2 66 bài, 829 chỗ sửa ([hồ sơ](edits/batch-02-testing/)). **Nhóm business-analysis đã rà xong 64/64 bài**: 604 chỗ sửa ([hồ sơ](edits/batch-03-ba/)). **Nhóm devops-aws đã rà xong 60/60 bài**: 757 chỗ sửa ([hồ sơ](edits/batch-04-aws/)). **Nhóm devops-azure đã rà xong 58/58 bài**: 881 chỗ sửa ([hồ sơ](edits/batch-05-azure/)). Còn 350 bài thuộc các nhóm khác chưa rà. Chỗ nghi lỗi kỹ thuật ghi ở [technical-notes.md](technical-notes.md) |
| 1–5 | Chưa làm |

## Ràng buộc chung

- **Nguồn nội dung là `api/seed/content/*.md`.** Bài nào đã sửa trong CMS (`editedInCms: true`) thì seed sẽ **không** ghi đè. Sau khi sửa tệp, cần kiểm tra danh sách bài `keptCms` trong log seed và xử lý riêng từng bài: sửa tay trong CMS, hoặc chạy `--force-content` sau khi đã đối chiếu (xem [`deploy/README.md`](../../deploy/README.md)).
- **Không đổi `slug`.** Slug nằm trong URL, sitemap, liên kết nội bộ `/articles/<slug>` và `learning-paths.json`.
- Được phép sửa heading. Hiện không có liên kết nào trỏ tới `#anchor` của heading, và mục lục tự sinh lại từ nội dung.
- Không sửa nội dung trong khối code, inline code, tên riêng, hoặc trích dẫn nguyên văn từ tài liệu chuẩn (ISTQB, RFC…).
