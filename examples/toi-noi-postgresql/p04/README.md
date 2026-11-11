# P04 · Giao dịch, MVCC và đồng thời

Lab cho mười bài PG-031 … PG-040 trong [khung chương trình](../../../docs/postgresql-master-curriculum.md). Chạy trên **PostgreSQL 18 THẬT** (đã kiểm **18.1**) bằng runner chung [`pg/run-pg.mjs`](../README.md#p03-postgresql-18-that) như P03. Khác P03: dữ liệu NHỎ (vài chục dòng — mọi hiện tượng của phần này xảy ra trên vài dòng), gần như mọi tệp là **kịch bản nhiều phiên** có lịch chạy tất định, và có thêm **phép thử đồng thời** thật (`pg/run-dong-thoi.mjs`) cho PG-036 … PG-040.

Trạng thái kiểm chứng: **chạy thật** trên Windows 11 Pro, Intel Core i7-10700 (16 luồng), 64 GB RAM, NVMe SSD, Node 22.18.0, PostgreSQL 18.1 (bản cài EDB `C:\Program Files\PostgreSQL\18`). `npm run p04`: **613 bước, 0 FAIL** (17 expect-error, 47 expect-rows, 116 expect-value, 67 expect-line, 1 expect-md5, 11 expect-match/no-match, **25 expect-blocked, 3 expect-mat-ket-noi**), **44 giây** (nền 1,1 s + 10 bài 2–6 s/bài; phần lớn là các khoảng chờ CỐ Ý: timeout 200–300 ms, `pg_sleep`, runner chờ xác nhận bị chặn). Hai lần chạy độc lập cho đầu ra **giống hệt nhau** sau chuẩn hoá (test so khớp). Linux/macOS: **chưa chạy thử**.

## Chạy

```sh
cd examples/toi-noi-postgresql
npm run p04                 # nền + 10 tệp bài, in như p04/OUTPUT.txt, kết thúc "KẾT QUẢ: ĐẠT" (~45 s)
npm run p04:output          # ghi lại OUTPUT.txt sau khi sửa SQL
npm run p04:thu-nghiem      # thí nghiệm (p04/thu-nghiem/manifest.json)
npm run p04:measure         # PHÉP THỬ ĐỒNG THỜI cỡ đầy đủ (KHÔNG thuộc npm test) → ghi p04/DO-DAC.md
npm run p04:dong-thoi-nho   # cùng phép thử, cỡ NHỎ (~35 s), không ghi tệp — npm test chạy bản này
npm test                    # gồm test/p04.test.mjs: SKIP phần PostgreSQL nếu máy không có PostgreSQL 18

node pg/run-pg.mjs p04 --only 37                          # một bài (nền vẫn dựng, 1 s)
node pg/run-pg.mjs p04/thu-nghiem/manifest.json --only pg034
node pg/run-pg.mjs --nen p04/lab.json:p04 p04/thu-nghiem/moi.sql   # tệp CHƯA khai manifest, trên bản sao nền p04
node pg/run-dong-thoi.mjs p04/dong-thoi.mjs --only K038 --lap 5     # một nhóm kịch bản đồng thời, 5 lần
node pg/run-dong-thoi.mjs p04/dong-thoi.mjs --nho --json kq.json    # bản nhỏ, kết quả từng lần ra JSON
```

Prerequisites: Node ≥ 22; PostgreSQL ≥ 18 với contrib `pageinspect`, `pgstattuple` (bản cài chuẩn có sẵn); không cần dung lượng đáng kể (nền vài MB). Runner tìm binary qua `PG_BIN` › `pg_config --bindir` › `C:\Program Files\PostgreSQL\18\bin`; cấu hình cluster tạm, định dạng chỉ thị: [README gốc](../README.md#p03-postgresql-18-that). Mỗi tệp bài chạy trên một database MỚI chép từ nền ⇒ các bài độc lập (bài nào cũng bắt đầu với chuyến 400101–400110 ở trạng thái `requested`, ví 500.000 / 200.000 đ, sequence mã chuyến ở 400200).

## Nền (`lab.json` → `nen.p04`)

`p01/00-reset.sql` → `p01/01-schema.sql` → `p02/00-mo-rong-schema.sql` (chỉ LƯỢC ĐỒ, không fixture P01/P02) → ba tệp nền của P04 (dựng 1,1 s):

| Tệp | Nội dung |
|---|---|
| `00-mo-rong-schema.sql` | Chặn chạy thiếu nền / PostgreSQL < 18; extension `pageinspect`, `pgstattuple` (schema public). Thêm `trips.version` (+ `ck_trips_version_positive`); bảng `wallets` (ví VíXanh, `ck_wallets_balance_non_negative`), `trip_offers`, `trip_assignments` (**PK trip_id = một chuyến một tài xế**), `trip_claims` (nhật ký "bạn đã nhận chuyến" — CỐ Ý không UNIQUE), `driver_duty` (+ `ix_driver_duty_area`), `jobs` (+ partial index `ix_jobs_queued (priority, id) WHERE status = 'queued'`), `job_runs` (CỐ Ý không UNIQUE), `idempotency_keys` (PK key, `request_hash`), `outbox` (+ `ix_outbox_unpublished`), `broker_messages` (giả lập broker, CỐ Ý không UNIQUE), `processed_messages` (PK consumer + outbox_id), `notifications`. **Hàm tiện ích cho đầu ra tất định** (không phải mẫu production): `lab_dat_moc_xid()` + `xid_td(xid|xid8)` → `X+1`, `X+2`… (`<X` = trước mốc), `xid_x(n)` → xid8 của `X+n`, `snap_td(pg_snapshot)`, `lab_nhan(pid)` → nhãn phiên, **`lab_khoa()`** (pg_locks theo nhãn phiên: relation chỉ BẢNG trong schema toinoi, tuple, transactionid "xid của A", advisory; bỏ virtualxid), **`lab_cho()`** (pg_stat_activity: trạng thái, wait_event, `pg_blocking_pids` → nhãn, câu lệnh cắt 60 ký tự) |
| `01-fixture.sql` | 3 thành phố, 34 khu (y P02), 20 khách `0940000001…20`, 24 tài xế (SG 9101–9112, HN 9201–9208, DN 9301–9304, đều online), 2 quán, **40 chuyến**: 30 đã xong ngày 30/10 (mã 400001–400030; 27 hoàn thành, 3 huỷ: 400008/16/24) + **10 đang chờ nhận** (mã 400101–400110, khu 112/113/111 xoay vòng, cước 31.000 + 1.000·(n − 1) với n = mã − 400100, version 1); 100 sự kiện; 44 ví (khách 1001–1020 × 500.000 đ; tài xế 2001–2024 × 200.000 đ; tổng 14.800.000 đ); tài xế trực: **mỗi khu 2 người** (111: 9101, 9102 · 112: 9103, 9104 · 113: 9105, 9106 · 121 · 122 · 123 · 211–213, 221 · 311, 312); 10 việc `gui_thong_bao` (việc 1 ưu tiên 1, còn lại 5). Id dễ đọc: chuyến `a4000000-…-<mã>`, khách `c4000000-…-<n>`, tài xế `d4000000-…-<mã>`. `VACUUM (ANALYZE)` cuối tệp (bảng nhỏ ⇒ thống kê tất định) |
| `02-kiem-tra-fixture.sql` | Số dòng, trạng thái, md5 10 chuyến chờ, tổng ví, bất biến "mỗi khu 2 người trực", hàng đợi |

ctid của chuyến chờ: 400101 ở `(0,31)` … 400110 ở `(0,40)` — cả bảng `trips` nằm trong page 0 (dùng ở PG-032).

## Tệp → bài

| Tệp | Bài (slug) | Mức | Phiên | Minh hoạ |
|---|---|---|---|---|
| `31-transaction-commit-rollback.sql` | PG-031 `postgresql-transaction-commit-rollback` | CB | A, B | Chuyến + sự kiện cùng commit; B không thấy dữ liệu chưa commit; `now()` cố định trong giao dịch; lỗi giữa giao dịch ⇒ `25P02`, COMMIT trả `ROLLBACK`; SAVEPOINT / ROLLBACK TO; psql `ON_ERROR_ROLLBACK`; `\set AUTOCOMMIT off`; sequence không quay lui (mã nhảy cóc); **mất kết nối giữa giao dịch** (`pg_terminate_backend`) ⇒ không nửa vời; `idle_in_transaction_session_timeout` |
| `32-mvcc-snapshot.sql` | PG-032 `postgresql-mvcc-snapshot` | CB | A, B | xid chỉ cấp khi ghi; xmin/xmax của bản cũ và bản mới khi A chưa commit; `pg_current_snapshot()` (xmin/xmax/đang chạy); `pg_xact_status`; hai phiên bản trong page (pageinspect, HOT); snapshot câu lệnh (RC) vs giao dịch (RR); ROLLBACK để lại dòng chết + xmax của giao dịch abort; `pgstattuple` 3 dòng chết → VACUUM → 0; xid vs xid8 vs `txid_current()` |
| `33-khoa-dong-khoa-bang.sql` | PG-033 `postgresql-lock-row-table` | CB | A, B, C | UPDATE chặn UPDATE, không chặn SELECT; `lab_cho()` + `lab_khoa()` (pg_stat_activity, pg_locks, pg_blocking_pids); khoá dòng nằm trong xmax; FOR UPDATE chặn INSERT dòng con (FK = FOR KEY SHARE), FOR NO KEY UPDATE thì không; DDL ACCESS EXCLUSIVE chặn SELECT; DDL với `lock_timeout` ⇒ `55P03`; `statement_timeout` ⇒ `57014` (khi chờ khoá và khi `pg_sleep`); advisory lock mức phiên / mức giao dịch |
| `34-read-committed-repeatable-read.sql` | PG-034 `postgresql-isolation-read-committed-repeatable-read` | TB | A, B, C | CÙNG 4 lịch S1–S4 dưới RC rồi RR: đọc lại, phantom, lost update (đọc–tính–ghi), ghi tương đối; `40001` + thử lại; `default_transaction_isolation`; READ UNCOMMITTED = RC |
| `35-optimistic-version.sql` | PG-035 `postgresql-optimistic-version-conditional-update` | TB | A, B | Form ghi đè làm chuyến đã huỷ "sống lại"; `UPDATE … WHERE version = :v` + đọc ROW_COUNT; hai lần lưu chồng nhau (B chờ rồi 0 dòng); chuyển trạng thái có điều kiện; trigger tự tăng version |
| `36-for-update-skip-locked.sql` | PG-036 `postgresql-for-update-skip-locked` | TB | A, B, C | FOR UPDATE (xếp hàng) · NOWAIT (`55P03`) · SKIP LOCKED (4/5/6 ngay); mẫu "nhận việc ngắn" + thời hạn thuê ⇒ việc 7 chạy HAI lần (at-least-once); SKIP LOCKED **bỏ đói** việc ưu tiên 1 khi worker treo, đếm bằng SKIP LOCKED sai (0 vs 1); bẫy FK: FOR UPDATE SKIP LOCKED bỏ qua dòng chỉ bị FOR KEY SHARE |
| `37-deadlock-thu-tu-khoa.sql` | PG-037 `postgresql-deadlock-lock-order-timeout` | TB | A, B, C | Thanh toán (1001→2001) và hoàn tiền (2001→1001) chéo nhau ⇒ `40P01` (B — `deadlock_timeout` 200 ms, A 10 s); **nhật ký server có câu lệnh của cả hai phiên**; thử lại; sửa bằng `SELECT … ORDER BY id FOR NO KEY UPDATE`; `lock_timeout` làm lưới an toàn; bất biến tổng tiền |
| `38-serializable-retry.sql` | PG-038 `postgresql-serializable-retry` | NC | A, B, C | Write skew "mỗi khu ≥ 1 người trực": RR để lọt (khu 111 còn 0 người), SERIALIZABLE ⇒ COMMIT của B `40001` "pivot, during commit attempt", thử lại ⇒ từ chối; SIReadLock mức bảng (Seq Scan) ⇒ **báo động giả** giữa hai khu khác nhau; ép index ⇒ SIReadLock mức dòng ⇒ cả hai commit; cách khác: khoá dòng khu (`areas … FOR NO KEY UPDATE`) |
| `39-idempotency-outbox.sql` | PG-039 `postgresql-idempotency-outbox-transaction` | NC | A, B, C, D | INSERT … ON CONFLICT DO NOTHING trên khoá idempotency, lưu và trả lại kết quả lần đầu; khoá dùng cho request khác ⇒ lỗi; hàm `dat_chuyen()`; request trùng ĐỒNG THỜI (B chờ ở index duy nhất rồi trả kết quả của A); request đầu bị huỷ giữa chừng ⇒ request lặp tự làm, không nửa vời; publisher SKIP LOCKED, chết giữa "gửi" và "đánh dấu" ⇒ broker nhận TRÙNG; consumer chống trùng bằng `processed_messages` |
| `40-thuc-hanh-nhan-chuyen.sql` | PG-040 `postgresql-thuc-hanh-tranh-chap-nhan-chuyen` | NC | A, B, C | Bốn cách nhận chuyến — (a) đọc rồi ghi không khoá ⇒ HAI người thắng; (b) UPDATE có điều kiện; (c) FOR UPDATE; (d) PK `trip_assignments` (`23505` / ON CONFLICT) — với lịch chạy tất định 2–3 tài xế; bảng bàn giao bất biến |
| `thu-nghiem/` | PG-033, PG-034 | — | A, B | Xem [thu-nghiem/README.md](thu-nghiem/README.md) |
| `dong-thoi.mjs` → `DO-DAC.md` | PG-036 … PG-040 | — | 8 phiên | Phép thử đồng thời lặp nhiều lần (xem dưới) |

## Số liệu và lịch chạy then chốt từng bài

Mọi số dưới đây có trong `OUTPUT.txt` (tất định). Ký hiệu lịch: `A:` = bước của phiên A, `⏳` = runner thấy bước đó đang chờ khoá (`wait_event_type = 'Lock'`), `✔` = bước đang chờ chạy xong sau khi được nhả. xid in tương đối: `X` = mốc, `X+1` = giao dịch ghi đầu tiên sau mốc, `<X` = dữ liệu từ trước mốc (fixture).

**PG-031.** Lịch §1: `A: BEGIN` · `A: INSERT trips … RETURNING code` → **400200** · `A: INSERT trip_events` · `B: đếm` → **0 | 0** · `A: requested_at = occurred_at` → **t** (`now()` = đầu giao dịch) · `A: COMMIT` · `B: đếm` → **1 | 1**. §2: chèn chuyến (mã **400201**) rồi sự kiện `'dang_tim'` ⇒ `23514 ck_trip_events_status` ⇒ `SELECT 1` ⇒ **`25P02` "current transaction is aborted, commands ignored until end of transaction block"** ⇒ `COMMIT` in thẻ **`ROLLBACK`** ⇒ B: 0 chuyến. §3 SAVEPOINT: chuyến **400202** + 1 sự kiện sau `ROLLBACK TO SAVEPOINT`; `\set ON_ERROR_ROLLBACK on`: chuyến **400203**, câu lỗi không làm hỏng giao dịch. §4 `\set AUTOCOMMIT off`: chuyến **400204**, `pg_current_xact_id_if_assigned() IS NOT NULL` = t, B thấy 0 tới khi A `COMMIT`. §5 `B: pg_terminate_backend(pid, 5000)` → t, nhật ký `[A] FATAL: terminating connection due to administrator command` · `A: COMMIT` → **MẤT KẾT NỐI** · B: **0 | 0** · A kết nối lại: `da_luu = f`; `idle_in_transaction_session_timeout = 200ms` + `B: pg_sleep(0.5)` ⇒ nhật ký `[A] FATAL: terminating connection due to idle-in-transaction timeout`, chuyến 400206 không còn. Bất biến cuối: chuyến 400200/400202/400203/400204 mỗi chuyến 1 sự kiện requested; mã **400201, 400205, 400206 bị "đốt"**; 0 chuyến thiếu sự kiện.

**PG-032.** `A: BEGIN; SELECT` → `pg_current_xact_id_if_assigned()` = **NULL**; `A: UPDATE 400101 fare 35000` RETURNING → **35000 | 2 | X+1 | 0**. B (A chưa commit): **31000 | 1 | <X | X+1** (bản cũ, xmax = A); `snap_td(pg_current_snapshot())` = **`xmin=X+1 xmax=X+1 đang chạy={}`** (xmax của snapshot = xid lớn nhất ĐÃ KẾT THÚC + 1 — X+1 ≥ xmax nên vô hình mà không cần nằm trong danh sách; bẫy dễ giải thích sai); `pg_xact_status(X+1)` = **in progress**; pageinspect page 0: ô **31** `<X | X+1 | (0,41) | hot_updated t` và ô **41** `X+1 | 0 | (0,41) | heap_only t`. Sau COMMIT: B thấy **35000 | 2 | X+1 | 0**, `committed`, snapshot `xmin=X+2 xmax=X+2 đang chạy={}`. RR: B chụp 32000 ⇒ A (X+2) đổi 36000 ⇒ B vẫn **32000 | <X | X+2** tới COMMIT. ROLLBACK: 400103 vẫn **33000 | <X | X+3 | aborted** (xmax ≠ 0 nhưng dòng sống); `pgstattuple` **40 | 3** dòng chết → VACUUM → **40 | 0**. Kiểu: **xid8 | bigint | xid | t**.

**PG-033.** Lịch §1: `A: BEGIN; UPDATE 400104 +1000` → 35000 · `B: SELECT` → 34000 (không chờ) · `B: UPDATE +500` ⏳ **Lock: transactionid — bị chặn bởi [A]** · `C: lab_cho()` → `A | idle in transaction | Client: ClientRead` / `B | active | Lock: transactionid | A` · `C: lab_khoa()` → 6 dòng: A `relation trips RowExclusiveLock t`, A `transactionid xid của A ExclusiveLock t`, B `relation trips RowExclusiveLock t`, **B `transactionid xid của A ShareLock f`**, B `transactionid xid của B ExclusiveLock t`, **B `tuple trips (0,34) ExclusiveLock t`** · `A: COMMIT` · B ✔ **35500** (cộng trên bản mới, version 3). §2: `A: FOR UPDATE` ⇒ xmax của dòng = xid của A (khoá dòng không có trong pg_locks); `B: INSERT trip_events` cho chuyến đó ⏳ (FK cần FOR KEY SHARE); với `FOR NO KEY UPDATE` thì INSERT **không** chờ, nhưng FOR NO KEY UPDATE thứ hai chờ và đọc lại `cancelled`. §3: `ALTER TABLE … ADD COLUMN` trong giao dịch ⇒ `SELECT count(*)` ⏳ **Lock: relation**; lab_khoa: A `AccessExclusiveLock t`, B `AccessShareLock f`; ROLLBACK ⇒ B ✔ 40. DDL với `lock_timeout 300ms` ⇒ ⏳ rồi **`55P03` "canceling statement due to lock timeout"**, C đọc bình thường (không xếp hàng sau DDL). §4: `statement_timeout 300ms` ⇒ **`57014`** cả khi chờ khoá lẫn `pg_sleep(1)`; giao dịch B hỏng (`25P02`); 400106 = **37000**. §5 advisory: `pg_try_advisory_lock` = f, `pg_advisory_lock` ⏳ **Lock: advisory**; lab_khoa `advisory 400107 ExclusiveLock t/f`; khoá mức phiên phải `pg_advisory_unlock` (cả hai bên trả t).

**PG-034.** Bảng so sánh (cùng lịch, cùng dữ liệu, đầu tệp có bảng tóm tắt):

| Lịch | READ COMMITTED | REPEATABLE READ |
|---|---|---|
| S1 đọc lại 400107 sau khi B sửa | 37000 → **40000** | 37000 → **37000** (giao dịch sau: 40000) |
| S2 đếm chuyến chờ khu 113 sau khi B chèn | 3 → **4** (phantom) | 3 → **3** |
| S3 A, B cùng đọc 500000, A ghi 520000 (commit), B ghi 550000 | B ⏳ rồi ✔ ⇒ **550000 — mất +20000 của A** | B ⏳ rồi ✔ **`40001` "could not serialize access due to concurrent update"**, COMMIT → ROLLBACK; 520000; B thử lại ⇒ **570000** |
| S4 `balance = balance + x` | B ⏳ rồi ✔ RETURNING **570000** (đọc lại bản mới) | B ⏳ rồi ✔ **`40001`**; 520000 (phải thử lại) |

**PG-035.** §1 không kiểm: A đọc **38000 | requested | 1**, B huỷ, A lưu form ⇒ **45000 | requested** (lệnh huỷ mất). §2: B huỷ `WHERE version = 1` → **cancelled | 2**; A `WHERE version = :v` → **0 dòng**, `:ROW_COUNT` = **0**; đọc lại **38000 | cancelled | 2**. §3 chồng nhau: A (chưa commit) → version 3; B `WHERE version = 2` ⏳ rồi ✔ **0 dòng**; kết quả **41000 | 3**. §4: hai tài xế nhận 400109 `WHERE status = 'requested'` → A 1 dòng (accepted), B **0 dòng**; huỷ `WHERE status IN (…)` → cancelled; "đã đón" sau khi huỷ → 0 dòng. §5 trigger `tang_version`: UPDATE quên version vẫn 1 → **2**; UPDATE có `version = version + 1` → **3** (không tăng đôi).

**PG-036.** Thứ tự xử lý (bảng `job_runs` cuối bài): **1 A · 2 B · 3 A · 4 A · 5 B · 6 C · 7 A,B · 8 B · 9 C · 10 B · 11 B · 12 B**; 12/12 `done`; việc duy nhất ≠ 1 lần là **7 | 2**. §1 FOR UPDATE: A lấy 1, B ⏳ rồi ✔ **2** (bỏ dòng 1 đã done, lấy tiếp). §2 NOWAIT: **`55P03` "could not obtain lock on row in relation "jobs""**. §3 SKIP LOCKED: A, B, C nhận **4, 5, 6** không chờ. §4 lease 200 ms: A nhận 7 (`running`, attempts 1), "gửi SMS", chết ⇒ dọn dẹp trả 7 về hàng (attempts 1) ⇒ B nhận 7 (attempts 2) ⇒ **7 chạy hai lần (A,B)**. §5 bỏ đói: việc 11 (ưu tiên 1) bị A treo giữ; B, C làm 8, 9, 10; SKIP LOCKED của B trả **0 dòng**; đếm `1 | 0` (thường vs SKIP LOCKED); `pg_terminate_backend(A)` ⇒ B lấy **11**. §5b: C chèn `job_runs` cho việc 12 (FOR KEY SHARE) ⇒ `FOR UPDATE SKIP LOCKED` → **0 dòng**, `FOR NO KEY UPDATE SKIP LOCKED` → **12**.

**PG-037.** Lịch §1: `A: BEGIN; 1001 −30000` → 470000 · `B: BEGIN; 2001 −10000` → 190000 · `A: 2001 +30000` ⏳ **bị chặn bởi [B]** · `C: lab_cho()` → `A | active | Lock: transactionid | B`, `B | idle in transaction` · `B: 1001 +10000` (`lab: cho-xong`) ⏳ **bị chặn bởi [A]** rồi **`40P01` deadlock detected** — `DETAIL: Process [B] waits for ShareLock on transaction <XID>; blocked by process [A]. / Process [A] waits for ShareLock on transaction <XID>; blocked by process [B].` + `HINT: See server log for query details.`; nhật ký server (log_lock_waits on cho B): `LOG: process [B] detected deadlock while waiting for ShareLock on transaction <XID> after T ms`, `DETAIL: Process holding the lock: [A]. Wait queue: .`, rồi `ERROR: deadlock detected` kèm **câu lệnh của CẢ HAI phiên** (`Process [B]: UPDATE … id = 1001`, `Process [A]: UPDATE … id = 2001`) · A ✔ **230000** (khoản trừ của B đã huỷ) · `A: COMMIT` · `B: COMMIT` → ROLLBACK ⇒ **1001 = 470000, 2001 = 230000, tổng 14.800.000**; B thử lại ⇒ 480000 / 220000. §2 khoá theo id: B ⏳ ở `SELECT … ORDER BY id FOR NO KEY UPDATE` (chưa giữ gì) ⇒ không vòng ⇒ **460000 / 240000**. §3 `SET LOCAL lock_timeout = '300ms'` ⇒ ⏳ rồi `55P03`. Tổng cuối **14.800.000**.

**PG-038.** §1 RR: A, B đều đếm **2**, mỗi người tắt trực của mình, cả hai COMMIT ⇒ khu 111 còn **0** người, `khu_vi_pham() = 1`. §2 SERIALIZABLE: `lab_khoa()` → A, B `relation driver_duty SIReadLock`; hai UPDATE không chờ; `A: COMMIT` ok; `B: COMMIT` ⇒ **`40001` "could not serialize access due to read/write dependencies among transactions" — `DETAIL: Reason code: Canceled on identification as a pivot, during commit attempt.`** ⇒ còn **1 | 0**; B thử lại: đếm **1** ⇒ từ chối, ROLLBACK. §3 báo động giả: A ở khu 112, B ở khu 113 (không liên quan) ⇒ B COMMIT vẫn **`40001`** (Seq Scan ⇒ SIReadLock cả bảng); `SET enable_seqscan = off` ⇒ SIReadLock chỉ trên tuple (`driver_duty (0,35)`, `(0,37)`) + page index ⇒ **cả hai COMMIT**, 112 và 113 mỗi khu còn 1. §4 khoá dòng `areas 111 FOR NO KEY UPDATE` ở RC: B ⏳, sau khi A commit B đếm **1** ⇒ từ chối. Cuối: `khu_vi_pham() = 0`.

**PG-039.** §1: lần đầu `INSERT … ON CONFLICT DO NOTHING RETURNING key` → 1 dòng, chuyến **400200**, outbox id **1**, response **`{"status": "requested", "trip_code": 400200}`**; gửi lại → **0 dòng**, trả response + `cung_noi_dung = t`. §2 `dat_chuyen()`: lần 1 `{"status": "requested", "lap_lai": false, "trip_code": 400201}`, lần 2 cùng mã `lap_lai: true`, khác nội dung (fare 99000) ⇒ **`P0001` "idempotency key dat-0002-bbbb đã dùng cho một request khác"**. §3 đồng thời: A (chưa commit) tạo **400202**; B ⏳ **Lock: transactionid — bị chặn bởi [A]** (chờ ở pk_idempotency_keys); A COMMIT ⇒ B ✔ `lap_lai: true, trip_code: 400202`. §4: A tạo 400203 (chưa commit), B ⏳; `C: pg_terminate_backend(A)` ⇒ B ✔ **tự tạo 400204** (`lap_lai: false`); A `COMMIT` → MẤT KẾT NỐI; 400203 = **0 dòng**; bảng khoá → chuyến: `dat-0001…400200`, `0002…400201`, `0003…400202`, `0004…400204`, mỗi chuyến **1 sự kiện, 1 outbox**. §5: outbox id **1, 2, 3, 5** (id 4 thuộc giao dịch đã huỷ); publisher A lấy 1, 2, gửi broker, bị huỷ trước khi đánh dấu; publisher B lấy **1, 2, 3, 5**, gửi, đánh dấu ⇒ broker: tin 1 và 2 **mỗi tin 2 lần** (`publisher-A,publisher-B`). §6 consumer: seq 1, 2, 5, 6 "đã gửi thông báo", seq 3, 4 "bỏ qua (đã xử lý)" ⇒ **mỗi sự kiện đúng 1 thông báo**.

**PG-040.** (a) không khoá: cả A và B đọc `requested`, A ghi + commit, B ghi đè + commit ⇒ claim **9101 | 9102** và **9102 | 9102** (hai người được báo nhận, DB chỉ còn 9102). (b) có điều kiện, 3 tài xế: A 1 dòng (9101); B ⏳ **bị chặn bởi [A]** (Lock: transactionid); C ⏳ **Lock: tuple — bị chặn bởi [B]** (xếp hàng sau B); A COMMIT ⇒ B ✔ 0 dòng, C ✔ 0 dòng. (c) FOR UPDATE: B ⏳ rồi ✔ đọc **accepted** ⇒ không ghi. (d) PK: B ⏳ rồi ✔ **`23505 pk_trip_assignments`**; C (`ON CONFLICT DO NOTHING`) ⏳ rồi ✔ **0 dòng**. Bảng bàn giao: `400101 | a_khong_khoa | 2 | 1 | VI PHẠM`, `400102 | b_co_dieu_kien | 1 | 1 | đạt`, `400103 | c_for_update | 1 | 1 | đạt`, `400104 | d_unique | 1 | 1 | đạt`; phân công `400104 | 9101 | 9101`.

## Phép thử đồng thời (`dong-thoi.mjs` → `DO-DAC.md`)

`npm run p04:measure` = `node pg/run-dong-thoi.mjs p04/dong-thoi.mjs`: mỗi kịch bản mở **8 phiên psql thật** chạy ĐUA nhau (không phải lịch tất định), lặp **3 lần**, mỗi lần trên database mới chép từ nền; đếm kết quả phía client và **kiểm bất biến bằng SQL** sau mỗi lần; ghi `p04/DO-DAC.md` kèm cấu hình máy. `npm test` chạy cỡ nhỏ (`--nho`: 4 phiên, 1 lần) và chỉ kiểm bất biến "phải đúng" (số đếm dao động — KHÔNG so). Cỡ đầy đủ / nhỏ:

| Kịch bản | Bài | Tải đầy đủ (nhỏ) | Bất biến kiểm cứng |
|---|---|---|---|
| K040-a … K040-e | PG-040 | 1.000 chuyến × 8 tài xế, mỗi chuyến cả 8 cùng bấm (rào) (60 × 4) | (b)(c)(d)(e): mỗi chuyến **đúng 1** người thắng, người thắng = `trips.driver_id`, không chuyến nào không ai thắng; (a) cố ý sai ⇒ chỉ ĐẾM vi phạm |
| K036-skip / -cho / -nowait | PG-036 | 600 việc × 8 worker, "xử lý" `pg_sleep(0,002)` khi giữ khoá (60 × 4) | mọi việc `done`, mỗi việc **đúng 1** `job_runs` |
| K037-theo-chieu / -theo-id | PG-037 | 8 phiên × 150 chuyển tiền giữa 4 ví, `deadlock_timeout 20ms`, thử lại ≤ 20 (4 × 25) | tổng 4 ví = 800.000; 0 giao dịch bỏ cuộc; **theo-id: 0 lần 40P01** |
| K038-rr / -ser / -ser-index | PG-038 | 200 vòng × 8 tài xế (4 khu × 2 người) cùng xin nghỉ (20 × 4) | SERIALIZABLE: **0 khu** hết người trực qua mọi vòng (thử lại ≤ 10, backoff full jitter trần 50 ms); RR: chỉ đếm |
| K039 | PG-039 | 400 khoá × 2 request trùng đồng thời, 10 % request chết trước COMMIT, 4 publisher (10 % chết sau khi gửi), 2 consumer nhận MỌI tin (40 khoá, 20 %) | mỗi khoá đúng 1 chuyến; số chuyến = số khoá; mỗi chuyến 1 sự kiện + 1 outbox; 0 outbox chưa gửi; mỗi sự kiện **đúng 1** thông báo; tin trùng ở broker chỉ ĐẾM |

Kết quả cỡ đầy đủ (trích `DO-DAC.md`, máy ở dòng đầu README này, 3 lần — xem tệp để có từng lần):

Lệnh chạy 7,4 phút (11/11/2026, tải CPU nền 6 %); **mọi bất biến "phải đúng" ĐẠT ở cả 3 lần** của mọi kịch bản.

| Kịch bản | Giây/lần (min–max) | Kết quả then chốt (3 lần) |
|---|---|---|
| K040-a không khoá | 7,26–7,33 | **1.000/1.000 chuyến có > 1 người thắng** (cả 8 tài xế được báo "đã nhận" — 8.000 lượt; 7.000 lượt ≠ `trips.driver_id`) — mọi lần |
| K040-b UPDATE có điều kiện | 1,3–1,5 | 1.000 thắng / 7.000 thua, 0 vi phạm — nhanh nhất (một câu, autocommit) |
| K040-c FOR UPDATE | 3,9–4,0 | 1.000 / 7.000, 0 vi phạm |
| K040-d PK trip_assignments | 3,3–3,9 | 1.000 / 7.000, 0 vi phạm |
| K040-e code (a) + REPEATABLE READ + thử lại | 15,5–17,0 | 1.000 / 7.000, 0 vi phạm, **7.000 lần `40001` + thử lại mỗi lần** (đúng 7 = số người thua mỗi chuyến) |
| K036-skip | 1,21–1,32 (456–496 việc/s) | 600 việc, mỗi worker 74–76 việc, `rong_gia` 2–7 |
| K036-cho (FOR UPDATE) | 9,4 (≈ 64 việc/s) | 600 việc nhưng chia lệch: worker ít nhất **0–2**, nhiều nhất 115–159 việc (xếp hàng nối đuôi) |
| K036-nowait | 9,3 | **25.317–26.774 lỗi `55P03`** cho 600 việc; mỗi worker 56–91 việc |
| K037-theo-chieu | 30,8–32,9 | 1.200 chuyển tiền xong, **1.122–1.184 lần `40P01`** (gần 1 deadlock mỗi giao dịch), tổng 4 ví 800.000 |
| K037-theo-id | 14,5–14,9 | 1.200 xong, **0 lần `40P01`**, tổng 800.000 — nhanh gấp ~2 lần |
| K038-rr | 0,9 | **800/800 khu-vòng hết người trực** (cả hai người trong khu được nghỉ ở MỌI vòng: 1.600 được nghỉ) |
| K038-ser (Seq Scan) | 9,6–11,1 | 0 vi phạm; 800 được nghỉ / 800 từ chối; **2.488–2.539 lần `40001`** (báo động giả giữa các khu) |
| K038-ser-index | 3,2–3,5 | 0 vi phạm; 800 / 800; **đúng 800 lần `40001`** (= một xung đột thật mỗi khu mỗi vòng) |
| K039 | 7,9–9,0 | 400 khoá ⇒ 400 chuyến (400 tạo mới + 400 trả lại kết quả cũ), 78–80 request chết, 9–15 publisher chết ⇒ **45–75 tin trùng ở broker**, consumer bỏ qua 490–550 lần, **mỗi sự kiện đúng 1 thông báo** |

Đọc thêm: `pg_sleep(0,002)` / `pg_sleep(0,001)` trên Windows thực tế ngủ **~15,6 ms** (độ phân giải bộ định thời — K036-cho xử lý tuần tự 600 việc trong 9,4 s ≈ 15,6 ms/việc), nên thời gian K036/K037 bị chi phối bởi bước "xử lý"/"nghỉ" giả lập, không phải bởi PostgreSQL; trên Linux các số này sẽ nhỏ hơn nhiều. So TỈ LỆ giữa các cách trong cùng nhóm.

Bản nhỏ (`npm run p04:dong-thoi-nho`, lần chạy trong `npm test`) đã cho cùng hình dạng: K040-a 60/60 chuyến có 4 người thắng (240 lượt "đã nhận"), (b)(c)(d)(e) 60 lượt thắng / 180 thua, (e) 180 lần `40001` + thử lại; K037-theo-chieu 24 lần `40P01` / theo-id 0; K038-rr 40 khu vi phạm / 20 vòng, -ser 78 lần thử lại, -ser-index 40; K039 15 tin trùng, 0 vi phạm.

## Không chạy được / chưa chạy

- **Linux/macOS: chưa chạy thử** (mã đã tính tới như P03). Đặc biệt `pg_terminate_backend` + psql thoát mã 2 và thông điệp "server closed the connection unexpectedly" mới kiểm trên Windows; runner in dạng chuẩn hoá nên không phụ thuộc chữ của libpq.
- **Mất kết nối do MẠNG** (đứt TCP giữa client và server, `tcp_keepalives_*`, server bị kill -9 / mất điện) — chưa chạy; lab chỉ mô phỏng bằng `pg_terminate_backend`, `idle_in_transaction_session_timeout`, `transaction_timeout` và (trong phép thử đồng thời) giết tiến trình psql. Chưa chạy — thiết kế: dùng `--keep` rồi `pg_ctl stop -m immediate` giữa giao dịch, khởi động lại, đối chiếu.
- **"COMMIT đã tới server nhưng client không nhận được phản hồi"** (mục §5 PG-031 nói tới) — chưa tái hiện được tất định bằng runner (cần cắt mạng đúng sau khi server commit). Bài nên trình bày như lập luận + cách xử lý (đọc lại trạng thái / idempotency key), không như kết quả lab.
- **Hai-pha (PREPARE TRANSACTION)**, `SERIALIZABLE READ ONLY DEFERRABLE`, advisory lock theo hai khoá int4, `pg_locks` của SIReadLock trên page index (lab_khoa lọc bỏ index) — chưa có tệp.
- Broker trong PG-039 là bảng giả lập trong cùng database (kết nối riêng, không commit chung) — chưa nối broker thật.

## Dọn

Runner tự dừng và xoá cluster tạm (`%TEMP%\toinoi-pg-*`) sau mỗi lệnh, kể cả lệnh đồng thời. Nếu bị giết thô: xem [README gốc › Dọn môi trường](../README.md#dọn-môi-trường). Không có dữ liệu nào ghi ra ngoài thư mục tạm và `p04/` (OUTPUT.txt, DO-DAC.md).

## Ghi chú cho người viết bài

- **Lịch chạy là thứ tự GỬI, không phải thứ tự THỰC HIỆN.** Một bước "⏳ đang chờ khoá" chưa chạy xong; kết quả của nó in ở dòng "✔ bước dòng N đã chạy xong" sau bước làm nó được nhả (thường là COMMIT/ROLLBACK của phiên khác). Khi trích lịch vào bài, đặt kết quả của B SAU COMMIT của A.
- **`xid` trong bài là tương đối** (`X+1`…). Nếu bài in xid thật, nói rõ chúng là số toàn cluster, tăng theo mọi giao dịch GHI (giao dịch chỉ đọc không tốn xid), và khác trên máy người đọc.
- **xmax của snapshot ≠ "xid kế tiếp sẽ cấp"**: PostgreSQL 18 đặt xmax = (xid lớn nhất đã kết thúc) + 1. Ở PG-032, giao dịch đang chạy X+1 không nằm trong danh sách "đang chạy" (`{}`) vì X+1 ≥ xmax. Đừng vẽ sơ đồ snapshot với xmax = xid mới nhất + 1.
- **xmax ≠ 0 không có nghĩa dòng đã bị xoá**: nó cũng là dấu khoá dòng (FOR UPDATE — PG-033 §2) và còn nguyên sau ROLLBACK (PG-032 §4). Khoá dòng KHÔNG xuất hiện trong `pg_locks` (chỉ có khoá trên xid và khoá `tuple` của người chờ).
- **FOR UPDATE quá mạnh cho "đọc rồi sửa"**: nó chặn cả INSERT dòng con (FK lấy FOR KEY SHARE) và làm SKIP LOCKED bỏ qua dòng chỉ bị FOR KEY SHARE (PG-036 §5b). Khuyên `FOR NO KEY UPDATE` khi không xoá/đổi khoá chính.
- **Read Committed "đọc lại dòng" khi UPDATE gặp khoá** (EvalPlanQual) — vì vậy `balance = balance + x` và `WHERE version = :v` / `WHERE status = 'requested'` ĐÚNG ở RC dù chạy đồng thời (B chờ rồi 0 dòng). Nhưng câu đó không quét lại bảng: dòng vừa đổi để TRỞ NÊN khớp bị bỏ sót (thí nghiệm `pg034-…`). Repeatable Read thì báo `40001` thay vì đọc lại — kể cả với ghi tương đối (S4).
- **Lỗi giữa giao dịch: COMMIT không báo lỗi mà trả thẻ `ROLLBACK`** — driver/ORM phải kiểm thẻ hoặc trạng thái giao dịch. psql `ON_ERROR_ROLLBACK` chỉ là tiện ích của psql (savepoint ngầm mỗi câu).
- **Ai là nạn nhân deadlock** phụ thuộc phiên nào chờ đủ `deadlock_timeout` trước — lab ép B (200 ms) vs A (10 s). Với cấu hình mặc định (1 s cho cả hai), thường là phiên CHỜ SAU (đến lượt kiểm sau) nhưng không nên viết thành quy tắc. Chi tiết câu lệnh của các bên chỉ có trong **nhật ký server**, không có trong lỗi trả cho client.
- **SERIALIZABLE**: lỗi `40001` ở ví dụ write skew rơi vào **COMMIT** của giao dịch sau (không phải câu UPDATE) — ứng dụng phải sẵn sàng thử lại khi COMMIT lỗi. Bảng nhỏ ⇒ Seq Scan ⇒ SIReadLock cả bảng ⇒ **báo động giả** giữa hai khu không liên quan; bảng lớn có index thì khoá mịn hơn. Số lần thử lại ở K038 (seq vs index) là bằng chứng định lượng — trích kèm khoảng min–max.
- **K040-a (không khoá) với "rào" cho ra vi phạm ở gần như MỌI chuyến** — vì cả 8 phiên đọc `requested` trước khi ai kịp commit, và UPDATE không điều kiện của người sau CHỜ rồi vẫn ghi đè. Ngoài đời tỉ lệ thấp hơn nhiều (bấm không đồng thời đến vậy) — đó chính là lý do lỗi này lọt qua test thủ công. Đừng trích tỉ lệ như con số production.
- **K040-e**: đúng code SAI của (a) nhưng chạy REPEATABLE READ + thử lại ⇒ không vi phạm, trả giá bằng ~(số phiên − 1) lần `40001` mỗi chuyến. "Mức cô lập cao + retry" là một cách sửa hợp lệ khi không sửa được code đọc–ghi.
- **SKIP LOCKED không công bằng và không nhất quán**: đếm việc bằng SKIP LOCKED sai; việc bị worker treo giữ sẽ bị bỏ đói; `rong_gia` trong K036 đếm số lần worker thấy "hàng rỗng" trong khi vẫn còn việc. NOWAIT + thử lại tốn nhiều lỗi `55P03` (xem DO-DAC) — đừng dùng làm hàng đợi.
- **Idempotency**: request lặp ĐỒNG THỜI phải CHỜ (không lỗi) ở index duy nhất — nhờ `ON CONFLICT DO NOTHING` chờ giao dịch đang chèn cùng khoá. Request đầu chết ⇒ request lặp tự làm (không cần dọn khoá mồ côi) vì khoá được chèn TRONG CÙNG giao dịch với chuyến. Thiết kế "chèn khoá, commit, rồi xử lý" (hai giao dịch) cần thêm trạng thái "đang xử lý" + thời hạn — lab không làm.
- **Outbox chỉ bảo đảm ít nhất một lần**: lab cho thấy trùng xảy ra khi publisher chết giữa "gửi" và "đánh dấu"; số tin trùng ở K039 phụ thuộc tỉ lệ chết đặt trong đặc tả (10 %), không phải đặc tính của PostgreSQL.
- **Thời gian/thông lượng ở DO-DAC.md** (việc/giây, giây mỗi lần) đo trên MỘT máy, ống psql ↔ Node ~0,3 ms/lượt; chỉ so tương đối giữa các cách trong cùng bảng.
- Hàm `dat_chuyen()` có HAI bản: trong `39-idempotency-outbox.sql` §2 và trong `dong-thoi.mjs` (`HAM_039`) — sửa một bản thì sửa bản kia.
