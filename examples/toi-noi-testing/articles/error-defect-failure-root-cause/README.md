# TEST-004 · Error, Defect, Failure và Root Cause

Bài: `/articles/error-defect-failure-root-cause`.

- `buggy-cancel.ts` — bản chép của `src/domain/cancel.ts` có **một** defect cố ý (`<=` thay vì `<`, trái SPEC R-CANCEL-02). Không sửa `src/`.
- `chuoi-loi.test.ts` — 7 ca vitest: cùng một ca kiểm thử đạt trên bản đúng và đỏ trên bản lỗi (`it.fails`), failure chỉ xuất hiện ở đúng mốc 2:00, defect không được thực thi khi chuyến ở `arriving`, và một failure do môi trường (lệch đồng hồ giả định) dù mã đúng.

```bash
npx vitest run articles/error-defect-failure-root-cause --reporter=verbose
```

Kết quả mong đợi: `7 passed` (ca `it.fails` được tính là đạt vì nó PHẢI thất bại).
