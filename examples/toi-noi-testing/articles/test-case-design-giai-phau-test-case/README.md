# Giải phẫu một test case (TCD-02)

Thư mục của bài [Giải phẫu một test case](https://antfarms.cloud/articles/test-case-design-giai-phau-test-case), nhánh TCD · Thiết kế Test Case. Test basis: SPEC R-ACCEPT-01…07 (tài xế nhận chuyến).

| Tệp | Vai trò |
|---|---|
| `test-case-schema.ts` | Schema zod của một tài liệu test case: ID, tiêu đề, mục tiêu, ưu tiên, truy vết, phiên bản test basis, môi trường, tiền điều kiện, test data, bước, kết quả mong đợi, hậu điều kiện |
| `lint-test-case.ts` | Bộ lint 7 luật `TC-L01…07` cho lỗi viết hay gặp: kết quả mong đợi mơ hồ, không quan sát được, không tất định; thiếu tiền điều kiện; nhiều mục tiêu; phụ thuộc test case khác; truy vết tới mã không có trong SPEC |
| `tc-accept.ts` | 5 test case viết tốt `TC-ACCEPT-001…005` và 7 bản viết dở `TC-ACCEPT-901…907` (mỗi bản hỏng đúng một chỗ) |
| `test-case-lint.test.ts` | Bản tốt khớp schema và sạch lint; bản dở vẫn khớp schema nhưng bị lint bắt đúng luật |
| `tc-accept-execution.test.ts` | Chạy 5 test case tốt qua HTTP (`createApp` + supertest, kho trong bộ nhớ, `FixedClock`), test data đọc từ chính tài liệu test case |

## Lệnh chạy

```bash
npx vitest run articles/test-case-design-giai-phau-test-case
```

Kết quả mong đợi: 2 file, **25 test đạt** (20 test schema/lint + 5 test case R-ACCEPT chạy thật).

Không sửa `src/`.
