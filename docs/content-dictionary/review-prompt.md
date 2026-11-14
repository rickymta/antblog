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
- Đợt 7 (50 bài PostgreSQL): 771 đề xuất. Người duyệt chỉnh 2 đề xuất làm lệch mức độ khẳng định so với bản gốc: "là phần thưởng của bảng tĩnh" bị đổi thành "chỉ phát huy tác dụng với bảng tĩnh" (quá mạnh), và "Chỉ vì…" bị đổi thành "Nhờ…" (mất ý giới hạn). Từ "mỗi" theo nghĩa khẩu ngữ là "chỉ" ("đo mỗi đoạn replay"), nên khi gặp phải đọc ngữ cảnh trước khi đánh giá.
- Câu mơ hồ mà agent phải đoán nghĩa thì người duyệt cần xem kèm ngữ cảnh, và ghi vào technical-notes để tác giả xác nhận.
- Trên Windows, truyền **thư mục** cho `content:apply` thay vì liệt kê từng tệp, để tránh lỗi "command line is too long".
- Việc thống nhất "ca" → "test case" chiếm phần lớn đề xuất. Agent thường chỉ sửa ở văn xuôi chính và để nguyên trong bảng hay hình. Cách làm này chấp nhận được.
- Vòng 2 (248 bài: location-routing, rust, python, csharp, data-analysis, design-pattern và một bài system-design) chạy theo quy trình ba tầng:
  1. **Haiku** đọc từng lô, đối chiếu khối code, output và hình với văn xuôi, rồi lập danh sách cụm phải giữ (`lockedPhrases`), các nghi vấn kỹ thuật và gợi ý lỗi ngôn ngữ. Danh sách cụm phải giữ là để văn xuôi không bị sửa lệch với chuỗi in trong code hay output lab, lỗi mà vòng 1 người duyệt phải tự bắt.
  2. **Sonnet** soạn đề xuất `{old, new, reason, severity}` cho từng bài và tự chạy `--dry-run`.
  3. **Opus** duyệt **mọi** đề xuất (chia 20 phần), tự chỉnh hoặc bỏ đề xuất lệch nghĩa và kiểm chứng nghi vấn kỹ thuật. Ở vòng 1 người duyệt chỉ đọc mục bị gắn cờ cộng một mẫu; bài vòng 2 dài hơn và nhiều thuật ngữ riêng của từng loạt, nên đọc mẫu không đủ.
- Số liệu vòng 2: Sonnet soạn 6.524 đề xuất, trung bình khoảng 26 đề xuất mỗi bài vì bài dài. Opus chỉnh 217 và bỏ 27, tức khoảng 3,7%; sau khi áp, `content:apply` từ chối 0. Trong 175 nghi vấn kỹ thuật, Opus xác nhận 91 và loại 84 báo nhầm (thường vì Haiku chỉ đọc một câu mà bỏ qua ngữ cảnh của đoạn, hoặc không nhận ra số liệu là tổng của cả nhóm).
- Các lỗi lặp lại ở nhiều phần của vòng 2:
  - **Đổi lẻ một thuật ngữ đã dùng xuyên loạt**: "đường nóng" thành "hot path", "production" thành "môi trường thật", "engine" thành "công cụ", "tạo tác" (chuỗi in trong output lab) thành "đặc điểm nhân tạo", hoặc đổi tên một quy tắc do bài khác định nghĩa («không tăng giá lén», «lối rẽ»). Trước khi đổi một thuật ngữ phải grep cả loạt và các khối output.
  - **Mỗi agent dịch một khái niệm một kiểu**: implement thành "cài đặt", "triển khai" hay "hiện thực"; caller thành "người gọi", "bên gọi" hay "nơi gọi"; câu mở đầu dùng chung cả loạt mỗi bài sửa một khác. Nên chốt bản chuẩn trong prompt trước khi giao (ví dụ loạt Rust: implement là "cài đặt", vì "triển khai" đã dành cho deploy).
  - **Thay "Nó" bằng danh từ nhưng sai chủ thể hoặc mở rộng phạm vi**: gọi tham số `parse_float` là "hàm này", thêm "mỗi… đều", hoặc viết hoa danh từ mới khi nó đứng sau dấu hai chấm.
  - **Thêm hoặc bớt từ chỉ mức độ**: thêm "rất", "có thể", "thường", "nên" (đọc thành lời khuyên), hoặc bỏ "chỉ", "cùng", "tự động". Bài học của đợt 4 vẫn đúng: mọi đề xuất chạm vào từ phủ định hay từ giới hạn đều phải đọc kỹ.
  - **Lặp từ khi thay từ tiếng Anh**: "bộ test 16 test case", "kết quả" đặt cạnh "kết quả" sau khi đổi "output".
  - **Tự thêm ý kỹ thuật khi diễn giải**: "mã hoá để không lộ nội dung" cho base64, "đẩy bộ đệm" thành "ghi xuống đĩa", tự thêm "bước build cuối mất 30 giây".
  - **Thống nhất chưa trọn**: "cuộc đua", "nguồn sự thật", "pin" được sửa ở câu này nhưng còn sót ở câu khác cùng bài, trong hình hoặc ở bài khác của loạt. Khi đã chốt một thuật ngữ thì quét hết bài, kể cả bảng, `aria-label` và chữ trong SVG.
- Duyệt riêng từng phần không bắt được chỗ lệch giữa các phần. Lượt soát tích hợp sau vòng 2 ([hồ sơ](edits/batch-14-integration/)) áp thêm 55 chỗ sửa trên 37 bài: "Output thật" (và "Kết quả thật") còn sót ở 30 chỗ dù ba loạt đã chốt "Kết quả chạy thật", "cuộc đua", "lỗi đua" và "nguồn sự thật" còn ở 8 chỗ, "thứ bậc lớn" còn ở 6 chỗ dù python đã chốt "bậc độ lớn", "thời gian tường" còn ở 2 bài, loạt Rust còn 5 chỗ "hiện thực" bên cạnh "cài đặt", và 4 chữ "kiểm" đứng một mình do chính đề xuất vòng 2 đưa vào. Đợt sau nên giao cho người duyệt cuối bảng lựa chọn thuật ngữ của mọi phần (`_termChoices.tsv`) để grep ngay.
