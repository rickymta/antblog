# TEST-030 · Shift-left Testing

Bài: `/articles/shift-left-testing`.

Minh hoạ shift-left trên story «Giới hạn 5 lần tạo chuyến / 60 giây» (SPEC R-API-13): tiêu chí chấp nhận chốt trong buổi refinement được viết thành test trước khi code (ATDD).

- `gioi-han-tao-chuyen.acceptance.test.ts` — 7 ca vitest + supertest ở mức API cho AC-1…AC-8 (AC-5 và AC-6 chung một ca), đồng hồ giả `FixedClock`. Chạy trên lab thật ⇒ đạt.
- `naive-rate-limit.ts` — bản cài đặt «hiểu theo bản nháp story» (phút đồng hồ cố định, đếm cả lần bị chặn, Retry-After làm tròn xuống). Không sửa `src/`.
- `ban-nhap-vs-ac.test.ts` — 4 ca: cùng AC-4 và AC-6 chạy trên bộ giới hạn thật (đạt) và bản nháp (`it.fails` — phải thất bại).

```bash
npx vitest run articles/shift-left-testing --reporter=verbose
```

Kết quả mong đợi: `Test Files 2 passed`, `Tests 11 passed` (2 ca `it.fails` được tính là đạt vì PHẢI thất bại).
