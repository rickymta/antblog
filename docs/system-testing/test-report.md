# Báo cáo khởi động SYS-01

Ngày: 13/11/2026. Môi trường: Windows, Node v24.15.0, npm 11.12.1; dependency lab cài theo package-lock bằng `npm ci --no-audit --no-fund`. Vitest v3.2.7. Chạy trên working tree có bài/test mới, chưa commit.

## Kết quả thực đo

| Phép kiểm | Lệnh / thư mục | Kết quả |
|---|---|---|
| Smoke mới + smoke hiện có | `npm test -- articles/system-testing-smoke-test tests/smoke` tại `examples/toi-noi-testing` | PASS: 4 file, 25 test; 4,60 giây toàn lượt |
| SYS-01 riêng trong lượt trên | `articles/system-testing-smoke-test/smoke.test.ts` | PASS: 3 case; 3,028 giây gồm khởi tạo/reset PGlite |
| TypeScript lab | `npm run typecheck` tại `examples/toi-noi-testing` | PASS, exit 0 |
| Build web + admin | `npm run build` tại root | PASS, exit 0; Vite cảnh báo một số chunk admin >500 kB |
| Whitespace diff | `git diff --check` tại root | PASS |

Lần thử trước khi cài dependency thất bại ở bước nạp config do thiếu Vitest local. `npx` thử dùng Vitest 5.1.0 ngoài lab nhưng không giải quyết được import. Đã cài dependency đúng lockfile rồi chạy lại bằng npm script của lab, kết quả trên dùng Vitest 3.2.7. Đây là lỗi chuẩn bị môi trường, chưa phải defect sản phẩm.

## Phạm vi kết luận

Đường health, chặn người dùng vô danh, báo giá → đặt → nhận → hoàn tất → đọc lại → kiểm thu tiền hoạt động trong lab API/PGlite với ví giả. Đủ điều kiện tiếp tục xây bài Sanity và Functional. Không có lỗi sản phẩm được phát hiện trong 25 ca này; không suy ra toàn hệ thống không có lỗi.

Chưa chạy toàn bộ regression của mọi bài, browser E2E, tải lớn, pentest, backup/restore hoặc deployment staging. Chưa thực hiện thí nghiệm inject lỗi đề xuất ở bài Smoke. Chưa xuất bản nội dung lên CMS. SYS-02…SYS-16 mới có đề cương, yêu cầu lab và nguồn khởi đầu trong roadmap.

## Chạy lại

```sh
cd examples/toi-noi-testing
npm ci --no-audit --no-fund
npm test -- articles/system-testing-smoke-test tests/smoke
npm run typecheck
```

Không cần DB ngoài; mỗi test có seed riêng qua reset, suite mới đóng PGlite sau khi chạy. Khi thay đổi SPEC hoặc dependency, ghi phiên bản và chạy lại trước khi dùng báo cáo để ra quyết định.
