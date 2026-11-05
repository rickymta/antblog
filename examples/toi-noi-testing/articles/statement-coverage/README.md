# TEST-043 · Statement Coverage

Bài: `/articles/statement-coverage`.

- `bo-phu-cau-lenh.ts` — bốn ca TC-01…TC-04 thiết kế để phủ 100% câu lệnh của `src/domain/pricing.ts` (kết quả mong đợi lấy từ SPEC: R-FARE-09, R-VALID-01, R-PARCEL-03/04, R-FOOD-01/04/05) và ca biên dữ liệu TC-05 (20,04 kg — R-PARCEL-01/02). Viết thành hàm nhận một bản cài đặt để chạy trên cả mã thật lẫn bản lỗi.
- `phu-cau-lenh.test.ts` — chạy TC-01…TC-05 trên mã thật của lab.
- `pricing-loi.ts` — bản CÓ LỖI chép từ `src/domain/pricing.ts`, khác đúng một dòng (kiểm loại xe bằng khối lượng chưa làm tròn). Không sửa `src/`.
- `ban-loi.test.ts` — cùng bộ ca trên bản lỗi: TC-01…TC-04 xanh và vẫn phủ 100% câu lệnh; TC-05 khai `it.fails` (đặt `SHOW_RED=1` để thấy failure thật).

Chạy từ gốc lab (bash). `--coverage.experimentalAstAwareRemapping` cho vitest đếm câu lệnh theo cây cú pháp; bỏ cờ đó thì coverage-v8 đếm theo dòng (xem mục 6 của bài). Thư mục báo cáo riêng để không đụng `reports/coverage` của lệnh khác.

```bash
COV="--coverage.enabled --coverage.reporter=text --coverage.reportsDirectory=reports/coverage-statement-coverage"

# Bước 1 — chỉ TC-01: 50% câu lệnh (8/16)
npx vitest run articles/statement-coverage/phu-cau-lenh.test.ts -t "TC-01" $COV --coverage.include=src/domain/pricing.ts --coverage.experimentalAstAwareRemapping
# Bước 2 — TC-01…TC-04: 100% câu lệnh (16/16), 90,9% nhánh (10/11)
npx vitest run articles/statement-coverage/phu-cau-lenh.test.ts -t "TC-0[1-4]" $COV --coverage.include=src/domain/pricing.ts --coverage.experimentalAstAwareRemapping
# Bước 3 — cùng bốn ca trên bản lỗi: 4 passed, 100% câu lệnh của pricing-loi.ts
npx vitest run articles/statement-coverage/ban-loi.test.ts -t "TC-0[1-4]" $COV --coverage.include=articles/statement-coverage/pricing-loi.ts --coverage.experimentalAstAwareRemapping
# TC-05 lộ defect (failure thật)
SHOW_RED=1 npx vitest run articles/statement-coverage/ban-loi.test.ts
# Toàn bộ bài (như npm test): 10 passed
npx vitest run articles/statement-coverage
```

PowerShell: đặt `$env:SHOW_RED='1'` trước lệnh; thay `$COV` bằng các cờ viết đầy đủ.
