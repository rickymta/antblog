# TEST-007 — Quy trình kiểm thử (Test Process)

Bài: `/articles/quy-trinh-kiem-thu-test-process`.

`nhan-chuyen.test.ts` là sản phẩm của hoạt động **test implementation** trong bài: bảy test case TC-ACC-01…07
cho tính năng «tài xế nhận chuyến» (SPEC R-ACCEPT), mỗi tên test mang mã test case và mã yêu cầu để log chạy test
đọc được như một bảng truy vết. Không sửa `src/`.

```bash
npx vitest run articles/quy-trinh-kiem-thu-test-process --reporter=verbose
```

Kết quả mong đợi: `Tests  7 passed (7)`. R-ACCEPT-06 (thứ tự danh sách chuyến chờ) cố ý chưa có test — bài dùng nó
làm ví dụ khoảng trống mà ma trận truy vết làm lộ ra.
