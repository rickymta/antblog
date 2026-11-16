# P02 · T-SQL cho nghiệp vụ

Lab cho mười bài MS-011 … MS-020 trong [khung chương trình](../../../docs/mssql-master-curriculum.md). Sản phẩm của phần: **bộ truy vấn T-SQL có expected output, đối soát số dòng và tổng tiền**. Cách cài và chạy chung ở [README của lab](../README.md).

```sh
npm run db:up
npm run p02          # p01/00 + p01/01 rồi p02/00 → 20: 16 file, 331 PASS, kết quả như OUTPUT.txt (~1 phút)
npm run p02:output   # ghi lại p02/OUTPUT.txt sau khi sửa SQL
npm test             # gồm test/p02.test.mjs
```

Trạng thái kiểm chứng: **SQL Server 2022 16.0.4325.4 (RTM-CU28) Developer Edition trên Linux**, Docker Desktop 29.9, Windows 11 — 16 file, 331 PASS, 0 FAIL; chạy lại cho cùng `OUTPUT.txt`.

## Nền: lược đồ P01, dữ liệu 30 ngày giống hệt lab PostgreSQL

- P02 chạy trên **lược đồ** P01 (`p01/00-reset.sql` + `p01/01-schema.sql`), **không** nạp fixture 15 chuyến của P01 và không áp migration biển số của `p01/06`. Chạy `node run.mjs p02` một mình sẽ dừng ngay ở `00` với lỗi `Chưa có lược đồ P01 …`.
- `01-fixture.sql` sinh **tất định** bộ dữ liệu 30 ngày bằng cùng hàm băm và công thức với [lab PostgreSQL P02](../../toi-noi-postgresql/p02/README.md). Đã đối chiếu từng dòng của `trips`, `trip_events`, `trip_ratings`, `payments` (28.576 dòng) giữa hai engine: **giống hệt**. Để được như vậy, phút đặt chuyến được tính bằng hàm `lab.p02_phut`, tái hiện đúng cách PostgreSQL làm tròn phép chia `numeric` (khác biệt duy nhất nếu tính chính xác là chuyến 101126: 12:49 thay vì 12:50). Hình dạng dữ liệu, các điểm cài sẵn và đáp án 7 sai lệch đối soát xem README của lab PostgreSQL; số liệu trùng khớp.
- Mốc chụp **2026-10-31 01:00 giờ VN**. 3.174 chuyến (2.707 hoàn thành, 461 huỷ, 6 đang chạy), 20.685 sự kiện, 1.891 đánh giá, 2.826 giao dịch; doanh thu kỳ tháng 10 **182.287.000 đ**.
- Mỗi file 11–20 chạy được riêng sau `00 → 02` và **không đổi dữ liệu** (ghi trong `BEGIN TRAN … ROLLBACK`, hoặc tạo đối tượng riêng chạy lặp được). Riêng `18` dùng `DBCC CHECKIDENT … RESEED` để trả IDENTITY về 103173 sau khi rollback — chỉ làm trên lab.

## Mở rộng lược đồ (`00-mo-rong-schema.sql`)

| Đối tượng | Nội dung | Khác lab PostgreSQL |
|---|---|---|
| `toinoi.areas` | Cây khu vực: thành phố → vùng → khu; FK tự tham chiếu; CHECK gốc ⇔ không có cha ⇔ có `city_id`; `parent_id <> id` | CHECK "A ⇔ B" viết thành `(A AND B) OR (NOT A AND NOT B)` |
| `trips.pickup_area_id` | Khu đón, NULL = mất GPS | — |
| `toinoi.trip_ratings` | 0..1 dòng mỗi chuyến; `stars tinyint` 1–5 hoặc NULL; `tip` ≥ 0 hoặc NULL | `stars` là `tinyint` ⇒ `AVG(stars)` trả số nguyên (MS-011) |
| `toinoi.payments` | 0..n dòng mỗi chuyến; `provider_ref` duy nhất khi có | UNIQUE của SQL Server chỉ cho **một** NULL ⇒ dùng `CREATE UNIQUE INDEX … WHERE provider_ref IS NOT NULL` |
| `toinoi.menu_items`, `toinoi.menu_item_imports` | Tạo trong `17-upsert-merge.sql` | — |
| `lab.so_doi_soat` (view) | Sổ đối soát của MS-020, để lại sau khi chạy để mở trong DBStudio | SQL Server không có view tạm |

## Các file

| File | Bài | PASS | Minh hoạ chính |
|---|---|---|---|
| `00-mo-rong-schema.sql` | nền | 1 | Chặn chạy thiếu P01 bằng `THROW`; 4 đối tượng mới |
| `01-fixture.sql` | nền | 15 | Sinh dữ liệu tất định (`GENERATE_SERIES`, `CHOOSE`, scalar UDF), cài ca khó và 7 sai lệch |
| `02-kiem-tra-fixture.sql` | nền | 9 | Hồ sơ dữ liệu H1–H9 + bất biến K1–K9 (`GREATEST`, `CROSS APPLY`) |
| `11-null-case-isnull-coalesce.sql` | MS-011 | 42 | `= NULL` vs `IS NULL`, `ANSI_NULLS`, `CASE … WHEN NULL`, `ISNULL` lấy kiểu đối số đầu (cắt chuỗi) vs `COALESCE`, `AVG` số nguyên, `IS DISTINCT FROM`, chia 0 (8134) và `NULLIF` |
| `12-group-by-rollup.sql` | MS-012 | 37 | Mẫu số, lỗi 8120/207, WHERE vs HAVING, `ROLLUP`/`CUBE`/`GROUPING SETS`, `GROUPING()`, `STRING_AGG`, `SUM(int)` tràn 8115 |
| `13-join-apply.sql` | MS-013 | 39 | Số dòng trước/sau JOIN, tổng tiền bị nhân, gom trước rồi JOIN, anti-join, ON vs WHERE, `CROSS`/`OUTER APPLY` top-N mỗi nhóm |
| `14-subquery-exists-cte.sql` | MS-014 | 21 | Subquery vô hướng (512, 130), IN/EXISTS, bẫy `NOT IN` + NULL, CTE tính lại mỗi lần tham chiếu, `#temp` vs biến bảng |
| `15-window-function.sql` | MS-015 | 25 | ROWS vs RANGE (4194), frame mặc định với khoá trùng, mệnh đề `WINDOW`, lịch `GENERATE_SERIES`, `IGNORE NULLS`, compatibility level 150 vs 160 |
| `16-ngay-gio-mui-gio.sql` | MS-016 | 24 | Bảy cách lọc ngày 15/10, `datetime` .997/.999, `DATEDIFF` trên `datetimeoffset`, `DATE_BUCKET` mặc định theo UTC, `DATETRUNC`, `DATEFIRST`, giờ mùa hè New York |
| `17-upsert-merge.sql` | MS-017 | 29 | `MERGE` với khoá trùng (2627, 8672), UNIQUE ngoài `ON`, `$action`, `HOLDLOCK`, mẫu `UPDATE` rồi `INSERT … WHERE NOT EXISTS`, `NOT MATCHED BY SOURCE` |
| `18-phan-trang-keyset.sql` | MS-018 | 28 | Con trỏ một cột mất/lặp dòng, không có so sánh bộ (4145), keyset hai cột, chứng minh 18 mã/4 trang, OFFSET trôi |
| `19-cay-khu-vuc.sql` | MS-019 | 16 | Recursive CTE, độ sâu và đường đi, `MAXRECURSION` (530, 310), cộng dồn chuyến theo cây, `hierarchyid` |
| `19-chu-trinh.sql` | MS-019 | 9 | Cây có vòng vẫn qua mọi ràng buộc; phát hiện vòng bằng chuỗi đường đi; `XACT_STATE() = -1` sau lỗi 530 |
| `20-doi-soat-thanh-toan.sql` | MS-020 | 36 | View sổ đối soát, đáp án 7 chuyến (+268.000 đ), đối soát số dòng, ca khó không bị báo, ngày giao dịch vs ngày đặt, `FOR JSON PATH` |
| `OUTPUT.txt` | mọi bài | — | Kết quả thật của `npm run p02`; test so khớp (bỏ dòng "Đã xoá database ToiNoi cũ") |

Các dòng `Msg …` trong `OUTPUT.txt` là lỗi **mong đợi** được file lab in ra để minh hoạ, không phải lỗi bị bỏ sót (runner dừng file và báo FAIL khi có lỗi không bắt).
