# P01 · Nền tảng và mô hình dữ liệu

Lab cho mười bài MS-001 … MS-010 trong [khung chương trình](../../../docs/mssql-master-curriculum.md). Sản phẩm của phần: **schema tối thiểu cho khách, tài xế, quán, chuyến và lịch sử trạng thái; fixture hợp lệ/sai; kiểm tra bất biến** — trên SQL Server 2022 thật. Cách cài và chạy chung ở [README của lab](../README.md).

```sh
npm run db:up && npm run p01     # 8 file, 80 PASS, kết quả như OUTPUT.txt
npm test                         # so khớp OUTPUT.txt, thử rollback, thí nghiệm MS-004
```

Trạng thái kiểm chứng: **SQL Server 2022 16.0.4325.4 (RTM-CU28) Developer Edition trên Linux**, Docker Desktop 29.9, Node 22.18.0, Windows 11 — 80 PASS, 0 FAIL.

## Các file

| File | Minh hoạ | Bài dùng | Kết quả mong đợi |
|---|---|---|---|
| `00-reset.sql` | Xoá/tạo database `ToiNoi` collation `Vietnamese_100_CI_AS`, schema `toinoi` và `lab`, thủ tục kiểm tra; đọc phiên bản | MS-001, MS-002 | `ToiNoi\|sa\|dbo\|dbo\|Vietnamese_100_CI_AS`, `16.0.4325.4\|CU28` |
| `01-schema.sql` | 6 bảng, ràng buộc và DEFAULT đặt tên, PK NONCLUSTERED + clustered `code` cho `trips`, extended property `MS_Description` | MS-003, MS-004, MS-005 | Bảng 13 cột của `trips`; 5 index, chỉ `uq_trips_code` là CLUSTERED |
| `02-fixture-valid.sql` | 3 thành phố, 6 khách, 6 tài xế, 3 quán, 15 chuyến; `N'…'` cho chữ có dấu | MS-005, MS-006, MS-007, MS-010 | 7 PASS; bảng 8 trạng thái × 3 loại |
| `03-fixture-invalid.sql` | 34 câu sai, mỗi câu vi phạm đúng một quy tắc (515, 2627, 547, 544, 2628, 220, 8115, 245, 241, 8169) + bẫy `DATEFORMAT` không báo lỗi | MS-004, MS-005, MS-010 | 37 PASS; dữ liệu vẫn 3/6/6/3/15 |
| `04-lifecycle.sql` | Chuyến mới qua 7 trạng thái bằng `UPDATE … OUTPUT … INTO` biến bảng + `INSERT` lịch sử trong một transaction; lỗi 333 khi OUTPUT INTO thẳng bảng có CHECK; UPDATE có điều kiện ra 0 dòng; DELETE bị FK chặn rồi `DELETE … OUTPUT` | MS-006 | Mã chuyến mới **100032** (17 giá trị IDENTITY bị tiêu ở 03), hoàn thành sau 31 phút, 7 dòng `trip_events` |
| `05-select.sql` | Khoảng nửa mở theo giờ VN, ngày UTC vs ngày lưu, tie-breaker, `TOP … WITH TIES`, `OFFSET/FETCH`, NULL khi sắp xếp, `NOT IN` gặp NULL, collation tiếng Việt và `_AI` | MS-007 | 3 PASS; 8 vs 9 chuyến ngày 02/11 |
| `06-migration.sql` | Expand → backfill → contract cho `drivers.vehicle_plate`: `WITH NOCHECK`, `is_not_trusted`, `WITH CHECK CHECK CONSTRAINT` thất bại vì dữ liệu cũ sai rồi sửa, `ALTER COLUMN … NOT NULL` | MS-009 | 8 PASS |
| `06-migration-rollback.sql` | Hoàn tác hai mức: nới lỏng NULL → sao lưu rồi gỡ cột; chạy lặp được | MS-009 | Không chạy trong `npm run p01`; test chạy 2 lần |
| `07-checks.sql` | 9 bất biến liên bảng/ràng buộc mong đợi 0, khoá ngoại thiếu index, `DBCC CHECKCONSTRAINTS`, kiểm kê | MS-008, MS-010 | 11 PASS; số dòng cuối 3/6/7/3/16/7 |
| `OUTPUT.txt` | Kết quả thật của `npm run p01` | Mọi bài | Test so khớp (bỏ dòng "Đã xoá database ToiNoi cũ") |
| `thu-nghiem/ms004-kieu-du-lieu.sql` | Thiếu `N'…'` làm mất chữ (ễ, ẵ → ?), UTF-8 collation, money/float/int, SUM tràn, datetime vs datetime2, `AT TIME ZONE`, NEWSEQUENTIALID | MS-004 | `npm run p01:thu-nghiem`, 3 PASS |

## Khác biệt có chủ đích so với lab PostgreSQL

Cùng nghiệp vụ, cùng dữ liệu, nhưng thiết kế theo SQL Server:

| Chủ đề | PostgreSQL (lab PG P01) | SQL Server (lab này) |
|---|---|---|
| Cách lưu bảng | Heap; mọi index là index phụ | Clustered index quyết định thứ tự lưu; `trips` clustered theo `code`, PK `id` nonclustered |
| Định danh | `uuid` + `gen_random_uuid()` | `uniqueidentifier` + `NEWSEQUENTIALID()` (danh mục) / `NEWID()` (chuyến) |
| Chuỗi | `text` | `nvarchar` cho chữ người đọc, `varchar` cho mã ASCII; collation tiếng Việt |
| Thời điểm | `timestamptz` (lưu UTC, hiển thị theo phiên) | `datetimeoffset(3)` (lưu kèm độ lệch, so sánh theo UTC) |
| CHECK dạng "A ⇔ B" | `(A) = (B)` | `(A AND B) OR (NOT A AND NOT B)` — T-SQL không có kiểu boolean |
| Định dạng chuỗi | Regex `~` | `LIKE` với lớp ký tự + `COLLATE Latin1_General_BIN2` |
| Lấy lại dòng vừa ghi | `RETURNING` | `OUTPUT inserted.* / deleted.*`; `OUTPUT INTO` cấm bảng đích có CHECK/FK |
| Ràng buộc chưa kiểm dữ liệu cũ | `NOT VALID` → `VALIDATE CONSTRAINT` | `WITH NOCHECK` (`is_not_trusted = 1`) → `WITH CHECK CHECK CONSTRAINT` |
| Chú thích | `COMMENT ON` | Extended property `MS_Description` |
| Index cho FK | Không tự tạo | Không tự tạo (K10 trong 07 liệt kê) |
