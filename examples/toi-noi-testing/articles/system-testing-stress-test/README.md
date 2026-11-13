# SYS-09 — Stress Test

[Bài đọc](../../../../docs/system-testing/09-stress-test.md) · [Báo cáo](../../../../docs/system-testing/09-test-report.md).

Từ `examples/toi-noi-testing`, Node >=22.12; `npm ci` nếu thiếu dependency:

```sh
npx tsx articles/system-testing-stress-test/stress.ts
npm run typecheck
```

Server/PGlite riêng, loopback cổng ngẫu nhiên. Warm-up 20; các bước 30/s×3s, 150/s×4s, 600/s×4s, 30/s×3s: 3.180 slots. Cap 40 in-flight, lag>100ms thì drop, request timeout 2s, ngừng gửi slot mới sau 10 lỗi. Các bước drain; không reset DB giữa bước. Sau profile luôn thử một quote và health để quan sát, kể cả khi safety stop đã bỏ các slots cuối.

High-load drops hoặc p95 vượt 500ms được ghi như suy giảm, không tự làm exit nonzero. Baseline/return-low phải không drop/lỗi và p95<500ms; bất kỳ request error hoặc probe thất bại làm exit 1. Exit 0 không đồng nghĩa mọi slot tải cao đều đạt.

JSON sinh ở `reports/stress/sys09-stress.json` tương đối với thư mục chạy. Evidence đã lưu: `stress-observed.json`. Mức 30/s là normal giả định của lab, không phải telemetry production. Không nhận URL bên ngoài, không sửa src sản phẩm. Không dùng để suy ra capacity hoặc RTO/RPO.
