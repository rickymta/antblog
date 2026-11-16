# Test analysis và test condition cho voucher (TCD-01)

Thư mục của bài [Test analysis: từ test basis tới test condition](https://antblog.xyz/articles/test-case-design-test-analysis-test-condition), nhánh TCD · Thiết kế Test Case. Test basis: SPEC R-VOUCHER-01…08, R-FARE-06, R-FARE-07, R-FOOD-05.

| Tệp | Vai trò |
|---|---|
| `test-conditions.ts` | Kết quả test analysis viết thành dữ liệu có kiểu: 23 test condition `TCOND-VOUCHER-01…23` (statement, basis, ưu tiên, rủi ro) và 5 câu hỏi gửi người viết đặc tả `Q-01…05` |
| `traceability.test.ts` | Kiểm chính kết quả phân tích: mọi R-VOUCHER có ≥ 1 condition (xuôi), mọi mã trích dẫn có trong SPEC (ngược), quy ước mã, phân bố ưu tiên, condition ưu tiên cao đều có bằng chứng chạy được, statement không chứa giá trị cụ thể |
| `voucher-conditions.test.ts` | Bằng chứng chạy được cho 22/23 condition qua use case thật (`createQuote`, `createTrip`, `cancelByCustomer`), kho trong bộ nhớ, `FixedClock`. TCOND-VOUCHER-23 bị chặn bởi Q-04 |

## Lệnh chạy

```bash
npx vitest run articles/test-case-design-test-analysis-test-condition
```

Kết quả mong đợi: 2 file, **32 test đạt** (8 test truy vết + 24 test bằng chứng).

Không sửa `src/`. Dữ liệu là voucher mẫu của SPEC Phụ lục A và khách `C001`.
