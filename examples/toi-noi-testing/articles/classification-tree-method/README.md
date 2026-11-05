# TEST-036 · Classification Tree Method

Bài: `/articles/classification-tree-method`.

- `cay-phan-loai.ts` — cây phân loại của `POST /api/quotes` dưới dạng dữ liệu: 7 classification (Dịch vụ, Khối lượng — con của `parcel`, Loại xe, Thành phố, Mưa, Cao điểm, Voucher), ràng buộc loại trừ (`isInfeasible`), oracle mức loại kết quả theo thứ tự R-VALID-01 (`expectedOutcome`, gồm hai ràng buộc quy tắc R-FOOD-06 và R-PARCEL-02), kiểm each-choice coverage (`uncoveredClasses`, và bản tính hiệu ứng che lỗi `uncoveredEffective`), sinh bộ base choice (`baseChoice`), đổi tổ hợp thành test data (`toRequest`).
- `cay-phan-loai.test.ts` — 25 test: 4 test về mô hình (đếm 360/300 tổ hợp, each-choice, bộ «ngây thơ» bị che lỗi, base choice 12 dòng), 9 dòng each-choice E1–E9 chạy trên API (kiểm mã kết quả và tổng tiền tính tay), 12 dòng base choice B01–B12 chạy trên API (kiểm mã kết quả). Không sửa `src/`.

```bash
npx vitest run articles/classification-tree-method --reporter=verbose
```

Kết quả mong đợi: `25 passed`; in ra số tổ hợp `{ OK: 128, VOUCHER_NOT_APPLICABLE: 64, CITY_NOT_SUPPORTED: 54, VEHICLE_TOO_SMALL: 24, VALIDATION_ERROR: 30 }`, hai class bị che của bộ ngây thơ và danh sách B01–B12.
