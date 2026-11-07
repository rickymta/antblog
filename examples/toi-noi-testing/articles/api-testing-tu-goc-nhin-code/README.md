# TEST-050 — API Testing từ góc nhìn code

Bài: `/articles/api-testing-tu-goc-nhin-code`.

Đọc handler `POST /api/trips` (chuỗi middleware trong `src/api/app.ts`, `src/api/idempotency.ts`, `createTrip` trong `src/app/trips.ts`), liệt kê nhánh N01…N21 và viết supertest phủ từng nhánh. Nhánh khó gây ra dùng điểm tiêm của `AppDeps` (không sửa `src/`):

- lỗi kho: bọc `customers`/`trips` bằng bản ném lỗi (N08, N20, N21);
- yêu cầu chồng nhau tất định: «cổng chờ» (`gate()`) giữ `quotes.findById` (N13 `IDEMPOTENCY_IN_PROGRESS`, N18 nhánh `markUsed = false`);
- thời gian: `FixedClock` (N07 giới hạn tần suất, N16 hết hạn báo giá).

```bash
npx vitest run articles/api-testing-tu-goc-nhin-code               # 22 passed (2 ca đánh dấu it.fails)
SHOW_RED=1 npx vitest run articles/api-testing-tu-goc-nhin-code    # 2 failed, 20 passed
npx vitest run articles/api-testing-tu-goc-nhin-code --coverage --coverage.include='src/api/**' --coverage.reporter=text
```

PowerShell: `$env:SHOW_RED='1'; …` (xong thì `Remove-Item Env:SHOW_RED`).

Hai ca `it.fails`:

- **N02b** — JSON hỏng và thiếu `x-user` ⇒ 400 `INVALID_JSON`, trong khi R-VALID-02 đặt xác thực (401) trước mọi kiểm tra định dạng: `express.json()` chạy trước `authenticate`. Lệch thứ tự cần chủ đặc tả quyết (sửa code hoặc ghi rõ ngoại lệ vào SPEC).
- **N21** — kho chuyến lỗi khi chèn ⇒ 500, khoá idempotency được nhả (R-IDEM-05) nhưng `quotes.markUsed` đã chạy NGOÀI giao dịch tạo chuyến ⇒ gửi lại cùng khoá nhận 409 `QUOTE_ALREADY_USED`, không có chuyến nào. Tái hiện trên PGlite thật bằng đơn 9 dòng giá trần (xem `articles/loop-testing`): 500 rồi 409.

Coverage `src/api` (chỉ bộ test của bài): branch 90,36% — nhánh còn thiếu thuộc endpoint khác, `testControl`, `logErrors`, `max = 0` của giới hạn tần suất và các nhánh mảng/khoá trùng của `canonicalJson` (bài TEST-049 phủ).
