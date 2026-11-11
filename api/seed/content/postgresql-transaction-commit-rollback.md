---
slug: postgresql-transaction-commit-rollback
title: "Transaction, COMMIT và ROLLBACK: chuyến và sự kiện cùng thành công hoặc cùng huỷ"
summary: "CSKH Tới Nơi mở một chuyến vừa đặt và thấy màn lịch sử trống: server đã ghi trips rồi chết trước khi ghi trip_events. Học giao dịch, COMMIT, ROLLBACK, SAVEPOINT và cách app biết dữ liệu thật sự đã lưu."
category: postgresql
level: beginner
tags: [postgresql, pg-031, transaction, commit, rollback, savepoint, autocommit, toi-noi]
language: vi
---

Khi khách bấm «Đặt xe», API của Tới Nơi ghi **hai** thứ: một dòng trong `trips` và sự kiện `requested` trong `trip_events` — nhật ký trạng thái mà màn lịch sử của CSKH đọc. Một tối, tổng đài viên mở chuyến của một khách đang gọi khiếu nại: chuyến có, cước có, nhưng lịch sử **trống trơn**. Server đã chạy xong câu `INSERT` thứ nhất, rồi container bị khởi động lại trước câu thứ hai.

Hai câu lệnh đúng cú pháp, mỗi câu đều thành công hoặc thất bại gọn gàng — vấn đề là *giữa* chúng. Bài này dùng giao dịch (transaction) để biến hai câu thành **một đơn vị**: cùng được lưu hoặc cùng biến mất, kể cả khi lỗi, khi mất kết nối, khi lập trình viên quên `COMMIT`.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-031 · Phần 4/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/postgresql-thuc-hanh-toi-uu-truy-van">PG-030 · Thực hành: tối ưu truy vấn bằng bằng chứng</a> (bài cuối Phần 3) và <a href="/articles/postgresql-insert-update-delete-returning">PG-006 · INSERT, UPDATE, DELETE và RETURNING</a>. Lab: <code>examples/toi-noi-postgresql/p04</code> (tệp <code>31-transaction-commit-rollback.sql</code>; thí nghiệm <code>thu-nghiem/pg031-nhan-chuyen-nguyen-tu-now.sql</code>). Sau bài này, bạn chứng minh được việc ghi chuyến và sự kiện cùng thành công hoặc cùng bị huỷ.</p>
</div>

## 1. Dữ liệu và cách chạy

Phần 4 chạy trên **PostgreSQL 18.1 thật** (Phần 1–2 dùng PGlite 17.5), mỗi tệp trên một database mới chép từ nền `p04`: 10 chuyến đang chờ mã 400101–400110, sequence mã chuyến đang ở **400200**. Mỗi bài có **hai phiên** — hai kết nối psql riêng: **A** là API đặt chuyến, **B** là người quan sát (CSKH hoặc một request khác).

```bash
cd examples/toi-noi-postgresql
node pg/run-pg.mjs p04 --only 31
node pg/run-pg.mjs --nen p04/lab.json:p04 p04/thu-nghiem/pg031-nhan-chuyen-nguyen-tu-now.sql
```

Runner gửi từng bước theo thứ tự trong tệp — đó là **lịch chạy** trong các bảng T1, T2… dưới đây; kết quả chép từ `p04/OUTPUT.txt`. Dễ nhầm: `id` (uuid đặt tay) của mỗi chuyến mới lệch một đơn vị so với **mã** `code` do sequence cấp — chuyến `…400201` mang mã 400200.

## 2. Giao dịch là gì

**Giao dịch** gom nhiều câu lệnh thành một thao tác «tất cả hoặc không có gì». Tài liệu PostgreSQL tóm lại: giao dịch "bundles multiple steps into a single, all-or-nothing operation". Ba lệnh điều khiển:

- `BEGIN` mở giao dịch.
- `COMMIT` chốt: mọi thay đổi trở thành vĩnh viễn và phiên khác bắt đầu thấy.
- `ROLLBACK` huỷ: mọi thay đổi từ `BEGIN` biến mất như chưa từng có.

Bốn tính chất **ACID**: nguyên tử (atomicity), nhất quán (consistency — ràng buộc vẫn đúng sau giao dịch), cô lập (isolation — giao dịch dở không lộ ra), bền vững (durability — đã báo COMMIT thì sập máy cũng không mất; WAL thuộc Phần 5). Bài này tập trung vào **A** và một phần **I**; PG-032, PG-034 đi sâu phần cô lập.

Không gõ `BEGIN` thì sao? PostgreSQL vẫn bọc **mỗi câu lệnh** trong một giao dịch ngầm và tự `COMMIT` nếu câu đó thành công. Đó là lý do bug ở phần mở bài xảy ra: hai câu `INSERT` chạy riêng là **hai** giao dịch, câu thứ nhất đã commit trước khi câu thứ hai bắt đầu.

## 3. Hai câu, một giao dịch: B không thấy gì cho tới khi A COMMIT

Phiên A ghi chuyến và sự kiện trong một giao dịch. Giữa chừng, phiên B đếm:

```sql
-- Phiên A
BEGIN;
INSERT INTO trips (id, kind, city_id, customer_id, fare, pickup_area_id)
VALUES ('a4000000-0000-4000-8000-000000400201', 'ride', 1, 'c4000000-0000-4000-8000-000000000011', 45000, 111)
RETURNING code;
INSERT INTO trip_events (trip_id, status)
VALUES ('a4000000-0000-4000-8000-000000400201', 'requested');

-- Phiên B: đếm chuyến và sự kiện của chuyến đó
SELECT (SELECT count(*) FROM trips WHERE id = 'a4000000-…-000000400201') AS chuyen,
       (SELECT count(*) FROM trip_events WHERE trip_id = 'a4000000-…-000000400201') AS su_kien;
```

| Bước | Phiên A (API) | Phiên B (CSKH) | Kết quả |
|---|---|---|---|
| T1 | `BEGIN` | | `BEGIN` |
| T2 | `INSERT INTO trips … RETURNING code` | | `400200`, `INSERT 0 1` |
| T3 | `INSERT INTO trip_events …` | | `INSERT 0 1` |
| T4 | | đếm chuyến, sự kiện | **`0 \| 0`** — không chờ, không thấy |
| T5 | `SELECT t.requested_at = e.occurred_at` | | `t` |
| T6 | `COMMIT` | | `COMMIT` |
| T7 | | đếm lại | **`1 \| 1`** |

Ở T4, dữ liệu đã nằm trong bảng nhưng chưa commit, nên chỉ A thấy. B không bị chặn, chỉ được trả lời theo trạng thái đã commit — PG-032 giải thích B «nhìn» bằng cách nào. PostgreSQL không cho đọc dữ liệu chưa commit của phiên khác ở mức cô lập nào; `READ UNCOMMITTED`, theo tài liệu, hoạt động như Read Committed.

T5 là một chi tiết nhỏ nhưng hay gây thắc mắc: `requested_at` của chuyến và `occurred_at` của sự kiện **bằng nhau tuyệt đối** dù chèn bằng hai câu khác nhau. Cả hai lấy mặc định `now()`, mà `now()` trả về **thời điểm bắt đầu giao dịch**, đứng yên suốt giao dịch. Muốn giờ thật đang trôi, dùng `clock_timestamp()` (bài tập 3).

## 4. Một câu lỗi làm hỏng cả giao dịch

Phiên A ghi chuyến mới, rồi ghi sự kiện với trạng thái gõ sai `'dang_tim'`:

```sql
BEGIN;
INSERT INTO trips (…) VALUES ('a4000000-…-000000400202', 'ride', 1, …, 52000, 112) RETURNING code;   -- 400201
INSERT INTO trip_events (trip_id, status, occurred_at)
VALUES ('a4000000-0000-4000-8000-000000400202', 'dang_tim', TIMESTAMPTZ '2026-10-31 01:00:00+07');
SELECT 1;
COMMIT;
```

```text
ERROR:  new row for relation "trip_events" violates check constraint "ck_trip_events_status"
DETAIL:  Failing row contains (102, a4000000-0000-4000-8000-000000400202, dang_tim, 2026-10-31 01:00:00+07).
SELECT 1;
ERROR:  current transaction is aborted, commands ignored until end of transaction block
COMMIT;
ROLLBACK
```

Ba điều cần nhớ:

1. Sau câu lỗi, giao dịch ở trạng thái **hỏng** (aborted). Mọi câu tiếp theo, kể cả `SELECT 1`, bị từ chối với SQLSTATE **`25P02`** cho tới khi giao dịch kết thúc.
2. Gõ `COMMIT` lúc này **không báo lỗi**. Server trả về thẻ lệnh (command tag) **`ROLLBACK`** — tức là đã huỷ.
3. Câu `INSERT INTO trips` thành công trước đó cũng bị huỷ theo: phiên B đếm chuyến `…400202` ra **0**.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">«COMMIT xong» chưa chắc là đã lưu</span>
<p>Ứng dụng viết theo kiểu «gửi COMMIT, không ném lỗi thì báo khách đặt thành công» sẽ báo sai đúng trong trường hợp này. Đọc kết quả của COMMIT (thẻ lệnh hoặc trạng thái giao dịch mà driver trả về), và để lỗi của câu bên trong giao dịch dừng luồng xử lý thay vì bị nuốt. Lỗi giữa giao dịch thì xử lý bằng <code>ROLLBACK</code> chủ động, rồi quyết định thử lại hay báo lỗi.</p>
</div>

## 5. SAVEPOINT: huỷ một phần, giữ phần còn lại

Đôi khi chỉ muốn bỏ đúng câu vừa lỗi. **`SAVEPOINT`** đặt một điểm lưu bên trong giao dịch; `ROLLBACK TO SAVEPOINT` lùi về đó, giữ mọi thứ trước nó, và đưa giao dịch đang hỏng về trạng thái dùng tiếp được.

```sql
BEGIN;
INSERT INTO trips (…) VALUES ('a4000000-…-000000400203', 'parcel', 1, …, 38000, 113) RETURNING code;  -- 400202
SAVEPOINT truoc_su_kien;
INSERT INTO trip_events (trip_id, status, occurred_at) VALUES ('a4000000-…-000000400203', 'dang_tim', …);  -- 23514
ROLLBACK TO SAVEPOINT truoc_su_kien;
INSERT INTO trip_events (trip_id, status) VALUES ('a4000000-0000-4000-8000-000000400203', 'requested');
COMMIT;
```

Server trả `ROLLBACK` cho dòng `ROLLBACK TO` (chỉ là tên lệnh — giao dịch vẫn sống), `INSERT 0 1` cho sự kiện đúng, rồi `COMMIT`. Phiên B thấy `400202 | 1`: chuyến có, đúng một sự kiện.

Biến **`ON_ERROR_ROLLBACK`** của psql (`\set ON_ERROR_ROLLBACK on`) **tự** đặt savepoint ngầm trước mỗi câu trong giao dịch và lùi về đó khi câu lỗi. Lab chạy lại kịch bản lỗi với biến này: chuyến **400203**, câu `'dang_tim'` lỗi, câu `'requested'` vẫn chạy, không còn `25P02`, `COMMIT` trả `COMMIT`. Đây là tính năng của **psql**, không phải server: app dùng driver không có nó, và mỗi savepoint là một giao dịch con có chi phí.

## 6. Autocommit của psql

Mặc định psql ở chế độ **autocommit bật**: câu nào không nằm sau `BEGIN` tự commit ngay. `\set AUTOCOMMIT off` đảo lại: psql tự gửi `BEGIN` ngầm trước câu đầu tiên và **chờ bạn** `COMMIT` hoặc `ROLLBACK`.

| Bước | Phiên A (`\set AUTOCOMMIT off`, không gõ BEGIN) | Phiên B | Kết quả |
|---|---|---|---|
| T1 | `INSERT` chuyến (mã `400204`) rồi `INSERT` sự kiện | | `INSERT 0 1` × 2 |
| T2 | `SELECT pg_current_xact_id_if_assigned() IS NOT NULL` | | `t` — đang trong giao dịch |
| T3 | | đếm chuyến `…400205` | **0** |
| T4 | `COMMIT` | đếm lại | `COMMIT`, rồi **1** |

`pg_current_xact_id_if_assigned()` trả mã giao dịch nếu đã được cấp (sau câu ghi đầu tiên), NULL nếu chưa — PG-032 dùng nhiều. Bật lại autocommit, cùng câu kiểm `IS NULL` ra `t`: câu lẻ không để giao dịch nào mở. Tắt autocommit mà thoát phiên chưa commit thì mất việc đã làm. Driver và ORM mỗi loại có mặc định riêng; đọc tài liệu thư viện bạn dùng thay vì đoán.

## 7. Mã chuyến nhảy cóc: sequence không quay lui

Đến đây, bài đã tạo các chuyến mã 400200, 400202, 400203, 400204. Mã **400201** biến mất cùng giao dịch hỏng ở mục 4 — nhưng sequence không trả nó lại. Tài liệu giải thích: để các giao dịch song song không phải chờ nhau, giá trị `nextval` đã cấp **không được thu hồi** khi giao dịch huỷ, nên sequence không dùng được cho dãy số «không lỗ hổng». Hai mã 400205, 400206 ở mục 8 cũng bị «đốt» như vậy.

Với `trips.code` đó là chấp nhận được: mã dùng để gọi tên chuyến, không dùng để đếm. Nếu nghiệp vụ đòi số liên tục (số hoá đơn VAT chẳng hạn) thì phải thiết kế riêng, không dựa vào sequence.

## 8. Mất kết nối giữa giao dịch và phiên «quên COMMIT»

Server API có thể chết, mạng có thể đứt, sau khi đã ghi mà trước khi `COMMIT`. Lab giả lập bằng cách cho B «rút dây» A với `pg_terminate_backend(pid, 5000)` — gửi tín hiệu kết thúc backend của A và chờ tối đa 5 giây tới khi nó thoát hẳn:

| Bước | Phiên A | Phiên B | Kết quả |
|---|---|---|---|
| T1 | `BEGIN` | | |
| T2 | `INSERT INTO trips … RETURNING code` | | `400205` |
| T3 | `INSERT INTO trip_events …` | | `INSERT 0 1` |
| T4 | | `SELECT pg_terminate_backend(pid, 5000) … 'lab-A'` | `t`; nhật ký server: `[A] FATAL: terminating connection due to administrator command` |
| T5 | `COMMIT` | | psql thoát mã 2, «server closed the connection unexpectedly» |
| T6 | | đếm chuyến, sự kiện của `…400206` | **`0 \| 0`** |
| T7 | kết nối **mới**, `SELECT EXISTS (…) AS da_luu` | | `f` |

Không có «nửa chuyến»: khi backend chết, giao dịch đang mở của nó bị huỷ trọn. Phiên A ở T7 là kết nối mới — `SET search_path` phải chạy lại, giao dịch cũ không còn.

Trường hợp khó hơn là mất kết nối **sau khi** gửi `COMMIT`: server có thể đã commit xong rồi mới đứt mạng, client chỉ thấy lỗi kết nối. Lab chưa tái hiện được tình huống này một cách tất định (cần cắt mạng đúng sau lúc server commit) — đây là lập luận, không phải kết quả chạy: client **không biết** kết quả, nên đừng mặc định «thất bại, gửi lại». Cách an toàn là kết nối lại và đọc trạng thái như T7, hoặc gửi kèm idempotency key để gửi lại không tạo chuyến trùng (PG-039).

Lỗi ngược lại là **quên COMMIT**: code mở giao dịch, ghi, rồi đi gọi một API ngoài và treo. Phiên đó ở trạng thái «idle in transaction», giữ khoá và giữ snapshot cũ làm VACUUM không dọn được phiên bản chết (thí nghiệm hai phiên ở [PG-026](/articles/postgresql-index-only-scan-visibility-map)). Tham số `idle_in_transaction_session_timeout` cắt các phiên như vậy:

| Bước | Phiên A | Phiên B | Kết quả |
|---|---|---|---|
| T1 | `SET idle_in_transaction_session_timeout = '200ms'`; `BEGIN` | | |
| T2 | `INSERT INTO trips … RETURNING code` | | `400206` |
| T3 | (ngồi yên) | `SELECT pg_sleep(0.5)` | nhật ký: `[A] FATAL: terminating connection due to idle-in-transaction timeout` |
| T4 | `COMMIT` | | mất kết nối |
| T5 | | đếm chuyến `…400207` | **0** |

Mặc định tham số này bằng 0 (tắt). Có thể đặt riêng cho vai trò mà app dùng (`ALTER ROLE … SET …`) để không ảnh hưởng phiên quản trị; giá trị 200 ms chỉ để lab chạy nhanh. PG-033 so nó với `lock_timeout`, `statement_timeout`, `transaction_timeout`.

Bất biến cuối bài (B kiểm): đếm sự kiện `requested` theo chuyến mã ≥ 400200 ra đúng bốn dòng `400200 | 1`, `400202 | 1`, `400203 | 1`, `400204 | 1`; câu đếm chuyến **không** có sự kiện `requested` (`NOT EXISTS`) trên cả 44 chuyến của bảng ra **0**. Đó là bằng chứng bàn giao của bài: bốn chuyến được lưu trọn vẹn, ba lần huỷ (lỗi, rút dây, quên COMMIT) không để lại gì ngoài ba mã bị đốt.

## 9. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Chuyến có mà lịch sử trống | Hai câu chạy autocommit riêng | Bọc trong một `BEGIN … COMMIT` |
| App báo «đặt thành công» mà DB không có | Không kiểm kết quả COMMIT (thẻ `ROLLBACK`) | Kiểm thẻ/trạng thái hoặc để driver ném lỗi; `ROLLBACK` chủ động khi có lỗi |
| Sau một lỗi, mọi câu báo `25P02` | Giao dịch đã hỏng | `ROLLBACK` (hoặc `ROLLBACK TO SAVEPOINT`) |
| Script psql chạy «êm» nhưng thiếu dữ liệu | `ON_ERROR_ROLLBACK on` nuốt câu lỗi | Chỉ bật khi gõ tay; script dùng `ON_ERROR_STOP` |
| Mã chuyến nhảy cóc | Sequence không quay lui khi huỷ | Bình thường; không dùng sequence cho số liên tục |
| Phiên «idle in transaction» hàng phút | Gọi mạng/chờ người dùng trong giao dịch | Gọi ngoài giao dịch; đặt `idle_in_transaction_session_timeout` |

## 10. Đánh đổi

- **Giao dịch rộng hay hẹp.** Gom càng nhiều câu càng an toàn về nguyên tử, nhưng giao dịch càng dài càng giữ khoá lâu (PG-033) và giữ snapshot cũ. Giữ giao dịch ngắn: tính toán, gọi API ngoài trước; trong giao dịch chỉ đọc–ghi database.
- **Thứ không rollback được.** SMS đã gửi, tiền đã trừ ở hệ khác không quay lui theo `ROLLBACK`. Đừng gọi chúng giữa giao dịch; ghi «việc cần làm» vào bảng trong cùng giao dịch rồi xử lý sau (outbox, PG-039).
- **SAVEPOINT.** Hữu ích khi thật sự muốn bỏ qua một phần (thử chèn, lỗi thì làm cách khác). Dùng cho mọi câu như `ON_ERROR_ROLLBACK` thì tốn giao dịch con và dễ che lỗi.

## 11. Bài tập

Lời giải chạy thật trong thí nghiệm `p04/thu-nghiem/pg031-nhan-chuyen-nguyen-tu-now.sql` (không có trong lab chính).

**Bài 1.** Tài xế 9101 nhận chuyến 400101: `UPDATE trips` (trạng thái `accepted`, gán tài xế, `accepted_at`) và `INSERT` sự kiện `accepted`, trong một giao dịch. Câu `INSERT` gõ nhầm `trip_id` thành `…400199`. Sau `COMMIT`, chuyến 400101 ở trạng thái gì, có tài xế không, mấy sự kiện?

<details><summary><b>Lời giải</b></summary>
<p>Câu <code>INSERT</code> vi phạm khoá ngoại (<code>23503 fk_trip_events_trip</code>, chuyến …400199 không tồn tại), giao dịch hỏng, <code>COMMIT</code> trả <code>ROLLBACK</code>. UPDATE cũng bị huỷ: phiên B thấy <code>requested | t | 1</code> (chưa có tài xế, 1 sự kiện). Làm lại với đúng <code>trip_id</code>:</p>
<pre><code class="language-sql">BEGIN;
UPDATE trips
SET status = 'accepted', driver_id = 'd4000000-0000-4000-8000-000000009101',
    accepted_at = now(), version = version + 1, updated_at = now()
WHERE code = 400101 AND status = 'requested';
INSERT INTO trip_events (trip_id, status)
VALUES ('a4000000-0000-4000-8000-000000400101', 'accepted');
COMMIT;</code></pre>
<p>B thấy <code>accepted | f | 2 | t</code>: có tài xế, 2 sự kiện, giờ sự kiện mới nhất bằng đúng <code>accepted_at</code> (cùng <code>now()</code> của giao dịch). Không bao giờ tồn tại trạng thái «đã có tài xế mà lịch sử chưa ghi».</p>
</details>

**Bài 2.** Không gõ `BEGIN`, chạy một câu `INSERT` hai dòng sự kiện cho chuyến 400102: dòng 1 `'searching'` hợp lệ, dòng 2 `'dang_tim'` sai. Dòng 1 có được lưu không?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">INSERT INTO trip_events (trip_id, status)
VALUES ('a4000000-0000-4000-8000-000000400102', 'searching'),
       ('a4000000-0000-4000-8000-000000400102', 'dang_tim');</code></pre>
<p>Câu lỗi <code>23514 ck_trip_events_status</code>; số sự kiện của chuyến trước và sau đều là <code>1</code>. Một câu lệnh lẻ cũng là một giao dịch (ngầm) nên cũng nguyên tử: hoặc cả hai dòng, hoặc không dòng nào. Giao dịch tường minh chỉ cần khi có <b>nhiều</b> câu phải đi cùng nhau.</p>
</details>

**Bài 3.** Chứng minh trong một giao dịch, `now()` đứng yên còn `statement_timestamp()` và `clock_timestamp()` thì không.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">BEGIN;
CREATE TEMP TABLE moc AS SELECT now() AS n, clock_timestamp() AS c;
SELECT pg_sleep(0.3);
SELECT now() = n AS now_dung_yen,
       statement_timestamp() &gt; n + interval '0.25 second' AS statement_moi,
       clock_timestamp() &gt; c + interval '0.25 second' AS dong_ho_chay
FROM moc;
ROLLBACK;</code></pre>
<p>Kết quả <code>t | t | t</code>. <code>now()</code> (= <code>transaction_timestamp()</code>) là lúc giao dịch bắt đầu; <code>statement_timestamp()</code> là lúc câu lệnh hiện tại tới server; <code>clock_timestamp()</code> đổi cả trong một câu. Dùng <code>now()</code> khi muốn mọi dòng của một thao tác mang cùng mốc (như chuyến và sự kiện của nó), <code>clock_timestamp()</code> khi đo thời gian.</p>
</details>

## 12. Đọc tiếp

- [PostgreSQL 18 · Transactions (tutorial)](https://www.postgresql.org/docs/18/tutorial-transactions.html) — BEGIN/COMMIT, giao dịch ngầm, savepoint.
- [PostgreSQL 18 · SAVEPOINT](https://www.postgresql.org/docs/18/sql-savepoint.html).
- [PostgreSQL 18 · psql](https://www.postgresql.org/docs/18/app-psql.html) — biến `AUTOCOMMIT`, `ON_ERROR_ROLLBACK`.
- [PostgreSQL 18 · Sequence Manipulation Functions](https://www.postgresql.org/docs/18/functions-sequence.html) — vì sao sequence có lỗ hổng.
- [PostgreSQL 18 · Client Connection Defaults](https://www.postgresql.org/docs/18/runtime-config-client.html) — `idle_in_transaction_session_timeout`, `transaction_timeout`.
- [PostgreSQL 18 · Date/Time Functions](https://www.postgresql.org/docs/18/functions-datetime.html) — `now()`, `statement_timestamp()`, `clock_timestamp()`.
- [PostgreSQL 18 · Server Signaling Functions](https://www.postgresql.org/docs/18/functions-admin.html) — `pg_terminate_backend`.
- Lab: `p04/31-transaction-commit-rollback.sql`, `p04/OUTPUT.txt`; thí nghiệm (không có trong lab chính) `p04/thu-nghiem/pg031-nhan-chuyen-nguyen-tu-now.sql`.

Bài tiếp theo: [PG-032: MVCC và snapshot — mỗi phiên nhìn thấy gì?](/articles/postgresql-mvcc-snapshot)
