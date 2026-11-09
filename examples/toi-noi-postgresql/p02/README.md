# P02 · SQL cho nghiệp vụ

Lab cho mười bài PG-011 … PG-020 trong [khung chương trình](../../../docs/postgresql-master-curriculum.md). Sản phẩm của phần: **bộ truy vấn có expected output, đối soát số dòng và tổng tiền**. Cách cài chung ở [README của lab](../README.md); tóm tắt:

```sh
cd examples/toi-noi-postgresql
npm ci
npm run p02          # p01/00 + p01/01 rồi p02/00 → 20 trên PGlite 17.5, kết quả như OUTPUT.txt
npm run p02:output   # ghi lại p02/OUTPUT.txt sau khi sửa SQL
npm test             # gồm 6 ca của test/p02.test.mjs
```

Trạng thái kiểm chứng: **chạy thật trên PGlite 0.3.16 (PostgreSQL 17.5)**, Node 22.18.0, Windows 11 — 266 câu lệnh, 7 lỗi mong đợi, 37 kiểm số dòng, 13 kiểm giá trị, 109 kiểm nội dung, 3 kiểm md5, 0 FAIL; cả bộ chạy ~3–5 giây. **Chưa chạy trên PostgreSQL server thật.**

## Nền: lược đồ P01, dữ liệu riêng của P02

- P02 chạy trên **lược đồ** P01 (`p01/00-reset.sql` + `p01/01-schema.sql`), **không** nạp fixture 15 chuyến của `p01/02-fixture-valid.sql` và **không** áp migration biển số của `p01/06` (P02 không cần cột `drivers.vehicle_plate`; đúng lược đồ hồ sơ). Lý do không giữ 15 chuyến P01: P02 cần lịch sử 30 ngày có mã chuyến tăng theo giờ đặt và một mốc chụp dữ liệu duy nhất; 15 chuyến P01 (02–03/11) sẽ nằm sau mốc đó và làm vỡ cả hai điều kiện. Bài P02 vẫn dùng đúng tên bảng/cột và quy ước của P01 (tiền là `integer` đồng, thời điểm `timestamptz`, ràng buộc đặt tên `pk_/fk_/uq_/ck_`).
- `npm run p02` = `node run.mjs p01/00-reset.sql p01/01-schema.sql p02`. Chạy `node run.mjs p02` một mình sẽ dừng ngay ở `00` với lỗi `P0001 Chưa có lược đồ P01 …` kèm gợi ý.
- Psql (cách B, chưa chạy thử): một phiên, `\i /lab/p01/00-reset.sql`, `\i /lab/p01/01-schema.sql`, rồi `\i /lab/p02/<file>` theo thứ tự bảng dưới. `docker-compose.yml` đã mount `./p02` vào `/lab/p02`. Các file 11–20 cố ý chứa câu lỗi (`-- expect-error`) — đừng bật `ON_ERROR_STOP` cho chúng; `19-chu-trinh.sql` có `BEGIN … ROLLBACK`.
- Mỗi file đặt `search_path` và `TimeZone` ở đầu; file nào cũng chạy được riêng sau `00 → 02` (17 tạo bảng mới nên chỉ chạy một lần mỗi phiên dữ liệu; 16 đổi `TimeZone` giữa chừng rồi đặt lại).

## Mở rộng lược đồ (`00-mo-rong-schema.sql`)

| Đối tượng | Nội dung | Bài |
|---|---|---|
| `areas` | Cây khu vực điều phối (hư cấu, không phải đơn vị hành chính): `id`, `parent_id` (FK tự tham chiếu), `code` (unique), `name`, `level` city/district/zone, `city_id` chỉ ở gốc. CHECK: gốc ⇔ không có cha ⇔ có `city_id`; `parent_id <> id` (chặn vòng 1 nút, **không** chặn được vòng dài hơn) | PG-019 |
| `trips.pickup_area_id` | Khu đón, FK → `areas`; NULL = mất GPS (thiếu dữ liệu) | PG-011, PG-019 |
| `trip_ratings` | 0..1 dòng mỗi chuyến (PK `trip_id`): `stars` 1–5 hoặc NULL (bỏ trống sao), `tip` ≥ 0 hoặc NULL (bỏ qua bước tip), `rated_at` | PG-011, PG-012, PG-013 |
| `payments` | 0..n dòng mỗi chuyến: `kind` charge/refund/adjustment, `method` cash/wallet (VíXanh)/card, `status` pending/succeeded/failed, `amount` đồng (charge/refund > 0, adjustment có dấu), `provider_ref` UNIQUE (NULL ⇔ tiền mặt), `paid_at`. Thực thu = Σ charge − Σ refund + Σ adjustment (chỉ `succeeded`) | PG-013, PG-014, PG-020 |
| `menu_items`, `menu_item_imports` | Tạo trong `17-upsert-on-conflict.sql` (chỉ bài đó dùng): đích có PK `(merchant_id, sku)` + UNIQUE `(merchant_id, name)`; bảng nhập không ràng buộc nghiệp vụ | PG-017 |

## Fixture (`01-fixture.sql`) — hình dạng dữ liệu

**Mốc chụp: 2026-10-31 01:00 giờ VN.** Chuyến đặt 01/10 00:00 → 30/10 23:59 (+07) đều đã kết thúc; 6 chuyến đặt 00:30–00:58 ngày 31/10 đang chạy (mỗi chuyến một trạng thái). Sinh **tất định** bằng hàm băm số học `p02_hash(n, salt)` (nhân Knuth, không `random()`); hàm và hai bảng tạm kế hoạch bị xoá cuối file. Mã chuyến ghi thẳng bằng `OVERRIDING SYSTEM VALUE`, tăng theo giờ đặt (100000 … 103173), sequence được `setval` lên 103173; `id` = `70000000-…-<mã>`. Giờ đặt của chuyến sinh tự động **làm tròn tới phút** (nhiều chuyến trùng giờ); chuyến đặc biệt ghi giờ chính xác. Cuối file chạy `ANALYZE` (xem "Lưu ý").

| Bảng | Dòng | Ghi chú |
|---|---|---|
| `cities` | 3 | SG, HN, DN (như P01) |
| `areas` | 34 | 3 thành phố → 10 vùng → 21 khu; vùng `HN-NA` chưa có khu |
| `customers` | 601 | 600 khách `0910000001…0910000600` (1–300 SG, 301–510 HN, 511–600 DN; 20 % chuyến dồn vào 10 % khách đi nhiều) + 1 khách doanh nghiệp `cb000000-…-000000000001` |
| `drivers` | 54 | SG 8101–8124, HN 8201–8220, DN 8301–8310. **8219, 8220 chưa chạy chuyến nào.** `busy`: 8101, 8102, 8201, 8301 (có chuyến đang chạy) |
| `merchants` | 25 | SG 101–112, HN 201–208, DN 301–305 (`e1000000-…-000000000NNN`) |
| `trips` | 3.174 | bảng dưới |
| `trip_events` | 20.685 | Đủ lịch sử từng chuyến: hoàn thành 7 dòng, huỷ trước nhận 3, huỷ sau nhận 5; dòng đầu = `requested_at`, dòng cuối = `updated_at` |
| `trip_ratings` | 1.891 | ~70 % chuyến hoàn thành; 153 `stars` NULL; 541 `tip` NULL; 660 `tip` = 0 |
| `payments` | 2.826 | bảng dưới |

Chuyến theo thành phố × dịch vụ (02 · H1):

| TP | Dịch vụ | Tổng | Hoàn thành | Huỷ trước nhận | Huỷ sau nhận | Đang chạy |
|---|---|---|---|---|---|---|
| DN | food | 210 | 173 | 27 | 10 | 0 |
| DN | parcel | 72 | 68 | 1 | 3 | 0 |
| DN | ride | 178 | 148 | 17 | 12 | 1 |
| HN | food | 505 | 413 | 63 | 27 | 2 |
| HN | parcel | 165 | 145 | 4 | 16 | 0 |
| HN | ride | 438 | 383 | 41 | 14 | 0 |
| SG | food | 717 | 615 | 70 | 32 | 0 |
| SG | parcel | 252 | 196 | 16 | 39 | 1 |
| SG | ride | 637 | 566 | 56 | 13 | 2 |

Theo trạng thái: requested/searching/accepted/arriving/picked_up/in_transit 1 mỗi loại, **completed 2.707, cancelled 461**. Kỳ báo cáo tháng 10 (đặt trong [01/10, 31/10) giờ VN) = 3.168 chuyến đã kết thúc; doanh thu (Σ cước completed) **182.287.000 đ**.

Các điểm cài sẵn:

| Điểm | Giá trị | Dùng ở |
|---|---|---|
| NULL có chủ đích | `driver_id` NULL 297 (295 huỷ trước nhận + 2 đang tìm), `merchant_id` NULL 1.742 (mọi chuyến không phải food), `pickup_area_id` NULL 36 (mất GPS), cước 0 đ 38 (31 chuyến hoàn thành) | PG-011 |
| Quanh nửa đêm | 383 chuyến đặt 00:00–06:59 giờ VN (= ngày hôm trước theo UTC); theo giờ: 0h 68 · 1h 28 · 2h 39 · 3h 39 · 4h 25 · 5h 61 · 6h 123 | PG-016 |
| Năm chuyến biên ngày 15/10 | 101489 `00:00:00`, 101502 `06:59:59`, 101503 `07:00:00`, 101594 `23:59:59.5`, 101595 `16/10 00:00:00` (giờ VN) | PG-016 |
| Ngày trống | 20/10 Đà Nẵng có bão: 0 chuyến | PG-011, PG-015, PG-016 |
| Trùng giờ đặt | 148 nhóm, 305 chuyến; lớn nhất là 2 lô API của khách doanh nghiệp: 10/10 09:00:00 × 6 (100971–100976), 18/10 14:30:00 × 4 (101882–101885); 3 nhóm 3 chuyến lúc 13:12 | PG-015, PG-018 |
| Khách doanh nghiệp | 18 chuyến bưu kiện SG, 5/trang ⇒ cả hai lô vắt qua ranh giới trang | PG-018 |
| Giao dịch mỗi chuyến | 0 dòng: 485 chuyến · 1 dòng: 2.552 · 2 dòng: 137 | PG-013 |

Giao dịch theo loại × phương thức × trạng thái (02 · H8): adjustment/wallet/succeeded 2 (Σ −5.000) · charge/card failed 9, pending 1, succeeded 698 · charge/cash succeeded 1.054 · charge/wallet failed 87, succeeded 962 · refund/card succeeded 12 · refund/wallet succeeded 1.

**Ca khó nhưng ĐÚNG** (đối soát không được báo): 100042 thẻ lỗi rồi thu lại · 100048 trả một phần (ví 104.000 + thẻ 15.000) · 12 chuyến huỷ đã trừ thẻ rồi hoàn đủ (vd 100366) · 31 chuyến cước 0 đ không có giao dịch · **102235** ví trừ 82.000, cước sửa còn 72.000, adjustment −10.000 · **103165** tiền mặt thu thiếu 23.000/28.000, adjustment +5.000 ghi lúc 00:05 ngày 31/10 · **103162** một charge `pending` thừa bên cạnh lần thu thành công.

**Sai lệch cài sẵn — đáp án PG-020 (đúng 7 chuyến, tổng chênh +268.000 đ):**

| Mã | Ngày | TP | Dịch vụ | Trạng thái | Phải thu | Thực thu | Chênh | Loại |
|---|---|---|---|---|---|---|---|---|
| 100724 | 07/10 | HN | ride | completed | 76.000 | 0 | −76.000 | 3 THIẾU THANH TOÁN (không có giao dịch nào) |
| 101084 | 11/10 | DN | food | completed | 98.000 | 0 | −98.000 | 3 THIẾU THANH TOÁN (chỉ có charge thẻ `failed`) |
| 101310 | 13/10 | SG | food | completed | 85.000 | 170.000 | +85.000 | 4 THU TRÙNG (ví trừ 2 lần, cách 3 giây) |
| 101987 | 19/10 | SG | ride | cancelled | 0 | −72.000 | −72.000 | 1 HOÀN KHÔNG CÓ THU |
| 102443 | 24/10 | HN | parcel | completed | 54.000 | 45.000 | −9.000 | 5 LỆCH SỐ TIỀN (đảo chữ số) |
| 102642 | 25/10 | SG | ride | completed | 38.000 | 380.000 | +342.000 | 5 LỆCH SỐ TIỀN (thừa một số 0) |
| 102739 | 26/10 | SG | food | cancelled | 0 | 96.000 | +96.000 | 2 THU CHUYẾN HUỶ |

## Các file

Đánh số theo ID bài: `11-…` = PG-011 … `20-…` = PG-020; `00`–`02` là nền. Mọi câu trả bảng kết quả đều có ít nhất một kỳ vọng (test kiểm).

| File | Bài (slug) | Minh hoạ | Số đáng trích (đều có trong `OUTPUT.txt`) |
|---|---|---|---|
| `00-mo-rong-schema.sql` | nền | Chặn chạy thiếu P01; 4 đối tượng mới ở bảng trên | 9 bảng trong schema |
| `01-fixture.sql` | nền | Sinh dữ liệu tất định, cài ca khó và sai lệch, `ANALYZE` | `INSERT 0 3174` / `20685` / `1891` / `2821`(+5 cài sau) |
| `02-kiem-tra-fixture.sql` | nền | Hồ sơ dữ liệu H1–H9 + 9 bất biến K1–K9 (0 dòng), gồm B1–B5/B7 của `p01/07` | như các bảng trên |
| `11-null-case-coalesce.sql` | PG-011 `postgresql-null-case-coalesce` | `= NULL` vs `IS NULL`; CASE đọc nghĩa của NULL; LEFT JOIN phân biệt "không có dòng" với "cột NULL"; `avg` bỏ qua NULL; tip NULL ≠ 0; NULL lây qua phép cộng; `NULLIF(fare, 0)`; COALESCE để hiển thị; `<>` vs `IS DISTINCT FROM`; chia 0 → lỗi `22012` rồi `NULLIF` | `driver_id = NULL` → 0, `IS NULL` → 297; đánh giá: có sao 1.738 / chưa đánh giá 816 / không áp dụng 461 / bỏ trống sao 153; sao TB **4,22** vs COALESCE 0 → **3,88**; =5: 896, <>5: 842, NULL 153; tip TB 4.122 đ (người trả lời) vs 2.943 đ (mỗi lượt), tổng 5.565.000; cước TB 67.339 vs 68.119 (bỏ 0 đ); khu ≠ 111: `<>` 2.976 vs `IS DISTINCT FROM` 3.012; 20/10: DN NULL, HN 13,5 %, SG 14,3 % |
| `12-aggregate-group-by.sql` | PG-012 `postgresql-aggregate-group-by-having` | `count(*)/count(col)/count(DISTINCT)`; bảng TP × dịch vụ với mẫu số ghi rõ; 3 lỗi mẫu số; `42803`; nhóm theo PK; WHERE vs HAVING; ROLLUP + `grouping()`; hai mẫu số trong một dòng; `count(DISTINCT)` không cộng qua nhóm | 3.174 / 2.877 / 1.432 / 52 tài xế / 576 khách; tỉ lệ huỷ SG-parcel cao nhất 21,9 %; chia nguyên → 0 (và 14 thay 14,10); cước TB sai (gồm huỷ) SG 68.269 vs đúng 67.451; HAVING ≥ 63 chuyến: 7 tài xế (8101, 8109 cùng 68); tổng ROLLUP 2.707 / 182.287.000; độ phủ đánh giá 68,4–71,2 %, điểm 4,17–4,30; khách: food 441 + parcel 241 + ride 421 ≠ 569 |
| `13-join-cardinality.sql` | PG-013 `postgresql-join-quan-he-cardinality` | Đếm trước/sau JOIN; cộng cước sau JOIN; `sum(DISTINCT)` sai; tìm khoá nở dòng; gom trước rồi JOIN; anti-join; điều kiện ở ON vs WHERE; hai quan hệ 1–n nhân nhau; nhiều–một an toàn; EXISTS cho câu "có/không" | 2.707 chuyến → 2.800 dòng (INNER) / 2.832 (LEFT); Σ cước SAI **191.644.000** vs đúng **182.287.000**; `sum(DISTINCT)` 11.259.000; gom trước: thực thu 182.531.000 (+244.000 = sai lệch của chuyến hoàn thành); thiếu thu: 31 cước 0 đ + 2 có cước; ON vs WHERE 2.707 vs 2.674; 100042: 2 × 7 = 14 dòng; Σ thu 183.640.000 → 1.283.272.000 sau 2 JOIN; ví: 1.050 dòng vs 961 chuyến |
| `14-subquery-exists-cte.sql` | PG-014 `postgresql-subquery-exists-cte` | Subquery vô hướng/tương quan; lỗi `21000` khi trả >1 dòng; IN vs EXISTS; **bẫy NOT IN + NULL**; LATERAL top-N mỗi nhóm; chuỗi CTE b1→b4 có câu kiểm kê từng bước; MATERIALIZED/NOT MATERIALIZED | trên TB thành phố: DN 190, HN 436, SG 636; khách từng huỷ 266; `NOT IN` → **0 dòng**, `NOT EXISTS` → 8219, 8220; kiểm kê 3.168 / 2.688 / 3.168 / 3.168 / **7**; tổng chênh b4 = b3 = **+268.000** |
| `15-window-function.sql` | PG-015 `postgresql-window-function-frame` | Luỹ kế ROWS; TB 7 ngày trên lịch đầy đủ vs ROWS trên dữ liệu thưa vs RANGE `interval '6 days'`; frame mặc định RANGE với khoá trùng; `42P20`; row_number/rank/dense_rank với hoà; lag; bẫy last_value; `OVER ()` sau GROUP BY | DN 17–24/10: TB7 lịch đủ 20/10 = 672.571 vs ROWS thưa NULL (ngày không có dòng), 21/10 677.286 vs 797.143; RANGE khớp lịch đủ (0 ngày lệch); lô 10/10: đếm mặc định nhảy 3 → 9, ROWS 4…9; SG: 8101, 8109 cùng hạng 1, 8120 rank 3 / dense_rank 2; DN dense_rank ≤ 3 trả 4 người; tỉ trọng SG 50,95 %, HN 34,56 %, DN 14,49 % |
| `16-date-time-timezone.sql` | PG-016 `postgresql-date-time-timezone` | Hai cách hiển thị một khoảnh khắc; 6 cách lọc "ngày 15/10" ở phiên VN và phiên UTC; bảng hai biên; dựng biên từ tham số ngày; ngày VN vs ngày UTC; biên tháng; giờ trong ngày; `generate_series(date,…)` trả timestamptz; lịch + LEFT JOIN; interval; giờ mùa hè (New York) | Ngày 15/10: đúng **106**; phiên VN: `::date` 106, BETWEEN …23:59:59 **105**, `<= '2026-10-16'` **107**; phiên UTC: `::date`/BETWEEN/`<=` đều 105; 14–16/10 VN 100/106/103 vs UTC 102/105/104; tháng: 3.168 vs 3.174 (6 chuyến 31/10 lọt vào 30/10 UTC); 1 ngày ≠ 24 giờ ở New York 01/11/2026 |
| `17-upsert-on-conflict.sql` | PG-017 `postgresql-upsert-on-conflict` | Quy tắc R1–R5; lô trùng khoá → `21000`; khử trùng xong vẫn `23505 uq_menu_items_name` (ON CONFLICT chỉ bắt khoá chỉ định); bảng xem trước từng dòng; MERGE + `merge_action()` (rollback); UPSERT có `WHERE` (không ghi đè dữ liệu mới hơn, không UPDATE thừa); idempotent; DO NOTHING; `42P10`; `ON CONSTRAINT` | Lô 2 (8 dòng): 1 thêm, 2 cập nhật, 3 bỏ qua (R2/R3/R4), 2 từ chối ⇒ `INSERT 0 3`; chạy lại `INSERT 0 0`; DO NOTHING chèn 1/2 (CF-01), BB-01 giữ 59.000 |
| `18-pagination-keyset.sql` | PG-018 `postgresql-pagination-keyset` | Con trỏ chỉ theo `requested_at` (`<` mất dòng, `<=` lặp dòng); so sánh bộ `(requested_at, code) < (…)`; dạng viết tách; hai cột ngược chiều; recursive CTE đi hết mọi trang (chứng minh); OFFSET trôi khi có chuyến mới (rollback) | Con trỏ trang 1 = (18/10 14:30, 101884); `<` mất 101883, 101882; `<=` lặp 101885, 101884; 4 trang 5/5/5/3, **18 mã, 18 khác nhau**; OFFSET 5 lặp 101884 sau khi chèn 103174 |
| `19-cay-khu-vuc.sql` | PG-019 `postgresql-recursive-cte-cay-khu-vuc` (1/2) | Giải phẫu đệ quy; đi xuống với `SEARCH DEPTH FIRST`; đường đi bằng mảng; đi lên; bảng closure + cộng dồn chuyến/doanh thu theo cây; đối chiếu; kiểm cây lành có chốt độ sâu | SG: 15 nút; HN 1.094 chuyến / 62.246.000 đ, HN-NA 0; qua cây 3.138 = trực tiếp 3.138 (+36 mất GPS); 34 nút đi được từ gốc, sâu nhất 2, 22 lá |
| `19-chu-trinh.sql` | PG-019 (2/2) — fixture chu trình | Đảo cha hai vùng HN trong giao dịch: mọi ràng buộc vẫn qua; hệ quả im lặng; chốt độ sâu; mệnh đề `CYCLE`; tự viết bằng mảng; tìm nút không tới được gốc và nút nằm trên vòng; ROLLBACK | Vòng 21 → 221 → 22 → 211 → 21; HN tụt **1.094 → 340** chuyến, 5 nút; 7 nút không tới gốc; 4 nút trên vòng; sau ROLLBACK 0 |
| `20-doi-soat-thanh-toan.sql` | PG-020 `postgresql-thuc-hanh-sql-doi-soat` | Đặc tả đối soát; VIEW tạm `so_doi_soat`; đối soát tổng + số dòng; **đáp án 7 dòng**; tổng theo loại; ca khó nhưng đúng; quy tắc ngây thơ; chênh lệch thời điểm (ngày giao dịch vs ngày đặt); giao dịch pending cần theo dõi | 3.168 chuyến / phải thu 182.287.000 / thực thu 182.555.000 / chênh **+268.000**; 0 giao dịch ngoài phạm vi; theo loại: −72.000 · +96.000 · −174.000 · +85.000 · +333.000; số dòng 2.671 + 447 + 31 + 12 + 7 = 3.168; quy tắc ngây thơ báo **115** chuyến; 15/10: 5.862.000 vs 5.823.000, 16/10: 5.244.000 vs 5.228.000; tháng 10 theo ngày giao dịch 182.550.000 (+5.000 ghi ngày 31/10); pending: 103162 |
| `OUTPUT.txt` | mọi bài | Kết quả thật của `npm run p02` | Test so khớp từng byte |

## Chỉ thị kỳ vọng mới của runner

Ngoài `-- expect-error:` và `-- expect-rows:` của P01, P02 dùng thêm (định nghĩa trong `run.mjs`, mô tả ở [README lab](../README.md)):

- `-- expect-value: 182287000` — kết quả phải đúng 1 dòng × 1 cột với giá trị đó.
- `-- expect-line: 100724 | completed | 76000 | …` — lặp một dòng mỗi dòng kết quả; **toàn bộ** kết quả phải khớp đúng thứ tự, đúng số dòng. Ô cách nhau bằng `|`, khoảng trắng hai đầu ô bỏ qua (nên căn cột thoải mái), NULL viết `NULL`.
- `-- expect-md5: <32 hex>` — md5 của toàn bộ kết quả (dạng chuẩn: các ô nối `" | "`, các dòng nối xuống dòng, NULL là `NULL`). Chỉ dùng cho kết quả dài khó đọc (3 chỗ: 02-H1, 12 ROLLUP, 15 lô khách doanh nghiệp); sai thì runner in md5 thật.

Psql coi các dòng này là chú thích thường — tự đối chiếu bằng mắt.

## Lưu ý cho người viết bài

- Trích số từ `OUTPUT.txt`; dòng `-- [lab]` là của runner. Sửa SQL ⇒ `npm run p02:output` rồi `npm test`.
- **`ANALYZE` cuối fixture**: thiếu nó, cả bộ mất ~30 giây trên PGlite (vài truy vấn ở 14 và 20 chọn kế hoạch tồi, 4–8 giây mỗi câu); có nó ~3 giây; **kết quả không đổi** (đã so OUTPUT trước/sau). Đo một lần trên máy viết bài, không phải benchmark. Đây là chất liệu cho P03, đừng trích như số đo hiệu năng.
- Thứ tự kết quả luôn có khoá phụ (thường là `code`); `RETURNING` của MERGE/UPDATE được đặt trong `WITH … SELECT … ORDER BY` vì thứ tự xử lý tuỳ kế hoạch. `ORDER BY` trên chữ dùng `COLLATE "C"` (PGlite chỉ có collation C; xem README lab).
- Phiên bản tính năng (đối chiếu tài liệu chính thức): `MERGE` có từ 15; `RETURNING` + `merge_action()` của MERGE và MERGE trong `WITH` có từ **17** (release notes 17; trang `queries-with` bản 16 ghi rõ "but not MERGE"); `SEARCH`/`CYCLE` của CTE đệ quy từ 14; gộp CTE dùng một lần vào truy vấn chính từ 12. `date_trunc` dạng 3 tham số (có múi giờ) chạy được trên 17.5 — chưa ghi phiên bản bắt đầu.
- Hành vi đã quan sát, đúng tài liệu `INSERT`: dòng bị `ON CONFLICT DO UPDATE … WHERE` loại không có trong `RETURNING` và không được đếm trong thẻ `INSERT 0 n`; lệnh không được sửa một dòng hai lần (`21000`); không có index unique khớp `conflict_target` ⇒ lỗi (`42P10`).
- `generate_series(date, date, interval)` trả `timestamptz` (đã đo `pg_typeof`); file 15/16 dùng `DATE + số nguyên` để có kiểu `date`.
- `count(DISTINCT amount) FILTER (…)` dùng để phát hiện "thu trùng" giả định hai lần trả một phần không bằng nhau — fixture bảo đảm điều đó (phần thẻ 10/15/20 nghìn, cước > 40.000); bài thực hành nên nói rõ giới hạn của quy tắc.
- PG-019 không chạy truy vấn đệ quy "trần" trên dữ liệu có vòng (lặp vô hạn); bài chỉ mô tả. PGlite chỉ có một kết nối: các ý "hai phiên cùng UPSERT" để P04.
- Ngày 01/11/2026 New York lùi đồng hồ (đã quan sát `-04` → `-05`); Việt Nam không có giờ mùa hè.
