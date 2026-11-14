---
slug: postgresql-mvcc-snapshot
title: "MVCC và snapshot: mỗi phiên nhìn thấy gì?"
summary: "Tổng đài viên Tới Nơi mở chuyến 400101 đúng lúc server đang sửa cước: thấy 31.000 hay 35.000 đ, có phải chờ không? Chạy hai phiên để soi phiên bản dòng, xmin/xmax, snapshot và dòng chết mà ROLLBACK để lại."
category: postgresql
level: beginner
tags: [postgresql, pg-032, mvcc, snapshot, xmin-xmax, transaction-id, vacuum, toi-noi]
language: vi
---

Khách gọi tổng đài Tới Nơi hỏi cước chuyến 400101. Tổng đài viên mở màn chi tiết đúng lúc server điều phối đang chạy một giao dịch tăng cước chuyến đó từ 31.000 lên 35.000 đ vì khách đổi điểm đón. Màn hình của tổng đài viên hiện số nào? Màn hình có phải đứng chờ giao dịch kia xong không? Và nếu giao dịch kia `ROLLBACK`, có còn dấu vết gì không?

Ở PG-031, phiên B «không thấy» dữ liệu chưa commit của phiên A mà cũng không bị chặn. Bài này đi vào cơ chế phía sau: PostgreSQL không ghi đè dòng mà giữ **nhiều phiên bản**, và mỗi câu đọc nhìn qua một **snapshot**. Bài chứng minh mọi điều trên bằng hai phiên chạy thật, kèm cột hệ thống, trang dữ liệu và trạng thái giao dịch.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-032 · Phần 4/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/postgresql-transaction-commit-rollback">PG-031 · Transaction, COMMIT và ROLLBACK</a>; nên lướt <a href="/articles/heap-ctid-va-primary-key">PG-024 · Heap, ctid và HOT</a> và <a href="/articles/postgresql-index-only-scan-visibility-map">PG-026 · Index-only scan và visibility map</a>. Lab: <code>examples/toi-noi-postgresql/p04</code> (tệp <code>32-mvcc-snapshot.sql</code>; thí nghiệm <code>thu-nghiem/pg032-snapshot-dang-chay-rr-delete.sql</code>). Sau bài này, bạn chạy hai phiên để quan sát phiên bản dữ liệu và giải thích phiên nào thấy phiên bản nào.</p>
</div>

## 1. Dữ liệu và cách chạy

PostgreSQL **18.1 thật**, nền `p04`: chuyến 400101–400110 đang chờ, cước 31.000 + 1.000·n đ, `version` 1; cả bảng `trips` (40 chuyến) nằm trong page 0, chuyến 400101 ở ô `(0,31)`. Hai phiên: **A** là server điều phối, **B** là tổng đài viên.

```bash
cd examples/toi-noi-postgresql
node pg/run-pg.mjs p04 --only 32
node pg/run-pg.mjs --nen p04/lab.json:p04 p04/thu-nghiem/pg032-snapshot-dang-chay-rr-delete.sql
```

**Mã giao dịch** (transaction ID, **xid**) là số tăng dần dùng chung cho cả cluster, nên số thật khác nhau giữa các lần chạy và các máy. Lab in số **tương đối**: đầu tệp, `lab_dat_moc_xid()` lấy xid của chính nó làm mốc **X**; giao dịch ghi kế tiếp là **X+1**, rồi X+2…; `<X` là giao dịch đã commit từ trước mốc (dữ liệu fixture). Các hàm `xid_td()`, `snap_td()`, `xid_x()` chỉ là tiện ích hiển thị của lab (`p04/00-mo-rong-schema.sql`), không phải mẫu dùng ở production.

## 2. UPDATE không ghi đè: phiên bản dòng, xmin và xmax

**MVCC** (multiversion concurrency control — điều khiển đồng thời đa phiên bản) là cách PostgreSQL cho nhiều phiên cùng đọc–ghi: `UPDATE` không sửa dòng tại chỗ mà **tạo phiên bản mới** (row version, còn gọi là tuple), phiên bản cũ ở lại. Mỗi phiên bản mang hai **cột hệ thống** — không khai trong `CREATE TABLE` nhưng `SELECT` được nếu gọi tên:

- **`xmin`** — xid của giao dịch đã **tạo** ra phiên bản này.
- **`xmax`** — xid của giao dịch đã **xoá/thay thế** (hoặc đang khoá) phiên bản này; `0` nghĩa là chưa ai đụng tới.

Lợi ích, theo tài liệu: với MVCC, "reading never blocks writing and writing never blocks reading". Phần còn lại của bài kiểm chứng câu đó trên chuyến 400101.

## 3. Giao dịch chỉ đọc chưa được cấp xid

```sql
-- Phiên A
BEGIN;
SELECT fare FROM trips WHERE code = 400101;          -- 31000
SELECT pg_current_xact_id_if_assigned();             -- NULL
UPDATE trips SET fare = 35000, version = version + 1, updated_at = now()
WHERE code = 400101
RETURNING fare, version, xid_td(xmin) AS xmin, xid_td(xmax) AS xmax;
```

```text
 fare  | version | xmin | xmax
-------+---------+------+------
 35000 |       2 | X+1  | 0
(1 row)
```

Sau câu `SELECT`, A vẫn chưa có xid (NULL): PostgreSQL chỉ cấp xid ở **câu ghi đầu tiên** của giao dịch, giao dịch chỉ đọc không tiêu tốn xid. Câu `UPDATE` làm A nhận xid **X+1**, và `RETURNING` đọc chính phiên bản mới: `xmin = X+1` (A tạo ra nó), `xmax = 0`.

## 4. B đọc khi A chưa commit: thấy bản cũ, không phải chờ

A vẫn chưa `COMMIT`. Lịch chạy:

| Bước | Phiên A | Phiên B | Kết quả |
|---|---|---|---|
| T1 | `BEGIN`; `UPDATE 400101 SET fare = 35000` | | `35000 \| 2 \| X+1 \| 0` |
| T2 | | `SELECT fare, version, xmin, xmax … 400101` | **`31000 \| 1 \| <X \| X+1`** — trả ngay |
| T3 | | `snap_td(pg_current_snapshot())` | `xmin=X+1 xmax=X+1 đang chạy={}` |
| T4 | | `pg_xact_status(xid_x(1))` | `in progress` |
| T5 | | soi page 0 bằng `pageinspect` | hai phiên bản (dưới) |
| T6 | `COMMIT` | | `COMMIT` |
| T7 | | đọc lại 400101 | **`35000 \| 2 \| X+1 \| 0`** |
| T8 | | `pg_xact_status(xid_x(1))`; snapshot | `committed`; `xmin=X+2 xmax=X+2 đang chạy={}` |

Ở T2, B nhận **phiên bản cũ**: cước 31.000, `xmin` từ fixture, và `xmax = X+1` — A đã «đánh dấu thay thế» nó. Nhưng X+1 chưa commit (T4: `in progress`), nên với B bản cũ vẫn là bản đúng. B không chờ khoá nào: đó là «đọc không chặn ghi, ghi không chặn đọc».

T5 cho thấy cả hai phiên bản cùng nằm trong page của bảng (extension `pageinspect`, hàm `heap_page_items`):

```text
 lp | t_xmin | t_xmax | t_ctid | hot_updated | heap_only
----+--------+--------+--------+-------------+-----------
 31 | <X     | X+1    | (0,41) | t           | f
 41 | X+1    | 0      | (0,41) | f           | t
(2 rows)
```

Ô 31 là bản cũ, `t_ctid` trỏ sang bản mới ở ô 41. Bản mới nằm cùng page và không có cột nào có index bị đổi, nên đây là **HOT update** — cơ chế và điều kiện của HOT update đã được trình bày ở [PG-024](/articles/heap-ctid-va-primary-key), bài này chỉ cần thấy: hai phiên bản tồn tại song song, ai thấy bản nào là chuyện của snapshot.

Sau `COMMIT` (T7), câu đọc kế tiếp của B thấy bản mới. B đang ở mức mặc định **Read Committed**: **mỗi câu lệnh** chụp một snapshot mới, nên câu chạy sau COMMIT của A thấy dữ liệu A vừa commit.

## 5. Snapshot gồm những gì

**Snapshot** là «ảnh chụp» trạng thái các giao dịch tại một thời điểm, đủ để trả lời cho mọi xid: «lúc chụp, giao dịch này đã kết thúc chưa?». `pg_current_snapshot()` trả về ba phần, theo đúng định nghĩa của tài liệu:

| Phần | Ý nghĩa | Ở T3 |
|---|---|---|
| `xmin` | xid nhỏ nhất **còn đang chạy**; mọi xid nhỏ hơn đã kết thúc (commit hoặc huỷ) | `X+1` (A) |
| `xmax` | **xid lớn nhất đã kết thúc + 1**; mọi xid ≥ xmax coi như chưa kết thúc ⇒ vô hình | `X+1` |
| đang chạy (`xip`) | các xid trong `[xmin, xmax)` chưa kết thúc lúc chụp | `{}` |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Bẫy: xmax của snapshot không phải «xid sắp cấp»</span>
<p>Ở T3, A (X+1) rõ ràng đang chạy mà danh sách «đang chạy» lại rỗng. Lý do: giao dịch đã kết thúc gần nhất là X (lệnh đặt mốc), nên <code>xmax = X+1</code>; A có xid ≥ xmax nên đã vô hình mà không cần nằm trong danh sách. Hiểu snapshot theo kiểu «xmax = xid mới nhất + 1, mọi giao dịch đang chạy đều trong danh sách» là sai với PostgreSQL 18.</p>
</div>

Danh sách chỉ có phần tử khi một giao dịch **cũ hơn** vẫn chạy trong lúc một giao dịch **mới hơn** đã kết thúc. Thí nghiệm `pg032-snapshot-dang-chay-rr-delete.sql` §A dựng đúng cảnh đó: A (X+1) sửa chuyến 400104 và để mở, B chạy một câu lẻ sửa 400105 (X+2, commit ngay), rồi B chụp:

```text
              snap_td
-----------------------------------
 xmin=X+1 xmax=X+3 đang chạy={X+1}
```

X+2 đã kết thúc nên `xmax = X+3`; X+1 nhỏ hơn xmax mà chưa kết thúc nên phải được liệt kê. Cùng lúc, B đọc 400104 ra 34.000 (bản cũ — vô hình X+1) và 400105 ra 40.000 (bản của X+2 — đã commit trước khi chụp).

Từ ba phần đó, quy tắc nhìn thấy (rút gọn — bỏ qua thay đổi của chính giao dịch đang đọc, khoá dòng và giao dịch con) là:

1. Phiên bản **có thể thấy** nếu giao dịch tạo ra nó (`xmin`) **đã commit trước lúc chụp**: `xmin < xmax` của snapshot, không nằm trong danh sách đang chạy, và trạng thái là `committed`.
2. Và phiên bản **chưa bị thay**: `xmax = 0`, hoặc giao dịch ở `xmax` đã huỷ, hoặc chưa commit trước lúc chụp (theo cùng cách xét ở bước 1).

Áp vào T2: bản cũ có `xmin` từ fixture (đã commit) và `xmax = X+1` (≥ xmax của snapshot ⇒ coi như chưa commit) ⇒ thấy. Bản mới có `xmin = X+1` ⇒ không thấy. `pg_xact_status()` (đọc sổ trạng thái `pg_xact`) trả `in progress`, `committed` hoặc `aborted` cho giao dịch còn đủ gần — đó là thông tin PostgreSQL tra để quyết định bước 1 và 2.

## 6. Repeatable Read: một snapshot cho cả giao dịch

Read Committed chụp lại mỗi câu. **Repeatable Read** chụp **một lần** rồi giữ tới hết giao dịch:

| Bước | Phiên A | Phiên B | Kết quả |
|---|---|---|---|
| T1 | | `BEGIN ISOLATION LEVEL REPEATABLE READ` | |
| T2 | | `SELECT fare … 400102` — snapshot chụp ở đây | `32000` |
| T3 | `UPDATE 400102 SET fare = 36000` (câu lẻ, commit ngay) | | `36000 \| X+2` |
| T4 | | `SELECT fare, xmin, xmax … 400102` | **`32000 \| <X \| X+2`** |
| T5 | | `snap_td(pg_current_snapshot())` | `xmin=X+2 xmax=X+2 đang chạy={}` |
| T6 | | `COMMIT`, rồi `SELECT fare … 400102` | **`36000`** |

Ở T4, X+2 **đã commit** nhưng B vẫn đọc 32.000: snapshot của B chụp ở T2, khi X+2 chưa tồn tại, và T5 cho thấy B vẫn đang dùng đúng snapshot cũ (`xmax = X+2` ⇒ X+2 vô hình). Phiên bản cũ có `xmax = X+2` vẫn là bản đúng **với B**. Giao dịch kế tiếp của B mới thấy 36.000.

Lưu ý thời điểm chụp: tài liệu ghi snapshot của Repeatable Read lấy ở **câu lệnh đầu tiên không phải lệnh điều khiển giao dịch**, không phải ở `BEGIN` — bài tập 2 kiểm chứng. So sánh đầy đủ hai mức cô lập dưới cùng một lịch (đọc lại, phantom, mất cập nhật) là việc của [PG-034](/articles/postgresql-isolation-read-committed-repeatable-read).

## 7. ROLLBACK cũng để lại dấu vết

```sql
-- Phiên A
BEGIN;
UPDATE trips SET fare = 99000 WHERE code = 400103 RETURNING xid_td(xmin);   -- X+3
ROLLBACK;

-- Phiên B
SELECT fare, xid_td(xmin) AS xmin, xid_td(xmax) AS xmax, pg_xact_status(xid_x(3)) AS trang_thai_x3
FROM trips WHERE code = 400103;
```

```text
 fare  | xmin | xmax | trang_thai_x3
-------+------+------+---------------
 33000 | <X   | X+3  | aborted
(1 row)
```

`ROLLBACK` không «xoá» phiên bản 99.000 đã ghi vào page, cũng không xoá `xmax = X+3` trên bản cũ. Lệnh này chỉ ghi nhận X+3 là `aborted`. Từ đó, phiên đọc tra trạng thái và biết: bản 99.000 không bao giờ có hiệu lực, bản 33.000 **vẫn sống** dù `xmax ≠ 0`. Vì vậy đừng bao giờ suy «`xmax` khác 0 ⇒ dòng đã bị xoá»: giá trị đó còn có thể do giao dịch đang chạy, giao dịch đã huỷ, hoặc chỉ là khoá dòng (`FOR UPDATE` — [PG-033](/articles/postgresql-lock-row-table)).

Lúc này bảng `trips` có ba phiên bản không ai còn thấy — **dòng chết** (dead tuple): bản cũ của 400101 (thay bởi X+1), bản cũ của 400102 (thay bởi X+2), bản 99.000 của 400103 (X+3 huỷ). Extension `pgstattuple` đếm được:

```text
 tuple_count | dead_tuple_count
-------------+------------------
          40 |                3      -- trước VACUUM
          40 |                0      -- sau VACUUM toinoi.trips
```

(Ghép từ hai câu của lab.) **VACUUM** dọn phiên bản chết khi không còn snapshot nào có thể cần tới chúng. Ở đây không còn giao dịch nào mở nên dọn hết. Nếu một phiên giữ giao dịch mở lâu (kể cả «idle in transaction» ở PG-031), snapshot cũ của nó buộc VACUUM giữ lại phiên bản chết — thí nghiệm hai phiên ở [PG-026](/articles/postgresql-index-only-scan-visibility-map) đã đo hậu quả lên index-only scan. Đây là cái giá của MVCC: với ≈ 560 UPDATE/giây vào `trips` lúc cao điểm (hồ sơ Tới Nơi), mỗi giây có hàng trăm phiên bản chết mới cần dọn; cấu hình autovacuum thuộc Phần 5.

## 8. xid, xid8 và txid_current()

```text
 kieu_moi | kieu_cu | kieu_cot_he_thong | cung_mot_so
----------+---------+-------------------+-------------
 xid8     | bigint  | xid               | t
(1 row)
```

Ba cách gọi cùng một thứ: cột hệ thống `xmin`/`xmax` dùng kiểu **`xid` 32 bit**, quay vòng sau khoảng 4 tỉ giao dịch; `pg_current_xact_id()` trả **`xid8` 64 bit** gồm thêm «epoch» (số lần quay vòng) nên không quay vòng trong đời một cluster; `txid_current()` là tên cũ trả `bigint` — tài liệu ghi các hàm `txid_*` có từ trước khi có kiểu `xid8` (PostgreSQL 13), nay chỉ giữ để tương thích. Code mới dùng nhóm `pg_current_xact_id*`. Ưu tiên `pg_current_xact_id_if_assigned()` khi chỉ muốn hỏi, vì `pg_current_xact_id()` sẽ **cấp** xid nếu giao dịch chưa có. Quay vòng xid và việc «đóng băng» (freeze) phiên bản cũ là chủ đề bảo trì của Phần 5.

## 9. Hiểu sai thường gặp

| Hiểu sai | Thực tế (bằng chứng trong lab) |
|---|---|
| UPDATE sửa dòng tại chỗ | Tạo phiên bản mới; page có cả ô 31 lẫn ô 41 |
| Đọc phải chờ giao dịch đang ghi | B đọc ngay bản cũ ở T2 |
| `xmax ≠ 0` là dòng đã xoá | 400103 sống với `xmax = X+3` (`aborted`) |
| ROLLBACK xoá sạch mọi dấu vết | Để lại dòng chết (3 → VACUUM → 0) |
| Mọi giao dịch đều có xid | Giao dịch chỉ đọc: `NULL` |
| Snapshot liệt kê mọi giao dịch đang chạy | Chỉ những xid trong `[xmin, xmax)`; T3 rỗng dù A đang chạy |
| Repeatable Read chụp lúc `BEGIN` | Chụp ở câu đầu tiên (bài tập 2) |

## 10. Đánh đổi

- **Đọc không chờ, nhưng có thể «cũ».** B ở T2 thấy 31.000 trong khi A đã quyết 35.000. Với màn xem chi tiết thì ổn. Với «đọc rồi quyết định ghi» (hai tài xế cùng nhận một chuyến) thì đọc theo snapshot là chưa đủ — PG-033 (khoá dòng) và PG-035 (cập nhật có điều kiện) giải quyết trường hợp này.
- **Read Committed hay Repeatable Read.** RC luôn thấy dữ liệu mới nhất ở mỗi câu nhưng hai câu trong cùng giao dịch có thể lệch nhau; RR cho cả báo cáo một mốc nhất quán nhưng giữ snapshot lâu hơn và phải chịu lỗi xung đột khi ghi (PG-034).
- **Không gian và VACUUM.** Giữ nhiều phiên bản phải trả giá bằng dung lượng tạm thời và việc dọn dẹp; giao dịch mở lâu khiến việc dọn bị chặn.

## 11. Bài tập

Lời giải chạy thật trong thí nghiệm `p04/thu-nghiem/pg032-snapshot-dang-chay-rr-delete.sql` (không có trong lab chính).

**Bài 1.** A mở giao dịch, sửa cước 400104 (X+1) và **chưa commit**. B chạy một câu lẻ sửa 400105 (X+2). Dự đoán snapshot của B ngay sau đó, và cước B đọc được của 400104, 400105.

<details><summary><b>Lời giải</b></summary>
<p>Snapshot <code>xmin=X+1 xmax=X+3 đang chạy={X+1}</code>: X+2 là giao dịch đã kết thúc lớn nhất nên xmax = X+3; X+1 nằm trong khoảng mà chưa kết thúc nên có trong danh sách. B đọc <code>34000 | 40000</code>: 400104 vẫn là bản cũ (X+1 vô hình), 400105 là bản của X+2. Sau khi A commit, snapshot mới là <code>xmin=X+3 xmax=X+3 đang chạy={}</code>.</p>
</details>

**Bài 2.** B gõ `BEGIN ISOLATION LEVEL REPEATABLE READ` nhưng chưa chạy câu nào. A (câu lẻ) đổi cước 400106 thành 40.000. B đọc 400106. A lại đổi thành 41.000. B đọc lần nữa. Hai lần đọc của B ra bao nhiêu?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">-- B
BEGIN ISOLATION LEVEL REPEATABLE READ;
-- A: UPDATE trips SET fare = 40000 WHERE code = 400106;   (X+3, commit)
SELECT fare FROM trips WHERE code = 400106;                -- 40000: snapshot chụp ở đây
-- A: UPDATE trips SET fare = 41000 WHERE code = 400106;   (X+4, commit)
SELECT fare FROM trips WHERE code = 400106;                -- vẫn 40000
COMMIT;</code></pre>
<p>Lần 1 ra <code>40000</code> chứ không phải 36.000 ban đầu: snapshot của Repeatable Read chụp ở câu đầu tiên của giao dịch, lúc X+3 đã commit. Lần 2 vẫn <code>40000</code> vì X+4 commit sau khi chụp. Hệ quả thực tế: muốn «chốt mốc» báo cáo thì chạy câu đọc đầu tiên ngay, đừng làm việc khác giữa <code>BEGIN</code> và câu đó rồi giả định mốc là lúc <code>BEGIN</code>.</p>
</details>

**Bài 3.** `DELETE` có tạo phiên bản mới không? A xoá việc 3 trong hàng đợi `jobs` nhưng chưa commit; B đọc việc 3. Sau khi A commit, `pgstattuple('toinoi.jobs')` báo gì?

<details><summary><b>Lời giải</b></summary>
<p><code>DELETE</code> không tạo phiên bản mới, chỉ đặt <code>xmax</code> lên phiên bản hiện có. Trước khi A commit, B vẫn đọc được việc 3 với <code>3 | &lt;X | X+5</code> (xmax = xid của A). Sau commit, <code>count(*)</code> việc 3 ra 0, <code>pgstattuple</code> ra <code>9 | 1</code> (9 dòng sống, 1 dòng chết — chính phiên bản vừa xoá); <code>VACUUM toinoi.jobs</code> đưa về <code>9 | 0</code>. Xoá cũng sinh rác như sửa.</p>
</details>

## 12. Đọc tiếp

- [PostgreSQL 18 · Concurrency Control — Introduction](https://www.postgresql.org/docs/18/mvcc-intro.html) và [Transaction Isolation](https://www.postgresql.org/docs/18/transaction-iso.html).
- [PostgreSQL 18 · System Columns](https://www.postgresql.org/docs/18/ddl-system-columns.html) — `xmin`, `xmax`.
- [PostgreSQL 18 · System Information Functions](https://www.postgresql.org/docs/18/functions-info.html) — `pg_current_snapshot()`, `pg_xact_status()`, bảng các hàm `txid_*` cũ.
- [PostgreSQL 18 · Transactions and Identifiers](https://www.postgresql.org/docs/18/transaction-id.html) — khi nào cấp xid, `xid` và `xid8`.
- [PostgreSQL 18 · pageinspect](https://www.postgresql.org/docs/18/pageinspect.html), [pgstattuple](https://www.postgresql.org/docs/18/pgstattuple.html).
- Lab: `p04/32-mvcc-snapshot.sql`, `p04/OUTPUT.txt`; thí nghiệm (không có trong lab chính) `p04/thu-nghiem/pg032-snapshot-dang-chay-rr-delete.sql`.

Bài tiếp theo: [PG-033: Khoá dòng, khoá bảng và thời gian chờ](/articles/postgresql-lock-row-table)
