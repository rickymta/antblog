# SYS-16 — Báo cáo Deployment Test

Ngày chạy: 13/11/2026, Windows 11, Node.js 22.18.0, Vitest 3.2.7, tsx 4.23.15, PGlite 0.3.16. Máy chủ lab chạy thành tiến trình riêng trên cổng trống; migration chạy trên PGlite tạm, một kết nối, mỗi test một database.

## Lệnh và kết quả

Từ `examples/toi-noi-testing`:

```powershell
npm test -- articles/system-testing-deployment-test
npm run typecheck
```

- Vitest: **2 file, 8/8 test pass**; lượt cuối duration 9,32 giây, phần test 12,10 giây (hai file chạy song song). `readyAfterMs` của DP-01 qua bốn lượt chạy: 1.710, 2.275, 1.789, 1.926 ms — chỉ là quan sát trên máy lab, không phải SLO khởi động.
- TypeScript: **pass**, exit code 0 (sau khi sửa ba lỗi kiểu `unknown` ở lần chạy đầu).
- Sau các lượt chạy không còn tiến trình `node … src/server.ts` nào (kiểm bằng `Get-CimInstance Win32_Process`).

| Case | Bằng chứng assertion đã đạt |
| --- | --- |
| DP-01 | Health 200 `{status:'ok'}` trong hạn chót 15 s |
| DP-02 | `POST /api/test/reset` → 404 `NOT_FOUND` với `NODE_ENV=production` |
| DP-03 | Log có `kho: pglite`, `API test: tắt`, giờ hệ thống 2026-11-04T02:00:xxZ |
| DP-04 | Báo giá 35.000; tạo 201; năm lệnh tài xế 200; đọc lại completed/paid/35.000/`fare-2026.1` |
| DP-05 | Với `TOINOI_TEST_API=1`: `/api/test/reset` khác 404 và log ghi `API test: BẬT` |
| MG-01 | Sau thêm cột nullable: bản cũ tạo, nhận chuyến, đọc lại được; giá trị cột mới của dòng cũ còn nguyên sau khi bản cũ UPDATE dòng đó; dòng mới có NULL |
| MG-02 | Sau thêm cột NOT NULL không default: tạo chuyến → 500 `INTERNAL_ERROR` với thông điệp chung, không có dòng trips nào |
| MG-03 | Sau đổi tên `cancel_reason`: tạo chuyến → 500; đọc chuyến cũ → **200 nhưng trường `cancelReason` biến mất** khỏi phản hồi |

## Quan sát đáng chú ý

MG-03 cho thấy lỗi rollback không phải lúc nào cũng hiện thành 500: đường đọc dùng `SELECT *` và ánh xạ theo tên cột nên vẫn trả 200, chỉ thiếu một trường. Monitor chỉ theo dõi tỷ lệ 5xx sẽ thấy ghi lỗi nhưng không thấy đọc sai. Đây là hành vi quan sát được của mã lab, ghi lại làm ví dụ; không sửa `src/`.

## Diễn giải và giới hạn

Bằng chứng cho thấy: máy chủ khởi động được từ mã nguồn với cấu hình giống triển khai, phép kiểm cấu hình phân biệt được bản bật và tắt API kiểm thử, post-deploy smoke đi hết đường tiền qua HTTP thật, và mã hiện tại chịu được migration expand nhưng không chịu được migration contract sớm.

Không chạy: build artifact/container, triển khai lên staging, rolling/blue-green/canary, secret manager, TLS, nhiều instance, migration trên PostgreSQL nhiều kết nối hoặc dữ liệu lớn, đo thời gian khoá bảng, graceful shutdown, rollback thật giữa hai phiên bản mã khác nhau. Lab không có endpoint version nên không kiểm build ID qua HTTP.

Khi chạy toàn bộ `npm test` của lab: 85/86 file pass; 3 test của `articles/data-based-test-techniques-domain-testing` (TEST-040) đỏ cả khi chạy riêng, theo cùng một kiểu mỗi lần. File đó và `src/` không bị thay đổi trong đợt này, nên đây là lỗi sẵn có cần điều tra riêng, không thuộc SYS-16. Cập nhật cùng ngày: nguyên nhân là `bang-10000.csv` được checkout thành CRLF (`core.autocrlf=true`) trong khi `loadTable()` tách dòng bằng `'\n'`, để lại `'bike\r'`. Đã sửa parser thành `/\r?\n/`; chạy lại toàn bộ lab: 86/86 file, 1058 test pass.

Kết luận: đạt tiêu chí bài tập SYS-16 trong lab một tiến trình; không suy ra quy trình triển khai production đã được kiểm.
