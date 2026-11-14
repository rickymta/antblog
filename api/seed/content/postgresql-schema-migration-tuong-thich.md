---
slug: postgresql-schema-migration-tuong-thich
title: "Thay đổi schema mà dữ liệu cũ vẫn đọc được"
summary: "Tới Nơi cần bắt buộc phải lưu biển số xe của tài xế, trong khi bảng đã có dữ liệu và app cũ vẫn chạy. Làm migration theo expand–contract, NOT VALID rồi VALIDATE, siết NOT NULL không quét bảng, kèm kế hoạch khôi phục."
category: postgresql
level: advanced
tags: [postgresql, pg-009, migration, alter-table, constraint, expand-contract, toi-noi]
language: vi
---

Bộ phận an toàn của Tới Nơi chốt một yêu cầu mới: khách phải thấy **biển số xe** của tài xế trước khi lên xe, nên mọi tài xế đều phải có biển số đúng định dạng. Nghe như một dòng SQL: thêm cột `vehicle_plate text NOT NULL` vào bảng `drivers`. Nhưng bảng đã có tài xế đang chạy chuyến, biển số của họ nằm trong một file do bộ phận tuyển dụng gửi sang, và app đăng ký tài xế bản cũ — chưa biết gì về biển số — vẫn đang chạy trên máy chủ cho tới khi bản mới được triển khai hết.

Câu `ADD COLUMN … NOT NULL` chạy thẳng sẽ lỗi ngay vì dòng cũ không có giá trị; kể cả ép được, app cũ sẽ không thêm được tài xế mới. Bài này đổi schema theo từng bước mà ở mỗi bước dữ liệu cũ vẫn đọc được, ứng dụng cũ vẫn chạy, và luôn có đường lùi.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-009 · Phần 1/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/postgresql-create-table-constraints">CREATE TABLE và ràng buộc dữ liệu</a>, <a href="/articles/postgresql-insert-update-delete-returning">INSERT, UPDATE, DELETE với RETURNING</a>, <a href="/articles/postgresql-mo-hinh-quan-he-chuan-hoa">Chuẩn hoá và ranh giới mô hình</a>. Lab: <code>examples/toi-noi-postgresql/p01</code> (tệp <code>06-migration.sql</code>, <code>06-migration-rollback.sql</code>). Sau bài này, bạn thực hiện được một migration nhỏ theo expand–contract, kiểm dữ liệu cũ bằng <code>NOT VALID</code> → <code>VALIDATE</code> và có kế hoạch khôi phục chạy lại được.</p>
</div>

## 1. Migration tương thích ngược là gì?

**Schema migration** (di chuyển lược đồ) là một thay đổi có kiểm soát lên cấu trúc database: thêm cột, thêm ràng buộc, đổi kiểu, tách bảng… Các bài trước dựng schema trên database trống; từ đây schema đã có dữ liệu và người dùng.

Migration **tương thích ngược** (backward compatible) là migration mà sau khi chạy, ứng dụng **phiên bản cũ** vẫn đọc ghi được như trước. Triển khai hiếm khi tức thời: máy chủ được thay dần, app tài xế cập nhật theo từng người. Trong khoảng đó, bản cũ và bản mới **cùng chạy trên một schema**.

Ba thứ phải sống sót qua mỗi bước:

| Cần giữ | Hỏng khi | Ví dụ với `drivers` |
|---|---|---|
| Dữ liệu cũ | Ràng buộc mới không khớp dòng đã có | `NOT NULL` trên cột mà 6 tài xế cũ chưa có giá trị |
| Ứng dụng cũ | Schema đòi thứ app cũ không gửi, hoặc mất thứ app cũ cần | Bắt buộc biển số khi app cũ chưa gửi; đổi tên hay xoá cột app cũ đang đọc |
| Truy vấn đang chạy | Lệnh đổi schema giữ khoá nặng quá lâu | Quét cả bảng lớn trong khi mọi truy vấn khác phải chờ |

Thêm cột cho phép NULL là thay đổi **an toàn**. Xoá cột, đổi tên cột, thêm `NOT NULL` khi app cũ chưa ghi giá trị là thay đổi **phá vỡ** (breaking change) — không bị cấm, chỉ phải tách ra và làm sau cùng.

## 2. Expand – contract: mở rộng, chuyển dữ liệu, siết lại

Cách tách đó có tên là **expand–contract** (mở rộng – thu hẹp):

- **Expand** — thêm cấu trúc mới mà không ai bị ảnh hưởng (cột cho phép NULL, ràng buộc chưa rà dữ liệu cũ).
- **Migrate / backfill** — triển khai app mới dùng cấu trúc mới, rồi điền dữ liệu cho dòng cũ (**backfill**: điền ngược).
- **Contract** — khi mọi phiên bản app đã dùng cấu trúc mới và dữ liệu đã sạch, mới siết ràng buộc hoặc gỡ cấu trúc cũ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 900 250" role="img" aria-label="Ba giai đoạn expand, backfill và contract. App v1 chạy được ở hai giai đoạn đầu, bị từ chối ở giai đoạn contract. App v2 ghi biển số và chạy được ở cả ba giai đoạn.">
<rect x="180" y="16" width="225" height="46" rx="8" class="f-pk-soft s-pk"/><text x="292" y="36" text-anchor="middle" class="t-b">1 · Expand (B1–B2)</text><text x="292" y="53" text-anchor="middle" class="t-small">thêm cột cho phép NULL</text>
<rect x="420" y="16" width="225" height="46" rx="8" class="f-pk-soft s-pk"/><text x="532" y="36" text-anchor="middle" class="t-b">2 · Backfill (B3–B5)</text><text x="532" y="53" text-anchor="middle" class="t-small">điền + NOT VALID → VALIDATE</text>
<rect x="660" y="16" width="225" height="46" rx="8" class="f-pk-soft s-pk"/><text x="772" y="36" text-anchor="middle" class="t-b">3 · Contract (B6)</text><text x="772" y="53" text-anchor="middle" class="t-small">SET NOT NULL</text>
<text x="20" y="100" class="t-b">Cột vehicle_plate</text>
<rect x="180" y="78" width="225" height="36" rx="6" class="f-surface s-line"/><text x="292" y="101" text-anchor="middle" class="t-small">NULL được, chưa CHECK</text>
<rect x="420" y="78" width="225" height="36" rx="6" class="f-surface s-line"/><text x="532" y="101" text-anchor="middle" class="t-small">CHECK định dạng, NULL vẫn được</text>
<rect x="660" y="78" width="225" height="36" rx="6" class="f-surface s-line"/><text x="772" y="101" text-anchor="middle" class="t-small">CHECK + NOT NULL</text>
<text x="20" y="150" class="t-b">App v1</text><text x="20" y="166" class="t-small">không gửi biển số</text>
<rect x="180" y="128" width="465" height="36" rx="6" class="f-phys-soft s-phys"/><text x="412" y="151" text-anchor="middle" class="t-phys">ghi được (cột mới nhận NULL)</text>
<rect x="660" y="128" width="225" height="36" rx="6" class="f-warn-soft s-warn"/><text x="772" y="151" text-anchor="middle" class="t-warn">bị từ chối · 23502</text>
<text x="20" y="200" class="t-b">App v2</text><text x="20" y="216" class="t-small">luôn gửi biển số</text>
<rect x="180" y="178" width="705" height="36" rx="6" class="f-phys-soft s-phys"/><text x="532" y="201" text-anchor="middle" class="t-phys">ghi được ở mọi giai đoạn — phải triển khai xong TRƯỚC bước 3</text>
<text x="450" y="240" text-anchor="middle" class="t-small">Mỗi giai đoạn tự đứng được: dừng giữa chừng hệ thống vẫn chạy, quay lại giai đoạn trước vẫn được.</text>
</svg>
</div>
<figcaption><b>Hình 1 · Expand – contract cho cột biển số.</b> Bước phá vỡ app v1 (contract) chỉ chạy khi không còn máy chủ nào chạy v1. Mã bước B1–B6 khớp chú thích trong <code>06-migration.sql</code>.</figcaption>
</figure>

Lab chạy ba giai đoạn liền nhau để quan sát; ngoài đời, giai đoạn 2 và 3 có thể cách nhau nhiều ngày. Chạy `06` (cần dữ liệu `00`…`02`):

```bash
cd examples/toi-noi-postgresql
node run.mjs p01/00-reset.sql p01/01-schema.sql p01/02-fixture-valid.sql p01/06-migration.sql
```

## 3. Expand: thêm cột cho phép NULL

`06` bắt đầu bằng `SET lock_timeout = '5s'` (giải thích ở mục 7), đếm số tài xế cần biển số (6), rồi thêm cột **không có `DEFAULT`, cho phép NULL**:

```sql
ALTER TABLE drivers ADD COLUMN vehicle_plate text;
```

```text
ALTER TABLE
```

Theo [tài liệu `ALTER TABLE`](https://www.postgresql.org/docs/17/sql-altertable.html), thêm cột không `DEFAULT` hoặc `DEFAULT` không biến động (non-volatile) chỉ ghi vào **catalog** (bảng hệ thống mô tả schema), **không viết lại bảng**; dòng cũ đọc ra NULL hoặc giá trị mặc định (với hằng số, có từ PostgreSQL 11). `DEFAULT` biến động như `gen_random_uuid()` thì buộc viết lại toàn bộ bảng và index — với bảng `trips` mà hồ sơ dự án ước tính 730 triệu dòng/năm, đó là thao tác khác hẳn về thời gian và dung lượng đĩa.

Ứng dụng cũ liệt kê cột tường minh nên không thấy gì khác:

```sql
SELECT code, full_name, vehicle_type, status
FROM drivers
ORDER BY code;
```

Kết quả vẫn 6 dòng, 4 cột như trước. Code nào viết `SELECT *` thì bỗng nhận thêm một cột — đó là lý do ứng dụng không nên phụ thuộc vào `SELECT *`. Bước B2 giả lập app cũ đăng ký tài xế 8007 **không gửi biển số**; câu lệnh chạy được:

```sql
INSERT INTO drivers (id, code, full_name, city_id, vehicle_type, status, created_at)
SELECT 'd0000000-0000-4000-8000-000000008007', 8007, 'Mai Xuân Quý', 1, 'bike', 'offline',
       '2026-11-03 10:00:00+07'
WHERE NOT EXISTS (SELECT 1 FROM drivers WHERE code = 8007);
```

```text
INSERT 0 1
```

Dạng `INSERT … SELECT … WHERE NOT EXISTS` (Phần 2 giải thích) chỉ để file chạy lại được mà không lỗi trùng mã. Tài xế 8007 là dòng **mới** nhưng chưa có biển số — backfill phải xử lý cả dòng sinh ra trong giai đoạn expand.

## 4. Backfill: điền biển số từ nguồn

Biển số mỗi tài xế khác nhau, nên không có hằng số nào dùng làm `DEFAULT`. Lab nạp file của bộ phận tuyển dụng vào một **bảng tạm** (`CREATE TEMP TABLE` — chỉ sống trong phiên hiện tại) rồi điền sang `drivers`:

```sql
CREATE TEMP TABLE driver_plate_imports (
  driver_code integer PRIMARY KEY,
  plate       text NOT NULL
);

INSERT INTO driver_plate_imports (driver_code, plate) VALUES
  (8001, '59X1-123.45'), (8002, '59X2-234.56'), (8003, '29B1-345.67'),
  (8004, '30F-456.78'),  (8005, '43C1-567.89'), (8006, '43a 67890'),
  (8007, '59X3-789.01');

UPDATE drivers AS d
SET vehicle_plate = i.plate
FROM driver_plate_imports AS i
WHERE i.driver_code = d.code
  AND d.vehicle_plate IS NULL;
```

```text
INSERT 0 7
UPDATE 7
```

(Lab viết mỗi dòng `VALUES` một hàng; ở đây gộp cho gọn.) `UPDATE … FROM` là phần mở rộng của PostgreSQL: lấy giá trị từ bảng khác qua điều kiện nối. `AND d.vehicle_plate IS NULL` làm backfill **idempotent** (chạy lại cho cùng kết quả), không ghi đè giá trị đã sửa tay. Kiểm chứng ngay sau khi điền:

```sql
SELECT code, full_name FROM drivers WHERE vehicle_plate IS NULL;
```

```text
 code | full_name
------+-----------
(0 rows)
```

File nguồn cố ý có **một dòng sai định dạng** — tài xế 8006 với `43a 67890` (chữ thường, thiếu gạch). Backfill không biết điều đó; bước sau sẽ bắt.

Với bảng lớn, đừng điền bằng một câu `UPDATE` khổng lồ: chia lô (ví dụ theo khoảng `code`). Lý do nằm ở giao dịch dài, khoá dòng và việc dọn các tuple cũ — Phần 4 và Phần 5 sẽ đo; ở đây chỉ cần nhớ "nhiều câu nhỏ".

## 5. NOT VALID → VALIDATE: chặn dữ liệu mới trước, rà dữ liệu cũ sau

Thêm `CHECK` bình thường bắt PostgreSQL quét mọi dòng hiện có. Với `NOT VALID`, lần quét đó bị bỏ qua, nhưng ràng buộc **vẫn áp cho mọi `INSERT`/`UPDATE` từ giờ**. Tài liệu chỉ cho phép `NOT VALID` với `CHECK` và `FOREIGN KEY`.

```sql
ALTER TABLE drivers
  ADD CONSTRAINT ck_drivers_vehicle_plate_format
  CHECK (vehicle_plate ~ '^[0-9]{2}[A-Z][A-Z0-9]?-[0-9]{3}\.[0-9]{2}$') NOT VALID;
```

```text
ALTER TABLE
```

Toán tử `~` so khớp **biểu thức chính quy** (regular expression). Định dạng của lab là "2 số tỉnh + seri + `-` + 3 số + `.` + 2 số" — không phải quy chuẩn biển số đầy đủ. Ngay lập tức, dữ liệu **mới** sai bị chặn:

```sql
UPDATE drivers SET vehicle_plate = '59x1 12345' WHERE code = 8001;
```

```text
ERROR:  new row for relation "drivers" violates check constraint "ck_drivers_vehicle_plate_format"
DETAIL:  Failing row contains (d0000000-0000-4000-8000-000000008001, 8001, Bùi Văn Khoa, 1, bike, busy, 2026-08-01 09:00:00+07, 59x1 12345).
```

…trong khi dòng **cũ** sai vẫn nằm yên:

```sql
SELECT code, vehicle_plate
FROM drivers
WHERE vehicle_plate !~ '^[0-9]{2}[A-Z][A-Z0-9]?-[0-9]{3}\.[0-9]{2}$';
```

```text
 code | vehicle_plate
------+---------------
 8006 | 43a 67890
(1 row)
```

Hai điểm cần đọc kỹ. Thứ nhất, CHECK đạt khi biểu thức ra true **hoặc NULL** (đã gặp ở [PG-005](/articles/postgresql-create-table-constraints)), nên app cũ ghi biển số NULL vẫn qua — đúng ý ở giai đoạn này. Thứ hai, PostgreSQL ghi nhận rằng ràng buộc chưa được tin cậy trên toàn bảng: cột `convalidated` trong catalog `pg_constraint` là `f`.

Bước B5 rà dữ liệu cũ bằng `VALIDATE CONSTRAINT`:

```sql
ALTER TABLE drivers VALIDATE CONSTRAINT ck_drivers_vehicle_plate_format;
```

```text
ERROR:  check constraint "ck_drivers_vehicle_plate_format" of relation "drivers" is violated by some row
```

Thông báo **không nói dòng nào** sai — phải tự truy vấn như câu `!~` ở trên. Vì thế nên tìm dòng vi phạm **trước** khi validate bảng lớn: `VALIDATE` thất bại là một lần quét toàn bảng uổng phí. Hỏi lại bộ phận tuyển dụng, sửa dòng gốc rồi validate lại:

```sql
UPDATE drivers SET vehicle_plate = '43A-678.90'
WHERE code = 8006
RETURNING code, vehicle_plate;

ALTER TABLE drivers VALIDATE CONSTRAINT ck_drivers_vehicle_plate_format;
```

```text
 code | vehicle_plate
------+---------------
 8006 | 43A-678.90
(1 row)

UPDATE 1
ALTER TABLE
```

Vì sao tách hai bước? Không phải để bớt lần quét — `VALIDATE` vẫn quét cả bảng. Khác biệt là **mức khoá**: `VALIDATE CONSTRAINT` chỉ lấy `SHARE UPDATE EXCLUSIVE`, đọc ghi vẫn chạy song song; còn `ADD CONSTRAINT` thường quét bảng dưới khoá nặng nhất (mục 7).

## 6. Contract: siết NOT NULL mà không quét bảng dưới khoá nặng

Đây là bước **phá vỡ** app v1, chỉ chạy khi mọi phiên bản app đã gửi biển số. `SET NOT NULL` bình thường quét cả bảng để chứng minh không còn NULL, trong lúc giữ khoá `ACCESS EXCLUSIVE`. Theo tài liệu `ALTER TABLE`, nếu có sẵn một `CHECK` **đã validate** chứng minh cột không thể NULL thì lần quét được bỏ qua (từ PostgreSQL 12). Lab tận dụng điều này bằng một CHECK phụ:

```sql
ALTER TABLE drivers
  ADD CONSTRAINT ck_drivers_vehicle_plate_not_null
  CHECK (vehicle_plate IS NOT NULL) NOT VALID;

ALTER TABLE drivers VALIDATE CONSTRAINT ck_drivers_vehicle_plate_not_null;

SET client_min_messages = debug1;

ALTER TABLE drivers ALTER COLUMN vehicle_plate SET NOT NULL;

SET client_min_messages = notice;

ALTER TABLE drivers DROP CONSTRAINT ck_drivers_vehicle_plate_not_null;
```

`client_min_messages = debug1` bật thông báo gỡ lỗi để chính server cho biết nó làm gì:

```text
ALTER TABLE
ALTER TABLE
SET
DEBUG:  existing constraints on column "drivers.vehicle_plate" are sufficient to prove that it does not contain nulls
ALTER TABLE
SET
ALTER TABLE
```

Dòng `DEBUG` là bằng chứng: server dùng CHECK đã validate làm chứng cứ và không quét bảng. Ở bài tập 1 bạn sẽ thấy dòng `DEBUG` khác hẳn khi bỏ CHECK phụ. Sau đó CHECK phụ hết việc và bị gỡ — `NOT NULL` thay thế nó.

Từ giờ app cũ bị từ chối, đúng như kế hoạch:

```text
ERROR:  null value in column "vehicle_plate" of relation "drivers" violates not-null constraint
DETAIL:  Failing row contains (f0000000-0000-4000-8000-000000000030, 8030, Ứng Dụng Cũ, 1, bike, offline, 2026-11-03 11:00:00+07, null).
```

Kiểm chứng cuối migration — dữ liệu, trạng thái cột và ràng buộc:

```sql
SELECT conname, convalidated, pg_get_constraintdef(oid) AS dinh_nghia
FROM pg_constraint
WHERE conrelid = 'drivers'::regclass AND conname LIKE 'ck_drivers_vehicle_plate%'
ORDER BY conname;
```

```text
             conname             | convalidated |                                  dinh_nghia
---------------------------------+--------------+-------------------------------------------------------------------------------
 ck_drivers_vehicle_plate_format | t            | CHECK ((vehicle_plate ~ '^[0-9]{2}[A-Z][A-Z0-9]?-[0-9]{3}\.[0-9]{2}$'::text))
(1 row)
```

`information_schema.columns` báo `vehicle_plate | text | NO` và cả 7 tài xế có biển số đúng định dạng. `07-checks.sql` (C1) còn tìm mọi ràng buộc `NOT convalidated` trong schema — để không ai "quên" một `NOT VALID`.

## 7. Khoá bảng: lệnh nào nặng, lệnh nào nhẹ

Mỗi lệnh `ALTER TABLE` lấy một **khoá cấp bảng** (table lock). Hai mức cần biết trong bài này, theo [tài liệu về khoá tường minh](https://www.postgresql.org/docs/17/explicit-locking.html):

- **`ACCESS EXCLUSIVE`** xung đột với mọi mức khoá — kể cả `SELECT` thường cũng phải chờ. Tài liệu viết đây là mức duy nhất chặn một `SELECT` không có `FOR UPDATE/SHARE`.
- **`SHARE UPDATE EXCLUSIVE`** chỉ chặn các thay đổi schema khác và `VACUUM`; đọc và `INSERT/UPDATE/DELETE` vẫn chạy.

| Lệnh trong `06` (hoặc thường gặp) | Khoá theo tài liệu PG 17 | Quét / viết lại bảng? |
|---|---|---|
| `ADD COLUMN` không DEFAULT / DEFAULT hằng | `ACCESS EXCLUSIVE` | Không — chỉ sửa catalog, giữ khoá rất ngắn |
| `ADD CONSTRAINT … CHECK … NOT VALID` | `ACCESS EXCLUSIVE` | Không quét dữ liệu cũ |
| `ADD CONSTRAINT … CHECK` (không NOT VALID) | `ACCESS EXCLUSIVE` | Quét toàn bảng, không viết lại |
| `VALIDATE CONSTRAINT` | `SHARE UPDATE EXCLUSIVE` (FK thêm `ROW SHARE` trên bảng được tham chiếu) | Quét toàn bảng |
| `ALTER COLUMN … SET NOT NULL` | `ACCESS EXCLUSIVE` | Quét, trừ khi CHECK đã validate chứng minh không NULL |
| `ADD FOREIGN KEY` | `SHARE ROW EXCLUSIVE` trên cả hai bảng | Quét, trừ khi `NOT VALID` |
| `ALTER COLUMN … TYPE` | `ACCESS EXCLUSIVE` | Thường viết lại bảng và index |
| `DROP COLUMN` | `ACCESS EXCLUSIVE` | Không — cột chỉ bị ẩn, dung lượng thu hồi dần |

Quy tắc rút ra: **nặng = `ACCESS EXCLUSIVE` + quét hoặc viết lại bảng**. Expand–contract cố đưa mọi lần quét sang `VALIDATE` (khoá nhẹ), để các lệnh khoá nặng chỉ còn là sửa catalog trong chớp mắt.

Còn `SET lock_timeout = '5s'` ở đầu `06`? Theo [tài liệu cấu hình](https://www.postgresql.org/docs/17/runtime-config-client.html), `lock_timeout` huỷ câu lệnh phải chờ khoá lâu hơn ngưỡng (mặc định 0 = chờ vô hạn). Nếu một giao dịch dài đang đọc `drivers`, câu `ALTER` đứng chờ — và trong lúc chờ khoá nặng, truy vấn đến sau có thể phải xếp hàng sau nó (Phần 4 tái hiện bằng hai phiên). Thà `ALTER` thất bại sau 5 giây rồi thử lại còn hơn cả hệ thống đứng. Tài liệu khuyên không đặt tham số này trong `postgresql.conf` vì nó ảnh hưởng mọi phiên.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của lab</span>
<p>PGlite chỉ có <b>một kết nối</b>, không có phiên nào khác để bị chặn: lab không quan sát được khoá, thời gian chờ hay <code>lock_timeout</code> kích hoạt. Mọi mức khoá ở bảng trên lấy từ tài liệu PostgreSQL 17, không phải số đo. Phần 4 dùng hai phiên trên server thật để xem phiên giữ khoá và phiên bị chặn.</p>
</div>

## 8. Kế hoạch khôi phục: hai mức, chạy lại được

Migration nào cũng cần trả lời trước: "nếu hỏng thì lùi thế nào?". `06-migration-rollback.sql` có **hai mức**, chọn mức nhẹ nhất đủ giải quyết sự cố:

| Mức | Làm gì | Khi nào | Mất dữ liệu? |
|---|---|---|---|
| 1 · Nới lỏng | Gỡ CHECK, `DROP NOT NULL`, giữ cột | App cũ lỗi vì chưa gửi biển số | Không — sau này chạy lại B4–B6 |
| 2 · Gỡ hẳn cột | Sao lưu biển số sang bảng riêng rồi `DROP COLUMN` | Bỏ hẳn tính năng | Có, nếu không sao lưu |

Tệp rollback không nằm trong `node run.mjs p01` (runner bỏ qua `*-rollback.sql`); chạy tay sau phần 1, hai lần để chứng minh chạy lại được:

```bash
node run.mjs p01 p01/06-migration-rollback.sql p01/06-migration-rollback.sql
```

Lần chạy thứ hai không lỗi, chỉ có thông báo bỏ qua nhờ `IF EXISTS` / `IF NOT EXISTS`:

```text
NOTICE:  constraint "ck_drivers_vehicle_plate_format" of relation "drivers" does not exist, skipping
…
NOTICE:  relation "drivers_vehicle_plate_backup" already exists, skipping
…
 so_bien_so_da_sao_luu
-----------------------
                     7
(1 row)
```

Ba chi tiết đáng học trong tệp này:

- **`DROP NOT NULL` không có `IF EXISTS` cho cột**, nên lab đặt nó (và bước sao lưu) trong khối `DO` — đoạn mã thủ tục chạy một lần, ở đây chỉ để kiểm cột còn tồn tại.
- **Sao lưu dùng `ON CONFLICT … DO UPDATE`** (UPSERT, Phần 2 giải thích): chạy lại không nhân đôi — vẫn đúng 7 dòng.
- **Kiểm chứng sau rollback**: cột và ràng buộc `ck_drivers_vehicle_plate%` đã biến mất. `npm test` chạy chuỗi rollback → rollback → `06` → `07` để chứng minh migration làm lại được sau khi lùi.

Thứ tự triển khai cũng phải lùi theo: trước khi gỡ cột ở mức 2, phải gỡ mọi phiên bản app còn đọc/ghi `vehicle_plate` — nếu không, chính rollback là thay đổi phá vỡ.

## 9. Lỗi thường gặp và đánh đổi

- **Gộp expand và contract vào một lần triển khai.** Migration chạy lúc app mới khởi động trong khi app cũ còn chạy ⇒ app cũ lỗi ngay. Tách thành hai lần phát hành.
- **Quên dòng sinh ra trong giai đoạn expand.** Tài xế 8007 đăng ký sau B1 cũng cần backfill. Kiểm `WHERE vehicle_plate IS NULL` ngay trước bước contract.
- **Bỏ quên `NOT VALID`.** Dữ liệu mới vẫn bị chặn nên mọi thứ "trông ổn", nhưng dữ liệu cũ chưa được rà và `SET NOT NULL` không dựa vào được. Đưa truy vấn `NOT convalidated` vào kiểm tra sau triển khai.
- **Đổi tên hoặc đổi kiểu cột trực tiếp.** `RENAME COLUMN` phá vỡ mọi truy vấn dùng tên cũ; `ALTER COLUMN TYPE` thường viết lại bảng dưới khoá nặng. Làm theo kiểu cột mới → ghi cả hai → backfill → chuyển đọc → gỡ cột cũ (bài tập 3).
- **Rollback chưa từng chạy thử.** Kịch bản khôi phục chỉ đáng tin khi đã chạy trên dữ liệu giống thật, và chạy được hai lần liên tiếp.

Đánh đổi: expand–contract tốn **nhiều lần triển khai** và một khoảng thời gian schema "lửng" (cột có nhưng chưa bắt buộc). Đổi lại, không lúc nào hệ thống phải dừng và bước nào cũng có đường lùi rẻ. Với bảng nhỏ, được phép bảo trì có báo trước, làm gộp trong một bước vẫn có thể chấp nhận được — nhưng đó phải là quyết định có ý thức.

## 10. Bài tập

**Bài 1.** Trên schema vừa dựng từ `00 → 02`, thêm cột, điền biển số cho mọi tài xế rồi chạy `SET NOT NULL` **trực tiếp**, không qua CHECK phụ. Bật `client_min_messages = debug1` như lab. Dòng `DEBUG` khác gì so với mục 6, và trên bảng 60.000 tài xế điều đó có nghĩa gì?

<details><summary><b>Lời giải</b></summary>
<p>Tệp thử (không có trong lab, đã chạy bằng <code>node run.mjs p01/00-reset.sql p01/01-schema.sql p01/02-fixture-valid.sql &lt;tệp&gt;</code>):</p>
<pre><code class="language-sql">SET search_path TO toinoi;
ALTER TABLE drivers ADD COLUMN vehicle_plate text;
UPDATE drivers SET vehicle_plate = '59X1-000.0' || (code - 8000);
SET client_min_messages = debug1;
ALTER TABLE drivers ALTER COLUMN vehicle_plate SET NOT NULL;
SET client_min_messages = notice;</code></pre>
<p>Kết quả: <code>UPDATE 6</code>, rồi <code>DEBUG:  verifying table "drivers"</code> và <code>ALTER TABLE</code>. Lệnh vẫn thành công, nhưng thay vì "existing constraints … are sufficient to prove…", server báo nó đang <b>kiểm tra cả bảng</b>. Trên bảng lớn, lần quét đó diễn ra trong khi giữ <code>ACCESS EXCLUSIVE</code> — mọi truy vấn đọc ghi <code>drivers</code> phải chờ tới khi quét xong. CHECK phụ chuyển lần quét sang <code>VALIDATE</code> với khoá nhẹ.</p>
</details>

**Bài 2.** Sự cố xảy ra sau bước contract và đội vận hành đã chạy rollback **mức 2** (cột bị gỡ). Hôm sau quyết định bật lại tính năng. Viết SQL đưa cột và biển số quay lại từ bản sao lưu, rồi nói các bước còn lại để trở về trạng thái sau `06`.

<details><summary><b>Lời giải</b></summary>
<p>Đã chạy bằng <code>node run.mjs p01 p01/06-migration-rollback.sql &lt;tệp&gt;</code>:</p>
<pre><code class="language-sql">SET search_path TO toinoi;
ALTER TABLE drivers ADD COLUMN vehicle_plate text;
UPDATE drivers AS d
SET vehicle_plate = b.vehicle_plate
FROM drivers_vehicle_plate_backup AS b
WHERE b.driver_id = d.id;</code></pre>
<p>Kết quả <code>UPDATE 7</code>, cả 7 tài xế có lại biển số (8006 là <code>43A-678.90</code> đã sửa, vì bản sao lưu lấy dữ liệu sau migration). Đây chính là B1 + B3 với nguồn là bảng sao lưu thay cho file tuyển dụng. Tiếp theo: kiểm <code>WHERE vehicle_plate IS NULL</code> (tài xế đăng ký trong lúc cột đã bị gỡ cần nguồn khác), rồi B4–B6. Nếu không có bản sao lưu, dữ liệu đã mất cùng <code>DROP COLUMN</code> — lý do mức 1 là mặc định.</p>
</details>

**Bài 3.** Tới Nơi muốn đổi tên `drivers.full_name` thành `display_name`. Lập kế hoạch expand–contract (không dùng `RENAME COLUMN` một phát), ghi rõ phiên bản app nào chạy được ở mỗi bước và đường lùi.

<details><summary><b>Lời giải</b></summary>
<p>(1) Expand: thêm <code>display_name text</code> cho phép NULL — v1 không ảnh hưởng. (2) Triển khai v2 ghi <b>cả hai</b> cột, vẫn đọc <code>full_name</code>. (3) Backfill theo lô với điều kiện <code>display_name IS NULL</code>. (4) CHECK <code>IS NOT NULL</code> NOT VALID → VALIDATE → <code>SET NOT NULL</code>, thêm CHECK không rỗng như <code>ck_drivers_full_name_not_blank</code>. (5) v3 đọc <code>display_name</code>, vẫn ghi cả hai. (6) Hết v1/v2: v4 ngừng ghi <code>full_name</code>, bỏ NOT NULL của nó. (7) Contract: sao lưu nếu cần rồi <code>DROP COLUMN full_name</code>. Bước 1–6 lùi được bằng cách quay về app trước; chỉ bước 7 mất dữ liệu. <code>RENAME COLUMN</code> một phát làm mọi app cũ hỏng ngay.</p>
</details>

## 11. Đọc tiếp

- [Tài liệu PostgreSQL 17 — ALTER TABLE](https://www.postgresql.org/docs/17/sql-altertable.html): `NOT VALID`, `VALIDATE CONSTRAINT`, `SET NOT NULL`, khi nào viết lại bảng.
- [Tài liệu PostgreSQL 17 — Explicit Locking](https://www.postgresql.org/docs/17/explicit-locking.html): bảng xung đột giữa các mức khoá.
- [Tài liệu PostgreSQL 17 — `lock_timeout`](https://www.postgresql.org/docs/17/runtime-config-client.html).
- Lab: `examples/toi-noi-postgresql/p01/06-migration.sql`, `06-migration-rollback.sql`, kết quả thật trong `p01/OUTPUT.txt`.

Bài tiếp theo: [PG-010: Thực hành — schema chuyến xe có kiểm tra ràng buộc](/articles/postgresql-thuc-hanh-schema-chuyen-xe).
