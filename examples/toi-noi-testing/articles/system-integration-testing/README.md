# TEST-015 · System Integration Testing

Bài: `/articles/system-integration-testing` — Series 02, Testing Knowledge Hub «Tới Nơi».

```bash
npx vitest run articles/system-integration-testing --reporter=verbose
```

| Ca | Điểm tích hợp | Kiểm gì |
|---|---|---|
| SIT-01 | IP-1 VíXanh | Tới Nơi gửi đúng số tiền nguyên đồng, đúng khách, khoá `trip:<id>:fare` (R-PAY-03) |
| SIT-02 | IP-1 VíXanh | Mất phản hồi ⇒ thử lại cùng khoá, ví chỉ trừ một lần (R-PAY-04, R-NFR-05) |
| SIT-03 | IP-1 VíXanh | Bị từ chối ⇒ không thử lại, chuyến vẫn `completed`, `paymentStatus = failed` (R-PAY-06) |
| SIT-04 | IP-4 thời tiết | Cờ «đang mưa» của dịch vụ thời tiết đi vào giá: +8.000 đ chỉ ở thành phố đang mưa (R-GEN-05, R-FARE-05) |
| SIT-05 | IP-1 VíXanh | Bộ kiểm giả định A-1 về đối tác (cùng khoá ⇒ cùng giao dịch) — chạy với bản GIẢ |

Kết quả mong đợi: 5/5 ca xanh. Lưu ý: đây là kiểm **phía Tới Nơi** của giao diện, dùng bản giả của lab.
System integration testing thật cần sandbox của VíXanh — hàm `kiemGiaDinhViXanh(gateway)` được viết để
chạy lại nguyên vẹn với một client sandbox (chưa có trong lab). Hết giờ được rút từ 3 giây xuống 50 ms
để test nhanh. Không sửa `src/`.
