# TEST-008 — Kiểm thử trong SDLC: Waterfall, Agile và DevOps

Bài: `/articles/kiem-thu-trong-sdlc-agile-waterfall`.

`phi-huy-acceptance.test.ts` minh hoạ cách tiếp cận test-first (ATDD/BDD): bảy tiêu chí chấp nhận AC-1…AC-7 của câu
chuyện «Khách huỷ chuyến» (SPEC R-CANCEL), viết dạng Given/When/Then, chạy trên hàm `customerCancellationFee` của lab.
Trong một pipeline CI, tệp này chạy ở bước «Unit tests» và ở lại làm regression test. Không sửa `src/`.

```bash
npx vitest run articles/kiem-thu-trong-sdlc-agile-waterfall --reporter=verbose
```

Kết quả mong đợi: `Tests  7 passed (7)`.
