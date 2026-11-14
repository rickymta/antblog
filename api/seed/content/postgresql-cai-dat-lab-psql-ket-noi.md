---
slug: postgresql-cai-dat-lab-psql-ket-noi
title: "Dựng môi trường lab và kết nối bằng psql"
summary: "Muốn thử luật dữ liệu chuyến xe của Tới Nơi mà không đụng database thật? Dựng lab PostgreSQL 17.5 riêng bằng PGlite hoặc Docker, chạy truy vấn đầu tiên, ghi lại phiên bản và dọn sạch sau khi dùng."
category: postgresql
level: beginner
tags: [postgresql, pg-002, lab, psql, pglite, docker, toi-noi]
language: vi
---

Ngày đầu vào nhóm backend của Tới Nơi, bạn được giao kiểm tra một luật: "cước chuyến không được âm". Cách nhanh nhất có vẻ là mở công cụ quản trị, kết nối vào database đang chạy và thử chèn một chuyến cước −1.000 đ. Đó cũng là cách nhanh nhất để bỏ lại một dòng rác trong dữ liệu tiền của công ty, hoặc tệ hơn, xoá nhầm thứ không được xoá.

Người làm database cẩn thận luôn có một **lab**: môi trường riêng, dựng lại được bằng vài lệnh, biết chính xác phiên bản, dọn sạch khi xong. Bài này dựng lab cho Phần 1 theo hai cách, chạy truy vấn đầu tiên và ghi "phiếu lab" để mọi kết quả về sau có ngữ cảnh.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-002 · Phần 1/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/postgresql-la-gi-va-vai-tro-trong-toi-noi">PG-001 · PostgreSQL là gì và nằm ở đâu trong Tới Nơi?</a>. Lab: <code>examples/toi-noi-postgresql/p01</code>. Sau bài này, bạn tạo được lab riêng, chạy truy vấn đầu tiên, ghi lại phiên bản và biết cách dọn môi trường.</p>
</div>

## 1. Lab cần đạt bốn điều

| Yêu cầu | Lab của series đáp ứng thế nào |
|---|---|
| **Riêng** | dữ liệu hư cấu, nằm trong schema `toinoi`, không đụng database nào khác |
| **Tái lập được** | `00-reset.sql` xoá và dựng lại schema; fixture ghi sẵn UUID và thời điểm nên mỗi lần chạy ra cùng kết quả |
| **Phiên bản cố định** | PostgreSQL **17.5** ở cả hai cách chạy |
| **Dọn được** | một lệnh xoá schema, một lệnh xoá container và dữ liệu |

Hai cách chạy, khác nhau ở mức độ kiểm chứng:

| | Cách A · Node + PGlite | Cách B · Docker + psql |
|---|---|---|
| Cần cài | Node.js ≥ 22 | Docker |
| PostgreSQL | 17.5 biên dịch sang WebAssembly, chạy trong tiến trình Node | server 17.5 thật, ảnh `postgres:17.5-bookworm` |
| Kết nối | một kết nối duy nhất | nhiều phiên như server thật |
| Trạng thái kiểm chứng | **đã chạy thật** khi viết bài | **chưa chạy** trên máy viết bài |

Mọi kết quả trong Phần 1 chép từ cách A; chạy cách B thì so với `p01/OUTPUT.txt` và ghi lại chỗ khác biệt.

## 2. Cách A: Node + PGlite

**PGlite** là bản PostgreSQL chạy được trong Node, Bun, Deno hoặc trình duyệt; theo tài liệu của nó, mỗi instance chỉ có một kết nối độc quyền tới database. Lab ghim chính xác `@electric-sql/pglite` **0.3.16**, tương ứng PostgreSQL **17.5**. Không cần cài PostgreSQL hay Docker.

```bash
cd examples/toi-noi-postgresql
node --version     # cần >= 22; máy viết bài: v22.18.0
npm ci             # cài đúng PGlite 0.3.16 theo package-lock.json
npm run p01        # chạy p01/00 → 07 trên một database mới trong bộ nhớ
npm test           # bộ kiểm tra node:test
```

Kết quả thật của `npm run p01` kết thúc bằng:

```text
-- [lab] ===== Tổng kết =====
-- [lab] Câu lệnh đã chạy: 176
-- [lab] Lỗi mong đợi (expect-error): 38
-- [lab] Kiểm số dòng (expect-rows): 56
-- [lab] FAIL: 0
-- [lab] KẾT QUẢ: ĐẠT
```

và `npm test` báo `# pass 6`, `# fail 0`. "Lỗi mong đợi" là các câu **cố ý sai** trong `03-fixture-invalid.sql`, viết để chứng minh database từ chối dữ liệu xấu; PG-005 sẽ đi qua từng câu.

### Đọc kết quả của runner

`run.mjs` tách file SQL thành từng câu, gửi cho PGlite và in kết quả kiểu psql: bảng với `(n rows)`, thẻ lệnh như `INSERT 0 3`, `NOTICE:`, `ERROR:`/`DETAIL:`. Dòng bắt đầu bằng `-- [lab]` là chú thích của runner (psql thật không in), ví dụ mã lỗi SQLSTATE và `PASS`/`FAIL`.

Trước file đầu tiên, runner đặt cho phiên `TimeZone = Asia/Ho_Chi_Minh` (giờ hiển thị khớp hồ sơ Tới Nơi) và `client_min_messages = notice`. Mỗi file SQL của lab cũng tự đặt `search_path` và `TimeZone` ở đầu, nên chạy riêng từng file bằng psql vẫn ra cùng kết quả.

Chạy một vài file thay vì cả thư mục:

```bash
node run.mjs p01/00-reset.sql p01/01-schema.sql p01/02-fixture-valid.sql
node run.mjs p01 --no-echo        # chỉ in kết quả, không in lại câu lệnh
```

Thứ tự quan trọng: `05-select.sql` cần dữ liệu của `00`→`02`, nên luôn chạy kèm các file đó phía trước trong cùng một lệnh. Mỗi lệnh `node run.mjs` là một database **mới tinh** trong bộ nhớ, trừ khi bạn dùng `--persist` (mục 5).

## 3. Truy vấn đầu tiên

Tạo một file `thu-nghiem.sql` ở thư mục của riêng bạn (đừng sửa file trong `p01/`, bộ test đối chiếu `OUTPUT.txt` từng byte):

```sql
SELECT version();
SHOW server_version;
SHOW server_version_num;
SHOW timezone;
SELECT current_database(), current_user;
SELECT 1 + 1 AS ket_qua;
```

Chạy `node run.mjs <đường-dẫn>/thu-nghiem.sql`. Kết quả thật (đã lược phần in lại câu lệnh):

```text
-- [lab] Engine: PostgreSQL 17.5 qua PGlite 0.3.16 (WebAssembly, một kết nối)
                                       version
-------------------------------------------------------------------------------------…
 PostgreSQL 17.5 on aarch64-unknown-linux-gnu, compiled by emcc (Emscripten gcc/clang-like replacement + linker emulating GNU ld) 3.1.74 (1092ec30a3fb1d46b1782ff1b4db5094d3d06ae5), 32-bit
(1 row)
 server_version
----------------
 17.5
(1 row)
 server_version_num
--------------------
 170005
(1 row)
     TimeZone
------------------
 Asia/Ho_Chi_Minh
(1 row)
 current_database | current_user
------------------+--------------
 template1        | postgres
(1 row)
 ket_qua
---------
       2
(1 row)
```

Đọc từng dòng:

- **`version()`** trả về chuỗi mô tả phiên bản server. Chuỗi của PGlite nói "aarch64-unknown-linux-gnu … emcc … 32-bit" dù máy viết bài chạy Windows x64: đó là thông tin bản build WebAssembly, không phải máy của bạn. Server thật in chuỗi của bản build Linux.
- **`server_version`** là con số dễ đọc; **`server_version_num`** là dạng số để chương trình so sánh (170005 = 17.5). Tài liệu PostgreSQL khuyên chương trình dùng `server_version_num` thay vì tự tách chuỗi `version()`.
- **`SHOW`** cho thấy giá trị hiện tại của một tham số cấu hình (*run-time parameter*); `SHOW timezone` ra `Asia/Ho_Chi_Minh` vì runner đã đặt.
- **`current_database()`** là `template1` và **`current_user`** là `postgres` — đặc thù PGlite. Với Docker của lab, cả hai sẽ là `toinoi`. Lab chỉ dựa vào schema `toinoi`, không phụ thuộc tên database hay tên user.

### Múi giờ mặc định không phải múi giờ của bạn

Thêm ba câu sau vào file và chạy lại:

```sql
RESET TIME ZONE;
SHOW timezone;
SET TIME ZONE 'Asia/Ho_Chi_Minh';
```

```text
RESET
 TimeZone
----------
 GMT
(1 row)
SET
```

Khi bỏ thiết lập của runner, PGlite quay về `GMT`. Với ảnh Docker, giá trị mặc định thường là `Etc/UTC` (chưa kiểm — hãy chạy `SHOW timezone` và ghi vào phiếu lab). Vì mặc định khác nhau giữa các môi trường, lab **luôn đặt múi giờ tường minh**. Đây là chuyện của cách *hiển thị* thời điểm; PG-004 giải thích vì sao giá trị đã lưu không đổi khi đổi múi giờ phiên.

## 4. Ghi phiếu lab

Kết quả không kèm ngữ cảnh rất khó so sánh về sau, nên ghi lại môi trường trước mọi thí nghiệm. Phiếu của máy viết bài, lấy từ lần chạy thật:

| Mục | Giá trị | Lấy bằng |
|---|---|---|
| Node.js | v22.18.0 | `node --version` |
| PGlite | 0.3.16 | dòng `-- [lab] Engine` |
| `server_version` / `server_version_num` | 17.5 / 170005 | `SHOW …` |
| `TimeZone` mặc định → của lab | GMT → Asia/Ho_Chi_Minh | `RESET TIME ZONE; SHOW timezone;` |
| `DateStyle` | ISO, MDY | `SHOW DateStyle;` |
| `server_encoding` | UTF8 | `SHOW server_encoding;` |
| Collation database | `datcollate` = C, `datctype` = C.UTF-8 | câu SQL dưới |

```sql
SELECT datcollate, datctype FROM pg_database WHERE datname = current_database();
```

```text
 datcollate | datctype
------------+----------
 C          | C.UTF-8
(1 row)
```

**Collation** (quy tắc so sánh và sắp xếp chuỗi) ảnh hưởng tới thứ tự `ORDER BY` tên khách; server Docker có thể khác PGlite, vì vậy các truy vấn sắp xếp chữ của lab ghi rõ collation (PG-007). `DateStyle` = `ISO, MDY` là lý do chuỗi `'03/11/2026'` bị hiểu thành ngày 11 tháng 3 — bẫy này để dành cho PG-004.

## 5. Giữ dữ liệu giữa các lần chạy

Mặc định PGlite giữ database trong bộ nhớ và mất khi tiến trình kết thúc. Thêm `--persist <thư-mục>` để lưu xuống đĩa rồi chạy tiếp ở lệnh sau:

```bash
node run.mjs p01/00-reset.sql p01/01-schema.sql p01/02-fixture-valid.sql --persist .pgdata
node run.mjs dem-dong.sql --persist .pgdata
```

Với `dem-dong.sql` chứa câu đếm số dòng từng bảng, lần chạy thứ hai (một tiến trình hoàn toàn mới) vẫn thấy dữ liệu:

```sql
SELECT
  (SELECT count(*) FROM toinoi.cities)    AS cities,
  (SELECT count(*) FROM toinoi.customers) AS customers,
  (SELECT count(*) FROM toinoi.drivers)   AS drivers,
  (SELECT count(*) FROM toinoi.merchants) AS merchants,
  (SELECT count(*) FROM toinoi.trips)     AS trips;
```

```text
 cities | customers | drivers | merchants | trips
--------+-----------+---------+-----------+-------
      3 |         6 |       6 |         3 |    15
(1 row)
```

Mở thư mục `.pgdata` ra, bạn sẽ thấy đúng hình hài **thư mục dữ liệu** (*data directory*) mà PG-001 nhắc tới: file `PG_VERSION` chứa `17`, các thư mục `base`, `global`, `pg_wal`, `pg_xact`… và hai file cấu hình `postgresql.conf`, `pg_hba.conf`. Trên máy viết bài, thư mục này có dung lượng khoảng 27 MB. Các thư mục con được giải thích ở Phần 3 và 5; lúc này nhớ: **cụm database trên đĩa là một thư mục**, và đừng sửa tay file trong đó.

## 6. Cách B: Docker và psql

> **Chưa chạy trên máy viết bài** (máy không có Docker hay psql). Lệnh dưới đây lấy theo README của lab, tài liệu ảnh `postgres` và tài liệu psql. Mọi khối "kết quả" trong mục này là **ví dụ minh hoạ**, không phải kết quả đã chạy.

**psql** là client dòng lệnh chính thức của PostgreSQL: gõ câu SQL tương tác, chạy SQL từ file, và có thêm các **lệnh meta** bắt đầu bằng dấu `\` để xem cấu trúc, đổi định dạng hiển thị. File `docker-compose.yml` của lab dựng `postgres:17.5-bookworm` với user và database cùng tên `toinoi`, chỉ mở cổng trên `127.0.0.1:5433`, và gắn thư mục `p01` vào container ở `/lab/p01` (chỉ đọc).

```bash
cd examples/toi-noi-postgresql
docker compose up -d
docker compose ps                                   # chờ STATUS báo (healthy)
docker compose exec db psql -X -U toinoi -d toinoi  # psql bên trong container
```

`-X` bỏ qua file `~/.psqlrc` để mọi người có cùng cách hiển thị; `-U` là user, `-d` là database. Trong psql, chạy các file lab bằng `\i` trong **một** phiên, giống runner:

```text
\i /lab/p01/00-reset.sql
\i /lab/p01/01-schema.sql
\i /lab/p01/02-fixture-valid.sql
```

Sau đó dùng các truy vấn ở mục 3 và các lệnh meta chỉ psql mới có:

| Lệnh | Tác dụng |
|---|---|
| `\conninfo` | in thông tin kết nối hiện tại: database, user, socket hoặc host, cổng |
| `\dn` | liệt kê schema |
| `\dt toinoi.*` | liệt kê bảng trong schema `toinoi` |
| `\d trips` | cấu trúc bảng `trips` (cột, kiểu, ràng buộc, index) |
| `\q` | thoát psql |

Dưới đây là ví dụ minh hoạ (chưa chạy) cho `\conninfo` qua socket trong container; còn với `\dt toinoi.*` bạn sẽ thấy sáu bảng như PG-001 đã liệt kê:

```text
You are connected to database "toinoi" as user "toinoi" via socket in "/var/run/postgresql" at port "5432".
```

Lệnh meta là của **công cụ psql**, không phải SQL: server không hiểu chúng, và runner PGlite từ chối cả file nếu gặp. Thử đặt `\dt` vào một file rồi chạy bằng runner, kết quả thật:

```text
-- [lab] FAIL: Dòng 5: lệnh meta của psql (\…) không chạy được trong runner — chỉ dùng SQL thuần
```

Vì thế các file lab dùng `information_schema` thay cho `\d` (như PG-001 đã làm). Muốn chạy không tương tác, mỗi lệnh `psql -f` là một phiên riêng:

```bash
docker compose exec db psql -X -a -U toinoi -d toinoi -v ON_ERROR_STOP=1 \
  -f /lab/p01/00-reset.sql -f /lab/p01/01-schema.sql -f /lab/p01/02-fixture-valid.sql
```

`-a` in lại mọi dòng đọc được (để so với `OUTPUT.txt`), `-v ON_ERROR_STOP=1` dừng ngay khi có lỗi. **Đừng** bật `ON_ERROR_STOP` cho `03`, `04`, `06`: chúng cố ý chứa câu lỗi. Khi đọc từ file, psql thêm tiền tố `psql:/lab/p01/<file>:<dòng>:` trước mỗi `ERROR`. Có psql trên máy thì kết nối từ ngoài bằng `psql -h 127.0.0.1 -p 5433 -U toinoi -d toinoi` (mật khẩu mẫu trong `docker-compose.yml`, đừng dùng mật khẩu thật).

## 7. Dọn môi trường

Dọn dẹp là một phần của thí nghiệm, không phải việc làm thêm. Thử thật trên thư mục `--persist` ở mục 5:

```sql
DROP SCHEMA toinoi CASCADE;
SELECT count(*) AS so_schema_toinoi FROM information_schema.schemata WHERE schema_name = 'toinoi';
```

```text
NOTICE:  drop cascades to 6 other objects
DETAIL:  drop cascades to table toinoi.cities
drop cascades to table toinoi.customers
drop cascades to table toinoi.drivers
drop cascades to table toinoi.merchants
drop cascades to table toinoi.trips
drop cascades to table toinoi.trip_events
DROP SCHEMA
 so_schema_toinoi
------------------
                0
(1 row)
```

Đếm lại ở lần chạy sau thì nhận được `ERROR:  relation "toinoi.cities" does not exist`, vì schema đã biến mất cùng sáu bảng. `NOTICE` liệt kê đúng những gì bị xoá; hãy đọc nó mỗi lần dùng `CASCADE`.

| Muốn | Cách A (PGlite) | Cách B (Docker) |
|---|---|---|
| Về trạng thái trống, giữ môi trường | chạy lại `00-reset.sql` | `\i /lab/p01/00-reset.sql` hoặc `DROP SCHEMA toinoi CASCADE;` |
| Xoá dữ liệu đã lưu | xoá thư mục `--persist` (`rm -rf .pgdata`) | `docker compose down -v` (`-v` xoá cả volume) |
| Gỡ hẳn | xoá `node_modules/` | thêm `docker image rm postgres:17.5-bookworm` |

Chạy trong bộ nhớ (không `--persist`) thì không có gì để dọn: tiến trình kết thúc là database biến mất.

## 8. Khác biệt cần nhớ giữa PGlite và server thật

| Khía cạnh | PGlite 0.3.16 (đã đo) | Server Docker 17.5 |
|---|---|---|
| `TimeZone` mặc định | GMT | thường `Etc/UTC` (chưa kiểm) |
| Collation | C / C.UTF-8 | theo locale của ảnh (chưa kiểm) |
| Extension | chỉ `plpgsql` | bộ contrib có sẵn |

Phần 1 chỉ dùng SQL thuần và không cần extension, nên kết quả hai cách dự kiến giống nhau. Những bài cần hai phiên chạy song song (Phần 4) bắt buộc dùng server thật.

## 9. Lỗi thường gặp

- **Node quá cũ.** `npm test` dùng mẫu glob của `node --test`, cần Node 22.
- **Chạy lẻ một file cần dữ liệu.** `node run.mjs p01/05-select.sql` một mình sẽ báo không tìm thấy bảng: database mới tinh, chưa có schema. Chạy kèm `00`→`02`.
- **Quên `-v` khi `docker compose down`.** Container biến mất nhưng volume dữ liệu còn; lần `up` sau vẫn thấy dữ liệu cũ.
- **Cổng 5433 bị chiếm.** Đổi cổng phía máy trong `docker-compose.yml`.
- **Chạy lab vào database dùng chung.** `00-reset.sql` có `DROP SCHEMA … CASCADE`; chỉ chạy trên lab.

## 10. Bài tập

**Bài 1.** Điền phiếu lab của máy bạn theo bảng mục 4 (cách A hoặc B). Ghi riêng mục nào khác máy viết bài.

<details><summary><b>Lời giải</b></summary><p>Với cách A trên Node 22 và PGlite 0.3.16, các giá trị PostgreSQL phải trùng bảng mục 4; chỉ <code>node --version</code> có thể khác. Với cách B, kỳ vọng <code>server_version</code> 17.5, <code>current_database()</code> và <code>current_user</code> là <code>toinoi</code>; <code>TimeZone</code> mặc định và collation phải tự đo, đừng chép từ bài.</p></details>

**Bài 2.** Dùng `--persist` để: (1) chạy `00`→`02`; (2) ở lệnh thứ hai, đếm số chuyến bằng `SELECT count(*) FROM toinoi.trips;`; (3) ở lệnh thứ ba, dọn schema; (4) đếm lại. Dự đoán kết quả từng bước trước khi chạy.

<details><summary><b>Lời giải</b></summary><p>(1) Kết thúc với <code>INSERT 0 15</code> và bảng 8 trạng thái. (2) <code>15</code>: dữ liệu sống qua tiến trình mới nhờ thư mục lưu dữ liệu. (3) <code>NOTICE:  drop cascades to 6 other objects</code> rồi <code>DROP SCHEMA</code>. (4) <code>ERROR:  relation "toinoi.trips" does not exist</code>, SQLSTATE 42P01. Cuối cùng xoá thư mục <code>--persist</code>.</p></details>

**Bài 3.** Một đồng nghiệp gửi kết quả truy vấn có giờ lệch 7 tiếng so với của bạn trên cùng fixture. Bạn hỏi gì đầu tiên, và vì sao lab không phụ thuộc vào câu trả lời?

<details><summary><b>Lời giải</b></summary><p>Hỏi <code>SHOW timezone</code> trong phiên của họ: GMT/UTC và Asia/Ho_Chi_Minh lệch 7 giờ khi hiển thị. Lab không phụ thuộc vì runner và mọi file SQL đều đặt <code>SET TIME ZONE 'Asia/Ho_Chi_Minh'</code> tường minh; nếu họ gõ truy vấn tay trong một phiên psql mới mà chưa chạy file nào của lab, phiên đó vẫn ở múi giờ mặc định.</p></details>

## 11. Đọc tiếp

- [psql — PostgreSQL 17](https://www.postgresql.org/docs/17/app-psql.html): tuỳ chọn dòng lệnh và lệnh meta.
- [SHOW](https://www.postgresql.org/docs/17/sql-show.html) và [hàm thông tin hệ thống](https://www.postgresql.org/docs/17/functions-info.html): `version()`, `current_database()`, `current_user`.
- [Creating a Database Cluster](https://www.postgresql.org/docs/17/creating-cluster.html): thư mục dữ liệu.
- [Ảnh Docker `postgres`](https://hub.docker.com/_/postgres) và [tài liệu PGlite](https://pglite.dev/docs/).
- README lab: `examples/toi-noi-postgresql/README.md`.

Bài tiếp theo: [PG-003 · Bảng, dòng, cột và khoá trong nghiệp vụ chuyến xe](/articles/postgresql-bang-dong-cot-primary-foreign-key).
