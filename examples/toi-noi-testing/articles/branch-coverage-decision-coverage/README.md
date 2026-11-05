# TEST-044 · Branch Coverage / Decision Coverage

Bài: `/articles/branch-coverage-decision-coverage`.

- `ca-huy.ts` — chín ca BC-01…BC-09 phí huỷ của khách viết thành bảng dữ liệu (kết quả mong đợi từ SPEC R-CANCEL-01…06), kèm hàm đăng ký test chạy được trên nhiều bản cài đặt.
- `phu-nhanh.test.ts` — chạy BC-01…BC-09 trên mã thật `src/domain/cancel.ts`.
- `cancel-v2.ts` — bản TÁI CẤU TRÚC CÓ LỖI (mặc định + ghi đè bằng `if / else if` không có `else`): trường hợp tài xế trễ trả lý do `charged` thay vì `driver_late`. Không sửa `src/`.
- `ban-v2.test.ts` — cùng bộ ca trên bản lỗi; BC-07 khai `it.fails` (đặt `SHOW_RED=1` để thấy failure thật).

Chạy từ gốc lab (bash):

```bash
COV="--coverage.enabled --coverage.reporter=text --coverage.reportsDirectory=reports/coverage-branch-coverage"
INC="--coverage.include=src/domain/cancel.ts --coverage.experimentalAstAwareRemapping"

# Bước 1 — bốn ca theo bảng R-CANCEL-01: 81,25% câu lệnh, 81,25% nhánh (13/16), chưa phủ dòng 31, 34, 43
npx vitest run articles/branch-coverage-decision-coverage/phu-nhanh.test.ts -t "BC-0[1-4]" $COV $INC
# Bước 2 — thêm BC-05…07 từ báo cáo nhánh: 100% câu lệnh, 100% nhánh (16/16)
npx vitest run articles/branch-coverage-decision-coverage/phu-nhanh.test.ts -t "BC-0[1-7]" $COV $INC
# Bản v2 — bộ ca chỉ đủ phủ câu lệnh: 100% câu lệnh (15/15), 93,75% nhánh (15/16), nhánh chưa đi ở dòng 27
npx vitest run articles/branch-coverage-decision-coverage/ban-v2.test.ts -t "BC-0[12356]" $COV \
  --coverage.include=articles/branch-coverage-decision-coverage/cancel-v2.ts --coverage.experimentalAstAwareRemapping
# BC-07 lộ defect của v2
SHOW_RED=1 npx vitest run articles/branch-coverage-decision-coverage/ban-v2.test.ts
# Toàn bộ bài (như npm test): 18 passed
npx vitest run articles/branch-coverage-decision-coverage
```

Thêm `--coverage.reporter=html` để xem báo cáo HTML trong `reports/coverage-branch-coverage/`.
