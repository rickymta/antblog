# TEST-022 · Static Testing và Dynamic Testing

Bài: `/articles/static-testing-va-dynamic-testing`.

- `buggy-fare.ts` — bản chép ý của `src/domain/fare.ts` có 4 defect cố ý: D1 phụ phí mưa bị nhân 1,25 (trái R-FARE-03/05), D2 tính tiền bằng số thực (R-GEN-01), D3 chú thích sai + hệ số 1.25 viết cứng, D4 nhánh không bao giờ chạy tới. Không sửa `src/`.
- `cases.ts` — 6 ca: 4 ca «mỗi ca một điều kiện», ca tổ hợp R-FARE-09, ca ô tô 18,6 km cao điểm.
- `dynamic.test.ts` — dynamic testing: bộ test đơn điều kiện xanh trên bản lỗi; ca tổ hợp và ca ô tô đỏ (`it.fails` mặc định); phân tích failure bằng bảng chi tiết; đối chứng trên bản đúng.
- `d2-scan.test.ts` — quét 996 quãng đường hợp lệ: D2 lộ ở đúng 12 quãng (ô tô, chỉ cao điểm), 0 quãng với ô tô ngoài cao điểm và với xe máy (không mưa).
- `static-check.ts` + `static-check.test.ts` — static analysis tự viết, tối giản: đọc mã như văn bản, chỉ ra dòng 29 của `buggy-fare.ts` (quy tắc S1, S2) mà không thực thi hàm nào. Công cụ thật ở bài TEST-029.

```bash
npx vitest run articles/static-testing-va-dynamic-testing --reporter=verbose
SHOW_RED=1 npx vitest run articles/static-testing-va-dynamic-testing   # thấy 2 ca đỏ thật
```

Kết quả mong đợi: lệnh đầu `18 passed`; lệnh thứ hai `2 failed | 16 passed` (TC-05: 54000 thay vì 52000; TC-06: 288000 thay vì 287000).
