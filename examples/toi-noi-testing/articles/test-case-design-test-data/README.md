# TCD-05 · Test data cho từng test case

Bài: [`/articles/test-case-design-test-data`](../../../../api/seed/content/test-case-design-test-data.md). Chạy từ `examples/toi-noi-testing`:

```bash
npx vitest run articles/test-case-design-test-data
```

Kết quả khi viết bài (vitest 3.2.7, Node 22): **5 tệp, 39 test case đạt**. Chạy lại với `TZ=UTC`, `TZ=America/Los_Angeles` và `TZ=Asia/Ho_Chi_Minh` đều 39/39.

| Tệp | Nội dung | Số test case |
|---|---|---|
| `test-data-builder.ts` | `aCustomer`, `aVoucher`, `aRideRequest`, `WorldBuilder` (`at`, `withCustomer`, `withVoucher`, `withRain`, `withVoucherTrip`, `build`) — dữ liệu giả lập, chuyến tiền điều kiện dựng qua use case thật | — |
| `doc-lap-thu-tu.test.ts` | Ba test case lượt dùng voucher chạy theo 6 thứ tự, hai cách dựng dữ liệu: fixture dùng chung xanh trọn vẹn 1/6 thứ tự (6/18 lượt chạy đỏ), thêm `reset()` trước mỗi ca thì xanh 0/6 (ca 003 thiếu tiền điều kiện); builder xanh 6/6 | 14 |
| `luot-dung-nua-dem.test.ts` | Bảng TD-01…08 quanh nửa đêm giờ VN và nửa đêm UTC (R-VOUCHER-07, R-GEN-02); bản lỗi «ngày UTC» chỉ bị TD-02 và TD-04 bắt; TD-09/10 kiểm lượt lần hai khi tạo chuyến qua nửa đêm | 11 |
| `het-han-thoi-tiet.test.ts` | Hạn báo giá đúng bằng `expiresAt` (R-QUOTE-01); thời tiết là dữ liệu môi trường (R-FARE-05); mưa sau báo giá không đổi giá (R-QUOTE-04) | 6 |
| `bai-tap.test.ts` | Lời giải Bài tập 1: biên ngày hiệu lực voucher 24/12–26/12/2025 theo giờ VN (R-VOUCHER-04) | 4 |
| `builder.test.ts` | Builder chỉ chứa dữ liệu khai báo, hai lần build tách biệt, object dùng chung bị sửa, tiền điều kiện trái nghiệp vụ không dựng được | 4 |

Giới hạn: kho trong bộ nhớ; không kiểm dữ liệu trên PGlite. Bản lỗi «ngày UTC» chỉ thay hàm khoảng ngày, không sửa `src/`.
