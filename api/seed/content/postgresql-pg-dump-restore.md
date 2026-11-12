---
slug: postgresql-pg-dump-restore
title: "Backup logic với pg_dump: restore vào database riêng và đối soát cả dữ liệu lẫn quyền"
summary: "22:40, một script vận hành chạy DELETE không có WHERE trên bảng chuyến. Bản pg_dump đêm qua có cứu được không? Bài đi qua cách pg_dump chụp một snapshot nhất quán, những gì một dump schema không chứa (role, extension), rồi restore vào database riêng và đối soát số dòng, tổng tiền, digest, owner, ACL và một lệnh ghi bị cấm — kể cả khi --no-owner --no-acl làm dữ liệu đúng mà quyền mất."
category: postgresql
level: intermediate
tags: [postgresql, pg-046, backup, restore, pg-dump, pg-restore, pg-dumpall, toi-noi]
language: vi
---

22:40 tối thứ Năm, một script vận hành dọn dữ liệu thử nghiệm chạy nhầm sang database thật của Tới Nơi. Câu lệnh trong script là `DELETE FROM ops.trips` — **không có `WHERE`**. Màn hình điều phối trống trơn, báo cáo doanh thu ngày về 0. Người trực mở thư mục backup và thấy file `ops-0200.dump` của job `pg_dump` chạy lúc 02:00 sáng nay. Câu hỏi đầu tiên ai cũng hỏi: «có backup rồi, restore là xong chứ?»

Chưa chắc. Có một file backup không đồng nghĩa có thể khôi phục. Bản phục hồi phải đủ chuyến, đúng giá tiền, đúng chủ sở hữu (owner) và đúng quyền truy cập. Lần diễn tập trước, nhóm đã restore thành công — exit code 0 — nhưng tài khoản báo cáo không đọc được bảng nào, vì người chạy thêm `--no-owner --no-acl` để «bớt lỗi». Và kể cả restore hoàn hảo, file 02:00 cũng không có những chuyến phát sinh từ 02:00 đến 22:40.

Bài này chạy trọn một vòng **backup logic**: dump, xem bên trong archive, lấy phần cấp cluster, restore vào **database riêng** rồi đối soát. Giữ được chuyến tới 22:39 là việc của PITR ở PG-048.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-046 · Phần 5/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-wal-checkpoint-durability">PG-045 · WAL, checkpoint và độ bền dữ liệu</a> và <a href="/articles/postgresql-role-grant-least-privilege">PG-041 · Role, GRANT và quyền tối thiểu</a>. Lab: <code>examples/toi-noi-postgresql/p05</code>, PostgreSQL 18.1 thật; chạy <code>npm run p05</code> trong <code>examples/toi-noi-postgresql</code>. Runner tự dựng cluster tạm, tự tạo database đích và tự dọn; không trỏ nó vào server có sẵn. Sau bài này, bạn restore được một dump vào database riêng và chứng minh bằng số liệu rằng dữ liệu, owner và quyền đều khớp nguồn.</p>
</div>

## 1. Backup logic là gì

PostgreSQL có hai họ backup rất khác nhau:

- **Backup logic** (logical backup): đọc dữ liệu qua SQL như một client bình thường rồi ghi ra các lệnh/định nghĩa đủ để **dựng lại** đối tượng: `CREATE TABLE`, dữ liệu bảng, `ALTER TABLE … ADD CONSTRAINT`, `GRANT`… Công cụ là `pg_dump` (một database) và `pg_dumpall` (cả cluster). Kết quả không phụ thuộc cách dữ liệu nằm trên đĩa, nên restore được sang máy khác, hệ điều hành khác, thường cả phiên bản PostgreSQL mới hơn.
- **Backup vật lý** (physical backup): chép chính các file trong data directory cộng với WAL. Đó là base backup và PITR ở PG-048. Restore ra một bản sao từng byte của cả cluster, tại một thời điểm có thể chọn.

`pg_dump` có bốn định dạng đầu ra. Bài này dùng **custom format** (`-Fc`): một file archive nén, đọc bằng `pg_restore`, cho phép xem mục lục, chọn từng đối tượng và đổi thứ tự khi restore. Định dạng plain (`-Fp`, mặc định) là một script SQL chạy bằng `psql` — dễ đọc, nhưng muốn restore một phần thì phải tự cắt file. Định dạng directory (`-Fd`) là một thư mục, mỗi bảng một file, và là định dạng duy nhất cho phép **dump song song** bằng `-j`. Tar (`-Ft`) ít khi cần.

## 2. Một bản dump nhất quán đến mức nào

Script dọn dẹp chạy lúc 22:40, nhưng giả sử nó chạy lúc 02:00 — **trong khi** `pg_dump` đang đọc bảng. Bản dump có nửa trước nửa sau không?

Không. Theo tài liệu PostgreSQL, `pg_dump` tạo bản sao **nhất quán** ngay cả khi database đang được dùng đồng thời: nó mở một giao dịch, lấy **một snapshot** (ảnh chụp MVCC — xem <a href="/articles/postgresql-mvcc-snapshot">PG-032</a>) và đọc mọi bảng qua cùng snapshot đó. Mọi thay đổi commit sau khi snapshot được lấy đều không có trong dump. Thời điểm của một bản dump vì vậy là **lúc nó bắt đầu**, không phải lúc file ghi xong.

Ba hệ quả vận hành:

- `pg_dump` không chặn người đọc hay người ghi thông thường. Nó giữ khoá `ACCESS SHARE` trên các bảng đang dump, nên chặn các lệnh cần khoá mạnh nhất như phần lớn `ALTER TABLE` hay `DROP TABLE` cho tới khi dump xong. Một migration chạy giữa chừng job backup đêm có thể phải chờ.
- Snapshot mở lâu giữ lại **xmin horizon**. <a href="/articles/postgresql-vacuum-autovacuum-bloat">PG-044</a> đã cho thấy một snapshot `REPEATABLE READ` đang mở khiến VACUUM báo «500 are dead but not yet removable». Một job dump kéo dài hàng giờ trên database bận cũng giữ tuple chết như vậy trong suốt thời gian chạy.
- Bản dump chỉ nhất quán **bên trong một database**. `pg_dumpall` gọi `pg_dump` cho từng database lần lượt, nên các database khác nhau không cùng một thời điểm.

## 3. Phạm vi: database, schema và phần «cấp cluster»

Trước khi chạy bất cứ lệnh nào, phải trả lời được: bản dump này **bao phủ cái gì**?

**Một cluster** PostgreSQL chứa nhiều database. Một số đối tượng không thuộc database nào mà thuộc cả cluster — gọi là **globals**: role (người dùng/nhóm), membership giữa các role, tablespace, và các thiết lập gắn với role. `pg_dump` chỉ dump **một database**, nên nó **không** chứa role. Nó chỉ chứa *tham chiếu* tới role: «bảng này owner là `tn_owner`», «cấp `SELECT` cho `tn_report`». Restore sang một cluster chưa có các role đó thì các lệnh `ALTER … OWNER TO` và `GRANT` sẽ lỗi.

Lab còn hẹp hơn nữa: `pg_dump -n ops` chỉ lấy **schema `ops`**. Theo tài liệu `pg_dump`, khi dùng `-n`:

- Các schema khác bị bỏ, kể cả `public`.
- `pg_dump` **không cố dump các đối tượng khác mà schema được chọn phụ thuộc vào**. Không có gì bảo đảm bản dump restore được vào một database trống nếu có phụ thuộc ra ngoài schema.
- Extension không được dump. Lab có tạo extension `pgstattuple` ở PG-044 và mục lục bên dưới không có nó.
- Large object không được đưa vào trừ khi yêu cầu thêm (`-b`).

Tức là `-n ops` hợp với bài toán «sao lưu riêng phần dữ liệu vận hành để kiểm tra», **không phải** backup đầy đủ của database Tới Nơi. Ghi phạm vi này vào biên bản; đừng để người đọc biên bản tưởng đó là backup toàn bộ.

## 4. Chạy lab và mở archive

Lab P05 dựng một cluster PostgreSQL 18.1 tạm, nạp `00-schema.sql`: schema `ops` do `tn_owner` sở hữu, bảng `ops.trips` 100 chuyến chia đều cho tenant 1 và 2, mỗi chuyến 30.000 đ, **tổng 3.000.000 đ**. `tn_app` được `SELECT, INSERT, UPDATE`; `tn_report` chỉ `SELECT`; default privileges cho bảng mới cấp `SELECT` cho report (PG-041).

```bash
cd examples/toi-noi-postgresql
npm run p05
```

Lệnh dump trong `run.mjs` (cổng là cổng tạm do runner chọn):

```text
pg_dump -h 127.0.0.1 -p <PORT> -U postgres -d postgres -Fc -n ops -f <ROOT>/ops.dump
```

`-h`/`-p` luôn ghi tường minh để không vô tình dump server mặc định của máy. Kết quả:

```text
MEASURE 046 custom archive size bytes: 54337
```

54.337 byte cho 100 chuyến có vẻ nhiều. Lý do nằm ở mục lục. `pg_restore --list` đọc phần **TOC** (table of contents) của archive mà không cần kết nối database. Khối dưới chép từ `OUTPUT.txt`; runner đã bỏ dòng chú thích đầu file và số thứ tự/OID ở đầu mỗi dòng, để lại: loại mục, schema, tên, owner.

```text
SCHEMA - ops tn_owner
ACL - SCHEMA ops tn_owner
TABLE ops churn postgres
TABLE ops next_report tn_owner
ACL ops TABLE next_report tn_owner
TABLE ops trips tn_owner
ACL ops TABLE trips tn_owner
TABLE DATA ops churn postgres
TABLE DATA ops next_report tn_owner
TABLE DATA ops trips tn_owner
CONSTRAINT ops trips trips_pkey tn_owner
DEFAULT ACL ops DEFAULT PRIVILEGES FOR TABLES tn_owner
```

Đọc từng nhóm:

- **`SCHEMA` và `ACL - SCHEMA ops`**: lệnh tạo schema với owner `tn_owner`, cùng các `GRANT USAGE` cho app và report. Thiếu dòng ACL này thì report có quyền trên bảng mà vẫn không «đi vào» được schema.
- **`TABLE` / `TABLE DATA`**: định nghĩa bảng và dữ liệu là hai mục riêng. `pg_restore` tạo bảng trước, nạp dữ liệu, rồi mới tới **`CONSTRAINT … trips_pkey`** — primary key được dựng sau khi có dữ liệu, rẻ hơn kiểm từng dòng lúc chèn.
- **`ACL ops TABLE trips`**: các `GRANT` trên bảng. Đây là thứ biến «bảng có dữ liệu» thành «bảng dùng được đúng người».
- **`DEFAULT ACL`**: `ALTER DEFAULT PRIVILEGES` của PG-041. Không có nó, bảng mới tạo sau khi restore sẽ không tự cấp `SELECT` cho report.
- **`TABLE ops churn postgres`**: bảng thí nghiệm VACUUM của PG-044 vẫn nằm trong `ops`, owner là `postgres` vì runner tạo nó bằng superuser. Dump theo schema lấy **mọi thứ trong schema**, kể cả rác thí nghiệm, và phần lớn dung lượng archive nhiều khả năng đến từ dữ liệu bảng này chứ không phải 100 chuyến. Bảng tạm, bảng backup tay cũng lặng lẽ đi vào mọi bản dump theo cách này.

Điều quan trọng nhất là thứ **không có**: không một dòng `ROLE` nào. Runner kiểm điều đó:

```text
PASS 046 roles live in globals, not in schema dump: true
```

## 5. Lấy phần cấp cluster bằng pg_dumpall

Role được lấy bằng `pg_dumpall --globals-only`. Lab thêm `--no-role-passwords` để không đưa password hash vào file: lab không cần, và một file backup chứa verifier SCRAM là một file phải bảo vệ như kho mật khẩu.

```text
pg_dumpall -h 127.0.0.1 -p <PORT> -U postgres --globals-only --no-role-passwords
```

Runner lọc output, chỉ giữ các dòng `CREATE ROLE`/`ALTER ROLE` của bốn role `tn_`:

```text
CREATE ROLE tn_app;
ALTER ROLE tn_app WITH NOSUPERUSER INHERIT NOCREATEROLE NOCREATEDB LOGIN NOREPLICATION NOBYPASSRLS;
CREATE ROLE tn_monitor;
ALTER ROLE tn_monitor WITH NOSUPERUSER INHERIT NOCREATEROLE NOCREATEDB NOLOGIN NOREPLICATION NOBYPASSRLS;
CREATE ROLE tn_owner;
ALTER ROLE tn_owner WITH NOSUPERUSER INHERIT NOCREATEROLE NOCREATEDB NOLOGIN NOREPLICATION NOBYPASSRLS;
CREATE ROLE tn_report;
ALTER ROLE tn_report WITH NOSUPERUSER INHERIT NOCREATEROLE NOCREATEDB NOLOGIN NOREPLICATION NOBYPASSRLS;
```

Để ý: `ALTER ROLE tn_app … LOGIN` **không có** `PASSWORD`. Restore file này sang cluster mới thì `tn_app` tồn tại nhưng không đăng nhập được bằng password cũ — đặt lại password (từ kho bí mật, không phải từ file dump) là một bước riêng trong runbook. Membership như `tn_monitor` thuộc `pg_monitor` cũng nằm trong globals, nhưng bộ lọc của runner không in các dòng đó.

Vậy một bộ backup logic tối thiểu của database Tới Nơi gồm **hai** thứ: globals của cluster và dump của từng database. Thiếu globals, archive vẫn «đầy đủ» theo nghĩa file — nhưng không restore đúng được ở nơi khác.

## 6. Restore vào database riêng, không bao giờ đè lên nguồn

22:41, có người đề xuất `pg_restore --clean` thẳng vào database production cho nhanh. Đừng. Lý do:

- Chưa biết bản dump có đúng không. Đè lên nguồn mà sai thì mất luôn trạng thái hiện tại, kể cả dữ liệu sau 02:00 còn cứu được bằng cách khác.
- Trạng thái hiện tại là **bằng chứng** của sự cố. Đè lên là xoá bằng chứng.
- Database riêng cho phép so sánh hai bên bằng truy vấn rồi mới chọn: thay hẳn, hay chỉ chép lại những dòng đã mất.

Lab làm đúng quy trình đó trên cùng cluster:

```text
CREATE DATABASE p05_restored;
pg_restore -h 127.0.0.1 -p <PORT> -U postgres -d p05_restored --exit-on-error <ROOT>/ops.dump
```

`--exit-on-error` quan trọng hơn vẻ ngoài của nó. Mặc định `pg_restore` gặp lỗi ở một lệnh SQL thì **in lỗi rồi chạy tiếp**, và chỉ báo số lỗi bị bỏ qua ở cuối. Một restore «xong» với vài lỗi `GRANT` bị trôi trong log là đúng loại thành công giả mà bài này muốn tránh. Với cờ này, lỗi đầu tiên dừng restore và exit code khác 0. Nếu muốn «hoặc tất cả, hoặc không gì», có thêm `--single-transaction` (không dùng trong lab).

Lab restore vào cùng cluster nên `tn_owner`, `tn_app`, `tn_report` đã có sẵn. Đây là giới hạn được ghi trong README: lab **không** thử restore sang cluster mới với role mapping khác (mục 9).

Exit code 0 chỉ là cổng thứ nhất. Phần còn lại là đối soát.

## 7. Đối soát: COUNT chưa đủ

Hai bảng cùng 100 dòng vẫn có thể khác một giá tiền, khác tenant, khác trạng thái. Truy vấn đối soát của `run.mjs` gộp ba lớp vào một chuỗi:

```sql
SELECT count(*) || '/' || sum(fare) || '/' ||
       md5(string_agg(id || ':' || tenant_id || ':' || fare || ':' || status,
                      ',' ORDER BY id))
FROM ops.trips;
```

- `count(*)`: số dòng.
- `sum(fare)`: tổng nghiệp vụ, thứ kế toán sẽ hỏi.
- `md5(string_agg(… ORDER BY id))`: **digest** — nối mọi cột có nghĩa của mọi dòng theo thứ tự `id` tường minh rồi băm. Lệch một trường ở một dòng là lệch digest.

Runner chạy truy vấn này ở nguồn, rồi ở đích, và so khớp:

```text
PASS 046 restored data digest: 100/3000000/1d3675e3781bc03c33795a02bb3323e7
```

Digest này đủ cho fixture nhỏ với cột `NOT NULL` và ký tự phân cách không xuất hiện trong dữ liệu. Nó **không** phải checksum tổng quát: `NULL` làm cả biểu thức `||` thành `NULL` và dòng đó bị `string_agg` bỏ qua; chuỗi chứa `:` hay `,` có thể tạo hai dữ liệu khác nhau cùng một chuỗi nối. Với bảng lớn, cần chia theo khoảng khoá (mỗi ngày, mỗi 100.000 id một digest) để biết lệch ở đâu, serialize `NULL`/kiểu phức tạp tường minh, và vẫn kèm tổng nghiệp vụ. MD5 ở đây để phát hiện sai lệch vô tình, không phải bằng chứng chống giả mạo.

Lớp thứ hai là **owner và quyền**:

```sql
SELECT tableowner FROM pg_tables WHERE schemaname = 'ops' AND tablename = 'trips';
SELECT has_table_privilege('tn_report', 'ops.trips', 'SELECT');
```

```text
PASS 046 restored owner: tn_owner
PASS 046 restored report ACL: t
```

Lớp thứ ba là một **lệnh ghi bị cấm**: runner chạy `SET ROLE tn_report; UPDATE ops.trips SET fare = 0;` trên `p05_restored` và yêu cầu nó thất bại với SQLSTATE `42501` (thiếu quyền). Assertion này nằm trong `run.mjs`; `OUTPUT.txt` không in lại thông báo lỗi. Kiểm quyền dương («report đọc được») mà không kiểm quyền âm («report không sửa được») thì chưa biết bản restore có vô tình rộng quyền hơn nguồn không.

Ba lớp đó là định nghĩa «restore đạt» trong bài: **dữ liệu khớp, chủ sở hữu khớp, ma trận quyền khớp cả chiều được phép lẫn chiều bị cấm**.

## 8. Cùng một archive, restore kiểu «cho đỡ lỗi»

Quay lại lần diễn tập trước của nhóm. Khi restore sang môi trường thiếu role, `pg_restore` lỗi ở các lệnh `ALTER … OWNER TO` và `GRANT`. Cách «sửa» nhanh nhất là thêm `--no-owner --no-acl`. Lab tái hiện đúng việc đó với **cùng file dump**:

```text
CREATE DATABASE p05_restored_bare;
pg_restore … -d p05_restored_bare --exit-on-error --no-owner --no-acl <ROOT>/ops.dump
```

```text
PASS 046 bare restore digest: 100/3000000/1d3675e3781bc03c33795a02bb3323e7
PASS 046 bare restore owner: postgres
PASS 046 bare restore report ACL: f
```

Digest **giống hệt** — nếu chỉ đối soát dữ liệu, bản này «đạt». Nhưng:

- `--no-owner` bỏ các lệnh đặt owner, nên mọi đối tượng thuộc về **người chạy restore** — ở đây là superuser `postgres`. Migration chạy bằng `tn_owner` sẽ không `ALTER` được bảng của chính nó nữa.
- `--no-acl` bỏ mọi `GRANT`/`REVOKE`, kể cả default privileges. `tn_report` mất quyền đọc; `tn_app` cũng vậy. Ứng dụng lên lại sẽ nhận toàn lỗi `42501`.

Hai cờ này hợp lý khi dựng bản sao cho dev với mô hình role khác. Khi khôi phục production, chúng biến một lỗi hiển nhiên («role không tồn tại») thành một lỗi im lặng («restore xong, không ai dùng được» — hoặc tệ hơn, ai đó cấp `ALL` cho xong). Bài học: **đối soát quyền phải là một phần của nghiệm thu**, nếu không lab đã chứng minh rằng digest không bắt được loại sai này.

## 9. Sang một cluster khác (chưa chạy trong lab)

Lab chỉ restore trong cùng cluster. Sang cluster mới thì thứ tự là: globals trước, database sau. Dạng lệnh để thử, **chưa chạy trong lab này**, không có output đi kèm:

```text
# 1. Role và tablespace của cluster nguồn
psql -h <DICH> -U postgres -d postgres -f globals.sql
# 2. Đặt lại password cho role LOGIN từ kho bí mật (không có trong globals.sql)
# 3. Database trống rồi restore
createdb -h <DICH> -U postgres toinoi_restore
pg_restore -h <DICH> -U postgres -d toinoi_restore --exit-on-error toinoi.dump
```

Những điểm phải kiểm thêm khi làm thật:

- **Role mapping khác**: nếu cluster đích đặt tên role khác (ví dụ `prod_owner` thay vì `tn_owner`), không có cờ nào tự đổi tên. Cách thường dùng là tạo role cùng tên ở đích, hoặc restore với `--no-owner` rồi chạy một script `ALTER … OWNER TO` và `GRANT` *đã review* — và đối soát quyền giống mục 7.
- **Extension**: extension phải có sẵn trên máy đích (đúng phiên bản file cài đặt). Dump toàn database có `CREATE EXTENSION`; dump theo schema như lab thì không.
- **Phiên bản**: dùng công cụ của phiên bản đích hoặc mới hơn; archive do `pg_dump` mới tạo có thể không đọc được bằng `pg_restore` cũ.
- **Song song**: `pg_dump -Fd -j 4` dump nhiều bảng cùng lúc, `pg_restore -j 4` restore song song từ custom hoặc directory format. Lab không chạy và không đo hai cách này.

## 10. Quay lại 22:40: bản dump cứu được gì

Áp vào sự cố:

1. **Không đụng vào nguồn.** Chặn script, chặn ghi nếu cần, giữ nguyên hiện trạng để điều tra.
2. Restore `ops-0200.dump` (và globals nếu sang cluster khác) vào **database riêng**, `--exit-on-error`, đối soát count/tổng/digest so với số liệu ghi lúc dump, kiểm owner/ACL.
3. Biết chính xác bản này có gì: trạng thái lúc **02:00 bắt đầu dump**. Mọi chuyến từ 02:00 đến 22:40 không có ở đây.

Từ đó có hai lựa chọn, và cả hai đều không hoàn hảo. Thay database bằng bản restore thì mất gần 21 giờ dữ liệu — RPO (lượng dữ liệu chấp nhận mất) của backup logic chính là khoảng cách tới lần dump gần nhất. Hoặc giữ database hiện tại, chép từ bản restore sang những dòng đã bị xoá mà có trước 02:00, và chấp nhận rằng chuyến trong ngày vẫn mất. Không có cách nào chỉ bằng `pg_dump` để lấy lại trạng thái lúc 22:39. Đó là lý do cần base backup và WAL — PG-048.

## 11. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Restore sang máy mới lỗi «role does not exist» ở `OWNER TO`/`GRANT` | Chỉ dump database, quên globals | `pg_dumpall --globals-only`, restore globals trước |
| Restore «thành công», ứng dụng toàn `42501` | Dùng `--no-owner --no-acl` để né lỗi role | Tạo đúng role, restore lại có owner/ACL; đối soát quyền |
| Exit code 0 nhưng thiếu bảng/quyền | Không dùng `--exit-on-error`, lỗi bị bỏ qua và chỉ đếm ở cuối | Bật `--exit-on-error`; đọc toàn bộ log restore |
| Restore vào database rỗng lỗi thiếu hàm/kiểu/extension | Dump `-n` không kéo theo phụ thuộc ngoài schema | Dump toàn database, hoặc liệt kê và chuẩn bị phụ thuộc trước |
| Hai bên cùng `count(*)` mà vẫn lệch | Chỉ đếm dòng | Thêm tổng nghiệp vụ và digest có thứ tự |
| Migration ban đêm treo trong lúc backup | `pg_dump` giữ `ACCESS SHARE` trên các bảng đang dump | Lệch lịch, hoặc migration dùng `lock_timeout` |
| Restore đè lên production rồi mới thấy dump sai | Không có database đích riêng | Luôn restore vào database/cluster riêng, đối soát rồi mới quyết định |

## 12. Đánh đổi: backup logic hay backup vật lý

| Tiêu chí | Logic (`pg_dump`) | Vật lý (base backup + WAL, PG-048) |
|---|---|---|
| Đơn vị | Một database, schema hoặc bảng | Cả cluster |
| Thời điểm khôi phục | Đúng lúc bắt đầu dump | Bất kỳ điểm nào có WAL liên tục |
| RPO | Khoảng cách tới lần dump gần nhất | Phụ thuộc độ trễ archive WAL |
| Đổi phiên bản lớn / nền tảng | Được — đây là cách nâng cấp kinh điển | Không, phải cùng phiên bản lớn và kiến trúc |
| Restore một phần | Dễ: chọn mục trong TOC | Không: restore cả cluster rồi lấy phần cần |
| Thời gian restore với dữ liệu lớn | Chậm: phải nạp lại dữ liệu và dựng lại index | Nhanh hơn: chép file rồi replay WAL |
| Tác động lúc backup | Snapshot dài, khoá `ACCESS SHARE`, đọc toàn bộ bảng | I/O chép file; cần cấu hình archive |

Hai họ không thay thế nhau. Một chính sách hợp lý cho Tới Nơi: backup vật lý liên tục để có PITR; dump logic định kỳ (kèm globals) để có bản độc lập phiên bản, restore một phần được, và để thường xuyên chứng minh dữ liệu đọc được từ đầu tới cuối.

## 13. Bài tập

**Bài 1.** Thêm vào `ops.trips` một cột bắt buộc `city text NOT NULL DEFAULT 'HCM'` và một `CHECK (fare <= 5000000)`. Tạo dump mới, restore vào database riêng và chứng minh ràng buộc còn hiệu lực.

<details><summary><b>Lời giải phải chứng minh được</b></summary>
<p>(1) Exit code của <code>pg_restore --exit-on-error</code> là 0. (2) Digest ở nguồn và đích khớp — nhớ thêm <code>city</code> vào chuỗi nối, nếu không digest bỏ sót cột mới. (3) <code>pg_restore --list</code> của archive mới có mục cho constraint mới. (4) Ở đích, một <code>INSERT</code> có <code>fare = 6000000</code> bị từ chối với SQLSTATE vi phạm CHECK, và một <code>INSERT</code> có <code>city = NULL</code> bị từ chối vì <code>NOT NULL</code>. Nộp lệnh, SQLSTATE quan sát được và phiên bản công cụ; không chép thông báo lỗi từ chỗ khác.</p>
</details>

**Bài 2.** Đồng nghiệp đề xuất: «Restore đêm nào cũng lỗi role, cứ thêm <code>--no-owner --no-acl</code> cho xanh pipeline.» Viết phản hồi có bằng chứng.

<details><summary><b>Lời giải phải chứng minh được</b></summary>
<p>Dẫn lại ba dòng <code>PASS 046 bare restore …</code>: digest giống hệt nhưng owner thành <code>postgres</code> và <code>tn_report</code> mất <code>SELECT</code>. Chỉ ra rằng đối soát chỉ dữ liệu sẽ cho «đạt», nên pipeline phải kiểm cả owner, ACL dương và một lệnh ghi bị cấm. Đề xuất đúng là sửa nguyên nhân: restore globals trước, hoặc tạo role ở đích, rồi giữ owner/ACL.</p>
</details>

**Bài 3.** Thiết kế đối soát cho một bảng 50 triệu chuyến, có cột <code>NULL</code> được (<code>driver_id</code> khi chuyến bị huỷ trước khi có tài xế).

<details><summary><b>Lời giải phải chứng minh được</b></summary>
<p>Digest theo khoảng (ví dụ theo ngày <code>created_at</code>) để biết lệch ở khoảng nào; mỗi cột serialize tường minh, <code>NULL</code> được thay bằng một ký hiệu không thể xuất hiện trong dữ liệu (hoặc dùng <code>coalesce</code>/<code>format('%L', …)</code>); thứ tự trong mỗi khoảng theo khoá chính. Kèm tổng nghiệp vụ theo ngày và trạng thái. Chứng minh bằng cách cố ý sửa một <code>driver_id</code> từ <code>NULL</code> sang một giá trị ở đích và cho thấy digest của đúng ngày đó lệch. Ghi rõ đây là phát hiện sai lệch, không phải chữ ký chống giả mạo.</p>
</details>

## 14. Đọc tiếp

- [PostgreSQL 18 · SQL Dump](https://www.postgresql.org/docs/18/backup-dump.html): `pg_dump`, `pg_dumpall`, restore và các lưu ý về role, template.
- [PostgreSQL 18 · pg_dump](https://www.postgresql.org/docs/18/app-pgdump.html): định dạng, `-n`, `-j`, tính nhất quán, khoá.
- [PostgreSQL 18 · pg_restore](https://www.postgresql.org/docs/18/app-pgrestore.html): `--list`, `-L`, `--exit-on-error`, `--no-owner`, `--no-acl`, `-j`.
- [PostgreSQL 18 · pg_dumpall](https://www.postgresql.org/docs/18/app-pg-dumpall.html): `--globals-only`, `--no-role-passwords`.
- Lab: `examples/toi-noi-postgresql/p05/run.mjs` (khối PG-046), kết quả `p05/OUTPUT.txt`, giới hạn phép đo trong `p05/README.md`, cổng nghiệm thu trong `p05/RUNBOOK.md`.

Bài trước: [PG-045 · WAL, checkpoint và độ bền dữ liệu](/articles/postgresql-wal-checkpoint-durability). Bài tiếp theo: [PG-047 · Connection pool, giới hạn kết nối và timeout](/articles/postgresql-connection-pool-timeout). Muốn lấy lại dữ liệu tới 22:39 chứ không phải 02:00, xem [PG-048 · Khôi phục theo thời điểm với base backup và WAL](/articles/postgresql-pitr-base-backup-wal).
