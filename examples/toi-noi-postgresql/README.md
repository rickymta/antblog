# Lab PostgreSQL · Tới Nơi

Mã thực hành cho loạt bài **PostgreSQL qua dự án Tới Nơi**. Đọc cùng [khung chương trình](../../docs/postgresql-master-curriculum.md) và [hồ sơ dự án mẫu Tới Nơi](../../api/seed/content/du-an-mau-toi-noi.md) — lược đồ, tiền tệ (đồng, số nguyên) và múi giờ (Asia/Ho_Chi_Minh) của lab bám theo hồ sơ đó. Mọi tên người, số điện thoại, biển số trong lab đều **hư cấu**.

| Phần | Chủ đề | Thư mục | Trạng thái |
|---|---|---|---|
| P01 | Nền tảng và mô hình dữ liệu | [`p01/`](p01/README.md) | Có SQL, kết quả mẫu và test — đã chạy trên PGlite 17.5 |
| P02 | SQL cho nghiệp vụ | `p02/` | Chưa có |
| P03 | Lưu trữ, index và kế hoạch thực thi | `p03/` | Chưa có |
| P04 | Giao dịch, MVCC và đồng thời | `p04/` | Chưa có |
| P05 | Bảo mật, bảo trì và phục hồi | `p05/` | Chưa có |
| P06 | Mở rộng, tích hợp và vận hành đầu cuối | `p06/` | Chưa có |

Mỗi phần là một thư mục gồm các file `NN-*.sql` chạy theo thứ tự tên, một `README.md`, kết quả chạy thật `OUTPUT.txt` và một file test trong `test/`.

## Phiên bản cố định

| Thành phần | Phiên bản | Ghi chú |
|---|---|---|
| Node.js | ≥ 22 (đã kiểm 22.18.0) | `node --test` với mẫu glob cần Node 22 |
| `@electric-sql/pglite` | **0.3.16** (ghim chính xác trong `package.json` + `package-lock.json`) | Chạy **PostgreSQL 17.5** biên dịch sang WebAssembly |
| Ảnh Docker (cách B) | `postgres:17.5-bookworm` | Cùng bản vá 17.5; **chưa chạy thử** trên máy viết bài |
| Extension | Không dùng | `gen_random_uuid()` có sẵn trong lõi từ PostgreSQL 13 |

## Cách A · Node + PGlite (đã kiểm chứng)

Không cần cài PostgreSQL hay Docker. PGlite là PostgreSQL thật (mã nguồn PostgreSQL 17.5) chạy trong tiến trình Node, mỗi lần chạy là một database trống trong bộ nhớ.

```sh
cd examples/toi-noi-postgresql
npm ci                 # hoặc npm install — cài đúng PGlite 0.3.16 theo package-lock.json
npm run p01            # chạy p01/00 → 07 trên một database mới, in kết quả kiểu psql
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

và `npm test` báo `# pass 26`, `# fail 0` (6 test luồng chính + 20 thí nghiệm của bài trong `p01/thu-nghiem`). Toàn bộ kết quả thật nằm ở [`p01/OUTPUT.txt`](p01/OUTPUT.txt).

Các cách chạy khác của runner:

```sh
node run.mjs p01/00-reset.sql p01/01-schema.sql p01/02-fixture-valid.sql   # chọn file
node run.mjs p01 --no-echo                    # chỉ in kết quả, không in lại câu lệnh
node run.mjs p01 --persist .pgdata            # lưu database vào thư mục .pgdata
node run.mjs p01/05-select.sql --persist .pgdata   # chạy tiếp trên dữ liệu đã lưu
node run.mjs p01 p01/06-migration-rollback.sql     # thêm file hoàn tác migration
npm run p01:output                            # ghi lại p01/OUTPUT.txt sau khi sửa SQL
```

Chạy cả thư mục thì runner lấy các file `NN-*.sql` theo thứ tự tên và **bỏ qua `*-rollback.sql`**.

### Đọc kết quả của runner

- Câu lệnh được in lại (kèm chú thích phía trên nó), rồi tới kết quả: bảng căn cột với `(n rows)`, thẻ lệnh như `INSERT 0 3`, `UPDATE 1`, `DELETE 2`, `CREATE TABLE`, thông báo `NOTICE:` / `DEBUG:`, và lỗi `ERROR:` + `DETAIL:` + `HINT:` giống psql.
- Dòng bắt đầu bằng `-- [lab]` là chú thích của runner (psql thật không in): SQLSTATE, tên ràng buộc, bảng, cột của lỗi; kết quả `PASS`/`FAIL` của các kỳ vọng; tổng kết.
- Kỳ vọng được ghi bằng dòng chú thích **ngay trước** câu lệnh:
  - `-- expect-error: 23505 uq_customers_phone` — câu lệnh phải lỗi đúng SQLSTATE và đúng tên ràng buộc. Lỗi NOT NULL (`23502`) ở PostgreSQL 17 không có tên ràng buộc, nên ghi tên cột: `-- expect-error: 23502 full_name`. Lỗi không gắn ràng buộc/cột (sai định dạng `22P02`…) chỉ ghi mã.
  - `-- expect-rows: 0` — câu lệnh phải trả đúng số dòng (SELECT, RETURNING) hoặc đổi đúng số dòng (UPDATE/DELETE/INSERT theo thẻ lệnh).
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

Thứ tự đọc các dòng trùng giá trị sắp xếp (05, mục 6) phụ thuộc vị trí vật lý của dòng và thuật toán sắp xếp; cùng mã nguồn và cùng chuỗi câu lệnh thì kết quả dự kiến giống nhau, nhưng **chưa đối chiếu trên server thật** — bài viết nên trình bày đó là ví dụ minh hoạ, không phải hành vi được bảo đảm.

## Dọn môi trường

- **Cách A:** không có gì để dọn khi chạy trong bộ nhớ. Nếu đã dùng `--persist .pgdata` thì xoá thư mục đó (`rm -rf .pgdata`). Gỡ hẳn: xoá `node_modules/`.
- **Cách B:** chỉ xoá dữ liệu lab mà giữ container: trong psql chạy `\i /lab/p01/00-reset.sql` (hoặc `DROP SCHEMA toinoi CASCADE;`). Dừng và xoá toàn bộ, kể cả volume: `docker compose down -v`. Xoá ảnh đã tải: `docker image rm postgres:17.5-bookworm`.

## Cấu trúc

```text
toi-noi-postgresql/
├── README.md              ← tệp này
├── package.json           ← ghim @electric-sql/pglite 0.3.16; scripts p01, p01:output, test
├── package-lock.json
├── run.mjs                ← runner: tách câu lệnh, chạy trên PGlite, in kiểu psql, chấm expect-*
├── docker-compose.yml     ← PostgreSQL 17.5 thật cho cách B (chưa chạy thử)
├── test/
│   └── p01.test.mjs       ← node:test cho P01 và cho chính runner
└── p01/                   ← Nền tảng và mô hình dữ liệu (PG-001 … PG-010)
    ├── README.md
    ├── 00-reset.sql … 07-checks.sql, 06-migration-rollback.sql
    └── OUTPUT.txt         ← kết quả thật của `npm run p01`
```
