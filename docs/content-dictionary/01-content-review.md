# Giai đoạn 0 — Rà soát ngôn ngữ bài viết

## Mục tiêu

Đưa 606 bài tiếng Việt trong `api/seed/content` về cùng một chuẩn:

1. **Thuật ngữ đúng nghĩa và nhất quán** theo [bảng thuật ngữ](terminology.md). Không còn những từ dịch từng chữ làm người đọc không hiểu, ví dụ "kiểm sâu" thay cho in-depth testing.
2. **Câu chữ thuần Việt** theo [style guide](style-guide.md). Không lược mất danh từ, không theo trật tự từ của tiếng Anh, không tự ghép từ mới.

Phạm vi là ngôn ngữ, **không** sửa nội dung kỹ thuật. Nếu gặp lỗi kiến thức, ghi vào mục "Ghi chú kỹ thuật" của báo cáo đợt để xử lý riêng.

## Phân loại lỗi

| Mã | Loại | Ví dụ gốc | Đề xuất |
|---|---|---|---|
| **A** | Thuật ngữ bị dịch sai hoặc gượng | "bản build có đáng để **kiểm sâu**" | "bản build có đáng để **kiểm thử chuyên sâu**" |
| **B1** | Lược mất danh từ hoặc ngữ cảnh | "năm test đầu tiên cho **cước** Tới Nơi" | "năm test case đầu tiên cho **chức năng tính cước** của Tới Nơi" |
| **B2** | Cụm từ tự ghép, nghe cụt | "phụ phí **mưa**" | "phụ phí **trời mưa**" |
| **B3** | Trật tự từ hoặc cấu trúc câu kiểu tiếng Anh, giọng dịch máy | "Nó trả lời câu hỏi…", câu bị động "được … bởi …" dùng tràn lan | Viết lại câu theo cách người Việt nói |
| **C** | Một khái niệm được gọi bằng nhiều tên trong cùng bài hoặc cùng series | "ca kiểm thử" / "test case" / "case" lẫn lộn | Dùng thống nhất theo bảng thuật ngữ |

Mức độ:

- **Cao**: sai nghĩa hoặc người đọc không hiểu. Bắt buộc sửa.
- **Trung bình**: hiểu được nhưng gượng. Nên sửa.
- **Thấp**: có thể viết hay hơn. Chỉ sửa khi đang sửa câu đó vì lý do khác.

## Quy trình theo từng đợt

Mỗi đợt xử lý một nhóm chủ đề, khoảng 20–70 bài. Thứ tự đề xuất: **testing** (nơi phát hiện lỗi đầu tiên) → system-testing → ba → da → dotnet10/csharp → python → rust → postgresql → location → flutter/dart → aws → azure → các bài còn lại.

1. **Chạy `content:lint`** cho nhóm bài để lấy danh sách lỗi loại A dò được tự động (công cụ của giai đoạn 0b). Chạy `--fix` cho các rule chắc chắn, rồi đọc lại diff. Danh sách ứng viên trong [`survey-2026-11/`](survey-2026-11/) là điểm bắt đầu cho các nhóm đã khảo sát.
2. **Rà soát từng bài**, có AI hỗ trợ, theo checklist bên dưới. Mỗi bài cho ra một diff.
3. **Duyệt diff.** Người duyệt đọc diff, không đọc lại cả bài. Đây là chốt chặn chất lượng, không được bỏ qua.
4. **Kiểm tra tự động** trước khi commit:
   - `content:lint` không còn lỗi mức cao.
   - `npm test` trong `api` vẫn xanh (`seed-content.test.ts` kiểm tra frontmatter và cấu trúc).
   - Diff không đụng vào khối code, `slug`, `category`, `tags`, hay liên kết `/articles/...`.
5. **Commit theo nhóm**, ví dụ `content(testing): chuẩn hoá thuật ngữ và văn phong — đợt 1`.
6. **Bổ sung bảng thuật ngữ** nếu trong đợt gặp thuật ngữ mới. Một thay đổi trong bảng có thể ảnh hưởng tới các nhóm đã rà xong, nên `content:lint` sẽ bắt lại những chỗ đó.
7. **Đưa lên production**: `docker compose run --rm --build seed`. Đọc danh sách `keptCms` trong log và xử lý các bài đã sửa trong CMS (xem README).

## Checklist cho mỗi bài

- [ ] `title` và `summary` đọc lên tự nhiên. Đây là phần hiển thị nhiều nhất (thẻ bài, SEO, OG).
- [ ] Mọi thuật ngữ khớp với cột "dùng" của bảng thuật ngữ. Lần đầu xuất hiện có thể kèm giải nghĩa trong ngoặc.
- [ ] Không còn câu thiếu chủ ngữ hoặc danh từ khiến phải đoán ý.
- [ ] Một khái niệm chỉ có một tên trong cả bài.
- [ ] Không sửa: khối code, inline code, tên biến hay hàm, trích dẫn nguyên văn, số liệu, tên riêng.
- [ ] Không đổi ý kỹ thuật. Nếu nghi có lỗi kiến thức thì ghi chú lại, không tự sửa.
- [ ] Không thêm hay bớt đoạn văn, chỉ sửa câu chữ.

## Dùng AI hỗ trợ

Khối lượng khoảng 15 MB Markdown, quá lớn để rà hoàn toàn bằng tay. Cách làm:

- Mỗi tác vụ AI xử lý **một bài**, theo [mẫu prompt](review-prompt.md). AI **không** sửa bài, mà trả về danh sách đề xuất dạng `{old, new, reason, severity}`.
- Bước áp dụng do `npm run content:apply` thực hiện, với các chốt an toàn trong `api/seed/contentEdits.ts`:
  - `old` phải khớp nguyên văn và xuất hiện đúng một lần, trên một dòng.
  - Không được chạm vào khối code hay frontmatter (trừ `title`/`summary`).
  - Inline code, thẻ HTML, liên kết và URL phải giữ nguyên.
  - Sau khi sửa, kiểm tra lại toàn tệp: các khối code giữ nguyên, số dòng không đổi, frontmatter vẫn đọc được. Nếu không đạt thì huỷ toàn bộ thay đổi của tệp đó.
- Agent tự chạy `--dry-run` để sửa các đề xuất bị từ chối trước khi bàn giao.
- Model nhỏ (Haiku) dùng cho khảo sát và dò lỗi. Việc viết lại câu nên dùng model mạnh hơn, vì đó là phần cần cảm nhận ngôn ngữ.

## Đo tiến độ

- Số lỗi `content:lint` trên toàn bộ bài, theo từng nhóm. Mục tiêu: 0 lỗi mức cao.
- Số bài đã duyệt trên tổng số bài của mỗi nhóm, cập nhật vào bảng Trạng thái trong [README](README.md).
