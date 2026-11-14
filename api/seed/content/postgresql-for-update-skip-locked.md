---
slug: postgresql-for-update-skip-locked
title: "FOR UPDATE, NOWAIT và SKIP LOCKED: hàng đợi worker và những dòng bị bỏ qua"
summary: "Tới Nơi chạy nhiều worker gửi thông báo từ một bảng jobs: lúc thì worker xếp hàng chờ nhau, lúc thì khách nhận hai SMS, có lúc việc gấp nằm im. Bài so sánh FOR UPDATE, NOWAIT, SKIP LOCKED và chỉ rõ khi nào dòng bị bỏ qua."
category: postgresql
level: intermediate
tags: [postgresql, pg-036, for-update, skip-locked, nowait, job-queue, row-lock, toi-noi]
language: vi
---

Mỗi chuyến mới của Tới Nơi sinh một việc nền: gửi thông báo cho tài xế quanh khu đón, tính cước dự kiến, đối soát. Việc nằm trong bảng `jobs`, nhiều worker cùng lấy. Đội backend đã gặp đủ ba kiểu sự cố: tám worker chạy mà thông lượng như một worker; một khách nhận **hai** SMS cho cùng một chuyến; và một việc đối soát gấp nằm `queued` cả chục phút trong khi bảng điều khiển báo «hàng đợi rỗng».

Ba sự cố đến từ cùng một câu lệnh «lấy việc kế tiếp» với ba cách khoá dòng khác nhau. Bài này chạy cả ba trên PostgreSQL thật, đo chúng với 8 worker, và liệt kê chính xác những trường hợp `SKIP LOCKED` bỏ qua một dòng mà bạn không muốn nó bỏ qua.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-036 · Phần 4/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-optimistic-version-conditional-update">PG-035 · optimistic concurrency</a>, <a href="/articles/postgresql-lock-row-table">PG-033 · khoá dòng, khoá bảng</a>. Lab: <code>examples/toi-noi-postgresql/p04</code> (tệp <code>36-for-update-skip-locked.sql</code>, phép thử K036 trong <code>DO-DAC.md</code>, thí nghiệm <code>thu-nghiem/pg036-lease-fencing-lo-viec-idle-timeout.sql</code>). Sau bài này, bạn dựng và thử được một hàng đợi nhiều worker trên PostgreSQL và nói rõ trường hợp nào một việc bị bỏ qua, bị bỏ đói hay chạy hai lần.</p>
</div>

## 1. Hàng đợi `jobs` và câu «lấy việc kế tiếp»

Nền lab Phần 4 (PostgreSQL **18.1** thật, cluster tạm tự dọn) có bảng:

```sql
CREATE TABLE jobs (
  id         bigint      GENERATED ALWAYS AS IDENTITY,
  kind       text        NOT NULL,
  priority   smallint    NOT NULL DEFAULT 5,       -- 1 = gấp nhất
  status     text        NOT NULL DEFAULT 'queued', -- queued | running | done | failed
  attempts   integer     NOT NULL DEFAULT 0,
  locked_by  text,
  locked_at  timestamptz,
  done_at    timestamptz,
  …
);
CREATE INDEX ix_jobs_queued ON jobs (priority, id) WHERE status = 'queued';
```

Partial index (PG-025) chỉ chứa việc đang chờ, theo thứ tự worker cần. Hàng đợi đầu bài: việc **1** ưu tiên 1, việc **2…10** ưu tiên 5. Mỗi lần một worker *thật sự* xử lý một việc, nó ghi một dòng `job_runs (job_id, worker)` — bảng cố ý không có UNIQUE để lộ việc chạy hai lần. Bất biến: **mỗi việc đúng một dòng `job_runs`**.

Câu lấy việc:

```sql
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE …;
```

`FOR UPDATE` là **locking clause** (mệnh đề khoá): khoá dòng được trả về như thể sắp sửa chính các dòng đó. Ba biến thể ở cuối câu quyết định khi gặp dòng đang bị khoá thì làm gì: chờ (mặc định), `NOWAIT` (báo lỗi ngay), `SKIP LOCKED` (bỏ qua dòng đó). Chạy lab:

```bash
cd examples/toi-noi-postgresql
node pg/run-pg.mjs p04 --only 36
node pg/run-pg.mjs --nen p04/lab.json:p04 p04/thu-nghiem/pg036-lease-fencing-lo-viec-idle-timeout.sql
```

Ba phiên psql `[A]`, `[B]`, `[C]` là ba worker. Lịch in trong `OUTPUT.txt` là thứ tự *gửi*; bảng dưới xếp theo thứ tự *xảy ra*, bước bị chặn ghi ⏳ còn kết quả của bước đó nằm ở dòng ✔.

## 2. FOR UPDATE: đúng, nhưng xếp hàng

| Bước | [A] | [B] |
|---|---|---|
| T1 | `BEGIN; SELECT … FOR UPDATE` → **1** | |
| T2 | | `BEGIN; SELECT … FOR UPDATE` ⏳ *Lock: transactionid — bị chặn bởi [A]* |
| T3 | `INSERT INTO job_runs VALUES (1, 'A')`; `UPDATE jobs SET status = 'done' … WHERE id = 1` | |
| T4 | `COMMIT` | |
| T5 | | ✔ → **2** |
| T6 | | xử lý việc 2, `COMMIT` |

B nhắm cùng dòng đầu hàng (việc 1) nên phải chờ suốt thời gian A xử lý. Khi A commit, B giành được khoá dòng 1, kiểm tra lại điều kiện trên phiên bản mới (`done` ⇒ loại) rồi đi tiếp tới việc 2 — cùng cơ chế «đọc lại dòng» của Read Committed ở PG-035. Không ai nhận nhầm việc đã xong, nhưng worker nối đuôi nhau: tám worker nhắm cùng một dòng thì chỉ một người làm, bảy người chờ.

Tài liệu còn cảnh báo một chi tiết: ở Read Committed, `SELECT` có `ORDER BY` kèm locking clause có thể trả dòng *lệch thứ tự*, vì `ORDER BY` được áp trước, dòng bị đổi trong lúc chờ khoá thì không được sắp lại. Với hàng đợi dùng `LIMIT 1`, hệ quả là «việc kế tiếp» không phải lúc nào cũng là việc đứng đầu — chấp nhận được, nhưng đừng hứa thứ tự tuyệt đối.

## 3. NOWAIT: lỗi ngay

A giữ việc 3 trong một giao dịch mở. B hỏi với `NOWAIT`:

```sql
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE NOWAIT;
```

```text
ERROR:  could not obtain lock on row in relation "jobs"
```

SQLSTATE **`55P03`** (lock_not_available — cùng mã với `lock_timeout` ở PG-033). B không chờ, nhưng cũng không lấy được việc nào: lỗi làm hỏng giao dịch, B phải `ROLLBACK` và hỏi lại. `NOWAIT` hợp với thao tác *một* dòng cụ thể («mở chuyến 400108 để sửa — nếu ai đang sửa thì báo ngay»), không hợp với hàng đợi.

## 4. SKIP LOCKED: mỗi worker một việc, không chờ

```sql
-- A, B, C lần lượt, cả ba giao dịch đều đang mở
SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE SKIP LOCKED;
```

A nhận **4**, B nhận **5**, C nhận **6** — không bước nào chờ. Mỗi worker bỏ qua dòng đã bị người trước khoá và lấy dòng kế tiếp; xử lý xong (ghi `job_runs`, đánh dấu `done`) trong cùng giao dịch rồi commit. Tài liệu PostgreSQL nói thẳng cái giá: bỏ qua dòng bị khoá khiến dữ liệu hiện ra một cách **không nhất quán**, «not suitable for general purpose work», nhưng dùng được để tránh tranh chấp khi nhiều consumer cùng đọc một bảng kiểu hàng đợi. Mục 7 đi vào những chỗ «không nhất quán» đó.

`NOWAIT` và `SKIP LOCKED` chỉ áp cho khoá *dòng*; khoá mức bảng `ROW SHARE` mà câu lệnh cần vẫn chờ như thường (DDL đang giữ `ACCESS EXCLUSIVE` vẫn chặn được worker).

## 5. Đo với 8 worker: phép thử K036

Lịch tất định ở trên chỉ chứng minh từng hành vi. Phép thử đồng thời của lab (`p04/dong-thoi.mjs` → `p04/DO-DAC.md`) cho **8 phiên psql thật chạy đua**, 600 việc (ưu tiên 1–9), mỗi worker lặp: một giao dịch [lấy việc ⇒ «xử lý» bằng `pg_sleep(0.002)` *trong khi giữ khoá* ⇒ ghi `job_runs` ⇒ đánh dấu `done` ⇒ `COMMIT`] tới khi hết việc. Mỗi cách chạy **3 lần**, mỗi lần trên database mới.

Máy đo: Intel i7-10700 (16 luồng), 64 GB RAM, NVMe Kingston SA2000, Windows 11 Pro, Node 22.18, PostgreSQL 18.1, tải CPU nền ~6 %.

| Cách | Giây/lần (min–max) | Việc/giây | Mỗi worker làm (ít nhất – nhiều nhất) | Ghi chú |
|---|---|---|---|---|
| `SKIP LOCKED` | 1,21–1,32 | 456–496 | 74–76 | `rong_gia` 2–7 lần |
| `FOR UPDATE` (chờ) | 9,36–9,42 | 64 | 0–2 / 115–159 | chia rất lệch |
| `NOWAIT` + thử lại sau 1 ms | 9,30–9,31 | 64–65 | 56–70 / 78–91 | **25.317–26.774** lỗi `55P03` |

Cả ba cách đều giữ bất biến ở mọi lần: 600/600 việc `done`, **0** việc có số dòng `job_runs` khác 1. Khác nhau ở chỗ worker nào được làm việc và cái giá phải trả:

- **FOR UPDATE** xử lý gần như tuần tự: 600 việc trong ~9,4 s. Việc chia lệch tới mức có worker chỉ làm 0–2 việc.
- **NOWAIT** cũng chậm như vậy và phát sinh hơn 25 nghìn lỗi cho 600 việc: các worker liên tục va vào cùng dòng đầu hàng.
- **SKIP LOCKED** chia đều 74–76 việc/worker và nhanh gấp khoảng 7–8 lần. `rong_gia` đếm số lần SKIP LOCKED trả rỗng *trong khi vẫn còn việc* (mọi việc còn lại đang bị worker khác khoá) — worker phải hỏi lại.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Đọc số này cẩn thận</span>
<p>Trên Windows, <code>pg_sleep(0.002)</code> ngủ thực tế khoảng 15,6 ms (độ phân giải bộ định thời): 600 việc × 15,6 ms ≈ 9,4 s — đúng thời gian của cách FOR UPDATE; 600 ÷ 8 × 15,6 ms ≈ 1,2 s — đúng thời gian SKIP LOCKED. Thời gian bị chi phối bởi bước «xử lý» giả lập, không bởi PostgreSQL. Chỉ so <em>tỉ lệ</em> giữa ba cách trong cùng bảng; đừng suy ra thông lượng hàng đợi thật, và trên Linux các con số tuyệt đối sẽ khác.</p>
</div>

## 6. Việc có tác dụng ngoài database: thời hạn thuê và việc chạy hai lần

Ở mục 2–4, «xử lý» và «đánh dấu done» nằm cùng một giao dịch: commit thì cả hai cùng có hiệu lực, worker chết thì cả hai cùng mất và việc tự về hàng. Đó là **đúng một lần** — nhưng chỉ cho tác dụng *trong* database. Gửi SMS là gọi API bên ngoài, có thể mất vài giây; giữ giao dịch và khoá dòng suốt thời gian chờ mạng là điều PG-031/PG-033 khuyên tránh.

Mẫu thay thế (§4 của lab): **nhận việc bằng một câu ngắn**, commit ngay, xử lý *ngoài* giao dịch, rồi một câu thứ hai đánh dấu xong. Worker chết giữa chừng để lại việc `running` mãi ⇒ cần **thời hạn thuê** (lease): việc `running` quá hạn được tiến trình dọn dẹp trả về hàng.

```sql
UPDATE jobs SET status = 'running', locked_by = 'A', locked_at = clock_timestamp(), attempts = attempts + 1
WHERE id = (SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 FOR UPDATE SKIP LOCKED)
RETURNING id, status, attempts;
```

| Bước | Phiên | Việc | Kết quả |
|---|---|---|---|
| T1 | A | nhận việc (câu trên, autocommit) | `7 \| running \| 1` |
| T2 | A | gửi SMS (giả lập: `INSERT INTO job_runs VALUES (7, 'A')`) rồi «chết» trước khi đánh dấu done | |
| T3 | C | `pg_sleep(0.3)`; dọn dẹp `… WHERE status = 'running' AND locked_at < clock_timestamp() - interval '200 milliseconds'` | `7 \| 1` — về hàng đợi |
| T4 | B | nhận việc | `7 \| 2` |
| T5 | B | gửi SMS, `UPDATE … SET status = 'done' … WHERE id = 7 AND locked_by = 'B'` | `UPDATE 1` |
| T6 | C | `job_runs` của việc 7 | **`7 \| A,B`** |

Khách nhận hai SMS. Lease biến «việc kẹt mãi» thành «việc có thể chạy hai lần»: tác dụng ngoài database chỉ đạt **ít nhất một lần** (at-least-once). Bên nhận phải chống trùng bằng khoá idempotency (PG-039); hạn thuê chỉ điều chỉnh được *xác suất* chạy trùng (hạn ngắn ⇒ trùng nhiều, hạn dài ⇒ việc của worker chết chờ lâu).

Có một điều database làm được: giữ đúng *trạng thái* việc. Thí nghiệm §1 dùng `attempts` trả về lúc nhận làm **vé** (fencing token) của lượt thuê. A chậm, hết hạn, B nhận lại với `attempts = 2`; A tỉnh dậy đánh dấu done kèm vé cũ:

```sql
UPDATE jobs SET status = 'done', done_at = now(), locked_by = NULL, locked_at = NULL
WHERE id = 1 AND status = 'running' AND attempts = 1;   -- UPDATE 0
```

`UPDATE 0` — A biết lượt thuê của mình đã hết (đúng kiểu cập nhật có điều kiện của PG-035). Không có vé, câu này sẽ đánh dấu `done` một việc B đang làm dở. B đánh dấu với `attempts = 2` → `UPDATE 1`; kết quả cuối `done | 2 | A,B`: trạng thái đúng, SMS vẫn hai lần. (Thí nghiệm không có trong lab chính, nằm ở `p04/thu-nghiem/pg036-lease-fencing-lo-viec-idle-timeout.sql`.)

## 7. Khi nào SKIP LOCKED bỏ qua một dòng bạn cần?

### 7.1. Việc gấp bị bỏ đói

§5 của lab: thêm việc **11** ưu tiên 1. A nhận việc này rồi **treo** trong giao dịch mở (chờ một API không trả lời).

| Bước | Phiên | Câu | Kết quả |
|---|---|---|---|
| T1 | C | `INSERT INTO jobs (kind, priority) VALUES ('doi_soat', 1) RETURNING id` | 11 |
| T2 | A | `BEGIN; SELECT … FOR UPDATE SKIP LOCKED` | **11** — rồi treo |
| T3 | B, C, B | lấy-xử-lý-đánh dấu bằng một câu `WITH … SKIP LOCKED … UPDATE` | 8, 9, 10 |
| T4 | B | `SELECT … FOR UPDATE SKIP LOCKED` | **(0 rows)** — dù 11 vẫn `queued` |
| T5 | C | `pg_terminate_backend(pid)` của A | `t` — giao dịch A rollback, khoá nhả |
| T6 | B | lấy việc | **11** |

Việc ưu tiên cao nhất bị mọi worker khác vượt mặt, và hàng đợi trông như rỗng. Không có lỗi, không ai chờ — đúng là điều SKIP LOCKED được thiết kế để làm.

Khó ở chỗ *nhìn thấy* tình trạng này: khoá dòng không nằm trong `pg_locks` (PG-033 — chỉ hiện khi có người *chờ*, mà SKIP LOCKED không chờ). Thí nghiệm §3 dùng extension contrib **`pgrowlocks`**, đọc dấu khoá trong từng dòng của bảng:

```sql
SELECT j.id, j.status, p.modes, lab_nhan(p.pids[1]) AS phien
FROM jobs AS j JOIN pgrowlocks('jobs') AS p ON p.locked_row = j.ctid;
```

```text
 id | status |         modes         | phien
----+--------+-----------------------+-------
 11 | queued | {"For No Key Update"} | A
(1 row)
```

(`lab_nhan` là hàm tiện ích của lab, đổi PID thành nhãn phiên.) `pgrowlocks` đọc cả bảng và không cho ảnh chụp nhất quán — dùng để điều tra, không đặt trong vòng lặp giám sát mỗi giây trên bảng lớn. Cách giám sát thường xuyên rẻ hơn là tìm việc `queued` có `created_at` quá cũ, hoặc `pg_stat_activity` có phiên `idle in transaction` lâu.

Cách gỡ tự động là đặt giới hạn thời gian ở phía worker. Cùng thí nghiệm, A chạy `SET idle_in_transaction_session_timeout = '1s'` trước khi nhận việc 11; C ngủ 1,5 giây:

```text
-- [lab] nhật ký server: [A] FATAL:  terminating connection due to idle-in-transaction timeout
```

Ngay sau đó B lấy được **11**; câu kế tiếp của A báo mất kết nối, `pgrowlocks('jobs')` còn **0** dòng. Tham số này chỉ cắt phiên *ngồi yên* trong giao dịch; worker kẹt giữa một câu lệnh dài thì cần `statement_timeout`, và tốt nhất là đừng giữ giao dịch qua lời gọi mạng (mục 6).

### 7.2. Đếm bằng SKIP LOCKED

Cùng thời điểm T4 ở trên:

```sql
SELECT (SELECT count(*) FROM jobs WHERE status = 'queued') AS dem_thuong,
       (SELECT count(*) FROM (SELECT id FROM jobs WHERE status = 'queued' FOR UPDATE SKIP LOCKED) AS s) AS dem_skip_locked;
```

```text
 dem_thuong | dem_skip_locked
------------+-----------------
          1 |               0
(1 row)
```

SKIP LOCKED trả về «những dòng tôi khoá được ngay lúc này», không phải «những dòng đang có» — và câu đếm kiểu đó còn khoá mọi việc nó đếm được trong lúc chạy. Hãy dùng SKIP LOCKED để *lấy* việc; đếm và giám sát bằng `SELECT` thường.

### 7.3. Bẫy khoá ngoại: FOR UPDATE quá mạnh

`job_runs.job_id` tham chiếu `jobs.id`. Khi một giao dịch chèn dòng con, PostgreSQL kiểm tra khoá ngoại bằng khoá **FOR KEY SHARE** trên dòng cha (PG-033). Bảng xung đột khoá dòng trong tài liệu:

| Đang giữ → / Xin ↓ | FOR KEY SHARE | FOR SHARE | FOR NO KEY UPDATE | FOR UPDATE |
|---|---|---|---|---|
| FOR NO KEY UPDATE | — | xung đột | xung đột | xung đột |
| FOR UPDATE | **xung đột** | xung đột | xung đột | xung đột |

§5b của lab: C chèn việc 12, rồi trong giao dịch mở chèn `job_runs (12, 'kiem-tra')` — giữ FOR KEY SHARE trên 12. B hỏi:

```text
… FOR UPDATE SKIP LOCKED;          → (0 rows)
… FOR NO KEY UPDATE SKIP LOCKED;   → 12
```

Với `FOR UPDATE`, chỉ một lần ghi nhật ký trỏ tới việc là đủ khiến việc đó «biến mất» khỏi hàng đợi. `FOR NO KEY UPDATE` là khoá mà chính câu UPDATE bình thường (không đổi khoá) tự lấy — đủ để không hai worker nhận cùng việc, không chặn dòng con. Worker không xoá việc và không đổi `id` thì dùng `FOR NO KEY UPDATE`. Bất biến cuối §6 của lab: 12/12 việc `done`; việc duy nhất có số lần chạy khác 1 là **`7 | 2`** (mục 6).

### 7.4. Hết hạn thuê

Trường hợp thứ tư khiến việc chạy *thêm* một lần: lease dùng cột trạng thái, không dùng khoá dòng, nên worker khác nhận lại việc trong khi worker cũ có thể vẫn đang chạy (mục 6: `7 | A,B`).

| Trường hợp | SKIP LOCKED làm gì | Hệ quả | Xử lý |
|---|---|---|---|
| Worker treo giữ việc | Bỏ qua mãi | Việc (kể cả gấp nhất) bị bỏ đói | Giao dịch ngắn; `idle_in_transaction_session_timeout`; giám sát tuổi việc |
| Đếm/giám sát | Bỏ dòng đang khoá | Số sai (0 thay vì 1) | Đếm bằng `SELECT` thường |
| Dòng con chèn đồng thời | `FOR UPDATE` bỏ qua dòng chỉ bị FOR KEY SHARE | Việc «biến mất» tạm thời | `FOR NO KEY UPDATE SKIP LOCKED` |
| Lease hết hạn | Không còn khoá ⇒ nhận lại | Tác dụng ngoài DB chạy hai lần | Vé `attempts`; bên nhận chống trùng (PG-039) |

## 8. Lấy theo lô

Thí nghiệm §2: `… LIMIT 3 FOR NO KEY UPDATE SKIP LOCKED` trong hai giao dịch mở — A nhận **2, 3, 4**, B nhận **5, 6, 7**, không chờ. Lô lớn giảm số lượt hỏi nhưng giữ nhiều dòng lâu hơn và khiến mọi vấn đề ở mục 7 tệ hơn: một worker treo giữ cả lô. Tài liệu cũng lưu ý `LIMIT` dừng khoá khi đủ dòng, nhưng dòng bị `OFFSET` bỏ qua vẫn bị khoá — đừng dùng `OFFSET` trong câu lấy việc.

## 9. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Thêm worker không tăng thông lượng | `FOR UPDATE` chờ trên dòng đầu hàng | `SKIP LOCKED` |
| Log đầy lỗi `55P03` | Hàng đợi dùng `NOWAIT` + thử lại | `SKIP LOCKED` |
| Hai worker cùng xử lý một việc | `SELECT` không khoá rồi mới `UPDATE`, hoặc lease hết hạn | Khoá trong câu lấy việc; vé `attempts`; bên nhận chống trùng |
| Việc gấp nằm `queued` mãi | Worker treo trong giao dịch giữ khoá | Timeout phía worker, giao dịch ngắn |
| Bảng điều khiển báo hàng rỗng | Đếm bằng `SKIP LOCKED` | Đếm bằng `SELECT` thường |
| Việc biến mất khi có người ghi nhật ký | `FOR UPDATE` xung đột với FOR KEY SHARE | `FOR NO KEY UPDATE` |
| Worker cũ đánh dấu done việc người khác đang làm | Câu done không kèm điều kiện lượt thuê | `WHERE … AND attempts = :ve` |

## 10. Đánh đổi

- **Giữ giao dịch hay nhận ngắn.** Giữ giao dịch suốt lúc xử lý thì được «đúng một lần» với tác dụng trong database và tự về hàng khi worker chết; nhận ngắn + lease hợp với việc gọi mạng nhưng chỉ «ít nhất một lần».
- **Hàng đợi trong PostgreSQL hay broker riêng.** Một bảng `jobs` cho phép tạo việc *cùng giao dịch* với dữ liệu nghiệp vụ (nền tảng của outbox, PG-039) và không thêm hệ thống mới. Mỗi việc là vài lần UPDATE tạo phiên bản dòng chết; tải lớn cần chăm VACUUM (PG-032, Phần 5).

## 11. Bài tập

**Bài 1.** Viết câu «lấy, xử lý và đánh dấu xong một việc» bằng một câu lệnh, không thể bị chặn bởi dòng con được chèn đồng thời.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">WITH v AS (SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1
           FOR NO KEY UPDATE SKIP LOCKED),
     r AS (INSERT INTO job_runs (job_id, worker) SELECT id, 'B' FROM v)
UPDATE jobs SET status = 'done', done_at = now(), attempts = attempts + 1 FROM v WHERE jobs.id = v.id
RETURNING jobs.id;</code></pre>
<p>Đây là câu cuối §5b của lab: chạy khi C đang giữ FOR KEY SHARE trên việc 12 (sau đó rollback) vẫn trả <code>12</code>. Locking clause phải nằm <em>trong</em> CTE <code>v</code> — tài liệu nói mệnh đề khoá ở câu chính không áp vào truy vấn <code>WITH</code>.</p>
</details>

**Bài 2.** Worker A nhận việc theo kiểu «nhận ngắn», bị chậm quá hạn thuê, worker B đã nhận lại. Viết câu đánh dấu done của A sao cho A biết mình đã mất việc thay vì ghi đè.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">UPDATE jobs SET status = 'done', done_at = now(), locked_by = NULL, locked_at = NULL
WHERE id = 1 AND status = 'running' AND attempts = 1;   -- attempts = vé A nhận lúc lấy việc</code></pre>
<p>Thí nghiệm §1 ra <code>UPDATE 0</code> cho A, <code>UPDATE 1</code> cho B (vé 2); cuối cùng <code>done | 2 | A,B</code>. Vé chỉ bảo vệ trạng thái — SMS của A đã gửi, bên nhận vẫn phải chống trùng. Dùng <code>locked_by</code> làm vé không đủ khi cùng một worker nhận lại việc ở lượt sau.</p>
</details>

**Bài 3.** Bảng điều khiển hỏi «còn bao nhiêu việc gấp chưa ai làm, và việc nào đang bị giữ». Viết hai câu, nói rõ câu nào có thể chạy thường xuyên.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">-- thường xuyên: không khoá, không bỏ dòng
SELECT count(*) FROM jobs WHERE status = 'queued' AND priority = 1;
-- điều tra khi nghi có worker treo (đọc cả bảng, không nhất quán)
SELECT j.id, j.status, p.modes
FROM jobs AS j JOIN pgrowlocks('jobs') AS p ON p.locked_row = j.ctid;</code></pre>
<p>Ở thí nghiệm §3, câu thứ hai ra <code>11 | queued | {"For No Key Update"}</code> khi A treo. Không dùng <code>SKIP LOCKED</code> để đếm: lab cho <code>1 | 0</code>.</p>
</details>

## 12. Đọc tiếp

- [PostgreSQL 18 · SELECT — The Locking Clause](https://www.postgresql.org/docs/18/sql-select.html) — `NOWAIT`, `SKIP LOCKED`, `LIMIT`, `ORDER BY` lệch thứ tự ở Read Committed.
- [PostgreSQL 18 · Explicit Locking — Row-Level Locks](https://www.postgresql.org/docs/18/explicit-locking.html) — bốn chế độ khoá dòng và bảng xung đột.
- [PostgreSQL 18 · pgrowlocks](https://www.postgresql.org/docs/18/pgrowlocks.html).
- [PostgreSQL 18 · Client Connection Defaults](https://www.postgresql.org/docs/18/runtime-config-client.html) — `idle_in_transaction_session_timeout`, `statement_timeout`.
- Lab: `examples/toi-noi-postgresql/p04/36-for-update-skip-locked.sql`, `p04/OUTPUT.txt`, phép thử K036 trong `p04/DO-DAC.md`; thí nghiệm `p04/thu-nghiem/pg036-lease-fencing-lo-viec-idle-timeout.sql`.

Bài tiếp theo: [PG-037 · Deadlock, thứ tự lấy khoá và timeout](/articles/postgresql-deadlock-lock-order-timeout).
