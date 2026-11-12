# Lab PostgreSQL · Tới Nơi

Mã thực hành cho loạt bài **PostgreSQL qua dự án Tới Nơi**. Đọc cùng [khung chương trình](../../docs/postgresql-master-curriculum.md) và [hồ sơ dự án mẫu Tới Nơi](../../api/seed/content/du-an-mau-toi-noi.md) — lược đồ, tiền tệ (đồng, số nguyên) và múi giờ (Asia/Ho_Chi_Minh) của lab bám theo hồ sơ đó. Mọi tên người, số điện thoại, biển số trong lab đều **hư cấu**.

| Phần | Chủ đề | Thư mục | Trạng thái |
|---|---|---|---|
| P01 | Nền tảng và mô hình dữ liệu | [`p01/`](p01/README.md) | Có SQL, kết quả mẫu và test — đã chạy trên PGlite 17.5 |
| P02 | SQL cho nghiệp vụ | [`p02/`](p02/README.md) | Có SQL, fixture 30 ngày tất định, kết quả mẫu và test — đã chạy trên PGlite 17.5 (chạy trên lược đồ P01) |
| P03 | Lưu trữ, index và kế hoạch thực thi | [`p03/`](p03/README.md) | Có SQL, fixture 1 triệu chuyến tất định, kết quả mẫu, thí nghiệm, phép đo và test — **đã chạy trên PostgreSQL 18.1 thật** (cluster tạm, xem [P03+](#p03-postgresql-18-that)) |
| P04 | Giao dịch, MVCC và đồng thời | [`p04/`](p04/README.md) | Có SQL (10 kịch bản phần lớn NHIỀU PHIÊN, lịch chạy tất định), fixture nhỏ, kết quả mẫu, thí nghiệm, **phép thử đồng thời** (8 phiên chạy đua, kiểm bất biến) và test — **đã chạy trên PostgreSQL 18.1 thật** |
| P05 | Bảo mật, bảo trì và phục hồi | `p05/` | Chưa có |
| P06 | Mở rộng, tích hợp và vận hành đầu cuối | `p06/` | Chưa có |

Mỗi phần là một thư mục gồm các file `NN-*.sql` chạy theo thứ tự tên, một `README.md`, kết quả chạy thật `OUTPUT.txt` và một file test trong `test/`. Phần sau có thể chạy trên lược đồ của phần trước: P02 = `p01/00-reset.sql` + `p01/01-schema.sql` + `p02/` (không nạp fixture P01); P03 = lược đồ P01 + `p02/00-mo-rong-schema.sql` + `p03/00`–`03` (dữ liệu riêng của P03); P04 = lược đồ P01 + `p02/00-mo-rong-schema.sql` + `p04/00`–`02` (fixture nhỏ riêng của P04).

**Hai engine, hai runner.** P01–P02 chạy bằng **PGlite 0.3.16 = PostgreSQL 17.5** trong tiến trình Node (`run.mjs`, một kết nối, không cần cài gì). Từ **P03 trở đi** lab chạy trên **PostgreSQL 18 thật** (đã kiểm 18.1) qua `pg/run-pg.mjs`: runner tự `initdb` một cluster TẠM, chạy SQL bằng `psql`, rồi dừng và xoá cluster. Lý do: P03+ cần thứ PGlite không có — extension contrib (pageinspect, pg_buffercache, pg_trgm…), nhiều phiên đồng thời, file tạm, WAL, khởi động lại server, pgbench.

## Phiên bản cố định

| Thành phần | Phiên bản | Ghi chú |
|---|---|---|
| Node.js | ≥ 22 (đã kiểm 22.18.0) | `node --test` với mẫu glob cần Node 22 |
| `@electric-sql/pglite` | **0.3.16** (ghim chính xác trong `package.json` + `package-lock.json`) | Chạy **PostgreSQL 17.5** biên dịch sang WebAssembly |
| Ảnh Docker (cách B) | `postgres:17.5-bookworm` | Cùng bản vá 17.5; **chưa chạy thử** trên máy viết bài |
| Extension (P01–P02) | Không dùng | `gen_random_uuid()` có sẵn trong lõi từ PostgreSQL 13 |
| PostgreSQL (P03+) | **18.1** (initdb, pg_ctl, psql, pgbench của bản cài Windows `C:\Program Files\PostgreSQL\18`) | Bắt buộc ≥ 18 (runner kiểm). Extension contrib dùng: pageinspect, pg_buffercache, pg_visibility, pgstattuple, pg_walinspect, pg_trgm, btree_gist, unaccent, pg_stat_statements — đều có trong bản cài chuẩn. Không cần PostGIS, pgvector. P04 dùng thêm `transaction_timeout` (có từ PostgreSQL 17) |

## Cách A · Node + PGlite (đã kiểm chứng)

Không cần cài PostgreSQL hay Docker. PGlite là PostgreSQL thật (mã nguồn PostgreSQL 17.5) chạy trong tiến trình Node, mỗi lần chạy là một database trống trong bộ nhớ.

```sh
cd examples/toi-noi-postgresql
npm ci                 # hoặc npm install — cài đúng PGlite 0.3.16 theo package-lock.json
npm run p01            # chạy p01/00 → 07 trên một database mới, in kết quả kiểu psql
npm run p02            # p01/00 + p01/01 (lược đồ) rồi p02/00 → 20
npm test               # chạy bộ kiểm tra (node:test)
```

Kết quả mong đợi của `npm run p01` kết thúc bằng:

```text
-- [lab] Câu lệnh đã chạy: 176
-- [lab] Lỗi mong đợi (expect-error): 38
-- [lab] Kiểm số dòng (expect-rows): 56
-- [lab] FAIL: 0
-- [lab] KẾT QUẢ: ĐẠT
```

`npm run p02` kết thúc bằng:

```text
-- [lab] Câu lệnh đã chạy: 266
-- [lab] Lỗi mong đợi (expect-error): 7
-- [lab] Kiểm số dòng (expect-rows): 37
-- [lab] Kiểm giá trị (expect-value): 13
-- [lab] Kiểm nội dung (expect-line): 109
-- [lab] Kiểm md5 kết quả (expect-md5): 3
-- [lab] FAIL: 0
-- [lab] KẾT QUẢ: ĐẠT
```

và phần P01–P02 của `npm test` gồm 58 test (P01: 6 test luồng chính + 20 thí nghiệm của bài trong `p01/thu-nghiem`; P02: 6 test trong `test/p02.test.mjs` + 26 thí nghiệm của bài trong `p02/thu-nghiem`). Toàn bộ kết quả thật nằm ở [`p01/OUTPUT.txt`](p01/OUTPUT.txt) và [`p02/OUTPUT.txt`](p02/OUTPUT.txt).

Từ P03, `test/p03.test.mjs` thêm 6 test runner (luôn chạy) và một nhóm 26 test trên PostgreSQL thật (nền, 10 tệp bài, 12 thí nghiệm của bài, so OUTPUT.txt, kiểm đã dọn cluster). Kết quả đã chạy: **máy có PostgreSQL 18.1** ⇒ `# tests 90`, `# pass 90`, `# fail 0` (~3,6 phút, phần lớn là dựng nền P03); **giả lập máy chỉ có Node** (`PG_BIN=<thư mục không tồn tại> npm test`) ⇒ `# tests 64`, `# pass 64`, nhóm P03 hiện `# SKIP bỏ qua — PG_BIN=… không chứa psql/initdb/pg_ctl`.

Từ P04, `test/p04.test.mjs` thêm 5 test luôn chạy (chỉ thị mới, che PID/xid, bộ thử lại `giaoDich()`, mã thoát 3 của hai CLI khi không có PostgreSQL) và hai nhóm trên PostgreSQL thật: 26 test cho tệp bài/thí nghiệm (nền, 10 tệp bài, 12 thí nghiệm của bài, so OUTPUT.txt, dọn cluster) và 16 test cho **phép thử đồng thời cỡ nhỏ** (`--nho`: 14 kịch bản, chỉ kiểm bất biến "phải đúng", dọn cluster). Kết quả đã chạy (11/11/2026): **máy có PostgreSQL 18.1** ⇒ `# tests 137`, `# pass 137`, `# fail 0` (~3,2 phút — các tệp test chạy song song); **giả lập máy chỉ có Node** (`PG_BIN=<thư mục không tồn tại> npm test`) ⇒ `# tests 69`, `# pass 69`, ba nhóm P03/P04 hiện `# SKIP bỏ qua — PG_BIN=… không chứa psql/initdb/pg_ctl`.

Các cách chạy khác của runner:

```sh
node run.mjs p01/00-reset.sql p01/01-schema.sql p01/02-fixture-valid.sql   # chọn file
node run.mjs p01 --no-echo                    # chỉ in kết quả, không in lại câu lệnh
node run.mjs p01 --persist .pgdata            # lưu database vào thư mục .pgdata
node run.mjs p01/05-select.sql --persist .pgdata   # chạy tiếp trên dữ liệu đã lưu
node run.mjs p01 p01/06-migration-rollback.sql     # thêm file hoàn tác migration
npm run p01:output                            # ghi lại p01/OUTPUT.txt sau khi sửa SQL
npm run p02:output                            # ghi lại p02/OUTPUT.txt sau khi sửa SQL
```

Chạy cả thư mục thì runner lấy các file `NN-*.sql` theo thứ tự tên và **bỏ qua `*-rollback.sql`**.

### Đọc kết quả của runner

- Câu lệnh được in lại (kèm chú thích phía trên nó), rồi tới kết quả: bảng căn cột với `(n rows)`, thẻ lệnh như `INSERT 0 3`, `UPDATE 1`, `DELETE 2`, `CREATE TABLE`, `SELECT 3174` (của `CREATE TABLE … AS`), thông báo `NOTICE:` / `DEBUG:`, và lỗi `ERROR:` + `DETAIL:` + `HINT:` giống psql.
- Dòng bắt đầu bằng `-- [lab]` là chú thích của runner (psql thật không in): SQLSTATE, tên ràng buộc, bảng, cột của lỗi; kết quả `PASS`/`FAIL` của các kỳ vọng; tổng kết.
- Kỳ vọng được ghi bằng dòng chú thích **ngay trước** câu lệnh:
  - `-- expect-error: 23505 uq_customers_phone` — câu lệnh phải lỗi đúng SQLSTATE và đúng tên ràng buộc. Lỗi NOT NULL (`23502`) ở PostgreSQL 17 không có tên ràng buộc, nên ghi tên cột: `-- expect-error: 23502 full_name`. Lỗi không gắn ràng buộc/cột (sai định dạng `22P02`…) chỉ ghi mã.
  - `-- expect-rows: 0` — câu lệnh phải trả đúng số dòng (SELECT, RETURNING) hoặc đổi đúng số dòng (UPDATE/DELETE/INSERT, `CREATE TABLE … AS` theo thẻ lệnh).
  - `-- expect-value: 182287000` — kết quả phải đúng 1 dòng × 1 cột với giá trị (dạng chữ) đó.
  - `-- expect-line: 100724 | completed | 76000` — lặp mỗi dòng kết quả một chỉ thị; TOÀN BỘ kết quả phải khớp đúng thứ tự và đúng số dòng. Ô cách nhau bằng `|`, khoảng trắng hai đầu mỗi ô được bỏ qua, NULL viết `NULL` (như psql sau `\pset null NULL`).
  - `-- expect-md5: <32 ký tự hex>` — md5 của toàn bộ kết quả ở dạng chuẩn (ô nối `" | "`, dòng nối bằng xuống dòng, không tiêu đề); dùng cho kết quả dài. Sai thì runner in md5 thật.
  - Ba chỉ thị sau có từ P02; tổng kết chỉ in dòng đếm của chúng khi có dùng, nên kết quả P01 giữ nguyên từng byte.
- Lỗi khớp kỳ vọng là `PASS`, runner chạy tiếp. Lỗi ngoài kỳ vọng, câu lẽ ra phải lỗi mà chạy được, hoặc sai số dòng là `FAIL`; runner vẫn chạy hết rồi trả **exit code 1**.
- Trước file đầu tiên, runner đặt cho phiên `SET TIME ZONE 'Asia/Ho_Chi_Minh'` (giờ hiển thị khớp hồ sơ Tới Nơi) và `SET client_min_messages = notice` (mức mặc định của server thật; PGlite mặc định `debug3` nên sẽ in cả thông báo DEBUG nội bộ). Mỗi file SQL cũng tự đặt `search_path` và `TimeZone` ở đầu để chạy riêng lẻ bằng psql vẫn ra cùng kết quả.

## Cách B · PostgreSQL 17 thật qua Docker + psql

> **Chưa chạy trên máy viết bài** (máy không có Docker, psql hay PostgreSQL server). Các file SQL **chỉ được kiểm bằng PGlite 17.5**. Lệnh dưới đây theo tài liệu chính thức của ảnh `postgres` và của psql; nếu bạn chạy được, hãy so kết quả với `p01/OUTPUT.txt` và ghi lại khác biệt.

```sh
cd examples/toi-noi-postgresql
docker compose up -d
docker compose ps                    # chờ cột STATUS báo (healthy)

# Mở psql trong container (không cần cài psql trên máy):
docker compose exec db psql -X -U toinoi -d toinoi
```

Trong psql, chạy từng file bằng `\i` (một phiên duy nhất, giống runner):

```text
\i /lab/p01/00-reset.sql
\i /lab/p01/01-schema.sql
\i /lab/p01/02-fixture-valid.sql
\i /lab/p01/03-fixture-invalid.sql
\i /lab/p01/04-lifecycle.sql
\i /lab/p01/05-select.sql
\i /lab/p01/06-migration.sql
\i /lab/p01/07-checks.sql
```

P02 dùng lược đồ P01 nhưng dữ liệu riêng — trong một phiên mới (hoặc sau `\i /lab/p01/00-reset.sql`):

```text
\i /lab/p01/00-reset.sql
\i /lab/p01/01-schema.sql
\i /lab/p02/00-mo-rong-schema.sql
\i /lab/p02/01-fixture.sql
\i /lab/p02/02-kiem-tra-fixture.sql
\i /lab/p02/11-null-case-coalesce.sql
-- … 12 → 18, rồi 19-cay-khu-vuc.sql, 19-chu-trinh.sql, 20-doi-soat-thanh-toan.sql
```

Hoặc chạy không tương tác, mỗi lệnh `psql` là một phiên riêng:

```sh
# File không chứa lỗi cố ý: dừng ngay nếu có lỗi.
docker compose exec db psql -X -a -U toinoi -d toinoi -v ON_ERROR_STOP=1 \
  -f /lab/p01/00-reset.sql -f /lab/p01/01-schema.sql -f /lab/p01/02-fixture-valid.sql

# 03, 04, 06 CỐ Ý chứa câu lỗi: KHÔNG bật ON_ERROR_STOP, tự đối chiếu lỗi với
# dòng `-- expect-error:` phía trên mỗi câu (psql coi đó là chú thích thường).
docker compose exec db psql -X -a -U toinoi -d toinoi -f /lab/p01/03-fixture-invalid.sql
docker compose exec db psql -X -a -U toinoi -d toinoi -f /lab/p01/04-lifecycle.sql
docker compose exec db psql -X -a -U toinoi -d toinoi -v ON_ERROR_STOP=1 -f /lab/p01/05-select.sql
docker compose exec db psql -X -a -U toinoi -d toinoi -f /lab/p01/06-migration.sql
docker compose exec db psql -X -a -U toinoi -d toinoi -v ON_ERROR_STOP=1 -f /lab/p01/07-checks.sql
```

`-X` bỏ qua file cấu hình `~/.psqlrc`, `-a` in lại mọi dòng đầu vào (để so với `OUTPUT.txt`). Đã cài psql trên máy thì kết nối từ ngoài bằng `psql -h 127.0.0.1 -p 5433 -U toinoi -d toinoi` (mật khẩu giữ chỗ trong `docker-compose.yml`; đổi nếu muốn, nhưng đừng dùng mật khẩu thật). Khác biệt hình thức cần biết khi so với `OUTPUT.txt`: psql không in dòng `-- [lab]`, và khi đọc lệnh từ file (`-f` hoặc `\i`) psql thêm tiền tố `psql:/lab/p01/<file>:<dòng>:` trước mỗi `ERROR`.

Ở cách B, psql có các lệnh meta như `\d trips`, `\d+ trips` (xem chú thích `COMMENT ON`), `\dn`, `\timing`. Các file SQL **không dùng** lệnh meta để chạy được trên cả runner.

## Khác biệt giữa PGlite và server thật (quan trọng cho P01)

Đã kiểm trên PGlite 0.3.16 trừ khi ghi "chưa kiểm". Phía server thật là những gì tài liệu/ảnh Docker cho biết — **chưa đối chiếu trên máy viết bài**.

| Khía cạnh | PGlite 0.3.16 (đã đo) | Server `postgres:17.5-bookworm` | Ảnh hưởng tới P01 |
|---|---|---|---|
| Kết nối | **Một** kết nối duy nhất; câu lệnh chạy tuần tự | Nhiều phiên đồng thời | Không mô phỏng được hai tài xế bấm nhận cùng lúc, khoá chờ nhau, `lock_timeout` thực sự có tác dụng. Các ý đó để P04 trên server thật |
| Lệnh meta psql (`\d`, `\i`, `\timing`, `\gset`) | Không có — đó là lệnh của công cụ psql, không phải SQL; runner báo lỗi nếu gặp | Có khi dùng psql | Lab dùng `information_schema` / `pg_constraint` thay cho `\d` |
| Extension | `pg_available_extensions` chỉ có `plpgsql`; `CREATE EXTENSION pgcrypto` → `0A000 extension "pgcrypto" is not available` (contrib phải nạp qua tuỳ chọn JS của PGlite) | Có sẵn bộ contrib trong ảnh | P01 không dùng extension |
| `TimeZone` mặc định | `GMT` | Thường `Etc/UTC` với ảnh Docker (chưa kiểm — chạy `SHOW timezone`) | Runner và mọi file SQL đặt `Asia/Ho_Chi_Minh` tường minh |
| `client_min_messages` mặc định | `debug3` (in cả DEBUG nội bộ, vd. "drop auto-cascades to …") | `notice` | Runner đặt `notice`; 06 bật `debug1` có chủ đích cho một câu |
| Thông báo lạ khi ROLLBACK | Có `WARNING:  # 1743: aborting transaction N` — **riêng PGlite** | Không có | Runner ẩn dòng này |
| Collation database | `datcollate = C`, `datctype = C.UTF-8`, không có collation ICU | `en_US.utf8` (biến `LANG` của ảnh; chưa kiểm) | `ORDER BY full_name` có thể khác thứ tự: dưới `C`, "Đặng" đứng sau "Võ". 05 ghi `COLLATE "C"` để kết quả giống nhau |
| `DateStyle` | `ISO, MDY` | Dự kiến `ISO, MDY` theo locale en_US (chưa kiểm) | `'03/11/2026'::date` = 11/03/2026 — lab chỉ dùng ISO 8601 |
| Tên database / người dùng | `template1` / `postgres` | `toinoi` / `toinoi` | Lab chỉ dựa vào schema `toinoi`, không phụ thuộc tên database |
| `version()` | Ghi `PostgreSQL 17.5 on aarch64-unknown-linux-gnu, compiled by emcc …, 32-bit` dù chạy trên Windows/x64 | Chuỗi của bản build Linux thật | Runner chỉ in `server_version` = 17.5 |
| Bền vững dữ liệu | Mặc định trong bộ nhớ, mất khi tiến trình kết thúc; `--persist <thư mục>` để lưu | Volume Docker | Mỗi `npm run p01` bắt đầu từ database trống |
| Thông báo lỗi | Cùng mã nguồn ⇒ cùng nội dung, SQLSTATE, `DETAIL`, `HINT` (`lc_messages` rỗng ⇒ tiếng Anh) | Tiếng Anh với locale en_US | Bài viết có thể trích nguyên văn từ `OUTPUT.txt` |

P02 thêm hai điểm cần biết: (1) `01-fixture.sql` chạy `ANALYZE` sau khi nạp — trên PGlite, thiếu thống kê làm cả bộ P02 chậm từ ~3 giây lên ~30 giây (kết quả không đổi); server thật có autovacuum nhưng vẫn nên ANALYZE sau khi nạp lô lớn. (2) P02 dùng `MERGE … RETURNING merge_action()` và `MERGE` trong `WITH` — cần PostgreSQL **17** trở lên.

Thứ tự đọc các dòng trùng giá trị sắp xếp (05, mục 6) phụ thuộc vị trí vật lý của dòng và thuật toán sắp xếp; cùng mã nguồn và cùng chuỗi câu lệnh thì kết quả dự kiến giống nhau, nhưng **chưa đối chiếu trên server thật** — bài viết nên trình bày đó là ví dụ minh hoạ, không phải hành vi được bảo đảm.

<a id="p03-postgresql-18-that"></a>

## P03+ · PostgreSQL 18 thật (`pg/run-pg.mjs`)

Hạ tầng dùng chung cho P03 → P06. **Không thêm dependency npm**: runner chỉ dùng `child_process` gọi `initdb`, `pg_ctl`, `psql`, `pgbench` của bản PostgreSQL đã cài. Đã chạy thật trên Windows 11 + PostgreSQL 18.1; trên Linux/macOS **chưa chạy thử** (mã đã tính tới: chọn `LC_CTYPE=C.UTF-8`, bỏ đo bộ nhớ Windows).

### Yêu cầu

- Node ≥ 22. PostgreSQL **≥ 18** đã cài (không cần chạy service nào; runner tự dựng cluster riêng).
- Runner tìm binary theo thứ tự: biến môi trường `PG_BIN` › `pg_config --bindir` (nếu `pg_config` nằm trong `PATH`) › `C:\Program Files\PostgreSQL\18\bin`. Đặt `PG_BIN` mà thư mục không có `psql`/`initdb`/`pg_ctl` ⇒ coi như **không có PostgreSQL** (không dò tiếp): runner in lý do và thoát **mã 3**; test P03 bị **skip** (không FAIL) — `PG_BIN=/khong-co npm test` giả lập máy chỉ có Node.
- Chỗ trống ở thư mục tạm: ~2 GB khi chạy P03 (nền ~500 MB + bản sao cho từng bài, mỗi bản xoá ngay khi bài xong).

### Chạy

```sh
npm run p03                    # = node pg/run-pg.mjs p03 — nền + 10 tệp bài, ~2–3 phút, in kết quả như p03/OUTPUT.txt
npm run p03:output             # ghi lại p03/OUTPUT.txt
npm run p03:thu-nghiem         # các thí nghiệm của bài (p03/thu-nghiem/manifest.json)
npm run p03:measure            # PHÉP ĐO (không thuộc npm test, ~4 phút) → ghi p03/DO-DAC.md
npm run p04                    # = node pg/run-pg.mjs p04 — nền nhỏ + 10 kịch bản (phần lớn nhiều phiên), ~45 s
npm run p04:output | p04:thu-nghiem
npm run p04:measure            # PHÉP THỬ ĐỒNG THỜI (pg/run-dong-thoi.mjs, không thuộc npm test) → ghi p04/DO-DAC.md
npm run p04:dong-thoi-nho      # cùng phép thử, cỡ nhỏ (~35 s, không ghi tệp) — npm test chạy bản này

node pg/run-pg.mjs p03 --only 23                     # chỉ tệp bài có tên chứa "23" (nền vẫn dựng)
node pg/run-pg.mjs p03/thu-nghiem/manifest.json --only pg024
node pg/run-pg.mjs --nen p03/lab.json:p03 tam.sql    # chạy tệp lẻ (chưa khai manifest) trên bản sao nền p03
node pg/run-pg.mjs a.sql b.sql                       # tệp lẻ trên MỘT database trống, chung phiên (như run.mjs)
node pg/run-pg.mjs p03 p03/thu-nghiem/manifest.json --json kq.json   # nhiều đích một lần: nền chung chỉ dựng một lần
```

Tùy chọn: `--no-echo` (không in lại câu lệnh), `--an-nen` (không in đầu ra tệp nền), `--json <tệp>` (kết quả từng mục), `--keep` (giữ cluster CHẠY sau khi xong để gỡ lỗi — runner in lệnh `psql` để kết nối và lệnh dừng/xoá), `--attach <thư-mục-toinoi-pg-…>` (gỡ lỗi: chạy trên cluster đã `--keep`, dùng lại database nền có sẵn — vòng sửa tệp bài chỉ còn vài giây). Biến môi trường gỡ lỗi: `PG_LAB_TIMING=1` (in thời gian từng bước ra stderr), `PG_LAB_RAW=1` (không che thời gian đo — đừng dùng khi ghi OUTPUT). Mã thoát: 0 ĐẠT · 1 có FAIL · 2 lỗi dùng/runner · 3 không có PostgreSQL.

### Cluster tạm

Mỗi lần chạy: `initdb` vào `os.tmpdir()/toinoi-pg-<ngẫu nhiên>/data` với `-U postgres --auth=trust -E UTF8 --no-sync --locale=C --lc-ctype=<xem dưới> --locale-provider=builtin --builtin-locale=C.UTF-8` (checksum dữ liệu BẬT — mặc định của initdb 18), khởi động bằng `pg_ctl` ở **cổng trống ngẫu nhiên chỉ trên localhost**, xong thì `pg_ctl stop` và xoá cả thư mục — kể cả khi lỗi, Ctrl-C (SIGINT/SIGTERM/SIGBREAK) hay ngoại lệ (dọn đồng bộ trong `process.on('exit')`). Trên Windows tiến trình con của postgres có thể giữ thư mục vài giây sau khi dừng; runner thử xoá lại tới ~60 giây, không được thì in CẢNH BÁO kèm đường dẫn. Runner **không bao giờ** kết nối tới service PostgreSQL sẵn có của máy (vd. `postgresql-x64-18` ở cổng 5432) và gỡ mọi biến `PG*` của người dùng (PGHOST, PGPORT, PGPASSWORD…) khỏi môi trường của tiến trình con.

- **Collation** do provider `builtin` (C.UTF-8) quyết định ⇒ thứ tự sắp chữ giống nhau trên mọi hệ điều hành. **LC_CTYPE** là `en-US` (Windows) / `C.UTF-8` (nơi khác), ghi đè bằng `PG_LAB_LC_CTYPE`. Lý do: pg_trgm (và full-text search) phân loại "ký tự chữ" theo LC_CTYPE của database; với `LC_CTYPE=C`, `show_trgm('Cơm Tấm')` chỉ còn trigram của `c`, `m`, `t` — mọi chữ có dấu bị coi là dấu cách (đã thử: PG-028 sẽ sai hoàn toàn).
- Cấu hình ghi đè, GHI TƯỜNG MINH dù nhiều dòng trùng mặc định (nguồn: `LAB_SETTINGS` trong `pg/lib/cluster.mjs`):

| Tham số | Giá trị | Vì sao |
|---|---|---|
| `listen_addresses` / `port` | `localhost` / ngẫu nhiên | Không mở ra mạng, không đụng cổng 5432 |
| `shared_buffers` | 128MB (mặc định) | Bảng `trips` P03 (152 MB) LỚN HƠN shared_buffers — có hiện tượng vòng đệm Seq Scan, đẩy page ra |
| `work_mem` / `hash_mem_multiplier` | 4MB / 2 (mặc định) | PG-029 đo tràn đĩa so với mức mặc định |
| `maintenance_work_mem` | 64MB (mặc định) | Thời gian CREATE INDEX như cấu hình mặc định (fixture tự `SET` 512MB trong phiên nạp) |
| `effective_cache_size`, `random_page_cost`, `seq_page_cost`, `effective_io_concurrency` | 4GB, 4, 1, 16 (mặc định 18) | Cost model mặc định — kế hoạch giống máy người đọc chưa tinh chỉnh |
| `jit` | off | Bỏ thời gian biên dịch JIT dao động; kế hoạch không có khối "JIT:" |
| `max_parallel_workers_per_gather`, `max_parallel_maintenance_workers` | 0 | Không song song ⇒ kế hoạch và số buffer tất định, dễ đọc (bài nào cần thì `SET` lại) |
| `default_statistics_target` | 100 (mặc định) | ANALYZE lấy mẫu 30.000 dòng — PG-027 dựa vào đúng giới hạn này |
| `autovacuum` | **off** | Tái lập: không có VACUUM/ANALYZE chen vào giữa chừng làm đổi visibility map, thống kê, số buffer. Tệp lab tự chạy VACUUM/ANALYZE khi cần — trên server thật autovacuum làm việc đó |
| `track_io_timing` | off (mặc định) | Không có dòng "I/O Timings" dao động |
| `max_wal_size` / `checkpoint_timeout` | 4GB / 30min | Không có checkpoint tự động chen giữa phép đo WAL (PG-022, PG-025, PG-030 tự `CHECKPOINT`) |
| `log_checkpoints` | off | Nhật ký server gọn (mặc định của 18 là on) |
| `log_line_prefix` / `log_temp_files` | `'%m [%p] '` / -1 | Bài tự `SET log_temp_files = 0` khi cần (PG-029) |
| `shared_preload_libraries` | `pg_stat_statements` | Sẵn cho bài cần thống kê truy vấn (nạp lúc khởi động) |
| `timezone` / `log_timezone` | `Asia/Ho_Chi_Minh` | Giờ VN như hồ sơ Tới Nơi — kể cả khi người đọc tự mở psql vào cluster (`--keep`) |
| `datestyle`, `lc_messages` | `iso, mdy`, `C` | Định dạng ngày ISO, thông báo lỗi tiếng Anh như OUTPUT.txt |
| `max_connections` | 40 | Đủ cho kịch bản nhiều phiên + pgbench 8 phiên |

### Mỗi bài một database chép từ nền

Thư mục phần có `lab.json` (vd. `p03/lab.json`): `"nen"` khai các NỀN (danh sách tệp, đường dẫn tính từ thư mục lab), `"tep"` khai tệp bài (đường dẫn tính từ thư mục chứa manifest) và nền của nó. Runner dựng mỗi nền MỘT lần vào database `nen_<khoá>`, rồi với mỗi tệp: `CREATE DATABASE bai_NN TEMPLATE nen_<khoá> STRATEGY = FILE_COPY` → chạy tệp trong phiên mới → `DROP DATABASE`. Hệ quả: các bài độc lập (bài này tạo/xoá index không ảnh hưởng bài kia), mỗi bài bắt đầu với **shared_buffers không có page nào của database đó** (cache "lạnh" với PostgreSQL, nhưng page cache của hệ điều hành vẫn nóng), và thống kê ANALYZE giống nhau giữa các bài trong cùng một lần chạy. Manifest thí nghiệm (`p0N/thu-nghiem/manifest.json`) dùng cùng định dạng (`"tep"` = mảng `{ "file", "bai", "nen" }`); hai đích có cùng danh sách tệp nền thì dùng chung một nền.

### Đọc kết quả

Gần giống P01–P02 nhưng là **đầu ra thật của psql** (`-X`, `ON_ERROR_STOP=0`, không `-q` nên có thẻ lệnh `INSERT 0 3`, `CREATE INDEX`…): bảng aligned của psql (khoảng trắng cuối dòng bị cắt), `\pset null NULL` (NULL hiện chữ `NULL`), NOTICE/ERROR ở dạng ngắn kèm dòng `-- [lab] SQLSTATE … · constraint … · table … · column …` (runner bật `VERBOSITY verbose` để đọc tên ràng buộc rồi in lại gọn). Lệnh meta của psql DÙNG ĐƯỢC (`\gset`, `\timing`, `\d+`, `\x`…) — khác `run.mjs`. Chuẩn hoá khi in: `actual time=T..T`, `Planning/Execution Time: T ms`, `Time: T ms`, `Query Identifier: Q`, tên file tạm `pgsql_tmpPID.N`, đường dẫn cluster `<TMP>`, cổng `<PORT>`. Dòng `-- [lab] >>> kết quả biến động…` / `<<< hết phần biến động` bao phần kết quả THAY ĐỔI giữa các lần chạy (thống kê ANALYZE lấy mẫu): test so khớp OUTPUT.txt bỏ qua phần đó; khi so, test còn che `cost=… rows=… width=…` (ước lượng), nội dung dòng `Buffers:` (cách chia hit/read/written lệch nhẹ giữa các lần), `buffers full=` của WAL và số kB ở `Memory Usage/Disk Usage/Memory/Disk` — các con số quan trọng trong đó đã có chỉ thị `expect-match` kiểm riêng. Đã kiểm: hai lần chạy độc lập cho đầu ra giống hệt nhau sau chuẩn hoá này.

### Chỉ thị (dòng chú thích NGAY TRƯỚC bước)

| Chỉ thị | Ý nghĩa |
|---|---|
| `-- expect-error: SQLSTATE [tên]` | Bước PHẢI lỗi đúng mã (và đúng tên ràng buộc, hoặc tên cột với 23502) — như `run.mjs` |
| `-- expect-rows: n` | Biến `ROW_COUNT` của psql sau câu = n (số dòng trả về hoặc bị đổi) |
| `-- expect-value: v` · `-- expect-line: a \| b` · `-- expect-md5: hex` | Như `run.mjs` (đọc từ bảng aligned của psql; ô chứa chính ký tự `\|` thì dùng md5 hoặc đổi cột) |
| `-- expect-match: regex` (lặp được) | MỖI regex khớp ít nhất một dòng đầu ra (dùng cho kế hoạch: `Index Scan using … on trips`, `Heap Fetches: 0$`, `Sort Method: external merge`) |
| `-- expect-no-match: regex` | Không dòng nào khớp |
| `-- expect-range: lo hi` | Kết quả là 1 ô số trong [lo, hi] — cho giá trị phụ thuộc lấy mẫu; đầu ra bước tự được đánh dấu biến động |
| `-- expect-blocked` | (nhiều phiên) bước PHẢI rơi vào trạng thái chờ khoá (kèm `lab: cho-xong`: phải đã từng chờ khoá trước khi xong) |
| `-- expect-mat-ket-noi` | Bước làm phiên MẤT KẾT NỐI (psql thoát giữa bước: `pg_terminate_backend`, `idle_in_transaction_session_timeout`, `transaction_timeout`…). Runner in một dòng chuẩn hoá `✖ MẤT KẾT NỐI`; bước sau của phiên đó mở kết nối MỚI (in `mở kết nối MỚI`) — nhớ `SET` lại. Không có chỉ thị này mà mất kết nối ⇒ FAIL |
| `-- lab: bien-dong` | Đánh dấu đầu ra bước là biến động (test không so) |
| `-- lab: nhat-ky` | Sau bước, in các dòng MỚI của nhật ký server (bỏ tiền tố giờ/PID) — vd. `LOG:  temporary file: …` |
| `-- lab: restart` | `pg_ctl restart` TRƯỚC bước này (shared_buffers trống; mọi phiên mở lại — đặt lại `search_path` sau đó) |
| `-- lab: cho-ms n` | (nhiều phiên) chờ tối đa n ms (mặc định 3000) trước khi coi bước "vẫn đang chạy" |
| `-- lab: cho-xong` | (nhiều phiên) CHỜ bước chạy xong (tối đa 60 s) dù nó đang chờ khoá — cho bước tự kết thúc bằng lỗi sau một khoảng (`lock_timeout` 55P03, `statement_timeout` 57014, deadlock 40P01, `transaction_timeout`). Đã thấy nó chờ khoá thì in `⏳ đã chờ khoá (…) — bị chặn bởi [X] trước khi xong` |
| `-- @TEN` | Chạy bước trên phiên TEN (mỗi phiên = một tiến trình psql = một kết nối, `application_name = lab-TEN`) |

### Nhiều phiên

Tệp có bất kỳ `-- @TEN` nào là kịch bản nhiều phiên; bước không ghi phiên chạy trên phiên của bước trước. Runner gửi từng bước **theo thứ tự trong tệp**. Sau khi gửi, runner chờ bước xong; đồng thời một phiên quan sát ẩn hỏi `pg_stat_activity` mỗi 40 ms — nếu backend của bước đang `wait_event_type = 'Lock'` (3 lần liên tiếp) thì bước được ghi là **đang chờ**, kèm phiên đang chặn (`pg_blocking_pids` → nhãn phiên), và runner chạy tiếp bước sau. Sau mỗi bước xong, runner cho các bước đang chờ tối đa 2 giây để chạy nốt; bước nào xong thì in kết quả **ngay tại đó** ("✔ bước dòng N đã chạy xong (sau khi hết bị chặn)"). Nhờ vậy lịch chạy in ra tất định. Hết tệp mà còn bước chờ ⇒ chờ thêm 10 giây rồi FAIL. Ví dụ (trích `p03/thu-nghiem/pg024-cluster-hang-doi-khoa.sql`):

```text
-- [lab] [B]
-- expect-blocked
CLUSTER bai_cu.trips USING trips_pkey;
-- [lab] [B] ⏳ đang chờ khoá (Lock: relation) — bị chặn bởi [A]
-- [lab] PASS: bước bị chặn như mong đợi (expect-blocked)
…
-- [lab] [A]
COMMIT;
COMMIT

-- [lab] [B] ✔ bước dòng 15 đã chạy xong (sau khi hết bị chặn):
CLUSTER
```

Phát hiện "bị chặn" theo `wait_event_type = 'Lock'` (khoá dòng/bảng/tuple/transactionid/**advisory**); chờ kiểu khác (IO, LWLock, `pg_sleep`) được báo là "vẫn đang chạy" sau `cho-ms`. SERIALIZABLE không chờ mà báo lỗi `40001` (thường ở COMMIT) — chấm bằng `expect-error` ở bước đó. Bổ sung từ P04 (tương thích ngược — đầu ra P03 không đổi):

- **Lỗi của phiên bị chặn** chấm ở chính bước đó: `expect-blocked` + `expect-error: 40001` (hoặc 23505, 40P01…) — kết quả/lỗi in ở dòng `✔` khi nó chạy xong.
- **Bước tự hết giờ** (`lock_timeout`, `statement_timeout`, deadlock): dùng `-- lab: cho-xong` để runner đợi bước xong ngay tại chỗ (lịch chạy không đổi theo tốc độ máy). Deadlock tất định: đặt `SET deadlock_timeout` khác nhau cho hai phiên (vd. 10s và 200ms — phiên ngắn hơn chạy bộ dò, thấy vòng và bị huỷ).
- **Mất kết nối**: `-- expect-mat-ket-noi` (xem bảng chỉ thị). Hiện tượng: psql chạy không tương tác tự thoát mã 2 khi kết nối mất; thông điệp libpq không tất định (có lúc kèm `FATAL 57P01`, có lúc không) nên runner in dạng chuẩn hoá.
- **Che phần không tất định trong tệp nhiều phiên**: trong thông báo lỗi và nhật ký server, PID backend → nhãn phiên (`Process [B] waits for ShareLock on transaction <XID>; blocked by process [A].`, `Process holding the lock: [A]`), xid → `<XID>`, `after 201.681 ms` → `after T ms`; mỗi dòng nhật ký server (`lab: nhat-ky`) mang nhãn phiên đã ghi ra nó (`[B] ERROR:  deadlock detected`). Tệp một phiên (P03) không đổi.
- **xid tương đối và tên phiên trong SQL**: các tệp P04 dùng hàm `xid_td()`, `snap_td()`, `lab_khoa()`, `lab_cho()` của `p04/00-mo-rong-schema.sql` để in `X+1`, nhãn `A`/`B` thay số tuyệt đối (runner đặt `application_name = lab-<NHÃN>`).

### Phép thử đồng thời (`pg/run-dong-thoi.mjs`, từ P04)

Lịch chạy tất định chứng minh CƠ CHẾ; còn "đúng một người thắng" cần bằng chứng khi các phiên chạy ĐUA thật. `node pg/run-dong-thoi.mjs <đặc-tả.mjs>` (vd. `p04/dong-thoi.mjs`) dựng cluster tạm + nền như `run-pg.mjs`, rồi với mỗi kịch bản × mỗi lần: database mới chép từ nền → `chuan_bi` (SQL) → mở N phiên psql (`pg/lib/dong-thoi.mjs` · `Worker`: chế độ gọn QUIET/unaligned, `q()`/`qx()`/`v()`, `crash()` = giết tiến trình psql để giả lập ứng dụng chết, `reopen()`) → hàm `chay()` của đặc tả điều khiển các phiên chạy song song (`Promise.all`) → kiểm **bất biến** (SQL, hoặc từ bộ đếm phía client `tu_dem`). `giaoDich(w, fn, { muc, toi_da, thu_lai_khi })` chạy `fn` trong một giao dịch và **thử lại TOÀN BỘ** khi `40001`/`40P01` với backoff "full jitter", đếm `loi_<SQLSTATE>`, `thu_lai`, `het_luot_thu`. Bất biến `phai_dung: true` vi phạm ⇒ mã thoát 1; bất biến có `mong_doi` mà không `phai_dung` là MINH HOẠ cách làm sai (chỉ đếm). Tùy chọn: `--nho` (cỡ nhỏ, không ghi tệp), `--only <chuỗi>`, `--lap n`, `--ra <tệp.md>`, `--json <tệp>`. Số đếm DAO ĐỘNG giữa các lần ⇒ test chỉ kiểm bất biến; số đếm ghi vào `p0N/DO-DAC.md` kèm cấu hình máy và min–max.

### Đo (`pg/run-measure.mjs`)

`npm run p03:measure` đọc đặc tả `p03/do-dac.mjs` (các NHÓM đo, mỗi nhóm một database chép từ nền: `chuan_bi` → mỗi phép đo `thiet_lap` → lặp [`truoc_moi_lan` → đo → `sau_moi_lan`] × (1 khởi động + 7 lần, trừ khi ghi khác) → `go_bo`), in trung vị/min/max và ghi `p03/DO-DAC.md` kèm cấu hình máy (CPU, RAM, hệ điều hành, ổ đĩa), phiên bản PostgreSQL, cấu hình cluster, cách đo và giới hạn. Hai cách đo: `explain` (Execution Time của `EXPLAIN (ANALYZE, TIMING OFF, BUFFERS, FORMAT JSON)`) và `client` (`\timing` của psql, gồm commit). Nhóm `pgbench` chạy pgbench với script tuỳ chỉnh ở nhiều mức phiên × `PGOPTIONS`, ghi tps, độ trễ, số file tạm và đỉnh bộ nhớ riêng của các backend (Windows: PowerShell `Get-Process`; nơi khác ghi "chưa đo"). Tùy chọn: `--only <chuỗi>`, `--lap n`, `--ra <tệp.md>`.

## Dọn môi trường

- **Cách A:** không có gì để dọn khi chạy trong bộ nhớ. Nếu đã dùng `--persist .pgdata` thì xoá thư mục đó (`rm -rf .pgdata`). Gỡ hẳn: xoá `node_modules/`.
- **Cách B:** chỉ xoá dữ liệu lab mà giữ container: trong psql chạy `\i /lab/p01/00-reset.sql` (hoặc `DROP SCHEMA toinoi CASCADE;`). Dừng và xoá toàn bộ, kể cả volume: `docker compose down -v`. Xoá ảnh đã tải: `docker image rm postgres:17.5-bookworm`.
- **P03+:** runner tự dừng và xoá cluster tạm. Nếu tiến trình bị giết thô (kill -9, tắt máy) hoặc đã dùng `--keep`: dừng bằng `pg_ctl stop -D <thư-mục>/data -m fast` rồi xoá thư mục `toinoi-pg-*` trong thư mục tạm của hệ điều hành (`%TEMP%` trên Windows). Kiểm không còn tiến trình: `tasklist /FI "IMAGENAME eq postgres.exe"` chỉ còn các tiến trình của service (Session Name `Services`).

## Cấu trúc

```text
toi-noi-postgresql/
├── README.md              ← tệp này
├── package.json           ← ghim @electric-sql/pglite 0.3.16; scripts p01, p02, p03, p04 (+ :output), p03/p04:thu-nghiem, p03/p04:measure, p04:dong-thoi-nho, test
├── package-lock.json
├── run.mjs                ← runner P01–P02: tách câu lệnh, chạy trên PGlite, in kiểu psql, chấm expect-*
├── pg/                    ← runner P03+ trên PostgreSQL thật (không dependency npm)
│   ├── run-pg.mjs         ← CLI: cluster tạm, nền + bản sao từng bài, manifest, --json/--keep/--attach
│   ├── run-measure.mjs    ← CLI đo: đặc tả p0N/do-dac.mjs → p0N/DO-DAC.md
│   ├── run-dong-thoi.mjs  ← CLI phép thử đồng thời (P04+): đặc tả p0N/dong-thoi.mjs → p0N/DO-DAC.md
│   └── lib/               ← bin.mjs (tìm binary) · cluster.mjs (initdb/pg_ctl/dọn) · session.mjs (phiên psql)
│                            sqlsplit.mjs (tách bước + chỉ thị) · output.mjs (đọc/chuẩn hoá đầu ra) ·
│                            runner.mjs (chạy bước, nhiều phiên, chấm) · measure.mjs · machine.mjs ·
│                            dong-thoi.mjs (Worker, giaoDich có thử lại, bộ đếm)
├── docker-compose.yml     ← PostgreSQL 17.5 thật cho cách B (chưa chạy thử)
├── test/
│   ├── p01.test.mjs       ← node:test cho P01 và cho chính runner
│   ├── thu-nghiem.test.mjs← thí nghiệm của bài P01
│   ├── p02.test.mjs       ← node:test cho P02 và các chỉ thị expect-value/line/md5
│   ├── p02-thu-nghiem.test.mjs ← thí nghiệm của bài P02
│   ├── p03.test.mjs       ← mảnh runner P03+ (luôn chạy) + P03 trên PostgreSQL thật (skip khi không có)
│   └── p04.test.mjs       ← mảnh runner P04 (luôn chạy) + P04 + phép thử đồng thời nhỏ (skip khi không có)
├── p01/                   ← Nền tảng và mô hình dữ liệu (PG-001 … PG-010)
│   ├── README.md
│   ├── 00-reset.sql … 07-checks.sql, 06-migration-rollback.sql, thu-nghiem/
│   └── OUTPUT.txt         ← kết quả thật của `npm run p01`
├── p02/                   ← SQL cho nghiệp vụ (PG-011 … PG-020), chạy trên lược đồ p01/00 + 01
│   ├── README.md
│   ├── 00-mo-rong-schema.sql, 01-fixture.sql, 02-kiem-tra-fixture.sql
│   ├── 11-…sql … 20-…sql  ← mỗi file một bài, số file = số bài (19 có hai file)
│   ├── thu-nghiem/        ← thí nghiệm và lời giải bài tập của PG-011 … PG-020 (manifest.json)
│   └── OUTPUT.txt         ← kết quả thật của `npm run p02`
└── p03/                   ← Lưu trữ, index và kế hoạch thực thi (PG-021 … PG-030), PostgreSQL 18 thật
    ├── README.md, lab.json (nền + tệp bài)
    ├── 00-mo-rong-schema.sql, 01-fixture.sql, 02-kiem-tra-fixture.sql, 03-bai-cu-20k.sql   ← nền
    ├── 21-…sql … 30-…sql  ← mỗi file một bài
    ├── thu-nghiem/        ← thí nghiệm (manifest.json), gồm kịch bản nhiều phiên
    ├── do-dac.mjs → DO-DAC.md  ← đặc tả phép đo → kết quả đo thật kèm cấu hình máy
    └── OUTPUT.txt         ← kết quả thật của `npm run p03`
└── p04/                   ← Giao dịch, MVCC và đồng thời (PG-031 … PG-040), PostgreSQL 18 thật
    ├── README.md, lab.json (nền + tệp bài)
    ├── 00-mo-rong-schema.sql, 01-fixture.sql, 02-kiem-tra-fixture.sql   ← nền nhỏ (~1 s)
    ├── 31-…sql … 40-…sql  ← mỗi file một bài, phần lớn kịch bản nhiều phiên
    ├── thu-nghiem/        ← thí nghiệm (manifest.json)
    ├── dong-thoi.mjs → DO-DAC.md  ← đặc tả phép thử đồng thời → số đếm thật + bất biến, kèm cấu hình máy
    └── OUTPUT.txt         ← kết quả thật của `npm run p04`
```


## P05 · Bảo mật, bảo trì và phục hồi

Đã bổ sung PG-041–050. Chạy `npm run p05` trên PostgreSQL thật; cần thêm OpenSSL. Xem [hướng dẫn và giới hạn phép đo](p05/README.md), [runbook](p05/RUNBOOK.md) và [kết quả thực](p05/OUTPUT.txt). Lab dựng hai cluster tạm, kiểm TLS/SCRAM, quyền, lock, vacuum, backup logic, pool, RLS và PITR. Không chạy seed/deploy.
