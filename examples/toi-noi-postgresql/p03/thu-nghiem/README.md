# P03 · Thí nghiệm của bài (PG-021 … PG-030)

Nơi người viết bài đặt các thí nghiệm, lời giải bài tập và phép thử phụ trích trong bài. Mỗi tệp chạy trên một **database mới chép từ nền** khai trong `manifest.json` (cùng nền với `p03/lab.json`: lược đồ P01 + P02, `p03/00`–`03`), nên tệp được tự do sửa dữ liệu, tạo/xoá index mà không ảnh hưởng tệp khác.

```sh
npm run p03:thu-nghiem                                            # mọi tệp trong manifest (dựng nền ~1,5 phút)
node pg/run-pg.mjs p03/thu-nghiem/manifest.json --only pg024      # chỉ tệp có tên chứa "pg024"
node pg/run-pg.mjs --nen p03/lab.json:p03 p03/thu-nghiem/moi.sql  # thử một tệp CHƯA khai trong manifest
```

`npm test` chạy mọi mục của manifest (test `test/p03.test.mjs`, skip khi máy không có PostgreSQL 18).

## Thêm một thí nghiệm

1. Tạo `pg0NN-<mo-ta>.sql` trong thư mục này. Đầu tệp đặt `SET search_path TO toinoi;` (và `bai_cu` nếu dùng bộ dữ liệu bài cũ) — mỗi tệp chạy trong phiên mới.
2. Ghi chỉ thị kiểm cho mọi lỗi cố ý và kết quả then chốt (`-- expect-*`, xem README gốc của lab). Kết quả phụ thuộc thống kê lấy mẫu (ước lượng `rows=`, `n_distinct`…) chỉ kiểm bằng `expect-range` hoặc qua hàm `uoc_luong()`.
3. Thêm một dòng vào `"tep"` của `manifest.json`: `{ "file": "pg0NN-….sql", "bai": "PG-0NN", "nen": "p03" }`.
4. Chạy `node pg/run-pg.mjs p03/thu-nghiem/manifest.json --only pg0NN` tới khi ĐẠT.

## Các tệp hiện có

| Tệp | Bài | Phiên | Minh hoạ | Số đáng trích |
|---|---|---|---|---|
| `pg021-bai-tap.sql` | PG-021 | 1 | Lời giải 3 bài tập: tài xế 20007 (BitmapAnd, kế hoạch biến động), EXPLAIN ANALYZE UPDATE trong giao dịch ROLLBACK, sai số giả định độc lập; EXPLAIN (không ANALYZE) câu INSERT không chèn gì | 2/139 chuyến; sai số 1,7 / 7,7 / 1,0 |
| `pg022-bai-tap.sql` | PG-022 | 1 | Số liệu page dùng trong bài (`lp_len` page 0, chỗ trống page 373) + 3 bài tập về page/buffer | 1.000 chuyến liên tiếp = 19 page heap, 24 buffer; sau VACUUM Index Only Scan 61 page |
| `pg023-btree-bai-tap.sql` | PG-023 | 1 | Page gốc và số page từng tầng, lỗi ép kiểu cột, LIKE có tiền tố cố định, index trên `status`, khách B1, tra một mã qua `uq_trips_code` | lần đầu 7 page, chạy lại 4 page |
| `pg024-cluster-hang-doi-khoa.sql` | PG-024 | A, B, C, D | A đọc trong giao dịch mở ⇒ B `CLUSTER` chờ ACCESS EXCLUSIVE ⇒ C chỉ `SELECT` cũng bị chặn (xếp hàng sau B); D đọc `pg_locks`; A `COMMIT` ⇒ B xong ⇒ C xong | `pg_locks`: lab-A AccessShareLock granted · lab-B AccessExclusiveLock **chờ** · lab-C AccessShareLock **chờ** |
| `pg024-ctid-hot-bai-tap.sql` | PG-024 | A, B | Lặp ba UPDATE ①②③, ctid cũ trả 0 dòng, `lp_flags` sau `VACUUM (INDEX_CLEANUP ON)`, dự đoán HOT, `\d` sau CLUSTER | — |
| `pg025-bai-tap-skip-scan-partial-unique.sql` | PG-025 | 1 | Skip scan khi cột đầu nhiều giá trị, `IN (2,3)`, sửa bẫy generic plan với partial index, partial UNIQUE «mỗi tài xế một chuyến đang chở», `date_trunc` không khớp expression index, cột chỉ có trong predicate partial chặn HOT | `Index Searches: 20`; HOT `1000 \| 500` |
| `pg026-heap-fetch-hot-partial-include.sql` | PG-026 | 1 | VM fork, HOT và Heap Fetches 18.536 = 9.365 + 9.171, index entry đánh dấu chết, `VACUUM VERBOSE` «index scan bypassed», partial + Index Only Scan, INCLUDE so với khoá 4 cột | 18.536 → 12.339; cả hai index 58.892.288 B |
| `pg026-snapshot-cu-chan-vacuum.sql` | PG-026 | A, B | A giữ snapshot REPEATABLE READ ⇒ B sửa 1.000 dòng, `VACUUM` không dọn được ⇒ Index Only Scan `Heap Fetches: 1997`; A vẫn thấy giá cũ; sau `COMMIT`, `VACUUM` thường **bỏ qua dọn index** (ít dòng chết) ⇒ vẫn `Heap Fetches: 1000`; `VACUUM (INDEX_CLEANUP ON)` ⇒ `0` | 1997 → 1000 → 0 |
| `pg027-cong-thuc-uoc-luong.sql` | PG-027 | 1 | Tự tính công thức ngoài MCV, điều kiện biểu thức + `CREATE STATISTICS ON (expr)`, ghi đè `n_distinct`, ngưỡng autoanalyze với thống kê cũ | 11,1 vs 11; 5.000 vs 89.965 ⇒ ≈1,0 |
| `pg028-gioi-han-tung-loai-index.sql` | PG-028 | 1 | GIN pending list, jsonb `->>` vs `@>`, B-tree không phục vụ `@>`/`&&` trên range, `PRIMARY KEY … WITHOUT OVERLAPS` (PG 18), BRIN `pages_per_range` + dữ liệu mới chưa tóm tắt, pg_trgm với `LC_CTYPE=C` | 23P01, 23514; lossy 512 → 1.212 → 512; similarity 1,00 vs 0,14 |
| `pg029-spill-bai-tap.sql` | PG-029 | 1 | EXPLAIN không ANALYZE không có Sort Method, `temp_file_limit` (53400), `pg_stat_statements.temp_blks_written`, Sort Method từng worker khi song song, index `(fare DESC, code)` | Index Only Scan 4 page |
| `pg030-toi-uu-bai-tap.sql` | PG-030 | 1 | Keyset so với OFFSET (md5 trùng), HOT tách từng đợt với fillfactor 50, `plan_cache_mode = auto` 5 custom + 2 generic | trang 200: 29 vs 5 page |

Định dạng nhiều phiên (`-- @A`, `expect-blocked`, cách runner phát hiện bị chặn): README gốc của lab, mục «Nhiều phiên».
