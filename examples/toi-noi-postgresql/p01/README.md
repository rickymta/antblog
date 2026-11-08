# P01 · Nền tảng và mô hình dữ liệu

Lab cho mười bài PG-001 … PG-010 trong [khung chương trình](../../../docs/postgresql-master-curriculum.md). Sản phẩm của phần: **schema tối thiểu cho khách, tài xế và chuyến, fixture nhỏ hợp lệ/sai và bằng chứng kiểm tra ràng buộc**. Cách cài và chạy chung ở [README của lab](../README.md); tóm tắt:

```sh
cd examples/toi-noi-postgresql
npm ci
npm run p01      # 00 → 07 trên PGlite 17.5, kết quả như OUTPUT.txt
npm test         # 6 ca kiểm tra
```

Trạng thái kiểm chứng: **chạy thật trên PGlite 0.3.16 (PostgreSQL 17.5)**, Node 22.18.0, Windows 11 — 176 câu lệnh, 38 lỗi mong đợi khớp, 56 kiểm số dòng đạt, 0 FAIL. **Chưa chạy trên PostgreSQL server thật** (xem bảng khác biệt ở README của lab).

## Các file

| File | Minh hoạ | Bài dùng | Kết quả mong đợi |
|---|---|---|---|
| `00-reset.sql` | Schema riêng `toinoi`, `DROP SCHEMA … CASCADE`, `search_path`, `TimeZone` của phiên | PG-001 (`postgresql-la-gi-va-vai-tro-trong-toi-noi`), PG-002 (`postgresql-cai-dat-lab-psql-ket-noi`) | Lần đầu: `NOTICE:  schema "toinoi" does not exist, skipping`; một dòng `toinoi \| Asia/Ho_Chi_Minh \| 17.5` |
| `01-schema.sql` | 6 bảng, kiểu dữ liệu tiền/thời gian/định danh, ràng buộc đặt tên, `COMMENT ON`, xem cấu trúc qua `information_schema` | PG-003 (`postgresql-bang-dong-cot-primary-foreign-key`), PG-004 (`postgresql-kieu-du-lieu-tien-thoi-gian-uuid`), PG-005 (`postgresql-create-table-constraints`) | `CREATE TABLE` ×6, `CREATE INDEX` ×4, `COMMENT` ×16, bảng 13 cột của `trips` |
| `02-fixture-valid.sql` | Fixture cố định: 3 thành phố, 6 khách, 6 tài xế, 3 quán, 15 chuyến | PG-005, PG-006, PG-007, PG-010 (`postgresql-thuc-hanh-schema-chuyen-xe`) | `INSERT 0 3/6/6/3/15`; bảng 8 trạng thái × 3 loại |
| `03-fixture-invalid.sql` | 33 câu sai, mỗi câu vi phạm đúng một quy tắc: 3 NOT NULL, 4 UNIQUE/PK, 3 FK, 17 CHECK, 1 IDENTITY, 5 sai kiểu/định dạng; bẫy `DateStyle` không báo lỗi | PG-004, PG-005, PG-010 | 33 `ERROR` đều `PASS`; dữ liệu vẫn 3/6/6/15 |
| `04-lifecycle.sql` | Một chuyến đi qua 7 trạng thái bằng `INSERT/UPDATE … RETURNING`; UPDATE có điều kiện trả `UPDATE 0`; CTE `WITH … UPDATE … RETURNING` + `INSERT`; huỷ bản nháp, `DELETE` bị FK chặn rồi `DELETE … RETURNING` | PG-006 (`postgresql-insert-update-delete-returning`) | Chuyến **100029** `completed` sau 31 phút, 7 dòng `trip_events`; bản nháp 100030 bị xoá |
| `05-select.sql` | `WHERE` theo trạng thái/danh sách/khoảng thời gian nửa mở theo giờ VN, `ORDER BY` có tie-breaker, `LIMIT/OFFSET`, `NULLS LAST`, collation | PG-007 (`postgresql-select-filter-order-limit`) | Xem "Điểm đáng trích" bên dưới |
| `06-migration.sql` | Expand → backfill → contract: thêm `drivers.vehicle_plate`, `NOT VALID`, `VALIDATE` thất bại vì dữ liệu cũ sai rồi sửa, `SET NOT NULL` nhờ CHECK đã validate | PG-009 (`postgresql-schema-migration-tuong-thich`) | 3 lỗi mong đợi; dòng `DEBUG:  existing constraints on column "drivers.vehicle_plate" are sufficient to prove that it does not contain nulls` |
| `06-migration-rollback.sql` | Hoàn tác hai mức: nới lỏng (giữ dữ liệu) → gỡ cột sau khi sao lưu; chạy lặp được | PG-009 | Không chạy trong `npm run p01`; test chạy nó (2 lần) rồi chạy lại 06 và 07 |
| `07-checks.sql` | 13 truy vấn bất biến trả 0 dòng: mồ côi, trạng thái ↔ mốc, trùng, quy tắc nhiều bảng (tài xế/quán khác thành phố, tài xế hai chuyến, `busy` lệch), lịch sử khớp chuyến, ràng buộc chưa validate, tên sai quy ước; kiểm kê ràng buộc và số dòng | PG-010; tư liệu cho PG-008 (`postgresql-mo-hinh-quan-he-chuan-hoa`) | Mọi kiểm tra `(0 rows)`; số dòng cuối 3/6/7/3/16/7 |
| `OUTPUT.txt` | Kết quả thật của `npm run p01` (in lại câu lệnh + kết quả) | Mọi bài | Test so khớp từng byte; sửa SQL thì chạy `npm run p01:output` |

PG-008 không có file riêng: bài dùng chính schema và 07 làm ví dụ (xem mục "Tư liệu cho PG-008").

## Lược đồ

Bám lược đồ lõi của hồ sơ Tới Nơi — cùng tên bảng, cột, kiểu, giá trị mặc định, ba index của `trips` và index của `trip_events`.

| Bảng | Cột | PK | UNIQUE | FK | CHECK |
|---|---|---|---|---|---|
| `cities` | `id smallint`, `code`, `name` | `pk_cities` | `uq_cities_code` | — | `ck_cities_code_format` (2 chữ in hoa), `ck_cities_name_not_blank` |
| `customers` | `id uuid` (gen_random_uuid), `phone`, `full_name`, `created_at timestamptz` | `pk_customers` | `uq_customers_phone` | — | `ck_customers_phone_format` (`^0[0-9]{9}$`), `ck_customers_full_name_not_blank` |
| `drivers` | `id uuid`, `code integer`, `full_name`, `city_id`, `vehicle_type`, `status` (mặc định `offline`), `created_at` (+ `vehicle_plate` sau 06) | `pk_drivers` | `uq_drivers_code` | `fk_drivers_city` | `ck_drivers_code_positive`, `ck_drivers_full_name_not_blank`, `ck_drivers_vehicle_type` (bike/car), `ck_drivers_status` (offline/online/busy); sau 06: `ck_drivers_vehicle_plate_format` |
| `merchants` | `id uuid`, `city_id`, `name`, `created_at` | `pk_merchants` | — | `fk_merchants_city` | `ck_merchants_name_not_blank` |
| `trips` | `id uuid`, `code bigint GENERATED ALWAYS AS IDENTITY (START WITH 100000)`, `kind`, `city_id`, `customer_id`, `driver_id`, `merchant_id`, `status` (mặc định `requested`), `fare integer` (đồng), `requested_at`, `accepted_at`, `completed_at`, `updated_at` | `pk_trips` | `uq_trips_code` | `fk_trips_city`, `fk_trips_customer`, `fk_trips_driver`, `fk_trips_merchant` | 10 CHECK — bảng dưới |
| `trip_events` | `id bigint IDENTITY`, `trip_id`, `status`, `occurred_at` | `pk_trip_events` | — | `fk_trip_events_trip` | `ck_trip_events_status` (8 trạng thái) |

Mười CHECK của `trips`:

| Ràng buộc | Quy tắc |
|---|---|
| `ck_trips_kind` | `kind IN ('ride','food','parcel')` |
| `ck_trips_status` | 8 trạng thái của vòng đời |
| `ck_trips_fare_non_negative` | `fare >= 0` (0 đ hợp lệ — chuyến miễn phí) |
| `ck_trips_merchant_only_food` | `(kind = 'food') = (merchant_id IS NOT NULL)` |
| `ck_trips_driver_accepted_together` | có tài xế ⇔ có `accepted_at` |
| `ck_trips_unassigned_no_driver` | `requested`/`searching` ⇒ chưa có tài xế |
| `ck_trips_assigned_has_driver` | `accepted` … `completed` ⇒ phải có tài xế (`cancelled` thì tuỳ) |
| `ck_trips_completed_at_by_status` | `(status = 'completed') = (completed_at IS NOT NULL)` |
| `ck_trips_accepted_after_requested` | `accepted_at >= requested_at` |
| `ck_trips_completed_after_accepted` | `completed_at >= accepted_at` |

Kiểm kê cuối (07, C3): `cities` 1p/1u/2c · `customers` 1p/1u/2c · `drivers` 1p/1f/1u/5c · `merchants` 1p/1f/1c · `trip_events` 1p/1f/1c · `trips` 1p/4f/1u/10c. NOT NULL không nằm trong `pg_constraint` ở PostgreSQL 17 (đã kiểm: chỉ có `contype` c/f/p/u); theo ghi chú phát hành, PostgreSQL 18 mới lưu NOT NULL vào `pg_constraint` — chưa kiểm.

### Khác với hồ sơ Tới Nơi (có chủ đích)

1. **Mọi ràng buộc được đặt tên** `pk_/fk_/uq_/ck_` (hồ sơ để PostgreSQL tự đặt, trừ `uq_trips_code`). Ngữ nghĩa PK/FK/UNIQUE không đổi; thông báo lỗi chỉ thẳng vào quy tắc.
2. **Thêm CHECK nghiệp vụ** (định dạng số điện thoại và mã thành phố, tên không rỗng, `code > 0`, mọi CHECK của `trips` trừ hai danh sách kind/status vốn có trong hồ sơ, danh sách trạng thái cho `trip_events`). Script sinh dữ liệu của hồ sơ vẫn thoả mọi CHECK này: đã chạy chính script đó trên `00 → 01`, thu nhỏ còn 2.000 khách / 500 tài xế / 200 quán / 2.000 chuyến (logic giữ nguyên) — 0 lỗi, đủ 9 nhóm kind × status. Bản 200.000 chuyến không chạy xong trên PGlite trong vài phút (mỗi dòng đọc phần tử trong mảng 50.000 UUID), nên chưa đo trực tiếp. **Lưu ý:** dữ liệu kiểu hồ sơ KHÔNG qua được ba kiểm tra nhiều bảng của 07 — B1 (tài xế khác thành phố chuyến: 1.638/2.000 chuyến), B2 (quán khác thành phố: 780) và B5 (`updated_at` = giờ đặt, sớm hơn `accepted_at`/`completed_at`: 1.980) — vì script chọn tài xế/quán ngẫu nhiên và đặt `updated_at = requested_at`. Đây là tư liệu tốt cho PG-008/PG-010, hoặc là việc cần sửa ở script hồ sơ nếu muốn dữ liệu lớn "sạch".
3. **Chưa có `driver_locations`** — không cần cho P01.
4. **06 thêm `drivers.vehicle_plate`** làm bài tập migration; `06-migration-rollback.sql` gỡ nó. Phần sau nên bắt đầu từ `00 → 05` (hoặc chạy rollback) nếu muốn đúng lược đồ hồ sơ.

Kiểu dữ liệu: tiền là `integer` đồng (VND không có đơn vị lẻ; tối đa 2.147.483.647 đ/chuyến; `sum(integer)` trả `bigint` — đã kiểm `pg_typeof`), không dùng `real/double` (sai số nhị phân) hay `money` (định dạng phụ thuộc `lc_monetary`). Thời điểm là `timestamptz`. Định danh `uuid` v4 qua `gen_random_uuid()` (lõi PostgreSQL từ 13, không cần extension) + mã dễ đọc `code`.

## Fixture

Ảnh chụp dữ liệu lúc **2026-11-03 08:30 giờ VN**. UUID ghi sẵn để kết quả ổn định: khách `c0000000-…-00000000000N`, tài xế `d0000000-…-00000000800N` (đuôi = mã), quán `e0000000-…`, chuyến fixture `70000000-…-000000100NNN` (đuôi = mã chuyến, chỉ để dễ đối chiếu), chuyến của 04 `b0000000-…`, dòng dùng để thử lỗi `f0000000-…`. Số điện thoại `0900000001` … `0900000006`.

Chuyến theo trạng thái × loại sau 02 (15 chuyến):

| Trạng thái | ride | food | parcel | Tổng | Mã |
|---|---|---|---|---|---|
| requested | 1 | 0 | 0 | 1 | 100014 |
| searching | 0 | 1 | 0 | 1 | 100013 |
| accepted | 1 | 0 | 0 | 1 | 100012 |
| arriving | 0 | 0 | 1 | 1 | 100011 |
| picked_up | 0 | 1 | 0 | 1 | 100009 |
| in_transit | 1 | 0 | 0 | 1 | 100010 |
| completed | 3 | 2 | 2 | 7 | 100000, 100001, 100002, 100004, 100005, 100007, 100008 |
| cancelled | 1 | 1 | 0 | 2 | 100003 (huỷ sau khi nhận), 100006 (huỷ khi chưa ai nhận) |
| **Tổng** | 7 | 5 | 3 | 15 | |

Theo thành phố: SG 8, HN 5, DN 2. Tài xế: 8001/8002/8004/8005 `busy` (mỗi người đúng một chuyến đang chạy), 8003 `online`, 8006 `offline`. Chi tiết dụng ý: 100003 và 100004 cùng `requested_at` 12:00; 100007 đặt 23:50 ngày 02/11, 100008 đặt 00:10 ngày 03/11 (vẫn là 02/11 theo UTC); 100008 cước 0 đ. 15 chuyến fixture **không có** `trip_events` (như dữ liệu chuyển từ hệ cũ); 07-B7 chỉ xét chuyến có lịch sử.

Sau 07: 3 thành phố, 6 khách, 7 tài xế (thêm 8007 ở 06), 3 quán, 16 chuyến (thêm 100029; 100030 đã xoá), 7 `trip_events`. Completed: 8 chuyến.

## Điểm đáng trích cho bài viết (đều có trong `OUTPUT.txt`)

- **Mã chuyến nhảy cóc (PG-006):** chuyến mới ở 04 nhận mã **100029**, không phải 100015. Mười bốn số 100015–100028 đã bị tiêu ở 03: 1 câu hợp lệ bị `ROLLBACK` + 13 câu INSERT vào `trips` lỗi ở bước ràng buộc (NOT NULL/FK/CHECK). Các câu lỗi sai kiểu/định dạng (`22P02`, `22003`, `22007`, `22008`) và `428C9` **không** tiêu số — lỗi xảy ra khi đọc giá trị, trước khi IDENTITY được gọi. Cuối 04: `max(code)` 100029, 16 chuyến, `trips_code_seq.last_value` 100030. Tương tự, `trip_events.id` bắt đầu từ **2** vì số 1 bị câu lỗi `ck_trip_events_status` ở 03 tiêu mất.
- **UPDATE có điều kiện (PG-006):** tài xế 8001 nhận chuyến đã `accepted` ⇒ `(0 rows)` / `UPDATE 0`; cùng câu không có `AND status = 'searching'` sẽ ghi đè tài xế (lost update — không chạy trong lab).
- **FK chặn DELETE (PG-006):** `update or delete on table "trips" violates foreign key constraint "fk_trip_events_trip" on table "trip_events"` / `DETAIL:  Key (id)=(b0000000-…-000000000002) is still referenced from table "trip_events".`
- **CHECK nhận NULL:** `ck_trips_accepted_after_requested` không chặn chuyến chưa có `accepted_at` vì biểu thức ra NULL (tài liệu: CHECK đạt khi biểu thức là true **hoặc null**). Các quy tắc "phải có" được viết bằng `IS NULL`/`IS NOT NULL`.
- **Nhiều CHECK cùng vi phạm:** lần viết đầu, dòng `status = 'done'` vi phạm cả `ck_trips_status` lẫn CHECK tài xế cũ, và PostgreSQL báo `ck_trips_driver_by_status` — tên đứng trước theo thứ tự chữ cái (quan sát được 2 lần; tài liệu không hứa thứ tự này). Bài học đã áp dụng: mỗi CHECK một quy tắc dạng `NOT A OR B`, mỗi câu trong 03 vi phạm đúng một quy tắc.
- **Múi giờ (PG-007):** cùng dữ liệu, `requested_at::date = '2026-11-02'` đếm **9** ở phiên UTC còn `(requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date` đếm **8**; khoảng nửa mở `[2026-11-02 00:00+07, 2026-11-03 00:00+07)` trả 8 chuyến. Chuyến 100008 hiển thị `2026-11-02 17:10:00+00` ở phiên UTC.
- **Phân trang với ORDER BY không duy nhất (PG-007):** `ORDER BY kind LIMIT 3` → 100003, 100002, 100004; `ORDER BY kind LIMIT 3 OFFSET 3` → 100004, 100002, 100007 — hai chuyến lặp lại, 100009 và 100013 (food) bị bỏ qua. Thêm `, code` ⇒ trang 1 = 100002, 100003, 100004; trang 2 = 100009, 100013, 100001. Kết quả này lặp lại giống hệt khi chạy 05 ngay sau 02 hay sau 04 (đã thử), nhưng chưa đối chiếu trên server thật. (Thử nghiệm bỏ đi: dùng `UPDATE` trong giao dịch để đảo thứ tự hai dòng hoà — đảo được khi chạy ngay sau 02 nhưng KHÔNG đảo sau 03/04 vì phiên bản dòng mới dùng lại chỗ trống trong trang; quá phụ thuộc lịch sử nên không đưa vào lab.)
- **NULLS (PG-007):** `ORDER BY accepted_at DESC` mặc định đặt NULL lên đầu; 05 dùng `NULLS LAST`.
- **Collation:** PGlite xếp "Đặng Ngọc Hà" cuối cùng dưới `COLLATE "C"`.
- **DateStyle:** `'03/11/2026'::date` → `2026-03-11`, không có lỗi nào.
- **Migration (PG-009):** `VALIDATE CONSTRAINT` thất bại với `check constraint "ck_drivers_vehicle_plate_format" of relation "drivers" is violated by some row` (dòng 8006 `43a 67890`); sửa thành `43A-678.90` rồi validate lại. `SET NOT NULL` in `DEBUG:  existing constraints on column "drivers.vehicle_plate" are sufficient to prove that it does not contain nulls` — bằng chứng server bỏ qua lần quét bảng (từ PostgreSQL 12). Các nhận định về mức khoá (`SHARE UPDATE EXCLUSIVE` cho VALIDATE, `ACCESS EXCLUSIVE` cho SET NOT NULL) lấy từ tài liệu `ALTER TABLE`; **không quan sát được trên PGlite một kết nối**.

## Tư liệu cho PG-008 (chuẩn hoá)

- `trips.city_id` cùng tồn tại với `drivers.city_id` và `merchants.city_id`: dữ liệu lặp ⇒ có thể lệch (07-B1, B2). Một cách khoá bằng schema là FK hai cột `(merchant_id, city_id) → merchants (id, city_id)` kèm UNIQUE `(id, city_id)` — để làm bài tập, lab chưa áp dụng.
- `drivers.status = 'busy'` là dữ liệu suy ra từ `trips` ⇒ bất thường cập nhật khi quên đổi (07-B4; 04 đổi tay ở bước nhận và hoàn thành).
- `trip_events` tách khỏi `trips`: một chuyến nhiều sự kiện; 07-B7 kiểm sự kiện cuối khớp trạng thái hiện tại.

## Lưu ý cho người viết bài

- Trích kết quả từ `OUTPUT.txt`, không tự gõ lại. Dòng `-- [lab]` là của runner — khi trích như "kết quả psql" thì bỏ các dòng đó.
- Mọi file đặt `SET search_path TO toinoi;` và `SET TIME ZONE 'Asia/Ho_Chi_Minh';` ở đầu, nên chạy riêng một file bằng psql vẫn đúng (với 03–07 cần đã chạy 00–02 trước; 05 và 07 dùng dữ liệu của 04 và 06 cho đúng số dòng).
- Sửa SQL ⇒ `npm run p01:output` rồi `npm test` (test so `OUTPUT.txt` từng byte và đếm đủ mọi chỉ thị `expect-*`).
- Một số câu dùng cú pháp của phần sau để phục vụ kiểm tra (`count(*) FILTER`, `GROUP BY`, `JOIN LATERAL`, `NOT EXISTS`, `ON CONFLICT` trong rollback, CTE ghi dữ liệu). Bài P01 nên nói rõ "P02 giải thích" thay vì dạy lướt.
- Các câu concurrency (hai tài xế bấm cùng lúc thật sự), đo khoá, `lock_timeout` có tác dụng: cần server thật và hai phiên — để P04.

## Thí nghiệm trong bài

Câu SQL minh hoạ và lời giải bài tập của PG-001…PG-010 nằm ở [`thu-nghiem/`](thu-nghiem/README.md), mỗi tệp ghi rõ cần chạy sau nền nào; `npm test` kiểm cả các tệp này.

## Lưu ý về kiểm tra B1 của `07-checks.sql`

B1 so thành phố **hiện tại** của tài xế với mọi chuyến tài xế đó từng chạy. Khi tài xế chuyển thành phố hợp lệ, các chuyến cũ đã hoàn thành cũng bị báo — đây là chủ đề bài tập 3 của PG-008 (ràng buộc liên bảng phụ thuộc thời điểm, CHECK không diễn đạt được).
