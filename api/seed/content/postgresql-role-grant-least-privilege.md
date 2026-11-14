---
slug: postgresql-role-grant-least-privilege
title: "Role, GRANT và quyền tối thiểu: tách API, báo cáo và migration của Tới Nơi"
summary: "API, job báo cáo và migration của Tới Nơi cùng dùng tài khoản sở hữu bảng — một lỗi ở job báo cáo đủ xoá sạch dữ liệu chuyến. Tách owner, application, report và monitor; đọc ACL thật; kiểm cả quyền được phép lẫn bị cấm; và chỉ ra cái bẫy default privileges với bảng tạo mới."
category: postgresql
level: beginner
tags: [postgresql, pg-041, roles, grant, permissions, least-privilege, default-privileges, toi-noi]
language: vi
---

Sáng thứ Hai, bảng doanh thu của Tới Nơi trống trơn. Nhóm dữ liệu vừa sửa job báo cáo cuối tuần: một câu dọn bảng tạm viết thiếu tên schema, và vì job chạy bằng **cùng tài khoản** với API đặt chuyến — tài khoản đã tạo ra mọi bảng — câu `DELETE` rơi trúng bảng chuyến thật. PostgreSQL không ngăn gì cả: với nó, chủ bảng xoá dữ liệu của chính mình là hợp lệ.

Lỗi nằm ở chỗ ba chương trình rất khác nhau — API cần đọc và ghi chuyến, job báo cáo chỉ cần đọc, migration cần đổi cấu trúc bảng — dùng chung một danh tính có **mọi quyền**. Trước khi nói tới giám sát, VACUUM hay sao lưu ở các bài sau của Phần 5, Tới Nơi cần trả lời một câu: **mỗi chương trình được làm gì, trên đối tượng nào — và điều gì phải bị từ chối?**

Bài này dựng mô hình bốn vai, cấp quyền theo từng lớp, chứng minh bằng lab cả việc được phép lẫn việc bị cấm, rồi đi vào cái bẫy hay gặp nhất: bảng tạo **sau** khi cấp quyền.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-041 · Phần 5/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/postgresql-thuc-hanh-tranh-chap-nhan-chuyen">PG-040 · Thực hành: hai tài xế cùng nhận một chuyến</a> (bài cuối Phần 4). Lab: <code>examples/toi-noi-postgresql/p05</code> (PostgreSQL 18.1 thật, tệp <code>00-schema.sql</code>, phần «PG-041» trong <code>run.mjs</code>, kết quả <code>OUTPUT.txt</code>). Sau bài này, bạn tách được tài khoản ứng dụng, migration và báo cáo, đọc được ACL và chứng minh quyền bị cấm thật sự bị cấm.</p>
</div>

## 1. Dữ liệu và cách chạy

Lab Phần 5 chạy trên **PostgreSQL 18.1 thật**. Runner tạo một cluster tạm chỉ nghe localhost, chạy mọi phép kiểm, rồi dọn sạch kể cả khi có assertion thất bại. Cần Node 22+, binary PostgreSQL 18 và OpenSSL (OpenSSL dùng cho PG-042); đường dẫn cấu hình qua `PG_BIN`, `OPENSSL_BIN` như README của lab.

```bash
cd examples/toi-noi-postgresql
npm run p05
```

Dữ liệu là một **bản chiếu (projection) vận hành** thu nhỏ, độc lập với schema P01–P04 để các phép thử quản trị không đụng fixture cũ: bảng `ops.trips` có 100 chuyến hoàn thành, tenant 1 và tenant 2 mỗi bên 50 chuyến, mỗi chuyến 30.000 đ, tổng 3.000.000 đ. Dòng đầu `OUTPUT.txt`:

```text
P05 operations lab · PostgreSQL 18.1
Node v22.18.0 · win32/x64
Fixture: 100 trips, 2 tenants, total fare 3000000 VND.
```

Dòng `PASS …` là **cổng kiểm**: runner so giá trị thật với giá trị mong đợi, lệch thì dừng với mã thoát khác 0. Khối `--- 041 …` là **output psql thật**. Mọi kết quả trong bài chép từ đó; SQL nào lab không chạy, bài sẽ nói rõ.

## 2. Vài thuật ngữ trước khi cấp quyền

- **Role** (vai trò): đơn vị danh tính duy nhất của PostgreSQL. Không có khái niệm «user» và «group» tách biệt — `CREATE USER` chỉ là `CREATE ROLE … LOGIN`. Role có thuộc tính `LOGIN` thì dùng để đăng nhập; role `NOLOGIN` thường dùng làm **nhóm quyền**.
- **Owner** (chủ sở hữu): role tạo ra đối tượng (hoặc được chuyển quyền sở hữu). Owner luôn có mọi quyền trên đối tượng, và chỉ owner (hoặc thành viên của role owner, hoặc superuser) mới `ALTER`, `DROP` được nó. Quyền này **không cấp được bằng GRANT**.
- **Privilege** (đặc quyền): từng loại hành động trên từng loại đối tượng — `SELECT`, `INSERT`, `UPDATE`, `DELETE` trên bảng; `USAGE`, `CREATE` trên schema; `CONNECT` trên database…
- **ACL** (access control list — danh sách kiểm soát truy cập): nơi PostgreSQL lưu «ai có quyền gì, do ai cấp» cho từng đối tượng, ví dụ cột `pg_class.relacl` của bảng.
- **PUBLIC**: nhóm ngầm mà **mọi** role đều thuộc về. Cấp cho PUBLIC là cấp cho tất cả.
- **Superuser**: role bỏ qua mọi kiểm tra quyền. Ứng dụng không bao giờ cần nó.

[Tài liệu PostgreSQL 18 · Database Roles](https://www.postgresql.org/docs/18/user-manag.html) và [Privileges](https://www.postgresql.org/docs/18/ddl-priv.html) là hai trang nền cho cả bài.

## 3. Bốn vai, bốn bộ quyền

Tới Nơi có ít nhất bốn kiểu chương trình chạm vào database. Mỗi kiểu được một role riêng:

| Role | Thuộc tính | Trách nhiệm | Không được làm |
|---|---|---|---|
| `tn_owner` | `NOLOGIN` | Sở hữu schema `ops` và bảng; migration chạy dưới danh nghĩa này | Không đưa credential cho API hay job |
| `tn_app` | `LOGIN` | API đặt chuyến: đọc, thêm, sửa chuyến | `DELETE`, DDL, superuser |
| `tn_report` | `NOLOGIN` | Nhóm quyền cho job báo cáo: chỉ đọc | `INSERT`, `UPDATE`, `DELETE` |
| `tn_monitor` | `NOLOGIN` | Chẩn đoán: thành viên `pg_monitor` | Ghi dữ liệu nghiệp vụ |

Hai điểm thiết kế đáng nói:

- **Owner không đăng nhập.** `tn_owner` là `NOLOGIN`: không ai «đăng nhập làm owner» được. Migration chạy bằng một tài khoản có quyền chuyển sang `tn_owner` (`SET ROLE tn_owner`) chỉ trong lúc chạy. Mật khẩu của API bị lộ thì kẻ xấu vẫn không có quyền owner.
- **Report là nhóm, không phải tài khoản.** Mỗi job hay mỗi nhà phân tích có login riêng, rồi được thêm vào nhóm `tn_report`. Thu hồi một người không ảnh hưởng người khác; log ghi rõ ai làm gì.

`pg_monitor` là một **predefined role** (role định sẵn) cho phép đọc đầy đủ các view thống kê; PG-043 dùng nó. [Predefined Roles](https://www.postgresql.org/docs/18/predefined-roles.html).

## 4. Quyền theo từng lớp

Một câu `SELECT * FROM ops.trips` phải đi qua nhiều cổng, mỗi cổng một loại quyền:

| Lớp | Quyền cần | Mặc định trên PostgreSQL 18 |
|---|---|---|
| Kết nối | `pg_hba.conf` cho phép (PG-042) + `CONNECT` trên database | PUBLIC có `CONNECT` và `TEMPORARY` trên database mới |
| Schema | `USAGE` để «nhìn thấy» đối tượng bên trong; `CREATE` để tạo mới | Schema mới: chỉ owner. Schema `public`: PUBLIC chỉ còn `USAGE` (từ PG15) |
| Bảng | `SELECT`, `INSERT`, `UPDATE`, `DELETE`, `TRUNCATE`… | Chỉ owner |
| Cột | `SELECT (col)`, `UPDATE (col)` khi cần hẹp hơn bảng | — |
| Dòng | Row-level security (PG-049) | Tắt |

Hệ quả quan trọng: **`USAGE` trên schema không tự cấp `SELECT` trên bảng**, và ngược lại có `SELECT` trên bảng mà thiếu `USAGE` trên schema thì vẫn bị từ chối. Phải cấp đủ cả hai lớp.

Script lab (`p05/00-schema.sql`, trích, bỏ DDL bảng và fixture):

```sql
CREATE ROLE tn_owner NOLOGIN;
CREATE ROLE tn_app LOGIN;
CREATE ROLE tn_report NOLOGIN;
CREATE ROLE tn_monitor NOLOGIN;
GRANT pg_monitor TO tn_monitor;
CREATE SCHEMA ops AUTHORIZATION tn_owner;
SET ROLE tn_owner;
-- CREATE TABLE ops.trips (...) và INSERT 100 chuyến ở đây
GRANT USAGE ON SCHEMA ops TO tn_app, tn_report;
GRANT SELECT, INSERT, UPDATE ON ops.trips TO tn_app;
GRANT SELECT ON ops.trips TO tn_report;
ALTER DEFAULT PRIVILEGES IN SCHEMA ops GRANT SELECT ON TABLES TO tn_report;
RESET ROLE;
```

Script chạy bằng `postgres` nhưng tạo bảng và GRANT **sau** `SET ROLE tn_owner`, nên bảng thuộc `tn_owner` và người cấp (grantor) cũng là `tn_owner`. `tn_app` cố ý **không** có `DELETE`: Tới Nơi không xoá chuyến, chỉ đổi trạng thái — ma trận quyền phản ánh nghiệp vụ, không phải «đủ bộ CRUD». Dòng `ALTER DEFAULT PRIVILEGES` sẽ nói ở mục 7.

## 5. Chứng minh cả hai chiều

Một ma trận quyền chỉ đáng tin khi kiểm cả việc **được phép** lẫn việc **bị cấm**. Chỉ thử `SELECT` thành công thì không phát hiện được tài khoản cũng đang `DELETE` được.

Runner thử mỗi ca bằng **một tiến trình psql riêng** (để role không «sót» sang ca sau), đăng nhập bằng `postgres` rồi `SET ROLE` sang role cần thử; mọi kiểm tra quyền sau đó tính theo role mới. psql chạy với `VERBOSITY=verbose` nên lỗi kèm SQLSTATE.

Ca được phép — report đọc được:

```sql
SET ROLE tn_report; SELECT count(*) FROM ops.trips;
```

```text
PASS 041 report reads: 100
```

Ca bị cấm — report xoá, app đổi cấu trúc bảng:

```sql
SET ROLE tn_report; DELETE FROM ops.trips;
SET ROLE tn_app;    ALTER TABLE ops.trips ADD COLUMN bad int;
```

```text
--- 041 tn_report DELETE
  ERROR:  42501: permission denied for table trips
--- 041 tn_app ALTER TABLE
  ERROR:  42501: must be owner of table trips
```

Cùng SQLSTATE `42501` (`insufficient_privilege`) nhưng hai thông báo khác nhau, và sự khác biệt đó có lý do:

- `permission denied for table trips`: `DELETE` là một privilege có thể GRANT; `tn_report` đơn giản là không được cấp.
- `must be owner of table trips`: `ALTER TABLE` không phải privilege cấp được — nó gắn với **quyền sở hữu**. Muốn app đổi được cấu trúc bảng thì phải biến app thành owner (hoặc thành viên của owner), tức là trao luôn quyền `DROP`. Đó chính là lý do tách migration khỏi runtime.

Runner cũng khẳng định owner của bảng không phải tài khoản ứng dụng:

```text
PASS 041 application is not owner: tn_owner
```

Ứng dụng nên bắt `42501` theo SQLSTATE, không so chuỗi thông báo (chuỗi đổi theo phiên bản và `lc_messages`). Lưu ý phạm vi: `SET ROLE` từ superuser kiểm **ACL**, không kiểm **xác thực** — `tn_app` lúc này chưa có mật khẩu. Đăng nhập thật qua TLS là việc của PG-042.

## 6. Đọc ACL như đọc hợp đồng

Lỗi `42501` cho biết một việc bị cấm; muốn biết **tất cả** những gì đang được phép thì đọc ACL. Danh sách role của lab:

```text
--- 041 roles
    rolname   | login | super | bypassrls
  ------------+-------+-------+-----------
   tn_app     | t     | f     | f
   tn_monitor | f     | f     | f
   tn_owner   | f     | f     | f
   tn_report  | f     | f     | f
  (4 rows)
```

Chỉ `tn_app` đăng nhập được; không role nào là superuser hay có `BYPASSRLS` (bỏ qua row-level security — PG-049).

ACL của hai bảng trong schema `ops`:

```text
--- 041 table ACL
     relname   |                                relacl
  -------------+-----------------------------------------------------------------------
   next_report | {tn_owner=arwdDxtm/tn_owner,tn_report=r/tn_owner}
   trips       | {tn_owner=arwdDxtm/tn_owner,tn_app=arw/tn_owner,tn_report=r/tn_owner}
  (2 rows)
```

Mỗi phần tử có dạng `người_nhận=quyền/người_cấp`. Các chữ cái theo [bảng viết tắt ACL](https://www.postgresql.org/docs/18/ddl-priv.html#PRIVILEGE-ABBREVS-TABLE):

| Chữ | Quyền | Chữ | Quyền |
|---|---|---|---|
| `r` | SELECT («read») | `D` | TRUNCATE |
| `a` | INSERT («append») | `x` | REFERENCES |
| `w` | UPDATE («write») | `t` | TRIGGER |
| `d` | DELETE | `m` | MAINTAIN (từ PG17: VACUUM, ANALYZE, REINDEX…) |

Đọc dòng `trips`: owner có đủ `arwdDxtm`; `tn_app=arw` — INSERT, SELECT, UPDATE, **không có `d`**; `tn_report=r` — chỉ SELECT; cả hai đều do `tn_owner` cấp. Khớp đúng ma trận ở mục 3. Không có phần tử nào dạng `=r/…` (người nhận để trống), tức là PUBLIC không có quyền gì trên bảng.

Trong psql, `\dp ops.*` và `\ddp` hiển thị cùng thông tin (ACL bảng, default privileges).

## 7. Cái bẫy: bảng tạo sau khi cấp quyền

GRANT chỉ áp cho **bảng đã tồn tại** — kể cả `GRANT … ON ALL TABLES IN SCHEMA ops`, vốn chỉ là viết tắt cho các bảng đang có. Tuần sau migration tạo bảng `ops.trip_ratings`, job báo cáo sẽ nhận `permission denied` — trừ khi có **default privileges** (đặc quyền mặc định).

`ALTER DEFAULT PRIVILEGES` khai báo trước: «khi role X tạo bảng trong schema Y, tự cấp quyền Z cho role W». Hai giới hạn quan trọng theo [tài liệu](https://www.postgresql.org/docs/18/sql-alterdefaultprivileges.html):

1. Nó chỉ áp cho đối tượng **tạo sau** lệnh này, không sửa ACL của đối tượng cũ.
2. Nó gắn với **role tạo đối tượng**. Không ghi `FOR ROLE`, nó áp cho role hiện hành lúc chạy lệnh — ở lab là `tn_owner` (vì đang `SET ROLE tn_owner`). Bảng do role khác tạo, ví dụ `postgres`, không nhận quyền mặc định này.

```sql
SET ROLE tn_owner; CREATE TABLE ops.next_report(id int); RESET ROLE;
SELECT has_table_privilege('tn_report', 'ops.next_report', 'SELECT');
SELECT has_table_privilege('tn_app',    'ops.next_report', 'SELECT');
```

```text
PASS 041 future table default privileges: t
PASS 041 future table not granted to app: f
```

```text
--- 041 default ACL
   for_role | in_schema | type |       defaclacl
  ----------+-----------+------+------------------------
   tn_owner | ops       | r    | {tn_report=r/tn_owner}
  (1 row)
```

Default ACL đọc là: khi **`tn_owner`** tạo đối tượng loại `r` (relation — bảng, view…) trong `ops`, cấp `r` (SELECT) cho `tn_report`. Đúng như vậy, `next_report` trong ACL ở mục 6 có `tn_report=r/tn_owner` dù không ai chạy GRANT trên nó.

Còn `tn_app` **không** có gì trên bảng mới, vì default privileges chỉ khai báo cho report. Đúng như thiết kế, nhưng đây là bất ngờ quen thuộc trên production: migration tạo bảng, deploy API, request đầu tiên nhận `permission denied`. Hai cách chữa:

- **Migration tự GRANT** cho app ngay trong cùng script tạo bảng. Quyền của app được viết tường minh cạnh DDL, review được trong pull request. Cái giá: phải nhớ, và quên thì lỗi chỉ lộ khi chạy.
- **Default privileges cho app** (`… GRANT SELECT, INSERT, UPDATE ON TABLES TO tn_app`). Tiện, nhưng app tự động có quyền ghi trên **mọi** bảng tương lai trong schema, kể cả bảng nó không nên chạm tới. Điều này đi ngược nguyên tắc quyền tối thiểu.

Tới Nơi chọn cách đầu cho quyền ghi. Dù chọn cách nào, kiểm thử sau migration phải chạy bằng role ứng dụng, cả ca được phép lẫn ca bị cấm, không chỉ bằng owner.

Một biến thể nguy hiểm hơn mà lab **không chạy**: nếu migration lỡ chạy bằng `postgres` mà quên `SET ROLE tn_owner`, bảng mới thuộc `postgres`, report không nhận quyền mặc định, và các migration sau chạy bằng `tn_owner` sẽ nhận `must be owner` khi muốn `ALTER` nó. Truy vấn để tự kiểm sau mỗi migration (để bạn thử):

```sql
SELECT schemaname, tablename, tableowner
FROM pg_tables
WHERE schemaname = 'ops' AND tableowner <> 'tn_owner';
```

Kết quả đúng phải là 0 dòng.

## 8. Schema `public` và nhóm PUBLIC

Database nào cũng có sẵn schema `public`. Trước PostgreSQL 15, PUBLIC có `CREATE` trên schema này: mọi role, kể cả tài khoản ứng dụng, tạo được bảng và function trong `public` — kẻ chiếm được API có thể tạo function trùng tên và chờ ai đó gọi nhầm.

Từ PostgreSQL 15, PUBLIC chỉ còn `USAGE` trên `public`, và schema này thuộc role `pg_database_owner` (tức owner của database). Lab xác nhận trên PG 18.1:

```sql
SELECT has_schema_privilege('tn_app', 'public', 'CREATE');
```

```text
PASS 041 app cannot CREATE in schema public: f
```

Hai lưu ý:

- **Database nâng cấp từ bản cũ giữ nguyên ACL cũ.** Dump/restore hoặc `pg_upgrade` từ PG14 trở về trước mang theo quyền `CREATE` của PUBLIC; lab chỉ chứng minh mặc định của cluster `initdb` PG18 mới. Nếu database thật ra `t`, sửa bằng `REVOKE CREATE ON SCHEMA public FROM PUBLIC;` (để bạn thử, lab không chạy).
- **PUBLIC vẫn còn nhiều quyền mặc định khác**: `CONNECT` và `TEMPORARY` trên database, `EXECUTE` trên function mới tạo, `USAGE` trên schema `public`. Database chỉ dành cho vài role thì `REVOKE CONNECT ON DATABASE … FROM PUBLIC` rồi cấp lại từng role (lab không đổi các quyền này).

## 9. Membership: kế thừa và chuyển role

Thêm login `bao_cao_ngay` vào nhóm báo cáo:

```sql
CREATE ROLE bao_cao_ngay LOGIN;
GRANT tn_report TO bao_cao_ngay;
```

(SQL để bạn thử, lab không tạo role này.) Thành viên dùng quyền của nhóm theo hai cách:

- **Kế thừa (INHERIT)**: quyền của `tn_report` tự có hiệu lực khi `bao_cao_ngay` truy vấn, không cần làm gì thêm. Đây là mặc định cho role tạo bằng `CREATE ROLE` (bảng `pg_dumpall` của PG-046 cũng cho thấy bốn role lab đều `INHERIT`).
- **Chuyển role (SET ROLE)**: thành viên đổi hẳn role hiện hành sang `tn_report`. Đối tượng tạo sau khi `SET ROLE` sẽ thuộc role đó.

Từ PostgreSQL 16, hai khả năng này tách được trên **từng lần cấp thành viên**: `GRANT tn_owner TO migrator WITH INHERIT FALSE, SET TRUE` cho phép tài khoản migration chuyển sang owner khi chạy migration, nhưng bình thường không mang theo quyền owner. Xem [Role Membership](https://www.postgresql.org/docs/18/role-membership.html) và [GRANT](https://www.postgresql.org/docs/18/sql-grant.html).

Mặt trái: một login không có gì trong ACL vẫn có thể là thành viên của `tn_owner`. Rà quyền phải gồm cả `pg_auth_members`, hoặc hỏi thẳng bằng `has_table_privilege(...)` — hàm này đã tính membership.

## 10. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Job báo cáo xoá được dữ liệu | Dùng chung tài khoản owner với API | Tách role; owner `NOLOGIN` |
| `permission denied for table …` sau migration | Bảng mới không được GRANT; default privileges không bao gồm role này | GRANT trong migration, hoặc default privileges có chủ đích |
| Default privileges «không có tác dụng» | Bảng do role khác (vd. `postgres`) tạo, không phải role ghi trong `for_role` | Chạy migration bằng `SET ROLE tn_owner` hoặc khai báo `FOR ROLE` đúng |
| Có `SELECT` trên bảng mà vẫn bị từ chối | Thiếu `USAGE` trên schema | `GRANT USAGE ON SCHEMA …` |
| `must be owner of table …` từ API | Runtime chạy DDL | Chuyển DDL sang migration |
| Kiểm quyền chỉ bằng ca thành công | Không có bằng chứng cho ca bị cấm | Thêm assertion `42501` cho mỗi hành động cấm |
| Database nâng cấp vẫn cho tạo bảng trong `public` | ACL cũ trước PG15 được giữ | `REVOKE CREATE ON SCHEMA public FROM PUBLIC` |

## 11. Đánh đổi

- **Nhiều role hay ít role.** Mỗi role thêm vào là một dòng trong ma trận phải kiểm và rà định kỳ. Bốn vai ở đây là mức tối thiểu có ý nghĩa; tách thêm (vd. role riêng cho từng microservice) chỉ đáng khi các service thật sự cần quyền khác nhau.
- **Default privileges hay GRANT tường minh.** Default privileges tránh lỗi «quên GRANT» nhưng giấu quyền khỏi script migration; GRANT tường minh dễ review nhưng dễ quên. Tới Nơi dùng default privileges cho quyền đọc, GRANT tường minh cho quyền ghi.
- **Quyền cột.** `GRANT SELECT (id, fare) ON …` hẹp hơn quyền bảng, nhưng `SELECT *` của report sẽ lỗi và mỗi cột mới lại phải cấp. View thường dễ quản lý hơn (bài tập 1).
- **Bảo mật không thay thế được kiểm soát thay đổi.** Quyền tối thiểu giới hạn **thiệt hại** của một lỗi; nó không ngăn được migration tự tay xoá dữ liệu. Phục hồi khi thiệt hại đã xảy ra là việc của PG-046 và PG-048.

## 12. Bài tập

**Bài 1.** Một đối tác bảo hiểm cần thống kê số chuyến và tổng cước theo tenant, nhưng không được đọc từng chuyến. Tạo role `tn_partner` và một view tổng hợp; đối tác chỉ có quyền trên view.

<details><summary><b>Gợi ý và tiêu chí chấm</b></summary>
<pre><code class="language-sql">SET ROLE tn_owner;
CREATE VIEW ops.fare_by_tenant AS
  SELECT tenant_id, count(*) AS trips, sum(fare) AS total_fare
  FROM ops.trips GROUP BY tenant_id;
RESET ROLE;
CREATE ROLE tn_partner NOLOGIN;
GRANT USAGE ON SCHEMA ops TO tn_partner;
GRANT SELECT ON ops.fare_by_tenant TO tn_partner;</code></pre>
<p>SQL này không có trong lab; bạn tự chạy trên cluster lab hoặc database thử. View mặc định truy cập bảng gốc bằng quyền của <b>owner view</b> (<code>tn_owner</code>), nên <code>tn_partner</code> đọc được view mà không cần quyền trên <code>ops.trips</code>. Lời giải đúng phải chứng minh ba điều: <code>SET ROLE tn_partner; SELECT * FROM ops.fare_by_tenant</code> chạy được và tổng khớp fixture (100 chuyến, 3.000.000 đ); <code>SELECT * FROM ops.trips</code> bằng <code>tn_partner</code> nhận <code>42501</code>; và ACL của view không có quyền nào ngoài <code>r</code> cho <code>tn_partner</code>. Lưu ý: vì view chạy bằng quyền owner, nó bỏ qua row-level security của bảng gốc (owner được miễn RLS mặc định) — PG-049 bàn về <code>security_invoker</code> và trường hợp này. Default privileges của lab cũng sẽ tự cấp SELECT trên view cho <code>tn_report</code>; ghi điều đó vào ACL bàn giao.</p>
</details>

**Bài 2.** Thiết lập tài khoản migration `tn_migrator` đăng nhập được, chuyển sang `tn_owner` được, nhưng không tự kế thừa quyền owner. Chứng minh bảng nó tạo sau `SET ROLE tn_owner` nhận default privileges, còn bảng tạo **không** qua `SET ROLE` thì bị từ chối.

<details><summary><b>Gợi ý và tiêu chí chấm</b></summary>
<pre><code class="language-sql">CREATE ROLE tn_migrator LOGIN;
GRANT tn_owner TO tn_migrator WITH INHERIT FALSE, SET TRUE;</code></pre>
<p>Cú pháp <code>WITH INHERIT/SET</code> có từ PostgreSQL 16; lab không chạy đoạn này. Phải <b>đăng nhập thật</b> bằng <code>tn_migrator</code> (đặt mật khẩu và thêm dòng <code>pg_hba.conf</code> trên database thử của bạn — HBA của lab từ chối mọi login ngoài <code>tn_app</code> và <code>postgres</code>, xem PG-042). Lý do: <code>SET ROLE</code> kiểm membership của <b>session user</b>, nên trong phiên đăng nhập bằng superuser thì <code>SET ROLE</code> sang role nào cũng được và bài thử không chứng minh gì. Lời giải đúng phải cho thấy, trong phiên đăng nhập bằng <code>tn_migrator</code>: (1) <code>CREATE TABLE ops.x(id int)</code> bị từ chối với <code>42501</code> (thiếu <code>CREATE</code> trên schema <code>ops</code>), vì không kế thừa quyền owner; (2) <code>SET ROLE tn_owner; CREATE TABLE ops.x(id int)</code> chạy được và owner của <code>ops.x</code> là <code>tn_owner</code>; (3) <code>has_table_privilege('tn_report','ops.x','SELECT')</code> là <code>t</code>.</p>
</details>

**Bài 3.** Viết một truy vấn rà quyền dùng được trong runbook: liệt kê mọi bảng trong `ops` mà `tn_app` có `DELETE` hoặc `TRUNCATE`, hoặc mà owner khác `tn_owner`.

<details><summary><b>Gợi ý và tiêu chí chấm</b></summary>
<pre><code class="language-sql">SELECT c.oid::regclass AS bang, pg_get_userbyid(c.relowner) AS owner,
       has_table_privilege('tn_app', c.oid, 'DELETE')   AS app_delete,
       has_table_privilege('tn_app', c.oid, 'TRUNCATE') AS app_truncate
FROM pg_class c
WHERE c.relnamespace = 'ops'::regnamespace AND c.relkind IN ('r','p','v')
  AND (has_table_privilege('tn_app', c.oid, 'DELETE')
    OR has_table_privilege('tn_app', c.oid, 'TRUNCATE')
    OR pg_get_userbyid(c.relowner) <> 'tn_owner');</code></pre>
<p>Lab không chạy truy vấn này. Trên trạng thái lab ở mục 7, kết quả đúng là 0 dòng (theo ACL đã quan sát: <code>tn_app=arw</code> trên <code>trips</code>, không có quyền trên <code>next_report</code>, cả hai thuộc <code>tn_owner</code>). Lời giải đạt yêu cầu khi bạn chứng minh được nó <b>bắt được</b> vi phạm: tạm <code>GRANT DELETE ON ops.trips TO tn_app</code> thì truy vấn ra 1 dòng, <code>REVOKE</code> thì về 0. Dùng <code>has_table_privilege</code> thay vì đọc chuỗi <code>relacl</code> vì hàm tính cả membership và PUBLIC.</p>
</details>

## 13. Đọc tiếp

- [PostgreSQL 18 · Privileges](https://www.postgresql.org/docs/18/ddl-priv.html): các loại quyền, viết tắt ACL, quyền mặc định của PUBLIC.
- [PostgreSQL 18 · Schemas — Usage Patterns](https://www.postgresql.org/docs/18/ddl-schemas.html#DDL-SCHEMAS-PATTERNS): schema `public` và mặc định từ PG15.
- [PostgreSQL 18 · GRANT](https://www.postgresql.org/docs/18/sql-grant.html), [REVOKE](https://www.postgresql.org/docs/18/sql-revoke.html), [ALTER DEFAULT PRIVILEGES](https://www.postgresql.org/docs/18/sql-alterdefaultprivileges.html).
- [PostgreSQL 18 · Role Membership](https://www.postgresql.org/docs/18/role-membership.html) và [Predefined Roles](https://www.postgresql.org/docs/18/predefined-roles.html).
- [PostgreSQL 18 · System Information Functions](https://www.postgresql.org/docs/18/functions-info.html): `has_table_privilege`, `has_schema_privilege`.
- Lab: `examples/toi-noi-postgresql/p05/00-schema.sql`, phần PG-041 trong `p05/run.mjs`, kết quả `p05/OUTPUT.txt`, bảng xử lý sự cố `p05/RUNBOOK.md`.

Bài tiếp theo: [PG-042 · Xác thực và bảo vệ kết nối PostgreSQL](/articles/postgresql-ket-noi-auth-tls) — `tn_app` có quyền đúng rồi; giờ phải chứng minh chỉ đúng `tn_app`, qua đúng đường, mới đăng nhập được.
