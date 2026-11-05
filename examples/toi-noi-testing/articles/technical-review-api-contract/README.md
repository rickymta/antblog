# TEST-025 — Technical Review: review API contract

Bài: `/articles/technical-review-api-contract`.

| Tệp | Là gì |
|---|---|
| `openapi-trips.v0.yaml` | bản nháp OpenAPI của `POST /api/trips` đưa vào technical review — **có lỗi gài sẵn** (mã TR-xx) |
| `pricing-quote.v1.proto` | hợp đồng gRPC Booking → Pricing đang chạy (mốc so sánh tương thích) |
| `pricing-quote.v2-draft.proto` | bản nháp v2 thêm giao đồ ăn — **có lỗi gài sẵn** (mã PQ-xx) |
| `bang-chung-review.test.ts` | bằng chứng cho phát hiện: đối chiếu hợp đồng với lab (bản cài đặt tham chiếu của SPEC) và minh hoạ lỗi float của PQ-04 |

Technical review là static testing — phát hiện tìm bằng cách đọc. Test chỉ dùng để kiểm nhanh «hợp đồng nói X,
hệ thống làm Y» trước khi chốt mức nghiêm trọng. Lược đồ zod trong test là hợp đồng **đã sửa** của phản hồi 201.
Không sửa `src/`. Không có thư viện protobuf trong lab ⇒ hai tệp `.proto` chỉ để đọc, không biên dịch.

```bash
npx vitest run articles/technical-review-api-contract --reporter=verbose
```

Kết quả mong đợi: `Tests  8 passed (8)`.
