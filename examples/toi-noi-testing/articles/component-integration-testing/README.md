# TEST-013 · Component Integration Testing

Bài: `/articles/component-integration-testing`.

- `buggy-quote.ts` — năm bản «ghép lỗi» của `priceQuote` (`src/app/quote.ts`). Mọi component của `src/domain` giữ nguyên; mỗi bản sai đúng một mắt xích ghép nối: D1 truyền đồng vào hàm cần milli-đồng, D2 đưa số tiền đã làm tròn cho voucher (trái R-GEN-01), D3 không chuẩn hoá mã voucher (R-VOUCHER-02), D4 tính cao điểm theo giờ máy chủ UTC (R-GEN-02), D5 hỏi lượt dùng trước khi tính giá (R-VOUCHER-08). Chỉ hỗ trợ `ride`/`parcel`. Không sửa `src/`.
- `quote-integration.test.ts` — 19 ca vitest: 4 ca component (mỗi component đúng khi kiểm riêng), 5 ca integration IF-1…IF-5 trên `src/` thật (kho trong bộ nhớ, thời tiết giả, đồng hồ cố định), 5 ca `it.fails` chạy cùng IF-1…IF-5 trên bản ghép lỗi, 5 ca ghi lại failure quan sát được.

```bash
npx vitest run articles/component-integration-testing --reporter=verbose
```

Kết quả mong đợi: `19 passed` (ca `it.fails` được tính là đạt vì nó PHẢI thất bại).
