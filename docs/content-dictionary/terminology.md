# Bảng thuật ngữ chuẩn

Dữ liệu nằm ở [`api/seed/terminology.json`](../../api/seed/terminology.json), một nguồn duy nhất dùng cho ba việc:

1. **Rà soát bài viết**: `npm run content:lint` (thư mục `api`) dò các cách viết cần tránh.
2. **Hướng dẫn tác giả** viết bài mới.
3. **Dữ liệu khởi tạo cho từ điển keyword**: các mục có `glossary: true` được seed thành `Term` ở giai đoạn 1.

## Quy tắc chọn: giữ tiếng Anh hay dịch

Theo thứ tự ưu tiên:

1. **Giữ nguyên tiếng Anh** khi thuật ngữ là tên chuẩn của ngành mà người làm nghề ở Việt Nam vẫn nói bằng tiếng Anh, hoặc là tên trong giáo trình chuẩn (ISTQB, AWS…). Ví dụ: smoke test, regression test, test case, commit, rollback, deploy, cache, token, idempotent. Đặt `keepEnglish: true`.
2. **Dùng từ Việt đã phổ biến** nếu có và không gây hiểu sai. Ví dụ: kiểm thử (testing), cơ sở dữ liệu (database), giao dịch (transaction), khoá chính (primary key).
3. **Lần đầu xuất hiện trong bài, có thể ghép cả hai**: "kiểm thử hồi quy (regression test)", sau đó dùng một dạng thống nhất.
4. **Không tự đặt từ Việt mới bằng cách dịch từng chữ.** Ví dụ không chấp nhận: "kiểm sâu", "biên kỳ", "hạ máy". Nếu chưa có từ Việt chuẩn thì giữ tiếng Anh.

## Định dạng `terminology.json`

```jsonc
{
  "version": 1,
  "terms": [
    {
      "id": "in-depth-testing",          // duy nhất, kebab-case; sẽ thành slug của Term
      "use": "kiểm thử chuyên sâu",      // cách viết chuẩn trong bài tiếng Việt
      "english": "in-depth testing",     // tên gốc (nếu khác `use`)
      "keepEnglish": false,              // true = `use` chính là tiếng Anh, không dịch
      "glossary": true,                  // đưa vào từ điển keyword
      "definition": "…",                 // định nghĩa ngắn ≤ 300 ký tự (cho hộp thoại)
      "note": "…",                       // lý do, hiện trong kết quả lint
      "avoid": [                         // các cách viết cần tránh
        { "text": "kiểm sâu", "severity": "high", "autofix": true },
        { "text": "khử trùng", "severity": "high", "notFollowedBy": "lặp", "autofix": true },
        { "text": "GO", "severity": "low", "caseSensitive": true }
      ]
    }
  ],
  "patterns": [                          // quy tắc văn phong dạng regex, không gắn với thuật ngữ
    { "id": "phu-phi-mua", "pattern": "phụ phí mưa", "flags": "i",
      "suggest": "phụ phí trời mưa", "replace": "phụ phí trời mưa",
      "severity": "medium", "note": "cụm từ cụt" }
  ]
}
```

- `autofix: true` (trong `avoid`) hoặc `replace` (trong `patterns`) cho phép `--fix` tự thay. **Chỉ bật khi phép thay đúng ở mọi ngữ cảnh**, và phải xem hết các chỗ khớp trước khi bật. Ví dụ "hạ máy" không bật được, vì "hạ" là động từ và "máy đã cập nhật" là tân ngữ, nên câu phải viết lại.
- `notFollowedBy`: bỏ qua chỗ khớp nếu ngay sau đó là chuỗi này, để tránh sửa thành "khử trùng lặp lặp".

- Mỗi `avoid.text` được dò như một **cụm từ trọn vẹn**, với ranh giới từ hiểu chữ có dấu. Khoảng trắng bất kỳ trong bài đều khớp với một dấu cách trong rule.
- Mặc định không phân biệt hoa thường. Đặt `caseSensitive: true` cho từ viết tắt dễ trùng với từ thường.
- `patterns` dùng regex JS với cờ `gu`. Thêm `i` qua trường `flags` nếu cần.
- Mức độ: `high` (sai nghĩa hoặc không hiểu được) · `medium` (gượng) · `low` (có thể tốt hơn).

## Dùng `content:lint`

```bash
cd api
npm run content:lint                                  # toàn bộ bài, lỗi high thì exit 1
npm run content:lint -- --summary --fail-on=none      # chỉ thống kê
npm run content:lint -- ^system-testing ^unit-test    # lọc tên tệp bằng regex
npm run content:lint -- --json > lint.json            # xuất JSON cho công cụ khác
npm run content:lint -- --fix                         # tự sửa các rule có autofix/replace, rồi báo phần còn lại
```

Sau khi chạy `--fix`, luôn đọc lại `git diff --word-diff` trước khi commit.

Lint chỉ quét chữ người đọc nhìn thấy: văn xuôi, `title`/`summary`, chữ trong hình SVG, và giá trị các thuộc tính `aria-label`/`alt`/`title`. Lint bỏ qua phần còn lại của frontmatter, khối code, inline code, `<pre>`/`<code>`/`<script>`/`<style>`, các thuộc tính HTML khác và đích liên kết.

## Quy trình thêm hoặc sửa mục

1. Đề xuất mục mới trong báo cáo đợt rà soát, kèm ví dụ ngữ cảnh.
2. Chạy `npm run content:lint -- --summary --fail-on=none` để xem số chỗ bị ảnh hưởng **trước khi** chốt. Một rule bắt quá nhiều chỗ thường là dấu hiệu rule quá rộng.
3. `npm test -- content-lint` phải xanh (test kiểm tra tệp JSON hợp lệ và không trùng `id`).
