---
slug: postgresql-insert-update-delete-returning
title: "Thêm, sửa, xoá và đọc kết quả với RETURNING: một chuyến đi trọn vòng đời"
summary: "Hai tài xế Tới Nơi cùng bấm nhận một chuyến, app phải báo đúng người thắng. Bài dùng INSERT, UPDATE, DELETE … RETURNING và thẻ lệnh để đưa một chuyến qua 7 trạng thái, giải thích UPDATE 0 và giữ fixture chạy lại được."
category: postgresql
level: intermediate
tags: [postgresql, pg-006, dml, insert, update, delete, returning, toi-noi]
language: vi
---

09:00 ngày 03/11, chị Phạm Thu Dung đặt một chuyến xe máy ở Hà Nội. Ngay khi bấm «Đặt», app phải hiện **mã chuyến** — con số do database cấp, app không tự biết. Một phút sau, tài xế 8003 bấm «Nhận»; một giây sau nữa, tài xế 8001 cũng bấm «Nhận» cùng chuyến. App tài xế 8001 phải báo «chuyến đã có người nhận», không được cướp chuyến của 8003 mà không ai hay.

Cả hai yêu cầu quy về một câu hỏi: sau mỗi câu lệnh ghi, **database cho ứng dụng biết những gì?** Bài này đưa một chuyến đi hết vòng đời bằng `INSERT`, `UPDATE`, `DELETE` kèm `RETURNING`, đọc thẻ lệnh để biết bao nhiêu dòng thực sự đổi, và giữ bộ dữ liệu thử chạy lại được bao nhiêu lần cũng ra cùng kết quả.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-006 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-create-table-constraints">PG-005 · CREATE TABLE và ràng buộc dữ liệu</a>. Lab: <code>examples/toi-noi-postgresql/p01</code> (tệp <code>00-reset.sql</code>, <code>04-lifecycle.sql</code>). Sau bài này, bạn thực hiện được vòng đời một chuyến bằng INSERT/UPDATE/DELETE … RETURNING trên fixture có thể chạy lại, và hiểu một UPDATE trả 0 dòng nói lên điều gì.</p>
</div>

## 1. Database trả lời gì sau một câu ghi

`INSERT`, `UPDATE`, `DELETE` thuộc nhóm **DML** (Data Manipulation Language — câu lệnh thao tác dữ liệu). Mỗi câu chạy xong, PostgreSQL trả về hai thứ:

- **Thẻ lệnh** (command tag): tên lệnh kèm số dòng — `INSERT 0 1`, `UPDATE 1`, `DELETE 2`. Driver của các ngôn ngữ lập trình thường đưa con số này ra dưới tên «rows affected». Số `0` giữa `INSERT 0 1` là chỗ của OID cũ, theo tài liệu PostgreSQL 17 nó luôn là 0; con số sau mới là số dòng.
- **Kết quả của `RETURNING`** (nếu có): các dòng giống một câu SELECT, tính trên đúng những dòng câu lệnh vừa ghi.

| Câu lệnh | Thẻ lệnh nói gì | `RETURNING` trả về |
|---|---|---|
| `INSERT` | Số dòng đã thêm | Dòng như đã được thêm — gồm cả giá trị database tự điền (`DEFAULT`, IDENTITY) |
| `UPDATE` | Số dòng đã sửa; **0 không phải lỗi** | Giá trị **mới** của dòng sau khi sửa |
| `DELETE` | Số dòng đã xoá | Nội dung dòng **vừa bị xoá** |

`RETURNING` nhận danh sách cột hoặc biểu thức như phần chọn cột của SELECT (ví dụ `completed_at - requested_at`). Ở PostgreSQL 17, `RETURNING` của UPDATE chỉ thấy giá trị sau khi sửa; muốn biết giá trị cũ thì phải đọc trước hoặc thiết kế câu lệnh khác.

Vì sao không «INSERT rồi SELECT lại»? Câu SELECT thứ hai phải tìm lại đúng dòng vừa thêm và tốn thêm một vòng gọi mạng; `RETURNING` trả lời ngay trong cùng câu lệnh.

## 2. Fixture chạy lại được

Lab dựng mọi thứ trong schema riêng `toinoi`. Tệp `00-reset.sql` bắt đầu bằng:

```sql
DROP SCHEMA IF EXISTS toinoi CASCADE;
CREATE SCHEMA toinoi;
```

Chạy chuỗi cho bài này:

```bash
cd examples/toi-noi-postgresql
node run.mjs p01/00-reset.sql p01/01-schema.sql p01/02-fixture-valid.sql p01/03-fixture-invalid.sql p01/04-lifecycle.sql
```

Lần đầu, `DROP SCHEMA IF EXISTS` chỉ báo `NOTICE:  schema "toinoi" does not exist, skipping`. Chạy lại `00` trong cùng phiên (ví dụ chuỗi `00 → 01 → 02 → 04` hai lần liên tiếp), tệp này xoá sạch kết quả lần chạy trước:

```text
NOTICE:  drop cascades to 6 other objects
DETAIL:  drop cascades to table cities
…
drop cascades to table trip_events
DROP SCHEMA
```

và lượt thứ hai in ra **đúng** những mã chuyến, mốc thời gian như lượt đầu. Ba điều làm được việc đó:

1. **Dựng lại từ trống**, kể cả sequence của IDENTITY, nên mã chuyến lại bắt đầu từ 100000.
2. **UUID và mốc thời gian ghi sẵn** (`b0000000-…-000000000001` cho chuyến của chị Dung) thay cho `gen_random_uuid()`, `now()` mà ứng dụng thật dùng — để so kết quả từng byte.
3. **Chạy lại mà quên reset thì báo lỗi ngay**, không nhân đôi dữ liệu mà không ai hay: chạy `02` lần nữa mà không qua `00`, câu đầu tiên đã gặp `duplicate key value violates unique constraint "pk_cities"`. Khoá chính cố định biến «chạy nhầm hai lần» thành lỗi nhìn thấy được.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỉ chạy trên database lab</span>
<p><code>DROP SCHEMA … CASCADE</code> xoá mọi đối tượng trong schema. Lab cô lập trong <code>toinoi</code> để không đụng <code>public</code>; đừng bao giờ chạy nó trên database có dữ liệu thật.</p>
</div>

## 3. INSERT … RETURNING: khách đặt chuyến

Câu đặt chuyến chỉ ghi những gì đã biết. `status` lấy `DEFAULT 'requested'`, `driver_id`/`accepted_at` để NULL, `code` do IDENTITY cấp:

```sql
INSERT INTO trips (id, kind, city_id, customer_id, fare, requested_at, updated_at)
VALUES ('b0000000-0000-4000-8000-000000000001', 'ride', 2, 'c0000000-0000-4000-8000-000000000004',
        54000, '2026-11-03 09:00:00+07', '2026-11-03 09:00:00+07')
RETURNING code, kind, status, driver_id, fare, requested_at;
```

```text
  code  | kind |  status   | driver_id | fare  |      requested_at
--------+------+-----------+-----------+-------+------------------------
 100029 | ride | requested |           | 54000 | 2026-11-03 09:00:00+07
(1 row)

INSERT 0 1
```

App nhận ngay mã 100029 và trạng thái `requested` do database điền. `id` do «ứng dụng» sinh sẵn — cách làm phổ biến khi app cần biết định danh trước khi gửi xuống database; các câu sau tìm chuyến theo `id` này.

**Vì sao 100029 chứ không phải 100015?** Fixture hợp lệ có 15 chuyến, mã 100000–100014. Nhưng trước đó tệp `03-fixture-invalid.sql` ([PG-005](/articles/postgresql-create-table-constraints)) đã chạy: một câu INSERT hợp lệ bị `ROLLBACK` và 13 câu INSERT vào `trips` bị ràng buộc NOT NULL/FK/CHECK từ chối — tổng 14 lần IDENTITY cấp số, từ 100015 tới 100028. Tài liệu `CREATE SEQUENCE` nói rõ `nextval` không bao giờ bị hoàn tác, nên sequence không dùng được khi cần dãy số liền mạch. Các câu lỗi vì sai kiểu (`'45.000đ'`, ngày 30/02) và câu ghi tay vào cột IDENTITY (`428C9`) thì không tiêu số: chúng hỏng khi đọc giá trị, trước lúc xin số mới.

Chứng minh ngược: chạy `00 → 01 → 02 → 04`, bỏ `03`, chuyến của chị Dung nhận mã 100015. Cùng lý do, dòng `trip_events` đầu tiên của chuyến (thêm bằng `INSERT … RETURNING id, status, occurred_at`) có `id = 2`: số 1 đã bị câu vi phạm `ck_trip_events_status` ở `03` tiêu mất.

Bài học: `code` là để **tra cứu** («cho tôi xem chuyến 100029»), không phải để đếm số chuyến hay kiểm tra «có bị mất chuyến nào không».

## 4. UPDATE có điều kiện: chuyển đúng bước

Mỗi bước của vòng đời là một UPDATE, và điều kiện `WHERE` gồm hai phần: **dòng nào** (`id = …`) và **đang ở trạng thái nào** (`status = <trạng thái trước>`):

```sql
UPDATE trips
SET status = 'searching', updated_at = '2026-11-03 09:00:05+07'
WHERE id = 'b0000000-0000-4000-8000-000000000001' AND status = 'requested'
RETURNING code, status, updated_at;
```

```text
  code  |  status   |       updated_at
--------+-----------+------------------------
 100029 | searching | 2026-11-03 09:00:05+07
(1 row)

UPDATE 1
```

Tài xế 8003 nhận chuyến — chỉ thành công khi chuyến vẫn `searching`. Một câu đổi bốn cột, `RETURNING` xác nhận tài xế và mốc nhận:

```sql
UPDATE trips
SET status      = 'accepted',
    driver_id   = 'd0000000-0000-4000-8000-000000008003',
    accepted_at = '2026-11-03 09:01:00+07',
    updated_at  = '2026-11-03 09:01:00+07'
WHERE id = 'b0000000-0000-4000-8000-000000000001' AND status = 'searching'
RETURNING code, status, driver_id, accepted_at;
```

```text
  code  |  status  |              driver_id               |      accepted_at
--------+----------+--------------------------------------+------------------------
 100029 | accepted | d0000000-0000-4000-8000-000000008003 | 2026-11-03 09:01:00+07
(1 row)

UPDATE 1
```

Sau mỗi UPDATE, lab thêm một dòng `trip_events` ghi lại trạng thái mới (hồ sơ Tới Nơi: mỗi mũi tên vòng đời = một UPDATE `trips` + một dòng lịch sử). Tài xế cũng chuyển `busy`, bằng chính mẫu có điều kiện: `UPDATE drivers SET status = 'busy' WHERE code = 8003 AND status = 'online'` → `UPDATE 1`.

Ràng buộc của PG-005 vẫn canh mọi bước: chỉ đổi `status` sang `accepted` mà không gán tài xế thì gặp `ck_trips_assigned_has_driver` (đã thử trên chuyến 100013). Điều kiện `WHERE` lo **thứ tự** chuyển trạng thái; CHECK lo **tính nhất quán** của từng dòng.

## 5. UPDATE 0 nghĩa là gì

Một giây sau, tài xế 8001 bấm nhận cùng chuyến. App gửi đúng câu nhận chuyến, chỉ khác tài xế:

```sql
UPDATE trips
SET status      = 'accepted',
    driver_id   = 'd0000000-0000-4000-8000-000000008001',
    accepted_at = '2026-11-03 09:01:01+07',
    updated_at  = '2026-11-03 09:01:01+07'
WHERE id = 'b0000000-0000-4000-8000-000000000001' AND status = 'searching'
RETURNING code, status, driver_id;
```

```text
 code | status | driver_id
------+--------+-----------
(0 rows)

UPDATE 0
```

Không có lỗi nào. Tài liệu `UPDATE` ghi rõ số dòng bằng 0 nghĩa là không dòng nào được sửa và **không bị coi là lỗi**. Chuyến đã `accepted` nên điều kiện `status = 'searching'` không còn đúng, không dòng nào khớp `WHERE`. Ứng dụng phải tự đọc số dòng (hoặc thấy `RETURNING` rỗng) để báo «chuyến đã có người nhận». Code chỉ bắt exception sẽ tưởng đây là thành công.

**UPDATE 0 có hai nguyên nhân** khác hẳn nhau: không có dòng nào mang `id` đó, hoặc dòng có nhưng đang ở trạng thái khác. Muốn phân biệt, đọc lại theo `id` (câu thử thêm, chạy sau `04`):

```sql
SELECT code, status, driver_id
FROM trips
WHERE id = 'b0000000-0000-4000-8000-000000000001';
```

```text
  code  |  status   |              driver_id
--------+-----------+--------------------------------------
 100029 | completed | d0000000-0000-4000-8000-000000008003
(1 row)
```

Có dòng ⇒ trạng thái đã đi tiếp; 0 dòng ⇒ `id` sai. Đọc lại chỉ để **giải thích** cho người dùng, không dùng để quyết định ghi — quyết định đã nằm trong điều kiện `WHERE`.

### Nếu bỏ điều kiện trạng thái

Cùng câu nhận chuyến nhưng chỉ có `WHERE code = 100012` — chuyến 100012 của fixture đang ở trạng thái `accepted` do tài xế 8001 nhận. Thử trong `BEGIN … ROLLBACK` để không để lại dấu vết (giao dịch học ở Phần 4):

```sql
BEGIN;
UPDATE trips
SET status      = 'accepted',
    driver_id   = 'd0000000-0000-4000-8000-000000008003',
    accepted_at = '2026-11-03 09:45:00+07',
    updated_at  = '2026-11-03 09:45:00+07'
WHERE code = 100012
RETURNING code, status, driver_id, accepted_at;
ROLLBACK;
```

```text
  code  |  status  |              driver_id               |      accepted_at
--------+----------+--------------------------------------+------------------------
 100012 | accepted | d0000000-0000-4000-8000-000000008003 | 2026-11-03 09:45:00+07
(1 row)

UPDATE 1
```

`UPDATE 1`, không ràng buộc nào phản đối: dòng mới vẫn hợp lệ theo mọi CHECK. Tài xế 8001 đang chạy chuyến thì bị thay bằng 8003. Đây là **lost update** (cập nhật bị mất): một lần ghi đè lên kết quả của lần ghi trước mà không ai hay. Ràng buộc chỉ kiểm từng dòng có hợp lệ không, không kiểm dòng có đi đúng **bước** không — đó là việc của điều kiện trong `WHERE`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Lab chạy trên PGlite, một kết nối, các câu lệnh đi tuần tự: tài xế 8001 bấm «sau 1 giây» chứ không bấm cùng khoảnh khắc. Hai phiên chạy song song thật sự, khoá dòng và mức cô lập là việc của Phần 4, trên server thật.</p>
</div>

## 6. Hoàn thành: CHECK chặn, rồi gộp hai việc thành một câu

Ba bước `arriving`, `picked_up`, `in_transit` theo đúng mẫu trên. Tới bước hoàn thành, lab cố tình quên `completed_at`:

```sql
UPDATE trips SET status = 'completed', updated_at = '2026-11-03 09:31:00+07'
WHERE id = 'b0000000-0000-4000-8000-000000000001' AND status = 'in_transit';
```

```text
ERROR:  new row for relation "trips" violates check constraint "ck_trips_completed_at_by_status"
DETAIL:  Failing row contains (b0000000-0000-4000-8000-000000000001, 100029, ride, 2, …, completed, 54000, 2026-11-03 09:00:00+07, 2026-11-03 09:01:00+07, null, 2026-11-03 09:31:00+07).
```

Đây chính là sự cố mở bài PG-005, bị chặn ngay ở câu lệnh. Câu đúng ghi `completed_at` **và** dòng lịch sử trong một câu lệnh duy nhất:

```sql
WITH done AS (
  UPDATE trips
  SET status       = 'completed',
      completed_at = '2026-11-03 09:31:00+07',
      updated_at   = '2026-11-03 09:31:00+07'
  WHERE id = 'b0000000-0000-4000-8000-000000000001' AND status = 'in_transit'
  RETURNING id, status, completed_at
)
INSERT INTO trip_events (trip_id, status, occurred_at)
SELECT id, status, completed_at FROM done
RETURNING id, status, occurred_at;
```

```text
 id |  status   |      occurred_at
----+-----------+------------------------
  8 | completed | 2026-11-03 09:31:00+07
(1 row)

INSERT 0 1
```

Ý tưởng: kết quả `RETURNING` của UPDATE được dùng như một bảng tạm tên `done`, và câu INSERT đọc từ đó. Cú pháp `WITH` (CTE) sẽ giải thích ở Phần 2; điều cần nắm ở đây là **một câu lệnh thành công hoặc thất bại trọn vẹn**, nên không thể có chuyến `completed` mà thiếu dòng lịch sử. Nếu UPDATE không khớp dòng nào, `done` rỗng và INSERT thêm 0 dòng — chạy lại câu này lần nữa ra `INSERT 0 0`. Thẻ lệnh in ra là của câu INSERT ngoài cùng.

Các bước trước dùng hai câu riêng (UPDATE rồi INSERT), mỗi câu tự commit: nếu câu thứ hai lỗi, chuyến đã đổi trạng thái mà không có lịch sử. Gộp nhiều câu thành một đơn vị là chủ đề giao dịch của Phần 4.

Đọc lại kết quả cuối:

```text
  code  |  status   | fare  |      requested_at      |      accepted_at       |      completed_at      | tong_thoi_gian
--------+-----------+-------+------------------------+------------------------+------------------------+----------------
 100029 | completed | 54000 | 2026-11-03 09:00:00+07 | 2026-11-03 09:01:00+07 | 2026-11-03 09:31:00+07 | 00:31:00
(1 row)

 id |   status   |      occurred_at
----+------------+------------------------
  2 | requested  | 2026-11-03 09:00:00+07
  3 | searching  | 2026-11-03 09:00:05+07
  4 | accepted   | 2026-11-03 09:01:00+07
  5 | arriving   | 2026-11-03 09:01:30+07
  6 | picked_up  | 2026-11-03 09:07:00+07
  7 | in_transit | 2026-11-03 09:07:20+07
  8 | completed  | 2026-11-03 09:31:00+07
(7 rows)
```

Bảy trạng thái, bảy dòng lịch sử, chuyến dài 31 phút. Câu UPDATE lỗi ở bước hoàn thành không để lại gì: không đổi `trips`, không tiêu số `trip_events`.

## 7. DELETE … RETURNING, và vì sao Tới Nơi hầu như không xoá chuyến

Chuyến thật bị huỷ vẫn được giữ với trạng thái `cancelled` — cần cho đối soát, khiếu nại, thống kê tỷ lệ huỷ. `DELETE` chỉ dành cho dữ liệu tạo nhầm. Phần 2 của `04` tạo một bản nháp bưu kiện (mã 100030), huỷ nó bằng `WHERE … AND status IN ('requested', 'searching')`, rồi thử xoá:

```sql
DELETE FROM trips
WHERE id = 'b0000000-0000-4000-8000-000000000002' AND status = 'cancelled';
```

```text
ERROR:  update or delete on table "trips" violates foreign key constraint "fk_trip_events_trip" on table "trip_events"
DETAIL:  Key (id)=(b0000000-0000-4000-8000-000000000002) is still referenced from table "trip_events".
```

Khoá ngoại bảo vệ lịch sử: còn dòng `trip_events` trỏ tới thì không xoá được chuyến. Muốn xoá thật thì xoá con trước, và `RETURNING` cho biết đã xoá những gì:

```sql
DELETE FROM trip_events
WHERE trip_id = 'b0000000-0000-4000-8000-000000000002'
RETURNING status, occurred_at;
```

```text
  status   |      occurred_at
-----------+------------------------
 requested | 2026-11-03 09:10:00+07
 cancelled | 2026-11-03 09:10:40+07
(2 rows)

DELETE 2
```

```sql
DELETE FROM trips
WHERE id = 'b0000000-0000-4000-8000-000000000002' AND status = 'cancelled'
RETURNING code, kind, status, fare;
```

```text
  code  |  kind  |  status   | fare
--------+--------+-----------+-------
 100030 | parcel | cancelled | 25000
(1 row)

DELETE 1
```

`RETURNING` của DELETE là bản ghi cuối cùng của dòng — có thể lưu vào nhật ký trước khi nó biến mất. Điều kiện `AND status = 'cancelled'` là lưới an toàn: gõ nhầm `id` của một chuyến đang chạy thì câu xoá trả `DELETE 0`.

Cuối tệp:

```text
 ma_lon_nhat | so_chuyen | so_cap_gan_nhat
-------------+-----------+-----------------
      100029 |        16 |          100030
(1 row)
```

16 chuyến (15 fixture + 100029), mã lớn nhất 100029, nhưng sequence `trips_code_seq` (do IDENTITY tự tạo) đã cấp tới 100030. Chuyến kế tiếp nhận 100031 (đã thử một INSERT trong giao dịch rồi huỷ); số 100030 không bao giờ được dùng lại.

## 8. Lỗi thường gặp và đánh đổi

- **UPDATE/DELETE quên `WHERE`.** Câu đúng cú pháp, sửa mọi dòng. Thử trong giao dịch rồi huỷ trên lab: `UPDATE trips SET fare = 0;` trả `UPDATE 16`. Luôn chạy SELECT với cùng `WHERE` trước, và đọc số dòng sau.
- **Bỏ qua số dòng.** Coi «không có exception» là thành công thì tài xế 8001 được báo nhận chuyến dù `UPDATE 0`.
- **Đọc rồi mới ghi.** `SELECT status` thấy `searching` rồi `UPDATE … WHERE id = …` không kèm trạng thái: giữa hai câu, trạng thái có thể đã đổi. Đưa điều kiện vào chính câu UPDATE.
- **Coi mã IDENTITY là liên tục.** Lỗi ràng buộc, rollback, xoá đều để lại khoảng trống trong dãy mã. Đếm chuyến bằng `count(*)`, không bằng hiệu hai mã.
- **`RETURNING *` cho tiện.** Trả mọi cột, và đổi theo khi bảng thêm cột. Liệt kê cột cần dùng.
- **Xoá cứng hay giữ trạng thái huỷ.** Xoá làm bảng gọn nhưng mất dấu vết và đụng khoá ngoại; giữ `cancelled` tốn chỗ nhưng trả lời được câu hỏi đối soát. Với chuyến, Tới Nơi chọn giữ.

## 9. Bài tập

**Bài 1.** Khách huỷ chuyến 100013 (đồ ăn, đang `searching`). Viết câu UPDATE huỷ chuyến chỉ khi chưa có ai nhận, trả về `code, kind, status, updated_at`. Chạy hai lần liên tiếp thì lần hai ra gì? Câu đó áp lên chuyến 100012 (`accepted`) ra gì?

<details><summary><b>Lời giải</b></summary><pre><code class="language-sql">UPDATE trips
SET status = 'cancelled', updated_at = '2026-11-03 08:40:00+07'
WHERE code = 100013 AND status IN ('requested', 'searching')
RETURNING code, kind, status, updated_at;</code></pre>
<p>Đã chạy trên lab sau <code>00 → 04</code>: lần một trả một dòng <code>100013 | food | cancelled | 2026-11-03 08:40:00+07</code>, <code>UPDATE 1</code>. Lần hai <code>(0 rows)</code>, <code>UPDATE 0</code> — chuyến đã <code>cancelled</code>; bấm huỷ hai lần không gây hại. Áp lên 100012 cũng <code>UPDATE 0</code>: chuyến đã có tài xế không huỷ được theo luật «chưa ai nhận». Ứng dụng thật còn thêm một dòng <code>trip_events</code> trạng thái <code>cancelled</code>, nên gộp như mục 6.</p></details>

**Bài 2.** Tài xế bấm «Đã đón khách» và app nhận `UPDATE 0` từ câu `… SET status = 'picked_up' … WHERE id = $1 AND status = 'arriving'`. Liệt kê các nguyên nhân có thể và cách app phân biệt chúng.

<details><summary><b>Lời giải</b></summary><p>(1) <code>id</code> không tồn tại — lỗi dữ liệu phía app. (2) Chuyến đã <code>picked_up</code> hoặc đi xa hơn — thường do bấm hai lần hay mạng gửi lại; coi là đã xong. (3) Chuyến đã <code>cancelled</code> — báo tài xế. (4) Chuyến còn <code>accepted</code> — tài xế bỏ qua bước «đang tới đón»; app phải gửi đúng bước trước. Phân biệt bằng <code>SELECT status FROM trips WHERE id = $1</code>: 0 dòng là (1), còn lại xem <code>status</code>. Câu đọc này chỉ để giải thích, không thay điều kiện trong UPDATE.</p></details>

**Bài 3.** Chạy `node run.mjs p01/00-reset.sql p01/01-schema.sql p01/02-fixture-valid.sql p01/04-lifecycle.sql` (bỏ `03`). Chuyến của chị Dung nhận mã nào, dòng `trip_events` đầu tiên có `id` bao nhiêu? Giải thích khác biệt với `npm run p01`.

<details><summary><b>Lời giải</b></summary><p>Mã <strong>100015</strong> và <code>id = 1</code> (đã chạy). Không có <code>03</code> thì không câu nào tiêu số của hai sequence trước <code>04</code>. Khi có <code>03</code>: 14 số của <code>trips</code> (1 câu hợp lệ bị rollback + 13 câu bị ràng buộc từ chối) và 1 số của <code>trip_events</code> (câu vi phạm <code>ck_trip_events_status</code>) đã bị tiêu, nên ra 100029 và <code>id = 2</code>. Sequence không hoàn tác số đã cấp; đừng gắn ý nghĩa nghiệp vụ cho tính liên tục của mã.</p></details>

## 10. Đọc tiếp

- Trước đó: [PG-005 · CREATE TABLE và ràng buộc dữ liệu](/articles/postgresql-create-table-constraints).
- Fixture hợp lệ/sai và bằng chứng kiểm tra gom thành bản bàn giao: [PG-010](/articles/postgresql-thuc-hanh-schema-chuyen-xe).
- Bảng `trip_events` tách khỏi `trips` và quan hệ chuyến – sự kiện: [PG-008](/articles/postgresql-mo-hinh-quan-he-chuan-hoa).
- Tài liệu PostgreSQL 17: [Returning Data from Modified Rows](https://www.postgresql.org/docs/17/dml-returning.html), [INSERT](https://www.postgresql.org/docs/17/sql-insert.html), [UPDATE](https://www.postgresql.org/docs/17/sql-update.html), [DELETE](https://www.postgresql.org/docs/17/sql-delete.html), [CREATE SEQUENCE](https://www.postgresql.org/docs/17/sql-createsequence.html).

Bài tiếp theo: [PG-007 · SELECT đầu tiên: lọc, sắp xếp và giới hạn](/articles/postgresql-select-filter-order-limit)
