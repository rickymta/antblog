# TCD-03 · Test case logic và test case cụ thể

Bài: `/articles/test-case-design-logic-va-cu-the`.

Đơn đồ ăn (SPEC R-FOOD-01…06, cùng R-FARE-04/05/07/08, R-VOUCHER-06/07, R-CANCEL-03): viết 12 test case **logic** thành dữ liệu, mở rộng bằng bảng test data thành 19 test case **cụ thể** có kết quả mong đợi tính từ SPEC, chạy qua `POST /api/quotes` (supertest, kho bộ nhớ, `FixedClock`, thời tiết giả). Thêm test procedure có setup/teardown, test suite, test execution schedule và minh hoạ phụ thuộc giữa test case. Không sửa `src/`.

| Tệp | Nội dung |
|---|---|
| `test-case-logic.ts` | 12 test case logic TC-FOOD-001…012 (tiền điều kiện, đầu vào, kết quả mong đợi ở mức logic, ràng buộc lớp dữ liệu) và bảng TD-01…TD-19 kèm cột tính tay |
| `mo-rong.ts` | `moRong()`: logic × dòng dữ liệu ⇒ test case cụ thể (đồng hồ, thời tiết, thân yêu cầu, kết quả mong đợi). Tham số giá chép từ SPEC, tính bằng số nguyên «phần tư đồng», không import công thức từ `src/` |
| `thu-tuc.ts` | Runner test procedure (teardown chạy cả khi bước lỗi, ghi nhật ký), TP-FOOD-01/02/03, bản phụ thuộc TP-FOOD-03a/03b, `lapLich()` (thứ tự topo + ưu tiên), `chayTheoLich()` (blocked khi phụ thuộc trượt) |
| `logic-va-cu-the.test.ts` | 34 ca vitest |

```bash
npx vitest run articles/test-case-design-logic-va-cu-the --reporter=verbose
```

Kết quả mong đợi: `Test Files 1 passed (1)`, `Tests 34 passed (34)` —
5 ca kiểm chính bộ test case (12 logic, 19 dòng, 19 cụ thể; công thức khớp cột tính tay; ví dụ 3,3 km; dòng trượt lớp bị chặn),
19 test case cụ thể qua API,
3 ca test procedure (TP-FOOD-01 đúng nhật ký 9 dòng, TP-FOOD-02 phí 10.000 đ, teardown vẫn chạy khi bước lỗi),
3 ca phụ thuộc (03a→03b cùng môi trường đạt; 03b chạy riêng trượt; TP-FOOD-03 độc lập luôn đạt),
4 ca suite/lịch (suite hợp lệ, regression 22 thành phần; lịch theo phụ thuộc trước ưu tiên sau; vòng phụ thuộc bị từ chối; smoke trượt ⇒ 5 mục blocked).
