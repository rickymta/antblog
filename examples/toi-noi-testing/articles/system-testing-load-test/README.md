# SYS-08 — Load Test

[Bài đọc](../../../../docs/system-testing/08-load-test.md) · [Báo cáo](../../../../docs/system-testing/08-test-report.md).

Tại `examples/toi-noi-testing`, Node >=22.12; `npm ci` nếu cần:

```sh
npx tsx articles/system-testing-load-test/load.ts
npm run typecheck
```

Server/PGlite riêng trên cổng loopback ngẫu nhiên. Warm-up 20; lịch 10 RPS × 3s, 30 RPS × 8s, 10 RPS × 3s: 300 slots. Mỗi bước drain trước bước sau, không reset dữ liệu giữa các bước. Giới hạn 20 in-flight, lag >100ms thì drop, timeout 2s; dừng gửi request mới sau 10 lỗi và ghi các slot còn lại là failure-stop.

Expected bài tập: 300 sent, 0 drops/errors, mỗi bước p95<500ms, exit 0. Đây là ngưỡng giả định local, không phải SLO sản phẩm. File mới: `reports/load/sys08-load.json`; evidence lưu riêng: `load-observed.json`. Không nhận URL bên ngoài, không dùng DB của người dùng. Client/server/DB cùng process nên chưa dùng để xác định capacity production.
