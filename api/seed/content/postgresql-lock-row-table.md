---
slug: postgresql-lock-row-table
title: "Khoá dòng, khoá bảng và thời gian chờ: ai đang chặn ai?"
summary: "Lúc 01:00, tổng đài Tới Nơi báo màn sửa cước bị treo dù database không lỗi gì. Tạo lại hàng chờ khoá trên PostgreSQL 18.1, đọc pg_stat_activity, pg_locks, pg_blocking_pids để tìm phiên gốc, rồi đặt thời gian chờ có chủ đích."
category: postgresql
level: beginner
tags: [postgresql, pg-033, lock, row-lock, pg-locks, lock-timeout, advisory-lock, toi-noi]
language: vi
---

01:00, nhân viên tổng đài Tới Nơi bấm «Lưu» trên màn sửa cước chuyến 400104 và vòng xoay cứ quay mãi. Người thứ hai thử sửa cùng chuyến, cũng treo. Trong khi đó màn theo dõi chuyến vẫn hiện cước bình thường, log ứng dụng không có lỗi, CPU database gần như không làm gì.

Không có gì hỏng. Một phiên khác đã sửa chuyến đó trong một giao dịch **chưa kết thúc**, và mọi người ghi sau phải xếp hàng. PG-032 cho thấy người *đọc* không phải chờ; bài này trả lời phần còn lại: người *ghi* chờ cái gì, nhìn hàng chờ ở đâu, gỡ thế nào, và giới hạn thời gian chờ ra sao.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-033 · Phần 4/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/postgresql-mvcc-snapshot">PG-032 · MVCC và snapshot</a>. Lab: <code>examples/toi-noi-postgresql/p04</code> (tệp <code>33-khoa-dong-khoa-bang.sql</code>; thí nghiệm <code>thu-nghiem/pg033-go-roi-phien-giu-khoa.sql</code>, <code>thu-nghiem/pg033-transaction-timeout.sql</code>). Sau bài này, bạn nhận diện được phiên giữ khoá và phiên bị chặn, phân biệt khoá dòng với khoá bảng, và chọn đúng loại thời gian chờ.</p>
</div>

## 1. Dữ liệu và cách chạy

Lab Phần 4 chạy trên **PostgreSQL 18.1 thật** (Phần 1–2 dùng PGlite 17.5). Mỗi tệp chạy trên một database mới chép từ nền nhỏ: 10 chuyến đang chờ nhận 400101…400110 (400101 cước 31.000 đ, mỗi mã sau thêm 1.000 đ), `version = 1`.

```bash
cd examples/toi-noi-postgresql
node pg/run-pg.mjs p04 --only 33
node pg/run-pg.mjs --nen p04/lab.json:p04 p04/thu-nghiem/pg033-go-roi-phien-giu-khoa.sql
```

Mỗi **phiên** (session) A, B, C… là một kết nối psql riêng. Bước rơi vào chờ khoá được ghi `⏳`; kết quả của nó in ở dòng `✔` khi được nhả — các bảng lịch dưới đây xếp theo thứ tự xảy ra thật. Hàm `lab_cho()`, `lab_khoa()` của lab đọc `pg_stat_activity`/`pg_locks` và in nhãn phiên thay PID. Kết quả chép từ `p04/OUTPUT.txt`.

## 2. Hai người cùng sửa một chuyến

**Khoá** (lock) ngăn hai giao dịch cùng sửa một thứ; PostgreSQL tự lấy khoá khi chạy lệnh, không cần viết `LOCK`. Lịch §1 của lab:

| Bước | Phiên | Lệnh | Kết quả |
|---|---|---|---|
| T1 | A | `BEGIN; UPDATE trips SET fare = fare + 1000, version = version + 1 … WHERE code = 400104 RETURNING fare;` | 35000 — A giữ khoá dòng, chưa commit |
| T2 | B | `SELECT fare FROM trips WHERE code = 400104;` | **34000 ngay lập tức** — đọc bản đã commit |
| T3 | B | `UPDATE trips SET fare = fare + 500, version = version + 1 … WHERE code = 400104;` | **⏳ chờ** — Lock: transactionid, bị chặn bởi A |
| T4 | A | `COMMIT;` | A xong |
| T5 | B | (bước T3 chạy tiếp) | ✔ **35500**, `version` = 3 |

Người đọc (T2) không chờ: khoá dòng **không chặn SELECT thường**. Người ghi thứ hai chờ tới khi **giao dịch** của A kết thúc, không phải tới khi câu UPDATE của A xong (nó xong từ T1). Và B cộng 500 lên **bản mới** 35000: ở Read Committed, người chờ đọc lại dòng sau khi được khoá — PG-034 so chuyện này với Repeatable Read. Hệ quả cho màn sửa cước: nếu ứng dụng mở giao dịch lúc người dùng *mở* màn và chỉ COMMIT khi bấm «Lưu», cả thời gian người dùng ngồi nhìn màn hình là thời gian khoá bị giữ.

## 3. Ba nguồn để biết ai chặn ai

Giữa T3 và T4, phiên C đọc **`pg_stat_activity`** (mỗi kết nối một dòng: trạng thái, đang chờ gì, câu lệnh cuối) cùng hàm **`pg_blocking_pids(pid)`** (danh sách PID đang chặn một phiên):

```text
 phien |     trang_thai      |         cho         | bi_chan_boi |                           cau_lenh
-------+---------------------+---------------------+-------------+--------------------------------------------------------------
 A     | idle in transaction | Client: ClientRead  |             | UPDATE trips SET fare = fare + 1000, version = version + 1,
 B     | active              | Lock: transactionid | A           | UPDATE trips SET fare = fare + 500, version = version + 1, u
(2 rows)
```

A **`idle in transaction`** — mở giao dịch mà không chạy gì, `Client: ClientRead` là backend đợi ứng dụng gửi câu tiếp: hình dạng của «màn sửa cước chưa bấm Lưu». B `active` nhưng chờ `Lock: transactionid` — chờ giao dịch của A. Cột câu lệnh là câu **cuối cùng** phiên gửi, không nhất thiết là câu đang giữ khoá.

Rồi **`pg_locks`** — danh sách khoá đang giữ (`granted = t`) và đang xin (`f`):

```text
 phien |     loai      |  doi_tuong   |      che_do      | da_cap
-------+---------------+--------------+------------------+--------
 A     | relation      | trips        | RowExclusiveLock | t
 A     | transactionid | xid của A    | ExclusiveLock    | t
 B     | relation      | trips        | RowExclusiveLock | t
 B     | transactionid | xid của A    | ShareLock        | f
 B     | transactionid | xid của B    | ExclusiveLock    | t
 B     | tuple         | trips (0,34) | ExclusiveLock    | t
(6 rows)
```

Không dòng nào nói «A khoá chuyến 400104». Theo tài liệu, thông tin khoá dòng nằm **trên đĩa** chứ không trong bộ nhớ, nên thường không hiện ở `pg_locks`: A khoá dòng bằng cách ghi xid của mình vào `xmax` của phiên bản dòng (PG-032 — `xmax ≠ 0` không có nghĩa là dòng đã bị xoá). Mỗi giao dịch giữ `ExclusiveLock` trên chính xid của nó tới khi kết thúc; B chờ A bằng cách **xin `ShareLock` trên xid của A** (`da_cap = f`), chỉ được cấp khi A COMMIT hoặc ROLLBACK. Dòng `tuple trips (0,34)` là chỗ đứng đầu hàng của B trên đúng phiên bản dòng (ctid, PG-024). Hai khoá `relation trips RowExclusiveLock` là **khoá bảng** mà mọi UPDATE/INSERT/DELETE lấy; chúng không xung đột nhau, nên sửa các chuyến *khác nhau* vẫn song song được.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Hỏi pg_blocking_pids(), đừng tự nối pg_locks</span>
<p><code>pg_locks</code> không cho biết thứ tự trong hàng chờ; tài liệu khuyên dùng <code>pg_blocking_pids()</code>, hàm này tính cả «chặn mềm» (phiên đang chờ và đứng trước trong hàng). Nó cần khoá độc quyền bảng khoá trong chốc lát — dùng để chẩn đoán, đừng gọi liên tục.</p>
</div>

## 4. Bốn mức khoá dòng và bẫy khoá ngoại

Muốn «đọc rồi sửa» an toàn, ứng dụng khoá dòng ngay lúc đọc bằng `SELECT … FOR <mức>`. Từ yếu tới mạnh:

| Mức | Ai tự lấy | Chặn |
|---|---|---|
| `FOR KEY SHARE` | Kiểm khoá ngoại khi chèn dòng con | DELETE, đổi khoá, `FOR UPDATE` |
| `FOR SHARE` | — | Mọi UPDATE/DELETE, `FOR NO KEY UPDATE`, `FOR UPDATE` |
| `FOR NO KEY UPDATE` | UPDATE không đổi cột khoá | UPDATE/DELETE, `FOR SHARE`, `FOR NO KEY UPDATE`, `FOR UPDATE` — **không** chặn `FOR KEY SHARE` |
| `FOR UPDATE` | DELETE, UPDATE đổi cột có unique index dùng được cho khoá ngoại | Tất cả, kể cả `FOR KEY SHARE` |

Không mức nào chặn `SELECT` thường. Lab §2: A `BEGIN; SELECT … WHERE code = 400105 FOR UPDATE;` — dòng không bị sửa nhưng `xmax` đã là xid của A. B chèn một sự kiện cho chuyến đó (`INSERT INTO trip_events …`) thì **⏳ chờ** tới khi A COMMIT. Lặp lại với `FOR NO KEY UPDATE`: B chèn sự kiện **ngay**, còn `SELECT … FOR NO KEY UPDATE` của B vẫn **⏳ chờ** và sau khi A huỷ chuyến rồi COMMIT thì nhận `400105 | cancelled` — đọc lại bản mới.

Lý do: chèn dòng con phải kiểm khoá ngoại `trip_events.trip_id → trips.id`, tức lấy `FOR KEY SHARE` trên chuyến, và `FOR UPDATE` xung đột với nó. Khi không xoá dòng, không đổi khoá chính, dùng **`FOR NO KEY UPDATE`**: hai người sửa vẫn không chồng nhau mà việc ghi lịch sử chuyến không bị chặn.

## 5. Khoá bảng: DDL chặn cả người đọc

Mỗi lệnh lấy một **khoá bảng** (table-level lock): `SELECT` lấy `ACCESS SHARE`, UPDATE/INSERT/DELETE lấy `ROW EXCLUSIVE`, phần lớn `ALTER TABLE` cùng `DROP`, `TRUNCATE`, `CLUSTER`, `VACUUM FULL` lấy **`ACCESS EXCLUSIVE`** — chế độ duy nhất chặn `SELECT` thường. Lab §3: A `BEGIN; ALTER TABLE trips ADD COLUMN ghi_chu text;` (chưa commit); B `SELECT count(*) FROM trips;` **⏳ chờ** (Lock: relation; `lab_khoa()`: A `AccessExclusiveLock t`, B `AccessShareLock f`); A `ROLLBACK` ⇒ B ✔ 40 và cột `ghi_chu` không còn. DDL của PostgreSQL nằm trong giao dịch như dữ liệu (ROLLBACK huỷ cả `ADD COLUMN`), và khoá giữ tới **hết giao dịch** — migration bọc nhiều bước trong một giao dịch chặn người đọc từ bước DDL đầu tiên tới COMMIT.

Chiều ngược lại còn tệ hơn: DDL chờ một báo cáo đang đọc, và trong lúc nó chờ, mọi `SELECT` đến sau xếp hàng sau nó. [PG-024](/articles/heap-ctid-va-primary-key) đã chạy đúng cảnh này với `CLUSTER` (thí nghiệm `p03/thu-nghiem/pg024-cluster-hang-doi-khoa.sql`). Cách phòng là cho DDL một **`lock_timeout`** ngắn để nó bỏ cuộc thay vì đứng chặn hàng:

```sql
-- A: BEGIN; SELECT count(*) FROM trips;   -- báo cáo dài, giữ AccessShareLock
-- B:
SET lock_timeout = '300ms';
ALTER TABLE trips ADD COLUMN ghi_chu text;
```

```text
-- [lab] [B] ⏳ đã chờ khoá (Lock: relation) — bị chặn bởi [A] trước khi xong
ERROR:  canceling statement due to lock timeout
-- [lab] SQLSTATE 55P03
```

Ngay sau đó C đếm ra 40 không chờ dù A vẫn mở giao dịch: B đã rời hàng. Migration thật bọc vòng thử lại quanh câu DDL đó.

## 6. Bốn loại «hết giờ» và cái chúng huỷ

Cả bốn mặc định `0` (tắt). Tài liệu khuyên không đặt `lock_timeout`, `statement_timeout`, `transaction_timeout` ở `postgresql.conf` (ảnh hưởng mọi phiên) — đặt theo phiên, theo role, hoặc `SET LOCAL`.

| Tham số | Giới hạn | Khi hết giờ |
|---|---|---|
| `lock_timeout` | **Mỗi lần** chờ một khoá | Huỷ câu lệnh, `55P03`; phiên còn sống, giao dịch đang mở (nếu có) hỏng |
| `statement_timeout` | Tổng thời gian một câu (chờ khoá + chạy) | Huỷ câu lệnh, `57014` |
| `idle_in_transaction_session_timeout` | Thời gian ngồi yên trong giao dịch | **Cắt kết nối** (FATAL), giao dịch ROLLBACK (PG-031) |
| `transaction_timeout` (có từ PG 17) | Tổng thời gian giao dịch, kể cả khi đang làm việc | **Cắt kết nối**, giao dịch ROLLBACK |

Lab §4: A giữ khoá chuyến 400106; B đặt `statement_timeout = '300ms'` rồi UPDATE cùng chuyến:

```text
-- [lab] [B] ⏳ đã chờ khoá (Lock: transactionid) — bị chặn bởi [A] trước khi xong
ERROR:  canceling statement due to statement timeout
CONTEXT:  while updating tuple (0,36) in relation "trips"
-- [lab] SQLSTATE 57014
```

Câu kế của B (`SELECT 1`) gặp `25P02 current transaction is aborted`: giao dịch đã hỏng, phải ROLLBACK (PG-031). `SELECT pg_sleep(1)` không cần khoá cũng nhận `57014`. Chuyến 400106 cuối cùng là **37000** — chỉ có +1000 của A.

Thí nghiệm `pg033-transaction-timeout.sql`: A đặt `transaction_timeout = '300ms'`, sửa chuyến 400101 rồi chạy `pg_sleep(1)` — đang làm việc chứ không ngồi yên — và nhật ký server ghi:

```text
[A] FATAL:  terminating connection due to transaction timeout
[A] STATEMENT:  SELECT pg_sleep(1);
```

B đọc lại: 400101 vẫn **31000**. Tài liệu lưu ý: với `statement_timeout` khác 0, đặt `lock_timeout` bằng hoặc lớn hơn nó là vô nghĩa.

## 7. Gỡ hàng chờ lúc 01:00

Thí nghiệm `pg033-go-roi-phien-giu-khoa.sql` (không có trong lab chính) dựng lại sự cố với chuyến 400108 (38.000 đ), dùng SQL thường thay hàm lab: A sửa +1000 rồi ngồi yên; B sửa +200 (tự commit); C sửa +500. Phiên trực D chạy:

```sql
SELECT a.application_name, a.state, a.wait_event_type, a.wait_event,
       coalesce((SELECT string_agg(b.application_name, ',' ORDER BY b.application_name)
                 FROM pg_stat_activity AS b
                 WHERE b.pid = ANY (pg_blocking_pids(a.pid))), '') AS chan_boi
FROM pg_stat_activity AS a
WHERE a.application_name IN ('lab-A', 'lab-B', 'lab-C')
ORDER BY a.application_name;
```

```text
 application_name |        state        | wait_event_type |  wait_event   | chan_boi
------------------+---------------------+-----------------+---------------+----------
 lab-A            | idle in transaction | Client          | ClientRead    |
 lab-B            | active              | Lock            | transactionid | lab-A
 lab-C            | active              | Lock            | tuple         | lab-B
(3 rows)
```

C không chờ thẳng A mà chờ **B**: B đang giữ khoá `tuple` (chỗ đứng đầu hàng của dòng đó), C xếp sau. Hàng chờ là một chuỗi, nên tìm **gốc** = phiên đang chặn người khác mà bản thân không bị ai chặn:

```sql
SELECT a.application_name, a.state
FROM pg_stat_activity AS a
WHERE cardinality(pg_blocking_pids(a.pid)) = 0
  AND EXISTS (SELECT 1 FROM pg_stat_activity AS w WHERE a.pid = ANY (pg_blocking_pids(w.pid)));
```

Kết quả: `lab-A | idle in transaction`. Trên production thêm `pid`, `usename`, `now() - xact_start` để biết ai, mở giao dịch bao lâu.

Hai cách gỡ khác nhau ở cái bị huỷ. **Gỡ sai chỗ** — D chạy `pg_cancel_backend(pid)` cho lab-B: B nhận ✔ `57014 canceling statement due to user request`, chuyển `idle`; đọc lại hàng chờ thì `lab-C | active | Lock | transactionid | lab-A` — C chỉ tiến lên một bậc, gốc vẫn còn. **Gỡ đúng chỗ** — D chạy `pg_terminate_backend(pid, 5000)` cho lab-A: C ✔ ngay với `fare` **38500**; khi A quay lại gửi `COMMIT` thì **mất kết nối**. 38500 = 38000 + 500: +1000 của A bị ROLLBACK, +200 của B đã bị huỷ — cả hai phải làm lại. `pg_cancel_backend` huỷ **câu đang chạy**; `pg_terminate_backend` cắt **kết nối**, nhả mọi khoá. Đó là chữa cháy có hậu quả; cách bền là không để giao dịch ngồi yên và đặt `idle_in_transaction_session_timeout` cho role ứng dụng.

## 8. Advisory lock: khoá theo con số do ứng dụng đặt nghĩa

**Advisory lock** (khoá tư vấn) là khoá trên một con số PostgreSQL không gắn với bảng hay dòng nào — ứng dụng tự quy ước nghĩa (ở đây: mã chuyến). Lab §5: A `SELECT pg_advisory_lock(400107)`; B `pg_try_advisory_lock(400107)` → `f` (thử, không chờ), rồi `pg_advisory_lock(400107)` → **⏳ chờ** (Lock: advisory; `pg_locks` có `advisory 400107 ExclusiveLock`: A `t`, B `f`); A `pg_advisory_unlock` → `t` thì B ✔.

`pg_advisory_lock` là khoá **mức phiên**: giữ tới khi tự unlock hoặc phiên đóng — với connection pool, «phiên đóng» có thể là rất lâu sau. **`pg_advisory_xact_lock`** là khoá mức giao dịch: tự nhả khi COMMIT/ROLLBACK, không có hàm unlock; lab chạy nó trong `BEGIN … COMMIT` và sau đó `pg_locks` còn 0 khoá advisory. Ưu tiên loại sau (bài tập 2 cho thấy vì sao).

## 9. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách xử lý |
|---|---|---|
| Lưu bị treo, CPU database rảnh | Phiên khác `idle in transaction` giữ khoá dòng | Tìm gốc bằng `pg_blocking_pids`; sửa app để không giữ giao dịch qua thao tác người dùng |
| Chèn sự kiện cho chuyến bị chờ | Phiên khác giữ `FOR UPDATE` trên chuyến (xung đột với `FOR KEY SHARE` của khoá ngoại) | Dùng `FOR NO KEY UPDATE` |
| Migration nhỏ làm cả màn hình đứng | `ACCESS EXCLUSIVE` chờ sau giao dịch dài, người đọc xếp hàng sau nó | `SET lock_timeout` ngắn cho DDL + thử lại |
| Sau `57014`/`55P03`, mọi câu tiếp lỗi `25P02` | Hết giờ huỷ câu nhưng giao dịch đã hỏng | ROLLBACK rồi làm lại cả giao dịch |

## 10. Đánh đổi

- **Giao dịch theo màn hình** chắc chắn không ai chen vào, nhưng biến thời gian suy nghĩ của người dùng thành thời gian khoá. Màn sửa cước nên đọc, cho sửa ngoài giao dịch, rồi ghi một lần có kiểm `version` (PG-035).
- **`FOR UPDATE`** không «an toàn hơn» `FOR NO KEY UPDATE` cho đọc–sửa thường; nó chỉ chặn thêm dòng con (và làm `SKIP LOCKED` bỏ qua dòng — PG-036).
- **Timeout ngắn** lỗi sớm và rõ nhưng app phải biết thử lại; **0** thì không lỗi nhưng hàng chờ lan sang mọi request. Giữ `lock_timeout` < `statement_timeout`.

## 11. Bài tập

**Bài 1.** Viết câu thêm cột `ghi_chu` vào `trips` để chạy giờ cao điểm mà không làm đứng màn theo dõi chuyến. Câu đó gặp lỗi gì khi có báo cáo dài đang đọc bảng, và sau đó phải làm gì?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SET lock_timeout = '300ms';
ALTER TABLE trips ADD COLUMN ghi_chu text;
RESET lock_timeout;</code></pre>
<p>Có báo cáo đang giữ <code>AccessShareLock</code> ⇒ sau 300 ms: <code>55P03 canceling statement due to lock timeout</code>, người đọc đến sau không bị kẹt (lab §3). Script bắt <code>55P03</code>, chờ một lúc rồi thử lại; trong giao dịch thì dùng <code>SET LOCAL</code> và ROLLBACK trước khi thử lại.</p>
</details>

**Bài 2.** A chạy `BEGIN; SELECT pg_advisory_lock(400109); ROLLBACK;`. B chạy `SELECT pg_try_advisory_lock(400109);` ra gì? Sửa đoạn của A để ROLLBACK nhả khoá.

<details><summary><b>Lời giải</b></summary>
<p>B nhận <code>f</code>: khoá mức phiên không theo giao dịch — ROLLBACK không nhả nó, <code>pg_locks</code> vẫn có <code>lab-A | ExclusiveLock | t</code> tới khi A gọi <code>pg_advisory_unlock(400109)</code> (thí nghiệm <code>pg033-go-roi-phien-giu-khoa.sql</code> §4). Sửa:</p>
<pre><code class="language-sql">BEGIN;
SELECT pg_advisory_xact_lock(400109);
-- … việc cần làm …
COMMIT;  -- (hoặc ROLLBACK): khoá tự nhả</code></pre>
</details>

## 12. Đọc tiếp

- [PostgreSQL 18 · Explicit Locking](https://www.postgresql.org/docs/18/explicit-locking.html) — chế độ khoá bảng, bốn mức khoá dòng, advisory lock.
- [PostgreSQL 18 · pg_locks](https://www.postgresql.org/docs/18/view-pg-locks.html) — vì sao khoá dòng không hiện, chờ trên transactionid.
- [PostgreSQL 18 · System Information Functions](https://www.postgresql.org/docs/18/functions-info.html) — `pg_blocking_pids()`, chặn cứng và chặn mềm.
- [PostgreSQL 18 · Client Connection Defaults](https://www.postgresql.org/docs/18/runtime-config-client.html) — `lock_timeout`, `statement_timeout`, `transaction_timeout`, `idle_in_transaction_session_timeout`; [PostgreSQL 17 · Release Notes](https://www.postgresql.org/docs/17/release-17.html) — thêm `transaction_timeout`.
- Lab: `p04/33-khoa-dong-khoa-bang.sql`, `p04/OUTPUT.txt`; thí nghiệm `p04/thu-nghiem/pg033-go-roi-phien-giu-khoa.sql` (không có trong lab chính) và `p04/thu-nghiem/pg033-transaction-timeout.sql`.

Bài tiếp theo: [PG-034: Read Committed và Repeatable Read](/articles/postgresql-isolation-read-committed-repeatable-read).
