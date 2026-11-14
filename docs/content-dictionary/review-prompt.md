# Mẫu prompt rà soát một bài (giai đoạn 0c)

Dùng cho agent AI rà soát **một bài mỗi lần**. Thay `<slug>` bằng tên tệp không có đuôi `.md`, và `<edits-dir>` bằng thư mục chứa tệp đề xuất (ngoài repo, ví dụ scratchpad). Model khuyến nghị: Sonnet trở lên. Haiku chỉ nên dùng để khảo sát.

Agent **không** sửa bài. Nó tạo tệp JSON đề xuất và tự kiểm tra bằng `content:apply --dry-run`. Người phụ trách đợt chạy bước áp dụng thật và duyệt diff.

---

```text
Bạn là biên tập viên tiếng Việt cho blog kỹ thuật antblog. Nhiệm vụ: rà soát ngôn ngữ MỘT bài viết và tạo
tệp đề xuất sửa. TUYỆT ĐỐI KHÔNG sửa trực tiếp tệp bài viết.

BÀI: api/seed/content/<slug>.md
TỆP ĐẦU RA: <edits-dir>/<slug>.json

ĐỌC TRƯỚC:
1. docs/content-dictionary/style-guide.md
2. docs/content-dictionary/01-content-review.md
3. api/seed/terminology.json ("use" là cách viết chuẩn)
4. (Nếu có) mục của bài trong docs/content-dictionary/survey-2026-11/*.md — gợi ý chưa duyệt, có thể sai
Rồi đọc TOÀN BỘ bài.

SỬA GÌ: thuật ngữ dịch sai/gượng; câu thiếu chủ ngữ, động từ hoặc danh từ; cụm từ tự ghép; trật tự từ kiểu
tiếng Anh; giọng dịch máy; một khái niệm nhiều tên (thống nhất "test case", "phiên bản"). Ưu tiên mức cao và
trung bình; mức thấp chỉ khi sửa rõ ràng tốt hơn.
KHÔNG SỬA: nội dung kỹ thuật, số liệu, tên riêng, trích dẫn nguyên văn, khối code, inline code, thẻ HTML,
liên kết, slug/category/tags. Không thêm/bớt ý. Chính tả dấu kiểu cũ: huỷ, khoá, tuỳ, hoà.

ĐỊNH DẠNG ĐẦU RA (JSON, UTF-8):
{"file":"<slug>.md","edits":[{"old":"…","new":"…","reason":"…","severity":"high|medium|low"}]}
- "old" chép NGUYÊN VĂN, gọn trên một dòng, xuất hiện ĐÚNG MỘT LẦN; giữ ngắn nhất có thể.
- Inline code, thẻ HTML, liên kết trong "old" phải giữ nguyên trong "new".
- Sửa được title/summary (giữ dấu nháy) và chữ trong SVG (<text>, aria-label).

KIỂM TRA BẮT BUỘC:
  cd api && npm run content:apply -- "<edits-dir>/<slug>.json" --dry-run
Sửa các đề xuất bị từ chối cho tới khi "từ chối 0". Luôn dùng --dry-run.

TRẢ VỀ NGẮN (≤ 120 từ): số đề xuất theo mức độ, 3 ví dụ (gốc → mới), chỗ nghi lỗi kỹ thuật (chỉ ghi).
```

---

## Sau khi các agent xong

```bash
cd api
npm run content:apply -- <edits-dir>/*.json     # áp thật; in đề xuất bị từ chối và lý do
git diff --word-diff seed/content               # NGƯỜI duyệt từng thay đổi, revert chỗ không ổn
npm run content:lint && npm test                # chốt chặn
```

Lưu các tệp đề xuất đã áp vào `docs/content-dictionary/edits/<đợt>/` để có lý do (`reason`) cho từng thay đổi khi cần tra lại. Các chỗ agent nghi sai kỹ thuật thì ghi vào [technical-notes.md](technical-notes.md).

## Bài học từ đợt thử (8 bài testing)

- Trung bình khoảng 17 đề xuất mỗi bài, không có đề xuất nào bị chốt an toàn từ chối. Người duyệt chỉnh lại 6 đề xuất và loại 1, tức khoảng 5%.
- Các lỗi người duyệt hay phải sửa lại:
  - Viết hoa sai khi cụm sửa nằm giữa câu.
  - Lặp từ ("đội kiểm thử tiếp tục kiểm thử…").
  - Dịch một thuật ngữ đã quen dùng bằng tiếng Anh, ví dụ "burst".
  - Sửa văn xuôi khiến lệch với chữ tương ứng trong khối code, ví dụ "ảnh chụp giá".
  - Thêm chữ không có trong bản gốc.
- Đợt 2 (66 bài, mỗi agent 6 bài, prompt có thêm mục "bài học"): 830 đề xuất, trung bình khoảng 12,6 đề xuất mỗi bài. Người duyệt chỉ loại 1 đề xuất. Script gắn cờ (viết hoa giữa câu, lặp từ, mất thuật ngữ cần giữ, đổi số…) gắn cờ 17 đề xuất, và cả 17 đều chấp nhận được sau khi xem ngữ cảnh. Với 830 đề xuất, người duyệt đọc các mục bị gắn cờ cộng một mẫu ngẫu nhiên khoảng 40 mục, không đọc hết.
- Đợt 3 (64 bài BA): 605 đề xuất, trung bình khoảng 9,5 đề xuất mỗi bài. Người duyệt chỉnh 2 và loại 1. Có một agent làm quá dè dặt (29 đề xuất cho 6 bài, và ghi câu mơ hồ là "nghi lỗi" thay vì sửa). Gửi thêm yêu cầu rà lại thì agent đó bổ sung được 10 đề xuất. Khi số đề xuất mỗi bài thấp hẳn so với mức khảo sát, nên yêu cầu agent rà thêm một lượt.
- Đợt 4 (60 bài AWS, prompt có thêm "mục tiêu tham khảo khoảng 9–13 đề xuất mỗi bài"): 757 đề xuất, trung bình khoảng 12,6 đề xuất mỗi bài, không còn agent nào quá dè dặt. Người duyệt chỉnh 1 đề xuất: "không nằm trong writer" bị agent đổi thành "không chỉ nằm trong writer", làm đổi nghĩa. Bài học: phải xem kỹ mọi đề xuất thêm hoặc bớt từ phủ định hay từ giới hạn ("không", "chỉ", "riêng", "mọi").
- Đợt 5 (58 bài Azure): 879 đề xuất, trung bình khoảng 15 đề xuất mỗi bài. Không phải loại đề xuất nào. Script gắn cờ có thêm kiểm tra **đổi số lần từ phủ định hoặc giới hạn** (không, chưa, chỉ, riêng, mọi, luôn…), gắn cờ khoảng 50 mục. Người duyệt đọc hết các mục này và đều thấy giữ đúng nghĩa.
- `content:apply` trước đây từ chối mọi chỗ sửa trong `aria-label`, `alt`, `title`, vì coi cả thẻ HTML là phần phải giữ nguyên. Kết quả là lint bắt được lỗi ở đó nhưng không sửa được. Công cụ giờ đã cho sửa giá trị của ba thuộc tính này, còn các thuộc tính khác vẫn bị khoá.
- Đợt 6 (52 bài Flutter): 729 đề xuất. Đợt này phát hiện và sửa thêm hai lỗ hổng của `content:apply`:
  1. Dòng HTML có `<code>…</code>` (callout, `<p>`, `<li>`, `<details>`) không sửa được. Nguyên nhân là chữ nằm giữa hai thẻ bị coi là vùng code. Giờ cả cụm `<code>…</code>` được bảo vệ như một inline code: phải giữ nguyên, nhưng được nằm trong đoạn sửa.
  2. Văn xuôi nằm giữa hai khối code làm cả tệp bị huỷ, vì regex backtick chạy trên cả tệp đã ghép nhầm hai dấu ``` lại với nhau. Giờ bước kiểm tra tính token theo từng dòng.
  Sau khi sửa công cụ, gửi thêm yêu cầu cho 7 agent để thêm lại khoảng 22 đề xuất đã phải bỏ. Người duyệt chỉnh 2 đề xuất: một chỗ agent tự thêm thông tin, một chỗ mất chữ hoa đầu câu.
- Cách sửa chuẩn "trượt" → "không đạt" và "im lặng" → "không báo gì" thêm chữ "không" nhưng giữ đúng nghĩa. Đây là nguồn chính của các cờ "đổi số lần không".
- Câu mơ hồ mà agent phải đoán nghĩa thì người duyệt cần xem kèm ngữ cảnh, và ghi vào technical-notes để tác giả xác nhận.
- Trên Windows, truyền **thư mục** cho `content:apply` thay vì liệt kê từng tệp, để tránh lỗi "command line is too long".
- Việc thống nhất "ca" → "test case" chiếm phần lớn đề xuất. Agent thường chỉ sửa ở văn xuôi chính và để nguyên trong bảng hay hình. Cách làm này chấp nhận được.
