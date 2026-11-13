# SYS-13 — Báo cáo Data Integrity Test

Ngày chạy: 13/11/2026, Windows, Node.js 24.15.0, Vitest 3.2.7, PGlite 0.3.16. Database tạm, một kết nối; seed và FixedClock reset từng test. Test chạy trên mã trong cùng commit với báo cáo này.

## Lệnh và kết quả

Từ `examples/toi-noi-testing`:

```powershell
npm test -- articles/system-testing-data-integrity-test
npm run typecheck
```

- Vitest: **1 file, 6/6 test pass**, duration 2,54 giây; phần test 1,48 giây.
- TypeScript: **pass**, exit code 0.

| Case | Bằng chứng assertion đã đạt |
| --- | --- |
| DI-01 | Fare 35.000; đối soát không có sai lệch; requested rồi searching cùng mốc đồng hồ |
| DI-02 | 23505 khi dùng lại quote_id; chỉ còn chuyến ban đầu |
| DI-03 | 23503 ở event tham chiếu sai; chuyến mới/event đầu bị rollback; chuyến cũ và hai event còn nguyên |
| DI-04 | 23514 cho status lạ, 23502 cho fare NULL; dữ liệu chuyến không đổi |
| DI-05 | Discount 3.562,5 qua numeric/JSON/repository; fare 15.000; snapshot khớp |
| DI-06 | Trước inject: 0 sai lệch; sau tăng fare 1.000: đúng ID chuyến bị sửa |

## Diễn giải và giới hạn

Các assertion chứng minh bảo vệ schema đã chọn, atomicity trong `PgTripRepository.create()` và khả năng phát hiện sai lệch của query đã viết. DI-06 pass là bằng chứng phát hiện, không phải bằng chứng ngăn corruption: schema hiện chấp nhận cập nhật fare khác snapshot.

Không chạy concurrency nhiều kết nối, migration, crash/restore, đối soát ledger thật hoặc kiểm toàn bộ vòng đời event. Chưa chứng minh transaction bao trùm use case đặt chuyến và đánh dấu quote. Không chạy lại toàn bộ suite hay build website vì thay đổi chỉ thêm tài liệu và bộ test độc lập; typecheck kiểm mã TypeScript của lab.

Kết luận: đạt tiêu chí bài tập SYS-13; không suy ra production không có rủi ro mất hoặc sai lệch dữ liệu. Các bài tập nâng cao trong bài viết chưa được triển khai/chạy.
