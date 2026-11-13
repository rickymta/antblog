# SYS-13 — Data Integrity Test

[Bài học](../../../../docs/system-testing/13-data-integrity-test.md) · [Báo cáo](../../../../docs/system-testing/13-test-report.md)

Từ `examples/toi-noi-testing`:

```powershell
npm ci
npm test -- articles/system-testing-data-integrity-test
npm run typecheck
```

Sáu case dùng API + PGlite: đối soát snapshot/nhật ký; UNIQUE; rollback khi lỗi event thứ hai; CHECK/NOT NULL; numeric/JSON round-trip; phát hiện fare bị sửa trực tiếp. Mỗi case reset seed, suite đóng DB sau khi chạy. Không cần server hay database ngoài.

DI-06 cố ý làm hỏng dữ liệu trong DB test để kiểm query phát hiện. DI-03 chỉ chứng minh transaction của repository. Không kiểm crash durability, nhiều kết nối, migration hoặc ledger thanh toán thật. Không chạy các thao tác phá dữ liệu trên môi trường dùng chung.
