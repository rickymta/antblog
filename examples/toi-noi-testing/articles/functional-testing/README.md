# Functional Testing (TEST-017)

Thư mục của bài [Functional Testing](https://antblog.xyz/articles/functional-testing). Functional testing là một **test type**; ở đây cùng một loại test chạy ở hai test level (component và system qua API) và tách hai câu hỏi của ISO/IEC 25010: functional **correctness** và functional **completeness**.

| Tệp | Nội dung |
|---|---|
| `functional-correctness.test.ts` | FC-01…05 phụ phí bưu kiện R-PARCEL-03 (5,0 · 5,1 · 6,0 · 6,1 · 30 kg) ở mức component (`parcelSurcharge`) và qua `POST /api/quotes`; FC-06, FC-07 đầu vào sai; FC-08…10 tính nhất quán của bảng chi tiết cước |
| `functional-completeness.test.ts` | Bảng chi tiết R-FARE-10 có đủ 11 bước cho `ride`, `parcel`, `food`; danh sách bước đọc thẳng từ `SPEC.md` |
| `fare-fields.ts` | Ánh xạ tên bước trong SPEC ⇒ khoá JSON, hàm `missingFareFields` |
| `fare-ban-cu.json` | Bảng chi tiết GIẢ ĐỊNH thiếu 2 bước — chỉ để chứng minh bộ kiểm tra bắt được trường vắng |

## Lệnh chạy

```bash
npx vitest run articles/functional-testing
```

Kết quả mong đợi: 2 file, **20 test đạt** (15 correctness + 5 completeness).
