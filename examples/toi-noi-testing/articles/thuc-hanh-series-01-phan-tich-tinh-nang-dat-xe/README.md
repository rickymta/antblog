# TEST-S01-LAB — Thực hành Series 01: phân tích tính năng đặt xe

Bài: `/articles/thuc-hanh-series-01-phan-tich-tinh-nang-dat-xe`.

Bài thực hành là phân tích kiểm thử (mục tiêu, đối tượng, phạm vi, rủi ro, test condition). Thư mục này chỉ chứa
các test **kiểm chứng sự thật** mà lời giải mẫu dựa vào, mỗi test mang mã test condition `TCD-xx` và mã yêu cầu `R-…`
của `SPEC.md`. Không sửa `src/`.

```bash
npx vitest run articles/thuc-hanh-series-01-phan-tich-tinh-nang-dat-xe
```

Kết quả mong đợi: `1 passed (1)`, `11 passed (11)`.

| Test | Yêu cầu | Kiểm |
|---|---|---|
| TCD-01 | R-STATE-02/06, R-PAY-03 | luồng chính đủ 7 trạng thái, thu VíXanh một lần |
| TCD-02 | R-QUOTE-01 | báo giá còn hạn ở 5 phút − 1 ms, hết hạn ở đúng 5 phút |
| TCD-03 | R-QUOTE-04, R-FARE-09 | giá chuyến là ảnh chụp báo giá (52.000 đ) dù cao điểm và mưa đã hết |
| TCD-04 | R-IDEM-02, R-QUOTE-03 | gửi lại cùng khoá ⇒ cùng chuyến; khoá mới cho cùng báo giá ⇒ 409 |
| TCD-05 | R-ACCEPT-03 | hai tài xế cùng nhận ⇒ một người thắng (kho bộ nhớ + PGlite) |
| TCD-06 | R-STATE-05 | 4 phút 59 giây chưa huỷ, đủ 5 phút ⇒ `no_driver` |
| TCD-07 | R-PAY-06, R-PAY-04 | ví từ chối ⇒ chuyến vẫn `completed`, `paymentStatus = failed`, không thử lại |
| TCD-08 | R-CANCEL-01/02 | huỷ lúc 1 phút 59 giây ⇒ 0 đ; đúng 2 phút ⇒ 10.000 đ |
| Bài tập 2 (TCD-11) | R-STATE-03, R-API-10 | bỏ bước ⇒ 409 `INVALID_TRANSITION`; tài xế lạ ⇒ 404 |
| Ngoài phạm vi | R-GEN-06, R-API-14 | trường «điểm đón» bị bỏ qua — lab chỉ nhận quãng đường |

Giới hạn: test «một người thắng» chỉ kiểm logic cập nhật có điều kiện khi hai lệnh xen kẽ ở mức `await`
(PGlite một kết nối — xem README của lab), không chứng minh hành vi dưới giao dịch song song thật.
