# CRUD Testing (TEST-039)

Thư mục của bài [CRUD Testing](https://antblog.xyz/articles/crud-testing). CRUD testing theo CTAL-TA v4.0 mục 3.2.1 gồm hai phần: **completeness** (kiểm tĩnh trên ma trận CRUD) và **consistency** (kiểm động vòng đời từng thực thể). Thực thể chính: chuyến (`trips`) và các thực thể đi kèm.

| Tệp | Nội dung |
|---|---|
| `crud-matrix.ts` | Ma trận CRUD dựng từ SPEC: 10 chức năng (F01…F10) × 6 thực thể, 48 ô; hàm `missingOps` (completeness) và `matrixCells` |
| `crud-tracer.ts` | Bọc repository bằng Proxy để ghi thao tác C/R/U/D thật sự xảy ra theo từng chức năng (đo CRUD coverage); `failOnce` giả lập database lỗi |
| `crud-completeness.test.ts` | Completeness: thao tác nào thiếu ở từng thực thể (7 test) |
| `crud-consistency.test.ts` | Consistency qua API + PGlite: vòng đời L1…L6, ma trận vai trò × thao tác (42 ô), CRUD coverage (50 test, trong đó 1 `it.fails`) |
| `loi-giai-bai-tap-1.test.ts` | Lời giải bài tập 1: phủ nốt 4 ô R/D của khoá idempotency ở F08, F09 (5 test) |

## Lệnh chạy

```bash
npx vitest run articles/crud-testing
```

Kết quả mong đợi: 3 file, **62 test đạt**; dòng in ra `CRUD coverage = 44/48` (bộ chính, chưa tính lời giải bài tập).

`L6-kỳ vọng` là `it.fails`: nó khẳng định hành vi theo R-IDEM-05 (sau lỗi 5xx, thử lại cùng khoá phải đặt được chuyến) và hiện ĐANG sai trên lab — báo giá đã bị đánh dấu dùng trước khi ghi chuyến, không có giao dịch bao quanh (defect DR-039-01 trong bài). Khi lab sửa lỗi này, `it.fails` sẽ đỏ, nhắc đổi về `it`.
