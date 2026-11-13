# SYS-07 — Performance Test

[Bài đọc](../../../../docs/system-testing/07-performance-test.md) · [Báo cáo](../../../../docs/system-testing/07-test-report.md).

Tại `examples/toi-noi-testing`, Node >=22.12, `npm ci` nếu thiếu dependency:

```sh
npx tsx articles/system-testing-performance-test/baseline.ts
npm run typecheck
```

Script tự mở server memory trên cổng loopback ngẫu nhiên, không nhận URL bên ngoài; chạy closed model một request tại một thời điểm. Ba vòng, mỗi vòng warm-up 50 request, reset dữ liệu rồi đo 300 request. Mỗi request timeout 2 giây và kiểm HTTP 200, tổng 35.000, quoteId dạng chuỗi. Đóng server trong finally.

Kết quả mới ở `reports/performance/sys07-baseline.json`, gồm raw samples. Bản lưu của lượt trong báo cáo: [baseline-observed.json](baseline-observed.json). Lần chạy mới không tự ghi đè bản evidence đã commit.

Gate minh họa được đặt trước đo: không lỗi và p95 <1.000 ms mỗi vòng; fail trả exit code 1. Đây không phải SLO production. Client và server cùng process/event loop; CPU/RSS đo chung. Không chạy script này song song với bài benchmark khác nếu muốn giảm nhiễu.
