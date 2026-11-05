# TEST-024 — Walkthrough

Bài: `/articles/walkthrough`.

Bài chuẩn bị một buổi **walkthrough** luồng giao đồ ăn (SPEC R-FOOD-01…06) do BA dẫn, với 8 câu hỏi WT-Q1…Q8.
Walkthrough là static testing — không chạy gì. `cau-hoi-walkthrough.test.ts` là bước **theo sau** buổi họp:
mỗi câu hỏi mà SPEC đã trả lời được khoá lại thành một test (tên test = mã câu hỏi → mã yêu cầu), để câu trả lời
đã thống nhất không bị hiểu lại khác đi. Câu WT-Q8 (quán có được bù khi khách huỷ lúc món đã làm) SPEC chưa trả lời
⇒ chỉ test phần SPEC đã nói (phí huỷ 10.000 đ), phần còn lại là việc của PO. Không sửa `src/`.

```bash
npx vitest run articles/walkthrough --reporter=verbose
```

Kết quả mong đợi: `Tests  8 passed (8)`.
