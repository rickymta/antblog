# TCD-04 · Kết quả mong đợi và test oracle

Bài: `/articles/test-case-design-expected-result-test-oracle`.

Hai phần, không sửa `src/`:

1. **Oracle cước độc lập** (SPEC R-FARE-01…10, R-PARCEL-01…04, R-VOUCHER-06): tính bằng phân số BigInt, tham số chép từ SPEC, so **từng bước** của bảng chi tiết R-FARE-10 với hệ thống trên 16 dòng qua `POST /api/quotes` và 500 mẫu ngẫu nhiên có hạt giống; bẫy số thực; hai anti-pattern (chép kết quả từ hệ thống, assert quá lỏng/quá chặt).
2. **Khách huỷ chuyến** (R-API-11, R-CANCEL, R-STATE-06/07, R-ACCEPT-05, R-PAY-03…06, R-IDEM-02): kiểm mọi đầu ra quan sát được — phản hồi, trạng thái lưu, `trip_events`, trạng thái tài xế, lời gọi ví VíXanh giả và khoá idempotency.

| Tệp | Nội dung |
|---|---|
| `phan-so.ts` | Phân số BigInt: đọc số thập phân chính xác, cộng/trừ/nhân/so sánh, làm tròn trần và half-up |
| `oracle-cuoc.ts` | `oracleCuoc()` trả đủ 12 trường của bảng chi tiết; `soChiTiet()` liệt kê trường lệch |
| `bang-cuoc.ts` | 16 dòng OR-01…OR-16 kèm tổng tính tay |
| `buggy-fare.ts` | Bản chép `calculateFare` có defect: khi cao điểm, phụ phí mưa bị nhân 1,25 |
| `oracle-cuoc.test.ts` | 29 ca |
| `huy-chuyen.test.ts` | 9 ca (ví giả hết giờ 50 ms thay cho 3 giây để chạy nhanh; số lần thử lại giữ đúng 2) |

```bash
npx vitest run articles/test-case-design-expected-result-test-oracle --reporter=verbose
```

Kết quả mong đợi: `Test Files 2 passed (2)`, `Tests 38 passed (38)`.

- `oracle-cuoc.test.ts` (29): 5 ca kiểm chính oracle (R-FARE-09, R-FARE-02, R-PARCEL-03, R-FARE-08, 16 tổng tính tay), 16 ca so với hệ thống qua API, 3 ca mẫu ngẫu nhiên và số thực (oracle float sai ở 12/3.984 tổ hợp, đầu tiên là ô tô 18,6 km cao điểm: 288.000 thay vì 287.000 đ), 3 ca chép kết quả (bản lỗi tự «đạt» 14/14 dòng của chính nó; oracle bắt OR-01 54.000 ≠ 52.000; trên 500 mẫu bắt đúng 116 mẫu có cao điểm + mưa), 2 ca assert quá lỏng/quá chặt.
- `huy-chuyen.test.ts` (9): HUY-01…08 và HT-01.
